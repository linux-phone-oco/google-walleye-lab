#!/bin/sh
# walleye-phone-state-selftest: proves phone-state.sh against fake adb, fastboot, sysfs and tty; the phone is never touched.
# The banner the tty answers with is a line a phone printed (tests/fixtures/phone-text/banner-z6-try1.cap, sourced in its SOURCES.toml); the adb and fastboot lists, the sysfs and the other tty bytes stay typed (listed unsourced).
# Prints one line, `pass|fail walleye-phone-state-selftest host DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")" && pwd)
CAP="$HERE/../fixtures/phone-text/banner-z6-try1.cap"
T=$(mktemp -d) || exit 1; trap 'rm -rf "$T"' EXIT
fail() { echo "fail walleye-phone-state-selftest host $1"; exit 1; }
mkdir "$T/bin" "$T/usb"
printf '[fastboot]\nserial = "FA81N1A00790"\n' > "$T/device.toml"
mkfake() { printf '#!/bin/sh\n%s\n' "$2" > "$T/bin/$1"; chmod +x "$T/bin/$1"; }
gadget() { rm -rf "$T/usb"/*; mkdir "$T/usb/3-2"; echo walleye-first-boot > "$T/usb/3-2/serial"; echo "Pixel 2 walleye (mainline)" > "$T/usb/3-2/product"; }
nousb() { rm -rf "$T/usb"/*; mkdir "$T/usb/3-1"; echo postmarketOS > "$T/usb/3-1/serial"; echo "Qualcomm SM7150 Device" > "$T/usb/3-1/product"; }
# case NAME WANT_RC WANT_VERDICT : runs with the current fakes
run() { PATH="$T/bin:$PATH" SYSFS_USB="$T/usb" TTY="$T/tty" TTY_WAIT=1 CALL_WAIT=2 timeout -k 1 30 sh "$HERE/phone-state.sh" "$T/device.toml" 2>&1; }
case_() { out=$(run); rc=$?; [ "$rc" -eq "$2" ] || fail "$1: exit $rc, want $2: $out"; echo "$out" | grep -q "^verdict $3 " || fail "$1: verdict wrong: $out"; echo "$out" | grep -q 'hung' && fail "$1: the word hung was printed"; }
mkfake adb 'echo "List of devices attached"; echo "FA81N1A00790	device"'; mkfake fastboot 'exit 0'
# adb device with a stale gadget listed -> android
gadget; : > "$T/tty"; case_ "android with a stale gadget" 0 android
# gadget listed and tty beat -> mainline-answering
mkfake adb 'echo "List of devices attached"'; grep -m1 '^WALLEYE-BANNER beat=' "$CAP" > "$T/tty"; case_ "gadget and beat" 0 mainline-answering
# gadget listed and tty silent -> not known (mainline-silent), exit 2
: > "$T/tty"; case_ "gadget and silent tty" 2 mainline-silent
out=$(run); echo "$out" | grep -q 'NOT KNOWN' || fail "silent gadget does not say NOT KNOWN: $out"
# gadget listed, node missing -> still not known
rm -f "$T/tty"; case_ "gadget and no node" 2 mainline-silent
# fastboot lists the serial
nousb; mkfake fastboot 'echo "FA81N1A00790	fastboot"'; case_ "fastboot" 0 fastboot
# nothing (surya's gadget on the bus only) -> gone
mkfake fastboot 'exit 0'; case_ "nothing" 1 gone
# adb unauthorized -> not known
mkfake adb 'echo "List of devices attached"; echo "FA81N1A00790	unauthorized"'; case_ "unauthorized" 2 not-known
# the Pixel's own serial on the bus, nobody else sees it -> not known
mkfake adb 'echo "List of devices attached"'; rm -rf "$T/usb"/*; mkdir "$T/usb/3-3"; echo FA81N1A00790 > "$T/usb/3-3/serial"; echo 18d1 > "$T/usb/3-3/idVendor"; echo 4ee7 > "$T/usb/3-3/idProduct"; case_ "pixel serial only" 2 not-known
# both tools hang: bounded, gone, within 15 s
mkfake adb 'exec sleep 60'; mkfake fastboot 'exec sleep 60'; nousb
s=$(date +%s); case_ "hanging tools" 1 gone; e=$(( $(date +%s) - s )); [ "$e" -lt 15 ] || fail "hanging tools took $e s"
# adb in recovery (and "no permissions"), nothing on the bus -> not known, never gone
nousb; mkfake fastboot 'exit 0'
mkfake adb 'echo "List of devices attached"; echo "FA81N1A00790	recovery"'; case_ "recovery" 2 not-known
mkfake adb 'echo "List of devices attached"; echo "FA81N1A00790	no permissions (user in plugdev group; are your udev rules wrong?)"'; case_ "no permissions" 2 not-known
# another serial only -> gone
mkfake adb 'echo "List of devices attached"; echo "OTHER123	device"'; case_ "other serial" 1 gone
# tty answering with non-banner bytes
mkfake adb 'echo "List of devices attached"'; gadget; echo "/ # " > "$T/tty"; case_ "non-banner bytes" 0 mainline-answering
# the tty comes from the gadget's own interface, never a default node: with no /dev node of its own there is no read
mkdir -p "$T/usb/3-2/3-2:1.0/tty/ttyACM7"
out=$(PATH="$T/bin:$PATH" SYSFS_USB="$T/usb" TTY_WAIT=1 CALL_WAIT=2 sh "$HERE/phone-state.sh" "$T/device.toml" 2>&1); echo "$out" | grep -q 'witness tty no-node' || fail "gadget with no /dev node: $out"
# a bad wait value stops at the edge
out=$(TTY_WAIT=x sh "$HERE/phone-state.sh" "$T/device.toml"); [ $? -eq 2 ] || fail "bad TTY_WAIT: $out"
# no serial in the manifest -> not known
: > "$T/empty.toml"; out=$(sh "$HERE/phone-state.sh" "$T/empty.toml"); [ $? -eq 2 ] || fail "no serial: $out"
echo "pass walleye-phone-state-selftest host android, fastboot, answering, silent (not known), gone and not-known come out where they should; never 'hung'"
