#!/bin/sh
# walleye-osfact-nfc-ramdisk, run on the lab host: is the ramdisk of the osfact NFC image what the item says (item walleye-osfact-image-nfc)?
# Passes only when the ramdisk (an xz or gzip cpio) holds: /init equal to build/initramfs-slpi/init (NO added line: the NFC open is the host's, over the shell, as the NFC boot procedure does it) and to
# build/initramfs-osfact-nfc/init; /bin/walleye-nfc-up byte for byte the one of the proven NFC ramdisk (NFC_RAMDISK, the part device.toml's [images.nfc.recipe] names, f4b6c1d7: the opener that
# acked DEV_UP, research walleye-nfc-poll-no-tag-run); busybox with the sha256 of build/initramfs-modem/tools.sha256 (arm64); modes of /init and the tool 755, /bin/sh a link
# to busybox; and NO userland.tar and NO firmware (the root comes by the osfact push; NFC needs none). Read-only; extracts to a temp folder (mode 700) that it removes.
# Prints `pass|fail|skip walleye-osfact-nfc-ramdisk nfc DETAIL`; exit 0 only on pass. Environment: RAMDISK (the file), NFC_RAMDISK (the file); no defaults.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-osfact-nfc-ramdisk nfc $2"; [ "$1" = pass ]; exit $?; }
[ -r "${RAMDISK:-}" ] && [ -r "${NFC_RAMDISK:-}" ] || say skip "need RAMDISK=file and NFC_RAMDISK=file: not known yet"
for c in xz gzip cpio sha256sum cmp; do command -v $c >/dev/null || say skip "$c not installed: not known yet"; done
sum() { [ -f "$1" ] && sha256sum "$1" | cut -d" " -f1; }
unz() { case $(head -c2 "$1" | od -An -tx1 | tr -d ' ') in 1f8b) gzip -dc "$1" ;; *) xz -dc "$1" ;; esac; }
umask 077; T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
mkdir "$T/ours" "$T/nfc"
(cd "$T/ours" && unz "$(realpath "$RAMDISK")" | cpio -id --quiet 2>/dev/null)
(cd "$T/nfc" && unz "$(realpath "$NFC_RAMDISK")" | cpio -id --quiet 2>/dev/null)
[ -f "$T/ours/init" ] || say fail "$RAMDISK is not a cpio with an /init"
want_nfc=$(awk '/^\[images.nfc.recipe\]/{f=1;next} f&&/^ramdisk = /{gsub(/[^0-9a-f]/,"",$3);print $3;exit}' "$HERE/device.toml")
[ -n "$want_nfc" ] && [ "$(sum "$NFC_RAMDISK")" = "$want_nfc" ] || say fail "NFC_RAMDISK is not the ramdisk of [images.nfc.recipe] in device.toml"
[ -x "$T/nfc/bin/walleye-nfc-up" ] || say fail "$NFC_RAMDISK holds no /bin/walleye-nfc-up"
unz "$RAMDISK" | cpio -tv 2>/dev/null | awk '$NF=="init"||$NF=="bin/walleye-nfc-up"{ if ($1!="-rwxr-xr-x") bad=1 } $0 ~ /bin\/sh -> busybox/ {l=1} END{exit !(l&&!bad)}' || say fail "/init or /bin/walleye-nfc-up is not mode 755, or /bin/sh is not a link to busybox"
want_bb=$(sed -n 's/^\([0-9a-f]\{64\}\)  busybox-arm64$/\1/p' "$HERE/build/initramfs-modem/tools.sha256")
[ -n "$want_bb" ] && [ "$(sum "$T/ours/bin/busybox")" = "$want_bb" ] || say fail "/bin/busybox is not the recorded arm64 build"
cmp -s "$T/ours/init" "$HERE/build/initramfs-slpi/init" || say fail "/init differs from build/initramfs-slpi/init"
cmp -s "$T/ours/init" "$HERE/build/initramfs-osfact-nfc/init" || say fail "/init differs from build/initramfs-osfact-nfc/init"
cmp -s "$T/ours/etc/udhcpd.conf" "$HERE/build/initramfs-slpi/udhcpd.conf" || say fail "/etc/udhcpd.conf differs from build/initramfs-slpi/udhcpd.conf"
cmp -s "$T/ours/bin/walleye-nfc-up" "$T/nfc/bin/walleye-nfc-up" || say fail "/bin/walleye-nfc-up is not the proven NFC ramdisk's, byte for byte"
[ ! -e "$T/ours/userland.tar" ] || say fail "a userland.tar is in the ramdisk (the root comes by the osfact push)"
[ ! -e "$T/ours/lib/firmware" ] || say fail "firmware is in the ramdisk (NFC needs none)"
say pass "init = slpi's, unchanged; /bin/walleye-nfc-up is the proven tool (the NFC ramdisk's) byte for byte; udhcpd.conf is slpi's; busybox is the recorded arm64 build; no userland.tar, no firmware"
