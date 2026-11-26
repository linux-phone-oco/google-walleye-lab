#!/usr/bin/env rust-script
//! Install stage 1 as ONE command (item walleye-install-write-procedure; decisions 2026-11-20-walleye-install-stage-1-go,
//! 2026-11-19-walleye-attended-tries-one-command): the Alpine image written from Android to the system partition of the slot
//! Android is NOT on (read at the run: fastboot's current-slot equal to adb's slot suffix), then that partition READ BACK and
//! its sha256 equal to the image's, and every other partition of both slots read again and equal to the stored backup. ONE dd
//! writes, to system_a or system_b and no other partition; no boot, no set_active, no table, no erase. Any doubt stops it before
//! the write. The way back: Android's own slot is never written; the stored backup of the other one is put back by hand.
//!   procedures/install-rootfs-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.install-rootfs-on-the-pixel-2], [images.alpine-rootfs])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps::read_slot, phone_on};
use walleye_tries::{install::Install, settings::{Settings, INSTALL_PROCEDURE}, watch};

fn main() {
    let _run = procedure(INSTALL_PROCEDURE, "google-walleye");
    let install = Install::new(Settings::for_install()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(install.way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-install-stage-how-it-stands", install.battery_is_good());
    check_from("the backup of both slots is whole", "research:walleye-install-stage-how-it-stands", install.the_backup_is_whole());
    check_from("the image is the declared one and fits", "research:walleye-install-stage-how-it-stands", install.the_image_is_the_declared_one());
    step("say what to expect", install.author_is_told());
    step("take the phone to fastboot", install.phone_is_in_fastboot());
    check_from("the slot Android runs from is read, the other one is the target", "research:walleye-install-stage-how-it-stands", install.the_slot_is_read());
    step("the phone is on Android, adbd as root", install.the_phone_is_on_android_as_root());
    step("stage the image on the phone and check the copy", install.the_image_is_staged());
    run_step(read_slot(phone_on("android")).sourced("research:walleye-install-stage-how-it-stands")); // the phone says _a or _b, as the library reads it
    check_from("adb and fastboot name the same slot", "research:walleye-install-stage-how-it-stands", install.the_slots_agree());
    check_from("the target partition, read now, is its backup", "research:walleye-install-slot-readback-done-twice-same-bytes", install.the_backup_is_the_slot_as_it_is_now());
    step("write the image to the target partition: the library's adb_write_partition, one dd and a read-back by sha256", install.the_image_is_written());
    check_from("the partition reads back as the image", "research:walleye-install-slot-readback-done-twice-same-bytes", install.the_partition_reads_back_as_the_image());
    check_from("every other partition of both slots is as its backup", "research:walleye-install-slot-readback-done-twice-same-bytes", install.the_other_partitions_are_unchanged());
    step("take the staged copy off the phone", install.the_stage_is_removed());
    step("the phone is on Android (the way back, if it is not)", install.way_back());
    check_from("the phone is back on Android", "research:walleye-install-stage-how-it-stands", install.android_is_back());
}
