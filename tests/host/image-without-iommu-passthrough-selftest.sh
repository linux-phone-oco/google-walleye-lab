#!/bin/sh
# Proves tests/host/image-without-iommu-passthrough.sh with a stand-in `lpoco-bringup image ensure --dry-run` (the line of the real one, as image-header-equals-recipe-selftest.sh) and
# a stand-in unpack_bootimg that prints the text the real one printed for b-r5 (iommu.passthrough=1, the control) and for b-r5-smmu (tests/fixtures/image-text/, captured 2026-11-28).
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
FIX=$HERE/../fixtures/image-text
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME EXPECTED_WORD FIXTURE
  printf '#!/bin/sh\necho "x: in the store (%s/img)"\n' "$T" > "$T/bringup"
  printf '#!/bin/sh\ncat %s\n' "$FIX/$3" > "$T/unpack_bootimg"
  chmod +x "$T/bringup" "$T/unpack_bootimg"; : > "$T/img"; : > "$T/device.toml"
  got=$(PATH="$T:$PATH" LPOCO_BRINGUP="$T/bringup" sh "$HERE/image-without-iommu-passthrough.sh" x "$T/device.toml" | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
want "b-r5 (iommu.passthrough=1 on its line) fails" fail b-r5-unpack-info.txt
want "b-r5-smmu (the board's own line) passes" pass b-r5-smmu-unpack-info.txt
[ $total -eq 0 ] || { echo "image-without-iommu-passthrough-selftest: $total failed" >&2; exit 1; }
echo "pass image-without-iommu-passthrough-selftest"
