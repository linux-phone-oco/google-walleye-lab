#!/bin/sh
# Known cases for modem-tqftpserv-clean.sh. Source: the tqftpserv text the phone printed (tests/fixtures/phone-text/modem-tqftpserv-20261127-165116.cap): as captured it fails (refused reads, the mcfg.tmp loop);
# the passing and half-passing cases are that capture with lines taken out (grep -v), never lines typed. Exit 0 only when every case came out as expected.
HERE=$(cd "$(dirname "$0")" && pwd)
CHECK=$HERE/modem-tqftpserv-clean.sh
CAP=$HERE/../fixtures/phone-text/modem-tqftpserv-20261127-165116.cap
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
bad=0
case_() { # NAME EXPECT(pass|fail) WORD LOGFILE
  out=$(LOG=$4 sh "$CHECK"); rc=$?
  case "$2:$rc" in pass:0|fail:1) case "$out" in *"$3"*) echo "ok   $1";; *) echo "FAIL $1: '$out' lacks '$3'"; bad=1;; esac;; *) echo "FAIL $1: expected $2, got '$out'"; bad=1;; esac
}
case_ "the capture as printed fails" fail "refused: " "$CAP"
grep -v 'unable to open' "$CAP" > "$W/stderr-only.cap"
case_ "the stderr form of the refusals alone fails" fail "refused: " "$W/stderr-only.cap"
grep -v 'unable to open\|invalid path\|failed to open ota' "$CAP" > "$W/noref.cap"
case_ "the capture without its refused reads still shows the loop" fail "mcfg.tmp asked" "$W/noref.cap"
grep -v 'unable to open\|invalid path\|failed to open ota' "$CAP" | awk '/WRQ: \/readwrite\/mcfg.tmp/ { if (++n > 2) next } { print }' > "$W/clean.cap"
case_ "the capture without refused reads and with the loop cut to two passes" pass "no refused" "$W/clean.cap"
grep -v 'WRQ: /readwrite/mcfg.tmp' "$CAP" > "$W/noloop.cap"
case_ "the capture without the loop still has refused reads" fail "/readonly/firmware/image/wlanmdsp.mbn" "$W/noloop.cap"
: > "$W/empty.cap"
out=$(LOG=$W/empty.cap sh "$CHECK"); case "$out" in skip*) echo "ok   a log with no [TQFTP] line is a skip";; *) echo "FAIL empty: '$out'"; bad=1;; esac
out=$(sh "$CHECK"); case "$out" in skip*) echo "ok   no LOG is a skip";; *) echo "FAIL no LOG: '$out'"; bad=1;; esac
exit $bad
