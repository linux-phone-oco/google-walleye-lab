#!/bin/sh
# Proves tests/host/image-header-equals-recipe.sh with stand-in programs (no store, no phone): the stand-in `lpoco-bringup image ensure --dry-run`
# prints the line of the real one (captured text of lpoco-bringup 0.48.4: research walleye-image-recipe-z21-rebuilt-equal), the stand-in unpack_bootimg
# prints the lines of the real one (tools/walleye-tries/fixtures/rw-image-2477-unpack-info.txt, the text 2477...b54f printed).
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME EXPECTED_WORD CMDLINE_IN_HEADER [ADDITIONAL]
  printf '#!/bin/sh\necho "x: in the store (%s/img)"\n' "$T" > "$T/bringup"
  { echo "command line args: $3"; echo "additional command line args: ${4:-}"; } > "$T/info"
  printf '#!/bin/sh\ncat %s\n' "$T/info" > "$T/unpack_bootimg"
  chmod +x "$T/bringup" "$T/unpack_bootimg"; : > "$T/img"
  printf '[images.x.recipe]\ncmdline = "a b walleye.rw"\n' > "$T/device.toml"
  got=$(PATH="$T:$PATH" LPOCO_BRINGUP="$T/bringup" sh "$HERE/image-header-equals-recipe.sh" x "$T/device.toml" | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
want "a header equal to the recipe passes" pass "a b walleye.rw"
want "a header with a word more fails (2477's walleye.booted-slot)" fail "a b walleye.booted-slot walleye.rw"
want "a header with a word less fails" fail "a walleye.rw"
want "a word in the additional field fails" fail "a b walleye.rw" "walleye.booted-slot"
[ $total -eq 0 ] || { echo "image-header-equals-recipe-selftest: $total failed" >&2; exit 1; }
echo "pass image-header-equals-recipe-selftest"
