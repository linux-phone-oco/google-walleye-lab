#!/bin/sh
# Build the walleye kernel WITH the display path (item walleye-display-s6e3fa5): build-kernel.sh's tree and config plus kernel/panel-samsung-s6e3fa5.c
# (Kconfig and Makefile lines added to the drivers/gpu/drm/panel of the tree copy), build/walleye-display.config, and dt/msm8998-google-walleye-display.dts.
# Built only: nothing here touches a phone; the keeper tries the image (item walleye-try-display) after the UFS-off gate.
#   build-display.sh KERNEL_TREE OUT_DIR      (KERNEL_TREE: a worktree of the pmic-wdt topic, e.g. ~/Projects/walleye-build/wt-k-display)
# Writes OUT_DIR/{Image.gz,msm8998-google-walleye-display.dtb,panel-samsung-s6e3fa5.o,.config,SHA256SUMS-display}; checks stop the build.
set -eu
K=$(cd "${1:?usage: build-display.sh KERNEL_TREE OUT_DIR}" && pwd)
mkdir -p "${2:?usage: build-display.sh KERNEL_TREE OUT_DIR}"; OUT=$(cd "$2" && pwd)
HERE=$(cd "$(dirname "$0")/.." && pwd)
QCOM=$K/arch/arm64/boot/dts/qcom
PANEL=$K/drivers/gpu/drm/panel
cp "$HERE/dt/msm8998-google-walleye-overlay-stubs.dtsi" "$HERE/dt/msm8998-google-walleye.dts" "$HERE/dt/msm8998-google-walleye-display.dts" "$QCOM/"
cp "$HERE/kernel/panel-samsung-s6e3fa5.c" "$PANEL/"
grep -q 'msm8998-google-walleye-display.dtb' "$QCOM/Makefile" || printf 'dtb-$(CONFIG_ARCH_QCOM)\t+= msm8998-google-walleye-display.dtb\n' >> "$QCOM/Makefile"
grep -q 'panel-samsung-s6e3fa5' "$PANEL/Makefile" || printf 'obj-$(CONFIG_DRM_PANEL_SAMSUNG_S6E3FA5) += panel-samsung-s6e3fa5.o\n' >> "$PANEL/Makefile"
if ! grep -q 'DRM_PANEL_SAMSUNG_S6E3FA5' "$PANEL/Kconfig"; then
  # a menu entry inside the panel menu: before its closing endmenu
  awk '/^endmenu/ && !d { print "config DRM_PANEL_SAMSUNG_S6E3FA5\n\ttristate \"Samsung S6E3FA5 command mode DSI panel\"\n\tdepends on OF\n\tdepends on DRM_MIPI_DSI\n\tdepends on BACKLIGHT_CLASS_DEVICE\n\thelp\n\t  Say Y here if you want to enable support for the Samsung S6E3FA5\n\t  command mode AMOLED panel (Google Pixel 2, OnePlus 5).\n"; d=1 } { print }' "$PANEL/Kconfig" > "$PANEL/Kconfig.tmp" && mv "$PANEL/Kconfig.tmp" "$PANEL/Kconfig"
fi
grep -q 'DRM_PANEL_SAMSUNG_S6E3FA5' "$PANEL/Kconfig" || { echo "build-display: Kconfig entry not added" >&2; exit 1; }
mkdir -p "$OUT/b"
M="make -C $K ARCH=arm64 LLVM=1 O=$OUT/b"
$M defconfig >/dev/null
"$K/scripts/kconfig/merge_config.sh" -m -O "$OUT/b" "$OUT/b/.config" "$HERE/build/walleye.config" "$HERE/build/walleye-display.config" ${EXTRA_CONFIG:-} >/dev/null
$M olddefconfig >/dev/null
for s in DRM DRM_MSM DRM_PANEL_SAMSUNG_S6E3FA5 BACKLIGHT_CLASS_DEVICE DRM_MSM_DPU DRM_MSM_DSI DRM_MSM_DSI_10NM_PHY FRAMEBUFFER_CONSOLE; do
  grep -q "^CONFIG_$s=y" "$OUT/b/.config" || { echo "build-display: CONFIG_$s is not =y" >&2; exit 1; }
done
DTC_FLAGS=-@ $M -j"$(nproc)" Image.gz qcom/msm8998-google-walleye-display.dtb
cp "$OUT/b/arch/arm64/boot/Image.gz" "$OUT/b/arch/arm64/boot/dts/qcom/msm8998-google-walleye-display.dtb" "$OUT/b/drivers/gpu/drm/panel/panel-samsung-s6e3fa5.o" "$OUT/b/.config" "$OUT/"
D="$OUT/msm8998-google-walleye-display.dtb"
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; exit 1; fi; }
ck "panel node compatible" "$(fdtget "$D" /soc@0/display-subsystem@c900000/dsi@c994000/panel@0 compatible)" samsung,s6e3fa5
ck "mdss on" "$(fdtget "$D" /soc@0/display-subsystem@c900000 status)" okay
ck "UFS stays off in the display dtb" "$(fdtget "$D" /soc@0/ufshc@1da4000 status)" disabled
( cd "$OUT" && sha256sum Image.gz msm8998-google-walleye-display.dtb panel-samsung-s6e3fa5.o > SHA256SUMS-display )
cat "$OUT/SHA256SUMS-display"
