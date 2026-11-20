#!/bin/sh
# walleye-dtb-display-reset, run on the lab host: does the mdss node reset its block at probe (item walleye-display-reset-image; decision 2026-11-20-walleye-display-handover-reset-first)?
# Passes only when /soc@0/display-subsystem@c900000 holds resets = <&mmcc 5> (MDSS_BCR; the mmcc's reset controller takes one cell): msm_mdss_reset asks for one unnamed reset,
# so the property carries exactly that entry and there is no reset-names. Prints one line `pass|fail|skip walleye-dtb-display-reset kernel DETAIL`; exit 0 only on pass. Read-only.
# Environment: DTB (the dtb to read; default the reset image's dtb).
DTB=${DTB:-$HOME/Projects/walleye-build/images-display-reset/reset.dtb}
say() { echo "$1 walleye-dtb-display-reset kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtget >/dev/null || say skip "fdtget (dtc) not installed: not known yet"
[ -r "$DTB" ] || say skip "need DTB=$DTB readable: not known yet"
N=$(basename "$DTB")
M=/soc@0/display-subsystem@c900000
MM=$(fdtget "$DTB" /__symbols__ mmcc) || say fail "$N: no mmcc"
MMCC=$(fdtget -t u "$DTB" "$MM" phandle)
[ "$(fdtget -t u "$DTB" "$MM" '#reset-cells')" = 1 ] || say fail "$N: the mmcc does not take one reset cell"
fdtget -p "$DTB" "$M" | grep -qx resets || say fail "$N: $M has no resets"
[ "$(fdtget -t u "$DTB" "$M" resets)" = "$MMCC 5" ] || say fail "$N: $M's resets are not <&mmcc MDSS_BCR (5)>"
fdtget -p "$DTB" "$M" | grep -qx reset-names && say fail "$N: $M has reset-names (msm_mdss_reset asks for the unnamed one)"
say pass "$N: $M resets the block at probe with <&mmcc MDSS_BCR>"
