// SPDX-License-Identifier: MIT OR Apache-2.0
//! The frame of the stripes: what goes into the framebuffer, as a pure function of its geometry.

/// What the framebuffer says about itself (FBIOGET_VSCREENINFO and FBIOGET_FSCREENINFO).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub xres: u32,
    pub yres: u32,
    pub bits_per_pixel: u32,
    pub line_length: u32,
    pub red: Channel,
    pub green: Channel,
    pub blue: Channel,
}

/// One colour channel inside a pixel: where its lowest bit sits and how many bits it has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channel {
    pub offset: u32,
    pub length: u32,
}

/// The four stripes, left to right.
pub const STRIPES: [(u8, u8, u8); 4] = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)];

/// One pixel of the colour in the framebuffer's own layout.
fn pixel(g: &Geometry, (r, gr, b): (u8, u8, u8)) -> u32 {
    let put = |v: u8, c: Channel| {
        if c.length == 0 {
            return 0;
        }
        let top = if c.length >= 8 {
            u32::from(v) << (c.length - 8)
        } else {
            u32::from(v) >> (8 - c.length)
        };
        top << c.offset
    };
    put(r, g.red) | put(gr, g.green) | put(b, g.blue)
}

/// The whole frame, line_length bytes per line, yres lines: the stripes, each a quarter of the width
/// (the last takes the remainder), or all black.
pub fn frame(g: &Geometry, stripes: bool) -> Result<Vec<u8>, String> {
    if g.bits_per_pixel != 32 && g.bits_per_pixel != 16 {
        return Err(format!(
            "{} bits per pixel: only 32 and 16 are drawn",
            g.bits_per_pixel
        ));
    }
    let bytes = (g.bits_per_pixel / 8) as usize;
    if (g.line_length as usize) < g.xres as usize * bytes {
        return Err(format!(
            "line_length {} is shorter than {} pixels of {} bytes",
            g.line_length, g.xres, bytes
        ));
    }
    let mut line = vec![0u8; g.line_length as usize];
    if stripes {
        let w = (g.xres / 4) as usize;
        for x in 0..g.xres as usize {
            let s = (x / w.max(1)).min(3);
            let p = pixel(g, STRIPES[s]).to_le_bytes();
            line[x * bytes..(x + 1) * bytes].copy_from_slice(&p[..bytes]);
        }
    }
    Ok(line.repeat(g.yres as usize))
}

/// One act of the drawer on the console and the framebuffer, in the order it is done.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// KDSETMODE KD_GRAPHICS on /dev/tty0: fbcon stops repainting the text over the stripes
    ConsoleGraphics,
    Write,
    Pan,
    SetPar,
    Unblank,
    Hold,
    Clear,
    /// KDSETMODE KD_TEXT on /dev/tty0: the kernel log text comes back
    ConsoleText,
}

/// The acts of one run (decision 2026-11-19-walleye-stripes-text-mode-off-while-held): the console is in
/// KD_GRAPHICS before the draw and in KD_TEXT after the release, also when the draw failed (then nothing is held).
pub fn plan(draw_failed: bool) -> Vec<Act> {
    use Act::*;
    let mut acts = vec![ConsoleGraphics, Write, Pan, SetPar, Unblank];
    if !draw_failed {
        acts.push(Hold);
    }
    acts.extend([Clear, ConsoleText]);
    acts
}

/// KDSETMODE (linux/kd.h) and its two modes.
pub const KDSETMODE: u64 = 0x4B3A;
pub const KD_TEXT: u64 = 0;
pub const KD_GRAPHICS: u64 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_console_is_in_graphics_while_the_stripes_are_held_and_in_text_after() {
        use Act::*;
        assert_eq!(
            plan(false),
            [
                ConsoleGraphics,
                Write,
                Pan,
                SetPar,
                Unblank,
                Hold,
                Clear,
                ConsoleText
            ]
        );
    }

    #[test]
    fn the_console_goes_back_to_text_also_when_the_draw_failed() {
        let p = plan(true);
        assert_eq!(p.last(), Some(&Act::ConsoleText));
        assert!(!p.contains(&Act::Hold));
    }

    #[test]
    fn the_kd_constants_are_the_kernels() {
        assert_eq!((KDSETMODE, KD_TEXT, KD_GRAPHICS), (0x4B3A, 0, 1));
    }

    fn xrgb(xres: u32, yres: u32) -> Geometry {
        Geometry {
            xres,
            yres,
            bits_per_pixel: 32,
            line_length: xres * 4,
            red: Channel {
                offset: 16,
                length: 8,
            },
            green: Channel {
                offset: 8,
                length: 8,
            },
            blue: Channel {
                offset: 0,
                length: 8,
            },
        }
    }

    fn at(f: &[u8], g: &Geometry, x: u32, y: u32) -> u32 {
        let i = (y * g.line_length + x * 4) as usize;
        u32::from_le_bytes([f[i], f[i + 1], f[i + 2], f[i + 3]])
    }

    #[test]
    fn four_stripes_red_green_blue_white_across_the_full_panel() {
        let g = xrgb(1080, 1920);
        let f = frame(&g, true).unwrap();
        assert_eq!(f.len(), 1080 * 1920 * 4);
        for y in [0, 959, 1919] {
            assert_eq!(at(&f, &g, 0, y), 0xff0000);
            assert_eq!(at(&f, &g, 269, y), 0xff0000);
            assert_eq!(at(&f, &g, 270, y), 0x00ff00);
            assert_eq!(at(&f, &g, 539, y), 0x00ff00);
            assert_eq!(at(&f, &g, 540, y), 0x0000ff);
            assert_eq!(at(&f, &g, 809, y), 0x0000ff);
            assert_eq!(at(&f, &g, 810, y), 0xffffff);
            assert_eq!(at(&f, &g, 1079, y), 0xffffff);
        }
    }

    #[test]
    fn the_channel_offsets_decide_the_order_of_the_bytes() {
        let mut g = xrgb(8, 2);
        g.red = Channel {
            offset: 0,
            length: 8,
        };
        g.blue = Channel {
            offset: 16,
            length: 8,
        };
        let f = frame(&g, true).unwrap();
        assert_eq!(at(&f, &g, 0, 0), 0x0000ff);
        assert_eq!(at(&f, &g, 2, 0), 0x00ff00);
        assert_eq!(at(&f, &g, 4, 0), 0xff0000);
    }

    #[test]
    fn a_line_longer_than_the_pixels_keeps_its_padding_black() {
        let mut g = xrgb(8, 2);
        g.line_length = 40;
        let f = frame(&g, true).unwrap();
        assert_eq!(f.len(), 80);
        assert_eq!(&f[32..40], &[0u8; 8]);
        assert_eq!(&f[72..80], &[0u8; 8]);
    }

    #[test]
    fn sixteen_bits_pack_five_six_five() {
        let g = Geometry {
            xres: 8,
            yres: 1,
            bits_per_pixel: 16,
            line_length: 16,
            red: Channel {
                offset: 11,
                length: 5,
            },
            green: Channel {
                offset: 5,
                length: 6,
            },
            blue: Channel {
                offset: 0,
                length: 5,
            },
        };
        let f = frame(&g, true).unwrap();
        assert_eq!(u16::from_le_bytes([f[0], f[1]]), 0xf800);
        assert_eq!(u16::from_le_bytes([f[4], f[5]]), 0x07e0);
        assert_eq!(u16::from_le_bytes([f[8], f[9]]), 0x001f);
        assert_eq!(u16::from_le_bytes([f[12], f[13]]), 0xffff);
    }

    #[test]
    fn the_release_frame_is_black() {
        let f = frame(&xrgb(16, 4), false).unwrap();
        assert!(f.iter().all(|b| *b == 0));
    }

    #[test]
    fn an_unknown_depth_or_a_short_line_is_refused() {
        let mut g = xrgb(8, 2);
        g.bits_per_pixel = 24;
        assert!(frame(&g, true).is_err());
        let mut g = xrgb(8, 2);
        g.line_length = 16;
        assert!(frame(&g, true).is_err());
    }
}
