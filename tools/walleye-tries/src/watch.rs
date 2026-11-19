// SPDX-License-Identifier: MIT OR Apache-2.0
//! The way back as the last act of an attended try, however it ended (item walleye-pmic-program-way-back).
//! The runner ends the process at the first failed step or unanswered ask, and Ctrl-C ends it too: nothing of the
//! program runs after. So the try runs itself as a child; this parent waits for it, then runs the way back, then
//! ends with the child's own exit code (a failed or stopped run still counts as one). A dry run is not watched.

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Set in the child's environment: this process is the try itself, not its watcher.
pub const CHILD: &str = "WALLEYE_TRY_IS_THE_CHILD";
/// The file the child creates when it first acts on the phone: the parent runs the way back only if it exists.
pub const TOUCHED: &str = "WALLEYE_TRY_TOUCHED_FILE";

/// The child says it is about to act on the phone (nothing happens when this process is not a watched child).
pub fn phone_is_touched() {
    if let Some(p) = std::env::var_os(TOUCHED) {
        let _ = std::fs::write(p, "");
    }
}

/// Whether this process watches a child: a real run, and not the child itself.
pub fn watches(args: &[String]) -> bool {
    std::env::var_os(CHILD).is_none() && !args.iter().any(|a| a == "--dry-run" || a == "--help")
}

/// This program again, with its arguments, as the child.
pub fn the_try_again(touched: &std::path::Path) -> Result<Command, String> {
    let exe =
        std::env::current_exe().map_err(|e| format!("cannot find the program itself: {e}"))?;
    let mut c = Command::new(exe);
    c.args(std::env::args().skip(1))
        .env(CHILD, "1")
        .env(TOUCHED, touched);
    Ok(c)
}

/// Run the child to its end - or to `interrupted`, when it is killed - and give its exit code (130 when a signal
/// ended it).
pub fn run_child(mut child: Command, interrupted: &AtomicBool) -> Result<i32, String> {
    let mut child = child
        .spawn()
        .map_err(|e| format!("cannot start the try: {e}"))?;
    loop {
        let waited = child.try_wait();
        if let Err(e) = &waited {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e.to_string());
        }
        if let Ok(Some(status)) = waited {
            return Ok(status.code().unwrap_or_else(|| {
                128 + std::os::unix::process::ExitStatusExt::signal(&status).unwrap_or(2)
            }));
        }
        if interrupted.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(130);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// The watching parent of a real run: Ctrl-C and a hang-up are caught (they reach the child too), the try is run
/// again as the child, the way back `back` runs after it, and this process ends with the run's exit code.
pub fn watch_the_try(back: impl FnOnce() -> Result<(), String>) -> ! {
    if let Err(e) = ctrlc::set_handler(|| INTERRUPTED.store(true, Ordering::SeqCst)) {
        eprintln!("cannot catch Ctrl-C ({e}): a Ctrl-C would leave the phone where it is");
        std::process::exit(2);
    }
    let touched = std::env::temp_dir().join(format!("walleye-try-touched-{}", std::process::id()));
    let _ = std::fs::remove_file(&touched);
    match the_try_again(&touched) {
        Ok(child) => {
            let code = watch(child, &touched, &INTERRUPTED, back);
            let _ = std::fs::remove_file(&touched);
            std::process::exit(code)
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2)
        }
    }
}

/// The parent's whole part: the child, then the way back (`back`). The exit code of the run: the child's, and 1
/// when the child was clean but the phone did not return.
pub fn watch(
    child: Command,
    touched: &std::path::Path,
    interrupted: &AtomicBool,
    back: impl FnOnce() -> Result<(), String>,
) -> i32 {
    let code = run_child(child, interrupted).unwrap_or_else(|e| {
        eprintln!("{e}");
        1
    });
    if code != 0 {
        eprintln!("the run stopped (exit {code})");
    }
    // a run that never acted on the phone leaves it alone: it may be another session's
    if !touched.exists() {
        eprintln!("the phone was not touched: it is left as it is");
        return code;
    }
    eprintln!("the phone goes back to Android if it is not there");
    match back() {
        Ok(()) => code,
        Err(e) => {
            eprintln!("{e}");
            if code == 0 {
                1
            } else {
                code
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// A marker that exists: the child acted on the phone.
    fn marked() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("wt-marked-{}", std::process::id()));
        std::fs::write(&p, "").unwrap();
        p
    }

    fn sh(script: &str) -> Command {
        let mut c = Command::new("sh");
        c.args(["-c", script]);
        c
    }

    #[test]
    fn a_child_that_failed_still_gets_the_way_back_and_its_code_stands() {
        let ran = Cell::new(0);
        let no = AtomicBool::new(false);
        let code = watch(sh("exit 1"), &marked(), &no, || {
            ran.set(ran.get() + 1);
            Ok(())
        });
        assert_eq!((code, ran.get()), (1, 1));
    }

    #[test]
    fn a_clean_child_gets_the_way_back_too_and_it_costs_nothing_on_android() {
        let ran = Cell::new(0);
        let code = watch(sh("exit 0"), &marked(), &AtomicBool::new(false), || {
            ran.set(1);
            Ok(())
        });
        assert_eq!((code, ran.get()), (0, 1));
    }

    #[test]
    fn a_ctrl_c_kills_the_child_and_the_way_back_runs() {
        let ran = Cell::new(false);
        let stop = AtomicBool::new(true);
        let t0 = std::time::Instant::now();
        let code = watch(sh("sleep 30"), &marked(), &stop, || {
            ran.set(true);
            Ok(())
        });
        assert_eq!((code, ran.get()), (130, true));
        assert!(t0.elapsed().as_secs() < 5);
    }

    #[test]
    fn a_phone_that_does_not_return_ends_the_run_non_zero_even_after_a_clean_child() {
        let code = watch(sh("exit 0"), &marked(), &AtomicBool::new(false), || {
            Err("hold power".into())
        });
        assert_eq!(code, 1);
        let code = watch(sh("exit 2"), &marked(), &AtomicBool::new(false), || {
            Err("hold power".into())
        });
        assert_eq!(code, 2);
    }

    #[test]
    fn a_child_that_cannot_start_is_a_failed_run_with_the_way_back() {
        let ran = Cell::new(false);
        let code = watch(
            Command::new("/nonexistent/try"),
            &marked(),
            &AtomicBool::new(false),
            || {
                ran.set(true);
                Ok(())
            },
        );
        assert_eq!((code, ran.get()), (1, true));
    }

    #[test]
    fn a_run_that_never_touched_the_phone_leaves_it_alone() {
        let ran = Cell::new(false);
        let none = std::env::temp_dir().join("wt-never-marked");
        let _ = std::fs::remove_file(&none);
        let code = watch(sh("exit 1"), &none, &AtomicBool::new(false), || {
            ran.set(true);
            Ok(())
        });
        assert_eq!((code, ran.get()), (1, false));
    }

    #[test]
    fn a_dry_run_and_the_child_are_not_watched() {
        let a = |s: &[&str]| s.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert_eq!(
            watches(&a(&["--log", "x"])),
            std::env::var_os(CHILD).is_none()
        );
        assert!(!watches(&a(&["--dry-run"])));
        assert!(!watches(&a(&["--help"])));
    }
}
