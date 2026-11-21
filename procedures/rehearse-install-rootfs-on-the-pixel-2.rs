#!/usr/bin/env rust-script
//! The rehearsal of install stage 1 (item walleye-install-rehearsal; decision 2026-11-21-walleye-install-write-one-dd-gates): every step
//! of procedures/install-rootfs-on-the-pixel-2.rs except the write - the battery, the backup, the image, the slot read in fastboot and in
//! adb (they must agree), adb as root, the image staged and its copy hashed, the target partition read now and compared with its backup -
//! and a dd probe: the write's own dd flags copy a few blocks of the image to a small file in the stage folder, read back by sha256, removed.
//! It has no write step: NO partition is written. The staged copy is removed at the end and the phone ends on Android.
//!   procedures/rehearse-install-rootfs-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.rehearse-install-rootfs-on-the-pixel-2], [images.alpine-rootfs])
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.41.0" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::prelude::*;
use walleye_tries::{install::Install, settings::{Settings, REHEARSE_INSTALL_PROCEDURE}, watch};

fn main() {
    let _run = procedure(REHEARSE_INSTALL_PROCEDURE, "google-walleye");
    let install = Install::new(Settings::for_install()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(install.rehearsal_way_back()); // whatever ends a real run, the way back runs after
    }

    check("the battery is good enough", install.battery_is_good());
    check("the backup of both slots is whole", install.the_backup_is_whole());
    check("the image is the declared one and fits", install.the_image_is_the_declared_one());
    step("say what to expect", install.rehearsal_is_told());
    step("take the phone to fastboot", install.phone_is_in_fastboot());
    check("the slot Android runs from is read, the other one is the target", install.the_slot_is_read());
    step("the phone is on Android, adbd as root", install.the_phone_is_on_android_as_root());
    step("stage the image on the phone and check the copy", install.the_image_is_staged());
    check("adb and fastboot name the same slot", install.the_slots_agree());
    check("the target partition, read now, is its backup", install.the_backup_is_the_slot_as_it_is_now());
    step("the write's dd copies blocks of the image to a file and reads back the same", install.the_dd_works_on_a_file());
    step("take the staged copy and the probe off the phone", install.the_stage_is_removed());
    step("the phone is on Android (the way back, if it is not)", install.rehearsal_way_back());
    check("the phone is back on Android", install.android_is_back());
}
