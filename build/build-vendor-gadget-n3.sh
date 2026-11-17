#!/bin/sh
# vendor-gadget-n3 (backlog walleye-ffs-helper-build): the FunctionFS ACM console. Same image as vendor-gadget-n1/n2 (the stock control's boot.img, only the ramdisk swapped, xz); the ramdisk adds the static aarch64
# helper /bin/ffs-acm (ffs-acm/ffs-acm.c, built here with aarch64-linux-gnu-gcc -static) and the init initramfs-vendor5/init-n3. Existing images and inits stay byte for byte. Host only; nothing touches the phone.
#
#   build-vendor-gadget-n3.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [VENDOR4_RAMDISK]     (WORK_DIR as build-vendor-gadget4.sh)
# Writes OUT_DIR/ffs-acm, OUT_DIR/initramfs-vendor-n3.cpio.xz and OUT_DIR/vendor-gadget-n3.img, checks them (ok/FAIL, exit 1 on a FAIL), prints the sha256s.
set -eu
W=${1:?usage: build-vendor-gadget-n3.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [VENDOR4_RAMDISK]}
CTRL=${2:?}
OUT=${3:?}
V4=${4:-}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
[ -x "$GEN" ] || make -C "$W/linux-v7.3" ARCH=arm64 LLVM=1 O="$W/out/b" usr/gen_init_cpio >/dev/null
mkdir -p "$OUT"
aarch64-linux-gnu-gcc -static -Os -Wall -Wextra -Werror -o "$OUT/ffs-acm" "$HERE/ffs-acm/ffs-acm.c"
aarch64-linux-gnu-strip "$OUT/ffs-acm"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
P=$T/p
INFO=$(unpack_bootimg --boot_img "$CTRL" --out "$P")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
[ "$(field 'boot image header version')" = 0 ] || { echo "build-vendor-gadget-n3: the control is not header v0" >&2; exit 1; }
PSIZE=$(stat -c %s "$CTRL")
AV=$(avbtool info_image --image "$CTRL")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
h() { sha256sum "$1" | cut -c1-16; }
hdr() { echo "$1" | grep -v -e '^boot magic' -e 'ramdisk size'; }
S=n3
I=$HERE/initramfs-vendor5/init-$S
SPEC=$T/vendor$S.spec
{
  for d in /bin /sbin /etc /dev /dev/ffs /dev/pts /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  # no devtmpfs on the vendor kernel: every node the init needs comes from the cpio
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"; echo "nod /dev/zero 666 0 0 c 1 5"
  echo "nod /dev/kmsg 600 0 0 c 1 11";   echo "nod /dev/tty 666 0 0 c 5 0";    echo "nod /dev/ptmx 666 0 0 c 5 2"
  echo "nod /dev/urandom 444 0 0 c 1 9"
  echo "file /bin/busybox $BB 755 0 0"
  echo "slink /bin/sh busybox 777 0 0"
  echo "file /bin/ffs-acm $OUT/ffs-acm 755 0 0"
  echo "file /init $I 755 0 0"
} > "$SPEC"
"$GEN" -t 0 "$SPEC" > "$T/rd$S.cpio"
xz --check=crc32 --lzma2=preset=6e -c "$T/rd$S.cpio" > "$OUT/initramfs-vendor-$S.cpio.xz"
IMG=$OUT/vendor-gadget-$S.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$P/kernel" --ramdisk "$OUT/initramfs-vendor-$S.cpio.xz" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
pass=0; fail=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $S $1"; pass=$((pass+1)); else echo "FAIL $S $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$S")
ck "header equals the control's (offsets, page, os word, command line)" "$(hdr "$U")" "$(hdr "$INFO")"
ck "kernel blob is the control's" "$(h "$T/u$S/kernel")" "$(h "$P/kernel")"
ck "ramdisk is this step's xz" "$(h "$T/u$S/ramdisk")" "$(h "$OUT/initramfs-vendor-$S.cpio.xz")"
ck "ramdisk is xz with CRC32 as the control's" "$(file -b "$T/u$S/ramdisk" | cut -d, -f1-2)" "$(file -b "$P/ramdisk" | cut -d, -f1-2)"
ck "ramdisk has /init, /bin/busybox, /bin/ffs-acm, /dev/console, /dev/ptmx, /dev/kmsg, /dev/ffs" "$(xz -dc "$T/u$S/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e bin/busybox -e bin/ffs-acm -e dev/console -e dev/ptmx -e dev/kmsg -e dev/ffs)" 7
mkdir "$T/x$S" && (cd "$T/x$S" && xz -dc "$T/u$S/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "busybox in it is aarch64 static" "$(file -b "$T/x$S/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
ck "ffs-acm in it is aarch64 static" "$(file -b "$T/x$S/bin/ffs-acm" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
ck "ffs-acm in it is the one built here" "$(cmp -s "$T/x$S/bin/ffs-acm" "$OUT/ffs-acm" && echo same || echo differs)" same
ck "ffs-acm is statically linked (no interpreter)" "$(file -b "$T/x$S/bin/ffs-acm" | grep -c 'statically linked')" 1
ck "ffs-acm has no python or dynamic needs (readelf: no NEEDED)" "$(aarch64-linux-gnu-readelf -d "$OUT/ffs-acm" 2>/dev/null | grep -c NEEDED || true)" 0
ck "size is the control's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
mkdir "$T/v$S" && cp "$IMG" "$T/v$S/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v$S/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the init in the ramdisk is the source init" "$(cmp -s "$T/x$S/init" "$I" && echo same || echo differs)" same
ck "init has no 'set -e' and ends in a sleep loop" "$(grep -c -e '^set -e' "$I")$(grep -c '^while :; do sleep 3600; done$' "$I")" 01
ck "init reboots at 180 s (6 x sleep 30), reboot -f, then sysrq b" "$(grep -c -e 'while \[ \$n -lt 6 \]; do sleep 30;' -e 'reboot -f; sleep 5; echo b > /proc/sysrq-trigger' "$I")" 2
ck "init writes its kmsg marker before the first mount" "$(grep -n -e 'init start (before any mount)' -e '^mount ' "$I" | head -n1 | grep -c start)" 1
ck "init makes the ffs.acm function, a config symlink, mounts functionfs, starts the helper" "$(for pat in '^mkdir -p \$G/functions/ffs.acm' '^ln -s' '^mount -t functionfs acm /dev/ffs' '^/bin/ffs-acm &'; do printf %s "$(grep -c -e "$pat" "$I")"; done)" 1111
ck "init itself never writes UDC (the helper does, after ep0)" "$(grep -c '> \$G/UDC' "$I" || true)" 0
ck "init writes no function setting (dev_addr, host_addr, qmult, ifname)" "$(grep -c -E 'dev_addr|host_addr|qmult|ifname' "$I" || true)" 0
ck "helper writes the UDC after the descriptors and strings (source order)" "$(grep -n -e 'build_descriptors();' -e 'build_strings();' -e 'UDC_FILE, O_WRONLY' "$HERE/ffs-acm/ffs-acm.c" | cut -d: -f1 | sort -c -n 2>&1 | wc -l)" 0
if [ -n "$V4" ]; then mkdir "$T/r4$S" && (cd "$T/r4$S" && xz -dc "$V4" | cpio -id --quiet 2>/dev/null || true); ck "ramdisk differs from vendor-gadget4's only in /init, /bin/ffs-acm, /dev/ffs" "$(cd "$T/x$S" && find . -type f ! -name init ! -name ffs-acm -exec sha256sum {} + | sort)" "$(cd "$T/r4$S" && find . -type f ! -name init -exec sha256sum {} + | sort)"; fi
echo "vendor-gadget-$S: $pass ok, $fail failed"
(cd "$OUT" && sha256sum ffs-acm initramfs-vendor-n3.cpio.xz vendor-gadget-n3.img)
[ "$fail" -eq 0 ]
