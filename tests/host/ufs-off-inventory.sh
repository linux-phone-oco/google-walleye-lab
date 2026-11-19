#!/bin/sh
# walleye-ufs-off-gate, run on the lab host: which boot images of ours have UFS on? Runs tests/host/ufs-off-image.sh over every DIR/images*/boot-*.img (DIR default ~/Projects/walleye-build)
# and prints one line per image: `off|ON|unknown IMAGE DETAIL`, then a count. An ON image is NOT TO BOOT (decision 2026-11-19-walleye-ufs-off-by-a-gate). Exit 0 whatever the answer: it reports, the gate refuses.
# Environment: DTBO (as the gate).
HERE=$(cd "$(dirname "$0")" && pwd)
D=${1:-$HOME/Projects/walleye-build}
on=0; off=0; unk=0
for f in "$D"/images*/boot-*.img; do
  [ -r "$f" ] || continue
  r=$(sh "$HERE/ufs-off-image.sh" "$f" 2>&1 | head -n1)
  case $r in
    pass*) off=$((off+1)); echo "off ${f#"$D"/} ${r#pass walleye-ufs-off }" ;;
    fail*) on=$((on+1)); echo "ON ${f#"$D"/} ${r#fail walleye-ufs-off }" ;;
    *) unk=$((unk+1)); echo "unknown ${f#"$D"/} $r" ;;
  esac
done
echo "ufs-off-inventory: $on ON (not to boot), $off off, $unk unknown"
