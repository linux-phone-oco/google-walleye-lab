#!/bin/sh
# calls work: the modem's IMEI is read over QMI, ModemManager lists the modem, a SIM is present and registered, a call reaches a state.
# Shows skip (not known yet) while a tool or a SIM is absent; fail when the tool answered and the thing is not so.
# Sources: libqmi's qmicli (--dms-get-ids over qrtr://0), ModemManager's mmcli (-L, -m 0, --voice-list-calls); research walleye-modem-to-calls-how-it-stands.
# shellcheck source=/dev/null
. "${LPOCO_HEALTH_LIB:-/usr/lib/devlab/health/lib.sh}"

if ! command -v qmicli >/dev/null 2>&1; then
	say modem-imei-read modem-data skip "qmicli not installed: not read"
else
	imei=$(qmicli -d qrtr://0 --dms-get-ids 2>/dev/null | sed -n "s/.*IMEI: *'\([0-9]\{15\}\)'.*/\1/p")
	check modem-imei-read modem-data "$([ -n "$imei" ] && echo true)" "IMEI ${imei:+read (15 digits)}${imei:-not read over qrtr://0}"
fi

if ! command -v mmcli >/dev/null 2>&1; then
	for t in modem-mm-listed modem-sim-present modem-registered modem-call-signalling; do say "$t" modem-data skip "mmcli not installed: not read"; done
	exit 0
fi
check modem-mm-listed modem-data "$(mmcli -L 2>/dev/null | grep -q '/Modem/' && echo true)" "ModemManager lists a modem"
if ! mmcli -m 0 >/dev/null 2>&1; then
	for t in modem-sim-present modem-registered modem-call-signalling; do say "$t" modem-data skip "no modem 0: not read"; done
	exit 0
fi
info=$(mmcli -m 0 -K 2>/dev/null)
sim=$(printf '%s\n' "$info" | sed -n 's/^modem.generic.sim *: *//p')
check modem-sim-present modem-data "$([ -n "$sim" ] && [ "$sim" != "--" ] && echo true)" "sim: ${sim:---}"
state=$(printf '%s\n' "$info" | sed -n 's/^modem.generic.state *: *//p')
check modem-registered modem-data "$([ "$state" = registered ] || [ "$state" = connected ] && echo true)" "state: ${state:-unknown}"
calls=$(mmcli -m 0 --voice-list-calls 2>/dev/null)
# a listed call is the proof: the call test (a person's dial) leaves one
check modem-call-signalling modem-data "$(printf '%s' "$calls" | grep -q '/Call/' && echo true)" "calls listed: $(printf '%s' "$calls" | grep -c '/Call/')"
