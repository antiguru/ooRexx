# Task 2 review: a `DO`/`LOOP` header's values compile

Reviewed: `b8e9db0e5`, one commit, against `.superpowers/sdd/2026-08-12-condition-promotion/task-2-brief.md` and `task-2-report.md`.

* **Spec compliance: PASS.**
* **Code quality: CHANGES REQUESTED.**

The shipped behaviour is correct. The register hazard the dispatch named is
genuinely absent, the `>K>` order is preserved and I re-measured it against the
oracle, and every gate reproduces. What is wrong is prose: the report's and the
plan document's central measurement about the new case file is false in every
clause and is falsified by three mutations I ran, the new `debug_assert`'s
message attaches a consequence to the direction that cannot produce it, and the
one doc comment that states the cause of the divergence this task found was left
asserting the opposite.

---

## How this was verified

All runs were in a detached `git worktree` at `b8e9db0e5` under the session
scratchpad, never in the repository tree. `ootest` is untracked in this checkout
and was symlinked in. Sources were backed up with `cp`, restored from the copy,
and the restore verified with `git status --short` (clean but for the `ootest`
symlink) after every mutation; `rexx-run` was rebuilt after the last restore and
the oracle probes re-run against the rebuilt binary.

**Gates, re-run rather than taken from the report:**

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings`, **clean `CARGO_TARGET_DIR`** | exit 0; `rexx-num`, `rexx-parse`, `rexx-core`, `rexx-exec`, `rexx-extract`, `rexx-oracle`, `rexx-bench` all in the `Checking` list; no `warning:` or `error` line anywhere in the output |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1468 passed, 0 failed, 4 ignored** -- the report's numbers exactly |

`const _: () = assert!(size_of::<Op>() == 16)` is unchanged at `ir/mod.rs:79`
and holds: the workspace builds, and an `E0080` there is a compile error.

**Oracle captures, taken independently**, from a fresh empty directory, each
program under `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib
.../build/bin/rexx /abs/path/FILE </dev/null )` with stdout, stderr and status
as separate descriptors, then each program on each engine and diffed four ways.
Five programs: the three the case file commits, plus `trace r` over the
`DO OVER ... FOR` and `do qq over 4.5 for 2` under `trace i`.

* The three committed rows' expected blocks are **byte-identical** to what I
  captured from this crate, on both channels.
* The `to`/`by`/`for` row and the call-bound row are **byte-identical to the
  oracle** as well.
* The `DO OVER ... FOR` row differs from the oracle by exactly one line, on both
  engines, under `trace i` and under `trace r`, on both programs. Reproduced.

**Mutations.** Six, all bounded. Each is named where it is used below.

---

## The specific checks the brief asked for

**The register hazard: the landed code is safe, and I checked it two ways.**
`push_native` is register-neutral by construction -- its `Binary` arm is the only
one that allocates, it takes its own mark immediately before `alloc` and releases
to it after `Op::TraceOperator`, and no other arm touches the allocator. So the
top after a slot is the top before it, and the header's `dst` registers, which
are allocated in the enclosing scope and released past the `END`, are never below
anything `push_native` hands out. Confirmed dynamically as well (see F2 and F4).

**Per-slot decisions are genuinely independent.** The `match` is inside the
per-slot loop and shares nothing across iterations that could carry a decision
(a declining slot pushes one `Op::EvalExpr` and allocates nothing).
`a_header_slot_outside_the_native_set_leaves_the_other_slots_native` pins the
declining-first direction from the stream; the native-first direction is the same
code path and is not pinned, which is fine.

**The `>K>` echo order per header value is unchanged.** Structurally the echo
still sits between the slot's value and its `Op::LoopHeaderValue`; the new
`assert_keyword_echoes_precede_their_value` makes the second half of that an
assertion; the golden streams pin it; and my own oracle captures of `trace i`
over `do zi = 1 to zn by 2 for 2` and `do zi = 1 to length(zs)` match the C++
interpreter byte for byte, with each value's own intermediate lines in front of
its `>K>` and the next value evaluated only after. This is the property most at
risk and it is the one best covered.

**`chunk_node_at`'s new arm and its subset claim.** The arm resolves through
`loop_header_slot(body, u32::from(slot))`, which is the same function
`eval_chunk_expr`'s `Do`/`Loop` arm and `ir::compile`'s header arm read -- one
resolution, three readers, as the reworded `loop_header_slot` doc says. The
subset claim survives: `chunk_node_at`'s arms are `Assignment` 0, `Say` 0, `If`
0 and `Do`/`Loop` any-slot; `eval_chunk_expr`'s are those four plus
`Select` 0. Strict subset, and the four are exactly the slots `compile` enters
`push_native` for.

---

## Findings, most severe first

### F1 -- the report's and the plan's measurement of the new case file is false in every clause, and three mutations falsify its conclusion

The report (Concern 2) and `docs/superpowers/plans/2026-08-12-condition-promotion.md`
Step 4 both say:

> the case-file harness's own catcher is a `loop-header-boundaries` row every
> time -- that file is alphabetically first and `datadriven` stops at the first
> mismatch, so these rows are not even reached under those mutations.

and the report concludes:

> I found no mutation that any row of `loop-header-values` uniquely catches, and
> **none that it catches at all**.

Every part of the mechanism is wrong, and the conclusion is wrong with it.

* **There is no sort.** `datadriven::walk` -> `test_files` builds its list from
  `fs::read_dir` and never sorts. Order is filesystem order. In this checkout
  `read_dir` returns `loop-header-values` *before* `loop-header-boundaries`; in
  the repository tree it returns them the other way round. The observation was
  an artifact of one directory's inode order.
* **`walk` does not stop at the first mismatch across files.** It runs every
  file and accumulates one failure per file, panicking at the end with all of
  them. Only `TestFile::run_normal` breaks, and only within one file.
* **Neither mechanism is what stops this harness anyway.** The engine-vs-engine
  comparison is an `assert_eq!` *inside* the callback (`ir_dual.rs:946`), so the
  first disagreeing stanza panics out of `walk` entirely, before datadriven ever
  compares anything to an expected block.

Measured, with **every other case file moved out of the directory** so that
`loop-header-values` is the only thing the harness can see:

| mutation | `both_engines_agree_on_every_case_file`, file alone |
|---|---|
| MU2 -- `chunk_node_at` loses its `Do`/`Loop` arm (this task's own Step 2) | **FAILED**, stdout `"1\n2\n"` vs `""` -- the call-bound row |
| MU4 -- the header's call op addressed at slot `0` | **FAILED**, same row |
| MU6 -- the `>K>` echo emitted in front of the slot's ops | **FAILED**, the `to`/`by`/`for` row, on stderr |
| MU1 -- the header arm never takes the native path | ok (correct: the promotion's contract is that the bytes do not move) |

And with the whole directory present, under MU2, the reported catcher **is** a
`loop-header-values` row: the only row anywhere in `ir_dual_cases` whose stdout
is `1\n2\n` is this file's call-bound row.

So the file has measured catching power for the two mutations that hit this
task's own code, and for the echo-order mutation the brief names as the property
most at risk -- including on the row the report singles out as "the weakest, and
if one row goes it is this one". The honest statement is *caught, but not
uniquely*: MU2, MU4 and MU6 each redden pre-existing tests as well. "None that it
catches at all" is not that statement, and it is the sentence that licensed
calling the file inert.

Fix: correct the report's Concern 2, and correct plan Step 4 -- the false
sentence is committed there, and the plan is what the next fix round regenerates
a brief from.

### F2 -- the new `debug_assert_eq!`'s message and its doc name a consequence the condition cannot produce, and the direction it can actually fire in is behaviourally inert

`compile.rs`, after the header loop:

```
// ... an operand register `push_native` took for a slot is released inside
// that call, so nothing a body clause is handed later can be one the running
// loop still reads.
debug_assert_eq!(
    registers.mark().0,
    header_top.0,
    "a header slot left a register allocated above the header's own, and the \
     loop's body would be handed it while the loop is still running"
);
```

A register *left allocated* above the header's own is precisely one the body is
**not** handed -- the body's clauses mark and allocate above it. The consequence
in the message belongs to the opposite inequality (a slot releasing *below*
`header_top`), which is the harmful direction and which `push_native` cannot
produce, because it releases to a mark it took itself. The `assert_eq!` is right
and worth keeping -- it is an equality, so it does guard the harmful direction
against a future change -- but as things stand the only way it can fire is the
harmless one.

Measured: leaking one register per header slot, with the assertion removed,
leaves the **entire `ir_dual` suite green**, population sweep included (9 tests,
0 failures). Six golden register pins move and nothing else. A leak is waste, not
corruption, and the sentence "so nothing a body clause is handed later can be one
the running loop still reads" does not follow from what is asserted.

Fix: say what the equality is for on each side -- above `header_top` is a
register nothing will reuse until the `END`, below it is a header value handed to
the body while `LoopState` still reads it.

### F3 -- `HeaderRole::keyword()`'s own doc still asserts what this task measured false, two lines under the variant doc that was corrected

`run.rs`:

```
/// The `>K>` tag this value's own echo carries, or `None` for the roles
/// the oracle echoes nothing for.
pub(crate) fn keyword(self) -> Option<&'static str> {
    match self {
        HeaderRole::Initial | HeaderRole::OverFor => None,
```

`OverFor` answers `None` and the oracle echoes `>K>   "FOR" => "1"` for it --
reproduced here on both engines under `trace i` and `trace r`. This is the exact
function the report, the commit message and the corrected `OverFor` variant doc
all name as the cause of the divergence, and it is the one place that still says
the opposite. The correction round fixed the two comments it went looking for and
missed the sentence between them.

Fix: `None` is the tag this table withholds; `Initial` is measured to match the
oracle and `OverFor` is measured not to, and the variant doc already says which
is which.

### F4 -- nothing behavioural in the tree catches the header-register hazard, in either direction

The brief asks whether anything would catch the hazard if the code were not safe.
Measured, by releasing the header's registers at the region's end instead of past
the `END` -- the exact hazard the enclosing-scope allocation exists to prevent,
a body clause handed a register the running loop still reads:

* `cargo test -p rexx-exec --test ir_dual --no-fail-fast`: **9 passed, 0 failed**,
  including `both_engines_agree_across_every_population`.
* `--lib`: exactly two tests move -- the pre-existing nested-loop register test,
  and this task's new `a_header_operands_register_goes_back_to_the_body`.

So the new golden test **is** load-bearing -- it is one of only two things in the
tree that see the real hazard, and it also catches the leak direction. Worth
saying plainly, because it is the opposite of the report's own verdict on its
new tests. But the answer to the brief's question is that the whole discipline
rests on compile-time register pins and the new `debug_assert`; the differential
harness cannot see it, because a `DO OVER` target's `ObjRef` is only reachable
through the register on the compiled engine and nothing in the corpus collects
while a loop is running. Not this task's to close; worth recording.

### F5 -- "reddens thirteen tests" is a count of a mutable in-repo aggregate, in a comment

`assert_keyword_echoes_precede_their_value`'s doc: *"Emitting the echo in front of
the slot's own ops instead of behind them reddens thirteen tests with this check
removed"*. I ran that mutation over the whole workspace and the count is thirteen
today, which is exactly the problem `rust/CLAUDE.md` describes: it is the size of
a set, it is a gate total, and the next task that adds a loop test falsifies it.
The same sentence is in the commit message, where it cannot be edited.

Fix: name what the failure looks like without counting it -- the population and
case-file sweeps, `trace_oracle`'s control-variable transcripts and the pinned
streams all move, because the echo reads a register nothing has written.

### F6 -- the new golden test's doc gives a mechanism that contradicts itself

`a_header_operands_register_goes_back_to_the_body`: *"a header temporary that
outlived its slot would push that assignment to register 3 **and be overwritten
by it**"*. If the assignment is pushed to register 3, the temporary at register 2
is exactly what is *not* overwritten. The two halves are the two opposite
failures and the sentence runs them together. The test itself is right and, per
F4, is one of the two that catch the real hazard -- so this is the mechanism
sentence, not the pin.

### F7 -- the brief's `trace r` requirement was dropped without being recorded as dropped

Step 4 as dispatched: *"Oracle-captured, under `trace r` and `trace i`"*. All
three landed rows are `trace i`. The plan rewrite carefully names the two rows
that were dropped and why (the `DO FOREVER`, the plain counted loop merged into
the `to`/`by`/`for` row) and says nothing about this one. `trace r` is a distinct
instrument here -- it prints the `>K>` lines and none of the intermediate lines,
so it isolates the echo order from the value's own ops -- and the coverage that
exists for it is `loop-header-boundaries`', not this task's. Cheap to add; what
matters more is that a requirement was rewritten out rather than reported unmet.

### F8 -- the divergence is recorded in a doc comment and a case-file row, not in the gated ledger

`docs/superpowers/plans/phase-4-exclusions.txt` is where this project's oracle
deviations live and is what `run.rs`'s own `loop_header_plan` doc cites for
Deviation 1. The `DO OVER ... FOR` gap is in none of it. By that file's own rule
a row is a plan amendment rather than a file edit, so **not** editing it was
right; what is missing is the amendment being asked for. Leaving the behaviour
unfixed was the right call -- it is one line, it moves the output of every
`DO OVER ... FOR` on both engines, and it deserves its own oracle-gated task --
but "pinned by a transcript" is a weaker guarantee than the ledger the project
otherwise gates on.

### F9 (nit) -- a set's size in an edited comment

`a_counted_loop_compiles_its_header_to_a_clause_region_and_its_body_to_generic`:
*"Three instructions -- `DO`, `nop`, `END`"*. The sentence was edited in this
commit (the "six ops of the seven" clause was correctly removed from it) and the
count at its head was kept. Pre-existing, but it was in the hunk.

### F10 (nit) -- ragged wrap left by an edit

`ir/mod.rs`, `Op::TraceKeyword`: *"...waited for the trace ops.** A / loop header
interleaves..."* -- the shortened opening left a two-word line. `cargo fmt` does
not reflow doc comments, so it stays.

---

## The five concerns, adjudicated

**1. The pre-existing `DO OVER ... FOR` oracle divergence.** Upheld, reproduced
independently, and the corrections are honest. My own captures: the oracle prints
`>K>   "FOR" => "1"` for `do qq over zs for 1` and `>K>   "FOR" => "2"` for
`do qq over 4.5 for 2`; this crate prints nothing, on both engines, under
`trace i` and under `trace r`; the two engines agree with each other everywhere.
Both corrected doc comments now say what is true, and the committed row's own
comment states plainly that its expected block is not the oracle's. Leaving it
unfixed was right -- one line, both engines, every `DO OVER ... FOR`, and no
oracle gate in this task's scope. The gap in the correction is F3, and the ledger
question is F8.

**2. The new case file has no measured catching power.** **Rejected, on
measurement.** The `datadriven` mechanism is misdescribed on all three points
(no sort, all files walked, and the stop here is a panic in the callback), and
the file alone reddens the case-file test under MU2, MU4 and MU6 -- see F1. The
retained rows earn their place on evidence rather than on the transcript
argument: the call row catches this task's own Step 2 being removed, and the
`to`/`by`/`for` row -- the one the report offers to drop -- catches the echo-order
mutation. The `DO OVER ... FOR` row is the only record of that gap anywhere in
the tree, which I verified: no other test or case file in the crate runs a
`DO OVER ... FOR` under trace at all.

**3. MU3 did not terminate; MU4 is the substitute.** Upheld as stated, and the
report is honest that MU4 covers only the address half. But "could not be
completed" overstates it: the population sweep is one test, and a bounded run was
available. I ran MU3 (`loop_header_slot(body_node, 0)`) against the lib tests and
three named `ir_dual` tests, skipping the sweep: **9 lib tests** redden --
including both new header golden tests, the corpus shape sweep and four `ir::drive` tests -- plus `both_engines_agree_on_every_case_file`,
`..._loop_shape` and `..._branch_shape`. The expression half is well covered; the
answer was one flag away.

**4. `Op::TraceKeyword`'s missing adjacency assertion is not a coverage gap.**
Upheld. `Op::TraceKeyword` is indeed the one echo op with no adjacency assertion
and is in the `None` group of `assert_region_ops_name_their_clause`; the sibling
assertions do look one place back and that shape is genuinely unavailable now;
and the pairing the new assertion states is the right invariant. The reasoning
that it adds failure *shape* rather than coverage is correct and is measured --
MU6 reddens the tree without it. The two rows its doc cites in
`loop-header-boundaries` (`do i = 1 to 'a' by 2` and `do i = 1 to 'a' by zf()`)
both exist and say what the doc claims. The defect is F5, the count.

**5. `push_value` would have emitted an identical stream.** **Verified true.**
`push_value` is `if native_shape(expr, ROOT) { push_native(..., ROOT, dst) } else
{ ops.push(EvalExpr { index, slot, dst }) }`, which for any slot that has an
expression is exactly the landed arm, argument for argument. The only slot it
cannot serve is one with no expression, which is why the direct form collapses
two push sites into one -- and that is what the landed comment says ("they are one
arm because they are one answer"), without repeating Task 1's `Op::Condition`
reason. Task 1's reason genuinely does not transfer: an `IF`'s fallback
`Op::EvalExpr` does the validation and the `>>>` itself, a header slot's does
not and still owes its `Op::LoopHeaderValue`. Correct claim, correct code,
correct comment.

---

## What is right, and worth saying so

* The register discipline is sound, and `a_header_operands_register_goes_back_to_the_body`
  is the only new test in the diff that a mutation shows to be load-bearing for
  something nothing else pins (F4). It catches both directions.
* The `>K>` order is preserved and I confirmed it against the oracle, not only
  against the tree.
* `assert_keyword_echoes_precede_their_value` states a real invariant, in the
  only direction still available, and its doc is honest that it buys failure
  shape rather than coverage.
* `Seen::native_header_values` is a real anti-vacuity guard and its comment
  describes what it guards (the corpus) without overclaiming.
* Extending `check_body` rather than narrowing its comment is the right
  direction, and the `loop_header_slot` dependency it introduces is disclosed
  with the right half named as still independent (`root_of`).
* The three case-file rows are genuine oracle captures: byte-identical to my own,
  and two of them byte-identical to the oracle.
* The divergence was found by this task's own captures, disclosed rather than
  papered over, and the two doc comments that had asserted the opposite were
  corrected -- one of them in a golden test's doc, where it would have been
  easiest to leave.
