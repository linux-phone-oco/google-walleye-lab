#!/bin/sh
# Counts what image L3 must be (item walleye-display-late-probe-l3-image), writes SHA256SUMS-display-late-l3 in IMAGES_DIR:
#   check-display-late-l3.sh IMAGES_DIR L1_BOOT_IMG PARTS_DIR     (PARTS_DIR: build-display-late-l3-dtb.sh's OUT_DIR; L1_BOOT_IMG: images-display-late-l/boot-l1.img)
# boot-l3.img is L1 with exactly the dtb appended to the kernel changed: ramdisk, command line, header words, size and Image.gz are L1's; the AVB footer verifies; tests/host/ufs-off-image.sh passes. Exit 1 on any FAIL.
set -eu
D=${1:?usage: check-display-late-l3.sh IMAGES_DIR L1_BOOT_IMG PARTS_DIR}
L=${2:?}
P=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
same() { cmp -s "$1" "$2" && echo same || echo differs; }
rm -f "$D/SHA256SUMS-display-late-l3"   # a failed run leaves no earlier sums standing
IMG=$D/boot-l3.img
lpoco-bringup image unpack "$L" --out "$T/l1" >/dev/null
lpoco-bringup image unpack "$IMG" --out "$T/l3" >/dev/null
hdr() { grep -v '^#' "$1/boot.toml"; }
ck "the ramdisk is L1's" "$(same "$T/l3/ramdisk" "$T/l1/ramdisk")" same
ck "the command line and header words are L1's" "$([ "$(hdr "$T/l3")" = "$(hdr "$T/l1")" ] && echo same || echo differs)" same
ck "the size is L1's" "$(stat -c %s "$IMG")" "$(stat -c %s "$L")"
cp "$IMG" "$T/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate passes (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
ck "the dtb is l3.dtb" "$(same "$T/l3/appended.dtb" "$P/l3.dtb")" same
head -c $(( $(stat -c %s "$T/l1/kernel") - $(stat -c %s "$T/l1/appended.dtb") )) "$T/l1/kernel" > "$T/Image.gz"
ck "the kernel is L1's Image.gz then l3.dtb" "$(cat "$T/Image.gz" "$P/l3.dtb" | cmp -s - "$T/l3/kernel" && echo same || echo differs)" same
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
dts "$T/l1/appended.dtb" > "$T/l1.dts"; dts "$T/l3/appended.dtb" > "$T/l3.dts"
ck "against L1's dtb (re-diffed from the image) only the vdd-supply line comes in" "$(diff "$T/l1.dts" "$T/l3.dts" | grep "^[<>]" | sed "s/^\([<>]\)[[:space:]]*/\1/" | tr "\n" "|")" ">vdd-supply = <0x25>;|"
DSI=$(fdtget "$T/l3/appended.dtb" /__symbols__ mdss_dsi0); L1A=$(fdtget "$T/l3/appended.dtb" /__symbols__ vreg_l1a_0p875)
ck "the dsi node's vdd-supply is the l1a rail's phandle" "$(fdtget "$T/l3/appended.dtb" "$DSI" vdd-supply)" "$(fdtget "$T/l3/appended.dtb" "$L1A" phandle)"
(cd "$P" && sha256sum -c SHA256SUMS-display-late-l3-parts >/dev/null 2>&1) || { echo "check-display-late-l3: the parts do not match their sums" >&2; exit 1; }
[ $total -eq 0 ] || { echo "check-display-late-l3: $total checks failed, no sums written" >&2; exit 1; }
(cd "$D" && sha256sum boot-l3.img | tee SHA256SUMS-display-late-l3)
