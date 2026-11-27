#!/bin/sh
# Known cases for modem-firmware.sh: a folder of files sized as the phone's own modem.mdt says (source: the readelf text the phone's file gave, tests/fixtures/phone-text/walleye-modem-mdt-readelf.cap)
# passes, and each defect made in a copy fails with its own word. Exit 0 only when every case came out as expected.
HERE=$(cd "$(dirname "$0")" && pwd)
CHECK=$HERE/modem-firmware.sh
HEADERS=$(cat "$HERE/../fixtures/phone-text/walleye-modem-mdt-readelf.cap"); export HEADERS
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
bad=0
mk() { # DIR: empty files of the sizes the header text gives
  mkdir -p "$1"; : > "$1/modem.mdt"; : > "$1/mba.mbn"
  printf '%s\n' "$HEADERS" | awk '$1=="NULL"||$1=="LOAD" { printf "%d %d\n", n++, strtonum($5) }' | while read -r i sz; do
    [ "$sz" -gt 0 ] && truncate -s "$sz" "$(printf '%s/modem.b%02d' "$1" "$i")"
  done
}
want() { # NAME WORD DIR
  out=$(FW_DIR="$3" sh "$CHECK") && { echo "FAIL $1: the check passed"; bad=1; return; }
  case "$out" in *"$2"*) echo "ok   $1";; *) echo "FAIL $1: '$out' lacks '$2'"; bad=1;; esac
}
mk "$W/good"
if out=$(FW_DIR="$W/good" sh "$CHECK"); then echo "ok   the good folder passes"; else echo "FAIL the good folder: $out"; bad=1; fi
mk "$W/gap";    rm "$W/gap/modem.b12";                     want "a segment file missing" "modem.b12 missing" "$W/gap"
mk "$W/short";  truncate -s 100 "$W/short/modem.b17";      want "a segment file of the wrong size" "modem.b17 is 100 bytes" "$W/short"
mk "$W/extra";  : > "$W/extra/modem.b15"; truncate -s 1 "$W/extra/modem.b15"; want "a file for a header without bytes" "the header has no bytes" "$W/extra"
mk "$W/beyond"; truncate -s 5 "$W/beyond/modem.b40";      want "a segment file beyond the headers" "beyond the 29 headers" "$W/beyond"
mk "$W/nomba";  rm "$W/nomba/mba.mbn";                     want "mba.mbn missing" "no mba.mbn" "$W/nomba"
mk "$W/nomdt";  rm "$W/nomdt/modem.mdt";                   want "modem.mdt missing" "no modem.mdt" "$W/nomdt"
out=$(sh "$CHECK"); case "$out" in skip*) echo "ok   no FW_DIR is a skip";; *) echo "FAIL no FW_DIR: '$out'"; bad=1;; esac
exit $bad
