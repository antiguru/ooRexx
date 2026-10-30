### Spec compliance

* ✅ Spec compliant for Task 8.8g, reviewed from `a9bf23ed4e072fe3b02b39942206cd03051df9ad` through `ef81bfc3e` using `task-8.8g-source-review.diff`. The retained Rust addition is the two pinning-feature tests and their shared witness helper in `rust/crates/rexx-exec/src/scheduler/tests/mutants.rs:233`. The helper uses the specified real REPLY, started SORTWITH and ready-worker program, and its RETURN twin changes only the terminal statement. The FIFO configuration, deadline, exact language result, actual contention below the floor, sorting-key counters, total counters and balanced pinning are checked at `mutants.rs:246`, `mutants.rs:258`, `mutants.rs:267` and `mutants.rs:270`.
* ✅ The compiled experiment judges M6 by activity attribution. `docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m6/mutation.diff:8` deletes only the SLICE clear while retaining the comment and deferred-flag reset. `mutant-reply.stderr:4` records exactly one `[Native("SORTWITH"), SortComparator]` deferral, and `mutant-reply.stderr:7` identifies the intended failed assertion after the exact language-result check. `mutant-return.stdout:5` records the independently executed passing control, while `restoration-check.json:5` records an empty baseline-versus-restored production diff and byte equality.
* ✅ Closure preserves the historical evidence and separates all six catches. `.superpowers/sdd/queued/2026-10-07-unjudged-scheduler-mutants.md:19` labels the old judgement as withdrawn, and its correction at line 23 identifies the omitted REPLY producer and links the earlier five catches separately. The inventory hunk changes only disposition columns 4 and 6 at `docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv:65`. The appended corrections at `.superpowers/sdd/2026-10-18-phase-9d/progress.md:363` and `task-8.8-controller-handoff.md:53` retain root ownership of remaining obligations.
* ⚠️ This scoped review does not establish timer-mode observational equivalence, whole-batch gates, performance acceptance, ruling 84 or Tasks 9/10 completion. `task-8.8g-report.md:66` explicitly leaves those obligations with the controller. The source review contains no new production hook, unsafe code, dependency or retained scheduler change. Existing parent performance changes and the user archive were outside this review and were not modified.

### Strengths

* The shared program builder keeps the negative control aligned with the REPLY witness, while separate named tests prevent the primary mutant panic from skipping the control (`rust/crates/rexx-exec/src/scheduler/tests/mutants.rs:234`, `mutants.rs:296`, `mutants.rs:303`). Exact exit status, stdout and stderr are asserted before pinning counters, so a language failure cannot qualify as the catch (`mutants.rs:258`). The per-key and total assertions rule out silently attributing the request to another pin stack (`mutants.rs:276`, `mutants.rs:281`). No duplicated scheduler implementation or fabricated activity state supplies the observation.

### Issues

* Critical: None.
* Important: None.
* Minor: None.

### Assessment

**Task quality:** Approved.

The mutation and separate negative control support the claimed reachable request-attribution invariant, and the inverse restoration preserves production behavior. No high-confidence correctness or invariant finding remains in this task's scope.

### Checked risks and limits

* Evidence provenance and false-positive selection: Read all six raw named-run descriptors and outputs. Statuses are `0/0/101/0/0/0`, every invocation selected exactly one test, every run recorded nine steps and six contended steps, and only mutant REPLY recorded a deferral. Baseline and restored scheduler hashes match, while both mutant descriptors use the distinct mutated hash. The retained final module output reports seven passes at `docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m6/focused-final.stdout:11`; final formatting and pinning-feature all-target clippy descriptors report status zero without warning output.
* Historical-ledger preservation: The review package omitted the progress and controller-handoff content despite including them in its stat, so a focused `rg` inspection checked their M6 entries. Both append the compiled correction and preserve the prior entries rather than treating old measurements as new evidence. The queued-record and inventory hunks were sufficient to verify their disposition changes. No broader repository crawl or test rerun was needed.
* Validation noise: The initial formatting failure is retained and followed by a committed formatting-only correction and successful final check (`task-8.8g-report.md:51`). The expected mutant assertion failure is the intended experiment result, not an unresolved validation failure. The reported trailing whitespace belongs to retained raw output, and the authored-file check is recorded separately (`task-8.8g-report.md:58`). No additional test was run because the existing named experiment answers the concrete attribution risk.
