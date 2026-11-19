#!/bin/sh
# pk5g (backlog walleye-pk5-gadget-stages): g1, g2, g3 = pk5 (OUR kernel + porthole's taimen dtb, alive) with ONLY the ramdisk changed: initramfs-mainline/init-pk5g
# (one init; the stage is /stage in the ramdisk) brings the mainline gadget up one step per image, each holding 60 s. Host only; nothing touches the phone.
#   build-pk5g.sh WORK_DIR PK5_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-g1.img,boot-g2.img,boot-g3.img,initramfs-g[123].cpio.xz,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL) that kernel, header and size equal pk5's
# and that the three ramdisks differ in /stage alone.
set -eu
W=${1:?usage: build-pk5g.sh WORK_DIR PK5_BOOT_IMG OUT_DIR}
PK5=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
CFG=$W/out-ml/.config
I=$HERE/initramfs-mainline/init-pk5g
total=0
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$PK5" "$I" "$CFG"; do [ -s "$f" ] || { echo "build-pk5g: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$PK5" --out "$T/p")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$PK5")
AV=$(avbtool info_image --image "$PK5")
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
hdr() { echo "$1" | grep -v -e 'ramdisk size'; }
for s in 1 2 3; do
  echo $s > "$T/stage$s"
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
    echo "file /bin/busybox $BB 755 0 0"
    echo "slink /bin/sh busybox 777 0 0"
    echo "file /init $I 755 0 0"
    echo "file /stage $T/stage$s 644 0 0"
  } > "$T/g$s.spec"
  "$GEN" -t 0 "$T/g$s.spec" > "$T/rd$s.cpio"
  xz --check=crc32 --lzma2=preset=6e -c "$T/rd$s.cpio" > "$OUT/initramfs-g$s.cpio.xz"
  IMG=$OUT/boot-g$s.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
    --kernel "$T/p/kernel" --ramdisk "$OUT/initramfs-g$s.cpio.xz" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$s")
  ck "g$s kernel (Image.gz + dtb) bytes equal pk5's" "$(cmp -s "$T/p/kernel" "$T/u$s/kernel" && echo same || echo differs)" same
  ck "g$s header (offsets, page, os word, command line, kernel size) equals pk5's but the ramdisk size" "$(hdr "$U")" "$(hdr "$INFO")"
  ck "g$s ramdisk differs from pk5's (the one change)" "$(cmp -s "$T/p/ramdisk" "$T/u$s/ramdisk" && echo same || echo differs)" differs
  ck "g$s ramdisk is this build's xz" "$(cmp -s "$T/u$s/ramdisk" "$OUT/initramfs-g$s.cpio.xz" && echo same || echo differs)" same
  mkdir "$T/x$s" && (cd "$T/x$s" && xz -dc "$T/u$s/ramdisk" | cpio -id --quiet 2>/dev/null || true)
  ck "g$s init in the ramdisk is init-pk5g" "$(cmp -s "$T/x$s/init" "$I" && echo same || echo differs)" same
  ck "g$s /stage says $s" "$(cat "$T/x$s/stage")" "$s"
  ck "g$s ramdisk has /init, /stage, /bin/busybox, /dev/console" "$(xz -dc "$T/u$s/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e stage -e bin/busybox -e dev/console)" 4
  ck "g$s busybox in it is aarch64 static" "$(file -b "$T/x$s/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
  ck "g$s size is pk5's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  mkdir "$T/v$s" && cp "$IMG" "$T/v$s/boot.img"
  ck "g$s AVB footer verifies" "$(avbtool verify_image --image "$T/v$s/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
done
ck "g1 and g2 ramdisks list the same files" "$(cd "$T/x1" && find . | sort | md5sum)" "$(cd "$T/x2" && find . | sort | md5sum)"
ck "g2 and g3 ramdisks list the same files" "$(cd "$T/x2" && find . | sort | md5sum)" "$(cd "$T/x3" && find . | sort | md5sum)"
ck "g1/g3 trees differ in /stage only" "$(diff -rq "$T/x1" "$T/x3" | sed 's/ and .*//' )" "Files $T/x1/stage"
ck "init has no bashisms (busybox ash -n)" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
ck "init has no ACM, no shell loop, no /dev/ttyGS0" "$(grep -v '^#' "$I" | grep -c -e acm -e ttyGS0 -e 'ash -i')" 0
ck "init holds 60 s (one beat per second, 60 beats)" "$(grep -c -e '-lt 60 \]' "$I")" 1
for a in ash mount mkdir dd ls ln head sleep reboot uname; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$(echo "$a" | tr a-z A-Z)=y" "$W/busybox-arm64.src/.config")" 1
done
for k in USB_GADGET USB_CONFIGFS USB_CONFIGFS_MASS_STORAGE USB_DWC3 USB_DWC3_QCOM USB_DWC3_DUAL_ROLE PHY_QCOM_QUSB2 QCOM_WDT CONFIGFS_FS DEVTMPFS RD_XZ BLK_DEV_INITRD; do
  ck "kernel config has CONFIG_$k" "$(grep -cE "^CONFIG_$k=[ym]$" "$CFG")" 1
done
(cd "$OUT" && sha256sum initramfs-g1.cpio.xz initramfs-g2.cpio.xz initramfs-g3.cpio.xz boot-g1.img boot-g2.img boot-g3.img | tee SHA256SUMS)
[ $total -eq 0 ]
