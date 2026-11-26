#!/bin/sh
# SPDX-License-Identifier: MIT OR Apache-2.0
# Checks device.toml twice with lpoco-lab (decisions 2026-11-18-device-manifest-schema-from-rust and
# 2026-11-19-host-follows-releases (c)), so a file the host cannot read cannot land:
#   1. the release its first line pins: `#:schema .../lab-tools/lpoco-lab-vX.Y.Z/manifest/device.schema.json` names
#      the schema of one release - editors complete from it, and here that release's lpoco-lab reads the file;
#   2. the newest lpoco-lab release (the host follows each release within minutes; `bin/attended check --host`
#      covers the lag at run time, so lab.toml, which is private, is not read).
# Each downloads the release's lpoco-lab asset, checks its sha256, and runs `lpoco-lab manifest check --device FILE`.
#   LPOCO_TAGS          the lpoco-lab release tags, newest first, one per line (else `gh release list`, needs GH_TOKEN)
#   LPOCO_RELEASE_BASE  where releases are (default: github.com/linux-phone-oco/lab-tools/releases/download)
#   DEVICE_TOML         the file to check (default: device.toml)
set -eu
device="${DEVICE_TOML:-device.toml}"
base="${LPOCO_RELEASE_BASE:-https://github.com/linux-phone-oco/lab-tools/releases/download}"
asset="lpoco-lab-x86_64-unknown-linux-gnu"
work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
fail() { echo "check-device-toml: $*" >&2; exit 1; }

fetch() { # TAG: that release's lpoco-lab into $work/TAG, its sha256 checked; fails quietly when the asset is not there
  mkdir -p "$work/$1"
  curl -fsSL -o "$work/$1/lpoco-lab" "$base/$1/$asset" && curl -fsSL -o "$work/$1/sha256" "$base/$1/$asset.sha256" || return 1
  want=$(cut -d' ' -f1 "$work/$1/sha256")
  have=$(sha256sum "$work/$1/lpoco-lab" | cut -d' ' -f1)
  [ -n "$want" ] && [ "$want" = "$have" ] || fail "the $1 download does not match its .sha256"
  chmod +x "$work/$1/lpoco-lab"
}
check() { # TAG LABEL: the file read by that release's lpoco-lab
  echo "check-device-toml: $device against $1 ($2)"
  "$work/$1/lpoco-lab" manifest check --device "$device" || fail "$1 does not read $device (above)"
}

# the pin: the release in the first line's schema URL
first=$(head -n 1 "$device")
case "$first" in
  '#:schema '*) ;;
  *) fail "$device has no '#:schema URL' first line: the pin of the lpoco-lab release whose schema it follows" ;;
esac
pin=$(printf '%s\n' "$first" | sed -n 's|.*/\(lpoco-lab-v[0-9][0-9.]*\)/manifest/device\.schema\.json$|\1|p')
[ -n "$pin" ] || fail "the first line of $device names no lpoco-lab-vX.Y.Z release: $first"
fetch "$pin" || fail "the pinned release $pin has no $asset and .sha256 attached"
check "$pin" "pinned"

if [ -n "${LPOCO_TAGS:-}" ]; then
  printf '%s\n' "$LPOCO_TAGS" >"$work/tags"
else
  gh release list -R linux-phone-oco/lab-tools --limit 40 --json tagName,isDraft,isPrerelease \
    -q '.[] | select(.isDraft | not) | select(.isPrerelease | not) | .tagName' >"$work/tags" ||
    fail "the releases of linux-phone-oco/lab-tools could not be listed"
fi
# the newest lpoco-lab release whose asset is already attached (a release's assets arrive a minute after it)
tag=""
for t in $(grep '^lpoco-lab-v' "$work/tags" | head -5); do
  if fetch "$t"; then tag=$t; break; fi
done
[ -n "$tag" ] || fail "none of the newest lpoco-lab releases has its $asset and .sha256 attached"
check "$tag" "newest"
