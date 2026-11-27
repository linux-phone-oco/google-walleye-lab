#!/bin/sh
# Proves tests/health/base/modem-mm-starts.sh on mmcli's own words, never touching the phone. The words are ModemManager 1.24.2's source (research walleye-mmcli-list-words):
#   mmcli -L prints one line per modem, "    PATH [MANUFACTURER] MODEL" (cli/mmcli-manager.c output_modem_info, FOUND_ACTION_PREFIX of four spaces) and, with none,
#   "No modems were found" (cli/mmcli-output.c dump_output_list_human: "No %s were found" with the field's name "modems");
#   a failure of mmcli itself starts "error:" (mmcli.c), e.g. "error: couldn't find the ModemManager process in the bus".
# These are stand-ins made from those format strings, NOT captured from a phone: the first real run (the keeper's) is the capture. The text carries no IMEI: the checks name none.
# The kept read is the modem boot's mmcli.txt: ---mmcli-L and ---mmcli-m0 sections, ids redacted. The suite's words (lib.sh) are lab-tools' own, read from its origin/main.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
CHECK="$HERE/../health/base/modem-mm-starts.sh"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
git -C "${LAB_TOOLS:-$HOME/Projects/linux-phone-oco/lab-tools}" show origin/main:health/suite/lib.sh > "$T/lib.sh"
total=0
want() { # NAME TEST EXPECTED_WORD READ_FILE
  got=$(LPOCO_HEALTH_LIB="$T/lib.sh" LPOCO_MM_READ="$4" sh "$CHECK" | awk -v t="$2" '$2 == t { print $1 }')
  if [ "$got" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$got', want $3"; total=$((total+1)); fi
}
LIST='    /org/freedesktop/ModemManager1/Modem/0 [QUALCOMM INCORPORATED] MSM8998'
printf -- '---mmcli-L\n%s\n---mmcli-m0\nerror: couldn'"'"'t find modem\n---end\n' "$LIST" > "$T/one.txt"
printf -- '---mmcli-L\nNo modems were found\n---mmcli-m0\nerror: couldn'"'"'t find modem\n---end\n' > "$T/none.txt"
printf -- '---mmcli-L\nerror: couldn'"'"'t find the ModemManager process in the bus\n---mmcli-m0\n---end\n' > "$T/nobus.txt"
printf -- '---mmcli-L\n---mmcli-m0\n---end\n' > "$T/blank.txt"
: > "$T/empty.txt"
want "one modem listed"                          modem-mm-starts pass "$T/one.txt"
want "'No modems were found'"                     modem-mm-starts fail "$T/none.txt"
want "mmcli cannot reach ModemManager (error:)"  modem-mm-starts fail "$T/nobus.txt"
want "a read with no mmcli words skips"          modem-mm-starts skip "$T/blank.txt"
want "an empty read skips"                       modem-mm-starts skip "$T/empty.txt"
want "a missing read file skips"                 modem-mm-starts skip "$T/absent.txt"
detail=$(LPOCO_HEALTH_LIB="$T/lib.sh" LPOCO_MM_READ="$T/one.txt" sh "$CHECK" | grep ' modem-mm-starts ')
case "$detail" in *"1 modem"*"Modem/0"*) echo "ok   the detail counts the modems and names the path" ;; *) echo "FAIL detail: $detail"; total=$((total+1)) ;; esac
[ $total -eq 0 ] || { echo "modem-mm-starts-selftest: $total failed" >&2; exit 1; }
echo "pass modem-mm-starts-selftest"
