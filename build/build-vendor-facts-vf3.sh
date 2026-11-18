#!/bin/sh
# vendor-facts-vf3 (backlog walleye-vf3-gadget-first-progressive): the stock control's boot.img with only the ramdisk swapped (xz), init = initramfs-vendor5/init-vf3: vf2 reordered - the gadget first, then the dump written progressively
# into a 4 MiB text window (beat block 0, framed BEGIN/END blocks after). Read on the host by tests/host/vendor-facts-poll.sh, started before the boot. Host only; nothing touches the phone.
#
#   build-vendor-facts-vf3.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [M1_BOOT_IMG]     (WORK_DIR as build-vendor-gadget4.sh)
# Writes OUT_DIR/initramfs-vendor-vf3.cpio.xz and OUT_DIR/boot-vf3.img, checks each (ok/FAIL, exit 1 on a FAIL), prints the sha256s.
set -eu
W=${1:?usage: build-vendor-facts-vf3.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR [M1_BOOT_IMG]}
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
for S in vf3; do
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
  ck "init holds to 90 s of uptime, then reboot -f, then sysrq b" "$(grep -c -e 'lt 90 \]; do sleep 2; done' -e 'sync; reboot -f; sleep 5; echo b > /proc/sysrq-trigger' "$I")" 2
  ck "order: window file, marker, proc, beat, gadget bind (g-bind), THEN debugfs and every dump source" "$(grep -n -e '^dd if=/dev/zero of=\$LOGF' -e '^beat_once 0$' -e '^mount -t proc' -e '^( b=0; while' -e 'gst g-bind' -e '^mount -t debugfs' -e '^chunk cmdline' "$I" | cut -d: -f1 | tr '\n' ' ' | awk '{ok=1; for(i=2;i<=NF;i++) if ($i<$(i-1)) ok=0; print ok ? "in-order" : "wrong"}')" in-order
  ck "the beat rewrites only block 0 every 2 s (bs=4096 count=1, sleep 2)" "$(grep -c -e 'dd if=/tmp/walleye-beat-txt of=\$LOGF bs=4096 count=1 conv=sync,notrunc' -e 'beat_once \$b; sleep 2; done ) &$' "$I")" 2
  ck "the stream is flushed from the last 4096-byte boundary (skip=K seek=K+1, notrunc) and synced" "$(grep -c -e 'dd if=\$OUT of=\$LOGF bs=4096 skip=\$K seek=\$((K + 1)) conv=notrunc' -e '^  sync$' "$I")" 2
  ck "every source is framed BEGIN name / END name rc= and flushed; the budget is checked" "$(grep -c -e 'echo "BEGIN \$NAME" >> \$OUT' -e 'echo "END \$NAME rc=\$RC' -e 'gt \$BUDGET' -e '^  flush$' "$I")" 4
  ck "the stream ends with a DONE line, flushed" "$(grep -c -e '^echo "DONE up=' "$I")" 1
  ck "nothing goes to /dev/console, and a debugfs read is bounded by timeout" "$(grep -v '^#' "$I" | grep -c '/dev/console')$(grep -c 'timeout 5 cat' "$I")$(grep -c 'timeout 3 cat' "$I")" 011
  ck "no function is passed to timeout (busybox timeout runs programs)" "$(grep -c -F 'timeout 5 "$@"' "$I")" 0
  ck "the init never sends a command's stdout to /dev/null (only stderr, with 2>)" "$(grep -v '^#' "$I" | grep -c -E '(^|[^2])> ?/dev/null')" 0
  ck "applets the init uses are in the busybox build's .config (incl. od, tr classes)" "$(for ap in TAR GZIP CAT DD MOUNT TIMEOUT FIND SED GREP STAT HEAD TAIL TR CUT WC SYNC DMESG PRINTF SLEEP OD SORT LS LN MKDIR UNAME REBOOT ECHO ASH FEATURE_TR_CLASSES; do grep -q "^CONFIG_$ap=y" "$W/busybox-arm64.src/.config" && printf 1 || printf 0; done)" 11111111111111111111111111111
  ck "init mounts debugfs and reads, never writes, the phone's files" "$(grep -v '^#' "$I" | grep -c -E 'mmcblk|/dev/block|/data|/system|mkfs|fastboot|dd of=/dev/(sd|mmc)')" 0
  ck "init redacts (serialno, MAC-shaped, 15-digit; dt names; no /sys/class/net)" "$(grep -v '^#' "$I" | grep -c -e 'serialno=REDACTED' -e 'XX:XX:XX:XX:XX:XX' -e 'IMEI-SHAPED' -e 'serial-number')$(grep -v '^#' "$I" | grep -c 'class/net')" 30
  ck "init has m1's gadget parts: configfs, mass_storage, config dir, symlink, UDC; no ncm/net" "$(for pat in 'mount -t configfs' 'mass_storage.0' 'configs/c.1' 'ln -s' 'g-bind' 'ncm|telnetd|udhcpd|ifconfig|dev_addr|host_addr'; do [ "$(grep -v '^#' "$I" | grep -c -E -e "$pat")" -ge 1 ] && printf 1 || printf 0; done)" 111110
  ck "init asks for the device tree, clk, regulator, pinctrl, gpio, interrupts, iomem, usb, dwc3, udc, dmesg head and tail" "$(for k in '/proc/device-tree' 'clk_summary' 'regulator_summary' 'pinctrl' 'debug/gpio' '/proc/interrupts' '/proc/iomem' 'debug/usb' 'dwc3' 'class/udc' 'dmesg_head' 'dmesg_tail'; do grep -v '^#' "$I" | grep -q -e "$k" && printf 1 || printf 0; done)" 111111111111
  if [ -n "$M1" ]; then
    MI=$(unpack_bootimg --boot_img "$M1" --out "$T/m1u")
    ck "header equals m1's (bytes the same as the control's)" "$(hdr "$U")" "$(hdr "$MI")"
    ck "kernel blob is m1's" "$(h "$T/u$S/kernel")" "$(h "$T/m1u/kernel")"
    ck "footer property block equals m1's" "$(avbtool info_image --image "$IMG" | sed -n 's/^ *\(Prop\|Algorithm\|Partition Name\|Salt\).*/&/p' | sort | md5sum)" "$(avbtool info_image --image "$M1" | sed -n 's/^ *\(Prop\|Algorithm\|Partition Name\|Salt\).*/&/p' | sort | md5sum)"
  fi
  echo "boot-$S: $pass ok, $fail failed"
  total=$((total+fail))
done
(cd "$OUT" && sha256sum initramfs-vendor-vf3.cpio.xz boot-vf3.img | tee SHA256SUMS-vf3)
[ $total -eq 0 ]
