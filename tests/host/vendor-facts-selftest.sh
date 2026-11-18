#!/bin/sh
# walleye-vendor-facts-selftest, host only (no phone): builds a window file the way the vf1 init does (64-byte header + gzipped tar in an 8 MiB file), runs vendor-facts-read.sh on it (VF_FILE),
# and checks the three outcomes the reader must have: a good dump passes and lands where VF_OUT says; a dump with a MAC-shaped word is refused and deleted; a torn stream (header longer than the body) fails.
# Prints ok/FAIL lines and `pass|fail walleye-vendor-facts-selftest host N ok, M failed`; exit 0 only when none failed.
HERE=$(cd "$(dirname "$0")" && pwd)
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
if [ $fail -eq 0 ]; then echo "pass walleye-vendor-facts-selftest host $pass ok, $fail failed"; exit 0; fi
echo "fail walleye-vendor-facts-selftest host $pass ok, $fail failed"; exit 1
