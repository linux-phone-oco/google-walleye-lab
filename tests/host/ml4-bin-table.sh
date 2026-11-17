#!/bin/sh
# walleye-try-ml4 reading aid and host test: the bin table of ml4's timing channel, checked against the init source and the built image.
#   ml4-bin-table.sh INIT_SOURCE            prints the table (score -> sleep seconds -> the time Android is back, with the control base) and checks the init says it
#   ml4-bin-table.sh INIT_SOURCE SECONDS    reads one measured delay (reset time minus the control's base 25-36 s taken as BASE=30 unless BASE is set) into a score
# Pure shell, no phone. Exit 0 only when the init source states the same checklist and the 5 s bins.
set -eu
I=${1:?usage: ml4-bin-table.sh INIT_SOURCE [SECONDS_FROM_FASTBOOT_BOOT_TO_ADB]}
BASE=${BASE:-30}
fail=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; fail=1; fi; }
ck "init counts 7 checks (c2..c7 plus the always-1 start)" "$(grep -c -E '^\[ .*\] && c[2-7]=1$|^grep -q .* && c5=1$' "$I")" 6
ck "init sleeps 5 s per point" "$(grep -c -e "5 \* score" "$I")" 2
ck "init opens the watchdog before the checklist" "$( [ "$(grep -n 'exec 3> /dev/watchdog' "$I" | cut -d: -f1)" -lt "$(grep -n '^c2=0' "$I" | cut -d: -f1)" ] && echo yes || echo no)" yes
ck "init ends with reboot -f and a sysrq b fallback" "$(grep -c "^say \"ml4 self-reboot now\"; sync; reboot -f; sleep 5; echo b > /proc/sysrq-trigger$" "$I")" 1
echo "score  sleep   Android back after fastboot boot (base $BASE s)"
s=1; while [ $s -le 7 ]; do echo "$s      $((5*s)) s    ~$((BASE + 12 + 5*s)) s"; s=$((s+1)); done
echo "(12 s = the bootloader's own start before init; ~70 s with no score = the safety watchdog fired: init never ran or hung before the sleep)"
if [ -n "${2:-}" ]; then
  d=$(( $2 - BASE - 12 )); [ $d -ge 0 ] || d=0
  echo "measured $2 s -> delay $d s -> score $(( (d + 2) / 5 ))"
fi
[ $fail -eq 0 ]
