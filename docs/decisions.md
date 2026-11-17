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
