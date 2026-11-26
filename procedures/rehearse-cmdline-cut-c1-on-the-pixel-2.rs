#!/usr/bin/env rust-script
//! The rehearsal of check C1 (item walleye-cmdline-cut-check-slot-a-procedure; procedures/cmdline-cut-c1-on-the-pixel-2.rs): every step of it except the slot
//! switch, the boot and what follows - the battery, the debug image and its header, the stored boot chain, the reboot tool, the card read from Android (the
//! GPT of each disk, the root partition's hash), the guard with fastboot, the slots' flags - and after it the card and the boot chain read again and compared.
//! It switches no slot, boots nothing, writes no partition, and needs no go. The phone ends on Android.
//!   procedures/rehearse-cmdline-cut-c1-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.rehearse-cmdline-cut-c1-on-the-pixel-2], [images.ufs-root-booted-slot-debug])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::cmdline_cut::CmdlineCut;
use walleye_tries::lpoco_attended::prelude::*;
use walleye_tries::{settings::{Settings, REHEARSE_CMDLINE_CUT_PROCEDURE}, watch};


fn main() {
    let _run = procedure(REHEARSE_CMDLINE_CUT_PROCEDURE, "google-walleye");
    let c = CmdlineCut::new(Settings::for_rehearse_cmdline_cut()); // every key from device.toml, read here
    let root = c.shared();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(c.rehearsal_way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", root.battery_is_good());
    check_from("the image is the declared debug image: its telnet word once, no slot, names one root", "research:walleye-cmdline-tail-cut-candidates", c.the_image_is_the_declared_debug_image());
    check_from("the stored boot chain is whole and the reboot tool is the declared one", "research:walleye-rootfs-boots-run-5-passed", c.the_stored_set_and_the_tool_are_good());
    step("say what to expect", c.rehearsal_is_told());
    step("the phone is on Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("adb reads the slot Android runs from", "research:walleye-rootfs-boots-run-5-passed", c.the_slot_is_read_by_adb());
    check_from("the card is read from Android: its partition tables and the root partition", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_read_before());
    step("take the phone to fastboot", root.phone_is_in_fastboot());
    check_from("fastboot agrees on the slot and the image's root is the other one", "research:walleye-rootfs-boots-run-5-passed", root.the_guard_holds());
    check_from("the slots' flags are read", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_read());
    step("take the phone out of fastboot: Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("the partition tables and the root partition are as before", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_as_before());
    check_from("the boot chain read again equals the stored set", "research:walleye-rootfs-boots-run-5-passed", root.the_boot_chain_is_as_stored());
    check_from("both slots' flags are as before", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_as_before());
}
