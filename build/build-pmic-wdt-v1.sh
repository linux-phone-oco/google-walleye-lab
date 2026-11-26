#!/bin/sh
# The two boot images of the attended PMIC watchdog v1 try (item walleye-pmic-wdt-v1-image; decisions 2026-11-19-walleye-pmic-wdt-v1-test-first and -reboot-and-next):
#   A = the next-based v1 tree (linux-next next-20261118 + the six patches of docs research/upstream-forks/2026-11-19/pmic-wdt-gen2-v1/), so what is mailed is what boots;
#   B = v7.3 + linux-phone-oco/linux topic/pmic-wdt-wake-in-suspend at 8b2283da3 (the patches 1 and 2 of v1 are f5bb57d27 and 8b2283da3 there: the same code), the fallback for the same visit.
# Both: the walleye dts of this repository (UFS off, &pm8998_wdt okay), build/walleye.config + build/walleye-wdt.config (CONFIG_DYNAMIC_DEBUG), and init-wdt1 as the ramdisk: unarmed at boot, the banner prints the PON
# registers and the S2 type read back, the shell has wdt-bite and wdt-reboot. The command line is z21's with its gpio dyndbg replaced by dyndbg="file drivers/watchdog/pm8916_wdt.c +p" (the driver's two dev_dbg lines
# reach the banner). Header (offsets, os version, page size), size and AVB footer are z21's. Built only - no phone run; the keeper tries one image once (walleye-try-pmic-wdt-v1), UFS off by the gate first.
#   build-pmic-wdt-v1.sh WORK_DIR Z21_BOOT_IMG KERNEL_A_OUT KERNEL_B_OUT OUT_DIR
# KERNEL_x_OUT is build-kernel.sh's OUT_DIR, run with EXTRA_CONFIG=build/walleye-wdt.config. WORK_DIR holds busybox-arm64, busybox-x86_64 (+ busybox-arm64.src). GEN_INIT_CPIO (default
# WORK_DIR/out-z20/b/usr/gen_init_cpio) is the kernel's own cpio packer. Environment DTBO as in ufs-gate.sh. Writes OUT_DIR/{boot-wdt1-a.img,boot-wdt1-b.img,SHA256SUMS-wdt1}; every check stops the build
# (ok/FAIL, exit 1, nothing written).
set -eu
W=${1:?usage: build-pmic-wdt-v1.sh WORK_DIR Z21_BOOT_IMG KERNEL_A_OUT KERNEL_B_OUT OUT_DIR}
Z21=${2:?}
KA=${3:?}
KB=${4:?}
OUT=${5:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
GEN=${GEN_INIT_CPIO:-$W/out-z20/b/usr/gen_init_cpio}
BB=$W/busybox-arm64
I=$HERE/build/initramfs-mainline/init-wdt1
OLDI=$HERE/build/initramfs-mainline/init-z14
CONF=$HERE/build/initramfs-mainline/udhcpd-z14.conf
DYN='dyndbg="file drivers/watchdog/pm8916_wdt.c +p"'
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$Z21" "$I" "$OLDI" "$CONF" "$DTBO" "$W/busybox-x86_64" "$W/busybox-arm64.src/.config" "$KA/Image.gz" "$KA/.config" "$KA/msm8998-google-walleye.dtb" "$KB/Image.gz" "$KB/.config" "$KB/msm8998-google-walleye.dtb"; do
  [ -s "$f" ] || { echo "build-pmic-wdt-v1: missing $f" >&2; exit 1; }
done
INFO=$(unpack_bootimg --boot_img "$Z21" --out "$T/z21")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$Z21")
AV=$(avbtool info_image --image "$Z21")
OLDCMD=$(field 'command line args')
case $OLDCMD in *' dyndbg="file drivers/gpio/gpiolib.c '*) NEWCMD="${OLDCMD%% dyndbg=*} $DYN";; *) echo "build-pmic-wdt-v1: z21's command line has no gpio dyndbg to replace: $OLDCMD" >&2; exit 1;; esac
echo "command line: $NEWCMD"
ck "the new command line is z21's without its gpio dyndbg, plus the driver's" "$(printf '%s\n' "$NEWCMD" | sed "s/ $(printf '%s' "$DYN" | sed 's/[][\.*^$/]/\\&/g')\$//")" "clk_ignore_unused ignore_loglevel panic=5 androidboot.hardware=walleye androidboot.console=ttyMSM0 androidboot.dtbo_idx=3"
ck "the command line names no ufs" "$(printf '%s\n' "$NEWCMD" | grep -ci ufs || true)" 0
# the ramdisk
for a in IFCONFIG UDHCPD TELNETD DMESG WATCHDOG SYNC REBOOT; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$a=y" "$W/busybox-arm64.src/.config")" 1
done
ck "init-wdt1 (busybox ash -n) has no syntax error" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
ck "the host selftest passes on init-wdt1 (tests/host/pmic-wdt-v1-init-selftest.sh)" "$(sh "$HERE/tests/host/pmic-wdt-v1-init-selftest.sh" "$I" | cut -d' ' -f1 | tail -n1)" pass
ck "every init of ours writes the serial walleye-first-boot (check-init-serials.sh)" "$(sh "$HERE/build/check-init-serials.sh" "$HERE/build" >/dev/null 2>&1 && echo yes || echo no)" yes
pack() { # INIT OUT_XZ
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
    echo "file /bin/busybox $BB 755 0 0"; echo "slink /bin/sh busybox 777 0 0"; echo "file /init $1 755 0 0"
    echo "file /etc/udhcpd.conf $CONF 644 0 0"
  } > "$T/spec"
  "$GEN" -t 0 "$T/spec" > "$T/rd.cpio"
  xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$2"
}
pack "$OLDI" "$T/old.xz"
ck "packing init-z14 gives z21's ramdisk byte for byte (the packing is the same)" "$(cmp -s "$T/old.xz" "$T/z21/ramdisk" && echo same || echo differs)" same
pack "$I" "$T/wdt1.xz"
# one image
one() { # NAME KDIR VERSION_PATTERN (a string of the kernel's own "Linux version" line)
  n=$1; k=$2
  for s in QCOM_WDT PM8916_WATCHDOG POWER_RESET_QCOM_PON DYNAMIC_DEBUG USB_CONFIGFS_ACM USB_CONFIGFS_ECM SCSI_UFS_QCOM; do
    ck "$n: CONFIG_$s is y" "$(sed -n "s/^CONFIG_$s=//p" "$k/.config")" y
  done
  ck "$n: CONFIG_DEBUG_GPIO is not set (that is the diagnostic kernel's)" "$(grep -c '^CONFIG_DEBUG_GPIO=y' "$k/.config" || true)" 0
  ck "$n: CONFIG_WATCHDOG_NOWAYOUT is not set (with it a watchdog cannot be stopped, and the stop-on-reboot proof would mean nothing)" "$(grep -c '^# CONFIG_WATCHDOG_NOWAYOUT is not set' "$k/.config")" 1
  zcat "$k/Image.gz" > "$T/kernel-$n"
  ck "$n: the kernel is the one meant ($3)" "$(grep -ao 'Linux version [^ ]*' "$T/kernel-$n" | head -n1 | sed -n "s/.*\($3\).*/\1/p")" "$3"
  ck "$n: the kernel holds the driver's read-back line (patch 1: S2 reset control was) and the PM8998 compatible" "$(grep -ac 'S2 reset control was' "$T/kernel-$n" | sed 's/^[1-9][0-9]*$/yes/')|$(grep -ac 'qcom,pm8998-wdt' "$T/kernel-$n" | sed 's/^[1-9][0-9]*$/yes/')" "yes|yes"
  GCC=/soc@0/clock-controller@100000
  ck "$n: the built dtb has clock_gcc at the empty stub (the dts cannot name the real gcc: the stub dtsi holds the label)" "$(fdtget "$k/msm8998-google-walleye.dtb" /__symbols__ clock_gcc)" /vendor-overlay-stubs/clock_gcc
  D=$T/dtb-$n.dtb
  cp "$k/msm8998-google-walleye.dtb" "$D"
  fdtput -t s "$D" /__symbols__ clock_gcc "$GCC"   # as z3 and every image since: the overlay's clock users must name the real gcc
  ck "$n: clock_gcc is the real gcc in the image dtb" "$(fdtget "$D" /__symbols__ clock_gcc)" "$GCC"
  ck "$n: the PMIC watchdog node is the PM8998 one, enabled" "$(fdtget "$D" /soc@0/spmi@800f000/pmic@0/pon@800/watchdog compatible 2>&1 | head -n1) $(fdtget "$D" /soc@0/spmi@800f000/pmic@0/pon@800/watchdog status 2>&1 | head -n1)" "qcom,pm8998-wdt okay"
  ck "$n: UFS is off in the dtb" "$(fdtget "$D" /soc@0/ufshc@1da4000 status) $(fdtget "$D" /soc@0/phy@1da7000 status)" "disabled disabled"
  ck "$n: the SoC watchdog (qcom,kpss-wdt) is in the dtb for the petter" "$(fdtget "$D" /soc@0/watchdog@17817000 compatible 2>&1 | head -n1)" qcom,kpss-wdt
  cat "$k/Image.gz" "$D" > "$T/blob-$n"
  set --
  while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
  IMG=$T/boot-wdt1-$n.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$NEWCMD" \
    --kernel "$T/blob-$n" --ramdisk "$T/wdt1.xz" -o "$IMG"
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u-$n")
  ck "$n: the kernel blob is Image.gz then the dtb, byte for byte" "$(cmp -s "$T/u-$n/kernel" "$T/blob-$n" && echo same || echo differs)" same
  ck "$n: the ramdisk is wdt1's, byte for byte" "$(cmp -s "$T/u-$n/ramdisk" "$T/wdt1.xz" && echo same || echo differs)" same
  ck "$n: the command line is the new one" "$(printf '%s\n' "$U" | sed -n 's/^command line args: *//p')" "$NEWCMD"
  for h in 'kernel load address' 'ramdisk load address' 'kernel tags load address' 'page size' 'os version' 'os patch level' 'boot image header version'; do
    ck "$n: header $h is z21's" "$(printf '%s\n' "$U" | sed -n "s/^$h: *//p")" "$(field "$h")"
  done
  ck "$n: the size is z21's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  KS=$(stat -c %s "$T/blob-$n"); KO_=$(printf %d "$(field 'kernel load address')"); RO=$(printf %d "$(field 'ramdisk load address')")
  ck "$n: the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
  mkdir "$T/v-$n" && cp "$IMG" "$T/v-$n/boot.img"
  ck "$n: the AVB footer verifies" "$(avbtool verify_image --image "$T/v-$n/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$n: the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$IMG" >/dev/null 2>&1 && echo pass || echo fail)" pass
  for t in touch-gpio-single-claimant overlay-collisions dtbo-overlay-merges overlay-clock-gcc i2c-qup-after-merge touch-power-after-merge adreno-smmu-disabled; do
    ck "$n: tests/host/$t.sh on the dtb" "$(DTB="$D" QUIET=1 sh "$HERE/tests/host/$t.sh" | tail -n1 | cut -d' ' -f1)" pass
  done
}
one a "$KA" "next-20261118"
one b "$KB" "g8b2283da3"
[ $total -eq 0 ] || { echo "build-pmic-wdt-v1: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$T/boot-wdt1-a.img" "$T/boot-wdt1-b.img" "$OUT/"
(cd "$OUT" && sha256sum boot-wdt1-a.img boot-wdt1-b.img | tee SHA256SUMS-wdt1)
