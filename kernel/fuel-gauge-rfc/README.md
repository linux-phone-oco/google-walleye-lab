# The fuel gauge RFC, three patches of eight

**The patch files are gone (2026-11-23): the series is the branch `topic/walleye-fuel-gauge-rfc` of linux-phone-oco/linux, proved equal to them (kernel/README.md).** The numbering below follows the commits' order.

Brief: the patches of the RFC "Qualcomm 3rd gen fuel gauge support" that the Pixel 2's fuel gauge needs, kept as received so `build/build-fuel-gauge.sh` builds the same kernel every time (item walleye-fuel-gauge).

Source: [PATCH RFC 0/8], David Heidelberg for Joel Selvaraj's driver, 2025-11-24, message `20251124-pmi8998_fuel_gauge-v1-0-dd3791f61478@ixit.cz` on lore.kernel.org (not in mainline; carried by sdm845-next). Authors are in each patch; licence of each: GPL-2.0 as the kernel's.

| file | patch | what |
|---|---|---|
| 0001 | 1/8 | the binding, `qcom,pmi8998-fg` |
| 0002 | 2/8 | the driver, `drivers/power/supply/pmi8998_fg.c` |
| 0003 | 3/8 | the `pmi8998.dtsi` node, disabled |

Patches 4/8 to 8/8 are other boards' dts (pm660, beryllium, axolotl, lavender, sargo) and are not taken. They apply with `git am -3` on v7.3 plus the pmic-wdt topic (e123d3bff), cleanly; `SHA256SUMS` pins them.
