#!/bin/sh
# walleye-ci-image-ensure-rebuild, run on the lab host: does `make land-check` refuse a landing that sets a declared
# sha256 its recipe does not build, and pass one that changes no image? In a throwaway worktree of HEAD, one digit of
# [images.z21] sha256 is changed and committed, then `make land-check LAND_BASE=<HEAD before it>` must end non-zero with
# the line captured in research/walleye-bootimage/2026-11-25/ensure-changed-since-refused.txt ("its recipe makes sha256 ...,
# not the declared ..."); with no commit on top it must pass. Needs nix (the rebuild is in its sandbox), about 40 s.
# Prints `pass|fail|skip walleye-land-check-rebuild DETAIL`; exit 0 only on pass.
say() { echo "$1 walleye-land-check-rebuild $2"; [ "$1" = pass ]; exit $?; }
command -v nix-store >/dev/null || say skip "nix-store not installed: not known yet"
command -v git >/dev/null || say skip "git not installed: not known yet"
W=$(mktemp -d "${TMPDIR:-/tmp}/land-check-rebuild.XXXXXX") || say fail "no scratch folder"
trap 'git worktree remove --force "$W" >/dev/null 2>&1; rm -rf "$W"' EXIT
git worktree add --detach "$W" HEAD >/dev/null 2>&1 || say fail "no worktree of HEAD"
cd "$W" || say fail "cannot enter $W"
base=$(git rev-parse HEAD)
make -s land-check LAND_BASE="$base" >/dev/null 2>&1 || say fail "land-check refused a landing that changes no image"
grep -q '^sha256 = "bdc0a89179d2d7' device.toml || say fail "device.toml no longer declares z21 at bdc0a891: the test's source line moved"
sed -i 's/^sha256 = "bdc0a89179d2d7/sha256 = "bdc0a89279d2d7/' device.toml
git -c user.email=t@t -c user.name=t commit -qam "z21 sha256 its recipe does not build" || say fail "no commit"
out=$(make -s land-check LAND_BASE="$base" 2>&1) && say fail "land-check passed a declared sha256 its recipe does not build"
case "$out" in
  *"z21: its recipe makes sha256 bdc0a89179d2d7"*", not the declared bdc0a89279d2d7"*) say pass "refused: $(echo "$out" | tail -1)" ;;
  *) say fail "refused, but not by the rebuild's line: $(echo "$out" | tail -1)" ;;
esac
