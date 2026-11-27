# The modem ramdisk's userspace: rmtfs, qrtr-ns and tqftpserv, static aarch64 (musl), by nix, with no systemd and no glibc.
#   nix --extra-experimental-features 'nix-command flakes' build --impure -f build/modem-tools.nix -o OUT
#   (builds all four; a named tool alone: `nix build ... -f build/modem-tools.nix NAME -o OUT` with NAME rmtfs, qrtr, tqftpserv or qmicli, each OUT/bin/... - the order of the links of the one-call form is not relied on;
#   the toolchain builds on first use - about an hour with a cold cache)
# Why each override (research walleye-rmtfs-static-nix-ok): rmtfs links libudev only in sharedmem.c, so libudev-zero (a sysfs reader for systemd-less systems) stands in;
# qrtr's meson asks for systemd only to install a unit file (-Dsystemd-service=disabled) and builds a shared lib by itself (static_library);
# tqftpserv needs systemd only for its unit file, and keeps /lib/firmware (nixpkgs rewrites it to a nix path).
let
  pkgs = (import (builtins.getFlake "nixpkgs") { config = { allowUnsupportedSystem = true; allowBroken = true; }; }).pkgsCross.aarch64-multiplatform-musl.pkgsStatic;
  qrtr = (pkgs.qrtr.override { systemd = pkgs.libudev-zero; }).overrideAttrs (o: {
    buildInputs = [ ];
    mesonFlags = (o.mesonFlags or [ ]) ++ [ "-Dsystemd-service=disabled" ];
    postPatch = (o.postPatch or "") + ''
      sed -i "s/shared_library('qrtr'/static_library('qrtr'/; /version: meson.project_version(),/d" lib/meson.build
    '';
  });
in {
  rmtfs = pkgs.rmtfs.override { udev = pkgs.libudev-zero; inherit qrtr; };
  inherit qrtr;
  tqftpserv = (pkgs.tqftpserv.override { inherit qrtr; systemdLibs = pkgs.zstd; }).overrideAttrs (o: { postPatch = ""; });
  # qmicli (item walleye-modem-qmicli-in-image): libqmi 1.38 with its QRTR transport (libqrtr-glib), no introspection, no man pages, no udev (libgudev is for the MBIM/QMI-over-cdc-wdm hotplug that the ramdisk does not have)
  # (no gi-docgen means the `doc` output stays empty and nix refuses the build: it is dropped from outputs)
  qmicli = (pkgs.libqmi.override { withIntrospection = false; withMan = false; }).overrideAttrs (o: { outputs = [ "out" "dev" ]; });
}
