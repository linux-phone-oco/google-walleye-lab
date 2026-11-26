// SPDX-License-Identifier: MIT OR Apache-2.0
//! Install stage 1 as ONE command (item walleye-install-write-procedure; decision 2026-11-20-walleye-install-stage-1-go): the Alpine
//! image written from Android to the system partition of the slot Android is NOT on, read back and compared. Each method is one
//! step of procedures/install-rootfs-on-the-pixel-2.rs, returns what that step runs and fails with what it saw.
//!
//! What it writes: one `dd` to `by_name/system_a` or `by_name/system_b` - the slot Android is not on, read at the run (fastboot's
//! `current-slot` equal to adb's `ro.boot.slot_suffix`, never typed) - and nothing else: no boot, no set_active, no GPT, no erase.
//! It refuses on any doubt: the read-back of that slot missing or not whole, the partition read fresh not equal to it, the slot
//! unread or unequal in the two readings, the image not the one `device.toml` declares, a partition name that is not system_a or
//! system_b. The way back: Android's own slot is never touched, so leaving the written slot as it is leaves a phone that runs as
//! before; putting the backup back is a dd, said when a stop needs it and run by hand.

use crate::android::{plain_path, Android, BLOCK};
use crate::lab::Lab;
use crate::settings::Settings;
use crate::{host, watch};
use lpoco_attended::phone_on;
use lpoco_attended::steps::adb_write_partition;
use lpoco_attended::Step;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

type Done = Result<(), String>;

/// The flags of the one write, and of the rehearsal's probe: the same, so the probe proves this dd.
const DD_FLAGS: &str = "conv=fsync,notrunc";

/// The rehearsal's probe: this file in the stage folder, this many blocks of the staged image.
pub const PROBE_NAME: &str = "walleye-dd-probe";
pub const PROBE_BLOCKS: u64 = 16;

/// The slot the phone's fastboot says it is on, from `fastboot getvar current-slot` (it answers on standard error).
pub fn parse_current_slot(text: &str) -> Result<String, String> {
    let value = text
        .lines()
        .find_map(|l| {
            l.split_once("current-slot:")
                .map(|(_, v)| v.trim().to_string())
        })
        .ok_or_else(|| format!("fastboot told no current-slot: {}", host::last_line(text)))?;
    match value.as_str() {
        "a" | "b" => Ok(value),
        other => Err(format!(
            "fastboot's current-slot is {other:?}, neither a nor b"
        )),
    }
}

/// The slot the write goes to: the one Android is not on. Anything but `a` or `b` is a doubt, and a doubt writes nothing.
pub fn other_slot(current: &str) -> Result<&'static str, String> {
    match current {
        "a" => Ok("b"),
        "b" => Ok("a"),
        other => Err(format!(
            "the slot Android runs from is {other:?}, neither a nor b: nothing is written"
        )),
    }
}

/// The two readings of the slot Android runs from agree: adb's `ro.boot.slot_suffix` (`_b`) and fastboot's `current-slot` (`b`).
pub fn slots_agree(adb_suffix: &str, fastboot_slot: &str) -> Result<String, String> {
    let adb_slot = adb_suffix.trim().strip_prefix('_').unwrap_or("");
    let fastboot_slot = fastboot_slot.trim();
    for (who, slot) in [("adb", adb_slot), ("fastboot", fastboot_slot)] {
        if slot != "a" && slot != "b" {
            return Err(format!("{who} told no slot a or b (read {slot:?} from {adb_suffix:?} / {fastboot_slot:?}): nothing is written"));
        }
    }
    if adb_slot == fastboot_slot {
        Ok(fastboot_slot.to_string())
    } else {
        Err(format!(
            "the two readings of the slot disagree: adb says {adb_slot}, fastboot says {fastboot_slot}: nothing is written"
        ))
    }
}

/// The allow-list handed to the library's `adb_write_partition`: the only partitions this program writes, `system_a` and
/// `system_b`, by name (the library refuses the one of the slot the phone runs from, so only the other slot's is written).
/// Source: decision 2026-11-20-walleye-install-stage-1-go (the Alpine image goes to the system partition of the other slot).
pub const WRITABLE: [&str; 2] = ["system_a", "system_b"];

/// The rehearsal's dd: the write's flags, from a staged file to a file in the stage folder - never to a partition.
pub fn probe_command(staged: &str, probe: &str, blocks: u64) -> Result<String, String> {
    if plain_path(probe)?.starts_with("/dev/") {
        return Err(format!("{probe} is a device: the probe writes a file"));
    }
    Ok(format!(
        "dd if={} of={probe} bs={BLOCK} count={blocks} {DD_FLAGS}",
        plain_path(staged)?
    ))
}

/// The sha256 a `sha256sums` file records for `file` (`HASH  file` lines).
pub fn recorded_sha256(sums: &str, file: &str) -> Result<String, String> {
    let hash = sums
        .lines()
        .filter_map(|l| l.split_once(char::is_whitespace))
        .find(|(_, name)| name.trim().trim_start_matches('*') == file)
        .map(|(h, _)| h.to_lowercase())
        .ok_or_else(|| format!("no line for {file} in the backup's sha256sums"))?;
    if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(hash)
    } else {
        Err(format!(
            "the backup's line for {file} holds no sha256: {hash:?}"
        ))
    }
}

/// The image can be written and read back whole: a whole number of blocks, and no longer than the partition. Its blocks.
pub fn image_fits(image_bytes: u64, partition_bytes: u64) -> Result<u64, String> {
    if image_bytes == 0 || !image_bytes.is_multiple_of(BLOCK) {
        return Err(format!(
            "the image is {image_bytes} bytes: not a whole number of {BLOCK}-byte blocks, so it cannot be read back whole"
        ));
    }
    if image_bytes > partition_bytes {
        return Err(format!(
            "the image ({image_bytes} bytes) is longer than the partition ({partition_bytes} bytes)"
        ));
    }
    Ok(image_bytes / BLOCK)
}

/// What the author is told before the write: the numbers are the settings', never typed twice.
pub fn expect(s: &Settings) -> String {
    let mib = std::fs::metadata(&s.image)
        .map(|m| (m.len() / (1024 * 1024)).to_string())
        .unwrap_or_else(|_| "?".into());
    format!(
        "One command, about 25 minutes, the phone stays on Android (adb root; it visits fastboot once to read its slot). It writes the Alpine image ({mib} MiB, sha256 {sha}...) with ONE dd to the system partition of the slot Android is NOT on (read from the phone: fastboot current-slot, equal to adb's slot suffix) and nothing else: no boot partition, no slot switch, no partition table, no erase. \
         Before the write that partition is read and must equal the sha256 of its stored backup; after it, the partition is read back and must hash to the image's sha256, and every other partition of both slots is read again and must equal its backup. \
         Any doubt stops it before the write. The way back: Android's own slot is never touched, so the phone runs as before even if the write is cut off; the stored backup goes back with one dd, which the run prints if it stops.",
        sha = s.image_sha256.chars().take(12).collect::<String>()
    )
}

/// What the author is told before the rehearsal (the keeper runs it; nothing in it writes a partition).
pub fn expect_rehearsal(s: &Settings) -> String {
    format!(
        "The rehearsal of the install write, about 10 minutes, the phone stays on Android (it visits fastboot once to read its slot). It runs every step of the write except the write: the battery, the backup, the image, the slot read in fastboot and in adb (they must agree), adb as root, the image staged and its copy hashed, the target partition read now and compared with its backup, and a dd probe - the write's own dd flags copy {} blocks of the image to a small file in {} and read back by sha256, then the file is removed. NO partition is written; the staged copy is removed at the end.",
        PROBE_BLOCKS, s.stage_dir
    )
}

#[derive(Clone)]
pub struct Install {
    s: Rc<Settings>,
    lab: Lab,
    android: Android,
    /// the slot fastboot said Android runs from, once read
    current: Rc<RefCell<Option<String>>>,
}

impl Install {
    pub fn new(s: Settings) -> Install {
        let s = Rc::new(s);
        let lab = Lab::new(s.clone());
        Install {
            android: Android::new(s.clone(), lab.clone()),
            lab,
            s,
            current: Rc::new(RefCell::new(None)),
        }
    }

    fn slot(&self) -> Result<String, String> {
        self.current.borrow().clone().ok_or_else(|| {
            "the slot Android runs from is not read yet: nothing is written".to_string()
        })
    }

    /// The partition the write goes to: `system_` and the slot that is not Android's.
    fn target(&self) -> Result<String, String> {
        let target = format!("system_{}", other_slot(&self.slot()?)?);
        if WRITABLE.contains(&target.as_str()) {
            Ok(target)
        } else {
            Err(format!("{target:?} is not on the allow-list {WRITABLE:?}"))
        }
    }

    fn image_bytes(&self) -> Result<u64, String> {
        std::fs::metadata(&self.s.image)
            .map(|m| m.len())
            .map_err(|e| format!("{}: {e}", self.s.image.display()))
    }

    /// Where the image is staged on the phone.
    fn staged(&self) -> Result<String, String> {
        let name = self
            .s
            .image
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("the image has no file name")?;
        plain_path(&format!(
            "{}/{name}",
            self.s.stage_dir.trim_end_matches('/')
        ))
        .map(str::to_string)
    }

    fn by_name_of(&self, name: &str) -> String {
        format!("{}/{name}", self.s.by_name.trim_end_matches('/'))
    }

    /// One adb call for this lab's phone; what it said, or why it failed.
    fn adb(&self, args: &[&str]) -> Result<String, String> {
        self.android.adb(args)
    }

    fn adb_suffix(&self) -> Result<String, String> {
        self.android.slot_suffix()
    }

    /// The sha256 of `blocks` blocks of a file or partition on the phone (all of it when None), read as root over adb and hashed here.
    fn phone_sha256(&self, path: &str, blocks: Option<u64>) -> Result<String, String> {
        self.android.sha256(path, blocks)
    }

    /// What the backup records for `name.img`, and how big the file is.
    fn backup_of(&self, name: &str) -> Result<(String, u64), String> {
        let sums = self.s.backup.join("sha256sums");
        let text =
            std::fs::read_to_string(&sums).map_err(|e| format!("{}: {e}", sums.display()))?;
        let file = format!("{name}.img");
        let sha = recorded_sha256(&text, &file)?;
        let bytes = std::fs::metadata(self.s.backup.join(&file))
            .map_err(|e| format!("backup of {name}: {e}"))?
            .len();
        Ok((sha, bytes))
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The image is the one device.toml declares (its sha256), a whole number of blocks, and fits the system partition of the backup.
    pub fn the_image_is_the_declared_one(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            let (_, partition_bytes) = u.backup_of("system_a")?;
            let (_, other_bytes) = u.backup_of("system_b")?;
            image_fits(u.image_bytes()?, partition_bytes.min(other_bytes)).map(|_| ())
        }
    }

    /// The backup folder is a whole read-back of both slots (the lab's own check: sizes as the phone measured, sha256 matching).
    pub fn the_backup_is_whole(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        let dir = self.s.backup.to_string_lossy().into_owned();
        move || {
            lab.script("slot-readback-check.sh", &[&dir], &[])
                .map(|_| ())
        }
    }

    /// Say what to expect, before the first act on the phone.
    pub fn author_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect(&self.s);
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    /// Say what to expect of the rehearsal.
    pub fn rehearsal_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect_rehearsal(&self.s);
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    /// The phone is in fastboot (the shared step); the first act on the phone, so the watcher owns the way back.
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            watch::phone_is_touched();
            lab.phone_is_in_fastboot()
        }
    }

    /// Fastboot says which slot Android runs from; kept for the steps that follow.
    pub fn the_slot_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut c = Command::new(&u.s.fastboot);
            c.args(["-s", &u.lab.serial()?, "getvar", "current-slot"]);
            let slot = parse_current_slot(&host::run(c)?.text)?;
            other_slot(&slot)?;
            *u.current.borrow_mut() = Some(slot);
            Ok(())
        }
    }

    /// The phone is on Android again (out of fastboot if it is there) and adbd answers as root.
    pub fn the_phone_is_on_android_as_root(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.android.is_up_as_root()
    }

    /// adb's slot suffix and fastboot's current slot are the same slot.
    pub fn the_slots_agree(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || slots_agree(&u.adb_suffix()?, &u.slot()?).map(|_| ())
    }

    /// The target partition, read fresh, is what the backup recorded: the backup is the way back of THIS slot as it is now.
    pub fn the_backup_is_the_slot_as_it_is_now(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let target = u.target()?;
            let (recorded, _) = u.backup_of(&target)?;
            let fresh = u.phone_sha256(&u.by_name_of(&target), None)?;
            if fresh == recorded {
                Ok(())
            } else {
                Err(format!(
                    "{target} read now is {fresh}, the backup records {recorded}: the backup is not this partition, nothing is written"
                ))
            }
        }
    }

    /// The image is copied to the phone and the copy hashes to the image's sha256.
    pub fn the_image_is_staged(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            let staged = u.staged()?;
            u.adb(&["push", &u.s.image.to_string_lossy(), &staged])?;
            let blocks = u.image_bytes()? / BLOCK;
            let got = u.phone_sha256(&staged, Some(blocks))?;
            if got == u.s.image_sha256.trim().to_lowercase() {
                Ok(())
            } else {
                Err(format!(
                    "the staged copy is {got}, the image is {}: nothing is written",
                    u.s.image_sha256
                ))
            }
        }
    }

    /// THE write: one dd of the staged copy to the target partition. The gate is checked again right here, whatever ran before:
    /// the slot read twice, the partition's name, the image's sha256.
    pub fn the_image_is_written(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.write_gates()?.execute()
    }

    /// The gates of the write, checked right before it, and the write itself as the library's step: `adb_write_partition` (lpoco-attended
    /// 0.46.1, on the Android connection: device.toml [connections.android], adb, user root), which refuses a partition off [`WRITABLE`] or of the slot the phone runs from, a staged image that is not whole blocks or
    /// not the declared sha256, writes with one `dd` and fails when the partition read back does not hash to the image's sha256.
    fn write_gates(&self) -> Result<Step, String> {
        let slot = self.slot()?;
        slots_agree(&self.adb_suffix()?, &slot)?;
        let target = self.target()?;
        host::image_has_sha256(&self.s.image, &self.s.image_sha256)?;
        Ok(adb_write_partition(
            phone_on("android"),
            &WRITABLE,
            &self.s.by_name,
            &target,
            &self.staged()?,
            self.s.image_sha256.trim(),
        ))
    }

    /// The first bytes of the partition, as many as the image has, hash to the image's sha256 (what follows the image is padding).
    pub fn the_partition_reads_back_as_the_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let target = u.target()?;
            let got = u.phone_sha256(&u.by_name_of(&target), Some(u.image_bytes()? / BLOCK))?;
            let want = u.s.image_sha256.trim().to_lowercase();
            if got == want {
                Ok(())
            } else {
                Err(format!(
                    "{target} reads back as {got}, not as the image's {want}"
                ))
            }
        }
    }

    /// Every partition of the slot list but the one written is read again and equals its backup (the slot-readback-check's names).
    pub fn the_other_partitions_are_unchanged(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let target = u.target()?;
            let list = u.s.host_script("slot-parts.list");
            let names =
                std::fs::read_to_string(&list).map_err(|e| format!("{}: {e}", list.display()))?;
            let mut changed = Vec::new();
            let listed: Vec<&str> = names.split_whitespace().collect();
            if !listed.contains(&"system_a") || !listed.contains(&"system_b") {
                return Err(format!(
                    "{} does not list system_a and system_b: nothing would be checked",
                    list.display()
                ));
            }
            for name in names.split_whitespace().filter(|n| *n != target) {
                let (recorded, _) = u.backup_of(name)?;
                let now = u.phone_sha256(&u.by_name_of(name), None)?;
                if now != recorded {
                    changed.push(format!("{name} is {now}, its backup {recorded}"));
                }
            }
            if changed.is_empty() {
                Ok(())
            } else {
                Err(format!("changed since the backup: {}", changed.join("; ")))
            }
        }
    }

    /// The staged copy is removed from the phone.
    pub fn the_stage_is_removed(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.clean_stage()
    }
    /// The staged copy and the dd probe are taken off the phone, and are gone.
    pub fn clean_stage(&self) -> Done {
        let staged = self.staged()?;
        let probe = self.probe_path()?;
        self.adb(&["shell", &format!("rm -f {staged} {probe}")])?;
        self.adb(&["shell", &format!("test ! -e {staged} && test ! -e {probe}")])
            .map(|_| ())
    }

    fn probe_path(&self) -> Result<String, String> {
        plain_path(&format!(
            "{}/{PROBE_NAME}",
            self.s.stage_dir.trim_end_matches('/')
        ))
        .map(str::to_string)
    }

    /// The rehearsal's dd proof: the real write's dd, flags and block size, copies the first blocks of the staged image to a small
    /// file in the stage folder, which reads back as the same bytes of the image; the file is removed. No partition is named.
    pub fn the_dd_works_on_a_file(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let (staged, probe) = (u.staged()?, u.probe_path()?);
            let blocks = PROBE_BLOCKS.min(u.image_bytes()? / BLOCK);
            let command = probe_command(&staged, &probe, blocks)?;
            let proof = (|| {
                u.adb(&["shell", &command])?;
                let got = u.phone_sha256(&probe, Some(blocks))?;
                let want = host::sha256_of_head(&u.s.image, blocks * BLOCK)?;
                if got == want {
                    Ok(())
                } else {
                    Err(format!(
                        "the dd probe file reads back as {got}, the image's same bytes are {want}: this dd does not copy as the write needs"
                    ))
                }
            })();
            let _ = u.adb(&["shell", &format!("rm -f {probe}")]);
            proof
        }
    }

    /// The rehearsal's way back: the stage and the probe off the phone, the phone on Android. It wrote no partition.
    pub fn rehearsal_way_back(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            if std::env::var_os(watch::CHILD).is_some() {
                return Ok(());
            }
            let result = u.lab.way_back();
            let _ = u.clean_stage();
            println!(
                "  the rehearsal wrote no partition: only a file in {}, now removed",
                u.s.stage_dir
            );
            result
        }
    }

    /// The phone is back on Android (the shared step).
    pub fn android_is_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.android_is_back()
    }

    /// The way back, the last step: the staged copy is taken off the phone when it can be, and the phone ends on Android whatever
    /// stopped the run. The slot Android runs from was never written; the stored backup of the other one goes back by hand
    /// (a dd by hand). The watching parent runs it after the child ends, whatever ended it.
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            if std::env::var_os(watch::CHILD).is_some() {
                return Ok(());
            }
            let result = u.lab.way_back();
            if let Ok(staged) = u.staged() {
                let _ = u.adb(&["shell", &format!("rm -f {staged}")]);
            }
            println!(
                "  the way back: Android's slot was never written. To put the other slot's system partition back as it was: dd if=BACKUP/system_X.img (in {}) to {}/system_X, X = the slot Android is not on",
                u.s.backup.display(),
                u.s.by_name
            );
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::executable;
    use crate::settings::Try;
    use std::path::Path;

    const IMAGE: usize = 8192;
    const PART: usize = 20480;

    #[test]
    fn the_slot_fastboot_reads_is_a_or_b_and_the_other_one_is_the_target() {
        assert_eq!(
            parse_current_slot("current-slot: b\nFinished. Total time: 0.001s").unwrap(),
            "b"
        );
        assert_eq!(
            parse_current_slot("(bootloader) current-slot: a").unwrap(),
            "a"
        );
        assert!(parse_current_slot("current-slot: \nFinished").is_err());
        assert!(parse_current_slot("current-slot: c").is_err());
        assert!(parse_current_slot("no such variable").is_err());
        assert_eq!(other_slot("a").unwrap(), "b");
        assert_eq!(other_slot("b").unwrap(), "a");
        for doubt in ["", "c", "_b", "ab", "unknown"] {
            assert!(other_slot(doubt).is_err(), "{doubt:?}");
        }
    }

    #[test]
    fn the_two_readings_of_the_slot_must_agree() {
        assert_eq!(slots_agree("_b\n", "b").unwrap(), "b");
        assert_eq!(slots_agree("_a", "a").unwrap(), "a");
        assert!(slots_agree("_a", "b").unwrap_err().contains("disagree"));
        for bad in ["", "_", "_c", "b b"] {
            assert!(slots_agree(bad, "b").is_err(), "{bad:?}");
        }
    }

    #[test]
    fn only_system_a_and_system_b_are_ever_on_the_allow_list() {
        assert_eq!(WRITABLE, ["system_a", "system_b"]);
        for name in ["boot_a", "vendor_b", "system", "userdata", "sda", ""] {
            assert!(!WRITABLE.contains(&name), "{name:?}");
        }
    }

    #[test]
    fn the_target_is_the_other_slots_system_partition() {
        for (android_on, target) in [("a", "system_b"), ("b", "system_a")] {
            let b = bench("target", android_on, &format!("_{android_on}"), true);
            *b.install.current.borrow_mut() = Some(android_on.into());
            assert_eq!(b.install.target().unwrap(), target);
        }
    }

    #[test]
    fn the_recorded_sha256_is_found_by_file_name_and_a_missing_one_is_named() {
        let h = "a".repeat(64);
        let sums = format!("{h}  system_a.img\n{}  system_b.img\n", "b".repeat(64));
        assert_eq!(recorded_sha256(&sums, "system_a.img").unwrap(), h);
        assert!(recorded_sha256(&sums, "boot_a.img")
            .unwrap_err()
            .contains("boot_a.img"));
        assert!(recorded_sha256("xyz  system_a.img\n", "system_a.img").is_err());
    }

    #[test]
    fn an_image_of_whole_blocks_no_longer_than_the_partition_fits() {
        assert_eq!(image_fits(1_073_741_824, 2_684_354_560).unwrap(), 262_144);
        assert_eq!(image_fits(4096, 4096).unwrap(), 1);
        assert!(image_fits(4097, 8192).is_err());
        assert!(image_fits(0, 8192).is_err());
        assert!(image_fits(8192, 4096).unwrap_err().contains("longer"));
    }

    // ---- the stand-in phone: an adb that runs the phone's commands here, on files, and a fastboot that names the slot ----

    struct Bench {
        d: std::path::PathBuf,
        install: Install,
    }

    fn sha256_of_bytes(b: &[u8], d: &Path) -> String {
        let f = d.join("tmp-sha");
        std::fs::write(&f, b).unwrap();
        let o = Command::new("sha256sum").arg(&f).output().unwrap();
        String::from_utf8_lossy(&o.stdout)
            .split_whitespace()
            .next()
            .unwrap()
            .to_string()
    }

    /// A lab folder with an image, a loop file per slot as the partitions, a backup of the partitions, an adb and a fastboot stand-in.
    fn bench(name: &str, fastboot_slot: &str, adb_suffix: &str, backup_matches: bool) -> Bench {
        let d = std::env::temp_dir().join(format!("wt-install-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let by_name = d.join("by-name");
        let backup = d.join("backup");
        let stage = d.join("stage");
        for dir in [
            &by_name,
            &backup,
            &stage,
            &d.join("images"),
            &d.join("tests/host"),
        ] {
            std::fs::create_dir_all(dir).unwrap();
        }
        let image: Vec<u8> = (0..IMAGE).map(|i| (i % 251) as u8).collect();
        std::fs::write(d.join("images/rootfs.ext4"), &image).unwrap();
        let mut sums = String::new();
        for slot in ["a", "b"] {
            let part: Vec<u8> = (0..PART)
                .map(|i| ((i * 7 + slot.len() + slot.as_bytes()[0] as usize) % 253) as u8)
                .collect();
            std::fs::write(by_name.join(format!("system_{slot}")), &part).unwrap();
            let kept = if backup_matches || slot == "x" {
                part.clone()
            } else {
                vec![0; PART]
            };
            std::fs::write(backup.join(format!("system_{slot}.img")), &kept).unwrap();
            sums.push_str(&format!(
                "{}  system_{slot}.img\n",
                sha256_of_bytes(&kept, &d)
            ));
            let other: Vec<u8> = (0..4096)
                .map(|i| ((i + slot.as_bytes()[0] as usize) % 249) as u8)
                .collect();
            std::fs::write(by_name.join(format!("boot_{slot}")), &other).unwrap();
            std::fs::write(backup.join(format!("boot_{slot}.img")), &other).unwrap();
            sums.push_str(&format!(
                "{}  boot_{slot}.img\n",
                sha256_of_bytes(&other, &d)
            ));
        }
        std::fs::write(backup.join("sha256sums"), sums).unwrap();
        std::fs::write(
            d.join("tests/host/slot-parts.list"),
            "boot_a\nboot_b\nsystem_a\nsystem_b\n",
        )
        .unwrap();
        std::fs::write(d.join("device.toml"), "[connection]\nserial = \"SER1\"\n").unwrap();
        // the phone: `adb -s SER shell|exec-out CMD` runs CMD here; push copies; getprop answers the slot suffix; id -u says root
        let adb = d.join("adb");
        executable(
            &adb,
            &format!(
                "#!/bin/sh\necho \"$*\" >> \"{d}/adb.calls\"\nshift 2\ncase \"$1\" in\n shell|exec-out) shift\n  case \"$*\" in 'id -u'|'sync; echo 3 > /proc/sys/vm/drop_caches') echo 0; exit 0;; 'getprop ro.boot.slot_suffix') echo '{suffix}'; exit 0;; esac\n  exec sh -c \"$*\";;\n push) cp \"$2\" \"$3\";;\n root) echo restarting adbd as root;;\n get-state) echo device;;\nesac\n",
                d = d.display(),
                suffix = adb_suffix
            ),
        );
        let fastboot = d.join("fastboot");
        executable(
            &fastboot,
            &format!("#!/bin/sh\ncase \"$*\" in *'getvar current-slot'*) echo 'current-slot: {fastboot_slot}' >&2; echo 'Finished.' >&2;; *) echo \"$*\" >> \"{d}/fastboot.other\";; esac\n", d = d.display()),
        );
        let (adb, fastboot) = (
            adb.to_string_lossy().into_owned(),
            fastboot.to_string_lossy().into_owned(),
        );
        let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
        let mut s = Settings::stand_in_with(
            Try::Install,
            &[
                ("tools.adb", leak(adb)),
                ("tools.fastboot", leak(fastboot)),
                ("images.dir", leak(d.to_string_lossy().into_owned())),
                ("images.alpine-rootfs.path", "images/rootfs.ext4"),
                ("backup", leak(backup.to_string_lossy().into_owned())),
                ("stage_dir", leak(stage.to_string_lossy().into_owned())),
                ("by_name", leak(by_name.to_string_lossy().into_owned())),
            ],
        );
        s.lab = d.clone();
        s.image_sha256 = sha256_of_bytes(&image, &d);
        Bench {
            install: Install::new(s),
            d,
        }
    }

    fn partition(b: &Bench, name: &str) -> Vec<u8> {
        std::fs::read(b.d.join("by-name").join(name)).unwrap()
    }

    /// The write step as a test runs it: walleye's own gates (`write_gates`, which builds the library's `adb_write_partition` for the
    /// target), then the dd the library's step sends, stood in on the bench's adb. The library's step itself reaches the phone through
    /// devlab, which a unit test has none of (its `procedure()` is called once per process): the step is tested upstream, against
    /// captured phone text and a stand-in devlab (lab-tools attended/tests/install.rs, lpoco-attended-v0.46.1). The dd flags below are
    /// that step's script (attended/src/install.rs `write_script`: bs=4096, conv=fsync,notrunc).
    fn write_as_the_library_does(i: &Install) -> Done {
        i.write_gates()?;
        let command = format!(
            "dd if={} of={}/{} bs={BLOCK} count={} {DD_FLAGS}",
            i.staged()?,
            i.s.by_name,
            i.target()?,
            i.image_bytes()? / BLOCK
        );
        i.adb(&["shell", &command]).map(|_| ())
    }

    fn run_all(b: &Bench) -> Done {
        let i = &b.install;
        (i.the_image_is_the_declared_one())()?;
        (i.the_slot_is_read())()?;
        (i.the_image_is_staged())()?;
        (i.the_slots_agree())()?;
        (i.the_backup_is_the_slot_as_it_is_now())()?;
        write_as_the_library_does(i)?;
        (i.the_partition_reads_back_as_the_image())()?;
        (i.the_other_partitions_are_unchanged())()?;
        (i.the_stage_is_removed())()
    }

    #[test]
    fn the_image_goes_to_the_slot_android_is_not_on_and_reads_back_as_itself() {
        let b = bench("ok", "b", "_b", true);
        let (a0, b0) = (partition(&b, "system_a"), partition(&b, "system_b"));
        let (boot_a0, boot_b0) = (partition(&b, "boot_a"), partition(&b, "boot_b"));
        run_all(&b).unwrap();
        let calls = std::fs::read_to_string(b.d.join("adb.calls")).unwrap();
        let writes: Vec<&str> = calls.lines().filter(|l| l.contains("of=")).collect();
        assert_eq!(writes.len(), 1, "exactly one write: {calls}");
        assert!(
            writes[0].contains(&format!("of={}/system_a ", b.d.join("by-name").display())),
            "{}",
            writes[0]
        );
        let image = std::fs::read(b.d.join("images/rootfs.ext4")).unwrap();
        let a1 = partition(&b, "system_a");
        assert_eq!(
            &a1[..IMAGE],
            &image[..],
            "the image is at the start of system_a"
        );
        assert_eq!(
            &a1[IMAGE..],
            &a0[IMAGE..],
            "the padding after the image is as it was"
        );
        assert_eq!(
            partition(&b, "system_b"),
            b0,
            "Android's own slot is untouched"
        );
        assert_eq!(
            (partition(&b, "boot_a"), partition(&b, "boot_b")),
            (boot_a0, boot_b0)
        );
        assert!(
            std::fs::read_dir(b.d.join("stage"))
                .unwrap()
                .next()
                .is_none(),
            "the staged copy is removed"
        );
    }

    #[test]
    fn android_on_slot_a_sends_the_write_to_system_b() {
        let b = bench("slot-a", "a", "_a", true);
        let a0 = partition(&b, "system_a");
        run_all(&b).unwrap();
        assert_eq!(partition(&b, "system_a"), a0);
        let image = std::fs::read(b.d.join("images/rootfs.ext4")).unwrap();
        assert_eq!(&partition(&b, "system_b")[..IMAGE], &image[..]);
    }

    #[test]
    fn nothing_is_written_when_the_two_readings_of_the_slot_disagree() {
        let b = bench("disagree", "b", "_a", true);
        let before = (partition(&b, "system_a"), partition(&b, "system_b"));
        let e = run_all(&b).unwrap_err();
        assert!(e.contains("disagree"), "{e}");
        assert_eq!(
            (partition(&b, "system_a"), partition(&b, "system_b")),
            before
        );
    }

    #[test]
    fn nothing_is_written_when_the_read_back_is_not_what_the_partition_holds_now() {
        let b = bench("backup", "b", "_b", false);
        let before = partition(&b, "system_a");
        let e = run_all(&b).unwrap_err();
        assert!(e.contains("system_a") && e.contains("backup"), "{e}");
        assert_eq!(partition(&b, "system_a"), before);
        let calls = std::fs::read_to_string(b.d.join("adb.calls")).unwrap_or_default();
        assert!(!calls.contains("of="), "no dd was sent: {calls}");
    }

    #[test]
    fn nothing_is_written_for_an_image_that_is_not_the_declared_one() {
        let b = bench("image", "b", "_b", true);
        std::fs::write(b.d.join("images/rootfs.ext4"), vec![9u8; IMAGE]).unwrap();
        let e = run_all(&b).unwrap_err();
        assert!(e.contains("device.toml records"), "{e}");
        assert!(!std::fs::read_to_string(b.d.join("adb.calls"))
            .unwrap_or_default()
            .contains("of="));
    }

    #[test]
    fn the_write_step_itself_refuses_the_slot_android_runs_from() {
        // the slot read says Android is on a, but the phone reports b at the write: the step stops before any dd
        let b = bench("late", "a", "_b", true);
        *b.install.current.borrow_mut() = Some("a".into());
        (b.install.the_image_is_staged())().unwrap();
        let e = (b.install.the_image_is_written())().unwrap_err();
        assert!(e.contains("disagree") || e.contains("runs from"), "{e}");
        assert!(!std::fs::read_to_string(b.d.join("adb.calls"))
            .unwrap()
            .contains("of="));
    }

    #[test]
    fn a_partition_that_reads_back_wrong_is_a_failure_that_names_both_sums() {
        let b = bench("readback", "b", "_b", true);
        (b.install.the_slot_is_read())().unwrap();
        // nothing written: the partition does not start with the image
        let e = (b.install.the_partition_reads_back_as_the_image())().unwrap_err();
        assert!(e.contains("system_a") && e.contains("image"), "{e}");
    }

    #[test]
    fn a_changed_partition_of_the_other_slot_is_found() {
        let b = bench("changed", "b", "_b", true);
        run_all(&b).unwrap();
        let mut boot = partition(&b, "boot_b");
        boot[0] ^= 0xff;
        std::fs::write(b.d.join("by-name/boot_b"), boot).unwrap();
        let e = (b.install.the_other_partitions_are_unchanged())().unwrap_err();
        assert!(e.contains("boot_b"), "{e}");
    }

    #[test]
    fn the_phone_is_taken_out_of_fastboot_and_adbd_is_asked_for_root() {
        let b = bench("root", "b", "_b", true);
        // the lab's own android check passes; fastboot lists the phone once, then reboots it
        executable(
            &b.d.join("tests/host/idle-android.sh"),
            "#!/bin/sh\necho 'pass idle-android host up'\n",
        );
        let fb = b.d.join("fastboot");
        executable(
            &fb,
            &format!("#!/bin/sh\ncase \"$*\" in *devices*) echo 'SER1 fastboot';; *reboot*) echo reboot >> \"{0}/fastboot.rebooted\";; esac\n", b.d.display()),
        );
        (b.install.the_phone_is_on_android_as_root())().unwrap();
        assert!(
            b.d.join("fastboot.rebooted").exists(),
            "fastboot reboot was sent"
        );
        assert!(
            std::fs::read_to_string(b.d.join("adb.calls"))
                .unwrap()
                .contains("root"),
            "adb root was asked"
        );
    }

    fn rehearse(b: &Bench) -> Done {
        let i = &b.install;
        (i.the_image_is_the_declared_one())()?;
        (i.the_slot_is_read())()?;
        (i.the_image_is_staged())()?;
        (i.the_slots_agree())()?;
        (i.the_backup_is_the_slot_as_it_is_now())()?;
        (i.the_dd_works_on_a_file())()?;
        (i.the_stage_is_removed())()
    }

    #[test]
    fn the_rehearsal_writes_a_file_in_the_stage_folder_and_no_partition() {
        let b = bench("rehearse", "b", "_b", true);
        let before: Vec<Vec<u8>> = ["system_a", "system_b", "boot_a", "boot_b"]
            .iter()
            .map(|n| partition(&b, n))
            .collect();
        rehearse(&b).unwrap();
        let after: Vec<Vec<u8>> = ["system_a", "system_b", "boot_a", "boot_b"]
            .iter()
            .map(|n| partition(&b, n))
            .collect();
        assert_eq!(before, after, "no partition changed");
        let calls = std::fs::read_to_string(b.d.join("adb.calls")).unwrap();
        let writes: Vec<&str> = calls.lines().filter(|l| l.contains("of=")).collect();
        assert_eq!(writes.len(), 1, "one dd, the probe: {calls}");
        assert!(
            writes[0].contains(&format!("of={}/", b.d.join("stage").display()))
                && !writes[0].contains("by-name"),
            "{}",
            writes[0]
        );
        assert!(
            writes[0].contains("conv=fsync,notrunc"),
            "the real write's flags: {}",
            writes[0]
        );
        assert!(
            std::fs::read_dir(b.d.join("stage"))
                .unwrap()
                .next()
                .is_none(),
            "stage and probe removed"
        );
    }

    #[test]
    fn a_probe_that_reads_back_wrong_fails_the_rehearsal() {
        let b = bench("probe-bad", "b", "_b", true);
        // an adb that corrupts the probe file after the dd wrote it
        let real = b.install.s.adb.clone();
        let bad = b.d.join("adb-bad");
        executable(
            &bad,
            &format!("#!/bin/sh\n\"{real}\" \"$@\"\ncase \"$*\" in *of=*walleye-dd-probe*) f=$(echo \"$*\" | sed -n 's/.*of=\\([^ ]*\\).*/\\1/p'); printf Z | dd of=\"$f\" bs=1 count=1 conv=notrunc 2>/dev/null;; esac\n"),
        );
        let mut s = (*b.install.s).clone();
        s.adb = bad.to_string_lossy().into_owned();
        let bad_install = Install::new(s);
        (bad_install.the_image_is_staged())().unwrap();
        let e = (bad_install.the_dd_works_on_a_file())().unwrap_err();
        assert!(e.contains("probe"), "{e}");
    }

    #[test]
    fn the_stage_and_the_probe_are_cleaned_off_the_phone() {
        let b = bench("clean", "b", "_b", true);
        std::fs::write(b.d.join("stage/walleye-dd-probe"), "x").unwrap();
        std::fs::write(b.d.join("stage/rootfs.ext4"), "x").unwrap();
        b.install.clean_stage().unwrap();
        assert!(std::fs::read_dir(b.d.join("stage"))
            .unwrap()
            .next()
            .is_none());
    }

    #[test]
    fn the_author_is_told_the_command_what_it_writes_where_and_the_way_back() {
        let b = bench("told", "b", "_b", true);
        let t = expect(&b.install.s);
        for word in ["system", "slot", "Android", "backup", "sha256"] {
            assert!(t.contains(word), "{word}: {t}");
        }
    }
}
