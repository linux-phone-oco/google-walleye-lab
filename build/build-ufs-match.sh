#!/bin/sh
# UFS-ON, NOT TO BOOT (backlog walleye-ufs-match-android-z; decisions 2026-11-19-walleye-ufs-image-builds-before-the-go, 2026-11-19-walleye-ufs-version-read-in-our-driver):
# z21 with a new kernel and z21's dtb plus the UFS overlay (dt/msm8998-google-walleye-ufs-match.dtso), whose UFS path matches Android's for the Pixel 2's card (SK hynix H28U74301AMR): the one quirk Android applies to it
# (HOST_PA_SAVECONFIGTIME) and the NO_PURGE guard, the host's UFS_HW_VERSION (0xE4) read and printed in the image's own ufs-qcom init, the MSM8998 PHY table
# chosen by it (3.0.0 or 3.1.0, any other version stops the PHY), the card's RST_n pulse on tlmm pin 153, the l26 rail always on. Header, command line,
# ramdisk and AVB footer are z21's, byte for byte.
# The image is built and reviewed and NEVER BOOTED: it is written to a directory whose name says ufs-on, with a NOT-TO-BOOT note, it is never put in a keeper's
# queue, and tests/host/ufs-off-image.sh refuses it by design (checked below: the gate fails it, and nothing here weakens or skips the gate). The try is a separate item
# that waits for the author's explicit go on UFS (decision 2026-11-18-walleye-ufs-wait).
#   build-ufs-match.sh Z21_BOOT_IMG KERNEL_TREE KERNEL_OUT_DIR OUT_DIR
# KERNEL_TREE is the tree KERNEL_OUT_DIR was built from (build-kernel.sh, run with EXTRA_CONFIG=build/walleye-diag.config, z21's kernel config).
# Writes OUT_DIR/{NOT-TO-BOOT-ufs-on-ufs-match.img,msm8998-google-walleye-ufs-match.dtb,NOT-TO-BOOT.txt,SHA256SUMS-ufs-match}; checks (ok/FAIL, exit 1, nothing written).
# Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
Z21=${1:?usage: build-ufs-match.sh Z21_BOOT_IMG KERNEL_TREE KERNEL_OUT_DIR OUT_DIR}
KT=${2:?}
KO=${3:?}
OUT=${4:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
case $(basename "$OUT") in NOT-TO-BOOT-*) ;; *) echo "build-ufs-match: OUT_DIR must be named NOT-TO-BOOT-... (a UFS-on image is never written where a queue or the inventory looks)" >&2; exit 1 ;; esac
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
NEWDTB=$T/ufs-match.dtb
for f in "$Z21" "$DTBO" "$KO/Image.gz" "$KO/.config" "$HERE/dt/msm8998-google-walleye-ufs-match.dtso" "$KT/drivers/ufs/host/ufs-qcom.c"; do [ -s "$f" ] || { echo "build-ufs-match: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-ufs-match: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
INFO=$(unpack_bootimg --boot_img "$Z21" --out "$T/z21")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z21/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z21")
AV=$(avbtool info_image --image "$Z21")
# the tree: what the kernel carries (source checks, the tests/host/ufs-match-android.sh proof) and that it is the tree the Image was built from
ck "the ufs, phy and pinctrl files of the tree are committed (clean against HEAD)" "$(git -C "$KT" diff --quiet HEAD -- drivers include && [ -z "$(git -C "$KT" ls-files --others -- drivers include)" ] && echo clean || echo dirty)" clean
HEADHASH=$(git -C "$KT" rev-parse --short=12 HEAD)
BANNERS=$(zcat "$KO/Image.gz" | LC_ALL=C grep -a -o "Linux version [^ ]*" | sort -u)
ck "the Image's banner names the tree's HEAD commit g$HEADHASH, and only it" "$(printf '%s\n' "$BANNERS" | grep -c -E -e "-g$HEADHASH(-dirty)?$")/$(printf '%s\n' "$BANNERS" | grep -c .)" 1/1
ck "the Image carries the code of the tree (strings of the purge guard, the version print, the phy refusal, the two read-only prints), whatever the tree looks like now" "$(zcat "$KO/Image.gz" | LC_ALL=C grep -a -c -e 'purge refused on this device' -e 'UFS_HW_VERSION %u.%u.%u' -e 'unknown UFS host version' -e 'UFS_HOST_LEFT HCS' -e 'UFS_PHY_LINECFG_DISABLE')" 5
ck "the kernel config builds UFS, its phy and the msm8998 pinctrl in" "$(grep -c -e '^CONFIG_SCSI_UFS_QCOM=y$' -e '^CONFIG_PHY_QCOM_QMP_UFS=y$' -e '^CONFIG_PINCTRL_MSM8998=y$' "$KO/.config")" 3
ck "the kernel config is z21's (the diagnostic one: dynamic debug and gpio debug)" "$(grep -c -e '^CONFIG_DYNAMIC_DEBUG=y$' -e '^CONFIG_DEBUG_GPIO=y$' "$KO/.config")" 2
ck "the new Image.gz is not z21's (the kernel change)" "$(cmp -s "$KO/Image.gz" "$T/k.gz" && echo same || echo differs)" differs
# the dtb: z21's, with the UFS overlay merged; the decompiled difference is exactly the UFS delta, line by line
dtc -@ -I dts -O dtb -o "$T/ufs-match.dtbo" "$HERE/dt/msm8998-google-walleye-ufs-match.dtso" 2>"$T/dtc.err" || { cat "$T/dtc.err" >&2; echo "build-ufs-match: the overlay does not compile" >&2; exit 1; }
fdtoverlay -i "$T/k.dtb" -o "$NEWDTB" "$T/ufs-match.dtbo" || { echo "build-ufs-match: the overlay does not merge into z21's dtb" >&2; exit 1; }
dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null > "$T/k.dts"
dtc -I dtb -O dts "$NEWDTB" 2>/dev/null > "$T/n.dts"
diff "$T/k.dts" "$T/n.dts" > "$T/dts.diff" || true
echo "---- dts difference z21 -> ufs-match ----"; cat "$T/dts.diff"; echo "----"
ck "z21's dtb has UFS off, the new one on (the accident the gate exists for, on purpose)" "$(fdtget "$T/k.dtb" /soc@0/ufshc@1da4000 status) $(fdtget "$T/k.dtb" /soc@0/phy@1da7000 status)/$(fdtget "$NEWDTB" /soc@0/ufshc@1da4000 status) $(fdtget "$NEWDTB" /soc@0/phy@1da7000 status)" "disabled disabled/okay okay"
WANT=$(printf '%s\n' '< gpio-ranges = <0x27 0x00 0x00 0x96>;' '< gpio-reserved-ranges = <0x00 0x04 0x51 0x04>;' '< status = "disabled";' '< status = "disabled";' \
  '> gpio-ranges = <0x27 0x00 0x00 0x9a>;' '> gpio-reserved-ranges = <0x00 0x04 0x51 0x04 0x96 0x03>;' '> regulator-always-on;' '> reset-gpios = <0x27 0x99 0x01>;' '> status = "okay";' '> status = "okay";' | sort | tr '\n' '|')
ck "the dtb differs from z21's in exactly the UFS delta (10 changed lines: two statuses off to on, the reset gpio, gpio-ranges 150 to 154, the sdc2 pads reserved, l26 always on)" "$(grep '^[<>]' "$T/dts.diff" | sed 's/^\([<>]\)[[:space:]]*/\1 /' | sort | tr '\n' '|')" "$WANT"
ck "the tree and the dtb carry what Android's UFS path carries for the card (tests/host/ufs-match-android.sh)" "$(sh "$HERE/tests/host/ufs-match-android.sh" "$KT" "$NEWDTB" 2>&1 | tail -n1)" "ufs-match-android: all checks ok"
ck "the dtb keeps the touch power regulator, the stub container disabled and the gpio-reserved-ranges' first two (as z21)" "$(fdtget "$NEWDTB" /vendor-overlay-stubs status) $(fdtget -t u "$NEWDTB" /soc@0/pinctrl@3400000 gpio-reserved-ranges | cut -d' ' -f1-4)" "disabled 0 4 81 4"
ck "the overlay still merges into the new dtb" "$(DTB="$NEWDTB" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "the overlay overwrites nothing of ours (tests/host/overlay-collisions.sh)" "$(DTB="$NEWDTB" QUIET=1 sh "$HERE/tests/host/overlay-collisions.sh" | tail -n1 | cut -d' ' -f1)" pass
ck "after the overlay merge the clock users name gcc" "$(DTB="$NEWDTB" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
ck "exactly one available node names the touch power gpio" "$(DTB="$NEWDTB" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
# the image
cat "$KO/Image.gz" "$NEWDTB" > "$T/blob"
set --
while IFS= read -r pr; do if [ -n "$pr" ]; then set -- "$@" --prop "$pr"; fi; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/NOT-TO-BOOT-ufs-on-ufs-match.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/z21/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
hdr() { printf '%s\n' "$1" | grep -v -e '^kernel_size' -e 'ramdisk size' -e 'kernel size'; }
ck "the image's Image.gz is the new build's, byte for byte" "$(cmp -s "$T/uk.gz" "$KO/Image.gz" && echo same || echo differs)" same
ck "the image's dtb is the new dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$NEWDTB" && echo same || echo differs)" same
ck "the image's header (command line included) equals z21's but the sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "the image's ramdisk is z21's, byte for byte" "$(cmp -s "$T/u/ramdisk" "$T/z21/ramdisk" && echo same || echo differs)" same
ck "the image's size is z21's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "the image's AVB info equals z21's but the digest and the sizes of the kernel it hashes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e '^Original image size' -e '^VBMeta offset' -e '^ *Image Size:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the image's AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
KS=$(stat -c %s "$T/blob"); KO_=$(printf %d "$(field 'kernel load address')"); RO=$(printf %d "$(field 'ramdisk load address')")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
# the gate refuses it, by design: the image stays out of every queue
ck "the UFS-off gate FAILS this image because UFS is on in it (the gate is not weakened: it is the proof the lab's tools refuse the image)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -c1-30)" "fail walleye-ufs-off UFS is on"
[ $total -eq 0 ] || { echo "build-ufs-match: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/NOT-TO-BOOT-ufs-on-ufs-match.img"; cp "$NEWDTB" "$OUT/msm8998-google-walleye-ufs-match.dtb"
cat > "$OUT/NOT-TO-BOOT.txt" <<EON
NOT-TO-BOOT-ufs-on-ufs-match.img has UFS ON. It is built and reviewed and was never booted. It is not to be booted, and not to be put in a keeper's queue, until the author says go on UFS
(decision 2026-11-18-walleye-ufs-wait; backlog walleye-ufs-match-android-z is the build, the try is a separate item). tests/host/ufs-off-image.sh refuses it by design.
No way back from an erased Pixel 2 bootloader is known (research walleye-edl-way-back).
EON
(cd "$OUT" && sha256sum NOT-TO-BOOT-ufs-on-ufs-match.img msm8998-google-walleye-ufs-match.dtb | tee SHA256SUMS-ufs-match)
