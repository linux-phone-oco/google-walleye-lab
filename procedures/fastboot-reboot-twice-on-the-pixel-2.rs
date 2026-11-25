#!/usr/bin/env rust-script
//! The timed fastboot excursion as ONE command (item walleye-fastboot-reboot-twice-procedure, for the try walleye-fastboot-reboot-twice-timed-try; research
//! walleye-fastboot-reboot-stays-in-fastboot-causes-ranked, decision 2026-11-19-walleye-attended-tries-one-command): from Android the phone is taken to its bootloader,
//! `fastboot reboot` is sent and timed (Android waited for up to back_wait), a second `fastboot reboot` only when the phone still lists in fastboot, then the phone goes to
//! the bootloader once more and `fastboot continue` is sent, timed. Every call's line (fastboot's words, the time it was sent, the seconds until Android answered) is kept in
//! the run's folder (legs.txt, verdict.txt) for the research row. NOTHING is flashed, set or written and no image is booted, so no go is read; before and after, the card's
//! partition tables, the root partition, the stored boot chain and both slots' flags are read and must be equal; the way back is the shared one.
//!   procedures/fastboot-reboot-twice-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.fastboot-reboot-twice-on-the-pixel-2])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::prelude::*;
use walleye_tries::reboot_twice::RebootTwice;
use walleye_tries::{settings::{Settings, REBOOT_TWICE_PROCEDURE}, watch};


fn main() {
    let _run = procedure(REBOOT_TWICE_PROCEDURE, "google-walleye");
    let t = RebootTwice::new(Settings::for_reboot_twice()); // every key from device.toml, read here
    let root = t.shared();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(t.way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-rootfs-boots-run-5-passed", root.battery_is_good());
    check_from("the stored boot chain is whole", "research:walleye-rootfs-boots-run-5-passed", t.the_stored_set_is_good());
    step("say what to expect", t.author_is_told());
    step("the phone is on Android, adbd as root", root.the_phone_is_on_android_as_root());
    check_from("adb reads the slot Android runs from", "research:walleye-rootfs-boots-run-5-passed", t.the_slot_is_read_by_adb());
    check_from("the card is read from Android: its partition tables and the root partition", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_read_before());
    step("take the phone to fastboot", root.phone_is_in_fastboot());
    check_from("the slots' flags are read", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_read());
    step("send `fastboot reboot` (a second only if the phone still lists), then `fastboot continue` on a second excursion; each call and its time kept", t.the_legs_are_sent());
    step("the phone is on Android (the way back, if it is not)", t.way_back());
    check_from("the phone is back on Android", "research:walleye-fastboot-reboot-stays-in-fastboot-causes-ranked", root.android_is_back());
    step("Android answers as root again", root.the_phone_is_on_android_as_root());
    check_from("the partition tables and the root partition are as before", "research:walleye-stop-stage1-check-17-three-answers", root.the_card_is_as_before());
    check_from("the boot chain read again equals the stored set", "research:walleye-rootfs-boots-run-5-passed", root.the_boot_chain_is_as_stored());
    check_from("both slots' flags are as before", "research:walleye-rootfs-boots-run-5-passed", root.the_slot_flags_are_as_before());
}
