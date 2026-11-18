#!/bin/sh
# walleye-battery-selftest: proves battery-ok.sh and idle-android.sh against fake fastboot/adb programs (the phone is never touched).
# Prints one line, `pass|fail walleye-battery-selftest host DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d) || exit 1; trap 'rm -rf "$T"' EXIT
fail() { echo "fail walleye-battery-selftest host $1"; exit 1; }
mkfake() { # NAME BODY
  printf '#!/bin/sh\n%s\n' "$2" > "$T/$1"; chmod +x "$T/$1"; }
run() { PATH="$T:$PATH" sh "$HERE/$1" 2>&1; }
# fastboot at 3892 mV: pass; at 3600: fail; no tool answering: skip (exit 1 too, nothing measured)
mkfake fastboot 'case "$*" in *battery-voltage*) echo "battery-voltage: 3892mV" >&2;; esac'; mkfake adb 'exit 1'
out=$(run battery-ok.sh); [ $? -eq 0 ] || fail "3892 mV did not pass: $out"
case $out in pass*3892*) ;; *) fail "3892 mV line wrong: $out";; esac
mkfake fastboot 'case "$*" in *battery-voltage*) echo "battery-voltage: 3600mV" >&2;; esac'
out=$(run battery-ok.sh); [ $? -eq 1 ] || fail "3600 mV did not exit 1: $out"
case $out in fail*3600*) ;; *) fail "3600 mV line wrong: $out";; esac
# Android: dumpsys battery
mkfake fastboot 'exit 1'
mkfake adb 'case "$*" in *dumpsys*) printf "Current Battery Service state:\n  level: 80\n  voltage: 4012\n";; esac'
out=$(run battery-ok.sh); [ $? -eq 0 ] && case $out in pass*4012*) ;; *) false;; esac || fail "adb 4012 mV: $out"
mkfake adb 'exit 1'
out=$(run battery-ok.sh); [ $? -eq 2 ] && case $out in skip*not\ known*) ;; *) false;; esac || fail "no reading is not skip: $out"
# Android running, fastboot would hang waiting for a device: adb answers, no hang
mkfake fastboot 'exec sleep 60'
mkfake adb 'case "$*" in *dumpsys*) printf "  voltage: 4001\n";; esac'
s=$(date +%s); out=$(run battery-ok.sh); rc=$?; e=$(( $(date +%s) - s ))
[ $rc -eq 0 ] && [ $e -lt 15 ] && case $out in pass*4001*android*) ;; *) false;; esac || fail "adb with hanging fastboot: rc $rc, $e s: $out"
# neither answers (both hang): not measured, exit 2, within 15 s
mkfake adb 'exec sleep 60'
s=$(date +%s); out=$(run battery-ok.sh); rc=$?; e=$(( $(date +%s) - s ))
[ $rc -eq 2 ] && [ $e -lt 15 ] && case $out in skip*not\ known*) ;; *) false;; esac || fail "both hanging: rc $rc, $e s: $out"
# idle-android.sh: Android booted -> pass; fastboot -> fail
mkfake adb 'case "$*" in *get-state*) echo device;; *boot_completed*) echo 1;; *) :;; esac'
mkfake fastboot 'exit 0'
out=$(run idle-android.sh); [ $? -eq 0 ] || fail "Android booted did not pass: $out"
mkfake adb 'exit 1'; mkfake fastboot 'echo FA81N1A00790	fastboot'
out=$(run idle-android.sh); [ $? -eq 1 ] && case $out in fail*fastboot*) ;; *) false;; esac || fail "fastboot not failed: $out"
echo "pass walleye-battery-selftest host battery-ok and idle-android pass and fail where they should"
