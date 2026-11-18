#!/bin/sh
# walleye-dmesg-clean, run on the lab host: after the bootloader's overlay merge, do the overlay's clock users point at the REAL gcc?
# The overlay's serial@c16f000, i2c@c179000 (and others) say `clocks = <&clock_gcc ID ...>`. If clock_gcc is an empty stub (no #clock-cells) the
# kernel reads the clock ID as a phandle ("OF: ...: could not find phandle -1934992561") and the serial waits on the stub as its supplier.
# Passes only when, in DTB merged with the dtbo entry the bootloader picks, the first clocks cell of those two nodes is the phandle of /soc@0/clock-controller@100000.
# Prints `pass|fail|skip walleye-dmesg-clean kernel DETAIL`; exit 0 only on pass. Writes only a temp file.
# Environment: DTB (our dtb, default the built z dtb), DTBO (dtbo.img), DTBO_IDX (default 3).
W=$HOME/Projects/walleye-build
DTBO=${DTBO:-$W/control/dtbo.img}
DTB=${DTB:-$W/images-z/msm8998-google-walleye-z.dtb}
IDX=${DTBO_IDX:-3}
say() { echo "$1 walleye-dmesg-clean kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtoverlay >/dev/null || say skip "fdtoverlay (dtc) not installed"
[ -r "$DTBO" ] && [ -r "$DTB" ] || say skip "need DTBO=$DTBO and DTB=$DTB readable"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT
e=$(xxd -s $((32+32*IDX)) -l 8 -p "$DTBO") || say skip "cannot read the dtbo table"
size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
[ "$size" -gt 0 ] || say skip "dtbo entry $IDX is empty"
tail -c +$((off+1)) "$DTBO" | head -c "$size" > "$T/ov"
out=$(fdtoverlay -i "$DTB" -o "$T/m" "$T/ov" 2>&1) || say fail "overlay does not merge: $(echo "$out" | tr '\n' ' ' | cut -c1-120)"
gcc=$(fdtget -t u "$T/m" /soc@0/clock-controller@100000 phandle 2>/dev/null) || say fail "no phandle on /soc@0/clock-controller@100000 in the merged dtb"
bad=
for p in /soc@0/i2c@c179000 /soc@0/serial@0c16f000 /soc@0/bt_wcn3990 /soc@0/mnh-sm; do
  c=$(fdtget -t u "$T/m" "$p" clocks 2>/dev/null | awk '{print $1}')
  [ "$c" = "$gcc" ] || bad="$bad $p:clocks[0]=${c:-none}(want $gcc)"
done
[ -z "$bad" ] || say fail "clock users not on the real gcc:$bad"
say pass "the four clock users (i2c@c179000, serial@c16f000, bt_wcn3990, mnh-sm) have clocks[0] = gcc (phandle $gcc) after the merge"
