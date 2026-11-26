// SPDX-License-Identifier: MIT OR Apache-2.0
//! The built binary's surface, on the host: --dry-run, refusals, and the real kernel's answer for a device this host lacks.

use std::process::Command;

fn run(args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_walleye-nfc-up"))
        .args(args)
        .env_remove("WALLEYE_NFC_UP_INDEX")
        .output()
        .unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into(),
        String::from_utf8_lossy(&o.stderr).into(),
    )
}

#[test]
fn the_dry_run_prints_both_requests_and_opens_no_socket() {
    let (rc, out, _) = run(&["--index", "1", "--dry-run"]);
    assert_eq!(rc, 0);
    assert!(out.contains("NFC_CMD_DEV_UP for nfc1"), "{out}");
    assert_eq!(out.lines().count(), 2, "{out}");
}

#[test]
fn an_unknown_flag_stops_the_run_before_anything_is_sent() {
    let (rc, _, err) = run(&["--dryrun"]);
    assert_ne!(rc, 0);
    assert!(err.contains("--dryrun"), "{err}");
}

#[test]
fn an_open_this_host_cannot_make_prints_the_kernels_answer_and_ends_1() {
    // either the host has no NFC family (stderr says so) or it has and answers an errno for a device that is not there (stdout, as it came); both end 1, so the procedure's rc= reads it
    let (rc, out, err) = run(&["--index", "4000000"]);
    assert_eq!(rc, 1, "{out}{err}");
    assert!(
        err.contains("no family \"nfc\"") || out.contains("NFC_CMD_DEV_UP answered: errno"),
        "{out}{err}"
    );
}

#[test]
fn the_dry_run_with_a_poll_adds_the_poll_step_and_without_one_does_not() {
    let (rc, out, _) = run(&["--dry-run", "--poll", "10"]);
    assert_eq!(rc, 0);
    assert_eq!(out.lines().count(), 3, "{out}");
    assert!(
        out.contains("wait 10 s for NFC_EVENT_TARGETS_FOUND"),
        "{out}"
    );
}

#[test]
fn a_poll_that_is_not_a_number_is_refused() {
    let (rc, _, err) = run(&["--poll", "soon"]);
    assert_ne!(rc, 0);
    assert!(err.contains("poll"), "{err}");
}

#[test]
fn a_poll_longer_than_an_hour_is_refused_before_anything_is_sent() {
    let (rc, _, err) = run(&["--poll", "99999999999999"]);
    assert_eq!(rc, 2);
    assert!(err.contains("3600"), "{err}");
}
