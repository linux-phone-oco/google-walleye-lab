#!/bin/sh
# Known cases for modem-firmware-served.sh. Source: the tqftpserv text the phone printed (tests/fixtures/phone-text/modem-tqftpserv-20261127-165116.cap: the modem asked for wlanmdsp.mbn under /readonly/firmware/image/ and was refused);
# the vendor pull here is a made-up folder whose sha256sums name a stand-in file of that name (the check only compares a tar's file with the sums beside it). Exit 0 only when every case came out as expected.
HERE=$(cd "$(dirname "$0")" && pwd)
CHECK=$HERE/modem-firmware-served.sh
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
bad=0
mkdir -p "$W/v/firmware" "$W/good" "$W/other" "$W/nofile"
printf 'stand-in modem wlan firmware\n' > "$W/v/firmware/wlanmdsp.mbn"
(cd "$W/v" && sha256sum ./firmware/wlanmdsp.mbn > sha256sums)
cp "$W/v/firmware/wlanmdsp.mbn" "$W/good/"; printf 'mba\n' > "$W/good/mba.mbn"
printf 'a different file\n' > "$W/other/wlanmdsp.mbn"
printf 'mba\n' > "$W/nofile/mba.mbn"
for d in good other nofile; do tar -c -C "$W/$d" -f "$W/$d.tar" .; done
want() { # NAME WORD TAR
  out=$(TAR=$3 VENDOR=$W/v sh "$CHECK") && { echo "FAIL $1: the check passed"; bad=1; return; }
  case "$out" in *"$2"*) echo "ok   $1";; *) echo "FAIL $1: '$out' lacks '$2'"; bad=1;; esac
}
if out=$(TAR=$W/good.tar VENDOR=$W/v sh "$CHECK"); then echo "ok   a tar with the file as the pull has it passes"; else echo "FAIL the good tar: $out"; bad=1; fi
want "a tar without the refused file" "wlanmdsp.mbn is not in the tar" "$W/nofile.tar"
want "a tar with another file of that name" "wlanmdsp.mbn differs from the phone's pull" "$W/other.tar"
printf '' > "$W/empty.cap"
out=$(TAR=$W/good.tar VENDOR=$W/v LOG=$W/empty.cap sh "$CHECK") && { echo "FAIL a log with no refused request passed"; bad=1; } || { case "$out" in *"no refused"*) echo "ok   a log with no refused request fails";; *) echo "FAIL empty log: '$out'"; bad=1;; esac; }
mkdir -p "$W/v2"; : > "$W/v2/sha256sums"
want_v() { out=$(TAR=$W/good.tar VENDOR=$W/v2 sh "$CHECK") && { echo "FAIL a vendor pull without the file passed"; bad=1; return; }; case "$out" in *"not in the vendor pull"*) echo "ok   a vendor pull without the file fails";; *) echo "FAIL vendor: '$out'"; bad=1;; esac; }
want_v
out=$(sh "$CHECK"); case "$out" in skip*) echo "ok   no TAR is a skip";; *) echo "FAIL no TAR: '$out'"; bad=1;; esac
exit $bad
