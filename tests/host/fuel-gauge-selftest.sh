#!/bin/sh
# Proves tests/host/fuel-gauge-check.sh on text our phone printed (tests/fixtures/phone-text/: the ufs-root kernel's dmesg and power-supply read, Android b's power-supply read), never touching the phone:
# a phone with no qcom-battery supply fails, no files skip. No capture of our kernel's qcom-battery exists yet (fg1 was tried 2026-11-19 and only its numbers were recorded: not known yet as text),
# so the probed battery, its tolerance and its faults are in fuel-gauge-typed-selftest.sh.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
F="$HERE/../fixtures/phone-text"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME EXPECTED_WORD DIR FASTBOOT_MV
  got=$(sh "$HERE/fuel-gauge-check.sh" "$3" "$4" | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
mk() { # NAME SUPPLY_CAP - a read directory of two captured files (the '#' provenance lines of the .cap dropped)
  mkdir -p "$T/$1"; cp "$F/dmesg-ufs-root-20261125-043720.cap" "$T/$1/dmesg.txt"; grep -v '^#' "$F/$2" > "$T/$1/power-supply-facts.txt"; }
mk ufs power-supply-facts-ufs-root-20261125-043720.cap;  want "the ufs-root read (a charger, no qcom-battery) fails" fail "$T/ufs" 4375
mk android power-supply-facts-android-b-2026-11-25.cap; want "Android's read (its supply is named battery) fails" fail "$T/android" 4375
mkdir "$T/empty"; want "no files skip" skip "$T/empty" 4375
[ $total -eq 0 ] || { echo "fuel-gauge-selftest: $total failed" >&2; exit 1; }
echo "pass fuel-gauge-selftest"
