# Phase 5d Task 1 — row ownership, and 5c's close

**BASE `28c898208`.** Tree held alone. Plan
`docs/superpowers/plans/2026-09-03-phase-5d.md`, spec
`docs/superpowers/specs/2026-09-03-phase-5d-foundation.md` (D75, D77).

**Tree hash before the first gate run: `148299cd876c6ab09104f2194ca4ff8f99427ced`
(`git write-tree` over the staged change; this report is added on top at commit
time and nothing reads it).**
**Tree hash after the last gate run: `148299cd876c6ab09104f2194ca4ff8f99427ced`
— unchanged.**

**A first gate run was started at tree `9ac482af63ec854c132643b2ddc47a3703c66b48`
and voided.** I edited a doc comment in `corpus.rs` while it was between gates,
which makes the whole run a measurement of no single tree. It was killed after
`G1 = 0` and `G2 = 0` and restarted from scratch at the tree above; every
status below is from the second run.

---

## What landed

### (a) Table C's method-row owner is now a property of the row's class

`corpus/docs/class-set.txt` gained a ninth column, `method-owner`, derived by
`rust/crates/rexx-extract/src/docs/classes.rs` (`METHOD_OWNER`,
`DEFAULT_METHOD_OWNER`, `STACK_FRAME_OWNER`, `method_owner`) and compared both
directions by `rust/crates/rexx-extract/tests/extract_docs.rs`. No value is
typed into the `.txt`: it was regenerated with

```
cargo run -p rexx-extract --bin rexx-extract-docs -- --oodocs ../oodocs --interpreter ../interpreter --out corpus/docs
```

Values: `File`, `Stream`, `StreamSupplier` → `7`; `Alarm`, `Ticker` → `6`;
`StackFrame` → `deferred-rexxcontext-stackframes`; everything else `5c`.
`class_rows` panics if `METHOD_OWNER` names a class the set does not carry, so
a typo cannot fall silently to the default.

`rust/crates/rexx-exec/tests/gate_table_c.rs`'s
`const METHOD_PHASE: &str = "5c"` is gone, replaced by `method_row_owner`.

### (b) Never-agrees is derived, not listed

`method_row_owner` answers `NEVER_AGREES` (`"never-expected-to-agree"`) for
`status == "unreachable" && arm == "instance"`, from two columns that already
exist. Not keyed on class: `Buffer~new` and `Pointer~new` are class-arm and
agree today, and the run shows them keeping owner `5c` rather than being filed
as impossible:

```
  Buffer    class     loud=no  5c                        1  row(s): agree=1        buffer__class.rex
  Pointer   class     loud=no  5c                        1  row(s): agree=1        pointer__class.rex
  Pointer   instance  loud=yes never-expected-to-agree   5  row(s): unanswered=5   pointer__instance.rex
```

### (c) Table D re-owned in `owning_phase`

`rust/crates/rexx-exec/tests/gate_table_d.rs`:
`("::REQUIRES", "LIBRARY") => Some("7")` and
`("::REQUIRES", "NAMESPACE") => Some("5d")`, above the `::OPTIONS | ::RESOURCE
| ::REQUIRES | ::ROUTINE` arm and beside the `("::ROUTINE", "EXTERNAL")`
precedent.

### (d) 5c closed, and the corpus-coverage gap closed

* `CLOSED_PHASES` is `["5a", "5b", "5c"]`
  (`rust/crates/rexx-exec/tests/gate_tables/mod.rs`).
* `rust/corpus/phase-5c.txt` created, wired into every subset list:
  `corpus.rs`, `coverage.rs`, `ir_dual.rs`, `collect_stress.rs` and
  `trace_oracle.rs`'s unguarded literal.
* `coverage.rs` gained `EXPECTED_SUBSET_5C` and
  `phase_5c_subset_matches_the_committed_list`.
* The three interim witness binaries are deleted:
  `rust/crates/rexx-exec/tests/variable_reference.rs`, `stem_object.rs`,
  `string_makearray.rs`.
* `rust/corpus/unfiled.txt` created (header only, no entries) and
  `corpus.rs`'s `every_lang_program_is_run_or_named_unfiled` reads it beside
  the subset union.

**`every_closed_phase_this_table_owns_rows_for_is_gated` still sees its
subject.** It now enumerates owners by mapping `method_row_owner` over the
committed row set (`method_row_owners()`) rather than reading a constant, and
asserts that set is non-empty before filtering. Control 1b below is the proof
it can still fail on a method-row owner.

---

## The numbers

**Table C**, `REXX_CORPUS_GATE=1` with no `REXX_PHASE_GATE`:

| owner | rows | not yet `agree` |
|---|---|---|
| `5a` | 135 | 0 |
| `5b` | 6 | 0 |
| `5c` | 1225 | 0 |
| `6` | 13 | 13 |
| `7` | 94 | 82 |
| `deferred-rexxcontext-stackframes` | 10 | 10 |
| `never-expected-to-agree` | 5 | 5 |

`gated by this run: 0 row(s)`. The method rows partition 1225 + 13 + 94 + 10 +
5 = 1347, and the open ones 13 + 82 + 10 + 5 = **110**. **The 110 did not
move**, and no phase in `CLOSED_PHASES` owns one of them.

**Table D**, same run:

| owner | rows | not yet `agree` |
|---|---|---|
| `5a` | 36 | 0 |
| `5b` | 2 | 0 |
| `5c` | 36 | 0 |
| `5d` | 1 | 1 |
| `7` | 2 | 2 |
| `deferred-parse-error-rendering` | 2 | 2 |

`gated by this run: 0 row(s)`. This is the plan's expected shape exactly.

**Corpus differential: a measured `354 of 354 matching`**, against the plan's
recorded `331 of 331` at BASE — I did not re-run the differential at BASE, so
the 331 is quoted, not measured here. The 23 newly filed programs all match,
which is why the two figures differ by exactly the 23.

---

## The corpus-coverage gap, and a plan figure corrected

342 `corpus/lang/*.rex` on disk, 319 filed, **23 unrun** — the plan's own
figure, confirmed. All 23 are filed now:

* 5c's three interim witnesses;
* the twelve `directive_options*` (see the stderr scope below);
* the eight that had been unrun for longer than 5c.

**The eight were each checked against the oracle before being filed**, one
program per run, oracle from `corpus/lang/` under
`( ulimit -v 1048576; LD_LIBRARY_PATH=… bin/rexx <abs> )`, crate from
`crates/rexx-exec/` under `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`,
three descriptors compared separately:

| program | oracle rc | ir rc | tree-walker rc | stdout | stderr | engines |
|---|---|---|---|---|---|---|
| `call_procedure` | 0 | 0 | 0 | same | same (empty) | agree |
| `condition_syntax` | 0 | 0 | 0 | same | same (empty) | agree |
| `do_variants` | 0 | 0 | 0 | same | same (empty) | agree |
| `gate_variants` | 0 | 0 | 0 | same (empty) | same (empty) | agree |
| `keyword_as_variable` | 0 | 0 | 0 | same | same (empty) | agree |
| `source_arg` | 0 | 0 | 0 | same | same (empty) | agree |
| `string_builtins` | 0 | 0 | 0 | same | same (empty) | agree |
| `whitespace_significant` | 0 | 0 | 0 | same | same (empty) | agree |

**None diverged, so nothing was held out and no investigation was opened.**

**`gate_variants.rex` is the weak one and it is named rather than glossed.**
Its oracle stdout is 0 bytes: everything with a runtime effect is behind `if 0
then` or in a routine or method nothing calls, because a gate driver loads it
as a package and would run its prolog. What its differential row still asserts
is that both interpreters *parse* it and *install* its `::annotate`, `::class`,
`::method` and `::routine` directives without either side raising — but it is
agreement over an empty stdout and should not be read as more.

**The plan's `334 of 334` was wrong and is corrected in the plan.** `334` is
`331` plus the three interim witnesses alone, which contradicts the same
task's instruction to file all 23. Measured: **354**. The plan
(`2026-09-03-phase-5d.md`) and the spec's criterion 8
(`2026-09-03-phase-5d-foundation.md`) both carry the correction with a note
saying what it replaced.

---

## The Deviation 7 stderr scope, carried across

`corpus.rs` compares a subset program's `stderr` as a **sequence**, so filing
`directive_options_trace_reply.rex` without carrying the multiset licence
across would have reintroduced that flake. Carried:

* `support/oracle.rs` gained `StderrComparison::Multiset` and
  `stderr_multiset`, the one implementation both harnesses sort with;
* `directive_options.rs`'s `stderr_for_comparison` now calls it, so its
  control `the_multiset_comparison_discards_ordering_and_nothing_else` is a
  control over the function `corpus.rs` uses;
* `corpus.rs` gained `CONCURRENTLY_TRACED` (that one program) and
  `stderr_mode`, which panics if a path is on both `RAW_STDERR_COMPARISON` and
  `CONCURRENTLY_TRACED`;
* `raw_stderr_comparison_only_names_programs_the_subset_actually_runs` is
  renamed `every_opted_in_stderr_comparison_names_a_program_the_subset_runs`
  and now holds both lists against the subset.

`docs/superpowers/plans/phase-4-exclusions.txt`'s Deviation 7 carries the
record: its "IT WAS NOT BUILT HERE" paragraph is replaced by what was built.

---

## Controls

Each prediction was written before the run.

### Control 1 — a class owner naming a closed phase reddens

**Predicted:** with `File`'s `method-owner` edited from `7` to `5b` (closed),
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c`
exits non-zero, the `5b` bucket grows by `File`'s rows with 50 open, and the
named gated probes are `gate-tables/methods/file__instance.rex`.

**Ran. CONFIRMED**, rc 101:

```
  5b: 66 rows, 50 not yet `agree`
gated by this run: 50 row(s) …
50 row(s) … the first of them are ["gate-tables/methods/file__instance.rex", …]
```

Reverted; `cmp` against the pre-control copy is byte-identical.

### Control 1b — the enumeration still sees a method-row owner

The check most at risk from replacing a constant with a column.

**Predicted:** with `File`'s owner set to `4a` — `corpus/phase-4a.txt` exists
and `"4a"` is not in `CLOSED_PHASES` —
`every_closed_phase_this_table_owns_rows_for_is_gated` fails naming `4a`, with
no environment variable set.

**Ran. CONFIRMED**, rc 101:

```
["4a"] own rows in gate table C and have a committed corpus subset file, …
but CLOSED_PHASES does not name them
```

Reverted.

### Control 2 — with 5c closed, `REXX_CORPUS_GATE=1` alone reddens a 5c row

**First attempt, FALSIFIED, and it is worth recording.** Deleting
`("String", "REVERSE", Arity::Fixed(0), native_reverse)` from
`dispatch.rs`'s `NATIVE_METHODS` moved **nothing**: rc 0, `gated by this run:
0`. The mutation was live — a probe run afterwards gives
`rexx-exec: method "REVERSE" of class "String" is not implemented (Phase 5)`
at rc 120 — so `hasMethod` answers `1` for a method with no body. That is
exactly the spec's premise (table C's method rows are `hasMethod` readbacks and
a hollow class scores full marks), met while trying to use it as a control.

**Second attempt, predicted:** adding `let answers = answers && name !=
"REVERSE";` to `native_has_method` makes `String reverse instance` — the only
`reverse` row in `class-methods.txt` — diverge on stdout, and
`REXX_CORPUS_GATE=1` **alone**, no `REXX_PHASE_GATE`, exits non-zero with one
gated row naming `gate-tables/methods/string__instance.rex`.

**Ran. CONFIRMED**, rc 101:

```
  5c: 1225 rows, 1 not yet `agree`
  String   instance  loud=no  5c   118  row(s): agree=117 diverge-stdout=1   string__instance.rex
      diverge-stdout reverse (fundclasses.xml:8582)
gated by this run: 1 row(s) …
```

Reverted; `cmp` against the pre-control copy is byte-identical.

### Control 3 — a never-agrees row is gated by neither disjunct

**Predicted:** under `REXX_CORPUS_GATE=1` with 5c closed and no
`REXX_PHASE_GATE`, the report shows `never-expected-to-agree: 5 rows, 5 not
yet agree`, `gated by this run: 0`, exit 0, and no assertion fires.

**Ran. CONFIRMED**, rc 0, mode line `verdicts gated for 5a, 5b, 5c`:

```
  Pointer   instance  loud=yes never-expected-to-agree   5  row(s): unanswered=5   pointer__instance.rex
  never-expected-to-agree: 5 rows, 5 not yet `agree`
gated by this run: 0 row(s) …
```

**Falsification, so the control can fail.** Predicted: naming the value in
`REXX_PHASE_GATE` makes the same five rows gate, proving they are visible to
the mechanism and excluded only because no phase name matches. Ran with
`REXX_PHASE_GATE=never-expected-to-agree REXX_CORPUS_GATE=1`: **CONFIRMED**,
rc 101, `gated by this run: 5 row(s)`, all
`gate-tables/methods/pointer__instance.rex`.

### Control 4 — the corpus-coverage check, all three directions

Not asked for; run because the check is new and its committed companion file
is empty.

| mutation | predicted | outcome |
|---|---|---|
| drop `lang/gate_variants.rex` from `phase-5c.txt` | red naming it as unfiled | **CONFIRMED**, rc 101, `{"lang/gate_variants.rex"} are in …/corpus/lang and named by no phase subset file` |
| name a filed program in `unfiled.txt` | red on "in both" | **CONFIRMED**, rc 101, `["lang/gate_variants.rex"] are named by a phase subset file and by unfiled.txt` |
| name a nonexistent path in `unfiled.txt` | red on "not a program in" | **CONFIRMED**, rc 101, `unfiled.txt names lang/no_such_program.rex, which is not a program in …` |

All three reverted; `cmp` against the pre-control copies is byte-identical.

---

## Gates

Run from `rust/`, each status read unpiped, appended to a file by the job and
read back in the turn this report was written.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

`memcap` was present (`command -v memcap` → `/home/moritz/.local/bin/memcap`).

**The phase gate, over both tables:**

| command | exit |
|---|---|
| `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |
| `REXX_PHASE_GATE=5d …` (same command) | **101** |

The `5d` run is red **by design at this task**: `gated by this run: 1 row(s)`,
`["gate-tables/directives/requires__namespace__subkeyword.rex"]`. That is the
one table row this phase moves and it is Task 5's. Gate 4 above already runs
both tables with `5c` closed and no `REXX_PHASE_GATE`, and reports
`gated by this run: 0 row(s)` on each — that is 5c's close.

Every figure quoted under "The numbers" is from gate 4's log, not from an
ad-hoc run: `354 of 354 matching` and both tables' per-owner tallies.

**Verified by running the flip, not by reasoning.** `git archive
148299cd876c6ab09104f2194ca4ff8f99427ced` extracted to
`/home/moritz/dev/repos/claude-build-scratch/task-1-5d/src2/` (real disk) with
its own `CARGO_TARGET_DIR`:

| # | command | exit |
|---|---|---|
| E1 | `cargo clippy --workspace --all-targets -- -D warnings`, **clean target directory** | **0** |
| E7 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| E8 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1` over tables C and D | **0** |

**Two earlier extract runs were rc 101 and neither was this change.** The first
(`E2`) failed 22 tests across seven binaries, every one of them reading
`ootest/` — a git-ignored SVN checkout a `git archive` cannot carry —
plus `rexx-bench-suite`, which looks for `target/release/rexx-run` **relative
to the crate root** and cannot see it through a custom `CARGO_TARGET_DIR`.
`corpus`, `coverage`, `collect_stress`, `gate_table_c` and `gate_table_d` all
passed in that run. Symlinking `ootest/` and `oodocs/` in left only the bench
binary (`E5`); symlinking `target` in fixed that too, and `E7` is the clean
run. Recorded because "the extract is red" and "the change is wrong" look
identical until the failures are read.

---

## What I did not do

* **I did not delete `crates/rexx-exec/tests/directive_options.rs`**, which
  both its own module doc and Deviation 7's text expected the 5c flip to
  delete. Its twelve programs are in `phase-5c.txt` now, but the binary still
  carries four things the differential does not: `stderr` compared **raw**
  rather than through Deviation 0, both engines rather than the default one,
  `CONCURRENTLY_TRACED`'s both-directions scope guard over the family's
  sources, and the `LOSTDIGITS` table, which drives programs of its own and
  has nothing to do with the corpus subset. Deleting it would have removed all
  four. Both docs are corrected to say so. **The plan does not name it among
  the binaries to delete** — it names the three interim witnesses, and those
  three are gone.
* **I did not create `corpus/phase-5d.txt`**, as instructed.
* **I did not make `REXX_PHASE_GATE=5d` green.** It is rc 101 with exactly one
  gated row, `requires__namespace__subkeyword`, which is Task 5's work and
  which D75 deliberately re-owned to `5d`. The plan's gate command is
  therefore red by design until Task 5 lands; `REXX_PHASE_GATE=5c` is the one
  this task closes and it is 0 on both tables.
* **I did not control `stderr_mode`'s both-lists panic.** It is an
  unreachable-today arm; nothing witnesses it firing.
* **`corpus/unfiled.txt` ships with no entries.** Its "in both lists" and
  "path does not exist" arms were controlled with temporary entries (control
  4) and reverted, so nothing in the committed tree exercises them.
* **I did not touch any of the 110 open table C rows**, and did not implement
  anything a `Family::Timer`, `Family::Stream`, `Family::File` or
  `Family::Queue` entry point needs (D77).
* **I did not read the benchmark axes.** Nothing in this change is on an
  execution path; the only crate-source change is
  `rexx-extract/src/docs/classes.rs`, which is a build-time extractor.
* **I did not re-run the eight long-unfiled programs more than once each.**
  All eight are rc 0 with empty stderr on both sides and no source of
  nondeterminism I could see, but that is one run each, not a stability
  measurement.

---

## Files

* `rust/crates/rexx-extract/src/docs/classes.rs`
* `rust/corpus/docs/class-set.txt` (generated)
* `rust/crates/rexx-exec/tests/gate_table_c.rs`
* `rust/crates/rexx-exec/tests/gate_table_d.rs`
* `rust/crates/rexx-exec/tests/gate_tables/mod.rs`
* `rust/crates/rexx-exec/tests/corpus.rs`
* `rust/crates/rexx-exec/tests/coverage.rs`
* `rust/crates/rexx-exec/tests/collect_stress.rs`
* `rust/crates/rexx-exec/tests/ir_dual.rs`
* `rust/crates/rexx-exec/tests/trace_oracle.rs`
* `rust/crates/rexx-exec/tests/directive_options.rs`
* `rust/crates/rexx-exec/tests/support/oracle.rs`
* `rust/corpus/phase-5c.txt` (new)
* `rust/corpus/unfiled.txt` (new)
* deleted: `rust/crates/rexx-exec/tests/variable_reference.rs`,
  `stem_object.rs`, `string_makearray.rs`
* `docs/superpowers/records/2026-09-03-phase-5d/task-1-report.md` (this file;
  the briefed `.superpowers/sdd/` path is git-ignored, and
  `docs/superpowers/records/` is where every earlier phase's task reports are
  committed)
* `docs/superpowers/plans/phase-4-exclusions.txt` (Deviation 7's record)
* `docs/superpowers/plans/2026-09-03-phase-5d.md` (corpus figure)
* `docs/superpowers/specs/2026-09-03-phase-5d-foundation.md` (criterion 8)

**Scratch left behind, not deleted:**
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task-1/`
(oracle runs of the eight, control logs, gate logs) and
`/home/moritz/dev/repos/claude-build-scratch/task-1-5d/` (on real disk: `src/`
is the voided extract of tree `9ac482af6`, `src2/` the extract of the committed
tree `148299cd8` with `ootest`, `oodocs` and `target` symlinked in, and
`target/` their shared `CARGO_TARGET_DIR`).
