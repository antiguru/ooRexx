# Task 6 report: trace as explicit instructions in the stream

Branch `plan/rust-rewrite`, four commits on top of `4f98b9dc`.

* `56ab1511` -- measure what a trace setting change costs a promoted clause (cases only)
* `36c36712` -- make the clause echo something a chunk emits rather than asks about
* `9572f527` -- put back what extracting the clause echo charged every clause
* `de05ea58` -- give the compiled emission decision a witness, and correct four claims

Suite: **1378 passed, 0 failed, 4 ignored**, green in dev, dev+STRICT, release and
release+STRICT, with clippy clean from a clean target directory.
Baseline was 1369/0/4. Nine tests added: the dual-engine case file, three golden, two
driver counters, and three for the compiler assertion fix round 1 added.

**Fix round 1 changed one finding of substance and four claims.** The substance is that
nothing pinned that the compiled emission decision is reached in production at all; the
four are a false number at a decision point, two `inline(always)` justifications with the
measurement missing or attributed to the wrong one, a counted assertion list, and a false
reason for not using `datadriven`. Each is marked where it appears.

## The op set, and why it is one op rather than a flag

Two variants where there was one.

| Op | Where | What it does |
| --- | --- | --- |
| `Clause { index, end }` | unchanged position | the clause's **unconditional** half: the clause line, the boundary, the temps frame, the failure site |
| `TraceClause { index }` | the first op of a `Clause` region | the clause's **conditional** half: the `*-*` echo |

`TraceClause` is emitted exactly when `ChunkTrace::echoes` answers yes for that
instruction, so its **presence in the stream is the decision**.
A chunk compiled under a setting that echoes nothing has no such op, and its promoted
clauses pay no gate, no `clause_site` allocation and no line to skip.

**The clause line could not go with it.** It feeds `SIGL`, condition objects and syntax
error messages, so it is semantics rather than trace; that is why the split is two ops
and not one op with a flag, and why `Clause` kept everything else it was doing.

**The echo is the region's *first* op, and `compile` asserts it.** The tree-walker echoes
a clause before it computes anything, so an `IF`'s `>>>` line follows its `*-*` line; an
echo emitted after the `EvalExpr` reverses them and moves no register and no jump.
`assert_trace_ops_open_a_clause_region` checks both halves of that contract on every
compiled body -- inside a region, and at its head -- in the shape
`assert_clause_regions_hold_no_clause_op` already had, with a refusal test for each half
and the neighbouring acceptance that stops a check which refuses every stream from
passing. Fix round 1 added it: the contract was stated in the op's doc comment beside the
function that asserts the *neighbouring* one, and was not itself asserted.

**Who emits it is a type, not a `bool`.** `crate::run::Echo` has two variants: `Gated`
asks the setting in force, `Compiled` says the question was already answered where the
clause unit cannot see it. Two answers rather than two settings of one switch, because
they are different kinds of statement.
`in_stepped_clause` keeps its old signature and passes `Gated`; `in_stepped_clause_with`
is the one the compiled stream reaches.

## Trace as a compilation input, and the cache key

`compile(body, plan, trace: ChunkTrace)`.

`ChunkTrace` is `{ clauses, labels }` -- exactly the two `TraceMode` fields the emitter
reads -- and `ChunkTrace::echoes` is the rule.
`Interp::tracing_clause` now delegates to that same `echoes`, so the compiled answer and
the run-time answer are one function called from two places rather than two copies.

**`compile` takes a `ChunkTrace` and nothing wider, and that is the point.** The
argument *is* half the cache key, so a compiler that could read a `TraceMode` field the
key does not carry would cache a chunk under a name that does not identify it. Narrowing
the argument makes that unexpressible rather than forbidden.

`Interp::chunks` is keyed on `(BodyKey, ChunkTrace)`. Widened rather than evicted:
eviction throws away the chunk a program that toggles `TRACE` is about to want again, so
two settings would mean one compile per change instead of two for the whole run.
`one_body_under_two_trace_settings_is_two_cached_chunks` asserts all three -- `compile`
ran twice, the two chunks are distinct `Rc`s with distinct streams, and asking again
under the first setting gives the *first* chunk back, which is the assertion an evicting
cache fails.

### What proves the key change, and a correction to the brief

**The brief's Step 2b test does not fail against the old key, and that is a consequence
of a decision I made deliberately rather than an omission.** The brief specifies "a body
entered once untraced and once under `trace i`, in that order, in one process; the second
entry must produce trace output", and says a cache still keyed on `BodyKey` alone returns
the untraced chunk and the assertion goes red.

It does not, because the driver also carries a run-time fallback (below), and that
fallback makes a wrongly-keyed chunk produce **correct output by a slower route**. The
case is in the tree (`one body entered untraced and then under trace r`) and it is green
under both keys -- measured, mutation **P4**.

I chose that over matching the brief, because the alternative is a wrong-output defect I
measured on the oracle and could not ship. See "The fallback" below.

So the key is pinned by `one_body_under_two_trace_settings_is_two_cached_chunks`, which
counts compiles and chunk identity rather than reading output. Mutation P4 makes it the
**only** failing test in the workspace: 1372 passed, 1 failed. Against the narrow key the
output stays right and every promoted clause of a re-entered body runs on the gated
fallback, which is exactly the cost D23 exists to remove.

## The fallback, which the brief did not ask for and correctness required

**A `TRACE` executed while a chunk is running changes the setting with nothing consulting
the cache.** The key answers entry; it cannot answer this.

Measured against the oracle before anything was written, from a clean directory:

```rexx
if 1 = 1 then trace r
if 1 = 1 then say 'x'
```

```text
     2 *-* if 1 = 1
       >>>   "1"
     2 *-*   then
     2 *-*     say 'x'
       >>>       "x"
```

The oracle echoes the second `IF`. Both engines agreed with it at `4f98b9dc`, so a
compile-time-only decision would have been a **regression this task introduced**, on a
four-line program.

So the driver reads, per promoted clause, whether the setting in force is still the one
the chunk carries, and a stale clause goes back to the run-time gate --
`Echo::Gated` instead of `Echo::Compiled`, and the region's `TraceClause` op skipped.
**Both directions in one decision**: doing only the first leaves a mid-body `TRACE R`
invisible to every promoted clause after it (mutation P2), and only the second leaves
`TRACE N` unable to switch one off (mutation P3).

Seven programs were measured against the oracle for this, all seven matching on both
engines before and after: trace turned on inside a branch before a later `IF`; turned
off inside a branch; turned on inside a loop body so the change has to reach later
passes; a body entered untraced then under `trace r`; turned on before a `SELECT`; a body
entered under `trace r` that then turns trace off; and `trace i` over a literal in an
assignment.

### Where the check sits, and why it is not a local

The obvious shape is a `run_ops` local refreshed after the ops that can run a `TRACE`.
Both halves of that are wrong here.

* **It charges programs that have nothing to decide.** A loop body enters `run_ops` once
  per pass, so initialising the local costs `emptyloop.rex` -- whose body holds no
  promoted clause at all -- 40.3009 to 40.7259 billion user instructions, 17 per pass.
* **It needs a list of which ops can change the setting**, and that list is an internal
  enumeration of the kind this crate has been burned by. Reading the setting in the arm
  that uses the answer needs no list at all.

Re-measured in fix round 1, against 40.3009 billion before this task: initialised at
every `run_ops` entry it is **40.7259** billion, 17 per pass; read in the `Clause` arm it
is **40.4009** billion, 4 per pass -- and those four are the extra `Op` variant in the
driver's match rather than the read, since `emptyloop` reaches no `Clause` op at all. A
clause that does have an echo to decide pays one comparison, the same one
`in_stepped_clause` used to make for it.

The comment at that decision point said "40.30 billion again", asserting a parity with
the base build that this report's own table denied. It now carries the measured values.

## What this does not reach

* **Interactive trace** (`?` prefix) is control flow rather than an emit and is untouched.
* **`TRACE VALUE expr` and inheritance across activations** are handled by the same two
  mechanisms as any other setting change (key on entry, comparison mid-body) and were not
  probed separately.
* **Under `Generic` the two regimes coexist.** An unpromoted instruction's echo still
  comes from `in_stepped_clause`'s run-time gate, shared with the tree-walker, and this
  task does not change that. It is why `Generic` clauses are byte-identical on both
  engines by construction.
* **Nothing was promoted.** Task 7 owns the first native expression op.

## Measurements

`perf stat -e instructions:u`, release, both engines, `4f98b9dc`'s binary against this
branch's, two repeats each and stable to five significant figures.

**Instruction count is the instrument, and that is a finding rather than a preference.**
`emptyloop.rex`'s wall clock moved about 2% between builds executing an *identical*
instruction count, so a wall-clock-only reading reported a tree-walker regression that
does not exist and would have hidden the two real ones below.

| program | engine | `4f98b9dc` | here | delta |
| --- | --- | --- | --- | --- |
| `emptyloop.rex` (25M passes, `nop` body) | tree-walker | 38,000,858,7xx | 38,000,858,5xx | 0.00% |
| `emptyloop.rex` | ir | 40,300,862,xxx | 40,400,863,xxx | **+0.25%** |
| `ifloop.rex` (3M passes, one promoted `IF`) | tree-walker | 13,197,082,xxx | 13,197,081,xxx | 0.00% |
| `ifloop.rex` | ir | 13,581,085,xxx | 13,632,086,xxx | **+0.38%** |
| `selloop.rex` (400k passes, `SELECT` + two `WHEN`s) | tree-walker | 3,730,008,xxx | 3,730,008,xxx | 0.00% |
| `selloop.rex` | ir | 4,007,616,xxx | 4,029,175,xxx | **+0.54%** |
| `toggle.rex` (300k passes, two `TRACE`s + one `IF`) | tree-walker | 1,495,158,xxx | 1,495,158,xxx | 0.00% |
| `toggle.rex` | ir | 1,568,062,xxx | 1,576,762,xxx | **+0.55%** |

Wall clock, `/usr/bin/time`, interleaved between arms within one sitting, three sittings,
under `ulimit -v 8388608`: `emptyloop` 2.74-2.75s against 2.79-2.82s on the tree-walker
and 2.87-2.88s against 2.91-2.94s on the compiled stream; `ifloop` 1.19-1.21 against
1.17-1.18 and 1.24-1.25 against 1.24-1.25; `selloop` 0.34-0.35 against 0.35 and 0.38
against 0.37-0.38; `toggle` 0.14 against 0.13-0.14 and 0.14 against 0.14. The
tree-walker's `emptyloop` row is the one that contradicts its own instruction count, and
is why the table above is the one to read.

### The untraced-chunk cost

**+0.25% on the per-clause floor, and it is not the trace decision.** `emptyloop`'s body
is one `nop`, so it has no promoted clause and asks the staleness question zero times;
the four instructions per pass are what the new `Op` variant and the extra `Chunk` field
cost the driver's dispatch. I did not chase it further.

**A promoted clause costs about 17 instructions more than before** (`ifloop`, 3M passes
with one promoted clause each; `selloop`, 400k passes with three, giving ~18 each). That
is the staleness comparison -- an index into the activation stack, a `ChunkTrace` built
from the setting, two byte comparisons -- plus the `Echo` branch, against the
`tracing_clause` call it replaced. It is the price of the fallback, and it buys the
mid-body correctness above.

**The tree-walker pays nothing anywhere**, which is the half that had to be true and
initially was not. Two separate costs, and fix round 1 re-measured both because the first
round had attributed them to each other:

* extracting the echo into `Interp::echo_stepped_clause` costs **525,000,000**
  instructions on `emptyloop` without `inline(always)` -- 38.0009 against 38.5259 billion
  on the tree-walker, and 40.4009 against 40.9259 on the compiled stream;
* the four `inline(always)` on `ChunkTrace::of`, `ChunkTrace::echoes`,
  `Interp::tracing_clause` and `Interp::chunk_trace` are worth **100,000,000** between
  them -- 38.0009 against 38.1009, and 40.4009 against 40.5009 -- because without them the
  `ChunkTrace` is materialised at the gate instead of folded away. Measured as a group;
  I did not measure the four individually.

Both numbers are now recorded at the annotations they justify. The first round put the
100 million against `echo_stepped_clause` and justified that annotation with wall clock,
which is the instrument this same report declares unreliable at that magnitude on that
program.

### What a `TRACE` change inside a loop costs

`toggle.rex` is `do i = 1 to 300000; trace l; if i > 0 then nop; trace o; end` -- two
setting changes and one promoted clause per pass, and neither setting echoes anything a
loop body with no label produces, so the measurement is the change and not output.

**+8.7 million instructions on 300k passes, 29 per pass, +0.55%.** Wall clock 0.14s on
both arms, indistinguishable.

**And no recompilation at all.** `chunk_for` is consulted once per activation entry, so a
`TRACE` inside a loop never reaches the cache; the whole cost is the per-clause staleness
comparison and the promoted clause running on the gated echo for the rest of the
activation. The plan's "a setting change invalidates the body's cached chunk" therefore
describes what happens at the *next entry* to that body, not at the change -- which is
why the figure is this small, and why the fallback had to exist at all.

## Tests, with mutation and adds-coverage evidence

Every mutation run was `cargo test --workspace --no-fail-fast`, restored from a `cp`
backup verified with `sha256sum -c`, and rebuilt. Counts are whole-workspace, so "1
failed" means nothing else in the tree catches it. The baseline is 1378 passed.

### `tests/ir_dual_cases/trace-settings`, seven stanzas, one test

Seven rows on the axis no other table here has: the setting in force when a clause runs
is not the setting its chunk was compiled under. Every expected byte measured against the
oracle before it was written down; every row passing with nothing promoted (commit
`56ab1511`, before the compiler emitted anything), which is what says a row can tell
whether it would ever have failed.

Boundaries rather than shapes: each setting change sits immediately before a **promoted
clause** -- an `IF` header, a `SELECT` header, a listed `WHEN` -- because an ordinary
clause is answered by the tree-walker's own gate whatever the compiler does.

| row | what it would catch |
| --- | --- |
| trace turned on inside a branch, before a later if | a compile-time decision with no way back to the gate |
| trace turned off inside a branch, before a later if | a fallback that can only *add* an echo |
| trace turned on inside a loop body, before a later if | a change that reaches the rest of its own pass and not the passes after it |
| one body entered untraced and then under trace r | the second entry running the first entry's stream |
| a body entered under trace r that then turns trace off | a compiled echo op that cannot be withdrawn (**found the gap**, P3) |
| trace turned on inside a branch, before a select | the fix being about `IF` rather than about clauses |
| trace i over a literal in an assignment | the first native expression op dropping `>L>` |

**Fix round 1 moved the rows into a `datadriven` file and withdrew the reason given for
not doing so.** That reason -- that a `*-*` echo of an `IF` header ends in a compared
space, which an external data file would not preserve -- is false, and checking it costs
one read of the parser: `datadriven` 0.9.0 pushes each expected line verbatim
(`expected.push_str(lines[i])`), so a trailing blank survives. It is now
`tests/ir_dual_cases/trace-settings`, read by
`both_engines_agree_on_every_case_file`, and the trailing blank is shown load-bearing by
mutation **P7** below.

**Two things the format did need, and neither was the stated one.** A `datadriven`
expected block ends at the first blank line, and a program's own output can contain one --
so every line of an expected block is tagged `rc>`, `out>` or `err>`, which also keeps a
compared trailing blank off the end of a line that would otherwise read as empty. And the
expected bytes here are the *oracle's*, where `datadriven`'s whole idiom is to regenerate
an expectation from the implementation: `REWRITE=1` would turn an oracle-measured
expectation into a self-consistent one, in a diff that looks like any other expectation
update. The test refuses to run under `REWRITE` for that reason, and the file's header
carries the oracle capture command instead.

### Golden tests (`src/ir/golden_tests.rs`), three added

* `a_traced_if_carries_its_clause_echo_as_an_op_of_the_region` -- the whole eleven-op
  stream and `op_of`, the neighbour of the untraced `IF` test, and the pair is D23's
  emission decision: one body, two settings, two streams.
* `a_traced_select_echoes_its_header_and_each_listed_when` -- one echo per promoted
  clause and none anywhere else; `SelectCaseText` stays outside the region; a `THEN`
  marker, a branch body and the `END` get none, since a second echo for a `Generic`
  clause would print it twice.
* `one_body_under_two_trace_settings_is_two_cached_chunks` -- the cache key, above.

### The witness that the mechanism is reached at all (fix round 1)

**Nothing pinned that the interpreter ever asks for a chunk under the setting in force**,
and two mutations proved it: making the entry lookup pass a constant `NORMAL`, and
answering `stale` yes for every clause. Each leaves the whole workspace green, and either
makes `Op::TraceClause` dead in production. No output test can close that, because the
run-time fallback is output-equivalent by design -- which is the same property that made
the brief's Step 2b test unable to witness the cache key.

`super::trace_op_echoes` is the counter, in the idiom `clause_op_entries` and the compile
counter already use here, and it counts where the echo is *emitted* rather than where the
op is fetched: the second mutation leaves the op in the stream and skips its work, so a
count of arrivals would stay green on it.

* `a_body_entered_under_trace_r_echoes_its_promoted_clause_from_the_chunk` -- a body
  entered through a `CALL` with `trace r` already in force, so its chunk is compiled to
  echo, asserting exactly one echo came from the op. Entered rather than set in the body,
  because a `TRACE` on a body's own first line runs after that body's chunk exists.
* `no_trace_op_echoes_without_the_engine_or_without_the_setting` -- two negative controls,
  answering two different degenerate counters: one that never moves (the tree-walker row)
  and one that moves for every promoted clause regardless of setting (the untraced row,
  which runs the identical construct with everything but `TRACE` unchanged).

### Mutation log

| # | Mutation | Result | Failing tests |
| --- | --- | --- | --- |
| P1 | `Op::TraceClause` never emitted | red | 1373/5 -- the five new tests and **nothing else**, so the emission path is entirely new coverage |
| P2 | the staleness answer forced to `false` | red | branch shapes + the corpus sweep + the case file |
| P3 | the compiled echo op ignores staleness | red | the case file only |
| P3b | P3 with the "entered under trace r, then turns trace off" stanza removed | **green** | nothing -- so that stanza is what catches it |
| P4 | the chunk cache keyed on `BodyKey` alone | red | `one_body_under_two_trace_settings_...` only; **output stays correct**, see the correction above |
| P5 | an `IF`'s echo emitted after its `EvalExpr` | red | the traced `IF` golden test + the case file |
| P6 | the compiled echo prints at indent 0 | red | the case file only |
| P6b | P6 with both entered-traced stanzas removed | **green** | so those two stanzas are what catch it |
| P7 | one expected `*-*` line's trailing blank deleted from the case file | red | the case file, naming the file and line -- so the format preserves the byte the withdrawn reason said it would lose |
| P8 | `ChunkTrace::of` drops `labels` | red | `trace_labels_covers_the_labels_only_mode` |
| R1 | the entry chunk lookup asks for a constant setting rather than the one in force | red | the new driver counter test **only** -- 1377/1 |
| R2 | every promoted clause answered `stale` | red | the new driver counter test **only** -- 1377/1 |

R1 and R2 are the pair fix round 1 exists to answer: both leave every output-comparing
test in the tree green, and each makes the compiled emission decision unreachable.

Earlier rounds of these mutations were run against intermediate shapes of the driver and
of the case table, and are not tabulated: the code they were applied to is not the code
that shipped. Every row above was run against the committed tree.

**P8 says less than it looks.** `labels` is load-bearing for the *run-time* gate, because
`tracing_clause` now goes through `ChunkTrace::echoes` -- that is what the existing
`TRACE L` test catches. It says nothing about `labels` belonging in the cache **key**;
see "What I could not verify".

### The `run.rs` unit tests under both arms

`Interp::new`'s `engine` was flipped to `Engine::Ir` in place, `cargo test -p rexx-exec
--lib` run, and the file restored from a `cp` backup verified with `sha256sum -c`.
**539 passed, 0 failed**, including every `run.rs` test that asserts exact
`interp.trace` bytes. This is a measurement, not a change: `Engine::TreeWalker` is still
the default and Task 11 owns flipping it.

### The trace oracle under both arms

All fifteen `tests/trace_oracle/` witnesses (thirteen in that directory plus
`corpus/lang/trace_output.rex` and `corpus/lang/pull_queue.rex`) run through
`rexx-run` under `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`: **stdout, stderr and
exit status byte-identical on all fifteen**. The tree-walker arm is what the committed
harness compares against the oracle, so byte identity carries the IR arm with it.

## An oracle divergence found and not fixed: `TRACE VALUE` emits no `>K>` line

**Pre-existing, both engines identical, and unrecorded anywhere until now** -- which is
the reason it is written out here in full rather than filed as a note.

The oracle traces a `TRACE VALUE expr` clause's own computed setting as a keyword value
line. This crate traces nothing there. Measured from a clean directory:

```rexx
trace r
zz = 'n'
trace value zz
say 'after'
```

```text
oracle       3 *-* trace value zz          here    3 *-* trace value zz
             >K>   "VALUE" => "n"                  (nothing)
```

Four facts a fix needs, each measured rather than inferred:

* **The line is emitted under the setting in force *before* the new one is installed.**
  `trace value zz` with `zz = 'n'` turns tracing off and still prints it; the same clause
  reached from an untraced program with `zz = 'r'` prints nothing at all.
* **It is gated on `results`, not on `intermediates`** -- present under `trace r` and
  under `trace i` alike. Under `trace i` it follows the `>V>` line the expression's own
  variable read produces, so it is emitted after the expression is evaluated.
* **The site is one arm and one missing call**: `run.rs`'s `Trace::Value`, which evaluates
  the expression, checks `is_whole_number`, and goes straight to `set_trace_mode`.
* **The machinery already exists.** `Interp::trace_keyword` is what emits it, and this
  crate already emits a `"VALUE"` keyword line for `SIGNAL VALUE` (`run.rs`) and for
  `PARSE VALUE` (`parse_template.rs`). So the fix is a call at a known site rather than
  new formatting.

**Confirmed pre-existing** by running the same program on `4f98b9dc`'s binary, on both
engines: neither prints it there either.

**It does not belong in `KNOWN_DIVERGENCES`.** That table's contract is a program the two
*engines* answer differently, with an assertion that is red if either side moves; here the
engines agree and it is the oracle they both differ from. It does not belong in
`phase-4-exclusions.txt` either, whose own header says adding a row is a plan amendment
rather than a file edit.

## Oracle divergences met

**None that are new, and none from the catalogued family.** Every one of the seven probe
programs measured for this task matches the oracle on both engines, before and after.
That is expected: the family Task 4b' catalogued needs a `CALL ON` handler that itself
raises a second trapped condition, and nothing here has one.

`KNOWN_DIVERGENCES` is untouched and
`the_known_engine_divergences_still_diverge_exactly_as_recorded` is green, so neither
recorded row moved.

## What I could not verify

**`labels` in the cache key is defensive and untested as a key component.** `compile`
asks `trace.echoes(is_label)` with `is_label` read off the instruction, which is right for
whatever gets promoted next -- but no promoted clause is a `LABEL` today, so `TRACE L` and
`TRACE N` compile identical streams and keying on `labels` costs one extra compile for a
program that moves between them and buys nothing measurable. I could not construct a test
that distinguishes the two chunks, and I did not remove the field: a rule that held only
because of what happens to be promoted is the kind that stops holding with nothing going
red.

**The `+0.25%` on `emptyloop` under the compiled stream is not attributed.** Four
instructions per pass on a body with no promoted clause, so it is not the staleness
comparison; the candidates are the extra `Op` variant in the driver's match and the extra
`Chunk` field. I measured it and stopped rather than chasing it.

**A future `TraceMode` field that affects the echo would be missed, and nothing
structural would catch it.** `ChunkTrace` is a hand-written projection of two of
`TraceMode`'s fields, so narrowing `compile`'s argument stops the compiler reading an
*unkeyed* field -- it does not stop someone widening `TraceMode` and teaching the echo to
read the new field through `tracing_clause` without adding it here. What would catch it is
making the projection non-optional: a `TraceMode` whose echo-relevant fields live in a
nested `ChunkTrace` the mode *contains* rather than one this function copies out, so
adding a field means choosing which of the two it goes in. I did not build that, because
`TraceMode` has four fields with one reader each and restructuring it is a change to
D17's own type rather than to this task's.

**The `Op::TraceClause` echo reads `clause_state.current_value_indent` rather than
recomputing `printed_indent`.** `in_stepped_clause_with` sets that field to the clause's
printed indent immediately before, and nothing between there and the op writes it, so the
two engines cannot print at different indents -- but that is an invariant of an ordering,
not of a type. Mutation P6 is what makes it observable; nothing makes it structural.

**Whether the fallback is the right long-term answer is not decided here.** It is what
keeps a mid-body `TRACE` correct today at about 17 instructions per promoted clause. The
alternatives -- recompiling and re-entering mid-body, or deopting the clause to the
tree-walker -- both need the driver to move a live program counter between two streams,
and neither is a change this task should make on its own.
