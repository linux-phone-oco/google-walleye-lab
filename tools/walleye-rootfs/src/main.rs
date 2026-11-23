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
    let building = C::BUILD.get_from(&r).is_some();
    if let Err(e) = preflight(building) {
        eprintln!("walleye-rootfs: FAILED: {e}");
        std::process::exit(1);
    }
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

/// apk.static and the Alpine keys, the exact versions rootfs/pins names, each hashed before it is unpacked, into WORK/tools and WORK/keys-pkg.
/// One fetch serves both builds (and a later run: a download already on disk is used when its sha256 is the pinned one).
fn fetch_tools(ctx: &Ctx) -> Result<Tools, String> {
    let dl = ctx.work.join("dl");
    fs::create_dir_all(&dl).map_err(|e| e.to_string())?;
    let pins = parse_pins(
        &fs::read_to_string(ctx.repo.join("rootfs/pins"))
            .map_err(|e| format!("rootfs/pins: {e}"))?,
    )?;
    let base = format!("{}/v{}/main/x86_64", ctx.mirror, ctx.release);
    for (pkg, into) in [("apk-tools-static", "tools"), ("alpine-keys", "keys-pkg")] {
        let pin = pins
            .iter()
            .find(|p| p.name == pkg)
            .ok_or(format!("rootfs/pins has no line for {pkg}"))?;
        let apk = dl.join(format!("{pkg}-{}.apk", pin.version));
        if !apk.exists() || check_pin(pin, &sha256(&apk)?).is_err() {
            run(
                "curl",
                &[
                    "-sfL",
                    &format!("{base}/{pkg}-{}.apk", pin.version),
                    "-o",
                    path(&apk),
                ],
            )?;
        }
        check_pin(pin, &sha256(&apk)?)?;
        say(&format!(
            "pinned: {pkg}-{} sha256 {}",
            pin.version, pin.sha256
        ));
        let dir = ctx.work.join(into);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        run("tar", &["-xzf", path(&apk), "-C", path(&dir)])?;
    }
    Ok(Tools {
        apk: ctx.work.join("tools/sbin/apk.static"),
        keys: ctx.work.join("keys-pkg/usr/share/apk/keys"),
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
    apply_database_modes(tree, &state)?;
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
    match fs::symlink_metadata(dump.join("root/.ssh/authorized_keys")) {
        Ok(_) => {
            say("ok: root's authorized_keys is in the image");
            check_authorized_keys_modes(&mut rep, ctx, img)?;
        }
        Err(_) => say("not known yet: no authorized key was given (--authorized-key FILE), so ssh has no way in; the consoles do"),
    }
    rep.ok(
        !files.iter().any(|f| {
            fs::symlink_metadata(dump.join(f)).is_ok_and(|m| m.is_file())
                && read_scratch(&dump.join(f)).map_or(true, |b| holds_private_key_bytes(&b))
        }),
        "no private key is in the image (every file read whole; one that cannot be read fails)",
    );
    check_links_and_programs(&mut rep, &dump, &files)?;
    check_started_programs(&mut rep, &dump, &files)?;
    check_database_modes(&mut rep, &dump, img)?;
    check_addresses(&mut rep, ctx, &dump)?;
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

/// The host tools a run needs, found on PATH before anything is downloaded or built.
fn preflight(building: bool) -> Result<(), String> {
    let check_only = ["e2fsck", "debugfs", "sha256sum"];
    let build_too = [
        "curl", "tar", "fakeroot", "mke2fs", "find", "touch", "truncate", "cp", "chmod", "chown",
    ];
    let mut need: Vec<&str> = check_only.to_vec();
    if building {
        need.extend(build_too);
    }
    let dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    let have = |t: &str| {
        dirs.iter().any(|d| {
            fs::metadata(d.join(t))
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
    };
    let missing = missing_tools(&need, &have);
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "host tools missing from PATH: {}",
            missing.join(", ")
        ))
    }
}

/// apk's own record of every file's owner and mode (--usermode could not apply them: bbsuid came out 0111, not 4111) is applied
/// through fakeroot's state, one chown and one chmod per (owner, mode) group, the files given as arguments.
fn apply_database_modes(tree: &Path, state: &Path) -> Result<(), String> {
    let db = fs::read_to_string(tree.join("lib/apk/db/installed")).map_err(|e| e.to_string())?;
    let present: Vec<(String, u32, u32, u32)> = database_files(&db)
        .into_iter()
        .filter(|(p, ..)| {
            fs::symlink_metadata(tree.join(p)).is_ok_and(|m| !m.file_type().is_symlink())
        })
        .collect();
    let mut applied = 0;
    for ((uid, gid, mode), files) in group_modes(&present) {
        for chunk in files.chunks(200) {
            let abs: Vec<String> = chunk
                .iter()
                .map(|f| path(&tree.join(f)).to_string())
                .collect();
            for (cmd, first) in [
                ("chown", format!("{uid}:{gid}")),
                ("chmod", format!("{mode:o}")),
            ] {
                let mut args = vec!["-i", path(state), "-s", path(state), "--", cmd, "-h"];
                if cmd == "chmod" {
                    args.pop();
                }
                args.push(&first);
                args.extend(abs.iter().map(String::as_str));
                run("fakeroot", &args)?;
            }
            applied += chunk.len();
        }
    }
    say(&format!(
        "apk database: owner and mode applied to {applied} paths"
    ));
    Ok(())
}

/// The image's tree as the dump shows it, for the link and presence checks.
fn tree_of(dump: &Path, files: &[String]) -> Result<Tree, String> {
    let mut t = Tree::default();
    for f in files {
        let m = fs::symlink_metadata(dump.join(f)).map_err(|e| e.to_string())?;
        t.add(
            f,
            if m.file_type().is_symlink() {
                Entry::Link(
                    fs::read_link(dump.join(f))
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        .into_owned(),
                )
            } else if m.is_dir() {
                Entry::Dir
            } else {
                Entry::File
            },
        );
    }
    Ok(t)
}

fn check_links_and_programs(rep: &mut Report, dump: &Path, files: &[String]) -> Result<(), String> {
    let tree = tree_of(dump, files)?;
    let dangling = tree.dangling_links();
    let links = files
        .iter()
        .filter(|f| fs::symlink_metadata(dump.join(f)).is_ok_and(|m| m.file_type().is_symlink()))
        .count();
    rep.ok(
        links > 0 && dangling.is_empty(),
        &format!(
            "no symlink ends nowhere ({links} links of {} entries followed){}",
            files.len(),
            dangling.join("; ")
        ),
    );
    let mut modes = Vec::new();
    for f in files {
        let m = fs::symlink_metadata(dump.join(f)).map_err(|e| e.to_string())?;
        if m.is_file() {
            modes.push((f.clone(), m.permissions().mode() & 0o7777));
        }
    }
    let plain = not_executable(&modes);
    rep.ok(
        plain.is_empty(),
        &format!(
            "every file in a bin, sbin or init.d directory can be run{}",
            plain.join("; ")
        ),
    );
    Ok(())
}

/// What inittab and the conf.d and init.d files start must be in the image.
fn check_started_programs(rep: &mut Report, dump: &Path, files: &[String]) -> Result<(), String> {
    let tree = tree_of(dump, files)?;
    let inittab = fs::read_to_string(dump.join("etc/inittab")).unwrap_or_default();
    let missing: Vec<String> = inittab_programs(&inittab)
        .into_iter()
        .filter(|p| !tree.exists(p))
        .collect();
    rep.ok(
        missing.is_empty(),
        &format!(
            "every program inittab starts is in the image{}",
            missing.join("; ")
        ),
    );
    for tty in ["tty1", "ttyGS0"] {
        match getty_respawns(&inittab, tty) {
            Ok(()) => rep.ok(
                true,
                &format!("inittab: a getty on {tty} respawns until its tty is there"),
            ),
            Err(e) => rep.ok(false, &e),
        }
    }
    let script_file = "etc/local.d/50-glass-prompt.start";
    let mode = fs::metadata(dump.join(script_file))
        .map(|m| std::os::unix::fs::PermissionsExt::mode(&m.permissions()))
        .unwrap_or(0);
    match glass_prompt_respawn(&inittab, &fs::read_to_string(dump.join(script_file)).unwrap_or_default(), mode) {
        Ok(()) => rep.ok(true, "the glass prompt comes last: /etc/local.d/50-glass-prompt.start makes init respawn the tty1 getty"),
        Err(e) => rep.ok(false, &e),
    }
    rep.ok(
        files.iter().any(|f| f == "etc/runlevels/default/local"),
        "the `local` service is in the default runlevel (it runs the local.d scripts at the end)",
    );
    rep.ok(
        ["bin/pkill", "usr/bin/pkill"]
            .iter()
            .any(|p| tree.exists(p)),
        "pkill is in the image (the local.d script uses it)",
    );
    rep.ok(
        ["bin/dmesg", "usr/bin/dmesg"]
            .iter()
            .any(|p| tree.exists(p)),
        "dmesg is in the image (the local.d script lowers the console loglevel with it)",
    );
    let mut known: Vec<String> = Vec::new();
    let mut scripts = Vec::new();
    for f in files.iter().filter(|f| f.starts_with("etc/init.d/")) {
        known.push(file_name(Path::new(f)));
        let text = fs::read_to_string(dump.join(f)).unwrap_or_default();
        known.extend(init_script_provides(&text));
        scripts.push((file_name(Path::new(f)), text));
    }
    // only what a runlevel starts: init.d holds scripts of packages nothing links (runsvdir wants runit, installed or not)
    let started: Vec<String> = files
        .iter()
        .filter(|f| f.starts_with("etc/runlevels/") && f.matches('/').count() == 3)
        .map(|f| file_name(Path::new(f)))
        .collect();
    let (mut services, mut commands) = (Vec::new(), Vec::new());
    let mut script_commands: Vec<(String, Vec<String>)> = Vec::new();
    for f in files
        .iter()
        .filter(|f| f.starts_with("etc/conf.d/") && started.contains(&file_name(Path::new(f))))
    {
        let (s, c) = conf_d_started(&fs::read_to_string(dump.join(f)).unwrap_or_default());
        services.extend(s.into_iter().map(|x| (f.clone(), x)));
        commands.extend(c);
    }
    for (name, text) in &scripts {
        if started.contains(name) {
            // a script names its program once per variant (sshd, sshd.pam, sshd.krb5): one of them must be there
            let variants: Vec<String> = conf_d_started(text)
                .1
                .iter()
                .filter_map(|c| command_default(c))
                .collect();
            if !variants.is_empty() {
                script_commands.push((name.clone(), variants));
            }
        }
    }
    let unknown: Vec<String> = services
        .iter()
        .filter(|(_, s)| !known.contains(s))
        .map(|(f, s)| format!("{f} names {s}"))
        .collect();
    rep.ok(
        unknown.is_empty(),
        &format!(
            "every service a conf.d file needs, uses or wants has an init.d script ({} names){}",
            services.len(),
            unknown.join("; ")
        ),
    );
    let strict: Vec<String> = commands.iter().filter_map(|c| command_default(c)).collect();
    let mut gone: Vec<String> = strict.iter().filter(|c| !tree.exists(c)).cloned().collect();
    for (name, variants) in &script_commands {
        if !variants.iter().any(|v| tree.exists(v)) {
            gone.push(format!("{name}: none of {}", variants.join(", ")));
        }
    }
    let checked = strict.len() + script_commands.len();
    rep.ok(
        checked > 0 && gone.is_empty(),
        &format!(
            "every program a started service runs is in the image ({} conf.d commands, {} init.d scripts with a command=; start() bodies are not read); {} runlevel services{}",
            strict.len(),
            script_commands.len(),
            started.len(),
            gone.join("; ")
        ),
    );
    Ok(())
}

/// A file of the scratch dump, whole: the dump is ours, so a mode without owner read (bbsuid 4111) is lifted first.
fn read_scratch(file: &Path) -> std::io::Result<Vec<u8>> {
    let mode = fs::metadata(file)?.permissions().mode();
    fs::set_permissions(file, fs::Permissions::from_mode(mode | 0o400))?;
    fs::read(file)
}

/// root's .ssh is 700 and its authorized_keys 600, both root's (sshd's StrictModes refuses otherwise).
fn check_authorized_keys_modes(rep: &mut Report, ctx: &Ctx, img: &Path) -> Result<(), String> {
    let cmds = ctx.work.join("check-ssh.stat-cmds");
    fs::write(&cmds, "stat /root/.ssh\nstat /root/.ssh/authorized_keys\n")
        .map_err(|e| e.to_string())?;
    let seen = debugfs_stats(&run("debugfs", &["-f", path(&cmds), path(img)])?);
    let has = |p: &str, mode: u32| {
        seen.iter()
            .any(|s| s.0 == p && (s.1, s.2, s.3) == (mode, 0, 0))
    };
    rep.ok(
        has("root/.ssh", 0o700) && has("root/.ssh/authorized_keys", 0o600),
        "root's .ssh is 0:0 mode 700 and its authorized_keys 0:0 mode 600",
    );
    Ok(())
}

/// The modes and owners apk records, read back from the ext4 itself (debugfs stat): the files with a special mode or another owner.
fn check_database_modes(rep: &mut Report, dump: &Path, img: &Path) -> Result<(), String> {
    let db = fs::read_to_string(dump.join("lib/apk/db/installed")).unwrap_or_default();
    let special: Vec<(String, u32, u32, u32)> = database_files(&db)
        .into_iter()
        .filter(|(p, ..)| {
            fs::symlink_metadata(dump.join(p)).is_ok_and(|x| !x.file_type().is_symlink())
        })
        .collect();
    let cmds = dump.with_extension("stat-cmds");
    fs::write(
        &cmds,
        special
            .iter()
            .map(|(p, ..)| format!("stat /{p}\n"))
            .collect::<String>(),
    )
    .map_err(|e| e.to_string())?;
    let out = run("debugfs", &["-f", path(&cmds), path(img)])?;
    let seen = debugfs_stats(&out);
    let wrong: Vec<String> = special
        .iter()
        .filter(|(p, u, g, m)| {
            seen.iter()
                .find(|s| &s.0 == p)
                .is_none_or(|s| (s.1, s.2, s.3) != (*m, *u, *g))
        })
        .map(|(p, u, g, m)| format!("{p} should be {u}:{g} {m:o}"))
        .collect();
    rep.ok(
        !special.is_empty() && wrong.is_empty(),
        &format!(
            "apk's owner and mode stand in the image for {} files and directories (read back with debugfs stat){}",
            special.len(),
            wrong.join("; ")
        ),
    );
    let bbsuid = seen.iter().find(|s| s.0 == "bin/bbsuid");
    rep.ok(
        bbsuid.is_some_and(|s| (s.1, s.2, s.3) == (0o4111, 0, 0)),
        "bin/bbsuid is 0:0 mode 4111, as busybox-suid installs it",
    );
    Ok(())
}

/// Every lab address in the image's own files is device.toml's network, and the gadget unit and sshd have its address itself.
fn check_addresses(rep: &mut Report, ctx: &Ctx, dump: &Path) -> Result<(), String> {
    let address = device_address(&ctx.repo)?;
    let toml = fs::read_to_string(ctx.repo.join("device.toml")).map_err(|e| e.to_string())?;
    let first = toml
        .lines()
        .find(|l| l.starts_with("addresses = "))
        .and_then(|l| addresses_in(l).first().map(|a| a.1.clone()));
    rep.ok(
        first.as_deref() == Some(address.as_str()),
        &format!("device.toml: shell_address is addresses[0] ({address})"),
    );
    let places = [
        ("etc/init.d/usb-gadget", true),
        ("etc/ssh/sshd_config.d/10-lab.conf", true),
        ("etc/udhcpd.conf", false),
        ("etc/conf.d/sshd", false),
    ];
    for (file, must_be_the_address) in places {
        let read = fs::read_to_string(dump.join(file));
        let text_ok = read.is_ok();
        let text = read.unwrap_or_default();
        let found: Vec<(usize, String)> = addresses_in(&text)
            .into_iter()
            .filter(|(_, a)| !a.starts_with("255."))
            .collect();
        let bad: Vec<String> = found
            .iter()
            .filter_map(|(n, a)| address_problem(a, &address).map(|p| format!("{file}:{n}: {p}")))
            .collect();
        let has = found.iter().any(|(_, a)| *a == address);
        rep.ok(
            text_ok
                && bad.is_empty()
                && (has || !must_be_the_address)
                && (!found.is_empty() || file == "etc/conf.d/sshd"),
            &format!(
                "{file}: {} lab addresses, all in device.toml's {address} network{}{}",
                found.len(),
                if must_be_the_address && !has {
                    "; the address itself is missing"
                } else {
                    ""
                },
                bad.join("; ")
            ),
        );
    }
    Ok(())
}
