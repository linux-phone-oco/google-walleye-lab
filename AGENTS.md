# Notes for agents

- **Everything about the phone is in `device.toml`.** No script in this repository names the phone; a
  tool that needs a fact about it reads the manifest. A fact that is missing goes into the manifest -
  and, if it is new for every phone, into devlab's manifest model (linux-phone-oco/lab-tools).
- **Reach the phone through devlab** (`devlab phone sh|root` with the commands on standard input, or the
  `devlab mcp` tools) - never by hand-written ssh with nested quoting.
- **Keep the phone known-working.** A test kernel is a second boot entry tried once by boot counting,
  falling back to stock by itself; never the initramfs debug shell on an unattended try. Every change
  ends with the phone's health report - read it.
- **Show what nobody measured as unknown**, never as broken: a check that cannot measure says `skip`
  and why; a part never working yet is a `gap` in `[health]`.
- **Before a test a person must witness** (listening, looking), say what to expect and what to check,
  and wait for their go.
- **A script that changes state needs explicit intent** (a verb such as `now`); lists are passed as
  arguments, never spliced into a shell string. Secrets stay on the lab machine, mode 600.
- **Record as you go** in linux-phone-oco/docs, the project's one database, through tad's named records
  (`tad r decision --project device-template ...`): decisions with what was rejected, the backlog, the log. The
  record of the work itself is the commits and pull requests; land with `land` (or `landit`).
