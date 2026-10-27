# Task 8.4 fix round 4 (close-time workspace test)

The controller ran `cargo test --release --no-fail-fast --workspace` at 75066653a (log:
docs/superpowers/records/2026-10-18-phase-9d/t8.4/close-workspace-tests.txt). Task 8.4's own reports and
re-reviews are task-8.4-report.md and task-8.4-rereview-r1/-r2/-r3.md; read the report's summary of the
parser move (declaration checks now run in the parser, `Declared`, `parse_known_program`) first.
Records go under docs/superpowers/records/2026-10-18-phase-9d/t8.4/r4/. Append a "Fix round 4" section
to task-8.4-report.md.

Fix these, each with the cause stated and the commit that introduced it (bisect by building, not by reading):
1. rexx-exec `--test coverage`, every_in_scope_variant_is_witnessed_by_the_phase_subsets:
   corpus/lang/directive_constant_expression_needs_class.rex now fails `parse_program` with 99.906 (the
   check moved into the parser). Decide by the test's purpose (it walks the subset programs' parsed
   instructions): the right fix may be in the test's parse call, the subset list, or the program. Do not
   weaken what the test proves; say what it proves before and after.
2. rexx-bench `--bin rexx-bench-suite`, the_axis_list_covers_every_bench_program: bench-programs/define.rex
   (added by 8d409845b, Task 8.3) is missing from AXES. Add the axis as the other axes are declared
   (role, runnability), and check every_declared_runnability_still_holds passes with
   rust/target/release/rexx built.
3. rexx-exec `--test directive_options`, lostdigits_raises_98_972_where_the_oracle_does_and_nowhere_else:
   the do-repeat-count row (`numeric digits 3 / do 123456789`) now answers ("", 158) on both sides, so
   the asserted divergence closed. Find which commit closed it and whether the new answer follows rexxref
   (LOSTDIGITS, DO count conversion); if correct, move the row to the agreeing set as the test's comment
   asks; if the crate's new answer is wrong, fix it.

Not yours (queued, leave): datetime_zone (DST item), rexxutil_process a_command_whose_child_was_reaped_elsewhere_fails_with_127 (passes 3/3 alone; a load flake, the controller queues it), and the arity tests that only need rust/target/release/rexx built (they pass once it is).

Then run the workspace test again the same way (`CARGO_INCREMENTAL=0 memcap 24G cargo test -j 4 --release --no-fail-fast --workspace` from rust/, with rust/target/release/rexx built first) and record it; every failure must be one of the three left above. Perf: callgrind on every program against f0164456d only if you change interpreter code (rexx-base-f0164456d in /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.4/), +0.05% libc-subtracted per program. fmt and clippy. Do not touch .superpowers/sdd/queued/, queued-inventory.tsv, task-8-batches.md, oracle-defects/ or phase-4-exclusions.txt. CARGO_INCREMENTAL=0; cap memory with memcap; never git add -A, git reset --hard, git checkout --, or bare git stash; rm only literal paths; no bash -c. Never go idle on a background command: run long commands in the foreground (timeout up to 600000) or poll them. Do not dispatch subagents. Commit, then report.
