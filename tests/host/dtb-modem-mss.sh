#!/bin/sh
# walleye-dtb-modem-mss, run on the lab host: does the modem variant's dtb enable the modem subsystem on the phone's own memory layout and firmware names (item walleye-modem-mss-dtb)?
# Passes only when (1) remoteproc_mss is okay; (2) its firmware-name is, in order, qcom/msm8998/google/walleye/mba.mbn and qcom/msm8998/google/walleye/modem.mdt (the names of the phone's own modem_b
# partition files mba.mbn and modem.mdt, research walleye-modem-blobs-found); (3) its mba, mpss and metadata children point at the labels mba_mem, mpss_mem and mdata_mem; (4) mba_mem and mpss_mem
# equal, by address and size, the phone's running fdt (FIXTURE, the phone's own text: pil_mba_region, modem_region), both no-map; (5) mdata_mem is the 0x4000-byte no-map block mainline allocates; (6) rmtfs_mem is the
# qcom,rmtfs-mem no-map block - its ADDRESS is not judged: the phone's running fdt names no rmtfs region, so where it belongs is not known yet.
# Prints one line `pass|fail|skip walleye-dtb-modem-mss kernel DETAIL`; exit 0 only on pass. Read-only. Needs fdtget (dtc) and awk. Environment: DTB (no default: the image build names it), FIXTURE (default
# tests/fixtures/phone-text/walleye-running-fdt-memory.cap in this lab).
HERE=$(cd "$(dirname "$0")/../.." && pwd)
FIXTURE=${FIXTURE:-$HERE/tests/fixtures/phone-text/walleye-running-fdt-memory.cap}
say() { echo "$1 walleye-dtb-modem-mss kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtget >/dev/null || say skip "fdtget (dtc) not installed: not known yet"
[ -r "${DTB:-}" ] || say skip "need DTB=PATH readable: not known yet"
[ -r "$FIXTURE" ] || say skip "need the phone's text $FIXTURE: not known yet"
N=$(basename "$DTB")
regs() { for c in $1; do printf '%s\n' "0x${c#0x}"; done | xargs -n 4 printf '%d %d %d %d\n' | awk '{ printf "%d %d\n", $1*4294967296+$2, $3*4294967296+$4 }'; }
phonecells() { awk -v n="$1" '$1==n || index($1, n "@")==1 { f=1; next } f&&/reg = </{ gsub(/.*<|>;.*/,""); print; exit } f&&/^ *};/{ exit }' "$FIXTURE"; }
label() { fdtget "$DTB" /__symbols__ "$1" 2>/dev/null; }
RP=$(label remoteproc_mss) || say fail "$N: no remoteproc_mss label"
# (1) (2)
[ "$(fdtget "$DTB" "$RP" status)" = okay ] || say fail "$N: remoteproc_mss is not okay"
[ "$(fdtget -t s "$DTB" "$RP" firmware-name 2>/dev/null | tr '\n' ' ')" = "qcom/msm8998/google/walleye/mba.mbn qcom/msm8998/google/walleye/modem.mdt " ] || say fail "$N: remoteproc_mss's firmware-name is not mba.mbn then modem.mdt of the walleye folder"
# (3)
for pr in mba:mba_mem mpss:mpss_mem metadata:mdata_mem; do
  child=${pr%%:*}; lbl=${pr#*:}
  p=$(label "$lbl") || say fail "$N: no label $lbl"
  MR=$(fdtget -t u "$DTB" "$RP/$child" memory-region 2>/dev/null); PH=$(fdtget -t u "$DTB" "$p" phandle 2>/dev/null)
  { [ -n "$MR" ] && [ "$MR" = "$PH" ]; } || say fail "$N: remoteproc_mss/$child's memory-region ('$MR') is not $lbl's phandle ('$PH')"
done
# (4)
for pr in mba_mem:pil_mba_region mpss_mem:modem_region; do
  lbl=${pr%%:*}; pn=${pr#*:}
  p=$(label "$lbl"); o=$(regs "$(fdtget -t x "$DTB" "$p" reg)")
  ph=$(phonecells "$pn"); [ -n "$ph" ] || say fail "$FIXTURE has no reg for $pn"
  [ "$o" = "$(regs "$ph")" ] || say fail "$N: $lbl is '$o', the phone's $pn is '$(regs "$ph")'"
  fdtget -p "$DTB" "$p" | grep -qx no-map || say fail "$N: $lbl is not no-map"
done
# (5)
p=$(label mdata_mem) || say fail "$N: no label mdata_mem"
[ "$(fdtget -t x "$DTB" "$p" size)" = "0 4000" ] || say fail "$N: mdata_mem is not 0x4000 bytes"
fdtget -p "$DTB" "$p" | grep -qx no-map || say fail "$N: mdata_mem is not no-map"
# (6)
p=$(label rmtfs_mem) || say fail "$N: no label rmtfs_mem"
[ "$(fdtget "$DTB" "$p" compatible)" = qcom,rmtfs-mem ] || say fail "$N: rmtfs_mem is not qcom,rmtfs-mem"
fdtget -p "$DTB" "$p" | grep -qx no-map || say fail "$N: rmtfs_mem is not no-map"
say pass "$N: remoteproc_mss okay with mba.mbn and modem.mdt, mba_mem and mpss_mem equal the phone's running fdt, metadata 0x4000, rmtfs_mem a qcom,rmtfs-mem block (its address not known yet)"
