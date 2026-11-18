#!/bin/sh
# z17 (backlog walleye-build-z17-touch-power): z16 with ONE change in the dtb: the touch controller's power switch, the pmi8998 gpio 2, is modelled and held on.
# Why: z16's rmi4_i2c bound on the qup bus but its first read failed -5 (research walleye-z16-part1-tried): the chip is silent. The phone's own dtbo overlay (entry 3, MP1;
# fragments 143 and 145) gives the controller synaptics,power-gpio = pmi8998 gpio 2, power-on-state 1 (active high), reset gpio89 active low, reset-active 1 ms, reset-delay 20 ms;
# rails l6 and l28 are the base's. z16 powered only the rails (l28, l6): the pmi8998 gpio 2 was never driven. The mainline rmi_i2c driver takes vdd and vio supplies and has no
# reset or power gpio of its own, so the gpio becomes a regulator-fixed (enable gpio pmi8998 2, active high, from l28, 20 ms start-up) and the touch node's vdd-supply points to it.
# The reset pin stays pulled up (released). The overlay's switch-gpio (tlmm gpio 75) is not known yet and is left alone.
# Everything else - kernel, ramdisk, header, command line, AVB footer, and the dtb but for this one node, its symbol and one property - is z16's.
# Built only - no phone run; the keeper tries it once (walleye-try-z17-part1).
#   build-z17.sh Z16_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-z17.img,msm8998-google-walleye-z17.dtb,SHA256SUMS-z17}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then). Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
Z16=${1:?usage: build-z17.sh Z16_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z16" "$DTBO"; do [ -s "$f" ] || { echo "build-z17: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z17: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z16")  $Z16"
INFO=$(unpack_bootimg --boot_img "$Z16" --out "$T/z16")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z16/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z16")
AV=$(avbtool info_image --image "$Z16")
N=/soc@0/qup@c179000
TS=$N/touchscreen@20
R=/regulator-touch-power
cp "$T/k.dtb" "$T/z17.dtb"
PMI=$(fdtget "$T/k.dtb" /__symbols__ pmi8998_gpios)
L28=$(fdtget "$T/k.dtb" /__symbols__ vreg_l28_3p0)
PMIPH=$(fdtget -t u "$T/k.dtb" "$PMI" phandle)
L28PH=$(fdtget -t u "$T/k.dtb" "$L28" phandle)
NEWPH=$(( $(dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null | sed -n 's/.*phandle = <\(0x[0-9a-f]*\)>.*/\1/p' | while read -r p; do echo $((p)); done | sort -n | tail -n1) + 1 ))
ck "z16 dtb has no $R yet" "$(fdtget -l "$T/k.dtb" / | grep -c '^regulator-touch-power$')" 0
ck "z16 dtb: pmi8998 gpio controller is qcom,pmi8998-gpio" "$(fdtget "$T/k.dtb" "$PMI" compatible | cut -d' ' -f1)" qcom,pmi8998-gpio
V0=$(fdtget -t x "$T/k.dtb" "$TS" vdd-supply)
fdtput -c "$T/z17.dtb" "$R"
fdtput -t s "$T/z17.dtb" "$R" compatible regulator-fixed
fdtput -t s "$T/z17.dtb" "$R" regulator-name touch-power
fdtput -t u "$T/z17.dtb" "$R" gpios "$PMIPH" 2 0
fdtput -t x "$T/z17.dtb" "$R" enable-active-high
fdtput -t u "$T/z17.dtb" "$R" vin-supply "$L28PH"
fdtput -t u "$T/z17.dtb" "$R" startup-delay-us 20000
fdtput -t u "$T/z17.dtb" "$R" phandle "$NEWPH"
fdtput -t s "$T/z17.dtb" /__symbols__ touch_power_regulator "$R"
fdtput -t u "$T/z17.dtb" "$TS" vdd-supply "$NEWPH"
cat "$T/k.gz" "$T/z17.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
# the dtb: the touch node's power chain
ck "z17 dtb: $R is regulator-fixed" "$(fdtget "$T/z17.dtb" "$R" compatible)" regulator-fixed
ck "z17 dtb: $R enables on pmi8998 gpio 2, active high (the phone's own overlay: power-gpio 2, power-on-state 1)" "$(fdtget -t u "$T/z17.dtb" "$R" gpios)/$(fdtget -t u "$T/z17.dtb" "$R" gpios | wc -w)" "$PMIPH 2 0/3"
ck "z17 dtb: $R has enable-active-high" "$(fdtget -p "$T/z17.dtb" "$R" | grep -c '^enable-active-high$')" 1
ck "z17 dtb: $R takes l28 (3.0 V) as its input" "$(fdtget -t u "$T/z17.dtb" "$R" vin-supply)" "$L28PH"
ck "z17 dtb: $R start-up delay 20 ms" "$(fdtget -t u "$T/z17.dtb" "$R" startup-delay-us)" 20000
ck "z17 dtb: the new phandle ($NEWPH) is defined once" "$(dtc -I dtb -O dts "$T/z17.dtb" 2>/dev/null | grep -c "phandle = <$(printf '0x%x' "$NEWPH")>")" 1
ck "z17 dtb: the touch controller's vdd-supply is $R" "$(fdtget -t u "$T/z17.dtb" "$TS" vdd-supply)" "$NEWPH"
ck "z17 dtb: the touch controller's vio-supply is still l6" "$(fdtget -t x "$T/z17.dtb" "$TS" vio-supply)" "$(fdtget -t x "$T/k.dtb" "$TS" vio-supply)"
ck "z17 dtb: /__symbols__ touch_power_regulator names it" "$(fdtget "$T/z17.dtb" /__symbols__ touch_power_regulator)" "$R"
ck "z16's vdd-supply was l28 itself" "$(printf %d "0x$V0")" "$L28PH"
# nothing else of the dtb changed: decompiled, z16 and z17 differ only in the new node, its symbol and the vdd-supply line
dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null > "$T/k.dts"
dtc -I dtb -O dts "$T/z17.dtb" 2>/dev/null > "$T/z17.dts"
ck "z17 dtb adds 12 lines to z16's decompiled dts (the node 10, the symbol, the changed vdd-supply) and removes one" "$(diff "$T/k.dts" "$T/z17.dts" | grep -c "^>")" 12
ck "z17 dtb: the one removed line is z16's vdd-supply" "$(diff "$T/k.dts" "$T/z17.dts" | sed -n 's/^<[[:space:]]*//p')" "vdd-supply = <$(printf '0x%02x' "$((0x$V0))")>;"
ck "z17 dtb keeps the qup node, qcom,i2c-qup-v2.2.1" "$(fdtget "$T/z17.dtb" "$N" compatible)" qcom,i2c-qup-v2.2.1
ck "z17 dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/z17.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "z17 dtb keeps qcom,msm-id <0x124 0x20001>" "$(fdtget -t x "$T/z17.dtb" / qcom,msm-id)" "124 20001"
ck "z17 dtb keeps the ramoops node at 0xa1100000" "$(fdtget -t x "$T/z17.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "z17 dtb: clock_gcc is the real gcc" "$(fdtget "$T/z17.dtb" /__symbols__ clock_gcc)" /soc@0/clock-controller@100000
ck "z17 dtb: the GPU smmu is disabled (the test passes)" "$(DTB="$T/z17.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d" " -f1)" pass
ck "after the overlay merge the clock users name gcc" "$(DTB="$T/z17.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the qup controller is ours, with the touch controller" "$(DTB="$T/z17.dtb" sh "$HERE/tests/host/i2c-qup-after-merge.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the touch controller's vdd-supply is still the new regulator" "$(DTB="$T/z17.dtb" sh "$HERE/tests/host/touch-power-after-merge.sh" | cut -d' ' -f1)" pass
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z17.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z16/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "z17 Image.gz is z16's, byte for byte" "$(cmp -s "$T/uk.gz" "$T/k.gz" && echo same || echo differs)" same
ck "z17 dtb is z17.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/z17.dtb" && echo same || echo differs)" same
ck "z17 header equals z16's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z17 ramdisk bytes equal z16's" "$(cmp -s "$T/u/ramdisk" "$T/z16/ramdisk" && echo same || echo differs)" same
ck "z17 size is z16's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z17 AVB info equals z16's but the digest" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z17 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z17: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z17.img"; cp "$T/z17.dtb" "$OUT/msm8998-google-walleye-z17.dtb"
(cd "$OUT" && sha256sum boot-z17.img msm8998-google-walleye-z17.dtb | tee SHA256SUMS-z17)
