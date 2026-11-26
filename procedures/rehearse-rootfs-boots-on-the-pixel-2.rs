#!/usr/bin/env rust-script
//! The rehearsal of the read-only UFS-on boot (item walleye-rootfs-boots-rehearsal; decision 2026-11-20-walleye-install-stage-1-go): every
//! step of procedures/rootfs-boots-on-the-pixel-2.rs except the boot - the battery, the image and its root, the stored boot chain and the
//! key, the card read from Android (the GPT of each disk, the root partition's hash), the guard with fastboot, the slots' flags, the run image made (the declared one with the slot read on its command line, checked equal in every other byte) - and
//! after it the card, the boot chain and the flags read again and compared. It boots nothing, writes no partition (only the made run image, in its own folder), and needs no go.
//! The phone ends on Android.
//!   procedures/rehearse-rootfs-boots-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.rehearse-rootfs-boots-on-the-pixel-2], [images.ufs-root])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::prelude::*;
use walleye_tries::{rootfs_boot::RootfsBoot, settings::{Settings, REHEARSE_ROOTFS_BOOT_PROCEDURE}, watch};

fn main() {
    let _run = procedure(REHEARSE_ROOTFS_BOOT_PROCEDURE, "google-walleye");
    let root = RootfsBoot::new(Settings::for_rehearse_rootfs_boot()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(root.rehearsal_way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", root.battery_is_good());
    check_from("the image is the declared one and names one root", "research:walleye-rootfs-boots-run-5-passed", root.the_image_is_the_declared_one());
    check_from("the stored boot chain is whole and the ssh key is private", "research:walleye-rootfs-boots-run-5-passed", root.the_stored_set_and_the_key_are_good());
    step("say what to expect", root.rehearsal_is_told());
    step("the phone is on Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("adb reads the slot Android runs from", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_is_read_by_adb());
    check_from("the card is read from Android: its partition tables and the root partition", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_read_before());
    step("take the phone to fastboot", root.phone_is_in_fastboot());
    check_from("fastboot agrees on the slot and the image's root is the other one", "research:walleye-rootfs-boots-run-5-passed", root.the_guard_holds());
    step("make this run's image: the declared one with the slot read on its command line", root.the_run_image_is_made());
    check_from("the slots' flags are read", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_read());
    step("take the phone out of fastboot: Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("the partition tables and the root partition are as before", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_as_before());
    check_from("the boot chain read again equals the stored set", "research:walleye-rootfs-boots-run-5-passed", root.the_boot_chain_is_as_stored());
    check_from("both slots' flags are as before", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_as_before());
}
