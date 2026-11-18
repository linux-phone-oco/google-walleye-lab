# y1 and y2: our dtb with gpio-reserved-ranges

The images of backlog item walleye-our-dts-gpio-reserved, built on the lab by `build/build-y.sh` (49 checks ok, rebuild byte-identical). Their hashes are in `SHA256SUMS`; on the lab they are in `~/Projects/walleye-build/images-y/`.

What differs from ml6a (our kernel, our dtb without the property, no earlycon): only the dtb, which now has `pinctrl@3400000 gpio-reserved-ranges = <0 4>, <81 4>` (dt/msm8998-google-walleye.dts, `&tlmm`). The Image.gz, the header and the command line are ml6a's, byte for byte. The dtb is `msm8998-google-walleye-y.dtb`; it differs from ml6a's dtb in that one property line only (checked), and the stock dtbo overlay still merges into it.

| Image | Ramdisk (source image) | Expected (keeper's rule) |
|---|---|---|
| y1 | ml5a's: marks /dev/kmsg, `reboot -f` at once | alive: back as Android in <= 66 s (dead: ~75 s) |
| y2 | pk5l's: the gadget init (ACM shell on ttyGS0 + mass_storage window, watchdog petted, `reboot -f` after 180 s) | `1d6b:0104` (serial walleye-first-boot) on the host within ~25 s of the jump, plain `lsusb` |

Try order: y1 twice, then y2 twice; battery first (`tests/host/battery-ok.sh`); Android left on a good slot.

A note on the source of the property: the research of the fdt compare says the running vendor fdt has no such property; the alive taimen dtb and porthole's walleye dtb have it, and the bisect (x1) shows that taking it out of the alive tree kills init.

Inputs (first 16 hex of sha256): ml6a `aceb7e7a47abcf07`, ml5a `b937e8fba212c08d`, pk5l `a313a0c3fe927a4c`, dtbo `fb09009b7946bf33`; kernel tree `~/Projects/walleye-build/linux-v7.3` (v7.3), `.config` of ml6a's kernel `out-ml/.config`.
The x1..xa images (`build/build-fdt-bisect.sh`) were built from our dtb BEFORE this property; to rebuild them, feed the script that dtb (dts at the revision before this change). The f dtb now carries the property too (it includes the walleye dts).
The comment in the dts says "a cause", not "the cause": y1 is the test of whether adding it is enough.
