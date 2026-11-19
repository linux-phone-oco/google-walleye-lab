// SPDX-License-Identifier: GPL-2.0-only
/*
 * Driver for the Samsung S6E3FA5 command-mode AMOLED panel (Google Pixel 2, OnePlus 5).
 *
 * Structure adapted to the v7.3 panel API from Jami Kettunen's out-of-tree driver for the OnePlus 5 (GPL-2.0-only).
 * The init sequence, reset sequence, off delays and timings are the Pixel 2's own, read as facts (register values, delays, porches) from the
 * phone's own overlay partition (dtbo entry 3, the S6E3FA5 node) and written here in our own words; the panel is not lit on the phone yet.
 */

#include <linux/backlight.h>
#include <linux/delay.h>
#include <linux/gpio/consumer.h>
#include <linux/module.h>
#include <linux/of.h>
#include <linux/regulator/consumer.h>

#include <video/mipi_display.h>

#include <drm/drm_mipi_dsi.h>
#include <drm/drm_modes.h>
#include <drm/drm_panel.h>

struct s6e3fa5 {
	struct drm_panel panel;
	struct mipi_dsi_device *dsi;
	struct regulator_bulk_data supplies[2];
	struct gpio_desc *reset_gpio;
};

static inline struct s6e3fa5 *to_s6e3fa5(struct drm_panel *panel)
{
	return container_of(panel, struct s6e3fa5, panel);
}

/* Reset: the stock tree's sequence is high 1 ms, low 1 ms, high 10 ms on the physical pin; the line is active low, so asserted = low */
static void s6e3fa5_reset(struct s6e3fa5 *ctx)
{
	gpiod_set_value_cansleep(ctx->reset_gpio, 0);
	usleep_range(1000, 2000);
	gpiod_set_value_cansleep(ctx->reset_gpio, 1);
	usleep_range(1000, 2000);
	gpiod_set_value_cansleep(ctx->reset_gpio, 0);
	usleep_range(10000, 11000);
}

/* The stock tree's on-commands, in order: sleep out, 120 ms, tearing effect on, 0x53 = 0x20, one vendor register (0xbb) between the 0xf0 unlock and lock, display on */
static int s6e3fa5_on(struct s6e3fa5 *ctx)
{
	struct mipi_dsi_multi_context dsi_ctx = { .dsi = ctx->dsi };

	ctx->dsi->mode_flags |= MIPI_DSI_MODE_LPM;

	mipi_dsi_dcs_exit_sleep_mode_multi(&dsi_ctx);
	mipi_dsi_msleep(&dsi_ctx, 120);
	mipi_dsi_dcs_set_tear_on_multi(&dsi_ctx, MIPI_DSI_DCS_TEAR_MODE_VBLANK);
	mipi_dsi_dcs_write_seq_multi(&dsi_ctx, MIPI_DCS_WRITE_CONTROL_DISPLAY, 0x20);
	mipi_dsi_dcs_write_seq_multi(&dsi_ctx, 0xf0, 0x5a, 0x5a);
	mipi_dsi_dcs_write_seq_multi(&dsi_ctx, 0xbb, 0x54, 0x00, 0x36, 0x20, 0xf3);
	mipi_dsi_dcs_write_seq_multi(&dsi_ctx, 0xf0, 0xa5, 0xa5);
	mipi_dsi_dcs_set_display_on_multi(&dsi_ctx);

	return dsi_ctx.accum_err;
}

static int s6e3fa5_prepare(struct drm_panel *panel)
{
	struct s6e3fa5 *ctx = to_s6e3fa5(panel);
	struct device *dev = &ctx->dsi->dev;
	int ret;

	ret = regulator_bulk_enable(ARRAY_SIZE(ctx->supplies), ctx->supplies);
	if (ret < 0) {
		dev_err(dev, "Failed to enable regulators: %d\n", ret);
		return ret;
	}

	s6e3fa5_reset(ctx);

	ret = s6e3fa5_on(ctx);
	if (ret < 0) {
		dev_err(dev, "Failed to initialize panel: %d\n", ret);
		gpiod_set_value_cansleep(ctx->reset_gpio, 1);
		regulator_bulk_disable(ARRAY_SIZE(ctx->supplies), ctx->supplies);
		return ret;
	}

	return 0;
}

static int s6e3fa5_unprepare(struct drm_panel *panel)
{
	struct s6e3fa5 *ctx = to_s6e3fa5(panel);
	struct mipi_dsi_multi_context dsi_ctx = { .dsi = ctx->dsi };

	ctx->dsi->mode_flags &= ~MIPI_DSI_MODE_LPM;

	mipi_dsi_dcs_set_display_off_multi(&dsi_ctx);
	mipi_dsi_msleep(&dsi_ctx, 10);
	mipi_dsi_dcs_enter_sleep_mode_multi(&dsi_ctx);
	mipi_dsi_msleep(&dsi_ctx, 150);

	gpiod_set_value_cansleep(ctx->reset_gpio, 1);
	regulator_bulk_disable(ARRAY_SIZE(ctx->supplies), ctx->supplies);

	return 0;
}

/* Command mode: the host chooses the timing; the stock tree gives 60 Hz, porches 18/82/10 and 8/16/2, 62x110 mm */
static const struct drm_display_mode s6e3fa5_mode = {
	.clock = (1080 + 18 + 10 + 82) * (1920 + 8 + 2 + 16) * 60 / 1000,
	.hdisplay = 1080,
	.hsync_start = 1080 + 18,
	.hsync_end = 1080 + 18 + 10,
	.htotal = 1080 + 18 + 10 + 82,
	.vdisplay = 1920,
	.vsync_start = 1920 + 8,
	.vsync_end = 1920 + 8 + 2,
	.vtotal = 1920 + 8 + 2 + 16,
	.width_mm = 62,
	.height_mm = 110,
};

static int s6e3fa5_get_modes(struct drm_panel *panel,
			     struct drm_connector *connector)
{
	struct drm_display_mode *mode;

	mode = drm_mode_duplicate(connector->dev, &s6e3fa5_mode);
	if (!mode)
		return -ENOMEM;

	drm_mode_set_name(mode);
	mode->type = DRM_MODE_TYPE_DRIVER | DRM_MODE_TYPE_PREFERRED;
	connector->display_info.width_mm = mode->width_mm;
	connector->display_info.height_mm = mode->height_mm;
	drm_mode_probed_add(connector, mode);

	return 1;
}

static const struct drm_panel_funcs s6e3fa5_panel_funcs = {
	.prepare = s6e3fa5_prepare,
	.unprepare = s6e3fa5_unprepare,
	.get_modes = s6e3fa5_get_modes,
};

/* The stock tree sets the level with an 8-bit DCS write (0x51, 1..255): one byte, not the two of the 16-bit helper */
static int s6e3fa5_bl_update_status(struct backlight_device *bl)
{
	struct mipi_dsi_device *dsi = bl_get_data(bl);
	u8 brightness = backlight_get_brightness(bl);
	int ret;

	ret = mipi_dsi_dcs_write(dsi, MIPI_DCS_SET_DISPLAY_BRIGHTNESS, &brightness, sizeof(brightness));

	return ret < 0 ? ret : 0;
}

static const struct backlight_ops s6e3fa5_bl_ops = {
	.update_status = s6e3fa5_bl_update_status,
};

static int s6e3fa5_probe(struct mipi_dsi_device *dsi)
{
	struct device *dev = &dsi->dev;
	const struct backlight_properties props = {
		.type = BACKLIGHT_RAW,
		.brightness = 255,
		.max_brightness = 255,
	};
	struct s6e3fa5 *ctx;
	int ret;

	ctx = devm_drm_panel_alloc(dev, struct s6e3fa5, panel,
				   &s6e3fa5_panel_funcs, DRM_MODE_CONNECTOR_DSI);
	if (IS_ERR(ctx))
		return PTR_ERR(ctx);

	ctx->supplies[0].supply = "vddio";
	ctx->supplies[1].supply = "vci";
	ret = devm_regulator_bulk_get(dev, ARRAY_SIZE(ctx->supplies), ctx->supplies);
	if (ret < 0)
		return dev_err_probe(dev, ret, "Failed to get regulators\n");

	ctx->reset_gpio = devm_gpiod_get(dev, "reset", GPIOD_OUT_HIGH);
	if (IS_ERR(ctx->reset_gpio))
		return dev_err_probe(dev, PTR_ERR(ctx->reset_gpio),
				     "Failed to get reset-gpios\n");

	ctx->dsi = dsi;
	mipi_dsi_set_drvdata(dsi, ctx);

	dsi->lanes = 4;
	dsi->format = MIPI_DSI_FMT_RGB888;
	/* command mode: no MIPI_DSI_MODE_VIDEO flag, the host reads the tearing-effect line */
	dsi->mode_flags = MIPI_DSI_CLOCK_NON_CONTINUOUS;

	ctx->panel.prepare_prev_first = true;

	ctx->panel.backlight = devm_backlight_device_register(dev, dev_name(dev), dev, dsi,
							       &s6e3fa5_bl_ops, &props);
	if (IS_ERR(ctx->panel.backlight))
		return dev_err_probe(dev, PTR_ERR(ctx->panel.backlight),
				     "Failed to create backlight\n");

	drm_panel_add(&ctx->panel);

	ret = mipi_dsi_attach(dsi);
	if (ret < 0) {
		dev_err(dev, "Failed to attach to DSI host: %d\n", ret);
		drm_panel_remove(&ctx->panel);
		return ret;
	}

	return 0;
}

static void s6e3fa5_remove(struct mipi_dsi_device *dsi)
{
	struct s6e3fa5 *ctx = mipi_dsi_get_drvdata(dsi);
	int ret;

	ret = mipi_dsi_detach(dsi);
	if (ret < 0)
		dev_err(&dsi->dev, "Failed to detach from DSI host: %d\n", ret);

	drm_panel_remove(&ctx->panel);
}

static const struct of_device_id s6e3fa5_of_match[] = {
	{ .compatible = "samsung,s6e3fa5" },
	{ /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, s6e3fa5_of_match);

static struct mipi_dsi_driver s6e3fa5_driver = {
	.probe = s6e3fa5_probe,
	.remove = s6e3fa5_remove,
	.driver = {
		.name = "panel-samsung-s6e3fa5",
		.of_match_table = s6e3fa5_of_match,
	},
};
module_mipi_dsi_driver(s6e3fa5_driver);

MODULE_AUTHOR("Jami Kettunen <jami.kettunen@protonmail.com>");
MODULE_DESCRIPTION("DRM driver for the Samsung S6E3FA5 command mode AMOLED DSI panel");
MODULE_LICENSE("GPL");
