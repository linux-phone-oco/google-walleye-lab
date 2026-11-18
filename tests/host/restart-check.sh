#!/bin/sh
# restart-check (backlog walleye-restart-gate): does the phone, once it enumerated as the mainline gadget (1d6b:0104), LEAVE the USB bus again within LIMIT seconds? An image whose own self-reboot does not take the phone off the bus has no restart path.
#   sh tests/host/restart-check.sh HOST_JOURNAL [LIMIT_SECONDS] [NOW_SECONDS_SINCE_EPOCH]
# HOST_JOURNAL: the host kernel's `journalctl -k -o short` lines of `usb 3-2` (any port) for one try, oldest first.
# pass: the gadget's "New USB device found, idVendor=1d6b, idProduct=0104" line is followed by a "USB disconnect" within LIMIT (default 200 s: the init's self-reboot is 180 s, research walleye-c3-hang-how-it-stands).
# fail (exit 1): no disconnect within LIMIT of the enumeration (the journal's last line, or NOW when given, counts as the end of the watch). skip (exit 2): no journal, or no gadget enumeration in it (not known yet).
J=${1:?usage: restart-check.sh HOST_JOURNAL [LIMIT_SECONDS] [NOW_EPOCH]}
LIMIT=${2:-200}
[ -f "$J" ] || { echo "skip: no journal $J (not known yet)"; exit 2; }
stamp() { date -d "$(echo "$1" | cut -c1-15)" +%s; }
UP=$(grep -m1 'idVendor=1d6b, idProduct=0104' "$J") || { echo "skip: no 1d6b:0104 enumeration in $J (not known yet)"; exit 2; }
T0=$(stamp "$UP")
DOWN=$(awk -v n="$(grep -n -m1 'idVendor=1d6b, idProduct=0104' "$J" | cut -d: -f1)" 'NR>n && /USB disconnect/ {print; exit}' "$J")
if [ -n "$DOWN" ]; then
  T1=$(stamp "$DOWN"); D=$((T1-T0))
  if [ "$D" -le "$LIMIT" ]; then echo "pass: the gadget left the bus ${D}s after it enumerated (limit ${LIMIT}s)"; exit 0; fi
  echo "fail: the gadget left the bus only ${D}s after it enumerated (limit ${LIMIT}s)"; exit 1
fi
if [ -n "${3:-}" ]; then T1=$3; else T1=$(stamp "$(tail -n1 "$J")"); fi
[ $((T1-T0)) -ge "$LIMIT" ] || { echo "skip: the watch is only $((T1-T0))s, shorter than the limit (not known yet)"; exit 2; }
echo "fail: the gadget enumerated at $(echo "$UP" | cut -c1-15) and no USB disconnect follows in $((T1-T0))s of watch (limit ${LIMIT}s): no restart path"
exit 1
