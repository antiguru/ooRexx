# Task 7: condition traps, `RAISE`, and `NOVALUE`

Status: **DONE_WITH_CONCERNS**. Commit `f906aabc` on `plan/rust-rewrite`,
parent `7ec66f84`.

Everything in the brief is implemented and measured. The concerns are three
stated residuals, one pre-existing divergence found in passing that is not
mine, and one design decision (`RAISE PROPAGATE`) that rests on four
transcripts rather than on a family of them. All are in "Concerns" at the end.

---

## 0. Baseline, confirmed before anything changed

At `7ec66f84`, from `rust/`:

| gate | result |
|---|---|
| `cargo test --workspace` | 955 passed, 0 failed |
| `cargo test -p rexx-exec --test corpus` (REPORT) | `37 of 37 matching` |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` (STRICT) | `mode: STRICT (the gate)`, `37 of 37 matching` |
| `cargo test -p rexx-exec --test assertions` | `4224 of 4259 rows passing` |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |

---

## 1. Where the probes ran

Two directories, both created by `mkdir` for this task, both outside the
repository:

* **Primary**: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe`
* **Clean re-verification**: `.../scratchpad/clean`

Every oracle run was wrapped exactly as `rust/CLAUDE.md` requires, with
stdout, stderr and the exit status captured as three separate descriptors:

```bash
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx "$F" ) >"$D/$1.out" 2>"$D/$1.err"
```

**A hazard that turned up mid-task and was closed rather than assumed away.**
The primary directory turned out to be shared with a sibling agent in the same
session, which had written its own `d*.rex`, `nl*.rex` and `t25.rex` into it
while this task was running. That is precisely the stale-`.rex` shape
`CLAUDE.md`'s probe rule warns about: the scratchpad is on the oracle's
external-routine search path, so a probe calling an unresolved name finds one
and runs it. None of this task's probes call an unresolved name -- every `CALL`
and every function call in them resolves to an internal label in the same file
-- but "should not have mattered" is not a measurement. So the seventeen
probes every conclusion below actually rests on were copied to the fresh,
otherwise-empty `clean` directory and re-run there against both interpreters:

```
MATCH p21  MATCH p22  MATCH p24  MATCH p25  MATCH p26  MATCH p34  MATCH p36
MATCH p41  MATCH q2   MATCH q6   MATCH r2   MATCH s1   MATCH t1   MATCH t2
MATCH v1   MATCH v3   MATCH ct
```

Same bytes, same exit codes. The shared directory did not influence anything.

**Appendix A** at the end carries all 85 probes verbatim -- source with line
numbers, oracle stdout, oracle stderr -- so nothing below has to be taken on
trust.

---

## 2. What `SIGL` is set to on a trap, and how it compares to `SIGNAL`

**One rule covers both, and it is not the rule the brief warned might be
assumed.**

> `SIGL` is the current clause line **of the activation in which the transfer
> happens**.

For `SIGNAL label` that is the `SIGNAL` clause itself, which is what Task 6
implemented. For a trap it is the clause of whichever activation's trap table
matched -- and that is *not* always the raising clause, because the trap that
fires is not always in the activation that raised.

The four transcripts that pin it, all with the same `say 1/0`:

| probe | where the raise is | where the trap is enabled | handler sees `SIGL` |
|---|---|---|---|
| `p01` | main, line 3 | main | `3` -- the raising clause |
| `p21` | `sub: procedure`, line 9 | main (inherited into `sub`) | `9` -- the callee's own line |
| `p24` | `fun`, line 8, after `signal off syntax` in `fun` | main only | `3` -- the caller's `call fun` clause |
| `p09` | inside `interpret "say 1/0"` on line 3 | main | `3` -- the enclosing `INTERPRET` clause, not the fragment |

`p21` is the one that separates "the trap is inherited and fires in the
callee" from "the condition unwinds and is caught in the caller": both predict
a trapped program, and only the pool tells them apart. `sub` is `PROCEDURE`d,
so its `zowner` is isolated, and the handler printed `owner= FUN-POOL sigl= 9`
-- the callee's.

`p24` is its complement: with `signal off syntax` as the callee's first clause
the same raise is caught in main, `owner= CALLER-POOL sigl= 3`. Traps are
therefore inherited **by copy**, not shared.

**How this maps onto the implementation, and why it needed no new state.**
`offer_to_trap` runs in `run_activation`'s own loop, once per activation, on
the `Err` path. `resolve_and_run_call` has already restored the caller's
`ClauseState` by the time the caller's loop sees a failure that escaped a
callee, so `set_sigl(self.clause_state.current_clause_line)` reads the right
line at every level with nothing threaded through. `p09`'s answer falls out of
`clause_line_override`, which Task 2 already had in force for a fragment.

`SIGNAL`'s own `set_sigl` call is unchanged; the trap is its third caller, as
the brief predicted.

---

## 3. The trap-resuming-mid-clause measurement, and its verdict

**The verdict: the shape is real, it is constructible, and our implementation
matches the oracle on it. No new defect of the Task 4 / Task 6 kind exists.**

### 3a. Constructing it

A `CALL ON` handler cannot produce it, and that is itself a measurement rather
than an assumption -- `p22`, `p25` and `q5` all show a `CALL ON` handler
running only after the raising clause has completely finished. `p25` is the
sharpest:

```
  1  call on user foo name uh
  2  zres = one(1)
  3  say 'zres=' zres
  4  exit 0
  5  one:
  6  raise user foo return 'ONEVAL'
  7  uh:
  8  zres = 'HANDLERVAL'
  9  return
```

prints `zres= HANDLERVAL`. The assignment on line 2 had already *stored*
`ONEVAL` before the handler ran and overwrote it. A handler that fired at the
raise would leave `ONEVAL`.

What does produce it is a **`SIGNAL ON` trap whose handler returns**. `p26`:

```
  1  signal on syntax
  2  zz = one(1) two(2)
  3  say 'zz=' zz
  4  exit 0
  5  one:
  6  say 1/0
  7  return 'ONEVAL'
  8  two:
  9  return 'SIGLIS' sigl
 10  syntax:
 11  return 'FROMHANDLER'
```

`one` raises on line 6; the trap `one` inherited from main fires *inside
`one`'s activation* and transfers to `syntax:`; that handler's `return
'FROMHANDLER'` returns from `one`; evaluation of the clause on line 2 resumes
and calls `two`. **Two activations inside one clause, with a trap and a
control transfer between them.** `two` reports `SIGL`, which is the exact
quantity Task 4 and Task 6 each shipped a defect on.

### 3b. The comparison

| | `zz` |
|---|---|
| oracle | `FROMHANDLER SIGLIS 2` |
| `rexx-run` | `FROMHANDLER SIGLIS 2` |

`SIGL` is `2`, the enclosing clause's line. A leak would have shown `6`
(`one`'s raise), `10`/`11` (the handler's) or `9` (`two`'s label).

**Why it is already right, stated so the next task can check the claim rather
than inherit it.** The route is covered by the mechanism the brief describes,
with nothing added: the trap transfers *within* `one`'s activation, so
`resolve_and_run_call`'s existing save/restore of `ClauseState` as one struct
copy is what puts the enclosing clause's line back when `one` returns. Task 7
added no per-clause state, so `ClauseState` gained no field.

The measurement is pinned by
`a_trap_that_resumes_mid_clause_leaves_the_enclosing_clauses_state_intact`,
and deleting `self.clause_state = saved_clause_state;` from
`resolve_and_run_call` makes it fail -- verified, section 7.

### 3c. One thing this route *did* find

Not `ClauseState`, but the same family of bug one level over. The first
implementation delivered a pending `CALL ON` condition at the next clause
boundary reached by *any* activation. In `zz = one(1) two(2)` that is `two`'s
first clause, so the handler ran inside `two` and reported `SIGL 8` against
the oracle's `2`, and printed before the `SAY` rather than after. `PendingTrap`
carries the depth of the activation whose table matched, and delivery waits
for that depth. Pinned by `a_call_trap_waits_for_the_raising_clause_to_finish`.

---

## 4. The temps-frame re-verification (inherited item I16)

**The 4a conclusion still holds, and here is the program.**

The conclusion rested on `step_in_temps_frame` being the single chokepoint
that heals the six `?`-skipped `pop_frame` sites in `eval.rs`. Reading the
code, that chokepoint is intact: `step_in_temps_frame` pops its frame
unconditionally, *before* returning the `Err`, and `offer_to_trap` runs after
that in `run_activation`'s loop. Task 1 did not move execution off it.

Reading is not measuring, so here is the measurement. `run.rs`'s
`a_trap_that_resumes_does_not_accumulate_temps_frames` runs this program
(shown at `cycles = 200`) and again at `cycles = 400`:

```rexx
signal on novalue name h
zcount = 0
top:
zcount = zcount + 1
say ((zprobe))
h:
signal on novalue name h
if zcount < 200 then signal top
say 'done' zcount
```

Each pass raises a `NOVALUE` from **inside a parenthesised expression**, which
is what puts `eval`'s own frame-opening sites on the path -- a raise from a
bare clause would exercise nothing the claim is about. The trap fires, control
transfers, the handler re-arms and signals back. The test asserts the program
really did trap that many times (`done 200` / `done 400`) and then compares
`interp.roots.temps_len()` between the two runs.

**Result: equal.** A single retained root per trapped-and-resumed cycle would
make the four-hundred-cycle number exactly two hundred larger.

**And the test can fail**: adding `let leaked = self.text(b"leak");
self.roots.push_temp(leaked);` to `offer_to_trap` -- which is outside any
`step_in_temps_frame` -- turns it red. See section 7.

The residual, unchanged from 4a: this measures the *steady state* of a
resuming trap, not the peak. A single `DO` loop still accumulates its temps
for the loop's whole run (`step_in_temps_frame`'s own doc comment says so),
and a trap inside one inherits that; nothing here changes it either way.

---

## 5. Which conditions are implemented, which are loud, and who owns what

### 5a. The trap table

`SIGNAL ON`/`OFF` and `CALL ON`/`OFF` are implemented for **every** condition
name the parser accepts, including `ANY`. Registering a trap for a condition
nothing in this phase raises is correct and free.

`ANY` is a real fallback key, not decoration: `u1` shows `signal on any`
trapping a plain `say 1/0` with `SIGL` set exactly as `signal on syntax`
would. `v1` shows the complement -- `call on any name uh` does **not** catch
the same condition, because a `CALL` trap resumes and a failed clause has
nowhere to resume into. That pair is why `offer_to_trap` declines a `call`
trap rather than asserting it cannot happen.

### 5b. Conditions this crate can actually raise

| condition | raised by | trapped | untrapped default |
|---|---|---|---|
| `SYNTAX` | every existing raiser, and `RAISE SYNTAX n.m` | yes | fatal report, `256 - major` |
| `NOVALUE` | an unset simple variable or compound read | yes | the derived name, unchanged |
| `HALT` | `RAISE HALT` only | yes, via `RAISE HALT ... RETURN` | fatal `Error 4.1`, rc 252 |
| `USER x`, `ERROR`, `FAILURE`, `NOTREADY`, `NOSTRING`, `LOSTDIGITS`, `NOMETHOD` | `RAISE` only | yes | ignored |

`NOVALUE` fires for a simple variable (`p03`) and for a compound (`u2`) and
**not** for a bare stem (`v3`: `say zunsetstem8.` prints `ZUNSETSTEM8.` and the
program carries on). That third row is not guessable and is why
`Interp::read_stem` deliberately has no `Novalue` answer while `Interp::read`
and `Interp::stem_get` both do.

`ERROR` and `FAILURE` are *registrable and raisable* but nothing in this phase
raises them on its own: they need command clauses, which are Phase 7's under
D18. `NOTREADY` needs I/O.

### 5c. What is still loud, with owners

| shape | message | owner |
|---|---|---|
| `CALL ns:name` | `CALL is not implemented (Phase 5)` | Phase 5 -- `Call::Qualified`, unchanged |
| `RAISE ... ADDITIONAL (a, b)` | `a parenthesised list is not implemented (Phase 5)` | Phase 5 -- `ExprKind::List` |
| `condition('C')` and friends | `routine "CONDITION" is not implemented (4c)` | 4c -- the builtin table |

The `ADDITIONAL (a, b)` case is worth being explicit about, because it is the
only `RAISE` shape that still fails. `ADDITIONAL` takes an array *object*, and
`(a, b)` parses as `ExprKind::List`, which is Phase 5's. `RAISE ... ARRAY (a,
b)` reaches byte-identical oracle output (`p42` against `p43`) and is
implemented, so nothing is unreachable -- only one spelling of it is. That is
why `InstructionKind::Raise` is `Owner::InScope` rather than arm-grained: the
gap belongs to the expression, is reported against the expression, and naming
`RAISE` as its owner would be false.

### 5d. Owner-table movements

| variant | before | after | why |
|---|---|---|---|
| `InstructionKind::Signal` | `Phase("4b")` | `InScope` | all three arms implemented |
| `InstructionKind::Raise` | `Phase("4b")` | `InScope` | see 5c |
| `InstructionKind::Call` | `Phase("4b")` | `Phase("Phase 5")` | **sideways**: `Call::Trap` moved in scope, leaving `Call::Qualified` alone, and that is Phase 5's. Leaving "4b" would have named a phase that owes nothing. |

Counts, re-derived: `INSTRUCTION_TAGS` 40 = 26 in scope + 2 (`4b`: `Push`,
`Queue`) + 4 (`4c`) + 7 (`Phase 5`) + 1 (`Phase 7`). Witness rows: 14 expected
instruction tags, `Call` expanding to `Call::Qualified` alone. `EXPECTED_
OUT_OF_SCOPE`, the four counts in `owners.rs`, the two in `loud.rs`, and the
`expand_for_witnesses` expansion were all updated together, and the three
copies of the ownership data (`owners.rs`, `loud.rs`, `lib.rs`'s
`instruction_owner`) agree -- `every_out_of_scope_variant_fails_loudly` is
what checks the third against the emitted stderr.

### 5e. The corpus witness

`rust/corpus/lang/condition_traps.rex`, added to `rust/corpus/phase-4b.txt`.
93 lines, under `trace r` throughout, matching the oracle byte for byte on
stdout, stderr and rc 214.

It witnesses `InstructionKind::Raise` (required, or
`every_in_scope_variant_is_witnessed_by_the_phase_subsets` fails) and also
`Signal::Trap` and `Call::Trap`. `EXPECTED_SUBSET` still pins `phase-4a.txt`
alone and is untouched.

**On the vacuity hazard the brief named.** The program accumulates one segment
per block into a single `ZWITNESS` string and prints the whole thing, so a
block that silently did not run *removes its segment* rather than leaving the
output unchanged. Every segment (`START`, `/SYNTAX-AT-49`, `/RAISED-AT-55`,
`/NOVALUE-AT-61`, `/TAIL-AT-67`, `/USER-CALLED-AT-73`, `/RAISER-RETURNED`) is
a value a handler set, and none of them is any variable's derived name or a
prefix of one -- a failure prints `ZWITNESS`, which reads as nothing. Each
segment embeds its own `SIGL`, so a trap that fires with the wrong line is a
different string rather than the same one. The program's own header states,
per block, which wrong answer that block would print.

It contains **no `EXIT <value>`**, deliberately: see Concern 4.

---

## 6. `RAISE`, whose delivery rules are the substance of this task

Nothing in `RAISE`'s grammar says its tail decides who may trap it, and that
is what it does. **A two-level program gives identical bytes for three of the
four rows**, which is why the first version of this table was wrong.

| spelling | trap search starts at |
|---|---|
| `RAISE SYNTAX n.m RETURN [e]` | the raising activation, outward |
| `RAISE SYNTAX n.m` / `... EXIT [e]` | the **outermost** activation only |
| `RAISE <other> ... RETURN [e]` | the raising activation's **caller**, outward |
| `RAISE <other>` / `... EXIT [e]` | **nobody**; the default action applies |

The transcripts that force each row apart:

* **Row 2 against row 1.** `q6`: `raise syntax 40.4` in `lev2`, with `signal
  on syntax name mid` in `lev1` and nowhere else -- **not trapped**, the
  ordinary fatal report at rc 216, `mid` never runs. `q2`: add `signal on
  syntax name outer` to the main body and `outer` fires, `SIGL 3`, `mid` still
  untouched. `q8`: the same three-level shape with an *ordinary* `say 1/0`
  instead traps in `lev2` with `SIGL 10`. So the `RAISE` search is not the
  ordinary search.
* **Row 3 against row 1.** `p30`: `raise syntax 40.4 return 'RETVAL-88'` in
  `fun` reports `SIGL 7`, `fun`'s own raise line. `r2`: the identical program
  with `raise user foo return 'RETVAL'` reports `SIGL 3`, the caller's clause.
  The two differ in nothing but the condition.
* **Row 4 against row 2.** `p48`: `signal on halt` on the line immediately
  above `raise halt` does **not** fire -- fatal `Error 4.1`, rc 252 -- where
  `p11`'s `signal on syntax` above `raise syntax 40.4` traps. At top level
  `Search::Top` would have offered it to exactly that trap, so row 4 needs its
  own variant (`Search::Nobody`).

`error.rs`'s `Search` enum is that table, one variant per row, each carrying
its transcript in its doc comment.

### 6b. The rest of `RAISE`, all measured

* **Untrapped defaults.** `HALT` reports (`p47`, rc 252); `USER`, `ERROR` and
  friends are silent at rc 0 (`p14`, `p45`). `Raised::reportable` is that
  split, spelled on `number != 0` because the condition-name space is
  open-ended and the numbering is not.
* **`RC`.** Three rows, no two sharing a rule: the *major* for any trapped
  `SYNTAX` however it arose (`v4`: `42` for `say 1/0`; `v2`: `40`, not `40.4`,
  for `raise syntax 40.4`); the raise's own argument for `ERROR`/`FAILURE`
  (`s1`: `5`); untouched for `NOVALUE` (`v5`: reads back as `RC`).
* **Sub `0` prints no second line.** `w1`: `raise syntax 40` gives the major
  line and stops, where `raise syntax 40.4` gives both. Reachable only through
  `RAISE`, which is why 4a never had to know.
* **`ADDITIONAL` and `ARRAY` are one substitution list.** `p42`/`p43` are
  byte-identical; `w2` shows a single non-array `ADDITIONAL 'JUSTONE'` filling
  `&1` and leaving `&2` as the literal `&2`.
* **`DESCRIPTION` is evaluated and discarded.** `p44`'s report is
  byte-identical with and without it; it is observable only through
  `condition('D')`, a 4c builtin.
* **The `>K>` trace lines**, in source order at the clause's own indent
  (`y1`, `y2`, `ct`): the condition's own name as the keyword *only for the
  three conditions that take a value* (`>K> "SYNTAX" => "40.4"`; `raise user
  marker` traces no such line), then `DESCRIPTION`, then `ADDITIONAL`/`ARRAY`
  (`>K> "ARRAY" => "an Array"`, the Array class's own default string form,
  emitted as a constant since this crate has no array object), then `RESULT`
  for either tail (`w3` shows `EXIT` tracing it exactly as `RETURN` does).
* **`RAISE PROPAGATE`.** Re-raises the condition whose handler is running,
  past every enclosing trap, at both depths measured (`p27`, `p41`), with the
  major line missing its ` running <path> line <n>` span. With no handler
  running it is `98.918` at rc 158 (`p49`). From inside a `CALL ON` handler
  whose condition has no report to give, the program ends silently at rc 0
  (`s2`). All four reproduced. See Concern 3.

### 6c. `INTERPRET`

`p09` (a raise inside a fragment, trapped outside) and `p10` (`interpret
"signal on syntax"` arming a trap the enclosing body then uses) both match.
Neither needed anything: `run_fragment` runs inside the creating activation,
so the trap table it edits and the loop that offers to it are the same one.

---

## 7. The defeat-the-mechanism checks

Every load-bearing test was checked by deleting or inverting exactly the line
it exists to protect. The script is
`.../scratchpad/mutate.sh`; each mutation was applied, the named test run, and
the file restored.

| mutation | test | result |
|---|---|---|
| stop clearing `failure_site`/`failure_sites` on a trap | `a_second_raise_after_a_trapped_one_reports_its_own_site` | **KILLED** |
| drop `PendingTrap::depth`'s guard | `a_call_trap_waits_for_the_raising_clause_to_finish` | **KILLED** |
| drop `self.clause_state = saved_clause_state;` | `a_trap_that_resumes_mid_clause_leaves_the_enclosing_clauses_state_intact` | **KILLED** |
| leave the fired trap in the table | `a_trap_is_disabled_when_it_fires_and_can_be_re_armed` | **SURVIVED** |
| make `Search::Caller` behave like `Search::Here` | `raise_delivery_depends_on_the_tail_and_on_the_condition` | **KILLED** |
| retain one root per trapped cycle | `a_trap_that_resumes_does_not_accumulate_temps_frames` | **KILLED** |

**The survivor was a real finding and is fixed.** That test's handler re-arms
the trap under a new label *before* raising again, and `insert` over a live
entry is indistinguishable from `insert` over an absent one -- so the removal
was never exercised. Its neighbour,
`a_second_raise_inside_a_handler_is_fatal_without_a_re_arm`, does go red
against the mutation, but by *looping* until the harness kills it, which is a
poor thing to leave as the only signal.

Added `the_trap_that_fired_is_removed_from_the_table`: it asserts directly
that the fired `SYNTAX` entry is gone from `interp.activation().traps` **and**
that an unrelated `NOVALUE` entry is still there -- without the second half
the assertion is satisfied by clearing the whole table or by never filling it.
It fails in microseconds against the mutation (verified). The over-claiming
test was renamed `a_trap_can_be_re_armed_inside_its_own_handler` and its doc
comment now says what it does and does not pin.

---

## 8. The test-harness change, which is the largest single thing here

`run.rs`'s test module used `run_activated`, a hand-rolled `run_bounded` loop
that reproduced `run_activation`'s `Flow` dispatch arm by arm. It had drifted
once already -- Task 6 had to teach it about `Flow::Signal`.

The condition-trap offer lives in `run_activation`'s own loop, one per
activation. So **eleven trap tests ran against a harness that could not
trap**, while every one of the same programs matched the oracle byte for byte
through `run_program`.

Those eleven went *red*, not green -- they assert values a handler set, and an
untrapped condition surfaced as a `Raised` instead. So this particular set was
never vacuous, and it is what surfaced the problem. **The hazard is what a
differently-shaped test would have done**: any assertion of the form "this
condition is fatal", or any check of an exit code, would have passed against a
harness where nothing could ever trap -- for the wrong reason, silently, and
in a file where trap tests are exactly what belongs. A harness that cannot
reach the code under test is a test that cannot fail, whatever this batch
happened to assert.

`run_activated` now *is* `interp.run_activation().map(Ended::value)`.
`activate` already pushed exactly the activation `Interp::run` pushes, so
there was never anything for the copy to supply. All 277 lib tests pass
against the real dispatch, including every pre-existing one.

---

## 9. Test output

From `rust/`, at `f906aabc`:

```
$ cargo test --workspace
TOTAL passed=956 failed=0

$ cargo test -p rexx-exec --test corpus
38 of 38 matching -- REPORT MODE, NOT THE GATE

$ REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set
38 of 38 matching
test result: ok. 9 passed; 0 failed; 1 ignored

$ cargo test -p rexx-exec --test assertions
4224 of 4259 rows passing -- REPORT MODE, NOT THE GATE
by kind:
  RUNTIME-BLOCKED: 35

$ cargo fmt --all --check          ; echo $?   ->  0
$ cargo clippy --workspace --all-targets -- -D warnings ; echo $?  ->  0
```

`rexx-exec`'s own lib tests went from 255 to 277: 21 new tests plus one
(`the_trap_that_fired_is_removed_from_the_table`) added after the mutation
check. Corpus 37 -> 38. Assertions unchanged at 4224/4259, as expected -- that
table is expression-level and contains no condition traps.

The full-probe regression after all edits: **78 of this task's 85 probes**
match the oracle byte for byte. The seven that do not are `p01`/`p16`/`p17`
(`condition()`, loud 4c), `p42` (`ADDITIONAL (a, b)`, loud Phase 5), and
`p38`/`q5`/`x1` (Concern 4). The directory also holds 14 files belonging to
the sibling agent, four of which differ; those are `INTERPRET` parse-error
reporting and are not this task's.

---

## Concerns

**1. `RAISE PROPAGATE`'s `active_condition` is never cleared.** It is set when
a trap fires and stays set. A `RAISE PROPAGATE` reached *after* a handler has
finished re-raises that handler's condition, where the oracle may well answer
`98.918`. Nothing measured pins that shape either way; the two that do pin
something -- inside a handler, and before any condition at all -- are both
reproduced. Stated in `exec_raise_propagate`'s own doc comment.

**2. Non-`SYNTAX` `RAISE ... RETURN` searches exactly one level out, not
outward.** `exec_raise` looks at the caller's trap table and, finding nothing,
applies the default action there. If the *grandparent* has a trap and the
parent does not, we ignore the condition where the oracle might propagate to
it. Unmeasured: the transcripts that exist (`p16`, `r2`, `p22`, `p25`, `s1`,
`p15`, `t1`) are all one level. Stated in `exec_raise`'s doc comment.

**3. `RAISE PROPAGATE`'s report format rests on four transcripts.** The
`Error 42:` form with no ` running <path> line <n>` span is measured at two
nesting depths (`p27`, `p41`), plus `98.918` (`p49`) and the silent `USER`
case (`s2`). That is a consistent set but a small one, and it is the one place
this task added a new *rendering* rather than a new behaviour. The alternative
was to leave `RAISE PROPAGATE` loud, which would have needed arm-grained
ownership inside `InstructionKind::Raise` on a `bool` field -- machinery this
tree does not have and that costs more than the risk. Flagging the trade
rather than hiding it.

**4. A pre-existing divergence found in passing, not mine and not fixed.**
`EXIT <expr>` emits no `>>>` value line under `trace r`. Oracle:

```
     2 *-* say 'a'
       >>>   "a"
     3 *-* exit 0
       >>>   "0"
```

ours stops after the `*-*`. Reproduced on a three-line program with no
conditions in it at all (`x1.rex`), so it is `InstructionKind::Exit`'s arm
(4a/Task 9), not anything here. The fix looks like one `trace_result` call,
but it is outside this task's remit and would move corpus bytes, so it is
reported rather than done. **It constrains corpus programs**: any `trace r`
witness containing `exit <value>` will diverge, which is why
`condition_traps.rex` has none and says so in its header.

**5. `condition()` remains loud, so no corpus program can read a trapped
condition's name.** `condition('C')`, `('I')`, `('D')`, `('O')` and `('S')`
are all 4c's builtin table. This is correct and expected, but it means the
`USER FOO`/`CALL`/`SIGNAL` values that `p01`, `p16` and `p17` measure are
implemented in `Raised::condition` and `Trap::call` without a differential
witness. `rust/corpus/lang/condition_syntax.rex` -- which exists in the corpus
directory but is listed in no subset -- is presumably waiting for exactly
that, and 4c can list it.

---

## Appendix A: every oracle transcript

All 85 probes, source with line numbers followed by the oracle's stdout and
stderr as separate descriptors. Directory as stated in section 1; the
seventeen decisive ones re-verified in a clean directory.

### `ct.rex`

```
  1  /* Condition traps, RAISE and NOVALUE (4b Task 7), under trace r throughout.
  2   *
  3   * Every block asserts a value a handler SET, never that the program reached
  4   * the end: a trap test that checks only the exit code is satisfied by a
  5   * program that never raised at all. ZWITNESS accumulates one segment per
  6   * block and the whole string is printed at the end, so a block that silently
  7   * did not run removes its segment rather than leaving the output unchanged.
  8   * No segment is a variable's derived name or any prefix of one -- an unset
  9   * Rexx variable reads as its own uppercased spelling, so a flag left unset
 10   * renders as plausible-looking data, and the values here are chosen so that
 11   * a failure prints ZWITNESS or ZUNSET_PROBE instead of something readable.
 12   *
 13   * What each block pins, and the wrong answer it would print:
 14   *
 15   *   1. SIGNAL ON SYNTAX. The trap fires on a raise from an ordinary
 16   *      expression and SIGL is the RAISING clause's line, not the SIGNAL ON
 17   *      clause's and not the handler's. An implementation that never traps
 18   *      gets the fatal 42.3 report instead of any output at all.
 19   *
 20   *   2. The trap is DISABLED once it fires. Block 2 re-arms SYNTAX under a
 21   *      different label and reaches that second label; an implementation that
 22   *      left the first trap armed would re-enter TRAP_SYNTAX and loop.
 23   *
 24   *   3. SIGNAL ON NOVALUE, on a simple variable and on a compound. A bare
 25   *      stem is deliberately NOT here: measured, `say zstem.` does not raise
 26   *      NOVALUE where `say zstem.1` does.
 27   *
 28   *   4. CALL ON USER, whose handler runs at the CLAUSE BOUNDARY rather than
 29   *      at the raise. The `call raiser` clause settles RESULT from the
 30   *      routine's own RAISE ... RETURN value first, and only then does the
 31   *      handler append its segment -- so the ordering of `/RAISER-RETURNED`
 32   *      and `/USER-CALLED` in the output is what distinguishes "resumed
 33   *      after the clause" from "transferred at the raise".
 34   *
 35   *   5. SIGNAL OFF. The second `say 1/0` is NOT trapped, which is what ends
 36   *      the program: the file's last three lines of stderr are the ordinary
 37   *      fatal report, and an implementation that ignored SIGNAL OFF would
 38   *      print the handler's output and exit 0 instead of 214.
 39   *
 40   * NOT here, deliberately: `exit <value>` anywhere under `trace r`. That
 41   * clause's own `>>>` value line is missing from this crate (a pre-existing
 42   * gap in the EXIT arm, unrelated to conditions), so a program containing one
 43   * would diverge for a reason that has nothing to do with what it is
 44   * witnessing. The program ends on an untrapped raise instead.
 45   */
 46  trace r
 47  signal on syntax name trap_syntax
 48  zwitness = 'START'
 49  say 1/0
 50  say 'unreachable-after-block-1'
 51  
 52  trap_syntax:
 53  zwitness = zwitness'/SYNTAX-AT-'sigl
 54  signal on syntax name trap_raise
 55  raise syntax 40.4 array ('ZORKROUTINE', 7)
 56  say 'unreachable-after-block-2'
 57  
 58  trap_raise:
 59  zwitness = zwitness'/RAISED-AT-'sigl
 60  signal on novalue name trap_novalue
 61  say zunset_probe
 62  say 'unreachable-after-block-3'
 63  
 64  trap_novalue:
 65  zwitness = zwitness'/NOVALUE-AT-'sigl
 66  signal on novalue name trap_tail
 67  say zunset_stem.1
 68  say 'unreachable-after-block-4'
 69  
 70  trap_tail:
 71  zwitness = zwitness'/TAIL-AT-'sigl
 72  call on user marker name trap_user
 73  call raiser
 74  zwitness = zwitness'/'result
 75  say 'accumulated:' zwitness
 76  signal block_five
 77  
 78  raiser:
 79  raise user marker return 'RAISER-RETURNED'
 80  
 81  trap_user:
 82  zwitness = zwitness'/USER-CALLED-AT-'sigl
 83  return
 84  
 85  block_five:
 86  signal on syntax name trap_never
 87  signal off syntax
 88  say 'final:' zwitness
 89  say 2/0
 90  say 'unreachable-after-block-5'
 91  
 92  trap_never:
 93  say 'THIS-HANDLER-MUST-NOT-RUN'
```

oracle stdout:
```
accumulated: START/SYNTAX-AT-49/RAISED-AT-55/NOVALUE-AT-61/TAIL-AT-67/USER-CALLED-AT-73/RAISER-RETURNED
final: START/SYNTAX-AT-49/RAISED-AT-55/NOVALUE-AT-61/TAIL-AT-67/USER-CALLED-AT-73/RAISER-RETURNED
```

oracle stderr:
```
    47 *-* signal on syntax name trap_syntax
    48 *-* zwitness = 'START'
       >>>   "START"
    49 *-* say 1/0
    52 *-* trap_syntax:
    53 *-* zwitness = zwitness'/SYNTAX-AT-'sigl
       >>>   "START/SYNTAX-AT-49"
    54 *-* signal on syntax name trap_raise
    55 *-* raise syntax 40.4 array ('ZORKROUTINE', 7)
       >K>   "SYNTAX" => "40.4"
       >K>   "ARRAY" => "an Array"
    58 *-* trap_raise:
    59 *-* zwitness = zwitness'/RAISED-AT-'sigl
       >>>   "START/SYNTAX-AT-49/RAISED-AT-55"
    60 *-* signal on novalue name trap_novalue
    61 *-* say zunset_probe
    64 *-* trap_novalue:
    65 *-* zwitness = zwitness'/NOVALUE-AT-'sigl
       >>>   "START/SYNTAX-AT-49/RAISED-AT-55/NOVALUE-AT-61"
    66 *-* signal on novalue name trap_tail
    67 *-* say zunset_stem.1
    70 *-* trap_tail:
    71 *-* zwitness = zwitness'/TAIL-AT-'sigl
       >>>   "START/SYNTAX-AT-49/RAISED-AT-55/NOVALUE-AT-61/TAIL-AT-67"
    72 *-* call on user marker name trap_user
    73 *-* call raiser
    78 *-*   raiser:
    79 *-*   raise user marker return 'RAISER-RETURNED'
       >K>     "RESULT" => "RAISER-RETURNED"
       >>>   "RAISER-RETURNED"
    81 *-*   trap_user:
    82 *-*   zwitness = zwitness'/USER-CALLED-AT-'sigl
       >>>     "START/SYNTAX-AT-49/RAISED-AT-55/NOVALUE-AT-61/TAIL-AT-67/USER-CALLED-AT-73"
    83 *-*   return
    74 *-* zwitness = zwitness'/'result
       >>>   "START/SYNTAX-AT-49/RAISED-AT-55/NOVALUE-AT-61/TAIL-AT-67/USER-CALLED-AT-73/RAISER-RETURNED"
    75 *-* say 'accumulated:' zwitness
       >>>   "accumulated: START/SYNTAX-AT-49/RAISED-AT-55/NOVALUE-AT-61/TAIL-AT-67/USER-CALLED-AT-73/RAISER-RETURNED"
    76 *-* signal block_five
    85 *-* block_five:
    86 *-* signal on syntax name trap_never
    87 *-* signal off syntax
    88 *-* say 'final:' zwitness
       >>>   "final: START/SYNTAX-AT-49/RAISED-AT-55/NOVALUE-AT-61/TAIL-AT-67/USER-CALLED-AT-73/RAISER-RETURNED"
    89 *-* say 2/0
    89 *-* say 2/0
Error 42 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/ct.rex line 89:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
```

### `p01.rex`

```
  1  signal on syntax
  2  zmark = 'M1'
  3  say 1/0
  4  say 'not reached'
  5  exit 9
  6  syntax:
  7  zmark = 'TRAPPED-77'
  8  say 'zmark=' zmark
  9  say 'sigl=' sigl
 10  say 'cond=' condition('C')
 11  say 'condname=' condition('E')
 12  exit 0
```

oracle stdout:
```
zmark= TRAPPED-77
sigl= 3
cond= SYNTAX
condname= 3
```

oracle stderr:
```
(empty)```

### `p02.rex`

```
  1  call on error name errh
  2  say 'main done'
  3  exit 0
  4  errh:
  5  say 'handler ran'
  6  return
```

oracle stdout:
```
main done
```

oracle stderr:
```
(empty)```

### `p03.rex`

```
  1  signal on novalue
  2  zmark = 'M1'
  3  say zunsetvar
  4  say 'after'
  5  exit 9
  6  novalue:
  7  zmark = 'TRAPPED-77'
  8  say 'zmark=' zmark
  9  say 'sigl=' sigl
 10  exit 0
```

oracle stdout:
```
zmark= TRAPPED-77
sigl= 3
```

oracle stderr:
```
(empty)```

### `p04.rex`

```
  1  say 'a'
  2  raise syntax 40.4
  3  say 'b'
```

oracle stdout:
```
a
```

oracle stderr:
```
     2 *-* raise syntax 40.4
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p04.rex line 2:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of &1; maximum expected is &2.
```

### `p05.rex`

```
  1  say fun(1)
  2  say 'after'
  3  exit 0
  4  fun: procedure
  5  raise syntax 40.4 return 'RETVAL-88'
```

oracle stdout:
```
(empty)```

oracle stderr:
```
     5 *-*   raise syntax 40.4 return 'RETVAL-88'
     1 *-* say fun(1)
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p05.rex line 5:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of &1; maximum expected is &2.
```

### `p06.rex`

```
  1  signal on syntax
  2  say fun(1)
  3  say 'after'
  4  exit 9
  5  fun: procedure
  6  raise syntax 40.4 return 'RETVAL-88'
  7  syntax:
  8  say 'zmark=TRAPPED-77'
  9  say 'sigl=' sigl
 10  exit 0
```

oracle stdout:
```
zmark=TRAPPED-77
sigl= 6
```

oracle stderr:
```
(empty)```

### `p07.rex`

```
  1  call on user foo name uh
  2  say 'a'
  3  raise user foo
  4  say 'b'
  5  exit 0
  6  uh:
  7  say 'handler-77'
  8  say 'sigl=' sigl
  9  return
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `p08.rex`

```
  1  signal on user foo
  2  say 'a'
  3  raise user foo
  4  say 'b'
  5  exit 9
  6  foo:
  7  say 'handler-77'
  8  say 'sigl=' sigl
  9  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `p09.rex`

```
  1  signal on syntax
  2  say 'a'
  3  interpret "say 1/0"
  4  say 'b'
  5  exit 9
  6  syntax:
  7  say 'handler-77'
  8  say 'sigl=' sigl
  9  exit 0
```

oracle stdout:
```
a
handler-77
sigl= 3
```

oracle stderr:
```
(empty)```

### `p10.rex`

```
  1  say 'a'
  2  interpret "signal on syntax"
  3  say 1/0
  4  say 'b'
  5  exit 9
  6  syntax:
  7  say 'handler-77'
  8  say 'sigl=' sigl
  9  exit 0
```

oracle stdout:
```
a
handler-77
sigl= 3
```

oracle stderr:
```
(empty)```

### `p11.rex`

```
  1  signal on syntax
  2  say 'a'
  3  raise syntax 40.4
  4  say 'b'
  5  exit 9
  6  syntax:
  7  say 'handler-77'
  8  say 'sigl=' sigl
  9  exit 0
```

oracle stdout:
```
a
handler-77
sigl= 3
```

oracle stderr:
```
(empty)```

### `p12.rex`

```
  1  call on user foo name uh
  2  say 'a'
  3  say fun(1)
  4  say 'b'
  5  exit 0
  6  fun:
  7  raise user foo
  8  return 'unreached'
  9  uh:
 10  say 'handler-77'
 11  say 'sigl=' sigl
 12  return
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `p13.rex`

```
  1  signal on user foo
  2  say 'a'
  3  say fun(1)
  4  say 'b'
  5  exit 9
  6  fun:
  7  raise user foo
  8  return 'unreached'
  9  foo:
 10  say 'handler-77'
 11  say 'sigl=' sigl
 12  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `p14.rex`

```
  1  say 'a'
  2  raise user foo
  3  say 'b'
  4  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `p15.rex`

```
  1  say 'a'
  2  say fun(1)
  3  say 'b'
  4  exit 0
  5  fun:
  6  raise user foo return 'RETVAL-88'
```

oracle stdout:
```
a
RETVAL-88
b
```

oracle stderr:
```
(empty)```

### `p16.rex`

```
  1  signal on user foo
  2  say 'a'
  3  call fun
  4  say 'b'
  5  exit 9
  6  fun:
  7  raise user foo return
  8  foo:
  9  say 'handler-77'
 10  say 'sigl=' sigl
 11  say 'cond=' condition('C')
 12  exit 0
```

oracle stdout:
```
a
handler-77
sigl= 3
cond= USER FOO
```

oracle stderr:
```
(empty)```

### `p17.rex`

```
  1  call on user foo name uh
  2  say 'a'
  3  call fun
  4  say 'b'
  5  exit 0
  6  fun:
  7  raise user foo return
  8  uh:
  9  say 'handler-77'
 10  say 'sigl=' sigl
 11  say 'cond=' condition('C')
 12  say 'instr=' condition('I')
 13  return
```

oracle stdout:
```
a
handler-77
sigl= 3
cond= USER FOO
instr= CALL
b
```

oracle stderr:
```
(empty)```

### `p18.rex`

```
  1  signal on syntax
  2  say 'a'
  3  call fun
  4  say 'b'
  5  exit 9
  6  fun:
  7  say 1/0
  8  return
  9  syntax:
 10  say 'handler-77'
 11  say 'sigl=' sigl
 12  exit 0
```

oracle stdout:
```
a
handler-77
sigl= 7
```

oracle stderr:
```
(empty)```

### `p19.rex`

```
  1  say 'a'
  2  call fun
  3  say 'b'
  4  exit 0
  5  fun:
  6  signal on syntax
  7  say 1/0
  8  say 'noreach'
  9  return
 10  syntax:
 11  say 'handler-77 sigl=' sigl
 12  return 'RETFROMHANDLER'
```

oracle stdout:
```
a
handler-77 sigl= 7
b
```

oracle stderr:
```
(empty)```

### `p20.rex`

```
  1  signal on syntax
  2  say 'a'
  3  signal off syntax
  4  say 1/0
  5  say 'b'
  6  exit 9
  7  syntax:
  8  say 'handler-77'
  9  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
     4 *-* say 1/0
Error 42 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p20.rex line 4:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
```

### `p21.rex`

```
  1  signal on syntax
  2  zowner = 'MAIN-POOL'
  3  say 'a'
  4  call fun
  5  say 'b'
  6  exit 9
  7  fun: procedure
  8  zowner = 'FUN-POOL'
  9  say 1/0
 10  return
 11  syntax:
 12  say 'owner=' zowner
 13  say 'sigl=' sigl
 14  exit 0
```

oracle stdout:
```
a
owner= FUN-POOL
sigl= 9
```

oracle stderr:
```
(empty)```

### `p22.rex`

```
  1  call on user foo name uh
  2  zowner = 'MAIN-POOL'
  3  say 'a' one(1) two(2)
  4  say 'b'
  5  exit 0
  6  one:
  7  raise user foo return 'ONEVAL'
  8  two:
  9  return 'TWOVAL'
 10  uh:
 11  say 'handler-77 owner=' zowner 'sigl=' sigl
 12  return
```

oracle stdout:
```
a ONEVAL TWOVAL
handler-77 owner= MAIN-POOL sigl= 3
b
```

oracle stderr:
```
(empty)```

### `p23.rex`

```
  1  call on user foo name uh
  2  say 'a'
  3  say 'z' one(1) two(2)
  4  say 'b'
  5  exit 0
  6  one: procedure
  7  raise user foo return 'ONEVAL'
  8  two: procedure
  9  return 'TWOVAL'
 10  uh:
 11  say 'handler-77 sigl=' sigl
 12  return
```

oracle stdout:
```
a
z ONEVAL TWOVAL
handler-77 sigl= 3
b
```

oracle stderr:
```
(empty)```

### `p24.rex`

```
  1  signal on syntax
  2  say 'a'
  3  call fun
  4  say 'b'
  5  exit 9
  6  fun:
  7  signal off syntax
  8  say 1/0
  9  return
 10  syntax:
 11  say 'handler-77 sigl=' sigl
 12  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `p25.rex`

```
  1  call on user foo name uh
  2  zres = one(1)
  3  say 'zres=' zres
  4  exit 0
  5  one:
  6  raise user foo return 'ONEVAL'
  7  uh:
  8  zres = 'HANDLERVAL'
  9  return
```

oracle stdout:
```
zres= HANDLERVAL
```

oracle stderr:
```
(empty)```

### `p26.rex`

```
  1  signal on syntax
  2  zz = one(1) two(2)
  3  say 'zz=' zz
  4  exit 0
  5  one:
  6  say 1/0
  7  return 'ONEVAL'
  8  two:
  9  return 'SIGLIS' sigl
 10  syntax:
 11  return 'FROMHANDLER'
```

oracle stdout:
```
zz= FROMHANDLER SIGLIS 2
```

oracle stderr:
```
(empty)```

### `p27.rex`

```
  1  signal on syntax
  2  say 'a'
  3  call fun
  4  say 'b'
  5  exit 9
  6  fun:
  7  signal on syntax name inner
  8  say 1/0
  9  return
 10  inner:
 11  say 'inner-77 sigl=' sigl
 12  raise propagate
 13  syntax:
 14  say 'outer-77 sigl=' sigl
 15  exit 0
```

oracle stdout:
```
a
inner-77 sigl= 8
```

oracle stderr:
```
     8 *-*   say 1/0
     3 *-* call fun
Error 42:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
```

### `p28.rex`

```
  1  say 'a'
  2  say fun(1)
  3  say 'b'
  4  exit 9
  5  fun:
  6  raise syntax 40.4 exit 'EXITVAL-88'
```

oracle stdout:
```
a
```

oracle stderr:
```
     6 *-*   raise syntax 40.4 exit 'EXITVAL-88'
     2 *-* say fun(1)
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p28.rex line 6:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of &1; maximum expected is &2.
```

### `p29.rex`

```
  1  signal on syntax
  2  say 'a'
  3  say fun(1)
  4  say 'b'
  5  exit 9
  6  fun:
  7  raise syntax 40.4 exit 'EXITVAL-88'
  8  syntax:
  9  say 'handler-77 sigl=' sigl
 10  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `p30.rex`

```
  1  signal on syntax
  2  say 'a'
  3  say fun(1)
  4  say 'b'
  5  exit 9
  6  fun:
  7  raise syntax 40.4 return 'RETVAL-88'
  8  syntax:
  9  say 'handler-77 sigl=' sigl
 10  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 7
```

oracle stderr:
```
(empty)```

### `p31.rex`

```
  1  signal on syntax
  2  say 'a'
  3  say fun(1)
  4  say 'b'
  5  exit 9
  6  fun: procedure
  7  raise syntax 40.4 exit 'EXITVAL-88'
  8  syntax:
  9  say 'handler-77 sigl=' sigl
 10  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `p32.rex`

```
  1  signal on syntax
  2  say 'a'
  3  say fun(1)
  4  say 'b'
  5  exit 9
  6  fun:
  7  raise syntax 40.4
  8  syntax:
  9  say 'handler-77 sigl=' sigl
 10  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `p33.rex`

```
  1  say 'a'
  2  say fun(1)
  3  say 'b'
  4  exit 9
  5  fun:
  6  say 1/0
  7  return 'X'
```

oracle stdout:
```
a
```

oracle stderr:
```
     6 *-*   say 1/0
     2 *-* say fun(1)
Error 42 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p33.rex line 6:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
```

### `p34.rex`

```
  1  signal on syntax
  2  say 'a'
  3  say 1/0
  4  say 'b'
  5  exit 9
  6  syntax:
  7  say 'first-77 sigl=' sigl
  8  say 2/0
  9  say 'c'
 10  exit 0
```

oracle stdout:
```
a
first-77 sigl= 3
```

oracle stderr:
```
     8 *-* say 2/0
Error 42 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p34.rex line 8:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
```

### `p35.rex`

```
  1  signal on syntax
  2  zowner = 'MAIN-POOL'
  3  say fun(1)
  4  exit 9
  5  fun: procedure
  6  zowner = 'FUN-POOL'
  7  raise syntax 40.4 return 'RETVAL-88'
  8  syntax:
  9  say 'owner=' zowner 'sigl=' sigl
 10  exit 0
```

oracle stdout:
```
owner= FUN-POOL sigl= 7
```

oracle stderr:
```
(empty)```

### `p36.rex`

```
  1  signal on syntax
  2  zowner = 'MAIN-POOL'
  3  say fun(1)
  4  exit 9
  5  fun: procedure
  6  zowner = 'FUN-POOL'
  7  raise syntax 40.4
  8  syntax:
  9  say 'owner=' zowner 'sigl=' sigl
 10  exit 0
```

oracle stdout:
```
owner= MAIN-POOL sigl= 3
```

oracle stderr:
```
(empty)```

### `p37.rex`

```
  1  say 'a'
  2  call fun
  3  say 'b'
  4  exit 0
  5  fun:
  6  signal on syntax
  7  zres = one(1)
  8  say 'zres=' zres
  9  return
 10  one:
 11  raise syntax 40.4 return 'RETVAL-88'
 12  syntax:
 13  say 'handler-77 sigl=' sigl
 14  return 'FROMHANDLER'
```

oracle stdout:
```
a
handler-77 sigl= 11
zres= FROMHANDLER
b
```

oracle stderr:
```
(empty)```

### `p38.rex`

```
  1  trace r
  2  signal on syntax
  3  say 'a'
  4  say 1/0
  5  say 'b'
  6  exit 9
  7  syntax:
  8  say 'handler-77'
  9  exit 0
```

oracle stdout:
```
a
handler-77
```

oracle stderr:
```
     2 *-* signal on syntax
     3 *-* say 'a'
       >>>   "a"
     4 *-* say 1/0
     7 *-* syntax:
     8 *-* say 'handler-77'
       >>>   "handler-77"
     9 *-* exit 0
       >>>   "0"
```

### `p39.rex`

```
  1  signal on novalue
  2  say 'a'
  3  zres = zunset1 || 'T'
  4  say 'zres=' zres
  5  exit 9
  6  novalue:
  7  say 'handler-77 sigl=' sigl
  8  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `p40.rex`

```
  1  signal on syntax
  2  say 'a'
  3  call lev1
  4  say 'b'
  5  exit 9
  6  lev1:
  7  zowner = 'LEV1'
  8  call lev2
  9  return
 10  lev2: procedure
 11  raise syntax 40.4 exit 'EXITVAL-88'
 12  syntax:
 13  say 'handler-77 sigl=' sigl 'owner=' zowner
 14  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3 owner= LEV1
```

oracle stderr:
```
(empty)```

### `p41.rex`

```
  1  say 'a'
  2  call lev1
  3  say 'b'
  4  exit 9
  5  lev1:
  6  signal on syntax name outer
  7  call lev2
  8  say 'lev1 resumed'
  9  return
 10  lev2:
 11  signal on syntax name inner
 12  say 1/0
 13  return
 14  inner:
 15  say 'inner-77 sigl=' sigl
 16  raise propagate
 17  outer:
 18  say 'outer-77 sigl=' sigl
 19  return
```

oracle stdout:
```
a
inner-77 sigl= 12
```

oracle stderr:
```
    12 *-*     say 1/0
     7 *-*   call lev2
     2 *-* call lev1
Error 42:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
```

### `p42.rex`

```
  1  say 'a'
  2  raise syntax 40.4 additional ('MYROUTINE', 3)
  3  say 'b'
```

oracle stdout:
```
a
```

oracle stderr:
```
     2 *-* raise syntax 40.4 additional ('MYROUTINE', 3)
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p42.rex line 2:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of MYROUTINE; maximum expected is 3.
```

### `p43.rex`

```
  1  say 'a'
  2  raise syntax 40.4 array ('MYROUTINE', 3)
  3  say 'b'
```

oracle stdout:
```
a
```

oracle stderr:
```
     2 *-* raise syntax 40.4 array ('MYROUTINE', 3)
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p43.rex line 2:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of MYROUTINE; maximum expected is 3.
```

### `p44.rex`

```
  1  say 'a'
  2  raise syntax 40.4 description 'my own description'
  3  say 'b'
```

oracle stdout:
```
a
```

oracle stderr:
```
     2 *-* raise syntax 40.4 description 'my own description'
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p44.rex line 2:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of &1; maximum expected is &2.
```

### `p45.rex`

```
  1  say 'a'
  2  raise error 5
  3  say 'b'
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `p46.rex`

```
  1  signal on error
  2  say 'a'
  3  raise error 5
  4  say 'b'
  5  exit 9
  6  error:
  7  say 'handler-77 sigl=' sigl 'rc=' rc
  8  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `p47.rex`

```
  1  say 'a'
  2  raise halt
  3  say 'b'
```

oracle stdout:
```
a
```

oracle stderr:
```
     2 *-* raise halt
Error 4 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p47.rex line 2:  Program interrupted.
Error 4.1:  Program interrupted with HALT condition.
```

### `p48.rex`

```
  1  signal on halt
  2  say 'a'
  3  raise halt description 'stopped'
  4  say 'b'
  5  exit 9
  6  halt:
  7  say 'handler-77 sigl=' sigl
  8  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
     3 *-* raise halt description 'stopped'
Error 4 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p48.rex line 3:  Program interrupted.
Error 4.1:  Program interrupted with HALT condition.
```

### `p49.rex`

```
  1  call on user foo name uh
  2  say 'a'
  3  raise propagate
  4  say 'b'
  5  exit 0
  6  uh:
  7  say 'handler-77'
  8  return
```

oracle stdout:
```
a
```

oracle stderr:
```
     3 *-* raise propagate
Error 98 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/p49.rex line 3:  Execution error.
Error 98.918:  No active condition available for PROPAGATE.
```

### `q1.rex`

```
  1  say 'a'
  2  say fun(1)
  3  say 'b'
  4  exit 9
  5  fun:
  6  raise syntax 40.4
```

oracle stdout:
```
a
```

oracle stderr:
```
     6 *-*   raise syntax 40.4
     2 *-* say fun(1)
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/q1.rex line 6:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of &1; maximum expected is &2.
```

### `q2.rex`

```
  1  signal on syntax name outer
  2  say 'a'
  3  call lev1
  4  exit 9
  5  lev1:
  6  signal on syntax name mid
  7  call lev2
  8  return
  9  lev2: procedure
 10  raise syntax 40.4
 11  mid:
 12  say 'mid-77 sigl=' sigl
 13  exit 0
 14  outer:
 15  say 'outer-77 sigl=' sigl
 16  exit 0
```

oracle stdout:
```
a
outer-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `q3.rex`

```
  1  signal on syntax
  2  say 'a'
  3  say 1/0
  4  exit 9
  5  syntax:
  6  say 'first-77 sigl=' sigl
  7  signal on syntax name second
  8  say 2/0
  9  say 'unreached'
 10  exit 8
 11  second:
 12  say 'second-77 sigl=' sigl
 13  exit 0
```

oracle stdout:
```
a
first-77 sigl= 3
second-77 sigl= 8
```

oracle stderr:
```
(empty)```

### `q4.rex`

```
  1  call on user foo name uh
  2  say 'a'
  3  say one(1)
  4  say 'b'
  5  exit 0
  6  one:
  7  raise user foo return 'ONEVAL'
  8  uh:
  9  say 'handler-77 entered'
 10  say two(2)
 11  return
 12  two:
 13  raise user foo return 'TWOVAL'
```

oracle stdout:
```
a
ONEVAL
handler-77 entered
TWOVAL
b
```

oracle stderr:
```
(empty)```

### `q5.rex`

```
  1  trace r
  2  call on user foo name uh
  3  say one(1)
  4  say 'b'
  5  exit 0
  6  one:
  7  raise user foo return 'ONEVAL'
  8  uh:
  9  say 'handler-77'
 10  return
```

oracle stdout:
```
ONEVAL
handler-77
b
```

oracle stderr:
```
     2 *-* call on user foo name uh
     3 *-* say one(1)
     6 *-*   one:
     7 *-*   raise user foo return 'ONEVAL'
       >K>     "RESULT" => "ONEVAL"
       >>>   "ONEVAL"
     8 *-*   uh:
     9 *-*   say 'handler-77'
       >>>     "handler-77"
    10 *-*   return
     4 *-* say 'b'
       >>>   "b"
     5 *-* exit 0
       >>>   "0"
```

### `q6.rex`

```
  1  say 'a'
  2  call lev1
  3  say 'z'
  4  exit 9
  5  lev1:
  6  signal on syntax name mid
  7  call lev2
  8  return
  9  lev2: procedure
 10  raise syntax 40.4
 11  mid:
 12  say 'mid-77 sigl=' sigl
 13  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
    10 *-*     raise syntax 40.4
     7 *-*   call lev2
     2 *-* call lev1
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/q6.rex line 10:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of &1; maximum expected is &2.
```

### `q7.rex`

```
  1  say 'a'
  2  call lev1
  3  say 'z'
  4  exit 9
  5  lev1:
  6  signal on syntax name mid
  7  call lev2
  8  return
  9  lev2: procedure
 10  raise syntax 40.4 exit
 11  mid:
 12  say 'mid-77 sigl=' sigl
 13  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
    10 *-*     raise syntax 40.4 exit
     7 *-*   call lev2
     2 *-* call lev1
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/q7.rex line 10:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of &1; maximum expected is &2.
```

### `q8.rex`

```
  1  say 'a'
  2  call lev1
  3  say 'z'
  4  exit 9
  5  lev1:
  6  signal on syntax name mid
  7  call lev2
  8  return
  9  lev2: procedure
 10  say 1/0
 11  mid:
 12  say 'mid-77 sigl=' sigl
 13  exit 0
```

oracle stdout:
```
a
mid-77 sigl= 10
```

oracle stderr:
```
(empty)```

### `q9.rex`

```
  1  signal on novalue
  2  say 'a'
  3  call fun
  4  say 'z'
  5  exit 9
  6  fun: procedure
  7  say zunset2
  8  return
  9  novalue:
 10  say 'handler-77 sigl=' sigl
 11  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 7
```

oracle stderr:
```
(empty)```

### `r1.rex`

```
  1  signal on user foo
  2  say 'a'
  3  call fun
  4  say 'z'
  5  exit 9
  6  fun:
  7  raise user foo
  8  foo:
  9  say 'handler-77 sigl=' sigl
 10  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `r2.rex`

```
  1  signal on user foo
  2  say 'a'
  3  say fun(1)
  4  say 'z'
  5  exit 9
  6  fun:
  7  raise user foo return 'RETVAL'
  8  foo:
  9  say 'handler-77 sigl=' sigl
 10  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `r3.rex`

```
  1  signal on syntax
  2  say 'a'
  3  say fun(1)
  4  say 'z'
  5  exit 9
  6  fun:
  7  raise syntax 40.4
  8  syntax:
  9  say 'handler-77 sigl=' sigl
 10  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `r4.rex`

```
  1  signal on user foo
  2  say 'a'
  3  say fun(1)
  4  say 'z'
  5  exit 9
  6  fun:
  7  raise user foo
  8  return 'unreached'
  9  foo:
 10  say 'handler-77 sigl=' sigl
 11  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `s1.rex`

```
  1  signal on error
  2  say 'a'
  3  call fun
  4  say 'z'
  5  exit 9
  6  fun:
  7  raise error 5 return
  8  error:
  9  say 'handler-77 sigl=' sigl 'rc=' rc
 10  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3 rc= 5
```

oracle stderr:
```
(empty)```

### `s2.rex`

```
  1  call on user foo name uh
  2  say 'a'
  3  call fun
  4  say 'z'
  5  exit 0
  6  fun:
  7  raise user foo return
  8  uh:
  9  say 'handler-77'
 10  raise propagate
```

oracle stdout:
```
a
handler-77
```

oracle stderr:
```
(empty)```

### `s3.rex`

```
  1  signal on syntax
  2  say 'a'
  3  say fun(1)
  4  say 'z'
  5  exit 9
  6  fun:
  7  raise user foo exit 'EXITVAL'
  8  syntax:
  9  say 'handler-77 sigl=' sigl
 10  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
(empty)```

### `s4.rex`

```
  1  signal on novalue
  2  say 'a'
  3  say zunset3
  4  say 'z'
  5  exit 9
  6  novalue:
  7  say 'handler-77 sigl=' sigl
  8  say zunset4
  9  say 'after second unset'
 10  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
ZUNSET4
after second unset
```

oracle stderr:
```
(empty)```

### `s5.rex`

```
  1  signal on syntax
  2  say 'a'
  3  call fun
  4  say 'z'
  5  exit 9
  6  fun:
  7  raise syntax 40.4 return 'RETVAL'
  8  say 'fun after raise'
  9  return 'NORMALRET'
 10  syntax:
 11  say 'handler-77 sigl=' sigl
 12  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 7
```

oracle stderr:
```
(empty)```

### `t1.rex`

```
  1  say 'a'
  2  call fun
  3  say 'z'
  4  exit 9
  5  fun:
  6  raise halt return
```

oracle stdout:
```
a
```

oracle stderr:
```
     6 *-*   raise halt return
     2 *-* call fun
Error 4 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/t1.rex line 6:  Program interrupted.
Error 4.1:  Program interrupted with HALT condition.
```

### `t2.rex`

```
  1  signal on syntax name nosuchlabel
  2  say 'a'
  3  say 1/0
  4  say 'z'
```

oracle stdout:
```
a
```

oracle stderr:
```
     3 *-* say 1/0
Error 16 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/t2.rex line 3:  Label not found.
Error 16.1:  Label "NOSUCHLABEL" not found.
```

### `u1.rex`

```
  1  signal on any
  2  say 'a'
  3  say 1/0
  4  say 'b'
  5  exit 9
  6  any:
  7  say 'handler-77 sigl=' sigl
  8  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `u2.rex`

```
  1  signal on novalue
  2  say 'a'
  3  say zunset7.1
  4  say 'b'
  5  exit 9
  6  novalue:
  7  say 'handler-77 sigl=' sigl
  8  exit 0
```

oracle stdout:
```
a
handler-77 sigl= 3
```

oracle stderr:
```
(empty)```

### `u3.rex`

```
  1  say 'a'
  2  say fun(1)
  3  exit 9
  4  fun:
  5  raise syntax 40.912
```

oracle stdout:
```
a
```

oracle stderr:
```
     5 *-*   raise syntax 40.912
     2 *-* say fun(1)
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/u3.rex line 5:  Incorrect call to routine.
Error 40.912:  &1 argument &2 must be a single-dimensional array; found "&3".
```

### `v1.rex`

```
  1  call on any name uh
  2  say 'a'
  3  say 1/0
  4  say 'b'
  5  exit 9
  6  uh:
  7  say 'handler-77 sigl=' sigl
  8  return
```

oracle stdout:
```
a
```

oracle stderr:
```
     3 *-* say 1/0
Error 42 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/v1.rex line 3:  Arithmetic overflow/underflow.
Error 42.3:  Arithmetic overflow; divisor must not be zero.
```

### `v2.rex`

```
  1  signal on syntax
  2  say 'a'
  3  raise syntax 40.4
  4  say 'b'
  5  exit 9
  6  syntax:
  7  say 'handler-77 rc=' rc
  8  exit 0
```

oracle stdout:
```
a
handler-77 rc= 40
```

oracle stderr:
```
(empty)```

### `v3.rex`

```
  1  signal on novalue
  2  say 'a'
  3  say zunsetstem8.
  4  say 'b'
  5  exit 9
  6  novalue:
  7  say 'handler-77 sigl=' sigl
  8  exit 0
```

oracle stdout:
```
a
ZUNSETSTEM8.
b
```

oracle stderr:
```
(empty)```

### `v4.rex`

```
  1  signal on syntax
  2  say 'a'
  3  say 1/0
  4  exit 9
  5  syntax:
  6  say 'handler-77 rc=' rc
  7  exit 0
```

oracle stdout:
```
a
handler-77 rc= 42
```

oracle stderr:
```
(empty)```

### `v5.rex`

```
  1  signal on novalue
  2  say 'a'
  3  say zunset9
  4  exit 9
  5  novalue:
  6  say 'handler-77 rc=' rc
  7  exit 0
```

oracle stdout:
```
a
handler-77 rc= RC
```

oracle stderr:
```
(empty)```

### `w1.rex`

```
  1  say 'a'
  2  raise syntax 40
  3  say 'b'
```

oracle stdout:
```
a
```

oracle stderr:
```
     2 *-* raise syntax 40
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/w1.rex line 2:  Incorrect call to routine.
```

### `w2.rex`

```
  1  say 'a'
  2  raise syntax 40.4 additional 'JUSTONE'
  3  say 'b'
```

oracle stdout:
```
a
```

oracle stderr:
```
     2 *-* raise syntax 40.4 additional 'JUSTONE'
Error 40 running /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/probe/w2.rex line 2:  Incorrect call to routine.
Error 40.4:  Too many arguments in invocation of JUSTONE; maximum expected is &2.
```

### `w3.rex`

```
  1  trace r
  2  say 'a'
  3  say fun(1)
  4  exit 0
  5  fun:
  6  raise user foo exit 'EXITVAL'
```

oracle stdout:
```
a
```

oracle stderr:
```
     2 *-* say 'a'
       >>>   "a"
     3 *-* say fun(1)
     5 *-*   fun:
     6 *-*   raise user foo exit 'EXITVAL'
       >K>     "RESULT" => "EXITVAL"
```

### `x1.rex`

```
  1  trace r
  2  say 'a'
  3  exit 0
```

oracle stdout:
```
a
```

oracle stderr:
```
     2 *-* say 'a'
       >>>   "a"
     3 *-* exit 0
       >>>   "0"
```

### `y1.rex`

```
  1  trace r
  2  signal on syntax name th
  3  raise syntax 40.4 description 'zdesc' additional 'zadd'
  4  say 'x'
  5  th:
  6  say 'trapped'
```

oracle stdout:
```
trapped
```

oracle stderr:
```
     2 *-* signal on syntax name th
     3 *-* raise syntax 40.4 description 'zdesc' additional 'zadd'
       >K>   "SYNTAX" => "40.4"
       >K>   "DESCRIPTION" => "zdesc"
       >K>   "ADDITIONAL" => "zadd"
     5 *-* th:
     6 *-* say 'trapped'
       >>>   "trapped"
```

### `y2.rex`

```
  1  trace r
  2  signal on user marker name th
  3  raise user marker description 'zdesc' return 'zret'
  4  say 'x'
  5  th:
  6  say 'trapped'
```

oracle stdout:
```
(empty)```

oracle stderr:
```
     2 *-* signal on user marker name th
     3 *-* raise user marker description 'zdesc' return 'zret'
       >K>   "DESCRIPTION" => "zdesc"
       >K>   "RESULT" => "zret"
```

### `y3.rex`

```
  1  trace r
  2  signal on halt name th
  3  call sub
  4  say 'x'
  5  sub:
  6  raise error 7 return 'zret'
  7  th:
  8  say 'trapped'
```

oracle stdout:
```
(empty)```

oracle stderr:
```
(empty)```

---

# Fix round 1

Commit `431e2698`, on top of `f906aabc`. Everything in section 0-9 above is
unchanged and still holds; this section records what the review found, what I
verified independently, and what changed.

Probes for this round ran from a fresh directory of their own,
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/r1`,
same wrapper and same three separate descriptors.

## Gate numbers

| gate | before | after |
|---|---|---|
| `cargo test --workspace` | 956 / 0 | **964 passed / 0 failed** |
| `REXX_CORPUS_GATE=1` corpus | `38 of 38` | **`38 of 38`** |
| assertions | 4224 / 4259 | **4224 / 4259** |
| `cargo fmt --all --check` | 0 | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | **0** |

`rexx-exec`'s lib tests went 277 -> 285: eight new, one per finding that
needed one.

## Every finding reproduced before it was fixed

I re-ran each one rather than taking the review's word for it. All reproduced
exactly:

| finding | probe | oracle | ours (before) |
|---|---|---|---|
| 1(a) dropped on `RETURN` | `pa2` | `end mark= HANDLER-AT 7` | `end mark= NOMARK` |
| 1(b) delivered into a later activation | `pa` | `HANDLER-AT 8` | `HANDLER-AT 11` |
| 2 `active_condition` never cleared | `pc` | `98.918`, rc 158 | silence, rc 0 |
| 2's other half (must **not** clear) | `pf` | 42.3, rc 214 | MATCH already |
| 5 `RAISE SYNTAX` validation | 8 programs | see below | 5 DIFF |

## Finding 1: two mechanisms, not one

The review asked for the mechanism rather than the two call sites. Working it
through produced two independent defects wearing one coat, and **a third shape
the review did not reach**, which is what proved the second mechanism
necessary.

**(i) Placement.** `deliver_pending_trap` was called at the bottom of
`run_activation`'s loop body, past a `match` two of whose arms `return`. It
now runs immediately after the clause finishes, *before* the `Flow` dispatch.
One site, and no path between "the clause has finished" and the check -- which
is the property the review asked for, and also the honest reading of what a
clause boundary is: `Flow::Return` means the clause *was* a `RETURN`, not that
it did not happen.

The `ObjRef` a `Flow::Return`/`Flow::Exit` carries is now rooted across the
handler. Its one-clause temps frame is already popped by then and the handler
is a whole nested activation that allocates -- the same window the `Exit`
arm's own comment describes, but with arbitrary user code inside it.

**(ii) Identity.** `PendingTrap` named its target by `activations.len()`. A
depth is unique only while its activation is live. `Activation` now carries an
`ActivationId`, minted from an `Interp` counter and never reused, and delivery
requires an exact match.

**(iii) The shape that needed (ii), which the review did not have.** Probe
`ps`: a pending condition whose activation is unwound by an error the *caller*
traps, after which the caller calls something else and lands at the dead
activation's depth.

```rexx
 1  signal on syntax name sh
 2  call on user foo name uh
 3  zmark = 'NOMARK'
 4  call aa
 5  say 'end mark=' zmark
 6  exit
 7  aa:
 8  signal off syntax
 9  zq = bb() + 1/0
10  return
11  bb:
12  raise user foo return 'BBVAL'
13  cc:
14  say 'in cc'
15  return
16  uh:
17  zmark = 'HANDLER-AT' sigl
18  return
19  sh:
20  say 'SYNTAX at' sigl
21  call cc
22  say 'after mark=' zmark
23  exit
```

Oracle: `SYNTAX at 4` / `in cc` / `after mark= NOMARK` -- the condition is
dropped. Before the identity fix we printed `after mark= HANDLER-AT 13`,
having run the handler inside `cc`. This is the only shape that separates the
two mechanisms, because here the activation never reaches another clause
boundary at all, so no amount of moving the check helps.

**Which test pins which, measured.** The two mutations kill exactly one test
each:

| mutation | `..._when_the_trapping_clause_is_a_return` | `..._into_a_later_activation_at_the_same_depth` | `..._whose_activation_is_gone_is_never_delivered` |
|---|---|---|---|
| revert the placement | KILLED | KILLED | survives |
| degrade identity to a depth | -- | survives | KILLED |

The middle column's survival is not a gap and is stated in that test's own doc
comment: once the check runs at `aa`'s `return bb()` boundary, the condition
is consumed before `cc` is ever called, so a depth suffices for *that* shape.
The third test is the one that cannot be satisfied by a depth.

## Finding 5: the oracle's actual validation rule, measured

Rather than only failing loudly, I measured what the oracle does. Nineteen
`raise syntax <arg>` programs:

```text
40.4 / 40 / 40.001 / 99 / 3 / 4 / 5 / 9 / 10 / 3.1 / 10.1 / 98.941  -> the catalogue entry
40.10 -> 98.941 found "40010"      3.5  -> 98.941 found "3005"
40.999 -> 98.941 found "40999"     88.1 -> 98.941 found "88001"
26.1  -> 98.941 found "26001"      99.5 -> 98.941 found "99005"
1     -> 98.941 found "1.0"        2    -> 98.941 found "2.0"
1.1   -> 98.941 found "1.1"        2.1  -> 98.941 found "2.1"
0 / 0.5 / 100 / 999 / 'abc'        -> 33.904, rc 223
```

The rule: **the major must be 1..=99**, else `33.904`; the sub is the digits
after the point read as a plain integer (`.001` is 1, `.10` is 10, measured
through `40.001` rendering `(40, 1)`); a pair the catalogue does not know is
`98.941` whose `&1` is `major * 1000 + sub`, **except** where the catalogue has
no `(major, 0)` entry at all, where it is the original `major.sub`.

That last clause is not fitted to two odd rows. I looked the majors up:
`lookup(1, 0)` and `lookup(2, 0)` are both `None`, and every other major in
1..=99 that I probed has an entry. "The major itself is unknown" and "the
major is known but this sub is not" are genuinely two cases and they render
differently.

This closes all three global-constraint violations -- the `<no message ...>`
placeholder at rc 216, the unrelated catalogue entry at rc 25, and `Error 0`
at **rc 0**. `raise_syntax_validates_its_argument` asserts, per row, both the
condition and that the exit code is not 0.

## Findings 2, 3, 4, 6, 7

* **2.** `deliver_pending_trap` clears `active_condition` in its
  `Ended::Returned` arm only. `pf` is why it is not symmetric: a `SIGNAL ON`
  handler that runs on must still find its condition. Both directions have a
  test.
* **3.** Both untested behaviours now have one:
  `a_call_trap_is_put_back_after_its_handler_returns` and
  `raise_propagate_of_an_unreportable_condition_ends_the_program_silently`.
  The second also asserts that no `Error 0` reaches stderr, which is the
  failure mode the review reached by accident.
* **4, 6, 7.** Corrected, not hedged: `run_source` no longer claims an empty
  `slots` map (`run_activation` passes `&plan.by_symbol`, and the coverage
  shift is stated as the improvement it is); `Delivery` names all three
  writers, `offer_to_trap`'s rewrite included; `pending_trap` no longer
  asserts the guarantee finding 1 broke, and now states the two properties
  the fix actually establishes.
* **`exec_raise_propagate`'s residual paragraph** is replaced by what was
  measured, and quotes the sentence it replaces.

## Concern 2 withdrawn

`pb` re-run here: `signal on user foo` in the main body, `signal off user foo`
in `lev1`, `raise user foo return` in `lev2`. **MATCH** -- the oracle does not
propagate to the grandparent either; `lev1` simply resumes. `Search::Caller`
stopping at the caller is measured behaviour, and its doc comment says so now.
Report Concern 2 is withdrawn.

## Concerns after this round

**1. `RAISE PROPAGATE`'s report format still rests on four transcripts.**
Unchanged from before; no new evidence either way.

**2. The pre-existing `EXIT <expr>` trace gap is unchanged** and still
constrains `trace r` corpus witnesses. Not this task's.

**3. A pre-existing parse-error divergence turned up while probing finding 5
and is not finding 5.** `raise syntax -1` is rc 221 with a clause echo on the
oracle and rc 120 loud here -- but so is `say 1 +`, which contains no `RAISE`
at all (probe `pw`). It is `execute`'s documented "parse errors are
deliberately not reproduced byte for byte" arm, and it is the only DIFF left
among this round's 42 probes.

Full regression: **39 of 42** fix-round probes match (the three are `pw`,
`pz`, `rt7`, all the parse-error class above), and **78 of 85** round-0 probes
match, unchanged -- `condition()` (4c), `ADDITIONAL (a, b)` (Phase 5), and the
`EXIT` trace gap.

---

# Fix round 2

Commit `26da3ac6`, on top of `431e2698`. Probes from
`.../scratchpad/r2`, same wrapper, three separate descriptors.

## Gate numbers

| gate | before | after |
|---|---|---|
| `cargo test --workspace` | 964 / 0 | **968 passed / 0 failed** |
| `REXX_CORPUS_GATE=1` corpus | `38 of 38` | **`38 of 38`** |
| assertions | 4224 / 4259 | **4224 / 4259** |
| `cargo fmt --all --check` | 0 | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | **0** |

`rexx-exec` lib tests 285 -> 289. All five findings reproduced before being
fixed; all nineteen new probes MATCH after.

## NEW 5, and who owns it

The re-review established the behaviour is identical on `f906aabc` and
concluded it is pre-existing. That is true and it does not make it someone
else's: **`f906aabc` is Task 7.** `git show 7ec66f84:...lib.rs | grep -c
pending_trap` is **0**; at `f906aabc` it is 2. The whole pending-delivery
mechanism arrived with this task, so "pre-existing on `f906aabc`" means
"present since Task 7 shipped". It is mine, and I took the lead's first
option.

**The fix is the mechanism, not a second copy of it.** The delivery rule now
lives in one function, `Interp::clause_boundary`, called from both places that
step a clause. There are exactly two, and that is the checkable part: `grep -n
"step_in_temps_frame("` outside its own definition and doc comments returns
`run_activation`'s loop and `run_bounded`'s loop, and nothing else.
`run_bounded` is where a `DO` body, a `WHEN`/`THEN` body and an `INTERPRET`
fragment execute, so a rule applied only in `run_activation` held for none of
them.

| probe | oracle | before | after |
|---|---|---|---|
| `do2` -- `call sub` then `say` in a 2-pass `DO` | handler after each `call`, `SIGL` 4 | `NOMARK` twice, then `HANDLER-AT 5` | MATCH |
| `sel1` -- the same inside `WHEN … DO` | `HANDLER-AT 5` | `NOMARK`, then `HANDLER-AT 6` | MATCH |
| `int1` -- inside an `INTERPRET` fragment | `HANDLER-AT 3` | `NOMARK` | MATCH |

The rooting `push_temp` moved into `clause_boundary` with the rule, so both
callers get it; the re-review's stress-mode control (deleting it panics `a
live value` on both the `Return` and `Exit` arms) therefore still covers both
call sites rather than one.

The two comments that asserted the property are now true of the code, and
`pending_trap`'s doc says *why* it is checkable: two callers, both named.

## NEW 1: restore, don't clear

One clause can queue a `CALL ON` condition **and** raise a `SIGNAL ON`-trapped
one -- `zq = sub() + 1/0` -- so a call handler can be delivered while a signal
handler is running. Round 1's `= None` wiped the condition the enclosing
handler was running for.

| | `raise propagate` in the `SIGNAL` handler |
|---|---|
| oracle | `42.3`, rc 214 |
| round 0 | silence, rc 0 |
| round 1 | `98.918`, rc 158 |
| **round 2** | **`42.3`, rc 214** |

`self.active_condition.take()` before the handler, restored in the
`Ended::Returned` arm -- and in the `Err` arm too, which is unmeasured and
stated as such at the code: no probe constructs a failing `CALL ON` handler
under an outer trap, and restoring is done there because the alternative is
exactly the state the `Returned` arm exists to prevent.

The adjacent case is what stops this being a disguised "never clear":
`restoring_none_is_still_the_common_case` and round 1's own
`a_returned_call_handler_leaves_no_active_condition_to_propagate` both still
give `98.918` when nothing was active.

## NEW 2 and NEW 4: the bound and the number syntax

The sub is bounded at `0..=999`, and each half of the argument goes through
`Number::parse` then `whole_value` -- the oracle's `numberValue`, not Rust's
`str::parse::<i64>`. A decimal point with an empty tail is rejected outright,
where no decimal point at all means sub 0.

```text
40.1000 / 40.1001 / 40.99999 / '40.'   -> 33.904 rc 223   (were 98.941 / Error 40)
'4E1'                                  -> (40, 0) rc 216  (was 33.904)
'40.1E2'                               -> 98.941 "40100"  (was 33.904)
40.999 / '+40' / 40.0                  -> unchanged, MATCH
```

The `u32` comment that asserted the opposite for `40.99999` **by name** is
gone rather than hedged.

## NEW 3: a claim that was asserted rather than counted

"Majors 1 and 2 are the only ones in 1..=99 with no `(major, 0)` entry" is
false in both places it appeared; there are **45** (1, 2, 12, 32, 50-87, 94,
95, 96). The code was always right -- it performs the lookup rather than
hard-coding a pair -- so only the comments changed, and the test now carries
`50.1` and `87` so the row set cannot be read as describing a two-element
special case. The report's own repetition of the claim is corrected by this
section.

This one is worth naming for what it was: a measured-sounding claim I did not
measure, in the round whose whole subject was measuring. Two probes agreeing
is not a count.

## Mutation checks

| mutation | test(s) | result |
|---|---|---|
| `run_bounded` loses its clause boundary | the `DO` and the `WHEN`/fragment tests | KILLED |
| the interrupted condition is cleared, not restored | `a_call_handler_restores_the_condition_it_interrupted` | KILLED |
| nothing is cleared at all (round 0) | `restoring_none_...`, `a_returned_call_handler_...` | KILLED |
| the `0..=999` sub bound dropped | `raise_syntax_validates_its_argument` | KILLED |
| Rexx number parsing replaced by `str::parse::<i64>` | same | KILLED |

## Regression

19 of 19 this round's probes MATCH. 39 of 42 round-1 probes (the three are
`pw`/`pz`/`rt7`, the pre-existing parse-error class -- `say 1 +` reproduces it
with no `RAISE` in it). 78 of 85 round-0 probes, unchanged: `condition()`
(4c), `ADDITIONAL (a, b)` (Phase 5), and the pre-existing `EXIT <expr>` trace
gap.

Both `r1` and `r2` are shared with a sibling agent that writes its own probes
into them; the counts above are over my own files only, listed explicitly
rather than globbed.

## Concerns after this round

1. **`RAISE PROPAGATE`'s report format** still rests on the same four
   transcripts. Unchanged.
2. **The `Err` arm of `deliver_pending_trap`'s restore is unmeasured** -- a
   `CALL ON` handler that fails under an outer trap. Stated at the code.
3. **The pre-existing `EXIT <expr>` trace gap** is unchanged and still
   constrains `trace r` corpus witnesses.
4. **The pre-existing parse-error reporting gap** (rc 120 against the
   oracle's rc 221 with a clause echo) is unchanged; it is `execute`'s
   documented arm, not this task's.

---

# Fix round 3

Commit `f9e6bf26`, on top of `26da3ac6`. Probes from `.../scratchpad/r3`.

| gate | before | after |
|---|---|---|
| `cargo test --workspace` | 968 / 0 | **970 passed / 0 failed** |
| `REXX_CORPUS_GATE=1` corpus | `38 of 38` | **`38 of 38`** |
| assertions | 4224 / 4259 | **4224 / 4259** |
| fmt / clippy | 0 / 0 | **0 / 0** |

## NEW-A: removing the enumeration instead of extending it

Three rounds each fixed the sites they knew about and each asserted an
exhaustiveness that was false a round later. The lead asked for a construction
rather than a fourth entry, and there is one.

**`ClauseState` moved to `clause.rs`.** A private field on a crate-root struct
is visible to `crate::run`, because a child module sees its ancestors' private
items; a private field on a struct in a *sibling* module is not. So the line
field is now unreachable from `run.rs`, and the only way to set it is
`Interp::enter_clause`, which hands back a `#[must_use] ClauseToken` that only
`Interp::end_clause` consumes -- and `end_clause` is what delivers a pending
handler.

**Measured, not asserted.** Deleting the `end_clause` call:

```text
error: unused variable: `token`
    --> crates/rexx-exec/src/run.rs:3452:13
     |
3452 |         let token = self.enter_clause(line);
     |             ^^^^^ help: if this is intentional, prefix it with an underscore: `_token`
error: could not compile `rexx-exec` (lib) due to 1 previous error
```

exit 101 under the clippy gate. A clause entered and never ended does not
build. It does **not** stop `let _token`, and rustc's own message advertises
that escape; nothing short of a scoped-closure API would close it. That is in
the module doc, not hidden.

**Why binding the two was the right axis.** The two symptoms were always one
omission: `do while zn < sub()` reported `SIGL` from a clause three lines away
*and* delivered its handler at the wrong moment, because the site that fails
to say "a new clause is starting" is the same site that fails to run what a
boundary owes. One call now fixes both.

Sites reached that no round enumerated: the `DO` header, the `WHILE` re-test,
and the **`UNTIL` re-test** -- which the re-review did not name either, and
which I found by probing the family rather than the two reported members.
Which clause a re-test belongs to also moves: the `DO` clause on the first
pass, the `END` clause after it (`SIGL` 4 then 7, measured).

| probe | oracle | before | after |
|---|---|---|---|
| `g1` `do i = 1 to sub()` | `HANDLER-AT 3`, before the body | `NOMARK`, then 4 after `END` | MATCH |
| `g4` `do while zn < sub()` | 4 then 7 | 5 then 5 | MATCH |
| `g5` `do until zn >= sub()` | 7 | 5 | MATCH |

**Where the rule lives now.** The boundary is inside `step_in_temps_frame`, so
`run_activation` and `run_bounded` get it without being listed. One rule
genuinely needs its own site: a clause that *failed* has not completed, and
whether its handler is owed depends on whether the failure is trapped here or
unwinds the activation. That is decided in exactly one place, `offer_to_trap`,
and the delivery goes there -- measured both ways (`ac1` delivers, `ps` does
not).

`ClauseEnd::Completed`/`Failed` replaced an `Option<&Flow>` that conflated "no
value" with "did not complete"; under the `Option` the error path started
delivering handlers it must not, caught immediately by
`a_pending_trap_whose_activation_is_gone_is_never_delivered`.

## NEW-B, C, D, E

* **B.** A handler that fails at a clause boundary is blamed on that clause
  (`3 *-* call sub`), not the enclosing `DO`. The no-trap control `f1` was
  already correct and stays correct.
* **C, D.** Both fell out with the restructure: `clause_boundary` is gone, so
  its fused doc comment is gone and `deliver_pending_trap` has its own again;
  `run_bounded` no longer synthesises a `Flow`, so its "returns the escaping
  `Flow` unchanged" is true once more.
* **E.** The weaker duplicate test is removed.

## The `Err` arm, decided

Reachable, and unobservable because every path that reads `active_condition`
passes through `offer_to_trap`, which overwrites it. Kept rather than deleted:
the alternative is not "nothing" but a wrong value that happens not to be
read, and it would become observable the day a reader does not go through
`offer_to_trap`. Said at the code, no longer described as unmeasured.

## Mutations

| mutation | result |
|---|---|
| the loop header does not enter a clause of its own | KILLED (by the `WHILE` test) |
| the re-test stays on the `DO` clause | KILLED |
| a failing boundary handler blamed on the enclosing instruction | KILLED |
| a failing clause delivers its pending handler | KILLED |
| the trap-resume boundary removed | KILLED |

The first survives `a_loop_header_is_a_clause_with_its_own_boundary` and is
killed by its sibling: that test pins the *boundary*, not the header's line,
because the `DO` instruction's own step has already set line 3. Both tests'
comments now say which property each one holds, stated because the mutation
said so rather than assumed because they look related.

## Regression

`g1`/`g4`/`g5`/`nb`/`f1` MATCH. 39 of 42 round-1 probes, 19 of 19 round-2, 78
of 85 round-0 -- all unchanged, and the ten misses are the same three
pre-existing classes: `condition()` (4c), `ADDITIONAL (a, b)` (Phase 5), the
`EXIT <expr>` trace gap, and the parse-error reporting arm.

---

# Fix round 4

Commit `9a4b57be`, on top of `f9e6bf26` (a new commit, not an amend -- the
re-review references that hash). Probes from `.../scratchpad/r4probe`, a directory
`mkdir`ed for this round and holding only my own files; oracle wrapper
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`,
stdout, stderr and exit status as three separate descriptors, absolute paths
for every redirect. A/B against a detached `git worktree` at `f9e6bf26` with
its own `rexx-run`, so "regression or pre-existing" is a measurement.

| gate | before | after |
|---|---|---|
| `cargo test --workspace` | 970 / 0 | **976 passed / 0 failed** |
| `REXX_CORPUS_GATE=1` corpus | `38 of 38`, STRICT | **`38 of 38`, STRICT** |
| assertions | 4224 / 4259 | **4224 / 4259** |
| `cargo fmt --all --check` | 0 | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | **0** |

Six new tests: five in `run.rs`'s own module, one in `collect_stress.rs`.

## The recurring error, and what was done about it instead of a fifth entry

Four rounds each shipped a fix that was right for the sites it knew about and
a *claim* that was false a round later. Round 3's move -- replace an
enumeration with a construction -- was right in kind, and its construction was
narrower than its documentation.

**The rule is now derived from the oracle rather than enumerated.** Read
directly: `RexxActivation::run`'s instruction loop calls
`processClauseBoundary()` after each `nextInst->execute()` returns
(`RexxActivation.cpp:642-654`). So a clause boundary sits **after every
instruction of the activation's own flat list**, and nothing else is one.
This crate diverges from that in exactly one way -- `IF`, `SELECT`,
`DO`/`LOOP` and `INTERPRET` resolve *other* instructions inside their own
`step` -- so "where does a clause boundary belong" stops being a list and
becomes a question with an answer: every construct that nests must end its own
header clause first, and `INTERPRET` is the one that must not, because there
the oracle runs the fragment in an activation of its own.

**And the answer is now checked at run time rather than asserted in prose.**
`Interp::in_clause` carries a `debug_assert`: a clause that begins at a
*different line* while a condition queued by an earlier clause of this same
activation is still waiting is the defect, and it now aborts the test suite
naming itself. Evidence that it is neither vacuous nor over-eager, RAN with
the round-3 code still in place for the four constructs:

| probe | shape | tripwire |
|---|---|---|
| `p2` | `if sub() = 'SV'`, `then` on the next line | **fires** (line 4 vs 3) |
| `p5` | a false `WHEN`, a later one wins | **fires** (5 vs 4) |
| `p7` | `select case sub()` | **fires** (4 vs 3) |
| `p8` | a false `WHEN` falling to `OTHERWISE` | **fires** (6 vs 4) |
| `p9` | a *true* `WHEN`, `then` on the next line | **fires** (5 vs 4) |
| `p1`, `p10`, `p11`, `p14`, `u7`, `n1` | the adjacent successes | silent |

`p9` is not in the re-review's list. The tripwire found it before I did.

It also fired on the first run against a shape that is **not** a defect -- a
`DO`'s control setup (`run_loop`) and its first header test (`run_repeating`)
are two clauses here and one instruction (`RexxInstructionControlledDo::
execute`) there. That is why the assertion compares lines rather than merely
checking for a waiting condition, and the exemption is stated at the code with
both cases it covers. What it therefore does **not** catch is a delivery that
is late in *time* but lands on the same line; that is written down where the
assertion is, rather than left as an implied guarantee.

## Important 1 -- NEW-1, the `ITERATE` regression, and a second divergence found with it

Reproduced first, all four byte-identical to the re-review's transcripts
(`u7` `2,4,4` vs ours `2,5,5`; `u8` `2,5,5` vs `2,7,7`; `u3` `2,4,6` vs
`2,6,6`; `u6` `2,5,8` vs `2,8,8`).

**The rule, from the oracle's own architecture rather than fitted to probes.**
`RexxInstructionBaseLoop::reExecute` is never called on its own -- it is
called *by* whichever instruction transfers control back to the loop, and
there are exactly three: the `DO`/`LOOP` clause on entry, `END`'s own
`execute` on a fall-through (`EndInstruction.cpp`, `LOOP_BLOCK` arm), and
`RexxActivation::iterate`. That instruction is still `current` while the
header runs, so it owns the re-test's line and its boundary. `HeaderClause`
(three variants) replaces round 3's `header_pass: bool`, and
`do_body_outcome` now answers `FellThrough`/`Iterated(line)`/`Escaped(flow)`
where it used to fold the first two into `Ok(None)` -- correct about the
control flow, wrong about the attribution, which is the whole finding.
The `ITERATE`'s line rides on `LeaveOrigin`, captured at the moment it steps
alongside `site` and `indent` and for the same reason.

**A second divergence, found while measuring the first.** `END` is not
executed at all when an `ITERATE` ends a pass, so it does not *echo* either.
RAN, `trace r` on `do while zn < 2 / zn = zn + 1 / iterate / end`: the oracle
prints `iterate` then `do while zn < 2`, with no `end` between them; we
printed `end`. Same root cause, same fix site, and no round or reviewer named
it.

Six probes now MATCH that did not: `u7`, `u8`, `u3`, `u6`, `n1` (`UNTIL` with
an `ITERATE`), `n3` (an inner loop's own `ITERATE`), plus `n6` (the trace
one), `q1` and `q7` (the delivery-timing twins). The adjacent successes `n2`
(nested inner loop, fall-through) and `n4` (a `CALL` as the last body clause)
were MATCH before and still are.

## Important 2 -- NEW-2, and a fifth site the re-review did not have

All four reproduced byte-identically. A fifth, `p9`, is the same family: a
**true** `WHEN` whose `then` is on the next line reported `SIGL` 5 where the
oracle reports 4.

Three sites now open a clause of their own, closed before anything nested
runs: `IF`'s condition, `SELECT`'s `CASE` expression, and each listed
`WHEN`/`WHEN CASE`'s condition (extracted to `scan_when` so the closure reads
as a named function). `INTERPRET` deliberately does not, and has a test that
fails if it is added by analogy.

The re-review's note that the single-line spellings "match only by
coincidence" is exactly right and is now load-bearing:
`a_single_line_then_reports_the_same_line_either_way` is the adjacent-success
test, and **its output is identical with and without the fix**. Measured: with
the per-`WHEN` clause removed it does fail, but on the tripwire, never on a
wrong value. That is the case a value assertion cannot see, and it is why the
tripwire is in the tree.

## Important 3 -- the obligation, strengthened; and what it still does not do

**Decision: strengthen, with the scoped-closure API the finding names.** The
token is gone. `Interp::in_clause(code, line, |it| …)` sets the line and runs
the boundary around a closure that *is* the clause. Everything the re-review's
A-table found escaping stops being writable rather than becoming caught: there
is no token to `let _token`, to `drop`, or to `mem::forget`, and an early
`return` or a `?` inside the closure returns from the closure with the
boundary still to come.

Re-attacked, each one built and its exit status read (`cargo build -p
rexx-exec --lib`, then `cargo clippy --workspace --all-targets -- -D warnings`,
then `cargo test -p rexx-exec --lib`):

| # | attack | build | clippy | tests | verdict |
|---|---|---|---|---|---|
| B1 | round 1's own Critical re-expressed -- `if matches!(flow, Ok(Flow::Return(_))) { return flow; }` before the boundary | 0 | 0 | **0** | **no longer an escape.** The only place it can be written is inside the closure, where it returns from the closure and the boundary still runs. Outside, the boundary has already run. Under round 3 the same three lines passed the gate and broke two tests |
| B2 | `self.clause_state = ClauseState::new()` from `run.rs` | 0 | 0 | 101 | **still expressible.** Nothing in the type stops it; this particular placement (inside `resolve_and_run_call`, so every call resets) is caught by tests, and that is a fact about the placement, not a guarantee -- see the residual below |
| B3 | `let saved = self.clause_state;` (round 3's own spelling) | **101** | 101 | -- | `E0507: cannot move out of self.clause_state … does not implement Copy` |
| B4 | forge a `SavedClauseState` in `run.rs` | **101** | 101 | -- | `E0603: tuple struct constructor SavedClauseState is private` |
| A1/A2/A3/A4/A13 | the five token spellings | -- | -- | -- | **not expressible**: there is no token |

**The residual, stated as bounds rather than as a guarantee**, and written in
`clause.rs`'s module doc in the same words:

* A site can decline to call `in_clause` at all. Nothing in the type system
  makes an instruction a clause. That is what the tripwire covers, and it
  covers it by behaviour, not by type.
* `run.rs` can reset the whole state to its zero value, because `Interp::new`
  needs `ClauseState::new` to be `pub(crate)` and Rust visibility cannot grant
  that to `lib.rs` while withholding it from `lib.rs`'s other children (B2).
  What it can no longer do is set the line to an *arbitrary* value: `ClauseState`
  is no longer `Copy`, the line field is private, and the save/restore
  `resolve_and_run_call` needs goes through `SavedClauseState`, which can only
  put back a value some `in_clause` produced.

Round 3's module doc said "`Interp::enter_clause` is the only way to set the
line", which was false. The replacement enumerates the two reachable spellings
and names which one is which.

Cost, measured rather than assumed: a 400,000-iteration loop with an `ITERATE`
runs 1.33/1.34/1.32 s on `f9e6bf26` and 1.34/1.35/1.35 s here -- about 1.5%,
and the `debug_assert` is absent from release entirely.

## Important 4 -- NEW-4, closed twice over

**By type**: the value the boundary roots now comes from the closure's own
return type through `ClauseValue`, so a site holding a `Flow` cannot choose
`None`. `ClauseEnd::Completed(Option<&Flow>)` is gone. The trait has **no
default method body**, so a new clause-body type has to answer the question
explicitly.

**By test**: `a_clause_value_survives_the_handler_its_boundary_runs`
(`collect_stress.rs`) runs three programs under
`run_program_collect_every_alloc` -- `return bb() || 'TAIL'` at an activation
boundary, the same inside a `DO` body, and the `EXIT` twin -- each asserting
its output and that it performed a non-zero number of collections. All three
MATCH the oracle. Mutation M7, `ClauseValue for Flow` returning `None`: all
three panic `a live value` at `value.rs:125:47`, **and the 296-test lib suite
stays green under the same mutation**, which is precisely the gap the
re-review found. The corpus subset does not contain the shape; that is why it
had no test before.

## Minors

* **NEW-5.** `lib.rs`'s `pending_trap` doc no longer names the deleted
  `clause_boundary` or the removed `run_activation` check, and no longer
  asserts the two-callers property. It states where the boundary lives, that
  the set of sites is derived rather than enumerated, and points at
  `clause.rs` for the rule and the residual.
* **NEW-6.** `deliver_pending_trap` returns `Option<HandlerExit>`, and
  `HandlerExit`'s only constructor is `from_ended(Ended) -> Option<HandlerExit>`,
  which answers `None` for `Ended::Returned`. So the invariant round 2
  announced with an `unreachable!` and round 3 lost is a type again -- and it
  is the *safe* direction rather than a louder failure: a future site that
  routes a `Returned` through it gets "the handler returned, carry on", never
  a `RETURN` rendered as an `EXIT`.
* **NEW-7.** Dissolved rather than answered. The `UNTIL` re-test's line and
  its boundary are one `in_clause` call, so "keep the line, drop the boundary"
  -- the mutation the re-review ran to show the boundary was inert -- is not
  expressible; dropping both is what `a_while_retest_belongs_to_the_do_clause_
  then_to_the_end_clause` fails on. The comment at that site says the boundary
  is currently unobservable **and why**: between the `UNTIL` test and the next
  top-of-loop test no user clause runs and nothing re-sets the clause line, so
  whichever of the two fires first delivers at the same line.

## Mutations

Every new test checked by deleting or inverting the line it protects.

| mutation | result |
|---|---|
| M1 an `ITERATE`-ended pass hands the re-test to `END` (round 3's rule) | KILLED `a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause` |
| M2 `END` echoes for an `ITERATE`-ended pass too | KILLED `end_does_not_echo_for_a_pass_an_iterate_ended` |
| M3 the `IF`'s condition is not a clause of its own | KILLED `a_construct_header_is_a_clause_with_its_own_boundary` |
| M4 `SELECT CASE`'s expression is not a clause of its own | KILLED the same |
| M5 a listed `WHEN`'s condition is not a clause of its own | KILLED that **and** `a_single_line_then_reports_the_same_line_either_way`, the latter on the tripwire rather than on a value |
| M6 `INTERPRET` ends its header clause before the fragment (the fix applied by analogy) | KILLED `an_interpret_clause_does_not_deliver_before_its_fragment_runs` |
| M7 `ClauseValue for Flow` roots nothing | KILLED `a_clause_value_survives_the_handler_its_boundary_runs` (`a live value`); the lib suite stays green |

## Regression

**63 probes, A/B against `f9e6bf26`: 17 DIFF -> MATCH, 43 MATCH on both, 3
DIFF on both, 0 MATCH -> DIFF.**

The three are one pre-existing class, confirmed DIFF on `f9e6bf26` too and
found in passing rather than introduced here: under `trace r`, a **controlled**
`DO`'s per-pass re-execution emits two `>>>` intermediate-value lines for the
control variable (its old and new values) that this crate does not produce --
`q11` (`do i = 1 to 2`), `z6` and `y4`. `y4` improved from three diffs to two:
its spurious `END` echo after an `ITERATE` is gone. Not this task's, and not
touched.

Coverage of what the round changed, all MATCH: the loop family (`q1`-`q11`,
`u1`-`u8`, `n1`-`n7`, `z1`-`z9`), the `IF`/`SELECT` family (`p1`-`p14`,
`y1`-`y3`), failure attribution in a loop header, a `WHEN`, a `SELECT CASE`
and an `IF` (`x1`-`x6`), the trap families the earlier rounds established
(`e5`/`e6`/`e7`/`f1`, `o1`/`o3`/`o4`), and the three rooting shapes
(`sr1`/`sr3`/`sr4`).

## Concerns after this round

1. **The tripwire is a `debug_assert`**, so it is absent from release builds
   and from `rexx-run` as shipped. It runs under `cargo test`, which includes
   the corpus gate and the assertions harness, so its evidence is 976 tests +
   38 corpus programs + 4259 assertion rows + 63 probes -- but a shape that
   only ever occurs in a release run is not covered.
2. **It catches the wrong-`SIGL` half only.** A delivery that is late in time
   but lands on the same line is invisible to it, and to `SIGL`. Stated at the
   code.
3. **A pre-existing `TRACE R` gap**: the control-variable `>>>` lines a
   controlled `DO` emits on re-execution (item above). Three of this round's
   probes see it; it is unrelated to conditions.
4. **`RAISE PROPAGATE`'s report format** still rests on the same four
   transcripts. Unchanged from rounds 1-3.
5. **The pre-existing `EXIT <expr>` trace gap and the parse-error reporting
   arm** are unchanged.

## Fix round 5

Scope: `task-7-rereview4.md`'s three new findings (NEW-1, NEW-2, NEW-3), all
labelled MINOR and all the same shape as every prior round's defect -- a
comment asserting an exhaustiveness or a fact that was false. Comment-only;
no executable code touched. Commit `1663538a`, on top of `9a4b57be`.

### NEW-1 -- `run.rs`, the `UNTIL` re-test's `in_clause` call

**Before:**

> dropping both is what `a_until_retest_reports_the_end_clauses_line` fails
> on.

`grep -rn "a_until_retest" crates/` found only that comment; no such test
exists. **Verified**: reproduced the M-U mutation from re-review 3/4 by hand
(dropped the whole `in_clause` call at this site and evaluated the `UNTIL`
condition directly, no line set, no boundary run), then ran
`cargo test -p rexx-exec --lib -- a_while_retest_belongs_to_the_do_clause_
then_to_the_end_clause a_loop_retest_after_an_iterate_belongs_to_the_
iterate_clause --test-threads=1`. Both fail under the mutation (one on its
own value assertion, the other on the fourth-site tripwire at the next
`in_clause` call). Reverted the mutation, confirmed `git status --porcelain`
empty before re-editing.

**After:**

> dropping both is what `a_while_retest_belongs_to_the_do_clause_then_to_
> the_end_clause` and `a_loop_retest_after_an_iterate_belongs_to_the_
> iterate_clause` fail on.

### NEW-2 -- `clause.rs:245-250`, the `ClauseValue` trait doc

**Before:**

> Here the value comes from the closure's own return type, so the site
> cannot choose.

**Verified false** (C1 from `task-7-rereview4.md`, reproduced by hand at
`step_in_temps_frame`'s own `in_clause` call, `run.rs`): computed the `Flow`
inside the closure, wrote it to a captured `&mut Option<Flow>`, and returned
`Ok(())` instead -- landing on `ClauseValue for ()`, which roots nothing.
`cargo build -p rexx-exec --lib` exit 0; `cargo clippy --workspace
--all-targets -- -D warnings` exit 0; `cargo test -p rexx-exec --lib` exit 0,
296 passed / 0 failed; `cargo test -p rexx-exec --test collect_stress`
**exit 101**, `a_clause_value_survives_the_handler_its_boundary_runs` fails,
panicking `a live value` at `value.rs:125:47`. Reverted; `git status
--porcelain` empty afterward.

**After:**

> Here the value comes from the closure's own return type, which narrows the
> *shape* a site can hand back -- but does not decide the question by
> itself. A site can still compute the `Flow` inside the closure, write it
> to a captured `&mut Option<Flow>`, and return `Ok(())`, landing on
> `ClauseValue for ()` and dropping the rooting the same way `Completed(None)`
> did: measured, build 0, clippy 0, all 296 lib tests green, and it is
> `a_clause_value_survives_the_handler_its_boundary_runs`
> (`tests/collect_stress.rs`) that catches it, panicking `a live value`. The
> type narrows what a site can return; that test is what actually pins the
> rooting.

### NEW-3 -- `clause.rs:97-98`, the module doc's residual enumeration

**Before:**

> **`run.rs` can reset the whole state to its zero value** --
> `self.clause_state = ClauseState::new()` compiles, because `Interp::new`
> needs `ClauseState::new` to be `pub(crate)` and Rust visibility cannot
> grant that to `lib.rs` while withholding it from `lib.rs`'s other
> children. What it *cannot* do any more is set the line to an arbitrary
> value: `ClauseState` is no longer `Copy`, the line field is private, and
> the save/restore `resolve_and_run_call` needs goes through
> [`SavedClauseState`], which can only put back a value some `in_clause`
> produced. So the reachable spellings are `in_clause` and "reset to line
> 0", and the second one is loud in its own right (`SIGL` 0 is not a line).

The sentence "What it *cannot* do any more is set the line to an arbitrary
value" is false, and the two-item enumeration built on it is short by at
least one. **Verified false** (C3 from `task-7-rereview4.md`, reproduced by
hand): wrapped `run_bounded`'s own `step_in_temps_frame` call in a
save/restore pair -- `let stale = self.save_clause_state(); let flow =
self.step_in_temps_frame(...)?; self.restore_clause_state(stale);` -- which
restores the state from *before* the step rather than the one it produced.
`cargo build -p rexx-exec --lib` exit 0; `cargo clippy --workspace
--all-targets -- -D warnings` exit 0; `cargo test -p rexx-exec --lib` exit 0,
296 passed / 0 failed. The stale line lands with no boundary at that moment
and nothing in the lib suite sees it. Reverted; `git status --porcelain`
empty afterward. Independently confirmed (by reading, not by a fresh
mutation) that `deliver_pending_trap` is also `pub(crate)`
(`run.rs:2544`) with a second call site outside `in_clause`
(`run.rs:2522`, the failed-clause boundary), which is the mirror gap the
re-review named: a boundary can run with no line set to pair it, for the
same visibility reason.

Rather than append a third bullet and risk being short by one again -- the
exact failure mode this task repeated four times running -- the bullet was
rewritten around the property that makes the list open in the first place
(this module cannot narrow its `pub(crate)` surface below what its own
legitimate callers need), with the two concrete instances named as examples
and an explicit statement that the list is what's reachable *today*, not a
proof that nothing else is:

**After:**

> **What this module has to expose to its own legitimate callers is also
> what a caller could misuse, and that is not closed by a type the way the
> "two halves came apart" family above is.** `self.clause_state =
> ClauseState::new()` compiles, because `Interp::new` needs
> `ClauseState::new` to be `pub(crate)` and Rust visibility cannot grant
> that to `lib.rs` while withholding it from `lib.rs`'s other children --
> that reset is loud in its own right (`SIGL` 0 is not a line).
> `save_clause_state`/`restore_clause_state` are `pub(crate)` for the same
> reason, so `resolve_and_run_call` can put a callee's caller state back
> after the callee returns; nothing stops `run.rs` from restoring a *stale*
> [`SavedClauseState`] at a moment other than the one it was taken from,
> which sets a nonzero line with no boundary attached at that moment --
> measured: builds, passes clippy, and passes all 296 lib tests, undetected.
> `deliver_pending_trap` is `pub(crate)` for the mirror reason (a failed
> clause's own boundary runs from `offer_to_trap`, not from `in_clause`),
> and nothing in the type system stops it running a boundary paired with no
> line set at all. **This is what is reachable through this module's own
> `pub(crate)` surface today, not a proof that nothing else is** -- the
> property behind all three is that a function this module must expose for
> one legitimate caller is a function every other `pub(crate)` caller can
> also reach.

### Gates re-verified after all three edits, each exit status read unpiped

* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0.
* `cargo test --workspace` -- exit 0, **976 passed / 0 failed** (summed
  across all 68 `test result:` lines in the run).
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` -- exit 0,
  mode STRICT, **38 of 38 matching**.
* `cargo test -p rexx-exec --test assertions` -- exit 0, **4224 of 4259**.

All five numbers identical to fix round 4's baseline. Working tree clean
before the commit except the two intended files; nothing else staged.
