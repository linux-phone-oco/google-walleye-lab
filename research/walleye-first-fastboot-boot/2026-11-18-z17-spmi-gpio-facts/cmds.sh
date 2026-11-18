echo "== dmesg filtered"; dmesg | grep -iE "spmi|pmi8998|pm8998|gpio|pinctrl|regulator-touch-power|rmi4|deferred"
echo "== spmi devices"; for d in /sys/bus/spmi/devices/*; do echo "$d -> $(readlink $d/driver)"; done
echo "== spmi drivers"; ls /sys/bus/spmi/drivers/
echo "== devices_deferred"; mount -t debugfs none /sys/kernel/debug 2>&1; cat /sys/kernel/debug/devices_deferred
echo "== debug gpio"; cat /sys/kernel/debug/gpio
echo "== gpiochips"; for c in /sys/class/gpio/gpiochip*; do echo "$c $(cat $c/label) base=$(cat $c/base) ngpio=$(cat $c/ngpio)"; done
echo "== regulators"; for r in /sys/class/regulator/*; do echo "$r $(cat $r/name) $(cat $r/state 2>&1)"; done
echo "== platform"; ls /sys/bus/platform/devices | grep -iE "touch|regulator"; ls /sys/bus/i2c/devices
echo "== end"
