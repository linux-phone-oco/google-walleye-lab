#!/bin/sh
# y1 and y2 (backlog walleye-our-dts-gpio-reserved): OUR dtb, now with pinctrl@3400000 gpio-reserved-ranges = <0 4>, <81 4> (the property whose absence killed every dtb of ours,
# research walleye-fdt-bisect-x-tried), under ml6a's kernel (our Image.gz, no earlycon, ml6a's header and command line). Host only; nothing touches the phone.
#   y1  ml5a's ramdisk: an init that marks /dev/kmsg and reboots at once. Expected alive: the phone is back as Android in <= 66 s.
#   y2  pk5l's ramdisk: the mainline gadget init (ACM shell on ttyGS0 + mass_storage window, watchdog petted, reboot -f at the end). Expected: 1d6b:0104 on the host within ~25 s of the jump.
#   build-y.sh WORK_DIR ML6A_BOOT_IMG ML5A_BOOT_IMG PK5L_BOOT_IMG OUT_DIR
# The dtb is compiled here from dt/ the way build-kernel.sh does (DTC_FLAGS=-@, the kernel tree's own make, out of tree in WORK_DIR/out-y) - only the dtb, the Image.gz is ml6a's, byte for byte.
# Writes OUT_DIR/{boot-y1.img,boot-y2.img,msm8998-google-walleye-y.dtb,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL, nothing written into OUT_DIR then).
# Environment: KERNEL_TREE (default WORK_DIR/linux-v7.3), DTBO (default WORK_DIR/control/dtbo.img; the overlay-merge check), CFG (the .config of ml6a's Image.gz, default WORK_DIR/out-ml/.config).
# NB: the f dtb (dt/msm8998-google-walleye-f.dts) includes the walleye dts and so carries the property too; build-fdt-bisect.sh (x1..xa) needs OURS_DTB from the dts BEFORE this property.
set -eu
W=${1:?usage: build-y.sh WORK_DIR ML6A_BOOT_IMG ML5A_BOOT_IMG PK5L_BOOT_IMG OUT_DIR}
ML6A=${2:?}
ML5A=${3:?}
PK5L=${4:?}
OUT=${5:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
K=${KERNEL_TREE:-$W/linux-v7.3}
export DTBO=${DTBO:-$W/control/dtbo.img}
I1=$HERE/build/initramfs-mainline/init-ml5a
I2=$HERE/build/initramfs-mainline/init-pk5l
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT; STG=$T/stage; mkdir -p "$STG"
for f in "$ML6A" "$ML5A" "$PK5L" "$I1" "$I2" "$HERE/dt/msm8998-google-walleye.dts" "$K/Makefile" "$W/busybox-x86_64" "$DTBO"; do [ -s "$f" ] || { echo "build-y: missing $f" >&2; exit 1; }; done
# split KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
split() {
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-y: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
# 1. our dtb from dt/, compiled by the kernel tree (the same copy-in and Makefile lines as build-kernel.sh)
QCOM=$K/arch/arm64/boot/dts/qcom
cp "$HERE/dt/msm8998-google-walleye-overlay-stubs.dtsi" "$HERE/dt/msm8998-google-walleye.dts" "$HERE/dt/msm8998-google-walleye-f.dts" "$QCOM/"
grep -q 'msm8998-google-walleye.dtb' "$QCOM/Makefile" || { echo "build-y: the kernel tree's Makefile lacks the walleye dtb line (run build-kernel.sh once)" >&2; exit 1; }
mkdir -p "$W/out-y/b"
M="make -C $K ARCH=arm64 LLVM=1 O=$W/out-y/b"
$M defconfig >/dev/null
DTC_FLAGS=-@ $M -j"$(nproc)" qcom/msm8998-google-walleye.dtb >"$T/dtb-make.log" 2>&1 || { cat "$T/dtb-make.log" >&2; exit 1; }
cp "$W/out-y/b/arch/arm64/boot/dts/qcom/msm8998-google-walleye.dtb" "$T/y.dtb"
# 2. the source images (their hashes are printed: the recipe's inputs)
for f in "$ML6A" "$ML5A" "$PK5L" "$DTBO"; do echo "input $(h "$f")  $f"; done
INFO=$(unpack_bootimg --boot_img "$ML6A" --out "$T/m6")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
unpack_bootimg --boot_img "$ML5A" --out "$T/m5" >/dev/null
unpack_bootimg --boot_img "$PK5L" --out "$T/pk" >/dev/null
split "$T/m6/kernel" "$T/k"
PSIZE=$(stat -c %s "$ML6A")
AV=$(avbtool info_image --image "$ML6A")
cat "$T/k.gz" "$T/y.dtb" > "$T/blob"
hdr() { printf "%s\n" "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size'; }
PY=/soc@0/pinctrl@3400000
gv() { fdtget -p "$1" "$PY" >/dev/null 2>&1 || { echo "no-node"; return; }; fdtget -t u "$1" "$PY" gpio-reserved-ranges 2>/dev/null || echo none; }
# the dtb itself
ck "y.dtb: gpio-reserved-ranges is <0 4 81 4>" "$(gv "$T/y.dtb")" "0 4 81 4"
ck "ml6a's dtb has no gpio-reserved-ranges (the cause)" "$(gv "$T/k.dtb")" none
dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null | grep -v 'gpio-reserved-ranges' | sed -E 's/phandle = <0x[0-9a-f]+>/phandle/' > "$T/k.dts"
dtc -I dtb -O dts "$T/y.dtb" 2>/dev/null | grep -v 'gpio-reserved-ranges' | sed -E 's/phandle = <0x[0-9a-f]+>/phandle/' > "$T/y.dts"
ck "y.dtb differs from ml6a's dtb in no line but gpio-reserved-ranges" "$(sha256sum < "$T/y.dts")" "$(sha256sum < "$T/k.dts")"
ck "y.dtb differs from ml6a's dtb in exactly one property line" "$(dtc -I dtb -O dts "$T/y.dtb" 2>/dev/null | grep -c 'gpio-reserved-ranges') / $(dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null | grep -c 'gpio-reserved-ranges')" "1 / 0"
ck "y.dtb keeps qcom,msm-id <0x124 0x20001>" "$(fdtget -t x "$T/y.dtb" / qcom,msm-id)" "124 20001"
ck "y.dtb has __symbols__ (the overlay merge needs them)" "$(fdtget -l "$T/y.dtb" / | grep -cx __symbols__)" 1
ck "the dtbo overlay merges into y.dtb" "$(DTB="$T/y.dtb" sh "$HERE/tests/host/dtbo-overlay-merges.sh" | cut -d' ' -f1)" pass
ck "the dtb source in dt/ names the property" "$(grep -c '^	gpio-reserved-ranges = <0 4>, <81 4>;$' "$HERE/dt/msm8998-google-walleye.dts")" 1
# 3. the two images
build_one() { # NAME RAMDISK_IMG_DIR SOURCE_INIT
N=$1; RD=$2; SI=$3
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$STG/boot-$N.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$RD" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$N")
split "$T/u$N/kernel" "$T/uk$N"
ck "$N Image.gz is ml6a's, byte for byte" "$(cmp -s "$T/uk$N.gz" "$T/k.gz" && echo same || echo differs)" same
ck "$N dtb is y.dtb, byte for byte" "$(cmp -s "$T/uk$N.dtb" "$T/y.dtb" && echo same || echo differs)" same
ck "$N header (offsets, page, os word, command line) equals ml6a's but the two sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "$N command line is ml6a's: no earlycon, no console=" "$(printf "%s\n" "$U" | sed -n 's/^command line args: *//p' | head -n1 | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
ck "$N command line equals ml6a's" "$(printf "%s\n" "$U" | sed -n 's/^command line args: *//p' | head -n1)" "$(field 'command line args')"
ck "$N ramdisk bytes equal the source image's" "$(cmp -s "$T/u$N/ramdisk" "$RD" && echo same || echo differs)" same
mkdir "$T/x$N" && (cd "$T/x$N" && xz -dc "$T/u$N/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "$N init in the ramdisk is $(basename "$SI")" "$(cmp -s "$T/x$N/init" "$SI" && echo same || echo differs)" same
ck "$N ramdisk has /init, /bin/busybox, /dev/console" "$(xz -dc "$T/u$N/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e bin/busybox -e dev/console)" 3
ck "$N busybox in it is aarch64 static" "$(file -b "$T/x$N/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
ck "$N init parses (busybox ash -n)" "$("$W/busybox-x86_64" ash -n "$SI" 2>&1 || echo syntax-error)" ""
ck "$N size is ml6a's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "$N AVB info equals ml6a's but the digest" "$(avbtool info_image --image "$IMG" | grep -v '^ *Digest:')" "$(printf '%s\n' "$AV" | grep -v '^ *Digest:')"
mkdir "$T/v$N" && cp "$IMG" "$T/v$N/boot.img"
ck "$N AVB footer verifies" "$(avbtool verify_image --image "$T/v$N/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
}
build_one y1 "$T/m5/ramdisk" "$I1"
build_one y2 "$T/pk/ramdisk" "$I2"
ck "y1 and y2 differ in the ramdisk only (same Image.gz + dtb blob)" "$(cmp -s "$T/uy1/kernel" "$T/uy2/kernel" && echo same || echo differs)" same
# the gadget init's needs from OUR kernel (the config of ml6a's Image.gz)
CFG=${CFG:-$W/out-ml/.config}; [ -s "$CFG" ] || { echo "build-y: missing $CFG" >&2; exit 1; }
CFG=${CFG:-$W/out-ml/.config}
for k in USB_GADGET USB_CONFIGFS USB_CONFIGFS_ACM USB_CONFIGFS_MASS_STORAGE USB_DWC3 USB_DWC3_QCOM PHY_QCOM_QUSB2 QCOM_WDT CONFIGFS_FS DEVTMPFS RD_XZ BLK_DEV_INITRD; do
  ck "kernel config has CONFIG_$k" "$(grep -cE "^CONFIG_$k=[ym]$" "$CFG")" 1
done
[ $total -eq 0 ] || { echo "build-y: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$STG/boot-y1.img" "$STG/boot-y2.img" "$OUT/"; cp "$T/y.dtb" "$OUT/msm8998-google-walleye-y.dtb"
(cd "$OUT" && sha256sum boot-y1.img boot-y2.img msm8998-google-walleye-y.dtb | tee SHA256SUMS)
