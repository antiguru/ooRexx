# Task 4b' report: promote `Select`, and build the frame stack it needs

Branch `plan/rust-rewrite`, six commits on top of `1535b030`.

* `37dacefc` -- measure SELECT's boundaries before anything is promoted
* `d53e0a8f` -- let the clause unit own a listed WHEN, not the scan loop
* `67f37050` -- take SELECT's three decisions out of the arm that makes them
* `75b4e770` -- flatten SELECT onto a stack of open construct frames
* `b52baa2b` -- keep the driver's per-clause cost where the promotion found it
* `f86cb0d5` -- give a promoted branch back the boundary flattening took (fix round 1)

Suite: **1369 passed, 0 failed, 4 ignored**, green in dev, dev+STRICT, release and
release+STRICT.
Baseline was 1364/0/4; the five new tests are three golden, one driver count and one
known-divergence pin.

## The frame stack

### What it is

`SelectFrame` (`src/ir/drive.rs`) is one branch of one `SELECT` that is currently running.
It carries the `SELECT`'s instruction index, its `LABEL` name, the branch's own instruction range
`[start, end)`, where `leave_select` resumes it, one past the branch's last **op**, and which of the
two kinds of branch it is.
The driver keeps a `Vec` of them and an escaping `Flow` walks it.

### Why it has to exist

The tree-walker resolves a matched `WHEN` as `run_bounded(code, when + 1, body_end, ...)?` followed
by `leave_select`, and the *Rust call stack* is the mechanism: it is what makes a `Flow` escaping the
branch meet that `leave_select` before it meets the enclosing range.
A flat op stream has no call between the two.
Without a frame, a `LEAVE` naming the `SELECT` would be absorbed against the enclosing body's range
and the construct would never see it.

### The four design decisions inside it

**The frame carries the branch's *instruction* range, and the escape walk uses `absorb`.**
That is what makes this a flattening rather than a second implementation of absorption: `settle`
absorbs against `frame.start`/`frame.end` exactly as `run_bounded` absorbs against the range it was
given, then hands what escaped to `leave_select`, then repeats against the next frame out.
One round per frame is one round per Rust call frame, in the same order.

**The frame also carries an op position, `op_end`, and that is the only place the two spaces meet
inside a frame.**
A branch running off its own end is `run_bounded` answering `Flow::Next` in the tree-walker; in a
flat stream it is the counter reaching the op the branch's last op is followed by.
`op_end` is `chunk.op_at(frame.end)`, computed when the frame is pushed.
The check sits at the top of the driver's loop, *before* the op is fetched, because the op at
`op_end` belongs to whatever follows the branch and must not run until the branch has been left.

**The stack is a local of `run_ops`, not state on `Interp`.**
A `Failure` unwinds out of `run_ops` with frames still open and the `Vec` is dropped, exactly as the
Rust frames it replaces are dropped.
A stack on `Interp` would need every raise in the crate to remember to unwind it.
A `DO` body inside a branch re-enters `run_ops` through `run_bounded_from_chunk` and gets a stack of
its own, which is the correct nesting: a `LEAVE` there meets the loop first and the `SELECT` frame
afterwards.

**The `OTHERWISE` redirect is expressed as op layout rather than as a decision in the driver.**
`Op::EnterOtherwise` sits at the `OTHERWISE` marker's entry in `Chunk::op_of`, which is the resume
table, so **every** arrival there opens the branch's frame: the scan running out of `WHEN`s
(`PatchKind::Resume`) and an absorbed `WHEN CASE`'s `Flow::Goto` landing on the marker
(`Absorbed::Resume` -> `op_at`).
I first wrote the explicit `select_escape` arm in the driver as well; mutation **M2** deleted it and
the whole workspace, corpus included, stayed green, because the layout had already answered it.
It is gone, and `SelectEscape`'s own doc comment now says it is the tree-walker's decision and why
the stream needs no copy.
What *does* redden is moving the op off that entry (**M1c**, **M4b**).

## The op set

Four new variants, plus one instruction added to an existing one.

| Op | Where | What it does |
| --- | --- | --- |
| `EvalExpr { index, slot: 0, dst }` | inside the header's `Clause` region | now also evaluates a `SELECT CASE`'s `CASE` expression, through `Interp::select_case` |
| `SelectCaseText { index, case }` | after the header's region | sets `Interp::current_case_text` from the register, the one hand-off an **absorbed** `WHEN CASE` has |
| `WhenTest { index, case, dst }` | inside a listed `WHEN`'s `Clause` region | runs `Interp::scan_when` and stores the logical answer `JumpUnless` reads back |
| `EnterWhen { select, when }` | after a `WHEN`'s region | pushes the frame over that `WHEN`'s branch |
| `EnterOtherwise { select }` | at the `OTHERWISE`'s `op_of` entry | pushes the frame over the `OTHERWISE` branch |

A `SELECT` compiles to a header region, a scan chain of `WHEN` regions each ending in a `JumpUnless`
to the next scan entry, and `Generic` ops for everything else -- the `THEN` markers, the branch
bodies, the `OTHERWISE` marker and the `END`.
Nothing jumps *past* a branch: a branch is left by its frame, not by an op.

`SelectCaseText` is deliberately **outside** the header's clause region.
The tree-walker sets `current_case_text` after the header clause has ended, so a `CALL ON` handler
delivered at that boundary cannot be the last writer of it, and putting the op inside the region
would have moved that.

The `CASE` value lives in a register allocated in the **enclosing** scope, which is the first use of
that half of the plan's register discipline.
The last `WHEN` of a `SELECT` is tested after every earlier `WHEN`'s branch has already run, so a
register released at a member clause's boundary would be handed out again while the value is still
being compared against.
`golden_tests.rs` previously said the mark's *position* was not observable in an emitted stream and
that a loop's control value would be the first construct to make it so; a `SELECT CASE` is.

## Sharing: what both engines enter

Nothing about `SELECT`'s semantics is written twice.
Three functions were extracted in `67f37050` before any op was emitted:

* `Interp::select_case` -- the `CASE` expression, its `>K>` line and the value.
* `otherwise_range` / `select_exit` / `when_targets` -- every branch bound, read from the nodes.
* `select_escape` -- the `OTHERWISE` redirect, which the tree-walker needs and the stream expresses
  as layout (above).

and `Interp::scan_when`, `Interp::leave_select`, `Interp::pop_search_frame`, `Interp::run_bounded`'s
`absorb` and `Interp::in_stepped_clause` were already shared.
The driver contains no comparison of a `WHEN CASE` value, no label search, no 28.5, no indent rule.

## Two defects found and fixed, both oracle-confirmed

### A listed `WHEN` did not own its clause

The scan loop discharged a clause's obligations by hand: it echoed the clause, passed the indent to
`scan_when` as an argument, and recorded the failure site of the *condition*.
It recorded the failure site of the clause's **boundary** nowhere.

Measured against the oracle, `call on user zx name h` / `select` / `when raiser() = 'V' then ...`
with a handler that divides by zero:

```
oracle       11 *-*     say 1/0
              3 *-*   when raiser() = 'V'
before        11 *-*   say 1/0
              2 *-* select
```

Two wrong answers from one omission: the wrong clause blamed, and the handler's own clauses printed
two columns short, because passing the indent as an argument is not the same as setting the field
`resolve_and_run_call` computes a callee's base from.
This is the exact analogue of the Critical defect Task 4b shipped for `IF`, in the construct with
more clause boundaries than `IF` has.

Entering `in_stepped_clause` deletes all four hand-rolled discharges.
It also gives a `WHEN` the clock invalidation, the `>I>` trace-entry decay and the GC temps frame
every other clause gets.
The decay is not observable: only an activation's *first* stepped clause is `Allowed`, and a
`SELECT` is already that clause wherever a `WHEN` could be second, so adding stepped clauses in the
middle cannot change which one is first.

### A `LEAVE` naming a `SELECT` from inside its `OTHERWISE` resumed one instruction short

`leave_select` had one resume for two arrivals.
For a matched `WHEN` the two coincide; for `OTHERWISE` they do not.
Measured under `trace r`:

```
oracle    4 *-*   otherwise / 4 *-*     leave s / 6 *-* say 'after'
before    4 *-*   otherwise / 4 *-*     leave s / 5 *-* end / 6 *-* say 'after'
```

The same `OTHERWISE` finishing normally *does* run the `END` (measured, and it is the adjacent
success pinned beside the failure).
`SelectResume { done, left }` is the fix; `select_exit` is the shared computation of `left`.
Confirmed pre-existing by building `1535b030` and running the probe there.

## What `OTHERWISE` gained, and why

`run_otherwise` ran `[otherwise_index + 1, end)` with the marker's echo and value indent written out
in front of it.
That hand-rolled pair is exactly what a second engine would have had to reproduce rather than share,
so the marker moved *inside* the range and is now an ordinary stepped clause; `step`'s own
`Otherwise` arm is already the no-op its execution is.
Byte-identical: `trace_clause` is `trace_stepped_clause(false, ..)` and the guard `trace_mode().all`
is `tracing_clause(false)`, so the echo is the same call with the same line and the same indent.

## Tests added, with mutation evidence

Every mutation run was `cargo test --workspace --no-fail-fast`, restored from a `cp` backup verified
with `sha256sum -c`, and rebuilt.
The counts below are of the whole workspace, so "1 failed" means no other test in the tree catches
the mutation.

### Dual-engine cases (`tests/ir_dual.rs`, `BRANCH_CASES`)

Eighteen added, every expected byte measured against the oracle before it was written down.
Twelve landed in `37dacefc`, **before** anything was promoted and with both engines delegating, so
they are known to have passed on the pre-promotion tree.

Boundaries, which is the axis the brief asked for:

| Case | What it would catch |
| --- | --- |
| a call on handler queued by a select case expression | a header that never ends its own clause, delivering after the first `WHEN` instead of before |
| a call on handler failing at a listed when's own boundary | a `WHEN` clause that records its own failure site but not its boundary's (**found the live defect**) |
| a call on handler failing at a when body clause's boundary | the body clause's own boundary being skipped or misattributed |
| procedure reached through a matched when | a branch inheriting the first-instruction permission |
| the escape elevation is restored after an escaped otherwise | `indent_offset` left in force past the `OTHERWISE` dispatch |

Shapes and flows:

| Case | What it would catch |
| --- | --- |
| select case under trace r, second when matches | the `>K> "CASE"` line, both `>>>` per comparison, the `THEN` marker |
| select with otherwise under trace r | the `OTHERWISE` marker's echo and the `END` that closes it |
| select case whose absorbed when case falls through to 7.3 | the residual indent riding an escape onto the `END` |
| select label whose absorbed when case escapes into otherwise | the F-EX1 redirect losing the search frame (28.3 instead of rc 0) |
| iterate naming a select is 28.5 | a matching label consuming an `ITERATE` |
| leave naming a select from inside a do block in its branch | the search walking the block's frame before the `SELECT`'s |
| leave naming a select from inside its otherwise | the two resumes collapsed to one |
| an otherwise that finishes normally runs its own end | the fix for the above over-applied |
| select inside a loop body, leaving the loop from a when | a bare `LEAVE` wrongly consumed by the `SELECT` |
| leave naming nothing, forwarded out of a when / out of an otherwise | `pop_search_frame`'s residual indent, on both dispatches |
| select case nested in a matched when of another | the inner `CASE` value reclaiming the outer's register |
| select inside an interpret fragment | the tree-walker arm the promotion left in place |

### Golden tests (`src/ir/golden_tests.rs`), three added

* `a_select_with_an_otherwise_compiles_to_a_scan_chain_and_two_frames` -- the whole nineteen-op
  stream and the `op_of` table, including `op_of[7] = 14` (the `EnterOtherwise`, not the marker).
* `a_select_cases_own_value_outlives_the_registers_its_whens_take` -- the `CASE` value is register 0
  and both `WHEN`s share register 1; `chunk.registers` is 2.
* `a_select_with_no_otherwise_scans_out_onto_its_own_end` -- the last `JumpUnless` names the `END`'s
  own op and no frame is opened, the neighbouring case that says `EnterOtherwise` belongs to the
  `OTHERWISE`.

### Driver count test (`src/ir/drive/tests.rs`), one added

`the_ir_engine_steps_a_selects_chosen_branch_from_the_chunk`, both paths, 6 clauses each.
Output cannot answer whether the IR engine drove a `SELECT` -- both engines resolve it through the
same functions and print the same bytes -- so this is a count.
With the construct left on the tree-walker it is 2 on either path.

### Mutation log

| # | Mutation | Result | Failing tests |
| --- | --- | --- | --- |
| MUT-A | `run.rs` reverted to `37dacefc` (the hand-rolled `WHEN` clause) | red | `both_engines_agree_on_every_branch_shape` only -- 1363 passed, 1 failed |
| M1 | `EnterOtherwise` emitted after `first_op_of.push` instead of before | **green** | none: with nothing else in that slot both entries name the same op. Discarded as a no-op mutation |
| M1c | `EnterOtherwise` not emitted at all | red | branch shapes + `a_select_with_an_otherwise_...` |
| M2 | driver's `select_escape` redirect deleted | **green** | none -- see "the `OTHERWISE` redirect is layout"; the arm was removed |
| M3b | the `CASE` register released at the header instead of past the `END` | red | branch shapes + **the corpus sweep** + `a_select_cases_own_value_...` |
| M4b | the scan's patch kind `Resume` -> `Enter`, skipping the frame op | red | branch shapes + `a_select_with_an_otherwise_...` |
| M5 | driver never restores `indent_offset` leaving `OTHERWISE` | **green** at first | nothing caught it; a case was added for it, after which |
| M5b | the same mutation, with that case present | red | branch shapes |
| M6 | `OTHERWISE`'s `left` resume collapsed to `done`, both engines | red | branch shapes |
| M7 | `settle` absorbs against the enclosing range instead of the frame's | red | branch shapes |
| M8 | `op_end` computed from `resume` instead of `body_end` | red | branch shapes + **the corpus sweep** + the driver count test |
| M9 | a `SELECT` arm guarded `if false` | **green** | the guard did not remove the arm; discarded |
| M9b | the `Select` arm deleted, so a `SELECT` compiles to `Generic` | red | all four new tests, and **not** the dual sweep -- which is the point of the count test |
| M10 | `WhenTest` emitted with `case: None` | red | branch shapes + the corpus sweep + `a_select_cases_own_value_...` |
| M11 | `SelectCaseText` always clears `current_case_text` | red | branch shapes |

Two mutations came back green and both were acted on rather than recorded and left: M2's subject was
deleted, M5's gap was filled with a case that reddens it (M5b).

## Performance

Not touched as a project.
The default engine is still `Engine::TreeWalker`, so nothing a default run does changed shape.
Measured with `/usr/bin/time`, release, **interleaved between arms within one sitting**, three
sittings, `1535b030`'s binary against this branch's:

| program | engine | before | after |
| --- | --- | --- | --- |
| `emptyloop.rex` | tree-walker | 2.76-2.77 | 2.64-2.90 |
| `emptyloop.rex` | ir | 2.84-2.87 | 2.86-2.87 |
| a 400k-iteration `SELECT` with three `WHEN`s | tree-walker | 1.01-1.02 | 1.02-1.04 |
| the same | ir | 1.03-1.04 | 1.06-1.08 |

The one number that needed an answer rather than a note: without `#[inline(always)]` on `settle` the
compiled stream's `emptyloop` is **3.39s**, a 19% regression, because the frame walk replaced an
`absorb` match written out in the driver's loop with a call made once per clause of every promoted
body.
The annotation puts it back at 2.86s.
That is the same discipline `in_clause` and `in_stepped_clause` already carry, with the measurement
in the doc comment (`b52baa2b`).

The residual `+3-4%` on the `SELECT` benchmark under the compiled stream is the frame stack itself:
one `pop_if` per op plus a push and a pop per branch.
The `+1-2%` on the same program under the *tree-walker* is the `WHEN` clause unit -- each listed
`WHEN` now pushes a temps frame and invalidates the clock, as every other clause does.
Neither was optimised.

## What I could not verify

**A `SELECT` header's failure-site indent still diverges from the oracle, and it is pre-existing.**
`call on user zx name h` / `select case raiser()` with a failing handler:

```
oracle       11 *-*     say 1/0        2 *-*   select case raiser()
here         11 *-*   say 1/0          2 *-* select case raiser()
```

The oracle raises the trace indent *before* evaluating the `CASE` expression, so the `SELECT`'s own
failure clause echoes at 2 where this crate reports 0, and the handler's activation is based two
columns short.
The *normal* trace of the same clause is at 0 on both (measured).
Confirmed present at `1535b030` and unchanged by anything here, because both engines record that site
through the same `in_stepped_clause`.
Not fixed: it is a rule about when the oracle's `traceIndent` moves, not about `SELECT`'s promotion,
and I did not want to change an indent rule on one probe.

**`select` / `otherwise` / `end` with no `WHEN` is error 7.1 at rc 249 on the oracle and rc 120 here**
(a parse-time refusal reported through the loud path rather than as a Rexx condition).
Pre-existing and unrelated.
It matters to this task only in that the body never parses, so `compile` never sees a `SELECT` with
an empty `whens` list, and the branch that emits a jump to the scan's first entry is therefore not
exercised by any program.
It is kept because the emission would otherwise depend on an adjacency nothing here asserts, and it
is written to be correct either way -- but it is not covered.

**The single-boundary claim above was wrong, and fix round 1 is the correction.**
It is kept here in full rather than deleted, because the shape of the error is the point: every
premise was true and the conclusion did not follow.
See "Fix round 1" below.

**`Interp::current_case_text` under a `CALL ON` handler that itself runs a `SELECT CASE`.**
Both engines set the field at the same point relative to the header's boundary, so they agree; what
neither does is save and restore it across a nested activation, which is the disclosed
nested-`SELECT CASE` limitation the field's own doc already carries.
I did not probe it.

## Fix round 1

### C1: the boundary a flattened construct stopped running

**What I got wrong.** The report above closed "the promoted `SELECT` has one clause boundary where
the tree-walker has two" with "I could not construct a program where the outer one delivers
anything", and listed the reason: every body clause, every `WHEN` and the `OTHERWISE` marker have
boundaries of their own, and nothing runs between the last of them and where the outer boundary
would sit.
Each of those is true.
The missing premise is that **a delivered handler can leave a new trap queued behind it** --
`in_clause` delivers at most one and does not re-check -- so the last member clause's boundary is
not the last boundary with *work*, which is the thing the argument actually needed.
The review constructed the program; I had reasoned instead of running, which is the one habit this
crate's own method section names.

```rexx
call on user zx name h        /* h raises zy */
call on user zy name g        /* g says SIGL */
select
when 1 = 1 then zq = raiser()
end
say 'after'
```

Oracle and tree-walker print `G ran 4` then `after`; the compiled stream printed `after` then
`G ran 6`, and in debug tripped `clause.rs`'s own assertion.

**The mechanism, measured across ten spellings.** The oracle closes a *taken* branch with a
synthetic instruction (`ast.rs`'s "Why there is no node for the synthetic end of a branch"), whose
boundary is where a re-queued trap is delivered, and which carries no source position of its own, so
`SIGL` stays on the branch's last clause.
The tree-walker gets that boundary from `step_in_temps_frame`, which wraps the whole `IF`/`SELECT`
arm.
A flattened construct has no wrapper, so it has to be an op.

**The fix.** `Interp::end_promoted_branch` is that boundary, entered by the driver at the two points
the wrapper would have run one: `Op::EndBranch` at the end of a promoted `IF`'s true branch, and the
close of a matched `WHEN`'s frame.

**And it is owed by a branch that was taken and by nothing else**, which is three separate
measurements and three separate "do not do it here" arms:

* an `IF` whose condition was false ran no branch, and the oracle jumps over the synthetic
  instruction -- `if raiser() = 'X' then nop` delivers at the *next* real clause. So the
  `JumpUnless` target is past the `EndBranch`, which the two no-`ELSE` golden tests pin.
* a redirect into `OTHERWISE` (F-EX1) is one `SELECT` still running, so the branch it left owes
  nothing yet -- `SIGL` 6, the `OTHERWISE` marker's own clause, not 5.
* `OTHERWISE`'s branch ends at the `END`, a real instruction with a boundary of its own -- `SIGL` 7,
  the `END`'s line.

**Nested branches each own one.** The oracle runs one synthetic instruction per branch, not one per
position, so `before` became a list per instruction, emitted innermost first (registration order is
outermost first, because an enclosing construct compiles first).
`nested_ifs_reuse_one_register` pins the two `EndBranch` ops and their order.

### C1's second half: `clause.rs`'s tripwire was a false positive

The `IF`-false-path case above **panicked** on `clause.rs:414` before it could be compared.
That assertion reads any condition still waiting when a later clause begins as a construct having
run an instruction without ending its header clause first.
A trap queued *during a delivery* is not that: the boundary that would have taken it had already
taken one, and the oracle defers it too.

Measured **with no construct anywhere in the program** -- `zq = raiser()` on line 3 whose handler
raises a second trapped condition -- the oracle prints `after` then `G ran 4`, this crate agrees, and
the assertion fired anyway.
Confirmed at `1535b030`, so it is older than this task and independent of any promotion.
`PendingTrap::queued_during_delivery` exempts exactly that and nothing else.

### C1's third part: the `OTHERWISE`-path divergence

Recorded rather than fixed, with the reason, and **pinned so it cannot move quietly**.

`KNOWN_DIVERGENCES` in `tests/ir_dual.rs` holds the two programs where the engines disagree, with
each engine's bytes and the oracle's, and
`the_known_engine_divergences_still_diverge_exactly_as_recorded` is red if either side moves --
including a fix, which should delete the row rather than update it.

Both rows are one mechanism: where the oracle has **no** end-of-branch instruction, the tree-walker's
wrapper runs a boundary anyway.
That is the `IF` false path and the `OTHERWISE` branch, and **the compiled stream is the one that
matches the oracle in both**.
Not fixed here because suppressing it means letting an instruction opt out of its own clause
boundary, which is precisely what `clause.rs` is built to make impossible; the honest fix is for the
tree-walker to stop resolving branches inside its own `step`, which is a change to `IF`/`SELECT`'s
design rather than to this phase's.

Neither row is reachable from any population -- a divergence needs a `CALL ON` handler that itself
raises a second trapped condition, and no corpus program or `ootest` row does that -- so the sweep's
"no divergence, unconditionally" contract is untouched by them.

### I1: three things were written in both engines

The report's "nothing about `SELECT`'s semantics is written twice" was false, and the evidence the
review gave is the right one: reproducing the `OTHERWISE` resume defect took a mutation of *both*
sites.

* the `SelectResume` pairing -> `when_resume` and `otherwise_resume`
* the `indent_offset` restore -> `leave_otherwise`, which is now the whole of leaving that branch
* the case-text hand-off -> `open_select_case`, which also carries *where* it happens (after the
  header clause, so a handler delivered at that boundary cannot be the last writer)

All four are `pub(crate)` in `run.rs` and called from both engines.
`N5` and `N6` below are the mutations that say each is load-bearing at its single site.

### M1, M2

Four doc comments carrying history rather than contract were rewritten (`Before::ThenEnd`'s "used
to", `SelectResume`'s "before this type existed" and "reproducing that defect took", `scan_when`'s
"an earlier version").
`settle`'s `inline(always)` doc now quotes the ranges that were measured -- 3.38-3.40s against
2.84-2.87s, 2.85-2.87s with the annotation -- and the report's table already did.

### Fix-round mutation log

Same method: `cargo test --workspace --no-fail-fast`, restored from a `cp` backup verified with
`sha256sum -c`, rebuilt.
Counts are whole-workspace, so "1 failed" means nothing else in the tree catches it.

| # | Mutation | Result | Failing tests |
| --- | --- | --- | --- |
| N1 | `Op::EndBranch` never emitted | red | branch shapes + all three `IF` golden tests |
| N2 | no boundary at a matched `WHEN`'s frame close | red | branch shapes only |
| N2b | N2 with the two new C1 cases removed | **green** | nothing -- so the cases add the coverage |
| N3 | `Op::EndBranch` emitted but a no-op in the driver | red | branch shapes only |
| N3b | N3 with the two new C1 cases removed | **green** | same |
| N4 | the tripwire exemption removed | red | the known-divergence pin |
| N5 | `otherwise_resume`'s `left` collapsed to `done` | red | branch shapes |
| N6 | `open_select_case` does not set `current_case_text` | red | branch shapes + five `run::tests` + one `lib` test |
| N7 | the F-EX1 redirect also closes the branch | **green** at first | nothing caught it |
| N7b | the same, with a case added for it | red | branch shapes |
| N7c | N7b with that case removed | **green** | so that case is what catches it |

`N7` repeats the pattern the first round hit twice: a green mutation is a finding, not a filing.
Here the answer was a case rather than a deletion, because the distinction is right on the merits
(one boundary per construct, not one per branch entered) and the oracle agrees with it -- `SIGL` 6
against the mutation's 5.

### What fix round 1 did not change

The performance work stands and the benchmark suite was not re-run, as instructed.
`Op::EndBranch` adds one op per taken `IF` branch and one `in_clause` per closed branch, whose fast
path is a `pending_trap.is_none()` test; nothing here was measured and nothing here claims a number.

### Oracle divergences still open, all pre-existing and all one family

Measured this round, both engines agreeing on each unless noted:

| program shape | oracle | here |
| --- | --- | --- |
| `IF` with an `ELSE`, handler re-queues in the `ELSE` branch | delivers at the branch's end | waits for the next clause |
| `DO` block, handler re-queues in the body | delivers at the `END`'s line | delivers at the body's |
| nested `IF`s, two re-queues | second delivery at `SIGL` 7 | at `SIGL` 6 |
| `IF` false path, handler re-queues | waits for the next clause | tree-walker delivers early (**engines differ**, IR correct) |
| `OTHERWISE`, handler re-queues | delivers at the `END`'s line | tree-walker at the branch's (**engines differ**, IR correct) |
| a `SELECT` header's failure-site indent | 2 | 0 |
| `select` / `otherwise` / `end` with no `WHEN` | 7.1 at rc 249 | loud refusal at rc 120 |

The first three are the elided end-of-branch instruction in constructs this task does not promote
(`ELSE`, `DO`) or at a depth whose `SIGL` numbering I did not decode.
The middle two are the pinned rows above.
The last two were already recorded before this round and are unchanged.
