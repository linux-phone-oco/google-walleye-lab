#!/bin/sh
# z4 (backlog walleye-shell-reliable): z3 with ONE change, the ramdisk's /init: initramfs-mainline/init-z4 (the ACM shell keeps the port open, raw, no echo, no hangup,
# has a controlling terminal, and respawns at once) instead of init-pk5l. Kernel, dtb, header, command line and AVB footer are z3's. Host only; nothing touches the phone.
#   build-z4.sh WORK_DIR Z3_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-z4.img,initramfs-z4.cpio.xz,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
# The ramdisk is packed the way build-pk5l.sh packs it; the first check packs it with init-pk5l and finds z3's ramdisk byte for byte, so the only difference is the init.
set -eu
W=${1:?usage: build-z4.sh WORK_DIR Z3_BOOT_IMG OUT_DIR}
Z3=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
I=$HERE/initramfs-mainline/init-z4
OLD=$HERE/initramfs-mainline/init-pk5l
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$Z3" "$I" "$OLD" "$W/busybox-x86_64"; do [ -s "$f" ] || { echo "build-z4: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$Z3" --out "$T/z3")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$Z3")
AV=$(avbtool info_image --image "$Z3")
pack() { # INIT OUT_XZ
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
    echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $1 755 0 0"
  } > "$T/spec"
  "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"
  xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$2"
}
pack "$OLD" "$T/old.xz"
ck "packing init-pk5l gives z3's ramdisk byte for byte (the packing is the same)" "$(cmp -s "$T/old.xz" "$T/z3/ramdisk" && echo same || echo differs)" same
mkdir -p "$T/o"; pack "$I" "$T/o/initramfs-z4.cpio.xz"
IMG=$T/boot-z4.img
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/z3/kernel" --ramdisk "$T/o/initramfs-z4.cpio.xz" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
hdr() { printf '%s\n' "$1" | grep -v -e 'ramdisk size'; }
ck "kernel (Image.gz + dtb) bytes equal z3's" "$(cmp -s "$T/z3/kernel" "$T/u/kernel" && echo same || echo differs)" same
ck "header equals z3's but the ramdisk size" "$(hdr "$U")" "$(hdr "$INFO")"
ck "command line equals z3's" "$(field 'command line args')" "$(printf '%s\n' "$U" | sed -n 's/^command line args: *//p' | head -n1)"
ck "ramdisk differs from z3's (the one change)" "$(cmp -s "$T/z3/ramdisk" "$T/u/ramdisk" && echo same || echo differs)" differs
ck "ramdisk is this build's xz" "$(cmp -s "$T/u/ramdisk" "$T/o/initramfs-z4.cpio.xz" && echo same || echo differs)" same
mkdir "$T/x" "$T/y" && (cd "$T/x" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true) && (cd "$T/y" && xz -dc "$T/z3/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "the init in the ramdisk is init-z4" "$(cmp -s "$T/x/init" "$I" && echo same || echo differs)" same
ck "the busybox in it is z3's" "$(cmp -s "$T/x/bin/busybox" "$T/y/bin/busybox" && echo same || echo differs)" same
ck "ramdisk lists the same files as z3's" "$(xz -dc "$T/u/ramdisk" | cpio -t 2>/dev/null | sort | tr '\n' ' ')" "$(xz -dc "$T/z3/ramdisk" | cpio -t 2>/dev/null | sort | tr '\n' ' ')"
ck "busybox in it is aarch64 static" "$(file -b "$T/x/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
for a in ash stty setsid cut tr sleep; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$(echo "$a" | tr a-z A-Z)=y" "$W/busybox-arm64.src/.config")" 1
done
ck "init has no bashisms (busybox ash -n)" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
ck "init sets the tty raw, no echo, no hangup" "$(grep -c 'stty -F /dev/ttyGS0 raw -echo -hupcl' "$I")" 1
ck "init holds the port open once per shell (command exec 4<>, a failed open retries)" "$(grep -c 'command exec 4<> /dev/ttyGS0 .. { sleep 1; continue; }' "$I")" 1
ck "init runs the shell in its own session on fd 4 (setsid, no cttyhack)" "$(grep -c 'setsid busybox ash -i <&4 >&4 2>&4' "$I")" 1
ck "init says every shell start in the kmsg with a counter" "$(grep -c 'shell on /dev/ttyGS0, start \$n' "$I")" 1
ck "the old per-open shell line is gone" "$(grep -c 'ash -i < /dev/ttyGS0' "$I")" 0
nocon() { sed -e '/^# the console/,/^) &$/d' -e '/^#/d' "$1"; }
ck "init outside the console block (comments aside) is init-pk5l's, byte for byte" "$(nocon "$OLD" | sha256sum | cut -c1-16)" "$(nocon "$I" | sha256sum | cut -c1-16)"
ck "size is z3's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "AVB info equals z3's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z4: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z4.img"; cp "$T/o/initramfs-z4.cpio.xz" "$OUT/"
(cd "$OUT" && sha256sum initramfs-z4.cpio.xz boot-z4.img | tee SHA256SUMS)
