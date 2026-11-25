#!/bin/sh
# Proves tests/host/charger-check.sh on text our phone printed (tests/fixtures/phone-text/: the ufs-root kernel's dmesg and power-supply read, Android b's power-supply read), never touching the phone:
# the probed charger with the cable in passes, the same reading twice and a phone with no *-charger supply fail, no files skip. Faults and cable-out readings no phone has printed are in charger-typed-selftest.sh.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
F="$HERE/../fixtures/phone-text"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME EXPECTED_WORD DIR [DIR2]
  got=$(sh "$HERE/charger-check.sh" "$3" ${4:+"$4"} | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
mk() { # NAME SUPPLY_CAP - a read directory of two captured files (the cuts of a .cap: its '#' provenance lines dropped)
  mkdir -p "$T/$1"; cp "$F/dmesg-ufs-root-20261125-043720.cap" "$T/$1/dmesg.txt"; grep -v '^#' "$F/$2" > "$T/$1/power-supply-facts.txt"; }
mk ufs power-supply-facts-ufs-root-20261125-043720.cap;  want "the ufs-root read (pmi8998-charger, type USB, online 1, clean dmesg) passes" pass "$T/ufs"
want "the same online twice fails" fail "$T/ufs" "$T/ufs"
mk android power-supply-facts-android-b-2026-11-25.cap;  want "Android's supplies (battery, bms, usb: no *-charger) fail" fail "$T/android"
mkdir "$T/empty"; want "no files skip" skip "$T/empty"
[ $total -eq 0 ] || { echo "charger-selftest: $total failed" >&2; exit 1; }
echo "pass charger-selftest"
