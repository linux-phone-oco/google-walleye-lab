#!/bin/sh
# banner-shell-selftest: banner-shell-check.sh against the z5 banner a phone printed (tests/fixtures/phone-text/banner-shell-z5b-try2.cap),
# the z6 banner it printed (banner-loop-z6-try1.cap, which has no shell= field), and cuts and edits of them; never touches the phone.
H=$(cd "$(dirname "$0")" && pwd)
FX="$H/../fixtures/phone-text"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
DEAD='s/alive=yes rc=-/alive=no rc=1/'
cp "$FX/banner-shell-z5b-try2.cap" "$T/live"
sed "$DEAD" "$T/live" > "$T/dead"
sed 's/shell=\[started=yes/shell=[started=no/' "$T/live" > "$T/notstarted"
cp "$FX/banner-loop-z6-try1.cap" "$T/noshell"
{ grep '^WALLEYE-BANNER beat=' "$T/live" | sed '$d' | sed "$DEAD"; grep '^WALLEYE-BANNER beat=' "$T/live" | tail -n1; } > "$T/lastlive"
{ grep '^WALLEYE-BANNER beat=' "$T/live" | sed '$d'; grep '^WALLEYE-BANNER beat=' "$T/live" | tail -n1 | sed "$DEAD"; } > "$T/lastdead"
awk '/^WALLEYE-BANNER beat=/ {n++; sub(/ n=1 /, " n=" n " ")} {print}' "$T/live" > "$T/flap"
sed 's/$/\r/' "$T/live" > "$T/crlf"
: > "$T/empty"
v() { sh "$H/banner-shell-check.sh" "$1" | head -n1 | cut -d: -f1; }
ck "the captured shell banner passes" "$(v "$T/live")" pass
ck "pass names the last banner line" "$(sh "$H/banner-shell-check.sh" "$T/live" | grep -c 'last banner: WALLEYE-BANNER beat=14 ')" 1
ck "dead shell fails" "$(v "$T/dead")" fail
ck "dead shell still prints its last banner line" "$(sh "$H/banner-shell-check.sh" "$T/dead" | grep -c 'last banner: WALLEYE-BANNER beat=14 ')" 1
ck "shell never started fails" "$(v "$T/notstarted")" fail
ck "banner without a shell field fails (the z6 banner)" "$(v "$T/noshell")" fail
ck "only the LAST line counts: dead then live passes" "$(v "$T/lastlive")" pass
ck "only the LAST line counts: live then dead fails" "$(v "$T/lastdead")" fail
ck "a shell alive but restarting every beat fails" "$(v "$T/flap")" fail
ck "CRLF line ends pass" "$(v "$T/crlf")" pass
ck "empty capture fails" "$(v "$T/empty")" fail
ck "no capture file skips (exit 2)" "$(sh "$H/banner-shell-check.sh" "$T/none" >/dev/null; echo $?)" 2
ck "pass exits 0, fail exits 1" "$(sh "$H/banner-shell-check.sh" "$T/live" >/dev/null; echo $?)$(sh "$H/banner-shell-check.sh" "$T/dead" >/dev/null; echo $?)" 01
[ $total -eq 0 ] && echo "banner-shell-selftest: all ok" || { echo "banner-shell-selftest: $total failed"; exit 1; }
