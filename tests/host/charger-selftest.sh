#!/bin/sh
# Proves tests/host/charger-check.sh against made-up reader output, never touching the phone.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME EXPECTED_WORD DIR [DIR2]
  got=$(sh "$HERE/charger-check.sh" "$3" ${4:+"$4"} | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
mk() { # NAME ONLINE [TYPE]
  mkdir -p "$T/$1"; printf '[    0.1] Booting Linux\n[    2.0] qcom-smbx 2-1000.charger: registered\n' > "$T/$1/dmesg.txt"
  printf '== pmi8998-charger\ntype=%s\nonline=%s\nvoltage_now=5000000\n' "${3:-USB}" "$2" > "$T/$1/power-supply-facts.txt"; }
mk in 1;  mk out 0
want "probed charger of type USB passes (one reading)" pass "$T/in"
want "online 1 then 0 follows the cable" pass "$T/in" "$T/out"
want "the same online twice fails" fail "$T/in" "$T/in"
mk wrongtype 1 Mains; want "a type other than USB fails" fail "$T/wrongtype"
mk fault 1; echo '[    2.1] qcom-smbx 2-1000.charger: Failed to get battery info' >> "$T/fault/dmesg.txt"; want "a battery-info fault fails" fail "$T/fault"
mk nosupply 1; printf '== battery\ntype=Battery\n' > "$T/nosupply/power-supply-facts.txt"; want "no *-charger supply fails" fail "$T/nosupply"
mk noonline 1; printf '== pmi8998-charger\ntype=USB\n' > "$T/noonline/power-supply-facts.txt"; want "an unread online value skips" skip "$T/noonline"
mkdir "$T/empty"; want "no files skip" skip "$T/empty"
[ $total -eq 0 ] || { echo "charger-selftest: $total failed" >&2; exit 1; }
echo "pass charger-selftest"
