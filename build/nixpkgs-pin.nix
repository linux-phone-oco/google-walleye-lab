# The one nixpkgs revision the modem builds (modem-tools.nix, modem-mm-rootfs.nix) take their packages from, so each recorded sha256 holds for it and not for whatever the registry follows.
# Source: `nix flake metadata nixpkgs --json` on 2026-11-27: locked rev 619c39f6eac5d1ed6b8f073a9414baf7034b2356 (nixpkgs-26.11pre1126808), tarball from releases.nixos.org, narHash below.
# To move it: change url and sha256 together, rebuild, and re-record every hash that moves with the image rows that pack it (said in the PR).
import (builtins.fetchTarball {
  url = "https://releases.nixos.org/nixpkgs/nixpkgs-26.11pre1126808.619c39f6eac5/nixexprs.tar.zst";
  sha256 = "sha256-3Xq+XlDu5l+UJ0cc+Jigs84+tz7Qu92NgXVHgJwC52M=";
})
