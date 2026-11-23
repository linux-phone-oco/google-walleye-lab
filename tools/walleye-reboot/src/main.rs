// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-reboot: restart the phone with a command string (`--command bootloader`), which busybox's reboot cannot pass.

mod cli;

use argenv_resolve::InputResolveExt;
use cli::Contract as C;
use walleye_reboot::{command_string, restart_with, Real};

fn main() {
    let r = C::parse_and_lint(argenv_resolve::OnProblems::FailOnError); // an unknown flag (--dryrun) stops here, before anything can restart the phone
    if C::HELP.get_from(&r).unwrap_or(false) {
        print!("{}", cli::usage());
        return;
    }
    let Some(command) = C::COMMAND.get_from(&r) else {
        eprintln!(
            "walleye-reboot: --command STRING is required\n\n{}",
            cli::usage()
        );
        std::process::exit(2);
    };
    if C::DRY_RUN.get_from(&r).unwrap_or(false) {
        match command_string(&command) {
            Ok(c) => println!("walleye-reboot: dry run: sync, then reboot(RESTART2, {c:?})"),
            Err(e) => {
                eprintln!("walleye-reboot: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    if let Err(e) = restart_with(&Real, &command) {
        eprintln!("walleye-reboot: {e}");
        std::process::exit(1);
    }
}
