#!/bin/sh
# Counts what the display bisect images d0, d1, d2 must be (item walleye-display-bisect-images), and writes SHA256SUMS in IMAGES_DIR:
#   check-display-bisect.sh IMAGES_DIR Z21_BOOT_IMG PARTS_DIR BOOT_DISPLAY_IMG     (PARTS_DIR: build-display-bisect-dtbs.sh's OUT_DIR; BOOT_DISPLAY_IMG: the tried display image, whose kernel Image.gz must be)
# Each image: the ramdisk, command line and size are z21's; the kernel is the display Image.gz; the dtb is z21's (d0), d1.dtb or d2.dtb; the footer verifies; tests/host/ufs-off-image.sh passes. Exit 1 on any FAIL.
set -eu
D=${1:?usage: check-display-bisect.sh IMAGES_DIR Z21_BOOT_IMG PARTS_DIR}
Z=${2:?}
P=${3:?}
B=${4:?usage: check-display-bisect.sh IMAGES_DIR Z21_BOOT_IMG PARTS_DIR BOOT_DISPLAY_IMG}
HERE=$(cd "$(dirname "$0")/.." && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
lpoco-bringup image unpack "$Z" --out "$T/z" >/dev/null
same() { cmp -s "$1" "$2" && echo same || echo differs; }
lpoco-bringup image unpack "$B" --out "$T/b" >/dev/null
head -c $(( $(stat -c %s "$T/b/kernel") - $(stat -c %s "$T/b/appended.dtb") )) "$T/b/kernel" > "$T/b.gz"
head -c $(( $(stat -c %s "$T/z/kernel") - $(stat -c %s "$T/z/appended.dtb") )) "$T/z/kernel" > "$T/z.gz"
ck "Image.gz is the display image's kernel" "$(same "$P/Image.gz" "$T/b.gz")" same
ck "Image.gz is not z21's kernel" "$(same "$P/Image.gz" "$T/z.gz")" differs
st() { fdtget "$1" "$2" status 2>/dev/null || echo absent; }
for d in d1 d2; do
  dsi=$(fdtget "$P/$d.dtb" /__symbols__ mdss_dsi0); phy=$(fdtget "$P/$d.dtb" /__symbols__ mdss_dsi0_phy)
  ck "$d.dtb: mdss, display controller okay" "$(st "$P/$d.dtb" "$(fdtget "$P/$d.dtb" /__symbols__ mdss)") $(st "$P/$d.dtb" "$(fdtget "$P/$d.dtb" /__symbols__ mdss_mdp)")" "okay okay"
  ck "$d.dtb: no panel node" "$(fdtget -l "$P/$d.dtb" "$dsi" | grep -c panel || true)" 0
  want=disabled; [ $d = d2 ] && want=okay
  ck "$d.dtb: DSI host and PHY $want" "$(st "$P/$d.dtb" "$dsi") $(st "$P/$d.dtb" "$phy")" "$want $want"
done
ck "z21 dtb, d1.dtb and d2.dtb are three different files" "$(cat "$T/z/appended.dtb" "$P/d1.dtb" "$P/d2.dtb" >/dev/null; { sha256sum < "$T/z/appended.dtb"; sha256sum < "$P/d1.dtb"; sha256sum < "$P/d2.dtb"; } | sort -u | wc -l)" 3
for i in d0 d1 d2; do
  IMG=$D/boot-$i.img
  lpoco-bringup image unpack "$IMG" --out "$T/$i" >/dev/null
  case $i in d0) W=$T/z/appended.dtb ;; *) W=$P/$i.dtb ;; esac
  ck "$i: the dtb is $(basename "$W" | sed 's/appended/z21/')" "$(same "$T/$i/appended.dtb" "$W")" same
  ck "$i: the kernel is the display Image.gz then that dtb" "$(cat "$P/Image.gz" "$W" | cmp -s - "$T/$i/kernel" && echo same || echo differs)" same
  ck "$i: the ramdisk is z21's" "$(same "$T/$i/ramdisk" "$T/z/ramdisk")" same
  ck "$i: the header (command line, offsets, os words) is z21's" "$(grep -v '^#' "$T/$i/boot.toml" | cmp -s - "$(grep -v '^#' "$T/z/boot.toml" | { cat > "$T/zb"; echo "$T/zb"; })" && echo same || echo differs)" same
  ck "$i: the size is z21's" "$(stat -c %s "$IMG")" "$(stat -c %s "$Z")"
  cp "$IMG" "$T/boot.img"
  ck "$i: the AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$i: the UFS-off gate passes (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$IMG" | cut -d' ' -f1 || true)" pass
done
[ $total -eq 0 ] || { echo "check-display-bisect: $total checks failed, no sums written" >&2; exit 1; }
(cd "$D" && sha256sum boot-d0.img boot-d1.img boot-d2.img | tee SHA256SUMS)
