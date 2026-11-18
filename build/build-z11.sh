#!/bin/sh
# z11 (backlog walleye-ufs-readonly): z10 with the UFS storage listed. TWO changes from z10, both for UFS and nothing else: the kernel (Image.gz and dtb: v7.3 + the pmic-wdt series as z9, UFS host and QMP phy built in, ufshc and ufsphy okay with their l20/l26/s4 rails)
# and the ramdisk's /init (initramfs-mainline/init-z11 = init-z10 + /bin/ufs-lsblk and /bin/ufs-hdr, reads only, mounts and writes nothing). Header, command line and AVB footer are z10's.
#   build-z11.sh WORK_DIR Z10_BOOT_IMG KERNEL_OUT_DIR Z9_KERNEL_OUT_DIR OUT_DIR
# KERNEL_OUT_DIR is build-kernel.sh's OUT_DIR for this tree (Image.gz, msm8998-google-walleye.dtb built with -@, .config); Z9_KERNEL_OUT_DIR is z9's (what z11's kernel differs from).
# Writes OUT_DIR/{boot-z11.img,initramfs-z11.cpio.xz,msm8998-google-walleye-z11.dtb,SHA256SUMS-z11}; checks (ok/FAIL, exit 1 on a FAIL, nothing written then).
# Environment: DTBO (default ~/Projects/walleye-build/control/dtbo.img).
set -eu
W=${1:?usage: build-z11.sh WORK_DIR Z10_BOOT_IMG KERNEL_OUT_DIR Z9_KERNEL_OUT_DIR OUT_DIR}
Z10=${2:?}
KO=${3:?}
CO=${4:?}
OUT=${5:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
I=$HERE/build/initramfs-mainline/init-z11
OLD=$HERE/build/initramfs-mainline/init-z10
GCC=/soc@0/clock-controller@100000
UFS=/soc@0/ufshc@1da4000
PHY=/soc@0/phy@1da7000
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$Z10" "$I" "$OLD" "$W/busybox-x86_64" "$DTBO" "$KO/Image.gz" "$KO/.config" "$KO/msm8998-google-walleye.dtb" "$CO/Image.gz" "$CO/.config" "$CO/msm8998-google-walleye.dtb"; do [ -s "$f" ] || { echo "build-z11: missing $f" >&2; exit 1; }; done
split() { # KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-z11: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
echo "input $(h "$Z10")  $Z10"
INFO=$(unpack_bootimg --boot_img "$Z10" --out "$T/z10")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split "$T/z10/kernel" "$T/k"
PSIZE=$(stat -c %s "$Z10")
AV=$(avbtool info_image --image "$Z10")
cp "$KO/msm8998-google-walleye.dtb" "$T/z11.dtb"
fdtput -t s "$T/z11.dtb" /__symbols__ clock_gcc "$GCC"
cp "$CO/msm8998-google-walleye.dtb" "$T/z9.dtb"
fdtput -t s "$T/z9.dtb" /__symbols__ clock_gcc "$GCC"
cat "$KO/Image.gz" "$T/z11.dtb" > "$T/blob"
# the kernel
for s in SCSI_UFSHCD SCSI_UFSHCD_PLATFORM SCSI_UFS_QCOM PHY_QCOM_QMP PHY_QCOM_QMP_UFS BLK_DEV_SD EFI_PARTITION PM8916_WATCHDOG CHARGER_QCOM_SMB2; do
  ck ".config has $s=y" "$(grep -c "^CONFIG_$s=y\$" "$KO/.config")" 1
done
ck "z9's .config had the UFS host glue as a module (the change is the =y)" "$(grep -c '^CONFIG_SCSI_UFS_QCOM=m$' "$CO/.config")" 1
ck "the new Image.gz is not z9's (the kernel change)" "$(cmp -s "$KO/Image.gz" "$CO/Image.gz" && echo same || echo differs)" differs
ck "Image.gz holds the qcom UFS host driver's generic compatible (the dtb's second string)" "$(zcat "$KO/Image.gz" | grep -aq 'qcom,ufshc' && echo yes || echo no)" yes
ck "Image.gz holds the msm8998 QMP UFS phy's compatible" "$(zcat "$KO/Image.gz" | grep -aq 'qcom,msm8998-qmp-ufs-phy' && echo yes || echo no)" yes
ck "Image.gz still holds the pmic watchdog and the charger" "$(zcat "$KO/Image.gz" | grep -aq 'qcom,pm8998-wdt' && zcat "$KO/Image.gz" | grep -aq qcom-smbx-charger && echo yes || echo no)" yes
# the dtb
ck "z11 dtb: ufshc is okay with its compatible" "$(fdtget "$T/z11.dtb" "$UFS" status)  $(fdtget "$T/z11.dtb" "$UFS" compatible)" "okay  qcom,msm8998-ufshc qcom,ufshc jedec,ufs-2.0"
ck "z11 dtb: ufsphy is okay" "$(fdtget "$T/z11.dtb" "$PHY" status)" okay
ck "z11 dtb: ufshc supplies vcc, vccq, vccq2, vdd-hba are all named (four phandles)" "$(for p in vcc vccq vccq2 vdd-hba; do fdtget -t u "$T/z11.dtb" "$UFS" $p-supply; done | wc -l | tr -d ' ')" 4
ck "z9's dtb has ufshc disabled (the dtb change)" "$(fdtget "$T/z9.dtb" "$UFS" status)" disabled
ck "z11 dtb: l20 is 2.96 V and l26 1.2 V (the rails of vcc and vccq)" "$(for r in vreg_l20a_2p95 vreg_l26a_1p2; do fdtget -t u "$T/z11.dtb" "$(fdtget "$T/z11.dtb" /__symbols__ $r)" regulator-min-microvolt; done | tr "\n" " ")" "2960000 1200000 "
ck "z11 dtb: clock_gcc is the real gcc" "$(fdtget "$T/z11.dtb" /__symbols__ clock_gcc)" "$GCC"
ck "z11 dtb keeps the pmic watchdog okay and the charger okay" "$(fdtget "$T/z11.dtb" /soc@0/spmi@800f000/pmic@0/pon@800/watchdog status) $(fdtget "$T/z11.dtb" /soc@0/spmi@800f000/pmic@2/charger@1000 status)" "okay okay"
ck "z11 dtb keeps gpio-reserved-ranges <0 4 81 4>" "$(fdtget -t u "$T/z11.dtb" /soc@0/pinctrl@3400000 gpio-reserved-ranges)" "0 4 81 4"
ck "z11 dtb keeps qcom,msm-id <0x124 0x20001>" "$(fdtget -t x "$T/z11.dtb" / qcom,msm-id)" "124 20001"
ck "z11 dtb keeps the ramoops node at 0xa1100000" "$(fdtget -t x "$T/z11.dtb" /reserved-memory/ramoops@a1100000 reg)" "0 a1100000 0 200000"
ck "z11 dtb: the GPU smmu is disabled (the test passes)" "$(DTB="$T/z11.dtb" sh "$HERE/tests/host/adreno-smmu-disabled.sh" | cut -d" " -f1)" pass
ck "after the merge the overlay's clock users name gcc" "$(DTB="$T/z11.dtb" sh "$HERE/tests/host/overlay-clock-gcc.sh" | cut -d' ' -f1)" pass
# the ramdisk
pack() { # INIT OUT_XZ
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
    echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $1 755 0 0"
  } > "$T/spec"
  "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"
  xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$2"
}
pack "$OLD" "$T/old.xz"
ck "packing init-z10 gives z10's ramdisk byte for byte (the packing is the same)" "$(cmp -s "$T/old.xz" "$T/z10/ramdisk" && echo same || echo differs)" same
mkdir -p "$T/o"; pack "$I" "$T/o/initramfs-z11.cpio.xz"
ck "init-z11 (busybox ash -n) has no syntax error" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
ck "init-z11 differs from init-z10 only by additions, the header comment and the step name" "$(diff "$OLD" "$I" | grep '^<' | cut -c1-12 | tr '\n' '|')" "< # z10 = z6|< say \"z10 i|"
ck "init-z11 mounts no disk and writes to none" "$(grep -v '^[[:space:]]*#' "$I" | grep -c -e 'mount[^|]* /dev/sd' -e '> */dev/sd' -e 'dd [^|]*of=/dev' -e 'dd [^|]*of="\$dev' -e 'mkfs' -e 'blkdiscard')" 0
ck "init-z11 sets every disk read-only (blockdev --setro) before anything can be typed to write it" "$(grep -v "^[[:space:]]*#" "$I" | grep -c "blockdev --setro")" 1
ck "the apps watchdog petter and the 90-beat self-reboot are kept" "$(grep -c -e 'while \[ \$n -lt 90 \]' -e 'self-reboot now' "$I")" 2
ck "the UFS lister's host selftest passes" "$(sh "$HERE/tests/host/ufs-lsblk-selftest.sh" 2>&1 | tail -n1)" "ufs-lsblk-selftest: all ok"
ck "every init of ours writes the serial walleye-first-boot (check-init-serials.sh)" "$(sh "$HERE/build/check-init-serials.sh" "$HERE/build" >/dev/null 2>&1 && echo yes || echo no)" yes
for a in ash mount grep sed tr head cat sleep mv chmod od dd ls cut; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$(echo "$a" | tr a-z A-Z)=y" "$W/busybox-arm64.src/.config")" 1
done
# the image
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-z11.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/o/initramfs-z11.cpio.xz" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
hdr() { printf '%s\n' "$1" | grep -v -e 'ramdisk size' -e 'kernel_size' -e 'kernel size'; }
ck "z11 Image.gz is the new build's, byte for byte" "$(cmp -s "$T/uk.gz" "$KO/Image.gz" && echo same || echo differs)" same
ck "z11 dtb is z11.dtb, byte for byte" "$(cmp -s "$T/uk.dtb" "$T/z11.dtb" && echo same || echo differs)" same
ck "z11 header equals z10's but the sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "z11 ramdisk is this build's xz" "$(cmp -s "$T/u/ramdisk" "$T/o/initramfs-z11.cpio.xz" && echo same || echo differs)" same
ck "z11 ramdisk differs from z10's (the init change)" "$(cmp -s "$T/u/ramdisk" "$T/z10/ramdisk" && echo same || echo differs)" differs
mkdir "$T/x" "$T/y" && (cd "$T/x" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true) && (cd "$T/y" && xz -dc "$T/z10/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "the init in the ramdisk is init-z11" "$(cmp -s "$T/x/init" "$I" && echo same || echo differs)" same
ck "the busybox in it is z10's" "$(cmp -s "$T/x/bin/busybox" "$T/y/bin/busybox" && echo same || echo differs)" same
ck "ramdisk lists the same files as z10's" "$(xz -dc "$T/u/ramdisk" | cpio -t 2>/dev/null | sort | tr '\n' ' ')" "$(xz -dc "$T/z10/ramdisk" | cpio -t 2>/dev/null | sort | tr '\n' ' ')"
ck "z11 size is z10's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
ck "z11 AVB info equals z10's but the digest and the image sizes" "$(avbtool info_image --image "$IMG" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')" "$(printf '%s\n' "$AV" | grep -v -e '^ *Digest:' -e 'Image Size:' -e 'Original image size:' -e 'VBMeta offset:')"
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "z11 AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-z11: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-z11.img"; cp "$T/o/initramfs-z11.cpio.xz" "$OUT/"; cp "$T/z11.dtb" "$OUT/msm8998-google-walleye-z11.dtb"
(cd "$OUT" && sha256sum initramfs-z11.cpio.xz msm8998-google-walleye-z11.dtb boot-z11.img | tee SHA256SUMS-z11)
