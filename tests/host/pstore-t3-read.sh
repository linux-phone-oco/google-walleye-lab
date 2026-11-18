#!/bin/sh
# walleye-pstore-t3-wdog-bite, run on the lab host with the phone back as Android (adb root): did the T3 marker survive an unpetted kpss-wdt bite in /sys/fs/pstore?
# pass = a console-ramoops file holds "walleye-t3-marker-5c1e", its last tick is under 15 (the bite came at ~10 s) and the init's "no bite" line is absent.
# Reads only. Prints one line; exit 0 pass, 1 fail, 2 not measured (no phone or no root). A marker with a late last tick or the no-bite line = the watchdog never bit.
[ "$(adb get-state 2>/dev/null)" = device ] || { echo "unknown walleye-pstore-t3-read no adb device (not measured)"; exit 2; }
[ "$(adb shell id -u 2>/dev/null | tr -d '\r')" = 0 ] || { echo "unknown walleye-pstore-t3-read adb is not root (not measured)"; exit 2; }
f=$(adb shell 'ls /sys/fs/pstore 2>/dev/null | grep -c console-ramoops' | tr -d '\r')
v=$(adb shell 'ls /data/vendor_ce/0/ramoops 2>/dev/null | grep -c console' | tr -d '\r')
all=$(adb shell 'cat /sys/fs/pstore/console-ramoops* /data/vendor_ce/0/ramoops/console* 2>/dev/null | grep "walleye-t3-marker-5c1e\|T3 no bite"' | tr -d '\r')
n=$(printf "%s\n" "$all" | grep -c "walleye-t3-marker-5c1e")
nb=$(printf "%s\n" "$all" | grep -c "T3 no bite")
last=$(printf "%s\n" "$all" | sed -n 's/.*tick \([0-9]*\).*/\1/p' | sort -n | tail -n1)
if [ "${n:-0}" -ge 1 ] && [ "${nb:-0}" -eq 0 ] && [ "${last:--1}" -ge 0 ] && [ "$last" -lt 15 ]; then echo "pass walleye-pstore-t3-read pstore holds the T3 marker, last tick $last, no 'no bite' line"; exit 0; fi
if [ "${n:-0}" -ge 1 ]; then echo "fail walleye-pstore-t3-read marker present but no bite (last tick ${last:-none}, no-bite lines $nb): the watchdog did not fire"; exit 1; fi
echo "fail walleye-pstore-t3-read no T3 marker ($f console-ramoops in /sys/fs/pstore, $v in vendor_ce): cause B, or C (reader/vendor ramoops) not separated"
exit 1
