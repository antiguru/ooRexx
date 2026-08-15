# Re-review of Task 7 fix round 2

Scope: commit `26da3ac6` against its parent `431e2698`. Did fix round 2
close the five findings of `task-7-rereview.md`, and did it introduce
anything new. **Not** a re-review of Task 7 as a whole; the earlier rounds'
settled verdicts are taken as settled.

Working tree clean at start and at end. Every tracked-file mutation below
was applied, measured, and reverted with
`git checkout -- rust/crates/rexx-exec/src/run.rs`; `git status --porcelain`
was empty after each restore and is empty now.

Every claim is labelled **RAN** or **REASONED**.

## How things were established

* `<scratchpad>/rr7b` -- a fresh probe directory, `mkdir`ed for this
  re-review, holding only my own files. Oracle wrapper
  `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx
  FILE )`, stdout/stderr/exit status as three separate descriptors,
  absolute paths for every redirect, `cd` into the probe directory before
  each run. 88 probes.
* `<scratchpad>/wt-r1` -- a detached `git worktree` at `431e2698` with its
  own `rexx-run`, so "is this new?" is a measurement and not an inference.
  Used on 31 probes.
* `<scratchpad>/stress2` -- a throwaway crate with a path dependency on
  `rexx-exec`, calling the `#[doc(hidden)]`
  `run_program_collect_every_alloc` and `run_program` on the same file and
  comparing all three descriptors. This is how the moved rooting was
  answered by running, **with a negative control**.

**Every probe result below was re-run at the end from binaries forcibly
rebuilt out of the restored source** (`touch` + `cargo build --release`,
both the repository's `rexx-run` and the stress crate), because some
probes were first taken while a mutation was in the tree and a stale
release binary reads exactly like a fresh one. The re-run reproduced every
verdict unchanged: **88 oracle comparisons, 79 MATCH and 9 DIFF** (`ac5`,
`e1`, `e2`, `e4`, `e5`, `g1`, `g4`, `t1`, `t3`), plus the 8 stress
comparisons, all `SAME`.

Gates, re-established here rather than taken from the report -- RAN, each
exit status read unpiped: `cargo fmt --all --check` **0**;
`cargo clippy --workspace --all-targets -- -D warnings` **0**;
`cargo test --workspace` **exit 0, 968 passed / 0 failed**;
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec` **exit 0, `38 of 38
matching`, mode STRICT, 4224 of 4259 assertion rows**;
`cargo test -p rexx-exec --lib` **289 passed**. The controller's baseline
reproduces exactly.

---

## Per-finding verdict

| finding | verdict |
|---|---|
| NEW 1 -- `active_condition = None` where the oracle restores | **closed** |
| NEW 2 -- the sub is not bounded at 1000 | **closed** |
| NEW 3 -- "majors 1 and 2 are the only ones" is false | **closed** |
| NEW 4 -- Rust integer parsing instead of `numberValue` | **closed** |
| NEW 5 -- "no path at all between the clause finishing and this check" | **partially closed** |

---

## NEW 1 -- closed

**Established: RAN**, three ways.

*By probe.* Four shapes, all **MATCH**:

| probe | shape | oracle = ours |
|---|---|---|
| `ac1` | `zq = sub() + 1/0` under both traps, `SIGNAL` handler ends in `raise propagate` | `UH ran 3` / `SH ran 3`, `Error 42.3`, rc **214** |
| `ac2` | the `SIGNAL` handler itself calls a routine raising the `CALL ON` condition, then propagates | rc 214 |
| `ac3` | `ac1` with the raising clause inside a `DO` body (NEW 1 x NEW 5) | rc 214 |
| `ac4` | the adjacent success: nothing active, restore `None`, `raise propagate` | `98.918`, rc **158** |

`ac1`, `ac2` and `ac3` are all **DIFF** on the `431e2698` worktree; `ac4`
is MATCH on both, which is the point of having it.

*By mutation.* RAN, each applied to the tracked source,
`cargo test -p rexx-exec --lib`, then restored:

| mutation | result |
|---|---|
| M7: the `Returned` arm clears (`= None`) instead of restoring -- round 1 verbatim | 288/1 -- kills `a_call_handler_restores_the_condition_it_interrupted`, alone |
| M8: the `Returned` arm never clears -- round 0 verbatim | 286/3 -- kills that test plus `a_returned_call_handler_leaves_no_active_condition_to_propagate` and `restoring_none_is_still_the_common_case` |

M7 surviving on the two "nothing was active" tests and M8 killing them is
exactly the discrimination the restore needs: the pair separates "restore"
from "clear" and from "never clear".

*The `Err` arm, which the report states is unmeasured.* I constructed the
shape and it is **reachable**, which the report does not claim either way:
with the arm replaced by `panic!("ERR-ARM-REACHED")` (RAN), four probes
panic -- `e1`/`e2` (a `CALL ON USER` handler that divides by zero under an
outer `SIGNAL ON SYNTAX`, at the `run_activation` and the `run_bounded`
boundary respectively) and `e4`/`e5` (the same with no outer trap). With
the arm panicking, **all 289 lib tests still pass**, so no test reaches it.

With the `self.active_condition = enclosing;` line simply deleted from that
arm (RAN), 289 tests pass *and* all nine of my `e*`/`ac*` probes give
byte-identical output. The restore there is unobservable in every shape I
could build, because the failure is immediately offered to a trap and
`offer_to_trap` overwrites `active_condition` on its way out. The comment
at the code says it is unmeasured and why; that is accurate, and now the
reachability is measured too.

---

## NEW 2 and NEW 4 -- closed

**Established: RAN**, two ways.

*By probe.* A 45-argument sweep of `raise syntax <arg>` -- negative subs,
leading zeros, leading/trailing/interior whitespace, `+`, upper- and
lower-case exponents, negative exponents, exponent overflow, empty string,
bare `.`, `.5`, `40.`, two decimal points, 21- and 23-digit halves, and
both bound boundaries -- is **45 of 45 MATCH**. The answers span all four
outcome classes the code has (18x `33.904`, 9x a catalogue entry at
rc 216, 12x `98.941`, and `'1e1'` reaching major 10), so the sweep
discriminates rather than agreeing on one answer.

**13 of the 45 are DIFF on `431e2698`**: `' 40 '`, `'40.1e1'`, `'40.1 '`,
`'1e1'`, `'40.-0'`, `'40.4 '`, `'40.1000'`, `'4E1'`, `'40.1E2'`, `'40.'`,
`'99.1000'`, `'40.4E0'`, `'40.4E1'`. So the fix generalises well past the
rows the report names -- whitespace, `E` notation and the `1000` boundary
at a second major are all closed by it and none was probed in the report.

*By mutation.* RAN:

| mutation | result |
|---|---|
| M4: drop `\|\| !(0..=999).contains(&sub)` | 288/1 -- `raise_syntax_validates_its_argument` |
| M5: `Number::parse(text)?.whole_value(…)` -> `text.parse::<i64>().ok()` | 288/1 -- same test |
| M6: `Some((_, ""))` rejected -> read as sub 0 | 288/1 -- same test |

Three separate clauses of the rule, three separate mutations, one test
that dies to each. The `u32` comment that asserted the opposite for
`40.99999` by name is gone from the source entirely (RAN, grep).

---

## NEW 3 -- closed

**Established: RAN**, by counting rather than by reading the report.
Parsing the generated catalogue
(`target/release/build/rexx-inventory-*/out/errors.rs`, the file
`rexx_inventory::errors::lookup` is `include!`d from) and asking which
majors in `1..=99` have no `(major, 0)` row gives **45**, and exactly the
list the comment now names: `1, 2, 12, 32, 50..=87, 94, 95, 96`. All 45
have no row at *any* sub, which is the property the `lookup(major, 0)`
branch is really testing.

Both comments now state 45; `grep` finds the old "majors 1 and 2 are the
only ones" wording nowhere in `run.rs`/`lib.rs` except inside the
correction itself. The test carries `50.1` and `87` so the row set cannot
be read as a two-element special case. The report's round-1 section still
carries its own softer version at line 3000 ("every other major in 1..=99
**that I probed** has an entry"), which is hedged and therefore true; the
round-2 section corrects the hard claim.

---

## NEW 5 -- partially closed

**The three named probes are closed, and eight more with them.**
RAN: `do2`, `sel1` and `int1` are **MATCH** here and **DIFF** on
`431e2698`, in timing and in `SIGL`. I added nine adjacent shapes through
`run_bounded`, of which **eight were DIFF on round 1 and are MATCH now**:

| probe | shape |
|---|---|
| `rb1` | `LEAVE` in the same body as the queuing clause |
| `rb2` | `ITERATE` in the same body |
| `rb3` | `return bb() \|\| 'TAIL'` inside a `DO` body in an internal routine |
| `rb4` | the handler `EXIT`s from inside a `DO` body (rc 7) |
| `rb10` | `DO WHILE` body |
| `rb11` | `IF … THEN DO` body |
| `rb12` | `OTHERWISE` body |
| `rb13` | `SIGNAL` out of the body in the same pass |
| `rb7` | (already MATCH on round 1 -- delivery into a *caller's* activation) |

`g2` (`WHEN`'s condition queues) and `g3` (`IF`'s condition queues) also
went DIFF -> MATCH.

**The two-caller claim is true as stated.** RAN, independently:
`grep -rn "step_in_temps_frame(" crates/ --include=*.rs` returns exactly
three lines -- the definition at `run.rs:3421` and the two call sites at
`run.rs:677` (`run_activation`'s loop) and `run.rs:3894` (`run_bounded`'s
loop). `fn step` has one non-doc caller, `step_in_temps_frame` itself
(`run.rs:3494`). `deliver_pending_trap` has one caller,
`clause_boundary`. So the rule really does live in one function reached
from two places.

**But "the two places that step a clause" is not the same as "every
clause", and the difference is measurable.** See NEW-A below: a `DO`
header is a clause, its boundary is inside `run_loop` rather than at
either `step_in_temps_frame` return, and the oracle delivers there. The
behaviour that NEW 5 asked about is better in nine shapes and still wrong
in two, and the three comments that assert the general property assert
something that is still false.

---

# New findings

## NEW-A (IMPORTANT, RAN) -- "exactly two places that step a clause" misses the `DO` header and the per-iteration condition, and three comments assert the general property anyway

`run_loop`/`run_repeating` resolve every iteration inside *one*
`step_in_temps_frame` call (`step_in_temps_frame`'s own doc comment says
so: "this call site fires exactly once, when the `DO`/`LOOP` instruction
is first stepped"). So the `DO` clause's own boundary -- which the oracle
re-executes and re-echoes once per pass -- is reached by neither caller of
`clause_boundary`.

| probe | oracle | `26da3ac6` | `431e2698` |
|---|---|---|---|
| `g1` -- `do i = 1 to sub()`, `sub` queues a trapped `USER` condition | `body 1 mark= HANDLER-AT 3` / `body 2 … 3` / `end … 3` | `body 1 mark= NOMARK` / `body 2 … HANDLER-AT 4` / `end … 4` | `NOMARK` / `NOMARK` / `HANDLER-AT 4` |
| `g4` -- `do while zn < 2 & sub() = 'SV'` | `body 1 … HANDLER-AT 4` / `body 2 … HANDLER-AT 7` / `end … 7` | `body 1 … HANDLER-AT 5` / `body 2 … 5` / `end … 6` | `NOMARK` / `NOMARK` / `HANDLER-AT 6` |

Both are wrong in timing (the oracle runs the handler *before* the first
body clause, ours after it) and in `SIGL` (the oracle attributes it to the
`DO`/`DO WHILE` clause, ours to a body clause). Neither is a regression --
round 1 was worse -- but both falsify the property the round asserts:

* `lib.rs`, `pending_trap`'s doc: "**"Every clause" means every clause,
  and that took a third go** … called from both -- and only both -- of the
  places that step a clause"; and the round-1 sentence it is stacked on,
  still present: "runs once per *clause*, on every path out of one".
* `run.rs:2489`, `clause_boundary`'s doc: "**The whole delivery rule, in
  one function, called from every place that steps a clause** -- and there
  are exactly two, which is what makes that checkable rather than
  aspirational".
* `run.rs:695`, the inline comment: "`clause_boundary` is the rule itself
  … see its own doc comment for why both callers are needed and why there
  are exactly two."

The grep those sentences rest on is true; the inference from it is not.
"The only callers of `step_in_temps_frame`" is a fact about that function,
not about Rexx clauses, and a `DO`/`LOOP` header is a Rexx clause that
`step_in_temps_frame` visits once for an arbitrary number of clause
boundaries. This is the same shape as NEW 5 -- a checkable-sounding
mechanism sentence that a probe falsifies -- reintroduced by the fix for
NEW 5, and `rust/CLAUDE.md` requires it corrected rather than hedged.

Either bound the claim to what holds (the check runs at every boundary
*of a stepped instruction*, and `DO`'s own header and `END` are not among
them), or take `g1`/`g4` as behaviour work for a task.

## NEW-B (MINOR, RAN) -- a failing `CALL ON` handler delivered inside a `DO`/`WHEN`/`INTERPRET` body now reports the enclosing `DO` clause as its second echo line

An error escaping `clause_boundary` in `run_bounded` is returned past the
inner clause's own `step_in_temps_frame` (which already returned `Ok`) and
is first seen as an error by the `step_in_temps_frame` call that is
stepping the *enclosing* `DO`, whose `record_failure_site` then claims it.

`e5` -- `call on user foo name uh`, `call sub` inside `do i = 1 to 1`,
handler `uh` does `zz = 1/0`:

| | stderr |
|---|---|
| oracle | `11 *-* zz = 1/0` / `3 *-* call sub` |
| `26da3ac6` | `11 *-* zz = 1/0` / **`2 *-* do i = 1 to 1`** |
| `431e2698` | `11 *-* zz = 1/0` (no second line) |

`e2` is the same with an outer `SIGNAL ON SYNTAX` and splits identically.
Round 2 fixed this shape's stdout (`UH ran 3`, no spurious `after call`)
and made its stderr wrong in a new way: round 1 omitted a line, round 2
prints one naming a clause that did not fail.

The control says the machinery is fine for everything else: `f1` -- an
ordinary `zz = 1/0` inside a routine called from a `DO` body, no traps at
all -- is **MATCH**, second echo line included. So this is specific to a
failure raised at the clause boundary rather than inside the clause.

## NEW-C (MINOR, RAN) -- `deliver_pending_trap`'s doc comment was left attached to the new `clause_boundary`, and `deliver_pending_trap` now has none

`run.rs:2470-2508` is one contiguous `///` block, and the `fn` it
documents is `clause_boundary` at `:2509`. Its first three paragraphs are
`deliver_pending_trap`'s, unchanged: "Runs a `CALL ON` trap's handler …",
"**The wait is the measured part.** `zres = one(1)` …", "The trap is
removed for the handler's duration and **put back afterwards** …". None of
that is `clause_boundary`'s behaviour -- it neither waits nor touches
`activation.traps`. `deliver_pending_trap` at `:2519` is left with no doc
comment at all, so the transcripts that justify its trap-removal
discipline no longer sit on the function that implements it.

There is also no blank `///` between `:2485` and `:2486`, so
"…running the handler a second time later." and "One Rexx clause has just
finished: …" render as one paragraph -- two summary sentences for two
different functions fused into one.

## NEW-D (MINOR, REASONED) -- `run_bounded`'s own doc comment is now false about what it can return

`run.rs:3876-3879` still says:

> Reaching `end` exactly … is the only way this returns `Ok(Flow::Next)`.
> **Every other exit returns the escaping `Flow` unchanged**, and the
> caller (`If`/`Select`) must check which happened rather than assume the
> former.

The new arm at `:3904-3913` returns `Ok(Flow::Exit(value))` synthesised
from the handler's `Ended::Exited` -- a `Flow` no stepped instruction
produced and which is not the escaping one (the stepped clause's `flow` is
typically `Flow::Next`). The function's own doc was not updated with the
arm. The related paragraph two above it ("That covers `Flow::Exit` today
…") reads as though `Flow::Exit` only ever arrives from below, which is
now also incomplete.

## NEW-E (MINOR, RAN) -- `restoring_none_is_still_the_common_case` is a strictly weaker duplicate of an existing test

The new test runs
`call on user foo name uh / call sub / raise propagate / … / uh: say 'UH
ran'; return` and asserts `(98, 918)`. The round-1 test
`a_returned_call_handler_leaves_no_active_condition_to_propagate` runs the
same program with one extra `say 'resumed'` clause and asserts `(98, 918)`
**and** the stdout. No mutation in my battery kills one without the other
(M7 leaves both green, M8 kills both), so the new test adds no coverage
and asserts less than the one it sits beside. Its own doc comment says as
much ("Round 1's own test for this still passes unchanged … this comment
exists so the pair is findable from either side"), which is honest, but
the report presents it as part of what stops the restore being a disguised
"never clear" -- and that job was already done.

---

## What I checked and found correct

Recorded because a clean result is part of the answer.

* **The moved rooting is still load-bearing, at both callers.** RAN, with
  a negative control. `sr1`/`sr2` put a heap value on the `Flow::Return`
  and `Flow::Exit` paths at `run_activation`'s boundary; `sr3`/`sr4` do
  the same at `run_bounded`'s (`return bb() || 'TAIL'` and
  `exit sub() || '2'` inside a `DO` body). All four MATCH the oracle and
  all four survive `run_program_collect_every_alloc` with real collections
  (129-134). With the two-line `push_temp` deleted from `clause_boundary`,
  **all four panic** on `a live value` -- `sr1`/`sr3` at `value.rs:125:47`
  and `sr2`/`sr4` at `value.rs:221:55`. Restored afterwards. So the move
  did not weaken it and it now covers the second caller for real.
* **The handler's own `EXIT` value survives its new, longer trip.**
  RAN. `Ended::Exited` from a handler delivered inside `run_bounded` is
  now repackaged as `Flow::Exit` and travels out through the enclosing
  instruction's `pop_frame` before anything reads it -- a window that did
  not exist in round 1. `sr6` (`DO` body), `sr7` (`WHEN` body) and `sr8`
  (`INTERPRET` fragment) each exit with a concatenated two-digit value:
  all three MATCH the oracle (rc 12/13/14) and all three give identical
  output under collect-on-every-allocation with 131-133 collections.
* **The blast radius of the `run_bounded` change is bounded by one line.**
  RAN (grep) and REASONED. `clause_boundary`'s first statement is
  `if self.pending_trap.is_none() { return Ok(None); }`, and
  `pending_trap` has exactly one writer in the crate (`run.rs:2841`, in
  `exec_raise`). A program that never queues a `CALL ON` condition
  therefore takes a bit-identical path through `run_bounded`.
* **`LEAVE`/`ITERATE`/`RETURN`/`SIGNAL`/loop control are unaffected.**
  RAN, the nine-probe battery above, plus round 1's own `pa2`, `pf`,
  `lv1` and `ps` re-run here: all MATCH.
* **Trace output over the changed path.** RAN. `t2` (`trace a` across a
  `WHEN` body with a delivered handler) is MATCH. `t1` (`trace r` across a
  two-pass `DO` with a delivered handler) DIFFs on two `>>>` intermediates
  per iteration -- and `t3`, the same loop with **no condition trap
  anywhere**, DIFFs identically, so that is a pre-existing `trace r` gap
  on `DO` re-iteration and not this change.
* **The `unreachable!` in `run_bounded` is currently sound.** REASONED:
  `deliver_pending_trap`'s only `Ok(Some(…))` is
  `Ok(Some(Ended::Exited(value)))`, and `clause_boundary` forwards it
  unchanged. It is a cross-function invariant enforced by a panic rather
  than by a type, which is the kind this project has been bitten by, but
  it holds today.
* **All four new tests can fail**, each to a targeted mutation (M2 kills
  the two `run_bounded` tests, M7/M8 the two restore tests). None is a
  test that cannot fail; NEW-E is about redundancy, not vacuity.
* **Gates reproduce exactly**: fmt 0, clippy 0, `cargo test --workspace`
  exit 0 / 968 passed / 0 failed, corpus gate exit 0 / `38 of 38` /
  STRICT / 4224 of 4259, `-p rexx-exec --lib` 289 passed.

## Pre-existing, found in passing, not this round's

Recorded so they are not re-discovered as regressions.

* **`SIGL` is clobbered when a `CALL ON` handler makes its own call.**
  RAN. `ac5` -- `zq = sub() + 1/0` under both traps, where the `USER`
  handler contains `call inner` -- gives the oracle `SH ran 3` and both
  `26da3ac6` and `431e2698` `SH ran 9`. Our `offer_to_trap` sets `SIGL`
  before `clause_boundary` runs the pending call handler, and the
  handler's own `CALL` overwrites it. Byte-identical on round 1, so not
  introduced here.
* **`trace r` omits the `>>>` intermediates on a `DO`'s re-iteration**
  (`t3`, no traps involved).
