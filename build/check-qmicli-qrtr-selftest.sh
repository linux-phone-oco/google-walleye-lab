#!/bin/sh
# Known cases for check-qmicli-qrtr.sh (a check is trusted only after it fails on a known bad case).
# The cases are made text, kept small for CI (the real binaries live in the nix store, not at test time); the real ones were seen on 2026-11-27: a8dd865f 0 names, acd5b8b5 5.
# The good case carries the two type names libqrtr-glib registers (QrtrNode, QrtrBus), the bad case is a binary of the 1.38.0 build without QRTR as nix built it on 2026-11-27 (qmicli's own words only: no type name of libqrtr-glib).
set -eu
H=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
printf 'QmiMessageCtlInternalAllocateCidQrtrInput\0%%sQRTR:\0URI is not a local file path\0' > "$T/bad"
printf 'QmiMessageCtlInternalAllocateCidQrtrInput\0QrtrNode\0QrtrBus\0' > "$T/good"
"$H/check-qmicli-qrtr.sh" "$T/good" > /dev/null || { echo "FAIL the good case is refused"; exit 1; }
if "$H/check-qmicli-qrtr.sh" "$T/bad" > /dev/null 2>&1; then echo "FAIL the bad case passes"; exit 1; fi
if "$H/check-qmicli-qrtr.sh" "$T/none" > /dev/null 2>&1; then echo "FAIL a missing file passes"; exit 1; fi
echo "check-qmicli-qrtr-selftest: good passes, bad and missing fail"
