#!/bin/sh
# Proves the phone-side scripts that put ModemManager on the modem image (item walleye-modem-mm-chroot-in-modem-up) on a fake phone: build/initramfs-modem/rootfs-recv and mm-up run for real, their tools
# (mount, mountpoint, nc, chroot, pidof, setsid) are stand-ins that only write down what they were asked, and the "pushed" stream is a real tar of the shape build/modem-mm-rootfs.nix makes (env/bin/ModemManager, env/bin/dbus-daemon).
# Passes when: rootfs-recv mounts a tmpfs, receives on 9001, unpacks, removes the tar and says done; mm-up binds proc, sys, dev and the firmware folder, THEN starts dbus, THEN ModemManager (in that order, the log is read);
# and each stop starts nothing: no rootfs, no qrtr-ns (modem-up not run), a second start, a stream that is not a tar, a second receive. The words are the scripts' own (the Rust side reads `rootfs-recv: done` and `mm-up: dbus and ModemManager run`).
# The image's size (under 32 MiB) is tests/host/modem-ramdisk.sh's and the image's own record: the two scripts add ~3 KiB to the cpio.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
SRC="$HERE/../../build/initramfs-modem"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
bad=0
ok() { echo "ok   $1"; }
no() { echo "FAIL $1"; bad=1; }
S="$T/stub"; mkdir -p "$S"
# the stand-in tools: each appends one line to $LOG; state of the fake phone is in files under $T/state
cat > "$S/mount" <<'X'
#!/bin/sh
echo "mount $*" >> "$LOG"
[ "$1" != -t ] || touch "$STATE/mounted"
X
cat > "$S/mountpoint" <<'X'
#!/bin/sh
[ -e "$STATE/mounted" ]
X
cat > "$S/nc" <<'X'
#!/bin/sh
echo "nc $*" >> "$LOG"
cat "$STREAM"
X
cat > "$S/setsid" <<'X'
#!/bin/sh
exec "$@"
X
cat > "$S/pidof" <<'X'
#!/bin/sh
grep -qx "$1" "$STATE/running"
X
cat > "$S/chroot" <<'X'
#!/bin/sh
echo "chroot $*" >> "$LOG"
p=$(basename "$2")
# the real dbus-daemon refuses --system together with --config-file (run 3, research walleye-modem-mm-first-try-run-3-dbus-system-and-config-file: its dbus.log said so), starts nothing and makes no socket
if [ "$p" = dbus-daemon ]; then
  case " $* " in *" --system "*) case " $* " in *" --config-file="*) echo "--config-file specified but configuration file /etc/dbus-1/system.conf already requested" >&2; exit 1 ;; esac ;; esac
fi
echo "$p" >> "$STATE/running"
[ "$p" != dbus-daemon ] || touch "$1/run/dbus/system_bus_socket"
X
cat > "$S/sleep" <<'X'
#!/bin/sh
/bin/sleep 0.05
X
chmod 755 "$S"/*
# the pushed stream: a tar of the build's shape
mkdir -p "$T/tree/nix/store/x/bin" "$T/tree/etc"; ln -s /nix/store/x "$T/tree/env"
printf 'root:x:0:0:root:/:/bin/sh\nmessagebus:x:100:100:dbus:/:/bin/false\n' > "$T/tree/etc/passwd"
printf '#!/bin/sh\n' > "$T/tree/nix/store/x/bin/ModemManager"; cp "$T/tree/nix/store/x/bin/ModemManager" "$T/tree/nix/store/x/bin/dbus-daemon"; chmod 755 "$T/tree/nix/store/x/bin"/*
tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner -cf "$T/good.tar" -C "$T/tree" .
printf 'not a tar' > "$T/junk.tar"
fresh() { # a new fake phone: its own root, state, log
  rm -rf "$T/root" "$T/state" "$T/logs"; mkdir -p "$T/state" "$T/logs"; : > "$T/state/running"; : > "$T/log"
}
run() { # SCRIPT STREAM -> output in $OUT, status in $RC
  OUT=$(LOG="$T/log" STATE="$T/state" STREAM="$2" MM_PATH="$S:$PATH" MM_ROOT="$T/root" MM_LOGS="$T/logs" MM_TAR="$T/pushed.tar" MM_WAIT=3 sh "$SRC/$1" 2>&1) && RC=0 || RC=$?
}
line() { grep -n "$1" "$T/log" | head -1 | cut -d: -f1; }

fresh; run rootfs-recv "$T/good.tar"
[ $RC = 0 ] && case "$OUT" in "rootfs-recv: done,"*) ok "rootfs-recv unpacks the pushed rootfs and says done" ;; *) no "rootfs-recv said: $OUT" ;; esac || no "rootfs-recv failed ($RC): $OUT"
[ -x "$T/root/nix/store/x/bin/ModemManager" ] && [ ! -e "$T/pushed.tar" ] && ok "the rootfs is unpacked and the received tar is removed" || no "unpacked rootfs or tar removal"
[ "$(line 'mount -t tmpfs')" -lt "$(line 'nc -l -p 9001')" ] && ok "the tmpfs is mounted before the push is received on port 9001" || no "mount/nc order"
run rootfs-recv "$T/good.tar"; [ $RC != 0 ] && case "$OUT" in *"received once only"*) ok "a second receive stops and says so" ;; *) no "second receive said: $OUT" ;; esac || no "a second receive passed"
fresh; run rootfs-recv "$T/junk.tar"; [ $RC != 0 ] && case "$OUT" in *"rootfs-recv: tar failed"*) ok "a stream that is not a tar says tar failed (the word the Rust side reads)" ;; *) no "junk said: $OUT" ;; esac || no "junk passed"

# mm-up: the rootfs unpacked and qrtr-ns running (as modem-up leaves them)
fresh; run rootfs-recv "$T/good.tar"; echo qrtr-ns >> "$T/state/running"; : > "$T/log"
run mm-up ""
[ $RC = 0 ] && case "$OUT" in "mm-up: dbus and ModemManager run in"*) ok "mm-up says dbus and ModemManager run" ;; *) no "mm-up said: $OUT" ;; esac || no "mm-up failed ($RC): $OUT"
b1=$(line 'mount --bind /proc'); b2=$(line 'mount --bind /sys'); b3=$(line 'mount --bind /dev'); b4=$(line 'mount --bind /lib/firmware'); d=$(line 'dbus-daemon'); m=$(line 'ModemManager')
[ -n "$b1" ] && [ -n "$b4" ] && [ -n "$d" ] && [ -n "$m" ] && [ "$b1" -lt "$b2" ] && [ "$b2" -lt "$b3" ] && [ "$b3" -lt "$b4" ] && [ "$b4" -lt "$d" ] && [ "$d" -lt "$m" ] && ok "binds (proc, sys, dev, firmware), then dbus, then ModemManager" || no "the order is not binds, dbus, ModemManager: $(cat "$T/log")"
grep -q "ModemManager --debug" "$T/log" && grep -q "^messagebus" "$T/root/etc/passwd" && ok "ModemManager runs with --debug; the pushed rootfs has root and messagebus" || no "ModemManager's flags or /etc/passwd"
grep -q "dbus-daemon .*--config-file=/env/share/dbus-1/system.conf" "$T/log" && grep -q "dbus-uuidgen --ensure=/etc/machine-id" "$T/log" && [ "$(line dbus-uuidgen)" -lt "$d" ] && ok "dbus starts from the store config, after a machine-id" || no "dbus-daemon without --config-file or no machine-id: $(cat "$T/log")"
run mm-up ""; [ $RC != 0 ] && case "$OUT" in *"started once only"*) ok "a second mm-up stops and says so" ;; *) no "second mm-up said: $OUT" ;; esac || no "second mm-up passed"
fresh; echo qrtr-ns >> "$T/state/running"; run mm-up ""
[ $RC != 0 ] && case "$OUT" in *"push the rootfs first"*) [ ! -s "$T/log" ] && ok "no rootfs: mm-up stops and starts nothing" || no "mm-up acted without a rootfs" ;; *) no "no-rootfs said: $OUT" ;; esac || no "no-rootfs passed"
fresh; run rootfs-recv "$T/good.tar"; : > "$T/log"; run mm-up ""
[ $RC != 0 ] && case "$OUT" in *"run modem-up first"*) [ ! -s "$T/log" ] && ok "no qrtr-ns: mm-up stops and starts nothing" || no "mm-up acted without qrtr-ns" ;; *) no "no-qrtr-ns said: $OUT" ;; esac || no "no-qrtr-ns passed"
# dbus that never makes its socket: the stop names the log
fresh; run rootfs-recv "$T/good.tar"; echo qrtr-ns >> "$T/state/running"; : > "$T/log"
printf '#!/bin/sh\necho "chroot $*" >> "$LOG"\necho "dbus: fake complaint" >&2\n' > "$S/chroot"; run mm-up ""
[ $RC != 0 ] && case "$OUT" in *"no socket after 3 s (see $T/logs/dbus.log)"*) ok "dbus without a bus socket stops mm-up before ModemManager" ;; *) no "no-socket said: $OUT" ;; esac || no "no-socket passed"
! grep -q "ModemManager --debug" "$T/log" || no "ModemManager started without a bus"
# the stop prints dbus's log after its words, so the run's mm-up.txt keeps what the reboot would take
case "$OUT" in *"== dbus.log"*"dbus: fake complaint"*) ok "the stop prints dbus.log (kept by the run before the way back)" ;; *) no "the stop did not print dbus.log: $OUT" ;; esac
# ModemManager that exits at once: its log is printed too
fresh; run rootfs-recv "$T/good.tar"; echo qrtr-ns >> "$T/state/running"; : > "$T/log"
printf '#!/bin/sh\necho "chroot $*" >> "$LOG"\np=$(basename "$2")\n[ "$p" != dbus-daemon ] || { touch "$1/run/dbus/system_bus_socket"; echo dbus-daemon >> "$STATE/running"; }\n[ "$p" != ModemManager ] || echo "mm: fake exit" >&2\n' > "$S/chroot"; run mm-up ""
[ $RC != 0 ] && case "$OUT" in *"ModemManager is not running"*"== ModemManager.log"*"mm: fake exit"*) ok "a ModemManager that stopped: its log is printed after the words" ;; *) no "mm stop said: $OUT" ;; esac || no "mm stop passed"
[ $bad -eq 0 ] || { echo "modem-boot-selftest: failed" >&2; exit 1; }
echo "pass modem-boot-selftest"
