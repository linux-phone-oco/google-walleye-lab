#!/usr/bin/env rust-script
//! The touch try on z21 (item walleye-attend-program-touch; decision 2026-11-19-walleye-attended-tries-one-command).
//! One command: the gate, the boot, the reader open and showing EVENTS-READY, THEN the author is asked to swipe
//! across the whole glass and tap each corner; then the way back and the count. Each step stops the run when it fails.
//!   procedures/touch-events-on-z21.rs [--dry-run] [--log FILE]      (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::prelude::*;
use walleye_tries::{settings::{Settings, TOUCH_PROCEDURE}, touch::Touch};

fn main() {
    let _run = procedure(TOUCH_PROCEDURE, "google-walleye");
    let z21 = Touch::new(Settings::for_touch());

    check("the battery is good enough", z21.battery_is_good());
    check("the image is the one built", z21.image_is_the_built_one());
    check("UFS is off in the image", z21.ufs_is_off());
    step("take the phone to fastboot", z21.phone_is_in_fastboot());
    step("boot the image from memory", z21.boot_image());
    check("the phone's shell answers and its dmesg is saved", z21.shell_answers());
    check("the touch controller is listed", z21.touch_device_is_listed());
    check("the reader's window is long enough", z21.window_is_long_enough());
    step("start the touch reader", z21.reader_is_started());
    check("the reader shows EVENTS-READY", z21.reader_shows_events_ready());
    ask_author("swipe across the whole glass and tap each corner, until the reader window closes / the terminal counted your touches (say ok when done)");
    step("wait for the reader's window to close", z21.window_has_closed());
    check("the phone is back on Android", z21.android_is_back());
    check("a touch made events", z21.events_are_counted());
}
