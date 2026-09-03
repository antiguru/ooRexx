# Phase 5c Task 3 — surface C, the classes with no `~new`

**BASE:** `dcd4d4e9c`, tree clean. **Committed at `b4adf8a1e`.** Everything below was measured on
this machine on 2026-09-03
against the pinned 5.3 oracle at `/home/moritz/dev/repos/ooRexx/build`, three descriptors read
separately, from a fresh empty directory per program, on `REXX_ENGINE=ir` and
`REXX_ENGINE=tree-walker` both.

---

## The one-paragraph version

**Two of the four moved: table C's 5c not-yet-agreeing count is 280 → 237.** `RexxInfo` and
`RexxContext` are `covered` through Task 1's construction route on `.RexxInfo` and `.context`, and
each probe is rc 0 and byte-identical to the oracle on stdout, stderr and exit status on both
engines. **`StackFrame` and `VariableReference` are blocked and stay `not-covered`**: the crate
refuses both of `StackFrame`'s documented routes, and it evaluates `>x` to the referenced variable's
*value*, so a `VariableReference` object does not exist here to be obtained. The `>x` decay is a
**silent** divergence at rc 0 that predates this phase, and it is the finding this task's measurement
turned up. No method body was
implemented and no interpreter behaviour changed — the whole code change is one narrowed assertion
in `rexx-extract`, two table entries and two doc corrections.

---

## The brief's premise sentence is false, and it is the one that framed the task

> Three of the four are already byte-identical to the oracle on all three descriptors -- their probes
> match the refusal. **That is why their rows have not moved.**

Measured at BASE on the committed probes, three descriptors read separately, from a fresh empty
directory:

| probe at BASE | oracle | crate (`ir`, `tree-walker`) | identical? |
|---|---|---|---|
| `rexxinfo__instance.rex` | rc 159, `97.1 Object "a RexxInfo" does not understand message "NEW".` | rc 159, same bytes | **yes** |
| `rexxcontext__instance.rex` | rc 163, `93.967 NEW method is not supported for the RexxContext class.` | rc 120, `rexx-exec: method "NEW" of class "RexxContext" is not implemented (Phase 5)` | no |
| `stackframe__instance.rex` | rc 163, same shape | rc 120, same shape | no |
| `variablereference__instance.rex` | rc 163, same shape | rc 120, same shape | no |

**One of the four, not three.** The BASE gate report says the same thing from the other side. All
four groups read `unanswered` there, but only `RexxInfo`'s carries `loud=no`:

```
RexxContext        instance  loud=yes 5c  15  row(s): unanswered=15
RexxInfo           instance  loud=no  5c  28  row(s): unanswered=28
StackFrame         instance  loud=yes 5c  10  row(s): unanswered=10
VariableReference  instance  loud=yes 5c   4  row(s): unanswered=4
```

and its *what the loud rows are waiting on* block lists `method "NEW" of class "RexxContext": 15`,
`... "StackFrame": 10` and `... "VariableReference": 4`, with no line for `RexxInfo`.

This does not change what the task had to do — the construction route was still the work, and the
refusal still moves no row — but the sentence would have made two probes look finished that were
not. It is corrected in the plan.

---

## What was committed

### `crates/rexx-extract/src/docs/classes.rs`

* **`CONSTRUCTION_PROGRAMS` gains `("REXXCONTEXT", ".context")` and `("REXXINFO", ".RexxInfo")`.**
  Both are the book's own route, named in the same paragraph as the sentence `UNCONSTRUCTIBLE`
  already quotes. `utilityclasses.xml:7540`-`7545`: "Instances of the RexxContext class can only be
  obtained via the .CONTEXT environment symbol, or by invoking the context method of the StackFrame
  class. They cannot be directly created by the user." `oneof.xml:380` is that symbol's own section,
  *The Rexx Context (.CONTEXT)*, and `oneof.xml:120` is *The RexxInfo Object (.RexxInfo)*. The
  book's executable examples spell them `.context` and `.RexxInfo` (`fundclasses.xml:3963`, `:630`),
  which is what the cells carry.
* **`coverage_of`'s guard is narrowed** from *the class has a swept `~new` outcome and it is not
  `new`* to *the swept `~new` outcome is not `new`*, and its message is corrected with it: it used
  to read "a bare ~new that **does not raise** on the oracle", which is false in exactly the case it
  now fires in.
* Two doc corrections, because `.context` and `.RexxInfo` are not constructors:
  `CONSTRUCTION_PROGRAMS`' own doc and the `class-set.txt` header text both said a committed program
  carries "the book's own constructor syntax", and now say "the book's own syntax for obtaining an
  instance".

**Why the guard had to move, and it is not a weakening.** `RexxInfo` is the **one** class-set name
absent from `CONSTRUCTION`: measured over the two lists, `comm` reports `REXXINFO` and nothing else.
That is by construction — `CONSTRUCTION`'s sweep sends `~new` only to `.environment` entries
answering `~isA(.Class)`, and `.RexxInfo`'s entry is an *instance*, which is the same fact
`class-set.txt`'s header already records and `INSTANCE_ENTRY_CLASS` already names. The old condition
read a missing row as evidence that the bare `~new` constructs; the new one reads only the row that
exists. **The property the old condition was reaching for is asserted elsewhere and still is**:
`class_rows` asserts `derived == measured` over the class-entry names and `CONSTRUCTION`'s keys, so
a *class*-entry name missing from `CONSTRUCTION` reddens there. Adding a `REXXINFO` row to
`CONSTRUCTION` instead is the obvious alternative and that same assertion forbids it — **control E
below is that edit, run**.

### The row sets and the probes

`corpus/docs/class-set.txt` and `class-methods.txt` regenerated with

```
cargo run -p rexx-extract --bin rexx-extract-docs -- --oodocs ../oodocs \
    --interpreter ../interpreter --out corpus/docs
```

`class-methods.txt`'s diff is 43 rows and nothing else — 28 `RexxInfo` and 15 `RexxContext`, each
`not-covered` → `covered`, checked by reducing the diff to `(sign, class, status)` triples.

`rexxinfo__instance.rex` and `rexxcontext__instance.rex` regenerated by a scratch script.
**The script's header template was validated against five already-committed `Constructs` probes
before it wrote anything** — `timespan`, `array`, `string`, `circularqueue` and `message`, rebuilt
from their own class name and program and compared with the committed bytes: all five IDENTICAL. And
`check_probe_text` re-derives all 96 probes from the row set on every gate run and compares in both
directions, so a script that had drifted from `method_probe_text` would have reddened rather than
shipped.

---

## The two that moved

Run from a fresh empty directory, one directory per program, the committed probe file itself:

| probe | oracle | `ir` | `tree-walker` | rows |
|---|---|---|---|---|
| `rexxinfo__instance.rex` (`o = .RexxInfo`) | rc 0, 28 lines | identical on all three descriptors | identical | 28 |
| `rexxcontext__instance.rex` (`o = .context`) | rc 0, 15 lines | identical | identical | 15 |

### The route's answer does not depend on where the probe stands

The brief's warning about `.context` — that it is evaluated inside the probe, so a route whose answer
changes with the probe's own shape will not compare stably. **Run rather than argued.** Four shapes
of the same 15 readbacks, on the oracle and on `ir`:

| shape | oracle stdout | crate stdout |
|---|---|---|
| the committed probe | rc 0, the reference | rc 0, the reference |
| the same with the comment header removed, `o = .context` on line 1 | identical to it | identical to it |
| 40 blank lines before it and a trailing `nop` | identical | identical |
| the whole body moved inside an internal routine reached by `CALL` | identical | identical |

The reason it holds is that every observable in the probe is a `hasMethod` readback, which is a
question about the class; nothing renders the object. That is an argument, and the table is the
measurement.

### No silent wrong answer came with either route

The Stem-shaped hazard the brief names: a route that answers the right `~class~id` and the wrong
everything else. Twelve `Object`-level observables were asked of each of the two objects, **one
program per pair** rather than one loop over all of them (a loop dies at the first loud refusal), on
the oracle and on both engines, three descriptors compared separately:

```
SAME    18
LOUD     4    ~isInstanceOf and ~copy, on both objects
SILENT   2    ~identityHash~length, on both objects
```

**Each of the three is a property of this crate that predates the routes, and that was measured
rather than assumed** — the same three observables asked of `.Object~new`, `.Array~new` and `'abc'`:

| observable | oracle | crate |
|---|---|---|
| `~identityHash~length` | `16` for all three | `3`, `3`, `10` |
| `~isInstanceOf(.Object)` | `1` for all three | rc 120 `method "ISINSTANCEOF" of class "Object" is not implemented` for all three |
| `~copy~class~id` | `Object`, `Array`, `String` | `Object` at rc 0; rc 120 `method "COPY" of class "Object"` for the other two |

The `identityHash` rendering is Task 2's finding, unfixed and untouched here. `~copy` on `.RexxInfo`
and on `.context` is the crate's `COPY` refusal meeting an oracle that raises rc 163 for those two
receivers — loud on both sides of the difference, so nothing silent. **`say o`, `~string`,
`~class~id`, `~objectName`, `~defaultName`, `~isA(.Object)`, `~request('STRING')`,
`~request('ARRAY')` and `~hasMethod('ZORK')` are byte-identical on all three descriptors on both
engines for both objects.**

---

## The two that did not move

### `StackFrame` — 10 rows

The book names exactly two routes (`utilityclasses.xml:9401`-`9407`): "obtained via the .CONTEXT
environment symbol or from a condition object created for a trapped condition". **This crate refuses
both**, rc 120 on both engines, from a fresh directory:

```
.context~stackFrames[1]~class~id
  crate   rc 120  rexx-exec: method "STACKFRAMES" of class "RexxContext" is not implemented (Phase 5)
  oracle  rc 0    StackFrame

signal on syntax ; zz = 1/0 ; syntax: o = condition('O')~stackFrames
  crate   rc 120  rexx-exec: CONDITION option "O" answers a Directory, which is not implemented
  oracle  rc 0    a List of one, whose ~firstItem~class~id is StackFrame
```

**The second route's shape is not the first's, and I got it wrong before running it.** My first
probe was `condition('O')~stackFrames[1]`, by analogy with the `.context` one, and it answered an
object whose `~class~id` is `Object` — which I read as `.nil` and wrote down as *the condition route
does not reach a StackFrame on the oracle either*. That was false. Asked properly, the condition
object **does** carry the entry (`condition('O')~hasIndex('STACKFRAMES')` is `1`) and
`~stackFrames` answers a `List` of one item — `[1]` on a `List` is an index handle rather than a
position, so it is the subscript that answered `.nil`, not the entry that was missing. The two
routes answer different collection classes: `.context~stackFrames` is an `Array`,
`condition('O')~stackFrames` is a `List`.

What survives the correction is the conclusion, for a different reason than I first gave: the
condition route needs a trap — a `SIGNAL ON`, a raising clause and a label — where the construction
cell holds one expression, so `.context~stackFrames[1]` is the only committable shape and
`RexxContext~stackFrames` is what it needs. That is a **method body**; the brief forbids implementing
one to move a row, and it is a `RexxContext` method row's own subject besides. It also needs a
`StackFrame` object this crate has no model for. `.StackFrame~id` answers `StackFrame` on both sides,
so the class exists here; it is instances that do not.

### `VariableReference` — 4 rows

The book is narrower still (`utilityclasses.xml:12552`-`12557`): "It can only be created using a
variable reference term." **This crate evaluates `>x` to the referenced variable's value**, so the
term yields no object to obtain:

```
o = >vr
say 'instance' o~class~id           oracle VariableReference   crate String
say 'instance' o~hasMethod("name")  oracle 1                   crate 0
say 'instance' o~hasMethod("request")  oracle 1                crate 1
say 'instance' o~hasMethod("unknown")  oracle 1                crate 0
say 'instance' o~hasMethod("value")    oracle 1                crate 0
```

**rc 0 and empty stderr on both sides, both engines** — a silent divergence, not a refusal, and the
one shape of defect this phase's instrument is worst at seeing. It is deliberate and predates this
phase: `8b87195bc` (2026-08-03) added
`run::tests::a_variable_reference_decays_to_the_referenced_value`, which asserts `say >p` prints
`p`'s value. That assertion is *correct* — the oracle prints the value too, because `SAY` of a
`VariableReference` renders its referent — so the test is not what has to change; the divergence
lives one message send later, at `(>x)~class~id`, which nothing asserts on either side.

**What the row needs is the object, not a longer construction cell.** Task 1 predicted
`VariableReference` would need a clause sequence where today's cell holds one expression. Measured on
the oracle, it does not: `o = >vr` over an **uninitialised** `vr` is a single expression and answers
a `VariableReference` at rc 0. `VariableReferenceOp::evaluate`
(`expression/VariableReferenceOp.cpp:109`-`117`, read directly) is `getVariableReference`, a push and
a `traceOperator` — the variable's value is never read, which is why an uninitialised one is fine. So
the cell `>vr` is committable the day this crate builds the object.

Building it is not a method body — it is `eval`'s `VariableReference` arm producing a new primitive
instead of decaying — but it is object-model work with a blast radius through `to_text` (`say >p`
must keep printing the value), argument passing and the `>O>` trace arm, and it is exactly the shape
Task 2 backed `Stem` out over. **I did not build it, and I am not recommending it be squeezed into
this phase's remaining tasks**; it wants its own subject.

### Both are also in `Pointer`'s position

Each still refuses `~new` loudly at rc 120 where the oracle raises 93.967. Matching that refusal
would make both probes byte-identical on three descriptors and **move zero rows** — a `not-covered`
group's oracle `stdout` is empty and `OracleShape` reads that as `unanswered` by design, which is
Task 2's finding and the brief's own warning. Task 2 already put the same observation to whoever owns
D73 for `Pointer`, whose `PointerClass::newRexx` is a bare `reportException` that constructs nothing.
Read directly, these two are the same function body to the byte:
`StackFrameClass::newRexx` (`classes/StackFrameClass.cpp:124`-`129`) and
`VariableReference::newRexx` (`classes/VariableReference.cpp:97`-`102`) are each
`reportException(Error_Unsupported_new_method, ((RexxClass *)this)->getId())` followed by
`return TheNilObject`, under the same `// we do not allow these to be allocated from Rexx code...`
comment. So does `RexxContext::newRexx` (`classes/ContextClass.cpp:96`-`101`), which stops mattering
here only because its probe no longer sends `~new`. **Two more classes' worth of the same one
decision, and it is not mine to take.**

---

## Table C, before and after

`cargo test --release -p rexx-exec --test gate_table_c`, report read from stderr:

| | BASE `dcd4d4e9c` | after |
|---|---|---|
| 5c rows | 1347 | 1347 |
| **5c not yet `agree`** | **280** | **237** |
| `agree` (whole table) | 1208 | 1251 |
| `diverge-both` | 29 | 29 |
| `unanswered` | 251 | 208 |
| loud | 87 | 72 |
| 5a / 5b not yet `agree` | 0 / 0 | 0 / 0 |

43 rows moved — 28 + 15 exactly. **"Nothing else moved" was diffed rather than inferred from the
totals:** the two reports' per-group verdict blocks were extracted and compared line by line, 96
group lines on each side, and exactly two differ — both `unanswered` → `agree`, neither in the other
direction. The per-group lines after:

```
RexxContext        instance  loud=no  5c  15  row(s): agree=15
RexxInfo           instance  loud=no  5c  28  row(s): agree=28
StackFrame         instance  loud=yes 5c  10  row(s): unanswered=10
VariableReference  instance  loud=yes 5c   4  row(s): unanswered=4
```

The loud block loses its `method "NEW" of class "RexxContext": 15` line and keeps `StackFrame: 10`
and `VariableReference: 4`.

---

## The controls

Each was run and each line below is read off the run's own output. After every one, the edited files
were restored **from copies under the session scratchpad** — never `git checkout --` — and each
restore verified with `cmp` before the next step. A to D ran before any gate; E ran between the two
gate runs, and the tree hash was read back to its pre-E value afterwards.

**A — a `covered` route that does not construct.** `CONSTRUCTION_PROGRAMS`' `REXXCONTEXT` cell
changed to `.RexxContext~new`, then the row sets and the probe regenerated so that the status, the
committed column and the probe's own `o = ` line all agree and nothing upstream of the run can
notice:

```
cargo test --release -p rexx-exec --test gate_table_c   ->  exit 101
  structural failures, which are red in every mode and are not verdicts `REXX_CORPUS_GATE`
  could relax:
    gate-tables/methods/rexxcontext__instance.rex: the oracle answered 0 line(s) where this
    row's probe asks for exactly 15. The row for RexxContext (instance arm) therefore has no
    answer to compare ...
```

This is the brief's *"`OracleShape::Exactly` will catch one that does not"*, run rather than relied
on. **I did rely on it as well** — it is what makes the two committed routes' `covered` a claim
about construction rather than a string — but it is not the only evidence: the two probes' rc 0 and
byte-identical output above is direct.

**B — the narrowed guard still fires.** `("ARRAY", ".Array~new")` added to `CONSTRUCTION_PROGRAMS`,
a class whose bare `~new` constructs:

```
cargo test -p rexx-extract --test extract_docs   ->  exit 101, 8 passed 1 failed
  panicked at crates/rexx-extract/src/docs/classes.rs:574:5:
  Array has a committed construction program and a bare ~new that constructs on the oracle,
  so the program is a route to nothing the bare ~new does not already reach
```

**C — the narrowing was load-bearing, not cosmetic.** The guard's *condition* reverted to BASE's
`matches!(construction, Some(code) if code != "new")` with everything else left as committed:

```
cargo test -p rexx-extract --test extract_docs   ->  exit 101, 8 passed 1 failed
  panicked at crates/rexx-extract/src/docs/classes.rs:573:5:  RexxInfo has a committed
  construction program and a bare ~new that constructs on the oracle ...
```

Only the condition was reverted, so the text quoted is my corrected message; BASE's own text —
"a bare ~new that **does not raise** on the oracle" — is what a reader would have got, and it is
false for `RexxInfo`, whose `.RexxInfo~new` is rc 159 `97.1`. That is why the message moved with the
condition.

**D — the probe generator.** Five committed `Constructs` probes rebuilt from the script's own header
template and compared with their committed bytes: `timespan`, `array`, `string`, `circularqueue`,
`message`, all IDENTICAL. Without this the script's agreement with `method_probe_text` would have
rested on `check_probe_text` alone, which reports a first differing line rather than telling you
which side is wrong.

**E — the alternative fix is forbidden, and that was run rather than read.** The obvious way to keep
BASE's guard untouched is to give `CONSTRUCTION` a `("REXXINFO", "97.1")` row. Added, with the
program left in place:

```
cargo test -p rexx-extract --test extract_docs   ->  exit 101, 8 passed 1 failed
  panicked at crates/rexx-extract/src/docs/classes.rs:477:5:
  assertion `left == right` failed: the class set the books produce and the swept .environment
  class entries disagree; CONSTRUCTION is a measurement of the image and this is where the two
  would drift apart
   left: {... "REXXCONTEXT", "REXXQUEUE", ...}
  right: {... "REXXCONTEXT", "REXXINFO", "REXXQUEUE", ...}
```

`CONSTRUCTION`'s key set must be exactly the **class**-entry names, and `RexxInfo`'s row is an
`instance` one. So the guard was the only place the change could go. I had read that assertion and
believed it; running it is what makes the sentence above evidence.

---

## Gates

Tree hash (`git diff HEAD | sha256sum`) before the first gate of the run reported here:
`b73b458924c7cd08929157c470c6c5dbbce41fac2e23d22d607bd12bfdeb9871`.

| # | command (from `rust/`) | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

Each status was written to a file by the runner and read back unpiped. **Zero `test result: FAILED`
lines across gates 3, 4 and 5**, which is the reading that matters rather than the exit status alone,
and each of the three reports 106 `test result: ok` binaries. Gate 4's corpus harness is **331 of 331
matching**; its table C report is the 237 above and its table D report is `5c: 38 rows, 35 not yet
agree`, unchanged and Task 5's. `memcap` was present (`command -v memcap` →
`/home/moritz/.local/bin/memcap`), so gate 5 is the `memcap` form rather than the `ulimit`
substitute.

Clippy re-checked both crates this change can reach — its stderr is `Checking rexx-extract`,
`Checking rexx-exec`, `Finished` — so its green is evidence the linter looked at the new code rather
than reusing a warm result.

Tree hash after the last gate: `b73b458924c7cd08929157c470c6c5dbbce41fac2e23d22d607bd12bfdeb9871`,
unchanged. The report file is the only thing written after the runs; it lives under the git-ignored
`.superpowers/`, so it is neither in the commit nor readable by any gate.

**The five were run twice, and the first run is not what is reported above.** It was green on the
same code and row sets — the same five zeroes, the same 106 `test result: ok` binaries with no
`FAILED` line, the same 331 of 331 — at tree hash
`bc3a16709402bbda71151dae8738c0be6ac0de5a7b8091aa252407fab2d60755`. Two things happened after it:
control E, whose edited file was restored from a copy and the hash re-read back to that same value;
and a **correction to the plan's Task 3 note**, which is a tracked file and does move the hash. That
correction is the `condition('O')~stackFrames` mistake described above — the first version of the
note stated the false conclusion I had drawn from the wrong subscript. Nothing in `rust/` reads
`docs/superpowers/plans/2026-09-02-phase-5c.md` (measured: no `.rs`, `.toml`, `.txt` or `.tsv` under
`crates/` or `corpus/` names it), so the first run's greens would still have covered the committed
code — but a gate line has to be about the bytes that ship, and re-running is cheaper than arguing
that an edit was harmless.

---

## What I did not do

* **`StackFrame` and `VariableReference` are not `covered`** — 14 rows, blocked and measured above.
  Neither blocker is mine to clear: one is a `RexxContext` method body, the other is a new primitive
  object.
* **I did not make either of their probes match the oracle's refusal.** It would move no row, and
  the brief says so; the D73 decision it belongs to is Task 2's `Pointer` finding, now carrying three
  more classes.
* **I did not fix the `>x` decay**, and I did not change
  `run::tests::a_variable_reference_decays_to_the_referenced_value`, which is not wrong.
* **I did not fix the `identityHash` rendering divergence** the observable sweep re-found. It is
  Task 2's, it predates both routes, and it is measured above on three objects that predate this
  phase.
* **No method body was implemented, and no interpreter behaviour changed at all.** The diff touches
  `rexx-extract`'s tables and doc text, two generated row sets and two generated probes; nothing
  under `crates/rexx-exec/src/`.
* **`corpus/refusal-sites.tsv` was not re-derived, because nothing this change touches can
  invalidate it.** It cites `crates/rexx-exec/src/*.rs` by line number and no file under that path is
  in the diff; `the_table_holds_every_constructor_the_source_defines` is green in the gate runs
  above, which is the check rather than my reading.
* **I did not touch `UNCONSTRUCTIBLE`, and nothing was reclassified.** All four keep the
  `Status::NotCovered` their sentence grounds — that field says *which of the two claims the
  reference makes*, not what the row's final status is, and `coverage_of`'s `NotCovered` arm is
  exactly where a committed program turns into `Coverage::Covered`. D73's `unreachable` pair,
  `Buffer` and `Pointer`, is untouched, and `coverage_of`'s assertion that an `unreachable` class may
  not carry a program is unchanged.
* **I did not run the phase gate** (`REXX_PHASE_GATE=5c`): Task 6 turns it on and 5c is not in
  `CLOSED_PHASES`.
* **I did not re-measure `CONSTRUCTION`.** Its doc says nothing does, by decision, and the two
  entries I added are in `CONSTRUCTION_PROGRAMS` rather than in it.
