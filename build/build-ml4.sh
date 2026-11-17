#!/bin/sh
# ml4 (backlog walleye-ml4-timing-channel): ml3 with ONLY the ramdisk's /init changed (the timing-channel init). Host only; nothing touches the phone.
#
#   build-ml4.sh WORK_DIR ML3_BOOT_IMG OUT_DIR
#     WORK_DIR   as build-mainline-light.sh (busybox-arm64, out/b/usr/gen_init_cpio)
# Writes OUT_DIR/{boot-ml4.img,initramfs-ml4.cpio.xz,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL) that kernel, dtb and header bytes equal ml3's.
set -eu
W=${1:?usage: build-ml4.sh WORK_DIR ML3_BOOT_IMG OUT_DIR}
ML3=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
I=$HERE/initramfs-mainline/init-ml4
for f in "$GEN" "$BB" "$ML3" "$I"; do [ -s "$f" ] || { echo "build-ml4: missing $f" >&2; exit 1; }; done
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
INFO=$(unpack_bootimg --boot_img "$ML3" --out "$T/p")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$ML3")
AV=$(avbtool info_image --image "$ML3")
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
} > "$T/ml4.spec"
"$GEN" -t 0 "$T/ml4.spec" > "$T/rd.cpio"
xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$OUT/initramfs-ml4.cpio.xz"
IMG=$OUT/boot-ml4.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/p/kernel" --ramdisk "$OUT/initramfs-ml4.cpio.xz" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
hdr() { echo "$1" | grep -v -e 'ramdisk size'; }
ck "kernel (Image.gz + dtb) bytes equal ml3's" "$(cmp -s "$T/p/kernel" "$T/u/kernel" && echo same || echo differs)" same
ck "header (offsets, page, os word, command line, kernel size) equals ml3's but the ramdisk size" "$(hdr "$U")" "$(hdr "$INFO")"
ck "ramdisk differs from ml3's (the one change)" "$(cmp -s "$T/p/ramdisk" "$T/u/ramdisk" && echo same || echo differs)" differs
ck "ramdisk is this build's xz" "$(cmp -s "$T/u/ramdisk" "$OUT/initramfs-ml4.cpio.xz" && echo same || echo differs)" same
mkdir "$T/x" && (cd "$T/x" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "the init in the ramdisk is the source init-ml4" "$(cmp -s "$T/x/init" "$I" && echo same || echo differs)" same
ck "ramdisk has /init, /bin/busybox, /dev/console" "$(xz -dc "$T/u/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e bin/busybox -e dev/console)" 3
ck "busybox in it is aarch64 static" "$(file -b "$T/x/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
ck "init has no bashisms (busybox ash -n)" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
ck "size is ml3's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
cp "$IMG" "$T/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
(cd "$OUT" && sha256sum initramfs-ml4.cpio.xz boot-ml4.img | tee SHA256SUMS)
[ $total -eq 0 ]
