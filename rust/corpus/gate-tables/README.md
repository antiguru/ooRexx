# Gate-table probe programs

**Not a differential corpus.** Almost every program here is the probe for one
row of a Phase 5 gate table -- `native-entries/` is the exception and has a
section of its own below -- and most of them diverge from the C++ oracle today
--
that is what the row records. Nothing here belongs in `phase-*.txt`, whose
entries mean "agrees with the oracle byte for byte"; a probe moves into a
phase subset file in the task that makes its row agree, and stays here as
well only if some row still needs it.

## `directives/`

Gate table D's probes: one per row of `../docs/directive-options.txt`, which
is the union of `dire.xml`'s and `interpreter/parser/DirectiveParser.cpp`'s
directive keywords. `crates/rexx-exec/tests/gate_table_d.rs` runs each one
under both crate engines and against the oracle, and derives the file name
from the row rather than looking it up, so a row and its probe cannot drift.
A row whose file is missing is a structural failure, and so is a file no row
names.

Each program exercises its row's keyword at its row's position and prints one
line, so the oracle side shows the program ran rather than that it produced
nothing.

## `concepts/`, `classes/`, `hierarchy/` and `methods/`

Gate table C's probes, run by `crates/rexx-exec/tests/gate_table_c.rs`:

* `concepts/` -- one per `<section id>` of `provide.xml`'s `provide` chapter,
  from `../docs/provide-sections.txt`. **Hand-written**, because nothing
  derives a program that exercises a documented mechanism; the file stem is
  the section id, so a section with no program is a missing file and
  structural.
* `classes/` -- one per row of `../docs/class-set.txt`, asking what the
  `.environment` entry renders as and what its class is -- questions any entry
  answers -- and then the questions the class surface is wired by, which only
  a class object answers.
* `hierarchy/` -- one per row of `../docs/hierarchy-edges.txt`, asserting the
  documented parent is **present in** the child's `~superClasses` and never
  that it is the whole answer.
* `methods/` -- one per (class, arm) of `../docs/class-methods.txt`, printing
  one line per row so that a single run answers the whole documented set for
  that class and arm.

**Every row's oracle side is checked for having answered at all, before any
verdict exists.** Two interpreters that fail identically agree on all three
descriptors, so a row naming a class this build does not ship would otherwise
read `agree` and count as satisfied. What the check is per directory:

* `classes/` -- the derived probe text, **gated on the row's `entry` column**:
  a `class` entry answers every question the probe asks, an `instance` entry
  only the opening ones. On top of that, the oracle has to resolve the name to
  an entry at all, which a line count cannot decide: an unresolved environment
  symbol evaluates to its own name as a string and answers those opening
  questions too.
* `hierarchy/` and `methods/` -- the derived probe text; a method group's
  instance arm may answer every line or none, since its `~new` either
  constructs or raises.
* `concepts/` -- a committed line count per section, because these probes are
  hand-written and nothing derives them.
* `directives/` -- **one line for most rows and none for the rows the oracle
  refuses**, which are named in `ORACLE_REFUSES` in
  `crates/rexx-exec/tests/gate_table_d.rs` and policed in both directions.
  Each of those probes asks for output before its directive -- asserted, not
  assumed -- and the oracle prints none of it, so what those rows are required
  to answer is the report it writes on `stderr` instead. `gate_table_d.rs`'s
  module doc carries what was measured about a `stdout` bound for them, and
  why the answer is not the same for all of them.

A row that fails its check keeps its place in the table and is reported as
`unanswered`. It is never `agree`, and it still counts against the gate.

## `native-entries/`

Not a gate table's probes: one program per family of `LIBRARY REXX` entry
point, run by `crates/rexx-exec/tests/native_entries.rs`. Each binds one entry
point this phase does not implement and sends to it, so what it pins is that
the bind succeeded and the *send* refused, naming the entry point and the
phase that owes it a body.

They live here rather than under `../lang/` for this directory's own reason:
the oracle answers every one of them, because it has the body this crate has
not built, so none of them can ever be a `phase-*.txt` row. `native_entries.rs`
asserts that -- it reads every subset file and fails on a line naming this
directory.

**Those last three are derived, text and all.** `gate_table_c.rs`'s
`class_probe_text`, `edge_probe_text` and `method_probe_text` are the
definition of what the programs contain, and every run compares the committed
file against the re-derivation in both directions. Editing one of them by hand
reddens the table; the way to change one is to change the derivation. A
path-only check -- table D's property -- cannot see a probe that asks about
the wrong subject, and a probe asking the wrong question agrees with the
oracle for the wrong reason.

`concepts/classmeth.rex` was `../lang/primitive_classes.rex` until it became
this table's probe for the `classmeth` section. It is still a parse fixture of
`crates/rexx-parse/src/instruction/tests.rs`, which is what it was reached as
before; what it is no longer is a file under `corpus/lang/` that no
`phase-*.txt` names.

## What still applies here, and what does not

**The determinism rule applies.** A program here must produce byte-identical
output on every run of the same interpreter, exactly as `../README.md`
requires -- `rexx-diff`'s self-test (`--cpp X --rs X`) walks `corpus/`
recursively and reads every `.rex` under it, including these, so a
non-deterministic probe would break it. Measured with this subtree in place,
2026-08-25: 575 programs, 0 divergences, exit 0.

**Agreement between the two interpreters does not apply**, which is the whole
difference. `rexx-diff --cpp <c++> --rs <rust>` over `corpus/` reports these
probes' divergences, and that is the table's subject rather than a defect in
the corpus.

The harnesses that read a phase subset file are unaffected: every copy of
`phase_subset_files_on_disk` read `corpus/` with a non-recursive `read_dir`
filtered to `phase-*.txt`, and `corpus.rs`'s
`the_differential_reads_every_phase_subset_file` asserts that set in both
directions.
