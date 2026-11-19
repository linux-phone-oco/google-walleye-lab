#!/bin/sh
# c1 (backlog walleye-charger-smb2): z3 with TWO changes: the kernel has the SMB2 charger and the round-robin ADC built in (CONFIG_CHARGER_QCOM_SMB2=y, CONFIG_QCOM_SPMI_RRADC=y),
# and the dtb (ours, from dt/) enables pmi8998's charger with a simple-battery of 2700 mAh / 4.4 V - plus z3's own dtb fix, /__symbols__/clock_gcc on the real gcc. Ramdisk, header,
# command line and AVB footer are z3's.
#   build-c1.sh Z3_BOOT_IMG KERNEL_OUT_DIR OUT_DIR
# KERNEL_OUT_DIR is build-kernel.sh's OUT_DIR for the walleye.config that has the two lines (Image.gz, msm8998-google-walleye.dtb built with -@, .config).
# Writes OUT_DIR/{boot-c1.img,msm8998-google-walleye-c1.dtb,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
# Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img), BASE_CFG (z2's kernel config, default ~/Projects/walleye-build/out-z2/.config).
set -eu
Z3=${1:?usage: build-c1.sh Z3_BOOT_IMG KERNEL_OUT_DIR OUT_DIR}
KO=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
BASE_CFG=${BASE_CFG:-$HOME/Projects/walleye-build/out-z2/.config}
GCC=/soc@0/clock-controller@100000
PMI=/soc@0/spmi@800f000/pmic@2
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z3" "$DTBO" "$KO/Image.gz" "$KO/.config" "$KO/msm8998-google-walleye.dtb" "$BASE_CFG"; do [ -s "$f" ] || { echo "build-c1: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-c1: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z3")  $Z3"
INFO=$(unpack_bootimg --boot_img "$Z3" --out "$T/z3")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z3/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z3")
AV=$(avbtool info_image --image "$Z3")
cp "$KO/msm8998-google-walleye.dtb" "$T/c1.dtb"
fdtput -t s "$T/c1.dtb" /__symbols__ clock_gcc "$GCC"
cat "$KO/Image.gz" "$T/c1.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
# the kernel
ck ".config has CHARGER_QCOM_SMB2=y" "$(grep -c '^CONFIG_CHARGER_QCOM_SMB2=y$' "$KO/.config")" 1
ck ".config has QCOM_SPMI_RRADC=y" "$(grep -c '^CONFIG_QCOM_SPMI_RRADC=y$' "$KO/.config")" 1
ck ".config differs from z2's kernel config in the charger and rradc lines only (depends-on lines may follow)" "$(diff "$BASE_CFG" "$KO/.config" | grep '^[<>]' | grep -vc -e 'CHARGER_QCOM_SMB2' -e 'QCOM_SPMI_RRADC')" 0
ck "the new Image.gz is not z3's (the kernel change)" "$(cmp -s "$KO/Image.gz" "$T/k.gz" && echo same || echo differs)" differs
ck "Image.gz holds the charger driver" "$(zcat "$KO/Image.gz" | grep -ac qcom-smbx-charger)" 1
# the dtb
ck "c1 dtb: the charger is okay" "$(fdtget "$T/c1.dtb" "$PMI/charger@1000" status)" okay
ck "c1 dtb: charger compatible" "$(fdtget "$T/c1.dtb" "$PMI/charger@1000" compatible)" qcom,pmi8998-charger
BAT=$(fdtget "$T/c1.dtb" /__symbols__ battery 2>/dev/null || echo none)
BATPH=$(fdtget -t u "$T/c1.dtb" "$BAT" phandle 2>/dev/null || echo none)
ck "c1 dtb: the charger's monitored-battery is the battery node's phandle" "$(fdtget -t u "$T/c1.dtb" "$PMI/charger@1000" monitored-battery)" "$BATPH"
ck "c1 dtb: battery is simple-battery" "$(fdtget "$T/c1.dtb" "$BAT" compatible)" simple-battery
ck "c1 dtb: battery 2700 mAh" "$(fdtget -t u "$T/c1.dtb" "$BAT" charge-full-design-microamp-hours)" 2700000
ck "c1 dtb: battery 4.4 V max" "$(fdtget -t u "$T/c1.dtb" "$BAT" voltage-max-design-microvolt)" 4400000
ck "c1 dtb: battery 3.4 V min" "$(fdtget -t u "$T/c1.dtb" "$BAT" voltage-min-design-microvolt)" 3400000
ck "c1 dtb: battery 50 mA termination" "$(fdtget -t u "$T/c1.dtb" "$BAT" charge-term-current-microamp)" 50000
ck "c1 dtb: the rradc node is there and not disabled" "$(fdtget "$T/c1.dtb" "$PMI/adc@4500" compatible); $(fdtget "$T/c1.dtb" "$PMI/adc@4500" status 2>&1 | grep -c disabled)" "qcom,pmi8998-rradc; 0"
ck "c1 dtb: clock_gcc is the real gcc" "$(fdtget "$T/c1.dtb" /__symbols__ clock_gcc)" "$GCC"
ck "c1 dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/c1.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "c1 dtb keeps qcom,msm-id <0x124 0x20001>" "$(fdtget -t x "$T/c1.dtb" / qcom,msm-id)" "124 20001"
ck "c1 dtb keeps the ramoops node at 0xa1100000" "$(fdtget -t x "$T/c1.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "z3's dtb has no charger okay (the test tells them apart)" "$(fdtget "$T/k.dtb" "$PMI/charger@1000" status)" disabled
ck "the dtbo overlay merges into c1.dtb" "$(DTB="$T/c1.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "after the merge the overlay's clock users name gcc" "$(DTB="$T/c1.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-c1.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z3/ramdisk" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "c1 Image.gz is the new build's, byte for byte" "$(cmp -s "$T/uk.gz" "$KO/Image.gz" && echo same || echo differs)" same
ck "c1 dtb is c1.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/c1.dtb" && echo same || echo differs)" same
ck "c1 header equals z3's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "c1 ramdisk bytes equal z3's" "$(cmp -s "$T/u/ramdisk" "$T/z3/ramdisk" && echo same || echo differs)" same
ck "c1 size is z3's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "c1 AVB info equals z3's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "c1 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-c1: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-c1.img"; cp "$T/c1.dtb" "$OUT/msm8998-google-walleye-c1.dtb"
(cd "$OUT" && sha256sum boot-c1.img msm8998-google-walleye-c1.dtb | tee SHA256SUMS)
