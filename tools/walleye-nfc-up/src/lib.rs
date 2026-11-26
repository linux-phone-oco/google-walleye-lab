// SPDX-License-Identifier: MIT OR Apache-2.0
//! Open an NFC device: the generic netlink family "nfc" is looked up in the kernel's controller (CTRL_CMD_GETFAMILY), then
//! NFC_CMD_DEV_UP is sent with the device index and the kernel's answer (an ack or an errno) is returned as it came.
//! Every number below is the kernel's own, from its uapi headers (v7.3): include/uapi/linux/netlink.h, genetlink.h
//! (GENL_ID_CTRL, CTRL_*) and nfc.h (NFC_GENL_NAME, NFC_GENL_VERSION, NFC_CMD_DEV_UP, NFC_ATTR_DEVICE_INDEX).
//! The socket sits behind [`Netlink`], so a test can stand in for the kernel and see exactly what was sent.

const NLMSG_ERROR: u16 = 2;
const NLM_F_REQUEST: u16 = 1;
const NLM_F_ACK: u16 = 4;
const GENL_ID_CTRL: u16 = 0x10;
const CTRL_CMD_GETFAMILY: u8 = 3;
const CTRL_ATTR_FAMILY_ID: u16 = 1;
const CTRL_ATTR_FAMILY_NAME: u16 = 2;
const NFC_GENL_NAME: &str = "nfc";
const NFC_GENL_VERSION: u8 = 1;
const NFC_CMD_DEV_UP: u8 = 2;
const NFC_ATTR_DEVICE_INDEX: u16 = 1;
const NLMSG_HDRLEN: usize = 16;
const GENL_HDRLEN: usize = 4;

/// One request and the one answer to it: the kernel's.
pub trait Netlink {
    fn exchange(&self, request: &[u8]) -> Result<Vec<u8>, String>;
}

/// What the kernel said to NFC_CMD_DEV_UP.
#[derive(Debug, PartialEq, Eq)]
pub enum Answer {
    /// an ack with error 0: the device is on
    Up,
    /// a negative error number's magnitude (EBUSY = already up, ERFKILL = blocked, ENODEV = no such device)
    Errno(i32),
}

impl Answer {
    /// The line the tool prints.
    pub fn line(&self, index: u32) -> String {
        match self {
            Answer::Up => format!("nfc{index}: NFC_CMD_DEV_UP answered: ok (the kernel acked it)"),
            Answer::Errno(e) => format!(
                "nfc{index}: NFC_CMD_DEV_UP answered: errno {e} ({})",
                std::io::Error::from_raw_os_error(*e)
            ),
        }
    }
}

fn pad4(n: usize) -> usize {
    (n + 3) & !3
}

fn message(
    kind: u16,
    flags: u16,
    seq: u32,
    cmd: u8,
    version: u8,
    attrs: &[(u16, Vec<u8>)],
) -> Vec<u8> {
    let mut body = vec![cmd, version, 0, 0];
    for (t, v) in attrs {
        body.extend(((4 + v.len()) as u16).to_ne_bytes());
        body.extend(t.to_ne_bytes());
        body.extend(v);
        body.resize(pad4(body.len()), 0);
    }
    let mut m = Vec::new();
    m.extend(((NLMSG_HDRLEN + body.len()) as u32).to_ne_bytes());
    m.extend(kind.to_ne_bytes());
    m.extend(flags.to_ne_bytes());
    m.extend(seq.to_ne_bytes());
    m.extend(0u32.to_ne_bytes()); // pid 0: the kernel
    m.extend(body);
    m
}

/// The controller's question: which id has the family "nfc"?
pub fn family_request(seq: u32) -> Vec<u8> {
    let mut name = NFC_GENL_NAME.as_bytes().to_vec();
    name.push(0);
    message(
        GENL_ID_CTRL,
        NLM_F_REQUEST,
        seq,
        CTRL_CMD_GETFAMILY,
        1,
        &[(CTRL_ATTR_FAMILY_NAME, name)],
    )
}

/// NFC_CMD_DEV_UP for the device, asking for an ack.
pub fn dev_up_request(family: u16, index: u32, seq: u32) -> Vec<u8> {
    message(
        family,
        NLM_F_REQUEST | NLM_F_ACK,
        seq,
        NFC_CMD_DEV_UP,
        NFC_GENL_VERSION,
        &[(NFC_ATTR_DEVICE_INDEX, index.to_ne_bytes().to_vec())],
    )
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_ne_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_ne_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// The error of an NLMSG_ERROR reply: 0 is an ack, else the errno; None when the reply is another message.
fn error_of(reply: &[u8]) -> Result<Option<i32>, String> {
    let kind = u16_at(reply, 4).ok_or("the reply is shorter than a netlink header")?;
    if kind != NLMSG_ERROR {
        return Ok(None);
    }
    let e = u32_at(reply, NLMSG_HDRLEN).ok_or("an error reply without its error number")? as i32;
    e.checked_neg()
        .map(Some)
        .ok_or_else(|| "an error reply with an impossible error number".to_string())
}

/// The id the controller gave the family, or the controller's error (ENOENT: the kernel has no NFC).
pub fn family_id(reply: &[u8]) -> Result<u16, String> {
    if let Some(e) = error_of(reply)? {
        return Err(format!(
            "the kernel has no family {NFC_GENL_NAME:?}: errno {e} ({})",
            std::io::Error::from_raw_os_error(e)
        ));
    }
    if u16_at(reply, 4) != Some(GENL_ID_CTRL) {
        return Err(
            "the controller answered with a message that is not a family description".into(),
        );
    }
    let len = (u32_at(reply, 0).ok_or("no length")? as usize).min(reply.len());
    let mut at = NLMSG_HDRLEN + GENL_HDRLEN;
    while at + 4 <= len {
        let alen = u16_at(reply, at).ok_or("short attribute")? as usize;
        let atype = u16_at(reply, at + 2).ok_or("short attribute")?;
        if alen < 4 {
            break;
        }
        if atype == CTRL_ATTR_FAMILY_ID {
            if alen < 6 {
                return Err("a family id attribute too short to hold the id".into());
            }
            return u16_at(reply, at + 4)
                .ok_or_else(|| "a family id attribute without its value".to_string());
        }
        at += pad4(alen);
    }
    Err("the controller's reply holds no family id".into())
}

/// The answer to NFC_CMD_DEV_UP: an ack (error 0) or an errno.
pub fn dev_up_answer(reply: &[u8]) -> Result<Answer, String> {
    match error_of(reply)? {
        Some(0) => Ok(Answer::Up),
        Some(e) => Ok(Answer::Errno(e)),
        None => Err("the kernel answered NFC_CMD_DEV_UP with a message that is not an ack".into()),
    }
}

/// The whole step: find the family, send DEV_UP, return the kernel's answer.
pub fn open_device(net: &dyn Netlink, index: u32) -> Result<Answer, String> {
    let family = family_id(&net.exchange(&family_request(1))?)?;
    dev_up_answer(&net.exchange(&dev_up_request(family, index, 2))?)
}

/// The two requests as hex, for --dry-run (the family id shown is a placeholder: it is the kernel's to give).
pub fn dry_run_lines(index: u32, poll_seconds: u64) -> Vec<String> {
    let hex = |m: Vec<u8>| m.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let mut lines = vec![
        format!("walleye-nfc-up: dry run: 1. ask the controller for family \"nfc\": {}", hex(family_request(1))),
        format!("walleye-nfc-up: dry run: 2. NFC_CMD_DEV_UP for nfc{index} (family id 0 stands for the id the kernel gives): {}", hex(dev_up_request(0, index, 2))),
    ];
    if poll_seconds > 0 {
        lines.push(format!("walleye-nfc-up: dry run: 3. join the group \"events\", NFC_CMD_START_POLL for nfc{index} (family id 0 as above), wait {poll_seconds} s for NFC_EVENT_TARGETS_FOUND, then NFC_CMD_STOP_POLL: {}", hex(start_poll_request(0, index, 3))));
    }
    lines
}

// The poll (include/uapi/linux/nfc.h v7.3: enum nfc_commands ... DEV_DOWN, DEP_LINK_UP, DEP_LINK_DOWN, START_POLL, STOP_POLL, GET_TARGET, EVENT_TARGETS_FOUND;
// enum nfc_attrs ... COMM_MODE, RF_MODE, DEVICE_POWERED, IM_PROTOCOLS; genetlink.h: CTRL_ATTR_MCAST_GROUPS and its group name and id).
const NFC_CMD_START_POLL: u8 = 6;
const NFC_CMD_STOP_POLL: u8 = 7;
const NFC_EVENT_TARGETS_FOUND: u8 = 9;
const NFC_ATTR_IM_PROTOCOLS: u16 = 13;
/// NFC_PROTO_*_MASK of JEWEL (1), MIFARE, FELICA, ISO14443, NFC_DEP, ISO14443_B, ISO15693 (7), or-ed: every initiator protocol the kernel knows.
const NFC_PROTO_ALL_MASK: u32 = 0xfe;
const CTRL_ATTR_MCAST_GROUPS: u16 = 7;
const CTRL_ATTR_MCAST_GRP_NAME: u16 = 1;
const CTRL_ATTR_MCAST_GRP_ID: u16 = 2;
const NLA_F_NESTED: u16 = 0x8000;
/// The longest poll: an hour (a larger number would overflow the deadline and the socket's timeout).
pub const MAX_POLL_SECONDS: u64 = 3600;
const NFC_GENL_MCAST_EVENT_NAME: &str = "events";

/// A socket that stays open for the poll: the kernel stops a poll when the socket that started it closes, and sends its events to the sockets in the group.
pub trait Session {
    /// One request, the one answer to it.
    fn exchange(&mut self, request: &[u8]) -> Result<Vec<u8>, String>;
    /// The next message of the group, or None when `wait` ran out first.
    fn next_event(&mut self, wait: std::time::Duration) -> Result<Option<Vec<u8>>, String>;
}

/// Where a [`Session`] comes from: a socket that has joined the multicast group before anything is sent.
pub trait Polling {
    fn session(&self, group: u32) -> Result<Box<dyn Session>, String>;
}

/// What the poll found.
#[derive(Debug, PartialEq, Eq)]
pub enum Polled {
    Target,
    NoTarget,
    /// no target, and the kernel refused STOP_POLL with this errno (EINVAL: the poll had already ended)
    NoTargetStopRefused(i32),
}

/// The attributes of a message between `at` and `end`, each as its type (nested flag off) and value.
fn attrs_of(b: &[u8], mut at: usize, end: usize) -> Vec<(u16, &[u8])> {
    let end = end.min(b.len());
    let mut out = Vec::new();
    while at + 4 <= end {
        let (Some(len), Some(kind)) = (u16_at(b, at), u16_at(b, at + 2)) else {
            break;
        };
        let len = len as usize;
        if len < 4 || at + len > end {
            break;
        }
        out.push((kind & !NLA_F_NESTED, &b[at + 4..at + len]));
        at += pad4(len);
    }
    out
}

fn msg_len(b: &[u8]) -> usize {
    u32_at(b, 0).map_or(0, |l| l as usize)
}

/// The id of the multicast group "events" in the controller's family reply.
pub fn events_group(reply: &[u8]) -> Result<u32, String> {
    let top = attrs_of(reply, NLMSG_HDRLEN + GENL_HDRLEN, msg_len(reply));
    let (_, groups) = top
        .iter()
        .find(|(t, _)| *t == CTRL_ATTR_MCAST_GROUPS)
        .ok_or("the family has no multicast groups")?;
    attrs_of(groups, 0, groups.len())
        .iter()
        .find_map(|(_, group)| {
            let a = attrs_of(group, 0, group.len());
            let name = a.iter().find(|(t, _)| *t == CTRL_ATTR_MCAST_GRP_NAME)?.1;
            let id = a.iter().find(|(t, _)| *t == CTRL_ATTR_MCAST_GRP_ID)?.1;
            (name.strip_suffix(&[0]).unwrap_or(name) == NFC_GENL_MCAST_EVENT_NAME.as_bytes())
                .then(|| u32_at(id, 0))?
        })
        .ok_or_else(|| format!("the family has no multicast group {NFC_GENL_MCAST_EVENT_NAME:?}"))
}

/// NFC_CMD_START_POLL for the device with every protocol, asking for an ack.
pub fn start_poll_request(family: u16, index: u32, seq: u32) -> Vec<u8> {
    message(
        family,
        NLM_F_REQUEST | NLM_F_ACK,
        seq,
        NFC_CMD_START_POLL,
        NFC_GENL_VERSION,
        &[
            (NFC_ATTR_DEVICE_INDEX, index.to_ne_bytes().to_vec()),
            (
                NFC_ATTR_IM_PROTOCOLS,
                NFC_PROTO_ALL_MASK.to_ne_bytes().to_vec(),
            ),
        ],
    )
}

/// NFC_CMD_STOP_POLL for the device, asking for an ack.
pub fn stop_poll_request(family: u16, index: u32, seq: u32) -> Vec<u8> {
    message(
        family,
        NLM_F_REQUEST | NLM_F_ACK,
        seq,
        NFC_CMD_STOP_POLL,
        NFC_GENL_VERSION,
        &[(NFC_ATTR_DEVICE_INDEX, index.to_ne_bytes().to_vec())],
    )
}

/// Is the message NFC_EVENT_TARGETS_FOUND of this family for this device?
pub fn is_targets_found(msg: &[u8], family: u16, index: u32) -> bool {
    u16_at(msg, 4) == Some(family)
        && msg.get(NLMSG_HDRLEN) == Some(&NFC_EVENT_TARGETS_FOUND)
        && attrs_of(msg, NLMSG_HDRLEN + GENL_HDRLEN, msg_len(msg))
            .iter()
            .any(|(t, v)| *t == NFC_ATTR_DEVICE_INDEX && u32_at(v, 0) == Some(index))
}

/// The poll after DEV_UP: join the events group, START_POLL, wait up to `seconds` for TARGETS_FOUND, STOP_POLL.
/// The kernel's own answer to START_POLL is an Err when it is an errno (the poll never ran).
pub fn poll(
    net: &dyn Netlink,
    polling: &dyn Polling,
    index: u32,
    seconds: u64,
) -> Result<Polled, String> {
    if seconds > MAX_POLL_SECONDS {
        return Err(format!(
            "--poll {seconds} is more than the {MAX_POLL_SECONDS} s a poll may last"
        ));
    }
    let reply = net.exchange(&family_request(1))?;
    let family = family_id(&reply)?;
    let mut s = polling.session(events_group(&reply)?)?; // joined before START_POLL: no event can pass unseen
    match dev_up_answer(&s.exchange(&start_poll_request(family, index, 3))?)? {
        Answer::Up => {}
        Answer::Errno(e) => {
            return Err(format!(
                "nfc{index}: NFC_CMD_START_POLL answered: errno {e} ({})",
                std::io::Error::from_raw_os_error(e)
            ))
        }
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    let found = loop {
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        if left.is_zero() {
            break Polled::NoTarget;
        }
        match s.next_event(left)? {
            None => break Polled::NoTarget,
            Some(m) if is_targets_found(&m, family, index) => break Polled::Target,
            Some(_) => {}
        }
    };
    if found == Polled::NoTarget {
        // a found target ends the poll in the kernel itself; only a poll that is still running is stopped
        if let Answer::Errno(e) = dev_up_answer(&s.exchange(&stop_poll_request(family, index, 4))?)?
        {
            return Ok(Polled::NoTargetStopRefused(e));
        }
    }
    Ok(found)
}

/// The real kernel: an AF_NETLINK / NETLINK_GENERIC socket.
pub struct Real;

/// One socket held open, a member of one multicast group.
struct RealSession {
    fd: libc::c_int,
    /// events that came between a request and its answer: next_event gives them first
    pending: std::collections::VecDeque<Vec<u8>>,
}

impl Drop for RealSession {
    fn drop(&mut self) {
        // SAFETY: the descriptor is this value's own
        unsafe { libc::close(self.fd) };
    }
}

impl RealSession {
    fn timeout(&self, wait: std::time::Duration) -> Result<(), String> {
        let wait = wait.max(std::time::Duration::from_millis(1)); // a zero timeval would block for ever
        let tv = libc::timeval {
            tv_sec: wait.as_secs() as libc::time_t,
            tv_usec: wait.subsec_micros() as libc::suseconds_t,
        };
        // SAFETY: a plain setsockopt on this socket; tv outlives the call
        let rc = unsafe {
            libc::setsockopt(
                self.fd,
                libc::SOL_SOCKET,
                libc::SO_RCVTIMEO,
                &tv as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::timeval>() as u32,
            )
        };
        (rc >= 0).then_some(()).ok_or_else(|| {
            format!(
                "netlink receive timeout: {}",
                std::io::Error::last_os_error()
            )
        })
    }

    /// One receive; None when the timeout ran out.
    fn receive(&self) -> Result<Option<Vec<u8>>, String> {
        let mut buf = vec![0u8; 8192];
        loop {
            // SAFETY: buf outlives the call
            let n =
                unsafe { libc::recv(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0) };
            if n >= 0 {
                buf.truncate(n as usize);
                return Ok(Some(buf));
            }
            let e = std::io::Error::last_os_error();
            match e.kind() {
                std::io::ErrorKind::Interrupted => continue,
                std::io::ErrorKind::WouldBlock => return Ok(None),
                _ => return Err(format!("netlink receive: {e}")),
            }
        }
    }
}

impl Session for RealSession {
    fn exchange(&mut self, request: &[u8]) -> Result<Vec<u8>, String> {
        self.timeout(std::time::Duration::from_secs(5))?;
        // SAFETY: the buffers outlive the call
        let sent = unsafe {
            let mut to: libc::sockaddr_nl = std::mem::zeroed();
            to.nl_family = libc::AF_NETLINK as u16;
            libc::sendto(
                self.fd,
                request.as_ptr() as *const libc::c_void,
                request.len(),
                0,
                &to as *const _ as *const libc::sockaddr,
                std::mem::size_of::<libc::sockaddr_nl>() as u32,
            )
        };
        if sent < 0 {
            return Err(format!("netlink send: {}", std::io::Error::last_os_error()));
        }
        // events of the group may come between the request and its answer: the answer is the message with the request's sequence number
        let seq = u32_at(request, 8);
        loop {
            match self.receive()? {
                None => return Err("netlink receive (EAGAIN: no answer in 5 s)".into()),
                Some(m) if u32_at(&m, 8) == seq => return Ok(m),
                Some(m) => self.pending.push_back(m),
            }
        }
    }

    fn next_event(&mut self, wait: std::time::Duration) -> Result<Option<Vec<u8>>, String> {
        if let Some(m) = self.pending.pop_front() {
            return Ok(Some(m));
        }
        self.timeout(wait)?;
        self.receive()
    }
}

impl Polling for Real {
    fn session(&self, group: u32) -> Result<Box<dyn Session>, String> {
        let err = |what: &str| format!("{what}: {}", std::io::Error::last_os_error());
        // SAFETY: plain socket calls; the descriptor is owned by the RealSession from here on (closed on every path)
        unsafe {
            let fd = libc::socket(
                libc::AF_NETLINK,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                libc::NETLINK_GENERIC,
            );
            if fd < 0 {
                return Err(err("netlink socket"));
            }
            let s = RealSession {
                fd,
                pending: Default::default(),
            };
            let mut me: libc::sockaddr_nl = std::mem::zeroed();
            me.nl_family = libc::AF_NETLINK as u16;
            if libc::bind(
                fd,
                &me as *const _ as *const libc::sockaddr,
                std::mem::size_of::<libc::sockaddr_nl>() as u32,
            ) < 0
            {
                return Err(err("netlink bind"));
            }
            if libc::setsockopt(
                fd,
                libc::SOL_NETLINK,
                libc::NETLINK_ADD_MEMBERSHIP,
                &group as *const _ as *const libc::c_void,
                std::mem::size_of::<u32>() as u32,
            ) < 0
            {
                return Err(err("netlink join the events group"));
            }
            Ok(Box::new(s))
        }
    }
}

impl Netlink for Real {
    fn exchange(&self, request: &[u8]) -> Result<Vec<u8>, String> {
        let err = |what: &str| format!("{what}: {}", std::io::Error::last_os_error());
        // SAFETY: plain socket calls on a descriptor this function owns and closes; the buffers outlive each call
        unsafe {
            let fd = libc::socket(
                libc::AF_NETLINK,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                libc::NETLINK_GENERIC,
            );
            if fd < 0 {
                return Err(err("netlink socket"));
            }
            let out = (|| {
                let tv = libc::timeval {
                    tv_sec: 5,
                    tv_usec: 0,
                };
                if libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_RCVTIMEO,
                    &tv as *const _ as *const libc::c_void,
                    std::mem::size_of::<libc::timeval>() as u32,
                ) < 0
                {
                    return Err(err("netlink receive timeout"));
                }
                let mut to: libc::sockaddr_nl = std::mem::zeroed();
                to.nl_family = libc::AF_NETLINK as u16;
                if libc::sendto(
                    fd,
                    request.as_ptr() as *const libc::c_void,
                    request.len(),
                    0,
                    &to as *const _ as *const libc::sockaddr,
                    std::mem::size_of::<libc::sockaddr_nl>() as u32,
                ) < 0
                {
                    return Err(err("netlink send"));
                }
                let mut buf = vec![0u8; 8192];
                let mut n;
                loop {
                    n = libc::recv(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0);
                    if n >= 0
                        || std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted
                    {
                        break;
                    }
                }
                if n < 0 {
                    return Err(err("netlink receive (EAGAIN: no answer in 5 s)"));
                }
                buf.truncate(n as usize);
                Ok(buf)
            })();
            libc::close(fd);
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// A stand-in for the kernel: answers in order and keeps what it was sent.
    struct Stand {
        answers: RefCell<Vec<Vec<u8>>>,
        sent: RefCell<Vec<Vec<u8>>>,
    }

    impl Netlink for Stand {
        fn exchange(&self, request: &[u8]) -> Result<Vec<u8>, String> {
            self.sent.borrow_mut().push(request.to_vec());
            Ok(self.answers.borrow_mut().remove(0))
        }
    }

    /// The controller's family reply layout (genetlink.h): header, genl header, CTRL_ATTR_FAMILY_ID (u16), CTRL_ATTR_FAMILY_NAME.
    fn family_reply(id: u16) -> Vec<u8> {
        let mut name = b"nfc".to_vec();
        name.push(0);
        message(
            GENL_ID_CTRL,
            0,
            1,
            1,
            2,
            &[
                (CTRL_ATTR_FAMILY_NAME, name),
                (CTRL_ATTR_FAMILY_ID, id.to_ne_bytes().to_vec()),
            ],
        )
    }

    /// netlink.h: struct nlmsgerr starts with the error number, 0 for an ack, then the request's header.
    fn error_reply(error: i32) -> Vec<u8> {
        let mut m = Vec::new();
        m.extend(36u32.to_ne_bytes());
        m.extend(NLMSG_ERROR.to_ne_bytes());
        m.extend(0u16.to_ne_bytes());
        m.extend(2u32.to_ne_bytes());
        m.extend(0u32.to_ne_bytes());
        m.extend(error.to_ne_bytes());
        m.extend(dev_up_request(0x1e, 0, 2)[..NLMSG_HDRLEN].to_vec());
        m
    }

    #[test]
    fn the_kernel_acks_and_the_device_is_up() {
        let k = Stand {
            answers: RefCell::new(vec![family_reply(0x1e), error_reply(0)]),
            sent: RefCell::new(vec![]),
        };
        assert_eq!(open_device(&k, 0), Ok(Answer::Up));
        let sent = k.sent.borrow();
        assert_eq!(sent[0], family_request(1));
        assert_eq!(sent[1], dev_up_request(0x1e, 0, 2)); // addressed to the family id the kernel gave
    }

    #[test]
    fn an_errno_is_the_answer_as_it_came() {
        for e in [16, 19, 132] {
            let k = Stand {
                answers: RefCell::new(vec![family_reply(0x1e), error_reply(-e)]),
                sent: RefCell::new(vec![]),
            };
            let a = open_device(&k, 0).unwrap();
            assert_eq!(a, Answer::Errno(e));
            assert!(a.line(0).contains(&format!("errno {e}")));
        }
    }

    #[test]
    fn a_kernel_without_the_family_stops_before_device_up_is_sent() {
        let k = Stand {
            answers: RefCell::new(vec![error_reply(-2)]),
            sent: RefCell::new(vec![]),
        };
        let e = open_device(&k, 0).unwrap_err();
        assert!(e.contains("errno 2"), "{e}");
        assert_eq!(k.sent.borrow().len(), 1);
    }

    #[test]
    fn the_requests_are_laid_out_as_the_kernel_reads_them() {
        let m = dev_up_request(0x1e, 3, 7);
        assert_eq!(m.len(), 16 + 4 + 8);
        assert_eq!(u32_at(&m, 0), Some(28));
        assert_eq!(u16_at(&m, 4), Some(0x1e));
        assert_eq!(u16_at(&m, 6), Some(NLM_F_REQUEST | NLM_F_ACK));
        assert_eq!(u32_at(&m, 8), Some(7));
        assert_eq!(&m[16..20], &[2, 1, 0, 0]); // NFC_CMD_DEV_UP, version 1
        assert_eq!(u16_at(&m, 20), Some(8)); // attribute length
        assert_eq!(u16_at(&m, 22), Some(1)); // NFC_ATTR_DEVICE_INDEX
        assert_eq!(u32_at(&m, 24), Some(3));
        let f = family_request(1);
        assert_eq!(u16_at(&f, 4), Some(GENL_ID_CTRL));
        assert_eq!(f.len() % 4, 0);
        assert_eq!(&f[24..28], b"nfc\0");
    }

    #[test]
    fn the_numbers_are_the_headers() {
        // include/uapi/linux/nfc.h enum nfc_commands: UNSPEC, GET_DEVICE, DEV_UP; enum nfc_attrs: UNSPEC, DEVICE_INDEX
        assert_eq!((NFC_CMD_DEV_UP, NFC_ATTR_DEVICE_INDEX), (2, 1));
        assert_eq!(libc::NETLINK_GENERIC, 16);
        assert_eq!(libc::NLMSG_ERROR as u16, NLMSG_ERROR);
        assert_eq!(libc::GENL_ID_CTRL as u16, GENL_ID_CTRL);
    }

    #[test]
    fn a_reply_that_is_not_what_it_should_be_is_refused_not_misread() {
        // a family id attribute of length 4 holds no id: the next attribute's bytes are not it
        let mut short = family_reply(0x1e);
        let at = NLMSG_HDRLEN + GENL_HDRLEN + 8; // after the name attribute ("nfc\0" is 8 bytes with its header)
        short[at..at + 2].copy_from_slice(&4u16.to_ne_bytes());
        assert!(family_id(&short).unwrap_err().contains("too short"));
        // a message of another type than the controller's
        let other = message(0x1e, 0, 1, 1, 1, &[]);
        assert!(family_id(&other)
            .unwrap_err()
            .contains("not a family description"));
        // an error number that cannot be negated
        assert!(error_of(&error_reply(i32::MIN)).is_err());
        // a DEV_UP answer that is no ack
        assert!(dev_up_answer(&family_reply(0x1e)).is_err());
    }

    use std::rc::Rc;
    use std::time::Duration;

    /// genetlink.h: CTRL_ATTR_MCAST_GROUPS is a nested array of groups, each a nested pair CTRL_ATTR_MCAST_GRP_NAME / _ID.
    fn family_reply_with_events(id: u16, group: u32) -> Vec<u8> {
        let nested = |kind: u16, inner: Vec<u8>| {
            let mut v = Vec::new();
            v.extend(((4 + inner.len()) as u16).to_ne_bytes());
            v.extend((kind | NLA_F_NESTED).to_ne_bytes());
            v.extend(inner);
            v.resize(pad4(v.len()), 0);
            v
        };
        let attr = |kind: u16, value: Vec<u8>| {
            let mut v = Vec::new();
            v.extend(((4 + value.len()) as u16).to_ne_bytes());
            v.extend(kind.to_ne_bytes());
            v.extend(value);
            v.resize(pad4(v.len()), 0);
            v
        };
        let mut other = attr(CTRL_ATTR_MCAST_GRP_NAME, b"other\0".to_vec());
        other.extend(attr(CTRL_ATTR_MCAST_GRP_ID, 99u32.to_ne_bytes().to_vec()));
        let mut events = attr(CTRL_ATTR_MCAST_GRP_NAME, b"events\0".to_vec());
        events.extend(attr(CTRL_ATTR_MCAST_GRP_ID, group.to_ne_bytes().to_vec()));
        let mut groups = nested(1, other);
        groups.extend(nested(2, events));
        let mut m = family_reply(id);
        m.extend(nested(CTRL_ATTR_MCAST_GROUPS, groups));
        let len = m.len() as u32;
        m[0..4].copy_from_slice(&len.to_ne_bytes());
        m
    }

    /// An NFC_EVENT_TARGETS_FOUND of the family for a device (nfc.h: it sends NFC_ATTR_DEVICE_INDEX).
    fn targets_found(family: u16, index: u32) -> Vec<u8> {
        message(
            family,
            0,
            0,
            NFC_EVENT_TARGETS_FOUND,
            NFC_GENL_VERSION,
            &[(NFC_ATTR_DEVICE_INDEX, index.to_ne_bytes().to_vec())],
        )
    }

    /// A stand-in for the poll's socket: answers the requests in order, then hands out the events, then times out.
    struct StandSession {
        answers: Vec<Vec<u8>>,
        events: Vec<Vec<u8>>,
        sent: Rc<RefCell<Vec<Vec<u8>>>>,
        waits: Rc<RefCell<Vec<Duration>>>,
    }

    impl Session for StandSession {
        fn exchange(&mut self, request: &[u8]) -> Result<Vec<u8>, String> {
            self.sent.borrow_mut().push(request.to_vec());
            Ok(self.answers.remove(0))
        }
        fn next_event(&mut self, wait: Duration) -> Result<Option<Vec<u8>>, String> {
            self.waits.borrow_mut().push(wait);
            Ok((!self.events.is_empty()).then(|| self.events.remove(0)))
        }
    }

    struct StandPolling {
        group: RefCell<Option<u32>>,
        session: RefCell<Option<StandSession>>,
    }

    impl Polling for StandPolling {
        fn session(&self, group: u32) -> Result<Box<dyn Session>, String> {
            *self.group.borrow_mut() = Some(group);
            Ok(Box::new(self.session.borrow_mut().take().unwrap()))
        }
    }

    /// The stand-ins for one poll: the controller's reply, the socket's answers and events; what was sent and the waits asked come back.
    #[allow(clippy::type_complexity)]
    fn polled(
        start_answer: i32,
        events: Vec<Vec<u8>>,
        seconds: u64,
    ) -> (
        Result<Polled, String>,
        Option<u32>,
        Vec<Vec<u8>>,
        Vec<Duration>,
    ) {
        let net = Stand {
            answers: RefCell::new(vec![family_reply_with_events(0x1e, 77)]),
            sent: RefCell::new(vec![]),
        };
        let sent = Rc::new(RefCell::new(vec![]));
        let waits = Rc::new(RefCell::new(vec![]));
        let p = StandPolling {
            group: RefCell::new(None),
            session: RefCell::new(Some(StandSession {
                answers: vec![error_reply(start_answer), error_reply(0)],
                events,
                sent: sent.clone(),
                waits: waits.clone(),
            })),
        };
        let out = poll(&net, &p, 0, seconds);
        let (s, w) = (sent.borrow().clone(), waits.borrow().clone());
        let group = *p.group.borrow();
        (out, group, s, w)
    }

    #[test]
    fn the_poll_joins_the_events_group_then_starts_and_a_target_ends_it() {
        let (out, group, sent, _) = polled(0, vec![targets_found(0x1e, 0)], 10);
        assert_eq!(out, Ok(Polled::Target));
        assert_eq!(group, Some(77)); // the id the controller gave "events", not "other"
        assert_eq!(sent, vec![start_poll_request(0x1e, 0, 3)]); // a found target ended the poll in the kernel: no STOP_POLL
    }

    #[test]
    fn no_target_in_its_time_is_said_and_the_poll_is_stopped() {
        let (out, _, sent, waits) = polled(0, vec![], 10);
        assert_eq!(out, Ok(Polled::NoTarget));
        assert_eq!(
            sent,
            vec![
                start_poll_request(0x1e, 0, 3),
                stop_poll_request(0x1e, 0, 4)
            ]
        );
        assert!(waits[0] <= Duration::from_secs(10) && waits[0] > Duration::from_secs(9));
    }

    #[test]
    fn an_event_for_another_device_or_another_kind_is_not_a_target() {
        let other_kind = message(
            0x1e,
            0,
            0,
            10, // NFC_EVENT_DEVICE_ADDED
            NFC_GENL_VERSION,
            &[(NFC_ATTR_DEVICE_INDEX, 0u32.to_ne_bytes().to_vec())],
        );
        let (out, _, _, waits) = polled(0, vec![targets_found(0x1e, 1), other_kind], 10);
        assert_eq!(out, Ok(Polled::NoTarget));
        assert_eq!(waits.len(), 3); // both events read, then the timeout
    }

    #[test]
    fn an_errno_to_start_poll_is_the_error_and_nothing_is_waited_for() {
        let (out, _, sent, waits) = polled(-16, vec![targets_found(0x1e, 0)], 10);
        let e = out.unwrap_err();
        assert!(e.contains("NFC_CMD_START_POLL answered: errno 16"), "{e}");
        assert_eq!(sent.len(), 1);
        assert!(waits.is_empty());
    }

    #[test]
    fn a_refused_stop_poll_is_said_not_dropped_and_a_poll_too_long_is_refused() {
        // the stand-in answers STOP_POLL with errno 22 (EINVAL): the poll had ended
        let net = Stand {
            answers: RefCell::new(vec![family_reply_with_events(0x1e, 77)]),
            sent: RefCell::new(vec![]),
        };
        let p = StandPolling {
            group: RefCell::new(None),
            session: RefCell::new(Some(StandSession {
                answers: vec![error_reply(0), error_reply(-22)],
                events: vec![],
                sent: Rc::new(RefCell::new(vec![])),
                waits: Rc::new(RefCell::new(vec![])),
            })),
        };
        assert_eq!(poll(&net, &p, 0, 1), Ok(Polled::NoTargetStopRefused(22)));
        let e = poll(&net, &p, 0, u64::MAX).unwrap_err();
        assert!(e.contains("3600"), "{e}");
    }

    #[test]
    fn the_poll_numbers_are_the_headers() {
        // nfc.h enum nfc_commands: UNSPEC, GET_DEVICE, DEV_UP, DEV_DOWN, DEP_LINK_UP, DEP_LINK_DOWN, START_POLL, STOP_POLL, GET_TARGET, EVENT_TARGETS_FOUND
        assert_eq!(
            (
                NFC_CMD_START_POLL,
                NFC_CMD_STOP_POLL,
                NFC_EVENT_TARGETS_FOUND
            ),
            (6, 7, 9)
        );
        // enum nfc_attrs: UNSPEC, DEVICE_INDEX, DEVICE_NAME, PROTOCOLS, TARGET_INDEX, TARGET_SENS_RES, SEL_RES, NFCID1, SENSB_RES, SENSF_RES, COMM_MODE, RF_MODE, DEVICE_POWERED, IM_PROTOCOLS
        assert_eq!(NFC_ATTR_IM_PROTOCOLS, 13);
        // NFC_PROTO_JEWEL 1 .. NFC_PROTO_ISO15693 7, NFC_PROTO_MAX 8: bits 1 to 7
        assert_eq!(NFC_PROTO_ALL_MASK, (1 << 8) - 2);
        assert_eq!(libc::CTRL_ATTR_MCAST_GROUPS as u16, CTRL_ATTR_MCAST_GROUPS);
        assert_eq!(
            libc::CTRL_ATTR_MCAST_GRP_NAME as u16,
            CTRL_ATTR_MCAST_GRP_NAME
        );
        assert_eq!(libc::CTRL_ATTR_MCAST_GRP_ID as u16, CTRL_ATTR_MCAST_GRP_ID);
    }

    #[test]
    fn the_poll_requests_are_laid_out_as_the_kernel_reads_them() {
        let m = start_poll_request(0x1e, 3, 5);
        assert_eq!(m.len(), 16 + 4 + 8 + 8);
        assert_eq!(&m[16..20], &[6, 1, 0, 0]); // NFC_CMD_START_POLL, version 1
        assert_eq!(u16_at(&m, 22), Some(1)); // NFC_ATTR_DEVICE_INDEX
        assert_eq!(u32_at(&m, 24), Some(3));
        assert_eq!(u16_at(&m, 30), Some(13)); // NFC_ATTR_IM_PROTOCOLS
        assert_eq!(u32_at(&m, 32), Some(0xfe));
        assert_eq!(stop_poll_request(0x1e, 3, 6).len(), 16 + 4 + 8);
    }

    /// A family reply without the group is refused, not read as group 0.
    #[test]
    fn a_family_without_the_events_group_is_refused() {
        assert!(events_group(&family_reply(0x1e))
            .unwrap_err()
            .contains("no multicast"));
    }

    /// The real kernel of this host: its family "nfc" (the nfc module is loaded here) lists the group "events" with an id, in the layout the stand-in copies.
    #[test]
    fn this_hosts_kernel_names_the_nfc_events_group() {
        let reply = Real.exchange(&family_request(1)).unwrap();
        if family_id(&reply).is_err() {
            return; // a kernel without the nfc module (a CI runner): the controller says ENOENT, and there is no group to read
        }
        let group = events_group(&reply).unwrap();
        assert!(group > 0, "{group}");
        // and a socket can join it: the poll's first step on the real kernel
        let mut s = Real.session(group).unwrap();
        assert_eq!(s.next_event(Duration::from_millis(20)), Ok(None));
    }

    /// The real kernel of this host, asked the same way: the controller knows itself ("nlctrl", id 16), and a family it lacks is answered by an errno.
    #[test]
    fn this_hosts_kernel_answers_the_controllers_question() {
        let id = |name: &str| {
            let mut n = name.as_bytes().to_vec();
            n.push(0);
            Real.exchange(&message(
                GENL_ID_CTRL,
                NLM_F_REQUEST,
                1,
                CTRL_CMD_GETFAMILY,
                1,
                &[(CTRL_ATTR_FAMILY_NAME, n)],
            ))
            .unwrap()
        };
        assert_eq!(family_id(&id("nlctrl")), Ok(GENL_ID_CTRL));
        let e = family_id(&id("walleye-no-such-family")).unwrap_err();
        assert!(e.contains("errno 2"), "{e}");
    }
}
