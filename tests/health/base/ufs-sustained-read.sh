#!/bin/sh
# walleye's own check: a 1024 MiB O_DIRECT read of the card's first disk ends while the shell keeps answering.
# Fails until the cause in research walleye-ufs-silent-under-load-causes-ranked is found and removed (backlog walleye-ufs-silent-ramped-pinned-try).
# Read only. Runs only where the UFS-on image is booted with the author's go: on any other kernel the card is not there and the check shows broken.
. /usr/lib/devlab/health/lib.sh
d=/dev/sda
[ -b "$d" ] || { check ufs-sustained-read storage-ufs false "no $d: the UFS-on image is not what runs"; exit 0; }
out=$(timeout 60 dd if="$d" of=/dev/null bs=1M count=1024 iflag=direct 2>&1)
rc=$?
check ufs-sustained-read storage-ufs "$([ "$rc" = 0 ] && echo true)" "dd rc=$rc: $(printf '%s' "$out" | tail -n1)"
