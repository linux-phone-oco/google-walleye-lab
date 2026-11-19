#!/bin/sh
# vendor-facts-vf2 (backlog walleye-vf2-marker-first): the stock control's boot.img with only the ramdisk swapped (xz), init = initramfs-vendor5/init-vf2: vf1's dump with the marker written at once, a beat, and a status line per step
# in a text region of the window (first 64 KiB), the blob at 65536 with its own magic. Read on the host by tests/host/vendor-facts-read.sh. Host only; nothing touches the phone.
#
#   build-vendor-facts-vf2.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [M1_BOOT_IMG]     (WORK_DIR as build-vendor-gadget4.sh)
# Writes OUT_DIR/initramfs-vendor-vf2.cpio.xz and OUT_DIR/boot-vf2.img, checks each (ok/FAIL, exit 1 on a FAIL), prints the sha256s.
set -eu
W=${1:?usage: build-vendor-facts-vf2.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [M1_BOOT_IMG]}
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
for S in vf2; do
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
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
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
  ck "init holds 9 rounds of sleep 10, then reboot -f, then sysrq b" "$(grep -c -e 'while \[ \$n -lt 9 \]; do' -e '^  sleep 10$' -e 'reboot -f; sleep 5; echo b > /proc/sysrq-trigger' "$I")" 3
  ck "init makes the window and writes the marker (paint) before the proc mount and before any gadget step" "$(grep -n -e '^dd if=/dev/zero of=\$LOGF' -e '^paint$' -e '^mount ' -e '^stp ' -e '^gadget$' "$I" | head -n3 | cut -d: -f2 | tr '\n' ' ')" "dd if=/dev/zero of=\$LOGF bs=1024 count=8192 2>/dev/null paint mount -t proc proc /proc "
  ck "the text region is m1's way (dd bs=65536 count=1 conv=sync,notrunc), line 1 WALLEYE-VF2 BEAT=" "$(grep -c -e 'dd if=\$T of=\$LOGF bs=65536 count=1 conv=sync,notrunc' -e "printf 'WALLEYE-VF2 BEAT=" "$I")" 2
  ck "a background beat repaints the region every 2 s" "$(grep -c -e '^( b=0; while :; do b=\$((b+1)); echo \$b > /tmp/walleye-beat; paint /tmp/walleye-text-b; sleep 2; done ) &$' "$I")" 1
  ck "every step runs through stp (start line before, rc and stderr after)" "$(grep -c -e 'note "start \$N \$NAME"' -e 'note "step \$N \$NAME rc=\$RC' -e '"\$@" 2> /tmp/walleye-err; RC=\$?' "$I")" 3
  ck "stp writes its start line before it says anything on the console, and a step with a failed grab ends with a non-zero rc" "$(grep -n -e 'note "start \$N \$NAME"' -e 'say "step \$N \$NAME: about to run"' "$I" | cut -d: -f1 | tr '\n' ' ' | awk '{print ($1<$2) ? "start-first" : "wrong"}')$(grep -c -e 'echo "\$1" >> /tmp/walleye-gf; return 1' -e 'RC=\$GF' "$I")" "start-first2"
  ck "paint takes snapshot and write under one flock, so the foreground and the beat never leave an older snapshot over a newer line" "$(grep -c -e '( flock 9$' -e '9> /tmp/walleye-lock' "$I")" 2
  ck "the init sends nothing to /dev/null except say(), the window file's dd and the stderr of the vendor-facts copies inside functions" "$(grep -v '^#' "$I" | grep -v -e 'say()' -e 'dd if=' -e 'grab' -e 'paint' | grep -c '/dev/null')" "$(grep -v '^#' "$I" | grep -v -e 'say()' -e 'dd if=' -e 'grab' -e 'paint' | grep -c -e '2>/dev/null')"
  ck "the steps are in the init: 3 mounts, 4 more statics, the gadget's 15" "$(grep -c -E '^stp mount-' "$I")$(for k in s-devicetree s-cmdline s-iomem s-ioports; do grep -q "stp $k " "$I" && printf 1 || printf 0; done)$(for k in g-mkdir g-idVendor g-idProduct g-bcdDevice g-bcdUSB g-strings g-serial g-manufacturer g-product g-function g-lun-file g-config g-config-name g-symlink g-bind; do grep -q "stp $k " "$I" && printf 1 || printf 0; done)" "31111111111111111111"
  ck "applets the init lists at start are in the busybox build's .config" "$(grep -c -e '^for ap in tar gzip cat dd mount timeout find sed grep stat head tail tr cut wc sync dmesg printf sleep flock; do' "$I")$(for ap in TAR GZIP CAT DD MOUNT TIMEOUT FIND SED GREP STAT HEAD TAIL TR CUT WC SYNC DMESG PRINTF SLEEP FLOCK; do grep -q "^CONFIG_$ap=y" "$W/busybox-arm64.src/.config" && printf 1 || printf 0; done)" "111111111111111111111"
  ck "the other applets the init calls (ls ln mkdir uname reboot echo ash) are in the build too" "$(for ap in LS LN MKDIR UNAME REBOOT ECHO ASH; do grep -q "^CONFIG_$ap=y" "$W/busybox-arm64.src/.config" && printf 1 || printf 0; done)" 1111111
  ck "the blob is at 65536: body at bs=64 seek=1025 first, header at seek=1024 after, magic WALLEYE-VF2-BLOB LEN= ROUND=" "$(grep -n -e 'seek=1025 conv=notrunc' -e 'WALLEYE-VF2-BLOB LEN=%010d ROUND=%02d' "$I" | cut -d: -f1 | tr '\n' ' ' | awk '{print ($2>$1) ? "body-first" : "wrong"}')" body-first
  ck "init mounts debugfs and reads, never writes, the phone's files" "$(grep -v '^#' "$I" | grep -c -E 'mmcblk|/dev/block|/data|/system|mkfs|fastboot|dd of=/dev/(sd|mmc)')" 0
  ck "init redacts as vf2 (serialno, MAC-shaped, 15-digit, cpuinfo Serial, dt names; no /sys/class/net)" "$(grep -v '^#' "$I" | grep -c -e 'serialno=REDACTED' -e 'XX:XX:XX:XX:XX:XX' -e 'grep -v -i' -e 'IMEI-SHAPED' -e 'serial-number')$(grep -v '^#' "$I" | grep -c 'class/net')" 50
  ck "init has m1's gadget parts: configfs, mass_storage, config dir, symlink, UDC; no ncm/net" "$(for pat in 'mount -t configfs' 'mass_storage.0' 'configs/c.1' 'ln -s' 'g-bind' 'ncm|telnetd|udhcpd|ifconfig|dev_addr|host_addr'; do [ "$(grep -v '^#' "$I" | grep -c -E -e "$pat")" -ge 1 ] && printf 1 || printf 0; done)" 111110
  ck "init asks for the device tree, clk, regulator, pinctrl, gpio, interrupts, iomem, ioports, usb, dwc3, udc, dmesg" "$(for k in '/proc/device-tree' 'clk_summary' 'regulator_summary' 'pinctrl' 'debug/gpio' '/proc/interrupts' '/proc/iomem' '/proc/ioports' 'debug/usb' 'dwc3' 'class/udc' 'dmesg'; do grep -v '^#' "$I" | grep -q -e "$k" && printf 1 || printf 0; done)" 111111111111
  if [ -n "$M1" ]; then
    MI=$(unpack_bootimg --boot_img "$M1" --out "$T/m1u")
    ck "header equals m1's (bytes the same as the control's)" "$(hdr "$U")" "$(hdr "$MI")"
    ck "kernel blob is m1's" "$(h "$T/u$S/kernel")" "$(h "$T/m1u/kernel")"
    ck "footer property block equals m1's" "$(avbtool info_image --image "$IMG" | sed -n 's/^ *\(Prop\|Algorithm\|Partition Name\|Salt\).*/&/p' | sort | md5sum)" "$(avbtool info_image --image "$M1" | sed -n 's/^ *\(Prop\|Algorithm\|Partition Name\|Salt\).*/&/p' | sort | md5sum)"
  fi
  echo "boot-$S: $pass ok, $fail failed"
  total=$((total+fail))
done
(cd "$OUT" && sha256sum initramfs-vendor-vf2.cpio.xz boot-vf2.img | tee SHA256SUMS-vf2)
[ $total -eq 0 ]
