#!/bin/sh
# Installs osfact from its pinned release (linux-phone-oco/lab-tools, tag osfact-vVERSION), never the host's own binary:
# the asset for this machine, checked against the release's own sha256 and against --version before it is kept. Prints
# the program's path (one line). The version line below is the one rhizoid's edge bumps. Place by environment.
set -eu
version="${OSFACT_VERSION:-0.4.2}"
[ "$(uname -m)" = x86_64 ] || { echo "install-osfact: no osfact release asset for $(uname -m)" >&2; exit 1; }
name="osfact-x86_64"
base="${OSFACT_BASE:-https://github.com/linux-phone-oco/lab-tools/releases/download}/osfact-v$version"
dest="${OSFACT_DEST:-${XDG_DATA_HOME:-$HOME/.local/share}/walleye-lab/bin}"; prog="$dest/osfact-$version"
if [ ! -x "$prog" ]; then
	work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
	curl -fsSL -o "$work/$name" "$base/$name" >&2
	curl -fsSL -o "$work/$name.sha256" "$base/$name.sha256" >&2
	want=$(cut -d' ' -f1 "$work/$name.sha256"); have=$(sha256sum "$work/$name" | cut -d' ' -f1)
	[ -n "$want" ] && [ "$want" = "$have" ] || { echo "install-osfact: the download does not match the release's checksum" >&2; exit 1; }
	chmod 755 "$work/$name"
	[ "$("$work/$name" --version)" = "osfact $version" ] || { echo "install-osfact: the download is not osfact $version" >&2; exit 1; }
	mkdir -p "$dest" && install -m 755 "$work/$name" "$prog"
fi
echo "$prog"
