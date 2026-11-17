#!/bin/sh
# walleye-mainline-acm-up, run on the lab host: does OUR mainline kernel give a working serial console on the phone (pk5l)?
# Passes only when (1) a USB device with serial walleye-first-boot / product "Pixel 2 walleye (mainline)" is bound by cdc_acm (a ttyACM under its sysfs tree)
# and (2) 'uname -a; cat /proc/cmdline' written to that tty is answered with a Linux line and the command line. Then, when the LUN is readable, the log window
# is read through tests/host/massstorage-read.sh and its first line printed. Prints one line, `pass|fail|skip walleye-mainline-acm-up kernel DETAIL`;
# exit 0 only on pass. Writes only to the phone's own console tty.
# Environment: ACM_WAIT seconds to wait for the tty (default 0: look once).
WAIT=${ACM_WAIT:-0}
HERE=$(cd "$(dirname "$0")" && pwd)
say() { echo "$1 walleye-mainline-acm-up kernel $2"; [ "$1" = pass ]; exit $?; }
n=0; TTY=
while :; do
  for d in /sys/bus/usb/devices/*; do
    [ -r "$d/serial" ] || continue
    [ "$(cat "$d/serial" 2>/dev/null)" = walleye-first-boot ] && [ "$(cat "$d/product" 2>/dev/null)" = "Pixel 2 walleye (mainline)" ] || continue
    for t in "$d"/*/tty/ttyACM*; do [ -e "$t" ] && TTY=/dev/${t##*/}; done
  done
  [ -n "$TTY" ] && break
  [ "$n" -ge "$WAIT" ] && say fail "no ttyACM on a walleye gadget after ${n}s (device not up, or cdc_acm did not bind)"
  sleep 1; n=$((n+1))
done
[ -w "$TTY" ] || say skip "$TTY not writable by this user"
stty -F "$TTY" raw -echo 115200 2>/dev/null || say fail "stty on $TTY failed"
M=pk5l-$$
out=$(timeout 10 sh -c "exec 3<>$TTY; printf 'echo $M-A; uname -a; cat /proc/cmdline; echo $M-Z\n' >&3; while read -r l <&3; do case \$l in $M-Z*) break;; *) echo \$l;; esac; done" 2>/dev/null)
case $out in *"$M-A"*Linux*clk_ignore_unused*) ;; *) say fail "$TTY bound, no answer with uname and cmdline in 10 s: $(echo "$out" | head -c 200 | tr '\n' ' ')";; esac
win=$("$HERE/massstorage-read.sh" 2>&1 | grep -m1 -e WALLEYE-LOG -e '^skip' -e '^fail')
say pass "$TTY answers: $(echo "$out" | grep Linux | head -n1 | cut -c1-80); window: ${win:-not read}"
