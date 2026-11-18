# y2 with the old reader (lab commit 733052d)

Run 2026-11-18 ~09:41 by walleye-keeper. Battery 4376 mV. Reader = `git show 733052d:tests/host/mainline-shell-read.sh`, started before the jump, WAIT=150. Image `~/Projects/walleye-build/images-y/boot-y2.img` (sha256 0579fcf4...).

- Phone left fastboot after 15 s, USB 1d6b:0104 "Pixel 2 walleye (mainline)" after 18 s, cdc_acm ttyACM0.
- Reader: all answers empty; only devices-deferred.txt had 171 bytes = the echo of the command line sent (so the host receives what the phone's tty echoes).
- Host: stty raw -echo 115200; fuser: only the reader; ModemManager inactive; ACL user:mll:rw; kernel log has no errors. Reader's last write: Permission denied (node re-created after the phone's reboot).
- Diff of readers (733052d..HEAD): one added `ask spmi-devices` line and a comment. Not the cause.
- Android back on slot b, adb up, nothing flashed. Raw output in ~/lab-material/walleye/y2-oldreader-094158 (not in git).

Next: the banner-init item (walleye-build-banner-init) and a raw cat of the tty before any command.
