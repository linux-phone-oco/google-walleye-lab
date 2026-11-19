#!/bin/sh
# Counts what the f3 split images g1..g3 must be (item walleye-display-f3-split-images), writes g1.diff and SHA256SUMS-display-f3-split in IMAGES_DIR:
#   check-display-f3-split.sh IMAGES_DIR F3_BOOT_IMG PARTS_DIR     (PARTS_DIR: build-display-f3-split-dtbs.sh's OUT_DIR; F3_BOOT_IMG: images-display-dtb-bisect/boot-f3.img)
# Each image is f3 with exactly one thing changed: g1 the command line (iommu.passthrough=1 added), g2 and g3 the dtb appended to the kernel; kernel Image.gz, ramdisk, header words, size are f3's;
# the AVB footer verifies and tests/host/ufs-off-image.sh passes. Exit 1 on any FAIL.
set -eu
D=${1:?usage: check-display-f3-split.sh IMAGES_DIR F3_BOOT_IMG PARTS_DIR}
F3=${2:?}
P=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
same() { cmp -s "$1" "$2" && echo same || echo differs; }
lpoco-bringup image unpack "$F3" --out "$T/f3" >/dev/null
hdr() { grep -v '^#' "$1/boot.toml" | grep -v '^cmdline'; }
cmd() { grep '^cmdline' "$1/boot.toml"; }
for i in g1 g2 g3; do
  IMG=$D/boot-$i.img
  lpoco-bringup image unpack "$IMG" --out "$T/$i" >/dev/null
  ck "$i: the ramdisk is f3's" "$(same "$T/$i/ramdisk" "$T/f3/ramdisk")" same
  ck "$i: the header words other than the command line are f3's" "$([ "$(hdr "$T/$i")" = "$(hdr "$T/f3")" ] && echo same || echo differs)" same
  ck "$i: the size is f3's" "$(stat -c %s "$IMG")" "$(stat -c %s "$F3")"
  cp "$IMG" "$T/boot.img"
  ck "$i: the AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$i: the UFS-off gate passes (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
done
ck "g1: the dtb and the kernel are f3's" "$(same "$T/g1/kernel" "$T/f3/kernel")" same
ck "g1: the command line is f3's with ' iommu.passthrough=1' added at its end" "$(cmd "$T/g1")" "$(cmd "$T/f3" | sed 's/"$/ iommu.passthrough=1"/')"
ck "g1: the command line differs from f3's" "$([ "$(cmd "$T/g1")" = "$(cmd "$T/f3")" ] && echo same || echo differs)" differs
for i in g2 g3; do
  ck "$i: the command line is f3's" "$([ "$(cmd "$T/$i")" = "$(cmd "$T/f3")" ] && echo same || echo differs)" same
  ck "$i: the dtb is $i.dtb" "$(same "$T/$i/appended.dtb" "$P/$i.dtb")" same
  head -c $(( $(stat -c %s "$T/f3/kernel") - $(stat -c %s "$T/f3/appended.dtb") )) "$T/f3/kernel" > "$T/Image.gz"
  ck "$i: the kernel is f3's Image.gz then that dtb" "$(cat "$T/Image.gz" "$P/$i.dtb" | cmp -s - "$T/$i/kernel" && echo same || echo differs)" same
done
(cd "$P" && sha256sum -c SHA256SUMS-display-f3-split-parts >/dev/null 2>&1) || { echo "check-display-f3-split: the parts do not match their sums" >&2; exit 1; }
{ echo "--- f3 command line"; echo "+++ g1 command line"; echo "-$(cmd "$T/f3")"; echo "+$(cmd "$T/g1")"; } > "$T/g1.diff"
ck "g1: the diff has one line out and one in" "$(grep -c '^[-+]cmdline' "$T/g1.diff")" 2
[ $total -eq 0 ] || { echo "check-display-f3-split: $total checks failed, no sums written" >&2; exit 1; }
cp "$T/g1.diff" "$D/g1.diff"
(cd "$D" && sha256sum boot-g1.img boot-g2.img boot-g3.img g1.diff g2.diff g3.diff | tee SHA256SUMS-display-f3-split)
