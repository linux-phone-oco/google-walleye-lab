#!/bin/sh
# the SoC reaches a deep sleep state: cpuidle states of cpu0 beyond the first were entered
. /usr/lib/devlab/health/lib.sh
check cpuidle-deep-entered deep-sleep "$(cat /sys/devices/system/cpu/cpu0/cpuidle/state[1-9]/usage 2>/dev/null | grep -qv '^0$' && echo true)" "cpuidle deeper than WFI entered"
