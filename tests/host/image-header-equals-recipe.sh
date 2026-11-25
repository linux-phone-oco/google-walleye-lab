#!/bin/sh
# walleye-image-header-equals-recipe, run on the lab host (item walleye-rw-image-header-equals-recipe): is the command line in the header of the image FILE
# the one its recipe gives? Run 4 booted 2477276882987c8e...b54f, whose header kept walleye.booted-slot, which [images.NAME.recipe].cmdline lacks:
# the sha256 was declared from a build script's output and `image ensure` found it in the store, so the recipe was never followed.
# Takes the file `lpoco-bringup image ensure NAME --dry-run` names (store, cache or seed: it builds nothing), reads its header with unpack_bootimg
# (captured text of it: tools/walleye-tries/fixtures/rw-image-2477-unpack-info.txt) and compares the words with the recipe's cmdline, in order.
# Usage: image-header-equals-recipe.sh NAME [DEVICE_TOML]. Prints `pass|fail|skip walleye-image-header-equals-recipe DETAIL`; exit 0 only on pass.
# Environment: LPOCO_BRINGUP (the program, default lpoco-bringup).
NAME=${1:-}
DEV=${2:-device.toml}
B=${LPOCO_BRINGUP:-lpoco-bringup}
say() { echo "$1 walleye-image-header-equals-recipe $2"; [ "$1" = pass ]; exit $?; }
[ -n "$NAME" ] || say fail "usage: image-header-equals-recipe.sh NAME [DEVICE_TOML]"
command -v "$B" >/dev/null || say skip "$B not installed: not known yet"
command -v unpack_bootimg >/dev/null || say skip "unpack_bootimg not installed: not known yet"
[ -r "$DEV" ] || say fail "no manifest at $DEV"
recipe=$(awk -v t="[images.$NAME.recipe]" '$0==t{on=1;next} /^\[/{on=0} on && /^cmdline *= *\047/{sub(/^cmdline *= *\047/,"");sub(/\047.*$/,"");print;exit} on && /^cmdline *=/{sub(/^cmdline *= *"/,"");sub(/".*$/,"");print;exit}' "$DEV")
[ -n "$recipe" ] || say fail "$DEV has no cmdline in [images.$NAME.recipe]"
out=$("$B" image ensure "$NAME" --device "$DEV" --dry-run 2>&1) || say fail "ensure $NAME: $(echo "$out" | head -1)"
img=$(echo "$out" | sed -n 's/.*(\(\/[^)]*\))$/\1/p' | tail -n 1)
[ -r "$img" ] || say skip "ensure names no image to read ($(echo "$out" | head -1)): not known yet"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
unpack_bootimg --boot_img "$img" --out "$T" > "$T/info" 2>&1 || say fail "unpack_bootimg could not read $img"
header=$(sed -n 's/^command line args: *//p; s/^additional command line args: *//p' "$T/info" | tr '\n' ' ')
[ "$(echo $header)" = "$(echo $recipe)" ] || say fail "$NAME's file $img has the header command line [$header], its recipe says [$recipe]"
say pass "$NAME: the header of $img is its recipe's command line"
