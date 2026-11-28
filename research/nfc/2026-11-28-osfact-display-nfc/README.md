# The osfact display-and-NFC image, built 2026-11-28 (item walleye-osfact-image-display-and-nfc)

What is here is what the build printed, kept for the research row walleye-osfact-display-nfc-image-packed-equal:
- `config-diff-identity-vs-this-kernel.txt`: `diff` of the identity image's kernel .config (out-display-late/.config) against this build's: only RFKILL, NFC, NFC_NCI, NFC_NXP_NCI, NFC_NXP_NCI_I2C, m to y (build/walleye-nfc.config).
- `kernel-build.log`, `SHA256SUMS-display-late`: build/build-display-late.sh of lab commit e121ca26 (EXTRA_CONFIG=build/walleye-nfc.config) on the tag int/walleye/display-smmu-id/v7.3/1 (b24d9bc68235).
- `dtb-checks.txt`: build/osfact-display-nfc-dtb.sh, every check ok. `ramdisk-check.txt`: tests/host/osfact-display-nfc-ramdisk.sh. `ufs-off-gate.txt`, `image-check.txt`, `image-info.txt`: the packed boot.img.
- `SHA256SUMS-parts`: the parts and the image (kept outside git in ~/Projects/walleye-build/osfact-display-nfc and images-osfact-display-nfc).
