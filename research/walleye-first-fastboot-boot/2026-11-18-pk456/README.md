Hashes and recipe of the pk4, pk5, pk6 images (one swap each between pk3 = porthole's kernel + taimen dtb and ml6a = our kernel + our dtb); images live on the lab machine in ~/Projects/walleye-build/images-pk456.

Built by `build/build-pk456.sh PK3_BOOT_IMG ML6A_BOOT_IMG ML5B_BOOT_IMG OUT_DIR` (31 checks, all ok; rebuild byte-identical). All: pk3's header, command line without earlycon, AVB footer, 32 MiB.
- pk4: porthole's Image.gz + OUR dtb (ml6a's, -@, overlay stubs), ramdisk init-ml5a (reboot at once)
- pk5: OUR Image.gz + porthole's taimen dtb (pk3's), ramdisk init-ml5a
- pk6: pk3's kernel blob + ramdisk of ml5b (3 s per score point, max 21 s)
Expected return (fastboot boot to adb; baselines: pk3 48 and 63 s, Android alone ~50 s with an init that reboots at once; no-init pattern 75 s): pk4, pk5 about 40-65 s = init ran, about 75 s = it did not. pk6: base (pk3's own time) + 3 s per score point, max +21 s, if init ran; about 75 s whatever the score = it did not.
Reading: pk4 fast and pk5 dead = our dtb is fine and our kernel/config is the cause; pk4 dead and pk5 fast = our dtb; both dead = both parts of ours matter or the image path; both fast = the pairing.
