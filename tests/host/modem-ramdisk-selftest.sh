#!/bin/sh
# Known cases for modem-ramdisk.sh: the ramdisk of the build passes, and each defect made in a copy of its tree fails with its own word (a check is trusted only after it fails on a known bad case).
# Environment: RAMDISK (the good one). Exit 0 only when every case came out as expected.
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
CHECK=$HERE/modem-ramdisk.sh
[ -r "${RAMDISK:-}" ] || { echo "skip walleye-modem-ramdisk-selftest need RAMDISK readable: not known yet"; exit 1; }
umask 077; W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
bad=0
mkdir "$W/tree" && (cd "$W/tree" && xz -dc "$RAMDISK" | cpio -id --quiet 2>/dev/null)
pack() { (cd "$W/$1" && find . | cpio -o -H newc --quiet 2>/dev/null | xz --check=crc32 -c) > "$W/$1.xz"; }
want() { # NAME WORD TREE
  pack "$3"; out=$(RAMDISK="$W/$3.xz" sh "$CHECK") && { echo "FAIL $1: the check passed"; bad=1; return; }
  case "$out" in *"$2"*) echo "ok   $1";; *) echo "FAIL $1: '$out' lacks '$2'"; bad=1;; esac
}
if out=$(RAMDISK="$RAMDISK" sh "$CHECK"); then echo "ok   the build's ramdisk passes"; else echo "FAIL the build's ramdisk: $out"; bad=1; fi
cp -a "$W/tree" "$W/nofsc";  rm "$W/nofsc/rmtfs/modem_fsc";                          want "a storage copy missing" "/rmtfs/modem_fsc" nofsc
cp -a "$W/tree" "$W/rw";     chmod 644 "$W/rw/rmtfs/modem_fs1";                      want "a storage copy writable" "not read-only" rw
cp -a "$W/tree" "$W/other";  chmod 644 "$W/other/rmtfs/modem_fs2"; printf x >> "$W/other/rmtfs/modem_fs2"; chmod 444 "$W/other/rmtfs/modem_fs2";                 want "a storage copy changed" "/rmtfs/modem_fs2 is not the phone's" other
cp -a "$W/tree" "$W/extra";  : > "$W/extra/rmtfs/modem_study";                       want "a fifth file in /rmtfs" "more than the four" extra
cp -a "$W/tree" "$W/stray"; : > "$W/stray/modem_b.img";                             want "a stray file anywhere" "not exactly the expected" stray
cp -a "$W/tree" "$W/tool";   printf x >> "$W/tool/bin/rmtfs";                       want "a tool changed" "/bin/rmtfs is not the recorded" tool
cp -a "$W/tree" "$W/qmi";    printf x >> "$W/qmi/bin/qmicli";                        want "qmicli changed" "/bin/qmicli is not the recorded" qmi
cp -a "$W/tree" "$W/noqmi";  rm "$W/noqmi/bin/qmicli";                               want "qmicli missing" "/bin/qmicli is not the recorded" noqmi
cp -a "$W/tree" "$W/up";     printf '#x\n' >> "$W/up/bin/modem-up";                  want "modem-up changed" "/bin/modem-up differs" up
cp -a "$W/tree" "$W/fw";     : > "$W/fw/lib/firmware/qcom/msm8998/google/walleye/mba.mbn"; want "a firmware file inside" "firmware files are in the ramdisk" fw
cp -a "$W/tree" "$W/init";   printf '#x\n' >> "$W/init/init";                        want "init changed" "window end is not" init
cp -a "$W/tree" "$W/old";    cp "$REPO/build/initramfs-slpi/init" "$W/old/init";                  want "slpi's own end, reboot -f first" "window end is not" old
exit $bad
