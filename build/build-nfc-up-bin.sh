#!/bin/sh
# The nfc0 opener for the NFC ramdisk (item walleye-nfc-nfc0-opener-in-ramdisk): tools/walleye-nfc-up built static for aarch64, so it runs in the ramdisk with no libraries.
#   build-nfc-up-bin.sh OUT_DIR      writes OUT_DIR/{walleye-nfc-up,SHA256SUMS-nfc-up-bin}; the unit tests run first and stop the build when one fails
# The std library is built from the toolchain's own source (rust-src) for aarch64-unknown-linux-gnu, linked static against the cross glibc (aarch64-linux-gnu-gcc); paths are remapped, so the same source gives the same bytes.
# Needs: cargo, rust-src, aarch64-linux-gnu-gcc with its static libc. The crates come from the cargo registry (Cargo.lock pins them).
set -eu
OUT=${1:?usage: build-nfc-up-bin.sh OUT_DIR}
HERE=$(cd "$(dirname "$0")/.." && pwd)
C=$HERE/tools/walleye-nfc-up
command -v aarch64-linux-gnu-gcc >/dev/null || { echo "build-nfc-up-bin: no aarch64-linux-gnu-gcc" >&2; exit 1; }
export RUSTC_BOOTSTRAP=1   # build-std is a nightly flag; the stable compiler takes it with this
cargo test --locked --quiet --manifest-path "$C/Cargo.toml" >&2
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
RUSTFLAGS="-C target-feature=+crt-static --remap-path-prefix=$HERE=/src --remap-path-prefix=$T=/target --remap-path-prefix=$HOME/.cargo=/cargo --remap-path-prefix=$(rustc --print sysroot)=/rustc" \
  cargo build --locked --release --manifest-path "$C/Cargo.toml" --target aarch64-unknown-linux-gnu -Zbuild-std=std,panic_abort --target-dir "$T/target" >&2
mkdir -p "$OUT"
cp "$T/target/aarch64-unknown-linux-gnu/release/walleye-nfc-up" "$OUT/walleye-nfc-up"
(cd "$OUT" && sha256sum walleye-nfc-up | tee SHA256SUMS-nfc-up-bin)
