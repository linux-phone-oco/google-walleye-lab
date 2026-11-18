#!/bin/sh
# walleye-pmic-wdt-bites, run on the lab host against a z8-style kernel (PMIC watchdog on, command loop on ttyACM): the PMIC watchdog (pm8998) bites unpetted.
# Opens the tty ONCE (fd 3), asks the watchdog's facts (identity, pretimeout, wakeup, bootstatus), then arms it with busybox `watchdog -F -T SECS -t 2` (-F: no fork, or the daemon keeps petting) and kills the petter with -9
# (no magic close: the watchdog stays armed) and times how long the tty lasts. Pass: the phone drops off USB between SECS-3 and SECS+15 s after arming (the init alone holds 180 s).
# Nothing is written to the phone beyond /tmp and a kmsg line. Answers are saved under OUT (default $HOME/lab-material/walleye/pmic-wdt-DATE, never in git), serials redacted.
# Environment: TTY (default /dev/ttyACM0), WAIT (s for the tty, 60), SECS (bite time, 10 to 60, default 20), OUT, DRY=1 (ask the facts, arm nothing; touch /tmp/hold).
# Prints `pass|fail walleye-pmic-wdt-bites host DETAIL`; exit 0 only on pass.
TTY=${TTY:-/dev/ttyACM0}; WAIT=${WAIT:-60}; SECS=${SECS:-20}
OUT=${OUT:-$HOME/lab-material/walleye/pmic-wdt-$(date +%Y%m%d-%H%M%S)}
trap "" HUP
say() { echo "$1 walleye-pmic-wdt-bites host $2"; [ "$1" = pass ]; exit $?; }
case $SECS in ''|*[!0-9]*) say fail "SECS must be a number from 10 to 60";; esac
[ "$SECS" -ge 10 ] && [ "$SECS" -le 60 ] || say fail "SECS must be from 10 to 60"
n=0; while [ ! -c "$TTY" ]; do [ "$n" -ge "$WAIT" ] && say fail "$TTY did not appear in ${WAIT}s"; sleep 1; n=$((n+1)); done
n=0; while [ ! -r "$TTY" ] || [ ! -w "$TTY" ]; do [ "$n" -ge 40 ] && say fail "$TTY not accessible to this user"; sleep 0.5; n=$((n+1)); done
mkdir -p "$OUT" && chmod 700 "$OUT" || say fail "cannot make $OUT"
command exec 3<>"$TTY" || say fail "cannot open $TTY"
timeout 5 stty raw -echo -hupcl 115200 <&3 2>/dev/null || true
redact() { sed -E -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/(androidboot\.serialno=)[^ ]*/\1REDACTED/g' -e 's/(serial(no|number)?[ =:]+)[0-9A-Za-z]{6,}/\1REDACTED/Ig'; }
M=wpwb-$$; i=0
ask() { i=$((i+1)); f="$OUT/$1.txt"
  timeout 0.3 cat <&3 >/dev/null 2>&1 || true
  printf '%s\n' "$3; echo $M-$i-E''ND" >&3
  S="while IFS= read -r l <&3; do l=\${l%\$(printf '\r')}; case \$l in *$M-$i-END) break;; esac; printf '%s\n' \"\$l\"; done"
  timeout "$2" sh -c "$S" 2>/dev/null | grep -v -e "$M-$i" -e WALLEYE-BANNER | redact > "$f"; }
printf '\n' >&3
ask hold 5 'touch /tmp/hold'
ask uname 15 'uname -r'
ask facts 15 'for d in /sys/class/watchdog/watchdog*; do echo "$d path=$(readlink -f $d) id=$(cat $d/identity 2>&1) pretimeout=$(cat $d/pretimeout 2>&1) wakeup=$(cat $d/device/power/wakeup 2>&1) bootstatus=$(cat $d/bootstatus 2>&1)"; done'
ask dmesg-wdt 15 'dmesg | grep -i -e wdt -e watchdog'
W=$(sed -n 's/^\(\/sys\/class\/watchdog\/watchdog[0-9]*\) path=[^ ]*spmi.*/\1/p' "$OUT/facts.txt" | head -n1 | tr -d '\r')
[ -n "$W" ] || say fail "no watchdog under an spmi device (the PMIC one; this kernel has no identity attribute) ($OUT/facts.txt)"
[ "${DRY:-0}" = 1 ] && say pass "dry run: $W present ($(grep -c . "$OUT/facts.txt") facts, nothing armed)"
D=/dev/${W##*/}
# arm, then drop the petter without the magic close; the tty answers nothing more once the phone resets
t0=$(date +%s)
printf '%s\n' "echo 'pmic-wdt: bite of $SECS s armed, unpetted' > /dev/kmsg; sync; busybox watchdog -F -T $SECS -t 2 $D & p=\$!; sleep 3; kill -9 \$p; echo $M-ARMED" >&3
timeout 15 sh -c "while IFS= read -r l <&3; do case \$l in *$M-ARMED*) break;; esac; done" 2>/dev/null
armed=$(date +%s)
# the phone is gone when the tty node is
gone=
while [ $(( $(date +%s) - armed )) -le $((SECS + 40)) ]; do
  [ -c "$TTY" ] || { gone=$(date +%s); break; }; sleep 0.5; done
[ -n "$gone" ] || say fail "the phone was still up $((SECS + 40)) s after arming: no bite (facts in $OUT)"
dt=$((gone - armed))
echo "armed at +$((armed - t0)) s, gone $dt s after arming (bite time $SECS s)" > "$OUT/timing.txt"
[ "$dt" -ge $((SECS - 3)) ] && [ "$dt" -le $((SECS + 15)) ] || say fail "gone $dt s after arming, not near $SECS s (the bite would be at $SECS s)"
say pass "$W bit: phone gone $dt s after arming for $SECS s (out $OUT)"
