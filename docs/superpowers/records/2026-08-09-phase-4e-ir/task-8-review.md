# Task 8 review: promote variable access

Reviewed `938de575` and `0036ca7c` only.
`304236d6`, `bd61e489` and `36bc2450` are documentation commits belonging to the team lead and were skipped, except where the later `e294cdf8` already lands corrections this review would otherwise have raised.

**Spec compliance: PASS.**
**Quality: high.**
No Critical or defect-class findings.
Two Important findings, both about prose that carries a false or over-scoped justification, neither about behaviour.

## What I ran

Gates, all from `rust/`, exit status read unpiped:

* `cargo test --workspace` dev -- 1401 passed, 0 failed, 4 ignored, exit 0.
* `cargo test --release --workspace` with `REXX_CORPUS_GATE`/`REXX_ASSERTIONS_GATE`/`REXX_BIF_GATE`/`REXX_KEYWORD_GATE` all set -- 1401 passed, 0 failed, 4 ignored, exit 0.
* `cargo test --workspace` dev under the same four gates -- 1401 passed, 0 failed, exit 0.
* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` warm -- exit 0.
* the same clippy from a **fresh** `CARGO_TARGET_DIR` -- 64 units checked, 0 warning lines, exit 0.
  This is the check `rust/CLAUDE.md` says a warm green does not substitute for, and it reproduces the report's own "64 crates" figure.
* each of the ten new tests by name, asserting a non-zero run count rather than a green exit -- all ten report `1 passed`.

No benchmark, `perf` or profiler was run.

## The trace hazard

**The fourteen recorded stanzas were re-measured against the C++ oracle, all fourteen, and every expected byte matches.**
Captured with the project's wrapper from a fresh `mkdir`'d directory, stdout and stderr as separate descriptors.
That covers the seven untraced rows and the seven `trace i`/`trace r` rows, including the two that carry the ordering claim:

* `zw = za.zi` gives `>C>   ZA.ZI => "ZA.3"` then `>V>   ZA.ZI => "v3"`.
* `say za.4`, whose tail resolves to nothing, still gives `>C>   ZA.4 => "ZA.4"` then `>V>   ZA.4 => "ZA.4"`.

So `>C>` before `>V>` whether or not the tail resolves is measured, not reasoned, and the ordering is checked against its neighbouring lines rather than for presence alone.

**The `trace i` half is real rather than a copy.**
Each traced stanza's expected block differs from its untraced twin by the whole `err>` region, and the `trace r` row requires the `>V>` lines to be *absent*, which is the direction a compiled-in emission decision gets wrong.

**Three further shapes checked against the oracle and both engines**, none of them in a case file, all byte-identical on all three:

* a compound and a bare stem read inside a `PROCEDURE EXPOSE` callee under `trace i`, which is the case where the value indent is offset -- the `>C>     ZA.ZI => "ZA.1"` and `>V>     ZA. => "d"` lines carry the callee's deeper indent and match exactly.
* a promoted read in a loop body under `trace i`.
* a compound read raising `NOVALUE` into a `SIGNAL ON NOVALUE` trap, `SIGL` 3 on all three.

## The clause region

`Op::Load` and `Op::TraceRead` are handled in `run_region_ops` (`rust/crates/rexx-exec/src/ir/drive.rs:934` and `:964`), inside `in_stepped_clause_with`, which is where `Op::EvalExpr` and `Op::Const` already sit.
They are refused in the outer `run_ops` loop with `Loud::op_not_driven` (`drive.rs:511`-`:512`), so a jump landing mid-region is loud rather than silent.
`compile::assert_region_ops_name_their_clause` lists both among the ops that carry no instruction index, which is correct -- neither does.

The failure site is exercised: `tests/ir_dual_cases/assignment-and-say:426` runs `signal on novalue` with `n1 = zundef` and asserts `SIGL` is the assignment's own line, on both engines.
`novalue_check` allocates nothing GC-visible, so the raise path has no rooting window either.

**A fragment never compiles to a chunk** (`run.rs:6884` passes `BodyEngine::TreeWalker` explicitly), which closes the one hazard I went looking for: `fragment_plan` returns slots translated into the *enclosing* frame, and a chunk compiled against a fragment's own local `Plan` would have carried local slot numbers into an enclosing-frame read.
It cannot arise.
`run.rs:1055` additionally asserts by `Rc::ptr_eq` that the chunk is compiled from the plan the activation runs with.

## GC rooting

`Op::Load` calls `read_symbol` and then `set_temp` with nothing between them, which is `Op::Const`'s own shape.
Neither `read_at`'s derived-name `text()` nor `read_stem_at`'s `alloc_with` is followed by a second allocation before the value is rooted, and `read_stem_at` additionally stores its new stem in the frame slot.
`Op::TraceRead` reads an already-rooted register.
No `unsafe`.

## The sharing rule

**Verified structurally and then behaviourally.**

Structurally: `eval_node`'s three arms (`eval.rs:441`-`:443`) and `Op::Load` (`drive.rs:956`) both enter `Interp::read_symbol`; `trace_intermediate`'s three arms (`eval.rs:228`-`:232`) and `Op::TraceRead` (`drive.rs:970`) both enter `Interp::echo_symbol_read`.
The old open-coded arms are deleted rather than left beside the new function, so there is no second copy to drift.

Behaviourally, and this is the part that could not be read off the diff: I swapped the `>C>`/`>V>` order **inside `echo_symbol_read`** and re-ran the dual harness.
The engine-against-engine comparison stayed green and the *recorded expectation* failed at `tests/ir_dual_cases/assignment-and-say:230`.
A second implementation would have shown as an engine divergence.
One implementation changing both arms at once is exactly what was observed.

`Interp::read` and `Interp::read_stem` keep real callers (`parse_template.rs:543`/`:552`, `run.rs:2404`/`:6284`/`:6290`/`:7019`), so the `_at` variants are not dead wrappers around a dead wrapper.

## The implementer's five concerns

### 1. The 154-instruction split -- supported by the code

The control build differed from the committed build only in whether `push_read` writes a resolved slot.
With a resolved slot, `Op::Load` reaches `read_at` with `Some(slot)` and returns; without one it reaches `read_at` with `None` and makes exactly one `HashMap<SymbolId, usize>::get` on `code.slots`.
That is the only difference on the executed path, so the -152/-2 split is attributed to the right thing.

The premise underneath the per-pass figure also holds: `bench-programs/varlookup.rex` has one promoted read per pass (`y = x`; `x = x + 1` stays on `EvalExpr`) and `bench-programs/alloc4c.rex` has one (`tab.i = i`; the `||` and the accumulator stay on `EvalExpr`).
`emptyloop.rex` has none, which is what makes the +0.74% control swing on that axis a genuine bound rather than an effect of the change.

I did not re-measure, per the instruction not to.

### 2. The unresolved arm -- measured, not reasoned, and the brief is wrong about why it exists

I instrumented three places with `eprintln!` and rebuilt: `push_read`'s `None` arm, `read_at`'s `code.slots` miss, and `Interp::slot_of`'s `grow_slots` arm.

**Across the whole workspace under all four gates, 1401 passed:**

| probe | hits |
|---|---:|
| `push_read` unresolved (`Simple`/`Stem`) | **0** |
| `read_at` falling through to `slot_of` | 3, all inside hand-built unit tests with an empty slots map |
| `grow_slots` | 1090 |

The `grow_slots` count is the positive control: `eprintln!` from library code does reach the captured stream, so the two zeros are zeros rather than a probe that never compiled.
This independently replicates the report's `panic!` measurement.

**Then I wrote the brief's three routes as programs and ran them on the IR engine.**
Eight programs: `INTERPRET` binding a name read by the body, a nested `INTERPRET`, an `INTERPRET` binding a stem, `DROP (v)` on a name the body reads, `DROP (v)` on a name the body does not otherwise read, a first `CALL` with `result` read, a `CALL` with `result` read before and after, and an `INTERPRET` whose target name is itself computed.
**None of the eight reached either read arm.**
Two of them reached `grow_slots`, and only for `SIGL`.

Four more programs, with the introduced name occurring **nowhere** in the compiled body, do reach `grow_slots`: `ZONLY` via `INTERPRET`, `ZGHOST` via `DROP (zn)`, and `RESULT` via a first `CALL` whose result is never read.

**So the brief's Step 3 sentence is false as a justification.**
The three routes do reach `grow_slots`, but only in the variant where the name occurs nowhere in the body -- and in exactly that variant no compiled read can name it, because `push_read` only ever sees a symbol that occurs as `ExprKind::Variable` or `ExprKind::Stem` in an `Assignment` value or a `SAY` expression, and `Plan::build`/`Plan::note` bind precisely those (`plan.rs:476`-`:479`).
The two conditions are mutually exclusive.

**The arm is right to keep and the report's reasoning for keeping it is sound**: `compile` must not depend on another function's exhaustiveness, and the report says plainly that the arm should not be described as exercised.
What is wrong is the plan, which still tells the next reader that the fallback is needed *because those three routes reach it*.
See finding I1.

### 3. The falsification -- confirmed, and extended

Reproduced in full, each with its own build:

* making `Op::TraceRead`'s emission a no-op reddens `both_engines_agree_on_every_case_file` and names `say n1`.
* it **stays red with `ir_dual_cases/variable-reads` moved out of the directory entirely**, on the same program, which is in `trace-settings`.
  So the new file is not what catches the general dropped `>V>`, exactly as the report says.
* dropping the echo for `SymbolRead::Compound` alone reddens on `say aa.zi`, from `assignment-and-say`, with the new file still held out.
* dropping the echo for `SymbolRead::Stem` alone leaves **the whole workspace** green with the file held out -- 1401 passed, 0 failed, all four gates -- and reddens `both_engines_agree_on_every_case_file` on `zt = zs.` with the file restored.

The module doc in `tests/ir_dual.rs:42`-`:50` records exactly this, both negatives included.
This is "can fail is not adds coverage" done the right way round, and the conclusion is correct.

Two things worth noting beside it.
Holding the file out does **not** change the suite count, because the directory walk is one test -- so a lost case file is invisible to a count, and the `cases > 0` guard is what stands between that and a vacuous pass.
And the same mutations leave `both_engines_agree_across_every_population` green, so the corpus sweep does not see a dropped intermediate line at all; the case-file harness is the only instrument here.

**The tripwire fires.**
Emitting `at + 1` from `push_read` reddens exactly four dual-engine tests with `a compiled read names a slot this body's plan does not give its symbol`, reproducing the report.

### 4. `alloc4c`'s two instruments -- disclosure adequate, presentation is not

No claim elsewhere in the report rests on `alloc4c`'s cycle axis.
The per-pass table, the -154 attribution, the control-build split and the predicted-versus-measured section are all instruction figures.
The report states the restriction and lists the cycle regression under "What I could not settle".

The gap is presentational: the per-axis instruction table bolds **-0.945%** for `alloc4c` in the same way it bolds `varlookup`'s **-3.794%**, and the spec's own rule is that both instruments are required for any claim.
A bolded cell reads as a claim, and the prose two sections later is what withdraws it.
See finding M4.
Task 7-M2 Step 8 re-expresses these figures through the harness, so the exposure is bounded.

### 5. The +0.74% bound -- one table is not covered by its own caveat

The caveat is scoped to "the instruction table", meaning the six per-axis moves, and there it is correctly applied: four of the six (+0.186%, +0.016%, +0.262%, +0.017%) are below the bound and the report says to read them as unmoved.
`varlookup`'s -3.794% is 5x the bound and `alloc4c`'s -0.945% is 1.3x it.

But the **tree-walker cost table** is not covered, and it is drawn from the very binary pair the bound was measured on.
`+0.21%` on `varlookup` and `+0.27%` on `compound` are head-against-control differences, and the +0.74% swing is a head-against-control difference on the same two binaries.
Those two figures are stated as signed quantities and are then generalised ("this is the same trade Task 7 took at 0.2% to 1.3%").
See finding I2.

## Coverage of the brief's cases

All five listed shapes are present, each untraced **and** under `trace i`: a simple symbol, an uninitialised symbol answering its own upcased name, a symbol after `DROP`, a compound, and one introduced by `INTERPRET`.
Two shapes beyond the list are there too -- a bare stem, and a `DROP` reaching its target through a run-time name -- and the bare stem is the one that turned out to carry the file's only unique contribution.
Fourteen stanzas, counted.

The stanzas assert something a degenerate implementation fails, and that is demonstrated rather than assumed: the order swap above failed the recorded block, and each of the three echo mutations failed the engine comparison.

The ten new Rust tests are non-degenerate.
`an_expression_that_only_contains_a_symbol_stays_on_the_general_path` is the adjacent success that stops the three golden cases being satisfied by a compiler that loads any expression containing a symbol, and it covers `.nil` and `>zv` as well as `zv + 1` -- the two expressions that look like a bare read and trace `>E>`/`>O>` instead.
`a_compiled_read_names_the_symbol_its_expression_does` is the one that pins the field `render` deliberately does not print, and it asserts against the parsed expression rather than against a symbol number.
`a_read_echo_behind_its_own_load_is_accepted` cannot fail on its own, which is what an accepted neighbour is for; it is paired with four refusals.

## Findings

### I1 (Important) -- the plan still carries a false justification for the unresolved arm

`docs/superpowers/plans/2026-08-09-phase-4e-ir.md`, Task 8 Step 3, and the generated `.superpowers/sdd/2026-08-09-phase-4e-ir/task-8-brief.md:20`:

> The fallback is not optional: `INTERPRET` introducing a name, `DROP (v)`, and a first `CALL` in a program that never writes `RESULT` all reach `grow_slots` at run time.

Measured above: all three reach `grow_slots` only when the name occurs nowhere in the compiled body, and in that case nothing can compile a read of it, so none of them reaches `push_read`'s fallback -- 0 hits across 1401 tests under four gates and across eight route programs written for the purpose.

`rust/CLAUDE.md` requires the correction to go into the plan rather than into a review or a report, because briefs regenerate from the plan.
The report's own "Corrections the plan needs" section lists two items and not this one, so the sentence is on course to be handed to Task 9 and Task 10 unchanged.

Fix: state the real reason, which the report already has -- `compile` must not depend on `Plan::build` being exhaustive, and a compound has no slot of its own, which is what makes the arm ordinary rather than defensive.

### I2 (Important) -- the report's precision caveat does not cover the table most exposed to it

`.superpowers/sdd/2026-08-09-phase-4e-ir/task-8-report.md`, "What this costs the tree-walker arm" against "A precision caveat, from the same control".

`+0.21%` and `+0.27%` are head-against-control differences; the unexplained `+0.74%` is a head-against-control difference on the same binary pair.
The caveat says "the small numbers **there**", pointing at the per-axis instruction table, and the tree-walker table sits above it uncovered.
The `emptyloop` tree-walker arm did reproduce exactly across that pair, which is a reason to think the effect is arm-specific -- but no mechanism was established, so nothing licenses exempting the tree-walker arm from it.

Fix: either extend the caveat to the tree-walker cost table, or record why that arm is believed exempt and mark it as belief rather than measurement.
The decision it supports -- keep the sharing rule and pay the cost -- was already taken in the plan and does not depend on the number.

### M1 (Minor) -- an unasserted layout claim, and its fraction is off

`rust/crates/rexx-exec/src/ir/mod.rs`, `ReadSlot`'s doc comment at `:460`: "the difference between an [`Op`] that stays the width every other variant already fits in and one that grows by a quarter".

Measured here: `size_of::<Op>()` is 12, the `Load` payload as written is 12, and the same payload with `Option<u32>` is 16.
So the widened `Op` is at least 16 -- a third more than 12, not a quarter, unless "a quarter" is read as a fraction of the new size.
The decision is right either way and the direction of the argument holds.

`rust/CLAUDE.md`'s rule for a load-bearing claim is to assert it in a test.
A `const _: () = assert!(size_of::<Op>() == 12);` would make this the kind of claim that cannot rot, and would also catch a later task widening `Op` by accident.

### M2 (Minor) -- a universal quantifier over a table that grows every task

`rust/crates/rexx-exec/src/ir/golden.rs:80`: "**The `symbol` field is deliberately not rendered**, and it is the only op field this function leaves out."

`render` is exhaustive over `Op` with no catch-all, so a new variant forces an edit to the function but not to this sentence.
Task 9 adds arithmetic ops.
This is the shape `rust/CLAUDE.md` describes as the one that actually rots, and the sentence loses nothing by dropping the "only".

### M3 (Minor) -- a count of the function's own arms, already corrected once

`rust/crates/rexx-exec/src/ir/compile.rs:689`: "**Three shapes, and each split is what the expression is rather than what is convenient.**"

It read "Two shapes" before this task and will read four after Task 9.
The sentence works without the number.

### M4 (Minor) -- a bolded cell asserting what the prose withdraws

`.superpowers/sdd/2026-08-09-phase-4e-ir/task-8-report.md`, `instructions:u` table, `alloc4c` row.
`-0.945%` is bolded as a result on an axis the report itself restricts to one instrument two sections later.
Unbolding it, or marking the row, would make the table say what the prose says.

### M5 (Minor, informational) -- the promoted set is narrower than "an expression slot"

Only an `Assignment`'s value and a `SAY`'s expression go through `push_value`, so a bare symbol in an `IF`/`WHEN` condition, a `SELECT` case expression or a `DO`/`LOOP` header still compiles to `Op::EvalExpr` (`compile.rs:356`, `:397`, `:457`).
That matches the set Task 7 promoted and is not a spec violation -- a literal in those positions is unpromoted too.
The report describes the promotion as "a whole expression slot that is a bare symbol" without naming the exclusion, and a reader of `e2.rex`-shaped output will see `>V>   ZJ => "2"` from a loop header and wonder why it is not a `Load`.
One sentence in the report would close it.

## What I could not settle

* **Whether the base suite really was 1391.**
  I measured 1401 at head, and the ten new tests account for the delta exactly, but reproducing 1391 needs a worktree build at `9da84dc3` and nothing rests on it.
* **The mechanism of the +0.74% control swing.**
  Not chaseable without the benchmarks this review may not run.
  Task 7-M2 Step 0b now sizes its negative control against this bound, which is the right place for it; what would settle it is a `.text` comparison between the two control binaries of the kind Task 4c ran, since that is what turned the last "code layout" attribution into a measurement.
* **Whether `alloc4c`'s cycle regression is noise.**
  The report says it cannot say, and neither can I without running the instrument.
  Task 7-M2 Step 8 re-runs these figures through the harness; that is what settles it.
</content>
</invoke>
