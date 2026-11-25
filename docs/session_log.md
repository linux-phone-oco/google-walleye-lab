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
: Built vendor-gadget-m1.img (the stock control's boot.img, only the ramdisk swapped; init-m1): a mass_storage gadget (functions/mass_storage.0, lun.0 ro=1 removable=1 cdrom=0 nofua=0, file = a 2 MiB file in the ramdisk), no ncm and no network, every lun attribute written only if it exists and each configfs step announced in kmsg first; every 5 s the first 64 KiB of the file is rewritten in place (beat, uname, cmdline, mounts, udc, dmesg tail; NUL padded); reboot -f at 180 s then sysrq b. build/build-vendor-gadget-m.sh: 22 checks ok. tests/host/massstorage-read.sh finds the LUN by our USB strings (sysfs), reads 64 KiB with O_DIRECT, prints the text, one pass/fail/skip line. sha256 vendor-gadget-m1.img a3cf08d388ab72c5e1532e7ae60c87c48e54524fda6ca2d9be7bf4aeb81f360a, initramfs-vendor-m1.cpio.xz dd59bce15a988dac7488b079deb6558fba2da24b620aa543ea4801f5ecfbc29e, in ~/Projects/walleye-build/images and its SHA256SUMS. Not measured yet: the host reader against a real LUN (the phone was not touched), whether this kernel exposes every lun attribute.

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
: CORRECTION (walleye-coordinator, 2026-11-17): the builder wrote here that the lab has no aarch64 Rust target - that is wrong. The lab has one: lpoco-health is built static for aarch64-unknown-linux-musl with the toolchain in ~/.local/share/devlab-cross (rustup + aarch64-linux-gnu-gcc as the linker; see lab-tools health/build.sh). The C helper of this image is therefore a PROBE only; before the helper is kept or reused it is rewritten in Rust (backlog walleye-ffs-helper-rust). The helper takes no arguments and reads no environment, every path is a constant, so there is nothing for argenv to declare. Threads: rejected for two fork pumps.

Rejected
: Mass-storage window in n3: n3 changes one thing (the channel) from n2. Not tested under qemu: FunctionFS needs a UDC.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2155-walleye-m1-try"></a>
### m1 try: gadget enumerates as mass storage, the host cannot read it

Entry date
: 2026-11-17

Done
: Took the lease; fastboot booted vendor-gadget-m1 (sha a3cf08d3...) from the first state: left fastboot at 12 s, enumerated 1d6b:0104 with our strings at 18 s, no panic, class 08 interface up, phone returned by itself to Android on a. Read of the log window failed: no usb_storage module on the host for kernel 7.3.3 (modules dir has 7.3.4 and 6.18.60-lts). Phone put back in fastboot (adb reboot bootloader): a active+successful, b unbootable retry 1, unchanged; nothing flashed.

Considered
: next: the host loads usb_storage (reboot into 7.3.4 - a host change, not mine), then repeat the m1 try and read the window; or the FunctionFS ACM helper, which needs no host storage driver

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2200-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Phone in fastboot, unlocked; slot a active and successful, b unbootable retry 1 (unchanged); stock Android on a; nothing flashed. m1 enumerates as our mass-storage gadget with no panic; its log window is unread because the lab host has no usb_storage module for kernel 7.3.3.

Considered
: next: walleye-try-massstorage again once the host has usb_storage (reboot into 7.3.4), or the FunctionFS ACM helper

---


<a id="2026-11-17-2210-walleye-how-the-phone-stands-4"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: n3 tried once: enumerates 1d6b:0104 with ACM class interface after 18 s, no panic; host lacks cdc_acm for running kernel 7.3.3 so ffs-acm-up fails; phone back on Android a by self-reboot, now in fastboot, slot a good, nothing flashed

Considered
: next: reboot the lab host into 7.3.4 (host owner), then retry n3 and the ACM test

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-builder-n3r"></a>
### Builder: vendor-gadget-n3r (Rust FunctionFS ACM helper)

Entry date
: 2026-11-17

Done
: Rewrote the helper in Rust (build/ffs-acm-rs, libc crate, panic=abort, static musl, linked by aarch64-linux-gnu-gcc as lab-tools health/build.sh does): same steps as the C probe, descriptors and strings checked byte for byte against the probe's by a unit test (tests/c-probe-blobs.bin). build-vendor-gadget-n3r.sh makes vendor-gadget-n3r.img: n3 with only /bin/ffs-acm swapped (init byte for byte, checked against initramfs-vendor-n3); 21 checks ok; second build gave identical hashes. Hashes in images/SHA256SUMS. No arguments or environment read, so no argenv. Phone not touched.

Considered
: nix crate instead of libc: more code for the same fork/ioctl calls

Rejected
: Replacing n3's C helper: it stays as the probe; n3r is a separate image so the try can compare

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2230-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: m1 tried once from the first state: runs, enumerates, usb-storage binds /dev/sda; host read refused (no disk group). Phone back in fastboot, unlocked, slot a active+successful, b unbootable retry 1; nothing flashed

Considered
: next: host read access then retry m1; rejected: changing host groups or using sudo myself (system setting, the author's)

---


<a id="2026-11-17-2245-walleye-overlay-stubs-build"></a>
### Overlay stubs build: s3o image

Entry date
: 2026-11-17

Done
: Generated the 130-stub compat dtsi from the dtbo entry 3 fixups, built our dtb with -@, host overlay-merge test passes, built image s3o (control + our new dtb) with checksums, queued the phone try.

Considered
: Stubs at the stock paths (would need a /soc node beside our soc@0); flat stubs under one parent node chosen: fdtoverlay and libufdt need only the symbol and a node.

Rejected
: Replacing s3 with the new dtb: s3's result stays reproducible, so the new image is s3o beside it.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2310-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: s4 tried once: overlay symbols fix the s3 hang (left fastboot, Android a at 36 s, reboot). Phone in fastboot, unlocked, slot a active+successful, b unbootable retry 1; nothing flashed; lease released

Considered
: next: our kernel with the symbols dtb (vendor-gadget with s3o dtb); read why the stock kernel resets with our dtb

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-builder-mainline-light"></a>
### Builder: ml1 and ml2 images (first light of our mainline kernel)

Entry date
: 2026-11-17

Done
: Built v7.3 + walleye.config (+QCOM_WDT, MASS_STORAGE, RD_XZ) into out-ml; added the watchdog node to the dts, build/initramfs-mainline/init-ml, build/build-mainline-light.sh (32 checks, rebuild gives the same hashes) and tests/host/mainline-light-up.sh. Images in ~/Projects/walleye-build/images-ml: boot-ml1.img 41d8efc0c6e5c3ed3086e66f605c39ddc4a0bcb8a0e266a2cbbd010738cc636b, boot-ml2.img 63b2977f1f21c5de40b2c8efcbb0f033860ad5fae2c3ee9eca052f77f5979add, initramfs-mainline-light.cpio.xz 9feb844211c3e5b636b2481eb920d36cdf698eaf4441090574edd943a69b7e01 (SHA256SUMS beside them). Not run: the init under QEMU (syntax checked only), dtc W=1 on the new node. An early hang (before the watchdog opens) still needs the author's power hold.

Considered
: ml2 pack: raw Image in an lz4 frame (-9 -BI -B7) as build-bisect.sh does

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-keeper-ml-light"></a>
### ml1 and ml2 first-light tries

Entry date
: 2026-11-17

Done
: ml1 then ml2, each once from the first state; both fail the light test; phone back in fastboot, slot a current and successful, nothing written

Considered
: next: read ramoops (author unlock) or UART; watchdog node address check

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-how-the-phone-stands-ml"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Phone in fastboot (FA81N1A00790), slot a current+successful, b unbootable retry 1; stock Android on a is what runs; no kernel kept

Considered
: next: panic read by ramoops or UART, decided by the walleye coordinator

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-builder-rust-ci"></a>
### Builder: Rust CI for ffs-acm-rs

Entry date
: 2026-11-17

Done
: Added a rust job to check.yml (fmt --check, clippy --all-targets -D warnings, test in build/ffs-acm-rs); all three pass on the lab first. Split walleye-code-hygiene-rust-ci: CI part done; C probe removal left open, blocked by walleye-ffs-rust-try because the coordinator's note says remove it only after n3r is tried and that try has not happened. No image built, nothing queued for the phone, phone not touched.

Considered
: removing the C probe now: n3 retry and n3r try are still open

Rejected
: Removing the C probe now: it would break the order the coordinator set

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-builder-ml3"></a>
### Builder: ml3, ramoops at the phone's own region

Entry date
: 2026-11-17

Done
: Moved the dts ramoops node to 0xa1100000 (2 MiB, record 0x80000, console 0x20000, no pmsg); rebuilt the dtb (build-kernel.sh, out-ml); build/build-ml3.sh made ~/Projects/walleye-build/images-ml3/boot-ml3.img with 10 checks ok: node by fdtget, old node gone, no carve-out overlap, PSTORE_CONSOLE and PSTORE_RAM =y, ramdisk and kernel bytes equal ml1's, only the dtb part differs. sha256 boot-ml3.img 5c2574706882f5c8f40b134bdf501e22c092093e494b5aad25efdabe4ba105c4; initramfs 9feb844211c3e5b636b2481eb920d36cdf698eaf4441090574edd943a69b7e01 (same as ml1); ml1 41d8efc0..., ml2 63b2977f... unchanged. Queued walleye-try-ml3-ramoops.

Considered
: A new kernel build dir: not needed, the kernel is unchanged and only the dtb is rebuilt.

Rejected
: Reading ramoops parameters from the phone: builders never touch it.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-keeper-m1-window"></a>
### m1 window read: lpoco-host acceptance passes

Entry date
: 2026-11-17

Done
: Took the lease; one m1 try from slot a; read the window on host 7.3.4 as the lab user (pass). Phone back in fastboot via adb reboot bootloader: a active+successful, b unbootable retry 1; nothing flashed.

Considered
: next: walleye-ffs-rust-try (n3r, ttyACM readable now); then ffs-helper-try n3 may close

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-how-the-phone-stands-m1"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Phone in fastboot (FA81N1A00790), unlocked, slot a active+successful, b unbootable retry 1; stock Android on a; no kernel kept; nothing flashed. m1 window readable by the lab user; the vendor kernel runs our init.

Considered
: next: walleye-ffs-rust-try (n3r), then the vendor kernel's reserved-memory/ramoops facts for mainline

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2340-walleye-ml3-try"></a>
### ml3 try: pstore empty

Entry date
: 2026-11-17

Done
: Took the lease; booted boot-ml3.img once from the first state (slot a, b unbootable by the first state, nothing flashed); no USB light; Android back on a after 72 s; adb root read /sys/fs/pstore (empty) and the ramoops parameters (0xa1100000, matches); wrote tests/host/ml3-ramoops-read.sh (fails); left the phone in fastboot on slot a, good, lease to release.

Considered
: Reading raw DDR at 0xa1100000 (no /dev/mem on this kernel); a ml4 with earlycon on the msm serial (blind without UART).

Rejected
: Re-trying ml3: the result is deterministic; ml1/ml2/ml3 all end alike.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2330-walleye-ml4-build"></a>
### ml4 built

Entry date
: 2026-11-17

Done
: build/build-ml4.sh, build/initramfs-mainline/init-ml4, tests/host/ml4-bin-table.sh; images-ml4 with checksums; walleye-try-ml4 queued for the phone

Considered
: reusing build-mainline-light.sh (its checks assert ml1's init, so a separate script)

Rejected
: not petting the watchdog in the sleep: scores 6-7 would be cut by the 30 s reset and bins would saturate

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2345-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: n3r tried once on slot b: gadget enumerates (1d6b:0104, walleye-first-boot) but cdc_acm refuses it: Zero length descriptor references (-22); ffs-acm-up fails. Phone fell back to Android by itself, rebooted to fastboot: unlocked, slot b active+successful, a good; nothing flashed; lease released

Considered
: next: n3r2 with CDC functional descriptors (header, call-mgmt, ACM) in the helper; amends walleye-ffs-acm-single-interface - asked of the coordinator

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-2350-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: ml4 tried twice from the first state (slot b, nothing flashed): back as Android at 75 s and 76 s, pstore empty. Phone in fastboot, unlocked, current slot b, a and b successful, lease walleye-keeper released at landing.

Considered
: next: tell score 7 from the watchdog (ml5 with a long score-independent sleep) - for the walleye coordinator

About
: walleye-first-fastboot-boot

---


<a id="2026-11-17-walleye-ml5-build"></a>
### ml5 built

Entry date
: 2026-11-17

Done
: build-ml5.sh, init-ml5a, init-ml5b, ml5-bin-table.sh; images in walleye-build/images-ml5 with SHA256SUMS; try walleye-try-ml5 queued

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0010-walleye-ml5-try"></a>
### ml5 try: kernel never reaches init

Entry date
: 2026-11-17

Done
: Took the lease; booted ml5a twice (Android back after 75 s, 75 s) and ml5b twice (75 s, 78 s) by fastboot boot from slot b (both slots good, nothing flashed). ml5a base 75 s is the ~70 s bin: the mainline kernel never reached init, as ml1-ml3; ml5b - ml5a = 0 and 3 s, so no init ran. Phone left in fastboot, slot b active and successful, a successful; lease released.

Considered
: UART/earlycon before init; a vendor-kernel-side check, for the coordinator.

Rejected
: Re-trying ml5: four tries agree, deterministic.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-pk-build"></a>
### pk1 and pk2 built

Entry date
: 2026-11-17

Done
: build/pk-kernel.sh, build/build-pk.sh; images in walleye-build/images-pk with SHA256SUMS (research/walleye-first-fastboot-boot/2026-11-18-pk); try walleye-try-pk queued

Considered
: porthole's own cmdline and stub dtbo: not taken, to keep ml5a's image the only other difference

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0030-walleye-pk-try"></a>
### pk1/pk2 try: kernel never reaches init

Entry date
: 2026-11-17

Done
: Booted pk1 twice then pk2 twice by fastboot boot from slot b (both slots good, nothing flashed). Android back after 75, 75, 75, 75 s: the ~70-78 s no-init bin, same as ml5a (75 s). porthole's v7.2.2+262-patch kernel (taimen/walleye dtb) does not reach init either; pk1 = pk2 so the dtb makes no difference. Returned to fastboot by adb reboot bootloader after each.

Considered
: Four tries agree, deterministic. Next for the coordinator: the failure is before init for every mainline kernel and both dtbs, so UART/earlycon or ramoops read (decision walleye-panic-read-order).

Rejected
: Re-trying pk1/pk2: four identical times

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0035-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Phone in fastboot, unlocked, slot b current, a and b both successful, retry 0; nothing flashed; lease released

Considered
: next: coordinator decides the no-init vector (ramoops read / UART) for the mainline kernels

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-nocon-build"></a>
### pk3 and ml6a built

Entry date
: 2026-11-18

Done
: build/build-nocon.sh; images in walleye-build/images-nocon with SHA256SUMS (research/walleye-first-fastboot-boot/2026-11-18-nocon); try walleye-try-no-earlycon queued

Considered
: keeping androidboot.console=ttyMSM0: it is a stock androidboot word, not a kernel console=, and the item says keep the stock androidboot pieces

Rejected
: building the images twice with different tools: one script on the recorded images keeps every other byte equal

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-17

Done
: Ran the no-earlycon tries; phone in fastboot, unlocked, slot b current, a and b successful, retry 0; nothing flashed; lease to be released

Considered
: next: coordinator decides the log channel (ramoops read / UART) for pk3 vs ml6a

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-pk456-build"></a>
### pk4, pk5, pk6 built

Entry date
: 2026-11-17

Done
: build/build-pk456.sh: one swap each between pk3 and ml6a, no earlycon; 31 checks ok, rebuild identical; hashes in research/walleye-first-fastboot-boot/2026-11-18-pk456; try walleye-try-bisect-pk queued for the keeper

Considered
: Splitting each kernel blob at the dtb that ends it (as check-bisect.sh) and repacking with pk3's header and AVB props

Rejected
: Rebuilding the kernels or dtbs: the swap uses the recorded images' own bytes, so nothing else can differ

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-dtb-bisect-build"></a>
### d1..d6 built (our dtb bisect)

Entry date
: 2026-11-18

Done
: build/build-dtb-bisect.sh: six images, each pk4 with a node group removed from our dtb by fdtput; 90 checks ok, rebuild identical; hashes in research/walleye-first-fastboot-boot/2026-11-18-dtb-bisect; try walleye-try-dtb-bisect queued for the keeper

Considered
: Diffing the decompiled dtbs by path and property first, so each group is a node we have and porthole lacks or places differently

Rejected
: Removing spmi@800f000 as a group: the vendor dtbo overlay needs it, so that image would fail at the bootloader's merge and test nothing; adding our nodes to porthole's dtb (pk5 + group) in this round: removal from the dead dtb needs no kernel or dts rebuild and the reverse direction waits for a null result

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-pk5l-build"></a>
### pk5l built (first mainline userspace image)

Entry date
: 2026-11-18

Done
: build/build-pk5l.sh, init-pk5l, tests/host/mainline-acm-up.sh; 39 checks ok, rebuild identical; hashes in research/walleye-first-fastboot-boot/2026-11-18-pk5l; try walleye-try-pk5l queued rank 1

Considered
: One gadget config with both functions so a single bind gives console and window; petting the watchdog with a held-open fd 3 and a hold file to stay up

Rejected
: Two images (ACM only, window only): a failure of one function would then hide the other; a getty/login: a respawned ash on the tty is the least that answers

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1700-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: pk5l tried twice from fastboot, no ACM, fell to Android both times (57 s, 42 s); phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel

Considered
: next: walleye-try-dtb-bisect (d1..d6); pk5l's init may not start at all, a console-less sign is the return time only

---


<a id="2026-11-18-walleye-pk5l2-build"></a>
### pk5l2 built (ramoops in the phone's own region)

Entry date
: 2026-11-18

Done
: build/build-pk5l2.sh: pk5l's kernel blob split, the dtb's ramoops node edited with fdtput to 0xa1100000/2 MiB (record 0x80000, console 0x20000, pmsg 0), reassembled with pk5l's ramdisk, header and footer; 20 checks ok, rebuild identical; try walleye-try-pk5l2 queued rank 1

Considered
: dtc round trip to rename the node to ramoops@a1100000

Rejected
: dtc dts round trip: it re-emits the whole dtb, so more than the ramoops node would change; fdtput keeps size and every other byte

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1800-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: pk4 and d1..d6 twice each, all back as Android at 75 s; phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel; lease released

Considered
: next: research/build item to diff our dtb vs porthole's and build d7+

---


<a id="2026-11-18-walleye-taimen-walleye-build"></a>
### Overlay analysed, taimen-to-walleye images built

Entry date
: 2026-11-18

Done
: Merged the dtbo overlay into pk5's and our dtb and diffed: inert on ours beyond one i2c node. Diffed porthole's taimen and walleye dtbs: 15 nodes disabled, ~100 absent. Built w1..w5, wr, wa (399 checks, identical rebuild; programmer review fixed phandle check, header, stale sums).

Considered
: Overlay-label neutralizing images L1..Ln as the item said

Rejected
: L1..Ln: the analysis falsified their premise

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2000-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: wa tried 3 times (42, 63, 63 s); phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel; lease released

Considered
: next: decide whether 63 s is a new return class (a ramoops read, or wa x2 more); then w1..w5, wr twice each as planned

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: w1..w5, wr x2 each tried (42-63 s, all alive); phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel; lease released

Considered
: next: bisect properties between taimen dtb and pk2's dtb (not nodes)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-property-bisect-build"></a>
### wp1..wp5 and wpa built (property bisect)

Entry date
: 2026-11-18

Done
: Diffed wa and pk2 property by property (phandles resolved): five groups, nothing in memory/reserved-memory/chosen/cpus/timer/psci. Built wp1..wp5, wpa with build/build-property-bisect.sh (68 checks ok, rebuild identical); queued walleye-try-property-bisect for the keeper

Considered
: wa vs pk2 differ in phandle numbering, node order and dtb size too; wpa tests whether any property matters. Mirror images from our dtb after the first results

Rejected
: Binary halves: five groups, singles are as cheap

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: wpa x2 tried (60 s each, alive); phone in fastboot, slot b current, a and b successful, nothing flashed, stock kernel; lease released

Considered
: next: bisect phandle order and dtb size between taimen dtb and pk2's dtb (not properties)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-pk5g-build"></a>
### g1..g3 built (mainline gadget staged on pk5)

Entry date
: 2026-11-18

Done
: build/build-pk5g.sh and init-pk5g: three images on pk5's kernel/dtb/header, ramdisks differing in /stage; 58 checks ok, rebuild identical; try walleye-try-pk5-stages queued

Considered
: three separate inits (rejected: one init + /stage keeps the diff to one file); 60 s holds so alive and dead are 30+ s apart

Rejected
: ACM and a shell loop in these steps: pk5l had them and ended early

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: phone is gone from USB (no fastboot, no adb; devlab fastboot state: off or needs hands); pk5g stages g1..g3 (backlog walleye-try-pk5-stages) not started; nothing flashed, stock kernel, last known slot b current, a and b successful

Considered
: next: a hand power-cycle (or cable reseat) brings it back to Android/fastboot; then take walleye-try-pk5-stages

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-battery-watch-built"></a>
### walleye battery watch built

Entry date
: 2026-11-18

Done
: tests/host/battery-ok.sh (fastboot getvar battery-voltage or adb dumpsys battery; exit 1 below 3700 mV, skip when unread), tests/host/idle-android.sh (end state: Android booted, not in fastboot), tests/host/battery-selftest.sh (fake fastboot/adb, seen failing first, then pass); keeper rule added to AGENTS.md. Phone not touched.

Considered
: reading the phone for real in this run

Rejected
: reading the live phone now: builders never touch a phone; the selftest uses fake fastboot and adb

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2330-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: g1,g2,g3 tried twice each (research walleye-pk5-stages-tried); phone back in Android on slot b (idle-android pass, battery 4377 mV), nothing flashed, stock kernel; lease released

Considered
: next: g3 never enumerated and g2 returns at 138 s; find what differs from expectation (UDC name, ordering, dtb usb node)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-t1-build"></a>
### T1 image built

Entry date
: 2026-11-18

Done
: build/build-t1.sh, init-t1, tests/host/pstore-t1-read.sh; boot-t1.img built from pk5, queued for the keeper

Considered
: Basing T1 on pk5l2 (its ramdisk is the ACM gadget; pk5 is the alive base)

Rejected
: Basing on pk5l2: sysrq c gives a deliberate end only if the init ran, and pk5l never showed it

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2345-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-try-pk5-stages was already done and recorded by an earlier keeper (2330, #114); I found the phone off USB, then in Android, read fastboot state (b current, a and b successful), and ran g1 x2 and g2 x1 again: same result, no 1d6b:0104. Phone in Android on b, nothing flashed, stock kernel, lease released

Considered
: next: walleye-pstore-t1-try says run only after a kernel has shown life; none has, so it is a turning point for the coordinator

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0400-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: T1 tried on b (fastboot boot only, nothing flashed): back in Android after 159 s; battery 4384 mV/100% before. /sys/fs/pstore empty, no ramoops dir: our marker and panic text did not survive (cause B). Phone in Android on b, stock kernel, adb root, slots a and b good; lease released

Considered
: next: coordinator decides what follows cause B (pstore not retained across the sysrq c reset; another way to see the kernel's output)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-t3-build"></a>
### T3 image built

Entry date
: 2026-11-18

Done
: build/build-t3.sh, init-t3, tests/host/pstore-t3-read.sh; boot-t3.img built from pk5, queued for the keeper (backlog walleye-pstore-t3-wdog-bite). Phone not touched.

Considered
: Setting the 10 s timeout with busybox watchdog -T and one ping (-t 3600); the dtb's timeout-sec 30 alone

Rejected
: Opening /dev/watchdog with 'exec 3>' in sh: leaves the default 30 s (dtb) and a failed open would end init; busybox watchdog has the ioctl for the timeout

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2359-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: T3 tried once: pstore empty, back as Android on b at 42 s, nothing flashed, stock kernel, fastboot reachable (slots a,b successful)

Considered
: next: turning point for the coordinator - empty pstore after an alive kernel with an unpetted watchdog does not tell no-bite from lost region; needs UART or another witness of init before more tries

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-vf1-build"></a>
### vf1 built

Entry date
: 2026-11-18

Done
: Built vf1 (init-vf1, build-vendor-facts.sh, vendor-facts-read.sh, selftest), reviewed by the programmer subagent, fixed, queued walleye-try-vf1 for the keeper.

Considered
: Copying the raw /sys/firmware/fdt for dtc: rejected, it carries the properties we drop by name; the per-property copy is enough.

Rejected
: A tar of live /proc and debugfs paths (sizes read as 0).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0430-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf1 tried twice from the first state: gadget up, LUN readable, header empty both times; phone back as Android on b, stock kernel, flags a and b successful, fastboot reachable, nothing flashed; host on 7.3.4 with usb_storage

Considered
: next: turning point for the coordinator - a first-round marker written at once by the vf1 init (or the dump step read) so an empty header can be told from a dead init; meanwhile the queue's other items wait on the author (unlock, UART)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-vf2-build"></a>
### vf2 built

Entry date
: 2026-11-18

Done
: init-vf2, build-vendor-facts-vf2.sh, reader and selftest extended; programmer review's five findings fixed; try queued

Considered
: a second 2 MiB window for the text (rejected: the LUN size cannot change after bind; one 8 MiB window with a 64 KiB text region)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0510-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf2 tried twice from the first state: init alive, steps run, beat ticks; phone left in fastboot on slot b, a and b successful, stock kernel, nothing flashed

Considered
: next: turning point for the coordinator - a poller that saves the whole 64 KiB text region every 2 s to files (not only the end) and fetches the blob the moment its header appears, since the phone leaves USB after ~45-60 s; reader's single-shot read is racy

---


<a id="2026-11-18-walleye-vf3-build"></a>
### vf3 built

Entry date
: 2026-11-18

Done
: init-vf3, build-vendor-facts-vf3.sh, vendor-facts-poll.sh and its selftest; programmer review's findings fixed; try queued

Considered
: appending with dd bs=1 seek bytes (rejected: slow; 4096-aligned rewrite of the last partial block instead); one nested dt/ directory tree in the poller (rejected: a block named dt/soc blocks dt/soc/spi; flat names with %)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2400-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf3 twice from the first state: no gadget ever, no blocks, back as Android at 78 s both times (dead-init class). Phone left in fastboot on slot b, flags a and b successful, stock kernel, nothing flashed, battery 4377 mV

Considered
: next: turning point for the coordinator - vf2 init was alive, vf3 (gadget first, then progressive dump) is not; diff vf2 vs vf3 init, find what kills it before the gadget bind

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-vf4-build"></a>
### vf4 built

Entry date
: 2026-11-18

Done
: init-vf4 (vf2's first 86 lines byte for byte to the bind, then vf3's progressive dump from byte 65536), build-vendor-facts-vf4.sh, poller parametrised (VF_MARK/VF_HEAD/VF_BLOCKS, steps.txt), vendor-facts-poll-selftest-vf4.sh; try queued

Considered
: Keeping vf2's blob code after the bind as dead code so the diff has additions only: rejected, the diff is checked as 'identical up to the bind, first hunk after it'. A second poller for vf4: rejected, one poller with a layout by environment.

Rejected
: Moving the gadget earlier again (vf3's order) before knowing whether the early bind killed it.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0535-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf4 tried twice (poller first): no gadget, 0 blocks, empty beat log, back as Android at 78 s both; phone in Android on b, stock kernel, nothing flashed, fastboot reachable earlier (a and b successful)

Considered
: next: turning point for the coordinator - vf4 (vf2's first 86 lines + progressive dump) died like vf3 though vf2 lived: diff vf2 init vs vf4 init before the bind, or retry vf2 to see whether its 43 s gadget repeats; other queue items wait on the author (unlock, UART)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0610-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vf control done: vf2 twice alive (gadget up, empty header), vf4 twice dead unwatched; phone back as Android on b, stock kernel, flags a and b successful, nothing flashed, fastboot reachable

Considered
: next: turning point for the coordinator - vf4 dies without the poller, so its progressive dump after the bind is the suspect; bisect the dump (vf4 with a vf2-like tail) or write a first-round marker at once

---


<a id="2026-11-18-walleye-vfx-build"></a>
### vfa..vfd built (one hardware source per image)

Entry date
: 2026-11-18

Done
: init-vfx (m1 plus one payload before the bind, chosen by /etc/walleye-vf), build-vendor-facts-vfx.sh (4 images, 23 checks each, rebuild identical), window-payload-read.sh and its selftest (18 ok); try queued as walleye-try-vfa

Considered
: Four init files: one init and a one-word file instead, so the four images differ in that file alone and the diff against m1 is one block. The beat in the window's last 4 KiB so a running init is told from a hung one without touching the payload. Header written first with no part, so a read that kills the kernel is told from a never-started init.

Rejected
: Keeping m1's 64 KiB text rewrite every 5 s: it would overwrite the payload. A tar or gzip of the payload, progressive appends and a poller: the dump after the bind is the suspect (vf3, vf4 dead).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0650-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: vfa twice, vfb, vfc once, vfd twice tried from the first state through devlab fastboot boot; phone is Android on slot b, stock kernel, a and b successful, nothing flashed, fastboot reachable; the live fdt, iomem, interrupts, cmdline and regulator summary are in the material path

Considered
: next: build mainline USB nodes from the vfa fdt (porthole/taimen dtb bisect can now be compared against the live walleye fdt); vfb's clk_summary needs a look at why the read fails (debugfs path); fix battery-ok.sh to time out while Android runs; queue's other items wait on the author (unlock, UART)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-fdt-compare-build"></a>
### fdt compare and x1..xa built

Entry date
: 2026-11-18

Done
: Decompiled and normalized our, the alive, wpa and the running fdt; listed what ours lacks; built x1..x7 and xa with build-fdt-bisect.sh (114 checks, rebuild identical, programmer-reviewed); wrote the USB read of the running fdt; queued the try.

Considered
: A phandle-rewriting merge tool; splicing at source level.

Rejected
: Both: see the decision 2026-11-18-walleye-bisect-from-alive-by-fdtput.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-battery-ok-timeout-built"></a>
### battery-ok.sh timeout built

Entry date
: 2026-11-18

Done
: battery-ok.sh reads adb first, then fastboot, every call bounded (timeout -k 1 6); neither answers: skip, exit 2, 14 s worst. Selftest has hanging fake adb/fastboot. Programmer review: fixed exec bit, -k, missing timeout(1), selftest outer guard.

Rejected
: fastboot first: hangs the Android case the first 7 s for nothing; a longer timeout: keeps total under the 15 s bound

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0700-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: fdt bisect x1..xa twice each from slot b: x1 and xa dead, others alive; phone back as Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: our dtb plus gpio-reserved-ranges <0 4>,<81 4>, try twice (alive expected); the earlier vf1 empty-header turning point may be this same cause

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-y-build"></a>
### y1 and y2 built (our dts with gpio-reserved-ranges)

Entry date
: 2026-11-18

Done
: Added &tlmm gpio-reserved-ranges <0 4>, <81 4> to our dts; build/build-y.sh builds boot-y1.img and boot-y2.img (49 checks, rebuild identical, hashes in the research material); programmer review; try queued as walleye-try-y.

Considered
: Taking the pshold/watchdog nodes along; rebuilding the whole kernel (only the dtb changes, Image.gz is ml6a's byte for byte).

Rejected
: Both, see decision 2026-11-18-walleye-our-dts-gets-gpio-reserved-only.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0800-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Tried y1 twice (alive, Android back 36 s) and y2 twice (gadget 1d6b:0104 at 18 s, ttyACM0): our dtb with gpio-reserved-ranges boots mainline to USB. Phone is Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader.

Considered
: next: read the ACM shell on y2 (dmesg, the facts the vendor dump was after); give mainline USB its nodes properly; consider restart@10ac000 and the apps watchdog from the fdt compare; upstream the gpio-reserved-ranges commit

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1900-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-keeper wrote tests/host/mainline-shell-read.sh (reads y2's ttyACM: dmesg first, facts, device lists, touch /tmp/hold, redacted, saved outside git) and mainline-shell-selftest.sh (fake phone on a socat pty; 8 ok); host only, phone untouched: Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader. Item walleye-first-mainline-shell stays open: the real read needs a y2 try

Considered
: next: try y2 (our dtb + gpio-reserved-ranges) twice from slot b with WAIT=60 mainline-shell-read.sh running, then write the research row of what the dmesg shows

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0830-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Reader fixed (echo-matching marker, ANSI strip) and run on y2: first mainline dmesg read. Phone in fastboot, slot b current, a/b successful, stock kernel, nothing flashed.

Considered
: next: PMIC spmi reg nodes (pm8998/pmi8998), smmu timeout, then charger/display

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-research-bringup-order"></a>
### walleye-research: the bring-up order plan

Entry date
: 2026-11-18

Done
: Read the first mainline map, the vendor hardware inventory and the mainline path; checked public sources for the charger, fuel gauge, haptics, amps and panel; wrote the plan material, one research row, one decision and eleven queued items with proofs.

Considered
: UFS gated by the PMIC (the brief's guess) - the vendor tree puts its rails on the RPM, so no. Display first.

Rejected
: Queuing a phone try: the phone is off USB (walleye-phone-needs-hands); the first tries come from the builds.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-pmic"></a>
### walleye builder: PMIC nodes, z1 built

Entry date
: 2026-11-18

Done
: walleye-pmic-spmi: dts includes pm8998/pmi8998, build-z.sh builds z1 (16 checks, rebuild identical), pmic-dmesg-check with selftest (12 ok; programmer review fixed the loose overlay filter, added 0-01 and the probe-failed line, the check says registered, not working), the reader also lists /sys/bus/spmi/devices; queued walleye-try-z for the keeper.

Considered
: Own pmic nodes named like the overlay's; zero-line proof.

Rejected
: See decision 2026-11-18-walleye-pmic-from-mainline-dtsi.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0900-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z1 tried once from slot b: alive, ttyACM0 up, reader denied the tty so the PMIC dmesg was not read; phone back as Android on b, stock kernel, nothing flashed, adb up, fastboot reachable on demand

Considered
: next: fix tty access for the keeper session (getfacl during the try; uaccess tag only reaches the seat user), then try z1 again with the reader waiting; coordinator to decide who fixes access (host prepare is lab-tools')

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-dmesg-clean-z2"></a>
### walleye-builder: dmesg-clean, z2 built

Entry date
: 2026-11-18

Done
: Found the smmu -110 cause (gpucc =m, no modules), set it =y, rebuilt the kernel, built boot-z2.img with 19 checks, wrote dmesg-clean-check.sh and its selftest (seen failing on the first map), queued walleye-try-z2 for the keeper.

Considered
: Pointing the overlay's clock_gcc label at the real gcc by fdtput for the serial fault.

Rejected
: Putting all three fixes in one image: one change per image.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0930-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z1 tried twice from slot b with the reader: gadget and ttyACM0 up at 18 s, ACL held, shell silent; phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader; walleye-try-z back to open

Considered
: next: does y2's ttyACM0 answer at all (y2 once with the reader)? else read facts through the mass_storage window; walleye-try-z2 after

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0945-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: y2 once then z1 once with the same reader: both silent (gadget 18 s, ttyACM0, no bytes); phone back as Android on slot b, stock kernel, nothing flashed, adb up, fastboot via adb reboot bootloader

Considered
: next: builder makes a banner init (ttyGS0 every 2 s, kmsg marker per step) on y2 first then z1; keeper reads with plain cat before any reader; control item walleye-try-y2-control done

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-dmesg-clean-z3"></a>
### walleye-builder: dmesg-clean, z3 built

Entry date
: 2026-11-18

Done
: Read the first dmesg: the four phandle lines and the serial wait are one cause (the clock_gcc stub has no #clock-cells). Wrote tests/host/overlay-clock-gcc.sh (all four users; seen failing on z2's dtb), build/build-z3.sh (z2 + the clock_gcc symbol on the real gcc, 15 checks, rebuild identical, reviewed by the programmer agent and its findings fixed), queued walleye-try-z3.

Considered
: Pointing the symbol at gcc by fdtput after the compile; the shell being silent on the phone, so the try may learn nothing until the banner init exists.

Rejected
: Disabling overlay nodes; a fake #clock-cells on the stub; folding other fixes into z3.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-0950-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: y2 tried with the old reader: silent too, only the tty echo arrives; phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: banner init (builder) and a raw cat of ttyACM0; devlab suite/phone unavailable: no password file at ~/.config/devlab/google-walleye/password (not created)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-charger-c1"></a>
### walleye-builder: charger c1 built

Entry date
: 2026-11-18

Done
: Enabled pmi8998's charger and a simple-battery (2700 mAh, 4.4 V) in dt/, built the two drivers in, wrote build/build-c1.sh (z3 + new kernel + our dtb, 27 checks, rebuild identical), tests/host/charger-check.sh with its selftest, and one reader step that saves each power supply's type/online/voltage; queued the try.

Considered
: Building c1 on z3, which still waits for its own try (the silent shell): the charger try reads the same way, so both wait on the banner init and reader work.

Rejected
: Modules for the charger (the initramfs carries none); a fuel-gauge node now (capacity stays not known yet); changing the PMIC's JEITA bounds.

About
: walleye-charger-smb2

---


<a id="2026-11-18-walleye-builder-banner"></a>
### walleye-builder: the banner image

Entry date
: 2026-11-18

Done
: Built boot-banner.img (y2 + init-banner), build/build-banner.sh with 40 checks, tests/host/banner-check.sh and banner-selftest.sh; queued walleye-try-banner for the keeper; marked walleye-build-banner-init done.

Considered
: Building on z3 instead of y2 (the item says y2: one change per image, and the banner tells reader from phone first).

Rejected
: A shell on ttyGS0 in the banner image: the banner must not depend on anything reading it. A held-open tty fd: a plain open-write-close per line is simpler and survives the host reopening the port.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1012-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: banner try once from b: banner seen (root read); phone back as Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: re-read the earlier empty tries as a permission problem; redo the dtb gpio-reserved-ranges try with a reader that can open ttyACM0 (root-exec, or mll into uucp / a udev rule, the author's host change)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-reader-eacces"></a>
### walleye-builder: reader retries denied tty, root-exec fallback

Entry date
: 2026-11-18

Done
: mainline-shell-read.sh retries the tty open while Permission denied (ACCESS_WAIT 20 s, 0.5 s steps); on lasting EACCES only, falls back to lab-agent root-exec for that one resolved /dev/ttyACM* node (stty, one-command writes, reads), prints the path used, logs each root use in OUT/root-exec.log. Selftest 21 ok (9 fail on old reader). Programmer review applied: positional node arg, EACCES-only, resolved-node check, timeouts, same stty on both paths, uid-0 guard.

Considered
: root cat in background as reader (tighter, not done)

Rejected
: chmod/chown/usermod, rule change: lab-prepare stays the author's; block readers unchanged: they skip on -r, not the same open

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-banner-serial-fix-built"></a>
### banner2 built: the serial fix

Entry date
: 2026-11-18

Done
: init-banner serial walleye-banner -> walleye-first-boot; build/check-init-serials.sh (every init, no unread form skipped); build-banner.sh runs it and takes BANNER_NAME; boot-banner2.img built twice identical, 41 checks; programmer review applied; walleye-banner-serial-fix done, walleye-try-banner2 queued for the keeper.

Considered
: check inside build-banner.sh only for init-banner

Rejected
: a name-guess grep over serial forms: it silently skipped gst/unquoted forms; now any unread serialnumber line fails

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1030-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: banner2 tried once from slot b: banner read as lab user, proof passes. Phone back as Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: z3 and c1 tries with the reader (our dtb plus gpio-reserved-ranges) twice each

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1120-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z3 tried twice from slot b (fastboot boot from memory, nothing flashed): both times the image came up as USB 1d6b:0104 at 18 s, but mainline-shell-read.sh got Permission denied opening /dev/ttyACM0 and saved an empty dmesg, so nothing is learned about the dmesg. Inside this keeper session (sandboxed Bash) a plain open of /dev/ttyACM0 is denied although getfacl shows user:mll:rw-; probably the sandbox, not the udev ACL. Phone is back as Android on slot b, stock kernel, both slots good, battery 4357 mV

Considered
: next: run the z3 reader with the sandbox off (needs the author's or coordinator's ok), or from a terminal tab of the user; walleye-try-z3 stays open. Not done: no third try, no root-exec (withdrawn)

---


<a id="2026-11-18-1135-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z3 try 1 booted once from b (gadget at 18 s); reader job never picked up by lab-agent, so no dmesg; phone Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader

Considered
: next: the coordinator fixes why submitted jobs are not picked up (agent running, jobs not seen), then z3 twice, c1 twice; z3-try stays open

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1140-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z3 booted once from slot b: gadget up at 18 s, back in Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader; reader denied /dev/ttyACM0 so no dmesg

Considered
: next: make /dev/ttyACM0 readable for mll (udev rule/group; host change, coordinator's), fresh reader, then z3 twice and c1 twice

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1148-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-keeper took walleye-try-z3 and booted z3 once from slot b (fastboot boot, nothing flashed) while the coordinator's tty watcher ran: the node was rw for mll all along; the reader saw silence; the Permission denied was the final write to the vanished node. Phone back as Android on b via adb, stock kernel, flags b/a successful, fastboot reachable via adb reboot bootloader. Item left open with the finding.

Considered
: next: read what z3's ramdisk init does on ttyGS0 (banner, shell, another tty) before any further try; a plain cat of the node during a try would show whether anything is sent. Earlier next (dtb plus gpio-reserved-ranges, twice) still stands.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-reader-drop-root-exec"></a>
### walleye-builder: reader drops the root-exec fallback

Entry date
: 2026-11-18

Done
: mainline-shell-read.sh no longer calls lab-agent root-exec (grep finds none in tests/host and build); the 20 s retry on Permission denied stays; a still-denied tty now fails with owner, group, ACL, the usb parent's serial and who holds the port. Selftest: root-exec cases replaced by a still-denied case (fails cleanly, lab-agent never called, nothing written to the phone); 19 checks ok, seen failing first.

Considered
: keeping root-exec as a logged last resort

Rejected
: root-exec fallback: the no-root decision stands (2026-11-18-walleye-banner-serial-not-root-exec), the cause was the image's serial

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-z4-the-acm-shell-that-answers"></a>
### z4: the ACM shell that answers

Entry date
: 2026-11-18

Done
: init-z4, build-z4.sh, one-fd reader, hang-up fake in the selftest, README; programmer review fixed three bugs; boot-z4.img 4115db3e2c678919cb3d3bd725d5065bbbbac1e6026732989c1eca3590043b20; try queued

Considered
: cttyhack, setsid -c

Rejected
: cttyhack (review)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1155-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader, slots a/b successful; z4 tried once: gadget up at 18 s but the ACM shell stayed silent (dmesg empty)

Considered
: next: z4's console block not seen on the phone; read the kmsg counter / pstore as root-less route, or decide vector with coordinator; item walleye-try-z4 left open

---


<a id="2026-11-18-z5-banner-reports-shell"></a>
### z5: the banner reports the shell's state

Entry date
: 2026-11-18

Done
: Built z5 (init-z5, build-z5.sh, banner-shell-check and two selftests), reviewed by the programmer agent, fixed its findings, queued walleye-try-z5.

Considered
: Redirecting the shell's own stderr to a file (loses the prompt and errors on the tty); a probe start whose output goes to a file instead

Rejected
: Having the banner run stty itself each beat: extra tty ioctls on the banner path; the console block records the tty flags once after its own stty

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0800-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z5 tried once from slot b (research walleye-z5-tried): shell alive, tty full of prompts and errors; phone back as Android on b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader, battery 4376 mV

Considered
: next: z6 = z5 with the tty set raw/no-echo before the shell starts (hypothesis: the banner echoes into the shell); the earlier x1 gpio-reserved-ranges item stays open

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1325-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z5b tried once from slot b: gadget up at 18 s; reader job never ran (lab-agent GitHub token 401). Phone is Android on slot b, stock kernel, nothing flashed, fastboot reachable via adb reboot bootloader, battery 4376 mV

Considered
: next: author renews lab-agent token (lab-agent connect), then z5b again with the same job script; the x1 gpio-reserved-ranges item stays open

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-z6-command-loop-built"></a>
### z6: a command loop instead of the silent ash

Entry date
: 2026-11-18

Done
: Built init-z6 (z5 with the console block replaced by a read/exec loop on one fd), build-z6.sh, the pty selftest z6-loop-selftest, banner-loop-check and its selftest; the reader leaves banner lines out. Reviewed by the programmer agent. Queued walleye-try-z6.

Considered
: Reading with < /dev/ttyGS0 per line as the item sketched; an end marker printed by the loop.

Rejected
: Reopening the tty per line (bytes arriving between closes can be lost, and a close with hangup could end the session): the port is opened once, as z4 decided. An own end marker (__END_n__): the reader's command already carries its marker, a second one would only add a line to each answer.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0900-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z6 tried once from slot b (research walleye-z6-tried): FIRST working mainline shell - commands answer, dmesg 590 lines (Linux 7.3.0), SPMI pm8998/pmi8998 devices 0-00..0-03 present, pmic-dmesg-check passes; no drm, no power_supply class. Phone is Android on slot b, stock kernel, nothing flashed, flags a and b successful, fastboot reachable via adb reboot bootloader, battery 4381 mV.

Considered
: next: read the 14 deferred/fail lines of the z6 dmesg (research/.../2026-11-19-z6-try1/read/dmesg.txt) and the PMIC children (charger, fuel gauge); the shell makes dmesg-driven bring-up possible: next item by the bring-up order (PMIC, clean probes, charger, UFS read-only). The earlier x1 gpio-reserved-ranges item is answered by y/y2 (in our dtb). Procedure that worked: stty loop + banner capture + one-fd reader as plain background processes started before fastboot boot now IMAGE.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-research-z6-dmesg-map"></a>
### walleye-research: z6 dmesg map

Entry date
: 2026-11-18

Done
: Read the z6 dmesg, grouped five faults, recorded research walleye-z6-dmesg-mapped, decision 2026-11-18-walleye-z6-dmesg-fix-order, reranked dmesg-clean/try-c1/ufs-readonly, queued dtbo-overlay-id (PR 218).

Considered
: fw_devlink=permissive; powering the GFX rail.

Rejected
: Both: hide or premature.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-z7-dmesg-clean-built"></a>
### z7: the GPU smmu off

Entry date
: 2026-11-18

Done
: Built z7 (build-z7.sh), the dts line, adreno-smmu-disabled check and selftest, dmesg-clean-check's gpu_gx count and known-overlay tolerance. Queued walleye-try-z7.

Considered
: Building z7 from the dts (would also bring the charger node, a second change).

Rejected
: A second change in the same image; one change per image keeps a try readable.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1000-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z7 tried once from slot b: dmesg clean of the gpu_gx WARN and smmu deferral (323 lines). Phone is Android on slot b, stock kernel, nothing flashed, flags a and b successful, fastboot reachable via adb reboot bootloader, battery 4385 mV.

Considered
: next: per decision 2026-11-18-walleye-z6-dmesg-fix-order - c1 on the z7 init with debugfs and facts, then UFS read-only, USB net, touch, display; the psci PC-mode and misaligned-image Firmware Bug lines remain (vendor-overlay noise left)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-c2-built"></a>
### c2 built

Entry date
: 2026-11-18

Done
: build/build-c2.sh: z7 + the charger kernel + the charger/battery dtb; images-c2 hashed; try queued, try-c1 closed as superseded

Considered
: re-applying the c1 diff by hand on z7's sources: the dts already carries both the charger and the smmu line, so the dtb is a plain build of the dts

Rejected
: a dtb diff against z7 by line count: z7's dtb is an older dts's and phandles shift; the check normalises values and compares the sorted lines

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: c2 tried once from slot b: command loop ok, charger deferred on tps61099 supplier (no driver), no power_supply class. Phone is Android on slot b, stock kernel, nothing flashed, flags a and b successful, fastboot reachable via adb reboot bootloader, battery 4376 mV.

Considered
: next: item walleye-charger-wait-tps61099 (dts choice: drop the supply link, stub tps61099, or fw_devlink=permissive) to the walleye coordinator; then c3; else the z7 order (UFS read-only, USB net, touch, display)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-z8-built-c2-plus-pmic-watchdog"></a>
### z8 built: c2 plus PMIC watchdog

Entry date
: 2026-11-18

Done
: Built kernel (v7.3 + e123d3bff, PM8916_WATCHDOG=y, dts pm8998_wdt okay wakeup-source), build/build-z8.sh with all checks ok (rebuilt twice, same hashes), programmer review applied. boot-z8.img sha256 797778f27d7aa0c8e93cbe6d101678132da3f94bc5536c672f4f740767dce0b5, dtb 1cb054698d37202a64c0408975023374ee429605cd08b04727f491a62c43946b, in ~/Projects/walleye-build/images-z8/. Not tried on the phone.

Considered
: tying Image.gz to the commit: only the compatible string is checked; not known yet beyond that.

About
: deep-sleep

---


<a id="2026-11-18-1500-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Took the lease; phone is Android on slot b, stock kernel, adb up (FA81N1A00790), nothing changed this run. phone-next offered only walleye-debug-without-display, which is blocked_by walleye-first-fastboot-boot, has no built image or test, and the pstore chase is paused by decision 2026-11-18-walleye-vendor-kernel-as-instrument; unattended, so not started. lpoco lab suite latest fails: no devlab password file for google-walleye (needs PHONE_PASS or ~/.config/devlab/google-walleye/password).

Considered
: next: walleye-charger-wait-tps61099 (needs research; decision 2026-11-18-walleye-charger-supplier-permissive-then-modelled already says permissive devlink first, then a modelled fixed regulator) - build c3 with fw_devlink=permissive, then a try on slot b; the queue's phone item should be that try once c3 is built. Also: phone-next lists a blocked item; its blocked_by is not honoured by the query.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-c3-built"></a>
### c3 built

Entry date
: 2026-11-18

Done
: build/build-c3.sh (failed first on a missing input), built through lpoco-lab job start, log read; try queued as walleye-try-c3; walleye-charger-supplier stays open for c4.

Considered
: lpoco-bringup image swap --cmdline (would work; c2's own script pattern kept for its AVB salt/props checks)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1510-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: PMIC watchdog: z8 had none (qcom-pon =m); z9 (=y) bit unpetted, gone 20 s after arming (pmic-wdt-bites pass, landed PR 236). Phone is OFF/not on USB since 14:54 after the bite: not Android, not fastboot - needs the author's hands (hold power ~15 s). Slots untouched (b current, a and b successful as of 14:49). Nothing flashed.

Considered
: next: author power cycle (item walleye-needs-power-cycle-after-pmic-bite), then check fastboot; lane coordinator to decide the PMIC bite's reset type before another bite; c3 try (c2 + fw_devlink=permissive, PR 235) still queued; wake-in-suspend waits for walleye to suspend at all

---


<a id="2026-11-18-1530-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: c3 tried once from slot b: charger probes with fw_devlink=permissive (charger-check pass). Walleye is HUNG in c3 (silent, USB 1d6b:0104, no adb/fastboot) since 15:13; may recover slowly as after z9, else needs the author's power hold. Nothing flashed, slot b active.

Considered
: next: item walleye-c3-hung-wait-then-power-cycle first; then c4 (tps61099 fixed regulator, walleye-charger-supplier). PMIC bite proof stays barred by decision 2026-11-18-walleye-no-pmic-bite-until-reset-type

---


<a id="2026-11-18-1535-c4-built"></a>
### c4 built

Entry date
: 2026-11-18

Done
: walleye-builder: read c3's try (charger probes with fw_devlink=permissive), read the tps61099 facts from the vfa fdt and the dtbo, modelled it in our dts, built boot-c4.img through lpoco-lab job start (log read from the journal; 26 checks ok, rebuilt identical), programmer review, queued walleye-try-c4.

Considered
: A regulator-fixed at the overlay's node; the build in the v7.3+wdt tree (it would add z8's watchdog to the dtb).

Rejected
: Both - see decision 2026-11-18-walleye-tps61099-fixed-regulator-supplier-disabled.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-c5-and-c6-built"></a>
### walleye builder: c5 and c6 built

Entry date
: 2026-11-18

Done
: Built c5/c6 as one image (charger as a module, loaded late by command line): build-c5.sh, job-c5.sh, init-c5, c5-loader-selftest; 40 checks ok, rebuild identical, programmer review fixed.

Considered
: Building c5 and c6 as two inits; one image keeps them equal but the command line.

Rejected
: Cap of 150 s for the delay: the 180 s self-reboot could race a late bind; 120 s used.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-phone-state-check-over-several-witnesses"></a>
### Phone-state check over several witnesses

Entry date
: 2026-11-18

Done
: Built tests/host/phone-state.sh: verdict android|fastboot|mainline-answering|mainline-silent|not-known|gone over adb, fastboot, sysfs usb, the gadget's own tty; never 'hung'. Selftest with fakes, programmer review fixed (tty echo/hupcl, tty node bound to the gadget, adb recovery states, age dropped). AGENTS.md rule added.

Considered
: Age of the listing from sysfs mtime

Rejected
: Age witness: sysfs mtimes are lazy and say nothing of enumeration time, so it is shown as not known yet; usb reset probe left out of the first version as the item says

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1610-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Took walleye-try-c5: tried once from slot b, restart gate passed (180 s). Phone is Android on slot b, stock kernel, nothing flashed, flags a and b successful, fastboot reachable by adb reboot bootloader, battery 4378 mV.

Considered
: next: walleye-try-c6 (charger module loaded 40 s after enumeration) - the gate passed, so it may run; it needs a PMIC-side deadman decision first if its try is longer than 180 s (decision restart-gate-before-charger-try); else walleye-dtbo-minimal-try

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-wire-phone-state"></a>
### Wire phone-state into restart-check and readers

Entry date
: 2026-11-18

Done
: restart-check.sh consults phone-state.sh when the journal has no disconnect; five readers print its verdict not 'hung'; selftest 11 ok

Considered
: consult by default without NOW

Rejected
: consulting phone-state against a recorded journal: today's phone proves nothing about it (programmer review)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-google-walleye-lab-follows-docs-device-layer"></a>
### google-walleye-lab follows docs' device layer

Entry date
: 2026-11-18

Done
: Copied device-template #26: devicebase.toml pin (docs 0.17.0), device:begin/end markers around the device tables, generic list removed, idealkb.toml why text, AGENTS.md line. follow.sh against docs v0.17.0 found the device part and every device statement already identical, so nothing else moved.

Considered
: Running follow.sh first with no markers: it would append a second copy of the device tables at the file's end.

Rejected
: Keeping our own generic list: docs' device-layer/ is the one owner.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-1840-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Phone is Android on slot b, stock kernel, nothing flashed, adb up. Took walleye-pmic-wdt-read-s2-regs: wrote tests/host/pmic-wdt-s2-regs-read.sh (read only), booted z9 from memory via lpoco lab fastboot boot now; it answered (1d6b:0104, uname 7.3.0-00004-ge123d3bff47d-dirty; spmi regmaps 0-00..0-03 named pmic-spmi under debugfs). First run: no PON register lines read. Second run (NOREBOOT) answers came back as blank lines only. Registers NOT yet read; item stays open. Phone left the mainline image by itself after about 15 min (off USB for part), back on Android b.

Considered
: next: debug the tty reader (answers blank; compare with mainline-shell-read.sh which worked; check the regmap registers format); retry z9; walleye-pmic-wdt-readback-image is the fallback; dtbo item stays held

About
: deep-sleep

---


<a id="2026-11-18-walleye-builder-z10"></a>
### walleye-builder: z10

Entry date
: 2026-11-18

Done
: Built z10 from z9 (build/build-z10.sh, 27 checks), init-z10 with /bin/pon-regs, the selftest and the phone test pmic-wdt-pon-regs-banner; programmer review applied; queued the keeper's try.

Considered
: A kernel change (S2 type by dt property with read-back) as the backlog first said: needs a kernel build and a push of the series, and the registers must be read before any type is chosen.

Rejected
: Kernel route now: the coordinator's order was the registers in the banner, reads only; the property comes after the values are known.

About
: deep-sleep

---


<a id="2026-11-18-1915-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Phone is Android on slot b, stock kernel, adb up, nothing flashed, flags unchanged. z10 tried once from memory: no PON register values (banner 'not read yet'); item walleye-pmic-wdt-z10-read-pon-regs stays open.

Considered
: next: why init-z10 never read the registers; walleye-pmic-wdt-read-s2-regs; walleye-dtbo-minimal-try question to coordinator still open

About
: deep-sleep

---


<a id="2026-11-19-walleye-builder-z11"></a>
### walleye-builder: z11, UFS read-only

Entry date
: 2026-11-18

Done
: Took walleye-ufs-readonly (first ready row). dts: l20/l26 rails, ufshc and ufsphy okay; config: UFS host and QMP UFS phy =y; init-z11 (z10 + ufs-lsblk, ufs-hdr, WALLEYE-BANNER-UFS); build-z11.sh with its checks; ufs-lsblk-selftest.sh and ufs-lsblk-check.sh (test walleye-ufs-lsblk). Built images-z11/boot-z11.img b4399ea3...; queued walleye-try-z11 for the keeper.

Considered
: Bringing the dtb change as a z9-style dtb-only image (rejected: the UFS host is =m in z9's kernel, so the kernel must change too).

Rejected
: Anything that writes or mounts: not in this image.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-z12"></a>
### walleye-builder: z12, Hynix UFS quirks

Entry date
: 2026-11-18

Done
: Took walleye-ufs-hynix-quirks. Kernel topic/ufs-hynix-quirks 12dca401a; build-z12.sh (15 checks); z12 sha256 f959c448 built, not run.

Considered
: Sony's pre-link-startup lane limit via Kconfig

Rejected
: Model-specific tables: the card's model is unknown

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2330-how-the-phone-stands-ufs"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, adb up. Item walleye-ufs-card-identity closed on the getvar-none branch (one fastboot visit, rebooted back).

Considered
: next: coordinator to say whether the exact UFS model needs a recovery root shell, since z11/z12 wait on it; walleye-pmic-wdt-z10-read-pon-regs is parked. Note: the earlier 2300 entry says the phone was off USB; it is back on Android now.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-z13"></a>
### Hynix quirks review fixes, z13 built

Entry date
: 2026-11-18

Done
: Fixed findings 1,4,5,6 of walleye-z12-built; a second programmer review caught that the raw-path guard keyed on msgcode was bypassable, moved it to ufshcd_issue_devman_upiu_cmd by transaction code; z13 built, checks ok.

Considered
: Propagating DME write errors (would fail the UFS probe)

Rejected
: Keeping the PA_AVAILTXDATALANES write: read-only per spec, would warn every boot. Guarding in ufs_bsg alone: bypassable.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Phone is Android (LineageOS userdebug) on slot b, stock kernel, nothing flashed or written; adb root was enabled (adbd restarted as root, disposable per decision 2026-11-18-walleye-android-disposable-no-bricking). Item walleye-ufs-card-identity-root done: UFS is SK hynix H28U74301AMR fw D003 64 GB (research walleye-ufs-card-identity-root).

Considered
: next: coordinator takes the UFS identity to the top coordinator for the UFS-probe decision; then walleye-pmic-wdt-read-s2-regs (read only); walleye-charger-wait-tps61099 dts choice; walleye-pmic-wdt-z10 stays parked

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-z14"></a>
### walleye builder: z14 USB networking built

Entry date
: 2026-11-18

Done
: Built z14 (z7 + ECM/usb0/udhcpd/telnetd in the init), 26 checks, rebuild identical; queued walleye-try-z14 for the keeper; walleye-usb-net in progress until the try

Considered
: Dropbear static for ssh (no key on the lab host, no sshd in earlier images); NCM (panicked on the vendor kernel; ECM is in this kernel and is what the first images used)

Rejected
: ssh in the initramfs now: needs a key and a second build; telnet on the private cable is what gadget-up.sh proves

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-steps-claimed"></a>
### Walleye's steps claimed on the bring-up chain

Entry date
: 2026-11-18

Done
: Added steps = 1 to devicebase.toml; wrote build/claim-steps.sh (ikb step per step) and ran it: 113 claims (17 done, 3 partial, 1 failing, 92 open), each draft claim of docs checked against this base's research, backlog and decisions. Corrected two: charger-online partial (c3 online 1, restart gate open), watchdog-way-back failing (z9 bite, no way back). In a scratch copy of docs the feed reads them, bringup-chain-walleye.sh passes, daily-phone.sh google-walleye fails honestly (66 of 82 not done). Programmer review: no blocking problem; fixed the claim date (re-run identical) and the charger note. Open: research row walleye-c3-tried says z9 came back after 10 min, the z9 row says not - the c3 row's wording is for the keeper to correct.

Rejected
: unknown for untried steps (the chain test refuses it); copying the draft whole (two claims contradicted the records)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1400-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-try-z14 done: z14 once from slot b, USB networking works (172.16.43.1 ping + telnet root shell); gadget-up.sh falls back to telnet when host lacks nc. Phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable.

Considered
: next: walleye-ufs-android-live / walleye-bootchain-readback (read only); z14 gives a network shell for probe reading

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2330-how-the-phone-stands-ufs-readback"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-ufs-android-live done: read-only adb root readback of Android's UFS bring-up recorded. Phone: Android on slot b, stock kernel, nothing flashed, fastboot reachable.

Considered
: next: walleye-bootchain-readback (read only, needs a test written first)

---


<a id="2026-11-19-1600-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: walleye-try-z15 part 1: z15 once from slot b over the network shell; touch probe absent because i2c@c179000 keeps the downstream compatible qcom,i2c-msm-v2. Phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable, slots a and b successful.

Considered
: next: walleye-build-z16-i2c-compatible (builder); z15 part 2 waits for the author; then walleye-ufs-android-live / walleye-bootchain-readback (read only)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-z16"></a>
### walleye-builder: z16 built

Entry date
: 2026-11-18

Done
: Read z15 part 1; found the overlay overwrites the qup node by name; wrote tests/host/i2c-qup-after-merge.sh (fails on z15, passes on z16) and build/build-z16.sh (26 checks, rebuild identical); recorded research, decision, the phone try queued.

Considered
: compatible edit, dtbo edit, soc symbol repoint, dtc round trip

Rejected
: all four, see the decision

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1700-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: z16 part 1 once from slot b: i2c bus and rmi4_i2c bind, probe fails -5 (controller silent). Phone back as Android on slot b, stock kernel, nothing flashed, fastboot reachable, slots a and b successful.

Considered
: next: walleye-build-z17 touch power/reset (builder): check vdd/vio supplies, reset gpio89, delay against vendor tree; z15/z16 part 2 waits for the author; then walleye-ufs-android-live / walleye-bootchain-readback (read only)

---


<a id="2026-11-18-walleye-research-ufs-android-way"></a>
### walleye-research: UFS the Android way

Entry date
: 2026-11-18

Done
: Compared Google's wahoo 4.4 UFS/PHY/dts with z13 and the live readings by extracted lists (PHY tables, quirks, dts props) and by hand (reset, power-up, link startup). Finding: Android resets the card and uses one quirk and 2 lanes; z13 does neither. Queued walleye-ufs-android-regs (phone, read-only) and walleye-ufs-match-android-z (build).

Considered
: Reading msm-4.4 against v7.3 line by line (the author said no: diverged too far); the Lineage fork's UFS files (not read: Google's stock branch is the pairing with the stock bootloader).

Rejected
: Claiming a match or a safe probe: abl's hand-over, the HW version and the erase cause are unread.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1800-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, fastboot reachable; read UFS host regs read-only on live Android (research walleye-ufs-android-regs-read); HW_VERSION and PHY unreadable, no /dev/mem

Considered
: next: walleye-bootchain-readback (read only, rank 4); a reader for 0x1da7000 PHY; walleye-build-z17 touch power (builder)

---


<a id="2026-11-18-walleye-builder-z17"></a>
### walleye-builder: z17 built

Entry date
: 2026-11-18

Done
: Read z16's probe -5 and the phone's own dtbo overlay: the touch power switch is pmi8998 gpio 2, never driven by z16. Wrote tests/host/touch-power-after-merge.sh (fails on z16, passes on z17) and build/build-z17.sh (rebuild identical), the same regulator in the dts; programmer review fixed; research walleye-z17-built, try walleye-try-z17-part1 queued.

Considered
: Rebuilding the dtb from the dts: the kernel tree clone is stale; z16 was a dtb edit too, so z17 edits z16's dtb with fdtput. Driving gpio 75: its job is not known yet.

Rejected
: A gpio hog instead of a regulator: the supply belongs to the touch node's vdd so the driver's own order and delay apply.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1900-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, fastboot reachable, slots a/b successful. Tried z17 once: touch still silent, rmi4 defers on regulator-touch-power because pmi8998@3 lacks reg (research walleye-z17-part1-tried)

Considered
: next: build z18 (pmi8998@3 reg, builder); walleye-ufs-android-regs and walleye-bootchain-readback remain

---


<a id="2026-11-18-walleye-builder-z18-stopped"></a>
### walleye-builder: z18 not built, its premise is false

Entry date
: 2026-11-18

Done
: Read z17's dtb, the overlay merge, the dmesg and v7.3 spmi.c: pmi8998@3 of ours has reg; the reg-less lines are the overlay's nodes and are skipped. Recorded research walleye-z18-premise-falsified and a decision; the item stays open with the next step.

Considered
: Building z18 as the item says; a facts-reading init on z17's image next.

Rejected
: A no-effect build and a phone try.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-2130-how-the-phone-stands-z17-facts"></a>
### How the phone stands (after z17 facts)

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed, a/b successful, fastboot reachable. z17 facts read once (no write): pmi8998 gpiochip2 exists, touch regulator still 'can't get GPIO'

Considered
: next: why gpiod_get fails on an existing chip (full dmesg order: pmic@2 gpio probe vs regulator-fixed); walleye-ufs-android-regs needs a reader decision; bootchain-readback remains

About
: walleye-first-fastboot-boot

---


<a id="2026-11-18-walleye-builder-overlay-collisions"></a>
### walleye-builder: overlay collisions test, fix, z19

Entry date
: 2026-11-18

Done
: Wrote the collision test and its selftest, found pcie0 (and the dts lacking z16's rename), fixed both in the dts, built z19 from z17 (rebuild identical), CI selftest job, build-all gate; programmer review fixed the gate's path.

Considered
: Comparing fragments by hand (fdtoverlay plus a flat diff is the same by-name merge); the phone's own dtbo (needs adb root; a builder never touches the phone).

Rejected
: Allow-listing pcie.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-research-touch-gpio"></a>
### walleye-research: touch gpio defer

Entry date
: 2026-11-18

Done
: Read v7.3 gpiolib, pinctrl and spmi-gpio source against the built z17 dtb: the specifier is valid (tests/host/touch-gpio-specifier.sh passes), the failure is a persistent -EPROBE_DEFER at an unnamed leaf; queued a read-only phone read and a dyndbg diagnostic build.

Considered
: dts hog, pinctrl-0 node, label rename, replacing the regulator, unbind/bind re-probe

Rejected
: all guesses or writes: see decision 2026-11-19-walleye-touch-gpio-diag-first

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2000-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed or booted; read UFS via debugfs (read only); fastboot reachable per last stand, adb root on

Considered
: next: walleye-try-z19, then walleye-bootchain-readback; UFS HW_VERSION/PHY need a reader (coordinator)

---


<a id="2026-11-19-walleye-builder-gpio-diag"></a>
### walleye-builder: gpio diag kernel, z20

Entry date
: 2026-11-18

Done
: Built z20: z19 with a dynamic-debug and DEBUG_GPIO kernel (EXTRA_CONFIG hook in build-kernel.sh, build/walleye-diag.config), command line plus dyndbg by full kernel paths; test first (fails on z19), checks ok, rebuild identical; the programmer review found the short path pinctrl/core.c matching nothing - fixed, test made exact; queued walleye-try-z20 for the keeper.

Considered
: drivers/base/dd.c +p (every deferral of every device: too loud); module-style short file names

Rejected
: A debugfs leaf read first (the coordinator's fallback, redundant once the diag kernel prints the gpio errno); a new kernel branch for the config (the tree is z15's, only a config fragment differs).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed; z20 tried once and done (touch gpio leaf = shared-gpio proxy, research walleye-z20-tried-touch-gpio-leaf); a and b successful, none unbootable

Considered
: next: walleye-touch-gpio-shared-proxy (build), then walleye-try-z19 and walleye-bootchain-readback; UFS HW_VERSION/PHY still need a reader (coordinator)

---


<a id="2026-11-18-walleye-part-compare-tool-built"></a>
### walleye: part-compare tool built

Entry date
: 2026-11-18

Done
: Built tests/host/part-compare.sh with per-part lists (touch first), selftest on fake captures and transports; programmer review fixed 10 defects before landing; no phone touched.

Considered
: One telnet session per command (kept: simple, same as touch-read.sh); a dt phandle resolver (left as a documented limit).

Rejected
: grep -P and bash-only constructs: host and phone shells are dash/busybox/toybox; Python: forbidden.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-18

Done
: Android on slot b, stock kernel, nothing flashed; z20 tried a second time for the touch compare (fell back by itself at 180 s; a and b successful, none unbootable); touch-android-compare done (research walleye-touch-android-compare-result)

Considered
: next: walleye-touch-gpio-shared-proxy (build) then add the synaptics node (tlmm 125/89/75, pmi gpio 2); walleye-ufs-android-regs still needs a mem reader (coordinator); walleye-try-z19 and walleye-bootchain-readback remain

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-z21"></a>
### walleye-builder: one claimant for the touch gpio

Entry date
: 2026-11-19

Done
: Found the second claimant of pmi8998 gpio 2 (overlay synaptics_dsx@20 under our stub i2c_5); test first (fails 2 on z19/z20 dtbs); stubs container disabled in generator and dtsi; a UFS-on z21 repack built, then withdrawn (NOT-TO-BOOT) when main's UFS gate arrived; no phone try queued; programmer review applied.

Considered
: Queuing the UFS-on z21; folding the UFS gate into this change

Rejected
: Booting any UFS-on image; the gate is its own item (walleye-ufs-off-gate).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-ufs-off-gate"></a>
### walleye-builder: UFS off by a gate, then z21

Entry date
: 2026-11-19

Done
: dts: ufshc/ufsphy disabled, variant dts for z11-z13; gate tests/host/ufs-off-image.sh (every appended dtb, alone and merged with dtbo entry 3, plus command line incl. additional args) with selftest and CI job; build/ufs-gate.sh wired into 56 build scripts and build-all; inventory and NOT-TO-BOOT marks on 58 images; z21 rebuilt UFS-off, gate fails z20 and passes z21; programmer review fixes: extra_cmdline, merge failure fails, unit-address match, image removed on failure.

Considered
: Gate on stock-kernel repacks (vendor-gadget, vendor-facts, bisect s0): they fail by design, they need a UFS-off dtb before they build again - not done here.

Rejected
: Matching empty vendor stub nodes named ufs* (they are not devices: first try made 96 images ON); renaming the images directories (scripts refer to them), markers beside the images instead.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, 4381 mV; z21 tried once and done: touch binds (rmi4 0-0020, Synaptics S3708AR input1), no gpiolib_shared deferral

Considered
: next: touch events need a touch (touch-read.sh with a person) and part-compare capture touch ours (z21 again with NOREBOOT=1); walleye-ufs-android-regs still needs a mem reader (coordinator); then walleye-try-z19/bootchain-readback

---


<a id="2026-11-19-2230-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, 4379 mV; z21 tried a second time for the touch part-compare capture, captured and diffed

Considered
: next: read the gpio section of compare-touch-diff-z21.txt (pmi8998 gpio 2 on ours vs Android) ; walleye-ufs-android-regs still needs a mem reader (coordinator); then z19, bootchain-readback

---


<a id="2026-11-19-licences-for-google-walleye-lab"></a>
### Licences for google-walleye-lab

Entry date
: 2026-11-19

Done
: Copied device-template #33's model; annotated per path; SPDX dual licence on our own dts and the stub generator; licence-check fails on main (no REUSE.toml), passes on the branch; programmer review found the C probe relicensed against its own GPL header - fixed.

Considered
: build-pk.sh's generated dts wrap porthole's board files: left GPL-2.0-only.

Rejected
: Writing our own coverage check: orchestration's order is the shared script.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0235-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Nothing changed. Android on slot b, stock kernel, nothing flashed (as the 22:30 hand-over). Keeper run 02:32 took no item: walleye-try-z21-touch-events needs a person at the phone and the run is unattended; lpoco lab cannot reach the phone: ~/.config/devlab/google-walleye/password is missing (no google-walleye dir at all). walleye-ufs-android-regs still waits on a mem reader.

Considered
: next: coordinator provides the walleye password file (mode 600) and, if the author is at the phone, a fresh go for z21 touch events; then z19, bootchain-readback

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-night-pause"></a>
### walleye lane: paused for the night, touch events still not known

Entry date
: 2026-11-19

Done
: The author paused for the night (relayed by the top coordinator): no new keeper, builder or research session until he or the top coordinator says resume; scheduled runs are disabled; no builder was running. Pixel 2 left on Android slot b, nothing flashed. Touch events on z21: Synaptics S3708AR bound (event1); the author swiped after the 20 s reader had closed; record corrected (research walleye-z21-touch-events-window-missed); go closed; the retry recipe sits in the item walleye-try-z21-touch-events (reader first and 90 s or more, /tmp/hold, ready at once, go of 90 minutes or more).

Considered
: Briefing a keeper directly as a subagent for the retry: held, since the author is away.

Rejected
: Reopening the go tonight: the author is not at the phone.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display"></a>
### walleye-builder: the display driver compiled from the phone's own panel facts, image built

Entry date
: 2026-11-19

Done
: Took the unlanded display branch of earlier runs; it did not compile (reset and on functions deleted). Read the S6E3FA5 node of the phone's dtbo and restored them with its sequences; built Image.gz and the display dtb, then boot-display.img on z21's ramdisk; every check ok incl. the UFS-off gate. Queued walleye-try-display (witness: person). Not lit: untried.

Rejected
: Lighting the panel with the OnePlus 5 init: not the Pixel 2's own.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-pmic-wdt-v1-images"></a>
### walleye-builder: the PMIC watchdog v1 images, A on next and B on v7.3

Entry date
: 2026-11-19

Done
: Rebuilt the upstreams lane's next tree from the tag next-20261118 plus the six patches (clean), built kernel A there and kernel B on v7.3 + topic 8b2283da3, wrote init-wdt1 (unarmed at boot; wdt-state, wdt-bite, wdt-reboot; SoC petter by identity; banner lines PON and WDT), its host selftest (seen failing on init-z14, passing on init-wdt1) and build/build-pmic-wdt-v1.sh (81 checks, rebuild identical). Found that the clock_gcc symbol must be fixed after the build and that the display image lacks it (backlog walleye-display-image-clock-gcc). Hashes in the try item and in research walleye-pmic-wdt-v1-images-built. A programmer review found seven issues (arming not read back, the self-reboot racing the bite, a racy selftest, a petter check by text only, and more); all fixed, the selftest now catches five mutations of the init, and the images were rebuilt (87 checks ok, rebuild identical).

Considered
: Waiting for the lane's own next tree (not on this machine); arming from the init after a delay; a dts property for the reset type (the driver always sets hard 0x07).

Rejected
: Using /dev/watchdog for the petter (may be the PMIC device, opening arms it). Calling the guard 'the dts type' (there is none).

About
: walleye-pmic-wdt-proof

---


<a id="2026-11-19-display-image-rebuilt-with-the-real-gcc"></a>
### Display image rebuilt with the real gcc

Entry date
: 2026-11-19

Done
: build-display-image.sh now sets /__symbols__/clock_gcc to /soc@0/clock-controller@100000 on the dtb it packs, as build-pmic-wdt-v1.sh does, checked before (the empty stub) and after, and runs overlay-clock-gcc.sh on it; the test was seen failing on the old image's dtb first. New hash b70feabf44a81fc3ad17c9905305b3c8379197d4d824581de2272acec2515cc8 (old 21a1134e retired); walleye-try-display note updated. No phone touched.

Considered
: fixing in build-display.sh (the dtb is built there): the image script already owns the packing and the checks, and the pmic script does it the same way

Rejected
: none

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-layer-declared"></a>
### walleye's device layer declared in device.toml

Entry date
: 2026-11-19

Done
: Added [layer] (name google-walleye, no needs, kernel linux-phone-oco/linux topic/pmic-wdt-wake-in-suspend with build/walleye.config and the walleye dtb, boot android-bootimg, firmware extract-from-partitions) and [boot] (header v0, base 0, kernel 0x8000, ramdisk 0x1000000, tags 0x100, page 4096, 32 MiB partition, msm-id, dtb appended, AVB footer). Checked with lpoco-manifest's own parser (Manifest::parse from lab-tools main 51e5e87, a scratch example): ok; a bad boot method is refused.

Considered
: The cmdline is the plain Android one of the z-line without the diagnostic dyndbg words.

Rejected
: Declaring UCM or udev files: walleye has none, none invented. Naming firmware partition files: the schema holds a method only, so the sources are said in a comment. Declaring [boot.gadget] here: that is walleye-init-from-roof.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-init-gaps"></a>
### walleye-builder: the generated init compared with ours

Entry date
: 2026-11-19

Done
: Declared [boot.gadget] in device.toml; generated the init and files with lpoco-bringup 0.24.0 and compared them with init-z14 and init-wdt1; recorded the gaps (research walleye-generated-init-gaps); sent them to the coordinator for orchestration

Considered
: Switching build/ to the generated init with the missing parts bolted on in walleye's own scripts

Rejected
: Retiring the init variants now: the generated init has no petter, no hold, no raw command loop and no banner, so the phone tries would lose their 180 s self-reboot; item stays open until the roof covers them

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-upstream-dts"></a>
### walleye-builder: the Pixel 2 board dts as an upstream series

Entry date
: 2026-11-19

Done
: Polished the candidate into a two-patch series on v7.3; binding, W=1 dtbs and relaxed style clean, strict 76 to 2, checkpatch 0 errors; programmer review applied in part; nothing mailed.

Considered
: A separate msm8998.dtsi wdt patch

Rejected
: Wdt patch without a bite proof; dual licence

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-upstream-dts-image"></a>
### walleye-builder: image of the exact upstream board dts

Entry date
: 2026-11-19

Done
: Built boot-upstream-dts.img (sha256 a8e4364813f3c82a35b9e5edcffb6650eac710e780119b27fa32dc2fa9521377): z21 with the dtb of the series' dts unedited plus a wrapper with overlay stubs and two deleted nodes; build/build-upstream-dts-image.sh, 31 checks, rebuilt identical, programmer review applied (tree-clean check, FDT_ERR_NOTFOUND check, tool failures stop the build). No phone touched.

Considered
: Editing the series' dts to carry the stubs

Rejected
: Editing the series' dts: the stubs and deletions are the stock bootloader's merge, not upstream's; kept in a wrapper

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-0820-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, 4369 mV; upstream board dts image tried once, booted to the usb0 shell, back on Android

Considered
: next: z19 try, bootchain-readback (read only), c4; read z21 gpio diff; z21 dmesg capture was truncated - recapture if a probe diff is wanted

---


<a id="2026-11-19-walleye-builder-bootchain-readback-scripts"></a>
### walleye-builder: boot-chain read-back reader, check and selftest

Entry date
: 2026-11-19

Done
: Wrote bootchain-readback.sh (read only, refuses without adb device, battery rule, clean DIR outside repositories, then runs its check), bootchain-readback-check.sh (32 names, measured size, sha256, modes, second read equal), bootchain-readback-selftest.sh with a fake adb that logs every call; CI job. Programmer review: five findings fixed (repository below a missing parent, reader trusting a failed su, call log, same DIR twice, echo of phone text).

Considered
: Sizes from the phone's own by-name and /proc/partitions rather than a hard-coded table; xbl 8104 sectors optional because its sector size is not recorded.

Rejected
: Hard-coded sizes: only xbl is measured. Plain adb exec-out dd: the brief names su -c.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2330-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adbd back to shell; bootchain read back (32 partitions, twice, identical sha256) and item walleye-bootchain-readback done

Considered
: next: z21 touch gpio section of compare-touch-diff-z21.txt; walleye-ufs-android-regs needs a mem reader (coordinator); pmic-wdt items stay parked

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-boot-image-align"></a>
### walleye-builder: the misaligned-image line is the bootloader's

Entry date
: 2026-11-19

Done
: Took walleye-boot-image-align (rank 9). Read the z7 dmesg and build-bootimg.sh: our kernel offset 0x8000 is the stock/taimen/LineageOS value, so the warning comes from where the bootloader loads the Image. Item closed as recorded-as-the-bootloader's; no phone touched, no image built. Left for the next builder: ffs-acm retirement, bisect series, fuel gauge, wifi.

Considered
: Whether a 2 MiB-aligned kernel offset is worth one keeper try.

Rejected
: Moving the offset to 0x200000: changes the load address on an unread bootloader and may overlap the ramdisk at 0x81000000, to silence one cosmetic line.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-retire-ffs-acm"></a>
### walleye-builder: the lab's own FunctionFS helpers retired

Entry date
: 2026-11-19

Done
: Removed the C probe, the Rust helper, their build scripts, the n3/n3r image scripts and init-n3, the rust CI job and the GPL-2.0 REUSE annotation for the probe; README licence line now has no exception. Checks pass.

Considered
: Taking the helper from the lpoco-ffs-acm crate for some image: no current image needs one (mainline uses native configfs ACM/NCM/ECM). Keeping tests/host/ffs-acm-up.sh.

Rejected
: Keeping init-n3 without its scripts: nothing builds from it; git history holds it. Removing ffs-acm-up.sh: it is the proof of a done row.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-series-files"></a>
### walleye-builder: the bisects as lpoco-bringup series files

Entry date
: 2026-11-19

Done
: Six series files in build/series; lpoco-bringup 0.24.0 image bisect reproduced 35 of 35 recorded images byte for byte; removed build-bisect.sh, check-bisect.sh, build-pk456.sh; README names the series. Not run on any phone.

Considered
: Leaving the dtb scripts whole

Rejected
: Deleting the dtb scripts: they make the dtbs

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-fuel-gauge"></a>
### walleye-builder: the fuel gauge image

Entry date
: 2026-11-19

Done
: Fetched the RFC, applied patches 1-3 on v7.3 plus the wdt topic, enabled the node, wrote the check and its selftest, built fg1 (37 checks, rebuild identical), dropped the pmi8998_fg overlay stub, queued the keeper's try (walleye-try-fuel-gauge).

Considered
: All eight patches; a table of the vendor profile.

Rejected
: Both, see the decision.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-20-0030-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adbd up; fg1 tried once: fuel gauge pass (capacity 99 %, 4374 mV = fastboot); item walleye-try-fuel-gauge done

Considered
: next: z21 touch gpio section of compare-touch-diff-z21.txt; walleye-ufs-android-regs needs a mem reader (coordinator); pmic-wdt items stay parked; upstream the fuel gauge result to the RFC thread is the author's

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-unattended-night"></a>
### walleye lane: the unattended night, 2026-11-19

Entry date
: 2026-11-19

Done
: Landed with the lane's builders and keepers (lab PRs 357-392): touch record corrected (the author swiped, window missed; retry recipe in the item); PMIC watchdog v1 images A (next) and B (v7.3 + topic 8b2283da3) built and gated, attended try queued with its plain-words ask, type 0x07 as the series sets; display image rebuilt with the clock_gcc fix (try waits for the author); device layer declared in device.toml; own ffs helpers and bisect scripts retired for the shared tools; the exact upstream board dts booted once to the usb0 shell (research walleye-upstream-dts-tried-boots), finding: the series' dtb needs overlay stubs to merge with the phone's dtbo; boot chain read back (32 partitions twice, identical sha256, files outside git); fuel gauge fg1 proven (capacity 99, 4374 mV = fastboot); boot image alignment is the bootloader's; four generated-init gaps written for orchestration (init adoption waits). The phone ended on Android slot b, nothing flashed. Waits for the author: touch retry, display try, PMIC v1 try (reboot test then bite), UFS (his go).

Considered
: Retrying the touch run at once: the author was gone; the retry has its recipe.

Rejected
: Booting any UFS-on image or the old z19/c4 tries: gate and decisions say no.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-research-ufs-reader"></a>
### walleye-research: how to read UFS_HW_VERSION on live Android

Entry date
: 2026-11-19

Done
: Read the vendor wahoo 4.4 UFS source: register is 0xE4 (earlier rows said 0xE0), version is 3.0.0 or 3.1.0 for certain, no read-only path on Android. Decided: no module; the matched image reads it in its own probe. Closed items walleye-ufs-regs-reader and walleye-ufs-android-regs.

Considered
: A read-only module, debugfs reset_controller, vendor kernel as instrument, skipping

Rejected
: module and reset_controller (cost, bus-hang, write)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-ufs-match"></a>
### walleye-builder: the matched UFS image, built and never booted

Entry date
: 2026-11-19

Done
: Built walleye-ufs-match-android-z: kernel topic/ufs-match-android (quirks, PHY tables by host version, RST_n gpio 153), z21 plus a UFS overlay, image NOT-TO-BOOT-ufs-on-ufs-match.img. The programmer review found 7 points; stop-at-init, the leaked tag, the weak tests, the dirty-Image and queue-name points are fixed, the scope points recorded. The kernel branch is local in ~/Projects/walleye-build/wt-k-ufsm (not pushed). Nothing touched a phone; the try is walleye-try-ufs-match, waiting for the author's go.

Considered
: pinctrl states for the reset; a board-file variant dts instead of an overlay (z21's dtb had drifted from the board file)

Rejected
: a full match of Android's path (link-startup pre-change, ICE not carried)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display-bisect-images"></a>
### walleye-builder: the display bisect images d0, d1, d2

Entry date
: 2026-11-19

Done
: Three boot images on z21's boot.img: d0 the display kernel with z21's dtb, d1 mdss and the display controller only, d2 plus DSI and its PHY, no panel node; a series file, a parts script and a count script (21+ checks ok; a z21-kernel negative run fails it), UFS-off gate passes on each, sha256 recorded. A programmer review found the count script too trusting; it now reads the display kernel and the stage dtbs itself. Not run on any phone.

Considered
: Whether the stale-looking dtb in images-display is read: build-display-image.sh reads it as raw input and fixes clock_gcc itself

Rejected
: Rebuilding the dtbs from source: the image's own bytes are the proof of the stage

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-1200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, flags clean. Display try 1 (keeper local_a27ce56b, on the Top Coordinator's go): boot-display.img left fastboot at 11:40:18, Android was back at 11:41:07 (49 s by the host log, 63 s by the keeper), no gadget ever enumerated, no shell, no pattern. The author saw the Google logo, then black, then Lineage within a minute (research walleye-display-try1-author-saw). Cause hunt: research walleye-display-try1-no-shell-what-is-known (clock_gcc fix is in the image; only the display nodes and DRM differ from z21). Display bisect images d0, d1, d2 built (PR 404); the try walleye-try-display-bisect runs next, unattended, read from the console (decision 2026-11-19-walleye-display-console-before-eyes).

Considered
: The keeper's line 'no root, no mainline shell password' is not the gap: the mainline shell is telnet without password and the kernel never reached the gadget, so the keeper's branch keeper/walleye-display-try-1 (which replaces this entry with that wording) is not merged; ramoops needs unlock and root and every mainline try left pstore empty.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2350-how-the-phone-stands"></a>
### How the phone stands (display bisect)

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up; display bisect: d0 shell ok, d1 loses it (Android in 57 s), d2 not tried; item walleye-try-display-bisect done

Considered
: next: split d1's mdss vs display controller nodes (builder); z21 touch gpio section; walleye-ufs-android-regs needs a mem reader; pmic-wdt items parked

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-display-late-probe-image"></a>
### walleye-builder: the display late-probe image and its host script

Entry date
: 2026-11-19

Done
: Image boot-display-late.img on z21's header, command line and footer: display path (drm, msm, the S6E3FA5 panel driver and their helpers, 9 modules) as modules with depmod output and an ordered /display-modules.list in the ramdisk, netconsole built in, d1's dtb with the display nodes on (clock_gcc on the real gcc), init-late (z14's init, self-reboot held by /tmp/hold). build-display-late.sh and build-display-late-image.sh (every check stops the build, 36+ checks, UFS-off gate and AVB footer among them). tests/host/display-late-probe.sh loads the list one module at a time, writes each step to the host before the load, streams the log by netconsole and a TCP /dev/kmsg stream, proves each channel by a marker; its selftest runs against a fake phone (19 checks, including a phone that dies at msm.ko). Not run on any phone. Sums in the keeper's folder: boot-display-late.img 673c9582...aff4ed.

Considered
: A new init with the loader inside (the host drives the steps instead); netconsole only (the TCP stream is the second channel); the older display worktree as the kernel tree

Rejected
: The older display worktree: its dts no longer compiles (pmi8998_fg label). /tmp for the work: it is a full 16 GB tmpfs, TMPDIR goes on /home

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-1305-walleye-keeper-late-probe-stands"></a>
### How the phone stands (late probe tried)

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adbd up, battery 4384 mV; display late-probe image tried once: no mainline gadget, Android back at 57 s (research walleye-display-late-probe-try1-no-gadget); item walleye-try-display-late-probe stays open

Considered
: next: late-probe kernel with the display nodes disabled in the dtb to tell dtb from kernel; ramoops read of the death; z21 touch gpio section; UFS needs a mem reader (coordinator)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display-dtb-bisect-images"></a>
### walleye-builder: the display dtb bisect images f0..f3

Entry date
: 2026-11-19

Done
: build-display-dtb-bisect-dtbs.sh (Image.gz cut from the late-probe image; f0..f3 dtbs by fdtput on z21's dtb; f1..f3.diff), series build/series/display-dtb-bisect.toml, check-display-dtb-bisect.sh, README section. Programmer review: loose value checks became exact added-line comparisons, parts sums verified by the check, scripts 755. Sums in ~/Projects/walleye-build/images-display-dtb-bisect/SHA256SUMS-display-dtb-bisect. Not run on any phone.

Considered
: Taking the groups from the late-probe image's own dtb (lacks z21's touch-power regulator, shifted phandles); cutting from z21's dtb keeps f0 byte for byte z21's

Rejected
: Rebuilding the dtbs from the display dts: phandles shift and f0 would not be z21's bytes

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-0100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adbd up (idle-android pass); display dtb bisect tried once each: f0,f1,f2 shell up, f3 no gadget; item walleye-try-display-dtb-bisect done

Considered
: next: walleye-display-f3-node-split (builder), then its tries; z21 touch gpio section; walleye-ufs-android-regs needs a mem reader (coordinator); pmic-wdt items parked; upstream the fuel gauge result is the author's

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-display-f3-split-images"></a>
### walleye-builder: the f3 split images g1..g3

Entry date
: 2026-11-19

Done
: Series display-f3-split.toml, parts script and check; g1 cmdline +iommu.passthrough=1, g2 f3 without mdss iommus, g3 mdss alone (display controller explicitly disabled); gate and AVB pass, rebuild identical, programmer review fixed two check gaps

Rejected
: leaving the display controller without status in g3: absent means enabled, which would not be mdss alone

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-0200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (boot_completed 1); display f3 split tried once each: g1 and g2 shell up, g3 no gadget; item walleye-try-display-f3-split done

Considered
: next: builder - late-probe image with the g2 dtb or g1 cmdline (then walleye-try-display-late-probe); z21 touch gpio section; walleye-ufs-android-regs needs a mem reader (coordinator); pmic-wdt items parked

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-display-late-probe-l-images"></a>
### walleye-builder: late-probe images L1 and L2

Entry date
: 2026-11-19

Done
: Built L1 (late image + iommu.passthrough=1) and L2 (late image, mdss iommus removed) with lpoco-bringup image bisect; build-display-late-l-dtb.sh, display-late-l.toml, check-display-late-l.sh; all checks pass, rebuild identical; try item walleye-try-display-late-probe now unblocked.

Considered
: Deriving from f3 (z21's dtb plus display nodes): rejected, f3 lacks the panel node the late probe needs.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-touch-program"></a>
### walleye-builder: the touch try as one program

Entry date
: 2026-11-19

Done
: tests/attended/touch-try.rs (13 named steps) over tools/walleye-tries; found z21's init has no hold; window follows the restart; reader open and EVENTS-READY before the author is asked; Android awaited before the count; CI job runs the crate's tests; unattended run queued for the keeper.

Considered
: A keeper-side shell recipe; an unattended switch in the program; a hold with a deadman (removed after review).

Rejected
: All three: the one program on the shared runner covers it.

About
: walleye-attend-program-touch

---


<a id="2026-11-19-1415-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (boot_completed 1, 4385 mV). Display late-probe L1 tried once: shell up, all 13 display modules load with the shell alive, msm probes to the dpu mmap, no drm card, no panel line (research walleye-display-late-probe-l1-modules-load-no-drm-card); item walleye-try-display-late-probe done. My own timeout 200 cut display-late-probe.sh at step 7; steps 7-13 by hand; use 400+ next time.

Considered
: next: walleye-display-why-no-drm-card (read-only reads on L1); then walleye-tas2557-firmware-source and walleye-haptics-i2c-answers (ready, read-only from Android); L2 not needed; builder: z21 touch gpio section; walleye-ufs-android-regs needs a mem reader (coordinator)

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-pmic-program"></a>
### walleye-builder: the PMIC watchdog try as one program

Entry date
: 2026-11-19

Done
: tests/attended/pmic-try.rs (23 steps) over tools/walleye-tries/src/pmic.rs: gates, reboot test first, bite last, prompts in the author's plain words, A then B fallback; shared phone/host helpers extended (run_partial, image_is_the_built_one); 29 tests with stand-in phone and host programs.

Considered
: Holding the init's restart with /tmp/hold at the S2 check (chosen after review); judging the bite by bootreason (recorded only: format unknown).

Rejected
: Copying touch's plumbing is accepted for now; a shared Lab type is queued (walleye-tries-shared-lab-plumbing).

About
: walleye-attend-program-pmic

---


<a id="2026-11-20-0230-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (idle-android pass); L1 tried once (nothing flashed): the DRM master waits for c994000.dsi, deferred silently, its node has no vdd-supply; item walleye-display-why-no-drm-card done

Considered
: next: builder - L3 = L1 + vdd-supply on the dsi node (the 0.9 V rail read from the phone), then walleye-try-display-late-probe; touch gpio z21 section; walleye-ufs-android-regs needs a mem reader; pmic-wdt parked; ready for a keeper: walleye-attend-touch-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-walleye-builder-display-late-probe-l3-image"></a>
### walleye-builder: late-probe image L3 (vdd-supply on the dsi node)

Entry date
: 2026-11-19

Done
: Built L3 = L1 + vdd-supply (l1a) on dsi@c994000 with lpoco-bringup image bisect; build-display-late-l3-dtb.sh, display-late-l3.toml, check-display-late-l3.sh; gate, AVB pass, rebuild identical; programmer review applied. sha256 9360c0579f1350128b8f43573d9900f8c74e08e0dac21e15089a628f20924e08.

Considered
: A rail read from the dtbo: it names none; l1a chosen by analogy (PHY vdds, driver's 0.9 V).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-drv2624-ported-from-porthole"></a>
### DRV2624 ported from porthole

Entry date
: 2026-11-19

Done
: Took porthole's patch 0018 onto v7.3 as an atomized series: binding (written here), driver (authorship kept), two fix commits after a programmer review (unbind work cancel, range checks, READ_ONCE, GPIOLIB), walleye dts node whose values match the stock overlay; per-chip record; host proof test seen failing and passing; no phone touched.

Considered
: Building a boot image and a try in this item (left to the next item, the part's i2c answer is read first).

Rejected
: Keeping porthole's patch verbatim with all fixes in the same commit: authorship and our changes must be separate commits. Adding a haptics table to device.toml now: the manifest model is lab-tools' (docs #638), not invented here.

About
: walleye-audio-haptics

---


<a id="2026-11-19-2330-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (idle-android pass). L3 tried once from memory: shell up, 13 display modules load, c994000.dsi still deferred silently, no DRM card. Item walleye-try-display-late-probe-l3 done; lease released.

Considered
: next: builder/research - which msm dsi host get defers (source read, then L4 with dev_err_probe or another vdd rail); keeper next: walleye-attend-pmic-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-touch-program-enters-fastboot-itself"></a>
### touch program enters fastboot itself

Entry date
: 2026-11-19

Done
: Added host::to_fastboot and Touch::phone_is_in_fastboot and the step in touch-try.rs; after programmer review: no serial fails fast, adb state is checked before the reboot, the timeout and no-serial cases have tests.

Considered
: Reusing pmic.rs back_to_fastboot directly (it is private to Pmic; the shared-plumbing item will move both onto host.rs).

Rejected
: Reading the serial with unwrap_or_default: an empty serial reboots whatever adb sees.

About
: walleye-attend-program-touch

---


<a id="2026-11-20-walleye-builder-display-late-probe-l4-image"></a>
### walleye-builder: late-probe image L4 (pinctrl labels on real tlmm groups)

Entry date
: 2026-11-19

Done
: Built L4 from L3 (dtb: four overlay labels repointed to tlmm groups, stubs removed; display dts changed to match); scripts, series, check; test fails on L3 and passes on L4; gate, AVB pass; rebuild identical; programmer review applied. sha256 1dfeebe2c1b6867a0f3fb9f18094e3755e9933cee668dbb177d5af27ed1750d9.

Considered
: Changing gen-overlay-stubs.sh: not needed, it stubs only labels our dtb lacks; the display dts deletes the four generic stubs instead (the main dts keeps them).

Rejected
: A full kernel/dtb rebuild for the image: the L1..L3 line patches the previous image's dtb, so L4 differs from L3 in the dtb only.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-1535-walleye-keeper-pmic-unattended-run"></a>
### PMIC program unattended run

Entry date
: 2026-11-19

Done
: Ran tests/attended/pmic-try.rs --log with stdin /dev/null on the lab machine: checks 1-3 (battery good, both images are the built ones, UFS off in both) pass; the run stopped at ask 4 (TWO_TESTS words) with 'nobody answered' before any boot, arming or bite. Phone untouched; idle-android passes afterwards (Android, adb device).

Considered
: Finding: the program asks the author BEFORE booting image A, so the shell, S2 type read and /tmp/hold steps (the item's 'must pass') do not run unattended; only the gates do. To exercise them unattended the program needs the first ask moved after the boot-A/shell/S2 checks, or a --no-ask-until-arm mode: builder work (tools/walleye-tries). Touch item walleye-try-z21-touch-events skipped: attended, its go expired (04:04 on 2026-11-19), author not present on this scheduled run.

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-1540-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, idle-android pass. Only the PMIC program's gates ran (no boot).

Considered
: next: builder reorders pmic-try asks (boot A before the first ask) then keeper reruns walleye-attend-pmic-unattended-run; walleye-tas2557-firmware-source, walleye-haptics-i2c-answers; touch needs a fresh go

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-adsp-q6-and-quaternary-mi2s-ported-from-porthole"></a>
### ADSP, q6 and Quaternary MI2S ported from porthole

Entry date
: 2026-11-19

Done
: Took porthole 0023, 0043, 0075, 0081-0083, 0085, 0159 onto v7.3 as eight patches with authorship kept (0006-0008 refitted at fuzz 3) and a ninth, the walleye dts (ADSP on, protection domain deleted on all seven services, LPASS SMMU, MultiMedia dais, Quaternary MI2S RX on SD1, pin groups); kernel/audio-adsp-q6/ with the part's record; host proof test tests/host/audio-adsp-q6-builds.sh seen failing (bad checksum) and passing; programmer review fixed: missing voice-service deletes, lost credits, test gaps, trap. No phone touched, dt/ unchanged.

Considered
: Porting 0084 whole (sound card and WCD934x codec nodes too); changing dt/ now.

Rejected
: The sound card node here: it names the TAS2557 amp and the codec, whose items come first. Porthole's pin labels: they collide with the overlay stubs, renamed. Marking the item done: its proof is a card in /proc/asound/cards on the phone, not reached.

About
: walleye-audio-haptics

---


<a id="2026-11-20-walleye-builder-display-late-probe-l5-image"></a>
### walleye-builder: late-probe image L5 (tlmm 94 reset gpio single claimant)

Entry date
: 2026-11-19

Done
: Found the second claimant of tlmm 94 (the vendor overlay's qcom,platform-reset-gpio on dsi0), wrote tests/host/display-reset-gpio-single-claimant.sh (fails on L4, passes on L5), built L5 by moving the mdss_dsi0 label onto a disabled stub; gate and AVB pass, rebuild identical, programmer review applied. sha256 fb813a9a702e11a2b3d6bd0b981dceecfb21fe0ca7947a03ebe45d62ed6c2ef5.

Considered
: Loading gpio-shared-proxy.ko from the shell: would work around, not remove, the second claimant.

Rejected
: Disabling dsi0: the panel is its child. Editing the overlay: the bootloader merges the phone's own.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-pmic-first-ask"></a>
### pmic-try: first ask after boot

Entry date
: 2026-11-19

Done
: Moved the TWO_TESTS ask in tests/attended/pmic-try.rs to after boot image A, the shell and the S2 type read; the reboot-test-first, bite-last order and the words are unchanged.

Considered
: Moving the REBOOT ask too: no, it already follows the checks.

Rejected
: Splitting TWO_TESTS wording: it reads fine after boot.

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-walleye-builder-image-l6-stripe-drawer"></a>
### walleye-builder: image L6 (stripe drawer)

Entry date
: 2026-11-19

Done
: Built the stripe drawer and image L6 from L5; programmer review fixed a struct size, logging and release; selftest, gate, rebuild identical.

Considered
: modetest; a libdrm dumb-buffer program

Rejected
: modetest: no aarch64 build; libdrm program: a library to cross-build where fbdev needs only ioctls

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-drv2624-image-built"></a>
### DRV2624 test image built

Entry date
: 2026-11-19

Done
: build/build-drv2624.sh and walleye-drv2624.config; boot-drv1.img built from z21 with the series applied in a new kernel tree, checks and review done, no phone touched.

Considered
: Driver as a module in the ramdisk.

Rejected
: A module: the initramfs carries no modules, so built in as for every image.

About
: walleye-audio-haptics

---


<a id="2026-11-20-0100-walleye-keeper-pmic-run-two"></a>
### PMIC program run 2: boot A and shell pass, check 6 fails

Entry date
: 2026-11-19

Done
: Ran tests/attended/pmic-try.rs unattended (stdin /dev/null) after the builder moved the first ask. First attempt stopped at step 4: phone idle on Android, not in fastboot; I ran adb reboot bootloader and reran. Gates 1-3, boot image A, shell answers pass; check 6 stopped: wdt-state pmic=none. Read from the shell: pm8916-wdt probed, /dev/watchdog0/1 exist, but the watchdog class devices have no identity attribute (no WATCHDOG_SYSFS), so the identity match fails. Nothing armed, nothing flashed. The image's own 180 s self-reboot returned the phone to Android slot b, boot_completed 1, adb up.

Considered
: next: builder walleye-wdt-state-without-sysfs-identity, then rerun; program could enter fastboot itself from Android

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-walleye-tries-one-lab-type-for-the-shared"></a>
### walleye-tries: one Lab type for the shared plumbing

Entry date
: 2026-11-19

Done
: Moved shell, script, serial, battery, android-back and to-fastboot into walleye_tries::lab::Lab; touch and pmic hold one. host::in_fastboot returns Result so an unrunnable fastboot fails at once; serial_of moved to host; unreadable device.toml named. pmic-try.rs starts with a phone-is-in-fastboot step (no adb reboot bootloader by hand). 32 tests pass, clippy clean.

Considered
: Closure-returning methods on Lab itself (would change both entry files)

Rejected
: Kept thin delegating step methods on Pmic and Touch so entry files stay unchanged

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-wdt-state-and-the-petter-find-watchdogs-by"></a>
### wdt-state and the petter find watchdogs by device link when sysfs has no identity

Entry date
: 2026-11-19

Done
: init-wdt1: the petter and wdt-state take the identity file when present, else the device link (*:watchdog = PMIC pon@800, *.watchdog = qcom_wdt SoC). Host selftest +4 cases without identity files, incl. PMIC-only chooses nothing (all pass); target busybox has CONFIG_READLINK. Programmer review: no blocking findings; open risk: the real readlink names are assumed from the item note, to be confirmed on the phone. Images rebuilt into ~/Projects/walleye-build/images-wdt1-sysfsless: A 9dfea09637a813fa677449780ac475a3b859e81135a056fadd8811dadc2f40b4, B 15f5e97d1ac4185fd4a3a5dae3b874cd5ee0267e694a4b454f9889d7efefe933 (earlier images-wdt1 untouched). Item stays open: its proof is the phone try.

Rejected
: enabling WATCHDOG_SYSFS: a ramdisk change is smaller and keeps the kernel as mailed

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-tries-read-device-toml"></a>
### Tries read device.toml

Entry date
: 2026-11-19

Done
: walleye-try-settings-into-device-toml: settings.rs rewritten over dev()/proc(), entry files on lpoco-attended v0.32.0, device.toml keys added, stand-in tests on Settings::stand_in; 31 tests, clippy, fmt, both --dry-run pass, a missing key fails the dry run naming it. The real run needs build/images linked to ~/Projects/walleye-build in the lab clone.

Considered
: Checking the declared sha256 in code too: the SHA256SUMS check already verifies; left.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-battery-floor-into-device-toml"></a>
### Battery floor into device.toml

Entry date
: 2026-11-19

Done
: Declared [health] battery_floor_mv and battery_source; programs read the floor with dev(); battery-ok.sh reads floor and tool names from the manifest, BATTERY_MIN_MV gone.

Rejected
: Keeping a 3700 default in the script: a missing floor would pass blind; it is skip now. Rewriting battery-ok.sh in Rust: the author said shell stays.

About
: walleye-attend-program-touch

---


<a id="2026-11-20-0300-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (idle-android pass). Touch program's unattended run done (z21 booted once from memory, EVENTS-READY, no author, phone back on Android). Gap: touch-try.rs lacks an enter-fastboot step (item walleye-touch-program-enter-fastboot). lpoco lab suite latest does not apply: this phone has no login.

Considered
: next: builder - touch-program-enter-fastboot, L3 dsi vdd-supply, z21-hold-init; keeper next: walleye-attend-pmic-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-1600-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: L4 tried once from slot b, nothing flashed: dsi binds, panel defers on reset-gpios (shared-GPIO proxy), no DRM card. Phone on Android slot b, stock kernel, idle-android pass.

Considered
: next: touch try (go open 2026-11-19); an L5 for the tlmm 94 proxy (walleye-display-late-probe-l5-gpio-shared-proxy); then walleye-attend-pmic-unattended-run. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2000-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up. Tried walleye-tas2557-firmware-source: shell user cannot read the vendor firmware (permission denied); stopped, recorded on the item. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: tas2557 firmware from the factory image's vendor.img on the host (no phone); keeper walleye-attend-pmic-unattended-run; walleye-haptics-i2c-answers

About
: walleye-first-fastboot-boot

---


<a id="2026-11-20-0100-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, idle-android pass. L5 tried once from memory: panel binds, DRM card0 with DSI-1 connected 1080x1920, shell alive, 13 modules load. Item walleye-try-display-late-probe-l5 done; lease released. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder/coordinator - panel enable and a frame (modetest or fbcon) one try, then ask the author to look; keeper next: walleye-modem-mss-proof, walleye-attend-pmic-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2330-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Re-read the TAS2557 firmware pull on the host: sha256 ffadec7e...b8107b0 and 101481 bytes still match the 2026-11-19 note; no porthole copy exists on this host, so the hash-match proof of walleye-tas2557-firmware-source still has nothing to compare to. Phone not touched this run: lpoco lab phone and suite do not apply (this phone has no login, no password file is needed). Phone as the last hand-over left it: Android on slot b, stock kernel, nothing flashed.

Considered
: next: walleye-try-display-stripes-unattended (rank 2, unattended) and walleye-attend-pmic-unattended-run; the firmware item waits on a porthole hash to compare

About
: walleye-audio-haptics

---


<a id="2026-11-20-0430-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (idle-android pass), battery 4333 mV; L6 tried once: stripes drawn by fbdev, connector connected/enabled, panel init commands sent without error; item walleye-try-display-stripes-unattended done in part (glass, drawer exit code unread). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder walleye-display-stripes-script-detach, then rerun the L6 try for the exit code and hold; attended look only after; keeper walleye-attend-pmic-unattended-run waits on the builder's ask move

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-0110-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up (boot_completed 1). PMIC try run 2 booted image A from memory and ended by its self-reboot; no arming. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder walleye-wdt-state-without-sysfs-identity (and walleye-display-stripes-script-detach), then keeper reruns walleye-attend-pmic-unattended-run; L6 stripes exit code still unread

About
: walleye-first-fastboot-boot

---


<a id="2026-11-20-0400-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, idle-android pass. Touch program unattended run 2 clean (no hand step). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: keeper walleye-attend-pmic-unattended-run, walleye-tas2557-firmware-source, walleye-haptics-i2c-answers; builder z21-hold-init, L3 dsi vdd-supply; no password file is needed: this phone has no login for lpoco lab suite or phone (AGENTS.md, lab PR 505)

About
: walleye-attend-program-touch

---


<a id="2026-11-19-1540-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up. Touch try on z21 attended run passed (3593 EV_ABS events, Synaptics S3708AR). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: keeper walleye-attend-pmic-unattended-run (needs builder to move the TWO_TESTS ask), walleye-tas2557-firmware-source and walleye-haptics-i2c-answers (read over adb and the mainline shell, no password file); builder z21-hold-init; password file for lpoco lab

---


<a id="2026-11-19-1610-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up and now left rooted-debugging as the author set it. TAS2557 PG21 firmware pulled read-only to ~/lab-material/walleye/tas2557 (mode 600, never git). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: walleye-attend-pmic-unattended-run, walleye-haptics-i2c-answers, walleye-tas2557-driver; builder z21-hold-init; no password file is needed: this phone has no login for lpoco lab suite or phone (AGENTS.md, lab PR 505)

About
: walleye-audio-haptics

---


<a id="2026-11-20-0600-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, 4370 mV. drv1 tried a second time: DRV2624 bound at 0-005a, id 0x03; item walleye-haptics-i2c-answers done. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder walleye-display-stripes-script-detach, then L6 rerun (walleye-try-display-stripes-unattended-2, unattended); the haptics effect try (play an effect, rail unmeasured) wants a builder row; no password file is needed: this phone has no login for lpoco lab suite or phone (AGENTS.md, lab PR 505)

About
: walleye-audio-haptics

---


<a id="2026-11-19-1830-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, 4383 mV, idle-android pass. Took walleye-try-display-stripes-unattended-2: L6 booted by fastboot from memory, drawer detached ran 60.3 s, exit 0, done. Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: builder walleye-display-late-probe-poll-stripes-rc; then walleye-attend-program-display (attended look); the PMIC unattended run (walleye-attend-pmic-unattended-run) is ready; no password file is needed: this phone has no login for lpoco lab suite or phone (AGENTS.md, lab PR 505)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2000-walleye-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, battery 4376 mV. drv1 tried once: DRV2624 answers at 0-005a (i2c-0), id 0x03, input registered. Item walleye-haptics-i2c-answers in progress (test itself not run). Suite not read: no login (no Linux userland on this phone, so no password file is needed).

Considered
: next: close the haptics item on the dmesg evidence or one more try that lists 0-005a/driver and the supply; then the next ready row is walleye-modem-mss-proof. The drv1 init restarts at 180 s even with /tmp/hold touched.

About
: walleye-audio-haptics

---


<a id="2026-11-19-late-probe-poll-stripes-rc"></a>
### display-late-probe polls for stripes-rc

Entry date
: 2026-11-19

Done
: display-late-probe.sh waits STRIPES_SECONDS then polls the drawer log every STRIPES_POLL (3) s for up to STRIPES_GRACE (40) real s for stripes-rc; selftest has a late drawer (seen failing before the change) and a never-reporting one (bounded fail). Real L6 run queued for the keeper.

Considered
: a longer fixed sleep

Rejected
: a longer fixed sleep: still a guess, and slow on every pass

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-keeper-pmic-run-three"></a>
### PMIC program run 3: steps 1-7 pass on the rebuilt pair

Entry date
: 2026-11-19

Done
: Ran tests/attended/pmic-try.rs for real with the rebuilt images (images-wdt1-sysfsless, sha256 checked; a build/images link to ~/Projects/walleye-build made in the worktree, untracked). Checks 1-3 pass; step 4 fastboot, step 5 image A booted with its banner, check 6 shell answers, check 7 pass: watchdog1 probed, S2 type 0x07, firmware type 0x1. Run stopped at ask 8 (nobody answered), as designed. wdt-state in the ramdisk read pmic=watchdog1 soc=watchdog0 s2-enabled=no without sysfs identity: the fix works on the phone. Way back: the ramdisk has no self-reboot and plain reboot did not leave it; reboot -f then sysrq b over the telnet shell took the phone to Android slot b.

Considered
: Image A's ramdisk stays up until told: the program's end after an unanswered ask leaves the phone in the test ramdisk, off adb and fastboot; a way back (reboot -f / sysrq b via the shell) is not in the program: a builder row. lpoco lab suite has no password file (no login on this phone).

About
: walleye-attend-program-pmic

---


<a id="2026-11-20-1200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed, adb up, 4385 mV, boot completed. PMIC program run 3: steps 1-7 pass on the rebuilt images; items walleye-attend-pmic-unattended-run and walleye-wdt-state-without-sysfs-identity done.

Considered
: next: the author's one command tests/attended/pmic-try.rs (reboot and bite tests); a builder row so the program takes the phone out of the test ramdisk when an ask goes unanswered (reboot -f then sysrq b over the shell); then the display stripes poll and L6 rerun; next ready: walleye-modem-mss-proof (20), walleye-slpi-boot-qrtr (21)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-20-1300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed or written; adb root restarted adbd (Android is throw-away). Read-only pull of modem_a/b, fsg, fsc, modemst1/2 into ~/lab-material/walleye/modem-2026-11-19; modem_b holds mba.mbn and modem.mdt+b00-b27. walleye-modem-mss-proof host step done.

Considered
: next: builder row for a remoteproc_mss test image (dts node, rmtfs, firmware in ramdisk), then one try on slot b; suite latest needs the phone password file at ~/.config/devlab/google-walleye/password (not present), so no suite run this time; next ready: walleye-slpi-boot-qrtr (21)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display-procedure"></a>
### Display try as one Rust procedure

Entry date
: 2026-11-19

Done
: procedures/display-stripes-on-the-pixel-2.rs on lpoco-attended v0.32.1 with steps in tools/walleye-tries/src/display.rs: gate, fastboot, boot L6, wait_for_gadget (library), shell polled and hold set with an on-phone deadman, modules loaded in list order with rc and liveness each, connector poll, expectation said, detached drawer polled to 'drew', look_for, drawer end rc 0 (60 s grace before the hold is released), Android back. Phone not touched. 48 stand-in tests, clippy, fmt, lpoco-attended check and --dry-run pass; programmer review found 5 defects, fixed.

Considered
: library boot_once/back_to_known_good/read_dmesg/check_probe go through devlab phone, which needs a login this phone lacks: used wait_for_gadget and look_for only

Rejected
: tests/host/display-late-probe.sh (reads the drawer result too early); a fixed sleep for the hold

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2130-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Real unattended run of procedures/display-stripes-on-the-pixel-2.rs: steps 1-12 passed, stopped at look_for with 'nobody answered'; phone restarted by the image's deadman into Android on slot b (adb device up). Stock kernel, nothing flashed or written. Log: ~/Projects/walleye-build/display-real.log (first try failed at once on a --log path with AGENT_BUILD unset; nothing ran).

Considered
: next: the program's stop at look_for leaves the phone in the test image until the deadman - it should reboot itself back when nobody answers (for the lane coordinator); suite latest needs ~/.config/devlab/google-walleye/password; next ready: walleye-slpi-boot-qrtr (21)

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-pmic-program-the-way-back-as-the-last-act"></a>
### PMIC program: the way back as the last act

Entry date
: 2026-11-19

Done
: tools/walleye-tries watch.rs: a real run of pmic-try.rs is watched by a parent that re-runs the program as a child, then runs Lab::way_back (already Android: nothing; fastboot: fastboot reboot; test shell: reboot -f, then echo b > /proc/sysrq-trigger; else the power-hold sentence, exit non-zero) and exits with the child's code, only if the child acted on the phone (marker file). 61 stand-in tests, clippy, fmt, dry-run lists the step. Programmer review applied: marker, no duplicate in the child, serial error falls through, signal exit codes.

Considered
: library back_to_known_good (systemd userland: does not fit the telnet shell); atexit hook; a library on_stop hook (lab-tools, not mine)

Rejected
: in-process wrapping of steps: the runner calls process::exit on a failed step or unanswered ask and Ctrl-C kills the process, nothing runs after

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-walleye-pmic-program-ab-and-witnesses"></a>
### PMIC program: Image type, A-to-B fallback test, boot reason and slot read

Entry date
: 2026-11-19

Done
: pmic.rs: an Image type (A, B) replaces loose paths; a stand-in test that A not showing a banner leads to B and says so (and keeps B for the second boot; both failing names image B); a new step after the bite pass reads ro.boot.bootreason and ro.boot.slot_suffix from Android and records them (unread when adb gives nothing). 64 tests pass, clippy -D warnings and fmt clean, --dry-run passes (26 steps). Default images already are the rebuilt sysfsless pair via device.toml [images.wdt1-a/b].

Rejected
: a failing step when the slot or reason is unread: the bite's own witnesses stand, so it is said, not judged

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-1855-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed. Ran the display stripes procedure once under the go; stripes drawn, no witness answer came back, so the procedure itself timed out waiting for the relay (research walleye-display-stripes-try2-no-answer); the author did look and answered at once, his words in research walleye-display-stripes-look-result: the glass lights, fbcon text visible, the stripes flashed briefly.

Considered
: next: re-run when the author is confirmed at the glass and a relay is live (ask him to answer straight to the keeper); else walleye-slpi-boot-qrtr (21)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display-at-boot"></a>
### Display at boot image B1 built

Entry date
: 2026-11-19

Done
: init-display, build-display-at-boot-image.sh, tests/host/display-at-boot-init.sh (behavioural, mutants fail); B1 built, rebuild identical, reviewed

Considered
: building the modules into the kernel (a new kernel build)

Rejected
: built-in DRM: it moves the display probe before the shell, which is what lost the shell in d1; init-loaded modules keep the shell alive

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-ufs-match-prints"></a>
### walleye-builder: the matched UFS image prints the host and PHY registers, read-only

Entry date
: 2026-11-19

Done
: walleye-ufs-match-prints: HCS/HCE at the version read, PCS+0x130 after PCS ready; image rebuilt (sha256 in the backlog note), identical rebuild, build-ufs-match.sh counts the two new strings (5). Programmer review clean; its one point (the PHY value is post-init, not a leftover) is in the commit message and the note. Never booted.

Considered
: reading the PHY before phy_init (clocks are off there: a fault risk)

Rejected
: none beyond that

About
: walleye-ufs-match-android-z

---


<a id="2026-11-19-walleye-research-ufs-solution-extracted"></a>
### walleye-research: the UFS solution extracted from Android

Entry date
: 2026-11-19

Done
: Closing table of Android's UFS path against the matched image after the gaps (walleye-ufs-solution-table); go note refreshed with the rebuilt image's sha and all gap findings.

Considered
: Naming the image 'sha pending': not needed, the prints build landed (#555).

About
: walleye-ufs-match-android-z

---


<a id="2026-11-19-walleye-builder-the-ufs-try-as-one-rust"></a>
### walleye-builder: the UFS try as one Rust procedure

Entry date
: 2026-11-19

Done
: walleye-attend-program-ufs: procedures/ufs-matched-image-on-the-pixel-2.rs over lpoco-attended v0.32.1, steps in tools/walleye-tries/src/ufs.rs, values in device.toml ([images.ufs-match], [procedures.ufs-matched-image-on-the-pixel-2]). The UFS-on boot step holds its own gate (sha256 and go rechecked right before the boot); the go is read from the item's row and must name the image's sha256; read only, watcher runs the way back. Not yet run for real: queued for the keeper without a go.

Considered
: go read by tad row parsing; a watcher parent for the way back (as the PMIC try)

Rejected
: date -d on go_until (reads 'tomorrow' as a go that never ends: strict date shape only); weakening or skipping tests/host/ufs-off-image.sh (untouched); the go check only as an earlier step (the boot step rechecks)

About
: walleye-ufs-match-android-z

---


<a id="2026-11-19-2130-walleye-keeper-banner-shell-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Took walleye-attend-program-ufs. lpoco-attended run ufs-matched-image-on-the-pixel-2 (no go) stopped at check 1 (battery: no voltage from adb or fastboot), phone untouched. The phone is NOT on Android as the last hand-over said: USB shows 1d6b:0104 'Pixel 2 walleye (mainline banner shell)' serial walleye-first-boot (enp10s0f3u2i2 172.16.43.2), not fastboot, no adb. Nothing flashed or written. Suite cannot run (no ~/.config/devlab/google-walleye/password).

Considered
: next: whoever left the banner-shell image on brings the phone back to Android by its way back, then re-run the UFS no-go run; item stays in-progress.

About
: walleye-attend-program-ufs

---


<a id="2026-11-19-2100-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: walleye-attend-program-ufs proven: real unattended run of the UFS procedure stopped at 'no go' and the phone came back to Android; nothing flashed, slot b, stock kernel, Android running (adb up)

Considered
: next: walleye-ufs-card-fw-rev-read (1, read-only adb root), walleye-display-at-boot try (4), pmic unattended run 4 (3); suite latest needs the phone password file (not present)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Took walleye-tas2557-firmware-source (rank 1). Re-checked: pulled blob ~/lab-material/walleye/tas2557/tas2557s_PG21_uCDSP.bin sha256 ffadec7e...06b8107b0 intact. Porthole's tree on the host carries only patches and an APKBUILD that installs the file from a package source dir that is not on the host and pins no hash for it, so the hash-match proof still has no reference. Phone untouched: Android on slot b, stock kernel, nothing flashed.

Considered
: next: item stays open; the proof needs porthole's TAS2557 firmware hash (its docs/BLOBS.md, not on host) or a second pull of tas2557s_uCDSP.bin; the driver try can use the pulled file meanwhile. Next keeper item: walleye-attend-pmic-unattended-run waits on a builder change; then walleye-modem-mss-proof.

About
: walleye-audio-haptics

---


<a id="2026-11-20-1400-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed; B1 tried once unattended from fastboot (13/13 display modules ok, DSI-1 connected, backlight write fails -110, glass not looked at); phone returned by the init's deadman. Tooling note: the run needs build/images linked to ~/Projects/walleye-build and the phone password file is still missing (suite and phone root unavailable).

Considered
: next: author's look at B1 (one command), then the backlight -110 on the dsi cmd dma (research row); next ready: walleye-ufs-card-fw-rev-read (1), walleye-attend-pmic-unattended-run-4 (3)

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2045-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed or changed. Read the UFS card read-only with adb root: SK hynix H28U74301AMR rev D003 (research walleye-ufs-card-revision); risk not known. walleye-ufs-card-fw-rev-read done.

Considered
: next: add the card line to research walleye-ufs-go-note's risk paragraph (coordinator or next keeper); walleye-display-at-boot waits on the author's look (one command); next ready: walleye-attend-pmic-unattended-run-4 (3). Phone password file still missing (suite unavailable).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2130-walleye-keeper-how-the-phone-stands-pmic-run4"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: PMIC program unattended run 4 passed: stopped at the first ask, phone left on Android slot b, stock kernel, nothing flashed. Tooling: build/images must link to ~/Projects/walleye-build (the dir itself); the phone password file is still absent (suite not read: no login).

Considered
: next: author's look at B1 display (one command); then PMIC attended try walleye-try-pmic-wdt-v1 (one command); ready: display-at-boot, slpi-boot-qrtr(21)

About
: walleye-attend-program-pmic

---


<a id="2026-11-19-1957-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Nothing changed: Android on slot b, stock kernel, nothing flashed, in Android per phone-state (adb device). Top ready item walleye-display-at-boot waits only on the author's look (attended, no go open); walleye-try-ufs-match's go window not read as open and it is risk-unknown, so not run; the sysfs-backlight item has no prepared program. Suite and phone root still unavailable (no phone password file).

Considered
: next: author's look at B1 (one command procedures/display-at-boot-on-the-pixel-2.rs); a prepared B1 backlight-write program for walleye-display-backlight-sysfs-write; phone password file at ~/.config/devlab/google-walleye/password

About
: walleye-display-at-boot

---


<a id="2026-11-19-2140-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed. Matched UFS image tried once read only: version 3.1.0, PHY table ok, link stuck HS_GEAR1 HIBERN8; phone back on Android by itself. B1 display still waits on the author's look.

Considered
: next: why the link stays in HS_GEAR1/HIBERN8 (research); walleye-display-at-boot author's look; the coordinator decides a second UFS try

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-walleye-builder-display-look-b2"></a>
### Display look image B2 and the procedure built

Entry date
: 2026-11-19

Done
: tools/walleye-stripes: plan() and KDSETMODE on /dev/tty0 (KD_GRAPHICS while held, KD_TEXT after, also after a failed draw), console state in its log; build/build-display-look-image.sh builds B2 = B1 + the drawer (control byte-for-byte, AVB, UFS-off gate, rebuild identical, b9dbd162...); walleye-tries display_boot: backlight write, stripes held, console checks, 5 new tests; procedures/display-at-boot-on-the-pixel-2.rs on B2 with two look_for; all pins to lpoco-attended v0.35.1

Considered
: shipping B1's look first and the stripes after: the drawer part was cheap (one plan function, one ioctl)

Rejected
: a separate L7 image for the KD fix: B2 carries it and is what the author looks at; running the aarch64 drawer on the host to test: not possible, its logic is the plan tests and the strings check in the image build

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-1500-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed; B2 tried once unattended from fastboot by the real procedure (13/13 modules ok, DSI-1 connected, panel init and backlight -110 as in B1, stopped at look 1 'nobody answered'); phone returned to Android only by the init's 600 s deadman, not by the run.

Considered
: next: the procedure's way back when it stops at an ask (so the bar's 'phone back on Android' holds without waiting 10 min), programmer review, then tell the Top Coordinator: ready, one command; the -110 on the dsi cmd dma (research walleye-display-backlight-timeout); next ready after: walleye-ufs-card-fw-rev-read, walleye-attend-pmic-unattended-run-4. Tooling: phone password file missing.

---


<a id="2026-11-19-walleye-builder-l7"></a>
### Builder: image L7 and the display look with a way back

Entry date
: 2026-11-19

Done
: Built L7 (L6 + the KD_GRAPHICS drawer, build/build-display-l7-image.sh, control byte for byte, rebuild identical); rewrote procedures/display-stripes-on-the-pixel-2.rs as the watched look (look 1 kernel log, look 2 stripes held) on lpoco-attended 0.35.1; display.rs gained the console-mode checks, the two expect texts, the touch mark and the way_back step; 117 tests, clippy, fmt, dry-run (18 steps), check --host ok; programmer review; queued the keeper's unattended run for real.

Considered
: Making B2 the base of the look (init-display loads the modules itself): B2 is not a good look yet (DSI -110, SMMU faults, pclk0 rcg warning), so L6's load order leads.

Rejected
: A new L6-style build script copy per image: this one takes its control from L6's own ramdisk, so the next drawer change needs no new control.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2240-keeper-l7-run"></a>
### Keeper L7 stripes run: 18/18, looks answered by a file

Entry date
: 2026-11-19

Done
: Took walleye-display-look-l7 (rank 1): dry-run 18 steps ok, then the real run with --answer-limit 120: 18/18 passed, the way back left the phone on Android (idle-android passes). Both looks were answered ok by a file written by user 1000, not by the keeper, so the unanswered-stop was not exercised. Android on slot b, stock kernel, nothing flashed.

Considered
: next: re-run with nobody answering, or ask the author whether he answered and what he saw on the glass; then the others by rank. Tooling: build/images -> ~/Projects/walleye-build needed per worktree; phone password file missing.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-walleye-builder-ufs-link-works-slow"></a>
### walleye-builder: the UFS link check reads, it does not fail on a low gear

Entry date
: 2026-11-19

Done
: the_link_is_read replaces the HS-G3 check: three results (full speed, works slow, down); try 1's real link.txt reads as works, slow; stand-in tests for gear 1, gear 3, down and BROKEN; check and dry-run pass

Rejected
: failing on any gear below 3 (try 1's only FAIL, the card read fine); treating only OFF as down (the reviewer: BROKEN is down too)

About
: walleye-ufs-match-android-z

---


<a id="2026-11-19-2245-keeper-l7-unanswered-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed; L7 unanswered run for real passed (unanswered look 1 -> phone back by the watcher in ~24 s); item walleye-display-look-l7-unanswered-run done

Considered
: next: walleye-display-backlight-sysfs-write has nothing left to run (record done, look bundled); next ready: walleye-display-look-b2, walleye-slpi-boot-qrtr, walleye-nfc-nxp-nci-node. Tooling: phone password file ~/.config/devlab/google-walleye/password still missing (lpoco lab phone sh / suite unavailable).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-19-2130-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Took the lease, read the queue. Android on slot b, stock kernel, nothing flashed, not in fastboot (lpoco lab fastboot state). Took no item: the first ready row (walleye-display-backlight-sysfs-write) is already recorded and waits on the author's look; the rank-1 walleye-display-delay-bisect has no image, procedure or tad records yet, and lpoco lab phone/suite cannot run: ~/.config/devlab/google-walleye/password is missing (a secret, not made by a keeper). Nothing on the phone changed.

Considered
: next: someone makes the delay-bisect procedure (one RAM boot per delay 5/15/25/35/50 s, read-only reads first) and a way back by the watcher on an unanswered ask (decision 2026-11-19-walleye-l7-way-back-by-the-watcher); the phone password file written by the author or coordinator; then a keeper takes walleye-display-delay-bisect.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-ci-checks-device-toml-with-the-newest-lpoco-lab"></a>
### CI checks device.toml with the newest lpoco-lab

Entry date
: 2026-11-19

Done
: Copied device-template PR 36's check (script, its stand-in test, the device-toml job) into this lab's CI; ran the test, the real check on main's device.toml and on a copy with an unknown key.

Considered
: Writing it again here; a pinned lpoco version (lab.toml is private, the host follows releases).

Rejected
: Pinning a version: the host follows each release within minutes, so the newest release is the reader that matters.

About
: walleye-attend-program-touch

---


<a id="2026-11-19-walleye-builder-delay-bisect-build"></a>
### Delay bisect build

Entry date
: 2026-11-19

Done
: init-display-delay, build-display-delay-images.sh (five images, all checks pass, rebuilt identical), tests/host/display-delay-init.sh, walleye-tries display_delay.rs (scoring from the reached line, settled read, every module must say ok), procedures/display-delay-bisect-on-the-pixel-2.rs, device.toml keys, README; programmer review: 3 findings fixed

Considered
: Reading the mmcc/autorefresh registers before the load (optional in the bisect item): left to the keeper run

About
: walleye-display-s6e3fa5

---


<a id="2026-11-19-2345-how-the-phone-stands-after-delay-bisect"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: Android on slot b, stock kernel, nothing flashed. The display passes on mainline (L7 looks seen by the author). Delay bisect ran unattended for real, 5 RAM boots: the signs (SMMU faults, rcg warning, DSI -110) show at 5 s and 15 s and are zero from 25 s; item done. The older hand-over 2026-11-20-1500 predates L7 and is stale.

Considered
: next: narrow 15-25 s or reserve the splash region with simple-framebuffer (decision 2026-11-19-walleye-display-early-load-splash-handover); then walleye-slpi-boot-qrtr, walleye-nfc-nxp-nci-node. Tooling: phone password file ~/.config/devlab/google-walleye/password missing (lpoco lab phone/suite); a keeper worktree needs build/images -> ~/Projects/walleye-build.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-0005-walleye-keeper-how-the-phone-stands-pmic-run5"></a>
### How the phone stands

Entry date
: 2026-11-19

Done
: PMIC program unattended run 5 (answer limit 120) passed: stopped at the first ask, phone back on Android slot b, stock kernel, nothing flashed or armed. Top ready row walleye-display-backlight-sysfs-write was already tried on B2 (waits on the author's look), so the rank-1 item was taken. Tooling: build/images needs a link to ~/Projects/walleye-build; phone password file still absent.

Considered
: next: author's one command for the PMIC attended try (walleye-try-pmic-wdt-v1); display look via walleye-display-look-l7; next ready after: walleye-ufs-card-fw-rev-read

About
: walleye-attend-program-pmic

---


<a id="2026-11-20-walleye-tries-flakes"></a>
### walleye-tries flaky tests fixed

Entry date
: 2026-11-19

Done
: Reproduced the ETXTBSY and sysrq flakes under CPU load, wrote fixture::executable and fake::phone_marking, moved the reboot, sysrq and fastboot way-back tests to restart markers; programmer review applied (best-effort comment, /proc panic, simpler labs, marker write error shown); 50 of 50 loaded full-suite runs pass.

Considered
: Locking every spawn; longer margins.

Rejected
: Retries.

About
: walleye-attend-program-touch

---


<a id="2026-11-20-walleye-builder-splash-region-dts"></a>
### Splash region dts and image B-s5

Entry date
: 2026-11-19

Done
: Built B-s5: B-d5 with the splash region reserved and a simple-framebuffer in its dtb; source dts, build script and dtb test written, checks ok, rebuild identical, reviewed; try queued behind a small procedure change (image name).

Considered
: Compiling the new dts in the display kernel tree: refused by missing fuel-gauge labels, so B-d5's dtb is edited as L4 was from L3. Changing the Rust delay procedure here: left as its own item.

Rejected
: Putting the try on the keeper's queue now: the delay procedure only knows b-dN images.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-delay-bisect-takes-the-image-s-name-prefix"></a>
### Delay bisect takes the image's name prefix

Entry date
: 2026-11-20

Done
: Added image_prefix to the display delay bisect: DelayImage carries its name, settings read images.PREFIX+delay, device.toml records the default b-d; B-s5 runs as delays=5, image_prefix=b-s.

Considered
: an optional key (dev/proc cannot read a missing key, so the default sits in device.toml)

Rejected
: a second procedure for the splash image: a copy of the same steps

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-1700-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-20

Done
: Android on slot b, stock kernel, nothing flashed; B-s5 tried once unattended from fastboot by the delay bisect (delays=5, image_prefix=b-s, device.toml put back): 13/13 modules ok, SMMU faults 10, rcg 4, DSI -110 5 - nothing vanished, same as B-d5; phone back on Android by the run's own way back.

Considered
: next: read why simple-framebuffer did not hand over (did it bind? no log line), then the handover in the driver or a load after 25 s (research walleye-display-splash-try-signs-stay); next ready after: walleye-display-look-b2, walleye-slpi-boot-qrtr. Tooling: phone password file missing (state from phone-state.sh).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-walleye-builder-pon-bites"></a>
### Builder: the four PMIC bites as one command, two images and the plumbing

Entry date
: 2026-11-20

Done
: Looked around (item, research walleye-wdt-reset-boots-charger-mode-how-it-stands, v1 procedure and plumbing, lpoco-attended 0.39 groups and physical steps). Built a kernel worktree of the shared linux clone at A's commit with a driver patch (qcom,s2-type, hard by default) in $AGENT_BUILD, built it (out-wdt-warm), generalised build-pmic-wdt-v1.sh by env variables (default run reproduces wdt1 a and b byte for byte), wrote init-wdt2 (wider pon-regs, probe for any type, timeout argument; selftest extended for both inits), built wdt2-hard and wdt2-warm (91 checks each run, rebuild identical), wrote bites.rs (11 stand-in tests: cable in with Android back, a dark phone, the bootloader by itself, cable out, the guard on the type, the image names), settings keys and the entry file; research and decision recorded. Programmer subagent review before landing.

Considered
: Running the cable-out bite with a fixed 20 s timeout (no time to pull the cable: the arm takes a timeout); decoding register bits in the lab (offsets not confirmed on this phone: raw plus a few names from data); one group per whole bite (the dark-phone asks depend on what the lab saw, so the groups are built as the run goes).

Rejected
: Rebuilding the older six entries on 0.39 here (their own items); naming PON_ON_REASON at a guessed offset; reading the registers only in Android (raw registers need root).

About
: walleye-pmic-wdt-v1-image

---


<a id="2026-11-20-walleye-paused-at-the-authors-word"></a>
### The walleye lane paused at the author's word, 2026-11-20

Entry date
: 2026-11-20

Done
: The author's word, relayed by the Top Coordinator: 'lets wrap it up gently for now. let everything open in work complete gently to a stable pause point'. Since the resume the lane did: PMIC follow-up (research walleye-wdt-reset-boots-charger-mode-how-it-stands: the hard reset type 0x07 power-cycles the PMIC so the bootloader shows its charger screen with USB in, read not measured; the upstreams call: v1 keeps 0x07, a warm type would be a later DT-property patch; the cable-out bite folded into the four-bite item walleye-wdt-bite-pon-reasons-read, PR 629, 631, 634, 638); the display: splash region dts built (PR 632), the delay procedure takes an image prefix (PR 636), the keeper's try of B-s5 at 5 s: SMMU faults 10, rcg 4, DSI -110 5, nothing vanished (PR 642), then the research walleye-display-handover-how-others-found and the decision 2026-11-20-walleye-display-handover-reset-first: the faults are msm's new empty SMMU domain meeting the bootloader's live scan-out, simple-framebuffer never bound (no driver in the image), no board hands over cleanly; next image B-r5 = B-d5 plus resets MDSS_BCR (PR 644); the author's look at B2 parked at rank 50; orchestration's lpoco-attended 0.38 to 0.39.1 (declared transport, boot_once with the sha check) noted on walleye-declare-transport-login (PR 635, 641, and this change); the outgoing series live in docs upstream/ (PR 637). THE PAUSE: the research session finished and landed; the builder parked the four-bites item on the pushed branch walleye-wdt-pon-bites-procedure (4b83994): built and host-checked, review found 3 blockers, listed in the item's note with how to resume; its research, decision and log are landed here from that branch. The splash keeper's lease read 'kept by walleye-keeper, renewed 18 min ago' after its run said 'lease released': the keeper session had ended, so I ran lpoco-lab keeper release walleye-keeper (status: free). PHONE: Android slot b, phone-state verdict android (adb lists the serial as device), nothing armed, booted or flashed, lease free, no session of the lane running. RESUME ORDER: 1 walleye-display-reset-image (build B-r5), then the keeper's 5 s try with the signs table; 2 the four-bites branch (fix the 3 blockers, rebuild the warm image, land, the keeper's unattended run, and only then ask the author for the cable moves); 3 walleye-declare-transport-login (declare the images' sha256, drop the checksum-only helpers, keep the UFS-off gate); 4 walleye-display-scanout-registers; then the board dts and panel series in docs upstream/. Waiting on the author's word: UFS try 2 and the OPP image, the PMIC cable moves, the install stage decision. PHONE, LATER READING (01:57 CET): phone-state.sh says 'gone': no witness lists the phone (adb none, fastboot none, usb none). Not known yet why: most likely the author unplugged it at the pause; nobody of the lane touched it (no session was running, nothing armed, nothing flashed; the last read was 'android' at 01:35 CET). The next keeper reads phone-state.sh first and does not take 'gone' as a hang.

Considered
: Starting the builder on walleye-display-reset-image straight away (the next build); the camera and install-stage research items (ready, not started).

Rejected
: Landing the four-bites branch with its 3 blockers (the review's call stands: a half-fixed bite procedure must not reach main); starting any new keeper, builder or research at the pause (the author said to wrap up gently); landing the parked branch's code through a records-only copy (only its records are replayed here).

About
: walleye-pmic-wdt-v1-image

---


<a id="2026-11-20-walleye-builder-b-r5-built"></a>
### Image B-r5 built: B-d5 plus resets = <&mmcc MDSS_BCR> on mdss

Entry date
: 2026-11-20

Done
: Built B-r5 (build/build-display-reset-image.sh, test tests/host/dtb-display-reset.sh seen failing on B-d5's dtb then passing): B-d5's dtb with one added property, resets = <&mmcc 5> on /soc@0/display-subsystem@c900000; kernel, ramdisk (5 s), command line and size are B-d5's; 16 build checks ok (rebuild identical, AVB verifies, UFS-off gate and tests, single reset-gpio claimant, overlay merges). Files in ~/Projects/walleye-build/images-display-reset: boot-b-r5.img sha256 6dc0239e93b4f4b3a01aa1fd3c308aa5208d16f9545b900bfcc38f1ea70904d8, reset.dtb 95f3a8a0..., SHA256SUMS-display-reset. Item queued for the keeper (needs phone): one unattended RAM try, delay bisect at 5 s with image_prefix=b-r. Item stays open: its proof's second half (signs at 0) is the keeper's.

Considered
: Marking the item done now: no, its proof needs the phone run.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-walleye-research-install-stage"></a>
### walleye-research: the install stage, its way back and the author's yeses

Entry date
: 2026-11-20

Done
: Research walleye-install-stage-how-it-stands and the PROPOSED decision 2026-11-20-walleye-install-in-stages-proposed; seven backlog steps (rank 3-9) and tests/host/slot-readback-check.sh (fails until the read-back exists). THE AUTHOR'S YESES, one sentence each: (1) Let the lab read back the boot, dtbo, vbmeta, system and vendor partitions of both slots over adb root (read only). (2) Let the lab install debootstrap and qemu-user-static on this machine, and build Debian minimal first. (3) Let the lab write the new root image into the system partition of the slot Android is not on, while Android runs. (4) Let the lab boot our kernel from RAM with the phone's storage on, mounting that partition read-only. (5) Later: let the lab flash our boot image into that slot and switch the active slot. (6) Later: let our kernel write to the phone's storage (one partition, no erase, no purge). (7) Before a phone-UI image: choose Debian or Nura.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-2330-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-20

Done
: Android on slot b, stock kernel, nothing flashed. B-r5 (B-d5 plus resets = <&mmcc MDSS_BCR> on mdss) tried once unattended from fastboot at 5 s: SMMU faults 0, rcg warnings 0, DSI -110 0, 13 of 13 modules ok; phone back on Android by the run's own way back (phone-state: android). Item walleye-display-reset-image done; [images.b-r5] declared in device.toml; step display-boots-any-order partial. build/images is an untracked link to ~/Projects/walleye-build that a fresh worktree lacks (ln -s ~/Projects/walleye-build build/images).

Considered
: next: the reset in the board dts so any load order works at boot (walleye-display-dts-any-order; try B-r at 2 s via a delay of 2 image), then walleye-display-look-b2 for the author's look; keeper queue after: walleye-install-slot-readback (rank 3), walleye-slpi-boot-qrtr. Tooling: phone password file missing (state from phone-state.sh).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-walleye-builder-b-q5-built"></a>
### Image B-q5 built (spare): B-d5 plus a read-only register sampler (DPU, interface, pingpong, DSI)

Entry date
: 2026-11-20

Done
: Built B-q5 (build/build-display-regs-image.sh; tests/host/display-regs-init.sh seen failing on init-display-delay, then passing): B-d5's kernel, dtb, command line, size and 5 s delay; /init is init-display-regs (init-display-delay plus a regs block, added lines only); /regs.list (41 registers) and /regs-times (4 10 30); busybox devmem 32 bits, never given a value; each read said to the kernel log first so a hang names its register. 24 build checks ok. boot-b-q5.img sha256 86d0813a1ae47ab0d33803d4c4d5a1e24df0988aae760d83e0605b584e80596d in ~/Projects/walleye-build/images-display-regs. Programmer review found DISP_INTF_SEL at 0x104 (is 0x004): fixed, rebuilt. walleye-tries: the delay bisect waits for 'registers: done' when the log shows samples and keeps regs-d5.txt (a log without it is marked incomplete; one reading is single); 141 tests, clippy clean. device.toml has [images.b-q5]; run: delays = 5, image_prefix = b-q. Named risk: if the DPU is runtime-suspended at 10 or 30 s a read could take the SoC; the last read line is lost unless ramoops holds it. Not queued ahead: the coordinator ranked the item off the path (B-r5 is clean at 5 s); it keeps its wait.

Considered
: Samples at 5 s and 30 s only: 5 s races the module load that starts at 5 s, so 4, 10 and 30 s.

Rejected
: A new procedure (the delay bisect already boots a b-prefix image and keeps the log); debugfs reads (exist only after msm probes).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-2359-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-20

Done
: Android on slot b, stock kernel, nothing flashed or written. Item walleye-install-slot-readback done: slot partitions of both slots read twice, identical, kept in ~/lab-material/walleye/slot-readback-2026-11-20-{1,2}. Reader takes LIST from the environment.

Considered
: next: the stage 1 builds first (decision 2026-11-20), then walleye-display-dts-any-order, walleye-display-look-b2; fastboot getvar current-slot still wants a read at the next fastboot visit.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-walleye-alpine-rootfs-v1"></a>
### Alpine rootfs v1 built, reproducible

Entry date
: 2026-11-20

Done
: tools/walleye-rootfs (Rust, argenv, 7 unit tests, clippy clean, in CI) and rootfs/ (packages, runlevels, overlay) build the image rootless twice with one sha256 and check it by what is in it (41 checks). Image and checksum in ~/Projects/walleye-build/images-rootfs/.

Considered
: mke2fs -d on the directory: ctime of the source files made every build differ; a tar under fakeroot fixed it. apk --allow-untrusted: replaced by the alpine-keys package's keys. Running the busybox post-install under qemu: not needed, the applet list is in the package.

Rejected
: Hand-editing the image after mke2fs (debugfs): not reproducible by reading. A chown of the whole tree to root:root by a root command: fakeroot does it with no root.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-20-2359-z-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-20

Done
: Android on slot b, stock kernel, nothing flashed or written. Item walleye-install-current-slot-read done: adb ro.boot.slot_suffix _b and fastboot current-slot b agree; other slot is a. Phone left on Android after a read-only fastboot visit.

Considered
: next: the stage 1 builds first (decision 2026-11-20), then walleye-display-dts-any-order, walleye-display-look-b2.

---


<a id="2026-11-21-walleye-builder-rootfs-v1-hardening"></a>
### Alpine v1 hardening built

Entry date
: 2026-11-20

Done
: Eight hardening fixes in tools/walleye-rootfs, programmer review fixed (directory M: modes, vacuous counts, comments in address scan, command defaults); image rebuilt twice, one new sha256 57cf5fd1; open question on rootfs-boots answered

Considered
: Pinning the package versions too (rootfs/packages names only): left open, noted in the research row

Rejected
: Trusting the APKINDEX for apk.static: the pin replaces it

About
: walleye-display-s6e3fa5

---


<a id="2026-11-21-walleye-builder-ufs-root-image"></a>
### UFS-root image built

Entry date
: 2026-11-20

Done
: build/initramfs-root/init, build-initramfs.sh ufs-root modes, build-ufs-root-image.sh, tests/host/ufs-root-init.sh, config fragment walleye-ufs-root.config; image sha256 35b60fdaf0764e97e0b6de60cddc9a955a14928ba3577f7f110206387f966474; proof test seen failing first, then passing; review fixes applied

Considered
: chroot test with a loop device (needs root): QEMU instead; refusing when androidboot.slot_suffix is empty: left open so a bootloader that omits it does not block the try

Rejected
: an endless petter; fixed device names

About
: walleye-display-s6e3fa5

---


<a id="2026-11-21-walleye-keeper-lease-left-held"></a>
### Keeper runs left the lease held although they said released

Entry date
: 2026-11-21

Done
: Three keeper runs (the splash try, the slot read-back and the current-slot read) ended with 'lease released' in their reports; lpoco-lab keeper status afterwards said 'kept by walleye-keeper, renewed N min ago'. The coordinator released it by name twice, and the third time it held the lab tool update (lpoco 0.41) for over an hour. CAUSE (orchestration, 2026-11-21): release exits 1 with 'no device.toml here or above' from a folder without one, and a keeper transcript shows it run from ~/Projects/linux-phone-oco after its worktree was removed, in a chain that went on. AGENTS.md now tells a keeper to release inside the worktree (or pass --device) before removing it, and to read the status back. Orchestration queued lab-tools keeper-release-from-anywhere.

Considered
: Releasing the lease from the coordinator on every run (it is the keeper's own step and it hides the fault).

Rejected
: Letting the 90 minute lapse do it (it held a tool update for over an hour).

About
: walleye-install-current-slot-read

---


<a id="2026-11-21-ufs-try-2-built"></a>
### UFS try 2 built

Entry date
: 2026-11-21

Done
: Bumped walleye-tries and all six procedures to lpoco-attended 0.41.0; added gear/clock before-during-after, GPT hash and slot-flag steps with stand-in tests; reviewed: load polling paced, keys validated at the dry run, empty GPT hash refused, told text built from settings. Dry run and check --host pass.

Considered
: Slot flags via adb props; hashing the whole disk. Review point left: slot-retry-count may change across the reboot cycle and fail the exact flag compare (a false failure, the phone is on Android)

Rejected
: Both first: see decision 2026-11-21-ufs-try-2-reads-the-gear-under-load-by-a-bounded

About
: walleye-ufs-match-android-z

---


<a id="2026-11-21-0200-z-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-21

Done
: Android on slot b, stock kernel, nothing flashed or written. UFS try 2, parts that do not boot UFS-on: dry run 23 steps pass; walleye-tries 156 tests pass; the real run to the first ask passed checks 1-5 (battery, matched image sha and tests, fastboot visit, slot flags, expect-text) and stopped at check 6, no go for the image's sha256 - as designed. Phone returned to Android by itself. Note: a keeper worktree needs build/images linked to ~/Projects/walleye-build (untracked) before the procedure finds the image.

Considered
: next: the author's go for the one UFS-on try (the text: one command, about 5 min, reads sda up to 4096 MiB or 60 s, nothing written, glass dark, hold power 15 s if not on Android after 4 min); then run the procedure; else stage 1 builds, walleye-display-dts-any-order, walleye-display-look-b2.

About
: walleye-ufs-match-android-z

---


<a id="2026-11-21-walleye-declare-transport-login"></a>
### walleye declares transport and login

Entry date
: 2026-11-21

Done
: device.toml [connection] now has transport = telnet and login = none; lpoco lab suite latest says 'not applicable' instead of 'no password', manifest check reads the file. CI's lpoco-attended pin raised to 0.41.0 to read the fields. Item stays in progress for the helper-dropping half.

Considered
: transport = adb: that is for Android boots, ours are test images on telnet.

Rejected
: dropping the helpers in tools/walleye-tries in this change: it changes how tries boot the phone and needs a keeper-run proof, so a builder who never touches the phone leaves it queued.

About
: walleye-attend-program-display

---


<a id="2026-11-21-walleye-builder-rootfs-lab-key"></a>
### Alpine image with the lab ssh key

Entry date
: 2026-11-21

Done
: Made the lab-only ed25519 key (author's yes 2026-11-21), rebuilt the Alpine image twice with --authorized-key: sha256 57ea7152..., all checks ok. Added to walleye-rootfs: root's .ssh 0:0 700 and authorized_keys 0:0 600, and no private key in the image (every file read whole, unreadable fails, an ELF is skipped: ssh and sshd carry the armor words). The programmer review's four findings fixed. device.toml declares the new image as alpine-rootfs, v1 as alpine-rootfs-v1-no-key.

Considered
: Replacing v1's declaration outright: the item asked the old one to stay under its own name until the write.

Rejected
: Scanning only root/.ssh or only files under 16 KiB for a private key: a key could sit anywhere or in a bigger file.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-21-install-write-procedure-built"></a>
### Install write procedure built

Entry date
: 2026-11-21

Done
: procedures/install-rootfs-on-the-pixel-2.rs and tools/walleye-tries install.rs: 17 stand-in tests (loop files), review fixes, device.toml section.

Considered
: Re-run after a partial write recognises 'target already reads back as the image' (review finding 4): not built, a re-run refuses and says why; Ctrl-C leaving an orphan adb dd (finding 5): the way back does not check for a running dd, the keeper should look before trusting it.

Rejected
: Automatic restore; lpoco-attended 0.42.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-21-install-rehearsal-built"></a>
### Install rehearsal built

Entry date
: 2026-11-21

Done
: rehearse-install-rootfs-on-the-pixel-2.rs: 14 steps, no write step, dd probe on a file; install.rs gained the_dd_works_on_a_file, clean_stage, rehearsal_way_back.

Rejected
: A --rehearse flag on the write's procedure (the runner takes a fixed option set; one file with both paths could write).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-21-install-rehearsal-run"></a>
### Install rehearsal run for real

Entry date
: 2026-11-21

Done
: Linked build/images in the worktree; dry run passed (14 steps); real run passed 14 of 14, no partition written, phone on Android slot b, stage folder empty.

Considered
: next: the Top Coordinator brings the one command procedures/install-rootfs-on-the-pixel-2.rs to the author; then walleye-install-rootfs-boots-try.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-21-0330-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-21

Done
: Android on slot b, stock kernel, nothing flashed or written; the install rehearsal passed for real (14 of 14); the stage folder is empty; fastboot reachable.

Considered
: next: the author's go for the UFS-on try (item walleye-ufs-try-2-one-command), and the one command install-rootfs-on-the-pixel-2.rs brought by the Top Coordinator; keeper worktrees need build/images linked to ~/Projects/walleye-build.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-walleye-builder-rootfs-boots"></a>
### Root boot as one command

Entry date
: 2026-11-22

Done
: Built procedures/rootfs-boots-on-the-pixel-2.rs and walleye_tries::rootfs_boot (guard, go, ssh login, mounts, init lines, card read before/after); android.rs extracted from install.rs; device.toml [images.ufs-root], [procedures.rootfs-boots-on-the-pixel-2]. Reviewed by programmer; findings applied.

Considered
: pre_boot gates and boot_once

Rejected
: pre_boot (global), boot_once (gadget serial mismatch): see decision 2026-11-22-walleye-rootfs-boot-gates-in-the-program

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-rootfs-boots-rehearsal-built"></a>
### Root boot rehearsal built

Entry date
: 2026-11-22

Done
: procedures/rehearse-rootfs-boots-on-the-pixel-2.rs: 14 steps, the root boot's every step but the boot and the go (battery, image and its root, stored chain and key, card read from Android, fastboot guard, slot flags; then Android, card, boot chain and flags read again and compared). Own device.toml table and own out folder. Refactors: disk letters from luns (disk_letters, 1 to 26), one root_for, BLOCK and plain_path in android.rs. Dry run 14 of 14, cargo test 192 pass. Programmer review: two high findings fixed (the rehearsal stopped in fastboot before its re-read; it shared the real boot's out folder, which the boot chain read-back refuses when it has files). Queued for the keeper.

Considered
: A flag on the boot command (as the write got --rehearse): the install rehearsal is its own entry file, so the boot's entry stays one screen that boots; a check that the go row names the sha256 (review, medium) - left to the real run's own gate

Rejected
: Reading the go in the rehearsal: it boots nothing, so a go would be asked for nothing; ssh and the login steps: they need the booted image

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-walleye-keeper-install-written"></a>
### Install stage 1 run: Alpine written to system_a

Entry date
: 2026-11-22

Done
: Battery ok (4352 mV), dry run 16/16, then the real run: one dd to system_a, read back equal to the image, other partitions equal to backup, phone on Android slot b. Told the Top Coordinator at the dd and at the end. Lease released, status free. The first PR's boot-chain check hung 29 min on the runner; redone on a fresh branch.

Considered
: null

Rejected
: null

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-1815-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-22

Done
: Android on slot b, stock kernel; Alpine rootfs (sha256 57ea7152) written to system_a and read back equal; every other partition of both slots equal to the backup; stage empty; fastboot reachable; lease free.

Considered
: next: step 4 of the install go - boot our kernel from RAM with UFS on, mount system_a read-only with tmpfs overlay, log in over usb-network (needs the author's go for the UFS-on try, item walleye-ufs-try-2-one-command). Way back for system_a: the stored backup in ~/lab-material/walleye/slot-readback-2026-11-20-2, only on the coordinator's word.

Rejected
: null

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-1945-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-22

Done
: Root boot rehearsal run for real, 14 of 14 pass, item done. Phone on Android slot b, stock kernel; Alpine in system_a unchanged (card hash equal before and after); fastboot not left; nothing booted.

Considered
: next: the install go step 4, boot ufs-root from RAM, needs the author's go (item walleye-ufs-try-2-one-command / walleye-install-rootfs-boots-try); a fresh worktree needs build/images linked to ~/Projects/walleye-build; a rehearsal run takes ~15 min, give it no short timeout

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-2010-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-22

Done
: Root boot run 1 under the open go: checks 1-15 passed (booted from RAM, ssh as root), stopped at check 16: no androidboot.slot_suffix, init guard failed open. Phone back on Android slot b, stock kernel; Alpine in system_a untouched by design; fastboot not left

Considered
: next: coordinator decides how the init learns the slot (slot_suffix on the cmdline from the procedure) and reopens the go; no rerun by me

Rejected
: null

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-walleye-land-regen-conflicts"></a>
### land fails again on generated-data conflicts: an example for the structure review

Entry date
: 2026-11-22

Done
: The keeper's run-1 records could not land: land's regeneration of docs and generated data conflicted with other sessions' landings of the same files (the keeper's PR 698 earlier, its branch keeper/walleye-rootfs-boots-run-1 now). Every time the coordinator closed the PR and replayed the records on fresh main with ikb (PR 700, and this one); the first PR of the install write also hung 29 minutes on a CI check. The pattern: any two sessions landing within minutes of each other conflict on data/*.json and docs/*.md, which are generated from the same tables. Sign for the structure review: the generated files are the same lines for everyone.

Considered
: A land that regenerates after the rebase (orchestration's land-regen is enabled and still conflicts when two PRs are open at once); one landing session per repository.

Rejected
: Hand-resolving the generated files (they are regenerated by canon and render, a hand merge would drift).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-walleye-builder-ramroot-image"></a>
### Alpine in RAM on the glass: image B-a5 and its one command

Entry date
: 2026-11-22

Done
: tools/walleye-ramroot (build/check, 11 unit tests), build/initramfs-ramroot/init (deadman 300 s), tests/host/ramroot-init.sh (QEMU, 4 cases), image B-a5 (sha256 575bbe43...76e0) built twice identical and checked, device.toml entries, CI step, procedure module walleye-tries ramroot.rs; control seen failing (--check on B-r5). Programmer review: 7 findings, all fixed.

Rejected
: Python; touching the phone.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-walleye-keeper-ramroot-run1"></a>
### Alpine in RAM: unattended run 1

Entry date
: 2026-11-22

Done
: Ran procedures/alpine-login-on-the-glass-on-the-pixel-2.rs unattended up to the look: 15 checks and the deadman hold passed, nothing flashed; phone back on Android. First attempt stopped at check 2 before touching the phone: my fresh worktree had no build/images (git-ignored), fixed by a symlink to ~/Projects/walleye-build.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-2100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-22

Done
: Alpine-in-RAM image B-a5 run unattended to the look: checks 1-15 passed, deadman held, nothing flashed. Phone back on Android slot b, stock kernel; fastboot not left (adb device); system_a untouched by this run. next: the author's look as the same one command on lpoco-attended with the answer file; coordinator reopens the go for the root boot (slot_suffix) separately

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-2020-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-22

Done
: Phone on Android slot b, stock kernel, adb device, nothing changed by this run. Top ready item walleye-display-alpine-ramroot-image waits only on the author's look at the glass (unattended run already done, a rerun with nobody there adds nothing); took walleye-slpi-boot-qrtr host-first: stock Android's live /proc/device-tree lists pil_slpi_region@94300000 but reg is root-only; research fdt-compare already has slpi 0x94c00000 +0xf00000 vs our 0x94b00000 +0x700000; the slpi_v2.mdt program headers (the deciding fact) sit in /vendor/firmware, readable only with adb root, which no decision covers for this item

Considered
: next: coordinator decides whether adb root for read-only pulls of slpi_v2.mdt is within the recorded decisions (readbacks used it); then readelf -l gives the p_paddr and the carve-out is settled; empty dir ~/lab-material/walleye/slpi-firmware-20261122 left for it

About
: walleye-slpi-boot-qrtr

---


<a id="2026-11-22-walleye-builder-boot-slot-on-cmdline"></a>
### The root boot puts the slot on the command line; the init fails closed

Entry date
: 2026-11-22

Done
: init refuses without androidboot.slot_suffix (_a/_b), QEMU cases 9 and 10; the root boot and its rehearsal make this run's image (lpoco-bringup image swap, declared cmdline + the slot adb read; checked equal in every byte but the command line and the one AVB digest; both sha256 kept); the boot gate boots only the made file, re-checked against its sha256; image rebuilt twice identical, sha256 c3373d7199d26a403a7ac9b10c8c95f4dd17f41d358eb59de095efd6bb6c7ba9 in device.toml; 209 tests, clippy, fmt, dry runs pass; programmer review applied

Considered
: walleye.root checked against the reading (kept the suffix: Android's own bootloader gives it)

Rejected
: editing header bytes in our code; any byte change after the content; a typed slot

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-2050-walleye-keeper-the-look-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-22

Done
: The look ran (B-a5): checks 1-15 pass, the author saw boot text ending 'starting sshd', 'starting local', blinking cursor, no login prompt (research walleye-ramroot-b-a5-the-look-no-login-prompt). Phone on Android slot b, stock kernel, nothing flashed; lease released

Considered
: next: builder reads why no getty prompt shows on tty0 (inittab / OpenRC local) and rebuilds; then one more look

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-the-login-prompt-on-the-glass-tty1-and-the"></a>
### The login prompt on the glass: tty1 and the screen read

Entry date
: 2026-11-22

Done
: Builder: getty on tty1 (inittab, securetty, rootfs and ramroot checks), rootfs v3 and B-a5 rebuilt and declared, tools/walleye-tries ramroot gains the wider read and the screen check (/dev/vcs polled until HOSTNAME login:), procedure check 15.

Considered
: tty0 kept with a second getty on tty1; reading the screen through fbgrab

Rejected
: see decision 2026-11-22-walleye-glass-getty-on-tty1-screen-measured

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-2200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-22

Done
: Rehearsal with slot on cmdline passed 15/15 for real; item walleye-install-boot-slot-on-cmdline done. Phone on Android slot b, stock kernel, nothing flashed or booted

Considered
: next: item walleye-display-alpine-getty-on-the-glass (rank 1, unattended run, needs rootfs v3 written to system_a first); coordinator opens the go for the root boot. a fresh worktree needs link build/images -> ~/Projects/walleye-build

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-2300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-22

Done
: Ran the Alpine login procedure on B-a5 for real, unattended: checks 1-16 pass, but the prompt is mid-screen (row 84) and init output follows it, so the glass shows no prompt at the foot: items not marked done. Phone on Android slot b, stock kernel, nothing flashed

Considered
: next: walleye-install-rootfs-boots-try (go open until 23:43, attended: one command, needs rootfs v3 in system_a); walleye-display-alpine-getty-experiments (build) takes the new finding: check 15 must read the cursor row. Fresh worktree: ln -s ~/Projects/walleye-build build/images (the folder itself)

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-2125-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-22

Done
: Ran the rootfs-boots rerun for real under the open go: checks 1-16 pass (booted, ssh login), stopped at check 17, the init did not hand over to the root's /sbin/init; not rerun. Phone on Android slot b, stock kernel, nothing flashed; post-run GPT/boot-chain read-back did not run (not known yet)

Considered
: next: coordinator decides; an item for why the init did not switch_root; run the bootchain read-back (tests/host/bootchain-readback.sh) before the next try

About
: walleye-display-s6e3fa5

---


<a id="2026-11-22-walleye-stable-pause-for-the-steward"></a>
### paused for the structure review, 2026-11-22

Entry date
: 2026-11-22

Done
: The Top Coordinator asked for a stable pause after the stage 1 boot rerun's result. WHERE THINGS STAND. DONE TODAY: install stage 1 step 1 (read-back of both slots, twice, equal) and the write (the Alpine rootfs v2 with the lab key written to system_a: one dd, read back equal, every other partition equal, PR 700); the stage 1 boot as a command (procedures/rootfs-boots-on-the-pixel-2.rs, rehearsed 14/14 and 15/15) and its first two runs on the real phone (run 1: booted, ssh login by the lab key, stopped on the missing slot suffix; run 2 with the slot on the command line: booted, ssh login passes, stopped at check 17, the init did not hand over to the root's init, nothing written); the quick glass image B-a5 (Alpine in RAM under B-r5's display kernel): the author SAW the kernel log and OpenRC's text on the glass, the lab measured the login prompt on the screen but mid-screen (row 84) with init output after it; the stage 2 and stage 3 verdicts (research against Android: set_active is the bootloader's own GPT write, allowed by the Top Coordinator's reading with conditions; our kernel sends no command class Android's UFS path did not for one rw mount with nodiscard); the author's standing rules (go per stage, research-then-proceed, no attended look); lpoco 0.41 installed with login = key and the declared transport. LANDED and read back from main; no PR of the lane is open. PHONE: Android slot b, stock kernel, nothing armed, no go open (the boot rerun go is spent), no session of the lane running. RESUME ORDER (the Top Coordinator's, 2026-11-22): (1) THE INIT HANDOVER FIX: item walleye-rootfs-boots-run-2-diagnose first (the procedure reads the evidence on a failed check: dmesg, mounts, ps, the init's log, the card's disks), then the fix of why the init did not switch_root to the root's /sbin/init (run 2, check 17); (2) THE RERUN of the stage 1 boot under a new go naming the image; (3) THE LOGIN-ORDER FIX (item walleye-display-alpine-getty-experiments: check 15 on the cursor row, the prompt respawned at the end of the default runlevel, B-a5 rebuilt, a keeper's run); (4) UFS TRY 2; (5) STAGE 2 (walleye-install-flash-boot-procedure, set_active with the Top Coordinator's conditions), then stage 3, the firmware front, the display any-order image, the merged UFS-plus-display image (needs rootfs v3 written first) and the PMIC four-bites command. NOT KNOWN YET: the post-run checks (the GPT of every disk, the boot chain and the slot flags equal to before) did NOT run in either rerun of the stage 1 boot (run 1 stopped at check 16, run 2 at check 17), so whether the boots changed anything on the card is not measured: it is measured by the next run, and tests/host/bootchain-readback.sh is run before and after. OPEN FOR THE AUTHOR: nothing waits on his word now. SIGNS FOR THE STRUCTURE REVIEW: any two sessions landing within minutes conflict on the generated data and docs (I closed and replayed six times today); keeper runs left a lease held three times (cause: release from a folder without device.toml; fixed in lab-tools 0.42); a launcher refuses a new session while an old one is still open (a stopped keeper stayed 'running' until its queue drained); a queued message to a running keeper was not acted on before the keeper ended; the app pauses a session's messages to others after 20 without a user message (this session wrote notes to the shared folder instead); lane-next lists a dropped item first and prefers in-progress items to ranks.

Considered
: Starting the glass builder before the pause (it can finish after: no phone run); a diagnostic keeper run of the init before the pause (new phone work after the result, the Top Coordinator said to run nothing new).

Rejected
: Rerunning the boot with the same go (the go is spent for the run and the cause is not read yet).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-run-logs-moved-into-research"></a>
### Run logs moved from the lab machine's root into research

Entry date
: 2026-11-23

Done
: Steward (structure review, decision docs 2026-11-23-structure-lab-machine-homes), PR 735: rootfs-boots-run-2.out/.log into research/walleye-display-s6e3fa5/2026-11-22/ (now the material of research walleye-rootfs-boots-run-2-init-did-not-hand-over), pmic-run4.log into research/walleye-pmic-wdt-proof/2026-11-19/, walleye-display-at-boot-1795116139.log into research/walleye-display-s6e3fa5/2026-11-19/, each with a README. No serial, MAC or key in any (counted by the walleye coordinator: 0).

Considered
: AGENT_BUILD (not kept: these are cited evidence).

Rejected
: Leaving them at the root (no row named them; the root holds clones only).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-walleye-resumes-after-the-structure-review"></a>
### walleye resumes after the structure review, 2026-11-23

Entry date
: 2026-11-23

Done
: The Top Coordinator said the structure review is done (MAP.md in docs) and walleye resumes under the author's standing go, in the order of PR 733: the init-handover fix (check 17), the stage 1 rerun, the login-order fix, UFS try 2, stage 2, stage 3. The steward moved the four run logs into research folders (PRs 735, 736) and removed the lane's clean worktrees; the diagnose item's note now names the new path of run 2's log. Two builder items for the kernel flow were added after the phone items (ranks 7 and 8). Never the bootloader partitions, the GPT written by our code, erase or purge. The phone is on Android b, the lease free, no go open.

Considered
: Starting the kernel topics builder before the phone items (rejected: the phone work needs no new kernel, and the Top ordered it beside the phone work).

Rejected
: Leaving the old log path in the diagnose item (the builder would look for a file that has moved).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-walleye-builder-run-2-diagnose"></a>
### Builder: the boot procedure reads its evidence on a failed check

Entry date
: 2026-11-23

Done
: tools/walleye-tries rootfs_boot: the way back, while the phone is still on the image, FIRST reads the evidence over ssh read-only (EVIDENCE_COMMAND), prints it into the log and keeps evidence.txt; THEN sends the phone away; after it, when the run stopped after the boot, the card, boot chain and slot flags are measured against what was read before, and the way back runs once more so the phone never ends in fastboot. Review by the programmer found and fixed: the parent had no slot suffix (now kept in slot-suffix.txt), a same-day folder's old files passed for this run's (a new run forgets them), a measure left in fastboot (way back after), measures with no boot (skipped, said as not known yet). 217 tests, clippy, fmt, dry run pass. Not done: a partial evidence read is lost when ssh fails mid-read (the helper keeps only the last line).

Considered
: reading inside the child on a failed check: the runner ends the process at the first failed step, so the watching parent's way back is the one place that always runs

Rejected
: a new check step for the evidence: a check after the failed one never runs; a mount or fsck probe of the card: the read stays read-only

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-any-order-display-images-built-b-r2-and-b-rp2-on"></a>
### Any-order display images built: B-r2 and B-rp2 on a dtb with the kernel's own mdss reset

Entry date
: 2026-11-23

Done
: kernel/display-mdss-reset: one msm8998.dtsi patch (resets = <&mmcc MDSS_BCR> on mdss). The dtb is built from v7.3 plus the fuel-gauge dts patch and today's board dts, so it also enables i2c@c17a000 with the drv2624 node (checked as the only new node). build/build-display-any-order-images.sh makes B-r2 (modules not before 2 s, B-r5's order) and B-rp2 (panel before msm; the panel's modinfo depends are checked to load before it), each built twice identical, AVB, UFS-off gate, single-claimant and overlay tests pass; in device.toml with sha256. Review by the programmer subagent: three findings fixed.

Considered
: The lab levers (dsi vdd-supply, mdss_dsi0 stub, clock_gcc) stay binary levers as in every display image; only the reset moved into the kernel's tree.

Rejected
: A reversed module list for the other order: insmod needs dependencies first.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-b-a5-rebuilt-so-the-login-prompt-prints-last"></a>
### B-a5 rebuilt so the login prompt prints last

Entry date
: 2026-11-23

Done
: Check 15 tightened to the last non-blank row with a stand-in test; rootfs v4 with the local.d respawn script and its checks; B-a5 rebuilt on v4 (ad91dcd7, identical twice); procedures check and check --host pass.

Considered
: The script in the rootfs overlay (so the system_a root has the fix too) rather than only in the ramroot userland.

Rejected
: ::wait for openrc default (a hanging service could hide the console); changing the kernel command line (not this item).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-0600-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Run 3 of rootfs-boots on image c3373d71 ran unattended: checks 1-16 pass, stopped at 17 though the hand-over happened (PID 1 /sbin/init, overlay root, card ro); GPT, boot chain, slot flags equal after. Phone on Android slot b, stock kernel, nothing flashed

Considered
: next: coordinator decides: move say 'handing over' before the mount --moves in build/initramfs-root/init (rebuild, new image sha, new go) or read PID 1 in the check; no rerun by the keeper

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-0620-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Delay bisect at 2 s on B-r2 and B-rp2, unattended, nothing flashed: 0/0/0 and 13 of 13 on both; item walleye-display-dts-any-order done; step display-any-order partial. Phone on Android slot b, stock kernel, fastboot reachable on demand, nothing written

Considered
: next: coordinator's open call on rootfs-boots check 17 (say before mount --move, or read PID 1) still waits; then walleye-display-alpine-getty-experiments (built, unmeasured), the author's look walleye-display-look-b2

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-check-17-proves-the-hand-over-from-the-live"></a>
### Check 17 proves the hand-over from the live state

Entry date
: 2026-11-23

Done
: walleye-builder: PID 1 (/proc/1/cmdline) read in the one ssh read; check 17 passes when PID 1 is /sbin/init and /lower is the partition the init named, read-only under the overlay; the lost 'handing over' line is no longer required. Programmer review applied (no tr, dead field removed, seams tested, PID 1 in the evidence).

Considered
: /proc/1/exe: both inits run busybox, cmdline tells them apart.

Rejected
: Moving the say before the --moves or rebuilding the image: changes the declared sha, needs a new go and rehearsal (coordinator's decision).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-walleye-ufs-root-display-image"></a>
### U-d5: the UFS root with the display in one image

Entry date
: 2026-11-23

Done
: Built image U-d5 (a4558827): a kernel worktree of topic/ufs-match-android with two commits (MDSS reset, panel driver) on branch wip/walleye/ufs-root-display; build-display-late.sh with the diag and ufs-root fragments gave Image, 13 modules and the display dtb; build/build-ufs-root-display-image.sh merges the matched UFS overlay into the display dtb, packs build/initramfs-root-display/init with the modules, and makes the matched header/AVB image; 42 checks, built twice identical, also under another locale. Added tests/host/ufs-root-display-init.sh (text equals the root init plus the display block, block between the last guard and the partition wait, and every case of ufs-root-init.sh on it), INIT_DIR/DISPLAY_FILES in build-initramfs.sh, lib-display-levers.sh. The programmer review found a real bug in my edit of tests/host/ufs-root-init.sh (HERE used before it was set: the plain run failed), a locale-dependent sort in the cpio spec (the sha changed from 648b048a to a4558827 once fixed), a clean-tree check that was too narrow and two weak test lines; all fixed. Item done; boot item walleye-install-ufs-root-display-boots-try queued behind a go and a v4 write of system_a.

Considered
: One init for both images (the root init with an optional display block); the display modules built in instead of as modules (the delay at 5 s is what B-r5 proved, so modules it is); a dtb from the matched z21 dtb plus display fdtput (the board file's own display dts with the UFS overlay merged says the same with less hand work).

Rejected
: Editing the root init in place (its sha is named by a go); building the image into the shared images directory a queue reads (UFS-on images only go to NOT-TO-BOOT-... directories).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-getty-console-loglevel-v5"></a>
### Rootfs v5 and B-a5 rebuilt with the console loglevel lowered

Entry date
: 2026-11-23

Done
: dmesg -n 3 added to the local.d script before the getty respawn; walleye-rootfs's glass_prompt_respawn now fails without it or with it after the pkill or at a level above 3 (test seen failing first); dmesg-in-image check; ROOTFS_IMAGE v5 in walleye-ramroot; v5 and B-a5 built (each twice identical); the procedure made executable.

Considered
: loglevel=3 on the kernel command line (a header word, kernel-side: the item's own fallback); printk.devkmsg or console_loglevel in sysctl (needs procfs tools beyond the script).

Rejected
: Filtering the sync_state lines (they are the kernel's honest report; only the console is quieted).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-0720-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Root boot rerun 5 passed all 26 steps; item walleye-install-rootfs-boots-try done; step rootfs-boots done. Phone on Android slot b, stock kernel, nothing flashed

Considered
: next: walleye-display-alpine-getty-experiments, the author's look walleye-display-look-b2; ufs-root-display image U-d5 boot try

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-stage-2-flash-boot-procedure"></a>
### Stage 2 as one command and its rehearsal, built and reviewed

Entry date
: 2026-11-23

Done
: tools/walleye-tries/src/flash_boot.rs, procedures/install-boot-on-the-pixel-2.rs and rehearse-install-boot-on-the-pixel-2.rs, device.toml tables; the checker for the bootloader's slot switch (flags, GPT bits/GUID swaps/CRCs) tested on synthetic GPTs and a stand-in fastboot that records every call; the programmer review found a missing image-root-vs-slot check, a blind set_active in the way back, a possible reboot into the flashed slot after a failed switch back, stale markers and an unclean per-day folder, and a GPT rule that allowed too much: all fixed with tests. 244 tests, clippy, fmt, lpoco-attended check. Never run against the phone. The init gate fails on the declared ufs-root image today (item walleye-install-stage2-init-booted-slot); the run is item walleye-install-flash-boot-run behind the go.

Considered
: Reading the GPT through fastboot only (it cannot read a partition back); a no-change set_active in the rehearsal (not known whether the ABL takes it: reads only).

Rejected
: A way back that writes set_active blind (it reads current-slot first); an init gate that only looks for the old refusal words (it now needs the init to declare the property).

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-0735-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: B-a5 v5 RAM run: checks 1-14 pass, the prompt is the last real row; check 15 failed on its own ---end marker (fixed in this change, SCREEN_COMMAND). Phone back on Android slot b, stock kernel, nothing flashed

Considered
: next: rerun procedures/alpine-login-on-the-glass-on-the-pixel-2.rs unattended (needs build/images -> ~/Projects/walleye-build in a fresh worktree), then the author's look walleye-display-look-b2; item walleye-display-alpine-getty-experiments stays in-progress

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-0750-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: B-a5 v5 unattended run passes checks 1-17 incl. check 15; item walleye-display-alpine-getty-experiments done. Phone on Android slot b, stock kernel, nothing flashed, fastboot reachable

Considered
: next: the author's look at the glass (B-a5 and walleye-display-look-b2); ufs-root-display image U-d5 boot try; rust-script needs --force after a tools change

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-walleye-builder-stage2-init-booted-slot"></a>
### Builder: the stage 2 init - walleye.booted-slot, the say before the moves, the pon reboot-mode test

Entry date
: 2026-11-23

Done
: Both root inits: the word walleye.booted-slot (root must be system$SLOT; says walleye-init: mounts-the-booted-slot on a code line); without it the stage 1 guard refuses as before (worded 'the slot Android is on'); 'handing over' said before the moves in the root init. QEMU cases 12-14 and a kmsg check (seen failing first); display text test now exact; the gate reads the mark on code lines only (review finding); build-ufs-root*-image.sh counts the reworded guard; tests/host/dtb-pon-reboot-mode.sh. cargo test 244, both QEMU proofs pass.

Considered
: A separate stage 2 init file; loosening the guard for everyone

Rejected
: A separate file: one init with a word is one text to prove; loosening: stage 1 must keep refusing its own slot.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-1130-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Took walleye-install-flash-boot (first ready): the stage 2 rehearsal dry-runs 17 of 17 but stops at check 2 for real - the declared ufs-root image has no boot.img and the stage 2 image item is open (build). Added that blocker to the item. Nothing written to the phone: Android slot b, stock kernel, nothing flashed, fastboot reachable (phone-state.sh: adb device)

Considered
: next: the build lane makes walleye-install-stage2-boot-image; then the rehearsal (no go), then the coordinator's go naming its sha256 on walleye-install-flash-boot-run. Next ready items after it: walleye-slpi-boot-qrtr (rank 21), nfc node, display-late-probe-poll

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-walleye-builder-stage2-boot-image"></a>
### Builder: the stage 2 boot image

Entry date
: 2026-11-23

Done
: build/build-ufs-root-image.sh BOOTED_SLOT=yes (word on the command line, own file name, 2 new checks, text by variant); image built twice identical, 28 checks ok: sha256 e337de53446aa98e4795e214648427e22b467b3ab39f2d208e201951ff4098f1 (root system_a), declared as [images.ufs-root-booted-slot]; install-boot and its rehearsal point at it; the stage 1 variant with the new init (no word) builds too (e21ac306, 26 checks) so the new init does not break stage 1; test: the init gate refuses c3373d71 and accepts the new image (walleye-tries 247 tests, clippy, fmt, make check); rehearsal dry run 17/17. Never booted

Considered
: the keeper runs the rehearsal for real (adb/fastboot need the phone) and then the flash under the coordinator's go naming this sha256

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-0830-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Ran UFS try 2 (walleye-try-ufs-match): checks 1-15 passed, link reaches HS-G3 2 lanes under load; step 16 sample timed out at 10 s, run stopped, way back ran. Phone on Android slot b, stock kernel, nothing flashed or written.

Considered
: next: coordinator decides a second run after raising the sample timeout in the procedure (writes nothing); no second run by the keeper. Then walleye-ufs-try-2 and its one-command item stay open.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-23-stage-1-way-back-through-the-bootloader-first"></a>
### Stage 1 way back through the bootloader first

Entry date
: 2026-11-23

Done
: Built the bootloader-first way back of the stage 1 boot try: reboot bootloader over ssh, wait up to 60 s for fastboot, current-slot said against the slot at the start, fastboot reboot; forced reboots stay as fallback ('bootloader not reached'). Dry run 26 steps, 251 tests, clippy, fmt pass; programmer review fixes applied. No set_active, flash, erase or oem. Not run on a phone.

Rejected
: A device.toml key for the 60 s wait: one constant until a second device needs another

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-ufs-try-2-sample-step-fixed"></a>
### UFS try 2 sample step fixed

Entry date
: 2026-11-23

Done
: load state and gear read split onto two shells, gear_wait=30 s in device.toml, unanswered reads recorded with times, samples saved on every ending, stand-in phone gained a stuck-read row; 254 tests, clippy, fmt, dry run 23 steps, check --host ok; programmer review: samples-on-error and hung-read test taken, silent pass with unanswered reads left (the stall is shown in gear-load.txt)

Considered
: a bigger timeout alone

Rejected
: a bigger timeout alone: it would not tell the cause and would still block the state poll

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-0900-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Stage 1 rerun 6 (c3373d71) passed 26 of 26; bootloader way back not reached (fastboot saw no phone in 60 s), forced way back ran; card, boot chain, slot flags equal; phone on Android slot b, stock kernel, nothing flashed, fastboot reachable

Considered
: next: the coordinator decides on the bootloader way back (reboot bootloader from Linux does not reach fastboot - cause not known yet); walleye-install-flash-boot waits on the stage 2 boot image; then walleye-slpi-boot-qrtr

---


<a id="2026-11-23-pd-mapper-in-rootfs"></a>
### pd-mapper in the Alpine image

Entry date
: 2026-11-23

Done
: walleye-rootfs builds pd-mapper, libqrtr and liblzma static for aarch64 from three pinned archives, installs usr/sbin/pd-mapper; is_static_aarch64_elf test seen failing first, then made total after review; image sha256 22b4fed2... identical over three runs

Considered
: Alpine aarch64 -dev packages (musl): no musl cross compiler here

Rejected
: a dynamic build (glibc on a musl image); copying postmarketOS patches unread (upstream master builds as is); a shell build script (practice 21)

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-0915-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Took walleye-try-ufs-match (try 2 run 2): dry run 23/23, real run stopped at step 16 after 8 HS-G3 samples (shell silent 10 s); way back ran; Android slot b, stock kernel, nothing flashed or written to the card

Considered
: next: coordinator decides third run (load in a lower-priority/bounded form, or read the after GPT and flags separately as PR 776 did); walleye-install-flash-boot waits on the stage 2 image

---


<a id="2026-11-23-walleye-kernel-topics-branches-pushed-lists"></a>
### Walleye kernel topics: branches pushed, lists written, reproduction differs by two commits

Entry date
: 2026-11-23

Done
: Gave the three held commits branches on the fork before anything else; cut topics on v7.3 for fuel gauge, UFS (six commits), audio (8 of 9 patches), panel, MDSS reset; pushed all; wrote kernel/topics.toml; ran rhizoid assemble for ufs-off and ufs-on with --reproduces: DIFFERS in pm8916_wdt.c only (the two reboot/reset-type fixes of the topic tip).

Considered
: Moving topic/pmic-wdt-wake-in-suspend back to e123d3bff (force push of a branch the upstream row cites); splitting the two fixes into a stacked topic

Rejected
: Calling the result equal: the tool says differs; a branch pointing at e123d3bff only to satisfy the check hides the two fixes

About
: walleye-kernel-dts

---


<a id="2026-11-23-walleye-keeper-run-2-gpt-equal"></a>
### walleye keeper: run 2 GPT re-read EQUAL

Entry date
: 2026-11-23

Done
: Took walleye-ufs-try-2-run-2-gpt-equal (first ready): read sda's GPT from Android as root, read only; hash 2de02f2b...57cb equals the run's; boot chain 32 partitions same bytes as bootchain-2026-11-19-2; slot b active; nothing written. Phone left on Android, fastboot reachable.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-1300-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel, nothing flashed, nothing written this stretch. UFS try 2 run 2 left the card's GPT unchanged (research walleye-ufs-try-2-run-2-gpt-equal-result, EQUAL). Next: the install stage 2 image from the build lane; the silent shell under load is a builder item; then walleye-install-flash-boot, slpi-boot-qrtr, nfc node

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-walleye-frozen-lists"></a>
### Frozen kernel lists reproduce the shipped images

Entry date
: 2026-11-23

Done
: Added ufs-off-e123d3bff and ufs-on-3644090c7 to kernel/topics.toml on archive/pmic-wdt-e123d3bff; rhizoid assemble --reproduces says equal for both; tags pushed; kernel_tag on four images

Considered
: backport/pmic-wdt-wake-in-suspend/v7.3 as the pin's name, which the lpoco wrapper accepts; an archive ref is the steward's rule for a kept old commit

Rejected
: tagging wdt1-a/b and the other z images: their kernels are not shown equal to a list

About
: walleye-kernel-series-to-topics

---


<a id="2026-11-23-walleye-kernel-tag-lines-landed"></a>
### walleye kernel_tag lines landed

Entry date
: 2026-11-23

Done
: The four kernel_tag lines (z21, ufs-match, ufs-root, ufs-root-booted-slot) added to device.toml after the CI pin moved to lpoco-attended 0.44.3; lpoco-attended check passes; tags read back from the fork.

Considered
: Moving the pin by hand (rejected earlier: orchestration did it in PR 790).

About
: walleye-kernel-dts

---


<a id="2026-11-23-1000-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel, nothing flashed, nothing written. Stage 2 rehearsal passed 17/17 after two walleye-tries fixes (AVB check via boot.img link; GPT entry count read per header: LUNs have 48/4/4/20/8/8 entries, not 128).

Considered
: next: walleye-install-flash-boot-run stays behind the Top Coordinator's go; then slpi-boot-qrtr, nfc node. Rebuilding procedures needs rust-script --force after a walleye-tries change.

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-hynix-archive-tag"></a>
### ufs-hynix-quirks becomes an archive tag on the fork

Entry date
: 2026-11-23

Done
: Tagged archive/ufs-hynix-quirks at d49d54405f11949951c34db236cb3cc2d83f9a66 and pushed the tag to linux-phone-oco/linux; read back with git ls-remote --tags (same commit). Only then deleted the branches topic/ufs-hynix-quirks and the duplicate archive/ufs-hynix-quirks; read back once more: the tag present, both branches gone. Other archive/* branches left (the frozen lists name archive/pmic-wdt-e123d3bff as a branch). Item walleye-kernel-series-to-topics stays in-progress: step 5 (lab dts after assembly), the live lists' tags and the lab-tools wrapper gaps are left.

Considered
: A tag of the same name beside the branch would be ambiguous, so the duplicate branch went too.

Rejected
: Converting the other archive/* branches in this pass: the frozen lists take branches.

About
: walleye-kernel-dts

---


<a id="2026-11-23-walleye-reboot-the-way-back-pushes-the-tool-to"></a>
### walleye-reboot: the way back pushes the tool to /tmp and sends 'bootloader' through RESTART2

Entry date
: 2026-11-23

Done
: New crate tools/walleye-reboot: sync, then reboot(2) RESTART2 with --command; only 'bootloader' is allowed; unknown flags fail (argenv parse_and_lint) so a typo of --dry-run cannot restart a host. build/build-reboot-bin.sh builds it static aarch64, same bytes twice (the target dir is remapped too); device.toml [images.walleye-reboot] declares its sha256. rootfs_boot's way back checks the sha256, pushes the file over ssh into /tmp and runs it; tests cover the push (whole file, before the run, nothing but /tmp) and the refusal on a wrong sha. tests/host/walleye-reboot-restart2.sh; CI line. Review by the programmer agent: the unknown-flag reboot, the non-reproducible build and the late --reboot-bin failure were found and fixed or removed.

Considered
: Installing the tool in the rootfs on system_a (walleye-rootfs --reboot-bin): built and checked on a real image first, then taken out on the coordinator's word: the try must not rewrite system_a, so rootfs v6 is a later item

Rejected
: Putting it in the images c3373d71 or e337de53: they stay as they are. A 'reboot bootloader' string through busybox: it drops the argument (research walleye-reboot-reason-bootloader-read-finding)

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-builder-ufs-try-2-silent-shell"></a>
### walleye-builder: UFS try 2 silent shell

Entry date
: 2026-11-23

Done
: Silent state reads no longer stop step 16; recorded with time, fresh shell, wait to limit+60 s; load_mb 2048; test, clippy, fmt, dry run, host check pass.

Considered
: nice/ionice on dd (untried in the phone); the logs hold no phone text, so the stand-in test reuses run 1's printed gear lines and the run 2 silence is modelled by the stand-in's hung shell.

Rejected
: Only raising the 10 s.

About
: walleye-first-fastboot-boot

---


<a id="2026-11-23-1040-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel, nothing flashed. Rerun 7 of stage 1 (RESTART2): checks 1-21 passed, bootloader reached, but Android not back in 240 s after fastboot reboot; after-stop way back home, GPT/boot chain/slot flags equal (research walleye-rootfs-boots-run-7-bootloader-reached-android-late)

Considered
: next: the coordinator decides on the Top's rule (bootloader reached -> stage 2 go; why fastboot reboot stayed in fastboot 240 s is not known yet); no second run without the coordinator; then walleye-install-flash-boot, slpi-boot-qrtr, nfc node

---


<a id="2026-11-23-stage-2-way-back-asks-with-the-restart2-tool"></a>
### Stage 2 way back asks with the RESTART2 tool

Entry date
: 2026-11-23

Done
: Stage 2's switch_back and stage 1's way back share reboot_tool::ask_for_the_bootloader; review by the programmer subagent: fixed the late sha check (now before the flash), the loose command guard (exact), errors after the bootloader (hand command), stale wording. Fixture bootloader-way-back-reached.txt is rerun 7's printed line; the stage 2 noexec text is a stand-in because rerun 7 recorded no ssh error text.

Considered
: moving ssh_args out of rootfs_boot to remove a module cycle (left: a rename-only change for another item)

Rejected
: a second copy of the push-and-run in flash_boot (the item says one function)

About
: walleye-bootimage

---


<a id="2026-11-23-1100-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel, nothing flashed. UFS try 2 run 3 ran and stopped at step 16 (shell silent under load, as run 2); GPT re-read EQUAL, boot chain 32 partitions same bytes; adb root left on after the read

Considered
: next: coordinator closes try 2 (no fourth run); then install stage 2 image from the build lane, slpi-boot-qrtr, nfc node

About
: walleye-first-fastboot-boot

---


<a id="2026-11-23-1330-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel, nothing flashed or written. Stage 2 rehearsal 2 (code of PR 817) passed 17 of 17; item rehearsal-2 done

Considered
: next: coordinator opens the stage 2 go on walleye-install-flash-boot-run for image e337de53; then slpi-boot-qrtr, nfc node; the silent shell under load stays a builder item

About
: walleye-bootimage

---


<a id="2026-11-23-1140-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel; boot_a now holds image e337de53 (flashed, read back equal); slot a flags retry 1, successful no (from the flash); stage 2 run 1 stopped at check 18, no set_active made

Considered
: next: coordinator decides whether check 18 allows the flash's own flag reset; slot a successful=no is to be known before any set_active

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-1500-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel; boot_a holds the stage 2 image e337de53 (run 1's flash), its slot bits reset by the flash; read from Android, nothing written: only sda's boot_a attribute byte and its CRCs differ from before the flash, five disks equal, chain as flashed; item walleye-stage2-run-1-gpt-after-flash done

Considered
: next: the coordinator settles check 18 (the flash's flag reset is allowed) and decides the stage 2 rerun; then transport-real-phone-run, nfc node

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-builder-ufs-ramped-pinned"></a>
### UFS try: ramped, pinned-low read built

Entry date
: 2026-11-23

Done
: Ramp, governor pin, RAM log, first-silent-step result, after-silence tolerance and PON read in tools/walleye-tries (load.rs) and the UFS procedure; no stage 2 file touched; nothing run on the phone. Review by the programmer found a silent step still failing the run on a dead phone and silences on gear reads being errors: both fixed.

Considered
: Reading the RAM log again after the way back: the phone's RAM is gone then.

Rejected
: Continuing the ramp past the first silent step (more load on a stalled phone for no new cause); making the silent step an error (it is the try's result).

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-builder-check-18-flash-flags"></a>
### Stage 2 check 18 allows what the flash sets

Entry date
: 2026-11-23

Done
: Check 18 amended to exactly the flashed slot's successful (no) and positive retry count; tests from run 1's text; nothing else may differ.

Considered
: Hard-coding retry 1 (observed): AOSP says usually 3.

Rejected
: Dropping check 18 or comparing only the current slot: too loose; b's flags and a's unbootable must stay equal.

About
: walleye-bootimage

---


<a id="2026-11-23-1215-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel; boot_a holds e337de53 (run 1's flash); stage 2 rerun 2 stopped at check 9 before any flash, nothing written, phone healthy on Android

Considered
: next: the coordinator amends check 9 (boot_a already the image); no further run by the keeper on this item

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-stage-2-check-9-accepts-the-declared-image-in"></a>
### Stage 2 check 9 accepts the declared image in boot_a

Entry date
: 2026-11-23

Done
: Amended check 9 (flash_boot.rs, boot_partition_can_be_flashed) and its labels in both install-boot procedures; tests, clippy, dry runs; review fixes applied.

Considered
: A newtype or a step-level stand-in test for the argument order (now, want, declared are all &str).

Rejected
: Reflashing stock boot_a first (an extra write); keeping the strict check.

About
: walleye-bootimage

---


<a id="2026-11-23-1600-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel; boot_a holds the stage 2 image e337de53; rehearsal 3 ran for real, 17 of 17, nothing written; item walleye-install-flash-boot-rehearsal-3 done; the bar before the stage 2 run item goes back on the queue is met

Considered
: next: coordinator decides check 18 and the stage 2 rerun; then walleye-ufs-silent-ramped-pinned-try (needs the author's go for the new procedure), transport-real-phone-run, nfc node

About
: walleye-bootimage

---


<a id="2026-11-23-1240-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Stage 2 rerun 3 under the go for e337de53: checks 1-15 passed, stopped at check 16 (flash changed nothing because boot_a already held the image); nothing set_active; phone on Android slot b, stock kernel; boot_a holds e337de53

Considered
: next: coordinator amends check 16 (unchanged boot_a is right when it already held the image) or says to restore boot_a first; then one more run; item walleye-install-flash-boot-run stays in-progress

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-stage-2-check-16-amended-and-the-29-steps"></a>
### Stage 2 check 16 amended and the 29 steps audited for a repeat

Entry date
: 2026-11-23

Done
: Check 16 no longer demands that boot_a changed: only_the_target_may_change, tests on a stand-in adb (repeat passes, first run passes, boot_a differs fails, dtbo changed fails, boot_b changed fails); a walk of steps 8-20 and the way back against the phone's state after rerun 3 (stand-in stateful fastboot); the audit of all 29 steps recorded as research.

Considered
: Flashing the stock boot_a back first (the keeper's second option).

Rejected
: Restoring the stock boot_a before the run: an extra write to restore a precondition when the check, not the phone, was wrong.

About
: walleye-bootimage

---


<a id="2026-11-23-1315-walleye-keeper-stage2-run-4"></a>
### Stage 2 run 4: stops at step 21, phone on Android b

Entry date
: 2026-11-23

Done
: Fresh worktree; linked build/images (AGENTS.md); dry run 29 of 29; real run: steps 1-20 passed, step 21 no ssh from slot a within 170 s; the phone returned to Android slot b by watchdog; read slot flags through devlab (a unbootable, b successful current), devlab fastboot reboot now; idle-android passes. Research walleye-stage2-run4-stops-at-step-21-no-ssh-from-slot-a. Item walleye-install-flash-boot-run set open, not rerun.

Considered
: Rerun at once (go still open to 15:00)

Rejected
: Rerun: the item says a second stop is not rerun by the keeper; the coordinator decides what to try (serial/console, longer wait, ramdisk log to a card file)

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-1316-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel, successful and current; boot_a holds stage 2 image e337de53, slot a unbootable (retry 0, never successful); host idle-android passes; lease to be released. Next: coordinator decides how to see the new slot's boot (stage 2 run 4 failed at step 21 with no ssh), then walleye-ufs-silent-ramped-pinned-try (needs the author's go), transport-real-phone-run, nfc node.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-1630-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel, boot_a holds stage 2 image e337de53; read the pstore item: empty, the phone had rebooted since run 4; cmdline compared; nothing written, no reboot; item walleye-stage2-run4-pstore-read done

Considered
: next: coordinator decides check 18 and the stage 2 rerun, with a pstore read before any Android reboot after it; then walleye-ufs-silent-ramped-pinned-try (needs the author's go), transport-real-phone-run, nfc node

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-builder-pstore-first"></a>
### Builder: stage 2 reads pstore first on the fallback

Entry date
: 2026-11-23

Done
: flash_boot step 21: no ssh now waits up to fallback_wait (420 s) sending nothing, reads the failed boot over adb first (pstore listing and files, cmdline, boot reasons, slot), keeps failed-boot-read.txt, then the existing way back runs; the rehearsal has the same read as a read-only check 17. 310 tests, clippy, fmt, dry runs of both procedures (29 and 18 steps), check --host ok. Tests written with the change, not seen failing first. rust-script caches a script by its own text: after a change in tools/walleye-tries run it with --force or the dry run shows the old crate.

Considered
: adb root before the first read; a [boot] manifest key

Rejected
: see decision 2026-11-23-walleye-stage2-pstore-first-then-way-back

About
: walleye-bootimage

---


<a id="2026-11-23-1400-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel, boot_a holds stage 2 image e337de53; rehearsal 4 passed 18 of 18, nothing written, pstore-first read works on Android (pstore empty after the rehearsal's own bootloader excursion); item walleye-install-flash-boot-rehearsal-4 done; phone back on Android slot b, adb device

Considered
: next: coordinator opens the stage 2 run (walleye-install-flash-boot-run) when it chooses; then walleye-ufs-silent-ramped-pinned-try, transport-real-phone-run, nfc node. Note: the item said NO reboot; I read it as no fastboot reboot verb because the procedure's step 10 is adb reboot bootloader and back, as rehearsals 1-3

About
: walleye-bootimage

---


<a id="2026-11-23-1425-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Stage 2 run 5 under the fresh go: needed build/images symlink in the worktree; checks 1-17 pass, check 18 stops it (flash clears slot a's unbootable bit too); nothing written beyond the no-op flash; Android slot b, stock kernel, boot_a holds e337de53; research row written

Considered
: next: coordinator widens check 18 (unbootable bit) and decides the rerun; then ufs ramped try (needs author's go), transport-real-phone-run, nfc node

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-1730-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel, boot_a holds stage 2 image e337de53; read run 5's GPTs from Android: only boot_a's attribute byte (0xb2 to 0x3b) and the CRCs differ, five disks equal, boot chain equal; nothing written, no reboot; item walleye-stage2-run-5-gpt-after-flash done

Considered
: next: coordinator decides check 18 (unbootable clear and priority 2 to 3 are attribute bits only) and the stage 2 rerun, a pstore read before any Android reboot; then walleye-ufs-silent-ramped-pinned-try (needs the author's go), transport-real-phone-run, nfc node

About
: walleye-bootimage

---


<a id="2026-11-23-stage-2-check-18-clears-unbootable-audit-by"></a>
### Stage 2 check 18 clears unbootable, audit by state

Entry date
: 2026-11-23

Done
: Run 5's text turned into tests (failing first), check 18 amended, walk tested from four states, dry run and check --host passed, programmer review fixed; no phone

Rejected
: allowing any unbootable change: no to yes must stay refused

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-builder-the-eleven-kernel-worktrees"></a>
### walleye-builder: the eleven kernel worktrees moved to the worktree home (part A of the clone move)

Entry date
: 2026-11-23

Done
: git worktree move of wt-k-display, drv, fg, next, pmicb, ufsm, ufsroot, upstream-dts, z12, z20, z8 from ~/Projects/walleye-build to ~/.local/share/agent-worktrees/linux/; tips identical before and after, uncommitted dts files carried along. Four files name the new place (device.toml kernel_tree, two build scripts' usage lines, build/README.md). tests/host/ufs-match-android.sh passes from the new place. Part B (the clone itself, linux-v7.3 worktree, the ~30 build scripts naming it) left for the surya coordinator's ok.

Considered
: moving the clone and linux-v7.3 in the same change (the coordinator split it; surya's paths would change)

Rejected
: removing the dirty trees (their dts files are only in the worktrees); touching images-* or NOT-TO-BOOT-* while stage 2 run 6 is live

About
: walleye-kernel-dts

---


<a id="2026-11-23-1505-walleye-keeper-run-6-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Android slot b, stock kernel; boot_a holds stage 2 image e337de53; run 6 stalled at check 16 (adb dd read-back), host child killed, nothing sent after the flash; item walleye-install-flash-boot-run still in progress, rerun waits for the coordinator to record it on the item

Considered
: next: coordinator records the one rerun on the item; the read-back of boot_a should have a bounded timeout

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-walleye-builder-the-kernel-clone-moved-to"></a>
### walleye-builder: the kernel clone moved to ~/Projects/linux-phone-oco/linux (part B)

Entry date
: 2026-11-23

Done
: Listed every branch and worktree of the clone against origin (ls-remote): two were on no remote and are pushed (archive/walleye-dts-polish, wip/walleye/ufs-root-display). mv of the clone, git worktree repair of 14 worktrees (tips and dirty files as before). Paths: google-walleye-lab build/README.md, surya-lab README.md plus a note item for its coordinator, docs rhizoid.graph.toml (paired PRs landed together).

Considered
: moving linux-v7.3 and surya's two worktrees to the worktree home too

Rejected
: moving linux-v7.3 (about 30 scripts compute it from WORK_DIR; no gain now) and surya's worktrees (surya is paused; kept in place is the Top's condition-1 option)

About
: walleye-kernel-dts

---


<a id="2026-11-23-walleye-builder-the-kernel-patch-files-removed"></a>
### walleye-builder: the kernel patch files removed after proof, the live int tags pushed

Entry date
: 2026-11-23

Done
: 17 kernel .patch files, 3 SHA256SUMS and the panel .c removed: each series proved equal to its topic branch (patch-id per commit, cmp for the panel); audio 0009 kept as a judged difference. build-kernel.sh, build-display*.sh and the audio and drv2624 host tests follow the branches (counts and a sha256 pinned). Programmer review found no defect, its four low points are fixed or recorded. int/walleye/ufs-off/v7.3/1 and ufs-on/v7.3/1 pushed and read back.

Considered
: applying 0009 to dt/ (changes the dtb of future images); removing LICENSES/BSD-3-Clause.txt

Rejected
: keeping the series files beside the branches (two copies to drift)

About
: walleye-kernel-dts

---


<a id="2026-11-23-1530-walleye-keeper-stage2-run-7"></a>
### Stage 2 run 7

Entry date
: 2026-11-23

Done
: Ran stage 2 once under the go of PR 870: stops at step 21 as run 4 did (no ssh, watchdog reset to Android b in ~1 min, pstore empty); log, out and failed-boot-read.txt kept in the research folder; phone on Android b, no second run

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-walleye-research-fastboot-reboot"></a>
### walleye-research: why fastboot reboot stayed in fastboot

Entry date
: 2026-11-23

Done
: Read rerun 7's kept log, out and the way back code; public LK sources; ranked four causes; queued the smallest try for the keeper; item done

Considered
: web search found nothing specific to Pixel 2's closed bootloader; kept the LK reading marked as generic

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-research-watchdog-prechecks"></a>
### walleye-research: the watchdog prechecks

Entry date
: 2026-11-23

Done
: Read the stock apps wdt node on Android b (read-only adb), e337de53's dtb and the three .config, qcom-wdt/watchdog_dev v7.3, the stock watchdog_v2.c, our init; one research row

Considered
: bootloader_log region unreadable (no /dev/mem)

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-builder-stage2-debug-image"></a>
### walleye-builder: the stage 2 debug image built, its init proved in QEMU, step 21 keeps the host's USB log

Entry date
: 2026-11-23

Done
: Built build/initramfs-root-debug/init (root init + two blocks behind walleye.debug), tests/host/ufs-root-debug-init.sh (QEMU: D0 text, D1 watchdog petted and gadget up before the card with the host side leasing, pinging and speaking telnet and reading the log, then the root's init runs and the gadget is released; D2 the root never appears: the init holds with the gadget answering; D3 the root init's proof with this init in place), DEBUG_INIT=yes in build-ufs-root-image.sh and DEBUG_GADGET=1 in build-initramfs.sh. Image ec741a98 built twice identical, 30 checks ok, from a kernel worktree at 3644090c7 (the shared linux-v7.3 tree is at c03c1a14507d and dirty: not used). Rust: Settings.usb_log_match, SWITCHED_AT_FILE, keep_the_hosts_usb_log (journalctl -k --since @T, lines of 3-2, serial redacted) in step 21 on every path, tested against the real c5 host journal; 322 tests, fmt, clippy; the rehearsal dry run passes. device.toml: [images.ufs-root-booted-slot-debug], both stage 2 tables name it. Not done: the real run (a keeper, under a go that names ec741a98).

Considered
: The watchdog timeout from sysfs max_timeout against probing busybox -T values; chosen max_timeout (no probing, no daemon restarts).

Rejected
: A python test harness or ssh in the init: no python of our own; the lab key is in the root, not the initramfs.

About
: walleye-bootimage

---


<a id="2026-11-23-1800-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Took walleye-ufs-silent-ramped-pinned-try (first ready row). Dry run of procedures/ufs-matched-image-on-the-pixel-2.rs: all 25 steps pass. Real run stopped at check 6: the go on walleye-try-ufs-match does not name image sha 461077869fb2 (item is done; the decision says the new procedure needs a new go line). The ready list showed the item ready without that go. Phone went to fastboot by step 3 and back by the way back; Android slot b, adb device, nothing booted, flashed or written. A fresh worktree needs build/images linked to ~/Projects/walleye-build (AGENTS.md) before a real run.

Considered
: Granting the go myself: refused, the go is the author's.

About
: walleye-ufs-match-android-z

---


<a id="2026-11-23-walleye-builder-state-model"></a>
### walleye-builder: the stage 2 state model, the table applied

Entry date
: 2026-11-23

Done
: Check 9 patched for earlier images, then superseded by the coordinator's order: state-model harness over 40 start states, the research table applied (8, 9, 18, 20, rehearsal flags), one hole closed (the way back must be good at check 13); programmer review of the first part clean but for stale comments.

Considered
: Keeping an earlier_images list: dropped, the research says what boot_a held is not asserted. Comparing Android's slot flags equal at 24: no, a set_active may reset successful (AOSP update_attempter_android l.368), that would be a new first-run expectation.

Rejected
: Patching check 8 alone (the five stops are one sign); folding 29 into 27 in this change (a relax, not a state).

About
: walleye-bootimage

---


<a id="2026-11-23-1710-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Took walleye-install-flash-boot-rehearsal-5: real run 18 of 18, nothing written. Phone on Android slot b (adb device), stock slots as stored; no kept kernel change; healthy.

Considered
: next: the coordinator opens walleye-install-flash-boot-run with a go and an OUTCOMES paragraph; boot_a holds e337de53 (copy kept), stock backup on file. Fresh worktree needs build/images linked to ~/Projects/walleye-build.

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-builder-android-reference"></a>
### walleye-builder: Android's kernel as a read-only reference

Entry date
: 2026-11-23

Done
: Found the phone's kernel commit in LineageOS/android_kernel_google_wahoo lineage-22.1, added it as read-only remote android of the linux clone with a local tag; recorded.

Considered
: Google's android-msm-wahoo-4.4-android11 as the item said: probed in a scratch repo, 4.4.223 of 2020, no such commit.

Rejected
: Fetching Google's wahoo branches or tags: not the build the phone runs.

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-keeper-rehearsal-6"></a>
### walleye-keeper: stage 2 rehearsal 6 with the whole stdout kept

Entry date
: 2026-11-23

Done
: Took walleye-install-flash-boot-rehearsal-6 (first ready row), linked build/images, ran the rehearsal: 18 of 18, read only, phone on Android slot b (idle-android pass). Kept the stdout and the slot files as a fixture; research row and backlog done.

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-keeper-how-the-phone-stands-2"></a>
### walleye-keeper: how the phone stands

Entry date
: 2026-11-23

Done
: Phone on Android slot b, adb device, nothing flashed or set. boot_a holds e337de53 (unbootable yes, retry 0); b successful. Next ready: walleye-transport-real-phone-run (rank 15); ufs-matched try still waits on the author's go naming image 461077869fb2.

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-keeper-stage2-run8"></a>
### Stage 2 run 8 (debug image)

Entry date
: 2026-11-23

Done
: Real run of install-boot-on-the-pixel-2.rs on the debug image ec741a98 under the coordinator's go: steps 1-20 passed (boot_a flashed and read back, set_active a), step 21 no ssh: outcome 2, the kernel ran and its init refused for want of androidboot.slot_suffix; read through telnet from the host. Phone back on Android b by itself, boot_a still holds the debug image, GPT slot switch only; no second run.

Considered
: Writing the cmdline by hand from the shell: not the keeper's change

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-1815-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Took walleye-install-flash-boot-run (go open, debug image ec741a98); ran stage 2 once: steps 1-20 pass, step 21 outcome 2 (kernel ran, telnet answered, root init refused: no androidboot.slot_suffix from the bootloader). Phone fell back to Android slot b by itself; adb device, battery 4375 mV. boot_a now holds the debug image, slot a not current; GPT changed only by the slot switch (to be read in the item's own later checks). Research walleye-stage2-run8-debug-image-kernel-runs-slot-suffix-missing.

Considered
: next: no second run of this image; the image's cmdline or init must carry the slot (decision for the coordinator); then a new go line for the new sha. Lease released.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-walleye-builder-stage2-cmdline"></a>
### Stage 2 cmdline fix built

Entry date
: 2026-11-23

Done
: Header command line cut from 372 to 149 characters (dyndbg to a run-time write, androidboot.console dropped), a budget and a margin check from captured lines in build-ufs-root-image.sh; the three inits read the slot from four sources and fail closed; QEMU cases 15-20 (root=PARTUUID, bootconfig, device tree line) pass with the root, debug and display init proofs; debug image rebuilt after the programmer review (sha256 a0a55cd0, built twice identical, 35 checks); cargo test, clippy, fmt pass. No phone. The first debug test failure came from two QEMU tests run at once; it passes alone. A git checkout of the root init by mistake lost its edits; they were regenerated from the one source text and re-proven.

Considered
: A device-tree QEMU test: x86 has none, so the init's own line is run on a property file. Review points taken: stderr noise on the device-tree read, a hex-length guard, honest wording on the cut (observed, cause unproven) and on the PARTUUID source (unproven on the phone).

Rejected
: Running the procedure's dry run: it reads the phone, which a builder does not touch. Rebuilding the other image rows: not this item; noted in device.toml.

About
: walleye-bootimage

---


<a id="2026-11-23-1910-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Rehearsal 7 passed 18/18 with the rebuilt debug image a0a55cd0, nothing written. Phone on Android slot b (stock kernel), adb device, boot_a holds run 8's image ec741a98, stock backup on file. The run was NOT made: the go row names ec741a98, not the new sha.

Considered
: next: coordinator opens a go naming the full sha256 a0a55cd0... on walleye-install-flash-boot-run; then one command procedures/install-boot-on-the-pixel-2.rs. Lease released.

About
: walleye-bootimage

---


<a id="2026-11-23-1935-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Stage 2 run 9 made: kernel+init+Alpine root ran from boot_a (telnet read, sda22), step 21 refused for a cut cmdline (no slot_suffix); way back ran, phone on Android slot b, adb device; boot_a holds image a0a55cd0

Considered
: next: coordinator decides step 21 accepting the slot from root=PARTUUID or the DT node, or a cmdline-cut fix; no second run without a new go. Lease released.

---


<a id="2026-11-23-walleye-builder-step21-slot-source"></a>
### walleye-builder: step 21 reads which slot booted

Entry date
: 2026-11-23

Done
: Took walleye-stage2-step21-slot-source (rank 1; the first row of the ready query, walleye-next-image-from-an-int-tag, waits on a kernel change that run 9 made unnecessary, its step 2 is not buildable now). Step 21 amended by the Top Coordinator's rule; tests first (failed with run 9's own refusal), then the code; research walleye-stage2-step21-slot-source-built.

Considered
: Comparing to fastboot's current-slot from the switch's own saved flags (step 20 already ties them to the target slot). The device-tree test value is AOSP's, not captured.

Rejected
: Reading only the first source that answers: a disagreeing second source would pass unseen. Adding step 21 to the state walk: the walk's bench has no ssh login; the step has its own tests.

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-stage2-walk-step21"></a>
### Walk reaches step 21 and the way back

Entry date
: 2026-11-23

Done
: Extended the offline four-state walk (walleye-tries flash_boot.rs) with step 10, 21, 22, the way back and its flags on a stand-in phone that answers run 9's captured text; 16 of 32 states pass, 16 stop at 13; a test says the DT slot_suffix has no captured sample. 333 tests, clippy, fmt clean; no phone.

Considered
: Queueing a read-only keeper read of the DT node and the new slot's GPT: the item allows the test route; left to the coordinator.

Rejected
: Typing a DT value or a new-slot GPT as a fixture: not captured text (two walleye runs were lost to stand-ins on 2026-11-23). Steps 53 on in the walk: they need lab scripts, not asked.

About
: walleye-bootimage

---


<a id="2026-11-23-2025-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Stage 2 run 10 made: step 21 PASSED (slot _a from root=PARTUUID = current-slot, our Alpine root from sda22, ssh login); check 22 stopped on sdb: xbl_a/xbl_b are on different disks, the checker pairs per disk; way back ran, phone on Android slot b, adb device; boot_a holds image a0a55cd0

Considered
: next: builder makes check 22 pair xbl_a/xbl_b across sdb and sdc (test on the captured gpt files of install-boot-20261123-201216), then one more run needs a fresh go. Lease released.

About
: walleye-display-s6e3fa5

---


<a id="2026-11-23-stage-2-reads-over-adb-are-bounded"></a>
### Stage 2 reads over adb are bounded

Entry date
: 2026-11-23

Done
: Bounded sha256 and copy_to reads in tools/walleye-tries android.rs, retried once on a stall, with stand-in tests of a stalled stream

Considered
: The first row (walleye-next-image-from-an-int-tag) waits on a stage 2 kernel fix that was not decided (the fix chosen is a rootfs reboot string, no dts change), so the next ranked buildable item was taken

Rejected
: stat -L for size: a block device reports 0 (found in review); a per-chunk deadline: a whole-read deadline from size is simpler

About
: walleye-bootimage

---


<a id="2026-11-23-2105-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Stage 2 run 11 passed all 29 steps (our kernel and Alpine root from boot_a on slot a, ssh, GPT changed only by the switch, way back). Phone on Android slot b, adb device; boot_a holds image a0a55cd0

Considered
: next: production init image (petter first, no telnet) as the next stage 2 image under a new go; make fresh worktrees find build/images (untracked) so a keeper need not link ~/Projects/walleye-build

---


<a id="2026-11-23-walleye-stage2-done-for-the-debug-image"></a>
### Stage 2 done for the debug image: our kernel and Alpine root boot from the phone's own storage on slot a and the way back returns

Entry date
: 2026-11-23

Done
: Stage 2 run 11 (image a0a55cd0, PR 933) passed all 29 steps: boot_a holds our image, set_active a by the bootloader's own switch, the kernel started (host USB log: the debug gadget, then Alpine), the slot read from the bootloader's root=PARTUUID equal to fastboot's current-slot, /dev/sda22 mounted read-only under the overlay, ssh login as root, the GPT of the new slot differing only by the switch (check 22 pairs by name across six disks: 44 entries' slot bits and 42 type GUIDs, CRCs right), the way back (the RESTART2 tool, set_active b, fastboot reboot) bringing Android b home, the boot chain read again equal to the stored set. The road: runs 1-7 stopped on checks that assumed the state of a first run (18, 9, 16, 18 again, 8: five stops, one sign), on a watchdog we never saw (run 7: no ssh, bootreason watchdog), run 8 (the debug image, kernel runs, telnet) found the bootloader cuts the command line at the same word (slot_suffix lost) and the init refused for want of the slot, run 9 (slot from root=PARTUUID) booted our root, run 10 passed step 21 and stopped on check 22's per-disk pairing, run 11 passed all. The tools that made it possible: the lab host's USB log, the debug init's gadget, the by-state walk, the rehearsals, the Android kernel as a read-only reference (LineageOS 22.1, 113bd1cfa6f5).

Considered
: Opening a go for e337de53 a third time (no: it could not tell anything new apart, the debug image could). A hardcoded slot in the image (no: it hides the cause and works in slot a only).

Rejected
: Restoring the stock boot_a before each run (a write to satisfy a check). Reading the pstore as the evidence channel after a watchdog bite (a hard reset keeps nothing).

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-builder-production-image"></a>
### Production stage 2 image built from the int/ tag

Entry date
: 2026-11-23

Done
: Production init and its QEMU test, PRODUCTION_INIT/KERNEL_TAG in build-ufs-root-image.sh, build-ufs-root-production.sh; reviewed by the programmer, its findings fixed (deadline, dead-petter restart, SIGKILL, test top, tag identity); image b74af863 built from int/walleye/ufs-on/v7.3/1, twice identical, 37 checks; device.toml row and the stage 2 command and rehearsal name it.

Considered
: Where the run stands: rehearsal, four-state walk and the run need the keeper and a go.

Rejected
: Debug init with the word off; kernel from a tree by hand.

About
: walleye-bootimage

---


<a id="2026-11-23-2200-walleye-keeper-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Rehearsal 8 (read only) passed 18 of 18 with production image b74af863 declared; the four-state walk (cargo test walk, 57) passes; items cmdline-too-long and step21-slot-source closed on run 11's proof. Phone on Android slot b, adb device, boot_a holds a0a55cd0, nothing written. build/images must be a link to ~/Projects/walleye-build itself (not its images/ folder)

Considered
: next: the ONE stage 2 run with the production image needs a new go naming b74af863 and outcomes (walleye-install-flash-boot-run is done by run 11; a new item or reopening is the coordinator's); then walleye-wdt0-first-instant-read

About
: walleye-bootimage

---


<a id="2026-11-23-2230-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Stage 2 run 12 with the production image b74af863 passed all 29 steps (research walleye-stage2-run12-production-image-all-29-passed): flash read back, new slot booted from the card, ssh answered with no debug gadget, GPT changed only by the slot switch, way back to Android b, boot chain as stored. Phone on Android slot b by adb, boot_a holds b74af863, nothing else written. Items walleye-install-flash-boot-run and walleye-stage2-production-image done. No bringup step claimed: rootfs-boots is already done.

Considered
: next: walleye-wdt0-first-instant-read; stage 3 stays gated by the UFS items. build/images must link to ~/Projects/walleye-build in a fresh worktree

About
: walleye-bootimage

---


<a id="2026-11-23-check-22-xbl-cross-disk-item-closed-on-run-12"></a>
### Check 22 xbl cross-disk item closed on run 12

Entry date
: 2026-11-23

Done
: Read the item on main: the checker fix (gpt_switch_only_across) is already landed, and run 12 passed all 29 steps with check 22 and 28 (6 disks, 0 slot-bit/type-GUID diffs). Marked the item done; no code change needed.

Considered
: Rebuilding the fix: not needed, it is on main with its tests.

About
: walleye-bootimage

---


<a id="2026-11-23-2300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: UFS try 2 run 4 stopped at step 16 before any load: no powersave governor on the UFS devfreq (userspace, simple_ondemand). Nothing written. Phone on Android slot b by adb; boot_a holds b74af863; no kept kernel change.

Considered
: next: coordinator changes the procedure to pin through userspace at min_freq; then the ramped try; walleye-wdt0-first-instant-read after

About
: walleye-first-fastboot-boot

---


<a id="2026-11-23-walleye-builder-attended-pin-is-one-pin"></a>
### walleye's lpoco-attended is one pin

Entry date
: 2026-11-23

Done
: Moved the pin to tools/walleye-tries/Cargo.toml alone: lib.rs re-exports lpoco_attended, the 13 procedures take it from there, check.yml reads the locked revision. Picked up an earlier unlanded branch and rebased it. No phone run, no behavior change in any procedure.

Considered
: A walleye-tries bin running the CLI (the item's wording): the CLI is a bin of lab-tools' lpoco-attended package, so it would be a copy.

Rejected
: A wrapper bin copying the CLI; a second tag in check.yml.

About
: walleye-bootimage

---


<a id="2026-11-23-walleye-builder-ufs-ramp-pin-userspace"></a>
### The ramped UFS try pins the clock low with userspace

Entry date
: 2026-11-23

Done
: Step 16 follows the research row walleye-ufs-hold-the-clock-read: userspace governor, set_freq 50 MHz, read back; refuse before any load unless the clock is 50 MHz and the governor is userspace (max_freq on a node without it); gear after the pin kept. Review found the first read-back passed on the idle 50 MHz when the write failed; fixed and tested. Tests use run 4's captured log. No phone.

Considered
: max_freq only (kept as the fallback); reading min_freq to choose the clock (the research names 50 MHz and run 4's idle clock is 50 MHz: a constant with its source)

Rejected
: adding DEVFREQ_GOV_POWERSAVE to the config to pass the step (the research and the item forbid it); trusting cur_freq alone as the read-back (already 50 MHz at idle).

About
: walleye-first-fastboot-boot

---


<a id="2026-11-23-2330-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: UFS try 2 run 5 (ramped, pinned by userspace): dry run passed 25 of 25; the real run stopped at check 6, before the phone was touched: the item walleye-try-ufs-match note names the image only as 461077869fb2 (12 hex), and ufs.rs go_decision needs the full 64-hex sha256 461077869fb275ab8803b8bb771050d4f19724ccfd62e197dc1f3fe3a6297df5 in the row (the go covers the image its note names). Phone on Android slot b, nothing booted, nothing written; the image on disk hashes to that sha. Also: a fresh worktree needs build/images linked to ~/Projects/walleye-build (checks 1-2 fail without it).

Considered
: next: coordinator re-issues the go on walleye-try-ufs-match with the full sha in the note (ikb backlog-go --id walleye-try-ufs-match --until ... --outcomes ... plus the sha), then one command: rust-script procedures/ufs-matched-image-on-the-pixel-2.rs; the other ready items need a test or a go first

Rejected
: Writing the sha into the item's note myself: the go is the author's gate, the coordinator reissues it; loosening go_decision to a prefix: a stop is read before the fix, and the exact-sha check is the safeguard

About
: walleye-first-fastboot-boot

---


<a id="2026-11-24-0020-ufs-try-2-run-6"></a>
### UFS try 2 run 6 stopped at step 16

Entry date
: 2026-11-23

Done
: Linked build/images to ~/Projects/walleye-build (the top level, not its images/ folder: the first rerun stopped at check 2, phone untouched). Run 6 passed checks 1-15, stopped at step 16: pin read back 100 MHz not 50 MHz, no load ran, phone returned to Android.

Considered
: next: coordinator decides the pin (read min_freq/available_frequencies, retry set_freq after min/max_freq write) - no second run without the coordinator

About
: walleye-first-fastboot-boot

---


<a id="2026-11-24-0025-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: Phone on Android slot b, adb device, nothing flashed, nothing written; run 6 of UFS try 2 stopped at step 16 (pin read back 100 MHz), no load ran.

Considered
: next: coordinator decides the pin change (capture min_freq and available_frequencies; why 50 MHz at check 15 but 100 MHz after the write) and re-issues the go; build/images in a fresh worktree is a link to ~/Projects/walleye-build itself

About
: walleye-first-fastboot-boot

---


<a id="2026-11-24-0020-two-keepers-one-go"></a>
### Two keeper sessions were live, taking the phone lease in turn; the one go was used twice

Entry date
: 2026-11-24

Done
: Found at 00:17 on the Top Coordinator's word: local_4cf13bce (dispatched 22:47, stopped at check 6 by the short sha, idle waiting for the go) and local_ca73871e (my dispatch at 00:07, the launcher refused nothing because the first was not running): ca73871e ran run 6 (00:10, step 16 refused, pin read back 100 MHz, PR 968) and ended 00:18; 4cf13bce, woken by the re-issued go, started run 7 at 00:15 (RAM boot, hold on, at 'the link is read'). The phone lease was taken in turn, never at once. Not stopped: a kill mid-run leaves the phone held in the debug image; the older keeper was told to finish its way back, record only what differs, run nothing more, and end.

Considered
: Gap closed three ways: my rule (message the idle keeper, never dispatch a second), the go is consumed by its run (item walleye-go-is-consumed-by-its-run), and the author's one-agent-at-a-time rule; the launcher itself (refuse while a session of the task is not ended) is asked of orchestration.

Rejected
: Stopping the live run's session at once: it leaves the held RAM-booted phone without its way back.

About
: walleye-bootimage

---


<a id="2026-11-24-0000-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-23

Done
: UFS run 5 ran after the go named the full sha: probes and card read as before, step 16 refused (cur_freq 100 MHz after set_freq 50 MHz, userspace governor), no load. Android slot b by adb; nothing flashed or written; no kept kernel change.

Considered
: next: coordinator decides the pin (write min_freq too / max_freq, read available_frequencies first) and the GPT hash 6cd7e48c vs 2de02f2b difference; no second run without them

About
: walleye-first-fastboot-boot

---


<a id="2026-11-24-walleye-research-ufs-pin-100mhz-read"></a>
### Why the UFS pin read back 100 MHz

Entry date
: 2026-11-23

Done
: walleye-research (scheduled research lead) took the lane's ready research item walleye-ufs-pin-100mhz-read (rank 1; the first listed row, walleye-device-config-sources-read, is in progress with its remaining parts waiting on their steps). Read v7.3 devfreq, governor_userspace, ufshcd and gcc-msm8998 at topic/ufs-match-android 0f3429f65ce4 and Android 4.4.302's ufshcd and clock-gcc-8998 at the local tag, and runs 6-7's captured files; recorded research walleye-ufs-pin-100mhz-read with three new sources, decision 2026-11-24-walleye-ufs-pin-asserts-the-holding-setting, unblocked the builder item walleye-ufs-pin-print-its-evidence. No phone.

Considered
: Whether 100 MHz meant G3 or a refused write; whether to queue a 50 MHz entry for gcc-msm8998's ufs_axi_clk_src as Android has.

Rejected
: Queuing the clock-table change now: nothing yet ties the 100 MHz core clock to the silence under load; the next run's evidence (debugfs clk_rate) comes first.

About
: walleye-bootimage

---


<a id="2026-11-23-step-16-prints-its-evidence-and-asserts-the"></a>
### Step 16 prints its evidence and asserts the holding setting

Entry date
: 2026-11-23

Done
: walleye-tries step 16 captures the node at three points plus the scaling log, asserts userspace/set_freq 50 MHz/HS_GEAR1/cur_freq 50 or 100 MHz; tests from run 6's captured text; review fixes: write output kept, gear compared by number.

Considered
: Review notes left: step closure length, a typed Gear newtype, tests for set_freq=undefined.

Rejected
: Typed stand-in fixture of the new capture format: no run has printed it yet.

About
: walleye-bootimage

---


<a id="2026-11-24-0115-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-24

Done
: Run 8 of UFS try 2 (the one run of the go, now ended): step 16 passed with the PR 975 pin (userspace, set_freq 50 MHz read back, cur_freq 100 MHz, HS_GEAR1); 64 MiB read fine; shell silent at the 256 MiB step, gear at the silence not known, GPT after not read (shell dead); phone back on Android slot b by itself, nothing flashed or written; stock kept

Considered
: next: coordinator reads the run (research walleye-ufs-try-2-run-8-pin-holds-shell-silent-at-256mib): silence is not shown to be the gear; candidates are the 100 MHz core clock vs Android 50 MHz, the read path, or the shell stack on usb0 starving under O_DIRECT load; then decides a further read-only try (e.g. smaller steps 64/128, or console on the ACM tty instead of telnet). walleye-gpt-hash-from-android-read still open (needs adb root, not done here)

---


<a id="2026-11-24-ufs-procedure-fits-the-image-s-180-s"></a>
### UFS procedure fits the image's 180 s

Entry date
: 2026-11-24

Done
: Built decision 2026-11-24-walleye-ufs-procedure-fits-the-image (a)-(d) in tools/walleye-tries ufs.rs/load.rs and the procedure; programmer review found an unanswered uptime read erased the silence result and a vacuous assert: both fixed. 352 tests, clippy -D warnings, fmt pass.

Considered
: Polling a down link until device_wait (kept: only a down link waits).

Rejected
: Keeping the release-hold step: the image has no hold, way_back reboots from the shell. Not measured: /proc/uptime fixtures are format-standard, not captured; start uptime is kept in gear-load.txt only.

About
: walleye-ufs-match-android-z

---


<a id="2026-11-24-0205-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-24

Done
: Run 9 of UFS try 2 (the reopened go, now ended): pin held, 64/256/1024 MiB read at HS_GEAR1 all rc=0, shell answered throughout, GPT equal 6cd7e48c, phone on Android slot b by itself, nothing flashed or written; stock kept

Considered
: next: coordinator reads research walleye-ufs-try-2-run-9-ramp-passes-at-hs-gear1 and decides: an unpinned load (G3) read-only try, or stage 3's write test inside the 180 s budget. walleye-try-ufs-match left in-progress (its proof names HS-G3). Note for the next keeper: build/images must be a symlink to ~/Projects/walleye-build in a fresh worktree

About
: walleye-first-fastboot-boot

---


<a id="2026-11-24-ufs-procedure-the-unpinned-ramp-variant"></a>
### UFS procedure: the unpinned ramp variant

Entry date
: 2026-11-24

Done
: WALLEYE_UFS_PIN=pinned|unpinned declared with argenv (env only: the attended runner refuses flags it does not declare, and the env is linted so a typo is refused); unpinned writes nothing to the clock, reads governor/clock/gear before and after each ramp step and after the ramp, asserts nothing about the gear, adds 2048 MiB only when the step fits before the image's reboot (left-out step named in ramp-result.txt); tests from run 9's console, run 3's G3 sample, run 6's idle gear. Programmer review found 9 points; fixed per-step reads, false governor-after, hidden left-out step, shared dispatch, typed fixtures.

Considered
: a --unpinned flag; the 2048 step as device.toml data (kept a constant guarded to be larger than the last step)

Rejected
: a flag: lpoco-attended's parse_and_lint(FailOnError) would refuse it before the procedure ran

About
: walleye-ufs-match-android-z

---


<a id="2026-11-24-0300-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-24

Done
: Run 10 of UFS try 2 (the unpinned ramp, the go now ended): HS_GEAR3 2 lanes 200 MHz under load, 64/256/1024/2048 MiB reads rc=0, shell answered throughout, GPT equal 6cd7e48c, phone on Android slot b by itself, nothing flashed or written; stock kept; walleye-try-ufs-match done

Considered
: next: coordinator decides stage 3's write test (the reads are no longer the suspect; the 180 s image timer is). Ready for the keeper: gpt-hash-from-android-read and vendor-firmware-pull (one Android visit), wdt0-first-instant-read. build/images in a fresh worktree is a symlink to ~/Projects/walleye-build

About
: walleye-first-fastboot-boot

---


<a id="2026-11-24-go-decision-names-the-sha-field-it-wants"></a>
### go_decision names the sha field it wants

Entry date
: 2026-11-24

Done
: ufs.rs go_decision takes the item id; a short sha in the go note (a prefix of the image sha256, 8+ hex) is refused naming the field and what was found, no sha says found none; the exact 64-hex check is unchanged. Callers in ufs, flash_boot (install-boot gate) and rootfs_boot pass the go_item. Host tests from the captured run-5 row (research walleye-go-row-run5-short-sha); 369 tests, clippy, fmt clean. Reviewed by the programmer subagent.

Considered
: ikb backlog-go --image-sha256 as its own option: ASKED of idealkb (orchestration), not done here; until it lands the note carries the sha

Rejected
: accepting a prefix in go_decision: loosens the exact-sha safeguard

About
: walleye-bootimage

---


<a id="2026-11-24-every-check-of-the-13-procedures-names-its"></a>
### Every check of the 13 procedures names its source; lpoco-attended 0.45.2

Entry date
: 2026-11-24

Done
: check(..) became check_from(what, research:ID, ..) in all 145 checks, sources from the research rows of each try and the five walleye-stop-* rows; the pin moved to 0.45.2 in the same change (rhizoid PR 966); 13 dry runs ok, a check without a source stops at check 1 (negative run); 369 tests, clippy, fmt. Reviewed by the programmer subagent; its five weak fits fixed.

Considered
: a procedures/SOURCES.toml table: later with the gate

Rejected
: one source per procedure: too coarse, a check should name the row of its own value

About
: walleye-bootimage

---


<a id="2026-11-24-walleye-image-recipe-z21"></a>
### z21's recipe row landed from PR 951 rebased

Entry date
: 2026-11-24

Done
: PR 951's z21 recipe, z20 control, proof test and its CI selftest rebased onto main (data and docs re-recorded, not merged); ensure z21 passes on the host's 0.48.4.

Considered
: merging 951's generated data/docs: they conflict, re-recording is the canon

About
: walleye-bootimage

---


<a id="2026-11-24-a-fresh-worktree-links-its-own-build-images"></a>
### A fresh worktree links its own build/images

Entry date
: 2026-11-24

Done
: link_images in walleye-tries settings: the for_* constructors link build/images to ~/Projects/walleye-build when absent; 3 host tests; dry run in this fresh worktree made the link; 372 tests, clippy, fmt.

Considered
: refusing with the fix instead of linking: a CI runner has no walleye-build, so absent target does nothing

About
: walleye-bootimage

---


<a id="2026-11-24-walleye-declares-its-ufs-off-kernel-build-it"></a>
### walleye declares its ufs-off kernel build; it reproduces itself

Entry date
: 2026-11-24

Done
: [kernel_builds.walleye-ufs-off] in device.toml; build 1 gave the hash, build 2 was equal (Nix accepted it); tests/host/kernel-build.sh; the shipped z21 kernel's version line shows why a rebuild cannot equal it.

Considered
: the dts patch and fuel-gauge commits as patches: Image.gz holds no dtb, and a .patch file in a lab is refused by MAP.md; the fuel gauge belongs to a ufs-on tag

Rejected
: declaring the shipped kernel's hash: a rebuild can never meet it

About
: walleye-bootimage

---


<a id="2026-11-24-walleye-kernel-build-row-waits-for-attended"></a>
### The kernel build row waits for an attended release with manifest 0.47

Entry date
: 2026-11-24

Done
: PR 996's CI refused device.toml's [kernel_builds] (lpoco-attended 0.45.2 reads it with lpoco-manifest 0.46.0); the row, measured equal twice, is kept as research and lands in device.toml with the next attended release (lab-tools main already has manifest 0.47.0).

About
: walleye-bootimage

---


<a id="2026-11-24-walleye-coordinator-handover"></a>
### Walleye coordinator hand-over, 2026-11-24 04:10 CET: in flight, waits, order, decided-not-recorded

Entry date
: 2026-11-24

Done
: STATE: Stage 1 and 2 done (Alpine root boots from the phone's own storage on slot a; way back to Android slot b proven). UFS reads proven at HS_GEAR1 pinned and HS_GEAR3 unpinned up to 2 GiB (the old 'silent shell' was the matched image's own 180 s self-reboot, fixed in the procedure). Writes untried: that is stage 3. Phone is on Android slot b, nothing flashed or written, no go open, no keeper session live (all ended keepers archived). IN FLIGHT: one session only, builder local_2251b1fb (fresh, Sonnet 5.5, started 04:04 CET) on walleye-install-rw-write-procedure (rank 2), then walleye-go-is-consumed-by-its-run (rank 3); told to land, read back from main and report after each. LANDED TODAY: PRs 989 (go gate wants the full 64-hex sha256), 991 (lpoco-attended 0.45.2, 145 checks name research ids), 992/994/998/999 (my records), 993 (z21 recipe row, ensure serves it equal, proof test), 995 (fresh worktrees link build/images to walleye-build itself), 997 (kernel build walleye-ufs-off declared: two Nix builds equal, 88afb4d7...97ac, kept as research/walleye-bootimage/2026-11-24/kernel-build-row.toml, not yet in device.toml). WAITS: walleye-declares-its-kernel-build waits on walleye-lpoco-attended-carries-manifest-0-47 (lpoco-attended-v0.45.2 reads manifests with lpoco-manifest 0.46.0 which refuses [kernel_builds]; tag lpoco-manifest-v0.47.0 exists, no attended release carries it; asked of orchestration 2026-11-24, the Top says it follows deps-bump-releases-its-crate; then rhizoid's bump PR for tools/walleye-tries/Cargo.toml, then the builder puts the row into device.toml and runs tests/host/kernel-build.sh walleye-ufs-off); walleye-declares-ufs-on-kernel-build (rank 4) waits behind it; stage 3's image takes its kernel as a sha256 part meanwhile (as PR 993 does for z20) and goes through lpoco bringup image ensure; the declared ufs-on kernel replaces that part later. ORDER OF THE NEXT ITEMS (ONE session at a time, the author's rule 2026-11-24): (1) builder: stage 3 procedure, go-consumed gate; (2) keeper, one read-only Android visit combining walleye-gpt-hash-from-android-read (adb root; expect sda hash 6cd7e48c21bc7a1f2cea8c12dc773003ca71a8681f2b3f0c43f428d55d250174) and walleye-vendor-firmware-pull (copy only into ~/lab-material/walleye/vendor-firmware-DATE, mode 700, outside git, sha256sums), then walleye-wdt0-first-instant-read and walleye-cmdline-cut-check-slot-a (rank 5); (3) the stage 3 run (keeper, walleye-install-rw-write-test): needs a go whose OUTCOMES name the image by its FULL 64-hex sha256 (ikb backlog-go --id X --until T --outcomes ...), the keeper's landing ends the go (--until the run end, outcomes 'used by run N'); never bootloader partitions, GPT written by our code, erase, purge; (4) builder rank 3 items: walleye-check22-one-sided-same-disk, walleye-flaky-fixture-test, walleye-image-recipes-other-images, walleye-kernel-series-to-topics, walleye-stage2-way-back-reboot-twice, then walleye-checks-name-their-sources parts (a) host scripts' SOURCES.toml for tests/host/*.sh, (b) captured-text fixtures with provenance lines, (c) CI gate refusing unsourced checks, (d) finer per-check pass (stop checks first), then walleye-retire-build-scripts, walleye-procedures-on-v0-34; (5) research (Opus 5.5 only at a turning point; set with set_session_model id claude-opus-5-5): walleye-device-config-sources-read remaining, walleye-camera-camss-diff, walleye-pstore-t4-pon-warm-notifier, the watchdog items. THE ONE REPORT to the Steward (local_94eceeb7-ca01-4987-9041-6611635a1945) goes when walleye-checks-name-their-sources parts (a) to (c) have landed, nothing before. DECIDED, NOT YET RECORDED ELSEWHERE: (a) the author's rule 2026-11-24 (relayed by the Top): one agent at a time, no fan-outs, short briefs (item id), builders on Sonnet 5.5, keeper launcher refuses while a non-archived keeper ran in the last 3 h so archive each keeper (mcp__ccd_session_mgmt__archive_session) once its result is recorded; a builder with a very long context is archived and a fresh one started with run_scheduled_task walleye-builder, then messaged the item id; (b) the sign that land waited 72 min on a GitHub job with conclusion success but status in_progress (PR 993): re-run that job with gh run rerun RUN --job ID; told the lanes waker (local_7475f2b4), not an item anywhere; (c) ikb backlog needs takes only phone, research or build; (d) an ikb backlog rewrite replaces the whole item, so re-ranking is replayed with jq-read fields; (e) commit titles at most 100 characters, end with the Co-Authored-By line; generated-data conflicts on a keeper PR are replayed on fresh main and the PR closed. QUESTION FOR THE AUTHOR, via the Top, not yet asked: should Linux become the phone's default boot? It cannot be permanent until slot a is marked SUCCESSFUL (the bootloader falls back to b when tries run out); needs research on marking slot a successful from Linux, the bootloader's own set_active only with the conditions of decision 2026-11-22-walleye-author-research-then-proceed. Asked of orchestration and the ikb session, answers not yet seen: ikb backlog-go --image-sha256 option; the launcher refusing while ANY session of a task is not ended (the Top says done for keepers).

Considered
: Messaging the idle builder for the next items versus starting a fresh one: fresh chosen because its context (691+ messages) cost tokens on every turn. Whether the stage 3 image could wait for the kernel row: no, it takes the kernel as a sha256 part now.

Rejected
: Waiting for the lpoco-attended release before stage 3 (it would stall the phone lane; the sha256 part is the same shape as z21's). A second concurrent session (the author's one-at-a-time rule).

About
: walleye-bootimage

---


<a id="2026-11-24-host-scripts-name-their-sources-the-gate"></a>
### host scripts name their sources; the gate

Entry date
: 2026-11-24

Done
: tools/walleye-sources (argenv, 9 tests seen failing first), tests/host/SOURCES.toml for 98 scripts (74 sourced, 24 listed not yet sourced), check.yml job; programmer review fixes: empty reason, id-less data line, unreadable names refused

Considered
: a gate over procedures too (part 1's check_from already carries them)

Rejected
: marking the selftests sourced by the row of the thing they test: their fixtures are typed, so they are listed as not yet sourced

About
: walleye-bootimage

---


<a id="2026-11-24-rw-write-item-looked-at-and-split"></a>
### rw-write item looked at and split

Entry date
: 2026-11-24

Done
: read the stage 1 and 2 procedures, the production init and device.toml image rows; split walleye-install-rw-write-procedure into the image, the plumbing and the procedure file, each ranked 2 and blocked in order; walleye-go-is-consumed-by-its-run read: it needs a record of when a go was set, which the item row does not carry, so its design is a decision first

Considered
: building the procedure in one run: about 2500 lines of plumbing like rootfs_boot.rs plus a new image; a half-built one would fail lpoco-attended check

About
: walleye-bootimage

---


<a id="2026-11-24-banner-selftests-captured"></a>
### Banner selftests on text a phone printed

Entry date
: 2026-11-24

Done
: banner-selftest, banner-loop-selftest and banner-shell-selftest read tests/fixtures/phone-text/*.cap (the z6 and z5b banners the phone printed, copied from research); each failing case is a sed/awk/tac edit of that text; walleye-sources gates the fixtures folder in check.yml; 3 of 24 unsourced rows now sourced.

Considered
: A provenance line inside each .cap (it would be a line the phone did not print and breaks the banner checks) - a SOURCES.toml row per file instead, through the gate that already exists.

Rejected
: A new gate program for fixtures: walleye-sources already takes --dir, --table, --ext.

About
: walleye-bootimage

---


<a id="2026-11-24-rw-write-image"></a>
### The rw-write image

Entry date
: 2026-11-24

Done
: The production init got a walleye.rw switch (blocks that cut away to the production init); QEMU proof, build script, a recipe row, ensure equal.

Considered
: Where the petter's files live in a rw root: /.walleye as a tmpfs on one empty directory.

Rejected
: Anything that changes the ro image's ramdisk.

About
: walleye-bootimage

---


<a id="2026-11-24-rw-write-plumbing"></a>
### The rw write's plumbing

Entry date
: 2026-11-24

Done
: rw_write.rs with its judges and settings, tested from captured text; no phone

Considered
: Typing the dd and fsck output (refused: the write and its read-back run on the host on a known case; the fsck line is the one Android printed)

Rejected
: Writing the ssh/adb steps without a phone to test them against: left to walleye-install-rw-write-procedure

About
: walleye-bootimage

---


<a id="2026-11-24-rw-write-procedure"></a>
### The rw write procedure and its rehearsal

Entry date
: 2026-11-24

Done
: RootfsBoot got an rw mode; the entry and its rehearsal; the go item; dry runs of both; no phone.

Considered
: A separate step struct (copying the root boot's 2400 lines); an rw flag on the existing struct reuses every shared step

Rejected
: Writing fsck's clean line as an expectation: the step keeps the answer and says NOT KNOWN YET when the line is absent

About
: walleye-bootimage

---


<a id="2026-11-24-1400-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-24

Done
: rw-write rehearsal passed 15/15; the one run stopped at check 14 (no USB gadget 1d6b:0104), nothing written, every table, the boot chain and the flags equal, Android on slot b by itself; stock kept

Considered
: next: research reads the run folder ($AGENT_BUILD/walleye-rw) for why the gadget did not come up; no second try before it; remaining ready: gpt-hash-from-android-read and vendor-firmware-pull (one Android visit), wdt0-first-instant-read. build/images in a fresh worktree must link to ~/Projects/walleye-build (not its images/ folder)

About
: walleye-bootimage

---


<a id="2026-11-24-walleye-research-why-rw-run-1-had-no-gadget"></a>
### walleye-research: why rw run 1 had no gadget

Entry date
: 2026-11-24

Done
: Read the rw init against the run's command line: the image (cb79c975, header read) carries walleye.booted-slot, the run boots it from memory on slot b with root system_a, the init's stage 2 guard refuses before any mount. Counted by tests/host/ufs-root-rw-init.sh (pass, case R3 is run 1's line). Phone read only (adb as shell: Android up 664 s, pstore not readable without root, not changed). Recorded research walleye-rw-run-1-refused-by-booted-slot, decision 2026-11-24-walleye-rw-run-2-after-image-without-booted-slot, build items walleye-rw-image-without-booted-slot and walleye-rw-write-reads-pstore-on-a-stop (walleye-rw-write-run now blocked by both), the failing test in rw_write.rs (ignored until the fix).

Considered
: The first ready row was walleye-device-config-sources-read, but its remaining parts wait for their steps; took the coordinator's newest research item, walleye-rw-run-1-why-no-gadget, placed by PR 1013.

Rejected
: adb root to read pstore: it changes adbd's state, and the boot's console was already gone (Android rebooted since).

About
: walleye-bootimage

---


<a id="2026-11-24-walleye-dmesg-clean-selftest-on-captured-phone"></a>
### walleye: dmesg-clean selftest on captured phone text

Entry date
: 2026-11-24

Done
: dmesg-clean-selftest.sh now copies the z7 (clean), z6 (smmu deferred, gpu_gx) and c2 (charger deferred) reads from research into tests/fixtures/phone-text, 7 cases pass; the typed cases split into dmesg-clean-typed-selftest.sh, listed not yet sourced; CI runs the captured selftest.

Considered
: cut the -110 and phandle cases or keep typed

Rejected
: keeping typed lines inside the sourced selftest: it would claim a phone printed them

About
: walleye-checks-name-their-sources

---


<a id="2026-11-24-walleye-rw-image-without-walleye-booted-slot"></a>
### walleye: rw image without walleye.booted-slot

Entry date
: 2026-11-24

Done
: Removed walleye.booted-slot from the rw image recipe, rebuilt it (sha256 1bfb3667...7bcf), un-ignored the guard test, added the declared-line case R1 to ufs-root-rw-init.sh seen failing on the old line.

Considered
: renaming the image (ufs-root-booted-slot-production-rw misleads now)

Rejected
: rename: every procedure, go item and record names it; the sha256 is what the go names

About
: walleye-rw-image-without-booted-slot

---


<a id="2026-11-24-walleye-the-rw-write-reads-the-pstore-on-a-stop"></a>
### walleye: the rw write reads the pstore on a stop

Entry date
: 2026-11-24

Done
: the_stop_in_the_console judge on four QEMU-captured init-line fixtures plus a phone-printed start line; way_back reads console-ramoops on Android and judges it, read only.

Considered
: waiting for a phone capture of a refusal

Rejected
: typing the lines: the fixtures are cuts of the real init's console; a phone pstore capture does not exist yet

About
: walleye-rw-write-reads-pstore-on-a-stop

---


<a id="2026-11-24-1525-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-24

Done
: rw write run 2: write of 16 MiB equal, kernel log clean, tables and boot chain equal, root changed; stopped at check 26 (first fastboot reboot stayed in fastboot, the way back's leg 1 brought Android back on slot b); stock kept, nothing flashed

Considered
: next: research judges fsck on a mounted root (free-count diffs) and the step 24 way back (fastboot reboot twice); a fsck of the unmounted partition from Android is the cleaner proof; remaining ready: gpt-hash-from-android-read and vendor-firmware-pull, wdt0-first-instant-read

About
: walleye-bootimage

---


<a id="2026-11-24-1540-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-24

Done
: Android slot b, stock kept, nothing flashed or written; one read-only visit did gpt-hash (equal), vendor-firmware-pull (136 files) and the unmounted fsck (rc 0, free counts wrong)

Considered
: next: research judges the stale free counts (e2fsck -fn after a clean rw mount+umount is the way to tell); remaining ready: wdt0-first-instant-read, cmdline-cut-check-slot-a, fastboot-reboot-twice-timed-try

About
: walleye-bootimage

---


<a id="2026-11-24-walleye-pmic-dmesg-selftest-on-captured-phone"></a>
### walleye: pmic-dmesg selftest on captured phone text

Entry date
: 2026-11-24

Done
: pmic-dmesg-selftest.sh now runs on the z7 try read (dmesg.txt, spmi-devices.txt copied to tests/fixtures/phone-text, rows in its SOURCES.toml) and cuts of it (each spmi device removed, dmesg cut short, files missing): 9 cases pass, a step of check.yml. The three typed faults (a reg-less node of ours, another address, a probe failure) moved to pmic-dmesg-typed-selftest.sh, listed not yet sourced. Gate: 80 sourced, 21 not yet sourced, 0 refused (same 21: one sourced, one typed split added).

Considered
: appending typed fault lines to the captured dmesg

Rejected
: appending typed lines to the captured text in the sourced selftest: it would claim a phone printed them

About
: walleye-checks-name-their-sources

---


<a id="2026-11-24-walleye-the-rw-write-leaves-the-bootloader"></a>
### walleye: the rw write leaves the bootloader itself and ends its root clean

Entry date
: 2026-11-24

Done
: Cause read from the code and run 2's kept text (bootloader-way-back.txt: bootloader reached, fastboot reboot follows): the watched child's way back step returns at once (the parent runs it after the child ends), so after step 24 nothing sent fastboot reboot and check 26 waited its 240 s in the bootloader; the parent's leg 1 then brought Android. the_phone_is_sent_back now sends fastboot reboot (Lab::fastboot_reboot, shared with the way back's leg 1) when the bootloader is reached; test seen failing first. The clean end of research walleye-rw-fsck-counts-stale-superblock: settle_rw runs SETTLE_COMMAND (lists the files the boot wrote, sync, remount ro, reads the root's mount line back), keeps settled.txt and says the remount rc and ro or not; a part that did not print is not known yet.

Considered
: a second sent step in the procedure (renumbers the 30 steps); making the child's way back step do the leg

Rejected
: changing the procedure's step list: the fix is where the bootloader is reached, one place for every procedure using the_phone_is_sent_back

About
: walleye-bootimage

---


<a id="2026-11-24-walleye-a-run-takes-its-image-from-image-ensure"></a>
### walleye: a run takes its image from image ensure, never a stale build file

Entry date
: 2026-11-24

Done
: host::the_image_is_ensured: when the declared image's file does not have the declared sha256, lpoco-bringup image ensure NAME names a store path (parsed from captured ensure text: research/walleye-bootimage/2026-11-23/ensure-z21.txt and the 0.48.4 dry-run line), its sha256 must equal the declared one, and it replaces the build file (copy beside, rename); anything else stops the run at the image check, before any phone step, naming both shas. Wired in RootfsBoot::the_image_is_the_declared_one for tries that set Settings.image_name (the rootfs boot and the rw write). 3 host tests and a step test; 398 tests pass, clippy clean.

Considered
: making the image path the store path in Settings (every later step then reads a file we do not own; the run image is also made beside the build file)

Rejected
: refusing only: the keeper would again mend the file by hand, as in run 2

About
: walleye-bootimage

---


<a id="2026-11-24-1700-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-24

Done
: Android slot b, stock kept; run 3 of the rw write passed 30/30 and the way back worked, the root stayed rw (remount ro Resource busy), e2fsck -fn from Android b one wrong free-blocks line; host lpoco-attended upgraded 0.44.3 to 0.45.4 (old binary kept as .before-0.45.4)

Considered
: next: research which process holds the rw root open before run 4; settled.txt's find -newer lists image files, not the writer

About
: walleye-bootimage

---


<a id="2026-11-24-rw-clean-end-researched"></a>
### rw clean end: researched, QEMU test fails today

Entry date
: 2026-11-24

Done
: Read the rw init, runlevels, inittab, settle_rw and run 3's settled.txt. Wrote tests/host/rw-root-clean-end.sh (QEMU), seen failing on today's runlevels with run 3's e2fsck signature, passing with mount-ro after killprocs, busy with mount-ro alone. Decision and build item recorded.

Considered
: capturing the write-open fds on the phone first

Rejected
: that capture as the first step: the Alpine path ends clean without naming the holder

About
: walleye-bootimage

---


<a id="2026-11-24-phone-state-selftest-banner"></a>
### phone-state-selftest on a captured banner, in CI

Entry date
: 2026-11-24

Done
: the gadget-and-beat case reads the banner line a phone printed (banner-z6-try1.cap); phone-state-selftest.sh is a step of check.yml (it ran in no job); its unsourced reason now says what stays typed: adb/fastboot lists, sysfs, other tty bytes. adreno-smmu-disabled-selftest and touch-selftest looked at: no phone print of a dtb or of touch input exists, so they stay unsourced.

Considered
: Capturing adb/fastboot device lines from research: none holds one (grep of research/ found no FA81N1A00790 device line).

Rejected
: Marking phone-state-selftest sourced: only one of its fixtures is captured.

About
: walleye-bootimage

---


<a id="2026-11-24-rw-end-through-openrc"></a>
### rw root ends through mount-ro and a last reboot service

Entry date
: 2026-11-24

Done
: runlevels: shutdown killprocs, mount-ro, savecache, walleye-end; init.d/walleye-end always reboots (bootloader first when asked, from /run/walleye); rw-root-clean-end.sh fails before, passes in 3 scenarios in QEMU. Image rebuild and rw_write.rs left.

Considered
: Making walleye-end depend on named services: 'after *' keeps it last whatever is added.

Rejected
: Putting walleye-reboot into the rootfs: the lab already pushes it, and a /run copy keeps the image unchanged by it.

About
: walleye-bootimage

---


<a id="2026-11-24-procedures-run-own-cli"></a>
### procedures run their own lpoco-attended

Entry date
: 2026-11-24

Done
: bin/attended, CI uses it, host test for no bare call; procedures/README says how.

Considered
: cargo run -p lpoco-attended from the steps crate: refused, the dependency is not a workspace member. A [[bin]] in walleye-tries re-exporting the CLI: the bin's main lives in lab-tools, not in the lib.

Rejected
: cargo install into ~/.local/bin: that is the host binary again.

About
: walleye-bootimage

---


<a id="2026-11-24-rw-clean-end-image-and-way-back"></a>
### rw image rebuilt with the clean end; the way back asks init to end

Entry date
: 2026-11-24

Done
: rw init installs walleye-end and the shutdown links; image 2477276882987c8e1330f56b91cb07bac2afdcbe97e7b03e72b127f2bb69b54f built twice equal, recipe ramdisk and sha256 in device.toml; rootfs_boot.rs the_clean_end_is_tried, 3 tests; QEMU tests follow the init's own file.

Considered
: Rewriting system_a whole (out of scope); a rootfs-only change (the root on the card is already written: only the init reaches it); shipping the service in the ramdisk as a separate file (the heredoc is compared byte for byte with the rootfs file by tests/host/ufs-root-rw-init.sh).

Rejected
: A remount inside the live ssh session (busy: petters, syslogd hold the root open).

About
: walleye-bootimage

---


<a id="2026-11-24-1800-ensured-path-reads-stdout-then-stderr"></a>
### ensured_path finds the store path when stderr's status line follows it

Entry date
: 2026-11-24

Done
: Run 4's rehearsal stopped at check 2 (image ensure named no store path): lpoco-bringup prints the path on stdout and 'in the store' on stderr, host::run joins stdout then stderr, ensured_path took only the last line. Test first (failed, then passed), fix: the last bare absolute path line; nothing booted or written

Considered
: rejected: running ensure by hand and copying the file past the gate; next: rehearsal and run 4 again under the same go

About
: walleye-bootimage

---


<a id="2026-11-24-1830-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-24

Done
: Android slot b, stock kept, nothing written or flashed; run 4 of the rw write stopped at check 14 (no gadget): the new image's header carries walleye.booted-slot, which its recipe lacks; PR 1038 fixed ensured_path (stdout then stderr) earlier in the run

Considered
: next: research why the built rw image keeps the control's walleye.booted-slot; rebuild and check header equals recipe before run 5

About
: walleye-bootimage

---


<a id="2026-11-24-the-gate-reads-procedures-too-every-check-from"></a>
### The gate reads procedures too: every check_from cites a row that exists

Entry date
: 2026-11-24

Done
: walleye-sources --procedures DIR reads each procedures/*.rs as tokens (comments, strings, char and raw literals are not code): every check_from(..) (172, over several lines too) must have a string-literal second argument citing a research, source or file row that exists; a bare check(..) is refused; the option stands alone (exit 2 with --dir/--table/--ext); a step of check.yml. 19 tests, the new ones seen failing first; the review's findings (second literal vs second argument, comments, char literals, methods) each have one.

Considered
: a procedures/SOURCES.toml table beside the sources in the code

Rejected
: procedures/SOURCES.toml: the source already sits in the call (check_from), a second table would drift from it

About
: walleye-bootimage

---


<a id="2026-11-24-the-rw-image-is-its-recipe-s-07df9842-header"></a>
### The rw image is its recipe's: 07df9842, header without walleye.booted-slot

Entry date
: 2026-11-24

Done
: Run 4 stopped because 2477...b54f carried walleye.booted-slot in its header, which the recipe lacks. The build script now takes the recipe's cmdline; image ensure built 07df9842b0bdd4fe22b1277fff99ca340742ca7ada3d76c260ea6a5a8cc9dd8a from the recipe, equal twice; the host check of the file's header and check 2 refuse a header unequal to the recipe's, each seen failing on 2477 first. Nothing on the phone.

Considered
: compare only the sha256 (what check 2 did); build the image by the script and declare its output (what let 2477 through)

Rejected
: declaring the build script's output: the recipe must be what makes the image, so ensure builds it and Nix refuses any other bytes

About
: walleye-bootimage

---


<a id="2026-11-24-1920-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-24

Done
: Android slot b, stock kept; rw write run 5 passed 30/30, the init's end reached fastboot by itself, e2fsck -fn on unmounted system_a clean (stage 3 passes); system_a now holds one removed 16 MiB test file's history, nothing flashed

Considered
: next: remaining ready: wdt0-first-instant-read, cmdline-cut-check-slot-a, fastboot-reboot-twice-timed-try; pstore still not a proven console instrument

About
: walleye-bootimage

---


<a id="2026-11-24-check-22-one-sided-same-disk"></a>
### check 22 one-sided same-disk

Entry date
: 2026-11-24

Done
: Narrowed check 22 to boot_a/boot_b one-sided; never-list in device.toml; tests first (seen failing), programmer review addressed (doc, accepted whole-byte exemption).

Rejected
: masking bits; typed list

About
: walleye-bootimage

---


<a id="2026-11-24-way-back-sends-fastboot-reboot-twice"></a>
### way back sends fastboot reboot twice

Entry date
: 2026-11-24

Done
: Lab::way_back: a second bounded fastboot reboot when the phone still lists in fastboot, each leg timed; tests first (seen failing), review addressed.

Rejected
: a third reboot (the item says one more); a new reboot function in flash_boot.rs (the shared way back in lab.rs serves both stages)

About
: walleye-bootimage

---


<a id="2026-11-24-a-go-ends-with-its-run"></a>
### a go ends with the run that used it

Entry date
: 2026-11-24

Done
: tools/walleye-tries: ufs::go_gate refuses a spent go, ufs::spend_go writes the exclusive marker right before the boot or flash of the ufs, rootfs and flash procedures; tests first (compile-failing), then a flash walk of two runs on one go.

Considered
: reading the run's research row or session-log entry after the go (the item's proposal)

Rejected
: the research-row compare: the row has no go-set time and research dates are days; a later run on the same day cannot be told from an earlier one. A marker per (item, go_until) says it exactly.

About
: walleye-bootimage

---


<a id="2026-11-24-walleye-coordinator-handover-evening"></a>
### Walleye coordinator hand-over, 2026-11-24 evening: stage 3 passed; builders on rank-3 items

Entry date
: 2026-11-24

Done
: STAGE 3 PASSED: run 5 (PR 1046) wrote 16 MiB to system_a through our kernel, read back equal, the root ended clean through Alpine's shutdown (mount-ro, walleye-end), e2fsck -fn from Android clean; tables and boot chain equal. Image 07df9842...dd8a, built by image ensure from its recipe (PR 1044). Runs 1 and 4 stopped on walleye.booted-slot in the image header (PRs 1012, 1039); run 3 on a busy remount (PR 1027). Every go is closed. Also landed: pd-mapper config (1052), flaky fixture fix (1051), check 22 never-list in boot-chain.toml (1053), way back reboots twice (1054), go spent-marker and kernel fork tags with no branch deleted (1055; the Top's rule: tip tag and git cherry first), recipes l7/b1/b2 (1056). IN FLIGHT: builder local_02098168 (walleye-image-recipes-other-images in-progress: wdt1-a/b, b-dN left); archive it when idle. WAITS: walleye-ci-image-ensure-rebuild on lab-tools image-rebuild-changed-since; proof (1) of walleye-go-is-consumed-by-its-run, a line in docs phone-keeper.md, sent to orchestration who owns the briefs; walleye-checks-name-their-sources: 21 host scripts need text a phone printed (a keeper capture item still to write). NEXT: keeper items wdt0-first-instant-read (its test still to write), cmdline-cut-check-slot-a, fastboot-reboot-twice-timed-try; kernel-series-to-topics step 5. SIGNS: launched builders take their launcher's top row before the coordinator's message arrives; keepers left used gos open; pgrep -f wait loops match themselves (two ended by pid); the app's cross-session send cap paused this coordinator's messages until the user types. The morning question for the author (Linux as default boot) is still not asked.

Considered
: Reusing the unarchivable run 3 keeper for runs 4 and 5 rather than starting a second keeper.

Rejected
: Deleting fork branches without a tip tag and a cherry check (the Top's condition).

About
: walleye-bootimage

---


<a id="2026-11-24-image-recipes-b-dn"></a>
### Recipes for the five delay images

Entry date
: 2026-11-24

Done
: b-d5..b-d50 each a recipe of control b2 plus a ramdisk sha256; image ensure --rebuild equals each declared sha256; image-recipe.sh passes. wdt1-a/b left: their kernel, dtb and cmdline differ from b2's.

Considered
: Giving wdt1-a/b a recipe now: no control image shares their kernel and dtb

About
: walleye-bootimage

---


<a id="2026-11-24-image-recipes-b-sqr"></a>
### Recipes for b-s5, b-r5, b-q5

Entry date
: 2026-11-24

Done
: Three recipes (control b-d5; dtb for b-s5 and b-r5, ramdisk for b-q5), each rebuilt equal by image ensure --rebuild; image-recipe.sh passes for each.

Considered
: wdt1-a/b and the alpine/ufs images still need parts as files of the manifest

Rejected
: A recipe with the unpacked kernel file plus the dtb: it appends the dtb twice and does not equal the declared sha256 (measured)

About
: walleye-bootimage

---


<a id="2026-11-24-wdt0-read-image-built"></a>
### WDT0-read image built from its recipe

Entry date
: 2026-11-24

Done
: walleye-builder: build/initramfs-root-wdt0/init (production init plus walleye.wdt0 blocks: the four WDT0 registers read first, read only, kept in the kernel log and /.walleye/wdt0, /run/walleye/wdt0), tests/host/wdt0-init.sh (seen failing on the production init), build/build-ufs-root-wdt0.sh (21 checks), the image and its recipe in device.toml, rebuilt equal by image ensure. Item walleye-wdt0-read-image done; walleye-wdt0-try-procedure opened for the keeper's one command.

Considered
: A word-less init that always reads; reading from a kernel module or earlycon.

Rejected
: Always-on read (the production init stays untouched without the word, like walleye.rw); pstore as the only place (not a proven console here); the cmdline with walleye.booted-slot (the init refuses it for a from-memory boot on the other slot).

About
: walleye-bootimage

---


<a id="2026-11-25-wdt0-try-procedure"></a>
### The WDT0 try as one command

Entry date
: 2026-11-25

Done
: Built the WDT0 try on the root boot's steps (RootfsBoot::wdt0 mode, Settings Try::Wdt0, device.toml tables for the try and its rehearsal), a judge in src/wdt0.rs, two procedures. Tests 430 pass, check and dry run pass; the phone was not touched.

Considered
: A new RootfsBoot-free procedure; a separate read after the login

Rejected
: Copying the root boot into a new module (3000 lines to keep equal): one mode of the same steps instead, as the rw write did. Reading the registers over devmem from the lab: the init already reads them first, before anything can move them.

About
: walleye-bootimage

---


<a id="2026-11-25-0105-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Android slot b, stock kept; nothing flashed. Rehearsal of the WDT0 try passed 15/15, then the real run (go used by run 1): booted image 17a9c21d once from memory, kernel log shows EN 0, STS 0, BARK 0x7FFFF, BITE 0xFFFFF; run stopped at check 17 (/run/walleye/wdt0 lost at hand-over), way back by fastboot reboot, Android in 31 s; idle-android pass, battery 4375 mV. Go closed.

Considered
: next: builder fix of the file's survival (or check 17 reading dmesg) and its rehearsal; remaining ready: cmdline-cut-check-slot-a, fastboot-reboot-twice-timed-try. No step claim: the proof did not pass as written.

About
: walleye-bootimage

---


<a id="2026-11-25-kernel-topics-read-back"></a>
### Kernel topics item: what the fork shows today

Entry date
: 2026-11-25

Done
: Read the fork linux-phone-oco/linux with git ls-remote on 2026-11-25: tag archive/ufs-hynix-quirks at d49d54405 is there, no branch topic/ufs-hynix-quirks or archive/ufs-hynix-quirks remains (the archive/ufs-hynix-quirks-d49d54405 branch is the one left); live tags int/walleye/ufs-off/v7.3/1 and int/walleye/ufs-on/v7.3/1 and both frozen tags are pushed; device.toml carries kernel_tag on the ufs-off, ufs-on and rw-write rows. So the hynix archive job, the live lists' tags and the kernel_tag lines are done. Left of walleye-kernel-series-to-topics: step 5 (lab-only dts applied after assembly, an image rebuilt byte for byte from its tag) and the retirement of build-*.sh, which are the items walleye-declares-ufs-on-kernel-build, walleye-image-recipes-other-images and walleye-retire-build-scripts; the item stays open for them. No kernel built in this run.

Considered
: Starting a kernel build for step 5 now: it is the same work as walleye-declares-ufs-on-kernel-build (the [kernel_builds] row and two equal builds), so one builder does it once, from that item.

Rejected
: Marking the item done: step 5 and its proof (an image equal from its tag) are not shown yet.

About
: walleye-kernel-dts

---


<a id="2026-11-25-wdt0-check17-from-kernel-log"></a>
### WDT0 check 17 judges the kernel log when the file is empty

Entry date
: 2026-11-25

Done
: wdt0::judge falls back to the kernel log's readings when /run/walleye/wdt0 is empty; the four values are kept in wdt0.txt. Test on run 1's nine printed lines (fixtures/wdt0-run1-dmesg.txt), seen failing first; all 433 walleye-tries tests, clippy, fmt pass.

Considered
: Making the init keep the readings where the root's /run does not hide them: needs a new image and a phone run.

Rejected
: Changing the init: a rebuild plus a second boot for a file the kernel log already carries; the log is the phone's own printed text. A file that holds readings must still equal the log.

About
: walleye-bootimage

---


<a id="2026-11-25-0200-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Android slot b, stock kept; nothing flashed or booted. Read-only Android captures for walleye-phone-text-captures landed as fixtures (13 sourced). Item stays open: our kernel's charger/fuel-gauge text needs a stage 2 boot.

Considered
: next: walleye-phone-text-captures kernel half via a rehearsed stage 2 read run; check 17 fix still with builder; ready: cmdline-cut-check-slot-a, fastboot-reboot-twice-timed-try

About
: walleye-bootimage

---


<a id="2026-11-25-kernel-text-capture-step"></a>
### Kernel text capture step in the rootfs-boots procedure

Entry date
: 2026-11-25

Done
: Added our_kernels_text_is_captured (step 20) and a fixtures key; proof test seen failing first, then 436 pass. Reviewed by the programmer subagent; fixed: the empty-glob case, the run image's sha256 in provenance, one dmesg read (the judged one), dead device.toml keys.

Considered
: Writing SOURCES rows from the step; leaving that to the landing of a real run's folder.

Rejected
: A second dmesg read over ssh: unjudged text under a clean name. Typed stand-in fixtures: the practice forbids them.

About
: walleye-bootimage

---


<a id="2026-11-25-0405-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Android slot b, stock kept; nothing flashed or booted. Rehearsal of rootfs-boots passed (all steps, card/chain/flags equal). The real run via bin/attended run stopped at 'the author's go is recorded on the item': item walleye-install-rootfs-boots-try has go_until 2026-11-25T07:00 but image_sha256 NULL; the run image is c3373d7199d26a403a7ac9b10c8c95f4dd17f41d358eb59de095efd6bb6c7ba9. Procedures run by bin/attended run NAME (real ones are not executable; rehearse-* are). Stop read, nothing narrowed.

Considered
: next: coordinator records the go naming that sha256 (ikb backlog-go --image-sha256 ...) on walleye-install-rootfs-boots-try, then walleye-kernel-text-capture-run runs with one command: bin/attended run rootfs-boots-on-the-pixel-2

About
: walleye-bootimage

---


<a id="2026-11-25-0500-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Ran rootfs-boots-on-the-pixel-2 under its go: kernel text fixtures captured (ufs-root power-supply and dmesg). Phone on Android slot b, stock kept, healthy, adb lists it; nothing flashed. Used by run 1. Mistake: my timeout 580 cut the wrapper at check 25 (exit 143); post-stop measurement showed everything as before.

Considered
: next: walleye-phone-text-captures can take the new fixtures into the selftests (charger/fuel-gauge/part-compare text); then walleye-cmdline-cut-check-slot-a. Run attended commands without a short timeout (about 11 min).

About
: walleye-bootimage

---


<a id="2026-11-25-walleye-land-check-rebuild"></a>
### land-check rebuilds touched images

Entry date
: 2026-11-25

Done
: Added make land-check (LAND_BASE default origin/main), bin/install-bringup.sh (pinned 0.51.1, checksum-checked) and tests/host/land-check-rebuild.sh; the test failed before the target and passes after.

Considered
: Using lpoco-bringup from PATH: no pin, the host copy moves with the lab. Running the rebuild in CI: no nix there; land runs on the lab.

Rejected
: The host binary (unpinned); a stand-in sha256 mismatch text (the refusal line is captured from the real tool).

About
: walleye-bootimage

---


<a id="2026-11-25-host-selftests-onto-captured-phone-text"></a>
### Host selftests onto captured phone text

Entry date
: 2026-11-25

Done
: charger-selftest and fuel-gauge-selftest now run on the ufs-root dmesg and power-supply captures and Android b's; typed cases split to charger-typed-selftest and fuel-gauge-typed-selftest (wired into check.yml); bootchain-readback-selftest takes its by-name table from the Android b capture; SOURCES rows updated, walleye-sources gate 0 refused.

Considered
: Moving part-compare and display-late-probe: no captured text of their reads exists.

Rejected
: Typing a qcom-battery block as a stand-in: the brief forbids a typed stand-in for a live-phone check; left typed and named missing instead.

About
: walleye-bootimage

---


<a id="2026-11-25-retire-nine-build-scripts"></a>
### Nine build scripts retired

Entry date
: 2026-11-25

Done
: Ran image ensure for every recipe image (all equal); removed the nine scripts those images name; reworded comments; item stays in-progress

Considered
: judging all 99 scripts in one change

Rejected
: removing a script whose image has no equal rebuild

About
: walleye-kernel-dts

---


<a id="2026-11-25-0700-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Phone on Android slot b, stock kept, adb lists it, nothing flashed or set this run. Took walleye-cmdline-cut-check-slot-a (first ready row) and read it: C1 needs the slot flag set to a, a fastboot boot and a read, with no prepared one-command procedure (rootfs-boots-on-the-pixel-2 does not carry it); typing it by hand is refused by the keeper rule. The next ready item, walleye-phone-text-captures-rest, needs our kernel's qcom-battery text from an fg1 image boot and a root /proc/partitions: no prepared procedure either, Android's /proc/partitions is permission denied.

Considered
: next: a builder adds a procedure step (or rehearsal+run pair) for C1 and a fg1-image variant of the rootfs-boots capture, then the keeper runs each as one command. Until then the ready queue has no item that is one prepared command.

About
: walleye-bootimage

---


<a id="2026-11-25-walleye-builder-cmdline-cut-c1-procedure"></a>
### walleye builder: the C1 procedure and its rehearsal

Entry date
: 2026-11-25

Done
: Built check C1 as one command (procedure and rehearsal) on the root boot's shared steps plus tools/walleye-tries/src/cmdline_cut.rs: judge/cmdline_in tested on captured text (Android b's whole line; the captured RAM-boot dmesg line with its lab-added suffix removed - no full slot-a cut line was ever captured), the debug-image header check, device.toml tables, check and dry-run pass.

Considered
: Composing RootfsBoot (shared steps) rather than a new mode flag in it: only go_is_open_now widened to pub(crate).

Rejected
: The production image (no telnet), a slot on the header, a plain reboot as the way back - see decision 2026-11-25-c1-reads-the-debug-image-and-goes-back-through-the-bootloader.

About
: walleye-bootimage

---


<a id="2026-11-25-walleye-debug-image-recipe"></a>
### Recipe for the stage 2 debug image

Entry date
: 2026-11-25

Done
: [images.ufs-root-booted-slot-debug.recipe] (control ufs-root-booted-slot, ramdisk 1fe3f95d, the 149-character cmdline); ensure rebuilds a0a55cd0 equal; image-recipe.sh passes; nc and sha256sum are in the ramdisk's busybox.

Considered
: Rebuilding the ramdisk with build/build-ufs-root-image.sh DEBUG_INIT=yes: not needed, the 2026-11-23 image's own ramdisk is on disk and unpacks exactly

Rejected
: Copying the image into the store by hand: ensure with a seed does it and checks the sha256

About
: walleye-bootimage

---


<a id="2026-11-25-0830-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Phone on Android slot b, stock, nothing flashed or set. Took walleye-cmdline-cut-check-slot-a (go open until 11:06). Ran its rehearsal (procedures/rehearse-cmdline-cut-c1-on-the-pixel-2.rs): checks 1-6 ok; check 7 (card read from Android) stalled: adb read of /dev/block/by-name/system_a got 92545020 of 2684354560 bytes in the 760 s bound (~120 KB/s), adb killed, retried; my own timeout 900 then ended the run (exit 143). Way back: phone already on Android. The real run was NOT started: the rehearsal did not pass. The go on the item is NOT used (no run N), left open.

Considered
: next: read why the adb pull of system_a is ~120 KB/s (usb link, adb on the host, adbd root dd vs pull) before any change; a builder may bound the root-partition hash by reading on the phone (sha256sum there) instead of through adb; rerun the rehearsal without an outer timeout shorter than its own bounds (it needs more than 900 s when it retries).

About
: walleye-bootimage

---


<a id="2026-11-25-card-read-hashed-on-the-phone"></a>
### Card read hashed on the phone

Entry date
: 2026-11-25

Done
: walleye-tries android.rs sha256 now runs dd | sha256sum on the phone and reads one digest line; tests (443) pass; item walleye-card-read-slow queued for the keeper (needs phone) with its rehearsal as proof; cause of the slow stream not measured

Considered
: reading the media speed first with a phone-side dd to /dev/null: needs the phone, a builder never touches it; left to the keeper's rehearsal

Rejected
: raising the 760 s bound: hides a stream that varies 70x; a bigger bound would only make the keeper wait longer

About
: walleye-bootimage

---


<a id="2026-11-25-0850-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Phone on Android slot b, stock, nothing flashed or set. Took walleye-card-read-slow: rehearsal of C1 ran with the root hash made on the phone, 14/14 passed, check 7 in 12 s (was 760 s+ over adb); item done. Way back leg 1 31 s. The go on walleye-cmdline-cut-check-slot-a (until 11:06) is still open and unused; the real C1 run was not started - one piece per keeper.

Considered
: next: walleye-cmdline-cut-check-slot-a real run under its open go (rehearsal now passes); then walleye-transport-real-phone-run (rank 15). Open: why the adb stream stalls (adbd or host server), only if the old path is used again.

About
: walleye-bootimage

---


<a id="2026-11-25-0935-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Phone on Android slot b, stock, nothing flashed; card, boot chain and flags equal to before (rehearsal read after). Ran C1 under its go (used by run 1, go closed): CUT at androidboot.pk=g-pxl-p on slot a (research walleye-cmdline-cut-c1-slot-a-cuts), item done. The run's way back stopped at step 16 (nc job control, backlog walleye-reboot-send-nc-job-control); I finished the same steps by hand over telnet and lpoco lab fastboot set-active/reboot, no hands needed.

Considered
: next: C3 (the bootloader cuts the tail on slot a whatever our header) is for the coordinator to rank; then walleye-reboot-send-nc-job-control fix (lab builder) before any run that boots the debug image again; then walleye-transport-real-phone-run (rank 15).

About
: walleye-bootimage

---


<a id="2026-11-25-nc-job-control"></a>
### nc listener without job-control stop

Entry date
: 2026-11-25

Done
: Shell::background and listen_command in walleye-tries; proof tests seen failing then passing; programmer review read

Considered
: run between markers with </dev/null; a netstat wait instead of the 1 s sleep

Rejected
: marker-wrapped '&' line: invalid shell syntax

About
: walleye-bootimage

---


<a id="2026-11-25-fastboot-reboot-twice-procedure"></a>
### Procedure and rehearsal for the timed fastboot excursion

Entry date
: 2026-11-25

Done
: tools/walleye-tries reboot_twice.rs, procedures/fastboot-reboot-twice-on-the-pixel-2.rs and its rehearse- pair, device.toml tables; 456 tests, clippy, bin/attended check and --dry-run pass (12 and 15 steps); each leg's line and time go to legs.txt and verdict.txt. Review addressed: refused call stops the run, no-continue said in the verdict, in_fastboot checked before each send, a test reading both entry files. No phone touched.

Considered
: a fake adb test of the continue-from-Android branch (not done: the branch is the shared phone_is_in_fastboot, rehearsed on the phone first)

Rejected
: a go for a try that boots no image

About
: walleye-bootimage

---


<a id="2026-11-25-1100-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Ran the timed fastboot excursion (rehearsal 12/12, run 15/15): fastboot reboot and fastboot continue each returned Android in 29-32 s; phone on Android slot b, stock, nothing flashed or set; card, boot chain, flags equal. Item walleye-fastboot-reboot-twice-timed-try done. Not run: no go was involved.

Considered
: next: walleye-reboot-send-nc-job-control phone proof rides on the next debug-image run; C3 for the coordinator to rank; walleye-transport-real-phone-run (rank 15)

About
: walleye-bootimage

---


<a id="2026-11-25-phone-text-from-material"></a>
### Phone text captures: found already printed, fixtures made

Entry date
: 2026-11-25

Done
: Read the item, the rootfs-boots capture step and the lab material; found fg1's qcom-battery text and Android's root /proc/partitions already captured; made three fixtures with SOURCES rows, moved fuel-gauge-selftest onto fg1, closed the procedure item as superseded, left the rest item with what is truly open.

Considered
: A fg1 capture variant of rootfs-boots (fg1 is a tty shell image, not the ssh root image); a /proc/partitions section in the root boot's capture.

Rejected
: A new boot procedure for text that exists: it would cost a go and a phone run to print what the lab already holds.

About
: walleye-bootimage

---


<a id="2026-11-25-1230-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Android b part-compare touch capture read over adb (read only) and kept as fixtures; part-compare-selftest moved onto it; item walleye-phone-text-captures-rest done. Phone on Android slot b, stock, nothing flashed or set. No go was involved.

Considered
: next: walleye-reboot-send-nc-job-control phone proof rides on the next debug-image run; walleye-display-late-answers-capture rides on the next late-probe run; walleye-transport-real-phone-run (rank 15)

---


<a id="2026-11-25-bootchain-selftest-on-the-real-proc-partitions"></a>
### bootchain selftest on the real /proc/partitions

Entry date
: 2026-11-25

Done
: bootchain-readback-selftest.sh takes every partition row and size from the Android b /proc/partitions capture (proc-partitions-android-b-2026-11-23.cap) instead of typed 8 KiB rows; the xbl case flipped to the phone's own 8104 sectors, seen failing first (xbl_a was 12288 bytes) then passing. The first ready row (walleye-checks-name-their-sources, part 7+) waits only on text no phone has printed (input-devices, events, typed fault cases) so nothing of it was buildable on the host; this ranked item was taken instead.

Considered
: Sourcing touch-selftest from the Android b touch capture: its input.txt is the vendor 'input-name=' form, not /proc/bus/input/devices, and no events were read, so the script's reader formats are not what the phone printed - stays unsourced.

Rejected
: Keeping a typed xbl 12 KiB row next to the real ones: the check's own size rule would then disagree with the phone.

About
: walleye-bootimage

---


<a id="2026-11-25-retire-ufs-root-scripts"></a>
### Two UFS-root build scripts retired

Entry date
: 2026-11-25

Done
: Recipes for ufs-root, ufs-root-booted-slot and production; ensure --rebuild equal for each; removed build-ufs-root-image.sh and build-ufs-root-production.sh; recipe selftests and the UFS gate selftest pass

Considered
: building the kernel from the tag in the sandbox (kernel_builds) for the production Image.gz

Rejected
: retiring build-ufs-match.sh: its image is the recipes' control and has no equal rebuild yet

About
: walleye-kernel-dts

---


<a id="2026-11-25-walleye-builder-cmdline-budget-check"></a>
### walleye builder: the recipe cmdline budget check

Entry date
: 2026-11-25

Done
: Built tools/walleye-cmdline-budget in Rust with argenv: it reads device.toml, measures each [images.*.recipe].cmdline against 160 and fails one over; 9 tests (seen failing against a stub first; the programmer review added the bootable, non-string and --max cases) run it on the real device.toml and on copies lengthened or given bad exceptions. It passes device.toml today (six recipes, the two old long ones excused as NOT-TO-BOOT). Run by make land-check and a CI job; programmer review before landing.

Considered
: A per-image limit in device.toml; a shell check

Rejected
: See decision 2026-11-25-recipe-cmdline-budget-is-a-rust-check-with-a-bootable-gate

About
: walleye-bootimage

---


<a id="2026-11-25-walleye-retire-ufs-root-display"></a>
### ufs-root-display image rebuilds from its recipe; its script retired

Entry date
: 2026-11-25

Done
: Unpacked U-d5 (a4558827) with lpoco bringup image unpack, split its kernel part into Image.gz (4b29511c) and the appended display dtb (8e9a161c), took ufs-match as control (461077869 equals its declared sha), added [images.ufs-root-display.recipe]; lpoco bringup image ensure ufs-root-display --seed built it equal to the declared sha256. Removed build/build-ufs-root-display-image.sh and lib-display-levers.sh (its only user), fixed the comments in device.toml, kernel/README.md and ufs-off-image-selftest.sh (still passes).

Considered
: Rebuilding the kernel from its wip branch: the image's kernel part is a recipe input like ufs-root's, the tag-less wip kernel is not republished.

Rejected
: Keeping lib-display-levers.sh: nothing sources it any more; git history keeps it.

About
: walleye-kernel-dts

---


<a id="2026-11-25-retire-the-display-lib-branch-of-build-initramfs"></a>
### Retire the display lib branch of build-initramfs.sh

Entry date
: 2026-11-25

Done
: Judged the DISPLAY_FILES lib/ branch of build/build-initramfs.sh (ufs-root mode): its only caller, build-ufs-root-display-image.sh, was retired, the tests pass only a list and a delay; removed it, kept the rest. tests/host/ufs-root-display-init.sh passes after the cut.

Considered
: Removing the whole DISPLAY_FILES block: the host tests of the display init still pack the list and delay through it.

Rejected
: Removing the whole block: ufs-root-init.sh and ufs-root-display-init.sh use it.

About
: walleye-kernel-dts

---


<a id="2026-11-25-walleye-slpi-carveout-dts-stage-1"></a>
### SLPI carve-out dts and its host test

Entry date
: 2026-11-25

Done
: dt: slpi_mem 0x94c00000/15 MiB, mba 0x94500000, venus 0x94700000, mainline's ipa/gpu/wlan nodes deleted (wifi's memory-region dropped), ipa_gpu_mem 0x95b00000, remoteproc_slpi okay with slpi_v2.mdt. tests/host/dtb-slpi-carveout.sh (+selftest, 6 known bad cases) against the phone's own fdt text (fixture). Compiled with cpp+dtc against the ufs-on kernel tree. Found the shipped kernels have the PAS driver as a module: image split to walleye-slpi-image-kernel-pas.

Considered
: Checking DRAM from the board dts: its /memory has size 0 (the bootloader fills it), so the phone's own /memory is the bound.

Rejected
: Building the image on the shipped kernel: no PAS driver built in, no modules in the ramdisk.

About
: sensors

---


<a id="2026-11-25-ufs-on-kernel-build-declared"></a>
### ufs-on kernel build declared

Entry date
: 2026-11-25

Done
: device.toml [kernel_builds.walleye-ufs-on], two builds equal, test passes, no phone

Considered
: reusing the ufs-off config hash

Rejected
: ufs-off row's 5424de9b: stale against today's walleye.config

About
: walleye-bootimage

---


<a id="2026-11-25-walleye-builder-slpi-pas-config"></a>
### SLPI image: the PAS options =y, proved on the host

Entry date
: 2026-11-25

Done
: Wrote tests/host/kernel-slpi-pas.sh (seen failing), then set QCOM_Q6V5_PAS, QCOM_SYSMON, RPMSG_QCOM_GLINK_SMEM =y in build/walleye.config; the merged defconfig reads them and QCOM_MDT_LOADER back =y. Item stays in-progress: the int tag, kernel build, dtb and image remain.

Considered
: Modules in the ramdisk instead of =y (needs the tag's own module tree; the research left it not known).

Rejected
: QCOM_PDR_HELPERS =y: selected only by PMIC_GLINK and APR, no user under drivers/remoteproc in v7.3, so the research's list was wider than the SLPI start needs.

About
: sensors

---


<a id="2026-11-25-walleye-builder-slpi-image"></a>
### SLPI image built

Entry date
: 2026-11-25

Done
: device.toml [kernel_builds.walleye-ufs-off-slpi] and [images.slpi] with their recipes, build/initramfs-slpi spec; kernel built twice equal, image rebuilt equal, host proofs 1-5 pass; no phone

Considered
: a new int tag (assemble refuses: untagged topic commits); modules in the ramdisk (a build of the tag's module tree)

Rejected
: a new tag: the tag already holds the list and only the config changes; modules: the options =y need none

About
: sensors

---


<a id="2026-11-25-1500-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Phone on Android slot b, stock, nothing flashed or set; adb up. Took walleye-slpi-boot-qrtr (rank 4, ready): its image is built (sha 7c7dff61...) but NO procedure exists for it (procedures/ has none; tests/health/base/slpi-sensor-service.sh needs qrtr-lookup the ramdisk lacks), so no prepared one-command run to make; I typed no steps by hand. The other ready items (reboot-send-nc, display-late-answers) ride on the next debug/late-probe run; transport-real-phone-run is rank 15, read-mostly, not started.

Considered
: next: a builder writes procedures/slpi-boot-on-the-pixel-2.rs (gate, fastboot boot of [images.slpi] from memory, read remoteproc state + kernel log, way back to Android b) and a qrtr-lookup in the ramdisk; then the keeper runs it. Meanwhile walleye-transport-real-phone-run is the next runnable item.

About
: walleye-bootimage

---


<a id="2026-11-25-walleye-builder-slpi-qrtr-lookup"></a>
### SLPI ramdisk: qrtr-lookup, image rebuilt

Entry date
: 2026-11-25

Done
: build/initramfs-slpi/spec lists /bin/qrtr-lookup (static aarch64 from the pinned qrtr, qrtr-lookup.sha256, tests/host/qrtr-lookup.sh seen failing first); ramdisk 59d0cb60 and image slpi b9819427 rebuilt equal, gates pass; procedure not built (item stays open); no phone

Considered
: dynamic musl qrtr-lookup from the Alpine build tree (needs libc in the ramdisk); a Rust qrtr client (no musl target, a new program for a ten-line tool); the whole procedure in one run

Rejected
: the Alpine one: dynamic; a Rust rewrite: a second implementation of the pinned tool; the whole procedure now: its plumbing (settings, module, rehearsal) is a landing of its own, so it is left as the named remainder

About
: sensors

---


<a id="2026-11-25-walleye-builder-slpi-boot-procedure"></a>
### SLPI boot procedure and its rehearsal

Entry date
: 2026-11-25

Done
: slpi_boot.rs (SlpiBoot: the shared root-boot steps plus the image gate, the boot from memory, gadget-up, one telnet read judged into up/offline/crashed/absent and a qrtr service count, the answer kept), Try::SlpiBoot settings, the two device.toml tables, the two entry files; the judge tested on a captured kernel log (negative) and on the kernel's source words (positive, named in research walleye-slpi-boot-judged-words); lpoco-attended check --host and the whole crate's tests pass; no phone touched

Considered
: a go on the item (the touch procedure on z21 reads none: UFS off by the gate and RAM-only are the guard); reading the remoteproc over ssh (the image has no sshd, telnet is its shell); failing the run when the SLPI is not up (the answer is what the try is for)

Rejected
: a typed stand-in for a phone's SLPI text as a fixture: the positive case names the kernel's source format instead and says it is not captured; a slot switch like C1: nothing here needs the other slot, Android's slot stays current so a plain reboot returns to Android

About
: sensors

---


<a id="2026-11-25-1800-how-the-phone-stands"></a>
### How the phone stands

Entry date
: 2026-11-25

Done
: Phone on Android slot b, stock, nothing flashed or set; adb up. Ran walleye-slpi-boot-qrtr: rehearsal 13/13, then the real run 18/18: SLPI up from the phone's own firmware, 23 qrtr services; text kept in research/walleye-slpi-boot-qrtr/. No bring-up step claimed: remoteprocs-running needs adsp, cdsp, modem (slpi is not among them). No go was open on the item.

Considered
: next: move the SLPI judge tests onto the captured text and run slpi-sensor-service.sh; identify the sensor services (ids 349-362) by a builder; then walleye-transport-real-phone-run (rank 15) or the next debug-image run for reboot-send-nc and display-late-answers.

About
: walleye-bootimage

---


<a id="2026-11-25-slpi-judges-on-captured-text"></a>
### SLPI judges on the phone's captured text

Entry date
: 2026-11-25

Done
: Kept the SLPI boot read and dmesg as fixtures (slpi-boot-qrtr-*.cap, SOURCES rows); two Rust tests judge the read (running, 23 services, verdict equal to the run's verdict.txt) and the whole log (slpi started and up, no failure); slpi-sensor-service.sh takes HEALTH_LIB and REMOTEPROC_ROOT, checks the remoteproc named slpi, and skips services it cannot name; new tests/host/slpi-sensor-service-selftest.sh built from the capture, seen failing first.

Considered
: naming ids 349-362 as the sensor service by guess

Rejected
: Treating any instance-50 service as the sensor service: no source names them. A stand-in qrtr table typed by hand: the capture is used.

About
: sensors

---


<a id="2026-11-25-recipe-b-a5"></a>
### Recipe for b-a5

Entry date
: 2026-11-25

Done
: [images.b-a5.recipe]: control b-r5, ramdisk 55c1145e; ensure --rebuild equal, image-recipe.sh passes; wdt1-a/b and the alpine-rootfs/ufs images left

Considered
: wdt1-a/b: own kernel, not touched this run

About
: walleye-bootimage

---


<a id="2026-11-25-image-recipes-wdt1"></a>
### wdt1 recipes rebuild equal, blocked by the cmdline budget

Entry date
: 2026-11-25

Done
: Rebuilt wdt1-a and wdt1-b equal from recipes (research row, rows kept in its README); land-check refused them: 168-byte command line over 160; device.toml unchanged.

Considered
: Excusing wdt1-a in the over-budget table: refused, only a NOT-TO-BOOT image may be excused.

About
: walleye-bootimage

---


<a id="2026-11-25-wdt1-pair-shorter-dyndbg"></a>
### wdt1 pair rebuilt with the short dyndbg

Entry date
: 2026-11-25

Done
: wdt1-a/b recipe rows in device.toml (control z20 and wdt1-a, the shipped parts by sha256, cmdline with dyndbg file pm8916_wdt.c, 151 bytes), sha256s 7630009d and 4a4ae255 from image ensure, equal twice; image-recipe.sh and the cmdline budget pass. Not booted.

Considered
: dyndbg matches by basename (kbasename in lib/dynamic_debug.c), on-phone print not known yet

Rejected
: Excusing wdt1-a in the over-budget table, retiring the pair (decision 2026-11-25-wdt1-pair-shorter-dyndbg)

About
: walleye-bootimage

---


<a id="2026-11-25-wdt1-pair-at-declared-path"></a>
### wdt1 pair put at its declared path

Entry date
: 2026-11-25

Done
: Ran lpoco-bringup image ensure wdt1-a and wdt1-b (both equal to the declared sha256 7630009d / 4a4ae255); moved the old 168-byte pair into images-wdt1-sysfsless/old-168-byte-cmdline/, copied the ensured pair to the declared path, wrote SHA256SUMS-wdt1 from sha256sum, checked against device.toml. Nothing flashed, no phone touched.

Considered
: next recipes: z20, l6, ufs-match, alpine-rootfs*, b-r2, b-rp2, walleye-reboot (no recipe row yet)

About
: walleye-bootimage

---


<a id="2026-11-25-walleye-ufs-off-kernel-build-declared-in-device"></a>
### walleye-ufs-off kernel build declared in device.toml

Entry date
: 2026-11-25

Done
: Moved the declared [kernel_builds.walleye-ufs-off] row from research into device.toml; kernel-build.sh passes; ensure resolves to the store kernel whose sha256 equals the declared one.

Considered
: A cold rebuild to re-prove: not needed, the store path is the Nix build of that recipe and its bytes were hashed.

Rejected
: Declaring the shipped (-dirty) kernels' hashes: a rebuild cannot equal them.

About
: walleye-bootimage
