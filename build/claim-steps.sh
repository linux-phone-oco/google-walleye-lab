#!/bin/sh
# Claim walleye's steps on docs' bring-up chain (item walleye-steps-claimed): one `ikb step` per step, each checked
# against this base's own records (research rows, backlog, decisions) on 2026-11-18. A done claim names its proof (a
# test in tests/host, a build check or docs' proof id) and its run (a research row of this base) and the image it ran
# on; what the records do not prove is partial or open, and a step nobody has tried is open - never failing. Run in a
# branch of this base; every claim stops the script when it fails. Re-running replaces the claims (a step of that id is
# replaced), so a later try changes one line here and the claim with it.
set -eu
CLAIMED=2026-11-18  # the day these claims were checked; change it with the claims, so a re-run stays identical
c() { ikb step --updated "$CLAIMED" "$@" >/dev/null; }

# prepare: what the stock phone is
c --step bootloader-unlocked --status done --proof fastboot-unlocked --run google-walleye-bootloader-first-look \
  --proven-on stock-bootloader
c --step stock-measured --status partial --proof android-reference --run android-reference-walleye-measured \
  --notes "read only on LineageOS 15 over the stock vendor; the stock Android of the Pixel 2 itself is not measured"
c --step stock-facts-read --status done --proof stock-facts-recorded --run walleye-stock-ids-recorded \
  --proven-on stock-boot.img
c --step slot-state-known --status done --proof slot-report --run walleye-first-state-control-accepted \
  --notes "slot flags read and the first state restored; walleye-slot-flip-why says how a failed boot flips them"

# boot
c --step control-image-boots --status done --proof fastboot-boot-control --run walleye-try-s0-control-healthy-state \
  --proven-on control
c --step boot-image-packs --status done --proof build/check-bootimg.sh --run walleye-bootimage --proven-on pk3/y2
c --step dtbo-accepted --status done --proof tests/host/dtbo-overlay-merges.sh --run walleye-s4-overlay-symbols-try \
  --proven-on s4 --notes "with the stock dtbo (entry 3); our own minimal dtbo is not tried (decision 2026-11-19-walleye-minimal-dtbo-try)"
c --step initramfs-gadget-emulated --status done --proof build/build-initramfs.sh --run walleye-initramfs-usb-gadget
c --step stock-kernel-runs-our-init --status done --proof tests/host/massstorage-read.sh \
  --run 2026-11-17-walleye-m1-window-read --proven-on m1
c --step crash-window --status partial --proof tests/host/pstore-t1-read.sh --run walleye-pstore-warm-reset-how-it-stands \
  --notes "mass_storage window read on the vendor kernel (m1); pstore empty after every mainline try, so a mainline crash is not yet seen"

# kernel
c --step kernel-builds --status done --proof dtb-builds-with-ids --run walleye-kernel-dts
c --step reference-kernel-boots --status done --proof reference-alive --run walleye-pk-bisect-returns --proven-on pk3/pk5
c --step kernel-reaches-init --status done --proof kernel-alive --run walleye-y-tried --proven-on y2
c --step first-dmesg-mapped --status done --proof dmesg-map --run walleye-z6-dmesg-mapped --proven-on z6
c --step dmesg-clean --status done --proof tests/host/dmesg-clean-check.sh --run walleye-z7-tried --proven-on z7

# console and gadget
c --step uart-console --status open --notes "waits the cable: walleye-get-uart-setup"
c --step gadget-enumerates --status done --proof tests/host/mainline-light-up.sh --run walleye-y-tried --proven-on y2
c --step banner-on-console --status done --proof tests/host/banner-check.sh --run walleye-banner2-tried --proven-on banner2
c --step shell-on-console --status done --proof tests/host/mainline-shell-read.sh --run walleye-z6-tried --proven-on z6

# power
c --step pmic-bound --status done --proof tests/host/pmic-dmesg-check.sh --run walleye-z7-tried --proven-on z7
c --step charger-online --status partial --proof tests/host/charger-check.sh --run walleye-c3-tried --proven-on c3 \
  --notes "c2 deferred it (tps61099 has no driver); c3 with fw_devlink=permissive reads type USB, online 1, only one cable state seen and voltage_now read 0, and the phone then went silent: no restart gate yet (decision 2026-11-18-walleye-restart-gate-before-charger-try)"
c --step watchdog-way-back --status failing --proof tests/host/pmic-wdt-bites.sh --run walleye-z9-tried-pmic-wdt-bites \
  --proven-on z9 --notes "the PMIC watchdog bit and the phone stayed off: no way back yet; no more bites until the reset type is a restart (decision 2026-11-18-walleye-no-pmic-bite-until-reset-type)"

# every other step of the chain: nothing tried yet on this phone, so open (not failing)
for s in \
  firmware-reachable pd-mapper-before-remoteproc panel-lit camera-bus-bound usb-network \
  remoteprocs-running touch-input gpu-renders bt-controller-up image-sensor-identified \
  lab-link-held-awake audio-codec-attached wifi-joins raw-frame-captured shell-autosleep-tamed \
  sound-card-registered camera-stream-viewable speaker-amps-fed mic-captures focus-and-flash \
  battery-gauge-reads pon-reason-read storage-readonly reset-types-right reset-state-survives \
  key-reset-recovers rootfs-boots suspend-way-back power-off-stays-off thermal-zones-sane \
  health-suite-runs fastrpc-helpers-stable buttons-report haptics-buzz ucm-hifi-profile \
  modem-file-services nfc-reads-tag usb-host-works s2idle-cycle-returns charger-mode-entered \
  rtc-alarm-powers-on speaker-heard earpiece-plays headset-jack modem-seen bt-pairs-carries-audio \
  sensor-hub-stable suspend-wakes-quiet radios-survive-suspend shell-runs-with-hardware \
  update-applies kernel-upgrade-keeps-steps linux-runs off-mode-charging mic-hears-speaker sim-ready \
  emergency-numbers-known sensors-exposed gnss-fix fingerprint-unlocks photo-taken soc-floor-reached \
  on-screen-keyboard lock-and-unlock update-rolls-back modem-registered proximity-reads \
  auto-rotate-follows brightness-follows-light video-recorded sensors-across-suspend sms-both-ways \
  mobile-data call-audio-path resume-without-stall coprocessor-restart-recovers mms-both-ways \
  voice-call-both-ways hotspot-shares-data real-suspend-under-shell notifications-arrive \
  cycles-repeat-clean proximity-blanks-in-call volte-call call-wakes-from-sleep standby-current \
  alarm-wakes-from-sleep all-day-battery days-without-reboot daily-usable-phone complete-phone
do
  c --step "$s" --status open
done
