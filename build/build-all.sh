#!/bin/sh
# The whole first walleye image, from the kernel tree to the checked boot images. One command, no phone:
#
#   build/build-all.sh [WORK_DIR]        WORK_DIR defaults to ~/Projects/walleye-build; the tree is WORK_DIR/linux-v7.3
#
# Leaves WORK_DIR/images/boot-a.img and boot-b.img (see build/README.md for what the two are).
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
W=${1:-$HOME/Projects/walleye-build}
# KEEP_KERNEL=1: reuse WORK_DIR/out (a new boot image variant needs no new kernel, and the old images stay byte for byte)
if [ "${KEEP_KERNEL:-0}" != 1 ] || [ ! -f "$W/out/Image.gz" ]; then "$HERE/build-kernel.sh" "$W/linux-v7.3" "$W/out"; fi
# the dtb again with the kernel's extra dtc checks (W=1): none may come from our file
[ "${KEEP_KERNEL:-0}" = 1 ] && [ -f "$W/out/dtb-w1.log" ] || make -C "$W/linux-v7.3" ARCH=arm64 LLVM=1 O="$W/out/b" W=1 qcom/msm8998-google-walleye.dtb > "$W/out/dtb-w1.log" 2>&1 || { cat "$W/out/dtb-w1.log"; exit 1; }
if grep -n 'msm8998-google-walleye' "$W/out/dtb-w1.log" | grep -i 'warning'; then echo "build-all: dtc warnings from our dts" >&2; exit 1; fi
echo "dtc W=1: no warning from msm8998-google-walleye.dts"
[ -x "$W/busybox-arm64" ] || "$HERE/build-busybox.sh" arm64 "$W"
"$HERE/build-initramfs.sh" phone "$W" "$W/initramfs-phone.cpio.gz"
"$HERE/build-bootimg.sh" "$W/out/Image.gz" "$W/out/msm8998-google-walleye.dtb" "$W/initramfs-phone.cpio.gz" "$W/images"
ls -l "$W/images"/boot-*.img
sha256sum "$W/images"/boot-*.img
