# Phase 6.1 parse perf round 2 (controller)

Criterion 7 (spec section 8) is the last open item of Phase 6.1. parse is +1.1983% Ir against base61 `/tmp/claude-1000/p61/t1/bin/base/rexx-run`, and the budget is +0.5%. Read `task-12-report.md` sections "## Parse attribution" and "## Parse perf round", and `final-fix-report.md`'s perf section. Since then `cf167fba1` added about 7.5K Ir of fixed cost per program, which is accepted.

**Target.** parse at or below +0.5% against base61. Measure every program with `rust/bench-programs/callgrind.sh` after every commit, not a subset. No program may newly exceed +0.5%, and no program already inside the budget may rise by more than 0.1 point.

**Where the cost is.** About 11M Ir per run is the sweep's freed-body byte accounting (`Heap::collect`, `freed_bytes += body.held_bytes()` for every freed object, about 784K frees over 12 collections). Removing it entirely should put parse near +0.48%. The rest is exec_parse (Task 5a's per-piece charge test) and two accepted codegen shifts.

**Candidate designs.** Measure before choosing; neither is mandated.
* (A) Survivor sum. The mark phase already visits every survivor and matches its body to trace it. Sum `held_bytes` there in release, as the debug build already does, set the running figure to that sum after the sweep, and drop the freed-bytes read. Cost then scales with survivors, not garbage. A Task 12 variant that summed "the smaller of survivors and dead" measured worse (+1.61%). Find out why before rejecting (A); it probably paid for both counts.
* (B) A per-object or per-slot holds-bytes marker that the sweep tests before reading the body. Every site that charges or grows bytes must set it. That has been this phase's defect class (missed charge sites), so (B) needs the compiler to enforce the sites (a type, not a convention) and a debug check that the marker agrees with `held_bytes`.

**Invariant.** The running live-byte figure must stay exact: the `Heap::collect` debug assertion must hold, and the byte-accounting tests and R10's `bytes_due` behaviour must be unchanged. UNINIT resurrection must still count resurrected bodies. Under (A), the resurrect loop already visits them; check it.

Rules:
* Commit per change, each with its all-program cgdiff in the message or the report.
* The per-step check is fmt, `cargo clippy -p rexx-exec --all-targets --features pinning,sharing -- -D warnings`, the workspace clippy, `cargo test -p rexx-exec -p rexx-core -p rexx-api`, and the corpus pair. Run `refusal_sites` too if a file holding a constructor changes.
* Use `CARGO_INCREMENTAL=0` and `-j 4`, with your own target dir under `/tmp/claude-1000/p61/pr2/`. Check `df -i /tmp`. Delete scratch target dirs when done.
* Run every interpreter under memcap and timeout.
* Never git checkout, restore, stash or reset. No `bash -c`. rm only with literal paths.
* Build scratch variants from `git archive` copies (`touch` the tree and require a `Compiling rexx-exec` line).
* No perf runs during gates.

If the target is met:
1. Commit.
2. Run `.superpowers/sdd/2026-10-07-phase-6-1/p61-gates/bggates.sh` once, alone, on the final HEAD, and wait for its status file to say finished.
3. Append `## Parse round 2` to `docs/superpowers/plans/phase-6-1-gate.md` with the commits, the all-program table and the gate line, and update criterion 7's line and roadmap row 6.1.

If neither design reaches +0.5%, stop. Report both designs' all-program tables and cgdiffs, and change no records.

The report goes to `parse-round2-report.md`. Return only the status, the commits, the perf line, the gate line and any concerns. Do not dispatch subagents.
