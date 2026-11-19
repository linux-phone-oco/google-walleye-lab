#!/usr/bin/env rust-script
//! The display try on L6 (item walleye-attend-program-display; decision 2026-11-19-walleye-attended-tries-one-command).
//! One command: the gate, the phone to fastboot, L6 from memory, the display modules loaded, the panel read, the stripe
//! drawer started detached and polled until it has drawn; THEN the author is asked to look at the glass; then the
//! drawer's end and the way back to Android. Each step stops the run when it fails; with nobody to answer it stops at look_for.
//!   procedures/display-stripes-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.32.1" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    display::{Display, LOOK_FOR},
    settings::{Settings, DISPLAY_PROCEDURE},
};

fn main() {
    let _run = procedure(DISPLAY_PROCEDURE, "google-walleye");
    let l6 = Display::new(Settings::for_display());

    check("the battery is good enough", l6.battery_is_good());
    check("the image is the one built", l6.image_is_the_built_one());
    check("UFS is off in the image", l6.ufs_is_off());
    step("take the phone to fastboot", l6.phone_is_in_fastboot());
    step("boot the image from memory", l6.boot_image());
    run_step(steps::wait_for_gadget(30, 5));
    step("the shell answers; set the hold and its deadman", l6.shell_answers_and_holds());
    step("load the display modules in the list's order", l6.modules_are_loaded());
    check("the DSI connector says connected", l6.connector_is_connected());
    step("say what to expect", l6.author_is_told());
    step("start the stripe drawer, detached", l6.drawer_is_started());
    check("the drawer drew the stripes", l6.stripes_are_drawn());
    look_for(LOOK_FOR);
    check("the drawer ended with 0", l6.drawer_has_ended());
    check("the phone is back on Android", l6.android_is_back());
}
