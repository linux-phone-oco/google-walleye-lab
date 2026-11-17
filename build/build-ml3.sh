#!/bin/sh
# ml3 (backlog walleye-ml3-vendor-ramoops): ml1 with only the dts ramoops node moved to the phone's own region.
# Host only; nothing touches the phone.
#
#   build-ml3.sh WORK_DIR CONTROL_BOOT_IMG KERNEL_OUT_DIR OUT_DIR OLD_ML1_DTB
#     KERNEL_OUT_DIR  build-kernel.sh's OUT_DIR built from the dts with ramoops@a1100000
#     OLD_ML1_DTB     the dtb ml1 was built with (the one with ramoops@a1810000), to prove what differs
# Builds the mainline-light pair with the new dtb, keeps ml1's name as boot-ml3.img, and checks that
# ml3 differs from the recorded ml1 only where the dtb is. Writes OUT_DIR/{boot-ml3.img,initramfs-mainline-light.cpio.xz,SHA256SUMS}.
set -eu
W=${1:?usage: build-ml3.sh WORK_DIR CONTROL_BOOT_IMG KERNEL_OUT_DIR OUT_DIR OLD_ML1_DTB}
CTRL=${2:?}
KO=${3:?}
OUT=${4:?}
OLD=${5:?}
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
"$HERE/build-mainline-light.sh" "$W" "$CTRL" "$KO" "$T/ml" >"$T/ml.log" || { cat "$T/ml.log"; exit 1; }
mkdir -p "$OUT"
cp "$T/ml/boot-ml1.img" "$OUT/boot-ml3.img"
cp "$T/ml/initramfs-mainline-light.cpio.xz" "$OUT/"
D=$KO/msm8998-google-walleye.dtb
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
ck "ramoops node is at 0xa1100000, 2 MiB, record 0x80000, console 0x20000" \
  "$(for p in reg record-size console-size; do fdtget -t x "$D" /reserved-memory/ramoops@a1100000 $p; done | tr "\n" " ")" "0 a1100000 0 200000 80000 20000 "
ck "no pmsg-size, no ftrace-size in it" "$(fdtget -p "$D" /reserved-memory/ramoops@a1100000 | grep -c -e pmsg -e ftrace)" 0
ck "the old ramoops node is gone" "$(fdtdump "$D" 2>/dev/null | grep -c 'ramoops@a1810000')" 0
ck "the old dtb had it at a1810000 (so this is the change)" "$(fdtdump "$OLD" 2>/dev/null | grep -c 'ramoops@a1810000')" 1
ck "no other carve-out overlaps 0xa1100000..0xa1300000" "$(fdtdump "$D" 2>/dev/null | awk '/reserved-memory/,/clocks \{/' | grep -E 'reg = <0x00000000 0x[0-9a-f]{8} 0x00000000 0x[0-9a-f]{8}>' | grep -v a1100000 | sed 's/.*<0x00000000 0x\([0-9a-f]*\) 0x00000000 0x\([0-9a-f]*\)>.*/\1 \2/' | while read -r b s; do [ $((0x$b)) -lt $((0xa1300000)) ] && [ $((0x$b + 0x$s)) -gt $((0xa1100000)) ] && echo overlap; done | wc -l)" 0
ck "kernel config has PSTORE_CONSOLE and PSTORE_RAM =y" "$(grep -c -x -e CONFIG_PSTORE_CONSOLE=y -e CONFIG_PSTORE_RAM=y "$KO/.config")" 2
# ml3 vs recorded ml1: the kernel (gzip) is the same bytes, the dtb differs, the ramdisk is the same
ml1=${ML1_IMG:-}
if [ -n "$ml1" ] && [ -s "$ml1" ]; then
  mkdir "$T/a" "$T/b"
  unpack_bootimg --boot_img "$ml1" --out "$T/a" >/dev/null
  unpack_bootimg --boot_img "$OUT/boot-ml3.img" --out "$T/b" >/dev/null
  ck "ramdisk equals ml1's" "$(cmp -s "$T/a/ramdisk" "$T/b/ramdisk" && echo same || echo differs)" same
  n=$(stat -c %s "$KO/Image.gz"); m=$(stat -c %s "$OLD"); k=$(stat -c %s "$D")
  ck "kernel (Image.gz) part equals ml1's" "$(head -c "$n" "$T/a/kernel" | sha256sum | cut -c1-16)" "$(head -c "$n" "$T/b/kernel" | sha256sum | cut -c1-16)"
  ck "ml1's dtb part is the old dtb, ml3's is the new" "$(tail -c "$m" "$T/a/kernel" | cmp -s - "$OLD" && echo y || echo n)$(tail -c "$k" "$T/b/kernel" | cmp -s - "$D" && echo y || echo n)" yy
  ck "header (cmdline included) equals ml1's" "$(sed -n 's/^command line args: *//p;s/^page size: *//p;s/^os version: *//p' < "$(unpack_bootimg --boot_img "$ml1" --out "$T/a" >"$T/ia"; echo "$T/ia")" | sha256sum | cut -c1-16)" "$(unpack_bootimg --boot_img "$OUT/boot-ml3.img" --out "$T/b" | sed -n 's/^command line args: *//p;s/^page size: *//p;s/^os version: *//p' | sha256sum | cut -c1-16)"
fi
(cd "$OUT" && sha256sum initramfs-mainline-light.cpio.xz boot-ml3.img | tee SHA256SUMS)
[ $total -eq 0 ]
