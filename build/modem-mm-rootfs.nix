# The ModemManager rootfs pushed into the modem image's RAM (decision 2026-11-27-walleye-mm-rootfs-built-by-nix): the aarch64 (glibc) closure of ModemManager (polkit off, systemd off, QRTR on),
# dbus, mmcli, qmicli (libqmi with its QRTR transport) and udevd (eudev), packed as ONE tar: /nix/store/... closure, `env/` a symlink farm of the programs (env/bin, env/sbin), `etc`, `var/run`, `proc`, `sys`, `dev`, `run`, `tmp` empty mount points.
#   nix --extra-experimental-features 'nix-command flakes' build --impure -f build/modem-mm-rootfs.nix -o OUT      (OUT/modem-mm-rootfs.tar)
#   sha256sum OUT/modem-mm-rootfs.tar > build/modem-mm-rootfs.sha256 (the name inside: modem-mm-rootfs.tar), then TAR=OUT/modem-mm-rootfs.tar sh tests/host/modem-rootfs-selftest.sh
# The tar is built with sorted names, mtime 0 and owner 0: two builds are byte-equal (the test compares with the record). It stays out of git, like the firmware tar; no modem storage and no firmware inside.
# Why each override: nixpkgs' ModemManager sets -Dqrtr from withIntrospection (a cross build has none), so qrtr is switched on by hand with libqrtr-glib as an input, as modem-tools.nix does for qmicli;
# withPolkit/withSystemd off (the phone has neither: the research's own fallback is -Dpolkit=no); libqmi gets the same qrtr switch.
let
  pkgs = (import (builtins.getFlake "nixpkgs") { config = { allowUnsupportedSystem = true; allowBroken = true; }; }).pkgsCross.aarch64-multiplatform;
  host = import (builtins.getFlake "nixpkgs") { };
  libqrtr-glib = pkgs.libqrtr-glib.overrideAttrs (o: {
    nativeBuildInputs = builtins.filter (d: !(builtins.any (p: builtins.substring 0 (builtins.stringLength p) (d.name or "") == p) [ "gtk-doc" "gobject-introspection" "docbook" ])) (o.nativeBuildInputs or [ ]);
    mesonFlags = (o.mesonFlags or [ ]) ++ [ "-Dintrospection=false" "-Dgtk_doc=false" ];
  });
  libqmi = (pkgs.libqmi.override { withIntrospection = false; withMan = false; }).overrideAttrs (o: {
    outputs = [ "out" "dev" ];
    buildInputs = (o.buildInputs or [ ]) ++ [ libqrtr-glib ];
    mesonFlags = (builtins.filter (f: f != "-Dqrtr=false") (o.mesonFlags or [ ])) ++ [ "-Dqrtr=true" ];
  });
  modemmanager = (pkgs.modemmanager.override { inherit libqmi; withPolkit = false; withSystemd = false; withIntrospection = false; }).overrideAttrs (o: {
    buildInputs = (o.buildInputs or [ ]) ++ [ libqrtr-glib ];
    mesonFlags = (builtins.filter (f: f != "-Dqrtr=false") (o.mesonFlags or [ ])) ++ [ "-Dqrtr=true" ];
    doInstallCheck = false;
  });
  env = pkgs.buildEnv {
    name = "modem-mm-env";
    paths = [ modemmanager libqmi pkgs.dbus pkgs.eudev pkgs.busybox ];
    pathsToLink = [ "/bin" "/sbin" "/libexec" "/share/dbus-1" "/lib/udev" ];
    ignoreCollisions = true;
  };
  closure = host.closureInfo { rootPaths = [ env ]; };
in
host.runCommand "modem-mm-rootfs" { nativeBuildInputs = [ host.gnutar ]; } ''
  mkdir -p $out root/etc root/var/run root/proc root/sys root/dev root/run root/tmp
  ln -s ${env} root/env
  sed 's|^/||' ${closure}/store-paths > list
  # the closure from /, then the mount points and the symlink from root/ (each -C applies to the names after it)
  R=$PWD
  tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner -cf $out/modem-mm-rootfs.tar -C / --files-from="$R/list" -C "$R/root" .
''
