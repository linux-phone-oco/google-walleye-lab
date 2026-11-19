#!/bin/sh
# Images for backlog walleye-our-dtb-bisect: pk4 (porthole's kernel + OUR dtb; it dies, no init) with one group of nodes removed from OUR dtb each.
# If an image reaches init (Android back at ~42-50 s, not ~70-75 s), the group removed holds the node that stops boot. Host only; nothing touches the phone.
#   build-dtb-bisect.sh PK4_BOOT_IMG PK5_BOOT_IMG DTBO_IMG OUT_DIR
# Everything but the dtb is pk4's: header, command line, ramdisk (init-ml5a, reboot at once), porthole's Image.gz, AVB footer.
# Groups (node paths in our dtb; fdtput -r removes the node and what is under it):
#   d1 watchdog@17817000 (ours: kpss-wdt only, no timeout-sec; porthole has a node at the same address, compatible "qcom,apss-wdt-msm8998", timeout-sec 30)
#   d2 reserved-memory ramoops@a1100000 (ours; porthole's is at 0xb0000000)
#   d3 reserved-memory memory@94b00000 (our slpi carve-out; porthole's slpi is 0x95800000, 0xf00000)
#   d4 the RPM regulators (remoteproc glink-edge rpm-requests regulators-0 and regulators-1, ours; porthole's are pm8998-regulators and pmi8998-regulators)
#   d5 usb@a8f8800 (dwc3 glue) and phy@c012000 (qusb2 phy)
#   d6 d1 + d2 + d3 together (what we added or moved beside the carve-outs)
# pk4 (porthole's kernel + our dtb) is the known-dead image; pk5 (our kernel + porthole's dtb) and pk3 (porthole's kernel and dtb) are the known-alive ones; all built already. Removing a node also leaves what depends on it (supplies, phandles) dangling: a result names the node and its dependants. spmi@800f000 is not a group: the overlay needs it, an image without it would not merge.
# Writes OUT_DIR/boot-d1..d6.img, the dtbs and SHA256SUMS; checks (ok/FAIL, exit 1 on a FAIL) that each image differs from pk4's by the dtb only,
# that the dtb lost exactly the named nodes, and that the stock dtbo overlay (entry 3) still merges.
set -eu
PK4=${1:?usage: build-dtb-bisect.sh PK4_BOOT_IMG PK5_BOOT_IMG DTBO_IMG OUT_DIR}
PK5=${2:?}
DTBO=${3:?}
OUT=${4:?}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$PK4" "$PK5" "$DTBO"; do [ -s "$f" ] || { echo "build-dtb-bisect: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$PK4" --out "$T/pk4")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split() {
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-dtb-bisect: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
split "$T/pk4/kernel" "$T/k"
e=$(xxd -s $((32+32*3)) -l 8 -p "$DTBO")
dd if="$DTBO" of="$T/ov.dtb" bs=1 skip=$((0x$(echo "$e" | cut -c9-16))) count=$((0x$(echo "$e" | cut -c1-8))) 2>/dev/null
PSIZE=$(stat -c %s "$PK4")
AV=$(avbtool info_image --image "$PK4")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
hdr() { echo "$1" | grep -v -e 'kernel_size'; }
CMD=$(field 'command line args')
WDT=/soc@0/watchdog@17817000
RAM=/reserved-memory/ramoops@a1100000
SLP=/reserved-memory/memory@94b00000
RPM1=/remoteproc/glink-edge/rpm-requests/regulators-0
RPM2=/remoteproc/glink-edge/rpm-requests/regulators-1
USB=/soc@0/usb@a8f8800
PHY=/soc@0/phy@c012000
groups() {
  case $1 in
    d1) echo "$WDT" ;; d2) echo "$RAM" ;; d3) echo "$SLP" ;; d4) echo "$RPM1 $RPM2" ;; d5) echo "$USB $PHY" ;; d6) echo "$WDT $RAM $SLP" ;;
  esac
}
for W in d1 d2 d3 d4 d5 d6; do
  cp "$T/k.dtb" "$T/$W.dtb"
  for n in $(groups $W); do
    ck "$W node $n is in our dtb" "$(fdtget -l "$T/k.dtb" "$(dirname "$n")" 2>/dev/null | grep -cx "$(basename "$n")")" 1
    fdtput -r "$T/$W.dtb" "$n"
    ck "$W node $n is gone" "$(fdtget -l "$T/$W.dtb" "$(dirname "$n")" 2>/dev/null | grep -cx "$(basename "$n")")" 0
  done
  cat "$T/k.gz" "$T/$W.dtb" > "$T/blob-$W"
  IMG=$OUT/boot-$W.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$CMD" \
    --kernel "$T/blob-$W" --ramdisk "$T/pk4/ramdisk" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  cp "$T/$W.dtb" "$OUT/$W.dtb"
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$W")
  split "$T/u$W/kernel" "$T/u$W/k"
  ck "$W header (offsets, page, os word, command line) equals pk4's but the sizes" "$(hdr "$U" | grep -v 'ramdisk size')" "$(hdr "$INFO" | grep -v 'ramdisk size')"
  ck "$W ramdisk is pk4's" "$(h "$T/u$W/ramdisk")" "$(h "$T/pk4/ramdisk")"
  ck "$W Image.gz is pk4's (porthole's)" "$(h "$T/u$W/k.gz")" "$(h "$T/k.gz")"
  ck "$W dtb differs from pk4's" "$([ "$(h "$T/u$W/k.dtb")" != "$(h "$T/k.dtb")" ] && echo yes || echo no)" yes
  ck "$W dtb is the one written" "$(h "$T/u$W/k.dtb")" "$(h "$T/$W.dtb")"
  ck "$W size is pk4's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  ck "$W command line has no earlycon and no console=" "$(echo "$CMD" | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
  mkdir "$T/v$W" && cp "$IMG" "$T/v$W/boot.img"
  ck "$W AVB footer verifies" "$(avbtool verify_image --image "$T/v$W/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb reads" "$(fdtdump "$T/$W.dtb" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb has /__symbols__" "$(fdtdump "$T/$W.dtb" 2>/dev/null | grep -c '__symbols__' | sed 's/^[1-9].*/some/')" some
  ck "$W dtb: the stock dtbo overlay (entry 3) merges" "$(fdtoverlay -i "$T/$W.dtb" -o "$T/merged-$W" "$T/ov.dtb" >/dev/null 2>&1 && echo merges || echo fails)" merges
done
# the dtbs stand pairwise apart; pk4 is ours (ramoops@a1100000), pk5 is porthole's (ramoops@b0000000)
ck "six different dtbs" "$(for W in d1 d2 d3 d4 d5 d6; do h "$OUT/$W.dtb"; done | sort -u | wc -l)" 6
ck "pk4 dtb is ours: has ramoops@a1100000, not ramoops@b0000000" "$(fdtget -l "$T/k.dtb" /reserved-memory | grep -c -e '^ramoops@a1100000$')$(fdtget -l "$T/k.dtb" /reserved-memory | grep -c -e '^ramoops@b0000000$')" 10
unpack_bootimg --boot_img "$PK5" --out "$T/pk5" >/dev/null; split "$T/pk5/kernel" "$T/p5"
ck "pk5 dtb is porthole's: has ramoops@b0000000, not ramoops@a1100000" "$(fdtget -l "$T/p5.dtb" /reserved-memory | grep -c -e '^ramoops@b0000000$')$(fdtget -l "$T/p5.dtb" /reserved-memory | grep -c -e '^ramoops@a1100000$')" 10
ck "pk5 dtb differs from pk4's" "$([ "$(h "$T/p5.dtb")" != "$(h "$T/k.dtb")" ] && echo yes || echo no)" yes
[ $total -eq 0 ] || { rm -f "$OUT"/boot-d?.img "$OUT"/d?.dtb "$OUT"/SHA256SUMS; echo "build-dtb-bisect: $total checks failed, nothing left in $OUT" >&2; exit 1; }
(cd "$OUT" && sha256sum boot-d1.img boot-d2.img boot-d3.img boot-d4.img boot-d5.img boot-d6.img d1.dtb d2.dtb d3.dtb d4.dtb d5.dtb d6.dtb | tee SHA256SUMS)
[ $total -eq 0 ]
