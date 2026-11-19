#!/bin/sh
# z15 (backlog walleye-touch-rmi4): z14 with the kernel and the dtb changed: the kernel has the Synaptics RMI4 touch driver built in (rmi_i2c, function 12; build/walleye.config), the dtb has the touch controller on blsp1_i2c5 (0x20, irq gpio125, reset gpio89, rails pm8998 l6 and l28).
# Ramdisk (z14's init: ACM console, ECM + telnet on 172.16.43.1), header, command line and AVB footer are z14's, byte for byte. Built only - no phone run; the keeper tries it once, a person touching the panel (tests/host/touch-read.sh, then touch-check.sh).
#   build-z15.sh Z14_BOOT_IMG KERNEL_OUT_DIR OUT_DIR
# KERNEL_OUT_DIR is build-kernel.sh's OUT_DIR for the tree of linux-phone-oco/linux topic/pmic-wdt-wake-in-suspend (Image.gz, msm8998-google-walleye.dtb built with -@, .config). Writes OUT_DIR/{boot-z15.img,msm8998-google-walleye-z15.dtb,SHA256SUMS-z15};
# checks (ok/FAIL, exit 1 on a FAIL, nothing written then). Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
Z14=${1:?usage: build-z15.sh Z14_BOOT_IMG KERNEL_OUT_DIR OUT_DIR}
KO=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
GCC=/soc@0/clock-controller@100000
TS=/soc@0/i2c@c179000/touchscreen@20
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z14" "$DTBO" "$KO/Image.gz" "$KO/.config" "$KO/msm8998-google-walleye.dtb"; do [ -s "$f" ] || { echo "build-z15: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z15: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z14")  $Z14"
INFO=$(unpack_bootimg --boot_img "$Z14" --out "$T/z14")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z14/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z14")
AV=$(avbtool info_image --image "$Z14")
cp "$KO/msm8998-google-walleye.dtb" "$T/z15.dtb"
fdtput -t s "$T/z15.dtb" /__symbols__ clock_gcc "$GCC"
cat "$KO/Image.gz" "$T/z15.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
# the kernel
for s in RMI4_CORE RMI4_I2C RMI4_F12 INPUT_EVDEV I2C_QUP PINCTRL_MSM8998 PM8916_WATCHDOG CHARGER_QCOM_SMB2 USB_CONFIGFS_ECM; do
  ck ".config has $s=y" "$(grep -c "^CONFIG_$s=y$" "$KO/.config")" 1
done
ck "the new Image.gz is not z14's kernel" "$(cmp -s "$KO/Image.gz" "$T/k.gz" && echo same || echo differs)" differs
ck "Image.gz holds the RMI4 I2C driver (syna,rmi4-i2c at least once)" "$(zcat "$KO/Image.gz" | grep -ac 'syna,rmi4-i2c' | awk '{print ($1>0)?"yes":"no"}')" yes
ck "z14's kernel has no RMI4 driver (the change is this one)" "$(zcat "$T/k.gz" | grep -ac 'syna,rmi4-i2c')" 0
# the dtb
ck "z15 dtb: the touch controller is a syna,rmi4-i2c at 0x20" "$(fdtget "$T/z15.dtb" "$TS" compatible)/$(fdtget -t x "$T/z15.dtb" "$TS" reg)" "syna,rmi4-i2c/20"
ck "z15 dtb: blsp1_i2c5 is okay" "$(fdtget "$T/z15.dtb" "$(dirname "$TS")" status)" okay
ck "z15 dtb: interrupt gpio125, falling edge (type 2)" "$(fdtget -t u "$T/z15.dtb" "$TS" interrupts)" "125 2"
ck "z15 dtb: the reset pin gpio89 and the interrupt pin gpio125 have their pin states" "$(dtc -I dtb -O dts "$T/z15.dtb" 2>/dev/null | grep -c -e 'pins = "gpio125"' -e 'pins = "gpio89"')" 2
rail() { # PROPERTY: the regulator-min-microvolt of the node the touch node's PROPERTY points at
  ph=$(fdtget -t x "$T/z15.dtb" "$TS" "$1")
  dtc -I dtb -O dts "$T/z15.dtb" 2>/dev/null | awk -v ph="$ph" '/regulator-min-microvolt/{v=$3} /phandle = /{p=$3; gsub(/[<>;]/,"",p); if (p=="0x" ph) {gsub(/[<>;]/,"",v); print v; exit}}'
}
ck "z15 dtb: vdd-supply is a rail of 3.008 V (pm8998 l28, 0x2de600)" "$(rail vdd-supply)" 0x2de600
ck "z15 dtb: vio-supply is a rail of 1.808 V (pm8998 l6, 0x1b9680)" "$(rail vio-supply)" 0x1b9680
RPM=$(fdtget "$T/z15.dtb" /__symbols__ rpm_requests)/regulators-0
ck "z15 dtb: l6 hangs from s5 and l28 from bob (the rpm node's supplies name them)" "$(fdtget -t x "$T/z15.dtb" "$RPM" vdd_l6-supply)/$(fdtget -t x "$T/z15.dtb" "$RPM" vdd_l16_l28-supply)" "$(fdtget -t x "$T/z15.dtb" "$(fdtget "$T/z15.dtb" /__symbols__ vreg_s5a_2p04)" phandle)/$(fdtget -t x "$T/z15.dtb" "$(fdtget "$T/z15.dtb" /__symbols__ vreg_bob)" phandle)"
ck "z15 dtb: function 12 is there" "$(fdtget -t u "$T/z15.dtb" "$TS/rmi4-f12@12" reg)" 18
ck "z15 dtb: clock_gcc is the real gcc" "$(fdtget "$T/z15.dtb" /__symbols__ clock_gcc)" "$GCC"
ck "z15 dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/z15.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "z15 dtb keeps qcom,msm-id <0x124 0x20001>" "$(fdtget -t x "$T/z15.dtb" / qcom,msm-id)" "124 20001"
ck "z15 dtb keeps the ramoops node at 0xa1100000" "$(fdtget -t x "$T/z15.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "z15 dtb: the GPU smmu is disabled (the test passes)" "$(DTB="$T/z15.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d" " -f1)" pass
ck "after the overlay merge the clock users name gcc" "$(DTB="$T/z15.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z15.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z14/ramdisk" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "z15 Image.gz is the new build's, byte for byte" "$(cmp -s "$T/uk.gz" "$KO/Image.gz" && echo same || echo differs)" same
ck "z15 dtb is z15.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/z15.dtb" && echo same || echo differs)" same
ck "z15 header equals z14's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z15 ramdisk bytes equal z14's" "$(cmp -s "$T/u/ramdisk" "$T/z14/ramdisk" && echo same || echo differs)" same
ck "z15 size is z14's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z15 AVB info equals z14's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z15 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z15: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z15.img"; cp "$T/z15.dtb" "$OUT/msm8998-google-walleye-z15.dtb"
(cd "$OUT" && sha256sum boot-z15.img msm8998-google-walleye-z15.dtb | tee SHA256SUMS-z15)
