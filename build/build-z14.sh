#!/bin/sh
# z14 (backlog walleye-usb-net): z7 with ONE change, the ramdisk: init-z14 (z6's init + an ECM function beside the ACM console, usb0 172.16.43.1, one DHCP lease, telnetd) and /etc/udhcpd.conf (172.16.43.2).
# Kernel Image.gz, dtb, header, command line and AVB footer are z7's, byte for byte (z7's kernel carries ECM and RNDIS: checked below). Built only - no phone run; the keeper tries it once (tests/host/gadget-up.sh).
#   build-z14.sh WORK_DIR Z7_BOOT_IMG OUT_DIR
# WORK_DIR holds busybox-arm64, busybox-x86_64 (+ busybox-arm64.src) and out/b/usr/gen_init_cpio (build-kernel.sh). Writes OUT_DIR/{boot-z14.img,initramfs-z14.cpio.xz,SHA256SUMS-z14}; checks (ok/FAIL, exit 1, nothing written).
set -eu
W=${1:?usage: build-z14.sh WORK_DIR Z7_BOOT_IMG OUT_DIR}
Z7=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
I=$HERE/build/initramfs-mainline/init-z14
OLD=$HERE/build/initramfs-mainline/init-z6
CONF=$HERE/build/initramfs-mainline/udhcpd-z14.conf
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$Z7" "$I" "$OLD" "$CONF" "$W/busybox-x86_64" "$W/busybox-arm64.src/.config"; do [ -s "$f" ] || { echo "build-z14: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z14: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
INFO=$(unpack_bootimg --boot_img "$Z7" --out "$T/z7")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z7/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z7")
AV=$(avbtool info_image --image "$Z7")
# the kernel carries what the gadget needs
ck "z7's kernel holds the CDC Ethernet Control Model (ECM) function" "$(zcat "$T/k.gz" | grep -ac 'Control Model (ECM)')" 1
# the busybox has the applets the network block runs
for a in IFCONFIG UDHCPD TELNETD; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$a=y" "$W/busybox-arm64.src/.config")" 1
done
# the init
ck "init-z14 (busybox ash -n) has no syntax error" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
ck "init-z14 differs from init-z6 only by the header, the step name, the ECM function and its link, and the network block" "$(diff "$OLD" "$I" | grep '^<' | cut -c1-14 | tr '\n' '|')" '< say "z6 init|< mkdir -p $G/|< echo "ACM" >|'
ck "init-z14 binds the ECM function in the one configuration with ACM" "$(grep -v '^[[:space:]]*#' "$I" | grep -c -e 'ln -s \$G/functions/ecm.usb0 \$G/configs/c.1/' -e 'ln -s \$G/functions/acm.GS0 \$G/configs/c.1/')" 2
ck "init-z14 puts usb0 on the manifest's address 172.16.43.1 and nothing on 172.16.42" "$(grep -v '^[[:space:]]*#' "$I" | grep -c '172\.16\.43\.1')/$(grep -v '^[[:space:]]*#' "$I" | grep -c '172\.16\.42')" 3/0
ck "init-z14 starts telnetd only on 172.16.43.1 (never all addresses)" "$(grep -c '^    telnetd -l /bin/sh -b 172.16.43.1 ' "$I")" 1
ck "the manifest's address is 172.16.43.1" "$(sed -n 's/^addresses *= *\["\([^"]*\)".*/\1/p' "$HERE/device.toml")" 172.16.43.1
ck "udhcpd.conf leases exactly 172.16.43.2 on usb0" "$(grep -c -e '^start 172.16.43.2$' -e '^end 172.16.43.2$' -e '^interface usb0$' "$CONF")" 3
ck "init-z14 keeps the watchdog petter, the self-reboot after 90 beats and the serial walleye-first-boot" "$(grep -c -e 'while \[ \$n -lt 90 \]' -e 'self-reboot now' -e 'walleye-first-boot' "$I")" 3
ck "init-z14 mounts no disk and writes to none" "$(grep -v '^[[:space:]]*#' "$I" | grep -c -e 'mount[^|]* /dev/sd' -e '> */dev/sd' -e 'dd [^|]*of=/dev' -e 'mkfs' -e 'blkdiscard')" 0
ck "every init of ours writes the serial walleye-first-boot (check-init-serials.sh)" "$(sh "$HERE/build/check-init-serials.sh" "$HERE/build" >/dev/null 2>&1 && echo yes || echo no)" yes
# the ramdisk
pack() { # INIT OUT_XZ [WITH_CONF]
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
    echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $1 755 0 0"
    [ -z "${3:-}" ] || echo "file /etc/udhcpd.conf $3 644 0 0"
  } > "$T/spec"
  "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"
  xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$2"
}
pack "$OLD" "$T/old.xz"
ck "packing init-z6 gives z7's ramdisk byte for byte (the packing is the same)" "$(cmp -s "$T/old.xz" "$T/z7/ramdisk" && echo same || echo differs)" same
mkdir -p "$T/o"; pack "$I" "$T/o/initramfs-z14.cpio.xz" "$CONF"
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z14.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/z7/kernel" --ramdisk "$T/o/initramfs-z14.cpio.xz" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
hdr() { printf '%s\n' "$1" | grep -v -e 'ramdisk size' -e 'kernel_size' -e 'kernel size'; }
ck "z14 kernel (Image.gz + dtb) is z7's, byte for byte" "$(cmp -s "$T/u/kernel" "$T/z7/kernel" && echo same || echo differs)" same
ck "z14 header equals z7's but the sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z14 ramdisk is this build's xz" "$(cmp -s "$T/u/ramdisk" "$T/o/initramfs-z14.cpio.xz" && echo same || echo differs)" same
ck "z14 ramdisk differs from z7's (the init change)" "$(cmp -s "$T/u/ramdisk" "$T/z7/ramdisk" && echo same || echo differs)" differs
mkdir "$T/x" "$T/y" && (cd "$T/x" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true) && (cd "$T/y" && xz -dc "$T/z7/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "the init in the ramdisk is init-z14" "$(cmp -s "$T/x/init" "$I" && echo same || echo differs)" same
ck "the busybox in it is z7's" "$(cmp -s "$T/x/bin/busybox" "$T/y/bin/busybox" && echo same || echo differs)" same
ck "the ramdisk's files are z7's plus etc/udhcpd.conf" "$(xz -dc "$T/u/ramdisk" | cpio -t 2>/dev/null | sort | tr '\n' ' ')" "$( { xz -dc "$T/z7/ramdisk" | cpio -t 2>/dev/null; echo etc/udhcpd.conf; } | sort | tr '\n' ' ')"
ck "z14 size is z7's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z14 AVB info equals z7's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z14 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z14: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z14.img"; cp "$T/o/initramfs-z14.cpio.xz" "$OUT/"
(cd "$OUT" && sha256sum initramfs-z14.cpio.xz boot-z14.img | tee SHA256SUMS-z14)
