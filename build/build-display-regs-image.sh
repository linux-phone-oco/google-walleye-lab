#!/bin/sh
# Image B-q5 (item walleye-display-scanout-registers; decision 2026-11-20-walleye-display-handover-reset-first: read the DPU registers to learn what stops at 25 s): B-d5 whose init also reads the scan-out's registers.
#   build-display-regs-image.sh BD5_BOOT_IMG LATE_DIR OUT_DIR      (BD5_BOOT_IMG: build-display-delay-images.sh's boot-b-d5.img; LATE_DIR: build-display-late.sh's OUT_DIR; environment WORK, default ~/Projects/walleye-build: busybox-arm64)
# Three things differ from B-d5's ramdisk: /init is init-display-regs (init-display-delay plus the regs block, added lines only), and the files /regs.list (build/initramfs-mainline/display-regs.list) and /regs-times ("4 10 30"):
# at 4 s (the bootloader's splash, before the modules), 10 s (after msm probes) and 30 s (after the flip near 25 s) the init reads every listed register with devmem, read only, and says it to the kernel log.
# Kernel, dtb, command line (clk_ignore_unused keeps the clocks on), header words, size, modules, list, drawer and the 5 s delay are B-d5's. The ramdisk is packed the way build-display-delay-images.sh packs it:
# first with init-display-delay, which must be B-d5's ramdisk byte for byte (the control), then with the regs init. Made twice, identical.
# Writes OUT_DIR/{boot-b-q5.img,b-q5.diff (the file list of B-d5's ramdisk against it),SHA256SUMS-display-regs}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
IMG=${1:?usage: build-display-regs-image.sh BD5_BOOT_IMG LATE_DIR OUT_DIR}
D=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
W=${WORK:-$HOME/Projects/walleye-build}
BB=$W/busybox-arm64
GEN=${GEN:-$D/b/usr/gen_init_cpio}
IM=$HERE/build/initramfs-mainline
IDD=$IM/init-display-delay
IDR=$IM/init-display-regs
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$IMG" "$D/display-modules.list" "$BB" "$GEN" "$IDD" "$IDR" "$IM/display-regs.list" "$IM/display-regs-times"; do [ -s "$f" ] || [ -x "$f" ] || { echo "build-display-regs-image: missing $f" >&2; exit 1; }; done
SUM=$(cd "$(dirname "$IMG")" && grep " boot-b-d5.img\$" SHA256SUMS-display-delay | cut -d" " -f1)
ck "the input is B-d5: its sha256 is the one recorded (SHA256SUMS-display-delay)" "$(sha256sum "$IMG" | cut -d" " -f1)" "$SUM"
ROOT=$D/modroot
same() { cmp -s "$1" "$2" && echo same || echo differs; }
hdr() { grep -v '^#' "$1/boot.toml"; }
listing() { xz -dc "$1" | cpio -t 2>/dev/null | sort; }
lpoco-bringup image unpack "$IMG" --out "$T/d5" >/dev/null
mkdir "$T/d5rd" && (cd "$T/d5rd" && xz -dc "$T/d5/ramdisk" | cpio -id 2>/dev/null || true)
BIN=$T/d5rd/bin/walleye-stripes
ck "B-d5's ramdisk holds the drawer, an aarch64 static executable" "$(file -b "$BIN" | grep -c 'ARM aarch64.*statically linked\|ARM aarch64.*static-pie linked')" 1
ck "B-d5's /display-delay says 5" "$(cat "$T/d5rd/display-delay")" 5
spec() { # INIT DELAYFILE [EXTRA-SPEC-FILE]
  for dr in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $dr 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $1 755 0 0"
  echo "file /etc/udhcpd.conf $IM/udhcpd-z14.conf 644 0 0"
  echo "file /display-modules.list $D/display-modules.list 644 0 0"
  echo "file /display-delay $2 644 0 0"
  [ -z "${3:-}" ] || cat "$3"
  echo "file /bin/walleye-stripes $BIN 755 0 0"
  (cd "$ROOT" && find lib -type d -printf 'dir /%p 755 0 0\n')
  (cd "$ROOT" && find lib -type f -printf "file /%p $ROOT/%p 644 0 0\n")
}
pack() { spec "$@" > "$T/spec"; "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"; xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio"; }
printf '5\n' > "$T/delay"
pack "$IDD" "$T/delay" > "$T/control.xz"
ck "the ramdisk made with init-display-delay and the 5 s delay is B-d5's, byte for byte (control)" "$(same "$T/control.xz" "$T/d5/ramdisk")" same
ck "init-display-regs is init-display-delay plus added lines only, and reads read only (tests/host/display-regs-init.sh)" "$(sh "$HERE/tests/host/display-regs-init.sh" | cut -d' ' -f1 || true)" pass
ck "the delay init still passes its own test (tests/host/display-delay-init.sh)" "$(sh "$HERE/tests/host/display-delay-init.sh" | cut -d' ' -f1 || true)" pass
ck "the regs init still loads every list line, logs failures, is backgrounded (tests/host/display-at-boot-init.sh)" "$(INIT="$IDR" sh "$HERE/tests/host/display-at-boot-init.sh" 2>/dev/null | cut -d' ' -f1 || true)" pass
ck "every init of ours writes the serial walleye-first-boot" "$(sh "$HERE/build/check-init-serials.sh" "$HERE/build" >/dev/null 2>&1 && echo yes || echo no)" yes
echo "file /regs.list $IM/display-regs.list 644 0 0" > "$T/extra"; echo "file /regs-times $IM/display-regs-times 644 0 0" >> "$T/extra"
make_one() { # DIR
  d=$1; mkdir -p "$d"
  pack "$IDR" "$T/delay" "$T/extra" > "$d/ramdisk.xz"
  lpoco-bringup image swap "$IMG" --ramdisk "$d/ramdisk.xz" --out "$d/boot.img" >/dev/null
}
make_one "$T/q"; make_one "$T/again"
Q=$T/q/boot.img
ck "made twice, the two images are identical (rebuild identical)" "$(same "$Q" "$T/again/boot.img")" same
lpoco-bringup image unpack "$Q" --out "$T/u" >/dev/null
ck "the kernel (Image.gz and dtb) is B-d5's" "$(same "$T/u/kernel" "$T/d5/kernel")" same
ck "the command line and header words are B-d5's" "$([ "$(hdr "$T/u")" = "$(hdr "$T/d5")" ] && echo same || echo differs)" same
ck "the command line keeps clk_ignore_unused (the clocks stay on for the reads)" "$(grep -c 'clk_ignore_unused' "$T/u/boot.toml")" 1
ck "the size is B-d5's" "$(stat -c %s "$Q")" "$(stat -c %s "$IMG")"
ck "the ramdisk is the packed one, byte for byte" "$(same "$T/u/ramdisk" "$T/q/ramdisk.xz")" same
ck "the ramdisk differs from B-d5's" "$(same "$T/u/ramdisk" "$T/d5/ramdisk")" differs
listing "$T/d5/ramdisk" > "$T/d5.files"; listing "$T/u/ramdisk" > "$T/b-q5.files"
diff -u "$T/d5.files" "$T/b-q5.files" | sed '1,2d' > "$T/b-q5.diff" || true
ck "against B-d5's ramdisk the file list gains regs.list and regs-times and loses nothing" "$(grep -c '^+' "$T/b-q5.diff") $(grep -c '^-' "$T/b-q5.diff") $(grep '^+' "$T/b-q5.diff" | grep -c 'regs.list$\|regs-times$')" "2 0 2"
mkdir "$T/rd" && (cd "$T/rd" && xz -dc "$T/u/ramdisk" | cpio -id 2>/dev/null || true)
ck "the ramdisk's /init is init-display-regs, byte for byte" "$(same "$T/rd/init" "$IDR")" same
ck "the ramdisk's /regs.list and /regs-times are the shipped ones" "$(same "$T/rd/regs.list" "$IM/display-regs.list") $(same "$T/rd/regs-times" "$IM/display-regs-times")" "same same"
ck "the ramdisk's /display-delay says 5, the drawer and the module list are B-d5's" "$(cat "$T/rd/display-delay") $(same "$T/rd/bin/walleye-stripes" "$BIN") $(same "$T/rd/display-modules.list" "$T/d5rd/display-modules.list")" "5 same same"
ck "the busybox in the ramdisk has devmem" "$(strings "$T/rd/bin/busybox" | grep -c '^devmem$')" 1
mkdir "$T/v" && cp "$Q" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$Q" >/dev/null 2>&1 && echo pass || echo fail)" pass
ck "the UFS-off tests pass (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$Q" | cut -d' ' -f1 || true)" pass
ck "the panel reset gpio has one claimant in the image's dtb" "$(DTB="$T/u/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
[ $total -eq 0 ] || { echo "build-display-regs-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"; rm -f "$OUT/boot-b-q5.img" "$OUT/b-q5.diff" "$OUT/SHA256SUMS-display-regs"
cp "$Q" "$OUT/boot-b-q5.img"; cp "$T/b-q5.diff" "$OUT/b-q5.diff"
(cd "$OUT" && sha256sum boot-b-q5.img b-q5.diff | tee SHA256SUMS-display-regs)
