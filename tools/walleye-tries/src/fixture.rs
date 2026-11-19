// SPDX-License-Identifier: MIT OR Apache-2.0
//! Test fixtures: stand-in programs the tests run (a fastboot, an adb, a check script).

use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

/// Write an executable script at `path`, whole and closed before anyone can run it, and not held open for
/// writing by any process.
///
/// The file is a new one (an older file of that name is unlinked first, so a copy still running keeps its
/// own), created with its mode, written, synced and closed. That alone is not enough: the tests run in threads
/// of one process, and a thread that forks while this file is open hands the child a copy of the handle until
/// the child's exec; exec of a file open for writing anywhere fails with "Text file busy". So the helper waits,
/// for a bounded time, until no process holds the file open - a condition it reads, not a try it repeats. The wait is best-effort: a child is listed in /proc only once the fork has finished, a window of microseconds it cannot see.
pub fn executable(path: &Path, body: &str) {
    let _ = std::fs::remove_file(path);
    {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o755)
            .open(path)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        f.write_all(body.as_bytes()).unwrap();
        f.sync_all().unwrap();
    }
    let path = path.canonicalize().unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    while held_open(&path) {
        assert!(
            Instant::now() < until,
            "{} is still open in another process after 10 s",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Some process of ours has `path` open (a forked child that has not yet exec'd).
fn held_open(path: &Path) -> bool {
    let procs = std::fs::read_dir("/proc").unwrap_or_else(|e| panic!("/proc: {e}"));
    procs
        .flatten()
        .filter(|p| {
            p.file_name()
                .to_string_lossy()
                .bytes()
                .all(|b| b.is_ascii_digit())
        })
        .filter_map(|p| std::fs::read_dir(p.path().join("fd")).ok())
        .flat_map(|fds| fds.flatten())
        .any(|fd| std::fs::read_link(fd.path()).is_ok_and(|l| l == path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixture_runs_at_once_while_other_threads_spawn() {
        let d = std::env::temp_dir().join(format!("wt-fixture-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let busy: Vec<_> = (0..4)
            .map(|_| {
                std::thread::spawn(|| {
                    for _ in 0..200 {
                        let _ = std::process::Command::new("true").status();
                    }
                })
            })
            .collect();
        for i in 0..200 {
            let p = d.join(format!("s{i}"));
            executable(&p, "#!/bin/sh\nexit 0\n");
            assert!(std::process::Command::new(&p).status().unwrap().success());
        }
        busy.into_iter().for_each(|t| t.join().unwrap());
        std::fs::remove_dir_all(d).ok();
    }
}
