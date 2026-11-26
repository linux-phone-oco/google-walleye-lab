#!/bin/sh
# pon-block-selftest (item walleye-wdt-pon-control-procedure): proves on the host, never touching the phone, that /bin/pon-block of the pon-read image reads the PON block 0xC0..0xCB raw and only reads.
#   sh tests/host/pon-block-selftest.sh   (needs no arguments; busybox ash if ~/Projects/walleye-build/busybox-x86_64 exists, else sh)
# The reader runs against a fake debugfs tree (DBG=...) shaped like the kernel's regmap debugfs (DBG/regmap/NAME/{name,registers}, one "ADDR: VAL" line per register, as drivers/base/regmap/regmap-debugfs.c prints,
# the shape pon-regs-selftest.sh uses): values found per regmap, a map without the PON range left out, nothing found says so with the regmap names, a read that failed (XX) is ?? and not a number, nothing in the tree changes.
# Also: the fixture of the steps crate is the first line this reader prints on this tree; the reader writes through no variable.
set -u
H=$(cd "$(dirname "$0")" && pwd)
READER=$H/../../build/initramfs-mainline/pon-block
BBX=$HOME/Projects/walleye-build/busybox-x86_64
SH=sh; [ -x "$BBX" ] && SH="$BBX ash"
[ -f "$READER" ] || { echo "FAIL no $READER"; exit 1; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
mkdir -p "$T/d/regmap/spmi-0" "$T/d/regmap/spmi-2" "$T/d/regmap/other" "$T/d/regmap/xx"
echo spmi-0 > "$T/d/regmap/spmi-0/name"; echo spmi-2 > "$T/d/regmap/spmi-2/name"; echo other > "$T/d/regmap/other/name"; echo xx > "$T/d/regmap/xx/name"
i=0; while [ $i -le 2303 ]; do printf "%04x: 00\n" $i; i=$((i+1)); done | sed -e "s/^08c0: 00/08c0: 01/" -e "s/^08c2: 00/08c2: 80/" -e "s/^08c5: 00/08c5: 04/" -e "s/^08c7: 00/08c7: 02/" -e "s/^08cb: 00/08cb: ff/" > "$T/d/regmap/spmi-0/registers"
printf '%s\n' "08c0: 10" "08c1: 11" "08c2: 12" "08c3: 13" "08c4: 14" "08c5: 15" "08c6: 16" "08c7: 17" "08c8: 18" "08c9: 19" "08ca: 1a" "08cb: 1b" > "$T/d/regmap/spmi-2/registers"
printf '%s\n' "0000: 00" "0100: 01" > "$T/d/regmap/other/registers"
printf '%s\n' "08c0: XX" "08c1: XX" > "$T/d/regmap/xx/registers"
chmod a-w "$T/d/regmap/spmi-0/registers"
touch "$T/mark"; sleep 1
OUT=$(DBG=$T/d NOMOUNT=1 $SH "$READER")
ck "first line: the sid-0 regmap's twelve registers, in order, raw" "$(printf '%s\n' "$OUT" | sed -n 1p)" "pon-block 0xc0=01 0xc1=00 0xc2=80 0xc3=00 0xc4=00 0xc5=04 0xc6=00 0xc7=02 0xc8=00 0xc9=00 0xca=00 0xcb=ff src=[spmi-0]"
ck "second line: the other pmic's registers, same form" "$(printf '%s\n' "$OUT" | sed -n 2p)" "pon-block 0xc0=10 0xc1=11 0xc2=12 0xc3=13 0xc4=14 0xc5=15 0xc6=16 0xc7=17 0xc8=18 0xc9=19 0xca=1a 0xcb=1b src=[spmi-2]"
ck "a regmap without the PON range prints nothing" "$(printf '%s\n' "$OUT" | grep -c 'src=\[other\]')" 0
ck "a regmap whose reads all failed (XX) prints nothing, not zeros" "$(printf '%s\n' "$OUT" | grep -c 'src=\[xx\]')" 0
ck "the steps crate fixture is the first line the reader printed here" "$(cat "$H/../../procedures/steps/fixtures/pon-block-reader-output.txt")" "$(printf "%s\n" "$OUT" | sed -n 1p)"
ck "two lines in all" "$(printf '%s\n' "$OUT" | wc -l | tr -d ' ')" 2
ck "the reader changed no file of the tree (reads only)" "$(cd "$T/d" && find . -type f -newer "$T/mark" | wc -l | tr -d ' ')" 0
mkdir -p "$T/p/regmap/m"; echo m > "$T/p/regmap/m/name"
printf '%s\n' "08c0: 01" "08c1: XX" "08c2: 03" > "$T/p/regmap/m/registers"
ck "a register that failed to read is ?? in a line whose others read" "$(DBG=$T/p NOMOUNT=1 $SH "$READER" | cut -d' ' -f1-4)" "pon-block 0xc0=01 0xc1=?? 0xc2=03"
mkdir -p "$T/e/regmap/other"; echo other > "$T/e/regmap/other/name"; printf '%s\n' "0000: 00" > "$T/e/regmap/other/registers"
ck "no PON registers anywhere: a line that says so and names the regmaps" "$(DBG=$T/e NOMOUNT=1 $SH "$READER")" "pon-block none regmaps=[other] dbgfs=[yes]"
mkdir "$T/f"
ck "no regmap directory: says so" "$(DBG=$T/f NOMOUNT=1 $SH "$READER")" "pon-block none regmaps=[] dbgfs=[no]"
ck "the reader writes no register (no redirect into a variable or a registers file, no tee, no debugfs write)" "$(grep -v '^[[:space:]]*#' "$READER" | grep -c -e '>[^&]*\$' -e '>[^&]*/registers' -e 'tee ' -e '> */sys/kernel/debug')" 0
ck "busybox ash -n" "$($SH -n "$READER" 2>&1 || echo syntax-error)" ""
[ $total -eq 0 ] || { echo "pon-block-selftest: $total FAIL"; exit 1; }
echo "pon-block-selftest: all ok"
