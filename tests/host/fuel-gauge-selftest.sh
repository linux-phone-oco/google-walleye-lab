#!/bin/sh
# Proves tests/host/fuel-gauge-check.sh against made-up reader output, never touching the phone.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME EXPECTED_WORD DIR FASTBOOT_MV
  got=$(sh "$HERE/fuel-gauge-check.sh" "$3" "$4" | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
mk() { # NAME CAPACITY VOLTAGE_UV [TYPE]
  mkdir -p "$T/$1"; printf '[    0.1] Booting Linux\n[    2.0] pmi8998-fg 2-0040.fuel-gauge: registered\n' > "$T/$1/dmesg.txt"
  printf '== pmi8998-charger\ntype=USB\nonline=1\n== qcom-battery\ntype=%s\ncapacity=%s\nvoltage_now=%s\n' "${4:-Battery}" "$2" "$3" > "$T/$1/power-supply-facts.txt"; }
mk good 62 3950000
want "capacity and voltage within 100 mV pass" pass "$T/good" 3900
want "voltage 150 mV off fails" fail "$T/good" 3800
want "a non-numeric fastboot reading skips" skip "$T/good" none
mk wrongtype 62 3950000 Mains; want "a type other than Battery fails" fail "$T/wrongtype" 3950
mk big 101 3950000; want "capacity above 100 fails" fail "$T/big" 3950
mk nocap "cat: read error" 3950000; want "a supply present with an unreadable capacity fails" fail "$T/nocap" 3950
mk rt 62 3950000; echo "[    2.1] pmi8998-fg 2-0040.fuel-gauge: Failed to read temperature: -5" >> "$T/rt/dmesg.txt"; want "a runtime read fault fails" fail "$T/rt" 3950
mk pf 62 3950000; echo "[    2.1] qcom-spmi 0-02: probe with driver pmi8998-fg failed with error -22" >> "$T/pf/dmesg.txt"; want "a probe failure line fails" fail "$T/pf" 3950
mk fault 62 3950000; echo '[    2.1] pmi8998-fg 2-0040.fuel-gauge: Failed to clear IMA exception: -110' >> "$T/fault/dmesg.txt"; want "a probe fault fails" fail "$T/fault" 3950
mk nosupply 62 3950000; printf '== pmi8998-charger\ntype=USB\n' > "$T/nosupply/power-supply-facts.txt"; want "no qcom-battery supply fails" fail "$T/nosupply" 3950
mkdir "$T/empty"; want "no files skip" skip "$T/empty" 3950
[ $total -eq 0 ] || { echo "fuel-gauge-selftest: $total failed" >&2; exit 1; }
echo "pass fuel-gauge-selftest"
