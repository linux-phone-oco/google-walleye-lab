# Device template

**A new phone's lab is a new `device.toml`.** Use this template to make `VENDOR-CODENAME-lab` for a phone
running postmarketOS; everything else - reaching the phone, flashing, provisioning, a test kernel tried
once and falling back by itself, the phone checking its own health after every boot, gathering its
reports into the picture, and a door for agents - comes from
[lpoco lab](https://github.com/linux-phone-oco/lab-tools), which reads this manifest. Surya
([linux-phone-oco/surya-lab](https://github.com/linux-phone-oco/surya-lab)) is the first phone made
this way, not the mould.

## Start

1. **Use this template**, name it `VENDOR-CODENAME-lab`; give it the topics `oco-project` and
   `oco-kind-device-lab` so the observatory finds it.
2. Fill `device.toml`: every value marked `CHANGE`, and `[health]` - what the phone has, what it lacks,
   what has never worked yet. Unknown fields are refused, so a typo cannot pass silently.
3. Put the phone user's password in `~/.config/devlab/CODENAME/password` (mode 600) on the lab machine.

## The base

A device made from this template has its own open base (tad, `data/`): its queue, tries, measurements, decisions,
research and parts. `make setup` installs tad and `ikb`; `ikb --help` lists what can be recorded, `ikb help RECORD`
how. The standing tables follow [grenudi/ideal-knowledge-base](https://github.com/grenudi/ideal-knowledge-base)
(`idealkb.toml`); the device's own are in `tables.toml`. Fill the device (`ikb device ...`) and declare what is
claimed of each part (`ikb part`: status, evidence, date) - [linux-phone-oco/docs](https://github.com/linux-phone-oco/docs)
pulls `parts` (contract in `devicebase.toml`) and derives the device's support status from them; it types nothing of
its own. The phone's queue is `tad q phone-queue`, its keeper's next item `tad q phone-next`; `lpoco lab gather` files runs
with the `test-run` record.

## The conveyor

    lpoco lab flash now IMAGE BOOTLOADER      an image onto the phone, the manifest's fixes baked in
    lpoco lab provision now                   packages and Wi-Fi - and then the phone's health
    lpoco lab suite put now --runner FILE     the health suite onto the phone: it checks itself after every boot
    lpoco lab kernel install NAME ... / try NAME   a test kernel beside stock, booted once - then its health
    lpoco lab phone reboot                    and wait - then the new boot's health
    lpoco lab gather now                      the phone's reports into tests/results and the picture's tables
    lpoco lab mcp                             the phone for any agent (MCP, stdio)

Every change lpoco lab makes ends with the phone's own health report: nobody has to remember to check.
A part the phone never had working is a `gap` - measured on every boot, but not ill health.

## What lives here

- `device.toml` - the phone, as facts.
- `tests/health/` - this phone's own checks, if any, beside the generic ones (see its README).
- `tests/results/` - every health report the phone wrote, as it wrote it (`lpoco lab gather now`).
- `provision/packages.txt` - what a freshly flashed phone gets.
- its backlog, decisions, milestones and log are kept in
  [linux-phone-oco/docs](https://github.com/linux-phone-oco/docs), the project's one database (project =
  `linux-phone-oco/device-template`), and shown on [its face](https://linux-phone-oco.github.io/docs/).

## Licence

Data, docs and research: [CC-BY-SA-4.0](LICENSE). Scripts and code: MIT OR Apache-2.0, at your option
([`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE)), except the C probe `build/ffs-acm`, GPL-2.0-only by its own header. Our own device trees: GPL-2.0-only OR BSD-2-Clause;
the board file derived from the wahoo files keeps their GPL-2.0-only. [`REUSE.toml`](REUSE.toml) says it per path, the
texts are in [`LICENSES/`](LICENSES), and `bin/licence-check` (`make verify`, CI) keeps every tracked file covered.
Third-party material (LineageOS images, firmware, porthole's pieces) keeps its own licence and is not relicensed.
