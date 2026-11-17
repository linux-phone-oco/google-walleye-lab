#!/bin/sh
# vendor-gadget-m1 (backlog walleye-massstorage-window): the stock control's boot.img with only the ramdisk swapped (xz), init = initramfs-vendor5/init-m1: a mass_storage gadget whose LUN is a 2 MiB
# file in the ramdisk, rewritten every 5 s with kmsg, dmesg tail, cmdline, mounts (the host reads it raw: tests/host/massstorage-read.sh). Existing images and inits stay byte for byte. Host only; nothing touches the phone.
#
#   build-vendor-gadget-m.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [VENDOR4_RAMDISK]     (WORK_DIR as build-vendor-gadget4.sh)
# Writes OUT_DIR/initramfs-vendor-m1.cpio.xz and OUT_DIR/vendor-gadget-m1.img, checks each (ok/FAIL, exit 1 on a FAIL), prints the sha256s.
set -eu
W=${1:?usage: build-vendor-gadget-m.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [VENDOR4_RAMDISK]}
CTRL=${2:?}
OUT=${3:?}
V4=${4:-}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
[ -x "$GEN" ] || make -C "$W/linux-v7.3" ARCH=arm64 LLVM=1 O="$W/out/b" usr/gen_init_cpio >/dev/null
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
P=$T/p
INFO=$(unpack_bootimg --boot_img "$CTRL" --out "$P")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
[ "$(field 'boot image header version')" = 0 ] || { echo "build-vendor-gadget-m: the control is not header v0" >&2; exit 1; }
PSIZE=$(stat -c %s "$CTRL")
AV=$(avbtool info_image --image "$CTRL")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
h() { sha256sum "$1" | cut -c1-16; }
hdr() { echo "$1" | grep -v -e '^boot magic' -e 'ramdisk size'; }
total=0
for S in m1; do
  I=$HERE/initramfs-vendor5/init-$S
  SPEC=$T/vendor$S.spec
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    # no devtmpfs on the vendor kernel: every node the init needs comes from the cpio
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"; echo "nod /dev/zero 666 0 0 c 1 5"
    echo "nod /dev/kmsg 600 0 0 c 1 11";   echo "nod /dev/tty 666 0 0 c 5 0";    echo "nod /dev/ptmx 666 0 0 c 5 2"
    echo "nod /dev/urandom 444 0 0 c 1 9"
    echo "file /bin/busybox $BB 755 0 0"
    echo "slink /bin/sh busybox 777 0 0"
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
  ck "ramdisk has /init, /bin/busybox, /dev/console, /dev/ptmx, /dev/kmsg" "$(xz -dc "$T/u$S/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e bin/busybox -e dev/console -e dev/ptmx -e dev/kmsg)" 5
  mkdir "$T/x$S" && (cd "$T/x$S" && xz -dc "$T/u$S/ramdisk" | cpio -id --quiet 2>/dev/null || true)
  ck "busybox in it is aarch64 static" "$(file -b "$T/x$S/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
  ck "size is the control's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  mkdir "$T/v$S" && cp "$IMG" "$T/v$S/boot.img"
  ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v$S/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "the init in the ramdisk is the source init" "$(cmp -s "$T/x$S/init" "$I" && echo same || echo differs)" same
  ck "init has no 'set -e' and ends in a sleep loop" "$(grep -c -e '^set -e' "$I")$(grep -c '^while :; do sleep 3600; done$' "$I")" 01
  ck "init reboots at 180 s (36 rounds of sleep 5), reboot -f, then sysrq b" "$(grep -c -e 'while \[ \$n -lt 36 \]; do' -e '^  sleep 5$' -e 'reboot -f; sleep 5; echo b > /proc/sysrq-trigger' "$I")" 3
  ck "init writes its kmsg marker before the first mount" "$(grep -n -e 'init start (before any mount)' -e '^mount ' "$I" | head -n1 | grep -c start)" 1
  # exactly the parts m1 has (1 = present): configfs mount, mass_storage function, config dir, symlink, UDC bind; none of ncm, telnetd, udhcpd, ifconfig
  got=$(for pat in '^mount -t configfs' '^mkdir -p \$G/functions/mass_storage.0' '^mkdir -p \$G/configs/c.1' '^ln -s' '> \$G/UDC' 'ncm|telnetd|udhcpd|ifconfig|dev_addr|host_addr'; do [ "$(grep -v '^#' "$I" | grep -c -E -e "$pat")" -ge 1 ] && printf 1 || printf 0; done)
  ck "init has exactly this step's parts (configfs, mass_storage function, config dir, symlink, UDC, no ncm/net)" "$got" 111110
  ck "init says what it is about to do before each configfs step (8 say lines)" "$(grep -c "step m1: about to" "$I")" 8
  ck "init writes only the attributes file, ro, removable, cdrom, nofua of the lun" "$(grep -o -E 'for a in [a-z ]+' "$I" | tr '\n' ' ')" "for a in ro removable for a in cdrom nofua "
  ck "init has the lun read-only for the host (ro=1) and a 2 MiB file" "$(grep -c -e 'echo 1 > \$L/\$a' -e 'bs=1024 count=2048' "$I")" 2
  ck "init keeps the window to 64 KiB, padded (no stale tail)" "$(grep -c 'bs=65536 count=1 conv=sync,notrunc' "$I")" 1
  if [ -n "$V4" ]; then mkdir "$T/r4$S" && (cd "$T/r4$S" && xz -dc "$V4" | cpio -id --quiet 2>/dev/null || true); ck "ramdisk differs from vendor-gadget4's only in /init" "$(cd "$T/x$S" && find . -type f ! -name init -exec sha256sum {} + | sort)" "$(cd "$T/r4$S" && find . -type f ! -name init -exec sha256sum {} + | sort)"; fi
  echo "vendor-gadget-$S: $pass ok, $fail failed"
  total=$((total+fail))
done
(cd "$OUT" && sha256sum initramfs-vendor-m1.cpio.xz vendor-gadget-m1.img)
[ $total -eq 0 ]
