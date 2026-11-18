#!/bin/sh
# walleye-idle-android, run on the lab host when a keeper session ends: is the idle phone booted into Android, not parked in fastboot?
# (fastboot mode draws power and charges little: the phone went dark once.) Passes only when adb sees the phone as `device` and
# sys.boot_completed is 1, and fastboot does not list it. Prints one line, `pass|fail walleye-idle-android host DETAIL` - the
# end state to record. Read-only.
#
#   tests/host/idle-android.sh [DEVICE_TOML]      (default: ../../device.toml)
HERE=$(cd "$(dirname "$0")" && pwd)
TOML=${1:-$HERE/../../device.toml}
say() { echo "$1 walleye-idle-android host $2"; [ "$1" = pass ]; exit $?; }
SER=$(sed -n 's/^serial *= *"\([^"]*\)".*/\1/p' "$TOML" 2>/dev/null | head -n1)
if [ -n "$SER" ] && fastboot devices 2>/dev/null | grep -q "$SER"; then
  say fail "the phone is in fastboot: fastboot reboot, then wait for Android"
fi
st=$(adb ${SER:+-s "$SER"} get-state 2>/dev/null)
[ "$st" = device ] || say fail "adb state '${st:-none}', not device: Android is not up"
bc=$(adb ${SER:+-s "$SER"} shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')
[ "$bc" = 1 ] || say fail "adb sees the phone but sys.boot_completed is '${bc:-empty}'"
say pass "Android booted, adb device, not in fastboot"
