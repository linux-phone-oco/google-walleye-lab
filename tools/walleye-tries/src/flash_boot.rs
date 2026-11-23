// SPDX-License-Identifier: MIT OR Apache-2.0
//! Install stage 2 as ONE command (item walleye-install-flash-boot-procedure; decisions 2026-11-22-walleye-author-research-then-proceed,
//! 2026-11-19-walleye-attended-tries-one-command; research walleye-install-stage2-android-verdict): our boot image flashed to the boot
//! partition of the slot Android is NOT on, read back, that slot made active by the bootloader's own `fastboot set_active` (the call
//! Android's OTA makes, a GPT write by the bootloader's code, not ours), the phone observed coming up from the card on ssh, and the way
//! back `fastboot set_active <Android's slot>` made and checked in the same run. Each method is one step of
//! procedures/install-boot-on-the-pixel-2.rs (or of its rehearsal), returns what that step runs and fails with what it saw.
//!
//! What it writes: ONE `fastboot flash boot_<other>` (the name is built from the slot read two ways, checked against the allow-list of
//! device.toml, never typed), and the bootloader's `set_active` of the slot just flashed and read back, and `set_active` of Android's slot
//! as the way back. No bootloader partition (xbl, abl ...), no GPT bytes of ours, no erase, no purge. The GPT is READ before, and after
//! the switch, and the only bytes that may differ are the slot attribute bits, the A/B type-GUID swap and the CRCs (`gpt_switch_only`).
//! The condition that cannot be proved on the host is said as not known yet: how our kernel reaches the bootloader again (the
//! `to_bootloader` command of device.toml) - when it does not, the one hand step is said with the exact fastboot command.

use crate::android::{plain_path, Android};
use crate::install::{other_slot, parse_current_slot, slots_agree};
use crate::lab::Lab;
use crate::rootfs_boot::{
    disk_letters, identity_is_private, mounts_are_read_only, pid1_is_the_roots_init, sections,
    ssh_args,
};
use crate::settings::Settings;
use crate::ufs::{go_decision, now_epoch};
use crate::{host, watch};
use std::collections::BTreeMap;
use std::process::Command;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The marker files of the run's folder the watching parent reads: the slot Android runs from, the slot made active, and that Android's slot was made active again.
pub const ANDROID_SLOT_FILE: &str = "android-slot.txt";
pub const SWITCHED_FILE: &str = "switched-to.txt";
pub const RESTORED_FILE: &str = "restored.txt";

// ---------------------------------------------------------------------------------------------------------------------
// what may be flashed, and with what
// ---------------------------------------------------------------------------------------------------------------------

/// The one partition the flash names: `boot_` and the slot Android is NOT on, and only when device.toml's allow-list holds it. Everything else is refused:
/// the boot partition of Android's own slot, any other name, a list that names a partition that is not `boot_a` or `boot_b`.
pub fn flash_target(allow: &str, android_slot: &str) -> Result<String, String> {
    let other = other_slot(android_slot.trim())?;
    let allowed: Vec<&str> = allow.split_whitespace().collect();
    if let Some(odd) = allowed.iter().find(|n| **n != "boot_a" && **n != "boot_b") {
        return Err(format!(
            "flash_allow in device.toml names {odd:?}: only boot_a and boot_b may be on it"
        ));
    }
    let target = format!("boot_{other}");
    if target == format!("boot_{}", android_slot.trim()) {
        return Err(format!(
            "{target} is the boot partition of the slot Android runs from: never flashed"
        ));
    }
    if !allowed.contains(&target.as_str()) {
        return Err(format!(
            "{target} is not on device.toml's flash_allow ({allow:?})"
        ));
    }
    Ok(target)
}

/// The one flash: `fastboot -s SERIAL flash boot_X IMAGE`, nothing else on the line.
pub fn flash_args(serial: &str, target: &str, image: &str) -> Result<Vec<String>, String> {
    if target != "boot_a" && target != "boot_b" {
        return Err(format!(
            "{target:?} is not boot_a or boot_b: nothing is flashed"
        ));
    }
    Ok(["-s", serial, "flash", target, plain_path(image)?]
        .into_iter()
        .map(str::to_string)
        .collect())
}

/// The bootloader's slot switch: `fastboot -s SERIAL set_active a|b`.
pub fn set_active_args(serial: &str, slot: &str) -> Result<Vec<String>, String> {
    match slot {
        "a" | "b" => Ok(["-s", serial, "set_active", slot]
            .into_iter()
            .map(str::to_string)
            .collect()),
        other => Err(format!("{other:?} is not slot a or b: nothing is switched")),
    }
}

/// The slot a boot partition name belongs to.
pub fn slot_of_boot(partition: &str) -> Result<&str, String> {
    match partition {
        "boot_a" => Ok("a"),
        "boot_b" => Ok("b"),
        other => Err(format!("{other:?} is not boot_a or boot_b")),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// the slot flags
// ---------------------------------------------------------------------------------------------------------------------

/// The lines of [`host::slot_lines`] as a map: `slot-successful:a` -> `yes`, `current-slot` -> `b` (the value is what follows the last colon).
pub fn flags_of(lines: &str) -> BTreeMap<String, String> {
    lines
        .lines()
        .filter_map(|l| l.trim().rsplit_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

/// After `set_active TO` (from `FROM`): the flags changed exactly as the bootloader's own switch describes (research walleye-install-stage2-android-verdict): `current-slot` is TO, TO is
/// not successful, not unbootable and has a retry count of 1 to 7; every other flag - FROM's too - is what it was, and no flag appeared or went.
pub fn switch_is_the_abls(before: &str, after: &str, from: &str, to: &str) -> Result<(), String> {
    let (b, a) = (flags_of(before), flags_of(after));
    if b.keys().ne(a.keys()) {
        return Err(format!(
            "the flags' names changed:\n{before}\nthen\n{after}"
        ));
    }
    if b.get("current-slot").map(String::as_str) != Some(from) {
        return Err(format!(
            "before the switch the current slot was not {from}: {before}"
        ));
    }
    let own = [
        "current-slot".to_string(),
        format!("slot-successful:{to}"),
        format!("slot-unbootable:{to}"),
        format!("slot-retry-count:{to}"),
    ];
    for (k, v) in &b {
        if !own.contains(k) && a[k] != *v {
            return Err(format!("{k} changed from {v} to {}: the switch changes only slot {to}'s own flags and the current slot", a[k]));
        }
    }
    let want = |k: &str, w: &str| -> Done {
        match a.get(k).map(String::as_str) {
            Some(v) if v == w => Ok(()),
            other => Err(format!(
                "after the switch {k} is {other:?}, the bootloader's switch leaves {w:?}"
            )),
        }
    };
    want("current-slot", to)?;
    want(&format!("slot-successful:{to}"), "no")?;
    want(&format!("slot-unbootable:{to}"), "no")?;
    let retry = a
        .get(&format!("slot-retry-count:{to}"))
        .and_then(|v| v.parse::<u8>().ok())
        .ok_or("no retry count for the new slot")?;
    if !(1..=7).contains(&retry) {
        return Err(format!(
            "the new slot's retry count is {retry}: the switch leaves 1 to 7"
        ));
    }
    Ok(())
}

/// After the way back: Android's slot is the current one and is not unbootable, and no flag appeared or went. (The flags of its boot counting, successful and the retry count, are the bootloader's and Android's own to set.)
pub fn way_back_restored(before: &str, after: &str, android: &str) -> Result<(), String> {
    let (b, a) = (flags_of(before), flags_of(after));
    if b.keys().ne(a.keys()) {
        return Err(format!(
            "the flags' names changed:\n{before}\nthen\n{after}"
        ));
    }
    if a.get("current-slot").map(String::as_str) != Some(android) {
        return Err(format!(
            "after the way back the current slot is not {android}: {after}"
        ));
    }
    if a.get(&format!("slot-unbootable:{android}"))
        .map(String::as_str)
        != Some("no")
    {
        return Err(format!(
            "Android's slot {android} is unbootable after the way back: {after}"
        ));
    }
    let retry = a
        .get(&format!("slot-retry-count:{android}"))
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(0);
    if a.get(&format!("slot-successful:{android}"))
        .map(String::as_str)
        != Some("yes")
        && retry == 0
    {
        return Err(format!(
            "Android's slot {android} is neither successful nor has tries left after the way back: {after}"
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------------------
// the GPT, read before and after: the bootloader's switch may change only the slot bits, the A/B type GUIDs and the CRCs
// ---------------------------------------------------------------------------------------------------------------------

const GB: usize = 4096;
const HEAD_BLOCKS: usize = 6;
const TAIL_BLOCKS: usize = 5;
/// The bytes of one disk's dump: its first six blocks (the protective MBR, the header, 16 KiB of entries), then its last five (16 KiB of entries, the backup header).
pub const DUMP_LEN: usize = (HEAD_BLOCKS + TAIL_BLOCKS) * GB;
const ENTRIES: usize = 128;
const ENTRY: usize = 128;
const P_HDR: usize = GB;
const P_ENT: usize = 2 * GB;
const B_ENT: usize = HEAD_BLOCKS * GB;
const B_HDR: usize = (HEAD_BLOCKS + 4) * GB;

/// The command that prints one disk's dump as hex: the same blocks `rootfs_boot::gpt_command` hashes, kept whole. `dev` is the node prefix on the phone (`/dev/block/sd` on Android, `/dev/sd` on our root).
pub fn gpt_dump_command(dev: &str, letter: char) -> Result<String, String> {
    if !letter.is_ascii_lowercase() {
        return Err(format!("{letter:?} is not a disk letter (a to z)"));
    }
    let d = format!("{}{letter}", plain_path(dev)?);
    Ok(format!(
        "sz=$(cat /sys/block/sd{letter}/size); [ \"${{sz:-0}}\" -gt 64 ] || exit 1; (dd if={d} bs={GB} count={HEAD_BLOCKS} 2>/dev/null; dd if={d} bs={GB} skip=$((sz/8-{TAIL_BLOCKS})) count={TAIL_BLOCKS} 2>/dev/null) | od -An -v -tx1 | tr -d ' \\n'"
    ))
}

/// The bytes of a hex dump; a dump that is not hex or not a whole dump is an error (an empty read is never "unchanged").
pub fn dump_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex: String = hex.split_whitespace().collect();
    if !hex.len().is_multiple_of(2) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("the GPT dump is not hex".into());
    }
    let bytes: Vec<u8> = (0..hex.len() / 2)
        .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap_or(0))
        .collect();
    if bytes.len() != DUMP_LEN {
        return Err(format!(
            "the GPT dump is {} bytes, a whole one is {DUMP_LEN}",
            bytes.len()
        ));
    }
    Ok(bytes)
}

/// CRC-32 (IEEE, as the GPT uses it).
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn le32(d: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([d[at], d[at + 1], d[at + 2], d[at + 3]])
}

/// The dump is a GPT as the bootloader writes it: both headers say "EFI PART", 128 entries of 128 bytes, and the header and entries CRCs are right.
pub fn gpt_is_whole(d: &[u8]) -> Result<(), String> {
    for (which, hdr, ent) in [("primary", P_HDR, P_ENT), ("backup", B_HDR, B_ENT)] {
        if &d[hdr..hdr + 8] != b"EFI PART" {
            return Err(format!("the {which} GPT header has no EFI PART signature"));
        }
        if le32(d, hdr + 80) as usize != ENTRIES || le32(d, hdr + 84) as usize != ENTRY {
            return Err(format!(
                "the {which} GPT header does not say {ENTRIES} entries of {ENTRY} bytes"
            ));
        }
        let size = le32(d, hdr + 12) as usize;
        if !(92..=GB).contains(&size) {
            return Err(format!(
                "the {which} GPT header size {size} is not 92 to {GB}"
            ));
        }
        let mut h = d[hdr..hdr + size].to_vec();
        h[16..20].fill(0);
        if crc32(&h) != le32(d, hdr + 16) {
            return Err(format!("the {which} GPT header CRC is wrong"));
        }
        if crc32(&d[ent..ent + ENTRIES * ENTRY]) != le32(d, hdr + 88) {
            return Err(format!("the {which} GPT entries CRC is wrong"));
        }
    }
    Ok(())
}

fn entry_name(d: &[u8], at: usize) -> String {
    let units: Vec<u16> = d[at + 56..at + ENTRY]
        .chunks(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|u| *u != 0)
        .collect();
    String::from_utf16_lossy(&units)
}

/// The partner of an A/B partition name (`boot_a` <-> `boot_b`); None for a name without a slot suffix.
fn partner(name: &str) -> Option<String> {
    let (stem, slot) = name.rsplit_once('_')?;
    match slot {
        "a" => Some(format!("{stem}_b")),
        "b" => Some(format!("{stem}_a")),
        _ => None,
    }
}
/// What the bootloader's slot switch may do to one disk, and nothing more (the verdict's conditions 3): both dumps are whole GPTs (right signatures, right CRCs); every byte that differs lies in a
/// CRC field of a header, in the type GUID (bytes 0-15) of an entry, or in byte 6 of an entry's attributes (bits 48-55 of the GPT: the priority, active, retry, successful and unbootable bits of the
/// slot); attribute bytes change only on A/B entries; a type GUID that changed is the GUID its A/B partner had and the partner's changed to this entry's old one (a swap, both ways); the same holds for
/// the backup copy of the entries on its own; and a primary copy that agreed with the backup still does. Returns what changed, counted, for the log.
pub fn gpt_switch_only(before: &[u8], after: &[u8]) -> Result<String, String> {
    if before.len() != DUMP_LEN || after.len() != DUMP_LEN {
        return Err("a GPT dump is not whole".into());
    }
    gpt_is_whole(before).map_err(|e| format!("before: {e}"))?;
    gpt_is_whole(after).map_err(|e| format!("after: {e}"))?;
    let allowed = |at: usize| -> bool {
        for hdr in [P_HDR, B_HDR] {
            if (hdr + 16..hdr + 20).contains(&at) || (hdr + 88..hdr + 92).contains(&at) {
                return true;
            }
        }
        for base in [P_ENT, B_ENT] {
            if (base..base + ENTRIES * ENTRY).contains(&at) {
                let inside = (at - base) % ENTRY;
                return inside < 16 || inside == 54;
            }
        }
        false
    };
    if let Some(at) = (0..DUMP_LEN).find(|&i| before[i] != after[i] && !allowed(i)) {
        return Err(format!("byte {at} of the dump changed and it is not a slot bit, a type GUID or a CRC (block {}, offset {})", at / GB, at % GB));
    }
    let whole = |d: &[u8], base: usize| d[base..base + ENTRIES * ENTRY].to_vec();
    if whole(after, P_ENT) != whole(after, B_ENT) && whole(before, P_ENT) == whole(before, B_ENT) {
        return Err("the primary and the backup entries agreed before and disagree now".into());
    }
    let (mut attrs, mut swaps) = (0usize, 0usize);
    for (copy, base) in [("primary", P_ENT), ("backup", B_ENT)] {
        let names: Vec<String> = (0..ENTRIES)
            .map(|i| entry_name(before, base + i * ENTRY))
            .collect();
        for i in 0..ENTRIES {
            let at = base + i * ENTRY;
            let name = &names[i];
            let twin = partner(name).and_then(|p| names.iter().position(|n| *n == p));
            if before[at + 54] != after[at + 54] {
                if twin.is_none() {
                    return Err(format!(
                        "{copy}: the slot bits of {name:?} changed and it has no A/B partner"
                    ));
                }
                if copy == "primary" {
                    attrs += 1;
                }
            }
            if before[at..at + 16] != after[at..at + 16] {
                let twin = twin.ok_or_else(|| {
                    format!("{copy}: the type GUID of {name:?} changed and it has no A/B partner")
                })?;
                let tat = base + twin * ENTRY;
                if after[at..at + 16] != before[tat..tat + 16]
                    || after[tat..tat + 16] != before[at..at + 16]
                {
                    return Err(format!(
                        "{copy}: the type GUID of {name} changed and it is not a swap with its partner's"
                    ));
                }
                if copy == "primary" {
                    swaps += 1;
                }
            }
        }
    }
    Ok(format!("{attrs} entries' slot bits and {swaps} type GUIDs changed, the CRCs are right, nothing else"))
}

// ---------------------------------------------------------------------------------------------------------------------
// the image
// ---------------------------------------------------------------------------------------------------------------------

/// The init of the image refuses the slot it is booted from (the stage 1 root init does: it mounts the system partition of the OTHER slot, so a boot from the slot's own boot
/// partition, whose bootloader gives `androidboot.slot_suffix` = that slot, would refuse the root it was flashed for).
pub fn init_refuses_its_own_slot(init: &str) -> bool {
    init.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .any(|l| l.contains("the slot Android runs from"))
}

/// The line an init carries when it is made for stage 2: it mounts the system partition of the slot it is booted from. The gate wants this declared, not only the old refusal gone
/// (an init whose refusal is worded differently would pass a gate that only looks for the old words).
pub const BOOTED_SLOT_MARK: &str = "walleye-init: mounts-the-booted-slot";

/// The init may be flashed for stage 2: it does not refuse its own slot and it declares that it mounts the booted slot's system partition.
pub fn init_accepts_the_booted_slot(init: &str) -> Result<(), String> {
    if init_refuses_its_own_slot(init) {
        return Err("the image's init refuses the slot Android runs from (it mounts the OTHER slot's system partition): booted from its own boot partition it would refuse its root. Stage 2 needs the init that mounts the booted slot's own system partition - not flashed".into());
    }
    if !init
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .any(|l| l.contains(BOOTED_SLOT_MARK))
    {
        return Err(format!("the image's init does not say {BOOTED_SLOT_MARK:?} on a code line (a comment is not enough): that it mounts the booted slot's own system partition is not shown - not flashed"));
    }
    Ok(())
}

/// What the author is told before the command, from the settings.
pub fn expect(s: &Settings) -> String {
    format!(
        "One command, about 15 minutes: our boot image is flashed to the boot partition of the slot Android is NOT on (read two ways, built from device.toml's list {:?}), read back, and that slot is made active by the bootloader's own set_active - the call Android's OTA makes, a write to the GPT by the bootloader, with the slot flags and the GPT of its {} disks read before and after. The phone then boots from the card and the lab logs in; then the way back, set_active of Android's slot, is made in the same run and the phone ends on Android. A stop at any step runs the way back.",
        s.flash_allow, s.luns
    )
}

/// What the author is told before the rehearsal.
pub fn expect_rehearsal(s: &Settings) -> String {
    format!(
        "The rehearsal of stage 2, about 5 minutes: every step of the flash except the flash and the set_active - the image checked, the slot read two ways, the flash target built and its command shown, the slot flags and the GPT of {} disks read twice and equal, the boot chain as stored - and nothing written. set_active to the slot already active is not written: whether the bootloader takes it as a no-change write is not known yet. The phone ends on Android.",
        s.luns
    )
}

// ---------------------------------------------------------------------------------------------------------------------
// the steps
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct FlashBoot {
    s: Rc<Settings>,
    lab: Lab,
    android: Android,
    /// the ssh program (`ssh`; a test names a stand-in)
    ssh: String,
    /// the tad program (`tad`; a test names a stand-in): the go is read from the item's row
    tad: String,
}

impl FlashBoot {
    pub fn new(s: Settings) -> FlashBoot {
        FlashBoot::with_ssh(s, "ssh")
    }

    pub fn with_ssh(s: Settings, ssh: &str) -> FlashBoot {
        FlashBoot::with_programs(s, ssh, "tad")
    }

    pub fn with_programs(s: Settings, ssh: &str, tad: &str) -> FlashBoot {
        let s = Rc::new(s);
        let lab = Lab::new(s.clone());
        FlashBoot {
            android: Android::new(s.clone(), lab.clone()),
            lab,
            s,
            ssh: ssh.to_string(),
            tad: tad.to_string(),
        }
    }

    fn saved(&self, name: &str) -> Result<String, String> {
        std::fs::read_to_string(self.s.out.join(name)).map_err(|e| format!("{name}: {e}"))
    }

    fn marked(&self, name: &str) -> Option<String> {
        self.saved(name)
            .ok()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
    }

    /// The slot Android runs from, as the run read it (kept in the folder: the watching parent reads it too).
    fn android_slot(&self) -> Result<String, String> {
        self.marked(ANDROID_SLOT_FILE)
            .ok_or_else(|| "the slot Android runs from is not read yet: nothing is written".into())
    }

    fn target(&self) -> Result<String, String> {
        flash_target(&self.s.flash_allow, &self.android_slot()?)
    }

    fn by_name_of(&self, name: &str) -> String {
        format!("{}/{name}", self.s.by_name.trim_end_matches('/'))
    }

    fn fastboot(&self, args: &[String]) -> Result<String, String> {
        let mut c = Command::new(&self.s.fastboot);
        c.args(args);
        let said = host::run(c)?;
        if said.ok {
            Ok(said.text)
        } else {
            Err(format!("fastboot {}: {}", args.join(" "), said.text))
        }
    }

    fn fastboot_getvar(&self, var: &str) -> Result<String, String> {
        self.fastboot(&["-s".into(), self.lab.serial()?, "getvar".into(), var.into()])
    }

    fn ssh(&self, command: &str, seconds: u64) -> Result<String, String> {
        let mut c = Command::new("timeout");
        c.args([
            "--kill-after=5".to_string(),
            seconds.to_string(),
            self.ssh.clone(),
        ])
        .args(ssh_args(
            &self.s.identity,
            &self.s.ssh_user,
            &self.s.address,
            self.s.ssh_port,
            command,
        ));
        let said = host::run(c)?;
        if said.ok {
            Ok(said.text)
        } else {
            Err(format!("ssh: {}", host::last_line(&said.text)))
        }
    }

    /// The battery is above the lab's minimum (the shared step).
    pub fn battery_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.battery_is_good()
    }

    /// The image is the declared one: its sha256, a NOT-TO-BOOT name, the size of the 32 MiB boot partitions, its AVB footer verifies, and the root it names is the system partition of
    /// the slot it is flashed to (checked again at the flash, with the slot read).
    pub fn the_image_is_the_declared_one(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            if !u.s.image.to_string_lossy().contains("NOT-TO-BOOT") {
                return Err(format!("{} is not under a NOT-TO-BOOT name: a UFS-on image never lives where a queue looks", u.s.image.display()));
            }
            let (backup_boot, size) = {
                let f = u.s.backup.join("boot_a.img");
                (
                    f.clone(),
                    std::fs::metadata(&f)
                        .map_err(|e| format!("{}: {e}", f.display()))?
                        .len(),
                )
            };
            let have = std::fs::metadata(&u.s.image)
                .map_err(|e| format!("{}: {e}", u.s.image.display()))?
                .len();
            if have != size {
                return Err(format!("the image is {have} bytes, the boot partition's backup {} is {size}: a whole partition is flashed and read back", backup_boot.display()));
            }
            let mut c = Command::new("avbtool");
            c.args(["verify_image", "--image"]).arg(&u.s.image);
            let said = host::run(c)?;
            if !said.ok {
                return Err(format!(
                    "the image's AVB footer does not verify: {}",
                    host::last_line(&said.text)
                ));
            }
            let bytes =
                std::fs::read(&u.s.image).map_err(|e| format!("{}: {e}", u.s.image.display()))?;
            println!(
                "  the image names its root: {}",
                crate::rootfs_boot::root_in_image(&bytes)?
            );
            Ok(())
        }
    }

    /// The init inside the image accepts being booted from the slot it is flashed to: the stage 1 root init refuses exactly that, so this gate stops a flash that could only boot to a refusal.
    pub fn the_init_accepts_the_slot_it_is_booted_from(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let d = u.s.out.join("image-unpacked");
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).map_err(|e| format!("{}: {e}", d.display()))?;
            let mut c = Command::new(&u.s.bringup);
            c.args(["image", "unpack"])
                .arg(&u.s.image)
                .arg("--out")
                .arg(&d);
            let said = host::run(c)?;
            if !said.ok {
                return Err(format!("the image could not be unpacked: {}", said.text));
            }
            let init = crate::flash_boot::init_of_ramdisk(&d.join("ramdisk"))?;
            init_accepts_the_booted_slot(&init)
        }
    }

    /// The stored boot chain is whole and the key the lab logs in with is private.
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

    pub fn author_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect(&self.s);
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    pub fn rehearsal_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect_rehearsal(&self.s);
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    /// The author's go covers this image now (the coordinator's go on the item, naming the image's sha256).
    pub fn go_is_open(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.go_now()
    }

    /// The go, asked at the start and again just before the flash.
    fn go_now(&self) -> Done {
        let u = self;
        {
            let mut c = Command::new(&u.tad);
            c.args(["q", "row", "--id", &u.s.go_item, "--format", "csv"])
                .current_dir(&u.s.lab);
            let said = host::run(c)?;
            if !said.ok {
                return Err(format!(
                    "no go: the item {} could not be read: {}",
                    u.s.go_item, said.text
                ));
            }
            let now = now_epoch().ok_or("no go: the clock could not be read")?;
            go_decision(&said.text, now, &u.s.image_sha256)
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

    /// adb says which slot Android runs from; kept, and the earlier runs of the day forgotten.
    pub fn the_slot_is_read_by_adb(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let suffix = u.android.slot_suffix()?;
            let slot = suffix.trim().strip_prefix('_').unwrap_or("").to_string();
            other_slot(&slot)?;
            if u.marked(SWITCHED_FILE).is_some() && u.marked(RESTORED_FILE).is_none() {
                return Err(format!(
                    "an earlier run of this day switched to a slot and never switched back: the phone is on Android now, so it was put back by hand; remove {} and {} and run again",
                    u.s.out.join(SWITCHED_FILE).display(),
                    u.s.out.join(ANDROID_SLOT_FILE).display()
                ));
            }
            // the folder is cleared whole: files and folders (the boot chain read-back refuses a folder with files)
            if let Ok(entries) = std::fs::read_dir(&u.s.out) {
                for e in entries.flatten() {
                    let p = e.path();
                    let _ = if p.is_dir() {
                        std::fs::remove_dir_all(&p)
                    } else {
                        std::fs::remove_file(&p)
                    };
                }
            }
            u.lab.save(ANDROID_SLOT_FILE, &slot)?;
            println!("  adb: slot suffix {}", suffix.trim());
            Ok(())
        }
    }

    /// The boot partition about to be overwritten, read now from Android, is what the stored backup recorded: the flash can be undone. The partition not flashed, its dtbo and vbmeta, and the
    /// flashed slot's dtbo and vbmeta are hashed too, kept for the end.
    pub fn the_slot_partitions_are_read_before(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let target = u.target()?;
            let sums = std::fs::read_to_string(u.s.backup.join("sha256sums"))
                .map_err(|e| format!("backup: {e}"))?;
            let want = crate::install::recorded_sha256(&sums, &format!("{target}.img"))?;
            let now = u.android.sha256(&u.by_name_of(&target), None)?;
            if now != want {
                return Err(format!("{target} read now is {now}, the stored backup says {want}: the flash could not be undone"));
            }
            u.read_parts("parts-before.txt")?;
            println!("  {target} is its backup ({want})");
            Ok(())
        }
    }

    fn read_parts(&self, name: &str) -> Done {
        let mut text = String::new();
        for p in [
            "boot_a", "boot_b", "dtbo_a", "dtbo_b", "vbmeta_a", "vbmeta_b",
        ] {
            text.push_str(&format!(
                "{p} {}\n",
                self.android.sha256(&self.by_name_of(p), None)?
            ));
        }
        self.lab.save(name, text.trim_end())
    }

    /// The GPT of every disk, read from Android whole, kept.
    fn read_gpts_android(&self, name: &str) -> Done {
        let mut text = String::new();
        for letter in disk_letters(self.s.luns)? {
            let said = self
                .android
                .adb(&["shell", &gpt_dump_command("/dev/block/sd", letter)?])?;
            dump_bytes(&said)
                .and_then(|d| gpt_is_whole(&d))
                .map_err(|e| format!("sd{letter}: {e}"))?;
            text.push_str(&format!(
                "sd{letter} {}\n",
                said.split_whitespace().collect::<String>()
            ));
        }
        self.lab.save(name, text.trim_end())
    }

    pub fn the_gpt_is_read_before(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.read_gpts_android("gpt-before.txt")?;
            println!(
                "  {} disks' GPT kept, each whole with the right CRCs",
                u.s.luns
            );
            Ok(())
        }
    }

    /// The rehearsal's second read: the GPT again, equal to the first (a bootloader that is not asked to switch changes nothing).
    pub fn the_gpt_reads_the_same_again(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.read_gpts_android("gpt-again.txt")?;
            let (a, b) = (u.saved("gpt-before.txt")?, u.saved("gpt-again.txt")?);
            if a != b {
                return Err("the GPT read twice from Android is not the same".into());
            }
            for line in a.lines() {
                let (disk, hex) = line.split_once(' ').ok_or("a GPT line has no disk")?;
                let d = dump_bytes(hex).map_err(|e| format!("{disk}: {e}"))?;
                gpt_switch_only(&d, &d).map_err(|e| format!("{disk}: {e}"))?;
            }
            Ok(())
        }
    }

    /// The phone is in fastboot (the shared step).
    pub fn phone_is_in_fastboot(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            watch::phone_is_touched();
            lab.phone_is_in_fastboot()
        }
    }

    /// Fastboot's current slot is read now, equals adb's, and the flash target is built from it and the allow-list. Shown, not yet used.
    pub fn the_guard_holds(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let slot = parse_current_slot(&u.fastboot_getvar("current-slot")?)?;
            let adb = format!("_{}", u.android_slot()?);
            slots_agree(&adb, &slot)?;
            let target = flash_target(&u.s.flash_allow, &slot)?;
            u.the_root_is_the_flashed_slots_own(&slot)?;
            let image = u.s.image.to_string_lossy().into_owned();
            let args = flash_args("SERIAL", &target, &image)?;
            println!(
                "  Android runs from slot {slot}; the one flash would be: fastboot {}",
                args.join(" ")
            );
            Ok(())
        }
    }

    /// The root the image names (`walleye.root=`) is the system partition of the slot the image is flashed to - the slot Android is NOT on - and not Android's own.
    fn the_root_is_the_flashed_slots_own(&self, android_slot: &str) -> Done {
        let bytes =
            std::fs::read(&self.s.image).map_err(|e| format!("{}: {e}", self.s.image.display()))?;
        let root = crate::rootfs_boot::root_in_image(&bytes)?;
        let want = crate::rootfs_boot::root_for(android_slot)?;
        if root == want {
            Ok(())
        } else {
            Err(format!("the image names {root} as its root, the slot it is flashed to has {want}: Android runs from slot {android_slot}, nothing is flashed"))
        }
    }

    fn flags(&self, name: &str) -> Done {
        let flags = host::slot_flags(&self.s.fastboot, &self.lab.serial()?)?;
        self.lab.save(name, &flags)
    }

    /// The slots' flags, read in fastboot, kept.
    pub fn the_slot_flags_are_read(&self, file: &'static str) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.flags(file)
    }

    /// The rehearsal: the flags read again equal the first (nothing was asked of the bootloader).
    pub fn the_slot_flags_are_the_same(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.flags("slots-again.txt")?;
            let (a, b) = (u.saved("slots-before.txt")?, u.saved("slots-again.txt")?);
            if a == b {
                Ok(())
            } else {
                Err(format!(
                    "the slot flags changed with nothing asked:\n{a}\nthen\n{b}"
                ))
            }
        }
    }

    /// THE FLASH: one `fastboot flash boot_<other> IMAGE`. The target is built here from the slot read and the allow-list; the author's go is checked just before.
    pub fn the_boot_image_is_flashed(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let target = u.target()?;
            let slot = parse_current_slot(&u.fastboot_getvar("current-slot")?)?;
            slots_agree(&format!("_{}", u.android_slot()?), &slot)?;
            u.go_now()?;
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            u.the_root_is_the_flashed_slots_own(&slot)?;
            let image = u.s.image.to_string_lossy().into_owned();
            let args = flash_args(&u.lab.serial()?, &target, &image)?;
            u.lab
                .save("flashed.txt", &format!("{target} {}", u.s.image_sha256))?;
            u.fastboot(&args).map(|_| ())
        }
    }

    /// Back on Android (the active slot is still Android's): the boot partition flashed reads back as the image, whole; the partition not flashed and the dtbo and vbmeta of both are as before.
    pub fn the_partition_reads_back_as_the_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let target = u.target()?;
            let got = u.android.sha256(&u.by_name_of(&target), None)?;
            if got != u.s.image_sha256.to_lowercase() {
                return Err(format!(
                    "{target} reads back as {got}, the image is {}",
                    u.s.image_sha256
                ));
            }
            u.read_parts("parts-readback.txt")?;
            let (a, b) = (u.saved("parts-before.txt")?, u.saved("parts-readback.txt")?);
            let changed: Vec<String> = a
                .lines()
                .zip(b.lines())
                .filter(|(x, y)| x != y)
                .map(|(x, _)| x.split_whitespace().next().unwrap_or("?").to_string())
                .collect();
            if changed != [target.clone()] {
                return Err(format!("the partitions that changed with the flash are {changed:?}, expected only {target}"));
            }
            println!("  {target} reads back as the image; no other boot, dtbo or vbmeta partition changed");
            Ok(())
        }
    }

    /// The flash changed no slot flag.
    pub fn the_flash_changed_no_flag(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.flags("slots-after-flash.txt")?;
            let (a, b) = (
                u.saved("slots-before.txt")?,
                u.saved("slots-after-flash.txt")?,
            );
            if a == b {
                Ok(())
            } else {
                Err(format!("the flash changed the slot flags:\n{a}\nthen\n{b}"))
            }
        }
    }

    /// THE SWITCH: `fastboot set_active <slot just flashed and read back>`, and only that slot. The intent is written first, so a stop in the middle still runs the way back.
    pub fn the_slot_is_made_active(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let target = u.target()?;
            if u.marked("parts-readback.txt").is_none() {
                return Err("the flash is not read back: nothing is switched".into());
            }
            let slot = slot_of_boot(&target)?.to_string();
            let now = parse_current_slot(&u.fastboot_getvar("current-slot")?)?;
            if now != u.android_slot()? {
                return Err(format!(
                    "fastboot's current slot is {now}, not Android's {}: nothing is switched",
                    u.android_slot()?
                ));
            }
            u.lab.save(SWITCHED_FILE, &slot)?;
            u.fastboot(&set_active_args(&u.lab.serial()?, &slot)?)
                .map(|_| ())
        }
    }

    /// After the switch, in fastboot: the flags changed as the bootloader's own switch describes.
    pub fn the_flags_are_the_switch_and_nothing_else(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.flags("slots-after-switch.txt")?;
            let to = slot_of_boot(&u.target()?)?.to_string();
            switch_is_the_abls(
                &u.saved("slots-before.txt")?,
                &u.saved("slots-after-switch.txt")?,
                &u.android_slot()?,
                &to,
            )
        }
    }

    /// The phone boots from the card (`fastboot reboot`: the bootloader boots the active slot) and the lab logs in by ssh: root's login on our image, booted with the slot suffix of the slot
    /// made active, PID 1 the root's /sbin/init, the card read-only under the overlay. The GPT of every disk is read from there and kept.
    pub fn the_new_slot_boots_and_the_lab_logs_in(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.fastboot(&["-s".into(), u.lab.serial()?, "reboot".into()])?;
            let until = Instant::now() + Duration::from_secs(u.s.device_wait);
            let read = loop {
                match u.ssh("echo ---id; id -u; echo ---cmdline; cat /proc/cmdline; echo ---pid1; cat /proc/1/cmdline; echo; echo ---mounts; cat /proc/mounts; echo ---end", 40) {
                    Ok(text) if text.contains("---end") => break text,
                    other => {
                        if Instant::now() >= until {
                            return Err(format!("no ssh login within {} s as {}@{}: {}", u.s.device_wait, u.s.ssh_user, u.s.address, other.err().unwrap_or_else(|| "the read did not finish".into())));
                        }
                        std::thread::sleep(Duration::from_secs(3));
                    }
                }
            };
            u.lab
                .save("new-slot-read.txt", &crate::touch::redact(&read))?;
            let parts = sections(&read);
            let get = |n: &str| {
                parts
                    .iter()
                    .find(|(k, _)| k == n)
                    .map(|(_, t)| t.as_str())
                    .ok_or_else(|| format!("the read has no {n} section"))
            };
            if get("id")?.trim() != "0" {
                return Err("the login is not root's".into());
            }
            let want = format!("androidboot.slot_suffix=_{}", slot_of_boot(&u.target()?)?);
            if !get("cmdline")?.split_whitespace().any(|w| w == want) {
                return Err(format!("the kernel was not booted with {want} (the bootloader booted the other slot): {}", get("cmdline")?.trim()));
            }
            pid1_is_the_roots_init(get("pid1")?)?;
            let device = mounts_are_read_only(get("mounts")?)?;
            println!("  booted from the card with {want}; {device} is mounted read-only under the overlay");
            let mut gpt = String::new();
            for letter in disk_letters(u.s.luns)? {
                let hex = u.ssh(&gpt_dump_command("/dev/sd", letter)?, 60)?;
                gpt.push_str(&format!(
                    "sd{letter} {}\n",
                    hex.split_whitespace().collect::<String>()
                ));
            }
            u.lab.save("gpt-new-slot.txt", gpt.trim_end())
        }
    }

    fn gpt_lines(text: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        text.lines()
            .map(|l| {
                let (disk, hex) = l.split_once(' ').ok_or("a GPT line has no disk")?;
                Ok((
                    disk.to_string(),
                    dump_bytes(hex).map_err(|e| format!("{disk}: {e}"))?,
                ))
            })
            .collect()
    }

    /// The GPT read from the new slot's root, against the one read from Android before: only the slot switch.
    pub fn the_gpt_changed_only_by_the_switch(
        &self,
        after: &'static str,
    ) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let (b, a) = (
                FlashBoot::gpt_lines(&u.saved("gpt-before.txt")?)?,
                FlashBoot::gpt_lines(&u.saved(after)?)?,
            );
            if b.len() != a.len() || b.iter().zip(&a).any(|(x, y)| x.0 != y.0) {
                return Err("the disks of the two reads are not the same".into());
            }
            for ((disk, before), (_, after)) in b.iter().zip(&a) {
                let said = gpt_switch_only(before, after).map_err(|e| format!("{disk}: {e}"))?;
                println!("  {disk}: {said}");
            }
            Ok(())
        }
    }

    /// THE WAY BACK to the bootloader and the switch back: our kernel is asked to restart into the bootloader (`to_bootloader` of device.toml: whether our kernel can do that is the thing the host cannot prove),
    /// the phone is waited for in fastboot, `fastboot set_active <Android's slot>` is made, and the flags are checked.
    pub fn the_switch_is_made_back(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || u.switch_back()
    }

    fn switch_back(&self) -> Done {
        let android = self.android_slot()?;
        if !self.lab.in_fastboot().unwrap_or(false) {
            // the phone hangs up as it goes down: the end of the answer is not read
            let _ = self.ssh(&self.s.to_bootloader, 10);
            let until = Instant::now() + Duration::from_secs(self.s.device_wait);
            while !self.lab.in_fastboot().unwrap_or(false) {
                if Instant::now() >= until {
                    return Err(self.hand_step(&android));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
        // a phone that is already on Android's slot (the switch never happened, or was done by hand) is not written to
        let now = parse_current_slot(&self.fastboot_getvar("current-slot")?)?;
        if now != android {
            self.fastboot(&set_active_args(&self.lab.serial()?, &android)?)?;
        }
        self.lab.save(RESTORED_FILE, &android)?;
        Ok(())
    }

    /// The one hand step that is left when our kernel does not reach the bootloader, with the exact command.
    fn hand_step(&self, android: &str) -> String {
        format!(
            "the phone did not reach the bootloader within {} s of `{}` (not known yet whether our kernel can restart into it): hold power and volume-down until the bootloader screen shows, then run: fastboot set_active {android} && fastboot reboot",
            self.s.device_wait, self.s.to_bootloader
        )
    }

    /// In fastboot, after the way back: Android's slot is current again and not unbootable.
    pub fn the_flags_are_back(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.flags("slots-after-way-back.txt")?;
            way_back_restored(
                &u.saved("slots-before.txt")?,
                &u.saved("slots-after-way-back.txt")?,
                &u.android_slot()?,
            )
        }
    }

    /// The phone is back on Android (the shared step).
    pub fn android_is_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || lab.android_is_back()
    }

    /// On Android again: the boot, dtbo and vbmeta partitions are as the flash left them (only the flashed boot differs from before) and the GPT is read again.
    pub fn the_partitions_are_as_the_flash_left_them(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.read_parts("parts-after.txt")?;
            let (a, b) = (u.saved("parts-readback.txt")?, u.saved("parts-after.txt")?);
            if a == b {
                u.read_gpts_android("gpt-after.txt")
            } else {
                Err(format!(
                    "the boot, dtbo and vbmeta partitions changed after the flash:\n{a}\nthen\n{b}"
                ))
            }
        }
    }

    /// The boot chain read again equals the stored set: no bootloader partition changed.
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

    /// The way back, the last step however the run ended (the watching parent runs it after the child ends): when Android's slot was switched away from and not switched back, the phone is
    /// taken to the bootloader and switched back; then the shared way back leaves it on Android. What it could not do is said with the one hand step.
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            if std::env::var_os(watch::CHILD).is_some() {
                return Ok(());
            }
            let switched = u.marked(SWITCHED_FILE).is_some() && u.marked(RESTORED_FILE).is_none();
            let back = if switched { u.switch_back() } else { Ok(()) };
            if back.is_err() && u.lab.in_fastboot().unwrap_or(false) {
                // `fastboot reboot` would boot the active slot, which is ours: the phone stays in the bootloader and the hand command is said
                return back;
            }
            let android = u.lab.way_back();
            back?;
            android
        }
    }

    /// The rehearsal's way back: nothing was written, so the shared way back is all it needs.
    pub fn rehearsal_way_back(&self) -> impl FnOnce() -> Done + 'static {
        let lab = self.lab.clone();
        move || {
            if std::env::var_os(watch::CHILD).is_some() {
                return Ok(());
            }
            lab.way_back()
        }
    }
}

/// The `init` file of a boot image's ramdisk (gzip or xz compressed cpio), as text.
pub fn init_of_ramdisk(ramdisk: &std::path::Path) -> Result<String, String> {
    for tool in ["gzip", "xz"] {
        let mut dec = Command::new(tool);
        dec.args(["-dc"])
            .arg(ramdisk)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        let Ok(mut child) = dec.spawn() else { continue };
        let Some(out) = child.stdout.take() else {
            continue;
        };
        let cpio = Command::new("cpio")
            .args(["-i", "--quiet", "--to-stdout", "init"])
            .stdin(std::process::Stdio::from(out))
            .stderr(std::process::Stdio::null())
            .output()
            .map_err(|e| format!("cannot run cpio: {e}"))?;
        let ok = child.wait().map(|s| s.success()).unwrap_or(false);
        if ok && !cpio.stdout.is_empty() {
            return Ok(String::from_utf8_lossy(&cpio.stdout).into_owned());
        }
    }
    Err(format!("no /init could be read from {}", ramdisk.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- the flash target and the commands ----

    #[test]
    fn only_the_boot_partition_of_the_slot_android_is_not_on_may_be_flashed() {
        assert_eq!(flash_target("boot_a boot_b", "b").unwrap(), "boot_a");
        assert_eq!(flash_target("boot_a boot_b", "a").unwrap(), "boot_b");
        assert_eq!(flash_target("boot_a", "b").unwrap(), "boot_a");
        // Android's own slot is never the target, whatever the list says
        assert!(flash_target("boot_b", "b")
            .unwrap_err()
            .contains("not on device.toml's flash_allow"));
        // a list that names anything but a boot partition is refused whole
        for bad in ["boot_a abl_a", "xbl_a", "boot", "system_a", "boot_a boot_c"] {
            assert!(flash_target(bad, "b").is_err(), "{bad}");
        }
        // doubts about the slot flash nothing
        for slot in ["", "c", "_b", "ab"] {
            assert!(flash_target("boot_a boot_b", slot).is_err(), "{slot:?}");
        }
    }

    #[test]
    fn the_flash_and_the_switch_are_one_fastboot_call_each_with_one_partition_or_slot() {
        let a = flash_args("SER", "boot_a", "/x/NOT-TO-BOOT-img/boot.img").unwrap();
        assert_eq!(
            a,
            [
                "-s",
                "SER",
                "flash",
                "boot_a",
                "/x/NOT-TO-BOOT-img/boot.img"
            ]
        );
        for bad in [
            "boot_b2",
            "abl_a",
            "xbl_a",
            "boot",
            "userdata",
            "boot_a boot_b",
            "",
        ] {
            assert!(flash_args("SER", bad, "/x").is_err(), "{bad:?}");
        }
        assert!(flash_args("SER", "boot_a", "/x y").is_err());
        assert!(flash_args("SER", "boot_a", "/x;erase").is_err());
        assert_eq!(
            set_active_args("SER", "a").unwrap(),
            ["-s", "SER", "set_active", "a"]
        );
        assert!(set_active_args("SER", "c").is_err());
        assert!(set_active_args("SER", "a b").is_err());
        assert_eq!(slot_of_boot("boot_b").unwrap(), "b");
        assert!(slot_of_boot("system_b").is_err());
        // no erase, format, oem or wipe verb anywhere in what this module builds
        let all = [a.join(" "), set_active_args("S", "b").unwrap().join(" ")].join(" ");
        for verb in [
            "erase", "format", "oem", "wipe", "flashall", "update", "gpt",
        ] {
            assert!(!all.contains(verb), "{verb}: {all}");
        }
    }

    // ---- the slot flags ----

    const BEFORE: &str = "current-slot:b\nslot-retry-count:a:7\nslot-retry-count:b:7\nslot-successful:a:yes\nslot-successful:b:yes\nslot-unbootable:a:no\nslot-unbootable:b:no";

    fn after_switch() -> String {
        BEFORE
            .replace("current-slot:b", "current-slot:a")
            .replace("slot-successful:a:yes", "slot-successful:a:no")
            .replace("slot-retry-count:a:7", "slot-retry-count:a:6")
    }

    #[test]
    fn the_bootloaders_switch_is_accepted_and_anything_else_is_not() {
        assert!(switch_is_the_abls(BEFORE, &after_switch(), "b", "a").is_ok());
        // the other slot's own flags changed
        let bad = after_switch().replace("slot-successful:b:yes", "slot-successful:b:no");
        assert!(switch_is_the_abls(BEFORE, &bad, "b", "a")
            .unwrap_err()
            .contains("slot-successful:b"));
        // the new slot is unbootable, or still successful, or has no tries left
        for (what, v) in [
            (
                "unbootable",
                after_switch().replace("slot-unbootable:a:no", "slot-unbootable:a:yes"),
            ),
            (
                "successful",
                after_switch().replace("slot-successful:a:no", "slot-successful:a:yes"),
            ),
            (
                "retry 0",
                after_switch().replace("slot-retry-count:a:6", "slot-retry-count:a:0"),
            ),
            (
                "retry 9",
                after_switch().replace("slot-retry-count:a:6", "slot-retry-count:a:9"),
            ),
            (
                "not switched",
                after_switch().replace("current-slot:a", "current-slot:b"),
            ),
            (
                "a flag went",
                after_switch()
                    .replace("slot-unbootable:b:no\n", "")
                    .replace("\nslot-unbootable:b:no", ""),
            ),
        ] {
            assert!(switch_is_the_abls(BEFORE, &v, "b", "a").is_err(), "{what}");
        }
        // the slot before was not the one named
        assert!(switch_is_the_abls(BEFORE, &after_switch(), "a", "b").is_err());
    }

    #[test]
    fn the_way_back_leaves_android_current_and_bootable() {
        assert!(way_back_restored(BEFORE, BEFORE, "b").is_ok());
        assert!(way_back_restored(BEFORE, &after_switch(), "b").is_err());
        let unb = BEFORE.replace("slot-unbootable:b:no", "slot-unbootable:b:yes");
        assert!(way_back_restored(BEFORE, &unb, "b")
            .unwrap_err()
            .contains("unbootable"));
        assert!(
            way_back_restored(BEFORE, &BEFORE.replace("\nslot-retry-count:a:7", ""), "b").is_err()
        );
    }

    // ---- the GPT ----

    fn name16(name: &str) -> Vec<u8> {
        name.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    /// A synthetic dump of one disk with the A/B pairs given, each entry: type GUID filled with a byte, attributes filled with a byte.
    fn dump(entries: &[(&str, u8, u8)]) -> Vec<u8> {
        let mut d = vec![0u8; DUMP_LEN];
        for hdr in [P_HDR, B_HDR] {
            d[hdr..hdr + 8].copy_from_slice(b"EFI PART");
            d[hdr + 12..hdr + 16].copy_from_slice(&92u32.to_le_bytes());
            d[hdr + 80..hdr + 84].copy_from_slice(&128u32.to_le_bytes());
            d[hdr + 84..hdr + 88].copy_from_slice(&128u32.to_le_bytes());
        }
        for (i, (name, guid, attr)) in entries.iter().enumerate() {
            for base in [P_ENT, B_ENT] {
                let at = base + i * ENTRY;
                d[at..at + 16].fill(*guid);
                d[at + 16..at + 32].fill(0x77); // the partition's own GUID: never changes
                d[at + 54] = *attr; // the slot bits live in byte 6 of the 8 attribute bytes
                let n = name16(name);
                d[at + 56..at + 56 + n.len()].copy_from_slice(&n);
            }
        }
        fix_crcs(&mut d);
        d
    }

    fn fix_crcs(d: &mut [u8]) {
        for (hdr, ent) in [(P_HDR, P_ENT), (B_HDR, B_ENT)] {
            let ec = crc32(&d[ent..ent + ENTRIES * ENTRY]);
            d[hdr + 88..hdr + 92].copy_from_slice(&ec.to_le_bytes());
            d[hdr + 16..hdr + 20].fill(0);
            let hc = crc32(&d[hdr..hdr + 92]);
            d[hdr + 16..hdr + 20].copy_from_slice(&hc.to_le_bytes());
        }
    }

    fn before() -> Vec<u8> {
        dump(&[
            ("boot_a", 1, 0x10),
            ("boot_b", 2, 0x20),
            ("system_a", 3, 0x10),
            ("system_b", 4, 0x20),
            ("userdata", 9, 0),
        ])
    }

    /// What the bootloader's switch to slot a does: the type GUIDs of every A/B pair swap, the attributes change, the CRCs are recomputed, in both copies.
    fn switched() -> Vec<u8> {
        let mut d = before();
        for base in [P_ENT, B_ENT] {
            for (x, y) in [(0usize, 1usize), (2, 3)] {
                let (ax, ay) = (base + x * ENTRY, base + y * ENTRY);
                let gx: Vec<u8> = d[ax..ax + 16].to_vec();
                let gy: Vec<u8> = d[ay..ay + 16].to_vec();
                d[ax..ax + 16].copy_from_slice(&gy);
                d[ay..ay + 16].copy_from_slice(&gx);
                d[ax + 54] = 0x30;
                d[ay + 54] = 0x00;
            }
        }
        fix_crcs(&mut d);
        d
    }

    #[test]
    fn crc32_is_the_ieee_one() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn an_unchanged_gpt_and_the_bootloaders_switch_pass() {
        assert!(gpt_switch_only(&before(), &before()).is_ok());
        let said = gpt_switch_only(&before(), &switched()).unwrap();
        assert!(
            said.contains("4 entries") && said.contains("4 type GUIDs"),
            "{said}"
        );
    }

    #[test]
    fn anything_but_the_switch_fails_with_where() {
        // a partition renamed
        let mut d = switched();
        for base in [P_ENT, B_ENT] {
            d[base + 4 * ENTRY + 56] ^= 1;
        }
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before(), &d)
            .unwrap_err()
            .contains("not a slot bit"));
        // the first and last usable LBA moved (a resize)
        let mut d = switched();
        d[P_HDR + 40] ^= 1;
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before(), &d).is_err());
        // a type GUID changed to something that is not the partner's
        let mut d = before();
        for base in [P_ENT, B_ENT] {
            d[base + 4 * ENTRY..base + 4 * ENTRY + 16].fill(0xEE);
        }
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before(), &d)
            .unwrap_err()
            .contains("no A/B partner"));
        let mut d = before();
        for base in [P_ENT, B_ENT] {
            d[base..base + 16].fill(0xEE);
        }
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before(), &d)
            .unwrap_err()
            .contains("not a swap"));
        // the CRC was not recomputed
        let mut d = switched();
        d[P_HDR + 88] ^= 0xFF;
        assert!(gpt_switch_only(&before(), &d)
            .unwrap_err()
            .contains("after:"));
        // the protective MBR changed
        let mut d = before();
        d[10] = 1;
        assert!(gpt_switch_only(&before(), &d).is_err());
        // a partition's own GUID changed
        let mut d = before();
        for base in [P_ENT, B_ENT] {
            d[base + 16] ^= 1;
        }
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before(), &d).is_err());
        // a one-sided GUID change: the partner kept its GUID, so two entries would share one
        let mut d = before();
        for base in [P_ENT, B_ENT] {
            let g: Vec<u8> = d[base + ENTRY..base + ENTRY + 16].to_vec();
            d[base..base + 16].copy_from_slice(&g);
        }
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before(), &d)
            .unwrap_err()
            .contains("not a swap"));
        // slot bits changed on a partition that has no A/B partner
        let mut d = before();
        for base in [P_ENT, B_ENT] {
            d[base + 4 * ENTRY + 54] ^= 0x40;
        }
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before(), &d)
            .unwrap_err()
            .contains("slot bits"));
        // an attribute byte that is not the slot byte (a GPT required-partition flag, say)
        let mut d = before();
        for base in [P_ENT, B_ENT] {
            d[base + 48] ^= 1;
        }
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before(), &d)
            .unwrap_err()
            .contains("not a slot bit"));
        // only the backup copy of a swap is wrong
        let mut d = switched();
        d[B_ENT..B_ENT + 16].fill(0xEE);
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before(), &d).is_err());
        // an empty or short read is no GPT
        assert!(gpt_switch_only(&before(), &[0u8; 10]).is_err());
        assert!(dump_bytes("").is_err());
        assert!(dump_bytes("zz").is_err());
    }

    #[test]
    fn the_dump_command_reads_and_writes_nothing() {
        let c = gpt_dump_command("/dev/block/sd", 'a').unwrap();
        assert!(
            c.contains("if=/dev/block/sda") && c.contains("od -An -v -tx1"),
            "{c}"
        );
        assert_eq!(c.matches("of=").count(), 0, "{c}");
        for forbidden in [
            "> /dev/sd",
            ">/dev/sd",
            "> /dev/block",
            ">/dev/block",
            "mkfs",
            "sgdisk",
            "fastboot",
        ] {
            assert!(!c.contains(forbidden), "{forbidden}");
        }
        assert!(gpt_dump_command("/dev/block/sd", 'A').is_err());
        assert!(gpt_dump_command("/dev/sd;x", 'a').is_err());
        // a real dump round-trips through the hex the command prints
        let hex: String = switched().iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(dump_bytes(&hex).unwrap(), switched());
    }

    // ---- the image's init ----

    #[test]
    fn the_images_inits_pass_the_stage_2_gate_and_keep_the_stage_1_guard_behind_the_word() {
        for path in [
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../build/initramfs-root/init"
            ),
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../build/initramfs-root-display/init"
            ),
        ] {
            let init = std::fs::read_to_string(path).unwrap();
            assert!(
                init_accepts_the_booted_slot(&init).is_ok(),
                "{path}: {:?}",
                init_accepts_the_booted_slot(&init)
            );
            // the stage 1 guard is still there, asked for only without the word
            assert!(
                init.contains("walleye.booted-slot")
                    && init.contains("is the system of the slot Android is on"),
                "{path}: the guard of stage 1 is gone"
            );
        }
        assert!(!init_refuses_its_own_slot(
            "#!/bin/sh\n# the slot Android runs from is only a comment here\nexec /sbin/init\n"
        ));
        assert!(!init_refuses_its_own_slot(
            "mount the booted slot's own system partition\n"
        ));
    }

    // ---- the steps against a stand-in fastboot that records what it is asked ----

    use crate::fixture::executable;

    fn bench(name: &str, fastboot_says: &str) -> (FlashBoot, std::path::PathBuf) {
        bench_image(name, fastboot_says, None)
    }

    fn bench_image(
        name: &str,
        fastboot_says: &str,
        image: Option<String>,
    ) -> (FlashBoot, std::path::PathBuf) {
        let d = std::env::temp_dir().join(format!("wt-flash-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("device.toml"), "[connection]\nserial = \"SER1\"\n").unwrap();
        let fb = d.join("fastboot");
        executable(
            &fb,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/fastboot.calls\ncase \"$4\" in current-slot) echo \"current-slot: {fastboot_says}\" >&2 ;; esac\ncase \"$1\" in devices) printf 'SER1\\tfastboot\\n' ;; esac\nexit 0\n",
                d = d.display()
            ),
        );
        let mut s = Settings::stand_in(crate::settings::Try::FlashBoot);
        s.lab = d.clone();
        s.out = d.join("out");
        s.fastboot = fb.to_string_lossy().into_owned();
        // a stand-in image that names its root, the sha256 of it, and a tad that says the go is open for exactly that sha256
        let made = d.join("NOT-TO-BOOT-boot.img");
        std::fs::write(&made, b"console=ttyMSM0 walleye.root=system_a padding").unwrap();
        s.image = image.map_or(made.clone(), std::path::PathBuf::from);
        s.image_sha256 = host::sha256_of(&made).unwrap();
        let tad = d.join("tad");
        executable(&tad, &go_row(&s.image_sha256, true));
        std::fs::create_dir_all(&s.out).unwrap();
        (
            FlashBoot::with_programs(s, "ssh", &tad.to_string_lossy()),
            d,
        )
    }

    /// A tad stand-in whose row for the go item names the sha256 and, when open, a go_until far in the future.
    fn go_row(sha: &str, open: bool) -> String {
        let until = if open {
            "\"go_until\":\"2099-01-01T00:00:00+00:00\""
        } else {
            "\"go_until\":null"
        };
        format!("#!/bin/sh\necho '{{\"id\":\"go\",{until},\"note\":\"{sha}\"}}'\n")
    }

    fn calls(d: &std::path::Path) -> String {
        std::fs::read_to_string(d.join("fastboot.calls")).unwrap_or_default()
    }

    #[test]
    fn the_flash_is_one_call_for_the_one_partition_after_both_readings_of_the_slot_agree() {
        let (f, d) = bench("flash", "b");
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        (f.the_boot_image_is_flashed())().unwrap();
        let c = calls(&d);
        assert_eq!(
            c.lines().filter(|l| l.contains(" flash ")).count(),
            1,
            "{c}"
        );
        assert!(
            c.contains(&format!(
                "-s SER1 flash boot_a {}",
                d.join("NOT-TO-BOOT-boot.img").display()
            )),
            "{c}"
        );
        for verb in ["erase", "format", "oem", "set_active", "reboot"] {
            assert!(!c.contains(verb), "{verb}: {c}");
        }
    }

    #[test]
    fn no_flash_when_the_slot_is_unread_or_the_two_readings_disagree() {
        let (f, d) = bench("unread", "b");
        assert!((f.the_boot_image_is_flashed())()
            .unwrap_err()
            .contains("not read yet"));
        let (f, d2) = bench("disagree", "a");
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        assert!((f.the_boot_image_is_flashed())()
            .unwrap_err()
            .contains("disagree"));
        let (f, d3) = bench("garbage", "b");
        f.lab.save(ANDROID_SLOT_FILE, "c").unwrap();
        assert!((f.the_boot_image_is_flashed())().is_err());
        for dir in [d, d2, d3] {
            assert!(!calls(&dir).contains(" flash "), "{}", calls(&dir));
        }
    }

    #[test]
    fn set_active_waits_for_the_read_back_and_names_only_the_slot_flashed() {
        let (f, d) = bench("switch", "b");
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        assert!((f.the_slot_is_made_active())()
            .unwrap_err()
            .contains("not read back"));
        assert!(!calls(&d).contains("set_active"));
        f.lab.save("parts-readback.txt", "boot_a x").unwrap();
        (f.the_slot_is_made_active())().unwrap();
        assert!(calls(&d).contains("-s SER1 set_active a"), "{}", calls(&d));
        assert!(!calls(&d).contains("set_active b"));
        assert_eq!(
            std::fs::read_to_string(d.join("out").join(SWITCHED_FILE))
                .unwrap()
                .trim(),
            "a"
        );
    }

    #[test]
    fn the_way_back_to_androids_slot_is_made_from_fastboot_and_marked() {
        let (f, d) = bench("back", "a");
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        // the phone is in fastboot (the stand-in lists it): the switch back is `set_active b`, the slot Android runs from
        (f.the_switch_is_made_back())().unwrap();
        assert!(calls(&d).contains("-s SER1 set_active b"), "{}", calls(&d));
        assert_eq!(
            std::fs::read_to_string(d.join("out").join(RESTORED_FILE))
                .unwrap()
                .trim(),
            "b"
        );
    }

    /// The gate on the lab's real images when they are on this machine (a machine without them says so and skips: not known yet, not a pass of the gate).
    #[test]
    fn the_init_gate_refuses_the_stage_1_image_of_the_lab() {
        let home = std::env::var("HOME").unwrap_or_default();
        let root = format!("{home}/Projects/walleye-build/NOT-TO-BOOT-ufs-on-images-ufs-root/NOT-TO-BOOT-ufs-on-ufs-root.img");
        if !std::path::Path::new(&root).exists()
            || Command::new("lpoco-bringup")
                .arg("--version")
                .output()
                .is_err()
        {
            eprintln!("NOT KNOWN YET: the lab's image or lpoco-bringup is not on this machine: the init gate is not run on a real image here");
            return;
        }
        let (f, _d) = bench_image("gate", "b", Some(root));
        let said = (f.the_init_accepts_the_slot_it_is_booted_from())().unwrap_err();
        assert!(
            said.contains("refuses the slot Android runs from"),
            "{said}"
        );
    }

    /// The gate on the stage 2 image of the lab (item walleye-install-stage2-boot-image): its init passes the gate and its command line carries the word walleye.booted-slot.
    #[test]
    fn the_init_gate_accepts_the_stage_2_image_of_the_lab() {
        let home = std::env::var("HOME").unwrap_or_default();
        let root = format!("{home}/Projects/walleye-build/NOT-TO-BOOT-ufs-on-images-ufs-root-booted-slot/NOT-TO-BOOT-ufs-on-ufs-root-booted-slot.img");
        if !std::path::Path::new(&root).exists()
            || Command::new("lpoco-bringup")
                .arg("--version")
                .output()
                .is_err()
        {
            eprintln!("NOT KNOWN YET: the lab's stage 2 image or lpoco-bringup is not on this machine: the init gate is not run on it here");
            return;
        }
        let (f, _d) = bench_image("gate2", "b", Some(root));
        (f.the_init_accepts_the_slot_it_is_booted_from())().unwrap();
        let bytes = std::fs::read(&f.s.image).unwrap();
        assert!(crate::rootfs_boot::header_cmdline(&bytes)
            .unwrap()
            .split_whitespace()
            .any(|w| w == "walleye.booted-slot"));
    }
    #[test]
    fn an_init_passes_the_gate_only_when_it_declares_that_it_mounts_the_booted_slot() {
        let declared = format!(
            "#!/bin/busybox sh\nsay \"{BOOTED_SLOT_MARK} (system_a)\"\nexec switch_root /newroot /sbin/init\n"
        );
        // the mark in a comment only is not a declaration
        assert!(init_accepts_the_booted_slot(&format!(
            "#!/bin/busybox sh\n# {BOOTED_SLOT_MARK}\nexec switch_root /newroot /sbin/init\n"
        ))
        .is_err());
        assert!(init_accepts_the_booted_slot(&declared).is_ok());
        // the old refusal still there
        let both = format!(
            "{declared}[ \"$ROOT\" != \"system$SLOT\" ] || refuse \"the slot Android runs from\"\n"
        );
        assert!(init_accepts_the_booted_slot(&both)
            .unwrap_err()
            .contains("refuses"));
        // the refusal worded differently, nothing declared: a gate that only looked for the old words would pass this
        let silent = "#!/bin/busybox sh\n[ \"$ROOT\" != \"system$SLOT\" ] || refuse \"root is the running slot\"\n";
        assert!(init_accepts_the_booted_slot(silent)
            .unwrap_err()
            .contains("does not say"));
        assert!(init_accepts_the_booted_slot("").is_err());
    }

    #[test]
    fn the_root_the_image_names_must_be_the_flashed_slots_own_system_partition() {
        // the stand-in image names system_a: Android on b flashes boot_a, root system_a is right; Android on a flashes boot_b, root system_a would be Android's own
        let (f, _d) = bench("root-ok", "b");
        f.the_root_is_the_flashed_slots_own("b").unwrap();
        let e = f.the_root_is_the_flashed_slots_own("a").unwrap_err();
        assert!(
            e.contains("system_a") && e.contains("system_b") && e.contains("nothing is flashed"),
            "{e}"
        );
    }

    #[test]
    fn the_flash_needs_the_go_open_for_this_image_at_the_flash_itself() {
        let (f, d) = bench("nogo", "b");
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        executable(&d.join("tad"), &go_row(&f.s.image_sha256, false));
        assert!((f.the_boot_image_is_flashed())()
            .unwrap_err()
            .contains("go"));
        // a go for another image opens nothing
        executable(&d.join("tad"), &go_row(&"cd".repeat(32), true));
        assert!((f.the_boot_image_is_flashed())()
            .unwrap_err()
            .contains("no go"));
        assert!(!calls(&d).contains(" flash "), "{}", calls(&d));
        // a changed image file is not flashed
        executable(&d.join("tad"), &go_row(&f.s.image_sha256, true));
        std::fs::write(&f.s.image, b"changed walleye.root=system_a").unwrap();
        assert!((f.the_boot_image_is_flashed())().is_err());
        assert!(!calls(&d).contains(" flash "), "{}", calls(&d));
    }

    #[test]
    fn a_failed_start_after_an_unfinished_switch_is_refused_and_a_phone_already_on_androids_slot_is_not_written(
    ) {
        let (f, d) = bench("stale", "b");
        // the switch back finds the current slot already Android's: nothing is written, the run is marked restored
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        (f.the_switch_is_made_back())().unwrap();
        assert!(!calls(&d).contains("set_active"), "{}", calls(&d));
        assert_eq!(
            std::fs::read_to_string(d.join("out").join(RESTORED_FILE))
                .unwrap()
                .trim(),
            "b"
        );
    }

    #[test]
    fn the_hand_step_names_the_exact_command() {
        let (f, _d) = bench("hand", "a");
        let said = f.hand_step("b");
        assert!(
            said.contains("fastboot set_active b && fastboot reboot")
                && said.contains("reboot bootloader"),
            "{said}"
        );
    }

    #[test]
    fn what_the_author_is_told_comes_from_the_settings() {
        let s = Settings::stand_in(crate::settings::Try::FlashBoot);
        let said = expect(&s);
        assert!(
            said.contains("boot_a boot_b")
                && said.contains("6 disks")
                && said.contains("set_active"),
            "{said}"
        );
        assert!(expect_rehearsal(&s).contains("nothing written"));
    }
}
