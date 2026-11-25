// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-cmdline-budget: every image recipe's command line stays within the header budget.

mod cli;

use argenv_resolve::InputResolveExt;
use cli::Contract as C;
use walleye_cmdline_budget::{check, DEFAULT_MAX};

fn main() {
    let r = C::parse_and_lint(argenv_resolve::OnProblems::FailOnError);
    if C::HELP.get_from(&r).unwrap_or(false) {
        print!("{}", cli::usage());
        return;
    }
    let device = C::DEVICE
        .get_from(&r)
        .unwrap_or_else(|| "device.toml".into());
    let exceptions = C::EXCEPTIONS
        .get_from(&r)
        .unwrap_or_else(|| "tests/host/cmdline-over-budget.toml".into());
    let max = match C::MAX.get_from(&r) {
        None => DEFAULT_MAX,
        Some(n) => match usize::try_from(n) {
            Ok(m) if m >= 1 => m,
            _ => {
                eprintln!("walleye-cmdline-budget: --max must be 1 or more, not {n}");
                std::process::exit(2);
            }
        },
    };
    let (text, ok) = check(&device, &exceptions, max);
    print!("{text}");
    if !ok {
        std::process::exit(1);
    }
}
