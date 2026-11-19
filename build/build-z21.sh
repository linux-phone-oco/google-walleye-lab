#!/bin/sh
# NOT TO BOOT AS IS: z20 has UFS on (decision 2026-11-19-walleye-ufs-off-by-a-gate); this repack keeps it. The z21 to try is built by walleye-ufs-off-gate (z20 with UFS off) and carries the same stub change from the dtsi.
# z21 (backlog walleye-touch-gpio-shared-proxy): z20 with ONE change in the dtb: the container of the overlay stubs, /vendor-overlay-stubs, is status = "disabled".
# Why: z20's dmesg (research walleye-z20-tried-touch-gpio-leaf) says regulator-touch-power waits on a shared GPIO, served by gpiolib_shared.proxy.0, which never registers. The kernel makes a gpio
# "shared" when two available nodes name it by a *-gpio(s) property (drivers/gpio/gpiolib-shared.c, gpio_shared_of_traverse; it skips a node that is not available and does not enter its children). The
# second claimant is not ours: the phone's own dtbo overlay (entry 3) adds synaptics_dsx@20 with synaptics,power-gpio <pmi8998 gpio 2> under the label i2c_5, which our dtb holds as an empty stub. The overlay
# cannot be edited (the bootloader merges the phone's own), but where the stub sits can: disabled, the container and everything the overlay puts in it is not scanned. tests/host/touch-gpio-single-claimant.sh
# merges the overlay and counts the available claimants (z20: 2, z21: 1). Nothing else changes: kernel (the diagnostic one), ramdisk, header, command line (dyndbg) and AVB footer are z20's, byte for byte.
# Built only - no phone run; the keeper tries it once from slot b (walleye-try-z21).
#   build-z21.sh Z20_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-z21.img,msm8998-google-walleye-z21.dtb,SHA256SUMS-z21}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then). Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
Z20=${1:?usage: build-z21.sh Z20_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z20" "$DTBO"; do [ -s "$f" ] || { echo "build-z21: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z21: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z20")  $Z20"
INFO=$(unpack_bootimg --boot_img "$Z20" --out "$T/z20")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z20/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z20")
AV=$(avbtool info_image --image "$Z20")
C=/vendor-overlay-stubs
cp "$T/k.dtb" "$T/z21.dtb"
ck "z20 dtb: the stub container has no status" "$(fdtget "$T/k.dtb" "$C" status 2>/dev/null || echo none)" none
ck "z20 dtb: the claimant test fails (two available claimants)" "$(DTB="$T/k.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | sed 's/ walleye.*kernel \([0-9]*\) available.*/ \1/')" "fail 2"
fdtput -t s "$T/z21.dtb" "$C" status disabled
cat "$T/k.gz" "$T/z21.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
# the dtb
ck "z21 dtb: the stub container is disabled" "$(fdtget "$T/z21.dtb" "$C" status)" disabled
dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null > "$T/k.dts"
dtc -I dtb -O dts "$T/z21.dtb" 2>/dev/null > "$T/z21.dts"
ck "z21 dtb adds one line to z20's decompiled dts (the status), removes none" "$(diff "$T/k.dts" "$T/z21.dts" | grep -c '^>') $(diff "$T/k.dts" "$T/z21.dts" | grep -c '^<')" "1 0"
ck "z21 dtb keeps the touch power regulator, gpios <pmi8998 gpio 2>" "$(fdtget -t u "$T/z21.dtb" /regulator-touch-power gpios | cut -d' ' -f2)" 2
ck "z21 dtb keeps the stub i2c_5 the overlay targets" "$(fdtget "$T/z21.dtb" /__symbols__ i2c_5)" /vendor-overlay-stubs/i2c_5
ck "z21 dtb keeps /__symbols__/clock_gcc at the real gcc" "$(fdtget "$T/z21.dtb" /__symbols__ clock_gcc)" /soc@0/clock-controller@100000
ck "z21 dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/z21.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "z21 dtb keeps the ramoops node at 0xa1100000" "$(fdtget -t x "$T/z21.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "z21 dtb: the GPU smmu is disabled (the test passes)" "$(DTB="$T/z21.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d" " -f1)" pass
# after the overlay merge
ck "z21 dtb: exactly one available node names the touch power gpio (tests/host/touch-gpio-single-claimant.sh)" "$(DTB="$T/z21.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
ck "z21 dtb: the overlay overwrites nothing of ours (tests/host/overlay-collisions.sh)" "$(DTB="$T/z21.dtb" QUIET=1 sh "$HERE/tests/host/overlay-collisions.sh" | tail -n1 | cut -d' ' -f1)" pass
ck "the overlay still merges into z21's dtb" "$(DTB="$T/z21.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the clock users name gcc" "$(DTB="$T/z21.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the qup controller is ours, with the touch controller" "$(DTB="$T/z21.dtb" sh "$HERE/tests/host/i2c-qup-after-merge.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the touch controller's vdd-supply is still the touch regulator" "$(DTB="$T/z21.dtb" sh "$HERE/tests/host/touch-power-after-merge.sh" | cut -d' ' -f1)" pass
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z21.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z20/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "z21 Image.gz is z20's, byte for byte" "$(cmp -s "$T/uk.gz" "$T/k.gz" && echo same || echo differs)" same
ck "z21 dtb is z21.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/z21.dtb" && echo same || echo differs)" same
ck "z21 header (command line included) equals z20's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z21 ramdisk bytes equal z20's" "$(cmp -s "$T/u/ramdisk" "$T/z20/ramdisk" && echo same || echo differs)" same
ck "z21 size is z20's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z21 AVB info equals z20's but the digest and the sizes of the kernel it hashes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
KS=$(stat -c %s "$T/blob"); KO_=$(printf %d "$(field "kernel load address")"); RO=$(printf %d "$(field "ramdisk load address")")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
ck "z21 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the host test passes on z21 (tests/host/gpio-diag-boot.sh)" "$(IMG="$IMG" sh "$HERE/tests/host/gpio-diag-boot.sh" | cut -d' ' -f1)" pass
[ $total -eq 0 ] || { echo "build-z21: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z21.img"; cp "$T/z21.dtb" "$OUT/msm8998-google-walleye-z21.dtb"
(cd "$OUT" && sha256sum boot-z21.img msm8998-google-walleye-z21.dtb | tee SHA256SUMS-z21)
