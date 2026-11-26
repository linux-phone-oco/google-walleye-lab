#!/usr/bin/env rust-script
//! The author's look at the Pixel's screen on L7 (item walleye-display-look-l7; decisions 2026-11-19-walleye-attended-tries-one-command,
//! 2026-11-19-walleye-stripes-text-mode-off-while-held). L7 is L6's load order, the display modules loaded by this host procedure in
//! order with pauses, and the drawer that puts the console in KD_GRAPHICS while the stripes are held. One command: the gate, the phone
//! to fastboot, L7 from memory (nothing flashed), the modules loaded, the DSI connector read connected; THEN look 1: the kernel log's
//! text on the glass; the stripes drawn and HELD (the console in KD_GRAPHICS), look 2: four stripes red, green, blue, white; the
//! drawer's end (the console back in KD_TEXT). A real run is watched: whatever ends it - an unanswered look, a failed step, Ctrl-C -
//! the way back runs after it: the step groups of lpoco-attended 0.46.3, phone_ready(l7) and leave_known_good (device.toml [kernels] way_back: procedures/way-back-on-the-pixel-2.rs), and the watching parent as the safety for Ctrl-C.
//! The person steps use the prompt and answer files: ~/.cache/lpoco-attended/<device>/prompt and answer.
//!   procedures/display-stripes-on-the-pixel-2.rs [--dry-run] [--log FILE] [--answer-limit SECONDS, at most 600: with no terminal the answer file is waited for only when a limit is given]   (its values: device.toml, keys listed in procedures/steps/src/settings.rs)
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "steps" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    display::{Display, KERNEL, LOOK_FOR_LOG, LOOK_FOR_STRIPES},
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

    check_from("the battery is good enough", "research:walleye-display-l7-built", l7.battery_is_good());
    check_from("the image is the one built", "research:walleye-display-l7-built", l7.image_is_the_built_one());
    check_from("UFS is off in the image", "research:walleye-display-l7-built", l7.ufs_is_off());
    step("take the phone to fastboot", l7.phone_is_in_fastboot());
    run_group(steps::phone_ready(KERNEL)); // boots [images.l7] from memory, its gadget on the bus, the shell answering
    step("set the hold and its deadman", l7.shell_answers_and_holds());
    step("load the display modules in the list's order", l7.modules_are_loaded());
    check_from("the DSI connector says connected", "research:walleye-display-late-probe-l5-tried-dsi-connected-1080x1920", l7.connector_is_connected());
    step("say what to expect", l7.author_is_told());
    look_for(LOOK_FOR_LOG);
    step("say what the stripes are", l7.stripes_are_announced());
    step("start the stripe drawer, detached", l7.drawer_is_started());
    check_from("the drawer drew the stripes, the console in graphics mode", "research:walleye-display-stripes-l6-tried-draw-calls-ok", l7.stripes_are_drawn());
    look_for(LOOK_FOR_STRIPES);
    check_from("the drawer ended with 0 and the console is back in text mode", "research:walleye-display-stripes-l6-tried-2-drawer-held-60s", l7.drawer_has_ended());
    run_group(steps::leave_known_good()); // LAST: the declared way back (device.toml [kernels]), then Android answers
}
