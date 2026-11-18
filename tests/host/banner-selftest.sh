#!/bin/sh
# banner-selftest: banner-check.sh against fake captures; never touches the phone.
H=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
printf 'WALLEYE-BANNER beat=0 up=19.1s steps=9 last=[step 9 banner writer started]\r\nWALLEYE-BANNER beat=1 up=21.1s steps=9 last=[x]\r\nWALLEYE-BANNER beat=2 up=23.1s steps=9 last=[x]\r\n' > "$T/good"
: > "$T/empty"
printf 'WALLEYE-BANNER beat=0 up=1s steps=1 last=[x]\nWALLEYE-BANNER beat=1 up=3s steps=1 last=[x]\n' > "$T/two"
printf 'WALLEYE-BANNER beat=3 up=1s steps=1 last=[x]\nWALLEYE-BANNER beat=1 up=3s steps=1 last=[x]\nWALLEYE-BANNER beat=2 up=5s steps=1 last=[x]\n' > "$T/back"
printf '\000\377garbage\n' > "$T/junk"
ck "good capture passes" "$(sh "$H/banner-check.sh" "$T/good" | cut -d: -f1)" pass
ck "empty capture fails" "$(sh "$H/banner-check.sh" "$T/empty" | cut -d: -f1)" fail
ck "two lines fail" "$(sh "$H/banner-check.sh" "$T/two" | cut -d: -f1)" fail
ck "beats going back fail" "$(sh "$H/banner-check.sh" "$T/back" | cut -d: -f1)" fail
ck "junk fails" "$(sh "$H/banner-check.sh" "$T/junk" | cut -d: -f1)" fail
ck "no file skips (exit 2)" "$(sh "$H/banner-check.sh" "$T/none" >/dev/null; echo $?)" 2
ck "pass exits 0, fail exits 1" "$(sh "$H/banner-check.sh" "$T/good" >/dev/null; echo $?)$(sh "$H/banner-check.sh" "$T/empty" >/dev/null; echo $?)" 01
[ $total -eq 0 ] && echo "banner-selftest: all ok" || { echo "banner-selftest: $total failed"; exit 1; }
