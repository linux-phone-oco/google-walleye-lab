// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-tas2557-builds: the proof walleye-tas2557-driver-builds, the entry reads as the procedure.

mod cli;

use argenv_resolve::InputResolveExt;
use cli::Contract as C;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use walleye_tas2557_builds::steps::{self, Plan};
use walleye_tas2557_builds::{binding, conclude, driver_build, line, overlay, series, Verdict};

const PINNED_IMAGE: &str =
    "alpine@sha256:020dfcbaaf4cc1078bf2d9c7ba31a8466e334061dcd2f248001d68f79e52c000";

fn main() -> ExitCode {
    let r = C::parse_and_lint(argenv_resolve::OnProblems::FailOnError);
    if C::HELP.get_from(&r).unwrap_or(false) {
        print!("{}", cli::usage());
        return ExitCode::SUCCESS;
    }
    let plan = match plan_from(&r) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("walleye-tas2557-builds: {e}");
            return ExitCode::from(2);
        }
    };
    let v = prove(&plan);
    println!("{}", line(&v));
    if matches!(v, Verdict::Pass(_)) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn plan_from(r: &argenv_resolve::Resolution) -> Result<Plan, String> {
    let kernel = C::KERNEL
        .get_from(r)
        .ok_or("--kernel KERNEL_CLONE is required")?;
    let work = match C::WORK.get_from(r) {
        Some(w) => PathBuf::from(w),
        None => PathBuf::from(
            std::env::var("AGENT_BUILD")
                .map_err(|_| "no --work and no AGENT_BUILD: large outputs stay on disk")?,
        )
        .join("tas2557-builds"),
    };
    fs::create_dir_all(&work).map_err(|e| format!("{}: {e}", work.display()))?;
    Ok(Plan {
        kernel: kernel.into(),
        topic: C::TOPIC
            .get_from(r)
            .unwrap_or_else(|| "topic/walleye-tas2557-v3".into()),
        base: C::BASE.get_from(r).unwrap_or_else(|| "v7.3".into()),
        base_dtb: C::BASE_DTB.get_from(r).map(PathBuf::from),
        overlay: C::OVERLAY
            .get_from(r)
            .unwrap_or_else(|| "dt/msm8998-google-walleye-tas2557.dtso".into())
            .into(),
        work,
        image: C::IMAGE.get_from(r).unwrap_or_else(|| PINNED_IMAGE.into()),
        cross_compile: C::CROSS_COMPILE
            .get_from(r)
            .unwrap_or_else(|| "aarch64-linux-gnu-".into()),
    })
}

/// The procedure: the series, then a tree of it, then the three checks that read it.
fn prove(p: &Plan) -> Verdict {
    for r in [&p.base, &p.topic] {
        if !steps::ref_exists(&p.kernel, r) {
            return Verdict::Skip(format!("{} has no {r}: not known yet", p.kernel.display()));
        }
    }
    let text = match steps::commits(p) {
        Ok(t) => t,
        Err(e) => return Verdict::Fail(e),
    };
    if let Err(e) = keep(p, "commits.txt", &text) {
        return Verdict::Fail(e);
    }
    let tip = match series(&text) {
        Ok(t) => t,
        Err(e) => return Verdict::Fail(format!("series: {e}")),
    };
    if let Err(e) = steps::sits_on_base(p) {
        return Verdict::Fail(format!("series: {e}"));
    }
    let tree = match steps::Tree::of_topic(p) {
        Ok(t) => t,
        Err(e) => return Verdict::Fail(e),
    };
    let parts = [
        (
            "driver build",
            judged(
                p,
                "build-W1.txt",
                &format!("{}gcc", p.cross_compile),
                driver_build,
                || steps::driver_build(p, &tree),
            ),
        ),
        (
            "binding",
            judged(p, "dt_binding_check.txt", "docker", binding, || {
                steps::binding_check(p, &tree)
            }),
        ),
        ("overlay", overlay_part(p, &tree)),
    ];
    conclude(&tip, &parts)
}

/// One step and its judge: a missing tool is `skip` (not known yet), a step that fails to run is `fail`.
fn judged(
    p: &Plan,
    file: &str,
    program: &str,
    judge: fn(&str) -> Result<(), String>,
    step: impl FnOnce() -> Result<String, String>,
) -> Verdict {
    if !steps::have(program) {
        return Verdict::Skip(format!("no {program}: not known yet"));
    }
    match step() {
        Err(e) => Verdict::Fail(e),
        Ok(text) => {
            if let Err(e) = keep(p, file, &text) {
                return Verdict::Fail(e);
            }
            match judge(&text) {
                Ok(()) => Verdict::Pass(String::new()),
                Err(e) => Verdict::Fail(e),
            }
        }
    }
}

fn overlay_part(p: &Plan, tree: &steps::Tree) -> Verdict {
    let Some(base) = &p.base_dtb else {
        return Verdict::Skip("no --base-dtb: the overlay merge is not known yet".into());
    };
    for tool in ["fdtoverlay", "fdtget", "dtc", "cpp"] {
        if !steps::have(tool) {
            return Verdict::Skip(format!("no {tool}: not known yet"));
        }
    }
    match steps::overlay_read(p, tree, base) {
        Err(e) => Verdict::Fail(e),
        Ok(text) => {
            if let Err(e) = keep(p, "overlay-merge.txt", &text) {
                return Verdict::Fail(e);
            }
            match overlay(&text) {
                Ok(()) => Verdict::Pass(String::new()),
                Err(e) => Verdict::Fail(e),
            }
        }
    }
}

/// The text each step printed stays in the work folder, for a research row.
fn keep(p: &Plan, file: &str, text: &str) -> Result<(), String> {
    let path = p.work.join(file);
    fs::write(&path, text).map_err(|e| format!("cannot keep {}: {e}", path.display()))
}
