#!/bin/sh
# the NFC controller (PN553) is bound to nxp-nci and an NFC device exists
. /usr/lib/devlab/health/lib.sh
check nfc-present nfc "$(ls /sys/class/nfc/nfc0 >/dev/null 2>&1 && echo true)" "nfc0 from nxp-nci"
