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
# the kernel has the ramoops backend: the parameter names of pstore/ram (ramoops.console_size ...) are in its decompressed Image. This replaces "the kernel is b-r5's" (2026-11-28, item
# walleye-display-identity-client-list-image: B-r5-smmu-id's kernel is a new build, so its bytes are not b-r5's); the backend itself is what the stock layout needs. The unpacked kernel part ends
# with the appended dtb: gzip reads the stream and says so after its end, which is not a failure here.
if [ -r "$T/p/kernel" ]; then
  n=$(gzip -dc < "$T/p/kernel" 2>/dev/null | grep -ac 'ramoops\.console_size')
  [ "${n:-0}" -ge 1 ] || say fail "the kernel part names no ramoops.console_size parameter: the ramoops backend is not shown"
fi
say pass "$IN: ramoops at $got_reg, record $want_record, console $want_console, pmsg $want_pmsg, no ecc: stock's layout"
