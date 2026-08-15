# Task 1 report: correct the record before anything reads it

**Status: DONE**

**Commit:** `64ee0369` (`plan/rust-rewrite`) -- the three tracked-file corrections and the
master-plan amendments.
`.superpowers/` SDD artifacts are gitignored (`.gitignore:19`), so the two corrections there
(`rexxcps-measurement.md`, `task-6-report.md`) are on disk but not part of any commit.

## Step 1: reproduction

From a fresh scratch directory with `say 1` in `t.rex`, wrapping both interpreters at
`ulimit -v` 100000 / 400000 / 600000:

```
oracle 100000 -> 0
rust   100000 -> 101
oracle 400000 -> 0
rust   400000 -> 101
oracle 600000 -> 0
rust   600000 -> 0
```

Matches the brief's expected values exactly. Confirmed the constant's ownership:
`/bin/grep -rn INTERPRETER_STACK_BYTES` finds it only at `rust/crates/rexx-exec/src/lib.rs:309`
(plus derived binary/build artifacts); zero hits anywhere under `/home/moritz/dev/repos/ooRexx/`
(grep exit 1). So **this crate** reserves the 512 MiB, not the oracle -- the claim in
`perf-baseline.md` and its followers was backwards.

## Step 2: corrected every occurrence found

Searched `/bin/grep -arn "512 MiB\|INTERPRETER_STACK_BYTES\|512 \* 1024" docs/ .superpowers/`
(135 hits) and read each one's surrounding paragraph, not just the matched line, to tell
correctly-attributed mentions ("this crate reserves...", "D19's 512 MiB thread-stack
reservation", "oracle has about 1000 MiB... this crate has about 500") from the reversed ones.

Fixed, in place, aligned to `rust/CLAUDE.md`'s wording ("this crate has ~500 MiB to allocate in
where the oracle has ~1000"):

* `docs/superpowers/plans/perf-baseline.md:260-264` -- the origin. Was "the oracle reserves 512
  MiB... and `rexx-run` reserves none". Now: this crate reserves it and the oracle reserves
  nothing comparable, with the ~500 MB / ~1000 MB split spelled out. Kept the paragraph; only
  the subject and the follow-on figure changed.
* `docs/superpowers/plans/phase-4-exclusions.txt:2189-2195` -- the `TRUNC`/value-magnitude
  KNOWN GAP row said "the oracle reserves 512 MiB for its interpreter stack, so the same
  `ulimit -v` leaves this crate roughly half the room". The conclusion (this crate gets half
  the room) was already correct; only the causal attribution was backwards. Corrected the
  subject and re-wrapped the paragraph to the file's ~78-column width.
* `.superpowers/sdd/2026-08-04-phase-4c-builtins-and-parse/rexxcps-measurement.md:83-87` --
  named in the brief. Corrected the header and the paragraph in place (not a stranded note),
  with an inline "correction, 2026-08-08" flag since this is a historical SDD artifact.
* `.superpowers/sdd/2026-08-04-phase-4c-builtins-and-parse/task-6-report.md:586-589` -- **not**
  in the brief's named list, but the same reversed sentence ("the oracle reserves 512 MiB for
  its interpreter stack") appears here too (§6, D3). Caught by the broad Step 4 grep across
  `.superpowers/`, so corrected it the same way, with the same inline flag.

**Checked and left alone, no reversal present despite matching the grep:**
`task-5-report.md` (the two names file lines, :391 and :470, are attribution-neutral -- they
say "the 512 MiB `INTERPRETER_STACK_BYTES` reservation" without naming which side holds it, so
they were never actually wrong, unlike what `phase-4d-performance-design.md:153` implies by
grouping it with the other three); `task-5-review.md:227-233`, `task-3-report.md:930-935`,
`task-13-brief.md:154`, `task-13-report.md:408-410`, `task-3-rereview.md:144-148`,
`memory-gap-investigation.md:83-127`, `progress.md:451-456` (4c dir) -- all already
correctly attribute the reservation to this crate.
`docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md:1692` is a scope note that
never names a subject and needed no change.

**Deliberately left unedited:** two `.diff` files
(`.superpowers/sdd/2026-08-04-phase-4c-builtins-and-parse/review-d6e8a8c7..9c3d96df.diff:24` and
`review-bf272e63..88b3701f.diff`) carry the reversed sentence inside a historical code-review
diff. These are records of what a past review actually said, not living documentation;
editing them would misrepresent history rather than correct it. Not in the brief's file list.

## Step 3: master plan amendments (`2026-07-27-rust-rewrite.md`)

* `:442` (roadmap row) -- `samples/rexxcps.rex` runs clean and within the ratio bar" now has
  "the ratio bar" parenthetically defined as "the Global Constraints `:39` parity shipping
  gate", with an explicit "**not** `:40`'s 1.5x figure, which scopes to Phase 1" -- matching the
  design spec's own account of why an earlier draft's 6.7x-vs-10.0x confusion happened.
* `:473` -- "Phase 4 is split into three sub-phases" -> "four", 4d's role stated (measures
  against the `:39` parity gate, attributes the gap, closes it or records a debt), and the
  closing sentence changed from "closes when 4c closes" to "closes when 4d closes". Added a
  citation to `docs/superpowers/specs/2026-08-08-phase-4d-performance-design.md` alongside the
  existing 4a design-spec citation.
* Roadmap row `:442`'s Phase column: "split 4a / 4b / 4c" -> "split 4a / 4b / 4c / 4d", for
  consistency with the prose change at `:473`.
* Checked neighbours for staleness: `:108` and `:381-390` (D9, the performance-gate decision
  block) don't name a sub-phase and needed no change; `:459` (S0's entry condition, "4c")
  is unaffected -- the BIF audit work becomes possible the moment 4c closes regardless of when
  4d closes, and the file's own text already argues that explicitly at `:462`.

## Step 4: verification

`/bin/grep -arn "512 MiB\|INTERPRETER_STACK_BYTES\|512 \* 1024" docs/ .superpowers/` after all
edits: the only remaining "oracle reserves 512 MiB" text is inside the two historical `.diff`
files noted above. Every live document now attributes the reservation to this crate.
Re-ran `/bin/grep -arn "oracle reserves\|oracle has no equivalent\|the rust binary has no
equivalent"` as a second, narrower check for the exact reversed phrasing: same result.

`git status --short` shows the three tracked files modified and committed; `git diff` on each
was read in full before committing. Tree is otherwise clean.

## Concerns

* `phase-4d-performance-design.md:153` characterizes `task-5-report.md` as one of "four
  documents" carrying the reversed claim. On inspection it doesn't -- its two `INTERPRETER_STACK_BYTES`
  mentions never name a subject. This isn't a new error in that spec (it's a loose paraphrase,
  not a quote), but a later reader taking it as a literal file list would expect an edit that
  doesn't exist here. Flagging rather than fixing the design spec, since that file is out of
  this task's stated file list and amending specs wasn't part of the brief.
* The two `.diff` files with the old (wrong) wording still exist as-is, by design (historical
  record). If a future grep-based check treats "any occurrence" as failing, it will need an
  exclusion for `*.diff` under `.superpowers/`, since those two are permanent.
* No code was touched. No `.rs` file was read for editing purposes beyond the read-only
  `lib.rs:309` lookup used to verify the constant's location.
