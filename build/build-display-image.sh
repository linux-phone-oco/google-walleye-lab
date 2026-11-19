#!/bin/sh
# The boot image of the display try (item walleye-display-s6e3fa5): z21's image with the display kernel (Image.gz with the S6E3FA5 panel driver, DPU and DSI built in) and the display dtb in place of z21's pair; ramdisk, header,
# command line and AVB footer are z21's. UFS is off in the dtb (the gate, decision 2026-11-19-walleye-ufs-off-by-a-gate, checked here on the finished image). Built only - no phone run; the keeper tries it once from slot b (walleye-try-display).
#   build-display-image.sh Z21_BOOT_IMG DISPLAY_DIR OUT_DIR      (DISPLAY_DIR: build-display.sh's OUT_DIR)
# Writes OUT_DIR/{boot-display.img,SHA256SUMS-display-img}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
Z21=${1:?usage: build-display-image.sh Z21_BOOT_IMG DISPLAY_DIR OUT_DIR}
D=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z21" "$D/Image.gz" "$D/msm8998-google-walleye-display.dtb"; do [ -s "$f" ] || { echo "build-display-image: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$Z21" --out "$T/z21")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$Z21")
AV=$(avbtool info_image --image "$Z21")
cat "$D/Image.gz" "$D/msm8998-google-walleye-display.dtb" > "$T/blob"
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf "%s\n" "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-display.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z21/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf "%s\n" "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
ck "the kernel blob is Image.gz then the display dtb, byte for byte" "$(cmp -s "$T/u/kernel" "$T/blob" && echo same || echo differs)" same
ck "the ramdisk is z21's, byte for byte" "$(cmp -s "$T/u/ramdisk" "$T/z21/ramdisk" && echo same || echo differs)" same
ck "the command line is z21's" "$(printf "%s\n" "$U" | sed -n 's/^command line args: *//p')" "$(field 'command line args')"
ck "the size is z21's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
# compressed size, as z21 measured: z21 boots with 43.4 MB uncompressed against a ramdisk at 16 MB, so the loader does not unpack in place
KS=$(stat -c %s "$T/blob"); KO_=$(printf %d "$(field "kernel load address")"); RO=$(printf %d "$(field "ramdisk load address")")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate passes the image (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d" " -f1 || true)" pass
ck "the display dtb keeps the touch power gpio single-claimant after the overlay merge" "$(DTB="$D/msm8998-google-walleye-display.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
ck "the overlay still merges into the display dtb" "$(DTB="$D/msm8998-google-walleye-display.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
[ $total -eq 0 ] || { echo "build-display-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-display.img"
(cd "$OUT" && sha256sum boot-display.img | tee SHA256SUMS-display-img)
