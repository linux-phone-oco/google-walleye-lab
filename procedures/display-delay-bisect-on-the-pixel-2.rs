#!/usr/bin/env rust-script
//! The display delay bisect as one command (item walleye-display-delay-bisect-build; research walleye-display-b2-dsi-clock-warning,
//! decision 2026-11-19-walleye-display-early-load-splash-handover). B2's kernel, dtb and modules fail when msm loads at about 2 s of uptime and
//! are clean at 66 s: where between them do the signs stop? Five images B-d5, B-d15, B-d25, B-d35, B-d50 are B2 whose init waits the
//! number of seconds in its own /display-delay before it loads the display modules (rebuilt by `lpoco bringup image ensure`; its script is retired), so the load time
//! does not depend on the shell. For each, in a row: the phone to fastboot, the image from memory (nothing flashed; steps::phone_ready),
//! the shell answering with the image's own delay, the kernel log read once the init says it is through the list and the three signs counted (SMMU context faults,
//! `rcg didn't update its configuration`, DSI -110), the way back to Android (steps::leave_known_good). The result table (delay, faults, warnings, -110) is
//! kept in ~/lab-material/walleye/display-delay-PREFIX-DAY/table.txt (PREFIX: b-d, or the image_prefix) and said in the log; it is written before the
//! last way back, which ends the run. NO person step: nobody is asked, the glass is not looked at.
//! A failed step or a panic ends with the way back to Android (the phone_ready group's own); a signal (Ctrl-C, SIGTERM, SIGHUP) runs the owed way backs too (lpoco-attended 0.46.3; exit 130/143/129).
//!   procedures/display-delay-bisect-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml, keys listed in procedures/steps/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    display_delay::DisplayDelay,
    settings::{Settings, DISPLAY_DELAY_PROCEDURE},
};

fn main() {
    let _run = procedure(DISPLAY_DELAY_PROCEDURE, "google-walleye");
    let bisect = DisplayDelay::new(Settings::for_display_delay());

    check_from("the battery is good enough", "research:walleye-display-delay-bisect-flips-at-25s", bisect.battery_is_good());
    check_from("every image is the one built (sha256)", "research:walleye-display-delay-bisect-flips-at-25s",
        bisect.every_image_is_the_built_one(),
    );
    check_from("UFS is off in every image", "research:walleye-display-delay-bisect-flips-at-25s", bisect.ufs_is_off_in_every_image());
    let delays = bisect.delays();
    for (at, &delay) in delays.iter().enumerate() {
        let kernel = bisect.kernel(delay).expect("device.toml names an image for each delay");
        let ready = steps::phone_ready(&kernel); // built first: a kernel [kernels] test does not list ends the run before the phone is touched
        step(
            &format!("{kernel}: take the phone to fastboot"),
            bisect.phone_is_in_fastboot(),
        );
        run_group(ready); // boots [images.NAME] from memory, nothing flashed, its gadget on the bus
        step(
            &format!("{kernel}: the shell answers and the image has its delay"),
            bisect.shell_answers_and_the_image_has_its_delay(delay),
        );
        check_from(
            &format!("{kernel}: the modules load at {delay} s and the three signs are counted"),
            "research:walleye-display-delay-bisect-flips-at-25s",
            bisect.modules_load_and_the_signs_are_scored(delay),
        );
        if at + 1 == delays.len() {
            // nothing may follow the last leave_known_good: the table goes before it
            step(
                "write the result table: delay, faults, warnings, -110",
                bisect.the_table_is_written(),
            );
        }
        run_group(steps::leave_known_good()); // the declared way back (device.toml [kernels]), then Android answers
    }
}
