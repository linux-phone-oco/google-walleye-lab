#!/bin/sh
# walleye-pmic-dmesg-selftest, host only: pmic-dmesg-check.sh on text our phone printed (tests/fixtures/phone-text/pmic-dmesg-z7-try1-*.cap, copies of the z7
# try read) and cuts of it: the read passes, each missing spmi device, a short dmesg and a missing file fail or skip. Typed faults are in pmic-dmesg-typed-selftest.sh.
HERE=$(cd "$(dirname "$0")" && pwd)
F="$HERE/../fixtures/phone-text"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT; fail=0; pass=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
mk() { mkdir "$T/$1"; cp "$F/pmic-dmesg-z7-try1-dmesg.cap" "$T/$1/dmesg.txt"; cp "$F/pmic-dmesg-z7-try1-spmi-devices.cap" "$T/$1/spmi-devices.txt"; }
r() { sh "$HERE/pmic-dmesg-check.sh" "$T/$1" | cut -d' ' -f1; }
mk good; ck "the z7 read (bound, only the overlay's two reg-less nodes): pass" "$(r good)" pass
mk none; grep -v "does not have 'reg' property" "$T/none/dmesg.txt" > "$T/none/d"; mv "$T/none/d" "$T/none/dmesg.txt"; ck "the z7 read without its reg-less lines: pass" "$(r none)" pass
for d in 0-00 0-01 0-02 0-03; do mk "no$d"; grep -vx "$d" "$T/no$d/spmi-devices.txt" > "$T/no$d/s"; mv "$T/no$d/s" "$T/no$d/spmi-devices.txt"; ck "$d cut from the z7 list: fail" "$(r "no$d")" fail; done
mk short; head -n 100 "$T/short/dmesg.txt" > "$T/short/d"; mv "$T/short/d" "$T/short/dmesg.txt"; ck "the z7 dmesg cut to 100 lines (under half the map): fail" "$(r short)" fail
mk nolist; rm "$T/nolist/spmi-devices.txt"; ck "no spmi-devices.txt: skip" "$(r nolist)" skip
mkdir "$T/empty"; ck "nothing read: skip" "$(r empty)" skip
echo "$([ $fail = 0 ] && echo pass || echo fail) walleye-pmic-dmesg-selftest host $pass ok, $fail failed"; [ $fail = 0 ]
