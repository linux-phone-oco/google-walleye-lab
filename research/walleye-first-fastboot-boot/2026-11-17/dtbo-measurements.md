Measurements for walleye-dtbo-overlay-merge-why-how-it-stands (2026-11-17; host only, nothing written to the phone).
dtbo.img: ~/Projects/walleye-build/control/dtbo.img, sha256 fb09009b...b8f6 (LineageOS 22.2 20261113, official). Read with dtc, fdtget, fdtoverlay.
- Table: magic d7b7ab1e, 13 entries of 32 bytes (size, offset, id, rev). Entries 0-5 id 0x1bc rev 0,1,2,3,4,0x80; 6-12 ids 0x190,0x490,0x590,0x690,0x790,0x890,0xa90. The control's cmdline androidboot.dtbo_idx=3 (id 0x1bc rev 3), set by the bootloader itself.
- Overlay entries have no compatible, msm-id or board-id; they are pure fragments (entry 3: 147 fragments, __fixups__, __local_fixups__, __symbols__). Every fragment's target is a label (target = <&cci>), no target-path.
- entry 3's __fixups__ names 137 labels (cci, tlmm, soc, mdss_mdp, pm8998_gpios, clock_gcc, ...).
- stock dtb (bisect/parts/stock.dtb): __symbols__ with 986 labels, all 137 present. fdtoverlay stock + entry 3 -> merges (463118 bytes).
- our dtb (bisect/parts/our.dtb, built without -@): no __symbols__. fdtoverlay -> FDT_ERR_BADOFFSET "base blob does not have a '/__symbols__' node".
- Our dtb recompiled with -@ would carry only labels the mainline sources define: of the 137, ~14 appear as labels in msm8998*.dtsi/pm8998/pmi8998 (rough grep of linux v7.3); ~123 would still be missing, and our soc node is soc@0, not soc.
- Test: tests/host/dtbo-overlay-merges.sh: fail on our dtb, pass on stock dtb.
