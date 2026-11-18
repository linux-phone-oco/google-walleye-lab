#!/bin/sh
# banner-shell-selftest: banner-shell-check.sh against made-up captures of the z5 banner; never touches the phone.
H=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
L() { printf 'WALLEYE-BANNER beat=%s up=%ss steps=9 last=[step 9 x] shell=[%s]\r\n' "$1" "$((19+$1*2))" "$2"; }
{ L 0 'started=no'; L 1 'started=yes n=1 pid=40 alive=yes rc=- err=[none]'; L 2 'started=yes n=1 pid=40 alive=yes rc=- err=[none]'; L 3 'started=yes n=1 pid=40 alive=yes rc=- err=[none]'; } > "$T/live"
{ L 0 'started=yes n=1 pid=40 alive=no rc=1 err=[none]'; L 1 'started=yes n=2 pid=41 alive=no rc=1 err=[ash: can'"'"'t open]'; L 2 'started=yes n=3 pid=42 alive=no rc=1 err=[none]'; } > "$T/dead"
{ L 0 'started=no'; L 1 'started=no'; L 2 'started=no'; } > "$T/notstarted"
printf 'WALLEYE-BANNER beat=0 up=1s steps=1 last=[x]\nWALLEYE-BANNER beat=1 up=3s steps=1 last=[x]\nWALLEYE-BANNER beat=2 up=5s steps=1 last=[x]\n' > "$T/noshell"
{ L 0 'started=yes n=1 pid=40 alive=no rc=1 err=[none]'; L 1 'started=yes n=1 pid=40 alive=no rc=1 err=[none]'; L 2 'started=yes n=1 pid=40 alive=yes rc=- err=[none]'; } > "$T/lastlive"
{ L 0 'started=yes n=1 pid=40 alive=yes rc=- err=[none]'; L 1 'started=yes n=1 pid=40 alive=yes rc=- err=[none]'; L 2 'started=yes n=1 pid=40 alive=no rc=1 err=[none]'; } > "$T/lastdead"
{ L 0 'started=yes n=55 pid=40 alive=yes rc=- probe=0 err=[none]'; L 1 'started=yes n=56 pid=41 alive=yes rc=- probe=0 err=[none]'; L 2 'started=yes n=57 pid=42 alive=yes rc=- probe=0 err=[none]'; } > "$T/flap"
: > "$T/empty"
ck "live shell passes" "$(sh "$H/banner-shell-check.sh" "$T/live" | head -n1 | cut -d: -f1)" pass
ck "pass names the last banner line" "$(sh "$H/banner-shell-check.sh" "$T/live" | grep -c 'last banner: WALLEYE-BANNER beat=3 ')" 1
ck "dead shell fails" "$(sh "$H/banner-shell-check.sh" "$T/dead" | head -n1 | cut -d: -f1)" fail
ck "dead shell still prints its last banner line" "$(sh "$H/banner-shell-check.sh" "$T/dead" | grep -c 'last banner: WALLEYE-BANNER beat=2 ')" 1
ck "shell never started fails" "$(sh "$H/banner-shell-check.sh" "$T/notstarted" | head -n1 | cut -d: -f1)" fail
ck "banner without a shell field fails (the banner2 image)" "$(sh "$H/banner-shell-check.sh" "$T/noshell" | head -n1 | cut -d: -f1)" fail
ck "only the LAST line counts: dead then live passes" "$(sh "$H/banner-shell-check.sh" "$T/lastlive" | head -n1 | cut -d: -f1)" pass
ck "only the LAST line counts: live then dead fails" "$(sh "$H/banner-shell-check.sh" "$T/lastdead" | head -n1 | cut -d: -f1)" fail
ck "a shell alive but restarting every beat fails" "$(sh "$H/banner-shell-check.sh" "$T/flap" | head -n1 | cut -d: -f1)" fail
ck "empty capture fails" "$(sh "$H/banner-shell-check.sh" "$T/empty" | head -n1 | cut -d: -f1)" fail
ck "no capture file skips (exit 2)" "$(sh "$H/banner-shell-check.sh" "$T/none" >/dev/null; echo $?)" 2
ck "pass exits 0, fail exits 1" "$(sh "$H/banner-shell-check.sh" "$T/live" >/dev/null; echo $?)$(sh "$H/banner-shell-check.sh" "$T/dead" >/dev/null; echo $?)" 01
[ $total -eq 0 ] && echo "banner-shell-selftest: all ok" || { echo "banner-shell-selftest: $total failed"; exit 1; }
