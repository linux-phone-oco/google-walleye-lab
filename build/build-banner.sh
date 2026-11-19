#!/bin/sh
# banner (backlog walleye-build-banner-init): y2's image (our kernel, our dtb, y2's header, command line and AVB footer) with ONLY the ramdisk changed: its /init is
# initramfs-mainline/init-banner, a one-function ACM gadget that writes a banner to /dev/ttyGS0 every 2 s from the moment the tty exists and a kmsg marker per step. Host only.
# BANNER_NAME (default banner) names the outputs; every init of ours must write the serial walleye-first-boot or the build stops.
#   build-banner.sh WORK_DIR Y2_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-$NAME.img,initramfs-$NAME.cpio.xz,SHA256SUMS-$NAME}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
set -eu
W=${1:?usage: build-banner.sh WORK_DIR Y2_BOOT_IMG OUT_DIR}
Y2=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
CFG=${CFG:-$W/out-ml/.config}
NAME=${BANNER_NAME:-banner}
case $NAME in ""|*[!A-Za-z0-9._-]*) echo "build-banner: BANNER_NAME must be [A-Za-z0-9._-]+" >&2; exit 1;; esac
I=$HERE/initramfs-mainline/init-banner
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$Y2" "$I" "$CFG" "$W/busybox-x86_64"; do [ -s "$f" ] || { echo "build-banner: missing $f" >&2; exit 1; }; done
echo "input $(sha256sum "$Y2" | cut -c1-16)  $Y2"
INFO=$(unpack_bootimg --boot_img "$Y2" --out "$T/p")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$Y2")
AV=$(avbtool info_image --image "$Y2")
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
} > "$T/banner.spec"
"$GEN" -t 0 "$T/banner.spec" > "$T/rd.cpio"
xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$T/initramfs-$NAME.cpio.xz"
IMG=$T/boot-$NAME.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/p/kernel" --ramdisk "$T/initramfs-$NAME.cpio.xz" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
hdr() { printf "%s\n" "$1" | grep -v -e '^ramdisk size' -e '^ramdisk_size'; }
ck "kernel (Image.gz + dtb) bytes equal y2's" "$(cmp -s "$T/p/kernel" "$T/u/kernel" && echo same || echo differs)" same
ck "header equals y2's but the ramdisk size" "$(hdr "$U")" "$(hdr "$INFO")"
ck "command line equals y2's" "$(printf "%s\n" "$U" | sed -n 's/^command line args: *//p' | head -n1)" "$(field 'command line args')"
ck "ramdisk differs from y2's (the one change)" "$(cmp -s "$T/p/ramdisk" "$T/u/ramdisk" && echo same || echo differs)" differs
ck "ramdisk is this build's xz" "$(cmp -s "$T/u/ramdisk" "$T/initramfs-$NAME.cpio.xz" && echo same || echo differs)" same
mkdir "$T/x" && (cd "$T/x" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "the init in the ramdisk is init-banner" "$(cmp -s "$T/x/init" "$I" && echo same || echo differs)" same
ck "ramdisk has /init, /bin/busybox, /dev/console" "$(xz -dc "$T/u/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e bin/busybox -e dev/console)" 3
ck "busybox in it is aarch64 static" "$(file -b "$T/x/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
for a in ash mount mkdir ls ln cut head tail wc uname reboot sync sleep true; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$(echo "$a" | tr a-z A-Z)=y" "$W/busybox-arm64.src/.config")" 1
done
ck "init puts the banner on ttyGS0 and each step on kmsg (text present)" "$(grep -c -e "> /dev/ttyGS0" -e "> /dev/kmsg" "$I" | awk "{print (\$1>=3)?\"yes\":\"no\"}")" yes
ck "every init of ours writes the serial walleye-first-boot (check-init-serials.sh)" "$(sh "$HERE/check-init-serials.sh" "$HERE" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "init parses under busybox ash (ash -n)" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
for k in USB_GADGET USB_CONFIGFS USB_CONFIGFS_ACM USB_DWC3 USB_DWC3_QCOM PHY_QCOM_QUSB2 QCOM_WDT WATCHDOG PRINTK CONFIGFS_FS DEVTMPFS RD_XZ BLK_DEV_INITRD; do
  ck "kernel config has CONFIG_$k" "$(grep -cE "^CONFIG_$k=y$" "$CFG")" 1
done
ck "size is y2's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "AVB info equals y2's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-banner: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-$NAME.img"; cp "$T/initramfs-$NAME.cpio.xz" "$OUT/"
(cd "$OUT" && sha256sum initramfs-$NAME.cpio.xz boot-$NAME.img | tee SHA256SUMS-$NAME)
