# Phase 6.1 final review fix dispatch (controller)

Read `final-review.md` in this directory. Its probes are under `/tmp/claude-1000/p61/final/probes/`, and the wrapper is `/tmp/claude-1000/p61/final/probe.sh`. The rulings for each finding are below and are recorded in `progress.md`.

* **Finding 1, fixed.** R11 applies at builtin argument positions. Where the oracle's answer at a position is a defined error at the program's own line (88.909, 40.12, 40.23, 40.19, 40.28, 40.904, 93.904, per the review's table), that position raises the same error with the oracle's message text. Error numbers and inserts must match, including "a Q" for the original object and "The NIL object", and the argument numbering the oracle uses. The target positions the review lists as licensed (SIGSEGV, Error 5, garbage, 93.943, the REXX-package 88.909) keep the loud refusal.
  * Find the shared conversion site rather than patching each builtin. If no single site exists, message the controller with the site list before widening.
  * Witnesses: a datadriven case file (not a const table in Rust), with one row per position in the review's table. The expected output is the oracle's, rechecked at 2 runs each. Use a second STRING answer that is a different non-string object (for example a Directory) on a sample of positions, so that "defined over varied inputs" is witnessed.
  * Update Deviation 30 and the spec R11 line so they state the split between non-target and target positions. Re-derive `refusal-sites.tsv` in the same commit if lines move.
* **Finding 3, fixed.** `Array~append` must be linear whatever the trailing empty slots. Keep a last-item index, as the oracle's `lastItem` field does, and maintain it at every site that writes or clears a slot (put, remove, empty, new(n), sparse put, reshape, and anything else that touches the slots). That set of sites is the defect class here: enumerate the sites by grep, and commit the command and the list to the report. Add a test for each of the review's five shapes (b to e), and a debug assertion at the accessor that the cached index equals the recomputed one, then invert it once to show the probe is live. Correct the Task 12 mapping row 33 and the 11a claim with the new evidence.
* **Minor 5, fixed.** The spec section 2 sentence and the Task 12 report's "For Moritz" line state 472 (`activation.rs:502`, since `d27a9d441`). Delete each false sentence or replace it. Do not reword it into a new claim.
* **Findings 2 and 4 and Minor 6: queue only.** Add one file per item to `.superpowers/sdd/queued/` in the existing style, with the title, finder and date, probe, both engines' output, and suspected site. Do not fix them.

Rules:
* Commit per item, each with its test.
* The per-step check is fmt, `cargo clippy -p rexx-exec --all-targets --features pinning,sharing -- -D warnings`, `cargo test -p rexx-exec`, and the corpus pair.
* Use `CARGO_INCREMENTAL=0` and `-j 4`.
* Every interpreter run goes under memcap and timeout, from a fresh empty directory.
* Never git checkout, restore, stash or reset. Restore mutants by an exact edit and confirm with `git diff --stat`. No `bash -c`. rm only with literal paths. Check `df -i /tmp`.

Perf:
* Run callgrind on ALL programs (`rust/bench-programs/callgrind.sh`) against base61 `/tmp/claude-1000/p61/t1/bin/base/rexx-run` after each code commit.
* No program may newly exceed +0.5%. parse must not rise above its current +1.1979%.
* If either fails, stop and message the controller.

Then:
1. Commit everything.
2. Run `.superpowers/sdd/2026-10-07-phase-6-1/p61-gates/bggates.sh` once, alone, on the final HEAD, and wait for the status file to say finished.
3. Append a `## Final fix` section to `docs/superpowers/plans/phase-6-1-gate.md` with the commits, the gate result and the perf table.

The report goes to `final-fix-report.md`. Return only the status, the commits, the gate line, the perf line and any concerns. Do not dispatch subagents. Message the controller (SendMessage to "main") before widening scope.
