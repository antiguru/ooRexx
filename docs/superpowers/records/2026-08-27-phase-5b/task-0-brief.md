## Task 0: `corpus/phase-5b.txt` and its wiring

**Goal.** The file four later tasks write into exists and is read everywhere a phase subset file is
read.

**Why it is a task.** 5a had the same one and said so in the file's own header: its Task 1 "built the
harness plumbing this file needs ... and this file wired into every harness that reads a phase subset
list, and added no corpus program of its own." 5b's plan had no such task, so whichever of Tasks 2,
3, 5 or 7 landed first would have inherited four red tests and one silent narrowing.

**Build.** `corpus/phase-5b.txt` in `phase-5a.txt`'s shape and format, read by the same `read_subset`,
plus its wiring into every reader:

* `SUBSET_FILES` in `crates/rexx-exec/tests/corpus.rs`, `coverage.rs`, `collect_stress.rs` and
  `ir_dual.rs`. **These four are guarded** against a directory listing of `corpus/phase-*.txt`, so
  each goes red on the commit that creates the file and green when its literal is updated.
* **The unguarded literal in `crates/rexx-exec/tests/trace_oracle.rs`**, whose own doc comment says
  it is "the one call site those four do not cover, and a phase subset file added and forgotten
  *here* would silently keep this check measuring the union as it stood before."
* **`EXPECTED_SUBSET_5B` in `coverage.rs`**, beside `EXPECTED_SUBSET_5A`, with the test that pins the
  file's entries against it. Nothing forces this one to exist either.

Two of the six are silent, which is why they are named individually rather than as "the harnesses".

**Done when** the file exists, all six sites read it, and **the control is recorded as run**: removing
`phase-5b.txt` from any one of the four guarded literals reddens that binary, and removing it from the
`trace_oracle.rs` literal reddens nothing -- which is the property that makes the last one worth
naming.

---

