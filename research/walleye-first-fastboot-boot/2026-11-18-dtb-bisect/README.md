Hashes and recipe of the d1..d6 images: pk4 (porthole's kernel + OUR dtb, the known-dead one) with one group of nodes removed from our dtb each. Images live on the lab machine in ~/Projects/walleye-build/images-dtb.

Built by `build/build-dtb-bisect.sh PK4_BOOT_IMG PK5_BOOT_IMG DTBO_IMG OUT_DIR` (90 checks, all ok; rebuild byte-identical). Everything but the dtb is pk4's: header, no-earlycon command line, ramdisk init-ml5a (reboot at once), porthole's Image.gz, AVB footer, 32 MiB. The stock dtbo overlay (entry 3) merges into every dtb.
- d1: no watchdog@17817000 (ours: compatible kpss-wdt only; porthole has the same address with "qcom,apss-wdt-msm8998", timeout-sec 30)
- d2: no reserved-memory ramoops@a1100000 (porthole's ramoops is at 0xb0000000)
- d3: no reserved-memory memory@94b00000 (our slpi carve-out 0x700000; porthole's slpi is 0x95800000, 0xf00000)
- d4: no RPM regulators (remoteproc glink-edge rpm-requests regulators-0 and regulators-1; porthole has pm8998-regulators and pmi8998-regulators instead)
- d5: no usb@a8f8800 (dwc3 glue) and no phy@c012000 (qusb2 phy)
- d6: d1 + d2 + d3 together
Not a group: spmi@800f000 (the dtbo overlay needs it: without it the overlay does not merge, so the image would not be the test).
Expected return (fastboot boot to adb): init ran = Android back at about 40-65 s; no init = about 75 s. The first image to come back early names its group; if d6 returns early and d1..d3 do not, two of the three together are needed; if none returns early, the cause is in what is left (cpus, clocks, pinctrl, thermal, iommus: the nodes porthole's file declares differently), and the next round bisects by adding our nodes to porthole's dtb (pk5 + group).
