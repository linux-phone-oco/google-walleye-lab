#!/bin/sh
# The modem image's /init on stdout: slpi's init with its window-end line (the one that starts 'sync; reboot -f') replaced, in place, by the one line of window-end
# (decision 2026-11-27-walleye-modem-image-ends-by-sysrq-b-with-ipa). Stops when slpi's init has no such line, or more than one. Used by build-modem-ramdisk.sh and tests/host/modem-ramdisk.sh.
#   modem-init.sh SLPI_INIT WINDOW_END
# The init's header comment (line 8) still says 'reboot -f, sysrq b behind it': true of slpi, not of the modem image, which ends with sysrq b first.
set -eu
SLPI=${1:?usage: modem-init.sh SLPI_INIT WINDOW_END}
END=${2:?usage: modem-init.sh SLPI_INIT WINDOW_END}
[ "$(wc -l < "$END")" = 1 ] || { echo "modem-init: $END is not one line" >&2; exit 1; }
[ "$(grep -c '^  sync; reboot -f; sleep 5; echo b > /proc/sysrq-trigger$' "$SLPI")" = 1 ] || { echo "modem-init: $SLPI has not exactly one window end" >&2; exit 1; }
NEW=$(cat "$END") awk '/^  sync; reboot -f; sleep 5; echo b > \/proc\/sysrq-trigger$/ { print ENVIRON["NEW"]; next } { print }' "$SLPI"
