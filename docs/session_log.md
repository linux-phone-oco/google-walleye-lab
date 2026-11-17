<!-- Generated from data/ by tad. Do not edit by hand. -->

# Session Log

<a id="2026-11-17-device-base"></a>
### The device base, from device-template

Entry date
: 2026-11-17

Done
: google-walleye-lab gets its own open tad base from linux-phone-oco/device-template (#4, #5): tables.toml, idealkb.toml (0.9.1), devicebase.toml (parts = 1), generic, records/, queries/, Makefile, bin/install-tad.sh and install-ikb.sh, the tad CI; AGENTS.md and oco.toml now point recording here. device.toml is unchanged (devlab's). Rows of this lane move here from docs next, under the keeper lease (docs decision 2026-11-17-commons-and-device-bases).

Considered
: Recording this in docs as before; the base is the place from now on, so its first row is its own making.

---


<a id="2026-11-17-the-first-walleye-images-built"></a>
### The first walleye images built

Entry date
: 2026-11-17

Done
: Built v7.3 + own msm8998-google-walleye.dts (clang, defconfig + fragment), a busybox gadget initramfs with a QEMU x86_64 self-test, and two v0 boot images (taimen and Lineage offset sets) with a counting check; all through oco-agent jobs, nothing touched the phone. Lab PR 3. Images and SHA256SUMS in ~/Projects/walleye-build/images; reproduce with build/build-all.sh.

Considered
: qemu-system-aarch64 for the initramfs proof (not installed; installing is a change to the lab machine); porthole's cmdline pieces qcom_wdt.arm_on_probe and watchdog.open_timeout (they need porthole's patches, mainline ignores them).

Rejected
: a new kernel clone (reused surya's shared clone as a worktree, as surya does); python for the boot image check (shell + mkbootimg/unpack_bootimg/fdtget)

About
: walleye-kernel-dts

---


<a id="2026-11-17-walleye-builder-stock-ids"></a>
### Stock boot.img read; variants e and f built

Entry date
: 2026-11-17

Done
: Downloaded LineageOS walleye boot.img and dtbo.img (the author's word, decision walleye-control-image-download), verified sha256, unpacked on the host, read header and dtb ids (research walleye-stock-ids-recorded). Built e and f; check-bootimg counts 20 ok for each of a-f (lab PR 6).

Considered
: Building f with the stock board-id/pmic-id as asked: none exist in the stock dtb; chose qcom,board-id <8 0> from its qcom,mtp compatible, no pmic-id, labelled a guess.

Rejected
: qcom,board-id from the dtbo table id 0x1bc: Google's overlay id, not a Qualcomm platform id. Changing the os word: a was the only silent OKAY.

About
: walleye-loaderror-stock-ids

---


<a id="2026-11-17-1900-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Control, e and f tried on slot b by fastboot boot: all Load Error, flags unchanged. Phone in fastboot, slot b current and good, a untouched reserve, stock Android unchanged, nothing written.

Considered
: next: the author's word on a dtbo stub (a write) or on slot a flags; nothing more to try from the host side without it

---


<a id="2026-11-17-walleye-how-the-phone-stands-2"></a>
### How walleye stands

Entry date
: 2026-11-17

Done
: Control boot.img refused with Load Error on slot a too (slot state is not the cause); phone back on slot b via set-active, both slots bootable (retry 1 each), fastboot reachable, nothing written; lease released

Considered
: next: walleye-boot-refused-why - remaining causes need a dtbo/vbmeta write (walleye-loaderror-dtbo-stub waits on the author's word); never repeat the slot-a control

---


<a id="2026-11-17-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: First state restored and control accepted; phone in fastboot, unlocked, slot a active and successful, b untouched (retry 1); nothing flashed

Considered
: next: walleye-own-images-first-state (our image e, then b, by fastboot boot from this state); lease released

---


<a id="2026-11-17-walleye-images-g-h-i"></a>
### Images g, h, i: the gadget at 172.16.43.1

Entry date
: 2026-11-17

Done
: build-initramfs.sh takes NET (default 42: files used as they are, so a-f stay byte for byte; 43 rewrites 172.16.42 to 172.16.43 and ECM MAC byte 2a to 2b into WORK_DIR/net43). Built g=e, h=b, i=f; check-bootimg counts the address in /init and the lease; selftest PASS at NET 42 and 43. sha256 g cc3cc1af..., h 3090bbcd..., i 5fdcd4ca...; images/SHA256SUMS and DIFFERENCES.txt updated.

Considered
: Editing init in place to 43; an address file inside the ramdisk

Rejected
: Both: they change the a-f ramdisk bytes while a keeper is trying e and b

About
: walleye-initramfs-usb-gadget

---


<a id="2026-11-17-builder-vendor-gadget2-image"></a>
### Builder: vendor-gadget2 image

Entry date
: 2026-11-17

Done
: Found the panic cause in the stock kernel's config, built an xz ramdisk + init for the vendor kernel, vendor-gadget2.img (control with only the ramdisk swapped, AVB footer), 10 checks ok, landed google-walleye-lab PR 12, queued the try.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2030-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: s0n (no footer) tried from the healthy state: left fastboot, off USB like s0; the phone is OFF USB and needs a hand power hold (Power 10-30 s), then boots Android on a (a active+successful); nothing flashed

Considered
: next: after the hand reset, the parts bisect (s2 kernel, s3 dtb) one at a time; the footer is ruled out

---


<a id="2026-11-17-walleye-vendor-gadget3"></a>
### vendor-gadget3 built

Entry date
: 2026-11-17

Done
: vendor-gadget3.img: vendor-gadget2 with only /init changed (early kmsg marker, 240 s self-reboot timer cancelled by /tmp/hold, reboot -f then sysrq b). Checked the stock config: MAGIC_SYSRQ=y, enable mask 0x1, PSTORE_RAM y. 14 checks ok incl. ramdisk equal to gadget2's apart from /init. Landed google-walleye-lab 0f14a92. Try queued.

Considered
: timer in the telnet shell only (dies with no shell); a watchdog (none known in the vendor config)

Rejected
: putting the timer after the gadget setup: a failed gadget would then never return

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2035-walleye-gadget3-try"></a>
### vendor-gadget3 tried twice from the first state: kernel_panic, back on a, gadget not shown

Entry date
: 2026-11-17

Done
: devlab fastboot boot now vendor-gadget3.img (sha 608afa37...) from the first state (a current+successful, b unbootable, retry 1), twice. Try 1: left fastboot after 12 s, Android on a after 107 s. Try 2 (gadget-up watcher and lsusb log running): left after 12 s, Android on a after 76 s. ro.boot.bootreason=kernel_panic after both; no 240 s timer reboot. walleye-gadget-up not passed: its watcher output was lost when my cleanup killed the shell, so no pass is claimed; the lsusb log shows 18d1:d001 fastboot-class devices at 20:33:41-20:34:41 (strings not read: not known yet whether ours). After: a active, Android booted (sys.boot_completed 1). Nothing flashed.

Considered
: next: a third try with the watcher kept alive (GADGET_WAIT=120, own job, output to a file) and the d001 device serial/product read at once; panic cause unseen - a deeper question if a clean watcher also shows nothing.

---


<a id="2026-11-17-rows-moved-from-docs"></a>
### Walleye's rows moved here from docs

Entry date
: 2026-11-17

Done
: Under the lease orchestration-move: 41 backlog items (device google-walleye or project google-walleye-lab, lab-tools' own items stay in docs), 20 research, 7 decisions, 10 log entries and their research/ material, replayed from docs through this base's records; 21 parts seeded from docs' support_status of google-walleye (all unknown). Kept as they were, except: an about naming the device or a docs subject (android-reference) is left empty (the base is the device's; across bases: docs about-across-bases); six decision ids gain their date (2026-11-17-...) as this base's ids require. Added queries/lane-next.sql (--needs phone|research|build) for the research and build launchers. From now on walleye records here, through ikb; docs keeps what helps every device.

Considered
: Moving lab-tools' items with device google-walleye too: they are the conveyor's, helping every device.

---


<a id="2026-11-17-2055-walleye-gadget3-third-try"></a>
### vendor-gadget3 third try: panic before the gadget

Entry date
: 2026-11-17

Done
: Took the lease, watcher first (own job, 130 s), fastboot boot of vendor-gadget3 from the first state; read the result; phone back on Android a, fastboot reachable, slot a good.

Considered
: next: a builder makes a kmsg-only init image (separates ramdisk unpack from gadget bring-up)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget4"></a>
### vendor-gadget4 built

Entry date
: 2026-11-17

Done
: vendor-gadget4.img: the control with only the ramdisk swapped, an idle init (kmsg marker, proc, alive markers, reboot -f at 120 s, sysrq b fallback; no gadget). 14 checks ok, AVB verifies, size 33554432. sha256 vendor-gadget4.img db23f804a975d4077568cba458814a8af834846a59b2f8d90662ec21e6abbf1b; initramfs-vendor4.cpio.xz 0900a9db50731a4b0fb9e0a9428e3f91e4d8b993a69178d4605b480f9d09bcd4 (in ~/Projects/walleye-build/images, SHA256SUMS). Expected: a panic at ~75 s means not our gadget; a return at ~120 s means the gadget bring-up panics.

Considered
: alive markers every 30 s (pstore/console trail if it panics)

Rejected
: mounting configfs/pstore in the idle init: it would blur the split

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2105-walleye-how-the-phone-stands-3"></a>
### gadget4 try: the panic is the gadget bring-up

Entry date
: 2026-11-17

Done
: Took the lease; from the first state fastboot boot of vendor-gadget4.img: left fastboot 15 s, back on Android a at 156 s, bootreason reboot, boot_completed 1. Left in fastboot, slot a active and successful, b unbootable retry 1 (unchanged); nothing flashed.

Considered
: next: split the gadget bring-up of gadget3 into steps (configfs, functions, UDC bind), a try for each

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget5"></a>
### vendor-gadget5a/b/c built

Entry date
: 2026-11-17

Done
: Three images for the gadget bring-up bisect, each the control with only the ramdisk swapped, gadget4's init (kmsg marker, 30 s alive markers, reboot -f at 120 s, sysrq b) plus one step: 5a configfs + gadget dir (ids, strings); 5b + ncm function and config symlink; 5c + UDC bind, usb0 172.16.43.1, telnetd. build/build-vendor-gadget5.sh: 14 checks each, all ok, AVB verifies, size 33554432, ramdisk differs from gadget4's only in /init. Files in ~/Projects/walleye-build/images, SHA256SUMS. sha256: vendor-gadget5a.img a89fc3d027fe50c7990d6ba6de13bd7976000d63a5d6dd7a6be92db60fd92a87; vendor-gadget5b.img 2e19c15c8ce00f0f24b2477f5a4617948b4e27a6e079832c49772548bffdc26e; vendor-gadget5c.img d09280cce8860a45c8c76701f319686b40948fa2d264341145255d6f31746662; initramfs-vendor5a.cpio.xz eeb644653738c001611363c6b7c8f542a96dbe4537b18cccbdea0880a101a81c; 5b 163c6ef6940e25b13e8dc89bee745459ec59588de6560866b0628f40b9734ad8; 5c 5a9f77fb0ff759b8e4e2218c9fc0d46506de5f15ad1e4d6b8d1f8fc003217073. Outcome table (tries from the first state, each in order a, b, c, stop at the first panic): panic = bootreason kernel_panic, back on Android a in ~75 s; no panic = phone returns by itself at ~120 s with bootreason reboot. First panic at 5a: configfs mount or gadget dir creation; at 5b: the ncm function or config; at 5c: the UDC bind (the dwc3 controller) or the network/telnetd part. 5c without a panic: the gadget is up and gadget-up.sh (read-only ping/telnet to 172.16.43.1) can pass during the 120 s. No hands expected in any outcome: the init returns by itself, a panic falls back by itself.

Considered
: one script for all three (a loop), one init file per step so each image's source is readable alone

Rejected
: a single image with a step chosen at run time (no way to pass it: the kernel command line is the control's and must stay)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2125-walleye-gadget-bisect"></a>
### Gadget bisect: 5b panics

Entry date
: 2026-11-17

Done
: Took the lease; tried vendor-gadget5a (no panic, back at 156 s) and 5b (kernel_panic, back at 75 s) from the first state on the active slot a as the earlier gadget tries did; stopped at the first panic. Phone left in fastboot, a active+successful, b unbootable retry 1 (unchanged), nothing flashed.

Considered
: next: builder splits 5b: mkdir ncm function only, then the config symlink; or acm/rndis instead of ncm

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget5b-split"></a>
### vendor-gadget5b1..b4 built

Entry date
: 2026-11-17

Done
: Four images, each the control with only the ramdisk swapped, gadget5a's init plus one ncm step: b1 mkdir functions/ncm.usb0; b2 + dev_addr, host_addr; b3 + configs/c.1 and its string; b4 + symlink (= 5b). build/build-vendor-gadget5b.sh, 15 checks each ok (header = control's, kernel = control's, xz ramdisk, AVB verifies, 33554432 bytes, ramdisk differs from gadget4's only in /init, exact step parts). sha256: 5b1 a50624ada0e1c1a2 5b2 2d1bacd3056d28ba 5b3 95f5a5ec86925374 5b4 b923eeb243d3cb50 (full in images/SHA256SUMS). Closed walleye-kmsg-only-init: gadget4 was already built and tried.

Considered
: three images as the backlog said

Rejected
: Three images b1..b3 with config dir and symlink together: a panic in b3 would not say which of the two.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget-n"></a>
### vendor-gadget-n1, n2 built

Entry date
: 2026-11-17

Done
: Two images, each the stock control with only the ramdisk swapped (xz), vendor-gadget5a's init plus the ncm step without any write to the function's settings (walleye-ncm-panic-why): n1 = 5b4 minus the dev_addr/host_addr writes (function, config with its string, symlink; no bind); n2 = n1 + UDC bind, ip link set usb0 address, telnetd on 172.16.43.1. build/build-vendor-gadget-n.sh, 17 checks each ok (header and kernel = control's, AVB verifies, 33554432 bytes, ramdisk differs from gadget4's only in /init, exact step parts, grep for dev_addr|host_addr|qmult|ifname finds none). All earlier images (a-i, gadget1-5c, 5b1-b4, s0-s3) byte for byte as before (sha256sum -c: 0 differ); rebuilt twice with the same hashes. Files in ~/Projects/walleye-build/images, in SHA256SUMS there. sha256: vendor-gadget-n1.img b2e96b420e566d35acc415e964690feed9ecf7d67d0fbcebc8dbe467c67793b2, vendor-gadget-n2.img b007d1e3b54c9da90ed8dc5cb44ee098c9c06776a3ffc39ea64d98a0e4282a31; ramdisks initramfs-vendor-n1.cpio.xz f64aa41a5bfcd792ebb6cf06d4b52257ede6c7e2b77c843d027102bbacdc128d, initramfs-vendor-n2.cpio.xz d10d20db747081618fc4d1e01c75524ed82405387a1620a1c48fdf2c170ccfa4.

Considered
: n2's bind is announced inside its if (not at a line start), so the check counts 4 announcements, not 5. The usb0 address is set by ip link (busybox has CONFIG_IP and FEATURE_IP_LINK); ifconfig hw ether is the fallback if ip is refused.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2230-walleye-ncm-split"></a>
### ncm split: b2 panics

Entry date
: 2026-11-17

Done
: Took the lease; tried vendor-gadget5b1 (returns, reboot) and 5b2 (kernel_panic at 75 s) by fastboot boot from the first state; phone left in fastboot, a active, nothing flashed

Considered
: b3, b4 not tried (stop at the first panic); next: the noattr try with vendor-gadget-n1, n2

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2240-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Fastboot, unlocked, slot a active and successful, b unbootable retry 1; nothing flashed; ncm split done (b2 panics), lease released

Considered
: next: walleye-try-ncm-noattr (n1 then n2, from the first state: Android on a, then adb reboot bootloader)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2155-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: n1 returned; n2 enumerated with our strings and no panic; host cdc_ncm bind failed (MAC string not served). Fastboot, unlocked, slot a active+successful, b unbootable retry 1; nothing flashed; lease to be released

Considered
: next: walleye-gadget-host-bind (ecm / rndis images, builder), then ffs-helper if those fail

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-vendor-gadget-m1"></a>
### vendor-gadget-m1 built: the mass-storage log window

Entry date
: 2026-11-17

Done
: Built vendor-gadget-m1.img (the stock control's boot.img, only the ramdisk swapped; init-m1): a mass_storage gadget (functions/mass_storage.0, lun.0 ro=1 removable=1 cdrom=0 nofua=0, file = a 2 MiB file in the ramdisk), no ncm and no network, every lun attribute written only if it exists and each configfs step announced in kmsg first; every 5 s the first 64 KiB of the file is rewritten in place (beat, uname, cmdline, mounts, udc, dmesg tail; NUL padded); reboot -f at 180 s then sysrq b. build/build-vendor-gadget-m.sh: 18 checks ok. tests/host/massstorage-read.sh finds the LUN by our USB strings (sysfs), reads 64 KiB with O_DIRECT, prints the text, one pass/fail/skip line. sha256 vendor-gadget-m1.img a3cf08d388ab72c5e1532e7ae60c87c48e54524fda6ca2d9be7bf4aeb81f360a, initramfs-vendor-m1.cpio.xz dd59bce15a988dac7488b079deb6558fba2da24b620aa543ea4801f5ecfbc29e, in ~/Projects/walleye-build/images and its SHA256SUMS. Not measured yet: the host reader against a real LUN (the phone was not touched), whether this kernel exposes every lun attribute.

Considered
: Writing the window by dd in place with NUL padding (no stale tail) against rewriting the file whole (the LUN would be reopened).

Rejected
: An inquiry_string attribute to name the LUN: not in the brief's attribute list, the host finds the LUN by the usb strings instead.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-builder-vendor-gadget-n3-image"></a>
### Builder: vendor-gadget-n3 image (FunctionFS ACM console)

Entry date
: 2026-11-17

Done
: Built the static aarch64 helper build/ffs-acm/ffs-acm.c (descriptors V2 FS/HS/SS then strings to ep0, only then the UDC bind; one class 2/2/1 interface with 3 endpoints; pty shell bridged to ep2/ep3 by two forked pumps; setup requests for line coding answered, others stalled; no function attribute written), init build/initramfs-vendor5/init-n3 (functions/ffs.acm, devpts, functionfs mount, helper in the background, self-reboot at 180 s) and build/build-vendor-gadget-n3.sh (21 checks, all ok). Images in ~/Projects/walleye-build/images, added to SHA256SUMS: ffs-acm fea51dd8fc5057ced8ac358607210676612c68209edb4a86a61c64086fef1f04; initramfs-vendor-n3.cpio.xz 0be5a7f05f878a8be608a333d410a5619fc10940bd85cbe1635444a196337a9e; vendor-gadget-n3.img 6672eb844fb5435ebb336e33a0b59db069bc2e62a6e01f98d760f5646b1d214c. tests/host/ffs-acm-up.sh fails without the phone, as it must. Not run on the phone.

Considered
: Rust helper: no aarch64 Rust target on the lab, so the C helper of the decision was built with aarch64-linux-gnu-gcc -static; the helper takes no arguments and reads no environment, every path is a constant, so there is nothing for argenv to declare. Threads: rejected for two fork pumps.

Rejected
: Mass-storage window in n3: n3 changes one thing (the channel) from n2. Not tested under qemu: FunctionFS needs a UDC.

About
: walleye-first-fastboot-boot
