// SPDX-License-Identifier: MIT OR Apache-2.0
//! The SLPI boot (item walleye-slpi-boot-procedure, for the try walleye-slpi-boot-qrtr; research walleye-slpi-image-kernel-pas and walleye-slpi-qrtr-lookup):
//! the SLPI image ([images.slpi]: z21's init, the kernel with the PAS driver, the phone's own SLPI firmware, /bin/qrtr-lookup) is booted ONCE from fastboot, from
//! memory (nothing flashed, no slot touched), and over its telnet the lab reads what the SLPI did: the state of every remoteproc, the kernel's lines on remoteproc,
//! slpi, glink and qrtr, and the services `qrtr-lookup` lists. The answer is kept in the run's folder (read.txt, dmesg.txt, verdict.txt); whatever it is - up, offline,
//! crashed, no service - is an answer, not a failure of the run. The way back is the shared one (`reboot -f` on the image's shell, `fastboot reboot` in the bootloader):
//! Android's slot was never changed, so a plain reboot returns to Android.
//!
//! Where each judged word comes from (no text of the SLPI running was captured yet: the first real run is the capture, and until then the checks name the kernel's source):
//!  - the state words `offline`, `running`, `crashed`: drivers/remoteproc/remoteproc_sysfs.c, `rproc_state_string` (read in the Linux 7.2.2 tree of ~/Projects/walleye-build/porthole);
//!  - the remoteproc's name `slpi`: drivers/remoteproc/qcom_q6v5_pas.c, the msm8998 SLPI descriptor's `.sysmon_name = "slpi"`, which `devm_rproc_alloc` takes as the name;
//!  - `remote processor slpi is now up` and `powering up slpi`: drivers/remoteproc/remoteproc_core.c, `rproc_start` and `rproc_boot`;
//!  - qrtr-lookup's table (a header line `  Service Version Instance Node  Port`, then one row of five numbers per service): src/lookup.c of the qrtr commit rootfs/pins pins.

use crate::host;
use crate::lab::Lab;
use crate::rootfs_boot::RootfsBoot;
use crate::settings::Settings;
use std::rc::Rc;
use std::time::Duration;

type Done = Result<(), String>;

/// The files of the run's folder; an earlier run of the day's are forgotten at the start (a stale answer is never this run's).
pub const STALE: [&str; 3] = ["read.txt", "dmesg.txt", "verdict.txt"];
/// The remoteproc the boot is about.
pub const SLPI_NAME: &str = "slpi";
/// The header line of qrtr-lookup's table.
pub const QRTR_HEADER: &str = "Service Version Instance Node  Port";

/// The one read over telnet: every remoteproc's name and state, the kernel's lines on the SLPI and its transport, qrtr-lookup's list; marks between the parts.
pub const READ_COMMAND: &str = "echo ---remoteproc; for d in /sys/class/remoteproc/*; do [ -e $d/name ] || continue; echo \"$(cat $d/name) $(cat $d/state)\"; done; echo ---dmesg; dmesg | grep -i -e remoteproc -e remote.processor -e slpi -e qrtr -e glink -e sysmon; echo ---qrtr; qrtr-lookup 2>&1; echo ---end";

/// What a remoteproc's state file said.
#[derive(Debug, PartialEq, Clone)]
pub enum State {
    Offline,
    Running,
    Crashed,
    /// a word the lab has no name for (suspended, deleted, detached, invalid): said as it is
    Other(String),
}

fn state_of(word: &str) -> State {
    match word {
        "offline" => State::Offline,
        "running" => State::Running,
        "crashed" => State::Crashed,
        other => State::Other(other.to_string()),
    }
}

fn section<'a>(text: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let mark = format!("---{name}");
    let mut lines = text.lines().map(str::trim_end);
    lines.find(|l| *l == mark)?;
    Some(lines.take_while(|l| !l.starts_with("---")).collect())
}

/// Every remoteproc of the read, `(name, state)` in the order listed. A read without the part, or a line that is not a name and a state, is an error: nothing is concluded.
pub fn remoteprocs_in(text: &str) -> Result<Vec<(String, State)>, String> {
    let lines = section(text, "remoteproc").ok_or("the read has no remoteproc part")?;
    lines
        .into_iter()
        .filter(|l| !l.trim().is_empty())
        .map(
            |l| match l.split_whitespace().collect::<Vec<_>>().as_slice() {
                [name, state] => Ok((name.to_string(), state_of(state))),
                _ => Err(format!("a remoteproc line is not `name state`: {l:?}")),
            },
        )
        .collect()
}

/// What the kernel's lines say about the SLPI: it came up, it was started, a line that tells of a failure.
#[derive(Debug, PartialEq, Default)]
pub struct Log {
    pub started: bool,
    pub up: bool,
    pub failures: Vec<String>,
}

/// The kernel's lines (a whole log, or the grep of it that [`READ_COMMAND`] makes) judged for the SLPI. A failure is a line of the SLPI's own transport or driver that says `fail`, `error`, `timed out` or `crash`.
pub fn log_says(text: &str) -> Log {
    let mut log = Log::default();
    for line in text.lines() {
        let l = line.to_ascii_lowercase();
        if l.contains(&format!("powering up {SLPI_NAME}")) {
            log.started = true;
        }
        if l.contains(&format!("remote processor {SLPI_NAME} is now up")) {
            log.up = true;
        }
        let about = l.contains(SLPI_NAME)
            || l.contains("remoteproc")
            || l.contains("glink")
            || l.contains("qrtr")
            || l.contains("sysmon");
        if about
            && [
                "fail",
                "error",
                "timed out",
                "timeout",
                "crash",
                "unable",
                "watchdog",
            ]
            .iter()
            .any(|w| l.contains(w))
        {
            log.failures.push(line.trim().to_string());
        }
    }
    log
}

/// The number of services qrtr-lookup listed: the rows after its header line. A read whose part has no header is an error with what the tool said (no socket, no tool).
pub fn services_in(text: &str) -> Result<usize, String> {
    let lines = section(text, "qrtr").ok_or("the read has no qrtr part")?;
    let at = lines
        .iter()
        .position(|l| l.split_whitespace().eq(QRTR_HEADER.split_whitespace()))
        .ok_or_else(|| {
            format!(
                "qrtr-lookup printed no table; it said: {}",
                lines.first().map_or("nothing", |l| l.trim())
            )
        })?;
    Ok(lines[at + 1..]
        .iter()
        .filter(|l| {
            let w: Vec<&str> = l.split_whitespace().collect();
            w.len() >= 5
                && [0, 2, 3, 4].iter().all(|&i| w[i].parse::<u32>().is_ok())
                && (w[1].parse::<u32>().is_ok() || w[1] == "N/A")
        })
        .count())
}

/// What the boot read, whole.
#[derive(Debug, PartialEq)]
pub struct Answer {
    /// the SLPI's remoteproc state, or none when no remoteproc is named `slpi`
    pub slpi: Option<State>,
    pub others: Vec<(String, State)>,
    pub log: Log,
    pub services: Result<usize, String>,
}

/// The answer of a telnet read. Only a read that has not got its parts is an error.
pub fn judge(text: &str) -> Result<Answer, String> {
    let all = remoteprocs_in(text)?;
    let log = log_says(
        &section(text, "dmesg")
            .ok_or("the read has no dmesg part")?
            .join("\n"),
    );
    let services = services_in(text);
    let (slpi, others): (Vec<_>, Vec<_>) = all.into_iter().partition(|(n, _)| n == SLPI_NAME);
    Ok(Answer {
        slpi: slpi.into_iter().next().map(|(_, s)| s),
        others,
        log,
        services,
    })
}

impl Answer {
    /// The line the research row records: what was counted, not why.
    pub fn said(&self) -> String {
        let state = match &self.slpi {
            Some(State::Running) => "remoteproc slpi is running".to_string(),
            Some(State::Offline) => "remoteproc slpi is offline".to_string(),
            Some(State::Crashed) => "remoteproc slpi CRASHED".to_string(),
            Some(State::Other(w)) => format!("remoteproc slpi is {w}"),
            None => format!(
                "no remoteproc is named {SLPI_NAME} (the ones listed: {})",
                self.names()
            ),
        };
        let log = match (self.log.started, self.log.up) {
            (_, true) => "the kernel says `remote processor slpi is now up`".to_string(),
            (true, false) => {
                "the kernel says it powered slpi up and never that it is up".to_string()
            }
            (false, false) => "the kernel does not say it started slpi".to_string(),
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

    fn names(&self) -> String {
        self.others
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// What the author is told before the run.
pub fn expect(luns: u64) -> String {
    format!("The SLPI boot, about 8 minutes: the SLPI image is booted once from memory (UFS off by the gate, nothing flashed, no slot changed), and over its telnet the lab reads every remoteproc's state, the kernel's lines on slpi, glink and qrtr, and the services qrtr-lookup lists. Whatever the SLPI did is the answer. The image restarts the phone by itself after about 3 minutes; the way back sends `reboot -f` on its shell first. Android is checked back with the card's {luns} disks, the boot chain and the slots' flags read as before.")
}

/// What the author is told before the rehearsal.
pub fn expect_rehearsal(luns: u64) -> String {
    format!("The rehearsal of the SLPI boot, about 5 minutes: every step but the boot - the image's sha256 and the UFS-off gate, the stored boot chain, the card read from Android ({luns} disks and the root hashed), the trips to the bootloader and back, the slots' flags. Nothing is booted or written; the phone ends on Android. No go is needed or read.")
}

/// The boot: the shared steps of the root boot (battery, Android as root, the card, fastboot, the flags) through [`RootfsBoot`], and the steps of its own.
#[derive(Clone)]
pub struct SlpiBoot {
    s: Rc<Settings>,
    lab: Lab,
    root: RootfsBoot,
}

impl SlpiBoot {
    pub fn new(s: Settings) -> SlpiBoot {
        let root = RootfsBoot::new(s.clone());
        let s = Rc::new(s);
        SlpiBoot {
            lab: Lab::new(s.clone()),
            s,
            root,
        }
    }

    pub fn shared(&self) -> &RootfsBoot {
        &self.root
    }

    /// adb reads the slot (the shared step) after the files of an earlier run of the day are forgotten.
    pub fn the_slot_is_read_by_adb(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        let shared = self.root.the_slot_is_read_by_adb();
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

    /// The image is the declared one (its sha256 is the manifest's) and UFS is off in it (the gate every image passes before a boot).
    pub fn the_image_is_the_declared_ufs_off_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            u.lab.ufs_is_off(&u.s.image)
        }
    }

    /// The stored boot chain is whole (the lab's own check).
    pub fn the_stored_set_is_good(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let dir = u.s.bootchain.to_string_lossy().into_owned();
            u.lab
                .script("bootchain-readback-check.sh", &[&dir], &[])
                .map(|_| ())
        }
    }

    /// The boot itself, from memory: the gate right before it is the battery, the sha256 and UFS off, read again.
    pub fn boot_the_image(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            u.lab.battery_is_good()?;
            host::image_has_sha256(&u.s.image, &u.s.image_sha256)?;
            u.lab.ufs_is_off(&u.s.image)?;
            u.lab.boot_image(&u.s.image)
        }
    }

    /// The gadget is up on USB and the shell answers (the lab's gadget-up check, waiting up to a minute).
    pub fn the_gadget_is_up(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let toml = u.s.device_toml();
            u.lab.script(
                "gadget-up.sh",
                &[&toml.to_string_lossy()],
                &[("GADGET_WAIT", "60")],
            )?;
            u.lab.shell_when_up().map(|_| ())
        }
    }

    /// The phone is read over telnet and judged; the read and the whole kernel log are kept, the answer too. Whatever the answer is, the step passes.
    pub fn the_slpi_is_read(&self) -> impl FnOnce() -> Done + 'static {
        let u = self.clone();
        move || {
            let mut sh = u.lab.shell_when_up()?;
            let text = sh.run(READ_COMMAND, Duration::from_secs(60))?;
            u.lab.save("read.txt", &crate::touch::redact(&text))?;
            let dmesg = sh.run("dmesg", Duration::from_secs(20))?;
            u.lab.save("dmesg.txt", &crate::touch::redact(&dmesg))?;
            let said = judge(&text)?.said();
            u.lab.save("verdict.txt", &said)?;
            println!("  {said}");
            Ok(())
        }
    }

    /// The way back, the last step of the run, the shared one: the phone ends on Android whatever stopped the run (only the watching parent runs it).
    pub fn way_back(&self) -> impl FnOnce() -> Done + 'static {
        self.root.rehearsal_way_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real kernel log of our image (UFS root boot, 2026-11-25): it has a remoteproc line (the glink_ssr device) and no slpi line.
    const CAPTURED_LOG: &str =
        include_str!("../../../tests/fixtures/phone-text/dmesg-ufs-root-20261125-043720.cap");

    /// The kernel's own format strings, filled with the name the msm8998 descriptor gives (remoteproc_core.c `rproc_boot`, `rproc_start`; NOT captured from a phone).
    const SOURCE_UP: &str = "[    4.1] remoteproc remoteproc0: powering up slpi\n[    4.9] remoteproc remoteproc0: remote processor slpi is now up";
    /// remoteproc_sysfs.c state words and the `name` file's `slpi` (qcom_q6v5_pas.c); NOT captured.
    const SOURCE_STATE_RUNNING: &str = "slpi running\nmodem offline";
    /// lookup.c: the header, then `%9u %7u %8u %4u %5u` per service; the values are made up, the shape is the source's.
    const SOURCE_TABLE: &str = "  Service Version Instance Node  Port\n       15       1        0    9     2\n       16       1        0    9     3";

    fn read(rproc: &str, dmesg: &str, qrtr: &str) -> String {
        format!("---remoteproc\n{rproc}\n---dmesg\n{dmesg}\n---qrtr\n{qrtr}\n---end")
    }

    #[test]
    fn a_captured_log_without_the_slpi_says_it_never_started() {
        // source: the captured kernel log; no slpi line is in it (the glink_ssr line has no failure word)
        let log = log_says(CAPTURED_LOG);
        assert_eq!(log, Log::default());
    }

    #[test]
    fn the_kernels_own_words_say_started_and_up() {
        let log = log_says(SOURCE_UP);
        assert!(log.started && log.up && log.failures.is_empty());
    }

    #[test]
    fn a_failure_line_of_the_driver_or_its_transport_is_kept() {
        let log = log_says("remoteproc remoteproc0: powering up slpi\nqcom_q6v5_pas 5800000.remoteproc: start timed out\nunrelated driver: error 5");
        assert!(log.started && !log.up);
        assert_eq!(log.failures.len(), 1, "{:?}", log.failures);
        let log = log_says("remoteproc remoteproc0: request_firmware failed: -2");
        assert_eq!(log.failures.len(), 1);
    }

    #[test]
    fn the_table_counts_its_rows_and_a_missing_table_says_what_the_tool_said() {
        assert_eq!(services_in(&read("", "", SOURCE_TABLE)), Ok(2));
        assert_eq!(
            services_in(&read("", "", "  Service Version Instance Node  Port")),
            Ok(0)
        );
        let e = services_in(&read(
            "",
            "",
            "qrtr-lookup: socket: Address family not supported by protocol",
        ))
        .unwrap_err();
        assert!(
            e.contains("printed no table") && e.contains("Address family"),
            "{e}"
        );
        assert!(services_in("---remoteproc\n---end").is_err());
    }

    #[test]
    fn the_states_are_read_by_name() {
        let r = remoteprocs_in(&read(SOURCE_STATE_RUNNING, "", "")).unwrap();
        assert_eq!(
            r,
            [
                ("slpi".to_string(), State::Running),
                ("modem".to_string(), State::Offline)
            ]
        );
        assert!(remoteprocs_in(&read("slpi", "", "")).is_err());
        assert!(remoteprocs_in("nothing").is_err());
        assert_eq!(state_of("suspended"), State::Other("suspended".into()));
    }

    #[test]
    fn a_running_slpi_with_services_is_said_up_in_every_part() {
        let a = judge(&read(SOURCE_STATE_RUNNING, SOURCE_UP, SOURCE_TABLE)).unwrap();
        assert_eq!(a.slpi, Some(State::Running));
        assert_eq!(a.services, Ok(2));
        assert_eq!(
            a.said(),
            "remoteproc slpi is running; the kernel says `remote processor slpi is now up`; no failure line; qrtr-lookup lists 2 service(s)"
        );
    }

    #[test]
    fn an_absent_slpi_is_said_so_with_the_names_listed_and_a_tool_that_failed_is_not_known() {
        let a = judge(&read("modem offline", "", "qrtr-lookup: not found")).unwrap();
        assert_eq!(a.slpi, None);
        let said = a.said();
        assert!(
            said.starts_with("no remoteproc is named slpi (the ones listed: modem)"),
            "{said}"
        );
        assert!(
            said.contains("not known yet: qrtr-lookup printed no table"),
            "{said}"
        );
        assert!(said.contains("does not say it started slpi"), "{said}");
    }

    #[test]
    fn a_crashed_or_offline_slpi_is_an_answer() {
        let a = judge(&read("slpi crashed", "remoteproc remoteproc0: powering up slpi\nremoteproc remoteproc0: crash detected in slpi: type fatal error", "")).unwrap();
        assert_eq!(a.slpi, Some(State::Crashed));
        assert_eq!(a.log.failures.len(), 1);
        assert!(a.said().starts_with("remoteproc slpi CRASHED; the kernel says it powered slpi up and never that it is up; 1 failure line(s)"));
        assert_eq!(
            judge(&read("slpi offline", "", "")).unwrap().slpi,
            Some(State::Offline)
        );
    }

    #[test]
    fn the_read_command_has_the_marks_the_judge_reads() {
        for part in ["remoteproc", "dmesg", "qrtr", "end"] {
            assert!(READ_COMMAND.contains(&format!("echo ---{part}")), "{part}");
        }
        assert!(READ_COMMAND.contains("/sys/class/remoteproc/*"));
        assert!(READ_COMMAND.contains("qrtr-lookup"));
    }

    /// Read from the entry files themselves: the real run boots, the rehearsal does not; neither reads a go or sends the reboot tool; both end in the shared way back.
    #[test]
    fn the_entries_boot_only_in_the_real_run_and_read_no_go() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../procedures/");
        let real = std::fs::read_to_string(format!("{dir}slpi-boot-on-the-pixel-2.rs")).unwrap();
        let rehearsal =
            std::fs::read_to_string(format!("{dir}rehearse-slpi-boot-on-the-pixel-2.rs")).unwrap();
        assert!(real.contains("boot_the_image") && real.contains("the_slpi_is_read"));
        assert!(!rehearsal.contains("boot_the_image") && !rehearsal.contains("the_slpi_is_read"));
        for text in [&real, &rehearsal] {
            for banned in ["author_has_said_go", "set_active", "the_guard_holds"] {
                assert!(!text.contains(banned), "{banned}");
            }
            assert!(text.contains("b.way_back()"));
            assert!(text.contains("the_image_is_the_declared_ufs_off_image"));
        }
        assert!(real.contains("Settings::for_slpi_boot()"));
        assert!(rehearsal.contains("Settings::for_rehearse_slpi_boot()"));
    }
}

#[cfg(test)]
mod review_tests {
    use super::*;

    fn read(rproc: &str, dmesg: &str, qrtr: &str) -> String {
        format!("---remoteproc\n{rproc}\n---dmesg\n{dmesg}\n---qrtr\n{qrtr}\n---end")
    }

    #[test]
    fn a_diag_row_with_na_version_is_counted() {
        // shape from the pinned qrtr's lookup.c line 240: `%9u %s %8u %4u %5u %s (%s)`; the values are typed
        let t = "  Service Version Instance Node  Port\n       15       1        0    9     2\n     4097 N/A        0    9     4 DIAG (x)";
        assert_eq!(services_in(&read("", "", t)), Ok(2));
    }

    #[test]
    fn the_driver_faults_of_the_source_are_failures() {
        // format strings of qcom_q6v5_pas.c and qcom_q6v5.c, filled by hand (not captured)
        for l in [
            "qcom_q6v5_pas 5800000.remoteproc: unable to resolve memory-region",
            "remoteproc remoteproc0: watchdog received: x",
            "remoteproc remoteproc0: watchdog without message",
        ] {
            assert_eq!(log_says(l).failures.len(), 1, "{l}");
        }
    }

    #[test]
    fn a_read_command_guards_the_empty_glob() {
        assert!(READ_COMMAND.contains("[ -e $d/name ] || continue"));
        // busybox prints the empty part as nothing: no remoteproc at all is an answer
        let a = judge(&read("", "", "x")).unwrap();
        assert_eq!(a.slpi, None);
    }
}
