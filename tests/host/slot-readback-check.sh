#!/bin/sh
# walleye-slot-readback-check, run on the lab host: is DIR a complete read-back of the Pixel 2's slot partitions (the way back of the install
# stage: boot, dtbo, vbmeta, misc, system, vendor of both slots)? The same rules as bootchain-readback-check.sh, with slot-parts.list.
# Fails until the keeper has read them back (item walleye-install-slot-readback). Reads files only.
#
#   tests/host/slot-readback-check.sh DIR [DIR2]
HERE=$(cd "$(dirname "$0")" && pwd)
LIST=$HERE/slot-parts.list NAME=walleye-slot-readback-check exec sh "$HERE/bootchain-readback-check.sh" "$@"
