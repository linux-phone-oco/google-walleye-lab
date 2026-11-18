# Notes for agents


**Owner**: `walleye-coordinator` - the lane coordinator of the Pixel 2 (the top coordinator is its parent). It queues the lane (tad q phone-next, lane-next), starts the keeper, builder and research launchers, decides what needs no hands and records it here; the keeper (`walleye-keeper`) holds the phone, the builder and research sessions never touch it.

Principles of linux-phone-oco, cited and never restated ([`tad q principles`](https://github.com/linux-phone-oco/docs) in docs): [as-good-as-android](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#as-good-as-android), [measured-not-claimed](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#measured-not-claimed), [mainline-first](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#mainline-first), [no-manual-steps](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#no-manual-steps), [rust-with-argenv](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#rust-with-argenv).

- **Everything about the phone is in `device.toml`.** No script in this repository names the phone; a
  tool that needs a fact about it reads the manifest. A fact that is missing goes into the manifest -
  and, if it is new for every phone, into lpoco lab's manifest model (linux-phone-oco/lab-tools).
- **Reach the phone through lpoco lab** (`lpoco lab phone sh|root` with the commands on standard input, or the
  `lpoco lab mcp` tools) - never by hand-written ssh with nested quoting.
- **Keep the phone known-working.** A test kernel is a second boot entry tried once by boot counting,
  falling back to stock by itself; never the initramfs debug shell on an unattended try. Every change
  ends with the phone's health report - read it.
- **Measured, not claimed** ([principle](https://github.com/linux-phone-oco/docs/blob/main/docs/concepts.md#measured-not-claimed)): a check that cannot measure says `skip`
  and why; a part never working yet is a `gap` in `[health]`.
- **Keeper's battery rule** (item walleye-battery-watch): before every try run `sh tests/host/battery-ok.sh`; it prints the voltage
  and exits 1 below 3700 mV (exit 2, `skip`, when nothing answers within 15 s: not known yet; stop and record too) - then do not try: stop, record the value, and leave the phone charging in Android. Never park the phone
  in fastboot between tries (fastboot draws power and charges little). End every session with the phone booted into Android on a
  good slot (`fastboot reboot`, then adb) and record `sh tests/host/idle-android.sh`'s line as the end state. `sh tests/host/battery-selftest.sh`
  proves both scripts against fake fastboot and adb, never touching the phone.
- **Where is the phone? Ask `sh tests/host/phone-state.sh`** (item walleye-phone-state-witnesses): a verdict over adb, fastboot, the USB bus by sysfs and the tty's answer - `android`, `fastboot`, `mainline-answering`, `mainline-silent` ("NOT KNOWN"), `not-known`, `gone`; it never says hung. A recipe that waits for the phone's return calls it - restart-check.sh does when the journal shows no disconnect (android/fastboot = restarted, mainline-answering = no restart path, else NOT KNOWN), and the gadget-up and window readers print its verdict when no gadget is found; lsusb alone is one witness and proves nothing (research walleye-c3-hang-unproven). `sh tests/host/phone-state-selftest.sh` proves it against fakes.
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
| what you may take now, and why the rest may not | `tad q ready --for keeper\|builder\|research` (`--show ready` for the picks only) - the one picking query; a brief names it and carries no picking SQL |
| where something is said | `tad q find --text WORDS` |
| one record whole | `tad q row --id ID` |
| what was recorded lately | `tad q recent` (`--days N`) |
| anything else | `tad statements`: every query and record, with its parameters |

At a task's start ask only about its subject (`about --subject S`, or `next --about S`) and act. Every
answer gives each record by its brief, its one line. Read deeper - `row --id`, `--whole true`, a file past
its first paragraph, another subject, `tad statements` - only when a step needs more. Long answers read best
with `--format csv`.

## Recording
Record with `ikb` (`sh bin/install-ikb.sh` once installs it): `ikb --help` lists the records of this base, one line each; `ikb help RECORD` gives the one you need, its options written from `records/*.sql`. Then `ikb RECORD --option value`: long text from a file or stdin (`--found @-`), the id made from the brief, canon and render done after - never SQL, never by hand. Each record names what it is about (`--about`) and, for a decision or a research, its brief (the record in one line; the title is the brief of the rest), then land. A change to a row no record covers (a status, a wait, a field of a decision, a source taken away): `ikb set TABLE --key K --field NAME=VALUE`, `ikb remove TABLE --key K` - checked, never `tad sql`. Where `tad q about` says a summary is due for a subject you recorded into, renew it: `ikb summary --about S --says ...` - how it stands now, naming the records that carry it.
A thing there are many of with one shape is a row; prose read once in order is a file (README, this entry,
a brief), and a file opens with its own brief: its first paragraph says what it holds. Research is both: its finding a row, its own material - notes, excerpts, measurements - a dated
folder the row names (`--material auto` makes `research/ABOUT/DATE/`); redone, a new row `--supersedes` the old.
Work headed to others (a patch, finding, correction or question for a project, list, wiki, standard, person or another base) is an upstream item: `tad q upstream` shows where each stands; record with `ikb upstream-item --from-work ID`. A patch carried in a fork until its upstream has it names the fork (`--carried-in FORK-ID/BRANCH`; the fork is an upstream of its own: `ikb upstream --fork-of ORIGINAL`); `tad q carried` says which to keep and which to drop at the next rebase. Its tag column - UPSTREAM, FROMGIT, FROMLIST, OURS - is how far the patch has travelled, read off the item's state; the fork's commit subjects carry it as `FROMLIST(item-id): subject` (`bin/check-fork-tags.sh BASE REPO RANGE` finds one that disagrees). A workaround standing in for an upstream fault is marked `WORKAROUND(item-id)` in place and its place goes in the item's `--local-ref`; it ends when the item lands. Move an item on as the replies come (`ikb upstream-state --state accepted --reviewed-by "Name"`), never leave it as it was sent.
A decision never edited when it changes: a new one `--supersedes` it whole or `--amends` it in part, says who it binds (`--scope`) and the backlog item that carries it out (`--tracked-by`); `tad q decided` reads where each stands - superseded, in work, carried out, untracked - from that work.
<!-- idealkb:end -->
