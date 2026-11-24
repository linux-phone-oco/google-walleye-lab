// SPDX-License-Identifier: MIT OR Apache-2.0
//! The read-only UFS-on boot of our kernel on the Alpine root as ONE command (item walleye-install-boot-procedure; decisions
//! 2026-11-20-walleye-install-stage-1-go, 2026-11-19-walleye-attended-tries-one-command): the UFS-root image (the card's system
//! partition of the slot Android is NOT on, mounted read-only with a tmpfs overlay by the image's own init) booted once from the
//! phone's fastboot, from memory; the lab logs in over usb-network by ssh with its declared key and reads what is mounted and what
//! the kernel said; then the way back. Each method is one step of procedures/rootfs-boots-on-the-pixel-2.rs, returns what that
//! step runs and fails with what it saw.
//!
//! What it refuses before the boot: a slot read two ways that disagree, a root partition that is the slot Android runs from
//! (the init's own guard fails open when the bootloader gives no `androidboot.slot_suffix` on a fastboot boot, so THIS program
//! carries the guard), an image that is not the declared one, a stored backup that is not whole, and an author's go that is not
//! recorded on the item for this image. What it proves after: the card was mounted read-only (`ro`, `noload`, no `discard`), the
//! root is an overlay, the init saw the slot adb saw, the kernel log has no purge or oops, the GPT of every disk, the root
//! partition and the boot chain read from Android are what they were before, and the slots' flags are unchanged.
//! NOTHING is written to the card: no mount rw, no dd to a disk, no fstrim, no purge.

use crate::android::Android;
use crate::install::{other_slot, parse_current_slot, slots_agree};
use crate::lab::Lab;
use crate::settings::Settings;
use crate::touch::redact;
use crate::ufs::{go_decision, now_epoch, Probe};
use crate::{host, watch};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The tag the image's init puts on every line it says to the kernel log.
pub const INIT_TAG: &str = "walleye-ufs-root:";

/// The word on the image's command line that names the partition the root is.
pub const ROOT_KEY: &str = "walleye.root=";

/// The made image's name in the run's folder (never where a queue looks: it has UFS on).
pub const RUN_IMAGE: &str = "NOT-TO-BOOT-run-image.img";

/// The disks of the card as Android names them (`/dev/block/sda` ...): `luns` of them, the letters a, b, c ... (a card has at most 26).
pub fn disk_letters(luns: u64) -> Result<Vec<char>, String> {
    match u8::try_from(luns) {
        Ok(n) if (1..=26).contains(&n) => Ok((b'a'..b'a' + n).map(char::from).collect()),
        _ => Err(format!("the card has {luns} disks: expected 1 to 26")),
    }
}

/// The root partition the image boots for the phone that runs from this slot: the system partition of the OTHER slot. `_b` and `b` both name slot b.
pub fn root_for(current: &str) -> Result<String, String> {
    let slot = current.trim();
    let slot = slot.strip_prefix('_').unwrap_or(slot);
    Ok(format!("system_{}", other_slot(slot)?))
}

/// What the author is told before the boot: the numbers are the settings', never typed twice.
pub fn expect(s: &Settings) -> String {
    format!("One command, about 10 minutes: the phone boots our UFS-on image once, from memory; its root is the Alpine system partition of the slot Android is NOT on, mounted READ-ONLY with an overlay in memory. The lab logs in over usb-network by ssh and reads the mounts and the kernel log; NOTHING is written to the card (the partition tables of its {} disks, the root partition and the boot chain are hashed before and after and must be equal). The glass stays dark. The phone returns to Android by itself; if it is not on Android after 4 minutes, hold the power button about 15 seconds.", s.luns)
}

/// What the author is told before the rehearsal.
pub fn expect_rehearsal(s: &Settings) -> String {
    format!("The rehearsal of the root boot, about 5 minutes: every step of the boot except the boot itself, run for real - the card read from Android (the partition tables of its {} disks and the root partition hashed), the image, the guard with fastboot, the slots' flags, the boot chain read back and compared with the stored set. Nothing is booted and nothing is written to the card; the phone is in fastboot for a minute and ends on Android. The author's go is not needed and not read.", s.luns)
}

/// The partition the image names as its root on its command line (`walleye.root=system_a`), read from the image file's bytes.
/// Two different names, none, or a name that is not `system_a` or `system_b` is an error.
pub fn root_in_image(image: &[u8]) -> Result<String, String> {
    let key = ROOT_KEY.as_bytes();
    let mut found: Vec<String> = Vec::new();
    for at in image
        .windows(key.len())
        .enumerate()
        .filter(|(_, w)| *w == key)
        .map(|(i, _)| i)
    {
        let rest = &image[at + key.len()..];
        let name: String = rest
            .iter()
            .take_while(|b| b.is_ascii_alphanumeric() || **b == b'_')
            .map(|b| *b as char)
            .collect();
        if !found.contains(&name) {
            found.push(name);
        }
    }
    match found.as_slice() {
        [one] if one == "system_a" || one == "system_b" => Ok(one.clone()),
        [] => Err(format!("the image's command line has no {ROOT_KEY}")),
        other => Err(format!(
            "the image's command line names {other:?} as its root: not exactly one of system_a, system_b"
        )),
    }
}

/// The guard before the boot: adb's suffix (`_b`) and fastboot's current slot (`b`) are the same slot, and the root the image names is
/// the system partition of the OTHER slot. Returns that partition. Any doubt is an error and nothing is booted.
pub fn guard(adb_suffix: &str, fastboot_slot: &str, image_root: &str) -> Result<String, String> {
    let current = slots_agree(adb_suffix, fastboot_slot)?;
    let wanted = root_for(&current)?;
    if image_root == wanted {
        Ok(wanted)
    } else {
        Err(format!(
            "the image's root is {image_root} but the phone runs from slot {current}: the root must be {wanted}, the slot Android is not on: nothing is booted"
        ))
    }
}

/// The word the bootloader puts on the command line of a slotted boot (a fastboot boot from memory gets none), which the image's init needs.
pub const SLOT_KEY: &str = "androidboot.slot_suffix=";

/// Where the boot image header (version 0) keeps its command line: 512 bytes at 64, and 1024 more (`extra_cmdline`) at 608; the 32 bytes between are the id.
const CMDLINE_AT: usize = 64;
const CMDLINE_LEN: usize = 512;
const EXTRA_AT: usize = 608;
const EXTRA_LEN: usize = 1024;

fn cstr(field: &[u8]) -> String {
    let end = field.iter().position(|b| *b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

/// The command line the image's header carries (`cmdline` then `extra_cmdline`, version 0); an image that is not a boot image is an error.
pub fn header_cmdline(image: &[u8]) -> Result<String, String> {
    if image.len() < EXTRA_AT + EXTRA_LEN || &image[..8] != b"ANDROID!" {
        return Err("not a boot image: no ANDROID! header".into());
    }
    if word(image, 40) != 0 {
        return Err(format!(
            "the header is version {}: only version 0 is read here",
            word(image, 40)
        ));
    }
    let mut line = cstr(&image[CMDLINE_AT..CMDLINE_AT + CMDLINE_LEN]);
    line.push_str(&cstr(&image[EXTRA_AT..EXTRA_AT + EXTRA_LEN]));
    Ok(line)
}

/// The command line of THIS run's image: the declared one plus the slot the phone runs from, which the procedure read (`_a` or `_b`).
/// A declared line that already names a slot, a suffix that is not `_a` or `_b`, or a line the header's own field cannot hold is an error.
pub fn run_cmdline(declared: &str, suffix: &str) -> Result<String, String> {
    if suffix != "_a" && suffix != "_b" {
        return Err(format!(
            "the slot suffix read is [{suffix}]: not _a or _b, no image is made"
        ));
    }
    if declared.split_whitespace().any(|w| w.starts_with(SLOT_KEY)) {
        return Err(format!("the declared command line already has a {SLOT_KEY} word: the slot is the run's, never the image's"));
    }
    let line = format!("{declared} {SLOT_KEY}{suffix}");
    if line.len() >= CMDLINE_LEN {
        return Err(format!(
            "the command line would be {} bytes: the header's own field holds {}",
            line.len(),
            CMDLINE_LEN - 1
        ));
    }
    Ok(line)
}

/// The bytes of the digest the AVB footer's hash descriptor holds (sha256), which `image swap` writes again for the new header.
const DIGEST_LEN: usize = 32;

fn word(image: &[u8], at: usize) -> usize {
    u32::from_le_bytes([image[at], image[at + 1], image[at + 2], image[at + 3]]) as usize
}

/// Where the image's content ends and the AVB footer's area begins: the header's page, then kernel, ramdisk and second, each rounded up to a page.
fn content_end(image: &[u8]) -> Result<usize, String> {
    header_cmdline(image)?;
    let page = word(image, 36);
    if page == 0 {
        return Err("the header's page size is 0".into());
    }
    let pages = |n: usize| n.div_ceil(page) * page;
    Ok(page + pages(word(image, 8)) + pages(word(image, 16)) + pages(word(image, 24)))
}

/// The made image equals the declared one in every byte except the command line (the header's `cmdline` and `extra_cmdline`
/// fields), and its command line is exactly `want`: kernel, ramdisk, device tree, header words and id are the same bytes, and after the content
/// only the footer's one digest may differ (the tool signs what it made again).
/// Returns what it counted, for the log.
pub fn differs_only_in_the_command_line(
    declared: &[u8],
    made: &[u8],
    want: &str,
) -> Result<String, String> {
    if declared.len() != made.len() {
        return Err(format!(
            "the made image is {} bytes, the declared one {}",
            made.len(),
            declared.len()
        ));
    }
    let in_command_line = |i: usize| {
        (CMDLINE_AT..CMDLINE_AT + CMDLINE_LEN).contains(&i)
            || (EXTRA_AT..EXTRA_AT + EXTRA_LEN).contains(&i)
    };
    let differing: Vec<usize> = (0..declared.len())
        .filter(|i| declared[*i] != made[*i])
        .collect();
    // the tool signs again what it made: the AVB footer's one sha256 digest (32 bytes) lies after the header, kernel, ramdisk and second
    let signed_from = content_end(declared)?;
    let outside: Vec<usize> = differing
        .iter()
        .copied()
        .filter(|i| !in_command_line(*i))
        .collect();
    if let Some(first) = outside.iter().find(|i| **i < signed_from) {
        return Err(format!("the made image differs from the declared one outside its command line, first at byte {first}"));
    }
    if let (Some(first), Some(last)) = (outside.first(), outside.last()) {
        if last - first >= DIGEST_LEN {
            return Err(format!("the made image differs after its content from byte {first} to byte {last}: more than the footer's one digest"));
        }
    }
    // the command line stands alone in its field: a NUL ends it and nothing but zeros follows, and the extra field is empty
    let used = cstr(&made[CMDLINE_AT..CMDLINE_AT + CMDLINE_LEN]).len();
    let tail_is_zero = made[CMDLINE_AT + used..CMDLINE_AT + CMDLINE_LEN]
        .iter()
        .all(|b| *b == 0)
        && made[EXTRA_AT..EXTRA_AT + EXTRA_LEN].iter().all(|b| *b == 0);
    if !tail_is_zero {
        return Err("the made image's command line field holds bytes after its end, or extra_cmdline is not empty".into());
    }
    if outside.len() > DIGEST_LEN {
        return Err(format!("the made image differs from the declared one in {} bytes after its content: more than the footer's one digest", outside.len()));
    }
    let got = header_cmdline(made)?;
    if got != want {
        return Err(format!(
            "the made image's command line is [{got}], not [{want}]"
        ));
    }
    if differing.is_empty() {
        return Err(
            "the made image is the declared one: no slot was put on its command line".into(),
        );
    }
    Ok(format!(
        "{} bytes differ: the header's command line and, after the content, at most the footer's digest",
        differing.len()
    ))
}

/// What the image's init said to the kernel log.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct InitSaw {
    /// `root=[...]` of its start line
    pub root: Option<String>,
    /// `running slot suffix=[...]` of its start line (empty when the bootloader gave none)
    pub suffix: Option<String>,
    /// the card's device, from `root is /dev/NAME (PARTNAME=...)`
    pub device: Option<String>,
    pub partname: Option<String>,
    /// a `refused: ...` line
    pub refused: Option<String>,
}

fn between<'a>(text: &'a str, open: &str) -> Option<&'a str> {
    let rest = text.split_once(open)?.1;
    Some(rest.split_once(']')?.0)
}

impl InitSaw {
    pub fn of(dmesg: &str) -> InitSaw {
        let mut saw = InitSaw::default();
        for line in dmesg.lines().filter(|l| l.contains(INIT_TAG)) {
            let said = line.split_once(INIT_TAG).map_or("", |p| p.1).trim();
            if said.starts_with("start,") {
                saw.root = between(said, "root=[").map(str::to_string);
                saw.suffix = between(said, "running slot suffix=[").map(str::to_string);
            } else if let Some(rest) = said.strip_prefix("root is /dev/") {
                saw.device = rest.split_whitespace().next().map(str::to_string);
                saw.partname = rest
                    .split_once("PARTNAME=")
                    .map(|p| p.1.trim_end_matches(')').trim().to_string());
            } else if let Some(why) = said.strip_prefix("refused:") {
                saw.refused = Some(why.trim().to_string());
            }
        }
        saw
    }

    /// The init saw what the lab expects: no refusal, the slot suffix adb read (a missing one is a failed-open guard), the named root,
    /// that root's PARTNAME on one device. The hand-over is not read here: the init's last line is lost (see pid1_is_the_roots_init).
    pub fn matches(&self, root: &str, adb_suffix: &str) -> Done {
        if let Some(why) = &self.refused {
            return Err(format!("the image's init refused: {why}"));
        }
        let suffix = self
            .suffix
            .as_deref()
            .ok_or("the kernel log has no start line of the image's init")?;
        if suffix.is_empty() {
            return Err("the bootloader gave the init no androidboot.slot_suffix: its guard failed open, the run stops".into());
        }
        if suffix != adb_suffix.trim() {
            return Err(format!(
                "the init saw slot suffix {suffix:?}, adb read {:?}",
                adb_suffix.trim()
            ));
        }
        if self.root.as_deref() != Some(root) {
            return Err(format!(
                "the init's root is {:?}, the lab expected {root}",
                self.root
            ));
        }
        if self.partname.as_deref() != Some(root) || self.device.is_none() {
            return Err(format!(
                "the init mounted {:?} (PARTNAME {:?}), not {root}",
                self.device, self.partname
            ));
        }
        Ok(())
    }
}

/// PID 1 is the root's /sbin/init (what `exec switch_root /newroot /sbin/init` leaves), not the initramfs init: the hand-over from the live state,
/// since the init's own 'handing over' line goes to /dev/kmsg after /dev was moved and is lost.
pub fn pid1_is_the_roots_init(pid1_cmdline: &str) -> Result<(), String> {
    match pid1_cmdline
        .split(|c: char| c.is_whitespace() || c == '\0')
        .find(|w| !w.is_empty())
    {
        Some("/sbin/init") => Ok(()),
        Some(other) => Err(format!(
            "PID 1 is {other:?} ({}), not the root's /sbin/init: the init did not hand over",
            pid1_cmdline.trim()
        )),
        None => Err(
            "PID 1's command line is empty or unread: not known, the hand-over is not proved"
                .into(),
        ),
    }
}

/// The overlay root's lower layer is the very partition the init named, mounted read-only.
pub fn lower_is_the_named_card(proc_mounts: &str, device: &str) -> Result<(), String> {
    let lower = mounts_are_read_only(proc_mounts)?;
    if lower != format!("/dev/{device}") {
        return Err(format!("/lower is {lower}, the init named /dev/{device}"));
    }
    Ok(())
}

/// The option words of a mount line.
fn options(line: &str) -> Vec<&str> {
    line.split_whitespace()
        .nth(3)
        .map_or(Vec::new(), |o| o.split(',').collect())
}

/// `/proc/mounts` says: the root is the overlay on `/lower`, the card's partition is mounted at `/lower` read-only (`ro`, `norecovery` as ext4 prints `noload`,
/// no `discard`), and no block device of the card is mounted anywhere writable. Returns the card's device.
pub fn mounts_are_read_only(proc_mounts: &str) -> Result<String, String> {
    let lines: Vec<&str> = proc_mounts.lines().collect();
    let field = |l: &str, n: usize| l.split_whitespace().nth(n).unwrap_or("").to_string();
    let roots: Vec<&&str> = lines.iter().filter(|l| field(l, 1) == "/").collect();
    let overlay = roots
        .iter()
        .find(|l| field(l, 2) == "overlay")
        .ok_or("no overlay is mounted as /")?;
    if !options(overlay).contains(&"lowerdir=/lower") {
        return Err(format!(
            "the overlay's lower layer is not /lower: {overlay}"
        ));
    }
    let lower: Vec<&&str> = lines.iter().filter(|l| field(l, 1) == "/lower").collect();
    let [lower] = lower.as_slice() else {
        return Err(format!("{} mounts at /lower, expected one", lower.len()));
    };
    let device = field(lower, 0);
    if !device.starts_with("/dev/") || field(lower, 2) != "ext4" {
        return Err(format!("/lower is not an ext4 block device: {lower}"));
    }
    let opts = options(lower);
    if !opts.contains(&"ro") {
        return Err(format!("/lower is not mounted with ro: {lower}"));
    }
    // the init mounts with `noload`; ext4 prints that option as `norecovery` in /proc/mounts (run 4 of the stage 1 boot: `ro,relatime,norecovery`)
    if !opts.iter().any(|o| ["norecovery", "noload"].contains(o)) {
        return Err(format!(
            "/lower is not mounted with norecovery (noload): {lower}"
        ));
    }
    for refused in ["rw", "discard"] {
        if opts.contains(&refused) {
            return Err(format!("/lower is mounted with {refused}: {lower}"));
        }
    }
    for l in &lines {
        let dev = field(l, 0);
        let card = dev.starts_with("/dev/sd") || dev.starts_with("/dev/mmcblk");
        if card && !options(l).contains(&"ro") {
            return Err(format!("a card device is mounted writable: {l}"));
        }
    }
    Ok(device)
}

/// The command that hashes the primary and the backup GPT of one disk as Android names it: a read, never a write.
pub fn gpt_command(letter: char) -> Result<String, String> {
    if !letter.is_ascii_lowercase() {
        return Err(format!("{letter:?} is not a disk letter (a to z)"));
    }
    let d = format!("/dev/block/sd{letter}");
    Ok(format!(
        "sz=$(cat /sys/block/sd{letter}/size); [ \"${{sz:-0}}\" -gt 64 ] || exit 1; (dd if={d} bs=4096 count=6 2>/dev/null; dd if={d} bs=4096 skip=$((sz/8-5)) count=5 2>/dev/null) | sha256sum"
    ))
}

/// The ssh key is a file only its owner can read: ssh refuses the key otherwise, and a key others read is no key.
pub fn identity_is_private(path: &Path) -> Done {
    let meta = std::fs::metadata(path).map_err(|e| format!("identity {}: {e}", path.display()))?;
    if !meta.is_file() {
        return Err(format!("identity {} is not a file", path.display()));
    }
    let mode = meta.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(format!(
            "identity {} has mode {mode:o}: group or world can read it (chmod 600)",
            path.display()
        ));
    }
    Ok(())
}

/// The ssh arguments for one command on the phone: the declared key only, no agent, no prompt, a point-to-point link whose host key
/// is new at every boot (the root is a tmpfs overlay), so no known-hosts file is kept or read.
pub fn ssh_args(
    identity: &Path,
    user: &str,
    address: &str,
    port: u16,
    command: &str,
) -> Vec<String> {
    let o = |s: &str| ["-o".to_string(), s.to_string()];
    let mut a: Vec<String> = Vec::new();
    a.extend(["-F".to_string(), "/dev/null".to_string()]);
    for opt in [
        "BatchMode=yes",
        "ControlPath=none",
        "ProxyCommand=none",
        "IdentitiesOnly=yes",
        "StrictHostKeyChecking=no",
        "UserKnownHostsFile=/dev/null",
        "LogLevel=ERROR",
        "ConnectTimeout=5",
        "ServerAliveInterval=5",
        "ServerAliveCountMax=3",
    ] {
        a.extend(o(opt));
    }
    a.extend(["-i".into(), identity.to_string_lossy().into_owned()]);
    a.extend(["-p".into(), port.to_string(), format!("{user}@{address}")]);
    a.push(command.to_string());
    a
}

/// The sections of the one read the lab makes after the login: `---NAME` lines in the output separate them.
pub fn sections(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("---") {
            out.push((name.trim().to_string(), String::new()));
        } else if let Some(last) = out.last_mut() {
            last.1.push_str(line);
            last.1.push('\n');
        }
    }
    out
}

/// The one read: what is running, what is mounted, what the kernel said, what the root's init manages.
pub const READ_COMMAND: &str = "echo ---id; id -u; echo ---kernel; uname -r; echo ---cmdline; cat /proc/cmdline; echo ---pid1; cat /proc/1/cmdline; echo; echo ---mounts; cat /proc/mounts; echo ---dmesg; dmesg; echo ---services; rc-status --crashed 2>&1 || true; echo ---net; ip -o addr show usb0 2>&1; echo ---end";

/// What the lab reads, read-only, when a check after the boot failed and the phone is still on the image (item
/// walleye-rootfs-boots-run-2-diagnose): the kernel log's tail and the init's own lines, the command line, the mounts, the
/// processes, the root and where the init may have put the new root, any log the init keeps, the card's partition names, the space.
pub const EVIDENCE_COMMAND: &str = "echo ---dmesg-tail; dmesg | tail -n 300; echo ---init-lines; dmesg | grep 'walleye-ufs-root:'; echo ---cmdline; cat /proc/cmdline; echo ---pid1; cat /proc/1/cmdline; echo; echo ---mounts; cat /proc/mounts; echo ---ps; ps; echo ---root; ls -l /; echo ---new-root; ls -l /new_root /mnt /sysroot 2>&1; echo ---init-logs; for f in /run/*.log /var/log/*.log /tmp/*.log /*.log; do [ -f \"$f\" ] && { echo \"== $f\"; tail -n 100 \"$f\"; echo; }; done; echo ---partnames; for u in /sys/class/block/*/uevent; do grep -H PARTNAME \"$u\"; done; echo ---df; df 2>&1; echo ---end";

/// The slot suffix adb read, kept so the watching parent can measure the card as the run would have.
pub const SUFFIX_FILE: &str = "slot-suffix.txt";

/// The file the evidence is kept in, in the run's folder.
pub const EVIDENCE_FILE: &str = "evidence.txt";

/// The way back's first try (items walleye-way-back-reboot-bootloader-first, walleye-reboot-reason-restart2-tool): busybox's `reboot bootloader`
/// drops the argument, so the lab pushes its static walleye-reboot over ssh into the phone's tmpfs (`/tmp` of the overlay: nothing touches the card)
/// and runs it there; the tool hands `bootloader` to reboot(2) RESTART2. The file is the one device.toml's `[images.walleye-reboot]` names, by sha256.
/// (one copy for both stages: [`crate::reboot_tool`])
pub use crate::reboot_tool::{
    BOOTLOADER_COMMAND, ON_PHONE as REBOOT_TOOL_ON_PHONE, SEND_COMMAND as SEND_TOOL_COMMAND,
};

/// How long fastboot has to list the phone after [`BOOTLOADER_COMMAND`].
pub const BOOTLOADER_WAIT: Duration = Duration::from_secs(60);

/// What the bootloader try saw, kept in the run's folder (a later way back in the same run does not try twice).
pub const BOOTLOADER_FILE: &str = "bootloader-way-back.txt";

/// What the rw write's clean end saw (init asked to end through its shutdown level), kept in the run's folder; tried once a run.
pub const CLEAN_END_FILE: &str = "clean-end-way-back.txt";

/// Where a post-run measurement stands, read from the run's folder alone.
#[derive(Debug, PartialEq)]
pub enum Measure {
    /// the run's own step already measured it (and judged it)
    Done,
    /// nothing was read before the boot: there is nothing to equal
    NothingBefore,
    /// not measured yet: the way back measures it
    Pending,
}

/// Whether a post-run measurement is still to be made: `before` and `after` are names in the run's folder `out` (or, for the
/// boot chain, `before` is the stored set's own path).
pub fn measure_status(before: &Path, after: &Path) -> Measure {
    if after.exists() {
        Measure::Done
    } else if !before.exists() {
        Measure::NothingBefore
    } else {
        Measure::Pending
    }
}

/// One state the way back measures after a stop: what it is, what was read before, where the after goes, and the measure.
struct AfterStop<'a> {
    what: &'static str,
    before: std::path::PathBuf,
    after: std::path::PathBuf,
    measure: Box<dyn Fn() -> Done + 'a>,
}

impl<'a> AfterStop<'a> {
    fn new(
        what: &'static str,
        before: std::path::PathBuf,
        after: std::path::PathBuf,
        measure: Box<dyn Fn() -> Done + 'a>,
    ) -> Self {
        AfterStop {
            what,
            before,
            after,
            measure,
        }
    }
}

/// The two ways the lab sends the phone away from this image: busybox's forced reboot, then the sysrq trigger.
pub const REBOOT_COMMANDS: [&str; 2] = ["reboot -f", "echo b > /proc/sysrq-trigger"];
/// The rw write's clean end over ssh (research walleye-rw-fsck-counts-stale-superblock: the root has no journal, so its free-count totals reach the
/// superblock only when it is unmounted or remounted read-only): the root is synced and remounted read-only FIRST (the list is slow and must not cost the remount its time), then the files written since boot are listed (-mmin from /proc/uptime
/// ; NOT KNOWN YET that busybox find takes it), and the root's mount line is read back so the end is said, not assumed. It ends in true: a read that printed nothing is not an ssh failure.
pub const SETTLE_COMMAND: &str = "echo ---remount; sync; mount -o remount,ro / 2>&1; echo rc=$?; echo ---mount; grep ' / ' /proc/mounts | tail -n 1; echo ---written; find / -xdev -type f -mmin -$(( $(cut -d. -f1 /proc/uptime) / 60 + 1 )) 2>&1 | head -n 200; true";

fn section<'a>(parts: &'a [(String, String)], name: &str) -> Result<&'a str, String> {
    parts
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, t)| t.as_str())
        .ok_or_else(|| format!("the read has no {name} section"))
}

/// The boot try: its settings, the lab it shares with the other tries, the adb view of Android, what it read before.
#[derive(Clone)]
pub struct RootfsBoot {
    s: Rc<Settings>,
    lab: Lab,
    android: Android,
    /// the ssh program (`ssh`; a test names a stand-in)
    ssh: String,
    /// how long fastboot has to list the phone after the bootloader command ([`BOOTLOADER_WAIT`]; a test names a short one)
    bootloader_wait: Duration,
    /// adb's suffix and fastboot's slot as read before the boot, and the root they give
    before: Rc<std::cell::RefCell<Before>>,
    /// the rw write's mode (`RootfsBoot::rw_write`): the image must carry `walleye.rw`, the root is mounted read-write, the card must CHANGE in its root only
    rw: bool,
    /// the rw write's rehearsal: nothing is booted and nothing written, so the card must be EQUAL after, as in the root boot
    rehearsal: bool,
}

#[derive(Default, Clone)]
struct Before {
    suffix: String,
    root: String,
    /// THIS run's image once made (the declared one with the slot on its command line) and its sha256 as made: what is booted must be these bytes
    run_image: Option<(std::path::PathBuf, String)>,
}

impl RootfsBoot {
    pub fn new(s: Settings) -> RootfsBoot {
        RootfsBoot::with_ssh(s, "ssh")
    }

    pub fn with_ssh(s: Settings, ssh: &str) -> RootfsBoot {
        let s = Rc::new(s);
        let lab = Lab::new(s.clone());
        RootfsBoot {
            android: Android::new(s.clone(), lab.clone()),
            lab,
            s,
            ssh: ssh.to_string(),
            bootloader_wait: BOOTLOADER_WAIT,
            before: Rc::default(),
            rw: false,
            rehearsal: false,
        }
    }

    /// The rw write's try: the same steps on the rw-write image (item walleye-install-rw-write-procedure); the steps that differ judge the rw state
    /// (`crate::rw_write`), the rest are the root boot's own.
    pub fn rw_write(s: Settings) -> RootfsBoot {
        let mut b = RootfsBoot::new(s);
        b.rw = true;
        b
    }

    /// The rw write's rehearsal: every step of the write but the boot (nothing booted, nothing written): the card must be as before.
    pub fn rw_write_rehearsal(s: Settings) -> RootfsBoot {
        let mut b = RootfsBoot::rw_write(s);
        b.rehearsal = true;
        b
    }

    pub(crate) fn settings(&self) -> &Settings {
        &self.s
    }

    /// The root partition the run expects (`system_a`/`system_b`), read from the slot before the boot.
    pub(crate) fn the_root(&self) -> Result<String, String> {
        self.root()
    }

    pub(crate) fn on_phone(&self, command: &str, seconds: u64) -> Result<String, String> {
        self.ssh(command, seconds)
    }

    pub(crate) fn kept(&self, name: &str, text: &str) -> Done {
        self.lab.save(name, text)
    }

    pub(crate) fn section_of_read(&self, name: &str) -> Result<String, String> {
        self.read_section(name)
    }

    fn root(&self) -> Result<String, String> {
        let b = self.before.borrow();
        if b.root.is_empty() {
            Err("the slot is not read yet: nothing is booted".into())
        } else {
            Ok(b.root.clone())
        }
    }

    fn by_name_of(&self, name: &str) -> String {
        format!("{}/{name}", self.s.by_name.trim_end_matches('/'))
    }

    /// One command on the phone over ssh, as the declared key; what it said, or why it failed. Bounded: `timeout` ends a hung link.
    fn ssh(&self, command: &str, seconds: u64) -> Result<String, String> {
        self.ssh_from(command, seconds, None)
    }

    /// [`Self::ssh`] with a file on the command's standard input (how a tool is sent to the phone's tmpfs).
    fn ssh_from(
        &self,
        command: &str,
        seconds: u64,
        stdin: Option<&std::path::Path>,
    ) -> Result<String, String> {
        crate::reboot_tool::ssh_from(&self.s, &self.ssh, command, seconds, stdin)
    }

    fn saved(&self, name: &str) -> Result<String, String> {
        std::fs::read_to_string(self.s.out.join(name)).map_err(|e| format!("{name}: {e}"))
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The image is the declared one (its sha256), lives in a NOT-TO-BOOT folder, and names exactly one root partition on its command line.
    pub fn the_image_is_the_declared_one(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            if !u.s.image_name.is_empty() {
                println!(
                    "  {}",
                    host::the_image_is_ensured(
                        &u.s.bringup,
                        &u.s.image_name,
                        &u.s.device_toml(),
                        &u.s.image,
                        &u.s.image_sha256
                    )?
                );
            }
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            if !u.s.image.to_string_lossy().contains("NOT-TO-BOOT") {
                return Err(format!(
                    "{} is not under a NOT-TO-BOOT name: a UFS-on image never lives where a queue looks",
                    u.s.image.display()
                ));
            }
            let bytes =
                std::fs::read(&u.s.image).map_err(|e| format!("{}: {e}", u.s.image.display()))?;
            let root = root_in_image(&bytes)?;
            // the word walleye.rw makes the init mount the root read-write: the rw write's image must carry it once, every other image must not
            if u.rw {
                crate::rw_write::cmdline_carries_the_word(&header_cmdline(&bytes)?)?;
            } else if bytes
                .split(|b| b.is_ascii_whitespace() || *b == 0)
                .any(|w| w == crate::rw_write::WORD.as_bytes())
            {
                return Err(format!(
                    "the image's command line carries {}: this try is the read-only boot",
                    crate::rw_write::WORD
                ));
            }
            println!("  the image names its root: {root}");
            Ok(())
        }
    }

    /// The stored boot chain is a whole read-back (the lab's own check), the set the boot chain is compared with at the end; and the
    /// key the lab logs in with is private.
    pub fn the_stored_set_and_the_key_are_good(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            identity_is_private(&u.s.identity)?;
            let dir = u.s.bootchain.to_string_lossy().into_owned();
            u.lab
                .script("bootchain-readback-check.sh", &[&dir], &[])
                .map(|_| ())
        }
    }

    /// Say what to expect, before the first act on the phone.
    pub fn author_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = if self.rw {
            crate::rw_write::expect(self.s.write_mib, self.s.luns)
        } else {
            expect(&self.s)
        };
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    /// Say what to expect of the rehearsal.
    pub fn rehearsal_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = if self.rw {
            crate::rw_write::expect_rehearsal(self.s.luns)
        } else {
            expect_rehearsal(&self.s)
        };
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    /// The rehearsal's way back: nothing was booted, so the phone is on Android or in fastboot and the shared way back is all it needs
    /// (no ssh is tried). Only the watching parent runs it, after the child ends, whatever ended it; the run itself leaves fastboot by its own step.
    pub fn rehearsal_way_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            if std::env::var_os(watch::CHILD).is_some() {
                return Ok(());
            }
            lab.way_back()
        }
    }

    /// The phone is on Android with adbd as root (the first act on the phone, so the watcher owns the way back).
    pub fn the_phone_is_on_android_as_root(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            watch::phone_is_touched();
            u.android.is_up_as_root()
        }
    }

    /// adb says which slot Android runs from; kept for the guard.
    pub fn the_slot_is_read_by_adb(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let suffix = u.android.slot_suffix()?;
            u.forget_earlier_runs();
            u.before.borrow_mut().suffix = suffix.trim().to_string();
            u.lab.save(SUFFIX_FILE, suffix.trim())?;
            println!("  adb: slot suffix {}", suffix.trim());
            Ok(())
        }
    }

    /// A new run forgets what an earlier run of the same day kept in this folder: its after-files would pass for this run's.
    fn forget_earlier_runs(&self) {
        for name in [
            "card-before.txt",
            "read.txt",
            "write.txt",
            "readback.txt",
            "fsck.txt",
            "dmesg-after.txt",
            "remove.txt",
            "card-after.txt",
            "slots-before.txt",
            "slots-after.txt",
            EVIDENCE_FILE,
            SUFFIX_FILE,
            BOOTLOADER_FILE,
            CLEAN_END_FILE,
        ] {
            let _ = std::fs::remove_file(self.s.out.join(name));
        }
        let _ = std::fs::remove_dir_all(self.s.out.join("bootchain-after"));
    }

    /// The card as it is now, read from Android: the GPT of each of its disks and the root partition's sha256, kept for the end.
    pub fn the_card_is_read_before(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.read_the_card("card-before.txt")
    }

    /// Android's card again, after the boot: equal to what was read before.
    pub fn the_card_is_as_before(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.read_the_card("card-after.txt")?;
            let (a, b) = (u.saved("card-before.txt")?, u.saved("card-after.txt")?);
            if u.rw && !u.rehearsal {
                // the write must reach the root partition: every table equal, the root changed
                return crate::rw_write::card_judge(
                    &a,
                    &b,
                    u.s.luns,
                    crate::rw_write::root_rule(&u.s.out),
                );
            }
            if a == b {
                Ok(())
            } else {
                Err(format!("the card changed:\n{a}\nthen\n{b}"))
            }
        }
    }

    fn read_the_card(&self, name: &str) -> Done {
        let mut text = String::new();
        let letters = disk_letters(self.s.luns)?;
        for letter in &letters {
            let said = self.android.adb(&["shell", &gpt_command(*letter)?])?;
            let digest = crate::ufs::load::digest_of(&said)
                .filter(|d| d != crate::ufs::load::EMPTY_INPUT)
                .ok_or_else(|| format!("sd{letter}: no GPT digest in {said:?}"))?;
            text.push_str(&format!("gpt sd{letter} {digest}\n"));
        }
        let root = root_for(&self.before.borrow().suffix)?;
        let digest = self.android.sha256(&self.by_name_of(&root), None)?;
        text.push_str(&format!("root {root} {digest}\n"));
        self.lab.save(name, text.trim_end())?;
        println!("  {} disks and {root} hashed", letters.len());
        Ok(())
    }

    /// The phone is in fastboot (the shared step).
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.phone_is_in_fastboot()
    }

    /// The guard: fastboot's current slot is read now, equals adb's, and the image's root is the other slot's system partition.
    pub fn the_guard_holds(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut c = Command::new(&u.s.fastboot);
            c.args(["-s", &u.lab.serial()?, "getvar", "current-slot"]);
            let slot = parse_current_slot(&host::run(c)?.text)?;
            let bytes =
                std::fs::read(&u.s.image).map_err(|e| format!("{}: {e}", u.s.image.display()))?;
            let suffix = u.before.borrow().suffix.clone();
            let root = guard(&suffix, &slot, &root_in_image(&bytes)?)?;
            u.before.borrow_mut().root = root.clone();
            u.lab.save(
                "guard.txt",
                &format!("adb suffix {suffix}\nfastboot slot {slot}\nroot {root}"),
            )?;
            println!("  the root is {root}; Android runs from slot {slot}");
            Ok(())
        }
    }

    /// THIS run's image is made: the declared image with the slot adb read (and fastboot agreed on) put on its command line, by the
    /// boot-image tool's own `image swap` (no byte edited here). A fastboot boot from memory gives the kernel no `androidboot.slot_suffix`,
    /// and the image's init fails closed without it. The made image must equal the declared one in every byte but its command line; both
    /// sha256 are kept (the declared one stays the one the go names). Nothing is booted, nothing written but the made image in the run's folder.
    pub fn the_run_image_is_made(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.root()?;
            let declared =
                std::fs::read(&u.s.image).map_err(|e| format!("{}: {e}", u.s.image.display()))?;
            let line = run_cmdline(&header_cmdline(&declared)?, &u.before.borrow().suffix)?;
            std::fs::create_dir_all(&u.s.out).map_err(|e| format!("{}: {e}", u.s.out.display()))?;
            let made = u.s.out.join(RUN_IMAGE);
            let _ = std::fs::remove_file(&made); // a stale image never stands in for this run's
            let mut c = Command::new(&u.s.bringup);
            c.arg("--device")
                .arg(u.s.device_toml())
                .args(["image", "swap"])
                .arg(&u.s.image)
                .args(["--cmdline", &line, "--out"])
                .arg(&made);
            let said = host::run(c)?;
            if !said.ok {
                return Err(format!("the run image was not made: {}", said.text));
            }
            let bytes = std::fs::read(&made).map_err(|e| format!("{}: {e}", made.display()))?;
            let counted = differs_only_in_the_command_line(&declared, &bytes, &line)?;
            let (was, now) = (host::sha256_of(&u.s.image)?, host::sha256_of(&made)?);
            u.lab.save(
                "run-image.txt",
                &format!("declared {was}\nmade {now}\ncommand line {line}\n{counted}"),
            )?;
            println!("  the run image {now} is the declared {was} with the slot on its command line ({counted})");
            u.before.borrow_mut().run_image = Some((made, now));
            Ok(())
        }
    }

    fn slots(&self, name: &str) -> Done {
        let flags = host::slot_flags(&self.s.fastboot, &self.lab.serial()?)?;
        self.lab.save(name, &flags)
    }

    /// The slots' flags, read in fastboot before the boot.
    pub fn the_slot_flags_are_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.slots("slots-before.txt")
    }

    /// Back on Android: the phone goes to fastboot once more, the flags are read and equal the first, and it goes back to Android.
    pub fn the_slot_flags_are_as_before(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.lab.phone_is_in_fastboot()?;
            let read = u.slots("slots-after.txt");
            let back = u.lab.way_back();
            read?;
            back?;
            let (a, b) = (u.saved("slots-before.txt")?, u.saved("slots-after.txt")?);
            if a == b {
                Ok(())
            } else {
                Err(format!("the slot flags changed:\n{a}\nthen\n{b}"))
            }
        }
    }

    /// The author's go covers this image now: the item's row (`tad q row`) names the image's sha256 and has `go_until` in the future.
    fn go_is_open_now(&self) -> Done {
        let mut c = Command::new("tad");
        c.args(["q", "row", "--id", &self.s.go_item, "--format", "csv"])
            .current_dir(&self.s.lab);
        let said = host::run(c)?;
        if !said.ok {
            return Err(format!(
                "no go: the item {} could not be read: {}",
                self.s.go_item, said.text
            ));
        }
        let now = now_epoch().ok_or("no go: the clock could not be read")?;
        go_decision(&said.text, now, &self.s.image_sha256, &self.s.go_item)
    }

    /// The author's go is open on the item. Without it the run stops here, "no go", and the way back leaves the phone on Android.
    pub fn author_has_said_go(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.go_is_open_now()
    }

    /// The boot itself, after the gate's other checks: this run's made image is the file booted (never the declared one), and it is still the
    /// bytes that were checked when it was made (their sha256) and still the declared image with only the slot added.
    fn boot_the_made_image(&self) -> Done {
        let (made, sha) = self.before.borrow().run_image.clone().ok_or(
            "the run image is not made: the slot is not on its command line, nothing is booted",
        )?;
        self.the_made_image_is_this_runs(&made, &sha)?;
        self.lab.boot_image(&made)
    }

    /// The gate right before the boot: the file about to boot is the file that was checked (its sha256), and still the declared image with
    /// the slot adb read on its command line and nothing else changed.
    fn the_made_image_is_this_runs(&self, made: &Path, sha: &str) -> Done {
        let now = host::sha256_of(made)?;
        if now != sha {
            return Err(format!(
                "the run image changed since it was made: {now}, made as {sha}"
            ));
        }
        let declared =
            std::fs::read(&self.s.image).map_err(|e| format!("{}: {e}", self.s.image.display()))?;
        let bytes = std::fs::read(made).map_err(|e| format!("{}: {e}", made.display()))?;
        let line = run_cmdline(&header_cmdline(&declared)?, &self.before.borrow().suffix)?;
        differs_only_in_the_command_line(&declared, &bytes, &line).map(|_| ())
    }

    /// The step that boots the UFS-on image from the phone's fastboot, from memory: nothing is written. It holds its own gate, right
    /// before the boot, whatever ran before: the battery, the image's sha256, the guard having read its slot, and the go.
    pub fn boot_the_root_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.root()?;
            u.lab.battery_is_good()?;
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            u.go_is_open_now()?;
            u.boot_the_made_image()
        }
    }

    /// The lab logs in over usb-network by ssh as the declared key and makes the one read, polled until `device_wait` is over: the
    /// gadget, the network and sshd come up one after the other. The read is kept (serials redacted).
    pub fn the_login_works_and_the_phone_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let until = Instant::now() + Duration::from_secs(u.s.device_wait);
            let read = loop {
                match u.ssh(READ_COMMAND, 40) {
                    Ok(text) if text.contains("---end") => break text,
                    other => {
                        if Instant::now() >= until {
                            return Err(format!(
                                "no ssh login within {} s as {}@{}: {}",
                                u.s.device_wait,
                                u.s.ssh_user,
                                u.s.address,
                                other
                                    .err()
                                    .unwrap_or_else(|| "the read did not finish".into())
                            ));
                        }
                        std::thread::sleep(Duration::from_secs(3));
                    }
                }
            };
            u.lab.save("read.txt", &redact(&read))
        }
    }

    fn read_section(&self, name: &str) -> Result<String, String> {
        let text = self.saved("read.txt")?;
        section(&sections(&text), name).map(str::to_string)
    }

    /// The login is root's, on the kernel we built, and the root's own services have none crashed.
    pub fn the_phone_is_the_image_we_booted(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let id = u.read_section("id")?;
            if id.trim() != "0" {
                return Err(format!("the login is uid {:?}, not root's 0", id.trim()));
            }
            let crashed = u.read_section("services")?;
            if !crashed.trim().is_empty() {
                return Err(format!("services crashed: {}", crashed.trim()));
            }
            let net = u.read_section("net")?;
            if !net.contains(&u.s.address) {
                return Err(format!(
                    "usb0 has not the address {}: {}",
                    u.s.address,
                    net.trim()
                ));
            }
            println!("  kernel {}", u.read_section("kernel")?.trim());
            Ok(())
        }
    }

    /// The init saw the slot adb saw and mounted the named partition; the guard did not fail open; and the hand-over happened: PID 1 is the
    /// root's /sbin/init and the overlay's lower layer is that partition (the live state, not the init's lost last log line).
    pub fn the_init_saw_the_slot_adb_saw(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let dmesg = u.read_section("dmesg")?;
            let saw = InitSaw::of(&dmesg);
            let before = u.before.borrow().clone();
            saw.matches(&before.root, &before.suffix)?;
            pid1_is_the_roots_init(&u.read_section("pid1")?)?;
            if !u.rw {
                lower_is_the_named_card(
                    &u.read_section("mounts")?,
                    saw.device.as_deref().unwrap_or(""),
                )?;
            }
            println!(
                "  the init mounted /dev/{} (PARTNAME {}) with slot suffix {}",
                saw.device.as_deref().unwrap_or("?"),
                saw.partname.as_deref().unwrap_or("?"),
                saw.suffix.as_deref().unwrap_or("?")
            );
            Ok(())
        }
    }

    /// The card is mounted read-only under an overlay, and nothing of the card is mounted writable.
    pub fn the_root_is_mounted_read_only(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let device = mounts_are_read_only(&u.read_section("mounts")?)?;
            println!(
                "  {device} is mounted read-only (ro, norecovery, no discard) under the overlay"
            );
            Ok(())
        }
    }

    /// The kernel log has no purge, no oops, no UFS failure.
    pub fn the_kernel_log_is_clean(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let dmesg = u.read_section("dmesg")?;
            if let Some(t) = Probe::of(&dmesg).trouble.first() {
                return Err(format!("the kernel log says: {t}"));
            }
            if let Some(t) = lpoco_attended::steps::trouble_lines(&dmesg).first() {
                return Err(format!("the kernel log says: {t}"));
            }
            Ok(())
        }
    }

    /// The rw write only: before the phone is sent away the root is settled over ssh - `sync`, then `mount -o remount,ro /` (the way back's own step; the init never remounts) - so a stop does not
    /// cut off what was written. What it said is printed; a failure is NOT KNOWN YET, never a stop (the way back goes on).
    fn settle_rw(&self) {
        if !self.rw || self.rehearsal {
            return;
        }
        match self.ssh(SETTLE_COMMAND, 60) {
            Ok(said) => {
                let name = if self.s.out.join("settled.txt").exists() {
                    "settled-again.txt"
                } else {
                    "settled.txt"
                };
                let _ = self.lab.save(name, &said);
                println!(
                    "  the root is settled before the phone is sent away: {}",
                    settled_said(&said)
                );
            }
            Err(e) => {
                println!("NOT KNOWN YET: the root could not be settled (sync, remount ro): {e}")
            }
        }
    }

    /// The phone is sent away from the image: to its bootloader first ([`RootfsBoot::the_bootloader_is_tried`]), and when that is
    /// not reached busybox's forced reboot over ssh (in the rw write the root is settled first: sync, remount ro).
    pub fn the_phone_is_sent_back(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            if u.the_clean_end_is_tried() {
                u.lab.fastboot_reboot();
                return Ok(());
            }
            u.settle_rw();
            if u.the_bootloader_is_tried() {
                // the watched child's own way back step sends nothing (the parent runs it after the child ends), so the
                // bootloader is left here: run 2 waited its whole Android wait in fastboot at the check after it
                u.lab.fastboot_reboot();
            } else {
                // the phone hangs up as it goes down: the end of the answer is not read
                let _ = u.ssh(REBOOT_COMMANDS[0], 10);
            }
            Ok(())
        }
    }

    /// The way back from the image through the bootloader (item walleye-way-back-reboot-bootloader-first): `walleye-reboot --command bootloader` over
    /// ssh, fastboot lists the phone within the wait, its `current-slot` is read and said against the slot adb read at the start, and
    /// the shared way back then sends `fastboot reboot` and waits for Android. Nothing is set, flashed or erased. True when the
    /// bootloader was reached; false says "bootloader not reached" with what was seen, and the forced reboots follow.
    /// Tried once a run: what it saw is kept in [`BOOTLOADER_FILE`].
    pub fn the_bootloader_is_tried(&self) -> bool {
        if self.s.out.join(BOOTLOADER_FILE).exists() {
            return false;
        }
        let sent = self.the_bootloader_is_asked_for(); // the phone hangs up as it goes down
        let seen = self.fastboot_lists_the_phone();
        let (reached, said) = match seen {
            Ok(()) => (true, self.the_bootloader_slot_is_said()),
            Err(e) => {
                let ssh = sent.err().unwrap_or_else(|| "ssh answered".into());
                (false, format!(
                    "bootloader not reached: {e} within {} s (the ssh said: {ssh}); the forced way back runs",
                    self.bootloader_wait.as_secs()
                ))
            }
        };
        println!("{said}");
        let _ = self.lab.save(BOOTLOADER_FILE, &said);
        reached
    }

    /// Fastboot lists the phone within `bootloader_wait`, or why not.
    fn fastboot_lists_the_phone(&self) -> Result<(), String> {
        let until = Instant::now() + self.bootloader_wait;
        loop {
            match self.lab.in_fastboot() {
                Ok(true) => return Ok(()),
                Ok(false) if Instant::now() < until => {}
                Ok(false) => return Err("fastboot lists no phone".to_string()),
                Err(e) => return Err(e),
            }
            std::thread::sleep(Duration::from_secs(1).min(self.bootloader_wait));
        }
    }

    /// The rw write's way back (item walleye-rw-end-through-openrc-mount-ro): init is asked to end, so the root goes through its shutdown level (`mount-ro`, then the
    /// walleye-end service, which runs the tool left on /run and always reboots) instead of a remount inside the live ssh session. True when fastboot lists the phone afterwards; false
    /// leaves the old way (settle, the bootloader try, the forced reboots) to run, and says why. Tried once a run: what it saw is kept in [`CLEAN_END_FILE`].
    pub fn the_clean_end_is_tried(&self) -> bool {
        if !self.rw || self.rehearsal || self.s.out.join(CLEAN_END_FILE).exists() {
            return false;
        }
        let sent = crate::reboot_tool::ask_for_the_clean_end(&self.s, &self.ssh);
        let said = match (&sent, self.fastboot_lists_the_phone()) {
            (Err(e), _) => format!("clean end not asked for: {e}; the old way back runs"),
            (Ok(_), Ok(())) => "clean end reached: init was asked to end, the shutdown level ran and fastboot lists the phone; fastboot reboot follows".to_string(),
            (Ok(_), Err(e)) => format!("clean end not reached: {e} within {} s; the old way back runs", self.bootloader_wait.as_secs()),
        };
        println!("{said}");
        let _ = self.lab.save(CLEAN_END_FILE, &said);
        said.starts_with("clean end reached")
    }

    /// The tool is checked against its declared sha256, sent to the phone's tmpfs over ssh, and run with `bootloader`.
    /// An error names the step that failed (the forced way back follows in every case).
    fn the_bootloader_is_asked_for(&self) -> Result<String, String> {
        crate::reboot_tool::ask_for_the_bootloader(&self.s, &self.ssh, BOOTLOADER_COMMAND)
    }

    /// In the bootloader: its current slot against the one adb read at the start (equal is the proof the way back changed no slot).
    fn the_bootloader_slot_is_said(&self) -> String {
        let mut c = Command::new(&self.s.fastboot);
        c.args([
            "-s",
            &self.lab.serial().unwrap_or_default(),
            "getvar",
            "current-slot",
        ]);
        let slot = host::run(c).and_then(|said| parse_current_slot(&said.text));
        let start = self
            .before
            .borrow()
            .suffix
            .trim_start_matches('_')
            .to_string();
        match slot {
            Ok(s) if s == start => format!("bootloader reached: fastboot lists the phone, current-slot {s} equals the slot at the start; fastboot reboot follows"),
            Ok(s) if start.is_empty() => format!("bootloader reached: fastboot lists the phone, current-slot {s}; the slot at the start is not known yet; fastboot reboot follows"),
            Ok(s) => format!("bootloader reached: fastboot lists the phone, current-slot {s} is NOT the slot at the start ({start:?}); fastboot reboot follows and the flags are measured"),
            Err(e) => format!("bootloader reached: fastboot lists the phone, its current-slot is not known yet ({e}); fastboot reboot follows"),
        }
    }

    /// The phone is on Android again and the boot chain is read back and equals the stored set (the lab's own scripts).
    pub fn the_boot_chain_is_as_stored(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let after = u.s.out.join("bootchain-after");
            let after = after.to_string_lossy().into_owned();
            let toml = u.s.device_toml();
            u.lab.script(
                "bootchain-readback.sh",
                &[&after],
                &[("DEVICE_TOML", &toml.to_string_lossy())],
            )?;
            let stored = u.s.bootchain.to_string_lossy().into_owned();
            u.lab
                .script("bootchain-readback-check.sh", &[&stored, &after], &[])
                .map(|_| ())
        }
    }

    /// The phone is back on Android (the shared step).
    pub fn android_is_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.android_is_back()
    }

    /// The evidence of a failed check, read FIRST while the phone is still on the image: one read-only ssh command
    /// ([`EVIDENCE_COMMAND`]), kept in the run's folder (serials redacted) and printed into the log. A read that cannot be
    /// made is said as not known yet, never as a failure of the way back.
    pub fn the_evidence_is_read(&self) -> Done {
        match self.ssh(EVIDENCE_COMMAND, 40) {
            Ok(text) if text.contains("---end") => {
                let text = redact(&text);
                println!("the phone is on the image: the evidence of the stop, read-only:\n{text}");
                self.lab.save(EVIDENCE_FILE, &text)
            }
            Ok(_) => {
                println!("NOT KNOWN YET: the evidence read did not finish");
                Ok(())
            }
            Err(e) => {
                println!("NOT KNOWN YET: the evidence could not be read: {e}");
                Ok(())
            }
        }
    }

    /// After the way back, whatever the run measured of the phone's state and did not reach (a failed check stops the run before
    /// its own measures) is measured here: the card, the boot chain and the slots' flags against what was read before. Each is
    /// said; what was not read before is said as not known yet. The first that differs fails this step, after all were measured.
    pub fn the_post_run_is_measured(&self) -> Done {
        if !self.s.out.join("slots-before.txt").exists() {
            println!("NOT KNOWN YET: the run stopped before the boot: nothing was booted, nothing is measured after it");
            return Ok(());
        }
        self.android.is_up_as_root()?;
        if let Ok(suffix) = self.saved(SUFFIX_FILE) {
            self.before.borrow_mut().suffix = suffix.trim().to_string(); // the parent never read the slot itself
        }
        let measured = self.the_post_run_measures();
        let back = self.lab.way_back(); // a measure that went to fastboot and stopped there still ends on Android
        measured?;
        back
    }

    fn the_post_run_measures(&self) -> Done {
        let out = &self.s.out;
        let measures = [
            AfterStop::new(
                "the partition tables and the root partition",
                out.join("card-before.txt"),
                out.join("card-after.txt"),
                Box::new(|| (self.the_card_is_as_before())()),
            ),
            AfterStop::new(
                "the boot chain",
                self.s.bootchain.clone(),
                out.join("bootchain-after"),
                Box::new(|| (self.the_boot_chain_is_as_stored())()),
            ),
            AfterStop::new(
                "both slots' flags",
                out.join("slots-before.txt"),
                out.join("slots-after.txt"),
                Box::new(|| (self.the_slot_flags_are_as_before())()),
            ),
        ];
        let mut differs = Vec::new();
        for m in &measures {
            let what = m.what;
            match measure_status(&m.before, &m.after) {
                Measure::Done => {}
                Measure::NothingBefore => {
                    println!("NOT KNOWN YET: {what}: nothing was read before the boot to equal")
                }
                Measure::Pending => match (m.measure)() {
                    Ok(()) => println!("measured after the stop: {what} are as before"),
                    Err(e) => differs.push(format!("{what}: {e}")),
                },
            }
        }
        if differs.is_empty() {
            Ok(())
        } else {
            Err(differs.join("\n"))
        }
    }

    /// The rw write only: the phone is back on Android, so the previous boot's console is read (`adb root`, then [`crate::rw_write::PSTORE_COMMAND`], read only),
    /// kept in the run's folder and judged ([`crate::rw_write::the_stop_in_the_console`]): a refusal of the init was lost once to the reboot after it
    /// (research walleye-rw-run-1-refused-by-booted-slot). A read that cannot be made is said as not known yet, never as a failure of the way back.
    fn the_stop_is_read_in_the_pstore(&self) {
        if !self.rw || self.rehearsal {
            return;
        }
        let _ = self.android.adb(&["root"]);
        std::thread::sleep(Duration::from_secs(3));
        match self
            .android
            .adb(&["shell", crate::rw_write::PSTORE_COMMAND])
        {
            Ok(text) if !text.contains("Permission denied") => {
                let text = crate::touch::redact(&text);
                println!("the previous boot's console, read from Android:\n{text}");
                if let Err(e) = self.lab.save(crate::rw_write::PSTORE_FILE, &text) {
                    println!("NOT KNOWN YET: the console could not be kept: {e}");
                }
                println!(
                    "  {}",
                    crate::rw_write::the_stop_in_the_console(&text).said()
                );
            }
            Ok(_) => println!("NOT KNOWN YET: the pstore is closed to adb (no root)"),
            Err(e) => println!("NOT KNOWN YET: the pstore could not be read: {e}"),
        }
    }

    /// The way back, the last step: the phone ends on Android whatever stopped the run. On the image it is sent away by ssh (the
    /// forced reboot, then the sysrq trigger), then the shared way back; the image's own watchdog hold resets it in the end anyway.
    /// The watching parent runs it after the child ends, whatever ended it.
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            if std::env::var_os(watch::CHILD).is_some() {
                return Ok(());
            }
            if !u.lab.is_on_android_now() && !u.lab.in_fastboot().unwrap_or(false) {
                // the phone may be on the image: ssh comes up in its own time, so it is tried for as long as a boot takes
                let until = Instant::now() + Duration::from_secs(u.s.device_wait);
                while Instant::now() < until {
                    if u.ssh("true", 8).is_ok() {
                        let _ = u.the_evidence_is_read(); // FIRST, read-only; then the phone is sent away
                        u.settle_rw();
                        if !u.the_bootloader_is_tried() {
                            for command in REBOOT_COMMANDS {
                                let _ = u.ssh(command, 10);
                                if u.lab.android_is_back().is_ok() {
                                    break;
                                }
                            }
                        }
                        break;
                    }
                    std::thread::sleep(Duration::from_secs(3));
                }
            }
            u.lab.way_back()?;
            u.the_stop_is_read_in_the_pstore(); // BEFORE any other reboot; read only
            u.the_post_run_is_measured() // a stop before the run's own measures leaves the state known, not unknown
        }
    }
}

/// What [`SETTLE_COMMAND`] answered, in a line: the remount's rc, whether the root's mount line says `ro`, and how many files the boot wrote.
/// A part that did not print, or printed an error, is said as not known yet - never as zero or as rw.
pub fn settled_said(text: &str) -> String {
    let part = |from: &str, to: &str| -> Option<&str> {
        let a = text.find(from)? + from.len();
        let rest = &text[a..];
        Some(rest.find(to).map_or(rest, |b| &rest[..b]))
    };
    let rc = part("---remount", "---mount")
        .and_then(|r| r.lines().rev().find_map(|l| l.trim().strip_prefix("rc=")))
        .map_or("remount rc: not known yet".to_string(), |c| {
            format!("remount rc={c}")
        });
    let ro = part("---mount", "---written")
        .and_then(|m| m.lines().rev().find(|l| !l.trim().is_empty()))
        .and_then(|l| l.split_whitespace().nth(3))
        .map_or("the root's mount line: not known yet".to_string(), |opts| {
            if opts.split(',').any(|o| o == "ro") {
                "the root is mounted ro".into()
            } else {
                format!("the root is NOT ro ({opts:?})")
            }
        });
    let written = part("---written", "\u{0}")
        .filter(|w| !w.lines().any(|l| l.starts_with("find:")))
        .map_or("files written: not known yet".to_string(), |w| {
            format!(
                "files written since boot: {}",
                w.lines().filter(|l| l.starts_with('/')).count()
            )
        });
    format!("{rc}; {ro}; {written}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::executable;
    use crate::settings::Try;

    const DMESG: &str = "[    2.0] walleye-ufs-root: start, kernel 7.3.0, root=[system_a] running slot suffix=[_b]\n[    9.1] walleye-ufs-root: root is /dev/sda13 (PARTNAME=system_a)\n[    9.9] walleye-ufs-root: handing over to the root's /sbin/init\n";
    const MOUNTS: &str = "overlay / overlay rw,relatime,lowerdir=/lower,upperdir=/ovl/upper,workdir=/ovl/work 0 0\n/dev/sda13 /lower ext4 ro,relatime,noload 0 0\ntmpfs /run tmpfs rw,nosuid 0 0\ndevtmpfs /dev devtmpfs rw 0 0\n";

    #[test]
    fn the_root_the_image_names_is_exactly_one_of_the_two_system_partitions() {
        let img = b"\0\0console=ttyMSM0 walleye.root=system_a\0pad";
        assert_eq!(root_in_image(img).unwrap(), "system_a");
        assert_eq!(
            root_in_image(b"x walleye.root=system_b root=/dev/ram x walleye.root=system_b")
                .unwrap(),
            "system_b"
        );
        assert!(root_in_image(b"nothing here")
            .unwrap_err()
            .contains("no walleye.root="));
        assert!(root_in_image(b"walleye.root=system_a walleye.root=system_b").is_err());
        for foreign in [
            "walleye.root=userdata",
            "walleye.root=",
            "walleye.root=system_c",
        ] {
            assert!(root_in_image(foreign.as_bytes()).is_err(), "{foreign}");
        }
    }

    #[test]
    fn the_disks_come_from_the_count_and_the_root_from_the_slot() {
        assert_eq!(disk_letters(6).unwrap(), vec!['a', 'b', 'c', 'd', 'e', 'f']);
        assert_eq!(disk_letters(1).unwrap(), vec!['a']);
        assert_eq!(disk_letters(26).unwrap().last(), Some(&'z'));
        for bad in [0, 27, 300] {
            assert!(disk_letters(bad).is_err(), "{bad}");
        }
        assert_eq!(root_for("_b\n").unwrap(), "system_a");
        assert_eq!(root_for("a").unwrap(), "system_b");
        for doubt in ["", "_", "_c", "system_a"] {
            assert!(root_for(doubt).is_err(), "{doubt:?}");
        }
    }

    #[test]
    fn the_rehearsal_says_it_boots_nothing_and_needs_no_go() {
        let said = expect_rehearsal(&Settings::stand_in(crate::settings::Try::RootfsBoot));
        assert!(
            said.contains("6 disks") && said.contains("Nothing is booted"),
            "{said}"
        );
        assert!(
            said.contains("no go") || said.contains("go is not needed"),
            "{said}"
        );
    }

    #[test]
    fn the_rehearsal_has_no_step_that_boots_and_its_own_folder() {
        let entry = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../procedures/rehearse-rootfs-boots-on-the-pixel-2.rs"
        ))
        .unwrap();
        for boots in [
            "boot_the_root_image",
            "author_has_said_go",
            "the_phone_is_sent_back",
            "boot_image",
            "the_login_works",
        ] {
            assert!(
                !entry.contains(boots),
                "the rehearsal's entry names {boots}"
            );
        }
        let (boot, rehearsal) = (
            Settings::stand_in(crate::settings::Try::RootfsBoot).out,
            crate::settings::Settings::stand_in(crate::settings::Try::RootfsBoot)
                .out_dir("rootfs-boots-rehearsal"),
        );
        assert_ne!(boot, rehearsal);
    }

    #[test]
    fn the_rw_rehearsal_has_no_step_that_boots_writes_or_sends_away_and_its_own_folder() {
        let entry = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../procedures/rehearse-rw-write-on-the-pixel-2.rs"
        ))
        .unwrap();
        for boots in [
            "boot_the_root_image",
            "author_has_said_go",
            "the_phone_is_sent_back",
            "boot_image",
            "the_login_works",
            "the_bounded_write_is_made",
            "the_test_file_is_removed",
            "the_filesystem_is_checked",
            "rw.way_back()",
        ] {
            assert!(
                !entry.contains(boots),
                "the rw rehearsal's entry names {boots}"
            );
        }
        assert!(entry.contains("rw_write_rehearsal") && entry.contains("rehearsal_way_back"));
        let (write, rehearsal) = (
            Settings::stand_in(Try::RwWrite).out,
            Settings::stand_in(Try::RwWrite).out_dir("rw-write-rehearsal"),
        );
        assert_ne!(write, rehearsal);
    }

    #[test]
    fn the_guard_wants_two_readings_that_agree_and_the_other_slots_system() {
        assert_eq!(guard("_b\n", "b", "system_a").unwrap(), "system_a");
        assert_eq!(guard("_a", "a", "system_b").unwrap(), "system_b");
        // the image's root is the slot Android runs from
        assert!(guard("_b", "b", "system_b")
            .unwrap_err()
            .contains("must be system_a"));
        // the readings disagree, or one is missing
        assert!(guard("_a", "b", "system_a")
            .unwrap_err()
            .contains("disagree"));
        assert!(guard("", "b", "system_a").is_err());
        assert!(guard("_b", "", "system_a").is_err());
    }

    #[test]
    fn the_init_lines_are_read_and_a_failed_open_guard_stops_the_run() {
        let saw = InitSaw::of(DMESG);
        assert_eq!(saw.device.as_deref(), Some("sda13"));
        assert!(saw.matches("system_a", "_b").is_ok());
        // no suffix from the bootloader: the init's guard failed open
        let open = DMESG.replace("suffix=[_b]", "suffix=[]");
        let e = InitSaw::of(&open).matches("system_a", "_b").unwrap_err();
        assert!(e.contains("failed open"), "{e}");
        // another slot than adb read
        assert!(saw
            .matches("system_a", "_a")
            .unwrap_err()
            .contains("adb read"));
        // another root
        assert!(saw.matches("system_b", "_b").is_err());
        // a refusal is said with its reason
        let refused = "walleye-ufs-root: refused: no partition named system_a after 90s";
        let e = InitSaw::of(refused).matches("system_a", "_b").unwrap_err();
        assert!(e.contains("refused") && e.contains("no partition"), "{e}");
        // the init's last line is lost after /dev moved: the log alone no longer decides the hand-over
        let half = DMESG.lines().take(2).collect::<Vec<_>>().join("\n");
        assert!(InitSaw::of(&half).matches("system_a", "_b").is_ok());
        assert!(InitSaw::of("").matches("system_a", "_b").is_err());
    }

    #[test]
    fn only_a_read_only_card_under_an_overlay_passes() {
        assert_eq!(mounts_are_read_only(MOUNTS).unwrap(), "/dev/sda13");
        for (what, bad) in [
            (
                "rw",
                MOUNTS.replace("ro,relatime,noload", "rw,relatime,noload"),
            ),
            ("no noload or norecovery", MOUNTS.replace(",noload", "")),
            ("discard", MOUNTS.replace("noload", "noload,discard")),
            (
                "no overlay",
                MOUNTS.replace("overlay / overlay", "/dev/sda13 / ext4"),
            ),
            (
                "other lower",
                MOUNTS.replace("lowerdir=/lower", "lowerdir=/x"),
            ),
            (
                "a second card mount",
                format!("{MOUNTS}/dev/sdb1 /mnt ext4 rw 0 0\n"),
            ),
            ("not ext4", MOUNTS.replace("ext4", "vfat")),
            ("no lower", MOUNTS.replace(" /lower ", " /elsewhere ")),
        ] {
            assert!(mounts_are_read_only(&bad).is_err(), "{what}");
        }
        // a read-only second mount of the card is no harm
        assert!(mounts_are_read_only(&format!("{MOUNTS}/dev/sdb1 /mnt ext4 ro 0 0\n")).is_ok());
    }

    #[test]
    fn the_gpt_command_reads_a_disk_and_writes_nothing() {
        for letter in disk_letters(6).unwrap() {
            let c = gpt_command(letter).unwrap();
            assert!(c.contains(&format!("if=/dev/block/sd{letter}")), "{c}");
            assert!(!crate::ufs::load::writes_to_the_card(&c), "{c}");
            assert!(Command::new("sh")
                .args(["-n", "-c", &c])
                .status()
                .unwrap()
                .success());
        }
        for bad in ['/', 'A', '0'] {
            assert!(gpt_command(bad).is_err());
        }
    }

    #[test]
    fn the_commands_write_nothing_to_the_card() {
        for c in [READ_COMMAND, EVIDENCE_COMMAND, REBOOT_COMMANDS[0]] {
            assert!(
                Command::new("sh")
                    .args(["-n", "-c", c])
                    .status()
                    .unwrap()
                    .success(),
                "{c}"
            );
            for forbidden in [
                "of=",
                "mkfs",
                "fstrim",
                "blkdiscard",
                "mount ",
                "umount",
                "> /dev/",
            ] {
                assert!(!c.contains(forbidden), "{forbidden} in {c}");
            }
        }
    }

    #[test]
    fn the_key_must_be_private_and_ssh_uses_it_alone() {
        let d = std::env::temp_dir().join(format!("wt-rb-key-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let key = d.join("id");
        std::fs::write(&key, "k").unwrap();
        std::fs::set_permissions(&key, PermissionsExt::from_mode(0o600)).unwrap();
        assert!(identity_is_private(&key).is_ok());
        std::fs::set_permissions(&key, PermissionsExt::from_mode(0o640)).unwrap();
        assert!(identity_is_private(&key).unwrap_err().contains("chmod 600"));
        assert!(identity_is_private(&d).is_err());
        assert!(identity_is_private(&d.join("none")).is_err());
        let a = ssh_args(&key, "root", "172.16.43.1", 22, "uname -r");
        for want in [
            "BatchMode=yes",
            "IdentitiesOnly=yes",
            "UserKnownHostsFile=/dev/null",
            "root@172.16.43.1",
        ] {
            assert!(a.iter().any(|x| x == want), "{want} in {a:?}");
        }
        assert_eq!(a.last().unwrap(), "uname -r");
    }

    #[test]
    fn a_log_without_a_final_newline_does_not_swallow_the_next_marker() {
        // the same shape as the screen's: a file's last line has no newline, so the marker after it would land at that line's end
        let d = std::env::temp_dir().join(format!("wt-evlogs-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("init.log"), "first\nlast line, no newline").unwrap();
        let from = EVIDENCE_COMMAND.find("echo ---init-logs").unwrap();
        let to = EVIDENCE_COMMAND.find("echo ---partnames").unwrap();
        let loop_ = EVIDENCE_COMMAND[from..to].replace(
            "/run/*.log /var/log/*.log /tmp/*.log /*.log",
            &format!("{}/*.log", d.display()),
        );
        let out = Command::new("sh")
            .args(["-c", &format!("{loop_}echo ---partnames; echo ---end")])
            .output()
            .unwrap();
        let parts = sections(&String::from_utf8(out.stdout).unwrap());
        assert!(
            section(&parts, "init-logs")
                .unwrap_or_default()
                .contains("last line, no newline\n"),
            "{parts:?}"
        );
        assert!(section(&parts, "partnames").is_ok(), "{parts:?}");
    }

    #[test]
    fn the_read_is_split_into_its_sections() {
        let parts = sections("junk\n---id\n0\n---kernel\n7.3.0\nmore\n---end\n");
        assert_eq!(section(&parts, "id").unwrap(), "0\n");
        assert_eq!(section(&parts, "kernel").unwrap(), "7.3.0\nmore\n");
        assert!(section(&parts, "mounts").is_err());
    }

    /// A lab whose ssh is a stand-in that answers the one read from a file, as the phone would.
    fn rig(name: &str, read: &str, ssh_exit: i32) -> RootfsBoot {
        let d = std::env::temp_dir().join(format!("wt-rb-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("read.out"), read).unwrap();
        let ssh = d.join("ssh");
        executable(
            &ssh,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/ssh.calls\ncat {d}/read.out\nexit {ssh_exit}\n",
                d = d.display()
            ),
        );
        let key = d.join("key");
        std::fs::write(&key, "k").unwrap();
        std::fs::set_permissions(&key, PermissionsExt::from_mode(0o600)).unwrap();
        let mut s = Settings::stand_in(Try::RootfsBoot);
        s.out = d.join("out");
        s.identity = key;
        s.address = "172.16.43.1".into();
        s.device_wait = 1;
        let mut r = RootfsBoot::with_ssh(s, &ssh.to_string_lossy());
        std::fs::create_dir_all(&r.s.out).unwrap();
        let before = Before {
            suffix: "_b".into(),
            root: "system_a".into(),
            run_image: None,
        };
        *Rc::get_mut(&mut r.before).unwrap().borrow_mut() = before;
        r
    }

    fn read_of(dmesg: &str, mounts: &str, id: &str, net: &str, services: &str) -> String {
        format!("---id\n{id}\n---kernel\n7.3.0\n---cmdline\nx walleye.root=system_a\n---pid1\n/sbin/init \n---mounts\n{mounts}---dmesg\n{dmesg}---services\n{services}---net\n{net}\n---end\n")
    }

    const NET: &str = "5: usb0    inet 172.16.43.1/24 brd 172.16.43.255 scope global usb0";

    #[test]
    fn a_good_read_passes_every_step_after_the_login() {
        let r = rig("good", &read_of(DMESG, MOUNTS, "0", NET, ""), 0);
        (r.the_login_works_and_the_phone_is_read())().unwrap();
        (r.the_phone_is_the_image_we_booted())().unwrap();
        (r.the_init_saw_the_slot_adb_saw())().unwrap();
        (r.the_root_is_mounted_read_only())().unwrap();
        (r.the_kernel_log_is_clean())().unwrap();
        let calls = std::fs::read_to_string(r.s.out.parent().unwrap().join("ssh.calls")).unwrap();
        assert!(
            calls.contains("-i")
                && calls.contains("root@172.16.43.1")
                && calls.contains("BatchMode=yes"),
            "{calls}"
        );
    }

    #[test]
    fn pid1_and_the_lower_layer_are_read_exactly() {
        for ok in [
            "/sbin/init",
            "/sbin/init -q",
            "  /sbin/init \n",
            "/sbin/init\0",
        ] {
            assert!(pid1_is_the_roots_init(ok).is_ok(), "{ok:?}");
        }
        for bad in [
            "init",
            "/bin/busybox init",
            "/bin/busybox\0sh\0/init\0",
            "/init",
            "/sbin/init2",
            "",
        ] {
            assert!(pid1_is_the_roots_init(bad).is_err(), "{bad:?}");
        }
        assert!(lower_is_the_named_card(MOUNTS, "sda13").is_ok());
        for device in ["sda1", "sda", ""] {
            assert!(
                lower_is_the_named_card(MOUNTS, device).is_err(),
                "{device:?}"
            );
        }
    }

    #[test]
    fn the_hand_over_is_proved_by_the_live_state_not_by_the_lost_log_line() {
        let with_pid1 = |name: &str, dmesg: &str, mounts: &str, pid1: &str| {
            let read = read_of(dmesg, mounts, "0", NET, "")
                .replace("---pid1\n/sbin/init \n", &format!("---pid1\n{pid1}\n"));
            let r = rig(name, &read, 0);
            (r.the_login_works_and_the_phone_is_read())().unwrap();
            r
        };
        // the handing-over line missing (lost after /dev moved): passes by the live state
        let lost = DMESG.lines().take(2).collect::<Vec<_>>().join("\n") + "\n";
        let r = with_pid1("lost", &lost, MOUNTS, "/sbin/init ");
        (r.the_init_saw_the_slot_adb_saw())().unwrap();
        // PID 1 is still the initramfs init: fails, with the reading in the message
        let r = with_pid1("initramfs", DMESG, MOUNTS, "/bin/sh /init ");
        let e = (r.the_init_saw_the_slot_adb_saw())().unwrap_err();
        assert!(
            e.contains("/bin/sh") && e.contains("did not hand over"),
            "{e}"
        );
        // PID 1 unread: not known, still a stop
        let r = with_pid1("unread", DMESG, MOUNTS, "");
        assert!((r.the_init_saw_the_slot_adb_saw())()
            .unwrap_err()
            .contains("not known"));
        // the root is not the overlay: fails
        let r = with_pid1(
            "noovl",
            DMESG,
            &MOUNTS.replace("overlay / overlay", "/dev/sda13 / ext4"),
            "/sbin/init ",
        );
        assert!((r.the_init_saw_the_slot_adb_saw())()
            .unwrap_err()
            .contains("overlay"));
        // the lower layer is another partition than the init named: fails
        let r = with_pid1(
            "other",
            DMESG,
            &MOUNTS.replace("/dev/sda13 /lower", "/dev/sda14 /lower"),
            "/sbin/init ",
        );
        assert!((r.the_init_saw_the_slot_adb_saw())()
            .unwrap_err()
            .contains("named"));
    }

    #[test]
    fn every_bad_read_stops_at_the_step_that_sees_it() {
        let case = |name: &str, read: String| {
            let r = rig(name, &read, 0);
            (r.the_login_works_and_the_phone_is_read())().unwrap();
            r
        };
        let r = case("notroot", read_of(DMESG, MOUNTS, "1000", NET, ""));
        assert!((r.the_phone_is_the_image_we_booted())()
            .unwrap_err()
            .contains("not root"));
        let r = case("crashed", read_of(DMESG, MOUNTS, "0", NET, "sshd crashed"));
        assert!((r.the_phone_is_the_image_we_booted())()
            .unwrap_err()
            .contains("crashed"));
        let r = case("nonet", read_of(DMESG, MOUNTS, "0", "no usb0", ""));
        assert!((r.the_phone_is_the_image_we_booted())()
            .unwrap_err()
            .contains("usb0"));
        let r = case(
            "open",
            read_of(
                &DMESG.replace("suffix=[_b]", "suffix=[]"),
                MOUNTS,
                "0",
                NET,
                "",
            ),
        );
        assert!((r.the_init_saw_the_slot_adb_saw())()
            .unwrap_err()
            .contains("failed open"));
        let r = case(
            "rw",
            read_of(
                DMESG,
                &MOUNTS.replace("ro,relatime", "rw,relatime"),
                "0",
                NET,
                "",
            ),
        );
        assert!((r.the_root_is_mounted_read_only())().is_err());
        let purge = format!("{DMESG}[ 12.0] ufshcd: purge refused on this device\n");
        let r = case("purge", read_of(&purge, MOUNTS, "0", NET, ""));
        assert!((r.the_kernel_log_is_clean())()
            .unwrap_err()
            .contains("purge refused"));
        let oops = format!("{DMESG}[ 12.0] Kernel panic - not syncing\n");
        let r = case("oops", read_of(&oops, MOUNTS, "0", NET, ""));
        assert!((r.the_kernel_log_is_clean())().is_err());
    }

    #[test]
    fn the_evidence_is_read_over_ssh_kept_and_never_written_to_the_card() {
        let said =
            "---dmesg-tail\nx\n---init-lines\nwalleye-ufs-root: no card\n---partnames\n---end\n";
        let r = rig("evidence", said, 0);
        r.the_evidence_is_read().unwrap();
        let kept = r.saved(EVIDENCE_FILE).unwrap();
        assert!(kept.contains("walleye-ufs-root: no card"), "{kept}");
        let calls = std::fs::read_to_string(r.s.out.parent().unwrap().join("ssh.calls")).unwrap();
        for part in [
            "dmesg",
            "/proc/mounts",
            "/proc/cmdline",
            "ps;",
            "ls -l /;",
            "PARTNAME",
            "df",
        ] {
            assert!(calls.contains(part), "{part} not asked: {calls}");
        }
    }

    #[test]
    fn a_new_run_forgets_the_files_of_an_earlier_one_and_a_stop_before_the_boot_measures_nothing() {
        let r = rig("forget", "", 0);
        for n in [
            "card-before.txt",
            "read.txt",
            "write.txt",
            "readback.txt",
            "fsck.txt",
            "dmesg-after.txt",
            "remove.txt",
            "card-after.txt",
            "slots-after.txt",
            EVIDENCE_FILE,
        ] {
            std::fs::write(r.s.out.join(n), "old").unwrap();
        }
        std::fs::create_dir_all(r.s.out.join("bootchain-after")).unwrap();
        r.forget_earlier_runs();
        for n in [
            "card-before.txt",
            "read.txt",
            "write.txt",
            "readback.txt",
            "fsck.txt",
            "dmesg-after.txt",
            "remove.txt",
            "card-after.txt",
            "slots-after.txt",
            EVIDENCE_FILE,
            "bootchain-after",
        ] {
            assert!(!r.s.out.join(n).exists(), "{n} kept");
        }
        // no slots-before.txt: nothing was booted, nothing is measured and the phone is not touched
        assert!(r.the_post_run_is_measured().is_ok());
    }

    #[test]
    fn evidence_that_cannot_be_read_is_not_known_yet_and_stops_nothing() {
        let r = rig("noevidence", "", 255);
        assert!(r.the_evidence_is_read().is_ok());
        assert!(r.saved(EVIDENCE_FILE).is_err());
        let r = rig("halfevidence", "---dmesg-tail\nx\n", 0);
        assert!(r.the_evidence_is_read().is_ok());
        assert!(r.saved(EVIDENCE_FILE).is_err());
    }

    #[test]
    fn a_measure_is_made_after_a_stop_only_when_there_is_a_before_and_no_after() {
        let d = std::env::temp_dir().join(format!("wt-rb-measure-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let (before, after) = (d.join("before"), d.join("after"));
        assert_eq!(measure_status(&before, &after), Measure::NothingBefore);
        std::fs::write(&before, "b").unwrap();
        assert_eq!(measure_status(&before, &after), Measure::Pending);
        std::fs::write(&after, "a").unwrap();
        assert_eq!(measure_status(&before, &after), Measure::Done);
    }

    #[test]
    fn a_login_that_never_answers_is_named_after_the_wait() {
        let r = rig("nologin", "", 255);
        let e = (r.the_login_works_and_the_phone_is_read())().unwrap_err();
        assert!(
            e.contains("no ssh login within 1 s as root@172.16.43.1"),
            "{e}"
        );
    }

    #[test]
    fn the_boot_step_holds_its_own_gate() {
        // no guard read yet: nothing is booted
        let r = rig("gate", "", 0);
        *r.before.borrow_mut() = Before::default();
        let e = (r.boot_the_root_image())().unwrap_err();
        assert!(e.contains("slot is not read"), "{e}");
    }

    #[test]
    fn the_boot_step_stops_at_each_gate_before_any_boot() {
        let r = rig("gates", "", 0);
        let d = r.s.out.parent().unwrap().to_path_buf();
        let host = d.join("lab/tests/host");
        std::fs::create_dir_all(&host).unwrap();
        let battery = host.join("battery-ok.sh");
        std::fs::write(&battery, "echo pass battery\nexit 0\n").unwrap();
        let img = d.join("NOT-TO-BOOT-g.img");
        std::fs::write(&img, "x walleye.root=system_a").unwrap();
        let mut s = (*r.s).clone();
        s.lab = d.join("lab");
        s.image = img.clone();
        s.lpoco = "/nonexistent/lpoco-must-not-run".into();
        let boot = |s: &Settings| {
            let r = RootfsBoot::new(s.clone());
            *r.before.borrow_mut() = Before {
                suffix: "_b".into(),
                root: "system_a".into(),
                run_image: None,
            };
            r.boot_the_root_image()()
        };
        // a wrong sha256: refused before the go is even looked at
        s.image_sha256 = "0".repeat(64);
        assert!(boot(&s).unwrap_err().contains("device.toml records"));
        // the right sha256 and no go the lab can read: refused, lpoco never ran
        let sha = Command::new("sha256sum").arg(&img).output().unwrap();
        s.image_sha256 = String::from_utf8_lossy(&sha.stdout)
            .split_whitespace()
            .next()
            .unwrap()
            .to_string();
        let e = boot(&s).unwrap_err();
        assert!(e.contains("no go") || e.contains("\"tad\""), "{e}");
        // a failing battery gate comes first
        std::fs::write(&battery, "echo fail battery\nexit 1\n").unwrap();
        assert!(!boot(&s).unwrap_err().contains("tad"));
    }

    /// A boot image header (version 0) with the command line, then 4 KiB of "kernel" (one page) and one page of footer.
    fn boot_image(cmdline: &str) -> Vec<u8> {
        let mut b = vec![0u8; 12288];
        b[..8].copy_from_slice(b"ANDROID!");
        b[8..12].copy_from_slice(&4096u32.to_le_bytes()); // kernel_size
        b[36..40].copy_from_slice(&4096u32.to_le_bytes()); // page_size
        b[CMDLINE_AT..CMDLINE_AT + cmdline.len()].copy_from_slice(cmdline.as_bytes());
        for (i, x) in b[4096..8192].iter_mut().enumerate() {
            *x = (i % 251) as u8;
        }
        b
    }

    #[test]
    fn the_run_command_line_is_the_declared_one_plus_the_slot_read() {
        assert_eq!(
            run_cmdline("a b walleye.root=system_a", "_b").unwrap(),
            "a b walleye.root=system_a androidboot.slot_suffix=_b"
        );
        for bad in ["", "b", "_c", "_b\n", "system_a"] {
            assert!(
                run_cmdline("a", bad).unwrap_err().contains("not _a or _b"),
                "{bad}"
            );
        }
        let e = run_cmdline("a androidboot.slot_suffix=_a", "_b").unwrap_err();
        assert!(e.contains("already has"), "{e}");
        let e = run_cmdline(&"x".repeat(500), "_b").unwrap_err();
        assert!(e.contains("header's own field"), "{e}");
    }

    #[test]
    fn the_header_gives_its_command_line_and_a_non_image_gives_none() {
        assert_eq!(header_cmdline(&boot_image("a b")).unwrap(), "a b");
        assert!(header_cmdline(b"short").is_err());
        let mut bad = boot_image("a");
        bad[0] = b'X';
        assert!(header_cmdline(&bad)
            .unwrap_err()
            .contains("not a boot image"));
    }

    #[test]
    fn the_made_image_may_differ_in_the_command_line_and_nowhere_else() {
        let declared = boot_image("a walleye.root=system_a");
        let want = run_cmdline("a walleye.root=system_a", "_b").unwrap();
        let made = boot_image(&want);
        assert!(differs_only_in_the_command_line(&declared, &made, &want)
            .unwrap()
            .contains("command line"));
        // a byte of the kernel, of a header word, of the id: refused
        for at in [4096usize, 8191, 8, 20, 600] {
            let mut m = made.clone();
            m[at] ^= 1;
            let e = differs_only_in_the_command_line(&declared, &m, &want).unwrap_err();
            assert!(e.contains("outside its command line"), "{at}: {e}");
        }
        // the footer's digest, signed again by the tool: 32 bytes after the content pass, 33 do not
        let mut signed = made.clone();
        signed[9000..9032].fill(7);
        assert!(differs_only_in_the_command_line(&declared, &signed, &want).is_ok());
        signed[9032] = 7;
        let e = differs_only_in_the_command_line(&declared, &signed, &want).unwrap_err();
        assert!(e.contains("more than the footer's one digest"), "{e}");
        // another command line than the one asked for
        let e = differs_only_in_the_command_line(
            &declared,
            &boot_image("a walleye.root=system_a androidboot.slot_suffix=_a"),
            &want,
        )
        .unwrap_err();
        assert!(e.contains("not ["), "{e}");
        // the same image: no slot was put on it
        let e = differs_only_in_the_command_line(&declared, &declared, "a walleye.root=system_a")
            .unwrap_err();
        assert!(e.contains("no slot was put"), "{e}");
        // another size
        assert!(differs_only_in_the_command_line(&declared, &made[..8000], &want).is_err());
    }

    /// A stand-in for `lpoco-bringup image swap IMG --cmdline TEXT --out OUT`: copies the image and writes the text in the header's command line.
    fn bringup(d: &Path, also_change: Option<usize>) -> String {
        let p = d.join("bringup");
        let touch = also_change.map_or(String::new(), |at| {
            format!("printf 'Z' | dd of=\"$OUT\" bs=1 seek={at} conv=notrunc status=none\n")
        });
        executable(
            &p,
            &format!("#!/bin/sh\nwhile [ $# -gt 0 ]; do case $1 in --cmdline) CL=$2; shift ;; --out) OUT=$2; shift ;; --device) shift ;; image|swap) ;; *) IMG=$1 ;; esac; shift; done\ncp \"$IMG\" \"$OUT\"\nprintf '%s' \"$CL\" | dd of=\"$OUT\" bs=1 seek=64 conv=notrunc status=none\n{touch}"),
        );
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn the_run_image_is_made_by_the_tool_and_kept_with_both_sha256() {
        let r = rig("run-image", "", 0);
        let d = r.s.out.parent().unwrap().to_path_buf();
        let img = d.join("NOT-TO-BOOT-d.img");
        std::fs::write(&img, boot_image("a walleye.root=system_a")).unwrap();
        let mut s = (*r.s).clone();
        s.image = img.clone();
        s.bringup = bringup(&d, None);
        let r = RootfsBoot::new(s.clone());
        *r.before.borrow_mut() = Before {
            suffix: "_b".into(),
            root: "system_a".into(),
            run_image: None,
        };
        (r.the_run_image_is_made())().unwrap();
        let (made, sha) = r.before.borrow().run_image.clone().unwrap();
        assert!(made.ends_with(RUN_IMAGE));
        assert_eq!(
            header_cmdline(&std::fs::read(&made).unwrap()).unwrap(),
            "a walleye.root=system_a androidboot.slot_suffix=_b"
        );
        let kept = std::fs::read_to_string(s.out.join("run-image.txt")).unwrap();
        assert!(
            kept.contains("declared ")
                && kept.contains("made ")
                && kept.contains("androidboot.slot_suffix=_b"),
            "{kept}"
        );
        assert_eq!(
            std::fs::read(&img).unwrap(),
            boot_image("a walleye.root=system_a"),
            "the declared image is untouched"
        );
        // the gate before the boot accepts it, and sees a file changed since
        r.the_made_image_is_this_runs(&made, &sha).unwrap();
        let mut bytes = std::fs::read(&made).unwrap();
        bytes[5000] ^= 1;
        std::fs::write(&made, &bytes).unwrap();
        assert!(r
            .the_made_image_is_this_runs(&made, &sha)
            .unwrap_err()
            .contains("changed since"));
    }

    #[test]
    fn a_tool_that_changes_more_than_the_command_line_makes_no_run_image() {
        let r = rig("run-image-bad", "", 0);
        let d = r.s.out.parent().unwrap().to_path_buf();
        let img = d.join("NOT-TO-BOOT-d.img");
        std::fs::write(&img, boot_image("a walleye.root=system_a")).unwrap();
        let mut s = (*r.s).clone();
        s.image = img;
        s.bringup = bringup(&d, Some(5000));
        let r = RootfsBoot::new(s);
        *r.before.borrow_mut() = Before {
            suffix: "_b".into(),
            root: "system_a".into(),
            run_image: None,
        };
        let e = (r.the_run_image_is_made())().unwrap_err();
        assert!(e.contains("outside its command line"), "{e}");
        assert!(r.before.borrow().run_image.is_none());
        // no slot read, no guard: no image
        let r2 = RootfsBoot::new((*r.s).clone());
        assert!((r2.the_run_image_is_made())()
            .unwrap_err()
            .contains("slot is not read"));
    }

    #[test]
    fn the_boot_step_wants_the_run_image_made() {
        let r = rig("no-run-image", "", 0);
        let d = r.s.out.parent().unwrap().to_path_buf();
        let host = d.join("lab/tests/host");
        std::fs::create_dir_all(&host).unwrap();
        std::fs::write(host.join("battery-ok.sh"), "echo pass battery\nexit 0\n").unwrap();
        // the stand-in's tad answers a go for this image's sha256
        let img = d.join("NOT-TO-BOOT-g.img");
        std::fs::write(&img, boot_image("a walleye.root=system_a")).unwrap();
        let sha = host::sha256_of(&img).unwrap();
        let mut s = (*r.s).clone();
        s.lab = d.join("lab");
        s.image = img;
        s.image_sha256 = sha.clone();
        s.lpoco = "/nonexistent/lpoco-must-not-run".into();
        let e = {
            // the go is read by the real `tad`, which is not told of this item: the run stops there, before any image is looked at
            let rb = RootfsBoot::new(s);
            *rb.before.borrow_mut() = Before {
                suffix: "_b".into(),
                root: "system_a".into(),
                run_image: None,
            };
            rb.boot_the_root_image()().unwrap_err()
        };
        assert!(
            e.contains("no go") || e.contains("tad") || e.contains("run image is not made"),
            "{e}"
        );
    }

    #[test]
    fn the_made_image_and_not_the_declared_one_is_what_boots() {
        let r = rig("boots-made", "", 0);
        let d = r.s.out.parent().unwrap().to_path_buf();
        let img = d.join("NOT-TO-BOOT-d.img");
        std::fs::write(&img, boot_image("a walleye.root=system_a")).unwrap();
        let lpoco = d.join("lpoco");
        executable(
            &lpoco,
            &format!("#!/bin/sh\necho \"$@\" >> {}/lpoco.calls\n", d.display()),
        );
        let mut s = (*r.s).clone();
        s.image = img.clone();
        s.bringup = bringup(&d, None);
        s.lpoco = lpoco.to_string_lossy().into_owned();
        let r = RootfsBoot::new(s);
        *r.before.borrow_mut() = Before {
            suffix: "_b".into(),
            root: "system_a".into(),
            run_image: None,
        };
        // not made: nothing boots
        assert!(r.boot_the_made_image().unwrap_err().contains("not made"));
        assert!(!d.join("lpoco.calls").exists());
        (r.the_run_image_is_made())().unwrap();
        r.boot_the_made_image().unwrap();
        let calls = std::fs::read_to_string(d.join("lpoco.calls")).unwrap();
        assert!(calls.trim_end().ends_with(RUN_IMAGE), "{calls}");
        assert!(!calls.contains("NOT-TO-BOOT-d.img"), "{calls}");
        // a file changed after it was checked: nothing more boots
        let (made, _) = r.before.borrow().run_image.clone().unwrap();
        let mut b = std::fs::read(&made).unwrap();
        b[5000] ^= 1;
        std::fs::write(&made, b).unwrap();
        assert!(r
            .boot_the_made_image()
            .unwrap_err()
            .contains("changed since"));
        assert_eq!(
            std::fs::read_to_string(d.join("lpoco.calls"))
                .unwrap()
                .lines()
                .count(),
            1
        );
    }

    #[test]
    fn a_header_of_another_version_or_a_dirty_command_line_field_is_refused() {
        let declared = boot_image("a");
        let want = run_cmdline("a", "_b").unwrap();
        let mut v1 = boot_image(&want);
        v1[40] = 1;
        assert!(header_cmdline(&v1).unwrap_err().contains("version 1"));
        let mut dirty = boot_image(&want);
        dirty[CMDLINE_AT + 400] = b'x'; // after the line's NUL
        assert!(differs_only_in_the_command_line(&declared, &dirty, &want)
            .unwrap_err()
            .contains("after its end"));
        let mut extra = boot_image(&want);
        extra[EXTRA_AT] = b'x';
        assert!(differs_only_in_the_command_line(&declared, &extra, &want).is_err());
        // 32 bytes spread over the footer are not one digest
        let mut spread = boot_image(&want);
        spread[9000] = 1;
        spread[9100] = 1;
        assert!(differs_only_in_the_command_line(&declared, &spread, &want)
            .unwrap_err()
            .contains("one digest"));
    }
    #[test]
    fn the_image_must_be_a_declared_not_to_boot_image_with_one_root() {
        let r = rig("image", "", 0);
        let mut s = (*r.s).clone();
        let d = r.s.out.parent().unwrap().to_path_buf();
        let img = d.join("plain.img");
        std::fs::write(&img, "x walleye.root=system_a").unwrap();
        let sha = {
            let o = Command::new("sha256sum").arg(&img).output().unwrap();
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .unwrap()
                .to_string()
        };
        s.image = img.clone();
        s.image_sha256 = sha.clone();
        let named = RootfsBoot::new(s.clone());
        let e = (named.the_image_is_the_declared_one())().unwrap_err();
        assert!(e.contains("NOT-TO-BOOT"), "{e}");
        let ok = d.join("NOT-TO-BOOT-x.img");
        std::fs::copy(&img, &ok).unwrap();
        s.image = ok;
        assert!((RootfsBoot::new(s.clone()).the_image_is_the_declared_one())().is_ok());
        // a stale build file and an image named for ensure: the step asks ensure, and when that cannot mend it the run stops here, before any phone step
        let failing = d.join("lpoco-bringup-fails");
        crate::fixture::executable(&failing, "#!/bin/sh\necho no recipe >&2\nexit 1\n");
        let mut stale = s.clone();
        stale.image_sha256 = "0".repeat(64);
        stale.image_name = "x".into();
        stale.bringup = failing.to_string_lossy().into_owned();
        let e = (RootfsBoot::new(stale).the_image_is_the_declared_one())().unwrap_err();
        assert!(
            e.contains("device.toml records") && e.contains("no recipe"),
            "{e}"
        );
        s.image_sha256 = "0".repeat(64);
        assert!((RootfsBoot::new(s).the_image_is_the_declared_one())().is_err());
    }

    /// The rw write's try on a stand-in ssh that answers every command with `answer` (the stand-in is the program, not text a phone printed:
    /// these tests prove the wiring of the steps; what the phone prints is judged by rw_write's own tests on captured text).
    fn rw_rig(name: &str, answer: &str) -> RootfsBoot {
        let d = std::env::temp_dir().join(format!("wt-rw-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("answer.out"), answer).unwrap();
        let ssh = d.join("ssh");
        executable(
            &ssh,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/ssh.calls\ncat {d}/answer.out\n",
                d = d.display()
            ),
        );
        let key = d.join("key");
        std::fs::write(&key, "k").unwrap();
        std::fs::set_permissions(&key, PermissionsExt::from_mode(0o600)).unwrap();
        let mut s = Settings::stand_in(Try::RwWrite);
        s.out = d.join("out");
        s.identity = key;
        s.address = "172.16.43.1".into();
        let mut r = RootfsBoot::rw_write(s);
        r.ssh = ssh.to_string_lossy().into_owned();
        std::fs::create_dir_all(&r.s.out).unwrap();
        *Rc::get_mut(&mut r.before).unwrap().borrow_mut() = Before {
            suffix: "_b".into(),
            root: "system_a".into(),
            run_image: None,
        };
        r
    }

    #[test]
    fn the_image_must_carry_walleye_rw_in_the_rw_try_and_must_not_in_the_read_only_one() {
        let r = rw_rig("image", "");
        let d = r.s.out.parent().unwrap().to_path_buf();
        let put = |name: &str, cmdline: &str| {
            let img = d.join(format!("NOT-TO-BOOT-{name}.img"));
            std::fs::write(&img, boot_image(cmdline)).unwrap();
            let sha = String::from_utf8_lossy(
                &Command::new("sha256sum").arg(&img).output().unwrap().stdout,
            )
            .split_whitespace()
            .next()
            .unwrap()
            .to_string();
            (img, sha)
        };
        let with = put("rw", "walleye.root=system_a walleye.booted-slot walleye.rw");
        let without = put("ro", "walleye.root=system_a walleye.booted-slot");
        let judge = |rw: bool, (img, sha): &(std::path::PathBuf, String)| {
            let mut s = (*r.s).clone();
            s.image = img.clone();
            s.image_sha256 = sha.clone();
            let t = if rw {
                RootfsBoot::rw_write(s)
            } else {
                RootfsBoot::new(s)
            };
            (t.the_image_is_the_declared_one())()
        };
        assert!(judge(true, &with).is_ok());
        assert!(judge(true, &without)
            .unwrap_err()
            .contains("read-only image"));
        assert!(judge(false, &without).is_ok());
        assert!(judge(false, &with)
            .unwrap_err()
            .contains("this try is the read-only boot"));
    }

    #[test]
    fn the_write_steps_keep_what_the_phone_said_and_judge_its_codes() {
        let ok = rw_rig("write-ok", "16+0 records in\n16+0 records out\nrc=0\n");
        (ok.the_bounded_write_is_made())().unwrap();
        assert!(ok.saved("write.txt").unwrap().contains("rc=0"));
        let calls = std::fs::read_to_string(ok.s.out.parent().unwrap().join("ssh.calls")).unwrap();
        assert!(
            calls.contains("dd if=/dev/zero of=/walleye-rw-write.test bs=1M count=16 conv=fsync"),
            "{calls}"
        );
        let bad = rw_rig("write-bad", "dd: error writing\nrc=1\n");
        assert!((bad.the_bounded_write_is_made())()
            .unwrap_err()
            .contains("rc=1"));
        assert!(bad.saved("write.txt").unwrap().contains("error writing"));
        let gone = rw_rig("remove", "rc=0\n");
        (gone.the_test_file_is_removed())().unwrap();
        assert!((rw_rig("remove-bad", "rc=1\n").the_test_file_is_removed())().is_err());
        // the read-back and the fsck judge what the phone's commands print: the size, the zeros, no refusal marker
        let size = 16 * 1024 * 1024;
        (rw_rig("rb", &format!("---size\n{size}\n---nonzero\n0\n---end\n"))
            .the_file_reads_back_whole())()
        .unwrap();
        assert!((rw_rig(
            "rb-bad",
            &format!("---size\n{size}\n---nonzero\n7\n---end\n")
        )
        .the_file_reads_back_whole())()
        .is_err());
    }

    #[test]
    fn the_rw_go_gate_wants_the_full_sha256_of_the_declared_image() {
        // the image the manifest declares for the rw write has a 64-hex sha256, and a go row that names a short form or none is refused
        let raw: toml::Value = toml::from_str(
            &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../device.toml"))
                .unwrap(),
        )
        .unwrap();
        let sha = raw["images"]["ufs-root-booted-slot-production-rw"]["sha256"]
            .as_str()
            .unwrap();
        assert_eq!(sha.len(), 64);
        let now = crate::ufs::now_epoch().unwrap();
        let go = |note: &str| {
            format!("backlog,\"{{\"\"note\"\":\"\"{note}\"\",\"\"go_until\"\":\"\"2099-01-01T00:00:00+00:00\"\"}}\"")
        };
        crate::ufs::go_decision(
            &go(&format!("go for {sha}")),
            now,
            sha,
            "walleye-rw-write-run",
        )
        .unwrap();
        assert!(crate::ufs::go_decision(
            &go(&format!("go for {}", &sha[..8])),
            now,
            sha,
            "walleye-rw-write-run"
        )
        .unwrap_err()
        .contains("64 hex"));
        assert!(
            crate::ufs::go_decision(&go("go"), now, sha, "walleye-rw-write-run")
                .unwrap_err()
                .contains("found none")
        );
        assert!(crate::ufs::go_decision(
            &go(&format!("go for {sha}")).replace("2099", "2001"),
            now,
            sha,
            "walleye-rw-write-run"
        )
        .is_err());
        // the entry's boot step holds the same gate: with no tad row for the go item it stops before any boot
        let r = rw_rig("go", "");
        assert!((r.author_has_said_go())().is_err());
    }
}

/// Checks 15, 16 and 17 of the root boot (the login is root's on the image, the init saw the slot and handed over, the card is read-only under the
/// overlay) run OFFLINE against what the phone really printed in runs 3 and 4 of the stage 1 boot, through the same step functions the procedure
/// calls. The fixtures are the keeper's files, copied byte for byte from `~/lab-material/walleye/rootfs-boots-20261123-054445` (run 3) and
/// `.../rootfs-boots-20261123-062054` (run 4): `read.txt` is the one read after the login, `evidence.txt` the read of the stop. The reading of
/// PID 1 is the one of run 4's read.txt (`---pid1`); run 3's read.txt predates that section, so its PID 1 is taken from the `ps` of run 3's own
/// evidence.txt (the line of PID 1), and the test says so.
#[cfg(test)]
mod real_runs {
    use super::*;
    use crate::fixture::executable;
    use crate::settings::Try;
    use std::os::unix::fs::PermissionsExt;

    const RUN3_READ: &str = include_str!("../fixtures/rootfs-boots-run3-read.txt");
    const RUN3_EVIDENCE: &str = include_str!("../fixtures/rootfs-boots-run3-evidence.txt");
    const RUN4_READ: &str = include_str!("../fixtures/rootfs-boots-run4-read.txt");

    /// Run 4's captured read as it is: it has the `---pid1` section.
    fn run4() -> String {
        assert!(RUN4_READ.contains("\n---pid1\n"));
        RUN4_READ.to_string()
    }

    /// Run 3's captured read with the PID 1 section its read did not yet have, taken from the ps of its own evidence (PID 1's command).
    fn run3() -> String {
        assert!(
            !RUN3_READ.contains("---pid1"),
            "run 3's read had no pid1 section"
        );
        let ps = sections(RUN3_EVIDENCE)
            .into_iter()
            .find(|(n, _)| n == "ps")
            .expect("run 3's evidence has a ps section")
            .1;
        let pid1 = ps
            .lines()
            .find(|l| l.split_whitespace().next() == Some("1"))
            .expect("run 3's ps lists PID 1");
        let command = pid1
            .split_whitespace()
            .skip(3)
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(command, "/sbin/init");
        RUN3_READ.replacen(
            "---mounts\n",
            &format!("---pid1\n{command} \n---mounts\n"),
            1,
        )
    }

    fn rig(name: &str, read: &str) -> RootfsBoot {
        let d = std::env::temp_dir().join(format!("wt-rb-real-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("read.out"), read).unwrap();
        let ssh = d.join("ssh");
        executable(&ssh, &format!("#!/bin/sh\ncat {}/read.out\n", d.display()));
        let key = d.join("key");
        std::fs::write(&key, "k").unwrap();
        std::fs::set_permissions(&key, PermissionsExt::from_mode(0o600)).unwrap();
        let mut s = Settings::stand_in(Try::RootfsBoot);
        s.out = d.join("out");
        s.identity = key;
        s.address = "172.16.43.1".into();
        s.device_wait = 1;
        let mut r = RootfsBoot::with_ssh(s, &ssh.to_string_lossy());
        std::fs::create_dir_all(&r.s.out).unwrap();
        // what the runs read before the boot: adb's suffix _b, so the root is system_a
        *Rc::get_mut(&mut r.before).unwrap().borrow_mut() = Before {
            suffix: "_b".into(),
            root: "system_a".into(),
            run_image: None,
        };
        r
    }

    /// Checks 15, 16, 17 in the procedure's order: the first error stops, as a check does.
    fn checks_15_to_17(r: &RootfsBoot) -> Result<(), String> {
        (r.the_login_works_and_the_phone_is_read())()?;
        (r.the_phone_is_the_image_we_booted())()
            .map_err(|e| format!("check 15 (root's login, network, services): {e}"))?;
        (r.the_init_saw_the_slot_adb_saw())()
            .map_err(|e| format!("check 16 (the init saw the slot, handed over): {e}"))?;
        (r.the_root_is_mounted_read_only())()
            .map_err(|e| format!("check 17 (the card is read-only under the overlay): {e}"))
    }

    #[test]
    fn the_state_run_4_captured_passes_checks_15_to_17() {
        checks_15_to_17(&rig("run4", &run4())).unwrap();
    }

    #[test]
    fn the_state_run_3_captured_passes_checks_15_to_17_now_that_the_check_reads_the_live_state() {
        checks_15_to_17(&rig("run3", &run3())).unwrap();
    }

    #[test]
    fn the_phones_own_mount_line_is_read_only_whatever_the_word_for_noload() {
        let line = "/dev/sda22 /lower ext4 ro,relatime,norecovery 0 0";
        assert!(RUN4_READ.contains(line) && RUN3_READ.contains(line));
        let mounts = format!("overlay / overlay rw,relatime,lowerdir=/lower,upperdir=/ovl/upper,workdir=/ovl/work,uuid=on 0 0\n{line}\n");
        assert_eq!(mounts_are_read_only(&mounts).unwrap(), "/dev/sda22");
        assert!(mounts_are_read_only(&mounts.replace("norecovery", "noload")).is_ok());
        for (what, bad) in [
            ("rw", mounts.replace("ro,relatime", "rw,relatime")),
            ("no ro", mounts.replace("ro,relatime,", "")),
            ("neither word", mounts.replace(",norecovery", "")),
            (
                "discard",
                mounts.replace("norecovery", "norecovery,discard"),
            ),
        ] {
            assert!(mounts_are_read_only(&bad).is_err(), "{what}");
        }
    }

    #[test]
    fn a_captured_state_that_is_wrong_fails_with_its_reading_in_the_error() {
        // the card mounted read-write
        let rw = run4().replace(
            "/dev/sda22 /lower ext4 ro,relatime,norecovery",
            "/dev/sda22 /lower ext4 rw,relatime,norecovery",
        );
        let e = checks_15_to_17(&rig("rw", &rw)).unwrap_err();
        assert!(
            (e.contains("check 16") || e.contains("check 17"))
                && e.contains("rw,relatime,norecovery"),
            "{e}"
        );
        // PID 1 is the initramfs init: the hand-over did not happen
        let pid1 = run4().replacen("\n---pid1\n/sbin/init", "\n---pid1\n/init", 1);
        let e = checks_15_to_17(&rig("pid1", &pid1)).unwrap_err();
        assert!(e.contains("check 16") && e.contains("\"/init\""), "{e}");
        // the overlay's lower layer is another partition than the one the init named
        let other = run4().replace("/dev/sda22 /lower", "/dev/sda23 /lower");
        let e = checks_15_to_17(&rig("other", &other)).unwrap_err();
        assert!(
            e.contains("check 16") && e.contains("/dev/sda23") && e.contains("/dev/sda22"),
            "{e}"
        );
        // no overlay as /
        let flat = run4().replace("overlay / overlay", "/dev/sda22 / ext4");
        let e = checks_15_to_17(&rig("flat", &flat)).unwrap_err();
        assert!(e.contains("overlay"), "{e}");
    }
}

/// The way back through the bootloader (item walleye-way-back-reboot-bootloader-first), both outcomes, with stand-ins for ssh and
/// fastboot that answer from files: fastboot's getvar answer is the text the phone printed on a recorded visit
/// (tests fixtures/fastboot-getvar-all-on-b.txt, a copy of research/walleye-first-fastboot-boot/2026-11-17-on-b/getvar-before.txt).
/// NOT KNOWN YET: the text of `fastboot devices` was never recorded in a keeper's log; the stand-in lists the serial, which is all
/// the lab's check reads of it.
#[cfg(test)]
mod bootloader_way_back {
    use super::*;
    use crate::fixture::executable;
    use crate::settings::Try;
    use std::os::unix::fs::PermissionsExt;

    const GETVAR: &str = include_str!("../fixtures/fastboot-getvar-all-on-b.txt");

    /// A phone that, when asked `walleye-reboot --command bootloader`, shows up in fastboot (`listed`) or does not.
    fn rig(name: &str, listed: bool, suffix: &str) -> (RootfsBoot, std::path::PathBuf) {
        let d = std::env::temp_dir().join(format!("wt-bl-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("getvar.out"), GETVAR).unwrap();
        let ssh = d.join("ssh");
        executable(
            &ssh,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/ssh.calls\ncase \"$*\" in *'cat > /run/walleye/walleye-reboot'*) cat > {d}/pushed-run;; *' reboot') {on};; *'cat > /tmp/walleye-reboot'*) cat > {d}/pushed;; *'/tmp/walleye-reboot --command bootloader'*) {on};; esac\nexit 0\n",
                d = d.display(),
                on = if listed { format!("touch {}/in-fastboot", d.display()) } else { ":".to_string() }
            ),
        );
        let fastboot = d.join("fastboot");
        executable(
            &fastboot,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/fastboot.calls\ncase \"$*\" in\n*devices*) [ -f {d}/in-fastboot ] && echo SER1 fastboot;;\n*getvar*) cat {d}/getvar.out >&2;;\nesac\nexit 0\n",
                d = d.display()
            ),
        );
        let key = d.join("key");
        std::fs::write(&key, "k").unwrap();
        std::fs::set_permissions(&key, PermissionsExt::from_mode(0o600)).unwrap();
        let mut s = Settings::stand_in(Try::RootfsBoot);
        s.out = d.join("out");
        s.lab = d.clone();
        s.identity = key;
        let tool = d.join("walleye-reboot");
        std::fs::write(&tool, b"\x7fELF stand-in for the static tool").unwrap();
        s.reboot_tool_sha256 = host::sha256_of(&tool).unwrap();
        s.reboot_tool = tool;
        s.address = "172.16.43.1".into();
        s.fastboot = fastboot.to_string_lossy().into_owned();
        std::fs::write(d.join("device.toml"), "[connection]\nserial = \"SER1\"\n").unwrap();
        let mut r = RootfsBoot::with_ssh(s, &ssh.to_string_lossy());
        r.bootloader_wait = Duration::from_secs(1);
        std::fs::create_dir_all(&r.s.out).unwrap();
        r.before.borrow_mut().suffix = suffix.into();
        (r, d)
    }

    fn calls(d: &Path, name: &str) -> String {
        std::fs::read_to_string(d.join(name)).unwrap_or_default()
    }

    #[test]
    fn a_phone_that_reaches_the_bootloader_on_the_same_slot_is_said_so_and_sent_no_forced_reboot() {
        let (r, d) = rig("reached", true, "_b");
        assert!((r.the_phone_is_sent_back())().is_ok());
        let said = r.saved(BOOTLOADER_FILE).unwrap();
        assert!(
            said.contains("bootloader reached") && said.contains("current-slot b equals"),
            "{said}"
        );
        let ssh = calls(&d, "ssh.calls");
        assert!(
            ssh.contains("walleye-reboot --command bootloader") && !ssh.contains("reboot -f"),
            "{ssh}"
        );
        // only reads: no set_active, flash, erase or oem
        let fb = calls(&d, "fastboot.calls");
        for word in ["set_active", "flash", "erase", "oem"] {
            assert!(!fb.contains(word), "{word}: {fb}");
        }
        // run 2 (2026-11-24, ~/lab-material/walleye/rw-write-20261124-150514/bootloader-way-back.txt): the bootloader was reached and the watched child sent nothing more, so Android was not back at the next check; the child now leaves the bootloader itself
        assert!(
            fb.lines().any(|l| l.ends_with(" reboot")),
            "no fastboot reboot after the bootloader was reached: {fb}"
        );
    }

    #[test]
    fn a_settle_that_printed_nothing_is_said_as_not_known_yet() {
        let said = settled_said("");
        assert_eq!(said.matches("not known yet").count(), 3, "{said}");
    }

    #[test]
    fn a_settle_is_read_by_its_markers_and_a_find_error_is_not_zero_files() {
        // the markers are SETTLE_COMMAND's own; the mount line is the form of /proc/mounts (fields: device, point, type, options)
        let ok = "---remount\nrc=0\n---mount\n/dev/sda22 / ext4 ro,relatime 0 0\n---written\n/var/log/a\n/etc/b\n";
        assert_eq!(
            settled_said(ok),
            "remount rc=0; the root is mounted ro; files written since boot: 2"
        );
        let rw = "---remount\nmount: busy\nrc=1\n---mount\n/dev/sda22 / ext4 rw,relatime 0 0\n---written\nfind: bad -mmin\n";
        let said = settled_said(rw);
        assert!(
            said.contains("rc=1")
                && said.contains("NOT ro")
                && said.contains("files written: not known yet"),
            "{said}"
        );
        let said = settled_said("---remount\nrc=0\n---mount\n---written\n");
        assert!(said.contains("mount line: not known yet"), "{said}");
    }

    #[test]
    fn a_bootloader_on_another_slot_is_said_not_the_slot_at_the_start() {
        let (r, _) = rig("otherslot", true, "_a");
        assert!(r.the_bootloader_is_tried());
        assert!(r
            .saved(BOOTLOADER_FILE)
            .unwrap()
            .contains("NOT the slot at the start"));
    }

    #[test]
    fn a_phone_that_is_not_listed_says_bootloader_not_reached_and_the_forced_reboot_follows() {
        let (r, d) = rig("notreached", false, "_b");
        assert!((r.the_phone_is_sent_back())().is_ok());
        let said = r.saved(BOOTLOADER_FILE).unwrap();
        assert!(
            said.contains("bootloader not reached") && said.contains("fastboot lists no phone"),
            "{said}"
        );
        let ssh = calls(&d, "ssh.calls");
        assert!(
            !calls(&d, "fastboot.calls")
                .lines()
                .any(|l| l.ends_with(" reboot")),
            "a bootloader that was not reached must not be sent fastboot reboot"
        );
        assert!(
            ssh.contains("walleye-reboot --command bootloader") && ssh.contains("reboot -f"),
            "{ssh}"
        );
        assert!(!calls(&d, "fastboot.calls").contains("getvar"));
    }

    #[test]
    fn the_rw_write_ends_through_init_with_the_tool_and_marker_on_run_and_no_remount() {
        let (mut r, d) = rig("cleanend", true, "_b");
        r.rw = true;
        assert!((r.the_phone_is_sent_back())().is_ok());
        let said = r.saved(CLEAN_END_FILE).unwrap();
        assert!(said.starts_with("clean end reached"), "{said}");
        let ssh = calls(&d, "ssh.calls");
        assert_eq!(
            std::fs::read(d.join("pushed-run")).unwrap(),
            std::fs::read(&r.s.reboot_tool).unwrap()
        );
        let (push, end) = (
            ssh.find("cat > /run/walleye/walleye-reboot").unwrap(),
            ssh.rfind(" reboot\n").unwrap(),
        );
        assert!(push < end, "{ssh}");
        assert!(ssh.contains(": > /run/walleye/to-bootloader"), "{ssh}");
        // nothing remounts in the live session, nothing is forced, the old tool path is not used
        for word in ["remount", "reboot -f", "sysrq", "/tmp/walleye-reboot"] {
            assert!(!ssh.contains(word), "{word}: {ssh}");
        }
        assert!(!d.join("out").join(BOOTLOADER_FILE).exists());
        let fb = calls(&d, "fastboot.calls");
        assert!(fb.lines().any(|l| l.ends_with(" reboot")), "{fb}");
        for word in ["set_active", "flash", "erase", "oem"] {
            assert!(!fb.contains(word), "{word}: {fb}");
        }
    }

    #[test]
    fn a_clean_end_that_does_not_reach_fastboot_leaves_the_old_way_back_to_run() {
        let (mut r, d) = rig("cleanend-no", false, "_b");
        r.rw = true;
        assert!((r.the_phone_is_sent_back())().is_ok());
        let said = r.saved(CLEAN_END_FILE).unwrap();
        assert!(
            said.contains("clean end not reached") && said.contains("the old way back runs"),
            "{said}"
        );
        let ssh = calls(&d, "ssh.calls");
        assert!(
            ssh.contains("remount,ro") && ssh.contains("reboot -f"),
            "{ssh}"
        );
        assert!(!calls(&d, "fastboot.calls")
            .lines()
            .any(|l| l.ends_with(" reboot")));
    }

    #[test]
    fn a_read_only_or_rehearsed_run_never_tries_the_clean_end() {
        let (r, d) = rig("cleanend-ro", true, "_b");
        assert!(!r.the_clean_end_is_tried());
        let (mut r2, _) = rig("cleanend-re", true, "_b");
        r2.rw = true;
        r2.rehearsal = true;
        assert!(!r2.the_clean_end_is_tried());
        assert!(!calls(&d, "ssh.calls").contains("/run/walleye"));
    }

    #[test]
    fn the_tool_is_sent_to_the_tmpfs_whole_and_run_from_there_and_nothing_else_is_written() {
        let (r, d) = rig("sent", true, "_b");
        assert!(r.the_bootloader_is_tried());
        assert_eq!(
            std::fs::read(d.join("pushed")).unwrap(),
            std::fs::read(&r.s.reboot_tool).unwrap()
        );
        let ssh = calls(&d, "ssh.calls");
        let (push, run) = (
            ssh.find("cat > /tmp/walleye-reboot").unwrap(),
            ssh.find("/tmp/walleye-reboot --command bootloader")
                .unwrap(),
        );
        assert!(push < run, "{ssh}");
        // the card is never named: only /tmp, after the phone's address
        for line in ssh.lines() {
            let said = line.split_once("root@172.16.43.1 ").unwrap().1;
            assert!(
                said.contains("/tmp/walleye-reboot") && !said.contains("/dev/"),
                "{said}"
            );
        }
    }

    #[test]
    fn the_sentence_for_a_reached_bootloader_is_the_one_rerun_7_printed() {
        let (r, _d) = rig("fixture", true, "_b");
        assert!(r.the_bootloader_is_tried());
        assert_eq!(
            r.saved(BOOTLOADER_FILE).unwrap().trim(),
            include_str!("../fixtures/bootloader-way-back-reached.txt").trim()
        );
    }
    #[test]
    fn a_tool_that_is_not_the_declared_one_is_never_sent() {
        let (mut r, d) = rig("wrongtool", true, "_b");
        let mut s = (*r.s).clone();
        s.reboot_tool_sha256 = "0000".into();
        r.s = std::rc::Rc::new(s);
        assert!(!r.the_bootloader_is_tried());
        let said = r.saved(BOOTLOADER_FILE).unwrap();
        assert!(said.contains("not the declared one"), "{said}");
        assert_eq!(calls(&d, "ssh.calls"), "");
    }

    #[test]
    fn the_bootloader_is_tried_once_a_run() {
        let (r, d) = rig("once", false, "_b");
        assert!(!r.the_bootloader_is_tried());
        assert!(!r.the_bootloader_is_tried());
        assert_eq!(
            calls(&d, "ssh.calls")
                .matches("walleye-reboot --command bootloader")
                .count(),
            1
        );
        r.forget_earlier_runs();
        assert!(!r.s.out.join(BOOTLOADER_FILE).exists());
    }
}
