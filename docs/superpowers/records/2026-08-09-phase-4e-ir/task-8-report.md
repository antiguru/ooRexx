# Task 8: promote variable access -- report

BASE `9da84dc3`.
Commits, in order: `938de575` (the dual cases, before anything was promoted) and `0036ca7c` (the
promotion). Both hashes read back with `git log`. This report is not one of them: `.superpowers` is
in `.gitignore`, so the ledger is on disk and outside version control.

Suite **1391 -> 1401**, 0 failed, 4 ignored, green in dev, dev+STRICT, release and release+STRICT
(`REXX_CORPUS_GATE`/`REXX_ASSERTIONS_GATE`/`REXX_BIF_GATE`/`REXX_KEYWORD_GATE` all set), each exit
status read unpiped. The base count was taken by running the same command in a worktree at
`9da84dc3`, so the `+10` is measured rather than counted off the diff.
`cargo fmt --all --check` 0.
`cargo clippy --workspace --all-targets -- -D warnings` 0, re-run from an empty target directory
(64 crates checked, `rexx-exec` among them).
`Interp::chunks_refused` is asserted zero across the corpus by the dual harness, which passes.

## The op set

Two ops, and the split between them is [`Op::Const`]/[`Op::TraceLiteral`]'s exactly.

| op | what it does | why it is its own op |
|---|---|---|
| `Load { symbol, read, at, dst }` | produces one bare symbol's value into register `dst`, through `Interp::read_symbol` | `eval.rs` is not entered; the value comes from a frame slot |
| `TraceRead { symbol, read, src }` | echoes the `>V>` line of the read in `src`, and `>C>` in front of it for a compound | `eval.rs` emits those lines as a **side effect** of evaluating; a load emits nothing |

`read` is a `SymbolRead` -- `Simple`, `Stem` or `Compound` -- because a compiled op holds no borrow
of the node it was emitted for. It lives in `eval.rs` beside the function that dispatches on it, the
way `HeaderRole` lives in `run.rs`.

A promoted `Assignment` whose value is a bare symbol is now `Clause`, optionally `TraceClause`,
`Load`, `TraceRead`, `Store`; a `SAY` is the same with `Say` in place of `Store`.

## What is promoted, and what deliberately is not

Only a whole expression slot that is a bare symbol. `ExprKind::Variable`, `ExprKind::Stem` and
`ExprKind::Compound` emit the pair; everything else stays on `Op::EvalExpr` entire, including an
expression that merely *contains* a symbol. `an_expression_that_only_contains_a_symbol_stays_on_the_general_path`
is the adjacent success that pins it, with `zw = zv + 1`, `zw = .nil` and `zw = >zv` -- the last two
because they are the expressions that look like a bare read and are not one: they trace `>E>` and
`>O>` rather than `>V>`, so a `Load` for either would emit a line the oracle prints nowhere.
`ExprKind::Constant` stays on `EvalExpr` for the reason `Op::Const` already gives: its bytes are in
the symbol table, which `compile` does not take.

## The sharing rule

`eval_node`'s three arms and `trace_intermediate`'s three became two `pub(crate)` functions in
`eval.rs`, entered from the tree-walker where those arms were and from the driver from a register:

* `Interp::read_symbol(code, read, id, at)` -- the derived name an unset simple read answers, the
  `Body::Stem` a bare stem miss allocates, the `compound_parts`/`tail_key`/`stem_get` chain a
  compound resolves, and the `NOVALUE` check two of the three make and the third does not.
* `Interp::echo_symbol_read(code, read, id, value)` -- `>C>` then `>V>`, gated on
  `trace_mode().intermediates` before anything is rendered, which is `echo_literal`'s own shape.

The three reads stay three operations sharing one entry point rather than becoming one operation:
`read_symbol` dispatches on the kind. A comment in `eval_node` that said the `NOVALUE` split "could
not be done by routing all three reads through one place" was corrected rather than left, since this
change routes them through one place; what it was right about -- that the three are not one read --
is now stated on `read_symbol` itself, with the aliasing and `NOVALUE` measurements it rests on.

`Interp::read` and `Interp::read_stem` kept their existing callers and gained `read_at` and
`read_stem_at` underneath, which take the slot when a compiler already resolved it. That is one
resolution made at two times, not two resolutions.

## The compile-time slot, and why a compound has none

`compile` now reads its `plan` argument, for one thing: `plan.by_symbol`, which is the map
`Code::slots` is a view of at run time. `Op::Load` carries the answer in a `ReadSlot` -- a `u32`
with one reserved value rather than an `Option<u32>`, because eight bytes there would widen every
op in every chunk for a field two of them carry. A slot too wide to encode is unresolved rather
than a refusal: the read still has a correct answer and the run-time path computes it, so a body
must not fall back to the tree-walker over an optimisation.

A compound never carries one, and that is the ordinary case rather than a defensive arm: its read
goes through the *stem's* slot and a tail key worked out at the read site, and `Plan::note_compound_name`
binds both by name with no `SymbolId` to hang them on.

**The tripwire.** `run_ops` asserts in debug that a resolved slot equals what `code.slots` gives the
same symbol. A chunk run against another body's plan would otherwise read somebody else's slot and
answer with it. Made to fire: emitting `at + 1` reddens four dual-engine tests with that message.

**The unresolved arm for a simple variable or a stem is never taken by anything this suite runs**,
measured rather than reasoned: replacing the fallback with a `panic!` and running the whole workspace
under all four gates gives 1401 passed, 0 failed and no panic. `Plan::build` is exhaustive over the
body, so a symbol an instruction reads is bound -- but that is a property of another function, and
the arm stays because `compile` must not depend on it. It should not be described as exercised.

## Trace evidence

`trace_intermediate`'s arms say what a read owes and the oracle confirmed every line. Captured with
the project's wrapper, from a fresh directory, stdout and stderr as separate descriptors:

```text
     3 *-* zw = zv                       4 *-* zw = za.zi
       >V>   ZV => "val"                   >C>   ZA.ZI => "ZA.3"
       >>>   "val"                         >V>   ZA.ZI => "v3"
       >=>   ZW <= "val"                   >>>   "v3"
                                           >=>   ZW <= "v3"
```

`>C>` comes first whether or not the tail resolves -- `say za.4` gets `>C>   ZA.4 => "ZA.4"` then
`>V>   ZA.4 => "ZA.4"` -- which is the ordering row in the case file.

**The case file.** `tests/ir_dual_cases/variable-reads`, fourteen stanzas: a set simple variable, an
unset one, one after `DROP`, one after a `DROP` that learned its target's name at run time, a bare
stem, a compound whose tail is computed, and a binding `INTERPRET` made that no compiled clause ever
wrote -- each untraced and under `trace i`, plus a `trace r` row that requires the lines to be
absent. Every expected byte was measured against the C++ oracle, and all fourteen were confirmed
identical on the oracle, on this crate's tree-walker and on its IR arm at `938de575`, before anything
was promoted -- so each row could tell whether it would ever have failed.

**The compile-time assertion.** `assert_read_echoes_follow_their_load` pins position, register,
symbol **and** read kind. The symbol is the half a literal's echo cannot get wrong: `>V>` is tagged
with the name, so an echo naming another symbol prints the right value under the wrong one, and a
compound's `>C>` would resolve the wrong tail. Four `should_panic` witnesses and one accepted
neighbour.

## The falsification run

Making `Op::TraceRead`'s emission a no-op reddens `both_engines_agree_on_every_case_file`, which
names the program:

```text
stderr differs between engines
 left:  ... 3 *-* say n1 \n >V>   N1 => "lit" \n >>>   "lit"
 right: ... 3 *-* say n1 \n                       >>>   "lit"
```

**And it stays red with `variable-reads` held out of the directory entirely**, because that program
is in `trace-settings`, a file written for something else. So the general case is already covered
and the new file is not what catches it. The same holds for the compound: dropping the echo for
`SymbolRead::Compound` alone reddens the sweep on `say aa.zi` from an existing file.

What the new file *is* the only catcher for is narrower, and it was found by looking: dropping the
echo for a **bare stem** alone leaves every dual-engine test in the workspace green without
`variable-reads`, and reddens `both_engines_agree_on_every_case_file` on `zt = zs.` with it. That
sentence and its two negatives are now in `ir_dual.rs`'s own module doc.

## Measurement

`perf stat -e instructions:u,cycles:u`, both engine arms from **one** build, arm order rotated per
round, medians, under `ulimit -v 8388608`, machine otherwise idle. Base binary built from a
worktree at `9da84dc3`; head binary from `0036ca7c`. Seven rounds for `varlookup` and `alloc4c`,
five for the rest. Wall clock was not used.

Instruction counts are deterministic: the seven-round spread is ±0.00% on every axis but `compound`
(±0.002%) and `startup` (±0.08%). Cycle spreads are ±0.2% to ±2.0%.

**`instructions:u`**

| axis | base TW | base IR | base IR/TW | head TW | head IR | head IR/TW | head IR arm |
|---|---:|---:|---:|---:|---:|---:|---:|
| `varlookup` | 72.5807e9 | 77.1217e9 | 1.06256 | 72.6567e9 | 74.1957e9 | **1.02118** | **-3.794%** |
| `alloc4c`&nbsp;† | 15.9843e9 | 16.3043e9 | 1.02002 | 15.9863e9 | 16.1503e9 | **1.01026** | **-0.945%** |
| `emptyloop` | 37.8507e9 | 40.3257e9 | 1.06539 | 37.9007e9 | 40.4007e9 | 1.06596 | +0.186% |
| `arith` | 30.4630e9 | 30.7257e9 | 1.00862 | 30.4620e9 | 30.7307e9 | 1.00882 | +0.016% |
| `compound` | 77.2634e9 | 78.4587e9 | 1.01547 | 77.4484e9 | 78.6642e9 | 1.01570 | +0.262% |
| `strings` | 85.1798e9 | 86.6257e9 | 1.01698 | 85.1648e9 | 86.6407e9 | 1.01733 | +0.017% |
| `startup` | 538,065 | 562,259 | 1.04389 | 540,068 | 563,365 | 1.04300 | +0.2% |

† **`alloc4c`'s bolded figures are an instruction claim and nothing more.** Its `cycles:u` moves the
other way, and the row is marked here so the table does not have to be read alongside the section two
below to say so.

The base column reproduces the ratios the brief quotes for `9da84dc3` -- 1.06256, 1.00861, 1.01546,
1.01698, 1.02002, 1.06539 -- to the last digit or one off it, which is the instrument agreeing with
the record it is being compared against.

**`cycles:u`, as ratios**

| axis | base IR/TW | head IR/TW |
|---|---:|---:|
| `varlookup` | 1.04239 | **1.01685** |
| `alloc4c` | 1.01410 | 1.03385 |
| `emptyloop` | 1.02752 | 1.02854 |
| `arith` | 1.02337 | 1.01314 |
| `compound` | 1.04612 | 1.02908 |
| `strings` | 1.01068 | 1.02842 |

**`alloc4c` is the axis where the two instruments disagree**, and it is worth reading rather than
averaging away. Its IR arm executes 0.945% *fewer* instructions and spends 1.5% *more* cycles, while
its tree-walker arm spends 0.4% fewer cycles for 0.013% more instructions. It is the
allocation-churn axis, so its cycles are dominated by the allocator rather than by the interpreter
loop, and both arms' cycle medians move about a per cent between the two binaries with no
instruction change to match. The instruction figure is the one this change can claim.

### Two problem sizes

`varlookup` at n = 19,000,000 and 38,000,000; `alloc4c` at 1,000,000 and 2,000,000. The difference
divided by n is the per-pass cost, with fixed cost cancelled.

| per pass | base TW | base IR | head TW | head IR | IR change |
|---|---:|---:|---:|---:|---:|
| `varlookup` | 3820.0 | 4059.0 | 3824.0 | 3905.0 | **-154** |
| `alloc4c` | 16040.1 | 16360.2 | 16042.2 | 16206.2 | **-154** |

Both bodies execute exactly one promoted read per pass -- `y = x` and `tab.i = i` -- and both save
**154 instructions**, independently. The IR-minus-TW per-pass gap goes from 239 to 81 on `varlookup`
and from 320 to 164 on `alloc4c`.

### Where the 154 comes from, measured rather than attributed

A control build with `push_read` emitting `ReadSlot::UNRESOLVED` unconditionally -- the native op
without the compile-time slot, which the suite shows is behaviour-neutral:

| per pass | base IR | IR, op only | IR, op + slot |
|---|---:|---:|---:|
| `varlookup` | 4059.0 | 4057.0 | 3905.0 |
| `alloc4c` | 16360.2 | 16358.2 | 16206.2 |

**-2 from the native op and -152 from the compile-time slot, on both axes.** Bypassing `eval`'s
wrapper and `eval_chunk_expr`'s dispatch is worth almost nothing; the whole of the win is not making
a `HashMap<SymbolId, usize>` probe per read.

### Predicted against measured

**The brief's prediction basis is the wrong half of 4d-1's attribution, and the control build above
is what says so.** The brief says to predict against half of 4d-1's `-10.7%`, since one of
`varlookup`'s two assignments has a promoted read. That gives about `-5.4%`; measured is `-3.79%`
on the IR arm's instructions. But `-10.7%` is P2's figure, on **wall clock**, for replacing the
*name*-keyed `Interp::slot_of` lookup at an assignment's **target** -- collecting the `20.2%`
`phase-4d-attribution.md` attributes to `slot_of` on `varlookup` and paying an id-keyed lookup back.
This change removes an **id-keyed** lookup, which that same page attributes separately at `12.5%` on
`varlookup` and explicitly records as "bounded but not prototyped".

So the figure to predict against was `12.5%` divided over the id-keyed lookups a pass makes, not half
of `10.7%`. Measured, one such lookup is 152 instructions, 3.75% of the IR arm's per-pass cost and
3.98% of the tree-walker's -- consistent with `12.5%` covering about three of them per pass. (That
last is arithmetic on a profile percentage, not a count I measured.)

**Yes, this is the same change 4d-1 measured**, applied to the read side rather than the target
side, and the id-keyed half rather than the name-keyed one. The target side is still unresolved on
both engines and still worth about the same per assignment.

### What this costs the tree-walker arm

The sharing rule is not free here, and a third control build says how much: `eval_node`'s three arms
open-coded again, IR side untouched.

| TW arm, `instructions:u` | base | head (shared) | control (open-coded) |
|---|---:|---:|---:|
| `varlookup` | 72.5807e9 | 72.6567e9 | 72.5047e9 |
| `compound` | 77.2634e9 | 77.4497e9 | 77.2383e9 |
| `emptyloop` | 37.8507e9 | 37.9007e9 | 37.9007e9 |

So the extraction costs the tree-walker about **+0.21%** on `varlookup` and **+0.27%** on `compound`,
and nothing on `emptyloop`, whose `+0.05e9` against base survives the control and therefore comes
from `read`/`read_stem` gaining the slot-hint branch rather than from the extraction. `arith` and
`strings` are unmoved either way. This is the same trade Task 7 took at 0.2% to 1.3%.

`#[inline]` on `read_symbol` was tried and dropped: it produced instruction counts exactly equal to
the committed build on `varlookup`, `emptyloop` and `alloc4c` and within `compound`'s own spread, so
it bought nothing and a no-op annotation is not worth shipping.

**A precision caveat, from the same control, and it covers the table above it as well as the one
further up.** That build's `emptyloop` **IR** arm reads 40.7007e9 against the committed build's
40.4007e9 -- +0.74% on an axis whose body holds no promoted read at all. The mechanism was not
established and I did not chase it. It is larger than four of the six per-axis moves in the
instruction table, so the small numbers there should be read as "unmoved", not as signed quantities.

**And `+0.21%` and `+0.27%` are differences across that same binary pair**, so nothing licenses
reading them more finely than the `+0.74%` allows. What can be said for them is weaker and is worth
separating from what was measured:

* *Measured*: `emptyloop`'s tree-walker arm reads 37,900,659,582 on the committed build and
  37,900,660,317 on the control -- 735 instructions apart in 37.9 billion, which is exact for this
  purpose. So whatever produced the `+0.74%` on the IR arm of that same pair did not touch the
  tree-walker arm of it.
* *Belief, not measurement*: that this makes the effect arm-specific, and the tree-walker figures
  therefore finer than `±0.74%`. One axis reproducing is consistent with an arm-specific effect and
  equally consistent with an axis-specific one, and no mechanism was established for either. The
  honest bound on `+0.21%` and `+0.27%` is the same `±0.74%`; what they support is a direction and
  an order of magnitude.

The decision those two numbers sit under -- keep the sharing rule and pay for it -- was taken in the
plan and does not rest on their precision.

## What I could not settle

* **Why `emptyloop`'s tree-walker arm executes 2 more instructions per pass.** It survives the
  open-coded control, so it is the slot-hint branch in `read`/`read_stem`, but I did not identify
  which per-pass read reaches it. 0.13% on the axis, and it moves that axis' ratio by +0.0006.
* **`alloc4c`'s cycle regression.** Instructions fall and cycles rise, on an allocator-bound axis
  whose arm medians move about a per cent between binaries. I can say the instruction count is the
  claim; I cannot say the cycle number is noise.
* **The `+0.74%` swing above**, which is the honest bound on how finely these binaries can be
  compared -- including the `+0.21%` and `+0.27%` the sharing rule costs, which are differences
  across that same pair.

## Corrections to the plan -- both landed, plus one I did not ask for

Raised at hand-off and fixed by the controller at `e294cdf8`, with the precision bound carried into
Task 7-M2's harness step in the same commit. Recorded here as what was wrong and why, not as an open
request.

A third, which was the review's I1 and not on my list: the plan's "the fallback is not optional:
`INTERPRET` / `DROP (v)` / a first `CALL`" sentence was falsified by the `panic!` probe above -- none
of those three reaches the unresolved arm, because the read is written in the body and the body's own
plan therefore binds it. Corrected at `bea6b2eb` to the two reasons that survive: `compile` must not
depend on `Plan::build` being exhaustive, and a compound has no slot of its own.

1. **`task-8-brief.md` line 3**, and the identical line at `docs/superpowers/plans/2026-08-09-phase-4e-ir.md:947`:
   > "4d-1 measured assign-by-symbol-id at -10.7% there as a reverted prototype, which is the figure
   > to predict against and to beat or explain."

   `-10.7%` is P2's **wall-clock** figure for the **name**-keyed lookup at an assignment target.
   This task removes the **id**-keyed lookup at a read, which `phase-4d-attribution.md` attributes
   separately at `12.5%` on `varlookup` and records as not prototyped. Predicting against half of
   `10.7%` compares an instruction count to a wall-clock number about a different lookup. The
   sentence in the following paragraph -- "Predict against that, not against the whole of -10.7%" --
   inherits the same error.

2. **The dispatch message** (not the brief file) says the golden tests use "committed `.ops` files
   under `testdata/ir/`". There is no `testdata` directory and no `.ops` file in the tree; the golden
   expectations are inline strings in `ir/golden_tests.rs` compared against `ir/golden.rs`'s
   `render`. Anything generated from that sentence will look for files that do not exist.
