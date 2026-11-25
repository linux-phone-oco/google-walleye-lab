#!/usr/bin/env rust-script
//! Check C1 of the cut command line as ONE command (item walleye-cmdline-cut-check-slot-a-procedure, run by item walleye-cmdline-cut-check-slot-a;
//! research walleye-cmdline-tail-cut-candidates, its cmdline-cut-checks.md; decision 2026-11-19-walleye-attended-tries-one-command): with slot a
//! the CURRENT slot, the stage 2 debug image is booted once from fastboot, from memory (nothing flashed, its own command line, no slot put on it),
//! and /proc/cmdline is read over its telnet: does androidboot.slot_suffix=_a arrive, or is the line cut after androidboot.pk=g-pxl-p?
//! Before: adb's slot and fastboot's agree, the image's root is the other slot's system, the card, the boot chain and the slots' flags are read, and the
//! author's go is recorded on the item the manifest names for this image's sha256. The ONE change is `fastboot set_active` of the slot Android is not on
//! (its intent written first); the way back sends the reboot tool over the image's nc (sha256 checked on the phone), goes through the bootloader, sets
//! Android's slot current BEFORE any reboot, and the card, the boot chain and the flags are read again. NOTHING is flashed or written to the card.
//! The answer (whole or cut, with its length) is kept in the run's folder (cmdline.txt, verdict.txt) for the research row.
//!   procedures/cmdline-cut-c1-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.cmdline-cut-c1-on-the-pixel-2], [images.ufs-root-booted-slot-debug])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::cmdline_cut::CmdlineCut;
use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{settings::{Settings, CMDLINE_CUT_PROCEDURE}, watch};


fn main() {
    let _run = procedure(CMDLINE_CUT_PROCEDURE, "google-walleye");
    let c = CmdlineCut::new(Settings::for_cmdline_cut()); // every key from device.toml, read here
    let root = c.shared();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(c.way_back()); // whatever ends a real run, Android's slot is set back and the phone sent to Android
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", root.battery_is_good());
    check_from("the image is the declared debug image: its telnet word once, no slot, names one root", "research:walleye-cmdline-tail-cut-candidates", c.the_image_is_the_declared_debug_image());
    check_from("the stored boot chain is whole and the reboot tool is the declared one", "research:walleye-rootfs-boots-run-5-passed", c.the_stored_set_and_the_tool_are_good());
    step("say what to expect", c.author_is_told());
    step("the phone is on Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("adb reads the slot Android runs from", "research:walleye-rootfs-boots-run-5-passed", c.the_slot_is_read_by_adb());
    check_from("the card is read from Android: its partition tables and the root partition", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_read_before());
    step("take the phone to fastboot", root.phone_is_in_fastboot());
    check_from("fastboot agrees on the slot and the image's root is the other one", "research:walleye-rootfs-boots-run-5-passed", root.the_guard_holds());
    check_from("the slots' flags are read", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_read());
    check_from("the author's go is recorded on the item", "research:walleye-rootfs-boots-run-5-passed", root.author_has_said_go());
    step("make the slot Android is not on current (the one change; intent written first)", c.the_other_slot_is_made_current());
    step("boot the debug image from memory with its own command line, nothing flashed", c.boot_the_debug_image());
    run_step(steps::wait_for_gadget(30, 5));
    check_from("the command line is read over telnet: whole with the slot suffix, or cut at the pk word", "research:walleye-cmdline-tail-cut-candidates", c.the_command_line_is_read());
    step("send the phone to its bootloader: the reboot tool over the image's nc, checked on the phone", c.the_phone_is_sent_to_the_bootloader());
    check_from("in the bootloader Android's slot is made current again and its flags are good", "research:walleye-cmdline-tail-cut-candidates", c.the_switch_is_made_back());
    step("the phone is on Android (the way back, if it is not)", c.way_back());
    check_from("the phone is back on Android", "research:walleye-rootfs-boots-run-5-passed", root.android_is_back());
    step("Android answers as root again", root.the_phone_is_on_android_as_root());
    check_from("the partition tables and the root partition are as before", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_as_before());
    check_from("the boot chain read again equals the stored set", "research:walleye-rootfs-boots-run-5-passed", root.the_boot_chain_is_as_stored());
}
