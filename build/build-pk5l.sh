#!/bin/sh
# pk5l (backlog walleye-pk5-light): pk5 (OUR kernel + porthole's taimen dtb, no earlycon; it ran init) with ONLY the ramdisk's /init changed: initramfs-mainline/init-pk5l,
# a gadget with an ACM console and the mass_storage log window. Host only; nothing touches the phone.
#   build-pk5l.sh WORK_DIR PK5_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-pk5l.img,initramfs-pk5l.cpio.xz,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL) that kernel, dtb and header bytes equal pk5's and
# that the kernel config has what the gadget needs.
set -eu
W=${1:?usage: build-pk5l.sh WORK_DIR PK5_BOOT_IMG OUT_DIR}
PK5=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
CFG=$W/out-ml/.config
I=$HERE/initramfs-mainline/init-pk5l
total=0
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$PK5" "$I" "$CFG"; do [ -s "$f" ] || { echo "build-pk5l: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$PK5" --out "$T/p")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$PK5")
AV=$(avbtool info_image --image "$PK5")
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
{
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"
  echo "slink /bin/sh busybox 777 0 0"
  echo "file /init $I 755 0 0"
} > "$T/pk5l.spec"
"$GEN" -t 0 "$T/pk5l.spec" > "$T/rd.cpio"
xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$OUT/initramfs-pk5l.cpio.xz"
IMG=$OUT/boot-pk5l.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/p/kernel" --ramdisk "$OUT/initramfs-pk5l.cpio.xz" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
hdr() { echo "$1" | grep -v -e 'ramdisk size'; }
ck "kernel (Image.gz + dtb) bytes equal pk5's" "$(cmp -s "$T/p/kernel" "$T/u/kernel" && echo same || echo differs)" same
ck "header (offsets, page, os word, command line, kernel size) equals pk5's but the ramdisk size" "$(hdr "$U")" "$(hdr "$INFO")"
ck "command line has no earlycon and no console=" "$(echo "$U" | sed -n 's/^command line args: *//p' | head -n1 | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
ck "ramdisk differs from pk5's (the one change)" "$(cmp -s "$T/p/ramdisk" "$T/u/ramdisk" && echo same || echo differs)" differs
ck "ramdisk is this build's xz" "$(cmp -s "$T/u/ramdisk" "$OUT/initramfs-pk5l.cpio.xz" && echo same || echo differs)" same
mkdir "$T/x" && (cd "$T/x" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "the init in the ramdisk is init-pk5l" "$(cmp -s "$T/x/init" "$I" && echo same || echo differs)" same
ck "ramdisk has /init, /bin/busybox, /dev/console" "$(xz -dc "$T/u/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e bin/busybox -e dev/console)" 3
ck "busybox in it is aarch64 static" "$(file -b "$T/x/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
for a in ash mount mkdir dd ls ln cut head tail dmesg uname reboot sleep touch; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$(echo "$a" | tr a-z A-Z)=y" "$W/busybox-arm64.src/.config")" 1
done
ck "init has no bashisms (busybox ash -n)" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
# what the gadget needs from OUR kernel's config (out-ml/.config is the config of pk5's Image.gz: ml6a's kernel)
for k in USB_GADGET USB_CONFIGFS USB_CONFIGFS_ACM USB_CONFIGFS_MASS_STORAGE USB_DWC3 USB_DWC3_QCOM USB_DWC3_DUAL_ROLE PHY_QCOM_QUSB2 QCOM_WDT CONFIGFS_FS DEVTMPFS PSTORE_RAM RD_XZ BLK_DEV_INITRD; do
  ck "kernel config has CONFIG_$k" "$(grep -cE "^CONFIG_$k=[ym]$" "$CFG")" 1
done
ck "size is pk5's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
cp "$IMG" "$T/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
(cd "$OUT" && sha256sum initramfs-pk5l.cpio.xz boot-pk5l.img | tee SHA256SUMS)
[ $total -eq 0 ]
