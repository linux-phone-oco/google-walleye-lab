#!/bin/sh
# walleye-modem-up-ipa-order, run on the lab host: does build/initramfs-modem/modem-up bind the IPA after the firmware check and before the daemons (item walleye-modem-net-port-ipa-rebind-after-push)?
# Sources: the order is the driver's: ipa_probe loads ipa_fws itself for qcom,gsi-loader self (drivers/net/ipa/ipa_main.c on tag int/walleye/ufs-off-modem/v7.3/1) and reads it from /lib/firmware, so the set must be whole first;
# the failed boot probe at 3.8 s is the phone's own text (research walleye-modem-mm-first-try-run-6-ipa-probes-before-firmware); the device name 1e40000.ipa is the dts node ipa@1e40000 (dt/msm8998-google-walleye-modem.dts).
# Passes only when, in modem-up, the line order is: sha256sum -c, then the unbind-if-bound and bind of 1e40000.ipa through /sys/bus/platform/drivers/ipa, then the start of qrtr-ns, tqftpserv and rmtfs.
# Prints `pass|fail walleye-modem-up-ipa-order modem DETAIL`; exit 0 only on pass. Environment: MODEM_UP (default build/initramfs-modem/modem-up).
HERE=$(cd "$(dirname "$0")/../.." && pwd)
F=${MODEM_UP:-$HERE/build/initramfs-modem/modem-up}
say() { echo "$1 walleye-modem-up-ipa-order modem $2"; [ "$1" = pass ]; exit $?; }
[ -r "$F" ] || say fail "no $F"
n() { grep -n -m1 -- "$1" "$F" | cut -d: -f1; }
sums=$(n 'sha256sum -c SHA256SUMS')
unb=$(n 'echo \$IPA > \$IPAD/unbind')
bind=$(n 'echo \$IPA > \$IPAD/bind')
qrtr=$(n '^setsid qrtr-ns')
rmtfs=$(n '^setsid rmtfs')
for v in "$sums" "$unb" "$bind" "$qrtr" "$rmtfs"; do [ -n "$v" ] || say fail "a step is missing in $F (sums '$sums', unbind '$unb', bind '$bind', qrtr-ns '$qrtr', rmtfs '$rmtfs')"; done
grep -q '^IPA=1e40000.ipa$' "$F" || say fail "the device is not 1e40000.ipa"
grep -q '^IPAD=/sys/bus/platform/drivers/ipa$' "$F" || say fail "the driver folder is not /sys/bus/platform/drivers/ipa"
[ "$sums" -lt "$unb" ] && [ "$unb" -lt "$bind" ] && [ "$bind" -lt "$qrtr" ] && [ "$qrtr" -lt "$rmtfs" ] || say fail "order is wrong: sums $sums, unbind $unb, bind $bind, qrtr-ns $qrtr, rmtfs $rmtfs"
say pass "sums line $sums, unbind $unb, bind $bind, then qrtr-ns $qrtr and rmtfs $rmtfs"
