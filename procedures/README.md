# procedures - what a keeper runs on the phone, one file per procedure

Each `NAME-on-the-pixel-2.rs` is one whole try as one command (rust-script, on lab-tools' attended runner):
its checks, the change, and the way back. A keeper runs one under a go; a builder writes it. Plumbing shared
by several lives in `tools/walleye-tries`.

Run them with `bin/attended run|check NAME ...`: it builds the lpoco-attended the steps crate (tools/walleye-tries) is locked to, once per revision, and never uses the host's `~/.local/bin/lpoco-attended` (tests/host/no-bare-lpoco-attended.sh).
