#!/bin/sh
# walleye-phone-state, run on the lab host: what state is the phone in? A VERDICT OVER SEVERAL WITNESSES, never lsusb alone
# (research walleye-c3-hang-unproven: a gadget stayed listed 44 min while the phone ran Android; a USB host drops a device only on a disconnect event).
# Witnesses, each bounded by timeout(1), read-only:
#   adb      `adb devices` for the manifest serial: device | unauthorized | offline | the state adb names (recovery, sideload, no permissions...) | none
#   fastboot `fastboot devices`: the manifest serial listed | none
#   usb      sysfs /sys/bus/usb/devices: our gadget (serial walleye-first-boot or product "Pixel 2 walleye (mainline)") | the Pixel's own serial | none
#   tty      only when our gadget is listed: the gadget's own tty node (from its sysfs interfaces; TTY overrides), a WALLEYE-BANNER line or any bytes
#            within TTY_WAIT s: banner | answers | silent | no-node | no-access (could not be set up or opened) | n/a
#   age      not measured: sysfs keeps no enumeration time (a stale listing has an age that proves nothing); shown as "not known yet"
# Verdict (one word; it NEVER says hung):
#   android            adb says device (a stale gadget listing is outvoted)
#   fastboot           fastboot lists the serial
#   mainline-answering our gadget is listed and the tty answered
#   mainline-silent    our gadget is listed and the tty is silent: "NOT KNOWN: the host may hold a stale enumeration"
#   not-known          adb lists the serial in another state, or witnesses disagree otherwise
#   gone               no witness lists the phone
# Prints one line per witness (`witness NAME VALUE`) and `verdict WORD DETAIL`. Exit 0 for android, fastboot, mainline-answering; 1 for gone; 2 for
# mainline-silent and not-known (not known yet). Changes nothing on the phone: the tty is set raw -echo -hupcl clocal first (a failure there means no read),
# so reading it neither echoes into the phone's shell nor hangs it up on close. At most about 25 s: adb 7 + fastboot 7 + stty 3 + read TTY_WAIT+1.
#
#   tests/host/phone-state.sh [DEVICE_TOML]      (default: ../../device.toml)
# Environment: TTY (default: the gadget's own node), TTY_WAIT seconds (default 5), CALL_WAIT seconds per adb/fastboot call (default 6), SYSFS_USB (default /sys/bus/usb/devices).
HERE=$(cd "$(dirname "$0")" && pwd)
TOML=${1:-$HERE/../../device.toml}
TW=${TTY_WAIT:-5}; CW=${CALL_WAIT:-6}; SYS=${SYSFS_USB:-/sys/bus/usb/devices}
case $TW$CW in *[!0-9]*|'') echo "verdict not-known TTY_WAIT and CALL_WAIT must be whole seconds"; exit 2;; esac
command -v timeout >/dev/null || { echo "verdict not-known timeout(1) is missing, so no call can be bounded"; exit 2; }
SER=$(awk '/^\[/ {s=($0=="[fastboot]")} s && /^serial *=/ {gsub(/^[^"]*"|".*$/,""); print; exit}' "$TOML" 2>/dev/null)
[ -n "$SER" ] || { echo "verdict not-known no [fastboot] serial in $TOML"; exit 2; }

# adb: the state column of our serial's line, whatever it is
A=none
out=$(timeout -k 1 "$CW" adb devices 2>/dev/null | tr -d '\r' | awk -v s="$SER" '$1==s {$1=""; sub(/^ +/,""); print; exit}')
[ -n "$out" ] && A=$out
# fastboot (only reads the list; bounded)
F=none
timeout -k 1 "$CW" fastboot devices 2>/dev/null | tr -d '\r' | awk -v s="$SER" '$1==s {f=1} END {exit !f}' && F=listed
# usb by sysfs: ours (strings our init sets), or the Pixel's own serial; never by VID:PID alone
U=none; DEV=
for d in "$SYS"/*; do
  [ -r "$d/serial" ] || continue
  s=$(cat "$d/serial" 2>/dev/null); p=$(cat "$d/product" 2>/dev/null)
  if [ "$s" = walleye-first-boot ] || [ "$p" = "Pixel 2 walleye (mainline)" ]; then U=gadget; DEV=$d; break; fi
  if [ "$s" = "$SER" ]; then U=pixel:$(cat "$d/idVendor" 2>/dev/null):$(cat "$d/idProduct" 2>/dev/null); fi
done
# tty, only for our gadget, and only the node that belongs to it
T=n/a
if [ "$U" = gadget ]; then
  N=${TTY:-}
  if [ -z "$N" ]; then
    for t in "$DEV":*/tty/*; do [ -e "$t" ] && { N=/dev/${t##*/}; break; }; done
  fi
  if [ -z "$N" ] || { [ ! -c "$N" ] && [ ! -p "$N" ] && [ ! -f "$N" ]; }; then T=no-node
  else
    ok=1
    [ -c "$N" ] && { timeout 3 stty -F "$N" raw -echo -hupcl clocal 2>/dev/null || ok=0; }
    if [ $ok = 1 ]; then
      got=$(timeout -k 1 "$TW" cat "$N" 2>/dev/null | tr -d '\000\r' | head -c 4096)
      if [ -n "$got" ]; then T=answers; case $got in *WALLEYE-BANNER*) T=banner;; esac
      elif [ -r "$N" ]; then T=silent; else T=no-access; fi
    else T=no-access; fi
  fi
fi

echo "witness adb $A"; echo "witness fastboot $F"; echo "witness usb $U"; echo "witness tty $T"; echo "witness age not known yet"
v() { echo "verdict $1 $2"; exit "$3"; }
case $A in
  device) v android "adb lists the serial as device (usb $U); a stale listing cannot outvote it" 0;;
  none) ;;
  *) v not-known "adb lists the serial as '$A': the phone is there but not usable from this host" 2;;
esac
[ "$F" = listed ] && v fastboot "fastboot lists $SER" 0
case $U in
  gadget) case $T in
      answers|banner) v mainline-answering "our gadget is listed and the tty answered ($T)" 0;;
      *) v mainline-silent "NOT KNOWN: the host may hold a stale enumeration - our gadget is listed, adb and fastboot do not see the phone, the tty is $T" 2;;
    esac;;
  pixel:*) v not-known "the Pixel's serial is on the bus ($U) but adb and fastboot do not list it" 2;;
esac
v gone "no witness lists the phone (adb none, fastboot none, usb none)" 1
