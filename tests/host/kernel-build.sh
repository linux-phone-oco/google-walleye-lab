#!/bin/sh
# walleye-declares-its-kernel-build, run on the lab host: does device.toml declare [kernel_builds.NAME] that `lpoco bringup image ensure` can follow?
# Usage: kernel-build.sh NAME [DEVICE_TOML]. Asks `lpoco-bringup image ensure NAME --dry-run` (reads the store, the cache and the seed, changes none):
# its line (captured text of lpoco-bringup 0.48.4, research walleye-kernel-build-declared-reproduces-itself, research/walleye-bootimage/2026-11-24/kernel-ensure-2.log)
# is `NAME: in the store (PATH)`, `NAME: from the cache (PATH)` or `NAME: would be built from its recipe (...)`. Anything else, or a non-zero exit, is a fail.
# Prints `pass|fail|skip walleye-kernel-build DETAIL`; exit 0 only on pass. Environment: LPOCO_BRINGUP (default lpoco-bringup), LPOCO_IMAGE_SEED (default build).
NAME=${1:-}
DEV=${2:-device.toml}
B=${LPOCO_BRINGUP:-lpoco-bringup}
say() { echo "$1 walleye-kernel-build $2"; [ "$1" = pass ]; exit $?; }
[ -n "$NAME" ] || say fail "usage: kernel-build.sh NAME [DEVICE_TOML]"
command -v "$B" >/dev/null || say skip "$B not installed: not known yet"
command -v nix-store >/dev/null || say skip "nix-store not installed: not known yet"
[ -r "$DEV" ] || say fail "no manifest at $DEV"
grep -qxF "[kernel_builds.$NAME]" "$DEV" || say fail "$DEV declares no [kernel_builds.$NAME]"
out=$("$B" image ensure "$NAME" --device "$DEV" --seed "${LPOCO_IMAGE_SEED:-build}" --dry-run 2>&1) || say fail "ensure $NAME: $(echo "$out" | head -1)"
case "$out" in
  "$NAME: in the store ("*|"$NAME: from the cache ("*|"$NAME: would be built from its recipe ("*) say pass "$out" ;;
  *) say fail "ensure $NAME said: $(echo "$out" | head -1)" ;;
esac
