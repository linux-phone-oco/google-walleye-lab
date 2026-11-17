<!-- Generated from data/ by tad. Do not edit by hand. -->

# Backlog

<a id="android-reference-walleye"></a>
### Measure Android on the Pixel 2 before anything is flashed

Status
: in-progress

Note
: Measured read-only 2026-11-17 (research android-reference-walleye-measured): build, sensors, battery, boot time, audio config. Left: sleep current (needs a meter, radios stated), audio levels (tone source and microphone), boot time from power on

Opened
: 2026-11-17


<a id="walleye-dtbo-overlay-id"></a>
### Find out which dtbo overlay id walleye's bootloader needs before a mainline kernel boots

Status
: open

Note
: The taimen bootloader refuses a kernel unless dtbo holds an overlay with id 2704 (kcxt-dtbo-wahoo-mainline); walleye's id is not known. Read it without writing: from the stock dtbo (Android booted, or the factory image - decision needed, proprietary download). Research: google-walleye-how-others-booted.

Opened
: 2026-11-17


<a id="walleye-usb-uart-cable"></a>
### USB-C debug UART cable for walleye (no display): decide on 'fastboot oem uart enable' first

Status
: open

Note
: pmOS wiki lists the Android Debug Cable as tested with walleye. The oem command's effect (RAM flag or persistent) is not known, so it counts as a change to the phone: a decision first. Needs the author's hands for the cable. Research: google-walleye-how-others-booted.

Opened
: 2026-11-17


<a id="walleye-kernel-dts"></a>
### Build v7.3 with a minimal msm8998-google-walleye.dts

Status
: done

Note
: Built 2026-11-17: dt/msm8998-google-walleye.dts (lab PR 3) + v7.3 + build/walleye.config; counted: dtb builds, no dtc warning at W=1 from our file, msm-id 292/131073 (0x124/0x20001), 0 bootargs/stdout-path, Image.gz 15038692 bytes. Run: build/build-kernel.sh via oco-agent. Next: the keeper's one fastboot boot; UFS/display/touch stay off.

Opened
: 2026-11-17

Proof
: make qcom/msm8998-google-walleye.dtb builds with no dtc warning from our nodes, the dtb holds msm-id 0x124/0x20001 and no /chosen/bootargs, and Image.gz builds (test to write in lab-tools: walleye-dtb-check)


<a id="walleye-initramfs-usb-gadget"></a>
### Initramfs that brings up a USB gadget (ACM console + RNDIS/ECM) and telnet/ssh on 172.16.42.1

Status
: in-progress

Note
: Built (build/initramfs, busybox 1.36.1 static arm64, 1148576 bytes gz). Proven under x86_64 QEMU with dummy_hcd (build/selftest-initramfs.sh): gadget binds, lease 172.16.42.2, ping and telnet command answered on 172.16.42.1, ttyGS0 exists. NOT proven on arm64: qemu-system-aarch64 is not installed here, so the arm64 busybox and the real UDC first run on the phone try. No ssh yet (telnet only; dropbear is a later piece).

Opened
: 2026-11-17

Proof
: In QEMU aarch64 with dummy_hcd the initramfs creates the gadget and answers on its address (test: walleye-initramfs-gadget-qemu)


<a id="walleye-debug-without-display"></a>
### Reach a hung or crashed walleye without a display: warm reset, pstore, UART

Status
: open

Note
: ramoops at 0xa1810000/0x200000, qcom_wdt.arm_on_probe=1, watchdog.open_timeout=300, reboot=w (porthole); the Pixel debug cable is not in the lab.

Waits
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: after a deliberate panic the next boot lets the keeper read the panic log from /sys/fs/pstore (test: walleye-pstore-panic)


<a id="walleye-ufs-readonly"></a>
### UFS probes and the partitions are listed, read-only

Status
: open

Note
: Enable ufshc + ufsphy with the vcc/vccq rails from the wahoo dtsi; mount nothing, write nothing; this is also the way to read walleye's own boot.img header.

Waits
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: lsblk in the initramfs lists the UFS LUNs and by-name partitions without a write (test: walleye-ufs-lsblk)


<a id="walleye-display-s6e3fa5"></a>
### Display: Samsung S6E3FA5 command-mode panel through DPU/DSI

Status
: open

Note
: No mainline driver; start from the out-of-tree S6E3FA5 driver (OnePlus 5) and porthole's sw43402 cmd-mode work for the DPU side; init sequence from the GPL downstream dtsi; reset gpio94, TE gpio10 (walleye) - the panel and its rails are walleye's own.

Waits
: walleye-ufs-readonly (not required) / walleye-first-fastboot-boot; a person witnesses the first light

Opened
: 2026-11-17

Needs
: phone

Proof
: the DRM connector is connected and a test pattern is seen by a person (test: walleye-display-pattern)


<a id="walleye-touch-rmi4"></a>
### Touch: Synaptics RMI4 on blsp1_i2c5 (0x20, irq gpio125, reset gpio89)

Status
: open

Note
: Mainline rmi_i2c driver; supplies pm8998 l6 and l28; xd boards add a pmi8998 power gpio - which board revision the lab phone is, is not known.

Waits
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: evtest shows touch events from the RMI4 device when the panel is touched (test: walleye-touch-events)


<a id="walleye-bootimage"></a>
### Boot image: header v0, base 0, kernel 0x8000, ramdisk 0x1000000, tags 0x100, page 4096, dtb appended

Status
: in-progress

Note
: Four images, 17 checks each, 0 failed: a,b as before; c,d = a,b with os_version 8.1.0 / patch level 2020-01 (4 header bytes differ). sha256 in ~/Projects/walleye-build/images/SHA256SUMS. Header v0 fits walleye per public sources (walleye-bootimage-header-public); stock header still unread. Untried: the dtbo overlay id (walleye-dtbo-overlay-id) and an AVB footer.

Waits
: walleye's stock boot.img header (android-reference-walleye) to confirm the offsets

Opened
: 2026-11-17

Proof
: unpack_bootimg of the result shows the offsets and cmdline, the dtb is inside the kernel blob, size is under the phone's max-download-size (test: walleye-bootimg-unpack)


<a id="walleye-load-error-why"></a>
### What the Pixel 2's bootloader dislikes in our boot image (Load Error), found from public sources, then the image variants to try

Status
: done

Note
: Answered: research walleye-load-error-how-it-stands, decision 2026-11-17-walleye-load-error-control-first; steps walleye-loaderror-* (stock-ids, variants-build, control, variants-try, dtbo-stub).

Opened
: 2026-11-17

Needs
: research

Proof
: a recommended image recipe, each step cited, and the next 2-3 variants to build (research row), consistent with the tries recorded in walleye-first-fastboot-boot-tries*


<a id="walleye-loaderror-stock-ids"></a>
### Read walleye's stock boot header and dtb ids (qcom,board-id, pmic-id, msm-id) from a public LineageOS walleye build, host only

Status
: done

Note
: Recorded in research walleye-stock-ids-recorded; control image at ~/Projects/walleye-build/control/boot.img (sha256 48dbc55c...7468), dtbo.img beside it (fb09009b...b8f6).

Opened
: 2026-11-17

Needs
: build

Proof
: walleye-stock-ids-recorded: research row naming the header fields and the dtb's board-id/pmic-id/msm-id, each with the file hash


<a id="walleye-loaderror-variants-build"></a>
### Build variants e,f: blueline recipe (second 0, os word 0) and e plus the stock board-id/pmic-id in the dts; check-bootimg counts both

Status
: done

Note
: Built, lab PR 6 on main: ~/Projects/walleye-build/images/boot-e.img 112ab284...2daf and boot-f.img 7a42e5bd...461a (sha256 in images/SHA256SUMS; a-d unchanged). e = a with the command line cut to earlycon+console (second_offset 0 is a no-op: mkbootimg already writes second 0); f = e + qcom,board-id <8 0> (stock has none). os word 0.

Opened
: 2026-11-17

Needs
: build

Proof
: check-bootimg.sh passes for e and f and counts board-id/pmic-id in f


<a id="walleye-loaderror-control"></a>
### Control try: fastboot boot of a public walleye stock-format image (LineageOS recovery or TWRP) with devlab's before/after slot report

Status
: done

Note
: proven by research walleye-control-e-f-tries: control, e and f all Load Error, flags unchanged

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-fastboot-boot-control: devlab report shows the phone left fastboot (18d1:4ee0 gone for 90 s polls) or Load Error, recorded either way as a research row


<a id="walleye-loaderror-variants-try"></a>
### Try e then f (one at a time) with fastboot boot, after the control, with before/after slot report

Status
: done

Note
: proven by research walleye-control-e-f-tries: control, e and f all Load Error, flags unchanged

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-fastboot-boot-variants: each image's outcome recorded as research; success is a USB id other than 18d1:4ee0 appearing (our gadget) for e or f


<a id="walleye-loaderror-dtbo-stub"></a>
### dtbo_b stub with overlay id for walleye - only if the control boots and e,f still end in Load Error

Status
: open

Note
: A write to the phone: needs the author's word and the way back first (stock dtbo from the LineageOS zip read in walleye-loaderror-stock-ids, flashed back with fastboot flash dtbo_b). Overlay id for walleye is not known: read from the stock dtbo table, not guessed (kcxt's 2704 is for taimen).

Waits
: the author's word (a write to the phone)

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-dtbo-stub-boot: after the stub, fastboot boot of e or f leaves fastboot; stock dtbo flashed back and its hash equals the one from the zip


<a id="walleye-first-fastboot-boot"></a>
### First fastboot boot of the walleye kernel: the phone leaves fastboot and its gadget answers

Status
: in-progress

Note
: Six images and the official LineageOS control all refused with Load Error: the cause is the phone's state or what fastboot boot needs, not our image (walleye-control-e-f-tries). A dtbo/vbmeta write stays the author's.

Waits
: walleye-boot-refused-why (research), walleye-control-on-slot-a

Opened
: 2026-11-17

Needs
: phone

Proof
: lsusb on the host shows the gadget within 60 s of fastboot boot and ssh/telnet to 172.16.42.1 answers (test: walleye-gadget-up)


<a id="walleye-bootstrap-method"></a>
### The best-known way to bootstrap an Android phone to mainline almost from zero, compared with what the walleye lane did

Status
: done

Note
: Done 2026-11-17: research walleye-bootstrap-method-how-it-stands, decision 2026-11-17-walleye-vendor-kernel-first; next walleye-vendor-gadget-image, walleye-gadget-up-check, then walleye-vendor-gadget-try.

Opened
: 2026-11-17

Needs
: research

Proof
: a research row with sources, a comparison, a decision with what was rejected: landed


<a id="walleye-scripts-call-bringup"></a>
### walleye's build and check scripts become calls to bringup

Status
: open

Note
: After bringup-image-v0, -swap and -initramfs land: build-bootimg, build-bisect, check-bootimg, check-bisect, build-initramfs, build-all become one-line calls (or go) with the header facts in device.toml [boot]; build-kernel, build-busybox and selftest-initramfs stay as they are (make and qemu calls). Proof: the images a-i and s0-s3 rebuilt by bringup are byte for byte the ones in ~/Projects/walleye-build/images (SHA256SUMS), or differ only where the item says why.

Waits
: bringup-image-swap

Opened
: 2026-11-17


<a id="walleye-control-on-slot-a"></a>
### The control boot.img again with slot a active (the state of the very first tries)

Status
: done

Note
: proven by devlab's before/after slot report and fastboot text: control refused (Load Error) on slot a; phone back on b

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: devlab's before/after slot report and the phone's full fastboot text for the control on slot a; then the phone back to current b, a recoverable

Rank
: 1


<a id="walleye-load-error-first-state"></a>
### Restore the first state: boot Android on slot a to completion, adb reboot bootloader, control once

Status
: done

Note
: proven by devlab fastboot state: control accepted from restored first state (walleye-first-state-control-accepted); after the author's power-cycle Android booted on slot a, adb reboot bootloader, fastboot reachable, slot a active and successful

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: devlab's before/after slot report (a successful yes before the try) and the fastboot text of the control try, accepted (phone leaves fastboot) or refused; the phone back on a bootable slot

Rank
: 1


<a id="walleye-load-error-restart-host"></a>
### If refused: cold restart, then the same try on the same slot, with host checks

Status
: open

Note
: Only when the first-state try is refused. Power off fully (not fastboot reboot), start into the bootloader, record fastboot --version and that the phone is on a direct USB port; the control once; the phone back on its start slot.

Waits
: walleye-load-error-first-state

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: devlab's before/after slot report and the fastboot text of one control try after the cold restart, with the fastboot version and port in the log; the phone back on its start slot


<a id="walleye-load-error-uart"></a>
### Read the bootloader's own reason for Load Error over the Android Debug Cable (UART)

Status
: open

Note
: The bootloader logs the reason to the serial console; the Nura wiki names the Pixel 2 Android Debug Cable. Hands needed; the phone itself is not changed.

Waits
: the author's debug cable

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: a captured serial log of one refused control try that names the failing step (LoadImageAndAuth, dtbo, vbmeta or other), attached to a research row


<a id="walleye-boot-refused-why"></a>
### Why the Pixel 2's bootloader refuses even LineageOS's official boot.img from memory (Load Error), and what makes fastboot boot work on it

Status
: done

Note
: Answered in research walleye-boot-refused-why-how-it-stands and decision walleye-load-error-vector; next phone items: walleye-load-error-first-state, then walleye-load-error-restart-host, walleye-load-error-uart

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: research

Proof
: a research row naming the cause or the ranked causes with sources, and the next experiments each with its way back; consistent with the tries in walleye-control-e-f-tries


<a id="walleye-own-images-first-state"></a>
### Our images e, then b, by fastboot boot from the first state (Android booted on slot a, then adb reboot bootloader)

Status
: done

Note
: proven by devlab fastboot boot now: e and b accepted from the first state, neither started, slot flags unchanged; see walleye-own-images-e-b-first-state

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: devlab's before/after slot report and the phone's full fastboot text for each; lsusb polling (by the manifest serial, not VID:PID: surya's gadget 18d1:d001 is on the same host) shows whether a Linux gadget of ours appears (lsusb on the host within 60 s, 172.16.42.1 answering: test walleye-gadget-up)

Rank
: 1


<a id="walleye-vendor-gadget-try"></a>
### fastboot boot the repacked control from the first state; does the gadget answer?

Status
: in-progress

Note
: tried twice, Load Error; gadget-up fails

Waits
: walleye-vendor-gadget-image,walleye-gadget-up-check

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-gadget-up passes (phone enumerates by serial, 172.16.43.1 answers) and the slot report shows what changed


<a id="walleye-vendor-kernel-facts"></a>
### From the running vendor kernel, read the facts our dts guesses (read-only)

Status
: done

Note
: read from the running vendor kernel; see research walleye-vendor-kernel-facts-s1-panic

Waits
: walleye-vendor-gadget-try

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: a research row walleye-vendor-kernel-facts with cmdline, ramoops region and ids, checked against the dts (test: walleye-gadget-up passes during the read)


<a id="walleye-try-g-43"></a>
### Try image g (e at 172.16.43.1) by fastboot boot

Status
: done

Note
: Not needed: the vendor kernel ran with our ramdisk (research walleye-vendor-kernel-facts-s1-panic); mainline images g/h wait until the vendor-kernel gadget (walleye-try-vendor-gadget2) gives a shell and the facts. Closed by the coordinator 2026-11-17.

About
: walleye-initramfs-usb-gadget

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-gadget-up: after devlab fastboot boot now IMAGE the phone leaves fastboot, lsusb shows our gadget (1d6b:0104, walleye-first-boot) within 60 s and 172.16.43.1 answers ping and telnet; devlab's before/after slot report and the fastboot text kept


<a id="walleye-try-h-43"></a>
### Try image h (b at 172.16.43.1) by fastboot boot

Status
: done

Note
: Not needed: the vendor kernel ran with our ramdisk (research walleye-vendor-kernel-facts-s1-panic); mainline images g/h wait until the vendor-kernel gadget (walleye-try-vendor-gadget2) gives a shell and the facts. Closed by the coordinator 2026-11-17.

Waits
: walleye-try-g-43

About
: walleye-initramfs-usb-gadget

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-gadget-up: after devlab fastboot boot now IMAGE the phone leaves fastboot, lsusb shows our gadget (1d6b:0104, walleye-first-boot) within 60 s and 172.16.43.1 answers ping and telnet; devlab's before/after slot report and the fastboot text kept


<a id="walleye-initramfs-address-43"></a>
### walleye's gadget initramfs answers on 172.16.43.1, not surya's 172.16.42.1

Status
: done

Note
: Landed google-walleye-lab #8, #9 (main 4eb1a74). g=e, h=b, i=f with the ramdisk built NET=43; check-bootimg 22 ok each; a-f sha256 identical to before; QEMU selftest PASS at 42 and 43. Tries queued: walleye-try-g-43, walleye-try-h-43.

About
: walleye-initramfs-usb-gadget

Opened
: 2026-11-17

Needs
: build

Proof
: images g and h (e and b with the initramfs address changed, nothing else) built, checked by the same check-bootimg, SHA256SUMS updated; images a-f byte-identical to before


<a id="walleye-bisect-stock-to-ours"></a>
### Bisect from the accepted control to our image: stock kernel+dtb with our initramfs, then one part of ours at a time

Status
: done

Note
: Built and checked on the lab (google-walleye-lab #10, main 01f9138): ~/Projects/walleye-build/bisect/boot-s{0,0n,1,2,3}.img, SHA256SUMS there. s1 ramdisk=ours (172.16.43.1) 3f658748...; s2 kernel=ours as lz4 frame 0bb339b4...; s3 dtb=ours 14d95ef5...; extras s0 (stock parts repacked with footer, 859f0153...; differs from the control in 53 bytes: header id + footer digest) and s0n (no footer/padding, the shape of a-i, 46b575c7...). Try order for the keeper: s0n or s0 first only if the tool chain is in doubt, then s1, s2, s3. Differences stock vs ours: see build/README.md in the lab (no AVB footer/32 MiB pad, os word 0 vs 15.0.0/2026-11, gzip vs lz4 frame kernel, 51 KB dtb with no bootargs vs 376 KB with, cmdline, gzip vs xz ramdisk).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: images s1..s3 built by a script from control/boot.img's own parts (unpack_bootimg) with exactly one part swapped each, check-bootimg ok, hashes in SHA256SUMS; the keeper's tries then show which swap makes the bootloader stop at 'Booting OKAY'


<a id="walleye-try-bisect-s1"></a>
### Try bisect image s1 (ramdisk = ours) by fastboot boot from the first state

Status
: done

Note
: no start, fell back to a; see walleye-try-s1-s2-s3

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-gadget-up for s1; for s2, s3, s0n: the phone leaves fastboot (starts) or stays at 'Booting OKAY', recorded


<a id="walleye-try-bisect-s2"></a>
### Try bisect image s2 (kernel = ours (lz4 frame)) by fastboot boot from the first state

Status
: done

Note
: no start, fell back to a

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-gadget-up for s1; for s2, s3, s0n: the phone leaves fastboot (starts) or stays at 'Booting OKAY', recorded


<a id="walleye-try-bisect-s3"></a>
### Try bisect image s3 (dtb = ours) by fastboot boot from the first state

Status
: done

Note
: accepted, not started, stays in fastboot

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-gadget-up for s1; for s2, s3, s0n: the phone leaves fastboot (starts) or stays at 'Booting OKAY', recorded


<a id="walleye-try-bisect-s0n"></a>
### Try bisect image s0n (stock parts without AVB footer (control of the footer)) by fastboot boot from the first state

Status
: done

Note
: proven by s0n: left fastboot after 12 s, then off USB, same as s0; no writes

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-gadget-up for s1; for s2, s3, s0n: the phone leaves fastboot (starts) or stays at 'Booting OKAY', recorded


<a id="walleye-vendor-gadget-image"></a>
### Repack the control boot.img with our gadget initramfs (kernel, dtb, cmdline untouched)

Status
: done

Note
: Done: ~/Projects/walleye-build/images/vendor-gadget.img (= bisect s1), sha256 3f6587480d1f53fde918de8ebeef67641245e4ee7cbf9704e787075314f3b4e4, 44 bisect checks ok (lab #10, #11). Control header, os word 15.0.0/2026-11, cmdline and AVB footer kept; ramdisk ours at 172.16.43.1 (gzip). Next: walleye-vendor-gadget-try.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: check-bootimg counts the repack: kernel/dtb/cmdline/offsets byte-equal to the control, ramdisk is ours (test: walleye-vendor-gadget-image-built)


<a id="walleye-gadget-up-check"></a>
### Write the failing check walleye-gadget-up in the lab's tests/health

Status
: done

Note
: Done: google-walleye-lab tests/host/gadget-up.sh (host-side: the phone cannot run it; GADGET_WAIT=90 waits). Finds the gadget by our strings (serial walleye-first-boot, product 'Pixel 2 walleye (mainline)'), not VID:PID nor the fastboot serial; then ping and telnet on the manifest address. Run with the phone in fastboot: 'fail ... no USB device with our gadget strings', exit 1.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: the check exists in google-walleye-lab/tests/health, runs, and fails with the phone in fastboot (test: walleye-gadget-up)


<a id="walleye-vendor-gadget-v2"></a>
### A gadget initramfs the vendor 4.4.302 kernel can run: gzip, aarch64, NCM, mknod, init that never exits (image vendor-gadget2)

Status
: done

Note
: Built and landed: google-walleye-lab a41a721 (PR 12), build/build-vendor-gadget.sh; image ~/Projects/walleye-build/images/vendor-gadget2.img sha256 9c0a8946cb65dc46311d6efc87da6393c152e3878d148bc773cc360180806ea4, ramdisk initramfs-vendor.cpio.xz 67c3d77915c4cd3e0c274444984c18f136278a196cf1cdbe6d0da29801ff9f91; 10 checks ok. Try queued as walleye-try-vendor-gadget2.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: vendor-gadget2.img = the control with only the ramdisk swapped (header, AVB footer, 32 MiB as vendor-gadget.img), check-bisect ok; the ramdisk is a gzip cpio of aarch64 binaries (file/readelf shown); then the keeper's try shows walleye-gadget-up (tests/host/gadget-up.sh) or, failing that, a new bootreason


<a id="walleye-try-vendor-gadget2"></a>
### Try vendor-gadget2.img (the control with an xz gadget ramdisk) by fastboot boot from the first state

Status
: done

Note
: Superseded by walleye-try-vendor-gadget3: the same image without the self-reboot timer (decision walleye-try-returns-without-hands). Never tried.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-gadget-up (tests/host/gadget-up.sh, GADGET_WAIT=90): a USB device with our strings enumerates and 172.16.43.1 answers ping and telnet; else record the bootreason

Rank
: 2


<a id="walleye-vendor-gadget3-selfreboot"></a>
### vendor-gadget3: vendor-gadget2 plus a self-reboot timer (240 s, cancelled by /tmp/hold) and a way for the shell to reboot (sysrq b)

Status
: done

Note
: Done, landed google-walleye-lab 0f14a92 (PR 13): build/build-vendor-gadget3.sh, 14 checks ok (ramdisk differs from vendor-gadget2's only in /init; timer line, kmsg node, AVB verify). Stock config: CONFIG_MAGIC_SYSRQ=y, default enable 0x1, PSTORE_RAM y. Init: kmsg marker before the first mount; timer 'sleep 240; skip if /tmp/hold; reboot -f; sysrq b'. a-i, s0-s3, vendor-gadget/2 sums unchanged (checked). Try: walleye-try-vendor-gadget3.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: vendor-gadget3.img = vendor-gadget2 with only the ramdisk's init changed; check script ok; the init's timer and /tmp/hold shown by sh -n and unpacking the ramdisk; hashes in SHA256SUMS; the keeper's try then returns without hands or answers gadget-up


<a id="walleye-try-vendor-gadget3"></a>
### Try vendor-gadget3.img (vendor-gadget2 plus a 240 s self-reboot) by fastboot boot from the first state

Status
: done

Note
: Third try done, watcher alive: kernel_panic before any gadget strings (research walleye-gadget3-third-try). Next: a kmsg-only init build (builder), then try it.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: walleye-gadget-up (tests/host/gadget-up.sh, GADGET_WAIT=90): a USB device with our strings enumerates and 172.16.43.1 answers ping and telnet; and without hands the phone is back in fastboot or Android within ~300 s of the boot (or kept up with touch /tmp/hold over telnet); else record the bootreason

Rank
: 1


<a id="walleye-panic-60s-why"></a>
### Why the vendor 4.4.302 kernel panics about 60 s after fastboot boot with our ramdisk, and how to read the panic text

Status
: done

Note
: Facts in the lab base: research walleye-vendor-kernel-facts-s1-panic (config.gz facts, ramoops 0xa1100000 2 MiB, encryption), the tries of vendor-gadget (gzip, no /init), gadget2/3 (xz): kernel_panic ~74-107 s after fastboot boot, no gadget of ours seen; the stock-ramdisk control boots Android to the logo instead. Public sources, read-only. Name any write (UART enable, dtbo) only with its way back; a UART cable is a purchase for the author.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: research

Proof
: a research row with sources: the likely causes ranked (subsystem restart level panic after PIL/SSR timeouts, MSM watchdog bark/bite not petted, rcu/hung-task, an init exit, vendor ramoops handling), how each is told apart by one cheap try, and how the previous boot's ramoops/console text can be read (vendor init.ramoops.sh encrypts it into /data/vendor_ce/0/ramoops after user unlock: what exactly triggers it, whether an unlock by the author after the try is enough, other routes: /proc/last_kmsg, bootloader log, UART cable with its cost and the fastboot oem uart question)


<a id="walleye-try-vendor-gadget4"></a>
### Try vendor-gadget4.img (idle init, no gadget) by fastboot boot from the first state

Status
: done

Note
: proven by the try: back on Android a at 156 s, bootreason reboot (not kernel_panic): our idle init ran; the gadget bring-up of gadget2/3 is what panics. Next: bisect the gadget bring-up - the coordinator's call.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: the bootreason and the time the phone is back on Android a after fastboot boot: panic at ~75 s = the panic is independent of our gadget code; a return at ~120 s with bootreason not kernel_panic = our init ran and the gadget bring-up of gadget2/3 is what panics; the init's /dev/kmsg marker is not readable from Android (record that)

Rank
: 4


<a id="walleye-cmdline-read"></a>
### Read the running Android's full cmdline (skip_initramfs? root=? dm=?) and the ramoops service files, read-only

Status
: done

Note
: Answered by the vendor-gadget4 try (lab PR 25): our idle init RAN (bootreason=reboot at ~120 s, not kernel_panic), so skip_initramfs is not in effect and the ramdisk is used. The cmdline itself was already read earlier (walleye-vendor-kernel-facts-s1-panic).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: test walleye-boot-cmdline-known: the full /proc/cmdline and chosen/bootargs and /vendor/etc/init/*ramoops* are recorded as a research row; says whether skip_initramfs is present

Rank
: 1


<a id="walleye-panic-time-measure"></a>
### Measure when the panic happens: T_android from a plain adb reboot against the gap of a panic try

Status
: done

Note
: Not needed now: the panic belongs to the gadget bring-up (gadget4 idle init does not panic); the bisect of the bring-up (walleye-gadget-bisect) is the next measure.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: test walleye-panic-time: the 1 s watcher log gives T_android (reboot) and the try's gap; panic time = gap - T_android, written as a research row

Rank
: 2


<a id="walleye-ramoops-after-unlock"></a>
### After one panic try: unlock by the author, then read /data/vendor_ce/0/ramoops as root

Status
: open

Note
: Ramoops region 0xa1100000 (2 MiB) survives the warm reset; the vendor init encrypts it after unlock. Try may be any existing image that panics (gadget3), no new build.

Waits
: the author's unlock after the try; Android on a must not be rebooted before the read

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: test walleye-panic-text: a research row quoting the first panic lines (Kernel panic - not syncing: ...) from the ramoops file, or recording that the file is absent for this kind of reboot

Rank
: 3


<a id="walleye-kmsg-only-init"></a>
### vendor-gadget4: an idle init (no gadget): kmsg marker, sleep 120, reboot -f

Status
: open

Note
: Waits for the cmdline read: if skip_initramfs is present the ramdisk never runs and this try says nothing; build only if absent.

Waits
: walleye-cmdline-read

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: vendor-gadget4.img = the control with only the ramdisk swapped; the keeper's try shows whether the phone panics at the same gap


<a id="walleye-gadget-bisect"></a>
### Split the gadget bring-up into three images, one step each: configfs+gadget dir, +NCM function and config, +UDC bind

Status
: open

Note
: Lab PR 25: gadget4 (idle init) boots, runs and returns by itself at ~120 s (bootreason=reboot), whereas gadget2/3 (full gadget) panic ~75 s after fastboot boot: the panic is in the gadget bring-up. One panic per image kills the log, so one step per image. Everything else byte-identical to gadget4 (control's kernel, dtb, cmdline, header, footer). WHEN DONE: queue the keeper's tries as walleye-try-gadget-bisect (needs phone, rank 1): g5a, then g5b, then g5c, each from the first state, STOP at the first panic (bootreason kernel_panic): that step is the culprit. If g5c does NOT panic, the gadget is up: gadget-up.sh passes (read-only telnet). Note in the item the outcome table (what each result means, hands or not: none expected; init reboots at 120 s, a panic falls back in ~75 s).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: g5a, g5b, g5c = gadget4's init (kmsg marker, 30 s heartbeat, reboot -f at 120 s) plus steps: g5a mounts configfs and creates the gadget dir with idVendor/idProduct/strings (our serial walleye-first-boot, product 'Pixel 2 walleye (mainline)') but no function; g5b adds the ncm function and the config symlink, no UDC bind; g5c adds the UDC bind (a800000.dwc3) and the host-side network address/telnetd; checks ok as for gadget4; hashes in SHA256SUMS
