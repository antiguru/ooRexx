## Task 5: gate table C -- the concept and class surface

**Goal.** The second `#[test]`, sharing Task 4's harness: one row per `provide.xml` section, one
wiring row per class and per documented hierarchy edge, and one method row per (class, method) pair.

**Build:**

* **Concept rows.** One per section id, at every nesting level, each naming its probe program **and
  its negative control**. A section with no program is structural.
* **Wiring rows.** One per class in `class-set.txt`, asking `~id`, `~class`, `~superClass`,
  `~superClasses`, `~metaClass` and `~isA(.Class)`; one per edge in `hierarchy-edges.txt`, asserting
  the documented edge is **present in** the child's `~superClasses` -- never that it is the whole
  answer, since the hierarchy list gives one edge per class by construction. Plus the **`ArgUtil`
  assertion** from Task 3, which is the only thing covering the comment-stripping rule.
* **Method rows.** One per (class, method, arm), owned by 5c, reported not gated here. **The probe
  corpus is one program per class, not one per pair**: a single run asks the whole documented set and
  prints one line per name, because every row here costs an oracle process launch.
* **M9.** `corpus/lang/primitive_classes.rex` asserts `~id` across the primitive classes and is
  reached only as a parse fixture by `rexx-parse/src/instruction/tests.rs:1901`. Its job belongs to
  the wiring rows; it is either promoted into the row set's probe corpus or deleted, and either way it
  stops looking like coverage.

**Runnable now, and mostly red -- which is correct and is not a red suite.** Measured, `.array~id` is
rc 120 today, so most wiring rows read `diverge-status` and `loud`. They are 5a rows and become red
only when Task 24 closes the phase; until then the table is a progress report and the number of
non-`agree` 5a rows is what each task reports. The method rows' instance arm needs `~new`, which no
phase before 5b has: on the crate both engines refuse identically, so the rows are verdicts against
5c and not structural failures.

**The two rows this spec exists for, and their controls.** `unkno` and `reqstr` get concept rows here
and are red until Tasks 12 and 14. **Their negative controls -- delete the `UNKNOWN` step, delete the
`makeString` limb -- are recorded as run in those two tasks, not here**, because deleting a mechanism
requires having built it. That forward obligation is named in both directions: Task 12 and Task 14
each carry it in their "Done when".

**The four falsifying mutations, none of which can be run in this task, and two of which do not fire
on a table C row at all.** A mutation control demonstrates a row going from green to red, and **a row
that is already red cannot demonstrate anything** -- measured, `.array~id` is rc 120 today, so every
wiring row is red the moment it is created. Each control is therefore owned by the task where the
thing it checks first reads `agree`, and each of those tasks carries it in its "Done when":

1. drop a class from the registry -- its wiring row cannot answer and reddens. **Task 9**, on a table
   C row.
2. answer an own-scope query from a flattened all-scopes dictionary -- measured,
   `.Array~method("STRING")` raises 97.1 on the oracle although every Array instance answers `STRING`,
   and `.K~method("M")` raises for a class-side method the class genuinely has, so a flattened build
   answers where the oracle raises. **Task 9, and not on a table C row** -- see below.
3. implement a section's mechanism silently wrongly -- the concept row compares three descriptors
   under `Raw`, so `makeString` returning the wrong string reddens at rc 0. **Task 14**, on the
   `reqstr` concept row.
4. drop the operator-frame line -- **there is no table C row for it**, and the control fires on
   Task 6's corpus programs instead. See below.

**Two things table C does not have, said here rather than discovered by a control that cannot fire.**

* **No row for the operator-frame traceback line.** The spec names it as one of exactly two mechanisms
  with **no documented section**, "outside every denominator in this document" -- and table C's concept
  rows are one per `provide.xml` section id. So mutation 4 cannot fire on this table. Its instrument is
  the three corpus programs Task 6 commits to `phase-5a.txt` and `RAW_STDERR_COMPARISON`, where
  dropping the frame line reddens the corpus gate byte-exactly. This is the spec's own named hole
  reaching the gate, not a gap this plan introduces.
* **No scope row class.** The spec is two-minded -- table C's readback table carries a scope question,
  and its prose says the class arm of that question "is therefore **not** a row class in table C". The
  plan resolves it in the direction that adds no derivation: **the documentation supplies no expected
  answer for the scope question** (it says a class documents a method, never at which scope it is
  defined), so a scope row's key would come from the book and its expectation only from the oracle.
  The instrument instead is the `~method` probes Task 9 commits as corpus programs. **What that costs
  is stated rather than implied:** no row polices the scope question across the documented set, and
  the guard is a handful of named classes. If the spec author wants the row class, Task 5 builds it
  and Task 3 keys it, and it is measurable class-side -- `.Array~method("APPEND")~class` is
  `The Method class` while `.Array~method("STRING")` raises, both with no `~new`.

**What this task can and does record as run is the structural half**, which is red from the first
commit and therefore demonstrable now: delete a probe program; make one engine's answer differ from
the other's; edit the committed row file in each direction. Those are the failures the table exists
to be incapable of hiding, and they do not depend on any row being green.

**What it cannot see.** *Is m defined at X's own scope* has **no instrument on the class arm** --
measured, `.Array~hasMethod("ID")` and `.Array~hasMethod("DEFINE")` are both 1 though `id` and
`define` are `Class`'s, and `.Array~method("ID")` raises because `~method` reads the instance
dictionary. So a build that moved a class method between scopes is not caught here; the place it is
caught is a scope-override send (`~m:scope`), which has its own corpus programs from the old plan's
Task 5. Also: `~instanceMethods` is not a readback -- `.Array~instanceMethods` contains `DEFINE`,
`ID` and `OF` and not `APPEND`.

**Done when** every row has a probe, the three structural controls are recorded as run, each of the
four verdict mutations is named against the task that owes it, M9 is disposed of, and the five gate
commands pass. No sitting.

---

