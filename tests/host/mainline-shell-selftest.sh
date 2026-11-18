#!/bin/sh
# walleye-mainline-shell-selftest, host only: a fake phone (a shell on a pty made by socat, with touch and reboot faked) answers mainline-shell-read.sh.
# Checks: a pass with the files saved; the serial and a MAC redacted; touch /tmp/hold and reboot -f reached the fake (not run on the host); a dead tty fails.
HERE=$(cd "$(dirname "$0")" && pwd)
[ "$(id -u)" != 0 ] || { echo "skip walleye-mainline-shell-selftest host (root: chmod 000 would not deny)"; exit 0; }
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
# Permission denied: the node is denied for the first 3 s, then allowed (the uaccess ACL arriving late): the plain path retries and passes.
start_phone() { : > $FAKELOG; rm -f $T/tty; socat PTY,link=$T/tty,raw,echo=0 EXEC:"sh $T/phone.sh",pty,raw,echo=0 2>/dev/null & SP=$!
  n=0; while [ ! -e $T/tty ] && [ $n -lt 50 ]; do sleep 0.1; n=$((n+1)); done; PTYN=$(readlink -f $T/tty); }
kill $SP 2>/dev/null; sleep 0.3
start_phone; chmod 000 "$PTYN"; ( sleep 3; chmod 600 "$PTYN" ) &
TTY=$T/tty OUT=$T/out3 WAIT=2 CMD_TIMEOUT=3 ACCESS_WAIT=10 sh "$HERE/mainline-shell-read.sh" > $T/r3 2>&1; r=$?
ck "late ACL: retry passes" "$r" 0
ck "late ACL: plain path" "$(grep -c 'path used: plain' $T/r3)" 1
# Still denied: the reader fails cleanly after the retry, says what it saw (owner, group, ACL, the usb parent's serial, who holds the port), and never calls lab-agent (a fake one logs any call).
mkdir -p $T/bin; cat > $T/bin/lab-agent <<B
#!/bin/sh
echo "\$*" >> $T/la.calls; exit 2
B
chmod +x $T/bin/lab-agent
kill $SP 2>/dev/null; sleep 0.3; start_phone; chmod 000 "$PTYN"; : > $T/la.calls
PATH=$T/bin:$PATH TTY=$T/tty OUT=$T/out4 WAIT=2 CMD_TIMEOUT=3 ACCESS_WAIT=1 sh "$HERE/mainline-shell-read.sh" > $T/r4 2>&1; r=$?
ck "denied: fails cleanly" "$r" 1
ck "denied: fail line names the port" "$(grep -c '^fail walleye-mainline-shell-read host .*Permission denied' $T/r4)" 1
ck "denied: says the owner and group" "$(grep -c 'owner=.* group=' $T/r4)" 1
ck "denied: says the ACL" "$(grep -c 'acl=' $T/r4)" 1
ck "denied: says the usb parent's serial" "$(grep -c 'usb-serial=' $T/r4)" 1
ck "denied: says who holds the port" "$(grep -c 'held-by=' $T/r4)" 1
ck "denied: lab-agent never called" "$(wc -l < $T/la.calls)" 0
ck "denied: nothing written to the phone" "$(grep -c 'reboot' $FAKELOG)" 0
ck "denied: no root-exec log" "$([ -e $T/out4/root-exec.log ] && echo yes || echo no)" no
echo "$([ $fail = 0 ] && echo pass || echo fail) walleye-mainline-shell-selftest host $pass ok, $fail failed"; [ $fail = 0 ]
