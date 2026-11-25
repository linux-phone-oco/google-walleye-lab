#!/bin/sh
# the SLPI is running and the sensor service answers on qrtr
# HEALTH_LIB and REMOTEPROC_ROOT only let tests/host/slpi-sensor-service-selftest.sh run this on text the phone printed.
. "${HEALTH_LIB:-/usr/lib/devlab/health/lib.sh}"
rp=${REMOTEPROC_ROOT:-/sys/class/remoteproc}
slpi=false
for d in "$rp"/remoteproc*; do
	[ "$(cat "$d/name" 2>/dev/null)" = slpi ] && [ "$(cat "$d/state" 2>/dev/null)" = running ] && slpi=true
done
check slpi-running sensors "$slpi" "the remoteproc named slpi is running"
table=$(qrtr-lookup 2>/dev/null)
if printf '%s\n' "$table" | grep -qi 'sensor'; then
	check sensor-service sensors true "sensor service on qrtr"
elif [ "$(printf '%s\n' "$table" | grep -c '^ *[0-9]')" -gt 0 ]; then
	# the SLPI lists services whose ids qrtr does not name (349-362 on the Pixel 2): which one is the sensor service is not known yet
	say sensor-service sensors skip "$(printf '%s\n' "$table" | grep -c '^ *[0-9]') services on qrtr, none named sensor: not known yet"
else
	check sensor-service sensors false "sensor service on qrtr"
fi
