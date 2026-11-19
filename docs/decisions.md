<!-- Generated from data/ by tad. Do not edit by hand. -->

# Decisions

<a id="2026-11-17-walleye-slot-flags-tries-on-b"></a>
### 2026-11-17-walleye-slot-flags-tries-on-b

Date
: 2026-11-17

Brief
: Tries run on slot b with a left as it is; set-active (metadata only) is allowed to keep a slot bootable

Decision
: 1) Leave the flags as they are now (b current and successful, a unbootable but successful, its image untouched and known good: Android ran on it and was measured); no set_active now. 2) Further 'devlab fastboot boot now' tries run with b as the active slot. 3) The keeper reads devlab's before/after slot report on every try and records it with the phone's full fastboot text. 4) devlab's 'set-active now SLOT' (metadata only: priority, retry, flags, partition GUIDs, boot LUN; undone by set_active back) is allowed for exactly one purpose: when a try leaves no good active slot (current slot unbootable, or Android not reachable), the keeper makes b active again (b is the tries' slot; a stays the untouched reserve), and a only when the author wants Android on a. [fastboot] slot_switch = true in the lab's device.toml; writes stays false. 5) Nothing is flashed or erased: a stub dtbo (id 2704, which the Pixel 2 XL bootloader needs) is a write with no known way back (fastboot cannot read the stock dtbo) and stays the author's decision, to be asked only if the tries show the bootloader refusing our kernel for that reason. 6) Next try variants are the builder's: header os_version and patch level set (ours are 0), panic=10 kept (a hang needs hands).

Why
: Research walleye-slot-flip-why: a failed fastboot boot on the active slot makes Qualcomm's ABL flag it unbootable and switch to the other successful slot, which is what we saw; the kernel probably never ran. set_active is metadata only and reversible, a and b both boot Android and fastboot stays reachable, so the way back is on record (a two-way door).

Rejected
: set_active a now (a stays successful=no, the next failed try flips it again, a write for nothing); set_active before every try (a write per try); flashing a stub dtbo now (no known way back: the author's decision); waiting for the author (the flags are reversible and the way back is recorded).

About
: walleye-first-fastboot-boot

Research
: walleye-slot-flip-why


<a id="2026-11-17-walleye-control-image-download"></a>
### 2026-11-17-walleye-control-image-download

Date
: 2026-11-17

Brief
: The walleye lane may download LineageOS's official walleye boot.img as the boot control; downloads of public checksummed files are the lane's call

Decision
: On the author's words of 2026-11-17 ("Yes, download it"; then "no need to ask for my permission for downloads ... by roles expertise can be handled by themselves"): download LineageOS's official walleye boot.img (build 20261113, sha256 48dbc55c76ee47840e645707b6bf3ac322099c06358178ad8ddf2c6a53c17468; download.lineageos.org publishes no recovery.img for walleye: on A/B LineageOS carries the recovery in boot) from download.lineageos.org only, verify the sha256, keep it in ~/Projects/walleye-build/control/ with its URL, size and checksum recorded, read its header and dtb ids on the host (unpack_bootimg, fdtget; data, never run on the host), and boot it once from memory as the control (devlab fastboot boot now, tries on b, slot flags read before and after). Nothing is flashed. From now on downloads of public, checksummed files that the lane's work needs are the lane's decision, recorded with source and checksum; the author is asked only for his hands, a write that cannot be undone, money or outward actions.

Why
: Research walleye-load-error-how-it-stands: a stock-format image answers whether the Load Error is our image or the phone's state; boot.img is the smallest official file holding the true header.

Rejected
: TWRP or other sources (worse provenance); building a control ourselves (not a control); asking the author per download (his word: not needed).

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-slot-set-active-a-once"></a>
### 2026-11-17-walleye-slot-set-active-a-once

Date
: 2026-11-17

Brief
: One reversible control on slot a: set-active a, control boot.img once, then back to b

Decision
: The keeper may run devlab fastboot set-active now a (metadata only), try the LineageOS control boot.img once on slot a (the state of the very first tries), record devlab's before/after report and the full fastboot text, and then set-active b if the phone ends in fastboot on a failed slot, so the phone ends as it is now (b current and successful). No flash, no dtbo or vbmeta write. Extends walleye-slot-flags-tries-on-b.

Why
: The control (a real stock-format image) also gets Load Error on b with a unbootable: the one change from the first tries is the slot state; set-active is metadata only, a's image is untouched and both slots boot Android, fastboot stays reachable - a recorded way back.

Rejected
: Flashing a stub dtbo or vbmeta (no known way back); waiting for the author (a two-way door: the lane's call); leaving the slot question open while only researching (a cheap test answers it).

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-load-error-vector"></a>
### 2026-11-17-walleye-load-error-vector

Date
: 2026-11-17

Brief
: Restore the first state (Android booted on slot a, then bootloader) and try the control; then cold restart, host checks, UART; no write

Decision
: Order for the keeper: (1) walleye-load-error-first-state: set-active a, let a's Android finish booting, adb reboot bootloader, the control once; (2) walleye-load-error-restart-host if refused; (3) walleye-load-error-uart with the author's cable; writes (dtbo, vbmeta) come back as new items with their way back after those.

Why
: The control on slot a was refused too, so the slot flags alone are out; the only accepted image was tried from a phone that had booted Android on a normally, and that state (a successful=yes, bootloader entered from Android) is the one thing not repeated since. It writes nothing and set-active b is the way back.

Rejected
: Flash a stub dtbo or vbmeta now: no way back for the stock dtbo and no evidence it is the cause. Change the image again: the Lineage control already fails with a stock header. Treat the string as one fault: reports show restart, cable, set_active and reflash all cure it, so each is a separate small try.

About
: walleye-first-fastboot-boot

Research
: walleye-boot-refused-why-how-it-stands


<a id="2026-11-17-walleye-bisect-from-the-control"></a>
### 2026-11-17-walleye-bisect-from-the-control

Date
: 2026-11-17

Brief
: After 'Booting OKAY' with no start, bisect from the accepted stock control, not UART or a dtbo write

Decision
: Our images e and b are accepted from the first state but never start; the stock control boots. The next step is a bisect: images s1 (stock kernel + stock dtb + our initramfs), s2 (our kernel + stock dtb), s3 (stock kernel + our dtb), one swap each, built from the control's own unpacked parts; the keeper tries them from the first state. UART (the author's cable) and a dtbo/vbmeta write come after, only if the bisect leaves the question open.

Why
: A bisect uses only what we can read and rebuild on the host, changes nothing on the phone, and s1 may give a working USB shell on the stock kernel with our initramfs, a real foothold; research walleye-own-images-e-b-first-state.

Rejected
: UART first (needs the author's cable, a purchase or wait); a stub dtbo or vbmeta write (no way back for the stock dtbo, no evidence it is the cause: acceptance already happens); changing our images at random again.

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-vendor-kernel-first"></a>
### 2026-11-17-walleye-vendor-kernel-first

Date
: 2026-11-17

Brief
: Walleye boots the vendor kernel with our gadget initramfs first (the control image, ramdisk swapped); lk2nd, a 4.4 build, UART-first as a blocker rejected

Decision
: Before any more tries of our own mainline images, add the method's step 3 at its cheapest: take the control (LineageOS walleye boot.img, accepted from the first state), change only its ramdisk to our gadget initramfs (172.16.43.1), keep its kernel, dtb and cmdline, and fastboot boot it from the first state. If the gadget answers, read cmdline, ids, reserved memory (ramoops, cont-splash), rails and USB mode from the running vendor kernel, read-only, and record them; only then our mainline image e/b, with the ramoops address and dts facts taken from what was read. The UART cable stays queued in parallel as the feedback channel of last resort; its oem decision is unchanged. The failing test walleye-gadget-up (by manifest serial, never VID:PID) is the proof of every step.

Why
: Every sibling (OnePlus 5, Mi 6, Pixel 3 XL, taimen) had a feedback channel or a known-good first; the lane has run no kernel on this phone and its one accepted image gave no sign of life. The ramdisk swap needs no kernel build and no write to the phone, uses the one image format the bootloader has accepted, and separates 'the bootloader accepts and starts it' from 'our kernel and dts are right'. It also yields the facts our dts guesses (ramoops address, simplefb, USB mode) from observation. Confirmed by main while this was written: our images e and b were accepted from the first state and neither started (walleye-own-images-e-b-first-state), so a second silent mainline try tells nothing more.

Rejected
: lk2nd / second stage: no msm8998 target, device list ends at msm8996, stock bootloader here is ABL not LK, maintainer closed a newer-SoC request; a port is very high cost and unproven under ABL. U-Boot under ABL (sdm845 path): no msm8998 port known, high cost. Building the vendor 4.4 kernel from GPL source: medium cost, unneeded while the Lineage image already holds a built one. Full Halium/Droidian/Ubuntu Touch install: high cost, forum reports for walleye poor, little for mainline. Going straight on with our mainline images (pmOS sibling practice): failure there tells nothing; ordered after the known-good. UART first as a gate: needs the author's hands and an oem command whose persistence is unknown (a change to the phone); kept parallel, not blocking. Flashing boot_a or a stub dtbo: writes, the author's word; only if the control and the swapped image both still end in Load Error.

Research
: walleye-bootstrap-method-how-it-stands


<a id="2026-11-17-walleye-try-returns-without-hands"></a>
### 2026-11-17-walleye-try-returns-without-hands

Date
: 2026-11-17

Brief
: Every walleye try image carries its own way back; the author's hands are only for what no image can undo

Decision
: No image goes to the phone without a recorded way back that needs no hands. Outcomes of a vendor-gadget try: (1) gadget up: the keeper explores read-only over telnet and ends the try with the init's own reboot (sysrq b / reboot -f) which boots Android on slot a (a is successful), then adb reboot bootloader; (2) kernel panic: the bootloader falls back to Android on a after ~75 s (seen with s1); (3) a silent hang (no panic, no gadget): the init arms a self-reboot timer (reboot -f after 240 s unless a file /tmp/hold exists, which a telnet session may create) - so no hands; (4) a hang before init or in the kernel: the Qualcomm watchdog bites and the phone restarts into a (the s1 case). Hands (hold Power 10-30 s) remain only for a phone that neither panics nor resets (a kernel hang with the watchdog off), and for images that boot a mismatched Android (the stock-ramdisk control images: stopped for good). A try is announced to the coordinator with this table filled for the image.

Why
: The author held Power three times today for the same pattern (the control-style images boot Android with a system of another build and stick on the logo); the init that never exits would add a fourth kind unless it reboots by itself.

Rejected
: Asking the author for each hang (his attention is the scarce resource); an init that exits (panics, but falls back in 75 s: acceptable only as the last way); controlling power from the lab (no power control exists: a USB power switch or relay is a purchase for the author).

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-panic-read-order"></a>
### 2026-11-17-walleye-panic-read-order

Date
: 2026-11-17

Brief
: Read the panic in this order: read-only cmdline and timing first, ramoops after one author unlock second, UART cable third; no write before them

Decision
: 1) Read the running Android's full /proc/cmdline and chosen/bootargs and the control header cmdline (read-only): is skip_initramfs there? 2) Measure the panic time with the 1 s watcher against a plain-reboot Android-boot time. 3) Run gadget4 (idle init) only if skip_initramfs is absent. 4) After a panic try, boot Android, ask the author to unlock once, read /data/vendor_ce/0/ramoops as root before any further reboot. 5) If still dark, the author buys the Android Debug Cable (USB-C passthrough + FT232) and the keeper runs fastboot oem uart enable (way back: oem uart disable).

Why
: A system-as-root phone ignores a swapped boot ramdisk when the cmdline has skip_initramfs, which would explain no gadget ever and costs only reads to check; the panic time decides between a kernel-side 60 s timeout and an early fault; ramoops survives the warm reset but the vendor service consumes it, so the unlock is the one hand needed; UART answers everything but is a purchase and a bootloader setting.

Rejected
: UART first: sure but a purchase and a persisted bootloader setting before the free reads. A dtbo or kernel-cmdline write to move the console: a write with no need yet. Rebuilding the init again (gadget5) before knowing the ramdisk runs: spends tries on a guess. /proc/last_kmsg: gone since Android 6 (console-ramoops instead).

About
: walleye-first-fastboot-boot

Research
: walleye-panic-60s-how-it-stands


<a id="2026-11-17-walleye-ncm-without-attributes"></a>
### 2026-11-17-walleye-ncm-without-attributes

Date
: 2026-11-17

Brief
: Keep ncm as the channel but never write or read its attributes; try one image without the dev_addr/host_addr writes, split into steps

Decision
: The gadget inits drop every write to functions/ncm.usb0 attributes (dev_addr, host_addr, qmult, ifname). Build ncm-noattr images in steps b1 (mkdir function), b3 (config + symlink), c (UDC bind, usb0 up, telnetd); the device-side MAC is set afterwards with ip link. Fall back to FunctionFS with a static helper only if bind panics.

Why
: In the wahoo 4.4 source the ncm net_device is made at UDC bind, so before it the attribute stores dereference NULL and after the symlink they return EBUSY; every image that panicked wrote those attributes, every one that returned did not have a function.

Rejected
: FunctionFS ffs helper first: needs new code and a userspace daemon, no gain if ncm works. gsi.rndis/rmnet: needs IPA and firmware at early init, risky. mass_storage read-only: no log stream, host-visible only. MTP/ACC: need a daemon. Patching the vendor kernel's f_ncm: we boot the unmodified control kernel on purpose.

About
: walleye-first-fastboot-boot

Research
: walleye-ncm-panic-why-how-it-stands


<a id="2026-11-17-walleye-host-channel-order"></a>
### 2026-11-17-walleye-host-channel-order

Date
: 2026-11-17

Brief
: n2 enumerates but the host cannot bind NCM: ECM/RNDIS are not in this kernel, so a mass-storage log window now and a FunctionFS ACM helper next

Decision
: Our gadget reached the host for the first time (n2: 1d6b:0104 with our strings after 18 s, no panic); cdc_ncm failed on 'failed to get mac address' because the NCM function cannot be given its addresses on this kernel (the attribute writes panic it, so it serves none). The vendor config has ECM, RNDIS, EEM, ACM and SERIAL NOT set, so ecm/rndis variants (walleye-gadget-host-bind as queued) are impossible and dropped. Order: (1) m1, a mass_storage gadget (in the kernel: MASS_STORAGE=y) exporting a fixed-size file in the ramdisk as a read-only LUN; the init rewrites it in place every 5 s with dmesg, cmdline, mounts, heartbeat; the host reads the raw device with O_DIRECT (a one-way, crude but real log window); (2) a FunctionFS helper (a tiny static aarch64 C program declaring CDC-ACM descriptors on ffs, bridging to a shell) so the host's cdc_acm gives /dev/ttyACM0: research item walleye-ffs-helper, run in parallel and now needed, not a fallback. NCM stays out.

Why
: The config facts (research walleye-vendor-kernel-facts-s1-panic) rule out ECM/RNDIS; mass_storage needs only in-kernel code and a file, so it gives diagnostics at once; FunctionFS with ACM descriptors gives an interactive console with host drivers that exist everywhere.

Rejected
: Chasing the NCM MAC string (the attribute writes panic the kernel; the function is unusable without them); building ecm/rndis images (not in the kernel); waiting for the UART cable (a purchase) when two in-kernel routes exist; adbd from the Android system (bionic userland, heavy).

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-ffs-acm-single-interface"></a>
### 2026-11-17-walleye-ffs-acm-single-interface

Date
: 2026-11-17

Brief
: FunctionFS console: one class-2 interface with three endpoints (no CDC functional descriptors), descriptors written to ep0 before the UDC bind, pty shell bridge in a static C helper

Decision
: Build the helper as in the research: one interface (class 2/2/1), interrupt-in + bulk-out + bulk-in, descriptors for FS/HS/SS and strings written to ep0 before the UDC bind, no function attribute written, ep0 class requests answered, a pty shell bridged to the bulk endpoints. Two items: build the helper and image (needs build), then one phone try proven by tests/host/ffs-acm-up.sh (fails today, written with this decision).

Why
: f_fs rejects CDC functional descriptors in 4.4.302 and 7.3 (read in source); cdc_acm's probe accepts a single combined interface of three endpoints (read in source); this needs no kernel option the vendor kernel lacks. Not yet run on the phone.

Rejected
: Full CDC-ACM descriptor set (header/call-mgmt/ACM/union) over FunctionFS: ep0 write fails -EINVAL. configfs acm/serial/ecm/rndis functions: not in the vendor kernel. Android adbd as the channel: bionic userland, already rejected. A libusb-style custom vendor-class interface: needs a host program instead of the stock cdc_acm driver. Writing function attributes: the 5b panic path.

About
: walleye-first-fastboot-boot

Research
: walleye-ffs-acm-helper-how-it-stands


<a id="2026-11-17-walleye-dtb-carries-overlay-symbols"></a>
### 2026-11-17-walleye-dtb-carries-overlay-symbols

Date
: 2026-11-17

Brief
: Our dtb is built with -@ and carries stub nodes for every label the vendor dtbo overlay names, so the bootloader's overlay merge can succeed

Decision
: Generate from the overlay's __fixups__ a compat dtsi (the 137 labels, paths as in the stock dtb's __symbols__), include it in msm8998-google-walleye.dts, build with -@, and gate on tests/host/dtbo-overlay-merges.sh passing on the host before any phone try; then one s3-like image (stock kernel, ours dtb).

Why
: fdtoverlay: overlay 3 merges into stock dtb, fails on ours for lack of __symbols__; libufdt treats that as an error; s3 is the only bisect variant with such a dtb and the only one stuck after Booting OKAY.

Rejected
: Stock dtb for first light: the kernel would run on the vendor tree, wrong for mainline drivers, and it hides the problem. Patching the stock dtb with our nodes: larger, same end. Writing dtbo to drop the overlays: a write with no way back (fastboot cannot read it), not proposed. Only -@: still ~123 of 137 labels missing. Only changing compatible: the merge never reads it.

About
: walleye-first-fastboot-boot

Research
: walleye-dtbo-overlay-merge-why-how-it-stands


<a id="2026-11-17-walleye-mainline-light-images"></a>
### 2026-11-17-walleye-mainline-light-images

Date
: 2026-11-17

Brief
: First light images ml1 (Image.gz) and ml2 (lz4 frame): the control's header and footer, our kernel, our -@ dtb, a mass_storage init that never pets the SoC watchdog

Decision
: ml1/ml2 are packed from the control's header, offsets and AVB footer with our kernel (v7.3 + QCOM_WDT=y), our -@ dtb with a watchdog@17817000 node, and init-ml (mass_storage window, serial walleye-first-boot, product 'Pixel 2 walleye (mainline)', /dev/watchdog opened and never petted). Command line: earlycon=msm_serial_dm,0xc1b0000 clk_ignore_unused ignore_loglevel panic=5 plus the stock androidboot.* words; the vendor-kernel words (lpm_levels, msm_rtb, swiotlb, service_locator, ...) are dropped. Proof: tests/host/mainline-light-up.sh (USB strings, no host permission).

Why
: The item asks for exactly these parts; mainline v7.3 msm8998.dtsi has no apss watchdog node and QCOM_WDT was a module (the initramfs has none), so without both the phone could not return by itself. The watchdog address/bark SPI are the downstream MSM8998 values, not measured: if the node faults the kernel panics and panic=5 resets, which also returns.

Rejected
: qcom_wdt.arm_on_probe=1/reboot=w (porthole's out-of-tree patches, no mainline effect); keeping the vendor cmdline (vendor-only options); a network function (ncm attribute writes panicked the vendor kernel, and the host has no ECM/RNDIS)

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-ramoops-at-the-phones-region"></a>
### 2026-11-17-walleye-ramoops-at-the-phones-region

Date
: 2026-11-17

Brief
: Our dts puts ramoops at the phone's own region 0xa1100000/2 MiB (record 0x80000, console 0x20000, no pmsg), so stock Android can read our kernel's log

Decision
: dt/msm8998-google-walleye.dts: ramoops@a1100000, reg 0xa1100000 size 0x200000, record-size 0x80000, console-size 0x20000, no pmsg or ftrace; PSTORE_CONSOLE and PSTORE_RAM stay =y. No other carve-out in our dtb reaches 0xa1100000 (the highest ends at 0x95800000).

Why
: Lab research walleye-vendor-kernel-facts-s1-panic: the phone's ramoops is 0xa1100000, 2 MiB, record 524288, console 131072; a log written at 0xa1810000 is not read by Android. Checked on the built dtb by fdtget/fdtdump in build/build-ml3.sh.

Rejected
: Keeping 0xa1810000 (taimen/porthole value): Android would not parse it. Keeping pmsg-size 0x80000: the vendor values have none, and it shrinks the console zone. Reading /sys/module/ramoops/parameters on the phone: a builder never touches the phone; the facts research already holds them.

About
: walleye-first-fastboot-boot

Research
: walleye-ml1-ml2-no-light


<a id="2026-11-17-walleye-android-is-a-stepping-stone"></a>
### 2026-11-17-walleye-android-is-a-stepping-stone

Date
: 2026-11-17

Brief
: Android on walleye is a stepping stone to mainline: its settings are the lane's to set

Decision
: The Android on the Pixel 2 (LineageOS 15 on slot a, an older build on slot b) is kept only as the way back, the measured reference and the tool that reads our kernel's logs; the goal is mainline Linux on the phone, as on surya. The lane may set Android's settings for the work (screen lock off, USB debugging, developer options) without asking, recording what it set; the author's keep-or-restore question is closed for the lock. What can still lose the reference (a flash or erase of a slot's partitions) needs a recorded decision as before.

Why
: The author's words (via the top coordinator, 2026-11-17): the only reason Android is still on the phone is so mainline Linux can be put on it, the whole purpose of the project, like surya.

Rejected
: Treating each Android setting as the author's to restore (it only slows the lane); wiping Android now (it still reads our crash logs and is the way back).

About
: walleye-first-fastboot-boot


<a id="2026-11-17-walleye-ffs-console-parked"></a>
### 2026-11-17-walleye-ffs-console-parked

Date
: 2026-11-17

Brief
: The FunctionFS console on the vendor kernel is parked: cdc_acm cannot bind it; the mainline kernel's own ACM is the console

Decision
: The FunctionFS ACM console on the vendor 4.4.302 kernel is parked, not pursued. Two findings contradict each other and close the route: the research (walleye-ffs-acm-helper-how-it-stands) found that f_fs rejects CDC class-specific (0x24) descriptors in this kernel, and the keeper's try of n3r (walleye-cdc-acm-needs-functional-descriptors) found that the host's cdc_acm refuses the interface without them ('Zero length descriptor references', -22). So cdc_acm cannot bind a FunctionFS gadget here. The vendor-kernel facts we need come one-way through the mass_storage log window (m1, proven by lpoco-host's acceptance test). The interactive console is for OUR mainline kernel, where configfs ACM (CONFIG_USB_CONFIGFS_ACM) is native and gives /dev/ttyACM0 without FunctionFS. The C probe's removal (walleye-code-hygiene-rust-ci) is unblocked; the Rust helper stays in the tree as a built, tested artifact until the removal PR decides.

Why
: Two independent facts make the route a dead end on this kernel; every further variant would cost tries for a console we do not need now; the one-way window already gives the vendor kernel's facts, and the mainline kernel's own ACM function is the console that matters.

Rejected
: n3r2 with CDC header/call-management/ACM functional descriptors in FunctionFS (f_fs rejects them); a vendor-class interface bound by usbserial generic new_id (needs host root writes through lab-prepare for a console we do not need yet); more FunctionFS tries.

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-bisect-from-the-alive-dtb"></a>
### 2026-11-18-walleye-bisect-from-the-alive-dtb

Date
: 2026-11-18

Brief
: Bisect from the alive taimen dtb toward walleye (additive), not by neutralizing overlay labels in our dtb

Decision
: Build images from pk5 (alive) that take on the walleye dtb's differences group by group; wa is the whole difference at node and status level

Why
: Research walleye-overlay-enables-what-result: the overlay does less on our dtb than on the taimen dtb, so neutralizing its labels cannot find a missing thing; the base differs by 15 disabled nodes and ~100 absent ones, the same for porthole's own walleye dtb

Rejected
: Images L1..Ln (our dtb, overlay labels pointed at dummies): the overlay adds nothing live to ours that taimen lacks. More subtraction from our dtb: 13 boots found nothing.

About
: walleye-first-fastboot-boot

Research
: walleye-overlay-enables-what-result


<a id="2026-11-18-walleye-property-bisect-plan"></a>
### 2026-11-18-walleye-property-bisect-plan

Date
: 2026-11-18

Brief
: Property bisect: one image per group plus the all-groups control wpa, not binary halves

Decision
: Build wp1..wp5 (one group each) and wpa (all five), tried wpa first

Why
: Only five groups: one try per group costs 12 boots, the same as two halving rounds, and shows interactions through wpa; wpa also tests the premise that the cause is a property at all

Rejected
: Binary-halving images (g1+g2 / g3+g4+g5): five groups do not need them. The mirror images o1.. from our dtb: only if wpa/wp1..wp5 leave the cause in a group ours alone; our dtb's diff to wa is not yet measured

About
: walleye-first-fastboot-boot

Research
: walleye-property-diff


<a id="2026-11-18-walleye-warm-reset-by-watchdog-bite-first"></a>
### 2026-11-18-walleye-warm-reset-by-watchdog-bite-first

Date
: 2026-11-18

Brief
: Prove a kernel alive first; then try a deliberate sysrq-c reset, then an unpetted-watchdog bite, before any kernel patch for a PMIC warm reset

Decision
: Order of tries for the pstore question: T1 sysrq c with panic=5 (expected empty, measures cause B), T3 unpetted kpss-wdt bite (expected present), only then T4 a PON reset-type reboot notifier patch. cmdline reboot=w alone is not a try worth an image slot beyond T1 (PSCI has no RESET2 here). None of it before a kernel has shown life.

Why
: msm_restart_prepare of the vendor kernel shows plain reboot is a PMIC hard reset and only panic/dload/commanded reboot are warm; mainline sets no reset type and psci runs before the watchdog restart handler; the bite is the one warm-capable path already in our dts and init.

Rejected
: A PON reset-type kernel patch first: more work and the register values are not yet read from source. reboot=warm first: needs PSCI 1.1 SYSTEM_RESET2, not present as far as known. Reading more of the empty pstore on stock Android: it cannot tell a missing kernel from a lost region.

About
: walleye-first-fastboot-boot

Research
: walleye-pstore-warm-reset-how-it-stands


<a id="2026-11-18-walleye-t1-gate-met"></a>
### 2026-11-18-walleye-t1-gate-met

Date
: 2026-11-18

Brief
: The pstore T1 try may run: our mainline kernel with the taimen dtb has shown life (init held 60 s)

Decision
: The gate of the pstore tries ("run only after a kernel has shown life") is met: the g1 and g2 tries of pk5-stages (lab PRs 114 and 116) returned at ~108 s and 138 s, i.e. our init held its 60 s, so our mainline kernel with porthole's Pixel 2 XL device tree runs and executes our init; only its USB gadget does not enumerate. The pstore T1 try (deliberate panic after 20 s, ramoops at the phone's own region) may run now; its result separates 'the console of a living kernel survives a panic reset' from 'it does not'.

Why
: The timing bins alone are not an observation of the kernel, but a return at 108 s or later cannot be a dead-kernel reset (those return at 69-78 s): an init that holds 60 s ran.

Rejected
: Waiting for a USB light first (the gadget does not enumerate, so the gate would never open); lifting the gate silently without a record.

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-vendor-kernel-as-instrument"></a>
### 2026-11-18-walleye-vendor-kernel-as-instrument

Date
: 2026-11-18

Brief
: Pause the pstore chase; dump the running hardware's facts through the vendor kernel and the log window, then give mainline USB its nodes

Decision
: The pstore chase (T1, T3, ml*, pk*) is paused: after many images, no witness of our mainline init exists without a UART, and every further pstore try is read through the same doubt (T3: back at 42 s, no marker: no bite, a lost ramoops region or no init cannot be told apart). Instead the working vendor kernel is used as the INSTRUMENT: our init on the vendor 4.4.302 kernel (proven: m1 enumerates and its window is read by the lab user) mounts debugfs and dumps what the mainline device tree needs into a larger log window: the running device tree (/proc/device-tree, merged with the overlay: the walleye hardware as the phone itself describes it), clk_summary, regulator_summary, pinctrl/gpio state, interrupts, iomem/ioports, the USB controller and PHY state in peripheral mode (dwc3 debugfs, the phy registers' bound clocks and regulators), the reserved-memory map, the PMIC/SPMI devices, the dmesg of that boot. These are facts from the running hardware (GPL kernel, observed behaviour), the clean-room wall holds (no blobs read). They give the dwc3/phy/regulator/clock nodes that mainline USB needs, so the mainline gadget can enumerate: the first real light. The UART cable stays the proper channel and is the author's purchase.

Why
: Each pstore try is a one-bit answer through a channel we cannot verify (the same empty pstore means no bite, no region or no init); the vendor kernel is alive, verified and readable through the lab-prepare window; the facts it holds are exactly what mainline USB lacks.

Rejected
: More pstore images without a witness (T4 pon notifier, further watchdog variants); waiting idle for the UART cable.

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-bisect-from-alive-by-fdtput"></a>
### 2026-11-18-walleye-bisect-from-alive-by-fdtput

Date
: 2026-11-18

Brief
: Bisect from the alive wpa dtb toward ours by fdtput edits per group, emptying overlay targets

Decision
: The x images are wpa with one group made equal to ours, by fdtput node removals and property edits on the dtb; a node the dtbo overlay targets is emptied, not removed. The groups change disjoint lines; xa (all groups) differs from ours only in pinned lines.

Why
: wpa is the alive tree closest to ours (same properties as pk2), so each group is a small, named change; keeping the overlay merging keeps the bootloader's step identical across images, so only the group can change the outcome.

Rejected
: A dtb merge tool that rewrites phandles by path (days of work, new code to trust); source-level splicing (ours and porthole's trees have different bases); removal from ours again (d1..d6 did that: all dead, uninformative); starting from pk5 (further from ours: more groups per image).

About
: walleye-first-fastboot-boot

Research
: walleye-fdt-bisect-x-built


<a id="2026-11-18-walleye-our-dts-gets-gpio-reserved-only"></a>
### 2026-11-18-walleye-our-dts-gets-gpio-reserved-only

Date
: 2026-11-18

Brief
: Give our dts the gpio-reserved-ranges property alone, one commit; leave pshold and the apps watchdog out until the try says they matter

Decision
: dt/msm8998-google-walleye.dts gets &tlmm gpio-reserved-ranges = <0 4>, <81 4>, with a comment naming its source (the alive taimen dtb, porthole's dtb, and the x1 bisect). Images y1 and y2 are ml6a plus that one dtb change.

Why
: Of 16 boots in the fdt bisect, x1 (the property removed from the alive tree) was dead twice and x2-x7 alive: the property is the cause. One change keeps the effect attributable (the item's own rule).

Rejected
: Adding restart@10ac000 (qcom,pshold) and the apps watchdog now: x4 was alive without them being ours, so the finding does not say they matter; they come after y1/y2 are read. Putting the property in a separate overlay or only in the image's dtb with fdtput: the source of truth is our dts, which the images must be built from. Changing the ramdisks or command line: they are the proven ml5a/pk5l ones, unchanged.

About
: walleye-first-fastboot-boot

Research
: walleye-y-built


<a id="2026-11-18-walleye-bringup-order"></a>
### 2026-11-18-walleye-bringup-order

Date
: 2026-11-18

Brief
: Bring walleye up in this order: PMIC, clean probes, charger, UFS read-only; then USB net, fuel gauge, touch, display, Wi-Fi; audio and the radios last

Decision
: The lane's builders and keeper take items by rank: 1 walleye-pmic-spmi, 2 walleye-dmesg-clean, 3 walleye-charger-smb2, 4 walleye-ufs-readonly; the unranked items follow in the order of the plan table. The research session itself queued each item with its needs and proof. Parts without a mainline driver (S6E3FA5 panel, PMI8998 fuel gauge, DRV2624, TAS2557) are upstream candidates: a clean-room port from the vendor's GPL sources, then sent upstream.

Why
: The first mainline dmesg shows the SPMI PMIC unbound, the smmu and serial probes failing; every later part rests on those. UFS rails are RPM regulators, so UFS does not wait for the PMIC. qcom_smbx is already in mainline, so a charging phone costs a dts node, and it ends the battery constraint of the keeper's rule. UFS read-only gives the phone's boot.img header and the firmware partitions the radios need.

Rejected
: Display first: the panel driver is new and a person must witness it, while ACM already reads a dmesg. Wi-Fi first: needs UFS to read firmware. Adopting porthole's 263 taimen patches whole: taimen-focused, one untested walleye board (mainline-path research). A fuel gauge before the charger: the gauge is an unmerged RFC; the charger is merged. PMIC as the gate of UFS (the brief's guess): the vendor tree puts UFS rails on the RPM.

About
: walleye-first-fastboot-boot

Research
: walleye-bringup-order-plan-made


<a id="2026-11-18-walleye-pmic-from-mainline-dtsi"></a>
### 2026-11-18-walleye-pmic-from-mainline-dtsi

Date
: 2026-11-18

Brief
: Include mainline's pm8998.dtsi and pmi8998.dtsi in our dts; leave the overlay's two reg-less nodes alone and tolerate exactly them in the test

Decision
: dt/msm8998-google-walleye.dts includes pm8998.dtsi and pmi8998.dtsi as the OnePlus 5(T) and Yoshino boards do; the stubs lose the five labels now real. The host check names the two nodes of the dtbo overlay (qcom,pm8998@0, qcom,pmi8998@3) as not ours and fails on any other reg-less spmi node.

Why
: Mainline binds PMICs through those two files with reg <usid SPMI_USID>; the reg-less lines of the first dmesg are the bootloader-merged overlay's downstream nodes, which no board file can remove.

Rejected
: Writing our own pmic nodes with the vendor-style names qcom,pm8998@0 so the overlay merges into them: a name no mainline file uses. Demanding zero reg-less lines: unreachable while the stock dtbo is merged, and the dtbo cannot be dropped (the bootloader picks it by idx 3). Porting the vendor PMIC node set whole: the charger and fuel gauge need their own bring-up (next items).

About
: walleye-first-fastboot-boot

Research
: walleye-z1-built


<a id="2026-11-18-walleye-gpucc-built-in"></a>
### 2026-11-18-walleye-gpucc-built-in

Date
: 2026-11-18

Brief
: Build CONFIG_MSM_GPUCC_8998 into the walleye kernel (=y); z2 is z1 with only that change

Decision
: build/walleye.config sets MSM_GPUCC_8998=y and build-kernel.sh refuses a kernel without it; image z2 = y2's image with z1's dtb and the new Image.gz, nothing else changed.

Why
: The initramfs carries no modules and the smmu's power domain is gpucc's GDSC (research walleye-smmu-timeout-gpucc-module). One change per image keeps a try readable.

Rejected
: Disabling the adreno_smmu node (status disabled): hides the fault and the GPU would need it later. Loading gpucc as a module from the initramfs: a second moving part for one driver. Fixing the serial supplier and phandles in the same image: two causes in one try cannot be told apart.

About
: walleye-first-fastboot-boot

Research
: walleye-smmu-timeout-gpucc-module


<a id="2026-11-18-walleye-clock-gcc-symbol-to-real-gcc"></a>
### 2026-11-18-walleye-clock-gcc-symbol-to-real-gcc

Date
: 2026-11-18

Brief
: z3 = z2 with only /__symbols__/clock_gcc pointed at the real gcc (fdtput); the stub node stays

Decision
: build/build-z3.sh sets /__symbols__/clock_gcc to /soc@0/clock-controller@100000 in z2's dtb; kernel, ramdisk, header, command line and AVB footer are z2's.

Why
: The overlay's clock users need #clock-cells to parse; the real gcc has it and a phandle, so one symbol line fixes the serial supplier and all four phandle lines. A post-compile edit leaves the generated stubs file alone (the other 124 stubs are still needed by the merge).

Rejected
: Disabling the overlay's nodes: the overlay is the bootloader's, not ours to edit. Giving the stub #clock-cells: a fake clock provider the serial would then bind against. Dropping the clock_gcc stub from the generator: dtc cannot put a second label on mainline's gcc node from our dts. More changes in the same image: one change per image.

About
: walleye-first-fastboot-boot

Research
: walleye-clock-gcc-stub-no-cells


<a id="2026-11-18-walleye-tty-read-through-root-exec"></a>
### 2026-11-18-walleye-tty-read-through-root-exec

Date
: 2026-11-18

Brief
: Read the gadget's tty through lab-agent root-exec (that one node, recorded) until the udev rule is fixed

Decision
: The banner try (lab PR 175) proved the phone speaks: 7 WALLEYE-BANNER lines, beats 11..17, gadget bound at ~54 s, read with a root-exec cat. The silence of the earlier reads was the host: /dev/ttyACM0 was root:uucp 0660 for the lab user (mll is in neither uucp nor, for this node, the udev rule's group), although /etc/udev/rules.d/70-lpoco-google-walleye.rules (lpoco 0.6.0) sets GROUP users MODE 0660 for SUBSYSTEM tty with ATTRS serial walleye-first-boot. Until orchestration settles why the rule does not apply to the tty node (to test with the gadget up: udevadm info and udevadm test of the node, the rule order against 60-serial.rules/50-udev-default), the keepers and readers may read and write THAT ONE device node through the lab's own `lab-agent root-exec -- COMMAND` (cat, stty, a single printf to the node: the commands named in the reader), every use recorded; no other root use, no host change, no group change.

Why
: Every try of the lane after first light learns through this tty; the author is away and nothing may wait on a host rule fix; root-exec is the lab's own sanctioned way to run a root command; the rule that should make it unnecessary is orchestration's tool to fix.

Rejected
: Waiting for orchestration's fix before reading anything more (the lane stalls); asking the author to add mll to uucp (a host change, his or orchestration's); changing the rule or the group myself (not the lane's).

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-banner-serial-not-root-exec"></a>
### 2026-11-18-walleye-banner-serial-not-root-exec

Date
: 2026-11-18

Brief
: The banner tty was denied because the image's serial was wrong; the root-exec reading (decision 2026-11-18-walleye-tty-read-through-root-exec) is withdrawn

Decision
: The root-exec reading of the gadget tty (decision 2026-11-18-walleye-tty-read-through-root-exec) is withdrawn: orchestration found the real cause. The banner image's USB serial is 'walleye-banner' (build/initramfs-mainline/init-banner line 39) while every other image and device.toml say walleye-first-boot, so the host rule (serial walleye-first-boot: GROUP users, MODE 0660) correctly did not apply to its tty node. No host or root change is needed: the banner init gets the serial walleye-first-boot (its product string may stay distinct) and the same serial check as build-mainline-light.sh has for init-ml; the readers keep their plain open with the retry on Permission denied and never use root-exec.

Why
: Orchestration read the host's kernel log (Product: Pixel 2 walleye (mainline banner), SerialNumber: walleye-banner): the rule is right, our image broke the contract of device.toml [host]; fixing the image keeps root out of the lane.

Rejected
: Reading as root through lab-agent root-exec (it hid our own bug and used a privilege the lane did not need); asking the author to change groups.

About
: walleye-first-fastboot-boot


<a id="2026-11-19-walleye-reader-runs-as-a-lab-job"></a>
### 2026-11-19-walleye-reader-runs-as-a-lab-job

Date
: 2026-11-18

Brief
: The tty reader runs as a lab-agent job: the keeper's sandbox denies the tty open

Decision
: The reader of the gadget's tty runs as a lab job, not inside the keeper session's sandbox: the keeper's Bash is sandboxed and denies the open of /dev/ttyACM0 although the ACL allows the lab user (z3 try, lab PR 184: getfacl shows user:mll:rw-, yet 'Permission denied' inside the sandbox; the banner2 try earlier read it). Practice 6 of the ecosystem is the right route anyway: tests and device work run on the lab machine through lab-agent. So: the keeper starts the reader FIRST as `lab-agent submit` of a small script (the reader invocation, WAIT=120, OUT=its material path, then `lab-agent wait`/`logs` for the output after the try), then runs `devlab fastboot boot now IMAGE` as before, then reads the job's output; the saved dmesg and facts land in the same material directory. The keeper does not disable its sandbox, does not use root-exec, and changes no permission; the job runs with the lab agent's own rights.

Why
: It is the lab's sanctioned way to run host-side checks unsandboxed (lab-agent submit), it needs no change of any permission, and the reader script and its self-test stay as they are.

Rejected
: Asking the keeper to disable its sandbox (a permission of the session, not the lane's to grant); running the reader through the author's terminal tab (a workaround of the same sandbox); root-exec (withdrawn).

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-z4-shell-keeps-port-open"></a>
### 2026-11-18-walleye-z4-shell-keeps-port-open

Date
: 2026-11-18

Brief
: z4: the init holds ttyGS0 open raw/-hupcl and the reader keeps one fd; no cttyhack

Decision
: z4 = z3 with only the init's console block changed (fd 4 held, stty raw -echo -hupcl, setsid busybox ash -i, retry on failed open, respawn at once, kmsg counter); the reader opens the tty once and never reopens

Why
: Echo with no answer means no process read ttyGS0: ash -i < ttyGS0 ends on host close and the old reader reopened per command. Review of busybox 1.36 source: cttyhack ignores stdin and may move the shell off ttyGS0.

Rejected
: setsid cttyhack (moves or no-ops, review); setsid -c (CONFIG_FEATURE_SETSID_CTTY not in our busybox); keeping the per-command reopen with a longer wait (the hang-up is the fault)

About
: walleye-first-fastboot-boot


<a id="2026-11-19-walleye-reader-plain-background-process"></a>
### 2026-11-19-walleye-reader-plain-background-process

Date
: 2026-11-18

Brief
: The tty reader runs as a plain background process of the keeper; the lab-agent job route is withdrawn

Decision
: The lab-agent job route for the tty reader (decision 2026-11-19-walleye-reader-runs-as-a-lab-job) is withdrawn: lab-agent's GitHub token answers 401 (journal: poll error ... lab-agent-jobs ... 401 every 5 s), so no job is ever picked up (z3 try, z5b try). The reader runs as a plain background process of the keeper (as in the z4 and z5 tries, which worked: the keeper opened the tty and read it), started BEFORE the boot with nohup, its output in the lab-material path; the keeper's own sandbox did open /dev/ttyACM0 in those tries (the 'sandbox' denial of the z3 try was the reader's last reboot write after the node vanished, lab research walleye-z3-ttyacm-not-a-permission-problem). No lab-agent submit, no root-exec.

Why
: The job route cannot work while lab-agent is retired; the plain background process has worked twice; the lane must not wait on the author for a token of a tool being retired.

Rejected
: Renewing lab-agent's token (the author's hands, for a retiring tool); running the reader through the coordinator each time (it does not scale and keeps the coordinator on the critical path).

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-z6-dmesg-fix-order"></a>
### 2026-11-18-walleye-z6-dmesg-fix-order

Date
: 2026-11-18

Brief
: Fix the z6 dmesg in this order: disable adreno_smmu, then c1 on the z6 init with debugfs and facts, UFS read-only, USB net, touch, display; leave vendor-overlay noise

Decision
: 1 walleye-dmesg-clean: dts sets &adreno_smmu disabled (no gpu_gx WARN, no deferral, gcc/gpucc sync_state clears for it); the reader mounts debugfs and saves devices_deferred. 2 walleye-try-c1 rebuilt on the z6 init (charger node, rradc). 3 walleye-ufs-readonly (ufshc, ufsphy, rails, UFS and QMP-UFS config). Then usb-net, touch, display. The vendor-overlay noise (serial@c16f000, i2c@c179000, bt_wcn3990, mnh-sm, reg-less PMIC nodes), psci -3 and the misaligned-image line are recorded as known and not fixed until walleye-dtbo-overlay-id says whether the overlay is needed.

Why
: Only G1 is a kernel WARN and the only one that holds a probe; the rest is noise from a overlay we do not own or firmware we cannot change. The charger and UFS each need their own dts and config, not a dmesg fix.

Rejected
: Fixing the vendor clock ids with a translation table in the dtb (a dtb cannot translate ids, and the overlay stays the bootloader's). fw_devlink=permissive to hide sync_state (hides, does not fix). Powering the GFX rail now (the rail is not known yet and GPU is far down the order).

About
: walleye-first-fastboot-boot

Research
: walleye-z6-dmesg-mapped

Tracked by
: walleye-dmesg-clean


<a id="2026-11-18-walleye-charger-supplier-permissive-then-modelled"></a>
### 2026-11-18-walleye-charger-supplier-permissive-then-modelled

Date
: 2026-11-18

Brief
: Charger deferred on tps61099: permissive devlink to prove, then a modelled fixed regulator

Decision
: For the charger's deferral on its supplier /soc@0/tps61099 (no driver for the boost converter): two steps, the cheap one first. (1) c3 = c2 with ONLY the kernel command line changed, fw_devlink=permissive added: it proves whether the charger probes once the unmet supplier no longer blocks it (the charger driver does not need that supplier to run); one try. (2) the proper model, if (1) shows the charger working: the tps61099 as the board describes it, read from the phone's running fdt (the material path of vfa: the node's compatible, the enable GPIO and the regulator it makes), as a regulator-fixed (or gpio-controlled fixed regulator) node in OUR dts with the GPIO from the running fdt, so the supply link is met without a driver for the chip; then c4 = c2 with that dts and without fw_devlink=permissive, and fw_devlink back to its default. The choice and its reasons go into the decision row.

Why
: Permissive devlink is a one-argument experiment that answers 'is the supply the only blocker?' at the cost of one image; the modelled regulator is the right end state and needs no driver; dropping the supply link in the dts would hide a real part of the board.

Rejected
: Removing the supplier from the dts only (hides the hardware); waiting for a tps61099 driver (none exists; a fixed regulator is what the board does for this boost); leaving fw_devlink=permissive forever.

About
: walleye-first-fastboot-boot


<a id="2026-11-18-z8-is-c2-plus-the-pmic-watchdog-on-z7-s-ramdisk"></a>
### 2026-11-18-z8-is-c2-plus-the-pmic-watchdog-on-z7-s-ramdisk

Date
: 2026-11-18

Brief
: z8 is c2 plus the PMIC watchdog, on z7's ramdisk, header and AVB; the series' tree is required by build-kernel.sh

Decision
: z8 = c2's kernel config and dtb (charger and battery kept) + CONFIG_PM8916_WATCHDOG=y (series at linux-phone-oco/linux e123d3bff on v7.3) + &pm8998_wdt { wakeup-source; status okay }. Image from z7's ramdisk, header, command line, AVB. build-kernel.sh now refuses a tree without pm8998_wdt.

Why
: Row walleye-kernel-pmic-wdt says z8 on the z7 base after the charger item; the dts and walleye.config are linear, so the charger stays (c2 deferred on tps61099, noise not a crash). One variable changes against c2: the watchdog.

Rejected
: z7 plus watchdog without the charger: would need the charger disabled by fdtput and a second config; more machinery, less like the tree.

About
: deep-sleep

Tracked by
: walleye-kernel-pmic-wdt


<a id="2026-11-18-walleye-no-pmic-bite-until-reset-type"></a>
### 2026-11-18-walleye-no-pmic-bite-until-reset-type

Date
: 2026-11-18

Brief
: No more PMIC bites until its reset type is a restart with a way back

Decision
: No further PMIC watchdog bite on walleye until the bite's reset type is known to be a restart with a recorded way back. The first bite (z9, lab PR 236) proved the watchdog bites 20 s after arming, but the phone then stayed off the USB bus (neither Android nor fastboot) and needs the author's hands (hold power ~15 s): the PMIC's bite ended in a power-off, not a reset. Until the research below answers it, phone work that needs the phone to come back by itself is limited to the tries that return through the SoC (the kernel's own reboot, panic=5, the SoC watchdog), which have always come back; the PMIC bite is not one of them.

Why
: The author is away and nothing may need his hands (his word); an experiment whose normal outcome is a dead phone is not one that returns by itself; the reset type of the PMIC's S2 stage is a PMIC setting (PON), readable and settable by the kernel, so the fix is a change of that setting, not a retry.

Rejected
: Repeating the bite to look for a different outcome; asking the author for a power-cycle after every bite; disabling the PMIC watchdog series (it is the upstreams lane's proof row and the series itself is fine).

About
: deep-sleep


<a id="2026-11-19-walleye-minimal-dtbo-try"></a>
### 2026-11-19-walleye-minimal-dtbo-try

Date
: 2026-11-18

Brief
: Keep the stock dtbo until the kernel's real faults are fixed; then try one minimal dtbo (empty overlays, stock ids) on slot b

Decision
: No dtbo write now: the id is known (0x1bc rev 3) and the overlay's noise is harmless with clk_ignore_unused. Later, one try with a dtbo built on the host from the stock table with every overlay empty, original dtbo.img kept for restore, health report after.

Why
: Overlay hypothesis was falsified as the cause of no-start (walleye-overlay-enables-what-result); a dtbo write cannot be read back, so it waits until it can buy something.

Rejected
: Writing a one-entry dtbo with only 2704: wrong id for walleye. Writing now: no fault it fixes. Dropping the dtbo partition: bootloader may refuse, no way back.

About
: walleye-first-fastboot-boot

Research
: walleye-dtbo-overlay-id-how-it-stands

Scope
: walleye keeper

Tracked by
: walleye-dtbo-minimal-try


<a id="2026-11-18-walleye-pmic-bite-read-then-warm"></a>
### 2026-11-18-walleye-pmic-bite-read-then-warm

Date
: 2026-11-18

Brief
: Read the PON S2 registers first (no bite), then bite only with the type read back as warm; hard after warm restarts

Decision
: Settle the power-off by reading 0x54-0x57, 0x5a, 0x5b, 0xc5 on z9 (no bite), make the S2 type a dt property written with the downstream sequence and read back, choose warm (0x01) for the first bite, hard (0x07) only once warm restarted the phone. The series' fixed hard default is raised to the upstreams lane as a question once measured.

Why
: qcom-pon is ruled out as the overwriter; the vendor never writes the WD type, so no source tells the walleye value; off-the-bus does not prove power-off. A read costs no bite. Warm resets the MSM without cycling PMIC rails, the same class as the reboots that always came back.

Rejected
: Another bite with hard to look (breaks the no-bite decision); switching to shutdown (never restarts); patching qcom-pon to own the type (it writes none of it, and the type belongs to the watchdog); guessing the vendor fdt value (the vendor never sets it).

About
: deep-sleep

Research
: walleye-pmic-bite-reset-type-how-it-stands

Scope
: walleye keeper, builder, upstreams lane

Tracked by
: walleye-pmic-wdt-bite-try-warm


<a id="2026-11-18-walleye-tps61099-fixed-regulator-supplier-disabled"></a>
### 2026-11-18-walleye-tps61099-fixed-regulator-supplier-disabled

Date
: 2026-11-18

Brief
: Model the TPS61099 as our own fixed regulator and disable the overlay's driverless /soc@0/tps61099, not a regulator-fixed at that node

Decision
: Step 2 of the charger-supplier decision is done this way: a regulator-fixed node of ours (pm8998 gpio 5, active high, no boot-on) carries the board's boost rail, and our dts sets status disabled on /soc@0/tps61099 so the overlay's copy of that node (compatible ti,tps61099, no driver) makes no device and fw_devlink DROPS the charger's wait (it is not met: external-vbus-supply still names the overlay's child, and qcom_smbx asks for no regulator, so the regulator has no consumer yet). c4 = c2 with that dtb only, default fw_devlink.

Why
: The bootloader merges the vendor overlay over our dtb and replaces a compatible we give /soc@0/tps61099, so the regulator-fixed the first decision named cannot live at that node (checked with fdtoverlay on the real dtbo entry 3). The regulator is what the board does for the rail; the disabled node is the one unmodelled thing, kept narrow and commented.

Rejected
: regulator-fixed at /soc@0/tps61099 (the overlay overwrites its compatible); a child-node regulator under the overlay's node (no device is made for a child of a non-bus node); leaving fw_devlink=permissive (hides every missing supplier); a tps61099 driver (none exists, a kernel change for one gpio); dropping the supplier from the dts alone (hides the part).

About
: walleye-first-fastboot-boot

Research
: walleye-c4-built

Amends
: 2026-11-18-walleye-charger-supplier-permissive-then-modelled

Tracked by
: walleye-try-c4


<a id="2026-11-18-walleye-restart-gate-before-charger-try"></a>
### 2026-11-18-walleye-restart-gate-before-charger-try

Date
: 2026-11-18

Brief
: No image that probes the charger is tried until a restart gate passes on its control; c4 waits behind c5 (module unloaded) and c6 (module loaded late)

Decision
: Every image that probes the charger is tried in two steps from one build: c5 (driver module not loaded) must pass tests/host/restart-check.sh first; then c6 loads it 40 s after enumeration. The gate reads the host journal: the gadget leaves the bus within 200 s of enumerating. No long try (more than the 180 s self-reboot) of a charger-probing image until the gate has passed with it; the deadman for such a try is the PMIC watchdog with a proven restart S2 type, never the apps watchdog or the init's reboot alone.

Why
: c3 left the bus never again for 28+ min (host journal) while the identical init restarted c2 at exactly 180 s: the reset request itself did not take effect, so no in-image reboot is a way back; the only visible difference is the charger probe. One build with the load decided by the command line keeps c5 and c6 byte-equal but for the cmdline (the c3 method), so a gate fail isolates the probe.

Rejected
: Trying c4 next (it probes the charger from boot: it would very likely hang again and cost the author's hands without telling which register); making the init stop the charger before reboot (the driver has no stop path and the cause is unproven); waiting for the apps watchdog (never shown to restart this phone; T3 left no marker); reading pstore after the hold (a PMIC hard reset loses DDR); a bisect of fw_devlink alone (the dmesg shows the charger the only new probe).

About
: walleye-first-fastboot-boot

Research
: walleye-c3-hang-how-it-stands

Scope
: walleye keeper, builder; walleye-try-c4, walleye-try-c5, walleye-try-c6

Tracked by
: walleye-try-c6


<a id="2026-11-18-walleye-c6-waits-for-the-author"></a>
### 2026-11-18-walleye-c6-waits-for-the-author

Date
: 2026-11-18

Brief
: c6 (the charger loaded late) waits for the author's presence; the safe tries and the builds go first

Decision
: c6 (the charger driver loaded 40 s into the try) is the one try of the lane whose realistic bad outcome is a phone that does not restart by itself (c3's reading, unproven). With the author away it waits for a moment when he can be at the phone, and runs then in ONE batch with the other risky tries (a PMIC bite read-back, the dtbo write), so a single power-hold covers them. Until then the phone queue works on tries that return by themselves: the build items go first (UFS read-only, USB networking, the phone-state wiring). No PMIC-side deadman is invented for the try: the PMIC watchdog's reset type is itself unproven (decision 2026-11-18-walleye-pmic-bite-read-then-warm).

Why
: The author's word: nothing may need his hands while he is away; one hold for several tries is cheaper for him than one per hang; the build and research queue is long enough to use the time.

Rejected
: Running c6 now and parking the phone on a hang (it would block UFS, networking and every other phone item until he is back); a deadman through the PMIC bite (its reset type is not known to restart).

About
: walleye-first-fastboot-boot


<a id="2026-11-19-walleye-ufs-listed-by-sysfs-and-one-dd"></a>
### 2026-11-19-walleye-ufs-listed-by-sysfs-and-one-dd

Date
: 2026-11-18

Brief
: List the UFS from sysfs and read the boot header with one read-only dd, in the init; no lsblk, no mount, no by-name links

Decision
: The z11 init lists the UFS itself: /bin/ufs-lsblk reads the SCSI disks and partitions (GPT names from the uevent PARTNAME) from sysfs, /bin/ufs-hdr reads 48 bytes of the partition boot_a with one dd. The proof test walleye-ufs-lsblk (tests/host/ufs-lsblk-check.sh) passes on a LUN, a partition named boot_a, the banner equal to the command's answer, and a header with magic ANDROID!.

Why
: Busybox 1.36 has no lsblk and the initramfs has no udev, so no /dev/disk/by-name exists; the kernel's own sysfs names every partition and needs no write. One dd of one sector is the least that reads a header.

Rejected
: A static lsblk or util-linux in the ramdisk (a second build for a listing sysfs gives); a udev or mdev rule for by-name links (more running code near the disk); mounting a partition read-only (a mount can replay a journal or write); reading the whole boot partition (nothing needs more than the header page).

About
: walleye-ufs-readonly

Research
: walleye-z11-built

Scope
: walleye

Tracked by
: walleye-try-z11


<a id="2026-11-18-walleye-ufs-research-before-the-try"></a>
### 2026-11-18-walleye-ufs-research-before-the-try

Date
: 2026-11-18

Brief
: No UFS probe on the phone until the yoshino failure mode is understood for the Pixel 2's card

Decision
: The z11 try (UFS probed by our own kernel) does not run until a research row has answered whether probing the Pixel 2's UFS can damage what the phone boots from. The builder found a hazard in the mainline record: the Sony Xperia XZ1 family (yoshino, MSM8998) keeps UFS disabled upstream because its card needs quirks and the probe left the bootloader erased. UFS holds this phone's bootloader and both Android slots, which is the only way back; a read-only disk setting in our init does not protect against what the host controller's reset and link startup do. So: first research (the Pixel 2's UFS device vendor/model/firmware, read from the running Android over adb: /sys/class/scsi_host, /sys/devices/platform/soc/*.ufshc, the vendor kernel's ufs dmesg lines, getprop; what upstream says about yoshino's failure mode, which quirks apply, and whether our kernel's UFS driver would apply them), then a decision with what is learned; if any real chance of a wipe remains the try needs the author's word (a risk to the phone's recovery goes up to him).

Why
: The lane's rule: nothing that can leave the phone without its bootloader or the Android reference runs on a guess; UFS is the one device whose misbehaviour cannot be undone from fastboot.

Rejected
: Running z11 on the strength of the read-only disk flags; waiting for the author without research (he would be asked an unready question).

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-ufs-try-needs-quirks-and-the-authors-word"></a>
### 2026-11-18-walleye-ufs-try-needs-quirks-and-the-authors-word

Date
: 2026-11-18

Brief
: z11 (UFS probe) stays unrun: first backport Sony's Hynix quirks in our kernel; then the try goes to the author, who owns the recovery risk

Decision
: The research is done: a real chance of the Sony failure remains (Hynix card, unpublished mechanism, no way back from an erased bootloader). Steps: (1) build item walleye-ufs-hynix-quirks: our kernel gets Sony's Hynix quirk set in the ufs-qcom fixups, a new image z12 with the UFS probe, built only; (2) research item walleye-ufs-card-identity: learn the card's model and firmware without probing it, to see whether it is one of Sony's two purge models; (3) only then the phone item walleye-try-z11 goes to the author with the plain risk. Until his word the UFS-enabled images do not run.

Why
: The measured card is Hynix, the maker Sony's quirks single out; upstream says nobody tested the mechanism and an erased bootloader has no known recovery; measured-not-claimed forbids running on a read-only init alone.

Rejected
: Run z11 now on the read-only init (the erase is not a write from our side, so read-only flags prove nothing); never enable UFS (no storage, no mainline distro); a bootloader backup as protection (unreadable without root, useless when fastboot is gone); copy the OnePlus 5 setup as proof (its card is not known to be Hynix).

About
: walleye-first-fastboot-boot

Research
: walleye-ufs-hazard-verdict

Amends
: 2026-11-18-walleye-ufs-research-before-the-try

Scope
: walleye lane: keeper, builder, coordinator, the author

Tracked by
: walleye-ufs-hynix-quirks


<a id="2026-11-18-walleye-hynix-quirks-in-the-fixups-table"></a>
### 2026-11-18-walleye-hynix-quirks-in-the-fixups-table

Date
: 2026-11-18

Brief
: Carry Sony's three Hynix UFS quirks in ufs-qcom's fixups table for every Hynix card; one lane applies after the device is read

Decision
: Backport as quirks keyed on SKHYNIX ANY_MODEL in our fork topic/ufs-hynix-quirks; z12 changes only the kernel

Why
: Sony: a probe without them erased the bootloader; purge refusal in core covers every path through ufshcd_query_flag

Rejected
: A Kconfig switch for one lane before link startup (Sony's way): restricts every UFS device of the kernel; model/revision-specific no-purge: the card's model is unreadable without root

About
: walleye-first-fastboot-boot

Research
: walleye-z12-built

Tracked by
: walleye-ufs-hynix-quirks


<a id="2026-11-18-walleye-android-disposable-no-bricking"></a>
### 2026-11-18-walleye-android-disposable-no-bricking

Date
: 2026-11-18

Brief
: Android on walleye is throw-away; the one hard line is bricking

Decision
: The author's word (2026-11-18, relayed by the top coordinator, verbatim): "if they need root access for more detailed analisys, they can add it, again android is to be treated like throw away , the only concern to avoid bricking the phone as much as we can if possible without crazy hurdles". So: (1) the Android on walleye is disposable: the lane may root it (LineageOS userdebug adb root, or a Magisk-patched boot on the inactive slot), change its settings, read anything from it, and data loss there does not matter; (2) the one hard line is bricking: never write the bootloader or its siblings (xbl, abl, devcfg, tz, hyp, keymaster and the other firmware partitions) or the partition tables, and keep at least one slot booting at all times with a recorded way back (the original boot/dtbo/vbmeta of a slot read back over adb root into the lab with sha256 BEFORE a write to that partition, restore by fastboot flash); (3) 'no crazy hurdles': a write to a slot's boot, dtbo or vbmeta partition with that backup and the other slot intact is the lane's call, recorded; fastboot flash/erase/set_active outside that remain refused by the guards; (4) UFS probing from our kernel stays decided separately (decision 2026-11-18-walleye-ufs-try-needs-quirks-and-the-authors-word): the author's bound is bricking, and a UFS probe is exactly that risk, so it goes to him with what root learned.

Why
: It is his word, in his own terms, for the thing that ends the lane (a bricked phone) and the thing that does not (Android's data and settings); it widens the reading side (root) at once and the writing side only with a backup.

Rejected
: Treating Android's state as precious (it slows every analysis); treating partition writes as free (the bootloader family and the partition tables stay out of any write); asking the author for each root step.

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-ufs-wait"></a>
### 2026-11-18-walleye-ufs-wait

Date
: 2026-11-18

Brief
: No UFS probe for now: wait for an EDL way back or the author's explicit go

Decision
: The author's answer on the UFS probe, verbatim (2026-11-18, relayed by the top coordinator): "Yes, wait". So: no UFS probe from our kernel on the phone for now. z13 (kernel topic/ufs-hynix-quirks d49d54405, boot-z13.img) stays BUILT AND UNRUN. The UFS items wait until an EDL way back for walleye is known, or the author says go explicitly. The lane goes on with USB networking, touch and display from the RAM-booted ramdisk. Researching an EDL way back (is EDL reachable on the Pixel 2, test points, what the public record says of recovering a Pixel 2 whose bootloader is gone) is a research item in its own right.

Why
: His bound is bricking; a UFS probe carries the one risk the lane cannot undo (the yoshino failure left the bootloader erased, and no way back is known); the real parts that need no storage are enough work for a long time.

Rejected
: Probing once for the identity 'to see' (the identity is already read as root); probing with the quirks and a guard that is checked only by source counts.

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-edl-no-way-back-wait-holds"></a>
### 2026-11-18-walleye-edl-no-way-back-wait-holds

Date
: 2026-11-18

Brief
: No way back is known, so the UFS wait holds until the author's explicit go; first, read back the boot chain as files (no risk)

Decision
: The condition 'an EDL way back is known' (decision 2026-11-18-walleye-ufs-wait) is NOT met: no Google-signed MSM8998 firehose programmer is public, no walleye test point is published, Google's repair tool does not cover the Pixel 2 (research 2026-11-18-walleye-edl-way-back). So the UFS items stay waiting; only the author's explicit go lifts it, knowing that an erased bootloader means the phone is lost. Meanwhile one risk-free step is queued for the keeper: read back the phone's own boot chain (xbl_a/b and the partitions listed in the research) over adb root as mode-600 files with sha256, kept on the lab machine, never in a repository. It restores nothing without a matching programmer; it costs nothing and is what a programmer or a service route would need.

Why
: Evidence: the owner of the edl tool says no Pixel loader was released or leaked (2021); a ~200-loader survey lists none; every Pixel 2 hard-brick thread 2018-2025 ends without a method; a loader for any other phone is refused by the fused PK hash. The author's bound is bricking; this keeps it.

Rejected
: Treating Google's factory image or fastboot as a way back (needs a running bootloader, the very thing lost). Buying or downloading a leaked loader from service sites (clean-room, and a mismatch is refused). Opening the phone to hunt test points (no public map; risk to the only reference phone). Entering EDL with adb reboot edl to 'see' (no recovery value without a programmer; a state change on the phone for nothing). Lifting the wait on our own because the probe is 'probably safe' (the mechanism of the Sony erase is unpublished).

About
: walleye-first-fastboot-boot

Research
: 2026-11-18-walleye-edl-way-back

Amends
: 2026-11-18-walleye-ufs-wait

Scope
: walleye lane: coordinator, keeper, builders; the author


<a id="2026-11-18-walleye-steps-open-not-failing"></a>
### 2026-11-18-walleye-steps-open-not-failing

Date
: 2026-11-18

Brief
: Steps nobody has tried are claimed open; failing only where a check ran and failed

Decision
: Of the 113 steps of docs' bring-up chain, walleye claims 17 done (each with proof, run and image), 3 partial, 1 failing (watchdog-way-back, from the z9 bite) and 92 open. A step nobody has tried on this phone is open, not unknown and not failing; charger-online is partial because c3 probed it online but the restart gate is not passed.

Why
: docs' bringup-chain-walleye test needs every step claimed; unknown would fail it, failing would say a check ran. open says: not done, not tried.

Rejected
: unknown for the untried steps (the chain test refuses it); the draft's failing for charger-online (c3's charger-check passed); the draft's open for watchdog-way-back (z9 bit and the phone stayed off)

About
: walleye-first-fastboot-boot

Research
: walleye-z9-tried-pmic-wdt-bites


<a id="2026-11-18-walleye-z16-rename-the-qup-node"></a>
### 2026-11-18-walleye-z16-rename-the-qup-node

Date
: 2026-11-18

Brief
: z16: rename our QUP node qup@c179000 so the overlay's i2c@c179000 cannot overwrite it; not a compatible edit

Decision
: Name our controller node at 0xc179000 qup@c179000 (same-length substitution of node and its two symbol paths in z15's dtb). The overlay's i2c@c179000 then stands as a separate driverless node. Check after the overlay merge (fdtoverlay with dtbo entry 3), not on our dtb alone.

Why
: The merge is by node name, so our dtb alone shows the right compatible while the merged tree has the downstream one (z15 proved it on the phone and fdtoverlay reproduces it).

Rejected
: Setting the compatible in our dtb (the item's first wording): already the value, overwritten by the overlay. Editing the dtbo: it is the phone's own partition and the control we compare against. Repointing the soc symbol or emptying the overlay target: hits every other fragment on soc. A dtc round trip to rename: rewrites the whole dtb for a one-word change; the byte substitution leaves nine bytes different.

About
: walleye-first-fastboot-boot

Research
: walleye-z16-built

Scope
: walleye builds from z16 on keep the node name qup@c179000

Tracked by
: walleye-build-z16-i2c-compatible


<a id="2026-11-19-walleye-overlay-collisions-as-a-class"></a>
### 2026-11-19-walleye-overlay-collisions-as-a-class

Date
: 2026-11-18

Brief
: The overlay merges by name: list every collision with one host test and rename them all; the dtbo write only as a fallback

Decision
: The stock dtbo overlay (entry 3, picked by the bootloader) is merged into our dtb by name, and a node of ours with the same path and name as one the overlay writes is overwritten (z16: i2c@c179000 under &soc; z17: pmi8998@3 under the spmi bus). Seen twice, so the class gets the root answer, not a third node-by-node fix: (a) a host test lists EVERY collision at once (every node path the overlay writes by name against our dtb's paths, with the properties it writes), and ALL colliding nodes are renamed in one change, keeping labels and phandles (the qup@c179000 move), so the overlay's nodes stand beside ours and ours stay intact; the test then guards every later dts change (a new node that collides fails the check before any phone try). (b) is not available: the bootloader chooses the entry and passes androidboot.dtbo_idx itself; its source is not public; nothing on a fastboot boot is known to make it skip or pick another entry (and writing nothing is the lane's rule). (c) the minimal dtbo write stays the fallback only for what cannot be renamed (a node whose name the binding fixes, like soc or the root), attended and with the phone's own dtbo backed up first; it becomes the answer only if the collision test finds such a node, and then the top coordinator puts it to the author. Class lines for the record: (1) what happened: the overlay rewrote our i2c node and our pmi8998 node by name, two phone tries apart; (2) why it fits the class 'a sign seen twice': one cause (a merge by name at boot that we do not control), two nodes found by two tries; (3) the answer: list the whole class with one host test and fix it whole, then make the test a gate.

Why
: It is the cheapest answer that holds: host-side, no phone try to find each collision, no write to the phone, and it turns a repeating surprise into a check.

Rejected
: Fixing nodes one by one as tries find them (each costs a phone try); writing a minimal dtbo now (an attended write for what renames can solve); steering the bootloader's overlay choice (not known to be possible).

About
: walleye-first-fastboot-boot


<a id="2026-11-18-walleye-z18-not-built-pmi8998-reg-no-effect"></a>
### 2026-11-18-walleye-z18-not-built-pmi8998-reg-no-effect

Date
: 2026-11-18

Brief
: Do not build z18 'pmi8998@3 gets a reg': it cannot change the pmi8998 gpio; first read why pmic@2's gpio is absent (facts from the init), then change the dtb

Decision
: The backlog item walleye-build-z18-pmi8998-reg stays open with a corrected note; no image is built from it. After the overlay-collisions test, the next build is z17's image plus an init (or loglevel) that reads the spmi devices, devices_deferred and the gpiochips, so the dtb change that follows rests on a read fact.

Why
: The reg-less nodes are the overlay's and the spmi core skips them with continue (v7.3 spmi.c); ours have reg; the config has the drivers. A build that cannot change the outcome costs a phone try.

Rejected
: Adding a reg to qcom,pmi8998@3 in our dtb: merged by name it would sit beside or inside our pmic@3 at the same usid and cannot reach pmic@2's gpio. Deleting the overlay's nodes: the bootloader merges the overlay, we cannot. Building z18 anyway as a try: no change, a wasted phone slot.

About
: walleye-first-fastboot-boot

Research
: walleye-z18-premise-falsified

Tracked by
: walleye-build-z18-pmi8998-reg


<a id="2026-11-18-walleye-overlay-collision-fix-stub-label"></a>
### 2026-11-18-walleye-overlay-collision-fix-stub-label

Date
: 2026-11-18

Brief
: A label the overlay targets is a stub, never a real node: delete mainline's pcie0 (disabled) and stub its label; the qup rename goes into the dts too

Decision
: The collision test (tests/host/overlay-collisions.sh) is the gate: an overwritten property of a node of ours is a collision unless the allow file names it. Fixes: /delete-node/ &pcie0 with a stub pcie0 label (regenerated stubs), blsp1_i2c5 redefined as qup@c179000; z19 carries the pcie one on z17. build-all.sh runs the test on the built dtb; CI runs its selftest.

Why
: z16 and z17/z18 were found by phone tries; the class is found by one host merge. The control dtbo is not in CI (proprietary), so CI proves the test, the lab's build gates the dts.

Rejected
: Allow-listing the pcie overwrite (a disabled node today; a later dts change enabling pcie would silently take the vendor reg); renaming the pcie node (the overlay reaches it by label); a Python or C diff tool (no new Python).

About
: walleye-first-fastboot-boot

Research
: walleye-z19-built

Tracked by
: walleye-overlay-collisions


<a id="2026-11-19-walleye-touch-gpio-diag-first"></a>
### 2026-11-19-walleye-touch-gpio-diag-first

Date
: 2026-11-18

Brief
: Settle which defer leaf the touch gpio hits with a read-only z-next read and a dyndbg diagnostic kernel; change no dts until a leaf is named

Decision
: No dts change is queued: the specifier is proven valid. Step 1: read-only z-next reads of the pinctrl and gpio-device facts. Step 2: a diagnostic image with CONFIG_DYNAMIC_DEBUG=y and CONFIG_DEBUG_GPIO=y and dyndbg for gpiolib, gpiolib-of and pinctrl core on the command line, so the next boot prints the exact leaf and errno. The fix is written from that line.

Why
: Source reading proves the specifier and removes candidate causes but cannot choose between two defer leaves; a dts guess would burn a z build and a phone try on a premise already falsified once (z18 renaming, lab PR 314).

Rejected
: Guess a dts fix (hog the gpio, add a pinctrl-0 pin node, rename the label, change the number): the specifier and pin config are proven not at fault; a hog uses the same request path and a failing hog drops the chip. Replace the regulator-fixed by something else or drop the supply: hides a defect every pmic gpio consumer will meet. Force a re-probe by unbind/bind on the phone: a write, and it repeats the silent failure without debug prints.

About
: walleye-first-fastboot-boot

Research
: walleye-touch-gpio-defer-leaves


<a id="2026-11-19-walleye-compare-with-android-for-every-part"></a>
### 2026-11-19-walleye-compare-with-android-for-every-part

Date
: 2026-11-18

Brief
: Every part starts with a captured comparison against live Android, before theory

Decision
: The author's question (2026-11-19, via the top coordinator, verbatim): "why cant we use comparison with what android does for everything like we did for storage ? is it too prici on time and effort ? cant be automated with small makeshift tools for analysis ?". The answer, and the method from now on: yes, for every part. Android runs the same hardware correctly every boot, and the live phone is readable as root (disposable Android: decision 2026-11-18-walleye-android-disposable-no-bricking). So the first step of every part, BEFORE theory about why our kernel does not bring it up, is a comparison tool run: capture the part's state on live Android over adb root (gpio, pinctrl, regulators, clocks, the part's device-tree nodes from /proc/device-tree, its driver's sysfs/debugfs, its dmesg lines), capture the same files from our kernel over the network shell, and diff them normalised: which pins are driven at what level and direction, which regulators are on at what voltage, which clocks run at what rate, which properties the running fdt carries that ours does not. It is cheap (one read-only phone visit of minutes and a small script of shell/awk/dtc; no Python), it replaces rounds of theory (the touch lane has used six phone tries on guesses), and it can be automated: the capture and diff script is kept in the lab (tests/host/part-compare.sh PART with a per-part file list), and orchestration may lift it into lpoco-bringup as a general helper. Order for each part: (1) the comparison run, (2) read what differs, (3) change the dts or config for exactly that, (4) the try.

Why
: Cost: small; benefit: it answers 'what does Android do that we do not' by measurement, which the theory rounds (z17, z18) could not; it is how storage was understood.

Rejected
: Reasoning from kernel source about why a part fails before looking at what the working Android sets (the pattern of the touch lane); building a diagnostic kernel for every part first.

About
: walleye-first-fastboot-boot


<a id="2026-11-19-walleye-ufs-off-by-a-gate"></a>
### 2026-11-19-walleye-ufs-off-by-a-gate

Date
: 2026-11-18

Brief
: UFS off in every image by a host gate; the accident told to the author plainly

Decision
: Until the author says go, UFS is OFF in every image of the lane, by a gate and not by hope: (1) the shared dts gets &ufshc and &ufsphy status = "disabled" (the z11/z13 UFS experiments live in their own dts variant, not the shared one); (2) tests/host/ufs-off-image.sh IMAGE (or DTB) unpacks a boot image, takes every appended dtb, merges the stock dtbo overlay entry 3 when the overlay could enable it, and fails if ufshc@1da4000 or the ufs phy is enabled; it runs on EVERY image before a keeper boots it (the keeper recipe says so) and in build-all.sh and CI on every built image; (3) the existing images that have UFS on (z11 to z20 and any built from main since) are listed and NOT booted again; a z21 is built with UFS off to continue the touch work; (4) the author is told plainly what happened, with the evidence that the card enumerated and was read at least four times and nothing was erased (bootloader intact), as information for his decision, not as a licence. Class lines: (a) what happened: a UFS enable meant for one parked image landed in the shared dts and every later image probed the card; (b) why it fits the class 'a decision held in prose, not in a check': the wait was a sentence in a record, the images were the thing that acted; (c) the answer: put every held decision where the artefact is checked (the image's dtb), and make the keeper test the artefact, not the item.

Why
: The author's bound is bricking; the lane broke the letter of his 'Yes, wait' without meaning to; the right response is a check that makes it impossible again, and honesty about what happened.

Rejected
: Leaving UFS on because the card survived (a few reads are no proof of safety in every sequence); hiding the finding; reverting images by hand without a gate.

About
: walleye-first-fastboot-boot


<a id="2026-11-19-walleye-stubs-disabled"></a>
### 2026-11-19-walleye-stubs-disabled

Date
: 2026-11-19

Brief
: Disable the overlay-stubs container: the stock overlay's nodes under our stub labels must not claim our gpios in the kernel's shared-gpio scan

Decision
: /vendor-overlay-stubs is status = disabled (generator and dtsi).

Why
: gpiolib-shared scans every available node; the overlay's synaptics_dsx@20 under stub i2c_5 named pmi8998 gpio 2 like our touch regulator, making it shared with a proxy that never registers (research walleye-z20-tried-touch-gpio-leaf). Disabled nodes and their children are skipped; the merge by label is unaffected.

Rejected
: Editing or replacing the vendor dtbo (the phone's own, not ours to change); moving only i2c_5 out of the container (the same claim recurs for any other stub); dropping our regulator's gpio (the vendor wiring is right; the second claim is the fault); fixing the shared-gpio proxy (a core problem for a claim that should not exist).

About
: walleye-first-fastboot-boot

Research
: walleye-z21-built

Scope
: walleye dtb, build/gen-overlay-stubs.sh

Tracked by
: walleye-touch-gpio-shared-proxy


<a id="2026-11-19-walleye-pmic-wdt-v1-test-first"></a>
### 2026-11-19-walleye-pmic-wdt-v1-test-first

Date
: 2026-11-19

Brief
: Test the PMIC watchdog v1 on the Pixel 2 before the series is sent: one attended bite, at least through patch 2

Decision
: The author said "Yes, test first" (relayed by the top coordinator, docs 604). Build a Pixel 2 image with the v1 series (docs research/upstream-forks/2026-11-19/pmic-wdt-gen2-v1: patch 1 sets the reset type with S2 disabled and reads it back, patch 2 stops the watchdog on reboot; the pm8998 support and node as the series has them), with the UFS-off gate passing, and make one bite on the author's next lab visit, as an attended item (witness person). Pass: the phone restarts by itself (same slot, uptime reset, bootstatus CARDRESET). Arm only when the type reads back as the one set (warm 0x01 first, hard 0x07 only if warm restarted); a power-off means the author holds power about 15 s. The series stays unsent until the result is in; the result goes to the upstreams coordinator.

Why
: Patch 1 was written for exactly the z9 case (the phone powered off at the bite) and was never tested on this phone; sending a fix untested on the one board that showed the fault would put an unproven claim in the cover letter. The earlier decision (no bite until the reset type reads back as a restart) is met by the read-back in patch 1.

Rejected
: Sending the series first and testing later: the cover letter says patch 1 is untested on the Pixel 2. Arming the bite from the command loop on boot: a restart would land in a loop; arm only after the read-back, as the warm-first decision says.

About
: walleye-pmic-wdt-proof


<a id="2026-11-19-walleye-pmic-wdt-v1-reboot-and-next"></a>
### 2026-11-19-walleye-pmic-wdt-v1-reboot-and-next

Date
: 2026-11-19

Brief
: PMIC watchdog v1 test: also the stop-on-reboot path, and an image from the next-based tree, in the one attended session

Decision
: Add to the attended session (the upstreams lane's list of what is unproven, docs 607, relayed by the top coordinator): (1) the bite with patch 1 restarts; (2) arm, then a clean reboot, and the next boot shows S2 off and no loop; (3) build image A from the next-based v1 tree itself so what is mailed is what booted, and image B (v7.3 plus six patches) as the fallback if next does not boot walleye, both built before the visit. The reboot test goes before the bite.

Why
: Patch 2 (stop on reboot) has a hardware-found reason on another phone and no test on this one; two proofs in one visit keep the author's attention cost to one session; building both images ahead means a failed next boot costs no second visit.

Rejected
: One image only, the v7.3 backport: what boots would not be what is mailed. A second visit for the reboot path: more of the author's time.

About
: walleye-pmic-wdt-proof

Amends
: 2026-11-19-walleye-pmic-wdt-v1-test-first


<a id="2026-11-19-walleye-pmic-wdt-v1-image-choices"></a>
### 2026-11-19-walleye-pmic-wdt-v1-image-choices

Date
: 2026-11-19

Brief
: PMIC v1 images: the S2-type guard compares with 7 (what the driver sets); the SoC petter is found by identity; image A is the six mailed patches on the next tag

Decision
: (1) wdt-bite and wdt-reboot refuse unless the PMIC watchdog device exists and the S2 type reads back as 7, the type the driver always sets (RESET_TYPE_HARD); wdt-bite 1 asks for the warm type on purpose. (2) The SoC watchdog petter opens the device whose sysfs identity is qcom_wdt, never /dev/watchdog. (3) Image A is next-20261118 plus the six patches of the upstreams lane's series applied here with git am, not the lane's own tree. (4) The command line replaces z21's gpio dyndbg with the driver's dyndbg; no DEBUG_GPIO. (5) The clock_gcc symbol is pointed at the real gcc after the build, as z3 did, and the image build checks it.

Why
: (1) driver source of v7.3 topic and of the v1 tree: pm8916_wdt_set_reset_type writes RESET_TYPE_HARD and fails the probe if the read-back differs; the dts has no type property, so 'warm first' in the item cannot be asked of this driver, and a read-back of 1 would mean the firmware's type stayed. (2) With the PMIC watchdog enabled in the dts either device can be watchdog0, and opening a watchdog device starts it: the first image of this kind must not arm by opening the wrong one. (3) the lane's tree is not on this machine; the tag e8017d290 matches the base of its notes and all six patches applied clean, so the content is the mailed one. (4) the diagnostic gpio output would bury the two lines the proof needs. (5) tests/host/overlay-clock-gcc.sh fails on a dtb straight from the kernel build.

Rejected
: Arming at boot after a delay: the item and the author's 'Yes, test first' want an attended, explicit bite. Using /dev/watchdog as before: may arm the PMIC. A dts property for the type: no such binding. Waiting for the lane's tree: it would block the one attended visit on a copy that is the same patches.

About
: walleye-pmic-wdt-proof

Research
: walleye-pmic-wdt-v1-images-built

Tracked by
: walleye-pmic-wdt-v1-image


<a id="2026-11-19-walleye-pmic-wdt-v1-type-is-seven"></a>
### 2026-11-19-walleye-pmic-wdt-v1-type-is-seven

Date
: 2026-11-19

Brief
: The v1 bite try uses the type the series sets, hard 0x07; warm-first cannot be asked of this driver

Decision
: The attended PMIC try (walleye-try-pmic-wdt-v1) bites with the series as it will be mailed: the driver always sets S2 type 0x07 (hard reset) and the dts cannot choose, so the guard compares the read-back with 7 and the keeper stops, with no bite, if the banner reads anything else (0x01 means the driver did not set it). The earlier plan warm 0x01 first is dropped for this try. The way back stays: nothing is written to storage, the reboot test goes first, the bite last with the author at the phone, and a power-off or a loop ends with him holding power 15 s.

Why
: The try's value is that what is mailed is what booted; a dts choice of type would test something the series does not do. The danger of a hard type is the one already accepted by the author's Yes, test first: the phone may power off and need his hands, which costs 15 s of holding power and writes nothing.

Rejected
: Adding a dts property for the type just for the test: the test would no longer be the series. Warm 0x01 first by patching the driver locally: same reason.

About
: walleye-pmic-wdt-proof

Amends
: 2026-11-18-walleye-pmic-bite-read-then-warm


<a id="2026-11-19-walleye-upstream-dts-trim"></a>
### 2026-11-19-walleye-upstream-dts-trim

Date
: 2026-11-19

Brief
: Upstream board dts: APSS wdt dropped, adreno_smmu kept disabled with a reason, UFS off, GPL-2.0-only kept

Decision
: The upstream series drops the APSS qcom,kpss-wdt node (no msm8998.dtsi patch), keeps &adreno_smmu disabled as a board override with a comment, leaves UFS out, and keeps GPL-2.0-only.

Why
: The wdt address and bark interrupt are downstream values and no run showed a bite resetting the phone (T3 was the alive class); the smmu's power domain needs the GPU rail nothing describes; UFS is off by decision 2026-11-19-walleye-ufs-off-by-a-gate; the file derives from GPL-only wahoo board files so it cannot be dual-licensed by us.

Rejected
: A wdt patch to msm8998.dtsi now (no proof, a wrong value would reset real boards); dropping the adreno_smmu node override (left on it logs gpu_gx WARNs and holds sync_state); GPL-2.0-only OR BSD-3-Clause (not ours to give).

About
: walleye-first-fastboot-boot

Research
: walleye-upstream-dts-series

Tracked by
: walleye-upstream-dts-polish


<a id="2026-11-19-walleye-bisect-by-series"></a>
### 2026-11-19-walleye-bisect-by-series

Date
: 2026-11-19

Brief
: Walleye's bisects are series files for lpoco-bringup image bisect; build-bisect, check-bisect and build-pk456 go, the dtb-making scripts stay

Decision
: build/series/*.toml declare every bisect done so far; build-bisect.sh, check-bisect.sh and build-pk456.sh are removed. build-dtb-bisect.sh, build-taimen-walleye.sh, build-property-bisect.sh and build-fdt-bisect.sh stay for the node and property edits that make their dtbs; the series take those dtbs as recorded.

Why
: The reproduction passed: 35 of 35 images byte-equal to the recorded sha256, each counted by the command (research walleye-series-reproduce-the-bisects).

Rejected
: Removing the four dtb scripts too: their fdtput edits are what made d*, w*, wp*, x* and image bisect does not edit dtbs, so the recipe would be lost. Series paths absolute: they would break in a worktree; ../../../../walleye-build holds for the clone and for worktrees beside it.

About
: walleye-first-fastboot-boot

Research
: walleye-series-reproduce-the-bisects

Tracked by
: walleye-use-lpoco-bisect


<a id="2026-11-19-walleye-fuel-gauge-three-patches"></a>
### 2026-11-19-walleye-fuel-gauge-three-patches

Date
: 2026-11-19

Brief
: Fuel gauge try: carry the RFC's first three patches as files, enable the node in our dts, keep z21's ramdisk

Decision
: Build fg1 from v7.3 + pmic-wdt topic + RFC patches 1-3 (kept as files in kernel/fuel-gauge-rfc/ with SHA256SUMS), our dts enables &pmi8998_fg with monitored-battery and the charger, z21's ramdisk and command line unchanged but for dyndbg on the driver. Judged by tests/host/fuel-gauge-check.sh against battery-ok.sh's fastboot reading, within 100 mV.

Why
: Only the binding, driver and node are needed for walleye; z21's init already has the command loop that reads power_supply. One change per try.

Rejected
: Taking all eight patches: five are other boards' dts. Porting the vendor battery profile into a table of ours: the driver does not load one. Changing the init for the try: a second change in one image.

About
: walleye-first-fastboot-boot

Research
: walleye-fuel-gauge-built

Tracked by
: walleye-try-fuel-gauge
