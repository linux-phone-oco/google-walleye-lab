#!/bin/sh
# walleye-gadget-up, run on the lab host (the phone cannot run it): did a boot image of ours leave fastboot?
# Passes only when (1) a USB device on this host enumerates as walleye's Linux gadget - found by the strings our init sets
# (serial walleye-first-boot, product "Pixel 2 walleye (mainline)"), never by VID:PID (surya's gadget 18d1:d001 is on the same
# host) and never by the fastboot serial alone (fastboot shows that serial too) - and (2) the manifest's address answers ping and
# telnet. Prints one line, `pass|fail|skip walleye-gadget-up kernel DETAIL`; exit 0 only on pass. Reads the host's USB tree and
# talks to the phone's address; changes nothing.
#
#   tests/host/gadget-up.sh [DEVICE_TOML]      (default: ../device.toml next to tests/)
# Environment: GADGET_WAIT seconds to wait for the gadget to show up (default 0: look once).
HERE=$(cd "$(dirname "$0")" && pwd)
TOML=${1:-$HERE/../../device.toml}
WAIT=${GADGET_WAIT:-0}
say() { echo "$1 walleye-gadget-up kernel $2"; [ "$1" = pass ]; exit $?; }
[ -r "$TOML" ] || say skip "no manifest $TOML"
ADDR=$(sed -n 's/^addresses *= *\["\([^"]*\)".*/\1/p' "$TOML" | head -n1)
[ -n "$ADDR" ] || say skip "no [connection] addresses in $TOML"
n=0; DEV=
while :; do
  for d in /sys/bus/usb/devices/*; do
    [ -r "$d/serial" ] || continue
    if [ "$(cat "$d/serial" 2>/dev/null)" = walleye-first-boot ] || [ "$(cat "$d/product" 2>/dev/null)" = "Pixel 2 walleye (mainline)" ]; then DEV=${d##*/}; fi
  done
  [ -n "$DEV" ] || [ "$n" -ge "$WAIT" ] && break
  sleep 1; n=$((n+1))
done
[ -n "$DEV" ] || say fail "no USB device with our gadget strings; phone-state: $(sh "$(dirname "$0")/phone-state.sh" 2>&1 | grep "^verdict" | head -n1 | grep . || echo unavailable)"
ping -c 2 -W 3 "$ADDR" >/dev/null 2>&1 || say fail "gadget $DEV enumerated but $ADDR does not answer ping"
if command -v nc >/dev/null 2>&1; then T="nc -w 5 $ADDR 23"; else T="timeout 8 telnet $ADDR 23"; fi
R=$( (sleep 1; echo "echo gadget-up-\$((6*7))"; sleep 2; echo exit) | $T 2>/dev/null | tr -d '\000-\010\013-\037\377' | grep -o 'gadget-up-42' | tail -1)
[ -n "$R" ] || say fail "gadget $DEV, $ADDR pings but telnet on 23 gives no answer"
say pass "gadget $DEV enumerated, $ADDR answers ping and telnet"
