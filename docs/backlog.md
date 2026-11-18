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

Needs
: research

Proof
: A research row says whether the bootloader needs the dtbo overlay to boot a mainline dtb, and so whether the vendor-overlay nodes in the dmesg can be dropped

Rank
: 8


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
: done

Note
: Done: the gadget initramfs works on our own kernel (first light y2, lab PR 152); the console reliability is walleye-shell-reliable.

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

Blocked by
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
: Add to dts: ufshc + ufsphy (QMP UFS) with the vcc/vccq rails from the wahoo dtsi (RPM regulators, not PMIC-gated); add CONFIG_SCSI_UFS_QCOM and the QMP UFS phy to walleye.config (none in z6); mount nothing, write nothing; also the way to read walleye's own boot.img header.

Blocked by
: walleye-dmesg-clean

Opened
: 2026-11-17

Needs
: build

Proof
: lsblk in the initramfs lists the UFS LUNs and by-name partitions without a write (test: walleye-ufs-lsblk)

Rank
: 4


<a id="walleye-touch-rmi4"></a>
### Touch: Synaptics RMI4 on blsp1_i2c5 (0x20, irq gpio125, reset gpio89)

Status
: open

Note
: Mainline rmi_i2c driver; supplies pm8998 l6 and l28; xd boards add a pmi8998 power gpio - which board revision the lab phone is, is not known.

Blocked by
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: evtest shows touch events from the RMI4 device when the panel is touched (test: walleye-touch-events)


<a id="walleye-bootimage"></a>
### Boot image: header v0, base 0, kernel 0x8000, ramdisk 0x1000000, tags 0x100, page 4096, dtb appended

Status
: done

Note
: Done: the v0 boot image with the control's header offsets is what every working image uses; the stock header was read from LineageOS's official boot.img (lab research walleye-stock-ids-recorded) and our images boot from fastboot boot (first light, PR 152).

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
: done

Note
: Not needed: no dtbo write was ever required; the bootloader accepts our images (first light).

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
: done

Note
: Done: first light (lab PR 152): our own kernel and device tree boot to USB (1d6b:0104, /dev/ttyACM0) from fastboot boot, nothing written. Remaining questions live in their own items.

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
: done

Note
: Not needed: the Load Error was the slot state after repeated refusals (set-active metadata fixes it); the host restart for the host's own modules was the lab reboot into 7.3.4, done by the author.

Blocked by
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
: done

Note
: Superseded: the Load Error was the slot state after refusals (set-active metadata), not a reason for UART; the UART cable stays under walleye-get-uart-setup.

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
: done

Note
: Superseded by the bisect of the bring-up (gadget4 idle init ran, gadget5a ok, 5b panics at the ncm step): see research 2026-11-17-walleye-vendor-gadget5-bisect and walleye-try-ncm-split.

Blocked by
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

Blocked by
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

Blocked by
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
: done

Note
: Already built and tried as vendor-gadget4 (log 2026-11-17-walleye-vendor-gadget4; research 2026-11-17-walleye-gadget4-idle-init-returns: returns at 156 s, the ramdisk runs, skip_initramfs absent). Closed by the builder.

Blocked by
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
: done

Note
: g5a, g5b, g5c built and checked (build/build-vendor-gadget5.sh, 14 checks each); hashes in SHA256SUMS and the log 2026-11-17-walleye-vendor-gadget5. Tries queued as walleye-try-gadget-bisect.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: g5a, g5b, g5c = gadget4's init (kmsg marker, 30 s heartbeat, reboot -f at 120 s) plus steps: g5a mounts configfs and creates the gadget dir with idVendor/idProduct/strings (our serial walleye-first-boot, product 'Pixel 2 walleye (mainline)') but no function; g5b adds the ncm function and the config symlink, no UDC bind; g5c adds the UDC bind (a800000.dwc3) and the host-side network address/telnetd; checks ok as for gadget4; hashes in SHA256SUMS


<a id="walleye-try-gadget-bisect"></a>
### Try vendor-gadget5a, 5b, 5c from the first state, in order; stop at the first panic

Status
: done

Note
: proven by tries: 5a ok (reboot, 156 s), 5b kernel_panic (75 s): culprit is the ncm function/config step; 5c not tried per stop rule. Record 2026-11-17-walleye-vendor-gadget5-bisect

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-ncm-split"></a>
### Split the NCM step of the gadget bring-up: function dir only, then its attributes, then the config symlink

Status
: done

Note
: Built: vendor-gadget5b1 (mkdir functions/ncm.usb0), 5b2 (+dev_addr, host_addr), 5b3 (+configs/c.1 and its string), 5b4 (+symlink = 5b); four steps not three, so the config dir is told apart from the symlink. 15 checks each ok, a-c, gadget1-5c byte-identical (SHA256SUMS -c). Try queued: walleye-try-ncm-split.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: images 5b1 (mkdir functions/ncm.usb0 only), 5b2 (5b1 + writing the function's attributes it has, e.g. host_addr/dev_addr/qmult, if the init wrote any), 5b3 (5b2 + the symlink into configs/b.1), all on gadget5a's init (configfs + gadget dir + strings + the usual 120 s reboot, heartbeat, kmsg markers BEFORE each step), checks as for 5a-c; hashes in SHA256SUMS


<a id="walleye-ncm-panic-why"></a>
### Why the ncm function (or its config symlink) panics the vendor msm-4.4.302 kernel, and which gadget function is safe and gives a channel

Status
: done

Note
: Answered: research walleye-ncm-panic-why-how-it-stands, decision 2026-11-17-walleye-ncm-without-attributes

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: research

Proof
: a research row with sources (the GPL wahoo kernel's f_ncm.c, configfs.c, the gsi/ipa gadget functions, how Android's USB HAL on Pixel 2 composes its gadgets: ffs.adb, mtp, rndis.gsi/ncm?, what the vendor's init.usb scripts write, public sources only): the likely cause ranked, the safe alternatives for a host-visible channel on this kernel (FunctionFS with a tiny static aarch64 helper, gsi functions, mass_storage with a read-only image as a crude log window, MTP, ACC), each with how one cheap image tells it, and a recommendation


<a id="walleye-try-ncm-split"></a>
### Try vendor-gadget5b1, then 5b2, 5b3, 5b4 by fastboot boot from the first state; stop at the first that panics

Status
: done

Note
: proven by b1 returns, b2 panics at 75 s (research 2026-11-17-walleye-ncm-split-b2-panics); next: the noattr try (n1, n2)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-ncm-noattr-images"></a>
### Build ncm-noattr images n1, n2: the 5b4 steps and the bind, without the dev_addr/host_addr writes

Status
: done

Note
: Built: vendor-gadget-n1.img b2e96b42..., vendor-gadget-n2.img b007d1e3... in ~/Projects/walleye-build/images, 17 checks each ok, hashes in SHA256SUMS there and in session log 2026-11-17-walleye-vendor-gadget-n. Next: walleye-try-ncm-noattr (keeper).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: check-bisect passes: grep for dev_addr|host_addr|qmult|ifname finds none in n1/n2 init, exactly the step's parts, hashes in SHA256SUMS

Rank
: 1


<a id="walleye-try-ncm-noattr"></a>
### Try ncm-noattr n1 then n2 from the first state, stop at the first panic

Status
: done

Note
: proven by n1/n2 tries: n1 returns, n2 binds without a panic and enumerates with our strings; host cdc_ncm fails on the MAC string (walleye-ncm-noattr-n1-n2). Next: a function the host binds without a MAC string (ecm, rndis) or FunctionFS helper

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: TEST

Rank
: 2


<a id="walleye-gadget-host-bind"></a>
### Get the host to bind the gadget (ecm/rndis variants)

Status
: done

Note
: Dropped: ECM and RNDIS are not in the vendor kernel config (decision walleye-host-channel-order). Replaced by walleye-massstorage-window and walleye-ffs-helper.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17


<a id="walleye-massstorage-window"></a>
### m1: a mass_storage gadget as a log window (device writes a file, the host reads the raw LUN)

Status
: done

Note
: Built: vendor-gadget-m1.img a3cf08d3..., initramfs dd59bce1... in ~/Projects/walleye-build/images (SHA256SUMS), 22 checks ok (build/build-vendor-gadget-m.sh); host reader tests/host/massstorage-read.sh (not yet run against a real LUN). Next: walleye-try-massstorage (keeper).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: vendor-gadget-m1.img = the control with only the ramdisk swapped; init as n2's up to the UDC bind but with function mass_storage.0 (lun.0/file = a 2 MiB file in the ramdisk filesystem created by dd, ro=1, removable=1) instead of ncm, NO attribute writes beyond file/ro/removable/cdrom/nofua (check each exists in this kernel's mass_storage configfs, read the stock config facts); every 5 s the init rewrites the file in place (dd conv=notrunc) with: kmsg-marker, uname, cmdline, dmesg tail, mounts, heartbeat counter; reboot -f at 180 s; checks as for n2; hashes in SHA256SUMS; the host-side reader script tests/host/massstorage-read.sh (find the LUN by our strings via sysfs vendor/model, read with dd iflag=direct, print the text) in the lab repo


<a id="walleye-ffs-helper"></a>
### FunctionFS with CDC-ACM descriptors and a tiny static aarch64 helper: the interactive console channel

Status
: done

Note
: Answered: research walleye-ffs-acm-helper-how-it-stands, decision 2026-11-17-walleye-ffs-acm-single-interface; build item walleye-ffs-helper-build, phone item walleye-ffs-helper-try.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: research

Proof
: a research row with the helper's design: the ffs descriptors and strings (CDC union/header/call-management/ACM functional descriptors, one interrupt-in and two bulk endpoints), the init steps (mount ffs, the order of writing descriptors before the UDC bind, as Android's own ffs.adb init composes it, read-only look at the public init.*usb*.rc), the bridge to a shell on a pty, how the host sees /dev/ttyACM0; the kernel's own GPL tools/usb/ffs-test.c as the base; which image tells, and its failure modes (panics, no enumerate)


<a id="walleye-try-massstorage"></a>
### Try m1 from the first state, then read the log window on the host for 90 s

Status
: done

Note
: m1 tried and its window read as the lab user (lab PR 70: lpoco-host acceptance passed); ml/pk variants superseded.

Waits
: host: usb_storage for the running kernel (reboot into 7.3.4)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-ffs-helper-build"></a>
### Write the FunctionFS ACM helper (C, static aarch64, GPL base ffs-test.c) and the n3 ramdisk/image

Status
: done

Note
: Built: images/vendor-gadget-n3.img sha256 6672eb844fb5435ebb336e33a0b59db069bc2e62a6e01f98d760f5646b1d214c (helper, ramdisk sums in the session log 2026-11-17-builder-vendor-gadget-n3-image); 21 checks ok

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: builds static aarch64 helper and image n3 with check-bootimg counting ok; tests/host/ffs-acm-up.sh exists and fails without the phone


<a id="walleye-ffs-helper-try"></a>
### Phone: boot image n3 once; host sees ttyACM and a shell answers

Status
: done

Note
: n3/n3r tried (lab PR 76): cdc_acm refuses a FunctionFS interface without functional descriptors: parked (decision 2026-11-17-walleye-ffs-console-parked).

Waits
: host: cdc_acm for the running kernel (reboot the lab host into 7.3.4)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: tests/host/ffs-acm-up.sh

Rank
: 2


<a id="walleye-ffs-helper-rust"></a>
### Rewrite the FunctionFS ACM helper in Rust (static aarch64-unknown-linux-musl), the C one stays a probe

Status
: done

Note
: Built: build/ffs-acm-rs (Rust, libc, musl aarch64, fmt/clippy clean, descriptors byte for byte the C probe's by a test), build/build-vendor-gadget-n3r.sh (21 checks ok, reproducible). sha256 ffs-acm-rs 5de3fff0e4aa6bdfa56e71ea8ac7bec6b588022d695aff818dee7150f070743f; initramfs-vendor-n3r.cpio.xz 43205c57b0849f3ee9363746832d5aa26d8228b6e5b20847d49026c79a896e7b; vendor-gadget-n3r.img 7eb5a98ae354959150aff3a4cb8deb4d8cd6399240863e5bdb654389dc79a7c0. Try queued: walleye-ffs-rust-try.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: a Rust crate in the lab (build/ffs-acm-rs or as the lab's layout has it) built with lab-tools health/build.sh's way (rustup in ~/.local/share/devlab-cross, aarch64-linux-gnu-gcc as the linker, cargo build --release --locked --target aarch64-unknown-linux-musl), the same descriptors/steps as the C helper (decision 2026-11-17-walleye-ffs-acm-single-interface), a new image n3r differing from n3 only in the helper binary; checks as for n3; hashes; cargo fmt/clippy clean; no arguments or environment, so no argenv; the keeper's try then repeats n3's proof (tests/host/ffs-acm-up.sh)


<a id="walleye-host-access-via-lab-prepare"></a>
### Give the lab user access to our gadget's /dev/sdX and /dev/ttyACM* through the lab-prepare tool, then read m1 and n3

Status
: done

Note
: Done: lpoco host prepare passed its acceptance test (lab PR 70, m1 read as the lab user); the banner serial finding closed the last tty question (decision 2026-11-18-walleye-banner-serial-not-root-exec).

Waits
: the lpoco lab-prepare tool (orchestration); the author will not add the udev rule by hand: its first use here is its acceptance test

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17


<a id="walleye-retry-ffs-acm-after-reboot"></a>
### Try n3 (FunctionFS ACM) again now the host runs 7.3.4 (cdc_acm loads): a console on /dev/ttyACM0

Status
: done

Note
: Superseded by walleye-ffs-rust-try: the console try uses the Rust helper image n3r (the C n3 is a probe).

Waits
: the lpoco lab-prepare tool (host access to /dev/ttyACM*); the author waits for the tool

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: tests/host/ffs-acm-up.sh passes: /dev/ttyACM0 appears on the host, a shell answers on it (uname -a, cat /proc/cmdline read-only), the phone returns by itself (the init's reboot)

Rank
: 2


<a id="walleye-dtbo-overlay-merge-why"></a>
### Our mainline dtb is never started (stock kernel + our dtb stays in fastboot): does the bootloader's dtbo overlay merge need __symbols__ and the nodes the overlays patch?

Status
: done

Note
: The host-independent work while the lab-prepare tool is awaited. Facts in the lab base: stock dtb has qcom,msm-id <0x124 0x20001>, compatible qcom,msm8998-mtp qcom,mtp, no board-id/pmic-id, 376 KB with bootargs; ours 51 KB, compatible google,walleye, no __symbols__ unless built with -@ (check the builder's script); research row walleye-boot-refused-why-how-it-stands ranked the dtbo merge as the first cause. dtbo.img is data, read with dtc only; clean room: structure and node names, no proprietary code. A write to dtbo is not proposed.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: research

Proof
: a research row: the structure of the LineageOS walleye dtbo.img (13 entries, id 0x1bc ...; androidboot.dtbo_idx=3 picks one) read with dtc/fdtdump from the public build already downloaded (~/Projects/walleye-build/control/dtbo.img, sha256 fb09009b...b8f6, official), which base-dtb paths/labels/__symbols__ the overlay fragments target, whether our dtb (compiled how: -@?) lacks them, how ABL selects and merges (public Qualcomm ABL source: dtbo idx, msm-id/board-id match, merge failure behaviour), the exact dtb properties to add or to compile, and one cheap image that tells (e.g. s3 rebuilt with our dtb + -@ symbols + the target nodes, or the stock dtb with only the compatible changed); say which of the bisect facts support it (s0n/s0 start; s3 'Booting OKAY' then stays in fastboot; s2 falls back)


<a id="walleye-dtb-overlay-symbols-build"></a>
### Build our dtb with -@ and the 137 overlay labels; host test passes

Status
: done

Note
: Done 2026-11-17: dt/msm8998-google-walleye-overlay-stubs.dtsi (130 stubs, generated by build/gen-overlay-stubs.sh from entry 3's __fixups__; 7 of 137 labels were ours), dtb built with -@ (DTC_FLAGS in build-kernel.sh). tests/host/dtbo-overlay-merges.sh PASSES on out/msm8998-google-walleye.dtb (sha256 a9cbbef6565dcf4db3a1df397a8a31f55ee58d784f596855262056761650d517). dtc W=1: no warning from our files. Image s3o = control with only our new dtb: ~/Projects/walleye-build/images/boot-s3o.img sha256 f61e5e8ccfa51bf6c6bad6c9e5a660a80970798e4e1f57831dd4374b30c27818 (build-bisect.sh 6th argument; check-bisect 54 ok 0 failed; s0, s0n, s1, s3 byte for byte as before).

Blocked by
: walleye-dtbo-overlay-merge-why

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: walleye-dtbo-overlay-merges


<a id="walleye-s4-overlay-symbols-try"></a>
### Phone: one image, stock kernel + our dtb with overlay symbols (s3 redone); read the result

Status
: done

Note
: proven by fastboot boot: left fastboot at 12 s, Android on a at 36 s, flags unchanged; see research walleye-s4-overlay-symbols-try

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-mainline-first-light"></a>
### First light of OUR mainline v7.3 kernel: the control's header/footer, our kernel (gzip and lz4 variants) + the overlay-symbols dtb + a gadget init that shows itself in lsusb

Status
: done

Note
: Built: ml1 and ml2 (see the decision 2026-11-17-walleye-mainline-light-images); try queued as walleye-try-mainline-light.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: images ml1 (kernel as Image.gz) and ml2 (kernel as lz4 frame like the stock) built by script from the control's own parts with exactly: our kernel, our -@ dtb (with the overlay stubs), our ramdisk; the ramdisk is a mainline-capable gadget init (mainline has devtmpfs and configfs: mass_storage or NCM-with-attributes are fine there, but keep it as simple as the vendor-kernel m1: enumerate with serial walleye-first-boot, product 'Pixel 2 walleye (mainline)', a 2 MiB mass_storage LUN window; open /dev/watchdog and never pet it (the SoC watchdog then resets the phone ~30-60 s later; if the dts needs the apss watchdog node enabled, enable it); cmdline: the stock cmdline except what mainline must have (earlycon msm_serial_dm,0xc1b0000 clk_ignore_unused ignore_loglevel panic=5 plus the stock androidboot.* kept), check-bisect-like checks ok, hashes in SHA256SUMS; the keeper's try then tells by lsusb (needs no host permissions): a device with our strings = first mainline light


<a id="walleye-try-mainline-light"></a>
### Phone: boot ml1, then ml2, each once from the first state; lsusb tells first light

Status
: done

Note
: Tried 2026-11-17: ml1, ml2 and (item walleye-try-ml3-ramoops) ml3 all fail: no USB light, Android on a at ~70 s. pstore read done, empty. Next: UART, the walleye coordinator's call.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: tests/host/mainline-light-up.sh

Rank
: 1


<a id="walleye-ml3-vendor-ramoops"></a>
### ml3: ml1 with the ramoops node at the phone's own region (0xa1100000, 2 MiB, vendor record/console sizes), so the stock Android can read our kernel's log after the reset

Status
: done

Note
: Built 2026-11-17: boot-ml3.img sha256 5c2574706882f5c8f40b134bdf501e22c092093e494b5aad25efdabe4ba105c4 in ~/Projects/walleye-build/images-ml3; ramoops@a1100000 checked with fdtget, no carve-out overlaps it, PSTORE_CONSOLE/RAM =y. Try queued: walleye-try-ml3-ramoops.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: boot-ml3.img = boot-ml1.img with only the dts ramoops node changed: reg <0x0 0xa1100000 0x0 0x200000>, record-size 0x80000, console-size 0x20000, no pmsg and no ftrace region unless the vendor values say so (read /sys/module/ramoops/parameters on Android a, and the vendor dts' reserved-memory node names in the public GPL dts, for pmsg/ftrace sizes and for whether 0xa1100000 collides with another carve-out in our dts: say it), checks as for ml1/ml2, hashes; also read-back: the new dtb's ramoops node printed with fdtget in the check; the kernel's console is saved into it by pstore/console (check CONFIG_PSTORE_CONSOLE and PSTORE_RAM are y in our config)


<a id="walleye-rust-ci-ffs-acm-rs"></a>
### CI runs cargo fmt --check, clippy -D warnings and test for build/ffs-acm-rs

Status
: done

Note
: Half of walleye-code-hygiene-rust-ci, done by walleye-builder: fmt, clippy -D warnings and test pass locally on the lab (1 test ok); job added to .github/workflows/check.yml

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: check.yml rust job green on the PR; read back from main


<a id="walleye-retry-massstorage-after-reboot"></a>
### Try m1 again now the host runs 7.3.4 (usb_storage loads): read the first text the phone writes

Status
: done

Note
: proven by tests/host/massstorage-read.sh pass: gadget 3-2 /dev/sda WALLEYE-LOG m1 beat=4 up=21.34s; research 2026-11-17-walleye-m1-window-read. lpoco-host acceptance: PASS. Next: walleye-ffs-rust-try (n3r) - ttyACM now readable too.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: massstorage-read.sh (tests/host) prints the init's window text from the phone: uname, cmdline, mounts, dmesg tail of the vendor kernel with our init running, the heartbeat counter growing; kept as material in the lab and recorded as research with what it shows

Rank
: 1


<a id="walleye-ffs-rust-try"></a>
### Phone: boot image n3r (the Rust FunctionFS ACM helper) once; host sees ttyACM and a shell answers

Status
: done

Note
: Tried once (n3r on slot b): the gadget enumerates but the host's cdc_acm refuses it (Zero length descriptor references, -22); f_fs cannot carry the CDC functional descriptors it needs. Parked: decision 2026-11-17-walleye-ffs-console-parked.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: tests/host/ffs-acm-up.sh

Rank
: 2


<a id="walleye-try-ml3-ramoops"></a>
### Phone: boot ml3 once from the first state, then read our kernel's console from the stock Android's pstore

Status
: done

Note
: Tried 2026-11-17: ml3 no light, Android on a at 72 s, pstore empty (research walleye-ml3-pstore-empty). Proof test tests/host/ml3-ramoops-read.sh fails as the facts stand. The remaining read is UART (decision walleye-panic-read-order step 5) or an earlycon/earlyprintk-free route; to the walleye coordinator.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: tests/host/ml3-ramoops-read.sh (to write: pstore console of our boot read back)

Rank
: 1


<a id="walleye-ml4-timing-channel"></a>
### ml4: a timing channel from our mainline init - the reset delay says how far the kernel came, no UART, no USB, no pstore needed

Status
: done

Note
: Built: images-ml4/boot-ml4.img sha256 105364591ce1f46a2a828caf1f80c851399487ab4a59bd777210a9a72e13ea84; try queued as walleye-try-ml4

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: boot-ml4.img = ml3 with ONLY the ramdisk's /init changed (kernel, dtb and header bytes equal ml3's): /init mounts proc/sys/devtmpfs, opens /dev/watchdog (a safety net: the SoC resets by itself ~30 s later if the rest hangs), counts which of a fixed checklist holds - (1) init reached userspace [always 1], (2) /sys/class/udc has an entry, (3) a dwc3 platform device is bound to its driver, (4) /sys/bus/platform/devices has a ramoops device, (5) /proc/iomem lists the ramoops region, (6) the PSCI/reset path answers (/sys/firmware/devicetree/base/psci exists), (7) UFS host is not enabled (expected 0) - then sleeps 5*score seconds (bins of 5 s) and runs reboot -f (sysrq b fallback); the checklist and the bin table written in the item/log; checks as ml3's plus: the init script has no bashisms, runs under busybox ash -n; a QEMU-free host test reading the bin table. The keeper's try then reads the time Android is back (host timestamps at fastboot-boot, at adb) and subtracts the known base (the control's: boot+Android a ~ 25-36 s); 70 s = the safety watchdog fired: init never ran or hung before the sleep


<a id="walleye-try-ml4"></a>
### Phone: boot image ml4 twice from the first state; read the reset time as a score

Status
: done

Note
: proven by two tries (75 s, 76 s, slot b, nothing flashed, pstore empty): bin table reads score 7 but is not told apart from the safety watchdog (ml1-ml3 back at 69-72 s); see research 2026-11-17-walleye-ml4-two-tries; the next image is the coordinator's call

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-ml5-timing-below-the-watchdog"></a>
### ml5: timing bins that finish BEFORE the dead-kernel reset (~33 s): a baseline image that reboots at once, and a score image with 3 s per point

Status
: done

Note
: built: images-ml5, hashes in research walleye-ml5-built; try walleye-try-ml5 queued

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: boot-ml5a.img = ml4 with /init changed to: mount proc/sys, kmsg marker, then reboot -f at once (sysrq b fallback), no watchdog games; boot-ml5b.img = ml4 with the sleep scaled to 3 s per score point (max 7 points = 21 s; the watchdog petting kept) and the bin table in tests/host/ml5-bin-table.sh; both with kernel/dtb/header bytes equal to ml3's; checks as ml4's; hashes in SHA256SUMS; the table states the expected return times: base = the time ml5a returns (fastboot boot to adb), ml5b = base + 3*score


<a id="walleye-try-ml5"></a>
### Phone: boot ml5a twice, then ml5b twice from the first state; record the four return times

Status
: done

Note
: proven by the phone: ml5a 75 s and 75 s, ml5b 75 s and 78 s (fastboot boot to adb, from slot b, nothing flashed); ml5a ~70 s+ = the kernel never reached init; ml5b equals ml5a within 3 s, so no init ran. Next: a way to see before init (earlycon/UART, or a vendor-kernel-side check), the coordinator's call.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-get-uart-setup"></a>
### Get the UART debug setup for the Pixel 2 (breakout + 1.8 V adapter, or USB-Cereal) and confirm it

Status
: open

Note
: The parts and wiring are in the hardware table (uart-debug-setup). Once owned: try it, record the log, and mark the fit confirmed with its evidence.

Waits
: the author: choosing and buying the parts (tad q hardware)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build


<a id="walleye-porthole-known-good"></a>
### A known-good mainline kernel on this phone: porthole's 7.2.2 Pixel 2 XL kernel (their config, patches, dts) with our ml5a-style immediate-reboot init, to see whether ANY mainline kernel reaches init here

Status
: done

Note
: built: pk1/pk2 in walleye-build/images-pk, checks ok, hashes in research/walleye-first-fastboot-boot/2026-11-18-pk; try queued as walleye-try-pk

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: pk1/pk2 images: the control's header/footer, porthole's kernel built from its public pmaports package (kernel version, config and patch series as they publish; taimen dts for pk1, their walleye dts for pk2), their appended dtb with the overlay symbols if their dtb lacks __symbols__ (check with tests/host/dtbo-overlay-merges.sh and say), our ml5a init (reboot at once) as the ramdisk; checks as ml5's; build recipe reproducible from the lab; sources and licences kept; hashes


<a id="walleye-try-pk"></a>
### Phone: boot pk1 twice, then pk2 twice from the first state; record the four return times

Status
: done

Note
: proven by four tries: pk1 75 s, 75 s; pk2 75 s, 75 s - no init ran

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-no-earlycon"></a>
### Every mainline image carries earlycon msm_serial_dm,0xc1b0000: on a phone whose debug UART is not enabled that is a likely first-printk hang. Build the same images WITHOUT earlycon/console on the UART

Status
: done

Note
: built: pk3/ml6a in walleye-build/images-nocon, 22 checks ok, hashes in research/walleye-first-fastboot-boot/2026-11-18-nocon; try queued as walleye-try-no-earlycon

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: pk3 = pk1 and ml6a = ml5a with ONLY the kernel command line changed: earlycon=... and any console=ttyMSM*/ttyHS* removed (say what the cmdline was and what remains; keep clk_ignore_unused ignore_loglevel panic=5 and the stock androidboot.* pieces); kernel/dtb/ramdisk/header/footer bytes equal to pk1's and ml5a's; checks as before; hashes


<a id="walleye-try-no-earlycon"></a>
### Phone: boot pk3 twice, then ml6a twice from the first state; record the four return times

Status
: done

Note
: proven by four returns: pk3 63/48 s, ml6a 75/75 s; ml6a no init, pk3 undecided; next: coordinator picks the log channel (ramoops read / UART)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-bisect-porthole-vs-ours"></a>
### Bisect porthole's kernel (pk3: reaches further) against ours (ml6a: dead): swap one part at a time, all without earlycon

Status
: done

Note
: built: pk4/pk5/pk6 in walleye-build/images-pk456, 31 checks ok, hashes in research/walleye-first-fastboot-boot/2026-11-18-pk456; try queued as walleye-try-bisect-pk

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: three images, each from pk3/ml6a's own bytes with exactly one swap, ramdisk = ml5a's init (reboot at once), cmdline = pk3's (no earlycon): pk4 = porthole's kernel + OUR -@ dtb (with overlay stubs) ; pk5 = OUR kernel (the ml6a one) + porthole's taimen dtb (with symbols/msm-id as in pk1); pk6 = porthole's kernel + pk3's dtb + the timing init of ml5b (3 s per point score) to learn which hardware their kernel shows in userspace; checks as before; hashes; the item states the expected return times


<a id="walleye-try-bisect-pk"></a>
### Phone: boot pk3 once, then pk4, pk5, pk6 twice each from the first state; record the seven return times and read pstore

Status
: done

Note
: proven by the seven returns in walleye-pk-bisect-returns: our dtb is the fault

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-pk5-light"></a>
### First light of OUR kernel: pk5 (our kernel + porthole's taimen dtb, no earlycon) with a gadget init: configfs ACM console (a shell on /dev/ttyGS0) and the mass_storage log window

Status
: done

Note
: built: boot-pk5l.img in walleye-build/images-pk5l, 39 checks ok, hashes in research/walleye-first-fastboot-boot/2026-11-18-pk5l; try queued as walleye-try-pk5l

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: boot-pk5l.img = pk5's kernel, dtb, header, footer, cmdline (bytes equal), ramdisk = a mainline gadget init: mount proc/sys/devtmpfs/configfs, gadget with strings serial walleye-first-boot / product 'Pixel 2 walleye (mainline)', functions acm.GS0 (shell: busybox ash on /dev/ttyGS0 through a getty-less loop, respawning) and mass_storage.0 (2 MiB file window rewritten every 5 s with uname, cmdline, dmesg tail, /proc/iomem, /sys/class/udc, pstore), UDC bound to the first /sys/class/udc entry; /dev/watchdog opened and petted for a bounded 180 s, then reboot -f (sysrq b fallback); a host test tests/host/mainline-acm-up.sh (finds the device by our strings, waits for /dev/ttyACM*, sends 'uname -a; cat /proc/cmdline', expects the answer; and reads the LUN window via massstorage-read.sh); checks as before; the mainline kernel config must have USB_CONFIGFS_ACM, MASS_STORAGE, DWC3 peripheral and the QCOM glue (say if our config lacks any; porthole's dtb carries its dwc3 node: say what the kernel needs from it)


<a id="walleye-our-dtb-bisect"></a>
### Find the node(s) of OUR dtb that stop boot: diff it against porthole's taimen dtb (the one that reaches init with our kernel), bisect by moving nodes

Status
: done

Note
: built: d1..d6 in walleye-build/images-dtb, 90 checks ok, hashes in research/walleye-first-fastboot-boot/2026-11-18-dtb-bisect; try queued as walleye-try-dtb-bisect

About
: walleye-first-fastboot-boot

Opened
: 2026-11-17

Needs
: build

Proof
: a research row naming the node or property whose presence in our dts stops boot (reset at ~33 s), found by images each with our dtb minus a node group (reserved-memory carve-outs, ramoops, the watchdog node we added, regulators, the dwc3/usb nodes, tlmm...), or porthole's dtb plus a node group of ours; the fix as a patch to dt/msm8998-google-walleye.dts; hashes


<a id="walleye-try-dtb-bisect"></a>
### Phone: boot pk4 once more, then d1..d6 twice each from the first state; record the thirteen return times and read pstore

Status
: done

Note
: 13 tries, all 75 s (no init); no group of d1..d6 is the cause; recorded walleye-dtb-bisect-tried

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 2


<a id="walleye-try-pk5l"></a>
### Phone: boot pk5l twice from the first state; read the ACM console and the window, record the first mainline dmesg

Status
: done

Note
: tried twice: no ACM, Android back at 57/42 s; recorded walleye-pk5l-tried; next walleye-try-dtb-bisect

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-pk5l-ramoops"></a>
### pk5l2: pk5l with the dtb's ramoops node moved to the phone's own region, so the panic that ends pk5l after ~5-20 s can be read from Android's pstore

Status
: done

Note
: built: boot-pk5l2.img in walleye-build/images-pk5l2, 20 checks ok, hashes in research/walleye-first-fastboot-boot/2026-11-18-pk5l2; try queued as walleye-try-pk5l2

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: boot-pk5l2.img = pk5l with ONLY the ramoops node of the (porthole taimen) dtb edited (fdtput/dtc on the dtb, no kernel or init change): reg 0xa1100000 size 0x200000, record-size 0x80000, console-size 0x20000, pmsg-size 0 (as in our decision 2026-11-17-walleye-ramoops-at-the-phones-region), the old node/region removed or checked for overlap; fdtget read-back in the checks; kernel config has PSTORE_CONSOLE and PSTORE_RAM (say); hashes; recipe in build/


<a id="walleye-try-pk5l2"></a>
### Phone: boot pk5l2 twice from the first state; after each read /sys/fs/pstore/* and /data/vendor_ce/0/ramoops/*, record the first mainline panic or console text

Status
: done

Note
: tried twice: no mainline panic text found (research walleye-pk5l2-tried); next: prove the mainline kernel reaches ramoops at all, or return to the dtb diff

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-overlay-enables-what"></a>
### What does the stock dtbo overlay (entry 3) switch on in OUR dtb that it does not in porthole's taimen dtb? Analyse it on the host, then neutralize label by label

Status
: done

Note
: analysed: overlay inert on our dtb; images w1..w5, wr, wa built (research walleye-overlay-enables-what-result, walleye-taimen-walleye-built); try queued

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: (1) a research row: fdtoverlay (or ufdt-equivalent) merges entry 3 of the control's dtbo.img into (a) pk5's taimen dtb (works) and (b) our dtb (dead), the merged dtbs decompiled and diffed: the nodes/properties the overlay sets or adds (status okay, panel, touch, regulators, clocks, pinctrl, charger...) that exist in ours as real SoC nodes but are absent or differently wired in taimen's; ranked as suspects (a driver that probes unclocked hardware and hangs the SoC is the pattern: a reset ~33 s after the jump); (2) images L1..Ln = our dtb (the ml6a one) with the __symbols__ entries of groups of those labels pointed at harmless dummy nodes so the overlay lands on the dummy and the real node is not enabled (the merge still succeeds: tests/host/dtbo-overlay-merges.sh passes), ramdisk = ml5a's init, cmdline as ml6a; checks, hashes; the bisect plan (binary halving of the label set)


<a id="walleye-try-taimen-walleye"></a>
### Phone: boot wa twice from the first state, then w1..w5 and wr twice each; record the return times (~42 s init ran, ~75 s dead)

Status
: done

Note
: proven by 12 tries, all alive (research walleye-w1-w5-wr-tried): cause is a property; next a property-level bisect

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-property-bisect"></a>
### Property-level bisect between wa (alive) and pk2 (dead): same node set and statuses, so a PROPERTY of pk2's dtb (or ours) stops boot

Status
: done

Note
: built: wp1..wp5, wpa (68 checks, rebuild identical), research walleye-property-diff and walleye-property-bisect-built; tried by walleye-try-property-bisect; mirror images from our dtb only if the cause sits in a group ours alone

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: (1) a research row: wa and pk2 decompiled (dtc -I dtb -O dts -s) and diffed property by property: every differing property listed by node (compatible, model, msm-id/board-id, memory reg, reserved-memory regions and their no-map/reusable, cpu capacities/idle-state, timer frequency, psci, chosen, pinctrl/regulator limits, clock rates, interrupts...), grouped (top-level, memory+reserved-memory, cpus, soc nodes...); (2) images wp1..wpn = wa with ONE group of pk2's property values copied in (binary-halving plan: first half/second half of the groups, then narrower), same kernel/ramdisk/cmdline as wa; checks, hashes; each states its expected return (alive <=66 s, dead >=69 s). Also build the mirror from OUR dtb: o1.. = our dtb with one group of wa's values copied in, if the first plan leaves the culprit in a group that is ours alone


<a id="walleye-try-property-bisect"></a>
### Phone: boot wpa twice from the first state, then (if dead) wp1..wp5 twice each; record the return times (<=66 s alive, >=69 s dead)

Status
: done

Note
: proven by wpa x2: 60 s, 60 s (alive); cause is not a property; next: phandle order / dtb size

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-pk5-gadget-stages"></a>
### Stage the mainline gadget bring-up on the ALIVE base (pk5: our kernel + porthole's taimen dtb): one step per image, each holds 60 s so alive and dead are 30 s apart

Status
: done

Note
: built: g1..g3, 58 checks ok, rebuild identical; try queued as walleye-try-pk5-stages

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: images g1..g3 on pk5's kernel/dtb/cmdline/header/footer (bytes equal), ramdisk init: mount proc/sys/devtmpfs/configfs, then g1 = gadget dir + strings (serial walleye-first-boot, product 'Pixel 2 walleye (mainline)') and nothing else; g2 = g1 + mass_storage.0 function (2 MiB file LUN, ro) linked into a config, NO UDC bind; g3 = g2 + UDC bind to the first /sys/class/udc entry (expects USB enumeration on the host as 1d6b:0104 within ~25 s of the jump: the first mainline light, readable with plain lsusb); every init pets /dev/watchdog and holds 60 s then reboot -f (sysrq b fallback); no ACM, no shell loop (pk5l had them and ended early); expected returns: alive (init ran) ~12+60+36 = ~108 s, dead ~75 s (outside watchdog), early panic with panic=5 ~45-60 s; checks, hashes; table in the item


<a id="walleye-try-pk5-stages"></a>
### Phone: boot g1, g2, g3 (images-g) twice each from the first state, stop at the first that dies; lsusb polled by port every second for 90 s; pstore read after each

Status
: done

Note
: proven by 6 tries: no gadget enumerates, g2 returns at 138 s; research walleye-pk5-stages-tried

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-phone-needs-hands"></a>
### The Pixel 2 is gone from USB since the pk5-stages tries (2026-11-18 ~02:30 UTC): phone work is parked until the author is back

Status
: done

Note
: Closed: the phone came back on its own (the keepers' tries since 2026-11-18 05:00 UTC ran on it; first light and the first mainline shell followed). The lesson stays in walleye-battery-watch and walleye-battery-ok-timeout.

Waits
: the author: the phone is off USB (hold Power 10-30 s; if it does not react, charge it first)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18


<a id="walleye-battery-watch"></a>
### The keeper reads the battery before every try and leaves the phone charging (in Android, not in fastboot) between tries

Status
: done

Note
: Built: battery-ok.sh, idle-android.sh, battery-selftest.sh (pass) and the keeper rule in AGENTS.md. Next: the keeper runs battery-ok.sh before each try and idle-android.sh at session end; first live reading is theirs.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: (1) tests/host/battery-ok.sh (lab): reads fastboot getvar battery-voltage (or adb dumpsys battery in Android), prints the value, exits 1 below 3700 mV; the keeper's brief (the lab's keeper note in the item/AGENTS.md) says to run it before each try and to stop and record when it fails; (2) the keeper ends each session with the phone booted into Android on a good slot (fastboot reboot, adb) while idle, not left in fastboot; checked by a test that records the end state


<a id="walleye-pstore-warm-reset"></a>
### Why our mainline kernel's console never survives in pstore, and how a MSM8998 kernel asks for a WARM reset (reboot=w, psci reset, qcom pon, the SDI/dload) so the ramoops region keeps its text

Status
: done

Note
: Answered in research walleye-pstore-warm-reset-how-it-stands and decision 2026-11-18-walleye-warm-reset-by-watchdog-bite-first; tries queued as walleye-pstore-t1/t3/t4

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: research

Proof
: a research row with sources (porthole's cmdline reboot=w and qcom_wdt.arm_on_probe, kernel docs, postmarketOS and kcxt notes, Qualcomm PSCI/TZ behaviour, the vendor kernel's own warm-reset path read from the GPL source): ranked causes of the empty pstore (DDR cleared by a cold reset, the bootloader zeroing the region, the ramoops node/size mismatch, the vendor reader consuming it), and one or two cheap image changes that would tell (cmdline reboot=warm, the init doing a psci warm reset, panic=5 plus a deliberate panic via sysrq c at a known time) with their expected pstore results


<a id="walleye-pstore-t1-sysrq-reset"></a>
### Image T1: init marks /dev/kmsg, waits 20 s, sysrq c (panic=5); read console-ramoops after the return

Status
: done

Note
: Built: images-t1/boot-t1.img (research walleye-t1-built); try queued as walleye-pstore-t1-try

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: tests/health: pstore holds the marker after the T1 return (expected absent: records cause B)


<a id="walleye-pstore-t3-wdog-bite"></a>
### Image T3: init opens /dev/watchdog (kpss-wdt) with a short timeout and never pets it; read console-ramoops after the bite

Status
: done

Note
: Tried once (lab PR 122): back at 42 s, no marker: inconclusive without a witness; the pstore chase is paused by decision 2026-11-18-walleye-vendor-kernel-as-instrument.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: tests/host/pstore-t3-read.sh: pstore holds the T3 marker, last tick under 15, no no-bite line (expected present)

Rank
: 1


<a id="walleye-pstore-t1-try"></a>
### Try T1: fastboot boot boot-t1.img, wait 90 s, read console-ramoops for the marker

Status
: done

Note
: ran 03:52 slot b: boot-t1.img back as Android after 159 s; battery 4384 mV 100% before; /sys/fs/pstore empty, no ramoops dir; marker absent = cause B

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: tests/host/pstore-t1-read.sh

Rank
: 1


<a id="walleye-vendor-facts-dump"></a>
### vf1: the vendor kernel with our init dumps the running hardware's facts into a bigger log window (device tree as merged, clocks, regulators, pinctrl/gpio, interrupts, iomem, dwc3/phy state, dmesg)

Status
: done

Note
: boot-vf1.img built, 23 checks ok, rebuild identical, selftest 9 ok, reviewed; try queued as walleye-try-vf1

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: boot-vf1.img = the control's kernel/dtb/cmdline/header/footer (bytes equal, as m1) with a ramdisk whose init mounts proc/sys/debugfs, creates a tar of the chosen files (proc/device-tree as a dtc-readable copy via cat of the files or a single fdt if /sys/firmware/fdt exists, debugfs clk/clk_summary, regulator/regulator_summary, pinctrl/*, gpio, /proc/interrupts, iomem, ioports, /sys/class/udc, dwc3 debugfs, /sys/kernel/debug/usb, dmesg), gzips it, writes it as a length-prefixed blob into an 8 MiB file exported as a read-only mass_storage LUN (m1's gadget order: dir, mass_storage, bind), rewrites it in place every 10 s while it grows, holds 90 s then reboot -f; tests/host/vendor-facts-read.sh reads the LUN (uaccess rule exists), checks the length prefix and gunzips the tar into research/google-walleye-lab/2026-11-18-vf/ (not in git if large: put it under the lab's material path and keep the index); checks as m1's; no write to the phone


<a id="walleye-try-vf1"></a>
### Phone: boot vf1 twice from the first state; the keeper reads the window with tests/host/vendor-facts-read.sh (battery first, phone left in Android afterwards); first run may be partial: say what was missing

Status
: done

Note
: vf1 tried twice (lab PR 125): enumerates, LUN readable, no header: superseded by vf2.

Waits
: coordinator: how to make the window show life first

Blocked by
: walleye-vendor-facts-dump

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-vf2-marker-first"></a>
### vf2: the vf1 dump with a marker written at once and a status line per step, so an empty header can be told from a dead init

Status
: done

Note
: boot-vf2.img built, 29 checks ok, rebuild identical, selftest 25 ok, reviewed; try queued as walleye-try-vf2

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: boot-vf2.img = vf1 with only the init changed: (1) write a plain-text header block WALLEYE-VF2 + a beat counter into the window within the first seconds, in m1's proven way (the same file/LUN setup as m1, which the lab user reads); (2) every dump step (mount debugfs, each file/dir copy, tar, gzip, length prefix) appends a status line 'step N name rc=X' into a text region of the window that never moves, before the blob region; (3) the blob is written after the text region at a fixed offset with its own magic and length; busybox applets used are checked present in the control's initramfs build (tar, gzip, cat, dd, mount), and failures are written, not hidden (no set -e swallowing, no >/dev/null on the steps); tests/host/vendor-facts-read.sh prints the status lines first, then extracts the blob if complete; selftest as vf1's; hashes; a programmer review


<a id="walleye-try-vf2"></a>
### Phone: boot vf2 twice from the first state; read the window with tests/host/vendor-facts-read.sh (step lines print first; battery first; phone left in Android afterwards); say whether the marker, the beat and which step the init reached

Status
: done

Note
: proven by try 2: marker, beat and init steps seen; blob unread - see walleye-vf2-tried-twice

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-vf3-gadget-first-progressive"></a>
### vf3: the gadget FIRST, then the dump written progressively (each file appended as read, plain text, no gzip/tar step), and a host poller that keeps everything the window ever showed

Status
: done

Note
: boot-vf3.img built, 24 checks ok, rebuild identical, selftests ok, reviewed; try queued as walleye-try-vf3

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: (1) boot-vf3.img = vf2 with only the ramdisk's init reordered: marker + gadget bound within the first seconds (as m1: ~18 s on the host), THEN the dump, one small source at a time (device tree files from /proc/device-tree as text/hex, clk_summary, regulator_summary, pinctrl/gpio, interrupts, iomem, dwc3/usb debugfs, dmesg head/tail), each appended at the next free offset of the 4 MiB window, framed with 'BEGIN name' / 'END name rc=N' lines, flushed (sync) after each; the init stays up until its hold (90 s) and the tail of the window carries a 'DONE' line; a read of a debugfs file that blocks is bounded by timeout 5; (2) tests/host/vendor-facts-poll.sh: started BEFORE the boot in its own background job, finds the LUN by our strings, copies the whole used part of the window every 2 s to numbered files, keeps the last complete copy, and fetches every new BEGIN..END block into research/.../vf/NAME the moment it appears; stops at 'DONE' or after 150 s; (3) selftest, programmer review, hashes


<a id="walleye-try-vf3"></a>
### Phone: start tests/host/vendor-facts-poll.sh FIRST in its own background job (never killed by the keeper's cleanup), then boot vf3 twice from the first state; battery first; phone left in Android afterwards; say how many blocks the poller fetched, whether DONE came, and the beat log

Status
: done

Note
: tried twice as asked: no gadget, no window, no blocks (poller fail x2); back as Android 78 s each; see walleye-vf3-tried-twice. Next: coordinator decides - vf2's order alive, vf3's reorder dead; diff the two inits

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-vf4-vf2-order-progressive"></a>
### vf4: keep vf2's proven init order up to the gadget bind byte for byte, then append vf3's progressive dump after the bind

Status
: done

Note
: built: boot-vf4.img, 27 checks ok, rebuild identical, selftests ok; research walleye-vf4-built; try queued as walleye-try-vf4

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: boot-vf4.img = vf2's init with ONLY the dump part added after the gadget bind: everything vf2 did before and including the UDC bind unchanged (diff of the init against vf2's shows additions only after the bind step); the added part is vf3's progressive framed dump (BEGIN/END lines, appended, synced, timeout 5 per read); the diff vf2->vf3 is written up in the research row with the likely cause of vf3's death (what it did before the bind that vf2 did not: say it), the diff vf2->vf4 shown; the poller of vf3 reused; checks, selftest, programmer review, hashes


<a id="walleye-try-vf4"></a>
### Phone: start tests/host/vendor-facts-poll.sh FIRST in its own background job with VF_MARK=WALLEYE-VF2 VF_HEAD=16 VF_BLOCKS=1040 VF_OUT=$HOME/Projects/walleye-build/material/vf/2026-11-18-vf4 (never killed by the keeper's cleanup), then boot boot-vf4.img twice from the first state; battery first; phone left in Android afterwards; say how many blocks the poller fetched, whether DONE came, the beat log, steps.txt's last lines (where it stopped), and when the phone left USB

Status
: done

Note
: proven by two tries: 0 blocks fetched, no gadget, back as Android at 78 s both; turning point for the coordinator: vf4 behaves like vf3 (dead), not vf2 - what differs between vf2 and vf4 before the bind

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-try-vf-control"></a>
### Controlled repeat: vf2 twice as before, then vf4 twice WITHOUT the poller, to tell the image from the way it was watched

Status
: done

Note
: proven by 4 tries: vf2 repeats (gadget ~50 s, empty header), vf4 dead twice unwatched; poller cleared; next: diff vf4 vs vf2 after the bind, bisect the dump

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: four tries from the first state (battery first, Android on a good slot, adb reboot bootloader), each recorded with the time the gadget appeared (plain lsusb by port, no lsusb -v, no reads of the LUN until 70 s after the jump), the time Android is back, and whether the init's beat/step text appears in a single read at ~70 s of /dev/sdX (massstorage-read.sh as in vf2's tries): order vf2, vf2, vf4, vf4; the finding: does vf2 repeat its ~43 s gadget? does vf4 (identical init to vf2 up to the bind) behave like vf2 when watched the same way? if vf4 is alive unwatched and dead with the poller, the poller (lsusb -v or the LUN reads during enumeration) is what breaks the phone

Rank
: 1


<a id="walleye-vfa-one-source-per-image"></a>
### One hardware source per image on m1's proven structure: vfa = /sys/firmware/fdt (the merged running device tree) + iomem + interrupts + cmdline; vfb = clk_summary; vfc = regulator_summary; vfd = the USB/dwc3 debugfs state

Status
: done

Note
: Built and checked: images in ~/Projects/walleye-build/images-vfx, research walleye-vfx-built; try queued as walleye-try-vfa.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: four images, each = m1 (its init, window file and gadget order byte for byte) plus ONE payload copied into the window file BEFORE the bind (cat to the window file with timeout 5; the payload sizes checked to fit the 2 MiB window, else split across images; the first 512 bytes of the window carry a plain 'WALLEYE-VFx' header, the payload's name and its byte length, then the payload as plain bytes, so the host reads it with massstorage-read.sh and one dd of the LUN; no tar, no gzip, no progressive appends, no poller); a host reader script extracts the payload by the header into the lab material path (the fdt kept as .dtb, with dtc -I dtb -O dts for the readable form); selftest; programmer review; hashes


<a id="walleye-try-vfa"></a>
### Try vfa twice, then vfb, vfc, vfd once each (twice if one dies): which single hardware source reads, which kills

Status
: done

Note
: proven by: vfa twice pass (fdt.dtb/dts kept in material path), vfb alive but clk_summary unreadable rc=1, vfc pass, vfd killed twice (research walleye-vfx-tried). Next: give mainline USB its nodes from the read fdt/iomem/interrupts; usb/dwc3 debugfs read kills the init, so it must not be read in the window.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: vfa twice, then vfb, vfc, vfd once each unless one dies (then twice), each recorded with the time the gadget appeared, the time Android is back, and the reader's one line (pass with the parts, or fail saying killed or hung); the finding: which source reads and which kills; /sys/firmware/fdt kept as fdt.dtb and fdt.dts in the lab material path if vfa passes

Rank
: 1


<a id="walleye-compare-with-the-running-fdt"></a>
### Compare OUR dtb, the alive taimen dtb (pk5) and the phone's RUNNING fdt (vfa): what the bootloader adds or needs, and what our tree lacks; then build the images that test it

Status
: done

Note
: Compared in walleye-fdt-compare; images x1..x7, xa built (walleye-fdt-bisect-x-built); queued walleye-try-fdt-bisect for the keeper; USB nodes: none to add, see the compare row

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: (1) a research row from the three decompiled trees (material path ~/Projects/walleye-build/material/vf/2026-11-18-vfx-vfa-try1/fdt.dts; wpa/pk5's dtb; our out dtb): what the ABL ADDS to the tree it hands the kernel (chosen, memory, reserved-memory entries, board-id/msm-id/pmic-id, cpu/psci/timer fixups, the overlay's changes): present in the running fdt, absent in the source dtbs; what OUR tree lacks that the alive taimen tree has and the running fdt has too (list the nodes and properties that are in BOTH alive trees but not ours: the likely required ones), in order of suspicion; the structural facts (dtb size, number of nodes, phandle count, root compatible, memory node form: reg ranges and cell sizes, whether our /memory exists); (2) images x1..xn = wa/pk5's alive dtb progressively REPLACED toward ours group by group (the other direction from the earlier bisects: start alive, move top-level groups to ours: root props, /memory, /reserved-memory, /cpus, /psci, /timer, /soc nodes in halves), same kernel/init as pk5 (immediate reboot), checks, hashes, expected returns (alive <= 66 s, dead >= 69 s); (3) the nodes our dts needs for USB peripheral on this phone read from the running fdt (usb@a800000, the phys, regulators, clocks, gpios, extcon/vbus detect) as a proposal for dt/msm8998-google-walleye.dts


<a id="walleye-battery-ok-timeout"></a>
### tests/host/battery-ok.sh must not hang when the phone runs Android: bound every fastboot call by a timeout and fall back to adb

Status
: done

Note
: built: adb first then fastboot, each call under timeout -k 1 6, skip exit 2 within 15 s; selftest with hanging fakes passes in 12 s; programmer review done

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: the script reads the battery from adb (dumpsys battery) when adb shows the phone, from fastboot getvar battery-voltage when fastboot does (each call under timeout 10), prints "not measured" and exits 2 when neither answers within 15 s; run with the phone in Android it ends in under 15 s; selftest with fake adb/fastboot in PATH; programmer review


<a id="walleye-use-lpoco-bisect"></a>
### Retire walleye's own bisect scripts (build-bisect.sh, check-bisect.sh and their kin) for `lpoco-bringup image bisect SERIES`: write walleye's series files

Status
: open

Note
: From orchestration (lab-tools PR 123, release v0.4.0): the one command `lpoco-bringup image bisect SERIES --out DIR` counts the swaps against the control and wraps lz4 kernels. Low rank; any builder run after the lab host is on 0.4.0. Keep the old scripts until the reproduction passes.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: walleye's series file(s) declare the swaps of the bisects done so far (pk4/pk5/pk6, d1-d6, w*/wa/wpa, wp*, the fdt compare when it exists) against the control image; `lpoco-bringup image bisect` reproduces at least pk5, wa and one d-image byte for byte (sha256 equal to the recorded ones, or the difference explained); the old scripts removed in the same change; checks and hashes; programmer review


<a id="walleye-try-fdt-bisect"></a>
### Phone: boot x4, x1, x2, x6, x5, x3, x7, xa twice each from the first state; record the sixteen return times

Status
: done

Note
: proven by 16 boots: x1 dead twice (79,78 s), xa dead (78,76), x2-x7 alive. Next: add gpio-reserved-ranges <0 4>,<81 4> to our dtb and try (alive expected)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-pstore-t4-pon-warm-notifier"></a>
### Kernel patch T4: a reboot notifier setting the PON PS_HOLD reset type to warm (read registers from vendor qpnp-power-on first)

Status
: open

Note
: Only if T3 fails or a plain reboot must keep DDR; candidate to send upstream

Waits
: paused by decision 2026-11-18-walleye-vendor-kernel-as-instrument: the pstore chase waits for a witness (UART) or the fdt bisect's result

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: research

Proof
: pstore holds the marker after a plain reboot -f


<a id="walleye-our-dts-gpio-reserved"></a>
### The cause of our dead dtbs: pinctrl@3400000 lacks gpio-reserved-ranges <0 4>, <81 4> (TZ-protected GPIOs). Put it into OUR dts and build images y1 (init: reboot at once) and y2 (the gadget: ACM console + mass_storage window)

Status
: done

Note
: Built: y1 and y2 by build/build-y.sh (49 checks ok, rebuild identical, hashes in research walleye-y-built); try queued as walleye-try-y (phone, rank 1).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: (1) dt/msm8998-google-walleye.dts gets the property exactly as in the alive trees and the running fdt (pinctrl@3400000 gpio-reserved-ranges = <0 4>, <81 4>), with a comment naming where it comes from (the phone running fdt, vfa) and why (those pins belong to the secure world: touching them resets the phone) - and the other missing items from walleye-fdt-compare that are cheap and certain (restart@10ac000 qcom,pshold, the apps watchdog) only if the finding says they matter, one commit each so the effect is attributable; (2) images y1 = our kernel (the ml6a one, no earlycon) + OUR dtb with the property + the ml5a immediate-reboot init; y2 = the same + the mainline gadget init of pk5l (ACM shell on ttyGS0 + mass_storage window, watchdog petted 180 s, reboot -f at the end) so the first USB light can appear; expected: y1 alive (<=66 s), y2 shows 1d6b:0104 on the host within ~25 s of the jump (plain lsusb); checks, hashes, programmer review; build recipe reproducible; (3) keep dtbo-overlay-merges.sh passing


<a id="walleye-try-y"></a>
### Phone: boot y1 twice, then y2 twice from the first state; battery first; read return times and the host's lsusb; leave Android on a good slot

Status
: done

Note
: proven by y1 alive x2 (36 s) and y2 gadget 1d6b:0104 at 18 s x2, research walleye-y-tried

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: TEST

Rank
: 1


<a id="walleye-first-mainline-shell"></a>
### The first shell of our own mainline kernel: read y2 over /dev/ttyACM0 - the whole dmesg first, then the facts

Status
: done

Note
: proven by reader pass: 13 answers, dmesg 336 lines; fixed reader's end marker matching the phone's echo

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: A script tests/host/mainline-shell-read.sh (new, in the lab, selftested with a fake tty) opens /dev/ttyACM0 as the lab user the moment it appears, sends commands one at a time with a read timeout, and saves under the lab material path (never into git without redaction): (1) dmesg in full - the first mainline kernel log of this phone; (2) uname -a, /proc/cmdline, /proc/cpuinfo, /proc/iomem, /proc/interrupts, /proc/version; (3) the device list: ls /sys/bus/platform/devices, deferred probes (/sys/kernel/debug/devices_deferred if debugfs is mounted), /sys/class/udc, /sys/class/power_supply, /sys/class/drm, /sys/class/input, /sys/class/regulator; (4) touch /tmp/hold to keep the phone up while reading, then reboot -f. Nothing is written to the phone beyond /tmp. The research row names what the dmesg shows works, what defers or fails (display, touch, UFS, charger, regulators): the real map of the next parts to bring up.

Rank
: 1


<a id="walleye-pmic-spmi"></a>
### PMIC first: bind pm8998 and pmi8998 on the SPMI bus (the first mainline dmesg: 'does not have reg property')

Status
: done

Note
: z1 built (hashes in research walleye-z1-built); try queued as walleye-try-z; the phone result is not known yet

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: walleye-pmic-bound: dmesg has no 'does not have reg property'; /sys/bus/spmi/devices lists the PMIC USIDs; rtc0 and the pon reboot reason are readable

Rank
: 1


<a id="walleye-bringup-order-plan"></a>
### Plan the real bring-up order from the first mainline map and queue the items

Status
: done

Note
: Done 2026-11-18: research walleye-bringup-order-plan-made, decision 2026-11-18-walleye-bringup-order; items queued: pmic-spmi(1), dmesg-clean(2), charger-smb2(3), ufs-readonly(4), usb-net, fuel-gauge, touch-rmi4, display-s6e3fa5, wifi-wcn3990, audio-haptics, radios-research

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: research

Proof
: A research row that orders the bring-up from the first mainline map and the vendor facts (the running fdt, the dmesg of y2, the vendor kernel's regulator_summary): the dependencies between the parts (PMIC and its regulators -> UFS (rootfs) and storage; PMIC/charger -> battery and safe charging while testing; arm-smmu -> UFS DMA, display, USB; clocks (gcc/rpmcc) -> everything; USB networking (a function the host can bind with the lab-prepare rules: ncm with its attributes written the mainline way, ecm or rndis, as the mainline kernel supports them natively) for ssh instead of a serial console; display (S6E3FA5 command-mode through DSI/DPU) and touch (Synaptics RMI4 i2c); audio, modem and Wi-Fi/BT later), the order with what each unlocks and its proof test (the kernel dmesg, a sysfs file, the shell), and for each part the backlog item (build and phone) queued by the research session itself in this base with --needs, proof and note; the first three items after the PMIC get ranks; the research names the parts mainline has for the Pixel 2's chips (panel driver for the S6E3FA5?, touchscreen RMI4, charger smb2 (pmi8998), fuel gauge, haptics DRV2624, audio WCD934x/TAS2557) and which would need new drivers (upstream candidates)


<a id="walleye-charger-smb2"></a>
### Charger and battery: enable pmi8998's smb2 charger (qcom_smbx) with a monitored battery of 2700 mAh / 4.4 V

Status
: done

Note
: c1 built (research walleye-c1-built), try queued as walleye-try-c1; proof walleye-charger-online comes from that try

Blocked by
: walleye-pmic-spmi

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: walleye-charger-online: a power_supply of type USB appears, its online flag follows the cable, and the battery voltage rises while connected

Rank
: 3


<a id="walleye-usb-net"></a>
### USB networking: ncm/ecm gadget with ssh on 172.16.43.1 instead of the ACM console

Status
: open

Note
: The ACM shell already works (y2). Host side: the lab-prepare rules; function ncm with attributes written the mainline way, ecm or rndis as fallback. Does not need the PMIC.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: walleye-gadget-up: ssh to 172.16.43.1 answers within 60 s of the boot


<a id="walleye-wifi-wcn3990"></a>
### Wi-Fi and Bluetooth: WCN3990 through ath10k snoc, firmware read from the phone's own vendor partition

Status
: open

Note
: ath10k is in mainline. The firmware is the phone's own, read from its partition at boot, never put in a repository.

Blocked by
: walleye-ufs-readonly,walleye-dmesg-clean

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: walleye-wlan-scan: wlan0 scans and finds a network; hci0 is up


<a id="walleye-audio-haptics"></a>
### Haptics (DRV2624) and speaker amps (TAS2557) with the WCD934x codec: drivers to write or port

Status
: open

Note
: Mainline has drv260x/drv2665/drv2667 (not DRV2624) and tas2552/2562/2764/2770/2780 (not TAS2557); porthole's patches 0018 and the audio ones are GPL starting points. The codec and the DSP firmware are the long part.

Blocked by
: walleye-ufs-readonly

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: research

Proof
: research row naming the driver path of each; then walleye-haptics-felt and walleye-tone-heard (a person)


<a id="walleye-radios-research"></a>
### Research items for modem, sensors (SLPI), camera, NFC, fingerprint and deep sleep: one question each

Status
: open

Note
: Components of docs with no mainline path yet. The lane's later lead takes them one by one after UFS read-only.

Blocked by
: walleye-ufs-readonly

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: research

Proof
: a research row per component with the chosen way and its failing test, as research-a-feature says


<a id="walleye-try-z"></a>
### Try z1 once from slot b with the shell reader waiting; read the PMIC result

Status
: done

Note
: Superseded by walleye-try-z3 (z3 contains the PMIC dtb of z1 and the gpucc fix of z2).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: sh tests/host/pmic-dmesg-check.sh DIR passes on the reader's output of z1 (images-z/boot-z1.img sha256 4abbac7d...fa2a): no reg-less spmi node of ours, /sys/bus/spmi/devices has 0-00 0-01 0-02 0-03, no probe of them failed

Rank
: 1


<a id="walleye-try-z2"></a>
### Try z2 once from slot b with the shell reader: is the dmesg clean of the smmu -110?

Status
: done

Note
: Superseded by walleye-try-z3 (z3 contains the PMIC dtb of z1 and the gpucc fix of z2).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: walleye-dmesg-clean: sh tests/host/dmesg-clean-check.sh DIR passes (smmu part: no -110 line; the serial and phandle parts are still expected to fail)

Rank
: 2


<a id="walleye-try-y2-control"></a>
### Control: y2 once with the reader, then z1 once: is the silent ttyACM0 of z1 the new dtb or the reader

Status
: done

Note
: proven by two tries: y2 and z1 both silent to the reader; the fault is the reader or the init's tty, not z1's dtb. Next: z1 variant with a banner every 2 s on ttyGS0 and a kmsg marker per step (builder), and a raw cat of the tty before any command

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: Two tries in this order, from slot b, battery first, the same reader (tests/host/mainline-shell-read.sh) started the same way before the boot: (1) y2 (the image that gave the first dmesg, lab PR 155) once: does the shell answer? (2) z1 once. For each: the time the gadget and /dev/ttyACM0 appear, whether the reader's first command gets an answer and after how long, the raw bytes the tty returns in the first 10 s (even if the reader finds no end marker), the host's dmesg lines for the gadget (cdc_acm bound? ttyACM0 created? disconnects?) and the lsusb -v of the device; and when the phone is back. The finding: is z1's silence in the new kernel/dtb (PMIC nodes: a hang or panic in userspace start, a changed probe order) or in the reader/timing (then y2 is silent too today)? If y2 answers and z1 does not, the next step is a z1 variant: the same init with the shell loop writing a banner to /dev/ttyGS0 every 2 s from the start (so the tty shows life even before it reads), and a kmsg marker per init step; to be built by a builder (queue it).

Rank
: 1


<a id="walleye-try-y2-old-reader"></a>
### y2 with PR 155's exact reader: is the silence the reader change or the host

Status
: done

Note
: proven by research walleye-y2-old-reader-tried: old reader also silent, only the tty echo arrived; reader change not the cause, host not deaf; next: walleye-build-banner-init

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: Run y2 (the image whose shell read worked in lab PR 155) from slot b, battery first, with the reader EXACTLY as it was at lab commit 733052d (git show 733052d:tests/host/mainline-shell-read.sh > a scratch copy, run that copy, not the current one), started before the boot as in the first read; record: bytes received and the first answer. Then, if it reads: the same y2 with the current reader (main) on a second boot, to confirm the reader change (commit 17ab08e, PMIC change touched the script) is the cause; the diff of the two readers goes in the record; if the old reader also gets 0 bytes, the host is the cause: record the tty settings as the reader opens it (stty -F /dev/ttyACM0 -a), fuser /dev/ttyACM0, ls -l, getfacl, systemctl is-active ModemManager, and the host's dmesg lines for the gadget, from the failing moment; do not change the host.

Rank
: 1


<a id="walleye-build-banner-init"></a>
### Build a banner init: y2's kernel and dtb with an init that writes a banner to /dev/ttyGS0 every 2 s from the start and a kmsg marker per step

Status
: done

Note
: Built 2026-11-18: images-banner/boot-banner.img sha256 3d9320f01567a9824f1371a218fa397cbedc4eb996e9409b613a458878a4a081, 40 checks, rebuild identical, reviewed; research walleye-banner-built; try queued as walleye-try-banner.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: banner image built with build checks, rebuild identical

Rank
: 1


<a id="walleye-try-c1"></a>
### Try c1 once from slot b with the shell reader: does pmi8998's charger probe, and is online 1 with the cable in?

Status
: open

Note
: Rebuild c1 on the z6 init (z6's dtb is z3's: no charger node); keeper recipe: battery-ok first, z6 plain background reader, saves power-supply-facts.txt and dmesg automatically; then charger-check.sh. Risks as before: float voltage 4.4 V, Type-C reset may drop the ACM link (a missing tty is a finding; z3/z6 is the fallback).

Blocked by
: walleye-charger-on-z7

About
: walleye-charger-smb2

Opened
: 2026-11-18

Needs
: phone

Proof
: walleye-charger-smb2: sh tests/host/charger-check.sh DIR passes on the reader's output of c1 (images-c1/boot-c1.img sha256 ddae19b523f2a68d902755f0e57d53c3b98c74b9b146d372bc40ea307aaee0c0)

Rank
: 3


<a id="walleye-try-banner"></a>
### Try banner once from slot b with a plain cat of ttyACM0: are the banner bytes on the host?

Status
: done

Note
: proven by banner-check.sh pass on a root read of ttyACM0 (7 lines, beats increasing); plain-user read was Permission denied: host needs uucp/udev for mll (author's call) - coordinator told

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: walleye-banner-seen: sh tests/host/banner-check.sh FILE passes on 'cat /dev/ttyACM0 > FILE' of banner (images-banner/boot-banner.img sha256 3d9320f01567a9824f1371a218fa397cbedc4eb996e9409b613a458878a4a081)

Rank
: 1


<a id="walleye-host-tty-rule"></a>
### The gadget's tty rule does not apply: ttyACM0 stays root:uucp 0660 for the lab user

Status
: done

Note
: Closed: not a host fault. The banner image's serial was walleye-banner, not walleye-first-boot, so the rule rightly did not apply (orchestration, host kernel log 10:06:55). Fix: walleye-banner-serial-fix.

Waits
: orchestration (lpoco host prepare): the rule does not reach the ttyACM node

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Proof
: Find why /etc/udev/rules.d/70-lpoco-google-walleye.rules (GROUP users, MODE 0660, TAG uaccess; SUBSYSTEM tty, ATTRS serial walleye-first-boot) leaves /dev/ttyACM0 as root:uucp 0660 for the lab user: with the gadget up (a keeper holds the phone: ask the coordinator for a window or look at the next keeper try's record) run udevadm info -a -n /dev/ttyACM0 (does ATTRS serial walleye-first-boot match on a parent?), udevadm test of the node (rule order against 60-serial.rules and 50-udev-default.rules, which rule sets GROUP last), ls -l and getfacl at the moment of the try. Fix in the lpoco host prepare manifest (device.toml [host]) not by hand. Then the root-exec use of decision 2026-11-18-walleye-tty-read-through-root-exec ends.


<a id="walleye-reader-eacces-retry"></a>
### The host readers retry the device node while it says Permission denied (the uaccess ACL arrives a moment after the node)

Status
: done

Note
: Built; selftest 21 ok; walleye-try-z already queued (phone, rank 1)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: tests/host/mainline-shell-read.sh (and the other readers that open a device node the same way) retry opening the node while it answers 'Permission denied', for up to 20 s in 0.5 s steps, set raw mode themselves (stty -F NODE raw -echo -hupcl), and give up with a fail line that says what they saw (the node's owner, group and ACL, the serial of the usb parent, who holds the port, whether the udev rule file is present); selftest with a fake denied for the first 3 s; NO root-exec, no chmod, no sudo (decision 2026-11-18-walleye-banner-serial-not-root-exec); programmer review

Rank
: 2


<a id="walleye-banner-serial-fix"></a>
### init-banner writes the wrong USB serial (walleye-banner): the host rule did not apply; fix it and check every init

Status
: done

Note
: Built 2026-11-18: images-banner2/boot-banner2.img sha256 680c0aa730d294e3d9c23a23c3abc4db993cfaff059e805229e4af6244ce9888, 41 checks, rebuild identical, reviewed; research walleye-banner2-built; check-init-serials.sh over every init; try queued as walleye-try-banner2.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: build/initramfs-mainline/init-banner writes the USB serial walleye-first-boot (the product string may stay 'Pixel 2 walleye (mainline banner)'); build-banner*.sh gets the same serial check build-mainline-light.sh has for init-ml (it fails when an init writes another serial); a rebuilt boot-banner2.img (everything else equal to boot-banner.img), 40+ checks, programmer review, hashes; every other init of ours is checked for the serial too (grep over build/initramfs-*/init*: one answer, walleye-first-boot, anything else fails the checks)

Rank
: 1


<a id="walleye-reader-drop-root-exec"></a>
### Drop the root-exec fallback from the shell reader: the no-root decision stands (2026-11-18-walleye-banner-serial-not-root-exec)

Status
: done

Note
: Done: root-exec dropped from the reader; selftest 19 ok; still-denied fails with owner, group, ACL, usb serial, holders.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: tests/host/mainline-shell-read.sh no longer calls lab-agent root-exec anywhere (grep finds no root-exec in tests/host and build): the retry on Permission denied stays (20 s), the fail line says what it saw (owner, group, ACL, the usb parent's serial, who holds the port); the selftest's root-exec cases are replaced by a case where the tty stays denied and the script fails cleanly with that detail; the OUT/root-exec.log mention is removed; the other readers stay as they are; programmer review

Rank
: 1


<a id="walleye-try-banner2"></a>
### Try banner2 once from slot b with a plain cat of ttyACM0 as the lab user: do the banner bytes reach it?

Status
: done

Note
: proven by banner-check pass on banner2.cap: 29 lines, beats increasing

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: walleye-banner-seen: sh tests/host/banner-check.sh FILE passes on 'cat /dev/ttyACM0 > FILE' as the lab user (no root) of banner2 (images-banner2/boot-banner2.img sha256 680c0aa730d294e3d9c23a23c3abc4db993cfaff059e805229e4af6244ce9888)

Rank
: 1


<a id="walleye-try-z3"></a>
### Try z3 once from slot b with the shell reader: is the dmesg fully clean?

Status
: done

Note
: Superseded by walleye-try-z4 (z4 = z3 + the init change that makes the shell answer).

Blocked by
: walleye-shell-reliable

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Proof
: walleye-dmesg-clean: sh tests/host/dmesg-clean-check.sh DIR passes on the reader's output of z3 (images-z3/boot-z3.img sha256 9bc4ec19939b411531ed3dbbb8a931018ddebdcc66b903bd2757ac05d5031f98)

Rank
: 1


<a id="walleye-shell-reliable"></a>
### Make the ACM shell answer reliably: the phone's tty must not hang up between commands, and the reader must keep one fd open (z4)

Status
: done

Note
: Built 2026-11-18: build/build-z4.sh (z3 + init-z4 only, rebuild identical, boot-z4.img sha256 4115db3e2c678919cb3d3bd725d5065bbbbac1e6026732989c1eca3590043b20), reader keeps one fd, selftest 26 ok with a hang-up-on-close fake that fails the old reader; programmer review fixed (no cttyhack, retry failed open, HUP). Try queued as walleye-try-z4.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: Two changes that together make the ACM shell reliable, each its own commit: (1) the init (build/initramfs-mainline/init-pk5l and the one the z images use): before the shell starts, `stty -F /dev/ttyGS0 raw -echo -hupcl` (so that a host closing and reopening the port does not hang up the shell's terminal), the shell loop respawns at once (no sleep), and the shell is started with `setsid` and `cttyhack`-style controlling terminal if busybox has it (say what applets exist), so `ash -i` has a tty; a kmsg line per (re)start of the shell with a counter; (2) tests/host/mainline-shell-read.sh holds ONE file descriptor open on the tty for the whole session (exec 3<>"$TTY"), writes each command to fd 3 and reads answers from fd 3 with a per-command timeout, never reopening the node per command (line 50's per-command redirection closes the port after each command, which hangs the phone's shell up); the selftest's fake phone is changed to reproduce the hang-up on close (a fake that exits its 'shell' when the node is closed) so the old reader fails the selftest and the new one passes; the image z4 = z3 with only the ramdisk's init changed, 35+ checks, programmer review, hashes

Rank
: 1


<a id="walleye-stale-waits-on-done"></a>
### Stale waits: waits and blocked_by naming items that are done

Status
: done

Note
: Done 2026-11-18: waits naming done items cleared on dmesg-clean, code-hygiene-rust-ci, display, fuel-gauge; done items closed (bootimage, first-fastboot-boot, load-error-uart, loaderror-dtbo-stub, host-access-via-lab-prepare, initramfs-usb-gadget).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18


<a id="walleye-dmesg-clean"></a>
### Clean the z6 dmesg: dts disables adreno_smmu (gpu_gx GDSC stuck, smmu deferred for ever); reader mounts debugfs

Status
: done

Note
: built: images-z7 (research walleye-z7-built); try queued as walleye-try-z7

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: walleye-dmesg-clean: no 'gpu_gx status stuck' WARN, no 5040000.iommu deferred line, devices_deferred readable and listing only the known overlay nodes

Rank
: 2


<a id="walleye-code-hygiene-rust-ci"></a>
### Remove the C probe of the ffs helper (build/ffs-acm/ffs-acm.c and what only builds n3 from it) once n3r is tried

Status
: open

Note
: CI half split off as walleye-rust-ci-ffs-acm-rs (done). This half waits by the coordinator's order until the keeper has tried n3r (walleye-ffs-rust-try), so the C probe stays buildable until the Rust helper is shown on the phone.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: build/ffs-acm/ffs-acm.c and anything that only builds n3 from it removed (n3's recorded hashes stay as history; n3r is the image to try); the lab's checks pass; read back from main


<a id="walleye-display-s6e3fa5"></a>
### Display: Samsung S6E3FA5 command-mode panel through DPU/DSI

Status
: open

Note
: No mainline driver; start from the out-of-tree S6E3FA5 driver (OnePlus 5) and porthole's sw43402 cmd-mode work for the DPU side; init sequence from the GPL downstream dtsi; reset gpio94, TE gpio10 (walleye) - the panel and its rails are walleye's own.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: the DRM connector is connected and a test pattern is seen by a person (test: walleye-display-pattern)


<a id="walleye-fuel-gauge"></a>
### Fuel gauge: test the PMI8998 FG gen3 driver (RFC series of 2025-11-24) on walleye; an upstream candidate

Status
: open

Note
: Not in mainline: lwn.net/Articles/1048226 (driver by Joel Selvaraj, carried by sdm845-next). Take the series as a patch file, test, report upstream (ikb upstream-item). The walleye battery profile is the vendor tree's, read in our own words.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: walleye-battery-capacity: capacity and voltage_now within 100 mV of the fastboot battery reading; the reading sent upstream as a Tested-by


<a id="walleye-try-z4"></a>
### Try z4 once from slot b with the one-fd shell reader: does the ACM shell answer, and is the dmesg fully clean?

Status
: done

Note
: tried once (research walleye-z4-tried-once); superseded by walleye-try-z5, which reports the shell's state

Blocked by
: walleye-banner-shell

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Proof
: walleye-dmesg-clean: sh tests/host/mainline-shell-read.sh answers all 21 files on z4 (images-z4/boot-z4.img sha256 4115db3e2c678919cb3d3bd725d5065bbbbac1e6026732989c1eca3590043b20) and sh tests/host/dmesg-clean-check.sh DIR passes on its dmesg

Rank
: 1


<a id="walleye-banner-shell"></a>
### z5: the banner (it works) reports the shell's state, so the silent shell explains itself

Status
: done

Note
: z5 built (research walleye-z5-built): boot-z5.img fee768d62219fd08ce1e90618ea5a7977dd3ca1c762b6a4563f9ca92dfcc8dd9; try queued as walleye-try-z5

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: boot-z5.img = the banner2 init (its banner on /dev/ttyGS0 works, read by the lab user with a plain cat, lab PR 182) PLUS the console block of init-z4 started in the background, and the banner line extended with the shell's state: shell started yes/no, restart count, the shell pid and whether it is alive (kill -0), the exit status of the last run, and the first error text of its start (stderr of the shell start captured into a file in /tmp and its first line in the banner); the banner keeps going when the shell dies; also a one-time line with what busybox offers (busybox --list | grep -c, and whether 'ash', 'sh', 'cttyhack', 'setsid', 'stty' are there) and `stty -F /dev/ttyGS0 -a` summarized (echo/icanon/hupcl); a kernel + dtb equal to z3/z4's; checks like banner2's and z4's, the serial check; the host test banner-shell-check.sh: reads the banner with a plain cat for 60 s, prints the last banner line and passes when it shows a live shell; programmer review; hashes

Rank
: 1


<a id="walleye-try-z5"></a>
### Try z5 once from slot b: a plain cat of the banner for 60-90 s says why the shell is silent

Status
: done

Note
: proven by banner-shell-check: pass, live shell, start count steady (research walleye-z5-tried)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: walleye-banner-shell-check: sh tests/host/banner-shell-check.sh runs (cat of /dev/ttyACM0, 60 s; images-z5/boot-z5.img sha256 fee768d62219fd08ce1e90618ea5a7977dd3ca1c762b6a4563f9ca92dfcc8dd9) and its last banner line is recorded as research: it shows what is wrong with the shell. Battery first, phone left in Android.

Rank
: 1


<a id="walleye-try-z5b"></a>
### z5 again with the host tty raw/no-echo the moment it appears: is the shell drowned by the host's echo of the banner?

Status
: done

Note
: tried once 2026-11-18 (second round, plain background reader, research walleye-z5b-tried-raw-tty): raw -echo removes the '> ' and 'Permission denied' noise (0 of each in a 30 s capture) but the shell still returns no command output, only banner beats; next item: z6 = z5 with the banner stopped (or sent to /dev/kmsg only) once the shell is up, so the shell's answers are alone on ttyGS0

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: z5 once from slot b with the HOST tty put in raw, no-echo mode the moment the node appears, before anything is read: `stty -F /dev/ttyACM0 raw -echo -hupcl` in a tight loop started before the boot (poll for the node every 0.1 s, run stty at once, once per appearance; note how many bytes had already arrived), THEN the banner capture for 30 s (cat), THEN tests/host/mainline-shell-read.sh (one fd, raw) once, after the banner shows 'shell alive'. Record: (a) did the '> ' prompts and 'ash: /: Permission denied' lines disappear with the host tty raw? (b) do the reader's commands now get answers (dmesg, uname)? (c) if not, the banner line's shell fields at the time. Expected if the hypothesis holds: the host tty has ECHO on by default, so every banner line the phone writes is echoed back into the phone's tty, the shell reads the banner text as commands ('> ' = an unfinished quote, 'ash: /: Permission denied' = a line parsed as running /), which also swallows the real commands. Then z4 and y2 were never silent for lack of a shell, only drowned: the reader must set the host tty raw/-echo BEFORE the first byte can arrive (stty in a loop on appearance), and the init should ignore input lines for a few seconds after start (or the shell start should wait for the first line from the host). Record the decision text for the reader in the record. Battery first; phone left in Android.

Rank
: 1


<a id="walleye-command-loop"></a>
### z6: replace the silent interactive ash by a plain read/exec command loop on ttyGS0 (the banner stays)

Status
: done

Note
: built: images-z6 (research walleye-z6-built); try queued as walleye-try-z6

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: boot-z6.img = z5 (banner2 + shell state in the banner) with the console block REPLACED by a command loop that needs no interactive ash, no job control and no controlling terminal: after stty -F /dev/ttyGS0 raw -echo -hupcl, `while :; do IFS= read -r line < /dev/ttyGS0 || { sleep 0.2; continue; }; [ -n "$line" ] || continue; sh -c "$line" > /tmp/out 2>&1; cat /tmp/out > /dev/ttyGS0; printf '\n__END_%s__\n' "$n" > /dev/ttyGS0; done` (the end marker carries the command counter, which the reader matches) - read the existing reader's marker convention in tests/host/mainline-shell-read.sh and use exactly that marker, so the reader works unchanged; the banner keeps running (its lines are tagged WALLEYE-BANNER so the reader can ignore them) and also reports the loop's state (commands run, last command, last rc); banner lines never carry the end marker; check script, selftest with a fake phone running the same loop in a pty (the loop is tested on the host with a pty pair so the claim 'a line in gets an answer out' is measured); programmer review; hashes

Rank
: 1


<a id="walleye-kernel-pmic-wdt"></a>
### walleye's v7.3 kernel with the PMIC watchdog series (linux-phone-oco/linux topic/pmic-wdt-wake-in-suspend) and &pm8998_wdt enabled, wakeup-source

Status
: open

Note
: docs decision 2026-11-18-upstream-forks. The branch is on v7.3 (local worktree ~/Projects/linux-phone-oco/linux-topics until the upstreams lane pushes it): merge it into walleye's test branch, CONFIG_PM8916_WATCHDOG=y, board: &pm8998_wdt { status = "okay"; wakeup-source; }.

About
: deep-sleep

Opened
: 2026-11-18

Needs
: build

Proof
: pmic-wdt-bites


<a id="walleye-pmic-wdt-proof"></a>
### Prove the PMIC watchdog way back on walleye (pm8998): it bites unpetted, and its bark wakes a suspend

Status
: open

Note
: Script https://github.com/linux-phone-oco/docs/blob/main/research/upstream-forks/2026-11-18/tests/pmic-wdt-way-back.sh: check, bite 20 then check (CARDRESET); wake only once walleye suspends at all. The second PMIC (pm8998) the RFC needs before it drops RFC (docs upstream item pmic-wdt-wake-in-suspend).

Blocked by
: walleye-kernel-pmic-wdt

About
: deep-sleep

Opened
: 2026-11-18

Needs
: phone

Proof
: pmic-wdt-bites,pmic-wdt-wakes-in-suspend


<a id="walleye-try-z6"></a>
### Try z6 once from slot b: do commands answer on the plain read/exec loop?

Status
: done

Note
: proven: commands answered on the loop, dmesg 590 lines with Linux version (research walleye-z6-tried)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: z6 once from slot b (images-z6/boot-z6.img sha256 7f8af938a4a5c07197f81d306e4f7122f358a6fd657db6b53898bd8bbe94a97a): the host stty loop (stty -F /dev/ttyACM0 raw -echo -hupcl at each appearance) + banner capture 20 s + tests/host/banner-loop-check.sh on it + the reader tests/host/mainline-shell-read.sh (one fd) as plain background processes (decision 2026-11-19-walleye-reader-plain-background-process); if commands answer: dmesg in full, the facts, the PMIC check; battery first; phone left in Android. Record whether the banner says loop=[started=yes ...] and whether the reader got dmesg ('Linux version').

Rank
: 1


<a id="walleye-z6-dmesg-map"></a>
### Read the first full dmesg of our own kernel (z6: 590 lines): every warning, deferral and failure with its cause, and queue the fixes in order

Status
: done

Note
: Mapped in research walleye-z6-dmesg-mapped; decision 2026-11-18-walleye-z6-dmesg-fix-order; items queued: dmesg-clean (2), try-c1 (3), ufs-readonly (4)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: research

Proof
: A research row that reads the full dmesg of the z6 try (the keeper's saved material path in the lab-material directory; the research row walleye-z6-tried names it): every line that is a warning, error, deferred probe, 'failed', timeout or 'not found', each with its cause as far as the kernel source and our dts show (a missing node, a missing driver in our config, a supplier, a clock, a regulator, a firmware), grouped by part (PMIC/regulators, clocks, smmu, UFS, display, touch, charger/fuel gauge, USB, serial, remoteprocs, sensors); what already works as proved by lines (SPMI 0-00..0-03, USB udc, watchdog, ramoops, RPM regulators); and for every group the backlog item (build, with --needs, rank and proof) queued by the research session itself, ordered by the bring-up order decision (2026-11-18-walleye-bringup-order): the dmesg-clean work (smmu timeout, serial supplier, phandles) is the first, then charger (c1 rebuilt on the z6 init), UFS read-only, then USB networking, touch, display; the first phone items carry the z6 init and the plain background reader so every later try reads its dmesg and /sys facts automatically (a keeper recipe in the item)

Rank
: 1


<a id="walleye-boot-image-align"></a>
### Kernel image misaligned at boot: check the boot image kernel offset against a 2 MiB boundary

Status
: open

Note
: Low rank, harmless; from research walleye-z6-dmesg-mapped (G4).

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: walleye-dmesg-clean: no '[Firmware Bug]: Kernel image misaligned' line, or recorded as the bootloader's

Rank
: 9


<a id="walleye-try-z7"></a>
### Try z7 once from slot b: is the dmesg free of the gpu_gx WARN and the smmu deferral?

Status
: done

Note
: proven by dmesg-clean-check pass on z7 (research walleye-z7-tried)

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: phone

Proof
: z7 once from slot b (images-z7/boot-z7.img sha256 2dc6ff0e13dc042086ee2f46be91afc35944ae87c4a242e36cda28934d107b30), the z6 recipe (walleye-try-z6: host stty loop, banner capture, banner-loop-check, the reader mainline-shell-read.sh as plain background processes); then tests/host/dmesg-clean-check.sh on the reader's directory must say pass (no gpu_gx status stuck WARN, no 5040000.iommu deferral, devices_deferred listing only the known overlay nodes); phone left in Android. Record the result as research.

Rank
: 1


<a id="walleye-charger-on-z7"></a>
### c2: the charger (c1's work) on the z7 base, so its try is read through the working command loop

Status
: open

Note
: c1 was built on z3's silent-ash init (lab PR 173); z6/z7 answer commands. WHEN DONE queue walleye-try-c2 (needs phone, rank 1) and close walleye-try-c1 as superseded (done, 'superseded by c2'): c2 once from slot b with the host stty loop + reader as plain background processes; read the dmesg, /sys/class/power_supply, charger-check.sh; battery first (the phone may be charging on USB: that is the test); phone left in Android.

About
: walleye-first-fastboot-boot

Opened
: 2026-11-18

Needs
: build

Proof
: boot-c2.img = z7 (our kernel, the z7 dtb with adreno_smmu off, the z6 command-loop init with the banner, the reader recipe) + the charger work of c1 (SMB2 charger/rradc built into the kernel; the charger and the 2700 mAh / 4.4 V battery nodes in the dtb): only the kernel config and the dtb differ from z7; the c1 diff re-applied on z7's sources by script; checks as z7's and c1's; tests/host/charger-check.sh (it exists from the c1 item) reads, through the reader, /sys/class/power_supply (the charger's online, status, voltage_now, current_now, health, type) and the dmesg lines of the charger driver; programmer review; hashes

Rank
: 1
