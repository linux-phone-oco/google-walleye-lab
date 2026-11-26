// SPDX-License-Identifier: MIT OR Apache-2.0
//! The touch reader: its own session to the phone, reading the touch controller's input node for a window
//! while a person touches the glass. It saves each event line the moment it arrives (a phone that restarts
//! under it leaves what it read) and says when the node is open (EVENTS-READY) - before anyone is asked.

use crate::phone::{spelled, Shell};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct State {
    /// the node was readable just before the reading began
    pub ready: bool,
    /// the window closed (or the phone hung up)
    pub done: bool,
    /// event lines saved so far (a line is one 24-byte input_event, as `od -w24` prints it)
    pub lines: usize,
    pub error: Option<String>,
}

pub struct Reader {
    state: Arc<(Mutex<State>, Condvar)>,
    handle: Option<JoinHandle<()>>,
}

/// An `od -tx1 -w24` line: 24 two-digit hex bytes.
pub fn is_event_line(l: &str) -> bool {
    let t: Vec<&str> = l.split_whitespace().collect();
    t.len() == 24
        && t.iter()
            .all(|b| b.len() == 2 && b.bytes().all(|c| c.is_ascii_hexdigit()))
}

impl Reader {
    /// Open the node `device` for `window` seconds, saving event lines to `events`.
    pub fn start(address: &str, port: u16, device: &str, window: u64, events: PathBuf) -> Reader {
        let state = Arc::new((Mutex::new(State::default()), Condvar::new()));
        let st = state.clone();
        let (address, device) = (address.to_string(), device.to_string());
        let handle = std::thread::spawn(move || {
            let set = |f: &dyn Fn(&mut State)| {
                let mut g = st.0.lock().unwrap_or_else(|e| e.into_inner());
                f(&mut g);
                st.1.notify_all();
            };
            let outcome = (|| -> Result<(), String> {
                let mut file =
                    File::create(&events).map_err(|e| format!("{}: {e}", events.display()))?;
                let mut sh = Shell::open(&address, port, Duration::from_secs(5))?;
                let command = format!(
                    "[ -r {device} ] && echo {ready}; timeout {window} od -An -v -tx1 -w24 {device}",
                    ready = spelled("EVENTS-READY")
                );
                sh.stream(
                    &command,
                    "EVENTS-END",
                    Instant::now() + Duration::from_secs(window + 15),
                    |l| {
                        if l == "EVENTS-READY" {
                            set(&|s| s.ready = true);
                        } else if is_event_line(l) {
                            let _ = writeln!(file, "{}", l.trim());
                            let _ = file.flush();
                            set(&|s| s.lines += 1);
                        }
                    },
                )
                .and_then(|ended| if ended { Ok(()) } else { Err("the session ended before the reader's end line (phone gone or window overrun)".to_string()) })
            })();
            set(&|s| {
                s.done = true;
                s.error = outcome.clone().err();
            });
        });
        Reader {
            state,
            handle: Some(handle),
        }
    }

    pub fn state(&self) -> State {
        self.state
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Wait until the node is open, the reader ended, or `wait` passed; the state then.
    pub fn wait_ready(&self, wait: Duration) -> State {
        let end = Instant::now() + wait;
        let mut g = self.state.0.lock().unwrap_or_else(|e| e.into_inner());
        while !g.ready && !g.done {
            let left = end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            g = self
                .state
                .1
                .wait_timeout(g, left)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
        g.clone()
    }

    /// Wait for the window to close (it is bounded: the phone's `timeout` and the stream's own limit).
    pub fn finish(mut self) -> State {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        self.state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phone::fake;

    fn ev(kind: &str) -> String {
        let mut b = vec!["00"; 24];
        b[16] = kind;
        b.join(" ")
    }

    #[test]
    fn event_lines_are_24_hex_bytes() {
        assert!(is_event_line(&format!(" {}", ev("03"))));
        assert!(!is_event_line("od: /dev/input/event1: No such file"));
        assert!(!is_event_line("00 00"));
    }

    #[test]
    fn the_reader_is_ready_first_then_saves_every_event_and_ends() {
        let port = fake::phone(vec![], vec![ev("03"), ev("03"), "garbage".into(), ev("00")]);
        let d = std::env::temp_dir().join(format!("wt-reader-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let file = d.join("events.txt");
        let r = Reader::start("127.0.0.1", port, "/dev/input/event1", 5, file.clone());
        let s = r.wait_ready(Duration::from_secs(5));
        assert!(s.ready, "{s:?}");
        let end = r.finish();
        assert_eq!((end.lines, end.done, end.error), (3, true, None));
        assert_eq!(std::fs::read_to_string(&file).unwrap().lines().count(), 3);
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn a_phone_that_is_not_there_ends_the_reader_with_the_reason() {
        let port = fake::closed();
        let file = std::env::temp_dir().join(format!("wt-reader-none-{}", std::process::id()));
        let r = Reader::start("127.0.0.1", port, "/dev/input/event1", 1, file.clone());
        let s = r.wait_ready(Duration::from_secs(5));
        assert!(!s.ready && s.done && s.error.is_some(), "{s:?}");
        std::fs::remove_file(file).ok();
    }
}
