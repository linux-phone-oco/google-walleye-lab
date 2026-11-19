#!/bin/sh
# mainline-light (backlog walleye-mainline-first-light): the control's boot.img header and AVB footer, OUR kernel, OUR -@ dtb, OUR mainline gadget init.
# Two images, the kernel's container being the one difference: ml1 = Image.gz, ml2 = the raw Image in an lz4 frame (as the stock kernel). Host only; nothing touches the phone.
#
#   build-mainline-light.sh WORK_DIR CONTROL_BOOT_IMG KERNEL_OUT_DIR OUT_DIR
#     WORK_DIR          as build-vendor-gadget-m.sh (busybox-arm64, linux-v7.3 for gen_init_cpio)
#     KERNEL_OUT_DIR    build-kernel.sh's OUT_DIR (Image.gz, msm8998-google-walleye.dtb, b/arch/arm64/boot/Image)
# Header fields of the control kept; the command line is the stock one without the vendor-kernel words (decision walleye-mainline-cmdline).
# Writes OUT_DIR/{initramfs-mainline-light.cpio.xz,boot-ml1.img,boot-ml2.img,SHA256SUMS}, checks each (ok/FAIL, exit 1 on a FAIL).
set -eu
W=${1:?usage: build-mainline-light.sh WORK_DIR CONTROL_BOOT_IMG KERNEL_OUT_DIR OUT_DIR}
CTRL=${2:?}
KO=${3:?}
OUT=${4:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
[ -x "$GEN" ] || make -C "$W/linux-v7.3" ARCH=arm64 LLVM=1 O="$W/out/b" usr/gen_init_cpio >/dev/null
for f in "$KO/Image.gz" "$KO/msm8998-google-walleye.dtb" "$KO/b/arch/arm64/boot/Image" "$BB"; do [ -s "$f" ] || { echo "build-mainline-light: missing $f" >&2; exit 1; }; done
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
INFO=$(unpack_bootimg --boot_img "$CTRL" --out "$T/p")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
[ "$(field 'boot image header version')" = 0 ] || { echo "build-mainline-light: the control is not header v0" >&2; exit 1; }
PSIZE=$(stat -c %s "$CTRL")
AV=$(avbtool info_image --image "$CTRL")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
# the stock words that name Android's own things stay; the vendor kernel's options go; what mainline must have comes first
CMD="earlycon=msm_serial_dm,0xc1b0000 clk_ignore_unused ignore_loglevel panic=5 $(field 'command line args' | tr ' ' '\n' | grep '^androidboot\.' | tr '\n' ' ' | sed 's/ $//')"
I=$HERE/initramfs-mainline/init-ml
{
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"
  echo "slink /bin/sh busybox 777 0 0"
  echo "file /init $I 755 0 0"
} > "$T/ml.spec"
"$GEN" -t 0 "$T/ml.spec" > "$T/rd.cpio"
xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$OUT/initramfs-mainline-light.cpio.xz"
lz4 -9 -BI -B7 -c "$KO/b/arch/arm64/boot/Image" > "$T/Image.lz4"
cat "$KO/Image.gz" "$KO/msm8998-google-walleye.dtb" > "$T/blob-ml1"
cat "$T/Image.lz4" "$KO/msm8998-google-walleye.dtb" > "$T/blob-ml2"
h() { sha256sum "$1" | cut -c1-16; }
hdr() { echo "$1" | grep -v -e '^boot magic' -e 'ramdisk size' -e '^kernel_size' -e 'command line args'; }
total=0
for S in ml1 ml2; do
  IMG=$OUT/boot-$S.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$CMD" \
    --kernel "$T/blob-$S" --ramdisk "$OUT/initramfs-mainline-light.cpio.xz" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  pass=0; fail=0
  ck() { if [ "$2" = "$3" ]; then echo "ok   $S $1"; pass=$((pass+1)); else echo "FAIL $S $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$S")
  ck "header equals the control's (offsets, page, os word) but size and command line" "$(hdr "$U")" "$(hdr "$INFO")"
  ck "command line is what mainline must have plus the stock androidboot.* words" "$(field 'command line args' | tr ' ' '\n' | grep -c '^androidboot\.')" "$(echo "$U" | sed -n 's/^command line args: *//p' | tr ' ' '\n' | grep -c '^androidboot\.')"
  ck "command line has earlycon, clk_ignore_unused, ignore_loglevel, panic=5, androidboot.dtbo_idx=3" "$(echo "$U" | sed -n 's/^command line args: *//p' | tr ' ' '\n' | grep -c -x -e 'earlycon=msm_serial_dm,0xc1b0000' -e clk_ignore_unused -e ignore_loglevel -e panic=5 -e androidboot.dtbo_idx=3)" 5
  ck "kernel blob is $S's" "$(h "$T/u$S/kernel")" "$(h "$T/blob-$S")"
  ck "our dtb ends the kernel blob" "$(tail -c "$(stat -c %s "$KO/msm8998-google-walleye.dtb")" "$T/u$S/kernel" | sha256sum | cut -c1-16)" "$(h "$KO/msm8998-google-walleye.dtb")"
  ck "our dtb has qcom,msm-id <0x124 0x20001>, no bootargs, /__symbols__" "$(fdtdump "$KO/msm8998-google-walleye.dtb" 2>/dev/null | grep -c -e 'qcom,msm-id = <0x00000124 0x00020001>' -e '__symbols__')$(fdtdump "$KO/msm8998-google-walleye.dtb" 2>/dev/null | grep -c bootargs)" 20
  ck "our dtb has the watchdog node" "$(fdtdump "$KO/msm8998-google-walleye.dtb" 2>/dev/null | grep -c 'watchdog@17817000')" 1
  ck "ramdisk is this build's xz" "$(h "$T/u$S/ramdisk")" "$(h "$OUT/initramfs-mainline-light.cpio.xz")"
  ck "ramdisk has /init, /bin/busybox, /dev/console" "$(xz -dc "$T/u$S/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e bin/busybox -e dev/console)" 3
  mkdir "$T/x$S" && (cd "$T/x$S" && xz -dc "$T/u$S/ramdisk" | cpio -id --quiet 2>/dev/null || true)
  ck "busybox in it is aarch64 static" "$(file -b "$T/x$S/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
  ck "the init in the ramdisk is the source init" "$(cmp -s "$T/x$S/init" "$I" && echo same || echo differs)" same
  ck "init opens /dev/watchdog and never writes to fd 3 again" "$(grep -c 'exec 3> /dev/watchdog' "$I")$(grep -c '>&3' "$I")" 10
  ck "init has no 'set -e', no ncm, no net, ends in a sleep loop" "$(grep -c -e '^set -e' "$I")$(grep -v '^#' "$I" | grep -c -E 'ncm|dev_addr|host_addr|ifconfig')$(grep -c '^while :; do sleep 3600; done$' "$I")" 001
  ck "init has product 'Pixel 2 walleye (mainline)' and serial walleye-first-boot" "$(grep -v '^#' "$I" | grep -c -e 'walleye-first-boot' -e 'Pixel 2 walleye (mainline)')" 2
  ck "size is the control's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  mkdir "$T/v$S" && cp "$IMG" "$T/v$S/boot.img"
  ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v$S/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  echo "boot-$S: $pass ok, $fail failed"
  total=$((total+fail))
done
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
ck "ml1 and ml2 differ only in the kernel blob (and what follows from it)" "$(cmp -l "$OUT/boot-ml1.img" "$OUT/boot-ml2.img" | wc -l | awk '{print ($1>0)}')" 1
ck "ml1's kernel is gzip, ml2's is an lz4 frame" "$(head -c2 "$T/blob-ml1" | od -An -tx1 | tr -d ' ')$(head -c4 "$T/blob-ml2" | od -An -tx1 | tr -d ' ')" 1f8b04224d18
(cd "$OUT" && sha256sum initramfs-mainline-light.cpio.xz boot-ml1.img boot-ml2.img | tee SHA256SUMS)
[ $total -eq 0 ]
