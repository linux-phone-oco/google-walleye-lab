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


<a id="pmic-wdt-s2-type"></a>
### watchdog: pm8916_wdt: take the S2 reset type from qcom,s2-type

Upstream
: linux-watchdog

Form
: patch

State
: draft

From work
: walleye-wdt-s2-type-topic-branch

Domain
: watchdog

Tags
: kernel,pm8916_wdt,pm8998

About
: walleye-pmic-wdt-v1-image

Local ref
: linux-phone-oco/linux topic/pmic-wdt-s2-type (tip cbdaf1c7791e)

Waits
: the four bites (walleye-wdt-bite-pon-reasons-read) say whether a warm type is wanted; the binding patch (walleye-wdt-s2-binding-patch); the author's word and sign-off

Note
: A try, not sent: stacked on the series' patch 1 (reset type with S2 disabled), on v7.3; built W=1 clean, reviewed, never booted.

Last update
: 2026-11-27
