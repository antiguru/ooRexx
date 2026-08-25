# Task 4 report: gate table D, and the shared gate-table harness

**Status: complete.** Two commits on `plan/rust-rewrite`, base `094fbbdd8`:

| commit | what |
|---|---|
| `042dbeda8` | the probe corpus -- one `.rex` per row, under `rust/corpus/gate-tables/directives/`, plus both READMEs |
| `db312da3e` | the shared harness (`rust/crates/rexx-exec/tests/gate_tables/`) and table D (`rust/crates/rexx-exec/tests/gate_table_d.rs`) |

**The headline number, with its predicate.** **10 rows of gate table D whose
committed owning phase is `5a` do not have verdict `agree`**, out of 36 rows
owned by `5a`, out of 79 rows in the table. The predicate for "not `agree`" is:
the crate's outcome and the oracle's outcome differ on at least one of exit
status, `stdout` or `stderr`, with `stderr` compared byte for byte. The
predicate for "owned by 5a" is: `owning_phase` in `gate_table_d.rs` answers
`"5a"` for that row. This task **establishes** the number rather than holding
it; every task after this one reports against it.

**No sitting is owed.** This task lands nothing in `src/` of `rexx-exec`,
`rexx-core`, `rexx-classes` or `rexx-lib`. `git diff --stat -- rust/crates/*/src/`
is empty across both commits; what landed is `tests/`, `corpus/` and two
READMEs, so the release binary the axes measure is byte-identical and a sitting
would measure noise.

---

## 1. The harness's shape

`rust/crates/rexx-exec/tests/gate_tables/mod.rs` -- the part Task 5 reuses:

* **`Verdict` and `verdict`** -- the five cells and the function that picks one.
* **`Descriptors`** -- the three-boolean comparison, kept uncollapsed until
  `verdict` collapses it.
* **`run_on_both_engines`** -- both crate engines in process through
  `Invocation::with_engine`, with an unconditional `assert!` that they agree on
  all three descriptors before a verdict exists.
* **`compare_raw`** -- `support::oracle::descriptor_diffs_with(..., StderrComparison::Raw)`
  on every row.
* **`is_loud`** and **`refused_construct`** -- the crate-side-only column, and
  the construct name for grouping.
* **`Structural`** and **`assert_no_structural_failures`** -- the channel that
  is red in every mode.
* **`CORPUS_GATE_ENV`, `PHASE_GATE_ENV`, `CLOSED_PHASES`, `verdict_is_gated`** --
  the gate modes.
* **`Report`** and **`emit_uncaptured`** -- the report, built as one payload and
  written through a `sh -c 'cat >&2'` child with inherited stderr, the mechanism
  `corpus.rs` established.

`gate_tables/orx.rs` is table D's own column and is described in section 5.

`rust/crates/rexx-exec/tests/gate_table_d.rs` is the table: row loading from
`rust/corpus/docs/directive-options.txt`, the row-to-probe derivation, the
committed `owning_phase` assignment, the run loop, and the two `#[test]`s.

### `StderrComparison::Raw` on every row, and why it is not a preference

`compare_raw` is the only comparison path the table has; there is no branch
that could give a row the normalised one. The default `Normalized` is
DEVIATION 0 and collapses the run of spaces between a trace line's own marker
and its content. `stderr` equality is an **input** to `verdict`, so a
normalised comparison would map a `diverge-stderr` row onto `agree` -- a green
row making a byte-for-byte claim nothing checked byte for byte.

**Rows that run under `TRACE`:** exactly one, `::OPTIONS TRACE subkeyword`,
whose probe is `say 'main'` under `::options trace i`. The oracle's `stderr`
for it is `     1 *-* say 'main'` followed by a `>L>` and a `>>>` line -- the
indent run normalisation would collapse. It is **not** a corpus program and is
not in any `phase-*.txt`, so it adds no entry to `corpus.rs`'s
`RAW_STDERR_COMPARISON`; that list names corpus programs, and this table
compares raw unconditionally rather than by opting rows in.

---

## 2. The verdict function, and the argument that its cells are exclusive and exhaustive

```rust
pub fn verdict(differs: Descriptors) -> Verdict {
    match (differs.status, differs.stdout, differs.stderr) {
        (false, false, false) => Verdict::Agree,
        (true,  false, false) => Verdict::DivergeStatus,
        (false, true,  false) => Verdict::DivergeStdout,
        (false, false, true)  => Verdict::DivergeStderr,
        (true,  true,  false) => Verdict::DivergeBoth,
        (true,  false, true)  => Verdict::DivergeBoth,
        (false, true,  true)  => Verdict::DivergeBoth,
        (true,  true,  true)  => Verdict::DivergeBoth,
    }
}
```

**The argument is that there is no argument to make in prose.** The domain is
`(bool, bool, bool)`, a cube of eight points, and every point is written out as
its own arm.

* **Exhaustive** -- a missing point is a non-exhaustive `match`, which is
  `E0004` and does not compile.
* **Mutually exclusive** -- a duplicated point is an unreachable pattern, which
  `unreachable_patterns` reports and `-D warnings` makes an error; and the
  function returns one value, so two cells cannot both hold a point in any case.
* **No precedence question** -- there is no `_` arm and no guard, so no arm's
  meaning depends on which arms precede it. Reordering the eight arms changes
  nothing.

**What the compiler cannot state, and the test that does.**
`the_verdict_function_partitions_the_descriptor_cube` walks the cube from
outside the function, groups the eight points by the cell each lands in, and
asserts (a) the groups' sizes sum to eight, (b) every one of `Verdict::all()`
has a non-empty preimage, (c) no other value appears. The failure it can
actually catch is **a cell with an empty preimage** -- a cell that cannot fire,
which is decoration.

**Recorded as run.** Changing `(false, false, true)` from `DivergeStderr` to
`DivergeBoth` makes that test fail, naming the cell:

```
the `diverge-stderr` cell has an empty preimage over the descriptor cube,
so no comparison can ever produce it
```

### `loud` is a column, never a verdict

`is_loud` reads the crate's outcome alone: exit status `NOT_IMPLEMENTED_EXIT`
**and** a `rexx-exec: ` line on `stderr`. Both are required, because a program
may `EXIT 120` of its own accord. Nothing about the oracle enters it, so it can
co-fire with any of the five cells, and it is reported as its own column and
counted separately.

**What it distinguishes, and what it distinguishes today.** `loud` separates a
gap from a wrong answer: a row that is not `agree` and not loud is this crate
answering confidently and differently. On this commit it distinguishes nothing
that the verdict does not, because **`loud` and non-`agree` are the same 48
rows** -- every table-D divergence today is a refusal, and none is a wrong
answer. That is a fact about the crate's current state, not about the column:
mutation C below drives the loud count from 48 to 2 while leaving all 48 rows
non-`agree`, which is exactly the "wrong answer rather than a gap" case the
column exists to name.

### A correction to the plan's own illustration

The brief says *"a row that is `diverge-status` and loud is exactly the
`::ANNOTATE` row this plan closes in Task 20"*, and the plan says the same of
table C's wiring rows (*"`.array~id` is rc 120 today, so most wiring rows read
`diverge-status` and `loud`"*).

**Measured, the five `::ANNOTATE` target rows are `diverge-both` and loud, not
`diverge-status`.** A loud refusal moves all three descriptors at once: it
happens at install time, so the program's own `say` never runs and `stdout` goes
from `main\n` to empty; it writes `rexx-exec: ...` where the oracle wrote
nothing; and the status goes from 0 to 120. `diverge-status` requires the crate
to produce the oracle's exact bytes on **both** byte channels and differ only in
status, which a refusal that prints its own message cannot do.

This matters for Task 5, which inherits the same expectation for a much larger
row set: table C's wiring rows will read `diverge-both`, not `diverge-status`.
Nothing about the mechanism changes -- the cell is still one of five, still
gated the same way -- but a task that asserts the expected cell by name would be
asserting the wrong one.

---

## 3. The row-to-probe mapping

**Derived, not tabulated.** `Row::probe_path` builds the path from the row's
own three fields:

```
gate-tables/directives/{directive without ::, lowercased}__{keyword lowercased}__{position, non-alphanumerics collapsed to _}.rex
```

so `::OPTIONS CONDITION value-of(ALL)` is
`gate-tables/directives/options__condition__value_of_all.rex`. There is no
mapping table, so there is no second place for a row's identity to live and no
place for a typo to hide; the only way a row can lose its probe is for the file
to be absent.

**Both directions, and both structural.** Before anything runs, the table
compares the set of derived paths against the set of `.rex` files on disk in
that directory:

* a row whose probe is missing -> `Structural`, *"a row has no probe program"*;
* a `.rex` no row names -> `Structural`, *"a probe program no row names, so
  nothing runs it"*.

**A row without a probe is never a skip.** The `Structural` list is asserted
unconditionally, before any gated assertion, so the table reddens on the missing
program in report mode as well as under the gate.

### What each probe does

Every probe is `say 'main'` plus the directive clause carrying the row's keyword
at the row's position, with whatever surrounding directives that keyword needs
(a `::CLASS` for a `::METHOD`/`::ATTRIBUTE`/`::CONSTANT`; a declared target for
an `::ANNOTATE`). Three probes read the option's effect back with a builtin
instead of saying `main`: `::OPTIONS DIGITS` says `digits()`, `::OPTIONS FUZZ`
says `fuzz()`, and the three `FORM` rows say `form()` -- the only options in
this table whose effect a Phase-5-reachable builtin can observe.

The `say` is not decoration: without it, an oracle that silently did nothing
would produce the same empty `stdout` as an oracle that ran the program.

**Two pairs of rows have textually identical probes**, in separate files, and
that is unavoidable rather than an oversight: `::OPTIONS ALL subkeyword` and
`::OPTIONS SYNTAX value-of(ALL)` are both exercised by `::options all syntax`,
because `ALL` cannot appear without one of its two values. The same holds for
each of the six condition options against `SYNTAX value-of(that option)`.

### Two probe choices worth naming

* **`::METHOD EXTERNAL` and `::ATTRIBUTE EXTERNAL` name `LIBRARY REXX`**, not a
  real shared library: `'LIBRARY REXX zzz_no_entry'`. That is the form the
  plan's Task 22 moves into 5a, and the oracle's answer for it is rc 166 with
  `Error 90.998: Unable to find external method "zzz_no_entry"` -- byte for byte
  the eager-bind shape Task 22's own "Done when" names. **`::ROUTINE EXTERNAL`
  keeps a real shared-library name** (`'LIBRARY zzznolib zzzr'`, rc 158,
  `98.903 Unable to load library`), because the plan says that form stays Phase
  7's.
* **`::REQUIRES` names things that do not exist** -- `zzznolib` for `LIBRARY`,
  `'zzznofile.rex'` for `NAMESPACE`. A probe naming a real library would depend
  on which native libraries this host has built, which is machine state no file
  in the checkout records.

---

## 4. The owning-phase column

`owning_phase` is a committed `match`, one arm per directive keyword, and it is
what decides whether a verdict mismatch becomes an exit status. **Nothing checks
that a row is filed under the right phase**; it is visible in a diff and read by
a human, which is the standing the global constraints set for the row sets
themselves. A row no arm covers is `Structural` -- there is no default.

| phase | rows | not `agree` | authority |
|---|---|---|---|
| `5a` | 36 | **10** | this phase's own subject: `::ANNOTATE`, `::ATTRIBUTE`, `::CLASS`, `::METHOD` |
| `5b` | 2 | 0 | `DELEGATE` on `::METHOD` and `::ATTRIBUTE` -- the plan puts it in 5b because `dire.xml` defines it as `expose` plus `forward to()` |
| `5c` | 38 | 35 | the plan's handover list: `::OPTIONS`, `::RESOURCE`, `::REQUIRES`'s two options, `::ROUTINE`'s option surface |
| `7` | 1 | 1 | `::ROUTINE EXTERNAL` naming a real shared library, which the plan says stays Phase 7's |
| `deferred-parse-error-rendering` | 2 | 2 | the two `cross-reference` rows -- see below |

`5a`'s ten:

```
::ANNOTATE ATTRIBUTE   ::ANNOTATE CLASS     ::ANNOTATE CONSTANT
::ANNOTATE METHOD      ::ANNOTATE ROUTINE   ::ATTRIBUTE EXTERNAL
::CLASS INHERIT        ::CLASS METACLASS    ::CLASS MIXINCLASS
::METHOD EXTERNAL
```

### The two rows nobody owns, and why they are not filed under a phase

The row set's two `cross-reference` rows are `::CLASS CLASS` and
`::RESOURCE LIBRARY`: the documentation section names the keyword and the
directive's own parser has no arm for it, so **both interpreters refuse them**.
Measured, they refuse with the *same* error number and sub-number -- 25.901 and
25.926 -- and differ only in rendering: the oracle exits 231 with a clause echo
and two `Error` lines, this crate exits 120 with one `rexx-exec: 25.901: ...`
line.

That divergence is `docs/superpowers/plans/phase-4-exclusions.txt`'s 2026-08-20
note, which states in terms that it is decided, that the number/sub/line **are**
gated in both directions by `rexx-parse/tests/errors.rs` against `rexxc`, and
that the rendering gap *"is not Phase 5 work"* and *"nobody owns it"*.

So these two rows can never read `agree` and no task in this plan can make them.
They are filed under `deferred-parse-error-rendering`, which is deliberately not
spelled like a phase so it can never match `REXX_PHASE_GATE` or sit in
`CLOSED_PHASES`. They are reported and never gated. **Had they been filed under
`5a`, Task 24 could not have closed the phase**, and the reason would have been
invisible in the count.

### An unresolved conflict I am flagging rather than resolving

The crate's own loud message for `::METHOD EXTERNAL` is
`is not implemented (Phase 7)`, while the plan's Task 22 -- a 5a task -- owns
`::METHOD ... EXTERNAL 'LIBRARY REXX name'`. I filed the row as `5a` on the
plan's authority, and Task 22's brief already asks it to *"say which of the
three `EXTERNAL` forms this task moves and which it does not"*. The crate's
owner string is one of the things that has to move with it.

---

## 5. The four `.orx` shapes, as derived by the scan

`gate_tables/orx.rs` lexes both files rather than citing them: nesting `/* */`
carried across lines, `--` to end of line, both string quotes, and the `;` that
ends a directive clause. It holds **no line number, class name or shape
location**; every answer below is what the scan produced on the run that
reported it, and the test makes a shape reaching **zero** occurrences a
`Structural` failure.

| shape | derivation | what the scan reached |
|---|---|---|
| a quoted class name, `MIXINCLASS`, and `INHERIT` naming more than one class | a `::CLASS` whose first operand token is quoted, whose tokens include unquoted `MIXINCLASS` and unquoted `INHERIT`, and whose `INHERIT` tail holds more than one token | `StreamClasses.orx:115` |
| a quoted `SUBCLASS` target | a `::CLASS` where the token after unquoted `SUBCLASS` is quoted | `StreamClasses.orx:371` |
| an install-time `::CONSTANT` sending a private class method of its own class | a `::CONSTANT` whose enclosing `::CLASS` is *C*, whose value tokens contain `.C~M`, where *M* is the name of a `::METHOD` under the same `::CLASS` in the same file carrying unquoted `PRIVATE` and unquoted `CLASS` | `StreamClasses.orx:548`, `StreamClasses.orx:549` |
| a directive carrying its body on the same physical line | a directive clause that ended at a `;` with non-whitespace after it on the same physical line | `CoreClasses.orx:151` and eleven more, `:152`-`:162` |

Read at the file this session, before the scan was written, and each is what the
brief describes:

* `StreamClasses.orx:115` -- `::CLASS 'InputOutputStream' public MIXINCLASS Object INHERIT InputStream OutputStream`
* `StreamClasses.orx:371` -- `::CLASS 'StreamSupplier' public subclass 'Supplier'   /* stream supplier class */`
* `StreamClasses.orx:546` -- `::method getSeparator private class external "LIBRARY REXX file_separator"`,
  `:548` -- `::constant separator (.File~getSeparator)`, under `::CLASS "File"` at `:506`
* `CoreClasses.orx:151` -- `::method string_cls_alnum; use strict arg; return xrange("alnum")`

**The scan reached the third shape at `:548` and `:549`, not at `:546`.** The
brief cites the range `:546`-`:549`, which is the whole construct: the two
`private class` methods and the two `::CONSTANT`s that send them. The shape as
defined is a property of a `::CONSTANT` clause, so the hits are the two
constants, and the two methods are what makes them hits.

**Recorded as run.** Narrowing shape 1's `INHERIT` tail requirement from "more
than one" to "more than two" makes the scan reach nothing and turns it into a
structural failure naming the shape:

```
orx shape: a quoted class name, MIXINCLASS, and INHERIT naming more than one class:
the scan reached no occurrence of this shape in either bootstrap file
```

### The usage column, and the over-count that fixing it removed

The per-row column answers "does either bootstrap file use this directive with
this keyword". Its predicate: for a `subkeyword` row, an unquoted token equal to
the keyword at an **option position**; for a `value-of(X)` row, an unquoted token
equal to the keyword immediately after an unquoted `X`. An option position is
one that is neither the name the directive defines nor an operand consumed by a
preceding keyword.

**Skipping operands is what makes the column true, and I measured that rather
than assuming it.** With the operand rule off, a sweep of both files for "a hit
where the matched token is actually some keyword's operand" found **exactly
one**: `::class "Singleton" mixinclass class public` at `CoreClasses.orx:3974`,
where `class` is `MIXINCLASS`'s operand. It landed on the `::CLASS CLASS` row --
the one row of the table whose own evidence says `::CLASS` has no `CLASS` arm at
all -- so the column would have reported a use the parser cannot accept. With
the rule on, that row reports no use, which is right.

`operands_consumed` is written from `DirectiveParser.cpp`'s per-directive
functions, and `::CLASS`'s `INHERIT` consumes to the end of the clause because
that arm's own comment says *"all tokens after the keyword will be consumed by
the INHERIT keyword"* and its loop runs to end-of-clause.

**What the bootstrap actually uses**, from the run:

| row | `CoreClasses.orx` | `StreamClasses.orx` |
|---|---|---|
| `::METHOD CLASS` | 56 clauses | 17 clauses |
| `::METHOD EXTERNAL` | 5 | 61 |
| `::METHOD PRIVATE` | 9 | 29 |
| `::METHOD ABSTRACT` | 19 | 6 |
| `::METHOD UNGUARDED` | 10 | 2 |
| `::CLASS PUBLIC` | 27 | 7 |
| `::CLASS MIXINCLASS` | 17 | 4 |
| `::CLASS SUBCLASS` | 4 | 1 |
| `::CLASS INHERIT` | 2 | 2 |
| `::ATTRIBUTE GET` | 9 | 3 |
| `::ATTRIBUTE UNGUARDED` | 6 | 0 |
| `::ATTRIBUTE CLASS` | 5 | 0 |
| `::ATTRIBUTE SET` | 1 | 2 |

Every other row of the table has no use in either file -- including all of
`::OPTIONS`, `::REQUIRES`, `::RESOURCE`, `::ROUTINE` and `::ANNOTATE`, and
including `::CLASS METACLASS`, `::CLASS PRIVATE`, `::CLASS ABSTRACT` and every
`::METHOD`/`::ATTRIBUTE` access option other than the five above.

---

## 6. Gate modes, and the phase-gating mechanism

**I introduced `REXX_PHASE_GATE` and `CLOSED_PHASES`. Neither existed anywhere
in `rust/` before this commit, so no earlier task's
`REXX_PHASE_GATE=5a ... cargo test` checked anything; from this commit it has a
subject and it fires.**

* `CORPUS_GATE_ENV` = `REXX_CORPUS_GATE`, read the same way `corpus.rs` reads it.
* `PHASE_GATE_ENV` = `REXX_PHASE_GATE`; unset or empty means no verdict gating.
* `CLOSED_PHASES` is a committed constant, **empty** as of this commit. The
  phase's own closing commit adds `5a` to it.
* `verdict_is_gated(phase)` = corpus gate on **and** (phase is the closing phase
  **or** phase is in `CLOSED_PHASES`).

**Measured, all four combinations:**

| invocation | table D's exit | why |
|---|---|---|
| `cargo test --release --test gate_table_d` | 0 | report mode |
| `REXX_CORPUS_GATE=1 cargo test --release --test gate_table_d` | 0 | no closing phase named, `CLOSED_PHASES` empty, so no verdict is gated |
| `REXX_PHASE_GATE=5a cargo test --release --test gate_table_d` | 0 | a closing phase without the corpus gate gates nothing |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` | **101** | 10 rows owned by `5a` are not `agree` |

**The fourth is red by design and will stay red until every 5a row agrees.**
The five gate commands are unaffected, because none of them sets
`REXX_PHASE_GATE`; that is why the corpus stays 106 of 106 and both corpus gate
commands exit zero. The plan's own Task 24 "Done when" -- *"every 5a row in both
tables reads `agree` under `REXX_PHASE_GATE=5a`"* -- is only a check at all if
this command is red before then. **Tasks 5 through 23 should read the count off
the report, not off the exit status**, and expect a non-zero exit from that
sixth command.

The report is written identically in both modes and reaches a plain `cargo test`
with no `--nocapture`, through the `sh -c 'cat >&2'` mechanism.

### The structural channel

Red in **every** mode, asserted before the gated assertion. Six kinds, each
verified to fire (section 7):

1. a row with no probe program;
2. a probe program no row names;
3. a row no `owning_phase` arm covers;
4. the two crate engines disagreeing on a probe;
5. an oracle run that did not finish;
6. a bootstrap shape the `.orx` scan reached zero occurrences of.

Kind 4 is an unconditional `assert!` inside `run_on_both_engines` that names the
program and prints both engines' three descriptors -- it aborts the run rather
than being collected, which is Task 1's shape: a structural failure routed
through a verdict channel is silently absorbed.

Kind 5 is `did_not_finish` on the `CppOutcome`'s `Termination`, checked before
`expect_exit_code` is called, so a killed or crashed oracle never contributes an
exit code to a comparison.

---

## 7. Mutations, recorded as run

### Mutation 2 -- the runnable half

The brief's mutation 2 has two halves. **The property half** -- *the table types
no expected oracle bytes at all* -- is asserted by review, and it holds: there is
no recorded oracle answer in `gate_table_d.rs`, in `gate_tables/`, or beside the
probes. Every row's oracle column comes from `oracle.run(&abs)` on the run that
reports it. Measured rather than
asserted: `/bin/grep -ac` over the three new source files for `rc=1`, `Error 9`,
`Error 4`, `Error 2`, `main` in quotes, and each of the five oracle exit statuses
this table's probes produce, finds **zero** of each; and
`/bin/grep -anE '[0-9]{3,}'` over the same three files lists only the copyright
year and two document dates in comments.

**The runnable half is to perturb the crate's answer on a row that is already
`agree` and confirm it reddens.** I ran it three ways, each perturbing a
different descriptor at the single `Outcome` construction in
`rexx-exec/src/lib.rs`, each reverted afterwards (`git diff` over
`crates/*/src/` empty, and the release `rexx-run` rebuilt after the last
revert):

| mutation | what it did | result |
|---|---|---|
| **A** -- append `!` to the crate's `stdout` | all 31 `agree` rows -> `diverge-stdout`; 5a's not-`agree` count 10 -> 36; gated rows 10 -> 36 | fires |
| **B** -- append `!` to the crate's `stderr` | all 31 `agree` rows -> `diverge-stderr`; gated rows 10 -> 36 | fires |
| **C** -- add 3 to the crate's exit status | all 31 `agree` rows -> `diverge-status`; gated rows 10 -> 36 | fires |

**Three of the five verdict cells have no witness in table D on the unmutated
commit** -- measured, the table is 31 `agree` and 48 `diverge-both`, and nothing
else. These three mutations are what witnessed `diverge-stdout`,
`diverge-stderr` and `diverge-status` on real rows and real oracle runs, so
every cell of the function has now been produced by a program rather than only
by the cube test. **The plan's own control asked for one perturbation; running
three is what turned "the cells are exhaustive" into "each cell fires".**

Mutation C also moved the `loud` count from 48 to 2 while leaving all 48 rows
non-`agree`: the two survivors are the parse-error rows, whose refusal returns
through a different site the mutation did not touch. That is the column doing
its job -- 46 rows non-`agree` and **not** loud is exactly "the crate answered,
wrongly".

### Mutation 3 -- delete a row's probe

`corpus/gate-tables/directives/class__public__subkeyword.rex` moved aside,
`cargo test --release --test gate_table_d` run in **report mode with no
environment variable set at all**:

```
structural failures, which are red in every mode and are not verdicts
`REXX_CORPUS_GATE` could relax:
  gate-tables/directives/class__public__subkeyword.rex: a row has no probe
  program. A row without one is structural and is never skipped: the table
  reddens on the missing program rather than shrinking to the rows that still
  have one
```

Exit 101. The report above it did shrink to 78 rows -- and the table was red
anyway, which is the point of the assertion being over the missing program
rather than over the count.

**The other direction, also run:** copying a probe to
`class__nosuchkeyword__subkeyword.rex` gives
*"a probe program no row names, so nothing runs it"*, also in report mode.

### Mutation 1 -- named against Task 7, not run here

The brief's mutation 1 -- narrow `ClassDirective`'s handling so a `MIXINCLASS`
is admitted at rc 0 through `subclass.is_some()` -- **is Task 7's and is not run
here, and I did not fake it.** Its discriminator is `.M~baseClass`, which this
crate cannot answer today. Applying the narrowing now leaves the row non-`agree`
because `~baseClass` is unimplemented, so the check would do the same thing
whether or not the claim is true. At Task 7 it is a real flip: `.M~baseClass`
agrees at `The Object class` and the narrowing makes it `The M class`, which is
`diverge-stdout`.

**Table D's `::CLASS MIXINCLASS` row is already non-`agree` today** for an
unrelated reason -- the crate refuses the keyword outright, `rexx-exec: ::CLASS
MIXINCLASS is not implemented (Phase 5)` -- which is the second reason the
mutation cannot demonstrate anything here.

### The other structural channels, each verified

| channel | how it was made to fire | what it printed |
|---|---|---|
| engines disagree | append `!` to `stdout` only when the engine is `Ir` | *"the two engines disagree on .../annotate__attribute__subkeyword.rex, which is a structural failure and not a verdict"*, with both engines' three descriptors |
| a row no phase arm covers | delete `::ANNOTATE` from `owning_phase`'s 5a arm | six rows, each *"no arm of `owning_phase` covers this row, so nothing owes it an `agree`"* |
| a shape the scan cannot reach | narrow shape 1's `INHERIT` requirement | *"the scan reached no occurrence of this shape in either bootstrap file"* |
| a verdict cell that cannot fire | map `(false,false,true)` to `DivergeBoth` | *"the `diverge-stderr` cell has an empty preimage over the descriptor cube"* |

The one structural channel I did **not** make fire is the oracle not finishing.
Doing so would need a probe that hangs or crashes the oracle, and this project's
rules forbid committing one; Task 1 verified that channel with its own hanging
program and `corpus.rs` carries the same check on the same `did_not_finish`
predicate.

---

## 8. Verification, and what I confirmed rather than trusted

### `corpus.rs`'s directory scan does not see the new subtree

Confirmed at the code, as the brief asks, and **at four sites rather than one**:
`corpus.rs`, `coverage.rs`, `collect_stress.rs` and `ir_dual.rs` each carry
their own `phase_subset_files_on_disk`, all four reading `corpus/` with a
**non-recursive** `read_dir` filtered to `starts_with("phase-")` **and**
`ends_with(".txt")`. Either property alone excludes
`corpus/gate-tables/directives/`.

**And the confirmation is an assertion that already exists**, not a sentence:
`corpus.rs`'s `the_differential_reads_every_phase_subset_file` holds
`SUBSET_FILES` equal to `phase_subset_files_on_disk()`. Had the subtree been
picked up, that test would fail. It passes, with the subtree committed, in all
five gate commands.

### The scan that *does* see it, which the brief did not name

`rexx-oracle`'s `rexx-diff` walks `corpus/` **recursively** for `*.rex`. Its
documented use is the self-test -- the same binary as both arms -- which needs
determinism, not agreement. **Measured with the subtree in place: 204 programs,
0 divergences, exit 0.** Its cross-implementation invocation now reports these
probes' divergences, which is the table's subject rather than a corpus defect;
both `corpus/README.md` and the new `corpus/gate-tables/README.md` say which
rule binds these files and which does not.

Nothing runs `rexx-diff` from a test, so this changes no gate.

### Every probe was run by hand through `rexx-run` under `timeout -s KILL 10`

All 79, from the committed location, from a fresh empty directory with absolute
paths. **None was killed by the timeout** (no rc 124, no rc 137). This is a
human check written down as one, because the crate side of the harness runs in
process and cannot be bounded the way Task 1 bounds the oracle side.

The by-hand sweep also produced the verdicts independently of the harness --
oracle plus `REXX_ENGINE=tree-walker` plus `REXX_ENGINE=ir`, three descriptors
compared with `cmp`, from a shell -- and got 31 `agree`, 48 `diverge-both`, 48
loud, 0 engine disagreements: the same numbers the harness reports.

### The oracle was run under the standard wrapper

`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 10 .../build/bin/rexx FILE )`
for the by-hand sweeps, three descriptors read separately, never `2>&1`, from a
fresh empty directory. In the harness it is `support::oracle`, which applies the
same `ulimit -v` and carries Task 1's deadline. Nothing in
`corpus/oracle-crashes.txt` was run.

### The five gate commands, each with its own exit status

From `rust/`, at `db312da3e`:

| command | exit |
|---|---|
| `cargo fmt --all --check` | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| `cargo test --release --workspace` | **0** |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0**, corpus **106 of 106 matching** |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0**, corpus **106 of 106 matching** |

And the sixth, which is not one of the five:

| command | exit |
|---|---|
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` | **101**, corpus **106 of 106**, table D **10 rows owned by 5a not `agree`** |

### No `unsafe`, and no dev-dependency

Nothing added. `emit_uncaptured` uses the `sh -c 'cat >&2'` child rather than a
raw-fd constructor for the reason `corpus.rs` records.

---

## 9. The performance pin, checked rather than assumed

No sitting is owed, but the staleness test is worth reporting because **it
already fails at my base, for a reason that is not about my task.**

The global constraints' test is: the pin's commit is an ancestor of the task's
base, **and** `git diff <pin-commit> HEAD -- rust/crates rust/Cargo.toml Cargo.toml`
is empty.

* First half: `git merge-base --is-ancestor 15a1ffa98 094fbbdd8` succeeds.
* Second half: **not empty at `094fbbdd8`, before this task.** The differing
  paths are Tasks 1 and 2's changes under `rexx-exec/tests/`, and Task 3's
  `rexx-extract` crate -- `src/bin/rexx-extract-docs.rs`, `src/docs.rs`,
  `src/docs/*.rs`, `src/lib.rs` -- plus `rexx-extract/tests/extract_docs.rs`.
* **None of the four crates the guard names has a `src/` change.**
  `git diff --name-only 15a1ffa98 HEAD -- rust/crates | grep /src/` lists only
  `rexx-extract` paths, and `cargo tree -p rexx-exec --edges normal` does not
  contain `rexx-extract`, so nothing that changed can reach `rexx-run`.

So the test as literally written mandates a rebuild that could not change the
binary -- which is the same defect the constraints warn about for the phrasing
they rejected: **a test phrased over more than what the binary is built from
fails on changes that cannot move it.** The narrower predicate that matches the
intent is `rust/crates/{rexx-exec,rexx-core,rexx-classes,rexx-lib,rexx-num,rexx-parse,rexx-inventory}/src`
plus the manifests -- `rexx-run`'s actual dependency closure.

**What I did not establish.** I did not verify by rebuild that the pin's sha256
still reproduces. An incremental rebuild from my working tree gave a different
hash, but that measurement cannot separate "the source moved" from "the build is
not bit-reproducible under `debug = true` from a dirty `target/`", so it
establishes nothing and is not offered as evidence. The clean-tree rebuild at
`15a1ffa98` that `PINNED.md` prescribes is what would settle it, and the first
task that owes a sitting should run it.

---

## 10. What the table cannot see

* **A keyword neither `dire.xml` nor `DirectiveParser.cpp` names is outside the
  denominator**, because the row set is exactly their union.
* **A construct that is not a directive keyword is outside it too.** That is
  table C's half, and it is why `unkno` and `reqstr` survived three reviews
  under a table-D-shaped gate.
* **A keyword this crate accepts and ignores reads `agree` when the oracle
  produces no observable difference either.** A probe's three descriptors are
  all the row can see, and for most of this surface the effect of an option is
  only visible through a reflection query. `::ATTRIBUTE DELEGATE` and
  `::METHOD DELEGATE` are the live instances: both read `agree` today, and
  neither row can distinguish "the crate installed a delegating accessor" from
  "the crate parsed the keyword and dropped it". That discriminator is Task 15's
  and Task 5's, not this table's.
* **The `.orx` usage column's operand table is hand-written from
  `DirectiveParser.cpp` and is not derived from it.** A change upstream to what
  a keyword consumes would move a hit without moving the table. The report
  prints every hit's `file:line`, so a reader can check one against the file
  rather than take the count.
* **The owning-phase assignment is unchecked.** A row filed under the wrong
  phase escapes gating and nothing here notices; it is visible in a diff.
* **The `loud` column distinguishes nothing on this commit**, because every
  non-`agree` row is a refusal. It starts carrying information the first time
  this crate answers a directive row wrongly rather than declining it.
* **The table says nothing about a directive with no options.** `::CONSTANT`
  has no row here, because it has no keyword in either authority -- and it is
  the directive the third bootstrap shape is about.
* **The oracle-did-not-finish channel is untested by this task**, for the reason
  in section 7.
* **Both `.orx` files are read from the C++ tree at a path this crate hardcodes.**
  If that tree moves, `scan` panics naming the missing file rather than reporting
  an empty scan -- but a tree that moved *and still had the files* with the
  shapes rewritten would show up only as a structural failure naming the shape,
  which is what the fourth control demonstrates.

---

# Fix round 1

Base `db312da3e`, landed as **`0d4a63feb`** (the lead's `45a10c7fe`, a document-only correction of
the plan's own `forbid` sentence, sits between them). Nine findings, all closed. **The 5a non-`agree` count is unchanged at 10** --
predicate: rows whose committed `owning_phase` answers `"5a"` and whose verdict is not `agree`, where
`agree` means the crate and the oracle match on exit status, `stdout` and `stderr` with `stderr`
compared raw; denominators 36 rows owned by 5a and 79 rows in the table. Whole table still 31 `agree`
and 48 `diverge-both`. **M1's fix does not move the row population**, which was the thing to check:
its subject is a row leaving the table without a verdict, and no committed probe does that.

## M1 -- a probe that cannot be canonicalised now has a channel of its own

**Reproduced before fixing.** Replacing `class__mixinclass__subkeyword.rex` with a symlink to a
nonexistent path gave, in report mode with no environment variable set: `78 rows`, **no structural
failure, exit 0**, and `5a: 35 rows, 9 not yet agree`. The gated count fell by one with nothing red.

The `else { continue; }` claimed the row had already been reported as a missing probe. It had not:
that check compares the set of derived paths against `read_dir`'s listing, and `read_dir` lists an
entry by name whatever the name resolves to. A dangling symlink passes it.

**Fixed** by pushing a `Structural` in the `Err` arm, naming the row, the path and the io error, with
the comment saying the earlier check answers a different question rather than pointing at it.
**Recorded as run:** the same symlink now gives, in report mode,

```
gate-tables/directives/class__mixinclass__subkeyword.rex: the probe is listed in the
directory but cannot be resolved: No such file or directory (os error 2). The row for
::CLASS MIXINCLASS subkeyword therefore has no program to run, which is structural --
it is never a row the table drops
```

exit 101.

## M2 -- the verdict function is asked for a typed answer

`support/oracle.rs` gains **`DescriptorDiff`**, a struct with one `bool` per channel, produced by
`descriptor_diff_with`. `descriptor_diffs_with`'s `Vec<&'static str>` is now `DescriptorDiff::labels()`
-- a *rendering* of that value rather than a second computation of it -- so every existing caller that
prints the list or tests it for emptiness is unchanged, and `compare_raw` reads
`diff.exit_code`/`diff.stdout`/`diff.stderr` instead of `contains(&"exit code")`.

**The review's own combined mutation is the control, and it was run in both directions.**

| tree | table D reports |
|---|---|
| unmutated | `agree: 31`, `diverge-both: 48`, `5a: 36 rows, 10 not yet agree` |
| **before the fix**, crate exit status wrong on every row **and** `"exit code"` renamed to `"exit status"` | byte-identical to unmutated -- the reviewer's measurement |
| **after the fix**, the same combined mutation | **`diverge-status: 31`**, `diverge-both: 48`, **`5a: 36 rows, 36 not yet agree`** |
| after the fix, the rename **alone** | `agree: 31`, `diverge-both: 48`, `5a: 10` -- unchanged, which is correct: renaming a display label must not move a verdict |

The mutation's exit-status half is now seen despite the rename, and the rename on its own is inert.

**A second thing that could still have gone wrong is now asserted.** `corpus.rs` reads the label list
for *emptiness*, so a channel silently dropped from the rendering would make a real divergence look
like agreement there -- a hazard the typed producer moves rather than removes.
`labels_name_exactly_the_channels_that_differ` walks all eight combinations of the three fields and
holds the rendering equal to them. **Recorded as run:** dropping the `exit code` label from
`labels()` fails it, in `gate_table_d` and in `corpus` alike, naming
`DescriptorDiff { stdout: false, stderr: false, exit_code: true }`.

## L1 -- the two-engine run checks `chunks_refused`

`run_on_both_engines` now asserts both arms report zero refused chunks, after the
descriptors-agree assertion and for a reason that assertion cannot cover: `Engine::Ir` runs a body
the instruction stream cannot hold on the tree-walker instead, so a refused body makes "the two
engines agree" a comparison of two tree-walker runs while the row is still reported as a two-engine
measurement. `ir_dual.rs` checks the same count.

**Latent, not actual: it passes on all 79 probes.** **Recorded as run** by adding one to the ir arm's
`chunks_refused` at the crate's `Outcome` site, which fires naming the program: *"a body of
.../annotate__attribute__subkeyword.rex did not run on the engine it was attributed to: the ir arm
refused 1 bodies to the tree-walker and the tree-walker arm counted 0"*.

## L2 -- the `FORM` readbacks

Measured at the oracle: `say form()` with no directive answers `SCIENTIFIC`, so
`::options form scientific` + `say form()` distinguished nothing.

* **`options__form__subkeyword.rex`** now sets `ENGINEERING`. Measured: `SCIENTIFIC` with no
  directive, `ENGINEERING` with it -- the readback discriminates.
* **`options__scientific__value_of_form.rex`** cannot be fixed the same way, because `SCIENTIFIC` is
  the default. Its `form()` is replaced by `say 'main'` and the file says why in a comment: this
  row's force is acceptance of the keyword, like every non-readback row here.

Both were re-run by hand through `rexx-run` under `timeout -s KILL 10` and through the oracle under
the standard wrapper.

## L3 -- the workspace lint is `deny`, not `forbid`

Corrected in `gate_tables/mod.rs`, and in the two files the wording was inherited from,
`corpus.rs`'s and `support/oracle.rs`'s. All three now say the lint is `unsafe_code = "deny"`, that an
`unsafe` site is therefore a **grantable exception** rather than a closed door, that the bar is
`rust/CLAUDE.md`'s and the granted set is asserted by `rexx-core/tests/unsafe_sites.rs`, and that
this particular exception was not asked for because a shell builtin already does the job.
`/bin/grep -rn 'unsafe_code = "forbid"' crates/` now matches nothing.

## L4 -- a mutable set's size

*"the four copies of `phase_subset_files_on_disk`"* is now *"every copy of"*, in `gate_table_d.rs`'s
module doc and in `corpus/gate-tables/README.md`.

## L5 -- the identical-probe account was short, and is now derived rather than counted

The report said *"two pairs"* and then described seven. Re-derived by `md5sum` over the committed
probes after L2's change, the groups sharing content are: `ALL`/`SYNTAX value-of(ALL)`; the six
`<condition>`/`SYNTAX value-of(<condition>)` pairs; `NUMERIC`/`INHERIT value-of(NUMERIC)`; and, new
from L2, `FORM`/`ENGINEERING value-of(FORM)`.

**They are one rule, not three kinds**, which is why the earlier prose could be short without looking
wrong: a keyword that takes a value cannot appear without one, and a value cannot appear without its
keyword, so a `<keyword> subkeyword` row and one of its `value-of(<keyword>)` rows are exercised by
the same clause. `SCIENTIFIC value-of(FORM)` left that set with L2's change, because its probe now
carries the comment saying its readback cannot discriminate.

## L6 -- every `agree` row of this table is acceptance-only

The earlier "what the table cannot see" named `::ATTRIBUTE DELEGATE` and `::METHOD DELEGATE` as *"the
live instances"* of accept-and-ignore. **The property holds for every `agree` row in the table.** No
probe here instantiates or invokes anything: each is `say 'main'` (or a `NUMERIC`-family readback)
plus directive clauses, so its three descriptors are the same whether the keyword's effect is
implemented or the keyword is parsed and dropped.

**The exceptions in principle are the rows whose probe reads the option back with a builtin**, and
because a `<keyword> subkeyword` row and its `value-of(<keyword>)` row are exercised by the same
clause -- L5's rule -- they come in the same shape: `::OPTIONS DIGITS`, `::OPTIONS FUZZ`,
`::OPTIONS FORM` and `::OPTIONS ENGINEERING value-of(FORM)`, the last two byte-identical since L2.
`::OPTIONS SCIENTIFIC value-of(FORM)` is *not* among them and its probe says so in the file, because
`SCIENTIFIC` is the default. Every one of these is a 5c row that does not `agree` today, so none of
them is currently an `agree` row an accept-and-ignore implementation could slip past -- which is why
the first sentence of this section holds without qualification.

**The limitation is forced, not chosen**, and the reviewer measured it: the obvious discriminators --
`.k~new~m` against `::method m abstract`, an outside call against `::method m private`, `o~at = 5`
against `::method at attribute`, `.k~new` against `::class k abstract` -- are oracle rc 163, rc 159,
rc 0 `5` and rc 158, and this crate answers rc 120 `method "NEW" of class "Object" is not
implemented` to all four. There is no probe this table could have carried at this commit that would
see the difference; that discriminator arrives with the tasks that implement the effects, and the
reflection half of it is table C's.

## L7 -- carried forward rather than decided

The `::ROUTINE EXTERNAL` row is left filed under `7`, and the consequence is now written where the
task that draws the boundary will meet it: in `owning_phase`'s own doc beside the arm, and in the
probe file's first comment. Both say the same thing -- a row's identity here is (directive, keyword,
position), so `'LIBRARY <lib> <entry>'` and `'LIBRARY REXX <entry>'` are **one row** and the probe
picks the first; if the `LIBRARY REXX` form moves into 5a the way `::METHOD`'s did, that behaviour
has no row in this table at all. Closing it needs the row set to distinguish the two forms, which is
`corpus/docs/directive-options.txt`'s shape and not this table's.

## Verification, at the fix-round commit

| command | exit |
|---|---|
| `cargo fmt --all --check` | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| `cargo test --release --workspace` | **0** |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0**, corpus **106 of 106** |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0**, corpus **106 of 106** |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` | **101**, corpus **106 of 106**, **10 rows owned by 5a not `agree`** -- the design, not a regression |

`git diff --stat -- rust/crates/*/src/` is empty across this round as well, so **still no sitting**:
every mutation touching `src/lib.rs` was reverted and the release `rexx-run` rebuilt afterwards.

## What I could not close in this round

* **The oracle-did-not-finish channel is still unfired**, for the reason it was before: making it
  fire needs a probe that hangs or crashes the oracle, which this project forbids committing.
* **`owning_phase`'s assignment is still unchecked by anything.** That is by design and is stated in
  the module doc; L7 narrows the one row where the boundary runs *inside* a row rather than beside
  it, but does not check the other seventy-eight.
* **`StderrComparison::Raw` still has no runtime witness in this table.** The reviewer measured it:
  flipping every row to `Normalized` leaves `agree: 31`, `diverge-both: 48` and the gated count at
  10, unchanged. The property is real and is an input to the verdict, but no committed probe's
  `stderr` differs only in a trace indent, so nothing here would notice the flip. The first probe
  whose row turns on a trace transcript is what makes it witnessed.

---

# Fix round 2

Base `0d4a63feb`, landed as **`fe4c95913`**. One low, one report-only low, three nits; all five
closed. **The 5a non-`agree` count is unchanged at 10 of 36** -- predicate: rows whose committed
`owning_phase` answers `"5a"` and whose verdict is not `agree`, `agree` meaning the crate and the
oracle match on exit status, `stdout` and `stderr` with `stderr` compared raw. Whole table still 31
`agree`, 48 `diverge-both`. **Still no sitting**: `git diff --stat -- crates/*/src/` is empty.

## N1 -- a deleted probe reported twice, and the second message was false

**Reproduced first.** Moving `class__public__subkeyword.rex` aside gave two `Structural` entries for
one row: the set check's *"a row has no probe program"*, and then the `Err` arm's *"the probe is
listed in the directory but cannot be resolved"* -- which asserts a listing entry that is not there.
Both cases produce the same `os error 2`, so the message's one distinguishing claim was the one thing
it did not check.

That is the commonest structural failure this table has -- it is the brief's own mutation 3 -- so it
is the message a reader meets most often, and it points at a symlink or permission problem that does
not exist.

**Fixed** by guarding the push on `on_disk.contains(&probe)`: the set the listing produced is exactly
the fact that separates the two cases. **Both cases run, each reporting once:**

| case | message | exit |
|---|---|---|
| probe deleted | `a row has no probe program. A row without one is structural and is never skipped...` | 101 |
| probe a dangling symlink | `the probe is listed in the directory but cannot be resolved: No such file or directory (os error 2). The row for ::CLASS PUBLIC subkeyword therefore has no program to run...` | 101 |

Both in report mode, with no environment variable set.

## N2 -- the readback-probe sentence was short by one, and contradicted L5

The L6 paragraph named `::OPTIONS DIGITS`, `::OPTIONS FUZZ` and `::OPTIONS FORM` as the rows whose
probe can tell acceptance from effect. **`::OPTIONS ENGINEERING value-of(FORM)` is a fourth**, and is
byte-identical to the `FORM` probe since round 1's L2 -- which is L5's own rule (a `<keyword>
subkeyword` row and its `value-of(<keyword>)` row are exercised by the same clause) arriving two
sections later and being contradicted. A reader counting the rows that would notice an
accept-and-ignore implementation would get one fewer than there are and conclude that a `value-of`
row never discriminates.

Rewritten to derive the set from L5's rule rather than list it, to name
`::OPTIONS SCIENTIFIC value-of(FORM)` as the one that is deliberately *not* in it, and to say the
thing that makes the section's opening sentence hold without qualification: every one of these is a
5c row that does not `agree` today, so none of them is an `agree` row an accept-and-ignore
implementation could slip past.

## The three nits

* **The new comment's example was not its arm's case.** It said a probe *"whose bytes cannot be
  reached"* takes the `Err` path. Measured: a `chmod 000` probe canonicalises fine and fails later,
  in `run_on_both_engines`'s own `fs::read`, panicking with the path -- still red, but not here. The
  comment now gives the dangling symlink as the case and names the unreadable file as the thing this
  arm is *not* about, since that is the other reading "unresolvable" invites.
* **`run_on_both_engines`'s doc said "the assertion" with two under it.** Both the function doc and
  the module doc's "Both engines run" section now carry both, and say why the second is not covered
  by the first: the ir arm refusing a body leaves the two arms *agreeing* while only one engine ran
  the program. That is the half a reader of table C would need.
* **The 93-character doc line is rewrapped**, along with two of this round's own at 81 and 95 that
  the same check found. Verified per file over `///`, `//!` and `// ` lines: **no line this task
  added exceeds 80 characters**, and `git diff` over the two shared files shows no added line above
  it. The four that remain long in those files predate this task. **No gate can see any of this** --
  `cargo fmt` does not rewrap doc comments -- so the instrument is a person looking, which is how it
  arrived.

## Verification, at `fe4c95913`

| command | exit |
|---|---|
| `cargo fmt --all --check` | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| `cargo test --release --workspace` | **0** |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0**, corpus **106 of 106** |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0**, corpus **106 of 106** |
| `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1 cargo test --release --workspace` | **101**, corpus **106 of 106**, **10 rows owned by 5a not `agree`** -- the design until the phase closes |

## What is still open, carried forward unchanged

The oracle-did-not-finish channel is unfired, `owning_phase`'s assignment is unchecked by anything by
design, and **`StderrComparison::Raw` still has no runtime witness in this table**: flipping every
row to `Normalized` moves no count, because no committed probe's `stderr` differs only in a trace
indent. The first row that turns on a trace transcript is what witnesses it.
