#!/bin/sh
# display-late-probe (items walleye-display-late-probe-image, walleye-try-display-late-probe), run on the lab host with the display late-probe image running on the phone:
# loads the display modules one at a time from the phone's shell, with the kernel log already streaming to this host, so a hang names its last step.
#   tests/host/display-late-probe.sh OUT_DIR
# Steps: wait for the shell (telnet on the phone's address); touch /tmp/hold (init-late then holds its self-reboot); a UDP listener on HOST_ADDR:6666 and a netconsole target made from the shell by configfs
# (printk loglevel 8); a TCP stream of /dev/kmsg from the phone (port 6667); each channel is proved by a marker line written to /dev/kmsg; then for every line of the phone's /display-modules.list, in its order:
# the step is written to OUT_DIR/steps.log HERE before the module is loaded, the load runs (msm and the panel with dyndbg=+p; drm.debug set once drm is in), the answer is written, a pause, and the shell is asked if it is alive.
# Writes OUT_DIR/{steps.log,netconsole.log,dmesg-stream.log,run.log,summary}. Prints `pass|fail display-late-probe host DETAIL`; exit 0 only on pass: every module loaded and the shell still answers.
# A fail names the step, the module and the last line each channel got - that line is the finding. It changes nothing on the phone but loading the display modules (no disk, no flash).
# Environment: PHONE_ADDR (172.16.43.1), HOST_ADDR (172.16.43.2, this host's side of the usb0 network), HOST_MAC (02:42:ac:10:2b:02, the host_addr init-late gives the ECM function),
# SHELL_WAIT seconds to wait for the shell (150), PAUSE seconds between steps (5), LOAD_WAIT seconds a load may take before it counts as no answer (20), SETTLE seconds to wait after the last step before the final check (30).
# STRIPES_SECONDS (20): when the phone has /bin/walleye-stripes (image L6), it is started after the settle and holds the four stripes that many seconds (0 leaves it out); its lines go to OUT_DIR/stripes.log and a nonzero exit fails the run.
set -u
ADDR=${PHONE_ADDR:-172.16.43.1}
HOSTIP=${HOST_ADDR:-172.16.43.2}
HMAC=${HOST_MAC:-02:42:ac:10:2b:02}
SHELL_WAIT=${SHELL_WAIT:-150}
PAUSE=${PAUSE:-5}
LOAD_WAIT=${LOAD_WAIT:-20}
SETTLE=${SETTLE:-30}
STRIPES_SECONDS=${STRIPES_SECONDS:-20}
OUT=${1:?usage: display-late-probe.sh OUT_DIR}
mkdir -p "$OUT" || exit 1
: > "$OUT/steps.log"; : > "$OUT/run.log"; : > "$OUT/netconsole.log"; : > "$OUT/dmesg-stream.log"
PIDS=
trap 'for p in $PIDS; do kill "$p" 2>/dev/null; done' EXIT
log() { echo "$*" | tee -a "$OUT/run.log" >&2; }
end() { echo "$1 display-late-probe host $2" | tee "$OUT/summary"; [ "$1" = pass ]; exit $?; }
# one command in a fresh telnet session; SECS is how long the session stays open for the answer; the shell's control characters are dropped
ph() { ( printf '%s\n' "$1"; sleep "$2"; echo exit ) | timeout "$(( $2 + 8 ))" telnet "$ADDR" 23 2>/dev/null | tr -d '\000-\010\013-\037\377'; }
last() { tail -n 1 "$1" 2>/dev/null | cut -c1-200; }
ip -4 -o addr show 2>/dev/null | grep -q "inet $HOSTIP/" || end fail "this host has no $HOSTIP (the phone's lease for the usb0 network)"
T0=$(date +%s)
until ph 'echo up-$((6*7))' 2 | grep -q 'up-42'; do
  n=$(( $(date +%s) - T0 )); [ "$n" -lt "$SHELL_WAIT" ] || end fail "no shell on $ADDR in $SHELL_WAIT s"
  sleep 1
done
n=$(( $(date +%s) - T0 ))
log "shell up after about $n s"
ph 'touch /tmp/hold; test -e /tmp/hold && echo hold-is-$((6*7))' 1 | grep -q 'hold-is-42' || end fail "could not touch /tmp/hold on the phone"
log "/tmp/hold set"
socat -u "UDP4-RECV:6666,bind=$HOSTIP" "OPEN:$OUT/netconsole.log,creat,append" 2>>"$OUT/run.log" &
PIDS="$PIDS $!"
N=/sys/kernel/config/netconsole/walleye
R=$(ph "echo 8 > /proc/sys/kernel/printk; mkdir $N && cd $N && echo usb0 > dev_name && echo $ADDR > local_ip && echo $HOSTIP > remote_ip && echo $HMAC > remote_mac && echo 6665 > local_port && echo 6666 > remote_port && echo 1 > enabled; echo nc-rc=\$?-end" 3)
log "netconsole target: $(printf '%s\n' "$R" | grep -o 'nc-rc=[0-9]*-end' | head -n1)"
ph "setsid nc -ll -s $ADDR -p 6667 -e cat /dev/kmsg </dev/null >/dev/null 2>&1 & echo stream-ok-\$((6*7))" 2 | grep -q stream-ok-42 || log "the TCP stream did not start"
n=0; until socat /dev/null "TCP:$ADDR:6667,connect-timeout=2" 2>/dev/null; do n=$((n + 1)); [ "$n" -lt 5 ] || { log "the TCP stream port 6667 does not answer"; break; }; sleep 1; done
socat -u "TCP:$ADDR:6667" "OPEN:$OUT/dmesg-stream.log,creat,append" 2>>"$OUT/run.log" &
PIDS="$PIDS $!"
MARK=late-probe-marker-$$
ph "echo $MARK > /dev/kmsg" 1 >/dev/null
n=0; NC=no; ST=no
while [ "$n" -lt 8 ]; do
  grep -q "$MARK" "$OUT/netconsole.log" 2>/dev/null && NC=yes
  grep -q "$MARK" "$OUT/dmesg-stream.log" 2>/dev/null && ST=yes
  [ $NC = yes ] && [ $ST = yes ] && break
  sleep 1; n=$((n + 1))
done
log "channels: netconsole (UDP 6666) carries the marker: $NC; the TCP dmesg stream (6667) carries it: $ST"
[ $NC = yes ] || [ $ST = yes ] || end fail "no log channel carries a marker (netconsole no, stream no): nothing loaded, a hang would name no step"
RAW=$(ph 'cat /display-modules.list; echo count-$(wc -l < /display-modules.list)' 2 | tr -d '\r')
LIST=$(printf '%s\n' "$RAW" | grep '^insmod /')
WANT=$(printf '%s\n' "$RAW" | sed -n 's/^count-\([0-9]*\)$/\1/p' | tail -n1)
[ -n "$LIST" ] || end fail "the phone has no /display-modules.list (netconsole $NC, stream $ST)"
LOST=
i=0
while IFS= read -r line; do
  i=$((i + 1))
  case $line in */msm.ko|*/panel-*) x=' dyndbg=+p' ;; *) x= ;; esac
  echo "step $i before: $line$x" >> "$OUT/steps.log"
  A=$(ph "$line$x; echo rc=\$?-step$i" "$LOAD_WAIT" | grep -o "rc=[0-9]*-step$i" | head -n1)
  echo "step $i after: ${A:-no answer}" >> "$OUT/steps.log"
  case $line in */drm.ko) ph 'echo 0x1ff > /sys/module/drm/parameters/debug' 1 >/dev/null ;; esac
  sleep "$PAUSE"
  if ph 'test -e /tmp/hold && echo alive-$((6*7))' 3 | grep -q 'alive-42'; then
    case ${A:-} in "rc=0-step$i") ;; *) LOST="step $i did not return 0 (${A:-no answer}) but the shell still answers: $line"; break ;; esac
  else
    LOST="the shell stopped answering after step $i: $line"; break
  fi
done <<EOL
$LIST
EOL
[ -n "$LOST" ] || [ "${WANT:-x}" = "$i" ] || LOST="the list on the phone has ${WANT:-an unreadable number of} lines, the script loaded $i"
# the binds of msm and the panel are asynchronous: settle, then ask again (the hold file is on a tmpfs, so a reset phone loses it)
[ -n "$LOST" ] || { sleep "$SETTLE"; ph 'test -e /tmp/hold && echo alive-$((6*7))' 3 | grep -q alive-42 || LOST="the shell stopped answering (or the phone reset) in the $SETTLE s after the last step"; }
if [ -n "$LOST" ]; then
  end fail "$LOST; last netconsole line: [$(last "$OUT/netconsole.log")]; last stream line: [$(last "$OUT/dmesg-stream.log")]; channels netconsole $NC stream $ST; see $OUT/steps.log"
fi
ph 'dmesg | tail -n 60' 3 > "$OUT/dmesg-tail.log"
DRAWN=
if [ "$STRIPES_SECONDS" -gt 0 ] && ph "test -x /bin/walleye-stripes && echo drawer-is-\$((6*7))" 2 | grep -q drawer-is-42; then
  # detached: a telnet session that ends early must not end the drawer; its log and exit code are read after the hold
  ph "rm -f /tmp/stripes.log; setsid sh -c '/bin/walleye-stripes --seconds $STRIPES_SECONDS > /tmp/stripes.log 2>&1; r=\$?; echo stripes-rc=\$r-end >> /tmp/stripes.log'</dev/null >/dev/null 2>&1 & echo started-\$((6*7))" 2 | grep -q started-42 || end fail "the stripe drawer did not start"
  sleep "$((STRIPES_SECONDS + 10))"
  ph 'cat /tmp/stripes.log' 3 | tr -d "\r" > "$OUT/stripes.log"
  log "the stripe drawer ran: $(grep -c walleye-stripes: "$OUT/stripes.log") lines in $OUT/stripes.log"
  grep -q "stripes-rc=0-end" "$OUT/stripes.log" || end fail "the stripe drawer did not end with 0; last line: [$(last "$OUT/stripes.log")]; see $OUT/stripes.log"
  DRAWN="; the stripes were drawn and held $STRIPES_SECONDS s"
fi
end pass "all $i modules loaded and the shell still answers; channels netconsole $NC stream $ST; last stream line: [$(last "$OUT/dmesg-stream.log")]$DRAWN"
