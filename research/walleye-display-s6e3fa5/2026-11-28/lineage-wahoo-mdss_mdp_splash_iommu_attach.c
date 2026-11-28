static int mdss_mdp_splash_iommu_attach(struct msm_fb_data_type *mfd)
{
	struct mdss_overlay_private *mdp5_data = mfd_to_mdp5_data(mfd);
	struct mdss_data_type *mdata = mdss_mdp_get_mdata();
	int rc, ret;

	/*
	 * iommu dynamic attach for following conditions.
	 * 1. it is still not attached
	 * 2. MDP hardware version supports the feature
	 * 3. configuration is with valid splash buffer
	 */
	if (mdata->mdss_util->iommu_attached() ||
		!mfd->panel_info->cont_splash_enabled ||
		!mdss_mdp_iommu_dyn_attach_supported(mdp5_data->mdata) ||
		!mdp5_data->splash_mem_addr ||
		!mdp5_data->splash_mem_size) {
		pr_debug("dynamic attach is not supported\n");
		return -EPERM;
	}

	rc = mdss_smmu_map(MDSS_IOMMU_DOMAIN_UNSECURE,
				mdp5_data->splash_mem_addr,
				mdp5_data->splash_mem_addr,
				mdp5_data->splash_mem_size,
				IOMMU_READ | IOMMU_NOEXEC);
	if (rc) {
		pr_debug("iommu memory mapping failed rc=%d\n", rc);
	} else {
		ret = mdss_iommu_ctrl(1);
		if (IS_ERR_VALUE(ret)) {
			pr_err("mdss iommu attach failed\n");
			mdss_smmu_unmap(MDSS_IOMMU_DOMAIN_UNSECURE,
					mdp5_data->splash_mem_addr,
					mdp5_data->splash_mem_size);
		} else {
			mfd->splash_info.iommu_dynamic_attached = true;
		}
	}

	return rc;
}

static void mdss_mdp_splash_unmap_splash_mem(struct msm_fb_data_type *mfd)
{
	struct mdss_overlay_private *mdp5_data = mfd_to_mdp5_data(mfd);

	if (mfd->splash_info.iommu_dynamic_attached) {

		mdss_smmu_unmap(MDSS_IOMMU_DOMAIN_UNSECURE,
				mdp5_data->splash_mem_addr,
				mdp5_data->splash_mem_size);
