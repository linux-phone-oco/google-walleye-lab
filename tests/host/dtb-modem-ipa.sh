#!/bin/sh
# walleye-dtb-modem-ipa, run on the lab host: does the modem variant's dtb carry the IPA node, every value from its source (item walleye-modem-net-port-ipa-node)?
# Source: tests/fixtures/kernel-text/msm8998-ipa-sources.txt (captured text: Google's wahoo dtsi A, the msm8998-mainline fork's ipa node B). Each value below names the block it comes from.
# Passes only when the node ipa@1e40000 (label ipa) is: compatible qcom,msm8998-ipa; okay; reg-names ipa-reg, ipa-shared, gsi with the fork's values (B), the gsi range equal wahoo's gsi-base (A);
# interrupts GIC SPI 333 (A: ipa-irq) and 432 (A: gsi-irq) then the two smp2p-in bits 0, 1 (B); iommus anoc2_smmu 0x18e0 and 0x18e2 (A: ipa_smmu_ap, ipa_smmu_uc); clock rpmcc RPM_SMD_IPA_CLK (68), name core (B);
# qcom,gsi-loader self (A: the firmware's pas-id 0xF is Trust Zone's, B2: the OnePlus 5's AP loads it), memory-region the phone's ipa/gpu block, firmware-name the phone's ipa_fws.mdt (lab-material vendor-firmware-2026-11-24);
# smem-states ipa_smp2p_out bits 0 and 1 named ipa-clock-enabled-valid, ipa-clock-enabled (B); clock numbers C, loader C2, firmware C3; and the smp2p-mpss node has both ipa entries (qcom,entry-name ipa). No interconnects: v7.3 has no msm8998 provider (research walleye-modem-net-port-how-it-stands).
# Prints one line `pass|fail|skip walleye-dtb-modem-ipa kernel DETAIL`; exit 0 only on pass. Read-only. Needs fdtget (dtc). Environment: DTB (no default: the image build names it).
say() { echo "$1 walleye-dtb-modem-ipa kernel $2"; [ "$1" = pass ]; exit $?; }
command -v fdtget >/dev/null || say skip "fdtget (dtc) not installed: not known yet"
[ -r "${DTB:-}" ] || say skip "need DTB=PATH readable: not known yet"
N=$(basename "$DTB")
sym() { fdtget "$DTB" /__symbols__ "$1" 2>/dev/null; }
ph() { fdtget -t u "$DTB" "$(sym "$1")" phandle 2>/dev/null; }
IPA=$(sym ipa) || say fail "$N: no ipa label"
[ -n "$IPA" ] || say fail "$N: no ipa label"
g() { fdtget "$@" 2>/dev/null; }
[ "$(g "$DTB" "$IPA" compatible)" = qcom,msm8998-ipa ] || say fail "$N: ipa is not qcom,msm8998-ipa"
[ "$(g "$DTB" "$IPA" status)" = okay ] || say fail "$N: ipa is not okay"
[ "$(g -t s "$DTB" "$IPA" reg-names | tr '\n' ' ')" = "ipa-reg ipa-shared gsi " ] || say fail "$N: ipa's reg-names are not ipa-reg, ipa-shared, gsi"
[ "$(g -t x "$DTB" "$IPA" reg)" = "1e40000 7000 1e47000 2000 1e04000 2c000" ] || say fail "$N: ipa's reg is not the fork's ipa-reg, ipa-shared and wahoo's gsi range"
[ "$(g -t s "$DTB" "$IPA" interrupt-names | tr '\n' ' ')" = "ipa gsi ipa-clock-query ipa-setup-ready " ] || say fail "$N: ipa's interrupt-names are not ipa, gsi, ipa-clock-query, ipa-setup-ready"
IN=$(g -t u "$DTB" "$IPA" interrupts-extended)
IV=$(echo "$IN" | tr -s ' ' ' ')
case "$IV" in "$(ph intc) 0 333 1 $(ph intc) 0 432 4 $(ph ipa_smp2p_in) 0 1 $(ph ipa_smp2p_in) 1 1") ;; *) say fail "$N: ipa's interrupts are '$IV', not SPI 333 edge, SPI 432 level, then smp2p-in bits 0 and 1 (edge)";; esac
[ "$(g -t u "$DTB" "$IPA" iommus)" = "$(ph anoc2_smmu) 6368 $(ph anoc2_smmu) 6370" ] || say fail "$N: ipa's iommus are not anoc2_smmu 0x18e0 and 0x18e2"
[ "$(g -t u "$DTB" "$IPA" clocks)" = "$(ph rpmcc) 68" ] || say fail "$N: ipa's clock is not rpmcc RPM_SMD_IPA_CLK (68)"
[ "$(g -t s "$DTB" "$IPA" clock-names)" = core ] || say fail "$N: ipa's clock-names is not core"
[ "$(g -t s "$DTB" "$IPA" qcom,gsi-loader)" = self ] || say fail "$N: ipa's qcom,gsi-loader is not self"
[ -n "$(ph ipa_gpu_mem)" ] && [ "$(g -t u "$DTB" "$IPA" memory-region)" = "$(ph ipa_gpu_mem)" ] || say fail "$N: ipa's memory-region is not the phone's ipa/gpu block (ipa_gpu_mem)"
[ "$(g -t s "$DTB" "$IPA" firmware-name)" = qcom/msm8998/google/walleye/ipa_fws.mdt ] || say fail "$N: ipa's firmware-name is not qcom/msm8998/google/walleye/ipa_fws.mdt"
[ "$(g -t s "$DTB" "$IPA" qcom,smem-state-names | tr '\n' ' ')" = "ipa-clock-enabled-valid ipa-clock-enabled " ] || say fail "$N: ipa's qcom,smem-state-names are not ipa-clock-enabled-valid, ipa-clock-enabled"
[ "$(g -t u "$DTB" "$IPA" qcom,smem-states)" = "$(ph ipa_smp2p_out) 0 $(ph ipa_smp2p_out) 1" ] || say fail "$N: ipa's qcom,smem-states are not ipa_smp2p_out bits 0 and 1"
for l in ipa_smp2p_out ipa_smp2p_in; do
  [ "$(g "$DTB" "$(sym $l)" qcom,entry-name)" = ipa ] || say fail "$N: $l's qcom,entry-name is not ipa"
  [ "$(dirname "$(sym $l)")" = /smp2p-mpss ] || say fail "$N: $l is not under smp2p-mpss"
done
[ "$(g -t u "$DTB" "$(sym ipa_smp2p_out)" '#qcom,smem-state-cells')" = 1 ] || say fail "$N: ipa_smp2p_out has not one state cell"
fdtget -p "$DTB" "$(sym ipa_smp2p_in)" | grep -qx interrupt-controller || say fail "$N: ipa_smp2p_in is no interrupt-controller"
fdtget -p "$DTB" "$IPA" | grep -qx interconnects && say fail "$N: ipa has interconnects, which v7.3 has no msm8998 provider for"
say pass "$N: ipa@1e40000 qcom,msm8998-ipa okay; the values of wahoo's dtsi and the OnePlus 5's node; the AP loads ipa_fws into the ipa/gpu block; smp2p-mpss has the two ipa entries"
