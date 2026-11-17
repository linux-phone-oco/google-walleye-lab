#!/bin/sh
# The first walleye boot images: header v0, the dtb appended to the gzipped kernel, the initramfs as ramdisk.
# Walleye's own stock boot.img header is not read yet, so the two offset sets the research names are both built:
#   a  taimen's stock values (porthole, verified against the stock TAIMEN boot.img): ramdisk 0x1000000, tags 0x100
#   b  LineageOS's wahoo board config: ramdisk 0x2000000, tags 0x1e00000
# (base 0, kernel offset 0x8000, second 0xf00000, page size 4096 in both.) No fastboot here: building only.
#
#   build-bootimg.sh IMAGE_GZ DTB RAMDISK_GZ OUT_DIR [DTB_F [RAMDISK_43]]   (DTB_F: the dtb with qcom,board-id, for f; RAMDISK_43: for g, h, i)
#
# Writes OUT_DIR/boot-{a,b,c,d}.img (and e, f when DTB_F is given, g, h, i when RAMDISK_43 is too) and checks each with check-bootimg.sh.
set -eu
KERNEL=${1:?usage: build-bootimg.sh IMAGE_GZ DTB RAMDISK_GZ OUT_DIR}
DTB=${2:?}
RAMDISK=${3:?}
OUT=${4:?}
DTB_F=${5:-}
RAMDISK_43=${6:-}   # the initramfs built with NET=43, for g and h
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
# e and f (after the stock boot.img of LineageOS 22.2 was read: ramdisk 0x1000000, tags 0x100, second 0 - a's offsets, the
# blueline recipe). The os word stays 0: a was the only silent OKAY. One change each against a:
#   e  second_offset 0 (a no-op: with no second image mkbootimg writes 0 either way, the header is a's) and a command line cut to earlycon + console
#   f  e plus qcom,board-id <8 0> in the dtb (the stock dtb has none; see dt/msm8998-google-walleye-f.dts)
if [ -n "$DTB_F" ]; then
  CMDLINE_E="earlycon=msm_serial_dm,0xc1b0000 console=ttyGS0,115200"
  mke() {  # mke NAME DTB [BOARD_ID]
    cat "$KERNEL" "$2" > "$OUT/kernel-dtb-$1"
    mkbootimg --header_version 0 --base 0x0 --kernel_offset 0x8000 --ramdisk_offset 0x1000000 --second_offset 0x0 \
      --tags_offset 0x100 --pagesize 4096 --kernel "$OUT/kernel-dtb-$1" --ramdisk "$RAMDISK" --cmdline "$CMDLINE_E" \
      -o "$OUT/boot-$1.img"
    BOARD_ID="${3:-}" "$HERE/check-bootimg.sh" "$OUT/boot-$1.img" 0x1000000 0x100
  }
  mke e "$DTB"
  mke f "$DTB_F" "8 0"
fi
# g, h and i: e, b and f with the initramfs answering on 172.16.43.1 (NET=43), nothing else changed - walleye and surya share one
# host and surya's own gadget is 172.16.42.1. Only the ramdisk differs; check-bootimg.sh also counts the address in /init.
#   g = e (the first try's image), h = b (LineageOS offsets, the second), i = f (e plus qcom,board-id)
if [ -n "$DTB_F" ] && [ -n "$RAMDISK_43" ]; then
  mkg() {  # mkg NAME KERNEL_DTB RAMDISK_OFFSET TAGS_OFFSET SECOND_OFFSET CMDLINE [BOARD_ID]
    mkbootimg --header_version 0 --base 0x0 --kernel_offset 0x8000 --ramdisk_offset "$3" --second_offset "$5" \
      --tags_offset "$4" --pagesize 4096 --kernel "$2" --ramdisk "$RAMDISK_43" --cmdline "$6" -o "$OUT/boot-$1.img"
    NET=43 BOARD_ID="${7:-}" "$HERE/check-bootimg.sh" "$OUT/boot-$1.img" "$3" "$4"
  }
  mkg g "$OUT/kernel-dtb-e" 0x1000000 0x100 0x0 "$CMDLINE_E"
  mkg h "$OUT/kernel-dtb" 0x2000000 0x1e00000 0xf00000 "$CMDLINE"
  mkg i "$OUT/kernel-dtb-f" 0x1000000 0x100 0x0 "$CMDLINE_E" "8 0"
fi
