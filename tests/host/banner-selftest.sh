#!/bin/sh
# banner-selftest: banner-check.sh against a capture a phone printed (tests/fixtures/phone-text/banner-z6-try1.cap, sourced in
# its SOURCES.toml) and cuts of it - two lines, beats reversed, nothing - and one line of junk bytes; never touches the phone.
H=$(cd "$(dirname "$0")" && pwd)
CAP="$H/../fixtures/phone-text/banner-z6-try1.cap"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
cp "$CAP" "$T/good"
sed 's/$/\r/' "$CAP" > "$T/crlf"
: > "$T/empty"
grep '^WALLEYE-BANNER beat=' "$CAP" | head -n 2 > "$T/two"
grep '^WALLEYE-BANNER beat=' "$CAP" | sed '1!G;h;$!d' > "$T/back"
printf '\000\377garbage\n' > "$T/junk"
ck "the captured text passes" "$(sh "$H/banner-check.sh" "$T/good" | cut -d: -f1)" pass
ck "the same text with CRLF line ends passes" "$(sh "$H/banner-check.sh" "$T/crlf" | cut -d: -f1)" pass
ck "empty capture fails" "$(sh "$H/banner-check.sh" "$T/empty" | cut -d: -f1)" fail
ck "two lines fail" "$(sh "$H/banner-check.sh" "$T/two" | cut -d: -f1)" fail
ck "beats going back fail" "$(sh "$H/banner-check.sh" "$T/back" | cut -d: -f1)" fail
ck "junk fails" "$(sh "$H/banner-check.sh" "$T/junk" | cut -d: -f1)" fail
ck "no file skips (exit 2)" "$(sh "$H/banner-check.sh" "$T/none" >/dev/null; echo $?)" 2
ck "pass exits 0, fail exits 1" "$(sh "$H/banner-check.sh" "$T/good" >/dev/null; echo $?)$(sh "$H/banner-check.sh" "$T/empty" >/dev/null; echo $?)" 01
[ $total -eq 0 ] && echo "banner-selftest: all ok" || { echo "banner-selftest: $total failed"; exit 1; }
