#!/bin/sh
# pmic-wdt-pon-regs-banner, run on the lab host against z10 (init-z10: the init reads the PON registers itself): READ the banner's WALLEYE-BANNER-PON line and ask the command `pon-regs`; arms nothing, writes nothing to the phone beyond /tmp.
# Answers go under OUT (default $HOME/lab-material/walleye/pon-regs-DATE, never in git), serials redacted.
# Environment: TTY (default /dev/ttyACM0), WAIT (s for the tty, 60), BEATS (s to listen for the banner line, 40), OUT, NOREBOOT=1 keeps the phone up (the init reboots itself at 180 s anyway).
# Prints `pass|fail pmic-wdt-pon-regs-banner host DETAIL`; exit 0 only on pass: the banner line has all seven values (no ??, no `none`) AND the command's answer equals them.
TTY=${TTY:-/dev/ttyACM0}; WAIT=${WAIT:-60}; BEATS=${BEATS:-40}
OUT=${OUT:-$HOME/lab-material/walleye/pon-regs-$(date +%Y%m%d-%H%M%S)}
trap "" HUP
say() { echo "$1 pmic-wdt-pon-regs-banner host $2"; [ "$1" = pass ]; exit $?; }
n=0; while [ ! -c "$TTY" ]; do [ "$n" -ge "$WAIT" ] && say fail "$TTY did not appear in ${WAIT}s"; sleep 1; n=$((n+1)); done
n=0; while [ ! -r "$TTY" ] || [ ! -w "$TTY" ]; do [ "$n" -ge 40 ] && say fail "$TTY not accessible to this user"; sleep 0.5; n=$((n+1)); done
mkdir -p "$OUT" && chmod 700 "$OUT" || say fail "cannot make $OUT"
command exec 3<>"$TTY" || say fail "cannot open $TTY"
timeout 5 stty raw -echo -hupcl 115200 <&3 2>/dev/null || true
redact() { sed -E -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/(androidboot\.serialno=)[^ ]*/\1REDACTED/g' -e 's/(serial(no|number)?[ =:]+)[0-9A-Za-z]{6,}/\1REDACTED/Ig'; }
# 1. the banner line (the first one with all seven values), listening up to BEATS seconds
timeout "$BEATS" sh -c 'while IFS= read -r l <&3; do case $l in *WALLEYE-BANNER-PON*0xc5=*) printf "%s\n" "$l"; break;; esac; done' 2>/dev/null | tr -d '\r' | redact > "$OUT/banner-pon.txt"
[ -s "$OUT/banner-pon.txt" ] || say fail "no WALLEYE-BANNER-PON line with registers in ${BEATS}s ($OUT)"
B=$(grep -o 'pon-regs 0x54=[0-9a-fA-F][^]]*\]' "$OUT/banner-pon.txt" | grep -v '??' | head -n1)
[ -n "$B" ] || say fail "banner line has no pon-regs line ($OUT/banner-pon.txt)"
case $B in *'??'*) say fail "banner values not all read: $B";; esac
# 2. the command's answer, first line
M=wpr-$$
timeout 0.3 cat <&3 >/dev/null 2>&1 || true
printf '%s\n' "pon-regs; echo $M-E''ND" >&3
timeout 20 sh -c "while IFS= read -r l <&3; do l=\${l%\$(printf '\r')}; case \$l in *$M-END) break;; esac; printf '%s\n' \"\$l\"; done" 2>/dev/null | grep -v -e "$M" -e WALLEYE-BANNER | redact > "$OUT/command.txt"
C=$(grep '^pon-regs 0x54=[0-9a-fA-F]' "$OUT/command.txt" | grep -v '??' | head -n1)
[ -n "$C" ] || say fail "the command pon-regs gave no answer ($OUT/command.txt)"
[ "${NOREBOOT:-0}" = 1 ] || printf '%s\n' 'reboot -f' >&3
[ "$B" = "$C" ] || say fail "banner and command differ: banner [$B] command [$C]"
say pass "$B (nothing armed; out $OUT)"
