#!/bin/sh
# the SLPI is running and the sensor service answers on qrtr
. /usr/lib/devlab/health/lib.sh
check slpi-running sensors "$(grep -qs running /sys/class/remoteproc/remoteproc*/state && echo true)" "a remoteproc is running"
check sensor-service sensors "$(qrtr-lookup 2>/dev/null | grep -qi 'sensor' && echo true)" "sensor service on qrtr"
