#!/bin/sh
# Proves tests/host/dmesg-clean-check.sh against made-up reader output, never touching the phone: every fault alone fails, the clean case passes, missing files skip.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME EXPECTED_WORD DIR
  got=$(sh "$HERE/dmesg-clean-check.sh" "$3" | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
mk() { mkdir -p "$T/$1"; printf '[    0.1] Booting Linux\n[    0.5] arm-smmu 5040000.iommu: probing hardware configuration...\n' > "$T/$1/dmesg.txt"
  printf "cat /sys/kernel/debug/devices_deferred || mount x && cat y; echo wmsr-1\nwmsr-1-END\n" > "$T/$1/devices-deferred.txt"; }
mk clean;    want "clean dmesg and empty devices_deferred pass" pass "$T/clean"
mk smmu;     echo '[   11.2] arm-smmu 5040000.iommu: probe with driver arm-smmu failed with error -110' >> "$T/smmu/dmesg.txt"; want "the smmu -110 fails" fail "$T/smmu"
mk phandle;  echo '[    0.08] OF: /soc@0/i2c@c179000: could not find phandle -1934992561' >> "$T/phandle/dmesg.txt"; want "a phandle line fails" fail "$T/phandle"
mk defer;    printf '5040000.iommu\tplatform: reason unknown\n' >> "$T/defer/devices-deferred.txt"; want "a deferred smmu fails" fail "$T/defer"
mk known;    printf 'c16f000.serial\tplatform: wait for supplier\nc179000.i2c\tplatform: x\n' >> "$T/known/devices-deferred.txt"; want "only known overlay nodes deferred passes" pass "$T/known"
mk supp;     printf '5040000.iommu\tplatform: wait for supplier c179000.i2c\n' >> "$T/supp/devices-deferred.txt"; want "a non-overlay device naming an overlay supplier fails" fail "$T/supp"
mk gpux;    echo '[    2.1] gpu_gx: status stuck at off' >> "$T/gpux/dmesg.txt"; want "a gpu_gx stuck WARN fails" fail "$T/gpux"
mk nodebugfs; printf "cat: can't open '/sys/kernel/debug/devices_deferred': No such file or directory\ncat: can't open '/sys/kernel/debug/devices_deferred': No such file or directory\n" > "$T/nodebugfs/devices-deferred.txt"; want "debugfs never mounted skips" skip "$T/nodebugfs"
mkdir "$T/empty"; want "no files skip" skip "$T/empty"
[ $total -eq 0 ] || { echo "dmesg-clean-selftest: $total failed" >&2; exit 1; }
echo "pass dmesg-clean-selftest"
