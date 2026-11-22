// SPDX-License-Identifier: MIT OR Apache-2.0
//! Why the getty on tty1 prints no login prompt although it runs (item walleye-display-alpine-getty-experiments): the READ-ONLY readings of each getty,
//! and the live experiments the lab runs on the booted image when the screen check fails. Pure data and parsing; the ssh calls are the ramroot try's.
//! Nothing here touches storage: the root of the image is a tmpfs, and an experiment that changes the inittab puts it back.

use crate::ramroot::screen_has_login_prompt;
use crate::rootfs_boot::sections;

/// Read-only: for each getty pid its wchan, kernel stack, open files and tty; the tty1 line discipline and nodes; the issue, hostname, utmp and the
/// init's log. `[g]etty` is a pattern that does not match its own command line.
pub const READINGS: &str = "for p in $(ps -o pid,args | awk '/[g]etty/{print $1}'); do echo \"---getty-$p\"; ps -o pid,args | awk -v p=$p '$1==p'; echo wchan:; cat /proc/$p/wchan 2>&1; echo; echo stack:; cat /proc/$p/stack 2>&1; echo fd:; ls -l /proc/$p/fd 2>&1; echo stat-tty:; awk '{print $7}' /proc/$p/stat 2>&1; done; echo ---stty; stty -F /dev/tty1 -a 2>&1; echo ---nodes; ls -l /dev/tty1 /dev/console /dev/vcs* 2>&1; echo ---issue; cat /etc/issue 2>&1; echo ---hostname; cat /etc/hostname 2>&1; echo ---utmp; ls -l /var/run/utmp /run/utmp 2>&1; echo ---openrc; ls /run/openrc 2>&1; echo ---dmesg-tail; dmesg | tail -n 40; echo ---end";

/// One live experiment: a name, what it asks, and the commands that start it (the screen is read 2 s later; [`CLEANUP`] ends it).
pub struct Experiment {
    pub name: &'static str,
    pub asks: &'static str,
    pub start: &'static str,
}

/// Run after every experiment: the getty started by hand (its pid is in /run/exp.pid; matching ps text would hit the ssh shell itself) ends.
pub const CLEANUP: &str =
    "kill $(cat /run/exp.pid 2>/dev/null) 2>/dev/null; rm -f /run/exp.pid; true";

/// The inittab without its tty1 line, so init does not respawn a getty over the experiment; the original is kept in /run (a tmpfs) for [`RESTORE`].
pub const FREE_TTY1: &str = "cp /etc/inittab /run/inittab.orig && grep -v '^tty1::' /run/inittab.orig > /etc/inittab && kill -HUP 1; sleep 1; kill $(ps -o pid,args | awk '/[g]etty.*tty1/{print $1}') 2>/dev/null; true";

/// The inittab as it was, read again by init.
pub const RESTORE: &str =
    "[ -f /run/inittab.orig ] && cat /run/inittab.orig > /etc/inittab; kill -HUP 1; true";

/// The experiments in order: (a) and (b) run with init's own getty in place, (c) to (f) after [`FREE_TTY1`].
pub const EXPERIMENTS: [Experiment; 6] = [
    Experiment {
        name: "a-write-to-tty1",
        asks: "does the VT accept a write and show it",
        start: "echo walleye-marker-a > /dev/tty1",
    },
    Experiment {
        name: "b-respawn",
        asks: "does init's respawned getty print",
        start: "kill $(ps -o pid,args | awk '/[g]etty 38400 tty1/{print $1}') 2>/dev/null; sleep 1",
    },
    Experiment {
        name: "c-getty-L",
        asks: "getty -L 38400 tty1 by hand, output to /dev/tty1",
        start: "(setsid /sbin/getty -L 38400 tty1 </dev/tty1 >/dev/tty1 2>&1 & echo $! > /run/exp.pid)",
    },
    Experiment {
        name: "d-getty-login",
        asks: "getty -n -l /bin/login 38400 tty1 linux",
        start: "(setsid /sbin/getty -n -l /bin/login 38400 tty1 linux </dev/tty1 >/dev/tty1 2>&1 & echo $! > /run/exp.pid)",
    },
    Experiment {
        name: "e-agetty",
        asks: "agetty --noclear tty1 linux",
        start: "if [ -x /sbin/agetty ]; then (setsid /sbin/agetty --noclear tty1 linux </dev/tty1 >/dev/tty1 2>&1 & echo $! > /run/exp.pid); else echo no agetty in the image; fi",
    },
    Experiment {
        name: "f-after-vhangup",
        asks: "getty started on stdout from /dev/tty1 after a vhangup of the console (the vhangup program when the image has it, else none: then it is c without -L)",
        start: "(setsid sh -c 'exec </dev/tty1 >/dev/tty1 2>&1; if command -v vhangup >/dev/null; then vhangup; fi; exec /sbin/getty 38400 tty1' & echo $! > /run/exp.pid)",
    },
];

/// The experiments that run after the inittab is freed of its tty1 line.
pub const FIRST_FREED: usize = 2;

/// The whole command of one experiment: start it, wait 2 s, read the screen's text and the getties, clean up.
pub fn command_of(e: &Experiment) -> String {
    format!(
        "printf '\\033c' >/dev/tty1; {}; sleep 2; echo ---ps; ps -o pid,args | awk '/[g]etty/'; {SCREEN_COMMAND_BODY}; {CLEANUP}",
        e.start
    )
}

/// The screen read of an experiment: the hostname, then the active VT and its text (/dev/vcs), in the sections [`shown_by`] reads.
const SCREEN_COMMAND_BODY: &str = "echo ---hostname; hostname; echo ---screen-of-the-active-vt; cat /sys/class/tty/tty0/active 2>&1; cat /dev/vcs 2>&1; echo ---end";

/// What one experiment showed.
pub struct Shown {
    pub name: &'static str,
    pub prompt: bool,
    pub text: String,
}

/// An experiment's output read: the prompt is on the screen when `HOSTNAME login:` is on a row of the text read from /dev/vcs.
pub fn shown_by(e: &Experiment, output: &str) -> Shown {
    let parts = sections(output);
    let part = |n: &str| {
        parts
            .iter()
            .find(|(k, _)| k == n)
            .map(|(_, t)| t.clone())
            .unwrap_or_default()
    };
    let (host, screen) = (part("hostname"), part("screen-of-the-active-vt"));
    Shown {
        name: e.name,
        prompt: screen_has_login_prompt(&screen, &host),
        text: output.to_string(),
    }
}

/// The finding for the log: the experiments in whose screen text `walleye login:` appeared name the cause, the first of them first.
pub fn verdict(shown: &[Shown]) -> String {
    let won: Vec<&str> = shown.iter().filter(|s| s.prompt).map(|s| s.name).collect();
    match won.first() {
        Some(first) => format!(
            "the login prompt appeared in: {}; the first, {first}, names the cause",
            won.join(", ")
        ),
        None => "no experiment made the login prompt appear in the screen's text: the cause is not named yet; read getty-readings.txt".to_string(),
    }
}

/// The text kept in getty-experiments.txt: each experiment with what it asks, what it said and the verdict.
pub fn log_of(shown: &[Shown]) -> String {
    let mut t = String::new();
    for (e, s) in EXPERIMENTS.iter().zip(shown) {
        t.push_str(&format!(
            "=== {} ({}): prompt {}\n{}\n",
            e.name,
            e.asks,
            if s.prompt { "SHOWN" } else { "not shown" },
            s.text.trim_end()
        ));
    }
    t.push_str(&format!("=== verdict: {}\n", verdict(shown)));
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ramroot::SCREEN_COMMAND;

    fn output(host: &str, screen: &str) -> String {
        format!("---ps\n 1 getty\n---hostname\n{host}\n---screen-of-the-active-vt\ntty1\n{screen}\n---end\n")
    }

    #[test]
    fn six_experiments_in_the_briefs_order_and_two_run_before_tty1_is_freed() {
        let names: Vec<&str> = EXPERIMENTS.iter().map(|e| e.name).collect();
        assert_eq!(
            names,
            [
                "a-write-to-tty1",
                "b-respawn",
                "c-getty-L",
                "d-getty-login",
                "e-agetty",
                "f-after-vhangup"
            ]
        );
        assert_eq!(FIRST_FREED, 2);
    }

    #[test]
    fn the_prompt_on_the_screen_names_the_experiment_that_made_it() {
        let shown = vec![
            shown_by(&EXPERIMENTS[0], &output("walleye", "walleye-marker-a")),
            shown_by(&EXPERIMENTS[2], &output("walleye", "walleye login: ")),
            shown_by(&EXPERIMENTS[3], &output("walleye", "walleye login: ")),
        ];
        assert!(!shown[0].prompt && shown[1].prompt);
        let v = verdict(&shown);
        assert!(
            v.contains("c-getty-L, d-getty-login") && v.contains("the first, c-getty-L"),
            "{v}"
        );
    }

    #[test]
    fn without_a_prompt_the_cause_is_not_named_yet() {
        let shown = vec![shown_by(&EXPERIMENTS[0], &output("walleye", "x"))];
        assert!(verdict(&shown).contains("not named yet"));
    }

    #[test]
    fn the_log_keeps_each_experiment_and_the_verdict() {
        let shown: Vec<Shown> = EXPERIMENTS
            .iter()
            .map(|e| shown_by(e, &output("walleye", "walleye login:")))
            .collect();
        let log = log_of(&shown);
        for e in &EXPERIMENTS {
            assert!(log.contains(e.name), "{log}");
        }
        assert!(log.contains("=== verdict:"));
    }

    #[test]
    fn every_command_the_phone_runs_is_valid_shell_and_the_readings_change_nothing() {
        let mut all: Vec<String> = EXPERIMENTS.iter().map(command_of).collect();
        all.extend([
            READINGS.to_string(),
            FREE_TTY1.to_string(),
            RESTORE.to_string(),
            SCREEN_COMMAND.to_string(),
        ]);
        for c in &all {
            let ok = std::process::Command::new("sh")
                .args(["-n", "-c", c])
                .status()
                .unwrap();
            assert!(ok.success(), "{c}");
        }
        for word in [" > ", ">>", "kill", "rm ", "mkfs", "mount", "dd "] {
            assert!(
                !READINGS.contains(word),
                "the readings must not contain {word:?}"
            );
        }
    }

    #[test]
    fn cleanup_ends_the_started_getty_by_its_pid_file_and_never_by_matching_ps_text() {
        assert!(!CLEANUP.contains("ps ") && CLEANUP.contains("/run/exp.pid"));
        for e in EXPERIMENTS.iter().skip(FIRST_FREED) {
            assert!(e.start.contains("echo $! > /run/exp.pid"), "{}", e.name);
        }
    }

    #[test]
    fn no_experiment_names_a_storage_device() {
        for e in &EXPERIMENTS {
            assert!(!command_of(e).contains("/dev/sd") && !command_of(e).contains("/dev/block"));
        }
    }
}
