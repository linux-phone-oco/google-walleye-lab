#!/bin/sh
# pmic-wdt-s2-regs-read, run on the lab host against z9 (PMIC watchdog child present, command loop on ttyACM): READ the pm8998 PON watchdog registers, arm nothing.
# Mounts debugfs, lists the spmi regmaps, and saves the PON registers 0x854-0x857, 0x85a, 0x85b, 0x8c5 (PON base 0x800; 0xc5 is the power-off reason) of SID 0 with the boot slot (cmdline).
# Answers go under OUT (default $HOME/lab-material/walleye/pmic-s2-regs-DATE, never in git), serials redacted. Nothing is written to the phone beyond /tmp and a debugfs mount.
# Environment: TTY (default /dev/ttyACM0), WAIT (s for the tty, 60), OUT, NOREBOOT=1 keeps the phone up.
# Prints `pass|fail pmic-wdt-s2-regs-read host DETAIL`; exit 0 only on pass (all seven registers read).
TTY=${TTY:-/dev/ttyACM0}; WAIT=${WAIT:-60}
OUT=${OUT:-$HOME/lab-material/walleye/pmic-s2-regs-$(date +%Y%m%d-%H%M%S)}
trap "" HUP
say() { echo "$1 pmic-wdt-s2-regs-read host $2"; [ "$1" = pass ]; exit $?; }
n=0; while [ ! -c "$TTY" ]; do [ "$n" -ge "$WAIT" ] && say fail "$TTY did not appear in ${WAIT}s"; sleep 1; n=$((n+1)); done
n=0; while [ ! -r "$TTY" ] || [ ! -w "$TTY" ]; do [ "$n" -ge 40 ] && say fail "$TTY not accessible to this user"; sleep 0.5; n=$((n+1)); done
mkdir -p "$OUT" && chmod 700 "$OUT" || say fail "cannot make $OUT"
command exec 3<>"$TTY" || say fail "cannot open $TTY"
timeout 5 stty raw -echo -hupcl 115200 <&3 2>/dev/null || true
redact() { sed -E -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/(androidboot\.serialno=)[^ ]*/\1REDACTED/g' -e 's/(serial(no|number)?[ =:]+)[0-9A-Za-z]{6,}/\1REDACTED/Ig'; }
M=wpsr-$$; i=0
ask() { i=$((i+1)); f="$OUT/$1.txt"
  timeout 0.3 cat <&3 >/dev/null 2>&1 || true
  printf '%s\n' "$3; echo $M-$i-E''ND" >&3
  S="while IFS= read -r l <&3; do l=\${l%\$(printf '\r')}; case \$l in *$M-$i-END) break;; esac; printf '%s\n' \"\$l\"; done"
  timeout "$2" sh -c "$S" 2>/dev/null | grep -v -e "$M-$i" -e WALLEYE-BANNER | redact > "$f"; }
printf '\n' >&3
ask hold 5 'touch /tmp/hold'
ask uname 15 'uname -r'
ask cmdline 15 'cat /proc/cmdline'
ask mount 15 'mount -t debugfs none /sys/kernel/debug 2>&1; ls /sys/kernel/debug/regmap 2>&1'
ask regmaps 15 'for d in /sys/kernel/debug/regmap/*; do echo "$d name=$(cat $d/name 2>&1)"; done'
ask regs 30 'for d in /sys/kernel/debug/regmap/*; do [ "$(cat $d/name 2>/dev/null)" = pmic-spmi ] && echo "== ${d##*/}" && grep -i -E "^0*(854|855|856|857|85a|85b|8c5):" $d/registers 2>&1; done'
ask regs-raw 30 'head -c 600 /sys/kernel/debug/regmap/0-00/registers 2>&1; ls -l /sys/kernel/debug/regmap/0-00 2>&1'
ask pon-dmesg 15 'dmesg | grep -i -e pon -e wdt -e watchdog -e pm8998'
[ "${NOREBOOT:-0}" = 1 ] || printf '%s\n' 'reboot -f' >&3
for r in 854 855 856 857 85a 85b 8c5; do grep -qi -E "^0*$r:" "$OUT/regs.txt" || say fail "register 0x$r not read ($OUT/regs.txt, $OUT/regmaps.txt)"; done
say pass "7 PON registers read, nothing armed (out $OUT)"
