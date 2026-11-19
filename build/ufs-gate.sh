#!/bin/sh
# The UFS-off gate of every build script that makes a boot image (decision 2026-11-19-walleye-ufs-off-by-a-gate): runs tests/host/ufs-off-image.sh on each image
# given and exits 1 unless every one passes - a skip is not a pass, an image with UFS on is NOT TO BOOT and its build stops here.
#   ufs-gate.sh IMAGE...        (environment: DTBO, as the test)
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
[ $# -gt 0 ] || { echo "usage: ufs-gate.sh IMAGE..." >&2; exit 2; }
rc=0
for img in "$@"; do
  r=$(sh "$HERE/../tests/host/ufs-off-image.sh" "$img" 2>&1 | head -n1)
  case $r in
    pass*) echo "ok   UFS is off in $(basename "$img")" ;;
    *) echo "FAIL UFS gate on $(basename "$img"): $r" >&2; echo "ufs-gate: $img is NOT TO BOOT (UFS on, or not shown off): the image is removed" >&2; rc=1; rm -f "$img" ;;
  esac
done
exit $rc
