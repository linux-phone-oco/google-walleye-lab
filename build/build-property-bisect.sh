#!/bin/sh
# Images for backlog walleye-property-bisect: `wa` (our kernel + the taimen dtb with every node and status of the walleye dtb; alive: 42/63/63 s) with ONE group of the
# PROPERTY differences to porthole's walleye dtb (pk2, dead: 69-78 s) copied in. Host only; nothing touches the phone.
#   build-property-bisect.sh WA_BOOT_IMG PK2_DTB OUT_DIR
# The property diff wa -> pk2 (phandles resolved to node paths by dts-norm.awk, research walleye-property-diff) is five groups, nothing in /memory, reserved-memory, chosen, cpus, timer, psci:
#   wp1 root compatible + model are the walleye's ("google,walleye", "google,wahoo", "qcom,msm8998", "qcom,msm8998-mtp"; "Google Pixel 2")
#   wp2 battery capacity: /battery charge-full-design-microamp-hours and the fuel-gauge qcom,battery-capacity-ua from pk2 (0x2932e0, wa has 0x35b600)
#   wp3 the panel (dsi@c994000/panel@0) has no compatible (wa has "lg,sw43402")
#   wp4 qcom,protection-domain = "avs/audio", "msm/adsp/audio_pd" on the seven apr services (q6core q6afe q6asm q6adm q6cvp q6cvs q6mvm), which wa lacks
#   wp5 leftovers inside nodes that are disabled in both: camss vdda/vdd_sec supplies removed, pcie reg and ranges from pk2, pcie vddpe-3v3-supply and pcie@0 endpoint-powered-off-in-suspend removed, pon resin linux,code removed
#   wpa all five: wa's properties equal pk2's (the check below; three raw lists - ipa iommus, mmcc clocks, pcie iommu-map - are only renumbered phandles and stay out of the comparison)
# Everything but the dtb is wa's: header, command line, ramdisk (reboot at once), Image.gz, AVB footer. Checks (ok/FAIL, exit 1 on a FAIL, nothing left in OUT_DIR then).
# Environment: DTBO (dtbo.img, default the control's ~/Projects/walleye-build/control/dtbo.img; entry 3 is the overlay).
set -eu
WA=${1:?usage: build-property-bisect.sh WA_BOOT_IMG PK2_DTB OUT_DIR}
PK2=${2:?}
OUT=${3:?}
DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
HERE=$(cd "$(dirname "$0")" && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
mkdir -p "$OUT"; rm -f "$OUT/SHA256SUMS"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$WA" "$PK2" "$DTBO" "$HERE/dts-norm.awk"; do [ -s "$f" ] || { echo "build-property-bisect: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$WA" --out "$T/wa")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split() {
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-property-bisect: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
norm() { dtc -I dtb -O dts -s "$1" 2>/dev/null > "$2.dts"; gawk -f "$HERE/dts-norm.awk" "$2.dts" "$2.dts" | LC_ALL=C sort | grep -v '^//__symbols__' > "$2"; }
split "$T/wa/kernel" "$T/k"
e=$(xxd -s $((32+32*3)) -l 8 -p "$DTBO")
dd if="$DTBO" of="$T/ov.dtb" bs=1 skip=$((0x$(echo "$e" | cut -c9-16))) count=$((0x$(echo "$e" | cut -c1-8))) 2>/dev/null
PSIZE=$(stat -c %s "$WA")
AV=$(avbtool info_image --image "$WA")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
hdr() { echo "$1" | grep -v -e 'kernel_size'; }
CMD=$(field 'command line args')
S=/soc@0
APR="$S/remoteproc@17300000/glink-edge/apr"
SVC="service@3 service@4 service@7 service@8 service@b service@a service@9"
PCIE=$S/pcie@1c00000
PANEL=$S/display-subsystem@c900000/dsi@c994000/panel@0
FG=$S/spmi@800f000/pmic@2/fuel-gauge@4000
# copy a property from pk2 to the image's dtb (as cells), or delete it
cp_cells() { v=$(fdtget -t x "$PK2" "$2" "$3"); fdtput -t x "$1" "$2" "$3" $v; }
del() { fdtput -d "$1" "$2" "$3"; }
g_wp1() { fdtput -t s "$1" / compatible google,walleye google,wahoo qcom,msm8998 qcom,msm8998-mtp; fdtput -t s "$1" / model "Google Pixel 2"; }
g_wp2() { cp_cells "$1" /battery charge-full-design-microamp-hours; cp_cells "$1" "$FG" qcom,battery-capacity-ua; }
g_wp3() { del "$1" "$PANEL" compatible; }
g_wp4() { for s in $SVC; do fdtput -t s "$1" "$APR/$s" qcom,protection-domain avs/audio msm/adsp/audio_pd; done; }
g_wp5() {
  del "$1" $S/camss@ca00000 vdda-supply; del "$1" $S/camss@ca00000 vdd_sec-supply
  cp_cells "$1" $PCIE reg; cp_cells "$1" $PCIE ranges; del "$1" $PCIE vddpe-3v3-supply; del "$1" $PCIE/pcie@0 endpoint-powered-off-in-suspend
  del "$1" $S/spmi@800f000/pmic@0/pon@800/resin linux,code
}
# expected changed lines of the normalized property diff wa -> image: removed (in wa only) / added (in the image only)
exp_wp1="2 2"; exp_wp2="2 2"; exp_wp3="1 0"; exp_wp4="0 7"; exp_wp5="7 2"; exp_wpa="12 13"
norm "$T/k.dtb" "$T/wa.norm"
norm "$PK2" "$T/pk2.norm"
for W in wp1 wp2 wp3 wp4 wp5 wpa; do
  cp "$T/k.dtb" "$T/$W.dtb"
  case $W in wpa) for g in wp1 wp2 wp3 wp4 wp5; do "g_$g" "$T/$W.dtb"; done ;; *) "g_$W" "$T/$W.dtb" ;; esac
  cat "$T/k.gz" "$T/$W.dtb" > "$T/blob-$W"
  IMG=$OUT/boot-$W.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$CMD" \
    --kernel "$T/blob-$W" --ramdisk "$T/wa/ramdisk" -o "$IMG"
sh "$(dirname "$0")/ufs-gate.sh" "$IMG" || exit 1
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  cp "$T/$W.dtb" "$OUT/$W.dtb"
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$W")
  split "$T/u$W/kernel" "$T/u$W/k"
  ck "$W header (offsets, page, os word, command line) equals wa's but the sizes" "$(hdr "$U" | grep -v 'ramdisk size')" "$(hdr "$INFO" | grep -v 'ramdisk size')"
  ck "$W ramdisk is wa's" "$(h "$T/u$W/ramdisk")" "$(h "$T/wa/ramdisk")"
  ck "$W Image.gz is wa's (ours)" "$(h "$T/u$W/k.gz")" "$(h "$T/k.gz")"
  ck "$W dtb differs from wa's" "$([ "$(h "$T/u$W/k.dtb")" != "$(h "$T/k.dtb")" ] && echo yes || echo no)" yes
  ck "$W dtb is the one written" "$(h "$T/u$W/k.dtb")" "$(h "$T/$W.dtb")"
  ck "$W size is wa's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  ck "$W command line has no earlycon and no console=" "$(echo "$(unpack_bootimg --boot_img "$IMG" --out "$T/c$W" | sed -n "s/^command line args: *//p")" | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
  mkdir "$T/v$W" && cp "$IMG" "$T/v$W/boot.img"
  ck "$W AVB footer verifies" "$(avbtool verify_image --image "$T/v$W/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb reads" "$(fdtdump "$T/$W.dtb" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb: the stock dtbo overlay (entry 3) merges" "$(fdtoverlay -i "$T/$W.dtb" -o "$T/merged-$W" "$T/ov.dtb" >/dev/null 2>&1 && echo merges || echo fails)" merges
  norm "$T/$W.dtb" "$T/$W.norm"
  d=$(diff "$T/wa.norm" "$T/$W.norm" || true)
  eval "want=\$exp_$W"
  ck "$W changes exactly its group: property lines removed/added against wa" "$(echo "$d" | grep -c '^<') $(echo "$d" | grep -c '^>')" "$want"
done
# wpa: every property line equals pk2's, but three raw lists whose phandles are only renumbered (ipa iommus, mmcc clocks, pcie iommu-map)
skip='soc@0/ipa: ipa@1e40000 iommus =|soc@0/mmcc: clock-controller@c8c0000 clocks =|soc@0/pcie0: pcie@1c00000 iommu-map ='
grep -Ev "$skip" "$T/wpa.norm" > "$T/wpa.cmp"; grep -Ev "$skip" "$T/pk2.norm" > "$T/pk2.cmp"
# the nodes and statuses wa already made equal to pk2's: the node set is not part of this comparison, only properties; lines of nodes absent from either side are the node diff and must not exist
ck "wpa property lines equal pk2's" "$(diff "$T/wpa.cmp" "$T/pk2.cmp" | wc -l)" 0
[ "$(diff "$T/wpa.cmp" "$T/pk2.cmp" | wc -l)" -eq 0 ] || diff "$T/wpa.cmp" "$T/pk2.cmp" | head -40
ck "six different dtbs" "$(for W in wp1 wp2 wp3 wp4 wp5 wpa; do h "$OUT/$W.dtb"; done | sort -u | wc -l)" 6
[ $total -eq 0 ] || { rm -f "$OUT"/boot-wp?.img "$OUT"/boot-wpa.img "$OUT"/wp?.dtb "$OUT"/wpa.dtb "$OUT"/SHA256SUMS; echo "build-property-bisect: $total checks failed, nothing left in $OUT" >&2; exit 1; }
(cd "$OUT" && sha256sum boot-wp1.img boot-wp2.img boot-wp3.img boot-wp4.img boot-wp5.img boot-wpa.img wp1.dtb wp2.dtb wp3.dtb wp4.dtb wp5.dtb wpa.dtb | tee SHA256SUMS)
