# Review of Task 7: condition traps, `RAISE`, and `NOVALUE`

Reviewed commit `f906aabc`, parent `7ec66f84`, branch `plan/rust-rewrite`.
Working tree clean at review start and at review end; no tracked file was left
modified. Where a tracked file was mutated to check that a test can fail, the
mutation and its restoration are stated at the finding.

Every finding below is labelled **RAN** or **REASONED**. "RAN" means a command
was executed and its output is quoted or summarised from that execution; the
oracle wrapper was `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib
.../build/bin/rexx FILE )` with stdout, stderr and exit status as three
separate descriptors, from a fresh empty directory created for this review
(`.../scratchpad/rev7`), and `rexx-run` was run from `rust/`.

---

## Verdict 1 -- spec compliance: **substantially met, with one global
constraint violated**

Every one of the brief's nine steps is done and every inherited item (I10,
I11, I13, I14, I16) is paid, including the two stale doc comments the brief
asked to be corrected and the open mid-clause-resumption question, which was
constructed, compared and independently re-confirmed here. `SIGL`'s rule was
measured rather than assumed and is right at every level I probed. The one
requirement not met is a **global** one rather than a step: `RAISE SYNTAX`
with an argument the catalogue does not know answers with a plausible Rexx
condition at a plausible rc (216, 25 or 0) instead of failing loudly outside
157..253 (finding 5). One further requirement -- `Raise` carrying all seven
fields -- is met in shape but not in validation.

## Verdict 2 -- task quality: **good work with one shipped wrong answer and a
narrow test gap; not ready to close as-is**

The engineering is careful and the report is unusually honest: the six
defeat-the-mechanism mutations reproduce exactly as claimed, survivor
included, and the survivor's diagnosis and fix are correct. The
`ClauseState` risk the brief singled out is genuinely closed. Against that:
one **Critical** control-flow defect that no test and no probe in the report
reaches (a pending `CALL ON` condition is dropped, or delivered into the wrong
activation, when the trapping activation's next flow is `RETURN`/`EXIT`); one
stated residual that a single probe turns from "unmeasured" into a divergence
(`active_condition` never cleared, finding 2); two implemented and documented
behaviours with **no test at all** in the entire `rexx-exec` suite, corpus gate
included (finding 3); and an unvalidated `RAISE SYNTAX` argument (finding 5).
Comment accuracy is the weakest dimension: three doc comments state something
false (findings 4, 6, 7), two of them describing exactly the property that
finding 1 breaks. No scope creep; the one large refactor (the test harness) is
justified and net-strengthening. Findings 1, 2 and 5 should be fixed before
this task closes; 3 should gain the two missing tests.

---

## Findings

### CRITICAL 1 -- a pending `CALL ON` condition is dropped, or delivered into an unrelated activation, whenever the trapping activation's next flow is `RETURN` or `EXIT`

**File:** `rust/crates/rexx-exec/src/run.rs:692-696` (the `Flow::Exit` and
`Flow::Return` arms of `run_activation`) against `run.rs:738-742` (the
`pending_trap` clause-boundary check).

**Established: RAN.** Four oracle/`rexx-run` comparisons, all DIFF.

`deliver_pending_trap` is called only at the *bottom* of `run_activation`'s
`while` body. `Flow::Return(value) => return Ok(Ended::Returned(value))` and
`Flow::Exit(value) => return Ok(Ended::Exited(value))` both `return` from
inside the `match` above it, so an activation whose clause resolves to a
`RETURN` never reaches the check. `PendingTrap::depth` is then stale: it names
a depth that this activation has just left.

Two distinct wrong answers follow.

**(a) The condition is silently dropped.** `pa2.rex`:

```rexx
 1  call on user foo name uh
 2  zmark = 'NOMARK'
 3  call aa
 4  say 'end mark=' zmark
 5  exit
 6  aa:
 7  return bb()
 8  bb:
 9  raise user foo return 'BBVAL'
10  uh:
11  zmark = 'HANDLER-AT' sigl
12  return
```

| | stdout | rc |
|---|---|---|
| oracle | `end mark= HANDLER-AT 7` | 0 |
| `rexx-run` | `end mark= NOMARK` | 0 |

The oracle runs the handler at `aa`'s own `return bb()` clause boundary --
i.e. the `RETURN` clause completes, *then* the handler runs, *then* the
activation returns. We never run it at all. `pa3.rex` is the same shape with
`zq = aa()` in place of `call aa` and gives the same split (oracle `after aa
zq= BBVAL mark= HANDLER-AT 7`, ours `... mark= NOMARK`).

**(b) The condition is delivered into a later, unrelated activation that
happens to sit at the same depth.** `pa.rex` adds a `call cc` after `call aa`:

| | stdout |
|---|---|
| oracle | `in cc` / `end mark= HANDLER-AT 8` |
| `rexx-run` | `in cc` / `end mark= HANDLER-AT 11` |

Line 8 is `return bb()` in `aa`; line 11 is the `cc:` label. `pa4.rex` pins
that it really is `cc`'s activation and not a stale line number: with `cc:
procedure` and a `say` inside the handler, the oracle prints `handler zowner=
MAIN-POOL sigl= 8` *before* `in cc`, and we print `handler zowner= MAIN-POOL
sigl= 11` -- our handler runs after `cc`'s first clause, in `cc`'s activation,
because `activations.len()` is 2 again and `trap_for` finds the trap `cc`
inherited.

This is the *same* family of defect that `PendingTrap::depth` was introduced to
fix (report section 3c), one step further out: depth equality is necessary but
not sufficient, because a depth can be re-entered by a different activation.
The report does not mention this shape and no test covers it -- every existing
`CALL ON` test has the trapping activation continue to an ordinary clause.

The narrow fix is to run the check on the `Return`/`Exit` paths as well
(before returning), which is what the oracle's ordering in `pa2` describes.
That alone does not close (b): a pending trap that still cannot be delivered
needs to be discarded rather than left on `Interp` for the next activation at
that depth to pick up. A depth alone does not identify an activation.

**What is *not* wrong, checked so the fix is not aimed at the wrong thing.**
Ordinary nesting and recursion are correct. `pg.rex` (raise two levels down,
trapping activation continues to an ordinary clause), `pk.rex` (trapping
activation's *next* clause is `exit 3`), `pl.rex` (its next clause is a
`SIGNAL`), `pn.rex` (three-deep self-recursion through `rec`), and `pj.rex`
(the handler itself calls a routine raising the same condition) are all
**MATCH**. The depth comparison is therefore right wherever the trapping
activation reaches its own next clause boundary; the defect is confined to the
two `Flow` arms that leave the loop without reaching it. -- RAN.

---

### IMPORTANT 2 -- `active_condition` is never cleared when a `CALL ON` handler returns, and the oracle's answer for that shape is measured, not open

**File:** `rust/crates/rexx-exec/src/run.rs:2480-2537` (`deliver_pending_trap`
sets `self.active_condition` and never clears it) and `run.rs`'s
`exec_raise_propagate` doc comment, which states the residual as unmeasured.

**Established: RAN.** `pc.rex`:

```rexx
 1  call on user foo name uh
 2  call sub
 3  say 'resumed'
 4  raise propagate
 5  say 'not reached'
 6  exit
 7  sub:
 8  raise user foo return 'SVAL'
 9  uh:
10  say 'UH ran'
11  return
```

| | stdout | stderr | rc |
|---|---|---|---|
| oracle | `UH ran` / `resumed` | `4 *-* raise propagate` + `Error 98 running <path> line 4:  Execution error.` + `Error 98.918:  No active condition available for PROPAGATE.` | 158 |
| `rexx-run` | `UH ran` / `resumed` | *(empty)* | 0 |

The report's Concern 1 says "the oracle **may well** answer `98.918`. Nothing
measured pins that shape either way." One probe pins it: the oracle does answer
98.918, and we answer silence at rc 0. The residual is a divergence, not an
open question, and the doc comment on `exec_raise_propagate` that presents it
as unmeasured is now inaccurate.

**The scope is narrower than the concern states, and that matters for the
fix.** `pf.rex` -- a `SIGNAL ON SYNTAX` handler that `SIGNAL`s onward to
another label and *then* does `raise propagate` -- is **MATCH** (both re-raise
the original 42.3, rc 214). So `active_condition` must *not* be cleared when a
`SIGNAL ON` handler runs on; the clearing belongs specifically at the point a
`CALL ON` handler returns, i.e. in `deliver_pending_trap`'s
`Ended::Returned(_)` arm. -- RAN, both directions.

---

### IMPORTANT 3 -- two implemented, documented behaviours have no test at all: deleting either leaves the entire `rexx-exec` suite green, corpus gate included

**Established: RAN.** Eleven further mutations of my own, each applied to a
tracked file, the whole `-p rexx-exec --lib` suite run, then
`git checkout -- <file>`; the two survivors were then re-run against
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec` (every target, including the
STRICT corpus gate). `git status --porcelain` was empty after each restore and
is empty now.

Nine of the eleven were killed, several by tests the report never names:

| mutation | outcome |
|---|---|
| `novalue_check` never gates (always raises on an unset read) | KILLED, 15 tests |
| `novalue_check` never raises | KILLED, 4 tests |
| traps not inherited by a callee (`traps: HashMap::new()`) | KILLED, 3 |
| a fired trap does not set `SIGL` | KILLED, 6 |
| a fired trap does not set `RC` | KILLED, 1 |
| `SIGNAL OFF`/`CALL OFF` is a no-op | KILLED, 2 |
| `RAISE PROPAGATE` does not restore the echo stack | KILLED, 1 |
| the trap label is resolved *after* the site is cleared | KILLED, 1 |
| `ANY` is never consulted as a fallback key | KILLED, 1 |

The two survivors:

**(a) `run.rs:2532-2534` -- putting a `CALL ON` trap back after its handler
returns.** Replacing

```rust
if let Some(trap) = removed {
    self.activation_mut().traps.insert(key, trap);
}
```

with `let _ = removed;` leaves 277/277 lib tests and the corpus gate green.
`deliver_pending_trap`'s own doc comment calls this out as measured -- "put
back afterwards, unlike a `SIGNAL ON` trap, which stays removed". It is a real
behaviour: `pq.rex` (raise, handler returns, raise the same condition again)
prints `UH 2 / mid / UH 4 / end` on both interpreters, so the trap must be
back. Nothing pins it.

**(b) `run.rs`'s `exec_raise_propagate` -- the `Raised::reportable()` guard.**
Deleting

```rust
if !active.raised.reportable() {
    return Ok(Flow::Exit(None));
}
```

also leaves everything green. This one is worse than untested: I ran the
mutated binary against `pr.rex` (`raise propagate` inside a `CALL ON USER`
handler) by accident before rebuilding, and it emits

```
     9 *-*   raise propagate
Error 0:  <no message 0.0 in the catalogue>
```

which is exactly the "a `Raised::condition` escaping untrapped would report
`Error 0`" failure mode that `novalue_check`'s own doc comment says the design
takes care to make unreachable. Correct behaviour (`pr.rex` MATCH: silent, rc
0) rests on one unguarded `if` with no test under it.

---

### IMPORTANT 4 -- `run_source`'s doc comment now states the opposite of what the harness does

**File:** `rust/crates/rexx-exec/src/run.rs:5947-5959` (the doc comment on
`run_source`).

**Established: read the source, then grep-confirmed the two sides.** Labelled
REASONED -- I did not execute anything that discriminates the two paths.

The comment still ends:

> `slots` is an empty map throughout: `read`/`slot_of`'s fallback chain
> answers correctly without the real plan's fast path, the same choice
> `eval.rs`'s own test helpers make.

That was true of the deleted `run_activated`, which built `Code { ..., slots:
&HashMap::new() }`. The replacement calls `Interp::run_activation`, which
builds `Code { ..., slots: &plan.by_symbol }` (`run.rs:602-606`), and
`Plan::assign` populates `by_symbol` (`plan.rs:507`). So every test in this
module now runs through the plan's fast path and the sentence describing the
opposite was left in place. The first sentence of the same comment ("a
miniature `run_activation`, through `run_bounded` rather than a hand-rolled
`for` loop") is false for the same reason.

`grep -n "slots: &"` confirms `eval.rs`, `stem.rs` and `plan.rs` still use
`&HashMap::new()`, so the by-name fallback keeps expression-level coverage;
what it loses is whole-program coverage. The coverage *shift* is defensible and
arguably an improvement -- the tests now exercise what production runs -- but
per `rust/CLAUDE.md` a comment that states something false must be corrected or
removed, and this one was not touched.

**The harness change itself is sound and in two places strengthens a test.** I
checked the three behavioural differences between the deleted loop and
`run_activation`, and none weakens anything:

* `Ended::value()` maps both `Returned` and `Exited` to the same
  `Option<ObjRef>` the old arms returned; falling off the end is
  `Exited(None)`, identical to the old `Flow::Next => Ok(None)`.
* `Flow::Signal(target) => pc = target` is exactly the old `start = target`
  resume.
* `run_bounded` never ran the `procedure_permitted` /
  `first_instruction_pending` hand-off (`run.rs:624-627`); it lives only in
  `run_activation`'s loop. So the first two rows of
  `a_procedure_that_is_not_a_calls_first_instruction_raises_17_1` -- the two
  top-level programs -- previously passed because the permission was *never
  granted at all*, and now pass because the `PROCEDURE` genuinely is not the
  first instruction executed. Same assertion, better reason. -- REASONED
  (source reading), against the 277/277 lib run.

---

### IMPORTANT 5 -- `RAISE SYNTAX`'s number argument is not validated, and three of the five wrong shapes are undocumented; one exits 0

**File:** `rust/crates/rexx-exec/src/run.rs`, `split_error_number` and its
caller in `exec_raise` (`raise.condition == b"SYNTAX"` branch).

**Established: RAN.** Eight `raise syntax <arg>` programs, oracle against
`rexx-run`.

| program | oracle | `rexx-run` | |
|---|---|---|---|
| `raise syntax 40.4` / `40.5` / `40.912` / `40` / `98.941` | catalogue report | identical | MATCH |
| `raise syntax 40.10` | `Error 98.918`-family: `Error 98.941: Unknown error number specified on RAISE SYNTAX; found "40010".` rc **158** | `Error 40.10:  <no message 40.10 in the catalogue>` rc **216** | DIFF |
| `raise syntax 40.999` | same 98.941, `found "40999"`, rc 158 | `<no message 40.999 in the catalogue>` rc 216 | DIFF |
| `raise syntax 999` | `Error 33.904: Incorrect expression result following SYNTAX keyword of RAISE instruction.` rc **223** | `Error 999 ...:  The RXSUBCOM parameters are incorrect.` rc **25** | DIFF |
| `raise syntax 'abc'` | 33.904, rc 223 | `Error 0:  <no message 0.0 in the catalogue>` rc **0** | DIFF |
| `raise syntax 0` | 33.904, rc 223 | `Error 0:  <no message 0.0 in the catalogue>` rc **0** | DIFF |

Three separable problems, all newly reachable because `RAISE` is the first
construct that lets a program name an arbitrary error number -- the code's own
comment says as much ("Reachable only through `RAISE`, which is why 4a never
had to know").

1. **A well-formed number with no catalogue entry** (`40.10`, `40.999`) is
   98.941 on the oracle. We render the internal placeholder string `<no
   message N.M in the catalogue>` straight to the user's stderr. Undocumented.
2. **A non-numeric or zero argument** is 33.904 rc 223. We produce `Error 0`
   at **rc 0** -- a report on stderr and a *successful* exit status.
   `split_error_number`'s doc says `0` "keeps that case a visible wrong answer
   rather than a panic"; rc 0 is not a visible wrong answer, it is a silent
   pass, and it is the same `Error 0` failure mode that finding 3(b) reaches
   by another route.
3. **A major with no `.sub`, out of range** (`999`) renders an unrelated
   catalogue entry and exits **25**. Undocumented.

Against the plan's global constraint -- "anything Phase 4b does not implement
fails loudly via `NOT_IMPLEMENTED_EXIT`, outside 157..253, never a plausible
Rexx condition" -- all three answer with plausible Rexx conditions at plausible
rcs (216, 25, 0) instead of failing loudly.

---

### MINOR 6 -- `Delivery`'s doc comment names one writer where there are three

**File:** `rust/crates/rexx-exec/src/error.rs`, the doc on `struct Delivery`:
"Only `run.rs`'s own `exec_raise` ever sets any of them."

**Established: RAN** (`grep -n "delivery.search = \|delivery.positionless = "`).
Seven assignments in three functions: `offer_to_trap` (`run.rs:2403`),
`exec_raise` (`:2717, :2730, :2769, :2778`) and `exec_raise_propagate`
(`:2819, :2820`). `offer_to_trap`'s is the load-bearing `Search::Caller ->
Search::Here` rewrite, so this is not a trivial omission -- it is the one
writer a reader most needs to know about.

### MINOR 7 -- `Interp::pending_trap`'s doc states the guarantee that finding 1 breaks

**File:** `rust/crates/rexx-exec/src/lib.rs`, the doc on `pending_trap`: "the
hazard would be a nested activation that failed to deliver it, and
`run_activation`'s check runs in every activation."

**Established: RAN** (finding 1's probes) **and reasoned** (`run.rs:692-696`).
The check runs in every activation but not on every path *out* of one. The
comment identifies exactly the right hazard and then asserts it is closed; it
is not. It should be corrected with the fix rather than separately.

### MINOR 8 -- `run_activated`'s `_program` parameter is dead

**File:** `rust/crates/rexx-exec/src/run.rs:5983`. `fn run_activated(interp:
&mut Interp, _program: &Program)` ignores its second argument; both call sites
(`run_source`, `run_source_traced`) still compute and pass it. Underscored, so
clippy is silent. Removing it and the two arguments is a three-line change that
would have made the "there was never anything for the copy to supply" claim
structural instead of stated. -- REASONED (source reading).

### MINOR 9 -- `Interp::trap_for` is `pub(crate)` with no cross-module caller

**File:** `rust/crates/rexx-exec/src/run.rs`, `pub(crate) fn trap_for`.
`grep` shows every call is inside `run.rs` (`novalue_check`, `offer_to_trap`,
`deliver_pending_trap`). `novalue_check` beside it genuinely needs
`pub(crate)` (called from `eval.rs:275, :309`); this one does not. -- RAN
(grep).

---

## What I checked and found correct

Recorded because several of these were named as risks, and a clean result is
part of the answer.

* **The six defeat-the-mechanism mutations reproduce exactly as reported.**
  -- RAN. I re-derived all six independently (apply, run the named test,
  `git checkout --`), including the reported survivor: `M4` (leave the fired
  trap in the table) is KILLED by `the_trap_that_fired_is_removed_from_the_
  table` and SURVIVED by `a_trap_can_be_re_armed_inside_its_own_handler`,
  precisely as report section 7 states. The accounting is honest and the
  survivor was correctly diagnosed and closed.

* **`PendingTrap` depth delivery is right everywhere the trapping activation
  reaches its own next clause.** -- RAN: `pg`, `pk`, `pl`, `pn` (three-deep
  self-recursion), `pj` (the handler itself raises the same condition again),
  all MATCH. Risk area 4 is closed except for the `Return`/`Exit` hole in
  finding 1.

* **`Search::Caller` searching exactly one level out is measured-correct, not
  a residual.** -- RAN. Report Concern 2 says "if the *grandparent* has a trap
  and the parent does not, we ignore the condition where the oracle might
  propagate to it". `pb.rex` builds precisely that (`signal on user foo` in
  main, `signal off user foo` in `lev1`, `raise user foo return` in `lev2`):
  **MATCH** -- the oracle does not propagate to the grandparent either. Concern
  2 should be downgraded from a residual to a measured behaviour; the code is
  right.

* **`ClauseState` needed no new member, and the trap-resumes-mid-clause route
  really is covered by the existing restore.** -- RAN (mutation M3 kills
  `a_trap_that_resumes_mid_clause_leaves_the_enclosing_clauses_state_intact`)
  and RAN (`pn`, `pg`). No newly introduced `Interp` field has the
  cached-per-clause shape: `pending_trap` is consumed rather than refreshed
  per clause, and `active_condition` is written only when a trap fires. Risk
  area 1 is closed.

* **`ANY` as a fallback key, in both directions.** -- RAN. `pd.rex` (`signal
  on any` traps a `NOVALUE`) and `pe.rex` (`call on any` does *not*) both
  MATCH, as does `v5.rex` (`call on any` declines a `SYNTAX`, ordinary fatal
  42.3 at rc 214).

* **`INTERPRET` interaction.** -- RAN. `pm.rex` (`interpret "raise syntax
  40.4"` inside a routine, `Search::Top` skipping the middle level to reach
  the main body's trap) MATCH.

* **The corpus witness matches the oracle byte for byte right now.** -- RAN.
  I copied `rust/corpus/lang/condition_traps.rex` into the probe directory and
  compared independently of the corpus harness: stdout, stderr and rc 214 all
  identical. Its vacuity design is sound -- the accumulated `ZWITNESS` string
  loses a segment if a block silently does not run, and every segment embeds
  its own `SIGL`.

* **Ownership movements are internally consistent.** -- RAN (the suite) and
  reasoned (arithmetic). `INSTRUCTION_TAGS` 40 = 26 in scope + 2 (`4b`) + 4
  (`4c`) + 7 (`Phase 5`) + 1 (`Phase 7`); witness tags 14 = 2+4+7+1 with
  `Call` expanding to `Call::Qualified` alone. `Call`'s sideways move to
  `Phase 5` is right, and `instruction_owner`, `EXPECTED_OUT_OF_SCOPE` and
  `INSTRUCTION_WITNESSES` agree. Marking `Raise` in scope is defensible: the
  one remaining gap, `RAISE ... ADDITIONAL (a, b)`, fails loudly as
  `rexx-exec: a parenthesised list is not implemented (Phase 5)` at **rc 120**
  -- outside 157..253 as the constraint requires -- while `ARRAY (a, b)`
  works. -- RAN.

* **Report Concern 4 is real, correctly attributed, and not this task's.** --
  RAN. `trace r` / `say 'a'` / `exit 0` reproduces the missing `>>>   "0"`.
  The diff touches nothing in `InstructionKind::Exit`'s execution arm (its only
  occurrence in the diff is the `owners.rs` table row), so it is pre-existing.

* **Gates.** -- RAN, from `rust/`: `cargo fmt --all --check` exit 0;
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0;
  `cargo test --workspace` all green; `cargo test -p rexx-exec --lib` 277
  passed / 0 failed; `REXX_CORPUS_GATE=1 cargo test -p rexx-exec` exit 0.

* **Inherited items.** I10 (`#[expect(dead_code)]` deleted, field read by
  `offer_to_trap`/`trap_for`), I11 (`failure_site` cleared, mutation-verified),
  I13 (`Novalue::Unset` read by `novalue_check`), I14 (`+++` correctly absent),
  the two stale `is_none()` doc comments at `record_failure_site` and
  `seal_site_level` corrected -- all present in the diff and all verified.

* **I16, the temps-frame re-verification.** -- RAN. Mutation M6 (retain one
  root per trapped cycle) kills
  `a_trap_that_resumes_does_not_accumulate_temps_frames`, so the test measures
  what it claims. The program raises from inside a parenthesised expression,
  which is the shape the chokepoint claim is about. The brief's "a program, not
  'it still holds'" requirement is met.

---

## Spec compliance

| brief requirement | verdict |
|---|---|
| Step 1: measure trap transcripts, stdout/stderr/rc separately | **met** -- 85 transcripts in Appendix A, wrapper and separate descriptors as required; I re-ran seventeen equivalents independently |
| Step 2: failing tests asserting handler-set values, not exit codes | **met** -- every new trap test asserts a handler-set value; no test asserts only an exit code |
| Step 3: trap table on `Activation`, dispatch from measurement | **met** -- `Activation::traps`, inherited by copy, never written back; inheritance direction measured and re-verified here (`pb`, and lib tests killed by mutation E3) |
| Step 4: `SIGNAL ON`/`OFF`, `CALL ON`/`OFF` | **met**, with the delivery hole in finding 1 |
| Step 5: `Raise` with all its fields and `RaiseResult` | **met** for the seven fields; **not met** for argument validation (finding 5) |
| Step 6: wire `Novalue::Unset`; delete the `#[expect(dead_code)]` | **met** |
| Step 7: clear `failure_site` on trap resumption, two-raise test | **met**, mutation-verified |
| Step 8: re-verify the temps-frame conclusion with a program | **met** |
| Step 9: suite, corpus gate, `owners.rs`, commit | **met** |
| I10 / I11 / I13 / I14 / I16 | **met** |
| `set_sigl`'s third caller, with `SIGL` measured not assumed | **met** -- the rule ("the clause line of the activation in which the transfer happens") is right at every level I probed |
| The two stale `failure_site` doc comments corrected | **met** |
| Brief's open item: construct the trap-resumes-mid-clause shape and compare | **met** -- constructed, compared, no `ClauseState` defect; independently re-confirmed here |
| Global: no `unsafe`; every new allocation through `Interp::alloc_with` | **cannot verify from diff** for `alloc_with` -- the new code allocates only through existing `self.text(...)`, which I did not trace to its allocator; `unsafe` is forbidden workspace-wide and clippy is clean |
| Global: unimplemented things fail loudly outside 157..253 | **not met** in the `RAISE SYNTAX` argument family (finding 5): rc 216, 25 and 0 |
| Global: value rendering fixed at creation | **cannot verify from diff** -- no new numeric rendering site was added; nothing in the diff reads `settings.digits()`/`form()` |

## Cannot verify from diff

* Whether every new allocation site goes through `Interp::alloc_with` rather
  than `Heap::alloc_with_uncollected`/`Heap::alloc`. The new code allocates
  only via `self.text(...)`, and I did not follow that to its allocator.
* Appendix A's 85 transcripts as a set. I re-ran seventeen equivalent probes
  of my own and the corpus witness rather than replaying the appendix; the
  report's own claim of "78 of 85 match" is not independently checked here.
* The report's claim that eleven trap tests went *red* rather than green
  against the old harness. The old harness is deleted, so this is not
  reproducible from the diff; the argument for it is sound but unverified.
