# Phase 5d Task 2 — the method-body table (D76), and D73's guard

**BASE `9738e9858`.** Tree held alone. Plan `docs/superpowers/plans/2026-09-03-phase-5d.md`
(Task 2), spec `docs/superpowers/specs/2026-09-03-phase-5d-foundation.md` (D73, D76, D77).

**Tree hash before the first gate run: `163dd8f92ef2c8c6abd0e0c5fee0cddb68b9a0c7`**
(`git write-tree` over the staged change; this report is added on top at commit time and nothing
reads it).
**Tree hash after the last gate run: `TREE_AFTER`.**

**Three earlier gate runs were started and voided**, each killed between gates rather than left
measuring a tree that no longer existed. Recorded because the third one is the interesting one.

* At `0ae1f25f7dbc691b5ea971ef9fb59d89900ef652`, after `G1 = 0` and `G2 = 0`: the committed table's
  header stated the gate rule incompletely.
* At `2cccb2f93181fc1c23fe104d34e5899a082de05b`, after `G1 = 0` and `G2 = 0`: what `answers` is
  actually worth belonged in the spec's D76 record and not only in this report.
* At `676b68662c57792a5bea334eeab554edff985990`, after `G1 = 0` and `G2 = 0`: the controller read
  the committed table and challenged the `diverge` rows. One of the seven was mine to fix and two
  more defects came out of chasing it — see "The controller's challenge" below.

**Killing the first left one orphaned `rexx` and one orphaned test binary**, found with
`pgrep -a -x rexx` and `pgrep -a -f target/release/deps/` and killed: the sweep's bounds cover a run
whose parent is alive and nothing else. The later two were killed with their process groups and left
neither.

---

## What landed

### (a) `rust/corpus/method-bodies.txt` — the table

**1347 rows, one per (class, method, arm) of `corpus/docs/class-methods.txt`**, each classified by
sending the documented name **with no arguments** to a real receiver: the class object for the class
arm, `class-set.txt`'s construction expression for the instance arm.

```
class <TAB> method <TAB> arm <TAB> verdict <TAB> evidence
```

Five columns, not four. The fifth is what makes the table readable rather than merely countable, and
it is measured on the run like every other column:

| verdict | evidence |
|---|---|
| `loud` | the construct the crate's own message named |
| `answers` | the exit status both sides gave |
| `diverge` | which channels moved (`diverge-stdout`, `diverge-both`, …) |
| `unstable` | which side's own answer moved: `this crate` or `the oracle` |

It sits beside `corpus/builtin-status.txt` rather than in `corpus/docs/`, because
`extract_docs.rs`'s `the_row_set_directory_holds_exactly_the_row_sets` asserts that directory holds
exactly what `rexx-extract-docs` writes, and this table is derived by **running** both interpreters
— which is what `builtin-status.txt` is too.

### (b) `rust/crates/rexx-exec/tests/method_bodies.rs` — the derivation and the gate

It re-runs the whole sweep on every run and holds the result against the committed table. **It is
not a generated artifact that a test reads; it is a gated test whose baseline is committed.**

* **pass one, crate-only, over every row** — both engines through `watchdog::run_bounded`, and
  `is_loud` on the result;
* **pass two, the oracle** — only for the rows pass one did not classify, and covered in full under
  "No verdict rests on an answer that moves by itself" below.

### (c) `rust/crates/rexx-exec/tests/unreachable_classes.rs` — D73's guard

Owed since 5c's spec made it a criterion. For every class `class-set.txt` marks `unreachable`,
derived **from the status column**, it runs the documented construction route on both engines and on
the oracle and fails if any of the three answers an instance. Both sides' three descriptors are in
the report:

```
  Buffer: .Buffer~new
      tree-walker  rc=120  out="" err="rexx-exec: method \"NEW\" of class \"Buffer\" is not implemented (Phase 5)\n"
      ir           rc=120  out="" err="rexx-exec: method \"NEW\" of class \"Buffer\" is not implemented (Phase 5)\n"
      oracle       rc=163  out="" err="       *-* Compiled method \"NEW\" with scope \"Buffer\".\n … Error 93.967:  NEW method is not supported for the Buffer class."
```

`Pointer` is the same shape. The empty set is a failure of its own, so the guard cannot pass by
having no subject.

### (d) `Oracle::run_in_zone`, in the shared `tests/support/oracle.rs`

One method, additive: [`Oracle::run`] with `TZ` set on the child alone. This process's own
environment is untouched, which matters because the in-process executor a differential harness
compares against reads the same one.

### (e) The plan and the spec carry the corrections

The spec's §2 sampled table gains the measured column and the reason the two differ; its D76 gains
"What Task 2 built, and the four points where it went past this section", the environment rule, and
the seven divergences. The plan's premise paragraph, its Task 2 shape table and its open question 1
are answered in place.

---

## The gate

**One rule, red in every mode.** `regressed(was, now)` is sixteen literal arms over the square, so
exhaustiveness and non-overlap are the compiler's:

* **a row that was not diverging may not start** — the brief's `loud` → `diverge`, and the same for
  `answers` → `diverge` and `unstable` → `diverge`;
* **a row that was answering may not stop** — my default, which the brief invited an argument
  against and I did not find one. It cost nothing to widen and control 3 is its witness.

That pair of sentences is what `regressed`'s doc, the module doc and the committed table's header
all say, in the same words. `loud` → `answers` is progress and is never gated. **A count of
implemented bodies is not a criterion and appears nowhere as one** — the only assertions are the
regression rule, the structural checks, and the drift check.

**It is not behind `REXX_CORPUS_GATE` or `REXX_PHASE_GATE`.** A refusal that turns into a wrong
answer is not a fact about any phase, so plain `cargo test --release --workspace` catches it.

### The refresh path, and why it cannot launder a regression

```
REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies
```

**84 s in `--release`** — measured from `cargo test`'s own reported duration for the test binary on a
warm target directory. That is the whole sweep: 2695 in-process crate runs (two engines over 1347
rows, plus one re-run for the row whose own answer moved) and 2027 oracle subprocesses. It was 68 s
before the environment probe was added; the probe costs 16 s.

Three properties, each of which had to be true for the table to be worth committing:

1. **It cannot go stale.** Any difference between the committed table and the run — a verdict, a
   piece of evidence, or the file's own header — is a hard failure naming the refresh command.
2. **It cannot be laundered.** The regression assertion runs **before** the refresh writes anything,
   so a tree with a `loud` → `diverge` row cannot be regenerated over. Measured in control 1b.
3. **It is reproducible.** Every non-refresh run reports
   `regressions this run: 0. other drift from the committed table: 0.`

**Two residuals, both named rather than discovered later.** A human can delete a row from the
committed file and refresh, which re-adds it with no baseline to compare against — two edits visible
in one diff, and the same trust level every committed baseline here rests on. And a *legitimate
reclassification* is blocked by the same guard: when the environment probe moved `DateTime today`
from `answers` to `diverge`, the refresh refused, correctly, and the only way forward was to rebuild
the baseline. I did that — the file was an hour-old draft that had never been committed, so nothing
shipped was overwritten — and I did **not** weaken the rule to make it pass. A task that improves
this instrument later will hit the same wall and should rebuild the same way, in its own commit,
with the diff read.

---

## No verdict rests on an answer that moves by itself

**This is the half the controller's challenge produced, and it is the most important thing in the
change.** A comparison is worth nothing where the answer behind it moves on its own, and two things
move it here.

**The oracle can be irreproducible.** `Object~identityHash` is `((uintptr_t)this) ^ UINTPTR_MAX`, so
it answers a different value on every run and there is nothing to match. Measured: five runs, five
values. Before this pass existed the table called it `diverge` — a false defect.

**The comparison can turn on the clock and the zone**, which is worse, because it makes a row's
verdict a fact about *when the sweep ran*. This crate's `.DateTime~today` answers the **UTC** date
where the oracle answers the **local** one, so against this machine's zone alone the two agree for
the twenty-two hours a day those coincide and differ for the other two.

So the crate's one answer is held against the oracle under **three** environments — this machine's
zone, `Etc/GMT+12` and `Etc/GMT-14`. Those two are twenty-six hours apart, so **no instant puts them
on the same calendar date**, which is exactly what makes a clock-reading row score the same at every
hour. Only agreement with all three is `answers`; anything less is a divergence, after the oracle
has been asked to repeat itself in the same environment. The repeat is a pass of its own, so its
window is the rest of the sweep rather than the milliseconds an adjacent run would give.

**Blast radius, measured before the design was chosen**: of the `answers` rows, exactly **one**
changes its oracle answer between those two zones — `DateTime today`, and it is the row the whole
mechanism exists for.

**What it cannot see, and it will matter later.** The environment is shifted on the oracle's side
only, because the crate runs in process and `TZ` is read per process. A body that *correctly*
answered differently per zone would match at most one of the three and read `diverge`. Nothing in
this crate does that today — its `DateTime` bodies ignore the offset, which is the defect itself —
and the task that lands one has to shift both sides rather than trust this. That is stated at the
site as well as here.

---

## What the table says at BASE

| verdict | rows |
|---|---|
| `loud` | 673 |
| `answers` | 665 |
| `diverge` | **7** |
| `unstable` | 2 |

By arm: the class arm is 134 rows (31 `loud`, 101 `answers`, 1 `diverge`, 1 `unstable`); the
instance arm is 1213 (642 `loud`, 564 `answers`, 6 `diverge`, 1 `unstable`).

**`loud` is not the same thing as "this method has no body", and the evidence column is what says so.**
Of the 673:

* **517** refuse on the row's **own** method — an empty method, and the honest reading of "hollow";
* **156** refuse on something else. The receiver's constructor (`method "NEW" of class "StackFrame"`
  ×10, `Pointer` ×5), a `LIBRARY REXX` entry point the body reaches (25 × `stream_uninit`), or a
  `String` method a `CoreClasses.orx` body calls (`SUBSTR` ×10, `RIGHT` ×9, `LEFT` ×4). Those rows
  are still refusals and the safety property still holds for them; what they are not is evidence
  about the named method.

### What `answers` is worth, and it is the number to calibrate the table by

A method needing arguments is sent none, so both sides raise and agree on **the raise**. Of the 665
`answers` rows:

| shared status | rows | what the two sides agreed on |
|---|---|---|
| `rc 0` | **117** | a result |
| `rc 163` | 466 | `93.9xx`, incorrect call to method — almost always a missing argument |
| `rc 168` | 53 | `88.9xx`, an argument the method requires or refuses |
| `rc 165` | 17 | `91.999`, the method returned no result to the `say` |
| `rc 159` | 12 | `97.2`, a PRIVATE method refused from outside the object |

So **117 rows of 1347 are a documented method answering a value this crate and the oracle agree
on.** The other 548 say the method exists and its argument checking agrees, which is a real property
and a much smaller one. The table's header says this in one sentence and the module doc in one
paragraph; the gated rule does not rest on it.

### The seven `diverge` rows are defects this instrument found, and I did not fix them

**All seven are `DateTime`, and they fall into two families.**

| row | what it is |
|---|---|
| `date` (instance) | crate rc 216, `40.19  DATE argument 2, "S", is not in the format described by argument 3, "N"` raised **inside** `CoreClasses.orx`'s own body, where the oracle answers `2020-01-02T00:00:00.000000` |
| `timeOfDay` (instance) | the same shape on `TIME`: `40.19  TIME argument 2, "L", is not in the format described by argument 3, "N"` |
| `toTimezone`, `toUtcTime`, `utcDate`, `utcIsoDate` (instance) | the crate answers as though the instance were UTC; the oracle applies the machine's offset — `2020-01-02T03:04:05.678901Z` against `…+02:00`. Measured: under `TZ=UTC` both sides agree, so these four are the offset and nothing else |
| `today` (class) | the crate answers the **UTC** date, the oracle the **local** one. Found only because a sweep ran after local midnight; invisible the other twenty-two hours a day |

**`Object~identityHash` is not among them.** It is `unstable [the oracle]`, and it is a licensed
divergence — deviation 4, `dispatch.rs:4232`, which records that ten oracle runs gave ten values.

**Task 5's overlap row is in the table and untouched**:
`Package	local	instance	loud	method "LOCAL" of class "Package"`. Whichever of the two tasks
lands second says so; nothing here moved it.

**The two `unstable` rows needed two different probes.** `DateTime new` on the class arm is
`unstable [this crate]` — its two engine runs differ, because its answer is the instant it is
evaluated at. `Object identityHash` is `unstable [the oracle]` — its two *oracle* runs differ.
Neither probe sees the other's case.

---

## The controller's challenge, and what came out of it

The controller read the committed table and said the seven `diverge` rows were mostly artifacts,
measuring `.DateTime~new~date`, `~timeOfDay` and `~utcIsoDate` on the oracle twice, 11 ms apart, and
finding it disagreed with itself.

**The premise did not apply to my rows, and I measured that rather than asserting it.** This table
does not send to `.DateTime~new`: `RECEIVER_OVERRIDES` replaces it with
`.DateTime~fromIsoDate('2020-01-02T03:04:05.678901')`, a fixed instant, for exactly the reason the
controller was pointing at. Under **my** receiver, five oracle passes spread over time:

| probe | distinct oracle answers over five passes |
|---|---|
| `date`, `timeOfDay`, `toTimezone`, `toUtcTime`, `utcDate`, `utcIsoDate` (my receiver) | **1 each** |
| `identityHash` | **5** |
| `DateTime new`, class arm | **5** |
| `timeOfDay`, `utcIsoDate` under the controller's `.DateTime~new` | **5 each** — their finding, reproduced |
| `date` under the controller's `.DateTime~new` | **1** — stable even there, at day granularity |

**So one of the seven was a misclassification and six were not.** `identityHash` is the one, and it
is now `unstable [the oracle]`, derived from the oracle's own two runs and not from any list.

**The controller's principle was right even though the evidence was not**, and taking it seriously
is what found two more things. The "your stability window is too short" point lands squarely — just
not on the rows named. Chasing it produced:

* **a seventh defect**: `DateTime~today` answers the UTC date, which no probe with a window shorter
  than a day can see, and which I found only because a sweep ran at 00:08 local;
* **a flaky-verdict class I would otherwise have shipped**: that row reads `answers` for
  twenty-two hours a day and `diverge` for two, so *either* committed value is a daily gate failure.
  The three-environment comparison is what makes it read `diverge` at every hour.

**One design I built and threw away, recorded because the counts did not show it was wrong.** The
first version of the fix folded "matches some environments but not all" into a fifth evidence value,
`unstable [the environment]`. Its numbers came out exactly as predicted — and it was still wrong:
it recorded a method *known* to be broken as one the harness could not classify. Recording a known
defect as unclassifiable is the wrong error to make, so the final shape has no such cell: anything
short of agreement everywhere is a divergence.

---

## The load-bearing premise, verified rather than inherited

D76 rests on "the refusal fires **before** argument handling", which the spec took from eight
sampled methods. I checked it two ways before building on it.

**Structurally.** `Interp::invoke` (`crates/rexx-exec/src/dispatch.rs:2164`) calls
`Interp::invocable` (`:2121`) **first** — before `seam::clear` and before any arity check — and
`invocable`'s closing `Loud::native_method` (`:2146`) is the refusal. The arity checks
(`Raised::too_many_method_arguments`, `too_many_external_arguments`) live only in the `Native` and
`External::Implemented` arms, i.e. only where a body was found. So the resolution refusal cannot
depend on the argument count.

**By measurement, over the whole set rather than a sample.** Predictions written first, in
`scratchpad/task-2/premise-prediction.txt`:

| | prediction | result |
|---|---|---|
| P1 | all 517 own-method `loud` rows give the identical status and identical `stderr` when re-sent with **one** argument | **CONFIRMED, 517 of 517** |
| P2 | the 156 downstream `loud` rows are **not** predicted invariant | **CONFIRMED**, 115 same and **41 differ** |
| P3 | no `answers`/`diverge` row is `is_loud` at zero arguments | **CONFIRMED, 0** |
| P4 | no `answers`/`diverge` row's zero-argument `stderr` says `is not implemented` | **CONFIRMED, 0** |

P2's 41 is the honest limit of a zero-argument sweep and it is why the table's header says the
classification is of that one send: a `loud` row may answer at another arity, and an `answers` row
may refuse at one.

**The send form is not load-bearing either, and that was measured too.** Four forms —
`say o~'M'()`, the same without the `say`, the bare `o~M`, and a literal receiver `'abcdef'~M` —
classify `String`'s 74 alphabetically named instance rows **identically**, 69 `loud` under each.

---

## The spec's sampled table was three classes wrong, and the plan and spec now carry the correction

| class | instance rows | sampled `loud` / reached | measured `loud` / reached |
|---|---|---|---|
| `String` | 118 | 80 / 38 | **113 / 5** |
| `Array` | 44 | 27 / 17 | 27 / 17 |
| `Directory` | 31 | 18 / 13 | 18 / 13 |
| `MutableBuffer` | 51 | 49 / 2 | **51 / 0** |
| `Method` | 17 | 14 / 3 | 14 / 3 |
| `TimeSpan` | 47 | 4 / 43 | **7 / 40** |
| | **308** | **192 / 116** | **230 / 78** |

`Array`, `Directory` and `Method` reproduce exactly. **I did not reconstruct what the sample did.**
What I ruled out is the probe form (four forms agree, above), and I checked the `String` figure
against the crate's own table by hand: `length`, `makeArray`, `makeString`, `reverse` and `upper`
answer and everything else refuses, which is exactly `NATIVE_METHODS`'s `String` rows. `TimeSpan`'s
seven all refuse downstream on a `String` or `Object` method, never on the `TimeSpan` method itself.

The premise the phase rests on is **stronger** after this, not weaker: 230 of those 308 rows are
hollow and green in table C, and over the whole documented set the figure is 673 of 1347.

---

## Controls

Every prediction written before the run, in `scratchpad/task-2/control-predictions.txt` and
`scratchpad/task-2/oracle-stability-prediction.txt`.

### Control 1 — the D76 gate, shown to fail (required)

**Mutation:** one line added to `dispatch.rs`'s `NATIVE_METHODS`,
`("String", "SUBSTR", Arity::Counted, native_reverse)`, so a `loud` row answers a reversed string
where the oracle raises `93.903`. **A wrong answer, not a removed implementation** — the shape the
controller warned about, from Task 1's own failed control.

**Predicted:** exit 101; the **regression** assertion, not the drift one; naming
`String substr (instance arm): loud -> diverge`.

**Ran. CONFIRMED**, rc 101, `regressions this run: 10`, that row first among them.

**Ten rows, not one, and that is the finding inside the control.** Nine `DateTime` rows were `loud`
*because* their `CoreClasses.orx` bodies reach `String~substr`; a wrong body there made all of them
answer wrongly. The one non-regressive drift is the same mechanism from the other side:
`DateTime daysInMonth` moved from `loud [method "SUBSTR" of class "String"]` to
`loud [method "WORD" of class "String"]` — the body got past `substr` and hit the next gap.

### Control 1b — the refresh refuses to launder it

**Predicted:** on the mutated tree, `REXX_METHOD_BODIES_REFRESH=1 …` also exits 101 and leaves
`corpus/method-bodies.txt` byte-identical.

**Ran. CONFIRMED**, rc 101, and `cmp` against the pre-control copy is byte-identical. Reverted;
`cmp` of `dispatch.rs` byte-identical.

### Control 2 — D73's guard, shown to fail by making one class constructible (required)

**Mutation:** `("Buffer", "NEW", Arity::Counted, native_new)` added to `NATIVE_CLASS_METHODS`.

**Predicted:** exit 101 naming `Buffer` on the tree-walker **and** on the ir, and **not** the oracle
and **not** `Pointer`.

**Ran. CONFIRMED**, rc 101, `["Buffer (.Buffer~new) on the tree-walker", "Buffer (.Buffer~new) on
the ir"]`, with the oracle still at rc 163. Reverted; `cmp` byte-identical.

### Control 3 — the `answers` half of the rule has a witness

Not required by the brief; run because a rule half nobody has seen fire is a rule half nobody has
checked.

**Mutation:** one committed `loud` row hand-edited to `answers` (`Array append`), no code change.

**Predicted:** exit 101, the regression assertion, `answers -> loud`.

**Ran. CONFIRMED**, rc 101,
`Array append (instance arm): answers -> loud [method "APPEND" of class "Array"]`. Reverted; `cmp`
byte-identical.

### Control 4 — the row-set check

**Mutation:** one row deleted from the committed table. **Predicted:** exit 101, a **structural**
failure, not a regression and not drift.

**Ran. CONFIRMED**, rc 101, naming the row as documented by `class-methods.txt` and carried by
nothing. Reverted; `cmp` byte-identical.

### Control 5 — the receiver cross-check against gate table C

**Mutation:** `instance_receiver`'s fallback changed from `.{name}~new` to `.{name}~new()`.

**Predicted:** exit 101 with structural failures naming Alarm, File, **Singleton**, StackFrame,
Stream, StreamSupplier, Ticker.

**Ran. CONFIRMED that it reddens, PREDICTION PARTLY FALSIFIED on the membership**: rc 101 with seven
structural failures naming Alarm, File, **Pointer**, StackFrame, Stream, StreamSupplier, Ticker.
`Singleton` and `Buffer` have **only class-arm rows** in `class-methods.txt`, so the check skips
them; `Pointer` has five instance rows and is in. The set is "classes with instance-arm rows and no
construction expression", not "not-covered classes". Reverted; `cmp` byte-identical.

### Control 6 — the header cannot drift either

**Mutation:** one word of the committed file's header changed, every row untouched. **Predicted:**
exit 101 with `0 row(s) drifted, and its text still differs, so what moved is the header`.

**Ran. CONFIRMED**, rc 101, that exact text. Reverted; `cmp` byte-identical.

### Control 7 — Task 1's failed control, run against this table

Task 1's first attempt at a negative control deleted `("String", "REVERSE", …)` from
`NATIVE_METHODS`; the mutation was live and **gate table C stayed green at rc 0**, because its
method rows are `hasMethod` readbacks. The controller relayed it as a warning. It is also the
sharpest available demonstration that this table sees what table C cannot, so I predicted and ran
it — see control 13, which supersedes it: by the time I got to it the second pass had been rebuilt,
and the result is reported there against the final shape.

### Control 8 — the oracle-reproducibility probe, on real data

**Predicted** (before the refresh that measured it): `Object identityHash` moves `diverge` →
`unstable [the oracle]`; `DateTime new` stays `unstable`, evidence moving from `-` to `this crate`;
the six `DateTime` rows stay `diverge`; no `answers` row moves; the sweep gets ~7 s slower.

**Ran. CONFIRMED**, all of it. The diff against the previous table was exactly two lines. The
timing prediction was right and I misread it once: the refresh run's 105 s included a rebuild, and
the sweep itself went 68 s → 76 s.

### Controls 10 to 12 — the environment probe, and one design thrown away

**C10 as first built was wrong and the run said so.** `DateTime today` came out `diverge`, not
`unstable`, because at 00:08 local the two sides genuinely disagree. That is the seventh defect,
and it meant the row's verdict turns on the hour.

**C11** rebuilt it as a three-environment match count with `1..2` folded into
`unstable [the environment]`. Predicted counts: loud 673, answers 665, diverge 6, unstable 3, oracle
2026 runs. **All five confirmed — and the design was still wrong**, because it recorded a known
defect as unclassifiable.

**C12** is the final shape: `matches == 3` is `answers`, anything less is a divergence after the
oracle has been asked to repeat itself. **Predicted:** loud 673, answers 665, diverge 7, unstable 2;
`today` → `diverge` and stably so; `identityHash` → `unstable [the oracle]`; `DateTime new` →
`unstable [this crate]`; and **the refresh refuses**, because `answers -> diverge` on `today` is a
regression against the draft baseline.

**Ran. CONFIRMED, all five**, the refusal included. I rebuilt the draft rather than weakening the
rule, and the rebuilt table differs from the draft in exactly one row, `DateTime today`.

**Shown stable rather than argued stable:** `Etc/GMT+12` answers `2026-09-03` and `Etc/GMT-14`
answers `2026-09-04` at the same instant, so the crate's single answer can match at most one of
them, at every hour of every day.

### Control 13 — the discriminator the controller asked for

*"It needs a control showing it distinguishes a genuine divergence from a nondeterministic one."*

**Mutation:** control 1's, re-applied against the final second-pass shape.

**Predicted:** exit 101, the regression assertion, `String substr (instance arm): loud -> diverge`
among the moved rows, and **no row reading `-> unstable`**.

**Ran. CONFIRMED**, rc 101, ten `loud -> diverge` regressions and **zero** `-> unstable`. Taken with
control 8 that is both directions: the same second pass moves `identityHash` **out** of `diverge`
and leaves ten genuine divergences **in** it. Reverted; `cmp` byte-identical.

---

## Gates

Run from `rust/` by a background job that wrote each status to a file as it went and a pidfile the
waiter keyed on; every status read unpiped in the turn this report was committed.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

`memcap` was present (`command -v memcap` → `/home/moritz/.local/bin/memcap`).

**The phase gate, over both tables:**

| command | exit |
|---|---|
| `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |
| `REXX_PHASE_GATE=5d …` (same command) | **101** |

G7's 101 is the expected pre-Task-5 state: one row, `requires__namespace__subkeyword`, which
Task 1 re-owned to `5d` and Task 5 implements. It is not a result of this task.

Table C at the gated tree, from G6's log, unchanged by this task: `5a 135/0`, `5b 6/0`,
`5c 1225/0`, `6 13/13`, `7 94/82`, `deferred-rexxcontext-stackframes 10/10`,
`never-expected-to-agree 5/5`, `gated by this run: 0`. **13 + 82 + 10 + 5 = 110 — the open rows did
not move**, which is what D77 requires of this task. Corpus differential `354 of 354`.

**This section was filled in by the controller, not by the task agent.** The agent templated it and
died between the gate run finishing at 00:48:46 and committing; the statuses above are read from
the run's own `statuses.txt`, and the staged tree was confirmed to be `163dd8f92ef2c8c6abd0e0c5fee0cddb68b9a0c7`,
byte-identical to the tree the gates measured. Nothing else in this report was written by anyone
but the agent.

---

## What I did not do

* **I implemented no method body (D77).** No `crates/*/src/` file changed: the diff is two test
  binaries, one added method on the shared oracle wrapper, one committed table, and two documents.
  No `Family::Timer`, `Family::Stream`, `Family::File` or `Family::Queue` entry point was touched,
  and no `class-set.txt` row was flipped to `covered`.
* **I did not fix the seven `diverge` rows**, as instructed. They are reported above and in the
  spec's D76, including the two the controller's challenge uncovered.
* **I did not create `corpus/phase-5d.txt`.**
* **I committed no corpus program**, so `corpus/unfiled.txt` is untouched and `corpus.rs`'s
  `every_lang_program_is_run_or_named_unfiled` sees nothing new. **The two test binaries are not
  interim witnesses for Task 6 to fold in and delete** — they are the instrument, and both stay.
* **I did not read the benchmark axes.** No execution path changed; the only Rust outside `tests/`
  is none at all.
* **I did not run `clippy` from a clean target directory.** `rust/CLAUDE.md` calls a same-session
  green provisional and asks for the clean-directory run at a phase boundary; this is not one.
* **I did not parallelise the sweep.** 84 s release is the serial figure and the debug gate carries
  the whole of it. `run_program` reserves an interpreter stack per run and I did not establish that
  concurrent in-process runs are safe.
* **I did not shift the environment on this crate's side.** `TZ` is per process and the crate runs
  in process, so a body that correctly answered differently per zone would read `diverge` here.
  Named at the site and above; nothing in the tree has that shape today.
* **I did not re-time the debug gate after the environment probe landed.** It was 275 s for this
  binary before; the probe adds oracle subprocesses, which cost the same in either profile, so the
  figure to expect is around 290 s — but that is arithmetic, not a measurement, and gate 5's own
  duration is the one to read.
* **I did not reconstruct the spec's original sample.** I ruled out the probe form and hand-checked
  `String`; what produced 80 and 38 is unknown.
* **Two arms of `regressed` have no run-time witness**: `answers -> unstable` and
  `unstable -> diverge`. Only `the_regression_rule_is_the_two_rules_it_states` covers them, which is
  a restatement of the table rather than a mutation of the tree.
* **I did not control `check_overrides`' arm** (an override naming a class the set does not carry),
  nor the `Body::parse` panic on an unknown verdict word.
* **The `unstable` cell has two occupants**, one per probe. If either stops being nondeterministic
  its probe loses its only subject and should be re-examined rather than kept.

---

## Files

* `rust/corpus/method-bodies.txt` (new, generated)
* `rust/crates/rexx-exec/tests/method_bodies.rs` (new)
* `rust/crates/rexx-exec/tests/unreachable_classes.rs` (new)
* `rust/crates/rexx-exec/tests/support/oracle.rs` (`Oracle::run_in_zone`)
* `docs/superpowers/specs/2026-09-03-phase-5d-foundation.md` (§2's table, D76's record)
* `docs/superpowers/plans/2026-09-03-phase-5d.md` (the premise, Task 2's shape table, open
  question 1)
* `docs/superpowers/records/2026-09-03-phase-5d/task-2-report.md` (this file; the briefed
  `.superpowers/sdd/` path is git-ignored, and `docs/superpowers/records/` is where every earlier
  task report is committed)

**Scratch left behind, not deleted:**
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task-2/`
(the prototype sweeps, the premise, probe-form, oracle-stability and timezone measurements with
their staging trees, the pre-control copies, every control log, and four gate-run directories —
three voided and one final) and
`/home/moritz/dev/repos/claude-build-scratch/task-2/target/` (a release build tree on real disk,
made before I established that `rust/target` was warm and usable; every gate ran against
`rust/target`).

**Stray processes:** `pgrep -a -x rexx` and `pgrep -a -x cargo` both report nothing after every
sweep and after every kill; every oracle run goes through `support::oracle`'s deadline and every
crate run through `watchdog::run_bounded`, but neither bound survives its parent being killed, which
is how the first voided run left two behind.
