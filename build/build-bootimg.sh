#!/bin/sh
# The first walleye boot images: header v0, the dtb appended to the gzipped kernel, the initramfs as ramdisk.
# Walleye's own stock boot.img header is not read yet, so the two offset sets the research names are both built:
#   a  taimen's stock values (porthole, verified against the stock TAIMEN boot.img): ramdisk 0x1000000, tags 0x100
#   b  LineageOS's wahoo board config: ramdisk 0x2000000, tags 0x1e00000
# (base 0, kernel offset 0x8000, second 0xf00000, page size 4096 in both.) No fastboot here: building only.
#
#   build-bootimg.sh IMAGE_GZ DTB RAMDISK_GZ OUT_DIR
#
# Writes OUT_DIR/boot-{a,b,c,d}.img and checks each with check-bootimg.sh.
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
mk() {  # mk NAME RAMDISK_OFFSET TAGS_OFFSET [OS_VERSION OS_PATCH_LEVEL]
  mkbootimg --header_version 0 --base 0x0 --kernel_offset 0x8000 --ramdisk_offset "$2" --second_offset 0xf00000 \
    --tags_offset "$3" ${4:+--os_version "$4" --os_patch_level "$5"} --pagesize 4096 --kernel "$OUT/kernel-dtb" --ramdisk "$RAMDISK" --cmdline "$CMDLINE" \
    -o "$OUT/boot-$1.img"
  "$HERE/check-bootimg.sh" "$OUT/boot-$1.img" "$2" "$3" "${4:-}" "${5:-}"
}
mk a 0x1000000 0x100
mk b 0x2000000 0x1e00000
# c and d: a and b with the header's os_version and patch level set (a and b carry none, as mkbootimg writes by default).
# The AOSP build puts both into every boot image (--os_version PLATFORM_VERSION --os_patch_level PLATFORM_SECURITY_PATCH);
# a bootloader's rollback check may read them. 8.1.0 is LineageOS 15.1's Android; 2020-01 is a pick, not read from the phone.
mk c 0x1000000 0x100 8.1.0 2020-01
mk d 0x2000000 0x1e00000 8.1.0 2020-01
