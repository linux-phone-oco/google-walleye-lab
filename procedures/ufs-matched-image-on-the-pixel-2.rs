#!/usr/bin/env rust-script
//! The UFS try on the matched image (item walleye-attend-program-ufs; decisions 2026-11-19-walleye-attended-tries-one-command,
//! 2026-11-19-walleye-ufs-hold-second-path, 2026-11-19-walleye-ufs-android-derived-solution).
//! One command: the battery, the matched image checked (its sha256 and tests/host/ufs-match-android.sh), the phone to fastboot,
//! THEN the one step that boots the UFS-on image, which runs only while the author's go is recorded on the item the manifest names
//! (without it the run stops there, "no go", and the phone goes back to Android); then the kernel log's probe lines, the link, the
//! card's disks and the purge read, READ ONLY (nothing is written to the card), then the way back.
//! The UFS-off gate (tests/host/ufs-off-image.sh) refuses this image by design and is not run here, not weakened and not bypassed.
//!   procedures/ufs-matched-image-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.35.1" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    settings::{Settings, UFS_PROCEDURE},
    ufs::Ufs,
    watch,
};

fn main() {
    let _run = procedure(UFS_PROCEDURE, "google-walleye");
    let card = Ufs::new(Settings::for_ufs());
    // a real run is watched: whatever ends it (no go, a failed check, Ctrl-C), the way back runs after
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(card.way_back());
    }

    check("the battery is good enough", card.battery_is_good());
    check("the image is the matched one, its tests pass", card.image_is_the_matched_one());
    step("take the phone to fastboot", card.phone_is_in_fastboot());
    step("say what to expect", card.author_is_told());
    check("the author's go is recorded on the item", card.author_has_said_go());
    step("boot the matched UFS-on image from memory, nothing flashed", card.boot_matched_image());
    run_step(steps::wait_for_gadget(30, 5));
    step("the shell answers and the hold is on", card.shell_answers_and_holds());
    check("the probe lines are in the kernel log", card.the_probe_lines_are_read());
    check("the link is read: full speed, or works, slow", card.the_link_is_read());
    check("the card's disks are listed, read only", card.the_luns_are_listed());
    check("no purge was asked", card.no_purge_was_asked());
    step("release the hold: the phone restarts by itself", card.the_hold_is_released());
    step("the phone is on Android (the way back, if it is not)", card.way_back());
    check("the phone is back on Android", card.android_is_back());
}
