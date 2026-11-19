#!/bin/sh
# Counts what the display dtb bisect images f0..f3 must be (item walleye-display-dtb-bisect-images), and writes SHA256SUMS in IMAGES_DIR:
#   check-display-dtb-bisect.sh IMAGES_DIR Z21_BOOT_IMG LATE_BOOT_IMG PARTS_DIR     (PARTS_DIR: build-display-dtb-bisect-dtbs.sh's OUT_DIR; LATE_BOOT_IMG: the late-probe image)
# Each image: the ramdisk (init-late and the display modules), command line, header words, size and footer are the late-probe image's; the kernel is the late-probe Image.gz, byte for byte, with the part's dtb
# appended (f0 = z21's dtb, f1..f3 = z21's plus one group); the dtb of each is read back from the image; tests/host/ufs-off-image.sh passes. Exit 1 on any FAIL.
set -eu
D=${1:?usage: check-display-dtb-bisect.sh IMAGES_DIR Z21_BOOT_IMG LATE_BOOT_IMG PARTS_DIR}
Z=${2:?}
L=${3:?}
P=${4:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
same() { cmp -s "$1" "$2" && echo same || echo differs; }
lpoco-bringup image unpack "$Z" --out "$T/z" >/dev/null
lpoco-bringup image unpack "$L" --out "$T/l" >/dev/null
head -c $(( $(stat -c %s "$T/l/kernel") - $(stat -c %s "$T/l/appended.dtb") )) "$T/l/kernel" > "$T/l.gz"
(cd "$P" && sha256sum -c SHA256SUMS-display-dtb-bisect-parts >/dev/null 2>&1) || { echo "check-display-dtb-bisect: the parts do not match their sums" >&2; exit 1; }
ck "Image.gz is the late-probe image's kernel" "$(same "$P/Image.gz" "$T/l.gz")" same
ck "f0.dtb is z21's dtb" "$(same "$P/f0.dtb" "$T/z/appended.dtb")" same
ck "the late-probe image's own dtb is not z21's (the kernel is the late one)" "$(same "$T/l/appended.dtb" "$T/z/appended.dtb")" differs
ck "the four dtbs are four different files" "$(for f in f0 f1 f2 f3; do sha256sum < "$P/$f.dtb"; done | sort -u | wc -l)" 4
ck "each diff is the decompiled z21 dtb against its part (f1..f3.diff are not empty)" "$(for f in f1 f2 f3; do [ -s "$P/$f.diff" ] && echo yes || echo no; done | sort -u)" yes
for i in f0 f1 f2 f3; do
  IMG=$D/boot-$i.img
  lpoco-bringup image unpack "$IMG" --out "$T/$i" >/dev/null
  ck "$i: the dtb is $i.dtb" "$(same "$T/$i/appended.dtb" "$P/$i.dtb")" same
  ck "$i: the kernel is the late-probe Image.gz then that dtb" "$(cat "$P/Image.gz" "$P/$i.dtb" | cmp -s - "$T/$i/kernel" && echo same || echo differs)" same
  ck "$i: the ramdisk is the late-probe image's" "$(same "$T/$i/ramdisk" "$T/l/ramdisk")" same
  ck "$i: the header (command line, offsets, os words) is the late-probe image's" "$(grep -v '^#' "$T/$i/boot.toml" | cmp -s - "$(grep -v '^#' "$T/l/boot.toml" | { cat > "$T/lb"; echo "$T/lb"; })" && echo same || echo differs)" same
  ck "$i: the size is the late-probe image's" "$(stat -c %s "$IMG")" "$(stat -c %s "$L")"
  cp "$IMG" "$T/boot.img"
  ck "$i: the AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$i: the UFS-off gate passes (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
done
mkdir "$T/rd" && (cd "$T/rd" && xz -dc "$T/f0/ramdisk" | cpio -id 2>/dev/null || true)  # cpio cannot mknod as a user: the device nodes are not checked here
ck "the ramdisk holds init-late, byte for byte" "$(same "$T/rd/init" "$HERE/build/initramfs-mainline/init-late")" same
ck "the ramdisk holds the module list" "$([ -s "$T/rd/display-modules.list" ] && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "check-display-dtb-bisect:  $total checks failed, no sums written" >&2; exit 1; }
(cd "$D" && sha256sum boot-f0.img boot-f1.img boot-f2.img boot-f3.img | tee SHA256SUMS-display-dtb-bisect)
