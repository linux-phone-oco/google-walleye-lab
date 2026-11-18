#!/bin/sh
# t3 (backlog walleye-pstore-t3-wdog-bite): pk5 (our kernel + porthole's taimen dtb, alive) with TWO changes: the dtb's ramoops node in the phone's own region
# 0xa1100000 (as t1) and the ramdisk = init-t3 (marker to /dev/kmsg, busybox watchdog -T 10 -t 3600: one ping, then the SoC watchdog bites). Command line is pk5's. Host only.
#   build-t3.sh WORK_DIR PK5_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-t3.img,initramfs-t3.cpio.xz,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL).
set -eu
W=${1:?usage: build-t3.sh WORK_DIR PK5_BOOT_IMG OUT_DIR}
PK5=${2:?}
OUT=${3:?}
HERE=$(cd "$(dirname "$0")" && pwd)
BB=$W/busybox-arm64
GEN=$W/out/b/usr/gen_init_cpio
CFG=$W/out-ml/.config
I=$HERE/initramfs-mainline/init-t3
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
mkdir -p "$OUT"; rm -f "$OUT/boot-t3.img" "$OUT/SHA256SUMS" "$OUT/initramfs-t3.cpio.xz"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$GEN" "$BB" "$PK5" "$I" "$CFG"; do [ -s "$f" ] || { echo "build-t3: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$PK5" --out "$T/p")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split() {
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-t3: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
split "$T/p/kernel" "$T/k"
N=$(fdtget -l "$T/k.dtb" /reserved-memory | grep "^ramoops@" || true)
ck "the dtb has exactly one ramoops node" "$(echo "$N" | grep -c .)" 1
[ "$(echo "$N" | grep -c .)" = 1 ] || { echo "build-t3: not exactly one ramoops node in the dtb" >&2; exit 1; }
P=/reserved-memory/$N
cp "$T/k.dtb" "$T/new.dtb"
fdtput -t x "$T/new.dtb" "$P" reg 0x0 0xa1100000 0x0 0x200000
fdtput -t x "$T/new.dtb" "$P" record-size 0x80000
fdtput -t x "$T/new.dtb" "$P" console-size 0x20000
fdtput -t x "$T/new.dtb" "$P" pmsg-size 0
cat "$T/k.gz" "$T/new.dtb" > "$T/blob"
{
  for d in /bin /sbin /etc /dev /proc /sys /tmp /run; do echo "dir $d 755 0 0"; done
  echo "nod /dev/console 600 0 0 c 5 1"; echo "nod /dev/null 666 0 0 c 1 3"
  echo "file /bin/busybox $BB 755 0 0"
  echo "slink /bin/sh busybox 777 0 0"
  echo "file /init $I 755 0 0"
} > "$T/t3.spec"
"$GEN" -t 0 "$T/t3.spec" > "$T/rd.cpio"
xz --check=crc32 --lzma2=preset=6e -c "$T/rd.cpio" > "$OUT/initramfs-t3.cpio.xz"
PSIZE=$(stat -c %s "$PK5")
AV=$(avbtool info_image --image "$PK5")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-t3.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$OUT/initramfs-t3.cpio.xz" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
h() { sha256sum "$1" | cut -c1-16; }
g() { fdtget -t x "$T/uk.dtb" "$P" "$1"; }
hdr() { printf "%s\n" "$1" | grep -v -e 'ramdisk size' -e 'kernel size'; }
ck "ramoops reg read back (0xa1100000, 2 MiB)" "$(g reg)" "0 a1100000 0 200000"
ck "ramoops record-size read back" "$(g record-size)" 80000
ck "ramoops console-size read back" "$(g console-size)" 20000
ck "ramoops pmsg-size read back is 0" "$(g pmsg-size)" 0
ov=0
for c in $(fdtget -l "$T/uk.dtb" /reserved-memory); do
  [ "$c" = "$N" ] && continue
  set -- $(fdtget -t x "$T/uk.dtb" "/reserved-memory/$c" reg 2>/dev/null || echo 0 0 0 0)
  a=$((0x$2)); s=$((0x$4))
  [ $s -gt 0 ] && [ $a -lt $((0xa1300000)) ] && [ $((a+s)) -gt $((0xa1100000)) ] && ov=$((ov+1))
done
ck "no other reserved-memory node overlaps the new region" "$ov" 0
ck "Image.gz is pk5's" "$(h "$T/uk.gz")" "$(h "$T/k.gz")"
ck "the dtb has the kpss-wdt node (apss-wdt-msm8998, 0x17817000)" "$(fdtget -t s "$T/uk.dtb" /soc@0/watchdog@17817000 compatible 2>/dev/null | tr " " "\n" | grep -cx -e qcom,kpss-wdt -e qcom,apss-wdt-msm8998)" 2
ck "dtb has the same size as pk5's" "$(stat -c %s "$T/uk.dtb")" "$(stat -c %s "$T/k.dtb")"
f1=$(dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null | grep -v -e 'ramoops' -e 'b0000000' -e 'a1100000' -e 'record-size' -e 'console-size' -e 'pmsg-size' | sha256sum)
f2=$(dtc -I dtb -O dts "$T/uk.dtb" 2>/dev/null | grep -v -e 'ramoops' -e 'b0000000' -e 'a1100000' -e 'record-size' -e 'console-size' -e 'pmsg-size' | sha256sum)
ck "the dtb differs from pk5's only in the ramoops node's properties" "$f1" "$f2"
ck "header (offsets, page, os word, command line) equals pk5's but the sizes" "$(hdr "$U")" "$(hdr "$INFO")"
ck "command line has panic=5" "$(printf "%s\n" "$U" | sed -n 's/^command line args: *//p' | head -n1 | tr ' ' '\n' | grep -cx 'panic=5')" 1
ck "command line has no earlycon and no console=" "$(printf "%s\n" "$U" | sed -n 's/^command line args: *//p' | head -n1 | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
ck "ramdisk is this build's xz" "$(cmp -s "$T/u/ramdisk" "$OUT/initramfs-t3.cpio.xz" && echo same || echo differs)" same
mkdir "$T/x" && (cd "$T/x" && xz -dc "$T/u/ramdisk" | cpio -id --quiet 2>/dev/null || true)
ck "init in the ramdisk is init-t3" "$(cmp -s "$T/x/init" "$I" && echo same || echo differs)" same
ck "busybox in it is aarch64 static" "$(file -b "$T/x/bin/busybox" | grep -o -e "ARM aarch64" -e "statically linked" | wc -l)" 2
ck "init holds the marker the reader greps" "$(grep -c walleye-t3-marker-5c1e "$I")" 2
ck "the reader greps the same marker" "$(grep -c walleye-t3-marker-5c1e "$HERE/../tests/host/pstore-t3-read.sh")" 3
ck "init has no bashisms (busybox ash -n)" "$("$W/busybox-x86_64" ash -n "$I" 2>&1 || echo syntax-error)" ""
ck "init writes to /dev/kmsg" "$(grep -c "> /dev/kmsg" "$I")" 1
ck "init opens the watchdog with a 10 s timeout and one ping" "$(grep -c -e "^watchdog -T 10 -t 3600 -F /dev/watchdog &$" "$I")" 1
ck "init has no sysrq c" "$(grep -c "echo c" "$I")" 0
for a in ash mount sleep reboot uname sync echo watchdog; do
  ck "busybox has the applet $a (arm64 build config)" "$(grep -cx "CONFIG_$(echo "$a" | tr a-z A-Z)=y" "$W/busybox-arm64.src/.config")" 1
done
for k in WATCHDOG WATCHDOG_CORE MAGIC_SYSRQ PSTORE PSTORE_CONSOLE PSTORE_RAM QCOM_WDT DEVTMPFS RD_XZ BLK_DEV_INITRD; do
  ck "kernel config has CONFIG_$k" "$(grep -cE "^CONFIG_$k=y$" "$CFG")" 1
done
ck "sysrq is fully enabled by default (0x1)" "$(grep -c '^CONFIG_MAGIC_SYSRQ_DEFAULT_ENABLE=0x1$' "$CFG")" 1
ck "size is pk5's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
cp "$IMG" "$T/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-t3: checks failed, nothing written" >&2; rm -f "$OUT/initramfs-t3.cpio.xz"; exit 1; }
cp "$IMG" "$OUT/boot-t3.img"
(cd "$OUT" && sha256sum initramfs-t3.cpio.xz boot-t3.img > SHA256SUMS && cat SHA256SUMS)
