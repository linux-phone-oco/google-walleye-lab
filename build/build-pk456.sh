#!/bin/sh
# pk4, pk5, pk6 (backlog walleye-bisect-porthole-vs-ours): pk3 (porthole's kernel + taimen dtb) and ml6a (our kernel + our dtb), both
# without earlycon, with exactly one part swapped each. Host only; nothing touches the phone.
#   build-pk456.sh PK3_BOOT_IMG ML6A_BOOT_IMG ML5B_BOOT_IMG OUT_DIR
# pk4 = pk3's kernel (Image.gz) + OUR dtb (ml6a's, -@ with the overlay stubs), ramdisk ml5a's init (reboot at once)
# pk5 = OUR kernel (ml6a's Image.gz) + porthole's taimen dtb (pk3's), ramdisk ml5a's init
# pk6 = pk3 with ONLY the ramdisk swapped for ml5b's (the timing init, 3 s per score point)
# Header, command line (pk3's, no earlycon) and AVB footer are pk3's throughout. Writes OUT_DIR/{boot-pk4,boot-pk5,boot-pk6}.img and SHA256SUMS;
# checks (ok/FAIL, exit 1 on a FAIL) that each image differs from its base by exactly the named part.
set -eu
PK3=${1:?usage: build-pk456.sh PK3_BOOT_IMG ML6A_BOOT_IMG ML5B_BOOT_IMG OUT_DIR}
ML6A=${2:?}
ML5B=${3:?}
OUT=${4:?}
total=0
ck() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; total=$((total+1)); fi; }
h() { sha256sum "$1" | cut -c1-16; }
mkdir -p "$OUT"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
for f in "$PK3" "$ML6A" "$ML5B"; do [ -s "$f" ] || { echo "build-pk456: missing $f" >&2; exit 1; }; done
INFO=$(unpack_bootimg --boot_img "$PK3" --out "$T/pk3")
unpack_bootimg --boot_img "$ML6A" --out "$T/ml6a" >/dev/null
unpack_bootimg --boot_img "$ML5B" --out "$T/ml5b" >/dev/null
field() { echo "$INFO" | sed -n "s/^$1: *//p" | head -n1; }
# split KERNEL_BLOB PREFIX: PREFIX.gz is the Image.gz, PREFIX.dtb the dtb that ends the blob
split() {
  o=
  for x in $(LC_ALL=C grep -obUaP '\xd0\x0d\xfe\xed' "$1" | cut -d: -f1); do
    ts=$(od -An -tu1 -j $((x+4)) -N4 "$1" | awk '{print $1*16777216+$2*65536+$3*256+$4}')
    [ $((x+ts)) -eq "$(stat -c %s "$1")" ] && o=$x
  done
  [ -n "$o" ] || { echo "build-pk456: no dtb ends $1" >&2; exit 1; }
  head -c "$o" "$1" > "$2.gz"; tail -c +$((o+1)) "$1" > "$2.dtb"
}
split "$T/pk3/kernel" "$T/pk"
split "$T/ml6a/kernel" "$T/our"
cat "$T/pk.gz" "$T/our.dtb" > "$T/blob-pk4"
cat "$T/our.gz" "$T/pk.dtb" > "$T/blob-pk5"
cp "$T/pk3/kernel" "$T/blob-pk6"
cp "$T/ml6a/ramdisk" "$T/rd-pk4"; cp "$T/ml6a/ramdisk" "$T/rd-pk5"; cp "$T/ml5b/ramdisk" "$T/rd-pk6"
PSIZE=$(stat -c %s "$PK3")
AV=$(avbtool info_image --image "$PK3")
set --
while IFS= read -r pr; do [ -n "$pr" ] && set -- "$@" --prop "$pr"; done <<EOP
$(echo "$AV" | sed -n "s/^ *Prop: \([^ ]*\) -> '\(.*\)'$/\1:\2/p")
EOP
hdr() { echo "$1" | grep -v -e 'kernel_size' -e 'ramdisk size'; }
CMD=$(field 'command line args')
for W in pk4 pk5 pk6; do
  IMG=$OUT/boot-$W.img
  mkbootimg --header_version 0 --base 0x0 --kernel_offset "$(field 'kernel load address')" --ramdisk_offset "$(field 'ramdisk load address')" \
    --second_offset 0x0 --tags_offset "$(field 'kernel tags load address')" --pagesize "$(field 'page size')" \
    --os_version "$(field 'os version')" --os_patch_level "$(field 'os patch level')" --cmdline "$CMD" \
    --kernel "$T/blob-$W" --ramdisk "$T/rd-$W" -o "$IMG"
  avbtool add_hash_footer --image "$IMG" --partition_size "$PSIZE" --partition_name boot --algorithm NONE \
    --salt "$(echo "$AV" | sed -n 's/^ *Salt: *//p' | head -n1)" "$@"
  U=$(unpack_bootimg --boot_img "$IMG" --out "$T/u$W")
  split "$T/u$W/kernel" "$T/u$W/k"
  ck "$W header (offsets, page, os word, command line) equals pk3's but the sizes" "$(hdr "$U")" "$(hdr "$INFO")"
  ck "$W command line has no earlycon and no console=" "$(echo "$U" | sed -n 's/^command line args: *//p' | head -n1 | tr ' ' '\n' | grep -c -e '^earlycon' -e '^console=')" 0
  ck "$W size is pk3's ($PSIZE)" "$(stat -c %s "$IMG")" "$PSIZE"
  mkdir "$T/v$W" && cp "$IMG" "$T/v$W/boot.img"
  ck "$W AVB footer verifies" "$(avbtool verify_image --image "$T/v$W/boot.img" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb reads" "$(fdtdump "$T/u$W/k.dtb" >/dev/null 2>&1 && echo yes || echo no)" yes
  ck "$W dtb has /__symbols__ (overlay-mergeable)" "$(fdtdump "$T/u$W/k.dtb" 2>/dev/null | grep -c '__symbols__' | sed 's/^[1-9].*/some/')" some
done
# pk4: kernel is pk3's, dtb is ours, ramdisk ml5a's
ck "pk4 Image.gz is pk3's (porthole's)" "$(h "$T/upk4/k.gz")" "$(h "$T/pk.gz")"
ck "pk4 dtb is ml6a's (ours)" "$(h "$T/upk4/k.dtb")" "$(h "$T/our.dtb")"
ck "pk4 dtb differs from pk3's" "$([ "$(h "$T/upk4/k.dtb")" != "$(h "$T/pk.dtb")" ] && echo yes || echo no)" yes
ck "pk4 ramdisk is ml6a's/pk3's (init-ml5a, reboot at once)" "$(h "$T/upk4/ramdisk")" "$(h "$T/pk3/ramdisk")"
# pk5: kernel is ours, dtb is porthole's, ramdisk ml5a's
ck "pk5 Image.gz is ml6a's (ours)" "$(h "$T/upk5/k.gz")" "$(h "$T/our.gz")"
ck "pk5 dtb is pk3's (porthole's taimen)" "$(h "$T/upk5/k.dtb")" "$(h "$T/pk.dtb")"
ck "pk5 Image.gz differs from pk3's" "$([ "$(h "$T/upk5/k.gz")" != "$(h "$T/pk.gz")" ] && echo yes || echo no)" yes
ck "pk5 ramdisk is ml6a's/pk3's (init-ml5a)" "$(h "$T/upk5/ramdisk")" "$(h "$T/pk3/ramdisk")"
# pk6: pk3 with only the ramdisk swapped
ck "pk6 kernel blob equals pk3's" "$(h "$T/upk6/kernel")" "$(h "$T/pk3/kernel")"
ck "pk6 ramdisk is ml5b's (timing init)" "$(h "$T/upk6/ramdisk")" "$(h "$T/ml5b/ramdisk")"
ck "pk6 ramdisk differs from pk3's" "$([ "$(h "$T/upk6/ramdisk")" != "$(h "$T/pk3/ramdisk")" ] && echo yes || echo no)" yes
ck "pk6 ramdisk /init is the timing init (sleep per point)" "$(xz -dc "$T/upk6/ramdisk" | cpio -i --to-stdout init 2>/dev/null | grep -c 'sleep')" "$(xz -dc "$T/ml5b/ramdisk" | cpio -i --to-stdout init 2>/dev/null | grep -c 'sleep')"
ck "pk4 and pk5 and pk6 are three different images" "$(sha256sum "$OUT"/boot-pk4.img "$OUT"/boot-pk5.img "$OUT"/boot-pk6.img | cut -c1-64 | sort -u | wc -l)" 3
(cd "$OUT" && sha256sum boot-pk4.img boot-pk5.img boot-pk6.img | tee SHA256SUMS)
[ $total -eq 0 ]
