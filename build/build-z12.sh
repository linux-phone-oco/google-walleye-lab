#!/bin/sh
# z12 (backlog walleye-ufs-hynix-quirks): z11 with ONE change, the kernel: the Hynix UFS quirks (extended sync length, no purge, one TX lane) in ufs-qcom's fixups table
# (linux-phone-oco/linux topic/ufs-hynix-quirks). dtb, ramdisk, header, command line and AVB footer are z11's, byte for byte. Built only - no phone run; the UFS try waits for the author's word.
#   build-z12.sh WORK_DIR Z11_BOOT_IMG KERNEL_TREE KERNEL_OUT_DIR Z11_KERNEL_OUT_DIR OUT_DIR
# KERNEL_TREE is the tree KERNEL_OUT_DIR was built from (build-kernel.sh); Z11_KERNEL_OUT_DIR is z11's. Writes OUT_DIR/{boot-z12.img,SHA256SUMS-z12}; checks (ok/FAIL, exit 1, nothing written).
set -eu
W=${1:?usage: build-z12.sh WORK_DIR Z11_BOOT_IMG KERNEL_TREE KERNEL_OUT_DIR Z11_KERNEL_OUT_DIR OUT_DIR}
Z11=${2:?}
KT=${3:?}
KO=${4:?}
CO=${5:?}
OUT=${6:?}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z11" "$KT/drivers/ufs/host/ufs-qcom.c" "$KO/Image.gz" "$KO/.config" "$KO/msm8998-google-walleye.dtb" "$CO/Image.gz" "$CO/msm8998-google-walleye.dtb"; do [ -s "$f" ] || { echo "build-z12: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z12: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
INFO=$(unpack_bootimg --boot_img "$Z11" --out "$T/z11")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z11/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z11")
AV=$(avbtool info_image --image "$Z11")
# the fixups table of the built tree: SKHYNIX with the three quirks
TBL=$(sed -n '/^static struct ufs_dev_quirk ufs_qcom_dev_fixups\[\]/,/^};/p' "$KT/drivers/ufs/host/ufs-qcom.c")
ck "the fixups table names SKHYNIX" "$(printf '%s\n' "$TBL" | grep -c 'UFS_VENDOR_SKHYNIX')" 1
for q in EXTEND_SYNC_LENGTH NO_PURGE ONE_TX_LANE; do
  ck "the SKHYNIX entry has UFS_DEVICE_QUIRK_$q" "$(printf '%s\n' "$TBL" | sed -n '/SKHYNIX/,/},/p' | grep -c "UFS_DEVICE_QUIRK_$q")" 1
done
ck "the purge guard is in ufshcd_query_flag" "$(grep -c 'hba->dev_quirks & UFS_DEVICE_QUIRK_NO_PURGE' "$KT/drivers/ufs/core/ufshcd.c")" 1
ck "the kernel config still builds UFS in" "$(grep -c '^CONFIG_SCSI_UFS_QCOM=y$' "$KO/.config")" 1
ck "the new Image.gz is not z11's (the kernel change)" "$(cmp -s "$KO/Image.gz" "$CO/Image.gz" && echo same || echo differs)" differs
ck "the built dtb is z11's source dtb (same build inputs)" "$(cmp -s "$KO/msm8998-google-walleye.dtb" "$CO/msm8998-google-walleye.dtb" && echo same || echo differs)" same
ck "z11's kernel out dir is the Image.gz inside the z11 image" "$(cmp -s "$CO/Image.gz" "$T/k.gz" && echo same || echo differs)" same
cat "$KO/Image.gz" "$T/k.dtb" > "$T/blob"
set --
while IFS= read -r pr; do if [ -n "$pr" ]; then set -- "$@" --prop "$pr"; fi; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z12.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z11/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
hdr() { printf '%s\n' "$1" | grep -v -e 'ramdisk size' -e 'kernel_size' -e 'kernel size'; }
ck "z12 Image.gz is the new build's, byte for byte" "$(cmp -s "$T/uk.gz" "$KO/Image.gz" && echo same || echo differs)" same
ck "z12 dtb is z11's, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/k.dtb" && echo same || echo differs)" same
ck "z12 ramdisk is z11's, byte for byte" "$(cmp -s "$T/u/ramdisk" "$T/z11/ramdisk" && echo same || echo differs)" same
ck "z12 header equals z11's but the sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z12 size is z11's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z12 AVB info equals z11's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z12 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z12: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z12.img"
(cd "$OUT" && sha256sum boot-z12.img | tee SHA256SUMS-z12)
