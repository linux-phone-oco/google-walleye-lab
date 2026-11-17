#!/bin/sh
# walleye-try-ml3-ramoops, run on the lab host with the phone booted to Android (adb root): did OUR mainline kernel's console survive in the stock
# Android's pstore? pass = a file in /sys/fs/pstore holds our banner ("Linux version 7."). Reads only. Prints one line; exit 0 only on pass.
n=$(adb shell 'cat /sys/fs/pstore/console-ramoops* 2>/dev/null | grep -c "Linux version 7\."' | tr -d '\r')
if [ "${n:-0}" -ge 1 ]; then echo "pass walleye-ml3-ramoops-read pstore holds our kernel's console ($n banner lines)"; exit 0; fi
echo "fail walleye-ml3-ramoops-read no console-ramoops with our banner in /sys/fs/pstore ($(adb shell 'ls /sys/fs/pstore | wc -l' | tr -d '\r') files)"
exit 1
