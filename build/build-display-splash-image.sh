#!/bin/sh
# Image B-s5 (item walleye-display-splash-region-dts; decision 2026-11-19-walleye-display-flip-dts-not-timing): B-d5 whose dtb reserves the bootloader's splash and holds it with a simple-framebuffer.
#   build-display-splash-image.sh BD5_BOOT_IMG OUT_DIR      (BD5_BOOT_IMG: build-display-delay-images.sh's boot-b-d5.img)
# The dtb is B-d5's own (as the L4 dtb is L3's: the display kernel's tree cannot compile today's board file) with what dt/msm8998-google-walleye-display-splash.dts adds: /reserved-memory/memory@9d400000 (no-map)
# and /chosen/framebuffer@9d400000 with the eight MDSS clocks of the mmcc and its MDSS_GDSC. Kernel, ramdisk (init-display-delay, 5 s), command line, header words and size are B-d5's: the proof is the delay bisect on it
# at 5 s (research walleye-display-delay-bisect-flips-at-25s: B-d5 shows the faults) - do the SMMU faults, the pclk0 rcg warning and the DSI -110 vanish?
# Writes OUT_DIR/{boot-b-s5.img,splash.dtb,b-s5.diff (B-d5's decompiled dtb against it),SHA256SUMS-display-splash}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
IMG=${1:?usage: build-display-splash-image.sh BD5_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
TEST=$HERE/tests/host/dtb-display-splash.sh
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
[ -s "$IMG" ] || { echo "build-display-splash-image: missing $IMG" >&2; exit 1; }
SUM=$(cd "$(dirname "$IMG")" && grep " boot-b-d5.img\$" SHA256SUMS-display-delay | cut -d" " -f1)
ck "the input is B-d5: its sha256 is the one recorded (SHA256SUMS-display-delay)" "$(sha256sum "$IMG" | cut -d" " -f1)" "$SUM"
dts() { dtc -I dtb -O dts -q "$1" 2>/dev/null; }
same() { cmp -s "$1" "$2" && echo same || echo differs; }
make() { # DIR: the image and dtb in DIR
  d=$1; mkdir -p "$d"
  cp "$T/b5/appended.dtb" "$d/splash.dtb"; S=$d/splash.dtb
  MMCC=$(fdtget "$S" "$(fdtget "$S" /__symbols__ mmcc)" phandle)
  fdtput -c "$S" /reserved-memory/memory@9d400000
  fdtput -t x "$S" /reserved-memory/memory@9d400000 reg 0 9d400000 0 2400000
  fdtput "$S" /reserved-memory/memory@9d400000 no-map
  fdtput -t s "$S" /__symbols__ cont_splash_mem /reserved-memory/memory@9d400000
  F=/chosen/framebuffer@9d400000
  fdtput -t x "$S" /chosen '#address-cells' 2
  fdtput -t x "$S" /chosen '#size-cells' 2
  fdtput "$S" /chosen ranges
  fdtput -c "$S" "$F"
  fdtput -t s "$S" "$F" compatible simple-framebuffer
  fdtput -t x "$S" "$F" reg 0 9d400000 0 2400000
  fdtput -t u "$S" "$F" width 1080
  fdtput -t u "$S" "$F" height 1920
  fdtput -t u "$S" "$F" stride 4320
  fdtput -t s "$S" "$F" format a8r8g8b8
  fdtput -t u "$S" "$F" clocks $MMCC 62 $MMCC 64 $MMCC 70 $MMCC 67 $MMCC 72 $MMCC 82 $MMCC 65 $MMCC 74
  fdtput -t u "$S" "$F" power-domains $MMCC 4
  fdtput -t s "$S" /__symbols__ framebuffer0 "$F"
  head -c $(( $(stat -c %s "$T/b5/kernel") - $(stat -c %s "$T/b5/appended.dtb") )) "$T/b5/kernel" > "$d/Image.gz"
  lpoco-bringup image swap "$IMG" --kernel "$d/Image.gz" --dtb "$S" --out "$d/boot.img" >/dev/null
}
lpoco-bringup image unpack "$IMG" --out "$T/b5" >/dev/null
ck "the test fails on B-d5's dtb (control: no splash region, no framebuffer)" "$(DTB="$T/b5/appended.dtb" sh "$TEST" | cut -d' ' -f1 || true)" fail
make "$T/s"; make "$T/again"
ck "made twice, the two images are identical (rebuild identical)" "$(same "$T/s/boot.img" "$T/again/boot.img")" same
lpoco-bringup image unpack "$T/s/boot.img" --out "$T/u" >/dev/null
ck "the image's dtb is the made dtb" "$(same "$T/u/appended.dtb" "$T/s/splash.dtb")" same
ck "the test passes on the made dtb (tests/host/dtb-display-splash.sh)" "$(DTB="$T/s/splash.dtb" sh "$TEST" | cut -d' ' -f1 || true)" pass
ck "the image's kernel is B-d5's Image.gz then the made dtb, byte for byte" "$(cat "$T/s/Image.gz" "$T/s/splash.dtb" | cmp -s - "$T/u/kernel" && echo same || echo differs) $(cat "$T/s/Image.gz" "$T/b5/appended.dtb" | cmp -s - "$T/b5/kernel" && echo same || echo differs)" "same same"
ck "the ramdisk is B-d5's, byte for byte (init-display-delay, /display-delay 5)" "$(same "$T/u/ramdisk" "$T/b5/ramdisk")" same
ck "the command line and header words are B-d5's" "$([ "$(grep -v '^#' "$T/u/boot.toml")" = "$(grep -v '^#' "$T/b5/boot.toml")" ] && echo same || echo differs)" same
ck "the size is B-d5's" "$(stat -c %s "$T/s/boot.img")" "$(stat -c %s "$IMG")"
dts "$T/b5/appended.dtb" | sed 's/phandle = <0x[0-9a-f]*>;/phandle;/' > "$T/old.dts"
dts "$T/s/splash.dtb" | sed 's/phandle = <0x[0-9a-f]*>;/phandle;/' > "$T/new.dts"
diff -u "$T/old.dts" "$T/new.dts" | sed '1,2d' > "$T/b-s5.diff" || true
ck "against B-d5's dtb no line is removed and the symbols table gains two labels" "$(grep -c '^-' "$T/b-s5.diff") $(grep -c 'cont_splash_mem = \|framebuffer0 = ' "$T/b-s5.diff")" "0 2"
ck "the dtb's added lines are the two nodes, /chosen's three properties and the node properties (21 lines in all)" "$(grep -c '^+' "$T/b-s5.diff")" 21
mkdir "$T/v" && cp "$T/s/boot.img" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$T/s/boot.img" >/dev/null 2>&1 && echo pass || echo fail)" pass
ck "the UFS-off tests pass (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$T/s/boot.img" | cut -d' ' -f1 || true)" pass
ck "the panel reset gpio has one claimant in the image's dtb" "$(DTB="$T/u/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
ck "the stock overlay still merges into the dtb (tests/host/dtbo-overlay-merges.sh)" "$(DTB="$T/s/splash.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "the touch power gpio keeps one claimant (tests/host/touch-gpio-single-claimant.sh)" "$(DTB="$T/s/splash.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
[ $total -eq 0 ] || { echo "build-display-splash-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"; rm -f "$OUT/boot-b-s5.img" "$OUT/splash.dtb" "$OUT/b-s5.diff" "$OUT/SHA256SUMS-display-splash"
cp "$T/s/boot.img" "$OUT/boot-b-s5.img"; cp "$T/s/splash.dtb" "$OUT/splash.dtb"; cp "$T/b-s5.diff" "$OUT/b-s5.diff"
(cd "$OUT" && sha256sum boot-b-s5.img splash.dtb b-s5.diff | tee SHA256SUMS-display-splash)
