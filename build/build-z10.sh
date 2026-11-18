#!/bin/sh
# z10 (backlog walleye-pmic-wdt-readback-image): z9 with ONE change, the ramdisk's /init: initramfs-mainline/init-z10 (init-z6 plus /bin/pon-regs: the PON watchdog registers 0x54-0x57, 0x5a, 0x5b, 0xc5 read by the init itself from the regmap debugfs,
# put into the banner as a WALLEYE-BANNER-PON line and answered by the command `pon-regs`). Reads only: no bite is armed, no register is written. Kernel, dtb, header, command line and AVB footer are z9's. Host only; nothing touches the phone.
#   build-z10.sh WORK_DIR Z9_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-z10.img,initramfs-z10.cpio.xz,SHA256SUMS-z10}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
# The first check packs init-z6 and finds z9's ramdisk byte for byte, so the packing is the same and the only difference is the init.
set -eu
W=${1:?usage: build-z10.sh WORK_DIR Z9_BOOT_IMG OUT_DIR}
Z9=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
I=$HERE/initramfs-mainline/init-z10
OLD=$HERE/initramfs-mainline/init-z6
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$Z9" "$I" "$OLD" "$W/busybox-x86_64"; do [ -s "$f" ] || { echo "build-z10: missing $f" >&2; exit 1; }; done
echo "input $(sha256sum "$Z9" | cut -c1-16)  $Z9"
INFO=$(unpack_bootimg --boot_img "$Z9" --out "$T/z9")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$Z9")
AV=$(avbtool info_image --image "$Z9")
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
ck "packing init-z6 gives z9's ramdisk byte for byte (the packing is the same)" "$(cmp -s "$T/old.xz" "$T/z9/ramdisk" && echo same || echo differs)" same
mkdir -p "$T/o"; pack "$I" "$T/o/initramfs-z10.cpio.xz"
IMG=$T/boot-z10.img
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/z9/kernel" --ramdisk "$T/o/initramfs-z10.cpio.xz" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
hdr() { printf '%s\n' "$1" | grep -v -e 'ramdisk size'; }
ck "kernel (Image.gz + dtb) bytes equal z9's" "$(cmp -s "$T/z9/kernel" "$T/u/kernel" && echo same || echo differs)" same
ck "header equals z9's but the ramdisk size" "$(hdr "$U")" "$(hdr "$INFO")"
ck "command line equals z9's" "$(field 'command line args')" "$(printf '%s\n' "$U" | sed -n 's/^command line args: *//p' | head -n1)"
ck "ramdisk differs from z9's (the one change)" "$(cmp -s "$T/z9/ramdisk" "$T/u/ramdisk" && echo same || echo differs)" differs
ck "ramdisk is this build's xz" "$(cmp -s "$T/u/ramdisk" "$T/o/initramfs-z10.cpio.xz" && echo same || echo differs)" same
mkdir "$T/x" "$T/y" && (cd "$T/x" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true) && (cd "$T/y" && xz -dc "$T/z9/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "the init in the ramdisk is init-z10" "$(cmp -s "$T/x/init" "$I" && echo same || echo differs)" same
ck "the busybox in it is z9's" "$(cmp -s "$T/x/bin/busybox" "$T/y/bin/busybox" && echo same || echo differs)" same
ck "ramdisk lists the same files as z9's" "$(xz -dc "$T/u/ramdisk" | cpio -t 2>/dev/null | sort | tr '\n' ' ')" "$(xz -dc "$T/z9/ramdisk" | cpio -t 2>/dev/null | sort | tr '\n' ' ')"
for a in ash mount grep sed tr head cat sleep mv chmod; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$(echo "$a" | tr a-z A-Z)=y" "$W/busybox-arm64.src/.config")" 1
done
ck "init-z10 (busybox ash -n) has no syntax error" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
ck "init-z10 differs from init-z6 only by the z10 block, the PON banner line, the header comment and the step name" "$(diff "$OLD" "$I" | grep '^<' | cut -c1-12 | tr '\n' '|')" "< # z6 (back|< say \"z6 in|"
ck "init-z10 arms no PMIC watchdog and writes no register (no watchdog1, no write into debugfs)" "$(grep -v '^[[:space:]]*#' "$I" | grep -c -e 'watchdog1' -e '> */sys/kernel/debug' -e '> *[^ ]*/registers')" 0
ck "the apps watchdog petter and the 90-beat self-reboot are kept" "$(grep -c -e 'while \[ \$n -lt 90 \]' -e 'self-reboot now' "$I")" 2
ck "the PON reader's host selftest passes" "$(sh "$HERE/../tests/host/pon-regs-selftest.sh" 2>&1 | tail -n1)" "pon-regs-selftest: all ok"
ck "every init of ours writes the serial walleye-first-boot (check-init-serials.sh)" "$(sh "$HERE/check-init-serials.sh" "$HERE" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "size is z9's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "AVB info equals z9's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z10: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z10.img"; cp "$T/o/initramfs-z10.cpio.xz" "$OUT/"
(cd "$OUT" && sha256sum initramfs-z10.cpio.xz boot-z10.img | tee SHA256SUMS-z10)
