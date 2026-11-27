#!/bin/sh
# walleye-modem-mm-rootfs-build, run on the lab host: is the ModemManager rootfs tar what the item says?
#   TAR=file sh tests/host/modem-rootfs.sh
# Passes only when the tar lists the six programs ModemManager needs on the phone (ModemManager, dbus-daemon, mmcli, qmicli, udevd, and the qrtr transport inside qmicli/ModemManager is checked by its library name libqrtr-glib),
# when the programs are aarch64 ELF (read from the tar's own bytes), when no modem storage or firmware is inside (decision 2026-11-27-walleye-modem-image-stays-out-of-the-store), and when its sha256
# equals the one recorded in build/modem-mm-rootfs.sha256 (a second build of the same expression must equal it). Source of the names: item walleye-modem-mm-rootfs-build and decision 2026-11-27-walleye-mm-rootfs-built-by-nix.
# It also fails a tar (or mm-up) lacking what dbus and ModemManager read at start: the bus config and ModemManager's policy in the env share, dbus-uuidgen, messagebus in /etc/passwd and /etc/group, and mm-up's --config-file, /run/dbus and machine-id (item walleye-modem-mm-rootfs-dbus-config).
# Prints `pass|fail|skip walleye-modem-mm-rootfs modem DETAIL`; exit 0 only on pass. Read-only; extracts to a temp folder (mode 700) that it removes.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-modem-mm-rootfs modem $2"; [ "$1" = pass ]; exit $?; }
[ -r "${TAR:-}" ] || say skip "need TAR=file: not known yet"
for c in tar sha256sum file; do command -v $c >/dev/null || say skip "$c not installed: not known yet"; done
LIST=$(tar -tf "$TAR" 2>/dev/null) || say fail "$TAR is not a tar"
for p in ModemManager dbus-daemon mmcli qmicli udevd; do
  echo "$LIST" | grep -Eq "(^|/)(s?bin|libexec)/$p\$" || say fail "$p is not in the tar"
done
echo "$LIST" | grep -Eq '(^|/)libqrtr-glib[^/]*\.so' || say fail "libqrtr-glib is not in the tar (no QRTR transport)"
echo "$LIST" | grep -Eq '(^|/)(modemst[12]|modem_fs[12cg]|fsg|fsc|mba\.mbn|modem\.mdt)$' && say fail "modem storage or firmware is inside the tar"
# What dbus and ModemManager read at start (item walleye-modem-mm-rootfs-dbus-config; source: run 2's dbus.log, research walleye-modem-mm-first-try-run-2-dbus-no-config, and dbus's own share/dbus-1/system.conf):
# nix keeps dbus's config in the store, so mm-up starts the daemon with --config-file to the env's share (proved by running mm-up in modem-boot-selftest.sh); the file, ModemManager's bus policy in the same share, the messagebus user and group, and dbus-uuidgen (machine-id) must be in the tar.
echo "$LIST" | grep -Eq -- '-modem-mm-env/share/dbus-1/system\.conf$' || say fail "dbus's share/dbus-1/system.conf is not in the tar (the bus config)"
echo "$LIST" | grep -Eq -- '-modem-mm-env/share/dbus-1/system\.d/org\.freedesktop\.ModemManager1\.conf$' || say fail "ModemManager's bus policy (system.d/org.freedesktop.ModemManager1.conf) is not in the tar"
echo "$LIST" | grep -Eq -- '-dbus-[^/]*/bin/dbus-uuidgen$' || say fail "dbus-uuidgen is not in the tar (no machine-id)"
echo "$LIST" | grep -Eq '^\./etc/passwd$' || say fail "/etc/passwd is not in the tar"
echo "$LIST" | grep -Eq '^\./etc/group$' || say fail "/etc/group is not in the tar"
umask 077; T=$(mktemp -d); trap 'chmod -R u+w "$T"; rm -rf "$T"' EXIT
tar -xf "$TAR" -C "$T" --wildcards '*/bin/ModemManager' '*/bin/mmcli' '*/bin/dbus-daemon' '*/bin/qmicli' '*/bin/udevd' || say fail "could not extract the five programs from the tar"
n=$(find "$T" -type f | wc -l)
[ "$n" -ge 5 ] || say fail "only $n of ModemManager, dbus-daemon, mmcli, qmicli, udevd could be read from the tar"
[ -z "$(find "$T" -type f -exec sh -c 'file "$1" | grep -q "ELF 64-bit LSB.*ARM aarch64" || echo "$1"' _ {} \;)" ] || say fail "a program in the tar is not an aarch64 ELF"
P=$(tar -xOf "$TAR" ./etc/passwd 2>/dev/null); G=$(tar -xOf "$TAR" ./etc/group 2>/dev/null)
echo "$P" | grep -q "^root:" || say fail "/etc/passwd has no root (ModemManager runs as root)"
echo "$P" | grep -q "^messagebus:x:[0-9]*:100:" || say fail "/etc/passwd has no messagebus user with gid 100 (the bus config's <user>)"
echo "$G" | grep -q "^messagebus:x:100:" || say fail "/etc/group has no messagebus group 100"
want=$(sed -n 's/^\([0-9a-f]\{64\}\)  modem-mm-rootfs.tar$/\1/p' "$HERE/build/modem-mm-rootfs.sha256" 2>/dev/null)
[ -n "$want" ] || say fail "no sha256 recorded in build/modem-mm-rootfs.sha256"
got=$(sha256sum "$TAR" | cut -d' ' -f1)
[ "$got" = "$want" ] || say fail "sha256 $got differs from the recorded $want"
say pass "ModemManager, dbus-daemon, mmcli, qmicli, udevd and libqrtr-glib are in the tar; aarch64 ELF; no modem storage or firmware; sha256 $got equals the record"
