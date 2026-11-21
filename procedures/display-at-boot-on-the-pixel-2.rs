#!/usr/bin/env rust-script
//! The author's look at the Pixel's screen as one command: image B2 (items walleye-display-look-b2 and walleye-display-at-boot;
//! decision 2026-11-19-walleye-attended-tries-one-command). The gate, the phone to fastboot, B2 from memory (nothing flashed),
//! the shell answering with no host script having loaded a module, every one of the 13 display modules said `walleye-display: ok`
//! by the image's own init, card0 and its DSI-1 connector read connected, the display lines recorded, the backlight written once
//! through sysfs and the kernel log read again; THEN look 1: the kernel log's text on the glass; the stripes drawn and HELD
//! (the console in KD_GRAPHICS), look 2: four stripes red, green, blue, white; the way back to Android. The init holds its own
//! self-reboot for 600 s (its deadman). Each step stops the run when it fails; with nobody to answer it stops at the first look_for.
//! The person steps use the prompt and answer files: ~/.cache/lpoco-attended/<device>/prompt and answer (lpoco-attended 0.34+).
//!   procedures/display-at-boot-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.41.0" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    display_boot::{DisplayBoot, LOOK_FOR, LOOK_FOR_STRIPES},
    settings::{Settings, DISPLAY_BOOT_PROCEDURE},
};

fn main() {
    let _run = procedure(DISPLAY_BOOT_PROCEDURE, "google-walleye");
    let b2 = DisplayBoot::new(Settings::for_display_boot());

    check(
        "the image is the one built (sha256)",
        b2.image_is_the_built_one(),
    );
    check("UFS is off in the image", b2.ufs_is_off());
    check("the battery is good enough", b2.battery_is_good());
    step("take the phone to fastboot", b2.phone_is_in_fastboot());
    step(
        "boot the image from memory, nothing flashed",
        b2.boot_image(),
    );
    run_step(steps::wait_for_gadget(30, 5));
    step(
        "the shell answers by itself and the init's hold is on",
        b2.shell_answers_and_the_hold_is_on(),
    );
    check(
        "every display module said ok in the kernel log",
        b2.every_module_said_ok(),
    );
    check(
        "card0 and the DSI-1 connector read connected",
        b2.connector_is_connected(),
    );
    step(
        "record the kernel log's display lines",
        b2.display_lines_are_recorded(),
    );
    step(
        "write the backlight once through sysfs and read the log again",
        b2.backlight_is_written(),
    );
    step("say what to expect", b2.author_is_told());
    look_for(LOOK_FOR);
    step("say what the stripes are", b2.stripes_are_announced());
    step(
        "draw the stripes held, the console in graphics mode",
        b2.stripes_are_drawn_held(),
    );
    look_for(LOOK_FOR_STRIPES);
    check(
        "the drawer ended with 0 and the console is back in text mode",
        b2.stripes_have_ended(),
    );
    step("send the phone back to Android", b2.way_back());
    check("the phone is back on Android", b2.android_is_back());
}
