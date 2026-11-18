#!/bin/sh
# Proves tests/host/overlay-collisions.sh on a small fake base dtb and a fake dtbo, both made here:
#  - an overlay fragment that targets a label of ours and rewrites a property we have      -> fail, reported with our path
#  - an overlay node of the name of one of ours that rewrites a property of it            -> fail
#  - the same overlay, the label pointed at a stub node (as the real fix does)            -> pass, the write goes to the stub
#  - an overlay that only adds properties and nodes                                       -> pass, listed as additions
#  - an allow line for the one rewritten property                                         -> pass
#  - a node of another name beside ours (the qup@c179000 way)                             -> pass
#  - no dtb or no dtbo                                                                    -> skip
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
total=0
want() { # NAME DTB DTBO ALLOW EXPECTED [GREP]
  got=$(DTB=$2 DTBO=$3 ALLOW=${4:-$T/none.allow} sh "$HERE/overlay-collisions.sh" 2>&1 || true)
  v=$(printf '%s\n' "$got" | tail -n1 | cut -d' ' -f1)
  if [ "$v" = "$5" ] && { [ -z "${6:-}" ] || printf '%s\n' "$got" | grep -q -- "$6"; }; then echo "ok   $1"; else echo "FAIL $1: got '$v', want '$5' ${6:+with '$6'}"; printf '%s\n' "$got" | tail -n 4; total=$((total+1)); fi
}
: > "$T/none.allow"
# a dtbo with one entry at index 3, as the real one is: 32-byte header, 32-byte entries (size, offset, id, rev, ...), big-endian
be32() { o=$(printf '\\%03o\\%03o\\%03o\\%03o' $(($1>>24&255)) $(($1>>16&255)) $(($1>>8&255)) $(($1&255))); printf "$o"; }
mkdtbo() { # OVERLAY_DTB OUT: header (32 bytes), four entries (size, offset, id, rev, 4 custom words), the overlay is entry 3
  s=$(stat -c %s "$1"); base=$((32+4*32))
  { be32 $((0xd7b7ab1e)); be32 $((base+s)); be32 32; be32 32; be32 4; be32 32; be32 4096; be32 0
    i=0; while [ $i -lt 3 ]; do be32 0; be32 0; be32 0; be32 0; be32 0; be32 0; be32 0; be32 0; i=$((i+1)); done
    be32 "$s"; be32 "$base"; be32 1; be32 1; be32 0; be32 0; be32 0; be32 0
    cat "$1"; } > "$2"
}
cat > "$T/base.dts" <<'D'
/dts-v1/;
/ {
	soc {
		dev0: i2c@1000 { compatible = "ours,i2c"; reg = <0x1000 0x10>; status = "okay"; };
		dev1: gpu@2000 { compatible = "ours,gpu"; status = "disabled"; };
		other@3000 { compatible = "ours,other"; };
	};
	stubs { stub_dev0: dev0stub {}; };
};
D
dtc -@ -I dts -O dtb -o "$T/base.dtb" "$T/base.dts" 2>/dev/null
# 1: a fragment on our label rewrites compatible
cat > "$T/ov1.dts" <<'D'
/dts-v1/;
/plugin/;
/ { fragment@0 { target = <&dev0>; __overlay__ { compatible = "vendor,i2c"; }; }; };
D
# 2: a node by our name under our parent rewrites compatible
cat > "$T/ov2.dts" <<'D'
/dts-v1/;
/plugin/;
/ { fragment@0 { target = <&soc_label>; __overlay__ { i2c@1000 { compatible = "vendor,i2c"; }; }; }; };
D
# 3: only additions: a property on our node, a new node
cat > "$T/ov3.dts" <<'D'
/dts-v1/;
/plugin/;
/ { fragment@0 { target = <&dev1>; __overlay__ { vendor,extra = <1>; newnode { reg = <0>; }; }; }; };
D
# 4: a node of another name beside ours
cat > "$T/ov4.dts" <<'D'
/dts-v1/;
/plugin/;
/ { fragment@0 { target = <&soc_label>; __overlay__ { vendor-i2c@1000 { compatible = "vendor,i2c"; }; }; }; };
D
dtc -@ -I dts -O dtb -o "$T/ov1.dtb" "$T/ov1.dts" 2>/dev/null
# ov2 needs a label for the parent node: give the base a second dts with it
sed 's/^\tsoc {/\tsoc_label: soc {/' "$T/base.dts" > "$T/base2.dts"
dtc -@ -I dts -O dtb -o "$T/base2.dtb" "$T/base2.dts" 2>/dev/null
dtc -@ -I dts -O dtb -o "$T/ov2.dtb" "$T/ov2.dts" 2>/dev/null
dtc -@ -I dts -O dtb -o "$T/ov3.dtb" "$T/ov3.dts" 2>/dev/null
dtc -@ -I dts -O dtb -o "$T/ov4.dtb" "$T/ov4.dts" 2>/dev/null
for n in 1 2 3 4; do mkdtbo "$T/ov$n.dtb" "$T/ov$n.img"; done
# the fake image really holds the overlay as entry 3
ck=$(xxd -s $((32+32*3)) -l 8 -p "$T/ov1.img"); [ "$((0x$(echo "$ck" | cut -c1-8)))" -eq "$(stat -c %s "$T/ov1.dtb")" ] || { echo "FAIL selftest: the fake dtbo table is wrong" >&2; exit 1; }
want "a fragment on our label rewriting compatible fails" "$T/base.dtb" "$T/ov1.img" "" fail "collision /soc/i2c@1000  compatible"
want "the failure counts one collision on one node" "$T/base.dtb" "$T/ov1.img" "" fail "count 1 collisions on 1 nodes"
want "a node of our name rewriting compatible fails" "$T/base2.dtb" "$T/ov2.img" "" fail "collision /soc/i2c@1000  compatible"
# the real fix: the label points at a stub, our node is left alone
cp "$T/base.dtb" "$T/fixed.dtb"; fdtput -t s "$T/fixed.dtb" /__symbols__ dev0 /stubs/dev0stub
want "the label pointed at a stub passes" "$T/fixed.dtb" "$T/ov1.img" "" pass "count 0 collisions"
want "an overlay of additions only passes" "$T/base.dtb" "$T/ov3.img" "" pass "add       /soc/gpu@2000: vendor,extra"
printf '# the intended one\n/soc/i2c@1000 compatible the vendor driver is not wanted anyway\n' > "$T/ok.allow"
want "an allow line for the rewritten property passes" "$T/base.dtb" "$T/ov1.img" "$T/ok.allow" pass "allowed   /soc/i2c@1000 compatible"
printf '/soc/i2c@1000 *\n' > "$T/ok2.allow"
want "an allow line for the whole node passes" "$T/base.dtb" "$T/ov1.img" "$T/ok2.allow" pass
want "a node of another name beside ours passes" "$T/base2.dtb" "$T/ov4.img" "" pass "count 0 collisions"
want "no dtb skips" "$T/missing.dtb" "$T/ov1.img" "" skip
want "no dtbo skips" "$T/base.dtb" "$T/missing.img" "" skip
[ $total -eq 0 ] || { echo "overlay-collisions-selftest: $total failed" >&2; exit 1; }
echo "pass overlay-collisions-selftest"
