// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-sources: the gate that every check names its source (practice: a check names where its expectation came from).

mod cli;

use argenv_resolve::InputResolveExt;
use cli::Contract as C;
use std::path::Path;
use walleye_sources::{run_gate, Gate};

fn main() {
    let r = C::parse_and_lint(argenv_resolve::OnProblems::FailOnError);
    if C::HELP.get_from(&r).unwrap_or(false) {
        print!("{}", cli::usage());
        return;
    }
    let gate = Gate {
        root: C::ROOT.get_from(&r).unwrap_or_else(|| ".".into()),
        dir: C::DIR.get_from(&r).unwrap_or_else(|| "tests/host".into()),
        table: C::TABLE
            .get_from(&r)
            .unwrap_or_else(|| "tests/host/SOURCES.toml".into()),
        ext: C::EXT.get_from(&r).unwrap_or_else(|| "sh".into()),
    };
    let (text, ok) = run_gate(&gate, Path::new(&gate.root));
    print!("{text}");
    if !ok {
        std::process::exit(1);
    }
}
