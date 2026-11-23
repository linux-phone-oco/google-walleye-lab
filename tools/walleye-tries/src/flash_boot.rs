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

/// After the flash of `boot_X` (check 18; research walleye-stage2-flash-resets-flags-documented, AOSP's A/B spec): a write to a slot's partition makes
/// the bootloader clear that slot's `successful` and reset its retry count. So EXACTLY slot X's `slot-successful` ends `no` and its
/// `slot-retry-count` ends a positive number (observed 1, AOSP says usually 3: the number is not fixed here); `slot-unbootable:X`, the other slot's
/// every flag, the current slot and every name are what they were, and no flag appeared or went.
pub fn flash_resets_only_the_flashed_slots_flags(
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
    let (successful, retry) = (
        format!("slot-successful:{flashed}"),
        format!("slot-retry-count:{flashed}"),
    );
    for (k, v) in &b {
        if *k != successful && *k != retry && a[k] != *v {
            return Err(format!("{k} changed from {v} to {}: the flash changes only slot {flashed}'s successful and retry count\n{before}\nthen\n{after}", a[k]));
        }
    }
    match a.get(&successful).map(String::as_str) {
        Some("no") => {}
        other => {
            return Err(format!(
                "after the flash {successful} is {other:?}, the bootloader's reset leaves \"no\"\n{before}\nthen\n{after}"
            ))
        }
    }
    match a.get(&retry).and_then(|v| v.parse::<u8>().ok()) {
        Some(n) if n >= 1 => Ok(()),
        other => Err(format!(
            "after the flash {retry} is {other:?}, the bootloader's reset leaves a positive count\n{before}\nthen\n{after}"
        )),
    }
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

/// Check 9's decision: the partition read now is the stock backup (`want`, from the backup's sha256sums), or it already holds the declared image (a flash that read back equal)
/// while the backup file is on file and still hashes to `want`, so the flash can still be undone. Anything else fails, and says which way.
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
    if now == want {
        return Ok(format!("{target} is its backup ({want})"));
    }
    if now != declared {
        return Err(format!("{target} read now is {now}, neither the stored backup ({want}) nor the declared image ({declared}): the flash could not be undone"));
    }
    match backup_file {
        Err(e) => Err(format!("{target} holds the declared image, but the stock backup {target}.img is not on file or unreadable: {e}: the flash could not be undone")),
        Ok(on_file) if on_file.to_lowercase() != want => Err(format!("{target} holds the declared image, but the backup file {target}.img hashes to {on_file}, not the recorded {want}: the flash could not be undone")),
        Ok(_) => Ok(format!("{target} already holds the declared image ({declared}); the stock backup is on file and hashes to its record ({want}): the flash can be undone")),
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

    /// The boot partition about to be overwritten, read now from Android, is what the stored backup recorded, or the declared image with the backup on file and still hashing to its record: the flash can be undone. The partition not flashed, its dtbo and vbmeta, and the
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
            let said = only_the_target_may_change(&a, &b, &target, &u.s.image_sha256)?;
            println!("  {target} reads back as the image; {said}");
            Ok(())
        }
    }

    /// The flash changed only what the bootloader resets on a write: the flashed slot's successful (to no) and retry count (to a positive number).
    pub fn the_flash_changed_no_flag(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.flags("slots-after-flash.txt")?;
            let (a, b) = (
                u.saved("slots-before.txt")?,
                u.saved("slots-after-flash.txt")?,
            );
            let target = u.target()?; // boot_a or boot_b: the slot is its last letter
            flash_resets_only_the_flashed_slots_flags(&a, &b, &target[target.len() - 1..])
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
            &"#!/bin/sh\nD=@D@\necho \"$@\" >> $D/fastboot.calls\nS=$D/fb-state\ncase \"$1\" in devices) printf 'SER1\\tfastboot\\n'; exit 0;; esac\ncase \"$3\" in\n getvar) case \"$4\" in\n  current-slot) echo \"current-slot: $(sed -n 's/^current-slot://p' $S)\" >&2;;\n  all) sed 's/^/(bootloader) /' $S >&2;;\n esac;;\n flash) cp \"$5\" \"$D/by-name/$4\"; X=\"${4#boot_}\"; sed -i -e \"s/^slot-successful:$X:.*/slot-successful:$X:no/\" -e \"s/^slot-retry-count:$X:.*/slot-retry-count:$X:1/\" $S;;\n set_active) X=\"$4\"; sed -i -e \"s/^current-slot:.*/current-slot:$X/\" -e \"s/^slot-successful:$X:.*/slot-successful:$X:no/\" -e \"s/^slot-retry-count:$X:.*/slot-retry-count:$X:7/\" -e \"s/^slot-unbootable:$X:.*/slot-unbootable:$X:no/\" $S;;\nesac\nexit 0\n"
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

    #[test]
    fn leftovers_of_an_earlier_run_are_cleared_at_step_8_and_an_unfinished_switch_stops_it() {
        let (f, _d) = the_phone_after_rerun_3("leftovers", FLAGS_AFTER_RUN_1);
        f.lab.save("flashed.txt", "boot_a old").unwrap();
        f.lab.save("parts-before.txt", "boot_a old").unwrap();
        (f.the_slot_is_read_by_adb())().unwrap();
        assert!(f.marked("flashed.txt").is_none() && f.marked("parts-before.txt").is_none());
        f.lab.save(SWITCHED_FILE, "a").unwrap();
        assert!((f.the_slot_is_read_by_adb())()
            .unwrap_err()
            .contains("never switched back"));
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
        flash_resets_only_the_flashed_slots_flags(RUN1_BEFORE, RUN1_AFTER, "a").unwrap();
    }

    #[test]
    fn the_retry_count_is_any_positive_number_not_one() {
        for n in ["1", "3", "7"] {
            let after = with(
                RUN1_AFTER,
                "slot-retry-count:a:1",
                &format!("slot-retry-count:a:{n}"),
            );
            flash_resets_only_the_flashed_slots_flags(RUN1_BEFORE, &after, "a").unwrap();
        }
        let zero = with(RUN1_AFTER, "slot-retry-count:a:1", "slot-retry-count:a:0");
        assert!(
            flash_resets_only_the_flashed_slots_flags(RUN1_BEFORE, &zero, "a")
                .unwrap_err()
                .contains("positive")
        );
        let junk = with(RUN1_AFTER, "slot-retry-count:a:1", "slot-retry-count:a:x");
        assert!(flash_resets_only_the_flashed_slots_flags(RUN1_BEFORE, &junk, "a").is_err());
    }

    #[test]
    fn nothing_else_may_change() {
        let bad = |from: &str, to: &str| {
            flash_resets_only_the_flashed_slots_flags(RUN1_BEFORE, &with(RUN1_AFTER, from, to), "a")
                .unwrap_err()
        };
        assert!(bad("slot-retry-count:b:0", "slot-retry-count:b:1").contains("slot-retry-count:b")); // b's count
        assert!(bad("slot-successful:b:yes", "slot-successful:b:no").contains("slot-successful:b")); // b's flags
        assert!(bad("slot-unbootable:b:no", "slot-unbootable:b:yes").contains("slot-unbootable:b"));
        assert!(bad("slot-unbootable:a:no", "slot-unbootable:a:yes").contains("slot-unbootable:a")); // the flashed slot becomes unbootable
        assert!(bad("current-slot:b", "current-slot:a").contains("current-slot"));
        // the current slot moved
    }

    #[test]
    fn the_flashed_slot_must_end_not_successful() {
        let still = with(RUN1_AFTER, "slot-successful:a:no", "slot-successful:a:yes");
        assert!(
            flash_resets_only_the_flashed_slots_flags(RUN1_BEFORE, &still, "a")
                .unwrap_err()
                .contains("slot-successful:a")
        );
    }

    #[test]
    fn a_flag_that_appeared_or_went_is_an_error_and_the_other_slot_is_not_allowed_to_reset() {
        let gone = RUN1_AFTER.replace("slot-unbootable:b:no", "");
        assert!(
            flash_resets_only_the_flashed_slots_flags(RUN1_BEFORE, gone.trim(), "a")
                .unwrap_err()
                .contains("names changed")
        );
        // the same text is NOT allowed for slot b: b's flags would be the ones that changed nothing, a's would have changed
        assert!(flash_resets_only_the_flashed_slots_flags(RUN1_BEFORE, RUN1_AFTER, "b").is_err());
    }

    const STOCK: &str = "2a5cd3b20ef0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DECLARED: &str = "e337de53446aa98e4795e214648427e22b467b3ab39f2d208e201951ff4098f1";

    // check 9 on the values of rerun 2 (the stock sha is the printed prefix 2a5cd3b2 padded): boot_a read now is e337de53 (the declared image, flashed and read back equal by run 1), the stored backup says 2a5cd3b2
    #[test]
    fn boot_a_that_holds_the_declared_image_passes_while_the_backup_is_on_file() {
        let said =
            boot_partition_can_be_flashed("boot_a", DECLARED, STOCK, DECLARED, Ok(STOCK.into()));
        assert!(said.unwrap().contains("can be undone"));
    }

    #[test]
    fn boot_a_that_is_the_backup_passes_as_before() {
        let said =
            boot_partition_can_be_flashed("boot_a", STOCK, STOCK, DECLARED, Err("not read".into()));
        assert!(said.unwrap().contains("is its backup"));
    }

    #[test]
    fn boot_a_that_is_neither_fails_and_says_so() {
        let other = "ab".repeat(32);
        let said =
            boot_partition_can_be_flashed("boot_a", &other, STOCK, DECLARED, Ok(STOCK.into()))
                .unwrap_err();
        assert!(said.contains("neither the stored backup"), "{said}");
    }

    #[test]
    fn the_declared_image_without_the_backup_file_fails() {
        let said = boot_partition_can_be_flashed(
            "boot_a",
            DECLARED,
            STOCK,
            DECLARED,
            Err("boot_a.img: No such file".into()),
        )
        .unwrap_err();
        assert!(said.contains("not on file"), "{said}");
    }

    #[test]
    fn the_declared_image_with_a_changed_backup_file_fails() {
        let changed = "cd".repeat(32);
        let said = boot_partition_can_be_flashed("boot_a", DECLARED, STOCK, DECLARED, Ok(changed))
            .unwrap_err();
        assert!(said.contains("hashes to"), "{said}");
    }
}
