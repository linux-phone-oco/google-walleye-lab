#!/bin/sh
# walleye-ufs-lsblk, run on the lab host against z11 (init-z11 lists the UFS itself): READ the banner's WALLEYE-BANNER-UFS line and ask the commands `ufs-lsblk` and `ufs-hdr boot_a`; nothing is written to the phone beyond /tmp, no disk is mounted, written or discarded.
# Answers go under OUT (default $HOME/lab-material/walleye/ufs-lsblk-DATE, never in git), serials redacted.
# Environment: TTY (default /dev/ttyACM0), WAIT (s for the tty, 60), BEATS (s to listen for the banner line, 50), OUT, NOREBOOT=1 keeps the phone up (the init reboots itself at 180 s anyway).
# Prints `pass|fail walleye-ufs-lsblk host DETAIL`; exit 0 only on pass: the banner line has luns>=1 and a partition named boot_a, AND the command's answer equals the banner's lines, AND ufs-hdr boot_a answers with magic ANDROID!.
TTY=${TTY:-/dev/ttyACM0}; WAIT=${WAIT:-60}; BEATS=${BEATS:-50}
OUT=${OUT:-$HOME/lab-material/walleye/ufs-lsblk-$(date +%Y%m%d-%H%M%S)}
trap "" HUP
say() { echo "$1 walleye-ufs-lsblk host $2"; [ "$1" = pass ]; exit $?; }
n=0; while [ ! -c "$TTY" ]; do [ "$n" -ge "$WAIT" ] && say fail "$TTY did not appear in ${WAIT}s"; sleep 1; n=$((n+1)); done
n=0; while [ ! -r "$TTY" ] || [ ! -w "$TTY" ]; do [ "$n" -ge 40 ] && say fail "$TTY not accessible to this user"; sleep 0.5; n=$((n+1)); done
mkdir -p "$OUT" && chmod 700 "$OUT" || say fail "cannot make $OUT"
command exec 3<>"$TTY" || say fail "cannot open $TTY"
timeout 5 stty raw -echo -hupcl 115200 <&3 2>/dev/null || true
redact() { sed -E -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/(androidboot\.serialno=)[^ ]*/\1REDACTED/g' -e 's/(serial(no|number)?[ =:]+)[0-9A-Za-z]{6,}/\1REDACTED/Ig'; }
# 1. the banner line (the first one that lists a LUN), listening up to BEATS seconds
timeout "$BEATS" sh -c 'while IFS= read -r l <&3; do case $l in *WALLEYE-BANNER-UFS*luns=[1-9]*) printf "%s\n" "$l"; break;; esac; done' 2>/dev/null | tr -d '\r' | redact > "$OUT/banner-ufs.txt"
[ -s "$OUT/banner-ufs.txt" ] || say fail "no WALLEYE-BANNER-UFS line with a LUN in ${BEATS}s ($OUT): the UFS did not probe, or not yet"
B=$(sed -n 's/^.*WALLEYE-BANNER-UFS beat=[0-9]* \[\(.*\)\]$/\1/p' "$OUT/banner-ufs.txt" | head -n1)
[ -n "$B" ] || say fail "banner line has no ufs-lsblk lines ($OUT/banner-ufs.txt)"
case $B in *'name=boot_a '*|*'name=boot_a|'*|*'name=boot_a') ;; *) say fail "banner lists no partition named boot_a: $B";; esac
# 2. the commands' answers
ask() { # COMMAND FILE
  M=wul-$$-$1
  timeout 0.3 cat <&3 >/dev/null 2>&1 || true
  printf '%s\n' "$2; echo $M-E''ND" >&3
  timeout 25 sh -c "while IFS= read -r l <&3; do l=\${l%\$(printf '\r')}; case \$l in *$M-END) break;; esac; printf '%s\n' \"\$l\"; done" 2>/dev/null | grep -v -e "$M" -e WALLEYE-BANNER | redact > "$3"
}
ask 1 ufs-lsblk "$OUT/command-lsblk.txt"
ask 2 'ufs-hdr boot_a' "$OUT/command-hdr.txt"
[ "${NOREBOOT:-0}" = 1 ] || printf '%s\n' 'reboot -f' >&3
C=$(grep '^ufs-lsblk ' "$OUT/command-lsblk.txt" | tr '\n' '|' | sed 's/|$//')
[ -n "$C" ] || say fail "the command ufs-lsblk gave no answer ($OUT/command-lsblk.txt)"
H=$(grep '^ufs-hdr part=boot_a .* magic=ANDROID! ' "$OUT/command-hdr.txt" | head -n1)
[ -n "$H" ] || say fail "ufs-hdr boot_a gave no header with magic ANDROID! ($OUT/command-hdr.txt): $(head -n1 "$OUT/command-hdr.txt")"
[ "$B" = "$C" ] || say fail "banner and command differ (the list may have grown between them): banner [$B] command [$C]"
say pass "$(printf '%s\n' "$B" | cut -d'|' -f1); $H (nothing written; out $OUT)"
