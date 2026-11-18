#!/bin/sh
# vendor-facts-vf1 (backlog walleye-vendor-facts-dump): the stock control's boot.img with only the ramdisk swapped (xz), init = initramfs-vendor5/init-vf1: m1's mass_storage window, 8 MiB, holding a gzipped tar of the running hardware's facts
# (device tree as merged, clocks, regulators, pinctrl/gpio, interrupts, iomem, dwc3/udc state, dmesg), read on the host by tests/host/vendor-facts-read.sh. Host only; nothing touches the phone.
#
#   build-vendor-facts.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [M1_BOOT_IMG]     (WORK_DIR as build-vendor-gadget4.sh)
# Writes OUT_DIR/initramfs-vendor-vf1.cpio.xz and OUT_DIR/boot-vf1.img, checks each (ok/FAIL, exit 1 on a FAIL), prints the sha256s.
set -eu
W=${1:?usage: build-vendor-facts.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [M1_BOOT_IMG]}
CTRL=${2:?}
OUT=${3:?}
M1=${4:-}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
[ -x "$GEN" ] || make -C "$W/linux-v7.3" ARCH=arm64 LLVM=1 O="$W/out/b" usr/gen_init_cpio >/dev/null
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
P=$T/p
INFO=$(unpack_bootimg --boot_img "$CTRL" --out "$P")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
[ "$(field 'boot image header version')" = 0 ] || { echo "build-vendor-facts: the control is not header v0" >&2; exit 1; }
PSIZE=$(stat -c %s "$CTRL")
AV=$(avbtool info_image --image "$CTRL")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
h() { sha256sum "$1" | cut -c1-16; }
hdr() { echo "$1" | grep -v -e '^boot magic' -e 'ramdisk size'; }
total=0
for S in vf1; do
  I=$HERE/initramfs-vendor5/init-$S
  SPEC=$T/vendor$S.spec
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    # no devtmpfs on the vendor kernel: every node the init needs comes from the cpio
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"; echo "nod /dev/zero 666 0 0 c 1 5"
    echo "nod /dev/kmsg 600 0 0 c 1 11";   echo "nod /dev/tty 666 0 0 c 5 0";    echo "nod /dev/ptmx 666 0 0 c 5 2"
    echo "nod /dev/urandom 444 0 0 c 1 9"
    echo "file /bin/busybox $BB 755 0 0"
    echo "slink /bin/sh busybox 777 0 0"
    echo "file /init $I 755 0 0"
  } > "$SPEC"
  "$GEN" -t 0 "$SPEC" > "$T/rd$S.cpio"
  xz --check=crc32 --lzma2=preset=6e -c "$T/rd$S.cpio" > "$OUT/initramfs-vendor-$S.cpio.xz"
  IMG=$OUT/boot-$S.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
    --kernel "$P/kernel" --ramdisk "$OUT/initramfs-vendor-$S.cpio.xz" -o "$IMG"
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  pass=0; fail=0
  ck() { if [ "$2" = "$3" ]; then echo "ok   $S $1"; pass=$((pass+1)); else echo "FAIL $S $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$S")
  ck "header equals the control's (offsets, page, os word, command line)" "$(hdr "$U")" "$(hdr "$INFO")"
  ck "kernel blob is the control's" "$(h "$T/u$S/kernel")" "$(h "$P/kernel")"
  ck "ramdisk is this step's xz" "$(h "$T/u$S/ramdisk")" "$(h "$OUT/initramfs-vendor-$S.cpio.xz")"
  ck "ramdisk is xz with CRC32 as the control's" "$(file -b "$T/u$S/ramdisk" | cut -d, -f1-2)" "$(file -b "$P/ramdisk" | cut -d, -f1-2)"
  ck "ramdisk has /init, /bin/busybox, /dev/console, /dev/ptmx, /dev/kmsg" "$(xz -dc "$T/u$S/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e bin/busybox -e dev/console -e dev/ptmx -e dev/kmsg)" 5
  mkdir "$T/x$S" && (cd "$T/x$S" && xz -dc "$T/u$S/ramdisk" | cpio -id --quiet 2>/dev/null || true)
  ck "busybox in it is aarch64 static" "$(file -b "$T/x$S/bin/busybox" | cut -d, -f1-2)" "ELF 64-bit LSB executable, ARM aarch64"
  ck "size is the control's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  mkdir "$T/v$S" && cp "$IMG" "$T/v$S/boot.img"
  ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v$S/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "the init in the ramdisk is the source init" "$(cmp -s "$T/x$S/init" "$I" && echo same || echo differs)" same
  ck "init has no 'set -e' and ends in a sleep loop" "$(grep -c -e '^set -e' "$I")$(grep -c '^while :; do sleep 3600; done$' "$I")" 01
  ck "init holds 90 s (9 rounds of sleep 10), then reboot -f, then sysrq b" "$(grep -c -e 'while \[ \$n -lt 9 \]; do' -e '^  sleep 10$' -e 'reboot -f; sleep 5; echo b > /proc/sysrq-trigger' "$I")" 3
  ck "init writes its kmsg marker before the first mount" "$(grep -n -e 'init start (before any mount)' -e '^mount ' "$I" | head -n1 | grep -c start)" 1
  # exactly the parts vf1 has (1 = present): configfs mount, mass_storage function, config dir, symlink, UDC bind; none of ncm, telnetd, udhcpd, ifconfig
  got=$(for pat in '^mount -t configfs' '^mkdir -p \$G/functions/mass_storage.0' '^mkdir -p \$G/configs/c.1' '^ln -s' '> \$G/UDC' 'ncm|telnetd|udhcpd|ifconfig|dev_addr|host_addr'; do [ "$(grep -v '^#' "$I" | grep -c -E -e "$pat")" -ge 1 ] && printf 1 || printf 0; done)
  ck "init has exactly m1's gadget parts (configfs, mass_storage function, config dir, symlink, UDC, no ncm/net)" "$got" 111110
  ck "init says what it is about to do before each step (9 say lines)" "$(grep -c "step vf1: about to" "$I")" 9
  ck "init writes only the attributes file, ro, removable, cdrom, nofua of the lun" "$(grep -o -E 'for a in [a-z ]+' "$I" | tr '\n' ' ')" "for a in ro removable for a in cdrom nofua "
  ck "init has the lun read-only for the host (ro=1) and an 8 MiB file" "$(grep -c -e 'echo 1 > \$L/\$a' -e 'bs=1024 count=8192' "$I")" 2
  ck "init mounts debugfs and reads, never writes, the phone's files (no write to /dev/block, /data, /system, mmc)" "$(grep -v '^#' "$I" | grep -c -E 'mmcblk|/dev/block|/data|/system|mkfs|fastboot|dd of=/dev/(sd|mmc)')" 0
  ck "init asks for the device tree, clk, regulator, pinctrl, gpio, interrupts, iomem, ioports, usb, dwc3, udc, dmesg" "$(for k in '/proc/device-tree' 'clk_summary' 'regulator_summary' 'pinctrl' 'debug/gpio' '/proc/interrupts' '/proc/iomem' '/proc/ioports' 'debug/usb' 'dwc3' 'class/udc' 'dmesg'; do grep -v '^#' "$I" | grep -q -e "$k" && printf 1 || printf 0; done)" 111111111111
  ck "init redacts: serial, MAC-shaped words, cpuinfo Serial, device-tree mac-address/serial-number/imei names, 15-digit runs, bootargs, no /sys/class/net" "$(grep -v '^#' "$I" | grep -c -e 'serialno=REDACTED' -e 'XX:XX:XX:XX:XX:XX' -e 'grep -v -i' -e 'IMEI-SHAPED' -e 'serial-number')$(grep -v '^#' "$I" | grep -c 'class/net')" 50
  ck "init writes the blob body first, then the length header (64 bytes, WALLEYE-VF1 LEN= ROUND=)" "$(grep -n -e 'bs=64 seek=1 conv=notrunc' -e "WALLEYE-VF1 LEN=%010d ROUND=%02d" "$I" | cut -d: -f1 | tr '\n' ' ' | awk '{print ($2>$1) ? "body-first" : "wrong"}')" body-first
  if [ -n "$M1" ]; then
    MI=$(unpack_bootimg --boot_img "$M1" --out "$T/m1u")
    ck "header equals m1's (bytes the same as the control's)" "$(hdr "$U")" "$(hdr "$MI")"
    ck "kernel blob is m1's" "$(h "$T/u$S/kernel")" "$(h "$T/m1u/kernel")"
    ck "footer property block equals m1's" "$(avbtool info_image --image "$IMG" | sed -n 's/^ *\(Prop\|Algorithm\|Partition Name\|Salt\).*/&/p' | sort | md5sum)" "$(avbtool info_image --image "$M1" | sed -n 's/^ *\(Prop\|Algorithm\|Partition Name\|Salt\).*/&/p' | sort | md5sum)"
  fi
  echo "boot-$S: $pass ok, $fail failed"
  total=$((total+fail))
done
(cd "$OUT" && sha256sum initramfs-vendor-vf1.cpio.xz boot-vf1.img | tee SHA256SUMS-vf1)
[ $total -eq 0 ]
