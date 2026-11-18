#!/bin/sh
# walleye-build-gpio-diag-kernel, run on the lab host: does a boot image make the kernel say, at boot, why the touch power gpio defers?
# Passes only when (1) the image's kernel has the pr_debug sites compiled in (gpiolib-of's format string "can't parse '%s' property of node" is in the Image: it exists only with CONFIG_DYNAMIC_DEBUG or DEBUG,
# so a kernel with only the debugfs core fails) and (2) its command line has a dyndbg="..." that turns on, by the paths the kernel matches (a dyndbg file query is a whole-string match against the source
# path, so "pinctrl/core.c" would match nothing), drivers/gpio/gpiolib.c, drivers/gpio/gpiolib-of.c, drivers/pinctrl/core.c and drivers/pinctrl/devicetree.c, and (3) keeps the image's other words
# (ignore_loglevel, the dtbo index). Says nothing of the phone's dmesg (that is the keeper's try). Prints `pass|fail|skip walleye-gpio-diag-boot kernel DETAIL`; exit 0 only on pass. Writes nothing but a temp dir.
# Environment: IMG (default the built z20 image).
IMG=${IMG:-$HOME/Projects/walleye-build/images-z20/boot-z20.img}
say() { echo "$1 walleye-gpio-diag-boot kernel $2"; [ "$1" = pass ]; exit $?; }
command -v unpack_bootimg >/dev/null || say skip "unpack_bootimg not installed"
[ -r "$IMG" ] || say skip "need IMG=$IMG readable"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
INFO=$(unpack_bootimg --boot_img "$IMG" --out "$T/u") || say fail "$IMG does not unpack"
CMD=$(printf '%s\n' "$INFO" | sed -n 's/^command line args: *//p' | head -n1)
n=$(zcat "$T/u/kernel" 2>/dev/null | LC_ALL=C grep -a -c "can't parse '%s' property of node")
[ "$n" -ge 1 ] || say fail "the kernel has no pr_debug sites in gpiolib-of (no dynamic debug)"
Q=$(printf '%s\n' "$CMD" | sed -n 's/.*dyndbg="\([^"]*\)".*/\1/p')
[ -n "$Q" ] || say fail "the command line has no quoted dyndbg=\"...\": '$CMD'"
for f in drivers/gpio/gpiolib.c drivers/gpio/gpiolib-of.c drivers/pinctrl/core.c drivers/pinctrl/devicetree.c; do
  case "; $Q;" in *"; file $f +p;"*) ;; *) say fail "dyndbg does not turn on $f (exact path, +p): '$Q'" ;; esac
done
for w in ignore_loglevel androidboot.dtbo_idx=3; do
  case " $CMD " in *" $w "*) ;; *) say fail "the command line lost $w: '$CMD'" ;; esac
done
say pass "pr_debug sites are in the kernel and dyndbg turns on the files by their kernel paths: $Q"
