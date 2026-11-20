// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-rootfs: the Pixel 2's plain Alpine hardware-test rootfs (item walleye-install-rootfs-alpine-min).
//! No root, no qemu: apk.static installs into a directory (--usermode --no-scripts), the build writes what the skipped scripts
//! would have (OpenRC links, inittab, the usb gadget unit), mke2fs -d under fakeroot makes the ext4 image, built twice and compared.

use argenv_resolve::{parse, InputResolveExt};
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use walleye_rootfs::cli::{usage, Contract as C};
use walleye_rootfs::*;

const IMAGE_MIB: u64 = 1024;
const FILE_TIME: &str = "@1767225600"; // 2026-01-01, every file's time, so the image is the same twice
const FS_UUID: &str = "7e1c3a52-0b6d-4f0e-9a41-2d8a6c1e5b90";
const MUST_BE_INSTALLED: &[&str] = &[
    "alpine-base",
    "busybox",
    "openssh",
    "qrtr",
    "rmtfs",
    "tqftpserv",
    "e2fsprogs",
];
const NOT_STARTED: &[&str] = &["qrtr-ns", "qrtr", "rmtfs", "tqftpserv"];

struct Ctx {
    repo: PathBuf,
    work: PathBuf,
    out: PathBuf,
    release: String,
    mirror: String,
    key: Option<PathBuf>,
}

fn main() {
    let r = parse(&C::records());
    if C::HELP.get_from(&r).unwrap_or(false) {
        print!("{}", usage());
        return;
    }
    let work = PathBuf::from(C::WORK.get_from(&r).unwrap_or_else(|| "/tmp".into())).join("rootfs");
    let ctx = Ctx {
        repo: PathBuf::from(C::REPO.get_from(&r).unwrap_or_else(|| ".".into())),
        out: C::OUT
            .get_from(&r)
            .map(PathBuf::from)
            .unwrap_or_else(|| work.join("walleye-alpine.ext4")),
        work,
        release: C::RELEASE.get_from(&r).unwrap_or_else(|| "3.24".into()),
        mirror: C::MIRROR
            .get_from(&r)
            .unwrap_or_else(|| "https://dl-cdn.alpinelinux.org/alpine".into()),
        key: C::KEY.get_from(&r).map(PathBuf::from),
    };
    let result = match (C::BUILD.get_from(&r), C::CHECK.get_from(&r)) {
        (Some(w), None) if w == "now" => build_twice_and_check(&ctx),
        (None, Some(img)) => check_image(&ctx, Path::new(&img)),
        _ => Err(format!(
            "say what to do: --build now, or --check IMAGE\n{}",
            usage()
        )),
    };
    if let Err(e) = result {
        eprintln!("walleye-rootfs: FAILED: {e}");
        std::process::exit(1);
    }
}

fn build_twice_and_check(ctx: &Ctx) -> Result<(), String> {
    let tools = fetch_tools(ctx)?;
    let first = build_image(ctx, &tools, 1)?;
    let second = build_image(ctx, &tools, 2)?;
    let (a, b) = (sha256(&first)?, sha256(&second)?);
    say(&format!("sha256 of build 1: {a}\nsha256 of build 2: {b}"));
    if a != b {
        return Err("the two builds differ: the image is not reproducible".into());
    }
    // the image is checked where it was built; only a checked image gets its final name and its .sha256
    check_image(ctx, &first)?;
    fs::rename(&first, &ctx.out).map_err(|e| e.to_string())?;
    fs::write(
        sha_file(&ctx.out),
        format!("{a}  {}\n", file_name(&ctx.out)),
    )
    .map_err(|e| e.to_string())?;
    say(&format!("image: {} ({a})", ctx.out.display()));
    Ok(())
}

/// apk.static and the Alpine keys, from the mirror's main index, into WORK/tools and WORK/keys.
fn fetch_tools(ctx: &Ctx) -> Result<Tools, String> {
    let dl = ctx.work.join("dl");
    fs::create_dir_all(&dl).map_err(|e| e.to_string())?;
    let base = format!("{}/v{}/main/x86_64", ctx.mirror, ctx.release);
    let index_file = dl.join("APKINDEX.tar.gz");
    run(
        "curl",
        &[
            "-sfL",
            &format!("{base}/APKINDEX.tar.gz"),
            "-o",
            path(&index_file),
        ],
    )?;
    let index = run("tar", &["-xzOf", path(&index_file), "APKINDEX"])?;
    for (pkg, into) in [("apk-tools-static", "tools"), ("alpine-keys", "keys-pkg")] {
        let ver = index_version(&index, pkg).ok_or(format!("{pkg} is not in the index"))?;
        let apk = dl.join(format!("{pkg}-{ver}.apk"));
        run(
            "curl",
            &["-sfL", &format!("{base}/{pkg}-{ver}.apk"), "-o", path(&apk)],
        )?;
        let dir = ctx.work.join(into);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        run("tar", &["-xzf", path(&apk), "-C", path(&dir)])?;
    }
    let keys = ctx.work.join("keys-pkg/usr/share/apk/keys");
    Ok(Tools {
        apk: ctx.work.join("tools/sbin/apk.static"),
        keys,
    })
}

struct Tools {
    apk: PathBuf,
    keys: PathBuf,
}

/// One whole build: the tree, its configuration, the image. Returns the image's path.
fn build_image(ctx: &Ctx, tools: &Tools, n: u32) -> Result<PathBuf, String> {
    let tree = ctx.work.join(format!("tree-{n}"));
    let img = ctx.work.join(format!("build-{n}.ext4"));
    say(&format!(
        "build {n}: install the packages into {}",
        tree.display()
    ));
    install_packages(ctx, tools, &tree)?;
    say(&format!("build {n}: write the configuration"));
    write_configuration(ctx, &tree)?;
    say(&format!("build {n}: make the ext4 image"));
    make_image(ctx, &tree, &img)?;
    Ok(img)
}

fn install_packages(ctx: &Ctx, tools: &Tools, tree: &Path) -> Result<(), String> {
    let _ = fs::remove_dir_all(tree);
    fs::create_dir_all(tree).map_err(|e| e.to_string())?;
    let (main, community) = (
        format!("{}/v{}/main", ctx.mirror, ctx.release),
        format!("{}/v{}/community", ctx.mirror, ctx.release),
    );
    let packages =
        fs::read_to_string(ctx.repo.join("rootfs/packages")).map_err(|e| e.to_string())?;
    let wanted: Vec<&str> = packages
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let mut args = vec![
        "--usermode",
        "--root",
        path(tree),
        "--keys-dir",
        path(&tools.keys),
        "--initdb",
        "--no-scripts",
        "--no-cache",
        "--no-interactive",
        "--arch",
        "aarch64",
        "-X",
        &main,
        "-X",
        &community,
        "add",
    ];
    args.extend(wanted);
    run(path(&tools.apk), &args)?;
    Ok(())
}

/// What the skipped package scripts and a first setup would write: files from rootfs/overlay, the OpenRC links, root's login.
fn write_configuration(ctx: &Ctx, tree: &Path) -> Result<(), String> {
    run(
        "cp",
        &[
            "-r",
            "--no-preserve=ownership,timestamps",
            &format!("{}/.", ctx.repo.join("rootfs/overlay").display()),
            path(tree),
        ],
    )?;
    fs::write(
        tree.join("etc/apk/repositories"),
        format!(
            "{m}/v{r}/main\n{m}/v{r}/community\n",
            m = ctx.mirror,
            r = ctx.release
        ),
    )
    .map_err(|e| e.to_string())?;
    link_busybox_applets(tree)?;
    // apk's log names the build's own directory and times: a build record, not part of the system
    let _ = fs::remove_file(tree.join("var/log/apk.log"));
    let levels =
        fs::read_to_string(ctx.repo.join("rootfs/runlevels")).map_err(|e| e.to_string())?;
    for (level, svc) in runlevel_lines(&levels) {
        let init = tree.join("etc/init.d").join(&svc);
        if !init.exists() {
            return Err(format!(
                "runlevel {level}: /etc/init.d/{svc} is not in the image"
            ));
        }
        let dir = tree.join("etc/runlevels").join(&level);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        symlink(format!("/etc/init.d/{svc}"), dir.join(&svc))
            .map_err(|e| format!("{level}/{svc}: {e}"))?;
    }
    // a hardware-test layer: root has no password on the two consoles (getty); sshd takes a key only (10-lab.conf)
    let shadow = tree.join("etc/shadow");
    let text = fs::read_to_string(&shadow).map_err(|e| e.to_string())?;
    fs::write(&shadow, text.replacen("root:*:", "root::", 1)).map_err(|e| e.to_string())?;
    if let Some(key) = &ctx.key {
        let dir = tree.join("root/.ssh");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        fs::copy(key, dir.join("authorized_keys")).map_err(|e| e.to_string())?;
        run("chmod", &["700", path(&dir)])?;
        run("chmod", &["600", path(&dir.join("authorized_keys"))])?;
    }
    Ok(())
}

/// ext4 from the directory, owned by root (fakeroot), every time fixed: the same bytes twice.
fn make_image(ctx: &Ctx, tree: &Path, img: &Path) -> Result<(), String> {
    let state = ctx.work.join("fakeroot.state");
    let _ = fs::remove_file(img);
    run(
        "find",
        &[
            path(tree),
            "-exec",
            "touch",
            "-h",
            "-d",
            FILE_TIME,
            "{}",
            "+",
        ],
    )?;
    run(
        "fakeroot",
        &[
            "-s",
            path(&state),
            "--",
            "chown",
            "-R",
            "-h",
            "0:0",
            path(tree),
        ],
    )?;
    keep_modes_of_unreadable_files(tree, &state)?;
    // a tar carries owner, mode and mtime and no ctime: mke2fs copies from it, the same bytes every time
    let tarball = PathBuf::from(format!("{}.tar", tree.display()));
    run(
        "fakeroot",
        &[
            "-i",
            path(&state),
            "--",
            "tar",
            "-C",
            path(tree),
            "--sort=name",
            "--numeric-owner",
            "--format=gnu",
            "--mtime=@1767225600",
            "-cf",
            path(&tarball),
            ".",
        ],
    )?;
    run("truncate", &["-s", &format!("{IMAGE_MIB}M"), path(img)])?;
    let hash_seed = format!("hash_seed={FS_UUID}");
    run_with_env(
        "mke2fs",
        &[
            "-q",
            "-F",
            "-t",
            "ext4",
            "-O",
            "^has_journal",
            "-L",
            "walleye-alpine",
            "-U",
            FS_UUID,
            "-E",
            &hash_seed,
            "-d",
            path(&tarball),
            path(img),
        ],
        ("E2FSPROGS_FAKE_TIME", "1767225600"),
    )?;
    Ok(())
}

/// A file apk made unreadable to its owner (bbsuid) cannot be copied by tar or mke2fs: it is made readable for real and fakeroot
/// is told the mode the file had, as apk --usermode installed it (apk's own database says 4111 for bbsuid; not applied: noted in the research row).
fn keep_modes_of_unreadable_files(tree: &Path, state: &Path) -> Result<(), String> {
    for rel in walk(tree, tree)? {
        let file = tree.join(&rel);
        let meta = fs::symlink_metadata(&file).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() || meta.permissions().mode() & 0o400 != 0 {
            continue;
        }
        let mode = meta.permissions().mode() & 0o7777;
        fs::set_permissions(&file, fs::Permissions::from_mode(mode | 0o600))
            .map_err(|e| e.to_string())?;
        run(
            "fakeroot",
            &[
                "-i",
                path(state),
                "-s",
                path(state),
                "--",
                "chmod",
                &format!("{mode:o}"),
                path(&file),
            ],
        )?;
    }
    Ok(())
}

/// The item's proof, read from the image itself: every line says what was counted; any failure stops the run.
fn check_image(ctx: &Ctx, img: &Path) -> Result<(), String> {
    let mut rep = Report::default();
    let size = fs::metadata(img)
        .map_err(|e| format!("{}: {e}", img.display()))?
        .len();
    rep.ok(
        size <= MAX_IMAGE_BYTES,
        &format!("the image is {} MiB sparse, under 2.5 GiB", size >> 20),
    );
    rep.ok(
        run("e2fsck", &["-fn", path(img)]).is_ok(),
        "e2fsck: the ext4 is clean",
    );
    let dump = ctx.work.join("check-dump");
    let _ = fs::remove_dir_all(&dump);
    fs::create_dir_all(&dump).map_err(|e| e.to_string())?;
    run(
        "debugfs",
        &["-R", &format!("rdump / {}", dump.display()), path(img)],
    )?;
    let files = walk(&dump, &dump)?;
    say(&format!("the image holds {} entries", files.len()));
    let release = fs::read_to_string(dump.join("etc/alpine-release")).unwrap_or_default();
    rep.ok(
        release.starts_with(&ctx.release),
        &format!(
            "Alpine {} (alpine-release: {})",
            ctx.release,
            release.trim()
        ),
    );
    rep.ok(
        fs::read_to_string(dump.join("etc/apk/arch"))
            .unwrap_or_default()
            .trim()
            == "aarch64",
        "apk arch is aarch64",
    );
    let installed = fs::read_to_string(dump.join("lib/apk/db/installed")).unwrap_or_default();
    let pkgs = installed_packages(&installed);
    rep.ok(
        !pkgs.is_empty(),
        &format!("the apk database lists {} packages", pkgs.len()),
    );
    for want in MUST_BE_INSTALLED {
        rep.ok(
            pkgs.iter().any(|(n, _)| n == want),
            &format!("package {want} is installed"),
        );
    }
    for (n, _) in &pkgs {
        if let Some(why) = forbidden_package(n) {
            rep.ok(false, &why);
        }
    }
    let bad: Vec<String> = files.iter().filter_map(|f| forbidden_path(f)).collect();
    rep.ok(
        bad.is_empty(),
        &format!(
            "no file of the phone's firmware or modem data, no phone UI ({} entries checked){}",
            files.len(),
            bad.join("; ")
        ),
    );
    let mut head = [0u8; 20];
    let busybox = fs::File::open(dump.join("bin/busybox"))
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut head));
    rep.ok(
        busybox.is_ok() && is_aarch64_elf(&head),
        "bin/busybox is an aarch64 ELF",
    );
    let inittab = fs::read_to_string(dump.join("etc/inittab")).unwrap_or_default();
    for tty in ["tty0", "ttyGS0"] {
        rep.ok(
            inittab
                .lines()
                .any(|l| l.starts_with(&format!("{tty}::respawn:/sbin/getty"))),
            &format!("inittab: getty on {tty}"),
        );
    }
    for exe in [
        "sbin/openrc",
        "usr/sbin/sshd",
        "sbin/getty",
        "sbin/apk",
        "bin/sh",
        "bin/login",
        "usr/sbin/udhcpd",
        "sbin/ip",
    ] {
        rep.ok(
            fs::symlink_metadata(dump.join(exe)).is_ok(),
            &format!("{exe} is there"),
        );
    }
    let levels =
        fs::read_to_string(ctx.repo.join("rootfs/runlevels")).map_err(|e| e.to_string())?;
    for (level, svc) in runlevel_lines(&levels) {
        rep.ok(
            fs::symlink_metadata(dump.join("etc/runlevels").join(&level).join(&svc)).is_ok(),
            &format!("OpenRC {level}: {svc}"),
        );
    }
    let started: Vec<&String> = files
        .iter()
        .filter(|f| {
            f.starts_with("etc/runlevels/")
                && NOT_STARTED.iter().any(|s| f.ends_with(&format!("/{s}")))
        })
        .collect();
    rep.ok(
        started.is_empty(),
        "qrtr, rmtfs and tqftpserv are installed and linked into no runlevel",
    );
    let gadget = fs::read_to_string(dump.join("etc/init.d/usb-gadget")).unwrap_or_default();
    let address = device_address(&ctx.repo)?;
    rep.ok(
        gadget.contains(&format!("{address}/24")),
        &format!("the usb gadget unit uses device.toml's address {address}"),
    );
    rep.ok(
        fs::read_to_string(dump.join("etc/shadow"))
            .unwrap_or_default()
            .lines()
            .any(|l| l.starts_with("root::")),
        "root logs in on the consoles without a password",
    );
    match fs::metadata(dump.join("root/.ssh/authorized_keys")) {
        Ok(_) => say("ok: root's authorized_keys is in the image"),
        Err(_) => say("not known yet: no authorized key was given (--authorized-key FILE), so ssh has no way in; the consoles do"),
    }
    let sum = sha256(img)?;
    say(&format!("sha256 {sum}"));
    if let Ok(recorded) = fs::read_to_string(sha_file(img)) {
        rep.ok(
            recorded.split_whitespace().next() == Some(sum.as_str()),
            "the image matches its .sha256 file",
        );
    }
    say("NOT in v1: pd-mapper (items walleye-install-rootfs-pd-mapper), lpoco-health (walleye-install-rootfs-health-binary)");
    fs::write(ctx.work.join("packages.txt"), package_list(&pkgs)).map_err(|e| e.to_string())?;
    say(&format!(
        "package list: {}",
        ctx.work.join("packages.txt").display()
    ));
    rep.end()
}

#[derive(Default)]
struct Report {
    failed: Vec<String>,
}

impl Report {
    fn ok(&mut self, good: bool, what: &str) {
        say(&format!("{}: {what}", if good { "ok" } else { "FAIL" }));
        if !good {
            self.failed.push(what.to_string());
        }
    }
    fn end(self) -> Result<(), String> {
        if self.failed.is_empty() {
            Ok(())
        } else {
            Err(format!("{} check(s) failed", self.failed.len()))
        }
    }
}

fn device_address(repo: &Path) -> Result<String, String> {
    let toml = fs::read_to_string(repo.join("device.toml")).map_err(|e| e.to_string())?;
    toml.lines()
        .find_map(|l| {
            l.strip_prefix("shell_address = \"")
                .and_then(|r| r.split('"').next())
                .map(str::to_string)
        })
        .ok_or_else(|| "device.toml has no shell_address".into())
}

/// Every entry under `dir`, relative to `base`, symlinks not followed.
fn walk(base: &Path, dir: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = e.map_err(|e| e.to_string())?.path();
        out.push(
            p.strip_prefix(base)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .into_owned(),
        );
        if fs::symlink_metadata(&p)
            .map_err(|e| e.to_string())?
            .is_dir()
        {
            out.extend(walk(base, &p)?);
        }
    }
    Ok(out)
}

fn sha256(file: &Path) -> Result<String, String> {
    Ok(run("sha256sum", &[path(file)])?
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string())
}

fn sha_file(img: &Path) -> PathBuf {
    PathBuf::from(format!("{}.sha256", img.display()))
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn path(p: &Path) -> &str {
    p.to_str().unwrap_or_default()
}

fn say(line: &str) {
    println!("walleye-rootfs: {line}");
}

fn run(cmd: &str, args: &[&str]) -> Result<String, String> {
    let o = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;
    if !o.status.success() {
        return Err(format!(
            "{cmd} {}: {}\n{}",
            args.join(" "),
            o.status,
            String::from_utf8_lossy(&o.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

fn run_with_env(cmd: &str, args: &[&str], env: (&str, &str)) -> Result<String, String> {
    let o = Command::new(cmd)
        .args(args)
        .env(env.0, env.1)
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;
    if !o.status.success() {
        return Err(format!(
            "{cmd}: {}\n{}",
            o.status,
            String::from_utf8_lossy(&o.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

/// busybox's post-install (skipped) makes one link per applet, from the package's own lists: busybox's, and busybox-extras' (udhcpd).
fn link_busybox_applets(tree: &Path) -> Result<(), String> {
    for (list, binary) in [
        ("busybox", "/bin/busybox"),
        ("busybox-extras", "/bin/busybox-extras"),
    ] {
        let file = tree.join("etc/busybox-paths.d").join(list);
        let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        for rel in busybox_links(&text) {
            let link = tree.join(&rel);
            if fs::symlink_metadata(&link).is_ok() {
                continue;
            }
            fs::create_dir_all(link.parent().ok_or("a link without a directory")?)
                .map_err(|e| e.to_string())?;
            symlink(binary, &link).map_err(|e| format!("{rel}: {e}"))?;
        }
    }
    Ok(())
}
