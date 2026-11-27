#!/bin/sh
# walleye-modem-tqftpserv-clean, run on the lab host: did a modem boot's tqftpserv log show the modem served (item walleye-modem-tqftpserv-serves-what-the-modem-asks)?
# Source: the tqftpserv text the phone printed in run modem-boot-20261127-165116 (tests/fixtures/phone-text/modem-tqftpserv-20261127-165116.cap): there the log had 9 counted write requests for /readwrite/mcfg.tmp (the loop), refused reads of
# /readonly/firmware/image/wlanmdsp.mbn and /readwrite/ota_firewall/ruleset. Passes only when the log has no `unable to open ... (2), reject` of a /readonly or ota_firewall file and fewer than 3 write requests for any one /readwrite file.
# The 3 is a choice, not a measure: the capture shows 9 and no run has shown a clean boot yet - the first clean boot decides the threshold (and whether ota_firewall/ruleset is asked at all); until then a fail here is not a proof that the modem is unserved.
# Environment: LOG (the tqftpserv text, a daemon-logs.txt section or the whole file; no default).
say() { echo "$1 walleye-modem-tqftpserv-clean modem $2"; [ "$1" = pass ]; exit $?; }
[ -r "${LOG:-}" ] || say skip "need LOG=file (a modem boot's tqftpserv log): not known yet"
grep -q '^\[TQFTP\]' "$LOG" || say skip "$LOG has no [TQFTP] line: tqftpserv said nothing, not known yet"
# the log is block-buffered stdout beside unbuffered stderr (modem-up redirects both to a file): a line may be cut and glued to a stderr line, so lines are split at "tqftpserv:" and matched unanchored
norm=$(sed 's/tqftpserv:/\ntqftpserv:/g' "$LOG")
refused=$(printf '%s\n' "$norm" | sed -n 's#.*unable to open \(/readonly/[^ ]*\|/readwrite/ota_firewall/[^ ]*\) (2), reject.*#\1#p; s#^invalid path \(/readonly/[^ ,]*\), rejecting.*#\1#p; s#^tqftpserv: failed to open \(ota_firewall/[^:]*\):.*#\1#p' | sort -u | tr '\n' ' ')
[ -z "$refused" ] || say fail "refused: $refused"
loop=$(printf '%s\n' "$norm" | sed -n 's#.*WRQ: \(/readwrite/[^ ]*\).*#\1#p' | sort | uniq -c | awk '$1>=3 { printf "%s asked %d times; ", $2, $1 }')
[ -z "$loop" ] || say fail "write loop: $loop"
say pass "no refused /readonly or ota_firewall request and no /readwrite file written 3 times or more"
