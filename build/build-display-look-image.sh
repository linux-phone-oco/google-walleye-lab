#!/bin/sh
# Image B2 (item walleye-display-look-b2): B1 plus the stripe drawer /bin/walleye-stripes in the ramdisk. Nothing else changes: init (init-display), kernel, dtb, command line, header words, size, modules and list are B1's.
#   build-display-look-image.sh B1_BOOT_IMG LATE_DIR STRIPES_BIN OUT_DIR
#   (LATE_DIR: build-display-late.sh's OUT_DIR, the ramdisk's modules and list; STRIPES_BIN: build-stripes-bin.sh's walleye-stripes, the build with KD_GRAPHICS while held; environment WORK, default ~/Projects/walleye-build: busybox-arm64)
# The ramdisk is made the way build-display-at-boot-image.sh makes it: first without the drawer, which must be B1's ramdisk byte for byte (the control), then with it. The drawer is started by the procedure over the shell, not by init.
# Writes OUT_DIR/{boot-b2.img,b2.diff (the file list of B1's ramdisk against B2's),SHA256SUMS-display-look}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
B1=${1:?usage: build-display-look-image.sh B1_BOOT_IMG LATE_DIR STRIPES_BIN OUT_DIR}
D=${2:?}
BIN=${3:?}
OUT=${4:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
W=${WORK:-$HOME/Projects/walleye-build}
BB=$W/busybox-arm64
GEN=${GEN:-$D/b/usr/gen_init_cpio}
ID=$HERE/build/initramfs-mainline/init-display
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$B1" "$BIN" "$D/display-modules.list" "$BB" "$GEN" "$ID"; do [ -s "$f" ] || [ -x "$f" ] || { echo "build-display-look-image: missing $f" >&2; exit 1; }; done
# (the old outputs are removed only when every check passed, just before the new ones are copied)
B1SUM=$(cd "$(dirname "$B1")" && grep " boot-b1.img\$" SHA256SUMS-display-at-boot | cut -d" " -f1)
ck "the control is B1: its sha256 is the one recorded (SHA256SUMS-display-at-boot)" "$(sha256sum "$B1" | cut -d" " -f1)" "$B1SUM"
ck "the drawer is an aarch64 static executable" "$(file -b "$BIN" | grep -c 'ARM aarch64.*statically linked\|ARM aarch64.*static-pie linked')" 1
ck "the drawer names the console modes (it is aarch64: it cannot run here; its logic is the unit tests of tools/walleye-stripes)" "$(strings "$BIN" | grep -cF -e KD_GRAPHICS -e KD_TEXT | awk "{print (\$1>0)}")" 1
ROOT=$D/modroot
spec() { # [with the drawer]
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $ID 755 0 0"
  echo "file /etc/udhcpd.conf $HERE/build/initramfs-mainline/udhcpd-z14.conf 644 0 0"
  echo "file /display-modules.list $D/display-modules.list 644 0 0"
  [ "${1:-}" = drawer ] && echo "file /bin/walleye-stripes $BIN 755 0 0"
  (cd "$ROOT" && find lib -type d -printf 'dir /%p 755 0 0\n')
  (cd "$ROOT" && find lib -type f -printf "file /%p $ROOT/%p 644 0 0\n")
}
pack() { spec "$@" > "$T/spec"; "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"; xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio"; }
lpoco-bringup image unpack "$B1" --out "$T/b1" >/dev/null
pack > "$T/control.xz"
ck "the ramdisk made without the drawer is B1's, byte for byte (control)" "$(cmp -s "$T/control.xz" "$T/b1/ramdisk" && echo same || echo differs)" same
pack drawer > "$T/ramdisk.xz"
lpoco-bringup image swap "$B1" --ramdisk "$T/ramdisk.xz" --out "$T/boot-b2.img" >/dev/null
IMG=$T/boot-b2.img
lpoco-bringup image unpack "$IMG" --out "$T/b2" >/dev/null
same() { cmp -s "$1" "$2" && echo same || echo differs; }
hdr() { grep -v '^#' "$1/boot.toml"; }
ck "the kernel (Image.gz and dtb) is B1's" "$(same "$T/b2/kernel" "$T/b1/kernel")" same
ck "the command line and header words are B1's" "$([ "$(hdr "$T/b2")" = "$(hdr "$T/b1")" ] && echo same || echo differs)" same
ck "the size is B1's" "$(stat -c %s "$IMG")" "$(stat -c %s "$B1")"
ck "the ramdisk is the packed one, byte for byte" "$(same "$T/b2/ramdisk" "$T/ramdisk.xz")" same
ck "the ramdisk differs from B1's" "$(same "$T/b2/ramdisk" "$T/b1/ramdisk")" differs
listing() { xz -dc "$1" | cpio -t 2>/dev/null | sort; }
listing "$T/b1/ramdisk" > "$T/b1.files"; listing "$T/b2/ramdisk" > "$T/b2.files"
diff -u "$T/b1.files" "$T/b2.files" | sed '1,2d' > "$T/b2.diff" || true
ck "against B1's ramdisk the file list gains bin/walleye-stripes and loses nothing" "$(grep -c '^+' "$T/b2.diff") $(grep -c '^-' "$T/b2.diff") $(grep '^+' "$T/b2.diff" | grep -c 'bin/walleye-stripes$')" "1 0 1"
mkdir "$T/rd" && (cd "$T/rd" && xz -dc "$T/b2/ramdisk" | cpio -id 2>/dev/null || true)
ck "the ramdisk's /bin/walleye-stripes is the built drawer, byte for byte" "$(same "$T/rd/bin/walleye-stripes" "$BIN")" same
ck "the ramdisk's /init is init-display, byte for byte" "$(cmp -s "$T/rd/init" "$ID" && echo same || echo differs)" same
ck "the init loads the display modules itself (tests/host/display-at-boot-init.sh)" "$(INIT="$T/rd/init" sh "$HERE/tests/host/display-at-boot-init.sh" | cut -d' ' -f1 || true)" pass
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$IMG" >/dev/null 2>&1 && echo pass || echo fail)" pass
ck "the UFS-off tests pass (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
ck "the panel reset gpio has one claimant in the image's dtb" "$(DTB="$T/b2/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
ck "every init of ours writes the serial walleye-first-boot" "$(sh "$HERE/build/check-init-serials.sh" "$HERE/build" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-display-look-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"; rm -f "$OUT/boot-b2.img" "$OUT/b2.diff" "$OUT/SHA256SUMS-display-look"
cp "$IMG" "$OUT/boot-b2.img"; cp "$T/b2.diff" "$OUT/b2.diff"
(cd "$OUT" && sha256sum boot-b2.img b2.diff | tee SHA256SUMS-display-look)
