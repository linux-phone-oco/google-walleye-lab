#!/bin/sh
# walleye-reboot-restart2, run on the lab host (item walleye-reboot-reason-restart2-tool): the stage 1 way back can hand `bootloader` to reboot(2) RESTART2.
# (1) the tool's tests pass: the string given is the string passed to reboot(2) (a stand-in syscall), only `bootloader` is allowed, an unknown flag stops the run;
# (2) the file device.toml's [images.walleye-reboot] names has its recorded sha256 and is a static aarch64 ELF (`file`);
# (3) the way back's tests pass: the tool is sent whole into the phone's /tmp and run from there, never sent when its sha256 is not the declared one, nothing is written to the card.
# Whether the bootloader then stays in fastboot is NOT shown here: that is the keeper's stage 1 run (item walleye-reboot-bootloader-from-linux-try).
# Prints `pass|fail|skip walleye-reboot-restart2 kernel DETAIL`; exit 0 only on pass. Read-only. Environment: none.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-reboot-restart2 kernel $2"; [ "$1" = pass ]; exit $?; }
cargo test --locked --quiet --manifest-path "$HERE/tools/walleye-reboot/Cargo.toml" >/dev/null 2>&1 || say fail "the tool's tests fail (the string given is not the string passed to reboot(2))"
command -v file >/dev/null || say skip "file not installed: not known yet whether the tool is a static aarch64 ELF"
P=$(awk '/^\[images.walleye-reboot\]/{f=1;next} /^\[/{f=0} f&&$1=="path"{gsub(/"/,"",$3);print $3}' "$HERE/device.toml")
S=$(awk '/^\[images.walleye-reboot\]/{f=1;next} /^\[/{f=0} f&&$1=="sha256"{gsub(/"/,"",$3);print $3}' "$HERE/device.toml")
D=$(awk '/^\[images\]/{f=1;next} /^\[/{f=0} f&&$1=="dir"{gsub(/"/,"",$3);print $3}' "$HERE/device.toml")
[ -n "$P" ] && [ -n "$S" ] || say fail "device.toml has no [images.walleye-reboot] path and sha256"
F=$HERE/$D/$P
[ -r "$F" ] || say skip "$F is not readable (build/images link missing?): not known yet"
[ "$(sha256sum "$F" | cut -d' ' -f1)" = "$S" ] || say fail "$F is not the sha256 device.toml records"
case $(file -b "$F") in
  *"ARM aarch64"*static*) ;;
  *) say fail "$F is not a static aarch64 ELF: $(file -b "$F")" ;;
esac
cargo test --locked --quiet --manifest-path "$HERE/procedures/steps/Cargo.toml" bootloader_way_back >/dev/null 2>&1 || say fail "the way back's tests fail (the tool is not sent to /tmp and run with bootloader)"
say pass "the tool's tests pass, $F is the declared static aarch64 file, and the way back sends it to /tmp and runs it with bootloader; that the bootloader stops in fastboot is not measured here"
