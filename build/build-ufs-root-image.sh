#!/bin/sh
# UFS-ON, NOT TO BOOT except through the go (backlog walleye-install-ufs-root-image; decisions 2026-11-20-walleye-install-stage-1-go, 2026-11-20-walleye-install-in-stages-proposed,
# 2026-11-19-walleye-ufs-hold-second-path): the matched UFS image's kernel code and dtb (UFS on, byte for byte), the matched header and AVB footer, a kernel with the overlay filesystem built in,
# and OUR OWN initramfs (build/initramfs-root/init, no mkinitfs) that mounts the card's system_a (or system_b) READ-ONLY (ext4 ro,nodiscard,noload), puts a tmpfs overlay on it and hands over to
# the root's /sbin/init. The card is never written by it. The command line is the matched one plus `walleye.root=system_a`; the init refuses the slot Android is on unless walleye.booted-slot is given (stage 2).
# The image is built and checked here and NEVER BOOTED here: it is written to a directory named NOT-TO-BOOT-..., with a NOT-TO-BOOT.txt that names the only way it boots (the one command of
# walleye-install-boot-procedure, run from walleye-install-rootfs-boots-try, which waits for the author's go on stage 1 step 4); tests/host/ufs-off-image.sh refuses it by design (checked
# below: the gate fails it, and nothing here weakens or skips the gate).
#   build-ufs-root-image.sh MATCHED_IMG KERNEL_TREE KERNEL_OUT_DIR WORK_DIR OUT_DIR
# MATCHED_IMG is the matched image (build-ufs-match.sh's NOT-TO-BOOT-ufs-on-ufs-match.img); KERNEL_TREE the tree KERNEL_OUT_DIR was built from (build-kernel.sh with
# EXTRA_CONFIG="build/walleye-diag.config build/walleye-ufs-root.config"); WORK_DIR holds busybox-arm64 (build-busybox.sh).
# Environment: ROOT_PART (system_a, default, or system_b: the slot Android is NOT on; the procedure reads the slot before it boots anything), DTBO (default ~/Projects/walleye-build/control/dtbo.img),
# BOOTED_SLOT (unset, default: the RAM-boot variant of stage 1, the old guard; "yes": the STAGE 2 variant that is flashed - the command line also carries the word walleye.booted-slot, so the init mounts exactly
# the system of the slot it is booted from, and ROOT_PART must be the system of the slot it is flashed to; written as NOT-TO-BOOT-ufs-on-ufs-root-booted-slot.img, a name of its own, never mistaken for the stage 1 image).
# Writes OUT_DIR/{NOT-TO-BOOT-ufs-on-ufs-root[-booted-slot].img,NOT-TO-BOOT.txt,SHA256SUMS-ufs-root[-booted-slot]}; checks (ok/FAIL, exit 1, nothing written). The image is built twice and the two must be the same bytes.
set -eu
MATCHED=${1:?usage: build-ufs-root-image.sh MATCHED_IMG KERNEL_TREE KERNEL_OUT_DIR WORK_DIR OUT_DIR}
KT=${2:?}
KO=${3:?}
W=${4:?}
OUT=${5:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
ROOT_PART=${ROOT_PART:-system_a}
case $ROOT_PART in system_a|system_b) ;; *) echo "build-ufs-root-image: ROOT_PART is system_a or system_b, not $ROOT_PART" >&2; exit 1 ;; esac
BOOTED_SLOT=${BOOTED_SLOT:-}
case $BOOTED_SLOT in
  "") WORD=""; SUFFIX="" ;;
  yes) WORD=" walleye.booted-slot"; SUFFIX="-booted-slot" ;;
  *) echo "build-ufs-root-image: BOOTED_SLOT is unset (stage 1) or yes (stage 2), not $BOOTED_SLOT" >&2; exit 1 ;;
esac
DEBUG_INIT=${DEBUG_INIT:-}   # "yes" (with BOOTED_SLOT=yes): the stage 2 DEBUG image - init build/initramfs-root-debug/init, the command line also carries walleye.debug, written as ...-booted-slot-debug (backlog walleye-stage2-debug-image)
INITD=initramfs-root
case $DEBUG_INIT in
  "") ;;
  yes) [ "$BOOTED_SLOT" = yes ] || { echo "build-ufs-root-image: DEBUG_INIT=yes needs BOOTED_SLOT=yes" >&2; exit 1; }; WORD="$WORD walleye.debug"; SUFFIX="-booted-slot-debug"; INITD=initramfs-root-debug ;;
  *) echo "build-ufs-root-image: DEBUG_INIT is unset or yes, not $DEBUG_INIT" >&2; exit 1 ;;
esac
PRODUCTION_INIT=${PRODUCTION_INIT:-}   # "yes" (with BOOTED_SLOT=yes): the stage 2 PRODUCTION image - init build/initramfs-root-production/init (the petters first, no gadget, no telnet, no debug word), written as ...-booted-slot-production (backlog walleye-stage2-production-image)
case $PRODUCTION_INIT in
  "") ;;
  yes) [ "$BOOTED_SLOT" = yes ] && [ -z "$DEBUG_INIT" ] || { echo "build-ufs-root-image: PRODUCTION_INIT=yes needs BOOTED_SLOT=yes and no DEBUG_INIT" >&2; exit 1; }; SUFFIX="-booted-slot-production"; INITD=initramfs-root-production ;;
  *) echo "build-ufs-root-image: PRODUCTION_INIT is unset or yes, not $PRODUCTION_INIT" >&2; exit 1 ;;
esac
KERNEL_TAG=${KERNEL_TAG:-}   # an int/ tag (item walleye-next-image-from-an-int-tag): KERNEL_TREE is a checkout of exactly that tag, the Image's banner must name its commit, and the kernel is NOT the matched image's (the banner and 'same code as matched' checks are about the tag instead)
case $(basename "$OUT") in NOT-TO-BOOT-*) ;; *) echo "build-ufs-root-image: OUT_DIR must be named NOT-TO-BOOT-... (a UFS-on image is never written where a queue or the inventory looks)" >&2; exit 1 ;; esac
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
GEN=$KO/b/usr/gen_init_cpio
for f in "$MATCHED" "$DTBO" "$KO/Image.gz" "$KO/.config" "$W/busybox-arm64" "$GEN" "$HERE/build/$INITD/init" "$KT/drivers/ufs/host/ufs-qcom.c"; do [ -s "$f" ] || { echo "build-ufs-root-image: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-ufs-root-image: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
INFO=$(unpack_bootimg --boot_img "$MATCHED" --out "$T/m")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/m/kernel" "$T/mk"
PSIZE=$(stat -c %s "$MATCHED")
AV=$(avbtool info_image --image "$MATCHED")
# THE HEADER COMMAND LINE IS UNDER A BUDGET (item walleye-stage2-cmdline-too-long; research walleye-stage2-run8-debug-image-kernel-runs-slot-suffix-missing): the bootloader appends its own words (about 1100 characters, androidboot.slot_suffix
# among the last) after the header's and the kernel got only 1363 characters in run 8, the slot suffix cut off. The matched image's own header carries the diagnostic dyndbg="..." string (about 200 characters; the debug init writes the same
# lines to /sys/kernel/debug/dynamic_debug/control instead) and androidboot.console (Android's, which no init of ours reads): both are left out here.
BUDGET=${CMDLINE_BUDGET:-160}   # characters of the header command line; the margin check below is what decides, this refuses an image that grows back
CUT=1363        # the length of the command line run 8 got (research walleye-bootimage/2026-11-23/run8-telnet-debug-read.txt: 1363 characters, 1364 with the newline): an OBSERVED cut for our image, its cause (a fixed buffer or a total) not proven; the next phone run is the proof
MARGIN=100      # characters that must stay free after the bootloader's own words
HDR=$(field 'command line args' | sed 's/ *dyndbg="[^"]*"//g' | tr ' ' '\n' | grep -v -e '^androidboot\.console=' -e '^$' | paste -sd' ')
CMDLINE="$HDR walleye.root=$ROOT_PART$WORD"
# the budget and the margin, from captured text: run 8's /proc/cmdline (our header's words first, then the bootloader's, cut at 1364 characters) and Android b's (the bootloader's words to the end, which run 8's lost)
R8=$HERE/research/walleye-bootimage/2026-11-23/run8-telnet-debug-read.txt
AB=$HERE/research/walleye-bootimage/2026-11-23/android-b-cmdline-and-boot-props.txt
for f in "$R8" "$AB"; do [ -s "$f" ] || { echo "build-ufs-root-image: missing $f" >&2; exit 1; }; done
R8LINE=$(grep -a '^clk_ignore_unused' "$R8" | head -n1 | tr -d '\r')
ck "run 8's captured command line is the 1363 characters the bootloader passed (1364 with the newline of /proc/cmdline), and it ends after androidboot.pk=g-pxl-p, before androidboot.slot_suffix" "${#R8LINE} $(printf '%s' "$R8LINE" | grep -c 'androidboot\.pk=g-pxl-p$')" "$CUT 1"
BLWORDS="${R8LINE#* walleye.debug }$(head -n1 "$AB" | sed 's/.*androidboot\.pk=g-pxl-p//')"   # every word the bootloader adds after ours: run 8's, then the ones its cut lost, from Android b's line
ck "the header command line is within the budget of $BUDGET characters" "$([ ${#CMDLINE} -le "$BUDGET" ] && echo yes || echo "no: ${#CMDLINE}")" yes
ck "the header command line and the bootloader's words fit under the $CUT the bootloader passes with at least $MARGIN to spare (header ${#CMDLINE} + 1 + bootloader words ${#BLWORDS})" "$([ $((CUT - ${#CMDLINE} - 1 - ${#BLWORDS})) -ge "$MARGIN" ] && echo yes || echo "no: $((CUT - ${#CMDLINE} - 1 - ${#BLWORDS})) left")" yes
ck "the header command line has no dyndbg and no androidboot.console (run 8's line shows both in the part the bootloader cut at)" "$(printf '%s' "$CMDLINE" | grep -c -e dyndbg -e 'androidboot\.console')/$(printf '%s' "$R8LINE" | grep -c -e 'dyndbg=' -e 'androidboot\.console=')" 0/1
ck "the bootloader's last words, androidboot.slot_suffix among them, are in Android b's line and not in run 8's (what the cut lost)" "$(printf '%s' "$R8LINE" | grep -c 'androidboot\.slot_suffix')/$(head -n1 "$AB" | grep -c 'androidboot\.slot_suffix=_b want_initramfs rootwait ro init=/init')" 0/1
# the kernel: the tree it was built from, the code the matched image proved, the options this image needs
ck "the ufs, phy and pinctrl files of the tree are committed (clean against HEAD)" "$(git -C "$KT" diff --quiet HEAD -- drivers include && [ -z "$(git -C "$KT" ls-files --others -- drivers include)" ] && echo clean || echo dirty)" clean
HEADHASH=$(git -C "$KT" rev-parse --short=12 HEAD)
BANNERS=$(zcat "$KO/Image.gz" | LC_ALL=C grep -a -o "Linux version [^ ]*" | sort -u)
ck "the Image's banner names the tree's HEAD commit g$HEADHASH, and only it" "$(printf '%s\n' "$BANNERS" | grep -c -E -e "-g$HEADHASH(-dirty)?$")/$(printf '%s\n' "$BANNERS" | grep -c .)" 1/1
if [ -n "$KERNEL_TAG" ]; then
  ck "the tree's HEAD is the commit of the tag $KERNEL_TAG (the kernel is taken from the tag, never from a tree by hand)" "$(git -C "$KT" rev-parse HEAD)" "$(git -C "$KT" rev-parse "$KERNEL_TAG^{commit}")"
  ck "the tree differs from the tag only in the lab's device tree files (dts, the Makefile line of build-kernel.sh): no kernel code by hand" "$(git -C "$KT" status --porcelain | grep -c -v ' arch/arm64/boot/dts/qcom/')" 0
else
  ck "the Image has the same banner as the matched image's (the same tree HEAD)" "$(zcat "$KO/Image.gz" | LC_ALL=C grep -a -o 'Linux version [^ ]*' | sort -u)" "$(zcat "$T/mk.gz" | LC_ALL=C grep -a -o 'Linux version [^ ]*' | sort -u)"
fi
ck "the Image carries the matched code (the purge guard, the version print, the phy refusal, the two read-only prints)" "$(zcat "$KO/Image.gz" | LC_ALL=C grep -a -c -e 'purge refused on this device' -e 'UFS_HW_VERSION %u.%u.%u' -e 'unknown UFS host version' -e 'UFS_HOST_LEFT HCS' -e 'UFS_PHY_LINECFG_DISABLE')" 5
ck "the kernel config builds the UFS host, its phy, the msm8998 pinctrl, the sd disk driver, the GPT partition table, ext4 and the overlay filesystem in" "$(grep -c -e '^CONFIG_SCSI_UFS_QCOM=y$' -e '^CONFIG_PHY_QCOM_QMP_UFS=y$' -e '^CONFIG_PINCTRL_MSM8998=y$' -e '^CONFIG_BLK_DEV_SD=y$' -e '^CONFIG_EFI_PARTITION=y$' -e '^CONFIG_EXT4_FS=y$' -e '^CONFIG_OVERLAY_FS=y$' -e '^CONFIG_TMPFS=y$' -e '^CONFIG_DEVTMPFS=y$' "$KO/.config")" 9
ck "the kernel config is the matched one's (the diagnostic one: dynamic debug and gpio debug)" "$(grep -c -e '^CONFIG_DYNAMIC_DEBUG=y$' -e '^CONFIG_DEBUG_GPIO=y$' "$KO/.config")" 2
ck "the new Image.gz is not the matched one's (the overlay filesystem is built in)" "$(cmp -s "$KO/Image.gz" "$T/mk.gz" && echo same || echo differs)" differs
ck "the matched dtb has UFS on (the matched image is the one this image rests on)" "$(fdtget "$T/mk.dtb" /soc@0/ufshc@1da4000 status) $(fdtget "$T/mk.dtb" /soc@0/phy@1da7000 status)" "okay okay"
# the initramfs: ours, twice built, the same bytes
GEN=$GEN INIT_DIR=$HERE/build/$INITD "$HERE/build/build-initramfs.sh" ufs-root "$W" "$T/ramdisk1.cpio.gz" >/dev/null
GEN=$GEN INIT_DIR=$HERE/build/$INITD "$HERE/build/build-initramfs.sh" ufs-root "$W" "$T/ramdisk2.cpio.gz" >/dev/null
ck "the initramfs builds to the same bytes twice" "$(cmp -s "$T/ramdisk1.cpio.gz" "$T/ramdisk2.cpio.gz" && echo same || echo differs)" same
mkdir "$T/rd" && (cd "$T/rd" && gzip -dc "$T/ramdisk1.cpio.gz" | cpio -idm --quiet -f 'dev/*')
ck "the initramfs' /init is build/$INITD/init, byte for byte" "$(cmp -s "$T/rd/init" "$HERE/build/$INITD/init" && echo same || echo differs)" same
ck "the initramfs carries the arm64 busybox and no kernel module list" "$(cmp -s "$T/rd/bin/busybox" "$W/busybox-arm64" && [ ! -e "$T/rd/modules.list" ] && echo yes || echo no)" yes
ck "the init's only mount of the card is read-only, nodiscard, noload; it has no mkfs, fsck, discard, trim, rw or remount" "$(grep -c -e 'mount -t ext4 -o ro,nodiscard,noload ' "$T/rd/init")/$(grep -v '^#' "$T/rd/init" | grep -c -w -e mkfs -e mke2fs -e fsck -e e2fsck -e dd -e fstrim -e blkdiscard -e 'remount' -e 'rw,')" 1/0
ck "the init refuses without walleye.root, for a foreign name, without androidboot.slot_suffix (fails closed) and for the slot Android is on" "$(grep -v '^#' "$T/rd/init" | grep -c -e 'no walleye.root on the command line' -e 'is not system_a or system_b' -e 'the slot Android is on' -e 'no androidboot.slot_suffix on the command line')" 4
ck "the init takes the word walleye.booted-slot and says walleye-init: mounts-the-booted-slot (the stage 2 init of PR 765)" "$(grep -v '^#' "$T/rd/init" | grep -c -e 'say "walleye-init: mounts-the-booted-slot' -e '= walleye.booted-slot ')" 2
ck "the proof of the init passes (tests/host/ufs-root-init.sh: QEMU on a loop-made test rootfs, read-only, overlay, refusals)" "$(GEN=$GEN sh "$HERE/tests/host/ufs-root-init.sh" "$W" | cut -d' ' -f1)" pass
ck "the command line carries the word walleye.debug exactly when this is the debug variant (DEBUG_INIT=$DEBUG_INIT), and the init has its two DEBUG blocks exactly then" "$(printf '%s\n' "$CMDLINE" | tr ' ' '\n' | grep -c -x walleye.debug)/$(grep -c '^# DEBUG-BLOCK-BEGIN' "$T/rd/init")" "$([ -n "$DEBUG_INIT" ] && echo 1/2 || echo 0/0)"
[ -z "$DEBUG_INIT" ] || ck "the proof of the debug init passes (tests/host/ufs-root-debug-init.sh: the watchdog petted, the gadget up before the card, a refusal held, the gadget released, no word = the root init)" "$(GEN=$GEN sh "$HERE/tests/host/ufs-root-debug-init.sh" "$W" | cut -d' ' -f1)" pass
[ -z "$PRODUCTION_INIT" ] || ck "the proof of the production init passes (tests/host/ufs-root-production-init.sh: the petters before the slot and the card, no gadget or debug word, the root init's whole proof with it in its place)" "$(GEN=$GEN sh "$HERE/tests/host/ufs-root-production-init.sh" "$W" | cut -d' ' -f1)" pass
[ -z "$PRODUCTION_INIT" ] || ck "the production image's command line has no walleye.debug and its init has no DEBUG block" "$(printf '%s\n' "$CMDLINE" | tr ' ' '\n' | grep -c -x walleye.debug)/$(grep -c 'DEBUG-BLOCK' "$T/rd/init")" 0/0
# the image, built twice
make_image() { # OUTFILE
  out=$1
  cat "$KO/Image.gz" "$T/mk.dtb" > "$T/blob"
  set --
  while IFS= read -r pr; do if [ -n "$pr" ]; then set -- "$@" --prop "$pr"; fi; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$CMDLINE" \
    --kernel "$T/blob" --ramdisk "$T/ramdisk1.cpio.gz" -o "$out"
  avbtool add_hash_footer --image "$out" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
}
IMG=$T/NOT-TO-BOOT-ufs-on-ufs-root$SUFFIX.img
make_image "$IMG"
make_image "$T/second.img"
ck "the image builds to the same bytes twice" "$(cmp -s "$IMG" "$T/second.img" && echo same || echo differs)" same
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
hdr() { printf '%s\n' "$1" | grep -v -e '^kernel_size' -e 'ramdisk size' -e 'kernel size' -e '^command line args'; }
ck "the image's Image.gz is the new build's, byte for byte" "$(cmp -s "$T/uk.gz" "$KO/Image.gz" && echo same || echo differs)" same
ck "the image's dtb is the matched image's (UFS on), byte for byte" "$(cmp -s "$T/uk.dtb" "$T/mk.dtb" && echo same || echo differs)" same
ck "the image's header equals the matched one's but the sizes and the command line" "$(hdr "$U")" "$(hdr "$INFO")"
ck "the image's command line is the matched one's without its dyndbg string and androidboot.console, plus walleye.root=$ROOT_PART$WORD, and nothing else" "$(printf '%s\n' "$U" | sed -n 's/^command line args: *//p' | head -n1)" "$CMDLINE"
ck "the command line carries the word walleye.booted-slot exactly when this is the stage 2 variant (BOOTED_SLOT=$BOOTED_SLOT)" "$(printf '%s\n' "$CMDLINE" | tr ' ' '\n' | grep -c -x walleye.booted-slot)" "$([ -n "$BOOTED_SLOT" ] && echo 1 || echo 0)"
ck "the command line names no ufs but the bootloader's own words, and no root=, no init=, no rw" "$(printf '%s\n' "$CMDLINE" | tr ' ' '\n' | grep -v '^androidboot\.' | grep -i -c -e ufs -e '^root=' -e '^init=' -e '^rw$')" 0
ck "the image's ramdisk is the initramfs built here, byte for byte" "$(cmp -s "$T/u/ramdisk" "$T/ramdisk1.cpio.gz" && echo same || echo differs)" same
ck "the image's size is the matched one's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "the image's AVB info equals the matched one's but the digest and the sizes of what it hashes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the image's AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
KS=$(stat -c %s "$T/blob"); KO_=$(printf %d "$(field 'kernel load address')"); RO=$(printf %d "$(field 'ramdisk load address')")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
# the gate refuses it, by design: the image stays out of every queue; the only go is the item's, read by the boot procedure
ck "the UFS-off gate FAILS this image because UFS is on in it (the gate is not weakened: it is the proof the lab's tools refuse the image)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -c1-30)" "fail walleye-ufs-off UFS is on"
[ $total -eq 0 ] || { echo "build-ufs-root-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/NOT-TO-BOOT-ufs-on-ufs-root$SUFFIX.img"
if [ -n "$BOOTED_SLOT" ]; then WHERE="the one command of install-boot-on-the-pixel-2 (item walleye-install-flash-boot-run, under the coordinator's go on that item for this sha256), which flashes it to the boot partition of the slot Android is not on - the slot whose system partition is $ROOT_PART - and boots it there"; else WHERE="the one command of walleye-install-boot-procedure, run from item walleye-install-rootfs-boots-try (decision 2026-11-20-walleye-install-stage-1-go, step 4), which stops unless the author's go is recorded on that item"; fi
cat > "$OUT/NOT-TO-BOOT.txt" <<EON
NOT-TO-BOOT-ufs-on-ufs-root$SUFFIX.img has UFS ON and mounts $ROOT_PART of the card read-only (tmpfs overlay). It is built and checked on the host and was never booted. It is not to be booted, and not
to be put in a keeper's queue, except by $WHERE.
tests/host/ufs-off-image.sh refuses it by design. No way back from an erased Pixel 2 bootloader is known (research walleye-edl-way-back).
EON
(cd "$OUT" && sha256sum NOT-TO-BOOT-ufs-on-ufs-root$SUFFIX.img | tee SHA256SUMS-ufs-root$SUFFIX)
