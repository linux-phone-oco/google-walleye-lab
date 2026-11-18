# Can probing the Pixel 2's UFS damage the bootloader? (2026-11-18)

## The phone, read over adb as the shell user (read-only)
- ro.boot.hardware.ufs = 64GB,Hynix; ro.boot.bootdevice = 1da4000.ufshc; bootloader mw8998-003.0085.00; slot _b.
- Vendor/model/firmware revision of the LUNs: NOT readable (sysfs scsi_device attrs are root-only, no su on the user build). Not known yet.
- scsi_host0 with LUNs 0-5 plus well-known LUNs (49456, 49476, 49488).
- (the Pixel 2 XL teardown names a Samsung KLUCG4J1 part; this phone's own bootloader says Hynix - the measurement wins for walleye.)

## What others found (researcher, read-only; sources recorded)
- Sony yoshino/tama/kumano/edo: mainline probing UFS without Sony's quirks erased the bootloader (hard brick). Mainline keeps UFS off in those dtsi.
- The Sony downstream patch (forward port, 2019) adds: UFS_DEVICE_QUIRK_NO_PURGE (refuse the purge flag/attribute: Hynix hB8aL1/hC8aL1 fw D001, Samsung KLUCG4J1 fw 0101, any device older than UFS 2.1),
  UFS_DEVICE_QUIRK_EXTEND_SYNC_LENGTH (all Hynix: PA_TxHsG1/2/3SyncLength 0x48), PA_AVAILTXDATALANES=1.
- No upstream source says WHY probing triggers the erase. Nobody published the mechanism; the author of the mainline Kumano series said he could not ask Sony. Sony bug tracker 799 (2023): "no-one dared to sacrifice their device".
- OnePlus 5/5T (MSM8998) boot from mainline UFS (ufshc okay, vcc/vccq/phy supplies): the SoC family and our rails are proven; their card is not known to be a Hynix.

## Our kernel (Linux 7.3 tree in ~/Projects/walleye-build, read)
- Core ufs_fixups for SK hynix: HOST_PA_SAVECONFIGTIME (+ HOST_VS_DEBUGSAVECONFIGTIME for model hB8aL1); ufs-qcom adds DELAY_BEFORE_LPM for Hynix.
- Missing against Sony's set: NO_PURGE, EXTEND_SYNC_LENGTH, AVAILTXDATALANES.
- No code in the driver issues the purge flag by itself: it is only reachable from userspace via ufs_bsg (UFS_RPMB_PURGE_ENABLE). Our init never opens bsg. So the purge is not the probe's doing, if our tree matches mainline; the downstream quirk filters a downstream/vendor path.
- The probe's own writes: host reset, link startup, NOP, device init flag (fDeviceInit), descriptor reads, power-mode change (HS-G1..G3, SyncLength), SCSI scan (INQUIRY/TUR/READ CAPACITY; also an autosuspend START STOP on well-known LUNs). No write/unmap/purge is sent by it.

## Way back
- If the bootloader is erased the phone is gone for fastboot. The only road is Qualcomm EDL/9008 with a firehose programmer for this device: not public; not known to be reachable on the Pixel 2 without test points. A bootloader.img from Google's public factory image is restorable only while fastboot still works - it does not help in the failing case.
- Android as shell user cannot read the block devices; no backup of the bootloader partitions from the phone is possible without root. `fastboot fetch` (bootloader/fastbootd) support: not known yet.

## Verdict
Chance: not zero, mechanism unknown. The Pixel 2's card is Hynix, the maker Sony's quirks single out (sync length for all Hynix, purge for two models). Our tree applies only two of the Hynix quirks. Failure mode: bootloader (UFS content) gone and no way back. So the z11 try as built needs the author's word; the safe step first is the build item that backports Sony's Hynix quirks (sync length, no purge, avail lanes) and a lower-power-mode-free probe, plus learning the card's model/fw with a read that does not need the probe (root-less ways: none known).
