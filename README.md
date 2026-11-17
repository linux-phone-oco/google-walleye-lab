# Device template

**A new phone's lab is a new `device.toml`.** Use this template to make `VENDOR-CODENAME-lab` for a phone
running postmarketOS; everything else - reaching the phone, flashing, provisioning, a test kernel tried
once and falling back by itself, the phone checking its own health after every boot, gathering its
reports into the picture, and a door for agents - comes from
[devlab](https://github.com/linux-phone-oco/lab-tools), which reads this manifest. Surya
([linux-phone-oco/surya-lab](https://github.com/linux-phone-oco/surya-lab)) is the first phone made
this way, not the mould.

## Start

1. **Use this template**, name it `VENDOR-CODENAME-lab`; give it the topics `oco-project` and
   `oco-kind-device-lab` so the observatory finds it.
2. Fill `device.toml`: every value marked `CHANGE`, and `[health]` - what the phone has, what it lacks,
   what has never worked yet. Unknown fields are refused, so a typo cannot pass silently.
3. Put the phone user's password in `~/.config/devlab/CODENAME/password` (mode 600) on the lab machine.

## The conveyor

    devlab flash now IMAGE BOOTLOADER      an image onto the phone, the manifest's fixes baked in
    devlab provision now                   packages and Wi-Fi - and then the phone's health
    devlab suite put now --runner FILE     the health suite onto the phone: it checks itself after every boot
    devlab kernel install NAME ... / try NAME   a test kernel beside stock, booted once - then its health
    devlab phone reboot                    and wait - then the new boot's health
    devlab gather now                      the phone's reports into tests/results and the picture's tables
    devlab mcp                             the phone for any agent (MCP, stdio)

Every change devlab makes ends with the phone's own health report: nobody has to remember to check.
A part the phone never had working is a `gap` - measured on every boot, but not ill health.

## What lives here

- `device.toml` - the phone, as facts.
- `tests/health/` - this phone's own checks, if any, beside the generic ones (see its README).
- `tests/results/` - every health report the phone wrote, as it wrote it (`devlab gather now`).
- `provision/packages.txt` - what a freshly flashed phone gets.
- its backlog, decisions, milestones and log are kept in
  [linux-phone-oco/docs](https://github.com/linux-phone-oco/docs), the project's one database (project =
  `linux-phone-oco/device-template`), and shown on [its face](https://linux-phone-oco.github.io/docs/).
