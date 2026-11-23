// SPDX-License-Identifier: MIT OR Apache-2.0
//! The phone while it runs Android, as the lab reaches it: adb calls, the slot Android runs from, adbd as root, and
//! the sha256 of a partition or file read over adb root. Shared by the install write and the root's boot try, which
//! both read the card from Android before and after they act (item walleye-install-boot-procedure).

use crate::host;
use crate::lab::Lab;
use crate::settings::Settings;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The block the image and the partitions are written and read back in (dd's `bs` for a read).
pub const BLOCK: u64 = 4096;

/// A path on the phone that goes into a command as it is: absolute, only letters, digits and `. _ - /`, no `..`.
pub(crate) fn plain_path(path: &str) -> Result<&str, String> {
    let plain = path.starts_with('/')
        && path.split('/').all(|p| p != "..")
        && path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-/".contains(c));
    if plain {
        Ok(path)
    } else {
        Err(format!(
            "{path:?} is not a plain absolute path (letters, digits, . _ - /)"
        ))
    }
}

/// How long adbd has to answer as root after `adb root` restarts it.
const ROOT_WAIT: Duration = Duration::from_secs(40);

/// Empties the phone's page cache so a read-back reads the media (adb root).
const DROP_CACHES: &str = "sync; echo 3 > /proc/sys/vm/drop_caches";

/// How often a bounded read looks at its deadline.
const POLL: Duration = Duration::from_millis(50);

#[derive(Clone)]
pub struct Android {
    s: Rc<Settings>,
    lab: Lab,
    bound: ReadBound,
}

impl Android {
    pub fn new(s: Rc<Settings>, lab: Lab) -> Android {
        Android {
            s,
            lab,
            bound: ReadBound::LAB,
        }
    }

    /// The same phone with another bound on its reads (a test gives a stalled stream seconds, not minutes).
    pub fn with_bound(mut self, bound: ReadBound) -> Android {
        self.bound = bound;
        self
    }

    /// One adb call for this lab's phone; what it said, or why it failed.
    pub fn adb(&self, args: &[&str]) -> Result<String, String> {
        let mut c = Command::new(&self.s.adb);
        c.args(["-s", &self.lab.serial()?]).args(args);
        let said = host::run(c)?;
        if said.ok {
            Ok(said.text)
        } else {
            Err(format!("adb {}: {}", args.join(" "), said.text))
        }
    }

    /// The slot suffix Android booted with (`_a`, `_b`), as adb reads it.
    pub fn slot_suffix(&self) -> Result<String, String> {
        self.adb(&["shell", "getprop ro.boot.slot_suffix"])
    }

    /// The phone is on Android (out of fastboot if it is there) and adbd answers as root.
    pub fn is_up_as_root(&self) -> Result<(), String> {
        if self.lab.in_fastboot()? {
            let mut c = Command::new(&self.s.fastboot);
            c.args(["-s", &self.lab.serial()?, "reboot"]);
            host::run(c)?;
        }
        self.lab.android_is_back()?;
        let _ = self.adb(&["root"]);
        let until = Instant::now() + ROOT_WAIT;
        loop {
            match self.adb(&["shell", "id -u"]) {
                Ok(id) if id.trim() == "0" => return Ok(()),
                other if Instant::now() >= until => {
                    return Err(format!(
                        "adbd does not answer as root within {} s (adb root): {other:?}",
                        ROOT_WAIT.as_secs()
                    ))
                }
                _ => std::thread::sleep(Duration::from_secs(2)),
            }
        }
    }

    /// The sha256 of `blocks` blocks of a file or partition on the phone (all of it when None), read as root over adb and hashed here.
    /// The read is bounded (`ReadBound`): a stalled stream ends it, once retried, with the partition and how far it got.
    pub fn sha256(&self, path: &str, blocks: Option<u64>) -> Result<String, String> {
        let path = plain_path(path)?;
        let (read, bytes) = match blocks {
            Some(n) => (
                format!("dd if={path} bs={BLOCK} count={n} 2>/dev/null"),
                n.saturating_mul(BLOCK),
            ),
            None => (
                format!("dd if={path} bs={BLOCK} 2>/dev/null"),
                self.size_of(path)?,
            ),
        };
        self.retried_once(path, || self.sha256_of_read(&read, path, bytes))
    }

    /// A partition on the phone copied whole into `dest`, read as root over adb (`exec-out dd`): what a flash is about to overwrite and nothing else holds (research walleye-stage2-checks-should-they-exist, row 9).
    /// Bounded and retried once like `sha256`.
    pub fn copy_to(&self, path: &str, dest: &std::path::Path) -> Result<(), String> {
        let path = plain_path(path)?;
        let bytes = self.size_of(path)?;
        self.retried_once(path, || self.copy_once(path, dest, bytes))
    }

    /// The bytes of a file or partition on the phone (`blockdev --getsize64`, or `stat -L` for a plain file, as root over adb).
    fn size_of(&self, path: &str) -> Result<u64, String> {
        // a block device has st_size 0: blockdev first, stat for a plain file
        let said = self.adb(&[
            "exec-out",
            &format!("blockdev --getsize64 {path} 2>/dev/null || stat -L -c %s {path}"),
        ])?;
        said.trim()
            .parse()
            .map_err(|_| format!("the size of {path} is not a number: {said:?}"))
    }

    /// One read, and a second if the first stalled; any other failure ends at once.
    fn retried_once<T>(
        &self,
        what: &str,
        mut read: impl FnMut() -> Result<T, ReadError>,
    ) -> Result<T, String> {
        match read() {
            Ok(v) => Ok(v),
            Err(ReadError::Failed(e)) => Err(e),
            Err(ReadError::Stalled(first)) => {
                eprintln!("{first}; reading {what} once more");
                read().map_err(|e| match e {
                    ReadError::Failed(e) => e,
                    ReadError::Stalled(second) => {
                        format!("{second} (the second try; the first: {first})")
                    }
                })
            }
        }
    }

    fn copy_once(&self, path: &str, dest: &std::path::Path, bytes: u64) -> Result<(), ReadError> {
        let file = std::fs::File::create(dest)
            .map_err(|e| ReadError::Failed(format!("{}: {e}", dest.display())))?;
        let mut adb = Command::new(&self.s.adb)
            .args([
                "-s",
                &self.lab.serial().map_err(ReadError::Failed)?,
                "exec-out",
                &format!("dd if={path} bs={BLOCK} 2>/dev/null"),
            ])
            .stdout(Stdio::from(file))
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| ReadError::Failed(format!("cannot run {}: {e}", self.s.adb)))?;
        let limit = self.bound.limit(bytes);
        let started = Instant::now();
        loop {
            match adb.try_wait() {
                Ok(Some(status)) if status.success() => return Ok(()),
                Ok(Some(status)) => {
                    return Err(ReadError::Failed(format!(
                        "the copy of {path} did not finish (adb {status})"
                    )))
                }
                Ok(None) if started.elapsed() >= limit => {
                    let _ = adb.kill();
                    let _ = adb.wait();
                    let got = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
                    return Err(ReadError::Stalled(stalled_text(path, got, bytes, limit)));
                }
                Ok(None) => std::thread::sleep(POLL),
                Err(e) => return Err(ReadError::Failed(format!("adb: {e}"))),
            }
        }
    }

    /// The sha256 of what a read command on the phone writes to standard output (`what` names it in an error; `bytes` is what it should
    /// write, which sets the time it has). The page cache is emptied first: a read must come from the media.
    fn sha256_of_read(&self, read: &str, what: &str, bytes: u64) -> Result<String, ReadError> {
        self.adb(&["shell", DROP_CACHES])
            .map_err(ReadError::Failed)?;
        let mut adb = Command::new(&self.s.adb)
            .args([
                "-s",
                &self.lab.serial().map_err(ReadError::Failed)?,
                "exec-out",
                read,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| ReadError::Failed(format!("cannot run {}: {e}", self.s.adb)))?;
        let mut sum = match Command::new("sha256sum")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = adb.kill();
                let _ = adb.wait();
                return Err(ReadError::Failed(format!("cannot run sha256sum: {e}")));
            }
        };
        let mut from = adb
            .stdout
            .take()
            .ok_or_else(|| ReadError::Failed("adb has no output to read".into()))?;
        let mut into = sum
            .stdin
            .take()
            .ok_or_else(|| ReadError::Failed("sha256sum has no input".into()))?;
        let got = Arc::new(AtomicU64::new(0));
        let counted = got.clone();
        let pump = std::thread::spawn(move || {
            let mut buf = vec![0u8; 1 << 20];
            loop {
                match from.read(&mut buf) {
                    Ok(0) | Err(_) => return,
                    Ok(n) => {
                        if into.write_all(&buf[..n]).is_err() {
                            return;
                        }
                        counted.fetch_add(n as u64, Ordering::Relaxed);
                    }
                }
            }
        });
        let limit = self.bound.limit(bytes);
        let started = Instant::now();
        while !pump.is_finished() {
            if started.elapsed() >= limit {
                let _ = adb.kill();
                let _ = adb.wait();
                let _ = sum.kill();
                let _ = sum.wait();
                let _ = pump.join();
                let so_far = got.load(Ordering::Relaxed);
                return Err(ReadError::Stalled(stalled_text(what, so_far, bytes, limit)));
            }
            std::thread::sleep(POLL);
        }
        let _ = pump.join();
        let status = adb.wait().map_err(|e| ReadError::Failed(e.to_string()))?;
        let out = sum
            .wait_with_output()
            .map_err(|e| ReadError::Failed(format!("cannot run sha256sum: {e}")))?;
        if !status.success() {
            return Err(ReadError::Failed(format!(
                "the read of {what} did not finish (adb {status})"
            )));
        }
        String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .next()
            .filter(|h| h.len() == 64)
            .map(str::to_string)
            .ok_or_else(|| ReadError::Failed(format!("sha256sum gave no sum for {what}")))
    }
}

/// How long one read over adb has: a floor and the time the bytes take at the slowest rate the phone has shown.
#[derive(Clone, Copy)]
pub struct ReadBound {
    pub floor: Duration,
    pub bytes_per_second: u64,
}

impl ReadBound {
    /// The floor is 120 s and the rate 4 MiB/s (a 2 GiB read has 632 s): UFS reads run at tens of MiB/s, so this only ends a stream that stopped.
    pub const LAB: ReadBound = ReadBound {
        floor: Duration::from_secs(120),
        bytes_per_second: 4 << 20,
    };

    fn limit(&self, bytes: u64) -> Duration {
        self.floor + Duration::from_secs(bytes / self.bytes_per_second.max(1))
    }
}

/// Why a read ended: it stalled (worth one more try), or it failed (not).
enum ReadError {
    Stalled(String),
    Failed(String),
}

fn stalled_text(what: &str, got: u64, bytes: u64, limit: Duration) -> String {
    format!(
        "the read of {what} stalled: {got} of {bytes} bytes after the {} s bound (adb killed)",
        limit.as_secs()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::executable;

    const PART: &str = "/dev/block/by-name/boot_a";
    const TEXT: &str = "boot_a as the stand-in phone has it\n";

    /// A bench whose adb is a stand-in: `stat` says the size of TEXT; a read runs `adb_exec_out` (a shell body that sees `$n`, the number of this read).
    fn bench(name: &str, adb_exec_out: &str) -> (Android, std::path::PathBuf) {
        let d = std::env::temp_dir().join(format!("wt-android-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("device.toml"), "[connection]\nserial = \"SER1\"\n").unwrap();
        let adb = d.join("adb");
        executable(
            &adb,
            &format!(
                "#!/bin/sh\nshift 2\ncase \"$1\" in\n exec-out) case \"$2\" in blockdev*) echo {size}; exit 0;; esac; echo x >> {d}/reads\n  n=$(wc -l < {d}/reads)\n  {body}\n  ;;\nesac\nexit 0\n",
                size = TEXT.len(),
                d = d.display(),
                body = adb_exec_out
            ),
        );
        let mut s = Settings::stand_in(crate::settings::Try::FlashBoot);
        s.lab = d.clone();
        s.adb = adb.to_string_lossy().into_owned();
        let s = Rc::new(s);
        let a = Android::new(s.clone(), Lab::new(s)).with_bound(ReadBound {
            floor: Duration::from_secs(1),
            bytes_per_second: 1 << 20,
        });
        (a, d)
    }

    fn sum_of_text(d: &std::path::Path) -> String {
        let f = d.join("text");
        std::fs::write(&f, TEXT).unwrap();
        host::sha256_of(&f).unwrap()
    }

    const PRINTS: &str = "printf '%s' 'boot_a as the stand-in phone has it\n'";
    const STALLS_AFTER_3: &str = "printf abc; exec sleep 600";

    #[test]
    fn a_whole_read_gives_the_sum_of_what_the_stream_carried() {
        let (a, d) = bench("whole", PRINTS);
        let want = sum_of_text(&d);
        assert_eq!(a.sha256(PART, None).unwrap(), want);
    }

    #[test]
    fn a_stalled_stream_fails_in_the_bound_naming_the_partition_and_how_far_it_got() {
        let (a, d) = bench("stalled", STALLS_AFTER_3);
        let started = Instant::now();
        let e = a.sha256(PART, None).unwrap_err();
        assert!(
            started.elapsed() < Duration::from_secs(15),
            "took {:?}",
            started.elapsed()
        );
        assert!(e.contains(PART) && e.contains("stalled"), "{e}");
        assert!(e.contains(&format!("3 of {} bytes", TEXT.len())), "{e}");
        let reads = std::fs::read_to_string(d.join("reads")).unwrap();
        assert_eq!(reads.lines().count(), 2, "one try and one retry");
    }

    #[test]
    fn a_stall_is_retried_once_and_the_second_read_may_finish() {
        let (a, d) = bench(
            "retry",
            &format!("if [ \"$n\" -eq 1 ]; then {STALLS_AFTER_3}; else {PRINTS}; fi"),
        );
        let want = sum_of_text(&d);
        assert_eq!(a.sha256(PART, None).unwrap(), want);
    }

    #[test]
    fn a_failed_read_is_not_retried() {
        let (a, d) = bench("failed", "exit 3");
        let e = a.sha256(PART, None).unwrap_err();
        assert!(e.contains("did not finish"), "{e}");
        assert_eq!(
            std::fs::read_to_string(d.join("reads"))
                .unwrap()
                .lines()
                .count(),
            1
        );
    }

    #[test]
    fn a_stalled_copy_fails_in_the_bound_and_a_finished_one_is_whole() {
        let (a, d) = bench("copy-stalled", STALLS_AFTER_3);
        let e = a.copy_to(PART, &d.join("out.img")).unwrap_err();
        assert!(
            e.contains(PART) && e.contains(&format!("3 of {} bytes", TEXT.len())),
            "{e}"
        );
        let (a, d) = bench("copy-whole", PRINTS);
        a.copy_to(PART, &d.join("out.img")).unwrap();
        assert_eq!(std::fs::read_to_string(d.join("out.img")).unwrap(), TEXT);
    }
}
