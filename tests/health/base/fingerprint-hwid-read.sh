#!/bin/sh
# the fingerprint sensor (FPC, irq GPIO86, reset GPIO128) answered its HWID register over SPI
. /usr/lib/devlab/health/lib.sh
check fingerprint-hwid-read fingerprint "$(dmesg 2>/dev/null | grep -qs 'fpc.*Chip type: FPC' && echo true)" "fpc HWID read over SPI"
