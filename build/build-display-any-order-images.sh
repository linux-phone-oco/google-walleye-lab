#!/bin/sh
# Images B-r2 and B-rp2 (item walleye-display-dts-any-order; decision 2026-11-20-walleye-display-handover-reset-first): the display modules load EARLY, on a board dtb that carries the mdss reset
# as the kernel's own msm8998.dtsi has it (kernel/display-mdss-reset), not as a lab fdtput.
#   build-display-any-order-images.sh BR5_BOOT_IMG LATE_DIR DTB OUT_DIR
#   (BR5_BOOT_IMG: build-display-reset-image.sh's boot-b-r5.img; LATE_DIR: build-display-late.sh's OUT_DIR, the modules and list; DTB: build/build-display.sh's msm8998-google-walleye-display.dtb
#    from a kernel tree with kernel/display-mdss-reset applied; environment WORK, default ~/Projects/walleye-build: busybox-arm64)
# B-r2  = B-r5's kernel, command line, header words, size, modules and module list, with the built dtb and the delay 2 s (a floor: the init loads after the gadget bind, so the keeper reads the "delay 2 s reached at uptime T s" line).
# B-rp2 = B-r2 with the module list in another order: panel-samsung-s6e3fa5 loaded BEFORE msm (B-r5, L6 and L7 load it after; the panel needs only drm).
# The dtb is built from TODAY'S board dts, so against B-r5's it also has the board's newer nodes: i2c@c17a000 enabled with vibrator@5a (drv2624, no driver in this kernel) and the pinctrl labels in another form; a check bounds the difference. The lab levers are added as in every display image. The ramdisk is made the way build-display-delay-images.sh makes it:
# first with B-r5's delay and list, which must be B-r5's ramdisk byte for byte (the control). Every image is made twice and must come out identical (rebuild identical).
# Writes OUT_DIR/{boot-b-r2.img,boot-b-rp2.img,any-order.dtb,b-rp2.diff (the module list of B-r2 against B-rp2),SHA256SUMS-display-any-order}; every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
IMG=${1:?usage: build-display-any-order-images.sh BR5_BOOT_IMG LATE_DIR DTB OUT_DIR}
D=${2:?}
BUILT=${3:?}
OUT=${4:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
W=${WORK:-$HOME/Projects/walleye-build}
BB=$W/busybox-arm64
GEN=${GEN:-$D/b/usr/gen_init_cpio}
IDD=$HERE/build/initramfs-mainline/init-display-delay
TEST=$HERE/tests/host/dtb-display-reset.sh
GCC=/soc@0/clock-controller@100000
MDSS=/soc@0/display-subsystem@c900000
DSI=$MDSS/dsi@c994000
PANEL=panel-samsung-s6e3fa5.ko
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$IMG" "$D/display-modules.list" "$BUILT" "$BB" "$GEN" "$IDD"; do [ -s "$f" ] || [ -x "$f" ] || { echo "build-display-any-order-images: missing $f" >&2; exit 1; }; done
SUM=$(cd "$(dirname "$IMG")" && grep " boot-b-r5.img\$" SHA256SUMS-display-reset | cut -d" " -f1)
ck "the control is B-r5: its sha256 is the one recorded (SHA256SUMS-display-reset)" "$(sha256sum "$IMG" | cut -d" " -f1)" "$SUM"
ROOT=$D/modroot
same() { cmp -s "$1" "$2" && echo same || echo differs; }
hdr() { grep -v '^#' "$1/boot.toml"; }
lpoco-bringup image unpack "$IMG" --out "$T/r5" >/dev/null
mkdir "$T/r5rd" && (cd "$T/r5rd" && xz -dc "$T/r5/ramdisk" | cpio -id 2>/dev/null || true)
BIN=$T/r5rd/bin/walleye-stripes
ck "B-r5's ramdisk holds the drawer, an aarch64 static executable" "$(file -b "$BIN" | grep -c 'ARM aarch64.*statically linked\|ARM aarch64.*static-pie linked')" 1
ck "B-r5's ramdisk init is init-display-delay, its delay file says 5" "$(same "$T/r5rd/init" "$IDD") $(cat "$T/r5rd/display-delay")" "same 5"
ck "the module list in the ramdisk is the late directory's" "$(same "$T/r5rd/display-modules.list" "$D/display-modules.list")" same
# the other order: the panel's insmod line moved before msm's
grep -v "/$PANEL\$" "$D/display-modules.list" | awk -v p="$(grep "/$PANEL\$" "$D/display-modules.list")" '/\/msm\.ko$/ { print p } { print }' > "$T/list-p"
ck "the other list has the same lines, a different order" "$(sort "$T/list-p" | md5sum | cut -d' ' -f1) $(same "$T/list-p" "$D/display-modules.list")" "$(sort "$D/display-modules.list" | md5sum | cut -d' ' -f1) differs"
ck "in the other list the panel comes before msm, in the first after it" "$(grep -n '/msm\.ko$\|/'$PANEL'$' "$T/list-p" | sed 's/.*\///' | tr '\n' ' ') $(grep -n '/msm\.ko$\|/'$PANEL'$' "$D/display-modules.list" | sed 's/.*\///' | tr '\n' ' ')" "$PANEL msm.ko  msm.ko $PANEL "
PDEPS=$(modinfo -F depends "$(find "$ROOT" -name "$PANEL")" | tr ',' ' ')
ck "every module the panel needs is loaded before it in the other list (modinfo depends: $PDEPS)" "$(for d in $PDEPS; do awk -v d="/$d.ko" -v p="/$PANEL" 'index($0,p) {exit} index($0,d) {f=1} END {print f+0}' "$T/list-p"; done | sort -u | tr '\n' ' ')" "1 "
spec() { # DELAYFILE LIST
  for dr in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $dr 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $IDD 755 0 0"
  echo "file /etc/udhcpd.conf $HERE/build/initramfs-mainline/udhcpd-z14.conf 644 0 0"
  echo "file /display-modules.list $2 644 0 0"
  echo "file /display-delay $1 644 0 0"
  echo "file /bin/walleye-stripes $BIN 755 0 0"
  (cd "$ROOT" && find lib -type d -printf 'dir /%p 755 0 0\n')
  (cd "$ROOT" && find lib -type f -printf "file /%p $ROOT/%p 644 0 0\n")
}
pack() { spec "$@" > "$T/spec"; "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"; xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio"; }
printf '5\n' > "$T/delay5"; printf '2\n' > "$T/delay2"
pack "$T/delay5" "$D/display-modules.list" > "$T/control.xz"
ck "the ramdisk made with B-r5's delay 5 and list is B-r5's, byte for byte (control)" "$(same "$T/control.xz" "$T/r5/ramdisk")" same
# the dtb: the built board dtb, clock_gcc at the real gcc
# the levers every display image's dtb carries beside the kernel's own tree (each its own build script's, repeated here only as one fdtput each): clock_gcc at the real gcc (build-display-image.sh),
# the dsi host's vdd-supply = l1a (build-display-late-l3-dtb.sh), the label mdss_dsi0 on an empty stub so the vendor overlay's qcom,platform-reset-gpio never reaches the real dsi (build-display-late-l5-dtb.sh)
STUBS=/vendor-overlay-stubs
make_dtb() {
  cp "$BUILT" "$1"
  fdtput -t s "$1" /__symbols__ clock_gcc "$GCC"
  fdtput -t x "$1" "$DSI" vdd-supply "$(fdtget -t x "$1" "$(fdtget "$1" /__symbols__ vreg_l1a_0p875)" phandle)"
  MAXPH=$(dtc -I dtb -O dts -q "$1" 2>/dev/null | sed -n 's/^[[:space:]]*phandle = <0x\([0-9a-f]*\)>;$/\1/p' | while read -r h; do echo $((0x$h)); done | sort -n | tail -n1)
  fdtput -c "$1" "$STUBS/mdss_dsi0"
  fdtput -t u "$1" "$STUBS/mdss_dsi0" phandle $((MAXPH+1))
  fdtput -t s "$1" /__symbols__ mdss_dsi0 "$STUBS/mdss_dsi0"
}
ck "the built dtb has clock_gcc at the empty stub (the fix is needed)" "$(fdtget "$BUILT" /__symbols__ clock_gcc)" /vendor-overlay-stubs/clock_gcc
ck "the test fails on B-r5's dtb with its resets property deleted (control)" "$(lpoco-bringup image unpack "$IMG" --out "$T/c" >/dev/null; fdtput -d "$T/c/appended.dtb" "$MDSS" resets; DTB="$T/c/appended.dtb" sh "$TEST" | cut -d' ' -f1 || true)" fail
make_dtb "$T/any-order.dtb"; make_dtb "$T/any-order-again.dtb"
ck "the dtb made twice is the same" "$(same "$T/any-order.dtb" "$T/any-order-again.dtb")" same
ck "clock_gcc is the real gcc in the image dtb" "$(fdtget "$T/any-order.dtb" /__symbols__ clock_gcc)" "$GCC"
ck "the dsi has vdd-supply and vdda-supply, as B-r5's dtb has them" "$(fdtget -p "$T/any-order.dtb" "$DSI" | grep -c "^vdd-supply\$\|^vdda-supply\$") $(fdtget -t x "$T/any-order.dtb" "$DSI" vdd-supply) $(fdtget -t x "$T/r5/appended.dtb" "$DSI" vdd-supply)" "2 $(fdtget -t x "$T/r5/appended.dtb" "$DSI" vdd-supply) $(fdtget -t x "$T/r5/appended.dtb" "$DSI" vdd-supply)"
ck "the test passes on the image dtb (resets = <&mmcc MDSS_BCR> from the kernel's own msm8998.dtsi)" "$(DTB="$T/any-order.dtb" sh "$TEST" | cut -d' ' -f1 || true)" pass
ck "the dtb's difference from B-r5's: /soc@0 has the same children, the only new node is vibrator@5a under i2c@c17a000" "$(fdtget -l "$T/any-order.dtb" /soc@0 | sort | md5sum | cut -d' ' -f1) $(fdtget -l "$T/any-order.dtb" /soc@0/i2c@c17a000 | tr '\n' ' ') $(fdtget -l "$T/r5/appended.dtb" /soc@0/i2c@c17a000 | tr '\n' ' ')" "$(fdtget -l "$T/r5/appended.dtb" /soc@0 | sort | md5sum | cut -d' ' -f1) vibrator@5a  "
head -c $(( $(stat -c %s "$T/r5/kernel") - $(stat -c %s "$T/r5/appended.dtb") )) "$T/r5/kernel" > "$T/Image.gz"
make_one() { # DELAYFILE LIST DIR
  d=$3; mkdir -p "$d"
  pack "$1" "$2" > "$d/ramdisk.xz"
  lpoco-bringup image swap "$IMG" --kernel "$T/Image.gz" --dtb "$T/any-order.dtb" --ramdisk "$d/ramdisk.xz" --out "$d/boot.img" >/dev/null
}
for v in "r2:$T/delay2:$D/display-modules.list" "rp2:$T/delay2:$T/list-p"; do
  n=${v%%:*}; rest=${v#*:}; df=${rest%%:*}; ls=${rest#*:}
  echo "--- B-$n"
  make_one "$df" "$ls" "$T/$n"; make_one "$df" "$ls" "$T/again-$n"
  I=$T/$n/boot.img
  ck "B-$n: made twice, the two images are identical (rebuild identical)" "$(same "$I" "$T/again-$n/boot.img")" same
  lpoco-bringup image unpack "$I" --out "$T/u$n" >/dev/null
  ck "B-$n: the kernel is B-r5's Image.gz then the image dtb, byte for byte" "$(cat "$T/Image.gz" "$T/any-order.dtb" | cmp -s - "$T/u$n/kernel" && echo same || echo differs)" same
  ck "B-$n: the appended dtb is the image dtb" "$(same "$T/u$n/appended.dtb" "$T/any-order.dtb")" same
  ck "B-$n: the command line and header words are B-r5's" "$([ "$(hdr "$T/u$n")" = "$(hdr "$T/r5")" ] && echo same || echo differs)" same
  ck "B-$n: the size is B-r5's" "$(stat -c %s "$I")" "$(stat -c %s "$IMG")"
  ck "B-$n: the ramdisk is the packed one, byte for byte" "$(same "$T/u$n/ramdisk" "$T/$n/ramdisk.xz")" same
  mkdir "$T/rd$n" && (cd "$T/rd$n" && xz -dc "$T/u$n/ramdisk" | cpio -id 2>/dev/null || true)
  ck "B-$n: /init is init-display-delay, /display-delay says 2" "$(same "$T/rd$n/init" "$IDD") $(cat "$T/rd$n/display-delay")" "same 2"
  ck "B-$n: the drawer is B-r5's" "$(same "$T/rd$n/bin/walleye-stripes" "$BIN")" same
  ck "B-$n: the module list is the one named" "$(same "$T/rd$n/display-modules.list" "$ls")" same
  (cd "$T/rd$n" && find . | sort) > "$T/f-$n"; (cd "$T/r5rd" && find . | sort) > "$T/f-r5"
  ck "B-$n: the ramdisk holds the same names as B-r5's" "$(same "$T/f-$n" "$T/f-r5")" same
  mkdir "$T/v$n" && cp "$I" "$T/v$n/boot.img"
  ck "B-$n: the AVB footer verifies" "$(avbtool verify_image --image "$T/v$n/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "B-$n: the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$I" >/dev/null 2>&1 && echo pass || echo fail)" pass
  ck "B-$n: the UFS-off tests pass (tests/host/ufs-off-image.sh)" "$(sh "$HERE/tests/host/ufs-off-image.sh" "$I" | cut -d' ' -f1 || true)" pass
  ck "B-$n: the panel reset gpio has one claimant in the image's dtb" "$(DTB="$T/u$n/appended.dtb" sh "$HERE/tests/host/display-reset-gpio-single-claimant.sh" | cut -d' ' -f1 || true)" pass
  ck "B-$n: the stock overlay still merges into the dtb (tests/host/dtbo-overlay-merges.sh)" "$(DTB="$T/any-order.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
  ck "B-$n: the touch power gpio keeps one claimant (tests/host/touch-gpio-single-claimant.sh)" "$(DTB="$T/any-order.dtb" sh "$HERE/tests/host/touch-gpio-single-claimant.sh" | cut -d' ' -f1)" pass
done
ck "B-r2 and B-rp2 differ only in the ramdisk" "$(same "$T/ur2/kernel" "$T/urp2/kernel") $(same "$T/ur2/ramdisk" "$T/urp2/ramdisk")" "same differs"
diff -u "$D/display-modules.list" "$T/list-p" | sed '1,2d' > "$T/b-rp2.diff" || true
[ $total -eq 0 ] || { echo "build-display-any-order-images: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"; rm -f "$OUT/boot-b-r2.img" "$OUT/boot-b-rp2.img" "$OUT/any-order.dtb" "$OUT/b-rp2.diff" "$OUT/SHA256SUMS-display-any-order"
cp "$T/r2/boot.img" "$OUT/boot-b-r2.img"; cp "$T/rp2/boot.img" "$OUT/boot-b-rp2.img"; cp "$T/any-order.dtb" "$OUT/any-order.dtb"; cp "$T/b-rp2.diff" "$OUT/b-rp2.diff"
(cd "$OUT" && sha256sum boot-b-r2.img boot-b-rp2.img any-order.dtb b-rp2.diff | tee SHA256SUMS-display-any-order)
