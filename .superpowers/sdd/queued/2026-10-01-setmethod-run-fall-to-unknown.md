# 2026-10-01-setmethod-run-fall-to-unknown

Found by Phase 6 S0/S1 final review slice A (2026-10-01), not fixed: Object~setMethod and Object~run are not implemented and fall through to the receiver's UNKNOWN instead of a loud refusal (silent wrong behaviour). Pre-existing. final-review-a.md B5, shapes-pin/s20_run_method.rex ends in 97.1.
