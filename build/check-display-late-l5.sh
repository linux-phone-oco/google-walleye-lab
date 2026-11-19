#!/bin/sh
# Counts what image L5 must be (item walleye-display-late-probe-l5-gpio-shared-proxy), writes SHA256SUMS-display-late-l5 in IMAGES_DIR:
#   check-display-late-l5.sh IMAGES_DIR L4_BOOT_IMG PARTS_DIR     (PARTS_DIR: build-display-late-l5-dtb.sh's OUT_DIR; L4_BOOT_IMG: images-display-late-l4/boot-l4.img)
# boot-l5.img is L4 with exactly the dtb appended to the kernel changed: ramdisk, command line, header words, size and Image.gz are L4's; the AVB footer verifies; tests/host/ufs-off-image.sh passes;
# tests/host/display-reset-gpio-single-claimant.sh passes on the dtb inside the image. Exit 1 on any FAIL.
set -eu
D=${1:?usage: check-display-late-l5.sh IMAGES_DIR L4_BOOT_IMG PARTS_DIR}
L=${2:?}
P=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
same() { cmp -s "$1" "$2" && echo same || echo differs; }
rm -f "$D/SHA256SUMS-display-late-l5"   # a failed run leaves no earlier sums standing
L4SUM=$(cd "$(dirname "$L")" && grep " boot-l4.img\$" SHA256SUMS-display-late-l4 | cut -d" " -f1)
ck "the control is L4: its sha256 is the one recorded (SHA256SUMS-display-late-l4)" "$(sha256sum "$L" | cut -d" " -f1)" "$L4SUM"
IMG=$D/boot-l5.img
lpoco-bringup image unpack "$L" --out "$T/l4" >/dev/null
lpoco-bringup image unpack "$IMG" --out "$T/l5" >/dev/null
hdr() { grep -v '^#' "$1/boot.toml"; }
ck "the ramdisk is L4's" "$(same "$T/l5/ramdisk" "$T/l4/ramdisk")" same
ck "the command line and header words are L4's" "$([ "$(hdr "$T/l5")" = "$(hdr "$T/l4")" ] && echo same || echo differs)" same
ck "the size is L4's" "$(stat -c %s "$IMG")" "$(stat -c %s "$L")"
cp "$IMG" "$T/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate passes (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
ck "the dtb is l5.dtb" "$(same "$T/l5/appended.dtb" "$P/l5.dtb")" same
head -c $(( $(stat -c %s "$T/l4/kernel") - $(stat -c %s "$T/l4/appended.dtb") )) "$T/l4/kernel" > "$T/Image.gz"
ck "the kernel is L4's Image.gz then l5.dtb" "$(cat "$T/Image.gz" "$P/l5.dtb" | cmp -s - "$T/l5/kernel" && echo same || echo differs)" same
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
dts "$T/l4/appended.dtb" > "$T/l4.dts"; dts "$T/l5/appended.dtb" > "$T/l5.dts"
ck "against L4's dtb (re-diffed from the images) the change is the one l5.diff records" "$(diff -u "$T/l4.dts" "$T/l5.dts" | sed 1,2d | cmp -s - "$P/l5.diff" && echo same || echo differs)" same
ck "the panel reset gpio has one claimant in the image's dtb (tests/host/display-reset-gpio-single-claimant.sh)" "$(DTB="$T/l5/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
ck "the same test fails on L4's dtb in L4's image (control)" "$(DTB="$T/l4/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" fail
(cd "$P" && sha256sum -c SHA256SUMS-display-late-l5-parts >/dev/null 2>&1) || { echo "check-display-late-l5: the parts do not match their sums" >&2; exit 1; }
[ $total -eq 0 ] || { echo "check-display-late-l5: $total checks failed, no sums written" >&2; exit 1; }
(cd "$D" && sha256sum boot-l5.img | tee SHA256SUMS-display-late-l5)
