Hashes and recipe of the pk1 and pk2 images (porthole's mainline kernel in ml5a's image); images live on the lab machine in ~/Projects/walleye-build/images-pk.

Kernel: kernel.org linux-7.2.2.tar.xz (sha512 6ca338fd...8b) + 262 patches + config of porthole-dev/pmaports taimen-bringup @ 0d62af2af5e72333a4d0e635f8bf338f191d4cc3, built by build/pk-kernel.sh; images by build/build-pk.sh (21 checks ok, rebuild byte-identical).
Expected on the phone (from ml5a/ml5b and s4): init ran = Android back on the first slot at about 36-45 s; kernel never reached init = about 70-78 s (ml5a 75/75 s).
