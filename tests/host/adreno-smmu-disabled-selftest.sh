#!/bin/sh
# Proves tests/host/adreno-smmu-disabled.sh on dtbs made here: no status fails, "okay" fails, "disabled" passes, no symbol fails, no file skips.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { got=$(DTB=$2 sh "$HERE/adreno-smmu-disabled.sh" | cut -d' ' -f1 || true); if [ "$got" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $3"; total=$((total+1)); fi; }
cat > "$T/a.dts" <<'D'
/dts-v1/;
/ { soc { smmu: iommu@5040000 { reg = <0 0x5040000 0 0x1000>; }; }; };
D
dtc -@ -I dts -O dtb -o "$T/none.dtb" "$T/a.dts" 2>/dev/null
cp "$T/none.dtb" "$T/okay.dtb"; fdtput -t s "$T/okay.dtb" /soc/iommu@5040000 status okay
cp "$T/none.dtb" "$T/off.dtb"; fdtput -t s "$T/off.dtb" /soc/iommu@5040000 status disabled
sed 's/smmu:/adreno_smmu:/' "$T/a.dts" > "$T/b.dts"; dtc -@ -I dts -O dtb -o "$T/sym-none.dtb" "$T/b.dts" 2>/dev/null
cp "$T/sym-none.dtb" "$T/sym-off.dtb"; fdtput -t s "$T/sym-off.dtb" /soc/iommu@5040000 status disabled
want "no symbol fails" "$T/none.dtb" fail
want "symbol, no status fails" "$T/sym-none.dtb" fail
want "symbol, disabled passes" "$T/sym-off.dtb" pass
want "no file skips" "$T/missing.dtb" skip
[ $total -eq 0 ] || { echo "adreno-smmu-disabled-selftest: $total failed" >&2; exit 1; }
echo "pass adreno-smmu-disabled-selftest"
