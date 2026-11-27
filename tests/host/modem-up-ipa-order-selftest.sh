#!/bin/sh
# Proves tests/host/modem-up-ipa-order.sh on the real modem-up and on edits of it: the real one passes; the bind moved after rmtfs, the bind removed, and the bind before the sums check each fail.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
SRC="$HERE/../../build/initramfs-modem/modem-up"
CHECK="$HERE/modem-up-ipa-order.sh"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { got=$(MODEM_UP="$3" sh "$CHECK" | cut -d' ' -f1 || true); if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got '$got', want $2"; total=$((total+1)); fi; }
want "the real modem-up" pass "$SRC"
grep -v 'echo \$IPA > \$IPAD/bind' "$SRC" > "$T/nobind"
want "no bind line" fail "$T/nobind"
grep -v '\$IPAD/\(un\)\{0,1\}bind' "$SRC" > "$T/late"; grep 'echo \$IPA > \$IPAD/' "$SRC" >> "$T/late"
want "the bind after the daemons" fail "$T/late"
awk '/sha256sum -c SHA256SUMS/ { held=$0; next } { print } /echo \$IPA > \$IPAD\/bind/ { print held }' "$SRC" > "$T/early"
want "the sums check after the bind" fail "$T/early"
[ $total -eq 0 ] || { echo "modem-up-ipa-order-selftest: $total failed" >&2; exit 1; }
echo "pass modem-up-ipa-order-selftest"
