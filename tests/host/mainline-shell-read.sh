#!/bin/sh
# walleye-mainline-shell-read, run on the lab host: read the first shell of OUR mainline kernel over the phone's ttyACM (the init holds 180 s, touch /tmp/hold keeps it).
# Waits for the tty (TTY, default /dev/ttyACM0), then sends commands ONE AT A TIME, each followed by an end marker, with a read timeout per command, and saves each answer under OUT
# (default $HOME/lab-material/walleye/mainline-shell-DATE, never inside git): dmesg in full first, then uname -a, /proc/{cmdline,cpuinfo,iomem,interrupts,version}, the device lists
# (/sys/bus/platform/devices, power_supply-facts (type, online, voltage per supply), /sys/bus/spmi/devices, devices_deferred if debugfs, /sys/class/{udc,power_supply,drm,input,regulator}); `touch /tmp/hold` runs right after the tty is up, `reboot -f` at the end
# unless NOREBOOT=1. Nothing is written to the phone beyond /tmp. IDs (serial in the cmdline, MAC-shaped words) are redacted before saving.
# Environment: TTY, OUT, WAIT (seconds to wait for the tty, default 60), CMD_TIMEOUT (per command, default 15; dmesg gets 3x), ACCESS_WAIT (seconds to retry a node that says Permission denied, default 20, steps of 0.5 s: the uaccess ACL arrives a moment after the node).
# When the plain open stays denied, the reader falls back to `lab-agent root-exec -- ...` for that ONE node only (decision 2026-11-18-walleye-tty-read-through-root-exec): stty raw, reading, one command written at a time. It prints which path it used and writes every root use as a line in OUT/root-exec.log. No chmod, chown or usermod.
# Prints `pass|fail walleye-mainline-shell-read host DETAIL`; exit 0 only on pass (dmesg non-empty and contains "Linux version").
TTY=${TTY:-/dev/ttyACM0}; WAIT=${WAIT:-60}; CT=${CMD_TIMEOUT:-15}
OUT=${OUT:-$HOME/lab-material/walleye/mainline-shell-$(date +%Y%m%d-%H%M%S)}
say() { echo "$1 walleye-mainline-shell-read host $2"; [ "$1" = pass ]; exit $?; }
n=0; while [ ! -c "$TTY" ] && [ ! -p "$TTY" ]; do
  [ "$n" -ge "$WAIT" ] && say fail "$TTY did not appear in ${WAIT}s"; sleep 1; n=$((n+1)); done
AW=${ACCESS_WAIT:-20}; RX="lab-agent root-exec --"; PATHUSED=plain; STTY="raw -echo -hupcl 115200"
rxlog() { :; }
OPENERR=; can_open() { OPENERR=$(timeout 3 sh -c 'exec 3<>"$1"' sh "$TTY" 2>&1); }
n=0; while ! can_open; do   # the node appears root:uucp first, logind adds the uaccess ACL a moment later: retry, 0.5 s steps
  [ "$n" -ge $((AW*2)) ] && break; sleep 0.5; n=$((n+1)); done
mkdir -p "$OUT" || say fail "cannot make $OUT"; chmod 700 "$OUT"
if can_open; then
  timeout 5 stty -F "$TTY" $STTY 2>/dev/null || true
  [ "$n" -gt 0 ] && echo "walleye-mainline-shell-read: node opened after $((n/2)).$((n%2*5)) s of Permission denied"
  RX=; ROOTLOG=
else
  case $OPENERR in *"Permission denied"*) ;; *) say fail "$TTY cannot be opened and it is not a permission problem: $OPENERR";; esac
  case $TTY in *\"*|*\'*|*" "*) say fail "TTY has a quote or space: no root fallback";; esac
  RT=$(readlink -f "$TTY"); case $RT in /dev/ttyACM*|/dev/pts/*) TTY=$RT;; *) say fail "$TTY resolves to $RT: root fallback only for /dev/ttyACM*";; esac
  PATHUSED=root-exec; ROOTLOG=$OUT/root-exec.log
  diag="$(ls -l "$TTY" 2>&1 | tr '\n' ' '); getfacl: $(getfacl -p "$TTY" 2>&1 | tr '\n' ' '); udev rule 70-lpoco-google-walleye.rules: $(ls /etc/udev/rules.d/70-lpoco-google-walleye.rules /usr/lib/udev/rules.d/70-lpoco-google-walleye.rules 2>/dev/null | head -n1 | grep . || echo absent)"
  echo "walleye-mainline-shell-read: $TTY still denied after ${AW}s; $diag"
  command -v lab-agent >/dev/null 2>&1 || say fail "$TTY denied after ${AW}s and lab-agent is not in PATH for the root-exec fallback ($diag)"
  echo "walleye-mainline-shell-read: root-exec for $TTY only (logged in $ROOTLOG)"
  rxlog() { echo "$(date +%H:%M:%S) root-exec: $*" >> "$ROOTLOG"; }
  rxlog "stty -F $TTY $STTY"
  timeout 10 lab-agent root-exec -- stty -F "$TTY" $STTY 2>/dev/null || say fail "root-exec stty on $TTY failed ($diag)"
fi
echo "walleye-mainline-shell-read: path used: $PATHUSED"
redact() { sed -E -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/(androidboot\.serialno=)[^ ]*/\1REDACTED/g' -e 's/([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}/XX:XX:XX:XX:XX:XX/g' -e 's/(serial(no|number)?[ =:]+)[0-9A-Za-z]{6,}/\1REDACTED/Ig'; }
M=wmsr-$$; i=0; got=0
ask() { # ask NAME TIMEOUT COMMAND: send, read to the marker, save redacted
  i=$((i+1)); f="$OUT/$1.txt"
  S="exec 3<>\"\$2\"; printf '%s\n' \"\$1; echo $M-$i-E''ND\" >&3; while IFS= read -r l <&3; do l=\${l%\$(printf '\r')}; case \$l in *$M-$i-END) break;; esac; printf '%s\n' \"\$l\"; done"
  if [ -n "$RX" ]; then rxlog "sh -c (write one command, read to the marker) on $TTY: $1 [$3]"; timeout $(($2+5)) $RX timeout "$2" sh -c "$S" sh "$3" "$TTY" 2>/dev/null | grep -v -e "$M-$i" | redact > "$f.tmp"
  else timeout "$2" sh -c "$S" sh "$3" "$TTY" 2>/dev/null | grep -v -e "$M-$i" | redact > "$f.tmp"; fi
  r=$?
  # the echoed command line is dropped by grep; an empty answer is kept as an empty file
  mv "$f.tmp" "$f"; [ -s "$f" ] && got=$((got+1))
}
twrite() { if [ -n "$RX" ]; then rxlog "sh -c printf to $TTY: $1"; timeout 10 lab-agent root-exec -- sh -c 'printf "%s\n" "$1" > "$2"' sh "$1" "$TTY" 2>/dev/null; else printf '%s\n' "$1" > "$TTY" 2>/dev/null; fi; }
twrite ''
ask hold 5 'touch /tmp/hold'
ask dmesg $((CT*3)) 'dmesg'
ask uname $CT 'uname -a'
for p in cmdline cpuinfo iomem interrupts version; do ask $p $CT "cat /proc/$p"; done
ask platform-devices $CT 'ls /sys/bus/platform/devices'
ask devices-deferred $CT 'cat /sys/kernel/debug/devices_deferred 2>&1 || mount -t debugfs none /sys/kernel/debug 2>&1 && cat /sys/kernel/debug/devices_deferred 2>&1'
ask spmi-devices $CT 'ls /sys/bus/spmi/devices 2>&1'
for c in udc power_supply drm input regulator; do ask class-$c $CT "ls /sys/class/$c 2>&1"; done
ask power-supply-facts $CT 'for d in /sys/class/power_supply/*; do echo "== ${d##*/}"; for k in type online status health usb_type voltage_now current_now; do echo "$k=$(cat $d/$k 2>&1)"; done; done'
[ "${NOREBOOT:-0}" = 1 ] || twrite 'reboot -f'
grep -q 'Linux version' "$OUT/dmesg.txt" 2>/dev/null || say fail "dmesg has no 'Linux version' line (see $OUT; $got answers non-empty)"
say pass "path $PATHUSED; $got answers saved under $OUT; dmesg $(wc -l < "$OUT/dmesg.txt") lines"
