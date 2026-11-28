// SPDX-License-Identifier: MIT OR Apache-2.0
//! The phone while it runs Android, as the lab reaches it: adb calls, the slot Android runs from, adbd as root, and
//! the sha256 of a partition or file read over adb root. Shared by the install write and the root's boot try, which
//! both read the card from Android before and after they act (item walleye-install-boot-procedure).

use crate::host;
use crate::lab::Lab;
use crate::settings::Settings;
use std::io::Read;
use std::process::{Command, Stdio};
use std::rc::Rc;
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

    /// The phone is on Android (out of fastboot if it is there). adbd runs as it is: a library call over `[connections.android]`
    /// (user = root) asks adbd for root itself, once per Phone (lab-tools 0.53.5).
    pub fn is_up(&self) -> Result<(), String> {
        if self.lab.in_fastboot()? {
            let mut c = Command::new(&self.s.fastboot);
            c.args(["-s", &self.lab.serial()?, "reboot"]);
            host::run(c)?;
        }
        self.lab.android_is_back()
    }

    /// The phone is on Android (out of fastboot if it is there) and adbd answers as root: walleye's own `adb root`, kept for the tries
    /// whose first reads are its own adb calls, with no library Phone before them.
    pub fn is_up_as_root(&self) -> Result<(), String> {
        self.is_up()?;
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

    /// The previous boot's console, read from Android's pstore (read only): adbd answers as root first ([`Android::is_up_as_root`], which waits for it - an `adb root` and a
    /// fixed sleep raced adbd, review of builder/ramoops-console 2026-11-28), then [`crate::rw_write::PSTORE_COMMAND`]. The serial is redacted. A pstore closed to adb is an
    /// error, never an empty console. A phone left in fastboot is rebooted to Android first (`is_up`). Shared by the rw write and the display try, which both read it before any other reboot.
    pub fn previous_console(&self) -> Result<String, String> {
        self.is_up_as_root()?;
        let text = self.adb(&["shell", crate::rw_write::PSTORE_COMMAND])?;
        if text.contains("Permission denied") {
            return Err("the pstore is closed to adb (no root)".to_string());
        }
        Ok(crate::touch::redact(&text))
    }

    /// The sha256 of `blocks` blocks of a file or partition on the phone (all of it when None), read as root over adb and hashed ON the phone (`dd | sha256sum`: one line crosses usb).
    /// The read is bounded (`ReadBound`): a stalled stream ends it, once retried, naming the partition (no byte count: only the digest comes back). A `dd` that failed is read from its own `dd=N` line, never taken for a short read.
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

    /// The sha256 of what a read command on the phone writes, hashed ON the phone (`read | sha256sum`): only the digest line crosses
    /// the usb link, so the adb stream's speed no longer sets the time (`what` names the read in an error; `bytes` is what it reads,
    /// which sets the time it has). The page cache is emptied first: a read must come from the media.
    fn sha256_of_read(&self, read: &str, what: &str, bytes: u64) -> Result<String, ReadError> {
        self.adb(&["shell", DROP_CACHES])
            .map_err(ReadError::Failed)?;
        let mut adb = Command::new(&self.s.adb)
            .args([
                "-s",
                &self.lab.serial().map_err(ReadError::Failed)?,
                "exec-out",
                &format!("{{ {{ {read}; echo dd=$? >&2; }} | sha256sum; }} 2>&1"),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| ReadError::Failed(format!("cannot run {}: {e}", self.s.adb)))?;
        let limit = self.bound.limit(bytes);
        let started = Instant::now();
        let status = loop {
            match adb.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() >= limit => {
                    let _ = adb.kill();
                    let _ = adb.wait();
                    return Err(ReadError::Stalled(format!(
                        "the read of {what} stalled: no sum after the {} s bound ({bytes} bytes to hash on the phone; adb killed)",
                        limit.as_secs()
                    )));
                }
                Ok(None) => std::thread::sleep(POLL),
                Err(e) => return Err(ReadError::Failed(format!("adb: {e}"))),
            }
        };
        let mut said = String::new(); // one line of 64 digits: the pipe holds it whole
        if let Some(mut out) = adb.stdout.take() {
            let _ = out.read_to_string(&mut said);
        }
        if !status.success() {
            return Err(ReadError::Failed(format!(
                "the read of {what} did not finish (adb {status})"
            )));
        }
        // exec-out carries no exit status: dd's own, echoed to the stream, is the proof the whole read happened
        if !said.lines().any(|l| l.trim() == "dd=0") {
            return Err(ReadError::Failed(format!(
                "the read of {what} on the phone did not end with dd=0: {said:?}"
            )));
        }
        said.split_whitespace()
            .find(|w| w.len() == 64 && w.bytes().all(|b| b.is_ascii_hexdigit()))
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| ReadError::Failed(format!("the phone gave no sum for {what}: {said:?}")))
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
                "#!/bin/sh\nshift 2\ncase \"$1\" in\n exec-out) case \"$2\" in blockdev*) echo {size}; exit 0;; esac; echo x >> {d}/reads\n  printf %s \"$2\" > {d}/cmd\n  n=$(wc -l < {d}/reads)\n  {body}\n  ;;\nesac\nexit 0\n",
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
    /// What the phone prints when it hashes TEXT itself: `sha256sum`'s one line.
    const HASHES: &str =
        "echo dd=0; printf '%s' 'boot_a as the stand-in phone has it\n' | sha256sum";
    /// A phone whose dd failed: sha256sum still prints the digest of what it got (here nothing).
    const DD_FAILED: &str = "echo dd=1; printf '' | sha256sum";
    const STALLS_AFTER_3: &str = "printf abc; exec sleep 600";

    #[test]
    fn a_whole_read_gives_the_sum_the_phone_printed() {
        let (a, d) = bench("whole", HASHES);
        let want = sum_of_text(&d);
        assert_eq!(a.sha256(PART, None).unwrap(), want);
    }

    #[test]
    fn a_stalled_read_fails_in_the_bound_naming_the_partition() {
        let (a, d) = bench("stalled", STALLS_AFTER_3);
        let started = Instant::now();
        let e = a.sha256(PART, None).unwrap_err();
        assert!(
            started.elapsed() < Duration::from_secs(15),
            "took {:?}",
            started.elapsed()
        );
        assert!(e.contains(PART) && e.contains("stalled"), "{e}");
        let reads = std::fs::read_to_string(d.join("reads")).unwrap();
        assert_eq!(reads.lines().count(), 2, "one try and one retry");
    }

    #[test]
    fn a_stall_is_retried_once_and_the_second_read_may_finish() {
        let (a, d) = bench(
            "retry",
            &format!("if [ \"$n\" -eq 1 ]; then {STALLS_AFTER_3}; else {HASHES}; fi"),
        );
        let want = sum_of_text(&d);
        assert_eq!(a.sha256(PART, None).unwrap(), want);
    }

    #[test]
    fn the_command_is_dd_of_the_partition_into_sha256sum_on_the_phone() {
        let (a, d) = bench("command", HASHES);
        a.sha256(PART, None).unwrap();
        let sent = std::fs::read_to_string(d.join("cmd")).unwrap();
        assert!(
            sent.contains(&format!("dd if={PART} bs=4096")) && sent.contains("| sha256sum"),
            "{sent}"
        );
        a.sha256(PART, Some(7)).unwrap();
        let sent = std::fs::read_to_string(d.join("cmd")).unwrap();
        assert!(sent.contains("count=7"), "{sent}");
    }

    #[test]
    fn a_dd_that_failed_is_a_failed_read_not_a_sum_of_nothing() {
        let (a, d) = bench("dd-failed", DD_FAILED);
        let e = a.sha256(PART, None).unwrap_err();
        assert!(e.contains("dd=0") && e.contains(PART), "{e}");
        assert_eq!(
            std::fs::read_to_string(d.join("reads"))
                .unwrap()
                .lines()
                .count(),
            1,
            "not retried"
        );
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
