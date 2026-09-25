# Scoped re-review: Surface Task 3, fix round 1

You are re-reviewing a fix round, not the whole task. The question you answer is:
for each finding below, was it actually fixed, and did the fix introduce anything
new that is wrong?

## Inputs

- Diff (2 commits, `034c1c7d7..6c96144d8`):
  `.superpowers/sdd/2026-09-14-phase-8-surface/review-034c1c7d7..6c96144d8.diff`
- Implementer's report, section "Fix round 1":
  `.superpowers/sdd/2026-09-14-phase-8-surface/task-3-report.md`
- Task brief: `.superpowers/sdd/2026-09-14-phase-8-surface/task-3-brief.md`
- Plan: `docs/superpowers/plans/2026-09-14-phase-8-surface.md` (Task 3 only)

## The findings this round was dispatched to fix

1. **Critical (both slices): fresh allocations in the native call path are
   unrooted.** A value built while answering a native API call could be collected
   before it is handed back.
2. **Important: the native call frame was rebuilt per call** where the oracle
   reuses it.
3. **Important, both slices: `logical_t`'s `found` is the wrong object.** The
   oracle reports, in the 88.900 condition, the *converted string*, not the
   object that was tested. Ruling: match the oracle.
4. **Important (host): `found` never sends `OBJECTNAME`/`DEFAULTNAME`.** The
   oracle's `stringValue` protocol is what produces the reported name. Ruling:
   match the oracle, after listing and measuring every caller of any shared
   helper you touch, so the change does not silently alter an unrelated site.
5. **Important (rows): `pointer_string` lacks glibc's `(nil)`.** `%p` prints
   `(nil)` for a null pointer on glibc, and the reverse conversion must read it.
6. Minors, listed in the fix-round dispatch and repeated in the report.
7. A `PROCEDURE`/`DEFAULTNAME` trap divergence, to be recorded in the exclusions
   rather than fixed.

## What to check, in order

For each of 1-7: read what the diff actually does, then decide whether the
finding is closed, partly closed, or still open. Say which, per finding.

Then look specifically for these, which is where this round is most likely to
have gone wrong:

- **Rooting (finding 1).** The round roots three things: a condition object's
  entries, `resolve_stream`'s name, and a `CALL ON` handler's condition object.
  Are those the only unrooted values on the paths the round touched? Walk the
  native call path and the condition-building path yourself and look for any
  other value held live across an allocation. A value that is reachable from a
  rooted object is fine; one that lives only in a Rust local is not.
- **Finding 4's guard.** The implementer did NOT change the shared
  `string_value_text`; it added a separate `Interp::native_found` that sends
  `OBJECTNAME` for an instance but leaves a buffer's or pointer's own string
  value alone. Check: (a) is the split faithful to the oracle, or does it just
  special-case the two cases that were measured? (b) does the new path duplicate
  logic that the shared helper already had, and can they now drift?
- **Finding 3 against finding 4.** The report says a logical's `found` is the
  converted string, and that the conversion itself runs `DEFAULTNAME`. Check
  that the two changes compose: no case where both run and the name is produced
  twice, or where neither runs.
- **Finding 5's acceptance rule.** `(nil)` is accepted case-insensitively after
  a whitespace run, with no sign, and not after an inner `0x`/`0X` prefix.
  Check the rule against the measured `sscanf` results the report quotes, and
  look for a form the rule accepts that glibc rejects (a false accept is worse
  than a false reject here).
- **The witnesses.** For every behavioural change, is there a test that fails
  without it? Check that the corpus rows added actually exercise the new path
  and are not satisfied by an earlier refusal. The report flags that the
  MutableBuffer and VariableReference 88.914 rows are witnessed only on the
  refusal side, because their success side is a loud refusal today: say whether
  that is acceptable or whether a different witness was available.
- **Prose.** Fix rounds in this project reliably introduce false statements in
  comments and records while the behaviour is right. Check every comment and
  record line the diff touches against what the code now does, including lines
  the diff did not change but whose subject it did. Two specific rules: a
  comment may never state the size of a set, and a claim of the form "every X"
  or "the only X" must be derivable by running something.

## One claim to check independently

The implementer reports that `gate_table_c` fails under `REXX_CORPUS_GATE=1`
with 82 `unanswered` closed-phase rows, and argues it is pre-existing because a
pristine tree **at `2d155158c`** reproduces it. `2d155158c` is this round's own
first commit, so that argument does not establish "pre-existing". Determine
whether the failure predates Task 3: the commit before Task 3's first commit is
`e0b30d156`. Build a pristine tree at that commit in your own directory with its
own `CARGO_TARGET_DIR` (see constraints below) and run the same test under
`REXX_CORPUS_GATE=1`. Report the row count you observe there. If it does not
reproduce at `e0b30d156`, that is a finding against this round or against Task 3.

## Global constraints that bind this diff

- `unsafe` is permitted only in `rust/crates/rexx-api/src/ffi.rs` and
  `rust/crates/rexx-api/src/load.rs`, each block carrying a `SAFETY:` note.
  `rust/crates/rexx-core/tests/unsafe_sites.rs` is the file-granular record. No
  `unsafe` in `tests/`.
- No process-global state: no `std::env::set_var`/`remove_var`, no
  `set_current_dir`, no writes to fd 0/1/2 from interpreter code.
- Comments are minimal: a one-sentence overview plus parameters, returns, panics
  and non-obvious properties only. No em-dashes. A comment may never state the
  size of a set.
- The C++ tree at the repo root and `samples/`, `build/`, `ootest/`, `oodocs/`,
  `testbinaries/` are read-only. `api/` headers are frozen.

## How to run things

- Oracle: from a **fresh empty directory**,
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  Compare stdout, stderr and exit status on three separate descriptors, never
  `2>&1`. Read `rust/corpus/oracle-crashes.txt` before running anything and
  never run its entries.
- Never register anything with the rxapi daemon. Never load a prebuilt extension
  that needs `librexx.so` or `librexxapi.so`.
- Scratch goes in this session's scratchpad directory, never in the repository.
  Use your own `CARGO_TARGET_DIR` under it, and **delete it before you report**:
  `/tmp` is a shared 63 GB tmpfs and cargo target directories have filled it.
  Check `df -h /tmp` before you start.
- The worktree is yours to read, not to write. Do not edit, commit or revert
  anything. Report findings; the fix round is dispatched separately.

## Report

Write your full report to
`.superpowers/sdd/2026-09-14-phase-8-surface/task-3-fix1-rereview.md` and return
only: a verdict line (APPROVED / CHANGES REQUESTED), the per-finding
closed/partly/open list, the `gate_table_c` row count at `e0b30d156`, and any new
findings as one line each with a file:line. Do not paste diffs or code blocks
into your returned message.
