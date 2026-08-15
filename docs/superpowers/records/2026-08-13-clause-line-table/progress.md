# SDD ledger -- plan: docs/superpowers/plans/2026-08-13-clause-line-table.md

Base: 9b2d416d3533c81dea80fbb1f907071f8b693891 (`9b2d416d3`, the plan's own commit).
Task 1: complete, accepted.
Code: `5b3be9536`. Record entry 35 and the plan's own correction pointer: `a8ea4f5be`.
Report: `.superpowers/sdd/2026-08-13-clause-line-table/task-1-report.md`.
Task 1: implemented -- `5b3be9536` (code), `a8ea4f5be` (record entry 35, numstat `136 0`).
Task 1: gates re-run by the controller at `a8ea4f5be`, unpiped: fmt 0, clippy 0 with no warning lines, `cargo test --workspace --no-fail-fast` rc 0, 1487 passed / 0 failed (1483 before, so four new tests).
Task 1: seven axes down, none up, -1.026% to -7.366%; `rexxcps` -3.811%. Tightest arm pair non-overlapping by 83.7x its wider span.
Task 1: the plan's own design finding was refuted by the task -- saving per removed search is monotone in body line count, 40.0 at 7 lines to 94.7 at 198. The controller checked the division on every axis and it reproduces. The plan's finding is left standing with a pointer, per the append rule's spirit.
Task 1: task review dispatched, `BodyKey` wrong-line question first priority.
Task 1: review approved -- no critical, no important. Four minor prose findings, all fixed by the controller at the commit above; gates re-run unpiped, 1487 passed / 0 failed.
Task 1: NOT taken -- the review's log2 restatement of the depth finding. floor(log2 n) + 1 puts an 8-line body at four steps alongside the 12- and 14-line ones, where it saves 42.0 against their 52.4 to 53.5, so the fit does not reproduce. Worth revisiting with a better variable; entry 35's monotone claim stands as measured.
Task 1: deferred minors for the final review -- the pointer assert at run.rs:8115 duplicates ir/drive.rs's `debug_assert_names_the_clause`; `Interp::clause_line` survives beside `clause_line_at` with one caller, and a future per-clause caller reaching for the wrong one silently gets the search back.
Task 1: complete.
