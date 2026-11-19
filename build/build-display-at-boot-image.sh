#!/bin/sh
# Image B1 (item walleye-display-at-boot): L5 with init-display as /init. Nothing else changes: kernel, dtb (iommu fix, pinctrl labels, reset-gpio claimant), command line, header words, size, modules and list are L5's.
#   build-display-at-boot-image.sh L5_BOOT_IMG LATE_DIR OUT_DIR
#   (LATE_DIR: build-display-late.sh's OUT_DIR, the ramdisk's modules and list; environment WORK, default ~/Projects/walleye-build: busybox-arm64)
# The ramdisk is made the way build-display-stripes-image.sh makes it: first with init-late, which must be L5's ramdisk byte for byte (the control), then with init-display.
# Writes OUT_DIR/{boot-b1.img,b1.diff (init-late against init-display),SHA256SUMS-display-at-boot}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
L5=${1:?usage: build-display-at-boot-image.sh L5_BOOT_IMG LATE_DIR OUT_DIR}
D=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
W=${WORK:-$HOME/Projects/walleye-build}
BB=$W/busybox-arm64
GEN=${GEN:-$D/b/usr/gen_init_cpio}
IL=$HERE/build/initramfs-mainline/init-late
ID=$HERE/build/initramfs-mainline/init-display
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$L5" "$D/display-modules.list" "$BB" "$GEN" "$IL" "$ID"; do [ -s "$f" ] || [ -x "$f" ] || { echo "build-display-at-boot-image: missing $f" >&2; exit 1; }; done
rm -f "$OUT/boot-b1.img" "$OUT/b1.diff" "$OUT/SHA256SUMS-display-at-boot"
L5SUM=$(cd "$(dirname "$L5")" && grep " boot-l5.img\$" SHA256SUMS-display-late-l5 | cut -d" " -f1)
ck "the control is L5: its sha256 is the one recorded (SHA256SUMS-display-late-l5)" "$(sha256sum "$L5" | cut -d" " -f1)" "$L5SUM"
ROOT=$D/modroot
spec() { # INIT
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $1 755 0 0"
  echo "file /etc/udhcpd.conf $HERE/build/initramfs-mainline/udhcpd-z14.conf 644 0 0"
  echo "file /display-modules.list $D/display-modules.list 644 0 0"
  (cd "$ROOT" && find lib -type d -printf 'dir /%p 755 0 0\n')
  (cd "$ROOT" && find lib -type f -printf "file /%p $ROOT/%p 644 0 0\n")
}
pack() { spec "$1" > "$T/spec"; "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"; xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio"; }
lpoco-bringup image unpack "$L5" --out "$T/l5" >/dev/null
pack "$IL" > "$T/control.xz"
ck "the ramdisk made with init-late is L5's, byte for byte (control)" "$(cmp -s "$T/control.xz" "$T/l5/ramdisk" && echo same || echo differs)" same
pack "$ID" > "$T/ramdisk.xz"
lpoco-bringup image swap "$L5" --ramdisk "$T/ramdisk.xz" --out "$T/boot-b1.img" >/dev/null
IMG=$T/boot-b1.img
lpoco-bringup image unpack "$IMG" --out "$T/b1" >/dev/null
same() { cmp -s "$1" "$2" && echo same || echo differs; }
hdr() { grep -v '^#' "$1/boot.toml"; }
ck "the kernel (Image.gz and dtb) is L5's" "$(same "$T/b1/kernel" "$T/l5/kernel")" same
ck "the command line and header words are L5's" "$([ "$(hdr "$T/b1")" = "$(hdr "$T/l5")" ] && echo same || echo differs)" same
ck "the size is L5's" "$(stat -c %s "$IMG")" "$(stat -c %s "$L5")"
ck "the ramdisk is the packed one, byte for byte" "$(same "$T/b1/ramdisk" "$T/ramdisk.xz")" same
listing() { xz -dc "$1" | cpio -t 2>/dev/null | sort; }
ck "the ramdisk's file list is L5's (only /init's content differs)" "$([ "$(listing "$T/b1/ramdisk")" = "$(listing "$T/l5/ramdisk")" ] && echo same || echo differs)" same
mkdir "$T/rd" && (cd "$T/rd" && xz -dc "$T/b1/ramdisk" | cpio -id 2>/dev/null || true)
ck "the ramdisk's /init is init-display, byte for byte" "$(cmp -s "$T/rd/init" "$ID" && echo same || echo differs)" same
diff -u "$IL" "$ID" | sed '1,2d' > "$T/b1.diff" || true
ck "init-display is init-late plus added lines only (no line of init-late removed)" "$(grep -c '^-' "$T/b1.diff" || true)" 0
ck "the init loads the display modules itself (tests/host/display-at-boot-init.sh)" "$(INIT="$T/rd/init" sh "$HERE/tests/host/display-at-boot-init.sh" | cut -d' ' -f1 || true)" pass
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$IMG" >/dev/null 2>&1 && echo pass || echo fail)" pass
ck "the UFS-off tests pass (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
ck "the panel reset gpio has one claimant in the image's dtb" "$(DTB="$T/b1/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
ck "every init of ours writes the serial walleye-first-boot" "$(sh "$HERE/build/check-init-serials.sh" "$HERE/build" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-display-at-boot-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-b1.img"; cp "$T/b1.diff" "$OUT/b1.diff"
(cd "$OUT" && sha256sum boot-b1.img b1.diff | tee SHA256SUMS-display-at-boot)
