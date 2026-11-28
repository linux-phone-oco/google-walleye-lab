#!/bin/sh
# walleye-osfact-display-nfc-ramdisk, run on the lab host: is the ramdisk of the osfact display-and-NFC image what the item says (item walleye-osfact-image-display-and-nfc)?
# equal to the file beside it; /bin/walleye-nfc-up byte for byte the one of the proven NFC ramdisk (NFC_RAMDISK, the part device.toml's [images.nfc.recipe] names: the opener that acked DEV_UP);
# /display-delay equal to the one of the identity-list image's ramdisk (ID_RAMDISK, the part [images.b-r5-smmu-id.recipe] names); busybox with the sha256 of build/initramfs-modem/tools.sha256 (arm64);
# /display-modules.list and each of its modules byte for byte the build's (LATE_DIR, build-display-late.sh's output for the kernel of the image), 13 of them, all under ONE kernel version;
# and NO userland.tar and NO firmware (the root comes by the osfact push; NFC needs none). Read-only; extracts to a temp folder (mode 700) that it removes.
# Prints `pass|fail|skip walleye-osfact-display-nfc-ramdisk display+nfc DETAIL`; exit 0 only on pass. Environment: RAMDISK, NFC_RAMDISK, ID_RAMDISK (files), LATE_DIR (folder); no defaults.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-osfact-display-nfc-ramdisk display+nfc $2"; [ "$1" = pass ]; exit $?; }
[ -r "${RAMDISK:-}" ] && [ -r "${NFC_RAMDISK:-}" ] && [ -r "${ID_RAMDISK:-}" ] && [ -d "${LATE_DIR:-}/modroot" ] || say skip "need RAMDISK, NFC_RAMDISK, ID_RAMDISK (files) and LATE_DIR (folder): not known yet"
for c in xz gzip cpio sha256sum cmp; do command -v $c >/dev/null || say skip "$c not installed: not known yet"; done
sum() { [ -f "$1" ] && sha256sum "$1" | cut -d" " -f1; }
unz() { case $(head -c2 "$1" | od -An -tx1 | tr -d ' ') in 1f8b) gzip -dc "$1" ;; *) xz -dc "$1" ;; esac; }
want_of() { awk -v n="$1" '$0 ~ "^\\[images\\." n "\\.recipe\\]" {f=1; next} f && /^ramdisk = / {gsub(/[^0-9a-f]/, "", $3); print $3; exit}' "$HERE/device.toml"; }
umask 077; T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
mkdir "$T/ours" "$T/nfc" "$T/id"
(cd "$T/ours" && unz "$(realpath "$RAMDISK")" | cpio -id --quiet 2>/dev/null)
(cd "$T/nfc" && unz "$(realpath "$NFC_RAMDISK")" | cpio -id --quiet 2>/dev/null)
(cd "$T/id" && unz "$(realpath "$ID_RAMDISK")" | cpio -id --quiet 2>/dev/null)
[ -f "$T/ours/init" ] || say fail "$RAMDISK is not a cpio with an /init"
[ "$(sum "$NFC_RAMDISK")" = "$(want_of nfc)" ] || say fail "NFC_RAMDISK is not the ramdisk of [images.nfc.recipe] in device.toml"
[ "$(sum "$ID_RAMDISK")" = "$(want_of b-r5-smmu-id)" ] || say fail "ID_RAMDISK is not the ramdisk of [images.b-r5-smmu-id.recipe] in device.toml"
[ -x "$T/nfc/bin/walleye-nfc-up" ] || say fail "$NFC_RAMDISK holds no /bin/walleye-nfc-up"
unz "$RAMDISK" | cpio -tv 2>/dev/null | awk '$NF=="init"||$NF=="bin/display-load"||$NF=="bin/walleye-nfc-up"{ if ($1!="-rwxr-xr-x") bad=1 } $0 ~ /bin\/sh -> busybox/ {l=1} END{exit !(l&&!bad)}' || say fail "/init, /bin/display-load or /bin/walleye-nfc-up is not mode 755, or /bin/sh is not a link to busybox"
want_bb=$(sed -n 's/^\([0-9a-f]\{64\}\)  busybox-arm64$/\1/p' "$HERE/build/initramfs-modem/tools.sha256")
[ -n "$want_bb" ] && [ "$(sum "$T/ours/bin/busybox")" = "$want_bb" ] || say fail "/bin/busybox is not the recorded arm64 build"
cmp -s "$T/ours/init" "$HERE/build/initramfs-osfact-display/init" || say fail "/init differs from build/initramfs-osfact-display/init"
cmp -s "$T/ours/bin/display-load" "$HERE/build/initramfs-osfact-display/display-load" || say fail "/bin/display-load differs from build/initramfs-osfact-display/display-load"
cmp -s "$T/ours/etc/udhcpd.conf" "$HERE/build/initramfs-slpi/udhcpd.conf" || say fail "/etc/udhcpd.conf differs from build/initramfs-slpi/udhcpd.conf"
cmp -s "$T/ours/bin/walleye-nfc-up" "$T/nfc/bin/walleye-nfc-up" || say fail "/bin/walleye-nfc-up is not the proven NFC ramdisk's, byte for byte"
cmp -s "$T/ours/display-delay" "$T/id/display-delay" || say fail "/display-delay differs from the identity-list image's"
cmp -s "$T/ours/display-modules.list" "$LATE_DIR/display-modules.list" || say fail "/display-modules.list differs from the build's (LATE_DIR)"
n=$(sed -n 's/^insmod *//p' "$T/ours/display-modules.list" | wc -l)
[ "$n" = 13 ] || say fail "the list has $n modules, not the 13 the display images load"
kv=$(ls "$T/ours/lib/modules")
[ "$(printf '%s\n' "$kv" | wc -l)" = 1 ] || say fail "more than one kernel version of modules in the ramdisk: $kv"
while IFS= read -r p; do
	cmp -s "$T/ours$p" "$LATE_DIR/modroot$p" || say fail "module $p is not the build's, byte for byte"
	case $p in /lib/modules/"$kv"/*) ;; *) say fail "module $p is not under the one kernel version $kv" ;; esac
done <<LIST
$(sed -n 's/^insmod *//p' "$T/ours/display-modules.list")
LIST
[ "$(find "$T/ours/lib/modules" -name '*.ko' | wc -l)" = "$n" ] || say fail "the ramdisk holds modules the list does not name"
[ ! -e "$T/ours/userland.tar" ] || say fail "a userland.tar is in the ramdisk (the root comes by the osfact push)"
[ ! -e "$T/ours/lib/firmware" ] || say fail "firmware is in the ramdisk (NFC needs none)"
say pass "init = display image's (slpi's plus the display-load line); /bin/walleye-nfc-up is the proven tool byte for byte; $n modules of $kv are the build's byte for byte; delay is the identity image's; busybox is the recorded arm64 build; no userland.tar, no firmware"
