#!/bin/sh
# The dtb of image L5 (item walleye-display-late-probe-l5-gpio-shared-proxy; research walleye-display-late-probe-l4-tried-dsi-binds-panel-defers-on-reset), from image L4's own dtb:
#   build-display-late-l5-dtb.sh L4_BOOT_IMG OUT_DIR      (L4_BOOT_IMG: images-display-late-l4/boot-l4.img)
# Why: L4's panel defers on "cannot find GPIO chip gpiolib_shared.proxy.6". The kernel (drivers/gpio/gpiolib-shared.c, gpio_shared_of_traverse) makes a gpio "shared" when two available nodes name it;
# the panel's reset-gpios <tlmm 94> has a second claimant, qcom,platform-reset-gpio <tlmm 94> that the vendor dtbo overlay (entry 3, fragment 89) adds to the dsi0 node itself (fragment 90 does the same to dsi1,
# which is not available, so the kernel skips it). The overlay cannot be edited, but where its label lands can: l5.dtb has the label mdss_dsi0 in /__symbols__ on a new stub /vendor-overlay-stubs/mdss_dsi0
# (the container is disabled, the kernel scans neither it nor what the overlay puts in it), so the overlay's properties never reach the real dsi@c994000. Same lever as z21 for the touch power gpio.
# Writes OUT_DIR/{l5.dtb,l5.diff (L4's decompiled dtb against l5.dtb),SHA256SUMS-display-late-l5-parts}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
IMG=${1:?usage: build-display-late-l5-dtb.sh L4_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
[ -s "$IMG" ] || { echo "build-display-late-l5-dtb: missing $IMG" >&2; exit 1; }
L4SUM=$(cd "$(dirname "$IMG")" && grep " boot-l4.img\$" SHA256SUMS-display-late-l4 | cut -d" " -f1)
ck "the input is L4: its sha256 is the one recorded (SHA256SUMS-display-late-l4)" "$(sha256sum "$IMG" | cut -d" " -f1)" "$L4SUM"
rm -f "$OUT/l5.dtb" "$OUT/l5.diff" "$OUT/SHA256SUMS-display-late-l5-parts"   # a failed run leaves no earlier outputs standing
lpoco-bringup image unpack "$IMG" --out "$T/l4" >/dev/null
L4=$T/l4/appended.dtb
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
C=/vendor-overlay-stubs
TEST=$HERE/tests/host/display-reset-gpio-single-claimant.sh
ck "the test fails on L4's dtb (control: two available nodes name tlmm 94)" "$(DTB="$L4" sh "$TEST" | grep -c " 2 available nodes name tlmm" || true)" 1
REAL=$(fdtget "$L4" /__symbols__ mdss_dsi0)
ck "L4's dtb: mdss_dsi0 names the real dsi node" "$REAL" /soc@0/display-subsystem@c900000/dsi@c994000
ck "L4's dtb: the stub container is disabled" "$(fdtget "$L4" "$C" status)" disabled
dts "$L4" > "$T/l4.dts"
MAXPH=$(sed -n 's/^[[:space:]]*phandle = <0x\([0-9a-f]*\)>;$/\1/p' "$T/l4.dts" | while read -r h; do echo $((0x$h)); done | sort -n | tail -n1)
cp "$L4" "$T/l5.dtb"
fdtput -c "$T/l5.dtb" "$C/mdss_dsi0"
fdtput -t u "$T/l5.dtb" "$C/mdss_dsi0" phandle $((MAXPH+1))
fdtput -t s "$T/l5.dtb" /__symbols__ mdss_dsi0 "$C/mdss_dsi0"
dts "$T/l5.dtb" > "$T/l5.dts"
diff -u "$T/l4.dts" "$T/l5.dts" | sed '1,2d' > "$T/l5.diff" || true
ck "l5: the diff against L4's dtb adds one stub node with a phandle and repoints the mdss_dsi0 symbol, nothing else" "$(grep -c '^+' "$T/l5.diff") $(grep -c '^-' "$T/l5.diff") $(grep '^[-+]' "$T/l5.diff" | grep -vc -E '^\+[[:space:]]*mdss_dsi0: mdss_dsi0 \{$|^\+[[:space:]]*phandle = <0x[0-9a-f]+>;$|^\+[[:space:]]*\};$|^\+$|^[-+][[:space:]]*(mdss_dsi0: )?dsi@c994000 \{$|^[-+][[:space:]]*mdss_dsi0 = "' || true)" "6 2 0"
ck "l5: the label mdss_dsi0 names the stub" "$(fdtget "$T/l5.dtb" /__symbols__ mdss_dsi0)" "$C/mdss_dsi0"
ck "l5: the real dsi node is unchanged (properties and the panel child)" "$(fdtget -p "$T/l5.dtb" "$REAL" | tr '\n' ' ') $(fdtget -l "$T/l5.dtb" "$REAL" | tr '\n' ' ')" "$(fdtget -p "$L4" "$REAL" | tr '\n' ' ') $(fdtget -l "$L4" "$REAL" | tr '\n' ' ')"
ck "l5: the panel keeps its reset-gpios" "$(fdtget -t u "$T/l5.dtb" "$REAL/panel@0" reset-gpios)" "$(fdtget -t u "$L4" "$REAL/panel@0" reset-gpios)"
ck "the test passes on l5.dtb (tests/host/display-reset-gpio-single-claimant.sh)" "$(DTB="$T/l5.dtb" sh "$TEST" | cut -d' ' -f1 || true)" pass
ck "the four display pinctrl labels still name real nodes (tests/host/dtb-display-pinctrl-labels.sh)" "$(DTB="$T/l5.dtb" sh "$HERE/tests/host/dtb-display-pinctrl-labels.sh" | cut -d' ' -f1)" pass
ck "the stock overlay still merges into l5 (tests/host/dtbo-overlay-merges.sh)" "$(DTB="$T/l5.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "the touch power gpio keeps one claimant in l5 (tests/host/touch-gpio-single-claimant.sh)" "$(DTB="$T/l5.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
[ $total -eq 0 ] || { echo "build-display-late-l5-dtb: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$T/l5.dtb" "$T/l5.diff" "$OUT/"
(cd "$OUT" && sha256sum l5.dtb l5.diff | tee SHA256SUMS-display-late-l5-parts)
