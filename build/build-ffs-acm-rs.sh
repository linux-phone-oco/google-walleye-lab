#!/bin/sh
# Build the Rust FunctionFS ACM helper, static aarch64 (musl), the way lab-tools health/build.sh builds lpoco-health:
# on the lab machine the target's standard library comes from the rustup kept in ~/.local/share/devlab-cross and the
# lab's aarch64 GCC links it.   build-ffs-acm-rs.sh OUT_FILE
set -eu
OUT=${1:?usage: build-ffs-acm-rs.sh OUT_FILE}
HERE=$(cd "$(dirname "$0")" && pwd)
T=aarch64-unknown-linux-musl
cd "$HERE/ffs-acm-rs"
if [ -d "$HOME/.local/share/devlab-cross/rustup" ] && [ ! -d "$(rustc --print target-libdir --target "$T" 2>/dev/null)" ]; then
  export RUSTUP_HOME="$HOME/.local/share/devlab-cross/rustup" CARGO_HOME="$HOME/.local/share/devlab-cross/cargo"
  nix-shell -p rustup --run "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc cargo +$(rustc -V | cut -d' ' -f2) build --release --locked --target $T"
else
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc cargo build --release --locked --target "$T"
fi
cp "target/$T/release/ffs-acm" "$OUT"
