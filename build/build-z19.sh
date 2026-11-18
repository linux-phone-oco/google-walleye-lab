#!/bin/sh
# z19 (backlog walleye-overlay-collisions): z17 with ONE change in the dtb: nothing of ours is a node or a label the stock dtbo overlay (entry 3) writes by name.
# Why: the bootloader merges the overlay into the dtb BY NAME; z16 (i2c@c179000) and z17/z18 (pmi8998@3) were collisions found one at a time by phone tries. tests/host/overlay-collisions.sh now lists every
# one on a dtb: on z17 it finds the QUP i2c at c179000 already fixed (z16) and ONE more, mainline's PCIe host /soc@0/pcie@1c00000 - the overlay's pcie0 fragment is aimed at our label pcie0 and
# rewrites its reg and ranges (the node is disabled, nothing enables it, so z17 would have lived; it is the class, not a fault seen). The fix is the dts's (dt/msm8998-google-walleye.dts): the node
# is deleted and the label pcie0 is a stub, like every other label the overlay names that is not ours. Here, on z17's dtb: delete /soc@0/pcie@1c00000, add /vendor-overlay-stubs/pcie0 (empty, with the
# deleted node's phandle) and point /__symbols__/pcie0 at it. Nothing else changes: kernel, ramdisk (z14's init), header, command line and AVB footer are z17's, byte for byte.
# Built only - no phone run; the keeper tries it once (walleye-try-z19).
#   build-z19.sh Z17_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-z19.img,msm8998-google-walleye-z19.dtb,SHA256SUMS-z19}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then). Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
Z17=${1:?usage: build-z19.sh Z17_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z17" "$DTBO"; do [ -s "$f" ] || { echo "build-z19: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z19: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z17")  $Z17"
INFO=$(unpack_bootimg --boot_img "$Z17" --out "$T/z17")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z17/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z17")
AV=$(avbtool info_image --image "$Z17")
P=/soc@0/pcie@1c00000
STUB=/vendor-overlay-stubs/pcie0
cp "$T/k.dtb" "$T/z19.dtb"
ck "z17 dtb has the PCIe host $P" "$(fdtget -l "$T/k.dtb" /soc@0 | grep -c '^pcie@1c00000$')" 1
ck "z17 dtb: it is disabled" "$(fdtget "$T/k.dtb" "$P" status)" disabled
ck "z17 dtb: /__symbols__/pcie0 is $P" "$(fdtget "$T/k.dtb" /__symbols__ pcie0)" "$P"
ck "z17 dtb has no $STUB" "$(fdtget -l "$T/k.dtb" /vendor-overlay-stubs | grep -c '^pcie0$')" 0
ck "z17 dtb: the overlay overwrites it (the collision test fails on z17: 2 collisions on the one node)" "$(DTB="$T/k.dtb" QUIET=1 sh "$HERE/tests/host/overlay-collisions.sh" | tail -n1 | sed "s/.*: //; s/, [0-9]* pure additions.*//")" "2 collisions on 1 nodes, 0 allowed"
PH=$(fdtget -t u "$T/k.dtb" "$P" phandle)
fdtput -r "$T/z19.dtb" "$P"
fdtput -c "$T/z19.dtb" "$STUB"
fdtput -t u "$T/z19.dtb" "$STUB" phandle "$PH"
fdtput -t s "$T/z19.dtb" /__symbols__ pcie0 "$STUB"
cat "$T/k.gz" "$T/z19.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
# the dtb
ck "z19 dtb: $P is gone" "$(fdtget -l "$T/z19.dtb" /soc@0 | grep -c '^pcie@1c00000$')" 0
ck "z19 dtb: /__symbols__/pcie0 is $STUB" "$(fdtget "$T/z19.dtb" /__symbols__ pcie0)" "$STUB"
ck "z19 dtb: the stub carries the deleted node's phandle, defined once" "$(dtc -I dtb -O dts "$T/z19.dtb" 2>/dev/null | grep -c "phandle = <$(printf '0x%x' "$PH")>")" 1
ck "z19 dtb: the stub is empty but for its phandle" "$(fdtget -p "$T/z19.dtb" "$STUB")" phandle
for l in pcie0_clkreq_default pcie0_perst_default pcie_phy; do ck "z19 dtb: /__symbols__/$l is z17's" "$(fdtget "$T/z19.dtb" /__symbols__ $l)" "$(fdtget "$T/k.dtb" /__symbols__ $l)"; done
# nothing else of the dtb changed: decompiled, z17 and z19 differ in the PCIe node (removed), the stub and its symbol
dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null > "$T/k.dts"
dtc -I dtb -O dts "$T/z19.dtb" 2>/dev/null > "$T/z19.dts"
ck "z19 dtb adds 5 lines to z17's decompiled dts (the stub node 4 incl. its blank line, the changed symbol)" "$(diff "$T/k.dts" "$T/z19.dts" | grep -c "^>")" 5
ck "z19 dtb removes the PCIe node (its lines and the blank line after it) and the old pcie0 symbol line, nothing else" "$(diff "$T/k.dts" "$T/z19.dts" | grep -c "^<")" "$(awk '/^\t\tpcie0: pcie@1c00000 \{/{on=1} on{n++} on&&/^\t\t\};/{on=0} END{print n+2}' "$T/k.dts")"
ck "z19 dtb keeps the qup node, qcom,i2c-qup-v2.2.1" "$(fdtget "$T/z19.dtb" /soc@0/qup@c179000 compatible)" qcom,i2c-qup-v2.2.1
ck "z19 dtb keeps the touch power regulator" "$(fdtget "$T/z19.dtb" /regulator-touch-power compatible)" regulator-fixed
ck "z19 dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/z19.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "z19 dtb keeps qcom,msm-id <0x124 0x20001>" "$(fdtget -t x "$T/z19.dtb" / qcom,msm-id)" "124 20001"
ck "z19 dtb keeps the ramoops node at 0xa1100000" "$(fdtget -t x "$T/z19.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "z19 dtb: clock_gcc is the real gcc" "$(fdtget "$T/z19.dtb" /__symbols__ clock_gcc)" /soc@0/clock-controller@100000
ck "z19 dtb: the GPU smmu is disabled (the test passes)" "$(DTB="$T/z19.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d" " -f1)" pass
# the collision test: every node the overlay writes by name is not ours
ck "z19 dtb: the overlay overwrites nothing of ours (tests/host/overlay-collisions.sh)" "$(DTB="$T/z19.dtb" QUIET=1 sh "$HERE/tests/host/overlay-collisions.sh" | tail -n1 | cut -d' ' -f1)" pass
ck "the overlay still merges into z19's dtb" "$(DTB="$T/z19.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the clock users name gcc" "$(DTB="$T/z19.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the qup controller is ours, with the touch controller" "$(DTB="$T/z19.dtb" sh "$HERE/tests/host/i2c-qup-after-merge.sh" | cut -d' ' -f1)" pass
ck "after the overlay merge the touch controller's vdd-supply is still the touch regulator" "$(DTB="$T/z19.dtb" sh "$HERE/tests/host/touch-power-after-merge.sh" | cut -d' ' -f1)" pass
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z19.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z17/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "z19 Image.gz is z17's, byte for byte" "$(cmp -s "$T/uk.gz" "$T/k.gz" && echo same || echo differs)" same
ck "z19 dtb is z19.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/z19.dtb" && echo same || echo differs)" same
ck "z19 header equals z17's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z19 ramdisk bytes equal z17's" "$(cmp -s "$T/u/ramdisk" "$T/z17/ramdisk" && echo same || echo differs)" same
ck "z19 size is z17's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z19 AVB info equals z17's but the digest" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z19 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z19: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z19.img"; cp "$T/z19.dtb" "$OUT/msm8998-google-walleye-z19.dtb"
(cd "$OUT" && sha256sum boot-z19.img msm8998-google-walleye-z19.dtb | tee SHA256SUMS-z19)
