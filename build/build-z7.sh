#!/bin/sh
# z7 (backlog walleye-dmesg-clean): z6 with ONE change, the dtb: the GPU's smmu (/__symbols__/adreno_smmu, iommu@5040000) gets status "disabled".
# Its power domain GPU_GX_GDSC cannot switch on (no GFX rail in our dts): six "gpu_gx status stuck" WARNs and a probe that defers for ever (research walleye-z6-dmesg-mapped, G1).
# The same line is in dt/msm8998-google-walleye.dts for every later build; z6's dtb is z3's (a dtb of an older dts), so this image edits that dtb with fdtput, the way build-z3.sh did.
# Kernel Image.gz, ramdisk (init-z6), header, command line and AVB footer are z6's.
#   build-z7.sh Z6_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-z7.img,msm8998-google-walleye-z7.dtb,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
set -eu
Z6=${1:?usage: build-z7.sh Z6_BOOT_IMG OUT_DIR}
OUT=${2:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
[ -s "$Z6" ] || { echo "build-z7: missing $Z6" >&2; exit 1; }
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z7: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z6")  $Z6"
INFO=$(unpack_bootimg --boot_img "$Z6" --out "$T/z6")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z6/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z6")
AV=$(avbtool info_image --image "$Z6")
N=$(fdtget "$T/k.dtb" /__symbols__ adreno_smmu)
cp "$T/k.dtb" "$T/z7.dtb"
fdtput -t s "$T/z7.dtb" "$N" status disabled
cat "$T/k.gz" "$T/z7.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
dts() { dtc -I dtb -O dts "$1" 2>/dev/null; }
dts "$T/k.dtb" > "$T/a.dts"; dts "$T/z7.dtb" > "$T/b.dts"
ck "z6 dtb: adreno_smmu is iommu@5040000" "$N" /soc@0/iommu@5040000
ck "z6 dtb: the smmu has no status (the test fails on it)" "$(DTB="$T/k.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d' ' -f1 || true)" fail
ck "z7 dtb: the smmu is disabled (the test passes)" "$(DTB="$T/z7.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d' ' -f1)" pass
ck "z7 dtb differs from z6's in the one status line" "$(diff "$T/a.dts" "$T/b.dts" | grep '^[<>]' | tr -s '\t' ' ' | tr '\n' '|')" '> status = "disabled";|'
ck "the dts source disables adreno_smmu too" "$(awk '/^&adreno_smmu \{/{f=1;next} f&&/status = "disabled";/{print "yes";exit} /^\}/{f=0}' "$HERE/dt/msm8998-google-walleye.dts")" yes
ck "the dtbo overlay still merges into z7.dtb" "$(DTB="$T/z7.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1 || true)" pass
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z7.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z6/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
ck "z7 Image.gz is z6's, byte for byte" "$(cmp -s "$T/uk.gz" "$T/k.gz" && echo same || echo differs)" same
ck "z7 dtb is z7.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/z7.dtb" && echo same || echo differs)" same
ck "z7 header equals z6's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z7 ramdisk (init-z6) equals z6's, byte for byte" "$(cmp -s "$T/u/ramdisk" "$T/z6/ramdisk" && echo same || echo differs)" same
ck "z7 size is z6's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z7 AVB info equals z6's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z7 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z7: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z7.img"; cp "$T/z7.dtb" "$OUT/msm8998-google-walleye-z7.dtb"
(cd "$OUT" && sha256sum boot-z7.img msm8998-google-walleye-z7.dtb | tee SHA256SUMS)
