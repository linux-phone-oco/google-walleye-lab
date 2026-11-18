#!/bin/sh
# Images for backlog walleye-overlay-enables-what (its alternative): pk5 (our kernel + porthole's TAIMEN dtb) reaches init; every walleye dtb dies
# (ours, and porthole's own, pk2). So the walleye dtbs lack, or leave disabled, something the taimen dtb has. This builds pk5 with that
# difference put in group by group (the bisect from the alive side: w1..w5 and wr are one group each on pk5, only `wa` is cumulative: pk5 made walleye-like).
# Host only; nothing touches the phone.
#   build-taimen-walleye.sh PK5_BOOT_IMG PK2_DTB OUT_DIR
# Everything but the dtb is pk5's: header, command line, ramdisk (init-ml5a, reboot at once), our Image.gz, AVB footer.
# Groups (node paths in porthole's taimen dtb; the facts: node sets and status of taimen (pk1/pk5 dtb) against walleye (pk2 dtb)):
#   w1 camss@ca00000 and cci@ca0c000 set status disabled (the taimen dtb enables them, the walleye dtb does not)
#   w2 dma-controller@17184000, iommu@5100000, video-codec@cc00000, slim-ngd@171c0000 set disabled
#   w3 the five i2c masters i2c@c179000 c17a000 c1b5000 c1b6000 c1b7000 set disabled
#   w4 pcie@1c00000 and its phy@1c06000 set disabled
#   w5 pon resin key and the pmic@3 led-controller set disabled
#   wr the nodes only taimen has: fingerprint, gpio-keys, the seven camera regulators, sound, thermal-sensor-skin, thermal-zones/skin-thermal removed
#   wa w1..w5 and wr together: the node set and every status equal to pk2's (the check below)
# Writes OUT_DIR/boot-w1..w5,wr,wa.img, the dtbs and SHA256SUMS; checks (ok/FAIL, exit 1 on a FAIL) that each image differs from pk5's by the dtb only,
# that the dtb changed exactly what its group names, that wa's nodes and statuses equal pk2's, and that the stock dtbo overlay (entry 3) still merges.
# Each group also removes the children of what it disables, and wr+wa turn four pinctrl labels the overlay needs into empty stubs (as pk2 has). wr leaves consumers whose supplies or pin states it removed with dangling references ON PURPOSE (a dead wr then means "taimen-only nodes", read with that in mind).
# Environment: DTBO (dtbo.img, default the control's ~/Projects/walleye-build/control/dtbo.img; entry 3 is the overlay).
set -eu
PK5=${1:?usage: build-taimen-walleye.sh PK5_BOOT_IMG PK2_DTB OUT_DIR}
PK2=${2:?}
OUT=${3:?}
DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
mkdir -p "$OUT"; rm -f "$OUT/SHA256SUMS"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$PK5" "$PK2" "$DTBO"; do [ -s "$f" ] || { echo "build-taimen-walleye: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$PK5" --out "$T/pk5")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split() {
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-taimen-walleye: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
# node paths and status lines of a dtb, without symbols, fixups and phandles (the phandle numbers differ between any two dtbs)
flat() {
  dtc -I dtb -O dts "$1" 2>/dev/null | awk -v mode="$2" '
    /^[ \t]*[^ \t;=]+.*\{$/ { s=$0; sub(/^[ \t]+/,"",s); sub(/ *\{$/,"",s); sub(/^[^ ]*: /,"",s); if (s=="/") s=""; st[++d]=s; p=""; for(i=1;i<=d;i++) p=p "/" st[i]; if (mode=="nodes" && p !~ /^\/\/(__|vendor-overlay-stubs)/) print p; next }
    /^[ \t]*\};$/ { d--; next }
    /^[ \t]*status = / { p=""; for(i=1;i<=d;i++) p=p "/" st[i]; if (mode=="status" && p !~ /^\/\/(__|vendor-overlay-stubs)/) { v=$0; sub(/^[ \t]+/,"",v); print p " " v } }
  ' | sort
}
split "$T/pk5/kernel" "$T/k"
e=$(xxd -s $((32+32*3)) -l 8 -p "$DTBO")
dd if="$DTBO" of="$T/ov.dtb" bs=1 skip=$((0x$(echo "$e" | cut -c9-16))) count=$((0x$(echo "$e" | cut -c1-8))) 2>/dev/null
PSIZE=$(stat -c %s "$PK5")
AV=$(avbtool info_image --image "$PK5")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
hdr() { echo "$1" | grep -v -e 'kernel_size'; }
CMD=$(field 'command line args')
S=/soc@0
OFF1="$S/camss@ca00000 $S/cci@ca0c000"
OFF2="$S/dma-controller@17184000 $S/iommu@5100000 $S/video-codec@cc00000 $S/slim-ngd@171c0000"
OFF3="$S/i2c@c179000 $S/i2c@c17a000 $S/i2c@c1b5000 $S/i2c@c1b6000 $S/i2c@c1b7000"
OFF4="$S/pcie@1c00000 $S/phy@1c06000"
OFF5="$S/spmi@800f000/pmic@0/pon@800/resin $S/spmi@800f000/pmic@3/led-controller@d300"
GONE1="$S/camss@ca00000/ports/port@0 $S/camss@ca00000/ports/port@1 $S/cci@ca0c000/i2c-bus@0/camera@1a $S/cci@ca0c000/i2c-bus@0/lens@72 $S/cci@ca0c000/i2c-bus@0/ois@3e $S/cci@ca0c000/i2c-bus@1/camera@10 $S/cci@ca0c000/i2c-bus@1/eeprom@51"
GONE5="$S/spmi@800f000/pmic@3/led-controller@d300/led-0"
GONER="$S/remoteproc@17300000/glink-edge/apr/service@4/dais/dai@22 $S/remoteproc@17300000/glink-edge/apr/service@7/dais/dai@0 $S/remoteproc@17300000/glink-edge/apr/service@7/dais/dai@1 $S/remoteproc@17300000/glink-edge/apr/service@7/dais/dai@2 $S/remoteproc@17300000/glink-edge/apr/service@7/dais/dai@3 $S/spmi@800f000/pmic@0/adc@3100/channel@51 $S/spmi@800f000/pmic@0/gpio@c000/cam0-avdd2-en-state $S/spmi@800f000/pmic@0/gpio@c000/cam0-avdd-en-state $S/spmi@800f000/pmic@0/gpio@c000/cam0-drv-en-state $S/spmi@800f000/pmic@0/gpio@c000/cam0-dvdd-en-state $S/spmi@800f000/pmic@0/gpio@c000/cam0-vaf-en-state $S/spmi@800f000/pmic@2/gpio@c000/cam1-avdd-en-state $S/spmi@800f000/pmic@2/gpio@c000/cam1-dvdd-en-state"
GONE2="$S/slim-ngd@171c0000/slim@1"
GONE3="$S/i2c@c179000/touchscreen@49 $S/i2c@c17a000/vibrator@5a $S/i2c@c1b5000/audio-codec@4d $S/i2c@c1b6000/nfc@28 $S/i2c@c1b7000/pmic@8 $S/i2c@c1b7000/proximity@29"
GONE="/fingerprint /gpio-keys /regulator-cam0-avdd /regulator-cam0-avdd2 /regulator-cam0-drv /regulator-cam0-dvdd /regulator-cam0-vaf /regulator-cam1-avdd /regulator-cam1-dvdd /sound /thermal-sensor-skin /thermal-zones/skin-thermal"
GONE="$GONE $S/pinctrl@3400000/bcm15602-default-state $S/pinctrl@3400000/easel-default-state $S/pinctrl@3400000/fingerprint-irq-state $S/pinctrl@3400000/fingerprint-reset-state $S/pinctrl@3400000/laser-xshut-default-state $S/pinctrl@3400000/nfc-enable-active $S/pinctrl@3400000/nfc-firm-active $S/pinctrl@3400000/nfc-int-active $S/pinctrl@3400000/quat-mi2s-active $S/pinctrl@3400000/quat-mi2s-sd1-active $S/pinctrl@3400000/ts-i2c-switch $S/pinctrl@3400000/ts-int-n $S/pinctrl@3400000/ts-reset-n "
groups() { # image -> "off:NODES" and "gone:NODES" words
  case $1 in
    w1) echo "off $OFF1"; echo "gone $GONE1" ;; w2) echo "off $OFF2"; echo "gone $GONE2" ;; w3) echo "off $OFF3"; echo "gone $GONE3" ;; w4) echo "off $OFF4" ;; w5) echo "off $OFF5"; echo "gone $GONE5" ;;
    wr) echo "gone $GONE $GONER" ;; wa) echo "off $OFF1 $OFF2 $OFF3 $OFF4 $OFF5"; echo "gone $GONE $GONER $GONE1 $GONE2 $GONE3 $GONE5" ;;
  esac
}
flat "$T/k.dtb" nodes > "$T/k.nodes"; flat "$T/k.dtb" status > "$T/k.status"
for W in w1 w2 w3 w4 w5 wr wa; do
  cp "$T/k.dtb" "$T/$W.dtb"
  nph=0
  groups $W > "$T/$W.groups"
  while read -r kind nodes; do
    for n in $nodes; do
      ck "$W node $n is in the taimen dtb" "$(grep -cx "/$n" "$T/k.nodes")" 1
      if [ "$kind" = off ]; then
        ck "$W node $n was enabled" "$(fdtget "$T/k.dtb" "$n" status 2>/dev/null || echo none)" okay
        fdtput -t s "$T/$W.dtb" "$n" status disabled
        ck "$W node $n is disabled" "$(fdtget "$T/$W.dtb" "$n" status)" disabled
      else
        # four pinctrl states are labels the overlay needs: the walleye dtb has an empty stub node for each (as our dtb does), so give the label a stub
        case $n in
          */pinctrl@3400000/nfc-enable-active|*/pinctrl@3400000/nfc-int-active|*/pinctrl@3400000/quat-mi2s-active|*/pinctrl@3400000/quat-mi2s-sd1-active)
            lab=$(basename "$n" | tr - _)
            fdtput -c "$T/$W.dtb" "/vendor-overlay-stubs/$lab"
            ck "$W stub phandle 0x7f0$nph is free in the taimen dtb" "$(dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null | grep -c "phandle = <0x7f0$nph>")" 0
            fdtput -t x "$T/$W.dtb" "/vendor-overlay-stubs/$lab" phandle "0x7f0$nph"; nph=$((nph+1))
            fdtput -t s "$T/$W.dtb" /__symbols__ "$lab" "/vendor-overlay-stubs/$lab" ;;
        esac
        fdtput -r "$T/$W.dtb" "$n"
        ck "$W node $n is gone" "$(fdtget -l "$T/$W.dtb" "$(dirname "$n")" 2>/dev/null | grep -cx "$(basename "$n")")" 0
      fi
    done
  done < "$T/$W.groups"
  cat "$T/k.gz" "$T/$W.dtb" > "$T/blob-$W"
  IMG=$OUT/boot-$W.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$CMD" \
    --kernel "$T/blob-$W" --ramdisk "$T/pk5/ramdisk" -o "$IMG"
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  cp "$T/$W.dtb" "$OUT/$W.dtb"
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$W")
  split "$T/u$W/kernel" "$T/u$W/k"
  ck "$W header (offsets, page, os word, command line) equals pk5's but the sizes" "$(hdr "$U" | grep -v 'ramdisk size')" "$(hdr "$INFO" | grep -v 'ramdisk size')"
  ck "$W ramdisk is pk5's" "$(h "$T/u$W/ramdisk")" "$(h "$T/pk5/ramdisk")"
  ck "$W Image.gz is pk5's (ours)" "$(h "$T/u$W/k.gz")" "$(h "$T/k.gz")"
  ck "$W dtb differs from pk5's" "$([ "$(h "$T/u$W/k.dtb")" != "$(h "$T/k.dtb")" ] && echo yes || echo no)" yes
  ck "$W dtb is the one written" "$(h "$T/u$W/k.dtb")" "$(h "$T/$W.dtb")"
  ck "$W size is pk5's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  ck "$W command line has no earlycon and no console=" "$(echo "$CMD" | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
  mkdir "$T/v$W" && cp "$IMG" "$T/v$W/boot.img"
  ck "$W AVB footer verifies" "$(avbtool verify_image --image "$T/v$W/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb reads" "$(fdtdump "$T/$W.dtb" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb has /__symbols__" "$(fdtdump "$T/$W.dtb" 2>/dev/null | grep -c '__symbols__' | sed 's/^[1-9].*/some/')" some
  ck "$W dtb: the stock dtbo overlay (entry 3) merges" "$(fdtoverlay -i "$T/$W.dtb" -o "$T/merged-$W" "$T/ov.dtb" >/dev/null 2>&1 && echo merges || echo fails)" merges
  flat "$T/$W.dtb" status > "$T/$W.status"
  ck "$W status lines changed: exactly the disabled nodes" "$(diff "$T/k.status" "$T/$W.status" | grep -c '^>')" "$(awk '$1=="off"{c+=NF-1} END{print c+0}' "$T/$W.groups")"
done
# wa is pk5 made walleye-like at node and status level: what the pk2 dtb (porthole's own walleye dtb, dead) has
flat "$PK2" nodes > "$T/pk2.nodes"; flat "$PK2" status > "$T/pk2.status"
flat "$OUT/wa.dtb" nodes > "$T/wa.nodes"; flat "$OUT/wa.dtb" status > "$T/wa.status"
ck "wa node set equals pk2's" "$(diff "$T/wa.nodes" "$T/pk2.nodes" | wc -l)" 0
[ "$(diff "$T/wa.nodes" "$T/pk2.nodes" | wc -l)" -eq 0 ] || diff "$T/wa.nodes" "$T/pk2.nodes" | head -50
ck "wa status lines equal pk2's" "$(diff "$T/wa.status" "$T/pk2.status" | wc -l)" 0
ck "seven different dtbs" "$(for W in w1 w2 w3 w4 w5 wr wa; do h "$OUT/$W.dtb"; done | sort -u | wc -l)" 7
ck "pk5 dtb is the taimen one (enabled pcie, gpio-keys present)" "$(fdtget "$T/k.dtb" /soc@0/pcie@1c00000 status)$(grep -cx '//gpio-keys' "$T/k.nodes")" okay1
ck "pk2 dtb is the walleye one (no gpio-keys, pcie disabled)" "$(grep -cx '//gpio-keys' "$T/pk2.nodes")$(fdtget "$PK2" /soc@0/pcie@1c00000 status)" 0disabled
[ $total -eq 0 ] || { rm -f "$OUT"/boot-w?.img "$OUT"/boot-wa.img "$OUT"/boot-wr.img "$OUT"/w?.dtb "$OUT"/wa.dtb "$OUT"/wr.dtb "$OUT"/SHA256SUMS; echo "build-taimen-walleye: $total checks failed, nothing left in $OUT" >&2; exit 1; }
(cd "$OUT" && sha256sum boot-w1.img boot-w2.img boot-w3.img boot-w4.img boot-w5.img boot-wr.img boot-wa.img w1.dtb w2.dtb w3.dtb w4.dtb w5.dtb wr.dtb wa.dtb | tee SHA256SUMS)
[ $total -eq 0 ]
