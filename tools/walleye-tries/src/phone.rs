// SPDX-License-Identifier: MIT OR Apache-2.0
//! The phone's shell over the network: telnetd on the gadget's address answers a root shell on a pty with no
//! password (the lab's cable, init-z14). One command at a time, its answer between two markers; nothing here
//! writes to the phone beyond what the caller's command does.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// A marker the shell's echo of a command line cannot be mistaken for: the command spells it split.
pub fn spelled(marker: &str) -> String {
    format!("{}''{}", &marker[..1], &marker[1..])
}

/// One telnet session to the phone.
pub struct Shell {
    reader: BufReader<TcpStream>,
    n: u32,
}

/// Telnet option negotiation (IAC ...) and the pty's control bytes are not the answer.
pub fn clean(raw: &[u8]) -> String {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        match raw[i] {
            255 if i + 1 < raw.len() && raw[i + 1] == 255 => {
                out.push(255);
                i += 2;
            }
            255 => {
                i += if i + 2 < raw.len() && (251..=254).contains(&raw[i + 1]) {
                    3
                } else {
                    2
                }
            }
            b'\r' | 0 => i += 1,
            b if b < 32 && b != b'\n' && b != b'\t' => i += 1,
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

impl Shell {
    /// Connect, bounded by `wait`.
    pub fn open(address: &str, port: u16, wait: Duration) -> Result<Shell, String> {
        let target = (address, port)
            .to_socket_addrs()
            .map_err(|e| format!("{address}:{port}: {e}"))?
            .next()
            .ok_or_else(|| format!("{address}:{port}: no address"))?;
        let s = TcpStream::connect_timeout(&target, wait)
            .map_err(|e| format!("the phone's shell at {address}:{port} does not answer: {e}"))?;
        s.set_read_timeout(Some(Duration::from_millis(500))).ok();
        Ok(Shell {
            reader: BufReader::new(s),
            n: 0,
        })
    }

    fn send(&mut self, line: &str) -> Result<(), String> {
        let s = self.reader.get_mut();
        s.write_all(line.as_bytes())
            .and_then(|_| s.write_all(b"\n"))
            .map_err(|e| format!("write to the phone: {e}"))
    }

    /// Read lines as they come until `done` says a line ends it, `until` passes, or the phone hangs up.
    /// Every cleaned line goes to `on_line` first. Returns whether `done` was seen.
    pub fn read_lines(
        &mut self,
        until: Instant,
        mut on_line: impl FnMut(&str),
        done: impl Fn(&str) -> bool,
    ) -> bool {
        let mut buf = Vec::new();
        while Instant::now() < until {
            match self.reader.read_until(b'\n', &mut buf) {
                Ok(0) => return false,
                Ok(_) if buf.last() == Some(&b'\n') => {
                    let line = clean(&buf);
                    buf.clear();
                    let line = line.trim_end_matches('\n');
                    on_line(line);
                    if done(line) {
                        return true;
                    }
                }
                Ok(_) => {}
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(_) => return false,
            }
        }
        false
    }

    /// Run a command and return what it printed, bounded by `wait`.
    pub fn run(&mut self, command: &str, wait: Duration) -> Result<String, String> {
        self.n += 1;
        let (b, e) = (format!("WT-BEGIN-{}", self.n), format!("WT-END-{}", self.n));
        // the markers are built from parts so the shell's echo of this line is not taken for them
        let line = format!(
            "echo WT-BEGIN-''{n}; {command}; echo WT-END-''{n}",
            n = self.n
        );
        self.send(&line)?;
        let mut lines = Vec::new();
        let mut inside = false;
        let seen = self.read_lines(
            Instant::now() + wait,
            |l| {
                if l == b {
                    inside = true;
                } else if l != e && inside {
                    lines.push(l.to_string());
                }
            },
            |l| l == e,
        );
        if seen {
            Ok(lines.join("\n"))
        } else {
            Err(format!(
                "`{command}` was not answered within {} s",
                wait.as_secs()
            ))
        }
    }

    /// Start a command whose output streams (no end marker of its own): `on_line` sees each line, until the
    /// command's own end line `end` or `until`.
    pub fn stream(
        &mut self,
        command: &str,
        end: &str,
        until: Instant,
        on_line: impl FnMut(&str),
    ) -> Result<bool, String> {
        self.send(&format!("{command}; echo {}", spelled(end)))?;
        let want = end.to_string();
        Ok(self.read_lines(until, on_line, move |l| l == want))
    }
}

#[cfg(test)]
pub(crate) mod fake {
    //! A stand-in phone: a thread that answers the shell's command lines from a table, and streams lines.
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    /// A port where something listens that hangs up at once: the phone is not there, and no other test can take it.
    pub fn closed() -> u16 {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for c in l.incoming() {
                drop(c)
            }
        });
        port
    }

    /// Answers `cmd` lines. A line `echo WT-BEGIN-''N; CMD; echo WT-END-''N` answers with its markers around
    /// the answer of the first table row whose key `CMD` contains; a line with `od -An` streams `events`.
    pub fn phone(table: Vec<(&'static str, &'static str)>, events: Vec<String>) -> u16 {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for conn in l.incoming() {
                let Ok(mut c) = conn else { continue };
                let table = table.clone();
                let events = events.clone();
                std::thread::spawn(move || {
                    let _ = c.write_all(&[255, 253, 1, b'\r', 0]); // negotiation noise first
                    let mut r = BufReader::new(c.try_clone().unwrap());
                    let mut line = String::new();
                    while r.read_line(&mut line).unwrap_or(0) > 0 {
                        let l = line.trim().to_string();
                        line.clear();
                        if let Some(rest) = l.strip_prefix("echo WT-BEGIN-''") {
                            let (n, cmd) = rest.split_once("; ").unwrap();
                            let (cmd, _) = cmd.rsplit_once("; echo WT-END").unwrap();
                            let _ = write!(c, "WT-BEGIN-{n}\r\n");
                            let answer = table
                                .iter()
                                .find(|(k, _)| cmd.contains(k))
                                .map(|(_, v)| *v)
                                .unwrap_or("");
                            for a in answer.lines() {
                                let _ = write!(c, "{a}\r\n");
                            }
                            let _ = write!(c, "WT-END-{n}\r\n");
                        } else if l.contains("od -An") {
                            let end = l.rsplit("echo ").next().unwrap_or("").replace("''", "");
                            let _ = write!(c, "EVENTS-READY\r\n");
                            for e in &events {
                                let _ = write!(c, "{e}\r\n");
                            }
                            let _ = write!(c, "{end}\r\n");
                        }
                    }
                });
            }
        });
        port
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_bytes_and_negotiation_are_not_the_answer() {
        assert_eq!(clean(&[255, 253, 1, b'o', b'k', b'\r', b'\n']), "ok\n");
        assert_eq!(clean(&[b'a', 255, 255, b'b']), "a\u{fffd}b");
    }

    #[test]
    fn a_command_answers_between_its_markers() {
        let port = fake::phone(vec![("uptime", "17.5 3.0\nsecond line")], vec![]);
        let mut sh = Shell::open("127.0.0.1", port, Duration::from_secs(2)).unwrap();
        let a = sh
            .run("cat /proc/uptime # uptime", Duration::from_secs(3))
            .unwrap();
        assert_eq!(a, "17.5 3.0\nsecond line");
    }

    #[test]
    fn the_marker_the_echo_cannot_be_taken_for() {
        assert_eq!(spelled("EVENTS-READY"), "E''VENTS-READY");
    }

    #[test]
    fn a_phone_that_hangs_up_is_said_plainly() {
        let port = fake::closed();
        let mut sh = Shell::open("127.0.0.1", port, Duration::from_secs(1)).unwrap();
        let e = sh.run("dmesg", Duration::from_secs(2)).unwrap_err();
        assert!(
            e.contains("not answered") || e.contains("write to the phone"),
            "{e}"
        );
    }
}
