#!/bin/sh
# walleye-image-without-iommu-passthrough, run on the lab host (item walleye-display-reset-smmu-on-image; research walleye-iommu-stock-android-maps-splash-before-attach):
# does the boot image NAME leave the SMMU on - no iommu.passthrough word in its header's command line? Stock Android keeps the SMMU on; B-r5 (iommu.passthrough=1) is the control.
# Reads the file `lpoco-bringup image ensure NAME --dry-run` names (store, cache or seed: it builds nothing) with unpack_bootimg, both command-line fields.
# Usage: image-without-iommu-passthrough.sh NAME [DEVICE_TOML]. Prints `pass|fail|skip walleye-image-without-iommu-passthrough DETAIL`; exit 0 only on pass.
# Environment: LPOCO_BRINGUP (the program, default lpoco-bringup).
NAME=${1:-}
DEV=${2:-device.toml}
B=${LPOCO_BRINGUP:-lpoco-bringup}
say() { echo "$1 walleye-image-without-iommu-passthrough $2"; [ "$1" = pass ]; exit $?; }
[ -n "$NAME" ] || say fail "usage: image-without-iommu-passthrough.sh NAME [DEVICE_TOML]"
command -v "$B" >/dev/null || say skip "$B not installed: not known yet"
command -v unpack_bootimg >/dev/null || say skip "unpack_bootimg not installed: not known yet"
[ -r "$DEV" ] || say fail "no manifest at $DEV"
out=$("$B" image ensure "$NAME" --device "$DEV" --dry-run 2>&1) || say fail "ensure $NAME: $(echo "$out" | head -1)"
img=$(echo "$out" | sed -n 's/.*(\(\/[^)]*\))$/\1/p' | tail -n 1)
[ -r "$img" ] || say skip "ensure names no image to read ($(echo "$out" | head -1)): not known yet"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
unpack_bootimg --boot_img "$img" --out "$T" > "$T/info" 2>&1 || say fail "unpack_bootimg could not read $img"
header=$(sed -n 's/^command line args: *//p; s/^additional command line args: *//p' "$T/info" | tr '\n' ' ')
[ -n "$(echo $header)" ] || say fail "$img: no command line read from its header"
set -f
for w in $header; do
  case $w in iommu.passthrough*) say fail "$NAME's header command line has $w" ;; esac
done
say pass "$NAME: no iommu.passthrough in the header of $img"
