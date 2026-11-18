#!/bin/sh
# pon-regs-selftest (backlog walleye-pmic-wdt-readback-image): proves init-z10's `pon-regs` reader and its banner line on the host, never touching the phone.
#   sh tests/host/pon-regs-selftest.sh   (needs no arguments; busybox ash if ~/Projects/walleye-build/busybox-x86_64 exists, else sh)
# The reader script is cut out of init-z10 (the heredoc between "cat > /bin/pon-regs" and its EOS) and run against a fake debugfs tree (DBG=...) shaped like the kernel's regmap debugfs:
# DBG/regmap/NAME/{name,registers}, one "ADDR: VAL" line per register. Checks: values found per regmap, a map without the PON range left out, nothing found says so with the regmap names,
# the reader only reads (no write to the tree), init-z10 arms no PMIC watchdog, and the banner prints the line.
set -u
H=$(cd "$(dirname "$0")" && pwd)
INIT=$H/../../build/initramfs-mainline/init-z10
BBX=$HOME/Projects/walleye-build/busybox-x86_64
SH=sh; [ -x "$BBX" ] && SH="$BBX ash"
[ -f "$INIT" ] || { echo "FAIL no $INIT"; exit 1; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
sed -n '/^cat > \/bin\/pon-regs <<.EOS.$/,/^EOS$/p' "$INIT" | sed '1d;$d' > "$T/pon-regs"
ck "the reader was found in init-z10" "$([ "$(wc -l < "$T/pon-regs")" -gt 5 ] && echo yes || echo no)" yes
mkdir -p "$T/d/regmap/spmi-0" "$T/d/regmap/spmi-2" "$T/d/regmap/other"
echo spmi-0 > "$T/d/regmap/spmi-0/name"; echo spmi-2 > "$T/d/regmap/spmi-2/name"; echo other > "$T/d/regmap/other/name"
reg() { printf '%s\n' "0000: 00" "0800: 11" "0854: 01" "0855: 02" "0856: 07" "0857: 80" "085a: 0c" "085b: 33" "08c5: a5" "0900: ff" > "$1"; }
reg "$T/d/regmap/spmi-0/registers"
printf '%s\n' "0854: 10" "0855: 20" "0856: 30" "0857: 40" "085a: 50" "085b: 60" "08c5: 70" > "$T/d/regmap/spmi-2/registers"
printf '%s\n' "0000: 00" "0100: 01" > "$T/d/regmap/other/registers"
chmod a-w "$T/d/regmap/spmi-0/registers"
touch "$T/mark"; sleep 1
OUT=$(DBG=$T/d NOMOUNT=1 $SH "$T/pon-regs")
ck "first line: the sid-0 regmap's seven registers, in order" "$(printf '%s\n' "$OUT" | sed -n 1p)" "pon-regs 0x54=01 0x55=02 0x56=07 0x57=80 0x5a=0c 0x5b=33 0xc5=a5 src=[spmi-0]"
ck "second line: the other pmic's registers, same form" "$(printf '%s\n' "$OUT" | sed -n 2p)" "pon-regs 0x54=10 0x55=20 0x56=30 0x57=40 0x5a=50 0x5b=60 0xc5=70 src=[spmi-2]"
ck "a regmap without the PON range prints nothing" "$(printf '%s\n' "$OUT" | wc -l | tr -d ' ')" 2
ck "the reader changed no file of the tree (reads only)" "$(cd "$T/d" && find . -type f -newer "$T/mark" | wc -l | tr -d ' ')" 0
mkdir "$T/e"; mkdir -p "$T/e/regmap/other"; echo other > "$T/e/regmap/other/name"; printf '%s\n' "0000: 00" > "$T/e/regmap/other/registers"
ck "no PON registers anywhere: a line that says so and names the regmaps" "$(DBG=$T/e NOMOUNT=1 $SH "$T/pon-regs")" "pon-regs none regmaps=[other] dbgfs=[yes]"
mkdir "$T/f"
ck "no regmap directory: says so" "$(DBG=$T/f NOMOUNT=1 $SH "$T/pon-regs")" "pon-regs none regmaps=[] dbgfs=[no]"
# the banner part
sed -n '/^      P=\$(cat \$PONF/,/WALLEYE-BANNER-PON/p' "$INIT" | sed 's/^      //' > "$T/bannerpart.sh"
ck "the banner part was found in init-z10 (2 lines)" "$(wc -l < "$T/bannerpart.sh" | tr -d ' ')" 2
printf '%s\n' "pon-regs 0x54=01 src=[spmi-0]" "pon-regs 0x54=10 src=[spmi-2]" > "$T/pon"
L=$( ( TTY=/dev/stdout; b=4; PONF=$T/pon; . "$T/bannerpart.sh" ) | grep WALLEYE-BANNER-PON )
ck "the banner line: starts WALLEYE-BANNER-PON, one line, ends with ]" "$L" "WALLEYE-BANNER-PON beat=4 [pon-regs 0x54=01 src=[spmi-0] pon-regs 0x54=10 src=[spmi-2]]"
L=$( ( TTY=/dev/stdout; b=1; PONF=$T/none; . "$T/bannerpart.sh" ) | grep WALLEYE-BANNER-PON )
ck "no reading yet: the banner says not read yet, not missing" "$L" "WALLEYE-BANNER-PON beat=1 [pon-regs not read yet]"
# the init arms nothing
ck "init-z10 never opens a second watchdog or writes a PMIC register (no watchdog1, no S2, no regmap write)" "$(grep -v '^[[:space:]]*#' "$INIT" | grep -c -e 'watchdog1' -e 'regmap/[^ ]* *>' -e '> */sys/kernel/debug' -e 'timeout *=' -e 'echo [0-9a-fx]* *> *[^ ]*/registers')" 0
ck "init-z10 petter is the SoC watchdog's only, self-reboot after 90 beats kept" "$(grep -c 'while \[ \$n -lt 90 \]' "$INIT")" 1
ck "init-z10 busybox ash -n" "$($SH -n "$INIT" 2>&1 || echo syntax-error)" ""
[ $total -eq 0 ] || { echo "pon-regs-selftest: $total FAIL"; exit 1; }
echo "pon-regs-selftest: all ok"
