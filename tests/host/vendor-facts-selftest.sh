#!/bin/sh
# walleye-vendor-facts-selftest, host only (no phone): builds a window file the way the vf1 init does (64-byte header + gzipped tar in an 8 MiB file), runs vendor-facts-read.sh on it (VF_FILE),
# and checks the three outcomes the reader must have: a good dump passes and lands where VF_OUT says; a dump with a MAC-shaped word is refused and deleted; a torn stream (header longer than the body) fails.
# Prints ok/FAIL lines and `pass|fail walleye-vendor-facts-selftest host N ok, M failed`; exit 0 only when none failed.
HERE=$(cd "$(dirname "$0")" && pwd)
RETRY_WAIT=0; export RETRY_WAIT
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
fail=0; pass=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
mkwin() { # mkwin DIR WINDOW [LENFIX]: tar+gzip DIR into the window
  tar cf - -C "$1" . | gzip -c > "$T/b.tgz"
  L=$(stat -c %s "$T/b.tgz"); [ -n "$3" ] && L=$((L+$3))
  if [ -n "$3" ]; then tr "\\000" x < /dev/zero | head -c 8388608 > "$2"; else dd if=/dev/zero of="$2" bs=1024 count=8192 2>/dev/null; fi
  dd if="$T/b.tgz" of="$2" bs=64 seek=1 conv=notrunc 2>/dev/null
  printf '%-63s\n' "$(printf 'WALLEYE-VF1 LEN=%010d ROUND=%02d' "$L" 3)" | dd of="$2" bs=64 count=1 conv=notrunc 2>/dev/null
}
good=$T/good; mkdir -p $good/dt/soc $good/debugfs/pinctrl $good/debugfs/dwc3 $good/sys
printf 'walleye\0' > $good/dt/model; printf '\0\0\0\1' > $good/dt/soc/reg
echo "[    0.0] Booting Linux on physical CPU 0x0" > $good/dmesg
echo "00000000-1fffffff : System RAM" > $good/iomem; echo "  1: 0 GIC" > $good/interrupts
echo "androidboot.serialno=REDACTED console=ttyMSM0" > $good/cmdline
echo "clk summary" > $good/debugfs/clk_summary; echo "reg summary" > $good/debugfs/regulator_summary; echo "gpio-0" > $good/debugfs/gpio
echo "maps" > $good/debugfs/pinctrl/pinctrl-maps; echo "ep0" > $good/debugfs/dwc3/ep0; echo "udc" > $good/sys/udc-list
mkwin $good $T/w1
VF_FILE=$T/w1 VF_OUT=$T/out1 VF_TRIES=2 sh "$HERE/vendor-facts-read.sh" > $T/r1 2>&1; r=$?
ck "a good window passes" "$r" 0
ck "the dump has the device tree and dmesg, index first line says nothing missing" "$(test -s $T/out1/dt/model && test -s $T/out1/dmesg && head -n1 $T/out1/INDEX.txt | grep -c 'missing: nothing')" 1
bad=$T/bad; cp -r $good $bad; echo "wlan0 link/ether 02:11:22:33:44:55" >> $bad/dmesg
mkwin $bad $T/w2
VF_FILE=$T/w2 VF_OUT=$T/out2 VF_TRIES=2 sh "$HERE/vendor-facts-read.sh" > $T/r2 2>&1; r=$?
ck "a window with a MAC-shaped word is refused" "$r" 1
ck "the refused dump is deleted" "$(test -e $T/out2 && echo kept || echo deleted)" deleted
bad2=$T/bad2; cp -r $good $bad2; touch $bad2/dt/local-mac-address
mkwin $bad2 $T/w3
VF_FILE=$T/w3 VF_OUT=$T/out3 VF_TRIES=2 sh "$HERE/vendor-facts-read.sh" > $T/r3 2>&1; r=$?
ck "a device-tree property named *mac* is refused" "$r" 1
mkwin $good $T/w4 2000
VF_FILE=$T/w4 VF_OUT=$T/out4 VF_TRIES=2 sh "$HERE/vendor-facts-read.sh" > $T/r4 2>&1; r=$?
ck "a torn stream (header longer than the body) fails" "$r" 1
printf 'garbage' | dd of=$T/w5 bs=1024 count=8192 conv=sync 2>/dev/null
VF_FILE=$T/w5 VF_OUT=$T/out5 VF_TRIES=1 sh "$HERE/vendor-facts-read.sh" > $T/r5 2>&1; r=$?
ck "a window with no WALLEYE-VF1 header fails" "$r" 1
bad3=$T/bad3; cp -r $good $bad3; printf "console=x androidboot.serialno=RZ8M123456 y" > $bad3/dt/chosen-bootargs; mkdir -p $bad3/dt/chosen; mv $bad3/dt/chosen-bootargs $bad3/dt/chosen/bootargs
mkwin $bad3 $T/w6
VF_FILE=$T/w6 VF_OUT=$T/out6 VF_TRIES=2 sh "$HERE/vendor-facts-read.sh" > $T/r6 2>&1; r=$?
ck "a serial in dt/chosen/bootargs is refused and nothing is left" "$r$(test -e $T/out6 && echo kept)" 1
bad4=$T/bad4; cp -r $good $bad4; echo "link 02-11-22-33-44-55" >> $bad4/dmesg
mkwin $bad4 $T/w7
VF_FILE=$T/w7 VF_OUT=$T/out7 VF_TRIES=2 sh "$HERE/vendor-facts-read.sh" > $T/r7 2>&1; r=$?
ck "a dash-form MAC is refused" "$r" 1
# ---- vf2: the window with a text region first and the blob at 65536
mkwin2() { # mkwin2 DIR WINDOW TEXTFILE [noblob]: 8 MiB window; text region (NUL padded to 64 KiB) from TEXTFILE; blob at 65536 unless noblob
  dd if=/dev/zero of="$2" bs=1024 count=8192 2>/dev/null
  dd if="$3" of="$2" bs=65536 count=1 conv=sync,notrunc 2>/dev/null
  [ "$4" = noblob ] && return 0
  tar cf - -C "$1" . | gzip -c > "$T/b2.tgz"; L=$(stat -c %s "$T/b2.tgz")
  dd if="$T/b2.tgz" of="$2" bs=64 seek=1025 conv=notrunc 2>/dev/null
  printf '%-63s\n' "$(printf 'WALLEYE-VF2-BLOB LEN=%010d ROUND=%02d' "$L" 4)" | dd of="$2" bs=64 seek=1024 count=1 conv=notrunc 2>/dev/null
}
printf 'WALLEYE-VF2 BEAT=7 UP=31.2\nstart 0 init (kernel 4.4.302)\napplet tar ok\nstart 1 mount-sysfs\nstep 1 mount-sysfs rc=0\nstart 2 mount-debugfs\nstep 2 mount-debugfs rc=255: mount: no such device\nstart 3 tar\n' > $T/txt
mkwin2 $good $T/v1 $T/txt
VF_FILE=$T/v1 VF_OUT=$T/outv1 VF_TRIES=2 sh "$HERE/vendor-facts-read.sh" > $T/rv1 2>&1; r=$?
ck "vf2: a good window passes" "$r" 0
ck "vf2: the step lines are printed before the result line" "$(grep -n -e 'step 2 mount-debugfs rc=255' -e '^pass walleye-vendor-facts-read' $T/rv1 | cut -d: -f1 | tr '\n' ' ' | awk '{print ($1<$2) ? "first" : "wrong"}')" first
ck "vf2: the dump lands and its index says vf2" "$(test -s $T/outv1/dmesg && head -n1 $T/outv1/INDEX.txt | grep -c '^# vf2 dump')" 1
mkwin2 $good $T/v2 $T/txt noblob
VF_FILE=$T/v2 VF_OUT=$T/outv2 VF_TRIES=2 sh "$HERE/vendor-facts-read.sh" > $T/rv2 2>&1; r=$?
ck "vf2: the marker with no blob fails" "$r" 1
ck "vf2: and says the marker, the steps ended, the failed rc and the last line" "$(grep -c -e 'fail walleye-vendor-facts-read.*marker WALLEYE-VF2 BEAT=7.*2 steps ended, 1 with a non-zero rc, 0 applets missing; last line: start 3 tar' $T/rv2)" 1
ck "vf2: nothing is left in the output directory of a failed read" "$(test -e $T/outv2 && echo kept || echo none)" none
mkwin2 $bad $T/v3 $T/txt
VF_FILE=$T/v3 VF_OUT=$T/outv3 VF_TRIES=2 sh "$HERE/vendor-facts-read.sh" > $T/rv3 2>&1; r=$?
ck "vf2: a blob with a MAC-shaped word is refused and deleted" "$r$(test -e $T/outv3 && echo kept)" 1
printf 'applet sed MISSING\nstart 1 x\n' > $T/txt2; sed -i '1i WALLEYE-VF2 BEAT=1 UP=2.0' $T/txt2
mkwin2 $good $T/v4 $T/txt2 noblob
VF_FILE=$T/v4 VF_OUT=$T/outv4 VF_TRIES=1 sh "$HERE/vendor-facts-read.sh" > $T/rv4 2>&1
ck "vf2: a missing applet is counted in the summary" "$(grep -c '1 applets missing' $T/rv4)" 1
# ---- vf2 init: its stp/paint/note functions run on the host against a window file, as the phone's init runs them
INIT=$HERE/../../build/initramfs-vendor5/init-vf2
a=$(grep -n '^say() ' "$INIT" | cut -d: -f1); b=$(grep -n '^say "vendor-facts-vf2 init start' "$INIT" | cut -d: -f1)
{ echo 'say() { :; }'; sed -n "$((a+1)),$((b-1))p" "$INIT" | sed "s|/tmp/walleye-|$T/walleye-|g"; grep '^grab() ' "$INIT"; cat <<'EOS'
dd if=/dev/zero of=$LOGF bs=1024 count=8192 2>/dev/null
: > $STAT; echo 0 > /tmp/walleye-beat
paint
stp good true
stp bad sh -c 'echo boom >&2; exit 3'
stp missing nonexistent-command-xyz
touch /tmp/walleye-unread; chmod 000 /tmp/walleye-unread
stp gf grab /tmp/walleye-unread /tmp/walleye-out
EOS
} | sed "s|/tmp/walleye-|$T/walleye-|g" > $T/initfn.sh
sh $T/initfn.sh > $T/initfn.out 2>&1
dd if=$T/walleye-log.img bs=65536 count=1 2>/dev/null | tr -d '\000' > $T/initwin
ck "init-vf2: line 1 of the window is the marker with a beat number" "$(head -n1 $T/initwin | grep -c '^WALLEYE-VF2 BEAT=[0-9]')" 1
ck "init-vf2: a good step has its start and its rc=0 line" "$(grep -c -e '^start 1 good$' -e '^step 1 good rc=0$' $T/initwin)" 2
ck "init-vf2: a failing step's real rc and stderr are written" "$(grep -c '^step 2 bad rc=3: boom' $T/initwin)" 1
ck "init-vf2: a missing command is written as a failed step, not hidden" "$(grep -c '^step 3 missing rc=127' $T/initwin)" 1
ck "init-vf2: a step in which a grab failed ends with a non-zero rc naming the file, not rc=0 (skipped as root)" "$([ $(id -u) -eq 0 ] && echo 1 || grep -c '^step 4 gf rc=1: .*walleye-unread' $T/initwin)" 1
ck "init-vf2: the window is 8 MiB and its blob header area (65536) is untouched" "$(stat -c %s $T/walleye-log.img) $(dd if=$T/walleye-log.img bs=64 skip=1024 count=1 2>/dev/null | tr -d '\000' | wc -c)" "8388608 0"
printf 'garbage, not ours' | dd of=$T/v5 bs=1024 count=8192 conv=sync 2>/dev/null
VF_FILE=$T/v5 VF_OUT=$T/outv5 VF_TRIES=1 sh "$HERE/vendor-facts-read.sh" > $T/rv5 2>&1; r=$?
ck "a window with no marker at all says so" "$r$(grep -c 'there is no marker' $T/rv5)" 11
VF_FILE=$T/v2 VF_OUT=$T/outv2b VF_TRIES=3 sh "$HERE/vendor-facts-read.sh" > $T/rv6 2>&1
ck "vf2: the marker with no blob is retried and, the beat frozen, called dead or hung" "$(grep -c 'DEAD or hung (beat frozen at 7' $T/rv6)" 1
if [ $fail -eq 0 ]; then echo "pass walleye-vendor-facts-selftest host $pass ok, $fail failed"; exit 0; fi
echo "fail walleye-vendor-facts-selftest host $pass ok, $fail failed"; exit 1
