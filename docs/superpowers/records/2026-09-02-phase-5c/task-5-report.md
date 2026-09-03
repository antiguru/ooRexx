# Phase 5c Task 5 — the reflection classes

**BASE:** `0b8c9c7ce`, tree clean. **Committed at `563074c18`.** Everything below was measured on this machine on 2026-09-03
against the pinned 5.3 oracle at `/home/moritz/dev/repos/ooRexx/build`, three descriptors read
separately, from a fresh empty directory per program, on `REXX_ENGINE=ir` and
`REXX_ENGINE=tree-walker` both.

---

## The one-paragraph version

**Five of the six moved: table C's 5c not-yet-agreeing count is 233 → 137**, 96 rows.
`Package` (38), `Class` (31), `Method` (17) and `Routine` (8) are `covered` through Task 1's
construction route on `.Class~package`, `.Object~subclass('k')`, `.Object~method('objectName')` and
`.routines~r` — each the reference's own syntax for obtaining an instance, and **each answering a
real object this crate already builds** rather than a fabricated one, so none of the four needed the
`~new` Task 2 measured as blocked. `TraceObject` (2) moved on the crate side: `receiver_kind`'s
`StringTable` arm now tests **descent** rather than class identity and carries the object's own
class. `Routine` is what made the construction cell grow — `.routines~r` needs a `::ROUTINE` below
the readbacks, so `class-set.txt` has an eighth `directives` column. **`StackFrame`'s 10 did not
move**: both documented routes are still refused at rc 120 on both engines, and clearing either
needs a method body plus an object model this crate has none of.

---

## The brief's framing, and where my answer departs from it

> **These are not blocked on a bare `~new`.** Task 2 measured each and left them precisely because
> each needs a real thing to reflect over: `Class` a class clone, `Method` a compiled executable,
> `Package` a loaded package, `Routine` a compiled executable.

That is right about `~new` and it is what made the four look expensive. What it misses is that the
image **already holds** a real thing of each of those four kinds and the reference names the route
to it. Measured one program each, rc 0 and byte-identical to the oracle on three descriptors on both
engines, and **none of the four needed a change to the interpreter** — the only interpreter change
in this commit is `TraceObject`'s:

| class | expression | `~class~id` | reference |
|---|---|---|---|
| `Class` | `.Object~subclass('k')` | `Class` | `fundclasses.xml:921` *subclass*; `:949` "For subclasses of the Object class, the default metaclass is the Class class"; the book's own example line is `myclass=.object~subclass("My class")` at `:724` |
| `Method` | `.Object~method('objectName')` | `Method` | `fundclasses.xml:700` *method*; `:713` "Returns the method object for the receiver class's definition for the method name" |
| `Package` | `.Class~package` | `Package` | `fundclasses.xml:881` *package*, whose example is `say .Class~package~name -- REXX` at `:897` |
| `Routine` | `.routines~r` + `::routine r` | `Routine` | `oneof.xml:438` *The ROUTINES StringTable*; `:449` "The StringTable values are the routine objects"; example `.routines~talk~call(...)` at `:457` with `::routine talk` at `:458` |

So `.Class~new`, `.Method~new`, `.Package~new` and `.Routine~new` are **still loud refusals here**
and no row needed them. Re-measured at head: `.Class~new('k')`, `.Method~new('m','nop')`,
`.Routine~new('r','nop')` and `.Package~new('p','nop')` are each rc 120
`rexx-exec: method "NEW" of class "..." is not implemented (Phase 5)` on both engines, against an
oracle that answers `Class`, `Method`, `Routine` and `Package` at rc 0. That is the phase's own premise applied to a constructor rather than to a
method: `'abc'~hasMethod('abbrev')` is 1 while `'abc'~abbrev('a')` is rc 120, and the row agrees.
It is worth saying plainly because it is the half a reader could mistake for the work being done:
**this task did not implement a single one of the four constructors Task 2 named.**

I did **not** fabricate an instance for any of them, which is the thing Task 2 declined and was
right to. Each object above is one the crate builds through a path of its own, which is why the
natives on its instance behaviour can read it: `.Class~package` is the object `~package` answers,
`.Object~method(...)` the object `Class~method` answers, and `.routines~r` the object a `::ROUTINE`
directive puts in `.ROUTINES`.

---

## What was built

### 1. `crates/rexx-extract/src/docs/classes.rs`

* **`CONSTRUCTION_PROGRAMS` gains `CLASS`, `METHOD`, `PACKAGE` and `ROUTINE`** — the four
  expressions in the table above.
* **`CONSTRUCTION_DIRECTIVES`** — a new const, class name to the directive the probe carries below
  its readbacks. One entry, `("ROUTINE", "::routine r")`.
* **`Route`** — `Coverage::Covered(String)` becomes `Coverage::Covered(Route)`, carrying the
  expression and the optional directive. The invariant Task 1 built is unchanged: there is still no
  way to reach `Covered` without naming the expression.
* **Two new assertions.** `coverage_of` refuses a directive committed for a class with no program
  (nothing would derive a probe to carry it), and `class_rows` refuses a directive for a name the
  class set does not carry — the same pair `CONSTRUCTION_PROGRAMS` already had.

### 2. `corpus/docs/class-set.txt`

Gains an eighth field, `directives`, appended so the seven before it keep their indices. Every row
but `Routine`'s carries `-`. The header states what it holds.

### 3. `crates/rexx-exec/tests/gate_table_c.rs`

* **`Construction::Constructs { program, directive }`**, parsed by `construction_of` from three
  fields rather than two. A directive on a non-`covered` row is a panic naming the class.
* **`method_probe_text` emits the directive** after the readbacks, and names it in the header. The
  `Constructs` header is one arm with the directive line interpolated, not two arms, so the two
  shapes cannot drift.
* **`check_interpolated_text` polices the directive** the way it already policed the expression —
  a single trimmed line free of `*/`.

### 4. `crates/rexx-exec/src/dispatch.rs` — the `TraceObject` fix

`Primitive::StringTable` becomes `Primitive::StringTable(ObjRef)` carrying the receiver's own class,
and `receiver_kind`'s guard becomes `classes.is_a(native.class(), model.string_table)` — descent
rather than identity. `receiver_behaviour` and `native_class` then read the carried class instead of
`model.string_table`.

**Why the class had to be carried and not just the arm widened.** `TraceObject` declares
`makeString` and `StringTable` does not, so a widened arm that still read the behaviour off
`.StringTable` would answer `StringTable`'s name set for a `TraceObject` — the wrong-object failure
the brief warns about, arrived at from the other direction. Measured, oracle rc 0:
`.TraceObject~new~hasMethod("makeString")` is `1` and `.StringTable~new~hasMethod("makeString")` is
`0`.

**`Directory` is unaffected and that was measured rather than reasoned.** Oracle rc 0:
`.Directory~isSubclassOf(.StringTable)` is `0`, `.Directory~superClasses` is `The Object class The
MapCollection class`, and `.StringTable~subclasses` names `The TraceObject class` and nothing else.
`.RexxContext`, `.Message`, `.Package`, `.Method` and `.Routine` are each `0` for the same question,
so no arm below the widened one is shadowed by it.

### 5. The probes

`class__instance.rex`, `method__instance.rex`, `package__instance.rex` and `routine__instance.rex`
are regenerated; `traceobject__instance.rex` is untouched — it was already `covered` on
`.TraceObject~new` and only the crate had to change.

### 6. `docs/superpowers/plans/2026-09-02-phase-5c.md`

The target section gains two paragraphs: what landed, and `StackFrame`'s remaining blocker with its
re-measurement. That section still said all six were addressable and unmoved, which is the fact the
next brief would have regenerated from. **It is the one tracked file outside `rust/` in the commit**,
flagged here because the controller owns the plan.

---

## The measurements

### The five that moved

Each is the **committed probe file**, run from a fresh empty directory, against a `rexx-run` whose
SHA-256 was read before the sweep and again after it and did not change
(`5154a7af6d1ef0cb0a0557d2da6bb691f02fc2d54b0225875e3cf30cf4a2c255`):

| probe | rows | oracle | `ir` | `tree-walker` |
|---|---|---|---|---|
| `class__instance.rex` (`o = .Object~subclass('k')`) | 31 | rc 0, 31 lines, all `instance 1` | identical on all three descriptors | identical |
| `method__instance.rex` (`o = .Object~method('objectName')`) | 17 | rc 0, 17 lines, all `instance 1` | identical | identical |
| `package__instance.rex` (`o = .Class~package`) | 38 | rc 0, 38 lines, all `instance 1` | identical | identical |
| `routine__instance.rex` (`o = .routines~r`, `::routine r` below) | 8 | rc 0, 8 lines, all `instance 1` | identical | identical |
| `traceobject__instance.rex` (`o = .TraceObject~new`) | 2 | rc 0, 2 lines | identical | identical |

**The answers are all `1`, on both sides** — worth stating, because a group of `0`s would compare
equal just as well and would mean the class answers none of its documented names.

### The routes' answers do not depend on where the probe stands

Task 3's check, repeated for these five. Each probe run in three shapes -- as committed, with the
header comment stripped so `o = ...` is line 1, and with 40 blank lines before the body -- on the
oracle and on both engines:

* the three shapes' oracle `stdout` are byte-identical to each other for all five, 31 / 17 / 38 / 8 /
  2 lines;
* every one of the ten repositioned probes is rc 0 and byte-identical to the oracle on all three
  descriptors on both engines.

`.Class~package` is the reason to ask: `~package` answers *a* package, and a route that answered the
**probe's** package would have compared stably here and meant something different. `.Class` is the
built-in class, so its package is the interpreter's own wherever the send stands. Measured, rc 0 on
the oracle and on both engines: `say .Class~package~name` is `REXX`, and it is still `REXX` in a
program that carries a `::ROUTINE` of its own -- so the object is not the probe's package. That
line is the book's own example (`fundclasses.xml:897`).

### No silent wrong answer came with any of the five routes

Thirteen `Object`-level observables asked of each of the five objects, **one program per pair**
rather than one loop over them (a loop dies at the first loud refusal), on the oracle and on both
engines, three descriptors compared separately, all against the same binary hash above:

```
65 pairs    SAME 50    LOUD 10    DIFF 5
```

**Every one of the fifteen non-`SAME` rows is a property of this crate that predates these routes**,
and each was measured on objects that predate them rather than assumed:

* **`~identityHash~length`** — all five `DIFF`s, `16` on the oracle against `3` here (`10` for the
  class object). This is Task 2's finding, unfixed and untouched: `.Object~new`, `'abc'`,
  `.Array~new` and `.StringTable~new` answer `16` against `3`, `10`, `3`, `3`.
* **`~isInstanceOf(.Object)`** — all five `LOUD`, `method "ISINSTANCEOF" of class "Object" is not
  implemented`. A plain `.Object~new` gets the same refusal.
* **`~copy`** — `LOUD` for `Method`, `Package`, `Routine` and `TraceObject`; `SAME` for the class
  object, because the oracle raises for that receiver too. `'abc'~copy` is the same refusal with no
  constructor involved.
* **`~request('ARRAY')` on the `TraceObject`** — `LOUD`, `MAKEARRAY` of class `StringTable`. This
  crate has no `MAKEARRAY` for any receiver, which is Task 2's measurement.

`say o`, `~string`, `~class~id`, `~objectName`, `~defaultName`, `~isA(.Object)`,
`~request('STRING')` and `~hasMethod('ZORK')` are byte-identical on all three descriptors on both
engines for all five objects.

### The `TraceObject` is a real collection, not an object that merely answers the right names

The `Stem` hazard asked properly. `.TraceObject~new` runs `CoreClasses.orx:4005`'s Rexx class-side
`NEW`, which stores three entries, bumps a class-side counter and returns. One program per
observable, oracle against both engines:

| program | oracle | crate |
|---|---|---|
| `o["NUMBER"]` | `1` | identical |
| `o["OPTION"]` | `N` | identical |
| `o["TIMESTAMP"]~class~id` | `DateTime` | identical |
| `o["TRACELINE"]` | `The NIL object` | identical |
| `o~at("NUMBER")` | `1` | identical |
| `o~makeString` | `?` | identical |
| `.TraceObject~counter` | `1` | identical |
| `say o` | `?` | identical |

`~makeString` there is the class's own Rexx body running: `CoreClasses.orx:4191`'s instance method
forwards to `:4073`'s `makeStringImpl`, whose option-`N` arm is `return str(traceObj["TRACELINE"])`.
It is not a default rendering that happens to agree.

### `StackFrame`, the one that did not move

10 rows. Re-measured at head against the final binary, from a fresh directory, both
engines:

```
.context~stackFrames[1]~class~id
  crate   rc 120  rexx-exec: method "STACKFRAMES" of class "RexxContext" is not implemented (Phase 5)
  oracle  rc 0    StackFrame

signal on syntax ; zz = 1/0 ; syntax: o = condition('O')~stackFrames ; say o~class~id o~firstItem~class~id
  crate   rc 120  rexx-exec: CONDITION option "O" answers a Directory, which is not implemented
  oracle  rc 0    List StackFrame
```

`.StackFrame~id` answers `StackFrame` on both sides at rc 0, so the class exists here; instances do
not. **The directive trailer this task built does not reach it**, and that is the one thing worth
adding to Task 3's account: a trailer goes *below* the readbacks, and the condition route needs a
`SIGNAL ON` and a label *above* them. Even with that, `condition('O')` needs a `Directory` this
crate does not build, and `.context~stackFrames` needs a `RexxContext` method body — which the brief
forbids implementing to move a row — on top of a `StackFrame` object with no model here. It is a
handover, with no owner inside 5c.

---

## Table C, before and after

`cargo test --release -p rexx-exec --test gate_table_c`, report read from stderr:

| | BASE `0b8c9c7ce` | after |
|---|---|---|
| 5c rows | 1347 | 1347 |
| **5c not yet `agree`** | **233** | **137** |
| `agree` (whole table) | 1255 | 1351 |
| `diverge-both` | 29 | 27 |
| `unanswered` | 204 | 110 |
| loud | 68 | 66 |
| 5a / 5b not yet `agree` | 0 / 0 | 0 / 0 |

96 rows moved — 31 + 17 + 38 + 8 + 2 exactly. **"Nothing else moved" was diffed rather than inferred
from the totals**: the two reports' per-group verdict blocks were extracted and compared line by
line, 96 group lines on each side, and exactly five differ — four `unanswered` → `agree` and one
`diverge-both` → `agree`, none in the other direction.

The residue, from the same report, is fully attributed:

```
Stem       27  diverge-both  5c-addressable, built and backed out by Task 2
File       50  unanswered    5d
Stream     24  unanswered    5d
StreamSupplier 8 unanswered  5d
Alarm       7  unanswered    method bodies, out of this phase
Ticker      6  unanswered    method bodies, out of this phase
Pointer     5  unanswered    D73, must not move
StackFrame 10  unanswered    handover -- above
```

That sums to 137, and 137 − 10 = the plan's 127.

---

## The controls

Each was run and each line below is read off the run's own output. After every one the edited files
were restored **from copies under the session scratchpad** — never `git checkout --` — and each
restore verified with `cmp` before the next step.

A, B1, B2 and C ran before the last two edits to `classes.rs` and `gate_table_c.rs` (a
`method_probe_text` refactor and two doc trims), so the `file:line:col` each panic printed no longer
points at the assertion. **The line numbers are dropped rather than adjusted** — the message text is
quoted verbatim and is what identifies the assertion.

**A — `covered` with no committed program.** `class-set.txt`'s `Routine` row's construction cell
hand-edited to `-`, the status left `covered`:

```
cargo test --release -p rexx-exec --test gate_table_c   ->  exit 101
  panicked at crates/rexx-exec/tests/gate_table_c.rs:
  class-set.txt records Routine as `covered` and carries no construction program for it.
  `covered` is exactly the claim that one is committed
```

**B1 — a construction directive on a `not-covered` row, at the gate.** `Alarm`'s directives cell
hand-edited from `-` to `::routine r`:

```
cargo test --release -p rexx-exec --test gate_table_c   ->  exit 101
  panicked at crates/rexx-exec/tests/gate_table_c.rs:
  class-set.txt records Alarm as `not-covered` and carries a construction directive for it.
  Only a `covered` row has a route to an instance
```

**B2 — the same mistake made in the extractor, where the row set could not carry it.**
`("ALARM", "::routine r")` added to `CONSTRUCTION_DIRECTIVES`:

```
cargo test --release -p rexx-extract --test extract_docs   ->  exit 101, 8 passed 1 failed
  panicked at crates/rexx-extract/src/docs/classes.rs:
  Alarm has a committed construction directive and no construction program, so nothing
  derives a probe that would carry the directive
```

**C — a `covered` route that does not construct, made *consistently* so nothing upstream of the run
can notice.** `CONSTRUCTION_DIRECTIVES` emptied, then the row sets and `routine__instance.rex`
regenerated, so the status, both committed columns and the probe's own text all agree and the probe
is `o = .routines~r` with no directive below it:

```
cargo test --release -p rexx-exec --test gate_table_c   ->  exit 101
  structural failures, which are red in every mode and are not verdicts `REXX_CORPUS_GATE`
  could relax:
    gate-tables/methods/routine__instance.rex: the oracle answered 0 line(s) where this
    row's probe asks for exactly 8. The row for Routine (instance arm) therefore has no
    answer to compare ...
```

**This is the control that matters most here**, and it does two jobs: it is Task 1's
`OracleShape::Exactly` re-run against the new machinery, and it is the evidence that the directive
trailer is **load-bearing rather than decorative** — without it the oracle itself answers nothing.

**D — the probe generator.** A scratch script re-derives all 96 probes from the committed row sets
and compares them with the committed bytes. It prints one line per probe, and the DIFFERS lines
were read three times: with `Class`, `Method` and `Package` newly `covered`, **exactly those three**;
with `Routine` added, **exactly `routine__instance.rex`**; and after every rewrite, **none**,
re-checked after the `method_probe_text` refactor that collapsed the two `Constructs` header arms
into one.
Without this the script's agreement with `method_probe_text` would rest on `check_probe_text` alone,
which reports a first differing line rather than telling you which side is wrong.

**E — the `TraceObject` fix is load-bearing.** `receiver_kind`'s descent test reverted to BASE's
identity test, everything else left as committed:

```
cargo test --release -p rexx-exec --lib dispatch::tests::a_string_table_subclass
  ->  exit 101, test result: FAILED. 0 passed; 1 failed; 760 filtered out
  assertion `left == right` failed
    left: (120, "", "rexx-exec: a message send to one of the interpreter's own objects is
                     not implemented (Phase 5)\n")
   right: (0, "TraceObject 1\nStringTable 0\n", "")
```

The run count is non-zero, so this is a failure and not a filter that matched nothing. The
gate-level half of the same control needs no separate run: `traceobject__instance` reads
`diverge-both=2` in the BASE report and `agree=2` after, which is the same fact from the other side.

---

## The test added

`dispatch::tests::a_string_table_subclass_answers_its_own_class_methods_and_entries`.

| half | what would redden it |
|---|---|
| `TraceObject 1` / `StringTable 0` | a build that read the behaviour off `.StringTable` rather than off the object — control E above |
| the entries line | an object that answered the right names over an empty body, which is the `Directory~new` shape Task 2 measured |

**"Can fail" is not "adds coverage", so the second question was asked.** The first half is caught by
`gate_table_c` as well — the two `TraceObject` rows are `diverge-both` at BASE — so what the test
buys there is an in-process witness rather than new coverage. The **second half is not covered by
anything else**: the committed probe asks `hasMethod` and nothing else, so a `TraceObject` that
answered both names over an empty entry table would leave that group `agree`. Every value in the
test is one the oracle produced, measured one program per row.

---

## Generated artifacts

* `corpus/docs/class-set.txt` and `class-methods.txt` — regenerated with
  `cargo run -p rexx-extract --bin rexx-extract-docs -- --oodocs ../oodocs --interpreter
  ../interpreter --out corpus/docs`. **What changed in each was derived from the diff rather than
  read off it.** `class-methods.txt`: 103 rows, and comparing the removed and added lines field by
  field, the columns that differ are exactly `status` and `reason` — the `(class, method, arm)` key
  sets are identical, and the rows are `Class` 32, `Method` 20, `Package` 40, `Routine` 11, which is
  each class's instance and class arms together. `class-set.txt`: all 63 rows gain the eighth field,
  62 of them `-` and `Routine`'s `::routine r`; four rows also change `status`, `reason` and
  `construction`.
* Four instance probes.
* **`corpus/refusal-sites.tsv` was not re-derived, and that was checked rather than assumed.** It
  cites `error.rs`, `lib.rs`, `native.rs`, `run.rs` and `trace.rs` by line number — measured over
  the file, `dispatch.rs` appears in it once, in a header sentence, with no line number — and this
  change touches none of those files. `the_table_holds_every_constructor_the_source_defines` is
  green in its own run and in the gates below, which is the check rather than my reading.

---

## A measurement trap this task walked into, and how it showed

**A stale binary outlived its revert, and the six-probe sweep taken with it was wrong.** After
control E restored `dispatch.rs`, `cargo test --release -p rexx-exec --lib` rebuilt the library test
target and left `target/release/rexx-run` as control E had built it. The probe sweep run immediately
afterwards reported `traceobject__instance` as **DIFF at rc 120** — a correct reading of a binary
that no longer matched the tree. It showed up only because that probe had been `SAME` earlier in the
session and nothing between could explain the change; the gate suite then rebuilt the binary, and
the same probe was `SAME` again.

**Every probe figure in this report is from a re-run after that**, against a binary whose SHA-256
was read before the sweep and again after it. `git status` said nothing about any of this — the
working tree was correct the whole time.

---

## Gates

Tree hash (`git diff HEAD | sha256sum`) before the first gate and after the last:
`ceddffaccb39fc2d74bad69fa9c44b283ae1df48bf186c0908fe6165c43b82b6`, unchanged. Each status was
written to its own file by the runner and read back unpiped.

| # | command (from `rust/`) | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

**Zero `test result: FAILED` lines across gates 3, 4 and 5**, which is the reading that matters
rather than the exit status alone, and each of the three reports **107** `test result: ok` binaries.
Gates 4 and 5 both report the corpus harness at **331 of 331 matching**, table C's 5c at **137** and
table D's 5c at **38 rows, 35 not yet `agree`** — table D unchanged, which is the plan's own Task 5
and not this one. `memcap` was present (`command -v memcap` → `/home/moritz/.local/bin/memcap`), so
gate 5 is the `memcap` form rather than the `ulimit` substitute.

Clippy's stderr is `Checking rexx-extract`, `Checking rexx-exec`, `Finished`, so its green is
evidence the linter looked at this change rather than reusing a warm result. **That check was
needed**: an earlier invocation in this session reported exit 0 with a bare `Finished ... in 0.04s`
and no `Checking` line at all, having re-examined nothing.

**An earlier run of the same five was started and abandoned, and it is not what the table reports.**
It reached gates 1, 2 and 3 green before I corrected four `oodocs` line citations, one of them in a
`classes.rs` doc comment — a tracked source file, so the bytes under test moved. It was killed
(runner and the orphaned `cargo` both, confirmed with `ps`) and the whole five restarted on the final
bytes. Its logs are under `gates/` in the scratchpad; the table above is `gates2/`.

---

## What I did not do

* **`StackFrame` is not `covered`** — 10 rows, blocked and measured above. Neither blocker is mine:
  one is a `RexxContext` method body, the other a `Directory` for `condition('O')`, and both then
  need a `StackFrame` object this crate has no model for.
* **I did not implement `Class~new`, `Method~new`, `Package~new` or `Routine~new`.** All four are
  still `rexx-exec: method "NEW" of class "..." is not implemented (Phase 5)` at rc 120 where the
  oracle constructs. The rows moved through a different documented route, which is what the phase's
  instrument asks for and is not the same claim.
* **`Stem`'s 27 rows are untouched**, per the brief. Nothing here makes them easier: they need a
  `Stem` string value or a refusal inside `Interp::to_text`, and my change touches neither.
* **No method body was implemented.**
* **The construction cell still holds one expression plus, now, one directive.** `Alarm` and
  `Ticker` need a trailing *clause* (`o~cancel`) rather than a directive, and I did not build that:
  nothing I can commit would exercise it, which is Task 1's reason and still the right one.
* **I did not re-measure `CONSTRUCTION`.** Its doc says nothing does, by decision; the four
  programs and the one directive I added are in `CONSTRUCTION_PROGRAMS` and
  `CONSTRUCTION_DIRECTIVES`.
* **I did not run the phase gate** (`REXX_PHASE_GATE=5c`): Task 6 turns it on and 5c is not in
  `CLOSED_PHASES`.
* **I did not touch table D**, which the plan's own Task 5 section is about. The plan's task
  numbering and this brief's disagree — the plan's Task 5 is surface D, `::OPTIONS` ×33 and
  `::REQUIRES` ×2 — and I worked the brief. Table D reads unchanged in the gate reports.

## What I created outside the repository

Nothing was deleted. Under the session scratchpad, `.../scratchpad/task-5/`:
`cmp3.sh`, `sweep.sh`, `genprobes.py`, `commitmsg.txt`, `gates/` (an abandoned run, below),
`gates2/`, `backup/`, `p/` (probe programs), `run.*` and `sw.*` directories (one per comparison),
and the gate and gate-table logs. No build tree was created anywhere, so nothing went under
`claude-build-scratch/`, and `df -h /tmp` read 51 G free before and after.
