#!/bin/sh
# Known cases for dtb-slpi-carveout.sh: a good dtb passes, and each defect it names is made in a copy of that dtb and fails with its own word (a check is trusted only after it fails on a known bad case).
# Source: the phone's own text, the check's fixture (tests/fixtures/phone-text/walleye-running-fdt-memory.cap); the defects are edits of the good dtb. Environment: DTB (the good dtb, as the check's).
# Exit 0 only when every case came out as expected.
HERE=$(cd "$(dirname "$0")" && pwd)
CHECK=$HERE/dtb-slpi-carveout.sh
for t in fdtget fdtput; do command -v $t >/dev/null || { echo "skip walleye-dtb-slpi-carveout-selftest kernel $t (dtc) not installed: not known yet"; exit 1; }; done
[ -r "${DTB:-}" ] || { echo "skip walleye-dtb-slpi-carveout-selftest kernel need DTB=PATH readable: not known yet"; exit 1; }
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
bad=0
want() { # NAME WORD DTB: the check must fail with WORD in its line
  out=$(DTB="$3" sh "$CHECK") && { echo "FAIL $1: the check passed"; bad=1; return; }
  case "$out" in *"$2"*) echo "ok   $1";; *) echo "FAIL $1: '$out' lacks '$2'"; bad=1;; esac
}
sym() { fdtget "$DTB" /__symbols__ "$1"; }
case_dtb() { cp "$DTB" "$W/$1.dtb"; echo "$W/$1.dtb"; }
if out=$(DTB="$DTB" sh "$CHECK"); then echo "ok   the good dtb passes"; else echo "FAIL the good dtb: $out"; bad=1; fi
SL=$(sym slpi_mem); MB=$(sym mba_mem); VE=$(sym venus_mem); MP=$(sym mpss_mem); IG=$(sym ipa_gpu_mem); RP=$(sym remoteproc_slpi)
d=$(case_dtb small);    fdtput -t x "$d" "$SL" reg 0 94c00000 0 700000; want "a 7 MiB slpi_mem" "not 15 MiB" "$d"
d=$(case_dtb moved);    fdtput -t x "$d" "$SL" reg 0 95c00000 0 f00000; want "slpi_mem at another base" "pil_slpi_region" "$d"
d=$(case_dtb nomap);    fdtput -d "$d" "$SL" no-map;                    want "slpi_mem without no-map" "not no-map" "$d"
d=$(case_dtb overlap);  fdtput -t x "$d" "$MB" reg 0 94b00000 0 200000; want "mba_mem inside slpi_mem" "overlap" "$d"
d=$(case_dtb outside);  fdtput -t x "$d" "$MB" reg 1 80000000 0 200000; want "a block past the phone's DRAM" "outside DRAM" "$d"
d=$(case_dtb easel);    fdtput -t x "$d" "$IG" reg 0 95c00000 0 100000; want "ipa_gpu_mem inside the phone's easel region" "ipa_gpu_mem is" "$d"
d=$(case_dtb venus);    fdtput -t x "$d" "$VE" reg 0 94700000 0 400000; want "venus_mem of another size" "venus_mem is" "$d"
d=$(case_dtb mpss);     fdtput -t x "$d" "$MP" reg 0 8cc00000 0 7000000; want "mpss_mem of mainline's size" "mpss_mem is" "$d"
d=$(case_dtb mba);      fdtput -t x "$d" "$MB" reg 0 94500000 0 100000; want "mba_mem of another size" "mba_mem is" "$d"
d=$(case_dtb straddle); fdtput -t x "$d" "$IG" reg 0 95b80000 0 100000; want "ipa_gpu_mem straddling the end of the phone's block" "straddles" "$d"
d=$(case_dtb off);      fdtput -t s "$d" "$RP" status disabled;         want "remoteproc_slpi disabled" "not okay" "$d"
d=$(case_dtb fw);       fdtput -t s "$d" "$RP" firmware-name qcom/msm8998/slpi.mdt; want "another firmware name" "firmware-name" "$d"
d=$(case_dtb nomr);     fdtput -d "$d" "$RP" memory-region;             want "remoteproc_slpi without a memory-region" "memory-region" "$d"
d=$(case_dtb mrnone);   fdtput -d "$d" "$RP" memory-region; fdtput -d "$d" "$SL" phandle; want "no memory-region and no phandle on slpi_mem" "memory-region" "$d"
d=$(case_dtb nosym);    fdtput -d "$d" /__symbols__ slpi_mem;           want "no slpi_mem label" "no slpi_mem label" "$d"
for l in ipa_fw_mem ipa_gsi_mem gpu_mem wlan_msa_mem; do
  d=$(case_dtb "back-$l"); fdtput -t s "$d" /__symbols__ "$l" /reserved-memory; want "the label $l back" "label $l is still" "$d"
done
exit $bad
