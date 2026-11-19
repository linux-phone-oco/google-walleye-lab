#!/bin/sh
# Images B-d5, B-d15, B-d25, B-d35, B-d50 (item walleye-display-delay-bisect-build): B2 whose init waits a delay baked into the image before it loads the display modules.
#   build-display-delay-images.sh B2_BOOT_IMG LATE_DIR OUT_DIR [DELAY...]      (DELAY: seconds of uptime, default 5 15 25 35 50)
#   (LATE_DIR: build-display-late.sh's OUT_DIR, the ramdisk's modules and list; environment WORK, default ~/Projects/walleye-build: busybox-arm64)
# Only two things differ from B2: /init is init-display-delay (init-display plus the wait, added lines only) and the file /display-delay holds the number of seconds
# of uptime. The drawer /bin/walleye-stripes is taken from B2's own ramdisk. Kernel, dtb, command line, header words, size, modules and list are B2's.
# The ramdisk is made the way build-display-look-image.sh makes it: first with init-display and no delay file, which must be B2's ramdisk byte for byte (the control), then once per delay.
# Every image is made twice and must come out identical (rebuild identical). Writes OUT_DIR/{boot-b-dN.img,b-dN.diff (the file list of B2's ramdisk against it),SHA256SUMS-display-delay};
# every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
B2=${1:?usage: build-display-delay-images.sh B2_BOOT_IMG LATE_DIR OUT_DIR [DELAY...]}
D=${2:?}
OUT=${3:?}
shift 3
DELAYS=${*:-5 15 25 35 50}
HERE=$(cd "$(dirname "$0")/.." && pwd)
W=${WORK:-$HOME/Projects/walleye-build}
BB=$W/busybox-arm64
GEN=${GEN:-$D/b/usr/gen_init_cpio}
ID=$HERE/build/initramfs-mainline/init-display
IDD=$HERE/build/initramfs-mainline/init-display-delay
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$B2" "$D/display-modules.list" "$BB" "$GEN" "$ID" "$IDD"; do [ -s "$f" ] || [ -x "$f" ] || { echo "build-display-delay-images: missing $f" >&2; exit 1; }; done
for n in $DELAYS; do case "$n" in ''|*[!0-9]*) echo "build-display-delay-images: a delay is a whole number of seconds, not '$n'" >&2; exit 1;; esac; done
seen=" "; for n in $DELAYS; do case "$seen" in *" $n "*) echo "build-display-delay-images: the delay $n is given twice" >&2; exit 1;; esac; seen="$seen$n "; done
B2SUM=$(cd "$(dirname "$B2")" && grep " boot-b2.img\$" SHA256SUMS-display-look | cut -d" " -f1)
ck "the control is B2: its sha256 is the one recorded (SHA256SUMS-display-look)" "$(sha256sum "$B2" | cut -d" " -f1)" "$B2SUM"
ROOT=$D/modroot
same() { cmp -s "$1" "$2" && echo same || echo differs; }
hdr() { grep -v '^#' "$1/boot.toml"; }
listing() { xz -dc "$1" | cpio -t 2>/dev/null | sort; }
lpoco-bringup image unpack "$B2" --out "$T/b2" >/dev/null
mkdir "$T/b2rd" && (cd "$T/b2rd" && xz -dc "$T/b2/ramdisk" | cpio -id 2>/dev/null || true)
BIN=$T/b2rd/bin/walleye-stripes
ck "B2's ramdisk holds the drawer, an aarch64 static executable" "$(file -b "$BIN" | grep -c 'ARM aarch64.*statically linked\|ARM aarch64.*static-pie linked')" 1
spec() { # INIT [DELAYFILE]
  for dr in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $dr 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $1 755 0 0"
  echo "file /etc/udhcpd.conf $HERE/build/initramfs-mainline/udhcpd-z14.conf 644 0 0"
  echo "file /display-modules.list $D/display-modules.list 644 0 0"
  [ -z "${2:-}" ] || echo "file /display-delay $2 644 0 0"
  echo "file /bin/walleye-stripes $BIN 755 0 0"
  (cd "$ROOT" && find lib -type d -printf 'dir /%p 755 0 0\n')
  (cd "$ROOT" && find lib -type f -printf "file /%p $ROOT/%p 644 0 0\n")
}
pack() { spec "$@" > "$T/spec"; "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"; xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio"; }
pack "$ID" > "$T/control.xz"
ck "the ramdisk made with init-display and no delay file is B2's, byte for byte (control)" "$(same "$T/control.xz" "$T/b2/ramdisk")" same
ck "init-display-delay is init-display plus added lines only (no line of init-display removed)" "$(diff -u "$ID" "$IDD" | sed '1,2d' | grep -v '^@@' | grep -c '^-' || true)" 0
ck "the delay init waits for /display-delay before it loads (tests/host/display-delay-init.sh)" "$(sh "$HERE/tests/host/display-delay-init.sh" | cut -d' ' -f1 || true)" pass
ck "the delay init still loads every list line, logs failures, is backgrounded (tests/host/display-at-boot-init.sh)" "$(INIT="$IDD" sh "$HERE/tests/host/display-at-boot-init.sh" 2>/dev/null | cut -d' ' -f1 || true)" pass
ck "every init of ours writes the serial walleye-first-boot" "$(sh "$HERE/build/check-init-serials.sh" "$HERE/build" >/dev/null 2>&1 && echo yes || echo no)" yes
make_one() { # DELAY DIR: the image of that delay in DIR
  n=$1; d=$2; mkdir -p "$d"
  printf '%s\n' "$n" > "$d/delay"
  pack "$IDD" "$d/delay" > "$d/ramdisk.xz"
  lpoco-bringup image swap "$B2" --ramdisk "$d/ramdisk.xz" --out "$d/boot.img" >/dev/null
}
for n in $DELAYS; do
  echo "--- B-d$n"
  make_one "$n" "$T/d$n"
  make_one "$n" "$T/again-d$n"
  IMG=$T/d$n/boot.img
  ck "B-d$n: made twice, the two images are identical (rebuild identical)" "$(same "$IMG" "$T/again-d$n/boot.img")" same
  lpoco-bringup image unpack "$IMG" --out "$T/u$n" >/dev/null
  ck "B-d$n: the kernel (Image.gz and dtb) is B2's" "$(same "$T/u$n/kernel" "$T/b2/kernel")" same
  ck "B-d$n: the command line and header words are B2's" "$([ "$(hdr "$T/u$n")" = "$(hdr "$T/b2")" ] && echo same || echo differs)" same
  ck "B-d$n: the size is B2's" "$(stat -c %s "$IMG")" "$(stat -c %s "$B2")"
  ck "B-d$n: the ramdisk is the packed one, byte for byte" "$(same "$T/u$n/ramdisk" "$T/d$n/ramdisk.xz")" same
  ck "B-d$n: the ramdisk differs from B2's" "$(same "$T/u$n/ramdisk" "$T/b2/ramdisk")" differs
  listing "$T/b2/ramdisk" > "$T/b2.files"; listing "$T/u$n/ramdisk" > "$T/b-d$n.files"
  diff -u "$T/b2.files" "$T/b-d$n.files" | sed '1,2d' > "$T/b-d$n.diff" || true
  ck "B-d$n: against B2's ramdisk the file list gains display-delay and loses nothing" "$(grep -c '^+' "$T/b-d$n.diff") $(grep -c '^-' "$T/b-d$n.diff") $(grep '^+' "$T/b-d$n.diff" | grep -c 'display-delay$')" "1 0 1"
  mkdir "$T/rd$n" && (cd "$T/rd$n" && xz -dc "$T/u$n/ramdisk" | cpio -id 2>/dev/null || true)
  ck "B-d$n: the ramdisk's /init is init-display-delay, byte for byte" "$(same "$T/rd$n/init" "$IDD")" same
  ck "B-d$n: the ramdisk's /display-delay says $n" "$(cat "$T/rd$n/display-delay")" "$n"
  ck "B-d$n: the drawer and the module list are B2's" "$(same "$T/rd$n/bin/walleye-stripes" "$BIN") $(same "$T/rd$n/display-modules.list" "$T/b2rd/display-modules.list")" "same same"
  mkdir "$T/v$n" && cp "$IMG" "$T/v$n/boot.img"
  ck "B-d$n: the AVB footer verifies" "$(avbtool verify_image --image "$T/v$n/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "B-d$n: the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$IMG" >/dev/null 2>&1 && echo pass || echo fail)" pass
  ck "B-d$n: the UFS-off tests pass (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
  ck "B-d$n: the panel reset gpio has one claimant in the image's dtb" "$(DTB="$T/u$n/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
done
[ $total -eq 0 ] || { echo "build-display-delay-images: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"; rm -f "$OUT/SHA256SUMS-display-delay"
for n in $DELAYS; do
  rm -f "$OUT/boot-b-d$n.img" "$OUT/b-d$n.diff"
  cp "$T/d$n/boot.img" "$OUT/boot-b-d$n.img"; cp "$T/b-d$n.diff" "$OUT/b-d$n.diff"
  (cd "$OUT" && sha256sum "boot-b-d$n.img" "b-d$n.diff" >> SHA256SUMS-display-delay)
done
cat "$OUT/SHA256SUMS-display-delay"
