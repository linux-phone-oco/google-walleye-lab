#!/bin/sh
# The image of the EXACT upstream board dts (backlog walleye-upstream-dts-image; research walleye-upstream-dts-series): z21 with its dtb replaced by one built from the
# series' msm8998-google-walleye.dts (research/walleye-first-fastboot-boot/2026-11-19-upstream-dts/, two patches on Linux v7.3), applied to the kernel tree and not changed by a byte.
# Kernel (Image.gz), ramdisk (the z14 network shell), header, command line and AVB footer are z21's, byte for byte; only the dtb differs.
# What the series' dts cannot carry, and the lab's dts has: the stub nodes the bootloader's overlay merge needs (the vendor dtbo targets 137 labels; a dtb without them: "FDT_ERR_NOTFOUND",
# see the first check). They are generated from the series' own dtb by build/gen-overlay-stubs.sh and added by a wrapper dts that includes the series' file unchanged (the wrapper and
# the stubs are in research/.../2026-11-19-upstream-dts-image/); the lab's clock_gcc fix is the same fdtput as every image since z3 (the stub label is retargeted at the real gcc).
# The touch node and the pwm of the lab's dts are not in the series: no touch is expected.
#   build-upstream-dts-image.sh Z21_BOOT_IMG KERNEL_TREE OUT_DIR
# KERNEL_TREE: a v7.3 tree with the series' two patches applied (git apply; checked here with git apply -R --check); it gets three files and two Makefile lines from here, taken away again at the end. The script builds in its own
# O= under OUT_DIR/.work and writes OUT_DIR/{boot-upstream-dts.img,msm8998-google-walleye-series.dtb,SHA256SUMS-upstream-dts};
# every check stops the build (ok/FAIL, exit 1, nothing written). Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
Z21=${1:?usage: build-upstream-dts-image.sh Z21_BOOT_IMG KERNEL_TREE OUT_DIR}
K=${2:?}
OUT=${3:?}
case $OUT in /*) ;; *) echo "build-upstream-dts-image: OUT_DIR must be absolute" >&2; exit 1;; esac
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
SER=$HERE/research/walleye-first-fastboot-boot/2026-11-19-upstream-dts
MAT=$HERE/research/walleye-first-fastboot-boot/2026-11-19-upstream-dts-image
QCOM=$K/arch/arm64/boot/dts/qcom
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
# the tree is left as it was found: what this script adds (the wrapper files, their Makefile lines) is taken away at the start and at the end
tidy() { sed -i '/msm8998-google-walleye-series/d' "$QCOM/Makefile"; rm -f "$QCOM"/msm8998-google-walleye-series*; }
tidy
trap 'tidy; rm -rf "$T"' EXIT
for f in "$Z21" "$DTBO" "$SER/0001-dt-bindings-arm-qcom-Add-Google-Pixel-2.patch" "$SER/0002-arm64-dts-qcom-msm8998-Add-Google-Pixel-2-walleye.patch" "$MAT/msm8998-google-walleye-series.dts" "$MAT/msm8998-google-walleye-series-stubs.dtsi" "$QCOM/msm8998-google-walleye.dts"; do
  [ -s "$f" ] || { echo "build-upstream-dts-image: missing $f" >&2; exit 1; }
done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-upstream-dts-image: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
# the tree is the series, applied
ck "the kernel tree is v7.3" "$(git -C "$K" describe --tags --exact-match HEAD 2>/dev/null)" v7.3
ck "the tree holds the series' patch 0001, exactly" "$(git -C "$K" apply -R --check "$SER/0001-dt-bindings-arm-qcom-Add-Google-Pixel-2.patch" 2>&1 && echo yes || echo no)" yes
ck "the tree holds the series' patch 0002, exactly (the dts is the series' file)" "$(git -C "$K" apply -R --check "$SER/0002-arm64-dts-qcom-msm8998-Add-Google-Pixel-2-walleye.patch" 2>&1 && echo yes || echo no)" yes
ck "nothing else of the tree is changed: only qcom.yaml, the Makefile and the series' dts differ from v7.3" "$(git -C "$K" status --porcelain | sed 's/^ *//' | LC_ALL=C sort | tr '\n' '|')" "?? arch/arm64/boot/dts/qcom/msm8998-google-walleye.dts|M Documentation/devicetree/bindings/arm/qcom.yaml|M arch/arm64/boot/dts/qcom/Makefile|"
INFO=$(unpack_bootimg --boot_img "$Z21" --out "$T/z21")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z21/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z21")
AV=$(avbtool info_image --image "$Z21")
# 1: the series' dtb as it is, with symbols: the overlay does not merge (the finding)
W=$OUT/.work/b
mkdir -p "$W"
M="make -C $K ARCH=arm64 LLVM=1 O=$W"
$M defconfig >"$T/make.log" 2>&1 || { tail -n 20 "$T/make.log" >&2; exit 1; }
export DTC_FLAGS=-@
$M -j"$(nproc)" qcom/msm8998-google-walleye.dtb >"$T/make.log" 2>&1 || { tail -n 20 "$T/make.log" >&2; exit 1; }
cp "$W/arch/arm64/boot/dts/qcom/msm8998-google-walleye.dtb" "$T/series.dtb"
ck "the series' dtb with symbols is not merged by the vendor overlay (the stubs are needed)" "$(DTB="$T/series.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | sed -n 's/^\(fail\).*\(FDT_ERR_NOTFOUND\).*/\1 \2/p' | head -n1)" "fail FDT_ERR_NOTFOUND"
# 2: the series' dts with the two overlay-collision deletions (the base), its dtb, and the stubs generated from it: the committed ones
mkdtb() { # NAME: the dts of that name is in the tree; add its Makefile line, build it
  grep -q "msm8998-google-walleye-$1.dtb" "$QCOM/Makefile" ||
    sed -i "s/^dtb-\$(CONFIG_ARCH_QCOM)\t+= msm8998-google-walleye.dtb\$/&\ndtb-\$(CONFIG_ARCH_QCOM)\t+= msm8998-google-walleye-$1.dtb/" "$QCOM/Makefile"
  grep -q "msm8998-google-walleye-$1.dtb" "$QCOM/Makefile" || { echo "build-upstream-dts-image: Makefile line for $1 not added" >&2; exit 1; }
  $M -j"$(nproc)" "qcom/msm8998-google-walleye-$1.dtb" >"$T/make.log" 2>&1 || { tail -n 20 "$T/make.log" >&2; exit 1; }
  cp "$W/arch/arm64/boot/dts/qcom/msm8998-google-walleye-$1.dtb" "$T/$1.dtb"
}
cp "$MAT/msm8998-google-walleye-series-base.dts" "$QCOM/"
mkdtb series-base
e=$(xxd -s $((32+32*3)) -l 8 -p "$DTBO"); size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
tail -c +$((off+1)) "$DTBO" | head -c "$size" > "$T/ov3.dtb"
sh "$HERE/build/gen-overlay-stubs.sh" "$T/ov3.dtb" "$T/series-base.dtb" "$T/stubs.dtsi"
ck "the stubs generated from the base dtb are the committed ones, byte for byte" "$(cmp -s "$T/stubs.dtsi" "$MAT/msm8998-google-walleye-series-stubs.dtsi" && echo same || echo differs)" same
# 3: the stubbed dtb
cp "$MAT/msm8998-google-walleye-series.dts" "$MAT/msm8998-google-walleye-series-stubs.dtsi" "$QCOM/"
mkdtb series
GCC=/soc@0/clock-controller@100000
cp "$T/series.dtb" "$T/new.dtb"
ck "the stubbed dtb has clock_gcc at the empty stub" "$(fdtget "$T/new.dtb" /__symbols__ clock_gcc)" /vendor-overlay-stubs/clock_gcc
fdtput -t s "$T/new.dtb" /__symbols__ clock_gcc "$GCC"
ck "clock_gcc is the real gcc in the image dtb" "$(fdtget "$T/new.dtb" /__symbols__ clock_gcc)" "$GCC"
ck "the stub container is disabled" "$(fdtget "$T/new.dtb" /vendor-overlay-stubs status)" disabled
ck "the model is the series'" "$(fdtget "$T/new.dtb" / model)" "Google Pixel 2"
ck "the compatible is the series': google,walleye then qcom,msm8998" "$(fdtget "$T/new.dtb" / compatible)" "google,walleye qcom,msm8998"
ck "UFS is off in the series' dts: ufshc and its phy are disabled" "$(fdtget "$T/new.dtb" /soc@0/ufshc@1da4000 status) $(fdtget "$T/new.dtb" /soc@0/phy@1da7000 status)" "disabled disabled"
ck "gpio-reserved-ranges <0 4 81 4> (the series has it)" "$(fdtget -t u "$T/new.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "the ramoops node at 0xa1100000" "$(fdtget -t x "$T/new.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "the debug uart is enabled" "$(fdtget "$T/new.dtb" /soc@0/serial@c1b0000 status)" okay
DTS=$(dtc -I dtb -O dts "$T/new.dtb" 2>/dev/null) || { echo "build-upstream-dts-image: dtc cannot read the dtb" >&2; exit 1; }
ck "no touch node (not in the series): neither the touch regulator nor a synaptics node" "$(printf '%s\n' "$DTS" | grep -c -e 'regulator-touch-power' -e 'syna,rmi4' || true)" 0
SOCN=$(fdtget -l "$T/new.dtb" /soc@0) || { echo "build-upstream-dts-image: fdtget cannot list /soc@0" >&2; exit 1; }
ck "no APSS watchdog node (the series dropped it; msm8998.dtsi has none)" "$(printf '%s\n' "$SOCN" | grep -c watchdog || true)" 0
# the dtb after the overlay merge
ck "the overlay merges and overwrites nothing of the series' (tests/host/overlay-collisions.sh)" "$(DTB="$T/new.dtb" QUIET=1 sh "$HERE/tests/host/overlay-collisions.sh" | tail -n1 | cut -d' ' -f1 || true)" pass
ck "the overlay merges into the dtb (tests/host/dtbo-overlay-merges.sh)" "$(DTB="$T/new.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1 || true)" pass
ck "after the overlay merge the clock users name gcc (tests/host/overlay-clock-gcc.sh)" "$(DTB="$T/new.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1 || true)" pass
ck "the GPU smmu is disabled (tests/host/adreno-smmu-disabled.sh)" "$(DTB="$T/new.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d' ' -f1 || true)" pass
# the image
cat "$T/k.gz" "$T/new.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-upstream-dts.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z21/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "Image.gz is z21's, byte for byte" "$(cmp -s "$T/uk.gz" "$T/k.gz" && echo same || echo differs)" same
ck "the dtb is the new one, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/new.dtb" && echo same || echo differs)" same
ck "the dtb differs from z21's" "$(cmp -s "$T/uk.dtb" "$T/k.dtb" && echo same || echo differs)" differs
ck "header (command line included) equals z21's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "ramdisk bytes equal z21's" "$(cmp -s "$T/u/ramdisk" "$T/z21/ramdisk" && echo same || echo differs)" same
ck "size is z21's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "AVB info equals z21's but the digest and the sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')"
KS=$(stat -c %s "$T/blob"); KO_=$(printf %d "$(field "kernel load address")"); RO=$(printf %d "$(field "ramdisk load address")")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate passes the image (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
[ $total -eq 0 ] || { echo "build-upstream-dts-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-upstream-dts.img"; cp "$T/new.dtb" "$OUT/msm8998-google-walleye-series.dtb"
(cd "$OUT" && sha256sum boot-upstream-dts.img msm8998-google-walleye-series.dtb | tee SHA256SUMS-upstream-dts)
