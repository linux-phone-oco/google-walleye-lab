// SPDX-License-Identifier: MIT OR Apache-2.0
//! The rw write's plumbing (item walleye-rw-write-plumbing; image: walleye-rw-write-image, `[images.ufs-root-booted-slot-production-rw]`): what the
//! procedure that boots the rw-write image once and writes a bounded file into the root partition asks of the phone, and how it judges what comes back.
//! Everything here is a function of text and numbers, tested without a phone; the steps that run it on the phone (ssh, adb, fastboot) compose these
//! on the pattern of [`crate::rootfs_boot::RootfsBoot`].
//!
//! The gates (before anything is written): the image's command line carries the word `walleye.rw` once and names exactly one root
//! ([`cmdline_carries_the_word`]); the root is the system partition of the slot Android is NOT on ([`target_is_allowed`]); the size of the write is
//! one number of the settings, 1 to [`MAX_WRITE_MIB`] ([`write_bytes`]). The measures: the card read from Android before and after
//! ([`card_changed_only_in_the_root`]: every disk's GPT equal, the root partition CHANGED - a write that did not reach the partition is a failure);
//! `/proc/mounts` after the boot ([`mounts_are_read_write`]); the file read back ([`readback_is_whole`]); the filesystem checked, not
//! repaired ([`fsck_is_clean`]); the kernel log ([`dmesg_is_clean`]).
//! The write itself ([`write_command`]) is ONE command on ONE fixed file of the root, zeros, bounded, synced: no other path, no other size.
//!
//! Sources: each judge's expectations cite where they came from in its doc: the init's own lines (build/initramfs-root-rw-write/init), the kernel
//! (fs/ext4/super.c, `EXT4-fs error`), e2fsck's own line as Android printed it (research/walleye-first-fastboot-boot/2026-11-18/dmesg-ufs.txt),
//! and captured runs of the root boot (tests fixtures/rootfs-boots-run3-*.txt, run4-*.txt). No rw boot has happened yet: the rw side of
//! `/proc/mounts` is run 4's captured text with its two root lines replaced, and the tests say so.

//! NOT KNOWN YET (no rw boot, no fsck on a mounted root has been captured): the rw lines of `/proc/mounts` (the positive case is run 4 with its two root lines replaced by what the init is built to make), the `mounts-rw` and `EXT4-fs error` log lines (typed from the init and the kernel source), and the stdout of `fsck.ext4 -n` on the mounted root (the judge is strict: the line Android printed on an unmounted partition, and `rc=0`; if the first run refuses a good root the run, not the judge, tells what it prints). The first real rw run replaces these by captured text. The procedure must judge the mounts BEFORE it writes (the write is not tied to the mounts judge by a type), and the init says `mounts-rw` BEFORE it mounts: the line shows which init ran, the mounts judge shows the mount.

use crate::rootfs_boot::{
    disk_letters, root_for, sections, InitSaw, RootfsBoot, INIT_TAG, ROOT_KEY,
};
use crate::ufs::Probe;

type Done = Result<(), String>;

/// The word on the image's command line that makes its init mount the root read-write (build/initramfs-root-rw-write/init).
pub const WORD: &str = "walleye.rw";

/// The line the rw init says to the kernel log when it mounts the root read-write (its `say "walleye-init: mounts-rw (..."`).
pub const INIT_RW_LINE: &str = "walleye-init: mounts-rw (";

/// The one file the write makes, in the root of the card's partition. No other path is ever written.
pub const TEST_FILE: &str = "/walleye-rw-write.test";

/// The most MiB one write may put in the file (the settings' `write_mib` is 1 to this).
pub const MAX_WRITE_MIB: u64 = 64;

/// The bytes of the write for a setting of `mib` MiB; a size outside 1 to [`MAX_WRITE_MIB`] is refused.
pub fn write_bytes(mib: u64) -> Result<u64, String> {
    if (1..=MAX_WRITE_MIB).contains(&mib) {
        Ok(mib * 1024 * 1024)
    } else {
        Err(format!(
            "the write is {mib} MiB: 1 to {MAX_WRITE_MIB} (write_mib of the procedure's table)"
        ))
    }
}

/// What the author is told before the run: the numbers come from the settings.
pub fn expect(mib: u64, luns: u64) -> String {
    format!("One command, about 10 minutes: the phone boots our UFS-on image once, from memory, with its root, the Alpine system partition of the slot Android is NOT on, mounted READ-WRITE (no overlay). The lab logs in over usb-network by ssh, writes ONE file of {mib} MiB of zeros into that root ({TEST_FILE}), syncs, reads it back, checks the filesystem without repairing it and reads the kernel log. The partition tables of the card's {luns} disks and the boot chain are hashed before and after and must be equal; the root partition must have changed. The glass stays dark. The phone returns to Android by itself; if it is not on Android after 4 minutes, hold the power button about 15 seconds.")
}

/// The command line the manifest's recipe gives the image `image` (`[images.NAME.recipe].cmdline`).
pub fn recipe_cmdline(manifest: &str, image: &str) -> Result<String, String> {
    let m: toml::Value = toml::from_str(manifest).map_err(|e| format!("device.toml: {e}"))?;
    let img = m
        .get("images")
        .and_then(|i| i.get(image))
        .ok_or_else(|| format!("device.toml has no image {image}"))?;
    img.get("recipe")
        .and_then(|r| r.get("cmdline"))
        .and_then(|c| c.as_str())
        .map(str::to_string)
        .ok_or_else(|| format!("[images.{image}] has no recipe with a cmdline"))
}

/// The image is built from its recipe and nothing else: the command line in the header of the file to boot is the recipe's, word for word.
pub fn header_equals_recipe(header: &str, recipe: &str) -> Result<(), String> {
    let (h, r): (Vec<&str>, Vec<&str>) = (
        header.split_whitespace().collect(),
        recipe.split_whitespace().collect(),
    );
    if h == r {
        return Ok(());
    }
    let extra: Vec<&&str> = h.iter().filter(|w| !r.contains(w)).collect();
    let missing: Vec<&&str> = r.iter().filter(|w| !h.contains(w)).collect();
    Err(format!(
        "the image's header command line is not its recipe's: words the recipe lacks {extra:?}, words the header lacks {missing:?}{}",
        if extra.is_empty() && missing.is_empty() { " (same words, other order or count)" } else { "" }
    ))
}

/// The gate on the image: its command line (the `cmdline` of `lpoco-bringup image info`, or the header) carries the word exactly once and
/// names exactly one root, `system_a` or `system_b`. Returns the root. An image without the word is the read-only image: refused here.
pub fn cmdline_carries_the_word(cmdline: &str) -> Result<String, String> {
    let words: Vec<&str> = cmdline.split_whitespace().collect();
    match words.iter().filter(|w| **w == WORD).count() {
        1 => {}
        0 => {
            return Err(format!(
                "the image's command line has no {WORD}: it is the read-only image"
            ))
        }
        n => {
            return Err(format!(
                "the image's command line has {n} words {WORD}, expected one"
            ))
        }
    }
    let roots: Vec<&str> = words
        .iter()
        .filter_map(|w| w.strip_prefix(ROOT_KEY))
        .collect();
    match roots.as_slice() {
        ["system_a"] => Ok("system_a".into()),
        ["system_b"] => Ok("system_b".into()),
        [] => Err(format!("the command line names no {ROOT_KEY}")),
        other => Err(format!(
            "the command line names {other:?} as the root: expected exactly system_a or system_b"
        )),
    }
}

/// The command line of an `image info` text (`cmdline = "..."`).
pub fn cmdline_of_info(info: &str) -> Result<String, String> {
    info.lines()
        .find_map(|l| {
            l.strip_prefix("cmdline = \"")
                .and_then(|r| r.strip_suffix('"'))
        })
        .map(str::to_string)
        .ok_or_else(|| "no cmdline line in the image info".to_string())
}

/// The gate on the target: the partition the write goes to is the system partition of the slot Android is NOT on (`_a`/`a`/`_b`/`b` is the slot
/// read), and is the one the image names. Anything else - the running slot, a name outside system_a/system_b - is refused.
pub fn target_is_allowed(current_slot: &str, image_root: &str) -> Result<String, String> {
    let other = root_for(current_slot)?;
    if image_root != other {
        return Err(format!(
            "the image names {image_root} as its root but the slot Android runs from is {}: the write may go only to {other}",
            current_slot.trim()
        ));
    }
    Ok(other)
}

/// The `rc=N` line a command of this module prints is `rc=0`.
fn returned_zero(text: &str, what: &str) -> Done {
    match text.lines().rev().find_map(|l| l.strip_prefix("rc=")) {
        Some("0") => Ok(()),
        Some(n) => Err(format!("{what} returned rc={n}")),
        None => Err(format!("{what} printed no rc= line")),
    }
}

/// The dd of the write came back `rc=0`.
pub fn write_is_done(text: &str) -> Done {
    returned_zero(text, "the write")
}

/// The one command that writes: `write_mib` MiB of zeros into [`TEST_FILE`], flushed to the partition (`conv=fsync`, then `sync`). It prints `rc=N` of the dd.
pub fn write_command(mib: u64) -> Result<String, String> {
    write_bytes(mib)?;
    Ok(write_command_for(TEST_FILE, mib))
}

fn write_command_for(path: &str, mib: u64) -> String {
    format!("dd if=/dev/zero of={path} bs=1M count={mib} conv=fsync 2>&1; echo rc=$?; sync")
}

/// The read-back: the page cache is dropped first (the bytes are read from the partition, not from memory), then the size of the file and how many bytes of it are not zero.
pub fn readback_command() -> String {
    readback_command_for(TEST_FILE)
}

fn readback_command_for(path: &str) -> String {
    format!("sync; echo 3 > /proc/sys/vm/drop_caches; echo ---size; stat -c %s {path}; echo ---nonzero; tr -d '\\000' < {path} | wc -c; echo ---end")
}

/// The read-back says the file is exactly `mib` MiB and all zeros.
pub fn readback_is_whole(text: &str, mib: u64) -> Done {
    let want = write_bytes(mib)?;
    let parts = sections(text);
    let get = |name: &str| -> Result<u64, String> {
        let body = parts
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, b)| b.trim())
            .ok_or_else(|| format!("the read-back has no section {name}"))?;
        body.parse::<u64>()
            .map_err(|_| format!("section {name} is {body:?}, not a number"))
    };
    let size = get("size")?;
    if size != want {
        return Err(format!("the file is {size} bytes, the write was {want}"));
    }
    let nonzero = get("nonzero")?;
    if nonzero != 0 {
        return Err(format!(
            "{nonzero} bytes of the file read back are not zero"
        ));
    }
    Ok(())
}

/// The check of the filesystem: read only (`-n`: it answers no to every repair). `device` is a `/dev/sdXN` of the card, nothing else.
pub fn fsck_command(device: &str) -> Result<String, String> {
    let ok = device.strip_prefix("/dev/sd").is_some_and(|r| {
        let mut c = r.chars();
        c.next().is_some_and(|l| l.is_ascii_lowercase()) && {
            let n = c.as_str();
            !n.is_empty() && n.chars().all(|d| d.is_ascii_digit())
        }
    });
    if !ok {
        return Err(format!("{device} is not a /dev/sdXN of the card"));
    }
    Ok(format!("fsck.ext4 -n {device} 2>&1; echo rc=$?"))
}

/// e2fsck's answer on the root: the line `DEVICE: clean, N/M files, A/B blocks` (the form Android's own e2fsck prints in the kernel log, research
/// walleye-first-fastboot-boot/2026-11-18/dmesg-ufs.txt line 42) and none of its refusals.
pub fn fsck_is_clean(text: &str, device: &str) -> Done {
    returned_zero(text, "fsck")?;
    for bad in [
        "UNEXPECTED INCONSISTENCY",
        "FILE SYSTEM WAS MODIFIED",
        "Filesystem still has errors",
        "contains a file system with errors",
    ] {
        if text.contains(bad) {
            return Err(format!("fsck said: {bad}"));
        }
    }
    let clean = text.lines().any(|l| {
        l.split_once(&format!("{device}: clean, "))
            .is_some_and(|(_, rest)| {
                rest.contains(" files, ") && rest.trim_end().ends_with(" blocks")
            })
    });
    if clean {
        Ok(())
    } else {
        Err(format!(
            "no `{device}: clean, N/M files, A/B blocks` line in fsck's answer"
        ))
    }
}

/// `/proc/mounts` after the boot with the word: the root is the card's partition `device` itself (`/dev/sdXN`), ext4, `rw`, with none of
/// `ro`, `discard`, `norecovery`/`noload`; nothing is an overlay or `/lower`; `/.walleye` is a tmpfs (the petter's files are not on the card).
/// Source: the init (build/initramfs-root-rw-write/init: `mount -t ext4 -o rw,nodiscard`, `mount -t tmpfs ... /newroot/.walleye`).
pub fn mounts_are_read_write(proc_mounts: &str, device: &str) -> Done {
    let field = |l: &str, n: usize| l.split_whitespace().nth(n).unwrap_or("").to_string();
    let at =
        |mp: &str| -> Vec<&str> { proc_mounts.lines().filter(|l| field(l, 1) == mp).collect() };
    let opts = |l: &str| -> Vec<String> { field(l, 3).split(',').map(str::to_string).collect() };
    if !at("/lower").is_empty() {
        return Err("/lower is mounted: this is the read-only image's root".into());
    }
    let roots = at("/");
    let [root] = roots.as_slice() else {
        return Err(format!("{} mounts at /, expected one", roots.len()));
    };
    if field(root, 2) == "overlay" {
        return Err(format!(
            "the root is an overlay, not the card's partition: {root}"
        ));
    }
    if field(root, 0) != device || field(root, 2) != "ext4" {
        return Err(format!("the root is not ext4 on {device}: {root}"));
    }
    let o = opts(root);
    if !o.iter().any(|w| w == "rw") {
        return Err(format!("the root is not mounted rw: {root}"));
    }
    for refused in ["ro", "discard", "norecovery", "noload"] {
        if o.iter().any(|w| w == refused) {
            return Err(format!("the root is mounted with {refused}: {root}"));
        }
    }
    let w = at("/.walleye");
    if !w.iter().any(|l| field(l, 2) == "tmpfs") {
        return Err("/.walleye is not a tmpfs: the petter's files would be on the card".into());
    }
    Ok(())
}

/// The kernel log after the run: the init said it mounted the root rw (`root` is `system_a`/`system_b`), and the log has no purge, oops, UFS
/// failure or `EXT4-fs error` (the kernel's own line, fs/ext4/super.c `__ext4_error`: `EXT4-fs error (device %s): ...`).
pub fn dmesg_is_clean(dmesg: &str, root: &str) -> Done {
    if !dmesg
        .lines()
        .any(|l| l.contains(INIT_RW_LINE) && l.contains(&format!("({root},")))
    {
        return Err(format!("the init did not say `{INIT_RW_LINE}{root}, ...`: the root was not mounted by the rw init"));
    }
    if let Some(t) = dmesg.lines().find(|l| l.contains("EXT4-fs error")) {
        return Err(format!("the kernel log says: {t}"));
    }
    if let Some(t) = Probe::of(dmesg).trouble.first() {
        return Err(format!("the kernel log says: {t}"));
    }
    if let Some(t) = lpoco_attended::steps::trouble_lines(dmesg).first() {
        return Err(format!("the kernel log says: {t}"));
    }
    Ok(())
}

/// The file of the run's folder the stop's console is kept in (the pstore's `console-ramoops*`, read from Android).
pub const PSTORE_FILE: &str = "pstore-console.txt";

/// The one read-only command that reads the previous boot's console on Android (as root: `adb root` first, which restarts adbd and not the phone).
/// `ls` first so an absent file is said, then every `console-ramoops*` whole.
pub const PSTORE_COMMAND: &str = "ls -la /sys/fs/pstore; cat /sys/fs/pstore/console-ramoops*";

/// The last line the rw init says before it hands over (its `say "handing over to the root's /sbin/init"`, build/initramfs-root-rw-write/init).
const HANDING_OVER: &str = "handing over to the root's /sbin/init";

/// What the previous boot's console (`console-ramoops`) says the rw init got to. Research walleye-rw-run-1-refused-by-booted-slot: run 1's refusal line
/// was lost when Android rebooted, so a stop is read from the pstore BEFORE any other reboot.
#[derive(Debug, PartialEq, Clone)]
pub enum StopSaid {
    /// the init refused: the reason it gave (`refused: WHICH ...`)
    Refused(String),
    /// the init mounted the root rw and said it hands over
    HandedOver,
    /// the init mounted the root rw and said nothing after: it stopped (or its last line was lost) after the mount
    MountedNotHandedOver,
    /// the init started and said nothing more: it stopped before the mount
    StartedNotMounted,
    /// no line of the init in the console (an empty or absent pstore, or another boot's text)
    Nothing,
}

impl StopSaid {
    /// What the console says, in one line for the run's log.
    pub fn said(&self) -> String {
        match self {
            StopSaid::Refused(why) => format!("the init REFUSED: {why}"),
            StopSaid::HandedOver => "the init mounted the root rw and handed over to the root's init".into(),
            StopSaid::MountedNotHandedOver => "the init mounted the root rw and said nothing after: it did not hand over (or the last line was lost)".into(),
            StopSaid::StartedNotMounted => "the init started and never mounted the root".into(),
            StopSaid::Nothing => "NOT KNOWN YET: the console has no line of the init (the pstore is empty or holds another boot)".into(),
        }
    }
}

/// Which stop the previous boot's console tells: the init's own lines ([`InitSaw::of`] for the start and the refusal, [`INIT_RW_LINE`] for the rw mount,
/// the hand-over line). A refusal wins over everything else it said before.
pub fn the_stop_in_the_console(console: &str) -> StopSaid {
    let saw = InitSaw::of(console);
    if let Some(why) = saw.refused {
        return StopSaid::Refused(why);
    }
    let said = |what: &str| {
        console
            .lines()
            .any(|l| l.contains(INIT_TAG) && l.contains(what))
    };
    if said(INIT_RW_LINE) {
        return if said(HANDING_OVER) {
            StopSaid::HandedOver
        } else {
            StopSaid::MountedNotHandedOver
        };
    }
    if saw.root.is_some() {
        StopSaid::StartedNotMounted
    } else {
        StopSaid::Nothing
    }
}

/// What the root partition's digest must do between the two card reads: nothing (the run stopped before the boot), whatever (it booted and mounted rw, which alone
/// rewrites the superblock and the journal, and made `/.walleye`), or change (the write was made: the read-back says it landed, this says the partition moved).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RootRule {
    Equal,
    Any,
    Changed,
}

/// The rule the run's folder gives: `write.txt` (the write was sent) means Changed, `read.txt` (the phone was booted and read) means Any, neither means Equal.
pub fn root_rule(out: &std::path::Path) -> RootRule {
    if out.join("write.txt").exists() {
        RootRule::Changed
    } else if out.join("read.txt").exists() {
        RootRule::Any
    } else {
        RootRule::Equal
    }
}

/// [`card_judge`] for a run that wrote: the root must have CHANGED.
pub fn card_changed_only_in_the_root(before: &str, after: &str, luns: u64) -> Done {
    card_judge(before, after, luns, RootRule::Changed)
}

/// The card read from Android before and after (`gpt sdX DIGEST` for each of the `luns` disks, then one `root NAME DIGEST`): every disk's table
/// equal, the same root named, the root's digest as the rule says. The lines must be exactly those, in that order, with 64-hex digests.
pub fn card_judge(before: &str, after: &str, luns: u64, rule: RootRule) -> Done {
    let mut want: Vec<String> = disk_letters(luns)?
        .iter()
        .map(|l| format!("gpt sd{l}"))
        .collect();
    want.push("root".into());
    let parse = |which: &str, text: &str| -> Result<Vec<Vec<String>>, String> {
        let rows: Vec<Vec<String>> = text
            .lines()
            .map(|l| l.split_whitespace().map(str::to_string).collect())
            .collect();
        if rows.len() != want.len() {
            return Err(format!(
                "the card read {which} has {} lines, expected {} ({} disks and the root)",
                rows.len(),
                want.len(),
                luns
            ));
        }
        for (row, key) in rows.iter().zip(&want) {
            let digest = row.last().map(String::as_str).unwrap_or("");
            let shape = if key == "root" {
                row.len() == 3 && row[0] == "root"
            } else {
                row.len() == 3 && format!("{} {}", row[0], row[1]) == *key
            };
            if !shape || digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(format!(
                    "the card read {which} has a line that is not `{key} DIGEST` (64 hex): {}",
                    row.join(" ")
                ));
            }
        }
        Ok(rows)
    };
    let (b, a) = (parse("before", before)?, parse("after", after)?);
    let n = b.len() - 1;
    for (x, y) in b[..n].iter().zip(&a[..n]) {
        if x != y {
            return Err(format!(
                "the partition table changed: {} then {}",
                x.join(" "),
                y.join(" ")
            ));
        }
    }
    let (x, y) = (&b[n], &a[n]);
    if x[1] != y[1] {
        return Err(format!(
            "the root partition is named {} before and {} after",
            x[1], y[1]
        ));
    }
    match (rule, x[2] == y[2]) {
        (RootRule::Changed, true) => Err(
            "the root partition is byte for byte what it was: the write did not reach the card"
                .into(),
        ),
        (RootRule::Equal, false) => {
            Err("the root partition changed although the run stopped before the boot".into())
        }
        _ => Ok(()),
    }
}

/// What the author is told before the rehearsal.
pub fn expect_rehearsal(luns: u64) -> String {
    format!("The rehearsal of the rw write, about 5 minutes: every step of the write except the boot and everything after it, run for real - the card read from Android (the partition tables of its {luns} disks and the root partition hashed), the image and its command line (it must carry {WORD} once), the guard with fastboot, the slots' flags, the boot chain read back and compared with the stored set. Nothing is booted and nothing is written to the card; the phone is in fastboot for a minute and ends on Android. The author's go is not needed and not read.")
}

/// What fsck's answer says, for the run's log: whether it printed a refusal marker, its `rc=`, and whether the `clean` line is there. NEVER a stop: `fsck.ext4 -n` on a root that is mounted
/// rw is known to warn and to count differences of a live filesystem, and what it prints here has not been captured (NOT KNOWN YET); the whole answer is kept in fsck.txt.
pub fn fsck_report(text: &str, device: &str) -> String {
    let marker = [
        "UNEXPECTED INCONSISTENCY",
        "FILE SYSTEM WAS MODIFIED",
        "Filesystem still has errors",
        "contains a file system with errors",
    ]
    .into_iter()
    .find(|m| text.contains(m));
    let rc = text
        .lines()
        .rev()
        .find_map(|l| l.strip_prefix("rc="))
        .unwrap_or("none");
    let clean = text
        .lines()
        .any(|l| l.contains(&format!("{device}: clean, ")));
    format!(
        "fsck.ext4 -n on the mounted root: rc={rc}, {}, {}; NOT KNOWN YET what it prints on a mounted root: the whole answer is in fsck.txt",
        if clean { "the clean line is there" } else { "no clean line" },
        marker.map_or("no refusal marker".to_string(), |m| format!("it says: {m}"))
    )
}

/// The steps of the rw write that are its own; the rest are the root boot's (`RootfsBoot::rw_write` makes the try).
impl RootfsBoot {
    /// The card's device the init named (`sda22`), from the kernel log read at the login.
    fn the_target_device(&self) -> Result<String, String> {
        let dmesg = self.section_of_read("dmesg")?;
        InitSaw::of(&dmesg)
            .device
            .ok_or_else(|| "the init's log names no device".to_string())
    }

    /// `/proc/mounts` says the root is the card's partition mounted rw, no overlay (the facts the init's own mount line makes).
    pub fn the_root_is_mounted_read_write(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let device = u.the_target_device()?;
            mounts_are_read_write(&u.section_of_read("mounts")?, &format!("/dev/{device}"))?;
            println!("  /dev/{device} is the root, ext4 rw, no overlay, no discard; /.walleye is a tmpfs");
            Ok(())
        }
    }

    /// The one bounded write: `write_mib` MiB of zeros into the one fixed file, synced; its answer is kept.
    pub fn the_bounded_write_is_made(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let said = u.on_phone(&write_command(u.settings().write_mib)?, 180)?;
            u.kept("write.txt", &said)?;
            println!("  {}", said.trim().replace('\n', " | "));
            write_is_done(&said)
        }
    }

    /// The file read back from the partition: its size and that every byte is zero.
    pub fn the_file_reads_back_whole(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let said = u.on_phone(&readback_command(), 120)?;
            u.kept("readback.txt", &said)?;
            readback_is_whole(&said, u.settings().write_mib)
        }
    }

    /// The filesystem checked, never repaired (`fsck.ext4 -n`); the whole answer is kept in fsck.txt. NOT KNOWN YET what it prints on a mounted root: the clean line is said when it is there.
    pub fn the_filesystem_is_checked(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let device = format!("/dev/{}", u.the_target_device()?);
            let said = u.on_phone(&fsck_command(&device)?, 300)?;
            u.kept("fsck.txt", &said)?;
            println!("{}", fsck_report(&said, &device));
            Ok(())
        }
    }

    /// The kernel log after the write: the rw init's line, no purge, oops, UFS failure or `EXT4-fs error`. Read again from the phone, kept in dmesg-after.txt.
    pub fn the_kernel_log_after_the_write_is_clean(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let dmesg = u.on_phone("dmesg", 60)?;
            u.kept("dmesg-after.txt", &dmesg)?;
            dmesg_is_clean(&dmesg, &u.the_root()?)
        }
    }

    /// The test file is removed and synced, so the root the phone leaves holds nothing of the run's.
    pub fn the_test_file_is_removed(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let said = u.on_phone(&format!("rm -f {TEST_FILE}; r=$?; sync; [ ! -e {TEST_FILE} ] && echo rc=$r || echo rc=gone-not"), 120)?;
            u.kept("remove.txt", &said)?;
            returned_zero(&said, "the removal")
        }
    }
}

#[cfg(test)]
mod tests {
    /// The text `unpack_bootimg` printed for the image of the run 4 stop, 2477276882987c8e...b54f (kept in fixtures/, 2026-11-24).
    const IMAGE_2477_INFO: &str = include_str!("../fixtures/rw-image-2477-unpack-info.txt");
    fn info_cmdline(info: &str) -> &str {
        info.lines()
            .find_map(|l| l.strip_prefix("command line args: "))
            .unwrap()
    }
    fn manifest() -> String {
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../device.toml")).unwrap()
    }
    /// Item walleye-rw-image-header-equals-recipe: run 4 booted 2477...b54f, whose header kept walleye.booted-slot, which its recipe lacks;
    /// check 2 read the sha256 but not the header. Fails on 2477...b54f.
    #[test]
    fn the_header_of_2477_is_not_its_recipes_command_line() {
        let recipe = recipe_cmdline(&manifest(), "ufs-root-booted-slot-production-rw").unwrap();
        assert!(!recipe.is_empty());
        let e = header_equals_recipe(info_cmdline(IMAGE_2477_INFO), &recipe).unwrap_err();
        assert!(e.contains("walleye.booted-slot"), "{e}");
    }
    /// The manifest no longer declares 2477 (the proof on the declared image's own header is tests/host/image-header-equals-recipe.sh).
    #[test]
    fn the_manifest_does_not_declare_the_image_whose_header_kept_the_word() {
        let v: toml::Value = toml::from_str(&manifest()).unwrap();
        let sha = v["images"]["ufs-root-booted-slot-production-rw"]["sha256"]
            .as_str()
            .unwrap();
        assert!(!sha.starts_with("2477276882987c8e"), "{sha}");
    }
    #[test]
    fn a_header_that_differs_from_the_recipe_by_a_word_is_refused() {
        let r = "a b walleye.rw";
        assert!(header_equals_recipe("a b walleye.rw", r).is_ok());
        assert!(header_equals_recipe("a b walleye.rw  ", r).is_ok());
        let more = header_equals_recipe("a b x walleye.rw", r).unwrap_err();
        assert!(
            more.contains("the recipe lacks [\"x\"], words the header lacks []"),
            "{more}"
        );
        let less = header_equals_recipe("a walleye.rw", r).unwrap_err();
        assert!(
            less.contains("the recipe lacks [], words the header lacks [\"b\"]"),
            "{less}"
        );
        assert!(header_equals_recipe("b a walleye.rw", r).is_err());
        assert!(header_equals_recipe("", r).is_err());
    }
    #[test]
    fn the_recipe_command_line_is_read_from_the_manifest() {
        let m = "[images.x.recipe]\ncmdline = \"a b\"\n[images.y]\npath = \"p\"\n";
        assert_eq!(recipe_cmdline(m, "x").unwrap(), "a b");
        assert!(recipe_cmdline(m, "y").unwrap_err().contains("no recipe"));
        assert!(recipe_cmdline(m, "z").unwrap_err().contains("no image z"));
    }
    use super::*;
    use crate::rootfs_boot::sections;

    // captured text: run 3 and run 4 of the root boot (read.txt as the lab read it from the phone), the card read of run 3, the rw image's own info
    const RUN4_READ: &str = include_str!("../fixtures/rootfs-boots-run4-read.txt");
    const CARD_BEFORE: &str = include_str!("../fixtures/rootfs-boots-run3-card-before.txt");
    const CARD_AFTER: &str = include_str!("../fixtures/rootfs-boots-run3-card-after.txt");
    const IMAGE_INFO: &str = include_str!("../fixtures/rw-write-image-info.txt");
    // Android's own e2fsck line in the kernel log of the phone (research walleye-first-fastboot-boot/2026-11-18/dmesg-ufs.txt)
    const DMESG_UFS: &str =
        include_str!("../../../research/walleye-first-fastboot-boot/2026-11-18/dmesg-ufs.txt");

    fn section(name: &str) -> String {
        sections(RUN4_READ)
            .into_iter()
            .find(|(n, _)| n == name)
            .unwrap()
            .1
    }

    /// run 4's mounts with its two root lines (the overlay and /lower) replaced by what the rw init makes: NOT printed by a phone, no rw boot has happened.
    fn rw_mounts_derived(device: &str) -> String {
        let mut out: Vec<String> = Vec::new();
        for l in section("mounts").lines() {
            if l.contains(" /lower ") {
                out.push(format!("{device} / ext4 rw,relatime 0 0"));
            } else if l.contains("overlay / overlay") {
                out.push("tmpfs /.walleye tmpfs rw,relatime,size=16384k,mode=755 0 0".into());
            } else {
                out.push(l.to_string());
            }
        }
        out.join("\n")
    }

    #[test]
    fn the_size_of_the_write_is_one_to_the_bound() {
        assert_eq!(write_bytes(1).unwrap(), 1_048_576);
        assert_eq!(write_bytes(MAX_WRITE_MIB).unwrap(), 64 * 1_048_576);
        for bad in [0, MAX_WRITE_MIB + 1, 1_000_000] {
            assert!(write_bytes(bad).is_err(), "{bad}");
            assert!(write_command(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_rw_image_carries_the_word_and_the_ro_image_does_not() {
        let rw = cmdline_of_info(IMAGE_INFO).unwrap();
        assert_eq!(cmdline_carries_the_word(&rw).unwrap(), "system_a");
        // the read-only production image's own command line is the rw one without the word
        let ro = rw.replace(" walleye.rw", "");
        assert!(cmdline_carries_the_word(&ro)
            .unwrap_err()
            .contains("read-only image"));
        assert!(cmdline_carries_the_word(&format!("{rw} walleye.rw"))
            .unwrap_err()
            .contains("2 words"));
        assert!(cmdline_carries_the_word(&format!("{rw} walleye.root=system_b")).is_err());
        assert!(cmdline_carries_the_word("walleye.rw")
            .unwrap_err()
            .contains("no walleye.root="));
        assert!(cmdline_carries_the_word("walleye.rw walleye.root=userdata").is_err());
        assert!(cmdline_of_info("nothing").is_err());
    }

    #[test]
    fn the_target_is_the_other_slots_system_and_nothing_else() {
        assert_eq!(target_is_allowed("_b", "system_a").unwrap(), "system_a");
        assert_eq!(target_is_allowed("a", "system_b").unwrap(), "system_b");
        assert!(target_is_allowed("_b", "system_b")
            .unwrap_err()
            .contains("may go only to system_a"));
        assert!(target_is_allowed("_a", "userdata").is_err());
        assert!(target_is_allowed("c", "system_a").is_err());
        assert!(target_is_allowed("", "system_a").is_err());
    }

    #[test]
    fn the_mounts_of_the_read_only_boot_are_refused_and_the_rw_root_is_taken() {
        // run 4's own text (a phone printed it): the overlay root with /lower ro
        assert!(mounts_are_read_write(&section("mounts"), "/dev/sda22")
            .unwrap_err()
            .contains("/lower is mounted"));
        let rw = rw_mounts_derived("/dev/sda22");
        mounts_are_read_write(&rw, "/dev/sda22").unwrap();
        assert!(mounts_are_read_write(&rw, "/dev/sda21")
            .unwrap_err()
            .contains("not ext4 on /dev/sda21"));
        for (from, to, want) in [
            ("ext4 rw,relatime", "ext4 ro,relatime", "not mounted rw"),
            (
                "ext4 rw,relatime",
                "ext4 rw,relatime,discard",
                "with discard",
            ),
            (
                "ext4 rw,relatime",
                "ext4 rw,relatime,norecovery",
                "with norecovery",
            ),
            ("ext4 rw,relatime", "ext4 rw,relatime,noload", "with noload"),
            ("/dev/sda22 / ext4", "/dev/sda22 / ext2", "not ext4"),
            (
                "tmpfs /.walleye tmpfs",
                "tmpfs /.walleye ext4",
                "not a tmpfs",
            ),
        ] {
            let bad = rw.replace(from, to);
            assert!(
                mounts_are_read_write(&bad, "/dev/sda22")
                    .unwrap_err()
                    .contains(want),
                "{to}"
            );
        }
        let no_walleye = rw
            .lines()
            .filter(|l| !l.contains("/.walleye"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(mounts_are_read_write(&no_walleye, "/dev/sda22")
            .unwrap_err()
            .contains("/.walleye"));
        assert!(mounts_are_read_write("", "/dev/sda22").is_err());
    }

    #[test]
    fn the_kernel_log_of_a_captured_run_is_clean_once_the_rw_init_said_its_line() {
        let dmesg = section("dmesg");
        // run 4 was the read-only image: its log has no rw line
        assert!(dmesg_is_clean(&dmesg, "system_a")
            .unwrap_err()
            .contains("not mounted by the rw init"));
        let said = format!("{dmesg}\n[    9.0] walleye-ufs-root: walleye-init: mounts-rw (system_a, ext4 rw,nodiscard)");
        dmesg_is_clean(&said, "system_a").unwrap();
        assert!(dmesg_is_clean(&said, "system_b").is_err());
        let err = format!("{said}\n[   20.0] EXT4-fs error (device sda22): ext4_lookup:1: inode #2: comm sh: deleted inode referenced");
        assert!(dmesg_is_clean(&err, "system_a")
            .unwrap_err()
            .contains("EXT4-fs error"));
    }

    #[test]
    fn the_card_read_must_change_in_the_root_only() {
        // run 3 read the card before and after the boot: byte for byte equal, so for a WRITE that is a failure (real text, both sides)
        assert!(card_changed_only_in_the_root(CARD_BEFORE, CARD_AFTER, 6)
            .unwrap_err()
            .contains("did not reach the card"));
        // mutations of the captured text: the root line's digest replaced (a changed root), one table's digest replaced
        let root_line = CARD_AFTER.lines().find(|l| l.starts_with("root ")).unwrap();
        let changed = CARD_AFTER.replace(root_line, &format!("root system_a {}", "1".repeat(64)));
        card_changed_only_in_the_root(CARD_BEFORE, &changed, 6).unwrap();
        let gpt = CARD_AFTER
            .lines()
            .find(|l| l.starts_with("gpt sdb "))
            .unwrap();
        let moved = changed.replace(gpt, &format!("gpt sdb {}", "2".repeat(64)));
        assert!(card_changed_only_in_the_root(CARD_BEFORE, &moved, 6)
            .unwrap_err()
            .contains("partition table changed"));
        // another root named; a different keyword on the after side; a short read; no disks; the wrong number of disks; a digest that is not hex
        assert!(card_changed_only_in_the_root(
            CARD_BEFORE,
            &changed.replace("root system_a", "root system_b"),
            6
        )
        .unwrap_err()
        .contains("named"));
        assert!(card_changed_only_in_the_root(
            CARD_BEFORE,
            &changed.replace("root system_a", "foo system_a"),
            6
        )
        .is_err());
        assert!(card_changed_only_in_the_root(CARD_BEFORE, "gpt sda x\n", 6).is_err());
        let only_root = format!("root system_a {}\n", "3".repeat(64));
        let only_root_after = format!("root system_a {}\n", "4".repeat(64));
        assert!(card_changed_only_in_the_root(&only_root, &only_root_after, 6).is_err());
        assert!(card_changed_only_in_the_root(CARD_BEFORE, &changed, 5).is_err());
        assert!(card_changed_only_in_the_root(
            CARD_BEFORE,
            &changed.replace(&"1".repeat(64), "zz"),
            6
        )
        .unwrap_err()
        .contains("64 hex"));
        assert!(card_changed_only_in_the_root("", "", 6).is_err());
        assert!(card_changed_only_in_the_root(CARD_BEFORE, &changed, 0).is_err());
    }

    #[test]
    fn the_returned_codes_of_the_write_and_the_check_are_judged() {
        write_is_done("2+0 records in\n2+0 records out\nrc=0\n").unwrap();
        assert!(write_is_done("rc=1\n").unwrap_err().contains("rc=1"));
        assert!(write_is_done("no code here").is_err());
        let line = DMESG_UFS
            .lines()
            .find(|l| l.contains("e2fsck:") && l.contains(": clean, "))
            .unwrap();
        let device = "/dev/block/platform/soc/1da4000.ufshc/by-name/userdata";
        fsck_is_clean(&format!("{line}\nrc=0"), device).unwrap();
        assert!(fsck_is_clean(&format!("{line}\nrc=4"), device)
            .unwrap_err()
            .contains("rc=4"));
    }

    #[test]
    fn fsck_is_clean_on_the_line_a_phone_printed_and_not_on_a_refusal() {
        let line = DMESG_UFS
            .lines()
            .find(|l| l.contains("e2fsck:") && l.contains(": clean, "))
            .expect("the captured e2fsck line");
        let device = "/dev/block/platform/soc/1da4000.ufshc/by-name/userdata";
        fsck_is_clean(&format!("{line}\nrc=0"), device).unwrap();
        assert!(fsck_is_clean(&format!("{line}\nrc=0"), "/dev/sda22")
            .unwrap_err()
            .contains("no `/dev/sda22: clean"));
        for bad in [
            "UNEXPECTED INCONSISTENCY; RUN fsck MANUALLY.",
            "FILE SYSTEM WAS MODIFIED",
        ] {
            assert!(
                fsck_is_clean(&format!("{line}\n{bad}\nrc=0"), device).is_err(),
                "{bad}"
            );
        }
        assert!(fsck_is_clean(
            &format!("{}\nrc=0", line.replace(": clean, ", ": not clean, ")),
            device
        )
        .is_err());
        assert!(fsck_is_clean("rc=0", device).is_err());
    }

    #[test]
    fn fsck_runs_read_only_on_a_card_partition_only() {
        assert_eq!(
            fsck_command("/dev/sda22").unwrap(),
            "fsck.ext4 -n /dev/sda22 2>&1; echo rc=$?"
        );
        for bad in [
            "",
            "/dev/sda",
            "/dev/sd22",
            "/dev/sdA1",
            "/dev/block/sda22",
            "/dev/sda22; rm",
            "sda22",
            "/dev/mmcblk0p1",
        ] {
            assert!(fsck_command(bad).is_err(), "{bad}");
        }
    }

    /// The write and its read-back, run on this host on a known case: the commands are the phone's, the file is a temporary one.
    fn run(cmd: &str) -> String {
        let o = std::process::Command::new("sh")
            .args(["-c", cmd])
            .output()
            .unwrap();
        String::from_utf8_lossy(&o.stdout).into_owned()
    }

    #[test]
    fn the_write_makes_the_file_and_the_read_back_judges_it() {
        let dir = std::env::temp_dir().join(format!("walleye-rw-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("rw-write.test");
        let path = file.to_str().unwrap();
        let said = run(&write_command_for(path, 2));
        assert!(said.contains("rc=0"), "{said}");
        let back = run(&readback_command_for(path));
        readback_is_whole(&back, 2).unwrap();
        // the same file judged against another size
        assert!(readback_is_whole(&back, 3)
            .unwrap_err()
            .contains("bytes, the write was"));
        // a file with one byte that is not zero
        run(&format!("printf x >> {path}"));
        let dirty = run(&readback_command_for(path));
        assert!(readback_is_whole(&dirty, 2).is_err());
        run(&format!("truncate -s 2097152 {path}; printf '\\001' | dd of={path} bs=1 seek=7 conv=notrunc 2>/dev/null"));
        assert!(readback_is_whole(&run(&readback_command_for(path)), 2)
            .unwrap_err()
            .contains("not zero"));
        std::fs::remove_dir_all(&dir).unwrap();
        // no file: the commands' own answer is not a whole read-back
        assert!(readback_is_whole("", 2).is_err());
        // the commands name the one fixed file and the bound
        assert!(write_command(16)
            .unwrap()
            .contains(&format!("of={TEST_FILE} bs=1M count=16 conv=fsync")));
        assert!(readback_command().contains(TEST_FILE));
    }

    #[test]
    fn the_root_must_do_what_the_run_got_to() {
        let dir = std::env::temp_dir().join(format!("wt-rule-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(root_rule(&dir), RootRule::Equal);
        std::fs::write(dir.join("read.txt"), "x").unwrap();
        assert_eq!(root_rule(&dir), RootRule::Any);
        std::fs::write(dir.join("write.txt"), "x").unwrap();
        assert_eq!(root_rule(&dir), RootRule::Changed);
        std::fs::remove_dir_all(&dir).unwrap();
        // run 3's card read, real text, equal on both sides: right for a run that stopped before the boot or before the write, wrong for one that wrote
        card_judge(CARD_BEFORE, CARD_AFTER, 6, RootRule::Equal).unwrap();
        card_judge(CARD_BEFORE, CARD_AFTER, 6, RootRule::Any).unwrap();
        assert!(card_judge(CARD_BEFORE, CARD_AFTER, 6, RootRule::Changed).is_err());
        let root_line = CARD_AFTER.lines().find(|l| l.starts_with("root ")).unwrap();
        let changed = CARD_AFTER.replace(root_line, &format!("root system_a {}", "1".repeat(64)));
        card_judge(CARD_BEFORE, &changed, 6, RootRule::Any).unwrap();
        card_judge(CARD_BEFORE, &changed, 6, RootRule::Changed).unwrap();
        assert!(card_judge(CARD_BEFORE, &changed, 6, RootRule::Equal)
            .unwrap_err()
            .contains("stopped before the boot"));
        // a table that moved fails under every rule
        let gpt = CARD_AFTER
            .lines()
            .find(|l| l.starts_with("gpt sdc "))
            .unwrap();
        let moved = CARD_AFTER.replace(gpt, &format!("gpt sdc {}", "2".repeat(64)));
        for rule in [RootRule::Equal, RootRule::Any, RootRule::Changed] {
            assert!(card_judge(CARD_BEFORE, &moved, 6, rule).is_err());
        }
    }

    #[test]
    fn fsck_is_reported_never_a_stop() {
        let line = DMESG_UFS
            .lines()
            .find(|l| l.contains("e2fsck:") && l.contains(": clean, "))
            .unwrap();
        let device = "/dev/block/platform/soc/1da4000.ufshc/by-name/userdata";
        assert!(fsck_report(&format!("{line}\nrc=0"), device).contains("the clean line is there"));
        let said = fsck_report("UNEXPECTED INCONSISTENCY; RUN fsck MANUALLY.\nrc=4", device);
        assert!(
            said.contains("rc=4")
                && said.contains("UNEXPECTED INCONSISTENCY")
                && said.contains("no clean line")
                && said.contains("NOT KNOWN YET")
        );
        assert!(fsck_report("", device).contains("rc=none"));
    }

    #[test]
    fn the_removal_judges_the_rm_and_that_the_file_is_gone() {
        let dir = std::env::temp_dir().join(format!("wt-rm-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f");
        let cmd = |p: &str| {
            format!("rm -f {p}; r=$?; sync; [ ! -e {p} ] && echo rc=$r || echo rc=gone-not")
        };
        std::fs::write(&path, "x").unwrap();
        let said = run(&cmd(path.to_str().unwrap()));
        assert!(said.contains("rc=0"), "{said}");
        // a file that cannot be removed (a directory with a file in it) is not 'gone'
        let stuck = dir.join("d");
        std::fs::create_dir_all(stuck.join("x")).unwrap();
        let said = run(&cmd(stuck.to_str().unwrap()));
        assert!(!said.contains("rc=0"), "{said}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
    /// Research walleye-rw-run-1-refused-by-booted-slot (item walleye-rw-run-1-why-no-gadget): the rw write boots the image FROM MEMORY
    /// (`fastboot boot`) while Android's slot stays current, and its root is the OTHER slot's system; the run adds only
    /// androidboot.slot_suffix=<the running slot> ([`crate::rootfs_boot::run_cmdline`]). The init's stage 2 guard
    /// (build/initramfs-root-rw-write/init: `[ "$ROOT" = "system$SLOT" ] || refuse "walleye.booted-slot: ..."`) refuses every such line that
    /// carries walleye.booted-slot - QEMU case R3 "booted-slot with a root that is not the booted slot's" of tests/host/ufs-root-rw-init.sh
    /// boots exactly run 1's line and sees the refusal. So the declared rw image's command line must not carry the word. Fails on
    /// image cb79c975 (run 1's); passes on 1bfb3667, rebuilt without it.
    #[test]
    fn the_rw_image_command_line_passes_the_inits_slot_guard_when_booted_from_memory() {
        let manifest: toml::Value = toml::from_str(
            &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../device.toml"))
                .unwrap(),
        )
        .unwrap();
        let declared = manifest["images"]["ufs-root-booted-slot-production-rw"]["recipe"]
            ["cmdline"]
            .as_str()
            .unwrap();
        let root = cmdline_carries_the_word(declared).unwrap();
        let running = if root == "system_a" { "_b" } else { "_a" };
        let line = crate::rootfs_boot::run_cmdline(declared, running).unwrap();
        assert!(
            !line.split_whitespace().any(|w| w == "walleye.booted-slot"),
            "the run's line [{line}] carries walleye.booted-slot with the root {root} while the running slot is {running}: the init refuses it (system{running} is not {root}) before any mount, no gadget comes up"
        );
    }
    /// Fixtures: the kernel-log lines of the REAL rw init (`^.*\[ts\] walleye-ufs-root:` cut out of the console of its QEMU proof,
    /// tests/host/ufs-root-rw-init.sh at loglevel=7, 2026-11-24: the R3 booted-slot refusal, the R1 run on the declared line, that run without its last
    /// line, an empty file). QEMU, not a phone: no pstore of a refused or handing-over rw boot has been captured (run 1's refusal was lost);
    /// the framing a phone's console-ramoops gives those lines is NOT KNOWN YET and is read when run 2 stops or passes.
    const PSTORE_REFUSED: &str = include_str!("../fixtures/rw-pstore-qemu-refused.txt");
    const PSTORE_HANDING_OVER: &str = include_str!("../fixtures/rw-pstore-qemu-handing-over.txt");
    const PSTORE_MOUNTED: &str =
        include_str!("../fixtures/rw-pstore-qemu-mounted-not-handed-over.txt");
    const PSTORE_EMPTY: &str = include_str!("../fixtures/rw-pstore-qemu-empty.txt");
    /// The phone's own text of a boot of the init that got no further than its start: run 3's kernel log (a phone printed it).
    const PHONE_START_ONLY: &str = "[    1.725738] walleye-ufs-root: start, kernel 7.3.0-00013-g3644090c7f6d-dirty, root=[system_a] running slot suffix=[_b]\n";
    #[test]
    fn the_console_of_a_stop_names_what_the_init_got_to() {
        match the_stop_in_the_console(PSTORE_REFUSED) {
            StopSaid::Refused(why) => assert!(why.starts_with("walleye.booted-slot:"), "{why}"),
            other => panic!("{other:?}"),
        }
        assert_eq!(
            the_stop_in_the_console(PSTORE_HANDING_OVER),
            StopSaid::HandedOver
        );
        assert_eq!(
            the_stop_in_the_console(PSTORE_MOUNTED),
            StopSaid::MountedNotHandedOver
        );
        assert_eq!(
            the_stop_in_the_console(PHONE_START_ONLY),
            StopSaid::StartedNotMounted
        );
        assert_eq!(the_stop_in_the_console(PSTORE_EMPTY), StopSaid::Nothing);
        assert_eq!(
            the_stop_in_the_console("ls: /sys/fs/pstore: No such file or directory\n"),
            StopSaid::Nothing
        );
    }
    #[test]
    fn a_refusal_wins_over_the_lines_before_it_and_each_stop_says_itself() {
        let refused_after_mount = format!(
            "{PSTORE_MOUNTED}[    9.9] walleye-ufs-root: refused: the petter of x is not alive\n"
        );
        assert!(matches!(
            the_stop_in_the_console(&refused_after_mount),
            StopSaid::Refused(w) if w.contains("petter")
        ));
        assert!(the_stop_in_the_console(PSTORE_REFUSED)
            .said()
            .contains("REFUSED: walleye.booted-slot"));
        assert!(StopSaid::Nothing.said().contains("NOT KNOWN YET"));
    }
    #[test]
    fn the_expectation_names_the_numbers() {
        let t = expect(16, 6);
        assert!(
            t.contains("16 MiB")
                && t.contains("6 disks")
                && t.contains("READ-WRITE")
                && t.contains(TEST_FILE)
        );
    }
}
