# Walleye bring-up order: the plan, with its dependencies and what each part is proven by

Made 2026-11-18 by walleye-research from public sources, the lab's research rows (first mainline map, the
vendor hardware inventory, the mainline path) and the vendor kernel's own device tree (GPL). The phone was
never touched. What is a fact is said as one; what is an inference is marked "inference - check".

## 1. What is (the starting point)

Our own mainline kernel (7.3.0) and dtb boot to a shell on ttyGS0 (y2, lab PR 152). Present in its dmesg: USB
udc, TLMM pinctrl, SPMI controller 800f000, RPM regulators and clocks, ramoops, watchdog, qfprom, remoteproc
nodes. Absent: drm, power_supply. Faults: `pm8998@0`/`pmi8998@3` "does not have reg property" (-22);
arm-smmu 5040000 probe timeout (-110); c16f000.serial waits on a supplier from our overlay stubs
(`clock_gcc`); `i2c@c179000` phandle not found; psci PC mode -3 (a firmware message, not ours to fix).

## 2. The dependencies (what must work before what)

    gcc / rpmcc clocks (present) ----> everything below
    RPM regulators (present: L20, L26, S4 ...) --> UFS rails, panel rail L14/L22 are PM8998 *RPM* regs
    SPMI bus + pm8998/pmi8998/pm8005 children  --> PON (reboot reasons), RTC, GPIO banks (pm8998 gpio6/7 amp
                                                    reset, gpio5 OTG boost, pmi8998 gpio2/14), RRADC (ADC),
                                                    charger smb2, fuel gauge, LPG/LED, flash LED
    arm-smmu 5040000 (clean probe) ---------> display (DPU), GPU, venus, USB if behind smmu, ufs? (inference - check)
    qusb2phy + dwc3 (present) -------------> USB gadget: ACM now; NCM/ECM networking next
    UFS (ufshc + qmp-ufs phy, vcc/vccq/vccq2 from RPM regulators) --> the partitions: boot.img header, the
                                                    vendor firmware partitions (wifi, modem, adsp, slpi), rootfs
    firmware partitions read --------------> Wi-Fi/BT (wcn3990), GPU zap, remoteprocs, modem, audio DSP, sensors
    DSI PHY + DPU + MDSS (needs smmu) + panel driver (new) --> display
    blsp1_i2c5 + L6/L28 supplies ----------> touch (RMI4)

Correction to the item text of the brief: UFS does **not** wait for the SPMI PMIC. Its three rails (VCC L20,
VCCQ L26, VCCQ2 S4) are RPM-managed regulators, which the first dmesg shows present. The SPMI PMIC unlocks
the charger, PON, RTC, ADC and the GPIO banks - so the charger and UFS can be worked in parallel once the
PMIC is bound (inference from the vendor tree and mainline's msm8998 boards - check at the first UFS try).

## 3. The order, what each unlocks, what proves it

| rank | part (item) | needs | unlocks | proof (a test that counts) |
|---|---|---|---|---|
| 1 | PMIC on SPMI (`walleye-pmic-spmi`) | build, then phone | charger, PON reason, RTC, ADC, GPIO banks | dmesg has no "does not have reg property"; `/sys/bus/spmi/devices` lists the PMIC USIDs; `/sys/class/rtc/rtc0` and pon reboot reason readable |
| 2 | clean probes: smmu, serial supplier, i2c phandle (`walleye-dmesg-clean`) | build, then phone | display, GPU, all DMA devices | `devices_deferred` empty with debugfs mounted; no `-110` and no "phandle not found" in dmesg |
| 3 | charger + battery node (`walleye-charger-smb2`) | build, then phone | safe testing: the phone charges under mainline; battery-ok.sh without Android | `power_supply` charger class appears, `online` follows the cable, type reads USB/DCP |
| 4 | UFS read-only (`walleye-ufs-readonly`) | build, then phone | boot.img header of the phone, vendor firmware, a rootfs | `lsblk` lists the LUNs and `by-name` partitions, nothing written (test walleye-ufs-lsblk) |
| - | USB networking (`walleye-usb-net`) | build, then phone | ssh, fast transfers, no serial console | ssh to 172.16.43.1 answers (test walleye-gadget-up) |
| - | fuel gauge (`walleye-fuel-gauge`) | build, then phone | battery percent and the real voltage | `capacity` and `voltage_now` within 100 mV of fastboot's battery reading |
| - | touch RMI4 (`walleye-touch-rmi4`) | build, then phone (a person touches) | input | evtest shows events (test walleye-touch-event) |
| - | display S6E3FA5 (`walleye-display-s6e3fa5`) | build (new driver), then phone (a person looks) | the whole UI | DRM connector connected, a pattern seen (test walleye-display-pattern) |
| - | Wi-Fi + Bluetooth (`walleye-wifi-wcn3990`) | build, then phone | the network | wlan0 scans and associates; hci0 up |
| - | haptics, speaker amps, audio (`walleye-audio-haptics`) | research/build | audio, vibration | `aplay` tone heard (person), force feedback felt |
| - | modem, sensors (SLPI), camera, NFC, fingerprint, deep sleep, USB OTG | research first | calls, data, sensors ... | their own research rows decide the proof |

The first three items after the PMIC are 2 (clean probes), 3 (charger) and 4 (UFS): the first is the cheapest
and clears faults that block display and DMA; the second protects the battery, the lane's tightest constraint
(the keeper's rule stops tries below 3700 mV; a charging mainline kernel ends that); the third gives the
phone's own boot.img header and the firmware partitions every later part needs.

## 4. What mainline v7.3 has for the Pixel 2's chips (to the best of public sources, 2026-11-18)

| part | in mainline? | note |
|---|---|---|
| SoC clocks, TLMM, RPM regs, SPMI, rradc | yes | `gcc-msm8998`, `msm8998.dtsi`; present in our first dmesg |
| PMI8998 SMB2 charger | **yes**, `drivers/power/supply/qcom_smbx.c` (`qcom,pmi8998-charger`, Casey Connolly) | needs the dtsi node enabled and a `monitored-battery` |
| PMI8998 fuel gauge (FG gen3) | **not merged**: RFC v1 of 8 patches, 2025-11-24, David Heidelberg (driver by Joel Selvaraj, `pmi8998_fg.c`) | carried by sdm845-next; an upstream candidate (test and review it for walleye) |
| UFS host + QMP phy | yes | `qcom,msm8998-ufshc` |
| USB gadget | yes (already works: y2) | dwc3 + qusb2phy |
| DPU/DSI | yes (msm8998-mdss, dpu, dsi-phy-10nm-8998) | needs smmu |
| panel S6E3FA5 | **no** in-tree driver (s6e3fa7/s6e3fc2x01/s6e3ha2/ha8 exist); an out-of-tree one by Jami Kettunen exists | a new driver: an upstream candidate; init sequence from the vendor GPL dtsi, ported in our own words |
| touch Synaptics RMI4 i2c | yes (`rmi_i2c`, f01/f11/f12/f1a) | dt node only |
| haptics TI DRV2624 | no: mainline has drv260x/drv2665/drv2667 | porthole patch 0018 (GPL) is a starting point; check whether drv260x covers it |
| speaker amps TAS2557 | no (tas2552/2562/2764/2770/2780 exist) | porthole carries one; an upstream candidate; audio also needs ADSP/APR |
| codec WCD934x | yes (slimbus + codec) | machine driver and DSP firmware are the work |
| Wi-Fi WCN3990 | yes (`ath10k` snoc) | firmware from the vendor partition |
| modem, SLPI, camera | partly: remoteproc `mss-pil`, `adsp-pas`; camss for 8998 is out of tree (check) | later research rows |

## 5. Ideal of the project: as good as Android, by the 21 components of docs

Phase 1 "reachable" (kernel, usb-otg groundwork, fastboot): items above rank 1-4 and usb-net.
Phase 2 "usable": display-panel, touchscreen, wifi, bluetooth, haptics, sensors (basic), usb-otg, fingerprint (later).
Phase 3 "a phone": audio-speaker, audio-mic, audio-earpiece, modem-data/-calls/-sms, camera-rear/-front, nfc,
deep-sleep. Walleye has no headphone jack (`absent = ["jack"]`: audio-headphone is not applicable by hardware).
Every component either has an item above or an item below with `needs research` (modem, sensors, camera,
nfc, fingerprint, deep-sleep). The item that proves each is its row in docs' `tests`.

## 6. Not known yet

Whether UFS needs the smmu probe to finish; whether the MP1 board revision uses the xd/xe touch wiring;
the real battery cell data under mainline (2700 mAh and 4.4 V are the vendor tree's published spec).

## 7. Sources

LWN article 1048226 (the fuel gauge RFC); torvalds/linux `qcom_smbx.c`; the lab's own rows
2026-11-18-walleye-first-mainline-shell-map, google-walleye-vendor-kernel-hardware, google-walleye-mainline-path;
kcxt/linux-wahoo, porthole-dev/pmaports taimen-bringup (read earlier, see the mainline path material).
