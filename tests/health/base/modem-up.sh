#!/bin/sh
# the modem is up: remoteproc 4080000.remoteproc (qcom-q6v5-mss) runs and qrtr lists the modem's services.
# Sources: the remoteproc's name is the device's (dt/msm8998-google-walleye-modem.dts, the node at 0x4080000), its state a word of
# /sys/class/remoteproc/*/state (kernel Documentation/ABI/testing/sysfs-class-remoteproc); the service ids are libqmi's QMI_SERVICE_*
# (libqmi-glib/qmi-enums.h: WDS 0x01, DMS 0x02, NAS 0x03, WMS 0x05, VOICE 0x09, UIM 0x0b) and the name qrtr-lookup prints for 21,
# "Modem embedded file system service" (research walleye-modem-mss-first-boot, tests/fixtures/phone-text/modem-read-20261127-104225.cap).
# Only node 0 rows count (the modem's: in the captured read the apps side, node 1, lists 14, 4096 and 64). REQUIRED to be up: 2 (device management) and 21 (the modem's own embedded file system service), both listed on node 0 in that capture.
# NAS, WDS and voice are printed present or absent, never required: a modem without a SIM may still list them, and what it lists is not known until read.
# LPOCO_MODEM_READ=FILE reads a kept telnet read (---remoteproc / ---qrtr sections) instead of the phone: the proof's way.
# shellcheck source=/dev/null
. "${LPOCO_HEALTH_LIB:-/usr/lib/devlab/health/lib.sh}"
MSS=4080000.remoteproc
REQUIRED="2 21"

# section NAME FILE - the lines between ---NAME and the next ---
section() { sed -n "/^---$1\$/,/^---/{/^---/!p}" "$2"; }

remoteprocs() { # NAME STATE per line
	if [ -n "${LPOCO_MODEM_READ:-}" ]; then section remoteproc "$LPOCO_MODEM_READ"; return; fi
	for d in /sys/class/remoteproc/remoteproc*; do
		[ -r "$d/name" ] && echo "$(cat "$d/name") $(cat "$d/state" 2>/dev/null)"
	done
}
qrtr_text() {
	if [ -n "${LPOCO_MODEM_READ:-}" ]; then section qrtr "$LPOCO_MODEM_READ"; else qrtr-lookup 2>/dev/null; fi
}

rp=$(remoteprocs)
if [ -z "$rp" ]; then
	say modem-remoteproc-running modem-data skip "no remoteproc listed: not read"
else
	state=$(printf '%s\n' "$rp" | awk -v n="$MSS" '$1 == n { print $2 }')
	check modem-remoteproc-running modem-data "$([ "$state" = running ] && echo true)" "$MSS: ${state:-not listed} (listed: $(printf '%s' "$rp" | tr '\n' ' '))"
fi

q=$(qrtr_text | awk 'NR > 1 && $1 ~ /^[0-9]+$/ && $4 == 0 { print $1 }' | sort -n -u)
if [ -z "$q" ]; then
	if [ -z "$rp" ]; then say modem-qrtr-services modem-data skip "no qrtr listing: not read"
	else say modem-qrtr-services modem-data fail "qrtr lists no service"; fi
else
	count=$(printf '%s\n' "$q" | wc -l | tr -d ' ')
	missing=""
	for s in $REQUIRED; do in_list "$s" "$(printf '%s' "$q" | tr '\n' ' ')" || missing="$missing $s"; done
	word() { if in_list "$2" "$(printf '%s' "$q" | tr '\n' ' ')"; then echo "$1 $2 present"; else echo "$1 $2 absent"; fi; }
	check modem-qrtr-services modem-data "$([ -z "$missing" ] && echo true)" "$count services; required 2 and 21${missing:+, missing:$missing}; $(word NAS 3), $(word WDS 1), $(word voice 9)"
fi
