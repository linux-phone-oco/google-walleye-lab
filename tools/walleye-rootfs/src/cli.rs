// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-rootfs' invocation surface, declared once with argenv.
//!   walleye-rootfs --build now | --check IMAGE   [--repo DIR] [--work DIR] [--out FILE] [--release VER] [--mirror URL] [--authorized-key FILE]

use argenv_resolve::*;

contract! {
    BUILD: String = Input {
        key: "build",
        ty: Type::String,
        arg: Some(Arg { value_name: "now", ..Arg::long("build") }),
        summary: "Build the image twice and compare (explicit intent: the value must be `now`).",
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
        env: Some(Env::new("WALLEYE_ROOTFS_REPO")),
        summary: "The google-walleye-lab checkout holding rootfs/ and device.toml (default: the current directory).",
        ..Input::EMPTY
    };

    WORK: String = Input {
        key: "work",
        ty: Type::String,
        arg: Some(Arg { value_name: "DIR", ..Arg::long("work") }),
        env: Some(Env::new("AGENT_BUILD")),
        summary: "Where downloads, trees and images go (default: $AGENT_BUILD/rootfs; large outputs belong on disk).",
        ..Input::EMPTY
    };

    OUT: String = Input {
        key: "out",
        ty: Type::String,
        arg: Some(Arg { value_name: "FILE", ..Arg::long("out") }),
        summary: "The image to write (default: WORK/walleye-alpine.ext4).",
        ..Input::EMPTY
    };

    RELEASE: String = Input {
        key: "release",
        ty: Type::String,
        arg: Some(Arg { value_name: "VER", ..Arg::long("release") }),
        summary: "The Alpine release (default 3.24).",
        ..Input::EMPTY
    };

    MIRROR: String = Input {
        key: "mirror",
        ty: Type::String,
        arg: Some(Arg { value_name: "URL", ..Arg::long("mirror") }),
        env: Some(Env::new("WALLEYE_ROOTFS_MIRROR")),
        summary: "The Alpine mirror (default https://dl-cdn.alpinelinux.org/alpine).",
        ..Input::EMPTY
    };

    KEY: String = Input {
        key: "authorized-key",
        ty: Type::String,
        arg: Some(Arg { value_name: "FILE", ..Arg::long("authorized-key") }),
        summary: "A public key put in root's authorized_keys: the one way in over usb-network (none given: none is put in, and the check says so).",
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
        "Usage: walleye-rootfs (--build now | --check IMAGE) [OPTIONS]\n\nBuild the Pixel 2's plain Alpine hardware-test rootfs without root, or check an image.\n\nArguments:\n",
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
