#!/bin/sh
# vendor-gadget4: the stock control's boot.img with only the ramdisk swapped, the ramdisk made for the vendor 4.4.302 kernel
# (research walleye-vendor-kernel-facts-s1-panic; its config: RD_GZIP not set, RD_XZ y, DEVTMPFS not set, NCM only).
# v1 (vendor-gadget.img) had a gzip ramdisk: that kernel cannot unpack it, so it panicked. Host only; nothing touches the phone.
#
#   build-vendor-gadget4.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR     (WORK_DIR as build-initramfs.sh: busybox-arm64, out/b)
# Writes OUT_DIR/initramfs-vendor4.cpio.xz (xz, CRC32 as the stock ramdisk) and OUT_DIR/vendor-gadget4.img (header, kernel blob,
# command line and AVB footer of the control; only the ramdisk differs), and checks both: prints ok/FAIL, exits 1 on a FAIL.
set -eu
W=${1:?usage: build-vendor-gadget4.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR}
CTRL=${2:?}
OUT=${3:?}
V2=${4:-}   # optional: vendor-gadget3's initramfs-vendor3.cpio.xz, to prove only /init (and the unused udhcpd.conf) differ
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
[ -x "$GEN" ] || make -C "$W/linux-v7.3" ARCH=arm64 LLVM=1 O="$W/out/b" usr/gen_init_cpio >/dev/null
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
SPEC=$T/vendor.spec
{
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  # no devtmpfs on the vendor kernel: every node the init and telnetd need comes from the cpio
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"; echo "nod /dev/zero 666 0 0 c 1 5"
  echo "nod /dev/kmsg 600 0 0 c 1 11";   echo "nod /dev/tty 666 0 0 c 5 0";    echo "nod /dev/ptmx 666 0 0 c 5 2"
  echo "nod /dev/urandom 444 0 0 c 1 9"
  echo "file /bin/busybox $BB 755 0 0"
  echo "slink /bin/sh busybox 777 0 0"
  echo "file /init $HERE/initramfs-vendor4/init 755 0 0"
} > "$SPEC"
"$GEN" -t 0 "$SPEC" > "$T/rd.cpio"
xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$OUT/initramfs-vendor4.cpio.xz"
# the control, unpacked: its own header fields, kernel blob and AVB footer properties
P=$T/p
INFO=$(unpack_bootimg --boot_img "$CTRL" --out "$P")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
[ "$(field 'boot image header version')" = 0 ] || { echo "build-vendor-gadget: the control is not header v0" >&2; exit 1; }
PSIZE=$(stat -c %s "$CTRL")
AV=$(avbtool info_image --image "$CTRL")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$OUT/vendor-gadget4.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$P/kernel" --ramdisk "$OUT/initramfs-vendor4.cpio.xz" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
# the checks
pass=0; fail=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
hdr() { echo "$1" | grep -v -e '^boot magic' -e 'ramdisk size'; }
ck "header equals the control's (offsets, page, os word, command line)" "$(hdr "$U")" "$(hdr "$INFO")"
ck "kernel blob (lz4 frame + dtb) is the control's" "$(h "$T/u/kernel")" "$(h "$P/kernel")"
ck "ramdisk is vendor-gadget4's xz, not the control's" "$(h "$T/u/ramdisk")" "$(h "$OUT/initramfs-vendor4.cpio.xz")"
ck "ramdisk is xz with CRC32 (the only format the kernel has besides none)" "$(file -b "$T/u/ramdisk" | cut -d, -f1-2)" "$(file -b "$P/ramdisk" | cut -d, -f1-2)"
ck "ramdisk unpacks and has /init, /bin/busybox, /dev/console, /dev/ptmx" "$(xz -dc "$T/u/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e bin/busybox -e dev/console -e dev/ptmx)" 4
ck "busybox in it is aarch64 static" "$(xz -dc "$T/u/ramdisk" | (cd "$T" && mkdir x && cd x && cpio -id --quiet bin/busybox && file -b bin/busybox | cut -d, -f1-2))" "ELF 64-bit LSB executable, ARM aarch64"
ck "size is the control's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
mkdir -p "$T/v"; cp "$IMG" "$T/v/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
mkdir "$T/r3" && (cd "$T/r3" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "the init in the ramdisk is the source init" "$(cmp -s "$T/r3/init" "$HERE/initramfs-vendor4/init" && echo same || echo differs)" same
I=$HERE/initramfs-vendor4/init
ck "init has no 'set -e' and ends in a sleep loop" "$(grep -c -e '^set -e' "$I")$(grep -c '^while :; do sleep 3600; done$' "$I")" 01
ck "init has no gadget, USB or network (no configfs, udc, ifconfig, telnetd, udhcpd)" "$(grep -c -e configfs -e usb_gadget -e udc -e ifconfig -e telnetd -e udhcpd "$I")" 0
ck "init writes its kmsg marker before the first mount, and the cpio has the kmsg node" "$(grep -n -e 'idle init start (before any mount)' -e '^mount ' "$I" | head -n1 | grep -c start)$(xz -dc "$T/u/ramdisk" | cpio -tv 2>/dev/null | grep -c ' dev/kmsg')" 11
ck "init reboots at 120 s (4 x sleep 30), reboot -f, then sysrq b" "$(grep -c -e 'while \[ \$n -lt 4 \]; do sleep 30;' -e 'reboot -f; sleep 5; echo b > /proc/sysrq-trigger' "$I")" 2
if [ -n "$V2" ]; then mkdir "$T/r2" && (cd "$T/r2" && xz -dc "$V2" | cpio -id --quiet 2>/dev/null || true); ck "ramdisk differs from vendor-gadget3's only in /init and etc/udhcpd.conf" "$(cd "$T/r3" && find . -type f ! -name init ! -name udhcpd.conf -exec sha256sum {} + | sort)" "$(cd "$T/r2" && find . -type f ! -name init ! -name udhcpd.conf -exec sha256sum {} + | sort)"; fi
echo "vendor-gadget4: $pass ok, $fail failed"
sha256sum "$OUT/initramfs-vendor4.cpio.xz" "$IMG"
[ $fail -eq 0 ]
