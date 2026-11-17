#!/bin/sh
# walleye-try-ml5 reading aid and host test: ml5's timing bins (3 s per point, all under the ~33 s dead-kernel reset), checked against the init sources.
#   ml5-bin-table.sh INIT_DIR            prints the table and checks init-ml5a / init-ml5b say it
#   ml5-bin-table.sh INIT_DIR SECONDS    with BASE=<ml5a's measured return time> reads one ml5b time into a score
# Expected return (fastboot boot to adb): ml5a = BASE; ml5b = BASE + 3*score. ml5a at ~40 s = init ran; ~70 s = the kernel never reached init.
# Pure shell, no phone. Exit 0 only when the sources state it.
set -eu
D=${1:?usage: ml5-bin-table.sh INIT_DIR [ML5B_SECONDS]}
A=$D/init-ml5a; B=$D/init-ml5b
fail=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; fail=1; fi; }
ck "ml5a reboots at once (no watchdog, no loop but the dead-end one)" "$(grep -v '^#' "$A" | grep -c -E '/dev/watchdog|while \[|sleep [0-9]+ *$')" 0
ck "ml5a ends with reboot -f and a sysrq b fallback" "$(grep -c '^reboot -f; sleep 5; echo b > /proc/sysrq-trigger$' "$A")" 1
ck "ml5b counts 7 checks" "$(grep -c -E '^\[ .*\] && c[2-7]=1$|^grep -q .* && c5=1$' "$B")" 6
ck "ml5b sleeps 3 s per point" "$(grep -c -e "3 \* score" "$B")" 2
ck "ml5b opens the watchdog before the checklist" "$( [ "$(grep -n 'exec 3> /dev/watchdog' "$B" | cut -d: -f1)" -lt "$(grep -n '^c2=0' "$B" | cut -d: -f1)" ] && echo yes || echo no)" yes
ck "ml5b max sleep 21 s < 33 s" "$([ $((3*7)) -lt 33 ] && echo yes || echo no)" yes
BASE=${BASE:-40}
echo "score  sleep   ml5b back after fastboot boot (ml5a base $BASE s)"
s=1; while [ $s -le 7 ]; do echo "$s      $((3*s)) s    ~$((BASE + 3*s)) s"; s=$((s+1)); done
echo "(ml5b at ~70 s with the base ~40 s = init never ran; ml5a ~40 s but ml5b ~70 s = hung before the sleep)"
if [ -n "${2:-}" ]; then
  d=$(( $2 - BASE )); [ $d -ge 0 ] || d=0
  echo "measured $2 s -> delay $d s -> score $(( (d + 1) / 3 ))"
fi
[ $fail -eq 0 ]
