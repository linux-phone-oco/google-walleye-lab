# Can the shipped UFS-off kernel start remoteproc_slpi? (2026-11-25, host only)

Question: the item walleye-slpi-carveout-dts asks for a UFS-off image whose ramdisk carries the SLPI firmware. Does the kernel of the shipped UFS-off images (z21: the kernel of tag int/walleye/ufs-off-e123d3bff/v7.3/1) have the drivers remoteproc_slpi needs built in?

Read: the `.config` of the kernel builds under `~/Projects/walleye-build/out-z`, `out-display-late`, `out-ufsm` (the ufs-off and ufs-on diagnostic kernels), and `build/walleye.config` of this lab.

Found (the same in all three builds): `CONFIG_QCOM_Q6V5_PAS=m`, `CONFIG_QCOM_MDT_LOADER=m`, `CONFIG_QCOM_SYSMON=m`, `CONFIG_RPMSG_QCOM_GLINK_SMEM=m`, `CONFIG_QCOM_PDR_HELPERS=m`, `CONFIG_QRTR=m`, `CONFIG_QRTR_SMD=m`, `CONFIG_QCOM_PD_MAPPER=m`;
built in: `CONFIG_REMOTEPROC=y`, `CONFIG_QCOM_SMP2P=y`, `CONFIG_RPMSG_QCOM_SMD=y`. `build/walleye.config` (newer than those builds) already sets QRTR, QRTR_SMD and QCOM_PD_MAPPER to `y` (decision 2026-11-24-plain-alpine-not-nura) but names none of the PAS, MDT loader, sysmon, glink or PDR options, so a kernel built with it still has the PAS driver as a module, and the initramfs carries no modules.

So: a ramdisk with the firmware alone does not start remoteproc_slpi on the shipped kernel. The image needs a kernel with those options `=y` (a new int tag: a config change in the kernel list) or a ramdisk with modules of a kernel build that matches the kernel byte for byte. NOT KNOWN YET which of the two is cheaper; the second needs the module tree of the tag's own build.
