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
