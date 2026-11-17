#!/bin/sh
# A static busybox 1.36.1 (GPL-2.0, from its source tarball, checked against its published sha256).
#
#   build-busybox.sh ARCH WORK_DIR        ARCH: arm64 (cross, the phone) or x86_64 (the self-test under QEMU)
#
# Leaves WORK_DIR/busybox-ARCH. Needs a C compiler for the arch (aarch64-linux-gnu-gcc for arm64) and static libc.
set -eu
ARCH=${1:?usage: build-busybox.sh arm64|x86_64 WORK_DIR}
W=${2:?usage: build-busybox.sh arm64|x86_64 WORK_DIR}
V=1.36.1
SHA=b8cc24c9574d809e7279c3be349795c5d5ceb6fdf19ca709f80cde50e47de314
case $ARCH in
  arm64) CROSS=aarch64-linux-gnu- ;;
  x86_64) CROSS= ;;
  *) echo "build-busybox: arch $ARCH" >&2; exit 2 ;;
esac
mkdir -p "$W/dl"
T=$W/dl/busybox-$V.tar.bz2
[ -f "$T" ] || curl -sfL -o "$T" "https://busybox.net/downloads/busybox-$V.tar.bz2"
echo "$SHA  $T" | sha256sum -c - >/dev/null
S=$W/busybox-$ARCH.src
rm -rf "$S"; mkdir -p "$S"; tar -xjf "$T" -C "$S" --strip-components=1
cd "$S"
make -s CROSS_COMPILE="$CROSS" defconfig
# static; tc does not build against current kernel headers and is not needed
sed -i -e 's/^# CONFIG_STATIC is not set/CONFIG_STATIC=y/' -e 's/^CONFIG_TC=y/# CONFIG_TC is not set/' .config
make -s CROSS_COMPILE="$CROSS" EXTRA_CFLAGS="-std=gnu11 -Wno-error" oldconfig </dev/null >/dev/null
make -s -j"$(nproc)" CROSS_COMPILE="$CROSS" EXTRA_CFLAGS="-std=gnu11 -Wno-error" busybox
cp busybox "$W/busybox-$ARCH"
file "$W/busybox-$ARCH" | cut -c1-200
