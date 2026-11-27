#!/bin/sh
# qmicli has libqmi's QRTR transport compiled in (item walleye-modem-qmicli-qrtr-enabled).
#   check-qmicli-qrtr.sh QMICLI   exit 1 when the binary carries no libqrtr-glib type (QrtrNode, QrtrBus)
# Why these names: libqmi 1.38's meson option `qrtr` links libqrtr-glib into qmi-device (nixpkgs' libqmi expression ties it to withIntrospection);
# libqrtr-glib registers the GObject types QrtrNode and QrtrBus, whose names are strings in any binary that links it.
# Seen failing on the 1.38.0 build of a8dd865f (nix store kbcsmf3f..-libqmi-static-..-1.38.0): 0 such strings, qmicli said 'URI is not a local file path' (research walleye-modem-qmicli-qrtr-uri-error).
set -eu
b=${1:?usage: check-qmicli-qrtr.sh QMICLI}
[ -f "$b" ] || { echo "FAIL $b is not a file" >&2; exit 1; }
n=$(strings "$b" | grep -c 'QrtrNode\|QrtrBus' || true)
[ "$n" -gt 0 ] || { echo "FAIL $b: no libqrtr-glib type name (QrtrNode, QrtrBus): built without QRTR" >&2; exit 1; }
echo "check-qmicli-qrtr: $b carries libqrtr-glib ($n type-name strings)"
