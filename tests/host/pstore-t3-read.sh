#!/bin/sh
# walleye-pstore-t3-wdog-bite, run on the lab host with the phone back as Android (adb root): did the T3 marker survive an unpetted kpss-wdt bite in /sys/fs/pstore?
# pass = a console-ramoops file holds "walleye-t3-marker-5c1e". Reads only. Prints one line; exit 0 pass, 1 fail (a root read found no marker), 2 not measured (no phone or no root).
[ "$(adb get-state 2>/dev/null)" = device ] || { echo "unknown walleye-pstore-t3-read no adb device (not measured)"; exit 2; }
[ "$(adb shell id -u 2>/dev/null | tr -d '\r')" = 0 ] || { echo "unknown walleye-pstore-t3-read adb is not root (not measured)"; exit 2; }
f=$(adb shell 'ls /sys/fs/pstore 2>/dev/null | grep -c console-ramoops' | tr -d '\r')
n=$(adb shell 'cat /sys/fs/pstore/console-ramoops* 2>/dev/null | grep -c "walleye-t3-marker-5c1e"' | tr -d '\r')
if [ "${n:-0}" -ge 1 ]; then echo "pass walleye-pstore-t3-read pstore holds the T3 marker ($n lines)"; exit 0; fi
echo "fail walleye-pstore-t3-read no T3 marker in /sys/fs/pstore ($f console-ramoops files)"
exit 1
