#!/bin/sh
# The dtb of image L2 (item walleye-display-late-probe-l-images; decision 2026-11-19-walleye-display-iommu-reading), from the late-probe image's own dtb (the full display dtb):
#   build-display-late-l-dtb.sh LATE_BOOT_IMG OUT_DIR      (LATE_BOOT_IMG: images-display-late/boot-display-late.img)
# L2.dtb = that dtb with the iommus property removed from the mdss node (display-subsystem@c900000), nothing else (the g2 recipe).
# Writes OUT_DIR/{l2.dtb,l2.diff (the decompiled late dtb against l2.dtb),SHA256SUMS-display-late-l-parts}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
IMG=${1:?usage: build-display-late-l-dtb.sh LATE_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
[ -s "$IMG" ] || { echo "build-display-late-l-dtb: missing $IMG" >&2; exit 1; }
lpoco-bringup image unpack "$IMG" --out "$T/late" >/dev/null
LATE=$T/late/appended.dtb
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
st() { fdtget "$1" "$2" status 2>/dev/null || echo absent; }
sym() { fdtget "$LATE" /__symbols__ "$1"; }
MDSS=$(sym mdss); MDP=$(sym mdss_mdp); DSI=$(sym mdss_dsi0); PHY=$(sym mdss_dsi0_phy)
ck "the late dtb has the iommus property on mdss" "$(fdtget "$LATE" "$MDSS" iommus)" "142 0"
ck "the late dtb has the panel node" "$(fdtget -l "$LATE" "$DSI" | grep -c '^panel@0$')" 1
cp "$LATE" "$T/l2.dtb"
fdtput -d "$T/l2.dtb" "$MDSS" iommus
dts "$LATE" > "$T/late.dts"; dts "$T/l2.dtb" > "$T/l2.dts"
diff -u "$T/late.dts" "$T/l2.dts" | sed '1,2d' > "$T/l2.diff" || true
changed() { grep '^[-+]' "$1" | sed 's/^\([-+]\)[[:space:]]*/\1/' | tr '\n' '|'; }
ck "l2: the diff against the late dtb is the iommus line, nothing else" "$(changed "$T/l2.diff")" "-iommus = <0x8e 0x00>;|"
ck "l2: no iommus on mdss; mdss, controller, DSI host, PHY okay; panel node kept" "$(fdtget -p "$T/l2.dtb" "$MDSS" | grep -c '^iommus$' || true) $(st "$T/l2.dtb" "$MDSS") $(st "$T/l2.dtb" "$MDP") $(st "$T/l2.dtb" "$DSI") $(st "$T/l2.dtb" "$PHY") $(fdtget -l "$T/l2.dtb" "$DSI" | grep -c '^panel@0$')" "0 okay okay okay okay 1"
ck "the stock overlay still merges into l2 (tests/host/dtbo-overlay-merges.sh)" "$(DTB="$T/l2.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "the touch power gpio keeps one claimant in l2 (tests/host/touch-gpio-single-claimant.sh)" "$(DTB="$T/l2.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
[ $total -eq 0 ] || { echo "build-display-late-l-dtb: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$T/l2.dtb" "$T/l2.diff" "$OUT/"
(cd "$OUT" && sha256sum l2.dtb l2.diff | tee SHA256SUMS-display-late-l-parts)
