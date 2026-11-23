#!/usr/bin/env rust-script
//! The author's look at Alpine's login prompt on the Pixel's glass as ONE command (item walleye-display-alpine-ramroot-image; the author's word of
//! 2026-11-22, relayed by the Top Coordinator; decision 2026-11-19-walleye-attended-tries-one-command): image B-a5 (B-r5's kernel, dtb and 13 display
//! modules, the Alpine userland unpacked into a tmpfs root, UFS off) booted once from fastboot, from memory, nothing flashed. The lab reads it over
//! usb-network by ssh with its key: the display modules loaded at 5 s with none of the three signs (SMMU faults, rcg warning, DSI -110), 13 of 13 said ok,
//! a getty on tty1 (the glass: VT 1 under fbcon) and on ttyGS0, sshd up, fbcon in the kernel log, the root a tmpfs, no purge or oops, and the SCREEN'S TEXT (/dev/vcs of the VT the glass shows) ENDING in `walleye login:` (the last non-blank row, the cursor's: init respawns the tty1 getty at the end of the default runlevel) - measured before the author is asked; THEN the look: the deadman held,
//! the author asked whether he sees the Alpine login prompt, the hold released; the phone sent back and Android checked. Each step stops the run when it fails;
//! with nobody to answer (the keeper's unattended run) it stops at the look, and the watching parent still runs the way back to Android.
//! The person step uses the prompt and answer files: ~/.cache/lpoco-attended/<device>/prompt and answer.
//!   procedures/alpine-login-on-the-glass-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.alpine-login-on-the-glass-on-the-pixel-2])
//!
//! ```cargo
//! [dependencies]
//! lpoco-attended = { git = "https://github.com/linux-phone-oco/lab-tools", tag = "lpoco-attended-v0.44.3" }
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use lpoco_attended::{prelude::*, steps};
use walleye_tries::{
    ramroot::{Ramroot, LOOK_FOR},
    settings::{Settings, RAMROOT_PROCEDURE},
    watch,
};

fn main() {
    let _run = procedure(RAMROOT_PROCEDURE, "google-walleye");
    let glass = Ramroot::new(Settings::for_ramroot()); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(glass.way_back()); // whatever ends a real run, the way back runs after
    }

    check("the battery is good enough", glass.battery_is_good());
    check(
        "the image is the declared one (sha256)",
        glass.the_image_is_the_declared_one(),
    );
    check("UFS is off in the image", glass.ufs_is_off());
    check("the ssh key is private", glass.the_key_is_private());
    step("say what to expect", glass.author_is_told());
    step("take the phone to fastboot", glass.phone_is_in_fastboot());
    step(
        "boot the image from memory, nothing flashed",
        glass.boot_the_image(),
    );
    run_step(steps::wait_for_gadget(30, 5));
    step(
        "log in over usb-network by ssh and read the phone",
        glass.the_login_works_and_the_phone_is_read(),
    );
    check(
        "it is root's login, usb0 has the address and no service crashed",
        glass.the_phone_is_the_image_we_booted(),
    );
    check(
        "the init spoke and the root is a tmpfs",
        glass.the_root_is_a_tmpfs_the_init_made(),
    );
    check(
        "the display modules loaded at the delay, 13 of 13 ok, the three signs zero",
        glass.the_modules_loaded_and_the_three_signs_are_zero(),
    );
    check(
        "a getty runs on tty1 (the glass) and on ttyGS0, and sshd runs",
        glass.the_getties_and_sshd_run(),
    );
    check(
        "the kernel log has fbcon",
        glass.the_kernel_log_has_fbcon(),
    );
    check(
        "the screen's text (/dev/vcs of the VT the glass shows) ends in `HOSTNAME login:` on the last non-blank row, the cursor's",
        glass.the_screen_holds_the_login_prompt(),
    );
    check(
        "the kernel log has no purge, oops or panic",
        glass.the_kernel_log_is_clean(),
    );
    step("hold the deadman for the look", glass.the_deadman_is_held());
    look_for(LOOK_FOR);
    step(
        "release the deadman after the look",
        glass.the_deadman_is_released(),
    );
    step(
        "send the phone back: the forced reboot",
        glass.the_phone_is_sent_back(),
    );
    step(
        "the phone is on Android (the way back, if it is not)",
        glass.way_back(),
    );
    check("the phone is back on Android", glass.android_is_back());
}
