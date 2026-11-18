#!/bin/sh
# walleye-charger-smb2, run on the lab host after mainline-shell-read.sh: did pmi8998's charger probe and does it show the cable?
# Reads the files the reader saved under DIR (dmesg.txt, power-supply-facts.txt: per supply a `== NAME` line, then KEY=VALUE lines), and checks:
#   1. dmesg has no failed charger probe ("Failed to get battery info", "failed to register power supply", "Couldn't set vbat max", a smb/charger probe error);
#   2. a supply named *-charger is there with type=USB;
#   3. with a second DIR (the same reading with the cable in the other state) its online value differs from the first's: online follows the cable.
# The battery voltage rising while connected is read from the Android side, not here (no fuel gauge yet: the charge level stays not known yet).
# Prints `pass|fail|skip walleye-charger-smb2 host DETAIL`; exit 0 only on pass. Reads only.
# Usage: charger-check.sh DIR [DIR_OTHER_CABLE_STATE]
D=${1:?usage: charger-check.sh DIR [DIR_OTHER_CABLE_STATE]}
O=${2:-}
say() { echo "$1 walleye-charger-smb2 host $2"; [ "$1" = pass ]; exit $?; }
[ -s "$D/dmesg.txt" ] || say skip "no dmesg in $D: not known yet"
[ -s "$D/power-supply-facts.txt" ] || say skip "no power-supply-facts.txt in $D: not known yet"
val() { # DIR KEY: the value of KEY in the first *-charger block
  awk -v k="$2" '/^== /{inb=($2 ~ /-charger$/); next} inb && index($0, k "=")==1 {print substr($0, length(k)+2); exit}' "$1/power-supply-facts.txt"
}
bad=$(grep -E 'Failed to get battery info|failed to register power supply|Couldn.t set vbat max|smb2?[a-z]* .*probe .*failed|pmi8998-charger.*failed' "$D/dmesg.txt" | sed 's/^[^]]*\] //' | head -n 3 | tr '\n' ' ')
[ -z "$bad" ] || say fail "charger probe fault: $bad"
grep -q '^== .*-charger$' "$D/power-supply-facts.txt" || say fail "no *-charger power supply in $D (supplies: $(grep '^== ' "$D/power-supply-facts.txt" | cut -c4- | tr '\n' ' '))"
t=$(val "$D" type)
[ "$t" = USB ] || say fail "charger type is '$t', want USB"
on=$(val "$D" online)
[ -n "$on" ] || say skip "the charger's online value was not read: not known yet"
if [ -n "$O" ]; then
  [ -s "$O/power-supply-facts.txt" ] || say skip "no power-supply-facts.txt in $O: not known yet"
  on2=$(val "$O" online)
  [ -n "$on2" ] || say skip "no online value in $O: not known yet"
  [ "$on" != "$on2" ] || say fail "online is $on in both readings: it does not follow the cable"
  say pass "charger probed, type USB, online $on then $on2 (follows the cable)"
fi
say pass "charger probed, type USB, online $on (give a second reading with the cable changed to prove it follows)"
