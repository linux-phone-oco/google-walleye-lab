#!/bin/sh
# walleye-mainline-shell-read, run on the lab host: read the first shell of OUR mainline kernel over the phone's ttyACM (the init holds 180 s, touch /tmp/hold keeps it).
# Waits for the tty (TTY, default /dev/ttyACM0), opens it ONCE (fd 3, kept to the end: closing the port hangs up the phone's shell, backlog walleye-shell-reliable), then sends commands ONE AT A TIME, each followed by an end marker, with a read timeout per command, and saves each answer under OUT
# (default $HOME/lab-material/walleye/mainline-shell-DATE, never inside git): dmesg in full first, then uname -a, /proc/{cmdline,cpuinfo,iomem,interrupts,version}, the device lists
# (/sys/bus/platform/devices, power_supply-facts (type, online, voltage per supply), /sys/bus/spmi/devices, devices_deferred if debugfs, /sys/class/{udc,power_supply,drm,input,regulator}); `touch /tmp/hold` runs right after the tty is up, `reboot -f` at the end
# unless NOREBOOT=1. Nothing is written to the phone beyond /tmp. IDs (serial in the cmdline, MAC-shaped words) are redacted before saving.
# Environment: TTY, OUT, WAIT (seconds to wait for the tty, default 60), CMD_TIMEOUT (per command, default 15; dmesg gets 3x), ACCESS_WAIT (seconds to retry a node that says Permission denied, default 20, steps of 0.5 s: the uaccess ACL arrives a moment after the node).
# When the open stays Permission denied after the retry, the reader fails and says what it saw (owner, group, ACL, the usb parent's serial, who holds the port); it never uses root (decision 2026-11-18-walleye-banner-serial-not-root-exec). No chmod, chown or usermod.
# Prints `pass|fail walleye-mainline-shell-read host DETAIL`; exit 0 only on pass (dmesg non-empty and contains "Linux version").
TTY=${TTY:-/dev/ttyACM0}; WAIT=${WAIT:-60}; CT=${CMD_TIMEOUT:-15}
OUT=${OUT:-$HOME/lab-material/walleye/mainline-shell-$(date +%Y%m%d-%H%M%S)}
trap "" HUP   # the phone dropping off USB at reboot -f must not kill the reader holding the port
say() { echo "$1 walleye-mainline-shell-read host $2"; [ "$1" = pass ]; exit $?; }
n=0; while [ ! -c "$TTY" ] && [ ! -p "$TTY" ]; do
  [ "$n" -ge "$WAIT" ] && say fail "$TTY did not appear in ${WAIT}s"; sleep 1; n=$((n+1)); done
AW=${ACCESS_WAIT:-20}; PATHUSED=plain; STTY="raw -echo -hupcl 115200"
# can_open only asks the kernel for access (test -r/-w): a probe that opened and closed the node would hang the phone's shell up.
OPENERR=; can_open() { [ -r "$TTY" ] && [ -w "$TTY" ]; }
n=0; while ! can_open; do   # the node appears root:uucp first, logind adds the uaccess ACL a moment later: retry, 0.5 s steps
  [ "$n" -ge $((AW*2)) ] && break; sleep 0.5; n=$((n+1)); done
mkdir -p "$OUT" || say fail "cannot make $OUT"; chmod 700 "$OUT"
# the one open of the port, kept as fd 3 until the end (nothing below reopens or closes it before then); `command exec` so a failed open does not end the shell
if can_open && command exec 3<>"$TTY"; then
  timeout 5 stty $STTY <&3 2>/dev/null || true
  [ "$n" -gt 0 ] && echo "walleye-mainline-shell-read: node opened after $((n/2)).$((n%2*5)) s of Permission denied"
else
  OPENERR=$(timeout 3 sh -c 'exec 3<>"$1"' sh "$TTY" 2>&1)
  case $OPENERR in *"Permission denied"*) ;; *) say fail "$TTY cannot be opened and it is not a permission problem: ${OPENERR:-no error text}";; esac
  d=$(readlink -f "$TTY" 2>/dev/null || echo "$TTY"); tn=${d##*/}
  own=$(ls -l "$d" 2>&1 | tr '\n' ' '); owner=$(stat -c '%U' "$d" 2>/dev/null || echo '?'); group=$(stat -c '%G' "$d" 2>/dev/null || echo '?')
  acl=$(getfacl -p "$d" 2>&1 | tr '\n' ' ')
  us=$(cat "/sys/class/tty/$tn/device/../serial" 2>/dev/null || echo 'unknown')
  if command -v fuser >/dev/null 2>&1; then hb=$(fuser "$d" 2>&1 | tr "\n" " " | head -c 200); elif command -v lsof >/dev/null 2>&1; then hb=$(lsof "$d" 2>&1 | tr "\n" " " | head -c 200); else hb=unknown; fi; [ -n "$hb" ] || hb="none visible to this user"
  say fail "$TTY: $OPENERR (after ${AW}s of retries; no root, decision 2026-11-18-walleye-banner-serial-not-root-exec): $own owner=$owner group=$group acl=$acl usb-serial=$us held-by=$hb"
fi
echo "walleye-mainline-shell-read: path used: $PATHUSED"
redact() { sed -E -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/(androidboot\.serialno=)[^ ]*/\1REDACTED/g' -e 's/([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}/XX:XX:XX:XX:XX:XX/g' -e 's/(serial(no|number)?[ =:]+)[0-9A-Za-z]{6,}/\1REDACTED/Ig'; }
M=wmsr-$$; i=0; got=0
ask() { # ask NAME TIMEOUT COMMAND: send, read to the marker, save redacted
  i=$((i+1)); f="$OUT/$1.txt"
  timeout 0.3 cat <&3 >/dev/null 2>&1 || true   # whatever a slow earlier answer left in the pipe is not this answer
  printf '%s\n' "$3; echo $M-$i-E''ND" >&3      # fd 3 is the one open of the port: written to and read from, never reopened
  S="while IFS= read -r l <&3; do l=\${l%\$(printf '\r')}; case \$l in *$M-$i-END) break;; esac; printf '%s\n' \"\$l\"; done"
  timeout "$2" sh -c "$S" 2>/dev/null | grep -v -e "$M-$i" | redact > "$f.tmp"
  # the echoed command line is dropped by grep; an empty answer is kept as an empty file
  mv "$f.tmp" "$f"; [ -s "$f" ] && got=$((got+1))
}
twrite() { printf '%s\n' "$1" >&3 2>/dev/null; }
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
