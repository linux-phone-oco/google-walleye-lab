#!/bin/sh
# walleye-dmesg-clean, run on the lab host: does the dtb keep the GPU's smmu (adreno_smmu, iommu@5040000) off?
# Its power domain is gpucc's GPU_GX_GDSC, which cannot switch on without a GFX rail nothing in our dts powers: six
# "gpu_gx status stuck" WARNs, a probe that defers for ever and a gcc/gpucc sync_state held (research walleye-z6-dmesg-mapped, G1).
# Passes only when the node /__symbols__/adreno_smmu names has status "disabled" in the dtb.
# Prints `pass|fail|skip walleye-dmesg-clean dtb DETAIL`; exit 0 only on pass. Reads only.
# Environment: DTB (our dtb, default the built z7 dtb).
DTB=${DTB:-$HOME/Projects/walleye-build/images-z7/msm8998-google-walleye-z7.dtb}
say() { echo "$1 walleye-dmesg-clean dtb $2"; [ "$1" = pass ]; exit $?; }
command -v fdtget >/dev/null || say skip "fdtget (dtc) not installed: not known yet"
[ -r "$DTB" ] || say skip "no dtb at $DTB: not known yet"
N=$(fdtget "$DTB" /__symbols__ adreno_smmu 2>/dev/null) || say fail "the dtb has no adreno_smmu symbol"
S=$(fdtget "$DTB" "$N" status 2>/dev/null || echo none)
[ "$S" = disabled ] || say fail "$N status is '$S', want disabled"
say pass "$N status disabled"
