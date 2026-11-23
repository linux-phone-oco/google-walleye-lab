# The lab levers every display image's dtb carries beside the kernel's own tree, sourced by the builders of those images (build/build-ufs-root-display-image.sh): each is one fdtput, from the script that first needed it.
#   display_levers DTB     edits DTB in place (a dtb built with DTC_FLAGS=-@ from the display board file)
# - clock_gcc at the real gcc (build-display-image.sh); - the dsi host's vdd-supply = l1a (build-display-late-l3-dtb.sh); - the label mdss_dsi0 on an empty stub node, so the vendor overlay's qcom,platform-reset-gpio never reaches the real dsi
# (build-display-late-l5-dtb.sh). build-display-any-order-images.sh carries the same lines inline: its images are recorded byte for byte, so it is not changed here.
display_levers() {
  _gcc=/soc@0/clock-controller@100000
  _dsi=/soc@0/display-subsystem@c900000/dsi@c994000
  _stubs=/vendor-overlay-stubs
  fdtput -t s "$1" /__symbols__ clock_gcc "$_gcc"
  fdtput -t x "$1" "$_dsi" vdd-supply "$(fdtget -t x "$1" "$(fdtget "$1" /__symbols__ vreg_l1a_0p875)" phandle)"
  _max=$(dtc -I dtb -O dts -q "$1" 2>/dev/null | sed -n 's/^[[:space:]]*phandle = <0x\([0-9a-f]*\)>;$/\1/p' | while read -r h; do echo $((0x$h)); done | sort -n | tail -n1)
  fdtput -c "$1" "$_stubs/mdss_dsi0"
  fdtput -t u "$1" "$_stubs/mdss_dsi0" phandle $((_max+1))
  fdtput -t s "$1" /__symbols__ mdss_dsi0 "$_stubs/mdss_dsi0"
}
