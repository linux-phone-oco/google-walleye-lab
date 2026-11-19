#!/bin/sh
# z3 (backlog walleye-dmesg-clean): z2 with ONE change, the dtb: /__symbols__/clock_gcc points at the real gcc (/soc@0/clock-controller@100000) instead of the empty stub.
# The vendor overlay's clock users (serial@c16f000, i2c@c179000, bt_wcn3990, mnh-sm) say `clocks = <&clock_gcc ID ...>`; the stub has no #clock-cells, so the kernel read each
# ID as a phandle ("could not find phandle -1934992561") and the serial waited on the stub as its supplier. After the bootloader's merge they now name gcc. Kernel, ramdisk, header,
# command line and AVB footer are z2's.
#   build-z3.sh Z2_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-z3.img,msm8998-google-walleye-z3.dtb,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
# Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
Z2=${1:?usage: build-z3.sh Z2_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
GCC=/soc@0/clock-controller@100000
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z2" "$DTBO"; do [ -s "$f" ] || { echo "build-z3: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z3: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z2")  $Z2"
INFO=$(unpack_bootimg --boot_img "$Z2" --out "$T/z2")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z2/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z2")
AV=$(avbtool info_image --image "$Z2")
cp "$T/k.dtb" "$T/z3.dtb"
fdtput -t s "$T/z3.dtb" /__symbols__ clock_gcc "$GCC"
cat "$T/k.gz" "$T/z3.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
dts() { dtc -I dtb -O dts "$1" 2>/dev/null; }
dts "$T/k.dtb" > "$T/a.dts"; dts "$T/z3.dtb" > "$T/b.dts"
ck "z2 dtb: clock_gcc is the stub" "$(fdtget "$T/k.dtb" /__symbols__ clock_gcc)" /vendor-overlay-stubs/clock_gcc
ck "z3 dtb: clock_gcc is the real gcc" "$(fdtget "$T/z3.dtb" /__symbols__ clock_gcc)" "$GCC"
ck "z3 dtb: gcc has #clock-cells 1" "$(fdtget -t u "$T/z3.dtb" "$GCC" '#clock-cells' | tr -d ' ')" 1
ck "z3 dtb: gcc has a phandle" "$(fdtget -t u "$T/z3.dtb" "$GCC" phandle | grep -c '^[0-9][0-9]*$')" 1
# dtc moves the label clock_gcc with the symbol; with that label taken off both sides only the symbols line differs
norm() { sed 's/clock_gcc: //' "$1"; }
ck "z3 dtb differs from z2's in the clock_gcc symbol line only" "$(diff "$(norm "$T/a.dts" > "$T/an"; echo "$T/an")" "$(norm "$T/b.dts" > "$T/bn"; echo "$T/bn")" | grep '^[<>]' | tr -s '\t' ' ' | tr '\n' '|')" '< clock_gcc = "/vendor-overlay-stubs/clock_gcc";|> clock_gcc = "/soc@0/clock-controller@100000";|'
mergeline() { DTB="$1" sh "$HERE/tests/host/dtbo-overlay-merges.sh" || true; }
ck "the dtbo overlay merges into z3.dtb" "$(mergeline "$T/z3.dtb" | cut -d' ' -f1)" pass
ck "after the merge the overlay's clock users name gcc" "$(DTB="$T/z3.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" || true)" "$(DTB="$T/z3.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | grep '^pass')"
ck "the same test fails on z2's dtb (it can tell them apart)" "$(DTB="$T/k.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" fail
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z3.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z2/ramdisk" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "z3 Image.gz is z2's, byte for byte" "$(cmp -s "$T/uk.gz" "$T/k.gz" && echo same || echo differs)" same
ck "z3 dtb is z3.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/z3.dtb" && echo same || echo differs)" same
ck "z3 header equals z2's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z3 ramdisk bytes equal z2's" "$(cmp -s "$T/u/ramdisk" "$T/z2/ramdisk" && echo same || echo differs)" same
ck "z3 size is z2's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z3 AVB info equals z2's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z3 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z3: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z3.img"; cp "$T/z3.dtb" "$OUT/msm8998-google-walleye-z3.dtb"
(cd "$OUT" && sha256sum boot-z3.img msm8998-google-walleye-z3.dtb | tee SHA256SUMS)
