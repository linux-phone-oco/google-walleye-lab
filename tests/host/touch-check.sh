#!/bin/sh
# walleye-touch-events, run on the lab host after touch-read.sh: did the Synaptics RMI4 touch controller probe and does a touch make input events?
# Reads the files the reader saved under DIR (dmesg.txt; input-devices.txt = /proc/bus/input/devices; events.txt = the raw bytes of the touch's event node, one 24-byte
# struct input_event per line as `od -An -tx1 -w24` prints them, read while a person touched the panel), and checks:
#   1. dmesg has no failed RMI4 probe ("rmi4 ... failed", "Failed to read", "Could not ... rmi", an i2c error at 0x20);
#   2. input-devices.txt lists an input device named like RMI4/Synaptics;
#   3. events.txt holds at least one EV_ABS event (type 0003: the touch's position or slot) - the proof; no events at all is `skip` (nobody touched: not known yet), never fail.
# Prints `pass|fail|skip walleye-touch-events host DETAIL`; exit 0 only on pass. Reads only.
# Usage: touch-check.sh DIR
D=${1:?usage: touch-check.sh DIR}
say() { echo "$1 walleye-touch-events host $2"; [ "$1" = pass ]; exit $?; }
[ -s "$D/dmesg.txt" ] || say skip "no dmesg in $D: not known yet"
[ -s "$D/input-devices.txt" ] || say skip "no input-devices.txt in $D: not known yet"
bad=$(grep -iE '(^|[] ])(rmi4?_[a-z0-9_.-]+|synaptics[a-z0-9_.-]*)[ :].*(failed|error|timeout)|Failed to read (device|RMI)|i2c.*[ -]0?x?0?20[: ].*(nack|fail|timeout)' "$D/dmesg.txt" | sed 's/^[^]]*\] //' | head -n 3 | tr '\n' ' ')
[ -z "$bad" ] || say fail "RMI4 probe fault: $bad"
name=$(sed -n 's/^N: Name="\(.*\)"$/\1/p' "$D/input-devices.txt" | grep -iE 'rmi|synaptics' | head -n1)
[ -n "$name" ] || say fail "no RMI4/Synaptics input device in input-devices.txt (devices: $(sed -n 's/^N: Name="\(.*\)"$/\1/p' "$D/input-devices.txt" | tr '\n' ',' ))"
[ -f "$D/events.txt" ] || say skip "input device '$name' is there, but no events were read: not known yet"
n=$(awk 'NF>=18 && $17=="03" && $18=="00"' "$D/events.txt" | wc -l)
[ "$n" -gt 0 ] || say skip "input device '$name' is there, but no touch event was read (was the panel touched while it read?): not known yet"
say pass "input device '$name' probed and $n EV_ABS events read from a touch"
