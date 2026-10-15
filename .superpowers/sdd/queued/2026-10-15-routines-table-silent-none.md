# .ROUTINES table builders answer Ok(None) silently

From the 9b1 final re-review (.superpowers/sdd/2026-10-14-phase-9b1/final-rereview.md, Minor 1).
`build_package_string_table` returns Result since 8fa4a5694, but a failed table allocation, missing
package data and a failed `directory_put` still answer `Ok(None)` (rexx-exec environment.rs ~1163,
~1166, ~1233), and `find_written_routine` does the same for an absent table (identities.rs ~1076).
Each should propagate or refuse loudly.

Also: rexx-bench/src/lib.rs bench-list comments from Phase 6.1 and 9a still begin "Added in ..."
(task history, comment rule).
