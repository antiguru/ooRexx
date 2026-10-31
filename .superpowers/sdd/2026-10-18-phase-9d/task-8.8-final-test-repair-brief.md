# Task 8.8 final test repairs

The resumed frozen validation at `6b6f3fb460049c6ef63f565ceb78e11b5582f165` executed all 156 default targets. It retained two failing assertions, with 3,801 tests passing, two failing and five existing ignored tests. Release corpus and debug corpus each matched 1,242 of 1,242 programs, while doctests passed 15 of 15 and pinning passed seven of seven. The release suite separately reproduced the SysFileTree shared-directory count mismatch, which requires its own controlled diagnosis and harness repair.

## Scope and ownership

Root released the source and index freeze after every validation command stopped. The original implementer owns the source and index only after root commits this brief, and root will avoid concurrent tracked writes during implementation. This is a new test-maintenance slice addressing observed gate failures, rather than another runtime review repair or a performance acceptance waiver. Tasks 9 and 10 remain outside scope, and no production runtime edit is authorized.

* Repair the dispatch-seam consumer inventory using the nine moved parameters proven in `/tmp/codex-p9d-t88-dispatch-seam-diagnosis.md`. Add only the three explicit module paths, retain the enumerated boundary and add negative assertions rejecting unlisted sibling paths in the existing test.
* Repair the TraceObject expected passing population using `/tmp/codex-p9d-t88-traceobject-population-diagnosis.md`. Require exactly the ten documented passing names and ten total rows, preserving outcome classification, descriptor masks, mode selection, exclusions and negative controls.
* Preserve comments and all existing test identities. If the affected concurrency file requires an ownership split, first make a pure move of the complete relevant responsibility with unchanged public surface, comments and test paths, and verify its population before changing assertions.
* Preserve all first-failure records. The retained failing default runs supply the red evidence for both existing assertions, so do not rerun them merely to produce another red result.
* Commit source before compiling or verifying. Run foreground bounded checks with `CARGO_INCREMENTAL=0`, Cargo jobs four and the existing `/tmp/codex-p9d-t88-target`; no services or background processes.

## Acceptance

The covering checks are all six dispatch-seam tests, the exact TraceObject outcome-table test with the corpus gate enabled, and its existing altered-test negative control. Formatting and affected-target clippy must pass, and the report must retain exact commands, output, status and test populations. Production Rust sources, Cargo inputs, the immutable measured normal pin and protected ooTest bytes must remain unchanged. Return source commits, evidence paths and a short report for scoped review, with broader validation and performance acceptance explicitly root-owned.

## Independent directory-race investigation

The retained SysFileTree diagnosis is `/tmp/codex-p9d-t88-sysfiletree-final-diagnosis/diagnosis.md`. After freeze release, a read-only diagnostic worker may run the proposed one-create and no-create witnesses on both immutable interpreters in exclusively owned temporary directories. This work must not modify tracked files or coordinate through the source index, and it must retain raw channels, executable hashes and exact name sets. A confirmed mechanism permits a separate minimal group-child temporary-directory isolation implementation, with the protected suite unchanged and no Flaky licence or assertion relaxation.
