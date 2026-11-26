#!/usr/bin/env rust-script
//! The touch try on z21 (item walleye-attend-program-touch; decision 2026-11-19-walleye-attended-tries-one-command).
//! One command: the gate, the boot, the reader open and showing EVENTS-READY, THEN the author is asked to swipe
//! across the whole glass and tap each corner; then the count and the way back (steps::phone_ready(z21), steps::leave_known_good last, lpoco-attended 0.46.3). Each step stops the run when it fails.
//!   procedures/touch-events-on-z21.rs [--dry-run] [--log FILE]      (its values: device.toml, keys listed in procedures/steps/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{settings::{Settings, TOUCH_PROCEDURE}, touch::{Touch, KERNEL}};

fn main() {
    let _run = procedure(TOUCH_PROCEDURE, "google-walleye");
    let z21 = Touch::new(Settings::for_touch());

    check_from("the battery is good enough", "research:walleye-z21-built", z21.battery_is_good());
    check_from("the image is the one built", "research:walleye-z21-built", z21.image_is_the_built_one());
    check_from("UFS is off in the image", "research:walleye-z21-built", z21.ufs_is_off());
    step("take the phone to fastboot", z21.phone_is_in_fastboot());
    run_group(steps::phone_ready(KERNEL)); // boots [images.z21] from memory, nothing flashed, its gadget on the bus, the shell answering
    check_from("the phone's dmesg is saved", "research:walleye-z21-built", z21.dmesg_is_saved());
    check_from("the touch controller is listed", "research:walleye-z21-touch-events-pass", z21.touch_device_is_listed());
    check_from("the reader's window is long enough", "research:walleye-z21-touch-events-window-missed", z21.window_is_long_enough());
    step("start the touch reader", z21.reader_is_started());
    check_from("the reader shows EVENTS-READY", "research:walleye-z21-touch-events-pass", z21.reader_shows_events_ready());
    ask_author("swipe across the whole glass and tap each corner, until the reader window closes / the terminal counted your touches (say ok when done)");
    step("wait for the reader's window to close", z21.window_has_closed());
    check_from("a touch made events", "research:walleye-z21-touch-events-pass", z21.events_are_counted()); // reads the saved files: before the way back
    run_group(steps::leave_known_good()); // LAST: the declared way back (device.toml [kernels]), then Android answers
}
