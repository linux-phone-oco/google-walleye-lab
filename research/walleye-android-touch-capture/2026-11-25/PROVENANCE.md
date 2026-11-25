# Android b part-compare capture of "touch", 2026-11-25 (walleye-keeper, read only)
Phone: Pixel 2 walleye, stock Android on slot b, kernel 4.4.302. Made by `tests/host/part-compare.sh capture touch android DIR` (adb root restarts adbd only; debugfs was already mounted; nothing written, flashed or set). Ids redacted by the script.
One file per list line of tests/host/part-compare.d/touch.list (clk, dmesg, driver-sysfs, dt-i2c, gpio, input, pinconf, pinmux, regulator, summary-regulator) and meta.txt.
Kept as selftest fixtures (tests/fixtures/phone-text/): gpio, regulator, dt-i2c.
