// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-ramroot: the Pixel 2's Alpine-in-RAM boot image B-a5 (item walleye-display-alpine-ramroot-image).
//! B-r5's kernel, dtb, command line, header words, size and 13 display modules as they are; a new ramdisk whose init waits 5 s, loads the modules, unpacks /userland.tar
//! (the userland of the ext4 image written to system_a, read out of that very image by debugfs under fakeroot) into a tmpfs and switches root to Alpine's /sbin/init.
//! Built twice and compared, then checked by what is in the image (every line says what was counted; any failure stops the run and writes nothing).

use argenv_resolve::{parse, InputResolveExt};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use walleye_ramroot::cli::{usage, Contract as C};
use walleye_ramroot::*;

const BASE_IMAGE: &str = "b-r5";
const ROOTFS_IMAGE: &str = "alpine-rootfs";
const IMAGE_NAME: &str = "boot-b-a5.img";
const SUMS_NAME: &str = "SHA256SUMS-display-ramroot";
const DELAY_SECONDS: &str = "5";
const MODULES: usize = 13;
/// Every applet build/initramfs-ramroot/init and its petter run (the init says them; a missing one would end the run or drop the petter).
const INIT_APPLETS: &[&str] = &[
    "mount",
    "cut",
    "sleep",
    "mkdir",
    "tar",
    "rm",
    "ls",
    "tr",
    "readlink",
    "cp",
    "cat",
    "chmod",
    "chroot",
    "setsid",
    "sync",
    "reboot",
    "uname",
    "switch_root",
    "sh",
];

struct Ctx {
    repo: PathBuf,
    work: PathBuf,
    images: PathBuf,
    out: PathBuf,
}

/// The two images the new one is made from, found by device.toml and read only when their sha256 is the recorded one.
struct Inputs {
    base: PathBuf,
    rootfs: PathBuf,
}

/// What is counted: one line per check, and every failure remembered.
#[derive(Default)]
struct Report {
    failed: Vec<String>,
}

impl Report {
    fn ck(&mut self, what: &str, got: impl AsRef<str>, want: impl AsRef<str>) {
        if got.as_ref() == want.as_ref() {
            say(&format!("ok   {what}"));
        } else {
            let line = format!(
                "FAIL {what}: got '{}', want '{}'",
                got.as_ref(),
                want.as_ref()
            );
            say(&line);
            self.failed.push(line);
        }
    }
    fn done(self, what: &str) -> Result<(), String> {
        if self.failed.is_empty() {
            Ok(())
        } else {
            Err(format!("{what}: {} check(s) failed", self.failed.len()))
        }
    }
}

fn main() {
    let r = parse(&C::records());
    if C::HELP.get_from(&r).unwrap_or(false) {
        print!("{}", usage());
        return;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let images = PathBuf::from(
        C::IMAGES
            .get_from(&r)
            .unwrap_or_else(|| format!("{home}/Projects/walleye-build")),
    );
    let ctx = Ctx {
        repo: fs::canonicalize(C::REPO.get_from(&r).unwrap_or_else(|| ".".into())).unwrap_or_else(
            |e| {
                eprintln!("walleye-ramroot: FAILED: --repo: {e}");
                std::process::exit(1)
            },
        ),
        work: PathBuf::from(C::WORK.get_from(&r).unwrap_or_else(|| "/tmp".into())).join("ramroot"),
        out: C::OUT
            .get_from(&r)
            .map(PathBuf::from)
            .unwrap_or_else(|| images.join("images-display-ramroot")),
        images,
    };
    if let Err(e) = preflight() {
        eprintln!("walleye-ramroot: FAILED: {e}");
        std::process::exit(1);
    }
    let result = match (C::BUILD.get_from(&r), C::CHECK.get_from(&r)) {
        (Some(w), None) if w == "now" => build_twice_and_check(&ctx),
        (None, Some(img)) => inputs(&ctx).and_then(|i| check_image(&ctx, &i, Path::new(&img))),
        _ => Err(format!(
            "say what to do: --build now, or --check IMAGE\n{}",
            usage()
        )),
    };
    if let Err(e) = result {
        eprintln!("walleye-ramroot: FAILED: {e}");
        std::process::exit(1);
    }
}

fn build_twice_and_check(ctx: &Ctx) -> Result<(), String> {
    let inp = inputs(ctx)?;
    let first = build_image(ctx, &inp, 1)?;
    let second = build_image(ctx, &inp, 2)?;
    let (a, b) = (sha256(&first)?, sha256(&second)?);
    say(&format!("sha256 of build 1: {a}\nsha256 of build 2: {b}"));
    if a != b {
        return Err("the two builds differ: the image is not reproducible".into());
    }
    check_image(ctx, &inp, &first)?;
    fs::create_dir_all(&ctx.out).map_err(|e| e.to_string())?;
    let (img, sums) = (ctx.out.join(IMAGE_NAME), ctx.out.join(SUMS_NAME));
    let _ = fs::remove_file(&img);
    fs::copy(&first, &img).map_err(|e| e.to_string())?;
    fs::write(&sums, format!("{a}  {IMAGE_NAME}\n")).map_err(|e| e.to_string())?;
    say(&format!("image: {} ({a})", img.display()));
    Ok(())
}

/// The base image and the rootfs image, each by device.toml's path, each with the recorded sha256.
fn inputs(ctx: &Ctx) -> Result<Inputs, String> {
    let toml = fs::read_to_string(ctx.repo.join("device.toml"))
        .map_err(|e| format!("{}: {e}", ctx.repo.join("device.toml").display()))?;
    let mut found = Vec::new();
    for name in [BASE_IMAGE, ROOTFS_IMAGE] {
        let (rel, want) = image_entry(&toml, name)?;
        let file = ctx.images.join(rel);
        let got = sha256(&file)?;
        if got != want {
            return Err(format!(
                "{}: sha256 {got}, device.toml [images.{name}] says {want}",
                file.display()
            ));
        }
        say(&format!(
            "input [images.{name}] {} sha256 {got}",
            file.display()
        ));
        found.push(file);
    }
    Ok(Inputs {
        rootfs: found.remove(1),
        base: found.remove(0),
    })
}

/// One whole build: the userland tar, the ramdisk, the image. Returns the image's path.
fn build_image(ctx: &Ctx, inp: &Inputs, n: u32) -> Result<PathBuf, String> {
    let dir = ctx.work.join(format!("build-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    say(&format!(
        "build {n}: the userland out of {}",
        inp.rootfs.display()
    ));
    let tar = userland_tar(&inp.rootfs, &dir)?;
    say(&format!("build {n}: unpack the base image"));
    let base = unpack_image(&inp.base, &dir.join("base"))?;
    let base_rd = unpack_ramdisk(&base.join("ramdisk"), &dir.join("base-ramdisk"))?;
    say(&format!("build {n}: pack the ramdisk"));
    let ramdisk = pack_ramdisk(ctx, &base_rd, &tar, &dir)?;
    say(&format!("build {n}: swap the ramdisk into the base image"));
    let img = dir.join(IMAGE_NAME);
    run(
        "lpoco-bringup",
        &[
            "image",
            "swap",
            path(&inp.base),
            "--ramdisk",
            path(&ramdisk),
            "--out",
            path(&img),
        ],
    )?;
    Ok(img)
}

/// The userland of an ext4 image as a tar: debugfs rdump under fakeroot (owner 0 0 as in the image), the setuid, setgid and sticky modes debugfs' rdump drops put back from the inodes, tar with every time fixed.
fn userland_tar(ext4: &Path, dir: &Path) -> Result<PathBuf, String> {
    let (tree, state, tar) = (
        dir.join("tree"),
        dir.join("fakeroot.state"),
        dir.join("userland.tar"),
    );
    fs::create_dir_all(&tree).map_err(|e| e.to_string())?;
    run(
        "fakeroot",
        &[
            "-s",
            path(&state),
            "--",
            "debugfs",
            "-R",
            &format!("rdump / {}", tree.display()),
            path(ext4),
        ],
    )?;
    let names = walk(&tree, &tree)?;
    if let Some(odd) = names.iter().find(|n| n.contains(char::is_whitespace)) {
        return Err(format!(
            "a path with a blank in it cannot be stat'ed by command file: {odd}"
        ));
    }
    let cmds = dir.join("stat.cmds");
    let body: String = names.iter().map(|n| format!("stat /{n}\n")).collect();
    fs::write(&cmds, body).map_err(|e| e.to_string())?;
    let stats = run("debugfs", &["-f", path(&cmds), path(ext4)])?;
    let special = special_modes(&stats);
    for (p, mode) in &special {
        let target = tree.join(p.trim_start_matches('/'));
        run(
            "fakeroot",
            &[
                "-i",
                path(&state),
                "-s",
                path(&state),
                "--",
                "chmod",
                &format!("{mode:o}"),
                path(&target),
            ],
        )?;
    }
    say(&format!(
        "the userland: {} paths, {} with setuid, setgid or sticky bits put back",
        names.len(),
        special.len()
    ));
    run(
        "fakeroot",
        &[
            "-i",
            path(&state),
            "--",
            "tar",
            "-C",
            path(&tree),
            "--sort=name",
            "--numeric-owner",
            "--format=gnu",
            &format!("--mtime={FILE_TIME}"),
            "-cf",
            path(&tar),
            ".",
        ],
    )?;
    Ok(tar)
}

fn unpack_image(img: &Path, into: &Path) -> Result<PathBuf, String> {
    let _ = fs::remove_dir_all(into);
    run(
        "lpoco-bringup",
        &["image", "unpack", path(img), "--out", path(into)],
    )?;
    Ok(into.to_path_buf())
}

/// The ramdisk's files in a folder (xz, then cpio, owner and modes as the archive has them where a user may).
fn unpack_ramdisk(ramdisk: &Path, into: &Path) -> Result<PathBuf, String> {
    let _ = fs::remove_dir_all(into);
    fs::create_dir_all(into).map_err(|e| e.to_string())?;
    let cpio = Command::new("xz")
        .args(["-dc", path(ramdisk)])
        .output()
        .map_err(|e| format!("xz: {e}"))?;
    if !cpio.status.success() {
        return Err(format!("xz -dc {}: {}", ramdisk.display(), cpio.status));
    }
    run_in_stdin(
        "cpio",
        &["-idm", "--quiet", "-f", "dev/*"],
        into,
        &cpio.stdout,
    )?;
    Ok(into.to_path_buf())
}

/// The new ramdisk: the base's module tree, module list, delay file and busybox, the new init and the userland tar.
fn pack_ramdisk(ctx: &Ctx, base_rd: &Path, tar: &Path, dir: &Path) -> Result<PathBuf, String> {
    let mut entries: Vec<(String, Kind)> = Vec::new();
    for d in ["bin", "sbin", "etc", "dev", "proc", "sys", "tmp", "run"] {
        entries.push((d.to_string(), Kind::Dir));
    }
    for rel in walk(base_rd, &base_rd.join("lib"))? {
        let p = base_rd.join(&rel);
        let m = fs::symlink_metadata(&p).map_err(|e| e.to_string())?;
        let kind = if m.is_dir() {
            Kind::Dir
        } else if m.is_file() {
            Kind::File { executable: false }
        } else {
            return Err(format!("{rel}: a module tree holds files and folders only"));
        };
        entries.push((rel, kind));
    }
    entries.push(("lib".into(), Kind::Dir));
    entries.push(("bin/busybox".into(), Kind::File { executable: true }));
    entries.push(("bin/sh".into(), Kind::Link("busybox".into())));
    entries.push((
        "display-modules.list".into(),
        Kind::File { executable: false },
    ));
    entries.push(("display-delay".into(), Kind::File { executable: false }));
    let mut spec = spec_lines(&entries, path(base_rd));
    spec.push_str("nod /dev/console 600 0 0 c 5 1\nnod /dev/null 666 0 0 c 1 3\n");
    spec.push_str(&format!(
        "file /init {} 755 0 0\nfile /userland.tar {} 644 0 0\n",
        ctx.repo.join("build/initramfs-ramroot/init").display(),
        tar.display()
    ));
    let (spec_file, out) = (dir.join("ramdisk.spec"), dir.join("ramdisk.xz"));
    fs::write(&spec_file, spec).map_err(|e| e.to_string())?;
    run(
        "lpoco-bringup",
        &[
            "initramfs",
            "build",
            "--spec",
            path(&spec_file),
            "--root",
            path(base_rd),
            "--codec",
            "xz",
            "--out",
            path(&out),
        ],
    )?;
    Ok(out)
}

/// The item's proof, read from the image itself: every line says what was counted; any failure stops the run.
fn check_image(ctx: &Ctx, inp: &Inputs, img: &Path) -> Result<(), String> {
    let mut rep = Report::default();
    let chk = ctx.work.join("check");
    let _ = fs::remove_dir_all(&chk);
    fs::create_dir_all(&chk).map_err(|e| e.to_string())?;
    let size = fs::metadata(img)
        .map_err(|e| format!("{}: {e}", img.display()))?
        .len();
    let base_size = fs::metadata(&inp.base).map_err(|e| e.to_string())?.len();
    say(&format!(
        "size: {size} bytes; the boot partition is {BOOT_PARTITION_BYTES}, fastboot takes {FASTBOOT_MAX_DOWNLOAD_BYTES} in one download"
    ));
    rep.ck(
        "the image is the size of B-r5 and fits the boot partition",
        format!("{} {}", size == base_size, size <= BOOT_PARTITION_BYTES),
        "true true",
    );
    let (u, bu) = (
        unpack_image(img, &chk.join("u"))?,
        unpack_image(&inp.base, &chk.join("b"))?,
    );
    let hdr = |d: &Path| -> Result<String, String> {
        Ok(fs::read_to_string(d.join("boot.toml"))
            .map_err(|e| e.to_string())?
            .lines()
            .filter(|l| {
                !l.starts_with('#') && !l.contains("ramdisk_size") && !l.contains("ramdisk size")
            })
            .collect::<Vec<_>>()
            .join("\n"))
    };
    rep.ck(
        "the command line and header words are B-r5's (the ramdisk's size apart)",
        (hdr(&u)? == hdr(&bu)?).to_string(),
        "true",
    );
    rep.ck(
        "the kernel (Image.gz and dtb) is B-r5's, byte for byte",
        same(&u.join("kernel"), &bu.join("kernel"))?,
        "same",
    );
    rep.ck(
        "the dtb appended to the kernel is B-r5's (mdss reset)",
        same(&u.join("appended.dtb"), &bu.join("appended.dtb"))?,
        "same",
    );
    rep.ck(
        "the ramdisk differs from B-r5's",
        same(&u.join("ramdisk"), &bu.join("ramdisk"))?,
        "differs",
    );
    fs::create_dir_all(chk.join("v")).map_err(|e| e.to_string())?;
    let copy = chk.join("v/boot.img"); // the gate removes a failing image: it gets a copy, named as avbtool and the other builders expect
    fs::copy(img, &copy).map_err(|e| e.to_string())?;
    rep.ck(
        "the AVB footer verifies",
        yes(Command::new("avbtool")
            .args(["verify_image", "--image", path(&copy)])
            .output()),
        "yes",
    );
    rep.ck(
        "the UFS-off gate (build/ufs-gate.sh) passes the image",
        yes(Command::new("sh")
            .arg(ctx.repo.join("build/ufs-gate.sh"))
            .arg(&copy)
            .output()),
        "yes",
    );
    let off = run_any(
        "sh",
        &[
            path(&ctx.repo.join("tests/host/ufs-off-image.sh")),
            path(img),
        ],
    )?;
    rep.ck(
        "the UFS-off tests pass (tests/host/ufs-off-image.sh)",
        first_line(&off).split(' ').next().unwrap_or(""),
        "pass",
    );
    let kernel_size = fs::metadata(u.join("kernel"))
        .map_err(|e| e.to_string())?
        .len();
    let room = headroom(0x8000, kernel_size, 0x100_0000);
    rep.ck(&format!("the kernel ends before the ramdisk loads (headroom {room} bytes > {MIN_HEADROOM_BYTES})"), (room > MIN_HEADROOM_BYTES as i64).to_string(), "true");
    let rd_size = fs::metadata(u.join("ramdisk"))
        .map_err(|e| e.to_string())?
        .len();
    say(&format!("the ramdisk is {rd_size} bytes and the kernel {kernel_size}: {} bytes of the boot partition are left for the header and the AVB footer and spare", BOOT_PARTITION_BYTES.saturating_sub(kernel_size + rd_size)));
    // the ramdisk
    let rd = unpack_ramdisk(&u.join("ramdisk"), &chk.join("rd"))?;
    let brd = unpack_ramdisk(&bu.join("ramdisk"), &chk.join("brd"))?;
    let init_src = ctx.repo.join("build/initramfs-ramroot/init");
    rep.ck(
        "the ramdisk's /init is build/initramfs-ramroot/init, byte for byte",
        same(&rd.join("init"), &init_src)?,
        "same",
    );
    let init_text = fs::read_to_string(&init_src).map_err(|e| e.to_string())?;
    rep.ck(
        "the init mounts no block device and has no mkfs, fsck, dd, trim, ext4 or remount",
        init_forbidden_words(&init_text).join(","),
        "",
    );
    rep.ck(
        "the init says the tag, unpacks the tar, and switches root",
        [
            INIT_TAG,
            "tar -xf /userland.tar",
            "exec switch_root /newroot /sbin/init",
        ]
        .iter()
        .filter(|w| init_text.contains(**w))
        .count()
        .to_string(),
        "3",
    );
    let delay = fs::read_to_string(rd.join("display-delay")).map_err(|e| e.to_string())?;
    rep.ck("the display delay is 5 s", delay.trim(), DELAY_SECONDS);
    rep.ck(
        "busybox and the module list are B-r5's, byte for byte",
        format!(
            "{} {}",
            same(&rd.join("bin/busybox"), &brd.join("bin/busybox"))?,
            same(
                &rd.join("display-modules.list"),
                &brd.join("display-modules.list")
            )?
        ),
        "same same",
    );
    let bb = fs::read(rd.join("bin/busybox")).map_err(|e| e.to_string())?;
    rep.ck(
        "busybox is an aarch64 executable with every applet the init and its petter run",
        format!(
            "{} {}",
            is_aarch64_elf(&bb),
            INIT_APPLETS.iter().all(|a| has_applet(&bb, a))
        ),
        "true true",
    );
    // the display modules, for THIS kernel
    let list = fs::read_to_string(rd.join("display-modules.list")).map_err(|e| e.to_string())?;
    let mods = module_paths(&list)?;
    rep.ck(
        "the module list names 13 modules",
        mods.len().to_string(),
        MODULES.to_string(),
    );
    let kernel_gz = chk.join("Image.gz");
    let dtb_size = fs::metadata(u.join("appended.dtb"))
        .map_err(|e| e.to_string())?
        .len();
    let blob = fs::read(u.join("kernel")).map_err(|e| e.to_string())?;
    fs::write(&kernel_gz, &blob[..blob.len() - dtb_size as usize]).map_err(|e| e.to_string())?;
    let banner = banner_release(&run("gzip", &["-dc", path(&kernel_gz)])?).unwrap_or_default();
    let mut vermagics = Vec::new();
    let mut missing = 0;
    for m in &mods {
        let f = rd.join(m.trim_start_matches('/'));
        if !f.exists() {
            missing += 1;
            continue;
        }
        let vm = run("modinfo", &["-F", "vermagic", path(&f)])?;
        vermagics.push(vermagic_release(&vm).unwrap_or_default());
    }
    rep.ck(
        "every module of the list is in the ramdisk",
        missing.to_string(),
        "0",
    );
    rep.ck(
        &format!("every module's vermagic is the kernel's release ({banner})"),
        vermagics
            .iter()
            .filter(|v| **v == banner && !banner.is_empty())
            .count()
            .to_string(),
        mods.len().to_string(),
    );
    rep.ck(
        "the module folder is named by the kernel's release",
        yes_if(rd.join("lib/modules").join(&banner).is_dir()),
        "yes",
    );
    // the userland
    let again = userland_tar(&inp.rootfs, &chk.join("again"))?;
    rep.ck("the ramdisk's /userland.tar is the system_a image's userland, byte for byte (made again from the ext4 image)", same(&rd.join("userland.tar"), &again)?, "same");
    let listing = run("tar", &["-tf", path(&rd.join("userland.tar"))])?;
    let paths = tar_paths(&listing);
    let lacking: Vec<&str> = MUST_BE_IN_THE_USERLAND
        .iter()
        .copied()
        .filter(|p| !paths.iter().any(|q| q == p))
        .collect();
    rep.ck("the userland has the init, OpenRC, getty, sshd, the gadget unit, the busybox and root's authorized_keys", lacking.join(","), "");
    let detail = run("tar", &["-tvf", path(&rd.join("userland.tar"))])?;
    let line_of = |p: &str| {
        detail
            .lines()
            .find(|l| l.ends_with(&format!(" ./{p}")))
            .unwrap_or("")
            .to_string()
    };
    rep.ck(
        "root's authorized_keys is mode 600 and no private key is in the userland",
        format!(
            "{} {}",
            line_of("root/.ssh/authorized_keys").starts_with("-rw-------"),
            paths.iter().any(|p| p.ends_with("id_ed25519")
                || p.ends_with("id_rsa")
                || p.contains("walleye-lab"))
        ),
        "true false",
    );
    rep.ck(
        "bbsuid keeps its setuid bit",
        line_of("bin/bbsuid").starts_with("---s--x--x").to_string(),
        "true",
    );
    let inittab = run(
        "tar",
        &["-xOf", path(&rd.join("userland.tar")), "./etc/inittab"],
    )?;
    rep.ck(
        "the inittab respawns a getty on tty0 (the glass) and on ttyGS0",
        inittab_problem(&inittab).unwrap_or_default(),
        "",
    );
    let forbidden: Vec<&String> = paths
        .iter()
        .filter(|p| p.starts_with("lib/firmware/") && p.len() > 13)
        .collect();
    rep.ck(
        "the userland holds no firmware",
        forbidden.len().to_string(),
        "0",
    );
    rep.done("the image")?;
    say("every check passed");
    Ok(())
}

/// Busybox keeps its applet names NUL-delimited in one table: `\0name\0` is the applet, not any string that holds the word.
fn has_applet(bb: &[u8], name: &str) -> bool {
    let mut needle = vec![0u8];
    needle.extend_from_slice(name.as_bytes());
    needle.push(0);
    bb.windows(needle.len()).any(|w| w == needle.as_slice())
}

fn is_aarch64_elf(head: &[u8]) -> bool {
    head.len() >= 20 && head[..4] == [0x7f, b'E', b'L', b'F'] && head[18] == 0xb7 && head[19] == 0
}

fn yes_if(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

fn yes(r: std::io::Result<std::process::Output>) -> &'static str {
    yes_if(r.is_ok_and(|o| o.status.success()))
}

fn same(a: &Path, b: &Path) -> Result<&'static str, String> {
    let (x, y) = (
        fs::read(a).map_err(|e| format!("{}: {e}", a.display()))?,
        fs::read(b).map_err(|e| format!("{}: {e}", b.display()))?,
    );
    Ok(if x == y { "same" } else { "differs" })
}

/// Every entry under `dir`, relative to `base`, symlinks not followed.
fn walk(base: &Path, dir: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    entries.sort();
    for p in entries {
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

fn path(p: &Path) -> &str {
    p.to_str().unwrap_or_default()
}

fn say(line: &str) {
    println!("walleye-ramroot: {line}");
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

/// A tool whose exit status carries a verdict the output also says (a test that prints `fail ...`): its output, whatever the status.
fn run_any(cmd: &str, args: &[&str]) -> Result<String, String> {
    let o = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

fn run_in_stdin(cmd: &str, args: &[&str], dir: &Path, input: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut child = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("{cmd}: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("no stdin")?
        .write_all(input)
        .map_err(|e| e.to_string())?;
    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd}: {status}"))
    }
}

/// The host tools a run needs, found on PATH before anything is built.
fn preflight() -> Result<(), String> {
    let need = [
        "debugfs",
        "fakeroot",
        "tar",
        "xz",
        "cpio",
        "gzip",
        "modinfo",
        "sha256sum",
        "avbtool",
        "lpoco-bringup",
        "sh",
    ];
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
