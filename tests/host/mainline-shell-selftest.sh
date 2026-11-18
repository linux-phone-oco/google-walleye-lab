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
# Permission denied: the node is denied for the first 3 s, then allowed (the uaccess ACL arriving late): the plain path retries and passes, no root-exec.
start_phone() { : > $FAKELOG; rm -f $T/tty; socat PTY,link=$T/tty,raw,echo=0 EXEC:"sh $T/phone.sh",pty,raw,echo=0 2>/dev/null & SP=$!
  n=0; while [ ! -e $T/tty ] && [ $n -lt 50 ]; do sleep 0.1; n=$((n+1)); done; PTYN=$(readlink -f $T/tty); }
kill $SP 2>/dev/null; sleep 0.3
start_phone; chmod 000 "$PTYN"; ( sleep 3; chmod 600 "$PTYN" ) &
TTY=$T/tty OUT=$T/out3 WAIT=2 CMD_TIMEOUT=3 ACCESS_WAIT=10 sh "$HERE/mainline-shell-read.sh" > $T/r3 2>&1; r=$?
ck "late ACL: retry passes" "$r" 0
ck "late ACL: plain path" "$(grep -c 'path used: plain' $T/r3)" 1
ck "late ACL: no root-exec log" "$([ -e $T/out3/root-exec.log ] && echo yes || echo no)" no
# Still denied: falls back to a fake `lab-agent root-exec` (it opens the node as root would), and logs each use.
mkdir -p $T/bin; cat > $T/bin/lab-agent <<B
#!/bin/sh
[ "\$1" = root-exec ] && [ "\$2" = -- ] || exit 2
shift 2; echo "\$*" >> $T/rx.calls; chmod 600 "$T/tty" 2>/dev/null; chmod 600 "\$(readlink -f $T/tty)"; exec "\$@"
B
chmod +x $T/bin/lab-agent
kill $SP 2>/dev/null; sleep 0.3; start_phone; chmod 000 "$PTYN"; : > $T/rx.calls
PATH=$T/bin:$PATH TTY=$T/tty OUT=$T/out4 WAIT=2 CMD_TIMEOUT=3 ACCESS_WAIT=1 sh "$HERE/mainline-shell-read.sh" > $T/r4 2>&1; r=$?
ck "denied: root-exec fallback passes" "$r" 0
ck "denied: says which path" "$(grep -c 'path used: root-exec' $T/r4)" 1
ck "denied: dmesg read through root" "$(grep -c 'Linux version' $T/out4/dmesg.txt)" 1
ck "denied: serial still redacted" "$(grep -c 'HT7ABCDEF123' $T/out4/cmdline.txt)" 0
ck "denied: stty raw through root-exec" "$(grep -c '^stty -F .* raw -echo -hupcl' $T/rx.calls)" 1
ck "denied: every root use logged" "$(grep -c 'root-exec:' $T/out4/root-exec.log)" "$(( $(wc -l < $T/rx.calls) ))"
sleep 0.5; ck "denied: reboot -f reached the fake via root" "$(grep -c 'reboot -f' $FAKELOG)" 1
ck "denied: only stty and sh used as root" "$(grep -vc '^stty \|^timeout .* sh -c\|^sh -c' $T/rx.calls)" 0
# Denied and no lab-agent at all: a fail that says what it saw.
kill $SP 2>/dev/null; sleep 0.3; start_phone; chmod 000 "$PTYN"
mkdir -p $T/nobin; for c in sh ls readlink getfacl sed grep date mkdir chmod sleep stty tr head mv wc timeout cat rm id; do ln -sf "$(command -v $c)" $T/nobin/$c; done
PATH=$T/nobin TTY=$T/tty OUT=$T/out5 WAIT=2 ACCESS_WAIT=1 sh "$HERE/mainline-shell-read.sh" > $T/r5 2>&1; r=$?
ck "denied, no lab-agent: fails" "$r" 1
ck "denied, no lab-agent: says what it saw" "$(grep -c 'not in PATH.*getfacl' $T/r5)" 1
echo "$([ $fail = 0 ] && echo pass || echo fail) walleye-mainline-shell-selftest host $pass ok, $fail failed"; [ $fail = 0 ]
