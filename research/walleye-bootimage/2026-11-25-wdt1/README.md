# Captured 2026-11-25 on the lab host, lpoco-bringup (installed), builder walleye-builder: image ensure wdt1-b --device device.toml --seed SEED --cache CACHE --rebuild.
# SEED held boot-z20.img and the parts unpacked from the shipped images (`image unpack` of boot-wdt1-a.img and boot-wdt1-b.img; kernel = the unpacked kernel file less its appended dtb: first 15998115 bytes for a, 15650181 for b).
# Unpacked: appended.dtb 1e167515 and ramdisk a491e1a2 equal in wdt1-a and wdt1-b; only the kernel differs (31cecab2 for a, 5fd97720 for b). Both headers' command line is the one in [images.wdt1-a.recipe].
# Result (ensure-wdt1.txt): wdt1-a and wdt1-b "rebuilt from its recipe, equal to its declared sha256" (9dfea096, 15f5e97d). The control of wdt1-a is z20, the control of wdt1-b is wdt1-a.
# NOT LANDED in device.toml: the recipe rows below rebuilt equal, but make land-check refused them: wdt1-a's header command line is 168 bytes, over the 160 the bootloader keeps
# (tests/host cmdline-budget; only a NOT-TO-BOOT image may be excused, and wdt1-a/b are in [kernels] test). The rows were:
#   [images.wdt1-a.recipe]  control = "z20"  kernel = "31cecab2abea68a47c0ff25eec59c99d6eac95c3062a6a90f25f2fbaddf16037"  dtb = "1e167515adb299579871bb4b0f0f8404924e5ff2f8dc7f7920ae19f592472a1a"
#                           ramdisk = "a491e1a2c263d78301346862bd1c83d9736e0bc4a13352fd696e2d58ea0ce636"  cmdline = the header line (clk_ignore_unused ... dyndbg="file drivers/watchdog/pm8916_wdt.c +p")
#   [images.wdt1-b.recipe]  control = "wdt1-a"  kernel = "5fd9772018032bff545a696d6c4926aa940e0c3187c557030aaa7897b734bce3"
# Seed parts: kernel = unpacked kernel less appended dtb (a: first 15998115 bytes, b: first 15650181). Next: a shorter dyndbg (a new image, new sha256) or the images retired from [kernels] test.
