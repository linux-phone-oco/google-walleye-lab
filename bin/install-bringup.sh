#!/bin/sh
# Installs lpoco-bringup from its pinned release (linux-phone-oco/lab-tools), never the host's own binary: the asset for
# this machine, checked against the release's own sha256 before it is kept. Prints the program's path (one line).
# make land-check runs the rebuild of every image a change touches with it. Version and place by environment.
set -eu
version="${LPOCO_BRINGUP_VERSION:-0.54.4}"
[ "$(uname -m)" = x86_64 ] || { echo "install-bringup: no lpoco-bringup release asset for $(uname -m)" >&2; exit 1; }
name="lpoco-bringup-x86_64-unknown-linux-gnu"
base="https://github.com/linux-phone-oco/lab-tools/releases/download/lpoco-bringup-v$version"
dest="${LPOCO_BRINGUP_DEST:-${XDG_DATA_HOME:-$HOME/.local/share}/walleye-lab/bin}"; prog="$dest/lpoco-bringup-$version"
if [ ! -x "$prog" ]; then
	work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
	curl -fsSL -o "$work/$name" "$base/$name" >&2
	curl -fsSL -o "$work/$name.sha256" "$base/$name.sha256" >&2
	want=$(cut -d' ' -f1 "$work/$name.sha256"); have=$(sha256sum "$work/$name" | cut -d' ' -f1)
	[ -n "$want" ] && [ "$want" = "$have" ] || { echo "install-bringup: the download does not match the release's checksum" >&2; exit 1; }
	mkdir -p "$dest" && install -m 755 "$work/$name" "$prog"
fi
[ "$("$prog" --version)" = "lpoco-bringup $version" ] || { echo "install-bringup: $prog is not lpoco-bringup $version" >&2; exit 1; }
echo "$prog"
