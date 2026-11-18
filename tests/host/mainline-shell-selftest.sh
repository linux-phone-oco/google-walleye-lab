#!/bin/sh
# walleye-mainline-shell-selftest, host only: a fake phone (a shell on a pty made by socat, with touch and reboot faked) answers mainline-shell-read.sh.
# Checks: a pass with the files saved; the serial and a MAC redacted; touch /tmp/hold and reboot -f reached the fake (not run on the host); a dead tty fails.
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); fail=0; pass=0
cleanup() { [ -n "$SP" ] && kill "$SP" 2>/dev/null; rm -rf "$T"; }; trap cleanup EXIT
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
cat > $T/phone.sh <<'P'
touch() { echo "$@" >> "$FAKELOG"; }
reboot() { echo "reboot $@" >> "$FAKELOG"; }
dmesg() { echo "[    0.000000] Linux version 6.x-walleye (lab) #1 SMP"; echo "[    1.0] wlan mac 02:aa:bb:cc:dd:ee"; }
uname() { echo "Linux walleye 6.x aarch64"; }
cat() { case $1 in /proc/cmdline) echo "console=ttyMSM0 androidboot.serialno=HT7ABCDEF123 clk_ignore_unused";; *) echo "fake $1";; esac; }
ls() { echo "fake-ls $1"; }
while IFS= read -r l; do eval "$l"; done
P
FAKELOG=$T/fake.log; export FAKELOG; : > $FAKELOG
socat PTY,link=$T/tty,raw,echo=0 EXEC:"sh $T/phone.sh",pty,raw,echo=0 2>/dev/null & SP=$!
n=0; while [ ! -e $T/tty ] && [ $n -lt 50 ]; do sleep 0.1; n=$((n+1)); done
TTY=$T/tty OUT=$T/out WAIT=2 CMD_TIMEOUT=3 sh "$HERE/mainline-shell-read.sh" > $T/r 2>&1; r=$?
ck "a fake phone passes" "$r" 0
ck "dmesg saved" "$(grep -c 'Linux version' $T/out/dmesg.txt)" 1
ck "cmdline serial redacted" "$(grep -c 'HT7ABCDEF123' $T/out/cmdline.txt)" 0
ck "cmdline kept otherwise" "$(grep -c clk_ignore_unused $T/out/cmdline.txt)" 1
ck "MAC in dmesg redacted" "$(grep -c '02:aa:bb' $T/out/dmesg.txt)" 0
ck "touch /tmp/hold reached the fake" "$(grep -c '/tmp/hold' $FAKELOG)" 1
sleep 0.5; ck "reboot -f reached the fake" "$(grep -c 'reboot -f' $FAKELOG)" 1
TTY=$T/none OUT=$T/out2 WAIT=1 sh "$HERE/mainline-shell-read.sh" > $T/r2 2>&1; r=$?
ck "no tty fails" "$r" 1
echo "$([ $fail = 0 ] && echo pass || echo fail) walleye-mainline-shell-selftest host $pass ok, $fail failed"; [ $fail = 0 ]
