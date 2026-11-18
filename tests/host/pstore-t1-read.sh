#!/bin/sh
# walleye-pstore-t1-sysrq-reset, run on the lab host with the phone back as Android (adb root): did the T1 marker survive sysrq c + panic=5 in /sys/fs/pstore?
# pass = a console-ramoops file holds "walleye-t1-marker-7f3a". Reads only. Prints one line; exit 0 pass, 1 fail (a root read found no marker), 2 not measured (no phone or no root).
[ "$(adb get-state 2>/dev/null)" = device ] || { echo "unknown walleye-pstore-t1-read no adb device (not measured)"; exit 2; }
[ "$(adb shell id -u 2>/dev/null | tr -d '\r')" = 0 ] || { echo "unknown walleye-pstore-t1-read adb is not root (not measured)"; exit 2; }
f=$(adb shell 'ls /sys/fs/pstore 2>/dev/null | grep -c console-ramoops' | tr -d '\r')
n=$(adb shell 'cat /sys/fs/pstore/console-ramoops* 2>/dev/null | grep -c "walleye-t1-marker-7f3a"' | tr -d '\r')
if [ "${n:-0}" -ge 1 ]; then echo "pass walleye-pstore-t1-read pstore holds the T1 marker ($n lines)"; exit 0; fi
echo "fail walleye-pstore-t1-read no T1 marker in /sys/fs/pstore ($f console-ramoops files; expected on the phone: cause B)"
exit 1
