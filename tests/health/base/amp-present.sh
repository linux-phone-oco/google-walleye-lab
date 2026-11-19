#!/bin/sh
# walleye's own check: both TAS2557 speaker amps and the DRV2624 haptics chip are bound by a driver.
# Fails until walleye-tas2557-driver and walleye-drv2624-driver work (decision 2026-11-19-walleye-audio-haptics-way).
. /usr/lib/devlab/health/lib.sh
for a in 0x4c 0x4d; do
	d=$(ls -d /sys/bus/i2c/devices/*-00"${a#0x}" 2>/dev/null | head -1)
	check "amp-present-$a" audio-speaker "$([ -n "$d" ] && [ -e "$d/driver" ] && echo true)" "TAS2557 at ${a} ${d:+bound: $(readlink "$d/driver" 2>/dev/null)}${d:-no i2c device}"
done
d=$(ls -d /sys/bus/i2c/devices/*-005a 2>/dev/null | head -1)
check haptics-chip-present haptics "$([ -n "$d" ] && [ -e "$d/driver" ] && echo true)" "DRV2624 at 0x5a ${d:-no i2c device}"
