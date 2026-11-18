#!/bin/sh
# c5-loader-selftest (backlog walleye-build-c5): runs init-c5's module-loader block on the host with stubs and MEASURES when the charger module would be loaded:
# never without walleye.charger_after, never with a bad value or an unbound gadget, once and after the asked seconds with a good one. Touches no phone.
#   sh tests/host/c5-loader-selftest.sh [INIT]   (default build/initramfs-mainline/init-c5)
set -u
H=$(cd "$(dirname "$0")" && pwd)
INIT=${1:-$H/../../build/initramfs-mainline/init-c5}
[ -f "$INIT" ] || { echo "skip: no $INIT (not known yet)"; exit 2; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
sed -n '/^CA=$/,/^esac$/p' "$INIT" | sed -e 's#/proc/cmdline#$CMDLINE#' -e 's#/dev/kmsg#/dev/null#g' -e "s#/tmp/#$T/#g" > "$T/block.sh"
ck "the loader block was found in init-c5" "$([ "$(wc -l < "$T/block.sh")" -gt 12 ] && echo yes || echo no)" yes
# one run: CMDLINE_TEXT BOUND(yes|no) -> what the block did, as "say-lines|sleeps|insmods"
run() {
  : > "$T/log"; printf '%s\n' "$1" > "$T/cmdline"; mkdir -p "$T/g"; UDC=a600000.usb
  if [ "$2" = yes ]; then echo "$UDC" > "$T/g/UDC"; else : > "$T/g/UDC"; fi
  (
    CMDLINE=$T/cmdline; G=$T/g
    say() { echo "say: $*" >> "$T/log"; }
    sleep() { echo "sleep $1" >> "$T/log"; }
    insmod() { echo "insmod $1" >> "$T/log"; }
    . "$T/block.sh"
    wait
  )
  tr '\n' '|' < "$T/log"
}
r=$(run "console=ttyMSM0 androidboot.x=1" yes);  ck "no argument: the module is not loaded" "$(echo "$r" | grep -c insmod)" 0
r=$(run "walleye.charger_after=40" yes);        ck "=40 and bound: it sleeps 40 and loads once" "$(echo "$r" | grep -o 'sleep 40|insmod /lib/modules/qcom_smbx.ko|')" "sleep 40|insmod /lib/modules/qcom_smbx.ko|"
ck "=40 and bound: exactly one insmod" "$(echo "$r" | grep -o insmod | wc -l | tr -d ' ')" 1
r=$(run "walleye.charger_after=40" no);         ck "=40 but the gadget is not bound: not loaded" "$(echo "$r" | grep -c insmod)" 0
r=$(run "walleye.charger_after=abc" yes);       ck "=abc: not loaded" "$(echo "$r" | grep -c insmod)" 0
r=$(run "walleye.charger_after=0" yes);         ck "=0: not loaded" "$(echo "$r" | grep -c insmod)" 0
r=$(run "walleye.charger_after=121" yes);       ck "=121: not loaded" "$(echo "$r" | grep -c insmod)" 0
r=$(run "walleye.charger_after=120" yes);       ck "=120: loaded" "$(echo "$r" | grep -o insmod | wc -l | tr -d " ")" 1
r=$(run "walleye.charger_after=" yes);          ck "empty value: not loaded" "$(echo "$r" | grep -c insmod)" 0
[ $total -eq 0 ] || { echo "c5-loader-selftest: $total failed" >&2; exit 1; }
echo "pass c5-loader-selftest"
