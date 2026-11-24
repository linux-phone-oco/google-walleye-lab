#!/usr/bin/env rust-script
//! UFS try 2 as ONE command (item walleye-ufs-try-2-one-command; decisions 2026-11-19-walleye-attended-tries-one-command,
//! 2026-11-19-walleye-ufs-hold-second-path, 2026-11-19-walleye-ufs-android-derived-solution): the matched UFS-on image booted
//! once from fastboot, from memory; the card read as in try 1; then the devfreq clock pinned low first (userspace governor, set_freq 50 MHz read back, gear HS_GEAR1; evidence kept at each point); the link gear and the clock before, during (a ramped, bounded
//! read-only O_DIRECT read of one disk) and 2 s after it; the disk's GPT hashed before and after; the slots' flags before and after.
//! The boot runs only while the author's go is recorded on the item the manifest names (no go: the run stops there and the phone
//! goes back to Android). NOTHING is written to the card. The UFS-off gate refuses this image by design and is not run, weakened or bypassed.
//! The pin is a choice (item walleye-ufs-unpinned-ramp-variant), declared once with argenv in tools/walleye-tries (ufs/choice.rs) as the environment
//! variable WALLEYE_UFS_PIN = pinned (the default, as runs 1 to 9) or unpinned: unpinned writes no governor and no set_freq (step 16 only reads and keeps
//! the node's governor, clocks and gear as found), reads them at every ramp step, asserts nothing about the gear and records what it saw
//! (gear-seen-unpinned.txt), and adds a 2048 MiB step after the settings' ramp only when its limit ends before the image's reboot.
//!   [WALLEYE_UFS_PIN=unpinned] procedures/ufs-matched-image-on-the-pixel-2.rs [--dry-run] [--log FILE]   (its values: device.toml [procedures.ufs-matched-image-on-the-pixel-2])
//!
//! ```cargo
//! [dependencies]
//! walleye-tries = { path = "../tools/walleye-tries" }
//! ```
// SPDX-License-Identifier: MIT OR Apache-2.0

use walleye_tries::lpoco_attended::{prelude::*, steps};
use walleye_tries::{settings::{Settings, UFS_PROCEDURE}, ufs::{choice::Pin, Ufs}, watch};

/// The clock step, by the choice: pinned as runs 1 to 9, or read as found and nothing written.
const PINNED: &str = "the UFS host's devfreq clock is pinned low (userspace governor, set_freq 50 MHz read back, gear HS_GEAR1, cur_freq 50 or 100 MHz as core_clk's rounding: research walleye-ufs-pin-100mhz-read) and its evidence kept, before any load";
const UNPINNED: &str = "the UFS host's devfreq clock is NOT pinned (WALLEYE_UFS_PIN=unpinned): its governor, clocks and gear are read as found and kept, nothing written, before any load";

fn main() {
    let _run = procedure(UFS_PROCEDURE, "google-walleye");
    let pin = Pin::from_process_env().unwrap_or_else(|e| {
        eprintln!("{UFS_PROCEDURE}: {e}");
        std::process::exit(1) // a wrong argument, as argenv exits for every lpoco program
    });
    let card = Ufs::new(Settings::for_ufs()).with_pin(pin); // every key from device.toml, read here
    let args: Vec<String> = std::env::args().skip(1).collect();
    if watch::watches(&args) {
        watch::watch_the_try(card.way_back()); // whatever ends a real run, the way back runs after
    }

    check_from("the battery is good enough", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.battery_is_good());
    check_from("the image is the matched one, its tests pass", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.image_is_the_matched_one());
    step("take the phone to fastboot", card.phone_is_in_fastboot());
    check_from("the slots' flags are read", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.the_slot_flags_are_read());
    step("say what to expect", card.author_is_told());
    check_from("the author's go is recorded on the item", "research:walleye-ufs-go-note", card.author_has_said_go());
    step("boot the matched UFS-on image from memory, nothing flashed", card.boot_matched_image());
    run_step(steps::wait_for_gadget(30, 5));
    step("the shell answers and the image's reboot deadline is read (its init has no hold: it reboots 180 s after its petter starts, research walleye-ufs-180s-is-the-init-self-reboot)", card.shell_answers_and_the_deadline_is_read());
    check_from("the probe lines are in the kernel log", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.the_probe_lines_are_read());
    check_from("the link is read once (up at any gear works; no wait for HS-G3 at idle)", "research:walleye-stop-ufs-try-2-step-16-three-answers", card.the_link_is_read());
    check_from("the card's disks are listed, read only", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.the_luns_are_listed());
    check_from("the card's six disks and the partition tables are read", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.the_card_is_identified());
    check_from("the GPT of the disk is hashed", "research:walleye-ufs-try-2-run-1-gpt-equal-result", card.the_gpt_is_hashed_before());
    check_from("the gear and the clock, before any load", "research:walleye-stop-ufs-try-2-step-16-three-answers", card.the_gear_is_read_before_load());
    if pin.is_pinned() {
        step(PINNED, card.the_clock_is_pinned_low());
    } else {
        step(UNPINNED, card.the_clock_is_read_as_found());
    }
    step("a ramped, bounded, read-only read of the disk (unpinned: plus a 2048 MiB step only when its limit ends before the reboot; each step only when its limit ends before the image's reboot deadline less 10 s, its start uptime kept), a RAM log line before each step, the shell polled every second, the gear sampled", card.the_gear_is_read_under_load());
    check_from("the gear and the clock, 2 s after the read", "research:walleye-stop-ufs-try-2-step-16-three-answers", card.the_gear_is_read_two_seconds_after());
    check_from("the GPT is hash-equal to before", "research:walleye-ufs-try-2-run-1-gpt-equal-result", card.the_gpt_is_unchanged());
    check_from("no purge was asked", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.no_purge_was_asked());
    step("the phone is on Android (the way back, if it is not)", card.way_back());
    check_from("the phone is back on Android", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.android_is_back());
    check_from("the PON reason is read on Android, read only", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.the_pon_reason_is_read());
    check_from("both slots' flags are as before", "research:walleye-ufs-match-tried-version-ok-link-stays-gear1", card.the_slot_flags_are_as_before());
}
