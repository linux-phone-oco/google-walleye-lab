// SPDX-License-Identifier: MIT OR Apache-2.0
//! The check against this lab's own device.toml (the real recipes) and against copies of it made longer or
//! shorter here. No length is typed: the tool prints the lengths it measures from the manifest's own lines.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(device: &str, exceptions: &str) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_walleye-cmdline-budget"))
        .args(["--device", device, "--exceptions", exceptions])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&o.stdout).to_string() + &String::from_utf8_lossy(&o.stderr);
    (o.status.code().unwrap_or(-1), text)
}

fn real_exceptions() -> String {
    lab()
        .join("tests/host/cmdline-over-budget.toml")
        .to_str()
        .unwrap()
        .to_string()
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("wcb-{}-{name}", std::process::id()));
    fs::create_dir_all(&d).unwrap();
    d
}

/// The lab's device.toml with the production image's recipe line lengthened by `extra` characters.
fn device_with_longer_production(extra: usize) -> String {
    let real = fs::read_to_string(lab().join("device.toml")).unwrap();
    let mut out = String::new();
    let mut inside = false;
    for line in real.lines() {
        if line.starts_with('[') {
            inside = line == "[images.ufs-root-booted-slot-production.recipe]";
        }
        if inside && line.starts_with("cmdline = \"") {
            let cut = line.rfind('"').unwrap();
            out.push_str(&format!("{}{}\"\n", &line[..cut], " x".repeat(extra / 2)));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[test]
fn the_lab_manifest_passes_today() {
    let (code, out) = run(
        lab().join("device.toml").to_str().unwrap(),
        &real_exceptions(),
    );
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("ufs-root-booted-slot-production:"), "{out}");
}

#[test]
fn a_recipe_over_the_budget_fails_naming_it() {
    let d = scratch("over");
    let f = d.join("device.toml");
    fs::write(&f, device_with_longer_production(40)).unwrap();
    let (code, out) = run(f.to_str().unwrap(), &real_exceptions());
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("fail walleye-cmdline-budget ufs-root-booted-slot-production:"),
        "{out}"
    );
}

#[test]
fn an_exception_for_an_image_no_longer_over_is_refused() {
    let d = scratch("stale");
    let f = d.join("exceptions.toml");
    let mut t = fs::read_to_string(real_exceptions()).unwrap();
    t.push_str("\n[over.ufs-root-booted-slot-production]\nwhy = \"stale\"\n");
    fs::write(&f, t).unwrap();
    let (code, out) = run(
        lab().join("device.toml").to_str().unwrap(),
        f.to_str().unwrap(),
    );
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("no longer over"), "{out}");
}

#[test]
fn an_exception_for_an_image_that_is_gone_is_refused() {
    let d = scratch("gone");
    let f = d.join("exceptions.toml");
    fs::write(&f, "[over.no-such-image]\nwhy = \"x\"\n").unwrap();
    let (code, out) = run(
        lab().join("device.toml").to_str().unwrap(),
        f.to_str().unwrap(),
    );
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("no-such-image"), "{out}");
}

#[test]
fn an_exception_without_a_reason_is_refused() {
    let d = scratch("nowhy");
    let f = d.join("exceptions.toml");
    fs::write(&f, "[over.ufs-root]\n").unwrap();
    let (code, out) = run(
        lab().join("device.toml").to_str().unwrap(),
        f.to_str().unwrap(),
    );
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("the exception has no why"), "{out}");
}

#[test]
fn the_long_recipes_without_their_exceptions_fail() {
    let d = scratch("none");
    let f = d.join("exceptions.toml");
    fs::write(&f, "").unwrap();
    let (code, out) = run(
        lab().join("device.toml").to_str().unwrap(),
        f.to_str().unwrap(),
    );
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("fail walleye-cmdline-budget ufs-root:"),
        "{out}"
    );
    assert!(
        out.contains("fail walleye-cmdline-budget ufs-root-booted-slot:"),
        "{out}"
    );
    assert!(
        !out.contains("fail walleye-cmdline-budget ufs-root-booted-slot-debug:"),
        "{out}"
    );
}

/// The lab's device.toml with a bootable image's recipe line (200 bytes, over the budget) appended.
fn device_with_bootable_over() -> String {
    let real = fs::read_to_string(lab().join("device.toml")).unwrap();
    format!(
        "{real}\n[images.fake-bootable]\npath = \"images/boot.img\"\n[images.fake-bootable.recipe]\ncmdline = \"{}\"\n",
        "a".repeat(200)
    )
}

#[test]
fn an_exception_for_a_bootable_image_is_refused() {
    let d = scratch("bootable");
    let dev = d.join("device.toml");
    fs::write(&dev, device_with_bootable_over()).unwrap();
    let f = d.join("exceptions.toml");
    let mut t = fs::read_to_string(real_exceptions()).unwrap();
    t.push_str("\n[over.fake-bootable]\nwhy = \"x\"\n");
    fs::write(&f, t).unwrap();
    let (code, out) = run(dev.to_str().unwrap(), f.to_str().unwrap());
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("can be booted"), "{out}");
}

#[test]
fn a_cmdline_that_is_not_a_string_fails() {
    let d = scratch("array");
    let dev = d.join("device.toml");
    let real = fs::read_to_string(lab().join("device.toml")).unwrap();
    fs::write(&dev, format!("{real}\n[images.fake-array]\npath = \"images/boot.img\"\n[images.fake-array.recipe]\ncmdline = [\"a\"]\n")).unwrap();
    let (code, out) = run(dev.to_str().unwrap(), &real_exceptions());
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("fake-array: the recipe's cmdline is not a string"),
        "{out}"
    );
}

#[test]
fn a_max_below_one_is_refused() {
    let o = Command::new(env!("CARGO_BIN_EXE_walleye-cmdline-budget"))
        .args(["--max", "-5"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
}
