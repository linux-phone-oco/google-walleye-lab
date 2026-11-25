#!/bin/sh
# walleye-qrtr-lookup (item walleye-slpi-boot-procedure), run on the lab host: does the pinned qrtr source build, static for aarch64 and twice equal, to the sha256 declared for the SLPI ramdisk's qrtr-lookup? With RAMDISK=FILE (the cpio.xz) the binary inside it is also compared to that sha256: without it the ramdisk is NOT proved to hold this build (it is a seed with no recipe).
# Same on this host only: the toolchain is the host's aarch64-linux-gnu gcc and static glibc, not pinned (rootfs/pins says the same); a failure on another host may be the compiler, not the source.
# The ramdisk spec lists /bin/qrtr-lookup, and its declared sha256 is build/initramfs-slpi/qrtr-lookup.sha256.
# Usage: qrtr-lookup.sh [TARBALL]  Environment: QRTR_TARBALL (the archive; default the one the rootfs build downloads, under $AGENT_BUILD/../*/rootfs/dl or ~/.cache/agent-build), CROSS (default aarch64-linux-gnu-), DECLARED (a file in place of the declared sha256 line)
# Prints `pass|fail|skip walleye-qrtr-lookup DETAIL`; exit 0 only on pass. Writes only a temp dir.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-qrtr-lookup $2"; [ "$1" = pass ]; exit $?; }
SPEC=$HERE/build/initramfs-slpi/spec
DECL=${DECLARED:-$HERE/build/initramfs-slpi/qrtr-lookup.sha256}
grep -qx 'file /bin/qrtr-lookup qrtr-lookup 755 0 0' "$SPEC" || say fail "$SPEC does not list /bin/qrtr-lookup"
[ -r "$DECL" ] || say fail "no declared sha256 at $DECL"
want=$(cut -d' ' -f1 "$DECL")
[ "${#want}" = 64 ] || say fail "$DECL does not start with a sha256"
pin=$(awk '$1=="qrtr" {print $2, $3}' "$HERE/rootfs/pins")
commit=${pin% *}; psha=${pin#* }
[ -n "$commit" ] && [ -n "$psha" ] || say fail "rootfs/pins has no qrtr row"
C=${CROSS:-aarch64-linux-gnu-}
command -v "${C}gcc" >/dev/null || say skip "no ${C}gcc: not known yet"
tb=${1:-${QRTR_TARBALL:-}}
[ -n "$tb" ] || tb=$(ls "$HOME"/.cache/agent-build/*/rootfs/dl/qrtr-"$commit".tar.gz 2>/dev/null | head -n1)
[ -r "$tb" ] || say skip "no archive of qrtr $commit (give TARBALL): not known yet"
[ "$(sha256sum "$tb" | cut -d' ' -f1)" = "$psha" ] || say fail "$tb is not the archive rootfs/pins pins ($psha)"
T=$(mktemp -d) || say skip "no temp dir"; trap 'rm -rf "$T"' EXIT INT TERM
tar -xzf "$tb" -C "$T" || say fail "cannot unpack $tb"
S=$(echo "$T"/qrtr-*)
for n in 1 2; do
  "${C}gcc" -O2 -static -ffile-prefix-map="$S"=. -s -o "$T/q$n" "$S/src/lookup.c" "$S/lib/logging.c" "$S/lib/qmi.c" "$S/lib/qrtr.c" -I"$S/include" >"$T/log" 2>&1 || say fail "build $n: $(tail -n1 "$T/log")"
done
a=$(sha256sum "$T/q1" | cut -d' ' -f1); b=$(sha256sum "$T/q2" | cut -d' ' -f1)
[ "$a" = "$b" ] || say fail "two builds differ: $a $b"
[ "$a" = "$want" ] || say fail "the build is $a, declared $want"
f=$(file -b "$T/q1")
case "$f" in *"ARM aarch64"*"statically linked"*) ;; *) say fail "file says: $f" ;; esac
if [ -n "${RAMDISK:-}" ]; then
  [ -r "$RAMDISK" ] || say fail "no ramdisk at $RAMDISK"
  mkdir "$T/rd" && (cd "$T/rd" && xz -dc "$RAMDISK" | cpio -id --quiet bin/qrtr-lookup) || say fail "cannot extract bin/qrtr-lookup from $RAMDISK"
  [ "$(sha256sum "$T/rd/bin/qrtr-lookup" | cut -d" " -f1)" = "$want" ] || say fail "the ramdisk's qrtr-lookup is not the declared build"
  in_rd=", and the ramdisk's copy is equal"
else in_rd=", the ramdisk NOT read (give RAMDISK)"; fi
say pass "qrtr $commit built twice equal: $a, $f, gcc $("${C}gcc" -dumpfullversion)$in_rd"
