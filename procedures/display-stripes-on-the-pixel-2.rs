#!/usr/bin/env rust-script
//! The author's look at the Pixel's screen on L7 (item walleye-display-look-l7; decisions 2026-11-19-walleye-attended-tries-one-command,
//! 2026-11-19-walleye-stripes-text-mode-off-while-held). L7 is L6's load order, the display modules loaded by this host procedure in
//! order with pauses, and the drawer that puts the console in KD_GRAPHICS while the stripes are held. One command: the gate, the phone
//! to fastboot, L7 from memory (nothing flashed), the modules loaded, the DSI connector read connected; THEN look 1: the kernel log's
//! text on the glass; the stripes drawn and HELD (the console in KD_GRAPHICS), look 2: four stripes red, green, blue, white; the
//! drawer's end (the console back in KD_TEXT). A real run is watched: whatever ends it - an unanswered look, a failed step, Ctrl-C -
//! the way back runs after it (`reboot -f` over the shell, then the sysrq trigger, then the power-hold sentence).
//! The person steps use the prompt and answer files: ~/.cache/lpoco-attended/<device>/prompt and answer.
//!   procedures/display-stripes-on-the-pixel-2.rs [--dry-run] [--log FILE] [--answer-limit SECONDS, at most 600: with no terminal the answer file is waited for only when a limit is given]   (its values: device.toml, keys listed in tools/walleye-tries/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.44.3" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    display::{Display, LOOK_FOR_LOG, LOOK_FOR_STRIPES},
    settings::{Settings, DISPLAY_PROCEDURE},
    watch,
};

fn main() {
    let _run = procedure(DISPLAY_PROCEDURE, "google-walleye");
    let l7 = Display::new(Settings::for_display());
    // a real run is watched: whatever ends it (an unanswered look, a failed check, Ctrl-C), the way back runs after
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(l7.way_back());
    }

    check("the battery is good enough", l7.battery_is_good());
    check("the image is the one built", l7.image_is_the_built_one());
    check("UFS is off in the image", l7.ufs_is_off());
    step("take the phone to fastboot", l7.phone_is_in_fastboot());
    step("boot the image from memory", l7.boot_image());
    run_step(steps::wait_for_gadget(30, 5));
    step("the shell answers; set the hold and its deadman", l7.shell_answers_and_holds());
    step("load the display modules in the list's order", l7.modules_are_loaded());
    check("the DSI connector says connected", l7.connector_is_connected());
    step("say what to expect", l7.author_is_told());
    look_for(LOOK_FOR_LOG);
    step("say what the stripes are", l7.stripes_are_announced());
    step("start the stripe drawer, detached", l7.drawer_is_started());
    check("the drawer drew the stripes, the console in graphics mode", l7.stripes_are_drawn());
    look_for(LOOK_FOR_STRIPES);
    check("the drawer ended with 0 and the console is back in text mode", l7.drawer_has_ended());
    step("send the phone back to Android", l7.way_back());
    check("the phone is back on Android", l7.android_is_back());
}
