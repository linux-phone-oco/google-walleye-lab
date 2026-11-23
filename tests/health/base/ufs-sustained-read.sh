#!/bin/sh
# walleye's own check: a ramped (64, 256, 1024 MiB) O_DIRECT read of the card's first disk ends while the shell keeps answering every second.
# It passes when the 1024 MiB read finishes and no second passed without the shell running (a heartbeat gap of 3 s or more is silence);
# it fails naming the FIRST SILENT STEP (its size, its exit code, the longest gap) and keeps every step's line in /tmp/ramp.log.
# Fails until the cause in research walleye-ufs-silent-under-load-causes-ranked is found and removed (backlog walleye-ufs-silent-ramped-pinned-build).
# Read only. Runs only where the UFS-on image is booted with the author's go: on any other kernel the card is not there and the check shows broken.
# Not run by the builder: queued for the keeper with the matched image's try (UFS_DISK, UFS_LOG: test stand-ins only).
. /usr/lib/devlab/health/lib.sh
d=${UFS_DISK:-/dev/sda}
log=${UFS_LOG:-/tmp/ramp.log}
[ -e "$d" ] && { [ -b "$d" ] || [ -n "$UFS_DISK" ]; } || { check ufs-sustained-read storage-ufs false "no $d: the UFS-on image is not what runs"; exit 0; }
now() { read -r up _ < /proc/uptime; echo "${up%.*}"; }
first_silent=""
for mb in 64 256 1024; do
  echo "step $mb MiB start uptime $(now)" >> "$log"
  rm -f /tmp/ramp.rc
  (timeout 60 dd if="$d" of=/dev/null bs=1M count=$mb iflag=direct 2>/dev/null; echo $? > /tmp/ramp.rc) &
  last=$(now); gap=0
  while [ ! -s /tmp/ramp.rc ]; do
    sleep 1; t=$(now)
    [ $((t - last)) -gt $gap ] && gap=$((t - last))
    last=$t
  done
  rc=$(cat /tmp/ramp.rc)
  echo "step $mb MiB end rc=$rc longest-gap=${gap}s uptime $(now)" >> "$log"
  if [ "$rc" != 0 ] || [ "$gap" -ge 3 ]; then first_silent="$mb MiB (dd rc=$rc, longest gap ${gap} s)"; break; fi
done
check ufs-sustained-read storage-ufs "$([ -z "$first_silent" ] && echo true)" "${first_silent:+first silent step $first_silent; }steps in $log"
