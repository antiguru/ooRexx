# Re-review of Task 7 fix round 4

Scope: commit `9a4b57be` against its parent `f9e6bf26`. Did fix round 4 close
the seven findings of `task-7-rereview3.md` (Important 1-4, the three Minors),
and did it introduce anything new. Earlier rounds are settled and are not
re-litigated.

Working tree clean at start and at end. Every tracked-file mutation below was
applied, built and/or tested, then reverted with
`git checkout -- <path>`; `git status --porcelain` was empty after each restore
and is empty now. Both the debug and the release `rexx-run` were rebuilt from
the restored source at the end.

Every claim is labelled **RAN** or **REASONED**.

## How things were established

* `<scratchpad>/rr4` -- a fresh probe directory `mkdir`ed for this re-review.
  It holds the implementer's own 63 probes (copied in, so they are my files),
  plus **30 probes of my own** (`w1`-`w30`) written for the shapes the risk
  brief named as uncovered. Oracle wrapper
  `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`,
  stdout/stderr/exit status as three separate descriptors, absolute paths for
  every redirect, every file listed explicitly rather than globbed from a
  shared directory.
* `<scratchpad>/wt-r3` -- a detached `git worktree` at `f9e6bf26` with its own
  release `rexx-run`, so "regression or pre-existing" is a measurement.
* **Harness sanity, RAN, before any tally was trusted**: the script `exit 9`s
  if either binary is missing; the two binaries are different files
  (1 372 224 vs 1 369 320 bytes, built at 00:55 and 00:40) and **disagree on 35
  of the 93 probes**, which is what proves both are live. The release binary
  was rebuilt from restored source after the last mutation and the whole sweep
  was re-run against it; the tally was identical.
* Fifteen source mutations (B1-B4, C1, C3, C3b, B2, B2b, M1, M2, M3, M5, M6,
  M7, M-U), each built and/or tested and reverted.

Gates, re-established here rather than taken from the report -- RAN, each exit
status read unpiped: `cargo fmt --all --check` **0**;
`cargo clippy --workspace --all-targets -- -D warnings` **0**;
`cargo test --workspace` **exit 0, 976 passed / 0 failed** (summed from every
`test result:` line); `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`
**exit 0, mode STRICT, `38 of 38 matching`**; assertions **4224 of 4259**. The
controller's baseline reproduces exactly.

---

## Per-finding verdict

| finding | verdict |
|---|---|
| Important 1 -- `ITERATE` re-test attributed to `END`; `u7`/`u8` a round-3 regression | **closed** |
| Important 2 -- four clause sites with no boundary at all | **closed** |
| Important 3 -- the `#[must_use] ClauseToken` obligation was narrow (3 of 13 attacks caught) | **closed** |
| Important 4 -- `ClauseEnd::Completed(None)` drops the GC rooting | **closed** |
| Minor -- `lib.rs:947-966` names a deleted function | **closed** |
| Minor -- round 2's `unreachable!` invariant no longer announces itself | **closed** |
| Minor -- the `UNTIL` re-test boundary is unobservable and untested | **closed** (one false comment left behind, NEW-1 below) |

---

## The regression sweep

**93 probes, A/B against `f9e6bf26`: 35 DIFF -> MATCH, 54 MATCH on both, 4 DIFF
on both, 0 MATCH -> DIFF.** RAN.

The implementer's own 63 reproduce their reported tally exactly (17 / 43 / 3 /
0). My own 30 add **18 DIFF -> MATCH, 11 MATCH on both, 1 DIFF on both, 0
MATCH -> DIFF**.

The four DIFF-on-both are one pre-existing class, confirmed DIFF on `f9e6bf26`
too and disclosed as a KNOWN GAP at `run.rs:4741` with the C++ citation: under
`TRACE R` a **controlled** `DO`'s per-pass re-execution emits two `>>>`
control-variable lines this crate does not produce (`q11`, `y4`, `z6`, and my
`w29`). Nothing else diverges.

My 30 probes were chosen for exactly the shapes the risk brief said the A/B
might not cover. All of these are **new** measurements, not re-runs:

| probe | shape | on `f9e6bf26` | now |
|---|---|---|---|
| `w3` | nested `IF` inside an `IF`'s `THEN`, inner `then` on the next line | `HANDLER-AT 5` | MATCH (`4`) |
| `w7` | `ITERATE` from inside a `SELECT` inside a loop | `2, 7, 7` | MATCH (`2, 5, 5`) |
| `w9` | a handler that **`EXIT`s** delivered at an `IF` header boundary | `H 3` | MATCH (`H 2`) |
| `w10` | a handler that `EXIT`s delivered at a false `WHEN`'s boundary | `b`, `H 4` | MATCH (`H 3`, `OTHERWISE` never runs) |
| `w13` | `trace r`, labelled `ITERATE out` from an inner loop | DIFF | MATCH |
| `w15` | labelled `ITERATE lab` out of an inner loop, `UNTIL` outer | `7, 7` | MATCH (`5, 5`) |
| `w16` | `trace r`, `DO OVER` with an `ITERATE` | DIFF | MATCH |
| `w17` | the re-test that **stops** the loop, after an `ITERATE` | `HANDLER-AT 7` | MATCH (`6`) |
| `w18` | a handler that `EXIT`s delivered at a `SELECT CASE` boundary | DIFF | MATCH |
| `w19`/`w20` | the same at a `WHILE` and an `UNTIL` re-test after an `ITERATE` | DIFF | MATCH |
| `w21` | a `SELECT` nested inside a true `WHEN`'s body | DIFF | MATCH |
| `w22` | an `IF` header inside a controlled `DO` body, queueing on pass 2 | DIFF | MATCH |
| `w23` | `LEAVE s` out of an `OTHERWISE` body with a queued condition | DIFF | MATCH |
| `w25` | `SIGNAL` out of an `OTHERWISE` body with a queued condition | DIFF | MATCH |
| `w26` | `RETURN` from inside a `WHEN` body, value crossing the boundary | DIFF | MATCH |
| `w28` | `IF` -> `THEN DO` -> `IF` three deep inside a controlled loop | DIFF | MATCH |
| `w30` | labelled `ITERATE lab` in an `UNTIL` loop under a trap | `HANDLER-AT 7` | MATCH (`6`) |

The adjacent successes, MATCH on both and still MATCH -- these are what pin the
rule rather than the numbers: `w1` (zero-trip `WHILE` whose header queues),
`w2` (zero-trip controlled `DO i = 1 to sub()`), `w4` (a **false** `IF` with
`then` on the next line, already right before the fix), `w5` (`SELECT` with no
`OTHERWISE` and no true `WHEN`, error 7.3 with a condition queued), `w6`
(`LEAVE out` from an inner loop), `w8`/`w24`/`w27` (`INTERPRET` containing a
construct, and a construct whose condition contains an `INTERPRET`), `w11` (a
handler that *fails* at an `IF` header boundary -- blame attribution), `w12`
(`SELECT CASE` where a later `WHEN CASE` queues), `w14` (`trace r`, `DO FOREVER`
left by `LEAVE`).

---

## Important 1 -- closed

RAN. All four re-review probes go DIFF -> MATCH (`u7` `2,4,4`; `u8` `2,5,5`;
`u3` `2,4,6`; `u6` `2,5,8`), and so do `n1`, `n3`, `q1`, `q7`, `z7`, `z8`, `z9`
and my own `w7`, `w15`, `w17`, `w19`, `w20`, `w30`. The two probes that
*regressed* in round 3 (`u7`, `u8`) are back to byte-identical.

The rule is not fitted to those probes. RAN (read): the C++ citation behind
`HeaderClause` checks out -- `RexxInstructionBaseLoop::reExecute` is called by
whichever instruction transferred control back to the loop, and that
instruction is still `current`. The **adjacent success** is in the test itself:
`a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause`'s second row is
the same loop with the `ITERATE` removed, which must stay on `END`, and its
third row is an `UNTIL` program where both attributions alternate.

Mutations, RAN:

* **M1** (an `ITERATE`-ended pass hands the re-test to `END`, i.e. round 3's
  rule): **KILLED** `a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause`,
  1 failure of 296.
* **M2** (`END` echoes for an `ITERATE`-ended pass too): **KILLED**
  `end_does_not_echo_for_a_pass_an_iterate_ended`, 1 failure of 296.

The second divergence the implementer found while measuring the first -- `END`
does not echo for an `ITERATE`-ended pass -- is real and now correct (`n6`, and
`y4` lost one of its three diffs).

`do_body_outcome`'s arms are right for the two cases the brief singled out.
RAN: a loop left by `LEAVE` returns `Escaped(Goto(resume))` before
`header_clause` is ever consulted (`w14`, `q2`, `q9`, `w6`, `w23` all MATCH), and
a loop ended by its control test runs the header's boundary at the
`END`/`ITERATE` line and then returns `Goto(resume)` (`w1`, `w2`, `q3`, `q4`,
`w17`, `w19` all MATCH). `Iterated(origin.clause_line)` reads the line captured
at the `ITERATE`'s own step alongside `site` and `indent`, so it honours
`clause_line_override` -- `w24` (`interpret 'iterate'` inside a loop) MATCHes.

## Important 2 -- closed

RAN. `p2`, `p5`, `p7`, `p8` all DIFF -> MATCH, and so does `p9`, the fifth site
the re-review did not have (a **true** `WHEN` with `then` on the next line).
Independently, my `w3`, `w9`, `w10`, `w18`, `w21`, `w22`, `w23`, `w25`, `w26`,
`w28` cover the same family in shapes nobody had probed and all go DIFF ->
MATCH. Both halves are fixed, not just the line: `p8` and `w10` are wrong in
*timing* before the fix (`OTHERWISE`'s body reads the handler's variable unset,
or runs at all when it must not).

Mutations, RAN:

* **M3** (the `IF`'s condition is not a clause of its own): **KILLED**
  `a_construct_header_is_a_clause_with_its_own_boundary`, and it failed **on
  the tripwire** -- "a clause at line 4 began while a condition queued by this
  activation's clause at line 3 was still waiting".
* **M5** (a listed `WHEN`'s condition is not a clause of its own): **KILLED**
  that test **and** `a_single_line_then_reports_the_same_line_either_way`, the
  latter on the tripwire, exactly as the implementer claims. That is the
  claim's substance -- the single-line spelling is the case a value assertion
  cannot see, and it is now covered by something.
* **M6** (`INTERPRET` ends its header clause before the fragment, i.e. the fix
  applied by analogy): **KILLED**
  `an_interpret_clause_does_not_deliver_before_its_fragment_runs`. The "must
  not" direction is guarded.

**The derivation from C++ holds.** RAN (read `RexxActivation.cpp:600-680`
myself). `processClauseBoundary()` has exactly one call site in that file, at
line 653, immediately after `nextInst->execute(this, &stack)` at 642, inside
the instruction loop and guarded only by the `clauseBoundary` fast-path flag.
So a boundary does sit after every instruction of the activation's own flat
list, and the cited range `642-654` is the right one.

**The "diverges only where a `step` resolves other instructions" claim is true
of this crate.** RAN (read): `run_bounded` has exactly six call sites --
`If`'s true branch (has its own header clause), a matched `WHEN`'s body (has
one), `run_otherwise`'s body, `run_loop`'s `Simple` arm, `run_repeating`'s body
(has one), and `run_fragment` (deliberately has none). The two without a header
clause are the two whose header evaluates **nothing**: an `OTHERWISE` marker and
a `Simple` `DO`/`LOOP` block have no expression that could queue a condition, so
there is no boundary for them to owe. The absorbed `When`/`WhenCase` arms
evaluate a condition and return a `Flow` without nesting anything, so they are
ordinary stepped clauses and get their boundary from `step_in_temps_frame`.
I found no seventh construct.

## Important 3 -- closed

**All thirteen of re-review 3's attacks are now either inexpressible or
no-ops.** The token is gone, so A1-A4, A12 and A13 have nothing to name; A8's
forgery is E0603 rather than E0451; A5/A6/A7/A9/A10 change meaning under a
closure. The table is in "Attacks, re-run" below, with two new escapes I found
that are *not* in the finding.

The sharpest claim -- "round 1's Critical is now harmless" -- **holds, and for
a stronger reason than the report gives**. RAN: writing
`if matches!(flow, Ok(Flow::Return(_))) { return flow; }` immediately before the
closure's own `flow` builds (0), passes clippy `-D warnings` (0) and passes 296
lib tests (0), because inside the closure both paths return the same value from
the same closure -- it is not merely caught, it is *literally a no-op*. Round
3's identical three lines skipped the boundary and broke two tests.

RAN, the three compile-level claims, each exit status read:

| # | attack | result |
|---|---|---|
| B3 | `let saved = self.clause_state;` (round 3's own spelling) | **E0507**, `cannot move out of self.clause_state` |
| B4 | `SavedClauseState(ClauseState::new())` from `run.rs` | **E0603**, `tuple struct constructor ... is private` |
| B2 | `self.clause_state = ClauseState::new()` | compiles; at a *harmless* placement 296 tests stay green, at the damaging one (after the restore) **17 tests fail** -- so the "loud in its own right" claim holds where it matters |

## Important 4 -- closed

**By test, which is the half that actually holds.** RAN, with the negative
control the finding asked for: with `ClauseValue for Flow` returning `None`
(the implementer's M7), `cargo test -p rexx-exec --lib` is **296 passed / 0
failed** and `cargo test -p rexx-exec --test collect_stress` **fails**, panicking
`a live value` at `value.rs:125:47`. That is precisely the gap NEW-4 named: the
lib suite cannot see it and the new test can. The test carries its own
anti-vacuity assertion (`stress.collections > 0` per row), so a run that
collected nothing could not pass it.

**By type, narrower than claimed** -- see NEW-2 below. `ClauseEnd::
Completed(Option<&Flow>)` is gone and the trait has no default body, so the
*literal* mutation the finding named is unwritable; but a site can still route
its `Flow` around the closure's return type and land on `()`.

## The three Minors -- closed

* **`lib.rs`'s `pending_trap` doc.** RAN (read the diff). It no longer names
  `clause_boundary`, no longer names `run_activation`'s removed check, and no
  longer asserts the two-callers property; it states that the site set is
  derived and points at `clause.rs`. The sentence NEW-5 quoted is gone.
* **The `unreachable!` invariant.** RAN (read). `deliver_pending_trap` returns
  `Option<HandlerExit>`; `HandlerExit`'s only constructor is
  `from_ended(Ended) -> Option<HandlerExit>`, answering `None` for
  `Ended::Returned`; and it is called at exactly one place
  (`run.rs:2663`) on an arm that has already matched `Ended::Exited(_)`. So the
  invariant is a type again, and it fails in the safe direction rather than
  rendering a `RETURN` as an `EXIT`. Better than round 2's `unreachable!`.
* **The `UNTIL` re-test boundary.** Closed in substance, RAN: the line and the
  boundary are one `in_clause` call, so re-review 3's mutation ("keep the line,
  drop the boundary") is genuinely not expressible, and **M-U** -- dropping the
  whole call and evaluating the `UNTIL` condition directly -- fails **two**
  tests (`a_while_retest_belongs_to_the_do_clause_then_to_the_end_clause` and
  `a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause`). The site is
  load-bearing. One false comment is left behind; see NEW-1.

---

# Attacks, re-run

Every row RAN: applied to the tracked source, `cargo build -p rexx-exec --lib`,
then `cargo clippy --workspace --all-targets -- -D warnings`, then
`cargo test -p rexx-exec --lib`, then reverted.

## Re-review 3's thirteen, against the scoped closure

| # | round-3 attack | against `in_clause` | verdict |
|---|---|---|---|
| A1 | delete the `end_clause` call | there is no `end_clause`; the boundary is the tail of `in_clause` | **not expressible** |
| A2 | `let _token = …` | no token | **not expressible** |
| A3 | `std::mem::forget(token)` | no token | **not expressible** |
| A4 | `drop(token)` | no token | **not expressible** |
| A5 | early `return` between the halves | inside the closure it returns from the closure and the boundary still runs; outside it the boundary already ran | **no longer an escape** |
| A5b | **round 1's own Critical re-expressed** | build 0, clippy 0, **tests 0** -- and it is a literal no-op, not merely caught | **no longer an escape** |
| A6 | a `?` between the halves | inside the closure it becomes the clause's own `Err`, which is what "a failed clause reached no boundary" already means | **no longer an escape** |
| A7 | set the line with no token (`self.clause_state = …`) | only the zero value is constructible (B2); an arbitrary line is not | **narrowed, not removed** -- see NEW-3 |
| A8 | forge the token | `E0603` | **caught** |
| A9 | satisfy the token with `ClauseEnd::Failed` on the `Ok` path | `ClauseEnd` is gone; the closure's `Result` is the only signal | **not expressible** |
| A10 | `ClauseEnd::Completed(None)` at a `Flow` site | the return type decides; `Completed` is gone | **not expressible as written** -- but see NEW-2 |
| A12 | A1 with `#[must_use]` removed | no attribute, no token | **not expressible** |
| A13 | `self.enter_clause(line);` bare | no token to leave unused | **not expressible** |

## New attacks, and what escapes now

| # | attack | build | clippy | lib tests | other | verdict |
|---|---|---|---|---|---|---|
| C1 | run the clause inside the closure but **smuggle the `Flow` out** through a captured `&mut Option<Flow>` and return `Ok(())` | 0 | 0 | **0** (296 pass) | `collect_stress` **fails**, `a live value` | **escapes the type, caught by the test** (NEW-2) |
| C3 | `save_clause_state()` … `restore_clause_state(stale)` around a step, i.e. set a **nonzero** clause line with no boundary | 0 | 0 | **0** (296 pass) | -- | **escapes entirely** at a subtle placement (NEW-3) |
| C3b | the same at a damaging placement (every clause reports the previous clause's line) | 0 | -- | 101 (9 fail) | -- | caught by tests |
| B2 | `self.clause_state = ClauseState::new()` | 0 | -- | 0 or 101 depending on placement | -- | expressible, named in the doc |

**Summary of what the scoped closure catches and what escapes.** It catches --
by making unwritable -- every spelling of "the two halves came apart" that
round 3's token left open, including round 1's own Critical defect, which is
the finding's headline and is now a no-op rather than a live mutation. It does
**not** catch three things, two of which the module doc names: declining to call
`in_clause` at all (the tripwire's job, and it does fire -- M3, M5); resetting
the state to line 0 (named, and loud); putting back a *stale* saved state at an
arbitrary moment (**not named**, NEW-3); and routing the clause's value around
the closure's return type (**not named**, NEW-2, caught by the new stress
test rather than by the type).

# The tripwire, tested rather than read

Four questions, all RAN.

* **Does it fire when it should?** Yes. M3 and M5 both abort the test suite with
  the assertion's own message and the right two line numbers. M3 fails exactly
  one test, M5 exactly two.
* **Is it reachable where it matters?** Yes under `cargo test`: the workspace
  `Cargo.toml` sets no `[profile]` overrides, so the dev profile keeps
  `debug-assertions = true`, and the corpus gate, the assertions harness and
  every integration test run under it. Confirmed by a **negative control**: with
  M3 applied I rebuilt the *debug* `rexx-run` and ran `p2` through it -- rc 101,
  `panicked at crates/rexx-exec/src/clause.rs:387:9`, tripwire message. It is
  absent from release, which the report discloses.
* **Is it over-eager?** No, over 93 shapes. RAN: I ran **all 93 probes through
  the debug `rexx-run`** -- zero tripwire firings, zero panics, and every
  debug stdout byte-identical to the release stdout it was compared against.
  Given the negative control above, that silence is a measurement rather than a
  broken instrument.
* **Is the stated exemption the true scope?** Substantially yes. The assertion
  is silent when the lines are equal **or** no condition is waiting for this
  activation. The comment lists two legitimate same-line pairs (a `DO`'s control
  setup and its first header test; `INTERPRET`'s `clause_line_override`) and
  then states the residual plainly -- "a delivery that is late in *time* but
  lands on the same line is invisible to it, and to `SIGL`". That residual, not
  the two bullets, is the true bound, and it is written down. Accepted.

# Performance

RAN, three shapes, three runs each, release binaries, HEAD vs `f9e6bf26`:

| shape | `f9e6bf26` | `9a4b57be` |
|---|---|---|
| `bench1` -- 400k controlled loop with an `IF` header and an `ITERATE` per pass | 1.35 / 1.36 / 1.35 s | 1.36 / 1.34 / 1.34 s |
| `bench2` -- 200k loop with a `SELECT` of two `WHEN`s + `OTHERWISE` per pass (three extra header clauses per iteration) | 0.88 / 0.88 / 0.89 s | 0.89 / 0.89 / 0.89 s |
| `bench3` -- 400k plain controlled loop, no construct header at all | 0.59 / 0.58 / 0.58 s | 0.58 / 0.58 / 0.58 s |

No measurable cost on any of the three, including the shape built specifically
to stress the new per-construct-header boundary. The implementer's "~1.5%" was
conservative; I cannot separate the two at this sample size. **Concern
withdrawn.**

---

# New findings

## NEW-1 (MINOR, RAN, parkable) -- a comment names a test that does not exist

`run.rs:4579-4580`, at the `UNTIL` re-test's `in_clause`:

> dropping both is what `a_until_retest_reports_the_end_clauses_line` fails on.

`grep -rn "a_until_retest" crates/` returns **only that comment**. No such test
exists anywhere in the tree. RAN: the tests that actually fail when the whole
call is dropped (M-U) are
`a_while_retest_belongs_to_the_do_clause_then_to_the_end_clause` (`run.rs:12010`)
and `a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause`. The
report's own Minors section names the first of those two correctly, so this is a
slip in the code comment alone.

`rust/CLAUDE.md` requires a false comment corrected or removed rather than
hedged, and "a comment naming a function that is not there" is literally the
shape of round 3's own NEW-5. The claim it decorates is *true* and was measured
here; only the name is wrong. One-word fix.

## NEW-2 (MINOR, RAN, parkable) -- Important 4's "by type" half is narrower than `clause.rs` states

`clause.rs:245-250` says of `ClauseValue`: "Round 3 expressed this as
`ClauseEnd::Completed(Option<&Flow>)`, where a site holding a `Flow` could pass
`None` … Here the value comes from the closure's own return type, so the site
cannot choose."

RAN (C1): a site *can* choose. Compute the `Flow` inside the closure, write it
to a captured `&mut Option<Flow>`, and return `Ok(())` -- `ClauseValue for ()`
then answers `None` and the rooting is silently dropped. Build **0**, clippy
`-D warnings` **0**, `cargo test -p rexx-exec --lib` **0 (296 passed)**. This is
not a contrived spelling: the file already smuggles two values out of
`in_clause` closures exactly this way (`&mut outcome` in the `WHEN` scan,
`case_text` in the `SELECT CASE` arm), so it is the idiom a future site would
reach for.

**Not load-bearing**, for one measured reason: the new `collect_stress` test
catches it. Under C1 that test fails with the same `a live value` panic as under
M7. So the round closed NEW-4 twice over and the *second* closure is the one
carrying the weight -- which inverts the report's emphasis but not its verdict.
The sentence quoted above should be bounded to what it does ("the literal
`Completed(None)` spelling is gone; the rooting is pinned by
`a_clause_value_survives_the_handler_its_boundary_runs`, not by the type") in
the same way the module doc already bounds its other claims.

## NEW-3 (MINOR, RAN, parkable) -- the residual enumeration in `clause.rs` is short by one spelling

`clause.rs:97-98`: "So the reachable spellings are `in_clause` and 'reset to
line 0', and the second one is loud in its own right."

RAN (C3): there is a third. `save_clause_state`/`restore_clause_state` are both
`pub(crate)`, so `run.rs` can take a state at one moment and put it back at
another, setting the clause line to a **nonzero** value with no boundary
attached. Wrapping `run_bounded`'s own `step_in_temps_frame` call in a
save/restore pair builds **0**, passes clippy **0** and passes **296 of 296**
lib tests. A blatant placement is caught (C3b, 9 failures), a subtle one is not
caught by anything.

Adjacent to it, and also unnamed: `deliver_pending_trap` is `pub(crate)` and has
two callers (`in_clause` and `offer_to_trap`), so "run a boundary with no
clause" is expressible too -- the mirror image of the residual the doc does
name.

Both are true residuals of a design that has to expose *something* for the
legitimate save/restore, and the doc's own standard is to state bounds rather
than guarantees; it just stops one bullet early. `SIGL` regression tests are
what actually cover this today.

---

## What I checked and found correct

Recorded because a clean result is part of the answer.

* **The C++ derivation.** RAN, read directly: one `processClauseBoundary()` call
  site, at `RexxActivation.cpp:653`, immediately after the `execute` at 642, in
  the instruction loop. The citation and the conclusion are both right.
* **The divergence set.** RAN, read: six `run_bounded` call sites; the two
  without a header clause (`run_otherwise`, `run_loop`'s `Simple` arm) have
  nothing to evaluate, so they can queue nothing; `run_fragment` is the
  deliberate exception and has M6 guarding it. No seventh construct.
* **`HandlerExit` is a real invariant.** RAN, read: one construction point, on an
  arm that has already matched `Ended::Exited(_)`, failing in the safe
  direction.
* **The `Ended` arm at each new site works.** RAN: `w9` (`IF` header), `w10`
  (false `WHEN`), `w18` (`SELECT CASE`), `w19`/`w20` (`WHILE`/`UNTIL` re-test
  after an `ITERATE`) -- a handler that `EXIT`s from each of the five new
  boundary sites, all MATCH including exit status.
* **Blame attribution at a new site.** RAN: `w11`, a handler that raises 42.3
  while delivered at an `IF` header boundary under `SIGNAL ON SYNTAX`, MATCHes
  (`SH 10`, rc 9).
* **Rooting is still load-bearing at all three shapes.** RAN: `sr1`/`sr3`/`sr4`
  MATCH, and `collect_stress` is green unmutated and red under both M7 and C1.
* **`INTERPRET` in every direction.** RAN: `p11`, `n5`, `w8` (a construct inside
  fragment text), `w24` (`interpret 'iterate'` inside a loop), `w27` (an
  `INTERPRET` inside a `WHEN`'s condition) all MATCH; M6 stops the fix being
  applied here by analogy.
* **Gates reproduce exactly**, listed above.

## Pre-existing, found in passing, not this round's

* The controlled-`DO` `>>>` control-variable trace lines (`q11`, `y4`, `z6`,
  `w29`) -- DIFF on `f9e6bf26` too, disclosed as a KNOWN GAP at `run.rs:4741`.
* The parse-error report format: an invalid `raise user foo return zn` gets
  error 35.1 from both, but the oracle echoes the clause and exits 221 where
  this crate prints one line and exits 120. Identical on `f9e6bf26`; the
  report's own standing concern 5 already names the parse-error reporting arm.
