<!-- Generated from data/ by tad. Do not edit by hand. -->

# Upstream Items

<a id="walleye-charger-supplier-node"></a>
### walleye: model the charger's supplier (tps61099 boost converter) as a regulator so fw_devlink needs no permissive

Upstream
: linux-arm-msm

Form
: patch

State
: draft

From work
: 2026-11-18-walleye-charger-supplier-permissive-then-modelled

Domain
: devicetree

About
: walleye-first-fastboot-boot

Waits
: the proof that the charger probes (c3 with fw_devlink=permissive), then the modelled fixed regulator in our dts; upstream only if walleye's dts goes upstream

Note
: stands in for fw_devlink=permissive on the kernel command line: that is the workaround, marked WORKAROUND(walleye-charger-supplier-node) where it is set; the item ends when the dts models the supplier

Last update
: 2026-11-18
