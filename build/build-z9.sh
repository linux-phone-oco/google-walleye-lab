#!/bin/sh
# z9 (backlog walleye-kernel-pmic-wdt): z8 with qcom-pon built in (the watchdog is its child; =m in z8 meant no PMIC watchdog device). The kernel is v7.3 with linux-phone-oco/linux topic/pmic-wdt-wake-in-suspend (commit e123d3bff),
# CONFIG_PM8916_WATCHDOG=y (defconfig has =m, the initramfs carries no modules); the dtb (ours, from dt/, adreno_smmu off, charger and battery as c2) has
# &pm8998_wdt { wakeup-source; status = "okay"; }. Ramdisk (the z6 command loop), header, command line and AVB footer are z7's, as c2's are.
#   build-z9.sh Z7_BOOT_IMG KERNEL_OUT_DIR C2_KERNEL_OUT_DIR OUT_DIR
# Not tied to the commit: only the series' compatible string in Image.gz shows the kernel has it. KERNEL_OUT_DIR is build-kernel.sh's OUT_DIR for the kernel with the series (Image.gz, msm8998-google-walleye.dtb built with -@, .config); C2_KERNEL_OUT_DIR is c2's (its .config and dtb are what z9 differs from).
# Writes OUT_DIR/{boot-z9.img,msm8998-google-walleye-z9.dtb,SHA256SUMS-z9}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
# Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
Z7=${1:?usage: build-z9.sh Z7_BOOT_IMG KERNEL_OUT_DIR C2_KERNEL_OUT_DIR OUT_DIR}
KO=${2:?}
CO=${3:?}
OUT=${4:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
GCC=/soc@0/clock-controller@100000
PMI=/soc@0/spmi@800f000/pmic@2
WDT=/soc@0/spmi@800f000/pmic@0/pon@800/watchdog
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z7" "$DTBO" "$KO/Image.gz" "$KO/.config" "$KO/msm8998-google-walleye.dtb" "$CO/Image.gz" "$CO/.config" "$CO/msm8998-google-walleye.dtb"; do [ -s "$f" ] || { echo "build-z9: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z9: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z7")  $Z7"
INFO=$(unpack_bootimg --boot_img "$Z7" --out "$T/z7")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z7/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z7")
AV=$(avbtool info_image --image "$Z7")
cp "$KO/msm8998-google-walleye.dtb" "$T/z9.dtb"
fdtput -t s "$T/z9.dtb" /__symbols__ clock_gcc "$GCC"
cp "$CO/msm8998-google-walleye.dtb" "$T/c2.dtb"
fdtput -t s "$T/c2.dtb" /__symbols__ clock_gcc "$GCC"
cat "$KO/Image.gz" "$T/z9.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
# the kernel
ck ".config has PM8916_WATCHDOG=y" "$(grep -c '^CONFIG_PM8916_WATCHDOG=y$' "$KO/.config")" 1
ck ".config differs from c2's kernel config in the PM8916_WATCHDOG and POWER_RESET_QCOM_PON lines only" "$(diff "$CO/.config" "$KO/.config" | grep '^[<>]' | grep -vc -e 'CONFIG_PM8916_WATCHDOG=' -e 'CONFIG_POWER_RESET_QCOM_PON=')" 0
ck "c2's .config has the charger and rradc, z9's keeps them" "$(grep -c -e "^CONFIG_CHARGER_QCOM_SMB2=y$" -e "^CONFIG_QCOM_SPMI_RRADC=y$" "$CO/.config" "$KO/.config" | tr "\n" " ")" "$CO/.config:2 $KO/.config:2 "
ck "the new Image.gz is not c2's (the kernel change)" "$(cmp -s "$KO/Image.gz" "$CO/Image.gz" && echo same || echo differs)" differs
ck "the new Image.gz is not z7's" "$(cmp -s "$KO/Image.gz" "$T/k.gz" && echo same || echo differs)" differs
ck "Image.gz holds the pm8916 watchdog driver's pm8998 compatible" "$(zcat "$KO/Image.gz" | grep -aq 'qcom,pm8998-wdt' && echo yes || echo no)" yes
ck "Image.gz still holds the charger driver" "$(zcat "$KO/Image.gz" | grep -aq qcom-smbx-charger && echo yes || echo no)" yes
# the dtb
ck "z9 dtb: the pmic watchdog is okay" "$(fdtget "$T/z9.dtb" "$WDT" status)" okay
ck "z9 dtb: the pmic watchdog compatible" "$(fdtget "$T/z9.dtb" "$WDT" compatible)" qcom,pm8998-wdt
ck "z9 dtb: the pmic watchdog has wakeup-source" "$(fdtget -p "$T/z9.dtb" "$WDT" | grep -c '^wakeup-source$')" 1
ck "the watchdog's parent exists in c2's dtb" "$(fdtget -l "$T/c2.dtb" "$(dirname "$WDT")" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "c2's dtb (v7.3 without the series) has no watchdog node (the checks tell them apart)" "$(fdtget -l "$T/c2.dtb" "$(dirname "$WDT")" | grep -c "^watchdog$")" 0
ck "z9 dtb: the charger is okay as in c2" "$(fdtget "$T/z9.dtb" "$PMI/charger@1000" status)" okay
ck "z9 dtb: clock_gcc is the real gcc" "$(fdtget "$T/z9.dtb" /__symbols__ clock_gcc)" "$GCC"
ck "z9 dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/z9.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "z9 dtb keeps qcom,msm-id <0x124 0x20001>" "$(fdtget -t x "$T/z9.dtb" / qcom,msm-id)" "124 20001"
ck "z9 dtb keeps the ramoops node at 0xa1100000" "$(fdtget -t x "$T/z9.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "z9 dtb: the GPU smmu is disabled as in z7 (the test passes)" "$(DTB="$T/z9.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d" " -f1)" pass
norm() { dtc -I dtb -O dts "$1" 2>/dev/null | sed 's/<[^>]*>/<>/g' | grep -v phandle | sort; }
norm "$T/c2.dtb" > "$T/a.dts"; norm "$T/z9.dtb" > "$T/b.dts"
ck "z9 dtb differs from c2's only by the watchdog node (the series' pm8998.dtsi node, okay, wakeup-source), in names and structure" "$(diff "$T/a.dts" "$T/b.dts" | grep '^[<>]' | tr -s '\t' ' ' | sed 's/^\([<>]\) */\1 /' | tr '\n' '|')" '> |> };|> compatible = "qcom,pm8998-wdt";|> interrupts = <>;|> pm8998_wdt = "/soc@0/spmi@800f000/pmic@0/pon@800/watchdog";|> pm8998_wdt: watchdog {|> status = "okay";|> wakeup-source;|'
ck "after the merge the overlay's clock users name gcc" "$(DTB="$T/z9.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z9.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z7/ramdisk" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "z9 Image.gz is the new build's, byte for byte" "$(cmp -s "$T/uk.gz" "$KO/Image.gz" && echo same || echo differs)" same
ck "z9 dtb is z9.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/z9.dtb" && echo same || echo differs)" same
ck "z9 header equals z7's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z9 ramdisk bytes equal z7's" "$(cmp -s "$T/u/ramdisk" "$T/z7/ramdisk" && echo same || echo differs)" same
ck "z9 size is z7's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z9 AVB info equals z7's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z9 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z9: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z9.img"; cp "$T/z9.dtb" "$OUT/msm8998-google-walleye-z9.dtb"
(cd "$OUT" && sha256sum boot-z9.img msm8998-google-walleye-z9.dtb | tee SHA256SUMS-z9)
