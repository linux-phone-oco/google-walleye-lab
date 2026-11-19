// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-stripes' invocation surface, declared once with argenv.
//!   walleye-stripes [--seconds N] [--device FILE] [--connector DIR] [--kmsg FILE]

use argenv_resolve::*;

contract! {
    SECONDS: u64 = Input {
        key: "seconds",
        ty: Type::Uint,
        default: Some(20),
        arg: Some(Arg { value_name: "N", ..Arg::long("seconds") }),
        env: Some(Env::new("WALLEYE_STRIPES_SECONDS")),
        summary: "How many seconds the stripes are held before the screen is cleared (default 20).",
        ..Input::EMPTY
    };

    DEVICE: String = Input {
        key: "device",
        ty: Type::String,
        arg: Some(Arg { value_name: "FILE", ..Arg::long("device") }),
        summary: "The framebuffer of the display (default /dev/fb0, the fbdev emulation of card0).",
        ..Input::EMPTY
    };

    CONNECTOR: String = Input {
        key: "connector",
        ty: Type::String,
        arg: Some(Arg { value_name: "DIR", ..Arg::long("connector") }),
        summary: "The connector's sysfs folder, read before and after the draw (default /sys/class/drm/card0-DSI-1).",
        ..Input::EMPTY
    };

    KMSG: String = Input {
        key: "kmsg",
        ty: Type::String,
        arg: Some(Arg { value_name: "FILE", ..Arg::long("kmsg") }),
        summary: "Where the log lines go besides standard output (default /dev/kmsg, so the netconsole carries them).",
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
        "Usage: walleye-stripes [OPTIONS]\n\nDraw four vertical stripes (red, green, blue, white) on the framebuffer, hold them, clear the screen, and say what the connector reports.\n\nArguments:\n",
    );
    for r in &Contract::records() {
        let sig = r.usage();
        if !sig.is_empty() {
            out.push_str(&format!("  {sig:<24} {}\n", r.summary.clone().unwrap_or_default()));
        }
    }
    out
}
