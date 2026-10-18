**CLOSED: 4b694fe7c.** its own "Closed 2026-09-29" section; no "SPIKE, not for commit" remains under rust/crates.

# FlatLoop still says "SPIKE, not for commit."

`rust/crates/rexx-exec/src/run/loops.rs:93` (found by the file-split final
review, 2026-09-25) carries "SPIKE, not for commit." on `FlatLoop`, which is
committed and in use. Predates the file-split plan. Decide whether the comment
or the code is what is stale, and fix that.

## Closed 2026-09-29 by 4b694fe7c

The comment was stale, not the code: Phase 6 S0 Task 1 adopted the flat-loop
path, removed the `REXX_NO_FLAT` toggle and dropped every flat-loop "SPIKE"
label.
