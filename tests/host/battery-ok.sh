#!/bin/sh
# walleye-battery-ok, run on the lab host before every try: is the phone's battery good enough to try an image?
# Reads the battery voltage - `fastboot getvar battery-voltage` when the phone is in fastboot, else `adb shell dumpsys battery`
# when Android runs - prints it, and exits 1 below BATTERY_MIN_MV (default 3700). No reading is `skip` (not known yet), exit 1 too:
# the keeper stops and records, it does not try blind. Prints one line, `pass|fail|skip walleye-battery-ok host DETAIL`.
# Read-only; changes nothing on the phone.
#
#   tests/host/battery-ok.sh [DEVICE_TOML]      (default: ../../device.toml)
# Environment: BATTERY_MIN_MV the lowest passing voltage in mV (default 3700).
HERE=$(cd "$(dirname "$0")" && pwd)
TOML=${1:-$HERE/../../device.toml}
MIN=${BATTERY_MIN_MV:-3700}
say() { echo "$1 walleye-battery-ok host $2"; [ "$1" = pass ]; exit $?; }
SER=$(sed -n 's/^serial *= *"\([^"]*\)".*/\1/p' "$TOML" 2>/dev/null | head -n1)
mv=; src=
if out=$(fastboot ${SER:+-s "$SER"} getvar battery-voltage 2>&1); then
  mv=$(echo "$out" | sed -n 's/.*battery-voltage: *\([0-9][0-9]*\).*/\1/p' | head -n1); src=fastboot
fi
if [ -z "$mv" ]; then
  out=$(adb ${SER:+-s "$SER"} shell dumpsys battery 2>/dev/null) &&
    mv=$(echo "$out" | tr -d '\r' | sed -n 's/^ *voltage: *\([0-9][0-9]*\).*/\1/p' | head -n1) && src=android
fi
[ -n "$mv" ] || say skip "no battery voltage from fastboot or adb: not known yet"
[ "$mv" -ge "$MIN" ] || say fail "$mv mV from $src, below $MIN mV: charge in Android, do not try; record it"
say pass "$mv mV from $src (minimum $MIN)"
