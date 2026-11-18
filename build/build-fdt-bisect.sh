#!/bin/sh
# Images for backlog walleye-compare-with-the-running-fdt: `wpa` (our kernel + the alive taimen dtb carrying all of porthole's walleye properties; alive: 60/60 s)
# with ONE group of its differences to OUR dtb (dead) made equal to ours - the bisect from the alive side toward ours. Host only; nothing touches the phone.
#   build-fdt-bisect.sh WPA_BOOT_IMG OURS_DTB OUT_DIR
# The groups, in the order of suspicion (research walleye-fdt-compare: what the alive taimen tree and the phone's running fdt have and ours lacks):
#   x1 pinctrl@3400000 gpio-reserved-ranges removed (<0 4>, <81 4>: the pins the secure world owns; ours has none, taimen/porthole have it)
#   x2 /chosen empty (#address-cells #size-cells ranges removed) and /reserved-memory as ours (ramoops@a1810000, memory@94b00000 for the modem remoteproc; porthole's b0000000, 95800000, f6900000 gone)
#   x3 cpu scaling gone: the three opp tables, cpufreq-hw, the two power-managers, the power-controller, the two lmh, per-cpu opp/power-domain/freq-domain/cooling properties, system-pc idle state, capacity-dmips-mhz as ours
#   x4 the platform nodes ours lacks: restart@10ac000 (qcom,pshold), watchdog@17817000 (apps wdt), imem@146bf000, pmu@1008000
#   x5 the seven interconnect@ providers and the interconnects/interconnect-names of their consumers
#   x6 statuses as ours: display, dsi, display phy, gpu, ufshc and its phy, the three remoteprocs, serial@c171000, wifi disabled; status removed from the clock controllers, two iommus, display-controller
#   x7 what x1..x6 leave: the remaining nodes and properties only wpa has (qfprom cells, pmic children, thermal extras, apr, camss, cci, panel, rpm regulators ...), root compatible, the rpmcc clock specifier, spmi interrupts, and the properties only ours has
#   xa x1..x7 together: differs from ours only in the lines pinned in build/fdt-bisect-xa-left.txt (the ours-only rpm regulators and overlay stubs, emptied overlay targets, supplies of consumers of removed regulators, renumbered phandle lists)
# A deleted node that the stock dtbo overlay (entry 3) needs is EMPTIED instead (properties and children gone, the node stays: ramoops@b0000000, cci, three pmics), so the overlay keeps merging. Output is staged and moved into OUT only when every check passed.
# Everything but the dtb is wpa's: header, command line, ramdisk (reboot at once), Image.gz, AVB footer. Checks (ok/FAIL, exit 1 on a FAIL, nothing left in OUT_DIR then).
# Environment: DTBO (dtbo.img, default the control's ~/Projects/walleye-build/control/dtbo.img).
set -eu
WPA=${1:?usage: build-fdt-bisect.sh WPA_BOOT_IMG OURS_DTB OUT_DIR}
OURS=${2:?}
OUT=${3:?}
DTBO=${DTBO:-$HOME/Projects/walleye-build/control/dtbo.img}
HERE=$(cd "$(dirname "$0")" && pwd)
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT; STG=$T/stage; mkdir -p "$STG"
for f in "$WPA" "$OURS" "$DTBO" "$HERE/dts-norm.awk"; do [ -s "$f" ] || { echo "build-fdt-bisect: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$WPA" --out "$T/wpa")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
split() {
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-fdt-bisect: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
# node paths of a dtb (no labels), one per line, sorted; the property lines of a dtb, phandles resolved to paths, labels stripped
nodes() { dtc -I dtb -O dts "$1" 2>/dev/null | awk '/^[ \t]*[^ \t;=]+.*\{$/ { s=$0; sub(/^[ \t]+/,"",s); sub(/ *\{$/,"",s); sub(/^[^ ]*: /,"",s); if (s=="/") s=""; st[++d]=s; p=""; for(i=1;i<=d;i++) p=p "/" st[i]; if (p=="") p="/"; sub(/^\/\//,"/",p); print p; next } /^[ \t]*\};$/ { d-- }' | grep -v -e '^/__' | sort; }
norm() { dtc -I dtb -O dts -s "$1" 2>/dev/null > "$2.dts"; gawk -f "$HERE/dts-norm.awk" "$2.dts" "$2.dts" | LC_ALL=C sort | grep -v '^//__symbols__' | sed -E 's#[A-Za-z0-9_,.+-]+: ##g; s#^//#/#' | sort > "$2"; }
split "$T/wpa/kernel" "$T/k"
e=$(xxd -s $((32+32*3)) -l 8 -p "$DTBO")
dd if="$DTBO" of="$T/ov.dtb" bs=1 skip=$((0x$(echo "$e" | cut -c9-16))) count=$((0x$(echo "$e" | cut -c1-8))) 2>/dev/null
PSIZE=$(stat -c %s "$WPA")
AV=$(avbtool info_image --image "$WPA")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
hdr() { echo "$1" | grep -v -e 'kernel_size'; }
CMD=$(field 'command line args')
S=/soc@0
PH0=$(dtc -I dtb -O dts "$T/k.dtb" 2>/dev/null | sed -n 's/^[ \t]*phandle = <\(0x[0-9a-f]*\)>;.*/\1/p' | while read -r p; do echo $((p)); done | sort -n | tail -n1)
PH1=$(printf '0x%x' $((PH0+1))); PH2=$(printf '0x%x' $((PH0+2)))
# the node paths the overlay's fragments target (resolved through the base dtb's own __symbols__): a node on this list is emptied, never removed
dtc -I dtb -O dts "$T/ov.dtb" 2>/dev/null | awk '/__fixups__/{f=1;next} f&&/^[ \t]*[A-Za-z0-9_]+ = /{n=$1; print n} /^[ \t]*};/{if(f)f=0}' | sort -u > "$T/ov-fix-labels"
: > "$T/ov-paths"
while IFS= read -r l; do p=$(fdtget -ts "$T/k.dtb" /__symbols__ "$l" 2>/dev/null || true); [ -n "$p" ] && echo "$p" >> "$T/ov-paths"; done < "$T/ov-fix-labels"
sort -u -o "$T/ov-paths" "$T/ov-paths"
exists() { fdtget -p "$1" "$2" >/dev/null 2>&1; }
# remove a node, or empty it when the overlay needs it (or one of its children)
rmn() {
  exists "$1" "$2" || return 0
  if grep -q -x -e "$2" -e "$2/.*" "$T/ov-paths" 2>/dev/null || awk -v p="$2" 'index($0,p "/")==1{f=1} END{exit !f}' "$T/ov-paths"; then
    for c in $(fdtget -l "$1" "$2"); do rmn "$1" "$2/$c"; done
    for pr in $(fdtget -p "$1" "$2"); do [ "$pr" = phandle ] || fdtput -d "$1" "$2" "$pr"; done
  else fdtput -r "$1" "$2"; fi
}
dp() { f=$1; n=$2; shift 2; exists "$f" "$n" || return 0; for pr in "$@"; do fdtget "$f" "$n" "$pr" >/dev/null 2>&1 && fdtput -d "$f" "$n" "$pr"; done; return 0; }
cp_cells() { v=$(fdtget -t x "$OURS" "$2" "$3"); fdtput -t x "$1" "$2" "$3" $v; }
g_x1() { dp "$1" $S/pinctrl@3400000 gpio-reserved-ranges; }
g_x2() {
  dp "$1" /chosen '#address-cells' '#size-cells' ranges
  for n in memory@95800000 ramoops@b0000000; do rmn "$1" /reserved-memory/$n; done
  dp "$1" /reserved-memory/memory@95600000 compatible
  fdtput -c "$1" /reserved-memory/memory@94b00000 /reserved-memory/ramoops@a1810000
  for p in reg; do cp_cells "$1" /reserved-memory/memory@94b00000 $p; cp_cells "$1" /reserved-memory/ramoops@a1810000 $p; done
  for p in record-size console-size pmsg-size; do cp_cells "$1" /reserved-memory/ramoops@a1810000 $p; done
  fdtput -t x "$1" /reserved-memory/memory@94b00000 no-map; fdtput -t s "$1" /reserved-memory/ramoops@a1810000 compatible ramoops
  fdtput -t x "$1" /reserved-memory/memory@94b00000 phandle $PH1; fdtput -t x "$1" /reserved-memory/ramoops@a1810000 phandle $PH2
  fdtput -t x "$1" $S/remoteproc@5800000 memory-region $PH1
}
g_x3() {
  for n in /cpu-gold-opp-table /cpu-silver-opp-table /cpr-opp-table /cpus/idle-states/system-pc $S/cpufreq-hw@17914800 $S/power-manager@17812000 $S/power-manager@17912000 $S/power-controller@179c8000 $S/lmh@179cc800 $S/lmh@179ce800; do rmn "$1" $n; done
  for c in 0 1 2 3 100 101 102 103; do
    n=/cpus/cpu@$c
    dp "$1" $n '#cooling-cells' operating-points-v2 power-domain-names power-domains qcom,freq-domain
    cp_cells "$1" $n capacity-dmips-mhz
    v=$(fdtget -t x "$1" $n cpu-idle-states | awk '{print $1, $2}'); fdtput -t x "$1" $n cpu-idle-states $v
  done
}
g_x4() { for n in $S/restart@10ac000 $S/watchdog@17817000 $S/imem@146bf000 $S/pmu@1008000; do rmn "$1" $n; done; }
g_x5() {
  for a in 1000000 1500000 1620000 1660000 1700000 1740000 17900000; do rmn "$1" $S/interconnect@$a; done
  for n in display-subsystem@c900000 gpu@5000000 ufshc@1da4000 usb@a8f8800 video-codec@cc00000; do dp "$1" $S/$n interconnects interconnect-names; done
}
g_x6() {
  for n in display-subsystem@c900000 display-subsystem@c900000/dsi@c994000 display-subsystem@c900000/phy@c994400 gpu@5000000 ufshc@1da4000 phy@1da7000 remoteproc@17300000 remoteproc@4080000 remoteproc@5800000 serial@c171000 wifi@18800000; do exists "$1" $S/$n && fdtput -t s "$1" $S/$n status disabled; done
  for n in clock-controller@5065000 clock-controller@c8c0000 iommu@5040000 iommu@cd00000 display-subsystem@c900000/display-controller@c901000; do dp "$1" $S/$n status; done
}
g_x7() {
  # top-most nodes only wpa has
  for n in $(sed -n '/^\/__/!p' "$T/only-wpa7" | awk '{ if (last=="" || index($0, last "/")!=1) { print; last=$0 } }'); do rmn "$1" "$n"; done
  # properties only wpa has on nodes both have
  while IFS='|' read -r n p; do dp "$1" "$n" "$p"; done < "$T/po-wpa7"
  # properties only ours has (empty ones and plain cells; phandle-valued ones stay out and are named by the check)
  while IFS='|' read -r n p; do
    case $p in *supply) continue ;; esac
    v=$(fdtget -t x "$OURS" "$n" "$p" 2>/dev/null || true); fdtput -t x "$1" "$n" "$p" $v
  done < "$T/po-ours"
  fdtput -t s "$1" / compatible google,walleye google,wahoo qcom,msm8998
  fdtput -t s "$1" $S/sram@778000 compatible qcom,rpm-msg-ram
  cp_cells "$1" $S/gpu@5000000/opp-table/opp-515000097 opp-level; cp_cells "$1" $S/gpu@5000000/opp-table/opp-670000048 opp-level
  fdtput -t s "$1" $S/video-codec@cc00000 clock-names core iface bus mbus
  fdtput -t x "$1" $S/video-codec@cc00000 power-domains $(fdtget -t x "$1" $S/video-codec@cc00000 power-domains | awk '{print $1, $2}')
  fdtput -t x "$1" $S/video-codec@cc00000 clocks $(fdtget -t x "$1" $S/video-codec@cc00000 clocks | awk '{for(i=1;i<=8;i++) printf "%s ", $i}')
  cp_cells "$1" $S/spmi@800f000 interrupts; dp "$1" $S/spmi@800f000 interrupts-extended
  for n in clock-controller@100000 clock-controller@5065000; do fdtput -t x "$1" $S/$n clocks $(fdtget -t x "$1" $S/$n clocks | awk '{$2=0; print}'); done
}
norm "$T/k.dtb" "$T/wpa.norm"; norm "$OURS" "$T/ours.norm"
nodes "$T/k.dtb" > "$T/wpa.nodes"; nodes "$OURS" > "$T/ours.nodes"
comm -23 "$T/wpa.nodes" "$T/ours.nodes" > "$T/only-wpa"
# the propery lines as (path|name <TAB> value)
kv() { awk '{i=index($0," = "); l=substr($0,1,i-1); v=substr($0,i+3); m=split(l,a," "); name=a[m]; path=substr(l,1,length(l)-length(name)-1); print path "|" name "\t" v}' "$1" | sort -t'	' -k1,1; }
kv "$T/wpa.norm" > "$T/wpa.kv"; kv "$T/ours.norm" > "$T/ours.kv"
cut -f1 "$T/wpa.kv" | sort > "$T/wk"; cut -f1 "$T/ours.kv" | sort > "$T/ok"
comm -23 "$T/wk" "$T/ok" | awk -F'|' -v f="$T/ours.nodes" 'BEGIN{while((getline l < f)>0) o[l]=1} o[$1]{print}' > "$T/po-wpa"
comm -13 "$T/wk" "$T/ok" | awk -F'|' -v f="$T/wpa.nodes" 'BEGIN{while((getline l < f)>0) o[l]=1} o[$1]{print}' > "$T/po-ours"
# x7 is what x1..x6 leave: the only-wpa nodes and properties still present once they are applied (the groups change disjoint lines)
cp "$T/k.dtb" "$T/g16.dtb"; for g in x1 x2 x3 x4 x5 x6; do "g_$g" "$T/g16.dtb"; done
nodes "$T/g16.dtb" > "$T/g16.nodes"; norm "$T/g16.dtb" "$T/g16.norm"; kv "$T/g16.norm" | cut -f1 | sort > "$T/g16.keys"
cut -d'|' -f1 "$T/g16.keys" | sort -u > "$T/g16.haveprops"
# a node x1..x6 already emptied (no property, no child left) is not x7's
comm -12 "$T/only-wpa" "$T/g16.nodes" | awk -v hp="$T/g16.haveprops" -v nd="$T/g16.nodes" 'BEGIN{while((getline l < hp)>0) p[l]=1; while((getline l < nd)>0) a[++n]=l} {k=0; if ($0 in p) k=1; else for(i=1;i<=n;i++) if (index(a[i], $0 "/")==1) {k=1; break}; if (k) print}' > "$T/only-wpa7"; sort "$T/po-wpa" | comm -12 - "$T/g16.keys" > "$T/po-wpa7"
exp_x1="0 0 1 0"; exp_x2="1 2 13 8"; exp_x3="92 0 504 16"; exp_x4="18 0 45 0"; exp_x5="7 0 32 1"; exp_x6="0 0 16 11"; exp_x7="158 0 613 29"; exp_xa="276 2 1222 63"
GROUPS_="x1 x2 x3 x4 x5 x6 x7"
for W in $GROUPS_ xa; do
  cp "$T/k.dtb" "$T/$W.dtb"
  case $W in xa) for g in $GROUPS_; do "g_$g" "$T/$W.dtb"; done ;; *) "g_$W" "$T/$W.dtb" ;; esac
  cat "$T/k.gz" "$T/$W.dtb" > "$T/blob-$W"
  IMG=$STG/boot-$W.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$CMD" \
    --kernel "$T/blob-$W" --ramdisk "$T/wpa/ramdisk" -o "$IMG"
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  cp "$T/$W.dtb" "$STG/$W.dtb"
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$W")
  split "$T/u$W/kernel" "$T/u$W/k"
  ck "$W header (offsets, page, os word, command line) equals wpa's but the sizes" "$(hdr "$U" | grep -v 'ramdisk size')" "$(hdr "$INFO" | grep -v 'ramdisk size')"
  ck "$W ramdisk is wpa's" "$(h "$T/u$W/ramdisk")" "$(h "$T/wpa/ramdisk")"
  ck "$W Image.gz is wpa's (ours)" "$(h "$T/u$W/k.gz")" "$(h "$T/k.gz")"
  ck "$W dtb differs from wpa's" "$([ "$(h "$T/u$W/k.dtb")" != "$(h "$T/k.dtb")" ] && echo yes || echo no)" yes
  ck "$W dtb is the one written" "$(h "$T/u$W/k.dtb")" "$(h "$T/$W.dtb")"
  ck "$W size is wpa's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  ck "$W command line has no earlycon and no console=" "$(echo "$(unpack_bootimg --boot_img "$IMG" --out "$T/c$W" | sed -n "s/^command line args: *//p")" | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
  mkdir "$T/v$W" && cp "$IMG" "$T/v$W/boot.img"
  ck "$W AVB footer verifies" "$(avbtool verify_image --image "$T/v$W/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb reads" "$(fdtdump "$T/$W.dtb" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb: the stock dtbo overlay (entry 3) merges" "$(fdtoverlay -i "$T/$W.dtb" -o "$T/merged-$W" "$T/ov.dtb" >/dev/null 2>&1 && echo merges || echo fails)" merges
  norm "$T/$W.dtb" "$T/$W.norm"; nodes "$T/$W.dtb" > "$T/$W.nodes"
  echo "info $W: $(diff "$T/wpa.nodes" "$T/$W.nodes" | grep -c '^<') nodes gone, $(diff "$T/wpa.nodes" "$T/$W.nodes" | grep -c '^>') nodes added, $(diff "$T/wpa.norm" "$T/$W.norm" | grep -c '^<') property lines gone/changed, $(diff "$T/wpa.norm" "$T/$W.norm" | grep -c '^>') added/changed"
done
ck "x1 changes exactly gpio-reserved-ranges" "$(diff "$T/wpa.norm" "$T/x1.norm" | grep '^[<>]' | sed 's/ = .*//' | sort -u | tr '\n' ',')" "< /soc@0/pinctrl@3400000 gpio-reserved-ranges,"
ck "x1 changes no node" "$(diff "$T/wpa.nodes" "$T/x1.nodes" | wc -l)" 0
ck "x4 removes exactly the four platform nodes" "$(diff "$T/wpa.nodes" "$T/x4.nodes" | grep '^<' | grep -v 'pmu@1008000/' | tr '\n' ' ')" "< /soc@0/imem@146bf000 < /soc@0/imem@146bf000/pil-reloc@94c < /soc@0/pmu@1008000 < /soc@0/restart@10ac000 < /soc@0/watchdog@17817000 "
# every group's size against wpa, pinned: nodes gone, nodes added, property lines gone/changed, added/changed
cnt() { echo "$(diff "$T/wpa.nodes" "$T/$1.nodes" | grep -c '^<') $(diff "$T/wpa.nodes" "$T/$1.nodes" | grep -c '^>') $(diff "$T/wpa.norm" "$T/$1.norm" | grep -c '^<') $(diff "$T/wpa.norm" "$T/$1.norm" | grep -c '^>')"; }
for g in $GROUPS_ xa; do eval "want=\$exp_$g"; ck "$g size against wpa (nodes gone/added, property lines gone/added)" "$(cnt $g)" "$want"; done
# the groups change disjoint lines: no node and no property line is changed by two of them
changed() { { diff "$T/wpa.nodes" "$T/$1.nodes" | grep '^[<>]' | sed 's/^/N /'; diff "$T/wpa.norm" "$T/$1.norm" | grep '^[<>]' | sed 's/^/P /'; } | sort -u > "$T/ch-$1"; }
for g in $GROUPS_; do changed $g; done
for a in $GROUPS_; do for b in $GROUPS_; do [ "$a" \< "$b" ] || continue; [ -z "${SHOW_OVERLAP:-}" ] || comm -12 "$T/ch-$a" "$T/ch-$b" | sed "s/^/overlap $a $b: /"; ov=0; case "$a $b" in "x3 x7"|"x4 x5") ov=1 ;; esac; ck "$a and $b change disjoint lines (but a property line whose target the other group removed: $ov)" "$(comm -12 "$T/ch-$a" "$T/ch-$b" | wc -l)" $ov; done; done
# references left dangling or raw (informational: a group that removes a provider leaves its consumers outside the group pointing at nothing, on purpose)
for g in $GROUPS_ xa; do echo "info $g: $(grep -c -e '?0x' -e '<RAW' "$T/$g.norm") unresolved references (wpa: $(grep -c -e '?0x' -e '<RAW' "$T/wpa.norm"), ours: $(grep -c -e '?0x' -e '<RAW' "$T/ours.norm"))"; done
# xa against ours: the exact lines that differ (nodes N, properties P; < xa, > ours) are pinned in fdt-bisect-xa-left.txt: emptied overlay targets, ours-only regulators and stubs, the rpm clock specifier, the supplies of consumers of removed regulators
{ diff "$T/xa.nodes" "$T/ours.nodes" | grep '^[<>]' | sed 's/^/N /'; diff "$T/xa.norm" "$T/ours.norm" | grep '^[<>]' | sed 's/^/P /'; } > "$T/xa-left"
if [ -n "${WRITE_XA_LEFT:-}" ]; then cp "$T/xa-left" "$WRITE_XA_LEFT"; fi
ck "xa differs from ours in exactly the pinned lines (fdt-bisect-xa-left.txt)" "$(sha256sum < "$T/xa-left")" "$(sha256sum < "$HERE/fdt-bisect-xa-left.txt")"
[ "$(sha256sum < "$T/xa-left")" = "$(sha256sum < "$HERE/fdt-bisect-xa-left.txt")" ] || diff "$HERE/fdt-bisect-xa-left.txt" "$T/xa-left" | head -30
ck "eight different dtbs" "$(for W in $GROUPS_ xa; do h "$STG/$W.dtb"; done | sort -u | wc -l)" 8
[ $total -eq 0 ] || { echo "build-fdt-bisect: $total checks failed, nothing written to $OUT" >&2; exit 1; }
(cd "$STG" && sha256sum boot-x1.img boot-x2.img boot-x3.img boot-x4.img boot-x5.img boot-x6.img boot-x7.img boot-xa.img x1.dtb x2.dtb x3.dtb x4.dtb x5.dtb x6.dtb x7.dtb xa.dtb > SHA256SUMS)
mkdir -p "$OUT"; rm -f "$OUT"/boot-x?.img "$OUT"/x?.dtb "$OUT"/SHA256SUMS
cp "$STG"/boot-x?.img "$STG"/x?.dtb "$STG"/SHA256SUMS "$OUT"/
cat "$OUT/SHA256SUMS"
