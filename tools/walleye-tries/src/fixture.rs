// SPDX-License-Identifier: MIT OR Apache-2.0
//! Test fixtures: stand-in programs the tests run (a fastboot, an adb, a check script).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Write an executable script at `path`, whole and closed before anyone can run it.
///
/// The file is written by a child process (`cat`), never by this one. The tests run in threads of one
/// process, and a thread that forks while this process holds the file open for writing hands its child a copy
/// of the handle until that child's exec; exec of a file open for writing anywhere fails with "Text file busy"
/// (ETXTBSY, os error 26). A write handle that never exists in this process cannot be inherited: the child
/// that holds it has exited, and been waited for, before this function returns. The only handle this process
/// holds is the pipe to the child, which a forked sibling may keep for microseconds and which no exec cares
/// about. An older file of that name is unlinked first, so a copy still running keeps its own.
pub fn executable(path: &Path, body: &str) {
    let _ = std::fs::remove_file(path);
    let mut writer = Command::new("sh")
        .args(["-c", "set -C; cat > \"$1\" && chmod 755 \"$1\"", "sh"])
        .arg(path)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    writer
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(body.as_bytes())
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let status = writer.wait().unwrap();
    assert!(status.success(), "{}: writing it failed", path.display());
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
