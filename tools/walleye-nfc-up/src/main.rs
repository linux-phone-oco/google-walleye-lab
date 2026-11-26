// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-nfc-up: power an NFC device on through the kernel's NFC netlink family and print the kernel's answer.

mod cli;

use argenv_resolve::{InputResolveDefaultExt, InputResolveExt};
use cli::Contract as C;
use walleye_nfc_up::{dry_run_lines, open_device, Answer, Real};

fn main() {
    let r = C::parse_and_lint(argenv_resolve::OnProblems::FailOnError); // an unknown flag (--dryrun) stops here, before anything is sent
    if C::HELP.get_from(&r).unwrap_or(false) {
        print!("{}", cli::usage());
        return;
    }
    let index = C::INDEX.get_from_or_default(&r).unwrap_or(0);
    let Ok(index) = u32::try_from(index) else {
        eprintln!("walleye-nfc-up: --index {index} is not a device number");
        std::process::exit(2);
    };
    if C::DRY_RUN.get_from(&r).unwrap_or(false) {
        dry_run_lines(index).iter().for_each(|l| println!("{l}"));
        return;
    }
    match open_device(&Real, index) {
        Ok(answer) => {
            println!("walleye-nfc-up: {}", answer.line(index)); // the kernel's own answer, as it came
            if answer != Answer::Up {
                std::process::exit(1); // an errno is not "on": the procedure reads rc= beside the line
            }
        }
        Err(e) => {
            eprintln!("walleye-nfc-up: {e}");
            std::process::exit(1);
        }
    }
}
