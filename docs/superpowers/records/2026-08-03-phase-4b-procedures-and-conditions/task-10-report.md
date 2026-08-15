# Task 10 report -- the 4b corpus and the collector

**Status: DONE.** Commit `3d6d68d7` (`git log`: `3d6d68d776a2df8d754c601c28c4113c4b046256`,
"4b Task 10: three combination witnesses, and the I19/I20 sweeps"), on top of `598be610`.

Gates, each run from `rust/` with its exit status read unpiped:

| gate | result |
|---|---|
| `cargo test --workspace` | **990 passed / 0 failed** (unchanged -- no new `#[test]` fns; the corpus/oracle growth is read by existing data-driven tests) |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | **44 of 44**, mode STRICT (was 41 of 41) |
| `cargo test -p rexx-exec --test assertions` | **4224 / 4259** (unchanged) |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |

Corpus count: the STRICT differential gate moved from 41 of 41 to 44 of 44
(`corpus/phase-4a.txt` unchanged at 30 non-comment lines; `corpus/phase-4b.txt`
grew from 11 to 14). The broader `find corpus -name '*.rex' | wc -l` count
(every `.rex` file under `rust/corpus/`, all phases) moved from 61 to 64.

---

## Pre-dispatch corrections, followed as given

1. I did not rebuild I20's union half (`collect_stress.rs:127-130` already
   read the union before I touched anything -- confirmed by running
   `cargo test -p rexx-exec --test collect_stress` as the very first command
   of this task, before any edit: 2 passed, 0 failed, against the union as it
   stood then). What I did was the open half: confirm the stress mode
   survives real call-frame combinations and report the result (Section 3
   below).
2. `assert_program_has_no_directives` is at `tests/coverage.rs:153` -- verified
   before relying on it; I did not touch it, and no new program needs a
   directive.
3. The three new programs went into `rust/corpus/lang/`, joining the eleven
   existing 4b programs there. I found no reader keying on a `proc/`
   subdirectory and no reason to split one phase across two directories --
   confirmed by grep before writing anything.
4. Read `heap.rs:81` before starting; its own doc comment is what Section 3
   below is checked against.

---

## 1. The three combination programs, and the mechanism each pins

Every construct already had a single-purpose witness before this task
(`every_in_scope_variant_is_witnessed_by_the_phase_subsets` passed on the
pre-existing 41-program union, unchanged since Task 9), so nothing here adds
a fourth single-construct file -- all three are combinations, satisfying "at
least three that combine two" as the actual deliverable rather than a floor
under a larger batch. Each was run against the live oracle from a fresh
directory (`( ulimit -v 1048576; LD_LIBRARY_PATH=... rexx FILE )`), matched
**byte for byte, unnormalised** (a raw `diff`, not merely equal after
`corpus.rs`'s own stderr-indent normalisation), and matched `rexx-run`
identically. Each was then checked against a real, minimal, one-site
mutation of `rexx-exec/src/run.rs`, built, run, observed to diverge, and
reverted by copy (`run.rs.orig.bak`, an `md5sum`-verified byte-identical
restore, never `git checkout --`); `git diff --stat` on `run.rs` was empty
before the commit above.

### `lang/raise_in_routine_in_loop.rex` -- PROCEDURE (Task 5) + RAISE/CALL ON (Task 7) inside a loop

`looper` is called from every pass of a three-pass loop and raises a trapped
USER condition on the middle pass only. Oracle-matched output:
`accum: 10:1 [H@5]TRAPPED:2 30:3`.

**Mechanism pinned:** `exec_raise`'s `Some(trap) if trap.call` arm
(`run.rs:3031-3042`) delivers the `RAISE ... RETURN` value as `Flow::Return(result)`,
which is what lets it reach `RESULT` at the *calling* clause. **Mutation:**
changed that line to `Ok(Flow::Return(None))`. Rebuilt, reran: the middle
segment reads `RESULT:2` (an unset variable's own derived name) instead of
`TRAPPED:2`; every other segment (the un-raised first and third passes, the
handler's own `[H@5]` segment) is byte-identical, which is what pins the
divergence to the value RAISE...RETURN carries across the call boundary
rather than to the surrounding loop or trap machinery. Reverted; `run.rs`
confirmed byte-identical to the pre-mutation copy.

The program also pins, without a separate mutation (both are load-bearing by
construction, not decoration): the routine's own remaining instruction
(`return zn * 10`, right after the `RAISE`) does not run on the trapped pass
-- visible because the middle segment is `TRAPPED:2`, not `20:2`; and the
loop's own state survives the trap delivery -- visible because the third
pass still computes `30`, not a corrupted `zi`.

### `lang/expose_stem_across_calls.rex` -- PROCEDURE EXPOSE (Task 5) of a stem, across three separate calls

`bump` is called once per loop pass; each call both reads the running total a
previous call left in `ST.SUM` and writes a new tail. The third call's
addition crosses 10 and raises a trapped USER condition whose handler writes
`ST.FLAG` through the same exposed table. Oracle-matched output:
`after loop: 1 4 9 14 H@5`.

**Mechanism pinned:** the `self.roots.alias_slot(inner, *slot, *target)`
loop inside `exec_procedure` (`run.rs:2000-2002`), which is what makes the
callee's exposed slot literally the caller's slot rather than a copy.
**Mutation:** emptied that loop's body. Rebuilt, reran: the program no
longer prints anything -- it dies on the **first** call already, before the
loop or the trap gets a chance to matter, with `Error 41.1: Nonnumeric value
("ST.SUM")` at rc 215, because `st. = 0`'s default lives only in the
caller's own table and the un-aliased callee's `ST.SUM` reads its own
derived name. (My header comment first stated this as a third-call failure
by reasoning rather than running it; running the mutation corrected that to
"fails on the first call" before it landed -- see `rust/CLAUDE.md`'s own
Method section on why running beats reasoning here.) Reverted; confirmed
clean.

### `lang/call_on_trap_rearms.rex` -- SIGNAL to a label (Task 6) between two raises of one CALL ON-trapped condition (Task 7)

`call on user zx name h` is armed once. `raiser` is called, raises the
condition, the handler fires and returns (resuming the interrupted clause),
a `SIGNAL skip` jumps over an unreachable segment, and `raiser` is called
**again** with no re-arming instruction anywhere. Oracle-matched output:
`S[H4]1:VK[H10]2:V`.

**Mechanism pinned:** `deliver_pending_trap`'s remove-then-reinsert of the
fired trap (`run.rs:2709-2714`) -- the trap is removed before the handler
runs (so the handler's own activation cannot re-trap itself) and reinserted
unconditionally afterward, which is what makes a `CALL ON` trap **not**
disarm on firing, unlike `SIGNAL ON`'s measured one-shot disarm
(`condition_traps.rex`'s own block 2). **Mutation:** skipped the
reinsertion. Rebuilt, reran: output becomes `S[H4]1:VK2:V` -- the `[H10]`
segment (the second firing) is simply gone, because the second raise finds
no trap armed and falls through `exec_raise`'s own "nothing traps it" arm,
returning `'V'` to its caller unhandled. Every other segment is unchanged,
pinning the missing segment specifically to re-arming rather than to the
raise or the `SIGNAL` jump. Reverted; confirmed clean.

### A side measurement that shaped these programs, not itself a corpus rule

A bare `RAISE cond` (no `RETURN`/`EXIT`/`ARRAY`) ends the **whole program**
silently at rc 0, measured identically whether raised at top level,
untrapped, or inside a routine called from a trapped caller (three separate
probes, all rc 0, no further output, no handler run). This matches
`exec_raise`'s own `if raise.result.is_none() || !returns { ... return
Ok(Flow::Exit(result)); }` (`run.rs:3005-3014`) -- our crate already agrees
with the oracle here, so this is a confirmation, not a divergence -- but it
made a bare `RAISE` the wrong building block for "resumes and then raises
again," which is why every raise in the corpus programs above spells out
`RETURN` explicitly.

---

## 2. The I19 sweep (Step 3)

**Nothing else found.** `pop_frame` has exactly one call site in the whole
crate (`run.rs:3749`, inside `step_in_temps_frame`), and `ClauseValue::rooted()`
(`clause.rs:275-286`) is an exhaustive match naming the only two `Flow`
variants that ever carry an `ObjRef` across it: `Flow::Return` and
`Flow::Exit` (confirmed against the `Flow` enum itself -- `Leave`/`Iterate`
carry a `SymbolId`/`LeaveOrigin`, `Goto`/`Signal` carry an index, nothing
else carries an `ObjRef` at all). Grepping every consumer of both:

* `exec_call` (the `CALL` instruction, `run.rs:3536-3573`) already re-roots
  explicitly at line 3563 (`self.roots.push_temp(value)`), with its own
  comment citing this exact window ("Same window `Flow::Exit`'s own arm
  documents, closed here rather than left open").
* `eval_call` (the `f(...)` expression form, `eval.rs:519-548`) returns its
  value through `eval()`'s own uniform recursive-descent path, whose only
  post-processing before returning to the caller is `trace_intermediate`
  (which renders already-computed bytes, not a fresh allocation). This path
  is exercised, not merely reasoned about: `lang/call_expression.rex`'s
  `say f(1) + 1` puts exactly this returned value through a further
  allocation (the `+ 1`) after the callee's own activation and clause
  frames have fully unwound, and it has been in the stress-tested union
  since Task 4 -- `cargo test -p rexx-exec --test collect_stress` passes on
  it today, with a per-program zero-collection check that would catch a
  silent no-op.
* The top-level `Return`/`Exit` path to `exit_code_for` is I19 itself, the
  one the pointer already names.

No second site. I did not touch `lib.rs`.

---

## 3. Running `collect_stress` over the grown union (Step 4)

`cargo test -p rexx-exec --test collect_stress`: **2 passed, 0 failed**, both
before and after this task's changes. `the_l0_subset_passes_again_under_collect_on_every_allocation`
iterates the union (now 44 programs, was 41) and asserts, per program: plain
and collect-every-allocation runs produce identical stdout/stderr/exit code,
and every single program performs at least one real collection (not merely
an aggregate total -- checked per program, so one silent program cannot hide
behind the rest).

**What running it produces:** it passes, with no panic anywhere in the grown
union, including the three new combination programs. Checked directly that
none of the three is a silent zero-collection pass (the test would have
named it if so; it did not).

**What running it over the *old* 41-program union already showed, measured
at the very start of this task before any edit:** also 2 passed, 0 failed --
so "the stress mode has never seen a call frame" was no longer literally
true of the tree at `598be610` (Tasks 3, 5, 6, 7 already landed call/procedure/
signal/condition corpus programs, each exercised by `collect_stress` as part
of every subsequent `cargo test --workspace`). What is genuinely new here is
depth, not first contact: a raise inside a called routine inside a
three-pass loop, a stem re-aliased across three separate calls with the
running total read back and written on each one, and a `CALL ON` trap
removed and reinserted twice from the same table entry within one program --
combinations no single existing program drove the collector through before.
Nothing panicked. What *would* have failed had any of the three underlying
mechanisms been missing is exactly what Section 1's three mutations show,
independently of the collector: each is a plain output divergence, not a
rooting panic, because none of the three mechanisms this task pins is itself
a rooting defect -- the collector's own question ("did a value survive
being unrooted across an activation boundary") is answered by the *existing*
`a_clause_value_survives_the_handler_its_boundary_runs` test and by Section 2
above, not by these three programs, which pin *value correctness*, not
*rooting*.

---

## 4. I26/D7's documentation debt (Step 5)

`rust/corpus/phase-4a.txt` and `rust/corpus/README.md` both said the three
`num/` `List`-using programs "stay ... for 4b or 4c, once `List` exists,"
which contradicted `phase-4-exclusions.txt`'s own Task 16 ruling (`List` is
Phase 5's -- it builds an `Array`, which is Phase 5's object model, not a 4b
or 4c one). Both corrected to say Phase 5, citing the exclusions file's
"EXPRKIND OWNERSHIP" section. This is the "documentation half" the plan
names; the code-level owner tables (`lib.rs`'s `expr_owner`, `owners.rs`'s
`EXPR_TAGS`) already said `"Phase 5"` correctly before this task -- I checked
before assuming they needed the same fix, and they did not.

---

## Concerns

None that block. Two things worth a reader's attention:

* The `expose_stem_across_calls.rex` header's mutation paragraph was wrong on
  its first draft (predicted a third-call failure by reasoning about where
  the raise sits, rather than running the mutation) and was corrected before
  landing, once run. Left in as the honest record rather than smoothed over,
  per `rust/CLAUDE.md`'s own instruction to run rather than reason -- it is
  the kind of mistake this task's own brief was written to guard against, and
  it happened anyway, one level down from the corpus program itself.
* `docs/superpowers/plans/phase-4-exclusions.txt`'s KNOWN GAPS section, read
  in full for this task: the "controlled-`DO` per-pass value lines" gap and
  its indent-decrement twin are **both** already closed/covered (Task 9's fix,
  and Deviation 0's normalisation, respectively) -- I did not find a
  still-open "controlled-DO per-pass `>>>` gap" distinct from those two. The
  one genuinely open row I did find and avoided is `TRACE ?` (all letters,
  including `?L`): the interactive-prefix banner lines are unimplemented.
  None of my three programs uses `TRACE ?` anything.

---

## Fix round 1

**Status: DONE.** Commit `8b4fd867` (`git log`: `8b4fd867e643d6f13541688c68531555afa0be94`,
"4b Task 10 fix round 1: drop two non-discriminating combinations, fix SIGL"),
on top of `3d6d68d7`.

Both disputes the coordinator's message raised in my favour were exactly
right (the coordinator's own words: "one of them was my error"), and are not
revisited here. What follows is the review's two Criticals and the Important
item, addressed in full.

### Critical 1 -- two of three combination programs were not new coverage

**Accepted, and confirmed independently rather than taken on faith.** I first
reproduced the review's own two findings, then spent real effort looking for
a mutation that *would* discriminate before concluding neither program could
be salvaged as designed.

**What I tried, each run from a byte-identical copy of `run.rs`
(`md5sum`-verified before and after every mutation, restored by `cp` from
that copy, never `git checkout --`), rebuilt, and run against a fixed set of
existing single-construct programs (`call_procedure_expose.rex`,
`condition_traps.rex`, `use_arg_forms.rex`, `signal_forms.rex`) alongside the
two disputed programs, from a fresh directory, oracle wrapper as mandated:**

1. **A frame leak** -- skipping `self.roots.pop_slots(callee.frame)` in
   `resolve_and_run_call`'s own `owns_frame` arm (`run.rs`, then around line
   3428). Result: **already caught**, and more severely than by either of my
   programs -- `call_procedure_expose.rex` panics (`thread panicked`, process
   exit 101) because it calls **four different** `PROCEDURE`d routines in
   sequence (`sub`, `bee`+`cee`, `plural`, `stemsub`), so a leaked frame from
   the first call already corrupts the second's own `frame_len` computation.
   `use_arg_forms.rex` panics the same way. This ruled the mutation out on
   its own: an existing program already exercises "more than one `PROCEDURE`
   call in sequence," so a program built specifically to call the *same*
   routine three times adds nothing here.
2. **A stale trace-indent restore** -- skipping `self.indent_offset =
   saved_offset;` in the same function, right beside `self.activation_indent
   = saved_base;`. Result: **inert**. All seven programs tested (the four
   existing ones plus my three) produced byte-identical output with and
   without the restore. Not a useful mutation at this call shape.
3. **A dropped `SIGL`** -- skipping `self.set_sigl(self.clause_state.line());`
   in `deliver_pending_trap`. Result: **inert**, for a reason worth recording
   rather than leaving as a dead end: `resolve_and_run_call`'s own argument
   evaluation sets `SIGL` again (`run.rs:3282`, `self.set_sigl(self.clause_state.line())`,
   run for every call including the handler's own `resolve_and_run_call`
   inside `deliver_pending_trap`), so the one instance removed here is
   redundant with a second site, not load-bearing on its own.

**Why nothing landed, read from the code rather than guessed after three
misses:** `DO`/`LOOP`'s own control-variable bookkeeping (`run_repeating`,
`run.rs`) lives in a Rust local on that function's own stack frame -- never
copied into any field on `Interp`/`Activation` -- so nothing a nested `CALL`
does to interpreter state can reach it; Rust's own call-stack discipline
protects it more strongly than any single-line mutation could threaten.
`PROCEDURE EXPOSE` (`exec_procedure`, `run.rs:1939-2020`) re-derives
*everything* fresh on every call: `outer = self.activation().frame` is read
from the current activation, not cached; `alias_slot` installs a plain
position redirect into a brand-new `push_slots` frame, never a value copy;
and the name-to-slot map a name like `ST.` resolves through
(`Plan::slot_of`, `plan.rs:528`) is **one shared map for the whole source
file**, not a per-routine one -- confirmed by reading `resolve_and_run_call`
passing `Rc::clone(&caller.plan)` into every callee's own `Activation`,
`activation.rs:349-370`'s own doc, and `Interp::slot_of`'s three-source
resolution order (`plan.rs:552-573`). A name present anywhere in the file's
text gets one canonical index established at parse time, unaffected by how
many times any routine referencing it is called. The one thing that *does*
grow between calls in `expose_stem_across_calls.rex` -- `RESULT`, added to
the caller's frame by `exec_call` only *after* the callee returns, since the
program never mentions `RESULT` literally -- never mattered, because `ST.`'s
own index was already below that growth's insertion point on the very first
call.

**Removed both `raise_in_routine_in_loop.rex` and
`expose_stem_across_calls.rex`.** `coverage.rs`'s
`every_in_scope_variant_is_witnessed_by_the_phase_subsets` still passes with
them gone -- nothing they constructed was otherwise unwitnessed.
`call_on_trap_rearms.rex` remains as the one program this task adds; its own
mutation (skipping `deliver_pending_trap`'s trap re-insertion) was re-run
against the same four existing programs plus itself and diverges **only**
on `call_on_trap_rearms.rex` -- none of the others arms a `CALL ON` trap and
then raises the identical condition a second time, which is exactly the
shape this program alone has.

I considered redesigning rather than removing (an `INTERPRET`-introduced
name between calls would force `frame_len`'s growth path to matter, since
such a name has no parse-time index at all), but that changes the
program's own construct pairing to `INTERPRET` + `PROCEDURE EXPOSE` rather
than the "exposed stem mutated by a callee" the plan named, and a fresh,
rushed combination invented under review pressure is exactly how the first
round's two weak programs happened. Better to report the honest gap than
ship a third guess.

### Critical 2 -- the I19 sweep's enumeration was false, redone properly

**Confirmed: seven `pop_frame` call sites, not one.** `grep -n "pop_frame("`
across `rexx-exec/src` and `rexx-core/src`: `eval.rs:591,651,701,764,804,891`
(six) and `run.rs:3749` (one, already covered in the original round). The
"exactly one call site" sentence was false and is retracted.

**Read all six `eval.rs` bodies directly** (`eval_prefix`, `eval_arithmetic`,
`concat`, `eval_compare`, `eval_logical`, `eval_logical_list`) rather than
inheriting the reviewer's own "documented as allocation-free" characterisation
as settled. All six share one shape, confirmed line by line:

* The function computes its own returned value (`self.number(...)` or
  `self.text(...)`, both allocating) and binds it to a local.
* `self.roots.pop_frame(frame)` runs on the very next line.
* `Ok(value)` (or `Ok(joined)`/`Ok(result)`) is the line after that -- no
  other statement anywhere between the allocation and the return.

So the value's own allocation happens **before** `pop_frame`, and nothing
happens **between** `pop_frame` and the value leaving the function. `pop_frame`
itself is `self.temps.truncate(frame.0)` (`rexx-core/src/roots.rs:141-143`) --
a bare `Vec` truncation, confirmed by reading it again rather than assumed
from the first round's summary of it -- so it cannot itself allocate. `concat`
(`eval.rs:681-703`) says as much in its own comment ("`joined` is unrooted
from here to the caller's own `push_temp`, and nothing between the two
allocates"); the other five say nothing but the code reads identically.

Once each of the six returns, its value flows back through `eval_node` into
`eval()` (`eval.rs:122-166`), whose only other action before returning to
*its* caller is `trace_intermediate`, which renders already-computed byte
vectors through `to_text` and mints no new `ObjRef` -- unaffected by this
round's correction, since that was checked in the first round already
(Section 2 above) and re-confirmed here rather than re-argued.

**Conclusion, now checked rather than assumed: all seven sites are
accounted for, and the run.rs one (I19 itself) remains the only site with
an actual *gap* between the pop and the value's next rooting** -- the six
in `eval.rs` have zero intervening code, where the run.rs site's own value
propagates through many further function returns before `exit_code_for`
ever sees it. No `lib.rs` change made, unchanged from the first round.

### Important -- SIGL, corrected

Both flagged headers were wrong in the same direction. Re-measured from a
live oracle run of each file **exactly as committed** (line numbers shift
whenever header prose changes, and both files' headers changed in this
round, so the citation is taken from the current file, not carried over):

* `call_on_trap_rearms.rex` (the one program that survives): SIGL reads
  **44 -> now 54** for the first handler firing and **50 -> now 60** for the
  second, both the *calling* clause's own line (`call raiser`), never
  `raiser`'s own `raise` line (56, now 66). Corrected in the file's own
  header and re-verified against the oracle after the correction (see the
  transcript below) -- not just asserted a second time.
* `raise_in_routine_in_loop.rex` carried the identical backwards claim. Moot
  now that the file is removed, but recorded here since the report's own
  quoted SIGL value (`[H@5]`, from a pre-header probe transcript) was also
  wrong for the same reason Critical 1's own header-shift note describes:
  the committed file's real transcript read `[H@46]`, not `[H@5]`.

Re-verified transcript for the corrected `call_on_trap_rearms.rex`, oracle
and `rexx-run` both, byte for byte:

```
S[H54]1:VK[H60]2:V

    52 *-* call on user zx name h
    53 *-* zlog = 'S'
    54 *-* call raiser
    65 *-*   raiser:
    66 *-*   raise user zx return 'V'
    69 *-*   h:
    70 *-*   zlog = zlog || '[H' || sigl || ']'
    ...
    60 *-* call raiser
    65 *-*   raiser:
    66 *-*   raise user zx return 'V'
    69 *-*   h:
    70 *-*   zlog = zlog || '[H' || sigl || ']'
```

Mutation transcript, re-run against the corrected file: skipping
`deliver_pending_trap`'s trap re-insertion gives `S[H54]1:VK2:V` (the
`[H60]` segment gone), matching the file's own header exactly.

### Gates, re-run in full

| gate | result |
|---|---|
| `cargo test --workspace` | **990 passed / 0 failed** |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | **42 of 42**, STRICT (was 44 of 44) |
| `cargo test -p rexx-exec --test assertions` | **4224 / 4259** (unchanged) |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |

Corpus count: 42 of 42 (41 of 41 before this task began, 44 of 44 after the
first round's three additions, 42 of 42 after this round's two removals).
`coverage.rs`'s `every_in_scope_variant_is_witnessed_by_the_phase_subsets`
still passes -- removing the two programs dropped no variant witness.

### Concerns

None that block. The honest state of Critical 1 is that this task now ships
**one** new combination program rather than three, because two of the three
did not hold up under scrutiny and I could not construct an honest
replacement for either in the time available -- see the "considered
redesigning rather than removing" paragraph above for why I did not simply
substitute a different pairing. If a future round wants a second genuinely
new combination, `INTERPRET` crossed with something else is the one avenue
this round's own investigation did not rule out (it was set aside for
changing the construct pairing under review pressure, not for being
unpromising).
