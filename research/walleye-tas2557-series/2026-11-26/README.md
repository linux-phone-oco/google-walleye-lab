# TAS2557 series: Boiano's v3 on v7.3, built and checked (2026-11-26, walleye-builder)

Item walleye-tas2557-driver, steps 1 and 2 (decision 2026-11-19-walleye-tas2557-from-upstream-v3). Nothing here touched the phone.

- Source: Boiano's [PATCH v3 0/2], message-id `20260717233402.414326-1-morf3089@gmail.com` (source lkml-boiano-tas2557-v3), mbox taken from
  `https://lore.kernel.org/linux-sound/MSGID/t.mbox.gz`, patches 1/2 and 2/2 applied with `git am --3way` on tag v7.3, no conflict.
- The series: branch `topic/walleye-tas2557-v3` of linux-phone-oco/linux, tip 4b5ddbd8f821, two commits (`commits.txt`), each subject prefixed
  `FROMLIST:` and ending in `(am from LORE-LINK)`; Boiano's authorship and Signed-off-by untouched. Pushed, read back with `git ls-remote`.
- `dt_binding_check.txt`: `make DT_SCHEMA_FILES=sound/ti,tas2557.yaml dt_binding_check` in alpine@sha256:020dfcba... (the lab's pinned alpine:edge, with
  py3-dt-schema), the one-liner of docs research upstream-forks/2026-11-19/dt-checks-in-a-container.md: the schema and both of its examples (one amp, the pair
  at 0x4c/0x4d) check, no warning. (`lpoco-bringup dt-check binding` refuses that image: it has no make; the image needs its tools baked in, as that note says.)
- `overlay-merge.txt`: the fdtoverlay merge and the read-back quoted below.
- `build-W1.txt` (make with W=1, defconfig + the module symbol): arm64 defconfig + `CONFIG_SND_SOC_TAS2557=m`, `W=1 sound/soc/codecs/tas2557.o`: no warning, exit 0.
- `checkpatch.txt`: `checkpatch --strict` over both commits: 0 errors, 0 checks. One warning ("added file(s), does MAINTAINERS need updating?") on the driver
  commit: patch 1 of the same series adds the MAINTAINERS entry, so it is a per-patch false positive. spdxcheck.py could not run (no python `ply` on the host,
  and we add none): the SPDX lines are Boiano's own.
- The dts (step 2): `dt/msm8998-google-walleye-tas2557.dtso`, an overlay, so no declared image's dtb changes. Checked: base = the walleye dts built on
  tag int/walleye/ufs-on/v7.3/1 (the tree whose labels it needs), the overlay merged with `fdtoverlay`; read back from the merged dtb: i2c@c1b5000 okay,
  `audio-codec@4c` compatible ti,tas2557, reg = <0x4c 0x4d>, reset-gpios = phandle 91 (gpio@c000, qcom,pm8998-gpio) 6 flags 1 (active low). Where each value is
  from: the stock kernel's `msm8998-audio-muskie.dtsi` node `ti_tas2557_4d` (android remote of the kernel clone, branch lineage-22.1).

## What is not known yet (the research's three unknowns, still open)

1. v3 on msm8998's quaternary MI2S and the PG2.1 firmware: needs the boot try.
2. Whether v3 picks `tas2557s_PG21_uCDSP.bin` itself or needs `firmware-name`: read in the driver, answered by the boot try (the file stays out of git).
3. The four supplies: the stock node has no supply property, but its driver asks for the pm8998 BoB regulator (tas2557-core.c:2973, ti,bob-fast-profile), so VBAT may be the BoB rail (a lead, not read); the overlay leaves them out and the binding requires
   them, so the node does not pass dt-validate yet. Same for which of pm8998 gpio 7 (left) or 8 (right) is the pair's INT, and the TDM slot numbers.
