#!/bin/sh
# Image L7 (item walleye-display-look-l7): L6 with the stripe drawer replaced by the build that puts the console in KD_GRAPHICS while the stripes are held (KD_TEXT after, also after a failed draw). Nothing else changes: init (init-late), kernel, dtb, command line, header words, size, modules and list are L6's.
#   build-display-l7-image.sh L6_BOOT_IMG LATE_DIR STRIPES_BIN OUT_DIR
#   (LATE_DIR: build-display-late.sh's OUT_DIR, the ramdisk's modules and list; STRIPES_BIN: build-stripes-bin.sh's walleye-stripes; environment WORK, default ~/Projects/walleye-build: busybox-arm64)
# The ramdisk is made the way build-display-stripes-image.sh makes it: first with the drawer L6 holds (taken out of L6's own ramdisk), which must be L6's ramdisk byte for byte (the control), then with the new drawer. The drawer is started by the procedure over the shell, not by init.
# Writes OUT_DIR/{boot-l7.img,l7.diff (the file list of L6's ramdisk against L7's: none; the drawer's bytes are the change),SHA256SUMS-display-l7}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
L6=${1:?usage: build-display-l7-image.sh L6_BOOT_IMG LATE_DIR STRIPES_BIN OUT_DIR}
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
for f in "$L6" "$BIN" "$D/display-modules.list" "$BB" "$GEN"; do [ -s "$f" ] || [ -x "$f" ] || { echo "build-display-l7-image: missing $f" >&2; exit 1; }; done
# (the old outputs are removed only when every check passed, just before the new ones are copied)
L6SUM=$(cd "$(dirname "$L6")" && grep " boot-l6.img\$" SHA256SUMS-display-stripes | cut -d" " -f1)
ck "the control is L6: its sha256 is the one recorded (SHA256SUMS-display-stripes)" "$(sha256sum "$L6" | cut -d" " -f1)" "$L6SUM"
ck "the drawer is an aarch64 static executable" "$(file -b "$BIN" | grep -c 'ARM aarch64.*statically linked\|ARM aarch64.*static-pie linked')" 1
ck "the drawer names KDSETMODE, KD_GRAPHICS and KD_TEXT (it is aarch64: it cannot run here; its logic is the unit tests of tools/walleye-stripes)" "$(for w in KDSETMODE KD_GRAPHICS KD_TEXT; do strings "$BIN" | grep -qF "$w" && printf 1 || printf 0; done)" 111
ROOT=$D/modroot
spec() { # DRAWER_FILE
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $HERE/build/initramfs-mainline/init-late 755 0 0"
  echo "file /etc/udhcpd.conf $HERE/build/initramfs-mainline/udhcpd-z14.conf 644 0 0"
  echo "file /display-modules.list $D/display-modules.list 644 0 0"
  echo "file /bin/walleye-stripes $1 755 0 0"
  (cd "$ROOT" && find lib -type d -printf 'dir /%p 755 0 0\n')
  (cd "$ROOT" && find lib -type f -printf "file /%p $ROOT/%p 644 0 0\n")
}
pack() { spec "$1" > "$T/spec"; "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"; xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio"; }
lpoco-bringup image unpack "$L6" --out "$T/l6" >/dev/null
mkdir "$T/rd6" && (cd "$T/rd6" && xz -dc "$T/l6/ramdisk" | cpio -id 2>/dev/null || true)
pack "$T/rd6/bin/walleye-stripes" > "$T/control.xz"
ck "the ramdisk made with L6's own drawer is L6's, byte for byte (control)" "$(cmp -s "$T/control.xz" "$T/l6/ramdisk" && echo same || echo differs)" same
pack "$BIN" > "$T/ramdisk.xz"
lpoco-bringup image swap "$L6" --ramdisk "$T/ramdisk.xz" --out "$T/boot-l7.img" >/dev/null
IMG=$T/boot-l7.img
lpoco-bringup image unpack "$IMG" --out "$T/l7" >/dev/null
same() { cmp -s "$1" "$2" && echo same || echo differs; }
hdr() { grep -v '^#' "$1/boot.toml"; }
ck "the kernel (Image.gz and dtb) is L6's" "$(same "$T/l7/kernel" "$T/l6/kernel")" same
ck "the command line and header words are L6's" "$([ "$(hdr "$T/l7")" = "$(hdr "$T/l6")" ] && echo same || echo differs)" same
ck "the size is L6's" "$(stat -c %s "$IMG")" "$(stat -c %s "$L6")"
ck "the ramdisk is the packed one, byte for byte" "$(same "$T/l7/ramdisk" "$T/ramdisk.xz")" same
ck "the ramdisk differs from L6's" "$(same "$T/l7/ramdisk" "$T/l6/ramdisk")" differs
listing() { xz -dc "$1" | cpio -t 2>/dev/null | sort; }
listing "$T/l6/ramdisk" > "$T/l6.files"; listing "$T/l7/ramdisk" > "$T/l7.files"
diff -u "$T/l6.files" "$T/l7.files" | sed '1,2d' > "$T/l7.diff" || true
ck "against L6's ramdisk the file list gains nothing and loses nothing" "$(grep -c '^[-+]' "$T/l7.diff")" 0
mkdir "$T/rd" && (cd "$T/rd" && xz -dc "$T/l7/ramdisk" | cpio -id 2>/dev/null || true)
ck "the ramdisk's /bin/walleye-stripes is the built drawer, byte for byte" "$(same "$T/rd/bin/walleye-stripes" "$BIN")" same
ck "the ramdisk's /bin/walleye-stripes is not L6's" "$(same "$T/rd/bin/walleye-stripes" "$T/rd6/bin/walleye-stripes")" differs
ck "the ramdisk's /init is L6's init-late" "$(cmp -s "$T/rd/init" "$HERE/build/initramfs-mainline/init-late" && echo same || echo differs)" same
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$IMG" >/dev/null 2>&1 && echo pass || echo fail)" pass
ck "the UFS-off tests pass (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
ck "the panel reset gpio has one claimant in the image's dtb" "$(DTB="$T/l7/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
ck "every init of ours writes the serial walleye-first-boot" "$(sh "$HERE/build/check-init-serials.sh" "$HERE/build" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-display-l7-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"; rm -f "$OUT/boot-l7.img" "$OUT/l7.diff" "$OUT/SHA256SUMS-display-l7"
cp "$IMG" "$OUT/boot-l7.img"; cp "$T/l7.diff" "$OUT/l7.diff"
(cd "$OUT" && sha256sum boot-l7.img l7.diff | tee SHA256SUMS-display-l7)
