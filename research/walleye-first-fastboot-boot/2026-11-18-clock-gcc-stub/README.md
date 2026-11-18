# clock_gcc stub: the serial supplier and the four phandle lines

Source: the first mainline dmesg (lab material mainline-shell-20261118-082300, not in git): lines 169-173 and 333, and devices-deferred.txt.
Check: `tests/host/overlay-clock-gcc.sh` (fail on z2's dtb: clocks[0]=306 stub, want 37 gcc; pass on z3's). z3 hashes: `SHA256SUMS`.
