#!/bin/sh
# vendor-facts-vfa..vfd (backlog walleye-vfa-one-source-per-image): four boot images, each = m1's structure (the control's boot.img with only the ramdisk swapped, xz) with init = initramfs-vendor5/init-vfx
# and ONE payload chosen by /etc/walleye-vf (a: fdt + iomem + interrupts + cmdline, b: clk_summary, c: regulator_summary, d: usb/dwc3 debugfs). The four ramdisks differ in that one file. Host only; nothing touches the phone.
# Read on the host with tests/host/window-payload-read.sh.
#
#   build-vendor-facts-vfx.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR M1_BOOT_IMG     (WORK_DIR as build-vendor-gadget4.sh)
# Writes OUT_DIR/initramfs-vendor-vf{a,b,c,d}.cpio.xz and OUT_DIR/boot-vf{a,b,c,d}.img, checks each (ok/FAIL, exit 1 on a FAIL), prints the sha256s into OUT_DIR/SHA256SUMS-vfx.
set -eu
W=${1:?usage: build-vendor-facts-vfx.sh WORK_DIR CONTROL_BOOT_IMG OUT_DIR M1_BOOT_IMG}
CTRL=${2:?}
OUT=${3:?}
M1=${4:?the m1 image is required: the images are checked against it}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
[ -x "$GEN" ] || make -C "$W/linux-v7.3" ARCH=arm64 LLVM=1 O="$W/out/b" usr/gen_init_cpio >/dev/null
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
P=$T/p
INFO=$(unpack_bootimg --boot_img "$CTRL" --out "$P")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
[ "$(field 'boot image header version')" = 0 ] || { echo "build-vendor-facts-vfx: the control is not header v0" >&2; exit 1; }
PSIZE=$(stat -c %s "$CTRL")
AV=$(avbtool info_image --image "$CTRL")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
h() { sha256sum "$1" | cut -c1-16; }
hdr() { echo "$1" | grep -v -e '^boot magic' -e 'ramdisk size'; }
I=$HERE/initramfs-vendor5/init-vfx
M1I=$HERE/initramfs-vendor5/init-m1
MI=$(unpack_bootimg --boot_img "$M1" --out "$T/m1u")
mkdir "$T/m1x" && (cd "$T/m1x" && xz -dc "$T/m1u/ramdisk" | cpio -id --quiet 2>/dev/null || true)
# the init with the vf block (between the '>>> vf' and '<<< vf' marks) and the hold loop taken out, against m1's from its first command to the bind
strip_vf() { sed -e '/^# >>> vf:/,/^# <<< vf$/d' "$1" | sed -n '/^\/bin\/busybox --install/,/^if \[ -z "\$UDC" \]/p'; }
total=0
for L in a b c d; do
  S=vf$L
  SPEC=$T/vendor$S.spec
  echo $L > "$T/walleye-vf-$L"
  {
    for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
    # no devtmpfs on the vendor kernel: every node the init needs comes from the cpio
    echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"; echo "nod /dev/zero 666 0 0 c 1 5"
    echo "nod /dev/kmsg 600 0 0 c 1 11";   echo "nod /dev/tty 666 0 0 c 5 0";    echo "nod /dev/ptmx 666 0 0 c 5 2"
    echo "nod /dev/urandom 444 0 0 c 1 9"
    echo "file /bin/busybox $BB 755 0 0"
    echo "slink /bin/sh busybox 777 0 0"
    echo "file /init $I 755 0 0"
    echo "file /etc/walleye-vf $T/walleye-vf-$L 644 0 0"
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
  ck "header equals m1's" "$(hdr "$U")" "$(hdr "$MI")"
  ck "kernel blob is the control's" "$(h "$T/u$S/kernel")" "$(h "$P/kernel")"
  ck "kernel blob is m1's" "$(h "$T/u$S/kernel")" "$(h "$T/m1u/kernel")"
  ck "ramdisk is this step's xz" "$(h "$T/u$S/ramdisk")" "$(h "$OUT/initramfs-vendor-$S.cpio.xz")"
  ck "ramdisk is xz with CRC32 as the control's" "$(file -b "$T/u$S/ramdisk" | cut -d, -f1-2)" "$(file -b "$P/ramdisk" | cut -d, -f1-2)"
  ck "ramdisk has /init, /etc/walleye-vf, /bin/busybox, /dev/console, /dev/ptmx, /dev/kmsg" "$(xz -dc "$T/u$S/ramdisk" | cpio -t 2>/dev/null | grep -cx -e init -e etc/walleye-vf -e bin/busybox -e dev/console -e dev/ptmx -e dev/kmsg)" 6
  mkdir "$T/x$S" && (cd "$T/x$S" && xz -dc "$T/u$S/ramdisk" | cpio -id --quiet 2>/dev/null || true)
  ck "busybox in it is aarch64 static and m1's" "$(file -b "$T/x$S/bin/busybox" | cut -d, -f1-2)$(h "$T/x$S/bin/busybox")" "ELF 64-bit LSB executable, ARM aarch64$(h "$T/m1x/bin/busybox")"
  ck "the files in it are m1's plus /etc/walleye-vf, and nothing else (names, modes)" "$(cd "$T/x$S" && find . -mindepth 1 -printf '%p %m %y\n' | sort | grep -v -e '^./etc/walleye-vf ' | md5sum)" "$(cd "$T/m1x" && find . -mindepth 1 -printf '%p %m %y\n' | sort | md5sum)"
  ck "/etc/walleye-vf says '$L'" "$(cat "$T/x$S/etc/walleye-vf")" "$L"
  ck "size is the control's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  mkdir "$T/v$S" && cp "$IMG" "$T/v$S/boot.img"
  ck "AVB footer verifies" "$(avbtool verify_image --image "$T/v$S/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "footer property block equals m1's" "$(avbtool info_image --image "$IMG" | sed -n 's/^ *\(Prop\|Algorithm\|Partition Name\|Salt\).*/&/p' | sort | md5sum)" "$(avbtool info_image --image "$M1" | sed -n 's/^ *\(Prop\|Algorithm\|Partition Name\|Salt\).*/&/p' | sort | md5sum)"
  ck "the init in the ramdisk is the source init" "$(cmp -s "$T/x$S/init" "$I" && echo same || echo differs)" same
  ck "the init is the same in all four images (only /etc/walleye-vf differs)" "$(h "$T/x$S/init")" "$(h "$I")"
  ck "the init's m1 part (first command to the gadget bind, the vf block out) is m1's, line for line" "$(strip_vf "$I" | md5sum)" "$(strip_vf "$M1I" | md5sum)"
  ck "the window file is m1's 2 MiB file, made once (the line is m1's, whole)" "$(grep -cxF 'dd if=/dev/zero of=$LOGF bs=1024 count=2048 2>/dev/null' "$I")" 1
  ck "the payload is put in the window before the bind (fill_window comes before the UDC write)" "$(awk '/^fill_window$/{f=NR} /echo "\$UDC" > \$G\/UDC/{u=NR} END{print (f>0 && u>f) ? "before" : "after"}' "$I")" before
  ck "no 'set -e', and the init ends in a sleep loop" "$(grep -c -e '^set -e' "$I")$(grep -c '^while :; do sleep 3600; done$' "$I")" 01
  ck "nothing after the bind writes the payload: the hold loop writes only the beat block (seek=511)" "$(sed -n '/^# the hold/,$p' "$I" | grep -c -e 'dd if=' )$(sed -n '/^# the hold/,$p' "$I" | grep -c 'seek=511')" 11
  ck "every read of a source is bounded by timeout 5 and no command's stdout goes to /dev/null" "$(grep -v '^#' "$I" | grep -c 'timeout 5 cat')$(grep -v '^#' "$I" | grep -c -E '(^|[^2])> ?/dev/null')" 30
  ck "applets the init uses are in the busybox build's .config" "$(for ap in CAT DD MOUNT TIMEOUT SED TR STAT HEAD CUT PRINTF SLEEP REBOOT ECHO ASH LS SYNC UNAME MV MKDIR LN; do grep -q "^CONFIG_$ap=y" "$W/busybox-arm64.src/.config" && printf 1 || printf 0; done)"  11111111111111111111
  ck "init reads, never writes, the phone's files" "$(grep -v '^#' "$I" | grep -c -E 'mmcblk|/dev/block|/data|/system|mkfs|fastboot|dd of=/dev/(sd|mmc)')" 0
  echo "boot-$S: $pass ok, $fail failed"
  total=$((total+fail))
done
cmp -s "$T/x"vfa/init "$T/x"vfb/init && cmp -s "$T/x"vfc/init "$T/x"vfd/init && echo "ok   the four inits are one file" || { echo "FAIL the four inits differ"; total=$((total+1)); }
(cd "$OUT" && sha256sum initramfs-vendor-vf[abcd].cpio.xz boot-vf[abcd].img | tee SHA256SUMS-vfx)
[ $total -eq 0 ]
