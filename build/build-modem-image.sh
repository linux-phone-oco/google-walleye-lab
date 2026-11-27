#!/bin/sh
# The modem boot image (item walleye-modem-mss-image): the slpi image's header, size and AVB footer with a new kernel, dtb and ramdisk, made by `lpoco-bringup image swap` and judged before anything is written.
#   build-modem-image.sh IMAGE_CONTROL IMAGE_KERNEL(Image.gz) MODEM_DTB_BUILT RAMDISK OUT_DIR
# IMAGE_CONTROL is the slpi image ([images.slpi]); MODEM_DTB_BUILT is dt/msm8998-google-walleye-modem.dts built on the tag's tree (DTC_FLAGS=-@); RAMDISK is build-modem-ramdisk.sh's. The dtb gets the real gcc's name for clock_gcc
# (as every image since z3: the bootloader's overlay merge needs it, tests/host/overlay-clock-gcc.sh). Writes OUT_DIR/boot-modem.img and OUT_DIR/SHA256SUMS-modem (mode 600, OUT_DIR 700): the image holds the phone's modem storage, never git, never the Nix store.
# Every check stops the build (ok/FAIL, exit 1, nothing written): the swap twice equal, the AVB footer verifies, the UFS-off gate and the dtb tests pass, the size and header are the control's, the kernel is the one given, the ramdisk is the one given.
# Environment: DTBO (the control's dtbo.img, as ufs-gate.sh), CMDLINE (default: the control's).
set -eu
CONTROL=${1:?usage: build-modem-image.sh IMAGE_CONTROL IMAGE_KERNEL MODEM_DTB_BUILT RAMDISK OUT_DIR}
KERNEL=${2:?}; DTB0=${3:?}; RAMDISK=${4:?}; OUT=${5:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
GCC=/soc@0/clock-controller@100000
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
umask 077; T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$CONTROL" "$KERNEL" "$DTB0" "$RAMDISK" "$DTBO"; do [ -s "$f" ] || { echo "build-modem-image: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$CONTROL" --out "$T/control")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
CMD=${CMDLINE:-$(field 'command line args')}
ck "the command line names no ufs" "$(printf '%s\n' "$CMD" | grep -ci ufs || true)" 0
cp "$DTB0" "$T/modem.dtb"
fdtput -t s "$T/modem.dtb" /__symbols__ clock_gcc "$GCC"
ck "clock_gcc is the real gcc in the image dtb" "$(fdtget "$T/modem.dtb" /__symbols__ clock_gcc)" "$GCC"
swap() { lpoco-bringup image swap "$CONTROL" --kernel "$KERNEL" --dtb "$T/modem.dtb" --ramdisk "$RAMDISK" --cmdline "$CMD" --out "$1" > "$1.log" 2>&1 || { cat "$1.log" >&2; return 1; }; }
swap "$T/boot-modem.img"; swap "$T/again.img"
ck "a second swap gives the same bytes" "$(sha256sum < "$T/again.img")" "$(sha256sum < "$T/boot-modem.img")"
U=$(unpack_bootimg --boot_img "$T/boot-modem.img" --out "$T/u")
ck "the size is the control's ($(stat -c %s "$CONTROL"))" "$(stat -c %s "$T/boot-modem.img")" "$(stat -c %s "$CONTROL")"
for h in 'kernel load address' 'ramdisk load address' 'kernel tags load address' 'page size' 'os version' 'os patch level' 'boot image header version'; do
  ck "header $h is the control's" "$(printf '%s\n' "$U" | sed -n "s/^$h: *//p")" "$(field "$h")"
done
ck "the command line is the one meant" "$(printf '%s\n' "$U" | sed -n 's/^command line args: *//p')" "$CMD"
cat "$KERNEL" "$T/modem.dtb" > "$T/blob"
ck "the kernel blob is Image.gz then the dtb, byte for byte" "$(cmp -s "$T/u/kernel" "$T/blob" && echo same || echo differs)" same
ck "the ramdisk is the one given, byte for byte" "$(cmp -s "$T/u/ramdisk" "$RAMDISK" && echo same || echo differs)" same
KS=$(stat -c %s "$T/blob"); KO=$(printf %d "$(field 'kernel load address')"); RO=$(printf %d "$(field 'ramdisk load address')")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO - KS))")" yes
mkdir "$T/v" && cp "$T/boot-modem.img" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$T/boot-modem.img" >/dev/null 2>&1 && echo pass || echo fail)" pass
ck "tests/host/dtb-modem-mss.sh passes the dtb" "$(DTB="$T/modem.dtb" sh "$HERE/tests/host/dtb-modem-mss.sh" | cut -d' ' -f1)" pass
ck "tests/host/dtb-modem-ipa.sh passes the dtb" "$(DTB="$T/modem.dtb" sh "$HERE/tests/host/dtb-modem-ipa.sh" | cut -d' ' -f1)" pass
ck "tests/host/modem-ramdisk.sh passes the ramdisk" "$(RAMDISK="$RAMDISK" sh "$HERE/tests/host/modem-ramdisk.sh" | cut -d' ' -f1)" pass
for t in touch-gpio-single-claimant overlay-collisions dtbo-overlay-merges overlay-clock-gcc i2c-qup-after-merge touch-power-after-merge adreno-smmu-disabled; do
  ck "tests/host/$t.sh on the dtb" "$(DTB="$T/modem.dtb" QUIET=1 sh "$HERE/tests/host/$t.sh" | tail -n1 | cut -d' ' -f1)" pass
done
[ $total -eq 0 ] || { echo "build-modem-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"; chmod 700 "$OUT"
cp "$T/boot-modem.img" "$OUT/boot-modem.img"
(cd "$OUT" && sha256sum boot-modem.img | tee SHA256SUMS-modem)
