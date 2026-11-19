#!/bin/sh
# walleye-display-delay-bisect-build, run on the lab host: does the image's init wait for the baked delay before it loads the display modules?
# Takes the display block of build/initramfs-mainline/init-display-delay (INIT=FILE) and runs it with `say` stubbed, /proc/uptime replaced by a
# file a counter advances once a second, /display-delay by a file holding 3 (then 0, then nothing, then text):
#  - with 3 the first insmod line runs no earlier than uptime 3 and the init says `walleye-display: delay 3 s reached at uptime T s` first;
#  - with 0, with no file and with a non-numeric file the modules load at once (no wait, never a hang);
#  - the block is backgrounded; init-display-delay is init-display plus added lines only; the hold of the self-reboot is kept.
# Prints `pass|fail walleye-display-delay kernel DETAIL`; exit 0 only on pass. Writes only a temp dir.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
INIT=${INIT:-$HERE/build/initramfs-mainline/init-display-delay}
BASE=${BASE:-$HERE/build/initramfs-mainline/init-display}
say() { echo "$1 walleye-display-delay kernel $2"; [ "$1" = pass ]; exit $?; }
[ -r "$INIT" ] || say fail "no init at $INIT"
T=$(mktemp -d) || say fail "no temp dir"; trap 'kill $(cat "$T/counter.pid" 2>/dev/null) 2>/dev/null; rm -rf "$T"' EXIT
sed -n '/^# display block begin/,/^# display block end/p' "$INIT" | grep -v '^# display block' > "$T/block"
[ -s "$T/block" ] || say fail "no display block in the init"
grep -q '^) &$' "$T/block" || say fail "the display block is not backgrounded"
grep -q '/display-delay' "$T/block" || say fail "the display block does not read /display-delay"
grep -q 'while \[ \$n -lt 90 \] || \[ -e /tmp/hold \]' "$INIT" || say fail "the init lost init-display's hold of the self-reboot"
diff -u "$BASE" "$INIT" | sed '1,2d' | grep -v '^@@' > "$T/diff"
[ "$(grep -c '^-' "$T/diff")" -eq 0 ] || say fail "a line of init-display is removed: $(grep '^-' "$T/diff" | head -n1)"
[ "$(grep -c '^+' "$T/diff")" -gt 0 ] || say fail "the init adds nothing to init-display"
printf 'true\ntrue\n' > "$T/list"
# a counter plays the uptime: 0, 1, 2, ... one per second, in the format of /proc/uptime
( u=0; while :; do echo "$u.00 0.00" > "$T/uptime.new"; mv "$T/uptime.new" "$T/uptime"; u=$((u+1)); sleep 1; done ) &
echo $! > "$T/counter.pid"
sleep 0.3
sed -e "s#/display-modules.list#$T/list#g" -e "s#/display-delay#$T/delay#g" -e "s#/proc/uptime#$T/uptime#g" -e "s#> /dev/kmsg#>> $T/kmsg#g" "$T/block" > "$T/run"
cat > "$T/harness" <<EOS
say() { echo "\$(cut -d. -f1 $T/uptime) \$*" >> $T/said; }
. $T/run
EOS
first_insmod_uptime() { awk '/^[0-9]+ display modules: loading/ {print $1; exit}' "$T/said"; }
try() { # NAME FILE-CONTENT (- = no file) WANT-DELAY-WORD
  rm -f "$T/delay" "$T/said" "$T/kmsg"; : > "$T/said"; : > "$T/kmsg"
  [ "$2" = - ] || printf '%s\n' "$2" > "$T/delay"
  s=$(date +%s)
  sh "$T/harness"
  e=$(date +%s)
  [ $((e - s)) -lt 2 ] || say fail "$1: the block did not return at once ($((e - s)) s): it is not backgrounded"
  n=0; while ! grep -q 'display modules: done' "$T/said" && [ $n -lt 30 ]; do sleep 0.5; n=$((n+1)); done
  grep -q 'display modules: done' "$T/said" || say fail "$1: the block never finished: $(tail -n1 "$T/said")"
  grep -q "walleye-display: delay $3 s reached at uptime [0-9]* s" "$T/kmsg" || say fail "$1: the init did not say 'delay $3 s reached' ($(tr '\n' '|' < "$T/kmsg"))"
}
try "delay 3" 3 3
u=$(first_insmod_uptime)
[ -n "$u" ] && [ "$u" -ge 3 ] || say fail "delay 3: the modules started at uptime '$u', before 3"
reached=$(sed -n 's/.*delay 3 s reached at uptime \([0-9]*\) s.*/\1/p' "$T/kmsg")
[ "$reached" -ge 3 ] || say fail "delay 3: said reached at uptime $reached, before 3"
[ "$(grep -c 'walleye-display: ok' "$T/kmsg")" -eq 2 ] || say fail "delay 3: want 2 ok lines: $(tr '\n' '|' < "$T/kmsg")"
for row in "zero|0|0" "no file|-|0" "text|soon|0"; do
  name=${row%%|*}; rest=${row#*|}; content=${rest%%|*}; want=${rest#*|}
  try "$name" "$content" "$want"
  u=$(first_insmod_uptime)
  now=$(cut -d. -f1 "$T/uptime")
  [ -n "$u" ] && [ $((now - u)) -le 3 ] || say fail "$name: the modules waited (started at uptime '$u', now $now)"
done
say pass "the display block waits for /display-delay seconds of uptime before the first insmod, says when it did, does not wait on 0, no file or text; init-display plus added lines only; the hold is kept"
