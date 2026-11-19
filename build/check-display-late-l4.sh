#!/bin/sh
# Counts what image L4 must be (item walleye-display-late-probe-l4-pinctrl-labels-image), writes SHA256SUMS-display-late-l4 in IMAGES_DIR:
#   check-display-late-l4.sh IMAGES_DIR L3_BOOT_IMG PARTS_DIR     (PARTS_DIR: build-display-late-l4-dtb.sh's OUT_DIR; L3_BOOT_IMG: images-display-late-l3/boot-l3.img)
# boot-l4.img is L3 with exactly the dtb appended to the kernel changed: ramdisk, command line, header words, size and Image.gz are L3's; the AVB footer verifies; tests/host/ufs-off-image.sh passes;
# tests/host/dtb-display-pinctrl-labels.sh passes on the dtb inside the image. Exit 1 on any FAIL.
set -eu
D=${1:?usage: check-display-late-l4.sh IMAGES_DIR L3_BOOT_IMG PARTS_DIR}
L=${2:?}
P=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
same() { cmp -s "$1" "$2" && echo same || echo differs; }
rm -f "$D/SHA256SUMS-display-late-l4"   # a failed run leaves no earlier sums standing
L3SUM=$(cd "$(dirname "$L")" && grep " boot-l3.img\$" SHA256SUMS-display-late-l3 | cut -d" " -f1)
ck "the control is L3: its sha256 is the one recorded (SHA256SUMS-display-late-l3)" "$(sha256sum "$L" | cut -d" " -f1)" "$L3SUM"
IMG=$D/boot-l4.img
lpoco-bringup image unpack "$L" --out "$T/l3" >/dev/null
lpoco-bringup image unpack "$IMG" --out "$T/l4" >/dev/null
hdr() { grep -v '^#' "$1/boot.toml"; }
ck "the ramdisk is L3's" "$(same "$T/l4/ramdisk" "$T/l3/ramdisk")" same
ck "the command line and header words are L3's" "$([ "$(hdr "$T/l4")" = "$(hdr "$T/l3")" ] && echo same || echo differs)" same
ck "the size is L3's" "$(stat -c %s "$IMG")" "$(stat -c %s "$L")"
cp "$IMG" "$T/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate passes (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
ck "the dtb is l4.dtb" "$(same "$T/l4/appended.dtb" "$P/l4.dtb")" same
head -c $(( $(stat -c %s "$T/l3/kernel") - $(stat -c %s "$T/l3/appended.dtb") )) "$T/l3/kernel" > "$T/Image.gz"
ck "the kernel is L3's Image.gz then l4.dtb" "$(cat "$T/Image.gz" "$P/l4.dtb" | cmp -s - "$T/l4/kernel" && echo same || echo differs)" same
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
dts "$T/l3/appended.dtb" > "$T/l3.dts"; dts "$T/l4/appended.dtb" > "$T/l4.dts"
ck "against L3's dtb (re-diffed from the images) the change is the one l4.diff records" "$(diff -u "$T/l3.dts" "$T/l4.dts" | sed 1,2d | cmp -s - "$P/l4.diff" && echo same || echo differs)" same
ck "the four display pinctrl labels name real nodes in the image's dtb (tests/host/dtb-display-pinctrl-labels.sh)" "$(DTB="$T/l4/appended.dtb" sh "$HERE/tests/host/dtb-display-pinctrl-labels.sh" | cut -d' ' -f1 || true)" pass
ck "the same test fails on L3's dtb in L3's image (control)" "$(DTB="$T/l3/appended.dtb" sh "$HERE/tests/host/dtb-display-pinctrl-labels.sh" | cut -d' ' -f1 || true)" fail
(cd "$P" && sha256sum -c SHA256SUMS-display-late-l4-parts >/dev/null 2>&1) || { echo "check-display-late-l4: the parts do not match their sums" >&2; exit 1; }
[ $total -eq 0 ] || { echo "check-display-late-l4: $total checks failed, no sums written" >&2; exit 1; }
(cd "$D" && sha256sum boot-l4.img | tee SHA256SUMS-display-late-l4)
