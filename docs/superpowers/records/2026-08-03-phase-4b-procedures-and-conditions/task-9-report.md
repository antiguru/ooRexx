# Task 9 report -- the call trace surface, and three absent value lines

**Status: DONE.** Commit `233fbd8b` (`git log`: `233fbd8bdc9159790db04fb392ca8c9ad134386e`,
"4b Task 9: the call trace surface, and three absent value lines"), on top of
`e72cc19f`. `d2f490bc` (the plan's own correction, committed by the controller
while this task ran) sits above it and touches no crate file.

Gates, each run from `rust/` with its exit status read unpiped:

| gate | result |
|---|---|
| `cargo test --workspace` | **986 passed / 0 failed** (was 979/0) |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | **40 of 40**, mode STRICT (was 39 of 39) |
| `cargo test -p rexx-exec --test assertions` | **4224 / 4259** (unchanged) |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |

All three absent-value-line gaps are closed, not two. Nothing was deferred for
budget. Two *new* gaps were found while probing and are recorded as KNOWN GAPS
with transcripts rather than fixed; both are stated below with the reason.

---

## 1. What the oracle actually does, measured

Every transcript below is a real run, oracle wrapper as mandated, from a fresh
directory created for this task. stdout, stderr and exit status were read as
three separate descriptors throughout; no `2>&1` anywhere.

### The two-argument `trace r` transcript in the brief

Reproduced exactly. On `e72cc19f` the only divergence in it was `use arg`'s own
pair of `>>>` lines:

```
oracle                          ours, at e72cc19f
     2 *-* call sub 1, 2             2 *-* call sub 1, 2
     5 *-*   sub:                    5 *-*   sub:
     6 *-*   procedure               6 *-*   procedure
     7 *-*   use arg a, b            7 *-*   use arg a, b
       >>>     "1"                 (nothing)
       >>>     "2"                 (nothing)
     8 *-*   return a + b            8 *-*   return a + b
       >>>     "3"                    >>>     "3"
       >>>   "3"                      >>>   "3"
```

So the label echo, `procedure`, the callee's `+2` indent and the doubled return
value were already right (Tasks 3 and 5); the activation indent base (Step 5)
was already in the tree as `Interp::activation_indent`, set to the calling
clause's printed indent plus two in `resolve_and_run_call`. **Step 5 needed no
code.** `static_indent`'s signature is unchanged, `MAX_CLAUSE_INDENT` is
untouched and not extended to value lines.

### `>A>` -- ARGUMENT, both call forms, intermediates-gated

```
trace i / call sub 1,,3

     2 *-* call sub 1,,3
       >L>   "1"
       >A>   "1"
       >A>   ""          <- an omitted position traces an EMPTY line, not none
       >L>   "3"
       >A>   "3"
     4 *-*   sub:
     5 *-*   use arg p, q, r
       >>>     "1"
       >=>     P <= "1"
       >>>     "3"
       >=>     R <= "3"  <- Q is dropped and traces neither line
```

`RexxInstruction::evaluateArguments` (`RexxInstruction.cpp:144`-`162`), read
directly: `traceArgument` after each `evaluate`, and
`traceArgument(GlobalNames::NULLSTRING)` for an omitted one. The gate is
`traceArgument`'s own `if (settings.intermediateTrace)` -- confirmed by
running the same program under `trace r`, which shows none of the three.

### `>F>` -- FUNCTION, expression form only

```
trace i / zz = sub(1, 2)

       >>>     "3"        <- the callee's RETURN, at the callee's indent
       >F>   SUB => "3"   <- the caller's, tag unquoted
       >>>   "3"
       >=>   ZZ <= "3"
```

`call sub 1, 2` traces no `>F>` at all -- the oracle's `traceFunction` call sits
in `ExpressionFunction::evaluate` (`ExpressionFunction.cpp:228`), which the
`CALL` instruction never goes through. That measured pair is what makes this a
one-form prefix rather than "the same line at a different place".

### `>R>` -- ALIAS, and it is RESULTS-level

The brief's I14 says `>R>` is results-level and it is. It also says the call
site shows `>O>   ">" => "PP"` and `>A>   "orig"`, and **that half is wrong on
both lines**. Measured:

```
trace i / orig = 'PP' / call sub >orig / ... / use arg >qq

     3 *-* call sub >orig
       >O>   ">" => "ORIG"    <- the NAME, upcased
       >A>   "PP"             <- the VALUE
     8 *-*   use arg >qq
       >R>     "ORIG" => "Q"  <- caller's name, then callee's name
```

The C++ agrees on the order: `traceVariableAlias(reference->getName(),
useRef->getName())` (`UseInstruction.cpp:167`). Neither operand of the `>R>`
line is a value.

Gating measured both ways: the same program under `trace r` shows the `>R>`
line and nothing else; under `trace l` it shows nothing at all.

`<orig` traces the identical `>O>   ">" => "ORIG"` -- the C++ passes the
literal `">"` regardless of which byte was written
(`VariableReferenceOp.cpp:116`). Probed rather than assumed.

### I31 -- the Controlled loop's re-tested pass

```
trace i / do ii = 1 to 2 / nop / end

     2 *-* do ii = 1 to 2
       >L>   "1"
       >L>   "2"
       >K>   "TO" => "2"
       >=>   II <= "1"      <- setup, at the DO clause's OWN indent
     3 *-*   nop
     4 *-* end
     2 *-* do ii = 1 to 2
       >V>     II => "1"    <- re-tested pass, at the BODY's indent
       >>>     "1"
       >>>     "2"
       >=>     II <= "2"
```

Four lines per re-tested pass, not two: the brief and the KNOWN GAP row named
only the `>>>` pair because they were written from a `trace r` transcript,
where `>V>`/`>=>` are invisible. All four come from the same
`DoBlock::checkControl` arm (`DoBlock.cpp:182`-`205`) and all four are closed.

Three further lines came with them, each measured separately:

* the **setup** `>=>`, at the `DO` clause's own indent rather than the body's,
  because the oracle's setup runs before the block is pushed;
* a **zero-trip** loop's setup `>=>` (`do ii = 5 to 3` still traces
  `>=>   II <= "5"`, which is the existing bound-before-test rule showing
  through the new line);
* **`DO OVER`'s** own `>=>`, at the *body's* indent even though it happens
  once, because `checkOver` runs with the block already pushed.

That indent split is the reason `bind_control` now takes an indent from its
caller instead of deriving one.

### `EXIT <expr>`

Confirmed exactly as the dispatch measured it, and the bare form is correctly
silent on both sides: `RexxInstructionExit::execute` goes through
`RexxInstructionExpression::evaluateExpression` (`RexxInstruction.cpp:223`-
`235`), whose `traceResult` is inside the `expression != OREF_NULL` arm.

### `RAISE ... ARRAY`, found while adding `>A>`

Two defects in the three lines `>A>` had to be inserted into, and the second is
a **stdout/stderr content** difference rather than a trace one:

```
trace i / raise syntax 40.4 array('R',,'X')

oracle:                          ours, at e72cc19f:
       >L>   "40.4"                >L>   "40.4"
       >K>   "SYNTAX" => "40.4"    >K>   "SYNTAX" => "40.4"
       >L>   "R"                   >K>   "ARRAY" => "an Array"   <- too early
       >A>   "R"                   >L>   "R"
       >A>   "R"                   >L>   "X"
       >A>   ""
       >L>   "X"
       >A>   "X"
       >A>   "X"
       >K>   "ARRAY" => "an Array"

Error 40.4:  ... maximum expected is .     Error 40.4:  ... maximum expected is X.
```

* **Order.** The `>K>` line comes *after* the elements. Invisible under
  `trace r`, which is why `lang/condition_traps.rex` -- which contains a
  `RAISE ... ARRAY` -- never saw it.
* **`>A>` fires twice per supplied element** and once, empty, for an omitted
  one. `RaiseInstruction.cpp:229`-`237` calls `traceArgument(arg)` on both
  sides of the array `put`. Reproduced as measured rather than tidied to one
  line: the criterion is agreement with the oracle, not with what the C++
  ought to have done.
* **An omitted element holds its place in the substitution list.** `&2` is a
  hole and substitutes as empty; `'X'` at index 3 is never used. We closed the
  gap up and printed `maximum expected is X.` -- readable, plausible, wrong.
  The code comment at that site had recorded this as "unmeasured, and the only
  unmeasured choice in this function", which is exactly the label that made it
  cheap to check.

---

## 2. The four both-DIFF probes -- settled by measurement, not by assumption

Task 7's round-4 review left four probes DIFF on both sides (`q11`, `y4`, `z6`,
`w29`). The dispatch required determining whether they are I31 or a third gap,
**assuming neither**.

They are I31. This is not an argument from their description: the four probe
files were found (`scratchpad/rr4/`), copied into a clean directory and re-run
against both binaries.

```
                      e72cc19f          233fbd8b
q11  trace r, do i = 1 to 2         DIFF(stderr)   MATCH
y4   trace r, ITERATE from a SELECT DIFF(stderr)   MATCH
z6   trace r, ITERATE in an IF      DIFF(stderr)   MATCH
w29  trace r, ITERATE + accumulator DIFF(stderr)   MATCH
```

MATCH here is all three descriptors, byte for byte, with no normalisation.

So the three absent-value-line gaps this task owns are **three**, not four:
I31 (which the four probes are), `EXIT <expr>`, and -- discovered here --
`RAISE ... ARRAY`'s own elements. All three are closed.

---

## 3. Regression sweep, A/B against `e72cc19f`

20 probes written for this task, chosen for the shapes the loop restructure
could plausibly break, run against both binaries and the oracle:

**18 DIFF -> MATCH, 2 MATCH on both, 1 DIFF on both, 0 MATCH -> DIFF.**

The two MATCH-on-both are the adjacent passing cases that pin the rule rather
than the fix: `w10` (`do 3` -- a `Count` loop, which has no control variable and
must gain no lines) and `w14` (`call sub` with no arguments and a bare `return`,
which must gain no `>A>`, no `>F>` and no `>>>`).

Shapes covered: `ITERATE`, `LEAVE`, negative `BY`, fractional `BY`, `FOR`,
nested controlled loops, `WHILE`, `UNTIL`, an unbounded `do ii = 1`,
`NUMERIC DIGITS` changed inside a loop body, a call inside a loop body, two
expression calls in one clause, nested `f(f(1))`, `call (nm)`, `use arg` with a
default, `use strict arg ... ...`, and a `SIGNAL ON SYNTAX` trap firing inside a
callee.

**The one DIFF on both is pre-existing and unrelated**: with a handler reached
by `SIGNAL ON SYNTAX` from inside a *called routine*, the oracle prints the
handler's clauses at the main body's indent and we print them two further in
(the callee's `activation_indent` is not unwound by the signal). Confirmed DIFF
on `e72cc19f` too. It is an *indent* divergence, so DEVIATION 0 covers it for
both harnesses; it is not recorded as a new gap for that reason, and it is
flagged as a concern below.

---

## 4. What was built

`rust/crates/rexx-exec/src/trace.rs`

* `trace_argument` (`>A>`), `trace_function` (`>F>`), `trace_alias` (`>R>`),
  each with its own gate and its own measured transcript in the doc comment.
  `>R>`'s gate is `results`; the other two are `intermediates`.
* The module doc's "nine prefixes reachable from pure-4a code" is now ten
  plus 4b's three.

`rust/crates/rexx-exec/src/eval.rs`

* `trace_intermediate` gains two arms: `ExprKind::VariableReference` (`>O>`
  with the referenced name) and `ExprKind::Call` (`>F>`). Both were previously
  swallowed by the `_ => {}` arm, which is why `say >pq` traced nothing at all.

`rust/crates/rexx-exec/src/run.rs`

* `resolve_and_run_call` traces `>A>` per argument position, reading the
  caller's indent fresh on each pass (an argument can itself contain a call).
* `eval_argument` evaluates the **reference node** rather than its inner
  variable, which is what routes it through the new `>O>` arm instead of the
  `>V>` one.
* `bind_use_target` traces `>>>`+`>=>` for a bound value and `>R>` for an
  alias; a dropped target still traces neither.
* The `Exit` arm traces `>>>`.
* `exec_raise`'s `array` arm: elements first, `>K>` last, `>A>` twice per
  element, and an omitted element pushes an empty substitution.
* `LoopState::Controlled` gains `stepped`; `loop_advance` takes `do_indent`
  and `loop_indent` and does the `BY` increment itself, tracing on both sides
  of it; `bind_control` traces `>=>`; **`loop_step` is deleted**.

The increment moving is the only structural change. Nothing runs between the
old site (bottom of pass) and the new one (top of the next `loop_advance`)
except `END`'s or an `ITERATE`'s transfer, so the settings the addition runs
under are the same. `FOR`, `TO`, bound-before-test and `ITERATE` were
re-verified by the sweep above and by the existing suite.

### Witnesses added

Five committed trace expectations and one live corpus program. **Every one of
the six diverges when run against the `e72cc19f` binary**, which is the
negative control for the whole set:

| witness | covers | on `e72cc19f` |
|---|---|---|
| `tests/trace_oracle/call_arguments.rex` | `>A>` at four position shapes, `USE ARG`'s value lines, `>O> ">"` | STDERR-DIFF |
| `tests/trace_oracle/function_call.rex` | `>F>` twice in one clause | STDERR-DIFF |
| `tests/trace_oracle/use_arg_alias.rex` | `>R>` **under `trace r`**, which is what pins its gate | STDERR-DIFF |
| `tests/trace_oracle/controlled_loop.rex` | the control variable's lines in both modes, ITERATE, negative `BY`, and `DO OVER` beside it | STDERR-DIFF |
| `tests/trace_oracle/exit_value.rex` | `EXIT <expr>`'s `>>>` | STDERR-DIFF |
| `corpus/lang/raise_array_substitution.rex` | `>A>` under `RAISE`, the `>K>` ordering, and the substitution hole | STDERR-DIFF |

`use_arg_alias.rex` runs under `trace r` deliberately: under `trace i` it would
pass just as well with the gate written wrongly as `intermediates`.

The bare-`EXIT`-traces-nothing case is deliberately *not* in `exit_value.rex` --
it is in three other witnesses that each reach a bare `exit`, so that half fails
if the arm ever traces unconditionally.

### Step 7 -- the coverage measure

`PREFIX_COVERAGE` in `tests/trace_oracle.rs` lists all nineteen prefixes, each
either `Witnessed` or `Owned(phase)`. `WITNESSED_PREFIX_COUNT = 13` and
`OUT_OF_SCOPE_PREFIX_COUNT = 6` are committed literals, and the test asserts:

1. the table's prefix set equals `support::TRACE_PREFIXES` (the nineteen read
   from `RexxActivation.cpp`'s own table), so it cannot drop a prefix to make
   the fraction look better or invent one to make it look worse;
2. its `Witnessed` subset equals `CLAIMED_PREFIXES`, which the pre-existing
   test already ties to what the committed `.expected` files contain;
3. both counts, and that they add up to the whole table;
4. every owner names a phase from a fixed list.

Nothing is printed. The number is only ever read by assertions.

Owners for the six: `+++` and `>.>` -> 4c (the command `RC(n)` line and
`PARSE`'s placeholder variable, both read from the C++); `>M>` and `>N>` ->
Phase 5; `>I>`/`<I<` -> 4c.

### Step 1 -- regeneration round-trip

All five pre-existing expectations regenerate byte-identical from the live
oracle using the recipe in the module doc. Ran, not assumed.

---

## 5. `>I>`/`<I<` -- the deferral, written as instructed and re-measured

The exclusions row is written as a **deferral to 4c alongside `::routine`
dispatch itself**, not as unreachability. Everything in it was re-measured for
this task rather than carried forward:

* `call zorkolo` with a `::routine zorkolo` present runs on the oracle at rc 0,
  so a `::routine` is reachable in 4b for any non-builtin name.
* A caller's `trace l` targeting a `::routine` emits **nothing**.
* `::options trace labels`, or a non-dynamic `trace l` as the routine's own
  first instruction (the `earlyTraceEntry` path, `RexxActivation.cpp:3629`-
  `3640`), makes both lines fire.
* **The content is not a clause echo.** Verbatim:
  `>I> Routine "ZORKOLO" in package "<absolute path>".` So reproducing it needs
  the package object, and the absolute path makes it unsuitable for a committed
  expectation -- a witness for these belongs in the live corpus.
* The three differences 4c will meet, each re-run: its own variable pool
  (a caller's `nn = 5` reads as `NN` inside), builtins shadow it (`call max 1, 9`
  with a `::routine max` sets `RESULT` to 9 and the routine never runs), and
  `TRACE` does not cross into it (a caller's `trace r` echoes none of its
  clauses).

---

## 6. Two new KNOWN GAPS -- recorded, not fixed, and why

Both are in `phase-4-exclusions.txt` with their transcripts. Adding a KNOWN GAP
row needs no amendment; neither is fixed here, and the reason is stated for each
rather than left to inference.

### `TRACE L` echoes nothing

The oracle echoes every label clause it **executes**, with the ordinary `*-*`
prefix. Measured:

```
trace l / say 'a' / signal there / say 'skipped' / there: / say 'b' /
call sub / exit / sub: / say 'c' / return

oracle stderr:            ours: (empty)
     5 *-* there:
     9 *-*   sub:
```

stdout and rc match; only these lines are missing. **This falsifies the brief's
I14 sentence** "an internal-label call with `trace l` first emits nothing, with
or without `PROCEDURE`" -- which is why it is recorded rather than left in a
report.

Not fixed here because `mode_from_setting`'s classification of `L` is read by
`TRACE()`'s own reported setting, by D17's three-field `TraceMode` argument and
by 4c's `>I>`/`<I<` row; changing it without them is how a fourth thing goes
wrong. The fix itself is small (a fourth `TraceMode` field and a gate at the one
clause-echo site, which already has the instruction). Owner unassigned.

### A compound control variable is stored in the wrong place

```
do aa.1 = 1 to 2 / nop / end / say aa.1

oracle: 3      e72cc19f: AA.1      233fbd8b: AA.1
```

`rexx_parse::Controlled::control` is a bare `SymbolId`, so `bind_control`
resolves `AA.1` through `slot_of` and creates a *simple* variable whose name
contains a dot. Under `trace i` the `>C>` line the oracle emits before each
control-variable line is missing for the same reason.

**Not introduced by this task**, and measured rather than argued: the stdout
difference is present on `e72cc19f` too. It became *visible* here, because the
`>=>`/`>V>` lines added are the first thing that prints the control variable's
name at all. Closing it needs a `rexx-parse` change, which no 4b task's file
list reaches.

---

## 7. Test-can-fail checks

Thirteen mutations, each applied to the committed tree, run, and reverted with
`git checkout`. **All thirteen went red; none stayed green.**

| # | mutation | test that went red |
|---|---|---|
| M1 | drop both `>A>` call-site lines | `call_arguments`, `function_call` |
| M2 | `>R>` gated on `intermediates` instead of `results` | `use_arg_alias` |
| M3 | drop the controlled loop's `>>>` pair | `controlled_loop` |
| M4b | drop `EXIT`'s `trace_result` (first occurrence only) | `exit_value` |
| M5 | drop the `>F>` arm in `trace_intermediate` | `function_call` |
| M6 | `eval_argument` evaluates the inner node again (`>V>` returns) | `call_arguments` |
| M7 | drop `USE ARG`'s `>>>`+`>=>` | `call_arguments`, `function_call` |
| M8 | drop `>=>` from `bind_control` | `controlled_loop` |
| M9 | `WITNESSED_PREFIX_COUNT` 13 -> 12 | coverage test |
| M10 | remove `>R>` from `PREFIX_COVERAGE` | coverage test |
| M11 | an owner names a phase not on the list | coverage test |
| M12 | `>K> "ARRAY"` back before the elements | corpus, 39 of 40 |
| M13 | an omitted array element closes the gap up again | corpus, 39 of 40 |

M4's first spelling replaced *two* identical snippets (the `Exit` and `Return`
arms share the same three lines at the same indentation) and so was not a
single-site mutation; M4b is the corrected one, and the fact that the sloppy
version also reddened `function_call` is itself the evidence that `RETURN`'s own
`>>>` is pinned.

Plus the negative control in section 4: all six new witnesses diverge against
the `e72cc19f` binary.

---

## 8. Documentation corrected

A comment that states something false has to be corrected or removed. Five did.

* **`phase-4-exclusions.txt`, "same root cause, same fix, and both close
  together"** (the indent-counter row's pointer at I31). Measurably false now:
  I31 closed alone and nothing about the indent counter changed. Replaced with
  the correction and with why the two are separable, so whoever ports the
  counter expects no help from this fix.
* **DEVIATION 0's "WHAT THIS DOES NOT CLOSE"** and **`static_indent`'s own doc**
  carried the same claim; both corrected.
* **`corpus/lang/condition_traps.rex`'s header** ("`exit <value>` ... would
  diverge") and **`phase-4b.txt`'s entry for it** -- the constraint is lifted.
* **`corpus/lang/signal_forms.rex`'s header** (first pass chosen to route
  around the gap). Re-measured rather than assumed: the second-pass shape
  (`if i = 2`) now agrees with the oracle on all three descriptors, rc 240.
* **`corpus/lang/mutation_controlled_order.rex`'s header** (same shape).
* **`exec_raise`'s "traced before the elements ... the elements produce no lines
  of their own"** -- both halves false, corrected with the transcript.
* **`lib.rs`'s fragment-indent test comment** ("omits two `>>>` lines this crate
  does not yet emit").

The KNOWN GAP row itself is closed and moved to CLOSED DEFECTS with its full
measured record, per that section's own convention.

`rust/crates/rexx-parse/tests/sourceline_oracle/` was regenerated for the three
edited corpus programs and the one new one, using that file's own documented
driver -- editing a corpus program's comments changes the committed
`SOURCELINE()` expectation, which is how `cargo test --workspace` caught it.

---

## 9. Concerns

1. **A `SIGNAL ON SYNTAX` handler reached from inside a callee prints at the
   callee's indent.** Pre-existing (DIFF on `e72cc19f` too), unrelated to this
   task, and covered by DEVIATION 0 for both harnesses, so no gate sees it. It
   is an indent divergence rather than a content one, which is why it has no
   KNOWN GAP row -- but a reader could reasonably want one, and I did not add
   it because DEVIATION 0's own "WHAT SURVIVES" paragraph already governs that
   class. Transcript: a `trace i` program whose callee raises into a
   main-body handler prints the handler's clauses at 0 on the oracle and at 2
   here.

2. **`TRACE L` is a real divergence with no owner.** Recorded, not fixed, for
   the reason in section 6. It is the cheapest remaining trace-surface gap and
   would suit a small dedicated task with the `TRACE()`-reporting question in
   its scope.

3. **The compound control variable is a `rexx-parse` change.** It is a *stdout*
   divergence, not a trace one, so it is a correctness gap rather than a
   cosmetic one -- worth an owner in 4c rather than sitting in KNOWN GAPS
   indefinitely.

4. **`>A>` has one reachable producer left unprobed**: `ExpressionList`
   (`ExpressionList.cpp:144`), the `(a, b)` list expression, which is Phase 5's
   and fails loudly here. Nothing in 4b can reach it, so it is not a gap; it is
   noted because `PREFIX_COVERAGE` marks `>A>` as witnessed and that claim is
   about 4b's producers.

5. **The corpus denominator moved from 39 to 40.** Deliberate -- the RAISE
   substitution hole is only observable in an untrapped report, whose absolute
   path makes it unsuitable for a committed expectation, so a live corpus
   program is the only instrument that can hold it.

---

# Fix round 1 -- review findings F1, F2, F4-F9

**Commit `94237403`** (`git log`: `94237403bb020bbc9f613168f47efd68d3df5a7f`,
"Task 9 review round 1: F1, F2, F4-F9"), on top of `233fbd8b`. F3 was the
controller's and landed as `38b2cb7b` before this round started.

Gates, each exit status read unpiped:

| gate | before | after |
|---|---|---|
| `cargo test --workspace` | 986 / 0 | **989 passed / 0 failed** |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 40 of 40 | **41 of 41**, STRICT |
| `cargo test -p rexx-exec --test assertions` | 4224 / 4259 | **4224 / 4259** |
| `cargo fmt --all --check` | 0 | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | **0** |

Two findings were closed with code rather than recorded, and both because
measuring them made the fix small. The rest are the named sentences,
corrected where they sit.

---

## F2 -- closed, and it was cheaper than the row would have been

**The measurement first.** `trace i` / `do ii = 1 to 3 ; ii = 10 ; end ;
say ii`:

```
oracle stdout: 11        233fbd8b: 4        e72cc19f: 4
       >V>     II => "10"        >V>     II => "1"
       >>>     "10"              >>>     "1"
       >>>     "11"              >>>     "2"
       >=>     II <= "11"        >=>     II <= "2"
```

The oracle reads `10` back, adds `1`, and `11 > 3` ends the loop after **one**
pass. We ran three. `DoBlock::checkControl`'s `control->evaluate` is a real
read of the variable -- which is *why* it traces `>V>` at all -- where this
crate rendered `LoopState::Controlled::current`, a value no body can reach.

**Cost, measured rather than estimated.** The fix is four lines in
`loop_advance`: read through `read_by_name` (exactly where `bind_control`'s
own write goes), `arith_operand` it, add `BY`. `current` is now only ever the
header's value, used on the first pass and never again.

**Two adjacent shapes came free**, and I probed them rather than assuming:

* a body that `DROP`s the control variable reads the derived name and fails
  41.1 on `("II")` -- `read_by_name`'s own miss answer;
* a body that assigns a non-numeric fails 41.1 on `("abc")` --
  `arith_operand`'s own raiser.

**Corrected at round 2 -- this sentence read "Both match byte for byte now"
and was an overclaim.** The `DROP` half matches only when `NOVALUE` is not
trapped; with `signal on novalue` armed the re-test raises `NOVALUE`, which
the reader used here could not express. See the round-2 section (NEW-1).
Both are what made the review's "milder instance" not mild at all: **every one of these failures was blamed on the
wrong clause.** The oracle blames whichever clause transferred control back
to the loop, at the loop body's indent:

```
fall-through:            4 *-*   end        Error 41 ... line 4
an ITERATE in the body:  5 *-*   iterate    ... line 5
the same ITERATE two blocks deeper:  still line 5, still indent 2 --
    its own lexical indent is 8, so `LeaveOrigin::indent` is the wrong
    number and `loop_indent` is the right one
```

`HeaderClause::Iterate` now carries the `ITERATE`'s own echo site, because
`LeaveOrigin` captured it a pass earlier and nothing else can recover it
afterwards. The overflow case the review named
(`do ii = 9E999999999 by 9E999999999 to 9E999999999`, oracle line 4, ours
line 2) is fixed by the same change and is byte-identical now.

**A/B against `233fbd8b`, ten new probes plus the six F2 shapes:**

* six F2 shapes: **all six DIFF -> MATCH** (`a1` stdout+stderr; `a2`/`a3`/
  `a4`/`a5` rc 215 vs 0 before; `a6` the overflow line).
* ten adjacent shapes: **six DIFF -> MATCH, four MATCH on both, 0 MATCH ->
  DIFF.** The four that match on both are the ones that pin the rule rather
  than the fix -- `numeric digits 3` with a fractional `BY` accumulating over
  four passes, a `WHILE` whose body re-assigns the control variable to its
  own value, a `FOR` loop left by `LEAVE`, and `ii = ii || ''` (which turns
  the control variable into a string that still converts). The six that
  changed: a body incrementing the control variable, a *callee* incrementing
  it through the shared pool, an `ITERATE` after a write, a nested loop whose
  inner body writes its own control variable, a negative `BY` with a write,
  and an `INTERPRET` doing the write.

**Witnesses.** `tests/trace_oracle/control_variable_reread.rex` (the read-back
and the trip count, with an ordinary loop beside it as the adjacent passing
case -- a witness with only the first loop would pass against an
implementation that read the variable and broke the ordinary case) and
`corpus/lang/loop_retest_blame.rex` (the blame line, live, untrapped because
the blame is only observable in the report; its `ITERATE` is two blocks deep
so that "the ITERATE's own indent" and "the loop body's indent" are different
numbers).

---

## F8 -- closed, because `mode_from_setting` is in this task's own file list

`TRACE L` echoed nothing here. The oracle echoes every **executed** `LABEL`
clause, with the ordinary `*-*` prefix, and nothing else. Measured across all
three ways a label is reached, in one program:

```
     3 *-* fellthrough:      <- fallen into
    10 *-*   sub:            <- a CALL target, at the callee's indent
     8 *-* there:            <- a SIGNAL target
```

**Read from the C++ rather than generalised from those three.**
`RexxInstructionLabel::execute` (`LabelInstruction.cpp`) traces through
`traceLabel` and nothing else; `traceLabel`'s gate is `tracingLabels()`; and
`TraceSetting.cpp:52`-`54` sets that flag in `traceAllFlags`,
`traceResultsFlags` and `traceIntermediatesFlags` as well as in
`setTraceLabels`. So `TraceMode` gains a fourth field that is **true wherever
`all` already is**, and the echo site's gate becomes
`all || (labels && is_label)`, which reduces to `all` in every mode but `L`.
That property is asserted directly in `trace.rs`'s own letter test, and it is
what keeps the blast radius to one letter rather than four.

The row moved from KNOWN GAPS to CLOSED DEFECTS. **`TRACE ?L` did not move
with it**: measured after the fix, `trace ?l` now agrees on the label line and
differs by exactly the two `+++` banner lines the `TRACE ?` row already owns,
so that row is unchanged and still owner-unassigned.

Eight `TRACE L` probes: seven MATCH, one (the `trace ?l` one) DIFF for that
pre-existing reason only. All eight were DIFF at `233fbd8b`.

Witness: `tests/trace_oracle/trace_labels.rex`. Its silent half is the point --
the `DO`, `SELECT` and `IF` between the labels produce no line, so "echo
everything under L" fails it just as "echo nothing" does.

---

## F1 -- corrected, and then pinned, which the finding said a test could not do

The finding is right about the mechanism and I have not disputed any of it:
`check_witness` and `corpus.rs` both compare through `normalize_stderr`, so no
`.expected` file and no corpus program can see a trace line's indent. Both
false sentences are replaced, and the module doc now states the limitation
positively rather than leaving it to be rediscovered.

**But "no test can pin it" was true only of those two harnesses.** A unit test
asserting `interp.trace` raw is outside either comparison function -- the same
instrument DEVIATION 0's own "WHAT SURVIVES" paragraph already names for the
shallow-depth indent witnesses. So rather than only correcting the comments, I
added `run.rs`'s
`task_9s_two_new_indents_are_the_oracles_own_and_normalisation_cannot_see_them`,
which asserts two full transcripts byte for byte (both captured from the
oracle with `cat -A`): a `Controlled` loop's setup `>=>` at the `DO`'s indent
against its re-tested pass's four lines two columns in, and `>F>` at the
caller's indent with the callee's `>>>` for the same value one line above it
and two columns further in.

**It goes red under exactly the two mutations the review used** to show the
witnesses could not (below). So F1's code was right, its comments were wrong,
and the gap the finding identified is now closed rather than only disclosed.

---

## F4-F7, F9

* **F4** -- `trace_oracle.rs:72`'s "all five ... the ten prefixes" is now
  "every witness ... the thirteen prefixes claimed". The module doc's
  "two witnesses carry no prefix of their own" paragraph is now three, since
  `control_variable_reread.rex` joins them -- and it is a different kind of
  content difference from the other two, which the paragraph now says: a value
  line that was present and **wrong**, not one that was absent.
* **F5** -- `trace_alias`'s "under `trace l` shows nothing at all" now reads
  "shows **no `>R>`**", and the reason it was wrong (the label echo) is no
  longer a gap at all after F8, so the sentence names the measurement rather
  than pointing at a row that has moved.
* **F6** -- `eval_argument`'s older sentence said "by evaluating the inner
  expression"; it now says "an expression", with the paragraph twelve lines
  below named as the one that says which.
* **F7** -- correcting it here, since a commit message cannot be amended: the
  `RAISE ... ARRAY` substitution-hole fix shows on **stderr** (the untrapped
  40.4 report), not stdout. `233fbd8b`'s message says "a stdout content
  difference"; the report body and the corpus program's own header both use
  the hedged "stdout/stderr", which is right, and the exclusions row says
  "stdout/stderr content difference" too.
* **F9** -- the duplicated gate in `bind_control` is kept and now says why it
  is not a second decision (`trace_assignment` carries the real one; this one
  exists so two `Vec`s are not built under `TRACE OFF` on every loop pass) and
  says so *at* the site, so a future divergence between the two cannot hide.

---

## Test-can-fail checks, this round

Seven mutations, each applied to the tree, run, and reverted. **All seven went
red.**

| # | mutation | test that went red |
|---|---|---|
| R1-M1 | `bind_indent` collapsed to `loop_indent` (the review's own F1 mutation) | `task_9s_two_new_indents...` |
| R1-M2 | `>F>` emitted at `indent + 2` (the review's second F1 mutation) | `task_9s_two_new_indents...` |
| R1-M3 | control variable read from the loop's saved value again | `control_variable_reread` |
| R1-M4 | a failed re-test blamed on the `DO` clause again | corpus, 40 of 41 |
| R1-M5 | `L` classified back to `TraceMode::OFF` | `trace_labels` |
| R1-M6 | `tracing_clause` ignores `is_label` (L echoes nothing) | `trace_labels` |
| R1-M7 | `tracing_clause` ignores `is_label` the other way (L echoes everything) | `trace_labels` |

R1-M6 and R1-M7 are the pair: the witness fails in both directions, which is
what makes it a witness for "labels only" rather than for "some output".

**One incident worth recording, because it cost real work.** The mutation
harness I used in round 0 reverted with `git checkout -- FILE`. That is a
revert only when the file is committed. Run in this round *before* committing,
its first invocation discarded every uncommitted change in `run.rs` -- the F2
fix, the F6/F8/F9 edits and the new unit test -- and its second invocation
then reported "STAYED GREEN" for a test that no longer existed, which is what
made the loss visible. The work was redone from this session's own record and
re-verified against every probe; the harness now saves a copy before editing
and restores from that, so it reverts to whatever was on disk rather than to
whatever was committed. The false green is the part worth keeping: a mutation
harness that cannot tell "the test passed" from "the test is gone" is itself a
test that cannot fail, and `cargo test <name>` exits 0 on zero matches.

---

## Concerns, updated

1. **`SIGNAL ON SYNTAX` handler indent** (round 0's concern 1) is unchanged
   and still pre-existing, still indent-only, still covered by DEVIATION 0.
2. **The compound control variable** (round 0's concern 3) is unchanged and
   still needs a `rexx-parse` change and an owner. F2's re-read does not
   touch it: `bind_control` and `read_by_name` now agree on the wrong slot
   rather than only writing to it, so the divergence is exactly as measured
   before.
3. **`TRACE ?` remains the only unowned trace-surface gap**, and after F8 it
   is the whole of what `trace ?l` diverges by.
4. **The corpus denominator moved 39 -> 40 -> 41.** Both additions are
   programs whose subject is only observable in an untrapped report, which a
   committed expectation cannot hold because it contains an absolute path.

---

# Fix round 2 -- re-review findings NEW-1 through NEW-6

**Commit `f02c3ed3`** (`git log`: `f02c3ed30dfe5f7360d7b7c6a9f8bdf6588ff79e`,
"Task 9 review round 2: NEW-1 through NEW-6"), on top of `94237403`.

Gates, each exit status read unpiped, each run count read rather than
inferred:

| gate | before | after |
|---|---|---|
| `cargo test --workspace` | 989 / 0 | **990 passed / 0 failed** |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 41 of 41 | **41 of 41**, STRICT |
| `cargo test -p rexx-exec --test assertions` | 4224 / 4259 | **4224 / 4259** |
| `cargo fmt --all --check` | 0 | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | **0** |

**Every claim about a number, a count or "every other site" in this round has
a command behind it, and the command is named in the text it justifies.** That
was the instruction, and it is also the diagnosis: NEW-3 and NEW-5 exist
because I wrote a gate number from memory and asserted a cross-site pattern
without grepping for it, in the same commit whose purpose was to correct three
comments of exactly that kind.

---

## NEW-1 -- a defect round 1 introduced, and the ordering under it

`read_by_name` returns the derived name on a miss and **reports nothing to
its caller**. The oracle's `control->evaluate` is a full evaluation, so a
dropped control variable raises `NOVALUE`. Measured, `signal on novalue name
nv` around `do ii = 1 to 3 ; drop ii ; end`:

```
oracle:  stdout "handler II", rc 0
94237403: stderr "Error 41.1 Nonnumeric value (\"II\")", rc 215
```

The fix is `Interp::read` + `novalue_check`, the pairing every other read site
in this crate already uses. Picking `read_by_name` in the first place was a
type-shaped mistake, not a semantic judgement: it took a `&[u8]` where
`bind_control` also took one, and that similarity is what made it look right.

**The ordering is measured.** Under `trace i` the oracle's failing re-test
emits the `DO` re-echo and then nothing at all -- no `>V>`, no `>>>` -- because
the raise happens inside the evaluation and never reaches `traceResult`. So
`novalue_check` runs *before* the two trace calls. That is not a comment-only
claim: **R2-M2** moves the check after them and the witness goes red, while
stdout and rc would still agree. Without that mutation the ordering would have
been an unpinned assertion of exactly the kind NEW-3 and NEW-5 are.

**Adjacent shapes, eight probes, all MATCH** (`v01`-`v08`, run against the
oracle with three descriptors from a directory `mkdir`'d for this round):
untrapped `DROP` still 41.1 on `("II")`; a trap armed but never fired; `SIGL`
in the handler; `signal off novalue` inside the body; a `DROP` followed by a
re-assignment in the same body; the trap armed in a *caller* while the loop
runs in a callee; a `DROP` on the second pass only; and `DO OVER` with a
dropped control variable.

One probe was thrown away rather than reported: my first `v07`
(`drop ii ; ii = 2`) **does not terminate on either side** -- the control
variable oscillates -- so its only difference was which side my own `timeout`
killed first. Replaced with a terminating shape. A non-terminating probe that
reports a divergence is a false positive waiting to be believed.

Witness: `tests/trace_oracle/control_variable_novalue.rex`, whose first loop is
the adjacent passing case (a body that leaves the control variable alone must
*not* trap) and which prints `SIGL` rather than a flag.

The overclaim is corrected in both places it appeared: the round-1 section of
this report (in place, marked) and `phase-4-exclusions.txt`'s own row.

---

## NEW-2 through NEW-6 -- five false statements, each fixed against a command

| # | the false statement | the command that settled it | what it says now |
|---|---|---|---|
| NEW-2 | `run.rs` listed `L` among the letters landing on `TraceMode::OFF` | `grep -n 'C\`/\`L\`/\`E\`' crates/rexx-exec/src/run.rs` -> one hit | `C`/`E`/`F`/`N`/`O`, with a note that `L` left the list at round 1 |
| NEW-3 | "leaves this file green and the workspace at 986/0" | `cargo test --workspace` -> 989 at that commit, and the mutation makes it 1 failed | "leaves this file green", full stop, plus why the workspace claim contradicts the next paragraph |
| NEW-4a | table row "`>>>` \| every witness below" | `grep -c '>>>' tests/trace_oracle/*.expected` -> `trace_labels.expected` is **0**, the other twelve non-zero | "every witness below **except `trace_labels.rex`**", and **asserted** |
| NEW-4b | "Three witnesses below carry no prefix of their own" | the same listing -> five witnesses now qualify | **no count at all**; the kinds are listed and `WITNESS_PREFIXES` is named as the list |
| NEW-5 | the pre-gate is a shape "every other tracing site in this file uses" | `grep -c 'tracing_intermediates()' crates/rexx-exec/src/run.rs` -> **1**, the site itself | "the only pre-gate of its kind in this file", with the wrong claim recorded and `step`'s `Assignment` arm's real shape stated |
| NEW-6 | `TraceMode::OFF`'s doc still named `setTraceLabels` | reading the two adjacent paragraphs | the name is out of the list, and the round-1 half-correction is described rather than silently replaced |

**NEW-4 got an assertion, not just a correction.** "Every witness below" is the
exact shape of prose that went stale the moment `trace_labels.rex` landed, and
prose cannot notice. `every_witness_still_emits_every_prefix_it_is_named_for`
now also asserts that every `WITNESS_PREFIXES` entry claims `*-*`, and that the
set claiming no `>>>` is exactly `["trace_labels"]`. **R2-M3** and **R2-M4**
show both go red. That is the same "turn the prose into data" move this file
made once before, after the H3 branch review; the count-carrying sentence
beside the table is now written to carry no count, because a count is what
keeps rotting.

While correcting these I swept for the same class rather than only the six
named: `grep -rn "986\|CLEFNO\|C\`/\`L\`"` over `rust/crates/` and `docs/`
found no further instances, and the one historical count I left
(`trace_oracle.rs`'s H3 note, "still passed all five tests") is now explicitly
"all five tests **that existed then**".

---

## The two parkables -- recorded where a reader meets them, not as new rows

**The under-indent after a completed inner loop.** Reproduced exactly as the
re-review describes (`trace r`, a plain `DO` around `do jj = 1 to 1`, then
`say 'after'`: oracle echoes at 0 and its `>>>` at 0, we print 2 and 2, with
nothing raising). **But it is not unrecorded** -- `phase-4-exclusions.txt`'s
indent row states the rule ("later clauses print two spaces lower than their
lexical depth") and I re-ran that row's own example to confirm it is the same
mechanism. What was missing is why a careful reviewer could not find it: every
example in that row *raises*, so a reader looking for "my ordinary trace output
is indented wrong" does not recognise it. The row now carries the ordinary
`trace r` shape beside the raising one, and says that is why. A duplicate row
would have been the worse fix -- two records of one divergence drift apart.

**`TRACE()`.** Measured: `say trace()` gives the oracle `N` at rc 0 and this
crate `rexx-exec: routine "TRACE" is not implemented (4c)` at rc 120. The
coupling argument that named it is gone with the `TRACE L` row, so that half is
moot. But the observation found something still live: DEVIATION 0's own list of
what "stays byte-exact in behaviour" opens with "TRACE() is readable by the
program", which is a property of the *oracle*, not of the agreement. Corrected
there, with the transcript.

---

## Test-can-fail checks, this round

Four mutations, each applied, run, and reverted **from a copy** -- never
`git checkout --`, per the rule now in `rust/CLAUDE.md`. Each run count was
read, not just the exit status.

| # | mutation | result |
|---|---|---|
| R2-M1 | the re-read uses the `NOVALUE`-blind reader again | `trace_oracle` `21 passed; 1 failed` |
| R2-M2 | `novalue_check` moved *after* the value lines | `trace_oracle` `21 passed; 1 failed` |
| R2-M3 | a witness stops claiming `*-*` | `every_witness...` `0 passed; 1 failed; 21 filtered` |
| R2-M4 | a second witness stops claiming `>>>` | `every_witness...` `0 passed; 1 failed; 21 filtered` |

R2-M2 is the one that matters most: it separates "raises the condition" from
"raises it at the right moment", and only stderr can tell them apart.

`git status --porcelain` is empty of source changes after every mutation, and
the full suite was re-run afterwards at 990/0 to confirm the tree is the
committed one.

---

## What I would tell the next round

The named pattern is real and I can state its mechanism: **when I correct a
comment I am writing about the code's *context* -- what other sites do, what
the gates say, how many things there are -- and that context is exactly the
part I cannot see from the line I am editing.** The fix that works is not more
care; it is that a sentence about context needs a command, in the same way an
assertion needs a mutation. Three of the six findings this round were
countable claims (`986`, "every witness", "every other tracing site"), and all
three would have been caught by the grep that took me under a minute once I
ran it.

Where I could, I removed the class rather than the instance: the count beside
the prefix table is gone rather than corrected, and the two "every witness"
rows are assertions now.

## Concerns, updated

1. **`SIGNAL ON SYNTAX` handler indent** and **the compound control variable**
   are unchanged from round 0's concerns 1 and 3.
2. **The under-indent after a completed inner loop** now has a findable
   record but is still open, still indent-only, still DEVIATION 0's.
3. **`TRACE ?`** remains the only unowned trace-surface gap.
4. **Nothing in this round changed behaviour outside `LoopState::Controlled`'s
   re-tested pass.** The 40+ probes from rounds 0 and 1 were all re-run at the
   end of this round: unchanged, with the same two pre-existing DIFFs
   (`w19`'s handler indent, `d03`'s `TRACE ?` banner).

---

# Fix round 3 -- NEW-F1, NEW-F2, and the class

**Commit `50da3045`** (`git log`: `50da304533f0f9ab0c2b3ae74de6a179a94687bc`,
"Task 9 review round 3: NEW-F1, NEW-F2, and two the sweep found"), on top of
`f02c3ed3`.

| gate | before | after |
|---|---|---|
| `cargo test --workspace` | 990 / 0 | **990 passed / 0 failed** |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 41 of 41 | **41 of 41**, STRICT |
| `cargo test -p rexx-exec --test assertions` | 4224 / 4259 | **4224 / 4259** |
| `cargo fmt --all --check` | 0 | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | **0** |

No behaviour changed this round. Every probe set from rounds 0-2 was re-run
against the oracle at the end: unchanged, with the same two pre-existing DIFFs
(`w19`'s handler indent, `d03`'s `TRACE ?` banner).

---

## NEW-F1 -- not updated from 1 to 2; the survey is gone

The comment quoted `grep -c 'tracing_intermediates()' ...` and gave the answer
`1`. Committing it made the answer `2`, because the comment's own text became
part of the corpus being counted.

I did not update the number. **The claim was not load-bearing** -- the point of
that comment is "this check decides whether to *build* the two `Vec`s, not
whether to print; `trace_assignment` holds the real gate". The census was
rhetorical support I added in round 1 *because* the reviewer had just caught me
generalising, and it has now been wrong twice in opposite directions (round 1
said other sites share the shape; round 2 said this is the only one).

What replaces it is a measurement of the thing the gate is actually for.
`do ii = 1 to 2000000` with a one-statement body, `TRACE OFF`, release build,
three runs each:

```
with the pre-gate:     3.13  3.13  3.15   s
without it:            3.22  3.22  3.23   s
```

about 40 ns per pass, which is the two allocations. So the gate stays, on a
number rather than on a pattern -- and a benchmark result about a fixed program
does not change when the file around it does.

## NEW-F2 -- the enumeration, then the durable half of it

Confirmed by running it rather than reading: of the four `self.read(code, ...)`
sites, one pairs with `novalue_check`, two bind the flag to `_` deliberately,
and the fourth is the one this task added. So "every other read site already
pairs with it" was 1 of 3.

Pasting the census would have fixed the falsehood and kept the liability --
line numbers move, callers get added. What the paragraph says now is the part
that does not move: **pairing is a per-site decision, not a convention**,
because a lookup that must not raise is a different operation from an
evaluation that must. That is the half a reader needs, and it stays true however
many callers there are.

The adjacent LESSON paragraph had the same shape -- it justified the two readers
by naming the constructs each was "for", and named the wrong one (the other
caller is in `stem.rs`, not `PROCEDURE EXPOSE`). It now argues from the
**signatures**: one reader returns a bare value and cannot report an unset read,
the other returns the value alongside a flag that says so. The compiler keeps
that honest; prose about callers does not.

## Two more of the same class, found before the review this time

I swept the lines this task *added* -- `git diff e72cc19f | grep '^+'` filtered
for survey phrasing -- rather than re-reading the files. Two genuine instances,
both fixed in this commit:

* **`run.rs`'s round-1 paragraph still said the read goes "through
  `read_by_name`"**, and that `read_by_name`'s miss answer produces the `DROP`
  case. Round 2 changed the reader and left the sentence behind, so the arm
  carried a paragraph contradicted by the paragraph below it. Same defect as
  NEW-F2, one round earlier, on the same twenty lines.
* **`phase-4b.txt`** said this task's corpus program runs under `trace i`,
  "the one program in this subset that is, which is what every other 4b program
  uses". Enumerating the subset (each listed program's first `TRACE` clause)
  showed the second half **already false**: four of the others set no trace at
  all. The sentence now gives the reason (`>A>` is intermediates-level, so
  nothing shows it under `trace r`) and no census.

---

## The class -- which mechanism, and why

**The distinction is not prose versus assertion.** It is *what the sentence
points at*:

* **Immutable referents** -- the oracle's bytes for a fixed program, a cited
  C++ line, a benchmark number, a past measurement. Across this whole task,
  in four commits and several hundred lines of comment, **not one of these has
  needed correcting.**
* **Mutable repo aggregates** -- how many call sites there are, which tests
  cover what, what the gate totals are, what "every other site" does. **Every
  false statement in this task has been one of these.** Eight now: the two
  round-0 "a witness pins this" claims, `986/0`, "`>>>` | every witness below",
  "Three witnesses", "every other tracing site", "every other read site", and
  the two I found myself this round.

That reframes the remedy. "Put a command behind every countable claim" was the
round-2 rule and it produced NEW-F1 directly: the command's search term joined
the corpus it measured. A rule about *how to evidence* a claim cannot fix a
claim that should not be in prose at all.

**The decision procedure I am adopting, in order:**

1. **Is the referent immutable?** (oracle behaviour at a fixed program, a C++
   citation, a measured number.) Then prose is fine and durable. Most of what a
   comment should say is this.
2. **Mutable repo aggregate, and load-bearing?** Make it an **assertion**. This
   is the one mechanism in this task with a clean record: the `PREFIX_COVERAGE`
   counts (round 0) and the two "every witness below" rows (round 2) have been
   through four commits and three reviews without needing correction, and both
   go red under mutation. It is worth the code exactly when the claim is doing
   work.
3. **Mutable repo aggregate, not load-bearing?** **Delete it.** Both of this
   round's findings were here, and so were both of my own. None of the four was
   holding anything up: each was decoration added to a claim that already stood
   on its own measurement.

And one form rule that generalises NEW-F1 beyond grep: **a claim must not be
falsifiable by the act of committing it.** Quoting a search term inside the
file being searched is the sharpest case; "the only site that does X" written
at the site that does X is the same shape one step removed.

**Why I am not adding a lint.** A test that greps comments for banned phrasing
would false-positive on the legitimate immutable statements -- "all five tests
that existed then", "all three ways a label is reached" -- and I would end up
hedging the lint. That is the same failure one level up, and this task has
enough evidence that my hedges are where the falsehoods live. The mechanism
that has actually worked is narrower and I would rather keep it that way:
assert the load-bearing surveys, delete the rest.

**Proposed for `rust/CLAUDE.md`**, if the coordinator wants it there (I have
not edited that file):

> A comment may state what the oracle does, what the C++ does, and what was
> measured. It may not state how many call sites there are, what every other
> site does, or what a gate currently totals -- those change without the
> sentence being reread. If such a claim is load-bearing, assert it in a test;
> if it is not, delete it. Never quote a search command inside the file it
> searches.

---

## Test-can-fail checks, this round

No behaviour changed, so there is nothing new for a test to pin -- and saying
so is the honest answer rather than inventing one. What was checked instead:

* The two comment fixes were verified by re-running the commands that falsified
  them: the literal that gave `2` now gives `1` (one real call site, no quoted
  term), and the read-site enumeration is the one pasted above.
* The benchmark behind the kept pre-gate was run six times (three with, three
  without) on a release build, with the file restored from a copy afterwards
  and `cmp` confirming the restore.
* Rounds 0-2's mutations were not re-run here; the previous two re-reviews
  re-ran eleven of them between them and found none green.
* Full suite after every edit, and every probe set from rounds 0-2 re-run
  against the live oracle at the end.

## Concerns, updated

1. Rounds 0-2's concerns are unchanged: the `SIGNAL ON SYNTAX` handler indent,
   the compound control variable, the completed-inner-loop under-indent, and
   `TRACE ?` -- all pre-existing, all recorded, none this task's to close.
2. **The one thing I would flag for round 4's reviewer**: the two claims I
   classified as "immutable" but which are really claims about the *language*
   rather than about the oracle at a fixed program -- "all three ways a label
   is reached" and "all nine of `check_trace_setting`'s accepted letters". Both
   are measured and I believe both are exhaustive, but exhaustiveness over a
   language is a weaker guarantee than a transcript, and if a seventh instance
   is going to come from anywhere it is one of those two.

---

# Fix round 4 -- the fourth label route, and the rule

**Commit `16a67ae2`** (`git log`: `16a67ae29409d21d65c12e9853c3e9dc5fdf2556`,
"Task 9 review round 4: the fourth label route"), on top of `50da3045`.

| gate | before | after |
|---|---|---|
| `cargo test --workspace` | 990 / 0 | **990 passed / 0 failed** |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 41 of 41 | **41 of 41**, STRICT |
| `cargo test -p rexx-exec --test assertions` | 4224 / 4259 | **4224 / 4259** |
| `cargo fmt --all --check` | 0 | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | **0** |

**The counts do not move**, and that is worth stating rather than leaving to
inference: the new route extends an existing witness (`trace_labels.rex`)
rather than adding a test or a corpus program, so there is no new `test
result:` line and no new corpus entry. `trace_labels.rex` lives in
`tests/trace_oracle/`, not `corpus/lang/`, so `sourceline_oracle` does not read
it and no expectation there needed regenerating.

---

## 1. The route claim -- corrected, and not replaced by a bigger number

I reproduced the reviewer's finding before acting on it: `trace l` with
`zz = sub()` echoes `     5 *-*   sub:` on the oracle, and this crate is
byte-identical on all three descriptors.

The claim appeared in four places (`trace.rs`'s `labels` field doc and its
`LABELS` const, `trace_oracle.rs`'s test doc, `trace_labels.rex`'s header, and
the exclusions row). **None of them now states a count.** What they state
instead is the property with its enumerating citation:

> A label echoes when the `LABEL` instruction is **executed**.
> `RexxInstructionLabel::execute` calls `traceLabel` and nothing else -- one
> C++ site that enumerates the whole condition. Nothing anywhere enumerates
> the ways control can arrive, so nothing here counts them.

The routes are now presented as *probed examples*. `trace_labels.rex`'s header
says so explicitly and names the four it exercises, with the new one marked as
the route that was missing and why the file was reopened.

**No behaviour was wrong at any point.** This crate matched the oracle on the
function-call route before anyone knew it was a route, because the echo gates
on the instruction's kind (`matches!(instruction.kind, InstructionKind::Label
{ .. })`) and not on how control arrived. The defect was entirely in the
sentence. That is worth recording in the exclusions row, and it is: a false
comment with correct code underneath is still the thing that took four review
rounds to find.

## 2. The witness

`trace_labels.rex` gains `say 'function call returns' zf()` and a `zf:` routine.
Regenerated from the live oracle; stdout, stderr and rc all byte-identical
(`cmp` on all three). The new line is `    56 *-*   zf:`, at the callee's own
`+2` indent exactly like the `CALL` target beside it.

Two mutations, restored from copies, run counts read:

| # | mutation | result |
|---|---|---|
| R4-M1 | `tracing_clause` ignores `is_label` (the mode goes silent) | `0 passed; 1 failed; 21 filtered` |
| R4-M2 | the `zf:` line deleted from the committed expectation | `0 passed; 1 failed; 21 filtered` |

R4-M2 is the one that answers "does the witness actually cover the new route" --
it shows that specific line is part of what the comparison reads.

**What no mutation can show, and I would rather say it than fake it:** there is
no route-specific branch in the implementation, so no mutation can break the
function-call route while leaving the other three working. The extended witness
buys coverage against a *future* implementation that grows such a branch -- the
reviewer's own framing, "a route nothing covers is where the next divergence
hides" -- not against a defect reachable today. Manufacturing a mutation to
claim otherwise would be the kind of test-shaped decoration this task has spent
four rounds removing.

## 3. The optional restore -- declined, and why

Round 3's deletion took a true sentence with the false one: `step`'s
`Assignment` arm builds `rendered` unconditionally because both `trace_result`
and the write need it. I checked it is still true (`run.rs:957`-`958`).

**Not restored.** Under the procedure I set out last round it is case 3: a claim
about another site in this repo (mutable referent), and no longer load-bearing,
because the pre-gate it used to support is now justified by a benchmark that
stands alone. "It was true when written" is not the criterion that has kept
sentences alive in this task -- every one of the eight false statements was true
when written. Restoring it would re-introduce the class for sentimental reasons.

---

## 4. The rule -- my read

**The provisional wording is right, and it is the right split.** "An
exhaustiveness claim is only as good as the enumeration behind it -- cite the
site that enumerates, or do not claim exhaustiveness" gets both headline cases
correct, and it gets them correct for the right reason rather than by
coincidence: nine letters survived because `parseTraceSetting`'s switch *is* the
enumeration, three routes died because nothing was.

**I would add one clause, on evidence from this task rather than taste.** The
rule as written licenses prose whenever *something* enumerates -- and
`WITNESS_PREFIXES` enumerates the witness set, yet "`>>>` | every witness below"
rotted anyway (NEW-4), because the enumeration is a `const` in the file next
door and it changed when `trace_labels.rex` landed. The C++ switch cannot change
under us; a Rust table three hundred lines away can, and nothing makes anyone
reread the prose when it does.

So the axis that actually predicts survival is not only "does an enumeration
exist" but "**can it change without the sentence being reread**":

> A comment may state what the oracle does, what the C++ does, and what was
> measured -- those referents do not change under us.
>
> An exhaustiveness claim ("every X", "the only Y", "all N ways") needs an
> enumerating site cited. If that site is **outside** this repo -- a C++ switch,
> the oracle's own table -- prose is fine. If it is **inside** this repo, assert
> it in a test instead: a const or a set of call sites changes without the
> sentence being reread.
>
> If nothing enumerates, do not claim exhaustiveness. Describe the property.
>
> And never quote a search command inside the file it searches.

That is three lines longer than the provisional version and each clause has a
case behind it in this task: paragraph 1 is the round-3 taxonomy (zero
corrections in four commits), paragraph 2's *outside* half is the nine letters
and its *inside* half is NEW-4, paragraph 3 is the three routes, paragraph 4 is
NEW-F1. I would be content with the shorter version if you prefer it -- it
catches the case that has cost the most -- but the inside/outside distinction is
the one that would have caught NEW-4, which the shorter version permits.

**One process note that belongs with the rule.** For this round's sweep I used
exact literal search rather than semantic search, deliberately: the question was
"does this exact phrase survive anywhere", which is a completeness question over
strings. Semantic search answers a different question well and this one badly.
An enumeration claim needs an enumeration tool.

---

## Concerns, updated

1. Rounds 0-2's standing concerns are unchanged: the `SIGNAL ON SYNTAX` handler
   indent, the compound control variable, the completed-inner-loop under-indent,
   and `TRACE ?`. All pre-existing, all recorded, none this task's to close.
2. **My round-3 risk flag has now been half-resolved and half-vindicated**, and
   the surviving half is where I would look next: "all nine accepted letters"
   survived because a C++ switch enumerates it, which the reviewer verified
   directly. I know of no other exhaustiveness claim left in this task's text --
   the sweep above found none -- but that is an absence of evidence over prose,
   which is exactly the weaker instrument this round was about.
