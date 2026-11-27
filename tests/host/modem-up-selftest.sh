#!/bin/sh
# Proves tests/health/base/modem-up.sh on text our phone printed (tests/fixtures/phone-text/modem-read-20261127-104225.cap: the first modem boot's read, research walleye-modem-mss-first-boot), never touching the phone:
# the kept read passes both tests; the same read without the 4080000.remoteproc line fails the remoteproc test; without the qrtr services, or without service 2 or 21, it fails the services test; no read skips.
# The suite's words (lib.sh) are lab-tools' own, read from its origin/main (a missing lib is an error, not a skip).
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
F="$HERE/../fixtures/phone-text/modem-read-20261127-104225.cap"
CHECK="$HERE/../health/base/modem-up.sh"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
git -C "${LAB_TOOLS:-$HOME/Projects/linux-phone-oco/lab-tools}" show origin/main:health/suite/lib.sh > "$T/lib.sh"
total=0
want() { # NAME TEST EXPECTED_WORD READ_FILE
  got=$(LPOCO_HEALTH_LIB="$T/lib.sh" LPOCO_MODEM_READ="$4" sh "$CHECK" | awk -v t="$2" '$2 == t { print $1 }')
  if [ "$got" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$got', want $3"; total=$((total+1)); fi
}
grep -v '^#' "$F" > "$T/kept.txt"
grep -v '^4080000.remoteproc ' "$T/kept.txt" > "$T/no-mss.txt"
sed '/^---qrtr$/,/^---end$/{/^---/!d}' "$T/kept.txt" > "$T/no-qrtr.txt"
grep -v " Device Management Service$" "$T/kept.txt" > "$T/no-dms.txt"
grep -v " Modem embedded file system service$" "$T/kept.txt" > "$T/no-efs.txt"
: > "$T/empty.txt"
want "the kept read: remoteproc running"                  modem-remoteproc-running pass "$T/kept.txt"
want "the kept read: the required qrtr services listed"   modem-qrtr-services      pass "$T/kept.txt"
want "a stand-in without the 4080000.remoteproc line"     modem-remoteproc-running fail "$T/no-mss.txt"
want "a read with no qrtr services"                       modem-qrtr-services      fail "$T/no-qrtr.txt"
want "a read without service 2 (device management)"       modem-qrtr-services      fail "$T/no-dms.txt"
want "a read without service 21 (embedded file system)"    modem-qrtr-services      fail "$T/no-efs.txt"
want "an empty read skips remoteproc"                     modem-remoteproc-running skip "$T/empty.txt"
want "an empty read skips services"                       modem-qrtr-services      skip "$T/empty.txt"
detail=$(LPOCO_HEALTH_LIB="$T/lib.sh" LPOCO_MODEM_READ="$T/kept.txt" sh "$CHECK" | grep ' modem-qrtr-services ')
case "$detail" in *"35 services"*"NAS 3 present"*"WDS 1 present"*"voice 9 present"*) echo "ok   the detail counts the 35 node-0 services and prints NAS, WDS, voice" ;; *) echo "FAIL detail: $detail"; total=$((total+1)) ;; esac
[ $total -eq 0 ] || { echo "modem-up-selftest: $total failed" >&2; exit 1; }
echo "pass modem-up-selftest"
