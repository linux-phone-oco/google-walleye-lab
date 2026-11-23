---
what: the bring-up lab of the Google Pixel 2 (walleye, MSM8998) - mainline Linux on it, Android kept as the reference
kept by: walleye coordinator (the lane); records by every role working on walleye
records in: this repository's own base (data/, through ikb); what helps every phone in linux-phone-oco/docs
branches: ROLE/WHAT (keeper/, builder/, research/, coord/), one change each, deleted when landed
project map: linux-phone-oco/docs MAP.md
---

# google-walleye-lab

The Pixel 2's lab: what the phone is (`device.toml`), what is queued, tried and learned about it (its own
base), the procedures a keeper runs on it, and how its root filesystem, kernel and boot images are built.
The tools it runs on are [lab-tools](https://github.com/linux-phone-oco/lab-tools)' `lpoco`; this lab was
made from [device-template](https://github.com/linux-phone-oco/device-template), whose README explains the
conveyor. Start with `AGENTS.md`; ask the base with `tad q phone-stands` and `tad q phone-queue`.

## Map

| folder or file | what it is |
|---|---|
| `device.toml` | the phone as facts: transports, partitions, images (`[images.*]`, by sha256), kernel, health |
| `procedures/` | one whole try per file, as one command a keeper runs under a go |
| `tools/` | this lab's own programs (rootfs and boot images, the tries' plumbing) |
| `build/` | scripts that build the kernel, initramfs and boot images into `~/Projects/walleye-build/` |
| `kernel/` | what the kernel is made of; becomes `topics.toml`, the list the test kernel is assembled from |
| `dt/` | device-tree sources; moving to kernel branches |
| `rootfs/` | the Alpine root filesystem: packages, pins, runlevels, overlay |
| `provision/` | what a freshly flashed phone is given |
| `tests/` | checks by where they run: CI, the lab machine, the phone; and the phone's reports |
| `research/` | the material behind research rows, `research/ABOUT/DATE/` |
| `data/`, `queries/`, `records/`, `tables.toml`, `idealkb.toml`, `devicebase.toml` | the base: its rows, its named queries and records, its schema and pins |
| `docs/` | pages rendered from data/, for people |
| `bin/` | the base's small install and check scripts |

Never at the top: run logs, worktrees, copies of other repositories. Each folder's README says what belongs in it.

## Licence

Data, docs and research: [CC-BY-SA-4.0](LICENSE). Scripts and code: MIT OR Apache-2.0, at your option
([`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE)). Our own device trees: GPL-2.0-only OR BSD-2-Clause;
the board file derived from the wahoo files keeps their GPL-2.0-only. [`REUSE.toml`](REUSE.toml) says it per path, the
texts are in [`LICENSES/`](LICENSES), and `bin/licence-check` (`make verify`, CI) keeps every tracked file covered.
Third-party material (LineageOS images, firmware, porthole's pieces) keeps its own licence and is not relicensed.
