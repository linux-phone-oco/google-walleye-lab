#!/bin/sh
# Proves the kernel test of tests/host/image-ramoops-stock-layout.sh (--kernel-part): a gzip of the lines the v7.3 build printed (tests/fixtures/kernel-text/ramoops-params-v7.3.txt, captured
# 2026-11-28 from the decompressed Image of int/walleye/display-smmu-id/v7.3/1) passes, the same lines without ramoops.console_size fail, and a stream with the dtb after its end (as an unpacked
# kernel part has) still passes.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
FIX=$HERE/../fixtures/kernel-text/ramoops-params-v7.3.txt
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
gzip -c "$FIX" > "$T/with.gz"
grep -v 'ramoops\.console_size' "$FIX" | gzip -c > "$T/without.gz"
{ cat "$T/with.gz"; printf '\320\015\376\355trailing'; } > "$T/with-dtb.gz"
want() { # NAME EXPECTED FILE
  got=$(sh "$HERE/image-ramoops-stock-layout.sh" --kernel-part "$3" | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
want "a kernel naming ramoops.console_size passes" pass "$T/with.gz"
want "the same lines without it fail" fail "$T/without.gz"
want "a stream with the dtb after its end still passes" pass "$T/with-dtb.gz"
[ $total -eq 0 ] || { echo "image-ramoops-stock-layout-selftest: $total failed" >&2; exit 1; }
echo "pass image-ramoops-stock-layout-selftest"
