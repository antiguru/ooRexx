# Review: Phase 4e Task 4b (`736bf080..8f542fe1`)

Reviewed at `992ade97`, whose only change over `8f542fe1` is plan prose, so the source under review is the tree as it stands.

## Verdicts

**Spec compliance: fails on one required property.**
Every listed deliverable is present and I verified each independently -- the op program counter, the absorption rule as one function both loops decide through, `Op::Generic`/`Loop`/`Clause` carrying their instruction index, `Op::EvalExpr`/`Jump`/`JumpUnless`, the register allocator implemented rather than redesigned, the compiler-side assertion proved to fire from a real compile, both `#[expect(dead_code)]` removals, and `ChunkTooLarge::what` keeping its `#[allow]`.
The property that fails is the one the brief named as the subtlest surface: a promoted clause does **not** discharge the clause unit's obligations exactly once.
Failure-site resolution for a `CALL ON` handler that fails at the promoted clause's own boundary is not discharged at all, which is an oracle-confirmed byte divergence between the two engines (finding C1).

**Task quality: good, with one blind spot and two false statements in the record.**
The work is rigorous where it looks: eleven mutations run with `--no-fail-fast`, restore-from-`cp` with `sha256sum -c`, a "can fail is not adds coverage" pass that concludes the new table adds nothing three of its own mutations did not already have, the split proposed rather than half-landed, a plan correction (7.3, not 93.4) written where the next reader sees it, and an honest "things I could not verify" section that names the unattributed 7%.
The blind spot is that the new sixteen-case table was built around branch *shapes* and never around the clause *boundary*, which is the surface the brief singled out -- so the one real defect is in the one place the task was told to press hardest.
The two false statements are the report's "the tree-walker arm runs identical code in all four binaries" (I1) and the plan's "the four `SELECT` shapes" (I3).

## Answer to question 1: did the promotion extract shared semantics?

**The decisions are extracted; the branch's execution is a second expression, necessarily, and it is the one part where the two forms are held together by an argument rather than by shared code.**
`if_targets` computes `false_target`/`resume` once for both engines, `eval_if_condition` is the one condition evaluation both call, `absorb` is the one range test both loops decide through, and `in_stepped_clause` is the one clause unit -- so no branch *selection* rule exists twice.
What does exist twice is "run the true branch and resume past the `ELSE`": the tree-walker runs it as a bounded sub-range and rewrites `Flow::Next` into `Goto(resume)`, the compiled form runs it in the enclosing range with a `JumpUnless` and a conditional branch-end `Jump`.
That is inherent to flattening and the implementer found and fixed the one place the two came apart (the two arrivals at an `ELSE`), but both live simultaneously in one run -- an `IF` inside a `SELECT`'s `WHEN` body still takes `step`'s `If` arm while a chunk is running, because `BodyEngine::Chunk` reaches only `run_loop`'s two `run_bounded` calls.

## Findings

### Critical

**C1. A promoted `IF` clause does not record its failure site when a `CALL ON` handler fails at its boundary -- divergent from the tree-walker and from the oracle.**

`ir/drive.rs:369` reads

```rust
let outcome = self.in_stepped_clause(code, index, instruction, source, |it| { ... })?;
```

The `?` discards the arm `step_in_temps_frame_with` deliberately keeps (`run.rs`, the `Err(failure) => { self.record_failure_site(...); Err(failure) }` arm, fix round 3's NEW-B).
`in_stepped_clause` records the site for a failure of the *work*; the outer `Err` is `in_clause`'s own, raised by `deliver_pending_trap`, and its attribution was left to the caller.
`Op::Generic` and `Op::Loop` go through `step_in_temps_frame`/`step_in_temps_frame_with` and keep it; `run_clause_region` is the one new caller and drops it.

Witness one, at the activation's own level:

```rexx
call on error name h
if sub() = 1 then say 'then'
say 'after'
exit
sub:
  raise error 5 return 1
h:
  zz = 1/0
  return
```

Oracle (`rc 214`) and `REXX_ENGINE=tree-walker` both print

```
     8 *-*   zz = 1/0
     2 *-* if sub() = 1
Error 42 running ... line 8:  Arithmetic overflow/underflow.
```

`REXX_ENGINE=ir` drops the `2 *-* if sub() = 1` line.

Witness two, inside a `DO` body, where the wrong clause is echoed rather than none -- the exact symptom NEW-B was measured against:

```rexx
call on error name h
do i = 1 to 1
  if sub() = 1 then say 'then'
end
...
```

tree-walker echoes `3 *-* if sub() = 1`, the IR engine echoes `2 *-* do i = 1 to 1`.

Control: the same program with the clause unpromoted (`zz = sub()` instead of the `IF`) agrees on both engines, so this is the `Clause` region path specifically and not a pre-existing IR-engine defect.

Fix, verified by patching and re-running both witnesses: replace the `?` with a match that calls `self.record_failure_site(code, index, source, instruction)` on the `Err` arm before propagating.
`step_in_temps_frame_with`'s own arm is the model, comment included.

Why nothing caught it: neither `BRANCH_CASES` nor `LOOP_CASES` pairs a promoted clause boundary with a failing `CALL ON` handler, and the corpus population sweep does not either.
A regression case belongs in `BRANCH_CASES` with the oracle bytes above.

### Important

**I1. The report's control reasoning rests on a false premise: the tree-walker arm does not run identical code in all four binaries.**

`parent` is `736bf080`, before this task, and this task changed the tree-walker arm:

* `run_bounded` was split, and its range test moved out of the loop into `absorb`;
* `step_in_temps_frame_with` now routes through the generic `in_stepped_clause` with a closure;
* `step`'s `If` arm now calls `if_targets`, which computes `skip_else` unconditionally where the old code computed it only inside `if holds`.

So the up-to-6% movement of the tree-walker arm between `parent` and the other three cannot be attributed to code layout alone, and the argument that dismisses a 7% signal as "barely above a 6% layout artifact" is unsound as stated.
It also supplies a mechanism for the implementer's own leading hypothesis about why the reverted control could not read zero: the driver rewrite touched the tree-walker's hot path, so no control that holds only the *engine selection* fixed can read zero.
Task 4b-M inherits this: its control has to hold the tree-walker arm's **source** fixed, not just its selection.

**I2. M10 and M11 survive on an unenforced compiler property, and nothing goes red when it stops holding.**

"Unobservable today" is **correct**, and I verified the mechanism rather than taking it: with the `Clause` arm's grant removed, `first_instruction_pending` survives the `IF`, the `THEN` marker's own `Generic` grants it one clause later, and `step` for the `THEN` marker -- which does not read the permission -- consumes it, so any `PROCEDURE` in the branch is still 17.1.
It does not stay true by construction.
The invariant it rests on is "every `Clause` region is immediately followed by a granting `Generic`", which is stated in a comment and asserted nowhere.
A flattened loop header (Task 4c) is the concrete break: its region would be followed by body ops running under `Granting::No`, so with M10 applied `first_instruction_pending` would survive the whole construct and a `PROCEDURE` after it would be wrongly permitted -- silently, because the surviving mutation has no witness to lose.
A `debug_assert` in `compile` tying the two together is one line and turns the documented survival into something a later task trips over rather than rediscovers.

**I3. The committed plan states a false count, of a mutable in-repo aggregate, in the sentence Task 4b' will read as its regression net.**

`docs/superpowers/plans/2026-08-09-phase-4e-ir.md:594`: "The four `SELECT` shapes it must keep are already in `BRANCH_CASES`".
`BRANCH_CASES` holds three: *select with otherwise, second when matches*, *select with no matching when*, *select case with no matching when*.
This is both wrong and the shape the task's own constraints forbid ("no counts of mutable in-repo aggregates in prose"); state the property, or assert it.

### Minor

**M1. `nested_ifs_reuse_one_register`'s stated reason for existing is false.**

Its doc comment says "sequential would also hold under a per-clause reset and nesting is what tells the two apart".
It does not: the outer `IF`'s register is released before the inner one is compiled, so a per-clause reset produces the identical stream.
Measured -- with `Registers::mark` forced to `Mark(0)`, which is exactly a per-clause reset, the whole `rexx-exec` lib suite stays green except `a_released_register_is_handed_out_again_and_a_nested_one_is_not`.
The test is fine and the unit test does the discriminating; the justification beside it is the "a test that cannot fail" shape this crate's own method warns about, written as if it could.

**M2. `BRANCH_CASES`'s doc comment overstates its own coverage.**

"An `IF` with and without an `ELSE`, on both paths" -- there is no no-`ELSE` true-path case.
That shape is covered only incidentally, by *if in a called label* and by `LOOP_CASES`' *simple block inside an if*.

**M3. `run_ops`'s `Op::Jump` arm bypasses `absorb`.**

`pc = *target; continue;` means a jump target outside `[start, end]` would be read as "the range completed" (`Flow::Next`) rather than escaping -- a silently wrong answer where every other exit from the loop is loud or absorbed.
Safe today only because a branch-end jump's target is inside the enclosing block by construction, which is the same unasserted parser-structure premise as the implementer's own "could not verify" item about `op_of` and range starts.
One `debug_assert` on the target covers both.

**M4. "Nothing in between reads it" is an unasserted in-repo exhaustiveness claim.**

I checked it and it holds: `Activation::pc` is read only by `run_activation`'s loop and `run_chunk_clauses`, and written only by `apply_flow`.
But the report equates the new staleness with the tree-walker's, and they are not the same size: the tree-walker leaves `pc` stale for the duration of one construct's step, the IR engine now never advances it at all through a straight-line body.
The claim is load-bearing for the whole op-counter design and lives in prose.

**M5. The activation-stack tripwire was weakened.**

`run_chunk_clauses`' `debug_assert_eq!(self.activations.len(), depth, ...)` used to run after every clause; it now runs only after an escaping flow, so it no longer covers a straight-line body.
`run_activation`'s equivalent still runs per clause, so the tree-walker kept what the IR arm lost.

## Things I verified rather than took

* **The compiler-side assertion fires on a real compiled stream, not only on a hand-built one.** With `Op::Clause`'s `end: at + 3` changed to `at + 4` -- a region that swallows the `THEN` marker's `Generic` -- `compile` panics with "a Clause region at op 0 holds an op that opens a clause of its own", from four golden tests. Restored and rebuilt.
* **The allocator test pins the property asked for.** With `release` also lowering the high-water mark, `the_high_water_mark_is_the_deepest_the_stack_reached` goes red, along with the `IF` clause-count test and all three `IF` goldens.
* **Both clause-count tripwires are derived, and unchanged for the right reason.** I derived them independently from the emitted streams: `do i = 1 to 3 / nop / end` is one `Loop` plus one body `Generic` per pass = 4; `do / nop / nop / end` is one `Loop` plus two = 3; the new `IF` test is 4 on each path (`Clause`, branch marker, branch body, the clause after the construct), and 2 under `Op::Generic` for `If`. All four match what is written down.
* **Gates.** `cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo test --workspace` 1364/0/4, `--release` 1364/0/4, `REXX_CORPUS_GATE=1` dev 1364/0/4. No em-dashes in the changed comments, no `unsafe`.
* **`in_clause` does not touch `current_value_indent`**, so moving that read inside the shared `eval_if_condition` is behaviour-preserving.
* **Four hand-written differential probes around the promoted `IF`** (`SELECT` containing an `IF`, `SIGNAL` out of a true branch, `LEAVE`/`ITERATE` through one, a `SIGNAL ON` trap fired from a branch, `TIME('L')` clock reuse, `PROCEDURE` after a branch) agree byte for byte on both engines.

## Candidate causes for the per-clause cost (for Task 4b-M, not measured here)

Ordered by how much of the hot path they touch.
Items 1, 6 and 7 change the **tree-walker** arm too, which is why no within-binary control could read zero.

1. **`absorb` moves a 64-byte value across a call boundary, per clause, on both arms.** Measured `size_of::<Flow>() == 64` and `size_of::<Absorbed>() == 64`. The range test used to be an inline match on a local in each loop; it is now an argument in and a return value out. `#[inline]` was measured at 0.2%, which is consistent with it already being inlined -- but the `Absorbed::Escaped(Flow)` round trip is a new value materialisation either way.
2. **Two bounds-checked `op_of` loads and one extra call frame added per `DO`-body pass**, not per clause. `run_bounded_from_chunk` does `chunk.op_at(start)` and `run_ops` then does `chunk.op_at(end)`; `run_bounded` is called once per pass from `run_repeating` and once from `LoopKind::Simple`. At `736bf080` the chunk arm did its `op_of` lookup only per clause, inside `step_from_chunk`, and nothing looked up `end` at all. `emptyloop` is 25e6 passes, so this is 50e6 loads and 25e6 call frames that did not exist -- the only candidate here that scales with the axis that moved most.
3. **`run_ops` takes eight arguments** where the old chunk arm's loop had five values in scope; `registers`, `chunk`, `at` and `granting` are newly threaded through a per-pass call.
4. **`granting.grants()` is a branch per clause** that did not exist.
5. **`BodyEngine` grew from 8 to 16 bytes** (measured) and is copied on every `step` -> `run_loop` -> `run_bounded` hop.
6. **`step_in_temps_frame_with` now calls the generic `in_stepped_clause` with a closure**, one more layer per clause on both arms.
7. **Tree-walker only, per `IF` executed:** `if_targets` computes `skip_else` on both paths where the old arm computed it only when the condition held.
