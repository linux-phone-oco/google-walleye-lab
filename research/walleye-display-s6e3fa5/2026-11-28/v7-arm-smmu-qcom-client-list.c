static const struct of_device_id qcom_smmu_client_of_match[] __maybe_unused = {
	{ .compatible = "qcom,adreno" },
	{ .compatible = "qcom,adreno-gmu" },
	{ .compatible = "qcom,glymur-mdss" },
	{ .compatible = "qcom,mdp4" },
	{ .compatible = "qcom,mdss" },
	{ .compatible = "qcom,qcm2290-mdss" },
	{ .compatible = "qcom,sar2130p-mdss" },
	{ .compatible = "qcom,sc7180-mdss" },
	{ .compatible = "qcom,sc7180-mss-pil" },
	{ .compatible = "qcom,sc7280-mdss" },
	{ .compatible = "qcom,sc7280-mss-pil" },
	{ .compatible = "qcom,sc8180x-mdss" },
	{ .compatible = "qcom,sc8280xp-mdss" },
	{ .compatible = "qcom,sdm670-mdss" },
	{ .compatible = "qcom,sdm845-mdss" },
	{ .compatible = "qcom,sdm845-mss-pil" },
	{ .compatible = "qcom,sm6115-mdss" },
	{ .compatible = "qcom,sm6350-mdss" },
	{ .compatible = "qcom,sm6375-mdss" },
	{ .compatible = "qcom,sm8150-mdss" },
	{ .compatible = "qcom,sm8250-mdss" },
	{ .compatible = "qcom,sm8450-mdss" },
	{ .compatible = "qcom,x1e80100-mdss" },
	{ }
};

static int qcom_smmu_init_context(struct arm_smmu_domain *smmu_domain,
		struct io_pgtable_cfg *pgtbl_cfg, struct device *dev)
{
	struct arm_smmu_device *smmu = smmu_domain->smmu;
	struct qcom_smmu *qsmmu = to_qcom_smmu(smmu);
	const struct of_device_id *client_match;
	int cbndx = smmu_domain->cfg.cbndx;

	smmu_domain->cfg.flush_walk_prefer_tlbiasid = true;

	client_match = qsmmu->data->client_match;

	if (client_match)
		qcom_smmu_set_actlr_dev(dev, smmu, cbndx, client_match);

	return 0;
}
