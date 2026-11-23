# ADSP, q6 and Quaternary MI2S: the series and the part's record

**Patches 0001-0008 are gone (2026-11-23): they are the branch `topic/walleye-audio-adsp-q6` of linux-phone-oco/linux, proved equal to them (kernel/README.md); 0009, the walleye dts, stays here as a file.** The numbering below follows the commits' order.

Brief: the audio DSP path of the Pixel 2 (item walleye-adsp-q6-mi2s), taken from porthole's tree onto kernel.org v7.3 as nine patches in four groups, each upstreamable on its own (decision 2026-11-19-walleye-audio-port-shape).
`tests/host/audio-adsp-q6-builds.sh KERNEL_GIT_TREE` proves the first eight apply, the sound objects build without a warning and the walleye dtb carries the nodes; `SHA256SUMS` pins all nine.

| file | what | author |
|---|---|---|
| 0001 | binding: `qcom,msm8998-sndcard` added to qcom,sm8250.yaml | Giuseppe Maggio (Signed-off-by kept; porthole 0085) |
| 0002 | the msm8998 machine driver `sound/soc/qcom/msm8998.c` (+ lpass-msm8998 variant data) | Jami Kettunen (porthole 0023) |
| 0003 | const on the lpass-msm8998 variant pointers | Giuseppe Maggio (porthole 0043) |
| 0004 | q6afe: the I2S word-clock source was inverted | Giuseppe Maggio (porthole 0075) |
| 0005 | the machine driver's bring-up module parameters dropped | Giuseppe Maggio (porthole 0081) |
| 0006 | msm8998.dtsi: the APR services (q6core, q6afe + dais, q6asm + dais, q6adm + routing) under the ADSP glink edge | AngeloGioacchino Del Regno (porthole 0082) |
| 0007 | msm8998.dtsi: slimbus, disabled | AngeloGioacchino Del Regno (porthole 0083) |
| 0008 | msm8998.dtsi: the voice services MVM, CVS, CVP | Giuseppe Maggio (porthole 0159) |
| 0009 | the walleye dts (applies to `dt/msm8998-google-walleye.dts`; its path is the kernel's): the ADSP on, protection-domain deleted on all seven services (porthole 0160 for the voice three), LPASS SMMU on, MultiMedia1-4 and Quaternary MI2S RX on SD1, its two pin groups as `quat_mi2s_pins` and `quat_mi2s_sd1_pins` (porthole's `quat_mi2s_active` and `quat_mi2s_sd1_active` collide with labels of `dt/msm8998-google-walleye-overlay-stubs.dtsi`; the sound card node of the amplifier series must use ours; nothing references them until it comes) | Casey Connolly (0027), Giuseppe Maggio (0084, 0160), assembled by grenudi |

Source: porthole-dev/pmaports, `device/testing/linux-postmarketos-qcom-msm8998-7.2/` (GPL-2.0, ported by their author; the lab's line on LLM-assisted patches is the author's, research walleye-audio-haptics-porthole-has-it). Patches 0006-0008 carried a fuzz-3 refit: v7.3's msm8998.dtsi has the same nodes about 1000 lines earlier.

## What any phone with this part needs

| | |
|---|---|
| Part | MSM8998's audio DSP (Hexagon, the "lpass" remoteproc, qcom,msm8998-adsp-pas) with the APR/q6 stack on it; Quaternary MI2S is one of its serial audio ports |
| Driver stands | mainline: the remoteproc, q6core/q6afe/q6asm/q6adm/q6routing, the LPASS SMMU node are in; the msm8998 machine driver, the APR/slimbus nodes of msm8998.dtsi and the sm8250 binding entry are not. porthole: 0023, 0027, 0043, 0075, 0081-0085, 0159, 0160. ours: this series |
| Firmware | the DSP's own, `adsp.mbn` (and its .mdt/.bNN pieces), from the phone's vendor partition by the keeper's provisioning; never in git (decision 2026-11-19-walleye-tas2557-firmware-stays-on-the-phone covers the amp's blob; the DSP's follows the same rule, `[layer.firmware]` of device.toml) |
| Quirks | Google's DSP firmware does not host the "avs/audio" protection domain: the locator answers, never lists it, so apr_probe() waits forever; delete `qcom,protection-domain` from the seven services (the four audio ones and MVM, CVS, CVP) in the board file. The LPASS SMMU must be on: without it q6asm's DMA address is a raw physical one above 4 GB the DSP cannot reach and the SoC dies when it touches the buffer. The sound card defers until its amplifier (the TAS2557 group) has probed. Routing names the codec's widgets: a widget that does not exist hangs the boot at the splash (porthole's measurement) |
| Supply | none of its own in the series; the codec's supplies come with the WCD934x group |
| Needs from the board | the firmware name, the protection domain removed from every service, the SMMU on, the MultiMedia frontends, the MI2S port with its data lines (walleye: SD1 only, the SoC master), the pin groups (gpio58/59 clocks driven high while idle, gpio61 data) |
| Not here | the `sound` card node: it names `&speaker_amp` of the TAS2557 series (walleye-tas2557-driver) and the codec's slimbus node (walleye-wcd934x-slimbus); porthole's patch 0084 is split along those lines |
| Proof test | build: `tests/host/audio-adsp-q6-builds.sh` (host). Phone: `/proc/asound/cards` lists a card (needs `sound_cards` in device.toml, item walleye-adsp-q6-mi2s; reached with the amp); before that, `/sys/class/remoteproc/*/state` of the ADSP reads `running` |
| Bring-up steps it covers | the ADSP firmware loads and boots, the APR services register (`/sys/bus/apr`), the q6 dais appear, the card binds once the amp is up |

Not done here: the walleye values are not yet keys of `device.toml` for a library proof step (docs #638); `dt/msm8998-google-walleye.dts` is unchanged (the 0009 patch is a file, applied by the try's build); no image, no phone touched.
