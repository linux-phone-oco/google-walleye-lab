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
/// When the switch was made (epoch seconds): the lab host's kernel log is read from then on (item walleye-stage2-debug-image).
pub const SWITCHED_AT_FILE: &str = "switched-at.txt";
/// The lab host's USB log of a run's wait for the new slot, the lines of the phone's port.
pub const HOST_USB_LOG_FILE: &str = "host-usb-log.txt";

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
// which slot booted (step 21)
// ---------------------------------------------------------------------------------------------------------------------

/// The slot a booted kernel says it runs from, and which sources said it (each `(source, slot)`, in the order of [`booted_slot`]).
#[derive(Debug, PartialEq)]
pub struct SlotAnswer {
    /// `a` or `b`
    pub slot: String,
    pub answers: Vec<(&'static str, String)>,
}

/// `_a` / `_b` (as Android's `androidboot.slot_suffix` and the device tree write them) as `a` / `b`; anything else is not a slot.
fn suffix_slot(suffix: &str) -> Result<String, String> {
    match suffix {
        "_a" => Ok("a".into()),
        "_b" => Ok("b".into()),
        other => Err(format!("{other:?} is not _a or _b")),
    }
}

/// Source 1: the word `androidboot.slot_suffix=_X` the bootloader puts on the command line. None when the word is not there.
fn slot_of_cmdline_word(cmdline: &str) -> Result<Option<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for value in cmdline
        .split_whitespace()
        .filter_map(|w| w.strip_prefix("androidboot.slot_suffix="))
    {
        let slot = suffix_slot(value).map_err(|e| format!("androidboot.slot_suffix: {e}"))?;
        if !found.contains(&slot) {
            found.push(slot);
        }
    }
    match found.as_slice() {
        [] => Ok(None),
        [one] => Ok(Some(one.clone())),
        _ => Err(format!(
            "the command line has androidboot.slot_suffix for more than one slot ({})",
            found.join(", ")
        )),
    }
}

/// Source 2: the `root=PARTUUID=0000NNNN-...` word the bootloader names the booted slot's system partition with: NNNN (hex) is the number of the partition, its entry in the GPT of the disk
/// (`root_disk`, the dump of the disk the root is mounted from) is named `system_a` or `system_b`. None when no `root=PARTUUID=` word is there.
fn slot_of_root_partuuid(cmdline: &str, root_disk: &[u8]) -> Result<Option<String>, String> {
    let Some(partuuid) = cmdline
        .split_whitespace()
        .filter_map(|w| w.strip_prefix("root=PARTUUID="))
        .next_back()
    else {
        return Ok(None);
    };
    let parts: Vec<&str> = partuuid.split('-').collect();
    let group = parts[0];
    if group.len() != 8
        || !group.bytes().all(|b| b.is_ascii_hexdigit())
        || parts[1..] != ["0000", "0000", "0000", "000000000000"]
    {
        return Err(format!(
            "root=PARTUUID={partuuid} is not the bootloader's form 0000NNNN-0000-0000-0000-000000000000"
        ));
    }
    let number =
        usize::from_str_radix(group, 16).map_err(|e| format!("root=PARTUUID={partuuid}: {e}"))?;
    let layout = gpt_layout(root_disk).map_err(|e| {
        format!("root=PARTUUID names partition {number}, the GPT of its disk is not whole: {e}")
    })?;
    if !(1..=layout.count).contains(&number) {
        return Err(format!(
            "root=PARTUUID names partition {number}, the disk's GPT has {} entries",
            layout.count
        ));
    }
    let at = P_ENT + (number - 1) * ENTRY;
    let name = entry_name(root_disk, at);
    match name.as_str() {
        "system_a" => Ok(Some("a".into())),
        "system_b" => Ok(Some("b".into())),
        other => Err(format!("root=PARTUUID names partition {number}, which the GPT calls {other:?}, not system_a or system_b")),
    }
}

/// Source 3: `/proc/device-tree/firmware/android/slot_suffix`, as the phone printed it (a NUL ends the string). None when the node is not there (an empty read).
fn slot_of_device_tree(printed: &str) -> Result<Option<String>, String> {
    let value = printed.trim_matches(|c: char| c == '\0' || c.is_whitespace());
    if value.is_empty() {
        return Ok(None);
    }
    suffix_slot(value)
        .map(Some)
        .map_err(|e| format!("device tree firmware/android/slot_suffix: {e}"))
}

/// Which slot the kernel booted from, from what the BOOTLOADER supplied and never from the words of our own boot.img, in the order Android's own init takes
/// (AOSP fs_mgr, fs_mgr_get_slot_suffix): the command line's `androidboot.slot_suffix`, else the `root=PARTUUID` the bootloader names the root with, mapped through the GPT of the root's disk,
/// else the device tree's `firmware/android/slot_suffix`. Every source that answers is read and they must agree; none, or two that differ, is an error. The answer says which sources spoke.
pub fn booted_slot(
    cmdline: &str,
    device_tree: &str,
    root_disk: &[u8],
) -> Result<SlotAnswer, String> {
    let sources: [(&'static str, Option<String>); 3] = [
        ("androidboot.slot_suffix", slot_of_cmdline_word(cmdline)?),
        ("root=PARTUUID", slot_of_root_partuuid(cmdline, root_disk)?),
        ("device-tree", slot_of_device_tree(device_tree)?),
    ];
    let answers: Vec<(&'static str, String)> = sources
        .into_iter()
        .filter_map(|(source, slot)| slot.map(|s| (source, s)))
        .collect();
    let Some((_, slot)) = answers.first().cloned() else {
        return Err("no source gives the booted slot: no androidboot.slot_suffix on the command line, no root=PARTUUID, no device-tree firmware/android/slot_suffix".into());
    };
    if let Some((source, other)) = answers.iter().find(|(_, s)| *s != slot) {
        let said: Vec<String> = answers
            .iter()
            .map(|(src, s)| format!("{src} says {s}"))
            .collect();
        return Err(format!(
            "the sources of the booted slot disagree ({}): {source} is not {slot}, it is {other}",
            said.join(", ")
        ));
    }
    Ok(SlotAnswer { slot, answers })
}

/// The image's own header must not carry a `root=` word: the root=PARTUUID the kernel sees is then the bootloader's, and the slot it names is the slot booted - not our own words.
pub fn header_has_no_root(image: &[u8]) -> Result<(), String> {
    let header = crate::rootfs_boot::header_cmdline(image)?;
    match header
        .split_whitespace()
        .find(|w| w.starts_with("root=") || w.starts_with("androidboot.slot_suffix="))
    {
        Some(word) => Err(format!("the image's own header command line carries {word}: the slot sources could no longer tell which slot the bootloader booted")),
        None => Ok(()),
    }
}

/// The disk letter of a root device `/dev/sdXN`.
fn disk_letter_of(device: &str) -> Result<char, String> {
    let name = device.strip_prefix("/dev/sd").unwrap_or("");
    let mut chars = name.chars();
    match (chars.next(), chars.as_str()) {
        (Some(letter), digits)
            if letter.is_ascii_lowercase()
                && !digits.is_empty()
                && digits.bytes().all(|b| b.is_ascii_digit()) =>
        {
            Ok(letter)
        }
        _ => Err(format!(
            "{device:?} is not a partition /dev/sd<letter><number>"
        )),
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

/// After `set_active TO` (from `FROM`; check 20, relaxed by research walleye-stage2-checks-should-they-exist, row 20): the goal itself - `current-slot` is TO, TO is not unbootable and has tries left (a retry count above 0).
/// Every other flag, FROM's included, is the bootloader's to set (it "may reset the successful bit": update_attempter_android.cc l.368): recorded in the flags file, never required equal here; FROM's state is judged at the way back (24).
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
    let want = |k: &str, w: &str| -> Done {
        match a.get(k).map(String::as_str) {
            Some(v) if v == w => Ok(()),
            other => Err(format!(
                "after the switch {k} is {other:?}, the switch leaves {w:?}"
            )),
        }
    };
    want("current-slot", to)?;
    want(&format!("slot-unbootable:{to}"), "no")?;
    match a
        .get(&format!("slot-retry-count:{to}"))
        .and_then(|v| v.parse::<u8>().ok())
    {
        Some(n) if n >= 1 => Ok(()),
        other => Err(format!(
            "the new slot's retry count is {other:?}: the switch leaves tries to boot with (above 0)"
        )),
    }
}

/// After the flash of `boot_X` (check 18; research walleye-stage2-checks-should-they-exist, row 18, after AOSP's update_engine, which puts the target slot in any flag state itself): what the flash may do to the FLASHED slot's flags
/// (successful, retry count, unbootable) is the bootloader's own and asserted nowhere - three stops came from asserting a value of it (runs 1 and 5). What stays asserted is the invariant: Android's slot, the current slot and every other flag
/// are what they were, and no flag appeared or went.
pub fn flash_leaves_androids_slot_alone(
    before: &str,
    after: &str,
    flashed: &str,
) -> Result<(), String> {
    let (b, a) = (flags_of(before), flags_of(after));
    if b.keys().ne(a.keys()) {
        return Err(format!(
            "the flags' names changed:\n{before}\nthen\n{after}"
        ));
    }
    let own = [
        format!("slot-successful:{flashed}"),
        format!("slot-retry-count:{flashed}"),
        format!("slot-unbootable:{flashed}"),
    ];
    for (k, v) in &b {
        if !own.contains(k) && a[k] != *v {
            return Err(format!("{k} changed from {v} to {}: the flash may change slot {flashed}'s own flags, nothing else\n{before}\nthen\n{after}", a[k]));
        }
    }
    Ok(())
}

/// Before anything is written: the slot Android runs from is the way back, so it must be good as the bootloader has it - successful, not unbootable (research walleye-stage2-walk-by-state-v-earlier-image: a slot b that is not
/// successful passed every step of the walk, the one unsafe state nothing stopped). The flags are the lines of [`host::slot_lines`].
pub fn android_slot_is_a_safe_way_back(flags: &str, android: &str) -> Result<(), String> {
    let f = flags_of(flags);
    for (key, want) in [
        (format!("slot-successful:{android}"), "yes"),
        (format!("slot-unbootable:{android}"), "no"),
    ] {
        match f.get(&key).map(String::as_str) {
            Some(v) if v == want => {}
            other => {
                return Err(format!(
                    "{key} is {other:?}, not {want:?}: the slot Android runs from is the way back and the bootloader does not count it good - nothing is written\n{flags}"
                ))
            }
        }
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
const HEAD_BLOCKS: usize = 2 + ENTRY_BLOCKS;
const TAIL_BLOCKS: usize = ENTRY_BLOCKS + 1;
/// The bytes of one disk's dump: its first six blocks (the protective MBR, the header, room for 128 entries), then its last five (room for 128 entries, the backup header). A table of fewer entries fills fewer blocks of that room: the backup copy lies just before the backup header.
pub const DUMP_LEN: usize = (HEAD_BLOCKS + TAIL_BLOCKS) * GB;
/// The blocks of entries each copy has room for in the dump.
const ENTRY_BLOCKS: usize = 4;
const ENTRY: usize = 128;
/// The most entries a table can have and be read whole.
const ENTRIES: usize = ENTRY_BLOCKS * GB / ENTRY;
const P_HDR: usize = GB;
const P_ENT: usize = 2 * GB;
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

fn le64(d: &[u8], at: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[at..at + 8]);
    u64::from_le_bytes(b)
}

/// How many entries a disk's GPT has and where its two copies lie in the dump: what the headers say, parsed once, then carried (the bootloader sizes the table to the disk: the phone's six
/// LUNs have 48, 4, 4, 20, 8 and 8 entries, not the 128 of a generic table).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Layout {
    /// entries in the table, 1 to `ENTRIES` (the most the dump's four blocks of entries hold), each `ENTRY` bytes
    count: usize,
    /// where the backup copy of the entries starts: in the blocks just before the backup header, as many as the table fills
    b_ent: usize,
}

impl Layout {
    fn primary(&self) -> std::ops::Range<usize> {
        P_ENT..P_ENT + self.count * ENTRY
    }
    fn backup(&self) -> std::ops::Range<usize> {
        self.b_ent..self.b_ent + self.count * ENTRY
    }
}

/// The layout of a dump that is a GPT as the bootloader writes it: both headers say "EFI PART", the same count of 1 to 128 entries of exactly 128 bytes, the entries lie where the headers' LBAs put them
/// (the primary's at LBA 2, the backup's in the one to four blocks before the backup header, enough for the table), and the header and entries CRCs are right.
fn gpt_layout(d: &[u8]) -> Result<Layout, String> {
    if d.len() != DUMP_LEN {
        return Err(format!(
            "the GPT dump is {} bytes, a whole one is {DUMP_LEN}",
            d.len()
        ));
    }
    let mut seen: Vec<(usize, usize)> = vec![];
    for (which, hdr) in [("primary", P_HDR), ("backup", B_HDR)] {
        if &d[hdr..hdr + 8] != b"EFI PART" {
            return Err(format!("the {which} GPT header has no EFI PART signature"));
        }
        let (count, size) = (le32(d, hdr + 80) as usize, le32(d, hdr + 84) as usize);
        if !(1..=ENTRIES).contains(&count) || size != ENTRY {
            return Err(format!(
                "the {which} GPT header says {count} entries of {size} bytes, not 1 to {ENTRIES} entries of {ENTRY} bytes"
            ));
        }
        let hsize = le32(d, hdr + 12) as usize;
        if !(92..=GB).contains(&hsize) {
            return Err(format!(
                "the {which} GPT header size {hsize} is not 92 to {GB}"
            ));
        }
        let mut h = d[hdr..hdr + hsize].to_vec();
        h[16..20].fill(0);
        if crc32(&h) != le32(d, hdr + 16) {
            return Err(format!("the {which} GPT header CRC is wrong"));
        }
        let (here, entries_at) = (le64(d, hdr + 24), le64(d, hdr + 72));
        let ent = if hdr == P_HDR {
            if (here, entries_at) != (1, 2) {
                return Err(format!(
                    "the primary GPT header is at LBA {here} and its entries at LBA {entries_at}, not 1 and 2"
                ));
            }
            P_ENT
        } else {
            let blocks = here.saturating_sub(entries_at) as usize;
            if !(1..=ENTRY_BLOCKS).contains(&blocks) || blocks * GB < count * ENTRY {
                return Err(format!(
                    "the backup GPT header at LBA {here} puts its {count} entries at LBA {entries_at}: not within the {} blocks before it that the dump holds, nor enough for them",
                    ENTRY_BLOCKS
                ));
            }
            B_HDR - blocks * GB
        };
        if crc32(&d[ent..ent + count * ENTRY]) != le32(d, hdr + 88) {
            return Err(format!("the {which} GPT entries CRC is wrong"));
        }
        seen.push((count, ent));
    }
    if seen[0].0 != seen[1].0 {
        return Err(format!(
            "the primary GPT header says {} entries and the backup {}: the copies disagree",
            seen[0].0, seen[1].0
        ));
    }
    Ok(Layout {
        count: seen[0].0,
        b_ent: seen[1].1,
    })
}

/// The dump is a whole GPT as the bootloader writes it (see `gpt_layout`).
pub fn gpt_is_whole(d: &[u8]) -> Result<(), String> {
    gpt_layout(d).map(|_| ())
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
    let lay = gpt_layout(before).map_err(|e| format!("before: {e}"))?;
    if gpt_layout(after).map_err(|e| format!("after: {e}"))? != lay {
        return Err("the GPT has another count of entries (or another place for them) after than before: a switch changes neither".into());
    }
    let allowed = |at: usize| -> bool {
        for hdr in [P_HDR, B_HDR] {
            if (hdr + 16..hdr + 20).contains(&at) || (hdr + 88..hdr + 92).contains(&at) {
                return true;
            }
        }
        for (base, span) in [(P_ENT, lay.primary()), (lay.b_ent, lay.backup())] {
            if span.contains(&at) {
                let inside = (at - base) % ENTRY;
                return inside < 16 || inside == 54;
            }
        }
        false
    };
    if let Some(at) = (0..DUMP_LEN).find(|&i| before[i] != after[i] && !allowed(i)) {
        return Err(format!("byte {at} of the dump changed and it is not a slot bit, a type GUID or a CRC (block {}, offset {})", at / GB, at % GB));
    }
    let same = |d: &[u8]| d[lay.primary()] == d[lay.backup()];
    if !same(after) && same(before) {
        return Err("the primary and the backup entries agreed before and disagree now".into());
    }
    let (mut attrs, mut swaps) = (0usize, 0usize);
    for (copy, base) in [("primary", P_ENT), ("backup", lay.b_ent)] {
        let names: Vec<String> = (0..lay.count)
            .map(|i| entry_name(before, base + i * ENTRY))
            .collect();
        for i in 0..lay.count {
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

/// Check 9's decision (research walleye-stage2-checks-should-they-exist, row 9; after fastboot and update_engine, which ask nothing of what a target held): what the partition holds now protects nothing - the flash overwrites it whatever it is -
/// so it is RECORDED (the stock backup `want`, the declared image, or content not recorded: said, never refused). What protects the way back is the stock backup: `backup_file` must be on file and still hash to `want`.
fn boot_partition_can_be_flashed(
    target: &str,
    now: &str,
    want: &str,
    declared: &str,
    backup_file: Result<String, String>,
) -> Result<String, String> {
    let (now, want, declared) = (
        now.to_lowercase(),
        want.to_lowercase(),
        declared.trim().to_lowercase(),
    );
    let holds = if now == want {
        format!("its backup ({want})")
    } else if now == declared {
        format!("the declared image ({declared})")
    } else {
        format!("content not recorded here ({now})")
    };
    match backup_file {
        Err(e) => Err(format!("{target} holds {holds}, but the stock backup {target}.img is not on file or unreadable: {e}: the flash could not be undone")),
        Ok(on_file) if on_file.to_lowercase() != want => Err(format!("{target} holds {holds}, but the backup file {target}.img hashes to {on_file}, not the recorded {want}: the flash could not be undone")),
        Ok(_) => Ok(format!("{target} holds {holds}; the stock backup is on file and hashes to its record ({want}): the flash can be undone")),
    }
}

/// Check 16's decision on the six partitions' hashes read before and after the flash (`name sha256` per line): no partition other than `target` changed. `target` itself may have changed
/// (a stock boot partition that took the image) or not (it already held the image, and a flash of the same bytes changes nothing: rerun 3 of 2026-11-23 stopped on demanding a change). That `target`
/// reads back as the image is checked by the caller from its own read.
fn only_the_target_may_change(
    before: &str,
    after: &str,
    target: &str,
    image: &str,
) -> Result<String, String> {
    let (b, a): (Vec<&str>, Vec<&str>) = (before.lines().collect(), after.lines().collect());
    let name = |l: &str| l.split_whitespace().next().unwrap_or("?").to_string();
    if b.is_empty()
        || !b.iter().any(|l| name(l) == target)
        || b.len() != a.len()
        || b.iter().zip(&a).any(|(x, y)| name(x) != name(y))
    {
        return Err(format!("the partitions read after the flash are not the ones read before:\n{before}\nthen\n{after}"));
    }
    if !a.iter().any(|l| {
        l.split_whitespace()
            .eq([target, image.to_lowercase().as_str()])
    }) {
        return Err(format!(
            "{target} does not read as {image} in the list read after the flash:\n{after}"
        ));
    }
    let changed: Vec<String> = b
        .iter()
        .zip(&a)
        .filter(|(x, y)| x != y)
        .map(|(x, _)| name(x))
        .collect();
    if let Some(other) = changed.iter().find(|n| *n != target) {
        return Err(format!("the partitions that changed with the flash are {changed:?}, expected none but {target}: {other} changed"));
    }
    Ok(if changed.is_empty() {
        format!("{target} held the image already and the flash changed no partition")
    } else {
        format!("only {target} changed, no other boot, dtbo or vbmeta partition")
    })
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
    /// the journal program (`journalctl`; a test names a stand-in): the lab host's kernel log of the wait
    journalctl: String,
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
            journalctl: "journalctl".to_string(),
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
        crate::reboot_tool::ssh_from(&self.s, &self.ssh, command, seconds, None)
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
            host::avb_footer_verifies("avbtool", &u.s.image)?;
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
            host::image_has_sha256(&u.s.reboot_tool, &u.s.reboot_tool_sha256).map_err(|e| {
                format!("the reboot tool the way back pushes is not the declared one: {e}")
            })?;
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
            // research walleye-stage2-checks-should-they-exist, row 8: a host marker of an earlier unfinished switch is the host's memory of an intention, not the phone's state (the phone is on Android now: adb answered); the flags and the GPT are read at 10 and 13 and are the truth, so the marker only gets a note
            if u.marked(SWITCHED_FILE).is_some() && u.marked(RESTORED_FILE).is_none() {
                println!("  note: an earlier run of this day switched to a slot and never wrote its way back; the phone is on Android now, and its flags and GPT are read below");
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

    /// The boot partition about to be overwritten, read now from Android, is what the stored backup recorded, (recorded, never refused: research walleye-stage2-checks-should-they-exist row 9), the backup being on file and still hashing to its record: the flash can be undone. The partition not flashed, its dtbo and vbmeta, and the
    /// flashed slot's dtbo and vbmeta are hashed too, kept for the end.
    pub fn the_slot_partitions_are_read_before(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let target = u.target()?;
            let sums = std::fs::read_to_string(u.s.backup.join("sha256sums"))
                .map_err(|e| format!("backup: {e}"))?;
            let want = crate::install::recorded_sha256(&sums, &format!("{target}.img"))?;
            let now = u.android.sha256(&u.by_name_of(&target), None)?;
            let file = u.s.backup.join(format!("{target}.img"));
            let said = boot_partition_can_be_flashed(
                &target,
                &now,
                &want,
                &u.s.image_sha256,
                host::sha256_of(&file),
            )?;
            if now.to_lowercase() != want.to_lowercase()
                && now.to_lowercase() != u.s.image_sha256.trim().to_lowercase()
            {
                // content nobody recorded is overwritten by the flash: a copy is kept first (research row 9)
                let kept = u.s.out.join(format!("{target}-before-the-flash.img"));
                u.android.copy_to(&u.by_name_of(&target), &kept)?;
                println!("  kept a copy of what {target} held: {}", kept.display());
            }
            u.read_parts("parts-before.txt")?;
            println!("  {said}");
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

    /// The slots' flags, read in fastboot, kept; read before the flash (`slots-before.txt`) they also stop the run when the slot Android runs from is not successful and bootable (the way back).
    pub fn the_slot_flags_are_read(&self, file: &'static str) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.flags(file)?;
            if file == "slots-before.txt" {
                android_slot_is_a_safe_way_back(&u.saved(file)?, &u.android_slot()?)?;
            }
            Ok(())
        }
    }

    /// The rehearsal: the flags read again are kept beside the first and a difference is SAID, not refused (research walleye-stage2-checks-should-they-exist, rehearsal note: Android may mark its slot successful on its own, so equality is no invariant).
    pub fn the_slot_flags_are_the_same(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.flags("slots-again.txt")?;
            let (a, b) = (u.saved("slots-before.txt")?, u.saved("slots-again.txt")?);
            if a != b {
                println!("  note: the slot flags changed with nothing asked of the bootloader (Android may mark its slot successful on its own):\n{a}\nthen\n{b}");
            }
            Ok(())
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
            let said = only_the_target_may_change(&a, &b, &target, &u.s.image_sha256)?;
            println!("  {target} reads back as the image; {said}");
            Ok(())
        }
    }

    /// After the flash: Android's slot, the current slot and every flag but the flashed slot's own are as they were (the flashed slot's flags are the bootloader's; research walleye-stage2-checks-should-they-exist, row 18).
    pub fn the_flash_changed_no_flag(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.flags("slots-after-flash.txt")?;
            let (a, b) = (
                u.saved("slots-before.txt")?,
                u.saved("slots-after-flash.txt")?,
            );
            let target = u.target()?; // boot_a or boot_b: the slot is its last letter
            flash_leaves_androids_slot_alone(&a, &b, &target[target.len() - 1..])
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
            u.lab.save(
                SWITCHED_AT_FILE,
                &now_epoch()
                    .ok_or("the clock could not be read")?
                    .to_string(),
            )?;
            u.fastboot(&set_active_args(&u.lab.serial()?, &slot)?)
                .map(|_| ())
        }
    }

    /// After the switch, in fastboot: the current slot is the new one, not unbootable, with tries left (row 20).
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
                match u.ssh("echo ---id; id -u; echo ---cmdline; cat /proc/cmdline; echo ---pid1; cat /proc/1/cmdline; echo; echo ---mounts; cat /proc/mounts; echo ---dt-slot; { tr -d '\\000' < /proc/device-tree/firmware/android/slot_suffix; } 2>/dev/null; echo; echo ---end", 40) {
                    Ok(text) if text.contains("---end") => break text,
                    other => {
                        if Instant::now() >= until {
                            let why = format!("no ssh login within {} s as {}@{}: {}", u.s.device_wait, u.s.ssh_user, u.s.address, other.err().unwrap_or_else(|| "the read did not finish".into()));
                            return Err(u.pstore_first_after_no_ssh(why));
                        }
                        std::thread::sleep(Duration::from_secs(3));
                    }
                }
            };
            u.lab
                .save("new-slot-read.txt", &crate::touch::redact(&read))?;
            match u.keep_the_hosts_usb_log() {
                Ok(usb_lines) => println!("  the host's USB log of the wait: {usb_lines} lines of {} in {HOST_USB_LOG_FILE}", u.s.usb_log_match),
                Err(e) => println!("  the host's USB log of the wait was not kept: {e}"),
            }
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
            let image =
                std::fs::read(&u.s.image).map_err(|e| format!("{}: {e}", u.s.image.display()))?;
            header_has_no_root(&image)?;
            pid1_is_the_roots_init(get("pid1")?)?;
            let device = mounts_are_read_only(get("mounts")?)?;
            let root_letter = disk_letter_of(&device)?;
            let mut gpt = String::new();
            let mut root_hex = None;
            for letter in disk_letters(u.s.luns)? {
                let hex = u.ssh(&gpt_dump_command("/dev/sd", letter)?, 60)?;
                let hex: String = hex.split_whitespace().collect();
                if letter == root_letter {
                    root_hex = Some(hex.clone());
                }
                gpt.push_str(&format!("sd{letter} {hex}\n"));
            }
            u.lab.save("gpt-new-slot.txt", gpt.trim_end())?;
            let root_hex = root_hex.ok_or_else(|| {
                format!("the root {device} is on sd{root_letter}, a disk this run did not read")
            })?;
            let root_disk = dump_bytes(&root_hex)
                .map_err(|e| format!("sd{root_letter}, the root's disk: {e}"))?;
            let answer = booted_slot(get("cmdline")?, get("dt-slot")?, &root_disk)?;
            let want = slot_of_boot(&u.target()?)?.to_string();
            let flags = flags_of(&u.saved("slots-after-switch.txt")?);
            let current = flags.get("current-slot").map(String::as_str).unwrap_or("");
            if answer.slot != want || current != want {
                return Err(format!(
                    "the booted slot is {} ({}), the switch made slot {want} current and fastboot's current-slot says {current:?}: the bootloader booted another slot",
                    answer.slot,
                    answer.answers.iter().map(|(s, v)| format!("{s} says {v}")).collect::<Vec<_>>().join(", ")
                ));
            }
            let said: Vec<String> = answer
                .answers
                .iter()
                .map(|(s, v)| format!("{s}: _{v}"))
                .collect();
            u.lab.save(
                "new-slot-source.txt",
                &format!(
                    "slot _{} from {} (answered: {})",
                    answer.slot,
                    answer.answers[0].0,
                    said.join(", ")
                ),
            )?;
            println!("  booted slot _{} from {} (answered: {}), equal to fastboot's current-slot; {device} is mounted read-only under the overlay", answer.slot, answer.answers[0].0, said.join(", "));
            Ok(())
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

    /// The lab host's kernel log since the switch, the lines of the phone's port (`usb_log_match`), kept as `host-usb-log.txt`: what the host saw of the phone's USB while the new slot booted
    /// (a gadget of ours, Android's own, or nothing: run 7 saw nothing for 67 s). Read-only: `journalctl -k`; the phone is sent nothing.
    fn keep_the_hosts_usb_log(&self) -> Result<usize, String> {
        let since: i64 = self
            .saved(SWITCHED_AT_FILE)?
            .trim()
            .parse()
            .map_err(|e| format!("{SWITCHED_AT_FILE}: {e}"))?;
        let mut c = Command::new(&self.journalctl);
        c.args(["-k", "--no-pager", "--since", &format!("@{since}")]);
        let said = host::run(c)?;
        if !said.ok {
            return Err(format!("journalctl: {}", said.text));
        }
        if !said.text.lines().any(|l| l.contains(" kernel: ")) {
            return Err(format!(
                "journalctl printed no kernel line at all (no right to read the journal?): {}",
                said.text.lines().next().unwrap_or("")
            ));
        }
        let (section, lines) = usb_log_section(&said.text, since, &self.s.usb_log_match);
        self.lab.save(HOST_USB_LOG_FILE, &section)?;
        Ok(lines)
    }

    /// The failed-boot file carries the host's USB log of the wait too (appended to what the phone said, or alone when Android did not answer); what was done, as words for the step's reason.
    fn the_hosts_usb_log_with_the_failed_boot(&self) -> String {
        let kept = self.keep_the_hosts_usb_log().and_then(|lines| {
            let section = self.saved(HOST_USB_LOG_FILE)?;
            let before = self.saved(FAILED_BOOT_FILE).unwrap_or_default();
            let text = if before.trim().is_empty() {
                section
            } else {
                format!("{}\n{section}", before.trim_end())
            };
            self.lab.save(FAILED_BOOT_FILE, &text)?;
            Ok(lines)
        });
        match kept {
            Ok(n) => format!("; the host's USB log of the wait ({n} lines of {}) is in {HOST_USB_LOG_FILE} and {FAILED_BOOT_FILE}", self.s.usb_log_match),
            Err(e) => format!("; the host's USB log of the wait was not kept: {e}"),
        }
    }

    /// The new slot gave no ssh. Sends the phone NOTHING (no reboot, no fastboot verb, no way back) while it waits up to `fallback_wait` seconds for the bootloader's own
    /// fallback to bring Android up and adb to answer; the FIRST thing it does then is the read-only read of the failed boot (pstore first), kept and printed. What it did
    /// is appended to the reason the step stops with; the existing way back runs after, from the watching parent.
    fn pstore_first_after_no_ssh(&self, why: String) -> String {
        println!(
            "  no ssh from the new slot: sending the phone nothing; waiting up to {} s for Android to answer adb, then the pstore is read first",
            self.s.fallback_wait
        );
        let waited = match self.android_answers_adb_within(self.s.fallback_wait) {
            Ok(w) => w,
            Err(e) => {
                return format!(
                    "{why}; {e}{}",
                    self.the_hosts_usb_log_with_the_failed_boot()
                )
            }
        };
        println!("  adb answered {waited} s after the wait began");
        let said = match self.read_the_failed_boot(FAILED_BOOT_FILE) {
            Ok(_) => format!("{why}; Android answered adb {waited} s after the wait began and the failed boot was read first: {FAILED_BOOT_FILE} in the run's folder (pstore, command line, boot reason, slot)"),
            Err(e) => format!("{why}; Android answered adb {waited} s after the wait began, but the read of the failed boot did not finish: {e}"),
        };
        format!("{said}{}", self.the_hosts_usb_log_with_the_failed_boot())
    }

    /// Waits, calling nothing that could restart the phone (`adb get-state` and the listing of fastboot's devices only read), until adb says `device`: the seconds it took. A phone that
    /// fastboot lists is not on Android: said, not waited for.
    fn android_answers_adb_within(&self, secs: u64) -> Result<u64, String> {
        let began = Instant::now();
        let until = began + Duration::from_secs(secs);
        loop {
            if self
                .android
                .adb(&["get-state"])
                .is_ok_and(|s| s.trim() == "device")
            {
                return Ok(began.elapsed().as_secs());
            }
            if self.lab.in_fastboot().unwrap_or(false) {
                return Err(
                    "the phone is in the bootloader, not on Android: nothing was read".into(),
                );
            }
            if Instant::now() >= until {
                return Err(format!("adb did not answer within {secs} s of the wait (the bootloader's fallback to Android took about 3 to 4 minutes in run 4): nothing was read"));
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    /// The read of a failed boot over adb, kept as `file` (the phone's serial redacted) and printed. When the pstore is closed to the shell user, `adb root` (it restarts adbd, not
    /// the phone) is asked once and the read is taken again, both kept.
    fn read_the_failed_boot(&self, file: &str) -> Result<String, String> {
        let take = || failed_boot_report(&mut |c| self.android.adb(&["shell", c]));
        let mut text = take();
        // adbd early in a boot may still be restarting: a listing that was not read is asked again, a few times
        for _ in 0..3 {
            if !text.contains("---pstore-listing\n(not read") {
                break;
            }
            std::thread::sleep(Duration::from_secs(2));
            text = take();
        }
        // what was read is kept and shown before anything else is asked of the phone
        let first = crate::touch::redact(&text);
        println!("{first}");
        self.lab.save(file, &first)?;
        if pstore_is_closed(&text) {
            let rooted = self.android.adb(&["root"]);
            text.push_str(&format!(
                "---adb-root\n{}\n",
                rooted.unwrap_or_else(|e| format!("(not done: {e})"))
            ));
            // adbd answers as root, not just as a device: the old adbd may still say `device` as it ends
            match self.adb_answers_as_root_within(ROOT_ADB_WAIT) {
                Ok(()) => text.push_str(&take()),
                Err(e) => text.push_str(&format!("---adb-root-wait\n(not done: {e})\n")),
            }
            let all = crate::touch::redact(&text);
            println!("{all}");
            self.lab.save(file, &all)?;
        }
        Ok(crate::touch::redact(&text))
    }

    fn adb_answers_as_root_within(&self, secs: u64) -> Done {
        let until = Instant::now() + Duration::from_secs(secs);
        loop {
            match self.android.adb(&["shell", "id -u"]) {
                Ok(id) if id.trim() == "0" => return Ok(()),
                other if Instant::now() >= until => {
                    return Err(format!(
                        "adbd did not answer as root within {secs} s: {other:?}"
                    ))
                }
                _ => std::thread::sleep(Duration::from_secs(1)),
            }
        }
    }

    /// The rehearsal's step, safe on Android as it is: the same read, taken now, has its pstore listing, the command line with the slot Android runs from, and a boot reason.
    pub fn the_failed_boot_read_works_on_android(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let text = u.read_the_failed_boot("rehearsal-failed-boot-read.txt")?;
            let parts = sections(&text);
            let get = |n: &str| {
                parts
                    .iter()
                    .rev()
                    .find(|(k, _)| k == n)
                    .map(|(_, t)| t.trim().to_string())
                    .ok_or_else(|| format!("the read has no {n} section"))
            };
            for n in ["pstore-listing", "cmdline", "bootreason", "slot-suffix"] {
                let body = get(n)?;
                if body.is_empty() || body.starts_with("(not read") {
                    return Err(format!("the {n} section was not read: {body:?}"));
                }
            }
            let want = format!("_{}", u.android_slot()?);
            if get("slot-suffix")? != want {
                return Err(format!(
                    "ro.boot.slot_suffix says {:?}, adb read {want:?} before",
                    get("slot-suffix")?
                ));
            }
            if !get("cmdline")?
                .split_whitespace()
                .any(|w| w == format!("androidboot.slot_suffix={want}"))
            {
                return Err("the command line does not carry the slot suffix".into());
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
            // the same way as stage 1's way back (one function, crate::reboot_tool): the static tool is checked, pushed to the phone's tmpfs and run there;
            // the phone hangs up as it goes down, so what it said is kept only to be named when the bootloader is not reached
            let sent = crate::reboot_tool::ask_for_the_bootloader(
                &self.s,
                &self.ssh,
                &self.s.to_bootloader,
            );
            let until = Instant::now() + Duration::from_secs(self.s.device_wait);
            while !self.lab.in_fastboot().unwrap_or(false) {
                if Instant::now() >= until {
                    return Err(self.hand_step(&android, sent.err().as_deref()));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
            println!(
                "way back: fastboot lists the phone after the reboot tool was asked (RESTART2)"
            );
        } else {
            println!("way back: the phone was already in the bootloader");
        }
        // in the bootloader: a failure from here on leaves the phone there on our slot, so it too carries the hand command
        self.set_android_slot_active(&android).map_err(|e| {
            format!("{e}; by hand: fastboot set_active {android} && fastboot reboot")
        })?;
        self.lab.save(RESTORED_FILE, &android)?;
        Ok(())
    }

    /// A phone that is already on Android's slot (the switch never happened, or was done by hand) is not written to.
    fn set_android_slot_active(&self, android: &str) -> Done {
        let now = parse_current_slot(&self.fastboot_getvar("current-slot")?)?;
        if now != android {
            self.fastboot(&set_active_args(&self.lab.serial()?, android)?)?;
        }
        Ok(())
    }

    /// The one hand step that is left when our kernel does not reach the bootloader, with the exact command and what the ask for the bootloader said when it failed.
    fn hand_step(&self, android: &str, asked: Option<&str>) -> String {
        let asked = asked.map_or(String::new(), |e| format!(" (the ask said: {e})"));
        format!(
            "the phone did not reach the bootloader within {} s of `{}`{asked} (rerun 7 reached it this way from stage 1's kernel; stage 2's is not known yet): hold power and volume-down until the bootloader screen shows, then run: fastboot set_active {android} && fastboot reboot",
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

    /// On Android again: the boot, dtbo and vbmeta partitions are as the flash left them (nothing differs from the read after the flash) and the GPT is read again.
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

// ---------------------------------------------------------------------------------------------------------------------
// the read of a failed boot (item walleye-stage2-pstore-first-on-fallback)
// ---------------------------------------------------------------------------------------------------------------------

/// The file the read is kept in, in the run's folder (the console of the new slot's boot, when Android's kernel kept it).
pub const FAILED_BOOT_FILE: &str = "failed-boot-read.txt";

/// The lines of a journal text (`journalctl -k --since @SINCE`) that carry `matches` (`usb 3-2`), with the phone's serial number redacted, under a heading that says what they are; and how many there are.
/// No line is said so: the host saw no USB event of the phone, which is itself the reading.
pub fn usb_log_section(journal: &str, since: i64, matches: &str) -> (String, usize) {
    let lines: Vec<String> = journal
        .lines()
        .filter(|l| l.contains(matches))
        .map(|l| match l.find("SerialNumber: ") {
            Some(at) if !l[at..].starts_with("SerialNumber: REDACTED") => {
                format!("{}SerialNumber: REDACTED", &l[..at])
            }
            _ => l.to_string(),
        })
        .collect();
    let head = format!("---host-usb-log (journalctl -k --since @{since}, lines with '{matches}')");
    if lines.is_empty() {
        (
            format!(
                "{head}\n(no line: the host saw no USB event of the phone's port since the switch)"
            ),
            0,
        )
    } else {
        (format!("{head}\n{}", lines.join("\n")), lines.len())
    }
}

/// The pstore of Android's kernel: the previous boot's console is `console-ramoops-0` there, until the next restart.
const PSTORE: &str = "/sys/fs/pstore";

/// How long adb has to answer again after `adb root` restarts adbd.
const ROOT_ADB_WAIT: u64 = 40;

/// What adb reads after the pstore, one adb call each, read only: nothing here restarts the phone, writes, or changes a property
/// (the test `the_failed_boot_read_only_reads` holds the list to that). The phone's own words for why the last boot ended.
pub const FAILED_BOOT_READS: [(&str, &str); 7] = [
    ("uptime", "uptime"),
    ("id", "id"),
    ("cmdline", "cat /proc/cmdline"),
    ("bootreason", "getprop ro.boot.bootreason"),
    ("sys-bootreason", "getprop sys.boot.reason"),
    ("slot-suffix", "getprop ro.boot.slot_suffix"),
    ("verifiedbootstate", "getprop ro.boot.verifiedbootstate"),
];

/// The names of the regular files in an `ls -la` listing (the pstore's `console-ramoops-0`, `dmesg-ramoops-0`); a name with a character
/// that is not a letter, a digit or one of `. _ -` is left out, never put into a command.
pub fn pstore_files(listing: &str) -> Vec<String> {
    listing
        .lines()
        .filter(|l| l.starts_with('-'))
        .filter_map(|l| l.split_whitespace().last())
        .filter(|n| {
            !n.is_empty()
                && n.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
        })
        .map(str::to_string)
        .collect()
}

/// The text of one section of a read: `---NAME` then what the phone said (an unreadable one says why).
fn section(name: &str, body: &str) -> String {
    format!("---{name}\n{}\n", body.trim_end())
}

/// One read-only command on the phone; a command that fails is written as not read, with its reason, and the read goes on.
fn said(ask: &mut dyn FnMut(&str) -> Result<String, String>, command: &str) -> String {
    ask(command).unwrap_or_else(|e| format!("(not read: {e})"))
}

/// The read of a failed boot, in its order: the pstore FIRST (the listing, then each file whole), then the rest. `ask` runs one read-only
/// command on the phone.
pub fn failed_boot_report(ask: &mut dyn FnMut(&str) -> Result<String, String>) -> String {
    let listing = said(ask, &format!("ls -la {PSTORE}"));
    let mut out = section("pstore-listing", &listing);
    for name in pstore_files(&listing) {
        let body = said(ask, &format!("cat {PSTORE}/{name}"));
        out.push_str(&section(&format!("pstore:{name}"), &body));
    }
    for (name, command) in FAILED_BOOT_READS {
        out.push_str(&section(name, &said(ask, command)));
    }
    out.push_str("---end\n");
    out
}

/// The listing or a file says the pstore is closed to this user (adb is the shell user, not root): the one thing that makes a second read worth asking `adb root` for.
pub fn pstore_is_closed(report: &str) -> bool {
    sections(report)
        .iter()
        .filter(|(k, _)| k.starts_with("pstore"))
        .any(|(_, t)| t.contains("Permission denied"))
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

    /// The backup entries of a table of 128 entries.
    const B_ENT: usize = HEAD_BLOCKS * GB;

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
        // research walleye-stage2-checks-should-they-exist, row 20: the other flags (the new slot's successful, Android's slot's own) are the bootloader's, recorded and judged at the way back, not here
        for ok in [
            after_switch().replace("slot-successful:b:yes", "slot-successful:b:no"),
            after_switch().replace("slot-successful:a:no", "slot-successful:a:yes"),
            after_switch().replace("slot-retry-count:a:6", "slot-retry-count:a:9"),
        ] {
            assert!(switch_is_the_abls(BEFORE, &ok, "b", "a").is_ok(), "{ok}");
        }
        // the new slot is unbootable or has no tries left, or the switch did not happen
        for (what, v) in [
            (
                "unbootable",
                after_switch().replace("slot-unbootable:a:no", "slot-unbootable:a:yes"),
            ),
            (
                "retry 0",
                after_switch().replace("slot-retry-count:a:6", "slot-retry-count:a:0"),
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

    /// The blocks the entry array of `count` entries fills (4096-byte blocks, 128-byte entries).
    fn blocks_for(count: usize) -> usize {
        (count * ENTRY).div_ceil(GB)
    }

    /// Where the backup copy of `count` entries lies in the dump: in the blocks just before the backup header.
    fn b_ent_of(count: usize) -> usize {
        B_HDR - blocks_for(count) * GB
    }

    /// A synthetic dump of one disk whose headers say `count` entries, with the A/B pairs given (at most `count`), each entry: type GUID filled with a byte, attributes filled with a byte.
    fn dump_n(count: usize, entries: &[(&str, u8, u8)]) -> Vec<u8> {
        let mut d = vec![0u8; DUMP_LEN];
        let last_lba: u64 = 1000;
        for (hdr, cur, ent) in [
            (P_HDR, 1, 2),
            (B_HDR, last_lba, last_lba - blocks_for(count) as u64),
        ] {
            d[hdr..hdr + 8].copy_from_slice(b"EFI PART");
            d[hdr + 12..hdr + 16].copy_from_slice(&92u32.to_le_bytes());
            d[hdr + 24..hdr + 32].copy_from_slice(&u64::to_le_bytes(cur));
            d[hdr + 72..hdr + 80].copy_from_slice(&u64::to_le_bytes(ent));
            d[hdr + 80..hdr + 84].copy_from_slice(&(count as u32).to_le_bytes());
            d[hdr + 84..hdr + 88].copy_from_slice(&128u32.to_le_bytes());
        }
        for (i, (name, guid, attr)) in entries.iter().enumerate().take(count) {
            for base in [P_ENT, b_ent_of(count)] {
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

    /// The CRCs as a writer makes them: the entries' over what the header says (count, and where its entry LBA puts them), then the header's own.
    fn fix_crcs(d: &mut [u8]) {
        for hdr in [P_HDR, B_HDR] {
            let count = u32::from_le_bytes(d[hdr + 80..hdr + 84].try_into().unwrap()) as usize;
            let ent = if hdr == P_HDR {
                P_ENT
            } else {
                let back = u64::from_le_bytes(d[hdr + 24..hdr + 32].try_into().unwrap())
                    - u64::from_le_bytes(d[hdr + 72..hdr + 80].try_into().unwrap());
                hdr - back as usize * GB
            };
            let ec = crc32(&d[ent..(ent + count * ENTRY).min(d.len())]);
            d[hdr + 88..hdr + 92].copy_from_slice(&ec.to_le_bytes());
            d[hdr + 16..hdr + 20].fill(0);
            let hc = crc32(&d[hdr..hdr + 92]);
            d[hdr + 16..hdr + 20].copy_from_slice(&hc.to_le_bytes());
        }
    }

    fn before_n(count: usize) -> Vec<u8> {
        dump_n(
            count,
            &[
                ("boot_a", 1, 0x10),
                ("boot_b", 2, 0x20),
                ("system_a", 3, 0x10),
                ("system_b", 4, 0x20),
                ("userdata", 9, 0),
            ],
        )
    }

    fn before() -> Vec<u8> {
        before_n(128)
    }

    /// What the bootloader's switch to slot a does: the type GUIDs of every A/B pair swap, the attributes change, the CRCs are recomputed, in both copies.
    fn switched_n(count: usize) -> Vec<u8> {
        let mut d = before_n(count);
        for base in [P_ENT, b_ent_of(count)] {
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

    fn switched() -> Vec<u8> {
        switched_n(128)
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
    fn a_gpt_says_how_many_entries_it_has_and_the_switch_passes_on_one_with_few() {
        // the phone's disks: 48 entries (sda), 4 (sdb, sdc), 20, 8 - all of 128 bytes
        for count in [4, 5, 8, 20, 48, 128] {
            assert!(gpt_is_whole(&before_n(count)).is_ok(), "{count}");
            assert!(gpt_switch_only(&before_n(count), &before_n(count)).is_ok());
            let said = gpt_switch_only(&before_n(count), &switched_n(count)).unwrap();
            let attrs = count.min(4);
            assert!(
                said.contains(&format!(
                    "{attrs} entries' slot bits and {attrs} type GUIDs"
                )),
                "{count}: {said}"
            );
        }
        // the bytes after the last entry (the rest of its block) are no slot bit
        let mut d = switched_n(4);
        d[P_ENT + 4 * ENTRY + 5] = 1;
        assert!(gpt_switch_only(&before_n(4), &d)
            .unwrap_err()
            .contains("not a slot bit"));
        // the entry array of the backup is where its header puts it, not at a fixed place
        let mut d = switched_n(4);
        d[b_ent_of(4)..b_ent_of(4) + 16].fill(0xEE);
        fix_crcs(&mut d);
        assert!(gpt_switch_only(&before_n(4), &d).is_err());
        // a disk that had 48 entries and has 4 has been repartitioned, not switched
        assert!(gpt_switch_only(&before_n(48), &before_n(4))
            .unwrap_err()
            .contains("entries"));
    }

    #[test]
    fn a_gpt_with_a_count_a_size_a_crc_or_a_place_that_is_wrong_is_not_whole() {
        let edit = |f: &dyn Fn(&mut Vec<u8>)| {
            let mut d = before_n(48);
            f(&mut d);
            d
        };
        // 129 entries do not fit the 4 blocks the dump holds; none is no table
        for count in [129u32, 0] {
            let d = edit(&|d| {
                for hdr in [P_HDR, B_HDR] {
                    d[hdr + 80..hdr + 84].copy_from_slice(&count.to_le_bytes());
                }
                fix_crcs(d);
            });
            assert!(gpt_is_whole(&d).unwrap_err().contains("entries"), "{count}");
        }
        // an entry size of 64 (or any but 128)
        let d = edit(&|d| {
            for hdr in [P_HDR, B_HDR] {
                d[hdr + 84..hdr + 88].copy_from_slice(&64u32.to_le_bytes());
            }
            fix_crcs(d);
        });
        assert!(gpt_is_whole(&d).unwrap_err().contains("entries"));
        // a wrong entries CRC, in either copy
        for hdr in [P_HDR, B_HDR] {
            let mut d = before_n(48);
            d[hdr + 88] ^= 1;
            // the header CRC is made right again, so only the entries' is wrong
            d[hdr + 16..hdr + 20].fill(0);
            let hc = crc32(&d[hdr..hdr + 92]);
            d[hdr + 16..hdr + 20].copy_from_slice(&hc.to_le_bytes());
            assert!(gpt_is_whole(&d).unwrap_err().contains("entries CRC"));
        }
        // the entries CRC covers count*128 bytes and no more
        let mut d = before_n(48);
        d[P_ENT + 48 * ENTRY + 1] = 1;
        assert!(gpt_is_whole(&d).is_ok());
        // the two copies of a disk agree on the count
        let d = edit(&|d| {
            d[B_HDR + 80..B_HDR + 84].copy_from_slice(&40u32.to_le_bytes());
            fix_crcs(d);
        });
        assert!(gpt_is_whole(&d).unwrap_err().contains("disagree"));
        // the backup's entries lie within the four blocks before its header and hold the table
        for blocks in [1u64, 5] {
            let d = edit(&|d| {
                let cur = u64::from_le_bytes(d[B_HDR + 24..B_HDR + 32].try_into().unwrap());
                d[B_HDR + 72..B_HDR + 80].copy_from_slice(&(cur - blocks).to_le_bytes());
                fix_crcs(d);
            });
            assert!(gpt_is_whole(&d).unwrap_err().contains("backup"), "{blocks}");
        }
        // the primary's entries start at LBA 2
        let d = edit(&|d| {
            d[P_HDR + 72..P_HDR + 80].copy_from_slice(&3u64.to_le_bytes());
            fix_crcs(d);
        });
        assert!(gpt_is_whole(&d).is_err());
        // a short dump is an error, not a panic
        assert!(gpt_is_whole(&[0u8; 10]).is_err());
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

    /// The gate on the stage 2 DEBUG image of the lab (item walleye-stage2-debug-image): its init passes the gate too, and its command line carries both words, walleye.booted-slot and walleye.debug.
    #[test]
    fn the_init_gate_accepts_the_stage_2_debug_image_of_the_lab() {
        let home = std::env::var("HOME").unwrap_or_default();
        let root = format!("{home}/Projects/walleye-build/NOT-TO-BOOT-ufs-on-images-ufs-root-booted-slot-debug/NOT-TO-BOOT-ufs-on-ufs-root-booted-slot-debug.img");
        if !std::path::Path::new(&root).exists()
            || Command::new("lpoco-bringup")
                .arg("--version")
                .output()
                .is_err()
        {
            eprintln!("NOT KNOWN YET: the lab's stage 2 debug image or lpoco-bringup is not on this machine: the init gate is not run on it here");
            return;
        }
        let (f, _d) = bench_image("gate3", "b", Some(root));
        (f.the_init_accepts_the_slot_it_is_booted_from())().unwrap();
        let bytes = std::fs::read(&f.s.image).unwrap();
        let cmdline = crate::rootfs_boot::header_cmdline(&bytes).unwrap();
        for word in ["walleye.booted-slot", "walleye.debug"] {
            assert!(cmdline.split_whitespace().any(|w| w == word), "{word}");
        }
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
        let said = f.hand_step("b", None);
        assert!(
            said.contains("fastboot set_active b && fastboot reboot")
                && said.contains("walleye-reboot --command bootloader"),
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

    // ---- the way back asks for the bootloader with the reboot tool, as stage 1's does (item walleye-install-stage2-way-back-restart2) ----

    /// A phone on our slot that goes to fastboot only when asked with the tool: `ssh` records its calls, keeps the pushed bytes, and on
    /// the tool's command (`run_says` = None) makes fastboot list the phone, or fails like a /tmp mounted noexec.
    fn bench_asked(name: &str, run_says: Option<&str>) -> (FlashBoot, std::path::PathBuf) {
        let (mut f, d) = bench(name, "a");
        let ssh = d.join("ssh");
        let run = match run_says {
            None => format!("touch {}/in-fastboot", d.display()),
            Some(said) => format!("echo '{said}' >&2; exit 126"),
        };
        executable(
            &ssh,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/ssh.calls\ncase \"$*\" in *'cat > /tmp/walleye-reboot'*) cat > {d}/pushed;; *'/tmp/walleye-reboot --command bootloader'*) {run};; esac\nexit 0\n",
                d = d.display()
            ),
        );
        let fb = d.join("fastboot");
        executable(
            &fb,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/fastboot.calls\ncase \"$4\" in current-slot) echo \"current-slot: a\" >&2 ;; esac\ncase \"$1\" in devices) [ -e {d}/in-fastboot ] && printf 'SER1\\tfastboot\\n' ;; esac\nexit 0\n",
                d = d.display()
            ),
        );
        let key = d.join("key");
        std::fs::write(&key, "k").unwrap();
        std::fs::set_permissions(&key, std::os::unix::fs::PermissionsExt::from_mode(0o600))
            .unwrap();
        let tool = d.join("walleye-reboot");
        std::fs::write(&tool, b"\x7fELF stand-in for the static tool").unwrap();
        let mut s = (*f.s).clone();
        s.reboot_tool_sha256 = host::sha256_of(&tool).unwrap();
        s.reboot_tool = tool;
        s.identity = key;
        s.address = "172.16.43.1".into();
        s.device_wait = 1;
        f.s = std::rc::Rc::new(s);
        f.ssh = ssh.to_string_lossy().into_owned();
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        (f, d)
    }

    fn ssh_calls(d: &std::path::Path) -> String {
        std::fs::read_to_string(d.join("ssh.calls")).unwrap_or_default()
    }

    #[test]
    fn the_switch_back_pushes_the_tool_runs_it_and_then_sets_androids_slot() {
        let (f, d) = bench_asked("asked", None);
        (f.the_switch_is_made_back())().unwrap();
        let ssh = ssh_calls(&d);
        let (push, run) = (
            ssh.find("cat > /tmp/walleye-reboot").unwrap(),
            ssh.find("/tmp/walleye-reboot --command bootloader")
                .unwrap(),
        );
        assert!(push < run && !ssh.contains("reboot bootloader"), "{ssh}");
        assert_eq!(
            std::fs::read(d.join("pushed")).unwrap(),
            std::fs::read(&f.s.reboot_tool).unwrap()
        );
        assert!(calls(&d).contains("-s SER1 set_active b"), "{}", calls(&d));
    }

    #[test]
    fn a_tmp_that_will_not_run_the_tool_ends_in_the_hand_step_with_the_exact_command() {
        let (f, d) = bench_asked("noexec", Some("sh: /tmp/walleye-reboot: Permission denied"));
        let said = (f.the_switch_is_made_back())().unwrap_err();
        assert!(
            said.contains("fastboot set_active b && fastboot reboot")
                && said.contains("Permission denied")
                && said.contains("/tmp/walleye-reboot --command bootloader"),
            "{said}"
        );
        assert!(!calls(&d).contains("set_active"), "{}", calls(&d));
    }

    #[test]
    fn a_tool_that_is_not_the_declared_one_is_never_sent_and_the_hand_step_says_so() {
        let (mut f, d) = bench_asked("wrongtool", None);
        let mut s = (*f.s).clone();
        s.reboot_tool_sha256 = "0000".into();
        f.s = std::rc::Rc::new(s);
        let said = (f.the_switch_is_made_back())().unwrap_err();
        assert!(said.contains("not the declared one"), "{said}");
        assert_eq!(ssh_calls(&d), "");
    }

    #[test]
    fn a_phone_already_in_the_bootloader_is_not_sent_the_tool() {
        let (f, d) = bench_asked("already", None);
        std::fs::write(d.join("in-fastboot"), "").unwrap();
        (f.the_switch_is_made_back())().unwrap();
        assert_eq!(ssh_calls(&d), "");
    }

    #[test]
    fn a_failing_set_active_in_the_bootloader_carries_the_hand_command() {
        let (f, d) = bench_asked("setfail", None);
        let fb = d.join("fastboot");
        executable(
            &fb,
            "#!/bin/sh\ncase \"$1\" in devices) printf 'SER1\\tfastboot\\n' ;; esac\ncase \"$4\" in current-slot) echo \"current-slot: a\" >&2 ;; esac\ncase \"$3\" in set_active) echo refused >&2; exit 1;; esac\nexit 0\n",
        );
        let said = (f.the_switch_is_made_back())().unwrap_err();
        assert!(
            said.contains("by hand: fastboot set_active b && fastboot reboot"),
            "{said}"
        );
    }

    #[test]
    fn the_stored_set_step_refuses_a_tool_that_is_not_the_declared_one() {
        let (mut f, _d) = bench_asked("earlysha", None);
        let mut s = (*f.s).clone();
        s.reboot_tool_sha256 = "0000".into();
        f.s = std::rc::Rc::new(s);
        let said = (f.the_stored_set_and_the_key_are_good())().unwrap_err();
        assert!(
            said.contains("not the declared one")
                || said.contains("private")
                || said.contains("key"),
            "{said}"
        );
    }

    // ---- check 16 against a stand-in phone: partitions are files, `adb exec-out CMD` runs CMD here (item walleye-stage2-check-16-and-repeat-audit) ----

    const PARTS: [&str; 6] = [
        "boot_a", "boot_b", "dtbo_a", "dtbo_b", "vbmeta_a", "vbmeta_b",
    ];

    /// A bench whose phone holds the six partitions as files: boot_a is `boot_a_holds` ("image" or "stock"), the others a line of their own.
    fn phone_with_partitions(name: &str, boot_a_holds: &str) -> (FlashBoot, std::path::PathBuf) {
        let (f, d) = bench(name, "b");
        let by = d.join("by-name");
        std::fs::create_dir_all(&by).unwrap();
        let adb = d.join("adb");
        executable(
            &adb,
            "#!/bin/sh\nshift 2\ncase \"$1\" in exec-out) shift; exec sh -c \"$*\";; esac\nexit 0\n",
        );
        let mut s = (*f.s).clone();
        s.adb = adb.to_string_lossy().into_owned();
        s.by_name = by.to_string_lossy().into_owned();
        let f = FlashBoot::with_programs(s, "ssh", &d.join("tad").to_string_lossy());
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        for p in PARTS {
            let bytes = match (p, boot_a_holds) {
                ("boot_a", "image") => std::fs::read(&f.s.image).unwrap(),
                _ => format!("{p} as the phone has it, {boot_a_holds}").into_bytes(),
            };
            std::fs::write(by.join(p), bytes).unwrap();
        }
        (f, d)
    }

    /// The text `read_parts` keeps, from the files now in the by-name folder.
    fn parts_text(f: &FlashBoot) -> String {
        PARTS
            .iter()
            .map(|p| {
                let sum = host::sha256_of(std::path::Path::new(&f.by_name_of(p))).unwrap();
                format!("{p} {sum}")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_flash_of_the_image_boot_a_already_holds_changes_nothing_and_check_16_passes() {
        // rerun 3: boot_a was e337de53 before the flash and after it
        let (f, _d) = phone_with_partitions("c16-repeat", "image");
        f.lab.save("parts-before.txt", &parts_text(&f)).unwrap();
        (f.the_partition_reads_back_as_the_image())().unwrap();
    }

    #[test]
    fn a_flash_that_replaced_the_stock_boot_a_still_passes_check_16() {
        let (f, _d) = phone_with_partitions("c16-first", "stock");
        f.lab.save("parts-before.txt", &parts_text(&f)).unwrap();
        std::fs::copy(&f.s.image, f.by_name_of("boot_a")).unwrap();
        (f.the_partition_reads_back_as_the_image())().unwrap();
    }

    #[test]
    fn check_16_fails_when_boot_a_differs_from_the_image_after_the_flash() {
        let (f, _d) = phone_with_partitions("c16-differs", "image");
        f.lab.save("parts-before.txt", &parts_text(&f)).unwrap();
        std::fs::write(f.by_name_of("boot_a"), b"not the image").unwrap();
        let said = (f.the_partition_reads_back_as_the_image())().unwrap_err();
        assert!(said.contains("reads back as"), "{said}");
    }

    #[test]
    fn check_16_fails_when_dtbo_changed_with_the_flash_even_though_boot_a_holds_the_image() {
        let (f, _d) = phone_with_partitions("c16-dtbo", "image");
        f.lab.save("parts-before.txt", &parts_text(&f)).unwrap();
        std::fs::write(f.by_name_of("dtbo_a"), b"a dtbo the flash changed").unwrap();
        let said = (f.the_partition_reads_back_as_the_image())().unwrap_err();
        assert!(said.contains("dtbo_a"), "{said}");
    }

    #[test]
    fn check_16_fails_when_the_other_slots_boot_changed_with_the_flash() {
        let (f, _d) = phone_with_partitions("c16-boot-b", "image");
        f.lab.save("parts-before.txt", &parts_text(&f)).unwrap();
        std::fs::write(f.by_name_of("boot_b"), b"boot_b the flash changed").unwrap();
        let said = (f.the_partition_reads_back_as_the_image())().unwrap_err();
        assert!(said.contains("boot_b"), "{said}");
    }

    // ---- THE REPEAT AUDIT (item walleye-stage2-check-16-and-repeat-audit): the command's steps that read or write a partition, a flag or the slot, run in order against the state the phone is in after
    // rerun 3: boot_a holds the image (e337de53), slot a's flags are what the flash of run 1 left (retry 1, successful no: research walleye-display-s6e3fa5, stage2-run.log), Android on b, the stock backup on file ----

    /// What `fastboot getvar all` said after run 1's flash (printed in that run's log), sorted as `slot_lines` keeps it.
    const FLAGS_AFTER_RUN_1: &str = "current-slot:b\nslot-retry-count:a:1\nslot-retry-count:b:0\nslot-successful:a:no\nslot-successful:b:yes\nslot-unbootable:a:no\nslot-unbootable:b:no\n";

    /// The phone of the repeat: partitions as files, a stateful fastboot (flags in a file; `flash` copies the image, `set_active X` does what run 1's reading says the bootloader does), adb answering the slot, the stock backup on file.
    fn the_phone_after_rerun_3(name: &str, flags: &str) -> (FlashBoot, std::path::PathBuf) {
        let (f, d) = phone_with_partitions(name, "image");
        std::fs::write(d.join("fb-state"), flags).unwrap();
        executable(
            &d.join("adb"),
            "#!/bin/sh\nshift 2\ncase \"$1\" in\n exec-out) shift; exec sh -c \"$*\";;\n shell) case \"$2\" in 'getprop ro.boot.slot_suffix') echo _b;; esac;;\nesac\nexit 0\n",
        );
        executable(
            &d.join("fastboot"),
            &"#!/bin/sh\nD=@D@\necho \"$@\" >> $D/fastboot.calls\nS=$D/fb-state\ncase \"$1\" in devices) printf 'SER1\\tfastboot\\n'; exit 0;; esac\ncase \"$3\" in\n getvar) case \"$4\" in\n  current-slot) echo \"current-slot: $(sed -n 's/^current-slot://p' $S)\" >&2;;\n  all) sed 's/^/(bootloader) /' $S >&2;;\n esac;;\n flash) cp \"$5\" \"$D/by-name/$4\"; X=\"${4#boot_}\"; sed -i -e \"s/^slot-successful:$X:.*/slot-successful:$X:no/\" -e \"s/^slot-retry-count:$X:.*/slot-retry-count:$X:1/\" -e \"s/^slot-unbootable:$X:.*/slot-unbootable:$X:no/\" $S;;\n set_active) X=\"$4\"; sed -i -e \"s/^current-slot:.*/current-slot:$X/\" -e \"s/^slot-successful:$X:.*/slot-successful:$X:no/\" -e \"s/^slot-retry-count:$X:.*/slot-retry-count:$X:7/\" -e \"s/^slot-unbootable:$X:.*/slot-unbootable:$X:no/\" $S;;\nesac\nexit 0\n"
                .replace("@D@", &d.to_string_lossy()),
        );
        let backup = d.join("backup");
        std::fs::create_dir_all(&backup).unwrap();
        std::fs::write(backup.join("boot_a.img"), b"the stock boot_a").unwrap();
        let stock = host::sha256_of(&backup.join("boot_a.img")).unwrap();
        std::fs::write(backup.join("sha256sums"), format!("{stock}  boot_a.img\n")).unwrap();
        let mut s = (*f.s).clone();
        s.backup = backup;
        (
            FlashBoot::with_programs(s, "ssh", &d.join("tad").to_string_lossy()),
            d,
        )
    }

    #[test]
    fn every_step_that_reads_or_writes_the_slot_passes_against_the_phone_after_rerun_3() {
        let (f, d) = the_phone_after_rerun_3("repeat-walk", FLAGS_AFTER_RUN_1);
        (f.the_slot_is_read_by_adb())().unwrap(); // 8
        (f.the_slot_partitions_are_read_before())().unwrap(); // 9: boot_a holds the image, the backup on file
        (f.the_guard_holds())().unwrap(); // 12
        (f.the_slot_flags_are_read("slots-before.txt"))().unwrap(); // 13
        (f.the_boot_image_is_flashed())().unwrap(); // 14
        (f.the_partition_reads_back_as_the_image())().unwrap(); // 16
        (f.the_flash_changed_no_flag())().unwrap(); // 18: a stays retry 1, successful no
        (f.the_slot_is_made_active())().unwrap(); // 19
        (f.the_flags_are_the_switch_and_nothing_else())().unwrap(); // 20: a: successful no, retry 7; b untouched
        (f.the_switch_is_made_back())().unwrap(); // the way back, in fastboot
        (f.the_flags_are_back())().unwrap();
        let c = calls(&d);
        assert_eq!(
            c.lines().filter(|l| l.contains(" flash ")).count(),
            1,
            "{c}"
        );
        assert!(
            c.contains("set_active a") && c.contains("set_active b"),
            "{c}"
        );
    }

    #[test]
    fn the_same_walk_passes_from_the_first_run_state_too() {
        let first = FLAGS_AFTER_RUN_1
            .replace("retry-count:a:1", "retry-count:a:0")
            .replace("successful:a:no", "successful:a:yes");
        let (f, _d) = the_phone_after_rerun_3("first-walk", &first);
        std::fs::write(f.by_name_of("boot_a"), b"the stock boot_a").unwrap(); // stock, as before any flash
        (f.the_slot_is_read_by_adb())().unwrap();
        (f.the_slot_partitions_are_read_before())().unwrap();
        (f.the_guard_holds())().unwrap();
        (f.the_slot_flags_are_read("slots-before.txt"))().unwrap();
        // the flash resets slot a's flags as run 1 printed
        (f.the_boot_image_is_flashed())().unwrap();
        (f.the_partition_reads_back_as_the_image())().unwrap();
        (f.the_flash_changed_no_flag())().unwrap();
        (f.the_slot_is_made_active())().unwrap();
        (f.the_flags_are_the_switch_and_nothing_else())().unwrap();
        (f.the_switch_is_made_back())().unwrap();
        (f.the_flags_are_back())().unwrap();
    }

    /// THE AUDIT BY STATE (item walleye-stage2-check-18-unbootable-clears): the whole walk against EACH state the command can start from, not the one state a run last saw.
    /// (i) stock: a retry 0, successful yes; (ii) after one flash: retry 1, successful no (run 1); (iii) after a failed boot of a: unbootable yes, retry 0, successful no (run 5's printed before-state);
    /// (iv) after a successful stage 2 and its way back: a retry 7, successful no, unbootable no, b successful yes.
    #[test]
    fn the_whole_walk_passes_from_each_state_the_command_can_start_from() {
        let flags = |a: (&str, &str, &str)| {
            format!(
                "current-slot:b\nslot-retry-count:a:{}\nslot-retry-count:b:0\nslot-successful:a:{}\nslot-successful:b:yes\nslot-unbootable:a:{}\nslot-unbootable:b:no\n",
                a.0, a.1, a.2
            )
        };
        let states = [
            ("stock", flags(("0", "yes", "no")), false),
            ("after-one-flash", flags(("1", "no", "no")), true),
            ("after-a-failed-boot", flags(("0", "no", "yes")), true),
            ("after-stage-2-and-back", flags(("7", "no", "no")), true),
        ];
        for (name, state, holds_image) in states {
            let (f, d) = the_phone_after_rerun_3(&format!("by-state-{name}"), &state);
            if !holds_image {
                std::fs::write(f.by_name_of("boot_a"), b"the stock boot_a").unwrap();
            }
            let at =
                |step: &str, r: Done| r.unwrap_or_else(|e| panic!("state {name}, {step}: {e}"));
            at("8", (f.the_slot_is_read_by_adb())());
            at("9", (f.the_slot_partitions_are_read_before())());
            at("12", (f.the_guard_holds())());
            at("13", (f.the_slot_flags_are_read("slots-before.txt"))());
            at("14", (f.the_boot_image_is_flashed())());
            at("16", (f.the_partition_reads_back_as_the_image())());
            at("18", (f.the_flash_changed_no_flag())());
            at("19", (f.the_slot_is_made_active())());
            at("20", (f.the_flags_are_the_switch_and_nothing_else())());
            at("way back", (f.the_switch_is_made_back())());
            at("way back flags", (f.the_flags_are_back())());
            assert!(calls(&d).contains("set_active b"), "state {name}");
        }
    }

    /// STATE (v) of the audit by state (item walleye-stage2-check-8-earlier-images; rehearsal 5, research walleye-install-flash-boot-rehearsal-5-stops-at-check-8): boot_a holds an EARLIER image of ours, recorded in device.toml as an [images.*] row,
    /// neither the stock backup nor the declared image; slot a's flags are those after a failed boot (unbootable yes, retry 0, successful no: run 5's printed before-state) and those after a flash (retry 1, successful no). The whole walk of the command passes against both.
    #[test]
    fn the_whole_walk_passes_when_boot_a_holds_an_earlier_image_of_ours() {
        let flags = |retry: &str, unbootable: &str| {
            format!(
                "current-slot:b\nslot-retry-count:a:{retry}\nslot-retry-count:b:0\nslot-successful:a:no\nslot-successful:b:yes\nslot-unbootable:a:{unbootable}\nslot-unbootable:b:no\n"
            )
        };
        for (name, state) in [
            ("failed-boot", flags("0", "yes")),
            ("after-a-flash", flags("1", "no")),
        ] {
            let (f, d) = the_phone_after_rerun_3(&format!("by-state-v-{name}"), &state);
            let earlier = b"an earlier image of ours, not the declared one";
            std::fs::write(f.by_name_of("boot_a"), earlier).unwrap();
            let at =
                |step: &str, r: Done| r.unwrap_or_else(|e| panic!("state v {name}, {step}: {e}"));
            at("8", (f.the_slot_is_read_by_adb())());
            at("9", (f.the_slot_partitions_are_read_before())());
            at("12", (f.the_guard_holds())());
            at("13", (f.the_slot_flags_are_read("slots-before.txt"))());
            at("14", (f.the_boot_image_is_flashed())());
            at("16", (f.the_partition_reads_back_as_the_image())());
            at("18", (f.the_flash_changed_no_flag())());
            at("19", (f.the_slot_is_made_active())());
            at("20", (f.the_flags_are_the_switch_and_nothing_else())());
            at("way back", (f.the_switch_is_made_back())());
            at("way back flags", (f.the_flags_are_back())());
            assert!(calls(&d).contains("set_active b"), "state v {name}");
        }
    }

    // ---- THE STATE MODEL (item walleye-stage2-state-model-walk): every start state the command can meet, the whole offline walk over each, one table ----
    // Sources of the flag states: stock = what the bootloader had before any flash (research walleye-stage2-walk-by-state state i); after one flash = run 1's printed `getvar all` (FLAGS_AFTER_RUN_1);
    // failed boot = run 5's printed before-state (retry 0, successful no, unbootable yes); after a successful stage 2 = state iv of that research (retry 7). b not successful: slot b with `successful no`, retry 0 - not a state any run printed, the unsafe case.
    // boot_a contents: the stock backup (what the backup folder holds), an earlier image of ours (e337de53 in rehearsal 5), the declared image, any other bytes.

    /// One step of the walk: its name and what it runs.
    type WalkStep = (&'static str, Box<dyn FnOnce() -> Done>);

    #[derive(Clone, Copy, Debug)]
    enum BootA {
        Stock,
        Earlier,
        Declared,
        Unknown,
    }

    /// Slot a's flags (retry, successful, unbootable) of each start state, and slot b's successful bit.
    const A_STATES: [(&str, (&str, &str, &str)); 4] = [
        ("stock", ("0", "yes", "no")),
        ("after-one-flash", ("1", "no", "no")),
        ("after-a-failed-boot", ("0", "no", "yes")),
        ("after-a-successful-stage-2", ("7", "no", "no")),
    ];

    /// What the walk over one state found.
    struct Walked {
        /// The first step that stopped, and why (None: every step passed).
        stopped: Option<(&'static str, String)>,
        /// The steps after it, never reached.
        unreachable: Vec<&'static str>,
        /// What `new-slot-source.txt` says (step 21), when the walk got that far.
        source: Option<String>,
        /// The way back went through the reboot tool: it was pushed and asked for the bootloader.
        asked_the_reboot_tool: bool,
    }

    /// The image of the walk: a boot image whose header command line is the debug image's (no root=, no slot word), as step 21 requires.
    fn walk_image() -> Vec<u8> {
        let mut image = vec![0u8; 12288];
        image[..8].copy_from_slice(b"ANDROID!");
        image[64..64 + DEBUG_HEADER.len()].copy_from_slice(DEBUG_HEADER.as_bytes());
        image
    }

    /// The phone of the whole walk: the repeat's phone (partitions as files, a stateful fastboot) that also boots the new slot: its ssh prints run 9's own text (the new slot's read) and sda's GPT as Android printed it,
    /// takes the reboot tool and then lists the phone in fastboot; `fastboot reboot` takes it out of fastboot. Android's adb answers the same GPT. The GPT of the new slot is not captured (run 9 stopped at step 21's
    /// source check before reading it): the new slot answers with the GPT Android printed, so step 22 is tested on "nothing changed" only (NOT SOURCED YET). Typed, not captured: the empty `---dt-slot` section (printed after run 9's `---end`, not before it), luns = 1 (only sda is captured), the address, device_wait, the key and the reboot tool stand-in bytes.
    fn the_phone_that_boots_the_new_slot(
        name: &str,
        flags: &str,
    ) -> (FlashBoot, std::path::PathBuf) {
        let (f, d) = the_phone_after_rerun_3(name, flags);
        let mut s = (*f.s).clone();
        let image = walk_image();
        std::fs::write(&s.image, &image).unwrap();
        std::fs::write(f.by_name_of("boot_a"), &image).unwrap();
        s.image_sha256 = host::sha256_of(&s.image).unwrap();
        executable(&d.join("tad"), &go_row(&s.image_sha256, true));
        s.luns = 1;
        s.device_wait = 1;
        s.address = "172.16.43.1".into();
        let key = d.join("key");
        std::fs::write(&key, "k").unwrap();
        std::fs::set_permissions(&key, std::os::unix::fs::PermissionsExt::from_mode(0o600))
            .unwrap();
        s.identity = key;
        let tool = d.join("walleye-reboot");
        std::fs::write(&tool, b"\x7fELF stand-in for the static tool").unwrap();
        s.reboot_tool_sha256 = host::sha256_of(&tool).unwrap();
        s.reboot_tool = tool;
        std::fs::write(d.join("read.txt"), RUN9_READ).unwrap();
        std::fs::write(d.join("dt.txt"), "").unwrap();
        std::fs::write(d.join("gpt.txt"), RUN9_GPT_SDA).unwrap();
        executable(
            &d.join("ssh"),
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/ssh.calls\ncase \"$*\" in\n *'cat > /tmp/walleye-reboot'*) cat > {d}/pushed;;\n *'/tmp/walleye-reboot --command bootloader'*) touch {d}/in-fastboot;;\n *'---id'*) cat {d}/read.txt; printf -- '---dt-slot\\n%s\\n' \"$(cat {d}/dt.txt)\";;\n *'/sys/block/sda/size'*) cut -d' ' -f2 {d}/gpt.txt;;\nesac\nexit 0\n",
                d = d.display()
            ),
        );
        executable(
            &d.join("adb"),
            &format!(
                "#!/bin/sh\nshift 2\ncase \"$1\" in\n exec-out) shift; exec sh -c \"$*\";;\n shell) case \"$2\" in\n  'getprop ro.boot.slot_suffix') echo _b;;\n  *'/sys/block/sda/size'*) cut -d' ' -f2 {d}/gpt.txt;;\n esac;;\nesac\nexit 0\n",
                d = d.display()
            ),
        );
        // the bootloader's fastboot: listed only while the phone is in it, `reboot` leaves it; everything else is the repeat's stateful stand-in
        std::fs::rename(d.join("fastboot"), d.join("fastboot-state")).unwrap();
        executable(
            &d.join("fastboot"),
            &format!(
                "#!/bin/sh\nD={d}\ncase \"$1\" in devices) [ -e $D/in-fastboot ] && printf 'SER1\\tfastboot\\n'; exit 0;; esac\ncase \"$3\" in reboot) echo \"$@\" >> $D/fastboot.calls; rm -f $D/in-fastboot; exit 0;; esac\nexec $D/fastboot-state \"$@\"\n",
                d = d.display()
            ),
        );
        std::fs::write(d.join("in-fastboot"), "").unwrap();
        let f = FlashBoot::with_programs(
            s,
            &d.join("ssh").to_string_lossy(),
            &d.join("tad").to_string_lossy(),
        );
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        (f, d)
    }

    /// The offline walk over one state, from the read of the slot (step 8) to the flags after the way back (the item's steps 8 to 22 and the way back): the first step that stopped, and why.
    fn walk_state(name: &str, boot_a: BootA, a: (&str, &str, &str), b_successful: &str) -> Walked {
        let flags = format!(
            "current-slot:b\nslot-retry-count:a:{}\nslot-retry-count:b:0\nslot-successful:a:{}\nslot-successful:b:{b_successful}\nslot-unbootable:a:{}\nslot-unbootable:b:no\n",
            a.0, a.1, a.2
        );
        let (f, d) = the_phone_that_boots_the_new_slot(name, &flags);
        let content: &[u8] = match boot_a {
            BootA::Stock => b"the stock boot_a",
            BootA::Earlier => b"an earlier image of ours, not the declared one",
            BootA::Declared => b"",
            BootA::Unknown => b"bytes nobody recorded",
        };
        if !matches!(boot_a, BootA::Declared) {
            std::fs::write(f.by_name_of("boot_a"), content).unwrap();
        }
        let steps: Vec<WalkStep> = vec![
            (
                "8 adb reads the slot",
                Box::new(f.the_slot_is_read_by_adb()),
            ),
            (
                "9 boot_a is flashable",
                Box::new(f.the_slot_partitions_are_read_before()),
            ),
            ("10 the GPT is read", Box::new(f.the_gpt_is_read_before())),
            ("12 the guard", Box::new(f.the_guard_holds())),
            (
                "13 flags read",
                Box::new(f.the_slot_flags_are_read("slots-before.txt")),
            ),
            ("14 flash", Box::new(f.the_boot_image_is_flashed())),
            (
                "16 reads back as the image",
                Box::new(f.the_partition_reads_back_as_the_image()),
            ),
            (
                "18 the flash changed no flag",
                Box::new(f.the_flash_changed_no_flag()),
            ),
            ("19 set_active", Box::new(f.the_slot_is_made_active())),
            (
                "20 the flags are the switch",
                Box::new(f.the_flags_are_the_switch_and_nothing_else()),
            ),
            (
                "21 the new slot boots, its slot is read",
                Box::new(f.the_new_slot_boots_and_the_lab_logs_in()),
            ),
            (
                "22 the GPT changed only by the switch",
                Box::new(f.the_gpt_changed_only_by_the_switch("gpt-new-slot.txt")),
            ),
            ("way back", Box::new(f.the_switch_is_made_back())),
            ("way back flags", Box::new(f.the_flags_are_back())),
        ];
        let mut stopped = None;
        let mut unreachable = Vec::new();
        for (step, run) in steps {
            if stopped.is_some() {
                unreachable.push(step);
            } else if let Err(e) = run() {
                stopped = Some((step, e));
            }
        }
        Walked {
            stopped,
            unreachable,
            source: f.marked("new-slot-source.txt"),
            asked_the_reboot_tool: ssh_calls(&d).contains("--command bootloader")
                && d.join("pushed").exists(),
        }
    }

    #[test]
    fn the_state_model_walks_every_start_state_and_the_table_is_the_decision() {
        let mut table = String::new();
        let mut surprises = Vec::new();
        for boot_a in [
            BootA::Stock,
            BootA::Earlier,
            BootA::Declared,
            BootA::Unknown,
        ] {
            for (a_name, a) in A_STATES {
                for (b_name, b_successful) in [("b-successful", "yes"), ("b-not-successful", "no")]
                {
                    let name = format!("model-{boot_a:?}-{a_name}-{b_name}");
                    let got = walk_state(&name, boot_a, a, b_successful);
                    table.push_str(&format!(
                        "{boot_a:?} | {a_name} | {b_name} | {}\n",
                        match &got.stopped {
                            None => format!(
                                "passes, the new slot read from {}",
                                got.source.as_deref().unwrap_or("?")
                            ),
                            Some((step, why)) => format!(
                                "stops at {step}: {}; unreachable: {}",
                                why.lines().next().unwrap_or(""),
                                got.unreachable.join(", ")
                            ),
                        }
                    ));
                    // the decision (research walleye-stage2-checks-should-they-exist, rows 8, 9, 18, 20): what boot_a held and the flags of the flashed slot never stop the command; the one unsafe start state is Android's slot not counted good (no way back)
                    let must_stop = b_successful == "no";
                    let stops_at_13 = got
                        .stopped
                        .as_ref()
                        .is_some_and(|(step, _)| step.starts_with("13 "));
                    // a state that passes reaches step 21 and reads the new slot as run 9's own text answers: root=PARTUUID names slot a (research walleye-stage2-run9-booted-alpine-cmdline-still-cut-no-slot-suffix)
                    let new_slot_from_run_9s_text = got
                        .source
                        .as_deref()
                        .is_some_and(|s| s.starts_with("slot _a from root=PARTUUID"));
                    if got.stopped.is_some() != must_stop
                        || (must_stop && (!stops_at_13 || got.source.is_some()))
                        || (!must_stop
                            && (!new_slot_from_run_9s_text || !got.asked_the_reboot_tool))
                    {
                        surprises.push(format!(
                            "{boot_a:?} | {a_name} | {b_name}: {:?}, source {:?}",
                            got.stopped, got.source
                        ));
                    }
                }
            }
        }
        std::fs::write(std::env::temp_dir().join("walleye-state-model.txt"), &table).ok();
        assert!(
            surprises.is_empty(),
            "states that stop or pass against the decision:\n{}\n\nthe table:\n{table}",
            surprises.join("\n")
        );
    }

    // The device tree's slot_suffix of THIS phone is printed nowhere yet (NOT KNOWN YET): run 9's read predates the `---dt-slot` section. This test says so, so the walk's new slot is not taken for a read of all three
    // sources; when a read of the node is captured, it is added as a fixture, this test is replaced by one that walks it, and the item's last line closes.
    #[test]
    fn the_device_trees_slot_suffix_has_no_captured_sample_yet_and_the_walk_answers_without_it() {
        assert!(
            !RUN9_READ.contains("---dt-slot"),
            "run 9's read now carries the device tree's slot_suffix: walk it (a captured sample exists)"
        );
        let got = walk_state("dt-not-sourced", BootA::Declared, ("0", "yes", "no"), "yes");
        assert!(got.stopped.is_none(), "{:?}", got.stopped);
        assert!(!got.source.unwrap().contains("device-tree"));
    }

    // research walleye-stage2-checks-should-they-exist, row 9: content nobody recorded is overwritten, so a copy of it is kept first; the backup and the declared image need none
    #[test]
    fn unrecorded_boot_a_content_is_copied_before_the_flash_and_recorded_content_is_not() {
        let (f, d) = the_phone_after_rerun_3("keep-copy", FLAGS_AFTER_RUN_1);
        std::fs::write(f.by_name_of("boot_a"), b"bytes nobody recorded").unwrap();
        (f.the_slot_is_read_by_adb())().unwrap();
        (f.the_slot_partitions_are_read_before())().unwrap();
        let kept = std::fs::read(f.s.out.join("boot_a-before-the-flash.img")).unwrap();
        assert_eq!(kept, b"bytes nobody recorded");
        let (g, _e) = the_phone_after_rerun_3("keep-none", FLAGS_AFTER_RUN_1);
        std::fs::write(g.by_name_of("boot_a"), b"the stock boot_a").unwrap();
        (g.the_slot_is_read_by_adb())().unwrap();
        (g.the_slot_partitions_are_read_before())().unwrap();
        assert!(!g.s.out.join("boot_a-before-the-flash.img").exists());
        let _ = d;
    }

    #[test]
    fn leftovers_of_an_earlier_run_are_cleared_at_step_8_and_an_unfinished_switch_marker_is_a_note_not_a_stop(
    ) {
        let (f, _d) = the_phone_after_rerun_3("leftovers", FLAGS_AFTER_RUN_1);
        f.lab.save("flashed.txt", "boot_a old").unwrap();
        f.lab.save("parts-before.txt", "boot_a old").unwrap();
        (f.the_slot_is_read_by_adb())().unwrap();
        assert!(f.marked("flashed.txt").is_none() && f.marked("parts-before.txt").is_none());
        f.lab.save(SWITCHED_FILE, "a").unwrap();
        // research walleye-stage2-checks-should-they-exist, row 8: the marker is the host's memory, the phone is on Android (adb answered); the step goes on and clears it
        (f.the_slot_is_read_by_adb())().unwrap();
        assert!(f.marked(SWITCHED_FILE).is_none());
    }

    #[test]
    fn only_the_target_may_change_decides_on_lists_the_target_is_in() {
        let (img, other) = ("ab".repeat(32), "cd".repeat(32));
        let list = |a: &str, d: &str| format!("boot_a {a}\nboot_b {other}\ndtbo_a {d}");
        let held = list(&img, &other);
        assert!(only_the_target_may_change(&held, &held, "boot_a", &img).is_ok());
        assert!(only_the_target_may_change(&list(&other, &other), &held, "boot_a", &img).is_ok());
        assert!(only_the_target_may_change(&held, &list(&img, &img), "boot_a", &img).is_err());
        assert!(only_the_target_may_change(&held, &list(&other, &other), "boot_a", &img).is_err());
        assert!(only_the_target_may_change("", "", "boot_a", &img).is_err());
        assert!(only_the_target_may_change(&held, "boot_a ab", "boot_a", &img).is_err());
        assert!(only_the_target_may_change(&held, &held, "boot_x", &img).is_err());
    }

    #[test]
    fn the_gpt_checker_takes_the_slot_bits_of_a_repeat_as_it_takes_a_first_run() {
        // slot a's bits after run 1's flash differ from the stock ones; the checker holds no value of the bits, only where they may change
        let mut repeat = before();
        for base in [P_ENT, b_ent_of(128)] {
            repeat[base + 54] = 0x01;
        }
        fix_crcs(&mut repeat);
        assert!(gpt_switch_only(&repeat, &repeat).is_ok());
        assert!(gpt_switch_only(&repeat, &switched()).is_ok());
        assert!(gpt_switch_only(&switched(), &repeat).is_ok());
    }

    // ---- the pstore is read first when the new slot gives no ssh (item walleye-stage2-pstore-first-on-fallback) ----
    // The fixtures are text the phone printed: the empty /sys/fs/pstore of the read after run 4 (PR 851), an `ls -la` of a folder with files from the same read, and the
    // command line of Android's slot b.

    const EMPTY_PSTORE: &str = include_str!("../fixtures/android-pstore-empty-listing.txt");
    const DIR_WITH_FILES: &str = include_str!("../fixtures/android-dir-listing-with-files.txt");
    const ANDROID_B_CMDLINE: &str = include_str!("../fixtures/android-b-cmdline.txt");

    #[test]
    fn the_pstore_the_phone_printed_empty_has_no_file_to_read_and_a_folder_with_files_names_them() {
        assert!(pstore_files(EMPTY_PSTORE).is_empty());
        assert_eq!(pstore_files(DIR_WITH_FILES), ["priv.pem", "pub.pem"]);
        let odd = "-r-------- 1 root root 5 2026-11-23 13:23 console;reboot\n-r-------- 1 root root 5 2026-11-23 13:23 console-ramoops-0\n";
        assert_eq!(pstore_files(odd), ["console-ramoops-0"]);
    }

    fn phone_says<'a>(
        listing: &'static str,
        asked: &'a mut Vec<String>,
    ) -> impl FnMut(&str) -> Result<String, String> + 'a {
        move |c: &str| {
            asked.push(c.to_string());
            Ok(match c {
                "ls -la /sys/fs/pstore" => listing.to_string(),
                "cat /proc/cmdline" => ANDROID_B_CMDLINE.to_string(),
                "getprop ro.boot.slot_suffix" => "_b\n".into(),
                "getprop ro.boot.bootreason" => "reboot\n".into(),
                _ if c.starts_with("cat /sys/fs/pstore/") => format!("console of {c}\n"),
                _ => "x\n".into(),
            })
        }
    }

    #[test]
    fn the_failed_boot_read_starts_with_the_pstore_and_reads_each_file_whole() {
        let mut asked = Vec::new();
        let report = failed_boot_report(&mut phone_says(DIR_WITH_FILES, &mut asked));
        assert_eq!(asked[0], "ls -la /sys/fs/pstore");
        assert_eq!(asked[1], "cat /sys/fs/pstore/priv.pem");
        assert_eq!(asked[2], "cat /sys/fs/pstore/pub.pem");
        let parts = sections(&report);
        let names: Vec<&str> = parts.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            &names[..3],
            ["pstore-listing", "pstore:priv.pem", "pstore:pub.pem"],
            "{names:?}"
        );
        assert_eq!(names.last(), Some(&"end"));
        let cmdline = &parts.iter().find(|(k, _)| k == "cmdline").unwrap().1;
        assert!(
            cmdline.contains("androidboot.slot_suffix=_b")
                && cmdline.contains("androidboot.verifiedbootstate=orange")
        );
        assert!(!pstore_is_closed(&report));
    }

    #[test]
    fn an_empty_pstore_is_a_listing_and_no_file_read_and_the_rest_still_follows() {
        let mut asked = Vec::new();
        let report = failed_boot_report(&mut phone_says(EMPTY_PSTORE, &mut asked));
        assert!(
            asked.iter().all(|c| !c.starts_with("cat /sys/fs/pstore/")),
            "{asked:?}"
        );
        assert!(
            report.contains("---pstore-listing\ntotal 0\n")
                && report.contains("---bootreason\nreboot\n"),
            "{report}"
        );
    }

    #[test]
    fn a_command_the_phone_refuses_is_written_as_not_read_and_a_closed_pstore_is_seen() {
        let mut ask = |c: &str| -> Result<String, String> {
            if c.starts_with("ls ") {
                Ok("ls: /sys/fs/pstore: Permission denied\n".into())
            } else {
                Err("adb: device offline".into())
            }
        };
        let report = failed_boot_report(&mut ask);
        assert!(pstore_is_closed(&report));
        assert!(
            report.contains("---uptime\n(not read: adb: device offline)"),
            "{report}"
        );
    }

    #[test]
    fn the_failed_boot_read_only_reads() {
        let mut asked = Vec::new();
        failed_boot_report(&mut phone_says(DIR_WITH_FILES, &mut asked));
        for c in &asked {
            let program = c.split_whitespace().next().unwrap();
            assert!(
                ["ls", "cat", "getprop", "uptime", "id"].contains(&program),
                "{c}"
            );
            for word in [
                "reboot", "fastboot", "setprop", "svc", "stop", "start", "rm", "dd", "echo",
                "sysrq", ">", "|", ";", "&",
            ] {
                assert!(
                    !c.split_whitespace().any(|w| w.contains(word)),
                    "{c} has {word}"
                );
            }
        }
    }

    /// Stage 2 at step 21 against a stand-in ssh that never logs in, a fastboot that lists the phone only on request and an adb that answers only
    /// when `up` exists: nothing but reads is sent while it waits.
    fn bench_no_ssh(
        name: &str,
        adb_up: bool,
        fallback_wait: u64,
    ) -> (FlashBoot, std::path::PathBuf) {
        let (mut f, d) = bench_asked(name, None);
        let adb = d.join("adb");
        std::fs::write(d.join("pstore.txt"), DIR_WITH_FILES).unwrap();
        std::fs::write(d.join("cmdline.txt"), ANDROID_B_CMDLINE).unwrap();
        if adb_up {
            std::fs::write(d.join("up"), "").unwrap();
        }
        executable(
            &adb,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/adb.calls\ncase \"$3\" in get-state) if [ -e {d}/up ]; then echo device; else echo \"error: device 'SER1' not found\" >&2; exit 1; fi;; shell) case \"$4\" in 'ls -la /sys/fs/pstore') cat {d}/pstore.txt;; 'cat /proc/cmdline') cat {d}/cmdline.txt;; 'getprop ro.boot.slot_suffix') echo _b;; 'getprop ro.boot.bootreason') echo reboot;; 'cat /sys/fs/pstore/'*) echo \"console of $4\";; *) echo x;; esac;; esac\nexit 0\n",
                d = d.display()
            ),
        );
        let mut s = (*f.s).clone();
        s.adb = adb.to_string_lossy().into_owned();
        s.fallback_wait = fallback_wait;
        f.s = std::rc::Rc::new(s);
        f.lab = Lab::new(f.s.clone());
        // the host side (item walleye-stage2-debug-image): the switch was made at a time, and the host's journal has the phone's port lines among others
        f.lab.save(SWITCHED_AT_FILE, "1700000000").unwrap();
        executable(
            &d.join("journalctl"),
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/journalctl.calls\ncat {d}/journal.txt\n",
                d = d.display()
            ),
        );
        std::fs::write(d.join("journal.txt"), host_journal()).unwrap();
        f.journalctl = d.join("journalctl").to_string_lossy().into_owned();
        f.android = Android::new(f.s.clone(), f.lab.clone());
        (f, d)
    }

    fn adb_calls(d: &std::path::Path) -> String {
        std::fs::read_to_string(d.join("adb.calls")).unwrap_or_default()
    }

    #[test]
    fn no_ssh_then_android_answers_adb_and_the_pstore_is_read_first_with_nothing_sent_that_restarts_the_phone(
    ) {
        let (f, d) = bench_no_ssh("pstore-first", true, 5);
        let said = (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap_err();
        assert!(
            said.contains("no ssh login within 1 s")
                && said.contains("failed-boot-read.txt")
                && said.contains("read first"),
            "{said}"
        );
        let adb = adb_calls(&d);
        let first_shell = adb.lines().find(|l| l.contains(" shell ")).unwrap();
        assert!(
            first_shell.ends_with("shell ls -la /sys/fs/pstore"),
            "{adb}"
        );
        assert!(adb.contains("shell cat /sys/fs/pstore/priv.pem"), "{adb}");
        for banned in ["reboot", "root", "setprop"] {
            assert!(!adb.contains(banned), "{banned} in {adb}");
        }
        // the only fastboot calls are the one `reboot` that started the boot, and the listing of devices
        let fb = calls(&d);
        assert_eq!(
            fb.lines().filter(|l| l.contains("reboot")).count(),
            1,
            "{fb}"
        );
        assert!(!fb.contains("set_active") && !fb.contains("flash"), "{fb}");
        // the one ssh that could restart the phone (the way back's tool) is not made by this step
        assert!(
            !ssh_calls(&d).contains("walleye-reboot"),
            "{}",
            ssh_calls(&d)
        );
        let kept = std::fs::read_to_string(f.s.out.join(FAILED_BOOT_FILE)).unwrap();
        assert!(
            kept.starts_with("---pstore-listing")
                && kept.contains("androidboot.serialno=REDACTED")
                && !kept.contains("FA81N1A00790"),
            "{kept}"
        );
    }

    #[test]
    fn no_ssh_and_no_adb_within_the_bound_reads_nothing_and_sends_nothing() {
        let (f, d) = bench_no_ssh("no-adb", false, 1);
        let said = (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap_err();
        assert!(
            said.contains("adb did not answer within 1 s") && said.contains("nothing was read"),
            "{said}"
        );
        assert!(!adb_calls(&d).contains(" shell "), "{}", adb_calls(&d));
        assert!(!std::fs::read_to_string(f.s.out.join(FAILED_BOOT_FILE))
            .unwrap()
            .contains("pstore"));
        assert!(!ssh_calls(&d).contains("walleye-reboot"));
    }

    #[test]
    fn a_phone_the_bootloader_lists_is_not_waited_for_as_android() {
        let (f, d) = bench_no_ssh("in-bootloader", false, 30);
        std::fs::write(d.join("in-fastboot"), "").unwrap();
        let said = (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap_err();
        assert!(said.contains("in the bootloader, not on Android"), "{said}");
    }

    #[test]
    fn the_rehearsals_step_reads_the_same_on_android_as_it_is_and_checks_the_slot() {
        let (f, d) = bench_no_ssh("rehearsal-read", true, 5);
        (f.the_failed_boot_read_works_on_android())().unwrap();
        assert!(f.s.out.join("rehearsal-failed-boot-read.txt").exists());
        assert!(!adb_calls(&d).contains("reboot"));
        // Android on the other slot than adb says: the step fails and says both
        f.lab.save(ANDROID_SLOT_FILE, "a").unwrap();
        let said = (f.the_failed_boot_read_works_on_android())().unwrap_err();
        assert!(said.contains("ro.boot.slot_suffix"), "{said}");
    }

    #[test]
    fn a_file_the_shell_user_may_not_read_closes_the_pstore_as_a_listing_does() {
        let report = "---pstore-listing\n-r--r----- 1 system log 9 x dmesg-ramoops-0\n---pstore:dmesg-ramoops-0\ncat: /sys/fs/pstore/dmesg-ramoops-0: Permission denied\n---end\n";
        assert!(pstore_is_closed(report));
    }

    #[test]
    fn a_closed_pstore_is_kept_as_read_then_adb_root_is_asked_and_the_read_is_taken_again_as_root()
    {
        let (f, d) = bench_no_ssh("closed", true, 5);
        executable(
            &d.join("adb"),
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/adb.calls\ncase \"$3\" in get-state) echo device;; root) touch {d}/rooted;; shell) case \"$4\" in 'ls -la /sys/fs/pstore') if [ -e {d}/rooted ]; then cat {d}/pstore.txt; else echo 'ls: /sys/fs/pstore: Permission denied'; fi;; 'id -u') if [ -e {d}/rooted ]; then echo 0; else echo 2000; fi;; 'cat /sys/fs/pstore/'*) echo \"console of $4\";; *) echo x;; esac;; esac\nexit 0\n",
                d = d.display()
            ),
        );
        let text = f.read_the_failed_boot("closed.txt").unwrap();
        let adb = adb_calls(&d);
        let (listing, root, again) = (
            adb.find("ls -la").unwrap(),
            adb.find(" root").unwrap(),
            adb.rfind("ls -la").unwrap(),
        );
        assert!(listing < root && root < again, "{adb}");
        assert!(text.contains("---pstore:priv.pem"), "{text}");
        let kept = std::fs::read_to_string(f.s.out.join("closed.txt")).unwrap();
        assert!(kept.contains("Permission denied") && kept.contains("---pstore:priv.pem"));
    }

    // ---- the host's USB log of the wait (item walleye-stage2-debug-image) ----

    /// What the lab host's `journalctl -k` printed in a real run (research walleye-first-fastboot-boot, 2026-11-18, c5: 21 lines of 3-2, the phone's serial among them) and one line that is not the phone's.
    fn host_journal() -> String {
        format!(
            "Nov 18 16:03:59 grig kernel: nvme nvme0: the line of another device\n{}",
            include_str!(
                "../../../research/walleye-first-fastboot-boot/2026-11-18/c5-host-journal.txt"
            )
        )
    }

    #[test]
    fn the_hosts_log_keeps_the_lines_of_the_phones_port_and_redacts_its_serial() {
        let (text, n) = usb_log_section(&host_journal(), 1_700_000_000, "usb 3-2");
        assert_eq!(n, 21, "{text}");
        assert!(
            text.starts_with(
                "---host-usb-log (journalctl -k --since @1700000000, lines with 'usb 3-2')\n"
            ),
            "{text}"
        );
        assert!(
            !text.contains("nvme") && !text.contains("FA81N1A00790"),
            "{text}"
        );
        assert!(
            text.contains("usb 3-2: SerialNumber: REDACTED")
                && text.contains("idVendor=18d1, idProduct=4ee0"),
            "{text}"
        );
    }

    #[test]
    fn a_host_that_saw_no_usb_event_of_the_phone_says_so_in_words() {
        let (text, n) =
            usb_log_section("Nov 23 15:12:40 grig kernel: nvme nvme0: x\n", 5, "usb 3-2");
        assert_eq!(n, 0);
        assert!(
            text.contains("no line: the host saw no USB event"),
            "{text}"
        );
    }

    #[test]
    fn no_ssh_keeps_the_hosts_usb_log_in_its_own_file_and_in_the_failed_boot_file_after_the_pstore()
    {
        let (f, d) = bench_no_ssh("host-log", true, 5);
        let said = (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap_err();
        assert!(
            said.contains("21 lines of 3-2") && said.contains("host-usb-log.txt"),
            "{said}"
        );
        let own = std::fs::read_to_string(f.s.out.join(HOST_USB_LOG_FILE)).unwrap();
        assert!(
            own.starts_with("---host-usb-log") && !own.contains("FA81N1A00790"),
            "{own}"
        );
        let kept = std::fs::read_to_string(f.s.out.join(FAILED_BOOT_FILE)).unwrap();
        assert!(
            kept.starts_with("---pstore-listing") && kept.contains("\n---host-usb-log"),
            "{kept}"
        );
        assert!(kept.find("---pstore-listing").unwrap() < kept.find("---host-usb-log").unwrap());
        // the journal is read from the switch on, kernel messages only, and nothing is sent to the phone for it
        let j = std::fs::read_to_string(d.join("journalctl.calls")).unwrap();
        assert_eq!(j.trim(), "-k --no-pager --since @1700000000", "{j}");
    }

    #[test]
    fn no_ssh_and_no_adb_still_keeps_the_hosts_usb_log_and_reads_nothing_of_the_phone() {
        let (f, _d) = bench_no_ssh("host-log-no-adb", false, 1);
        let said = (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap_err();
        assert!(
            said.contains("nothing was read") && said.contains("host-usb-log.txt"),
            "{said}"
        );
        let kept = std::fs::read_to_string(f.s.out.join(FAILED_BOOT_FILE)).unwrap();
        assert!(
            kept.starts_with("---host-usb-log") && !kept.contains("pstore"),
            "{kept}"
        );
    }

    #[test]
    fn a_journal_that_cannot_be_read_is_said_and_does_not_hide_the_failed_boot_read() {
        let (mut f, _d) = bench_no_ssh("host-log-broken", true, 5);
        f.journalctl = "/nonexistent/journalctl".into();
        let said = (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap_err();
        assert!(
            said.contains("the host's USB log of the wait was not kept")
                && said.contains("read first"),
            "{said}"
        );
        assert!(std::fs::read_to_string(f.s.out.join(FAILED_BOOT_FILE))
            .unwrap()
            .starts_with("---pstore-listing"));
    }

    #[test]
    fn the_switch_writes_the_time_the_host_log_starts_from() {
        let (f, d) = bench("switch-time", "b");
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        (f.the_boot_image_is_flashed())().unwrap();
        std::fs::write(d.join("out").join("parts-readback.txt"), "read back\n").unwrap();
        (f.the_slot_is_made_active())().unwrap();
        let at: i64 = std::fs::read_to_string(d.join("out").join(SWITCHED_AT_FILE))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert!((at - now_epoch().unwrap()).abs() < 60);
    }

    // ---- step 21: WHICH SLOT booted, from what the bootloader supplied (item walleye-stage2-step21-slot-source) ----
    // The stand-in phone prints run 9's own text (research walleye-stage2-run9-booted-alpine-cmdline-still-cut-no-slot-suffix, material
    // ~/lab-material/walleye/install-boot-20261123-192731): new-slot-read.txt, sda's GPT read from Android (gpt-before.txt) and slots-after-switch.txt.
    // The device-tree line is the one thing not captured: this phone's /proc/device-tree/firmware/android/slot_suffix has not been printed yet (NOT KNOWN YET), so the tests
    // that need it give the value AOSP's fs_mgr reads (`_a`), as a unit test of the rule only.
    const RUN9_READ: &str = include_str!("../fixtures/stage2-run9-new-slot-read.txt");
    const RUN9_GPT_SDA: &str = include_str!("../fixtures/stage2-run9-android-gpt-sda.txt");
    const RUN9_FLAGS: &str = include_str!("../fixtures/stage2-run9-slots-after-switch.txt");

    /// A stage 2 bench whose ssh prints `read`, the device-tree line and sda's GPT, with an image whose header command line is `header`, and the flags the switch left.
    fn bench_run9(
        name: &str,
        read: &str,
        dt_slot: &str,
        header: &str,
        flags: &str,
    ) -> (FlashBoot, std::path::PathBuf) {
        let (mut f, d) = bench_asked(name, None);
        std::fs::write(d.join("read.txt"), read).unwrap();
        std::fs::write(d.join("dt.txt"), dt_slot).unwrap();
        std::fs::write(d.join("gpt.txt"), RUN9_GPT_SDA).unwrap();
        executable(
            &d.join("ssh"),
            &format!(
                "#!/bin/sh\necho \"$@\" >> {d}/ssh.calls\ncase \"$*\" in\n *'---id'*) cat {d}/read.txt; printf -- '---dt-slot\\n%s\\n' \"$(cat {d}/dt.txt)\";;\n *'/sys/block/sda/size'*) cut -d' ' -f2 {d}/gpt.txt;;\nesac\nexit 0\n",
                d = d.display()
            ),
        );
        let mut image = vec![0u8; 12288];
        image[..8].copy_from_slice(b"ANDROID!");
        image[64..64 + header.len()].copy_from_slice(header.as_bytes());
        let mut s = (*f.s).clone();
        s.luns = 1;
        s.image = d.join("NOT-TO-BOOT-boot.img");
        std::fs::write(&s.image, image).unwrap();
        f.s = std::rc::Rc::new(s);
        f.lab = Lab::new(f.s.clone());
        f.android = Android::new(f.s.clone(), f.lab.clone());
        f.lab.save(ANDROID_SLOT_FILE, "b").unwrap();
        f.lab.save("slots-after-switch.txt", flags).unwrap();
        (f, d)
    }

    /// The words of the header command line the stage 2 debug image carries (no root=).
    const DEBUG_HEADER: &str = "clk_ignore_unused ignore_loglevel panic=5 androidboot.hardware=walleye androidboot.dtbo_idx=3 walleye.root=system_a walleye.booted-slot walleye.debug";

    #[test]
    fn run_9s_boot_without_the_slot_word_is_accepted_by_the_slot_the_bootloaders_root_names() {
        let (f, _d) = bench_run9("run9-step21", RUN9_READ, "", DEBUG_HEADER, RUN9_FLAGS);
        (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap();
        let source = std::fs::read_to_string(f.s.out.join("new-slot-source.txt")).unwrap();
        assert!(
            source.contains("root=PARTUUID") && source.contains("_a"),
            "{source}"
        );
        assert!(f.s.out.join("gpt-new-slot.txt").exists());
    }

    fn run9_cmdline() -> String {
        sections(RUN9_READ)
            .into_iter()
            .find(|(k, _)| k == "cmdline")
            .unwrap()
            .1
    }

    fn run9_sda() -> Vec<u8> {
        dump_bytes(RUN9_GPT_SDA.split_once(' ').unwrap().1).unwrap()
    }

    /// A command line with nothing of the three sources on it.
    const NO_SOURCE: &str =
        "clk_ignore_unused ignore_loglevel panic=5 androidboot.hardware=walleye";

    #[test]
    fn each_source_of_the_slot_answers_alone_and_the_answer_says_which() {
        let sda = run9_sda();
        // root=PARTUUID alone: run 9's own line (partition 0x16 = 22, system_a in sda's GPT)
        let got = booted_slot(&run9_cmdline(), "", &sda).unwrap();
        assert_eq!(got.slot, "a");
        assert_eq!(got.answers, vec![("root=PARTUUID", "a".to_string())]);
        // the word alone: Android's own line (fixtures/android-b-cmdline.txt carries androidboot.slot_suffix=_b; it also carries root=PARTUUID=0000002b = system_b, so BOTH sources answer and agree)
        let both = booted_slot(ANDROID_B_CMDLINE, "", &sda).unwrap();
        assert_eq!(both.answers.len(), 2, "{both:?}");
        let word_only: Vec<&str> = ANDROID_B_CMDLINE
            .split_whitespace()
            .filter(|w| !w.starts_with("root="))
            .collect();
        let got = booted_slot(&word_only.join(" "), "", &sda).unwrap();
        assert_eq!(got.answers.len(), 1);
        assert_eq!(
            (got.slot.as_str(), got.answers[0].0),
            ("b", "androidboot.slot_suffix")
        );
        // the device tree alone (the value AOSP's fs_mgr reads, NUL-ended as the node is; not printed by this phone yet)
        let got = booted_slot(NO_SOURCE, "_b\0\n", &sda).unwrap();
        assert_eq!(got.answers, vec![("device-tree", "b".to_string())]);
    }

    #[test]
    fn sources_that_agree_pass_and_the_first_in_the_order_leads() {
        let line = format!("{} androidboot.slot_suffix=_a", run9_cmdline().trim());
        let got = booted_slot(&line, "_a", &run9_sda()).unwrap();
        assert_eq!(got.slot, "a");
        assert_eq!(got.answers.len(), 3);
        assert_eq!(got.answers[0].0, "androidboot.slot_suffix");
    }

    #[test]
    fn two_sources_that_disagree_and_no_source_at_all_refuse() {
        let sda = run9_sda();
        let word_b = format!("{} androidboot.slot_suffix=_b", run9_cmdline().trim());
        let said = booted_slot(&word_b, "", &sda).unwrap_err();
        assert!(
            said.contains("disagree") && said.contains("root=PARTUUID says a"),
            "{said}"
        );
        let said = booted_slot(&run9_cmdline(), "_b", &sda).unwrap_err();
        assert!(
            said.contains("disagree") && said.contains("device-tree says b"),
            "{said}"
        );
        let said = booted_slot(NO_SOURCE, "", &sda).unwrap_err();
        assert!(said.contains("no source gives the booted slot"), "{said}");
        let said = booted_slot(
            "androidboot.slot_suffix=_a androidboot.slot_suffix=_b",
            "",
            &sda,
        )
        .unwrap_err();
        assert!(said.contains("more than one slot"), "{said}");
        assert!(booted_slot("androidboot.slot_suffix=a", "", &sda).is_err());
    }

    #[test]
    fn a_partuuid_that_names_no_system_partition_or_no_entry_is_refused() {
        let sda = run9_sda();
        // entry 1 of sda is not a system partition; the 0xFFFF entry does not exist
        let said = booted_slot(
            "root=PARTUUID=00000001-0000-0000-0000-000000000000",
            "",
            &sda,
        )
        .unwrap_err();
        assert!(said.contains("not system_a or system_b"), "{said}");
        let said = booted_slot(
            "root=PARTUUID=0000ffff-0000-0000-0000-000000000000",
            "",
            &sda,
        )
        .unwrap_err();
        assert!(said.contains("entries"), "{said}");
        assert!(booted_slot("root=PARTUUID=zz", "", &sda).is_err());
        for odd in [
            "root=PARTUUID=16",
            "root=PARTUUID=00000016",
            "root=PARTUUID=00000016-01",
            "root=PARTUUID=a1b2c3d4-0000-0000-0000-000000000000x",
        ] {
            assert!(
                booted_slot(odd, "", &sda)
                    .unwrap_err()
                    .contains("bootloader's form"),
                "{odd}"
            );
        }
        // a root= word that is not a PARTUUID gives no slot
        assert!(booted_slot("root=/dev/sda22", "", &sda)
            .unwrap_err()
            .contains("no source"));
    }

    #[test]
    fn an_image_whose_own_header_carries_root_is_refused() {
        let image = |header: &str| {
            let mut b = vec![0u8; 12288];
            b[..8].copy_from_slice(b"ANDROID!");
            b[64..64 + header.len()].copy_from_slice(header.as_bytes());
            b
        };
        header_has_no_root(&image(DEBUG_HEADER)).unwrap();
        let said = header_has_no_root(&image(
            "a root=PARTUUID=00000016-0000-0000-0000-000000000000 b",
        ))
        .unwrap_err();
        assert!(said.contains("carries root=PARTUUID"), "{said}");
        assert!(header_has_no_root(b"not a boot image").is_err());
        let said = header_has_no_root(&image("a androidboot.slot_suffix=_a")).unwrap_err();
        assert!(said.contains("androidboot.slot_suffix=_a"), "{said}");
        // and the step refuses with it, before it reads a slot
        let (f, _d) = bench_run9(
            "header-root",
            RUN9_READ,
            "",
            "a root=/dev/sda22",
            RUN9_FLAGS,
        );
        assert!((f.the_new_slot_boots_and_the_lab_logs_in())()
            .unwrap_err()
            .contains("carries root=/dev/sda22"));
    }

    #[test]
    fn the_step_refuses_a_slot_that_is_not_the_one_fastboot_says_is_current() {
        let flags_b = RUN9_FLAGS.replace("current-slot:a", "current-slot:b");
        let (f, _d) = bench_run9("flags-disagree", RUN9_READ, "", DEBUG_HEADER, &flags_b);
        let said = (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap_err();
        assert!(
            said.contains("another slot") && said.contains("current-slot says \"b\""),
            "{said}"
        );
        // the sources disagree among themselves
        let (f, _d) = bench_run9(
            "sources-disagree",
            RUN9_READ,
            "_b",
            DEBUG_HEADER,
            RUN9_FLAGS,
        );
        assert!((f.the_new_slot_boots_and_the_lab_logs_in())()
            .unwrap_err()
            .contains("disagree"));
        // none of them answers: the root names no slot and nothing else is there
        let no_root = RUN9_READ.replace("root=PARTUUID=00000016-0000-0000-0000-000000000000", "");
        let (f, _d) = bench_run9("no-source", &no_root, "", DEBUG_HEADER, RUN9_FLAGS);
        assert!((f.the_new_slot_boots_and_the_lab_logs_in())()
            .unwrap_err()
            .contains("no source gives the booted slot"));
    }

    #[test]
    fn the_device_tree_alone_passes_the_step_and_is_the_source_logged() {
        let no_root = RUN9_READ.replace("root=PARTUUID=00000016-0000-0000-0000-000000000000", "");
        let (f, _d) = bench_run9("dt-only", &no_root, "_a\n", DEBUG_HEADER, RUN9_FLAGS);
        (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap();
        let source = std::fs::read_to_string(f.s.out.join("new-slot-source.txt")).unwrap();
        assert!(source.starts_with("slot _a from device-tree"), "{source}");
    }

    #[test]
    fn the_read_asks_the_device_tree_node_and_writes_nothing() {
        let (f, d) = bench_run9("read-only", RUN9_READ, "", DEBUG_HEADER, RUN9_FLAGS);
        (f.the_new_slot_boots_and_the_lab_logs_in())().unwrap();
        let ssh = ssh_calls(&d);
        assert!(
            ssh.contains("/proc/device-tree/firmware/android/slot_suffix"),
            "{ssh}"
        );
        for banned in [">>", "rm ", "mount ", "umount", "reboot", "dd of="] {
            assert!(!ssh.contains(banned), "{banned} in {ssh}");
        }
    }
}

#[cfg(test)]
mod flash_flag_tests {
    use super::*;

    /// Stage 2 run 1's own printed flags (research/walleye-display-s6e3fa5/2026-11-23/stage2-run.log, check 18): before the flash and after it.
    const RUN1_BEFORE: &str = "current-slot:b\nslot-retry-count:a:0\nslot-retry-count:b:0\nslot-successful:a:yes\nslot-successful:b:yes\nslot-unbootable:a:no\nslot-unbootable:b:no";
    const RUN1_AFTER: &str = "current-slot:b\nslot-retry-count:a:1\nslot-retry-count:b:0\nslot-successful:a:no\nslot-successful:b:yes\nslot-unbootable:a:no\nslot-unbootable:b:no";

    fn with(text: &str, from: &str, to: &str) -> String {
        assert!(text.contains(from), "{from}");
        text.replace(from, to)
    }

    #[test]
    fn what_run_1_printed_is_the_flash_resetting_slot_a() {
        flash_leaves_androids_slot_alone(RUN1_BEFORE, RUN1_AFTER, "a").unwrap();
    }

    // research walleye-stage2-checks-should-they-exist, row 18: the flashed slot's own flags are the bootloader's, asserted nowhere
    #[test]
    fn the_flashed_slots_own_retry_count_is_not_asserted() {
        for n in ["0", "1", "3", "7", "x"] {
            let after = with(
                RUN1_AFTER,
                "slot-retry-count:a:1",
                &format!("slot-retry-count:a:{n}"),
            );
            flash_leaves_androids_slot_alone(RUN1_BEFORE, &after, "a").unwrap();
        }
    }

    #[test]
    fn nothing_else_may_change() {
        let bad = |from: &str, to: &str| {
            flash_leaves_androids_slot_alone(RUN1_BEFORE, &with(RUN1_AFTER, from, to), "a")
                .unwrap_err()
        };
        assert!(bad("slot-retry-count:b:0", "slot-retry-count:b:1").contains("slot-retry-count:b")); // b's count
        assert!(bad("slot-successful:b:yes", "slot-successful:b:no").contains("slot-successful:b")); // b's flags
        assert!(bad("slot-unbootable:b:no", "slot-unbootable:b:yes").contains("slot-unbootable:b"));
        assert!(bad("current-slot:b", "current-slot:a").contains("current-slot"));
        // the current slot moved
    }

    #[test]
    fn the_flashed_slot_may_end_successful_or_unbootable_whatever_it_was() {
        for (from, to) in [
            ("slot-successful:a:no", "slot-successful:a:yes"),
            ("slot-unbootable:a:no", "slot-unbootable:a:yes"),
        ] {
            flash_leaves_androids_slot_alone(RUN1_BEFORE, &with(RUN1_AFTER, from, to), "a")
                .unwrap();
        }
    }

    #[test]
    fn a_flag_that_appeared_or_went_is_an_error_and_the_other_slot_is_not_allowed_to_reset() {
        let gone = RUN1_AFTER.replace("slot-unbootable:b:no", "");
        assert!(
            flash_leaves_androids_slot_alone(RUN1_BEFORE, gone.trim(), "a")
                .unwrap_err()
                .contains("names changed")
        );
        // the same text is NOT allowed for slot b: b's flags would be the ones that changed nothing, a's would have changed
        assert!(flash_leaves_androids_slot_alone(RUN1_BEFORE, RUN1_AFTER, "b").is_err());
    }

    /// Stage 2 run 5's own printed flags (research/walleye-display-s6e3fa5/2026-11-23/stage2-run5.log.out, check 18): a after a failed boot (unbootable yes, retry 0), then after a no-op flash.
    const RUN5_BEFORE: &str = "current-slot:b\nslot-retry-count:a:0\nslot-retry-count:b:0\nslot-successful:a:no\nslot-successful:b:yes\nslot-unbootable:a:yes\nslot-unbootable:b:no";
    const RUN5_AFTER: &str = "current-slot:b\nslot-retry-count:a:1\nslot-retry-count:b:0\nslot-successful:a:no\nslot-successful:b:yes\nslot-unbootable:a:no\nslot-unbootable:b:no";

    #[test]
    fn what_run_5_printed_is_the_flash_clearing_unbootable_too() {
        flash_leaves_androids_slot_alone(RUN5_BEFORE, RUN5_AFTER, "a").unwrap();
    }

    #[test]
    fn unbootable_of_the_flashed_slot_may_go_either_way_and_slot_bs_stays() {
        let stays = with(RUN5_AFTER, "slot-unbootable:a:no", "slot-unbootable:a:yes");
        flash_leaves_androids_slot_alone(RUN5_BEFORE, &stays, "a").unwrap();
        // slot b's unbootable clears: not allowed (Android's slot is the way back)
        let b_before = with(RUN5_BEFORE, "slot-unbootable:b:no", "slot-unbootable:b:yes");
        assert!(flash_leaves_androids_slot_alone(&b_before, RUN5_AFTER, "a")
            .unwrap_err()
            .starts_with("slot-unbootable:b changed from yes to no"));
    }

    // check 9 on the printed text of rehearsal 5 (research walleye-install-flash-boot-rehearsal-5-stops-at-check-8): boot_a read now e337de53, the stored backup 2a5cd3b2, the declared ec741a98
    const R5_BACKUP: &str = "2a5cd3b20ef0da2b272d2dac460f868d4764d8bdb8cb426d62eee568f1344fb5";
    const R5_NOW: &str = "e337de53446aa98e4795e214648427e22b467b3ab39f2d208e201951ff4098f1";
    const R5_DECLARED: &str = "ec741a98476c6b59d1e3875d9604ad225ca1cc7c2b9d9a81f320bc5ea556ed5d";

    fn check_9(now: &str, backup_file: Result<String, String>) -> Result<String, String> {
        boot_partition_can_be_flashed("boot_a", now, R5_BACKUP, R5_DECLARED, backup_file)
    }

    #[test]
    fn what_boot_a_holds_is_recorded_and_never_refused_while_the_backup_is_on_file() {
        // rehearsal 5's own state: an earlier image of ours, neither the backup nor the declared image
        let said = check_9(R5_NOW, Ok(R5_BACKUP.into())).unwrap();
        assert!(said.contains("content not recorded here"), "{said}");
        assert!(said.contains("can be undone"), "{said}");
        assert!(check_9(R5_BACKUP, Ok(R5_BACKUP.into()))
            .unwrap()
            .contains("its backup"));
        assert!(check_9(R5_DECLARED, Ok(R5_BACKUP.into()))
            .unwrap()
            .contains("the declared image"));
        let other = "ab".repeat(32);
        assert!(check_9(&other, Ok(R5_BACKUP.into())).is_ok());
    }

    #[test]
    fn without_the_backup_on_file_and_equal_to_its_record_nothing_is_flashable_whatever_boot_a_holds(
    ) {
        for now in [R5_BACKUP, R5_DECLARED, R5_NOW] {
            let said = check_9(now, Err("boot_a.img: No such file".into())).unwrap_err();
            assert!(said.contains("not on file"), "{said}");
            let said = check_9(now, Ok("cd".repeat(32))).unwrap_err();
            assert!(said.contains("hashes to"), "{said}");
        }
    }

    // the way back must be good before the flash: run 1's printed flags (b successful yes, unbootable no), and the two ways b is not
    const FLAGS_AFTER_RUN_1: &str = "current-slot:b\nslot-retry-count:a:1\nslot-retry-count:b:0\nslot-successful:a:no\nslot-successful:b:yes\nslot-unbootable:a:no\nslot-unbootable:b:no\n";
    #[test]
    fn android_slot_must_be_successful_and_bootable_before_anything_is_written() {
        assert!(android_slot_is_a_safe_way_back(FLAGS_AFTER_RUN_1, "b").is_ok());
        let not_successful = FLAGS_AFTER_RUN_1.replace("successful:b:yes", "successful:b:no");
        assert!(android_slot_is_a_safe_way_back(&not_successful, "b")
            .unwrap_err()
            .contains("slot-successful:b"));
        let unbootable = FLAGS_AFTER_RUN_1.replace("unbootable:b:no", "unbootable:b:yes");
        assert!(android_slot_is_a_safe_way_back(&unbootable, "b")
            .unwrap_err()
            .contains("slot-unbootable:b"));
        assert!(android_slot_is_a_safe_way_back("current-slot:b", "b").is_err());
    }
}
