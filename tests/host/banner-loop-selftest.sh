#!/bin/sh
# banner-loop-selftest: banner-loop-check.sh against made-up captures of the z6 banner; never touches the phone.
H=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
L() { printf 'WALLEYE-BANNER beat=%s up=%ss steps=9 last=[step 9 x] loop=[%s]\r\n' "$1" "$((19+$1*2))" "$2"; }
OK='started=yes starts=1 cmds=3 last=[echo hi] rc=0 err=[none]'
{ L 0 'started=no starts=0 cmds=0 last=[-] rc=- err=[none]'; L 1 "$OK"; L 2 "$OK"; L 3 "$OK"; } > "$T/live"
{ L 0 'started=no starts=0 cmds=0 last=[-] rc=- err=[none]'; L 1 'started=no starts=0 cmds=0 last=[-] rc=- err=[none]'; L 2 'started=no starts=0 cmds=0 last=[-] rc=- err=[none]'; } > "$T/notstarted"
{ L 0 "$OK"; L 1 "$OK"; L 2 "$OK"; } > "$T/last"
printf 'WALLEYE-BANNER beat=0 up=1s steps=1 last=[x] shell=[started=yes n=1 pid=4 alive=yes rc=- probe=0 err=[none]]\nWALLEYE-BANNER beat=1 up=3s steps=1 last=[x] shell=[started=yes n=1 pid=4 alive=yes rc=- probe=0 err=[none]]\nWALLEYE-BANNER beat=2 up=5s steps=1 last=[x] shell=[started=yes n=1 pid=4 alive=yes rc=- probe=0 err=[none]]\n' > "$T/z5"
{ L 0 'started=yes starts=4 cmds=0 last=[-] rc=- err=[none]'; L 1 'started=yes starts=5 cmds=0 last=[-] rc=- err=[none]'; L 2 'started=yes starts=6 cmds=0 last=[-] rc=- err=[none]'; } > "$T/flap"
: > "$T/empty"
v() { sh "$H/banner-loop-check.sh" "$1" | head -n1 | cut -d: -f1; }
ck "an open loop passes" "$(v "$T/live")" pass
ck "pass names the last banner line" "$(sh "$H/banner-loop-check.sh" "$T/live" | grep -c 'last banner: WALLEYE-BANNER beat=3 ')" 1
ck "pass shows the commands run and the last one" "$(sh "$H/banner-loop-check.sh" "$T/live" | head -n1 | grep -c 'cmds=3 last=\[echo hi\] rc=0')" 1
ck "a loop never opened fails" "$(v "$T/notstarted")" fail
ck "the z5 banner (a shell= field, no loop=) fails" "$(v "$T/z5")" fail
ck "a loop that reopens every beat fails" "$(v "$T/flap")" fail
ck "an empty capture fails" "$(v "$T/empty")" fail
ck "no capture file skips (exit 2)" "$(sh "$H/banner-loop-check.sh" "$T/none" >/dev/null; echo $?)" 2
ck "pass exits 0, fail exits 1" "$(sh "$H/banner-loop-check.sh" "$T/live" >/dev/null; echo $?)$(sh "$H/banner-loop-check.sh" "$T/flap" >/dev/null; echo $?)" 01
[ $total -eq 0 ] && echo "banner-loop-selftest: all ok" || { echo "banner-loop-selftest: $total failed"; exit 1; }
