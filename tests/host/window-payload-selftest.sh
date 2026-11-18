#!/bin/sh
# walleye-window-payload-selftest, on the lab host: the init's fill_window (extracted from build/initramfs-vendor5/init-vfx between its '>>> fill' and '<<< fill' marks) is run on test sources for each image letter
# a, b, c and d into a 2 MiB window file, and tests/host/window-payload-read.sh (VF_FILE) must take every part back byte for byte; also a window cut to its room (TRUNC=1), a source that does not exist (rc shown), an empty header
# (killed in the first read), a foreign window and a redaction. Prints ok/FAIL per check; exit 1 on a FAIL. Host only; the phone is not involved.
#
#   tests/host/window-payload-selftest.sh [REPO_ROOT]
R=${1:-$(cd "$(dirname "$0")/../.." && pwd)}
INIT=$R/build/initramfs-vendor5/init-vfx
RD=$R/tests/host/window-payload-read.sh
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; pass=$((pass+1)); else echo "FAIL $1: got '$2', want '$3'"; fail=$((fail+1)); fi; }
sed -n '/^# >>> fill/,/^# <<< fill/p' "$INIT" > "$T/fill.sh"
ck "the fill section was found in the init" "$([ "$(wc -l < "$T/fill.sh")" -gt 20 ] && echo yes || echo no)" yes
# test sources: a real small device tree, text files, a cmdline with a serial, a debugfs-like tree
mkdir -p "$T/src" "$T/dbg/usb/ci_hdrc" "$T/dbg/a800000.dwc3"
printf '/dts-v1/;\n/ { model = "test"; compatible = "t,t"; chosen { }; };\n' > "$T/src/t.dts"
dtc -I dts -O dtb -o "$T/src/fdt" "$T/src/t.dts" 2>/dev/null
printf '00000000-00ffffff : System RAM\n' > "$T/src/iomem"
printf '           CPU0\n 12:   1 GIC  uart\n' > "$T/src/irq"
printf 'console=ttyMSM0 androidboot.serialno=ABC123XYZ foo=bar mac=00:11:22:33:44:55\n' > "$T/src/cmdline"
head -c 300000 /dev/zero | tr '\000' 'c' > "$T/src/clk"
printf 'vreg a 1\nvreg b 2\n' > "$T/src/reg"
echo "state: ok" > "$T/dbg/usb/ci_hdrc/state"; echo "regs" > "$T/dbg/a800000.dwc3/regdump"
mkwin() { # LETTER OUT [env...]: a 2 MiB window as the init would make it, fill_window run with the test sources
  L=$1; W=$2; shift 2
  dd if=/dev/zero of="$W" bs=1024 count=2048 2>/dev/null
  (
    mkdir -p "$T/vt"; rm -f "$T"/vt/vf-*
    VT="$T/vt"; LOGF=$W; VF=$L
    VF_FDT=$T/src/fdt; VF_IOMEM=$T/src/iomem; VF_IRQ=$T/src/irq; VF_CMDLINE=$T/src/cmdline; VF_CLK=$T/src/clk; VF_REG=$T/src/reg; VF_DBG=$T/dbg
    for kv in "$@"; do eval "$kv"; done
    . "$T/fill.sh"
    fill_window
  )
}
run() { # prints the reader's output last line; OUT dir in $T/out-$1
  VF_FILE=$2 VF_OUT=$T/out-$1 sh "$RD" 2>&1 | tail -n1
}
mkwin a "$T/wa"; RA=$(run a "$T/wa")
ck "vfa: the reader passes" "$(echo "$RA" | cut -d' ' -f1)" pass
ck "vfa: fdt.dtb is the source byte for byte" "$(cmp "$T/out-a/fdt.dtb" "$T/src/fdt" 2>&1 && echo same)" same
ck "vfa: fdt.dts is readable (model = \"test\")" "$(grep -c 'model = "test"' "$T/out-a/fdt.dts" 2>/dev/null)" 1
ck "vfa: iomem and interrupts are the sources" "$(cmp "$T/out-a/iomem" "$T/src/iomem" && cmp "$T/out-a/interrupts" "$T/src/irq" && echo same)" same
ck "vfa: the cmdline lost its serial and its MAC" "$(grep -c -e 'serialno=REDACTED' -e 'XX:XX:XX:XX:XX:XX' "$T/out-a/cmdline")" 1
ck "vfa: ... and has neither the serial nor the MAC" "$(grep -c -e ABC123XYZ -e '00:11:22:33:44:55' "$T/out-a/cmdline")" 0
mkwin b "$T/wb"; RB=$(run b "$T/wb")
ck "vfb: passes, not truncated (a 300000-byte clk_summary fits)" "$(echo "$RB" | cut -d' ' -f1)$(echo "$RB" | grep -c TRUNCATED)" pass0
ck "vfb: clk_summary is the source byte for byte" "$(cmp "$T/out-b/clk_summary" "$T/src/clk" && echo same)" same
mkwin c "$T/wc"; RC=$(run c "$T/wc")
ck "vfc: passes, regulator_summary is the source" "$(echo "$RC" | cut -d' ' -f1)$(cmp "$T/out-c/regulator_summary" "$T/src/reg" && echo same)" passsame
mkwin d "$T/wd"; RDD=$(run d "$T/wd")
ck "vfd: passes, usb has both directories' files, named" "$(echo "$RDD" | cut -d' ' -f1)$(grep -c -e '^== usb/ci_hdrc/state' -e '^== a800000.dwc3/regdump' "$T/out-d/usb")" pass2
# the room: a source over 2 MiB - 4 KiB - 512 is cut, says TRUNC=1, and the reader says TRUNCATED and passes
head -c 2200000 /dev/zero | tr '\000' 'x' > "$T/src/big"
mkwin b "$T/wt" VF_CLK=$T/src/big; RT=$(run t "$T/wt")
ck "a payload over the room: header says TRUNC=1, reader passes and says TRUNCATED" "$(head -n1 "$T/out-t/header.txt" | grep -c 'TRUNC=1')$(echo "$RT" | cut -d' ' -f1)$(echo "$RT" | grep -c TRUNCATED)" 1pass1
ck "... and the payload is exactly the room" "$(stat -c %s "$T/out-t/clk_summary")" $((2097152 - 4096 - 512))
# a source that does not exist: its rc is not 0, the reader fails and names it
mkwin c "$T/wm" VF_REG=$T/src/nothere; RM=$(run m "$T/wm")
ck "a missing source: the reader fails and names regulator_summary:rc=" "$(echo "$RM" | cut -d' ' -f1)$(echo "$RM" | grep -c 'regulator_summary:rc=')" fail1
# killed in the first read: only the first header (PARTS=0), nothing else
printf 'WALLEYE-VFa LEN=0000000000 PARTS=0 TRUNC=0\n' | dd of="$T/wk" bs=512 conv=sync 2>/dev/null; dd if=/dev/zero of="$T/wk" bs=1024 count=2048 seek=1 conv=notrunc 2>/dev/null
RK=$(run k "$T/wk")
ck "a header with no part: the reader fails and says killed or hung" "$(echo "$RK" | cut -d' ' -f1)$(echo "$RK" | grep -c 'killed or hung')" fail1
# a window that is not ours
dd if=/dev/zero of="$T/wz" bs=1024 count=2048 2>/dev/null; RZ=$(run z "$T/wz")
ck "an empty window: the reader fails 'header is not ours'" "$(echo "$RZ" | cut -d' ' -f1)$(echo "$RZ" | grep -c 'header is not ours')" fail1
# the beat block is read and shown
mkwin a "$T/wbt"; printf 'WALLEYE-VFa BEAT=7 UP=41.2\n' | dd of="$T/wbt" bs=4096 seek=511 conv=sync,notrunc 2>/dev/null; RBT=$(run bt "$T/wbt")
ck "the beat is in the reader's line" "$(echo "$RBT" | grep -c 'BEAT=7')" 1
ck "the init calls fill_window once" "$(grep -c '^fill_window$' "$INIT")" 1
echo "window-payload-selftest: $pass ok, $fail failed"
[ $fail -eq 0 ]
