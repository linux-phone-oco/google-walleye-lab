# Inline crypto engine (ICE) against the first UFS command: what Android's probe does, and what ours must

Research only: nothing built, the phone untouched. Clean room: facts, orders and register roles only; no vendor source is kept in any tree. 2026-11-19, walleye-research.

## Read
- Google `kernel/msm`, `android-msm-wahoo-4.4-android11` head `52750b8f2138c6e15cdd6c54ac8d2929ee696b95` (gitiles): `drivers/scsi/ufs/ufs-qcom.c`, `ufs-qcom-ice.c`, `ufshcd.c`, `drivers/crypto/msm/ice.c`, `arch/arm/boot/dts/qcom/msm8998.dtsi` (the `ufsice@1db0000` node).
- Mainline v7.3 (our build tree): `drivers/ufs/host/ufs-qcom.c`, `drivers/ufs/core/ufshcd-crypto.c`, `drivers/soc/qcom/ice.c`, `arch/arm64/boot/dts/qcom/msm8998.dtsi`, the ufshc binding.
- Same caveat as research 2026-11-18-ufs-android-way: the running Lineage fork was not read.

## Android's order (where ICE sits)
1. Probe, before the host exists: the ICE device is looked up by the `ufs-qcom-crypto` phandle (the ICE node, its own register block at 0x1db0000, 0x8000 long, separate from the host's 0x1da4000, 0x2500). Not probed yet: probe defers. Not enabled in the dt: a warning and the host goes on without it.
2. Host controller enable, **pre-change**: memory config, PHY power-up sequence, lane clocks on, **then ICE init**. This is before link startup and before any UFS command. A failed ICE init fails the enable.
3. ICE init: its four clocks on (ufs core, bus, iface, ice core at 300 MHz) and a bus vote; a secure-config call to the secure world (restores the ICE configuration after power collapse); read the ICE version register; read the fuse register (a blown disable fuse fails init); low-power-mode, optimisation, processing options, test bus; wait for the ICE built-in self test (versions 2.1 and later); for ICE major 3 or later "enabling is done at the storage controller", so the ICE side stops there; an interrupt enable. Init leaves ICE in **global bypass**.
4. Per command: the host asks the ICE driver whether this request has a key. No key means bypass, no key slot is programmed. For host major version 3 (this host's UFS_HW_VERSION is not read yet; 3.0.0 or 3.1.0 either way major 3) the per-command configuration returns at once: "nothing to do". Key slots are used only by file-based and full-disk encryption requests, never by the driver's own probe commands (NOP, query, descriptors, INQUIRY, READ CAPACITY, test unit ready, a read).
5. Resume, suspend, reset and error paths call the ICE hooks (clocks and memory retention): after the card path is running, not before it.

## What ICE can touch
Only: the ICE register block, its clocks and bus vote, the secure-world configuration call, and the crypto-capable part of the UFS transfer request (a crypto-enable bit in the request descriptor of a request that has a key). It does not write the host's link-startup, UniPro, PHY or reset registers, never issues a UFS command, never sees the card's identity. Android's order puts it after the PHY and before the link, but nothing in the order depends on it: it shares clocks (axi, ahb) with the host, which are on anyway. So **the earlier judgement holds: ICE cannot touch the link or the card.**

## Mainline against that
- Mainline's msm8998 ufshc node has no ICE reg, no `qcom,ice` property, no ICE clock; the binding takes an optional "ice" register block or an optional `qcom,ice` phandle, and the ICE binding lists no msm8998 compatible (sc7180, sm8450 and later only); whether the generic driver's register layout fits this older ICE is not known yet. 
- Without either, `devm_of_qcom_ice_get` returns -EOPNOTSUPP and `ufs_qcom_ice_init` only warns "Disabling inline encryption support" and returns 0; the host crypto capability bit is then not used (UFSHCD_CAP_CRYPTO stays clear), requests carry no crypto bits, and `ufs_qcom_ice_enable` does nothing. **Zero accesses to the ICE block, no clock, no secure call.** The probe order is otherwise untouched.
- The mainline msm8998 ufshc already lacks the ice clock (`core_clk_ice`) too: the freq table has no ICE entry. Nothing refers to the ICE register block.

## What would need adding for ICE (not for a probe)
Writing an ICE node (reg 0x1db0000, clocks axi/ahb/ice core with 300 MHz, GDSC), the msm8998 compatible in the ICE binding, and testing the key-slot path (requires a TZ that accepts the config call and a block layer that uses inline crypto: only for encryption). An agent-day or two of port plus a try; no consumer in our images (no fscrypt, no dm-crypt needing inline). Hazard if added wrongly: the secure call or a hung read on an unclocked ICE block is a bus hang (not a card hazard).

## Verdict, in plain words
For a read-only probe: **skip ICE.** It cannot reach the link or the card; its absence in mainline degrades to a warning; adding it adds only a new unclocked-register risk. Add it only when someone wants inline encryption (fscrypt on UFS with inline hardware): backlog it as an upstreamable port after the card path works. Not known yet: this host's UFS_HW_VERSION (the keeper's read-only read, already queued), and whether Lineage's fork changed the order.
