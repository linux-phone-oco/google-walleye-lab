# Why the ncm gadget step panics the walleye vendor kernel (read from public GPL source)

Read from the public android-msm-wahoo-4.4-android11 tree (the vendor's 4.4 line; the phone runs 4.4.302 - the branch is the nearest public one, not the exact build).

- f_ncm.c `ncm_alloc_inst()`: only `kzalloc` + `mutex_init` + `config_group_init_type_name` - `opts->net` stays NULL.
- `ncm_bind()` (runs at UDC bind): `ncm_opts->net = gether_setup_default()` - the net_device is made here.
- `ncm_alloc()` (runs at the config symlink): `opts->refcnt++`.
- u_ether_configfs.h attribute store (dev_addr, host_addr, qmult, ifname): `if (opts->refcnt) return -EBUSY; gether_set_dev_addr(opts->net, page)`; `gether_set_dev_addr` does `netdev_priv(net)` and `memcpy(dev->dev_mac...)` with no NULL check. The show side calls `gether_get_dev_addr(opts->net...)` also unchecked.
- Our images (vendor, 3, 5b, 5c) all write `functions/ncm.usb0/dev_addr` and `host_addr` right after the mkdir, before any symlink or bind: opts->net is NULL and refcnt is 0, so the store dereferences a NULL net_device. 5a has no function; gadget4 has no gadget: they return.
- There is no window in which a write is safe: before the symlink net is NULL (oops); after it refcnt != 0 (-EBUSY). The attributes cannot be set on this kernel, nor read before bind.
- Android's init.hardware.usb.rc never creates ncm (gsi.rndis, gsi.rmnet, ffs.*, mass_storage, accessory, ...): this kernel's ncm path has never been exercised by the vendor.
