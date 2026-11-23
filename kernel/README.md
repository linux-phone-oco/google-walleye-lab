# kernel - what the Pixel 2's kernel is made of

Today: patch series applied by build scripts (`audio-adsp-q6/`, `drv2624/`, `fuel-gauge-rfc/`) and a panel
driver source. Decided (docs 2026-11-23-structure-kernel-topics-and-integration): each series moves to its own
branch `topic/ITEM` in linux-phone-oco/linux, and this folder keeps only `topics.toml` - the written list of
topics the test kernel `int/walleye` is assembled from - and config fragments.

The kernel of image U-d5 (item walleye-install-ufs-root-display-image) is the local branch `wip/walleye/ufs-root-display` of `~/Projects/walleye-build/linux-v7.3` (a worktree of it builds): `topic/ufs-match-android`
plus two commits, `arm64: dts: qcom: msm8998: Add the MDSS reset` (the patch of `display-mdss-reset/`, applied with `git am`) and `drm/panel: Add the Samsung S6E3FA5 command mode panel (Pixel 2)` (`panel-samsung-s6e3fa5.c`
with its Kconfig and Makefile lines). It is built by `build/build-display-late.sh` with `EXTRA_CONFIG="build/walleye-diag.config build/walleye-ufs-root.config"` and made into the image by
`build/build-ufs-root-display-image.sh`. When the topics move to linux-phone-oco/linux, these two commits become topics of their own.
