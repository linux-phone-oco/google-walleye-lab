// SPDX-License-Identifier: MIT OR Apache-2.0
//! The built binary's surface, on the host: only --dry-run and refusals run here, never a real reboot.

use std::process::Command;

fn run(args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_walleye-reboot"))
        .args(args)
        .env_remove("WALLEYE_REBOOT_COMMAND")
        .output()
        .unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into(),
        String::from_utf8_lossy(&o.stderr).into(),
    )
}

#[test]
fn the_dry_run_says_the_string_it_would_pass() {
    let (rc, out, _) = run(&["--command", "bootloader", "--dry-run"]);
    assert_eq!(rc, 0);
    assert!(out.contains("RESTART2, \"bootloader\""), "{out}");
}

#[test]
fn no_command_is_refused_without_calling_the_kernel() {
    let (rc, _, err) = run(&[]);
    assert_eq!(rc, 2);
    assert!(err.contains("--command STRING is required"), "{err}");
    let (rc, _, err) = run(&["--command", "", "--dry-run"]);
    assert_eq!(rc, 1);
    assert!(err.contains("not allowed"), "{err}");
}

#[test]
fn a_string_off_the_allow_list_is_refused_even_for_real() {
    // refused before the kernel is asked, so this is safe to run on the host
    let (rc, _, err) = run(&["--command", "recovery"]);
    assert_eq!(rc, 1);
    assert!(err.contains("not allowed"), "{err}");
}

#[test]
fn an_unknown_flag_stops_the_run_before_the_kernel_is_asked() {
    // the typo of --dry-run must never fall through to a real reboot
    let (rc, _, err) = run(&["--command", "bootloader", "--dryrun"]);
    assert_ne!(rc, 0);
    assert!(err.contains("--dryrun"), "{err}");
}
