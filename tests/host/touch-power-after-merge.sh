#!/bin/sh
# walleye-build-z17-touch-power, run on the lab host: after the bootloader's overlay merge, is the touch controller's vdd-supply still our regulator-fixed on the
# pmi8998 power gpio (gpio 2, active high; the phone's own overlay: synaptics,power-gpio, power-on-state 1) with l28 as its input?
# Passes only when, in DTB merged with the dtbo entry the bootloader picks, the touchscreen@20 under the node at reg 0xc179000 has a vdd-supply whose node is regulator-fixed,
# has gpios <pmi8998 gpio controller, 2, 0> and enable-active-high.
# Prints `pass|fail|skip walleye-touch-power kernel DETAIL`; exit 0 only on pass. Writes only a temp file.
# Environment: DTB (our dtb, default the built z17 dtb), DTBO (dtbo.img), DTBO_IDX (default 3).
W=$HOME/Projects/walleye-build
DTBO=${DTBO:-$W/control/dtbo.img}
DTB=${DTB:-$W/images-z17/msm8998-google-walleye-z17.dtb}
IDX=${DTBO_IDX:-3}
say() { echo "$1 walleye-touch-power kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtoverlay >/dev/null || say skip "fdtoverlay (dtc) not installed"
[ -r "$DTBO" ] && [ -r "$DTB" ] || say skip "need DTBO=$DTBO and DTB=$DTB readable"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT
e=$(xxd -s $((32+32*IDX)) -l 8 -p "$DTBO") || say skip "cannot read the dtbo table"
size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
[ "$size" -gt 0 ] || say skip "dtbo entry $IDX is empty"
tail -c +$((off+1)) "$DTBO" | head -c "$size" > "$T/ov"
out=$(fdtoverlay -i "$DTB" -o "$T/m" "$T/ov" 2>&1) || say fail "overlay does not merge: $(echo "$out" | tr '\n' ' ' | cut -c1-120)"
N=
for n in $(fdtget -l "$T/m" /soc@0); do
  r=$(fdtget -t x "$T/m" "/soc@0/$n" reg 2>/dev/null | awk '{print $1}')
  [ "$r" = c179000 ] && [ "$(fdtget "$T/m" "/soc@0/$n" compatible 2>/dev/null)" = qcom,i2c-qup-v2.2.1 ] && N=/soc@0/$n
done
[ -n "$N" ] || say fail "no qup controller of ours at reg 0xc179000 after the merge"
vdd=$(fdtget -t u "$T/m" "$N/touchscreen@20" vdd-supply 2>/dev/null) || say fail "touchscreen@20 has no vdd-supply"
R=
for n in $(fdtget -l "$T/m" /); do
  [ "$(fdtget -t u "$T/m" "/$n" phandle 2>/dev/null)" = "$vdd" ] && R=/$n
done
[ -n "$R" ] || say fail "vdd-supply phandle $vdd names no top-level node"
[ "$(fdtget "$T/m" "$R" compatible 2>/dev/null)" = regulator-fixed ] || say fail "$R is not a regulator-fixed"
fdtget -p "$T/m" "$R" | grep -q '^enable-active-high$' || say fail "$R lacks enable-active-high"
g=$(fdtget -t u "$T/m" "$R" gpios 2>/dev/null)
set -- $g
[ $# -eq 3 ] && [ "$2" = 2 ] && [ "$3" = 0 ] || say fail "$R gpios is '$g', want <pmi8998-gpio 2 0>"
c=$(fdtget "$T/m" "$(fdtget "$T/m" /__symbols__ pmi8998_gpios)" compatible 2>/dev/null | cut -d' ' -f1)
[ "$(fdtget -t u "$T/m" "$(fdtget "$T/m" /__symbols__ pmi8998_gpios)" phandle)" = "$1" ] && [ "$c" = qcom,pmi8998-gpio ] || say fail "$R gpio phandle $1 is not the pmi8998 gpio controller"
say pass "$N touchscreen@20 vdd-supply is $R, pmi8998 gpio 2 active high"
