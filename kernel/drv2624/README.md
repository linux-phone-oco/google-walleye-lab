# DRV2624 haptics: the series and the chip's record

**The patch files are gone (2026-11-23): the series is the branch `topic/walleye-drv2624` of linux-phone-oco/linux, proved equal to them (kernel/README.md).** The numbering below follows the commits' order.

Brief: the vibration motor driver of the Pixel 2 (item walleye-drv2624-driver), taken from porthole's tree onto kernel.org v7.3 as five patches, each upstreamable on its own (decision 2026-11-19-walleye-audio-port-shape).
`tests/host/drv2624-builds.sh KERNEL_GIT_TREE` builds the four commits of `topic/walleye-drv2624` and the driver; the dts patch 0005 is gone because `dt/msm8998-google-walleye.dts` has its lines (checked at removal).

| file | what | author |
|---|---|---|
| 0001 | the binding, `ti,drv2624` (porthole has none; written here from the properties the driver reads) | grenudi |
| 0002 | the driver, `drivers/input/misc/drv2624.c`, porthole's patch 0018 unchanged but for one re-fitted Kconfig context line | Giuseppe Maggio (Signed-off-by kept), ported by grenudi |
| 0003 | our fixes: cancel queued play work and clear GO on unbind; an explicit `ti,ol-lra-frequency = <0>` no longer divides by zero | grenudi |
| 0004 | more fixes: the frequencies are range-checked at probe (the drive time and open-loop period registers cut a value silently), the amplitude is read once, `depends on GPIOLIB || COMPILE_TEST` | grenudi |
| 0005 | the walleye dts node on QUP6 (applies to `dt/msm8998-google-walleye.dts`, path as in the kernel; the test applies it in reverse to that file) | grenudi |

Source: porthole-dev/pmaports, `device/testing/linux-postmarketos-qcom-msm8998-7.2/0018-Input-drv2624-add-TI-DRV2624-haptics-driver.patch` (GPL-2.0-only, `Assisted-by: LLM`, ported by its author from TI's driver: not clean-room of the vendor code; the author settles the lab's line, research walleye-audio-haptics-porthole-has-it).

## What any phone with this part needs

| | |
|---|---|
| Part | TI DRV2624, haptic driver for an LRA or ERM, I2C, address 0x5a, nRESET pin |
| Driver stands | mainline: none (drv260x, drv2665, drv2667 only). porthole: patch 0018. ours: this series, binding included |
| Firmware | none. The part has a waveform ROM; this driver plays real-time (RTP) amplitudes only, nothing is loaded |
| Quirks | reset the part, wait for its boot (2 ms) before the first read; the id register 0x00 reads 0x0? in the high nibble (the driver checks the high nibble is 0); one amplitude only, so the two rumble magnitudes collapse; the sequencer, waveform RAM, diagnostics and auto-calibration are absent |
| Supply | not read for walleye yet (neither node nor driver holds a regulator); if the rail is a PMIC regulator that nothing else enables, the part stays off |
| Needs from the board | the bus and address, the reset gpio (active low), and the motor's numbers as raw register values: actuator (0 ERM, 1 LRA), loop (0 closed), rated voltage, overdrive clamp, LRA frequency, wave shape |
| Walleye's values | i2c `blsp1_i2c6` (QUP6, 0xc17a000, gpio43/44), 0x5a, reset gpio95; LRA, closed loop, rated 77, clamp 165, 155 Hz, square. The stock overlay's own, same as porthole's taimen node |
| Proof test | build: `tests/host/drv2624-builds.sh` (host). Bus: 0x5a answers and register 0 reads a high nibble of 0 (item walleye-haptics-i2c-answers, read-only from Android). Felt: force-feedback rumble from a shell moves the motor (item walleye-haptics-felt, a person) |
| Bring-up steps it covers | the bus is up, the part answers, the driver binds (`dmesg`, `/sys/bus/i2c/devices/6-005a`, an input device named `drv2624:haptics`), a rumble effect is played |

The overlay collision: the stock overlay's fragment for this part targets the label `i2c_6`, which in our dtb is a stub node (`msm8998-google-walleye-overlay-stubs.dtsi`), so its `drv2624@5a` lands in the stub and never on our `vibrator@5a`.

Not done here: the walleye values are not yet keys of `device.toml` for a library proof step (docs #638); the series is not in a boot image; no phone was touched.
