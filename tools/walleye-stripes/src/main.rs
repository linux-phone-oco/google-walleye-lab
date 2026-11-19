// SPDX-License-Identifier: MIT OR Apache-2.0
//! walleye-stripes: the drawer of image L6. It runs from the late-probe script once the display modules are loaded:
//! read what the connector says, draw four stripes (red, green, blue, white) on the framebuffer, hold them, clear the
//! screen, read the connector again. A log line goes to standard output and to /dev/kmsg at the draw and at the release.

mod cli;

use argenv_resolve::InputResolveExt;
use cli::Contract as C;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::time::Duration;
use walleye_stripes::{frame, Channel, Geometry};

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
const FBIOPUT_VSCREENINFO: libc::c_ulong = 0x4601;
const FBIOPAN_DISPLAY: libc::c_ulong = 0x4606;
const FBIOBLANK: libc::c_ulong = 0x4611;
/// FB_ACTIVATE_NOW | FB_ACTIVATE_FORCE: set_par runs, so the display is modeset and the frame committed
const ACTIVATE_NOW_FORCE: u32 = 128;

/// struct fb_bitfield
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct FbBitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

/// struct fb_var_screeninfo
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct VarScreenInfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: FbBitfield,
    green: FbBitfield,
    blue: FbBitfield,
    transp: FbBitfield,
    rest: [u32; 20],
}

/// struct fb_fix_screeninfo (only line_length is read)
#[repr(C)]
struct FixScreenInfo {
    id: [u8; 16],
    smem_start: libc::c_ulong,
    smem_len: u32,
    kind: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: libc::c_ulong,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

struct Log {
    kmsg: Option<File>,
}

impl Log {
    fn say(&mut self, text: &str) {
        let line = format!("walleye-stripes: {text}");
        let _ = writeln!(std::io::stdout(), "{line}");
        if let Some(k) = self.kmsg.as_mut() {
            // one write is one kmsg record
            let _ = k.write_all(format!("<6>{line}\n").as_bytes());
        }
    }
}

fn main() {
    let model = C::records();
    let r = argenv_resolve::parse(&model);
    if C::HELP.get_from(&r).unwrap_or(false) {
        print!("{}", cli::usage());
        return;
    }
    let (seconds, device, connector, kmsg) = (
        C::SECONDS.get_from(&r).unwrap_or(20),
        C::DEVICE.get_from(&r).unwrap_or_else(|| "/dev/fb0".into()),
        C::CONNECTOR
            .get_from(&r)
            .unwrap_or_else(|| "/sys/class/drm/card0-DSI-1".into()),
        C::KMSG.get_from(&r).unwrap_or_else(|| "/dev/kmsg".into()),
    );
    let mut log = Log {
        kmsg: OpenOptions::new().write(true).open(&kmsg).ok(),
    };
    if let Err(e) = run(&mut log, seconds, &device, &connector) {
        log.say(&format!("FAILED: {e}"));
        std::process::exit(1);
    }
}

fn run(log: &mut Log, seconds: u64, device: &str, connector: &str) -> Result<(), String> {
    connector_is_read(log, "before the draw", connector);
    let fb = OpenOptions::new()
        .write(true)
        .open(device)
        .map_err(|e| format!("open {device}: {e}"))?;
    let geometry = geometry_is_read(&fb, device)?;
    log.say(&format!(
        "{device}: {}x{} {} bpp, line {} bytes, r{}+{} g{}+{} b{}+{}",
        geometry.xres,
        geometry.yres,
        geometry.bits_per_pixel,
        geometry.line_length,
        geometry.red.offset,
        geometry.red.length,
        geometry.green.offset,
        geometry.green.length,
        geometry.blue.offset,
        geometry.blue.length
    ));
    let drawn = frame_is_written(log, &fb, &geometry, true);
    if drawn.is_ok() {
        log.say(&format!(
            "draw: four stripes (red, green, blue, white) on {device}, held {seconds} s"
        ));
        std::thread::sleep(Duration::from_secs(seconds));
    }
    // the release runs whatever the draw did, so no stripes are left standing
    let released = frame_is_written(log, &fb, &geometry, false);
    if released.is_ok() {
        log.say("release: the screen is cleared to black");
    }
    connector_is_read(log, "after the release", connector);
    drawn.and(released)
}

/// The connector's status, enabled and modes, one log line each (a file that cannot be read says so).
fn connector_is_read(log: &mut Log, when: &str, dir: &str) {
    for name in ["name", "modes"] {
        let said = std::fs::read_to_string(format!("/sys/class/graphics/fb0/{name}"))
            .map(|s| s.lines().take(2).collect::<Vec<_>>().join(" | "))
            .unwrap_or_else(|e| format!("unreadable ({e})"));
        log.say(&format!("fb0 {when}: {name} = {said}"));
    }
    for name in ["status", "enabled", "modes"] {
        let said = std::fs::read_to_string(format!("{dir}/{name}"))
            .map(|s| s.lines().take(3).collect::<Vec<_>>().join(" | "))
            .unwrap_or_else(|e| format!("unreadable ({e})"));
        log.say(&format!("connector {when}: {name} = {said}"));
    }
}

fn geometry_is_read(fb: &File, device: &str) -> Result<Geometry, String> {
    let mut var = VarScreenInfo::default();
    let mut fix: FixScreenInfo = unsafe { std::mem::zeroed() };
    let fd = fb.as_raw_fd();
    // SAFETY: the structs are the kernel's fb_var_screeninfo and fb_fix_screeninfo (sizes pinned by a test); fd is an open file
    if unsafe { libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var) } != 0 {
        return Err(format!(
            "{device}: FBIOGET_VSCREENINFO: {}",
            std::io::Error::last_os_error()
        ));
    }
    if unsafe { libc::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix) } != 0 {
        return Err(format!(
            "{device}: FBIOGET_FSCREENINFO: {}",
            std::io::Error::last_os_error()
        ));
    }
    let ch = |b: FbBitfield| Channel {
        offset: b.offset,
        length: b.length,
    };
    Ok(Geometry {
        xres: var.xres,
        yres: var.yres,
        bits_per_pixel: var.bits_per_pixel,
        line_length: fix.line_length,
        red: ch(var.red),
        green: ch(var.green),
        blue: ch(var.blue),
    })
}

/// Write the whole frame from the first byte, then make the display take it: pan, a forced set_par (a full modeset commit; a command-mode
/// panel is only sent a frame by a commit), unblank. Every ioctl's result is logged; a failed read of the screen info stops the run.
fn frame_is_written(
    log: &mut Log,
    mut fb: &File,
    g: &Geometry,
    stripes: bool,
) -> Result<(), String> {
    use std::io::{Seek, SeekFrom};
    let data = frame(g, stripes)?;
    fb.seek(SeekFrom::Start(0))
        .map_err(|e| format!("seek: {e}"))?;
    fb.write_all(&data)
        .map_err(|e| format!("write the frame: {e}"))?;
    let fd = fb.as_raw_fd();
    let mut var = VarScreenInfo::default();
    // SAFETY: var is the kernel's fb_var_screeninfo (160 bytes, pinned by a test) and fd is an open file
    if unsafe { libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var) } != 0 {
        return Err(format!(
            "FBIOGET_VSCREENINFO: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut report = |name: &str, rc: i32| {
        let said = if rc == 0 {
            "0".to_string()
        } else {
            std::io::Error::last_os_error().to_string()
        };
        log.say(&format!("{name} = {said}"));
    };
    // SAFETY: as above; the ioctls read or write only var, or take an integer
    report("FBIOPAN_DISPLAY", unsafe {
        libc::ioctl(fd, FBIOPAN_DISPLAY, &mut var)
    });
    var.rest[1] = ACTIVATE_NOW_FORCE; // activate
    report("FBIOPUT_VSCREENINFO (activate now, force)", unsafe {
        libc::ioctl(fd, FBIOPUT_VSCREENINFO, &mut var)
    });
    report("FBIOBLANK (unblank)", unsafe {
        libc::ioctl(fd, FBIOBLANK, 0 as libc::c_ulong)
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The kernel's sizes on a 64-bit machine (include/uapi/linux/fb.h): a wrong struct would let the kernel write past ours.
    #[test]
    fn the_structs_are_the_kernels_size() {
        assert_eq!(std::mem::size_of::<VarScreenInfo>(), 160);
        assert_eq!(std::mem::size_of::<FixScreenInfo>(), 80);
    }
}
