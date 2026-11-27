#!/bin/sh
# Known cases for dtb-modem-ipa.sh: the good dtb passes, the board dtb (no ipa) fails, and each defect it names is made in a copy of the good dtb and fails with its own word (a check is trusted only after it fails on a known bad case).
# Source of the good values: tests/fixtures/kernel-text/msm8998-ipa-sources.txt. Every value in the check names its block of that file: A, A3, B, B2 (the node), C (clock numbers), C2 (gsi-loader), C3 (the firmware set); the numbers are written in the check, not read from it. Environment: DTB (the good modem dtb), BOARD_DTB (the board dtb, same tree). Exit 0 only when every case came out as expected.
HERE=$(cd "$(dirname "$0")" && pwd)
CHECK=$HERE/dtb-modem-ipa.sh
for t in fdtget fdtput; do command -v $t >/dev/null || { echo "skip walleye-dtb-modem-ipa-selftest kernel $t (dtc) not installed: not known yet"; exit 1; }; done
[ -r "${DTB:-}" ] && [ -r "${BOARD_DTB:-}" ] || { echo "skip walleye-dtb-modem-ipa-selftest kernel need DTB and BOARD_DTB readable: not known yet"; exit 1; }
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
bad=0
want() { # NAME WORD DTB
  out=$(DTB="$3" sh "$CHECK") && { echo "FAIL $1: the check passed"; bad=1; return; }
  case "$out" in *"$2"*) echo "ok   $1";; *) echo "FAIL $1: '$out' lacks '$2'"; bad=1;; esac
}
case_dtb() { cp "$DTB" "$W/$1.dtb"; echo "$W/$1.dtb"; }
if out=$(DTB="$DTB" sh "$CHECK"); then echo "ok   the good dtb passes"; else echo "FAIL the good dtb: $out"; bad=1; fi
want "the board dtb (no ipa)" "no ipa label" "$BOARD_DTB"
IPA=$(fdtget "$DTB" /__symbols__ ipa); SO=$(fdtget "$DTB" /__symbols__ ipa_smp2p_out); SI=$(fdtget "$DTB" /__symbols__ ipa_smp2p_in)
d=$(case_dtb off);   fdtput -t s "$d" "$IPA" status disabled;                              want "ipa disabled" "is not okay" "$d"
d=$(case_dtb comp);  fdtput -t s "$d" "$IPA" compatible qcom,sdm845-ipa;                    want "sdm845's compatible" "not qcom,msm8998-ipa" "$d"
d=$(case_dtb reg);   fdtput -t x "$d" "$IPA" reg 1e00000 34000 1e84000 31fff 1e04000 2c000; want "wahoo's downstream reg" "ipa's reg" "$d"
d=$(case_dtb gsi);   fdtput -t x "$d" "$IPA" reg 1e40000 7000 1e47000 2000 1e04000 2000;   want "a short gsi range" "ipa's reg" "$d"
d=$(case_dtb iom);   P=$(fdtget -t u "$DTB" "$IPA" iommus | cut -d" " -f1); fdtput -t u "$d" "$IPA" iommus "$P" 6369 "$P" 6370; want "stream id 0x18e1" "iommus" "$d"
d=$(case_dtb clk);   fdtput -t u "$d" "$IPA" clocks "$(fdtget -t u "$DTB" "$IPA" clocks | cut -d' ' -f1)" 69; want "the active-only clock" "clock is not" "$d"
d=$(case_dtb ld);    fdtput -t s "$d" "$IPA" qcom,gsi-loader modem;                        want "the modem loads" "gsi-loader" "$d"
d=$(case_dtb fw);    fdtput -t s "$d" "$IPA" firmware-name qcom/msm8998/oneplus/ipa_fws.mbn; want "the OnePlus firmware name" "firmware-name" "$d"
d=$(case_dtb mr);    fdtput -d "$d" "$IPA" memory-region;                                  want "no memory-region" "memory-region" "$d"
d=$(case_dtb sn);    fdtput -t s "$d" "$IPA" qcom,smem-state-names ipa-clock-enabled ipa-clock-enabled-valid; want "state names swapped" "smem-state-names" "$d"
d=$(case_dtb en);    fdtput -t s "$d" "$SO" qcom,entry-name master-kernel;                 want "an out entry of another name" "qcom,entry-name" "$d"
d=$(case_dtb ic);    fdtput -t u "$d" "$IPA" interconnects 1 1 1 1;                        want "interconnects to nothing" "interconnects" "$d"
d=$(case_dtb nc);    fdtput -d "$d" "$SI" interrupt-controller;                            want "an in entry that is no interrupt controller" "interrupt-controller" "$d"

ph() { fdtget -t u "$DTB" "$(fdtget "$DTB" /__symbols__ "$1")" phandle; }
GI=$(ph intc); PI=$(ph ipa_smp2p_in)
irq() { fdtput -t u "$1" "$IPA" interrupts-extended "$GI" 0 "$2" "$3" "$GI" 0 "$4" "$5" "$PI" 0 1 "$PI" 1 1; }
d=$(case_dtb irqn);  irq "$d" 332 1 432 4;                                                 want "SPI 332, one defect" "ipa's interrupts" "$d"
d=$(case_dtb irqf);  irq "$d" 333 4 432 4;                                                 want "ipa irq level, one defect" "ipa's interrupts" "$d"
d=$(case_dtb irqg);  irq "$d" 333 1 432 1;                                                 want "gsi irq edge, one defect" "ipa's interrupts" "$d"
d=$(case_dtb rn);    fdtput -t s "$d" "$IPA" reg-names ipa-base ipa-shared gsi;           want "reg-names of wahoo" "reg-names" "$d"
d=$(case_dtb in);    fdtput -t s "$d" "$IPA" interrupt-names ipa gsi ipa-setup-ready ipa-clock-query; want "interrupt names swapped" "interrupt-names" "$d"
d=$(case_dtb cn);    fdtput -t s "$d" "$IPA" clock-names core_clk;                         want "clock name of wahoo" "clock-names" "$d"
d=$(case_dtb ss);    fdtput -t u "$d" "$IPA" qcom,smem-states "$(ph ipa_smp2p_out)" 1 "$(ph ipa_smp2p_out)" 0; want "state bits swapped" "qcom,smem-states" "$d"
d=$(case_dtb sc);    fdtput -t u "$d" "$SO" '#qcom,smem-state-cells' 2;                    want "two state cells" "state cell" "$d"
d=$(case_dtb mr0);   fdtput -t u "$d" "$IPA" memory-region "$(ph mpss_mem)";               want "memory-region on mpss_mem" "memory-region" "$d"
exit $bad
