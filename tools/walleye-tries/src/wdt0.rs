// SPDX-License-Identifier: MIT OR Apache-2.0
//! The WDT0 try's judgement (item walleye-wdt0-try-procedure): what the wdt0 image's init read from the apps watchdog's four registers
//! and kept at /run/walleye/wdt0 (and said to the kernel log), judged as text. The init is build/initramfs-root-wdt0/init (its read block, proof
//! tests/host/wdt0-init.sh); the registers are the base 0x17817000 plus the offsets android:drivers/soc/qcom/watchdog_v2.c@113bd1cfa6f5 defines
//! (research walleye-android-watchdog-read-fix). The judgement says whether the read happened and what it printed; what the values MEAN
//! (is the watchdog armed by the bootloader) is the research row's, written by the keeper from the run - this file claims nothing of it.

/// The word on the image's command line that makes its init read WDT0 (and nothing else).
pub const WORD: &str = "walleye.wdt0";

/// Every line of the init's reading starts with this tag.
pub const TAG: &str = "walleye-wdt0:";

/// The file the init keeps the reading in, in the booted root (a tmpfs: the card is not written).
pub const FILE: &str = "/run/walleye/wdt0";

/// The four registers, in the order the init reads them: name and address (WDT0 base 0x17817000 + EN 0x08, STS 0x0c, BARK_TIME 0x10, BITE_TIME 0x14).
pub const REGISTERS: [(&str, u64); 4] = [
    ("EN", 0x1781_7008),
    ("STS", 0x1781_700c),
    ("BARK_TIME", 0x1781_7010),
    ("BITE_TIME", 0x1781_7014),
];

/// What the lab told the author before the run.
pub fn expect(luns: u64) -> String {
    format!("The WDT0 try, about 6 minutes: the wdt0 image booted once from fastboot, from memory, on the slot Android is NOT on, root read-only as in the root boot (the card's {luns} disks and the boot chain read before and after and equal). Its init's first act reads the apps watchdog's four registers EN, STS, BARK_TIME and BITE_TIME with devmem, read only; the lab logs in by ssh, reads them from {FILE} and the kernel log, and judges that all four were read; the values are kept in the run's folder. Then the phone is sent back to Android. Nothing is written to the card or to any register.")
}

/// What the author is told before the rehearsal.
pub fn expect_rehearsal(luns: u64) -> String {
    format!("The rehearsal of the WDT0 try, about 5 minutes: every step but the boot, run for real - the card read from Android ({luns} disks and the root partition hashed), the image (its command line carries {WORD} once), the guard with fastboot, the slots' flags, the boot chain read back and compared. Nothing is booted and nothing is written; the phone is in fastboot for a minute and ends on Android. No go is needed or read.")
}

/// The image's command line carries the word exactly once (and not the rw word: this try never writes).
pub fn cmdline_carries_the_word(cmdline: &str) -> Result<(), String> {
    let words: Vec<&str> = cmdline.split_whitespace().collect();
    match words.iter().filter(|w| **w == WORD).count() {
        1 => {}
        0 => {
            return Err(format!(
                "the image's command line has no {WORD}: its init reads nothing"
            ))
        }
        n => {
            return Err(format!(
                "the image's command line has {n} words {WORD}, expected one"
            ))
        }
    }
    if words.contains(&crate::rw_write::WORD) {
        return Err(format!(
            "the image's command line carries {}: this try reads, it never writes",
            crate::rw_write::WORD
        ));
    }
    Ok(())
}

/// One register as the init read it.
#[derive(Debug, PartialEq)]
pub struct Reading {
    pub name: String,
    pub address: u64,
    /// the 32-bit value devmem printed, or devmem's own words when it failed
    pub value: Result<u32, String>,
}

/// The lines of `text` that carry the tag, whatever precedes it on the line (the kernel log puts its timestamp first), trimmed.
fn tagged(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|l| l.find(TAG).map(|i| l[i + TAG.len()..].trim()))
        .collect()
}

fn hex(word: &str) -> Option<u64> {
    u64::from_str_radix(word.strip_prefix("0x")?, 16).ok()
}

/// The readings in `text`: the lines `NAME ADDRESS VALUE` or `NAME ADDRESS FAILED WORDS`; the init's own `first read ...` and `read NAME ADDRESS` lines are
/// not readings. A register that is not one of the four, or at another address than its own, or twice, is refused: the init read something else.
pub fn readings(text: &str) -> Result<Vec<Reading>, String> {
    let mut out: Vec<Reading> = Vec::new();
    for line in tagged(text) {
        let mut w = line.split_whitespace();
        let (Some(name), Some(addr)) = (w.next(), w.next()) else {
            continue;
        };
        if name == "first" || name == "read" {
            continue;
        }
        let Some(&(_, own)) = REGISTERS.iter().find(|(n, _)| *n == name) else {
            return Err(format!(
                "the init read a register that is not one of the four: {line:?}"
            ));
        };
        if hex(addr) != Some(own) {
            return Err(format!(
                "{name} was read at {addr}, its own address is {own:#x}"
            ));
        }
        if out.iter().any(|r| r.name == name) {
            return Err(format!("{name} was read twice: {line:?}"));
        }
        let rest: Vec<&str> = w.collect();
        let value = match rest.as_slice() {
            ["FAILED", words @ ..] => Err(words.join(" ")),
            [v] => match hex(v).and_then(|n| u32::try_from(n).ok()) {
                Some(n) => Ok(n),
                None => {
                    return Err(format!(
                        "{name}'s value is not a 32-bit hex number: {line:?}"
                    ))
                }
            },
            _ => {
                return Err(format!(
                    "{name}'s line has neither one value nor FAILED: {line:?}"
                ))
            }
        };
        out.push(Reading {
            name: name.into(),
            address: own,
            value,
        });
    }
    Ok(out)
}

/// The judgement: the file the init kept and the kernel log say the same four readings, each a value. Fails naming what is not known:
/// nothing read (the word did not reach the init, or the hand-over lost the file), a register that failed (devmem's words), a missing one,
/// or the file and the kernel log disagreeing. Returns the readings as the init kept them.
pub fn judge(file: &str, dmesg: &str) -> Result<Vec<Reading>, String> {
    let kept = readings(file)?;
    if kept.is_empty() {
        return Err(format!("{FILE} holds no reading: the init did not read (the word {WORD} did not reach it, or the hand-over lost the file)"));
    }
    for (name, _) in REGISTERS {
        match kept.iter().find(|r| r.name == name) {
            None => {
                return Err(format!(
                    "{name} is not in {FILE}: the init read {} of the four",
                    kept.len()
                ))
            }
            Some(Reading {
                value: Err(words), ..
            }) => return Err(format!("{name} was not read: devmem said {words:?}")),
            Some(_) => {}
        }
    }
    let said = readings(dmesg)?;
    if said != kept {
        return Err(format!(
            "the kernel log's readings {said:?} are not the file's {kept:?}"
        ));
    }
    Ok(kept)
}

/// The readings as the run's folder keeps them, one line per register.
pub fn said(readings: &[Reading]) -> String {
    readings
        .iter()
        .map(|r| match &r.value {
            Ok(v) => format!("{} {:#010x} = {v:#010x}\n", r.name, r.address),
            Err(w) => format!("{} {:#010x} FAILED {w}\n", r.name, r.address),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sources: the init's own output, made by running its read block (build/initramfs-root-wdt0/init, `# WDT0-BLOCK-BEGIN (the read`) with a stand-in devmem
    // (tests/host/wdt0-init.sh does the same): the SHAPE is the init's, the values are the stand-in's and say nothing of a phone.
    const FILE_TEXT: &str = include_str!("../fixtures/wdt0-init-run-file.txt");
    const KMSG: &str = include_str!("../fixtures/wdt0-init-run-kmsg.txt");
    // Source: a rootfs boot's ssh read, printed by the phone (research walleye-rootfs-boots-run-4): the image of that boot had no WDT0 word.
    const PHONE_READ: &str = include_str!("../fixtures/rootfs-boots-run4-read.txt");

    fn at_phone_time(kmsg: &str) -> String {
        // the kernel log puts a timestamp before each line
        kmsg.lines()
            .map(|l| format!("[    7.123456] {l}\n"))
            .collect()
    }

    #[test]
    fn the_four_registers_are_the_base_plus_the_android_offsets() {
        for ((_, a), off) in REGISTERS.iter().zip([0x08, 0x0c, 0x10, 0x14]) {
            assert_eq!(*a, 0x1781_7000 + off);
        }
    }

    #[test]
    fn the_inits_file_and_kernel_log_are_judged_as_four_readings() {
        let r = judge(FILE_TEXT, &at_phone_time(KMSG)).unwrap();
        assert_eq!(
            r.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            ["EN", "STS", "BARK_TIME", "BITE_TIME"]
        );
        assert_eq!(r[0].value, Ok(3));
        assert!(said(&r).starts_with("EN 0x17817008 = 0x00000003\n"));
    }

    #[test]
    fn a_failed_register_is_named_with_devmems_words() {
        let failed = FILE_TEXT.replace(
            "STS 0x1781700c 0x00000000",
            "STS 0x1781700c FAILED devmem: mmap: Operation not permitted",
        );
        let e = judge(&failed, &failed).unwrap_err();
        assert!(
            e.contains("STS was not read") && e.contains("devmem: mmap: Operation not permitted"),
            "{e}"
        );
    }

    #[test]
    fn a_boot_that_read_nothing_says_so_with_a_phones_own_text() {
        let dmesg = PHONE_READ
            .split("---dmesg")
            .nth(1)
            .expect("the phone's read has a dmesg section");
        let e = judge("", dmesg).unwrap_err();
        assert!(e.contains("holds no reading"), "{e}");
    }

    #[test]
    fn a_missing_register_the_other_address_a_repeat_and_a_stranger_are_refused() {
        let without = FILE_TEXT.replace("walleye-wdt0: BITE_TIME 0x17817014 0x0000FFFF\n", "");
        assert!(judge(&without, KMSG)
            .unwrap_err()
            .contains("BITE_TIME is not in"));
        let moved = FILE_TEXT.replace("EN 0x17817008", "EN 0x1781700c");
        assert!(readings(&moved)
            .unwrap_err()
            .contains("its own address is 0x17817008"));
        let twice = format!("{FILE_TEXT}walleye-wdt0: EN 0x17817008 0x00000003\n");
        assert!(readings(&twice).unwrap_err().contains("twice"));
        let stranger = format!("{FILE_TEXT}walleye-wdt0: PET 0x1781700c 0x00000001\n");
        assert!(readings(&stranger)
            .unwrap_err()
            .contains("not one of the four"));
    }

    #[test]
    fn the_kernel_log_must_say_what_the_file_keeps() {
        let other = at_phone_time(KMSG).replace("0x0000AAAA", "0x0000BBBB");
        assert!(judge(FILE_TEXT, &other)
            .unwrap_err()
            .contains("are not the file's"));
        assert!(judge(FILE_TEXT, "")
            .unwrap_err()
            .contains("are not the file's"));
    }

    #[test]
    fn the_word_is_on_the_command_line_once_and_the_rw_word_never() {
        let ok = "clk_ignore_unused walleye.root=system_a walleye.wdt0";
        assert!(cmdline_carries_the_word(ok).is_ok());
        assert!(cmdline_carries_the_word("walleye.root=system_a")
            .unwrap_err()
            .contains("no walleye.wdt0"));
        assert!(cmdline_carries_the_word("walleye.wdt0 walleye.wdt0")
            .unwrap_err()
            .contains("2 words"));
        assert!(cmdline_carries_the_word("walleye.wdt0x").is_err());
        assert!(cmdline_carries_the_word(&format!("{ok} walleye.rw"))
            .unwrap_err()
            .contains("never writes"));
    }
}
