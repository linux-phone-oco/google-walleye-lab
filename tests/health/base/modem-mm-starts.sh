#!/bin/sh
# ModemManager starts and lists the modem: mmcli -L names a modem path.
# Sources: mmcli's own words, ModemManager 1.24.2 (research walleye-mmcli-list-words): -L prints "    PATH [MANUFACTURER] MODEL" per modem
# (cli/mmcli-manager.c output_modem_info), "No modems were found" when it answered with none (cli/mmcli-output.c dump_output_list_human),
# and "error: ..." when it could not ask ModemManager at all (e.g. the process is not on the bus). Not known yet (skip): no mmcli words read.
# LPOCO_MM_READ=FILE reads the modem boot's kept mmcli.txt (---mmcli-L / ---mmcli-m0 sections, ids redacted) instead of the phone: the proof's way.
# shellcheck source=/dev/null
. "${LPOCO_HEALTH_LIB:-/usr/lib/devlab/health/lib.sh}"
MODEM_PATH='/org/freedesktop/ModemManager1/Modem/[0-9][0-9]*'

# section NAME FILE - the lines between ---NAME and the next ---
section() { sed -n "/^---$1\$/,/^---/{/^---/!p}" "$2"; }

listing() {
	if [ -n "${LPOCO_MM_READ:-}" ]; then [ -r "$LPOCO_MM_READ" ] && section mmcli-L "$LPOCO_MM_READ"; return 0; fi
	command -v mmcli >/dev/null 2>&1 && mmcli -L 2>&1
	return 0
}

text=$(listing)
if [ -z "$text" ]; then
	say modem-mm-starts modem-data skip "no mmcli -L words: not read"
	exit 0
fi
count=$(printf '%s\n' "$text" | grep -c "$MODEM_PATH")
first=$(printf '%s\n' "$text" | grep -o "$MODEM_PATH" | head -n 1)
if [ "$count" -gt 0 ]; then
	check modem-mm-starts modem-data true "$count modem listed by ModemManager ($first)"
elif printf '%s\n' "$text" | grep -q '^No modems were found'; then
	check modem-mm-starts modem-data false "ModemManager answered: No modems were found"
elif printf '%s\n' "$text" | grep -q '^error:'; then
	check modem-mm-starts modem-data false "mmcli could not ask ModemManager: $(printf '%s\n' "$text" | grep -m1 '^error:')"
else
	say modem-mm-starts modem-data skip "mmcli -L said none of its known words: not judged"
fi
