#!/bin/sh
# c4 (backlog walleye-charger-supplier, second run): c2 with ONLY the dtb changed: our dts now models the charger's boost supply,
# the TPS61099 on pm8998 gpio 5, as a fixed regulator, and disables the vendor overlay's driverless /soc@0/tps61099 so the
# charger no longer waits on it. Kernel Image.gz, ramdisk, header (command line included: NO fw_devlink=permissive), size and AVB footer are c2's.
#   build-c4.sh C2_BOOT_IMG KERNEL_OUT_DIR OUT_DIR
# KERNEL_OUT_DIR is build-kernel.sh's OUT_DIR for the current dts (msm8998-google-walleye.dtb built with -@).
# Writes OUT_DIR/{boot-c4.img,msm8998-google-walleye-c4.dtb,SHA256SUMS-c4}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
# Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
C2=${1:?usage: build-c4.sh C2_BOOT_IMG KERNEL_OUT_DIR OUT_DIR}
KO=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
GCC=/soc@0/clock-controller@100000
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$C2" "$DTBO" "$KO/msm8998-google-walleye.dtb"; do [ -s "$f" ] || { echo "build-c4: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-c4: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(sha256sum "$C2" | cut -c1-16)  $C2"
INFO=$(unpack_bootimg --boot_img "$C2" --out "$T/c2")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/c2/kernel" "$T/k"
PSIZE=$(stat -c %s "$C2")
AV=$(avbtool info_image --image "$C2")
cp "$KO/msm8998-google-walleye.dtb" "$T/c4.dtb"
fdtput -t s "$T/c4.dtb" /__symbols__ clock_gcc "$GCC"
cat "$T/k.gz" "$T/c4.dtb" > "$T/blob"
# c2's command line has no fw_devlink argument, and neither has c4's
ck "c2's command line has no fw_devlink" "$(field 'command line args' | grep -c fw_devlink)" 0
# the dtb: the new nodes
ck "c4 dtb: the regulator is regulator-fixed" "$(fdtget "$T/c4.dtb" /regulator-tps61099-vbus compatible)" regulator-fixed
ck "c4 dtb: the regulator is named tps61099-vbus" "$(fdtget "$T/c4.dtb" /regulator-tps61099-vbus regulator-name)" tps61099-vbus
ck "c4 dtb: the regulator gpios is pm8998 gpio 5, active high" "$(fdtget -t u "$T/c4.dtb" /regulator-tps61099-vbus gpios | awk '{print $2, $3}')" "5 0"
GP=$(fdtget -t u "$T/c4.dtb" /regulator-tps61099-vbus gpios | awk '{print $1}')
ck "c4 dtb: that gpio phandle is the pm8998 gpio controller" "$(fdtget -t u "$T/c4.dtb" "$(fdtget "$T/c4.dtb" /__symbols__ pm8998_gpios)" phandle)" "$GP"
ck "c4 dtb: the regulator enables high" "$(fdtget -t s "$T/c4.dtb" /regulator-tps61099-vbus enable-active-high 2>&1 | wc -c)" 1
ck "c4 dtb: /soc@0/tps61099 is disabled" "$(fdtget "$T/c4.dtb" /soc@0/tps61099 status)" disabled
ck "c4 dtb keeps the charger okay" "$(fdtget "$T/c4.dtb" /soc@0/spmi@800f000/pmic@2/charger@1000 status)" okay
ck "c4 dtb: clock_gcc is the real gcc" "$(fdtget "$T/c4.dtb" /__symbols__ clock_gcc)" "$GCC"
ck "c4 dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/c4.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "c4 dtb keeps the ramoops node at 0xa1100000" "$(fdtget -t x "$T/c4.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "c4 dtb: the GPU smmu is disabled as in c2 (the test passes)" "$(DTB="$T/c4.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d" " -f1)" pass
ck "after the merge the overlay's clock users name gcc" "$(DTB="$T/c4.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
# the merge with the overlay the bootloader picks: the regulator stays, the overlay's own node stays disabled with its child
e=$(xxd -s 128 -l 8 -p "$DTBO")
size=$((0x$(echo "$e" | cut -c1-8))); off=$((0x$(echo "$e" | cut -c9-16)))
tail -c +$((off+1)) "$DTBO" | head -c "$size" > "$T/ov"
fdtoverlay -i "$T/c4.dtb" -o "$T/m" "$T/ov" || { echo "FAIL c4 dtb does not merge with the overlay"; total=$((total+1)); }
if [ -s "$T/m" ]; then
  ck "merged: /soc@0/tps61099 is still disabled" "$(fdtget "$T/m" /soc@0/tps61099 status)" disabled
  ck "merged: the overlay's tps61099 child is there (the charger's external-vbus-supply points at it)" "$(fdtget "$T/m" '/soc@0/tps61099/ti,tps61099' regulator-name)" tps61099-vbus
  ck "merged: the regulator is still regulator-fixed" "$(fdtget "$T/m" /regulator-tps61099-vbus compatible)" regulator-fixed
  ck "merged: the charger's external-vbus-supply is the overlay's child" "$(fdtget -t u "$T/m" /soc@0/spmi@800f000/pmic@2/charger@1000 external-vbus-supply)" "$(fdtget -t u "$T/m" '/soc@0/tps61099/ti,tps61099' phandle)"
fi
# the c4 dtb against c2's: names and structure, cell values set aside
norm() { dtc -I dtb -O dts "$1" 2>/dev/null | sed 's/<[^>]*>/<>/g' | grep -v phandle | sort; }
norm "$T/k.dtb" > "$T/a.dts"; norm "$T/c4.dtb" > "$T/b.dts"
ck "c4 dtb differs from c2's only by the regulator node (with its label) and the new /soc@0/tps61099 node with status disabled, in names and structure (cell values set aside)" "$(diff "$T/a.dts" "$T/b.dts" | grep '^[<>]' | tr -s '\t' ' ' | sed 's/^\([<>]\) */\1 /' | tr '\n' '|')" '> |> |> };|> };|> compatible = "regulator-fixed";|> enable-active-high;|> gpios = <>;|> regulator-name = "tps61099-vbus";|> status = "disabled";|> tps61099 {|> tps61099_vbus_regulator = "/regulator-tps61099-vbus";|> tps61099_vbus_regulator: regulator-tps61099-vbus {|'
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-c4.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/c2/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
ck "c4 Image.gz equals c2's, byte for byte" "$(cmp -s "$T/uk.gz" "$T/k.gz" && echo same || echo differs)" same
ck "c4 dtb is c4.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/c4.dtb" && echo same || echo differs)" same
ck "c4 dtb is not c2's" "$(cmp -s "$T/uk.dtb" "$T/k.dtb" && echo same || echo differs)" differs
ck "c4 header (command line included) equals c2's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "c4 ramdisk equals c2's, byte for byte" "$(cmp -s "$T/u/ramdisk" "$T/c2/ramdisk" && echo same || echo differs)" same
ck "c4 size is c2's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "c4 AVB info equals c2's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "c4 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-c4: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-c4.img"; cp "$T/c4.dtb" "$OUT/msm8998-google-walleye-c4.dtb"
(cd "$OUT" && sha256sum boot-c4.img msm8998-google-walleye-c4.dtb | tee SHA256SUMS-c4)
