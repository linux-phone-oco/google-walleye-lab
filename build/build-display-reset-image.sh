#!/bin/sh
# Image B-r5 (item walleye-display-reset-image; decision 2026-11-20-walleye-display-handover-reset-first): B-d5 whose mdss node resets its block at probe.
#   build-display-reset-image.sh BD5_BOOT_IMG OUT_DIR      (BD5_BOOT_IMG: build-display-delay-images.sh's boot-b-d5.img)
# The dtb is B-d5's own (the display kernel's tree cannot compile today's board file) with one property added: resets = <&mmcc MDSS_BCR> (5) on /soc@0/display-subsystem@c900000;
# msm_mdss_reset takes the one unnamed reset and pulses it before msm touches the block. Kernel, ramdisk (init-display-delay, 5 s), command line, header words and size are B-d5's: the proof is
# the delay bisect on it at 5 s (image_prefix=b-r) - do the SMMU faults, the pclk0 rcg warning and the DSI -110 vanish? (research walleye-display-handover-how-others-found: B-s5 left them).
# Writes OUT_DIR/{boot-b-r5.img,reset.dtb,b-r5.diff (B-d5's decompiled dtb against it),SHA256SUMS-display-reset}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
IMG=${1:?usage: build-display-reset-image.sh BD5_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
TEST=$HERE/tests/host/dtb-display-reset.sh
MDSS=/soc@0/display-subsystem@c900000
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
[ -s "$IMG" ] || { echo "build-display-reset-image: missing $IMG" >&2; exit 1; }
SUM=$(cd "$(dirname "$IMG")" && grep " boot-b-d5.img\$" SHA256SUMS-display-delay | cut -d" " -f1)
ck "the input is B-d5: its sha256 is the one recorded (SHA256SUMS-display-delay)" "$(sha256sum "$IMG" | cut -d" " -f1)" "$SUM"
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
same() { cmp -s "$1" "$2" && echo same || echo differs; }
make() { # DIR: the image and dtb in DIR
  d=$1; mkdir -p "$d"
  cp "$T/b5/appended.dtb" "$d/reset.dtb"; S=$d/reset.dtb
  MMCC=$(fdtget "$S" "$(fdtget "$S" /__symbols__ mmcc)" phandle)
  fdtput -t u "$S" "$MDSS" resets "$MMCC" 5
  head -c $(( $(stat -c %s "$T/b5/kernel") - $(stat -c %s "$T/b5/appended.dtb") )) "$T/b5/kernel" > "$d/Image.gz"
  lpoco-bringup image swap "$IMG" --kernel "$d/Image.gz" --dtb "$S" --out "$d/boot.img" >/dev/null
}
lpoco-bringup image unpack "$IMG" --out "$T/b5" >/dev/null
ck "the test fails on B-d5's dtb (control: no resets on mdss)" "$(DTB="$T/b5/appended.dtb" sh "$TEST" | cut -d' ' -f1 || true)" fail
make "$T/s"; make "$T/again"
ck "made twice, the two images are identical (rebuild identical)" "$(same "$T/s/boot.img" "$T/again/boot.img")" same
lpoco-bringup image unpack "$T/s/boot.img" --out "$T/u" >/dev/null
ck "the image's dtb is the made dtb" "$(same "$T/u/appended.dtb" "$T/s/reset.dtb")" same
ck "the test passes on the made dtb (tests/host/dtb-display-reset.sh)" "$(DTB="$T/s/reset.dtb" sh "$TEST" | cut -d' ' -f1 || true)" pass
ck "the image's kernel is B-d5's Image.gz then the made dtb, byte for byte" "$(cat "$T/s/Image.gz" "$T/s/reset.dtb" | cmp -s - "$T/u/kernel" && echo same || echo differs) $(cat "$T/s/Image.gz" "$T/b5/appended.dtb" | cmp -s - "$T/b5/kernel" && echo same || echo differs)" "same same"
ck "the ramdisk is B-d5's, byte for byte (init-display-delay, /display-delay 5)" "$(same "$T/u/ramdisk" "$T/b5/ramdisk")" same
ck "the command line and header words are B-d5's" "$([ "$(grep -v '^#' "$T/u/boot.toml")" = "$(grep -v '^#' "$T/b5/boot.toml")" ] && echo same || echo differs)" same
ck "the size is B-d5's" "$(stat -c %s "$T/s/boot.img")" "$(stat -c %s "$IMG")"
dts "$T/b5/appended.dtb" | sed 's/phandle = <0x[0-9a-f]*>;/phandle;/' > "$T/old.dts"
dts "$T/s/reset.dtb" | sed 's/phandle = <0x[0-9a-f]*>;/phandle;/' > "$T/new.dts"
diff -u "$T/old.dts" "$T/new.dts" | sed '1,2d' > "$T/b-r5.diff" || true
ck "against B-d5's dtb no line is removed and one is added (the resets property)" "$(grep -c '^-' "$T/b-r5.diff") $(grep -c '^+' "$T/b-r5.diff") $(grep -c '^+.*resets = ' "$T/b-r5.diff")" "0 1 1"
mkdir "$T/v" && cp "$T/s/boot.img" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$T/s/boot.img" >/dev/null 2>&1 && echo pass || echo fail)" pass
ck "the UFS-off tests pass (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$T/s/boot.img" | cut -d' ' -f1 || true)" pass
ck "the panel reset gpio has one claimant in the image's dtb" "$(DTB="$T/u/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
ck "the stock overlay still merges into the dtb (tests/host/dtbo-overlay-merges.sh)" "$(DTB="$T/s/reset.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "the touch power gpio keeps one claimant (tests/host/touch-gpio-single-claimant.sh)" "$(DTB="$T/s/reset.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
[ $total -eq 0 ] || { echo "build-display-reset-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"; rm -f "$OUT/boot-b-r5.img" "$OUT/reset.dtb" "$OUT/b-r5.diff" "$OUT/SHA256SUMS-display-reset"
cp "$T/s/boot.img" "$OUT/boot-b-r5.img"; cp "$T/s/reset.dtb" "$OUT/reset.dtb"; cp "$T/b-r5.diff" "$OUT/b-r5.diff"
(cd "$OUT" && sha256sum boot-b-r5.img reset.dtb b-r5.diff | tee SHA256SUMS-display-reset)
