#!/bin/sh
# Build the first walleye kernel: pristine kernel.org v7.3, arm64 defconfig + build/walleye.config, and our
# own dt/msm8998-google-walleye.dts copied into the tree (the tree itself stays pristine but for one Makefile line).
#
#   build-kernel.sh KERNEL_TREE OUT_DIR
#
# Writes OUT_DIR/{Image.gz,msm8998-google-walleye.dtb,.config}. Same ways as surya's kernel builds:
# a git worktree of the shared kernel clone, out-of-tree O=, clang (LLVM=1). Needs no phone.
set -eu
K=${1:?usage: build-kernel.sh KERNEL_TREE OUT_DIR}
OUT=${2:?usage: build-kernel.sh KERNEL_TREE OUT_DIR}
HERE=$(cd "$(dirname "$0")/.." && pwd)
QCOM=$K/arch/arm64/boot/dts/qcom
cp "$HERE/dt/msm8998-google-walleye-overlay-stubs.dtsi" "$HERE/dt/msm8998-google-walleye.dts" "$HERE/dt/msm8998-google-walleye-f.dts" "$QCOM/"
grep -q 'msm8998-google-walleye.dtb' "$QCOM/Makefile" ||
  sed -i 's/^dtb-\$(CONFIG_ARCH_QCOM)\t+= msm8998-asus-novago-tp370ql.dtb$/&\ndtb-$(CONFIG_ARCH_QCOM)\t+= msm8998-google-walleye.dtb/' "$QCOM/Makefile"
grep -q 'msm8998-google-walleye.dtb' "$QCOM/Makefile" || { echo "build-kernel: Makefile line not added" >&2; exit 1; }
grep -q 'msm8998-google-walleye-f.dtb' "$QCOM/Makefile" ||
  sed -i 's/^dtb-\$(CONFIG_ARCH_QCOM)\t+= msm8998-google-walleye.dtb$/&\ndtb-$(CONFIG_ARCH_QCOM)\t+= msm8998-google-walleye-f.dtb/' "$QCOM/Makefile"
grep -q 'msm8998-google-walleye-f.dtb' "$QCOM/Makefile" || { echo "build-kernel: f Makefile line not added" >&2; exit 1; }
mkdir -p "$OUT/b"
M="make -C $K ARCH=arm64 LLVM=1 O=$OUT/b"
$M defconfig >/dev/null
"$K/scripts/kconfig/merge_config.sh" -m -O "$OUT/b" "$OUT/b/.config" "$HERE/build/walleye.config" >/dev/null
$M olddefconfig >/dev/null
# the initramfs carries no modules: what the gadget stands on must be built in
for s in QCOM_WDT USB_CONFIGFS_MASS_STORAGE RD_XZ PHY_QCOM_QUSB2 USB_DWC3_QCOM USB_CONFIGFS USB_CONFIGFS_ACM USB_CONFIGFS_ECM USB_CONFIGFS_RNDIS PSTORE_RAM PINCTRL_MSM8998 MSM_GCC_8998 MSM_GPUCC_8998 SERIAL_MSM QCOM_SPMI_RRADC CHARGER_QCOM_SMB2; do
  grep -q "^CONFIG_$s=y" "$OUT/b/.config" || { echo "build-kernel: CONFIG_$s is not =y" >&2; exit 1; }
done
# -@: our dtb carries /__symbols__ and the stub nodes, so the bootloader's dtbo overlay merge can succeed (build/gen-overlay-stubs.sh)
DTC_FLAGS=-@ $M -j"$(nproc)" Image.gz qcom/msm8998-google-walleye.dtb qcom/msm8998-google-walleye-f.dtb
cp "$OUT/b/arch/arm64/boot/Image.gz" "$OUT/b/arch/arm64/boot/dts/qcom/msm8998-google-walleye.dtb" "$OUT/b/arch/arm64/boot/dts/qcom/msm8998-google-walleye-f.dtb" "$OUT/b/.config" "$OUT/"
ls -l "$OUT/Image.gz" "$OUT/msm8998-google-walleye.dtb"
