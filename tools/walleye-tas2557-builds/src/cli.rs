// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-tas2557-builds's invocation surface, declared once with argenv.
//!   walleye-tas2557-builds --kernel KERNEL_CLONE [--topic REF] [--base REF] [--base-dtb FILE] [--overlay FILE] [--work DIR] [--image IMAGE] [--cross-compile PREFIX]

use argenv_resolve::*;

contract! {
    KERNEL: String = Input {
        key: "kernel",
        ty: Type::String,
        arg: Some(Arg { value_name: "KERNEL_CLONE", ..Arg::long("kernel") }),
        env: Some(Env::new("WALLEYE_TAS2557_KERNEL")),
        summary: "The kernel git clone that holds the topic and its base (read through throwaway worktrees; nothing in it changes).",
        ..Input::EMPTY
    };

    TOPIC: String = Input {
        key: "topic",
        ty: Type::String,
        arg: Some(Arg { value_name: "REF", ..Arg::long("topic") }),
        env: Some(Env::new("WALLEYE_TAS2557_TOPIC")),
        summary: "The series' branch (default: topic/walleye-tas2557-v3).",
        ..Input::EMPTY
    };

    BASE: String = Input {
        key: "base",
        ty: Type::String,
        arg: Some(Arg { value_name: "REF", ..Arg::long("base") }),
        env: Some(Env::new("WALLEYE_TAS2557_BASE")),
        summary: "The mainline tag the series sits on (default: v7.3).",
        ..Input::EMPTY
    };

    BASE_DTB: String = Input {
        key: "base_dtb",
        ty: Type::String,
        arg: Some(Arg { value_name: "FILE", ..Arg::long("base-dtb") }),
        env: Some(Env::new("WALLEYE_TAS2557_BASE_DTB")),
        summary: "A walleye dtb built with symbols (DTC_FLAGS=-@) for the overlay to merge onto; without it the merge is `skip`, not known yet.",
        ..Input::EMPTY
    };

    OVERLAY: String = Input {
        key: "overlay",
        ty: Type::String,
        arg: Some(Arg { value_name: "FILE", ..Arg::long("overlay") }),
        env: Some(Env::new("WALLEYE_TAS2557_OVERLAY")),
        summary: "The overlay source (default: dt/msm8998-google-walleye-tas2557.dtso).",
        ..Input::EMPTY
    };

    WORK: String = Input {
        key: "work",
        ty: Type::String,
        arg: Some(Arg { value_name: "DIR", ..Arg::long("work") }),
        env: Some(Env::new("WALLEYE_TAS2557_WORK")),
        summary: "Where the build tree and outputs go (default: $AGENT_BUILD/tas2557-builds; large outputs stay on disk, not in /tmp).",
        ..Input::EMPTY
    };

    IMAGE: String = Input {
        key: "image",
        ty: Type::String,
        arg: Some(Arg { value_name: "IMAGE", ..Arg::long("image") }),
        env: Some(Env::new("WALLEYE_TAS2557_IMAGE")),
        summary: "The container image with py3-dt-schema (default: the lab's pinned alpine:edge digest).",
        ..Input::EMPTY
    };

    CROSS_COMPILE: String = Input {
        key: "cross_compile",
        ty: Type::String,
        arg: Some(Arg { value_name: "PREFIX", ..Arg::long("cross-compile") }),
        env: Some(Env::new("CROSS_COMPILE")),
        summary: "The arm64 compiler prefix (default: aarch64-linux-gnu-).",
        ..Input::EMPTY
    };

    HELP: bool = Input {
        key: "help",
        ty: Type::Bool,
        arg: Some(Arg { short: Some('h'), ..Arg::long("help") }),
        summary: "Print this usage and stop.",
        ..Input::EMPTY
    };
}

/// The usage `--help` prints, read from the declaration above.
pub fn usage() -> String {
    let mut out = String::from(
        "Usage: walleye-tas2557-builds --kernel KERNEL_CLONE [--topic REF] [--base REF] [--base-dtb FILE] [--overlay FILE] [--work DIR] [--image IMAGE] [--cross-compile PREFIX]\n\nRun the four checks of walleye-tas2557-driver-builds on the lab host and print `pass|fail|skip walleye-tas2557-driver-builds host DETAIL`; exit 0 only on pass.\n\nArguments:\n",
    );
    for r in &Contract::records() {
        let sig = r.usage();
        if !sig.is_empty() {
            out.push_str(&format!(
                "  {sig:<30} {}\n",
                r.summary.clone().unwrap_or_default()
            ));
        }
    }
    out
}
