#!/bin/sh
# z2 (backlog walleye-dmesg-clean): z2 with ONE more change, the kernel: CONFIG_MSM_GPUCC_8998=y instead of =m. The initramfs carries no modules, so the GPU clock
# controller (gpucc, the power domain arm-smmu 5040000 waits on) never probed and the smmu timed out with -110. Same dtb as z2 (dt/ now), same ramdisk, header, command line, AVB footer.
#   build-z2.sh WORK_DIR Y2_BOOT_IMG KERNEL_OUT_DIR OUT_DIR
# KERNEL_OUT_DIR is build-kernel.sh's OUT_DIR run with the walleye.config that has the line (Image.gz, .config). Its .config must differ from BASE_CFG (the kernel of y2, default WORK_DIR/out-ml/.config) in the gpucc line only.
# The dtb is compiled here from dt/ the way build-z.sh does (DTC_FLAGS=-@, out of tree in WORK_DIR/out-z).
# Writes OUT_DIR/{boot-z2.img,msm8998-google-walleye-z.dtb,Image-gpucc.gz,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL, nothing written into OUT_DIR then).
# Environment: KERNEL_TREE (default WORK_DIR/linux-v7.3), DTBO (default WORK_DIR/control/dtbo.img), BASE_CFG.
set -eu
W=${1:?usage: build-z2.sh WORK_DIR Y2_BOOT_IMG KERNEL_OUT_DIR OUT_DIR}
Y2=${2:?}
KO=${3:?}
OUT=${4:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
K=${KERNEL_TREE:-$W/linux-v7.3}
export DTBO=${DTBO:-$W/control/dtbo.img}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
BASE_CFG=${BASE_CFG:-$W/out-ml/.config}
for f in "$KO/Image.gz" "$KO/.config" "$BASE_CFG" "$Y2" "$HERE/dt/msm8998-google-walleye.dts" "$K/Makefile" "$DTBO"; do [ -s "$f" ] || { echo "build-z2: missing $f" >&2; exit 1; }; done
# split KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
split() {
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z2: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
# 1. our dtb from dt/, compiled by the kernel tree
QCOM=$K/arch/arm64/boot/dts/qcom
cp "$HERE/dt/msm8998-google-walleye-overlay-stubs.dtsi" "$HERE/dt/msm8998-google-walleye.dts" "$HERE/dt/msm8998-google-walleye-f.dts" "$QCOM/"
grep -q 'msm8998-google-walleye.dtb' "$QCOM/Makefile" || { echo "build-z2: the kernel tree's Makefile lacks the walleye dtb line (run build-kernel.sh once)" >&2; exit 1; }
mkdir -p "$W/out-z"
M="make -C $K ARCH=arm64 LLVM=1 O=$W/out-z"
$M defconfig >/dev/null
DTC_FLAGS=-@ $M -j"$(nproc)" qcom/msm8998-google-walleye.dtb >"$T/dtb-make.log" 2>&1 || { cat "$T/dtb-make.log" >&2; exit 1; }
cp "$W/out-z/arch/arm64/boot/dts/qcom/msm8998-google-walleye.dtb" "$T/z.dtb"
# 2. the source image (its hash is printed: the recipe's input)
echo "input $(h "$Y2")  $Y2"
INFO=$(unpack_bootimg --boot_img "$Y2" --out "$T/y2")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/y2/kernel" "$T/k"
PSIZE=$(stat -c %s "$Y2")
AV=$(avbtool info_image --image "$Y2")
cat "$KO/Image.gz" "$T/z.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
# the dtb itself: the SPMI children the first mainline dmesg lacked
S=/soc@0/spmi@800f000
ck "z.dtb: pm8998 at $S/pmic@0, reg <0 0>" "$(fdtget -t u "$T/z.dtb" "$S/pmic@0" reg 2>/dev/null || echo none)" "0 0"
ck "z.dtb: pmi8998 at $S/pmic@2, reg <2 0>" "$(fdtget -t u "$T/z.dtb" "$S/pmic@2" reg 2>/dev/null || echo none)" "2 0"
ck "z.dtb: pmi8998 at $S/pmic@3, reg <3 0>" "$(fdtget -t u "$T/z.dtb" "$S/pmic@3" reg 2>/dev/null || echo none)" "3 0"
ck "z.dtb: every child of the SPMI bus in the base dtb has a reg property" "$(for c in $(fdtget -l "$T/z.dtb" "$S"); do fdtget -t u "$T/z.dtb" "$S/$c" reg >/dev/null 2>&1 || echo "$c"; done | tr '\n' ' ')" ""
ck "y2's dtb has the SPMI bus but no PMIC node (the cause)" "$(fdtget -l "$T/k.dtb" "$S" >/dev/null 2>&1 && echo bus; fdtget -l "$T/k.dtb" "$S" 2>/dev/null | grep -c pmic || true)" "$(printf 'bus\n0')"
ck "z.dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/z.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "z.dtb keeps qcom,msm-id <0x124 0x20001>" "$(fdtget -t x "$T/z.dtb" / qcom,msm-id)" "124 20001"
ck "z.dtb has __symbols__ (the overlay merge needs them)" "$(fdtget -l "$T/z.dtb" / | grep -cx __symbols__)" 1
ck "the dtbo overlay merges into z.dtb" "$(DTB="$T/z.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
# 3. the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z2.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/y2/ramdisk" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/uz")
split "$T/uz/kernel" "$T/ukz"
ck "z2 Image.gz is the new build's, byte for byte" "$(cmp -s "$T/ukz.gz" "$KO/Image.gz" && echo same || echo differs)" same
ck "z2 Image.gz is not y2's (the one change)" "$(cmp -s "$T/ukz.gz" "$T/k.gz" && echo same || echo differs)" differs
ck "the new .config has MSM_GPUCC_8998=y" "$(grep -c '^CONFIG_MSM_GPUCC_8998=y$' "$KO/.config")" 1
ck "the new .config differs from y2's kernel config in the gpucc line only (a depends-on-it line may follow)" "$(diff "$BASE_CFG" "$KO/.config" | grep '^[<>]' | grep -vc 'GPUCC_8998')" 0
ck "z2 dtb is z.dtb, byte for byte" "$(cmp -s "$T/ukz.dtb" "$T/z.dtb" && echo same || echo differs)" same
ck "z2 header (offsets, page, os word, command line) equals y2's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z2 ramdisk (y2's gadget init) bytes equal y2's" "$(cmp -s "$T/uz/ramdisk" "$T/y2/ramdisk" && echo same || echo differs)" same
ck "z2 size is y2's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z2 AVB info equals y2's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/vz" && cp "$IMG" "$T/vz/boot.img"
ck "z2 AVB footer verifies" "$(avbtool verify_image --image "$T/vz/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z2: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z2.img"; cp "$T/z.dtb" "$OUT/msm8998-google-walleye-z.dtb"; cp "$KO/Image.gz" "$OUT/Image-gpucc.gz"
(cd "$OUT" && sha256sum boot-z2.img msm8998-google-walleye-z.dtb Image-gpucc.gz | tee SHA256SUMS)
