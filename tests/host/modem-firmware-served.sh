#!/bin/sh
# walleye-modem-firmware-served, run on the lab host: does the firmware tar hold every read-only file the modem asked tqftpserv for and was refused for want of it (item walleye-modem-tqftpserv-serves-what-the-modem-asks)?
# Source: the tqftpserv log the phone printed in run modem-boot-20261127-165116 (tests/fixtures/phone-text/modem-tqftpserv-20261127-165116.cap), and the tqftpserv build's own rules (strings of the pinned build: it serves /readonly/firmware/image/NAME from
# /lib/firmware/<the mss firmware's folder>/NAME; /readonly/vendor/firmware/ is a prefix that build does not know, so a vendor path is asked again under the image path and the file must be there).
# Passes only when each NAME of a `[TQFTP] RRQ: /readonly/firmware/image/NAME ... unable to open ... (2), reject` pair is in the tar, and byte for byte the phone's own /vendor/firmware pull (its sha256sums).
# /readwrite files (mcfg.tmp, ota_firewall/ruleset) are the modem's own: not a file of the tar, not judged here (not known yet). Read-only. Prints `pass|fail|skip walleye-modem-firmware-served modem DETAIL`; exit 0 only on pass.
# Environment: TAR (the firmware tar, no default), LOG (the tqftpserv text; default the fixture), VENDOR (default ~/lab-material/walleye/vendor-firmware-2026-11-24).
HERE=$(cd "$(dirname "$0")/../.." && pwd)
LOG=${LOG:-$HERE/tests/fixtures/phone-text/modem-tqftpserv-20261127-165116.cap}
VENDOR=${VENDOR:-$HOME/lab-material/walleye/vendor-firmware-2026-11-24}
say() { echo "$1 walleye-modem-firmware-served modem $2"; [ "$1" = pass ]; exit $?; }
[ -r "${TAR:-}" ] || say skip "need TAR=file (the firmware tar): not known yet"
[ -r "$VENDOR/sha256sums" ] || say skip "no $VENDOR/sha256sums (the phone's /vendor/firmware pull): not known yet"
names=$(sed -n 's|^\[TQFTP\] unable to open /readonly/firmware/image/\([^ ]*\) (2), reject$|\1|p' "$LOG" | sort -u)
[ -n "$names" ] || say fail "$LOG has no refused /readonly/firmware/image/ request"
tar -tf "$TAR" > /dev/null 2>&1 || say fail "$TAR is not a tar"
miss=$(for n in $names; do
  want=$(sed -n "s|^\([0-9a-f]\{64\}\)  ./firmware/$n\$|\1|p" "$VENDOR/sha256sums")
  if [ -z "$want" ]; then echo "$n is not in the vendor pull"
  elif ! tar -tf "$TAR" "./$n" > /dev/null 2>&1; then echo "$n is not in the tar"
  elif [ "$(tar -xOf "$TAR" "./$n" | sha256sum | cut -d' ' -f1)" != "$want" ]; then echo "$n differs from the phone's pull"; fi
done)
[ -z "$miss" ] || say fail "$(printf '%s' "$miss" | tr '\n' ';')"
say pass "the tar holds $(printf '%s' "$names" | tr '\n' ' ')- as the phone's /vendor/firmware pull has them (each was refused in the phone's log)"
