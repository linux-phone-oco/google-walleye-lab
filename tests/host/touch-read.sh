#!/bin/sh
# walleye-touch-events reader, run on the lab host with the phone in z15 (z14's init: telnet on the manifest's address): saves dmesg and /proc/bus/input/devices
# (od keeps its lines only through telnetd's pty, which z15 has; the verdict is skip when no event line was read.)
# under OUT, then reads the RMI4 event node for SECONDS (default 20) while A PERSON TOUCHES THE PANEL (say so, wait for their go before starting) into events.txt
# (one 24-byte input_event per line, od -tx1 -w24). Nothing is written to the phone beyond /tmp. Then run touch-check.sh OUT.
#   tests/host/touch-read.sh [DEVICE_TOML]      Environment: OUT (default $HOME/lab-material/walleye/touch-DATE), SECONDS_TO_READ (default 20)
HERE=$(cd "$(dirname "$0")" && pwd)
TOML=${1:-$HERE/../../device.toml}
OUT=${OUT:-$HOME/lab-material/walleye/touch-$(date +%Y%m%d-%H%M%S)}
S=${SECONDS_TO_READ:-20}
say() { echo "$1 walleye-touch-read host $2"; [ "$1" != fail ]; exit $?; }
ADDR=$(sed -n 's/^addresses *= *\["\([^"]*\)".*/\1/p' "$TOML" | head -n1)
[ -n "$ADDR" ] || say fail "no [connection] addresses in $TOML"
mkdir -p "$OUT" && chmod 700 "$OUT" || say fail "cannot make $OUT"
if command -v nc >/dev/null 2>&1; then T="nc -w $((S+15)) $ADDR 23"; else T="timeout $((S+20)) telnet $ADDR 23"; fi
clean() { tr -d '\000-\010\013-\037\377' | sed 's/\r//g'; }
run() { # FILE COMMAND: the answer between two markers
  ( sleep 1; echo "echo TR-BEGIN; $2; echo TR-END"; sleep "${3:-3}"; echo exit ) | $T 2>/dev/null | clean | sed -n '/^TR-BEGIN$/,/^TR-END$/p' | sed '1d;$d' > "$1"
}
run "$OUT/dmesg.txt" "dmesg" 4
[ -s "$OUT/dmesg.txt" ] || say fail "phone at $ADDR gave no dmesg (telnet/nc): not a dts verdict"
run "$OUT/input-devices.txt" "cat /proc/bus/input/devices" 3
H=$(sed -n '/[Rr][Mm][Ii]\|[Ss]ynaptics/,/^$/p' "$OUT/input-devices.txt" | sed -n 's/.*Handlers=.*\(event[0-9]*\).*/\1/p' | head -n1)
[ -n "$H" ] || { : > "$OUT/events.txt"; say fail "no RMI4 input device to read events from (saved $OUT)"; }
echo "walleye-touch-read: reading /dev/input/$H for $S s: touch the panel now"
run "$OUT/events.txt" "timeout $S od -An -v -tx1 -w24 /dev/input/$H" "$((S+3))"
N=$(wc -l < "$OUT/events.txt")
[ "$N" -gt 0 ] || say skip "saved dmesg and input-devices in $OUT, but no event line: nobody touched? (not known yet)"
say pass "saved dmesg, input-devices and $N event lines in $OUT"
