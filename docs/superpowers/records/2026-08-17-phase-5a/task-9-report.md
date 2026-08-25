# Task 9 report: the Object and Class reflection protocol

`~class`, `~id`, `~superClass`, `~superClasses`, `~metaClass`, `~isA`, `~isSubclassOf`, `~method`,
`~package` and `~identityHash` answer on both engines. `Array~makeString` and `Package~name` answer
beside them, because the table C wiring row cannot reach `agree` without the first and the brief's own
`.Array~package~name` row cannot be asked without the second.

Base commit `4729e5d3a`. Every differential below was run with the standard wrapper, one process each,
from a fresh empty directory, absolute paths, three descriptors read separately, `REXX_ENGINE` set to
`ir` and to `tree-walker` in turn.

## The five gate commands

Run from `rust/`, each status read unpiped from the command itself. Re-run in full at the end of fix
round 1, and these are that run's statuses.

| command | status |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --release --workspace` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 0 |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 |

The corpus is **143 of 143** under both gated commands, from 135 at the base commit: this task adds
eight programs. `memcap` was present (`/home/moritz/.local/bin/memcap`), so the fifth command is the
one the constraints name rather than the `ulimit -v` substitute.

## Each method's differential

### On a `::CLASS`-declared class and on the primitive classes

`corpus/lang/class_reflection.rex`, rc 0, stdout and stderr byte-identical on both engines. It asks
the whole protocol of `K`, `T` and `T2`, which the file declares, and of `.Object`, `.Class`,
`.Method`, `.Package`, `.Routine`, `.Supplier` and `.WeakReference`; and it asks `~class` and `~isA`
of a string, a number, `.nil`, an array and a package object, which are the receiver kinds a program
can put on the left of a send in this phase.

The rows that separate methods a build could confuse:

| row | oracle, and ours |
|---|---|
| `.T2~metaClass` / `.T2~class` | `The MC class` / `The Class class` -- Task 8's parting row, now readable from Rexx |
| `.T~superClass` / `.T~superClasses` | `The B class` / `The B class The MX class` -- the first entry, not the last |
| `.Array~isA(.Array)` / `.Array~isSubclassOf(.Array)` | `0` / `1` -- `~isA` goes through `~class` |
| `.Object~superClass` / `.Object~superClasses` | `The NIL object` / an array holding nothing |

`corpus/lang/class_package.rex`, rc 0, identical on both engines: `.Array~package` renders
`The REXX Package` and its `~name` is `REXX`, `.String~package~name` and `.Class~package~name` are
`REXX` too, and `.K~package` renders `a Package` with `~name` the file's own path -- the same string
`PARSE SOURCE`'s third word carries, which the last row of that program asks beside it.

`corpus/lang/class_reflection_argument_ladder.rex`, rc 168, identical on both engines: 88.901 naming
`method name` and `class`, 88.909 naming `method name`, 88.914 naming `class`, 93.902 from the
declared arity, and the two rows that answer. Row 11 is the third outcome and the reason a number is
in the list: a number has a string value, so `~method` reaches the lookup and raises 97.1 for its own
digits rather than 88.909.

`~identityHash` **has no differential row and cannot have one**: the oracle's answer is derived from
the object's address (measured, `-140442982827441` for `.Array` on one run) and this crate answers the
handle, which deviation 4 licenses. Measured, our answer for `.Array` is `8589934604`.
`dispatch.rs`'s `identity_hash_answers_a_number_that_follows_the_handle` is the whole instrument and
its doc says so.

### On the primitive classes, through table C

`corpus/gate-tables/classes/*.rex` is the derived per-class probe, and it asks exactly this protocol.
See the wiring count below.

## The `~method` scope behaviour, both directions, with the frame line

`corpus/lang/class_method_own_dictionary.rex`, rc 159, identical on both engines. The answering rows
first, then the raising ones, then the untrapped send whose traceback is compared as bytes:

```
1 answered a Method        .Array~method('APPEND')      Array's own
2 answered a Method        .Array~method('append')      upcased before the lookup
3 answered a Method        .Array~method('MAKESTRING')  Array's own
4 answered a Method        .Class~method('ID')          Class's own instance method
5 answered a Method        .Object~method('HASMETHOD')  Object's own
6 answered a Method        .K~method('OWN')             K's own ::METHOD
7 raised 97.1              .Array~method('STRING')      Object's, donated to Array's behaviour
8 raised 97.1              .Array~method('ID')          Class's, reached on the class side
9 raised 97.1              .K~method('SIDE')            K's own ::METHOD ... CLASS
10 raised 97.1             .K~method('NOSUCHNAME')      nothing defines it
hasmethod-string 1
hasmethod-id 1
hasmethod-side 1
```

The three `hasMethod` rows are what pin the rule to "this class's own dictionary": every name rows 7
to 9 raise for, `~hasMethod` answers `1` for.

The frame line, from that program's own untrapped send, byte-identical on both engines:

```
       *-* Compiled method "METHOD" with scope "Class".
    64 *-* say .Array~method('STRING')
Error 97 running .../class_method_own_dictionary.rex line 64:  Object method not found.
Error 97.1:  Object "The Array class" does not understand message "STRING".
```

`corpus/lang/class_method_class_side_raises.rex` is the other direction untrapped, rc 159, identical
on both engines:

```
       *-* Compiled method "METHOD" with scope "Class".
    14 *-* say .K~method('M')
Error 97 running .../class_method_class_side_raises.rex line 14:  Object method not found.
Error 97.1:  Object "The K class" does not understand message "M".
```

**Checked against `blame_native_method`'s rule rather than appended to a list.** That doc says the
line is owed wherever the oracle reached the failing native method's body by a message send, whatever
put that send there. Both refusals above are raised inside `RexxClass::method`'s body, which the oracle
entered from `~method` written in the source, so both are owed the line and both carry it. The
argument-ladder refusals and `makeString`'s are the same shape and carry it too, measured. No refusal
this task adds fails to fit the rule.

## The classes the `~method` programs cover

The brief says this named list is the whole extent of the protection against a build that flattens
every scope onto one class, so it is stated rather than left to be read off the file:

* **`Array`** -- `APPEND`, `MAKESTRING` and `TOSTRING` in its own dictionary, `STRING` in its
  flattened behaviour only, `ID` reachable on its class side only;
* **`Class`** -- `ID` in its own instance dictionary;
* **`Object`** -- `HASMETHOD` in its own instance dictionary;
* **`K`**, declared by `class_method_own_dictionary.rex` -- `OWN` from a `::METHOD`, `SIDE` from a
  `::METHOD ... CLASS`, `A` and `A=` from an `::ATTRIBUTE`, and `B` and `B=` from an `::ATTRIBUTE ...
  CLASS`. The two attribute pairs are rows 11 to 14 and the first version of this list stopped at row
  10, which is the protection stopping early rather than a transcription slip: `A=` is the only name
  any of these programs asks for that is not a plain symbol, and `B`/`B=` are the accessor shape of
  the class-side half that `SIDE` is the `::METHOD` shape of. Measured: `A` and `A=` answer, `B` and
  `B=` raise 97.1;
* **`K`**, declared by `class_method_class_side_raises.rex` -- `M` from a `::METHOD ... CLASS`, which
  is that program's untrapped refusal.

**The list is no longer the sole record of what the rows cover.** Naming the classes in the program's
own header comment was the third instance of an enumeration that its own file can outgrow, so that
comment now says what puts a name in the set and the rows are where membership is written down. This
list stays, because the brief asks the *task* for it and a report is where a measurement belongs.

There is no table C row for the scope question: `gate_table_c.rs`'s own module doc records that the
plan builds no scope row class and that these programs are the instrument instead.

## The table C wiring count

Measured by `cargo test --release -p rexx-exec --test gate_table_c`, whose oracle column is produced
by launching the oracle on the run that reports it.

| | base `4729e5d3a` | this task |
|---|---|---|
| `agree` | 0 | **14** |
| `diverge-stdout` | 0 | 11 |
| `diverge-both` | 63 | 38 |

**14 is the headline number.** The rows that now `agree`: `Class`, `Method`, `Object`, `Package`,
`Routine`, `Buffer`, `EventSemaphore`, `MutableBuffer`, `MutexSemaphore`, `Pointer`, `RexxContext`,
`StackFrame`, `Supplier`, `WeakReference`.

### The classes that remain non-`agree`, and why

**Eleven registered classes read `diverge-stdout`, and the criterion "the wiring rows for the classes
this crate registers read `agree`" is therefore not met.** Every one of the eleven differs on exactly
one line of the probe -- `superclasses` -- and the missing entries are exactly `CoreClasses.orx`'s own
`~inherit` calls. Diffed one by one, both engines:

| class | the entry the oracle has and we do not | `CoreClasses.orx` |
|---|---|---|
| `String` | `The Comparable class` | `:93` |
| `Array` | `The OrderedCollection class` | `:97` |
| `List` | `The OrderedCollection class` | `:98` |
| `IdentityTable` | `The MapCollection class` | `:103` |
| `Table` | `The MapCollection class` | `:104` |
| `StringTable` | `The MapCollection class` | `:105` |
| `Directory` | `The MapCollection class` | `:106` |
| `Relation` | `The MapCollection class` | `:107` |
| `Set` | `The MapCollection class`, `The SetCollection class` | `:108`, `:114` |
| `Bag` | `The MapCollection class`, `The SetCollection class` | `:109`, `:115` |
| `Message` | `The MessageNotification class`, `The AlarmNotification class` | `:118`, `:119` |

This is not a gap this task could close: the entries come from running the prologue, and
`native_classes.rs`'s module doc already assigns exactly this -- "Full post-prologue correctness for
the classes R8 names -- their `~superClasses`, their complete flattened method set -- is explicitly
**Task 13's**". The wiring rows for those eleven move when Task 13 lands, not before.

**Thirty-eight rows read `diverge-both`, and every one fails at `.NAME` rather than at this
protocol** -- `rexx-exec: environment symbol ".X" is not implemented (Phase 5)`. Three of them are the
deferrals the brief names, `Queue`, `Stem` and `VariableReference`, which `native_classes.rs` defers
for `Setup.cpp`'s `RemoveMethod`/`HideMethod` and which **Task 21** lifts. The other thirty-five are
classes `CoreClasses.orx` and `StreamClasses.orx` define, so they too are Task 13's: `Collection`,
`MapCollection`, `OrderedCollection`, `SetCollection`, `CircularQueue`, `Properties`, `Alarm`,
`AlarmNotification`, `Comparable`, `Comparator`, `CaselessComparator`, `ColumnComparator`,
`CaselessColumnComparator`, `DescendingComparator`, `CaselessDescendingComparator`,
`InvertingComparator`, `NumericComparator`, `DateTime`, `File`, `MessageNotification`, `Monitor`,
`Orderable`, `RexxInfo`, `RexxQueue`, `Singleton`, `StreamSupplier`, `Ticker`, `TimeSpan`,
`TraceObject`, `Validate`, `ArgUtil`, `InputStream`, `OutputStream`, `InputOutputStream`, `Stream`.

## Both controls, run

### Control 1: dropping a class from the registry reddens its table C wiring row

Added a `Deferral` for `WeakReference` to `native_classes.rs`'s `DEFERRALS`, rebuilt, and ran
`REXX_CORPUS_GATE=1 REXX_PHASE_GATE=5a cargo test --release -p rexx-exec --test gate_table_c`.

* the wiring verdicts moved from 14 `agree` / 11 `diverge-stdout` / 38 `diverge-both` to
  **13 / 11 / 39**;
* the `WeakReference` row itself moved from `agree` to `diverge-both`, reading
  `crate rc=120 err="rexx-exec: environment symbol \".WEAKREFERENCE\" is not implemented (Phase 5)"`
  against `oracle rc=0`.

**What the control could not see, stated because the exit status is the obvious thing to read and it
is not evidence here.** Measured: that command exits 101 at HEAD *without* the mutation as well,
reporting `gated by this run: 117 row(s)` -- 49 of the 63 wiring rows are non-`agree` before the
mutation, and the concept, edge and method families contribute the rest, so `REXX_PHASE_GATE=5a` is
non-zero either way until Tasks 13 and 21 land. The verdict change and the `agree` count are what
fire; the status is not.

Restored from a copy taken before the edit, and re-ran the table: 14 / 11 / 38 again, and
`git diff --stat` on `native_classes.rs` is empty.

### Control 2: answering `~method` from the flattened behaviour reddens this task's own programs

Two mutations, because the first one only reddens half of the pair and that is the point of there
being two programs. Both run as `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`.

| mutation in `native_method` | corpus | programs that redden, named by the report |
|---|---|---|
| `has_own_instance_method` -> `has_method` (the flattened **instance** behaviour) | 142 of 143 | `class_method_own_dictionary.rex` |
| `has_own_instance_method` -> `class_has_method` (the **class** behaviour) | 140 of 143 | `class_method_own_dictionary.rex`, `class_method_class_side_raises.rex`, `array_make_string.rex` |

**Both mutations were re-run in this fix round and the reddening programs read off the harness's own
report rather than reasoned about**, which is what makes the two-program necessity a measurement: the
first mutation leaves `class_method_class_side_raises.rex` green, so that program's own untrapped
refusal is reachable only by the second. The class-behaviour mutation reddens a third program now,
because `array_make_string.rex` gained `.Array~method('MAKESTRING')` and `.Array~method('TOSTRING')`
rows and `MAKESTRING` is not in `.Array`'s class behaviour -- so the `~method` rule is witnessed from
outside the two programs the brief names as well as inside them.

Under the first mutation row 7 of `class_method_own_dictionary.rex` answers `a Method` where the
oracle raises, and the program ends rc 0 with empty stderr instead of rc 159 with the frame line --
so the mutation is caught on all three descriptors. `class_method_class_side_raises.rex` stays green
there, because `M` is a class method and is absent from `K`'s instance behaviour flattened or not;
only the second mutation reaches it.

**Both mutations were caught by these programs and by nothing else.** Under each, the report named the
reddening programs and no other of the 143 moved, which answers "can fail is not adds coverage": no
pre-existing program covers `~method`'s dictionary at all.

Restored `dispatch.rs` from a copy and re-ran: 143 of 143.

## What else this task changed, and what catches a regression in it

* **`~superClasses` answers a `Body::Array`, so an array is a receiver and a value.** `to_text`,
  `text_len` and `to_number` gained arms; `try_text` answers `None` for one, which is a third cause of
  its `None` and is now named in its doc. Before this the value model panicked on a `Body::Array` in
  any of them, because nothing built one.
* **A name `.Array`'s behaviour does not answer is 97.1, not a loud refusal**, which is the same answer
  a `String` receiver already gets for a name `CoreClasses.orx:93` donates and this crate has not.
  `corpus/lang/array_unknown_method.rex` pins it, and pins the target text as well: the oracle names
  the receiver by `stringValue()`, which for an array is `an Array` and **not** the string value a
  string context asks for. `array_make_string.rex`'s `string-context` row is the other half of that
  pair.
* **An array in an operator that needs a number or a truth value is loud** --
  `eval.rs`'s `operator_operand_gap` gained the arm. This is a refusal where the oracle answers, so it
  needs its own instrument and cannot have a corpus row: `eval.rs`'s
  `an_operator_sent_to_an_object_is_loud` and `an_object_in_a_do_headers_numeric_position_is_loud`
  gained the array rows, with the oracle's own answer in each row's comment (97.1 for `+` and `&`,
  `0` for `=` and `==`, which is `Object`'s identity comparison and 5c's). **One of those comments was
  wrong before it was measured**: it claimed the oracle answers `1` for
  `say (.Object~superClasses = '')`, reasoning from identity comparison rather than running it. The
  oracle answers `0`, measured, and the corrected comment says why the row matters -- an empty array's
  items joined are the empty string, so a build converting through the string value answers `1`.
* **A package object is a receiver too**, and only because `.Package`'s instance behaviour here is
  `Setup.cpp`'s whole set: neither `CoreClasses.orx` nor `StreamClasses.orx` names `.Package`,
  checked by grep over both files. A `Body::Native` of any other class keeps its loud refusal, because
  `Directory`'s and `StringTable`'s behaviour *is* extended by the prologue and 97.1 there would be a
  wrong answer rather than a refusal.
* **`Interp::package_name` answers `Option`**, so a package handle this crate did not build refuses
  loudly instead of answering an empty name. Unreachable from a program; the shape is the crate's rule
  for an internal inconsistency rather than a case that fires.
* **Ownership moved in this commit**, as the constraints require: `coverage.rs`'s `EXPECTED_SUBSET_5A`
  gained the eight programs, and `collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS` gained
  `class_method_class_side_raises.rex` -- the one of the eight that allocates nothing, because its
  refusal upcases the looked-up name into a local buffer and takes the target from the registry.
  `crates/rexx-parse/tests/sourceline_oracle/` gained the eight expectation files, regenerated with
  the driver that test's module comment carries.

## The performance sitting

**The pin is not stale, checked before it was trusted.** `bench-baselines/pinned/rexx-run-15a1ffa98`
is present and its sha256 is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching `PINNED.md`.
`git log --oneline 15a1ffa98..4729e5d3a -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml`,
run at the base commit before any of this task's own landed, lists 34 commits, and every one of them is
named in this plan's `progress.md` -- checked by grepping the ledger for each hash in turn rather than
by reading the list. At `HEAD` the same command lists 36, the two extra being this task's own code
commits, which the ledger names once this report is filed.

Instrument `instructions:u`. `pinned>head` is `head / pinned`, so above 1 means head retires more.
The table below is the sitting labelled `18626fdb1` in `bench-baselines/phase-5a-arms.tsv`, run after
that commit landed. The file also holds a sitting labelled `4e9a0369f`, the first of this task's two
commits, whose `strings` and `alloc4c` rows are the ones the refinement moved; the four attribution
builds below are **not** in it, because none of them is a commit on this branch and a row measured
against a build nobody can check out is not a comparable reading.

**Every figure in this table is the row's `value_median` column**, and every one is from the
`18626fdb1` sitting the paragraph above names:

| axis | tw small | tw large | ir small | ir large |
|---|---|---|---|---|
| `alloc4c` | 1.000795 | 1.000775 | 1.001203 | 1.001156 |
| `arith` | 1.001006 | 1.001036 | 1.001284 | 1.001294 |
| `compound` | 1.000000 | 1.000000 | 1.000000 | 1.000000 |
| `emptyloop` | 1.000000 | 1.000000 | 1.000000 | 1.000000 |
| `strings` | 1.005750 | 1.005749 | 1.009685 | 1.009685 |
| `varlookup` | 1.000000 | 1.000000 | 1.000000 | 1.000000 |

**Two of those cells were wrong until the column was named, and not in the way the other one was.**
`alloc4c`'s two `large` cells read 1.000774 and 1.001157, which are medians -- but of the pre-commit
`9-refined` scratch sitting, not of the `18626fdb1` sitting this table's own caption names. So the
first error was reaching into the wrong column of the right row and this one was reading the right
column of the wrong run. Both are invisible in a bare grid and both are shut out by a header that says
where the figures come from.

Every axis is under the guard's 1% threshold, `strings`/`ir` at +0.9685% the closest. **It is the
change and not the layout, and the mechanism is narrowed to one line rather than argued.**

*Why it is not layout.* `compound`, `emptyloop` and `varlookup` read 1.000000 to six decimal places on
both arms. A layout artifact moves axes that execute no changed code; these do not move at all.

*The narrowing, by mutation.* Four sittings, each a build with one piece of this task's code removed,
each five rounds interleaved against the same pin:

`value_median` again, `ir` arm, `small` size; a dash is an axis that build did not run.

| build | `strings` ir | `alloc4c` ir |
|---|---|---|
| this task as committed | 1.009685 | 1.001203 |
| `Redirect` carrying the items in a `Vec` (the first shape) | 1.010616 | 1.002706 |
| `to_text`/`text_len` reverted to their pre-task shape | 1.008940 | **1.000000** |
| `operator_operand_gap`'s array arm removed | 1.009685 | -- |
| `try_text`'s `Body::Array(_) => None` arm removed | **1.000745** | -- |
| that arm kept and `try_text` marked `#[inline]` | 1.009685 | -- |
| that arm kept and `try_text` marked `#[inline(always)]` | 1.009685 | -- |

So: `alloc4c`'s +0.12% is `to_text` and `text_len` gaining their array arms, and **`strings`'s +0.89%
is the single `Body::Array(_) => None` line in `try_text`** -- the hottest function on that axis, which
`pos`, `substr`, `changestr`, `||` and `length` all reach three million times in
`bench-programs/strings.rex`. `operator_operand_gap`'s arm costs nothing measurable, and the cost is
not an inlining decision: `#[inline(always)]` on `try_text` reproduces 1.009685 to six decimals. The
first version of that control used `#[inline]`, which is a hint the compiler may decline, so it could
not have distinguished "inlining is not the cause" from "the hint was ignored"; `#[inline(always)]` is
the test and it was re-run.

**The per-pass counts, because the ratio hides what the delta is.** `strings` `per_pass`
`instructions:u`, the row's `value_median` column, rounded to two decimals -- and the `pinned` row is
each sitting's own reading of the pinned binary, which agrees across all of them at this precision:

| build | tw | ir | delta, both arms |
|---|---|---|---|
| pinned `15a1ffa98` | 9044.52 | 5369.52 | -- |
| `4e9a0369f` | 9101.52 | 5426.52 | **+57** |
| `18626fdb1` | 9096.52 | 5421.52 | **+52** |
| `try_text`'s arm removed | 9048.52 | 5373.52 | **+4** |

The delta is the same number of instructions on both engines, which is what says the added work is in
the value model that both share. So **`try_text`'s one arm is 48 instructions per pass and everything
else this task added is 4.** The ratio differs between the arms only because the denominators do --
the compiled engine does 5369 instructions per pass where the tree-walker does 9044 -- so the `ir` arm
is the binding one for the guard's threshold for that reason and not because the compiled engine pays
more.

**And the cost is not the arm's body running.** `bench-programs/strings.rex` builds no array, so
`Body::Array(_) => None` never executes on any of the three million passes. What the line changes is
the dispatch it sits in: `Body::Array` leaves the group that shared `try_text`'s `other =>
unreachable!()` arm and becomes a target of its own, and the match over `&object.body` compiles
differently as a result. The earlier report said "real work, not noise" and left the mechanism
unstated, which is an assertion where the measurement above is available.

**The stated consequence, because it is a fact about the next task and not a footnote about this one.**
The `ir` arm's 1% ceiling is 53.70 instructions per pass and this task stands at 52.00. **There are 1.7
instructions per pass of headroom** -- 0.03 points of ratio -- so the next task that adds an arm to
`to_text`, `text_len` or `try_text` crosses the guard on `strings` at the compiled engine. The three
attribution builds above price such an arm: `to_text` and `text_len` together cost 4 per pass and
`try_text` alone cost 48, so it is the `try_text` arm specifically that no one can afford next, and a
task needing one has to buy it back somewhere rather than absorb it.

*What was given back rather than reported.* The first shape of `Redirect` carried the array's items in
a `Vec`, which put drop glue on the path every heap string's rendering takes and read `strings` ir
**1.010616** -- over the threshold. Making `Redirect` `Copy` and looking the items up again in the
array arm alone brought it to 1.009685 and `alloc4c` from 1.002706 to 1.001203. That variant's own
figures are in `Redirect`'s doc comment, because they are evidence for the shape as it now stands.

*What the sitting could not see.* `cycles:u` moved between -4.5% and +3.5% across the axes with
`instructions:u` flat to a millionth on three of them, which is the noise this plan's constraints
already record; no `cycles:u` row here is a result on its own. And the sitting measures six axes: a
per-pass cost on a path none of them executes -- a send, a class declaration, a `::METHOD` body --
would read 1.000000 here and is not measured by anything in this task.

## Fix round 1

Six items, and one defect the round found that no item names.

### 1. The sixth guessed citation, and the two others without a line number

`PackageClass::getName` **does not exist.** The method is `PackageClass::getProgramName`
(`classes/PackageClass.hpp:147`, whose whole body is `{ return programName; }`), bound as `Name` by
`memory/Setup.cpp:1189`. Both lines printed before they were written down.

The reviewer's observation is the part worth keeping: it was the only C++ citation in the diff without
a line number and the only one that was wrong. **A citation with no line number is one nobody
printed**, so the round swept the diff for that shape rather than for that name --
`git diff 4729e5d3a..HEAD` piped through a `::`-name and `file.cpp:line` extractor -- and found two
more, both now carrying lines and both checked by printing them:

* `RexxObject::classObject` -> `classes/ObjectClass.cpp:1814`, whose whole body is
  `behaviour->getOwningClass()`. That body is now quoted in `native_class`'s doc, because it is the
  reason `~class` reads the owning class and `~metaClass` reads a field of the class.
* `runtime/MethodArguments.hpp` bare -> `:136` for the overload taking a position and `:161` for the
  one taking a name. `:161`'s `OREF_NULL` arm is
  `reportException(Error_Invalid_argument_noarg, name)`, which is the 88.901 this task's
  `missing_named_argument` renders, so that constructor's doc now cites it too.

One bare file reference stays bare and is not a citation: the claim that the prologue leaves `.Package`
alone is a claim about whole files. It is now recorded **as wide as its pattern** -- case-insensitive
`\.package\b` over `CoreClasses.orx` and `StreamClasses.orx` matches nothing in either, where the same
pattern for `.array` matches in both, which is the control that says the pattern can match.

### 2. A method object answers nothing, and its identity, both recorded

Measured, standard wrapper, fresh empty directory, three descriptors, **both engines identical to each
other**:

| probe | oracle | crate, `ir` and `tree-walker` |
|---|---|---|
| `.Array~method('APPEND')~class` | rc 0, `The Method class` | rc 120, `rexx-exec: a message send to one of the interpreter's own objects is not implemented (Phase 5)` |
| `.Array~method('APPEND')~class~id` | rc 0, `Method` | rc 120, the same refusal |
| `.Array~method('APPEND')~isA(.Method)` | rc 0, `1` | rc 120, the same refusal |
| `.Array~method('APPEND')~scope` | rc 0, `The Array class` | rc 120, the same refusal |
| `(.Array~method('APPEND') == .Array~method('APPEND'))` | rc 0, `1` | rc 120, `the operator `==` applied to one of the interpreter's own objects` |

Both are now recorded in `corpus/lang/class_method_own_dictionary.rex`, beside the rows that build the
object, which is where this corpus records what it cannot yet witness.

**Why the first four are one gap and not four.** `receiver_kind` admits a package object among the
interpreter's own objects and not a method object, because `~package~name` is the only thing this
task's probes ask of one. The first three rows are *this task's own protocol* reaching a receiver it
does not reach; `~scope` and the rest are `.Method`'s own protocol, which table C files under **5c**.

**Which task the identity half belongs to: 5c, with the method rows -- and it is not deviation 4's.**
The oracle's `1` is `RexxClass::method` handing back the `MethodClass` the instance dictionary already
holds (`classes/ClassClass.cpp:991`), not an identity model answering. Making it `1` here needs the
dictionary to hold method objects rather than the `MethodId` `rexx-classes`'s `MethodDict` holds, which
is the same thing `.Method`'s own protocol needs. Filing it under deviation 4 would put it where
nothing would ever fix it. Nothing silently disagrees today: the `==` is a loud refusal, not a wrong
answer.

### 3. Prose and enumerations

* `dispatch.rs`'s receiver-kind enumeration is **deleted**, not extended. The doc now says one row per
  receiver kind `receiver_kind` admits that is not a class object, and points at that function as where
  membership is decided.
* `class_method_own_dictionary.rex`'s covered-class enumeration is deleted the same way: the header now
  says what puts a name in the set and each raising row's own comment says which dictionary holds its
  name.
* `value.rs`'s "the only caller" was false -- there are callers at `:601`, `:729` and `:1047`, and
  `heap_to_number`'s does not go through `Redirect` at all. The comment now covers both routes.
* `corpus/phase-5a.txt`'s one-word wrap artifact is reflowed.

### 4. Two report claims corrected

Both are in their own sections above rather than here: the coverage list now runs to row 14 and says
why the attribute rows are the ones that were missing, and the sitting's "real work, not noise" is
replaced by the per-pass measurement and the actual mechanism.

### 5. Two controls sharpened

* `#[inline(always)]`, in the sitting section: it reproduces 1.009685 to six decimals, so the claim
  survives the test that `#[inline]` could not make.
* Control 2's per-mutation program names, in the controls section, read off the harness's report on a
  re-run.

### 6. `TOSTRING` is implemented beside `MAKESTRING`

They are the same C++ function at the same arity -- `memory/Setup.cpp:733` and `:734` both bind
`ArrayClass::toString` with `2` -- so it is one more `NATIVE_METHODS` row pointing at the same
implementation rather than a second function, and the comment says a second function is where the two
could come to disagree.

Measured before it was written: `~toString` agrees with `~makeString` on the default, `L` with and
without a separator, `C`, a lower-case option, an option omitted in place, and the empty list; it
raises the same 93.915 and the same `C`-with-separator 93.902; and `.Array~method('TOSTRING')` answers
`a Method`, so it is in `.Array`'s own dictionary exactly as `MAKESTRING` is. The traceback frame names
**the message that was sent** -- `Compiled method "TOSTRING" with scope "Array".` -- which
`array_make_string_refusals.rex`'s untrapped send is now spelled `~toString` to pin, so the frame's own
name is read rather than assumed to be the implementation's. `array_make_string.rex` carries a
`toString` twin for each `makeString` row.

### Which side of the sitting predicate this round falls on

**The code side of it is not reached, and that is measured rather than argued.** The round's only
code change is a `NATIVE_METHODS` row for `TOSTRING` pointing at the function `MAKESTRING` already
names, plus comments; nothing it touches is on `to_text`, `text_len` or `try_text`'s path, and
`bench-programs/strings.rex` neither declares a class nor sends a message, so it never builds the
object model that table is read into.

A six-axis, five-round sitting was run anyway, because the round changes `src/` and because the figure
it would confirm has 1.7 instructions per pass of headroom. It is labelled `5fd2002cf` in
`bench-baselines/phase-5a-arms.tsv` and was re-run against that commit after it landed.

**Every figure below is the row's `value_median` column**, `pinned>head` `across_builds`
`instructions:u`:

| axis | tw small | ir small | tw large | ir large |
|---|---|---|---|---|
| `alloc4c` | 1.000795 | 1.001203 | 1.000774 | 1.001157 |
| `arith` | 1.001006 | 1.001284 | 1.001036 | 1.001294 |
| `compound` | 1.000000 | 1.000000 | 1.000000 | 1.000000 |
| `emptyloop` | 1.000000 | 1.000000 | 1.000000 | 1.000000 |
| `strings` | 1.005750 | 1.009685 | 1.005749 | 1.009685 |
| `varlookup` | 1.000000 | 1.000000 | 1.000000 | 1.000000 |

And `strings` `per_pass` `instructions:u`, same column: head 9096.520911 tw and 5421.518032 ir against
the pin's 9044.520903 and 5369.518137 -- the same +52 on both engines as `18626fdb1`. So this round
adds no measurable work.

**It reproduces `18626fdb1` to six decimals on every axis but two cells, and those two say where the
noise floor is rather than that something moved.** `alloc4c`'s `tw large` and `ir large` read
1.000774 and 1.001157 here against 1.000775 and 1.001156 there -- a disagreement in the sixth decimal,
on the axis whose own delta is a thousandth. Every other cell is identical, `strings` included, and the
per-pass delta is +52 on both engines in both sittings. Saying "to six decimals on every axis" without
that exception would have been a claim two cells contradict.

**Naming the column in the header is the fix for more than this cell.** The run this replaces read
`1.001204` for `alloc4c`/`ir`/small, which is that row's `value_max` where its `value_median` is
`1.001203`: the data was right and the prose reached into the wrong column. That is the same confusion
that produced a false "min equal to max" claim two tasks ago, and a slash-separated run of bare figures
with no header is exactly where it can live unnoticed. Every table in this report that quotes one
figure per cell now says which column it is.

### The defect no item names: a corpus program printed an absolute path

`corpus/lang/class_package.rex` printed this file's own absolute path twice, on the
`k-package-name` and `parse-source` rows. `corpus/README.md`'s own rule forbids exactly that --
`PARSE SOURCE`'s third word "would put the filesystem in this program's output and break the corpus's
determinism rule", and `lang/parse_sources.rex` and `lang/source_arg.rex` both project it away. The
differential passed because both interpreters are handed the same path, which is why no gate caught it.

Projected away the same way, and the replacement pins **more** than printing it did: the program now
prints whether `.K~package~name` equals `PARSE SOURCE`'s third word (`1`), whether it equals `REXX`
(`0`), and whether `.Array~package~name` equals that word (`0`). A build answering `REXX` for every
class fails the second row, and a build answering the path for every class fails the third; printing
the path asserted neither.

## What I could not close

* **The eleven `diverge-stdout` wiring rows**, above. The criterion as the brief words it is not met,
  and the gap has the owner `native_classes.rs` already names: Task 13.
* **`~identityHash` has no differential row at all**, by deviation 4's own construction. The in-crate
  test is the whole instrument and cannot compare against the oracle.
* **`Object~string` does not answer**, and an array is where that first shows: `a~string` is
  `an Array` on the oracle and a loud refusal here. It is not in this task's list and `~string` is a
  table C method row, which the table files under 5c.
* **A method object answers no message, and `~method` mints a fresh one per send.** Recorded with the
  measurements in fix round 1's item 2 above and in
  `corpus/lang/class_method_own_dictionary.rex`; the identity half is 5c's.
* **The `strings` axis is +0.9685% of `instructions:u`, which is 52 instructions per pass and leaves
  1.7 of headroom**, narrowed to `try_text`'s array arm above. Nothing cheaper was found:
  `#[inline(always)]` recovered none of it, and the arm cannot be dropped, because without it a
  `Body::Array` reaching `try_text` falls into that match's `unreachable!` and aborts the process.
  Folding it into a `_ => None` fallback would recover it and would make `try_text`'s documented
  meaning of `None` false for a `Body::Instance`, so it was not taken. **The consequence is the next
  task's, stated in the sitting section rather than left here.**
* **`~instanceMethod`, `~instanceMethods`, `~methods`, `~subClasses` and `~request` do not answer.**
  None is in this task's list. `~request` reads `owningClass` on the oracle side
  (`ObjectClass.cpp:1916`), so it is the second reader of the field Task 8 split and will exercise it
  when its own task lands.
