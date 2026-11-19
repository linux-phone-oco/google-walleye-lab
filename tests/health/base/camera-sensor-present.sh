#!/bin/sh
# a camera sensor entity is registered with the media controller (camss + IMX362)
. /usr/lib/devlab/health/lib.sh
check camera-sensor-present camera-rear "$(grep -qs imx362 /sys/class/video4linux/v4l-subdev*/name && echo true)" "imx362 subdev"
