#!/usr/bin/env rust-script
//! UFS try 2 as ONE command (item walleye-ufs-try-2-one-command; decisions 2026-11-19-walleye-attended-tries-one-command,
//! 2026-11-19-walleye-ufs-hold-second-path, 2026-11-19-walleye-ufs-android-derived-solution): the matched UFS-on image booted
//! once from fastboot, from memory; the card read as in try 1; then the link gear and the clock before, during (a bounded
//! read-only O_DIRECT read of one disk) and 2 s after it; the disk's GPT hashed before and after; the slots' flags before and after.
//! The boot runs only while the author's go is recorded on the item the manifest names (no go: the run stops there and the phone
//! goes back to Android). NOTHING is written to the card. The UFS-off gate refuses this image by design and is not run, weakened or bypassed.
//!   procedures/ufs-matched-image-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.ufs-matched-image-on-the-pixel-2])
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.41.0" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::{prelude::*, steps};
use walleye_tries::{settings::{Settings, UFS_PROCEDURE}, ufs::Ufs, watch};

fn main() {
    let _run = procedure(UFS_PROCEDURE, "google-walleye");
    let card = Ufs::new(Settings::for_ufs()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(card.way_back()); // whatever ends a real run, the way back runs after
    }

    check("the battery is good enough", card.battery_is_good());
    check("the image is the matched one, its tests pass", card.image_is_the_matched_one());
    step("take the phone to fastboot", card.phone_is_in_fastboot());
    check("the slots' flags are read", card.the_slot_flags_are_read());
    step("say what to expect", card.author_is_told());
    check("the author's go is recorded on the item", card.author_has_said_go());
    step("boot the matched UFS-on image from memory, nothing flashed", card.boot_matched_image());
    run_step(steps::wait_for_gadget(30, 5));
    step("the shell answers and the hold is on", card.shell_answers_and_holds());
    check("the probe lines are in the kernel log", card.the_probe_lines_are_read());
    check("the link is read", card.the_link_is_read());
    check("the card's disks are listed, read only", card.the_luns_are_listed());
    check("the card's six disks and the partition tables are read", card.the_card_is_identified());
    check("the GPT of the disk is hashed", card.the_gpt_is_hashed_before());
    check("the gear and the clock, before any load", card.the_gear_is_read_before_load());
    step("a bounded read-only read of the disk, the gear and the clock sampled during it", card.the_gear_is_read_under_load());
    check("the gear and the clock, 2 s after the read", card.the_gear_is_read_two_seconds_after());
    check("the GPT is hash-equal to before", card.the_gpt_is_unchanged());
    check("no purge was asked", card.no_purge_was_asked());
    step("release the hold: the phone restarts by itself", card.the_hold_is_released());
    step("the phone is on Android (the way back, if it is not)", card.way_back());
    check("the phone is back on Android", card.android_is_back());
    check("both slots' flags are as before", card.the_slot_flags_are_as_before());
}
