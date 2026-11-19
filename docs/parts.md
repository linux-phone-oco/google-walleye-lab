<!-- Generated from data/ by tad. Do not edit by hand. -->

# Parts

### Row 1

Component
: audio-earpiece

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 2

Component
: audio-headphone

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 3

Component
: audio-mic

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 4

Component
: audio-speaker

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 5

Component
: bluetooth

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 6

Component
: camera-front

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 7

Component
: camera-rear

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 8

Component
: deep-sleep

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 9

Component
: fastboot

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 10

Component
: modem-calls

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 11

Component
: modem-data

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 12

Component
: modem-sms

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 13

Component
: nfc

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 14

Component
: sensors

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 15

Component
: usb-otg

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 16

Component
: wifi

Status
: unknown

Notes
: No source found reporting this on this phone (surveyed 2026-11-17: the Nura wiki, pmaports, the mainline tree, kcxt/linux-wahoo). Nobody here has measured it. Settles with: the phone's own test on a booted Linux kernel.

Updated
: 2026-11-17

Judged
: unknown


### Row 17

Component
: fingerprint

Status
: unknown

Evidence
: research/walleye-fingerprint/2026-11-19/README.md

Notes
: Identified from the vendor dtb 2026-11-19: FPC (compatible fpc,fpc1020), irq GPIO86, reset GPIO128, vdd_io PM8998 S4, SPI bus not named (TrustZone side). Exact chip and bus settle with an SPI HWID read; no driver on mainline or libfprint.

Updated
: 2026-11-19


### Row 18

Component
: haptics

Status
: partial

Evidence
: research walleye-drv1-tried-drv2624-probes

Notes
: DRV2624 bound at 0-005a with its input device (2026-11-19); no effect played yet; the rail that powers it is unread.

Updated
: 2026-11-19


### Row 19

Component
: kernel

Status
: partial

Evidence
: research walleye-upstream-dts-tried-boots

Notes
: Mainline v7.3 with our board dts boots RAM-booted from fastboot to a shell over USB (gadget 18 s); display, touch, fuel gauge, haptics and the UFS card read on it. Not installed: nothing is written to the phone (decision: no flash without a recorded decision, item walleye-install-stage-decision).

Updated
: 2026-11-19


### Row 20

Component
: storage

Status
: partial

Evidence
: research walleye-ufs-try1-card-readable

Notes
: UFS card read on mainline (read only, one try on the author's go): version 3.1.0, six LUNs, partitions listed, no errors; link at HS gear 1; nothing written; not used as a root filesystem.

Updated
: 2026-11-19


### Row 21

Component
: display-panel

Status
: partial

Evidence
: research walleye-display-l7-looks-seen

Notes
: Lit on mainline with L7 (the late-probe kernel, modules loaded by the procedure in a hand-picked order): the author saw the kernel log and four held stripes on the glass, 2026-11-19. Not yet from the board dts in any load order: B2 fails (pixel clock rcg warning, DSI -110, SMMU faults); research walleye-display-b2-dsi-clock-warning.

Updated
: 2026-11-19


### Row 22

Component
: touchscreen

Status
: works

Evidence
: research walleye-z21-touch-events-pass

Notes
: Works as an input device on z21 (RAM-booted): 3593 EV_ABS events from the Synaptics S3708AR while the author swiped, 2026-11-19. No UI uses it yet.

Updated
: 2026-11-19
