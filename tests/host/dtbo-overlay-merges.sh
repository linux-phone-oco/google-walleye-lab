#!/bin/sh
# walleye-dtbo-overlay-merges, run on the lab host: does the vendor dtbo overlay the bootloader picks (androidboot.dtbo_idx=3) merge
# into a base dtb? Passes only when `fdtoverlay` applies that overlay to the dtb named by DTB (default: the lab's built
# msm8998-google-walleye.dtb). The stock dtb passes; our dtb fails until it carries __symbols__ with the labels the overlay's
# __fixups__ name. Prints one line `pass|fail|skip walleye-dtbo-overlay-merges kernel DETAIL`; exit 0 only on pass. Writes only a temp file.
# Environment: DTB (base dtb), DTBO (dtbo.img), DTBO_IDX (entry, default 3).
DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
DTB=${DTB:-$HOME/Projects/walleye-build/out/msm8998-google-walleye.dtb}
IDX=${DTBO_IDX:-3}
say() { echo "$1 walleye-dtbo-overlay-merges kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtoverlay >/dev/null || say skip "fdtoverlay (dtc) not installed"
[ -r "$DTBO" ] && [ -r "$DTB" ] || say skip "need DTBO=$DTBO and DTB=$DTB readable"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT
# dtbo table: 32-byte header, then 32-byte entries (size, offset, id, rev, ...) big-endian; entry IDX at 0x20+32*IDX
e=$(xxd -s $((32+32*IDX)) -l 8 -p "$DTBO") || say skip "cannot read the dtbo table"
size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
[ "$size" -gt 0 ] || say skip "dtbo entry $IDX is empty"
dd if="$DTBO" of="$T/ov" bs=1 skip="$off" count="$size" 2>/dev/null
out=$(fdtoverlay -i "$DTB" -o "$T/merged" "$T/ov" 2>&1) && say pass "overlay $IDX merged into $(basename "$DTB")"
say fail "overlay $IDX does not merge into $(basename "$DTB"): $(echo "$out" | tr '\n' ' ' | cut -c1-160)"
