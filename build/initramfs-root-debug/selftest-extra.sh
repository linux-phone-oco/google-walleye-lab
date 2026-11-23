# the debug init's own checks, after the host side of the gadget is proven (the lines above are build/initramfs/selftest.sh at 172.16.43.1)
LOG=$( (echo "cat /tmp/walleye-debug.log"; sleep 2; echo exit) | INNS nc -w 5 172.16.43.1 23 2>&1 | tr -d '\000-\010\013-\037\377')
case $LOG in *"walleye-debug: telnetd on 172.16.43.1"*) echo "SELFTEST the log is read over telnet" ;; *) fail "the log /tmp/walleye-debug.log is not readable over telnet" ;; esac
case $LOG in *"walleye-debug: petting /dev/watchdog"*) echo "SELFTEST a watchdog is petted" ;; *) fail "no watchdog petter in the log" ;; esac
echo "SELFTEST PASS"
