# MDSS reset: the display block starts from a known state

**The patch files are gone (2026-11-23): the series is the branch `topic/walleye-mdss-reset` of linux-phone-oco/linux, proved equal to them (kernel/README.md).** The numbering below follows the commits' order.

Brief: one patch on kernel.org v7.3 (item walleye-display-dts-any-order; decision 2026-11-20-walleye-display-handover-reset-first): `resets = <&mmcc MDSS_BCR>` on the msm8998 `mdss` node.
The bootloader leaves the display block scanning out; `msm_mdss_reset()` pulses the one unnamed reset at probe, so msm starts from a known state however early or late it loads.
The common mdss binding (`mdss-common.yaml`) already allows `resets`, so no binding patch is needed. B-r5 proved the effect with the same property put in by fdtput (research walleye-display-reset-image-b-r5-tried-signs-vanish).

`tests/host/dtb-display-reset.sh` passes on a dtb built from a tree with this patch. `SHA256SUMS` pins the patch. `build/build-display-any-order-images.sh` builds the images B-r2 and B-rp2 on that dtb.

The dtb of the any-order images is built from kernel.org v7.3 (the display tree's commit e123d3bff) with this patch, the fuel-gauge series' dts patch (`kernel/fuel-gauge-rfc/0003`, the board file names `pmi8998_fg`)
and the board's `dt/` files, `DTC_FLAGS=-@ make qcom/msm8998-google-walleye-display.dtb`. It also has the board's i2c@c17a000 enabled with the drv2624 node (no driver in the display kernel); the image script checks that is the only new node.
The pin in `SHA256SUMS` is checked by hand: `sha256sum -c SHA256SUMS`.
