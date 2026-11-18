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
: pmic-bound

Status
: done

Proof
: tests/host/pmic-dmesg-check.sh

Run
: walleye-z7-tried

Proven on
: z7

Updated
: 2026-11-18


### Row 21

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


### Row 22

Step
: watchdog-way-back

Status
: failing

Proof
: tests/host/pmic-wdt-bites.sh

Run
: walleye-z9-tried-pmic-wdt-bites

Proven on
: z9

Notes
: the PMIC watchdog bit and the phone stayed off: no way back yet; no more bites until the reset type is a restart (decision 2026-11-18-walleye-no-pmic-bite-until-reset-type)

Updated
: 2026-11-18


### Row 23

Step
: firmware-reachable

Status
: open

Updated
: 2026-11-18


### Row 24

Step
: pd-mapper-before-remoteproc

Status
: open

Updated
: 2026-11-18


### Row 25

Step
: panel-lit

Status
: open

Updated
: 2026-11-18


### Row 26

Step
: camera-bus-bound

Status
: open

Updated
: 2026-11-18


### Row 27

Step
: usb-network

Status
: open

Updated
: 2026-11-18


### Row 28

Step
: remoteprocs-running

Status
: open

Updated
: 2026-11-18


### Row 29

Step
: touch-input

Status
: open

Updated
: 2026-11-18


### Row 30

Step
: gpu-renders

Status
: open

Updated
: 2026-11-18


### Row 31

Step
: bt-controller-up

Status
: open

Updated
: 2026-11-18


### Row 32

Step
: image-sensor-identified

Status
: open

Updated
: 2026-11-18


### Row 33

Step
: lab-link-held-awake

Status
: open

Updated
: 2026-11-18


### Row 34

Step
: audio-codec-attached

Status
: open

Updated
: 2026-11-18


### Row 35

Step
: wifi-joins

Status
: open

Updated
: 2026-11-18


### Row 36

Step
: raw-frame-captured

Status
: open

Updated
: 2026-11-18


### Row 37

Step
: shell-autosleep-tamed

Status
: open

Updated
: 2026-11-18


### Row 38

Step
: sound-card-registered

Status
: open

Updated
: 2026-11-18


### Row 39

Step
: camera-stream-viewable

Status
: open

Updated
: 2026-11-18


### Row 40

Step
: speaker-amps-fed

Status
: open

Updated
: 2026-11-18


### Row 41

Step
: mic-captures

Status
: open

Updated
: 2026-11-18


### Row 42

Step
: focus-and-flash

Status
: open

Updated
: 2026-11-18


### Row 43

Step
: battery-gauge-reads

Status
: open

Updated
: 2026-11-18


### Row 44

Step
: pon-reason-read

Status
: open

Updated
: 2026-11-18


### Row 45

Step
: storage-readonly

Status
: open

Updated
: 2026-11-18


### Row 46

Step
: reset-types-right

Status
: open

Updated
: 2026-11-18


### Row 47

Step
: reset-state-survives

Status
: open

Updated
: 2026-11-18


### Row 48

Step
: key-reset-recovers

Status
: open

Updated
: 2026-11-18


### Row 49

Step
: rootfs-boots

Status
: open

Updated
: 2026-11-18


### Row 50

Step
: suspend-way-back

Status
: open

Updated
: 2026-11-18


### Row 51

Step
: power-off-stays-off

Status
: open

Updated
: 2026-11-18


### Row 52

Step
: thermal-zones-sane

Status
: open

Updated
: 2026-11-18


### Row 53

Step
: health-suite-runs

Status
: open

Updated
: 2026-11-18


### Row 54

Step
: fastrpc-helpers-stable

Status
: open

Updated
: 2026-11-18


### Row 55

Step
: buttons-report

Status
: open

Updated
: 2026-11-18


### Row 56

Step
: haptics-buzz

Status
: open

Updated
: 2026-11-18


### Row 57

Step
: ucm-hifi-profile

Status
: open

Updated
: 2026-11-18


### Row 58

Step
: modem-file-services

Status
: open

Updated
: 2026-11-18


### Row 59

Step
: nfc-reads-tag

Status
: open

Updated
: 2026-11-18


### Row 60

Step
: usb-host-works

Status
: open

Updated
: 2026-11-18


### Row 61

Step
: s2idle-cycle-returns

Status
: open

Updated
: 2026-11-18


### Row 62

Step
: charger-mode-entered

Status
: open

Updated
: 2026-11-18


### Row 63

Step
: rtc-alarm-powers-on

Status
: open

Updated
: 2026-11-18


### Row 64

Step
: speaker-heard

Status
: open

Updated
: 2026-11-18


### Row 65

Step
: earpiece-plays

Status
: open

Updated
: 2026-11-18


### Row 66

Step
: headset-jack

Status
: open

Updated
: 2026-11-18


### Row 67

Step
: modem-seen

Status
: open

Updated
: 2026-11-18


### Row 68

Step
: bt-pairs-carries-audio

Status
: open

Updated
: 2026-11-18


### Row 69

Step
: sensor-hub-stable

Status
: open

Updated
: 2026-11-18


### Row 70

Step
: suspend-wakes-quiet

Status
: open

Updated
: 2026-11-18


### Row 71

Step
: radios-survive-suspend

Status
: open

Updated
: 2026-11-18


### Row 72

Step
: shell-runs-with-hardware

Status
: open

Updated
: 2026-11-18


### Row 73

Step
: update-applies

Status
: open

Updated
: 2026-11-18


### Row 74

Step
: kernel-upgrade-keeps-steps

Status
: open

Updated
: 2026-11-18


### Row 75

Step
: linux-runs

Status
: open

Updated
: 2026-11-18


### Row 76

Step
: off-mode-charging

Status
: open

Updated
: 2026-11-18


### Row 77

Step
: mic-hears-speaker

Status
: open

Updated
: 2026-11-18


### Row 78

Step
: sim-ready

Status
: open

Updated
: 2026-11-18


### Row 79

Step
: emergency-numbers-known

Status
: open

Updated
: 2026-11-18


### Row 80

Step
: sensors-exposed

Status
: open

Updated
: 2026-11-18


### Row 81

Step
: gnss-fix

Status
: open

Updated
: 2026-11-18


### Row 82

Step
: fingerprint-unlocks

Status
: open

Updated
: 2026-11-18


### Row 83

Step
: photo-taken

Status
: open

Updated
: 2026-11-18


### Row 84

Step
: soc-floor-reached

Status
: open

Updated
: 2026-11-18


### Row 85

Step
: on-screen-keyboard

Status
: open

Updated
: 2026-11-18


### Row 86

Step
: lock-and-unlock

Status
: open

Updated
: 2026-11-18


### Row 87

Step
: update-rolls-back

Status
: open

Updated
: 2026-11-18


### Row 88

Step
: modem-registered

Status
: open

Updated
: 2026-11-18


### Row 89

Step
: proximity-reads

Status
: open

Updated
: 2026-11-18


### Row 90

Step
: auto-rotate-follows

Status
: open

Updated
: 2026-11-18


### Row 91

Step
: brightness-follows-light

Status
: open

Updated
: 2026-11-18


### Row 92

Step
: video-recorded

Status
: open

Updated
: 2026-11-18


### Row 93

Step
: sensors-across-suspend

Status
: open

Updated
: 2026-11-18


### Row 94

Step
: sms-both-ways

Status
: open

Updated
: 2026-11-18


### Row 95

Step
: mobile-data

Status
: open

Updated
: 2026-11-18


### Row 96

Step
: call-audio-path

Status
: open

Updated
: 2026-11-18


### Row 97

Step
: resume-without-stall

Status
: open

Updated
: 2026-11-18


### Row 98

Step
: coprocessor-restart-recovers

Status
: open

Updated
: 2026-11-18


### Row 99

Step
: mms-both-ways

Status
: open

Updated
: 2026-11-18


### Row 100

Step
: voice-call-both-ways

Status
: open

Updated
: 2026-11-18


### Row 101

Step
: hotspot-shares-data

Status
: open

Updated
: 2026-11-18


### Row 102

Step
: real-suspend-under-shell

Status
: open

Updated
: 2026-11-18


### Row 103

Step
: notifications-arrive

Status
: open

Updated
: 2026-11-18


### Row 104

Step
: cycles-repeat-clean

Status
: open

Updated
: 2026-11-18


### Row 105

Step
: proximity-blanks-in-call

Status
: open

Updated
: 2026-11-18


### Row 106

Step
: volte-call

Status
: open

Updated
: 2026-11-18


### Row 107

Step
: call-wakes-from-sleep

Status
: open

Updated
: 2026-11-18


### Row 108

Step
: standby-current

Status
: open

Updated
: 2026-11-18


### Row 109

Step
: alarm-wakes-from-sleep

Status
: open

Updated
: 2026-11-18


### Row 110

Step
: all-day-battery

Status
: open

Updated
: 2026-11-18


### Row 111

Step
: days-without-reboot

Status
: open

Updated
: 2026-11-18


### Row 112

Step
: daily-usable-phone

Status
: open

Updated
: 2026-11-18


### Row 113

Step
: complete-phone

Status
: open

Updated
: 2026-11-18
