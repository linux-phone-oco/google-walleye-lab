# kernel - what the Pixel 2's kernel is made of

Each series is a branch `topic/ITEM` of linux-phone-oco/linux (the clone `~/Projects/linux-phone-oco/linux`, branches pushed to
the fork), and this folder keeps `topics.toml` - the written list of topics the test kernels `int/walleye/LIST/BASE/N` are
assembled from (decision 2026-11-23-structure-kernel-topics-and-integration) - the series' records, and config fragments.
Builds follow the branch: `build/build-kernel.sh` cherry-picks the fuel gauge topic into a tree that lacks it, `build/build-display*.sh`
read the panel driver from its topic, and `tests/host/audio-adsp-q6-builds.sh` and `drv2624-builds.sh` check the topic out.

## The series, as branches (2026-11-23: each .patch file proved equal to its branch, then removed)

Proof: `git patch-id --stable` of every commit of `v7.3..BRANCH` equals that of the file of the same number, in order; the panel driver is
byte for byte the file of the branch (`cmp`). The files are in this folder's history before the commit that removed them.

| series | branch | proved equal at | what was in the folder |
|---|---|---|---|
| audio-adsp-q6 | `topic/walleye-audio-adsp-q6` | a8be02235 | 0001-0008: equal to the 8 commits |
| drv2624 | `topic/walleye-drv2624` | 51274968e | 0001-0004: equal to the 4 commits; 0005 (the walleye dts) equals `dt/msm8998-google-walleye.dts`'s own lines (it reverse-applies to it) |
| fuel-gauge-rfc | `topic/walleye-fuel-gauge-rfc` | 064940411 | 0001-0003: equal to the 3 commits |
| display-mdss-reset | `topic/walleye-mdss-reset` | 58d8fd902 | 0001: equal to the commit |
| panel-samsung-s6e3fa5.c | `topic/walleye-panel-s6e3fa5` | db21bdf17 | the driver, byte for byte (and `wip/walleye/ufs-root-display` carries the same) |

KEPT, a difference named and judged: `audio-adsp-q6/0009-*.patch` (with its `SHA256SUMS`) is the walleye board dts change, not a commit of
the topic and not yet in `dt/msm8998-google-walleye.dts` (the audio test applies it to `dt/`): it stays until the audio is enabled in the
lab's dts or the board file's change has a topic of its own. The series' `README.md` records stay as they were.

The kernel of image U-d5 (item walleye-install-ufs-root-display-image) is the branch `wip/walleye/ufs-root-display` of the fork (pushed 2026-11-23, ac7eefecb;
a worktree of the clone builds it): `topic/ufs-match-android` plus the commits of `topic/walleye-mdss-reset` and `topic/walleye-panel-s6e3fa5`. It is built by
`build/build-display-late.sh` with `EXTRA_CONFIG="build/walleye-diag.config build/walleye-ufs-root.config"` and made into the image by `build/build-ufs-root-display-image.sh`.
