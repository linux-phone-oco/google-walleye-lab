#!/bin/sh
# Proves tests/host/image-recipe.sh against a stand-in lpoco-bringup that prints the status lines the real 0.47.1 prints
# (research walleye-image-recipe-z21-rebuilt-equal, ensure-z21.txt) and a stand-in nix-store; never touches a store or a phone.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
mkdir "$T/bin"; printf '#!/bin/sh\nexit 0\n' > "$T/bin/nix-store"
stub() { # BODY: the stand-in's behaviour for `image ensure z21 --dry-run`
  printf '#!/bin/sh\ncase "$*" in *--help*) echo "image ensure"; exit 0;; esac\n%s\n' "$1" > "$T/bin/bringup"
  chmod +x "$T/bin/nix-store" "$T/bin/bringup"
}
want() { # NAME EXPECTED_WORD DEVICE
  got=$(PATH="$T/bin:$PATH" LPOCO_BRINGUP=bringup sh "$HERE/image-recipe.sh" z21 "$3" | cut -d' ' -f1 || true)
  if [ "$got" = "$2" ]; then echo "ok   $1"; else echo "FAIL $1: got $got, want $2"; total=$((total+1)); fi
}
printf '[images.z21]\npath = "a"\n[images.z21.recipe]\ncontrol = "z20"\n' > "$T/with.toml"
printf '[images.z21]\npath = "a"\n' > "$T/without.toml"
stub 'echo "z21: in the store (/nix/store/x-boot-z21.img)"'
want "recipe declared, in the store" pass "$T/with.toml"
want "no recipe table, though the store holds it" fail "$T/without.toml"
stub 'echo "z21: would be built from its recipe (/nix/store/x-boot-z21.img)"'
want "recipe declared, would be built" pass "$T/with.toml"
stub 'echo "z21: something else"'
want "an unknown status line" fail "$T/with.toml"
stub 'echo "error: control z20 not declared" >&2; exit 2'
want "ensure refuses" fail "$T/with.toml"
want "no manifest" fail "$T/none.toml"
got=$(PATH="$T/bin:$PATH" LPOCO_BRINGUP=absent sh "$HERE/image-recipe.sh" z21 "$T/with.toml" | cut -d' ' -f1 || true)
if [ "$got" = skip ]; then echo "ok   no program is a skip"; else echo "FAIL no program: got $got"; total=$((total+1)); fi
[ "$total" -eq 0 ] || { echo "$total failed"; exit 1; }
echo "all passed"
