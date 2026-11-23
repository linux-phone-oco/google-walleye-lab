# The SLPI carve-out, read from the phone's own images (2026-11-23, host only)

No phone touched. Read from the lab machine's read-backs under ~/lab-material/walleye (never git): `slot-readback-2026-11-20-1/vendor_a.img`
(ext4, `/firmware/slpi_v2.*`), `slot-readback-2026-11-20-1/dtbo_a.img`, `~/Projects/walleye-build/material/vf/2026-11-18-vfx-vfa-try1/fdt.dtb`
(the fdt the phone's own kernel ran with), and Linux v7.3's `msm8998.dtsi`. The firmware's `.mdt`/`.b00-.b02` were copied to
`~/lab-material/walleye/slpi-firmware-20261122/` (mode 700/600); the segments `.b03-.b14` stay in `vendor_a.img`.
`slpi_v2.mdt` sha256 `45d8768a05932947c3fd6a922f4d3e692acd4f0dcfc49ea49a1020e9b1cc6b37`.

## The firmware's header (`readelf -lW slpi_v2.mdt`)

15 program headers; entry 0x93800000. Every LOAD segment has p_flags bit 27 (0x08000000, `QCOM_MDT_RELOCATABLE`) set, so
mainline's `qcom_mdt_load` loads the image at the start of the carve-out, whatever the link address. Lowest PhysAddr 0x93800000
(code), highest end 0x93fc5000 + 0x73b000 = **0x94700000**, so the image needs **0xF00000 = 15 MiB** of one `no-map` block. The hash
segment (type 0x02200000) sits at 0x94700000 and is not counted. A 7 MiB block (0x700000) is too small by 8 MiB.

## The phone's own layout

| | vendor source dtsi (named) | running fdt (what the phone used) | vendor dtbo, walleye entries (first two) | dtbo, the other seven entries | ours today | mainline v7.3 msm8998.dtsi |
|---|---|---|---|---|---|---|
| slpi | 0x94300000 | **0x94c00000, 0xf00000** | 0x94c00000, 0xf00000 | 0x94b00000, 0xf00000 | 0x94b00000, **0x700000** | 0x94300000, 0xf00000 |
| mba | 0x94100000 | 0x94500000, 0x200000 | same | 0x94900000 | 0x94900000 | 0x94100000 |
| video | 0x93c00000 | 0x94700000, 0x500000 | same | 0x94400000 | 0x94400000 | 0x93c00000 |
| ipa/gpu | 0x95200000 | 0x95b00000, 0x100000 | same | 0x95a00000 | four nodes 0x95200000-0x957fffff | 0x95200000 ... |

The vendor dtbo (`pil_slpi_mem` fragment: `reg = <0x00 0x94c00000 0x00 0xf00000>`) is what moves the block; the bootloader merges it
by label/name. Our dts gives `pil_slpi_mem` as an empty stub label, so the overlay writes nothing of ours and our own `slpi_mem` stays as we
wrote it (z21 dmesg: `reserved mem 0x94b00000..0x951fffff (7168 KiB) memory@94b00000`). The size is 15 MiB in every layout; only the base differs.
The earlier row's "0x94c00000/15 MiB HTC" is the walleye layout; "0x94b00000/7 MiB stock" is what our dts holds, not Google's.

## What the vendor node says (running fdt `qcom,ssc@5c00000`, `qcom,pil-tz-generic`)

reg 0x5c00000 (mainline: remoteproc@**5800000**, 0x4040: differs, NOT KNOWN YET which is right; the pas driver does not touch the registers
it maps, TZ owns the reset, so it may be harmless), interrupt SPI 390 (mainline same), pas-id 12 (mainline `qcom,msm8998-slpi-pas` knows it), vdd_px = pm8998 lvs2 (mainline
`px-supply = <&vreg_lvs2a_1p8>`), vdd_cx = pm8998_l27 level (mainline `power-domains = <&rpmpd RPMPD_SSCCX>`), smp2p remote pid 3 "master-kernel"/"slave-kernel"
(mainline `smp2p-slpi` the same), glink edge label "dsps", `qcom,smem-id 424 (0x1a8)`. Firmware name in the vendor tree: `slpi` / `slpi_v2` for the sensors.

## The answer

1. `slpi_mem`: reg `<0x0 0x94c00000 0x0 0xf00000>` (the running layout), no-map, label `slpi_mem` kept.
2. That range (0x94c00000-0x95b00000) swallows our four inherited nodes ipa_fw_mem@95200000, ipa_gsi_mem@95210000, gpu_mem@95600000, wlan_msa_mem@95700000:
   delete them and put the running layout's `0x95b00000 0x100000` (ipa/gpu) in, with mba 0x94500000/2 MiB and video 0x94700000/5 MiB (same shift, so nothing overlaps). Easel
   (0x95c00000, 64 MiB) stays out until the camera work.
3. `&remoteproc_slpi { firmware-name = "qcom/msm8998/google/walleye/slpi_v2.mdt"; status = "okay"; };` with the rest from mainline unchanged.
4. Rootfs: `/lib/firmware/qcom/msm8998/google/walleye/slpi_v2.mdt` and `.b00` ... `.b14` (the loader derives `.bNN` from the three-letter suffix; `.b14` is 0 bytes and stays), copied from the phone's `vendor_a:/firmware/` on the lab machine at image build, mode 644 in the image, never in git.
5. NOT KNOWN YET: that TZ authenticates the image at the base 0x94c00000 on our boot (it did for the vendor kernel); the reg 0x5800000 vs 0x5c00000 question; the ssctl/sensor services on qrtr; the FastRPC node (not needed for the first milestone).
