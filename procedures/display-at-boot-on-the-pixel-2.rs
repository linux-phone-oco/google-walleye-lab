#!/usr/bin/env rust-script
//! The display-at-boot try on B1 (item walleye-display-at-boot; decision 2026-11-19-walleye-attended-tries-one-command).
//! One command: the gate, the phone to fastboot, B1 from memory (nothing flashed), the shell answering with no host
//! script having loaded a module, every one of the 13 display modules said `walleye-display: ok` by the image's own init in
//! the kernel log, card0 and its DSI-1 connector read connected, the display lines of the kernel log recorded; THEN the
//! author is asked to look at the glass; then the way back to Android. The init holds its own self-reboot for 600 s (its
//! deadman: the phone restarts by itself after that). Each step stops the run when it fails; with nobody to answer it stops at look_for.
//!   procedures/display-at-boot-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.32.1" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    display_boot::{DisplayBoot, LOOK_FOR},
    settings::{Settings, DISPLAY_BOOT_PROCEDURE},
};

fn main() {
    let _run = procedure(DISPLAY_BOOT_PROCEDURE, "google-walleye");
    let b1 = DisplayBoot::new(Settings::for_display_boot());

    check(
        "the image is the one built (sha256)",
        b1.image_is_the_built_one(),
    );
    check("UFS is off in the image", b1.ufs_is_off());
    check("the battery is good enough", b1.battery_is_good());
    step("take the phone to fastboot", b1.phone_is_in_fastboot());
    step(
        "boot the image from memory, nothing flashed",
        b1.boot_image(),
    );
    run_step(steps::wait_for_gadget(30, 5));
    step(
        "the shell answers by itself and the init's hold is on",
        b1.shell_answers_and_the_hold_is_on(),
    );
    check(
        "every display module said ok in the kernel log",
        b1.every_module_said_ok(),
    );
    check(
        "card0 and the DSI-1 connector read connected",
        b1.connector_is_connected(),
    );
    step(
        "record the kernel log's display lines",
        b1.display_lines_are_recorded(),
    );
    step("say what to expect", b1.author_is_told());
    look_for(LOOK_FOR);
    step("send the phone back to Android", b1.way_back());
    check("the phone is back on Android", b1.android_is_back());
}
