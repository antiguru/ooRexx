# Task 3 report: the driver, engine selection, and the dual-engine harness

BASE `09cba5a8` (Task 2 plus its fix round).
Commit `533c48a5`.
Fix round 1: commit `777fa4a9`.

**Fix round 1 changed several claims below.** They are corrected in place rather than left standing
beside a retraction; see "Fix round 1" at the end for the list and the new mutation evidence.

## What was built

### The driver, `rust/crates/rexx-exec/src/ir/drive.rs`

`Interp::run_chunk(&mut self, code: &Code<'_>, chunk: &Chunk, source: Option<&ProgramSource>) ->
Result<Ended, Failure>` is the entry point, exactly the signature the brief specifies.
It is three functions rather than one, and each split has a reason that is not tidiness.

`run_chunk` opens the register region with `RootSet::reserve_temps(chunk.registers as usize)`,
delegates, and truncates with `pop_frame` on the way out.
The truncation covers the raised and loud paths as well as the successful one, which is why the
loop lives in a second function rather than inline: an early `return` from the loop would otherwise
skip it.

`run_chunk_clauses` is the outer level, one iteration per clause.
It reads the activation's `pc` as an **instruction** index, maps it through `chunk.op_of` to an op
index, grants the procedure permission, runs the clause, offers a failure to a trap, and applies the
resulting `Flow`.
The `pc` never becomes an op index, which is what lets `apply_flow` be reused unchanged: `Flow::Goto`
and `Flow::Signal` both carry instruction indices, and `Signal`'s resolve against the activation's
own body, which inside an `INTERPRET` fragment is not the body being stepped.

`run_clause_ops` is the inner level: it takes the clause's first op index and answers the clause's
`Flow`.
`Op::Generic` delegates to `Interp::step_in_temps_frame`.
`Op::Clause` and `Op::EvalExpr` have no constructor, so their arms are loud
(`Loud::op_not_driven`) rather than a panic, matching this crate's standing rule for a state the
type system admits and the code does not produce.

Two things the driver does **not** do, both deliberate:

* **It does not call `Interp::step`.** That is not the clause unit -- the wrapper carries the
  per-clause clock invalidation, the `>I>` trace-entry decay, `current_value_indent`, the `SIGL`
  clause line, the clause boundary through `in_clause`, the clause echo, the GC temps frame with its
  watermark tripwire, and failure-site resolution.
  This is not merely documented: `step` is a private `fn` of the `run` module and `ir::drive` is not
  a descendant of it, so the wrong delegation is a compile error.
  Verified by making it (mutation M5 below): `error[E0624]: method 'step' is private`.
* **It does not move the trap offer.** `self.offer_to_trap(code, failure)?` sits at the same
  position it holds in `run_activation`'s loop, between the clause and `apply_flow`, because the
  position is the semantics: one offer per activation, made by the activation that is unwinding.

### Engine selection

`Engine` (`TreeWalker` | `Ir`) is a `pub` enum in `invocation.rs`, a field of `Invocation`, set by
`Invocation::with_engine(self, engine: Engine) -> Invocation`, and threaded to `Interp::engine` by
`execute` the way `collect_every_alloc` already is.
`Invocation::into_parts` widened from a 2-tuple to a 3-tuple.
`Engine::TreeWalker` is the default, in `Invocation::none()` and in `Interp::new()`.

An earlier draft of this report said a `#[derive(Default)]` on the enum named the same arm "a third
time so no constructor can disagree". **That was false**: nothing called `Engine::default()`, so the
derive named an arm no code path consulted and could not have disagreed with anything. The derive is
removed. `Invocation::none()` is now the one place a caller who did not choose gets an answer, and
`an_invocation_that_chose_no_engine_runs_on_the_tree_walker` is what states it.

**The selection itself is inside `run_activation`, not at its two call sites, and that is a
deviation from the brief's Step 6 wording that makes the property it asks for stronger rather than
weaker.**
`run_activation` is the one function that runs an activation's body, so `Interp::run` and
`resolve_and_run_call` both reach the compiled stream through one decision, by construction.
Putting the decision at the two callers would mean rebuilding the `Code` value at each -- it is
built from the activation's program, plan and body selector at the top of `run_activation` -- and a
caller that was missed would tree-walk its whole body while a dual-engine comparison still passed.
The property the brief is protecting ("selection happens at both") is asserted anyway, and by a test
that can distinguish the two: see `the_ir_engine_drives_every_body_the_program_enters` below, which
counts three driven bodies for a program with a main body, a `CALL`ed label and a `::ROUTINE`.

`chunk_for` needs a `BodyKey`, and an `Activation` could not produce one: it carried the body
selector (`body`, the same integer `BodyKey::directive` carries) but not the program's id.
So `Activation` gained `program_id: ProgramId` and `Activation::body_key()`, and
`Activation::new`/`nested`/`routine` each gained a parameter for it.
The alternative considered and rejected was recovering the id by scanning `Interp::programs` for an
`Rc::ptr_eq` match, which is a reverse lookup that can fail and is O(n) per activation entry on a
phase whose subject is performance.
`Activation::nested` is now over clippy's argument threshold and carries an `#[allow]` with the
reason: the three parameters that could be bundled are `program`, `program_id` and `body`, and
bundling them would put a type between every reader and `Activation::program`, which is the field
the instruction loop clones its `Rc` from.

### Reporting a refusal

`Outcome` gained `pub chunks_refused: usize`, filled from `Interp::chunks_refused`.
It follows `collections` exactly -- a diagnostic counter on the public outcome, asserted by a gate --
and it exists because a refusal is invisible otherwise: a body the compiler declines runs on the
tree-walker under **both** arms, so the two agree and a dual-engine sweep passes while the engine
under test never ran.

### The dual-engine harness, `rust/crates/rexx-exec/tests/ir_dual.rs`

Four populations, built once and run twice each:

| population | source | cases |
| --- | --- | --- |
| `corpus` | `rust/corpus/phase-4a.txt`, `phase-4b.txt`, `phase-4c.txt` | 51 |
| `expressions` | `ootest/ooRexx/base/expressions`, via `extract_assertions` | 4259 |
| `bif` | `ootest/ooRexx/base/bif`, via `extract_bif` (value rows and raise rows) | 5185 |
| `keyword` | `ootest/ooRexx/base/keyword`, via `extract_keyword` | 896 |

An `ootest` population is named for its suite directory, which is what lets the pin compare the
declared list against the directories the sibling harnesses read with no mapping table in between.

10391 programs, 20782 runs, 8.4 seconds in the dev profile.
stdout, stderr and exit status are compared **unnormalised** -- there is no oracle in this
comparison, so `corpus.rs`'s DEVIATION 0 does not apply and a trace line's own indentation has to
match exactly.
`chunks_refused` is asserted zero on both arms, per case.

**Both arms run from one list**: `populations()` builds each `Case` once and `compare()` runs that
same `Case` twice, so "the two arms saw the same programs" is a property of the code rather than
something asserted about two separately built lists.

**Every completeness pin compares against something outside this file.**
The corpus half goes against the corpus directory listing; the `ootest` half goes against the suite
roots the sibling harnesses in `tests/` name in their own sources.
A pin against a second literal in the same file is worth nothing, because deleting a population and
the line declaring it is one edit -- measured, and the reason this changed in fix round 1.

**There is no REPORT mode, and the four gate variables are not read at all.**
The brief's Step 7 asks that every gate variable be STRICT in both arms.
This file goes further: it has no mode in which it exits 0 having found a divergence, so setting
`REXX_CORPUS_GATE`, `REXX_ASSERTIONS_GATE`, `REXX_BIF_GATE` and `REXX_KEYWORD_GATE` changes nothing
about what it checks.
A divergence between two engines running the same program is a defect at any point in the phase,
with nothing to forgive and no denominator to report, so the REPORT default that is right for the
four existing harnesses would have been the exact trap the brief warns about.
The four variables still matter for the *other* half of the argument, and the verification below
runs the whole workspace with all four set: this file says the two engines agree, and says nothing
about whether either is right; the four harnesses in STRICT say the tree-walker is right.
Neither substitutes for the other.

## What the harness proves, and what it does not

**It does not prove that a promoted construct is right, because nothing is promoted.**
Every instruction compiles to `Op::Generic`, which delegates its clause back to the tree-walker's own
clause unit, so an all-`Generic` program agrees with itself by construction.
This is the vacuity the brief names, and it is real: the sweep's 10391 programs are not evidence
about expression evaluation, arithmetic, or any construct's semantics.
Measured rather than reasoned: mutation M1 below deletes engine selection outright, and all four
`ir_dual` tests stay green.

What it does prove is everything *around* the delegation -- the part the driver adds and does not
share with `run_activation`.
Two of those are demonstrated rather than asserted, by mutations that go red only there:

* `grant_procedure_permission` removed from the driver's loop (M3) -- caught by the sweep alone.
* the trap offer replaced by a bare re-throw (M4) -- caught by the sweep alone.

And the driver's outer loop terminating where the tree-walker's does, the `pc` staying an
instruction index across `Goto`/`Signal`, and every body in the population compiling are each
covered by the same sweep.
The constructs the sweep reaches are not a guess: `ootest/ooRexx/base/keyword` has one
`.testGroup` per keyword, `SIGNAL`, `IF`, `SELECT`, `DO`, `LOOP`, `LEAVE`, `ITERATE`, `INTERPRET`,
`CALL`, `TRACE`, `USELOCAL` and `EXPOSE` among them, and `extract_keyword` turns each into
programs this file runs.

**Which engine actually ran is not observable from a program's output at this point in the phase.**
`ir_dual` makes no attempt to infer it.
That question is answered where it can be answered honestly, by counting `run_chunk` entries, in
`src/ir/drive/tests.rs`.

## Every test added, with its mutation evidence

Nine tests: three in `src/ir/drive/tests.rs`, five in `tests/ir_dual.rs`, one in
`src/invocation.rs`.
Every mutation below was applied to a `cp` backup-verified tree, run with `cargo test -p rexx-exec
--no-fail-fast` (the whole crate, so "nothing else caught it" is measured rather than assumed), and
restored from the backup with `sha256sum -c`.
No `git checkout --` was used at any point.

Unmutated crate total for reference: **655 passed, 0 failed**.

### `ir::drive::tests::the_ir_engine_drives_every_body_the_program_enters`

Runs a program with three bodies -- its main body, a `CALL`ed internal label, and a `::ROUTINE` --
under `Engine::Ir`, and asserts `run_chunk` was entered exactly three times.

* **M1**, engine selection disabled (`&& std::hint::black_box(false)` on the condition in
  `run_activation`): **RED, this test alone**, 654 passed / 1 failed.
  Every `ir_dual` test stayed green, which is the vacuity statement above, measured.
* **M2**, selection narrowed to the outermost activation (`&& self.activations.len() == 1`, which is
  what "selection happens at `Interp::run` only" looks like): **RED, this test alone**,
  654 passed / 1 failed.
* **M2b**, selection narrowed the other way (`&& self.activations.len() > 1`, which is what
  "selection happens at `resolve_and_run_call` only" looks like): **RED, this test alone**,
  654 passed / 1 failed.
  The reviewer raised this direction; running it closes the argument, since one count of three
  distinguishes both one-sided placements from the correct one.
* **M6**, the driver's clause loop skipped entirely: RED (also caught by two `ir_dual` tests).
* **M7**, `chunk_for` made to refuse every body: RED (also caught by three other tests).

Adds coverage: M1, M2 and M2b are caught by nothing else in the crate.

### `ir::drive::tests::the_tree_walker_drives_no_chunk_at_all`

The negative control: the same program on `Engine::TreeWalker` drives no chunk.

* **M11**, selection widened to both arms (`Engine::Ir | Engine::TreeWalker`): **RED, this test
  alone**, 654 passed / 1 failed.

Adds coverage: yes, measured -- nothing else goes red.
(Before fix round 1 this mutation reddened its sibling too, because the counter was process-wide and
the two tests contaminated each other. That it now reddens exactly one test is itself evidence the
per-thread counter isolates them.)

### `ir::drive::tests::no_body_is_refused_by_either_engine`

`Outcome::chunks_refused` is zero on both engines for a three-body program.

* **M7**, `chunk_for` made to refuse every body: **RED**, 651 passed / 4 failed (this test,
  `the_ir_engine_drives_every_body_the_program_enters`, the `ir_dual` sweep, and Task 2's own
  `chunk_for_compiles_a_body_once_across_repeated_lookups`).

Adds coverage: no, not for M7 -- three other tests catch it.
Kept because it is the only one of the four that asserts the *counter* rather than the driving, on
both engines, and the only one that reaches it through `run_program` and its own thread rather than
through `execute`.

### `invocation::tests::an_invocation_that_chose_no_engine_runs_on_the_tree_walker`

The default, and that neither `with_argument` nor `with_input` disturbs a chosen engine.

* **MD**, `Invocation::none()` switched to `Engine::Ir` -- which is precisely the change a later
  task makes: **RED, this test alone**, 654 passed / 1 failed.

Adds coverage: yes. It is also the whole of what fix round 1's I2 needed, from the other side: with
the default flipped, the entire crate runs every program on the compiled stream and **654 of 655
tests still pass**, the one failure being this test stating the old default. The counting tests
above are unaffected because they name their engine and count per thread.

### `both_engines_agree_on_an_all_generic_program`

The brief's own Step 1 test, plus an absolute anchor (`5\n`) that an arm-versus-arm comparison
cannot supply.

* **M6**, the driver's clause loop skipped entirely: **RED**.

Adds coverage: not for any mutation tried -- the sweep catches M6 as well.
Its unique content is the absolute expected output, which no self-comparison can provide.

### `the_dual_harness_reads_every_phase_subset_file`

`SUBSET_FILES` against the corpus directory listing.

* **M9**, `phase-4c.txt` dropped from `SUBSET_FILES`: **RED, this test alone**, 654 passed /
  1 failed.
  The sweep stayed green over the shrunken population, which is exactly the shape this pin exists
  for.

Adds coverage: yes, measured -- nothing else goes red.

### `the_sweep_runs_every_ootest_suite_a_sibling_harness_runs`

`OOTEST_SUITES` against the suite roots the other harnesses in `tests/` name in their own sources.

* **MA**, the `bif` suite deleted from `populations()`'s dispatch **and** from `OOTEST_SUITES`
  together -- the two-sided deletion, 5185 cases gone: **RED**, 653 passed / 2 failed (this test and
  `every_population_the_tree_calls_for_is_present_and_non_empty`).
  This is the mutation that was green before fix round 1, and it is the reason the pin no longer
  compares two in-repo lists.
* **MC**, the scanner's marker misspelled so it finds no suite at all: **RED**, 653 passed /
  2 failed. The pin's own emptiness guard is what fires, so a scanner that silently stopped matching
  cannot make the comparison vacuous.

Adds coverage: yes for both -- nothing else in the crate goes red for either.

### `every_population_the_tree_calls_for_is_present_and_non_empty`

The population names against a requirement derived from the tree: `corpus` because `rust/corpus/`
has phase subset files, and one per suite a sibling harness runs.

* **MB**, the corpus population (51 cases) removed from the builder: **RED, this test alone**,
  654 passed / 1 failed. The sweep stayed green over the remaining 10340 cases.
* **MA** and **MC**, above: RED here too.

Adds coverage: yes, measured. This is the test whose earlier version was the review's I1 -- it
compared `populations()`'s names against a literal beside them, so a one-edit deletion of both left
it green.

### `both_engines_agree_across_every_population`

The sweep itself.

* **M3**, `grant_procedure_permission` removed from the driver's loop: **RED, this test alone**,
  654 passed / 1 failed.
* **M4**, the trap offer replaced by `return Err(failure)`: **RED, this test alone**,
  654 passed / 1 failed.
* **M6**, the clause loop skipped: RED (with two others).
* **M7**, `chunk_for` refusing everything: RED (with three others).

Adds coverage: yes for M3 and M4 -- nothing else in the crate catches either.

### One mutation that stayed green, recorded rather than omitted

* **M10**, `self.roots.pop_frame(registers)` deleted from `run_chunk`: **GREEN**, 655 passed /
  0 failed.
  `chunk.registers` is `0` until a register allocator exists, so `reserve_temps(0)` adds nothing to
  truncate and no test in the tree can distinguish the two.
  A test asserting the temporaries stack's balance across a chunk was written and then **deleted**
  for exactly this reason: it could not fail, which is a defect rather than coverage.
  The first witness for the reserve/truncate pair is the task that gives `registers` a non-zero
  value.

### A mutation that could not be applied

* **M5**, `Op::Generic` delegating to `Interp::step` instead of `step_in_temps_frame`: does not
  compile.
  `error[E0624]: method 'step' is private`.
  The hazard the brief names is structurally excluded from `ir::drive`, not merely documented there.

## Dead-code annotations removed

All of them, except one field, which is named here rather than left silently in place.

* `chunk_for` (`plan.rs`) -- removed; `run_activation` calls it.
* `compile` (`ir/compile.rs`) -- removed; `chunk_for` calls it from production.
* `Chunk::ops`, `Chunk::op_of`, `Chunk::registers` (`ir/mod.rs`) -- removed; `run_clause_ops`,
  `run_chunk_clauses` and `run_chunk` read them respectively.
* `ChunkTooLarge`, the struct-level annotation (`ir/mod.rs`) -- removed; `chunk_for` matches on it.
* `Interp::chunks`, `Interp::chunks_refused` (`lib.rs`) -- removed.

**Not removed: `ChunkTooLarge::what`.**
The struct-level `allow` came off and a narrower one went on the field, because the field genuinely
has no production reader.
`chunk_for` is the only caller that sees an `Err`, and a refusal is not a failure there: it counts
the body in `chunks_refused` and runs it on the tree-walker, with nothing to print.
`#[derive(Debug)]` does not count as a read for dead-code analysis, and rustc says so explicitly in
the warning.
The alternatives were a fake reader (`let _ = too_large.what;`), which is worse than an annotation
that tells the truth, or deleting the field, which loses the only thing that would tell two refusal
reasons apart once a register allocator gives the type a second one.

`Op::Clause` and `Op::EvalExpr` keep their `#[expect(dead_code)]`, as the brief instructs.
Confirmed empirically that this task's `match` arms do not unfulfil those expectations: pattern
matching is not construction, which `golden.rs`'s renderer already demonstrated before this task.
`render` stays `#[cfg(test)]`.

## Files changed

* `rust/crates/rexx-exec/src/ir/drive.rs` -- new, the driver.
* `rust/crates/rexx-exec/src/ir/drive/tests.rs` -- new, the engine-selection tests.
* `rust/crates/rexx-exec/tests/ir_dual.rs` -- new, the dual-engine harness.
* `rust/crates/rexx-exec/src/ir/mod.rs` -- `mod drive;`, annotations removed, module doc corrected
  (it said `Interp::step`, which is the delegation the driver must not make).
* `rust/crates/rexx-exec/src/ir/compile.rs`, `src/plan.rs` -- annotations removed.
* `rust/crates/rexx-exec/src/invocation.rs` -- `Engine`, `with_engine`, `into_parts` widened, and
  the default-engine test.
* `rust/crates/rexx-exec/src/lib.rs` -- `Engine` re-exported, `Interp::engine`,
  `Outcome::chunks_refused`, `Loud::chunk_map_too_short`, `Loud::op_not_driven`, annotations
  removed, `Interp::run`'s local renamed to `program_id`.
* `rust/crates/rexx-exec/src/activation.rs` -- `Activation::program_id`, `Activation::body_key`,
  three constructor signatures.
* `rust/crates/rexx-exec/src/run.rs` -- engine selection in `run_activation`, the body-key
  assertion beside it, `program_id` threaded through `resolve_and_run_call`'s label and routine
  pushes.
* `rust/crates/rexx-exec/src/{eval,plan,queue,stem,trace}.rs`,
  `src/builtin/{convert,numeric}.rs` -- test helpers updated for the constructor signatures.

## Verification

Run from `rust/`, each status read unpiped.

```text
cargo fmt --all --check                                      0
CARGO_TARGET_DIR=<empty> cargo clippy --workspace \
  --all-targets -- -D warnings                               0
cargo test --workspace                                       0   1349 passed, 0 failed, 4 ignored
REXX_CORPUS_GATE=1 REXX_ASSERTIONS_GATE=1 \
  REXX_BIF_GATE=1 REXX_KEYWORD_GATE=1 cargo test --workspace  0   1349 passed, 0 failed, 4 ignored
REXX_CORPUS_GATE=1 ... cargo test --workspace --release      0   1349 passed, 0 failed, 4 ignored
```

Baseline at `09cba5a8` was **1340 passed, 0 failed**, measured in this session before any edit.
Nine tests were added and the total moved by exactly nine.
(Ten were written; the one that could not fail was deleted, per the M10 entry above.)

The STRICT runs report all four gates engaged -- `mode: STRICT` for each of
`REXX_ASSERTIONS_GATE`, `REXX_BIF_GATE`, `REXX_CORPUS_GATE`, `REXX_KEYWORD_GATE` -- and the corpus
differential reports **51 of 51 matching** against the oracle, in both profiles.

The clippy run was made from an **empty** target directory (`CARGO_TARGET_DIR` pointed at a fresh
path, removed afterwards), which is what this crate's own rule asks for: a same-session green
against a warm target is provisional, and the earlier draft of this report reported one.

The flaky `state_builtin_oracle::queued_null_line` the review saw did not appear in any of the three
full runs above.

## The plan key and the chunk key are now pinned

`run_activation` carries a `debug_assert` that the activation's `body_key()` names the plan the
activation is actually running with (`Rc::ptr_eq` against `Interp::plans`, a read, never a
`plan_for` that would cache the plan a second time under the wrong key).

Nothing else pins it, and the class is currently unreachable rather than absent: production loads
exactly one program, so every `ProgramId` is `0` and `BodyKey` is discriminated by `directive`
alone. All eleven construction sites were individually correct without it -- verified by mutation,
`installed.program` replaced with `ProgramId(999)` at the `::ROUTINE` push, which before the
assertion left the crate green and now fails **18** tests with the assertion's own message.
It becomes reachable the moment a second program can be loaded.

## The compiler assertion Step 4 asked for was deliberately not written

Step 4 asks that "no `Clause` op precedes a `Generic` op" be asserted in the compiler and not only
in prose. It was not written, and the omission was not reported at the time, which it should have
been.

Writing it here would be a check that cannot fail: `compile` emits `Op::Generic` for every
instruction and constructs `Op::Clause` nowhere, so any assertion over the emitted stream is
quantified over an empty set of `Clause` ops. It becomes a real check at the first task that emits
one. The reason the rule exists is in `run_clause_ops`'s own doc comment, at the delegation it
constrains: `step_in_temps_frame` echoes the clause, and the echo is not idempotent.

## What could not be verified

* **The register region's reserve/truncate pair has no witness**, because `chunk.registers` is `0`.
  M10 above is the measurement.
* **`ChunkTooLarge` cannot be reached from a test.** Producing one needs a body whose op stream
  exceeds `u32::MAX`. The refusal *path* is exercised by M7 (a mutation that forces the `Err` arm),
  which is what showed the counter and its assertions have teeth; the width check itself is not run.
* **`Op::Clause` and `Op::EvalExpr`'s driver arms are unreachable** while nothing constructs either
  variant, so their `Loud` messages have never been produced.
* **The `ir_dual` sweep says nothing about correctness of either engine.** Stated at length above,
  and measured by M1.
* **`run_fragment` is untouched**, per the brief: a fragment does not compile to a chunk in this
  phase, so an `INTERPRET` fragment's own clauses run on the tree-walker under both engines. The
  enclosing body still runs as a chunk, so the composition is exercised by every corpus program with
  an `INTERPRET` in it, but the fragment's instructions themselves are never driven.
* **The engine-selection counter is per thread, which is what makes it robust to the default
  changing**, and the tests reach the interpreter through `execute` rather than `run_program` so
  they run on the thread that reads it. That thread is a `libtest` thread with its ordinary stack,
  not the 512 MiB one `run_program` sizes, so a program deep enough to need that stack cannot be
  counted this way.

## Fix round 1

Commit `777fa4a9`, read back with `git log`. What changed, and why each was wrong.

* **I1.** `every_population_is_present_and_non_empty` compared `populations()`'s names against a
  literal in the same file. A two-sided deletion -- the population and its name, one edit -- left it
  green with half the sweep gone. Replaced by two pins that both compare against something outside
  the file: `the_sweep_runs_every_ootest_suite_a_sibling_harness_runs` (against the suite roots the
  sibling harnesses in `tests/` name) and `every_population_the_tree_calls_for_is_present_and_non_
  empty` (against a requirement derived from the tree). `ootest` populations are now named for their
  suite directory so no mapping table sits between the two sides. Evidence: MA and MB above.
* **I2.** The `run_chunk` counter was process-wide, serialised by a `Mutex` that could not exclude
  the tests which never took it. At the task that flips the default engine, every test in the
  process would drive chunks and the two counting tests would read 7 and 28 against 3 and 0. Fixed
  by making the counter per thread and having the tests enter through `execute` (which runs the
  interpreter on the calling thread) rather than `run_program` (which spawns one). The `Mutex` is
  gone. Rehearsed rather than reasoned: MD flips the default and 654 of 655 tests pass, the one
  failure being the test that states the old default.
* **M1.** Three doc comments said `chunks_refused` counts bodies. It counts refusals: there is no
  negative cache, so a refused body is recompiled and recounted on every entry. Corrected in
  `Outcome`, on `Interp`, in `chunk_for`, on `Engine::Ir`, and in both harness messages.
* **M2.** The report's justification for `#[derive(Default)]` on `Engine` was false -- nothing
  called `Engine::default()`. The derive is removed; see the engine-selection section above.
* **M3.** Two doc comments named a `corpus.rs` test "of this name" that does not exist. The real one
  is `the_differential_reads_every_phase_subset_file`, now named.
* **M4.** Two comments counted `run_activation`'s production call sites, which `rust/CLAUDE.md`
  forbids as a mutable in-repo aggregate. Both now state the property -- entering an activation is
  what reaches the compiled stream -- with no count.
* **M5.** `run_chunk`'s doc said "a clause spans a run of ops" in the present tense, where one
  clause is currently one op. Both places now say what the shape is built to support and what an
  `Op::Generic` actually is.
* **The `debug_assert` the review asked for**, pinning the plan key against the chunk key, with its
  own mutation evidence. See the section above.
* **Step 4's compiler assertion** recorded as deliberately not written, with the reason. See the
  section above.
