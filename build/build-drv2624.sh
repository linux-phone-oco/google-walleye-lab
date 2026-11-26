#!/bin/sh
# The boot image of the haptics try (item walleye-drv2624-image): z21's image with the kernel that holds the TI DRV2624 driver and the dtb that carries the walleye vibrator@5a node.
#   Kernel: kernel.org v7.3 + the pmic-wdt topic and the fuel gauge RFC (as every image since fg1) + the four driver commits of topic/walleye-drv2624 (binding, driver, two fixes).
#   The walleye dts change (once patch 0005) is already in dt/msm8998-google-walleye.dts, which build-kernel.sh copies in. The driver is built in (the initramfs carries no modules).
#   Ramdisk, header, size and AVB footer are z21's byte for byte (its init has the command loop, the telnet shell and the serial walleye-first-boot); the command line is z21's with its gpio dyndbg
#   replaced by the driver's one dev_dbg, the chip id (dyndbg="file drivers/input/misc/drv2624.c +p"). UFS off by the gate. Built only - no phone run; the keeper reads the part over the mainline shell
#   (ls /sys/bus/i2c/devices, the driver link, dmesg), as tests/health/base/amp-present.sh does on a phone with a Linux userland.
#   build-drv2624.sh Z21_BOOT_IMG KERNEL_OUT OUT_DIR
# KERNEL_OUT is build-kernel.sh's OUT_DIR, run with EXTRA_CONFIG=build/walleye-drv2624.config. Environment DTBO as in ufs-gate.sh. Writes OUT_DIR/{boot-drv1.img,SHA256SUMS-drv1};
# every check stops the build (ok/FAIL, exit 1, nothing written).
set -eu
Z21=${1:?usage: build-drv2624.sh Z21_BOOT_IMG KERNEL_OUT OUT_DIR}
K=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
export DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
DYN='dyndbg="file drivers/input/misc/drv2624.c +p"'
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$Z21" "$DTBO" "$K/Image.gz" "$K/.config" "$K/msm8998-google-walleye.dtb"; do
  [ -s "$f" ] || { echo "build-drv2624: missing $f" >&2; exit 1; }
done
INFO=$(unpack_bootimg --boot_img "$Z21" --out "$T/z21")
field() { printf '%s\n' "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
PSIZE=$(stat -c %s "$Z21")
AV=$(avbtool info_image --image "$Z21")
OLDCMD=$(field 'command line args')
case $OLDCMD in *' dyndbg="file drivers/gpio/gpiolib.c '*) NEWCMD="${OLDCMD%% dyndbg=*} $DYN";; *) echo "build-drv2624: z21's command line has no gpio dyndbg to replace: $OLDCMD" >&2; exit 1;; esac
echo "command line: $NEWCMD"
ck "the new command line is z21's without its gpio dyndbg, plus the driver's" "$(printf '%s\n' "$NEWCMD" | sed "s/ $(printf '%s' "$DYN" | sed 's/[][\.*^$/]/\\&/g')\$//")" "clk_ignore_unused ignore_loglevel panic=5 androidboot.hardware=walleye androidboot.console=ttyMSM0 androidboot.dtbo_idx=3"
ck "the command line names no ufs" "$(printf '%s\n' "$NEWCMD" | grep -ci ufs || true)" 0
for s in INPUT_DRV2624_HAPTICS DYNAMIC_DEBUG USB_CONFIGFS_ACM USB_CONFIGFS_ECM I2C_QUP; do
  ck "CONFIG_$s is y" "$(sed -n "s/^CONFIG_$s=//p" "$K/.config")" y
done
zcat "$K/Image.gz" > "$T/kernel"
ck "the kernel holds the driver (its compatible)" "$(grep -ac 'ti,drv2624' "$T/kernel" | sed 's/^[1-9][0-9]*$/yes/')" yes
ck "the kernel is v7.3-based" "$(grep -ao 'Linux version [^ ]*' "$T/kernel" | head -n1 | sed -n 's/.*\(7\.3\.0\).*/\1/p')" 7.3.0
ck "the kernel holds patch 0004 (its range-check message; the tree must carry topic/walleye-drv2624)" "$(grep -ac 'gives a drive time the register cannot hold' "$T/kernel" | sed 's/^[1-9][0-9]*$/yes/')" yes
GCC=/soc@0/clock-controller@100000
D=$T/dtb
cp "$K/msm8998-google-walleye.dtb" "$D"
fdtput -t s "$D" /__symbols__ clock_gcc "$GCC"   # as z3 and every image since: the overlay's clock users must name the real gcc
ck "clock_gcc is the real gcc in the image dtb" "$(fdtget "$D" /__symbols__ clock_gcc)" "$GCC"
VB=/soc@0/i2c@c17a000/vibrator@5a
ck "the vibrator node is the driver's compatible, reg 0x5a" "$(fdtget "$D" $VB compatible 2>&1 | head -n1) $(fdtget -t x "$D" $VB reg 2>&1 | head -n1)" "ti,drv2624 5a"
ck "the vibrator's reset gpio is tlmm 95, active low" "$(fdtget -t u "$D" $VB reset-gpios 2>&1 | awk '{print $2, $3}')" "95 1"
ck "the vibrator's bus (QUP6) is okay" "$(fdtget "$D" /soc@0/i2c@c17a000 status 2>&1)" okay
ck "the vibrator's motor values (actuator 1, loop 0, rated 77, clamp 165, f0 155, open-loop 155, wave 0)" "$(for p in actuator smart-loop rated-voltage odclamp-voltage lra-frequency ol-lra-frequency lra-wave-shape; do fdtget -t u "$D" $VB ti,$p 2>&1; done | tr '\n' ' ')" "1 0 77 165 155 155 0 "
ck "UFS is off in the dtb" "$(fdtget "$D" /soc@0/ufshc@1da4000 status) $(fdtget "$D" /soc@0/phy@1da7000 status)" "disabled disabled"
cat "$K/Image.gz" "$D" > "$T/blob"
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(printf '%s\n' "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-drv1.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$NEWCMD" \
  --kernel "$T/blob" --ramdisk "$T/z21/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(printf '%s\n' "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
ck "the kernel blob is Image.gz then the dtb, byte for byte" "$(cmp -s "$T/u/kernel" "$T/blob" && echo same || echo differs)" same
ck "the ramdisk is z21's, byte for byte" "$(cmp -s "$T/u/ramdisk" "$T/z21/ramdisk" && echo same || echo differs)" same
ck "the command line is the new one" "$(printf '%s\n' "$U" | sed -n 's/^command line args: *//p')" "$NEWCMD"
for h in 'kernel load address' 'ramdisk load address' 'kernel tags load address' 'page size' 'os version' 'os patch level' 'boot image header version'; do
  ck "header $h is z21's" "$(printf '%s\n' "$U" | sed -n "s/^$h: *//p")" "$(field "$h")"
done
ck "the size is z21's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
KS=$(stat -c %s "$T/blob"); KO_=$(printf %d "$(field 'kernel load address')"); RO=$(printf %d "$(field 'ramdisk load address')")
ck "the kernel blob ends before the ramdisk loads (headroom in bytes > 524288)" "$([ $((RO - KO_ - KS)) -gt 524288 ] && echo yes || echo "no: $((RO - KO_ - KS))")" yes
mkdir "$T/v" && cp "$IMG" "$T/v/boot.img"
ck "the AVB footer verifies" "$(avbtool verify_image --image "$T/v/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
ck "the UFS-off gate (build/ufs-gate.sh) passes the image" "$(sh "$HERE/build/ufs-gate.sh" "$IMG" >/dev/null 2>&1 && echo pass || echo fail)" pass
for t in touch-gpio-single-claimant overlay-collisions dtbo-overlay-merges overlay-clock-gcc i2c-qup-after-merge touch-power-after-merge adreno-smmu-disabled; do
  ck "tests/host/$t.sh on the dtb" "$(DTB="$D" QUIET=1 sh "$HERE/tests/host/$t.sh" | tail -n1 | cut -d' ' -f1)" pass
done
[ $total -eq 0 ] || { echo "build-drv2624: $total checks failed, nothing written" >&2; exit 1; }
mkdir -p "$OUT"
cp "$IMG" "$OUT/boot-drv1.img"
(cd "$OUT" && sha256sum boot-drv1.img | tee SHA256SUMS-drv1)
