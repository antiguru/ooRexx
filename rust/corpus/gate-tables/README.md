# Gate-table probe programs

**Not a differential corpus.** Every program here is the probe for one row of
a Phase 5 gate table, and most of them diverge from the C++ oracle today --
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

## What still applies here, and what does not

**The determinism rule applies.** A program here must produce byte-identical
output on every run of the same interpreter, exactly as `../README.md`
requires -- `rexx-diff`'s self-test (`--cpp X --rs X`) walks `corpus/`
recursively and reads every `.rex` under it, including these, so a
non-deterministic probe would break it. Measured with this subtree in place:
204 programs, 0 divergences, exit 0.

**Agreement between the two interpreters does not apply**, which is the whole
difference. `rexx-diff --cpp <c++> --rs <rust>` over `corpus/` reports these
probes' divergences, and that is the table's subject rather than a defect in
the corpus.

The harnesses that read a phase subset file are unaffected: all four copies of
`phase_subset_files_on_disk` read `corpus/` with a non-recursive `read_dir`
filtered to `phase-*.txt`, and `corpus.rs`'s
`the_differential_reads_every_phase_subset_file` asserts that set in both
directions.
