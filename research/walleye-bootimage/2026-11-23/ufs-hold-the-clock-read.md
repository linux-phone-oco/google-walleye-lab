# How to hold the UFS clock low: our config, mainline ufshcd, Android's (read 2026-11-23, walleye-research)
Read only; no phone. Mainline = v7.3 in ~/Projects/linux-phone-oco/linux; Android = reference/android-walleye/4.4.302-g113bd1cfa6f5.

## 1 Our kernel
- Governors built (keeper's capture, PR 948): available_governors = userspace, simple_ondemand. No powersave/performance: DEVFREQ_GOV_POWERSAVE is not set (drivers/devfreq/Kconfig v7.3: each governor is its own tristate). The ramped try's step 16 asked for "powersave": the procedure expected a governor nobody had read from the config.
- UFS: SCSI_UFS_QCOM=y, built in (build/walleye.config); the image carries no modules. Command line: clk_ignore_unused (build/build-bootimg.sh:20).
- DT (int/walleye/ufs-on/v7.3/1 msm8998.dtsi:1040): freq-table-hz core_clk 50000000..200000000, core_clk_unipro 37500000..150000000; no operating-points table, so hba->use_pm_opp is false.

## 2 Mainline ufshcd v7.3
- devfreq registered with simple_ondemand (ufshcd_devfreq_init, ufshcd.c:1719); two OPPs added from the first clock: min 50 MHz, max 200 MHz. ufs-qcom: polling 60 ms, up 70, down-diff 65.
- clkscale_enable=0 (ufshcd.c:1821): suspends scaling, then SCALES UP to max (freq = clki->max_freq; ufshcd_devfreq_scale(..., true)). It holds HIGH, never low. Writing 1 resumes. (Android's store does the same: ufshcd_devfreq_scale(hba, true).)
- A scale-down (ufshcd_devfreq_scale, :1511) = gear first, then clocks: ufshcd_scale_gear(new_gear,false), then ufshcd_scale_clks; ufs-qcom's clk_scale_notify does hibern8 enter/exit around it. ufs_qcom_freq_to_gear_speed returns DONT_CHANGE (0) when !use_pm_opp, so ufshcd_scale_gear takes the legacy path (:1412): scale-down sets gear_tx/rx to clk_scaling.min_gear, defaulting to UFS_HS_G1 (:1898). So a scale-down on OUR kernel is a link power-mode change HS-G3 -> HS-G1 (re-trained, not a reset), plus core 50 MHz / unipro 37.5 MHz. Not "the gear may stay 3": it drops, and a failed PMC is the risk (dev_err "failed err ... old gear/new gear").
- devfreq userspace governor (governor_userspace.c): echo userspace > governor; echo FREQ > userspace/set_freq calls update_devfreq at once; before any set_freq it keeps previous_freq. Generic devfreq also has min_freq/max_freq: writing max_freq=50000000 clamps simple_ondemand to the minimum. Both end in ufshcd_devfreq_target -> the same scale path.
- clk_ignore_unused: keeps unused clocks on; it does not stop clk_set_rate by the driver, so the scaling still lowers the rate; it matters for gating, not for this.

## 3 Android 4.4.302 (Lineage fork of Google's wahoo)
- devfreq added with "simple_ondemand" (drivers/scsi/ufs/ufshcd.c:7655); sysfs clkscale_enable (:9882-9920): 0 => suspend scaling and ufshcd_devfreq_scale(hba, true) = scale UP. Same meaning as mainline: hold = high.
- wahoo_defconfig builds only QCOM governors (BW_HWMON, MEMLAT, DEVBW, SPDM); no powersave/userspace line in the grep of DEVFREQ_GOV lines. Android's own way to hold UFS is therefore "scaling off = full speed".
- init: the keeper's captured dmesg (research/walleye-first-fastboot-boot/2026-11-18/dmesg-ufs.txt:33,47) shows /vendor/etc/init/hw/init.walleye.rc:50 writing clkscale_enable 0 at init and :499 writing 1 at sys.boot_completed=1 (both denied to the capturing shell, but the rc lines are the vendor's setting). The Lineage device tree's init scripts were not read (not in the commons): not known yet.
- DT freq-table of Android's ufshc: live fdt in research/.../2026-11-18/dt.txt (same 50/200 and 37.5/150 MHz for core/unipro, per the 2026-11-18 research).

## 4 The one way both support
Common to both: clkscale_enable (holds HIGH) and devfreq min_freq/max_freq (generic framework, both trees have the files). Neither Android nor mainline has a "hold low" interface of its own beyond that. For the try that needs the clock LOW: 
  N=/sys/class/devfreq/<the node whose device is 1da4000.ufshc>
  cat $N/available_frequencies $N/cur_freq (before)
  echo userspace > $N/governor; echo 50000000 > $N/userspace/set_freq    # mainline (ours)
  [or: echo 50000000 > $N/max_freq, governor unchanged: works on both trees]
Read back: cat $N/cur_freq (expect 50000000); the unipro clock 37500000 (debugfs clk summary gcc_ufs_unipro_core_clk, if debugfs is mounted); the link: /sys/devices/platform/soc@0/1da4000.ufshc/power_info/{gear,lane,mode} (ours; path is the node's device dir) expecting gear 1 after the scale-down, gear 3 before. A failed scale is in dmesg ("failed to scale", "old gear").
The hold is not permanent against the framework: the generic one is max_freq clamp, which survives load; userspace governor survives load too. clkscale_enable=1 would let ondemand rescale it.
What it does NOT hold: the gear (it drops to G1, see 2), runtime-PM/hibern8 states, the PHY.
Amendment for step 16 (item walleye-ufs-ramp-pin-by-userspace): replace "echo powersave > governor" with the userspace pair above (check "userspace" in available_governors, else the max_freq write), read cur_freq and power_info/gear before and after, and refuse if cur_freq is not 50000000. Do not add DEVFREQ_GOV_POWERSAVE to the config just to pass.

## 5 Not known yet
- Whether the PMC HS-G3 -> HS-G1 succeeds on this card/link (no one has run a scale-down on walleye; Android's scaling ran at 4.4 with its own path).
- The sysfs path names on the phone (devfreq node name; power_info present in our build): read from the keeper's next read-only sysfs listing.
- Lineage device tree init.walleye.rc itself (only its two lines are known from dmesg).
