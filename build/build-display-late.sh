#!/bin/sh
# The kernel of the display late probe (item walleye-display-late-probe-image): build-display.sh's tree and files, but the display path as MODULES (build/walleye-display-late.config) and netconsole built in.
# Built only: nothing here touches a phone. The modules the probe loads are staged with depmod output for the ramdisk (build-display-late-image.sh).
#   build-display-late.sh KERNEL_TREE OUT_DIR      (KERNEL_TREE: the display worktree, e.g. ~/Projects/walleye-build/wt-k-display)
# Writes OUT_DIR/{Image.gz,msm8998-google-walleye-display.dtb,modroot/lib/modules/KVER/... (the display modules and what they need, depmod run),display-modules.list (insmod lines, dependency order),.config,SHA256SUMS-display-late}; checks stop the build.
set -eu
K=$(cd "${1:?usage: build-display-late.sh KERNEL_TREE OUT_DIR}" && pwd)
mkdir -p "${2:?usage: build-display-late.sh KERNEL_TREE OUT_DIR}"; OUT=$(cd "$2" && pwd)
HERE=$(cd "$(dirname "$0")/.." && pwd)
QCOM=$K/arch/arm64/boot/dts/qcom
PANEL=$K/drivers/gpu/drm/panel
cp "$HERE/dt/msm8998-google-walleye-overlay-stubs.dtsi" "$HERE/dt/msm8998-google-walleye.dts" "$HERE/dt/msm8998-google-walleye-display.dts" "$QCOM/"
cp "$HERE/kernel/panel-samsung-s6e3fa5.c" "$PANEL/"
grep -q 'msm8998-google-walleye-display.dtb' "$QCOM/Makefile" || printf 'dtb-$(CONFIG_ARCH_QCOM)\t+= msm8998-google-walleye-display.dtb\n' >> "$QCOM/Makefile"
grep -q 'panel-samsung-s6e3fa5' "$PANEL/Makefile" || printf 'obj-$(CONFIG_DRM_PANEL_SAMSUNG_S6E3FA5) += panel-samsung-s6e3fa5.o\n' >> "$PANEL/Makefile"
if ! grep -q 'DRM_PANEL_SAMSUNG_S6E3FA5' "$PANEL/Kconfig"; then
  awk '/^endmenu/ && !d { print "config DRM_PANEL_SAMSUNG_S6E3FA5\n\ttristate \"Samsung S6E3FA5 command mode DSI panel\"\n\tdepends on OF\n\tdepends on DRM_MIPI_DSI\n\tdepends on BACKLIGHT_CLASS_DEVICE\n\thelp\n\t  Say Y here if you want to enable support for the Samsung S6E3FA5\n\t  command mode AMOLED panel (Google Pixel 2, OnePlus 5).\n"; d=1 } { print }' "$PANEL/Kconfig" > "$PANEL/Kconfig.tmp" && mv "$PANEL/Kconfig.tmp" "$PANEL/Kconfig"
fi
grep -q 'DRM_PANEL_SAMSUNG_S6E3FA5' "$PANEL/Kconfig" || { echo "build-display-late: Kconfig entry not added" >&2; exit 1; }
mkdir -p "$OUT/b"
M="make -C $K ARCH=arm64 LLVM=1 O=$OUT/b"
$M defconfig >/dev/null
"$K/scripts/kconfig/merge_config.sh" -m -O "$OUT/b" "$OUT/b/.config" "$HERE/build/walleye.config" "$HERE/build/walleye-display-late.config" ${EXTRA_CONFIG:-} >/dev/null
$M olddefconfig >/dev/null
want() { grep -q "^CONFIG_$1=$2\$" "$OUT/b/.config" || { echo "build-display-late: CONFIG_$1 is not =$2" >&2; exit 1; }; }
for s in DRM_MSM DRM_PANEL_SAMSUNG_S6E3FA5 DRM; do want $s m; done
for s in NETCONSOLE NETCONSOLE_DYNAMIC DYNAMIC_DEBUG DRM_MSM_DPU DRM_MSM_DSI DRM_MSM_DSI_10NM_PHY BACKLIGHT_CLASS_DEVICE USB_CONFIGFS_ECM CONFIGFS_FS; do want $s y; done
DTC_FLAGS=-@ $M -j"$(nproc)" Image.gz modules qcom/msm8998-google-walleye-display.dtb
rm -rf "$OUT/all"
$M modules_install INSTALL_MOD_PATH="$OUT/all" INSTALL_MOD_STRIP=1 >/dev/null
KVER=$(ls "$OUT/all/lib/modules")
[ "$(printf '%s\n' "$KVER" | wc -l)" -eq 1 ] || { echo "build-display-late: want one kernel version in modules_install, got: $KVER" >&2; exit 1; }
# the display set: msm and the panel driver with every module they need, found by modprobe against the full tree, copied to a staging root of their own, depmod run there
rm -rf "$OUT/modroot"; mkdir -p "$OUT/modroot"
modprobe -d "$OUT/all" -S "$KVER" --show-depends msm > "$OUT/l1"
modprobe -d "$OUT/all" -S "$KVER" --show-depends panel-samsung-s6e3fa5 > "$OUT/l2"
cat "$OUT/l1" "$OUT/l2" | awk '!s[$0]++' > "$OUT/l3"
grep -q '/msm\.ko' "$OUT/l3" || { echo "build-display-late: msm.ko not in the display set" >&2; exit 1; }
grep -q '/panel-samsung-s6e3fa5\.ko' "$OUT/l3" || { echo "build-display-late: the panel driver is not in the display set" >&2; exit 1; }
sed -n 's/^insmod *//p' "$OUT/l3" | sed "s/ *\$//; s#^$OUT/all##" > "$OUT/paths"
while IFS= read -r p; do mkdir -p "$OUT/modroot$(dirname "$p")"; cp "$OUT/all$p" "$OUT/modroot$p"; done < "$OUT/paths"
depmod -b "$OUT/modroot" "$KVER"
# the list the shell loads from, in dependency order, paths as the ramdisk has them
sed "s#^#insmod #" "$OUT/paths" > "$OUT/display-modules.list"
rm -f "$OUT/l1" "$OUT/l2" "$OUT/l3" "$OUT/paths"
cp "$OUT/b/System.map" "$OUT/"
cp "$OUT/b/arch/arm64/boot/Image.gz" "$OUT/b/arch/arm64/boot/dts/qcom/msm8998-google-walleye-display.dtb" "$OUT/b/.config" "$OUT/"
D="$OUT/msm8998-google-walleye-display.dtb"
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; exit 1; fi; }
ck "panel node compatible" "$(fdtget "$D" /soc@0/display-subsystem@c900000/dsi@c994000/panel@0 compatible)" samsung,s6e3fa5
ck "mdss on" "$(fdtget "$D" /soc@0/display-subsystem@c900000 status)" okay
ck "UFS stays off in the display dtb" "$(fdtget "$D" /soc@0/ufshc@1da4000 status)" disabled
ck "the display driver is a module: dpu_kms_init is in msm.ko (control)" "$(strings "$OUT/b/drivers/gpu/drm/msm/msm.ko" | grep -c dpu_kms_init | sed 's/^[1-9][0-9]*$/yes/')" yes
ck "the display driver is not in the kernel: dpu_kms_init is not in System.map" "$(grep -c " dpu_kms_init" "$OUT/b/System.map" || true)" 0
( cd "$OUT" && sha256sum Image.gz msm8998-google-walleye-display.dtb display-modules.list > SHA256SUMS-display-late && (cd modroot && find . -name '*.ko' | sort | xargs sha256sum) >> SHA256SUMS-display-late )
cat "$OUT/SHA256SUMS-display-late"; cat "$OUT/display-modules.list"
