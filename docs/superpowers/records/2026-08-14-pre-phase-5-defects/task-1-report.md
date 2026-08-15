# Task 1 report: `PROCEDURE` first in a `::ROUTINE` panics where the oracle raises 17.1

Status: **DONE_WITH_CONCERNS** -- the defect is fixed, and the work turned up a
second live divergence at the same field (fixed here, measured) and one
unrelated trace-indent divergence (not fixed, described at the end).

Commit: `3cf9fcaba8aa5db565c16a5187ef6c671fcb08ef` on `plan/rust-rewrite`, read
back with `git log -1` rather than written from memory. (This report is not in
the commit: `.gitignore:19` excludes `.superpowers/`.)

---

## Step 1: the oracle's actual rule, case by case

Every row is one program, run in a fresh empty directory
(`scratchpad/task1-oracle/`), under the standard wrapper
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`,
with stdout, stderr and exit status captured as three separate descriptors.
"first" means the `PROCEDURE` is the entered body's first executed instruction;
"after NOP" means one `nop` runs in that body first.

| # | entered by | `PROCEDURE` | oracle rc | oracle stderr |
|---|---|---|---|---|
| A1 | internal label, by `CALL` | first | **0** | empty -- it runs |
| A2 | internal label, by `CALL` | after NOP | 239 | `     7 *-*   procedure` / `     1 *-* call sub` / 17 / 17.1 |
| B1 | internal label, as a function | first | **0** | empty -- it runs |
| B2 | internal label, as a function | after NOP | 239 | `     6 *-*   procedure` / `     1 *-* qq = sub()` / 17 / 17.1 |
| C1 | `::ROUTINE`, by `CALL` | first | **239** | `     6 *-* procedure` / `     1 *-* call sub` / 17 / 17.1 |
| C2 | `::ROUTINE`, by `CALL` | after NOP | 239 | same, line 7 |
| D1 | `::ROUTINE`, as a function | first | **239** | `     5 *-* procedure` / `     1 *-* qq = sub()` / 17 / 17.1 |
| D2 | `::ROUTINE`, as a function | after NOP | 239 | same, line 6 |
| E1 | the main program | first | 239 | `     1 *-* procedure` / 17 / 17.1 (no second echo -- there is no caller) |
| E2 | the main program | after NOP | 239 | same, line 2 |
| G1 | `::METHOD` | first | **239** | `     7 *-* procedure` / `     2 *-* oo~meth` / 17 / 17.1 |
| G2 | `::METHOD` | after NOP | 239 | same, line 8 |

The full 17 banner in every failing row is

```
Error 17 running <path> line N:  Unexpected PROCEDURE.
Error 17.1:  PROCEDURE is valid only when it is the first instruction executed after an internal CALL or function invocation.
```

**The rule the oracle actually applies:** `PROCEDURE` is legal only as the
first instruction executed in an activation entered by an *internal* call --
a label in the running program's own source, reached by `CALL` or as a
function. A `::ROUTINE` is **not** an internal call, however it was reached,
and neither is a `::METHOD`. 17.1's own sentence turns out to be exactly
right, and the word doing the work in it is "internal".

Two details worth keeping, both of which are part of the expected bytes:

* **The echo order is failing clause first, calling clause second.** Rows A2,
  B2, C1-D2 and G1-G2 all show it. E1/E2 have only one line, because the top
  level has no caller.
* **The failing clause's indent differs by entry.** An internal label's
  clauses echo two columns in (A2, B2: `*-*   procedure`); a `::ROUTINE`'s and
  a `::METHOD`'s echo at indent 0 (C, D, G). `support::normalize_stderr`
  (DEVIATION 0) collapses exactly that run, so no corpus harness can see it --
  which is why the fix also carries an exact-bytes unit test.

**`::METHOD` is out of scope for the fix (Phase 5) and the row is recorded as
a decision, not a gap.** The oracle refuses a `PROCEDURE` there too, so when
`Entry::Method` lands it takes the same `false` arm as `Routine`; the
exhaustive `match` at the admission site will not compile until someone says
so explicitly.

### The same table for `USE LOCAL`, because it reads the same field

Captured because Step 2 showed a second reader. Same wrapper, same clean
directory (`scratchpad/task1-oracle/ul/`).

| entered by | `USE LOCAL` first | oracle rc |
|---|---|---|
| the main program | 98.993 "may only be used from method invocations" | 158 |
| `::ROUTINE`, by `CALL` | **98.993**, plus the two-line echo | 158 |
| `::ROUTINE`, as a function | **98.993**, plus the two-line echo | 158 |
| internal label, by `CALL` | 99.910, reported as `Error 99 ... Translation error` | 157 |
| internal label, as a function | 99.910, same | 157 |
| `::METHOD` | runs | 0 |

The two internal-label rows are a *translate-time* refusal (a `USE LOCAL`
that is not its body's first instruction), and `rexx-parse` already
intercepts them, so they never reach the executor.

---

## Step 2: every construction of `entered_by_call`, mapped to a case

`/bin/grep -rn "entered_by_call" rust/crates/` before the fix returned five
lines: three constructions and two readers.

| site | constructs | Step 1 cases it covers | does the oracle grant it? |
|---|---|---|---|
| `activation.rs:642` `Activation::new` | `entered_by_call: false` | E1, E2 (the main program) | **Yes.** `PROCEDURE` at top level is 17.1, and `false` refuses it. |
| `activation.rs:748` `Activation::nested` | `entered_by_call: true` | A1, A2, B1, B2 (an internal label, both routes) | **Yes.** Both routes run when the `PROCEDURE` is first. `invoke_call`'s `Entered::Label` arm is its only caller, and it is also used for a label called from *inside* a routine body (measured, see the sweep). |
| `activation.rs:800` `Activation::routine` | `entered_by_call: true` | C1, C2, D1, D2 (a `::ROUTINE`, both routes) | **No. This is the defect.** The oracle raises 17.1 in all four, and this crate admitted the instruction. |
| `run.rs:2377` reader, `exec_procedure` | -- | the `PROCEDURE` column | wrong for the `::ROUTINE` rows, by the row above |
| `run.rs:2532` reader, `exec_use` | -- | the `USE LOCAL` column | **also wrong for the `::ROUTINE` rows**, and in the opposite direction: it answered 99.910 rc 157 where the oracle answers 98.993 rc 158 |

Measured on the pre-fix binary rather than inferred, both engines:

```
c1_routine_call.rex  rc 101, stdout empty, panic at roots.rs:409  "grow_slots on a frame that is not the top one"
d1_routine_func.rex  rc 101, stdout empty, panic at roots.rs:254  "pop_slots on a frame that is not the top one"
u_routine_call.rex   rc 157, Error 99.910   (oracle: rc 158, Error 98.993)
u_routine_func.rex   rc 157, Error 99.910   (oracle: rc 158, Error 98.993)
```

**The `CALL` and the function route corrupt the frame stack differently**, and
that is why they panic at two different assertions: the `CALL` shape leaves
the *caller's* next unbound name growing a frame that is no longer the top
one, and the function shape fails earlier, on the way out, popping a frame
that is not the top. A single program only ever reaches the first.

**Where the wrong construction came from.** The field's own doc said "`USE
LOCAL` is error 99.910 in a called routine and error 98.993 at top level".
"Called routine" reads as both an internal label and a `::ROUTINE`; the
oracle answers differently for the two, and the bool could not tell them
apart. One sentence, two meanings, and both readers took the wrong one.

---

## What changed, and why

**`crates/rexx-exec/src/activation.rs`** -- `entered_by_call: bool` becomes
`entry: Entry`, a three-variant enum (`TopLevel`, `InternalCall`, `Routine`)
whose doc carries the Step 1 table including the `::METHOD` row. The three
constructors set the variant that names them: `new` -> `TopLevel`, `nested`
-> `InternalCall`, `routine` -> `Routine`.

A bool cannot express this. The two readers disagree about which entries they
want (`PROCEDURE` wants "internal call"; `USE LOCAL` wants "method
invocation"), and a `::ROUTINE` is on the far side of both -- so any single
bool is wrong for one of them. The enum is also the mechanism that makes
Phase 5 answer the question: both readers `match` exhaustively with no
wildcard, so adding `Entry::Method` will not compile until each site has
decided what it means.

**`crates/rexx-exec/src/run.rs`, `exec_procedure`** -- admission is now
`first_instruction && matches Entry::InternalCall`, written as an exhaustive
match. The wrong decision was admitting the instruction; the assertion at
`rexx-core/src/roots.rs:409` is untouched, as instructed.

**`crates/rexx-exec/src/run.rs`, `exec_procedure`** -- a new `assert!` that
the activation does not already own its frame, placed immediately after the
admission check. The invariant is established here and violated here; the
existing assertions catch it a step or two later, by which time the
instruction that caused it has returned and the message names a symptom
instead of a cause. Verified live: under the re-admission mutation it fires
with `a PROCEDURE admitted in an activation that already owns its frame` at
`run.rs:2401` instead of the `roots.rs` message.

**`crates/rexx-exec/src/run.rs`, `exec_use`** -- `USE LOCAL` now asks "is this
a method invocation", exhaustively over `Entry`, rather than "was this entered
by a call". No entry this crate can construct is a method invocation, so every
shape reaching the executor takes the 98.993 arm -- which is what the oracle
answers for both the top-level and the `::ROUTINE` rows.

That doc comment carried two statements the measurements falsify, and both
are corrected rather than hedged:

* "as a called routine's first instruction ... it is 99.910" -- measured
  98.993 rc 158 for a `::ROUTINE`; the 99.910 rows are internal labels and are
  a *translate-time* refusal.
* "Only the 98.993 shape reaches this function, and the 99.910 arm is
  unreached" -- the 99.910 arm **was** reached, by exactly the `::ROUTINE`
  shape, and it was the wrong answer. After the fix the sentence is true, and
  it now says which rows reach it.

**`corpus/lang/procedure_entry_rule.rex`** (new), listed in
`corpus/phase-4c.txt`, pinned in `tests/coverage.rs`'s `EXPECTED_SUBSET_4C`,
described in `corpus/README.md`'s 4c table, with its oracle line index
generated into `crates/rexx-parse/tests/sourceline_oracle/`.

**Four unit tests** in `run.rs`: the two `::ROUTINE` shapes with exact stderr
bytes, the neighbouring internal-label-as-a-function success, and `USE LOCAL`
first in a `::ROUTINE`.

### Step 3: which test convention, and why

The repository's existing convention for an oracle differential of a
*failing* program is **a corpus program listed in a phase subset file**. The
precedents are `lang/trace_numeric_request.rex` (Error 24.901 as its last
clause) and `lang/builtin_argument_range.rex` (a 40.903 range message, "Fatal,
so it is the last clause"). That route buys three things at once:
`tests/corpus.rs` runs it against the live oracle on stdout, stderr and exit
status separately; `tests/ir_dual.rs` runs it on **both engines** as part of
the corpus population; and `tests/collect_stress.rs` runs it under
collect-on-every-allocation. No new mechanism was invented.

One deliberate design choice inside the program: **17.1 is trappable**
(measured -- `signal on syntax` catches it, `rc` is 17, and the program runs
on). So blocks C and D trap the refusal and then assign a fresh variable in
the *caller*, which is exactly the `grow_slots` path the defect corrupted;
block E leaves the same refusal untrapped, because the exit status and the
two-line echo are only observable when it is. A purely fatal program can show
the refusal but not the recovery.

Two things the corpus program cannot do, which is why the unit tests exist
beside it:

* `support::normalize_stderr` collapses the space run after a trace marker,
  so the corpus comparison cannot see the `*-*` indent. The exact-bytes unit
  tests can, and they carry the oracle's `cat -A`'d transcript.
* A program dies once. The baseline panicked in block C, so block D was never
  reached in the pre-fix run -- the function-call shape needs its own test to
  be witnessed at all.

### Step 3/5 verification, before and after

The new corpus program, run under both engines from a clean directory:

```
baseline (HEAD, 625d653fb)  tree-walker  rc 101, stdout EMPTY, panic roots.rs:409
baseline (HEAD, 625d653fb)  ir           rc 101, stdout EMPTY, panic roots.rs:409
fixed                       tree-walker  rc 239, stdout and stderr byte-identical to the oracle
fixed                       ir           rc 239, stdout and stderr byte-identical to the oracle
```

and every Step 1 case file, both engines, compared against its oracle capture
on all three descriptors after the fix: **20 of 20 MATCH** (10 programs x 2
engines), including the A2/B2 two-column indent and the E1 single-line echo.
The `USE LOCAL` rows match too, on all three descriptors, for the top-level
and both `::ROUTINE` shapes.

---

## Step 6: the sweep, including what came back clean

**(a) Other instructions whose legality depends on how the activation was
entered.** After the fix, `entry` has exactly two readers, and they are the
only two instructions in scope with entry-dependent legality:
`exec_procedure` and `exec_use`'s `Use::Local` arm. The rest of that family in
the oracle -- standalone `EXPOSE`, `GUARD`, `REPLY`, `FORWARD` -- is
`Owner::Phase("Phase 5")` in `tests/owners.rs`, so each fails loudly with a
code outside the `256 - major` band and cannot give a wrong answer here.
`RETURN`/`EXIT` do behave differently in a routine, but through `Entered` and
`Failure::Exited` rather than through this field, and `lang/routine_dispatch.rex`
block D is already their witness. **Clean.**

**(b) Other places `owns_frame` is set true twice.** The writers are
`Activation::new` (`TopLevel`, true at construction), `Activation::nested`
(`InternalCall`, false at construction), `Activation::routine` (`Routine`,
true at construction) and `exec_procedure`'s swap. The one pairing that
double-sets is `routine` + `exec_procedure`, which is this defect and is now
unreachable; `TopLevel` + `exec_procedure` is refused by 17.1's top-level row;
`InternalCall` + `exec_procedure` is the single legal push. `run.rs:6498`'s
`owns_frame` is a same-named local in the `LEAVE`/`ITERATE` search-frame code
and is unrelated. The new `assert!` pins the property rather than leaving it
as prose. **Clean.**

**(c) Probes run rather than reasoned about** (`scratchpad/task1-oracle/sweep/`),
all three descriptors, both engines against the oracle:

| probe | result |
|---|---|
| a label called from **inside** a `::ROUTINE` body, `PROCEDURE` first | runs, rc 0, MATCH -- `nested` is used there too, and the fix does not over-refuse |
| the same, reached as a **function** from inside a routine | runs, rc 0, MATCH |
| two `PROCEDURE`s in a row in a called label | 17.1 rc 239, MATCH -- the permission is spent once |
| `PROCEDURE` after a `SIGNAL` inside a called label | 17.1 rc 239, **stderr differs by two columns** -- see below |

**(d) The corpus sweep.** Baseline and fixed `rexx-run` binaries, built from
the same tree with only the two source files differing, run over every corpus
program and diffed on all three descriptors:

```
phase-4a + 4b + 4c subset union:  52 programs run,  1 changed output
every .rex under rust/corpus/:    71 programs run,  1 changed output
```

The one change is `lang/procedure_entry_rule.rex` itself (rc 101 -> 239), which
did not exist before. No existing corpus program changed.

---

## Step 7: the mutation witness

Backups taken with `cp` to `scratchpad/task1-backup/`, with `sha256sum`
recorded before each mutation; restored with `cp`, `touch`ed, verified with
`sha256sum -c`, rebuilt, and re-run. Never `git checkout --`.

**The control first, because "can fail" is not "adds coverage".** The full
workspace suite was run at `HEAD` with the defect present and every new file
held out of the tree:

```
HEAD, defect present:   1497 -> 1493 passed, 0 failed, 4 ignored;  corpus 51 of 51 matching
```

So the pre-existing suite does not catch this defect at all -- there was no
existing test to add to. The four new tests and the corpus program are the
whole of the coverage.

**Mutation M1 -- re-admit the instruction** (`Entry::InternalCall | Entry::Routine => true`
in `exec_procedure`). Full workspace run, `--no-fail-fast`:

```
1492 passed, 5 failed
  run::tests::a_procedure_first_in_a_called_routine_is_17_1_with_the_calling_clause_second
  run::tests::a_procedure_first_in_a_routine_reached_as_a_function_is_17_1_too
  corpus::corpus_differential
  ir_dual::both_engines_agree_across_every_population
  collect_stress::the_l0_subset_passes_again_under_collect_on_every_allocation
```

`corpus_differential` fails even though that harness runs in REPORT mode and
always exits 0 on a mismatch: it runs the executor **in process**, so the
panic aborts the harness. The panic it aborts with is the new assertion,
naming the cause, rather than `roots.rs:409` naming the symptom.

**Mutation M2 -- restore the old `USE LOCAL` answer** (`Entry::InternalCall | Entry::Routine => true`
for `method_invocation`, which is precisely the pre-fix `!entered_by_call`):

```
run::tests::use_local_first_in_a_routine_raises_98_993_not_99_910  FAILED
    assertion `left == right` failed: 256 - 98, reached by CALL
      left: 157
     right: 158
2 tests ran (1 passed, 1 failed) -- a non-zero run count, so this is not the
"cargo test matched nothing" shape
```

Restore verified both times:

```
rust/crates/rexx-exec/src/activation.rs: OK
rust/crates/rexx-exec/src/run.rs: OK
rust/crates/rexx-exec/tests/coverage.rs: OK
rust/corpus/phase-4c.txt: OK
rust/corpus/README.md: OK
rust/corpus/lang/procedure_entry_rule.rex: OK
rust/crates/rexx-parse/tests/sourceline_oracle/procedure_entry_rule.txt: OK
```

**What the corpus program catches that the unit tests do not**, stated
honestly rather than assumed: under M1 both catch it independently, so neither
is load-bearing *for that mutation*. What the corpus program uniquely covers is
the two-engine comparison (`ir_dual` runs the corpus population; the unit tests
run only the default engine) and the trapped-then-continue path in blocks C
and D, which no unit test exercises. It was not deleted for that reason.

---

## Step 8: gates

Run from `rust/`, each unpiped, each status read on its own:

```
cargo fmt --all --check                                     0
cargo clippy --workspace --all-targets -- -D warnings       0
memcap 8G cargo test --release --workspace --no-fail-fast   0
    1497 passed, 0 failed, 4 ignored
    corpus differential: 52 of 52 matching (REPORT mode)
    ootest rows: 4224 of 4259 matching (unchanged from HEAD)
```

`clippy` recompiled `rexx-exec`, so it did examine the changed code; per
`rust/CLAUDE.md` a same-session green is still provisional and a phase
boundary wants a clean target directory.

---

## Found and deliberately not fixed

**A `SIGNAL` inside a `CALL`ed label does not reset the trace indent here,
and does on the oracle.** Isolated to a program with no `PROCEDURE` in it at
all, so it is independent of this task:

```rexx
call sub
say 'main'
exit 0
sub:
signal onward
onward:
zz = 1 / 0
return
```

```
oracle:  "     7 *-* zz = 1 / 0"     rc 214
here:    "     7 *-*   zz = 1 / 0"   rc 214
```

Same error, same exit status, two columns of stderr apart. It is invisible to
every oracle harness in the tree because DEVIATION 0's normalisation collapses
exactly that run. It is the same family as defect 3 of this plan (trap handler
clause indent) and is **not** the same site, so it wants its own entry rather
than being folded into this commit. I have not edited the shared plan file for
it, because other tasks in this round are live in the same tree; it needs
writing into the plan by whoever owns it.

**The parse-time clause echo for `USE LOCAL` in an internal label** still
differs from the oracle -- rc 120 with `rexx-exec: 99.910: ...` against the
oracle's rc 157 with a clause echo. The error number agrees. This is the
standing parse-error limitation `execute` documents in `lib.rs`, it is
unchanged by this task (measured on both the baseline and the fixed binary),
and it is out of scope.

**`::METHOD`** is Phase 5. Its row is measured and recorded in `Entry`'s own
doc so that the exclusion is a decision; the exhaustive matches will refuse to
compile when `Entry::Method` is added, which is the point.
