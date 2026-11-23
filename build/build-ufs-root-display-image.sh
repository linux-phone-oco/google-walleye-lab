#!/bin/sh
# UFS-ON, NOT TO BOOT except through the go (backlog walleye-install-ufs-root-display-image; decisions 2026-11-20-walleye-install-stage-1-go, 2026-11-20-walleye-display-handover-reset-first,
# 2026-11-19-walleye-ufs-hold-second-path): image U-d5, the stage 1 root on the glass. ONE kernel with UFS on AND the display: the matched UFS tree (topic/ufs-match-android) plus two commits (the mdss reset,
# the panel driver), the display path as modules (13, built for THIS kernel, in the initramfs), fbcon built in; the dtb is the board file's display dtb (mdss reset in the kernel's own form, the lab levers of every
# display image) with the UFS overlay of the matched image (dt/msm8998-google-walleye-ufs-match.dtso) merged in; the initramfs is build/initramfs-root-display/init: the display modules at 5 s of uptime, then the card's
# system partition READ-ONLY under a tmpfs overlay, then the root's /sbin/init. The header, the command line (plus walleye.root=system_a) and the AVB footer are the matched image's.
# The image is built and checked here and NEVER BOOTED here: it is written to a directory named NOT-TO-BOOT-..., with a NOT-TO-BOOT.txt that names the only way it boots (the one command of
# walleye-install-boot-procedure, run from walleye-install-rootfs-boots-try, which waits for the author's go); tests/host/ufs-off-image.sh refuses it by design (checked below: the gate fails it, nothing
# here weakens or skips the gate).
#   build-ufs-root-display-image.sh MATCHED_IMG KERNEL_TREE KERNEL_OUT_DIR WORK_DIR OUT_DIR
# MATCHED_IMG is the matched image (build-ufs-match.sh's NOT-TO-BOOT-ufs-on-ufs-match.img); KERNEL_TREE the display+UFS tree (branch wip/walleye/ufs-root-display of linux-v7.3, committed: the tree is the
# matched one plus the commits the checks name); KERNEL_OUT_DIR is build-display-late.sh's OUT_DIR run on that tree with EXTRA_CONFIG="build/walleye-diag.config build/walleye-ufs-root.config" (Image.gz, .config,
# the display dtb, modroot/, display-modules.list); WORK_DIR holds busybox-arm64 (build-busybox.sh).
# Environment: ROOT_PART (system_a default, or system_b), DELAY (seconds of uptime at which the display modules load, default 5), DTBO (default ~/Projects/walleye-build/control/dtbo.img).
# Writes OUT_DIR/{NOT-TO-BOOT-ufs-on-ufs-root-display.img,msm8998-google-walleye-ufs-root-display.dtb,NOT-TO-BOOT.txt,SHA256SUMS-ufs-root-display}; every check stops the build (ok/FAIL, exit 1, nothing written).
# The image is built twice and the two must be the same bytes.
set -eu
MATCHED=${1:?usage: build-ufs-root-display-image.sh MATCHED_IMG KERNEL_TREE KERNEL_OUT_DIR WORK_DIR OUT_DIR}
KT=${2:?}
KO=${3:?}
W=${4:?}
OUT=${5:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
ROOT_PART=${ROOT_PART:-system_a}
DELAY=${DELAY:-5}
case $ROOT_PART in system_a|system_b) ;; *) echo "build-ufs-root-display-image: ROOT_PART is system_a or system_b, not $ROOT_PART" >&2; exit 1 ;; esac
case $DELAY in ''|*[!0-9]*) echo "build-ufs-root-display-image: DELAY is a number of seconds, not $DELAY" >&2; exit 1 ;; esac
case $(basename "$OUT") in NOT-TO-BOOT-*) ;; *) echo "build-ufs-root-display-image: OUT_DIR must be named NOT-TO-BOOT-... (a UFS-on image is never written where a queue or the inventory looks)" >&2; exit 1 ;; esac
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
same() { cmp -s "$1" "$2" && echo same || echo differs; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
GEN=$KO/b/usr/gen_init_cpio
BUILT=$KO/msm8998-google-walleye-display.dtb
INITD=$HERE/build/initramfs-root-display
for f in "$MATCHED" "$DTBO" "$KO/Image.gz" "$KO/.config" "$KO/display-modules.list" "$BUILT" "$W/busybox-arm64" "$GEN" "$INITD/init" "$HERE/dt/msm8998-google-walleye-ufs-match.dtso" "$KT/drivers/ufs/host/ufs-qcom.c"; do [ -s "$f" ] || { echo "build-ufs-root-display-image: missing $f" >&2; exit 1; }; done
[ -d "$KO/modroot/lib/modules" ] || { echo "build-ufs-root-display-image: no $KO/modroot/lib/modules" >&2; exit 1; }
. "$HERE/build/lib-display-levers.sh"
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-ufs-root-display-image: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
INFO=$(unpack_bootimg --boot_img "$MATCHED" --out "$T/m")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/m/kernel" "$T/mk"
PSIZE=$(stat -c %s "$MATCHED")
AV=$(avbtool info_image --image "$MATCHED")
CMDLINE="$(field 'command line args') walleye.root=$ROOT_PART"
# --- the kernel: the matched tree plus the display commits, nothing else
MBANNER=$(zcat "$T/mk.gz" | LC_ALL=C grep -a -o 'Linux version [^ ]*' | sort -u)
MHEAD=$(printf '%s\n' "$MBANNER" | sed -n 's/.*-g\([0-9a-f]*\)\(-dirty\)\{0,1\}$/\1/p')
HEADHASH=$(git -C "$KT" rev-parse --short=12 HEAD)
BANNERS=$(zcat "$KO/Image.gz" | LC_ALL=C grep -a -o "Linux version [^ ]*" | sort -u)
ck "the tree has no change but the lines the display build adds to the dts Makefile and the board dts it copies in (git status of the whole tree)" "$(git -C "$KT" status --porcelain --untracked-files=all | grep -v -e '^ M arch/arm64/boot/dts/qcom/Makefile$' -e '^?? arch/arm64/boot/dts/qcom/msm8998-google-walleye' | wc -l)" 0
ck "the Image's banner names the tree's HEAD commit g$HEADHASH, and only it" "$(printf '%s\n' "$BANNERS" | grep -c -E -e "-g$HEADHASH(-dirty)?$")/$(printf '%s\n' "$BANNERS" | grep -c .)" 1/1
ck "the matched image's tree commit ($MHEAD) is an ancestor of this tree's HEAD" "$(git -C "$KT" merge-base --is-ancestor "$MHEAD" HEAD && echo yes || echo no)" yes
ck "the commits this tree adds to the matched one are exactly two, the mdss reset and the panel driver" "$(git -C "$KT" log --reverse --format=%s "$MHEAD"..HEAD | tr '\n' '|')" "arm64: dts: qcom: msm8998: Add the MDSS reset|drm/panel: Add the Samsung S6E3FA5 command mode panel (Pixel 2)|"
ck "they touch only msm8998.dtsi and the panel directory of the drm drivers" "$(git -C "$KT" diff --name-only "$MHEAD" HEAD | sed 's|^drivers/gpu/drm/panel/.*|PANEL|' | sort -u | tr '\n' ' ')" "arch/arm64/boot/dts/qcom/msm8998.dtsi PANEL "
ck "the Image carries the matched code (the purge guard, the version print, the phy refusal, the two read-only prints)" "$(zcat "$KO/Image.gz" | LC_ALL=C grep -a -c -e 'purge refused on this device' -e 'UFS_HW_VERSION %u.%u.%u' -e 'unknown UFS host version' -e 'UFS_HOST_LEFT HCS' -e 'UFS_PHY_LINECFG_DISABLE')" 5
ck "the kernel config builds the UFS host, its phy, the msm8998 pinctrl, the sd disk driver, the GPT table, ext4, the overlay filesystem, tmpfs and devtmpfs in" "$(grep -c -e '^CONFIG_SCSI_UFS_QCOM=y$' -e '^CONFIG_PHY_QCOM_QMP_UFS=y$' -e '^CONFIG_PINCTRL_MSM8998=y$' -e '^CONFIG_BLK_DEV_SD=y$' -e '^CONFIG_EFI_PARTITION=y$' -e '^CONFIG_EXT4_FS=y$' -e '^CONFIG_OVERLAY_FS=y$' -e '^CONFIG_TMPFS=y$' -e '^CONFIG_DEVTMPFS=y$' "$KO/.config")" 9
ck "the kernel config is the diagnostic one (dynamic debug and gpio debug)" "$(grep -c -e '^CONFIG_DYNAMIC_DEBUG=y$' -e '^CONFIG_DEBUG_GPIO=y$' "$KO/.config")" 2
ck "the display path is modules (DRM, DRM_MSM, the panel) and the console on the glass is built in (DPU, DSI, the 10nm PHY, backlight, fbdev emulation, fbcon)" "$(grep -c -e '^CONFIG_DRM=m$' -e '^CONFIG_DRM_MSM=m$' -e '^CONFIG_DRM_PANEL_SAMSUNG_S6E3FA5=m$' -e '^CONFIG_DRM_MSM_DPU=y$' -e '^CONFIG_DRM_MSM_DSI=y$' -e '^CONFIG_DRM_MSM_DSI_10NM_PHY=y$' -e '^CONFIG_BACKLIGHT_CLASS_DEVICE=y$' -e '^CONFIG_DRM_FBDEV_EMULATION=y$' -e '^CONFIG_FRAMEBUFFER_CONSOLE=y$' "$KO/.config")" 9
ck "the display driver is not in the kernel: dpu_kms_init is not in System.map" "$(grep -c ' dpu_kms_init' "$KO/System.map" || true)" 0
KVER=$(ls "$KO/modroot/lib/modules")
ck "the modules are for this kernel: one release dir, named in the Image's banner" "$(printf '%s\n' "$KVER" | wc -l) $(printf '%s\n' "$BANNERS" | grep -c -F -e "Linux version $KVER")" "1 1"
ck "every module's vermagic names that release" "$(find "$KO/modroot" -name '*.ko' | while read -r m; do modinfo -F vermagic "$m" | cut -d' ' -f1; done | sort | uniq -c | sed 's/^ *//')" "$(find "$KO/modroot" -name '*.ko' | wc -l) $KVER"
ck "the module list has 13 lines, every one an insmod of an absolute path that is a file of the staging root" "$(wc -l < "$KO/display-modules.list") $(grep -c '^insmod /' "$KO/display-modules.list") $(sed 's/^insmod //' "$KO/display-modules.list" | while read -r p; do [ -s "$KO/modroot$p" ] && echo y; done | wc -l)" "13 13 13"
# --- the dtb: the display board dtb (the any-order form: the mdss reset from the kernel's dts) with the lab levers, then the matched image's UFS overlay
cp "$BUILT" "$T/display.dtb"; display_levers "$T/display.dtb"
dtc -@ -I dts -O dtb -o "$T/ufs-match.dtbo" "$HERE/dt/msm8998-google-walleye-ufs-match.dtso" 2>"$T/dtc.err" || { cat "$T/dtc.err" >&2; echo "build-ufs-root-display-image: the overlay does not compile" >&2; exit 1; }
fdtoverlay -i "$T/display.dtb" -o "$T/new.dtb" "$T/ufs-match.dtbo" || { echo "build-ufs-root-display-image: the UFS overlay does not merge into the display dtb" >&2; exit 1; }
M=/soc@0/display-subsystem@c900000
UFS=$(fdtget "$T/new.dtb" /soc@0/ufshc@1da4000 status)/$(fdtget "$T/new.dtb" /soc@0/phy@1da7000 status)
ck "the built display dtb has UFS off, the merged dtb UFS on (the accident the gate exists for, on purpose)" "$(fdtget "$T/display.dtb" /soc@0/ufshc@1da4000 status)/$(fdtget "$T/display.dtb" /soc@0/phy@1da7000 status) $UFS" "disabled/disabled okay/okay"
TLMM=$(fdtget -t x "$T/new.dtb" "$(fdtget "$T/new.dtb" /__symbols__ tlmm)" phandle)
WANT=$(printf '%s\n' "< gpio-ranges = <0x$TLMM 0x00 0x00 0x96>;" '< gpio-reserved-ranges = <0x00 0x04 0x51 0x04>;' '< status = "disabled";' '< status = "disabled";' \
  "> gpio-ranges = <0x$TLMM 0x00 0x00 0x9a>;" '> gpio-reserved-ranges = <0x00 0x04 0x51 0x04 0x96 0x03>;' '> regulator-always-on;' "> reset-gpios = <0x$TLMM 0x99 0x01>;" '> status = "okay";' '> status = "okay";' | sort | tr '\n' '|')
dtc -I dtb -O dts "$T/display.dtb" 2>/dev/null > "$T/d.dts"; dtc -I dtb -O dts "$T/new.dtb" 2>/dev/null > "$T/n.dts"
diff "$T/d.dts" "$T/n.dts" > "$T/dts.diff" || true
ck "the display dtb and the image dtb differ in exactly the UFS delta of the matched image (10 lines: two statuses, the reset gpio, gpio-ranges 150 to 154, the sdc2 pads reserved, l26 always on)" "$(grep '^[<>]' "$T/dts.diff" | sed 's/^\([<>]\)[[:space:]]*/\1 /' | sort | tr '\n' '|')" "$WANT"
ck "the tree and the dtb carry what Android's UFS path carries for the card (tests/host/ufs-match-android.sh)" "$(sh "$HERE/tests/host/ufs-match-android.sh" "$KT" "$T/new.dtb" 2>&1 | tail -n1)" "ufs-match-android: all checks ok"
ck "the dtb keeps every /soc@0 child of the matched image's dtb" "$(fdtget -l "$T/mk.dtb" /soc@0 | sort > "$T/mkc"; fdtget -l "$T/new.dtb" /soc@0 | sort > "$T/newc"; comm -23 "$T/mkc" "$T/newc" | wc -l)" 0
ck "the display nodes are in: mdss and the panel, with the reset (tests/host/dtb-display-reset.sh)" "$(fdtget "$T/new.dtb" $M status) $(fdtget "$T/new.dtb" $M/dsi@c994000/panel@0 compatible) $(DTB="$T/new.dtb" sh "$HERE/tests/host/dtb-display-reset.sh" | cut -d' ' -f1)" "okay samsung,s6e3fa5 pass"
ck "clock_gcc is the real gcc, the dsi has vdd-supply and vdda-supply" "$(fdtget "$T/new.dtb" /__symbols__ clock_gcc) $(fdtget -p "$T/new.dtb" $M/dsi@c994000 | grep -c '^vdd-supply$\|^vdda-supply$')" "/soc@0/clock-controller@100000 2"
ck "the stock overlay still merges into the image dtb (tests/host/dtbo-overlay-merges.sh)" "$(DTB="$T/new.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "the overlay overwrites nothing of ours (tests/host/overlay-collisions.sh)" "$(DTB="$T/new.dtb" QUIET=1 sh "$HERE/tests/host/overlay-collisions.sh" | tail -n1 | cut -d' ' -f1)" pass
ck "after the overlay merge the clock users name gcc (tests/host/overlay-clock-gcc.sh)" "$(DTB="$T/new.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
ck "exactly one available node names the touch power gpio" "$(DTB="$T/new.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
ck "the panel reset gpio has one claimant" "$(DTB="$T/new.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
# --- the initramfs: ours, with the display files, twice built, the same bytes
DF=$T/df; mkdir -p "$DF"; cp -r "$KO/modroot/lib" "$DF/lib"; cp "$KO/display-modules.list" "$DF/display-modules.list"; printf '%s\n' "$DELAY" > "$DF/display-delay"
GEN=$GEN INIT_DIR=$INITD DISPLAY_FILES=$DF "$HERE/build/build-initramfs.sh" ufs-root "$W" "$T/ramdisk1.cpio.gz" >/dev/null
GEN=$GEN INIT_DIR=$INITD DISPLAY_FILES=$DF "$HERE/build/build-initramfs.sh" ufs-root "$W" "$T/ramdisk2.cpio.gz" >/dev/null
ck "the initramfs builds to the same bytes twice" "$(same "$T/ramdisk1.cpio.gz" "$T/ramdisk2.cpio.gz")" same
mkdir "$T/rd" && (cd "$T/rd" && gzip -dc "$T/ramdisk1.cpio.gz" | cpio -idm --quiet -f 'dev/*')
ck "the initramfs' /init is build/initramfs-root-display/init, byte for byte" "$(same "$T/rd/init" "$INITD/init")" same
ck "the initramfs carries the arm64 busybox, the delay ($DELAY), the module list, and each module of it" "$(same "$T/rd/bin/busybox" "$W/busybox-arm64") $(cat "$T/rd/display-delay") $(same "$T/rd/display-modules.list" "$KO/display-modules.list") $(sed 's/^insmod //' "$T/rd/display-modules.list" | while read -r p; do cmp -s "$T/rd$p" "$KO/modroot$p" && echo y; done | wc -l)" "same $DELAY same 13"
ck "the init's only mount of the card is read-only, nodiscard, noload; it has no mkfs, fsck, discard, trim, rw or remount" "$(grep -c -e 'mount -t ext4 -o ro,nodiscard,noload ' "$T/rd/init")/$(grep -v '^#' "$T/rd/init" | grep -c -w -e mkfs -e mke2fs -e fsck -e e2fsck -e dd -e fstrim -e blkdiscard -e 'remount' -e 'rw,')" 1/0
ck "the init refuses without walleye.root, for a foreign name, without androidboot.slot_suffix (fails closed) and for the slot Android runs from" "$(grep -v '^#' "$T/rd/init" | grep -c -e 'no walleye.root on the command line' -e 'is not system_a or system_b' -e 'the slot Android runs from' -e 'no androidboot.slot_suffix on the command line')" 4
ck "the proof of the init passes (tests/host/ufs-root-display-init.sh: the text against the root init, QEMU with a fake display, read-only, overlay, refusals)" "$(GEN=$GEN sh "$HERE/tests/host/ufs-root-display-init.sh" "$W" | cut -d' ' -f1)" pass
# --- the image, built twice
make_image() { # OUTFILE
  out=$1
  cat "$KO/Image.gz" "$T/new.dtb" > "$T/blob"
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
IMG=$T/NOT-TO-BOOT-ufs-on-ufs-root-display.img
make_image "$IMG"
make_image "$T/second.img"
ck "the image builds to the same bytes twice" "$(same "$IMG" "$T/second.img")" same
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
hdr() { printf '%s\n' "$1" | grep -v -e '^kernel_size' -e 'ramdisk size' -e 'kernel size' -e '^command line args'; }
ck "the image's Image.gz is the new build's, byte for byte" "$(same "$T/uk.gz" "$KO/Image.gz")" same
ck "the image's dtb is the merged dtb, byte for byte" "$(same "$T/uk.dtb" "$T/new.dtb")" same
ck "the image's header equals the matched one's but the sizes and the command line" "$(hdr "$U")" "$(hdr "$INFO")"
ck "the image's command line is the matched one plus walleye.root=$ROOT_PART, and nothing else" "$(printf '%s\n' "$U" | sed -n 's/^command line args: *//p' | head -n1)" "$CMDLINE"
ck "the command line names no ufs but the bootloader's own words, and no root=, no init=, no rw" "$(printf '%s\n' "$CMDLINE" | tr ' ' '\n' | grep -v '^androidboot\.' | grep -i -c -e ufs -e '^root=' -e '^init=' -e '^rw$')" 0
ck "the image's ramdisk is the initramfs built here, byte for byte" "$(same "$T/u/ramdisk" "$T/ramdisk1.cpio.gz")" same
ck "the image's size is the matched one's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "the image's AVB info equals the matched one's but the digest and the sizes of what it hashes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the image's AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
KS=$(stat -c %s "$T/blob"); KO_=$(printf %d "$(field 'kernel load address')"); RO=$(printf %d "$(field 'ramdisk load address')")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
# the gate refuses it, by design: the image stays out of every queue; the only go is the item's, read by the boot procedure
ck "the UFS-off gate FAILS this image because UFS is on in it (the gate is not weakened: it is the proof the lab's tools refuse the image)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -c1-30)" "fail walleye-ufs-off UFS is on"
[ $total -eq 0 ] || { echo "build-ufs-root-display-image: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/NOT-TO-BOOT-ufs-on-ufs-root-display.img"
cp "$T/new.dtb" "$OUT/msm8998-google-walleye-ufs-root-display.dtb"
cat > "$OUT/NOT-TO-BOOT.txt" <<EON
NOT-TO-BOOT-ufs-on-ufs-root-display.img has UFS ON and mounts $ROOT_PART of the card read-only (tmpfs overlay), with the display modules loaded at $DELAY s. It is built and checked on the host and was never booted.
It is not to be booted, and not to be put in a keeper's queue, except by the one command of walleye-install-boot-procedure, run from item walleye-install-rootfs-boots-try (decision
2026-11-20-walleye-install-stage-1-go, step 4), which stops unless the author's go is recorded on that item for this image's sha256. tests/host/ufs-off-image.sh refuses it by design.
No way back from an erased Pixel 2 bootloader is known (research walleye-edl-way-back).
EON
(cd "$OUT" && sha256sum NOT-TO-BOOT-ufs-on-ufs-root-display.img msm8998-google-walleye-ufs-root-display.dtb | tee SHA256SUMS-ufs-root-display)
