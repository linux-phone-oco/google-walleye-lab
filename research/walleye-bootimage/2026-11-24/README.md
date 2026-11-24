# The go row of run 5 (the short sha that stopped check 6)

go-row-run5-short-sha.csv is the row of the item walleye-try-ufs-match as `tad q row --id walleye-try-ufs-match --format csv`
printed it when UFS try 2 run 5 stopped at check 6 (2026-11-23 22:47): read from this base's data/backlog.json at the
commit before PR 967 (f6cb47a^). Its note names the image only as 461077869fb2 (12 hex); [images.ufs-match] sha256 is
461077869fb275ab8803b8bb771050d4f19724ccfd62e197dc1f3fe3a6297df5. Source of the stop: docs session_log, "UFS try 2 run 5".
Used by walleye-tries (ufs.rs) tests of item walleye-go-reads-sha-from-one-place.
