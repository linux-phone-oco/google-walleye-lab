#!/bin/sh
# walleye-dtb-slpi-carveout, run on the lab host: does the dtb carry the phone's running memory layout for the sensor hub (item walleye-slpi-carveout-dts; decision 2026-11-23-walleye-slpi-carveout-running-layout)?
# Passes only when (1) every /reserved-memory node of the dtb with a reg lies inside the phone's DRAM (the /memory banks of the phone's own running fdt: the board dts leaves its /memory to the bootloader), no two overlap, and each lies
# inside one reserved region of the phone's fdt or touches none of them; (2) the SLPI block (label slpi_mem) is exactly 15 MiB (0xf00000), no-map; (3) mpss_mem, mba_mem, venus_mem, slpi_mem and ipa_gpu_mem equal, by address and size, the
# phone's running fdt entries (FIXTURE, the phone's own text: modem_region, pil_mba_region, pil_video_region, pil_slpi_region, pil_ipa_gpu_region); (4) the labels the layout deleted (ipa_fw_mem, ipa_gsi_mem, gpu_mem, wlan_msa_mem) are gone;
# (5) remoteproc_slpi is okay with firmware-name qcom/msm8998/google/walleye/slpi_v2.mdt and its memory-region is slpi_mem. Sources: research walleye-slpi-carveout-answer (the table, the firmware's program headers).
# Prints one line `pass|fail|skip walleye-dtb-slpi-carveout kernel DETAIL`; exit 0 only on pass. Read-only. Needs fdtget (dtc) and awk; no gawk extension.
# Environment: DTB (the dtb to read; no default: the image build names it), FIXTURE (default tests/fixtures/phone-text/walleye-running-fdt-memory.cap in this lab).
HERE=$(cd "$(dirname "$0")/../.." && pwd)
FIXTURE=${FIXTURE:-$HERE/tests/fixtures/phone-text/walleye-running-fdt-memory.cap}
say() { echo "$1 walleye-dtb-slpi-carveout kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtget >/dev/null || say skip "fdtget (dtc) not installed: not known yet"
[ -r "${DTB:-}" ] || say skip "need DTB=PATH readable: not known yet"
[ -r "$FIXTURE" ] || say skip "need the phone's text $FIXTURE: not known yet"
N=$(basename "$DTB")
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
# CELLS (hex words, with or without 0x) -> one "START SIZE" line per four cells, decimal
regs() { for c in $1; do printf '%s\n' "0x${c#0x}"; done | xargs -n 4 printf '%d %d %d %d\n' | awk '{ printf "%d %d\n", $1*4294967296+$2, $3*4294967296+$4 }'; }
# the cells of a dtb node's reg; empty when it has none
dtbreg() { fdtget -t x "$DTB" "$1" reg 2>/dev/null; }
# the cells of the fixture's node named $1 (a name, or name@addr): its own reg only
phonecells() { awk -v n="$1" '$1==n || index($1, n "@")==1 { f=1; next } f&&/reg = </{ gsub(/.*<|>;.*/,""); print; exit } f&&/^ *};/{ exit }' "$FIXTURE"; }
fdtget -t x "$DTB" /reserved-memory '#address-cells' >/dev/null 2>&1 || say fail "$N: no /reserved-memory"
[ "$(fdtget "$DTB" /reserved-memory '#address-cells') $(fdtget "$DTB" /reserved-memory '#size-cells')" = "2 2" ] || say fail "$N: /reserved-memory is not 2 address cells and 2 size cells (the reads below take four cells)"
# the dtb's blocks, "START SIZE NAME"
for n in $(fdtget -l "$DTB" /reserved-memory); do
  r=$(dtbreg "/reserved-memory/$n") || continue
  [ -n "$r" ] || continue
  [ "$(echo $r | wc -w)" = 4 ] || say fail "$N: /reserved-memory/$n has a reg that is not one block of four cells"
  echo "$(regs "$r") $n"
done > "$W/blocks"
[ -s "$W/blocks" ] || say fail "$N: no reserved block with a reg"
# the phone's DRAM banks (its /memory reg) and its reserved regions, from the fixture's text
regs "$(awk '/^    memory \{/{m=1} m&&/reg = </{ gsub(/.*<|>;.*/,""); print; exit }' "$FIXTURE")" > "$W/dram"
[ -s "$W/dram" ] || say fail "$FIXTURE holds no /memory reg"
awk '/^        [^ ]+ \{/{ n=$1 } /^        [^ ]+ \{/,/^        \};/ { if ($0 ~ /reg = </) { gsub(/.*<|>;.*/,""); print } }' "$FIXTURE" | while read -r cells; do regs "$cells"; done > "$W/phone"
[ -s "$W/phone" ] || say fail "$FIXTURE holds no reserved region"
# (1) inside DRAM, no overlap, inside one phone region or clear of all
awk -v dram="$W/dram" -v phone="$W/phone" '
  BEGIN { while ((getline l < dram) > 0) { split(l,a," "); nd++; ds[nd]=a[1]; de[nd]=a[1]+a[2] }
          while ((getline l < phone) > 0) { split(l,a," "); np++; ps[np]=a[1]; pe[np]=a[1]+a[2] } k=1 }
  { s=$1; e=$1+$2; ok=0; for(i=1;i<=nd;i++) if (s>=ds[i] && e<=de[i]) ok=1
    if (!ok) { print "outside DRAM: " $3; bad=1 }
    for(j=1;j<k;j++) if (s<E[j] && S[j]<e) { print "overlap: " $3 " and " N[j]; bad=1 }
    in1=0; hit=0; for(i=1;i<=np;i++) { if (s>=ps[i] && e<=pe[i]) in1=1; if (s<pe[i] && ps[i]<e) hit=1 }
    if (hit && !in1) { print "straddles a region of the phone: " $3; bad=1 }
    S[k]=s; E[k]=e; N[k]=$3; k++ }
  END { exit bad }' "$W/blocks" > "$W/why" || say fail "$N: $(tr '\n' ';' < "$W/why")"
# (2) the SLPI block
SY=$(fdtget "$DTB" /__symbols__ slpi_mem 2>/dev/null) || say fail "$N: no slpi_mem label"
[ "$(regs "$(dtbreg "$SY")" | cut -d' ' -f2)" = 15728640 ] || say fail "$N: slpi_mem is not 15 MiB"
fdtget -p "$DTB" "$SY" | grep -qx no-map || say fail "$N: slpi_mem is not no-map"
# (3) the phone's entries
for pr in mpss_mem:modem_region mba_mem:pil_mba_region venus_mem:pil_video_region slpi_mem:pil_slpi_region ipa_gpu_mem:pil_ipa_gpu_region; do
  lbl=${pr%%:*}; pn=${pr#*:}
  p=$(fdtget "$DTB" /__symbols__ "$lbl" 2>/dev/null) || say fail "$N: no label $lbl"
  o=$(regs "$(dtbreg "$p")")
  ph=$(phonecells "$pn"); [ -n "$ph" ] || say fail "$FIXTURE has no reg for $pn"
  [ "$o" = "$(regs "$ph")" ] || say fail "$N: $lbl is '$o', the phone's $pn is '$(regs "$ph")'"
done
# (4) the deleted labels
for l in ipa_fw_mem ipa_gsi_mem gpu_mem wlan_msa_mem; do
  fdtget "$DTB" /__symbols__ "$l" >/dev/null 2>&1 && say fail "$N: the label $l is still in the dtb"
done
# (5) the remoteproc
RP=$(fdtget "$DTB" /__symbols__ remoteproc_slpi 2>/dev/null) || say fail "$N: no remoteproc_slpi"
[ "$(fdtget "$DTB" "$RP" status)" = okay ] || say fail "$N: remoteproc_slpi is not okay"
[ "$(fdtget "$DTB" "$RP" firmware-name)" = qcom/msm8998/google/walleye/slpi_v2.mdt ] || say fail "$N: remoteproc_slpi's firmware-name is not slpi_v2.mdt of the walleye folder"
MR=$(fdtget -t u "$DTB" "$RP" memory-region 2>/dev/null); PH=$(fdtget -t u "$DTB" "$SY" phandle 2>/dev/null)
{ [ -n "$MR" ] && [ "$MR" = "$PH" ]; } || say fail "$N: remoteproc_slpi's memory-region ('$MR') is not slpi_mem's phandle ('$PH')"
say pass "$N: slpi_mem 15 MiB at the phone's running base, the five blocks equal its fdt, no overlap, inside its DRAM, remoteproc_slpi okay with slpi_v2.mdt"
