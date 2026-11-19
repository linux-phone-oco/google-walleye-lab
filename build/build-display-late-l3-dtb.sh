#!/bin/sh
# The dtb of image L3 (item walleye-display-late-probe-l3-image; research walleye-display-why-dsi-defers-silently), from image L1's own dtb:
#   build-display-late-l3-dtb.sh L1_BOOT_IMG OUT_DIR      (L1_BOOT_IMG: images-display-late-l/boot-l1.img)
# l3.dtb = L1's dtb with vdd-supply = the l1a rail (0.88 V, vreg_l1a_0p875) on the DSI host node (dsi@c994000), nothing else.
# Why that rail (chosen by analogy, not read from the phone): msm8998's DSI config asks for 'vdd' (0.9 V in the driver) and 'vdda' (1.2 V); our dts already gives vdda the l2a 1.2 V rail and the DSI PHY's vdds the l1a 0.88 V rail,
# the nearest to 0.9 V. No mainline board supplies the msm8998 DSI vdd, and the phone's dtbo entry 3 names no DSI core rail (its dsi fragments carry only the panel's vci, wqhd-vddio, pinctrl). The try proves or refutes it.
# Writes OUT_DIR/{l3.dtb,l3.diff (L1's decompiled dtb against l3.dtb),SHA256SUMS-display-late-l3-parts}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
IMG=${1:?usage: build-display-late-l3-dtb.sh L1_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
[ -s "$IMG" ] || { echo "build-display-late-l3-dtb: missing $IMG" >&2; exit 1; }
L1SUM=$(cd "$(dirname "$IMG")" && grep " boot-l1.img\$" SHA256SUMS-display-late-l | cut -d" " -f1)
ck "the input is L1: its sha256 is the one recorded (SHA256SUMS-display-late-l)" "$(sha256sum "$IMG" | cut -d" " -f1)" "$L1SUM"
rm -f "$OUT/l3.dtb" "$OUT/l3.diff" "$OUT/SHA256SUMS-display-late-l3-parts"   # a failed run leaves no earlier outputs standing
lpoco-bringup image unpack "$IMG" --out "$T/l1" >/dev/null
L1=$T/l1/appended.dtb
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
sym() { fdtget "$L1" /__symbols__ "$1"; }
DSI=$(sym mdss_dsi0); L1A=$(sym vreg_l1a_0p875); L2A=$(sym vreg_l2a_1p2)
PH=$(fdtget "$L1" "$L1A" phandle); PHX=$(printf "%x" "$PH")
ck "L1's dtb: the DSI host has vdda-supply = the l2a rail" "$(fdtget "$L1" "$DSI" vdda-supply)" "$(fdtget "$L1" "$L2A" phandle)"
ck "L1's dtb: the DSI host has no vdd-supply" "$(fdtget -p "$L1" "$DSI" | grep -c '^vdd-supply$' || true)" 0
ck "L1's dtb: the l1a rail is 880000 microvolt (0.88 V)" "$(fdtget "$L1" "$L1A" regulator-min-microvolt) $(fdtget "$L1" "$L1A" regulator-max-microvolt)" "880000 880000"
cp "$L1" "$T/l3.dtb"
fdtput -t x "$T/l3.dtb" "$DSI" vdd-supply "$PHX"
dts "$L1" > "$T/l1.dts"; dts "$T/l3.dtb" > "$T/l3.dts"
diff -u "$T/l1.dts" "$T/l3.dts" | sed '1,2d' > "$T/l3.diff" || true
changed() { grep '^[-+]' "$1" | sed 's/^\([-+]\)[[:space:]]*/\1/' | tr '\n' '|'; }
ck "l3: the diff against L1's dtb is the vdd-supply line, nothing else" "$(changed "$T/l3.diff")" "+vdd-supply = <$(printf '0x%x' "$PH")>;|"
ck "l3: vdd-supply names the l1a rail, vdda-supply kept" "$(fdtget "$T/l3.dtb" "$DSI" vdd-supply) $(fdtget "$T/l3.dtb" "$DSI" vdda-supply)" "$PH $(fdtget "$L1" "$L2A" phandle)"
ck "the stock overlay still merges into l3 (tests/host/dtbo-overlay-merges.sh)" "$(DTB="$T/l3.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "the touch power gpio keeps one claimant in l3 (tests/host/touch-gpio-single-claimant.sh)" "$(DTB="$T/l3.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
[ $total -eq 0 ] || { echo "build-display-late-l3-dtb: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$T/l3.dtb" "$T/l3.diff" "$OUT/"
(cd "$OUT" && sha256sum l3.dtb l3.diff | tee SHA256SUMS-display-late-l3-parts)
