# Which UFS card is in walleye, learned without probing it from our kernel (2026-11-18)

Routes tried, read-only, phone on Android (LineageOS-based, adb as uid shell):
- getprop: ro.boot.hardware.ufs=64GB,Hynix; bootdevice 1da4000.ufshc. Vendor known, model and firmware revision NOT.
- /sys/block/sda/device/{vendor,model,rev}, /proc/scsi/scsi, scsi_host0 proc_name/unique_id: permission denied (shell user).
- /sys/devices/platform/soc/1da4000.ufshc (clkgate, rpm_lvl, ...): unreadable; no ufs class dir; dmesg: klogctl denied.
- dumpsys diskstats/storaged: sizes and latency only, no identity.
- Web search (public pages: LineageOS wiki, device trees): no page names the walleye card's model or firmware.
- Google factory image: bootloader.img does not carry the card's inquiry data; not a route.
Not tried (change the phone's state, so the keeper's): `fastboot getvar all` / `fastboot oem device-info` in bootloader (UFS identity not known to be exposed there); a root shell from the keeper's recovery (adb root in a recovery, then read /sys/block/sda/device/{model,rev} and the ufs descriptors under the host's sysfs).

Answer: not obtainable without root; vendor Hynix only. The Sony quirk set singles out Hynix as a vendor, so the backport of the Hynix quirks does not wait on the model.
