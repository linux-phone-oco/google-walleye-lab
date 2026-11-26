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
pub fn dry_run_lines(index: u32) -> Vec<String> {
    let hex = |m: Vec<u8>| m.iter().map(|b| format!("{b:02x}")).collect::<String>();
    vec![
        format!("walleye-nfc-up: dry run: 1. ask the controller for family \"nfc\": {}", hex(family_request(1))),
        format!("walleye-nfc-up: dry run: 2. NFC_CMD_DEV_UP for nfc{index} (family id 0 stands for the id the kernel gives): {}", hex(dev_up_request(0, index, 2))),
    ]
}

/// The real kernel: an AF_NETLINK / NETLINK_GENERIC socket.
pub struct Real;

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
