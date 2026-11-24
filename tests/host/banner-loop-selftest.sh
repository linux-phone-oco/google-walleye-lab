#!/bin/sh
# banner-loop-selftest: banner-loop-check.sh against the z6 banner a phone printed (tests/fixtures/phone-text/banner-loop-z6-try1.cap),
# the z5 banner it printed (banner-shell-z5b-try2.cap), and cuts and edits of them for each way to fail; never touches the phone.
H=$(cd "$(dirname "$0")" && pwd)
FX="$H/../fixtures/phone-text"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
cp "$FX/banner-loop-z6-try1.cap" "$T/live"
sed 's/loop=\[started=yes/loop=[started=no/' "$T/live" > "$T/notstarted"
cp "$FX/banner-shell-z5b-try2.cap" "$T/z5"
awk '/^WALLEYE-BANNER beat=/ {n++; sub(/starts=1 /, "starts=" n " ")} {print}' "$T/live" > "$T/flap"
sed 's/$/\r/' "$T/live" > "$T/crlf"
awk '/^WALLEYE-BANNER beat=/ {n++; if (n<=3) sub(/starts=1 /, "starts=0 ")} {print}' "$T/live" > "$T/early"
: > "$T/empty"
v() { sh "$H/banner-loop-check.sh" "$1" | head -n1 | cut -d: -f1; }
ck "the captured z6 banner passes" "$(v "$T/live")" pass
ck "pass names the last banner line" "$(sh "$H/banner-loop-check.sh" "$T/live" | grep -c 'last banner: WALLEYE-BANNER beat=10 ')" 1
ck "pass shows the commands run and the last one" "$(sh "$H/banner-loop-check.sh" "$T/live" | head -n1 | grep -c 'cmds=1 last=\[WALLEYE-BANNER  loop=?\] rc=127')" 1
ck "a loop never opened fails" "$(v "$T/notstarted")" fail
ck "the captured z5 banner (a shell= field, no loop=) fails" "$(v "$T/z5")" fail
ck "a loop that reopens every beat fails" "$(v "$T/flap")" fail
ck "CRLF line ends pass" "$(v "$T/crlf")" pass
ck "a start count that changed before the last 3 lines passes (only the last 3 count)" "$(v "$T/early")" pass
ck "an empty capture fails" "$(v "$T/empty")" fail
ck "no capture file skips (exit 2)" "$(sh "$H/banner-loop-check.sh" "$T/none" >/dev/null; echo $?)" 2
ck "pass exits 0, fail exits 1" "$(sh "$H/banner-loop-check.sh" "$T/live" >/dev/null; echo $?)$(sh "$H/banner-loop-check.sh" "$T/flap" >/dev/null; echo $?)" 01
[ $total -eq 0 ] && echo "banner-loop-selftest: all ok" || { echo "banner-loop-selftest: $total failed"; exit 1; }
