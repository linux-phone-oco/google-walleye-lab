// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-cmdline-budget's invocation surface, declared once with argenv.
//!   walleye-cmdline-budget [--device FILE] [--exceptions FILE] [--max N]

use argenv_resolve::*;

contract! {
    DEVICE: String = Input {
        key: "device",
        ty: Type::String,
        arg: Some(Arg { value_name: "FILE", ..Arg::long("device") }),
        env: Some(Env::new("WALLEYE_CMDLINE_BUDGET_DEVICE")),
        summary: "The manifest whose [images.NAME.recipe] command lines are measured (default: device.toml).",
        ..Input::EMPTY
    };

    EXCEPTIONS: String = Input {
        key: "exceptions",
        ty: Type::String,
        arg: Some(Arg { value_name: "FILE", ..Arg::long("exceptions") }),
        env: Some(Env::new("WALLEYE_CMDLINE_BUDGET_EXCEPTIONS")),
        summary: "The table of images allowed over the budget, each with its reason (default: tests/host/cmdline-over-budget.toml).",
        ..Input::EMPTY
    };

    MAX: i64 = Input {
        key: "max",
        ty: Type::Int,
        arg: Some(Arg { value_name: "N", ..Arg::long("max") }),
        env: Some(Env::new("WALLEYE_CMDLINE_BUDGET_MAX")),
        summary: "The most characters a recipe's command line may have (default: 160).",
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
        "Usage: walleye-cmdline-budget [--device FILE] [--exceptions FILE] [--max N]\n\nFail a recipe command line over the budget, an exception for an image that can be booted, one that is no longer over, or one that is gone.\n\nArguments:\n",
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
