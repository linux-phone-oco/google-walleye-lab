#!/bin/sh
# walleye-display-scanout-registers, run on the lab host: does the image's init read the display registers, and only read them?
# Takes the regs block of build/initramfs-mainline/init-display-regs (INIT=FILE) and runs it with `say` stubbed, /proc/uptime replaced by a counter file, a stand-in `devmem`
# that records its arguments (answers 0x000000NN; fails on an address ending 00 listed as failing), /regs.list and /regs-times replaced by temp files:
#  - init-display-regs is init-display-delay plus added lines only, and the display block is untouched;
#  - the block is backgrounded; every list line is read once per sample time, in order, no earlier than the sample's uptime, 32 bits, with NO value (devmem is never asked to write);
#  - `walleye-regs: read NAME ADDR` is said before each read and `walleye-regs: T=S NAME ADDR VALUE` after; a failed read says FAILED and the next line still runs;
#  - a missing list or times file is said and reads nothing; a non-numeric time is skipped;
#  - the shipped list (build/initramfs-mainline/display-regs.list) has unique names, 0x0c9 addresses on a 4-byte boundary, and the shipped times are numbers.
# Prints `pass|fail walleye-display-regs kernel DETAIL`; exit 0 only on pass. Writes only a temp dir.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
INIT=${INIT:-$HERE/build/initramfs-mainline/init-display-regs}
BASE=${BASE:-$HERE/build/initramfs-mainline/init-display-delay}
LIST=${LIST:-$HERE/build/initramfs-mainline/display-regs.list}
TIMES=${TIMES:-$HERE/build/initramfs-mainline/display-regs-times}
say() { echo "$1 walleye-display-regs kernel $2"; [ "$1" = pass ]; exit $?; }
[ -r "$INIT" ] || say fail "no init at $INIT"
T=$(mktemp -d) || say fail "no temp dir"; trap 'kill $(cat "$T/counter.pid" 2>/dev/null) 2>/dev/null; rm -rf "$T"' EXIT
sed -n '/^# regs block begin/,/^# regs block end/p' "$INIT" | grep -v '^# regs block' > "$T/block"
[ -s "$T/block" ] || say fail "no regs block in the init"
grep -q '^) &$' "$T/block" || say fail "the regs block is not backgrounded"
diff -u "$BASE" "$INIT" | sed '1,2d' | grep -v '^@@' > "$T/diff"
[ "$(grep -c '^-' "$T/diff")" -eq 0 ] || say fail "a line of init-display-delay is removed: $(grep '^-' "$T/diff" | head -n1)"
[ "$(grep -c '^+' "$T/diff")" -gt 0 ] || say fail "the init adds nothing to init-display-delay"
[ -r "$LIST" ] && [ -r "$TIMES" ] || say fail "no register list or times file"
names=$(grep -v '^#' "$LIST" | cut -d' ' -f1 | grep -c .)
[ "$names" -gt 0 ] || say fail "the register list is empty"
[ "$(grep -v '^#' "$LIST" | cut -d' ' -f1 | sort -u | wc -l)" -eq "$names" ] || say fail "a register name is listed twice"
bad=$(grep -v '^#' "$LIST" | grep -vcE '^[A-Z0-9_]+ 0x0c9[0-9a-f]{5}$' || true)
[ "$bad" -eq 0 ] || say fail "$bad list lines are not NAME 0x0c9xxxxx"
[ "$(grep -v '^#' "$LIST" | awk '{print substr($2, 10, 1)}' | grep -cvE '^[0-9a-f]$')" -eq 0 ] || say fail "an address is malformed"
odd=$(grep -v '^#' "$LIST" | awk '{ v = strtonum($2); if (v % 4) n++ } END { print n+0 }')
[ "$odd" -eq 0 ] || say fail "$odd addresses are not on a 4-byte boundary"
[ "$(tr -d '0-9 \n' < "$TIMES" | wc -c)" -eq 0 ] && [ -n "$(tr -d ' \n' < "$TIMES")" ] || say fail "the shipped times are not numbers"
mkdir "$T/bin"
cat > "$T/bin/devmem" <<EOS
#!/bin/sh
echo "\$*" >> $T/devmem.args
case "\$1" in *00) [ -e $T/failing ] && { echo "devmem: failed" >&2; exit 1; };; esac
echo "0x000000AB"
EOS
chmod +x "$T/bin/devmem"
( u=0; while :; do echo "$u.00 0.00" > "$T/uptime.new"; mv "$T/uptime.new" "$T/uptime"; u=$((u+1)); sleep 1; done ) &
echo $! > "$T/counter.pid"
sleep 0.3
sed -e "s#/regs.list#$T/list#g" -e "s#/regs-times#$T/times#g" -e "s#/proc/uptime#$T/uptime#g" -e "s#/tmp/regs.log#$T/regs.log#g" -e "s#> /dev/kmsg#>> $T/kmsg#g" "$T/block" > "$T/run"
cat > "$T/harness" <<EOS
PATH=$T/bin:\$PATH
say() { echo "\$(cut -d. -f1 $T/uptime) \$*" >> $T/said; }
. $T/run
EOS
printf '# a comment\nAAA 0x0c900000\nBBB 0x0c900004\n\nCCC 0x0c900008\n' > "$T/list"
try() { # NAME TIMES-CONTENT (- = no file) FAILING
  rm -f "$T/said" "$T/kmsg" "$T/devmem.args" "$T/regs.log" "$T/failing"; : > "$T/said"; : > "$T/kmsg"
  [ "$2" = - ] || printf '%s\n' "$2" > "$T/times"; [ "$2" != - ] || rm -f "$T/times"
  [ "$3" != yes ] || : > "$T/failing"
  s=$(date +%s); sh "$T/harness"; e=$(date +%s)
  [ $((e - s)) -lt 2 ] || say fail "$1: the block did not return at once: it is not backgrounded"
}
waitfor() { n=0; while ! grep -q "$1" "$T/said" "$T/kmsg" 2>/dev/null && [ $n -lt 40 ]; do sleep 1; n=$((n+1)); done; }
u0=$(cut -d. -f1 "$T/uptime")
try two "$((u0+2)) $((u0+4))" no
waitfor "registers: done"
grep -q "registers: done" "$T/said" || say fail "two samples: the block never said it was done"
[ "$(cat "$T/devmem.args" | wc -l)" -eq 6 ] || say fail "two samples of three registers: $(cat "$T/devmem.args" | wc -l) reads, want 6"
[ "$(sort -u "$T/devmem.args" | wc -l)" -eq 3 ] || say fail "the reads are not the three listed registers"
[ "$(grep -vc ' 32$' "$T/devmem.args")" -eq 0 ] || say fail "a read is not 32 bits with nothing after the width (devmem was given a value?)"
[ "$(awk 'NF != 2' "$T/devmem.args" | wc -l)" -eq 0 ] || say fail "devmem got other than ADDRESS WIDTH"
[ "$(head -3 "$T/devmem.args" | tr '\n' ' ')" = "0x0c900000 32 0x0c900004 32 0x0c900008 32 " ] || say fail "the first sample is not in list order"
first=$(grep -m1 'sample' "$T/kmsg" | sed 's/.*at uptime \([0-9]*\) s/\1/')
[ "$first" -ge $((u0+2)) ] || say fail "the first sample ran at uptime $first, before $((u0+2))"
[ "$(grep -c 'walleye-regs: read ' "$T/kmsg")" -eq 6 ] || say fail "the read said before each read: $(grep -c 'walleye-regs: read ' "$T/kmsg") of 6"
grep -q 'walleye-regs: T='"$((u0+4))"' CCC 0x0c900008 0x000000AB' "$T/kmsg" || say fail "the second sample's last value is not in the kernel log"
[ "$(wc -l < "$T/regs.log")" -eq 6 ] || say fail "/tmp/regs.log does not hold the six values"
awk '/walleye-regs: read AAA/ {r=NR} /walleye-regs: T=.* AAA/ {if (!r || r > NR) bad=1} END {exit bad}' "$T/kmsg" || say fail "a value is said before its read line"
try failing "$((u0+1))" yes
waitfor "registers: done"
[ "$(grep -c 'FAILED' "$T/kmsg")" -eq 1 ] || say fail "a failing read is not said FAILED (the one address ending 00): $(grep -c FAILED "$T/kmsg")"
[ "$(wc -l < "$T/devmem.args")" -eq 3 ] || say fail "a failed read stopped the next lines"
try text "x 3" no
waitfor "registers: done"
[ "$(wc -l < "$T/devmem.args")" -eq 3 ] || say fail "a non-numeric time is not skipped, the numeric one read once"
try notimes - no
sleep 1
grep -q 'NO .*list or .*times' "$T/said" || say fail "a missing times file is not said"
[ ! -e "$T/devmem.args" ] || say fail "a missing times file still read registers"
say pass "reads the list once per sample time, in order, 32 bits, read only; says before and after; a failure or a bad time does not stop it; a missing file is said"
