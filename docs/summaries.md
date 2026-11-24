<!-- Generated from data/ by tad. Do not edit by hand. -->

# Summaries

### Row 1

About
: walleye-ufs-match-android-z

Date
: 2026-11-24

Says
: The matched UFS-on image (sha256 4610778...df5) reads the Pixel 2's card on mainline: version 3.1.0, six LUNs, partition tables, no UFS errors, link up at HS gear 1 at idle (walleye-ufs-try1-card-readable, walleye-ufs-why-gear1-ranked); its UFS path was matched to Android's (walleye-ufs-solution-table). Try 2's 'shell silent under sustained read' is not shown to be UFS: in all four runs the load began 175-176 s after boot and the phone left at 180 s, the image's own z14 init rebooting after 90 watchdog beats, with no hold (walleye-ufs-180s-is-the-init-self-reboot). The procedure's idle link read waits 170 s for an HS-G3 that idle never shows. Decided: the procedure fits the image - idle link read once, no hold claim, each ramp step only before the reboot deadline - and the image is not rebuilt (2026-11-24-walleye-ufs-procedure-fits-the-image-not-the-image-the-procedure). Next: build item walleye-ufs-procedure-fits-the-180s (its failing test is in ufs.rs, ignored until then), then the phone run walleye-ufs-silent-ramped-pinned-try under a go on the same sha256. Stage 3 (rw) stays blocked; whether the card is silent under load is NOT KNOWN.

Condenses
: 2026-11-19-walleye-builder-the-ufs-try-as-one-rust, 2026-11-19-walleye-builder-ufs-link-works-slow, 2026-11-19-walleye-builder-ufs-match-prints, 2026-11-19-walleye-research-ufs-solution-extracted, 2026-11-19-walleye-ufs-android-derived-solution, 2026-11-19-walleye-ufs-linkstartup-do-not-carry, 2026-11-19-walleye-ufs-no-sync-length-quirk, 2026-11-21-0200-z-how-the-phone-stands, 2026-11-21-ufs-try-2-built, 2026-11-21-ufs-try-2-reads-the-gear-under-load-by-a-bounded, 2026-11-23-1800-walleye-keeper-how-the-phone-stands, 2026-11-23-walleye-ufs-silent-one-ramped-pinned-try, 2026-11-24-walleye-ufs-procedure-fits-the-image-not-the-image-the-procedure, walleye-ufs-180s-is-the-init-self-reboot, walleye-ufs-card-revision, walleye-ufs-gap-abl-handover-read, walleye-ufs-gap-lane1, walleye-ufs-gap-link-startup-read, walleye-ufs-gap-perf, walleye-ufs-gap-sony-mechanism-public-record, walleye-ufs-hynix-sync-length-verdict, walleye-ufs-ice-verdict, walleye-ufs-silent-under-load-causes-ranked, walleye-ufs-solution-table, walleye-ufs-try-2-built-not-booted, walleye-ufs-try-leaves-the-bus-at-180s-every-run, walleye-ufs-try1-card-readable, walleye-ufs-version-from-source, walleye-ufs-why-gear1-ranked
