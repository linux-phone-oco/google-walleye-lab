#!/bin/sh
# walleye-nfc-up, run on the lab host (item walleye-nfc-nfc0-opener-in-ramdisk): the NFC ramdisk holds a tool that can open nfc0.
# (1) the tool's tests pass (a stand-in kernel, and this host's real kernel answering the controller's question);
# (2) the file device.toml's [images.walleye-nfc-up] names has its recorded sha256 and is a static aarch64 ELF (`file`).
# That the phone's kernel answers it is NOT shown here: that is the nfc boot (item walleye-nfc-boot-run-2).
# Prints `pass|fail|skip walleye-nfc-up DETAIL`; exit 0 only on pass. Read-only. Environment: none.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-nfc-up $2"; [ "$1" = pass ]; exit $?; }
cargo test --locked --quiet --manifest-path "$HERE/tools/walleye-nfc-up/Cargo.toml" >/dev/null 2>&1 || say fail "the tool's tests fail"
command -v file >/dev/null || say skip "file not installed: not known yet whether the tool is a static aarch64 ELF"
P=$(awk '/^\[images.walleye-nfc-up\]/{f=1;next} /^\[/{f=0} f&&$1=="path"{gsub(/"/,"",$3);print $3}' "$HERE/device.toml")
S=$(awk '/^\[images.walleye-nfc-up\]/{f=1;next} /^\[/{f=0} f&&$1=="sha256"{gsub(/"/,"",$3);print $3}' "$HERE/device.toml")
D=$(awk '/^\[images\]/{f=1;next} /^\[/{f=0} f&&$1=="dir"{gsub(/"/,"",$3);print $3}' "$HERE/device.toml")
[ -n "$P" ] && [ -n "$S" ] || say fail "device.toml has no [images.walleye-nfc-up] path and sha256"
F=$HERE/$D/$P
[ -r "$F" ] || say skip "$F is not readable (build/images link missing?): not known yet"
[ "$(sha256sum "$F" | cut -d' ' -f1)" = "$S" ] || say fail "$F is not the sha256 device.toml records"
case $(file -b "$F") in
  *"ARM aarch64"*static*) ;;
  *) say fail "$F is not a static aarch64 ELF: $(file -b "$F")" ;;
esac
say pass "the tool's tests pass and $F is the declared static aarch64 file; that the phone's kernel answers it is not measured here"
