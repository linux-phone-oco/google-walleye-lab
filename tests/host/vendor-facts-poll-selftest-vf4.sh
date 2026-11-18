#!/bin/sh
# walleye-vendor-facts-poll-selftest-vf4, host only (no phone): runs the real functions of build/initramfs-vendor5/init-vf4 (flush, chunk, dtnode, cat_file, redact_filter) against a vf2-shaped window (8 MiB, a 64 KiB
# text region, the stream from byte 65536), then runs tests/host/vendor-facts-poll.sh on it with the vf4 layout (VF_MARK=WALLEYE-VF2 VF_HEAD=16 VF_BLOCKS=1040) and checks: the stream is in the window byte for byte from
# 65536 and never touches the text region; the poller fetches finished blocks whole and keeps the text region as steps.txt; a torn block is not fetched; DONE stops it; a MAC-shaped word in the stream or in
# the text region is refused and the output deleted. Prints ok/FAIL lines and `pass|fail walleye-vendor-facts-poll-selftest-vf4 host N ok, M failed`; exit 0 only when none failed.
HERE=$(cd "$(dirname "$0")" && pwd)
INIT=$HERE/../../build/initramfs-vendor5/init-vf4
POLL=$HERE/vendor-facts-poll.sh
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0; pass=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
fn() { awk -v n="$1" '$0 ~ "^" n "\\(\\) \\{" { print; if ($0 !~ /\}$/ || $0 ~ /\{$/) inside = 1; next } inside { print; if ($0 == "}") inside = 0 }' "$INIT"; }
{ for f in flush redact_filter chunk dtnode cat_file cat_dir; do fn $f; done; } | sed "s|/tmp/walleye-|$T/walleye-|g" > $T/fns.sh
ck "the init's functions were all found" "$(grep -c -E '^(flush|redact_filter|chunk|dtnode|cat_file|cat_dir)\(\) \{' $T/fns.sh)" 6
cat > $T/harness.sh <<EOS
LOGF=$T/win.img; OUT=$T/stream; HEADBLK=16; BUDGET=3800000; SPENT=0; PERSRC=262144; BLOCK=4096; WROTE=0
note() { echo "\$*" >> $T/steps; }
. $T/fns.sh
dd if=/dev/zero of=\$LOGF bs=1024 count=8192 2>/dev/null; : > \$OUT
{ printf 'WALLEYE-VF2 BEAT=3 UP=44.5\n'; echo "step 1 g-bind rc=0"; } | dd of=\$LOGF bs=65536 count=1 conv=sync,notrunc 2>/dev/null
EOS
printf 'console=ttyMSM0 androidboot.serialno=RZ8M123456 mac 02:11:22:33:44:55 imei 123456789012345\n' > $T/src1
i=0; : > $T/src2; while [ $i -lt 900 ]; do echo "line $i of the big source, padded to make the stream cross a few 4096-byte boundaries ..........." >> $T/src2; i=$((i+1)); done
failing() { echo partial; echo boom >&2; return 4; }
{ echo "failing() { echo partial; echo boom >&2; return 4; }"; cat <<EOS
. $T/harness.sh
chunk cmdline cat_file $T/src1
chunk big cat_file $T/src2
chunk bad failing
echo "DONE up=1 bytes=\$(stat -c %s \$OUT)" >> \$OUT; flush
EOS
} > $T/run1.sh
sh $T/run1.sh > $T/run1.out 2>&1
SZ=$(stat -c %s $T/stream)
ck "the window is 8 MiB and the stream (over several 4096 boundaries) is in it byte for byte from 65536" "$(stat -c %s $T/win.img) $(dd if=$T/win.img bs=65536 skip=1 2>/dev/null | head -c $SZ | cmp - $T/stream && echo same)" "8388608 same"
ck "the text region (first 64 KiB) is untouched by the stream" "$(dd if=$T/win.img bs=65536 count=1 2>/dev/null | tr -d '\000' | head -n1)" "WALLEYE-VF2 BEAT=3 UP=44.5"
ck "the init's redaction took the serialno, the MAC and the 15 digits out of the stream" "$(grep -c -e 'serialno=REDACTED' -e 'XX:XX:XX:XX:XX:XX' -e 'IMEI-SHAPED' $T/stream)$(grep -c -e 'RZ8M123456' -e '02:11:22:33:44:55' -e '123456789012345' $T/stream)" 10
ck "a failing source is framed with its real rc and the first words of its stderr" "$(grep -c '^END bad rc=4: boom' $T/stream)" 1
ck "each chunk put a 'chunk NAME' line in the text region first" "$(grep -c '^chunk ' $T/steps)" 3
PV="VF_MARK=WALLEYE-VF2 VF_HEAD=16 VF_BLOCKS=1040"
env VF_MARK=WALLEYE-VF2 VF_HEAD=16 VF_BLOCKS=1040 VF_FILE=$T/win.img VF_OUT=$T/o1 POLL_ONCE=1 sh $POLL > $T/p1.out 2>&1; r=$?
ck "the poller passes on a window that carries DONE" "$r" 0
ck "it fetched the 3 finished blocks and says DONE" "$(grep -c -e '^pass walleye-vendor-facts-poll host 3 blocks in .*, DONE' $T/p1.out)" 1
ck "each block is a file whose last line is its END line; the big one is whole (900 lines)" "$(tail -n1 $T/o1/vf/cmdline | cut -c1-11)|$(tail -n1 $T/o1/vf/big | cut -c1-7)|$(grep -c '^line ' $T/o1/vf/big)" "END cmdline|END big|900"
ck "the text region is kept as steps.txt" "$(head -n1 $T/o1/steps.txt)" "WALLEYE-VF2 BEAT=3 UP=44.5"
# torn: the stream cut inside the last block
head -c 6000 $T/stream | sed '$d' > $T/torn.txt
dd if=/dev/zero of=$T/win-torn.img bs=1024 count=8192 2>/dev/null
dd if=$T/win.img of=$T/win-torn.img bs=65536 count=1 conv=notrunc 2>/dev/null
dd if=$T/torn.txt of=$T/win-torn.img bs=65536 seek=1 conv=notrunc 2>/dev/null
env VF_MARK=WALLEYE-VF2 VF_HEAD=16 VF_BLOCKS=1040 VF_FILE=$T/win-torn.img VF_OUT=$T/o2 POLL_ONCE=1 sh $POLL > $T/p2.out 2>&1
ck "a torn window: only finished blocks are fetched, no DONE is said" "$(ls $T/o2/vf 2>/dev/null | wc -l | tr -d ' ')|$(grep -c 'no DONE, partial' $T/p2.out)" "1|1"
# a leak in the text region
cp $T/win.img $T/win-leak.img
printf 'step 9 x mac 02:11:22:33:44:55\n' | dd of=$T/win-leak.img bs=1 seek=200 conv=notrunc 2>/dev/null
env VF_MARK=WALLEYE-VF2 VF_HEAD=16 VF_BLOCKS=1040 VF_FILE=$T/win-leak.img VF_OUT=$T/o3 POLL_ONCE=1 sh $POLL > $T/p3.out 2>&1; r=$?
ck "a MAC-shaped word in the text region: fail, and the output is deleted" "$r$(test -e $T/o3 && echo exists)" 1
# no marker (a vf3 window read with the vf4 layout)
printf 'WALLEYE-VF3 BEAT=1 UP=2\n' | dd of=$T/win-m.img bs=4096 count=1 conv=sync 2>/dev/null; dd if=/dev/zero bs=4096 count=1100 2>/dev/null >> $T/win-m.img
env VF_MARK=WALLEYE-VF2 VF_HEAD=16 VF_BLOCKS=1040 VF_FILE=$T/win-m.img VF_OUT=$T/o4 POLL_ONCE=1 sh $POLL > $T/p4.out 2>&1; r=$?
ck "no vf2 marker: fail and nothing left" "$r$(test -e $T/o4 && echo exists)" 1
if [ $fail -eq 0 ]; then echo "pass walleye-vendor-facts-poll-selftest-vf4 host $pass ok, 0 failed"; exit 0; fi
echo "fail walleye-vendor-facts-poll-selftest-vf4 host $pass ok, $fail failed"; exit 1
