// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-ramroot's invocation surface, declared once with argenv.
//!   walleye-ramroot --build now | --check IMAGE   [--repo DIR] [--work DIR] [--images DIR] [--out DIR]

use argenv_resolve::*;

contract! {
    BUILD: String = Input {
        key: "build",
        ty: Type::String,
        arg: Some(Arg { value_name: "now", ..Arg::long("build") }),
        summary: "Build the image twice, compare, check, and write it (explicit intent: the value must be `now`).",
        ..Input::EMPTY
    };

    CHECK: String = Input {
        key: "check",
        ty: Type::String,
        arg: Some(Arg { value_name: "IMAGE", ..Arg::long("check") }),
        summary: "Check an existing image against the item's proof, and change nothing.",
        ..Input::EMPTY
    };

    REPO: String = Input {
        key: "repo",
        ty: Type::String,
        arg: Some(Arg { value_name: "DIR", ..Arg::long("repo") }),
        env: Some(Env::new("WALLEYE_RAMROOT_REPO")),
        summary: "The google-walleye-lab checkout holding device.toml, build/ and tests/ (default: the current directory).",
        ..Input::EMPTY
    };

    WORK: String = Input {
        key: "work",
        ty: Type::String,
        arg: Some(Arg { value_name: "DIR", ..Arg::long("work") }),
        env: Some(Env::new("AGENT_BUILD")),
        summary: "Where the trees and tars go (default: $AGENT_BUILD/ramroot; large outputs belong on disk).",
        ..Input::EMPTY
    };

    IMAGES: String = Input {
        key: "images",
        ty: Type::String,
        arg: Some(Arg { value_name: "DIR", ..Arg::long("images") }),
        env: Some(Env::new("WALLEYE_RAMROOT_IMAGES")),
        summary: "The folder device.toml's image paths are under (default ~/Projects/walleye-build, the lab's images).",
        ..Input::EMPTY
    };

    OUT: String = Input {
        key: "out",
        ty: Type::String,
        arg: Some(Arg { value_name: "DIR", ..Arg::long("out") }),
        summary: "The folder to write boot-b-a5.img and SHA256SUMS-display-ramroot to (default: IMAGES/images-display-ramroot).",
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
        "Usage: walleye-ramroot (--build now | --check IMAGE) [OPTIONS]\n\nBuild the Pixel 2's Alpine-in-RAM boot image (B-a5) from B-r5 and the system_a rootfs image, or check an image.\n\nArguments:\n",
    );
    for r in &Contract::records() {
        let sig = r.usage();
        if !sig.is_empty() {
            out.push_str(&format!(
                "  {sig:<28} {}\n",
                r.summary.clone().unwrap_or_default()
            ));
        }
    }
    out
}
