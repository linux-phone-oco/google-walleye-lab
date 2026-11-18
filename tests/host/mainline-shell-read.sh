#!/bin/sh
# walleye-mainline-shell-read, run on the lab host: read the first shell of OUR mainline kernel over the phone's ttyACM (the init holds 180 s, touch /tmp/hold keeps it).
# Waits for the tty (TTY, default /dev/ttyACM0), then sends commands ONE AT A TIME, each followed by an end marker, with a read timeout per command, and saves each answer under OUT
# (default $HOME/lab-material/walleye/mainline-shell-DATE, never inside git): dmesg in full first, then uname -a, /proc/{cmdline,cpuinfo,iomem,interrupts,version}, the device lists
# (/sys/bus/platform/devices, /sys/bus/spmi/devices, devices_deferred if debugfs, /sys/class/{udc,power_supply,drm,input,regulator}); `touch /tmp/hold` runs right after the tty is up, `reboot -f` at the end
# unless NOREBOOT=1. Nothing is written to the phone beyond /tmp. IDs (serial in the cmdline, MAC-shaped words) are redacted before saving.
# Environment: TTY, OUT, WAIT (seconds to wait for the tty, default 60), CMD_TIMEOUT (per command, default 15; dmesg gets 3x).
# Prints `pass|fail walleye-mainline-shell-read host DETAIL`; exit 0 only on pass (dmesg non-empty and contains "Linux version").
TTY=${TTY:-/dev/ttyACM0}; WAIT=${WAIT:-60}; CT=${CMD_TIMEOUT:-15}
OUT=${OUT:-$HOME/lab-material/walleye/mainline-shell-$(date +%Y%m%d-%H%M%S)}
say() { echo "$1 walleye-mainline-shell-read host $2"; [ "$1" = pass ]; exit $?; }
n=0; while [ ! -c "$TTY" ] && [ ! -p "$TTY" ]; do
  [ "$n" -ge "$WAIT" ] && say fail "$TTY did not appear in ${WAIT}s"; sleep 1; n=$((n+1)); done
n=0; while [ ! -w "$TTY" ]; do
  [ "$n" -ge 5 ] && say fail "$TTY not writable by this user"; sleep 1; n=$((n+1)); done
stty -F "$TTY" raw -echo 115200 2>/dev/null || true   # a fake tty (pty) may refuse nothing; a pipe does
mkdir -p "$OUT" || say fail "cannot make $OUT"; chmod 700 "$OUT"
redact() { sed -E -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/(androidboot\.serialno=)[^ ]*/\1REDACTED/g' -e 's/([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}/XX:XX:XX:XX:XX:XX/g' -e 's/(serial(no|number)?[ =:]+)[0-9A-Za-z]{6,}/\1REDACTED/Ig'; }
M=wmsr-$$; i=0; got=0
ask() { # ask NAME TIMEOUT COMMAND: send, read to the marker, save redacted
  i=$((i+1)); f="$OUT/$1.txt"
  timeout "$2" sh -c "exec 3<>'$TTY'; printf '%s\n' \"\$1; echo $M-$i-E''ND\" >&3; while IFS= read -r l <&3; do l=\${l%\$(printf '\r')}; case \$l in *$M-$i-END) break;; esac; printf '%s\n' \"\$l\"; done" sh "$3" 2>/dev/null | grep -v -e "$M-$i" | redact > "$f.tmp"
  r=$?
  # the echoed command line is dropped by grep; an empty answer is kept as an empty file
  mv "$f.tmp" "$f"; [ -s "$f" ] && got=$((got+1))
}
printf '\n' > "$TTY" 2>/dev/null
ask hold 5 'touch /tmp/hold'
ask dmesg $((CT*3)) 'dmesg'
ask uname $CT 'uname -a'
for p in cmdline cpuinfo iomem interrupts version; do ask $p $CT "cat /proc/$p"; done
ask platform-devices $CT 'ls /sys/bus/platform/devices'
ask devices-deferred $CT 'cat /sys/kernel/debug/devices_deferred 2>&1 || mount -t debugfs none /sys/kernel/debug 2>&1 && cat /sys/kernel/debug/devices_deferred 2>&1'
ask spmi-devices $CT 'ls /sys/bus/spmi/devices 2>&1'
for c in udc power_supply drm input regulator; do ask class-$c $CT "ls /sys/class/$c 2>&1"; done
[ "${NOREBOOT:-0}" = 1 ] || { printf 'reboot -f\n' > "$TTY" 2>/dev/null; }
grep -q 'Linux version' "$OUT/dmesg.txt" 2>/dev/null || say fail "dmesg has no 'Linux version' line (see $OUT; $got answers non-empty)"
say pass "$got answers saved under $OUT; dmesg $(wc -l < "$OUT/dmesg.txt") lines"
