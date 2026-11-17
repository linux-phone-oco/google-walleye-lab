#!/bin/sh
# The first walleye boot images: header v0, the dtb appended to the gzipped kernel, the initramfs as ramdisk.
# Walleye's own stock boot.img header is not read yet, so the two offset sets the research names are both built:
#   a  taimen's stock values (porthole, verified against the stock TAIMEN boot.img): ramdisk 0x1000000, tags 0x100
#   b  LineageOS's wahoo board config: ramdisk 0x2000000, tags 0x1e00000
# (base 0, kernel offset 0x8000, second 0xf00000, page size 4096 in both.) No fastboot here: building only.
#
#   build-bootimg.sh IMAGE_GZ DTB RAMDISK_GZ OUT_DIR
#
# Writes OUT_DIR/boot-a.img and OUT_DIR/boot-b.img and checks each with check-bootimg.sh.
set -eu
KERNEL=${1:?usage: build-bootimg.sh IMAGE_GZ DTB RAMDISK_GZ OUT_DIR}
DTB=${2:?}
RAMDISK=${3:?}
OUT=${4:?}
HERE=$(cd "$(dirname "$0")" && pwd)
# The device tree carries no bootargs (the bootloader appends this line to it): what the first try needs.
CMDLINE="clk_ignore_unused ignore_loglevel panic=10 earlycon=msm_serial_dm,0xc1b0000 console=ttyGS0,115200 reboot=w"
mkdir -p "$OUT"
cat "$KERNEL" "$DTB" > "$OUT/kernel-dtb"
mk() {  # mk NAME RAMDISK_OFFSET TAGS_OFFSET
  mkbootimg --header_version 0 --base 0x0 --kernel_offset 0x8000 --ramdisk_offset "$2" --second_offset 0xf00000 \
    --tags_offset "$3" --pagesize 4096 --kernel "$OUT/kernel-dtb" --ramdisk "$RAMDISK" --cmdline "$CMDLINE" \
    -o "$OUT/boot-$1.img"
  "$HERE/check-bootimg.sh" "$OUT/boot-$1.img" "$2" "$3"
}
mk a 0x1000000 0x100
mk b 0x2000000 0x1e00000
