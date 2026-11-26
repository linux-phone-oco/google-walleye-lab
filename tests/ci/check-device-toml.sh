#!/bin/sh
# SPDX-License-Identifier: MIT OR Apache-2.0
# Proof of bin/check-device-toml.sh against a stand-in release (file://): the first line pins the release that reads the
# file (a [kernels] device.toml pinned to an older reader is refused naming key and release, a missing pin line, a
# pin to a release not attached and a wrong sha256 are refused); the newest release reads it too (a newest that refuses
# is named), a newest whose asset is not attached yet is skipped for the one before.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
w=$(mktemp -d); trap 'rm -rf "$w"' EXIT
asset=lpoco-lab-x86_64-unknown-linux-gnu
release() { # VERSION: a stand-in lpoco-lab that reads [kernels] only from 0.32.0 on
  d="$w/rel/lpoco-lab-v$1"; mkdir -p "$d"
  cat >"$d/$asset" <<STUB
#!/bin/sh
f=\$4
if [ "$1" = 0.24.0 ] && grep -q '^\[kernels\]' "\$f"; then echo "\$f is not read by lpoco-lab $1: unknown field kernels" >&2; exit 1; fi
echo "\$f: read by lpoco-lab $1"
STUB
  (cd "$d" && sha256sum "$asset" >"$asset.sha256")
}
release 0.24.0; release 0.32.1
printf '[device]\ncodename = "x"\n[kernels]\ntest = ["a"]\n' >"$w/body.toml"
# a device.toml pins the release it is read by on its first line; these cases pin the newest tag whose asset is attached
run() {
  pin=$(printf '%s\n' "$1" | grep '^lpoco-lab-v' | while read -r t; do [ -d "$w/rel/$t" ] && { echo "$t"; break; }; done)
  { printf '#:schema https://raw.githubusercontent.com/linux-phone-oco/lab-tools/%s/manifest/device.schema.json\n' "${pin:-lpoco-lab-v0.99.0}"
    cat "$w/body.toml"; } >"$w/device.toml"
  LPOCO_TAGS="$1" LPOCO_RELEASE_BASE="file://$w/rel" DEVICE_TOML="$w/device.toml" sh "$here/bin/check-device-toml.sh" 2>&1
}

out=$(run "lpoco-lab-v0.24.0") && { echo "FAIL: 0.24.0 passed a [kernels] device.toml"; exit 1; }
echo "$out" | grep -q kernels && echo "$out" | grep -q 0.24.0 || { echo "FAIL: not naming key and release: $out"; exit 1; }
out=$(run "lpoco-v0.99.0
lpoco-lab-v0.32.1
lpoco-lab-v0.24.0") || { echo "FAIL: newest lpoco-lab (0.32.1) refused: $out"; exit 1; }
echo "$out" | grep -q "against lpoco-lab-v0.32.1" || { echo "FAIL: not the newest: $out"; exit 1; }
out=$(run "lpoco-lab-v0.99.0
lpoco-lab-v0.32.1") || { echo "FAIL: an unattached newest was not skipped: $out"; exit 1; }
echo "$out" | grep -q "against lpoco-lab-v0.32.1" || { echo "FAIL: skip not to 0.32.1: $out"; exit 1; }
out=$(run "lpoco-lab-v0.99.0") && { echo "FAIL: no asset at all passed"; exit 1; }
echo "tampered" >>"$w/rel/lpoco-lab-v0.32.1/$asset"
out=$(run "lpoco-lab-v0.32.1") && { echo "FAIL: a wrong sha256 passed"; exit 1; }
echo "$out" | grep -q sha256 || { echo "FAIL: wrong sha not said: $out"; exit 1; }
# the pin: the first line names the schema of one lpoco-lab release (decision 2026-11-18-device-manifest-schema-from-rust),
# and the file is read by that release - the newest (0.32.1) reads [kernels], the pinned 0.24.0 does not
url=https://raw.githubusercontent.com/linux-phone-oco/lab-tools
pinned() { # TAG: a device.toml whose first line pins TAG
  printf '#:schema %s/%s/manifest/device.schema.json\n[device]\ncodename = "x"\n[kernels]\ntest = ["a"]\n' "$url" "$1" >"$w/pinned.toml"
}
tags="lpoco-lab-v0.32.1
lpoco-lab-v0.24.0"
run_pinned() { LPOCO_TAGS="$tags" LPOCO_RELEASE_BASE="file://$w/rel" DEVICE_TOML="$w/pinned.toml" sh "$here/bin/check-device-toml.sh" 2>&1; }
# a restored asset: the tampered one above is replaced
release 0.32.1
pinned lpoco-lab-v0.24.0
out=$(run_pinned) && { echo "FAIL: a file pinned to 0.24.0 passed though the newest reads it: $out"; exit 1; }
echo "$out" | grep -q "0.24.0 (pinned)" && echo "$out" | grep -q kernels || { echo "FAIL: pinned release and key not named: $out"; exit 1; }
pinned lpoco-lab-v0.32.1
out=$(run_pinned) || { echo "FAIL: a file pinned to 0.32.1 refused: $out"; exit 1; }
echo "$out" | grep -q "against lpoco-lab-v0.32.1 (pinned)" || { echo "FAIL: the pin is not the release used: $out"; exit 1; }
pinned lpoco-lab-v0.77.0
out=$(run_pinned) && { echo "FAIL: a pin to a release that does not exist passed"; exit 1; }
echo "$out" | grep -q 0.77.0 || { echo "FAIL: missing pinned release not named: $out"; exit 1; }
cp "$w/body.toml" "$w/pinned-none.toml"
out=$(LPOCO_TAGS="$tags" LPOCO_RELEASE_BASE="file://$w/rel" DEVICE_TOML="$w/pinned-none.toml" sh "$here/bin/check-device-toml.sh" 2>&1) &&
  { echo "FAIL: a device.toml with no #:schema line passed"; exit 1; }
echo "$out" | grep -q '#:schema' || { echo "FAIL: the missing #:schema line is not named: $out"; exit 1; }
# the newest is checked too: a stand-in 0.50.0 refuses [future]; a file pinned to 0.32.1 (which reads it) is still refused
d="$w/rel/lpoco-lab-v0.50.0"; mkdir -p "$d"
cat >"$d/$asset" <<'STUB'
#!/bin/sh
if grep -q '^\[future\]' "$4"; then echo "$4 is not read by lpoco-lab 0.50.0: unknown field future" >&2; exit 1; fi
STUB
(cd "$d" && sha256sum "$asset" >"$asset.sha256")
{ printf '#:schema %s/lpoco-lab-v0.32.1/manifest/device.schema.json\n[device]\ncodename = "x"\n[future]\n' "$url"; } >"$w/future.toml"
out=$(LPOCO_TAGS="lpoco-lab-v0.50.0
lpoco-lab-v0.32.1" LPOCO_RELEASE_BASE="file://$w/rel" DEVICE_TOML="$w/future.toml" sh "$here/bin/check-device-toml.sh" 2>&1) &&
  { echo "FAIL: a file the newest release refuses passed: $out"; exit 1; }
echo "$out" | grep -q "0.50.0 (newest)" || { echo "FAIL: the newest release's refusal not named: $out"; exit 1; }
echo "check-device-toml: 13 cases pass"
