// SPDX-License-Identifier: MIT OR Apache-2.0
//! The modem boot (item walleye-modem-firmware-push-procedure, for the try walleye-try-modem-mss; decision 2026-11-27-walleye-modem-firmware-is-pushed-not-packed): the modem image
//! ([images.modem]: UFS off by the gate, the static rmtfs, qrtr-ns and tqftpserv, copies of the modem's storage, NO firmware) is booted ONCE from fastboot, from memory (nothing
//! flashed, no slot touched). Over the image's USB network (172.16.43.1) the lab then
//!   1. starts `firmware-recv` on the phone's telnet and pushes the firmware tar to its port ([`PUSH_PORT`]) - in Rust, with no `nc` on the host: the tar is 61 MB, it does not fit the boot image;
//!   2. waits for `firmware-recv: done` (the phone unpacked it), then runs `modem-up` (which checks the tar's own SHA256SUMS before it starts qrtr-ns, tqftpserv and rmtfs);
//!      Then (item walleye-modem-mm-push-in-procedure) it pushes the ModemManager rootfs tar to port 9001 (`rootfs-recv`, sha256 checked first), then `mm-up` runs dbus and ModemManager chrooted; its log is in the read of step 3;
//!   3. reads the remoteproc states, the kernel's modem lines, the daemons' logs and `qrtr-lookup`, and keeps them in the run's folder.
//!   4. asks `qmicli -d qrtr://0 --dms-get-ids` (read only), and keeps the answer, which holds the IMEI, only in the run's folder under ~/lab-material (item walleye-modem-qmicli-in-image).
//!
//! Whatever the modem did is the answer. The whole must end inside the init's self-reboot window ([`INIT_WINDOW_S`]); each step says before it starts whether the time left covers it.
//!
//! Where each judged word comes from (no text of the modem running was captured yet: the first real run is the capture):
//!  - the remoteproc's name is the device's name, `4080000.remoteproc`: qcom_q6v5_mss.c line 2091 `devm_rproc_alloc(&pdev->dev, pdev->name, ...)` and msm8998.dtsi `remoteproc_mss: remoteproc@4080000` (Linux 7.2.2 tree of ~/Projects/walleye-build/porthole);
//!  - `auto_boot = false` (qcom_q6v5_mss.c line 2098): nothing starts at boot, so state `offline` before `modem-up` is the expected one;
//!  - `powering up NAME` and `remote processor NAME is now up`: remoteproc_core.c lines 1935 and 1308;
//!  - the words `firmware-recv: done` and `modem-up: qrtr-ns, tqftpserv and rmtfs run`: the scripts build/initramfs-modem/firmware-recv and modem-up, whose text the image's ramdisk holds;
//!  - the port 9000 and the folder: the same firmware-recv script.

use crate::host;
use crate::lab::Lab;
use crate::rootfs_boot::RootfsBoot;
use crate::settings::Settings;
use crate::slpi_boot::{log_says_for, remoteprocs_in, section, services_in, Log, SlpiBoot, State};
use std::cell::Cell;
use std::io::Write;
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::rc::Rc;
use std::time::{Duration, Instant};

type Done = Result<(), String>;

/// The files of the run's folder; an earlier run of the day's are forgotten at the start (a stale answer is never this run's).
pub const STALE: [&str; 10] = [
    "read.txt",
    "dmesg.txt",
    "daemon-logs.txt",
    "push.txt",
    "modem-up.txt",
    "modem-up-check.txt",
    "verdict.txt",
    "qmi-ids.txt",
    "rootfs-push.txt",
    "mm-up.txt",
];
/// The modem's remoteproc: its device's name (see the sources above).
pub const MSS_NAME: &str = "4080000.remoteproc";
/// The port `firmware-recv` listens on (build/initramfs-modem/firmware-recv: `nc -l -p 9000`).
pub const PUSH_PORT: u16 = 9000;
/// The receivers' logs on the phone, written once: the commands below, the pushes and the read all name these.
pub const FIRMWARE_LOG: &str = "/tmp/firmware-recv.log";
pub const ROOTFS_LOG: &str = "/tmp/rootfs-recv.log";
/// The command that starts a receiver on the phone, its words and its log kept in the log named.
pub fn recv_command(receiver: &str, log: &str) -> String {
    format!("{receiver} > {log} 2>&1")
}
/// What the receiver prints when the set is unpacked, and what `modem-up` prints when the three daemons run.
pub const RECV_DONE: &str = "firmware-recv: done";
pub const UP_DONE: &str = "modem-up: qrtr-ns, tqftpserv and rmtfs run";
/// The port `rootfs-recv` listens on (build/initramfs-modem/rootfs-recv: `MM_PORT` defaults to 9001), the command that starts it, and the words it and `mm-up` end on (the scripts' own).
pub const ROOTFS_PORT: u16 = 9001;
pub const ROOTFS_DONE: &str = "rootfs-recv: done";
pub const MM_DONE: &str = "mm-up: dbus and ModemManager run";
/// The init's self-reboot, in seconds from the watchdog's open (build/initramfs-slpi/init: 90 beats of 2 s). Every step must end before it.
pub const INIT_WINDOW_S: u64 = 180;
/// Seconds the three daemons are given after `modem-up` before the read (rmtfs writes `start` to the remoteproc once it is ready).
pub const SETTLE_S: u64 = 8;
/// Seconds each phase may need from its start (its own bounds are longer; these are the usual costs), so a step can say whether it and all after it fit.
pub const UP_S: u64 = 20 + SETTLE_S;
pub const READ_S: u64 = 30;
/// Seconds `push_file` waits for the receiver's port to answer: the phone starts the receiver in the background just before, and `nc -l` listens within a second or two; 20 s is ten times that and well inside the init's 180 s.
pub const CONNECT_WAIT_S: u64 = 20;
/// Seconds after the last byte that the receiver may take to unpack and say its done words: the firmware set (61 MB, 31 files) said it inside this on the last real run, and the rootfs tar is only unpacked the same way; the rootfs's own time is the next try's to read.
pub const RECEIVER_SAYS_S: u64 = 60;
/// The USB network's throughput, an ASSUMPTION the next try checks: the last real run pushed the firmware tar (61501440 bytes) in under 25 s (research walleye-modem-mm-scripts-fit-the-image), so over 2.4 MB/s; the rootfs push has not been measured.
pub const PUSH_BYTES_PER_S: u64 = 2_400_000;
/// `mm-up` (build/initramfs-modem/mm-up): its bus wait is 10 s, then 1 s; the step is given 20.
pub const MM_UP_S: u64 = 20;
/// The usual costs the FIRMWARE step's guard counts (the bounds above sum past the window and would refuse every boot): from the last real run (modem-boot-20261127-124252, files in ~/lab-material), the firmware's done words to the read took 17 s (push.txt 12:43:48, read.txt 12:44:05) and the ids read under 1 s; `mm-up` 11 s from its script (bus wait 10 s, then 1 s).
pub const AFTER_FIRMWARE_USUAL_S: u64 = 17;
pub const MM_UP_USUAL_S: u64 = 11;
pub const ROOTFS_READ_USUAL_S: u64 = 2;

/// Seconds the ids read may need (it is cut at `IDS_TIMEOUT_S` by the phone's own `timeout`).
pub const IDS_S: u64 = 20;
pub const IDS_TIMEOUT_S: u64 = 15;
/// Seconds after the TERM that busybox `timeout -k` waits before it KILLs qmicli, so the shell always answers inside `IDS_S`.
pub const IDS_KILL_S: u64 = 2;
/// The read-only DMS question over QRTR (libqmi src/qmicli/qmicli.c: `-d, --device` takes "device path or QRTR URI (e.g. qrtr://0)", `-p` is the proxy flag; qmicli-dms.c `dms-get-ids`:
/// prints "Device IDs retrieved:" then ESN, IMEI and MEID lines). Its output holds the phone's IMEI: it is kept ONLY in the run's folder under ~/lab-material (qmi-ids.txt), never printed, logged or committed.
pub const IDS_COMMAND: &str = "timeout -k 2 15 qmicli -d qrtr://0 --dms-get-ids 2>&1";

/// The one read over telnet: every remoteproc's name and state, the kernel's modem lines, the daemons' logs, qrtr-lookup's list; marks between the parts.
pub const READ_COMMAND: &str = "echo ---remoteproc; for d in /sys/class/remoteproc/*; do [ -e $d/name ] || continue; echo \"$(cat $d/name) $(cat $d/state)\"; done; echo ---dmesg; dmesg | grep -i -e remoteproc -e remote.processor -e q6v5 -e mss -e rmtfs -e qrtr -e glink -e sysmon -e modem; echo ---logs; for f in /tmp/*.log; do echo \"== $f\"; cat $f; done; echo ---qrtr; qrtr-lookup 2>&1; echo ---end";

/// What a modem boot read, whole.
#[derive(Debug, PartialEq)]
pub struct Answer {
    /// the modem's remoteproc state, or none when no remoteproc has its name
    pub mss: Option<State>,
    pub others: Vec<(String, State)>,
    pub log: Log,
    pub services: Result<usize, String>,
}

/// The answer of a telnet read. Only a read that has not got its parts is an error.
pub fn judge(text: &str) -> Result<Answer, String> {
    let all = remoteprocs_in(text)?;
    let log = log_says_for(
        MSS_NAME,
        &section(text, "dmesg")
            .ok_or("the read has no dmesg part")?
            .join("\n"),
    );
    let services = services_in(text);
    let (mss, others): (Vec<_>, Vec<_>) = all.into_iter().partition(|(n, _)| n == MSS_NAME);
    Ok(Answer {
        mss: mss.into_iter().next().map(|(_, s)| s),
        others,
        log,
        services,
    })
}

impl Answer {
    /// The line the research row records: what was counted, not why.
    pub fn said(&self) -> String {
        let state = match &self.mss {
            Some(State::Running) => "remoteproc modem is running".to_string(),
            Some(State::Offline) => "remoteproc modem is offline".to_string(),
            Some(State::Crashed) => "remoteproc modem CRASHED".to_string(),
            Some(State::Other(w)) => format!("remoteproc modem is {w}"),
            None => format!(
                "no remoteproc is named {MSS_NAME} (the ones listed: {})",
                self.others
                    .iter()
                    .map(|(n, _)| n.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        };
        let log = match (self.log.started, self.log.up) {
            (_, true) => "the kernel says the modem processor is now up".to_string(),
            (true, false) => {
                "the kernel says it powered the modem up and never that it is up".to_string()
            }
            (false, false) => "the kernel does not say it started the modem".to_string(),
        };
        let failures = if self.log.failures.is_empty() {
            "no failure line".to_string()
        } else {
            format!(
                "{} failure line(s), the first: {}",
                self.log.failures.len(),
                self.log.failures[0]
            )
        };
        let services = match &self.services {
            Ok(0) => "qrtr-lookup lists no service".to_string(),
            Ok(n) => format!("qrtr-lookup lists {n} service(s)"),
            Err(e) => format!("not known yet: {e}"),
        };
        format!("{state}; {log}; {failures}; {services}")
    }
}

/// The kept reads hold no device id: touch::redact's serial, and every run of 14 to 16 digits (an IMEI, which ModemManager's debug log may print) is masked.
pub fn redact_ids(text: &str) -> String {
    let text = crate::touch::redact(text);
    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        out.push_str(if (14..=16).contains(&run.len()) {
            "<id masked>"
        } else {
            run
        });
        run.clear();
    };
    for c in text.chars() {
        if c.is_ascii_digit() {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    out
}

/// What the ids read says WITHOUT the ids: whether qmicli answered, never a value (the IMEI stays in the kept file). Source of the words: qmicli-dms.c prints
/// "Device IDs retrieved:" on an answer; every error of qmicli starts `error:`.
pub fn ids_say(text: &str) -> String {
    let lines = text.lines().filter(|l| !l.trim().is_empty()).count();
    if text.contains("Device IDs retrieved:") {
        format!("qmicli answered the DMS get-ids ({lines} lines, kept in qmi-ids.txt, not shown)")
    } else if text.lines().any(|l| l.trim_start().starts_with("error:")) {
        format!("qmicli said an error ({lines} lines, kept in qmi-ids.txt, not shown)")
    } else {
        format!("qmicli did not answer ({lines} lines, kept in qmi-ids.txt, not shown)")
    }
}

/// The ids answer is written to `qmi-ids.txt` in the run's folder (mode 700, under ~/lab-material), the file itself mode 600, and the sentence that is printed says its shape only.
/// This is the one place the answer is written: nothing else prints or saves it.
pub fn keep_the_ids(out: &std::path::Path, text: &str) -> Result<String, String> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let _ = std::fs::set_permissions(out, std::fs::Permissions::from_mode(0o700));
    let file = out.join("qmi-ids.txt");
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&file)
        .map_err(|e| format!("{}: {e}", file.display()))?;
    std::io::Write::write_all(&mut f, format!("{text}\n").as_bytes())
        .map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(ids_say(text))
}

/// The receiver's log says the set is unpacked, or says why it is not (its own `failed` lines); nothing yet is `None`: keep waiting.
pub fn push_says(log: &str) -> Option<Result<(), String>> {
    if log.lines().any(|l| l.trim_start().starts_with(RECV_DONE)) {
        return Some(Ok(()));
    }
    log.lines()
        .find(|l| l.contains("firmware-recv:") && l.contains("failed"))
        .map(|l| Err(l.trim().to_string()))
}

/// `rootfs-recv`'s log: done, or the first line of its own that is not done (every other line it prints ends it: build/initramfs-modem/rootfs-recv); nothing yet is `None`: keep waiting.
pub fn rootfs_says(log: &str) -> Option<Result<(), String>> {
    log.lines()
        .map(str::trim)
        .find(|l| l.starts_with("rootfs-recv:"))
        .map(|l| {
            if l.starts_with(ROOTFS_DONE) {
                Ok(())
            } else {
                Err(l.to_string())
            }
        })
}

/// `mm-up`'s answer: dbus and ModemManager run, or the line that says why not (its text is the whole answer).
pub fn mm_says(text: &str) -> Done {
    if text.lines().any(|l| l.trim_start().starts_with(MM_DONE)) {
        Ok(())
    } else {
        Err(format!("mm-up did not start ModemManager: {}", text.trim()))
    }
}

/// `modem-up`'s answer: the three daemons run, or the line that says why not (its text is the whole answer).
pub fn up_says(text: &str) -> Done {
    if text.lines().any(|l| l.trim_start().starts_with(UP_DONE)) {
        Ok(())
    } else {
        Err(format!(
            "modem-up did not start the daemons: {}",
            text.trim()
        ))
    }
}

/// Seconds a push of `bytes` takes at `PUSH_BYTES_PER_S` (rounded up).
pub fn push_s(bytes: u64) -> u64 {
    bytes.div_ceil(PUSH_BYTES_PER_S)
}

/// Seconds the firmware step and all after it need when the rootfs follows, at the usual costs: the firmware push, the daemons up and read, the rootfs push (from its size), `mm-up`, the final read.
pub fn firmware_to_end_usual_s(firmware_bytes: u64, rootfs_bytes: u64) -> u64 {
    push_s(firmware_bytes) + AFTER_FIRMWARE_USUAL_S + rootfs_tail_s(rootfs_bytes)
}

/// The size of a file to be pushed, read from the file.
fn size_of(path: &std::path::Path) -> Result<u64, String> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// The rootfs step and all after it, at the usual costs (the same set as the firmware step's guard, so a boot the firmware guard accepts the rootfs guard accepts after it): its push (from its size), `mm-up`, the read and the ids.
pub fn rootfs_tail_s(rootfs_bytes: u64) -> u64 {
    push_s(rootfs_bytes) + MM_UP_USUAL_S + ROOTFS_READ_USUAL_S
}

/// Whether a step of at most `need` seconds still fits the init's window, `elapsed` seconds in.
pub fn time_left_covers(elapsed: Duration, need_s: u64) -> Done {
    let left = INIT_WINDOW_S.saturating_sub(elapsed.as_secs());
    if left >= need_s {
        Ok(())
    } else {
        Err(format!(
            "{} s of the init's {INIT_WINDOW_S} s are gone and the step needs {need_s} s: the phone would restart under it",
            elapsed.as_secs()
        ))
    }
}

/// What the author is told before the run.
pub fn expect(luns: u64) -> String {
    format!("The modem boot, about 8 minutes: the modem image is booted once from memory (UFS off by the gate, nothing flashed, no slot changed); over its USB network the lab pushes the 61 MB firmware tar (no nc: a Rust copy to port {PUSH_PORT}), runs modem-up (it checks the tar's own SHA256SUMS first), pushes the ModemManager rootfs tar (about 185 MiB, to port {ROOTFS_PORT}; its sha256 checked first) and runs mm-up (dbus and ModemManager in a chroot) and reads every remoteproc's state, the kernel's modem lines, the daemons' logs and qrtr-lookup, then asks the modem its device ids over QRTR (qmicli, read only; the answer holds the IMEI, so it is kept in ~/lab-material and never shown). All of it ends inside the image's {INIT_WINDOW_S} s before it restarts the phone by itself; whatever the modem did is the answer. Android is checked back with the card's {luns} disks, the boot chain and the slots' flags read as before.")
}

/// What the author is told before the rehearsal.
pub fn expect_rehearsal(luns: u64) -> String {
    format!("The rehearsal of the modem boot, about 5 minutes: every step but the boot, the push and the read - the image's, the firmware tar's and the rootfs tar's sha256, the UFS-off gate, the stored boot chain, the card read from Android ({luns} disks and the root hashed), the trips to the bootloader and back, the slots' flags. Nothing is booted or written; the phone ends on Android. No go is needed or read.")
}

/// The firmware tar is sent to the phone's receiver: connected within `wait` (the listener starts a moment after `firmware-recv`), the file copied, the write side closed so `nc -l` on the phone ends.
pub fn push_file(
    address: &str,
    port: u16,
    file: &std::path::Path,
    wait: Duration,
) -> Result<u64, String> {
    let until = Instant::now() + wait;
    let target = (address, port)
        .to_socket_addrs()
        .map_err(|e| format!("{address}:{port}: {e}"))?
        .next()
        .ok_or_else(|| format!("{address}:{port} names no address"))?;
    let mut stream = loop {
        match TcpStream::connect_timeout(&target, Duration::from_secs(3)) {
            Ok(s) => break s,
            Err(e) if Instant::now() >= until => {
                return Err(format!(
                    "the receiver on {address}:{port} did not answer within {} s: {e}",
                    wait.as_secs()
                ))
            }
            Err(_) => std::thread::sleep(Duration::from_millis(500)),
        }
    };
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("the push: {e}"))?;
    let mut f = std::fs::File::open(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let n = std::io::copy(&mut f, &mut stream)
        .map_err(|e| format!("pushing {}: {e}", file.display()))?;
    stream
        .flush()
        .map_err(|e| format!("pushing {}: {e}", file.display()))?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|e| format!("closing the push: {e}"))?;
    Ok(n)
}

/// One push to a receiver on the phone: what is sent, where, how the receiver is started, whose log says it is done, and what is kept.
struct Push<'a> {
    file: &'a std::path::Path,
    port: u16,
    receiver: &'a str,
    log: &'a str,
    done: &'a str,
    says: fn(&str) -> Option<Result<(), String>>,
    kept: &'a str,
}

/// The boot: the shared steps of the SLPI boot (they hold for any UFS-off image booted once from memory) and the steps of its own.
#[derive(Clone)]
pub struct ModemBoot {
    s: Rc<Settings>,
    lab: Lab,
    base: SlpiBoot,
    /// when the image was booted: the init's window counts from here
    booted: Rc<Cell<Option<Instant>>>,
}

impl ModemBoot {
    pub fn new(s: Settings) -> ModemBoot {
        let base = SlpiBoot::new(s.clone());
        let s = Rc::new(s);
        ModemBoot {
            lab: Lab::new(s.clone()),
            s,
            base,
            booted: Rc::new(Cell::new(None)),
        }
    }

    pub fn shared(&self) -> &RootfsBoot {
        self.base.shared()
    }

    fn elapsed(&self) -> Result<Duration, String> {
        self.booted
            .get()
            .map(|t| t.elapsed())
            .ok_or_else(|| "the image was not booted by this run".to_string())
    }

    /// adb reads the slot (the shared step) after the files of an earlier run of the day are forgotten.
    pub fn the_slot_is_read_by_adb(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        let shared = self.shared().the_slot_is_read_by_adb();
        move || {
            for name in STALE {
                let _ = std::fs::remove_file(u.s.out.join(name));
            }
            shared()
        }
    }

    pub fn author_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect(self.s.luns);
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    pub fn rehearsal_is_told(&self) -> impl FnOnce() -> Done + 'static {
        let text = expect_rehearsal(self.s.luns);
        move || {
            println!("  {text}");
            Ok(())
        }
    }

    pub fn the_image_is_the_declared_ufs_off_image(&self) -> impl FnOnce() -> Done + 'static {
        self.base.the_image_is_the_declared_ufs_off_image()
    }

    /// The firmware tar is the declared one (its sha256 is the manifest's): what is pushed is what was packed and recorded.
    pub fn the_firmware_is_the_declared_tar(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || host::image_has_sha256(&u.s.firmware, &u.s.firmware_sha256)
    }

    pub fn the_stored_set_is_good(&self) -> impl FnOnce() -> Done + 'static {
        self.base.the_stored_set_is_good()
    }

    /// The boot itself, from memory; the init's window starts counting when it has been sent.
    pub fn boot_the_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        let boot = self.base.boot_the_image();
        move || {
            boot()?;
            u.booted.set(Some(Instant::now()));
            Ok(())
        }
    }

    pub fn the_gadget_is_up(&self) -> impl FnOnce() -> Done + 'static {
        self.base.the_gadget_is_up()
    }

    /// The rootfs tar is the declared one (its sha256 is the one `build/modem-mm-rootfs.sha256` recorded and device.toml names).
    pub fn the_rootfs_is_the_declared_tar(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || host::image_has_sha256(&u.s.rootfs, &u.s.rootfs_sha256)
    }

    /// The receiver is started on the phone, the tar is pushed to it, and the phone says it unpacked it; the receiver's log is kept (push.txt).
    pub fn the_firmware_is_pushed(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell_when_up()?;
            time_left_covers(
                u.elapsed()?,
                firmware_to_end_usual_s(size_of(&u.s.firmware)?, size_of(&u.s.rootfs)?),
            )?;
            host::image_has_sha256(&u.s.firmware, &u.s.firmware_sha256)?;
            u.push(
                &mut sh,
                &Push {
                    file: &u.s.firmware,
                    port: PUSH_PORT,
                    receiver: "firmware-recv",
                    log: FIRMWARE_LOG,
                    done: RECV_DONE,
                    says: push_says,
                    kept: "push.txt",
                },
            )
        }
    }

    /// The rootfs tar goes to `rootfs-recv` (port 9001) after `modem-up`, the same way; the step counts its own push and all after it (the rootfs, `mm-up`, the read, the ids). Kept as rootfs-push.txt.
    pub fn the_rootfs_is_pushed(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell_when_up()?;
            time_left_covers(u.elapsed()?, rootfs_tail_s(size_of(&u.s.rootfs)?))?;
            host::image_has_sha256(&u.s.rootfs, &u.s.rootfs_sha256)?;
            u.push(
                &mut sh,
                &Push {
                    file: &u.s.rootfs,
                    port: ROOTFS_PORT,
                    receiver: "rootfs-recv",
                    log: ROOTFS_LOG,
                    done: ROOTFS_DONE,
                    says: rootfs_says,
                    kept: "rootfs-push.txt",
                },
            )
        }
    }

    /// One push: the receiver is started on the phone, the file is sent to its port, the receiver's log is read until it says done or why not (within `RECEIVER_SAYS_S` of the push); the log is kept.
    fn push(&self, sh: &mut crate::phone::Shell, p: &Push) -> Done {
        sh.run(&format!("rm -f {}", p.log), Duration::from_secs(5))?;
        sh.background(&recv_command(p.receiver, p.log))?;
        let sent = push_file(
            &self.s.address,
            p.port,
            p.file,
            Duration::from_secs(CONNECT_WAIT_S),
        )?;
        let until = Instant::now() + Duration::from_secs(RECEIVER_SAYS_S);
        loop {
            let log = sh.run(&format!("cat {}", p.log), Duration::from_secs(10))?;
            let kept = format!("sent {sent} bytes\n{log}");
            match (p.says)(&log) {
                Some(Ok(())) => return self.lab.save(p.kept, &kept),
                Some(Err(e)) => {
                    self.lab.save(p.kept, &kept)?;
                    return Err(e);
                }
                None if Instant::now() >= until => {
                    self.lab.save(p.kept, &kept)?;
                    return Err(format!(
                        "the phone did not say `{}` within {RECEIVER_SAYS_S} s of the push; its log: {}",
                        p.done,
                        log.trim()
                    ));
                }
                None => std::thread::sleep(Duration::from_secs(2)),
            }
        }
    }

    /// `mm-up` runs once, after the rootfs is unpacked and `modem-up` has the modem running, and says dbus and ModemManager run; its words are kept (mm-up.txt). ModemManager's own log is in the read that follows.
    pub fn the_modemmanager_is_started(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell_when_up()?;
            time_left_covers(u.elapsed()?, MM_UP_S + READ_S + IDS_S)?;
            let said = sh.run("mm-up", Duration::from_secs(MM_UP_S))?;
            u.lab.save("mm-up.txt", &said)?;
            mm_says(&said)
        }
    }

    /// `modem-up` runs once and says the three daemons run; its words are kept (modem-up.txt).
    pub fn the_modem_is_started(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell_when_up()?;
            time_left_covers(u.elapsed()?, UP_S + READ_S + IDS_S)?;
            let said = sh.run("modem-up", Duration::from_secs(20))?;
            u.lab.save("modem-up.txt", &said)?;
            up_says(&said)?;
            std::thread::sleep(Duration::from_secs(SETTLE_S));
            Ok(())
        }
    }

    /// The phone is read over telnet and judged; the read, the kernel log and the daemons' logs are kept, the answer too. Whatever the answer is, the step passes.
    pub fn the_modem_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell_when_up()?;
            time_left_covers(u.elapsed()?, READ_S + IDS_S)?;
            let text = sh.run(READ_COMMAND, Duration::from_secs(40))?;
            u.lab.save("read.txt", &redact_ids(&text))?;
            u.lab.save("modem-up-check.txt", &u.health_check())?;
            let dmesg = sh.run("dmesg", Duration::from_secs(20))?;
            u.lab.save("dmesg.txt", &redact_ids(&dmesg))?;
            let logs = section(&text, "logs")
                .map(|l| l.join("\n"))
                .unwrap_or_default();
            u.lab.save("daemon-logs.txt", &redact_ids(&logs))?;
            let said = judge(&text)?.said();
            u.lab.save(
                "verdict.txt",
                &format!(
                    "{said}; read at {} s of the init's {INIT_WINDOW_S} s",
                    u.elapsed()?.as_secs()
                ),
            )?;
            println!("  {said}");
            Ok(())
        }
    }

    /// The modem's DMS get-ids is asked over QRTR, read only; the answer holds the IMEI, so it is kept in qmi-ids.txt of the run's folder (~/lab-material, mode 700) and only its shape is printed.
    /// Whatever qmicli says, the step passes: the modem's answer is the finding (not known yet when the modem is down).
    pub fn the_ids_are_read_and_kept_private(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell_when_up()?;
            time_left_covers(u.elapsed()?, IDS_S)?;
            let text = sh.run(IDS_COMMAND, Duration::from_secs(IDS_S))?;
            println!("  {}", keep_the_ids(&u.s.out, &text)?);
            Ok(())
        }
    }

    /// tests/health/base/modem-up.sh run on the read just kept (read.txt), as the phone's suite would run it. Evidence only: its `fail` lines are kept
    /// in modem-up-check.txt and never end the run (the verdict and dmesg are still kept); a lib or a shell that is not there is said, never guessed.
    /// The suite's words (lab-tools health/suite/lib.sh) come from `LPOCO_HEALTH_LIB`, else the lab-tools clone's file.
    fn health_check(&self) -> String {
        let lib = std::env::var("LPOCO_HEALTH_LIB").unwrap_or_else(|_| {
            format!(
                "{}/Projects/linux-phone-oco/lab-tools/health/suite/lib.sh",
                std::env::var("HOME").unwrap_or_default()
            )
        });
        health_text(
            &lib,
            &self.s.lab.join("tests/health/base/modem-up.sh"),
            &self.s.out.join("read.txt"),
        )
    }

    /// The way back, the last step of the run, the shared one (only the watching parent runs it on a failure).
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        self.base.way_back()
    }
}

/// What `sh CHECK` prints on the kept read, with the suite's words at `lib`; the reasons it could not run are the text, not an error.
fn health_text(lib: &str, check: &std::path::Path, read: &std::path::Path) -> String {
    if !std::path::Path::new(lib).is_file() {
        return format!("not run: the suite's lib.sh is not at {lib} (set LPOCO_HEALTH_LIB)");
    }
    let mut c = std::process::Command::new("sh");
    c.arg(check)
        .env("LPOCO_HEALTH_LIB", lib)
        .env("LPOCO_MODEM_READ", read);
    match host::run(c) {
        Ok(said) if said.ok => said.text,
        Ok(said) => format!("not run: modem-up.sh exited with an error: {}", said.text),
        Err(e) => format!("not run: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;

    fn read(rproc: &str, dmesg: &str, qrtr: &str) -> String {
        format!("---remoteproc\n{rproc}\n---dmesg\n{dmesg}\n---logs\n== /tmp/rmtfs.log\n---qrtr\n{qrtr}\n---end")
    }

    /// kernel format strings (remoteproc_core.c 1935, 1308) filled with the device's name (qcom_q6v5_mss.c 2091); NOT captured from a phone.
    const SOURCE_UP: &str = "remoteproc remoteproc1: powering up 4080000.remoteproc\nremoteproc remoteproc1: remote processor 4080000.remoteproc is now up";

    #[test]
    fn the_modem_is_found_by_its_device_name_and_a_running_one_with_services_is_said_up() {
        let a = judge(&read(
            "4080000.remoteproc running\n5c00000.remoteproc offline",
            SOURCE_UP,
            "  Service Version Instance Node  Port\n       15       1        0    0     2",
        ))
        .unwrap();
        assert_eq!(a.mss, Some(State::Running));
        assert_eq!(
            a.others,
            [("5c00000.remoteproc".to_string(), State::Offline)]
        );
        assert_eq!(a.services, Ok(1));
        assert_eq!(
            a.said(),
            "remoteproc modem is running; the kernel says the modem processor is now up; no failure line; qrtr-lookup lists 1 service(s)"
        );
    }

    #[test]
    fn an_offline_modem_before_modem_up_is_an_answer_and_a_missing_one_names_the_ones_listed() {
        let a = judge(&read(
            "4080000.remoteproc offline",
            "",
            "qrtr-lookup: not found",
        ))
        .unwrap();
        assert_eq!(a.mss, Some(State::Offline));
        assert!(
            a.said().contains("does not say it started the modem"),
            "{}",
            a.said()
        );
        let a = judge(&read("slpi running", "", "")).unwrap();
        assert!(
            a.said()
                .starts_with("no remoteproc is named 4080000.remoteproc (the ones listed: slpi)"),
            "{}",
            a.said()
        );
    }

    #[test]
    fn a_modem_failure_line_is_kept() {
        let a = judge(&read(
            "4080000.remoteproc crashed",
            "remoteproc remoteproc1: powering up 4080000.remoteproc\nqcom-q6v5-mss 4080000.remoteproc: fatal error received: x",
            "",
        ))
        .unwrap();
        assert_eq!(a.mss, Some(State::Crashed));
        assert_eq!(a.log.failures.len(), 1);
        assert!(a.said().starts_with("remoteproc modem CRASHED; the kernel says it powered the modem up and never that it is up; 1 failure line(s)"));
    }

    #[test]
    fn the_slpi_judgement_is_unchanged_by_the_name_it_is_given() {
        assert_eq!(crate::slpi_boot::log_says(SOURCE_UP), Log::default());
        assert!(log_says_for(MSS_NAME, SOURCE_UP).up);
    }

    #[test]
    fn the_receiver_is_done_failed_or_not_yet() {
        // the scripts' own words: build/initramfs-modem/firmware-recv
        assert_eq!(
            push_says("firmware-recv: done, 31 files in /lib/firmware/qcom/msm8998/google/walleye"),
            Some(Ok(()))
        );
        assert_eq!(
            push_says("firmware-recv: tar failed"),
            Some(Err("firmware-recv: tar failed".into()))
        );
        assert_eq!(
            push_says("firmware-recv: nc failed"),
            Some(Err("firmware-recv: nc failed".into()))
        );
        assert_eq!(push_says(""), None);
    }

    #[test]
    fn modem_up_is_said_by_its_own_last_line_and_stops_say_why() {
        assert!(up_says("modem-up: qrtr-ns, tqftpserv and rmtfs run (logs in /tmp/*.log)").is_ok());
        let e = up_says(
            "modem-up: the pushed set does not match its SHA256SUMS (see /tmp/firmware-sums.log)",
        )
        .unwrap_err();
        assert!(e.contains("does not match its SHA256SUMS"), "{e}");
        assert!(up_says("").is_err());
    }

    #[test]
    fn the_health_check_text_says_what_it_could_not_run_and_keeps_fail_lines() {
        let d = std::env::temp_dir().join(format!("wt-modem-hc-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let (lib, check, read) = (d.join("lib.sh"), d.join("check.sh"), d.join("read.txt"));
        std::fs::write(
            &check,
            "echo \"fail modem-remoteproc-running modem-data via $LPOCO_MODEM_READ\"\n",
        )
        .unwrap();
        let missing = health_text("/nonexistent/lib.sh", &check, &read);
        assert!(
            missing.starts_with("not run: the suite's lib.sh is not at /nonexistent/lib.sh"),
            "{missing}"
        );
        std::fs::write(&lib, "").unwrap();
        let said = health_text(lib.to_str().unwrap(), &check, &read);
        assert_eq!(
            said,
            format!(
                "fail modem-remoteproc-running modem-data via {}",
                read.display()
            )
        );
        std::fs::remove_dir_all(&d).unwrap();
    }

    /// The ids read's words are libqmi's own (qmicli-dms.c line 643 `Device IDs retrieved:`; qmicli prints `error: ...` on a failure) - NOT captured from a phone yet: the first real run is the capture.
    /// Whatever the text, the sentence says no value from it.
    #[test]
    fn the_ids_read_is_said_without_its_values() {
        let up = "[qrtr://0] Device IDs retrieved:\n\t      ESN: 'unknown'\n\t     IMEI: '000000000000000'\n\t     MEID: 'unknown'";
        let said = ids_say(up);
        assert!(
            said.starts_with("qmicli answered the DMS get-ids (4 lines"),
            "{said}"
        );
        assert!(!said.contains("000000000000000"));
        assert!(ids_say(
            "error: couldn't create client for the 'dms' service: Timeout was reached"
        )
        .starts_with("qmicli said an error"));
        assert!(ids_say("").starts_with("qmicli did not answer (0 lines"));
        assert!(ids_say("timeout: killed").starts_with("qmicli did not answer"));
    }

    /// The answer reaches one file only (mode 600 in a folder of mode 700), the printed sentence holds no value, and a second run replaces it.
    #[test]
    fn the_ids_are_kept_in_one_private_file_and_never_in_the_sentence() {
        use std::os::unix::fs::PermissionsExt;
        let d = std::env::temp_dir().join(format!("wt-ids-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let imei = "[qrtr://0] Device IDs retrieved:\n\t     IMEI: '000000000000000'";
        let said = keep_the_ids(&d, imei).unwrap();
        assert!(!said.contains("000000000000000"), "{said}");
        let file = d.join("qmi-ids.txt");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), format!("{imei}\n"));
        assert_eq!(
            std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(&d).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 1);
        keep_the_ids(&d, "error: x").unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "error: x\n");
        std::fs::remove_dir_all(&d).unwrap();
    }

    /// `-d` takes the QRTR URI and `-p` is the proxy flag (libqmi qmicli.c lines 79-104): the command must name the device with -d, and ask nothing but get-ids.
    #[test]
    fn the_ids_command_names_the_qrtr_node_with_d_and_asks_only_get_ids() {
        assert!(IDS_COMMAND.contains(" -d qrtr://0 ") && !IDS_COMMAND.contains(" -p "));
        assert_eq!(IDS_COMMAND.matches("--dms-").count(), 1);
        assert!(
            IDS_COMMAND.contains("--dms-get-ids")
                && !IDS_COMMAND.contains("--nas-")
                && !IDS_COMMAND.contains("--wds-")
        );
        assert!(IDS_COMMAND.starts_with(&format!("timeout -k {IDS_KILL_S} {IDS_TIMEOUT_S} ")));
        const { assert!(IDS_S > IDS_TIMEOUT_S + IDS_KILL_S) };
    }

    /// A 15-digit IMEI in a daemon log is masked, shorter and longer numbers (a timestamp, a byte count) are kept.
    #[test]
    fn the_kept_reads_mask_an_imei_and_keep_other_numbers() {
        let said = redact_ids("IMEI: '000000000000000' at 1795785213 in 61501440 bytes");
        assert_eq!(said, "IMEI: '<id masked>' at 1795785213 in 61501440 bytes");
    }

    /// The real entry runs the steps in the order the phone needs: the firmware, modem-up, the rootfs, mm-up (which needs qrtr-ns from modem-up and the rootfs), then the read and the ids.
    #[test]
    fn the_entry_runs_the_pushes_and_starts_in_order() {
        let real = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../procedures/modem-boot-on-the-pixel-2.rs"
        ))
        .unwrap();
        let at = |s: &str| real.find(s).unwrap_or_else(|| panic!("{s}"));
        let order = [
            "b.the_firmware_is_pushed",
            "b.the_modem_is_started",
            "b.the_rootfs_is_pushed",
            "b.the_modemmanager_is_started",
            "b.the_modem_is_read",
            "b.the_ids_are_read_and_kept_private",
        ];
        for w in order.windows(2) {
            assert!(at(w[0]) < at(w[1]), "{} before {}", w[0], w[1]);
        }
    }

    /// Every file a step saves is in STALE (read from this file's own text), so a run never keeps an earlier run's answer.
    #[test]
    fn every_file_the_steps_save_is_forgotten_at_the_start() {
        let text =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/modem_boot.rs"))
                .unwrap();
        for name in [
            "push.txt",
            "modem-up.txt",
            "modem-up-check.txt",
            "read.txt",
            "dmesg.txt",
            "daemon-logs.txt",
            "verdict.txt",
            "rootfs-push.txt",
            "mm-up.txt",
            "qmi-ids.txt",
        ] {
            assert!(
                text.matches(&format!("\"{name}\"")).count() >= 3,
                "{name} is not named in STALE, a step and this test"
            );
            assert!(STALE.contains(&name), "{name}");
        }
    }

    /// A push into a listener that never reads ends by its write timeout instead of hanging the run.
    #[test]
    fn a_stalled_receiver_ends_the_push() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let dir = std::env::temp_dir().join(format!("wt-stall-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("big");
        std::fs::File::create(&file)
            .unwrap()
            .set_len(128 << 20)
            .unwrap(); // sparse: no RAM
        let t = std::time::Instant::now();
        let e = push_file("127.0.0.1", port, &file, Duration::from_secs(2)).unwrap_err();
        assert!(e.contains("pushing"), "{e}");
        assert!(t.elapsed() < Duration::from_secs(40));
        drop(l);
    }

    #[test]
    fn the_guards_never_ask_more_after_than_before() {}

    #[test]
    fn the_init_window_refuses_a_step_that_would_not_fit() {
        assert!(time_left_covers(Duration::from_secs(60), 100).is_ok());
        let e = time_left_covers(Duration::from_secs(100), 100).unwrap_err();
        assert!(e.contains("100 s of the init's 180 s are gone"), "{e}");
        assert!(time_left_covers(Duration::from_secs(500), 1).is_err());
    }

    #[test]
    fn the_read_command_has_the_marks_the_judge_reads() {
        for part in ["remoteproc", "dmesg", "logs", "qrtr", "end"] {
            assert!(READ_COMMAND.contains(&format!("echo ---{part}")), "{part}");
        }
        assert!(READ_COMMAND.contains("[ -e $d/name ] || continue"));
        assert!(READ_COMMAND.contains("qrtr-lookup") && READ_COMMAND.contains("/tmp/*.log"));
    }

    /// The receiver the push talks to is the one the image holds: port and words are the script's.
    #[test]
    fn the_push_names_what_the_images_receiver_script_says() {
        let script = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../build/initramfs-modem/firmware-recv"
        ))
        .unwrap();
        assert!(script.contains(&format!("nc -l -p {PUSH_PORT}")));
        assert!(script.contains(RECV_DONE));
        let up = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../build/initramfs-modem/modem-up"
        ))
        .unwrap();
        assert!(up.contains(UP_DONE));
        assert!(up.contains("sha256sum -c SHA256SUMS"));
    }

    /// The lines a script of the image prints: every `echo "PREFIX: ..."` of its text, quotes and trailing `; exit` cut (the script's own words, never typed here).
    fn script_says(script: &str, prefix: &str) -> Vec<String> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/initramfs-modem/");
        std::fs::read_to_string(format!("{dir}{script}"))
            .unwrap()
            .split("echo \"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .filter(|w| w.starts_with(prefix))
            .map(|w| w.to_string())
            .collect()
    }

    /// Every word `rootfs-recv` can print (build/initramfs-modem/rootfs-recv): only its `done` is an answer of success, every other line of it ends the push with that line.
    #[test]
    fn rootfs_says_what_the_receivers_own_words_say() {
        let words = script_says("rootfs-recv", "rootfs-recv:");
        assert!(words.len() >= 6, "{words:?}");
        for w in &words {
            let said = rootfs_says(w);
            if w.starts_with(ROOTFS_DONE) {
                assert_eq!(said, Some(Ok(())), "{w}");
            } else {
                assert_eq!(said, Some(Err(w.clone())), "{w}");
            }
        }
        assert_eq!(rootfs_says(""), None);
        assert_eq!(rootfs_says("some other noise"), None);
    }

    /// Every word `mm-up` can print (build/initramfs-modem/mm-up): only its `dbus and ModemManager run` ends well; the others are why it stopped.
    #[test]
    fn mm_says_what_mm_ups_own_words_say() {
        let words = script_says("mm-up", "mm-up:");
        assert!(words.len() >= 6, "{words:?}");
        for w in &words {
            let said = mm_says(w);
            if w.starts_with(MM_DONE) {
                assert!(said.is_ok(), "{w}");
            } else {
                let e = said.unwrap_err();
                assert!(e.contains(w.as_str()), "{e}");
            }
        }
        assert!(mm_says("").is_err());
    }

    /// The port, the done words and the command the lab uses are the receiver's and mm-up's own.
    #[test]
    fn the_rootfs_push_names_what_the_images_scripts_say() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/initramfs-modem/");
        let recv = std::fs::read_to_string(format!("{dir}rootfs-recv")).unwrap();
        assert!(recv.contains(&format!("MM_PORT:-{ROOTFS_PORT}")));
        assert!(recv.contains(ROOTFS_DONE));
        assert_eq!(
            recv_command("rootfs-recv", ROOTFS_LOG),
            "rootfs-recv > /tmp/rootfs-recv.log 2>&1"
        );
        let up = std::fs::read_to_string(format!("{dir}mm-up")).unwrap();
        assert!(up.contains(MM_DONE));
        assert!(up.contains("run modem-up first"));
    }

    /// The rootfs tar's sha256 in device.toml (both tables) is the one `build/modem-mm-rootfs.sha256` recorded when the tar was built.
    #[test]
    fn the_declared_rootfs_sha256_is_the_one_the_build_recorded() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
        let built = std::fs::read_to_string(format!("{root}build/modem-mm-rootfs.sha256")).unwrap();
        let (sha, name) = built.split_once("  ").unwrap();
        assert_eq!(name.trim(), "modem-mm-rootfs.tar");
        let toml = std::fs::read_to_string(format!("{root}device.toml")).unwrap();
        for table in [
            "[procedures.modem-boot-on-the-pixel-2]",
            "[procedures.rehearse-modem-boot-on-the-pixel-2]",
        ] {
            let body = toml
                .split(table)
                .nth(1)
                .unwrap()
                .split("\n[")
                .next()
                .unwrap();
            assert!(
                body.contains(&format!("rootfs_sha256 = \"{sha}\"")),
                "{table}"
            );
            assert!(
                body.contains("rootfs = \"images-modem/modem-mm-rootfs.tar\""),
                "{table}"
            );
        }
    }

    /// The sizes the guards count: the firmware tar's (device.toml, [procedures.modem-boot-on-the-pixel-2] comment) and the rootfs tar's as nix built it (`stat` of the tar whose sha256 is build/modem-mm-rootfs.sha256).
    const FIRMWARE_BYTES: u64 = 61_501_440;
    const ROOTFS_BYTES: u64 = 194_375_680;

    /// The rootfs step counts its own push (from its size) and all after it, and fits the window after the firmware and modem-up (about 25 s counted by the last real run); a later boot is refused.
    #[test]
    fn the_rootfs_guards_count_the_push_and_everything_after_it() {
        assert_eq!(push_s(ROOTFS_BYTES), 81);
        let tail = rootfs_tail_s(ROOTFS_BYTES);
        assert_eq!(tail, 81 + MM_UP_USUAL_S + ROOTFS_READ_USUAL_S);
        assert!(time_left_covers(Duration::from_secs(25), tail).is_ok());
        assert!(time_left_covers(Duration::from_secs(90), tail).is_err());
    }

    /// The firmware step refuses a boot too late for the firmware push plus the rootfs push (the rootfs counted from its size and the stated throughput), and passes one in time.
    #[test]
    fn the_firmware_guard_counts_the_rootfs_push_too() {
        let need = firmware_to_end_usual_s(FIRMWARE_BYTES, ROOTFS_BYTES);
        assert_eq!(
            need,
            26 + AFTER_FIRMWARE_USUAL_S + 81 + MM_UP_USUAL_S + ROOTFS_READ_USUAL_S
        );
        assert!(need < INIT_WINDOW_S, "{need} s must fit the window");
        let late = Duration::from_secs(INIT_WINDOW_S - need + 1);
        let e = time_left_covers(late, need).unwrap_err();
        assert!(e.contains("the step needs"), "{e}");
        assert!(time_left_covers(Duration::from_secs(INIT_WINDOW_S - need), need).is_ok());
        // the rootfs counted: without it the same boot would pass the firmware-only need
        let late_for_rootfs = Duration::from_secs(INIT_WINDOW_S - need + 1);
        assert!(time_left_covers(
            late_for_rootfs,
            push_s(FIRMWARE_BYTES) + AFTER_FIRMWARE_USUAL_S
        )
        .is_ok());
    }

    /// The two guards agree: the latest boot the firmware guard accepts, after the firmware push and the daemons' read, is still accepted by the rootfs guard.
    #[test]
    fn a_boot_the_firmware_guard_accepts_the_rootfs_guard_accepts_after_it() {
        let need = firmware_to_end_usual_s(FIRMWARE_BYTES, ROOTFS_BYTES);
        let latest = INIT_WINDOW_S - need;
        assert!(time_left_covers(Duration::from_secs(latest), need).is_ok());
        let at_rootfs = latest + push_s(FIRMWARE_BYTES) + AFTER_FIRMWARE_USUAL_S;
        assert!(
            time_left_covers(Duration::from_secs(at_rootfs), rootfs_tail_s(ROOTFS_BYTES)).is_ok()
        );
    }

    /// A rootfs twice the size needs more time: the guard follows the file, not a constant.
    #[test]
    fn the_guard_follows_the_rootfs_size() {
        assert!(rootfs_tail_s(2 * ROOTFS_BYTES) > rootfs_tail_s(ROOTFS_BYTES) + 80);
        assert!(
            firmware_to_end_usual_s(FIRMWARE_BYTES, 2 * ROOTFS_BYTES) >= INIT_WINDOW_S,
            "twice the rootfs no longer fits the window: the guard refuses it"
        );
    }

    /// The push sends every byte and closes the write side (the phone's `nc -l` ends on that), tried against a listener of our own on the loopback.
    #[test]
    fn the_push_sends_every_byte_and_ends_the_stream() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let dir = std::env::temp_dir().join(format!("wt-push-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("fw.tar");
        let bytes: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(&file, &bytes).unwrap();
        let t = std::thread::spawn(move || {
            let (mut c, _) = l.accept().unwrap();
            let mut got = Vec::new();
            c.read_to_end(&mut got).unwrap(); // returns only if the sender closed its side
            got
        });
        let n = push_file("127.0.0.1", port, &file, Duration::from_secs(5)).unwrap();
        assert_eq!(n, bytes.len() as u64);
        assert_eq!(t.join().unwrap(), bytes);
    }

    #[test]
    fn a_receiver_that_never_listens_is_named_after_the_wait() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        drop(l);
        let e = push_file(
            "127.0.0.1",
            port,
            std::path::Path::new("/nonexistent"),
            Duration::from_millis(600),
        )
        .unwrap_err();
        assert!(e.contains("did not answer within"), "{e}");
    }

    /// Read from the entry files themselves: the real run boots, pushes and reads; the rehearsal does none of them; neither reads a go; both end in the shared way back.
    #[test]
    fn the_entries_boot_push_and_read_only_in_the_real_run() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../procedures/");
        let real = std::fs::read_to_string(format!("{dir}modem-boot-on-the-pixel-2.rs")).unwrap();
        let rehearsal =
            std::fs::read_to_string(format!("{dir}rehearse-modem-boot-on-the-pixel-2.rs")).unwrap();
        for step in [
            "boot_the_image",
            "the_firmware_is_pushed",
            "the_modem_is_started",
            "the_rootfs_is_pushed",
            "the_modemmanager_is_started",
            "the_modem_is_read",
            "the_ids_are_read_and_kept_private",
        ] {
            assert!(real.contains(step), "{step}");
            assert!(!rehearsal.contains(step), "{step}");
        }
        for text in [&real, &rehearsal] {
            for banned in ["author_has_said_go", "set_active", "the_guard_holds"] {
                assert!(!text.contains(banned), "{banned}");
            }
            assert!(text.contains("b.way_back()"));
            assert!(text.contains("the_image_is_the_declared_ufs_off_image"));
            assert!(text.contains("the_firmware_is_the_declared_tar"));
            assert!(text.contains("the_rootfs_is_the_declared_tar"));
        }
        assert!(real.contains("Settings::for_modem_boot()"));
        assert!(rehearsal.contains("Settings::for_rehearse_modem_boot()"));
    }
}
