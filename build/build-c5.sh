#!/bin/sh
# c5 and c6 (backlog walleye-build-c5): ONE image, the command line decides. c4's dtb, header, size and AVB footer; the kernel's charger driver (CONFIG_CHARGER_QCOM_SMB2) a MODULE instead of built in
# (rradc stays built in), the module in the ramdisk as /lib/modules/qcom_smbx.ko, and an init (initramfs-mainline/init-c5 = z6's) that loads it ONLY when the command line says walleye.charger_after=SECONDS,
# SECONDS after the gadget bound. c5: c4's command line (no argument: the module stays unloaded). c6: the same image with ONLY the command line + " walleye.charger_after=40".
#   build-c5.sh WORK_DIR C4_BOOT_IMG KERNEL_OUT_DIR OUT_DIR
# KERNEL_OUT_DIR is job-c5.sh's out-c5 (Image.gz, qcom_smbx.ko, .config). Needs no phone.
# Writes OUT_DIR/{boot-c5.img,boot-c6.img,initramfs-c5.cpio.xz,SHA256SUMS-c5}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
# Environment: BASE_CFG (c2's kernel config, default $WORK_DIR/out-c2/.config), ARG (the c6 argument, default walleye.charger_after=40).
set -eu
W=${1:?usage: build-c5.sh WORK_DIR C4_BOOT_IMG KERNEL_OUT_DIR OUT_DIR}
C4=${2:?}
KO=${3:?}
OUT=${4:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BASE_CFG=${BASE_CFG:-$W/out-c2/.config}
ARG=${ARG:-walleye.charger_after=40}
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
I=$HERE/initramfs-mainline/init-c5
OLD=$HERE/initramfs-mainline/init-z6
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$C4" "$I" "$OLD" "$BASE_CFG" "$KO/Image.gz" "$KO/qcom_smbx.ko" "$KO/.config"; do [ -s "$f" ] || { echo "build-c5: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-c5: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(sha256sum "$C4" | cut -c1-16)  $C4"
INFO=$(unpack_bootimg --boot_img "$C4" --out "$T/c4")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/c4/kernel" "$T/k"
PSIZE=$(stat -c %s "$C4")
AV=$(avbtool info_image --image "$C4")
OLDCMD=$(field 'command line args')
# the kernel and its module
ck ".config has CHARGER_QCOM_SMB2=m" "$(grep -c '^CONFIG_CHARGER_QCOM_SMB2=m$' "$KO/.config")" 1
ck ".config has QCOM_SPMI_RRADC=y (rradc stays built in)" "$(grep -c '^CONFIG_QCOM_SPMI_RRADC=y$' "$KO/.config")" 1
ck ".config differs from c2's in the one charger line" "$(diff "$BASE_CFG" "$KO/.config" | grep '^[<>]' | tr '\n' '|')" "< CONFIG_CHARGER_QCOM_SMB2=y|> CONFIG_CHARGER_QCOM_SMB2=m|"
ck "the new Image.gz is not c4's (the kernel change)" "$(cmp -s "$KO/Image.gz" "$T/k.gz" && echo same || echo differs)" differs
ck "Image.gz no longer holds the charger driver (it is the module)" "$(zcat "$KO/Image.gz" | grep -ac qcom-smbx-charger | sed 's/^[1-9][0-9]*$/some/')" 0
ck "the module holds the charger driver (compatible qcom,pmi8998-charger)" "$(grep -ac 'qcom,pmi8998-charger' "$KO/qcom_smbx.ko" | sed 's/^[1-9][0-9]*$/some/')" some
REL=$(zcat "$KO/Image.gz" | strings -a | sed -n 's/^Linux version \([^ ]*\) .*/\1/p' | head -n1)
ck "the module's vermagic names the kernel's release ($REL)" "$(modinfo -F vermagic "$KO/qcom_smbx.ko" | cut -d' ' -f1)" "$REL"
ck "the module's dependencies are built in (none to load first)" "$(modinfo -F depends "$KO/qcom_smbx.ko")" ""
# the ramdisk: packed as z6's, c4's byte for byte with init-z6; then with the module and init-c5
pack() { # INIT OUT_XZ WITH_MODULE(yes|no)
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    [ "$3" = yes ] && { echo "dir /lib 755 0 0"; echo "dir /lib/modules 755 0 0"; }
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
    echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $1 755 0 0"
    [ "$3" = yes ] && echo "file /lib/modules/qcom_smbx.ko $KO/qcom_smbx.ko 644 0 0"
  } > "$T/spec"
  "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"
  xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$2"
}
pack "$OLD" "$T/old.xz" no
ck "packing init-z6 gives c4's ramdisk byte for byte (the packing is the same)" "$(cmp -s "$T/old.xz" "$T/c4/ramdisk" && echo same || echo differs)" same
ck "init-c5 differs from init-z6 by added lines and one comment line only" "$(diff "$OLD" "$I" | grep -c '^<')" 1
ck "init-c5 differs from init-z6 in two hunks only: line 2 (comment) and lines added right after the bind line" "$(diff "$OLD" "$I" | grep "^[0-9]" | sed "s/,[0-9]*\$//" | tr "\n" " ")" "2c2 $(grep -n "gadget bound to" "$OLD" | cut -d: -f1)a$(( $(grep -n "gadget bound to" "$OLD" | cut -d: -f1)+1 )) "
ck "init-c5 parses (sh -n, a syntax check only)" "$(sh -n "$I" && echo yes || echo no)" yes
ck "the loader selftest passes" "$(sh "$HERE/../tests/host/c5-loader-selftest.sh" "$I" | tail -n1)" "pass c5-loader-selftest"
mkdir -p "$T/o"; pack "$I" "$T/o/initramfs-c5.cpio.xz" yes
# the images: c5 with c4's command line, c6 with the argument added
mkimg() { # NAME CMDLINE
  nm=$1; cl=$2
  set --
  while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
  cat "$KO/Image.gz" "$T/k.dtb" > "$T/blob"
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$cl" \
    --kernel "$T/blob" --ramdisk "$T/o/initramfs-c5.cpio.xz" -o "$T/o/boot-$nm.img"
  sh "$(dirname "$0")/ufs-gate.sh" "$T/o/boot-$nm.img" || exit 1
  avbtool add_hash_footer --image "$T/o/boot-$nm.img" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
}
ck "c4's command line has no walleye.charger_after yet" "$(printf '%s\n' "$OLDCMD" | grep -c walleye.charger_after)" 0
mkimg c5 "$OLDCMD"
mkimg c6 "$OLDCMD $ARG"
hdr() { printf '%s\n' "$1" | grep -v -e '^kernel_size' -e '^ramdisk size' -e '^kernel size' -e '^command line args'; }
avi() { avbtool info_image --image "$1" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:'; }
for n in c5 c6; do
  IMG=$T/o/boot-$n.img
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$n")
  split "$T/u$n/kernel" "$T/uk$n"
  ck "$n Image.gz is the new build's, byte for byte" "$(cmp -s "$T/uk$n.gz" "$KO/Image.gz" && echo same || echo differs)" same
  ck "$n dtb is c4's, byte for byte" "$(cmp -s "$T/uk$n.dtb" "$T/k.dtb" && echo same || echo differs)" same
  ck "$n ramdisk is the packed one, byte for byte" "$(cmp -s "$T/u$n/ramdisk" "$T/o/initramfs-c5.cpio.xz" && echo same || echo differs)" same
  ck "$n ramdisk is not c4's" "$(cmp -s "$T/u$n/ramdisk" "$T/c4/ramdisk" && echo same || echo differs)" differs
  ck "$n header equals c4's but the sizes and the command line" "$(hdr "$U")" "$(hdr "$INFO")"
  ck "$n size is c4's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  ck "$n AVB info equals c4's but the digest and the image sizes" "$(avi "$IMG")" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
  mkdir "$T/v$n" && cp "$IMG" "$T/v$n/boot.img"
  ck "$n AVB footer verifies" "$(avbtool verify_image --image "$T/v$n/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$n ramdisk lists the module and init, and nothing else new" "$(xz -dc "$T/u$n/ramdisk" | cpio -it 2>/dev/null | sort | tr '\n' ' ')" "bin bin/busybox bin/sh dev dev/console dev/null etc init lib lib/modules lib/modules/qcom_smbx.ko proc run sbin sys tmp "
  mkdir "$T/x$n" && (cd "$T/x$n" && xz -dc "$T/u$n/ramdisk" | cpio -id 2>/dev/null || true)  # the device nodes cannot be made as a user: their files are the ones read back
  ck "$n ramdisk's /lib/modules/qcom_smbx.ko is the built module" "$(cmp -s "$T/x$n/lib/modules/qcom_smbx.ko" "$KO/qcom_smbx.ko" && echo same || echo differs)" same
  ck "$n ramdisk's /init is init-c5" "$(cmp -s "$T/x$n/init" "$I" && echo same || echo differs)" same
done
ck "c5's command line is c4's" "$(unpack_bootimg --boot_img "$T/o/boot-c5.img" --out "$T/q5" | sed -n 's/^command line args: *//p')" "$OLDCMD"
ck "c6's command line is c4's plus ' $ARG'" "$(unpack_bootimg --boot_img "$T/o/boot-c6.img" --out "$T/q6" | sed -n 's/^command line args: *//p')" "$OLDCMD $ARG"
ck "c5 and c6 kernels are the same bytes" "$(cmp -s "$T/q5/kernel" "$T/q6/kernel" && echo same || echo differs)" same
ck "c5 and c6 ramdisks are the same bytes" "$(cmp -s "$T/q5/ramdisk" "$T/q6/ramdisk" && echo same || echo differs)" same
[ $total -eq 0 ] || { echo "build-c5: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$T/o/boot-c5.img" "$T/o/boot-c6.img" "$T/o/initramfs-c5.cpio.xz" "$OUT/"
echo "cmdline c5: $OLDCMD"; echo "cmdline c6: $OLDCMD $ARG"
(cd "$OUT" && sha256sum boot-c5.img boot-c6.img initramfs-c5.cpio.xz | tee SHA256SUMS-c5)
