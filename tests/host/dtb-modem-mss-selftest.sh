#!/bin/sh
# Known cases for dtb-modem-mss.sh: the good dtb passes, the board dtb without the modem fails, and each defect it names is made in a copy of the good dtb and fails with its own word (a check is trusted only after it
# fails on a known bad case). Source: the phone's own text, the check's fixture (tests/fixtures/phone-text/walleye-running-fdt-memory.cap); the defects are edits of the good dtb.
# Environment: DTB (the good modem dtb), BOARD_DTB (the board dtb built from dt/msm8998-google-walleye.dts, same tree). Exit 0 only when every case came out as expected.
HERE=$(cd "$(dirname "$0")" && pwd)
CHECK=$HERE/dtb-modem-mss.sh
for t in fdtget fdtput; do command -v $t >/dev/null || { echo "skip walleye-dtb-modem-mss-selftest kernel $t (dtc) not installed: not known yet"; exit 1; }; done
[ -r "${DTB:-}" ] && [ -r "${BOARD_DTB:-}" ] || { echo "skip walleye-dtb-modem-mss-selftest kernel need DTB and BOARD_DTB readable: not known yet"; exit 1; }
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
bad=0
want() { # NAME WORD DTB: the check must fail with WORD in its line
  out=$(DTB="$3" sh "$CHECK") && { echo "FAIL $1: the check passed"; bad=1; return; }
  case "$out" in *"$2"*) echo "ok   $1";; *) echo "FAIL $1: '$out' lacks '$2'"; bad=1;; esac
}
sym() { fdtget "$DTB" /__symbols__ "$1"; }
case_dtb() { cp "$DTB" "$W/$1.dtb"; echo "$W/$1.dtb"; }
if out=$(DTB="$DTB" sh "$CHECK"); then echo "ok   the good dtb passes"; else echo "FAIL the good dtb: $out"; bad=1; fi
want "the board dtb (modem off)" "not okay" "$BOARD_DTB"
RP=$(sym remoteproc_mss); MB=$(sym mba_mem); MP=$(sym mpss_mem); MD=$(sym mdata_mem); RM=$(sym rmtfs_mem)
d=$(case_dtb off);    fdtput -t s "$d" "$RP" status disabled;                                  want "remoteproc_mss disabled" "not okay" "$d"
d=$(case_dtb fw);     fdtput -t s "$d" "$RP" firmware-name qcom/msm8998/mba.mbn qcom/msm8998/modem.mdt; want "another firmware folder" "firmware-name" "$d"
d=$(case_dtb fwone);  fdtput -t s "$d" "$RP" firmware-name qcom/msm8998/google/walleye/mba.mbn; want "one firmware name" "firmware-name" "$d"
d=$(case_dtb mrnone); fdtput -d "$d" "$RP/mba" memory-region;                                 want "mba without a memory-region" "memory-region" "$d"
d=$(case_dtb mpss);   fdtput -t x "$d" "$MP" reg 0 8cc00000 0 7000000;                        want "mpss_mem of mainline's size" "the phone's modem_region" "$d"
d=$(case_dtb mba);    fdtput -t x "$d" "$MB" reg 0 94100000 0 200000;                         want "mba_mem at mainline's base" "mba_mem is" "$d"
d=$(case_dtb mbanm);  fdtput -d "$d" "$MB" no-map;                                            want "mba_mem without no-map" "mba_mem is not no-map" "$d"
d=$(case_dtb mdsize); fdtput -t x "$d" "$MD" size 0 8000;                                     want "metadata of another size" "mdata_mem is not 0x4000" "$d"
d=$(case_dtb rmcomp); fdtput -t s "$d" "$RM" compatible shared-dma-pool;                      want "rmtfs_mem of another kind" "not qcom,rmtfs-mem" "$d"
d=$(case_dtb nosym);  fdtput -d "$d" /__symbols__ mdata_mem;                                  want "no mdata_mem label" "no label mdata_mem" "$d"
d=$(case_dtb rmmpss); fdtput -t x "$d" "$RM" reg 0 8cc00000 0 200000;                         want "rmtfs_mem on mpss_mem" "overlaps mpss_mem" "$d"
d=$(case_dtb rmfar);  fdtput -t x "$d" "$RM" reg 2 0 0 200000;                                want "rmtfs_mem past the DRAM" "outside the phone's DRAM" "$d"
d=$(case_dtb rmsize); fdtput -t x "$d" "$RM" reg 0 88f00000 0 1000;                           want "rmtfs_mem of 4 KiB" "not 2 MiB" "$d"
d=$(case_dtb rmcl);   fdtput -t u "$d" "$RM" qcom,client-id 9;                                want "rmtfs_mem client 9" "client-id" "$d"
d=$(case_dtb mpnm);   fdtput -d "$d" "$MP" no-map;                                            want "mpss_mem without no-map" "mpss_mem is not no-map" "$d"
d=$(case_dtb rmnm);   fdtput -d "$d" "$RM" no-map;                                            want "rmtfs_mem without no-map" "rmtfs_mem is not no-map" "$d"
d=$(case_dtb fwswap); fdtput -t s "$d" "$RP" firmware-name qcom/msm8998/google/walleye/modem.mdt qcom/msm8998/google/walleye/mba.mbn; want "firmware names swapped" "firmware-name" "$d"
d=$(case_dtb mpmr);   fdtput -t u "$d" "$RP/mpss" memory-region "$(fdtget -t u "$DTB" "$MB" phandle)"; want "mpss pointing at mba_mem" "mpss's memory-region" "$d"
d=$(case_dtb mdmr);   fdtput -d "$d" "$RP/metadata" memory-region;                            want "metadata without a memory-region" "metadata's memory-region" "$d"
exit $bad
