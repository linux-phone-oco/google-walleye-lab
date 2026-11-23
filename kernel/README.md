# kernel - what the Pixel 2's kernel is made of

Today: patch series applied by build scripts (`audio-adsp-q6/`, `drv2624/`, `fuel-gauge-rfc/`) and a panel
driver source. Decided (docs 2026-11-23-structure-kernel-topics-and-integration): each series moves to its own
branch `topic/ITEM` in linux-phone-oco/linux, and this folder keeps only `topics.toml` - the written list of
topics the test kernel `int/walleye` is assembled from - and config fragments.
