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

/// True when the whole file is an aarch64 ELF64 with no PT_INTERP program header: nothing to load but itself, so no library to miss.
pub fn is_static_aarch64_elf(data: &[u8]) -> bool {
    const PT_LOAD: u32 = 1;
    const PT_INTERP: u32 = 3;
    const PHDR_MIN: usize = 56;
    if !(is_aarch64_elf(data) && data.len() >= 64 && data[4] == 2 && data[5] == 1) {
        return false;
    }
    let le16 = |at: usize| u16::from_le_bytes([data[at], data[at + 1]]);
    let le64 = |at: usize| u64::from_le_bytes(data[at..at + 8].try_into().unwrap());
    let (kind, size, count) = (le16(16), le16(54) as usize, le16(56) as usize);
    let Ok(off) = usize::try_from(le64(32)) else {
        return false;
    };
    if !(kind == 2 || kind == 3) || count == 0 || size < PHDR_MIN {
        return false;
    }
    let types: Option<Vec<u32>> = (0..count)
        .map(|i| {
            let at = off.checked_add(i.checked_mul(size)?)?;
            let end = at.checked_add(4)?;
            Some(u32::from_le_bytes(data.get(at..end)?.try_into().ok()?))
        })
        .collect();
    types.is_some_and(|t| t.contains(&PT_LOAD) && !t.contains(&PT_INTERP))
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

/// One pinned download: the package's name, its exact version and the sha256 of its .apk (supply chain: the build trusts nothing it did not hash).
#[derive(Debug, Clone, PartialEq)]
pub struct Pin {
    pub name: String,
    pub version: String,
    pub sha256: String,
}

/// The pins file: `NAME VERSION SHA256` per line, comments and blank lines left out.
pub fn parse_pins(text: &str) -> Result<Vec<Pin>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let w: Vec<&str> = l.split_whitespace().collect();
        if w.len() != 3 {
            return Err(format!(
                "pins line {}: want NAME VERSION SHA256, got `{l}`",
                n + 1
            ));
        }
        out.push(Pin {
            name: w[0].into(),
            version: w[1].into(),
            sha256: w[2].to_lowercase(),
        });
    }
    Ok(out)
}

/// Ok when the file's sha256 is the pinned one; the error names the pin and both sums.
pub fn check_pin(pin: &Pin, actual: &str) -> Result<(), String> {
    if pin.sha256.eq_ignore_ascii_case(actual) {
        Ok(())
    } else {
        Err(format!(
            "{}-{}: sha256 {actual} is not the pinned {}",
            pin.name, pin.version, pin.sha256
        ))
    }
}

/// Every path of apk's installed database with the owner and mode apk records, octal: a file's from its `a:UID:GID:MODE` line (after `R:`),
/// a directory's from its `M:UID:GID:MODE` line (after `F:`). A path with no such line has apk's default and nothing to apply.
pub fn database_files(db: &str) -> Vec<(String, u32, u32, u32)> {
    let mut out = Vec::new();
    let mut dir = String::new();
    let mut target: Option<String> = None;
    for line in db.lines() {
        if let Some(d) = line.strip_prefix("F:") {
            dir = d.to_string();
            target = Some(dir.clone());
        } else if let Some(f) = line.strip_prefix("R:") {
            target = Some(if dir.is_empty() {
                f.to_string()
            } else {
                format!("{dir}/{f}")
            });
        } else if line.starts_with("P:") {
            dir.clear();
            target = None;
        } else if let Some(a) = line.strip_prefix("a:").or_else(|| line.strip_prefix("M:")) {
            let w: Vec<&str> = a.split(':').collect();
            if let (Some(t), [u, g, m]) = (target.clone(), &w[..]) {
                if let (Ok(u), Ok(g), Ok(m)) = (u.parse(), g.parse(), u32::from_str_radix(m, 8)) {
                    out.push((t, u, g, m));
                }
            }
        }
    }
    out
}

/// The files by (uid, gid, mode): one chown and one chmod per group, the files passed as arguments.
pub fn group_modes(
    files: &[(String, u32, u32, u32)],
) -> std::collections::BTreeMap<(u32, u32, u32), Vec<String>> {
    let mut g: std::collections::BTreeMap<(u32, u32, u32), Vec<String>> = Default::default();
    for (p, u, gid, m) in files {
        g.entry((*u, *gid, *m)).or_default().push(p.clone());
    }
    g
}

/// What one path of the image is.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    File,
    Dir,
    Link(String),
}

/// Paths made at run time: a link into them is not dangling.
const MADE_AT_RUN: &[&str] = &["proc", "sys", "run", "dev", "tmp"];

/// The image's paths (relative, no leading slash), for the link check.
#[derive(Default)]
pub struct Tree(std::collections::BTreeMap<String, Entry>);

impl Tree {
    pub fn add(&mut self, path: &str, e: Entry) {
        self.0.insert(path.trim_start_matches('/').to_string(), e);
    }

    /// `path -> target` of every symlink that ends nowhere (links through links and linked directories are followed).
    pub fn dangling_links(&self) -> Vec<String> {
        self.0
            .iter()
            .filter_map(|(p, e)| match e {
                Entry::Link(t) if !self.resolves(&link_target(p, t), 0) => {
                    Some(format!("{p} -> {t}"))
                }
                _ => None,
            })
            .collect()
    }

    /// True when `path` (from the root) leads to something in the image, links followed (run-time directories count).
    pub fn exists(&self, path: &str) -> bool {
        self.resolves(path, 0)
    }

    fn resolves(&self, path: &str, depth: u32) -> bool {
        if depth > 40 {
            return false;
        }
        let comps: Vec<&str> = path
            .split('/')
            .filter(|c| !c.is_empty() && *c != ".")
            .collect();
        let mut cur: Vec<&str> = Vec::new();
        for (i, c) in comps.iter().enumerate() {
            if *c == ".." {
                cur.pop();
                continue;
            }
            cur.push(c);
            if cur.len() == 1 && MADE_AT_RUN.contains(c) {
                return true;
            }
            match self.0.get(&cur.join("/")) {
                Some(Entry::Link(t)) => {
                    let mut next = link_target(&cur.join("/"), t);
                    for r in &comps[i + 1..] {
                        next = format!("{next}/{r}");
                    }
                    return self.resolves(&next, depth + 1);
                }
                Some(_) => {}
                None => return false,
            }
        }
        true
    }
}

/// Where a link at `path` pointing to `target` leads, as a path from the root (a relative target starts at the link's directory).
fn link_target(path: &str, target: &str) -> String {
    match target.strip_prefix('/') {
        Some(t) => t.to_string(),
        None => match path.rsplit_once('/') {
            Some((dir, _)) => format!("{dir}/{target}"),
            None => target.to_string(),
        },
    }
}

/// The files (path, mode) in a program directory that no one may run: the programs of bin, sbin and init.d need an execute bit.
pub fn not_executable(files: &[(String, u32)]) -> Vec<String> {
    const PROGRAM_DIRS: &[&str] = &[
        "bin",
        "sbin",
        "usr/bin",
        "usr/sbin",
        "usr/local/bin",
        "usr/local/sbin",
        "etc/init.d",
    ];
    files
        .iter()
        .filter(|(p, m)| {
            p.rsplit_once('/')
                .is_some_and(|(d, _)| PROGRAM_DIRS.contains(&d))
                && m & 0o111 == 0
        })
        .map(|(p, _)| p.clone())
        .collect()
}

/// The fields of the inittab lines: (id, action, program with its arguments), comments and lines without a program left out.
fn inittab_lines(tab: &str) -> Vec<(String, String, String)> {
    tab.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let f: Vec<&str> = l.splitn(4, ':').collect();
            (f.len() == 4 && !f[3].trim().is_empty())
                .then(|| (f[0].to_string(), f[2].to_string(), f[3].trim().to_string()))
        })
        .collect()
}

/// The program (first word) every inittab line starts.
pub fn inittab_programs(tab: &str) -> Vec<String> {
    inittab_lines(tab)
        .into_iter()
        .filter_map(|(_, _, p)| p.split_whitespace().next().map(str::to_string))
        .collect()
}

/// What a conf.d file makes OpenRC start: the services it needs, uses or wants, and the `command=` it runs.
pub fn conf_d_started(text: &str) -> (Vec<String>, Vec<String>) {
    let (mut services, mut commands) = (Vec::new(), Vec::new());
    for l in text.lines().map(str::trim).filter(|l| !l.starts_with('#')) {
        let Some((k, v)) = l.split_once('=') else {
            continue;
        };
        let v = v.trim().trim_matches(|c| c == '"' || c == '\'');
        match k.trim() {
            "rc_need" | "rc_use" | "rc_want" => {
                services.extend(v.split_whitespace().map(str::to_string))
            }
            "command" if !v.is_empty() => commands.push(v.to_string()),
            _ => {}
        }
    }
    (services, commands)
}

/// The program a `command=` value names: the value itself, or the default of `${VAR:-"default"}`; None when it is any other expansion.
pub fn command_default(v: &str) -> Option<String> {
    if !v.contains('$') {
        return Some(v.to_string());
    }
    let d = v.strip_prefix("${")?.split_once(":-")?.1;
    let d = d.split('}').next()?.trim_matches(|c| c == '"' || c == '\'');
    (!d.is_empty() && !d.contains('$')).then(|| d.to_string())
}

/// Ok when the inittab's line for `tty` is a respawning getty: it is tried again until the gadget has made the tty.
pub fn getty_respawns(tab: &str, tty: &str) -> Result<(), String> {
    let (_, action, prog) = inittab_lines(tab)
        .into_iter()
        .find(|(id, _, _)| id == tty)
        .ok_or(format!("inittab has no line for {tty}"))?;
    if action != "respawn" {
        return Err(format!("inittab: {tty} is `{action}`, not respawn: the getty would end before the gadget binds"));
    }
    if prog.split_whitespace().next() != Some("/sbin/getty")
        || !prog.split_whitespace().any(|w| w == tty)
    {
        return Err(format!(
            "inittab: {tty} runs `{prog}`, not /sbin/getty on {tty}"
        ));
    }
    Ok(())
}

/// The glass prompt comes last (item walleye-display-alpine-getty-experiments): a script of /etc/local.d, which OpenRC's `local` service runs at the end of the default runlevel, kills the tty1 getty
/// the inittab respawns, so the fresh prompt prints below all of init's output. Err says what is missing: the script must be executable, its pkill pattern must match the inittab's own tty1 line.
pub fn glass_prompt_respawn(tab: &str, script: &str, mode: u32) -> Result<(), String> {
    let (_, _, prog) = inittab_lines(tab)
        .into_iter()
        .find(|(id, _, _)| id == "tty1")
        .ok_or("inittab has no line for tty1")?;
    if mode & 0o111 == 0 {
        return Err(format!(
            "the local.d script is mode {mode:o}: `local` runs only executable files"
        ));
    }
    let want = format!("pkill -f '^{}$'", prog.trim());
    let lines: Vec<&str> = script.lines().map(str::trim_start).collect();
    let pkill = lines
        .iter()
        .position(|l| l.starts_with(&want))
        .ok_or_else(|| format!("the local.d script has no line starting `{want}`"))?;
    // the console shows warnings and worse (`dmesg -n 3`) BEFORE the getty is respawned: the kernel's late info lines would print below the prompt
    let quiet = lines
        .iter()
        .position(|l| l.starts_with("dmesg -n 3 ") || *l == "dmesg -n 3")
        .ok_or("the local.d script has no `dmesg -n 3` line: late kernel info lines would print below the prompt")?;
    if quiet > pkill {
        return Err(
            "the local.d script lowers the console loglevel after it respawns the getty".into(),
        );
    }
    Ok(())
}

/// Every dotted address of a text with its line (1-based).
pub fn addresses_in(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or_default();
        for tok in line.split(|c: char| !(c.is_ascii_digit() || c == '.')) {
            let t = tok.trim_matches('.');
            let parts: Vec<&str> = t.split('.').collect();
            if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
                out.push((n + 1, t.to_string()));
            }
        }
    }
    out
}

/// Why an address found in the image is not the device's network (device.toml's address, its /24), if it is not.
pub fn address_problem(found: &str, device: &str) -> Option<String> {
    let net = |a: &str| a.rsplit_once('.').map(|(n, _)| n.to_string());
    (net(found) != net(device))
        .then(|| format!("{found} is not in device.toml's network of {device}"))
}

/// The tools of `needed` that `have` does not find.
pub fn missing_tools<'a>(needed: &[&'a str], have: &dyn Fn(&str) -> bool) -> Vec<&'a str> {
    needed.iter().copied().filter(|t| !have(t)).collect()
}

/// Whether a text holds the armor of a private key (OpenSSH, RSA, EC, plain PKCS8): none may be in the image.
pub fn holds_private_key(text: &str) -> bool {
    holds_private_key_bytes(text.as_bytes())
}

/// The same over any bytes, anywhere in them: `-----BEGIN [KIND ]PRIVATE KEY-----`; a program (ELF) is no key, ssh and sshd carry the words.
pub fn holds_private_key_bytes(data: &[u8]) -> bool {
    if data.starts_with(b"\x7fELF") {
        return false;
    }
    const BEGIN: &[u8] = b"-----BEGIN ";
    const END: &[u8] = b"PRIVATE KEY-----";
    data.windows(BEGIN.len()).enumerate().any(|(i, w)| {
        w == BEGIN
            && data[i + BEGIN.len()..]
                .windows(END.len())
                .take(12)
                .any(|e| e == END)
    })
}

/// The mode of debugfs's `stat` output (`Mode:  4111`), octal.
pub fn debugfs_mode(out: &str) -> Option<u32> {
    let rest = out.split("Mode:").nth(1)?;
    u32::from_str_radix(rest.split_whitespace().next()?, 8).ok()
}

/// Every `stat` block of a `debugfs -f` run: (path, mode, uid, gid).
pub fn debugfs_stats(out: &str) -> Vec<(String, u32, u32, u32)> {
    let mut res = Vec::new();
    let mut path: Option<String> = None;
    let mut mode = None;
    for l in out.lines() {
        if let Some(p) = l
            .strip_prefix("debugfs:")
            .and_then(|r| r.trim_start().strip_prefix("stat "))
        {
            path = Some(p.trim().trim_start_matches('/').to_string());
            mode = None;
        } else if l.contains("Mode:") {
            mode = debugfs_mode(l);
        } else if let (Some(rest), true) = (l.strip_prefix("User:"), path.is_some()) {
            let w: Vec<&str> = rest.split_whitespace().collect();
            let uid = w.first().and_then(|u| u.parse::<u32>().ok());
            let gid = w
                .iter()
                .position(|x| *x == "Group:")
                .and_then(|i| w.get(i + 1))
                .and_then(|g| g.parse::<u32>().ok());
            if let (Some(p), Some(m), Some(u), Some(g)) = (path.take(), mode, uid, gid) {
                res.push((p, m, u, g));
            }
        }
    }
    res
}

/// The virtual services an OpenRC script provides (`provide logger` in its depend()).
pub fn init_script_provides(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix("provide "))
        .flat_map(|r| r.split_whitespace().map(str::to_string).collect::<Vec<_>>())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 64-byte ELF64 header and one 56-byte program header of the given type: aarch64, little endian.
    fn elf_with_header_type(p_type: u32) -> Vec<u8> {
        let mut b = vec![0u8; 64 + 56];
        b[..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);
        b[4] = 2;
        b[5] = 1;
        b[16] = 2;
        b[18] = 0xb7;
        b[32..40].copy_from_slice(&64u64.to_le_bytes());
        b[54..56].copy_from_slice(&56u16.to_le_bytes());
        b[56..58].copy_from_slice(&1u16.to_le_bytes());
        b[64..68].copy_from_slice(&p_type.to_le_bytes());
        b
    }

    #[test]
    fn a_static_aarch64_elf_has_no_interpreter() {
        assert!(
            is_static_aarch64_elf(&elf_with_header_type(1)),
            "PT_LOAD only: static"
        );
        assert!(
            !is_static_aarch64_elf(&elf_with_header_type(3)),
            "PT_INTERP: dynamic"
        );
        let mut x86 = elf_with_header_type(1);
        x86[18] = 0x3e;
        assert!(!is_static_aarch64_elf(&x86), "not aarch64");
        assert!(!is_static_aarch64_elf(&[0x7f, b'E']), "truncated");
        let mut none = elf_with_header_type(1);
        none[56] = 0;
        assert!(!is_static_aarch64_elf(&none), "no program headers");
        let mut far = elf_with_header_type(1);
        far[32..40].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(
            !is_static_aarch64_elf(&far),
            "program headers out of the file"
        );
        let mut obj = elf_with_header_type(1);
        obj[16] = 1;
        assert!(
            !is_static_aarch64_elf(&obj),
            "a relocatable object is no program"
        );
        assert!(
            !is_static_aarch64_elf(&elf_with_header_type(4)),
            "no PT_LOAD"
        );
        let mut elf32 = elf_with_header_type(1);
        elf32[4] = 1;
        assert!(!is_static_aarch64_elf(&elf32), "ELF32");
    }

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

    #[test]
    fn pins_are_read_and_checked() {
        let p =
            parse_pins("# c\napk-tools-static 3.0.9-r0 aa11\n\nalpine-keys 2.6-r0 bb22\n").unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[1].name, "alpine-keys");
        assert_eq!(p[1].version, "2.6-r0");
        assert!(parse_pins("one two\n").is_err(), "a pin has three words");
        assert!(check_pin(&p[0], "AA11").is_ok(), "case does not matter");
        assert!(check_pin(&p[0], "ff")
            .unwrap_err()
            .contains("apk-tools-static"));
    }

    #[test]
    fn the_database_gives_owner_and_mode_of_every_file() {
        let db = "P:busybox\nV:1\nF:bin\nR:busybox\na:0:0:755\nZ:Q1x\nF:bin\nR:bbsuid\na:0:0:4111\n\nP:x\nV:2\nF:etc/ssh\nM:0:0:700\nR:ssh_config\na:0:0:644\nF:var\nR:nothing\n";
        let e = database_files(db);
        assert!(e.contains(&("bin/busybox".into(), 0, 0, 0o755)));
        assert!(e.contains(&("bin/bbsuid".into(), 0, 0, 0o4111)));
        assert!(e.contains(&("etc/ssh/ssh_config".into(), 0, 0, 0o644)));
        assert!(
            e.contains(&("etc/ssh".into(), 0, 0, 0o700)),
            "a directory's mode is its M: line"
        );
        assert!(
            !e.iter().any(|f| f.0 == "var/nothing"),
            "no a: line, no mode to apply"
        );
    }

    #[test]
    fn modes_are_grouped_so_one_command_sets_many() {
        let f = vec![
            ("a".to_string(), 0, 0, 0o644),
            ("b".to_string(), 0, 0, 0o644),
            ("c".to_string(), 0, 5, 0o4111),
        ];
        let g = group_modes(&f);
        assert_eq!(g.len(), 2);
        assert_eq!(g[&(0, 0, 0o644)], vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn links_are_followed_to_where_they_end() {
        let mut t = Tree::default();
        t.add("bin", Entry::Dir);
        t.add("usr", Entry::Dir);
        t.add("usr/lib", Entry::Dir);
        t.add("usr/lib/libc.so", Entry::File);
        t.add("lib", Entry::Link("usr/lib".into()));
        t.add("bin/a", Entry::Link("../lib/libc.so".into()));
        t.add("bin/b", Entry::Link("/usr/lib/nothing".into()));
        t.add("bin/c", Entry::Link("/proc/self/mounts".into()));
        t.add("bin/loop", Entry::Link("loop".into()));
        assert_eq!(
            t.dangling_links(),
            vec![
                "bin/b -> /usr/lib/nothing".to_string(),
                "bin/loop -> loop".to_string()
            ]
        );
    }

    #[test]
    fn programs_without_an_execute_bit_are_found() {
        let files = vec![
            ("usr/sbin/sshd".to_string(), 0o755),
            ("usr/bin/broken".to_string(), 0o644),
            ("etc/init.d/sshd".to_string(), 0o644),
            ("etc/hostname".to_string(), 0o644),
            ("bin/bbsuid".to_string(), 0o4111),
        ];
        assert_eq!(
            not_executable(&files),
            vec!["usr/bin/broken".to_string(), "etc/init.d/sshd".to_string()]
        );
    }

    #[test]
    fn what_inittab_and_conf_d_start_is_listed() {
        let tab = "# c\n::sysinit:/sbin/openrc sysinit\ntty1::respawn:/sbin/getty 38400 tty1\n::ctrlaltdel:/sbin/reboot\n::once:\n";
        assert_eq!(
            inittab_programs(tab),
            vec!["/sbin/openrc", "/sbin/getty", "/sbin/reboot"]
        );
        let conf =
            "# c\nrc_need=\"usb-gadget net\"\nrc_use='logger'\ncommand=/usr/sbin/sshd\nrc_want=\n";
        assert_eq!(
            conf_d_started(conf),
            (
                vec!["usb-gadget".to_string(), "net".into(), "logger".into()],
                vec!["/usr/sbin/sshd".to_string()]
            )
        );
    }

    #[test]
    fn getty_must_respawn_until_the_gadget_binds() {
        let good = "ttyGS0::respawn:/sbin/getty -L 115200 ttyGS0 vt100\n";
        let tab = "tty1::respawn:/sbin/getty 38400 tty1\n";
        let script = "#!/bin/sh\ndmesg -n 3 || true\npkill -f '^/sbin/getty 38400 tty1$' || true\n";
        assert!(glass_prompt_respawn(tab, script, 0o755).is_ok());
        assert!(
            glass_prompt_respawn(tab, script, 0o644).is_err(),
            "not executable"
        );
        assert!(
            glass_prompt_respawn(
                tab,
                "#!/bin/sh\npkill -f '^/sbin/getty 38400 tty1$' || true\n",
                0o755
            )
            .is_err(),
            "no console loglevel line"
        );
        assert!(
            glass_prompt_respawn(
                tab,
                "#!/bin/sh\npkill -f '^/sbin/getty 38400 tty1$' || true\ndmesg -n 3 || true\n",
                0o755
            )
            .is_err(),
            "the loglevel comes after the pkill: the respawned prompt is printed before it"
        );
        assert!(
            glass_prompt_respawn(
                tab,
                "#!/bin/sh\ndmesg -n 7 || true\npkill -f '^/sbin/getty 38400 tty1$' || true\n",
                0o755
            )
            .is_err(),
            "a loglevel that still prints info lines"
        );
        assert!(
            glass_prompt_respawn(tab, "pkill -f getty\n", 0o755).is_err(),
            "the pattern is not the inittab's line"
        );
        assert!(
            glass_prompt_respawn(
                "ttyGS0::respawn:/sbin/getty -L 115200 ttyGS0\n",
                script,
                0o755
            )
            .is_err(),
            "no tty1 line"
        );
        assert!(getty_respawns(good, "ttyGS0").is_ok());
        assert!(
            getty_respawns("ttyGS0::once:/sbin/getty -L 115200 ttyGS0\n", "ttyGS0")
                .unwrap_err()
                .contains("respawn")
        );
        assert!(
            getty_respawns("tty1::respawn:/sbin/getty 38400 tty1\n", "ttyGS0")
                .unwrap_err()
                .contains("no line")
        );
        assert!(getty_respawns("ttyGS0::respawn:/bin/sh\n", "ttyGS0")
            .unwrap_err()
            .contains("getty"));
    }

    #[test]
    fn addresses_in_a_file_are_found_and_judged() {
        assert_eq!(
            addresses_in("a 172.16.43.1/24\nListenAddress 172.16.43.1\nno 1.2.3\n"),
            vec![
                (1, "172.16.43.1".to_string()),
                (2, "172.16.43.1".to_string())
            ]
        );
        assert!(address_problem("172.16.43.1", "172.16.43.1").is_none());
        assert!(
            address_problem("172.16.43.2", "172.16.43.1").is_none(),
            "the lease is in the same /24"
        );
        assert!(address_problem("172.16.42.1", "172.16.43.1")
            .unwrap()
            .contains("172.16.42.1"));
    }

    #[test]
    fn missing_host_tools_are_all_named() {
        let have = |t: &str| t == "curl" || t == "tar";
        assert_eq!(
            missing_tools(&["curl", "tar", "fakeroot", "mke2fs"], &have),
            vec!["fakeroot", "mke2fs"]
        );
        assert!(missing_tools(&["curl"], &have).is_empty());
    }

    #[test]
    fn the_stat_of_a_file_gives_its_mode() {
        let out = "Inode: 12  Type: regular    Mode:  4111   Flags: 0x80000\nGeneration: 0\n";
        assert_eq!(debugfs_mode(out), Some(0o4111));
        assert_eq!(debugfs_mode("nothing"), None);
    }

    #[test]
    fn debugfs_stat_blocks_give_path_mode_and_owner() {
        let out = "debugfs: stat /bin/bbsuid\nInode: 12   Type: regular    Mode:  4111   Flags: 0x80000\nGeneration: 0\nUser:     0   Group:     0   Project:     0   Size: 123\ndebugfs: stat /etc/x\nInode: 13   Type: regular    Mode:  0644   Flags: 0x80000\nUser:  100   Group:   101   Project:     0   Size: 1\n";
        assert_eq!(
            debugfs_stats(out),
            vec![
                ("bin/bbsuid".to_string(), 0o4111, 0, 0),
                ("etc/x".to_string(), 0o644, 100, 101)
            ]
        );
    }

    #[test]
    fn virtual_services_are_read_from_the_script() {
        assert_eq!(
            init_script_provides("depend() {\n\tneed localmount\n\tprovide logger\n}\n"),
            vec!["logger".to_string()]
        );
    }

    #[test]
    fn a_command_with_a_default_is_read() {
        assert_eq!(
            command_default("/sbin/syslogd").as_deref(),
            Some("/sbin/syslogd")
        );
        assert_eq!(
            command_default("${SSHD_BINARY:-\"/usr/sbin/sshd\"}").as_deref(),
            Some("/usr/sbin/sshd")
        );
        assert_eq!(command_default("${X}"), None);
        assert_eq!(command_default("$cmd"), None);
    }

    #[test]
    fn a_private_key_is_told_by_its_armor() {
        assert!(holds_private_key(
            "-----BEGIN OPENSSH PRIVATE KEY-----\nabc\n"
        ));
        assert!(holds_private_key("x\n-----BEGIN RSA PRIVATE KEY-----\n"));
        assert!(!holds_private_key("ssh-ed25519 AAAAC3Nza lab\n"));
        assert!(!holds_private_key("-----BEGIN PUBLIC KEY-----\n"));
        assert!(holds_private_key("  -----BEGIN OPENSSH PRIVATE KEY-----"));
        assert!(holds_private_key(
            "{\"k\":\"-----BEGIN EC PRIVATE KEY-----\\n\"}"
        ));
        assert!(holds_private_key_bytes(
            b"\xff\x00-----BEGIN PRIVATE KEY-----"
        ));
        assert!(!holds_private_key_bytes(b"\xff\x00 nothing"));
        // a program names the armor in its strings (ssh, sshd): not a key
        assert!(!holds_private_key_bytes(
            b"\x7fELF\x02\x01-----BEGIN PRIVATE KEY-----"
        ));
    }

    #[test]
    fn addresses_in_comments_are_not_read() {
        assert_eq!(
            addresses_in("# 172.16.43.1\nListenAddress 0.0.0.0 # 172.16.43.1\n"),
            vec![(2, "0.0.0.0".to_string())]
        );
    }
}
