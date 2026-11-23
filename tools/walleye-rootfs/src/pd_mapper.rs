// SPDX-License-Identifier: MIT OR Apache-2.0
//! pd-mapper for the image: no Alpine package has it, so it is compiled static for aarch64 from three pinned source archives
//! (pd-mapper, qrtr's libqrtr, xz's liblzma) with aarch64-linux-gnu-gcc, once per run, before the two image builds.

use super::*;

const CROSS: &str = "aarch64-linux-gnu";
const SOURCES: [(&str, &str); 3] = [
    (
        "pd-mapper",
        "https://github.com/linux-msm/pd-mapper/archive/{v}.tar.gz",
    ),
    (
        "qrtr",
        "https://github.com/linux-msm/qrtr/archive/{v}.tar.gz",
    ),
    (
        "xz",
        "https://github.com/tukaani-project/xz/releases/download/v{v}/xz-{v}.tar.gz",
    ),
];

/// Build pd-mapper into WORK/pd-mapper/pd-mapper (stripped) and return its path.
pub fn build(ctx: &Ctx, pins: &[Pin]) -> Result<PathBuf, String> {
    let root = ctx.work.join("pd-mapper");
    fs::create_dir_all(ctx.work.join("dl")).map_err(|e| e.to_string())?;
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    for (name, url) in SOURCES {
        let pin = pins
            .iter()
            .find(|p| p.name == name)
            .ok_or(format!("rootfs/pins has no line for {name}"))?;
        let archive = ctx
            .work
            .join("dl")
            .join(format!("{name}-{}.tar.gz", pin.version));
        if !archive.exists() || check_pin(pin, &sha256(&archive)?).is_err() {
            run(
                "curl",
                &[
                    "-sfL",
                    &url.replace("{v}", &pin.version),
                    "-o",
                    path(&archive),
                ],
            )?;
        }
        check_pin(pin, &sha256(&archive)?)?;
        say(&format!(
            "pinned: {name} {} sha256 {}",
            pin.version, pin.sha256
        ));
        let dir = root.join(name);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        run(
            "tar",
            &[
                "-xzf",
                path(&archive),
                "-C",
                path(&dir),
                "--strip-components=1",
            ],
        )?;
    }
    let sysroot = root.join("sysroot");
    let cflags = format!("-O2 -ffile-prefix-map={}=.", root.display());
    let cc = format!("{CROSS}-gcc");
    say("pd-mapper: liblzma from xz");
    let xz_build = root.join("xz-build");
    fs::create_dir_all(&xz_build).map_err(|e| e.to_string())?;
    run_in(
        &xz_build,
        path(&root.join("xz/configure")),
        &[
            "-q",
            &format!("--host={CROSS}"),
            &format!("--prefix={}", sysroot.display()),
            "--disable-shared",
            "--enable-static",
            "--disable-xz",
            "--disable-xzdec",
            "--disable-lzmadec",
            "--disable-lzmainfo",
            "--disable-scripts",
            "--disable-doc",
            "--disable-nls",
            &format!("CC={cc}"),
            &format!("CFLAGS={cflags}"),
        ],
    )?;
    run("make", &["-C", path(&xz_build), "-s", "-j8", "install"])?;
    say("pd-mapper: libqrtr from qrtr");
    let qrtr = root.join("qrtr");
    let mut objects = Vec::new();
    for c in ["qrtr", "logging", "qmi"] {
        let obj = root.join(format!("{c}.o"));
        run(
            &cc,
            &[
                "-O2",
                &format!("-ffile-prefix-map={}=.", root.display()),
                &format!("-I{}", qrtr.join("include").display()),
                "-c",
                path(&qrtr.join(format!("lib/{c}.c"))),
                "-o",
                path(&obj),
            ],
        )?;
        objects.push(obj);
    }
    let libqrtr = sysroot.join("lib/libqrtr.a");
    let mut ar = vec!["rcsD", path(&libqrtr)];
    ar.extend(objects.iter().map(|o| path(o)));
    run(&format!("{CROSS}-ar"), &ar)?;
    fs::create_dir_all(sysroot.join("include")).map_err(|e| e.to_string())?;
    fs::copy(
        qrtr.join("include/libqrtr.h"),
        sysroot.join("include/libqrtr.h"),
    )
    .map_err(|e| e.to_string())?;
    say("pd-mapper: pd-mapper itself, static");
    let src = root.join("pd-mapper");
    run(
        "make",
        &[
            "-C",
            path(&src),
            "-s",
            &format!("CC={cc}"),
            &format!(
                "CFLAGS=-Wall {cflags} -I{}",
                sysroot.join("include").display()
            ),
            &format!(
                "LDFLAGS=-static -Wl,--build-id=none -L{} -lqrtr -llzma",
                sysroot.join("lib").display()
            ),
        ],
    )?;
    let out = root.join("pd-mapper-stripped");
    fs::copy(src.join("pd-mapper"), &out).map_err(|e| e.to_string())?;
    run(&format!("{CROSS}-strip"), &["--strip-all", path(&out)])?;
    Ok(out)
}
