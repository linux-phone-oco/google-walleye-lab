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
cp -a "$W/tree" "$W/notelf"; find "$W/notelf" -name ModemManager -type f -exec sh -c 'printf "#!/bin/sh\n" > "$1"' _ {} \; ; repack notelf; want "ModemManager not an ELF" "not an aarch64 ELF" "$W/notelf.tar"
cp -a "$W/tree" "$W/noqrtr"; find "$W/noqrtr" -name 'libqrtr-glib*' -exec rm -rf {} + ; repack noqrtr; want "no QRTR library" "libqrtr-glib is not in the tar" "$W/noqrtr.tar"
cp -a "$W/tree" "$W/nopw";  rm -f "$W/nopw/etc/passwd";                             repack nopw;  want "no passwd" "/etc/passwd is not in the tar" "$W/nopw.tar"
cp -a "$W/tree" "$W/nomb";  printf 'root:x:0:0:root:/:/bin/sh\n' > "$W/nomb/etc/passwd"; repack nomb;  want "no messagebus user" "no messagebus user" "$W/nomb.tar"
cp -a "$W/tree" "$W/nogr";  rm -f "$W/nogr/etc/group";                               repack nogr;  want "no group" "/etc/group is not in the tar" "$W/nogr.tar"
cp -a "$W/tree" "$W/nopol"; find "$W/nopol" -name org.freedesktop.ModemManager1.conf -exec rm -f {} + ; repack nopol; want "no ModemManager bus policy" "bus policy" "$W/nopol.tar"
cp -a "$W/tree" "$W/nosc";  find "$W/nosc" -path '*-modem-mm-env/share/dbus-1/system.conf' -exec rm -f {} + ; repack nosc; want "no bus config" "bus config" "$W/nosc.tar"
cp -a "$W/tree" "$W/nouu";  find "$W/nouu" -name dbus-uuidgen -exec rm -f {} + ;       repack nouu;  want "no dbus-uuidgen" "dbus-uuidgen is not in the tar" "$W/nouu.tar"
printf 'not a tar' > "$W/junk"; want "not a tar" "is not a tar" "$W/junk"
exit $bad
