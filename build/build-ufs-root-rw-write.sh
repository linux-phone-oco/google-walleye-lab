#!/bin/sh
# THE RW-WRITE IMAGE (backlog walleye-rw-write-image), UFS-ON, NOT TO BOOT except through the go: the stage 2 production image with ONE change in its ramdisk, build/initramfs-root-rw-write/init (the production init plus the blocks
# that act only with the word walleye.rw), and ONE word more on the header command line, walleye.rw. Kernel, dtb, header and AVB footer are the production image's; the init mounts the root ext4 rw,nodiscard, no overlay (tests/host/ufs-root-rw-init.sh).
# It is the image a recipe of device.toml names ([images.ufs-root-booted-slot-production-rw.recipe]: control, ramdisk by sha256, cmdline), so `lpoco bringup image ensure` rebuilds it equal to its declared sha256; this script makes the ramdisk
# the recipe names, swaps it in twice, and checks.
#   build-ufs-root-rw-write.sh CONTROL_IMG WORK_DIR OUT_DIR
# CONTROL_IMG is the production image ([images.ufs-root-booted-slot-production]); WORK_DIR holds busybox-arm64 (and busybox-x86_64 for the proof); OUT_DIR is named NOT-TO-BOOT-... Environment: GEN (a built tree's usr/gen_init_cpio), LPOCO_BRINGUP.
# Writes OUT_DIR/{NOT-TO-BOOT-ufs-on-ufs-root-booted-slot-production-rw.img,ramdisk-rw-write.cpio.gz,NOT-TO-BOOT.txt,SHA256SUMS-ufs-root-booted-slot-production-rw}; checks (ok/FAIL, exit 1, nothing written).
set -eu
CONTROL=${1:?usage: build-ufs-root-rw-write.sh CONTROL_IMG WORK_DIR OUT_DIR}
W=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
B=${LPOCO_BRINGUP:-lpoco-bringup}
NAME=ufs-root-booted-slot-production-rw
case $(basename "$OUT") in NOT-TO-BOOT-*) ;; *) echo "build-ufs-root-rw-write: OUT_DIR must be named NOT-TO-BOOT-... (a UFS-on image is never written where a queue or the inventory looks)" >&2; exit 1 ;; esac
GEN=${GEN:-$W/out/b/usr/gen_init_cpio}; export GEN
for f in "$CONTROL" "$W/busybox-arm64" "$GEN" "$HERE/build/initramfs-root-rw-write/init" "$HERE/device.toml"; do [ -s "$f" ] || { echo "build-ufs-root-rw-write: missing $f" >&2; exit 1; }; done
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
dev() { "$B" "$@" --device "$HERE/device.toml"; }
# the command line is the RECIPE'S, never the control's plus a word (item walleye-rw-image-header-equals-recipe: run 4's image kept walleye.booted-slot, which the recipe lacks)
CMDLINE=$(awk -v t="[images.$NAME.recipe]" '$0==t{on=1;next} /^\[/{on=0} on && /^cmdline *=/{sub(/^cmdline *= *"/,"");sub(/".*$/,"");print;exit}' "$HERE/device.toml")
[ -n "$CMDLINE" ] || { echo "build-ufs-root-rw-write: device.toml has no cmdline in [images.$NAME.recipe]" >&2; exit 1; }
INIT_DIR=$HERE/build/initramfs-root-rw-write "$HERE/build/build-initramfs.sh" ufs-root "$W" "$T/ramdisk1.cpio.gz" >/dev/null
INIT_DIR=$HERE/build/initramfs-root-rw-write "$HERE/build/build-initramfs.sh" ufs-root "$W" "$T/ramdisk2.cpio.gz" >/dev/null
ck "the initramfs builds to the same bytes twice" "$(cmp -s "$T/ramdisk1.cpio.gz" "$T/ramdisk2.cpio.gz" && echo same || echo differs)" same
mkdir "$T/rd" && (cd "$T/rd" && gzip -dc "$T/ramdisk1.cpio.gz" | cpio -idm --quiet -f 'dev/*')
ck "the initramfs' /init is build/initramfs-root-rw-write/init, byte for byte" "$(cmp -s "$T/rd/init" "$HERE/build/initramfs-root-rw-write/init" && echo same || echo differs)" same
ck "the initramfs carries the arm64 busybox and no kernel module list" "$(cmp -s "$T/rd/bin/busybox" "$W/busybox-arm64" && [ ! -e "$T/rd/modules.list" ] && echo yes || echo no)" yes
ck "with its RW-WRITE blocks cut the init is the production init, line for line" "$(sed '/^# RW-WRITE-BLOCK-BEGIN/,/^# RW-WRITE-BLOCK-END/d' "$T/rd/init" | cmp -s - "$HERE/build/initramfs-root-production/init" && echo same || echo differs)" same
ck "the proof of the rw init passes (tests/host/ufs-root-rw-init.sh: QEMU, rw root and a write on the partition with the word, the overlay without it, every guard)" "$(sh "$HERE/tests/host/ufs-root-rw-init.sh" "$W" | cut -d' ' -f1)" pass
ck "the proof of the production init still passes (the ro image's init is untouched)" "$(sh "$HERE/tests/host/ufs-root-production-init.sh" "$W" | cut -d' ' -f1)" pass
swap() { dev image swap "$CONTROL" --ramdisk "$T/ramdisk1.cpio.gz" --cmdline "$CMDLINE" --out "$1" >/dev/null; }
IMG=$T/NOT-TO-BOOT-ufs-on-ufs-root-booted-slot-production-rw.img
swap "$IMG"; swap "$T/second.img"
ck "the image builds to the same bytes twice" "$(cmp -s "$IMG" "$T/second.img" && echo same || echo differs)" same
ck "the image is not the production image (its own sha256)" "$([ "$(sha256sum < "$IMG")" != "$(sha256sum < "$CONTROL")" ] && echo differs || echo same)" differs
mkdir "$T/uc" "$T/ui"
unpack_bootimg --boot_img "$CONTROL" --out "$T/uc" > "$T/uc.info"; unpack_bootimg --boot_img "$IMG" --out "$T/ui" > "$T/ui.info"
ck "the kernel blob (Image.gz and dtb) is the production image's, byte for byte" "$(cmp -s "$T/uc/kernel" "$T/ui/kernel" && echo same || echo differs)" same
ck "the image's ramdisk is the initramfs built here, byte for byte" "$(cmp -s "$T/ui/ramdisk" "$T/ramdisk1.cpio.gz" && echo same || echo differs)" same
hdr() { grep -v -e 'ramdisk size' -e '^command line args' "$1"; }
ck "the header equals the production image's but the ramdisk size and the command line" "$(hdr "$T/ui.info")" "$(hdr "$T/uc.info" | sed 's/^$//')"
ck "the command line is the recipe's, word for word" "$(sed -n 's/^command line args: *//p' "$T/ui.info" | head -n1)" "$CMDLINE"
ck "the word walleye.rw is on the command line once, walleye.booted-slot is not (the init refuses it with a root that is not the booted slot's), walleye.root stays" "$(printf '%s\n' "$CMDLINE" | tr ' ' '\n' | grep -c -x walleye.rw)/$(printf '%s\n' "$CMDLINE" | tr ' ' '\n' | grep -c -x walleye.booted-slot)/$(printf '%s\n' "$CMDLINE" | tr ' ' '\n' | grep -c '^walleye.root=')" 1/0/1
ck "the command line stays under the budget of 160 characters (the bootloader cuts our line at 1363 with its own words after it; research walleye-stage2-run8-debug-image-kernel-runs-slot-suffix-missing)" "$([ ${#CMDLINE} -le 160 ] && echo yes || echo "no: ${#CMDLINE}")" yes
ck "the command line names no ufs but the bootloader's own words, and no root=, no init=, no bare rw" "$(printf '%s\n' "$CMDLINE" | tr ' ' '\n' | grep -v '^androidboot\.' | grep -i -c -e ufs -e '^root=' -e '^init=' -e '^rw$')" 0
ck "the image's size is the production image's" "$(stat -c %s "$IMG")" "$(stat -c %s "$CONTROL")"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the image's AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the image's AVB info equals the production image's but the digest and the sizes of what it hashes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')" "$(avbtool info_image --image "$CONTROL" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')"
ck "the UFS-off gate FAILS this image because UFS is on in it (the gate is not weakened)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -c1-30)" "fail walleye-ufs-off UFS is on"
[ $total -eq 0 ] || { echo "build-ufs-root-rw-write: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/NOT-TO-BOOT-ufs-on-$(basename "$IMG" | sed 's/^NOT-TO-BOOT-ufs-on-//')"
cp "$T/ramdisk1.cpio.gz" "$OUT/ramdisk-rw-write.cpio.gz"
cat > "$OUT/NOT-TO-BOOT.txt" <<EON
$(basename "$IMG") has UFS ON and, with the word walleye.rw its command line carries, mounts the system partition named on its command line READ-WRITE (ext4 rw,nodiscard, no overlay): the root is written. It is built and checked
on the host and was never booted. It is not to be booted, flashed or put in a keeper's queue except by the rw-write procedure (item walleye-rw-write-plumbing), under the coordinator's go that names THIS sha256.
tests/host/ufs-off-image.sh refuses it by design. The recipe that rebuilds it is [images.$NAME.recipe] of device.toml: control the production image, ramdisk $(sha256sum < "$T/ramdisk1.cpio.gz" | cut -d' ' -f1), cmdline "$CMDLINE".
EON
(cd "$OUT" && sha256sum "$(basename "$IMG")" ramdisk-rw-write.cpio.gz | tee SHA256SUMS-$NAME)
