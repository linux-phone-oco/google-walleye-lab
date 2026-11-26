// SPDX-License-Identifier: MIT OR Apache-2.0
//! The judges against the texts captured on 2026-11-26 (research walleye-tas2557-v3-builds, folder
//! research/walleye-tas2557-series/2026-11-26), and against the same texts spoiled one way each.

use walleye_tas2557_builds::{binding, conclude, driver_build, line, overlay, series, Verdict};

const COMMITS: &str =
    include_str!("../../../research/walleye-tas2557-series/2026-11-26/commits.txt");
const BINDING: &str =
    include_str!("../../../research/walleye-tas2557-series/2026-11-26/dt_binding_check.txt");
const BUILD: &str =
    include_str!("../../../research/walleye-tas2557-series/2026-11-26/build-W1.txt");
const OVERLAY: &str =
    include_str!("../../../research/walleye-tas2557-series/2026-11-26/overlay-merge.txt");

#[test]
fn the_captured_series_is_two_fromlist_commits_and_names_its_tip() {
    assert_eq!(
        series(COMMITS).unwrap(),
        "4b5ddbd8f82116df9fbaa91570316b271b909e7a"
    );
}

#[test]
fn a_series_of_one_or_a_commit_that_is_not_fromlist_fails() {
    let one = COMMITS.lines().next().unwrap();
    assert!(series(one).unwrap_err().contains("not two"));
    let spoiled = COMMITS.replace("FROMLIST: dt-bindings", "dt-bindings");
    assert!(series(&spoiled)
        .unwrap_err()
        .contains("not a FROMLIST commit"));
}

#[test]
fn the_captured_binding_check_passes() {
    binding(BINDING).unwrap();
}

#[test]
fn a_binding_check_with_a_warning_a_bad_exit_or_no_example_fails() {
    let warned = BINDING.replace("  STYLE", "  WARNING: x\n  STYLE");
    assert!(binding(&warned).unwrap_err().contains("says"));
    assert!(binding(&BINDING.replace("make_exit=0", "make_exit=2"))
        .unwrap_err()
        .contains("exits 2"));
    assert!(
        binding(&BINDING.replace("ti,tas2557.example.dtb", "other.example.dtb"))
            .unwrap_err()
            .contains("example")
    );
    assert!(binding(&BINDING.replace("make_exit=0\n", ""))
        .unwrap_err()
        .contains("did not finish"));
}

#[test]
fn the_captured_driver_build_passes() {
    driver_build(BUILD).unwrap();
}

#[test]
fn a_driver_build_with_a_warning_a_bad_exit_or_no_object_fails() {
    let warned = BUILD.replace(
        "make[1]: Leaving",
        "tas2557.c:1: warning: x\nmake[1]: Leaving",
    );
    assert!(driver_build(&warned).unwrap_err().contains("says"));
    assert!(driver_build(&BUILD.replace("build_exit=0", "build_exit=2"))
        .unwrap_err()
        .contains("exits 2"));
    assert!(
        driver_build(&BUILD.replace("CC [M]  sound/soc/codecs/tas2557.o\n", ""))
            .unwrap_err()
            .contains("not compiled")
    );
}

#[test]
fn the_captured_overlay_read_back_passes() {
    overlay(OVERLAY).unwrap();
}

#[test]
fn an_overlay_read_back_with_a_wrong_value_fails() {
    assert!(overlay(&OVERLAY.replace("reg: 4c 4d", "reg: 4c"))
        .unwrap_err()
        .contains("reg"));
    assert!(overlay(&OVERLAY.replace("okay", "disabled"))
        .unwrap_err()
        .contains("i2c status"));
    assert!(overlay(&OVERLAY.replace(
        "reset-gpios (phandle gpio flags): 91 6 1",
        "reset-gpios (phandle gpio flags): 91 6 0"
    ))
    .unwrap_err()
    .contains("active low"));
    assert!(overlay(&OVERLAY.replace("qcom,pm8998-gpio", "qcom,other"))
        .unwrap_err()
        .contains("not a qcom,pm8998-gpio"));
    assert!(overlay(&OVERLAY.replace("phandle 91 is", "phandle 92 is"))
        .unwrap_err()
        .contains("no line says"));
}

#[test]
fn the_conclusion_is_fail_before_skip_before_pass_and_prints_the_one_line() {
    let ok = || Verdict::Pass(String::new());
    let tip = "4b5ddbd8f821";
    let all = conclude(tip, &[("a", ok()), ("b", ok())]);
    assert!(line(&all).starts_with("pass walleye-tas2557-driver-builds host "));
    let skipped = conclude(tip, &[("a", ok()), ("b", Verdict::Skip("no dtb".into()))]);
    assert!(line(&skipped).starts_with("skip walleye-tas2557-driver-builds host b: no dtb"));
    let failed = conclude(
        tip,
        &[
            ("a", Verdict::Fail("x".into())),
            ("b", Verdict::Skip("y".into())),
        ],
    );
    assert_eq!(
        line(&failed),
        "fail walleye-tas2557-driver-builds host a: x"
    );
}

#[test]
fn the_declared_command_line_lints_and_prints_its_usage() {
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_walleye-tas2557-builds"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        out.contains("--base-dtb") && out.contains("--cross-compile"),
        "{out}"
    );
}

#[test]
fn no_kernel_clone_is_a_usage_error_not_a_verdict() {
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_walleye-tas2557-builds"))
        .env_remove("WALLEYE_TAS2557_KERNEL")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
}

#[test]
fn a_file_name_that_holds_error_is_not_an_error() {
    // seen on the lab run of 2026-11-26: a cold tree builds host tools first, `fdt_strerror.o` among them
    let cold = BUILD.replace(
        "  CC [M]",
        "  HOSTCC  scripts/dtc/libfdt/fdt_strerror.o\n  CC [M]",
    );
    driver_build(&cold).unwrap();
}

#[test]
fn a_schema_complaint_without_the_words_warning_or_error_fails_the_binding() {
    // the kernel's Makefile ends its checks with `|| true`: the exit stays 0, only the text tells
    let one = BINDING.replace(
        "  STYLE",
        "/src/x/ti,tas2557.example.dtb: audio-codec@4c: 'x' is a required property\n  STYLE",
    );
    assert!(binding(&one).unwrap_err().contains("required property"));
    let two = BINDING.replace(
        "  STYLE",
        "\tfrom schema $id: http://devicetree.org/schemas/sound/ti,tas2557.yaml\n  STYLE",
    );
    assert!(binding(&two).unwrap_err().contains("from schema"));
}
