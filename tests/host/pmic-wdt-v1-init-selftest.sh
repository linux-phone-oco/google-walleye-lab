#!/bin/sh
# pmic-wdt-v1-init-selftest (item walleye-pmic-wdt-v1-image): proves on the host, never touching the phone, that init-wdt1 reads the PMIC watchdog, arms it ONLY through the two explicit commands, only when the S2 type
# reads back as asked (or, written out, when the driver's own probe is trusted), that an arming that did not happen is said so and never followed by the reboot, that the self-reboot is held while armed, and that the
# petter picks the SoC watchdog by its identity whichever number each watchdog has.
#   sh tests/host/pmic-wdt-v1-init-selftest.sh [INIT]     (INIT default build/initramfs-mainline/init-wdt1; busybox ash if ~/Projects/walleye-build/busybox-x86_64 exists, else sh)
# The tools and the petter's search are cut out of the init (the heredocs between `cat > /bin/NAME <<'EOS'` and EOS, and the lines from `SOCWD=` to the first `done`), their /bin, /sys, /dev and /tmp paths pointed at a temp
# dir, and run against a fake sysfs, a fake PON line in a file, a fake dmesg and fake watchdog, reboot, sync and sleep programs. The fake `watchdog` writes its call and "arms" (rewrites the fake PON file) BEFORE the fake
# `sleep 1` of wdt-arm returns (then a 0.3 s grace for a fake that exits at once), so a check waits for what it needs and never for a clock. Prints `pass|fail pmic-wdt-v1-init-selftest host DETAIL`; exit 0 only on pass.
set -u
H=$(cd "$(dirname "$0")" && pwd)
INIT=${1:-$H/../../build/initramfs-mainline/init-wdt1}
BBX=$HOME/Projects/walleye-build/busybox-x86_64
SH=sh; [ -x "$BBX" ] && SH="$BBX ash"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
[ -f "$INIT" ] || { echo "fail pmic-wdt-v1-init-selftest host no $INIT"; exit 1; }
mkdir "$T/bin"
for n in wdt-state wdt-arm wdt-bite wdt-reboot; do
  sed -n "/^cat > \/bin\/$n <<.EOS.\$/,/^EOS\$/p" "$INIT" | sed '1d;$d' | sed -e "s#/bin/wdt-state#$T/bin/wdt-state#g" -e "s#/bin/wdt-arm#$T/bin/wdt-arm#g" -e "s#/bin/pon-regs#$T/bin/pon-regs#g" -e "s#/dev/kmsg#$T/kmsg#g" > "$T/bin/$n"
  chmod 755 "$T/bin/$n"
  ck "the init holds the tool $n" "$([ "$(wc -l < "$T/bin/$n")" -gt 2 ] && echo yes || echo no)" yes
done
# fakes. watchdog: writes its call, "arms" by turning the fake S2 enable bit on (unless FAKE_NOARM), stays alive a few seconds like the applet (unless FAKE_DIE: exits at once, as an applet that could not open the device).
cat > "$T/bin/watchdog" <<EOF
#!/bin/sh
echo "watchdog \$*" >> "$T/calls"
[ -z "\${FAKE_NOARM:-}" ] && sed 's/0x57=00/0x57=80/' "$T/pon" > "$T/pon.new" && mv "$T/pon.new" "$T/pon"
[ -n "\${FAKE_DIE:-}" ] && exit 1
exec /bin/sleep 5
EOF
# sleep: the 1 s of wdt-arm waits for the fake watchdog's write; the hold's 90 s ends soon on its own; the petter test makes every sleep a no-op
cat > "$T/bin/sleep" <<EOF
#!/bin/sh
[ -n "\${FAKE_SLEEP_NOOP:-}" ] && exit 0
case \$1 in
  1) n=0; while [ ! -s "$T/calls" ] && [ \$n -lt 60 ]; do /bin/sleep 0.05; n=\$((n+1)); done; /bin/sleep 0.3;;
  90) exec /bin/sleep 4;;
  *) exit 0;;
esac
EOF
for n in reboot sync; do printf '#!/bin/sh\necho "%s $*" >> "%s/calls"\n' "$n" "$T" > "$T/bin/$n"; done
printf '#!/bin/sh\ncat "%s/pon"\n' "$T" > "$T/bin/pon-regs"
chmod 755 "$T"/bin/*
PATH="$T/bin:$PATH"; export PATH
mkdir -p "$T/sys/class/watchdog/watchdog0" "$T/sys/class/watchdog/watchdog1"
echo qcom_wdt > "$T/sys/class/watchdog/watchdog0/identity"; echo "QCOM PM8916 PON WDT" > "$T/sys/class/watchdog/watchdog1/identity"
export SYS=$T/sys
DM="[    1.2] pm8916-wdt: S2 reset control was 0x1
[    1.2] pm8916-wdt: POFF reason: 0x0 0x20"
P7="pon-regs 0x54=01 0x55=02 0x56=07 0x57=00 0x5a=0c 0x5b=33 0xc5=00 src=[spmi-0]"
P1="pon-regs 0x54=01 0x55=02 0x56=01 0x57=00 0x5a=0c 0x5b=33 0xc5=00 src=[spmi-0]"
P7ON="pon-regs 0x54=01 0x55=02 0x56=07 0x57=80 0x5a=0c 0x5b=33 0xc5=00 src=[spmi-0]"
PNONE="pon-regs none regmaps=[] dbgfs=[no]"
run() { # PONLINE TOOL [ARGS...]: a fresh fake machine, then the tool
  p=$1; shift; t=$1; shift
  rm -f "$T/calls" "$T/hold"; printf '%s\n' "$p" > "$T/pon"
  DMESG=$DM HOLD=$T/hold WDT_SLEEP_HOLD=90 $SH "$T/bin/$t" "$@" 2>&1
}
called() { if [ -s "$T/calls" ]; then tr '\n' '|' < "$T/calls"; else echo none; fi; }
# the state line
S=$(run "$P7" wdt-state)
ck "wdt-state: both watchdogs by identity, the S2 type read back, S2 off, the source, the firmware's type, the probe seen" "$S" "wdt-state pmic=watchdog1 soc=watchdog0 s2-type=0x07 s2-enabled=no src=spmi-0 firmware-type=0x1 probe=pmic-device-registered kline-type=[?    1.2? pm8916-wdt: S2 reset control was 0x1] kline-poff=[?    1.2? pm8916-wdt: POFF reason: 0x0 0x20]"
ck "wdt-state: S2 enabled reads yes" "$(run "$P7ON" wdt-state | sed -n 's/.* s2-enabled=\([a-z]*\) .*/\1/p')" yes
ck "wdt-state is one line" "$(printf '%s\n' "$S" | wc -l | tr -d ' ')" 1
ck "wdt-state: no registers read says unread, not an error" "$(run "$PNONE" wdt-state | sed -n 's/.* s2-type=\([a-z0-9]*\) s2-enabled=\([a-z]*\) src=\([a-z]*\) .*/\1 \2 \3/p')" "unread unread none"
mv "$T/sys/class/watchdog/watchdog1" "$T/sys/class/watchdog/gone1"
ck "wdt-state: no PMIC device says so" "$(run "$P7" wdt-state | sed -n 's/.* pmic=\([a-z0-9]*\) .* probe=\([a-z-]*\) .*/\1 \2/p')" "none no-pmic-device"
OUT=$(run "$P7" wdt-bite); ck "wdt-bite without a PMIC device is refused (nothing armed, no hold)" "$(printf '%s\n' "$OUT" | grep -c '^wdt-bite: wdt-arm refused: no PMIC watchdog device')|$(called)|$([ -e "$T/hold" ] && echo hold || echo nohold)" "1|none|nohold"
mv "$T/sys/class/watchdog/gone1" "$T/sys/class/watchdog/watchdog1"
# arming
OUT=$(run "$P7" wdt-bite)
ck "wdt-bite (type 7 reads 7): busybox watchdog -F -T 20 -t 3600 on the PMIC device, once" "$(called)" "watchdog -F -T 20 -t 3600 /dev/watchdog1|"
ck "wdt-bite says it armed, with S2 enabled read before and after, and the hold" "$OUT" "wdt-bite: wdt-arm armed watchdog1 type=0x07 timeout=20 s2-enabled-before=no after=yes hold=90s; not petted, the bite is due in 20 s"
ck "wdt-bite holds the 180 s self-reboot (the hold file exists after the arming)" "$([ -e "$T/hold" ] && echo yes || echo no)" yes
OUT=$(run "$P1" wdt-bite); ck "wdt-bite when the type reads 1 (the firmware's, not set): refused, nothing armed" "$(printf '%s\n' "$OUT" | grep -c 'wdt-arm refused: the S2 type reads 0x01, wanted 7')|$(called)" "1|none"
OUT=$(run "$P1" wdt-bite 1); ck "wdt-bite 1 (asked for the warm type, which reads 1): armed" "$(called)" "watchdog -F -T 20 -t 3600 /dev/watchdog1|"
OUT=$(run "$P7" wdt-bite 99); ck "wdt-bite 99: refused as a bad type" "$(printf '%s\n' "$OUT" | grep -c 'TYPE must be 0..15')|$(called)" "1|none"
OUT=$(run "$PNONE" wdt-bite); ck "wdt-bite when the registers cannot be read: refused, and says how to trust the probe" "$(printf '%s\n' "$OUT" | grep -c "wdt-arm refused: the S2 type is not read.*wdt-bite 7 probe")|$(called)" "1|none"
OUT=$(run "$PNONE" wdt-bite 7 probe); ck "wdt-bite 7 probe, registers unread, PMIC device registered: armed on the probe's own read-back" "$(printf '%s\n' "$OUT" | grep -c 'type=0x07-by-probe')|$(called)" "1|watchdog -F -T 20 -t 3600 /dev/watchdog1|"
ck "...that run stays honest: S2 enabled after is unread, said so" "$(printf '%s\n' "$OUT" | grep -c 'after=unread')" 1
OUT=$(run "$P1" wdt-bite 7 probe); ck "wdt-bite 7 probe with a type that reads 1: still refused (probe only stands in for an unread type)" "$(printf '%s\n' "$OUT" | grep -c 'refused: the S2 type reads 0x01')|$(called)" "1|none"
case $INIT in
  *init-wdt1) OUT=$(run "$PNONE" wdt-bite 1 probe); ck "wdt-bite 1 probe: refused (probe trusts type 7 only)" "$(called)" none;;
  *) OUT=$(run "$PNONE" wdt-bite 1 probe); ck "wdt-bite 1 probe, registers unread: armed on the probe's read-back (wdt2: the driver checked the type its dts names)" "$(printf '%s\n' "$OUT" | grep -c 'type=0x01-by-probe')|$(called)" "1|watchdog -F -T 20 -t 3600 /dev/watchdog1|"
     OUT=$(run "$P7" wdt-bite 7 probe 45); ck "wdt-bite 7 probe 45: the timeout is the third argument" "$(printf '%s\n' "$OUT" | grep -c 'timeout=45')|$(called)" "1|watchdog -F -T 45 -t 3600 /dev/watchdog1|"
     OUT=$(run "$P7" wdt-bite 7 probe 200); ck "wdt-bite 7 probe 200: refused, the timeout is 1..127 s" "$(printf '%s\n' "$OUT" | grep -c 'TIMEOUT must be 1..127')|$(called)" "1|none";;
esac
# an arming that did not happen
OUT=$(FAKE_NOARM=1 run "$P7" wdt-bite); ck "an applet that runs but S2 never reads enabled: FAILED, and no success line" "$(printf '%s\n' "$OUT" | grep -c 'wdt-arm FAILED: S2 enabled reads no after the arming')|$(printf '%s\n' "$OUT" | grep -c 'wdt-arm armed')" "1|0"
OUT=$(FAKE_DIE=1 run "$P7" wdt-bite); ck "an applet that exits at once (device not opened): FAILED, and no success line" "$(printf '%s\n' "$OUT" | grep -c 'wdt-arm FAILED: the applet is not running')|$(printf '%s\n' "$OUT" | grep -c 'wdt-arm armed')" "1|0"
OUT=$(FAKE_NOARM=1 run "$P7" wdt-reboot); ck "wdt-reboot after a failed arming does not reboot" "$(called)" "watchdog -F -T 20 -t 3600 /dev/watchdog1|"
OUT=$(FAKE_DIE=1 run "$P7" wdt-reboot); ck "wdt-reboot with a dead applet does not sync or reboot" "$(called)" "watchdog -F -T 20 -t 3600 /dev/watchdog1|"
OUT=$(run "$P7" wdt-reboot)
ck "wdt-reboot arms, then syncs, then reboots with -f, in that order" "$(called)" "watchdog -F -T 20 -t 3600 /dev/watchdog1|sync |reboot -f|"
ck "wdt-reboot says it armed before it rebooted" "$(printf '%s\n' "$OUT" | grep -c 'wdt-reboot: wdt-arm armed watchdog1 type=0x07 timeout=20 s2-enabled-before=no after=yes hold=90s; rebooting cleanly now')" 1
OUT=$(run "$P1" wdt-reboot); ck "wdt-reboot refused (type mismatch) neither arms nor reboots" "$(called)" none
# a hold somebody set is kept, and not removed by us
rm -f "$T/calls"; printf '%s\n' "$P7" > "$T/pon"; : > "$T/hold-person"; DMESG=$DM HOLD=$T/hold-person WDT_SLEEP_HOLD=90 $SH "$T/bin/wdt-arm" 7 > /dev/null 2>&1
/bin/sleep 5; ck "a hold the person set is not removed by the 90 s timer (we did not make it)" "$([ -e "$T/hold-person" ] && echo kept || echo removed)" kept
# the petter's search: the SoC watchdog by identity, whichever number each has, and only when its node exists
sed -n '/^SOCWD=$/,/^done$/p' "$INIT" | sed -e "s#/sys/class/watchdog#$T/s2/class/watchdog#g" -e "s#/dev/#$T/dev/#g" > "$T/petter.sh"
ck "the petter's search was cut out of the init" "$([ "$(wc -l < "$T/petter.sh")" -gt 5 ] && echo yes || echo no)" yes
petter() { # SOC_NUM PMIC_NUM [NODE_NUMBERS...]: a sysfs with the two identities at those numbers (- = absent), /dev nodes as given; prints SOCWD's tail
  rm -rf "$T/s2" "$T/dev"; mkdir -p "$T/dev"
  s=$1; p=$2; shift 2
  if [ "$s" != - ]; then mkdir -p "$T/s2/class/watchdog/watchdog$s"; echo qcom_wdt > "$T/s2/class/watchdog/watchdog$s/identity"; fi
  if [ "$p" != - ]; then mkdir -p "$T/s2/class/watchdog/watchdog$p"; echo "QCOM PM8916 PON WDT" > "$T/s2/class/watchdog/watchdog$p/identity"; fi
  for n in "$@"; do : > "$T/dev/watchdog$n"; done
  FAKE_SLEEP_NOOP=1 $SH -c ". '$T/petter.sh'; printf '%s\n' \"\$SOCWD\"" | sed "s#^$T/dev/##"
}
ck "petter: SoC at watchdog0, PMIC at watchdog1: the SoC one" "$(petter 0 1 0 1)" watchdog0
ck "petter: PMIC at watchdog0, SoC at watchdog1: still the SoC one (the PMIC one is never opened)" "$(petter 1 0 0 1)" watchdog1
ck "petter: the SoC watchdog's /dev node is missing: none chosen (not the PMIC's)" "$(petter 0 1 1)" ""
ck "petter: only the PMIC watchdog exists: none chosen" "$(petter - 0 0)" ""
# a kernel without CONFIG_WATCHDOG_SYSFS attributes: no identity file, only the device link names the watchdog
petter_nid() { # SOC_NUM PMIC_NUM [NODE_NUMBERS...]
  rm -rf "$T/s2" "$T/dev"; mkdir -p "$T/dev"
  s=$1; p=$2; shift 2
  mkdir -p "$T/s2/class/watchdog/watchdog$s" "$T/s2/class/watchdog/watchdog$p"
  ln -s "../../../../17c10000.watchdog" "$T/s2/class/watchdog/watchdog$s/device"
  ln -s "../../../../0-0000:pon@800:watchdog" "$T/s2/class/watchdog/watchdog$p/device"
  for n in "$@"; do : > "$T/dev/watchdog$n"; done
  FAKE_SLEEP_NOOP=1 $SH -c ". '$T/petter.sh'; printf '%s\n' \"\$SOCWD\"" | sed "s#^$T/dev/##"
}
ck "petter without identity files: the SoC one by its device link" "$(petter_nid 0 1 0 1)" watchdog0
ck "petter without identity files: numbers swapped, still the SoC one" "$(petter_nid 1 0 0 1)" watchdog1
petter_pmic_only() { # a PMIC link and node, no SoC node: nothing may be chosen
  rm -rf "$T/s2" "$T/dev"; mkdir -p "$T/dev" "$T/s2/class/watchdog/watchdog0"
  ln -s "../../../../0-0000:pon@800:watchdog" "$T/s2/class/watchdog/watchdog0/device"; : > "$T/dev/watchdog0"
  FAKE_SLEEP_NOOP=1 $SH -c ". '$T/petter.sh'; printf '%s\n' \"\$SOCWD\"" | sed "s#^$T/dev/##"
}
ck "petter without identity files: only the PMIC watchdog: none chosen (never armed)" "$(petter_pmic_only)" ""
rm -rf "$T/nid"; mkdir -p "$T/nid/class/watchdog/watchdog0" "$T/nid/class/watchdog/watchdog1"
ln -s "../../../../17c10000.watchdog" "$T/nid/class/watchdog/watchdog0/device"; ln -s "../../../../0-0000:pon@800:watchdog" "$T/nid/class/watchdog/watchdog1/device"
ck "wdt-state without identity files: both found by the device link" "$(SYS=$T/nid run "$P7" wdt-state | cut -d' ' -f1-3)" "wdt-state pmic=watchdog1 soc=watchdog0"
# the boot text: what remains a text check
awk '/<<.EOS.$/ {h=1} !h {print} /^EOS$/ {h=0}' "$INIT" | grep -v '^[[:space:]]*#' > "$T/boot.sh"
ck "text: boot never runs wdt-bite, wdt-reboot or wdt-arm (the chmod of the files apart)" "$(grep -v '^chmod' "$T/boot.sh" | grep -c -e 'wdt-bite' -e 'wdt-reboot' -e 'wdt-arm')" 0
ck "text: boot never names /dev/watchdog" "$(grep -c '/dev/watchdog' "$T/boot.sh")" 0
ck "text: the only open of a watchdog device at boot is exec 3> of the identity-chosen SOCWD" "$(grep -c 'exec 3> "\$SOCWD"' "$T/boot.sh")|$(grep -c -e 'exec [0-9]*>' "$T/boot.sh")" "1|1"
ck "text: boot starts no busybox watchdog applet" "$(grep -c -e '^[[:space:]]*watchdog ' -e '[;|&(][[:space:]]*watchdog ' "$T/boot.sh")" 0
ck "text: the self-reboot at 90 beats stays and waits for /tmp/hold" "$(grep -c 'while \[ \$n -lt 90 \] || \[ -e /tmp/hold \]' "$T/boot.sh")" 1
ck "text: the banner prints the PON line and the WDT line" "$(grep -c -e 'WALLEYE-BANNER-PON beat' -e 'WALLEYE-BANNER-WDT beat' "$T/boot.sh")" 2
ck "text: the init mounts no disk and writes to none" "$(grep -c -e 'mount[^|]* /dev/\(sd\|block\|mmcblk\|nvme\)' -e '> */dev/sd' -e 'dd [^|]*of=/dev' -e 'mkfs' -e 'blkdiscard' "$INIT")" 0
ck "text: the serial is walleye-first-boot, usb0 is 172.16.43.1, telnetd only there" "$(grep -c -e 'walleye-first-boot' -e 'ifconfig usb0 172.16.43.1' -e 'telnetd -l /bin/sh -b 172.16.43.1' "$INIT")" 3
ck "busybox ash -n on the init" "$($SH -n "$INIT" 2>&1 || echo syntax-error)" ""
if [ $total -eq 0 ]; then echo "pass pmic-wdt-v1-init-selftest host all checks ok"; exit 0; fi
echo "fail pmic-wdt-v1-init-selftest host $total checks failed"; exit 1
