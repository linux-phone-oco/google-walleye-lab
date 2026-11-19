#!/bin/sh
# walleye-display-at-boot, run on the lab host: does the image's init load the display modules itself, so no host script has to?
# Takes the display block of build/initramfs-mainline/init-display (INIT=FILE; between "# display block begin" and "# display block end") and runs it with `say` stubbed on a list of lines (true, false, blank, a missing command, sleep, true):
# every line must run, failures be logged with their rc, the block be backgrounded, a missing list be said; and the init holds its self-reboot as init-late does.
# Prints `pass|fail walleye-display-at-boot kernel DETAIL`; exit 0 only on pass. Writes only a temp dir.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
INIT=${INIT:-$HERE/build/initramfs-mainline/init-display}
say() { echo "$1 walleye-display-at-boot kernel $2"; [ "$1" = pass ]; exit $?; }
[ -r "$INIT" ] || say fail "no init at $INIT"
T=$(mktemp -d) || say fail "no temp dir"; trap 'rm -rf "$T"' EXIT
sed -n '/^# display block begin/,/^# display block end/p' "$INIT" | grep -v '^# display block' > "$T/block"
[ -s "$T/block" ] || say fail "no display block in the init"
grep -q '^) &$' "$T/block" || say fail "the display block is not backgrounded"
grep -q 'while \[ \$n -lt 90 \] || \[ -e /tmp/hold \]' "$INIT" || say fail "the init lost init-late's hold of the self-reboot"
sed -e "s#/display-modules.list#$T/list#g" -e "s#> /dev/kmsg#>> $T/kmsg#g" "$T/block" > "$T/run"
printf 'true\nfalse\n\nnosuchcommand-xyz\nsleep 2\ntrue\n' > "$T/list"
: > "$T/kmsg"; : > "$T/said"
cat > "$T/harness" <<EOS
say() { echo "\$*" >> $T/said; }
. $T/run
EOS
s=$(date +%s)
sh "$T/harness"      # the block backgrounds itself: this returns at once
e=$(date +%s)
[ $((e - s)) -lt 2 ] || say fail "the block did not return at once ($((e - s)) s): it is not backgrounded"
n=0; while ! grep -q 'display modules: done' "$T/said" && [ $n -lt 20 ]; do sleep 0.5; n=$((n+1)); done
grep -q 'display modules: done' "$T/said" || say fail "the block never finished: $(tail -n1 "$T/said")"
[ "$(grep -c 'walleye-display: ok' "$T/kmsg")" -eq 3 ] || say fail "want 3 ok lines (true, sleep, true): $(tr '\n' '|' < "$T/kmsg")"
grep -q 'walleye-display: FAILED rc=1$' "$T/kmsg" || say fail "the failing line 'false' was not logged with rc=1"
grep -q 'FAILED rc=127$' "$T/kmsg" || say fail "the missing command was not logged with rc=127"
rm -f "$T/list"; : > "$T/said"
sh "$T/harness"; sleep 1
grep -q '^NO .*list' "$T/said" && ! grep -q 'done' "$T/said" || say fail "a missing list is not said (or reported done)"
say pass "the display block runs every line, logs failures with their rc, is backgrounded, says a missing list; the hold is kept"
