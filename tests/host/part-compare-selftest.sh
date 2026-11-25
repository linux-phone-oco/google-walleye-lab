#!/bin/sh
# part-compare-selftest: part-compare.sh against two fake captures and fake transports; never touches the phone.
H=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0; tab=$(printf '\t')
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
has() { case $2 in *"$3"*) echo "ok   $1";; *) echo "FAIL $1: no '$3' in: $2"; total=$((total+1));; esac; }
hasno() { case $2 in *"$3"*) echo "FAIL $1: found '$3'"; total=$((total+1));; *) echo "ok   $1";; esac; }
mkdir -p "$T/a" "$T/o"
# gpio: Android 4.4 style (base 911, name only) vs mainline style (base 0, label|consumer)
cat > "$T/a/gpio.txt" <<'X'
gpiochip0: GPIOs 911-1060, parent: platform/3400000.pinctrl, msmgpio:
 gpio-914 (                    |touch-reset        ) out hi
 gpio-920 (                    |touch-irq          ) in  hi IRQ
 gpio-930 (                    |only-android       ) out lo
gpiochip1: GPIOs 800-805, parent: platform/pmic.gpios, pmi8998:
 gpio-802 (touch-power) out hi
X
cat > "$T/o/gpio.txt" <<'X'
gpiochip0: GPIOs 0-149, parent: platform/3400000.pinctrl, 3400000.pinctrl:
 gpio-3   (                    |touch-reset        ) out hi
 gpio-9   (                    |touch-irq          ) in  lo IRQ
 gpio-40  (                    |only-ours          ) out lo
X
printf 'a-reg:enabled:1800000\nonly-a:enabled:3300000\nsame:disabled:0\n' > "$T/a/regulator.txt"
printf 'a-reg:enabled:1800000\nsame:disabled:0\nonly-o:enabled:1000000\n' > "$T/o/regulator.txt"
printf './touchscreen@20/reg=00 00 00 20\n./touchscreen@20/phandle=00 00 00 07\n./touchscreen@20/compatible=syna,rmi4-i2c\n./status=okay\n' | tr -d ' ' > "$T/a/dt-i2c.txt"
printf './touchscreen@20/reg=00000020\n./touchscreen@20/phandle=0000009f\n./touchscreen@20/compatible=syna,rmi4-i2c\n./status=disabled\n./clock-frequency=00061a80\n' > "$T/o/dt-i2c.txt"
printf '[    1.234567] rmi4_i2c 0-0020: probed\n[    2.0] same line\n' > "$T/a/dmesg.txt"
printf '[   10.5] same line\n[   11.0] cannot find GPIO chip gpiolib_shared.proxy\n' > "$T/o/dmesg.txt"
printf 'only-android-file\n' > "$T/a/extra.txt"
printf 'x\n' > "$T/a/meta.txt"
OUTP=$(sh "$H/part-compare.sh" diff "$T/a" "$T/o"); RC=$?
ck "diff exits 0 (it informs)" "$RC" 0
sect() { printf '%s\n' "$OUTP" | awk -v n="$1" -v g="$2" '$0=="== " n || index($0,"== " n " ")==1 {on=1; next} /^== /{on=0} on && /^-- /{cur=$0; next} on && index(cur,g)>0 {print}'; }
ck "gpio: Android has, we lack (pin 30 and pmic pin 2)" "$(sect gpio 'Android has' | sed 's/^ *//' | cut -c1-40 | tr '\n' '|')" "3400000.pinctrl/19  out lo only-android|pmic.gpios/2  out hi touch-power|"
has "gpio: we have, Android lacks" "$(sect gpio 'we have')" "3400000.pinctrl/40  out lo only-ours"
has "gpio: same pin, different level" "$(sect gpio 'same key')" "3400000.pinctrl/9  android=[in hi IRQ touch-irq] ours=[in lo IRQ touch-irq]"
hasno "gpio: pin 3 (equal) is not listed" "$(sect gpio 'Android has'; sect gpio 'we have'; sect gpio 'same key')" "touch-reset"
has "regulator: only-a is Android's" "$(sect regulator 'Android has')" "only-a  enabled:3300000"
has "regulator: only-o is ours" "$(sect regulator 'we have')" "only-o  enabled:1000000"
hasno "regulator: equal ones are silent" "$(sect regulator 'Android has'; sect regulator 'we have'; sect regulator 'same key')" "a-reg"
has "dt: clock-frequency is ours only" "$(sect dt-i2c 'we have')" "clock-frequency"
has "dt: status differs" "$(sect dt-i2c 'same key')" "./status  android=[okay] ours=[disabled]"
hasno "dt: phandle is dropped" "$OUTP" "phandle"
has "dmesg: rmi4 line is Android's, timestamps dropped" "$(sect dmesg 'Android has')" "rmi4_i2c 0-0020: probed"
has "dmesg: the proxy error is ours" "$(sect dmesg 'we have')" "gpiolib_shared.proxy"
hasno "dmesg: the same line is silent" "$(sect dmesg 'Android has'; sect dmesg 'we have')" "same line"
has "a file on one side only is not known yet" "$OUTP" "not captured on ours: not known yet"
hasno "meta.txt is not a part file" "$OUTP" "== meta"
mkdir -p "$T/e"; ck "two empty dirs: exit 0" "$(sh "$H/part-compare.sh" diff "$T/e" "$T/e" >/dev/null; echo $?)" 0
ck "bad usage exits 2" "$(sh "$H/part-compare.sh" diff 2>/dev/null; echo $?)" 2
# capture through fake transports
mkdir -p "$T/list" "$T/bin"
printf 'bs%secho a\\nb | tr -d "\\\\n"\n' "$tab" >> "$T/list/fake.list.extra"
printf '# fake part\nhello%secho one=1; echo two=2\nempty%strue\nthird%secho 3=three\nbs%secho a\\\\nb\n' "$tab" "$tab" "$tab" "$tab" > "$T/list/fake.list"
printf '#!/bin/sh\n[ "$1" = shell ] && { shift; cat >/dev/null; exec sh -c "$*"; }\n[ "$1" = devices ] && { echo "List of devices attached"; printf "ABC\\tdevice\\n"; }\nexit 0\n' > "$T/bin/fakeadb"; chmod +x "$T/bin/fakeadb"
export LISTDIR="$T/list"
ADB="$T/bin/fakeadb" sh "$H/part-compare.sh" capture fake android "$T/ca" >/dev/null 2>&1
ck "android capture: answer saved" "$(tr '\n' ' ' < "$T/ca/hello.txt" 2>/dev/null)" "one=1 two=2 "
ck "android capture: empty answer kept as an empty file" "$([ -e "$T/ca/empty.txt" ] && [ ! -s "$T/ca/empty.txt" ] && echo yes)" yes
ck "android capture: later list lines survive adb draining stdin" "$(cat "$T/ca/third.txt" 2>/dev/null)" "3=three"
ck "capture dir is private" "$(stat -c %a "$T/ca")" 700
OURS_SHELL=sh SETTLE=0 CALL_WAIT=0 sh "$H/part-compare.sh" capture fake ours "$T/co" >/dev/null 2>&1
ck "ours capture: answer saved between markers" "$(tr '\n' ' ' < "$T/co/hello.txt" 2>/dev/null)" "one=1 two=2 "
ck "ours capture: a backslash-n in a command reaches the phone as written" "$(cat "$T/co/bs.txt" 2>/dev/null)" "a\\nb"
ck "no list: exit 2" "$(sh "$H/part-compare.sh" capture nopart ours >/dev/null 2>&1; echo $?)" 2
ck "dead transport: exit 2, not a verdict" "$(OURS_SHELL=false SETTLE=0 CALL_WAIT=0 sh "$H/part-compare.sh" capture fake ours "$T/cx" >/dev/null 2>&1; echo $?)" 2
printf 'sn%secho androidboot.serialno=ABCDEF123 mac 00:11:22:33:44:55\n' "$tab" > "$T/list/red.list"
OURS_SHELL=sh SETTLE=0 CALL_WAIT=0 sh "$H/part-compare.sh" capture red ours "$T/cr" >/dev/null 2>&1
ck "ids are redacted" "$(cat "$T/cr/sn.txt")" "androidboot.serialno=REDACTED mac XX:XX:XX:XX:XX:XX"
# the shipped touch list parses: every line has a tab
bad=$(grep -v -e '^#' -e '^$' "$H/part-compare.d/touch.list" | grep -vc "$tab")
ck "touch.list: every command line has NAME<TAB>COMMAND" "$bad" 0
# Android 4.4 header without gpiochipN: or parent:, msm dbg_show lines, relative dirs, unclaimed pins
mkdir -p "$T/r/a" "$T/r/o"
printf 'GPIOs 911-1060, platform/3400000.pinctrl, msmgpio:\n gpio3   : out 1 2mA no pull\n' > "$T/r/a/gpio.txt"
printf 'gpiochip0: GPIOs 0-149, parent: platform/3400000.pinctrl, 3400000.pinctrl:\n gpio3   : out 0 2mA no pull\n' > "$T/r/o/gpio.txt"
printf 'pin 1 (gpio1): UNCLAIMED\npin 2 (gpio2): 1-0020 3400000.pinctrl:x function gpio group gpio2\n' > "$T/r/a/pinmux.txt"; : > "$T/r/o/pinmux.txt"
printf 'only\n' > "$T/r/o/zz.txt"
R=$(cd "$T/r" && sh "$H/part-compare.sh" diff a o)
has "msm gpio dbg_show format, 4.4 header: same pin, different level" "$R" "3400000.pinctrl/gpio3  android=[out 1 2mA no pull] ours=[out 0 2mA no pull]"
hasno "unclaimed pins are not listed" "$R" "gpio1"
has "pinmux: claimed pin is Android's" "$R" "gpio2"
has "relative dirs: a file only in the second dir is named" "$R" "== zz"
# the real Android b capture of touch (research walleye-android-touch-capture): the normalisers read what the phone printed
FX=$H/../fixtures/phone-text; mkdir -p "$T/ra" "$T/ro"
cp "$FX/part-compare-touch-android-b-gpio-2026-11-25.cap" "$T/ra/gpio.txt"; cp "$FX/part-compare-touch-android-b-regulator-2026-11-25.cap" "$T/ra/regulator.txt"; cp "$FX/part-compare-touch-android-b-dt-i2c-2026-11-25.cap" "$T/ra/dt-i2c.txt"
printf 'gpiochip0: GPIOs 0-149, parent: platform/3400000.pinctrl, 3400000.pinctrl:\n gpio3   : in  0 6mA no pull\n' > "$T/ro/gpio.txt"; printf 'pm8998_s4:disabled:1800000\n' > "$T/ro/regulator.txt"; : > "$T/ro/dt-i2c.txt"
RR=$(cd "$T" && sh "$H/part-compare.sh" diff ra ro)
has "real Android gpio text: same pin, different level" "$RR" "3400000.pinctrl/gpio3  android=[in 1 6mA no pull] ours=[in 0 6mA no pull]"
has "real Android regulator text: state differs" "$RR" "pm8998_s4  android=[enabled:1800000]"
has "real Android dt-i2c text: a node only Android has" "$RR" "./reg"
unset LISTDIR
[ $total -eq 0 ] && echo "part-compare-selftest: all ok" || { echo "part-compare-selftest: $total failed"; exit 1; }
