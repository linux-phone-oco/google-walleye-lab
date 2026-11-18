#!/bin/sh
# Rerun the whole comparison.  usage: run-all.sh WORKDIR KERNEL_GIT_DIR OUR_DTB LIVE_DT_TXT RESULT_DIR
#   e.g. run-all.sh /tmp/w ~/Projects/walleye-build/linux-v7.3 ~/Projects/walleye-build/out-z13/msm8998-google-walleye.dtb \
#        research/walleye-first-fastboot-boot/2026-11-18/dt.txt research/walleye-first-fastboot-boot/2026-11-18-ufs-android-way/results
set -eu
W=$1; G=$2; D=$3; L=$4; R=$5; H=$(dirname "$0"); mkdir -p "$W" "$R"
sh "$H/fetch-vendor.sh" "$W/v"
sh "$H/extract-ours.sh" "$G" d49d54405 "$W/o"
sh "$H/phy-tables.sh" "$W/v" "$W/o" "$R/phy"
sh "$H/quirks.sh" "$W/v" "$W/o" "$R/quirks"
sh "$H/dt-props.sh" "$L" "$D" "$R/dt"
