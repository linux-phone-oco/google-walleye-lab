#!/bin/sh
# walleye-battery-capacity (item walleye-fuel-gauge), run on the lab host after mainline-shell-read.sh: did the PMI8998 fuel gauge probe, and does it
# read the battery the way fastboot does? Reads the files the reader saved under DIR (dmesg.txt, power-supply-facts.txt: per supply a `== NAME` line,
# then KEY=VALUE lines) and the battery voltage in mV that tests/host/battery-ok.sh read from fastboot (or adb) just before the try, and checks:
#   1. dmesg has no failed fuel gauge probe (pmi8998_fg or qcom-battery lines saying failed, Failed to ..., IMA error);
#   2. a supply named qcom-battery is there, type Battery;
#   3. capacity is a whole number 0..100 and voltage_now (microvolts) is within TOLERANCE_MV (default 100) of the fastboot reading.
# Prints `pass|fail|skip walleye-battery-capacity host DETAIL`; exit 0 only on pass. A reading that cannot be compared is `skip` (not known yet), never pass. Reads only.
# Usage: fuel-gauge-check.sh DIR FASTBOOT_MV
D=${1:?usage: fuel-gauge-check.sh DIR FASTBOOT_MV}
FB=${2:?usage: fuel-gauge-check.sh DIR FASTBOOT_MV}
TOL=${TOLERANCE_MV:-100}
say() { echo "$1 walleye-battery-capacity host $2"; [ "$1" = pass ]; exit $?; }
case $FB in ''|*[!0-9]*) say skip "the fastboot reading '$FB' is not a number of mV: not known yet";; esac
[ -s "$D/dmesg.txt" ] || say skip "no dmesg in $D: not known yet"
[ -s "$D/power-supply-facts.txt" ] || say skip "no power-supply-facts.txt in $D: not known yet"
bad=$(grep -Ei 'pmi8998[-_]fg.*(fail|error)|qcom-battery.*(fail|error)|Failed to (read temperature|read IMA|configure interrupt|write IMA|locate the regmap|read SOC|clear IMA|read dma_status|write dma_ctl|register battery|get irq soc-delta|request soc-delta|register notifier)|IMA exception|probe with driver pmi8998.fg failed' "$D/dmesg.txt" | sed 's/^[^]]*\] //' | head -n 3 | tr '\n' ' ')
[ -z "$bad" ] || say fail "fuel gauge probe fault: $bad"
val() { awk -v k="$1" '/^== /{inb=($2 == "qcom-battery"); next} inb && index($0, k "=")==1 {print substr($0, length(k)+2); exit}' "$D/power-supply-facts.txt"; }
grep -q '^== qcom-battery$' "$D/power-supply-facts.txt" || say fail "no qcom-battery power supply in $D (supplies: $(grep '^== ' "$D/power-supply-facts.txt" | cut -c4- | tr '\n' ' '))"
t=$(val type)
[ "$t" = Battery ] || say fail "qcom-battery type is '$t', want Battery"
cap=$(val capacity); uv=$(val voltage_now)
case $cap in ''|*[!0-9]*) say fail "qcom-battery is there but capacity reads '$cap', not a number";; esac
case $uv in ''|*[!0-9]*) say fail "qcom-battery is there but voltage_now reads '$uv', not a number";; esac
[ "$cap" -le 100 ] || say fail "capacity $cap is above 100"
mv=$((uv / 1000)); d=$((mv - FB)); [ $d -ge 0 ] || d=$((-d))
[ $d -le "$TOL" ] || say fail "voltage_now $mv mV differs from the fastboot reading $FB mV by $d mV, more than $TOL"
say pass "qcom-battery probed: capacity $cap %, voltage_now $mv mV, $d mV from fastboot's $FB mV (tolerance $TOL)"
