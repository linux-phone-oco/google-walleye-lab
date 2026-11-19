#!/bin/sh
# walleye-battery-ok, run on the lab host before every try: is the phone's battery good enough to try an image?
# Reads the battery voltage - `adb shell dumpsys battery` when Android runs, else `fastboot getvar battery-voltage` when the phone is in fastboot -
# each call bounded by `timeout -k 1 6` - prints it, and exits 1 below the manifest's [health] battery_floor_mv. No reading in 15 s is `skip` (not known yet), exit 2:
# the keeper stops and records, it does not try blind. Prints one line, `pass|fail|skip walleye-battery-ok host DETAIL`.
# Read-only; changes nothing on the phone.
#
#   tests/host/battery-ok.sh [DEVICE_TOML]      (default: ../../device.toml)
# The floor is [health] battery_floor_mv and the programs are [tools] adb and fastboot of the manifest; a missing floor is `skip` (not known),
# never a number made up here.
HERE=$(cd "$(dirname "$0")" && pwd)
TOML=${1:-$HERE/../../device.toml}
TRY=6   # plus 1 s of -k: 7 s a call, 14 s for two
say() { echo "$1 walleye-battery-ok host $2"; case $1 in pass) exit 0;; skip) exit 2;; *) exit 1;; esac; }
command -v timeout >/dev/null || say skip "timeout(1) is missing, so no call can be bounded: not known yet"
SER=$(sed -n 's/^serial *= *"\([^"]*\)".*/\1/p' "$TOML" 2>/dev/null | head -n1)
# a key of a table of the manifest: key TABLE NAME
key() { awk -v t="[$1]" -v k="$2" '/^\[/ { in_t = ($0 == t) } in_t && $1 == k && $2 == "=" { v = $0; sub(/^[^=]*= */, "", v); sub(/ *#.*$/, "", v); gsub(/"/, "", v); print v; exit }' "$TOML" 2>/dev/null; }
MIN=$(key health battery_floor_mv); ADB=$(key tools adb); FASTBOOT=$(key tools fastboot)
case $MIN in ""|*[!0-9]*) say skip "no whole-number [health] battery_floor_mv in $TOML: the floor is not known";; esac
[ -n "$ADB" ] && [ -n "$FASTBOOT" ] || say skip "no [tools] adb or fastboot in $TOML: not known"
# Every call is bounded (TRY seconds each, two calls: under 15 s in all); fastboot waits for a device forever when Android runs.
mv=; src=
if out=$(timeout -k 1 "$TRY" "$ADB" ${SER:+-s "$SER"} shell dumpsys battery 2>/dev/null); then
  mv=$(echo "$out" | tr -d '\r' | sed -n 's/^ *voltage: *\([0-9][0-9]*\).*/\1/p' | head -n1); src=android
fi
if [ -z "$mv" ] && out=$(timeout -k 1 "$TRY" "$FASTBOOT" ${SER:+-s "$SER"} getvar battery-voltage 2>&1); then
  mv=$(echo "$out" | sed -n 's/.*battery-voltage: *\([0-9][0-9]*\).*/\1/p' | head -n1); src=fastboot
fi
[ -n "$mv" ] || say skip "no battery voltage from adb or fastboot within 7 s each: not known yet"
[ "$mv" -ge "$MIN" ] || say fail "$mv mV from $src, below $MIN mV: charge in Android, do not try; record it"
say pass "$mv mV from $src (minimum $MIN)"
