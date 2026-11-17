Hashes and recipe of the pk3 and ml6a images (pk1 and ml5a with only the kernel command line changed); images live on the lab machine in ~/Projects/walleye-build/images-nocon.

Built by `build/build-nocon.sh PK1_BOOT_IMG ML5A_BOOT_IMG OUT_DIR` (11 checks per image, all ok; rebuild byte-identical).
Command line was: `earlycon=msm_serial_dm,0xc1b0000 clk_ignore_unused ignore_loglevel panic=5 androidboot.hardware=walleye androidboot.console=ttyMSM0 androidboot.dtbo_idx=3`
now: `clk_ignore_unused ignore_loglevel panic=5 androidboot.hardware=walleye androidboot.console=ttyMSM0 androidboot.dtbo_idx=3`
(`androidboot.console=ttyMSM0` is a stock androidboot word read by Android userspace, not a kernel `console=`; the images had no `console=` word.)
Expected on the phone: init ran = Android back on the first slot at about 36-45 s (earlycon was the hang); about 70-78 s = it was not.
