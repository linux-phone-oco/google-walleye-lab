#!/bin/sh
# walleye-pmic-dmesg-typed-selftest, host only: pmic-dmesg-check.sh against typed faults no phone has printed (a reg-less node of ours, another address,
# a PMIC probe failure) appended to a typed dmesg. Listed not yet sourced in SOURCES.toml; the captured cases are in pmic-dmesg-selftest.sh.
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT; fail=0; pass=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
mk() { mkdir "$T/$1"; i=0; while [ $i -lt 300 ]; do echo "[ $i.0] line"; i=$((i+1)); done > "$T/$1/dmesg.txt"; printf '0-00\n0-01\n0-02\n0-03\n' > "$T/$1/spmi-devices.txt"; }
ov() { echo "spmi spmi-0: node /soc@0/spmi@800f000/qcom,pm8998@0 err (-22) does not have 'reg' property" >> "$T/$1/dmesg.txt"; echo "spmi spmi-0: node /soc@0/spmi@800f000/qcom,pmi8998@3 err (-22) does not have 'reg' property" >> "$T/$1/dmesg.txt"; }
r() { sh "$HERE/pmic-dmesg-check.sh" "$T/$1" | cut -d' ' -f1; }
mk mine; ov mine; echo "spmi spmi-0: node /soc@0/spmi@800f000/pmic@0 err (-22) does not have 'reg' property" >> "$T/mine/dmesg.txt"; ck "a reg-less node of ours: fail" "$(r mine)" fail
mk other; ov other; echo "spmi spmi-0: node /soc@0/spmi@800f000/qcom,pmi8998@2 err (-22) does not have 'reg' property" >> "$T/other/dmesg.txt"; ck "a reg-less node named like the overlay's but another address: fail" "$(r other)" fail
mk pf; ov pf; echo "[ 5.0] qcom,spmi-pmic: probe of 0-02 failed with error -22" >> "$T/pf/dmesg.txt"; ck "a PMIC probe failure: fail" "$(r pf)" fail
echo "$([ $fail = 0 ] && echo pass || echo fail) walleye-pmic-dmesg-typed-selftest host $pass ok, $fail failed"; [ $fail = 0 ]
