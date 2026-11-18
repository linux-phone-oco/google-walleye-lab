#!/bin/sh
# Proves tests/host/touch-check.sh against made-up reader output, never touching the phone.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME EXPECTED_WORD DIR
  got=$(sh "$HERE/touch-check.sh" "$3" | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
ev() { printf '00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 %s 00 35 00 00 00 00 00\n' "$1"; }  # type TT code 0x35 value 0
mk() { # NAME [DEVICE_NAME]
  mkdir -p "$T/$1"; printf '[    0.1] Booting Linux\n[    2.0] rmi4_i2c 1-0020: registering I2C-connected sensor\n' > "$T/$1/dmesg.txt"
  printf 'I: Bus=0018 Vendor=0000 Product=0000 Version=0000\nN: Name="%s"\nH: Handlers=event0\n\n' "${2:-synaptics_rmi4_i2c}" > "$T/$1/input-devices.txt"
  { ev 03; ev 03; ev 00; } > "$T/$1/events.txt"; }
mk good;                                    want "a probed controller with touch events passes" pass "$T/good"
mk syn "Synaptics RMI4 touchscreen sensor"; want "the driver's own input name matches" pass "$T/syn"
mk quiet; ev 00 > "$T/quiet/events.txt";    want "events without EV_ABS skip (nobody touched)" skip "$T/quiet"
mk noevents; rm "$T/noevents/events.txt";   want "no events file skips" skip "$T/noevents"
mk other "gpio-keys";                       want "no RMI4 input device fails" fail "$T/other"
mk fault; echo '[    2.1] rmi4_i2c 1-0020: Failed to read device status' >> "$T/fault/dmesg.txt"; want "a probe fault fails" fail "$T/fault"
mk nak;   echo "[    2.1] i2c_qup: 1-0x20 transfer failed nack" >> "$T/nak/dmesg.txt"; want "an i2c fault at 0x20 fails" fail "$T/nak"
mk unrel; echo "[    2.1] Permission check failed" >> "$T/unrel/dmesg.txt"; echo "[    2.2] i2c i2c-1: 0x2000 timeout" >> "$T/unrel/dmesg.txt"; want "unrelated failed lines do not fail the probe" pass "$T/unrel"
mkdir "$T/empty"; want "no files skip" skip "$T/empty"
[ $total -eq 0 ] || { echo "touch-selftest: $total failed" >&2; exit 1; }
echo "pass touch-selftest"
