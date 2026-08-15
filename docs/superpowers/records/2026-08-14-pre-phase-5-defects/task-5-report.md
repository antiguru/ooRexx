# Task 5 report: a `SELECT CASE` expression's clause boundary delivers a handler two columns short

**Commit:** `932e6b710` (base `6229d3776`).

**Status:** the defect is closed. `SELECT CASE`'s clause boundary is
byte-identical to the oracle on both engines, on stdout, stderr and exit
status, with stderr compared raw. **The `WHEN` case beside it did not move**,
and it now ships as a case of its own with a witness that it can fail.

> **The headline below was `SELECT CASE` is the last of them. It is false and
> fix round 2 has the refutation.** A plain `SELECT`'s boundary has the same
> defect and is still unfixed; the probe that reported it clean used the wrong
> shape. Step 2's survey table and its "two probes" section are corrected in
> place, and the reason the miss happened is fix round 2's section 2.

Every block instruction the oracle counts a level for was asked the same
question, each against a build without Task 2's fix as well. `SELECT CASE` is
the one this task fixed; **a plain `SELECT` turns out to have the same defect
by a route this survey's probes could not reach**, and is recorded rather than
fixed.

What was found and deliberately not fixed is in the last section below, and
each fix round adds to it -- read them together rather than that section alone.

---

## Step 1: the baselines, captured before anything changed

Tree at `6229d3776`. Oracle wrapper as CLAUDE.md specifies, every probe in a
fresh directory of its own holding nothing but that probe, three descriptors
read separately, both engines. **The two engines emitted identical bytes in
every case in this report**, so only one crate column is shown.

### The defect: a `CALL ON` handler queued by a `SELECT CASE` scrutinee

```rexx
trace r
call on user zx name h
zv = 'unset'
select case raiser()
  when 1 then say 'one'
  otherwise say 'other'
end
say 'after' zv
exit
raiser:
raise user zx return 2
h:
zv = 'set'
return
```

stdout `other` / `after set` and `rc 0` on all three, agreeing. stderr:

```
oracle                                    this crate (both engines)
     2 *-* call on user zx name h              2 *-* call on user zx name h
     3 *-* zv = 'unset'                        3 *-* zv = 'unset'
       >>>   "unset"                             >>>   "unset"
     4 *-* select case raiser()                4 *-* select case raiser()
    10 *-*   raiser:                          10 *-*   raiser:
    11 *-*   raise user zx return 2           11 *-*   raise user zx return 2
       >K>     "RESULT" => "2"                   >K>     "RESULT" => "2"
       >K>   "CASE" => "2"                       >K>   "CASE" => "2"
    12 *-*     h:                              12 *-*   h:
    13 *-*     zv = 'set'                      13 *-*   zv = 'set'
       >>>       "set"                            >>>     "set"
    14 *-*     return                          14 *-*   return
     5 *-*   when 1                             5 *-*   when 1
       >>>     "1"                                >>>     "1"
       >>>     "0"                                >>>     "0"
     6 *-*   otherwise                          6 *-*   otherwise
     6 *-*     say 'other'                      6 *-*     say 'other'
       >>>       "other"                           >>>       "other"
     7 *-* end                                  7 *-* end
     8 *-* say 'after' zv                       8 *-* say 'after' zv
       >>>   "after set"                          >>>   "after set"
     9 *-* exit                                  9 *-* exit
```

Four lines differ, all of them the handler's own activation, all two columns
short. Everything before and after the handler agrees, `>K>   "CASE"` included
-- which is what says the `SELECT` clause's *echo* was already right and only
its *boundary* was not.

### The adjacent success: the same trap queued by a `WHEN` condition

The same program with the `CASE` scrutinee moved into the `WHEN`:

```rexx
trace r
call on user zx name h
zv = 'unset'
select
  when raiser() = 1 then say 'one'
  otherwise say 'other'
end
say 'after' zv
exit
raiser:
raise user zx return 2
h:
zv = 'set'
return
```

**Byte-identical to the oracle on all three descriptors, on both engines,
before the fix.** The oracle's stderr, which is also this crate's:

```
     2 *-* call on user zx name h
     3 *-* zv = 'unset'
       >>>   "unset"
     4 *-* select
     5 *-*   when raiser() = 1
    10 *-*     raiser:
    11 *-*     raise user zx return 2
       >K>       "RESULT" => "2"
       >>>     "0"
    12 *-*     h:
    13 *-*     zv = 'set'
       >>>       "set"
    14 *-*     return
     6 *-*   otherwise
     6 *-*     say 'other'
       >>>       "other"
     7 *-* end
     8 *-* say 'after' zv
       >>>   "after set"
     9 *-* exit
```

Note what makes it the *adjacent* success rather than merely another passing
program. In both programs the handler runs at four columns. In the `SELECT
CASE` program the `SELECT` clause echoed at zero, so the handler is two levels
below the clause that queued the trap; in the `WHEN` program the `WHEN` clause
echoed at two, so the handler is one level below it. **One level of that
difference is the `SELECT`'s block, and the other is the handler's own
activation.** A rule stated as "a handler is one level below the clause that
queued it" fits the `WHEN` case and fails the `SELECT CASE` one; a rule stated
as "two levels" fits neither generally. The rule that fits both is the
oracle's own: the handler is one level below **whatever the counter reads when
the clause ends**, and a block instruction has already raised it by then.

---

## Step 2: is `SELECT CASE` the last of them?

### The enumeration, and where it comes from

The set is not guessed and not counted. Everything that raises the oracle's
`settings.traceIndent` inside an instruction's own `execute` is enumerated by
the C++ tree, and that enumeration is citable rather than assertable -- **not
because it is outside this repository, which it is not.** `interpreter/` is
git-tracked here, and **the files this enumeration cites** are byte-identical
to the oracle tree's copies, checked with `cmp` file by file. That is the whole
of what was checked: `diff -rq` over the two trees reports
`classes/NumberStringClass.cpp` differing, because the oracle repo carries an
uncommitted local edit to it, so no claim is made about the tree as a whole.
The axis that makes a citation safe is whether the
enumeration can change without the sentence being reread, and upstream C++
cannot: nothing on this branch edits it, and CLAUDE.md's first rule is that it
is read-only. Corrected in fix round 1; the citation stood, the reason given
for it did not.

```
$ /bin/grep -rn "newBlockInstruction" --include=*.cpp --include=*.hpp interpreter/
  execution/RexxActivation.hpp:308:  inline void newBlockInstruction(DoBlock *block)
                                       { pushBlockInstruction(block); blockNest++;
                                         settings.traceIndent++; }
  instructions/BaseDoInstruction.cpp:282
  instructions/SimpleDoInstruction.cpp:85
  instructions/SelectInstruction.cpp:139
  instructions/SelectInstruction.cpp:375

$ /bin/grep -rn "addBlockInstruction\|->indent()" --include=*.cpp --include=*.hpp interpreter/
  execution/RexxActivation.hpp:314:  inline void addBlockInstruction() { blockNest++; indent(); }
  instructions/SimpleDoInstruction.cpp:90
  instructions/BaseDoInstruction.cpp:316
  instructions/ThenInstruction.cpp:134, :136
  instructions/ElseInstruction.cpp:117, :119
  instructions/OtherwiseInstruction.cpp:65
```

Reading each `execute` gives the axis that decides whether the boundary is
observable at all -- **does that instruction's own clause evaluate anything
that could queue a condition, and does the level move before the clause
ends?**

| C++ site | instruction | clause evaluates | level moves before the boundary |
| --- | --- | --- | --- |
| `BaseDoInstruction.cpp:282` | `RexxInstructionBaseLoop::execute` | control expressions, in `setup` | yes, after them -- **Task 2** |
| `BaseDoInstruction.cpp:316` | `RexxInstructionBaseLoop::reExecute` | `iterate`'s tests | yes, before them -- **Task 2** |
| `SelectInstruction.cpp:375` | `RexxInstructionSelectCase::execute` | the `CASE` scrutinee | yes, after it -- **this task** |
| `SimpleDoInstruction.cpp:85` | `RexxInstructionSimpleDo::execute`, labelled | nothing | yes |
| `SimpleDoInstruction.cpp:90` | `RexxInstructionSimpleDo::execute`, unlabelled | nothing | yes |
| `SelectInstruction.cpp:139` | `RexxInstructionSelect::execute`, no `CASE` | nothing | yes |
| `ThenInstruction.cpp:134,136` | `RexxInstructionThen::execute` | nothing | yes, twice |
| `ElseInstruction.cpp:117,119` | `RexxInstructionElse::execute` | nothing | yes, twice |
| `OtherwiseInstruction.cpp:65` | `RexxInstructionOtherwise::execute` | nothing | yes |

So exactly three sites both move the level and evaluate something in the same
clause. Two are Task 2's and one is this task's.

### The probes, and the control

Twenty probes, each in a fresh directory of its own, each run under the oracle
and under **two** crate binaries on **both** engines with all three descriptors
compared raw:

* **`cur`** -- the tree under test.
* **`ctl`** -- the identical tree with Task 2's behavioural change removed:
  `settle_block_indent` made a no-op and the `activation_indent`/
  `indent_offset` clearing dropped from `Flow::Signal`. Built from a `cp`
  backup, the source restored from that copy and `sha256sum -c`'d afterwards.
  This is the control Task 2 used, and it is what makes a clean result mean
  something: **a probe clean on `cur` and red on `ctl` was repaired; one clean
  on both never had the defect.**

`ctl` also lacks this task's fix, so for the `select_case*` rows its red says
nothing beyond what `cur` already said before the fix. Its discriminating
value is entirely in the rows that came back clean.

| probe | construct exercised | `cur` before fix | `cur` after fix | `ctl` | reading |
| --- | --- | --- | --- | --- | --- |
| `select_case` | `select case raiser()` | **ERR** | same | ERR | this task |
| `select_case_nested` | the same inside a `do zi = 1 to 1` body | **ERR** | same | ERR | this task |
| `select_case_in_select` | the same inside another `SELECT`'s `WHEN` body | **ERR** | same | ERR | this task |
| `select_case_when_matches` | a listed `WHEN` matches rather than `OTHERWISE` | **ERR** | same | ERR | this task |
| `select_case_no_when` | no `WHEN` matches and there is no `OTHERWISE` | **ERR** | same | ERR | this task |
| `select_case_in_routine` | the `SELECT CASE` inside a `CALL`ed label | **ERR** | same | ERR | this task |
| `select_case_trace_i` | the same under `trace i` (`>L>`, `>=>` lines too) | **ERR** | same | ERR | this task |
| `do_header` | `do zi = 1 to raiser()` | same | same | **ERR** | repaired by Task 2 |
| `loop_until` | `do until raiser() > 0` | same | same | **ERR** | repaired by Task 2 |
| `when_condition` | `when raiser() = 1` | same | same | same | **never had it** |
| `when_case_compare` | `select case 2` / `when raiser()` | same | same | same | **never had it** |
| `simple_do` | plain `do` / `end`, raise before it and inside it | same | same | same | **never had it** |
| `simple_do_labelled` | `do label zl` / `end zl`, same | same | same | same | **never had it** |
| `plain_select_body` | plain `select`, raise in a `WHEN`'s body | same | same | same | **never had it** |
| `otherwise_body` | raise in an `OTHERWISE` body clause | same | same | same | **never had it** |
| `if_condition` | `if raiser() = 1` -- not a block instruction | same | same | same | **never had it** |
| `then_body` | raise in the instruction after a `THEN` | same | same | same | **never had it** |
| `else_body` | raise in the instruction after an `ELSE` | same | same | same | **never had it** |
| `plain_select_delayed` | a handler raising a *second* trapped condition in front of a plain `select` | same | same | same | ~~never had it~~ **probe artifact -- see below** |
| `simple_do_delayed` | the same in front of a plain `do` | same | same | same | ~~never had it~~ **probe artifact -- see below** |

`same` means byte-identical to the oracle on stdout, on raw stderr and on exit
status, under both `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`.

### The two probes that exist because an argument is not a measurement

> **These two probes are wrong, and the paragraph below records how.** They
> were built to test the argument and they tested a shape that cannot reach
> it. Both constructs do have the defect; fix round 2 measures it. The text is
> left standing because the *reason* it failed is the transferable part.

`SIMPLE DO`, plain `SELECT`, `THEN`, `ELSE` and `OTHERWISE` all move the level
and evaluate nothing, so the argument is that nothing can queue a condition
their boundary would deliver. That argument is exactly the shape this project
has been wrong about before, so it was run instead.

`plain_select_delayed` and `simple_do_delayed` are the attempt: a `CALL ON`
handler that itself raises a *second* trapped condition, queued while the
first is being delivered, immediately in front of the block instruction --
the one route by which a condition might still be pending when an instruction
that evaluates nothing runs. The oracle delivers it at the handler's own
clause and never carries it forward:

```
     4 *-* zv = raiser1()
    ...
    15 *-*   h:
    16 *-*   say 'in h' raiser2()
    13 *-*     raiser2:
    14 *-*     raise user zy return 8
       >K>       "RESULT" => "8"
       >>>     "in h 8"
    18 *-*     g:                 <- the second handler, inside h's own clause
    19 *-*     say 'in g'
       >>>       "in g"
    20 *-*     return
    17 *-*   return
     5 *-* select                 <- reached with nothing pending
```

**The handler raises inside an ordinary clause of its own, `say 'in h'
raiser2()`, so that clause's own boundary delivers it and it can never reach
the following instruction.** The probe could not have found the route whatever
the interpreter did. The shape that carries a condition forward is a handler
whose *last* clause is the raise -- `h: raise user zy return 1`, which ends the
handler and queues in one instruction, leaving nothing behind it to deliver.
That shape is in `ir_dual_cases/loop-header-boundaries`'s own rows, two files
from the one being edited, and it was not reached for.

So the route was looked for with an instrument that could not see it, and the
conclusion drawn -- "these boundaries have no observable this crate has found"
-- was true of the probe rather than of the interpreter.

---

## Step 3: the failing test

`tests/trace_indent.rs`, which compares **raw** stderr, plus stdout and the
exit code, under both engines. Two cases added:

* `call_on_at_a_select_case_boundary` -- the Step 1 program.
* `call_on_at_a_select_case_boundary_inside_a_loop` -- the same `SELECT CASE`
  inside a `do zi = 1 to 1` body, so the expected indent is four rather than
  two and a fix that wrote an absolute level would pass the first and fail
  this one. M6 below is the mutation that proves it does that job.

Run before the fix, both tests red on both engines:

```
2 of 9 cases disagree with the oracle under TreeWalker, in 2 descriptors,
compared raw -- this comparison is the whole point of this file and does not
go through DEVIATION 0
  call_on_at_a_select_case_boundary (TreeWalker): stderr, raw
  call_on_at_a_select_case_boundary_inside_a_loop (TreeWalker): stderr, raw
```

and the same two under `Ir`.

**DEVIATION 0 is untouched, and its blindness to these two cases is measured
rather than described**: `deviation_0_collapses_every_wrong_answer_this_file_
holds` passed in that same red run, which is the assertion that each new
case's real wrong answer and the oracle's bytes are *equal* after
`normalize_stderr`. So `tests/corpus.rs` and `tests/trace_oracle.rs` could not
have caught either case, before the fix or after.

---

## Step 4: the fix

### Where it goes, and why there

`Interp::select_case` (`rust/crates/rexx-exec/src/run.rs`), after the `>K>
"CASE"` line is traced and before the clause boundary:

```rust
self.trace_keyword(indent, "CASE", &text);
// ...
self.settle_block_indent(true, indent);
Ok(value)
```

`select_case` is the one function both engines reach: the tree-walker calls it
inside the header's `in_clause`, and the compiled stream reaches it from the
single `Op::EvalExpr` of the header's clause region (`ir/compile.rs` emits
`Op::Clause` / echo / `EvalExpr` / `close_region`). Both boundaries are past
this line, so one write serves both. `indent` is already a local there -- the
`SELECT`'s own printed indent, captured on the way in, which the `>K>` line is
traced at.

`true` is not a placeholder. `RexxInstructionSelectCase::execute` has exactly
one path that closes the block again inside its own clause,
`conditionalPauseInstruction()`, which is interactive debug and not
implemented here; the ordinary path always leaves it open. That asymmetry with
a loop header -- whose `terminate` closes it whenever the iteration test fails
-- is why `open` stays a caller's answer rather than something the callee
works out.

### The signature change, and which side of the scope line it falls on

`settle_block_indent` was `(entered: bool, do_indent: usize, loop_indent:
usize)` and is now `(open: bool, clause_indent: usize)`, deriving the open
answer as `clause_indent + 2`. Task 2's two call sites are updated; **no
behaviour changes at either**, because `loop_indent` was `do_indent + 2` at
the single place both were computed.

**This is not a second divergence fixed and it is not a refactor for its own
sake.** The new caller has no `loop_indent`-shaped local; keeping three
parameters would mean writing `settle_block_indent(true, indent, indent + 2)`,
duplicating at the call site the block-level derivation the function's own
contract owns, and putting two same-typed indents side by side at a third
site. A swapped pair there produces exactly the two-column answer this
function exists to stop. The Global Constraints' exception is about a site the
fix mechanically forces; this is weaker than that -- **the fix could have been
written without it** -- so I am flagging it rather than claiming the exception,
and it carries its own mutation witnesses (M3 and M4) that no other case
covers.

### The `WHEN` case did not move

Checked three separate ways:

1. The `when_condition` probe against the live oracle, after the fix, both
   engines, three descriptors: identical (Step 2's table).
2. The raw blast-radius sweep below: the `WHEN` case's program is one of the
   109 swept and its bytes did not change on either engine.
3. It now ships as `call_on_at_a_when_condition_boundary` in
   `tests/trace_indent.rs`, so a future change that moves it is a red test
   rather than something a probe has to be re-run to notice.

---

## Step 5: the corrected `ir_dual_cases/loop-header-boundaries` note

The note recorded this gap as measured and open, which the fix makes false. It
is corrected in place rather than hedged or deleted. Before:

```
# **The one sibling boundary still off by the same two columns is a `SELECT
# CASE`'s.** `SELECT` is a block instruction too, so the oracle's counter is
# already one deeper when that clause ends: measured, `call on user zx name h`
# / `select case raiser()` echoes the handler at `    11 *-*     h:` there and
# `    11 *-*   h:` here, on both engines. A trap queued by a `WHEN`
# condition instead agrees, because a `WHEN` is not a block instruction and
# its clause ends at the level it echoed.
```

After:

```
# **A `SELECT CASE`'s clause boundary is the same rule one construct over.**
# `SELECT` is a block instruction too, so the oracle's counter is already one
# deeper when that clause ends, and `Interp::settle_block_indent` is what both
# constructs reach. A trap queued by a `WHEN` condition settles at the level
# its clause echoed at instead, because a `WHEN` is not a block instruction.
# Both are held against the oracle's own bytes in `tests/trace_indent.rs`.
```

The mechanism sentence survives because it is still true and is what the next
reader of a loop-header boundary needs; the divergence claim goes and is
replaced by a pointer to the file that now holds both halves. **No expected
block in that file changed** -- `ir_dual.rs` is green and no row moved, which
is consistent with the sweep finding no `.rex` outside the two new cases
changing on any descriptor.

---

## Step 6: the mutation witness, the blast radius, the gates

### The mutation witness

Seven mutations, each applied to `run.rs` from a `cp` backup, built, run as
`memcap 8G cargo test -p rexx-exec --release --no-fail-fast` (**`--no-fail-fast`
because otherwise the run stops at the first catcher and "nothing else caught
it" is unmeasured**), then restored from the copy, `touch`ed, `sha256sum -c`'d
and rebuilt. Only M1 is a deletion; the rest are substitutions that make a
call answer wrongly, which is what catches a case a deletion cannot -- a
deleted write preceded by another write of the same value leaves the field
holding the right answer anyway.

| | mutation | red cases (both engines unless noted) |
| --- | --- | --- |
| M1 | delete the `settle_block_indent` call from `select_case` | the two `select_case` cases |
| M2 | `settle_block_indent(true, ..)` -> `(false, ..)` in `select_case` | the two `select_case` cases |
| M3 | in `settle_block_indent`, the open answer becomes `clause_indent` | the two `select_case` cases, `loop_header_that_enters`, `until_test_that_runs_another_pass`, and `ir_dual_cases/loop-header-boundaries:143` |
| M4 | in `settle_block_indent`, `open` and closed swapped | all six loop and select cases, and `ir_dual_cases/loop-header-boundaries:143` |
| M5 | `scan_when` settles a `WHEN` like a block instruction | `when_condition_boundary`, **tree-walker only**, and `ir_dual`'s branch-shape population as an engine disagreement |
| M6 | `settle_block_indent(true, 0)` in `select_case` -- absolute rather than relative | **only** `select_case_boundary_inside_a_loop` |
| M7 | `condition_value` settles an `IF`/`WHEN` condition like a block instruction | `when_condition_boundary`, both engines, and the same `ir_dual` population |

Four things this matrix says that a count of red mutations would not:

* **M6 is why the nested case exists.** The top-level `SELECT CASE` sits at
  indent 0, so `settle_block_indent(true, 0)` and `settle_block_indent(true,
  indent)` are the same call there and it stays green. Only the nested case
  distinguishes a relative fix from an absolute one.
* **M1 and M2 have the same red set**, because `false` and no call at all leave
  the field holding the same wrong value -- `step_in_temps_frame` already put
  the clause's own indent there. M2 therefore adds nothing over M1; it is
  reported because a substitution that happens to coincide with a deletion is
  worth knowing about rather than presenting as independent evidence.
* **M5 reddens one engine only.** A plain `WHEN`'s compiled condition never
  enters `scan_when` -- it leaves its value in a register and reaches
  `condition_value` through `Op::Condition` -- so `scan_when` is a tree-walker
  path for this shape. That left the `WHEN` case's `Ir` half unwitnessed, which
  is what M7 was written for; it reddens the case on both engines.
* **M5 is also the provenance of `call_on_at_a_when_condition_boundary.wrong`.**
  No shipped build has ever got that case wrong, so its wrong answer had to be
  constructed -- and having to construct one is precisely the property the case
  exists to hold. `tests/trace_indent.rs`'s module doc now says so, because the
  file's own convention is that a `.wrong` came from a build that got the case
  wrong and a reader would otherwise assume a pre-fix build.

**"Can fail" is not "adds coverage", so M1 was also run against the whole
workspace**, not just `rexx-exec`: `memcap 8G cargo test --release --workspace
--no-fail-fast` goes red on `every_case_matches_the_oracle_on_the_tree_walker`
and `every_case_matches_the_oracle_on_the_compiled_engine` **and nothing else**.
So the two new `select_case` cases are the only things in the workspace that
catch the defect, and the suite without them is green under the mutation.

### The blast radius, raw -- which is the evidence

Population: **every `.rex` file git tracks under `rust/` with the new cases
staged -- 109 programs** (72 `corpus`, 13 `tests/trace_oracle`, 11 `bench`,
10 `tests/trace_indent`, 3 `crates/rexx-num/tests`), listed by

```bash
git ls-files 'rust/*.rex' 'rust/**/*.rex'
```

Each run under both engines with the pre-fix binary (`6229d3776`) and the
post-fix binary, comparing stdout, stderr and exit status as **raw bytes**:

* **654 streams compared** (109 programs x 2 engines x 3 descriptors);
  **4 changed**.
* All 4 are stderr, and all 4 are the two new `select_case` cases x two
  engines.
* **No corpus program changed on any descriptor**, no `bench` program changed,
  no `trace_oracle` program changed, and no stdout or exit status changed
  anywhere in the population. `call_on_at_a_when_condition_boundary.rex` is in
  the population and did not change, which is the sweep's own statement that
  the adjacent success held.

`git ls-files` reads the index rather than `HEAD`, so the new case files were
staged before the population was taken -- otherwise they would have been
absent from the very sweep meant to measure them.

### Through the harness, which is not evidence

The corpus differential reports **53 of 53 matching** and the `ootest` row
differential **4224 of 4259**, unchanged. Both comparisons apply
`normalize_stderr`, so **neither can move for this defect**; they are recorded
only so that nobody later reads them as though they had been instruments. The
raw sweep above is the number that means something, and it says the same
thing more strongly: those programs' bytes did not change at all.

### The gates

All from `rust/`, exit status read unpiped:

| gate | result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| the same clippy with `CARGO_TARGET_DIR` set to an empty directory | exit 0, log shows every workspace crate re-checked including `rexx-exec` |
| `memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, **1509 passed, 0 failed** |

The clean-target clippy is there because a same-session green is provisional:
a warm target directory can report exit 0 without the linter having looked at
the code. The log was checked for the workspace crate names rather than only
for the exit status.

---

## Found and deliberately not fixed

**1. A malformed probe of my own found nothing new, and the check is worth
recording.** `do zl` / `end zl` is not a labelled `DO` -- the label spelling is
`do label zl` -- and the oracle raises 10.3 with a full traceback at `rc 246`
where this crate prints `rexx-exec: 10.3: Unexpected or unmatched END.` at
`rc 120`. This is **not a new divergence**: `lib.rs:2842` records parse errors
as deliberately not reproduced byte for byte, with the reason at the site. The
probe was corrected to `do label zl` and came back byte-identical on both
builds. Recorded because a reader of the survey table would otherwise wonder
why a `SIMPLE DO` probe was rewritten.

**2. `ir_dual_cases/loop-header-boundaries` names the size of its own sets, in
the file I edited.** Its header block reads "TWO LOOP-HEADER BOUNDARIES
DIVERGE FROM THE ORACLE AND ARE NOT ROWS HERE" and "One row differs from the
oracle and says so in its own comment". Both are counts of mutable in-repo
aggregates in prose, which the house rule forbids. Neither is false today and
neither is a divergence, so under "do not fix any other divergence a task
happens to find" I left them and am naming them here instead. The paragraph I
was sent to correct is adjacent to the first of them.

**3. The two loop-header boundaries that paragraph records still diverge**, and
they are not mine: the zero-pass requeue (`G ran 3` here, `G ran 6` on the
oracle) and the `UNTIL` requeue (`G ran 6` here, `G ran 5`). Neither is an
indent quantity and the fix does not touch either. Untouched and still recorded
in that file.

> **The causal sentence that stood here -- "both are the elided-instruction
> family" -- was wrong, and round 4's Additional item and fix round 5 replaced
> it.** The zero-pass one
> is the enclosing-clause cause; the `UNTIL` one has **no established cause**
> and its program is not named anywhere, so nothing about it can be checked.

**4. The scope call in Step 4** -- `settle_block_indent`'s signature -- is the
one thing here I would want adjudicated rather than assumed. It changes two of
Task 2's lines with no behavioural effect, and my argument for it is in Step 4
under "which side of the scope line it falls on". If the ruling is that it
should not have been taken, the revert is mechanical: restore the third
parameter and write `settle_block_indent(true, indent, indent + 2)` at the new
call site.

---

# Fix round 1

**Commit:** `17d6dc6ec`. Recording and comments only -- no behavioural
line changed, and `select_case`/`settle_block_indent`'s bodies are byte-identical
to `932e6b710`.

## 1: the divergence the survey could not see, now recorded

**Reproduced here before recording it**, on both engines, rather than relayed.
`call on user zx name h` / `select case raiser()` with a handler that fails
instead of returning:

```
oracle                                    this crate (both engines)
     2 *-* call on user zx name h              2 *-* call on user zx name h
     3 *-* select case raiser()                3 *-* select case raiser()
     9 *-*   raiser:                           9 *-*   raiser:
    10 *-*   raise user zx return 2           10 *-*   raise user zx return 2
       >K>     "RESULT" => "2"                   >K>     "RESULT" => "2"
       >K>   "CASE" => "2"                       >K>   "CASE" => "2"
    11 *-*     h:                              11 *-*     h:
    12 *-*     say 1/0                         12 *-*     say 1/0
    12 *-*     say 1/0                         12 *-*     say 1/0
     3 *-*   select case raiser()               3 *-* select case raiser()
Error 42 ... line 12                      Error 42 ... line 12
Error 42.3 ...                            Error 42.3 ...
```

`rc 214` and empty stdout on all three. **One line differs**: the enclosing
frame's traceback line, two columns short. The handler's own three lines above
it agree -- which they did not before `932e6b710`; running the pre-fix binary
on this same program shows four lines differing, so the fix repaired three of
them and left this one exactly as it was. **Not caused by the fix and not
fixed here.**

The loop-header analogue was run too and diverges identically, one line, both
engines:

```
oracle       3 *-*   do zi = 1 to raiser()
this crate   3 *-* do zi = 1 to raiser()
```

That is the instance `ir_dual_cases/loop-header-boundaries` already records as
"a traceback quantity rather than a transfer one". The `SELECT CASE` instance
is now recorded in the same file beside it, with both transcripts on lines of
their own so the two columns the sentence is about are visible.

## 2: why the survey could not see it, which is the transferable part

**Every probe in Step 2's survey used a handler that returns.** Twenty probes,
two crate builds, both engines, three descriptors -- and one instrument shape.
A handler that *fails* is a second observable of the same boundary, and it
found something on the first program it was pointed at.

> **This paragraph said the survey's conclusion was unaffected -- that `SELECT
> CASE` was still the last *boundary* divergence and the constructs reported
> clean were still clean. Fix round 2 refutes it**: a plain `SELECT`'s boundary
> has the same defect, reachable by a route none of these probes used. The
> reassurance was written in the same round that had just found the survey's
> instrument too narrow, which is the wrong moment to be reassured.

What the miss says is narrower and worse -- **a survey's breadth is the product
of its cases and its observables, and mine multiplied twenty by one.** The
count of probes read as thoroughness and was measuring only one of the two
factors.

## 3: three wording fixes

* `run.rs`, `settle_block_indent`'s doc -- "would put two same-typed indents
  side by side **at every call site**" quantified over call sites, which is a
  mutable in-repo aggregate. Now "at a call site", which says the same thing
  and cannot rot.
* `run.rs`, `select_case` -- "and **no program run so far** tells the
  difference" was a statement about my own probing history rather than about
  the code. Removed. The paragraph also restated at length an argument
  `Select`'s own arm already makes ("a plain `SELECT` simply has nothing that
  could have queued"), so the restatement is replaced by a citation of it. What
  survives is the part only a reader of *this* line needs: a plain `SELECT`
  never reaches this function at all, so nothing settles its boundary.
  ~~and that is not a gap~~ -- **the trailing clause was false and fix round 2
  removed it from the code**; the citation it pointed at was the same false
  inference, and is corrected there too.
* This report, Step 2 -- "outside this repo and therefore citable rather than
  assertable" was false about `interpreter/`, which is git-tracked in this
  repository. Corrected in place at the sentence itself: the citation stands on
  the real axis (an enumeration that cannot change without the sentence being
  reread), and `cmp` confirms **the files that enumeration cites** are
  byte-identical to the oracle tree's copies. **Narrowed in fix round 2**: the
  first wording claimed the whole tree, and `diff -rq` reports
  `classes/NumberStringClass.cpp` differing because the oracle repo carries an
  uncommitted local edit.

**Not fixed, as instructed:** the set-cardinality prose in
`loop-header-boundaries`'s own header block, which goes to the whole-branch
review with the other deferred minors.

## What was re-run, with its output

A recording-and-comments round, so the covering checks are the ones that read
what changed:

| check | why it covers this round | result |
| --- | --- | --- |
| `cargo test -p rexx-exec --test ir_dual` | the file the new note lives in is parsed and run by it | `9 passed; 0 failed` |
| `cargo test -p rexx-exec --test trace_indent` | the cases the fix is pinned by, unchanged and still raw | `11 passed; 0 failed` |
| `cargo fmt --all --check` | the doc comments reflowed | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | the two `run.rs` comment edits | exit 0 |
| `git diff 932e6b710 -- run.rs`, comment lines only | the claim that no behavioural line moved | additions and deletions are all `//` lines |

The full workspace suite was **not** re-run: nothing executable changed, and
the two test binaries that read the edited files were both run in full.

---

# Fix round 2

**Commit:** `8b6989aa4`. Comments, one case file's prose, and this
report. No behavioural line changed.

**Everything below was re-measured here rather than taken from the round's
brief.** All of it reproduced; nothing in the brief was wrong. The control is
`6229d3776`, the commit immediately before `932e6b710`, which isolates this
task's own change more tightly than the plan's base does.

## 1: the comment was false, and the measurement says why

`select_case`'s new note claimed a plain `SELECT` needs no boundary settled --
"its clause has nothing that could have queued a condition for the boundary to
deliver". The premise is true and the conclusion does not follow: **a boundary
delivers whatever is pending, not only what its own clause queued.**

A `CALL ON` handler whose last clause is `raise ... return` ends the handler
and queues a second trapped condition in one instruction, leaving nothing
behind it to take the delivery. The next clause boundary takes it. Measured
under `trace r`, both engines:

```
oracle                                    this crate (both engines)
     4 *-* zn = raiser1()                      4 *-* zn = raiser1()
    11 *-*   raiser1:                         11 *-*   raiser1:
    12 *-*   raise user zx return 5           12 *-*   raise user zx return 5
       >K>     "RESULT" => "5"                   >K>     "RESULT" => "5"
       >>>   "5"                                 >>>   "5"
    13 *-*   h:                                13 *-*   h:
    14 *-*   raise user zy return 1           14 *-*   raise user zy return 1
       >K>     "RESULT" => "1"                   >K>     "RESULT" => "1"
     5 *-* select                              5 *-* select
    15 *-*     g:                              15 *-*   g:
    16 *-*     say 'G ran' sigl                16 *-*   say 'G ran' sigl
       >>>       "G ran 5"                        >>>     "G ran 5"
    17 *-*     return                          17 *-*   return
```

`select` is line 5 and `SIGL` is 5, so it is **the plain `SELECT`'s own
boundary** delivering `g` -- and the handler lands two columns short. That is
this task's defect, on the construct the comment said had nothing there.
stdout and `rc 0` agree throughout.

### The discriminator, which is what refutes the comment's reasoning

The same program with `select case 1` -- a constant scrutinee, which queues
nothing either:

| build | `select case 1` + requeue, raw stderr, both engines |
| --- | --- |
| `6229d3776` (before this task's fix) | **differs**, the same two columns |
| `932e6b710` (after) | **byte-identical to the oracle** |

So the fix repaired this route for `SELECT CASE` precisely because
`select_case` settles unconditionally, and the plain `SELECT` is unfixed
because nothing settles it. **What decides is whether anything settled the
boundary, never what the clause queued** -- and "the clause queues nothing" is
the reasoning the measurement refutes, not merely a claim it contradicts.

Both comments corrected, neither hedged: the one this task added, and the
pre-existing sentence at `Select`'s own arm it cross-referenced ("a plain
`SELECT` simply has nothing that could have queued"), which is the same false
inference and was confirmed to be so.

**Not fixed.** Pre-existing, and the Global Constraints say to record.

## 2: why the survey missed it, corrected in Step 2

Step 2's headline claimed `SELECT CASE` was the last of them. It is not, and
the correction is at the headline itself rather than only here.

`plain_select_delayed`'s "never had it" was a **probe artifact**. That probe's
handler raised inside an ordinary clause of its own, `say 'in h' raiser2()`, so
that clause's boundary delivered it and it could never reach the following
instruction. **The probe could not have found the route whatever the
interpreter did**, which makes its green a property of the instrument.

Round 1 already recorded that the survey ran twenty probes with one observable.
This is the same defect one level down: it also ran them with **one shape of
the observable it did have**. The shape that carries a condition forward --
`h: raise user zy return 1` -- is not exotic and is not far away: it is sitting
in `ir_dual_cases/loop-header-boundaries`'s own rows, in the file this task
edited twice. **A probe was written for the route instead of borrowed from the
place the route was already demonstrated**, and rewriting it is what lost the
property that made it work.

## 3: the `DO` sibling, which is a different family and worse

The same requeue route pointed at a plain `do` / `end`, **with no `TRACE`
anywhere in the program**:

```
oracle          this crate (both engines)
G ran 5         body
body            G ran 6
after set 5     after set 5
```

The oracle delivers the second handler at the `do` clause's own boundary,
before the body runs, with `SIGL 5`. This crate runs the body's first clause
first and delivers at *that* clause's boundary, with `SIGL 6`. **Wrong order
and a different `SIGL`, on stdout, `rc 0` on both sides.** `do label zl`
behaves the same; `if ... then` is clean on this route; measured identical on
`6229d3776`, so it is untouched by this task.

**This is not an indent and not a trace quantity**, and it is recorded saying
so, because a reader who files it beside the indent divergences will mis-rank
it. Everything else this plan recorded needs `TRACE` on to observe; this needs
nothing.

> **The attribution that stood here -- the elided-boundary family, "a clause
> boundary at a position where this crate has no instruction to run one at" --
> is false, and the Additional-item section below replaces it.** The boundary
> exists and carries the right line; it is open across the body. The claim was
> inherited from the case file's header block, which was itself wrong.

## 4: the `cmp` claim, narrowed to what was checked

Round 1's own correction over-claimed: it said `interpreter/` is
"byte-identical to the oracle tree's copy, checked with `cmp`". `diff -rq`
reports `classes/NumberStringClass.cpp` differing, because the oracle repo
carries an uncommitted local edit. The **files the enumeration cites** are
byte-identical, checked file by file, and that is now all the sentence claims.

A false statement introduced by the correction written to remove a false
statement, which is this project's most reliable source of them.

## 5: the transcript that implied a program it did not name

Round 1's recorded transcript named `call on user zx name h` / `select case
raiser()` -- which puts `select case` on line 2 -- beside a transcript showing
`3 *-*`. The program it was measured on has `trace r` above them. Named, so
the two adjacent examples in that file no longer imply different programs.

## Recommended, not done in this round

`select case 1` + requeue is **a route this task's fix repaired that nothing
pins** -- it is not among the `trace_indent` cases, all of which use a
scrutinee that raises. The artefacts for it already exist: the oracle capture
is taken, and `6229d3776` produces the wrong answer a `.wrong` file needs. It
is not added here because this round was scoped to comments, prose and the
report, and adding a case changes what the suite asserts. **One case file away
if you want it.**

## What was re-run, with its output

| check | result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, **1509 passed, 0 failed** |
| `cargo test -p rexx-exec --test ir_dual` (parses the edited case file) | `9 passed; 0 failed` |
| `cargo test -p rexx-exec --test trace_indent` | `11 passed; 0 failed` |
| `git diff 17d6dc6ec -- run.rs`, filtered for non-`//` lines | empty -- every changed line is a comment |

---

# Fix round 2, residual

**No commit.** `.superpowers/` is gitignored (`.gitignore:19`, confirmed with
`git check-ignore -v`), so this report is untracked and the tree is unchanged.
No source file was touched in this pass.

The flagged residual was one of **four** stale copies. The finding named one;
re-reading the neighbourhood found three more, and two of them were in the same
two paragraphs the round-1 write-up used to describe its own corrections.

| where | the stale text | why it was false |
| --- | --- | --- |
| round 1, section 2 | "the survey's conclusion is unaffected: `SELECT CASE` is still the last *boundary* divergence, and the constructs reported clean are still clean" | refuted by round 2 -- a plain `SELECT` has the same defect |
| round 1, section 3 bullet | "...so nothing settles its boundary, **and that is not a gap**" | the exact conclusion round 2 removed from the code, left standing in the prose describing that code |
| round 1, section 3 bullet | "`cmp` confirms this repo's copy is byte-identical to the oracle tree's" | the unnarrowed claim -- **the one flagged** |
| status block, top | "One thing found and deliberately not fixed, one pre-existing prose defect..." | a count of this document's own findings, and two rounds have added three more |

All four corrected, none hedged, each marked so the audit trail survives.

**The shape, since it is now three rounds running.** Round 1 introduced a false
statement while removing one. Round 2 corrected that and left three copies of
its own corrections stale. **The correction lands at the sentence the finding
quotes, and the copies are wherever that sentence was summarised** -- which, in
a report that documents its own fix rounds, is systematically *inside the
section describing the fix*. Round 1's section 3 bullet was the write-up of the
comment fix, and it reproduced the false conclusion it was reporting the
removal of.

The reviewer's instruction -- re-read the neighbourhood, not the finding --
is what found three of the four. A grep for the flagged phrase alone would have
found one.

**Also corrected, unflagged**: the status block counted its own found-and-not-
fixed items. Replaced with a pointer to the section, which cannot rot as rounds
add to it.

## What was re-run

Nothing executable changed, so nothing was re-run. The last full verification
stands at `8b6989aa4`: fmt 0, clippy 0, `memcap 8G cargo test --release
--workspace --no-fail-fast` exit 0 with 1509 passed / 0 failed, and `git diff`
filtered for non-comment lines empty in both changed files.

---

# Fix round 3

**Commit:** `0a893bc31` (base `4ed992406`). One case file's prose and this report. No
behavioural line changed.

## 1: the rejected Important, and why it is the argument for naming programs

The re-review held that the `DO` entry's crate transcript should read `after
set 6` rather than `after set 5`, on the ground that "the only source of a `5`
on that line is the delivering clause's `SIGL`". **The number stands.** On the
program that was measured, the last statement is `say 'after' zv zn` and that
`5` is `zn` -- the value `raiser1` returned through `raise user zx return 5`.
It is not a `SIGL` and does not move when the delivering clause does.

**The reviewer was not careless, and this is the part worth keeping.** It
reconstructed a program from the recorded oracle triple, and its reconstruction
reproduces that triple exactly. Both measured here, both engines:

| program's last statement | oracle | this crate |
| --- | --- | --- |
| `say 'after' zv zn` (the one measured) | `G ran 5` / `body` / `after set 5` | `body` / `G ran 6` / **`after set 5`** |
| `say 'after' zv sigl` (the reconstruction) | `G ran 5` / `body` / `after set 5` | `body` / `G ran 6` / **`after set 6`** |

**Two different programs, byte-identical oracle transcripts, different crate
transcripts.** The reviewer measured its own program correctly and reported the
result as a defect in a record that was right.

The collision has a cause worth writing down rather than treating as bad luck:
`raise user zx return 5` happens to return **the same number as the `do`
clause's own line**, so on the oracle -- which delivers at that clause's
boundary -- `zn` and `SIGL` are both `5` and the two readings are
indistinguishable in the output. Only this crate, which delivers one clause
later, separates them. A record that did not name its program could not be
checked at all, and reading it more carefully would not have helped: **both
readings are consistent with every byte recorded.**

That is a better argument for the file's own "regenerate a row by running its
program" rule than any restatement of it, which is why it is here and not only
in a commit message.

## 2: what changed

Both entries added in fix round 2 now carry their program in full, laid out in
two columns with the reading order stated, so line numbers are recoverable.
**Checked rather than asserted**: both programs were mechanically extracted
back out of the committed comment prose and `diff`ed against the probe files
they were measured from -- identical, 17 lines each.

The `DO` entry also states outright that the `5` ending its third line is `zn`
and not a `SIGL`, and records the alternative reading and what it produces, so
the next reader who spots the coincidence finds it already answered.

The heading `AND THE ONE NOT TO FILE BESIDE THESE` said one thing while the
entry did another -- it is filed beside them, deliberately, and what was meant
was do not *rank* it with them. Now `RECORDED HERE BUT NOT RANKED WITH THESE`.

**This is the same defect round 1 fixed for the `SELECT CASE` entry, recurring
in the commit that fixed it.** Round 1 named that entry's program because its
transcript implied one it did not name; round 2 then added two entries with no
program at all. The lesson was applied to the instance and not to the practice.

## 3: one discrepancy in the round's own brief, for the record

The brief asked that the `DO` entry "state that the last statement prints
`zw`". The program's variable is `zn`; there is no `zw` in it. The substance is
identical -- that element is the returned value rather than a `SIGL` -- and the
entry names `zn`, which is what the measured program contains. Recorded because
copying the name across unchecked is exactly the failure this round is about.

## Deferred, untouched

The file's subject line at `:1-2` has drifted from what the file now holds. Not
touched, per instruction; it goes to the whole-branch review with the other
prose minors.

## Not started

Moritz's decision that the `DO` divergence is fixed before Phase 5 as its own
task is noted. Nothing here anticipates it: the entry records current behaviour
and stays a record until that task converts it.

## What was re-run, with its output

| check | result |
| --- | --- |
| both programs reconstructed from the comment prose, `diff`ed against the probes | identical, both |
| `cargo test -p rexx-exec --test ir_dual` (parses the edited file) | `9 passed; 0 failed` |
| `cargo test -p rexx-exec --test trace_indent` | `11 passed; 0 failed` |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, **1509 passed, 0 failed** |
| `git status --short` before staging | one modified path, the case file |
| `git diff 8b6989aa4 -- crates/rexx-exec/src/run.rs` | empty -- `run.rs` untouched since fix round 2 |

`git diff 8b6989aa4 --stat` also lists two plan files. **Those are not this
task's**: `4ed992406` landed between fix round 2 and this round, adding the new
`DO` task and suspending perf work for Phase 5. `git status` was read before
staging and showed the case file alone, and only that path was staged.

---

# Fix round 4

**Commit:** `672619950` (base `0a893bc31`). One case file's prose and this
report. No behavioural line changed; `git diff 0a893bc31 -- run.rs` is empty.

## The false singular in the plain-`SELECT` entry

The entry introduced "The stderr line that does not:" and showed one pair.
Re-measured here on both engines: **the whole of the handler's activation
differs**, not one line of it. The entry now shows all of it:

```
oracle                            this crate
    15 *-*     g:                     15 *-*   g:
    16 *-*     say 'G ran' sigl       16 *-*   say 'G ran' sigl
       >>>       "G ran 5"               >>>     "G ran 5"
    17 *-*     return                  17 *-*   return
```

The prose now reads "the handler's activation, every line of it two columns
short", which says the true thing **without counting** -- the singular was
false and a replacement plural would have been a cardinality claim in prose,
which the house rule forbids either way.

**The block was generated from the captured bytes rather than retyped**, and
then checked back: each row splits at one fixed column, and both cells of every
row compare equal to the oracle capture and the crate capture respectively.
That is the same discipline round 3 applied to the programs, applied to the
transcripts.

## Why this was worth correcting rather than deferring

The `SELECT CASE` traceback entry a few lines above **genuinely is one line**,
and says so; re-measured here, its differing-line count is 1 against this
entry's 4. Two adjacent entries, one truthfully singular and one falsely
singular, teach a reader the wrong thing about how entries in this file are
written -- worse than either error alone, because the true one lends the false
one credibility.

## The shape, now that it has appeared at four scales

Every round of this task has been the same defect at a different granularity,
and the progression is the useful part:

| round | what was recorded | what was understated |
| --- | --- | --- |
| 1 | the `SELECT CASE` traceback | the program it was measured on |
| 2 | a plain `SELECT` and a plain `DO` | both programs, again |
| 3 | the `DO` entry's transcript | which element the `5` was |
| 4 | the plain-`SELECT` transcript | how much of it differs |

**A record is written by someone who has just run the thing, and every one of
these omissions is invisible from that seat** -- the program is on screen, the
variable is obvious, the extent of the diff was just read. Each is only missing
to the reader, which is why none of them was caught by re-reading and all four
were caught by someone reconstructing from the record alone. The countermeasure
that has actually worked here is mechanical: extract the artefact back out of
the record and compare it to what was measured. Round 3 did that for the
programs and round 4 for the transcripts, and both found the record already
correct -- which is what a check is supposed to do most of the time.

## What was re-run, with its output

| check | result |
| --- | --- |
| the block's rows split at one column, both cells vs the captured bytes | **all four rows match** |
| differing-line counts re-measured | plain `SELECT` **4**, both engines; `SELECT CASE` traceback **1** |
| `cargo test -p rexx-exec --test ir_dual` | `9 passed; 0 failed` |
| `cargo test -p rexx-exec --test trace_indent` | `11 passed; 0 failed` |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, **1509 passed, 0 failed** |
| `git status --short` before staging | one modified path, the case file |
| `git diff 0a893bc31 -- crates/rexx-exec/src/run.rs` | empty |

## Additional item (commit `b27b950f1`): the `DO` entry's cause was misattributed

The entry filed the `DO` divergence under the elided-boundary family -- "the
oracle runs a clause boundary at a position where this crate has no instruction
to run one at". **That is false for this case. The boundary is not missing; it
is in the wrong place.** Re-measured and re-derived here rather than copied.

**The code path, read rather than assumed.** `in_stepped_clause` is
`enter_stepped_clause` / `work(self)` / `leave_stepped_clause`, so the boundary
runs after `work`; `work` is `step`, whose `Do`/`Loop` arm calls `run_loop`,
which resolves every iteration inside that one `step` call -- the crate's own
doc comment at `step_in_temps_frame` says so in as many words. So the `DO`'s
clause stays open across the body and `leave_clause` does not run until the
loop is over. The oracle's `RexxInstructionSimpleDo::execute` returns once it
has opened the block. Delivery goes to whichever boundary arrives first.

**The discriminator, measured here.** The same program with the body's `say`
removed -- so `do` is line 5 and `end` is line 6:

| | oracle | this crate, both engines |
| --- | --- | --- |
| empty body | `G ran 5` / `after set 5` | `G ran 5` / `after set 5` -- **agrees** |
| one body clause | `G ran 5` / `body` / `after set 5` | `body` / `G ran 6` / `after set 5` |

`SIGL` is 5 and not 6 or 7, so this crate's `DO` clause boundary **exists,
carries the right line, and delivers**. One clause in the body moves it. "No
boundary" and "boundary too late" predict different things here and it comes
out too late.

**A case that agrees is what pins a cause**, and this file had no instance of
that shape. It is now recorded beside the failing one.

**The contrast with the plain `SELECT` above it, both halves measured.**
`Select`'s arm wraps only the `CASE` evaluation in `in_clause` and closes it
before the `WHEN`s run, so that boundary is correctly placed: plain `SELECT`
gets `SIGL` right and only the indent wrong. `DO`'s is misplaced: under `trace
r` its handler's own lines are byte-identical to the oracle's -- the indent is
right -- and only their position and `SIGL` differ. **Two defects, not one**,
and the `SELECT` one is `settle_block_indent` living in `select_case` while the
`DO` one is structural.

### The header block was wrong the same way, and its own bullet said so

The header closes with "Both are the elided-instruction family". Its **first**
bullet already states the correct mechanism in its own text -- "the `DO` clause
encloses the whole loop here, so its boundary runs after the loop rather than
before the body" -- so the closing sentence contradicted the bullet it was
summarising. Re-measured before re-attributing: oracle `after` / `G ran 6`,
this crate `G ran 3` / `after`, both engines, exactly as recorded. `G ran 3`
names the `DO`'s own line, which is the same evidence the empty-body probe
gives.

Corrected to distribute: the zero-pass one is the enclosing-clause cause.
Numbers untouched -- only the attribution was wrong.

> **The other half of that correction was itself wrong and fix round 5
> withdraws it.** It re-endorsed the `UNTIL` bullet as the elided-instruction
> family. Nothing measurable supports that, and the bullet cannot be checked
> at all, because it does not name its program.

**Worth noting where the false attribution came from.** It was not invented: it
was taken from that closing sentence, which was the nearest authority and was
itself wrong. A record inherited a summary's error because the summary sat
above the bullets it summarised, and the bullet with the right answer was
further away than the sentence with the wrong one.

### What was re-run for this item

| check | result |
| --- | --- |
| empty-body probe, oracle vs both engines | **agrees**, `G ran 5` / `after set 5` |
| one-body-clause probe, re-measured | oracle `G ran 5` / `body` / `after set 5`; crate `body` / `G ran 6` / `after set 5` |
| the same program under `trace r` | handler's own lines byte-identical; position and `SIGL` differ |
| zero-pass bullet re-measured before re-attributing | oracle `after` / `G ran 6`; crate `G ran 3` / `after`, both engines |
| `in_stepped_clause`, `step`'s `Do` arm, `run_loop` | read in source; boundary runs after `work`, `run_loop` resolves the loop inside one `step` |
| all three programs reconstructed from the comment prose vs their probes | **identical**: 17, 17, 16 lines |
| `cargo test -p rexx-exec --test ir_dual` | `9 passed; 0 failed` |
| `cargo test -p rexx-exec --test trace_indent` | `11 passed; 0 failed` |
| `cargo fmt --all --check` / `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 / exit 0 |
| `memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, **1509 passed, 0 failed** |
| `git diff 672619950 -- crates/rexx-exec/src/run.rs` | empty |

---

# Fix round 5

**Commit:** `613983c0b` (base `b27b950f1`). One case file's prose and this
report. No behavioural line changed.

## 1: the `UNTIL` attribution is withdrawn, not replaced

Round 4's Additional item split the header block's two bullets and **re-endorsed**
the `UNTIL` one as the elided-instruction family. That was wrong in a way the
split itself should have caught: the zero-pass bullet was re-attributed because
its own text named a mechanism and its numbers reproduced, and the `UNTIL`
bullet got the leftover label without either check being possible.

**It is not checkable, because the bullet does not name its program.**
Reconstructing one from its own words has now been tried three times and given
three answers, none of them the bullet's:

| reconstruction | oracle | this crate |
| --- | --- | --- |
| the bullet's own text | delivers at the `END`'s clause, `G ran 5` | at the clause after the loop, `G ran 6` |
| the reviewer's | `after` / `G ran 6` | `G ran 5` / `after` -- the two sides swapped |
| the controller's, re-measured here | `after unset` / `G ran 7` | `G ran 6` / `after set`, both engines |

So the entry now says the cause is **not established** and the bullet **cannot
be checked**. Its numbers are left alone: choosing which program produced them
would be inventing a fourth reconstruction and filing it as the third. The
bullet stays -- it records a real observation someone made.

**This is the task's own lesson arriving one level up.** The two entries added
in fix round 2 name their programs because round 3 required it; the
pre-existing bullet beside them does not, and that is exactly the difference
between one that can be checked and one that cannot. The rule was applied to
the entries this task wrote and not to the neighbour it was reasoning from --
which is the same shape as inheriting the header's wrong attribution in the
first place.

## 2: the reconstruction that runs the other way

Re-measured here from Task 6's Step 1b rather than copied:

```
oracle       after unset / G ran 7
this crate   G ran 6 / after set        (both engines)
```

**Every other shape in the case file has this crate delivering later than the
oracle; this one delivers earlier.** Recorded in the case file with a pointer
to Step 1b, which names the program, rather than a second copy of it -- and the
pointer resolves: reconstructing from that description reproduced both
transcripts exactly, which is the check the `UNTIL` bullet fails.

## 3: the causal account's engine scope

The account was written from the tree-walker's path, and `step`'s own arm says
a promoted `DO` never comes through there -- so an implementer could fix that
path, measure the other, and see no change. Added, after verifying it in
source: `crate::ir::Op::LoopRun` sits inside the header's `Op::Clause` region
(`compile.rs` pushes `close_region` immediately after it) and calls
`run_loop_with_header`, which resolves the loop before that region closes.
Different code, same shape, which is why both engines diverge.

## 4: the stale copies -- three, not the two named

Swept rather than patched, as asked.

| where | stale claim | state |
| --- | --- | --- |
| "Found and deliberately not fixed", item 3 | "Both are the elided-instruction family" | bannered, corrected |
| round 2, section 3 | "It is the elided-boundary family ... which the case file's header block already describes" | bannered, corrected |
| **round 4's Additional item** | "the `UNTIL` one **is** the elided-instruction family" | **not named in the finding**; bannered, withdrawn |

The third was in the section that corrected the first two. Every remaining
occurrence of the word in this report is now inside a correction banner or is a
quotation of a superseded sentence.

**Also caught in the same pass, self-inflicted**: the banners I wrote for those
three referred to a "fix round 6", which does not exist -- this is round 5 and
the `DO` half was corrected in round 4's Additional item. Corrected before
committing. A cross-reference to a round number is a mutable aggregate like any
other, and it rotted inside the same edit that created it.

## The pattern, at its final count for this task

Six rounds, and every one of them the same defect at a different scale: the
program (round 1), both programs (round 2), which element a number was (round
3), how much of a transcript differed (round 4), the cause (round 4's
additional item), and the cause of the neighbour that the cause was reasoned
from (round 5). **Each round applied the previous round's lesson to the
instance it was handed and not to the practice**, which is why the next round
found the same shape one step out.

The only thing that has reliably caught it before a reviewer did is mechanical
reconstruction from the record -- and note what it cannot reach: it verifies
that a record's program and transcripts are self-consistent, and says nothing
about whether the *causal sentence* attached to them is true. Rounds 4 and 5
were both causal, and both were caught by a person, not a script.

## What was re-run, with its output

| check | result |
| --- | --- |
| the `UNTIL` reconstruction, oracle vs both engines | oracle `after unset` / `G ran 7`; crate `G ran 6` / `after set` |
| `Op::LoopRun` / `close_region` order, `run_loop_with_header` | read in source; region closes after the loop runs |
| all three programs reconstructed from the comment prose vs their probes | **identical**, all three |
| `cargo test -p rexx-exec --test ir_dual` | `9 passed; 0 failed` |
| `cargo test -p rexx-exec --test trace_indent` | `11 passed; 0 failed` |
| `cargo fmt --all --check` / `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 / exit 0 |
| `memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, **1509 passed, 0 failed** |
| `git status --short` before staging | one modified path, the case file |
| `git diff b27b950f1 -- crates/rexx-exec/src/run.rs` | empty |

---

# Final-review fixes

**Commit:** `7a0b31245` (base `613983c0b`). Comments and case-file prose. **No behavioural
line changed** -- `git diff 613983c0b -- run.rs` filtered for non-`///` lines is
empty.

All five re-derived here rather than accepted, and all five were real.

## I1 -- a true sentence left standing under a new subject

The `DO` account's closing sentence -- "Delivery then goes to whichever
boundary arrives first: with a body that is the body's first clause" -- is
about **this crate**. `613983c0b` inserted the compiled-path paragraph between
it and its subject, leaving the oracle as the nearest antecedent, and the
oracle does the opposite: it delivers at the `DO`'s own boundary, which is the
entry's whole point and what the transcript twelve lines down shows.

Fixed by naming the side in **every** sentence of the account rather than
relying on adjacency, and by moving the compiled-path paragraph after both
sides are stated. Adjacency is what broke; restoring it would leave the same
trap for the next insertion.

**This is the session's dominant failure mode occurring inside the correction
that named it** -- an edit that leaves a true sentence standing under a new
subject, exactly what the round-2 residual section describes for copies.
Splitting a paragraph is an edit that changes a sentence's subject without
touching the sentence.

## I2 -- the bullet still asserted what the block below withdrew

Both were live. The bullet's last sentence carried the elided-`END`
attribution; the block below said the cause was unestablished. The bullet now
says it names no program, so its observation cannot be re-run and its cause is
not established, and points at the withdrawal -- with what stood there kept
parenthetically, because it records what someone once concluded.

The reviewer's fourth reconstruction (line 5 the body, line 6 the `END`, and
delivering earlier rather than later) is **not** adopted as the bullet's
program. Four reconstructions, four results, none the bullet's -- which
strengthens "cannot be checked" rather than settling it.

## I3 -- both halves of the sentence were false, and I measured both

Re-ran the mutation across the whole workspace with `--no-fail-fast`:

| | result |
| --- | --- |
| catchers | **three**: the unit test, `ir::golden_tests::a_block_has_an_empty_header_region_and_a_do_over_for_echoes_both_its_target_and_count`, and `ir_dual`'s `loop-header-values:192` |
| is `loop-header-values` engine-to-engine only? | **no** -- I extracted its stanza and ran it on the live oracle: byte-identical, and *the mutation reddening it at all proves it*, since the mutation moves both engines the same way and an engine-to-engine check could not see it |

**Fixed by deletion, not correction.** A replacement naming the other catchers
would rot exactly as the original did -- it is the "what every other site does"
shape the house rule forbids. What survives is what a reader of that test
needs: the bytes are the oracle's, captured when and how, and
`HeaderRole::OverFor::keyword()` decides whether the crate emits the line. The
measurement lives here and in the commit message, which is where history goes.

**I nearly shipped the correction with two fresh violations in it** -- a first
draft said "It used to add that nothing else in the tree pinned the line"
(history in a comment) and named the two other catchers (a cross-test claim).
Caught on re-reading my own replacement, which is the fifth time this session a
fix has carried a new defect of the class it was fixing.

## I4 -- same shape, same treatment

"no other test or case file runs that shape under trace at all" is false: two
unit tests added in the same commit run it under `trace i` and `trace r`.
Deleted rather than corrected, for I3's reason. The row keeps its provenance,
which is what earns its place.

## I5 -- "Both quantities" / "The two places" with three bullets

Mine: `932e6b710` added the third bullet and left the count. Fixed by naming
the set -- "Every quantity below is the oracle's own `settings.traceIndent`"
and "Where that derivation parts company with the counter" -- rather than by
changing two to three, which would rot on the next bullet.

## Minors taken in this pass

* `loop-header-boundaries` set cardinalities: "Two rows are about a
  boundary..." -> "The requeue rows..."; "Three rows are about the header's
  own ordering" -> "Other rows..."; "One row differs from the oracle and says
  so" -> "A row that differs from the oracle says so"; **"TWO LOOP-HEADER
  BOUNDARIES DIVERGE..."**, which this task's own entries had already made an
  undercount, -> "LOOP-HEADER BOUNDARIES THAT DIVERGE ... ARE NOT ROWS HERE",
  with "Both were measured" -> "Each was measured".
* `oracle-crashes.txt`: "Three of the four are ... and one is ..." -> the modes
  named inline as an either/or, with each entry saying which it is.
* The citation of `task-4bp-report.md`, a gitignored path, dropped from the
  paragraph that withdraws the attribution it was supporting.

Carried, as instructed: the stale subject line at `loop-header-boundaries:1-2`
and the `trace_indent.rs` cosmetics.

## The sweep

Swept **the lines this plan added** rather than the files wholesale -- the
shape is a claim some later edit invalidated, and only this plan's own edits
are in reach. `git diff 625d653fb..HEAD -- rust/crates rust/corpus`, 2133 added
lines, grepped for exclusivity and for cardinality-plus-noun.

Beyond I3/I4/I5 it found two, and **neither is changed**:

* `run.rs:13979` "nothing else in the suite can see these bytes" (Task 1's).
  Not the same defect: it is derived from a stated mechanism -- DEVIATION 0
  collapsing the indent -- rather than from an enumeration of tests, and
  `trace_indent.rs` asserts that same mechanism in a test that can fail.
  Rewriting another task's substantiated comment at final review, with no
  measurement of my own, is the over-reach the constraints warn against.
* `trace_indent.rs`'s "Two counts, because one case can contribute up to three
  mismatches" and "A case is three files". Both describe enumerations the code
  itself carries (`["rex", "expected", "wrong"]`, the three descriptors), which
  is the carve-out the house rule states: code cannot rot the way prose about
  it does.

The wider sweep across the whole of every touched file returns a large set
dominated by pre-existing, locally-checkable uses ("the only reader and the
only writer" of a named field). That is a different job from this finding and
is not attempted here.

## What was re-run, with its output

| check | result |
| --- | --- |
| mutation E, whole workspace, `--no-fail-fast` | **3 catchers**, named above |
| `loop-header-values` `DO OVER ... FOR` stanza vs the live oracle | byte-identical |
| the two unit tests that falsify I4 | confirmed present, `trace i` and `trace r` |
| all three programs reconstructed from the case file vs their probes | identical |
| `cargo fmt --all --check` / `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 / exit 0 |
| `memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, **1509 passed, 0 failed** |
| `REXX_CORPUS_GATE=1` corpus gate | **53 of 53 matching** |
| `git diff 613983c0b -- run.rs`, non-comment lines | empty |

---

# Final-review fixes, round 2

**Commit:** `1f4176b47` (base `7a0b31245`). One case file's prose. `run.rs`
untouched since `7a0b31245`.

All three re-measured here. All three were real.

## NEW-1 -- the evidence pointer pointed the wrong way

"which is what the transcript **below** shows, `G ran 5` ahead of `body`". The
transcript containing `body` is *above* the sentence; the one below is the
empty-body variant, which has no `body` line and so cannot show what the
sentence claims. One word.

Worth noting what caused it: the sentence and its evidence were adjacent when
written, and the empty-body probe was inserted between them in a later round.
**Same mechanism as NEW-1's predecessor** -- an edit that invalidates a
sentence without touching it -- but on the *pointer* rather than the subject.
A directional word like "below" is a positional claim, and positions move.

## NEW-2 -- "always" was false, on a program measured here

Re-measured, both engines:

```
oracle       G ran 5 / K ran 6 / after 5
this crate   G ran 5 / after 5 / K ran 7
```

`K ran 6` names the `END`, not the `DO`. So the oracle delivers **the
condition that was pending when the `DO` ran** at the `DO`'s own boundary, and
a *second* condition requeued by that delivery at the `END`'s. The claim is
bounded to the first rather than dropped, because the contrast with this crate
is the entry's whole point, and the second condition is now recorded as what it
is: a different mechanism and a second defect -- **genuinely absent where this
entry's is merely late** -- with a pointer to Task 6's Step 1c, which names its
program.

## NEW-3 -- the withdrawal contradicted the step this branch hands to Task 6

The paragraph withdrew the elided-instruction attribution because "nothing
anyone can run supports it", while Step 1c names a program demonstrating
exactly that mechanism. Both statements were about different things and the
paragraph did not say so.

Now split explicitly: **nothing reproduces this bullet's own observation, which
is why its attribution is withdrawn; the mechanism itself is real and is
demonstrated on a different construct with a named program.** The distinction
matters concretely -- Task 6 has just been told to decide between two
mechanisms, and the file was telling it one of them was unsupported.

**Also removed while in that paragraph**: the count of reconstructions. It said
"three readers, three results", the reviewer's made four, and correcting it to
four would have been the same defect with a bigger number. It now says a
different answer every time, which cannot rot.

## Two defects I introduced in this round and caught before committing

* The paragraph I rewrote for NEW-3 kept a trailing clause reading "inventing a
  fourth reconstruction and recording it as the third" -- **ordinals I had just
  removed from the sentence three lines above**, in the same edit.
* That same line was left unwrapped at 121 columns.

Both fixed. Checked mechanically afterwards: no line this branch added to the
file exceeds 80 columns, and the 11 that do are all pre-existing.

## The optional item: declined, with the reason

The reviewer suggests restoring "`trace_oracle.rs` carries no witness for this
shape", which it verified true, because it answers why the transcript is baked
into a unit test rather than captured as a trace-oracle case.

**Declined.** It is the same rot-shape as the two halves removed beside it -- a
claim about another file's contents, in a comment nothing re-reads when that
file changes -- and the house rule's disposition for it is explicit: assert it
in a test if it is load-bearing, delete it if it is not. Nothing breaks if it
becomes false, so it is not load-bearing. Restoring it because it is true today
is the reasoning that put the other two there.

The question it answers is real, and the honest position is that **I do not
know the placement's actual reason** -- I did not write that test, and
inventing a justification that reads well would be worse than leaving the
question open. Recorded here so the choice is visible rather than silent.

## What was re-run, with its output

| check | result |
| --- | --- |
| the double-requeue program, oracle vs both engines | oracle `G ran 5` / `K ran 6` / `after 5`; crate `G ran 5` / `after 5` / `K ran 7` |
| which transcript contains `body` | the one *above* the sentence; the one below is the empty-body variant |
| line lengths of everything this branch added to the file | all within 80; the 11 over are pre-existing |
| all three programs reconstructed from the case file vs their probes | identical |
| `cargo test -p rexx-exec --test ir_dual` | `9 passed; 0 failed` |
| `cargo fmt --all --check` / `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 / exit 0 |
| `memcap 8G cargo test --release --workspace --no-fail-fast` | exit 0, **1509 passed, 0 failed** |
| `REXX_CORPUS_GATE=1` corpus gate | **53 of 53 matching** |
| `git diff 7a0b31245 -- run.rs` | empty |
