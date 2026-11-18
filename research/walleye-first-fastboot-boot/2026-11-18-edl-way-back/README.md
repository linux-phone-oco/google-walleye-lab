# Is there a known way back from an erased bootloader on the Pixel 2 (walleye)? (2026-11-18)

Public sources only. No programmer, leaked file or recovery tool was downloaded or run. The phone was only read
(`adb shell` as root: partition names and sizes, nothing copied).

## Verdict
**No known way back.** EDL (9008) is reached by the Pixel 2 when its boot chain is broken - several owners report it -
but the one thing that lets a host write to the phone in EDL, a firehose programmer signed for Google's key, has never
been public for the Pixel 2 (the owner of the `edl` tool, 2021: none released or leaked for Pixels except LG-built ones).
Five years of threads end with no recovery method. How sure: fairly (many independent negative reports; an absence is
not a proof, and a service-centre route may exist that we cannot see). So an erased bootloader on this phone is treated
as permanent loss of the phone.

## 1. Is EDL reachable?
- Yes when the boot chain is broken: walleye after a Nov 2018 factory-image flash-all, black, no logo, 'Qualcomm HS-USB QDLoader 9008' (Hovatek); a Pixel 2 that went black on an ordinary restart, Sahara HWID 0x0005e0e100420002 (MSM_ID 0x0005e0e1, OEM_ID 0x0042, MODEL_ID 2), PK hash 748921750a6ec93122dd7803860fe4f660080a937d6ae9b1d6792edd0886c199 (bkerler/edl #204); XDA 2019 'no bootloader, no recovery'. The PBL falls to Sahara when it finds no valid SBL (xbl).
- Forced entry from a working phone: `adb reboot edl` worked on the Nexus 6 (Aleph); for the Pixel 2 NOT KNOWN YET - no source tried it. It is not a recovery tool anyway (it gets you into the same locked door).
- Test points / button or cable method for walleye: NOT FOUND. Passware's public gallery lists SDM845, SDM710, SM8150 and others, no Google or MSM8998 entry. We have not opened the phone and will not for this.

## 2. Recovery of an erased Pixel 2 bootloader (public record)
- Google's own tool (pixelrepair.withgoogle.com) covers Pixel 3 and newer, Android 10+: not the Pixel 2; and it works through a running fastboot/recovery, not 9008.
- Factory image route (flash-all, fastboot flash bootloader): works only while the bootloader still runs fastboot.
- Reports: Hovatek (2018-2023), XDA 3920807 (2019-2023), Android SE 225868 (2020-2025), bkerler #204 (2021): every one asks for the programmer and none gets it. One XDA user says the device was recovered, no method given - a claim without evidence.
- Hovatek's two suggestions: a firehose programmer (not available), or boot from an SD-card 'debrick image' made on a working phone of the same model (a trick of older SoCs; whether the MSM8998 PBL does SD boot is NOT KNOWN YET; the Pixel 2 has no SD slot).

## 3. What a recovery would need
- A firehose programmer (`prog_ufs_firehose_*.elf` for MSM8998 with UFS): a small program, built by the phone maker from Qualcomm's xbl, that the PBL loads over USB (Sahara) and that then reads and writes the flash (Firehose protocol).
- It must be signed so that its root certificate hashes to the PK hash fused into this SoC; with Secure Boot on, a programmer for another phone, even another MSM8998, is refused (Renate). A Pixel 2 shows PK hash 7489217...; whether our phone's matches is NOT KNOWN YET (read only from EDL, which we do not enter).
- Public: the two catalogues (bkerler/Loaders, hoplik/Firehose-Finder) and Renate's survey of ~200 loaders (temblast.com/ref/loaders.htm): no Google/Pixel entry (searched the survey text for google, pixel, walleye, taimen: none). A 'sdm835' loader exists in the collection; it carries another maker's signature and will not match.
- Not to be done: buying or downloading leaked loaders from service sites (clean-room rules).

## 4. If a way back existed, what to prepare (and why it is only half-useful)
A read-back of the phone's own boot chain gives the files that a programmer-based flash would write back. Measured read-only on this phone (UFS, 512-byte sectors, slot _b active):
- xbl_a/xbl_b: 8104 sectors (4,149,248 bytes) each, in the boot LUNs sdb1 / sdc1.
- sda (LUN 0): abl_a 2048 (sda8), abl_b 2048 (sda29), tz_a 4096 (sda4), hyp_a 1024 (sda5), rpm_a 1024 (sda3), pmic_a 1024 (sda6), keymaster_a 1024, cmnlib_a 1024, cmnlib64_a 1024, devcfg_a 256, storsec_a 256, lockbooter_a 1024, apdp_a 512, msadp_a 512; the _b slots are the same sizes at sda24-sda43.
- sdd (LUN 3): ddr 2048 (sdd6), cdt 8 (sdd5), sec 256 (sdd7); sdf (LUN 5): devinfo 8 (sdf4). Some of these (sec, devinfo, ddr, cdt) hold device-specific data: keep as mode-600 files in the lab machine, never in a repository.
- Without a matching programmer the files restore nothing; they only matter if a programmer ever turns up (or Google's service route). They cost nothing and risk nothing (read as root, `dd` to the host), so they are worth having before any UFS probe.
- Factory image: Google's public walleye factory image holds bootloader.img and radio.img (the same partitions' content for the build), restorable only through fastboot.

## What is not known yet
- `adb reboot edl` on walleye; any test point; whether this phone's PK hash equals 7489217...; whether any service centre holds a programmer; whether MSM8998's PBL has an SD boot path.

## Sources
bkerler/edl #204; XDA 3920807; Hovatek 26042; XDA 4525079 (Renate); Android SE 225868; pixelrepair.withgoogle.com; Passware Qualcomm test-point gallery; Aleph Research, Exploiting Qualcomm EDL Programmers (2).
