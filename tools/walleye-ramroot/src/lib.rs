// SPDX-License-Identifier: MIT OR Apache-2.0
//! What the Alpine-in-RAM image is made of and how it is read back: pure functions, no process, no disk.

pub mod cli;

/// Every boot image of ours is this big: the boot partitions are 0x2000000 bytes (fastboot getvar partition-size:boot_a).
pub const BOOT_PARTITION_BYTES: u64 = 0x200_0000;
/// What fastboot takes in one download (getvar max-download-size on this phone).
pub const FASTBOOT_MAX_DOWNLOAD_BYTES: u64 = 0x2000_0000;
/// The ramdisk must load after the kernel with at least this much room (the headroom the other image builders hold).
pub const MIN_HEADROOM_BYTES: u64 = 524_288;
/// Every file's time in the tar and the ramdisk: 2026-01-01, so the image is the same twice.
pub const FILE_TIME: &str = "@1767225600";
/// The tag the init writes its kernel log lines with.
pub const INIT_TAG: &str = "walleye-ramroot:";

/// The `path` and `sha256` of `[images.NAME]` in device.toml.
pub fn image_entry(toml: &str, name: &str) -> Result<(String, String), String> {
    let header = format!("[images.{name}]");
    let mut inside = false;
    let (mut path, mut sha) = (None, None);
    for line in toml.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            inside = l == header;
            continue;
        }
        if !inside {
            continue;
        }
        let value = |key: &str| {
            l.strip_prefix(key)
                .and_then(|r| r.trim_start().strip_prefix('='))
                .and_then(|r| r.trim_start().strip_prefix('"'))
                .and_then(|r| r.split('"').next())
                .map(str::to_string)
        };
        path = path.or_else(|| value("path"));
        sha = sha.or_else(|| value("sha256"));
    }
    match (path, sha) {
        (Some(p), Some(s)) => Ok((p, s)),
        _ => Err(format!(
            "device.toml has no [images.{name}] with a path and a sha256"
        )),
    }
}

/// The paths of `debugfs -f` output of `stat` commands whose mode has setuid, setgid or sticky bits (the 07000 part), with the whole mode.
/// debugfs echoes `debugfs: stat PATH`, then prints `Inode: N   Type: regular    Mode:  04111   Flags: ...`.
pub fn special_modes(stat_output: &str) -> Vec<(String, u32)> {
    let mut out = Vec::new();
    let mut path: Option<String> = None;
    for line in stat_output.lines() {
        if let Some(p) = line.strip_prefix("debugfs: stat ") {
            path = Some(p.trim().to_string());
        } else if line.starts_with("Inode:") {
            let mode = line
                .split("Mode:")
                .nth(1)
                .and_then(|r| r.split_whitespace().next())
                .and_then(|m| u32::from_str_radix(m, 8).ok());
            if let (Some(p), Some(m)) = (path.take(), mode) {
                if m & 0o7000 != 0 {
                    out.push((p, m & 0o7777));
                }
            }
        }
    }
    out
}

/// One thing in the ramdisk's tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Dir,
    File { executable: bool },
    Link(String),
}

/// The gen_init_cpio spec of a tree: `rel` is the path inside the ramdisk without a leading slash, `src_root` where its files are read from.
/// Every owner is 0 0; a file is 644 or 755, a directory 755. Entries are written sorted, so the spec is the same twice.
pub fn spec_lines(entries: &[(String, Kind)], src_root: &str) -> String {
    let mut sorted: Vec<&(String, Kind)> = entries.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::new();
    for (rel, kind) in sorted {
        match kind {
            Kind::Dir => out.push_str(&format!("dir /{rel} 755 0 0\n")),
            Kind::File { executable } => out.push_str(&format!(
                "file /{rel} {src_root}/{rel} {} 0 0\n",
                if *executable { 755 } else { 644 }
            )),
            Kind::Link(target) => out.push_str(&format!("slink /{rel} {target} 777 0 0\n")),
        }
    }
    out
}

/// The kernel release a `Linux version X ...` banner names, if the text has one.
pub fn banner_release(text: &str) -> Option<String> {
    let rest = text.split("Linux version ").nth(1)?;
    let word: String = rest
        .chars()
        .take_while(|c| !c.is_whitespace() && *c != '\0')
        .collect();
    (!word.is_empty()).then_some(word)
}

/// The kernel release a module's vermagic names (its first word: `7.3.0-00007-g7430c717a94c-dirty SMP preempt mod_unload aarch64`).
pub fn vermagic_release(vermagic: &str) -> Option<String> {
    vermagic.split_whitespace().next().map(str::to_string)
}

/// The module files a list names: the last word of each `insmod PATH` line, blank lines left out.
pub fn module_paths(list: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for line in list.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let mut w = line.split_whitespace();
        match (w.next(), w.next(), w.next()) {
            (Some("insmod"), Some(p), None) if p.starts_with('/') => out.push(p.to_string()),
            _ => return Err(format!("not an `insmod /PATH` line: {line}")),
        }
    }
    Ok(out)
}

/// The room between the end of the kernel (at its load address) and the ramdisk's load address, in bytes (negative: they overlap).
pub fn headroom(kernel_load: u64, kernel_size: u64, ramdisk_load: u64) -> i64 {
    ramdisk_load as i64 - (kernel_load + kernel_size) as i64
}

/// The paths of a `tar -t` listing that do not start with `./`, `.` or a name, and so cannot be read as a path inside the root.
pub fn tar_paths(listing: &str) -> Vec<String> {
    listing
        .lines()
        .map(|l| l.trim_start_matches("./").trim_end_matches('/').to_string())
        .filter(|l| !l.is_empty() && l != ".")
        .collect()
}

/// What an Alpine root needs to start and to be reached, each as a path that must be in the userland tar.
pub const MUST_BE_IN_THE_USERLAND: &[&str] = &[
    "sbin/init",
    "sbin/openrc",
    "sbin/getty",
    "sbin/agetty",
    "etc/inittab",
    "etc/init.d/usb-gadget",
    "etc/init.d/sshd",
    "usr/sbin/sshd",
    "root/.ssh/authorized_keys",
    "bin/busybox",
];

/// Why the inittab does not give the glass and the serial a login, if it does not: a respawned getty on tty0 and one on ttyGS0.
pub fn inittab_problem(inittab: &str) -> Option<String> {
    let has = |tty: &str| {
        inittab.lines().map(str::trim).any(|l| {
            !l.starts_with('#') && l.starts_with(&format!("{tty}::respawn:")) && l.contains("getty")
        })
    };
    for tty in ["tty0", "ttyGS0"] {
        if !has(tty) {
            return Some(format!("/etc/inittab has no respawned getty on {tty}"));
        }
    }
    None
}

/// Words the init may not hold outside comments: it touches no block device.
pub const INIT_FORBIDDEN: &[&str] = &[
    "mkfs", "mke2fs", "fsck", "e2fsck", "dd", "fstrim", "blockdev", "ext4", "remount",
];

/// The forbidden words the init's code (comments left out) uses, as whole words.
pub fn init_forbidden_words(init: &str) -> Vec<String> {
    let code: String = init
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let mut found = Vec::new();
    for w in INIT_FORBIDDEN {
        let hit = code
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .any(|t| t == *w);
        if hit {
            found.push((*w).to_string());
        }
    }
    found
}

/// The first line of a tool's output that is not blank.
pub fn first_line(text: &str) -> String {
    text.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// The host tools a run needs and PATH lacks.
pub fn missing_tools<'a>(need: &[&'a str], have: &dyn Fn(&str) -> bool) -> Vec<&'a str> {
    need.iter().copied().filter(|t| !have(t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOML: &str = "[images.b-r5]\npath = \"images-display-reset/boot-b-r5.img\"\nsha256 = \"6dc0\"\n\n[images.other]\npath = \"x\"\nsha256 = \"y\"\n";

    #[test]
    fn the_image_entry_is_read_by_name() {
        assert_eq!(
            image_entry(TOML, "b-r5").unwrap(),
            ("images-display-reset/boot-b-r5.img".into(), "6dc0".into())
        );
        assert_eq!(image_entry(TOML, "other").unwrap().0, "x");
        assert!(image_entry(TOML, "b-r").is_err());
        assert!(image_entry("[images.z]\npath = \"p\"\n", "z").is_err());
    }

    #[test]
    fn only_special_bits_are_named() {
        let out = "debugfs: stat /bin/bbsuid\nInode: 17   Type: regular    Mode:  04111   Flags: 0x80000\ndebugfs: stat /bin/busybox\nInode: 18   Type: regular    Mode:  0755   Flags: 0x80000\ndebugfs: stat /tmp\nInode: 19   Type: directory    Mode:  01777   Flags: 0x80000\n";
        assert_eq!(
            special_modes(out),
            vec![("/bin/bbsuid".into(), 0o4111), ("/tmp".into(), 0o1777)]
        );
    }

    #[test]
    fn the_spec_is_sorted_and_owned_by_root() {
        let e = vec![
            ("lib/a.ko".to_string(), Kind::File { executable: false }),
            ("bin".to_string(), Kind::Dir),
            ("bin/sh".to_string(), Kind::Link("busybox".into())),
            ("bin/busybox".to_string(), Kind::File { executable: true }),
        ];
        assert_eq!(
            spec_lines(&e, "/r"),
            "dir /bin 755 0 0\nfile /bin/busybox /r/bin/busybox 755 0 0\nslink /bin/sh busybox 777 0 0\nfile /lib/a.ko /r/lib/a.ko 644 0 0\n"
        );
    }

    #[test]
    fn the_release_is_read_from_the_banner_and_the_vermagic() {
        let b = "xx\0Linux version 7.3.0-00007-g7430c717a94c-dirty (mll@host) #1 SMP";
        assert_eq!(
            banner_release(b).unwrap(),
            "7.3.0-00007-g7430c717a94c-dirty"
        );
        assert_eq!(banner_release("no banner"), None);
        assert_eq!(
            vermagic_release("7.3.0-00007-g7430c717a94c-dirty SMP preempt mod_unload aarch64")
                .unwrap(),
            "7.3.0-00007-g7430c717a94c-dirty"
        );
        assert_eq!(vermagic_release(""), None);
    }

    #[test]
    fn the_module_list_is_insmod_lines_of_absolute_paths() {
        assert_eq!(
            module_paths("insmod /lib/a.ko\n\ninsmod /lib/b.ko\n").unwrap(),
            vec!["/lib/a.ko", "/lib/b.ko"]
        );
        assert!(module_paths("modprobe a\n").is_err());
        assert!(module_paths("insmod lib/a.ko\n").is_err());
        assert!(module_paths("insmod /a.ko --force\n").is_err());
    }

    #[test]
    fn headroom_is_negative_when_they_overlap() {
        assert_eq!(headroom(0x8000, 15_768_943, 0x100_0000), 975_505);
        assert!(headroom(0x8000, 17_000_000, 0x100_0000) < 0);
    }

    #[test]
    fn the_inittab_needs_both_gettys() {
        let ok = "tty0::respawn:/sbin/getty 38400 tty0\nttyGS0::respawn:/sbin/getty -L 115200 ttyGS0 vt100\n";
        assert_eq!(inittab_problem(ok), None);
        assert!(inittab_problem("ttyGS0::respawn:/sbin/getty ttyGS0\n")
            .unwrap()
            .contains("tty0"));
        assert!(inittab_problem(
            "#tty0::respawn:/sbin/getty tty0\nttyGS0::respawn:/sbin/getty x\n"
        )
        .unwrap()
        .contains("tty0"));
        assert!(
            inittab_problem("tty0::once:/sbin/getty tty0\nttyGS0::respawn:/sbin/getty x\n")
                .is_some()
        );
    }

    #[test]
    fn the_init_check_reads_code_not_comments() {
        let init = "# no mkfs, no dd here\nmount -t tmpfs tmpfs /newroot\nsync\n";
        assert!(init_forbidden_words(init).is_empty());
        let bad = "mount -t ext4 -o ro /dev/sda1 /lower\ndd if=/dev/zero of=/dev/sda1\n";
        assert_eq!(init_forbidden_words(bad), vec!["dd", "ext4"]);
        // a word inside a longer name is not the word
        assert!(init_forbidden_words("echo addd fsckx xdd\n").is_empty());
    }

    #[test]
    fn tar_paths_drop_the_dot_and_the_slashes() {
        assert_eq!(
            tar_paths("./\n./bin/\n./bin/busybox\n"),
            vec!["bin", "bin/busybox"]
        );
    }

    #[test]
    fn missing_tools_names_only_the_absent() {
        let have = |t: &str| t == "tar";
        assert_eq!(missing_tools(&["tar", "xz"], &have), vec!["xz"]);
    }

    #[test]
    fn the_first_line_skips_blanks() {
        assert_eq!(first_line("\n  \n pass x\nmore"), "pass x");
    }
}
