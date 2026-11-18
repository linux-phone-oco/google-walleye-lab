#!/bin/sh
# z6-loop-selftest (backlog walleye-command-loop): runs init-z6's command loop on the host under a real pty (util-linux `script`) and MEASURES "a line in gets an answer out" - never touching the phone.
#   sh tests/host/z6-loop-selftest.sh [BUSYBOX_X86_64]   (default ~/Projects/walleye-build/busybox-x86_64)
# The loop's block is cut out of init-z6 and run with TTY = the pty's slave; the pty's master side is `script`: its stdin carries the lines (the way the host reader writes them, each with its own end-marker echo), its stdout is what the host reader would read.
# The host's own coreutils stand for the target's busybox applets (the build checks the target's list). Skipped (exit 2, not failed) when `script` is absent: not known yet.
set -u
H=$(cd "$(dirname "$0")" && pwd)
INIT=$H/../../build/initramfs-mainline/init-z6
BBX=${1:-$HOME/Projects/walleye-build/busybox-x86_64}
[ -x "$BBX" ] && [ -f "$INIT" ] || { echo "skip: no $BBX or no $INIT (not known yet)"; exit 2; }
command -v script >/dev/null 2>&1 || { echo "skip: no util-linux script (a pty is needed; not known yet)"; exit 2; }
T=$(mktemp -d); trap 'kill $(jobs -p) 2>/dev/null; rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
sed -n '/^# the console, a command loop/,/^) &$/p' "$INIT" | sed 's#/dev/kmsg#/dev/null#g' > "$T/console.sh"
ck "the command loop block was found in init-z6" "$([ "$(wc -l < "$T/console.sh")" -gt 20 ] && echo yes || echo no)" yes
sed -n '/^      S=no; N=0; C=0/,/^      \[ -f \$SH.state \] || E=/p' "$INIT" | sed 's/^      //' > "$T/bannerpart.sh"
ck "the banner's loop-state part was found in init-z6 (4 lines)" "$(wc -l < "$T/bannerpart.sh" | tr -d ' ')" 4
banner() { ( SH=$1; TTY=x; . "$T/bannerpart.sh"; echo "S=$S N=$N C=$C LC=$LC LR=$LR E=$E" ); }
ck "no state file: not started, 0 commands, the err says the block has not run" "$(banner "$T/none")" "S=no N=0 C=0 LC=- LR=- E=console block not run yet"
mkdir "$T/bin"; ln -s "$BBX" "$T/bin/busybox"; for a in ash sh; do ln -sf "$BBX" "$T/bin/$a"; done
printf '#!%s/bin/ash\nPATH=%s/bin:$PATH; TTY=$(tty); SH=%s/b; . %s/console.sh\nsleep 14\nkill 0\n' "$T" "$T" "$T" "$T" > "$T/run.sh"
mkfifo "$T/in"
timeout 20 script -qec "$T/bin/ash $T/run.sh" /dev/null < "$T/in" > "$T/out.raw" 2>/dev/null &
exec 5> "$T/in"   # held open: an end-of-file on the pty's input would end the session
send() { printf '%s\n' "$1" >&5; sleep 1; }
sleep 2
ck "the state file says started=yes after the open" "$(sed -n 's/^S=\([a-z]*\);.*/\1/p' "$T/b.state" 2>/dev/null)" yes
ck "the tty is raw, no echo, no hangup" "$(sed -n "s/.*TTYSUM=//p" "$T/b.state" 2>/dev/null)" "echo=off,icanon=off,hupcl=off"
send 'echo hello; echo M-1-END'
send 'echo oops >&2; echo M-2-END'
printf 'echo crok; echo M-3-END\r\n' >&5; sleep 1
send ''
send 'cat; echo M-4-END'
send "echo 'a\"b'; echo M-5-END"
send 'sleep 1; echo slow; echo M-6-END'
sleep 1
send 'exit 7'
tr -d '\r' < "$T/out.raw" > "$T/out"
ck "a line in gets its answer out (hello)" "$(grep -cx hello "$T/out")" 1
ck "...and then its end marker, after the answer" "$(sed -n '/^hello$/,/^M-1-END$/p' "$T/out" | tr '\n' ' ')" "hello M-1-END "
ck "stderr of the command is in the answer" "$(grep -cx oops "$T/out")" 1
ck "a line ending in CR LF is run without the CR (crok, then its marker)" "$(sed -n '/^crok$/,/^M-3-END$/p' "$T/out" | tr '\n' ' ')" "crok M-3-END "
ck "a command reading stdin ends at once (stdin is /dev/null): its marker comes" "$(grep -cx M-4-END "$T/out")" 1
ck "a quote in the command line is answered" "$(grep -cx 'a"b' "$T/out")" 1
ck "a slow command's answer and marker come after it ends" "$(sed -n '/^slow$/,/^M-6-END$/p' "$T/out" | tr '\n' ' ')" "slow M-6-END "
ck "nothing of the commands is echoed by the tty (each marker once, not twice)" "$(grep -c 'M-[0-9]-END' "$T/out")" 6
ck "answers come in the order sent" "$(grep -o '^M-[0-9]-END$' "$T/out" | tr '\n' ' ')" "M-1-END M-2-END M-3-END M-4-END M-5-END M-6-END "
. "$T/b.state" 2>/dev/null
ck "the state counts 7 commands (the empty line is not one)" "$C" 7
ck "the state keeps the last exit status (exit 7)" "$LR" 7
ck "the state keeps the last command" "$LC" "exit 7"
ck "the state file is sourceable after a command with quotes (it was: this ran)" "$S" yes
ck "the banner part reads the state: commands 7, last exit 7" "$(banner "$T/b" | sed 's/ N=[0-9]* / N=N /')" "S=yes N=N C=7 LC=exit 7 LR=7 E=none"
[ $total -eq 0 ] && echo "z6-loop-selftest: all ok" || { echo "z6-loop-selftest: $total failed"; exit 1; }
