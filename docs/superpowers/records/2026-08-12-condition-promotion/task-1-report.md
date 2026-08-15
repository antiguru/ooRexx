# Task 1 report: an `IF`'s condition compiles

Status: DONE_WITH_CONCERNS.
Two commits, both on `plan/rust-rewrite`.

## Commits

* `7ea946434` -- Split a condition at the evaluation, so the tail is one function (Steps 1-3).
* `9d6e55bcb` -- Compile an IF's condition instead of evaluating it whole (Steps 4-12).

## What changed, and why

### `run.rs`: the split (Step 1)

`eval_condition` did two things. It now does one and hands the rest to
`condition_value`, which takes an `ObjRef` and asks nothing about where it came
from. The frame push, the `to_text`, the `ConditionTrace` match and the
readback/validate fork all moved across unchanged.

The comma-list question travels as a `bool`. `eval_condition` computes it from
`condition.kind` exactly as the old code did at the bottom of the function;
`condition_value` reads the answer back rather than re-deciding it.

`ConditionTrace` became `pub(crate)` because it is now in a `pub(crate)`
signature -- `clippy::private_interfaces` refused the intermediate state, which
is how the need was found rather than guessed.
`raised_if_not_logical` became `pub(crate)` for the driver's call.

### `ir/mod.rs`: `Op::Condition { index: u32, reg: u16 }` (Step 4)

Beside `Op::JumpUnless`. The doc states what it replaces, that it reads and
writes one register, that `native_shape` has no `ExprKind::Logical` arm and
that this is what licenses `checked: false`, and that it is only valid inside a
`Clause` region.

`const _: () = assert!(size_of::<Op>() == 16)` still holds: `cargo build
--workspace --all-targets` was run **unpiped** and its whole diagnostic list
read -- no `E0080`, and the build finished. The variant is `u32 + u16`, smaller
than `Op::CallExpr`'s existing five fields.

### `ir/compile.rs`: the `If` arm (Step 5)

`native_shape` + `push_native` directly, not `push_value`, and the arm's
comment says why: the fallback is not the same op. A condition that declines
stays one `Op::EvalExpr` doing the whole job through `eval_chunk_expr`'s `If`
arm, so a `Condition` behind it would trace and validate twice. That is not a
hypothetical -- see mutation M2 below.

`assert_region_ops_name_their_clause`'s exhaustive match gained
`Op::Condition { index, .. }` in the `Some(*index)` group.

**The five adjacency assertions.** None needed a change and none broke. Each
scans forward for an *echo* op and looks one place **back** for the computing op
whose register and tag it repeats. `Op::Condition` is not an echo op, so no
scan selects it; and it sits *after* the last echo, so it is never the op an
echo looks back at. Nothing was added for it, per the dispatch. The check is not
an argument: all five run unconditionally inside `compile`, so the corpus shape
sweep and every golden test exercise them on every compiled body, and the suite
is green.

### `ir/drive.rs`: the region arm and the not-driven arm (Step 6)

The region arm reads the register, calls `condition_value` with
`ConditionTrace::Result(indent)`, `checked: false` and `raised_if_not_logical`,
and writes the logical value back into the same register. `indent` is read live
from `clause_state.current_value_indent`.

The op does not jump: it falls through to the next op in the `for region_op in
ops` loop, like `Op::Arith`. The outer loop's list gained
`Op::Condition { .. } => Loud::op_not_driven("Condition")`.

### `run.rs`: `chunk_node_at` (Step 7)

`(InstructionKind::If { condition, .. }, 0) => condition`, so `Op::CallExpr`'s
descent resolves for `if length(zs) > 3 then`. Without it every condition
holding a call reaches `Loud::call_op_off_its_node` -- measured, mutation M6.

The doc comment above it stated the *reason* an `If` was excluded ("an `If`'s
condition is validated to `0`/`1` ... the op behind this would take the value as
it comes and lose that"). That reason is now false -- the validation is a
separate op -- so it was rewritten rather than left. The subset relationship it
asserts survives, because `eval_chunk_expr` keeps its own `If` arm.

### `ir/golden.rs`, `ir/golden_tests.rs` (Steps 8-9)

Render arm: `Condition index=N reg=R`.

Four pinned `IF` streams moved, and their doc comments moved with them:

* `an_if_with_an_else_compiles_to_a_clause_region_and_two_jumps` -- "the region
  is exactly ... ops" prose replaced with what the region now *is*; the jump
  targets 10/15/8 became 16/21/14; `op_of[3]` 8-not-10 became 14-not-16.
* `an_if_with_no_else_emits_no_branch_end_jump` -- `EndBranch` at 8 became 14,
  `JumpUnless` to 9 became 15.
* `nested_ifs_reuse_one_register` -- **renamed** to
  `nested_ifs_reuse_their_registers`. `1 = 1` takes two registers now (the
  right operand takes one of its own), so the old name and the old
  "under a monotonic counter the inner `IF` would take register 1" were both
  falsified. The property the test is about -- the inner `IF` reuses what the
  outer released, so the high-water mark grows with depth and not with length
  -- is unchanged. The one reference to the old name, in
  `a_select_cases_own_value_outlives_the_registers_its_whens_take`'s doc, was
  updated.
* `a_traced_if_carries_its_clause_echo_as_an_op_of_the_region` -- "ops 5 to 9"
  became "ops 11 to 15"; "an echo emitted after the `EvalExpr`" became "after
  the condition's ops"; and the sentence about what follows the `*-*` line now
  names the operand and operator lines the condition produces as well as
  the `>>>`.

`compile.rs`'s own `If` arm comment said "the region is exactly two ops", which
this change falsifies. Corrected.

New test, per Step 9's last line:
`a_condition_outside_the_native_set_stays_one_eval_expr` (`if .nil then nop`),
which renders `EvalExpr` and no `Condition`.

### `ir/corpus_shape_tests.rs`

Not in the brief's file list, but required by it: `Root::of` is an exhaustive
match with no catch-all, so `Op::Condition` had to be classified. It is `None`
-- it is not a `push_native` arm and not an op an expression's own value can end
in. That made two neighbouring sentences loose ("an op that produces no value"
is not what `None` means there, since `Op::WhenTest` also writes a register),
so both were corrected.

Two further changes, and the second is why the first is worth having:

* `check_body`'s value match gained `InstructionKind::If { condition, .. } =>
  Some(condition)`. Its comment said "the two constructs that have one that can
  compile natively", which this change falsifies; extending the check is the
  correction that keeps the file's claim true rather than narrowing it.
  It works because `Op::Condition` and `Op::JumpUnless` are both `None` for
  `Root::of`, so the existing scan back finds the expression's own root op.
* `Seen::native_conditions`, asserted non-zero in `sweep_every_corpus_body`.
  Without it the new `If` row is satisfied vacuously by a *corpus* whose every
  condition is outside the native set: `root_of` would call for
  `Root::EvalExpr`, the stream would hold one, and the row would pass with the
  promotion never having fired. (It does **not** guard against the *compiler*
  ceasing to promote -- `root_of` is independent of `compile`, so that case
  reddens the row itself. Measured: M1.)

### `tests/ir_dual_cases/conditions` (Step 10)

Seven stanzas, in the six kinds the brief lists. Every expected block was
captured from the oracle first and then confirmed byte-identical on both
engines.

## Test counts

| point | passed | failed | ignored | cargo exit |
|---|---:|---:|---:|---:|
| before any edit | 1464 | 0 | 4 | 0 |
| after the split (Step 2) | 1464 | 0 | 4 | 0 |
| final | 1465 | 0 | 4 | 0 |

The one new run is `a_condition_outside_the_native_set_stays_one_eval_expr`.
The seven new `ir_dual` stanzas run inside `both_engines_agree_on_every_case_file`,
which is one test either way, so they add no run count.

Gates, from `rust/`:

* `cargo fmt --all --check` -- exit 0, read unpiped.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0, read
  unpiped from a status variable, not from a pipeline.
* The same clippy from a **clean** `CARGO_TARGET_DIR` under the scratchpad, per
  `rust/CLAUDE.md`'s rule that a same-session green is provisional -- exit 0,
  with `rexx-num`, `rexx-parse`, `rexx-core` and `rexx-exec` all in its
  `Checking` list and no `warning:` or `error` line anywhere in its output.
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0.

## Oracle captures

All seven programs were written into a fresh directory I `mkdir`ed
(`$SCRATCH/probe-conditions`), with absolute paths for every redirect, and run
with stdout, stderr and exit status captured as three separate descriptors.
None of the three crashing programs was run.

The invocation, once per program, exactly as the Global constraints give it:

```
( cd $SP && ulimit -v 1048576; \
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx $SP/cN.rex \
  </dev/null >$SP/cN.out 2>$SP/cN.err )
```

Then, from `rust/`, each program on each engine:

```
REXX_ENGINE=tree-walker ./target/debug/rexx-run $SP/cN.rex </dev/null >... 2>...
REXX_ENGINE=ir          ./target/debug/rexx-run $SP/cN.rex </dev/null >... 2>...
```

and four diffs per program: tree-walker vs ir on stdout and on stderr, and the
oracle vs the tree-walker on stdout and on stderr. The stderr comparison against
the oracle substitutes the program's own path on both sides, because a raised
condition's middle line prints it and the two runs are at different paths;
nothing else is normalised. **All 28 diffs were empty.**

| # | program | rc | what it holds |
|---|---|---:|---|
| c1 | `if 1 = 1 then say 'taken'` / `else say 'not taken'` | 0 | a condition that holds |
| c2 | `if 1 = 2 ...` | 0 | one that does not |
| c3 | `if 'x' then nop` | 222 | 34.1, message naming the IF keyword |
| c4 | `if 1, 'x' then nop` | 222 | 34.6, the comma list's own raiser |
| c5 | `trace r` over `if zn > 3` | 0 | the `>>>` line and its indent |
| c6 | `trace i` over `if length(zs) > 3` | 0 | `>V> >A> >F> >L> >O> >>>`, in that order |
| c7 | `trace r`, an `IF` in a routine called from a loop | 0 | the live indent, both passes |

c6 is the row that exercises Step 7: `length(zs)` takes an `Op::CallExpr`
addressed at `root.L`, which only resolves because `chunk_node_at` gained its
`If` arm.

## Mutations

Every run was the **whole workspace** under `memcap 8G cargo test --workspace
--no-fail-fast`, so "nothing else caught it" is measured rather than inferred
from a run that stopped at the first catcher. Files were backed up with `cp`,
restored from the backup, and the restore verified with `sha256sum -c`. No
`git checkout --` was used.

| id | mutation | tests reddened |
|---|---|---|
| M1 | the `If` arm never takes the native path (`if false && native_shape(...)`) | `corpus_shape_tests::every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops`, and the four golden `IF` streams |
| M2 | the **fallback** gains an `Op::Condition` behind its `EvalExpr` | `golden_tests::a_condition_outside_the_native_set_stays_one_eval_expr`, **and nothing else** |
| M3 | the driver passes `checked: true` | `ir_dual::both_engines_agree_on_every_branch_shape`, `ir_dual::both_engines_agree_on_every_case_file` |
| M4 | the driver computes the indent statically instead of live (`static_indent(&code.body.instructions, index)`) | `ir_dual::both_engines_agree_on_every_case_file` only |
| M5 | the driver drops the write-back of the logical value | 14 tests, across `ir_dual`, `ir::drive::tests`, `run::tests`, `trace_oracle` and the exempt-set table |
| M6 | `chunk_node_at` loses its `If` arm | `ir_dual::both_engines_agree_across_every_population`, `..._on_every_branch_shape`, `..._on_every_case_file`, `the_known_engine_divergences_still_diverge_exactly_as_recorded`, `the_exempt_set_matches_the_current_failures` |

What that says about the new tests, stated as measurement rather than as a
coverage count:

* **M1**: the corpus sweep's new `If` row does redden -- `root_of` is computed
  from the parse tree and still calls for `Root::Binary` while the stream ends
  in `Root::EvalExpr`. It is not the only catcher, though: the four golden
  streams redden too, and they are the more direct signal. For M1 the corpus
  file adds nothing, exactly as its own module doc records for the `CALL name`
  mutation. The `native_conditions` assertion did **not** fire under M1, because
  `check_body` panics before the sweep reaches it -- it guards a different case
  (a corpus with no native-shaped condition in it).
* **M2**: the new golden test is the **only** catcher in the workspace. This is
  the mutation the "use `native_shape`+`push_native`, not `push_value`"
  instruction is about, and without that test the double validation and the
  doubled `>>>` ship green. That measurement is now in the test's own doc, in
  the form `corpus_shape_tests.rs`'s module doc already uses.
* **M3**: the new case file's 34.1 stanza (c3) catches it -- and so does
  `BRANCH_CASES`' pre-existing "if condition is not a logical value" row, whose
  program is `if 'x' then say 'y'`. So c3 adds nothing over what was already
  there.
* **M4**: `both_engines_agree_on_every_case_file` is the only catcher, but the
  new file is **not** why. Re-run with `tests/ir_dual_cases/conditions` held out
  of the directory entirely, M4 still reddens, on a stanza in
  `tests/ir_dual_cases/trace-settings` (an `IF` inside a called label under
  `trace r`). So c7's live-indent axis was already witnessed.

I did not find a mutation that only `tests/ir_dual_cases/conditions` catches.
See Concerns.

## Concerns

1. **Most of the new case file duplicates witnesses that already existed, and I
   left it whole anyway.** c1/c2 are the same shape as four `BRANCH_CASES` rows;
   c3 was measured (M3) to be caught by a `BRANCH_CASES` row with a
   near-identical program; c5 is the same shape as a `trace-settings` stanza;
   c7's axis was measured (M4) to be already covered by `trace-settings`. Only
   c4 (34.6 -- no other dual-engine witness for a comma-list condition exists in
   the tree) and c6 (`trace i` over an operator above a call, which is Step 7's
   own shape) look unique, and I did not find a mutation that proves even those.
   `rust/CLAUDE.md`'s "can fail is not adds coverage" rule argues for deleting
   the redundant rows; the brief's Step 10 names all six kinds explicitly. I
   kept them, because trimming three of six mandated rows on my own judgement --
   with no mutation showing them worthless, only some showing other catchers
   exist -- is a call for whoever owns the plan, and the measurement is here to
   make it with. **If they go, Step 10 in
   `docs/superpowers/plans/2026-08-12-condition-promotion.md` should be
   corrected in the same change**, or the next fix round regenerates the brief
   and asks for them again.
2. **No case-file row covers a *traced declining* condition.** M2 is caught only
   by a golden op-stream test, which pins what `compile` emitted and not what
   running it prints. A row like `trace r` over `if .nil then nop` would put the
   doubled `>>>` in front of the dual-engine comparison too. It is not in the
   brief's list and I did not add it.
3. **The `checked` flag has no observable difference that I could construct.**
   M3 (`checked: true` in the driver) reddens, but only because a *native*
   condition then reads back a value nothing validated -- `if 'x'` answers false
   instead of raising 34.1. The comma-list direction the brief describes
   ("re-checking it would report 34.6 as 34.1") I could not reach: for a comma
   list, `eval_logical_list` raises 34.6 *during* `eval`, before
   `condition_value` is entered at all, so a list that gets as far as the
   readback always holds exactly `"0"` or `"1"` and re-checking it would agree.
   I have written no comment claiming otherwise -- `eval_condition`'s doc says a
   list "reaches `condition_value` already `checked`" and that re-checking
   *there* would misreport, which is the mechanism and not a claim that the flag
   is observable today. Worth a second opinion.
4. `nested_ifs_reuse_one_register` was renamed. Nothing outside
   `golden_tests.rs` referenced it (checked across `rust/` and `docs/`), but a
   rename is the kind of thing a later grep expects to find.

---

# Fix round 1

Against `.superpowers/sdd/2026-08-12-condition-promotion/task-1-review.md`. Spec
passed; every finding was a comment claiming a discrimination its evidence does
not support, or a premise this task falsified.

## The case file, and the plan's Step 10

Per the coordinator's ruling on concern 1: **c1, c2, c3, c5 and c7 deleted**,
c4 and c6 kept. (The ruling named c1/c2/c3/c5; c7 went with them, because the
review had measured it to add nothing -- with the whole file held out, its
mutation still reddened on a `trace-settings` stanza.) Per concern 2: a declining condition under trace added -- **two** rows,
not one, and the second is why (below).

`tests/ir_dual_cases/conditions` now holds four stanzas: the 34.6 comma list,
`trace i` over an operator above a call, `trace r` over `if .nil then nop`, and
`trace r` over `if 1, 1 then nop`.

Both new rows were captured from the oracle in a fresh `mkdir`ed directory under
the standard wrapper, then run on both engines. Four diffs each (tw vs ir on
stdout and stderr, oracle vs tw on stdout and stderr, with only the program's
own path normalised): **all eight empty**.

* `trace r` over `if .nil then nop` -- rc 222. It traces `>>>   "The NIL
  object"`, re-echoes the clause and raises 34.1. The row the coordinator asked
  for.
* `trace r` over `if 1, 1 then nop` -- rc 0, three `>>>` lines (one per element,
  one for the list's own result), all emitted inside `eval_logical_list`.

**The second row exists because the first cannot do the job the gap named.**
`if .nil` *refuses*: the fallback's `Op::EvalExpr` raises 34.1 before a second
validating op behind it could run, so the doubled `>>>` never appears. A comma
list is the declining shape that succeeds. Measured -- see R2MXB.

`docs/superpowers/plans/2026-08-12-condition-promotion.md` Step 10 was rewritten
in the same commit: the list now names what the tree holds, and a paragraph
under it says which four rows went, what was held out and re-run to decide each,
and that the `trace r` rows replaced them for a gap the dispatched list did not
cover.

## Mutations run this round

Whole workspace, `memcap 8G cargo test --workspace --no-fail-fast`, backup by
`cp`, restore verified with `sha256sum -c`, and `rexx-run` **rebuilt** after each
restore -- the review lost ten minutes to a stale binary and I did not want to
repeat it.

| id | mutation | reddened |
|---|---|---|
| R2MXB | the fallback gains `Op::Condition` behind its `Op::EvalExpr` | `ir::golden_tests::a_condition_outside_the_native_set_stays_one_eval_expr` **and** `ir_dual::both_engines_agree_on_every_case_file`, on the `if 1, 1` stanza -- the ir arm prints a fourth `>>>` |
| R2MXA | `eval_condition`'s `let checked = matches!(...)` -> `let checked = false` | **nothing**; 1465 passed, 0 failed, cargo exit 0 |
| R2MXC | the driver passes `checked: true` | not a suite run: `if 'x' then nop` on the ir engine exits **0** and falls through, against **222** on the tree-walker and 222 on both once restored |

R2MXB is the measurement behind the new `if 1, 1` row's comment and behind the
plan's sentence about why the `trace r` rows replaced the four that went. The
gap concern 2 named is now closed by an output instrument as well as by the
op-stream test.

R2MXA is the measurement behind `condition_value`'s doc: the tree-walker's
`true` is a spared check, not a different answer. R2MXC is the measurement
behind the same doc's claim that the *driver's* `false` is load-bearing.

## Findings, one by one

**F1 -- three discrimination claims in the case file.** All three prose passages
are gone with the rows they described. The header no longer says any row is what
catches anything; it says what the rows are (transcripts, compared byte for
byte) and which shapes they hold. The one discrimination claim that remains --
on the `if 1, 1` row -- is R2MXB and says what reddened.

The header also no longer says what `BRANCH_CASES`, `trace-settings` or
`trace_oracle` hold. A first draft of it did, as the justification for the
shrunken scope; that is a claim about other files in this repository, which
`rust/CLAUDE.md` puts in the "delete it or assert it" class, and it would rot the
moment a row moved. It lives in the plan and in this report instead, where it is
history rather than a comment nobody rereads.

**F2 -- the false 34.6 re-check mechanism, in all three places.**

* `run.rs`, `eval_condition`'s doc: rewritten. It now says `eval_logical_list`
  raises 34.6 during `eval` and otherwise answers `b"0"` or `b"1"` **and nothing
  else**, so the `checked` it hands on spares a check that could not have failed
  rather than one that would have raised the wrong number. Verified by reading
  the function: the `Ok` path is `self.text(if holds { b"1" } else { b"0" })`,
  unconditionally. The oracle measurement in the old sentence survives, restated
  as what it actually distinguishes -- *where* the raise happened: `if 'x', 1
  then` is 34.6 from inside the list, `if 'x' then` is 34.1 from the keyword.
* `run.rs`, `condition_value`'s doc: the import-by-reference is gone. It now
  states the split the reviewer asked for -- the flag decides an answer for the
  compiled caller (R2MXC) and spares a check for the tree-walker (R2MXA) -- with
  both measurements named.
* The case file's 34.6 stanza: the "a re-check would report 34.1 here" sentence
  is gone. The stanza now says which raiser fires and that the whole condition
  stays one `Op::EvalExpr`.

The `checked` parameter stays, per the reviewer's own reading.

**F3 -- "the oracle differential cannot see it".** Deleted. The header makes no
claim about what any other harness can or cannot see.

**F4 -- `register_not_logical` and `register_holds`.** Both rewritten to name the
mechanism instead of enumerating writers: a register a branch op reads is
written by the op that decided the branch, always as the small integer of the
`bool` it answered. That covers `Op::EvalExpr`, `Op::WhenTest` and
`Op::Condition` without listing them, and it is what `register_holds` actually
decodes.

**F5 -- `Op::Const`'s emission sites.** The enumeration is replaced by the
mechanism: `push_native`'s literal arm is what emits it, so a literal in any
slot `compile` offers that function compiles to one. The per-program execution
counts under it are untouched.

**F6 -- `root_of`'s doc.** Now "the op the expression `check_body` hands this
must end in", which does not enumerate the constructs.

**F7 -- the two releases.** Corrected: the right operand's register goes back
inside `push_native`'s own binary arm as soon as the operation has run; the
condition's is what the `IF`'s clause end releases.

**F8 -- `eval_if_condition`'s doc.** Now says it is the tree-walker's entry, and
the compiled stream's only for a condition `native_shape` declined, and names
where a condition that compiled goes instead.

**F9 -- the `unreachable!` message.** Now "this descends only into an expression
native_shape accepted", which attributes the invariant to the descent rather
than to one of its two entry points.

**F10 -- the cardinality nit.** Gone with the header rewrite.

## Gates, re-run after every edit

From `rust/`, each status read unpiped:

* `cargo fmt --all --check` -- exit 0
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0, **1465 passed, 0
  failed, 4 ignored**

Unchanged from `9d6e55bcb`: four case-file stanzas replace seven, and neither
count is a test count -- they all run inside `both_engines_agree_on_every_case_file`.

## Not fixed

Nothing. Every finding is addressed, and every sentence written this round that
claims a discrimination has a mutation behind it named in the table above.

---

# Fix round 2

Against `.superpowers/sdd/2026-08-12-condition-promotion/task-1-re-review.md`.
All prose. No behaviour changed, and the working tree at the end differs from
`0459167cc` in four files, none of them `compile.rs` or `drive.rs`.

The round's governing rule, from the dispatch: where a claim is not
load-bearing, **delete** it rather than rewrite it. Two of the seven were fixed
by deletion for that reason.

## N1 -- a measured uniqueness claim that this task's own later hunk falsified

`golden_tests.rs`'s `a_condition_outside_the_native_set_stays_one_eval_expr`
carried "reddens this test and nothing else. Measured 2026-08-12, the whole
workspace under `--no-fail-fast`". That was true when measured, at a tree
without the `if 1, 1` row -- and `0459167cc` added that row, which is a second
catcher, and my own table in this file records both. **Deleted**, not rewritten:
the paragraph was the only thing in the doc claiming uniqueness, and replacing
it with a list of the two catchers would put a cross-file aggregate into a
comment, which is the thing `rust/CLAUDE.md` says to delete or assert.

The other side, in the case file, is weakened from "this row is what catches" to
"this row catches", which is a claim about that row alone and does not go stale
when another catcher appears.

**The rule I should have followed, and now have**: a "nothing else catches this"
claim is measured at a tree, not at a commit, so it has to be re-measured after
the last edit of the commit that carries it. Every discrimination claim left in
this task's prose was re-measured at the final state of this round -- R3MXB
below.

## N2 -- "all from inside `eval`"

The `if 1, 1` stanza said its three `>>>` lines were all emitted inside `eval`.
They are not: `eval_logical_list` calls `trace_result` once per element it
evaluates, and the list's own result line comes from `condition_value`'s
`ConditionTrace::Result` -- which is exactly why a second `Op::Condition` behind
the fallback prints a **fourth** one. Corrected to say which line comes from
which.

## N3 -- "the whole line sequence comes from the one `Op::EvalExpr`"

The `if .nil` stanza. Of the recorded `err>` lines, the `EvalExpr` produces the
`>>>` and the raise; the first `*-*` is `Op::TraceClause`'s and the second `*-*`
and both `Error` lines are `Raised::report`'s. Rewritten to the plan's own
wording, which claims only what the `EvalExpr` still owes.

## N4 -- "measured out rather than argued out"

The plan's Step 10 paragraph claimed a method for all the dropped rows that was
used on two of them, and counted kinds ("four rows") where five stanzas went.
Rewritten to separate the two grounds explicitly: the 34.1 and live-indent rows
were measured out (held out, mutation re-run, still caught), and the plain
true, plain false and `trace r` rows were argued out by shape with no mutation
of their own. It now says five stanzas.

`0459167cc`'s commit message carries the wrong version and cannot be edited, so
the plan paragraph says which claim is in it and that the paragraph is the
correction.

## N5 -- the fourth copy of the false 34.6 mechanism

`run.rs`, `if_condition_that_is_a_comma_list_raises_34_6_not_34_1`'s doc: "--
re-checking `eval_logical_list`'s own result would misreport it". Same false
mechanism as F2, in a place the first review's "three times in the tree" did not
count. **Deleted** the clause; the sentence's own claim (a comma list is 34.6
whichever element fails, never 34.1) is true, oracle-measured and left alone.

## N6 -- "validates every element"

`eval_condition`'s doc. `eval_logical_list` breaks out of its loop on the first
element that is `0`, so a later element is never evaluated and never checked.
Corrected to "checks that each element it evaluates is exactly `0`/`1` ... stops
at the first that is `0`", with the oracle measurement inline: `if 0, 'x' then
nop` exits 0 and never evaluates `'x'`. **Measured myself this round**, on a
fresh empty directory under the standard wrapper: rc 0, stdout `fell through`,
stderr empty. The conclusion the doc draws is unaffected -- a list that returns
still returns exactly `b"0"` or `b"1"`.

## N7 -- the report's deletion list

Corrected above: c1, c2, c3, c5 **and c7** were deleted. The "now holds four
stanzas" sentence beside it was already right.

## The mutation, re-run at the final state of this commit

| id | mutation | reddened |
|---|---|---|
| R3MXB | the fallback gains `Op::Condition` behind its `Op::EvalExpr` | `ir::golden_tests::a_condition_outside_the_native_set_stays_one_eval_expr` **and** `ir_dual::both_engines_agree_on_every_case_file`, the latter on the `if 1, 1` stanza with the ir arm printing a fourth `>>>   "1"` |

Whole workspace, `memcap 8G cargo test --workspace --no-fail-fast`, `cp` backup,
restore verified with `sha256sum -c`, and `rexx-run` rebuilt and re-probed
afterwards (both engines back to five `>>>` lines on the `if 1, 1` program).

That run is the evidence for the one discrimination sentence this round
rewrote rather than deleted -- the `if 1, 1` stanza's -- and for the plan's
sentence about which of the two `trace r` rows carries an instrument. Two
catchers, which is why the golden test's uniqueness claim had to go rather than
be restated.

## Gates

From `rust/`, each status read unpiped, after the last edit:

* `cargo fmt --all --check` -- exit 0
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0, **1468 passed, 0
  failed, 4 ignored**

1468 is the baseline the `DO`/`LOOP` header task (`b8e9db0e5`) left; this round
adds and removes no test.

## Not fixed

Nothing. N1-N7 are all addressed.

---

# Fix round 3

Against `.superpowers/sdd/2026-08-12-condition-promotion/task-1-re-review-2.md`.
All prose; no `.rs` behaviour and no test changed. Per the dispatch, this round
**deletes** rather than repairs: four of the six edits remove a thought
entirely, and the two that rewrite remove a clause rather than replace it.

## NN1 -- a quotation attributed to a commit message that does not contain it

The plan's Step 10 closing sentence put "were measured out rather than argued
out" in quotation marks and attributed it to `0459167cc`'s message. That message
contains neither that phrase nor the second fragment; both are the plan's own
superseded wording. **The whole justification paragraph is deleted**, per the
ruling. Step 10 now lists what the file holds and points at this report for
which rows went and what decided each. It quotes nothing.

That paragraph is where all three rounds' plan-side findings came from. It is
gone rather than corrected a third time.

## NN5 -- the program name, and the plan's own copy of the false mechanism

* Step 10's bullet said `if 1, 1 then nop`; the stanza's program is `if 1, 1
  then say 'both'`, and the `say` produces two of the transcript's lines.
  Corrected in the bullet -- a reader regenerating the row from the plan needs
  the program that was captured.
* Step 1 carried "`eval_logical_list` has already validated every element, so
  its result is read back rather than re-checked, and re-checking it would
  report 34.6 as 34.1". Both halves deleted; the sentence now says only what
  `checked` means.

## NN2 -- a probe this tree had already retired for this exact distinction

`eval_condition`'s doc cited `if 0, 'x' then nop` exiting 0 as showing `'x'` is
never evaluated. `eval_logical_list`'s own doc records why that probe cannot
make that distinction -- a literal evaluates harmlessly, so skipping only the
*check* produces the same clean run -- and uses `(1/0)` instead. **The evidence
clause is deleted**, not re-cited: the plain statement "stops at the first that
is `0`" is checkable from `eval_logical_list` thirty lines away, which already
carries the discriminating probe and its `rc 214`.

I took that capture myself last round rather than trusting a handed-down number,
which was right, and took the retired probe, which was not. The rule I take from
it: when a doc near the one I am citing already records *which* probe
discriminates, that is the citation, and my own fresh capture of a weaker probe
does not replace it.

## NN3 -- a universal quantifier the same file refutes

The `if 1, 1` stanza said the list's result line comes from "the tail every
condition enters". A condition that raises during evaluation never reaches it,
and the file's own first stanza (`if 1, 'x'`) is exactly that. Now "the tail a
condition's value enters", which drops the quantifier and keeps what the
sentence is for.

## NN4 -- the fifth copy, and the closed set

`condition_value`'s `if checked` arm carried "`eval_logical_list` already
validated every element and answers exactly `b"0"`/`b"1"` (its own doc comment)"
-- the same falsity, with a parenthetical citing as authority a doc that says
the opposite. **Deleted whole**: the function's own doc comment already explains
why the readback is right, so the inline comment was redundant as well as false.

A sentence I wrote in round 1, thirty lines up, had the same defect and no
review named it: `condition_value`'s doc said a comma list's "own evaluation
already raised on any element that was not `0`/`1`". For `if 0, 'x'` nothing
raised and `'x'` is such an element. The quantifier is deleted; the guarantee
that survives is the return value, which is unconditional.

### The closed set

Searched at the **final state of this commit**, over `rust/crates`, `docs/` and
`.superpowers/`, for `already validated`, `validates/validated every element`,
`already checked`, `every element`, `re-check`/`recheck`.

**Live text carrying the claim -- all now corrected:**

| where | verdict |
|---|---|
| `run.rs`, `condition_value`'s `if checked` arm | **deleted** this round (NN4) |
| `run.rs`, `condition_value`'s doc, "raised on any element" | **deleted** this round (NN4, unnamed by any review) |
| `run.rs`, `eval_condition`'s doc | corrected round 2 (N6), evidence clause deleted this round (NN2) |
| `run.rs`, `if_condition_that_is_a_comma_list_raises_34_6_not_34_1`'s doc | clause deleted round 2 (N5) |
| `tests/ir_dual_cases/conditions`, the 34.6 stanza | deleted round 1 (F2) |
| `docs/.../2026-08-12-condition-promotion.md`, Step 1 | **deleted** this round (NN5) |
| `docs/.../2026-07-30-phase-4a-executor.md:739` | **deleted** this round -- see below |

**Live text near the claim that is true and was left alone:**

| where | verdict |
|---|---|
| `eval.rs`, `eval_logical_list`'s doc | the correct statement, and the source of the discriminating probe. Unchanged |
| `eval.rs:2250`, the short-circuit test's comment | describes a hypothetical mutant that evaluates every element; accurate |
| `run.rs:6903`, `eval_chunk_expr`'s doc | about the value an `EvalExpr` stores, which is validated whichever path produced it. True |
| `run.rs:7002`, `checked`'s parameter description | generic over "whatever produced `value`". True |
| `docs/.../2026-08-12-remaining-promotion-survey.md:26` | "stops at the first false; answers `1` when every element held". Accurate |

**One loose quantifier of the same shape, listed and not edited:**
`eval.rs:1090`, F4's note -- "every element traces its own `>>>`, under `TRACE R`
alone". Its subject is the trace *setting*, which is what it measures and gets
right; the quantifier is loose for a short-circuiting list, where an element
that is never evaluated never traces. Not the 34.6 claim, and correcting it
means writing a new sentence rather than removing one, which this round is not
for. Named here so it is not a sixth surprise.

**Historical records, not corrected:** the SDD reports, reviews and re-reviews
under `.superpowers/`, `task-1-brief.md`, and `vm-comparison.md:841`. The brief
regenerates from the plan, and the plan sentence it copied is deleted above.
`vm-comparison.md` reports what the brief said and is accurate as an
attribution; its design conclusion (a per-element validating op raises 34.6)
does not rest on the false half.

### The one edit outside the named items

`docs/superpowers/plans/2026-07-30-phase-4a-executor.md:739` is where this claim
originates -- Task 1's brief inherited it from there. It read "A comma list
raises **34.6** from inside `eval_logical_list`, which already does it, and
re-checking the result would replace 34.6 with 34.1." The trailing clause is
deleted. The instruction it sits in ("Do not check a comma-list condition
yourself") is complete and correct without it, and the paragraph's own
measurement is untouched. Leaving it is what would have produced a sixth
instance, which is the opposite of the closed set this round is for. Flagged
here because it is a different phase's plan and nobody asked for it.

## Gates

From `rust/`, each status read unpiped, after the last edit:

* `cargo fmt --all --check` -- exit 0
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0, **1468 passed, 0
  failed, 4 ignored**, the baseline unchanged

No mutation was re-run this round: nothing this round wrote makes a claim about
what a mutation reddens. The one such sentence in the tree -- the `if 1, 1`
stanza's -- is unchanged in substance and its measurement is round 2's R3MXB,
reproduced independently as R3B.

## Not fixed

Nothing from NN1-NN5. One adjacent loose quantifier (`eval.rs:1090`) is listed
above and deliberately left.
