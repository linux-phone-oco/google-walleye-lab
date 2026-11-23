// SPDX-License-Identifier: MIT OR Apache-2.0
//! The phone while it runs Android, as the lab reaches it: adb calls, the slot Android runs from, adbd as root, and
//! the sha256 of a partition or file read over adb root. Shared by the install write and the root's boot try, which
//! both read the card from Android before and after they act (item walleye-install-boot-procedure).

use crate::host;
use crate::lab::Lab;
use crate::settings::Settings;
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

#[derive(Clone)]
pub struct Android {
    s: Rc<Settings>,
    lab: Lab,
}

impl Android {
    pub fn new(s: Rc<Settings>, lab: Lab) -> Android {
        Android { s, lab }
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
    pub fn sha256(&self, path: &str, blocks: Option<u64>) -> Result<String, String> {
        let path = plain_path(path)?;
        let read = match blocks {
            Some(n) => format!("dd if={path} bs={BLOCK} count={n} 2>/dev/null"),
            None => format!("dd if={path} bs={BLOCK} 2>/dev/null"),
        };
        self.sha256_of_read(&read, path)
    }

    /// A partition on the phone copied whole into `dest`, read as root over adb (`exec-out dd`): what a flash is about to overwrite and nothing else holds (research walleye-stage2-checks-should-they-exist, row 9).
    pub fn copy_to(&self, path: &str, dest: &std::path::Path) -> Result<(), String> {
        let path = plain_path(path)?;
        let file = std::fs::File::create(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
        let status = Command::new(&self.s.adb)
            .args([
                "-s",
                &self.lab.serial()?,
                "exec-out",
                &format!("dd if={path} bs={BLOCK} 2>/dev/null"),
            ])
            .stdout(Stdio::from(file))
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("cannot run {}: {e}", self.s.adb))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("the copy of {path} did not finish (adb {status})"))
        }
    }

    /// The sha256 of what a read command on the phone writes to standard output (`what` names it in an error). The page cache
    /// is emptied first: a read must come from the media.
    fn sha256_of_read(&self, read: &str, what: &str) -> Result<String, String> {
        self.adb(&["shell", DROP_CACHES])?;
        let mut adb = Command::new(&self.s.adb)
            .args(["-s", &self.lab.serial()?, "exec-out", read])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot run {}: {e}", self.s.adb))?;
        let feed = adb.stdout.take().ok_or("adb has no output to read")?;
        let sum = Command::new("sha256sum").stdin(Stdio::from(feed)).output();
        let status = adb.wait().map_err(|e| e.to_string())?;
        let sum = sum.map_err(|e| format!("cannot run sha256sum: {e}"))?;
        if !status.success() {
            return Err(format!("the read of {what} did not finish (adb {status})"));
        }
        String::from_utf8_lossy(&sum.stdout)
            .split_whitespace()
            .next()
            .filter(|h| h.len() == 64)
            .map(str::to_string)
            .ok_or_else(|| format!("sha256sum gave no sum for {what}"))
    }
}
