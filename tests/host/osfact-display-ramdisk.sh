#!/bin/sh
# walleye-osfact-display-ramdisk, run on the lab host: is the ramdisk of the osfact display image what the item says (item walleye-osfact-image-display)?
# Passes only when the ramdisk (an xz cpio) holds: /init equal to build/initramfs-slpi/init plus ONE line, the start of /bin/display-load (build/initramfs-osfact-display/init);
# /bin/display-load equal to the file beside it; busybox with the sha256 of build/initramfs-modem/tools.sha256 (arm64); /display-delay, /display-modules.list and each module of that list
# byte for byte the ones of the b-a5 ramdisk (BA5_RAMDISK, the part device.toml's [images.b-a5.recipe] names, 55c1145e: the modules the glass was lit with, research walleye-ramroot-b-a5-v5-run-2-check-15-passes);
# and NO userland.tar (the root comes by the osfact push). Read-only; extracts to a temp folder (mode 700) that it removes. Prints `pass|fail|skip walleye-osfact-display-ramdisk display DETAIL`; exit 0 only on pass.
# Environment: RAMDISK (the file), BA5_RAMDISK (the b-a5 ramdisk file); no defaults.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-osfact-display-ramdisk display $2"; [ "$1" = pass ]; exit $?; }
[ -r "${RAMDISK:-}" ] && [ -r "${BA5_RAMDISK:-}" ] || say skip "need RAMDISK=file and BA5_RAMDISK=file: not known yet"
for c in xz cpio sha256sum diff; do command -v $c >/dev/null || say skip "$c not installed: not known yet"; done
sum() { [ -f "$1" ] && sha256sum "$1" | cut -d" " -f1; }
umask 077; T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
mkdir "$T/ours" "$T/b-a5"
(cd "$T/ours" && xz -dc "$(realpath "$RAMDISK")" | cpio -id --quiet 2>/dev/null)
(cd "$T/b-a5" && xz -dc "$(realpath "$BA5_RAMDISK")" | cpio -id --quiet 2>/dev/null)
[ -f "$T/ours/init" ] || say fail "$RAMDISK is not an xz cpio with an /init"
# the b-a5 reference is the ramdisk device.toml's [images.b-a5.recipe] names (read from it, not retyped)
want_ba5=$(awk '/^\[images.b-a5.recipe\]/{f=1;next} f&&/^ramdisk = /{gsub(/[^0-9a-f]/,"",$3);print $3;exit}' "$HERE/device.toml")
[ -n "$want_ba5" ] && [ "$(sum "$BA5_RAMDISK")" = "$want_ba5" ] || say fail "BA5_RAMDISK is not the ramdisk of [images.b-a5.recipe] in device.toml"
# what the kernel needs of the cpio: /init and /bin/display-load executable, /bin/sh a link to busybox
xz -dc "$RAMDISK" | cpio -tv 2>/dev/null | awk '$NF=="init"||$NF=="bin/display-load"{ if ($1!="-rwxr-xr-x") bad=1 } $0 ~ /bin\/sh -> busybox/ {l=1} END{exit !(l&&!bad)}' || say fail "/init or /bin/display-load is not mode 755, or /bin/sh is not a link to busybox"
[ -f "$T/b-a5/display-modules.list" ] || say fail "$BA5_RAMDISK is not the b-a5 ramdisk (no display-modules.list)"
want_bb=$(sed -n 's/^\([0-9a-f]\{64\}\)  busybox-arm64$/\1/p' "$HERE/build/initramfs-modem/tools.sha256")
[ -n "$want_bb" ] && [ "$(sum "$T/ours/bin/busybox")" = "$want_bb" ] || say fail "/bin/busybox is not the recorded arm64 build"
# /init: slpi's init and one added line, nothing else (diff prints exactly that line)
added=$(diff "$HERE/build/initramfs-slpi/init" "$T/ours/init" | grep -c '^[<>]')
[ "$added" = 1 ] && diff "$HERE/build/initramfs-slpi/init" "$T/ours/init" | grep -q '^> /bin/display-load &' || say fail "/init is not slpi's init plus the one display-load line"
cmp -s "$T/ours/init" "$HERE/build/initramfs-osfact-display/init" || say fail "/init differs from build/initramfs-osfact-display/init"
cmp -s "$T/ours/bin/display-load" "$HERE/build/initramfs-osfact-display/display-load" || say fail "/bin/display-load differs from the file in build/initramfs-osfact-display"
for f in display-delay display-modules.list; do cmp -s "$T/ours/$f" "$T/b-a5/$f" || say fail "/$f is not b-a5's"; done
n=0   # 13: device.toml [procedures.display-at-boot-on-the-pixel-2] modules
while IFS= read -r line; do
  [ -n "$line" ] || continue
  m=${line#insmod }
  [ -f "$T/ours$m" ] && cmp -s "$T/ours$m" "$T/b-a5$m" || say fail "module $m is not b-a5's, byte for byte"
  n=$((n+1))
done < "$T/b-a5/display-modules.list"
[ "$n" = 13 ] || say fail "the list names $n modules, not the 13 b-a5 loaded"
[ ! -e "$T/ours/userland.tar" ] || say fail "a userland.tar is in the ramdisk (the root comes by the osfact push)"
say pass "init = slpi's plus the display-load line; display-load, delay 5 s, the list and its $n modules are b-a5's byte for byte; busybox is the recorded arm64 build; no userland.tar"
