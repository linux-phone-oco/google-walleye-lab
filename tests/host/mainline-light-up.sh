#!/bin/sh
# walleye-mainline-light-up, run on the lab host: did OUR mainline kernel start? First light = a USB device whose strings are the ones our init sets
# (serial walleye-first-boot, product "Pixel 2 walleye (mainline)"), found in the host's USB tree - never by VID:PID (surya's gadget is on the same host) and
# never by the fastboot serial. Needs no host permission (sysfs, readable by everyone); no network, no block read.
# Prints one line, `pass|fail walleye-mainline-light-up kernel DETAIL`; exit 0 only on pass. Changes nothing.
#
#   tests/host/mainline-light-up.sh
# Environment: LIGHT_WAIT seconds to keep looking, once a second (default 0: look once).
WAIT=${LIGHT_WAIT:-0}
find_usb() {
  for d in /sys/bus/usb/devices/*; do
    [ -r "$d/serial" ] || continue
    if [ "$(cat "$d/serial" 2>/dev/null)" = walleye-first-boot ] && [ "$(cat "$d/product" 2>/dev/null)" = "Pixel 2 walleye (mainline)" ]; then echo "$d"; return; fi
  done
}
n=0; USB=
while :; do
  USB=$(find_usb)
  [ -n "$USB" ] && break
  [ "$n" -ge "$WAIT" ] && break
  sleep 1; n=$((n+1))
done
if [ -n "$USB" ]; then
  echo "pass walleye-mainline-light-up kernel ${USB##*/} $(cat "$USB/idVendor"):$(cat "$USB/idProduct") serial $(cat "$USB/serial") product $(cat "$USB/product") after ${n}s"
  exit 0
fi
echo "fail walleye-mainline-light-up kernel no USB device with serial walleye-first-boot and product 'Pixel 2 walleye (mainline)' after ${n}s"
exit 1
