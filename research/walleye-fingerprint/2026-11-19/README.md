# Walleye fingerprint: what the vendor dtb says (host only, 2026-11-19)

Read from the vendor dtbo image already on the host (`~/Projects/walleye-build/control/dtbo/entry.*`, decompiled
with `dtc -I dtb -O dts`) and strings of the vendor kernel image (`control/un/Image.raw`, Lineage 22.2 wahoo
4.4.302). Nothing on the phone was touched.

| fact | value |
|---|---|
| node | `fp_fpc1020`, `compatible = "fpc,fpc1020"` (every walleye/taimen overlay entry) |
| interrupt | tlmm GPIO 86 (0x56) |
| reset | tlmm GPIO 128 (0x80) |
| supply | `vdd_io-supply = <&pm8998_s4>` (the 1.8 V rail) |
| pinctrl | `fpc1020_reset_reset`, `fpc1020_reset_active`, `fpc1020_irq_active` (gpio 128, 128 high, 86) |
| SPI | **no spi child, no spi bus named in the node** |
| driver | `drivers/misc/fpr_FingerprintCard/fpc1020_platform_tee.c`: probe requests the two GPIOs, a wakelock `fpc_ttw_wl`, nothing else |
| TEE | the kernel carries QSEECOM; the sensor's data path is the TrustZone trustlet, not this driver |

Reading: the sensor is a FingerprintCard (FPC) part in the FPC1020 family driver's binding. The press report of
CrucialTec is not contradicted: CrucialTec builds the module, FPC makes the sensor chip in it. The exact chip
(FPC1020, 1075, 1140...) is not in the dtb; it is read from the HWID register over SPI.

The unknown that matters: which BLSP SPI controller the sensor sits on, and whether TrustZone (xPU) blocks
Linux from it. Precedent that Linux can: emainline's msm8996 experiment wrote a 100-line SPI driver and read
HWID ("Chip type: FPC1140C") on a phone of the same era.
