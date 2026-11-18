#!/bin/sh
# z16 (backlog walleye-build-z16-i2c-compatible): z15 with ONE change in the dtb: the node of the QUP i2c controller at 0xc179000 is named qup@c179000, not i2c@c179000.
# Why: z15's dtb already has compatible qcom,i2c-qup-v2.2.1 there, but the bootloader merges the vendor overlay's i2c@c179000 BY NAME into it and overwrites compatible
# (qcom,i2c-msm-v2), clocks, interrupts, dmas and pinctrl, so no mainline driver bound (research walleye-z15-part1-tried). A node of another name stays ours; the overlay's own
# node stands beside it, driverless (the host merge shows the properties; two nodes at one region and their order are for the phone try to show). The rename is three same-length substitutions (the node name, the two /__symbols__ paths blsp1_i2c5 and rmi4_f12), so the dtb keeps its size and layout.
# Kernel, ramdisk (z14's init), header, command line and AVB footer are z15's, byte for byte. Built only - no phone run; the keeper tries it once.
#   build-z16.sh Z15_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-z16.img,msm8998-google-walleye-z16.dtb,SHA256SUMS-z16}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then). Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
Z15=${1:?usage: build-z16.sh Z15_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z15" "$DTBO"; do [ -s "$f" ] || { echo "build-z16: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z16: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z15")  $Z15"
INFO=$(unpack_bootimg --boot_img "$Z15" --out "$T/z15")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z15/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z15")
AV=$(avbtool info_image --image "$Z15")
OLD=i2c@c179000; NEW=qup@c179000
ck "z15 dtb names the controller $OLD in three places (node, two symbol paths)" "$(LC_ALL=C grep -aoF "$OLD" "$T/k.dtb" | wc -l)" 3
LC_ALL=C sed "s/$OLD/$NEW/g" "$T/k.dtb" > "$T/z16.dtb"
cat "$T/k.gz" "$T/z16.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
# the dtb
ck "z16 dtb is z15's size" "$(stat -c %s "$T/z16.dtb")" "$(stat -c %s "$T/k.dtb")"
ck "z16 dtb differs from z15's only in the 3 renamed runs ('i2c' to 'qup' three times: 3 letters each, 9 bytes)" "$(cmp -l "$T/k.dtb" "$T/z16.dtb" | wc -l)" 9
ck "z16 dtb no longer has $OLD" "$(LC_ALL=C grep -aoF "$OLD" "$T/z16.dtb" | wc -l)" 0
N=/soc@0/qup@c179000
ck "z16 dtb: the controller is qcom,i2c-qup-v2.2.1 (i2c_qup matches)" "$(fdtget "$T/z16.dtb" "$N" compatible)" qcom,i2c-qup-v2.2.1
ck "z16 dtb: reg 0xc179000" "$(fdtget -t x "$T/z16.dtb" "$N" reg | awk '{print $1}')" c179000
ck "z16 dtb: status okay" "$(fdtget "$T/z16.dtb" "$N" status)" okay
ck "z16 dtb: clock-names core iface" "$(fdtget "$T/z16.dtb" "$N" clock-names)" "core iface"
ck "z16 dtb: the touch controller is a syna,rmi4-i2c at 0x20 on it" "$(fdtget "$T/z16.dtb" "$N/touchscreen@20" compatible)/$(fdtget -t x "$T/z16.dtb" "$N/touchscreen@20" reg)" "syna,rmi4-i2c/20"
ck "z16 dtb: function 12 is there" "$(fdtget -t u "$T/z16.dtb" "$N/touchscreen@20/rmi4-f12@12" reg)" 18
ck "z16 dtb: /__symbols__ blsp1_i2c5 follows the node" "$(fdtget "$T/z16.dtb" /__symbols__ blsp1_i2c5)" "$N"
ck "z16 dtb: /__symbols__ rmi4_f12 follows the node" "$(fdtget "$T/z16.dtb" /__symbols__ rmi4_f12)" "$N/touchscreen@20/rmi4-f12@12"
ck "z16 dtb: clock_gcc is the real gcc" "$(fdtget "$T/z16.dtb" /__symbols__ clock_gcc)" /soc@0/clock-controller@100000
ck "z16 dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/z16.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "z16 dtb keeps qcom,msm-id <0x124 0x20001>" "$(fdtget -t x "$T/z16.dtb" / qcom,msm-id)" "124 20001"
ck "z16 dtb keeps the ramoops node at 0xa1100000" "$(fdtget -t x "$T/z16.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "z16 dtb: the GPU smmu is disabled (the test passes)" "$(DTB="$T/z16.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d" " -f1)" pass
ck "after the overlay merge the clock users name gcc" "$(DTB="$T/z16.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the qup controller is ours, with the touch controller (the proof test passes)" "$(DTB="$T/z16.dtb" sh "$HERE/tests/host/i2c-qup-after-merge.sh" | cut -d' ' -f1)" pass
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z16.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z15/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "z16 Image.gz is z15's, byte for byte" "$(cmp -s "$T/uk.gz" "$T/k.gz" && echo same || echo differs)" same
ck "z16 dtb is z16.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/z16.dtb" && echo same || echo differs)" same
ck "z16 header equals z15's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z16 ramdisk bytes equal z15's" "$(cmp -s "$T/u/ramdisk" "$T/z15/ramdisk" && echo same || echo differs)" same
ck "z16 size is z15's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z16 AVB info equals z15's but the digest" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z16 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z16: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z16.img"; cp "$T/z16.dtb" "$OUT/msm8998-google-walleye-z16.dtb"
(cd "$OUT" && sha256sum boot-z16.img msm8998-google-walleye-z16.dtb | tee SHA256SUMS-z16)
