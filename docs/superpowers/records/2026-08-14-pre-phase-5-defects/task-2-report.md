# Task 2 report: a trap handler's clauses echo at the wrong indent

**Commits:** `691266bd9` (the fix, the tests, the one forced expectation),
`1638a0ea4` (a comment-only note recording the sibling boundary left alone).

**Status:** all three named instances now agree with the oracle byte for byte,
on both engines, under `trace r` and `trace i`, on stdout, stderr and exit
status. Two further divergences of the same shape were found: one is fixed as
part of the same call site (the opposite direction of instance two), one is a
different site and is left alone and recorded.

**It is two defects, not one.** Section 3 has the evidence and the refutation
of the stated hypothesis.

---

## Step 1: the baselines, captured before anything changed

Tree at `a48f00636`. Oracle wrapper as CLAUDE.md specifies, every probe from a
fresh `mktemp -d`, three descriptors read separately, both engines. **In every
instance below the two engines emitted identical bytes**, so no instance is
about the compiled form; only the tree-walker column is shown where they agree.

### Instance three -- a plain `SIGNAL` inside a `CALL`ed label, no trap anywhere

```rexx
trace r
call sub
exit 0
sub:
signal onward
onward:
zz = 1 / 0
```

stdout empty on all three, `rc 214` on all three. stderr:

```
oracle                                   this crate (both engines)
     2 *-* call sub                           2 *-* call sub
     4 *-*   sub:                             4 *-*   sub:
     5 *-*   signal onward                    5 *-*   signal onward
     6 *-* onward:                            6 *-*   onward:
     7 *-* zz = 1 / 0                         7 *-*   zz = 1 / 0
     7 *-* zz = 1 / 0                         7 *-*   zz = 1 / 0
     2 *-* call sub                           2 *-* call sub
Error 42 running <path> line 7:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
```

Two columns too deep from the label onward, the traceback's own repeat of line
7 included. The final `2 *-* call sub` agrees, and that is informative: it is
the *caller's* frame, whose indent was never wrong.

Under `trace i` the same program diverges in the same place and only there, the
two intermediate lines shifting with the clause they belong to:

```
oracle                    this crate
     7 *-* zz = 1 / 0          7 *-*   zz = 1 / 0
       >L>   "1"                 >L>     "1"
       >L>   "0"                 >L>     "0"
```

### Instance one -- `SIGNAL ON SYNTAX`, raised inside a callee

```rexx
trace r
signal on syntax
call sub
say 'not reached'
exit 0
sub:
return 1/0
syntax:
say 'in handler'
```

stdout `in handler` and `rc 0` on all three. stderr:

```
oracle                                   this crate (both engines)
     2 *-* signal on syntax                   2 *-* signal on syntax
     3 *-* call sub                           3 *-* call sub
     6 *-*   sub:                             6 *-*   sub:
     7 *-*   return 1/0                       7 *-*   return 1/0
     8 *-* syntax:                            8 *-*   syntax:
     9 *-* say 'in handler'                   9 *-*   say 'in handler'
       >>>   "in handler"                       >>>     "in handler"
```

Two columns, not the brief's four -- the brief's transcript was one call level
deeper. Under `trace i` the added `>L>   "in handler"` shifts with it.

**Which activation the handler runs in, measured rather than assumed.** With
`sub:` carrying a `PROCEDURE` and assigning `zv = 'SUB'`, the handler prints
`handler sees zv= SUB` on the oracle *and* here. So both interpreters run a
`SIGNAL ON` handler inside the callee's own activation (the callee inherits the
caller's trap table), and the divergence is not about which frame is live.

### Instance two -- `CALL ON` delivered at a loop header's boundary

The recorded instance names only its symptom, so the program is mine:

```rexx
trace r
call on user zx name h
do zi = 1 to raiser()
  say zi
end
say 'after'
exit 0
raiser:
raise user zx return 1
h:
say 'in h'
return
```

stdout `in h` / `1` / `after` and `rc 0` on all three. stderr diverges in four
lines, this time with the oracle **deeper**:

```
oracle                                   this crate (both engines)
     3 *-* do zi = 1 to raiser()             3 *-* do zi = 1 to raiser()
     8 *-*   raiser:                         8 *-*   raiser:
     9 *-*   raise user zx return 1          9 *-*   raise user zx return 1
       >K>     "RESULT" => "1"                 >K>     "RESULT" => "1"
       >K>   "TO" => "1"                       >K>   "TO" => "1"
    10 *-*     h:                            10 *-*   h:
    11 *-*     say 'in h'                    11 *-*   say 'in h'
       >>>       "in h"                        >>>     "in h"
    12 *-*     return                        12 *-*   return
     4 *-*   say zi                           4 *-*   say zi
```

Reproduces `13 *-*   h:` against `13 *-*     h:` as recorded in
`phase-4e-gate.md`, at this program's own line numbers. `trace i` diverges
identically, plus the handler's `>L>`.

---

## Step 2: where the indent comes from on each side

**The oracle keeps one counter**, `settings.traceIndent`, per activation
(`ActivationSettings.hpp:188`). It is incremented by `newBlockInstruction`
(`RexxActivation.hpp:308`) on entry to any block instruction, by `indent()` for
`THEN`/`ELSE`/`OTHERWISE`, and by one when an activation is created for an
internal call (`RexxActivation.cpp:229`, guarded so that an `INTERPRET`
activation does not step it). `signalTo` sets it to `0`
(`RexxActivation.cpp:2112`).

**This crate derives the number instead.** `Interp::printed_indent` is
`static_indent(instructions, target) + self.activation_indent +
self.indent_offset`: a lexical depth from the instruction list, plus a base set
once per activation (`invoke_call`: the calling clause's printed indent plus
two for a label, `0` for a `::ROUTINE`), plus an escape elevation. The value is
written into `clause_state.current_value_indent` at each clause boundary and
read from there by everything that traces.

The two agree wherever the counter's value is a function of lexical nesting.
They part company exactly where it is not, and there are two such places.

* `apply_flow`'s `Flow::Signal` arm moved `pc` and left both addends alone.
  Every `SIGNAL` reaches that one arm: a bare `SIGNAL label`, a `SIGNAL VALUE`,
  a `SIGNAL` that escaped an `INTERPRET` fragment, and the transfer a
  `SIGNAL ON` trap performs (`trap_or_propagate` ends `Ok(Flow::Signal(target))`).
* `run_repeating` runs a `DO`/`LOOP` header inside `in_clause`, and
  `in_clause`'s boundary is where a pending `CALL ON` handler is delivered
  (`clause.rs`, `leave_clause` -> `deliver_pending_trap` -> `invoke_call`, which
  bases the handler on `current_value_indent + 2`). At that boundary the field
  still held the value the `DO` clause *echoed* at.

---

## Step 3: one defect or two

**Two defects, one symptom.** The evidence is that each has its own site, its
own quantity, and its own mutation, and that fixing either leaves the other
exactly as wrong. **Measured per case, and read off the run rather than argued
from the code** (fix round 1; the first version of this paragraph asserted it
from runs whose output could not actually show it): mutation M1 reverts only
the `SIGNAL` half and turns red exactly the three `signal_*` cases, leaving all
**four** loop cases green; M2 and M3 revert only the loop half and between them
turn red exactly those four, two each, leaving all three `signal_*` cases
green. The two red sets are disjoint. Step 7's table has the per-case columns
and the two further mutations that split the loop half by call site.

**The stated hypothesis is refuted, and its prediction happens to hold.** The
hypothesis was *this crate keeps whatever indent is live at the transfer, where
the oracle sets it from the kind of transfer.*

* For the `SIGNAL` half it is right. The crate kept the live base; the oracle
  sets `0` because the transfer is a `SIGNAL`.
* For the `CALL ON` half it is wrong, and the second half of the sentence is
  the part that fails. This crate *does* set the handler's indent from the kind
  of transfer -- `invoke_call` adds two for a call, exactly as it does for an
  ordinary `CALL`, which is what the oracle's `internalCallTrap` does too. What
  is wrong is the **base** it adds to: the crate's notion of "the indent live at
  the transfer" is the last clause's *printed* indent, and the oracle's counter
  has moved on by then.

The prediction the hypothesis made -- "a fix which merely restores an indent
fixes the `SIGNAL` case and leaves the `CALL ON` case exactly as wrong" -- is
confirmed, by M1 and by the fact that the two halves needed separate code.

**Three directions, not two, and that is what pins the loop rule.** The brief
had two instances in opposite directions. There is a third:

| shape | oracle | this crate, before |
| --- | --- | --- |
| `do zi = 1 to raiser()`, `raiser` returns 1 (body runs) | `10 *-*     h:` | `10 *-*   h:` |
| `do zi = 1 to raiser()`, `raiser` returns 0 (no pass) | `10 *-*   h:` | `10 *-*   h:` |
| `do while raiser() < 1`, test false (no pass) | `10 *-*   h:` | `10 *-*     h:` |
| `do while raiser() < 1`, test true (body runs) | `12 *-*     h:` | `12 *-*     h:` |
| `do until raiser() > 0`, test true (loop ends) | `10 *-*   h:` | `10 *-*     h:` |
| `do until raiser() > 0`, test false (another pass) | `13 *-*     h:` | `13 *-*     h:` |
| `do raiser()` repeat count (body runs) | `10 *-*     h:` | `10 *-*   h:` |
| trap queued by a *body* clause | `10 *-*     h:` | `10 *-*     h:` |

One rule explains all eight rows: the handler is based on the counter **as it
stands when the header clause ends**, which is one level deeper than the
clause's own echo when the block is still open and unchanged when the loop is
over. The crate was two short on the controlled/repeat path and two long on the
`WHILE`/`UNTIL` path, because the `WHILE`/`UNTIL` code already wrote
`loop_indent` into the field for its own keyword line and the controlled path
never wrote anything. A fix that only moved the direction instance two names
would have broken two of these rows, which is why the table exists.

### Reading the transcripts against "the oracle is wrong"

**Decided against, on the transcripts.** The project's one confirmed upstream
trace-indent defect has an asymmetric shape -- two loop-exit paths in the C++,
one restoring a saved indent and one bare-decrementing it -- so it shows up as
an indent that *drifts* and never comes back. Nothing here has that shape:

* Every measurement above is reproduced by a single, self-consistent model (one
  counter, incremented on block entry and internal-call entry, zeroed by
  `signalTo`), and the model predicts the rows in **both** directions. A wrong
  side would have to be wrong in one direction only, or inconsistently.
* The two directions are produced by the *same* oracle code path with different
  data (`newBlockInstruction` versus `terminate`), not by two paths that
  disagree.
* Instance three needs no condition system at all, and the C++ that produces it
  is three lines with a comment saying what it is for (`doStack = OREF_NULL;
  blockNest = 0; settings.traceIndent = 0;`). It is a deliberate reset of block
  state, not a missed restore.

---

## What changed

`rust/crates/rexx-exec/src/run.rs`, two sites; `ir/drive.rs` needed nothing,
because both engines share `apply_flow` (`drive.rs:239`) and
`run_loop_with_header` (`drive.rs:1337`). Both engines were checked
independently on every probe rather than inferred from that.

**1. `apply_flow`'s `Flow::Signal` arm** now clears `activation_indent` and
`indent_offset` after moving `pc`. Both addends go, because the oracle's
counter is absolute rather than a base plus an elevation.

Setting the base to `0` rather than restoring a saved one is sufficient, and
the reason is measured: the oracle refuses a label inside any block instruction
-- `47.2` inside a `DO`/`LOOP`, `47.3` inside an `IF`, `47.4` inside a `SELECT`
-- so a `SIGNAL` target's `static_indent` is always `0`, and the sum is `0`
once both addends are cleared. Clearing it on *this* activation is right
because `invoke_call` already saves and restores both fields, which is what
makes the caller's indent come back after a `SIGNAL`ed callee falls off its end.

**2. A new `Interp::settle_block_indent`**, called at the end of the loop
header clause and at the end of the `UNTIL` clause, sets
`current_value_indent` to the body's indent when the block is still open and
the `DO` clause's when the loop is over. The header closure was reshaped into a
labelled block so that there is **one** call rather than one per exit -- three
`return`s each needing their own line is the hand-maintained-list shape this
tree has been bitten by before.

**3. `rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries`**, one
recorded row's expected bytes and the comment beside it. This is the forced
site under the plan's exception, and it is forced in the narrow sense: the row
records what this crate prints, this crate now prints something else, and the
comment asserted that *both* of the row's indents were not the oracle's, which
is no longer true. The row's other indent -- `2 *-* do i = 1 to raiser()`
against the oracle's `2 *-*   do i = 1 to raiser()` -- is untouched, is a
*traceback* quantity rather than a transfer one, and is on the not-fixed list
below. Attribution is preserved as the exception requires: the row's own
program is the mutation witness for M3 and M5, separately from the new test
file.

---

## Step 4: the test, and why it could not go through either harness

`rust/crates/rexx-exec/tests/trace_indent.rs` plus `tests/trace_indent/`.

DEVIATION 0 collapses the run of spaces between a trace marker and its content,
which is exactly this defect's signature, so `tests/corpus.rs` and
`tests/trace_oracle.rs` compare a wrong indent equal to a right one. The new
harness compares **raw** stderr, plus stdout and exit code, byte for byte, and
runs every case under `Engine::TreeWalker` and `Engine::Ir`. DEVIATION 0 itself
is untouched.

Seven cases, each a `.rex` program, a `.expected` captured from the oracle, and
a `.wrong` -- a transcript a real build produced that got that case's indent
wrong. `deviation_0_collapses_every_wrong_answer_this_file_holds` asserts, per
case, that `.wrong` and `.expected` differ **raw** and are **equal** under
`support::normalize_stderr`. That is the blindness measured rather than
described, and it is what says the raw comparison is not decoration.

Six of the seven `.wrong` files are the pre-fix build's own output. The seventh,
`call_on_at_an_until_test_that_runs_another_pass`, is the **adjacent passing
case**: the fix does not move it, and its `.wrong` comes from mutation M3. It is
in the set on purpose -- it is what pins that `settle_block_indent` answers
`loop_indent` for the right reason rather than by accident.

The case list is read from the directory rather than written in the file, and
`every_case_ships_all_three_files` turns a half-added case into a failure.

**Confirmed failing before the fix.** Against a build made by restoring
`run.rs` from `git show HEAD:` (a read-only recovery; the working file was
restored afterwards from a `cp` backup and `sha256sum -c`-verified), both
engine tests go red, and `deviation_0_...` passes -- which is the per-case
statement that all six moved cases really moved.

---

## Step 5: the blast radius

> **Superseded by fix round 1, and the numbers below are the corrected ones.**
> The first version of this section reported 92 programs and 10 changed
> streams. Both figures were real but were taken *before* the two `UNTIL`
> cases were added to `tests/trace_indent/`, and then written up as though
> taken at HEAD -- which made the count disagree with Step 4's own case list
> and made "every `.rex` under `rust/`" false, since the command behind it
> named two directories rather than the tree. Re-run at HEAD over a stated
> population; the round-1 note at the end of this report has the audit trail.

**Raw**, which is the number that means anything here. The population is
**every `.rex` file git tracks under `rust/` at HEAD -- 105 programs**, listed
by

```bash
git ls-files 'rust/*.rex' 'rust/**/*.rex'      # 105: 71 corpus, 23 crates,
                                               # 10 bench-programs, 1 bench-control
```

Each was run under both engines with the pre-fix binary (`691266bd9^`) and the
post-fix binary (HEAD), comparing stdout, stderr and exit status as raw bytes.

* **630 streams compared** (105 programs x 2 engines x 3 descriptors);
  **12 changed**.
* All 12 are stderr, and all 12 belong to the **six moved cases** under
  `tests/trace_indent/` -- six cases x two engines. That is now the same six
  Step 4 names.
* **No corpus program changed on any descriptor**, no `bench-programs` program
  changed, and no stdout or exit status changed anywhere in the population.

**The sweep is not vacuous, and here is its sensitivity.** Of the 105 programs,
54 write to stderr at all, 40 emit at least one `*-*` clause line, and 27 emit
at least one trace line at a non-zero indent. 19 corpus programs name `TRACE`,
13 name `SIGNAL`, 4 name `CALL ON`. So the population contains the constructs
and the trace modes; it simply contains no program that signals out of a called
label or delivers a `CALL ON` at a loop header.

**A second, larger population, and why its zero is worth almost nothing.**
`rust/corpus-l1/` holds 12,059 untracked `.rex` programs extracted from
`ootest`. Swept the same way: **72,354 streams compared, 0 changed.** That
number should not be read as strength. Measured on the same run: **none of the
12,059 emits a single `*-*` line**, only 4 write to stderr at all, and 12,055
exit 0 silently -- they are `::routine`/`::class` assertion shims that turn
tracing on nowhere. A trace-indent change could not have moved them, so the
zero is a property of the population rather than evidence about the fix. It is
recorded because the population exists on disk and someone would otherwise
sweep it and report the number as though it counted.

**Through the harness**, for completeness and not as evidence: the corpus
differential reports **52 of 52 matching** before the fix and 52 of 52 after,
on both profiles. That number cannot move for this defect -- DEVIATION 0 is
applied to its comparison -- and it is reported only so that nobody reads it
later as though it had been an instrument.

**What the sweep could not see.** The `ootest` populations `ir_dual.rs` draws
from are program *bodies* extracted from `.testGroup` files at run time, not
`.rex` files on disk, so they are outside this A/B. They are covered
engine-against-engine by `ir_dual.rs`, which is green, but that comparison is
this crate against itself and would not notice both engines being wrong
together.

**Recorded expectations that moved:** exactly one, the
`loop-header-boundaries` row described above. The full workspace suite is
**1508 passed, 0 failed**.

---

## Step 6: the three places it could hide

All four probes below were run on both engines, three descriptors, `trace r`,
**against the pre-fix build as well as the post-fix one** -- because a probe
that was already clean before the fix says something quite different from one
the fix repaired, and "came back clean" alone does not distinguish them.
**All four are clean after the fix.** Two of them were already clean before it.

**(a) A raise inside a callee inside a loop body.** `signal on syntax`, a
`do zi = 1 to 2` whose body does `call sub`, and `sub:` doing `return 1/0`.
**Wrong before the fix, clean after.** This is the one worth the most: it
composes both defects' constructs, and it is a shape neither instance covers.
The pre-fix bytes were four columns out, not two, because the callee's base was
computed from a `CALL` clause that was itself inside the loop:

```
oracle                                   this crate, before
    10 *-* syntax:                           10 *-*     syntax:
    11 *-* say 'in handler' sigl              11 *-*     say 'in handler' sigl
       >>>   "in handler 9"                      >>>       "in handler 9"
```

**(b) A `SIGNAL ON` and a `CALL ON` handler in the same program.**
`signal on syntax` plus `call on user zx name uh`, a loop body doing
`zq = raiser()` (queues the `CALL ON` condition) and then `say 1/0` (raises the
trapped `SYNTAX`), with both handlers reporting `SIGL`. **Clean before the fix
and clean after** -- both handlers run, in the oracle's order, at the oracle's
indents, with the oracle's `SIGL` values, and the fix moved nothing here. The
`CALL ON` handler in it is delivered at a *body* clause's boundary, which was
never the wrong quantity.

**(c) A handler that itself calls a routine.** Two shapes, because the two
handler kinds reach a callee differently.

* A `CALL ON` handler doing `call inner`: **clean before the fix and clean
  after**, for the same reason as (b) -- delivery at a body clause's boundary.
* A `SIGNAL ON` handler doing `call inner` and then running on to `exit 0`:
  **wrong before the fix, clean after**, and it is the widest pre-fix
  transcript of the four -- ten lines, because the handler's own callee
  inherited the handler's wrong base and so did everything the handler ran
  afterwards:

  ```
  oracle                       this crate, before
       7 *-* syntax:                7 *-*   syntax:
       8 *-* call inner             8 *-*   call inner
      11 *-*   inner:              11 *-*     inner:
      12 *-*   say 'INNER'         12 *-*     say 'INNER'
         >>>     "INNER"              >>>       "INNER"
      13 *-*   return              13 *-*     return
       9 *-* say 'done'             9 *-*   say 'done'
         >>>   "done"                 >>>     "done"
      10 *-* exit 0                10 *-*   exit 0
         >>>   "0"                    >>>     "0"
  ```

**Two further shapes checked while establishing the loop rule**, both clean
after the fix and both wrong before it in the *opposite* direction from
instance two: a `DO WHILE` whose first test is false, and a `DO UNTIL` whose
first test is true. Both are now cases in `tests/trace_indent/`.

---

## Step 7: the mutation witness

Backups by `cp`, restored by `cp`, `touch`ed, `sha256sum -c`-verified and
rebuilt between every row. Every run is
`memcap 8G cargo test --release --workspace --no-fail-fast`, so "nothing else
caught it" is measured across the whole workspace rather than truncated at the
first catcher.

| # | mutation | red? | what caught it | which cases went red |
| --- | --- | --- | --- | --- |
| M1 | the two clearing lines removed from `Flow::Signal` | red | `trace_indent` **only** | the three `signal_*`, and no loop case |
| M2 | `settle_block_indent` always answers `loop_indent` | red | `trace_indent` **only** | `..._does_not_enter`, `..._until_test_that_ends_the_loop` |
| M3 | `settle_block_indent` always answers `do_indent` | red | `trace_indent` and `ir_dual` | `..._that_enters`, `..._until_test_that_runs_another_pass` |
| M4 | the `settle_block_indent` call removed from the `UNTIL` clause | red | `trace_indent` **only** | `..._until_test_that_ends_the_loop` alone |
| M5 | the `settle_block_indent` call removed from the header clause | red | `trace_indent` and `ir_dual` | the two loop-header cases |

The last column is **read from the runs**, on both engines, and it is only
readable because fix round 1 made the harness report per case: before that the
engine tests asserted inside their loop and stopped at the first failure, which
under M2 is a loop case in sorted order, so the `signal_*` cases never ran. The
round-1 note at the end of this report has that audit trail. The two families'
red sets are disjoint, which is the measured form of the Step 3 conclusion, and
M2's and M3's red sets **partition** the four loop cases by direction, two
each.

**M4's and M5's do not partition them, and the case in neither is the
informative one** (corrected in fix round 2; the first version of this
paragraph claimed a partition here too). Their union is three of the four --
`..._until_test_that_ends_the_loop` from M4, the two loop-header cases from
M5 -- and `..._until_test_that_runs_another_pass` goes red under neither.
That is not a gap in the mutations: removing either call leaves
`current_value_indent` at `loop_indent`, which is exactly that case's right
answer, because `run_repeating` already writes `loop_indent` there for the
`UNTIL` keyword line before the clause opens. So the case is insensitive to
*removing* the call by construction, and what pins it instead is M3, which
makes the call answer the wrong value rather than not answering at all. A
mutation set that only ever deletes calls would not have covered it, which is
why M2 and M3 are in the table beside M4 and M5.

**"Can fail" versus "adds coverage."** M1, M2 and M4 are caught by nothing in
the workspace except the new file, so it adds coverage rather than merely being
able to go red. M3 and M5 are caught by the `loop-header-boundaries` row as
well, which is the honest reading of that row's re-recorded bytes: it now pins
the entered direction too. M4 and M5 exist specifically to show that both call
sites are load-bearing -- without M4 the `UNTIL` call would have been
untested, since the brief's instances do not reach it.

The green run has non-zero run counts: `trace_indent` is `11 passed; 0 failed`,
the workspace `1508 passed; 0 failed`.

---

## Gates

From `rust/`, statuses read unpiped:

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0, **run once
  from a `cargo clean` target directory** (20840 files, 5.1 GiB removed first),
  since a same-session green on a warm target is provisional.
* `memcap 8G cargo test --release --workspace --no-fail-fast` -- exit 0,
  **1508 passed, 0 failed**.
* The same under `REXX_CORPUS_GATE=1 REXX_ASSERTIONS_GATE=1 REXX_BIF_GATE=1
  REXX_KEYWORD_GATE=1` -- exit 0, all four STRICT banners printed, corpus
  **52 of 52 matching**.

One caution for whoever repeats this: `memcap 8G cargo test` immediately after
`cargo clean` was OOM-killed at the cap (`peak 8.0G`) while cargo was still
*compiling*. Build first, then run the test under the cap.

---

## Found and deliberately not fixed

**1. A `SELECT CASE`'s own clause boundary is two columns short, exactly as the
loop header's was.** Measured after the fix, both engines identical, oracle
different from both:

```
call on user zx name h
select case raiser()          oracle:      11 *-*     h:
  when 1 then say 'one'       this crate:  11 *-*   h:
```

Same mechanism: `SELECT` is a block instruction, so the oracle's counter is
already one deeper when that clause ends. A trap queued by a **`WHEN`
condition** instead agrees byte for byte, because a `WHEN` is not a block
instruction and its clause ends at the level it echoed -- measured, not
assumed. This is a different site (`Select`'s own arm in `run.rs`, not
`run_repeating`), the fix does not force it, and under the plan's constraint it
stays. It is recorded in `ir_dual_cases/loop-header-boundaries` as well as here,
because that file already carries the list of loop-header-adjacent divergences
and is what the next reader of this area opens. **It is not on any task's
list, and it should be.**

**2. The enclosing frame's traceback line, in the same `loop-header-boundaries`
row.** The oracle prints `     2 *-*   do i = 1 to raiser()` where this crate
prints `     2 *-* do i = 1 to raiser()`. The oracle's traceback reads each
activation's live counter (`RexxActivation.cpp:4050`), and the `DO` block is
still open in that frame; this crate reports the frame at the `DO` clause's own
recorded failure site, which carries no block level. A traceback quantity, not
a transfer one. Untouched.

**3. Two `CALL ON` *delivery-ordering* divergences already recorded in
`loop-header-boundaries`' header, re-checked and unchanged by this fix.** A
zero-pass loop whose handler requeues (oracle `after` then `G ran 6`, this crate
`G ran 3` then `after`) still behaves exactly as recorded. My own `UNTIL`
requeue probe also diverges in ordering, in the opposite direction from the one
that note describes, which means I did not reproduce that note's program and am
not claiming to have re-verified that specific row -- only that this fix writes
`current_value_indent` and nothing about delivery ordering, and that the
zero-pass row is byte-identical to its record.

**4. A label inside a block instruction reaches a loud path, not the oracle's
error.** Found while establishing that a `SIGNAL` target's lexical indent is
always zero: `signal inside` with `inside:` inside a `DO` gives the oracle
`4 *-* inside:` then `Error 47 ... 47.2` at `rc 209`, and gives this crate
`rexx-exec: 47.2: Unexpected label.` at `rc 120`. Same for `47.3` (`IF`) and
`47.4` (`SELECT`). That is the declared loud-path shape rather than a silent
wrong answer, and it is nothing to do with this task.

---

## A question for whoever reviews DEVIATION 0

Not an edit, per the brief. But this task is the first thing in the tree that
compares a trace indent to the oracle at all, and the number it produces is
that six transcripts were wrong in a way both differential harnesses were
structurally unable to report -- one of them recorded and unfixed since
`phase-4e-gate.md` on 2026-08-09, one found on 2026-08-13, and the sharpest
found only when Task 1 stumbled over it.

DEVIATION 0's own justification, in `phase-4-exclusions.txt`, is a specific
upstream defect: a repetitive `DO`/`LOOP` that completes a body pass and then
ends on a failing control test decrements the oracle's indent counter once too
often, so later clauses print two spaces low. That is a real divergence this
crate has decided not to reproduce, and the record is careful to note that the
*content* half of the same symptom was closed by fixing it rather than by
normalising it.

The observation is that the justification is about programs containing a
completed loop, and the normalisation applies to every trace line in every
program. A narrower rule -- normalise only the transcripts a recorded exclusion
names -- would have let `corpus.rs` see all six of these, at the cost of a
per-program exclusion list. Worth a decision either way; it is not one a task
reaching around the normalisation should take.

---

# Fix round 1

**Commit:** `3d4657377`. Harness only -- `run.rs` is untouched and byte-identical
to `691266bd9`, verified by `sha256sum` against `git show HEAD:` at the end of
every mutation round below. No behaviour changed, so the fix's own conclusions
stand as written; what changed is that two of them are now measured rather than
asserted, and one reported number was taken from the wrong tree state.

## I1 and I2 have one root cause, and it is worse than either finding says

Both findings are correct, and they are the same mistake seen twice: **the
sweep ran before the two `UNTIL` cases existed, and Step 5 wrote its numbers up
as though they had been taken at HEAD.**

The audit:

* At sweep time `tests/trace_indent/` held five cases, so
  `find corpus crates -name "*.rex"` from `rust/` returned **92** and the A/B
  found **10** changed streams -- five moved cases x two engines. Both figures
  were true of that tree.
* The two `UNTIL` cases were added afterwards, when the mutation matrix showed
  the `UNTIL` `settle_block_indent` call had no case covering it. That made six
  moved cases, and Step 4 was written against the new count while Step 5 kept
  the old one. Hence the reviewer's 12-versus-10.
* The same staleness produced I2's arithmetic. 105 tracked `.rex` files at HEAD
  minus my 92 is 13, which is what let two different 13-file exclusions be
  fitted to the number. The real exclusion was **11** files -- `bench-programs`
  (10) and `bench-control` (1) -- against a tree that then held 103 tracked
  `.rex` files. Neither of the reviewer's two candidate explanations is what
  happened, and that is exactly the reviewer's point: the report did not say,
  so the number could be reverse-engineered into more than one story.

**Not reconciled in prose -- re-measured.** Reconciling would have kept a
number nobody could reproduce. Step 5 above now names the population by the
command that produces it, and the whole A/B was re-run at HEAD:

```bash
git ls-files 'rust/*.rex' 'rust/**/*.rex'   # 105 programs, the population
# pre-fix arm:  git show 691266bd9^:rust/crates/rexx-exec/src/run.rs
# post-fix arm: git show HEAD:rust/crates/rexx-exec/src/run.rs
# each program, both engines, stdout + stderr + exit status kept separately
```

**630 streams compared, 12 changed**, all stderr, all six moved cases x two
engines. That agrees with Step 4's case list, and no corpus or bench program
moved on any descriptor.

I also swept `rust/corpus-l1/` (12,059 untracked programs): **72,354 streams,
0 changed** -- and Step 5 now says why that zero is nearly worthless, because
**none of those 12,059 programs emits a `*-*` line at all**. Reporting it
without that sentence would have been the same defect as I1 in a bigger font.

**What the numbers cost.** Nothing downstream: the corrected sweep reaches the
same conclusion the wrong one did, that no program outside this task's own
cases moved. The defect was in the claim, not the fix.

## I3: the harness could not report what Step 3 claimed

Confirmed exactly as described, and I reproduced the blind spot before changing
anything: the cases sort with `call_on_at_a_loop_header_that_does_not_enter`
first, which is the case M2 breaks, so the asserting loop ended there and the
three `signal_*` cases never executed. "Only the loop cases went red" was a
claim about a run that had not happened.

**Fixed at the harness rather than in the prose**, because the reviewer's
alternative -- restating Step 3 on disjoint-path grounds -- would leave the next
mutation round with the same unreadable output. `check_case` now answers a
`Vec<String>` of mismatches instead of asserting, and `check_every_case`
collects across every case and asserts once, naming each disagreement and its
engine. `deviation_0_collapses_every_wrong_answer_this_file_holds` and
`every_case_ships_all_three_files` were changed the same way, for the same
reason.

`datadriven` was considered and not used. This project prefers it for case
*tables*, and these cases are already files on disk with a per-case directory
convention shared with `tests/trace_oracle/`; moving them into a single
datadriven file would have replaced a readable convention with a second one and
would not have fixed the reporting, which was the actual defect.

**With that, the whole mutation matrix was re-run and Step 7's table now has a
per-case column.** The measured red sets are disjoint between the two families,
which is the Step 3 conclusion turned from an argument into a measurement:

* M1 -- the three `signal_*` cases, **and no loop case**;
* M2 -- `..._does_not_enter` and `..._until_test_that_ends_the_loop`;
* M3 -- `..._that_enters` and `..._until_test_that_runs_another_pass`;
* M4 -- `..._until_test_that_ends_the_loop` alone;
* M5 -- the two loop-header cases.

The reviewer's "both loop cases is four" is right and the table now says four.
M2 and M3 partition them by direction. **M4 and M5 do not partition them** --
that sentence was wrong when this round wrote it and is corrected in round 2
above, where the case their union misses is worked through.

M1, M2 and M4 were re-run across the **whole workspace** as well, to confirm
the only-catcher claim survives the harness change: each fails
`-p rexx-exec --test trace_indent` and nothing else.

## M1 (Minor): the premise is true as written now

Confirmed: `check_case` compared `String::from_utf8_lossy` on stdout and
stderr while the file's premise is byte for byte, and lossy conversion maps
every invalid sequence to one replacement character, so two different non-UTF-8
stderrs would have compared equal. Nothing was hidden -- every transcript here
is ASCII, and I checked rather than assumed -- but a premise that is only true
of today's data is the kind that gets relied on later.

Both comparisons are `&[u8]` now. The lossy rendering is kept, in a `describe`
helper used only to build the failure message, which is where it was doing
useful work in the first place.

## Covering tests

Harness change, so `trace_indent.rs` is the covering suite, plus the whole
workspace because the harness lives in it.

```
$ cargo fmt --all --check                                    -> 0
$ cargo clippy --workspace --all-targets -- -D warnings       -> 0
$ memcap 8G cargo test --release -p rexx-exec --test trace_indent
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
                                                             -> 0
$ memcap 8G cargo test --release --workspace --no-fail-fast
1508 passed, 0 failed                                        -> 0
```

Run counts are non-zero on both, and 11 is the same count the file ran before
this round -- the change was to what a failing run *says*, not to how many
assertions exist.

## Still open

Nothing from this round. The three findings are closed at the harness and in
Step 5 and Step 7 above, and the superseded numbers are struck through in place
rather than silently swapped, so the next reader can see which figure came from
which tree.

One thing worth flagging for whoever runs the next sweep on this branch:
**`rust/corpus-l1/` is 12,059 untracked programs that emit no trace output at
all**, so any sweep that includes it will report a large denominator and a zero
that means nothing. The tracked population is 105 files and is the one with the
constructs in it.

---

# Fix round 2

**Commit:** `a87682b4c`. Harness only again; `run.rs` untouched since
`691266bd9`. Both findings were defects **introduced by fix round 1**, and both
were false statements rather than wrong behaviour -- which is the ground they
were sent on and the right one.

## 1: the headline counted descriptors and called them cases

Confirmed by reading the code rather than the message: `check_case` pushes one
entry per failing *descriptor*, `flat_map` flattens them, and
`mismatches.len()` was printed as the numerator of "N of M cases" whose
denominator was `names.len()`. One case wrong on all three descriptors printed
`3 of 7 cases disagree`, and a wide enough regression would have printed a
numerator larger than its denominator.

**Two counts now, because there are two quantities**: distinct failing cases,
and the descriptors under them.

**A green run cannot witness a message, so I forced both shapes and read them.**
The pair matters more than either line, because the old wording made the two
scenarios print the *same sentence*:

```
one case wrong on all three descriptors
  before:  3 of 7 cases disagree with the oracle under TreeWalker, ...
  after:   1 of 7 cases disagree with the oracle under TreeWalker, in 3 descriptors, ...

three cases wrong on one descriptor each (mutation M1)
  before:  3 of 7 cases disagree with the oracle under TreeWalker, ...
  after:   3 of 7 cases disagree with the oracle under TreeWalker, in 3 descriptors, ...
```

The first shape was produced by editing one `.expected` file's `RC`, its stdout
and one stderr line together; the file was restored from a `cp` backup and
`sha256sum`-verified, and the second from mutation M1 with `run.rs` restored
the same way. The listed detail under the headline was already correct and is
unchanged.

## 2: "M4 and M5 partition the loop cases by call site" was false

Confirmed against the recorded runs, not re-derived from the code. M4's red set
is `..._until_test_that_ends_the_loop`; M5's is the two loop-header cases;
their union is **three** of the four loop cases, and
`..._until_test_that_runs_another_pass` is in neither.

**The reviewer's reason for the gap is right, and checking it is what makes the
correction worth more than a deletion.** `run_repeating` already writes
`loop_indent` into `current_value_indent` for the `UNTIL` keyword line, before
that clause opens. Removing either `settle_block_indent` call therefore leaves
the field holding `loop_indent`, which is exactly the right answer for the
continue direction -- so that case is insensitive to *deleting* a call by
construction, and it is M3, which makes the call answer the wrong value, that
pins it. A mutation set built only from deletions would have left it uncovered.

Corrected in both places it appeared -- Step 7 and the round-1 note -- rather
than in the one the finding cited. The same false sentence in two places is
still one false sentence, and fixing the cited copy alone is the hedge this
project's rule is aimed at. "M2 and M3 partition them by direction" is true and
stands; that pair's union is all four, disjoint.

## Covering test

```
$ cargo fmt --all --check                                    -> 0
$ cargo clippy --workspace --all-targets -- -D warnings      -> 0
$ cargo test --release -p rexx-exec --test trace_indent
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
                                                             -> 0
$ memcap 8G cargo test --release --workspace --no-fail-fast
1508 passed, 0 failed                                        -> 0
```

## Still open

Nothing from this round. Everything the re-review deferred to the final
whole-branch review is untouched, as instructed: the `.rex`-keyed case
enumeration, `run.rs:6490`'s comment, the unpinned repaired shapes, `trace i`
coverage, `.wrong` provenance, `settle_block_indent`'s two adjacent `usize`
parameters, and the two cosmetics.

**A note on this round's own shape, since it is the second in a row.** Round 1
fixed three findings and introduced two false statements; both were in prose I
wrote *about* a measurement rather than in the measurement. Neither survived
contact with the thing it described -- one is contradicted by reading the code
under it, the other by the run it summarises. That matches what this project
already records about correction rounds being where false statements enter, and
the only defence that worked here was the reviewer re-deriving both from the
artefacts instead of reading my sentences.
