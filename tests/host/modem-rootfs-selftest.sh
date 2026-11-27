#!/bin/sh
# Known cases for modem-rootfs.sh: the tar of the build passes, and each defect made in a copy fails with its own word.
# Environment: TAR (the good one). Exit 0 only when every case came out as expected.
HERE=$(cd "$(dirname "$0")" && pwd)
CHECK=$HERE/modem-rootfs.sh
[ -r "${TAR:-}" ] || { echo "skip walleye-modem-mm-rootfs-selftest need TAR readable: not known yet"; exit 1; }
umask 077; W=$(mktemp -d); trap 'chmod -R u+w "$W"; rm -rf "$W"' EXIT
bad=0
want() { # NAME WORD TARFILE
  out=$(TAR="$3" sh "$CHECK") && { echo "FAIL $1: the check passed"; bad=1; return; }
  case "$out" in *"$2"*) echo "ok   $1";; *) echo "FAIL $1: '$out' lacks '$2'"; bad=1;; esac
}
if out=$(TAR="$TAR" sh "$CHECK"); then echo "ok   the build's tar passes"; else echo "FAIL the build's tar: $out"; bad=1; fi
mkdir "$W/tree" && tar -xf "$TAR" -C "$W/tree" && chmod -R u+w "$W/tree"
repack() { (cd "$W/$1" && tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner -cf "$W/$1.tar" .); }
cp -a "$W/tree" "$W/nommcli"; find "$W/nommcli" -name mmcli -exec rm -f {} +;   repack nommcli; want "mmcli missing" "mmcli is not in the tar" "$W/nommcli.tar"
cp -a "$W/tree" "$W/noudev";  find "$W/noudev" -name udevd -exec rm -f {} +;     repack noudev;  want "udevd missing" "udevd is not in the tar" "$W/noudev.tar"
cp -a "$W/tree" "$W/fw";      : > "$W/fw/mba.mbn";                                repack fw;      want "firmware inside" "modem storage or firmware" "$W/fw.tar"
cp -a "$W/tree" "$W/st";      : > "$W/st/modemst1";                               repack st;      want "modem storage inside" "modem storage or firmware" "$W/st.tar"
cp -a "$W/tree" "$W/other";   : > "$W/other/extra";                               repack other;   want "a changed tar" "differs from the recorded" "$W/other.tar"
printf 'not a tar' > "$W/junk"; want "not a tar" "is not a tar" "$W/junk"
exit $bad
