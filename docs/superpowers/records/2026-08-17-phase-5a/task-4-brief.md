## Task 4: gate table D -- the directive and option surface

**Goal.** A `#[test]` whose rows are Task 3's `directive-options.txt`, whose verdicts come from
running programs, and which carries an oracle column per row.

**Build:**

* **The shared harness**, in `crates/rexx-exec/tests/gate_tables/`: row loading, the runner, the
  report, the gate mode, and **the verdict function** -- five cells (`agree`, `diverge-status`,
  `diverge-stdout`, `diverge-stderr`, `diverge-both`), mutually exclusive and jointly exhaustive over
  the three-descriptor comparison, so exactly one applies and no precedence question arises. **`loud`
  is a separate boolean column and never a verdict**: its predicate is on the crate's output alone, so
  it can co-fire with any cell, and a row that is `diverge-status` *and* loud is exactly the
  `::ANNOTATE` row this plan closes in Task 20.
* **Both crate runs, in-process**, selected by `Invocation::with_engine` -- the way `ir_dual.rs` does
  it. **Not `REXX_ENGINE`**, which only the `rexx-run` binary reads. The two crate outcomes must agree
  with each other on all three descriptors **before** a verdict is computed; a disagreement is a
  **structural** failure, not a verdict.
* **`StderrComparison::Raw` on every row.** The default `Normalized` is DEVIATION 0 and collapses the
  spaces after a trace prefix; stderr equality is an *input* to the verdict function, so normalising
  silently converts `diverge-stderr` into `agree`. Rows that run under `TRACE` are named as such and,
  if they are corpus programs, also listed in `corpus.rs`'s `RAW_STDERR_COMPARISON`.
* **One committed probe program per row**, under `rust/corpus/gate-tables/directives/`. A row without
  one is **structural**, red in both modes, never "skipped". These are not corpus programs: most
  diverge today, and `phase-5a.txt` requires agreement. A probe moves into `phase-5a.txt` in the task
  that makes it agree. (`corpus.rs`'s directory scan reads only `phase-*.txt`, so a new subtree adds
  no obligation -- confirm that in this task rather than trusting this sentence.)
* Columns: probe path; oracle rc/stdout/stderr; crate rc/stdout/stderr on **both** engines; verdict;
  `loud`; owning phase; and **which of `CoreClasses.orx` / `StreamClasses.orx` uses the shape,
  derived by scanning both files** rather than asserted. The scan must reach four shapes, **all four
  read at the file this session** -- the spec inherited three of them without re-reading and this plan
  does not: `::CLASS 'InputOutputStream' public MIXINCLASS Object INHERIT InputStream OutputStream`
  (`StreamClasses.orx:115`), a quoted class target (`:371`, `subclass 'Supplier'`), an install-time
  `::CONSTANT` sending a private class method of its own class (`:546`-`:549`), and a directive
  carrying its body on the same physical line (`CoreClasses.orx:151`).

**Verification, runnable now.** Every probe is a directive program, and both interpreters run
directive programs today -- measured above across the whole `::METHOD` option battery, `::CLASS`
`MIXINCLASS`/`METACLASS`/`INHERIT`, `::ANNOTATE`, `::CONSTANT` and `::REQUIRES`. The three named
falsifying mutations are **recorded as run**:

1. make the crate accept a still-refused keyword silently: the row's `loud` clears and its verdict
   becomes `agree` **only if the oracle agrees too**. This is M10 -- `ast.rs`'s `ClassDirective` holds
   `SUBCLASS` and `MIXINCLASS` in one slot separated by `mixin: bool`, so a narrowing written against
   `subclass.is_some()` admits `MIXINCLASS` at rc 0. **This one is Task 7's and not this task's**, and
   the reason is the same defect this plan relocated table C's controls for: its discriminator is
   `.M~baseClass`, which the crate cannot answer until Task 7, so applying the mutation here leaves
   the row non-`agree` **because `~baseClass` is unimplemented** -- the check does the same thing
   whether or not the claim is true, which this plan's own constraint calls decoration. At Task 7 it
   is a real flip: `.M~baseClass` agrees at `The Object class`, and the narrowing makes it
   `The M class`, which is `diverge-stdout`.
2. **the table types no expected oracle bytes at all** -- every one is re-read from the oracle on the
   run that uses it. That is a property of the code and is asserted by review, not by a mutation; the
   runnable half is to perturb the crate's answer on a row that is already `agree` and confirm it
   reddens. Green rows exist here from the first commit: measured, the six `::METHOD` options
   (`PUBLIC`, `PACKAGE`, `GUARDED`, `UNGUARDED`, `PROTECTED`, `UNPROTECTED`) and `::CLASS K PRIVATE`
   and `::CLASS K ABSTRACT` are all byte-identical on both sides today.
3. delete a row's probe: the table reddens on the missing program rather than shrinking.

**What it cannot see.** A keyword neither `dire.xml` nor `DirectiveParser.cpp` names is outside the
denominator. And the table says nothing about a construct that is not a directive keyword -- that is
table C's half, and it is why `unkno` and `reqstr` survived three reviews under a table-D-shaped gate.

**Done when** every row has a probe and a verdict, the report prints in both modes, **mutations 2 and
3 are recorded as run here and mutation 1 is named against Task 7**, and the five gate commands pass.
No sitting.

---

