# Where the UFS runs' 180 s comes from (item walleye-ufs-180s-read)

Read on the lab host, 2026-11-24, no phone. Counted from the files named; nothing here is inferred unless it says so.

## (1) The matched image

`~/Projects/walleye-build/NOT-TO-BOOT-ufs-on-images-ufs-match2/NOT-TO-BOOT-ufs-on-ufs-match.img`, sha256 4610778...df5 (the one
device.toml `[images.ufs-match]` records), unpacked read-only with `unpack_bootimg` into a scratch folder:

- header cmdline: `clk_ignore_unused ignore_loglevel panic=5 androidboot.hardware=walleye androidboot.console=ttyMSM0
  androidboot.dtbo_idx=3 dyndbg=...` - no 180, no `walleye.hold`, no watchdog word.
- ramdisk (xz cpio): its `/init` is byte-for-byte `build/initramfs-mainline/init-z14` (`cmp`: same; build-ufs-match.sh takes z21's
  ramdisk, which carries z14's init). Its line 8 and 27-35: a subshell opens `/dev/watchdog`, pets it every 2 s, and after
  **90 beats** writes `walleye-banner: self-reboot now` to kmsg and runs `sync; reboot -f`, sysrq b behind it.
  `grep -ci hold init` = **0**: this init knows no hold file and no hold word.
- `/tmp/hold` is honoured only by init-late (decision 2026-11-19-walleye-late-probe-init-hold: z14's petter loop with
  `|| [ -e /tmp/hold ]`) and by build/initramfs-root-debug/init (HOLD_SECONDS, walleye.hold). The UFS procedure's
  `hold_command` touches /tmp/hold and checks the file exists (`hold-is-42`): it proves a file, not a hold.
- kernel: qcom_wdt built in (strings `qcom,kpss-wdt`); dtb has `watchdog@17817000` compatible qcom,kpss-wdt, no timeout-sec.
  dmesg of every run has no `NO /dev/watchdog` line, so the petter opened the SoC watchdog. The 180 s is the init's own
  `reboot -f`, not a bite (a bite would need the petter to stop before its 90 beats).

## (2) Each run against the 180 s

The petter starts at 1.63-1.73 s uptime (dmesg `walleye-banner: step 2: about to start the watchdog petter`, runs 1, 2, 3, 8).
Bus appear/gone times: research walleye-ufs-try-leaves-the-bus-at-180s-every-run (journalctl -k of the host). Load start =
the mtime of the last file saved before the load (gear-before.txt; run 8 gear-pinned.txt), `ls --full-time`.

| run | folder | gadget on bus | link.txt saved | load starts | bus gone | load start after bus-on | load ran before the bus left |
|---|---|---|---|---|---|---|---|
| 1 | ufs-20261123-080827 | 08:08:59 | 08:11:54.5 | 08:11:54.8 | 08:11:59 | 175.8 s | about 4.2 s |
| 2 | ufs-20261123-085319 | 08:53:50 | 08:56:45.1 | 08:56:45.4 | 08:56:50 | 175.4 s | about 4.6 s |
| 3 | ufs-20261123-105319 | 10:53:49 | 10:56:44.6 | 10:56:45.0 | 10:56:49 | 176.0 s | about 4.0 s |
| 8 | ufs-20261124-010239 | 01:03:10 | 01:06:05.1 | 01:06:06.0 | 01:06:10 | 176.0 s | about 4.0 s |

The "silent about 4.5 s into the sustained read" of runs 1-3 (research walleye-ufs-silent-under-load-causes-ranked) is this
same 4.0-4.6 s: in every run the load began 4-5 s before the init's own reboot. Bus times are whole seconds; the shift between
bus-on and uptime 0 (about 3 s, the gadget bind) is NOT KNOWN to the tenth.

## (3) Why the link read takes 171 s

ufs-try2-run3.log: `begin the link is read` 1795427633, `ok` 1795427804 = **171 s**. `Ufs::the_link_is_read`
(tools/walleye-tries/src/ufs.rs) polls every 2 s until the link is HS-G3 on 2 lanes or `device_wait` ends; device.toml
`[boot] device_wait = 170`. At idle the link reads `gear=HS_GEAR1 link_state=HIBERN8 dev_pm=SLEEP` (run 8 link.txt; try 1 the
same; research walleye-ufs-why-gear1-ranked: clock scale-down at idle), so the wait never ends early: 170 s + the last read.
Everything else before the load (boot, shell, probe lines, LUNs, GPT, gear, pin) takes about 5 s.

## (4) The smallest change

The procedure, not the image: read the idle link once (up = works, slow; down = fail) instead of waiting for HS-G3 at idle,
so the load starts about 10 s after the shell answers, with about 165 s of the init's budget left; drop the hold claim (this
image has none) and read instead the reboot deadline from the image's own kmsg line (petter start uptime + 180 s); before each
ramp step read `/proc/uptime` and refuse a step whose limit would cross the deadline less a margin. The image and its sha256
stay as they are, so the author's go on it means what it meant. The next run must show first: uptime at the load start, and
the deadline, in its own files.
