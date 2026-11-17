# Notes for agents

Principles of linux-phone-oco, cited and never restated ([`tad q principles`](https://github.com/linux-phone-oco/docs) in docs): [as-good-as-android](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#as-good-as-android), [measured-not-claimed](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#measured-not-claimed), [mainline-first](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#mainline-first), [no-manual-steps](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#no-manual-steps), [rust-with-argenv](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#rust-with-argenv).

- **Everything about the phone is in `device.toml`.** No script in this repository names the phone; a
  tool that needs a fact about it reads the manifest. A fact that is missing goes into the manifest -
  and, if it is new for every phone, into devlab's manifest model (linux-phone-oco/lab-tools).
- **Reach the phone through devlab** (`devlab phone sh|root` with the commands on standard input, or the
  `devlab mcp` tools) - never by hand-written ssh with nested quoting.
- **Keep the phone known-working.** A test kernel is a second boot entry tried once by boot counting,
  falling back to stock by itself; never the initramfs debug shell on an unattended try. Every change
  ends with the phone's health report - read it.
- **Measured, not claimed** ([principle](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#measured-not-claimed)): a check that cannot measure says `skip`
  and why; a part never working yet is a `gap` in `[health]`.
- **Before a test a person must witness** (listening, looking), say what to expect and what to check,
  and wait for their go.
- **A script that changes state needs explicit intent** (a verb such as `now`); lists are passed as
  arguments, never spliced into a shell string. Secrets stay on the lab machine, mode 600.
- **Record as you go** in this device's own base (data/, kept by tad): `ikb --help`, then `ikb help RECORD` - decisions with
  what was rejected, research, the backlog (`ikb backlog`; `ikb backlog-needs` for the phone's queue), the log, and
  the parts (`ikb part`: what is claimed of each part, its evidence and date - docs pulls them and derives the
  device's support status, so type nothing of it there). What helps every device goes to linux-phone-oco/docs. The
  record of the work itself is the commits and pull requests; land with `land` (or `landit`).

<!-- idealkb:begin - kept by grenudi/ideal-knowledge-base at the release idealkb.toml names: change it there -->
## Finding what is known
Ask a query; never read `data/` or `docs/` to answer - they are the database's files and the pages for people.

| To know | Ask |
|---|---|
| how a subject stands, then all that is known of it | `tad q about --subject S` (`--whole true` for every record in full; `tad q about` lists the subjects, and where a summary is due) |
| the work ahead | `tad q next` (`--about S` for one subject) |
| where something is said | `tad q find --text WORDS` |
| one record whole | `tad q row --id ID` |
| what was recorded lately | `tad q recent` (`--days N`) |
| anything else | `tad statements`: every query and record, with its parameters |

At a task's start ask only about its subject (`about --subject S`, or `next --about S`) and act. Every
answer gives each record by its brief, its one line. Read deeper - `row --id`, `--whole true`, a file past
its first paragraph, another subject, `tad statements` - only when a step needs more. Long answers read best
with `--format csv`.

## Recording
Record with `ikb` (`sh bin/install-ikb.sh` once installs it): `ikb --help` lists the records of this base, one line each; `ikb help RECORD` gives the one you need, its options written from `records/*.sql`. Then `ikb RECORD --option value`: long text from a file or stdin (`--found @-`), the id made from the brief, canon and render done after - never SQL, never by hand. Each record names what it is about (`--about`) and, for a decision or a research, its brief (the record in one line; the title is the brief of the rest), then land. Where `tad q about` says a summary is due for a subject you recorded into, renew it: `ikb summary --about S --says ...` - how it stands now, naming the records that carry it.
A thing there are many of with one shape is a row; prose read once in order is a file (README, this entry,
a brief), and a file opens with its own brief: its first paragraph says what it holds. Research is both: its finding a row, its own material - notes, excerpts, measurements - a dated
folder the row names (`--material auto` makes `research/ABOUT/DATE/`); redone, a new row `--supersedes` the old.
Work headed to others (a patch, finding, correction or question for a project, list, wiki, standard, person or another base) is an upstream item: `tad q upstream` shows where each stands; record with `ikb upstream-item --from-work ID`.
<!-- idealkb:end -->
