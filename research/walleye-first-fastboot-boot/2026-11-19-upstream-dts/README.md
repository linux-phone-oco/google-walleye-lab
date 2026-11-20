# The Pixel 2 board dts as an upstream series (2026-11-19)

> **The outgoing copy lives in docs**: `upstream/linux-arm-msm/walleye-board-dts/` (item `walleye-board-dts`; decision `2026-11-20-outgoing-items-live-in-upstream`). The next version is made there. This folder stays as the v0 record, unchanged.

This folder holds a two-patch series on Linux v7.3 for the upstreams lane: `0001` adds `google,walleye`
to the msm8998 compatibles of `arm/qcom.yaml`, `0002` adds `msm8998-google-walleye.dts` and its
Makefile line. Nothing is mailed. The patches carry no Signed-off-by and no author of the author's
own: both are his, added when he sends. `SHA256SUMS` lists the files. Source: docs #573's candidate
(`research/upstream-forks/2026-11-19/walleye-dts-candidate.diff` in docs), polished.

## What changed from the candidate

- Comments say what the hardware is; no lab id, decision or path is left.
- Properties and children are in the order `dt-check-style --mode=strict` asks for.
- `chassis-type`, `aliases` (serial0 = the debug UART) and decimal `qcom,msm-id = <292 0x20001>`
  (as the OnePlus 5 has it) are added; the touch rails, the UFS rails and the UFS nodes are not there.
- The APSS `qcom,kpss-wdt` node is **dropped**, not sent as an msm8998.dtsi patch: its address and
  interrupt are the downstream values, and no run showed a bite resetting the phone (T3 ended as the
  alive class). It can come back as its own patch with a bite proof.
- `&adreno_smmu` stays **disabled** as a board override, with a comment saying why: its power domain
  needs the GPU rail, which nothing here describes. The reviewers may ask for it in the SoC dtsi.
- UFS stays **off**, by decision 2026-11-19-walleye-ufs-off-by-a-gate: the board lands without storage.
- Licence: GPL-2.0-only, kept, because the file derives from GPL-only wahoo board files; the
  readiness note's `GPL-2.0-only OR BSD-3-Clause` is not ours to give.

## Checks (container image lpoco-dt-check, dtschema in it)

- `dt-check binding --schema-files arm/qcom.yaml`: clean.
- `CHECK_DTBS=y W=1 qcom/msm8998-google-walleye.dtb`: no warning of ours; the two DSI
  `avoid_unnecessary_addr_size` warnings come from msm8998.dtsi.
- `dt-check-style --mode=relaxed`: clean. `--mode=strict`: 76 findings before, 2 now:
  `vreg_l2a_1p2` and `vreg_lvs2a_1p8` are never referenced in the file, but msm8998.dtsi references them
  and needs them defined (the OnePlus 5 does the same).
- `scripts/checkpatch.pl --strict`: 0 errors; patch 0002 has one warning, "does MAINTAINERS need
  updating", a false positive (the qcom entry covers the folder). Patch 0001: 0.

## Not known yet

- The dts as cut here has not booted: the lab image it comes from has. A boot of this exact tree is
  an attended try.
- Whether the maintainers want the GPU smmu off in the board file.

## Programmer review (2026-11-19), and what is still open before mailing

Done from it: the unused `vdd_l6`, `vdd_l16_l28`, `vdd_l26` supplies removed; the reserved-memory and
ramoops comments reworded; "must not be switched on" for the GPU domain.

Still open, for the author or the upstreams lane:
- The patches' author is the session identity, not the author's: re-author both and add his Signed-off-by.
- The commit message of 0002 should say why the carve-outs differ from msm8998.dtsi (the lab's
  porthole wahoo notes say the stock sizes differ: modem larger, SLPI smaller) and name where the
  wahoo files are published (kcxt/linux-wahoo); where `0xa1100000` was measured (the lab file says
  the stock system's own region; the porthole wahoo dtsi has 0xb0000000). If no remote processor needs
  the move yet, drop the four /delete-node/ overrides from this series.
- Boot this exact dtb once on the phone (an attended try) and put the result in the cover letter:
  the booted lab file differs (stubs include, pcie0 deleted, wdt, touch node).
- Optional: QCOM_ID_MSM8998 for the msm-id; a note on why no stdout-path; drop `regulator-allow-set-load`
  of s4 and the PMIC includes if nothing needs them.
