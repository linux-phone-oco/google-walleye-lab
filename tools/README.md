# tools - this lab's own programs (Rust)

One crate per tool: `walleye-rootfs` (the Alpine rootfs image), `walleye-ramroot` (the Alpine-in-RAM boot
image), `walleye-stripes` (the display stripe drawer), `walleye-reboot` (reboot(2) RESTART2 with `bootloader`, which busybox drops; `build/build-reboot-bin.sh OUT` makes the static aarch64 file, declared by sha256 in `[images.walleye-reboot]`, that the stage 1 way back pushes to the phone's /tmp), `walleye-tries` (the plumbing behind procedures/), `walleye-sources` (the gate that every tests/host script names its source in tests/host/SOURCES.toml: a research row, a source row, a file, or why not yet).
