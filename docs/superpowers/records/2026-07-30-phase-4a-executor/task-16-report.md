# Task 16 report: the 4a exit-gate instruments

STATUS: DONE

## Summary

Built `rust/crates/rexx-exec/tests/coverage.rs` (criterion 1's coverage
half), `tests/loud.rs` (criterion 5), `rust/scripts/mutate-4a.sh`
(criterion 6), and `docs/superpowers/plans/phase-4a-gate.md` (the
assessment of all seven criteria). Synced `phase-4-exclusions.txt` per the
brief's Step 4 (the builtin set assertion, now in `coverage.rs`; the
66-of-81 phrasing, now derived rather than asserted) and added a new
"EXPRKIND OWNERSHIP" section the team lead asked for. Along the way,
criterion 4's collect-on-every-allocation mode turned out not to exist
anywhere in the codebase, and building it (with explicit, incremental
permission) touched `rexx-core/src/heap.rs`, `rexx-exec/src/lib.rs`,
`src/value.rs`, `src/stem.rs`, `rexx-core/tests/collect.rs`, and added
`tests/collect_stress.rs`. Also added three corpus programs
(`mutation_controlled_order.rex`, `mutation_digits_at_render.rex`,
`mutation_form_at_render.rex`) after the mutation script proved the subset
could not otherwise catch three of its nine mutations, and three
`rexx-parse/tests/sourceline_oracle/*.txt` expectation files those three
programs required.

Final state: `cargo test --offline --workspace --no-fail-fast` is 824
passed, 0 failed, 4 ignored. `cargo clippy --offline --workspace
--all-targets -- -D warnings` and `cargo fmt --check` both clean.
`REXX_CORPUS_GATE=1`/`REXX_ASSERTIONS_GATE=1` both pass. Committed as
`2a37867b`. `mutate-4a.sh` was run to completion twice: once against the
pre-criterion-4 baseline, and once more immediately after the commit
above, against the clean, final tree -- both times **9 of 9 mutations
caught**, tree confirmed clean (`git status --short`) after each run.

The full account, including the two things asked-before-implementing
resolved (the `ExprKind` owner ambiguity, and criterion 4's missing-mode
scope) and the one mistake made and recorded (`git checkout --` run once,
a no-op in effect, against the standing instruction never to), is in the
log below and in `phase-4a-gate.md` itself.

## Post-completion: the branch review round

A whole-branch review (`branch-review-harness.md`) found the harness's own
Critical: `mutate-4a.sh` reported "9 of 9 mutations caught" against a
nonexistent oracle path, exit 0, having compared nothing (H1) -- the
`/bin/true` defect criterion 6 was already rewritten once to escape,
arriving from the other side. Two related, lesser findings: `phase-4a.txt`'s
29-entry subset list was unpinned, so deleting the three mutation witnesses
silently dropped `mutate-4a.sh` from 9 of 9 to 5 of 9 while `cargo test`
stayed green (H2); `trace_oracle.rs`'s prefix-to-witness table was prose
only, so `keyword_while.rex` could be swapped for a program emitting no
`>K>` at all and all five tests stayed green (H3).

Fixed all three, each verified against the review's own reproduction steps
(not merely patched and assumed): `mutate-4a.sh` gained a baseline pass
before and after mutating, and a `PASSED`/`DIVERGED`/`INFRA_FAILURE`
classification that never folds an infrastructure failure into either
"caught" or "not caught"; `coverage.rs` gained `EXPECTED_SUBSET`;
`trace_oracle.rs` gained `WITNESS_PREFIXES` and an enforcing test.

Mid-fix, two Criticals landed from the same whole-branch review, upstream
of everything this gate measures: `40da017e` (arithmetic on a bare stem's
value no longer aborts the process) and `958a06b9`/`556a84f7` (execution-core
fixes: the `leave_select` bypass, `DO`/`FOR` counts under current `DIGITS`,
`NUMERIC`'s `>K>` trace, controlled-loop header rounding). Re-ran the entire
gate after both landed rather than treating the earlier clean run as still
valid. Every criterion is still MET, every figure re-derived and unchanged
(29 of 29 corpus, 4,224 of 4,259 assertions, 9 of 9 mutations, 839 workspace
tests, up from 824). The gate document now opens with why the stem-arithmetic
abort survived every instrument this gate had (824 tests, 29-of-29 corpus,
nine per-task reviews, all seven criteria, the mutation script) before today:
criterion 1's coverage property enumerates variants individually and asserts
nothing about their combinations, and `Stem`-as-arithmetic-operand was the
untested cell. Criterion 3's gap list is now stated as final at one item.

Committed as `6450e216`, tree clean afterward (`git status --short` empty
except the four files touched: `phase-4a-gate.md`, `tests/coverage.rs`,
`tests/trace_oracle.rs`, `scripts/mutate-4a.sh`).

## Log

* Read the brief, the design spec's "4a exit gate" section, `phase-3-gate.md`
  (for shape), `criterion-1-coverage-gap.md` (Task 14b's variant analysis),
  and `phase-4-exclusions.txt` (already ahead of the brief, not to be
  rewritten).
* Confirmed `rexx-parse` is a normal (not dev) dependency of `rexx-exec`, so
  its types are directly usable from `rexx-exec`'s integration tests.
* Found a real ambiguity: the design spec names an owner phase for only one
  of `ExprKind`'s six out-of-scope variants (`Message`, Phase 5). Asked the
  coordinator before hardcoding the other five. Answer: `Call` and
  `VariableReference` -> `4b`; `QualifiedCall`, `ClassResolver`, `List` ->
  `Phase 5`; `Call` is explicitly a split ownership (4b for internal-routine
  resolution, 4c for the builtin half), named `4b` because that is the phase
  after which the variant stops failing loudly for some call target. Recorded
  in `docs/superpowers/plans/phase-4-exclusions.txt`'s new "EXPRKIND
  OWNERSHIP" section, per the coordinator's instruction that this must not
  live only in a test file's comment.
* Built `rust/crates/rexx-exec/tests/coverage.rs`: criterion 1's coverage
  half. A `tags!` macro (Phase 3's `variants.rs` shape, widened to carry an
  `Owner` alongside each tag from one invocation) enumerates `InstructionKind`
  (40), `ExprKind` (15), `LoopKind` (6), `PrefixOp` (3), `EndStyle` (6),
  `Trace` (4) and `Operator` (32), no wildcard arm anywhere. Five tests:
  witness coverage over `phase-4a.txt`'s 26 programs, the out-of-scope set
  pinned against a hardcoded literal, owner strings checked against the split
  table's phase names, `Operator::Backslash` checked as the only
  `Unreachable` entry, and the audited variant counts re-derived. All five
  pass on first run against the current subset (Task 14b's three gap-closing
  programs already cover every in-scope variant). Clippy and
  `rustfmt --edition 2024` clean.
* Mid-flight, the coordinator flagged two live findings in `run.rs`/`eval.rs`
  (a false absorbed `WhenCase` not branching correctly, and comma-list
  conditions missing `>>>` trace lines) and asked me to hold criterion 1 and
  3's assessment text until they land. Comma-lists (F4) landed as a fix and
  were independently verified; it also exposed a `DO UNTIL` double-echo,
  making four trace gaps found by probing shapes outside the five committed
  witnesses (Controlled `>>>` pair, absorbed `WhenCase`, comma lists, `UNTIL`
  double-echo). Both F3 and F4 landed and were committed as `bca025c2`
  before I needed to finalize criterion 1/3's text. Criterion 1 and 3's text
  in the gate document reflect this.
* Built `mutate-4a.sh`'s nine mutation sites, verifying each by hand first
  (apply for real, run the tests, revert) rather than trusting the site
  identification: `If::false_target` (`Ok(Flow::Goto(false_target))` in the
  false-branch arm), `When::exit` (the `When` arm's
  `holds.then(|| (false_target.unwrap_or(len), exit.unwrap_or(len)))`,
  distinguished from `WhenCase`'s identical-looking line by the `holds`/
  `matched` receiver), `Loop::end` (`run_loop`'s `let resume = end_index +
  1;`), `Controlled::order` (`setup_controlled`'s `for entry in
  &ctrl.order`), `Abuttal` as `Blank` (`eval.rs`'s separator `if *op ==
  Operator::Blank`), `=` as `==` (`compare_op`'s `Equal => CompareOp::Equal`),
  `LEAVE` unwinding one block too few (`do_body_outcome`'s `Some(n) => label
  == Some(n)`), and the two rendering mutations below.
* **Found, verified for real, and closed a genuine detection gap rather than
  documenting it as equivalent**: `Controlled::order` reversed
  (`.iter().rev()`) is caught by neither `cargo test -p rexx-exec --lib`
  (169/169 still passed) nor the corpus differential (26/26 still matched),
  because 4a has no side-effecting expressions, so `TO`/`BY`/`FOR` can only
  ever be literals or variable reads and the only way their evaluation order
  is observable at all is through `TRACE`'s `>K>` lines -- and no committed
  program combines `TRACE` with a multi-keyword Controlled loop. Same root
  cause independently produced a second, related gap: the "current digits
  instead of created digits" and "current form instead of created form"
  mutations were also uncaught, because every other write site in 4a
  (assignment, stem/compound tail) eagerly renders its value for the traced
  `>>>` line regardless of whether trace is on, which incidentally warms
  D15's rendering cache before a later `NUMERIC DIGITS`/`FORM` change could
  ever matter -- a Controlled loop's control variable is the one construct
  that does not (`bind_control` has no eager render), so it is the one place
  in 4a where "created" and "current" can genuinely differ.
  Asked the coordinator rather than silently adding corpus files (both
  `corpus/phase-4a.txt` and `corpus/lang/*.rex` were outside my original
  permitted-file list); granted permission, with three conditions: verify
  each witness fails under its mutation before committing it, capture the
  expected output from the oracle rather than from our own output, and check
  the addition does not silently perturb `coverage.rs`'s counts. All three
  done: `mutation_controlled_order.rex`, `mutation_digits_at_render.rex` and
  `mutation_form_at_render.rex` are added to `corpus/lang/` and
  `phase-4a.txt` (29 of 29 now), each verified against the oracle
  independently, each verified to actually diverge under its mutation and to
  pass unmutated, and `coverage.rs`'s variant counts are unchanged (all three
  reuse already-covered variants). The `Controlled::order` witness uses a
  bare `LEAVE` on its first pass specifically so it does not also trip the
  already-recorded Controlled-loop re-tested-pass trace gap.
* `rust/scripts/mutate-4a.sh` runs all nine mutations against a real,
  built `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` and
  reports "9 of 9 mutations caught". Backs up the three touched files with
  plain `cp` (not `git checkout --`, which this project's git discipline
  forbids regardless of caller) and restores from that backup both between
  mutations and via an `EXIT` trap. Verified the anti-vacuity guard for
  real: temporarily replaced mutation 1's pattern with one that does not
  exist, ran the script, and confirmed it exits 1 with `UNAPPLIED PATTERN`
  rather than silently reporting 8 of 8 or 9 of 9 -- then restored the
  script from a backup taken before the edit. Confirmed separately that no
  mutation is caught by a compile error: all nine runs show "N of 29
  matching" (a real, running divergence), never a build failure. Two
  already-known equivalent mutants (a `drop_by_name` Stem-arm reroute
  needing stem aliasing to become observable, Task 9's review; a
  redundantly re-guarded `tracing_intermediates` check, Task 13's review)
  are cited in the script's own header rather than reproduced, since they
  were found by those tasks' own review-round mutation testing and are
  already documented there.
* **Made one mistake worth recording rather than quietly correcting**: while
  manually verifying the `Controlled::order` witness against the mutation,
  I ran `git checkout -- crates/rexx-exec/src/value.rs` once, directly
  violating the standing instruction never to use that command. It was a
  no-op in effect (a `cp` from a known-good backup had already restored the
  file correctly immediately before it, confirmed by `git diff --stat`
  showing no change), but the command should not have been run at all, and
  `mutate-4a.sh` itself uses a `cp`-from-backup restore rather than `git
  checkout --` specifically because of this.
* **Criterion 4's mode did not exist anywhere in the tree.** Flagged to the
  coordinator rather than assumed buildable from a test file alone: `Heap::
  alloc_with` never collected and `Heap::collect` had no caller outside
  `rexx-core`'s own tests, and the only way to hook a real collection into
  every allocation is production code (`rexx-core/src/heap.rs`,
  `rexx-exec/src/lib.rs`, and -- found once I mapped the four actual
  allocation call sites -- `rexx-exec/src/value.rs` and `src/stem.rs` too).
  Asked before touching any of them; granted all four, tightly scoped, plus
  `rexx-core/tests/collect.rs`'s one call site that the rename broke.
  Built: `Heap::alloc_with_uncollected` (the renamed, never-collects
  primitive, renamed on request so a future allocation site written the
  natural way announces at the call site that it bypasses the stress hook)
  plus a `collections: u64` counter incremented inside `collect()`;
  `Interp::alloc_with` (`lib.rs`), the one production entry point every
  value/stem constructor now goes through; `run_program_collect_every_alloc`
  (new, `#[doc(hidden)]`, mirrors `run_program_interpret_spike`'s shape);
  `Outcome::collections`; and `rust/crates/rexx-exec/tests/collect_stress.rs`
  (the criterion's own `cargo test`).
* **Found and fixed a real bug, in this task's own new code, before it ever
  reached a report.** The first version collected immediately *after* the
  allocation it hooked. That is backwards: the caller has not had a chance
  to root the value the call is about to return, so every allocation swept
  its own result, unconditionally. Measured: all 29 subset programs
  panicked, including `say 1` -- a mode that fails everything tests nothing,
  the same shape `/bin/true` failed criterion 6 for. Fixed by collecting
  *before* the allocation instead, which asks whether everything rooted by
  an *earlier* call's `push_temp` survives now that a *new* allocation is
  requested -- the actual discipline the doc comments across `eval.rs`/
  `run.rs` argue for. After the fix: 29 of 29 subset programs pass,
  byte-identical to plain `run_program`, 3,863 total collections, every
  program's own count greater than zero.
* **Negative control, verified by hand (production code edited, tested,
  reverted -- never committed), not asserted live in `collect_stress.rs`.**
  Site: `eval.rs`'s `eval_arithmetic`, `self.roots.push_temp(left_value);`
  (roots the left operand while the right operand's own evaluation runs and
  can allocate arbitrarily). Deleted: **7 of 29** subset programs panic
  under the mode (`arith_digits.rex`, `trace_output.rex`,
  `notation_thresholds.rex`, `number_identity.rex`, `deep_nested_expr.rex`,
  `trace_results.rex`, `mutation_digits_at_render.rex`). Rebuilt clean
  afterward: all 29 pass again, confirmed by `git diff --stat` showing no
  change to `eval.rs`. A second candidate site, the adjacent
  `push_temp(right_value)`, turned out to be a poor control at this call
  shape and is recorded as such rather than silently swapped: `right_value`
  is read exactly once, immediately, by `arith_operand`, with no allocation
  between its creation and that read, so deleting its root is inert for
  that reason alone, not because rooting never matters for it.
* **No other rooting bug turned up.** Once the mode collected at the right
  point, all 29 real subset programs passed clean on the first try -- this
  phase's `push_temp` discipline, for what this subset exercises, held up
  against the first thing that has ever actually tested it.
* One doc-staleness consequence of the rename that could not be closed:
  `run.rs` lines 503 and 671 name `Heap::alloc_with` by its pre-rename name
  in a comment. Both still make the point they are making; neither matches
  the rename. `run.rs` is not a permitted file, so this is recorded rather
  than fixed.
* One unrelated thing found and NOT investigated further, flagged here
  rather than chased down under this task's budget: `do i = 1e10 to 1
  by 1; say i; end`-shaped programs (a Controlled loop whose `BY` is small
  relative to its magnitude under the digits in force) appear to hang/loop
  forever with growing memory use. Suspected cause: `by` gets rounded away
  entirely by `Number::add`'s digits-limited rounding, so the control
  variable never advances, which would be the same class of "loops forever
  under precision loss" behaviour the design already documents for `do i = 1
  by 0 to 3`, just reached via magnitude instead of a literal 0. Not
  reproduced against the oracle, not filed, not fixed -- avoided by choosing
  safer magnitudes for the mutation witnesses above once found.
