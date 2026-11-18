#!/bin/sh
# z5-console-selftest (backlog walleye-banner-shell): runs init-z5's console block and its banner writer's shell-state part on the host under busybox ash, never touching the phone.
#   sh tests/host/z5-console-selftest.sh [BUSYBOX_X86_64]   (default ~/Projects/walleye-build/busybox-x86_64)
# Case A: the tty is /dev/null (a character device where stty fails and the shell reads end-of-file): the stderr of the start lands in sh.err, the shell is respawned, rc and probe are kept.
# Case B: the tty is a real pty (util-linux `script`; skipped, not failed, when absent): the shell stays alive, the tty is raw/no echo/no hangup, the banner's alive=yes.
# The console loops end themselves (kill 0 in their own process group), so no looping process outlives a run.
# The host's own coreutils stand for the target's busybox applets (the build checks the target's list); only `busybox` itself is this shim.
set -u
H=$(cd "$(dirname "$0")" && pwd)
INIT=$H/../../build/initramfs-mainline/init-z5
BBX=${1:-$HOME/Projects/walleye-build/busybox-x86_64}
[ -x "$BBX" ] && [ -f "$INIT" ] || { echo "skip: no $BBX or no $INIT (not known yet)"; exit 2; }
T=$(mktemp -d); trap 'kill $(jobs -p) 2>/dev/null; rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
mkdir "$T/bin"; ln -s "$BBX" "$T/bin/busybox"
sed -n '/^# the console/,/^) &$/p' "$INIT" | sed 's#/dev/kmsg#/dev/null#g' > "$T/console.sh"
ck "the console block was found in init-z5" "$([ "$(wc -l < "$T/console.sh")" -gt 10 ] && echo yes || echo no)" yes
# the banner writer's sourcing part, as the init has it, run against a state file
sed -n '/^      S=no; N=0/,/^      \[ -f \$SH.state \] || E=/p' "$INIT" | sed 's/^      //' > "$T/bannerpart.sh"
ck "the banner writer's shell-state part was found in init-z5" "$([ "$(wc -l < "$T/bannerpart.sh")" -eq 5 ] && echo yes || echo no)" yes
banner() { # SH_PREFIX -> "S N P A RC PR E"
  ( SH=$1; TTY=x; . "$T/bannerpart.sh"; echo "S=$S N=$N P=$P A=$A RC=$RC PR=$PR E=$E TTYSUM=$TTYSUM" )
}
ck "no state file: started=no, err says the console block has not run" "$(banner "$T/none")" "S=no N=0 P=- A=no RC=- PR=- E=console block not run yet TTYSUM=unknown"
# case A
printf '#!/bin/sh\nTTY=/dev/null; SH=%s/a; PATH=%s/bin:$PATH\n. %s/console.sh\nsleep 4\nkill 0\n' "$T" "$T" "$T" > "$T/runA.sh"
setsid timeout 6 sh "$T/runA.sh" >/dev/null 2>&1
ck "A: state file written" "$([ -f "$T/a.state" ] && echo yes || echo no)" yes
. "$T/a.state" 2>/dev/null
ck "A: shell started (S=yes)" "$S" yes
ck "A: the shell was respawned (N >= 2 in 5 s)" "$([ "${N:-0}" -ge 2 ] && echo yes || echo no)" yes
ck "A: the shell's exit status is kept (0 at end-of-file)" "$RC" 0
ck "A: the probe start's status is kept (0)" "$PR" 0
ck "A: the failed stty of /dev/null is the first line of sh.err" "$(head -n1 "$T/a.err" | grep -c 'stty')" 1
ck "A: the banner shows the dead shell and the first error" "$(banner "$T/a" | sed 's/ P=[0-9]* / P=PID /; s/ N=[0-9]* / N=N /; s/ E=.*TTYSUM/ E=ERR TTYSUM/')" "S=yes N=N P=PID A=no RC=0 PR=0 E=ERR TTYSUM=echo=?,icanon=?,hupcl=?"
# case B
if command -v script >/dev/null 2>&1; then
  printf '#!/bin/sh\nTTY=$(tty); SH=%s/b; PATH=%s/bin:$PATH\n. %s/console.sh\nsleep 5\nkill 0\n' "$T" "$T" "$T" > "$T/runB.sh"
  (sleep 6 | timeout 7 script -qec "sh $T/runB.sh" /dev/null > /dev/null 2>&1) &   # stdin held open: an end-of-file there would end the shell
  sleep 3
  if [ -f "$T/b.state" ]; then
    S=; N=; P=; RC=; PR=; TTYSUM=; . "$T/b.state"
    ck "B: shell started once, still running (S=yes N=1 RC=-)" "$S $N $RC" "yes 1 -"
    ck "B: the shell pid is alive" "$(kill -0 "$P" 2>/dev/null && echo yes || echo no)" yes
    ck "B: tty raw, no echo, no hangup" "$TTYSUM" "echo=off,icanon=off,hupcl=off"
    ck "B: probe start status 0" "$PR" 0
    ck "B: the banner says alive=yes" "$(banner "$T/b" | sed -n 's/.* A=\([a-z]*\) .*/\1/p')" yes
  else
    ck "B: state file written under a pty" no yes
  fi
  wait
else
  echo "skip B: no util-linux script (a pty case, not known yet)"
fi
[ $total -eq 0 ] && echo "z5-console-selftest: all ok" || { echo "z5-console-selftest: $total failed"; exit 1; }
