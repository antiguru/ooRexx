# Task 16 review: `REPLY` and `GUARD` inside a method

Reviewed at `7eabdfb59` against `.superpowers/sdd/2026-08-17-phase-5a/task-16-brief.md`, the
task report, and the three-commit diff `b1016203c..7eabdfb59`.

**Spec compliance: PASS.**

**Task quality: CHANGES REQUESTED.**

## Spec compliance

Re-derived myself rather than taken from the report. Both brief transcripts, run from a fresh
empty directory with absolute paths, three descriptors compared separately, both engines, both
sides bounded:

```
method_guard_instruction  ir           AGREE rc=0
method_guard_instruction  tree-walker  AGREE rc=0
   oracle rc=0 stdout=[guarded|] stderr=[]
method_reply              ir           AGREE rc=0
method_reply              tree-walker  AGREE rc=0
   oracle rc=0 stdout=[replied|after|]
          stderr=[    15 *-* return 'returned'|Error 98 running .../method_reply.rex line 15:
                   Execution error.|Error 98.936:  RETURN cannot return a value after a REPLY.|]
```

`AGREE` is exit status, stdout and stderr each byte-identical, never `2>&1`. The
exit-status-0-with-a-traceback shape is reproduced as a pair, not as a status.

The control the brief requires reddens. Deleting the 98.936/98.937 raise from
`Interp::returned_value` and rebuilding leaves exit status and stdout untouched
(`method_reply` rc 0, stdout `replied`/`after`; `method_reply_exit_status` rc 7, stdout
`v`/`tail`) and empties stderr on both engines; the gated corpus goes to **183 of 185**,
naming `lang/method_reply.rex` and `lang/method_reply_exit_status.rex`. Restored afterwards.

`phase-4-exclusions.txt:3304`'s `GUARD` limb is retired as the brief requires.

## Findings

### 1. No `oracle-crashes.txt` entry for the hanging program (must-change, test-instrument)

**Confirmed, and it is the most severe thing here.** The hang is real. Measured twice under the
standard oracle wrapper with a 10-second kill: rc 137, nothing on stdout, nothing on stderr. A
third bounded run under `/usr/bin/time -v` reports **0.00 s user and 0.00 s system CPU over 5.00 s
elapsed**, so the oracle is blocked on a wait rather than spinning, which is what makes "for ever"
the right reading rather than "slow". The crate refuses it loudly on both engines, rc 120, naming
Phase 6.

`rust/corpus/oracle-crashes.txt` carries no entry. Its only `guard` occurrence is unrelated text in
the `SELECT` entry. The file's whole stated purpose is "so nobody has to rediscover them by
running them", and a wait with no end is the most expensive kind to rediscover.

Two things the entry needs beyond the program, the effect and the cause:

* **The header's taxonomy has to widen.** It currently says the failure modes "are not folded into
  one label: an entry is either a deterministic memory-safety defect in the C++ (SIGSEGV, rc 139)
  or a resource-exhaustion hazard (unbounded allocation, OOM-killed)". A blocking wait at zero CPU
  is neither. Leaving the entry under one of the two existing labels would make the header false.
* **The convention this task's own `Loud` skips.** Task 15's `Loud::array_index_hole` is the exact
  precedent: an oracle behaviour that cannot be captured, so a refusal instead of an answer. Its
  doc names both halves of the record -- "the program is in `corpus/oracle-crashes.txt` and must
  not be run" and "`dispatch.rs`'s `an_expanded_index_of_one_empty_slot_is_loud` is the
  instrument". `Loud::guard_when_false` names neither. The instrument exists and is live (proved
  below), but only the report says which it is, and the report is not in the repository.

### 2. The `dispatchclass` ir contribution is +0.42%, not +0.22% (must-change, prose)

**Confirmed and settled from the TSV, keeping `instrument` and `size` in every projection.** The
arithmetic gap is real and it is bigger on the large cell than on the small one.

Accumulated `pinned>head`, `instructions:u`:

| axis | arm | size | Task 15-fixround-2 | Task 16 | delta |
| --- | --- | --- | --- | --- | --- |
| dispatchclass | tw | large | 1.012597 | 1.015399 | +0.277% |
| dispatchclass | ir | large | 1.012279 | 1.016525 | +0.420% |
| dispatchclass | ir | small | 1.012257 | 1.016492 | +0.418% |

The tw arm reconciles exactly with the report's +0.28%. The ir arm does not: the report claims
+0.22% and a *fall* on the large cell (0.999541).

The cause is a contaminated median in one arm of one axis of the contribution sitting. Projecting
`median` against `min` for every absolute cell of `16-contribution`, every cell agrees to four
decimal places **except** `dispatchclass`/`base`/`ir`:

```
dispatchclass  base  ir  small  med=12916149038  min=12890142050  spread=0.2018%
dispatchclass  base  ir  large  med=25850879966  min=25730852836  spread=0.4665%
```

So at least three of five rounds of the base arm landed in a high mode. The report's own
"about 34 instructions per pass" is arithmetically right about the quantum -- the large cell's
mode gap is 120,027,130 instructions over ~3.53 M passes -- but it reads the wrong end of the
band. Recomputing from the uncontaminated `min`, which agrees with Task 15's independently
measured `base` median (25,731,033,256):

* ir small: 12,944,164,395 / 12,890,142,050 = **1.004191**
* ir large: 25,838,965,683 / 25,730,852,836 = **1.004202**

Both equal the accumulated delta to five decimal places, and the accumulated figure is tight
(`pinned>head` ir large min..max = 1.016521..1.016528). Nothing is unattributed. The **max** of
the report's own contribution band, `1.004202`, was the honest number and the median was the
artifact.

Two consequences:

* The reconciliation sentence is false on the ir arm. `1.012279 x 1.002169 = 1.014453`, against
  the 1.016525 read. On tw it is true: `1.012597 x 1.002767 = 1.015400`.
* The gate conclusion survives. **+0.42% is still under the 1% line**, so "no axis is over the 1%
  line on my own contribution" holds; only the disclosed magnitude is wrong.

`cycles:u` was not read as a result, correctly.

### 3. Three false universal claims about checks (must-change, prose)

Each is the shape where a negative claim is wider than the run behind it.

**(a) "`corpus_differential` was the *only* red test in the whole workspace."** Measured at HEAD
under the same 98.936 mutation: `cargo test --release -p rexx-exec --lib --no-fail-fast` gives
`run::tests::a_value_returned_after_a_reply_reports_the_oracles_own_98_936 ... FAILED`. So the
mutation reddens at least two things. The check was run on a tree that did not yet contain the
in-crate tests the same report goes on to add -- correct when run, false about what shipped.

**(b) "Deleting either raise reddens the gated corpus at 183 of 185."** Measured with the 98.935
raise deleted instead: **184 of 185**, one mismatch, `lang/method_reply_twice.rex`. 183 is the
98.936 figure only. The report gets this right in the body ("`method_reply_twice.rex` stayed
green, correctly") and wrong in the instrument table.

**(c) "The tw arm moves further than the ir arm on every axis, `dispatchclass` included."** False
on three of eight by the report's own table and corrected figures: `emptyloop` is equal (1.000000
both arms), `rexxcps` moves in opposite directions (tw 0.999679 down, ir 1.000193 up), and
`dispatchclass` inverts once finding 2 is applied -- ir +0.42% against tw +0.28%. The sentence
that follows it ("I have not attributed that to anything and would need a layout control to") is
the right instinct attached to a claim that does not hold.

### 4. "An order no single-threaded scheduling produces" overreaches, in the code (must-change, prose)

**The load-bearing claim holds and is better supported than the report says. The stronger claim
does not.** I re-ran the measurement myself, 30 runs per shape, bounded on both sides.

`say .K~m` / `say 'main-end'` over a replying method: **22 of 30** one way, 8 the other. Same in
kind as the reported 12 of 15 versus 3 of 15; the exact ratio is sampling noise. Racy, confirmed.

Two objects each replying: **five** distinct stdout orders in 30 runs.

```
19  A-replied|A-after|B-after|B-replied|main-end
 8  A-replied|B-after|A-after|B-replied|main-end
 1  A-replied|B-after|B-replied|main-end|A-after
 1  A-replied|B-after|B-replied|A-after|main-end
 1  A-replied|A-after|B-replied|main-end|B-after
```

Two problems with how this is written down, and the comment matters more than the report because
`Interp::deferred`'s doc carries the same sentence:

* The order quoted as what the shape "printed" is row 3, seen **1 time in 30**. The modal order,
  19 of 30, is different. A doc whose subject is non-determinism presents an outlier as the
  measurement.
* "An order no single-threaded scheduling produces" is not what the evidence supports. A
  single-threaded scheduler that yields at the `REPLY` and returns to the sender afterwards
  produces exactly that interleaving; what no scheduler produces is *all five* orders, and what
  this crate's defer-to-the-end model produces is none of the five. The defensible sentence is
  that the oracle has no single answer here, which is all the corpus-exclusion decision needs.

The exclusion decision itself is right, and I found a third racy shape supporting it: a
`SIGNAL ON SYNTAX` trap firing inside an owed body splits **15/15** on the oracle
(`replied|trapped SYNTAX|main-end` against `replied|main-end|trapped SYNTAX`), and this crate
answers with one of the two. Also clean and not in the report: a main body that *raises* after the
send (`say .K~m` then `say 1/0`) agrees on all three descriptors, rc 214, both engines.

### 5. The Phase 6 boundary marking is claimed complete and is not (must-change, prose plus a small code gap)

The brief requires the code, not only the report, to say which parts are legality and which are
scheduling. The report opens its list with "For Phase 6, marked in the code as well as here."

Every marker in the diff, exhaustively:

```
dispatch.rs  park_reply             **SCHEDULING.**
lib.rs       Interp::deferred       **SCHEDULING, and Phase 6 owns the whole of it.**
run.rs       exec_guard             **SCHEDULING, and Phase 6 owns it.**
run.rs       exec_reply             **LEGALITY, in the order the C++ takes it.**
run.rs       exec_reply             **SCHEDULING, and Phase 6 owns it: ...**
lib.rs       Loud::guard_when_false        message ends "(Phase 6)"
lib.rs       Loud::reply_inside_construct  message ends "(Phase 6)"
```

Listed in the report and unmarked in the code: `returned_value`'s 98.936/98.937 check (legality,
Phase 6 must keep it), `raised_guard_not_logical` (legality), `ReplyState` itself (legality), and
the entire `RootSet::park`/`release`/`live_parked`/`frame_aliases` plus
`Activation::object_roots`/`CallContext::object_roots` group (scheduling, so Phase 6 may delete
it -- which is precisely the thing a Phase 6 implementer reading `roots.rs` would not learn).

And one outright false sentence: "`Interp::deferred`, `Interp::park_reply`, `Interp::resume_reply`,
`Interp::run_deferred_replies` ... Its own doc comments say SCHEDULING." Two of the four do.
`resume_reply` and `run_deferred_replies` do not.

The substance is marked at the two instruction sites, which is why this is a small gap rather than
a spec failure; the claim about it is the defect.

### 6. `Activation::reply`'s doc says "Three readers" and there are four (must-change, prose)

The doc enumerates `REPLY` itself, `RETURN`/`EXIT`, and `Interp::enter_method_body`. The read
sites are:

```
dispatch.rs:1497  if callee.reply == ReplyState::Owed          (enter_method_body)
dispatch.rs:1690  debug_assert_ne!(callee.reply, ...)          (resume_reply)
run.rs:3617       if self.activation().reply != ReplyState::None   (exec_reply)
run.rs:3691       if value.is_some() && self.activation().reply != ...  (returned_value)
```

The fourth reader is the `debug_assert_ne!` this task added in its own self-review round 5. The
count was stale before the commit landed. This is the third consecutive task on which a fix round
has left an enumerated-set comment behind rather than removed it -- name the readers or assert the
count, do not write the number.

### 7. A test doc that contradicts itself, and a misfiled instrument (park or fix cheaply, prose)

`the_guard_instructions_answers_are_the_oracles_own` opens "Each row is the oracle's own answer,
measured on `build/bin/rexx`" and four sentences later says "The last two rows are the loud
refusals". Two of its five rows are refusals the oracle does not produce -- one where the oracle
blocks for ever, one where it answers something else entirely. The test's own name carries the
same false claim.

Related and more consequential: row 5 is the **sole** instrument for
`Loud::reply_inside_construct`, and it lives inside a test named for `GUARD`. Nothing in the code
says so; only the report's instrument table does. A later task renaming or splitting the guard
test would silently drop the reply refusal's only coverage.

### 8. Two smaller prose defects (park)

* "`emptyloop` is byte-for-byte unchanged on both arms." The ratio is 1.000000; the absolute
  counts differ (tw small 5,450,613,897 base against 5,450,613,987 changed). "Byte-for-byte" is
  the wrong phrase for a rounded ratio, and it is the phrase the confinement argument leans on.
* `coverage.rs`'s new comment says "the four legality refusals, one fatal each". Under the
  grouping that makes the count four, `method_reply_twice.rex` is rc 0 -- and the same sentence
  two clauses earlier contrasts a program "whose whole point is exit status 0 with a traceback"
  against fatality, so "fatal" is being used in the exit-status sense there.

## Confirmed without change needed

**Finding 5 of the dispatch, the stale control.** The rewrite is true of the code as it now
stands, and it is not a relocation. Both re-measured claims verified by running them:

```
.K~start('m')   oracle rc=0 stdout=[a Message|]
                ir / tree-walker rc=120  method "START" of class "Object" is not implemented
guarded self~m  oracle rc=0 stdout=[level 2 level 1 bottom|]
                ir / tree-walker rc=0 stdout=[level 2 level 1 bottom|]
```

The old premise was "the routes to a second activity are loud"; two of those routes now run, and
the entry says so in its own text rather than quietly substituting. The new premise -- neither
`REPLY` nor the `GUARD` instruction creates an activity -- is a fact about this crate's code
(`Interp::deferred` runs the owed body on the same activity after the main program), not about
which constructs are refused, so it does not fail the same way. It hands its successor to Phase 6
explicitly. The entry keeps its "list of routes checked, not an enumeration of routes" warning,
which is the part that makes the whole limb honest.

**Finding 4 of the dispatch, the nested-`REPLY` reason.** Checked against the code, not accepted.
It is **true**, for both constructs it names:

* `LoopState` is a local of `Interp::run_repeating`, a Rust stack frame. A `Flow::Return` out of a
  `REPLY` inside a `DO` unwinds it, and `pc` carries nothing that could rebuild it.
* `If`'s true branch runs inside `run_bounded`, whose `resume` target is a local too. Resuming at
  `pc = index + 1` inside a then-branch would fall through to the `Else` marker, which is
  `Ok(Flow::Next)`, and run the else-branch as well -- a wrong answer, not a refusal.

Both new refusals name Phase 6 in their message text, so the owner reaches the user.

**Finding 6 of the dispatch, the control and the in-crate tests.** Control verified above.
The ungated gap is real: the same mutated build, ungated, reports `183 of 185 matching -- REPORT
MODE, NOT THE GATE`, the same two mismatches, and `test corpus_differential ... ok` at **exit 0**.
So a legality check keyed on stderr alone had no instrument in the ungated gates and the in-crate
tests are the right answer. Three liveness proofs re-run myself, each reddening exactly its own
test with the others green:

```
98.936 deleted   a_value_returned_after_a_reply_reports_the_oracles_own_98_936  FAILED
                 a_second_reply_reports_the_oracles_own_98_935                  ok
                 the_guard_instructions_answers_are_the_oracles_own             ok
98.935 deleted   a_second_reply_reports_the_oracles_own_98_935                  FAILED
                 (other two ok)
M1 guard WHEN-false loud -> always a no-op
                 the_guard_instructions_answers_are_the_oracles_own             FAILED
```

M1 is the one that matters most, because it is the refusal the corpus structurally cannot carry.

**Finding 7 of the dispatch, the four axes.** Exactly four axes read over 1% accumulated at Task
16 -- `rexxcps` 1.0188, `dispatchclass` 1.0165, `strings` 1.0134, `compound` 1.0105 -- and all
four are named. Contributions reconcile against the TSV, bar finding 2: `compound` tw +0.073% /
ir 0, `strings` tw +0.055% / ir 0, `rexxcps` tw 0.99967 / ir 1.00015. Attribution is right where
it is stated (`dispatchclass` crossed during Task 14; `rexxcps` has no owner because its axis
joined at Task 15).

**The attribution the report left open: Task 14 owns `strings` and `compound` too.**

* `compound`/ir sat at 1.002618 from Task 11 through Task 13 and went to **1.010472 at Task 14**.
* `strings`/ir crossed once at Task 9 (1.010616), and Task 9's own fix round brought it back
  under to 1.009685, where it stayed through Task 13; it went to **1.013410 at Task 14** and has
  not moved since.
* `dispatchclass`/ir went 1.009267 to 1.011780 at Task 14, and `alloc4c`/ir went 1.002703 to
  1.004805 in the same task.

So Task 14 moved four ir axes up at once. Whether that is work or layout is not settled by these
numbers, and the layout-control lesson says an across-the-board single-task ir shift should not be
read as work without a do-nothing control.

**Counts re-derived.** 185 = 31 + 12 + 12 + 130 non-comment rows across `phase-4a.txt`,
`phase-4b.txt`, `phase-4c.txt`, `phase-5a.txt`; 177 before; 8 new `.rex` and 8 new
`sourceline_oracle` expectations added by the diff, and each expectation is byte-identical to its
program apart from the `count N` header (all eight checked, not sampled). "Six programs" for
`REPLY`'s corpus coverage is right.

**Binding constraints.** Zero non-ASCII bytes and zero occurrences of `unsafe` in the added Rust
lines. Both engines carry every construct; no `Op::Generic`. The release profile sets `debug`,
`lto` and `codegen-units` and **not** `debug-assertions`, so the report's claim about which gates
see the `debug_assert`s holds. `execute` is the only caller of `run_deferred_replies`, as
`Interp::deferred`'s doc says. The four types `object_roots` admits it cannot see hold no
`ObjRef` -- and `Settings` structurally cannot, since it lives in `rexx-num`, which does not
depend on `rexx-core`, which is stronger than the "holds none today" the report claims.

**The `INTERPRET` disclosure is accurate.** Measured: `interpret "reply 5"` inside a class method
is rc 157 with 99.924 on both sides, and the only difference is the traceback's missing first line
(`6 *-* reply 5`). Pre-existing, correctly characterised, correctly disclaimed as not this task's.

**The `Op::Return` self-review finding is real.** `returned_value` returned `Flow` before this
task, so it could not fail and nothing had exercised the `?` path. The `break 'cold Err(failure)`
fix is the right shape and the comment records the wrong answer it replaced.

## Tree state

I mutated `rust/crates/rexx-exec/src/run.rs` four times (98.936/98.937 raise deleted, 98.935 raise
deleted, guard `WHEN`-false loud replaced by a no-op) and restored it each time. Final state:
`md5sum` back to `84cacc864554c41f42f48ee09a6b89ec`, `git status --porcelain` empty, and the
release binary rebuilt from the restored source so no instrumented artifact outlives the revert.
All probes ran from a fresh empty scratch directory, never the scratchpad root, with both sides
bounded in time and memory and the three descriptors read separately.
