#!/bin/sh
# walleye-dtb-display-pinctrl-labels, run on the lab host: do the pinctrl labels the vendor dtbo overlay hangs on the dsi node
# (mdss_dsi_active, mdss_dsi_suspend, mdss_te_active, mdss_te_suspend: fragment 89 of overlay 3) name real pin groups in the dtb?
# The overlay's pinctrl-0/1 on dsi@c994000 resolve through the dtb's /__symbols__; a label that names a node under /vendor-overlay-stubs
# has no pin controller above it, so the kernel's pinctrl_bind_pins defers the dsi silently before dsi_dev_probe (research
# walleye-display-dsi-defer-pinctrl-stubs). Passes only when none of the four names a stub. Prints one line
# `pass|fail|skip walleye-dtb-display-pinctrl-labels kernel DETAIL`; exit 0 only on pass. Read-only.
# Environment: DTB (the dtb to read; default the display late-probe L3 dtb).
DTB=${DTB:-$HOME/Projects/walleye-build/images-display-late-l3/l3.dtb}
say() { echo "$1 walleye-dtb-display-pinctrl-labels kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtget >/dev/null || say skip "fdtget (dtc) not installed: not known yet"
[ -r "$DTB" ] || say skip "need DTB=$DTB readable: not known yet"
bad=
for l in mdss_dsi_active mdss_dsi_suspend mdss_te_active mdss_te_suspend; do
  p=$(fdtget "$DTB" /__symbols__ "$l" 2>/dev/null) || say fail "label $l is not in /__symbols__ of $(basename "$DTB"): the overlay's reference cannot resolve"
  case $p in /vendor-overlay-stubs/*) bad="$bad $l";; esac
done
[ -z "$bad" ] || say fail "$(basename "$DTB"): label(s)$bad name a stub under /vendor-overlay-stubs, which has no pin controller above it: dsi@c994000 defers silently"
say pass "the four display pinctrl labels name real nodes in $(basename "$DTB")"
