// SPDX-License-Identifier: MIT OR Apache-2.0
//! The steps that make the texts the checks judge. Each prints the same form the 2026-11-26 captures have
//! (research walleye-tas2557-v3-builds): the tool's own output, then the exit line (`make_exit=N`, `build_exit=N`)
//! or the `key: value` read-back lines.

use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// What the run was asked for, every path resolved.
pub struct Plan {
    pub kernel: PathBuf,
    pub topic: String,
    pub base: String,
    pub base_dtb: Option<PathBuf>,
    pub overlay: PathBuf,
    pub work: PathBuf,
    pub image: String,
    pub cross_compile: String,
}

/// A command's stdout and stderr together, and its exit status (-1 when it did not start or was killed).
pub struct Ran {
    /// stdout alone: what a following step may read as data.
    pub stdout: String,
    pub text: String,
    pub status: i32,
}

pub fn run(program: &str, args: &[&str], stdin: Option<&[u8]>) -> Ran {
    let mut c = Command::new(program);
    c.args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match c.spawn() {
        Ok(ch) => ch,
        Err(e) => {
            return Ran {
                stdout: String::new(),
                text: format!("{program}: {e}\n"),
                status: -1,
            }
        }
    };
    // written from a thread: a large input must not wait on output nobody reads yet
    let feeder = match (stdin, child.stdin.take()) {
        (Some(bytes), Some(mut pipe)) => {
            let bytes = bytes.to_vec();
            Some(std::thread::spawn(move || pipe.write_all(&bytes)))
        }
        _ => None,
    };
    match child.wait_with_output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).into_owned();
            let mut text = stdout.clone();
            text.push_str(&String::from_utf8_lossy(&o.stderr));
            if let Some(Err(e)) = feeder.map(|f| {
                f.join()
                    .unwrap_or_else(|_| Err(std::io::Error::other("feeder panicked")))
            }) {
                text.push_str(&format!("{program}: stdin: {e}\n"));
            }
            Ran {
                stdout,
                text,
                status: o.status.code().unwrap_or(-1),
            }
        }
        Err(e) => Ran {
            stdout: String::new(),
            text: format!("{program}: {e}\n"),
            status: -1,
        },
    }
}

/// Whether a program is on PATH.
pub fn have(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|d| d.join(program).is_file()))
}

pub fn ref_exists(kernel: &Path, r: &str) -> bool {
    run(
        "git",
        &[
            "-C",
            &kernel.to_string_lossy(),
            "rev-parse",
            "--verify",
            "-q",
            &format!("{r}^{{commit}}"),
        ],
        None,
    )
    .status
        == 0
}

/// `git log --format='%H %s' BASE..TOPIC`, newest first.
pub fn commits(p: &Plan) -> Result<String, String> {
    let r = run(
        "git",
        &[
            "-C",
            &p.kernel.to_string_lossy(),
            "log",
            "--format=%H %s",
            &format!("{}..{}", p.base, p.topic),
        ],
        None,
    );
    if r.status != 0 {
        return Err(format!(
            "git log {}..{}: {}",
            p.base,
            p.topic,
            r.text.trim()
        ));
    }
    Ok(r.text)
}

/// A throwaway worktree of the topic, removed when it goes out of scope.
pub struct Tree {
    kernel: PathBuf,
    pub path: PathBuf,
}

impl Tree {
    pub fn of_topic(p: &Plan) -> Result<Tree, String> {
        let path = p.work.join("tree");
        let _ = run(
            "git",
            &[
                "-C",
                &p.kernel.to_string_lossy(),
                "worktree",
                "remove",
                "--force",
                &path.to_string_lossy(),
            ],
            None,
        );
        let r = run(
            "git",
            &[
                "-C",
                &p.kernel.to_string_lossy(),
                "worktree",
                "add",
                "-q",
                "--detach",
                &path.to_string_lossy(),
                &p.topic,
            ],
            None,
        );
        if r.status != 0 {
            return Err(format!(
                "cannot make a worktree of {}: {}",
                p.topic,
                r.text.trim()
            ));
        }
        Ok(Tree {
            kernel: p.kernel.clone(),
            path,
        })
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = run(
            "git",
            &[
                "-C",
                &self.kernel.to_string_lossy(),
                "worktree",
                "remove",
                "--force",
                &self.path.to_string_lossy(),
            ],
            None,
        );
    }
}

fn make(tree: &Path, out: &Path, cc: &str, extra: &[&str]) -> Ran {
    let t = tree.to_string_lossy();
    let o = format!("O={}", out.to_string_lossy());
    let c = format!("CROSS_COMPILE={cc}");
    let mut args = vec!["-C", &t, "ARCH=arm64", &c, &o];
    args.extend_from_slice(extra);
    run("make", &args, None)
}

/// defconfig plus the module symbol, then `make W=1 sound/soc/codecs/tas2557.o`; the text ends `build_exit=N`.
pub fn driver_build(p: &Plan, tree: &Tree) -> Result<String, String> {
    let out = p.work.join("out-drv");
    let r = make(&tree.path, &out, &p.cross_compile, &["-s", "defconfig"]);
    if r.status != 0 {
        return Err(format!("defconfig: {}", r.text.trim()));
    }
    let cfg = out.join(".config");
    let mut text = std::fs::read_to_string(&cfg).map_err(|e| format!("{}: {e}", cfg.display()))?;
    text.push_str("CONFIG_SND_SOC_TAS2557=m\n");
    std::fs::write(&cfg, text).map_err(|e| format!("{}: {e}", cfg.display()))?;
    let r = make(&tree.path, &out, &p.cross_compile, &["-s", "olddefconfig"]);
    if r.status != 0 {
        return Err(format!("olddefconfig: {}", r.text.trim()));
    }
    let kept = std::fs::read_to_string(&cfg).unwrap_or_default();
    if !kept.lines().any(|l| l == "CONFIG_SND_SOC_TAS2557=m") {
        return Err("the Kconfig symbol SND_SOC_TAS2557 did not take".into());
    }
    let r = make(
        &tree.path,
        &out,
        &p.cross_compile,
        &["W=1", "sound/soc/codecs/tas2557.o"],
    );
    Ok(format!("{}build_exit={}\n", r.text, r.status))
}

/// The binding's schema check in the pinned container; the text ends `make_exit=N`.
pub fn binding_check(p: &Plan, tree: &Tree) -> Result<String, String> {
    let out = p.work.join("out-dt");
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let owner = std::fs::metadata(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let ids = format!("{}:{}", owner.uid(), owner.gid());
    let inside = format!(
        "apk add -q make gcc musl-dev flex bison yamllint py3-dt-schema dtc && make O=/out ARCH=arm64 DT_SCHEMA_FILES=sound/ti,tas2557.yaml dt_binding_check; echo make_exit=$?; chown -R {ids} /out"
    );
    let r = run(
        "docker",
        &[
            "run",
            "--rm",
            "-v",
            &format!("{}:/src:ro", tree.path.display()),
            "-v",
            &format!("{}:/out", out.display()),
            "-w",
            "/src",
            &p.image,
            "sh",
            "-c",
            &inside,
        ],
        None,
    );
    if !r.text.lines().any(|l| l.starts_with("make_exit=")) {
        return Err(format!("the container did not finish: {}", r.text.trim()));
    }
    Ok(r.text)
}

fn get(dtb: &Path, node: &str, prop: &str, kind: Option<&str>) -> Result<String, String> {
    let d = dtb.to_string_lossy();
    let mut args = vec![];
    if let Some(k) = kind {
        args.extend_from_slice(&["-t", k]);
    }
    args.extend_from_slice(&[&d, node, prop]);
    let r = run("fdtget", &args, None);
    if r.status != 0 {
        return Err(format!("fdtget {node} {prop}: {}", r.text.trim()));
    }
    Ok(r.text.trim().to_string())
}

/// Preprocess and compile the overlay, merge it onto the base dtb, read the node back; the `key: value` text.
pub fn overlay_read(p: &Plan, tree: &Tree, base: &Path) -> Result<String, String> {
    let cpp = run(
        "cpp",
        &[
            "-nostdinc",
            "-I",
            &tree.path.join("include").to_string_lossy(),
            "-undef",
            "-x",
            "assembler-with-cpp",
            &p.overlay.to_string_lossy(),
        ],
        None,
    );
    if cpp.status != 0 {
        return Err(format!("cpp of the overlay: {}", cpp.text.trim()));
    }
    let dtbo = p.work.join("ov.dtbo");
    let dtc = run(
        "dtc",
        &[
            "-@",
            "-I",
            "dts",
            "-O",
            "dtb",
            "-o",
            &dtbo.to_string_lossy(),
            "-",
        ],
        Some(cpp.stdout.as_bytes()),
    );
    if dtc.status != 0 {
        return Err(format!("dtc of the overlay: {}", dtc.text.trim()));
    }
    let merged = p.work.join("merged.dtb");
    let m = run(
        "fdtoverlay",
        &[
            "-i",
            &base.to_string_lossy(),
            "-o",
            &merged.to_string_lossy(),
            &dtbo.to_string_lossy(),
        ],
        None,
    );
    if m.status != 0 {
        return Err(format!("fdtoverlay: {}", m.text.trim()));
    }
    let sum = run("sha256sum", &[&dtbo.to_string_lossy()], None);
    let sha = sum
        .text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    let bus = "/soc@0/i2c@c1b5000";
    let node = format!("{bus}/audio-codec@4c");
    let reset = get(&merged, &node, "reset-gpios", Some("u"))?;
    let phandle = reset
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    let label = get(&merged, "/__symbols__", "pm8998_gpios", None)?;
    let label_phandle = get(&merged, &label, "phandle", Some("u"))?;
    let who = if label_phandle == phandle {
        let name = label.rsplit('/').next().unwrap_or_default();
        format!(
            "{name} compatible {}",
            get(&merged, &label, "compatible", None)?
        )
    } else {
        format!("not pm8998_gpios (that is phandle {label_phandle})")
    };
    Ok(format!(
        "ov.dtbo sha256: {sha}\ni2c status: {}\ncompatible: {}\nreg: {}\nreset-gpios (phandle gpio flags): {reset}\nphandle {phandle} is: {who}\n",
        get(&merged, bus, "status", None)?,
        get(&merged, &node, "compatible", None)?,
        get(&merged, &node, "reg", Some("x"))?,
    ))
}

/// The topic sits on its base: the base is the topic's ancestor, and the oldest series commit's parent.
pub fn sits_on_base(p: &Plan) -> Result<(), String> {
    let k = p.kernel.to_string_lossy();
    let r = run(
        "git",
        &["-C", &k, "merge-base", "--is-ancestor", &p.base, &p.topic],
        None,
    );
    if r.status != 0 {
        return Err(format!("{} is not an ancestor of {}", p.base, p.topic));
    }
    let first = run(
        "git",
        &[
            "-C",
            &k,
            "rev-list",
            "--max-parents=1",
            "--reverse",
            &format!("{}..{}", p.base, p.topic),
        ],
        None,
    );
    let oldest = first.stdout.lines().next().unwrap_or_default().to_string();
    let parent = run("git", &["-C", &k, "rev-parse", &format!("{oldest}^")], None).stdout;
    let base = run(
        "git",
        &["-C", &k, "rev-parse", &format!("{}^{{commit}}", p.base)],
        None,
    )
    .stdout;
    if oldest.is_empty() || parent.trim() != base.trim() {
        return Err(format!("the series does not start on {} itself", p.base));
    }
    Ok(())
}
