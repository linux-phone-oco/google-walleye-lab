#!/bin/sh
# SPDX-License-Identifier: MIT OR Apache-2.0
# The proof for the WDT0-read init (backlog walleye-wdt0-read-image; proof test walleye-wdt0-state-at-boot's host half), run on the lab host with no phone:
# build/initramfs-root-wdt0/init is the production init plus WDT0 blocks that act only with the word walleye.wdt0 on the command line.
#   wdt0-init.sh
# (W0) text: the WDT0 blocks are paired; cut, the init is build/initramfs-root-production/init line for line; the read block comes before the production petters block (the FIRST act), the copy block after it;
#      the only devmem call in the init is `devmem "$addr" 32` (two arguments: a read; devmem is never given a value), and it names no other register than the four of the list.
# (W1) the four registers: each address is the WDT0 base 0x17817000 plus the offset the Android driver defines (WDT0_EN 0x08, WDT0_STS 0x0C, WDT0_BARK_TIME 0x10, WDT0_BITE_TIME 0x14: android:drivers/soc/qcom/watchdog_v2.c@113bd1cfa6f5 lines 39-42,
#      read in research walleye-android-watchdog-read-fix).
# The read block run with a stand-in `devmem` that records its arguments (the stand-in answers like busybox's devmem: 0x followed by 8 hex digits), /proc/cmdline, /proc/uptime, /dev/kmsg and /tmp replaced by temp files:
# (W2) with the word: devmem is called four times, in the list's order, each `ADDRESS 32` and nothing else; the kernel log and /tmp/wdt0.log hold `walleye-wdt0: NAME ADDRESS VALUE` for each; (W3) a failed read says FAILED and devmem's words, and the next register is still read;
# (W4) without the word, and with a word that only contains it (walleye.wdt0x), devmem is never called and no log is made; (W5) the copy block, run with the log present, puts it byte for byte at /.walleye/wdt0 and /run/walleye/wdt0 of the new root; with no log it copies nothing.
# INIT: another init to judge, so the test is seen failing on one without the blocks (the production init). Prints `pass|fail walleye-wdt0-init DETAIL`; exit 0 only on pass. Writes only a temp dir.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
INIT=${INIT:-$HERE/build/initramfs-root-wdt0/init}
PROD=$HERE/build/initramfs-root-production/init
say() { echo "$1 walleye-wdt0-init $2"; [ "$1" = pass ]; exit $?; }
[ -r "$INIT" ] || say fail "no init at $INIT"
[ -r "$PROD" ] || say fail "no production init at $PROD"
B=$(grep -c '^# WDT0-BLOCK-BEGIN' "$INIT"); E=$(grep -c '^# WDT0-BLOCK-END' "$INIT")
[ "$B" = "$E" ] && [ "$B" -ge 1 ] || say fail "W0: the WDT0 blocks are not paired, or there are none ($B begin, $E end)"
[ "$(sed '/^# WDT0-BLOCK-BEGIN/,/^# WDT0-BLOCK-END/d' "$INIT")" = "$(cat "$PROD")" ] || say fail "W0: without its WDT0 blocks the init is not build/initramfs-root-production/init, line for line"
T=$(mktemp -d) || say fail "no temp dir"; trap 'rm -rf "$T"' EXIT
sed -n '/^# WDT0-BLOCK-BEGIN (the read: the first act)/,/^# WDT0-BLOCK-END/p' "$INIT" | grep -v '^# WDT0-BLOCK' > "$T/read"
sed -n '/^# WDT0-BLOCK-BEGIN (the values stay where the lab reads them)/,/^# WDT0-BLOCK-END/p' "$INIT" | grep -v '^# WDT0-BLOCK' > "$T/copy"
[ -s "$T/read" ] || say fail "W0: no read block"
[ -s "$T/copy" ] || say fail "W0: no copy block"
r=$(grep -n '^# WDT0-BLOCK-BEGIN (the read' "$INIT" | cut -d: -f1); p=$(grep -n '^# PRODUCTION-BLOCK-BEGIN (1:' "$INIT" | cut -d: -f1)
c=$(grep -n '^# WDT0-BLOCK-BEGIN (the values' "$INIT" | cut -d: -f1); p2=$(grep -n '^# PRODUCTION-BLOCK-END$' "$INIT" | tail -n1 | cut -d: -f1)
[ "$r" -lt "$p" ] || say fail "W0: the read block is not before the petters block: it is not the first act"
[ "$c" -gt "$p2" ] || say fail "W0: the copy block is not after the petters block"
dv=$(grep -n '^mount -t devtmpfs' "$INIT" | head -n1 | cut -d: -f1); ml=$(grep -n '^if \[ -f /modules.list \]' "$INIT" | head -n1 | cut -d: -f1)
[ -n "$dv" ] && [ -n "$ml" ] && [ "$r" -gt "$dv" ] && [ "$r" -gt "$ml" ] || say fail "W0: the read block is not after the mounts of /proc, /sys, /dev and the module load: /proc/cmdline and /dev/mem would not be there"
[ "$(grep -v '^ *#' "$INIT" | grep -c devmem)" = 2 ] && [ "$(grep -c 'devmem "\$addr" 32 < /dev/null' "$INIT")" = 1 ] || say fail "W0: the init has a devmem token that is not the one read \`devmem \"\$addr\" 32\` (and the words in the first echo)"
[ "$(cat "$T/read" "$T/copy" | grep -v '^ *#' | grep -c -e /dev/watchdog -e /dev/mem)" = 0 ] || say fail "W0: a WDT0 block names /dev/watchdog or /dev/mem"
# W1: the list, against the Android driver's offsets
BASE=$((0x17817000))
for want in EN:8 STS:12 BARK_TIME:16 BITE_TIME:20; do
  n=${want%%:*}; o=${want#*:}
  a=$(sed -n "s/.*[ ]$n:\(0x[0-9a-f]*\).*/\1/p" "$T/read" | head -n1)
  [ -n "$a" ] || say fail "W1: no address for $n in the list"
  [ $((a)) -eq $((BASE + o)) ] || say fail "W1: $n is $a, the WDT0 base 0x17817000 plus $o is $(printf '0x%x' $((BASE + o)))"
done
[ "$(sed -n 's/^  for r in \(.*\); do$/\1/p' "$T/read" | tr ' ' '\n' | grep -c .)" = 4 ] || say fail "W1: the list is not four registers"
mkdir "$T/bin"
cat > "$T/bin/devmem" <<EOS
#!/bin/sh
echo "\$*" >> $T/devmem.args
case "\$1" in 0x1781700c) [ -e $T/failing ] && { echo "devmem: mmap: Operation not permitted" >&2; exit 1; };; esac
echo "0x000000AB"
EOS
chmod +x "$T/bin/devmem"
echo "7.25 0.00" > "$T/uptime"
sed -e "s#/proc/cmdline#$T/cmdline#g" -e "s#/proc/uptime#$T/uptime#g" -e "s#/tmp/wdt0.log#$T/wdt0.log#g" -e "s#> /dev/kmsg#>> $T/kmsg#g" "$T/read" > "$T/read.run"
try() { # CMDLINE FAILING(yes|no)
  rm -f "$T/devmem.args" "$T/kmsg" "$T/wdt0.log" "$T/failing"; : > "$T/kmsg"
  printf '%s\n' "$1" > "$T/cmdline"; [ "$2" != yes ] || : > "$T/failing"
  PATH=$T/bin:$PATH sh "$T/read.run"
}
try "console=ttyMSM0 walleye.root=system_a walleye.wdt0" no
[ "$(cat "$T/devmem.args")" = "0x17817008 32
0x1781700c 32
0x17817010 32
0x17817014 32" ] || say fail "W2: devmem was not called four times, in order, as \`ADDRESS 32\`: $(tr '\n' '|' < "$T/devmem.args")"
for L in "walleye-wdt0: EN 0x17817008 0x000000AB" "walleye-wdt0: STS 0x1781700c 0x000000AB" "walleye-wdt0: BARK_TIME 0x17817010 0x000000AB" "walleye-wdt0: BITE_TIME 0x17817014 0x000000AB"; do
  grep -qxF "$L" "$T/wdt0.log" || say fail "W2: /tmp/wdt0.log has no line [$L]"
  grep -qxF "$L" "$T/kmsg" || say fail "W2: the kernel log has no line [$L]"
done
grep -qxF "walleye-wdt0: first read at uptime 7 s" "$T/wdt0.log" || say fail "W2: the log does not say the uptime of the first read"
try "walleye.root=system_a walleye.wdt0" yes
[ "$(wc -l < "$T/devmem.args")" = 4 ] || say fail "W3: a failed read stopped the others"
grep -qxF "walleye-wdt0: STS 0x1781700c FAILED devmem: mmap: Operation not permitted" "$T/wdt0.log" || say fail "W3: the failed read is not said with devmem's words: $(cat "$T/wdt0.log")"
grep -qxF "walleye-wdt0: BITE_TIME 0x17817014 0x000000AB" "$T/wdt0.log" || say fail "W3: the register after the failed one was not read"
for c in "walleye.root=system_a" "walleye.root=system_a walleye.wdt0x" "xwalleye.wdt0 walleye.root=system_a"; do
  try "$c" no
  [ ! -e "$T/devmem.args" ] && [ ! -e "$T/wdt0.log" ] || say fail "W4: devmem ran or a log was made for the command line [$c]"
done
mkdir -p "$T/n/.walleye" "$T/n/run/walleye"
sed -e "s#/tmp/wdt0.log#$T/wdt0.log#g" -e "s#/newroot#$T/n#g" "$T/copy" > "$T/copy.run"
printf 'a\nb\n' > "$T/wdt0.log"; ( say() { :; }; . "$T/copy.run" )
cmp -s "$T/wdt0.log" "$T/n/.walleye/wdt0" && cmp -s "$T/wdt0.log" "$T/n/run/walleye/wdt0" || say fail "W5: the log is not at /.walleye/wdt0 and /run/walleye/wdt0 of the new root, byte for byte"
rm -f "$T/wdt0.log" "$T/n/.walleye/wdt0" "$T/n/run/walleye/wdt0"; ( say() { :; }; . "$T/copy.run" )
[ ! -e "$T/n/.walleye/wdt0" ] && [ ! -e "$T/n/run/walleye/wdt0" ] || say fail "W5: with no log the copy block made a file"
say pass "W0-W5: the four registers WDT0 base+0x08/0x0c/0x10/0x14 are read first, read only, said to the kernel log, kept in /tmp/wdt0.log and copied to the new root; no word, no read; blocks cut, the production init"
