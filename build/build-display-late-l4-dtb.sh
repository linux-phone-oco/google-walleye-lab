#!/bin/sh
# The dtb of image L4 (item walleye-display-late-probe-l4-pinctrl-labels-image; research walleye-display-dsi-defer-pinctrl-stubs; decision 2026-11-19-walleye-display-l4-pinctrl-labels-not-trace), from image L3's own dtb:
#   build-display-late-l4-dtb.sh L3_BOOT_IMG OUT_DIR      (L3_BOOT_IMG: images-display-late-l3/boot-l3.img)
# l4.dtb = L3's dtb with the four labels the vendor dtbo overlay hangs on the dsi node (mdss_dsi_active, mdss_dsi_suspend, mdss_te_active, mdss_te_suspend) pointing at the real tlmm groups
# (mdss-dsi-active-state, mdss-dsi-suspend-state, mdss-te-state) in /__symbols__, and their four empty stubs under /vendor-overlay-stubs removed. Same result as dt/msm8998-google-walleye-display.dts now gives a fresh build.
# The overlay's pinctrl states are named mdss_default/mdss_sleep, so the kernel applies nothing from them and no pin gets a second owner.
# Writes OUT_DIR/{l4.dtb,l4.diff (L3's decompiled dtb against l4.dtb),SHA256SUMS-display-late-l4-parts}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
IMG=${1:?usage: build-display-late-l4-dtb.sh L3_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
[ -s "$IMG" ] || { echo "build-display-late-l4-dtb: missing $IMG" >&2; exit 1; }
L3SUM=$(cd "$(dirname "$IMG")" && grep " boot-l3.img\$" SHA256SUMS-display-late-l3 | cut -d" " -f1)
ck "the input is L3: its sha256 is the one recorded (SHA256SUMS-display-late-l3)" "$(sha256sum "$IMG" | cut -d" " -f1)" "$L3SUM"
rm -f "$OUT/l4.dtb" "$OUT/l4.diff" "$OUT/SHA256SUMS-display-late-l4-parts"   # a failed run leaves no earlier outputs standing
lpoco-bringup image unpack "$IMG" --out "$T/l3" >/dev/null
L3=$T/l3/appended.dtb
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
PIN=/soc@0/pinctrl@3400000
TEST=$HERE/tests/host/dtb-display-pinctrl-labels.sh
ck "the test fails on L3's dtb (control: the labels name stubs)" "$(DTB="$L3" sh "$TEST" | cut -d' ' -f1 || true)" fail
cp "$L3" "$T/l4.dtb"
for pair in mdss_dsi_active:mdss-dsi-active-state mdss_dsi_suspend:mdss-dsi-suspend-state mdss_te_active:mdss-te-state mdss_te_suspend:mdss-te-state; do
  label=${pair%%:*}; group=${pair#*:}
  ck "L3's dtb: $label names the stub /vendor-overlay-stubs/$label" "$(fdtget "$L3" /__symbols__ "$label")" "/vendor-overlay-stubs/$label"
  ck "L3's dtb: the group $PIN/$group exists with a phandle" "$(fdtget "$L3" "$PIN/$group" phandle | grep -c '^[0-9]' || true)" 1
  fdtput -t s "$T/l4.dtb" /__symbols__ "$label" "$PIN/$group"
  fdtput -r "$T/l4.dtb" "/vendor-overlay-stubs/$label"
done
dts "$L3" > "$T/l3.dts"; dts "$T/l4.dtb" > "$T/l4.dts"
diff -u "$T/l3.dts" "$T/l4.dts" | sed '1,2d' > "$T/l4.diff" || true
ck "l4: the diff against L3's dtb is 4 stub nodes gone (their 4 phandle lines), 3 group lines gaining labels, 4 symbol lines repointed, nothing else" "$(grep -c '^-[[:space:]]*phandle = ' "$T/l4.diff") $(grep -c '^+' "$T/l4.diff") $(grep '^[-+]' "$T/l4.diff" | grep -vc -E '^[-+][[:space:]]*mdss_[a-z_0-9]+: [a-z_0-9: -]*\{$|^[-+][[:space:]]*mdss_(dsi|te)_(active|suspend) = "|^-[[:space:]]*phandle = <0x[0-9a-f]+>;$|^-[[:space:]]*\};$|^-$' || true)" "4 7 0"
ck "the test passes on l4.dtb (tests/host/dtb-display-pinctrl-labels.sh)" "$(DTB="$T/l4.dtb" sh "$TEST" | cut -d' ' -f1 || true)" pass
for l in mdss_dsi_active mdss_dsi_suspend mdss_te_active mdss_te_suspend; do
  ck "l4: no stub $l left" "$(fdtget -l "$T/l4.dtb" /vendor-overlay-stubs | grep -c "^$l\$" || true)" 0
done
ck "the dsi's own vdd-supply (L3) is kept" "$(fdtget "$T/l4.dtb" "$(fdtget "$T/l4.dtb" /__symbols__ mdss_dsi0)" vdd-supply)" "$(fdtget "$L3" "$(fdtget "$L3" /__symbols__ mdss_dsi0)" vdd-supply)"
ck "the panel's own pinctrl still points at the same groups (phandles unchanged)" "$(fdtget "$T/l4.dtb" "$(fdtget "$T/l4.dtb" /__symbols__ mdss_dsi0)/panel@0" pinctrl-0)" "$(fdtget "$L3" "$(fdtget "$L3" /__symbols__ mdss_dsi0)/panel@0" pinctrl-0)"
ck "the stock overlay still merges into l4 (tests/host/dtbo-overlay-merges.sh)" "$(DTB="$T/l4.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "the touch power gpio keeps one claimant in l4 (tests/host/touch-gpio-single-claimant.sh)" "$(DTB="$T/l4.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
[ $total -eq 0 ] || { echo "build-display-late-l4-dtb: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$T/l4.dtb" "$T/l4.diff" "$OUT/"
(cd "$OUT" && sha256sum l4.dtb l4.diff | tee SHA256SUMS-display-late-l4-parts)
