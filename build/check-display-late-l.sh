#!/bin/sh
# Counts what the late-probe images L1 and L2 must be (item walleye-display-late-probe-l-images), writes l1.diff and SHA256SUMS-display-late-l in IMAGES_DIR:
#   check-display-late-l.sh IMAGES_DIR LATE_BOOT_IMG PARTS_DIR     (PARTS_DIR: build-display-late-l-dtb.sh's OUT_DIR; LATE_BOOT_IMG: images-display-late/boot-display-late.img)
# Each image is the late-probe image with exactly one thing changed: L1 the command line (iommu.passthrough=1 added), L2 the dtb appended to the kernel; kernel Image.gz, ramdisk, header words, size are the late image's;
# the AVB footer verifies and tests/host/ufs-off-image.sh passes. Exit 1 on any FAIL.
set -eu
D=${1:?usage: check-display-late-l.sh IMAGES_DIR LATE_BOOT_IMG PARTS_DIR}
L=${2:?}
P=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
same() { cmp -s "$1" "$2" && echo same || echo differs; }
lpoco-bringup image unpack "$L" --out "$T/late" >/dev/null
hdr() { grep -v '^#' "$1/boot.toml" | grep -v '^cmdline'; }
cmd() { grep '^cmdline' "$1/boot.toml"; }
for i in l1 l2; do
  IMG=$D/boot-$i.img
  lpoco-bringup image unpack "$IMG" --out "$T/$i" >/dev/null
  ck "$i: the ramdisk is the late image's" "$(same "$T/$i/ramdisk" "$T/late/ramdisk")" same
  ck "$i: the header words other than the command line are the late image's" "$([ "$(hdr "$T/$i")" = "$(hdr "$T/late")" ] && echo same || echo differs)" same
  ck "$i: the size is the late image's" "$(stat -c %s "$IMG")" "$(stat -c %s "$L")"
  cp "$IMG" "$T/boot.img"
  ck "$i: the AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$i: the UFS-off gate passes (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
done
ck "l1: the dtb and the kernel are the late image's" "$(same "$T/l1/kernel" "$T/late/kernel")" same
ck "l1: the command line is the late image's with ' iommu.passthrough=1' added at its end" "$(cmd "$T/l1")" "$(cmd "$T/late" | sed 's/"$/ iommu.passthrough=1"/')"
ck "l1: the command line differs from the late image's" "$([ "$(cmd "$T/l1")" = "$(cmd "$T/late")" ] && echo same || echo differs)" differs
ck "l2: the command line is the late image's" "$([ "$(cmd "$T/l2")" = "$(cmd "$T/late")" ] && echo same || echo differs)" same
ck "l2: the dtb is l2.dtb" "$(same "$T/l2/appended.dtb" "$P/l2.dtb")" same
ck "l2: the diff in IMAGES_DIR is the parts' diff" "$(same "$D/l2.diff" "$P/l2.diff")" same
head -c $(( $(stat -c %s "$T/late/kernel") - $(stat -c %s "$T/late/appended.dtb") )) "$T/late/kernel" > "$T/Image.gz"
ck "l2: the kernel is the late image's Image.gz then that dtb" "$(cat "$T/Image.gz" "$P/l2.dtb" | cmp -s - "$T/l2/kernel" && echo same || echo differs)" same
MDSS=$(fdtget "$T/late/appended.dtb" /__symbols__ mdss); MDP=$(fdtget "$T/late/appended.dtb" /__symbols__ mdss_mdp); DSI=$(fdtget "$T/late/appended.dtb" /__symbols__ mdss_dsi0); PHY=$(fdtget "$T/late/appended.dtb" /__symbols__ mdss_dsi0_phy)
st() { fdtget "$1" "$2" status 2>/dev/null || echo absent; }
panel() { fdtget -l "$1" "$DSI" | grep -c '^panel@0$' || true; }
ck "l1 (read back from its image): iommus kept, mdss, controller, DSI host and PHY okay, panel node there" "$(fdtget "$T/l1/appended.dtb" "$MDSS" iommus) $(for n in "$MDSS" "$MDP" "$DSI" "$PHY"; do st "$T/l1/appended.dtb" "$n"; done | tr "\n" " ")$(panel "$T/l1/appended.dtb")" "142 0 okay okay okay okay 1"
ck "l2 (read back from its image): no iommus on mdss, mdss, controller, DSI host and PHY okay, panel node there" "$(fdtget -p "$T/l2/appended.dtb" "$MDSS" | grep -c "^iommus$" || true) $(for n in "$MDSS" "$MDP" "$DSI" "$PHY"; do st "$T/l2/appended.dtb" "$n"; done | tr "\n" " ")$(panel "$T/l2/appended.dtb")" "0 okay okay okay okay 1"
(cd "$P" && sha256sum -c SHA256SUMS-display-late-l-parts >/dev/null 2>&1) || { echo "check-display-late-l: the parts do not match their sums" >&2; exit 1; }
{ echo "--- late command line"; echo "+++ l1 command line"; echo "-$(cmd "$T/late")"; echo "+$(cmd "$T/l1")"; } > "$T/l1.diff"
ck "l1: the diff has one line out and one in" "$(grep -c '^[-+]cmdline' "$T/l1.diff")" 2
[ $total -eq 0 ] || { echo "check-display-late-l: $total checks failed, no sums written" >&2; exit 1; }
cp "$T/l1.diff" "$D/l1.diff"
(cd "$D" && sha256sum boot-l1.img boot-l2.img l1.diff l2.diff | tee SHA256SUMS-display-late-l)
