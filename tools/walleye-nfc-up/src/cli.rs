// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-nfc-up's invocation surface, declared once with argenv.
//!   walleye-nfc-up [--index N] [--dry-run]

use argenv_resolve::*;

contract! {
    INDEX: u64 = Input {
        key: "index",
        ty: Type::Uint,
        arg: Some(Arg { value_name: "N", ..Arg::long("index") }),
        env: Some(Env::new("WALLEYE_NFC_UP_INDEX")),
        default: Some(0),
        summary: "The NFC device to open: 0 is nfc0 (default 0).",
        ..Input::EMPTY
    };

    DRY_RUN: bool = Input {
        key: "dry_run",
        ty: Type::Bool,
        arg: Some(Arg::long("dry-run")),
        summary: "Print the two netlink messages that would be sent and stop: no socket is opened.",
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
        "Usage: walleye-nfc-up [--index N] [--dry-run]\n\nAsk the kernel's NFC netlink family to power NFC device N on (NFC_CMD_DEV_UP) and print its answer: the kernel's own, an errno or ok.\n\nArguments:\n",
    );
    for r in &Contract::records() {
        let sig = r.usage();
        if !sig.is_empty() {
            out.push_str(&format!(
                "  {sig:<24} {}\n",
                r.summary.clone().unwrap_or_default()
            ));
        }
    }
    out
}
