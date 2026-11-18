#!/bin/sh
# walleye-massstorage-read, run on the lab host: read the log window of vendor-gadget-m1 (the phone's mass_storage LUN, a 2 MiB file its init rewrites every 5 s).
# Finds the LUN by the strings our init sets (serial walleye-first-boot, product "Pixel 2 walleye (mainline)"), never by VID:PID, then reads its first 64 KiB raw
# with O_DIRECT (iflag=direct, so the host's page cache never shows an old copy), prints the text (NULs dropped) and one summary line
# `pass|fail|skip walleye-massstorage-read kernel DETAIL`; exit 0 only on pass (pass = a block device of ours was read and it says WALLEYE-LOG).
# Reads the host's USB tree and the block device; changes nothing (the LUN is read-only for the host).
#
#   tests/host/massstorage-read.sh
# Environment: LUN_WAIT seconds to wait for the gadget to show up (default 0: look once);
#              LUN_WATCH seconds to keep reading every 5 s after the first read, printing each new beat (default 0: read once, print it whole);
#              LUN_DEV the block device to read instead of finding it (e.g. /dev/sdb); it must still be ours (checked by the same strings).
# The block device needs read access (the disk group or sudo); without it the result is skip, saying so.
WAIT=${LUN_WAIT:-0}
WATCH=${LUN_WATCH:-0}
say() { echo "$1 walleye-massstorage-read kernel $2"; [ "$1" = pass ]; exit $?; }
find_usb() {
  for d in /sys/bus/usb/devices/*; do
    [ -r "$d/serial" ] || continue
    if [ "$(cat "$d/serial" 2>/dev/null)" = walleye-first-boot ] || [ "$(cat "$d/product" 2>/dev/null)" = "Pixel 2 walleye (mainline)" ]; then echo "$d"; return; fi
  done
}
find_blk() { # the block device under the usb device $1
  for b in "$1"/*/host*/target*/*/block/*; do [ -e "$b" ] && { echo "/dev/${b##*/}"; return; }; done
}
n=0; USB=; BLK=
while :; do
  USB=$(find_usb)
  [ -n "$USB" ] && BLK=${LUN_DEV:-$(find_blk "$USB")}
  [ -n "$BLK" ] && break
  [ "$n" -ge "$WAIT" ] && break
  sleep 1; n=$((n+1))
done
[ -n "$USB" ] || say fail "no USB device with our gadget strings; phone-state: $(sh "$(dirname "$0")/phone-state.sh" 2>&1 | grep "^verdict" | head -n1 | grep . || echo unavailable)"
[ -n "$BLK" ] || say fail "gadget ${USB##*/} enumerated but no block device under it (usb-storage not bound, or the LUN has no medium yet)"
[ -r "$BLK" ] || say skip "gadget ${USB##*/} is $BLK but it is not readable by $(id -un) (disk group or sudo needed)"
read_once() { dd if="$BLK" bs=65536 count=1 iflag=direct 2>/dev/null | tr -d '\000'; }
T=$(mktemp); trap 'rm -f "$T"' EXIT
read_once > "$T"
head -n1 "$T" | grep -q '^WALLEYE-LOG' || say fail "gadget ${USB##*/} $BLK read, but the window does not start with WALLEYE-LOG (first line: $(head -n1 "$T" | cut -c1-60))"
if [ "$WATCH" -gt 0 ]; then
  end=$(( $(date +%s) + WATCH )); last=
  while :; do
    b=$(head -n1 "$T")
    [ "$b" != "$last" ] && { echo "$b"; last=$b; }
    [ "$(date +%s)" -ge "$end" ] && break
    sleep 5; read_once > "$T"
  done
  echo "=== the last window, whole ==="
fi
cat "$T"
say pass "gadget ${USB##*/} $BLK: $(head -n1 "$T")"
