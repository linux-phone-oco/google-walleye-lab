#!/bin/sh
# pk1 and pk2 (backlog walleye-porthole-known-good): porthole's kernel (kernel.org v7.2.2 + their numbered series, their config) with the
# board's own dtb, in ml5a's image: ml5a with ONLY the kernel blob (Image.gz + appended dtb) changed. Host only; nothing touches the phone.
#   build-pk.sh KERNEL_TREE KERNEL_OUT ML5A_BOOT_IMG DTBO_IMG OUT_DIR
#     KERNEL_TREE  patched v7.2.2 tree (build/pk-kernel.sh makes it)   KERNEL_OUT  its O= directory, Image.gz and qcom/msm8998-google-{taimen,walleye}.dtb built with DTC_FLAGS=-@
#     ML5A_BOOT_IMG  the recorded ml5a image (header, command line, ramdisk with init-ml5a, AVB footer are kept)
#     DTBO_IMG     the stock dtbo.img; entry 3 (androidboot.dtbo_idx=3) is the overlay the bootloader merges
# pk1 = taimen dtb (porthole's own, known to boot taimen), pk2 = walleye dtb (porthole patch 0029). Each dtb gets empty stub nodes for the
# labels the vendor overlay targets (gen-overlay-stubs.sh, decision 2026-11-17-walleye-dtb-carries-overlay-symbols) via a wrapper dts.
# Writes OUT_DIR/{boot-pk1.img,boot-pk2.img,pk1.dtb,pk2.dtb,Image.gz,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL).
set -eu
K=${1:?usage: build-pk.sh KERNEL_TREE KERNEL_OUT ML5A_BOOT_IMG DTBO_IMG OUT_DIR}
KO=${2:?}
ML5A=${3:?}
DTBO=${4:?}
OUT=${5:?}
HERE=$(cd "$(dirname "$0")" && pwd)
QCOM=$K/arch/arm64/boot/dts/qcom
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$KO/arch/arm64/boot/Image.gz" "$KO/arch/arm64/boot/dts/qcom/msm8998-google-taimen.dtb" "$KO/arch/arm64/boot/dts/qcom/msm8998-google-walleye.dtb" "$ML5A" "$DTBO"; do
  [ -s "$f" ] || { echo "build-pk: missing $f" >&2; exit 1; }
done
# the vendor overlay the bootloader picks: entry 3 of dtbo.img (32-byte header, 32-byte big-endian entries: size, offset, ...)
e=$(xxd -s $((32+32*3)) -l 8 -p "$DTBO")
dd if="$DTBO" of="$T/ov.dtb" bs=1 skip=$((0x$(echo "$e" | cut -c9-16))) count=$((0x$(echo "$e" | cut -c1-8))) 2>/dev/null
M="make -C $K ARCH=arm64 LLVM=1 O=$KO"
n=1
for B in taimen walleye; do
  W=pk$n
  "$HERE/gen-overlay-stubs.sh" "$T/ov.dtb" "$KO/arch/arm64/boot/dts/qcom/msm8998-google-$B.dtb" "$QCOM/pk-overlay-stubs-$B.dtsi" >/dev/null
  # porthole's walleye file has no qcom,msm-id (their taimen file has it); the control's dtb says <0x124 0x20001>, the id the bootloader selects on
  MSM=
  [ "$B" = walleye ] && MSM='/ { qcom,msm-id = <0x124 0x20001>; };'
  printf '// SPDX-License-Identifier: GPL-2.0-only\n// %s: porthole board file %s plus what the bootloader needs (build/build-pk.sh)\n#include "msm8998-google-%s.dts"\n%s\n#include "pk-overlay-stubs-%s.dtsi"\n' "$W" "$B" "$B" "$MSM" "$B" > "$QCOM/msm8998-google-$W.dts"
  grep -q "msm8998-google-$W.dtb" "$QCOM/Makefile" || sed -i "s/^dtb-\$(CONFIG_ARCH_QCOM)\t+= msm8998-google-$B.dtb\$/&\ndtb-\$(CONFIG_ARCH_QCOM)\t+= msm8998-google-$W.dtb/" "$QCOM/Makefile"
  grep -q "msm8998-google-$W.dtb" "$QCOM/Makefile" || { echo "build-pk: Makefile line for $W not added" >&2; exit 1; }
  DTC_FLAGS=-@ $M qcom/msm8998-google-$W.dtb >/dev/null
  cp "$KO/arch/arm64/boot/dts/qcom/msm8998-google-$W.dtb" "$OUT/$W.dtb"
  n=$((n+1))
done
cp "$KO/arch/arm64/boot/Image.gz" "$OUT/Image.gz"
INFO=$(unpack_bootimg --boot_img "$ML5A" --out "$T/p")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$ML5A")
AV=$(avbtool info_image --image "$ML5A")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
hdr() { echo "$1" | grep -v -e 'kernel_size' -e '^boot magic'; }
h() { sha256sum "$1" | cut -c1-16; }
for W in pk1 pk2; do
  D=$OUT/$W.dtb
  cat "$OUT/Image.gz" "$D" > "$T/blob-$W"
  IMG=$OUT/boot-$W.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
    --kernel "$T/blob-$W" --ramdisk "$T/p/ramdisk" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$W")
  ck "$W header (offsets, page, os word, command line, ramdisk size) equals ml5a's but the kernel size" "$(hdr "$U")" "$(hdr "$INFO")"
  ck "$W ramdisk bytes equal ml5a's (init-ml5a: reboot at once)" "$(cmp -s "$T/p/ramdisk" "$T/u$W/ramdisk" && echo same || echo differs)" same
  ck "$W kernel blob is Image.gz then $W.dtb" "$(h "$T/u$W/kernel")" "$(h "$T/blob-$W")"
  ck "$W kernel differs from ml5a's (the one change)" "$(cmp -s "$T/p/kernel" "$T/u$W/kernel" && echo same || echo differs)" differs
  ck "$W Image.gz is the kernel's (gzip, arm64 Image inside)" "$(gzip -dc "$OUT/Image.gz" | file -b - | cut -c1-18)" "Linux kernel ARM64"
  ck "$W dtb has qcom,msm-id <0x124 0x20001>, /__symbols__, no bootargs" "$(fdtdump "$D" 2>/dev/null | grep -c -e 'qcom,msm-id = <0x00000124 0x00020001>' -e '__symbols__')$(fdtdump "$D" 2>/dev/null | grep -c bootargs)" 20
  ck "$W dtb: the stock dtbo overlay (entry 3) merges" "$(fdtoverlay -i "$D" -o "$T/merged-$W" "$T/ov.dtb" >/dev/null 2>&1 && echo merges || echo fails)" merges
  ck "$W size is ml5a's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  mkdir "$T/v$W" && cp "$IMG" "$T/v$W/boot.img"
  ck "$W AVB footer verifies" "$(avbtool verify_image --image "$T/v$W/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
done
ck "pk1 and pk2 differ in the dtb only" "$(cmp -s "$OUT/boot-pk1.img" "$OUT/boot-pk2.img" && echo same || echo differs)" differs
ck "porthole's walleye board file is in the tree (patch 0029)" "$(grep -c 'google,walleye' "$QCOM/msm8998-google-walleye.dts")" 1
ck "kernel release is v7.2.2" "$(sed -n 's/^7\.2\.2.*/7.2.2/p;q' "$KO/include/config/kernel.release")" 7.2.2
(cd "$OUT" && sha256sum Image.gz pk1.dtb pk2.dtb boot-pk1.img boot-pk2.img | tee SHA256SUMS)
[ $total -eq 0 ]
