#!/bin/sh
# walleye-build-z16-i2c-compatible, run on the lab host: after the bootloader's overlay merge, is the mainline qup controller at 0xc179000 left as our dtb has it,
# with the touch controller on it? The vendor overlay's i2c@c179000 merges BY NAME into our node of that name and overwrites its compatible
# (qcom,i2c-msm-v2), clocks, interrupts, dmas and pinctrl; a node of another name stays ours and the overlay's own node stands beside it, driverless.
# Passes only when, in DTB merged with the dtbo entry the bootloader picks, exactly one node with reg 0xc179000 under /soc@0 has compatible qcom,i2c-qup-v2.2.1,
# clock-names "core","iface" and the touchscreen@20 child with compatible syna,rmi4-i2c.
# Prints `pass|fail|skip walleye-i2c-qup kernel DETAIL`; exit 0 only on pass. Writes only a temp file.
# Environment: DTB (our dtb, default the built z16 dtb), DTBO (dtbo.img), DTBO_IDX (default 3).
W=$HOME/Projects/walleye-build
DTBO=${DTBO:-$W/control/dtbo.img}
DTB=${DTB:-$W/images-z16/msm8998-google-walleye-z16.dtb}
IDX=${DTBO_IDX:-3}
say() { echo "$1 walleye-i2c-qup kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtoverlay >/dev/null || say skip "fdtoverlay (dtc) not installed"
[ -r "$DTBO" ] && [ -r "$DTB" ] || say skip "need DTBO=$DTBO and DTB=$DTB readable"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT
e=$(xxd -s $((32+32*IDX)) -l 8 -p "$DTBO") || say skip "cannot read the dtbo table"
size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
[ "$size" -gt 0 ] || say skip "dtbo entry $IDX is empty"
tail -c +$((off+1)) "$DTBO" | head -c "$size" > "$T/ov"
out=$(fdtoverlay -i "$DTB" -o "$T/m" "$T/ov" 2>&1) || say fail "overlay does not merge: $(echo "$out" | tr '\n' ' ' | cut -c1-120)"
found=
for n in $(fdtget -l "$T/m" /soc@0); do
  r=$(fdtget -t x "$T/m" "/soc@0/$n" reg 2>/dev/null | awk '{print $1}')
  [ "$r" = c179000 ] || continue
  c=$(fdtget "$T/m" "/soc@0/$n" compatible 2>/dev/null)
  [ "$c" = qcom,i2c-qup-v2.2.1 ] || continue
  found="$found $n"
done
set -- $found
[ $# -eq 1 ] || say fail "want exactly one node at reg 0xc179000 with compatible qcom,i2c-qup-v2.2.1 after the merge, got ($found ) - the overlay's i2c@c179000 overwrote ours"
N=/soc@0/$1
cn=$(fdtget "$T/m" "$N" clock-names 2>/dev/null)
[ "$cn" = "core iface" ] || say fail "$1 clock-names is '$cn', want 'core iface'"
tc=$(fdtget "$T/m" "$N/touchscreen@20" compatible 2>/dev/null)
[ "$tc" = syna,rmi4-i2c ] || say fail "$1 has no touchscreen@20 syna,rmi4-i2c (got '$tc')"
for p in $(fdtget -p "$DTB" "$N"); do
  [ "$(fdtget "$DTB" "$N" "$p" 2>/dev/null)" = "$(fdtget "$T/m" "$N" "$p" 2>/dev/null)" ] || say fail "$1 property $p changed by the merge"
done
oc=$(fdtget "$T/m" /soc@0/i2c@c179000 compatible 2>/dev/null)
[ "$oc" = qcom,i2c-msm-v2 ] || say fail "the overlay's own i2c@c179000 is not beside ours (compatible '$oc')"
say pass "after the merge $1 is qcom,i2c-qup-v2.2.1, every property of it as in our dtb, with touchscreen@20 syna,rmi4-i2c; the overlay's own i2c@c179000 (qcom,i2c-msm-v2) stands apart. Not shown: node order and probe of two nodes at one region - the phone try shows those"
