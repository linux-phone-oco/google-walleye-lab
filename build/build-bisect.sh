#!/bin/sh
# The bisect from the stock control to our image (decision walleye-bisect-from-the-control): images that are the control's own
# boot.img parts, with exactly one part swapped each. Host only; nothing here touches the phone.
#
#   build-bisect.sh CONTROL_BOOT_IMG OUR_IMAGE OUR_DTB OUR_RAMDISK_43 OUT_DIR [OUR_DTB_OVERLAY]
#     CONTROL_BOOT_IMG  the stock boot.img the bootloader accepts and starts (LineageOS 22.2 walleye)
#     OUR_IMAGE         our kernel, the raw arm64 Image (build-kernel.sh's out/b/arch/arm64/boot/Image)
#     OUR_DTB           our dtb (out/msm8998-google-walleye.dtb)
#     OUR_RAMDISK_43    our initramfs built NET=43 (172.16.43.1)
#     OUR_DTB_OVERLAY   optional: our dtb built with -@ and the overlay stubs (decision walleye-dtb-carries-overlay-symbols): adds s3o
#
# Every image takes the control's header fields (offsets, page size, os version and patch level, command line) as unpack_bootimg
# reads them, its kernel blob = lz4 frame of an Image + one appended dtb, its ramdisk, and its AVB hash footer (avbtool,
# algorithm NONE as the control's, padded to the control's size, same properties). Images (one change against the control):
#   s0   none: the control's parts packed again (proof that the tool chain reproduces the control: only the header id differs)
#   s0n  s0 without the AVB footer and padding (the shape of a-i): did the footer matter?
#   s1   ramdisk = ours (172.16.43.1); the stock 4.4 kernel, stock dtb, stock command line
#   s2   kernel  = ours, packed as an lz4 frame like the stock one (stock dtb, ramdisk, command line)
#   s3   dtb     = ours (stock kernel, ramdisk, command line)
#   s3o  dtb     = ours with __symbols__ and the 130 overlay stubs (stock kernel, ramdisk, command line); the dtbo overlay merges into it
# Writes OUT_DIR/boot-s*.img, OUT_DIR/parts/*, OUT_DIR/BISECT.txt and checks each image with check-bisect.sh.
set -eu
CTRL=${1:?usage: build-bisect.sh CONTROL_BOOT_IMG OUR_IMAGE OUR_DTB OUR_RAMDISK_43 OUT_DIR}
OURK=${2:?}
OURDTB=${3:?}
OURRD=${4:?}
OUT=${5:?}
OURDTBO=${6:-}
HERE=$(cd "$(dirname "$0")" && pwd)
P=$OUT/parts
rm -rf "$P"; mkdir -p "$P"
INFO=$(unpack_bootimg --boot_img "$CTRL" --out "$P/control")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
[ "$(field 'boot image header version')" = 0 ] || { echo "build-bisect: the control is not a header v0 image" >&2; exit 1; }
PAGE=$(field 'page size'); KADDR=$(field 'kernel load address'); RADDR=$(field 'ramdisk load address')
TADDR=$(field 'kernel tags load address'); OSV=$(field 'os version'); OSP=$(field 'os patch level'); CMD=$(field 'command line args')
# the control's kernel blob: an lz4 frame, then one flattened device tree to the end of the file
KB=$P/control/kernel
KSIZE=$(stat -c %s "$KB")
DOFF=
for o in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$KB" | cut -d: -f1); do
  TS=$(od -An -tu1 -j $((o+4)) -N4 "$KB" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
  [ $((o+TS)) -eq "$KSIZE" ] && DOFF=$o
done
[ -n "$DOFF" ] || { echo "build-bisect: no dtb ends the control's kernel blob" >&2; exit 1; }
head -c "$DOFF" "$KB" > "$P/stock-kernel.lz4"
tail -c +$((DOFF+1)) "$KB" > "$P/stock.dtb"
fdtdump "$P/stock.dtb" >/dev/null
lz4 -dc "$P/stock-kernel.lz4" > "$P/stock-Image" 2>/dev/null
# our kernel in the same container as the stock one: lz4 frame, independent 4 MiB blocks, content checksum (the control's FLG/BD)
lz4 -9 -BI -B7 -c "$OURK" > "$P/our-kernel.lz4"
[ "$(od -An -tx1 -N6 "$P/our-kernel.lz4" | tr -d ' ')" = "04224d186470" ] || [ "$(od -An -tx1 -N6 "$P/stock-kernel.lz4" | tr -d ' ')" = "$(od -An -tx1 -N6 "$P/our-kernel.lz4" | tr -d ' ')" ] \
  || { echo "build-bisect: our lz4 frame header differs from the control's" >&2; exit 1; }
# the control's AVB footer: reuse its properties and size
AV=$(avbtool info_image --image "$CTRL")
PSIZE=$(stat -c %s "$CTRL")
PROPS=$(avbtool info_image --image "$CTRL" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
mk() {  # mk NAME KERNEL_BLOB RAMDISK FOOTER(yes|no)
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$KADDR" --ramdisk_offset "$RADDR" --second_offset 0x0 \
    --tags_offset "$TADDR" --pagesize "$PAGE" --os_version "$OSV" --os_patch_level "$OSP" --cmdline "$CMD" \
    --kernel "$2" --ramdisk "$3" -o "$OUT/boot-$1.img"
  if [ "$4" = yes ]; then
    printf '%s\n' "$PROPS" > "$P/props.txt"
    avbtool add_hash_footer --image "$OUT/boot-$1.img" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
      --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" \
      $(while IFS= read -r pr; do [ -n "$pr" ] && printf -- '--prop %s ' "$pr"; done < "$P/props.txt")
  fi
}
cat "$P/stock-kernel.lz4" "$P/stock.dtb" > "$P/blob-stock"
cat "$P/our-kernel.lz4" "$P/stock.dtb" > "$P/blob-s2"
cat "$P/stock-kernel.lz4" "$OURDTB" > "$P/blob-s3"
mk s0  "$P/blob-stock" "$P/control/ramdisk" yes
mk s0n "$P/blob-stock" "$P/control/ramdisk" no
mk s1  "$P/blob-stock" "$OURRD"             yes
mk s2  "$P/blob-s2"    "$P/control/ramdisk" yes
mk s3  "$P/blob-s3"    "$P/control/ramdisk" yes
if [ -n "$OURDTBO" ]; then cat "$P/stock-kernel.lz4" "$OURDTBO" > "$P/blob-s3o"; mk s3o "$P/blob-s3o" "$P/control/ramdisk" yes; cp "$OURDTBO" "$P/our-overlay.dtb"; fi
cp "$P/control/ramdisk" "$P/stock-ramdisk"
cp "$OURRD" "$P/our-ramdisk"; cp "$OURDTB" "$P/our.dtb"
cat > "$OUT/BISECT.txt" <<EOF
control: $CTRL (sha256 $(sha256sum "$CTRL" | cut -d' ' -f1))
header: base 0, kernel $KADDR, ramdisk $RADDR, tags $TADDR, second 0, page $PAGE, os $OSV / $OSP, v0
command line: $CMD
s0   the control's parts packed again, AVB footer (algorithm NONE, padded to $PSIZE)
s0n  s0 without footer and padding (the shape of images a-i)
s1   ramdisk = ours (172.16.43.1, gzip)           stock kernel (lz4 frame, 4.4), stock dtb
s2   kernel  = ours (raw Image in an lz4 frame)   stock dtb, ramdisk
s3   dtb     = ours (msm8998-google-walleye.dtb)  stock kernel, ramdisk
$( [ -n "$OURDTBO" ] && echo "s3o  dtb     = ours with -@ and overlay stubs (msm8998-google-walleye.dtb, new)  stock kernel, ramdisk" )
EOF
"$HERE/check-bisect.sh" "$CTRL" "$OUT"
sha256sum "$OUT"/boot-s*.img
