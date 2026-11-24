#!/bin/sh
# walleye-image-recipe-z21, run on the lab host: does an image of device.toml have a recipe that `lpoco bringup image ensure` can follow?
# (decision 2026-11-23-image-recipes-on-nix-fixed-output-paths; lab-tools README "lpoco bringup image ensure").
# Usage: image-recipe.sh NAME [DEVICE_TOML]. Asks `lpoco-bringup image ensure NAME --dry-run`, which reads the store, the cache and the seed and changes none;
# its line (captured text of lpoco-bringup 0.47.1, research walleye-image-recipe-z21-rebuilt-equal) is `NAME: in the store (PATH)`,
# `NAME: from the cache (PATH)` or `NAME: would be built from its recipe (PATH)`. Anything else, or a non-zero exit (no recipe, a control or part
# the manifest does not declare), is a fail. The rebuild itself (equal to the declared sha256) is `lpoco-bringup image ensure NAME`, measured in the research row.
# The manifest must declare the recipe table: an image the store happens to hold passes ensure without one.
# Prints `pass|fail|skip walleye-image-recipe DETAIL`; exit 0 only on pass. Environment: LPOCO_BRINGUP (the program, default lpoco-bringup).
NAME=${1:-}
DEV=${2:-device.toml}
B=${LPOCO_BRINGUP:-lpoco-bringup}
say() { echo "$1 walleye-image-recipe $2"; [ "$1" = pass ]; exit $?; }
[ -n "$NAME" ] || say fail "usage: image-recipe.sh NAME [DEVICE_TOML]"
command -v "$B" >/dev/null || say skip "$B not installed: not known yet"
command -v nix-store >/dev/null || say skip "nix-store not installed: not known yet"
"$B" image ensure --help >/dev/null 2>&1 || "$B" --help 2>&1 | grep -q ensure || say skip "$B has no image ensure (lab-tools 0.45.0 or later): not known yet"
[ -r "$DEV" ] || say fail "no manifest at $DEV"
grep -qxF "[images.$NAME.recipe]" "$DEV" || say fail "$DEV declares no [images.$NAME.recipe]: ensure would serve only what the store happens to hold"
out=$("$B" image ensure "$NAME" --device "$DEV" --dry-run 2>&1) || say fail "ensure $NAME: $(echo "$out" | head -1)"
case "$out" in
  "$NAME: in the store ("*|"$NAME: from the cache ("*|"$NAME: would be built from its recipe ("*) say pass "$out" ;;
  *) say fail "ensure $NAME said: $(echo "$out" | head -1)" ;;
esac
