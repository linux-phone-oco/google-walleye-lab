#!/bin/sh
# walleye-vendor-facts-poll-selftest, host only (no phone): runs the real functions of build/initramfs-vendor5/init-vf3 (beat_once, flush, chunk, dtnode, cat_file, redact_filter) against a window file,
# then runs tests/host/vendor-facts-poll.sh on it (VF_FILE) and checks: the window equals the stream across 4096-byte boundaries; a finished block is fetched whole and redacted; a failing source shows its rc;
# a torn block (no END) is not fetched; DONE stops the poller; a window with a MAC-shaped word that got past the init's redaction is refused and deleted; a name with .. or / lands nowhere outside; no marker fails and leaves nothing.
# Prints ok/FAIL lines and `pass|fail walleye-vendor-facts-poll-selftest host N ok, M failed`; exit 0 only when none failed.
HERE=$(cd "$(dirname "$0")" && pwd)
INIT=$HERE/../../build/initramfs-vendor5/init-vf3
POLL=$HERE/vendor-facts-poll.sh
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0; pass=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
# the init's functions, taken out of the init itself (a one-line function ends on its own line, a longer one at a line "}")
fn() { awk -v n="$1" '$0 ~ "^" n "\\(\\) \\{" { print; if ($0 !~ /\}$/ || $0 ~ /\{$/) inside = 1; if (!inside) next; next } inside { print; if ($0 == "}") inside = 0 }' "$INIT"; }
{
  for f in beat_once flush stage redact_filter chunk dtnode cat_file cat_dir; do fn $f; done
} | sed "s|/tmp/walleye-|$T/walleye-|g" > $T/fns.sh
ck "the init's functions were all found (beat_once flush stage redact_filter chunk dtnode cat_file cat_dir)" "$(grep -c -E '^(beat_once|flush|stage|redact_filter|chunk|dtnode|cat_file|cat_dir)\(\) \{' $T/fns.sh)" 8
cat > $T/harness.sh <<EOS
LOGF=$T/win.img; OUT=$T/stream; BUDGET=3800000; SPENT=0; PERSRC=262144; BLOCK=4096; WROTE=0
. $T/fns.sh
dd if=/dev/zero of=\$LOGF bs=4096 count=1024 2>/dev/null; : > \$OUT
echo start > $T/walleye-stage; echo 0 > $T/walleye-bytes
beat_once 0
EOS
# sources: a text with an identifying word to redact, a big one crossing many 4096-byte boundaries, a device-tree node, a failing command
printf 'console=ttyMSM0 androidboot.serialno=RZ8M123456 mac 02:11:22:33:44:55 imei 123456789012345\n' > $T/src1
i=0; : > $T/src2; while [ $i -lt 900 ]; do echo "line $i of the big source, padded to make the stream cross a few 4096-byte boundaries ..........." >> $T/src2; i=$((i+1)); done
mkdir -p $T/dt%soc; printf 'walleye\0' > $T/dt%soc/model; printf '\000\000\000\001\377' > $T/dt%soc/reg; printf '02:11:22:33:44:55' > $T/dt%soc/local-mac-address
failing() { echo partial; echo "boom" >&2; return 4; }
cat > $T/run1.sh <<EOS
. $T/harness.sh
chunk cmdline cat_file $T/src1
chunk big cat_file $T/src2
chunk dt%soc dtnode $T/dt%soc
chunk dt%soc%spi dtnode $T/dt%soc
chunk bad failing
echo "DONE up=1 bytes=\$(stat -c %s \$OUT)" >> \$OUT; flush
EOS
sed -i "1i failing() { echo partial; echo boom >&2; return 4; }" $T/run1.sh
sh $T/run1.sh > $T/run1.out 2>&1
SZ=$(stat -c %s $T/stream)
ck "the window is 4 MiB and the stream (over several 4096 boundaries) is in it byte for byte from 4096" "$(stat -c %s $T/win.img) $(dd if=$T/win.img bs=4096 skip=1 2>/dev/null | head -c $SZ | cmp - $T/stream && echo same)" "4194304 same"
ck "block 0 is the beat line only, and nothing of the stream is in it" "$(dd if=$T/win.img bs=4096 count=1 2>/dev/null | tr -d '\000' | head -n1 | grep -c '^WALLEYE-VF3 BEAT=0 UP=.* STAGE=.* BYTES=')" 1
ck "the init's redaction took the serialno, the MAC and the 15 digits out of the stream" "$(grep -c -e 'serialno=REDACTED' -e 'XX:XX:XX:XX:XX:XX' -e 'IMEI-SHAPED' $T/stream)$(grep -c -e 'RZ8M123456' -e '02:11:22:33:44:55' -e '123456789012345' $T/stream)" 10
ck "a device-tree node: text is kept, binary is hex, a *mac-address* property is left out" "$(grep -c -e '^== model$' -e '^walleye|$' -e '^ 00 00 00 01 ff$' $T/stream)$(grep -c 'local-mac-address' $T/stream)" 60
ck "a failing source is framed with its real rc and the first words of its stderr" "$(grep -c '^END bad rc=4: boom' $T/stream)" 1
rm -f $T/win-keep; cp $T/win.img $T/win-good.img
# the poller on that window
VF_FILE=$T/win-good.img VF_OUT=$T/o1 POLL_ONCE=1 sh $POLL > $T/p1.out 2>&1; r=$?
ck "the poller passes on a window that carries DONE" "$r" 0
ck "it says DONE, the number of blocks and copies" "$(grep -c -e "^pass walleye-vendor-facts-poll host 5 blocks in .*, 1 copies, 1 beats, DONE" $T/p1.out)" 1
ck "every finished block is a file (cmdline, big, dt%soc, bad) and the END line is its last line" "$(for n in cmdline big dt%soc bad; do tail -n1 $T/o1/vf/$n | cut -c1-8 | tr '\n' ' '; done)" "END cmdl END big  END dt%s END bad  "
ck "the big block is whole (900 lines + frame)" "$(grep -c '^line ' $T/o1/vf/big)" 900
ck "the last complete copy is kept and the beat log has a line" "$(test -s $T/o1/last-complete.txt && echo kept)$(wc -l < $T/o1/beat.log)" "kept1"
# a torn window: the stream cut inside the last block
head -c 6000 $T/stream | sed '$d' > $T/torn.txt
dd if=/dev/zero of=$T/win-torn.img bs=4096 count=1024 2>/dev/null
dd if=$T/win-good.img of=$T/win-torn.img bs=4096 count=1 conv=notrunc 2>/dev/null
dd if=$T/torn.txt of=$T/win-torn.img bs=4096 seek=1 conv=notrunc 2>/dev/null
VF_FILE=$T/win-torn.img VF_OUT=$T/o2 POLL_ONCE=1 sh $POLL > $T/p2.out 2>&1; r=$?
ck "a torn window (cut inside a block) passes with the finished blocks only, no DONE" "$r$(grep -c 'no DONE, partial' $T/p2.out)" 01
ck "the torn block is not fetched, the one before it is" "$(test -e $T/o2/vf/cmdline && echo cmdline)$(test -e $T/o2/vf/big && echo big)" cmdline
# a window with the marker and a block that never ends
{ printf 'BEGIN half\nsome text\n'; } > $T/half.txt
dd if=$T/win-good.img of=$T/win-half.img bs=4096 count=1 2>/dev/null; dd if=/dev/zero of=$T/win-half.img bs=4096 seek=1 count=1023 conv=notrunc 2>/dev/null
dd if=$T/half.txt of=$T/win-half.img bs=4096 seek=1 conv=notrunc 2>/dev/null
VF_FILE=$T/win-half.img VF_OUT=$T/o3 POLL_ONCE=1 sh $POLL > $T/p3.out 2>&1; r=$?
ck "a marker with no finished block fails, naming the beat" "$r$(grep -c 'no block ended' $T/p3.out)" 11
# a MAC that got past the init (as if the redaction broke): refused, deleted
{ printf 'BEGIN leak\nlink 02:11:22:33:44:55 up\nEND leak rc=0\n'; } > $T/leak.txt
dd if=$T/win-good.img of=$T/win-leak.img bs=4096 count=1 2>/dev/null; dd if=/dev/zero of=$T/win-leak.img bs=4096 seek=1 count=1023 conv=notrunc 2>/dev/null
dd if=$T/leak.txt of=$T/win-leak.img bs=4096 seek=1 conv=notrunc 2>/dev/null
VF_FILE=$T/win-leak.img VF_OUT=$T/o4 POLL_ONCE=1 sh $POLL > $T/p4.out 2>&1; r=$?
ck "a MAC-shaped word in a finished block is refused and the output deleted" "$r$(test -e $T/o4 && echo kept)" 1
# names that must land nowhere
{ printf 'BEGIN ../evil\nx\nEND ../evil rc=0\nBEGIN /abs\nx\nEND /abs rc=0\nBEGIN ok%%name\ny\nEND ok%%name rc=0\n'; } > $T/names.txt
dd if=$T/win-good.img of=$T/win-names.img bs=4096 count=1 2>/dev/null; dd if=/dev/zero of=$T/win-names.img bs=4096 seek=1 count=1023 conv=notrunc 2>/dev/null
dd if=$T/names.txt of=$T/win-names.img bs=4096 seek=1 conv=notrunc 2>/dev/null
VF_FILE=$T/win-names.img VF_OUT=$T/o5/sub POLL_ONCE=1 sh $POLL > $T/p5.out 2>&1; r=$?
ck "a name with .. or a leading / is not written; a good one is" "$r$(test -e $T/o5/evil && echo evil)$(test -e /abs && echo abs)$(test -e $T/o5/sub/vf/ok%name && echo ok)" 0ok
# no marker at all
printf 'garbage, not ours' | dd of=$T/win-none.img bs=4096 count=1024 conv=sync 2>/dev/null
VF_FILE=$T/win-none.img VF_OUT=$T/o6 POLL_ONCE=1 sh $POLL > $T/p6.out 2>&1; r=$?
ck "a window with no WALLEYE-VF3 marker fails and leaves nothing" "$r$(test -e $T/o6 && echo kept)$(grep -c 'no window with our marker' $T/p6.out)" 11
# the loop ends at POLL_MAX when there is no DONE
VF_FILE=$T/win-torn.img VF_OUT=$T/o7 POLL_SECS=1 POLL_MAX=2 sh $POLL > $T/p7.out 2>&1; r=$?
ck "without DONE the poller stops at POLL_MAX (2 s) with its partial result" "$r$(grep -c 'no DONE, partial' $T/p7.out)" 01
# the loop stops at DONE long before POLL_MAX
s=$(date +%s); VF_FILE=$T/win-good.img VF_OUT=$T/o8 POLL_SECS=1 POLL_MAX=60 sh $POLL > $T/p8.out 2>&1; e=$(date +%s)
ck "with DONE the poller stops at once, not at POLL_MAX" "$([ $((e - s)) -lt 10 ] && echo fast)$(grep -c 'DONE' $T/p8.out)" fast1
ck "nested device-tree nodes (dt%soc and dt%soc%spi) are both fetched, as flat files" "$(test -s $T/o1/vf/dt%soc && test -s $T/o1/vf/dt%soc%spi && echo both)" both
# a hole: text, a block of NULs, then more text (a read across a rewrite): only the text before the hole is taken, and nothing after it is glued on
{ printf 'BEGIN a\nx\nEND a rc=0\n'; } > $T/h1.txt; { printf 'BEGIN b\ny\nEND b rc=0\n'; } > $T/h2.txt
dd if=$T/win-good.img of=$T/win-hole.img bs=4096 count=1 2>/dev/null; dd if=/dev/zero of=$T/win-hole.img bs=4096 seek=1 count=1023 conv=notrunc 2>/dev/null
dd if=$T/h1.txt of=$T/win-hole.img bs=4096 seek=1 conv=notrunc 2>/dev/null; dd if=$T/h2.txt of=$T/win-hole.img bs=4096 seek=3 conv=notrunc 2>/dev/null
VF_FILE=$T/win-hole.img VF_OUT=$T/o9 POLL_ONCE=1 sh $POLL > $T/p9.out 2>&1
ck "a hole ends the used part: the block before it is fetched, the one after it is not" "$(test -e $T/o9/vf/a && echo a)$(test -e $T/o9/vf/b && echo b)" a
# a torn last line (no newline) is dropped
printf 'BEGIN t\nz\nEND t rc=0\nBEGIN u\nq\nEND u rc' > $T/h3.txt
dd if=$T/win-good.img of=$T/win-line.img bs=4096 count=1 2>/dev/null; dd if=/dev/zero of=$T/win-line.img bs=4096 seek=1 count=1023 conv=notrunc 2>/dev/null
dd if=$T/h3.txt of=$T/win-line.img bs=4096 seek=1 conv=notrunc 2>/dev/null
VF_FILE=$T/win-line.img VF_OUT=$T/o10 POLL_ONCE=1 sh $POLL > $T/p10.out 2>&1
ck "a last line with no newline is not taken as a finished END" "$(test -e $T/o10/vf/t && echo t)$(test -e $T/o10/vf/u && echo u)" t
ck "a refused leak leaves no win/ copy and no last-complete (the whole output is gone)" "$(ls $T/o4 2>&1 | grep -c 'No such')" 1
# the init's budget: once spent, one budget line, then silence
cat > $T/run2.sh <<EOS
. $T/harness.sh
BUDGET=100
chunk one cat_file $T/src1
chunk two cat_file $T/src1
chunk three cat_file $T/src1
EOS
sh $T/run2.sh > /dev/null 2>&1
ck "when the budget is spent the init says so once and skips the rest silently" "$(grep -c '^BEGIN budget$' $T/stream)$(grep -c '^BEGIN two$' $T/stream)$(grep -c '^BEGIN three$' $T/stream)" 100
# the init's own order, as the poller depends on it
ck "the init binds the gadget before its first dump source (g-bind before 'chunk cmdline')" "$(grep -n -e 'gst g-bind' -e '^chunk cmdline' "$INIT" | cut -d: -f1 | tr '\n' ' ' | awk '{print ($1<$2) ? "first" : "wrong"}')" first
if [ $fail -eq 0 ]; then echo "pass walleye-vendor-facts-poll-selftest host $pass ok, $fail failed"; exit 0; fi
echo "fail walleye-vendor-facts-poll-selftest host $pass ok, $fail failed"; exit 1
