#!/bin/sh
# c3 (backlog walleye-charger-supplier): c2 with ONLY the kernel command line changed, fw_devlink=permissive added at its end.
# Kernel (Image.gz + dtb blob), ramdisk, header fields, size and AVB footer are c2's; the check counts that.
#   build-c3.sh C2_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-c3.img,SHA256SUMS-c3}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
set -eu
C2=${1:?usage: build-c3.sh C2_BOOT_IMG OUT_DIR}
OUT=${2:?}
ADD="fw_devlink=permissive"
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
[ -s "$C2" ] || { echo "build-c3: missing $C2" >&2; exit 1; }
echo "input $(sha256sum "$C2" | cut -c1-16)  $C2"
INFO=$(unpack_bootimg --boot_img "$C2" --out "$T/c2")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$C2")
AV=$(avbtool info_image --image "$C2")
OLD=$(field 'command line args')
ck "c2's command line does not have $ADD yet" "$(printf '%s\n' "$OLD" | grep -c "$ADD")" 0
NEW="$OLD $ADD"
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-c3.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$NEW" \
  --kernel "$T/c2/kernel" --ramdisk "$T/c2/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
ufield() { printf "%s\n" "$U" | sed -n "s/^$1: *//p" | head -n1; }
ck "c3 kernel (Image.gz + dtb) equals c2's, byte for byte" "$(cmp -s "$T/u/kernel" "$T/c2/kernel" && echo same || echo differs)" same
ck "c3 ramdisk equals c2's, byte for byte" "$(cmp -s "$T/u/ramdisk" "$T/c2/ramdisk" && echo same || echo differs)" same
ck "c3 command line is c2's plus ' $ADD'" "$(ufield 'command line args')" "$NEW"
ck "c3 header equals c2's but the command line" "$(printf '%s\n' "$U" | grep -v -e '^command line args')" "$(printf '%s\n' "$INFO" | grep -v -e '^command line args')"
ck "c3 size is c2's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "c3 AVB info equals c2's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "c3 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-c3: $total checks failed, nothing written" >&2; exit 1; }
echo "cmdline diff:"; echo "  c2: $OLD"; echo "  c3: $NEW"
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-c3.img"
(cd "$OUT" && sha256sum boot-c3.img | tee SHA256SUMS-c3)
