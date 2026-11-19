#!/bin/sh
# Image L6 (item walleye-display-stripes-image): L5 plus the stripe drawer /bin/walleye-stripes in the ramdisk. Nothing else changes: kernel, dtb, command line, header words and size are L5's.
#   build-display-stripes-image.sh L5_BOOT_IMG LATE_DIR STRIPES_BIN OUT_DIR
#   (LATE_DIR: build-display-late.sh's OUT_DIR, the ramdisk's modules and list; STRIPES_BIN: build-stripes-bin.sh's walleye-stripes; environment WORK, default ~/Projects/walleye-build: busybox-arm64)
# The ramdisk is made the way build-display-late-image.sh makes it; first without the drawer, which must be L5's ramdisk byte for byte (the control), then with it. The drawer is started by tests/host/display-late-probe.sh, not by init.
# Writes OUT_DIR/{boot-l6.img,l6.diff (the file list of L5's ramdisk against L6's),SHA256SUMS-display-stripes}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
L5=${1:?usage: build-display-stripes-image.sh L5_BOOT_IMG LATE_DIR STRIPES_BIN OUT_DIR}
D=${2:?}
BIN=${3:?}
OUT=${4:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
W=${WORK:-$HOME/Projects/walleye-build}
BB=$W/busybox-arm64
GEN=${GEN:-$D/b/usr/gen_init_cpio}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$L5" "$BIN" "$D/display-modules.list" "$BB" "$GEN"; do [ -s "$f" ] || [ -x "$f" ] || { echo "build-display-stripes-image: missing $f" >&2; exit 1; }; done
rm -f "$OUT/boot-l6.img" "$OUT/l6.diff" "$OUT/SHA256SUMS-display-stripes"   # a failed run leaves no earlier outputs standing
L5SUM=$(cd "$(dirname "$L5")" && grep " boot-l5.img\$" SHA256SUMS-display-late-l5 | cut -d" " -f1)
ck "the control is L5: its sha256 is the one recorded (SHA256SUMS-display-late-l5)" "$(sha256sum "$L5" | cut -d" " -f1)" "$L5SUM"
ck "the drawer is an aarch64 static executable" "$(file -b "$BIN" | grep -c 'ARM aarch64.*statically linked\|ARM aarch64.*static-pie linked')" 1
ROOT=$D/modroot
spec() { # [with the drawer]
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $HERE/build/initramfs-mainline/init-late 755 0 0"
  echo "file /etc/udhcpd.conf $HERE/build/initramfs-mainline/udhcpd-z14.conf 644 0 0"
  echo "file /display-modules.list $D/display-modules.list 644 0 0"
  [ "${1:-}" = drawer ] && echo "file /bin/walleye-stripes $BIN 755 0 0"
  (cd "$ROOT" && find lib -type d -printf 'dir /%p 755 0 0\n')
  (cd "$ROOT" && find lib -type f -printf "file /%p $ROOT/%p 644 0 0\n")
}
pack() { spec "$@" > "$T/spec"; "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"; xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio"; }
lpoco-bringup image unpack "$L5" --out "$T/l5" >/dev/null
pack > "$T/control.xz"
ck "the ramdisk made without the drawer is L5's, byte for byte (control)" "$(cmp -s "$T/control.xz" "$T/l5/ramdisk" && echo same || echo differs)" same
pack drawer > "$T/ramdisk.xz"
lpoco-bringup image swap "$L5" --ramdisk "$T/ramdisk.xz" --out "$T/boot-l6.img" >/dev/null
IMG=$T/boot-l6.img
lpoco-bringup image unpack "$IMG" --out "$T/l6" >/dev/null
same() { cmp -s "$1" "$2" && echo same || echo differs; }
hdr() { grep -v '^#' "$1/boot.toml"; }
ck "the kernel (Image.gz and dtb) is L5's" "$(same "$T/l6/kernel" "$T/l5/kernel")" same
ck "the command line and header words are L5's" "$([ "$(hdr "$T/l6")" = "$(hdr "$T/l5")" ] && echo same || echo differs)" same
ck "the size is L5's" "$(stat -c %s "$IMG")" "$(stat -c %s "$L5")"
ck "the ramdisk is the packed one, byte for byte" "$(same "$T/l6/ramdisk" "$T/ramdisk.xz")" same
ck "the ramdisk differs from L5's" "$(same "$T/l6/ramdisk" "$T/l5/ramdisk")" differs
# what the ramdisk holds, read back from the image: L5's files and the drawer, nothing else
listing() { xz -dc "$1" | cpio -t 2>/dev/null | sort; }
listing "$T/l5/ramdisk" > "$T/l5.files"; listing "$T/l6/ramdisk" > "$T/l6.files"
diff -u "$T/l5.files" "$T/l6.files" | sed '1,2d' > "$T/l6.diff" || true
ck "against L5's ramdisk the file list gains bin/walleye-stripes and loses nothing" "$(grep -c '^+' "$T/l6.diff") $(grep -c '^-' "$T/l6.diff") $(grep '^+' "$T/l6.diff" | grep -c 'bin/walleye-stripes$')" "1 0 1"
mkdir "$T/rd" && (cd "$T/rd" && xz -dc "$T/l6/ramdisk" | cpio -id 2>/dev/null || true)
ck "the ramdisk's /bin/walleye-stripes is the built drawer, byte for byte" "$(same "$T/rd/bin/walleye-stripes" "$BIN")" same
ck "the ramdisk's /init is L5's init-late" "$(cmp -s "$T/rd/init" "$HERE/build/initramfs-mainline/init-late" && echo same || echo differs)" same
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$IMG" >/dev/null 2>&1 && echo pass || echo fail)" pass
ck "the UFS-off tests pass (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
ck "the panel reset gpio has one claimant in the image's dtb" "$(DTB="$T/l6/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
[ $total -eq 0 ] || { echo "build-display-stripes-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-l6.img"; cp "$T/l6.diff" "$OUT/l6.diff"
(cd "$OUT" && sha256sum boot-l6.img l6.diff | tee SHA256SUMS-display-stripes)
