#!/bin/sh
# walleye-ffs-acm-up, run on the lab host: does the phone's FunctionFS gadget give a working serial console?
# Passes only when (1) a USB device with serial walleye-first-boot / product "Pixel 2 walleye (mainline)" is bound by cdc_acm
# (a tty under its sysfs tree) and (2) a command written to that tty is answered (marker round trip). Prints one line,
# `pass|fail|skip walleye-ffs-acm-up kernel DETAIL`; exit 0 only on pass. Writes only to the phone's own console tty.
# Environment: ACM_WAIT seconds to wait for the tty (default 0).
WAIT=${ACM_WAIT:-0}
say() { echo "$1 walleye-ffs-acm-up kernel $2"; [ "$1" = pass ]; exit $?; }
n=0; TTY=
while :; do
  for d in /sys/bus/usb/devices/*; do
    [ -r "$d/serial" ] || continue
    [ "$(cat "$d/serial" 2>/dev/null)" = walleye-first-boot ] || [ "$(cat "$d/product" 2>/dev/null)" = "Pixel 2 walleye (mainline)" ] || continue
    for t in "$d"/*/tty/ttyACM*; do [ -e "$t" ] && TTY=/dev/${t##*/}; done
  done
  [ -n "$TTY" ] && break
  [ "$n" -ge "$WAIT" ] && say fail "no ttyACM on a walleye gadget (device not up, or cdc_acm did not bind)"
  sleep 1; n=$((n+1))
done
[ -w "$TTY" ] || say skip "$TTY not writable by this user"
M=ffs-acm-$$
stty -F "$TTY" raw -echo 115200 2>/dev/null || say fail "stty on $TTY failed"
out=$(timeout 8 sh -c "exec 3<>$TTY; printf 'echo $M-\$((6*7))\n' >&3; while read -r l <&3; do case \$l in *$M-42*) echo \$l; break;; esac; done" 2>/dev/null)
case $out in *"$M-42"*) say pass "$TTY answers a shell command";; *) say fail "$TTY bound, no answer in 8 s";; esac
