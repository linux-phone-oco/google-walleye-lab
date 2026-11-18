#!/bin/sh
# walleye-pmic-dmesg-check, run on the lab host after mainline-shell-read.sh (item walleye-pmic-spmi): does the SPMI bus of our mainline kernel
# register the PMICs? Reads the files the reader saved under DIR (dmesg.txt, spmi-devices.txt) and checks:
#   1. no spmi "does not have 'reg' property" line for a node of OUR dtb. The two nodes the stock dtbo overlay adds under the bus
#      (qcom,pm8998@0 and qcom,pmi8998@3, downstream style, no reg) are named and counted but are not ours: no board file can remove them.
#   2. /sys/bus/spmi/devices registers the PMIC devices 0-00 and 0-01 (pm8998), 0-02 and 0-03 (pmi8998), and no dmesg line says a probe of one of them failed (registered and probed, not that a child works).
#   3. the dmesg is not shorter than HALF of BASELINE_LINES (default 336, the first map; a crude "did it still boot far").
# Prints `pass|fail|skip walleye-pmic-dmesg-check host DETAIL`; exit 0 only on pass. Reads only. Usage: pmic-dmesg-check.sh DIR
D=${1:?usage: pmic-dmesg-check.sh DIR (the output directory of the reader)}
say() { echo "$1 walleye-pmic-dmesg-check host $2"; [ "$1" = pass ]; exit $?; }
[ -s "$D/dmesg.txt" ] || say skip "no dmesg in $D: not known yet"
[ -f "$D/spmi-devices.txt" ] || say skip "no spmi-devices.txt in $D (the reader of before this item): not known yet"
reg=$(grep -c "does not have 'reg' property" "$D/dmesg.txt")
mine=$(grep "does not have 'reg' property" "$D/dmesg.txt" | grep -vcE 'spmi@800f000/qcom,pm8998@0 |spmi@800f000/qcom,pmi8998@3 ')
[ "$mine" -eq 0 ] || say fail "$mine reg-less spmi node(s) of ours in the dmesg"
have=$(tr '\n' ' ' < "$D/spmi-devices.txt")
for d in 0-00 0-01 0-02 0-03; do grep -qx "$d" "$D/spmi-devices.txt" || say fail "spmi device $d missing (found: $have)"; done
grep -qE "probe of 0-0[0-3] failed" "$D/dmesg.txt" && say fail "a PMIC device probe failed: $(grep -E "probe of 0-0[0-3] failed" "$D/dmesg.txt" | head -n 1)"
n=$(wc -l < "$D/dmesg.txt"); b=${BASELINE_LINES:-336}
[ $((n*2)) -ge "$b" ] || say fail "dmesg has $n lines, under half of the baseline $b"
say pass "spmi devices 0-00 0-01 0-02 0-03 present; $reg reg-less line(s), all the overlay's own; dmesg $n lines"
