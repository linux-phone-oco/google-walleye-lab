#!/bin/sh
# display-late-probe-selftest (item walleye-display-late-probe-image): proves on the host, never touching a phone, that display-late-probe.sh
#  (1) sets /tmp/hold before it loads anything and loads the modules in the phone's list order, one step at a time;
#  (2) writes each step to the host file BEFORE the load reaches the phone (the fake phone copies steps.log at the moment it receives the load);
#  (3) with a phone that dies at a load, fails naming that step and module, and gives the last line of each channel;
#  (4) with no shell, fails without loading anything; with no host address, fails before touching the phone.
#   sh tests/host/display-late-probe-selftest.sh [SCRIPT]     (default: display-late-probe.sh next to this file)
# The telnet, socat and ip it calls are fakes put first in PATH: the fake phone answers the script's commands from what each one names, a fake load that is to hang sleeps and then the whole phone is dead (it answers nothing).
# Prints `pass|fail display-late-probe-selftest host DETAIL`; exit 0 only on pass.
set -u
H=$(cd "$(dirname "$0")" && pwd)
S=${1:-$H/display-late-probe.sh}
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
[ -f "$S" ] || { echo "fail display-late-probe-selftest host no $S"; exit 1; }
mkdir "$T/bin"
cat > "$T/bin/telnet" <<'EOF'
#!/bin/sh
# the fake phone: reads the session's lines, answers by what they name. State in $FAKE/: dead (the phone is gone), noshell (never answers), calls (what it was asked, with steps.log as it was then)
IN=$(cat)
printf "%s\n" "$IN" > "$FAKE/echo"
[ -e "$FAKE/dead" ] && { sleep 3; exit 0; }
[ -e "$FAKE/noshell" ] && { sleep 1; exit 0; }
echo "CALL: $(printf '%s' "$IN" | head -n1 | cut -c1-90)" >> "$FAKE/calls"
case $IN in
  *"/display-modules.list"*) cat "$FAKE/list"; echo "count-$(wc -l < "$FAKE/list")";;
  *"touch /tmp/hold"*) printf "%s\n" "$IN"; [ -n "${FAKE_NOHOLD:-}" ] && exit 0; : > "$FAKE/held"; echo hold-is-42;;
  *"echo up-"*) echo up-42;;
  *"test -x /bin/walleye-stripes"*) [ -n "${FAKE_DRAWER:-}" ] && echo drawer-is-42;;
  *"setsid sh -c '/bin/walleye-stripes --seconds"*) echo "$IN" | grep -o -e "--seconds [0-9]*" >> "$FAKE/calls"; { echo "walleye-stripes: draw: four stripes"; echo "walleye-stripes: release: the screen is cleared to black"; echo "stripes-rc=${FAKE_DRAWER_RC:-0}-end"; } > "$FAKE/drawerlog"; echo started-42;;
  *"cat /tmp/stripes.log"*) k=$(( $(cat "$FAKE/reads" 2>/dev/null || echo 0) + 1 )); echo $k > "$FAKE/reads"; if [ "$k" -le "${FAKE_DRAWER_LATE:-0}" ]; then grep -v stripes-rc "$FAKE/drawerlog" 2>/dev/null; else cat "$FAKE/drawerlog" 2>/dev/null; fi;;
  *"echo alive-"*) printf "%s\n" "$IN"; [ -e "$FAKE/held" ] && echo alive-42;;
  *"stream-ok"*) printf "%s\n" "$IN"; echo stream-ok-42;;
  *"nc-rc"*) echo nc-rc=0-end;;
  *"late-probe-marker"*) m=$(printf '%s' "$IN" | grep -o 'late-probe-marker-[0-9]*' | head -n1); echo "$m" >> "$FAKE/out/netconsole.log"; echo "$m" >> "$FAKE/out/dmesg-stream.log";;
  *"insmod "*)
    line=$(printf '%s' "$IN" | head -n1); step=$(printf '%s' "$line" | grep -o 'step[0-9]*' | head -n1)
    { echo "load seen with held=$([ -e "$FAKE/held" ] && echo yes || echo no) and steps.log:"; cat "$FAKE/out/steps.log"; } >> "$FAKE/calls"
    case $line in
      *"${FAKE_HANG:-nomatch}"*) echo "[ 12.5] msm_dpu: probing" >> "$FAKE/out/netconsole.log"; echo "[ 12.5] msm_dpu: probing" >> "$FAKE/out/dmesg-stream.log"; : > "$FAKE/dead"; sleep 3;;
      *) echo "[ 9.0] loaded $line" >> "$FAKE/out/dmesg-stream.log"; echo "rc=0-$step";;
    esac;;
  *"dmesg"*) echo "[ 1.0] fake dmesg";;
  *) :;;
esac
EOF
printf '#!/bin/sh\nf=\nfor a; do case $a in OPEN:*) f=${a#OPEN:};; esac; done\nf=${f%%%%,*}\n[ -n "$f" ] || exit 0\n: >> "$f"\nsleep 60\n' > "$T/bin/socat"
printf '#!/bin/sh\necho "2: usb0    inet ${FAKE_HOSTIP:-172.16.43.2}/24 brd 172.16.43.255 scope global usb0"\n' > "$T/bin/ip"
chmod 755 "$T/bin/telnet" "$T/bin/socat" "$T/bin/ip"
printf 'insmod /lib/modules/k/kernel/drivers/gpu/drm/drm_kms_helper.ko\ninsmod /lib/modules/k/kernel/drivers/gpu/drm/drm.ko\ninsmod /lib/modules/k/kernel/drivers/gpu/drm/msm/msm.ko\ninsmod /lib/modules/k/kernel/drivers/gpu/drm/panel/panel-samsung-s6e3fa5.ko\n' > "$T/list"
run() { # NAME [ENV=VALUE...]: one run into $T/NAME; the fake phone's state in $T/NAME.fake
  n=$1; shift
  rm -rf "$T/$n.fake"; mkdir -p "$T/$n.fake/out"; cp "$T/list" "$T/$n.fake/list"
  env PATH="$T/bin:$PATH" FAKE="$T/$n.fake" PAUSE=0 SETTLE=0 LOAD_WAIT=1 SHELL_WAIT=4 "$@" sh "$S" "$T/$n.fake/out" > "$T/$n.stdout" 2> "$T/$n.stderr"
  echo $? > "$T/$n.rc"
}
# the fake phone writes into the script's OUT directory, which the fake knows as $FAKE/out
run ok
ck "a good phone: exit 0" "$(cat "$T/ok.rc")" 0
ck "a good phone: the summary line passes and says 4 modules" "$(sed -n 's/^\(pass display-late-probe host all 4 modules\).*/\1/p' "$T/ok.stdout")" "pass display-late-probe host all 4 modules"
ck "a good phone: each of 4 steps has a before and an after line" "$(grep -c '^step [0-9] \(before\|after: rc=0-step[0-9]\)' "$T/ok.fake/out/steps.log")" 8
ck "the loads were asked in the list's order" "$(grep -o 'step[0-9]*' "$T/ok.fake/out/steps.log" | head -n0; grep '^step [0-9] before' "$T/ok.fake/out/steps.log" | sed 's/.*\///; s/ dyndbg=+p//' | tr '\n' ' ')" "drm_kms_helper.ko drm.ko msm.ko panel-samsung-s6e3fa5.ko "
ck "/tmp/hold was set before the first load reached the phone" "$(grep -m1 '^load seen' "$T/ok.fake/calls" | sed 's/ and steps.*//')" "load seen with held=yes"
ck "step 3 was in steps.log before its load reached the phone" "$(awk '/^load seen/{n++} n==3 && /^step 3 before/{f=1} END{print f+0}' "$T/ok.fake/calls")" 1
ck "step 3 was NOT yet answered when its load reached the phone" "$(awk '/^load seen/{n++} n==3 && /^step 3 after/{f=1} END{print f+0}' "$T/ok.fake/calls")" 0
ck "msm and the panel are loaded with dyndbg, the others not" "$(grep -c 'dyndbg=+p' "$T/ok.fake/out/steps.log")/$(grep -c 'dyndbg' "$T/ok.fake/out/steps.log")" 2/2
ck "the run reports both channels carry the marker" "$(grep -c 'netconsole (UDP 6666) carries the marker: yes; the TCP dmesg stream (6667) carries it: yes' "$T/ok.stderr")" 1
run hang FAKE_HANG=msm.ko
ck "a phone that dies at msm.ko: exit 1" "$(cat "$T/hang.rc")" 1
ck "it fails naming step 3 and msm.ko" "$(sed -n 's/^fail display-late-probe host \(the shell stopped answering after step 3: insmod [^ ]*msm.ko\).*/\1/p' "$T/hang.stdout" | sed 's#insmod /.*/#insmod #')" "the shell stopped answering after step 3: insmod msm.ko"
ck "it gives the last netconsole line" "$(grep -c 'last netconsole line: \[\[ 12.5\] msm_dpu: probing\]' "$T/hang.stdout")" 1
ck "it gives the last stream line" "$(grep -c 'last stream line: \[\[ 12.5\] msm_dpu: probing\]' "$T/hang.stdout")" 1
ck "step 3 has a before line and no answer" "$(grep '^step 3' "$T/hang.fake/out/steps.log" | sed 's/before: .*/before/' | tr '\n' '|')" "step 3 before|step 3 after: no answer|"
ck "step 4 was never started" "$(grep -c '^step 4' "$T/hang.fake/out/steps.log")" 0
rm -rf "$T/ns.fake"; mkdir -p "$T/ns.fake/out"; : > "$T/ns.fake/noshell"
env PATH="$T/bin:$PATH" FAKE="$T/ns.fake" PAUSE=0 SETTLE=0 LOAD_WAIT=1 SHELL_WAIT=4 sh "$S" "$T/ns.fake/out" > "$T/ns.stdout" 2>/dev/null; echo $? > "$T/ns.rc"
ck "no shell: exit 1 and says so" "$(cat "$T/ns.rc") $(sed -n 's/^\(fail display-late-probe host no shell\).*/\1/p' "$T/ns.stdout")" "1 fail display-late-probe host no shell"
ck "no shell: nothing was loaded and /tmp/hold never set" "$(ls "$T/ns.fake/held" 2>/dev/null | wc -l) $(cat "$T/ns.fake/calls" 2>/dev/null | grep -c insmod)" "0 0"
rm -rf "$T/nh.fake"; mkdir -p "$T/nh.fake/out"; cp "$T/list" "$T/nh.fake/list"
env PATH="$T/bin:$PATH" FAKE="$T/nh.fake" FAKE_HOSTIP=10.0.0.9 PAUSE=0 SETTLE=0 LOAD_WAIT=1 SHELL_WAIT=4 sh "$S" "$T/nh.fake/out" > "$T/nh.stdout" 2>/dev/null; echo $? > "$T/nh.rc"
ck "no host address: exit 1, names the address, the phone untouched" "$(cat "$T/nh.rc") $(grep -c 'this host has no 172.16.43.2' "$T/nh.stdout") $(ls "$T/nh.fake/calls" 2>/dev/null | wc -l)" "1 1 0"
run nohold FAKE_NOHOLD=1
ck "a phone where touch /tmp/hold fails (the echoed command line still holds the word): exit 1, names the hold, nothing loaded" "$(cat "$T/nohold.rc") $(grep -c "could not touch /tmp/hold" "$T/nohold.stdout") $(grep -c insmod "$T/nohold.fake/calls")" "1 1 0"
run nodrawer
ck "a phone without /bin/walleye-stripes (L1 to L5): passes, the drawer is not asked for" "$(cat "$T/nodrawer.rc") $(grep -c "walleye-stripes --seconds" "$T/nodrawer.fake/calls") $(grep -c "stripes were drawn" "$T/nodrawer.stdout")" "0 0 0"
run drawer FAKE_DRAWER=1 STRIPES_SECONDS=7
ck "a phone with the drawer (L6): passes, asks it for the seconds, keeps its lines, says the stripes were drawn" "$(cat "$T/drawer.rc") $(grep -c -e "^--seconds 7$" "$T/drawer.fake/calls") $(grep -c "walleye-stripes: draw" "$T/drawer.fake/out/stripes.log") $(grep -c "stripes were drawn and held 7 s" "$T/drawer.stdout")" "0 1 1 1"
run drawerzero FAKE_DRAWER=1 STRIPES_SECONDS=0
ck "STRIPES_SECONDS=0 leaves the drawer out" "$(cat "$T/drawerzero.rc") $(grep -c "walleye-stripes --seconds" "$T/drawerzero.fake/calls")" "0 0"
run drawerlate FAKE_DRAWER=1 STRIPES_SECONDS=1 STRIPES_POLL=0 STRIPES_GRACE=20 FAKE_DRAWER_LATE=3
ck "a drawer whose rc line comes after the first reads (it needs time after the settle): the run polls and passes, after more than one read" "$(cat "$T/drawerlate.rc") $(grep -c "stripes were drawn and held 1 s" "$T/drawerlate.stdout") $([ "$(cat "$T/drawerlate.fake/reads")" -eq 4 ] && echo polled)" "0 1 polled"
run drawernever FAKE_DRAWER=1 STRIPES_SECONDS=1 STRIPES_POLL=1 STRIPES_GRACE=5 FAKE_DRAWER_LATE=99
ck "a drawer that never reports its rc: the poll is bounded, the run fails naming the log" "$(cat "$T/drawernever.rc") $(grep -c "the stripe drawer did not end with 0" "$T/drawernever.stdout") $([ "$(cat "$T/drawernever.fake/reads")" -ge 2 ] && echo polled)" "1 1 polled"
run drawerbad FAKE_DRAWER=1 FAKE_DRAWER_RC=1
ck "a drawer that ends with 1 fails the run and names its log" "$(cat "$T/drawerbad.rc") $(grep -c "the stripe drawer did not end with 0" "$T/drawerbad.stdout")" "1 1"
ck "the script is plain sh (sh -n)" "$(sh -n "$S" 2>&1 && echo ok)" ok
if [ $total -eq 0 ]; then echo "pass display-late-probe-selftest host every check passed"; else echo "fail display-late-probe-selftest host $total checks failed"; exit 1; fi
