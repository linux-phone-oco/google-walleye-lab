#!/bin/sh
# walleye-touch-gpio-get-fails, run on the lab host: does the touch power regulator's gpio specifier resolve, in the built dtb, to a valid line of the pmi8998 gpio chip?
# Passes only when regulator-touch-power's gpios <P N F> has P = the phandle of a node that is compatible qcom,pmi8998-gpio, #gpio-cells 2, N-1 (the spmi-gpio binding is 1-based:
# pmic_gpio_of_xlate returns N - 1) is below the chip's 14 lines and covered by the chip's own gpio-ranges (<self 0 0 14>). Says nothing of the runtime defer (that is the phone's read).
# Prints `pass|fail|skip walleye-touch-gpio-specifier kernel DETAIL`; exit 0 only on pass. Writes nothing. Environment: DTB (default the built z17 dtb).
DTB=${DTB:-$HOME/Projects/walleye-build/images-z17/msm8998-google-walleye-z17.dtb}
say() { echo "$1 walleye-touch-gpio-specifier kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtget >/dev/null || say skip "fdtget (dtc) not installed"
[ -r "$DTB" ] || say skip "need DTB=$DTB readable"
R=$(fdtget "$DTB" /__symbols__ touch_power_regulator 2>/dev/null) || say fail "no touch_power_regulator label in the dtb"
set -- $(fdtget -t u "$DTB" "$R" gpios 2>/dev/null)
[ $# -eq 3 ] || say fail "$R gpios has $# cells, want 3 (phandle, line, flags)"
P=$1; N=$2
G=
for s in $(fdtget -l "$DTB" /soc@0/spmi@800f000); do
  for c in $(fdtget -l "$DTB" "/soc@0/spmi@800f000/$s" 2>/dev/null); do
    n=/soc@0/spmi@800f000/$s/$c
    [ "$(fdtget -t u "$DTB" "$n" phandle 2>/dev/null)" = "$P" ] && G=$n
  done
done
[ -n "$G" ] || say fail "gpios phandle $P names no node under the spmi bus"
fdtget "$DTB" "$G" compatible | grep -q 'qcom,pmi8998-gpio' || say fail "$G is not a qcom,pmi8998-gpio"
[ "$(fdtget -t u "$DTB" "$G" "#gpio-cells")" = 2 ] || say fail "$G #gpio-cells is not 2"
set -- $(fdtget -t u "$DTB" "$G" gpio-ranges)
[ $# -eq 4 ] && [ "$1" = "$P" ] || say fail "$G gpio-ranges is not <self off pin count>: '$*'"
off=$2; cnt=$4; L=$((N - 1))
[ "$N" -ge 1 ] && [ "$L" -ge "$off" ] && [ "$L" -lt $((off + cnt)) ] || say fail "line $N (offset $L) is outside the chip's range $off..$((off + cnt - 1))"
say pass "gpios <$P $N> is line offset $L of $G, inside its $cnt-line self range"
