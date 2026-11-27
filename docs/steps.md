<!-- Generated from data/ by tad. Do not edit by hand. -->

# Steps

### Row 1

Step
: bootloader-unlocked

Status
: done

Proof
: fastboot-unlocked

Run
: google-walleye-bootloader-first-look

Proven on
: stock-bootloader

Updated
: 2026-11-18


### Row 2

Step
: stock-measured

Status
: partial

Proof
: android-reference

Run
: android-reference-walleye-measured

Notes
: read only on LineageOS 15 over the stock vendor; the stock Android of the Pixel 2 itself is not measured

Updated
: 2026-11-18


### Row 3

Step
: stock-facts-read

Status
: done

Proof
: stock-facts-recorded

Run
: walleye-stock-ids-recorded

Proven on
: stock-boot.img

Updated
: 2026-11-18


### Row 4

Step
: slot-state-known

Status
: done

Proof
: slot-report

Run
: walleye-first-state-control-accepted

Notes
: slot flags read and the first state restored; walleye-slot-flip-why says how a failed boot flips them

Updated
: 2026-11-18


### Row 5

Step
: control-image-boots

Status
: done

Proof
: fastboot-boot-control

Run
: walleye-try-s0-control-healthy-state

Proven on
: control

Updated
: 2026-11-18


### Row 6

Step
: boot-image-packs

Status
: done

Proof
: build/check-bootimg.sh

Run
: walleye-bootimage

Proven on
: pk3/y2

Updated
: 2026-11-18


### Row 7

Step
: dtbo-accepted

Status
: done

Proof
: tests/host/dtbo-overlay-merges.sh

Run
: walleye-s4-overlay-symbols-try

Proven on
: s4

Notes
: with the stock dtbo (entry 3); our own minimal dtbo is not tried (decision 2026-11-19-walleye-minimal-dtbo-try)

Updated
: 2026-11-18


### Row 8

Step
: initramfs-gadget-emulated

Status
: done

Proof
: build/build-initramfs.sh

Run
: walleye-initramfs-usb-gadget

Updated
: 2026-11-18


### Row 9

Step
: stock-kernel-runs-our-init

Status
: done

Proof
: tests/host/massstorage-read.sh

Run
: 2026-11-17-walleye-m1-window-read

Proven on
: m1

Updated
: 2026-11-18


### Row 10

Step
: crash-window

Status
: partial

Proof
: tests/host/pstore-t1-read.sh

Run
: walleye-pstore-warm-reset-how-it-stands

Notes
: mass_storage window read on the vendor kernel (m1); pstore empty after every mainline try, so a mainline crash is not yet seen

Updated
: 2026-11-18


### Row 11

Step
: kernel-builds

Status
: done

Proof
: dtb-builds-with-ids

Run
: walleye-kernel-dts

Updated
: 2026-11-18


### Row 12

Step
: reference-kernel-boots

Status
: done

Proof
: reference-alive

Run
: walleye-pk-bisect-returns

Proven on
: pk3/pk5

Updated
: 2026-11-18


### Row 13

Step
: kernel-reaches-init

Status
: done

Proof
: kernel-alive

Run
: walleye-y-tried

Proven on
: y2

Updated
: 2026-11-18


### Row 14

Step
: first-dmesg-mapped

Status
: done

Proof
: dmesg-map

Run
: walleye-z6-dmesg-mapped

Proven on
: z6

Updated
: 2026-11-18


### Row 15

Step
: dmesg-clean

Status
: done

Proof
: tests/host/dmesg-clean-check.sh

Run
: walleye-z7-tried

Proven on
: z7

Updated
: 2026-11-18


### Row 16

Step
: uart-console

Status
: open

Notes
: waits the cable: walleye-get-uart-setup

Updated
: 2026-11-18


### Row 17

Step
: gadget-enumerates

Status
: done

Proof
: tests/host/mainline-light-up.sh

Run
: walleye-y-tried

Proven on
: y2

Updated
: 2026-11-18


### Row 18

Step
: banner-on-console

Status
: done

Proof
: tests/host/banner-check.sh

Run
: walleye-banner2-tried

Proven on
: banner2

Updated
: 2026-11-18


### Row 19

Step
: shell-on-console

Status
: done

Proof
: tests/host/mainline-shell-read.sh

Run
: walleye-z6-tried

Proven on
: z6

Updated
: 2026-11-18


### Row 20

Step
: charger-online

Status
: partial

Proof
: tests/host/charger-check.sh

Run
: walleye-c3-tried

Proven on
: c3

Notes
: c2 deferred it (tps61099 has no driver); c3 with fw_devlink=permissive reads type USB, online 1, only one cable state seen and voltage_now read 0, and the phone then went silent: no restart gate yet (decision 2026-11-18-walleye-restart-gate-before-charger-try)

Updated
: 2026-11-18


### Row 21

Step
: firmware-reachable

Status
: open

Updated
: 2026-11-18


### Row 22

Step
: pd-mapper-before-remoteproc

Status
: open

Updated
: 2026-11-18


### Row 23

Step
: camera-bus-bound

Status
: open

Updated
: 2026-11-18


### Row 24

Step
: gpu-renders

Status
: open

Updated
: 2026-11-18


### Row 25

Step
: bt-controller-up

Status
: open

Updated
: 2026-11-18


### Row 26

Step
: image-sensor-identified

Status
: open

Updated
: 2026-11-18


### Row 27

Step
: lab-link-held-awake

Status
: open

Updated
: 2026-11-18


### Row 28

Step
: audio-codec-attached

Status
: open

Updated
: 2026-11-18


### Row 29

Step
: wifi-joins

Status
: open

Updated
: 2026-11-18


### Row 30

Step
: raw-frame-captured

Status
: open

Updated
: 2026-11-18


### Row 31

Step
: shell-autosleep-tamed

Status
: open

Updated
: 2026-11-18


### Row 32

Step
: sound-card-registered

Status
: open

Updated
: 2026-11-18


### Row 33

Step
: camera-stream-viewable

Status
: open

Updated
: 2026-11-18


### Row 34

Step
: speaker-amps-fed

Status
: open

Updated
: 2026-11-18


### Row 35

Step
: mic-captures

Status
: open

Updated
: 2026-11-18


### Row 36

Step
: focus-and-flash

Status
: open

Updated
: 2026-11-18


### Row 37

Step
: pon-reason-read

Status
: open

Updated
: 2026-11-18


### Row 38

Step
: reset-types-right

Status
: open

Updated
: 2026-11-18


### Row 39

Step
: reset-state-survives

Status
: open

Updated
: 2026-11-18


### Row 40

Step
: key-reset-recovers

Status
: open

Updated
: 2026-11-18


### Row 41

Step
: suspend-way-back

Status
: open

Updated
: 2026-11-18


### Row 42

Step
: power-off-stays-off

Status
: open

Updated
: 2026-11-18


### Row 43

Step
: thermal-zones-sane

Status
: open

Updated
: 2026-11-18


### Row 44

Step
: health-suite-runs

Status
: open

Updated
: 2026-11-18


### Row 45

Step
: fastrpc-helpers-stable

Status
: open

Updated
: 2026-11-18


### Row 46

Step
: buttons-report

Status
: open

Updated
: 2026-11-18


### Row 47

Step
: ucm-hifi-profile

Status
: open

Updated
: 2026-11-18


### Row 48

Step
: modem-file-services

Status
: open

Updated
: 2026-11-18


### Row 49

Step
: nfc-reads-tag

Status
: open

Updated
: 2026-11-18


### Row 50

Step
: usb-host-works

Status
: open

Updated
: 2026-11-18


### Row 51

Step
: s2idle-cycle-returns

Status
: open

Updated
: 2026-11-18


### Row 52

Step
: charger-mode-entered

Status
: open

Updated
: 2026-11-18


### Row 53

Step
: rtc-alarm-powers-on

Status
: open

Updated
: 2026-11-18


### Row 54

Step
: speaker-heard

Status
: open

Updated
: 2026-11-18


### Row 55

Step
: earpiece-plays

Status
: open

Updated
: 2026-11-18


### Row 56

Step
: headset-jack

Status
: open

Updated
: 2026-11-18


### Row 57

Step
: modem-seen

Status
: open

Updated
: 2026-11-18


### Row 58

Step
: bt-pairs-carries-audio

Status
: open

Updated
: 2026-11-18


### Row 59

Step
: sensor-hub-stable

Status
: open

Updated
: 2026-11-18


### Row 60

Step
: suspend-wakes-quiet

Status
: open

Updated
: 2026-11-18


### Row 61

Step
: radios-survive-suspend

Status
: open

Updated
: 2026-11-18


### Row 62

Step
: shell-runs-with-hardware

Status
: open

Updated
: 2026-11-18


### Row 63

Step
: update-applies

Status
: open

Updated
: 2026-11-18


### Row 64

Step
: kernel-upgrade-keeps-steps

Status
: open

Updated
: 2026-11-18


### Row 65

Step
: linux-runs

Status
: open

Updated
: 2026-11-18


### Row 66

Step
: off-mode-charging

Status
: open

Updated
: 2026-11-18


### Row 67

Step
: mic-hears-speaker

Status
: open

Updated
: 2026-11-18


### Row 68

Step
: sim-ready

Status
: open

Updated
: 2026-11-18


### Row 69

Step
: emergency-numbers-known

Status
: open

Updated
: 2026-11-18


### Row 70

Step
: sensors-exposed

Status
: open

Updated
: 2026-11-18


### Row 71

Step
: gnss-fix

Status
: open

Updated
: 2026-11-18


### Row 72

Step
: fingerprint-unlocks

Status
: open

Updated
: 2026-11-18


### Row 73

Step
: photo-taken

Status
: open

Updated
: 2026-11-18


### Row 74

Step
: soc-floor-reached

Status
: open

Updated
: 2026-11-18


### Row 75

Step
: on-screen-keyboard

Status
: open

Updated
: 2026-11-18


### Row 76

Step
: lock-and-unlock

Status
: open

Updated
: 2026-11-18


### Row 77

Step
: update-rolls-back

Status
: open

Updated
: 2026-11-18


### Row 78

Step
: modem-registered

Status
: open

Updated
: 2026-11-18


### Row 79

Step
: proximity-reads

Status
: open

Updated
: 2026-11-18


### Row 80

Step
: auto-rotate-follows

Status
: open

Updated
: 2026-11-18


### Row 81

Step
: brightness-follows-light

Status
: open

Updated
: 2026-11-18


### Row 82

Step
: video-recorded

Status
: open

Updated
: 2026-11-18


### Row 83

Step
: sensors-across-suspend

Status
: open

Updated
: 2026-11-18


### Row 84

Step
: sms-both-ways

Status
: open

Updated
: 2026-11-18


### Row 85

Step
: mobile-data

Status
: open

Updated
: 2026-11-18


### Row 86

Step
: call-audio-path

Status
: open

Updated
: 2026-11-18


### Row 87

Step
: resume-without-stall

Status
: open

Updated
: 2026-11-18


### Row 88

Step
: coprocessor-restart-recovers

Status
: open

Updated
: 2026-11-18


### Row 89

Step
: mms-both-ways

Status
: open

Updated
: 2026-11-18


### Row 90

Step
: voice-call-both-ways

Status
: open

Updated
: 2026-11-18


### Row 91

Step
: hotspot-shares-data

Status
: open

Updated
: 2026-11-18


### Row 92

Step
: real-suspend-under-shell

Status
: open

Updated
: 2026-11-18


### Row 93

Step
: notifications-arrive

Status
: open

Updated
: 2026-11-18


### Row 94

Step
: cycles-repeat-clean

Status
: open

Updated
: 2026-11-18


### Row 95

Step
: proximity-blanks-in-call

Status
: open

Updated
: 2026-11-18


### Row 96

Step
: volte-call

Status
: open

Updated
: 2026-11-18


### Row 97

Step
: call-wakes-from-sleep

Status
: open

Updated
: 2026-11-18


### Row 98

Step
: standby-current

Status
: open

Updated
: 2026-11-18


### Row 99

Step
: alarm-wakes-from-sleep

Status
: open

Updated
: 2026-11-18


### Row 100

Step
: all-day-battery

Status
: open

Updated
: 2026-11-18


### Row 101

Step
: days-without-reboot

Status
: open

Updated
: 2026-11-18


### Row 102

Step
: daily-usable-phone

Status
: open

Updated
: 2026-11-18


### Row 103

Step
: complete-phone

Status
: open

Updated
: 2026-11-18


### Row 104

Step
: panel-lit

Status
: done

Proof
: procedures/display-stripes-on-the-pixel-2.rs

Run
: research walleye-display-l7-looks-seen (lab PR 592, 594, 595)

Proven on
: boot-l7.img sha256 643d1f83c6aea4ecb2d08e95aebc7fb2694d86c5c021e8463dc902eff5b9e711

Notes
: The author saw the kernel log and four held vertical stripes on the glass, 2026-11-19, 'they look perfect'. The modules are loaded by the procedure in a hand-picked order; the display at boot in any order is not done (B2: items walleye-display-b2-dsi-clock-warning, walleye-display-dts-any-order).

Updated
: 2026-11-19


### Row 105

Step
: touch-input

Status
: done

Proof
: tests/attended/touch-try.rs

Run
: research walleye-z21-touch-events-pass (lab PR 485)

Proven on
: boot-z21.img sha256 bdc0a89179d2d7fdb75e729515fcaadb9a501d4c6fdb835dc1fc3cff4639c7ff

Notes
: Synaptics S3708AR probed; 3593 EV_ABS events read while the author swiped. No UI uses it yet.

Updated
: 2026-11-19


### Row 106

Step
: usb-network

Status
: done

Proof
: tests/host/gadget-up.sh

Run
: research walleye-z14-tried

Proven on
: boot-z14.img sha256 c2e616f872328b1a28df9ff37903796579e078a7a7ea2f66195a928beee64238

Notes
: ECM usb0 172.16.43.1 pings and telnet answers, 18 s after fastboot; every try since (z21, upstream-dts, L-images, the UFS try) used it.

Updated
: 2026-11-19


### Row 107

Step
: battery-gauge-reads

Status
: done

Proof
: tests/host/fuel-gauge-check.sh

Run
: research walleye-fg1-tried-capacity-matches

Proven on
: boot-fg1.img sha256 9f6a7033ae13e532b91874bb90e8a08803172ccfa552bbe6eb53d7f07086b99b

Notes
: pmi8998-fg probes; capacity 99 percent and 4374 mV equal fastboot's 4374 mV.

Updated
: 2026-11-19


### Row 108

Step
: haptics-buzz

Status
: partial

Proof
: tests/host/drv2624-builds.sh

Run
: research walleye-drv1-tried-drv2624-probes (lab PR 518)

Proven on
: boot-drv1.img sha256 96e85804db5478aae4525266997dfcf65f6990493cbb00ac3398114cc3501e8b

Notes
: DRV2624 answers at 0-005a, id 0x03, input drv2624:haptics registered. No effect has been played yet.

Updated
: 2026-11-19


### Row 109

Step
: watchdog-way-back

Status
: partial

Proof
: way-back

Run
: research walleye-pmic-wdt-v1-try-result

Proven on
: boot-wdt1-a.img (images-wdt1-sysfsless) sha256 9dfea09637a813fa677449780ac475a3b859e81135a056fadd8811dadc2f40b4

Notes
: A hung kernel is RESET by itself 20 s after arming, unpetted (PMIC watchdog v1, S2 type 7). It does not yet COME BACK to Android by itself with USB in: the phone boots off-mode charging and Android starts after a short power press (the author's words in the research row). Item walleye-wdt-reset-boots-charger-mode.

Updated
: 2026-11-19


### Row 110

Step
: pmic-bound

Status
: done

Proof
: pmic-bound

Run
: research walleye-pmic-wdt-v1-try-result

Proven on
: boot-wdt1-a.img (images-wdt1-sysfsless) sha256 9dfea09637a813fa677449780ac475a3b859e81135a056fadd8811dadc2f40b4

Notes
: The PMIC and its watchdog bind on mainline (pm8916-wdt gen2: pmic=watchdog1 soc=watchdog0), S2 type 0x7 read back, the stop on reboot leaves S2 off.

Updated
: 2026-11-19


### Row 111

Step
: display-any-order

Status
: partial

Proof
: display-boots-any-order

Run
: walleye-display-any-order-b-r2-b-rp2-clean

Proven on
: B-r2 c05ae4ec, B-rp2 29fce8b7

Notes
: two orders at 2 s clean (0/0/0, 13 of 13); autoload by modalias and three cold boots not yet

Updated
: 2026-11-23


### Row 112

Step
: rootfs-boots

Status
: done

Proof
: procedures/rootfs-boots-on-the-pixel-2.rs

Run
: walleye-rootfs-boots-run-6-passed-bootloader-not-reached

Proven on
: c3373d7199d26a403a7ac9b10c8c95f4dd17f41d358eb59de095efd6bb6c7ba9

Notes
: run 6: 26 of 26; bootloader way back not reached (forced reboot ran)

Updated
: 2026-11-23


### Row 113

Step
: storage-readonly

Status
: done

Proof
: ufs-matched-image-on-the-pixel-2

Run
: walleye-ufs-try-2-run-10-unpinned-ramp-passes-at-hs-gear3

Proven on
: ufs-match image sha256 461077869fb2

Notes
: Card read: UFS 3.1.0, 6 disks, GPT hashed equal before and after; unpinned ramp 64 to 2048 MiB read rc=0 at HS_GEAR3, shell answered every poll (run 10)

Updated
: 2026-11-24


### Row 114

Step
: storage-full-speed

Status
: partial

Proof
: ufs-matched-image-on-the-pixel-2

Run
: walleye-ufs-try-2-run-10-unpinned-ramp-passes-at-hs-gear3

Proven on
: ufs-match image sha256 461077869fb2

Notes
: Run 10: under an unpinned read the card chose HS_GEAR3, 2 lanes, HS_RATE_B, 200 MHz, reads to 2048 MiB fine. Idle stays HS_GEAR1 at 50 MHz. Reads only; writes not tried (stage 3).

Updated
: 2026-11-24


### Row 115

Step
: test-layer-boots

Status
: open

Notes
: not measured yet: lpoco-health test-layer-quiet has not run on a walleye boot; the rootfs-boots step is done (stage 1 Alpine, run 6) but its image is not shown to be the test layer

Updated
: 2026-11-26


### Row 116

Step
: display-boots-any-order

Status
: partial

Proof
: tests/host/dtb-display-reset.sh

Run
: research walleye-display-at-boot-step-groups-run-passed

Notes
: b2 at boot (2.5 s): 13 of 13 modules ok, connector connected on the step-group run; still not looked at on the glass, and three cold boots not yet

Updated
: 2026-11-26


### Row 117

Step
: remoteprocs-running

Status
: partial

Proof
: walleye-modem-up

Run
: walleye-modem-mss-second-boot

Proven on
: modem image ae22f965 (kernel int/walleye/ufs-off-modem/v7.3/1)

Notes
: mss runs (remoteproc 4080000 'now up', modem-up.sh passed live in run 2, PR 1270; first in run 1, PR 1265); adsp and cdsp not measured yet.

Updated
: 2026-11-27
