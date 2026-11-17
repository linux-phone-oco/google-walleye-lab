#!/bin/sh
# pk3 and ml6a (backlog walleye-no-earlycon): pk1 and ml5a with ONLY the kernel command line changed: earlycon=... and any console=ttyMSM*/ttyHS* words
# removed (androidboot.console=ttyMSM0 is a stock androidboot word and stays). Host only; nothing touches the phone.
#   build-nocon.sh PK1_BOOT_IMG ML5A_BOOT_IMG OUT_DIR
# Writes OUT_DIR/{boot-pk3.img,boot-ml6a.img,SHA256SUMS}; checks (ok/FAIL, exit 1 on a FAIL) that kernel, dtb (inside the kernel blob), ramdisk and
# every header word but the command line equal the source's, that the command line lost exactly the earlycon/console words, and that the AVB footer verifies.
set -eu
PK1=${1:?usage: build-nocon.sh PK1_BOOT_IMG ML5A_BOOT_IMG OUT_DIR}
ML5A=${2:?}
OUT=${3:?}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$PK1" "$ML5A"; do [ -s "$f" ] || { echo "build-nocon: missing $f" >&2; exit 1; }; done
build_one() { # SRC NAME
S=$1; N=$2
INFO=$(unpack_bootimg --boot_img "$S" --out "$T/p$N")
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
OLD=$(field 'command line args')
NEW=""
set -f
for w in $OLD; do
  case $w in earlycon|earlycon=*|console=ttyMSM*|console=ttyHS*) ;; *) NEW="${NEW:+$NEW }$w" ;; esac
done
echo "$N cmdline was: $OLD"
echo "$N cmdline now: $NEW"
PSIZE=$(stat -c %s "$S")
AV=$(avbtool info_image --image "$S")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
IMG=$OUT/boot-$N.img
mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
  --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
  --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$NEW" \
  --kernel "$T/p$N/kernel" --ramdisk "$T/p$N/ramdisk" -o "$IMG"
avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
  --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$N")
hdr() { echo "$1" | grep -v -e '^command line args'; }
ck "$N kernel blob (Image.gz + dtb) bytes equal the source's" "$(cmp -s "$T/p$N/kernel" "$T/u$N/kernel" && echo same || echo differs)" same
ck "$N ramdisk bytes equal the source's" "$(cmp -s "$T/p$N/ramdisk" "$T/u$N/ramdisk" && echo same || echo differs)" same
ck "$N header (offsets, sizes, page, os word) equals the source's but the command line" "$(hdr "$U")" "$(hdr "$INFO")"
ck "$N command line differs (the one change)" "$([ "$(echo "$U" | sed -n 's/^command line args: *//p' | head -n1)" != "$OLD" ] && echo differs || echo same)" differs
ck "$N command line has no earlycon and no console=" "$(echo "$U" | sed -n 's/^command line args: *//p' | head -n1 | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
ck "$N command line keeps clk_ignore_unused ignore_loglevel panic=5" "$(echo "$U" | sed -n 's/^command line args: *//p' | head -n1 | tr ' ' '\n' | grep -cx -e clk_ignore_unused -e ignore_loglevel -e 'panic=5')" 3
ck "$N command line keeps every androidboot.* word" "$(echo "$U" | sed -n 's/^command line args: *//p' | head -n1 | tr ' ' '\n' | grep '^androidboot\.' | sort | tr '\n' ' ')" "$(echo "$OLD" | tr ' ' '\n' | grep '^androidboot\.' | sort | tr '\n' ' ')"
ck "$N size is the source's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
NOWC=$(echo "$U" | sed -n 's/^command line args: *//p' | head -n1)
ck "$N command line is the source's words minus earlycon and console=ttyMSM*/ttyHS* only" "$NOWC" "$(echo "$OLD" | tr ' ' '\n' | grep -v -e '^earlycon' -e '^console=tty\(MSM\|HS\)' | tr '\n' ' ' | sed 's/ $//')"
ck "$N AVB info equals the source's but the digest" "$(avbtool info_image --image "$IMG" | grep -v '^ *Digest:')" "$(printf '%s\n' "$AV" | grep -v '^ *Digest:')"
mkdir "$T/v$N" && cp "$IMG" "$T/v$N/boot.img"
ck "$N AVB footer verifies" "$(avbtool verify_image --image "$T/v$N/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
}
build_one "$PK1" pk3
build_one "$ML5A" ml6a
(cd "$OUT" && sha256sum boot-pk3.img boot-ml6a.img | tee SHA256SUMS)
[ $total -eq 0 ]
