# Phase 5c: what it did, what it hands over, and why it does not flip

**Written 2026-09-03 at `64258d712`**, after 5c's implementation work finished. Its spec is
`docs/superpowers/specs/2026-09-02-phase-5c-class-methods.md` and its plan
`docs/superpowers/plans/2026-09-02-phase-5c.md`.

---

## What 5c delivered

| | at 4c553f383's successor | now |
|---|---|---|
| table C, 5c rows not `agree` | 868 | **110** |
| table D, 5c rows not `agree` | 35 | **2** |
| method probes byte-identical to the oracle | 63 of 96 | **89 of 96** |

Eleven commits: the construction route and surface B (`70bb5ea9b`), 24 constructors (`a55fc4bf6`),
the four no-`~new` routes (`b4adf8a1e`), the `>x` silent divergence (`989c307e2`), the reflection
classes (`563074c18`), `::OPTIONS` (`82a8ae86d`, `231a2bb71`), `Stem` (`a7b72abea`), both dark
benchmark axes and `identityHash` (`15fc08c72`), Deviation 7 (`d769a847c`), and the real `98.972`
with `String~makeArray` (`64258d712`).

**The phase's premise was wrong when it started and a plan review caught it before any code was
written.** Table C's method probes are `hasMethod` readbacks -- 96 of 96 have no `say` line that is
not `hasMethod` -- so a row agrees when the probe *constructs* and the class *answers the documented
name set*, never when a method works. The original spec's nine mechanisms would have moved zero
rows. That review was the highest-value hour of the phase.

---

## Why 5c does not flip, and what has to happen before any phase can

**Adding `"5c"` to `CLOSED_PHASES` fails.** Measured 2026-09-03 in a `git archive` extract of
`64258d712` with its own `CARGO_TARGET_DIR`: `gated by this run: 110 row(s)`, `test result:
FAILED`, rc 101. `verdict_is_gated` gates every row whose owning phase is closed, and
`METHOD_PHASE` is one const putting **all 1347 method rows under 5c**.

**Do not create `corpus/phase-5c.txt` either**, for the reason Task 4 measured: it obliges
`CLOSED_PHASES` to name 5c, which is the same failure by a longer route.

### The blocker is the row set's filing, not 5c's work

Of the 110:

| rows | classes | can they ever `agree`? |
|---|---|---|
| 82 | `File` `Stream` `StreamSupplier` | yes, when 5d lands |
| 23 | `StackFrame` `Alarm` `Ticker` | yes, when method bodies land |
| **5** | **`Pointer`** | **no, and this is the finding** |

`Pointer`'s five rows read `unanswered`, which is neither `agree` nor `diverge`: the probe raises at
`.Pointer~new` before any documented name is asked, so there is no comparison to make. **D73 makes
that refusal permanently correct** -- the reference says instances come only from native code
(`utilityclasses.xml:6910`), and `class-set.txt` marks the class `unreachable`. Task 3 measured the
consequence directly: matching the oracle's refusal byte for byte "would make each probe
byte-identical and move **zero** rows".

**So while `METHOD_PHASE` files every method row under one phase, and `Pointer`'s rows are among
them, no phase owning method rows can ever be closed.** That is a defect in the row set's filing and
it outlives 5c: 5d would inherit it unchanged.

### What the fix looks like, and who owns it

The owner of a method row has to become a property of its **class**, not a single const. The three
groups above are the evidence for what the values should be, and `Pointer` needs a value meaning
*this row is never expected to agree* rather than a phase name -- the `unreachable` status
`class-set.txt` already carries is the natural source.

**This is a spec-level decision about who owns what, so it belongs with 5d's spec rather than
smuggled into a flip.** It is small in code and consequential in meaning, which is exactly the shape
that should not be decided by the task that merely wants a green gate.

---

## Handovers, each with its measurement

### To Phase 5d -- I/O, 82 rows

`File` 50, `Stream` 24, `StreamSupplier` 8. The subject is the operating system and the divergences
are environmental, which is why they were split out on 2026-09-02.

**`StreamSupplier` has no public constructor**: `.StreamSupplier~new` is rc 159 `Error 97`, and its
only route is `.stream~new('f')~supplier`, rc 0. So its rows cannot leave `not-covered` until
`Stream` does. **5d owes a ruling on whether it is `unreachable`** in `class-set.txt`'s sense -- the
book calls it a snapshot of a stream, which is that status's shape, but the sentence has to be cited
and re-read rather than paraphrased.

### To a method-body phase -- 23 rows

* **`StackFrame` 10.** Both documented routes refuse at rc 120 on both engines, re-measured
  2026-09-03: `.context~stackFrames` is `method "STACKFRAMES" of class "RexxContext" is not
  implemented`, and `condition('O')` answers a `Directory` this crate does not build. The condition
  route needs a `SIGNAL ON` and a label *above* the readbacks, which the construction cell's
  trailer cannot express, and both routes need a `StackFrame` object model on a `RexxContext`
  method body.
* **`Alarm` 7 and `Ticker` 6.** `Alarm~init` reaches `DateTime`/`TimeSpan` arithmetic on every path
  and this crate refuses the operator applied to a user-class instance; `Ticker~init` refuses
  `String~sign` and then reaches `!createTimer`, `guard off` and `reply`. Both additionally need a
  **multi-clause construction cell**, because a live one keeps the interpreter running until it
  fires -- measured, `.Alarm~new(86400, ...)` printed its line and then outlived a 60 s bound, and
  two such interpreters ran for eight hours during this phase before being killed.

### To Phase 7

**`::REQUIRES x LIBRARY`**, re-filed beside `::ROUTINE EXTERNAL`, which that table already files
under 7. It needs a native library loader: none of the eight libraries this build ships loads, all
answering 98.903, and `::requires rexx library` is rc 0. A blanket 98.903 would agree on that row
and be a loud wrong answer elsewhere.

### Unowned, and each is a real divergence

* **`::REQUIRES 'file' NAMESPACE`.** The oracle finds a required file by four routes -- cwd, program
  directory, `REXX_PATH`, `PATH` -- plus extension appending. A narrower search turns an honest
  refusal into a loud wrong answer.
* **`~unknown`'s argument list.** `String~makeArray` now exists, and `~request('ARRAY')` uses it and
  agrees, but `unconverted_array_argument` does not send `MAKEARRAY`, so
  `o~unknown('LENGTH','notanarray')` under a syntax trap is still rc 120 here against the oracle's
  rc 0. The fix sends `MAKEARRAY` where the receiver's behaviour answers it, as `native_request`
  already does -- **and it must move `is_multi_dimensional_array`'s subject onto the converted
  array**, which is the part that makes it argument-conversion work rather than a one-liner.
* **`RootSet::promote` leaks one cell per referenced variable instance.** 5c introduced it with
  `>x`; nothing frees them. Closing it needs the collector to reach a cell from the references
  naming it. Stated at the field, handed over deliberately rather than by oversight.
* **A `Directory` subclass's entry writes** refuse where the oracle answers. Giving one a native
  body loses its variable pool (loud) **and its `UNINIT` (silent)**, so the obvious fix is worse
  than the gap.
* **`identityHash` is licensed, not matched** (`15fc08c72`): the oracle's answer is
  `((uintptr_t)this) ^ UINTPTR_MAX` and ten runs gave ten values, so there is nothing to match and
  matching its width would pin this machine's address range into the crate.

---

## Instruments this phase left, and one gap in them

**Left working:** the construction route (`class-set.txt`'s `construction` and `directives`
columns, carried by `Coverage::Covered` so the extractor cannot derive `covered` without naming a
route, with `OracleShape::Exactly` making a route that constructs nothing a structural failure);
Deviation 7's multiset stderr comparison with its six-mutation control; and three witness binaries.

**The gap, found 2026-09-03 and not closed:** a `corpus/lang/*.rex` named in no phase subset file is
**silently unrun**, and nothing catches it. `corpus.rs` pins which subset *files* the runner reads
and `coverage.rs` pins each file's *contents*, but nothing compares their union against the
directory -- 342 programs on disk, 331 in the differential. It was found by adding
`string_makearray.rex` and seeing the headline stay at `331 of 331`.

**Whoever writes `corpus/phase-5c.txt` folds in `variable_reference.rex`, `stem_object.rex` and
`string_makearray.rex`, deletes their three binaries, and should close this gap in the same change**
-- and must carry the stderr-comparison scope across first, because `corpus.rs` compares a subset
program's stderr as a **sequence** and `directive_options*` would bring Deviation 7's flake straight
back. The mechanism is `RAW_STDERR_COMPARISON` and `support::oracle::StderrComparison`.

---

## Two things that recurred, worth carrying into 5d

**The image usually already has the thing.** Three times a task was briefed to build a model and
found one: the reflection classes needed no constructors because `.Class~package`,
`.Object~subclass('k')`, `.Object~method(...)` and `.routines~r` already answer real objects;
`Stem` needed no object model because `Body::Stem` was already a value kind; and `identityHash`
needed no fix because it could not be matched at all. **Check what exists before designing.**

**A test pinning a refusal becomes a liability the moment the refusal ends.** Three had to be
un-pinned as implementations landed. A corpus program compared against the oracle does not have
this failure mode, and is the better instrument wherever it can be used.
