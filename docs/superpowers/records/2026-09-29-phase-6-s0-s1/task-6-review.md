# Task 6 review: `ObjRef` `!Send`/`!Sync`, bounded countdown reload, S0 gate

Base 97f564413, head b9f85325e. Verdict: **spec-compliant**; one factual defect in the perf
record (below), no code or behaviour defect.

## Spec Compliance

✅ Step 1 -- `ObjRef` is `!Send`/`!Sync` via `PhantomData<*const ()>` (handle.rs:72), matching
spec section 5 ("Isolation at the type level"). `compile_fail` doctests plus a positive control,
per Ruling P5.
✅ Step 2 -- the no-deadline reload is `Deadline::CLAUSES_PER_CHECK` (1024) instead of
`u32::MAX`, so the cold path runs on the same 1024-clause cadence with or without a deadline, per
spec section 4. The deadline-set reload is unchanged (still 1024 on the not-yet-expired arm).
✅ Step 3 -- S0 gate measured and recorded, with one flagged, unresolved wall-clock excursion
(`dispatch`) reported for a ruling rather than silently absorbed, per the section 7 stopping rule.
✅ Step 4 -- gates G1-G8 green at b9f85325e, tree clean, HEAD unchanged.
✅ Global constraints -- no new dependency, no `unsafe`, behaviour unchanged (corpus 655/655,
`deadline` 10/10, both re-run independently below).

## Strengths

- Every figure I checked was reproducible from the raw data still sitting in the task's scratch
  directory: the three `sha256`/`.text` builds, all four wall-clock control tables (`wall`,
  `wall2`, `wallA`, `wallctl` -- medians recomputed by hand from `wall.tsv` match the report to
  three decimal places), the `dispatch` Ir deltas from the raw `.row` files, and the gate log
  (`status.txt`, summed test counts) match verbatim.
- The `compile_fail` doctest reason was independently re-derived: extracted the probe, compiled it
  against the built rlib, got E0277 pointing at `handle.rs:72`'s `PhantomData` field, matching the
  report exactly.
- Re-ran `cargo test -p rexx-exec --test corpus --release` under `REXX_CORPUS_GATE=1` (655 of 655)
  and `cargo test -p rexx-exec --test deadline` (10/10) myself; both match the report.
- The removed `execute()` reload line is genuinely redundant: `Interp::new()` (called at the top of
  `execute`, lib.rs:2833) already sets `clause_countdown` to `CLAUSES_PER_CHECK`, and no clause runs
  between that call and the deadline assignment, so the deleted `if interp.deadline.is_some() {
  ... }` block was setting a value already in place, exactly as claimed.
- `dispatch`'s wall-clock excursion is handled correctly: flagged rather than chased, with three
  controls (independent re-run, byte-identical-binary control, Step-1-only build with a *negative*
  Ir delta) that rule out added work, consistent with the section 7 stopping rule and Task 2's own
  padding-control precedent. This became Ruling P15 (progress.md), which is the right order --
  Task 6 reported the finding, the ruling followed.
- No stray reader of the deleted `NO_DEADLINE_SPACING` constant; grepped the whole workspace.

## Issues

### Important

- **`docs/superpowers/plans/phase-6-perf.md:385`: stale noise-band figures for `sayloop` and
  `startup`, contradicting the table 300 lines above them in the same file.** The sentence reads
  "at most +0.3% beyond each program's noise band (recorded above; 0 for every program except
  `heapshape` 0.00000%, `sayloop` 0.0030%, `startup` 0.0045%, `rexxcps` 0.00006%)". The table
  actually "recorded above" (`phase-6-perf.md:64-89`, the pad1/pad2/pad3-vs-base table at the
  current Phase 6 base `1754a3b5a`) shows `sayloop` and `startup` at `0 (0.00000%)`, same as every
  other program except `heapshape` (100, 0.00000%) and `rexxcps` (11,030, 0.00006%) -- which *do*
  match the cited sentence. The 0.0030%/0.0045% figures are Task 2's *original* report prose
  (`task-2-report.md:78`), measured against the old base `9d863ccc5` before Ruling P8 moved the
  base to `1754a3b5a` and the table was fully re-measured (`833847a9b`, "Re-measure the Phase 6
  base at 1754a3b5a"); no trace of 0.0030%/0.0045% exists elsewhere in the current file. This is
  the "citing my own records from memory" failure mode this project already tracks: the record
  actually holds the true (zero) figures two tables up, and the prose two lines below states
  different, superseded ones.
  **Why it matters anyway (it doesn't change this gate):** the S0 Ir budget is noise-band + 0.3%,
  and 0.3% swamps a 0.0030%/0.0045% difference either way, so S0's pass verdict is unaffected. It
  matters because this file is the record later stages (S1-S4) read for "the noise band", and per
  this project's own comment rules a false statement here is fixed, not reworded or left standing.
  **Fix:** replace the parenthetical with the real exception list from the table two tables up:
  `heapshape` (100, 0.00000%) and `rexxcps` (11,030, 0.00006%) only; `sayloop` and `startup` belong
  in the "0" bucket at the current base.

## Assessment

**Task quality: Needs fixes** -- one-line correction to `phase-6-perf.md`'s S0-budget sentence
(Important, above). No code change is needed: `handle.rs`, `clause.rs` and `lib.rs` are correct,
gates are green, corpus and deadline behaviour are unchanged (independently re-run), and the
`dispatch` wall-clock excursion is handled exactly as the design's stopping rule asks.
