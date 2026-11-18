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
