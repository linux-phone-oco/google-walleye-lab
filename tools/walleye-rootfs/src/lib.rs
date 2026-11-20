// SPDX-License-Identifier: MIT OR Apache-2.0
//! What the image may and may not hold, and how to read Alpine's package database: pure functions, no process, no disk.

pub mod cli;

/// The size the item allows, sparse: 2.5 GiB.
pub const MAX_IMAGE_BYTES: u64 = 5 * 1024 * 1024 * 1024 / 2;

/// Words of paths that must never be in the image: the phone's firmware and modem data (nothing of the phone is copied).
const NO_PHONE_FILES: &[&str] = &[
    "lib/firmware/",
    "vendor/",
    "firmware/",
    "persist/",
    "modem/",
    "radio/",
    "efs/",
];
const NO_PHONE_SUFFIXES: &[&str] = &[".mbn", ".mdt", ".jsn", ".fw", ".elf.xz"];
/// Names of packages and programs of a phone UI: the image has none.
const NO_UI: &[&str] = &[
    "sxmo",
    "phosh",
    "plasma",
    "sway",
    "weston",
    "xorg",
    "xorg-server",
    "wayland",
    "wlroots",
    "gnome",
    "kde",
    "lightdm",
    "greetd",
    "firmware",
    "linux-firmware",
];

/// Why a path (relative, no leading slash) does not belong in the image, if it does not.
pub fn forbidden_path(path: &str) -> Option<String> {
    let p = path.trim_start_matches('/');
    for d in NO_PHONE_FILES {
        if p.starts_with(d) && p.len() > d.len() {
            return Some(format!(
                "{p}: under {d}, where a phone's firmware or modem data would be"
            ));
        }
    }
    for s in NO_PHONE_SUFFIXES {
        if p.ends_with(s) {
            return Some(format!("{p}: a firmware or modem file by its suffix {s}"));
        }
    }
    None
}

/// Why a package does not belong in the image, if it does not: a phone UI or firmware.
pub fn forbidden_package(name: &str) -> Option<String> {
    for w in NO_UI {
        if name == *w || name.starts_with(&format!("{w}-")) || name.starts_with(&format!("{w}_")) {
            return Some(format!("{name}: a UI or firmware package ({w})"));
        }
    }
    None
}

/// Name and version of every package in Alpine's installed database (`lib/apk/db/installed`: P: and V: lines).
pub fn installed_packages(db: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut name: Option<String> = None;
    for line in db.lines() {
        if let Some(n) = line.strip_prefix("P:") {
            name = Some(n.to_string());
        } else if let Some(v) = line.strip_prefix("V:") {
            if let Some(n) = name.take() {
                out.push((n, v.to_string()));
            }
        }
    }
    out
}

/// The list the research row holds: `name-version`, one per line, sorted.
pub fn package_list(pkgs: &[(String, String)]) -> String {
    let mut v: Vec<String> = pkgs.iter().map(|(n, ver)| format!("{n}-{ver}")).collect();
    v.sort();
    v.join("\n")
}

/// True when the bytes are an aarch64 ELF (e_machine 0xB7 at offset 18, little endian).
pub fn is_aarch64_elf(head: &[u8]) -> bool {
    head.len() >= 20 && head[..4] == [0x7f, b'E', b'L', b'F'] && head[18] == 0xb7 && head[19] == 0
}

/// The services of `runlevels` as (level, service), comments and blank lines left out.
pub fn runlevel_lines(text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            Some((it.next()?.to_string(), it.next()?.to_string()))
        })
        .collect()
}

/// The version of one package in an APKINDEX text (blocks split by blank lines, `P:` name, `V:` version), if it is there.
pub fn index_version(index: &str, pkg: &str) -> Option<String> {
    let want = format!("P:{pkg}");
    index.split("\n\n").find_map(|block| {
        let mut lines = block.lines();
        lines
            .clone()
            .any(|l| l == want)
            .then(|| lines.find_map(|l| l.strip_prefix("V:").map(str::to_string)))?
    })
}

/// The applet links `busybox --install -s` would make (its post-install script, skipped): the paths of
/// `etc/busybox-paths.d/busybox`, one per line, without the busybox binary itself.
pub fn busybox_links(paths: &str) -> Vec<String> {
    paths
        .lines()
        .map(str::trim)
        .map(|l| l.trim_start_matches('/'))
        .filter(|l| !l.is_empty() && *l != "bin/busybox")
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firmware_and_modem_paths_are_refused() {
        assert!(forbidden_path("lib/firmware/qcom/a630_sqe.fw").is_some());
        assert!(forbidden_path("/vendor/lib/x.so").is_some());
        assert!(forbidden_path("data/modem.mbn").is_some());
        assert!(forbidden_path("etc/inittab").is_none());
        assert!(
            forbidden_path("lib/firmware").is_none(),
            "the bare directory is no file"
        );
    }

    #[test]
    fn ui_packages_are_refused_and_the_wanted_ones_pass() {
        assert!(forbidden_package("phosh").is_some());
        assert!(forbidden_package("sxmo-utils").is_some());
        assert!(forbidden_package("linux-firmware-other").is_some());
        for ok in [
            "openssh",
            "qrtr",
            "rmtfs",
            "tqftpserv",
            "alpine-base",
            "busybox",
        ] {
            assert!(forbidden_package(ok).is_none(), "{ok}");
        }
    }

    #[test]
    fn the_package_database_is_read() {
        let db = "C:Q1x\nP:musl\nV:1.2.5-r1\nA:aarch64\n\nC:Q1y\nP:openssh\nV:10.0_p1-r0\n";
        let p = installed_packages(db);
        assert_eq!(
            p,
            vec![
                ("musl".into(), "1.2.5-r1".into()),
                ("openssh".into(), "10.0_p1-r0".into())
            ]
        );
        assert_eq!(package_list(&p), "musl-1.2.5-r1\nopenssh-10.0_p1-r0");
    }

    #[test]
    fn an_elf_is_told_by_its_machine() {
        let mut h = [0u8; 20];
        h[..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);
        h[18] = 0xb7;
        assert!(is_aarch64_elf(&h));
        h[18] = 0x3e; // x86_64
        assert!(!is_aarch64_elf(&h));
        assert!(!is_aarch64_elf(&h[..10]));
    }

    #[test]
    fn runlevel_lines_skip_comments() {
        let r = runlevel_lines("# c\n\nsysinit devfs\ndefault sshd\n");
        assert_eq!(
            r,
            vec![
                ("sysinit".into(), "devfs".into()),
                ("default".into(), "sshd".into())
            ]
        );
    }

    #[test]
    fn the_index_gives_a_version() {
        let idx = "C:a\nP:apk-tools\nV:3.0.9-r0\n\nC:b\nP:apk-tools-static\nV:3.0.9-r1\nA:x86_64\n";
        assert_eq!(
            index_version(idx, "apk-tools-static").as_deref(),
            Some("3.0.9-r1")
        );
        assert_eq!(index_version(idx, "apk-tools").as_deref(), Some("3.0.9-r0"));
        assert_eq!(index_version(idx, "nothing"), None);
    }

    #[test]
    fn busybox_links_are_the_listed_paths() {
        let l = busybox_links("bin/busybox\nbin/ash\n/sbin/getty\n\n");
        assert_eq!(l, vec!["bin/ash".to_string(), "sbin/getty".to_string()]);
    }
}
