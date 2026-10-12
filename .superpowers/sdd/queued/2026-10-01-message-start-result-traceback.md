# 2026-10-01-message-start-result-traceback

Found by Phase 6 S0/S1 final review slice A (2026-10-01), not fixed: Message~start(...)~result after the method raised: oracle traceback has one frame and prints the error report on stderr from the message's thread; this crate a three-frame traceback and nothing on stderr. S2 (start as a real activity). final-review-a.md B4.
