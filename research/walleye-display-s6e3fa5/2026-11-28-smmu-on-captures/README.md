# Captures for the SMMU-on display procedure (item walleye-display-smmu-on-procedure)

Whole texts a phone printed, kept so the procedure's checks name their source (tests/fixtures/phone-text/SOURCES.toml cuts from these):

- `b-r5-passthrough-clean-dmesg-d5-2026-11-20.txt`: kernel log of B-r5 (iommu.passthrough=1) at its 5 s load, ~/lab-material/walleye/display-delay-b-r-20261120-230943/dmesg-d5.txt: the clean case (SMMU faults 0), the panel bound, drm initialized, fb0.
- `b-d5-signs-dmesg-d5-2026-11-26.txt`: kernel log of B-d5 at 5 s, ~/lab-material/walleye/display-delay-b-d-20261126-085956/dmesg-d5.txt: the case with the signs (arm-smmu context faults).
- `b-a5-alpine-login-read-2026-11-23.txt`: B-a5's read over ssh, ~/lab-material/walleye/alpine-login-20261123-061501/read.txt; its `ps` lists the `card0-crtc0` thread.

Not captured yet: a debugfs DRM state reading (a CRTC's `active=`), so "a CRTC is active" is recorded, not judged, until the first run's text is kept.
- `b-r5-smmu-id-run-20261128-175948-dmesg.txt`: kernel log of B-r5-smmu-id (SMMU on, no dyndbg words), ~/lab-material/walleye/display-smmu-on-20261128-175948/dmesg.txt: the DSI host bound, the panel module inserted, no line of the panel driver's own (so its panel sign is not known yet).
