#!/bin/sh
# pk5l2 (backlog walleye-pk5l-ramoops): pk5l with ONLY the dtb's ramoops node edited (fdtput on porthole's taimen dtb inside the kernel blob):
# the phone's own region, as in decision 2026-11-17-walleye-ramoops-at-the-phones-region. Kernel Image.gz, ramdisk, header and command line are pk5l's.
# Host only; nothing touches the phone.
#   build-pk5l2.sh PK5L_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-pk5l2.img,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL).
set -eu
PK5L=${1:?usage: build-pk5l2.sh PK5L_BOOT_IMG OUT_DIR}
OUT=${2:?}
CFG=${CFG:-$HOME/Projects/walleye-build/out-ml/.config}  # config of pk5l's Image.gz (ml6a's kernel), not read from the image
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$PK5L" "$CFG"; do [ -s "$f" ] || { echo "build-pk5l2: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$PK5L" --out "$T/p")
field() { printf "%s\n" "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
# split KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
split() {
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-pk5l2: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
split "$T/p/kernel" "$T/k"
N=$(fdtget -l "$T/k.dtb" /reserved-memory | grep "^ramoops@" || true)
ck "the dtb has exactly one ramoops node" "$(echo "$N" | grep -c .)" 1
[ "$(echo "$N" | grep -c .)" = 1 ] || { echo "build-pk5l2: not exactly one ramoops node in the dtb" >&2; exit 1; }
cp "$T/k.dtb" "$T/new.dtb"
P=/reserved-memory/$N
fdtput -t x "$T/new.dtb" "$P" reg 0x0 0xa1100000 0x0 0x200000
fdtput -t x "$T/new.dtb" "$P" record-size 0x80000
fdtput -t x "$T/new.dtb" "$P" console-size 0x20000
fdtput -t x "$T/new.dtb" "$P" pmsg-size 0
cat "$T/k.gz" "$T/new.dtb" > "$T/blob"
PSIZE=$(stat -c %s "$PK5L")
AV=$(avbtool info_image --image "$PK5L")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$T/boot-pk5l2.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$(field 'command line args')" \
  --kernel "$T/blob" --ramdisk "$T/p/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u")
split "$T/u/kernel" "$T/uk"
h() { sha256sum "$1" | cut -c1-16; }
g() { fdtget -t x "$T/uk.dtb" "$P" "$1"; }
ck "ramoops reg read back (0xa1100000, 2 MiB)" "$(g reg)" "0 a1100000 0 200000"
ck "ramoops record-size read back" "$(g record-size)" 80000
ck "ramoops console-size read back" "$(g console-size)" 20000
ck "ramoops pmsg-size read back is 0" "$(g pmsg-size)" 0
ck "no reserved-memory node is left in the old region 0xb0000000" "$(for c in $(fdtget -l "$T/uk.dtb" /reserved-memory); do fdtget -t x "$T/uk.dtb" "/reserved-memory/$c" reg 2>/dev/null; done | grep -c b0000000)" 0
# overlap: no other reserved-memory node reg (addr,size) touches 0xa1100000..0xa1300000
ov=0
for c in $(fdtget -l "$T/uk.dtb" /reserved-memory); do
  [ "$c" = "$N" ] && continue
  set -- $(fdtget -t x "$T/uk.dtb" "/reserved-memory/$c" reg 2>/dev/null || echo 0 0 0 0)
  a=$((0x$2)); s=$((0x$4))
  [ $s -gt 0 ] && [ $a -lt $((0xa1300000)) ] && [ $((a+s)) -gt $((0xa1100000)) ] && ov=$((ov+1))
done
ck "no other reserved-memory node overlaps the new region" "$ov" 0
ck "Image.gz is pk5l's" "$(h "$T/uk.gz")" "$(h "$T/k.gz")"
ck "ramdisk is pk5l's" "$(cmp -s "$T/p/ramdisk" "$T/u/ramdisk" && echo same || echo differs)" same
ck "dtb differs from pk5l's" "$([ "$(h "$T/uk.dtb")" != "$(h "$T/k.dtb")" ] && echo yes || echo no)" yes
ck "dtb has the same size as pk5l's" "$(stat -c %s "$T/uk.dtb")" "$(stat -c %s "$T/k.dtb")"
# only the ramoops node differs: compare the two dtbs as source with the node's lines removed
ck "dtc reads both dtbs" "$(dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null | wc -l | sed "s/^0$/none/;s/^[1-9].*/some/")$(dtc -I dtb -O dts "$T/uk.dtb" 2>/dev/null | wc -l | sed "s/^0$/none/;s/^[1-9].*/some/")" somesome
dd1=$(dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null | grep -v -e 'ramoops' -e 'b0000000' -e 'a1100000' -e 'record-size' -e 'console-size' -e 'pmsg-size' | sha256sum)
dd2=$(dtc -I dtb -O dts "$T/uk.dtb" 2>/dev/null | grep -v -e 'ramoops' -e 'b0000000' -e 'a1100000' -e 'record-size' -e 'console-size' -e 'pmsg-size' | sha256sum)
ck "the dtb differs from pk5l's only in the ramoops node's properties" "$dd1" "$dd2"
ck "header (offsets, page, os word, command line, sizes) equals pk5l's" "$(echo "$U")" "$(echo "$INFO")"
ck "command line has no earlycon and no console=" "$(printf "%s\n" "$U" | sed -n 's/^command line args: *//p' | head -n1 | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
ck "size is pk5l's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
for k in PSTORE PSTORE_CONSOLE PSTORE_RAM; do
  ck "kernel config has CONFIG_$k" "$(grep -cE "^CONFIG_$k=[ym]$" "$CFG")" 1
done
cp "$IMG" "$T/boot.img"
ck "AVB footer verifies" "$(avbtool verify_image --image "$T/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
[ $total -eq 0 ] || { echo "build-pk5l2: checks failed, nothing written" >&2; exit 1; }
cp "$IMG" "$OUT/boot-pk5l2.img"
(cd "$OUT" && sha256sum boot-pk5l2.img > SHA256SUMS && cat SHA256SUMS)
[ $total -eq 0 ]
