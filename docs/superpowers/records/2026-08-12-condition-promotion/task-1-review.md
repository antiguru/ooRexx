# Task 1 review: an `IF`'s condition compiles

Reviewed: `7ea946434` and `9d6e55bcb` (the later `cbdeaab71` is a plan-document
edit by someone else and is out of scope).

* **Spec compliance: PASS.**
* **Code quality: CHANGES REQUESTED.**

Nothing in the shipped behaviour is wrong. Every finding below is a comment
that claims a discrimination its own test or row cannot make -- the class the
brief names as this project's recurring one -- plus two stale premises the task
left behind in prose it did or did not edit.

---

## How this was verified

All runs were done in a detached `git worktree` at `9d6e55bcb` under the
session scratchpad, never in the repository tree. `ootest` is untracked in this
checkout, so it was symlinked into the worktree; without it 26 tests fail for
a reason that has nothing to do with this task.

**Gates, re-run rather than taken from the report:**

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings`, **clean `CARGO_TARGET_DIR`** | exit 0; `rexx-num`, `rexx-parse`, `rexx-core`, `rexx-exec`, `rexx-extract`, `rexx-oracle`, `rexx-bench` all in the `Checking` list; no `warning:` or `error` line in the output |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1465 passed, 0 failed, 4 ignored** -- the report's number exactly |

`const _: () = assert!(size_of::<Op>() == 16)` is unchanged in `ir/mod.rs` and
still holds: the workspace builds, and an `E0080` there is a compile error.

**Mutations.** Every run was the whole workspace under `memcap 8G cargo test
--workspace --no-fail-fast`. Files were backed up with `cp`, restored from the
backup, and the restore checked with `sha256sum`. Three runs were made with
`tests/ir_dual_cases/conditions` moved out of the directory entirely, which is
the "can fail is not adds coverage" step.

| id | mutation | conditions file | reddened |
|---|---|---|---|
| MXA | `eval_condition`'s `let checked = matches!(condition.kind, ExprKind::Logical(_))` -> `let checked = false` | present | **nothing**; 1465 passed, 0 failed |
| MXB | the fallback gains `Op::Condition` behind its `Op::EvalExpr` | present | `ir::golden_tests::a_condition_outside_the_native_set_stays_one_eval_expr`, **and nothing else** |
| MXC | the driver passes `checked: true` | **held out** | `both_engines_agree_on_every_branch_shape` |
| MXD | the driver reads `static_indent(...)` instead of the live indent | **held out** | `both_engines_agree_on_every_case_file`, on `trace-settings`' "a body entered under trace r that then turns trace off" |
| MXE | `chunk_node_at` loses its `If` arm | **held out** | `both_engines_agree_on_every_branch_shape` (row "a call on handler failing at an if's own boundary", `if raiser() = 'V'`), `both_engines_agree_on_every_case_file`, `both_engines_agree_across_every_population` (7 of 10391, `[keyword] IF::test_13`/`test_14` among them), `the_known_engine_divergences_still_diverge_exactly_as_recorded`, `the_exempt_set_matches_the_current_failures` |

A stale `target/debug/rexx-run` from the MXE build produced a spectacular false
divergence before I rebuilt it. Recorded here because it looked exactly like a
shipped bug for about ten minutes.

**Oracle captures, taken independently.** Four of the seven new rows (c4, c5,
c6, c7 -- the three-line ones and the two long transcripts) were re-captured
from a fresh empty directory under the standard wrapper. All four are
**byte-identical** to what the file commits, modulo the program path in c4's
middle error line.

**Ten extra three-way probes** (oracle vs tree-walker vs ir, stdout/stderr/exit
status as three descriptors), on shapes the new file does not hold: `trace r`
over a comma-list condition; a user routine call in a condition under `trace r`;
the same inside a controlled loop; `trace i` over `if 'x'`; `trace r` over
`if .nil then nop`; a prefix condition `if \0`; a 42.3 raised mid-condition
inside a loop; a compound read in a condition under `trace i`; a 34.1 at an
elevated indent; `SIGNAL ON SYNTAX` over a 34.1. **No divergence, on any
channel, on any of the ten.**

---

## The specific checks the brief asked for

**`Op::Condition` is emitted only where the condition went native.** Confirmed
statically -- `compile.rs`'s `If` arm is an `if native_shape(...) { push_native;
push Condition } else { push EvalExpr }`, `push_native` has no `Op::EvalExpr` in
any arm, and no other site in the crate constructs `Op::Condition`. Confirmed
dynamically by MXB.

**`checked: false` is licensed.** `native_shape` still has no `ExprKind::Logical`
arm; a comma list reaches `_ => false` and the whole slot stays on
`Op::EvalExpr`. The flag is separately observable on the driver side (the
report's M3, and my MXC): a native condition that read back an unvalidated value
would answer `if 'x'` false instead of raising 34.1.

**The split preserves the frame push/pop and the trace ordering.** Mechanically
diffed the old `eval_condition` body from `let value = self.eval(...)` onward
against the new `condition_value` body: **one line differs**, `if matches!(
condition.kind, ExprKind::Logical(_))` -> `if checked`. The `push_frame`,
`push_temp`, `to_text`, both `ConditionTrace` arms, the `pop_frame` and the
readback fork are byte-identical, so `WHILE`/`UNTIL`'s `ConditionTrace::Keyword`
path is unchanged by construction.

**`chunk_node_at`'s subset claim.** Its three slot arms (`Assignment` 0, `Say`
0, `If` 0) are exactly the slots `compile` enters `push_native` for (the two
`push_value` sites plus the new direct call), and are a strict subset of
`eval_chunk_expr`'s five arms. Both halves of the doc's claim hold.

---

## Findings, most severe first

### F1 -- `tests/ir_dual_cases/conditions` makes three discrimination claims its own rows cannot make, two of them contradicted by the implementer's own mutation table

All three are in prose that presents a row as load-bearing.

* Header, the refusals paragraph: *"That row is what says the fallback still
  does the whole job rather than half of it."* MXB is exactly "the fallback does
  half of it", and the 34.6 row stays **green** under it -- `eval_logical_list`
  raises during `eval`, so the extra `Op::Condition` is never reached. The only
  catcher is the golden test.
* Header, the trace paragraph: *"the indent is read live rather than compiled
  in, and the last row is what makes that observable."* MXD with the whole file
  held out still reddens, on a `trace-settings` stanza. c7 does not make it
  observable; it re-observes it.
* The 34.6 stanza's own comment: *"the sub-number is what says so, because a
  re-check on top of the list's own answer would report 34.1 here."* MXA forces
  the re-check and the **entire workspace stays green**. See F2.

Fix: state what each row is (a transcript, an adjacent shape) rather than what
it uniquely catches, or delete the rows -- with the plan's Step 10 corrected in
the same change, as the report already says.

### F2 -- the comma-list "re-checking would misreport 34.6 as 34.1" mechanism is false, and the diff edited the sentence carrying it rather than correcting it

`eval_logical_list` validates every element and raises 34.6 **inside `eval`**,
before `condition_value` is entered; when it returns, its result is
`self.text(b"1")` or `self.text(b"0")` unconditionally. So a list that reaches
`condition_value` always holds exactly `"0"` or `"1"`, `logical_value` accepts
both, and `raise` can never fire for it. Measured: MXA, whole workspace green
with `checked` forced to `false`.

The claim appears three times in the tree, and the diff touched all three
neighbourhoods:

* `run.rs`, `eval_condition`'s doc -- pre-existing, and **edited by this diff**
  into "so it reaches `Interp::condition_value` already `checked`, and
  re-checking its result there would misreport that failure as 34.1/34.2".
  A false sentence that is rewritten and left false is worse than one nobody
  touched.
* `run.rs`, `condition_value`'s new doc -- "`Interp::eval_condition`'s own doc
  comment has which shape that is and what re-checking it would misreport",
  which imports it by reference.
* `tests/ir_dual_cases/conditions`, the 34.6 stanza -- F1(c).

The `checked` parameter itself should stay: the driver's `false` is load-bearing
(MXC). What has to go is the justification for the tree-walker's `true`, which
is behaviourally dead. Either say that plainly ("a list's result is already
exactly `0`/`1`, so the readback and the check agree; the flag exists for the
compiled caller, which has no such guarantee"), or drop the `matches!` and pass
`false` from both callers.

### F3 -- "the oracle differential cannot see it" is false for everything except the indent width

`conditions`' header: *"Trace, which is where a promoted expression diverges
first and where the oracle differential cannot see it, because
`tests/support/mod.rs` normalises the region these lines sit in ... so its
presence, its position among the operand and operator lines, and its indent are
all this file's to hold."*

DEVIATION 0 collapses **only** the run of spaces between a trace line's 3-byte
marker and its content. That module's own doc says so: "the presence, absence
and order of every line in `stderr`" are untouched. And
`tests/trace_oracle/controlled_loop.expected` is an oracle capture of `trace r`
over `if ii = 2 then iterate`, so an `IF` condition's `>>>` -- its presence and
its position -- is already pinned against the C++ interpreter directly. Only
the indent **width** is this file's alone.

### F4 -- a premise falsified in a file the task did not edit

`lib.rs`, `Loud::register_not_logical`: *"The only writer of such a register is
an `Op::EvalExpr` whose expression `eval_condition` has already validated as
exactly `0` or `1`."* The identical sentence is on `Interp::register_holds` in
`drive.rs`, which the task **did** edit.

`Op::Condition` is now a second writer of a register `Op::JumpUnless` reads.
(The sentence was already wrong -- `Op::WhenTest` writes one too, and has since
the `SELECT` promotion -- so this task compounded a pre-existing falsehood
rather than creating it. It is still the shape the plan declares as its failure
mode, and `drive.rs` was open in the same commit.)

### F5 -- `ir/mod.rs`'s `Op::Const` doc now enumerates the emission sites wrongly, and the enumeration is a premise for the measurement under it

*"A literal in an assignment's value position or a `SAY`'s expression position
is what emits it, and no registered benchmark axis has one **inside a measured
loop**: counting executions on each axis at `n` and at `2n` ..."*

An `IF`'s condition is now a third position -- `if 'x' then nop` compiles to
`Op::Const` + `Op::TraceLiteral` + `Op::Condition`. The counts survive (I
checked: no `bench-programs/*.rex` contains an `IF` at all), but the sentence
that licenses counting only two positions is false, and the next person to
re-count will count the wrong set.

### F6 -- `root_of`'s doc is falsified by this task's own change in the same file

`corpus_shape_tests.rs`: *"The op the value expression of a promoted
`Assignment` or `SAY` must end in."* This commit's `check_body` change adds
`InstructionKind::If { condition, .. } => Some(condition)`, so `root_of` is now
called for an `IF`'s condition as well. The neighbouring `Root::of` doc and the
`check_body` comment were both corrected; this one was missed.

### F7 -- "both are released at the `IF`'s own clause end" misplaces one of the two releases

`golden_tests.rs`, `an_if_with_an_else_compiles_to_a_clause_region_and_two_jumps`:
*"the comparison's right operand takes one of its own beside the register the
condition lands in, and both are released at the `IF`'s own clause end, so each
promoted `SAY` below gets register 0 back."*

`push_native`'s `Binary` arm releases the right operand's register immediately
after `Op::TraceOperator`, and carries its own comment saying why ("Held until
the operation has run: the right operand's register is live right up to it").
By the time the `If` arm's `registers.release(mark)` runs, `next` is already 1
and only register 0 is released there. The sentence is offered as the mechanism
for the reuse it explains, and it names the wrong site for half of it.

### F8 -- `eval_if_condition`'s doc no longer describes both engines

*"**The one implementation, entered from both engines**: `step`'s `If` arm calls
it inside its own `in_clause`, and `Op::EvalExpr` calls it for the compiled
form's `Clause` region."* For a native condition the compiled engine never
enters this function; it reaches `condition_value` directly. True now only for
the shapes `native_shape` declines, and the doc does not say so. `run.rs` is a
file this commit edited.

### F9 -- `push_native`'s `unreachable!` message names the wrong entry point

`unreachable!("push_value descends only into an expression native_shape
accepted")`. The `If` arm now enters `push_native` without going through
`push_value`. The invariant still holds (the arm checks `native_shape` first);
the attribution in the message does not.

### F10 (nit) -- a set's cardinality in a comment

`conditions`' header: "Three kinds of row hold that split." `rust/CLAUDE.md`
forbids naming a set's size; the three names that follow are the set.

---

## What is right, and worth saying so

* `a_condition_outside_the_native_set_stays_one_eval_expr` earns its place, and
  its own doc's "reddens this test and nothing else" is **verified** by MXB. It
  is the one new test in the diff that a mutation shows to be load-bearing.
* The `Seen::native_conditions` assertion is a real anti-vacuity guard and its
  comment describes it accurately (it guards the corpus, not the compiler, and
  does not claim otherwise).
* The `corpus_shape_tests.rs` scope addition beyond the brief's file list is
  justified: `Root::of` lives there (the brief said `compile.rs`), it is
  exhaustive with no catch-all, and extending `check_body` rather than
  narrowing its comment is the right direction.
* The oracle captures are genuine. Four re-taken independently, byte-identical.
* The rename is clean: no live reference to `nested_ifs_reuse_one_register`
  remains anywhere outside the SDD history, and the one doc reference in
  `a_select_cases_own_value_outlives_the_registers_its_whens_take` was updated.

---

## The four concerns

**1. Most of the new case file duplicates existing witnesses.** Upheld, and
every measurement in it reproduced. With the whole file held out: MXC still
reddens on `BRANCH_CASES`' `if 'x' then say 'y'` (c3 adds nothing), MXD still
reddens on `trace-settings` (c7 adds nothing), MXE still reddens on five tests
including `BRANCH_CASES`' `if raiser() = 'V'` and 7 of 10391 extracted `IF`
programs (c6 adds nothing). c1/c2 are the same shape as four `BRANCH_CASES`
rows; c5 is the same shape as a `trace-settings` stanza. My reading: **keep c4
and c6, drop c1/c2/c3/c5, and correct the plan's Step 10 in the same change.**
c4 is the only comma-list condition run on both engines anywhere in the tree,
and c6 is the only committed transcript of a `trace i` condition that is an
operator above a call -- a transcript row is a different instrument from a
pass/fail row, which is the counter-consideration the brief raises, and it is
the one that survives the measurements. What must go regardless is the prose
that made the redundant rows sound load-bearing (F1).

**2. No row covers a traced declining condition.** Upheld and material as a
single-instrument gap, not as a defect. MXB reddens exactly one test and it is
an op-stream test, so the doubled `>>>` is pinned by what `compile` emitted and
not by what running it prints. I ran `trace r` over `if .nil then nop` three
ways: the engines agree with each other and with the oracle, so nothing is
broken today. Adding that row would put the failure mode in front of the
dual-engine comparison as well, and is cheap.

**3. The `checked` flag has no observable difference on the comma-list side.**
Upheld, and stronger than the report puts it. The pre-existing `eval_condition`
comment is **false**, measured: MXA forces the re-check and the whole workspace
stays green. The diff does not merely fail to contradict it -- it **edits that
sentence and keeps the claim**, imports it by reference into `condition_value`'s
new doc, and restates it in the new case file's 34.6 stanza. See F1(c) and F2.

**4. The rename.** Clean. Nothing outside the SDD ledger referenced the old
name, and the one in-tree reference was updated. No action.
