#!/bin/sh
# walleye-image-ramoops-stock-layout, run on the lab host (item walleye-display-image-console-in-ramoops; decision 2026-11-28-walleye-display-try-needs-its-console-first):
# does the dtb of IMAGE|DTB lay the ramoops region out as stock Android lays it, so the stock system reads our kernel's console back after a warm reset?
# Expected values come from the captured stock dtbo text tests/fixtures/dtb-text/stock-dtbo3-ramoops.txt (the region's address and size, record-size, console-size, pmsg-size);
# the stock node has no ecc-size, so ours must have none. Usage: image-ramoops-stock-layout.sh IMAGE|DTB. Prints `pass|fail|skip walleye-image-ramoops-stock-layout DETAIL`; exit 0 only on pass.
# Environment: STOCK (the captured text, default beside this script's fixtures).
HERE=$(cd "$(dirname "$0")" && pwd)
STOCK=${STOCK:-$HERE/../fixtures/dtb-text/stock-dtbo3-ramoops.txt}
IN=${1:-}
say() { echo "$1 walleye-image-ramoops-stock-layout $2"; [ "$1" = pass ]; exit $?; }
[ -n "$IN" ] || say fail "usage: image-ramoops-stock-layout.sh IMAGE|DTB"
for c in fdtget unpack_bootimg awk; do command -v "$c" >/dev/null || say skip "$c not installed: not known yet"; done
[ -r "$IN" ] || say skip "no file at $IN: not known yet"
[ -r "$STOCK" ] || say fail "no captured stock text at $STOCK"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT
if [ "$(head -c 8 "$IN" | tr -d '\000')" = "ANDROID!" ]; then
  unpack_bootimg --boot_img "$IN" --out "$T/u" > "$T/info" 2>&1 || say fail "unpack_bootimg cannot read $IN"
  # the dtb appended to the kernel: the bootloader picks it; lpoco-bringup's unpack writes it as appended.dtb, so take the magic d00dfeed run after the kernel
  command -v lpoco-bringup >/dev/null || say skip "lpoco-bringup not installed: not known yet"
  lpoco-bringup image unpack "$IN" --device "${DEVICE_TOML:-device.toml}" --out "$T/p" > /dev/null 2>&1 || say fail "lpoco-bringup cannot unpack $IN"
  DTB=$T/p/appended.dtb
else
  DTB=$IN
fi
[ -r "$DTB" ] || say fail "no dtb read from $IN"
# a value of the stock text: the last cell of PROP's line (the stock properties are two cells, high zero; ours are one cell, as the mainline driver reads u32)
stock() { awk -v p="$1" '$1==p { gsub(/[<>;]/,""); print $NF; exit }' "$STOCK" | sed 's/^0x0*//'; }
want_record=$(stock record-size); want_console=$(stock console-size); want_pmsg=$(stock pmsg-size)
[ -n "$want_record" ] && [ -n "$want_console" ] && [ -n "$want_pmsg" ] || say fail "the captured stock text names no record, console or pmsg size"
# the region's address and size: the stock ramoops_region's reg, its last two cells
want_reg=$(awk '/^ *ramoops_region \{/ {f=1} f && $1=="reg" { gsub(/[<>;]/,""); print $4, $6; exit }' "$STOCK" | sed 's/0x0*//g')
[ -n "$want_reg" ] || say fail "the captured stock text names no ramoops_region reg"
N=/reserved-memory/ramoops@a1100000
fdtget "$DTB" "$N" compatible > /dev/null 2>&1 || say fail "no $N in the dtb"
got_reg=$(fdtget -t x "$DTB" "$N" reg | awk '{ print $2, $4 }')
[ "$got_reg" = "$want_reg" ] || say fail "reg is $got_reg, stock is $want_reg"
for p in record:$want_record console:$want_console pmsg:$want_pmsg; do
  name=${p%%:*}; want=${p#*:}
  got=$(fdtget -t x "$DTB" "$N" "$name-size" 2>/dev/null || echo none)
  [ "$got" = "$want" ] || say fail "$name-size is $got, stock is $want"
done
fdtget -d none "$DTB" "$N" ecc-size > "$T/ecc" 2>&1
[ "$(cat "$T/ecc")" = none ] || say fail "ecc-size is set ($(cat "$T/ecc")), stock has none"
# the kernel has the ramoops backend: an image's kernel is b-r5's (the recipe changes only dtb and command line); b-r5's captured boot registers ramoops (research walleye-display-reset-image-b-r5-tried-signs-vanish)
if [ -r "$T/p/kernel" ]; then
  B5=$(lpoco-bringup image ensure b-r5 --device "${DEVICE_TOML:-device.toml}" --dry-run 2>&1 | sed -n 's/.*(\(\/[^)]*\))$/\1/p' | tail -n 1)
  [ -r "$B5" ] || say skip "b-r5 is not in the store: not known yet"
  lpoco-bringup image unpack "$B5" --device "${DEVICE_TOML:-device.toml}" --out "$T/b5" > /dev/null 2>&1 || say fail "cannot unpack b-r5"
  # the unpacked kernel part ends with the appended dtb: compare the bytes before it (boot.toml says where the dtb is appended)
  kpart() { n=$(sed -n 's/.*appended to the kernel at \([0-9]*\)$/\1/p' "$1/boot.toml"); [ -n "$n" ] && head -c "$n" "$1/kernel"; }
  [ "$(kpart "$T/b5" | sha256sum)" = "$(kpart "$T/p" | sha256sum)" ] || say fail "the kernel is not b-r5's: ramoops backend not shown"
fi
say pass "$IN: ramoops at $got_reg, record $want_record, console $want_console, pmsg $want_pmsg, no ecc: stock's layout"
