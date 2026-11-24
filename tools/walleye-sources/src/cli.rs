// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-sources's invocation surface, declared once with argenv.
//!   walleye-sources [--root DIR] [--dir DIR] [--table FILE] [--ext EXT]

use argenv_resolve::*;

contract! {
    ROOT: String = Input {
        key: "root",
        ty: Type::String,
        arg: Some(Arg { value_name: "DIR", ..Arg::long("root") }),
        env: Some(Env::new("WALLEYE_SOURCES_ROOT")),
        summary: "The lab's repository root, where data/research.json, data/sources.json and `file:` sources are read (default: .).",
        ..Input::EMPTY
    };

    DIR: String = Input {
        key: "dir",
        ty: Type::String,
        arg: Some(Arg { value_name: "DIR", ..Arg::long("dir") }),
        env: Some(Env::new("WALLEYE_SOURCES_DIR")),
        summary: "The folder, under the root, whose files each need a source (default: tests/host).",
        ..Input::EMPTY
    };

    TABLE: String = Input {
        key: "table",
        ty: Type::String,
        arg: Some(Arg { value_name: "FILE", ..Arg::long("table") }),
        env: Some(Env::new("WALLEYE_SOURCES_TABLE")),
        summary: "The table of file to source, under the root (default: tests/host/SOURCES.toml).",
        ..Input::EMPTY
    };

    EXT: String = Input {
        key: "ext",
        ty: Type::String,
        arg: Some(Arg { value_name: "EXT", ..Arg::long("ext") }),
        env: Some(Env::new("WALLEYE_SOURCES_EXT")),
        summary: "The extension of the files that need a source (default: sh).",
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
        "Usage: walleye-sources [--root DIR] [--dir DIR] [--table FILE] [--ext EXT]\n\nRefuse a file with no source, a source that does not exist, and a table row for a file that is gone; list the files marked not yet sourced.\n\nArguments:\n",
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
