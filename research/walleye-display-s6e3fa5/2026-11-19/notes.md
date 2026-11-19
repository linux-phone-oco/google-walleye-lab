This folder holds the readings behind research walleye-display-b2-dsi-clock-warning: what the B2 and L6 logs show, side by side, and the dts facts. Nobody reads it unless the finding is questioned.

# Readings (logs under ~/lab-material/walleye, not in git)

| run | msm.ko loaded at | SMMU faults | pclk0 rcg warnings | DSI cmd -110 |
|---|---|---|---|---|
| display-boot-195643 (B) | 2.0 s, by the init | 10 | 4 | 1 |
| display-boot-202243 (B1) | 2.1 s | 10 | 4 | 2 |
| display-boot-214918 (B2) | 2.14 s | 10 | 4 | 2 |
| display-20261119-222155 (L6, by hand) | 66.3 s | 0 | 0 | 0 |

The kernel, the dtb, the module set and the module order are the same in all four (13 modules, the command line has clk_ignore_unused and iommu.passthrough=1).
The only difference the logs show is WHEN msm is loaded: about 2 s against about 66 s after boot.

B2 order: 2.140 msm.ko insmod; 2.202 panel ko; 2.2275 dpu bound dsi; 2.2371 dpu hardware revision; 2.2379 first SMMU context fault, cb=0, SID 0, iova 0x9d6f1500 (then 9 more, 0x9d720700 .. 0x9d771c00, stride about 0x8000-0x9000: a linear scan-out);
2.265 drm Initialized; 2.304 'pclk0_clk_src: rcg didn't update its configuration' (clk-rcg2.c:136), trace clk_pixel_set_rate <- dsi_link_clk_set_rate_6g <- msm_dsi_host_power_on <- dsi_mgr_bridge_pre_enable <- (fbcon commit) <- component bind <- dsi_dev_attach <- mipi_dsi_attach <- s6e3fa5_probe: the panel's probe runs the whole drm bind and the first modeset inside mipi_dsi_attach;
the warning repeats 4 times (2.304, 2.318, 2.331, 2.343: the clock core retries); 2.377 dsi_err_worker status=c; 2.596 DCS 0x11 -110; 2.800 DCS 0x51 -110; 2.889 '[dpu error] enc35 pp0 disable autorefresh failed'.
L6 (66.3 s): the same probe order, no fault, no warning, fb0 registered 66.6 s, panel shows the stripes.

# Facts from the v7.3 tree (~/Projects/walleye-build/linux-v7.3)

- msm8998.dtsi: mdss has iommus = <&mmss_smmu 0>; the dpu and dsi nodes have none; dsi0 has assigned-clock-parents = byte/pixel PLL of mdss_dsi0_phy; the phy gets iface + xo. These are complete and the same as the OnePlus 5/5T and Xperia XZ1 boards use (they set no display properties of their own beyond simplefb).
- msm8998-oneplus-common.dtsi and msm8998-fxtec-pro1.dts: reserved-memory cont_splash_mem memory@9d400000 no-map (0x2400000 / 0x2000000 long). OnePlus adds a simple-framebuffer on it with the mmcc clocks and the MDSS_GDSC so the unused-clock cleanup keeps the bootloader's scan-out alive.
- msm8998-google-walleye.dts has NO region at 0x9d400000: the boot log's reserved-memory list has nothing between 0x95700000 and ramoops 0xa1100000. The faulting iovas 0x9d6f1500..0x9d771c00 are inside the 0x9d400000 splash region of the same SoC.
- dpu_hw_pingpong.c:243 prints 'disable autorefresh failed' - the bootloader leaves the command-mode pingpong in autorefresh (the panel is cmd mode: mdss_dsi_samsung_s6e3fa5_1080p_cmd on the bootloader command line).
- the walleye dts has no clocks/power-domains pinned for the splash and no panel supply of the bootloader's.
