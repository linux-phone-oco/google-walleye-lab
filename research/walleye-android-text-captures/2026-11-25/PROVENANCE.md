# Android b text captures, 2026-11-25 (walleye-keeper, read only, nothing written to the phone)
Phone: Pixel 2 walleye FA81N1A00790, stock Android on slot b (ro.boot.slot_suffix=_b), no image flashed or booted.
- power-supply-facts.txt: adb shell, per /sys/class/power_supply/* a "== NAME" line then KEY=VALUE for type status present online health capacity voltage_now current_now temp charge_type (those readable).
- partition-table.txt: adb shell 'ls -l /dev/block/by-name; echo ---; cat /proc/partitions'; the /proc/partitions half says "Permission denied" for the unrooted shell and is kept as printed.
Android's supply is named "battery" (not qcom-battery): this is the reference text, not our kernel's.
