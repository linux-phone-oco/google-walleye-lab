# What cuts the bootloader's command line: the evidence counted (2026-11-23)

Read from captured text only (lab-material/walleye on the lab machine, this folder). No phone touched.

| boot | path | slot | line (chars) | ends |
|---|---|---|---|---|
| Android b, stock (android-b-cmdline-and-boot-props.txt) | normal | _b | 1401 | ...pk=g-pxl-p androidboot.slot_suffix=_b want_initramfs rootwait ro init=/init androidboot.force_normal_boot=1 |
| Android b after our failed boot (rehearsal-6-failed-boot-read.txt) | normal | _b | same tail | whole |
| fastboot boot, 12 captures (mainline-shell, c3, fg1, d0, z20, z21, display-dtb-bisect, display-f3-split) | RAM | _b | 1176 to 1393 | ...pk=g-pxl-p androidboot.slot_suffix=_b skip_initramfs rootwait ro init=/init |
| flashed boot_a, run 8 | normal | _a | 1363 | ends in the bootloader's words, at/after pk |
| flashed boot_a, run 9 (install-boot-20261123-192731) | normal | _a | 1136 | ...androidboot.pk=g-pxl-p |
| flashed boot_a, run 12, production image (install-boot-20261123-220443) | normal | _a | 1122 | ...androidboot.pk=g-pxl-p |

Counted from these:
1. Not a total-length limit: cut lines of 1122 and 1136 characters, whole lines of 1393 and 1401.
2. Not our header: headers of 149 and about 135 characters and the long dyndbg one all cut at the same word; the words the bootloader appends up to and including pk come to 986 characters in runs 9 and 12 (Android b: 990).
3. Not the AVB footer by itself: the fastboot-boot images (build-display-image.sh, build-mainline-light.sh) carry the same 'algorithm NONE' footer made by avbtool 1.4 and got the whole tail. The boot_a that run 12 found before its flash (boot_a-before-the-flash.img) is our earlier image, os version 15.0.0, patch 2026-11, not the stock one.
4. The two classes differ in exactly two things that go together, never apart in any capture: the path (flashed slot, normal boot, vs fastboot boot) and the slot (every cut line is slot _a; every whole line, stock or ours, is slot _b). No line with slot_suffix=_a has ever been read, from any image.
5. The cut falls exactly where the slot words begin: the next word the bootloader would add is androidboot.slot_suffix.
