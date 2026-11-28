#!/bin/sh
# The ramdisk of the osfact display-and-NFC image (item walleye-osfact-image-display-and-nfc): slpi's init plus the display-load line (as the osfact display image), the 13 display modules
# of THE KERNEL OF THIS IMAGE (build-display-late.sh's modroot of the tag int/walleye/display-smmu-id/v7.3/1 with walleye-nfc.config), and /bin/walleye-nfc-up (the NFC image's proven tool).
# Nothing here touches a phone. The spec is build/initramfs-osfact-display-nfc/spec plus one dir/file line per module found in LATE_DIR/modroot.
#   build-osfact-display-nfc-ramdisk.sh LATE_DIR DELAY_FILE NFC_UP ROOT OUT_RAMDISK
#   LATE_DIR: the output folder of build-display-late.sh (modroot/, display-modules.list); DELAY_FILE: a file holding the display delay (B-r5-smmu-id's /display-delay: 5);
#   NFC_UP: the walleye-nfc-up binary; ROOT: an empty folder to stage in; OUT_RAMDISK: the xz cpio to write
set -eu
LATE=${1:?usage: build-osfact-display-nfc-ramdisk.sh LATE_DIR DELAY_FILE NFC_UP ROOT OUT_RAMDISK}; DELAY=${2:?}; NFCUP=${3:?}; ROOT=${4:?}; OUT=${5:?}
HERE=$(cd "$(dirname "$0")/.." && pwd)
SRC=$HERE/build/initramfs-osfact-display-nfc
want_bb=$(sed -n 's/^\([0-9a-f]\{64\}\)  busybox-arm64$/\1/p' "$HERE/build/initramfs-modem/tools.sha256")
BB=${BUSYBOX:-$HOME/Projects/walleye-build/busybox-arm64}
[ "$(sha256sum "$BB" | cut -d' ' -f1)" = "$want_bb" ] || { echo "build-osfact-display-nfc-ramdisk: $BB is not the recorded arm64 busybox" >&2; exit 1; }
[ -z "$(ls -A "$ROOT" 2>/dev/null)" ] || { echo "build-osfact-display-nfc-ramdisk: ROOT is not empty" >&2; exit 1; }
mkdir -p "$ROOT"
cp "$BB" "$ROOT/busybox"; cp "$HERE/build/initramfs-osfact-display/init" "$HERE/build/initramfs-osfact-display/display-load" "$ROOT/"; cp "$HERE/build/initramfs-slpi/udhcpd.conf" "$ROOT/"
cp "$NFCUP" "$ROOT/walleye-nfc-up"; cp "$DELAY" "$ROOT/display-delay"; cp "$LATE/display-modules.list" "$ROOT/display-modules.list"
cp -r "$LATE/modroot/lib" "$ROOT/lib"
KVER=$(ls "$ROOT/lib/modules")
[ "$(printf '%s\n' "$KVER" | wc -l)" -eq 1 ] || { echo "build-osfact-display-nfc-ramdisk: want one kernel version in modroot, got: $KVER" >&2; exit 1; }
{ cat "$SRC/spec"
  (cd "$ROOT" && find "lib/modules/$KVER" -type d | sort | awk '{ print "dir /" $0 " 755 0 0" }')
  (cd "$ROOT" && find "lib/modules/$KVER" -type f -name '*.ko' | sort | awk '{ print "file /" $0 " " $0 " 644 0 0" }')
} | awk '!s[$0]++' > "$ROOT/spec"
# the list says insmod lines whose every file is in the spec
sed -n 's/^insmod *//p' "$ROOT/display-modules.list" | while IFS= read -r p; do grep -q "^file $p " "$ROOT/spec" || { echo "build-osfact-display-nfc-ramdisk: $p of the list is not in the ramdisk" >&2; exit 1; }; done
lpoco-bringup initramfs build --spec "$ROOT/spec" --root "$ROOT" --codec xz --out "$OUT"
sha256sum "$OUT"
