# dt - device-tree sources for the Pixel 2, built into the kernel by build/

`msm8998-google-walleye.dts` is the board; the others are variants of single tries. The board's dts is on
its way upstream (docs `upstream/linux-arm-msm/walleye-board-dts/`) and moves to its kernel branch
`topic/ITEM` in linux-phone-oco/linux; a try's variant moves to a `wip/walleye/...` branch (docs decision
2026-11-23-structure-kernel-topics-and-integration). Until then, `build/build-kernel.sh` copies these in.
