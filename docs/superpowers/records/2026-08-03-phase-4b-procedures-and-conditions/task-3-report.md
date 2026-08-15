# Task 3 report: the body selector, `CALL`, `RETURN`, and the shared variable pool

Status: **DONE_WITH_CONCERNS**. Everything in the brief is implemented and
verified; three findings below are things the brief could not have known and
that later tasks need. The concerns are recorded in "Concerns and disclosed
limitations" at the end, not hidden in the body.

Baseline confirmed green before any change: `cargo test --workspace` passed,
corpus `32 of 32 matching`, assertion table `4224 of 4259`.

---

## 1. What landed

* `Activation::body: Option<usize>` -- the body selector, `None` = `main`,
  `Some(i)` = `directives[i]`'s, the same shape `BodyKey::directive` carries.
  `activation::body_of` is the single function that turns the pair into a
  `&CodeBody`; `run_activation` reads it instead of hardcoding `&program.main`.
* `Activation::nested` -- the sibling constructor, inheriting `settings` and
  `trace_mode` from the caller.
* `Activation::trace_mode` -- moved off `Interp` (I4), with
  `Interp::trace_mode()`/`set_trace_mode()` as the accessors.
* `Flow::Return(Option<ObjRef>)` and `Ended { Returned, Exited }`.
  `run_activation` now answers `Ended`.
* `run.rs`'s `exec_call`, implementing `Call::Named` and `Call::Dynamic`,
  `RESULT`, the site stack, and the indent base.
* `MAX_ACTIVATION_DEPTH = 10_000`, raising the existing
  `Raised::insufficient_stack()`.
* Corpus witness `rust/corpus/lang/call_return.rex`, in `phase-4b.txt`.

`Call::Trap` (Task 7) and `Call::Qualified` (Phase 5) still fail loudly with
their own owners; their `loud.rs` witness rows are untouched.

---

## 2. Step 1: the combined depth budget, and whether the counter fires first

Bisected with `rexx-run` on the 512 MiB interpreter thread, unbounded `CALL`
recursion, varying the number of `DO` blocks around each activation's own
recursive `CALL`:

```
enclosing DO blocks per activation | deepest surviving (debug) | (release)
----------------------------------+---------------------------+----------
                                0 |                    22,534 |   133,150
                                1 |                    14,062 |    94,518
                                5 |                     5,616 |         -
                               25 |                     1,403 |         -
```

Program shape (`nesting = 1`):

```rexx
n = 0
call sub
exit
sub:
n = n + 1
do
  if n < LIMIT then call sub
end
return
```

Per level that is ~23.8 KB for a bare activation and ~14.4 KB for each further
`run_bounded` level, in debug. The `0` row already contains one `run_bounded`
level, because the recursion is guarded by an `IF`, whose arm runs its branch
through `run_bounded`.

**Answer to the question the step asks: partly.** At the chosen limit of
10,000 the counter fires before the native abort for the first two rows -- the
shapes a recursive routine actually has -- and does **not** for the last two. A
body with about four or more block levels around its own recursive `CALL` still
dies natively. That is not fixable by choosing a different constant: the budget
is shared between activations and `run_bounded` levels, and a counter over one
of them cannot bound the other. This is I34 restated with numbers, and the real
answer is a shared budget or a stack-headroom check, which the brief tells me
not to reopen (D19).

The oracle's own limit is **27,314 activations** (measured by trapping the
condition and printing the counter), reported as `Error 11.1`, `Insufficient
control stack space`, rc 245. That is above *every* debug row above, so
matching it would mean shipping a counter that never fires. 10,000 is chosen
against the debug profile because `cargo test` runs debug and debug is the
binding constraint.

Our report is identical in shape to the oracle's, differing only in the number
of echo lines (10,000 against 27,318) -- both print one `call sub` echo per
activation, then the same two report lines verbatim:

```
Error 11 running PATH line 6:  Control stack full.
Error 11.1:  Insufficient control stack space; cannot continue execution.
```

**And a second, sharper number.** On a default 2 MiB `cargo test` thread this
crate's debug build survives **fewer than 90 activations** (measured: 80
survives, 90 aborts). The recursion test was written as a unit test first and
aborted the whole test binary; it now lives in `tests/spike.rs` and goes
through `run_program`, which spawns the sized thread. Its doc comment says so
at its definition so it cannot be moved back by accident.

---

## 3. Is a `::routine` reachable in 4b? (Task 9's `>I>`/`<I<` decision)

**Yes, for any non-builtin name. Dispatching to one is deferred, not
impossible.** (Corrected in fix round 1 -- section 11 has what the first
version of this section got wrong and why.)

Structurally it is present: `parse_program(b"call foo\nexit\n::routine foo\nsay 1\n")`
gives `directives.len() == 1` with `RoutineDirective { name: "FOO", body: Some(..) }`,
so `Some(i)` is representable and `body_of` resolves it. There is a unit test
(`the_body_selector_resolves_a_routine_directive_and_rejects_a_bad_index`)
exercising exactly that, so the arm is not merely written.

And the oracle dispatches to it:

```rexx
call zorkolo
say 'result=' result
exit
::routine zorkolo
say 'in the routine'
return 'from-routine'
```

Oracle: `in the routine` / `result= from-routine`, rc 0. Ours: rc 120,
`rexx-exec: routine "ZORKOLO" is not implemented (4c)`.

**What is deferred is the resolution step in front of it.** A named call
resolves internal label, then builtin, then external, and the builtin table is
4c's. A name that *collides* with a builtin has to go to the builtin --
measured:

```rexx
call max 1,2               ::routine max ; return 'shadowed'   ->  RESULT = 2
say max(1,2)               ::routine max ; return 'shadowed'   ->  2
```

Dispatching non-builtin names today would ship a rule that is right until
someone names a routine `MAX`, and the failure mode is the bad one: silently
running the wrong routine rather than failing loudly. That is the trade being
deferred. `Activation::body`'s doc, `BodyKey::directive`'s doc and
`exec_call`'s doc all state it this way.

**Three measured differences Task 9 (and whoever sets `Some(i)`) inherits.** A
`::routine` activation is not an internal label's activation with a different
body -- it differs in ways the selector alone does not deliver:

1. **Its own variable pool.** `vv = 'caller'` then `call foo` into
   `::routine foo` prints `VV` (unset), and `ww` set inside does not survive.
   Not D9r's shared pool at all.
2. **`TRACE` does not cross into it.** With `trace r` in the caller, a
   `::routine`'s clauses are **not echoed at all** -- the only trace line is
   the caller's own `>>>` for the returned value:

   ```
        2 *-* call foo
          >>>   "77"
        3 *-* say 'r=' result
          >>>   "r= 77"
        4 *-* exit
   ```

   That is directly relevant to a `>I>`/`<I<` scope decision: whatever scoping
   rule Task 9 picks, a `::routine` boundary is a trace boundary and an
   internal label's is not.
3. **Builtins shadow it**, per the transcripts above.

---

## 4. The two composition shapes nobody had measured

### 4a. `CALL` inside an `INTERPRET` fragment

Program (`ci.rex`), `trace r` on line 1:

```rexx
trace r
interpret "call sub"
exit
sub:
ff = 7
return
```

Oracle stderr, and ours byte-identically:

```
     2 *-* interpret "call sub"
       >>>   "call sub"
     2 *-* call sub
     4 *-*   sub:
     5 *-*   ff = 7
       >>>     "7"
     6 *-*   return
     3 *-* exit
```

**The brief said "this task must not set `clause_line_override`". Running it
shows that is necessary but not sufficient: it must be *cleared*.** The
enclosing `INTERPRET` has already set it to 2, and the callee's clauses carry
their own lines 4, 5 and 6. Leaving the override in force prints all of them as
line 2. `exec_call` does `std::mem::take` on the field and restores it after.
Pinned by `a_call_inside_a_fragment_echoes_each_activations_own_line`.

**A second defect the same shape found.** The first implementation resolved the
label against `code.body`, the body being stepped. Inside a fragment that is the
*fragment's* body, whose `labels` is always empty (a label in interpreted text
is error 47.1), so every `CALL` inside an `INTERPRET` was unresolvable and
failed loudly. Resolution now goes against the running **activation's** body.
Both defects passed every test that had no `INTERPRET` in it.

On the error path (`cie.rex`), three levels:

```rexx
interpret "call sub"
exit
sub:
say 1/0
return
```
```
     4 *-*   say 1/0
     1 *-* call sub
     1 *-* interpret "call sub"
Error 42 running PATH line 4:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
```

rc 214, ours byte-identical.

### 4b. `INTERPRET` inside a called routine

```rexx
trace r
call sub
exit
sub:
interpret "gg = 8"
return
```
```
     2 *-* call sub
     4 *-*   sub:
     5 *-*   interpret "gg = 8"
       >>>     "gg = 8"
     5 *-*   gg = 8
       >>>     "8"
     6 *-*   return
     3 *-* exit
```

The fragment adds no indent of its own on top of the activation's, so all four
of the callee's lines sit at 2 -- consistent with the fragment's measured
delta 0 and the call's +2. Error path (`ice.rex`), three levels again:

```
     4 *-*   say 1/0
     4 *-*   interpret "say 1/0"
     1 *-* call sub
```

Both directions ours byte-identical to the oracle on stdout, stderr and rc.

---

## 5. The indent rule, measured at three caller indents

The brief warned that four agents have got an indent rule wrong here by
measuring one shape. Measured at three, plus two callee depths:

| caller's printed indent | callee's own nesting | callee clause echoes at |
|---|---|---|
| 0 (`call` at top level) | 0 (flat) | 2 |
| 0 | 1 (one `DO`) | 4 |
| 2 (`call` one `DO` deep) | 0 | 4 |
| 4 (`call` two `DO`s deep) | 0 | 6 |
| 4 | 1 | 8 |

Every row reproduces byte-identically. The rule is **the calling clause's
printed indent plus two**, set (not added) into `activation_indent`, with
`indent_offset` zeroed alongside it. `2 x depth` agrees only with row 1 and
predicts 2 where the truth is 4 and 6.

Three-deep, from inside a `DO` (`e2.rex`, error path):

```
     9 *-*       say 1/0
     6 *-*     call two
     2 *-*   call one
```

6, 4, 2 -- one echo per activation, innermost first, each at its own indent.

---

## 6. The measured semantics the brief listed

All three reproduce, and all three are pinned by tests.

* **`RESULT` is settled on return, not at the call.** `result = 'before'`,
  `call sub`, and the callee prints `inside result= before`. After
  `sub: return 42` the caller reads `42`; after a bare `return` it reads the
  derived name `RESULT`.
* **`CALL "SUB"` with `sub:` present** is Error 43.1 rc 213 on the oracle.
  4b's answer is the loud `4c` fallback, as instructed -- the oracle's 43.1 is
  a claim that nothing outside the label table matched either, which this phase
  cannot make. The literal form is not wired into the label table.
* **Omitted arguments.** `call sub 1,,3` parses as `[Some, None, Some]` and
  the oracle gives `[1] [Q] [3]` with `arg()` returning 3. See the concern
  below: this is unobservable in 4b.

Additional measured semantics, all pinned:

* `numeric digits 7` in a caller: the callee sees 7, sets its own 3, the caller
  still reports 7 after the return.
* `trace off` in a callee does not survive its return.
* `RETURN` with a value traces **twice** -- once at the callee's own clause
  indent, once at the caller's.
* `EXIT` in a callee, and a callee running off the end of the file, both end
  the program; the caller's next clause never runs.
* `RETURN` in the **main body** with no active call ends the program with its
  value, exactly like `EXIT`: `return 5` gives rc 5, bare `return` gives rc 0.
  (`loud.rs`'s deleted `Return` witness was `return 1`, which is why it now
  exits 1 rather than 120.)
* `CALL (expr)` searches the label table with the value **verbatim**:
  `nm = 'SUB'` finds `sub:`, `nm = 'sub'` is Error 43.1 `Could not find routine
  "sub"`. It also traces its target as a `>>>`, which `Call::Named` does not.
* A name bound by `INTERPRET` inside a callee is visible to the caller after
  the return, in both directions -- so `Activation::extra` is cloned in and
  moved back out with the frame it names slots in.

---

## 7. Test output

```
cargo test --workspace          879 passed; 0 failed; 4 ignored
corpus                          33 of 33 matching
assertion table                 4224 of 4259 rows passing   (unchanged)
cargo fmt --all --check         exit 0
cargo clippy --workspace --all-targets -- -D warnings   exit 0
```

No pre-existing expected-byte literal changed; verified by grepping the diff
for removed lines containing `*-*`, `>>>`, `.to_vec()` or `assert_eq` (empty).
The only removed test data are the three ownership rows that had to move.

**Mutation testing.** Every new assertion was checked against a broken
implementation; each mutation was applied, the targeted test run, and the file
restored byte-for-byte (verified with `cmp`).

| mutation | test | result |
|---|---|---|
| callee gets a fresh, properly popped slot frame | shares-the-pool | FAILED |
| same | runs-its-own-clauses (no variables) | **passed** |
| same | result-is-settled-on-return | FAILED |
| `activation_indent = 2 * depth` | indent-plus-two | FAILED |
| keep `clause_line_override` instead of clearing | call-inside-a-fragment | FAILED |
| drop the `seal_site_level` call | one-echo-per-activation | FAILED |
| drop the `extra` write-back | interpret-name-survives-return | FAILED |
| falling off the end returns instead of exiting | exit-or-fall-off | FAILED |
| `Settings::default()` instead of inheriting | numeric-inherited | FAILED |
| `TraceMode::OFF` instead of inheriting | trace-does-not-survive | FAILED |
| resolve labels against `code.body` | call-inside-a-fragment | FAILED |

The second row is the brief's own trap, confirmed: **a witness with no
variables in it passes against an implementation that correctly-but-wrongly
isolates every callee**, while the two witnesses with variables fail.

**Cross-checking against the oracle.** 24 probe programs written during this
task were run through both interpreters, reading stdout, stderr and exit status
as three separate descriptors. 23 are byte-identical on all three. The one
exception is a probe using `digits()`, a builtin, which fails loudly at rc 120
as it should.

---

## 8. Ownership tables

Three places, all edited together:

* `tests/owners.rs`: `Return` -> `Owner::InScope`; counts 21/8 -> 22/7;
  `EXPECTED_OUT_OF_SCOPE` loses the `Return` row. **`Call` stays
  `Owner::Phase("4b")`** -- the table is variant-grained and `Call::Trap` is
  still Task 7's.
* `tests/loud.rs`: the `Call::Named`, `Call::Dynamic` and `Return` witness rows
  are deleted; `expand_for_witnesses("Call")` shrinks from four arms to two;
  expected witness count 23 -> 20; in-scope count 21 -> 22. The
  `Call::Qualified` and `Call::Trap` rows are untouched and still fail loudly
  with owners `Phase 5` and `4b`.
* `src/lib.rs`'s `instruction_owner`: `Return` and the `Named`/`Dynamic` arms
  become `None`; `Trap` stays `"4b"`, `Qualified` stays `"Phase 5"`.

`EXPECTED_SUBSET` is untouched -- it pins `phase-4a.txt` alone, and the new
program is in `phase-4b.txt`.

`phase-4b.txt`'s header gained a paragraph stating that its "no variant
carrying an `Owner::Phase`" rule is **per arm** where a variant is split, since
the coarse `Call` tag would otherwise read as forbidding any `CALL` in this
subset.

`phase-4-exclusions.txt`'s KNOWN GAP row on the report's echo stack is closed:
both halves now measured matching. Two corrections were made to the mechanism
list that row carried -- it said "`clause_line_override` for the line", which
is the wrong direction, and it did not mention that label resolution has to go
against the activation's body rather than the stepped one.

---

## 9. Concerns and disclosed limitations

**C1. Arguments are evaluated and then discarded.** `USE ARG` (4b) and `ARG()`
(4c) are both still loud, so nothing in this phase can read an argument. The
observable half is implemented and tested -- each present argument is evaluated
in the caller, in order, before the callee starts, so `call sub 1/0` is Error
42.3 against the `CALL` clause at rc 214 -- and an omitted position is skipped
rather than evaluated. The values are rooted for the clause and dropped. They
are **not** stored on `Activation`: a field nothing reads is a `dead_code`
error under `-D warnings`, and the codebase's standing rule against speculative
scaffolding points the same way. Whoever lands `USE ARG` keeps the `Vec`
instead of discarding it, and needs the `Option`s intact. The `exec_call` loop
says so at the site. **The brief listed omitted-argument semantics as
something this task must reproduce; the part that is unobservable in 4b is not
reproduced, only prepared for.**

**C2. `Interp::depth` could not be added under that name.** The brief's
Interfaces section asks for `Interp::depth: usize`. That field already exists
and is `eval`'s recursion depth. The quantity this task needs is exactly
`self.activations.len()`, which is maintained by the push/pop that has to
happen anyway, so a second field could only disagree with it. There is a
`MAX_ACTIVATION_DEPTH` constant and the check reads `activations.len()`. If a
later task wants a named field, it is a rename, not a redesign -- but it should
know the interface it was promised is spelled differently.

**C3. The counter is decoration for deeply nested bodies**, per section 2.
Sized at 10,000 it protects flat and lightly nested recursion and not a body
with four or more block levels per activation. Recorded in the constant's own
doc and in `phase-4-exclusions.txt` rather than left implicit.

**C4. A test-visible behaviour change.** Moving `trace_mode` onto `Activation`
means `Interp::trace_mode()` requires a live activation. Fourteen unit tests in
`run.rs` set the mode before `run_source` pushed one, and two helpers in
`eval.rs`/`trace.rs` never activated at all; all were reordered (`run.rs` now
has `run_source_traced`/`say_output_traced`). No assertion changed, only the
order of setup. `trace_mode()` is deliberately strict rather than answering
`OFF` on an empty stack, because "no activation" is not a state anything in
production can be in while tracing.

**C5. `Loud::missing_body` is currently unreachable.** `body_of` returning
`None` needs a `Some(index)` selector, which nothing constructs. It is a `Loud`
rather than an `unreachable!` for the reason `Loud::instruction`'s own doc
gives, and `body_of`'s three cases (main, a real routine, out of range) are
unit-tested directly so the *function* is exercised even though this
*branch* of the caller is not.

---

## 10. Files changed

```
docs/superpowers/plans/phase-4-exclusions.txt
rust/corpus/lang/call_return.rex                              (new)
rust/corpus/phase-4b.txt
rust/crates/rexx-exec/src/activation.rs
rust/crates/rexx-exec/src/eval.rs
rust/crates/rexx-exec/src/lib.rs
rust/crates/rexx-exec/src/plan.rs
rust/crates/rexx-exec/src/run.rs
rust/crates/rexx-exec/src/trace.rs
rust/crates/rexx-exec/tests/loud.rs
rust/crates/rexx-exec/tests/owners.rs
rust/crates/rexx-exec/tests/spike.rs
rust/crates/rexx-parse/tests/sourceline_oracle/call_return.txt (new)
```

`src/error.rs` is **not** in that list, though the brief named it. The
activation site is pushed by `run.rs`'s existing `seal_site_level`, which
`exec_call` calls; `Raised::report` already builds the echo stack from
`failure_sites` + `failure_site` and needed no change. Verified by the
three-deep transcripts matching byte for byte.

---

## 11. Fix round 1 -- the `::routine` reachability claim (F1)

Commit: `e68d6a6c`. **Documentation only; no code changed, no behaviour
changed** -- the diff contains no non-comment line, checked by filtering it.

The review's single Important finding: section 3 above said a `::routine` is
**not reachable in 4b**. The measurement behind it was sound and the inference
from it was not.

**What went wrong.** I probed with `::routine max` and observed the builtin
winning. `max` is the one shape where the two competing hypotheses -- "a
`::routine` sits behind the builtin step" and "a `::routine` is unreachable
entirely" -- predict identical bytes, so the probe could not separate them. I
then wrote the stronger of the two.

Worse, I had already measured the falsifying case earlier in the same session
and did not connect it: probes `ro1.rex` and `ro2.rex` both used `::routine
foo`, and both dispatched on the oracle. The evidence against the claim was in
my own transcripts. The error was in reading "our resolution order would have
to answer *is this a builtin* first" as "therefore no `::routine` can be
reached", when that step only bites for names that actually collide.

**Verified during this fix round**, with the reviewer's own probe: `call
zorkolo` into `::routine zorkolo` gives `in the routine` / `result=
from-routine` at rc 0 on the oracle, and rc 120 `routine "ZORKOLO" is not
implemented (4c)` here.

**Restated in all four places** as: reachable for any non-builtin name;
deferred because the builtin resolution step in front of it is 4c's, and a
name colliding with a builtin would otherwise silently run the wrong routine
instead of failing loudly.

* `rust/crates/rexx-exec/src/activation.rs` -- `Activation::body`'s doc, which
  also now records why the `max` probe could not tell the two apart, since
  that is the reusable part.
* `rust/crates/rexx-exec/src/plan.rs` -- `BodyKey::directive`'s doc.
* `rust/crates/rexx-exec/src/run.rs` -- `exec_call`'s doc.
* Section 3 of this report.

The three measured differences a `::routine` activation has from an internal
label's -- its own variable pool, `TRACE` not crossing into it, builtins
shadowing it -- are unchanged and still correct; 4c inherits all three.

**Not implementing `::routine` dispatch in 4b is unchanged as a decision.**
Only the stated reason changes.

Verification after the fix:

```
cargo test --workspace          879 passed; 0 failed; 4 ignored
corpus                          33 of 33 matching
assertion table                 4224 of 4259 rows passing
cargo fmt --all --check         exit 0
cargo clippy --workspace --all-targets -- -D warnings   exit 0
```

The ten Minor findings are deferred to the whole-branch review and were not
touched.
