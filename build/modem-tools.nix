# The modem ramdisk's userspace: rmtfs, qrtr-ns and tqftpserv, static aarch64 (musl), by nix, with no systemd and no glibc.
#   nix build -f build/modem-tools.nix -o OUT
#   (builds all four; a named tool alone: `nix build ... -f build/modem-tools.nix NAME -o OUT` with NAME rmtfs, qrtr, tqftpserv or qmicli, each OUT/bin/... - the order of the links of the one-call form is not relied on;
#   the toolchain builds on first use - about an hour with a cold cache)
# Why each override (research walleye-rmtfs-static-nix-ok): rmtfs links libudev only in sharedmem.c, so libudev-zero (a sysfs reader for systemd-less systems) stands in;
# qrtr's meson asks for systemd only to install a unit file (-Dsystemd-service=disabled) and builds a shared lib by itself (static_library);
# tqftpserv needs systemd only for its unit file, and keeps /lib/firmware (nixpkgs rewrites it to a nix path).
let
  pkgs = (import ./nixpkgs-pin.nix { config = { allowUnsupportedSystem = true; allowBroken = true; }; }).pkgsCross.aarch64-multiplatform-musl.pkgsStatic;
  qrtr = (pkgs.qrtr.override { systemd = pkgs.libudev-zero; }).overrideAttrs (o: {
    buildInputs = [ ];
    mesonFlags = (o.mesonFlags or [ ]) ++ [ "-Dsystemd-service=disabled" ];
    postPatch = (o.postPatch or "") + ''
      sed -i "s/shared_library('qrtr'/static_library('qrtr'/; /version: meson.project_version(),/d" lib/meson.build
    '';
  });
  libqrtr-glib = pkgs.libqrtr-glib.overrideAttrs (o: {
    nativeBuildInputs = builtins.filter (d: !(builtins.any (p: builtins.substring 0 (builtins.stringLength p) (d.name or "") == p) [ "gtk-doc" "gobject-introspection" "docbook" ])) (o.nativeBuildInputs or [ ]);
    mesonFlags = (o.mesonFlags or [ ]) ++ [ "-Dintrospection=false" "-Dgtk_doc=false" ];
  });
in {
  rmtfs = pkgs.rmtfs.override { udev = pkgs.libudev-zero; inherit qrtr; };
  inherit qrtr;
  tqftpserv = (pkgs.tqftpserv.override { inherit qrtr; systemdLibs = pkgs.zstd; }).overrideAttrs (o: { postPatch = ""; });
  # qmicli (items walleye-modem-qmicli-in-image, walleye-modem-qmicli-qrtr-enabled): libqmi 1.38 with its QRTR transport, no introspection, no man pages, no udev (libgudev is for the MBIM/QMI-over-cdc-wdm hotplug that the ramdisk does not have)
  # nixpkgs' libqmi expression sets `-Dqrtr` (libqmi 1.38 meson_options.txt) from withIntrospection and adds libqrtr-glib only then: with introspection off the transport was left out and `-d qrtr://0` read as a file path.
  # So qrtr is switched on by hand and libqrtr-glib (static, its introspection and docs off) is a build input.
  # (no gi-docgen means the `doc` output stays empty and nix refuses the build: it is dropped from outputs)
  qmicli = (pkgs.libqmi.override { withIntrospection = false; withMan = false; }).overrideAttrs (o: {
    outputs = [ "out" "dev" ];
    buildInputs = (o.buildInputs or [ ]) ++ [ libqrtr-glib ];
    mesonFlags = (builtins.filter (f: f != "-Dqrtr=false") (o.mesonFlags or [ ])) ++ [ "-Dqrtr=true" ];
  });
}
