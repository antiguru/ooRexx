# Re-review of Task 7 fix round 3

Scope: commit `f9e6bf26` against its parent `26da3ac6`. Did fix round 3 close
the five findings of `task-7-rereview2.md` (NEW-A..NEW-E), and did it introduce
anything new. **Not** a re-review of Task 7 as a whole; the earlier rounds'
settled verdicts are taken as settled.

Working tree clean at start and at end. Every tracked-file mutation below was
applied, measured, and reverted with `git checkout -- <path>`;
`git status --porcelain` was empty after each restore and is empty now. The
release binary was rebuilt from the restored source at the end.

Every claim is labelled **RAN** or **REASONED**.

## How things were established

* `<scratchpad>/rr7c` -- a fresh probe directory `mkdir`ed for this re-review,
  holding only my own files. Oracle wrapper
  `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`,
  stdout/stderr/exit status as three separate descriptors, absolute paths for
  every redirect. 38 probes.
* `<scratchpad>/wt-r2` -- a detached `git worktree` at `26da3ac6` with its own
  `rexx-run`, so "is this a regression or a site the round never reached?" is a
  measurement and not an inference. Run on all 38 probes.
* `<scratchpad>/stress3` -- a throwaway crate with a path dependency on
  `rexx-exec`, calling the `#[doc(hidden)]` `run_program_collect_every_alloc`,
  used to answer the rooting question by running, with a negative control.
* Thirteen source mutations (A1-A13, M1-M4), each built and/or tested and then
  reverted.

Gates, re-established here rather than taken from the report -- RAN, each exit
status read unpiped: `cargo fmt --all --check` **0**;
`cargo clippy --workspace --all-targets -- -D warnings` **0**;
`cargo test --workspace` **exit 0, 970 passed / 0 failed**;
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec` **exit 0, `38 of 38 matching`,
mode STRICT, 4224 of 4259 assertion rows**. The controller's baseline
reproduces exactly.

---

## Per-finding verdict

| finding | verdict |
|---|---|
| NEW-A -- "exactly two places that step a clause" misses the `DO` header | **partially closed** |
| NEW-B -- a failing boundary handler blamed on the enclosing `DO` | **closed** |
| NEW-C -- `deliver_pending_trap`'s doc left on `clause_boundary` | **closed** |
| NEW-D -- `run_bounded`'s "returns the escaping `Flow` unchanged" | **closed** |
| NEW-E -- `restoring_none_is_still_the_common_case` a weaker duplicate | **closed** |

---

## NEW-A -- partially closed

**The behaviour half is a real, measured improvement.** RAN. Overall tally over
the 38 probes: **11 went DIFF -> MATCH, 16 were MATCH on both, 9 are DIFF on
both, and 2 went MATCH -> DIFF** (NEW-1). Eight of the eleven belong to NEW-A
(the other three are NEW-B's), and six of those eight are shapes no round or
review named:

| probe | shape | `26da3ac6` | now |
|---|---|---|---|
| `q2` | `do while zn < sub()` with a `LEAVE` in the body | 5, 5, 5 | MATCH (4, 8, 8) |
| `q5` | `do i = 1 to 9 for sub()` | `NOMARK`, then 4 | MATCH (3, 3, 3) |
| `q6` | `do sub()` (bare repeat count) | `NOMARK`, then 4 | MATCH |
| `q8` | nested loops, inner header queues | `NOMARK`, then 5 | MATCH |
| `q9` | `do forever while zn < sub()` | 5, 5, 5 | MATCH (4, 8, 8) |
| `q10` | `INTERPRET` text containing a loop whose header queues | `NOMARK`, then 4 | MATCH |
| `u1` | `SIGL` read from inside an `UNTIL` re-test, no trap at all | 3, 3 | MATCH (4, 4) |
| `u2` | `SIGL` read from inside a `WHILE` re-test | 2, 3, 3 | MATCH (2, 4, 4) |
| `q1`(partly), `q7`(partly) | `WHILE`/`UNTIL` re-test with an `ITERATE` | worse | improved, still DIFF |

`q2`/`q3`/`q4`/`q11` were already MATCH on round 2 and still are, which is the
adjacent-success half.

**But the round was asked for a construction that makes a fourth site
impossible or self-announcing, and there is a fourth, a fifth, a sixth and a
seventh site -- all silent.** RAN. See NEW-1 and NEW-2 below: four clause sites
still have no `enter_clause` at all and are measurably wrong, and the rule the
round *did* install for the loop re-test is wrong for a third member of the
family (`ITERATE`), which turns two previously-matching shapes into DIFFs.

**And the third of the three comments NEW-A named was not touched.** RAN. See
NEW-5: `lib.rs`'s `pending_trap` doc still asserts the exact property NEW-A
falsified, and now also names a function this commit deleted.

---

## NEW-B -- closed

**Established: RAN**, and it generalises past the reported shape.

| probe | shape | oracle second echo | `26da3ac6` | now |
|---|---|---|---|---|
| `e5` | failing handler at a `call sub` boundary inside `do i = 1 to 1` | `3 *-* call sub` | `2 *-* do i = 1 to 1` | **MATCH** |
| `e6` | the same inside a `WHEN`/`THEN DO` body | `4 *-* call sub` | `3 *-* do` | **MATCH** |
| `e7` | the same at top level, no enclosing construct | `2 *-* call sub` | *line missing entirely* | **MATCH** |
| `e2` | `e5` under an outer `SIGNAL ON SYNTAX` | `SH ran 11`, rc 5 | MATCH | MATCH |
| `f1` | the no-trap control: `zz = 1/0` inside `sub` called from a `DO` body | `2 *-* call sub` | MATCH | MATCH |

`e7` is the interesting one: round 2 omitted the second echo line at top level
too, which neither the report nor re-review 2 recorded. The control `f1` was
already correct and stays correct, so the fix is pinned to the boundary and not
to anything about `DO`.

The test `a_handler_that_fails_at_a_clause_boundary_blames_that_clause` can
fail: mutation **A9** (satisfy the token with `ClauseEnd::Failed` on the `Ok`
path) kills it along with eight others.

---

## NEW-C -- closed

**Established: RAN** (read). `clause_boundary` no longer exists;
`deliver_pending_trap` at `run.rs:2489` carries its own three paragraphs
(`run.rs:2473-2488`) -- "Runs a `CALL ON` trap's handler…", "**The wait is the
measured part.**", "The trap is removed for the handler's duration…" -- with no
fused second summary sentence and no missing `///` separator. The paragraphs
that were `clause_boundary`'s have moved into `clause.rs`, on the items that
implement them.

---

## NEW-D -- closed

**Established: RAN** (read). `run_bounded`'s body is back to
`Flow::Next => pc += 1` / in-range `Goto` / `other => return Ok(other)`; it
synthesises no `Flow` of its own, so "Every other exit returns the escaping
`Flow` unchanged" (`run.rs:3888-3890`) is true again as written. The `Flow::Exit` a
delivered handler's `EXIT` produces is now built one level down, inside
`step_in_temps_frame`, and reaches `run_bounded` as an ordinary escaping `Flow`
-- which is exactly what that sentence describes. The `unreachable!` that used
to guard the conversion is gone with it; see NEW-6 for what replaced it.

---

## NEW-E -- closed

**Established: RAN** (grep). `restoring_none_is_still_the_common_case` is gone
from the tree; `a_returned_call_handler_leaves_no_active_condition_to_propagate`
(`run.rs:11511`), the stronger test it duplicated, is unchanged and passing.

---

# The type obligation, attacked

This is the heart of the round, so it was attacked rather than read. Thirteen
mutations, each applied to the tracked source, built with `cargo build -p
rexx-exec --lib` **and** `cargo clippy -p rexx-exec --lib --all-targets --
-D warnings`, then reverted. All RAN.

| # | attack | `cargo build` | clippy `-D warnings` | verdict |
|---|---|---|---|---|
| A1 | delete the `end_clause` call outright | **0** (warning) | **101** | caught |
| A2 | `let _token = self.enter_clause(line);` | 0 | **0** | **escapes** (named by the implementer) |
| A3 | `std::mem::forget(token)` | 0 | **101** (`forget_non_drop`) | caught |
| A4 | `drop(token)` | 0 | **0** | **escapes** |
| A5 | an early `return` between `enter_clause` and `end_clause` | 0 | **0** | **escapes** |
| A5b | round 1's own Critical re-expressed: `if matches!(flow, Ok(Flow::Return(_))) { return flow; }` before the boundary | 0 | **0** | **escapes** the gate; caught only by two tests |
| A6 | a `?` between `enter_clause` and `end_clause` | 0 | **0** | **escapes** |
| A7 | set the clause line from `run.rs` with no token (`self.clause_state = ClauseState::new()` / a saved copy) | 0 | **0** | **escapes** |
| A8 | forge a `ClauseToken` in `run.rs` | **101** (`E0451`, private field) | 101 | caught |
| A9 | satisfy the token with `ClauseEnd::Failed` on the `Ok` path | 0 | **0** | **escapes** the gate; caught by 9 tests |
| A10 | `ClauseEnd::Completed(None)` at a site that has a `Flow` | 0 | **0** | **escapes** the gate **and every test** -- see NEW-4 |
| A12 | A1 with `#[must_use]` also removed | 0 | **101**, same `unused variable: token` | the attribute is not what caught A1 |
| A13 | `self.enter_clause(line);` as a bare statement, no binding | 0 | **101** (`unused … that must be used`) | this is what `#[must_use]` catches |

**Verdict: the obligation is real but narrow, and it is weaker than the module
doc claims.** It stops exactly one thing -- *never mentioning the token at
all*. Every path-sensitive omission (A4, A5, A5b, A6) compiles clean under the
full gate, and A5b is round 1's own Critical defect, re-expressible in three
lines. The private field genuinely blocks forgery (A8), which is worth having.
But "doing one without the other is not expressible" (`clause.rs:36-39`) is not
what was built: what was built is "a site that ignores the token entirely does
not pass clippy". Two spellings that a reviewer would plausibly write --
`drop(token)` and an early `return` -- are not named in the module doc's stated
residual, which lists only `let _token`.

Three further precision points, all RAN:

* **`cargo build` exits 0 with a warning**, and the whole test suite still
  runs. Only `clippy -D warnings` rejects A1. The module doc's bolded "a clause
  that is entered and never ended **does not build**" (`clause.rs:57`) is true
  only of the clippy gate, which the sentence two lines above does say --
  but the bolded claim is the one a reader takes away.
* **`#[must_use]` is not what produces the quoted transcript.** A12: with the
  attribute deleted, deleting the `end_clause` call still fails with the
  identical `error: unused variable: token`. That diagnostic is
  `unused_variables`. `#[must_use]` covers a different spelling (A13, the bare
  statement), so it is not decorative -- but `ClauseToken`'s own doc
  ("`#[must_use]` so that dropping one is a compile error … a site that begins
  a clause and never ends it does not build") attributes the guarantee to the
  wrong mechanism.
* **The module-privacy argument holds for the field and fails for the
  struct.** RAN. `current_clause_line` is private to `clause.rs`, there is no
  `Default`, no setter, no re-export, and `line()` is read-only -- so `run.rs`
  cannot name the field (A8-adjacent, confirmed by grep and by A7's variant
  that tried). But `ClauseState` is `#[derive(Copy, Clone)]` and the whole
  struct is assignable, so `self.clause_state = <a ClauseState>` sets the line
  with no token. That is not hypothetical: `run.rs:3162`/`run.rs:3202` do it
  today (`resolve_and_run_call`'s save/restore), and A7's fresh
  `self.clause_state = ClauseState::new();` compiles clean. The restore is
  correct -- it can only put back a value some `enter_clause` produced -- but
  the module doc's "`Interp::enter_clause` is the only way to set the line"
  (`clause.rs:39`, and the commit message) is false as written.

---

# New findings

## NEW-1 (IMPORTANT, RAN) -- a loop re-test after an `ITERATE` is attributed to the `END` clause; the oracle attributes it to the `ITERATE` clause, and two previously-matching shapes regressed

The round installed one rule -- "the `DO` clause on the first pass, the `END`
clause on every re-test after it" -- measured on two members of the family. A
third member falsifies it: when the body's last executed clause is an
`ITERATE`, the re-test belongs to the **`ITERATE`** clause.

Measured with **no condition trap anywhere**, so this is a plain `SIGL`
question:

| probe | shape | oracle | `26da3ac6` | now |
|---|---|---|---|---|
| `u7` | `do i = 1 to 3 while zs() < 3` / `zn = zn + 1` / `iterate` / `end` | `2, 4, 4` | **MATCH** | **`2, 5, 5`** |
| `u8` | `do label lab while zs() < 2` / … / `iterate lab` from an inner loop | `2, 5, 5` | **MATCH** | **`2, 7, 7`** |
| `u3` | `do while zs() < 2` with `if zn = 1 then iterate` | `2, 4, 6` | `2, 4, 5` | `2, 6, 6` |
| `u6` | the same with the `iterate` inside a `THEN DO` | `2, 5, 8` | `2, 5, 7` | `2, 8, 8` |

**`u7` and `u8` are regressions**: round 2 matched the oracle byte for byte and
round 3 does not. `u3`/`u6` each trade one correct value for another (round 2
had the `ITERATE` re-test right and the fall-through re-test wrong; round 3 has
it the other way round).

The same defect shows in delivery timing under a trap -- `q1` (`do while zn <
sub()` with an `ITERATE`): oracle `4, 7, 7`, ours `4, 9, 9`; and `q7`, the
`UNTIL` twin: oracle `NOMARK, 7, 7`, ours `NOMARK, 9, 9`.

Nothing catches this: 970 tests, `38 of 38` STRICT and 4224/4259 assertions all
pass with it.

Two comments assert the false rule and `rust/CLAUDE.md` requires them corrected
rather than hedged:

* `run.rs:4230-4232` -- "Which clause the loop header's own evaluation belongs
  to: the `DO` clause on the first pass, the `END` clause on every re-test
  after it."
* `run.rs:11783-11787`, `a_while_retest_belongs_to_the_do_clause_then_to_the_end_clause`'s doc -- "the `DO` clause on the first pass, the `END` clause on every
  one after".

The rule that fits all six probes is "the clause control last transferred
from": the `DO` clause on the first pass, the `ITERATE`'s own clause when an
`ITERATE` ended the pass, the `END` clause when the body fell through.

## NEW-2 (IMPORTANT, RAN) -- four clause sites still have no `enter_clause` and no boundary; `enter_clause`'s own doc says every site has one

Not regressions -- all four are byte-identical on `26da3ac6` -- but they are
the fourth, fifth, sixth and seventh members of the family the round set out to
close by construction, and they are silent: nothing in the type, the gate or
the test suite announces them.

| probe | shape | oracle | ours (and `26da3ac6`) |
|---|---|---|---|
| `p2` | `if sub() = 'SV'` with `then` on the **next** line | `HANDLER-AT 3` (the `IF` clause) | `HANDLER-AT 4` (the `THEN` marker) |
| `p5` | a `WHEN` whose condition queues and is **false**, a later `WHEN` wins | `HANDLER-AT 4` | `HANDLER-AT 5` (the next `WHEN`) |
| `p7` | `select case sub()` | `HANDLER-AT 3` (the `SELECT` clause) | `HANDLER-AT 4` |
| `p8` | a single `WHEN` whose condition queues and is false, falling to `OTHERWISE` | `oth mark= HANDLER-AT 4` | `oth mark= NOMARK`, then `HANDLER-AT 7` -- wrong in timing **and** line |

`p1`, `p3`, `p4`, `p6`, `p9` are the adjacent successes and are MATCH -- and
they show why this was not caught: `if sub() … then say …` on one line, and
`when sub() … then say …` on one line, both put the *next stepped* clause on
the same line as the clause that owed the boundary, so the wrong answer and the
right answer coincide. Moving `then` to its own line separates them.

The mechanism is the one `run_loop` had before this round: an `IF`'s condition,
a `WHEN`'s condition and a `SELECT CASE`'s expression are all evaluated inside
an enclosing instruction's own `step`, so the enclosing `step_in_temps_frame`'s
`enter_clause`/`end_clause` pair brackets far more than one clause, and the
first clause stepped *inside* it collects the boundary.

`clause.rs:191` therefore states something false: "**Every site that runs a
clause calls this**, and the set is wider than 'instructions stepped'". The set
is wider than the round made it, and by the round's own standard a bounded true
sentence is required rather than a hedge.

## NEW-3 (IMPORTANT, RAN) -- the token does not survive adversarial attack: `drop(token)`, an early `return` and a `?` all skip the boundary and pass the full gate

Detailed in the table above. The headline: **A5b re-expresses round 1's own
Critical defect** -- skip the boundary when the clause resolved to
`Flow::Return` -- in three lines that pass `cargo clippy --workspace
--all-targets -- -D warnings` cleanly. It is caught by two tests, which is
where round 1's defect was caught in the end anyway. The construction has
narrowed the class rather than removed it, and the module doc's two-sentence
statement of what is and is not guaranteed names only one of the four escapes I
found.

## NEW-4 (IMPORTANT, RAN) -- `ClauseEnd::Completed(None)` silently drops the GC rooting, and no gate or test catches it

`ClauseEnd::Completed(Option<&Flow>)` is new in this round; round 2's
`clause_boundary(&mut self, code, flow: &Flow)` took the `Flow` unconditionally,
so "forget the rooting" was not expressible. It is now.

RAN, with a negative control:

| mutation | clippy | `cargo test -p rexx-exec` | under `run_program_collect_every_alloc` |
|---|---|---|---|
| A10: `Completed(None)` at `step_in_temps_frame`'s `Ok` arm | **0** | **exit 0, 291 + all integration tests pass** | `sr1` and `sr3` **panic**, `a live value`, `value.rs:125:47` |
| A11: delete the `push_temp` inside `end_clause` (the negative control the doc names) | -- | -- | `sr1` and `sr3` panic, **identical site and message** |

So passing `None` where a `Flow` exists is exactly as destructive as deleting
the rooting the round's own doc says was "measured with a negative control", and
it is invisible to clippy, to the 970-test suite and to `collect_stress`. `sr1`
(`return bb() || 'TAIL'` at an activation boundary) and `sr3` (the same inside a
`DO` body) both pass unmutated, at 9 and 11 real collections.

This is latent -- the one site that has a `Flow` passes `Some(flow)` today, so
no output is wrong -- but it is a use-after-free hazard newly created by the
round whose stated purpose was removing hand-maintained obligations, and three
of the four call sites already spell the `None` form.

## NEW-5 (MINOR, RAN) -- `lib.rs`'s `pending_trap` doc, one of the three comments NEW-A named, is untouched and now names a deleted function

`git diff 26da3ac6 f9e6bf26 -- crates/rexx-exec/src/lib.rs` does not touch
`lib.rs:947-966`, which still says:

> `run_activation`'s check runs once per *clause*, on every path out of one --
> placed before the `Flow` dispatch precisely so a `RETURN` or `EXIT` cannot
> skip it …
>
> **"Every clause" means every clause, and that took a third go** … The check
> lives in `run.rs`'s `clause_boundary`, which is called from both -- and only
> both -- of the places that step a clause: `run_activation`'s loop and
> `run_bounded`'s. … The two callers are the two callers of
> `step_in_temps_frame`, which is what makes "every clause" checkable rather
> than aspirational.

`clause_boundary` was deleted by this commit; `run_activation`'s check was
removed by this commit; and the final sentence is verbatim the claim NEW-A
falsified. `git log -S clause_boundary -- lib.rs` returns `26da3ac6` as the last
touch, confirming round 3 left it alone. Round 3 corrected the two `run.rs`
copies (both vanished with the function) and missed the third, which was the one
NEW-A quoted first.

## NEW-6 (MINOR, RAN + REASONED) -- the `unreachable!` that announced the `Ended::Exited` invariant is gone, replaced by a silent `Ended::value()`

Round 2's `run_bounded` had
`unreachable!("clause_boundary reports only Ended::Exited")`. Both of round 3's
new sites -- `run.rs:3515`, `run.rs:2468` and four more in `run_repeating` (`4273`, `4297`, `4309`, `4377`) -- now write
`Ok(Flow::Exit(ended.value()))`, and `Ended::value()` (`run.rs:264`) collapses
`Returned` and `Exited` into the same `Option<ObjRef>`. RAN (read): the
invariant holds today, because `deliver_pending_trap`'s only `Ok(Some(…))` is
`Ok(Some(Ended::Exited(value)))`. REASONED: a future `deliver_pending_trap`
that returned `Returned` would now turn a `RETURN` into an `EXIT` in silence,
where round 2 aborted and said why. Re-review 2 flagged this invariant as "the
kind this project has been bitten by"; round 3 removed its only announcement
without replacing it with a type.

## NEW-7 (MINOR, RAN) -- the `UNTIL` re-test's *boundary* is unobservable and untested; only its `enter_clause` is load-bearing

The round presents the `UNTIL` re-test as the site "no round and no reviewer
enumerated", found by probing the family. Half of that site does nothing:

* **M4** -- replace the `until_token`'s `end_clause(…, Completed(None))` with
  `end_clause(…, Failed)`, i.e. keep the line change and remove the boundary --
  leaves **291 of 291** lib tests green, and produces **byte-identical stdout,
  stderr and exit status on all 38 probes in this re-review**, `q7` (the `UNTIL`
  delivery probe) and `u1` (the `UNTIL` `SIGL` probe) included. RAN.
* The reason, REASONED from the code and consistent with the sweep: between the
  `UNTIL` test and the top-of-loop `end_clause` no user clause runs, and both
  deliver at the same `end_line`, so the two boundaries are indistinguishable.
* The `enter_clause(end_line)` half **is** load-bearing: `u1` (`do until zs() >=
  2`, `zs` prints `SIGL`) is `4, 4` on the oracle and here, and `3, 3` on
  `26da3ac6`. RAN.

Not wrong, and cheap; but the round's own mutation table lists five mutations
that "KILLED", and this one -- the site it presents as its discovery -- has no
test that fails when its boundary is removed.

---

## What I checked and found correct

Recorded because a clean result is part of the answer.

* **`step_in_temps_frame` is otherwise unchanged on the hot path.** RAN (read
  and diff). The one semantic change beside the boundary is
  `clause_line(...).unwrap_or_else(|| self.clause_state.line())` replacing
  `if let Some(line) = … { … }` -- the same value in both branches.
  `end_clause`'s first two statements are the `ClauseEnd::Failed` early-out and
  `if self.pending_trap.is_none() { return Ok(None); }`, and `pending_trap` has
  exactly one writer in the crate (`exec_raise`), so a program that never
  queues a `CALL ON` condition takes a bit-identical path. Temps-frame
  lifetime, the debug watermark tripwire and `pop_frame` are untouched and
  still sit *before* the boundary.
* **`LEAVE`, `SIGNAL` out of a body, loop control and trace are unaffected.**
  RAN: `q2` (`LEAVE` from a `DO WHILE`), `q9` (`DO FOREVER WHILE` + `LEAVE`),
  `q4` (zero-trip controlled `DO`), `q3` (zero-trip `DO WHILE`), `q8` (nested
  loops), `u4` (`SIGL` from a `TO` expression) all MATCH.
* **The `offer_to_trap` exception is correct in every shape I could build.**
  RAN, with the adjacent success: `o1` (`zq = sub() + 1/0` under both traps,
  `UH ran 3` then `SH ran 3`, rc 3), `o2` (the same inside `INTERPRET`), `o3`
  (inside a `DO` body), `o4` (inside a `WHEN` body) all MATCH; `o5`, where the
  failure is *not* trapped here and unwinds the activation, delivers nothing at
  all and MATCHes at rc 214. "A failed clause has not completed" holds across
  nesting and across `INTERPRET`, and the pair `o1`/`o5` is what separates the
  two.
* **`o6` is pre-existing, not this round's.** RAN. A `CALL ON` handler that
  `EXIT`s while delivered from `offer_to_trap`: the oracle still runs the
  `SIGNAL ON SYNTAX` handler and ends at rc 3, we end at rc 7. `26da3ac6` gives
  rc 7 too, byte-identical.
* **The two new loop tests are not duplicates and each pins its own property.**
  RAN: **M1** (the header enters a clause but has no boundary) kills both;
  **M2** (`header_line = do_line` always) and **M3** (`header_line =
  self.clause_state.line()`) each kill only
  `a_while_retest_belongs_to_the_do_clause_then_to_the_end_clause`. So the
  sibling really does carry the line property alone, exactly as its doc comment
  says. (What no mutation reaches is the `ITERATE` case -- NEW-1.)
* **Rooting at the boundary is still load-bearing at both shapes.** RAN, with a
  negative control: `sr1`/`sr3` pass unmutated at 9/11 real collections and
  panic under A11.
* **Gates reproduce exactly.** Listed above.

## Pre-existing, found in passing, not this round's

* **`o6`** -- a `CALL ON` handler's `EXIT` beats an already-taken `SIGNAL ON
  SYNTAX` transfer (oracle rc 3, ours rc 7); identical on `26da3ac6`.
* **`p2`/`p5`/`p7`/`p8`** -- byte-identical on `26da3ac6`; recorded under NEW-2
  because they falsify a claim this round makes, not because they are new
  behaviour.
