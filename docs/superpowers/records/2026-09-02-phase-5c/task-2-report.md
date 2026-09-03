# Phase 5c Task 2 — surface A, the constructors

**BASE:** `486b53da9`. **Committed at `a55fc4bf6`**, with the `Message~result` evidence corrected at
`925450054` after the team lead caught a wrapper trap in it (`7ca0c2430` is their own half of that
correction). Everything below was measured on this machine on 2026-09-03 against the pinned
5.3 oracle at `/home/moritz/dev/repos/ooRexx/build`, three descriptors read separately, from a fresh
empty directory per program, on `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` both.

---

## The one-paragraph version

Table C's 5c not-yet-agreeing count is **815 → 280**, 535 rows over 18 groups, and **89 of the 96
method probes now match the oracle byte for byte on stdout, stderr and exit status on both
engines** (63 did at BASE). Nineteen classes gained a `NEW` row of their own — fifteen that build
an instance and four that check their arguments and then refuse — and five were opted into `covered`
through Task 1's construction route (`CircularQueue`, `Message`, `String`, `Supplier`,
`WeakReference`). **Five of my twenty are not done**: `Class`, `Method`, `Package` and `Routine`
refuse because each builds an object this crate has no model for, and `Stem` is **backed out**,
measured, because a constructed one answers `a Stem` where the oracle answers the stem's own value.
Along the way the work found and fixed a `USE STRICT ARG` defect that predates it, and found an
oracle indefinite block that is now in `corpus/oracle-crashes.txt`.

## The brief's arithmetic, corrected

The brief says the list is "the remaining **19**". Its own enumeration holds 23 names; removing
Task 3's `RexxContext`, `StackFrame` and `VariableReference` leaves **20**, and that is what I
worked. Re-swept at BASE, the distinct `~NEW` classes named by the 30 rc-120 probes are 24 including
`Pointer`; minus `Pointer` and Task 3's three, 20.

## What surface A actually was

Sweeping all 96 probes on **both** interpreters at BASE, rather than only on the crate, splits the
30 into two kinds that need opposite work:

| | probes | the oracle | what the crate had to do |
|---|---|---|---|
| **constructs** | 15 | rc 0 | build an instance |
| **raises** | 15 | rc 163/168/159 | raise the same error, byte for byte |

The second kind is invisible from the crate-only sweep the plan used, and it is half the surface.
`bag`, `directory`, `eventsemaphore`, `identitytable`, `list`, `monitor`, `mutablebuffer`,
`mutexsemaphore`, `properties`, `queue`, `relation`, `set`, `stem`, `table`, `traceobject` are the
first; `circularqueue`, `class`, `message`, `method`, `package`, `pointer`, `rexxcontext`,
`routine`, `stackframe`, `stream`, `streamsupplier`, `string`, `supplier`, `variablereference`,
`weakreference` are the second.

## Two things the brief did not say, both measured

**A row over a `not-covered` class cannot agree, however exactly the crate matches the oracle.**
With the constructors landed and nothing else, every raising probe I own — `circularqueue`, `class`,
`message`, `method`, `package`, `routine`, `streamsupplier`, `string`, `supplier`, `weakreference` —
matched the oracle on three descriptors, and **table C's `unanswered` count had not moved at all**:
444 at BASE and 444 there. `OracleShape` marks a group whose oracle `stdout` is empty as
`unanswered`, deliberately, so that two interpreters failing identically at a probe's first
instruction do not read as a satisfied row. Measured: 815 → 473, every one of those 342 rows from
the constructing half. Moving the other half needs the class to be `covered`, which is Task 1's
`CONSTRUCTION_PROGRAMS` route — so half of "surface A" is construction-program work rather than
constructor work.

**A constructor can introduce a silent wrong answer even when the probe passes.** `Stem` and
`MutableBuffer` both render as their contents rather than as `a Stem`/`a MutableBuffer` — measured,
oracle rc 0: `say .Stem~new('x')` is `x` and `say .MutableBuffer~new('abc')` is `abc`. A plain
instance answers the default name for both. They part company at what happens next, and only running
separates them: `MutableBuffer` declares its own `STRING` and `MAKESTRING`, so this crate **refuses
loudly** and no wrong answer ships; `Stem` declares neither, so the send falls through to
`Object~STRING` and this crate **answers `a Stem` at rc 0**. So `MutableBuffer` is in and `Stem` is
out. I built `Stem~new` first and took the measurement off the build rather than off the reading.

## What was built

### `crates/rexx-exec/src/dispatch.rs`

* **`NATIVE_CLASS_METHODS` gains a `NEW` row per class.** Eleven whose C++ `newRexx` is an
  allocation followed by `completeNewObject` and nothing else share [`native_new`], the row
  `Object~NEW` already had: `Bag`, `Directory`, `EventSemaphore`, `IdentityTable`, `List`,
  `MutexSemaphore`, `Queue`, `Relation`, `Set`, `Supplier`, `Table`. The rest have argument
  handling of their own and get a function each: `native_class_new`, `native_message_new`,
  `native_executable_new` (`Method` and `Routine` share it), `native_package_new`,
  `native_string_new`, `native_mutable_buffer_new`, `native_weak_reference_new`.
* **`NATIVE_METHODS` gains the `INIT` rows those constructors send to.** `HashCollection::initRexx`,
  `QueueClass::initRexx` and `ListClass::initRexx` differ only in their default capacity, which is
  not observable, so `native_hash_init` is renamed `native_capacity_init` and serves all of them
  (`Bag`, `Directory`, `IdentityTable`, `List`, `Queue`, `Relation`, `Set`, `StringTable`, `Table`).
  `native_supplier_init` is new and is where `.Supplier~new`'s own 93.903 comes from.
* **`UNINIT` rows for `EventSemaphore` and `MutexSemaphore`**, and they are not optional. Both
  classes declare `UNINIT`, so `new_instance` registers every instance for the finalizer sweep, and
  a method with no row refuses **at collection time** rather than at a send: the first build of this
  change constructed both correctly and then printed
  `rexx-exec: method "UNINIT" of class "EventSemaphore" is not implemented (Phase 5)` at rc 120 after
  the program's last line. Found by running, not by reading.
* **`native_message_result` grows an unsent arm.** `Interp::message_outcomes` has an entry for every
  message `~start` built; a `Message~new` object has none, and that absence is the only thing that
  tells the two apart.
* **`optional_length_argument`** is factored out of `native_capacity_init` so `MutableBuffer~new`'s
  own size check is the same code rather than a second copy.
* Two citations were wrong at BASE and are corrected in passing: `HashCollection::initRexx` is at
  `classes/support/HashCollection.cpp:63`, not `:120`, which is inside `setContents`; and
  `StringTable`'s `INIT` row cited `memory/Setup.cpp:842`, where `StringTable` has no row of its own
  at all -- the method is donated by `InheritInstanceMethods(IdentityTable)` at `:881`, out of that
  class's own `AddMethod("Init", HashCollection::initRexx, 1)` at `:841`.

### `crates/rexx-exec/src/lib.rs`

`Loud::unsent_message_result`, for `Message~result` on a message whose send has not been made. It
is a refusal because there is no oracle behaviour to match: see the oracle-crashes entry below.

### `crates/rexx-exec/src/run.rs` — a defect that predates this task

`USE STRICT ARG` reported the **call** error family inside a method, where the oracle reports the
**method** family, and it did not report the omitted-position error at all. Measured at BASE, one
program per row:

| | oracle | crate at BASE |
|---|---|---|
| `use strict arg a` in a method, no argument | 93.901, rc 163 | 40.3, rc 216 |
| the same with two arguments | 93.902, rc 163 | 40.4, rc 216 |
| `use strict arg a, b` in a method, `(, 2)` | 93.903, rc 163 | **nothing, rc 0** |
| `use strict arg a` in a routine, no argument | 40.3, rc 216 | 40.3, rc 216 |
| the same with two arguments | 40.4, rc 216 | 40.4, rc 216 |
| `use strict arg a, b` in a routine, `(, 2)` | 40.5, rc 216 | **nothing, rc 0** |

The third and sixth rows are silent wrong answers at BASE, in both contexts. The C++ has one
`inMethod()` switch at each of its three sites (`instructions/UseInstruction.cpp:103`, `:292`,
`:305`) and this crate had none; `exec_use_arg` now reads `Entry::Method` once and passes it down.
All six rows and two adjacent successes are byte-identical on both engines afterwards.

**This is in scope because a row needed it**: `circularqueue__instance` reaches
`CoreClasses.orx:1728`'s `use strict arg size`, and the whole 47-row group turned on the first table
row. I fixed all three sites rather than the one, because they are one `if strictChecking` block and
one switch, and correcting half of an instruction is how a neighbour stays wrong.

### The row sets and the probes

`CONSTRUCTION_PROGRAMS` gains five entries, each the book's own constructor shape and each measured
byte-identical on both engines:

| class | program | rows | book |
|---|---|---|---|
| `CircularQueue` | `.CircularQueue~new(5)` | 47 | `collclasses.xml:3356`, "The required *size* argument, a non-negative whole number" |
| `Message` | `.Message~new(.Object~new, 'STRING')` | 20 | `fundclasses.xml:1166`, whose *target* and *messagename* are the two required arguments |
| `String` | `.String~new('abc')` | 118 | `fundclasses.xml:5152`, "initialized with the characters in *stringvalue*" |
| `Supplier` | `.Supplier~new(.Array~new, .Array~new)` | 7 | `utilityclasses.xml:9956`, "must be an array of objects" |
| `WeakReference` | `.WeakReference~new(.Object~new)` | 1 | `utilityclasses.xml:12740`, "a reference to *object*" |

`corpus/docs/class-set.txt` and `class-methods.txt` were regenerated with
`cargo run -p rexx-extract --bin rexx-extract-docs -- --oodocs ../oodocs --interpreter ../interpreter
--out corpus/docs`, and five instance probes with a scratch script whose output was **first
validated against three already-committed probes of each shape** (`timespan` for `Constructs`,
`array` for a bare `~new` that constructs, `pointer` for `Raises`) and then run over all 63
instance-arm classes: it rewrote exactly the five whose row set changed and reproduced the other
58 byte for byte. `check_probe_text` re-derives all 96 from the row set on every gate run, so a
script that had drifted would have reddened rather than shipped.

## Table C, before and after

`cargo test --release -p rexx-exec --test gate_table_c`, report read from stderr:

| | BASE `486b53da9` | after |
|---|---|---|
| 5c rows | 1347 | 1347 |
| **5c not yet `agree`** | **815** | **280** |
| `agree` (whole table) | 673 | 1208 |
| `diverge-both` | 371 | 29 |
| `unanswered` | 444 | 251 |
| loud | 724 | 87 |
| 5a / 5b not yet `agree` | 0 / 0 | 0 / 0 |

535 rows moved, over 18 groups, and **no group moved the other way**: `Bag` 28, `CircularQueue` 47,
`Directory` 31, `EventSemaphore` 5, `IdentityTable` 24, `List` 38, `Message` 20, `Monitor` 4,
`MutableBuffer` 51, `MutexSemaphore` 3, `Properties` 39, `Queue` 43, `Relation` 28, `Set` 24,
`String` 118, `Supplier` 7, `Table` 24, `WeakReference` 1.

## The 96-probe sweep

Each probe run against the oracle and against both engines, three descriptors compared separately,
each in a fresh empty directory:

| | BASE | after |
|---|---|---|
| identical to the oracle on all three descriptors, both engines | 63 | **89** |
| whose group reads `agree` in table C | 62 | 80 |

**The two rows are different questions and the gap between them is the point.** Nine probes are
byte-identical and their groups still read `unanswered`: `Class`, `Method`, `Package`, `Routine`,
`Alarm`, `File`, `RexxInfo`, `StreamSupplier` and `Ticker` are `not-covered`, so the oracle's own
`stdout` is empty and `OracleShape` refuses to call that a satisfied row. `RexxInfo` is the one that
was already in that position at BASE, which is what makes the 63/62 split there.

The seven that are not:

| probe | why | owner |
|---|---|---|
| `pointer__instance` | deliberately not built -- see below | D73 / the plan |
| `rexxcontext__instance`, `stackframe__instance`, `variablereference__instance` | no `~new`; a Rexx-level route instead | Task 3 |
| `stream__instance` | `stream_init` is a Phase 7 native | 5d |
| `stem__instance` | a constructed `Stem` renders as `a Stem` where the oracle renders its value | measured below |
| `traceobject__instance` | `TraceObject` subclasses `StringTable`, and `receiver_kind`'s `StringTable` arm tests class identity rather than descent | unowned |

## `Pointer` still refuses, and this is what I ran

`.Pointer~new` was left alone: no row was added for it, and `corpus/refusal-sites.tsv` and
`class-set.txt` are unchanged for it. Run from a fresh directory, both engines, after the change:

```
crate   rc 120  stdout empty  stderr rexx-exec: method "NEW" of class "Pointer" is not implemented (Phase 5)
oracle  rc 163  stdout empty  stderr ... Error 93.967:  NEW method is not supported for the Pointer class.
```

`pointer__instance` is one of the seven probes above and its 5 rows are still `unanswered`.

**A finding for whoever owns D73, since it is one row's worth of work.** `PointerClass::newRexx`
(`classes/PointerClass.cpp:139`) is `reportException(Error_Unsupported_new_method, getId())` and
nothing else — it constructs nothing. A `("Pointer", "NEW", …)` row raising 93.967 would therefore
not be a constructor by any reading, and would move those 5 rows to `agree`. I did not add it
because the brief excludes `Pointer` from my list in as many words; the decision is not mine to
take.

## The oracle blocks on `Message~result` before its send

Found while building `Message~new`. `m = .Message~new(.Object~new, 'STRING')` then `say m~result`
**never returns**: measured under the standard wrapper with a kill deadline, rc 137 twice (15 s and
8 s), nothing on either descriptor. `MessageClass::result` waits for completion
(`classes/MessageClass.cpp:279`) and `MessageClass::newRexx` never dispatches, so nothing can
complete it.

**The first version of this entry proved "parked rather than spinning" with a number that did not
measure the interpreter, and the team lead caught it.** It quoted `/usr/bin/time -v timeout -s KILL
8 rexx …` reporting 0.00 s user and 0.00 s system over 8.00 s: GNU time's `wait4` sees `timeout`,
which burns nothing while it sleeps. Re-measured off `/proc/<pid>/stat` with the binary launched
directly and no `timeout` between the shell and it, `CLK_TCK` 100:

| program | state | utime at 1 s | utime at 7 s |
|---|---|---|---|
| `.Message~new(…)` then `~result` | `S` | 0 | 0 |
| `do forever; nop; end` | `R` | 99 | 700 |

The spinner is the control, and it is the half that makes the zeros mean anything — without it "0
ticks" is a reading, not evidence. **The wrapper was then run against that same spinner** and
reported `User time 0.00`, `System time 0.00`, `Percent of CPU this job got: 0%` over 6.00 s
elapsed, for a program `/proc` shows burning a whole core: the trap reproduces, and a park and a
spin are indistinguishable through it. `.superpowers/sdd/2026-08-27-phase-5b/watchdog-brief.md`
records the same trap under "Two measurement traps", so this is the second time it has bitten in
this phase's work.

**The conclusion was unchanged by the correction**, which is worth saying plainly: a condition-variable
wait is what `MessageClass::result` predicts, the crate's refusal was already right, and only the
sentence of evidence was wrong. That is the failure mode to watch for — a true claim resting on a
number that measures something else, which no amount of re-reading the sentence would have caught.

**A flag for whoever owns `oracle-crashes.txt` entry 7.** Its `GUARD ... WHEN` block carries "a
third bounded run under `/usr/bin/time -v` reports 0.00 s user and 0.00 s system over 5.00 s
elapsed", which is the same instrument and may have the same defect. I did **not** re-measure it:
the file's own header forbids running its programs to confirm an entry still holds, and that rule
outranks my curiosity. Its safety claim is unaffected either way — the rc 137 with empty descriptors
is direct evidence that it does not terminate; it is only the "parked, not spinning"
characterisation that rests on the suspect figure.

The neighbours are all clean at rc 0 under the same deadline and bound it to the unsent state:
`~completed` and `~hasError` answer `0` and `0`, `~messageName` and `~target` answer, `m~send` then
`m~result` answers, and `.Object~new~start('STRING')~result` answers. It is now the last entry of
`corpus/oracle-crashes.txt` with that account, and `Loud::unsent_message_result` is this crate's
refusal for the shape.

**The prose correction in that file is not mine.** By the time I went to fix it, the entry had
already been rewritten in the working tree by someone else -- state `S` at three and at seven
seconds, 0 utime and stime ticks between, one thread, `wchan` `futex_do_wait`, plus the wrapper
warning. That is an independent measurement of the same shape and it agrees with mine, so I left
their text standing and **added** the one thing it did not carry: the spinner control and the
wrapper run against it. I backed their version up to the session scratchpad and verified the copy
with `cmp` before editing, since nothing else in this task's tree was theirs to lose. They then
committed the file as `7ca0c2430`, carrying my addition with it; `Loud::unsent_message_result`'s
doc still held the old figure and is corrected separately at `925450054`, with all five gates
re-run because a doc-comment-only change is what shipped a red gate at `486b53da9`. **No upstream ticket** — the wait is what `~result` is specified to do, and
whether an interpreter should detect an unsatisfiable one is the design question entry 7 already
puts.

## No silent divergence was introduced, and that was run rather than reasoned

Every class whose instance this change can build — `Bag`, `CircularQueue`, `Directory`,
`EventSemaphore`, `IdentityTable`, `List`, `Message`, `Monitor`, `MutableBuffer`, `MutexSemaphore`,
`Properties`, `Queue`, `Relation`, `Set`, `String`, `Supplier`, `Table`, `WeakReference` — asked
thirteen `Object`-level observables each, one program per pair, against the oracle and against both
engines, in a fresh directory. Run on the tree as committed:

```
SAME   196    identical on all three descriptors, both engines
LOUD    20    the crate refuses at rc 120 where the oracle answers
SILENT  18    all of them one method, and it is not one this task touched -- below
```

One program per row rather than one loop over all of them, and that mattered: a loop dies at the
first loud refusal, because `signal on syntax` does not trap one. The first attempt was a loop; the
crate printed nine lines, refused, and the comparison reported 186 differences, every one of them a
line the run never reached.

The loud ones are `~request('ARRAY')` on `Bag`, `Directory`, `IdentityTable`, `List`, `Queue`,
`Relation`, `Set`, `Table`, `Properties`, `MutableBuffer`, `CircularQueue` and `String` -- this
crate has no `MAKEARRAY` for any receiver -- `~copy` on a `Message` and on a `String`, and `~string`,
the bare rendering and `~request('STRING')` on `MutableBuffer` and `CircularQueue`, whose own
`makeString` this crate does not implement. Both `~copy`s and `String`'s `~request` are refusals a
plain `'abc'` already gets: measured, `'abc'~copy` and `'abc'~request('ARRAY')` are `rexx-exec:
method "COPY" of class "Object"` and `... "MAKEARRAY" of class "String"` at rc 120 with no
constructor involved. `EventSemaphore`, `MutexSemaphore`, `Monitor`, `Message`, `WeakReference` and
`Supplier` answer `The NIL object` to both `~request`s on both sides.

Separate runs cover what each constructor's arguments would have carried:
`.Message~new(…)~send`, `.WeakReference~new(…)~value`, `.Supplier~new(…)~available`,
`.MutableBuffer~new('abc')~length` and `.Directory~new~put(…)` are each loud, and
`.Directory~new['X']`, `~at('X')` and `~zork` each answer `The NIL object` as the oracle does.

**The eighteen `SILENT` rows are one pre-existing divergence and none of them is a constructor's.**
`~identityHash~length` is `16` on the oracle and `3` here for every instance, `10` for a string.
Measured on objects that predate this change: `.Object~new`, `'abc'`, `.Array~new` and
`.StringTable~new` all answer `16` against `3`, `10`, `3`, `3`. So it is `native_identity_hash`'s
rendering, which this task did not touch, and it is reported here rather than fixed. It is also why
the first version of this sweep saw nothing: it asked `~identityHash > 0`, which is `1` on both
sides — a check that could not fail on the axis that was wrong.

## The one answer that is right for a reason worth asserting

`.Directory~new` is a plain instance with no hash body, so `~at` reads nothing and answers `.nil`.
That is the oracle's answer for an empty directory and a wrong answer for any other — it is correct
only while every write beside it refuses. `a_new_directory_reads_nil_for_every_index_and_refuses_every_write`
asserts the pair, so a later change that let a write through without giving the object a body
reddens rather than shipping a directory that silently forgets.

## Tests added

| test | what would redden it |
|---|---|
| `dispatch::tests::a_primitive_constructor_answers_an_instance_or_the_oracle_s_own_refusal` | a constructor that stopped building, or one that skipped its argument checks; the two halves fail on opposite mistakes |
| `dispatch::tests::a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state` | a constructor that fabricated the state its arguments carried, and the unsent-`~result` refusal |
| `dispatch::tests::a_new_directory_reads_nil_for_every_index_and_refuses_every_write` | a write that silently succeeded on a body-less directory |
| `dispatch::tests::a_semaphore_instance_runs_its_finalizer_without_refusing` | the `UNINIT` rows going away |
| `run::tests::use_strict_arg_reports_method_errors_in_a_method_and_call_errors_outside` | one error family for both contexts, and a check applied to plain `USE ARG` as well |

**Every expectation in them is a value the oracle produced, not one the crate agreed with itself
about.** The three constructor tests' answers come from the class sweep above; the `USE STRICT ARG`
rows and its two adjacent successes were run on the oracle one program each; and
`say d['X'] d~at('X') d~zork` and `o = .EventSemaphore~new; say 'built'` were each run on the oracle
in the exact shape the test asserts, rather than inferred from single-value runs beside them.

**Every one of the five is red at BASE, and that is measured rather than mutated.** `.Bag~new`,
`.Message~new(…)` and `.Directory~new` are all rc 120 there, so the three constructor tests fail
outright; three of the `USE STRICT ARG` test's six rows are the red BASE measurements in the table
above; and the `UNINIT` test is what the first build of this change actually printed, before the two
rows were added. No mutation was needed to show any of them can fail.

## Generated artifacts re-derived

* `corpus/refusal-sites.tsv` — **derived from the test's own source scan, never by shifting line
  numbers by hand.** `the_table_holds_every_constructor_the_source_defines` prints its derived rows
  and its committed rows as two lists when it fails; both passes below parsed the *derived* list and
  wrote those `surface` and `definition` values into the table, asserting that the committed value
  being replaced was exactly what the test reported on the other side. Twice, because a second
  constructor arrived after the first pass.
  The first: nineteen `run.rs` rows moved by 25 lines and `not_enough_method_arguments` and
  `too_many_method_arguments` gained the `body` surface, since they are now constructed in `run.rs`
  as well as in `dispatch.rs`. The second: `reply_inside_construct` moved and
  `unsent_message_result` is a new `send`-surface row with verdict `not-run`. The test was re-run
  after `cargo fmt` and after the comment moves, and reported no further drift.
* `corpus/docs/class-set.txt` and `class-methods.txt` — regenerated from the extractor; the five
  classes' `status`, `reason` and `construction` columns change and nothing else.
* Five instance probes.

## Gates

Tree hash (`git diff HEAD | sha256sum`) before the first gate:
`17e97773a6829e6378809fc18142c882cc6c666c43bed375cff8fd65efe2f322`.

| # | command (from `rust/`) | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

Each status read unpiped from its own file. **Zero `test result: FAILED` lines across gates 3, 4 and
5**, which is the reading that matters rather than the exit status alone, and each of the three
reports 106 `test result: ok` binaries. Gate 4's corpus harness is **331 of 331 matching**, as at
BASE, and its table C report is the 280 above; table D is unchanged at 35, which is Task 5's.

Tree hash after the last gate: `17e97773a6829e6378809fc18142c882cc6c666c43bed375cff8fd65efe2f322`,
unchanged.

**The five were run a second time for `925450054`**, the evidence correction, and reported the same:
fmt 0, clippy 0, and 0 `test result: FAILED` with 106 `ok` binaries on each of gates 3, 4 and 5,
corpus 331 of 331. That run's `git diff HEAD` hash moved mid-run and it was worth checking why: the
team lead committed `7ca0c2430` while it was in flight, so `HEAD` moved under it while the
working-tree bytes the gates were compiling did not change at all -- `oracle-crashes.txt` was
identical before and after, it merely stopped being a modification. A hash that moves is not by
itself an invalidated run, but it has to be explained rather than waved through. The report file is the one thing written after the runs, and no gate reads it -- it lives
under the git-ignored `.superpowers/`, so it is not in the commit either; Task 6 copies the directory
into `docs/superpowers/records/`.

**Three earlier gate runs were started and stopped before they finished**, and none is reported
above, because reviewing the diff while each ran found something to change. The first: three doc
blocks my insertions had orphaned from the rows they describe — `Class~ACTIVATE`, `Directory~[]` and
`RexxContext~PACKAGE` each had a comment that now sat above one of my rows. The second: five doc
comments claiming that some method "has no row in `NATIVE_METHODS` and refuses loudly", which is a
statement about where the implemented boundary sits and is the kind `rust/CLAUDE.md` records as the
one that actually rots; each now states the property instead. The third: the one of those that cites
a test named a test which did not assert the sentence beside it, so the bare rendering of a
`MutableBuffer` is now a row of that test.

## What I did not do

* **`Class`, `Method`, `Package` and `Routine` construct nothing** — 94 rows. Each checks exactly
  what the C++ checks before its allocation and then refuses loudly, so `.Class~new`,
  `.Method~new`, `.Package~new` and `.Routine~new` are byte-identical to the oracle and
  `.Class~new('k')` is `rexx-exec: method "NEW" of class "Class" is not implemented (Phase 5)`.
  What each needs: `RexxClass::newRexx` **clones** the receiver class (`ClassClass.cpp:1789`),
  which is a different factory from the `~subclass` this crate has; `Method` and `Routine` need
  `LanguageParser::createMethod`/`createRoutine` over the source argument; `Package` needs
  `loadRequires`. All four have live natives on their instance behaviour (`SCOPE`, `ANNOTATION`,
  `NAME`, `PUBLICCLASSES`), so a fabricated instance would hand those functions an object their
  side tables do not know — which is why I did not build one to move the rows.
* **`Stem~new` is backed out** — 27 rows — with the measurement above. Making it safe means either
  a `Stem` string value (a method body) or a refusal inside `Interp::to_text`, which is the hot
  path every `SAY` takes.
* **`TraceObject`** — 2 rows. `::class "TraceObject" subclass StringTable`'s class-side `new`
  forwards to `StringTable~new`, which builds a `Body::Native` whose class is `TraceObject`;
  `receiver_kind`'s `StringTable` arm tests `native.class() == model.string_table` and so falls
  through to the "one of the interpreter's own objects" refusal. Fixing it means
  `Primitive::StringTable` carrying the object's own class rather than reading the behaviour off
  `model.string_table`, because otherwise `hasMethod` would answer `StringTable`'s names for a
  `TraceObject`. Two rows did not justify that. It is the same axis as the `Directory` limitation
  above -- a primitive body this crate models for exactly one class and not for its descendants --
  and whoever gives `Directory~new` a hash body should look at both.
* **`StreamSupplier`** — 8 rows. It needs a `Stream`, which is 5d's.
* **`Pointer`** — 5 rows, deliberately, above.
* **No method body was implemented.** Every method that would read what a constructor's arguments
  carried refuses loudly, and the report says which ones per class.
* **I did not run the phase gate** (`REXX_PHASE_GATE=5c`): Task 6 turns it on and 5c is not in
  `CLOSED_PHASES`.
* **`.Supplier~new(1, 2)` refuses where the oracle answers.** `arrayArgument` converts a non-array
  with `requestArray`, and this crate has no `MAKEARRAY` for any receiver, so the refusal is the
  existing `unconverted_array_argument` one that `~UNKNOWN` already gives. Loud, and not a regression.
* **`.Message~new` with a third argument refuses**, rather than checking the `"AI"` argument-style
  option. The committed construction program passes two.
