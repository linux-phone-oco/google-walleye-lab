#!/bin/sh
# SPDX-License-Identifier: MIT OR Apache-2.0
# Proof of bin/install-osfact.sh against a stand-in release (file://): it installs the pinned osfact and prints its path;
# a download whose sha256 differs from the release's is refused and kept nowhere; a --version that is not the pinned one
# is refused; a second run keeps the installed program. The real release's text (osfact-v0.1.0: `osfact 0.1.0`, the
# asset name osfact-x86_64 and its .sha256 line) is the shape the stand-in copies (lab-tools #496, item walleye-install-osfact).
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
w=$(mktemp -d); trap 'rm -rf "$w"' EXIT
rel() { # VERSION SAYS: a stand-in release osfact-vVERSION whose program says SAYS
  d="$w/rel/osfact-v$1"; mkdir -p "$d"
  printf '#!/bin/sh\necho "osfact %s"\n' "$2" >"$d/osfact-x86_64"
  (cd "$d" && sha256sum osfact-x86_64 >osfact-x86_64.sha256)
}
run() { OSFACT_BASE="file://$w/rel" OSFACT_VERSION=$1 OSFACT_DEST="$w/bin" sh "$here/bin/install-osfact.sh" 2>"$w/err"; }
fail() { echo "FAIL: $1" >&2; cat "$w/err" >&2; exit 1; }
refused() { # NAME VERSION WORDS
  if run "$2" >"$w/out"; then fail "$1: installed"; fi
  grep -q "$3" "$w/err" || fail "$1: did not say '$3'"
}

rel 0.1.0 "0.1.0"
p=$(run 0.1.0) || fail "a good release is not installed"
[ "$p" = "$w/bin/osfact-0.1.0" ] || fail "path printed is '$p'"
[ "$("$p" --version)" = "osfact 0.1.0" ] || fail "the installed program is not the release's"
touch "$p.kept"; [ "$(run 0.1.0)" = "$p" ] || fail "second run"

rel 0.2.0 "0.2.0"; echo "0000000000000000000000000000000000000000000000000000000000000000  osfact-x86_64" >"$w/rel/osfact-v0.2.0/osfact-x86_64.sha256"
refused "wrong sha256" 0.2.0 "does not match the release's checksum"
[ ! -e "$w/bin/osfact-0.2.0" ] || fail "a mismatching download was kept"

rel 0.3.0 "0.9.9"
refused "wrong --version" 0.3.0 "is not osfact 0.3.0"
echo "install-osfact: ok"
