# Phase 8 L2 slice — residual fix round after the re-review

Read this first. It is your requirements.

## Where this fits

The final-review fix round (F1-F11, `e64202ae7..cf92ff4fb`) was re-reviewed in two slices. Gates at
`cf92ff4fb` are green apart from the long-standing failing set. The re-reviews found one defect the
round introduced, one pre-existing hole in the boundary the round's own premise relied on, and a set
of smaller items. You fix the ones ruled load-bearing below. Documentation under `docs/` is still a
separate pass after you; do not edit `docs/`.

Worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`, HEAD `cf92ff4fb`
when you start (check). Read `rust/CLAUDE.md`.

Inputs, read the named findings in full:
* `.superpowers/sdd/2026-09-14-phase-8/fix-rereview-boundary.md` (finding 1 and minors 3, 4)
* `.superpowers/sdd/2026-09-14-phase-8/fix-rereview-integration.md` (R1-R10)
* The previous round's brief and report, for the conventions and rulings you inherit:
  `final-fix-brief.md`, `final-fix-report.md`, and `progress.md` from "Fix dispatch pre-flight" on.
* Probes, all under `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/`:
  `final-fix/` (previous implementer), `rereview-int/src/<name>` and `rereview-int/p/<name>`
  (integration re-review: `f4b`, `f4a3`, `f4e`, `f5-t2`, `f5-t3`, `pk-retry`, ...), `boundary/`
  (boundary re-review: the safe-code probe and the thread-callback witness). Copy, don't modify.

## The fixes, in order, one commit each

Same discipline as the last round: reproduce on both sides first, witness, **prediction written
before each control**, fix, confirm.

**X1 — The public interface tables let safe code call a callback with a forged context.**
`ffi::METHOD_CONTEXT` is a `pub static` whose slots are safe `extern "C" fn`, so a
`#![forbid(unsafe_code)]` program can call a slot with a bare `RexxMethodContext_` or a forged
`Owned` and reach `owner_of`'s out-of-provenance read (boundary re-review finding 1: stable abort,
Miri UB). Ruling: **the slot types become `unsafe extern "C" fn`** across the `interface!` tables in
`layout.rs`, which is what calling a C function pointer that takes raw pointers is, and closes the
class for every table rather than one static. The ABI is unchanged; confirm `tests/layout.rs`'s
offset and size tests still pass and slice A's `offset_of`/`sizeof` comparison still matches
(`scratchpad/final-a/offsets.cpp` and its Rust twin). Any Rust test that calls a slot directly
moves into `ffi.rs`'s or `load.rs`'s unit tests, since `unsafe` may not appear in `tests/`. Correct
the `SAFETY:` note at `ffi.rs:247-249`. Witness: the re-review's safe probe no longer compiles
(a `compile_fail` doctest, checked to fail for the right reason on stable by removing what it
guards). If making every slot `unsafe` is blocked by something you find, stop and ask.

**X2 — R1 and R2: a routine's shared code is one object per library entry, found caselessly.**
F4's `LibraryCodeKey` keys routines by exact spelling; the oracle builds one `RoutineClass` per
table entry at load (`LibraryPackage.cpp:275-293`) and `resolveRoutine` answers it caselessly
(`:420-429`). And a second `::ROUTINE ... EXTERNAL` binder's routine is that same shared object, so
it reports the first binder's package (`DirectiveParser.cpp:2684-2693`, the discarded
`setPackageObject` answer), which the crate reports as the second binder's own. Methods stay keyed
per spelling, as measured. Correct the F4 comments that state routines behave like methods.
Witnesses: the re-review's `f4b` (spellings) and `f4a3` (two binders) shapes, rxmath, last path
component only.

**X3 — R3: bind as each directive is translated.** F4 binds a package's procedures only once every
directive resolved; the oracle sets the package in `createNativeMethod` while translating that
directive (`DirectiveParser.cpp:1381-1388`), so a later directive's failure does not undo it.
Witness: the re-review's `f4e` shape (trapped `loadPackage` of a package whose second directive
names a missing entry).

**X4 — R4: investigate first, then fix or park.** A retried `loadPackage` of a package whose
library failed answers it without its classes (oracle: classes installed on the retry). Predates the
round; F5's held version-refused library makes it newly reachable and F4 then names the wrong
package. Trace where the crate keeps the package on the failed first load. **If the fix is local to
the package cache (a failed load not kept, or kept and re-installed on the retry as the oracle's
control `pk-retry` shows it does for a failed base class), make it**, with the re-review's `f5-t3`
and `pk-retry` shapes as witnesses (`f5-t2` needs a forged extension, so it goes in scratch
transcripts, not the corpus). **If it is not local, do not fix it**: write the mechanism, the
sites, and a size estimate in the report, and stop there for X4.

**X5 — Small instruments and false statements in code.**
* R5: `rust/corpus/lang/library_search_path_fixed.env`'s `LD_LIBRARY_PATH` line is inert; remove it
  (and correct the file's comment). Then check whether `tests/support/sidecar.rs`'s environment part
  can be split per variable cheaply; if yes, do it and show it catches the removed line when
  reinstated; if not, say why.
* R10: `ir_recorded`'s assertions are satisfied by a 98.903 before the first clause. Give it an
  assertion that a program reaching the library path did not stop at a load failure it was not
  written to witness (the two `zorkolib` witnesses are the ones that are). Show removing the
  sidecar from `compare` now reddens it. `collect_stress`'s L0 test stays blocked by its
  pre-existing panic: leave it and say so.
* R7: `Interp::executable_package`'s doc (`environment.rs`) is false since F4; correct it to what
  the code does, and say in the report what `.Package~new(..., <unbound loadExternalMethod object>)`
  answers on the oracle, measured (the re-review did not run the oracle side).
* R8: `library_opens` counts asks that reach `load::open`; rename or re-document so the word is
  right.
* Boundary minor 4: `invoke.rs:44-49` `# Errors` still describes the pre-F9 rule.
* Boundary minor 3: a unit test inside `rexx-api` that reaches the five thread-table callbacks
  (`ffi.rs:266-293` at `cf92ff4fb`) so Miri has something to run there; the re-review's scratch
  witness is the shape. Run it under Miri (the previous report says how the scratch `RUSTUP_HOME`
  install was done) with a one-line mutant that Stacked Borrows catches, prediction first.

## Parked, not yours (recorded by the controller)

* R9, a user-defined `REQUEST` method never sent by any string conversion (predates, shared by every
  built-in conversion): KNOWN GAPS, owner to be named in the docs pass.
* R6, two wrong counts in F1's and F4's commit messages: commits are not amended; the correction is
  recorded in the ledger.
* The boundary re-review's minor 2 (Tree Borrows passes the unfixed tree): a report correction.

## Global constraints (binding, unchanged from the last round)

* `unsafe` only in `rust/crates/rexx-api/src/ffi.rs` and `src/load.rs`, each block with a
  `SAFETY:` note naming the invariant and who establishes it; none in `tests/`.
  `rust/crates/rexx-core/tests/unsafe_sites.rs` is the record.
* No new dependency beyond `libloading` 0.8.9. No edit under `api/`. The C++ tree, `build/`,
  `samples/`, `ootest/`, `oodocs/`, `testbinaries/` are read-only; no extension under `build/` is
  rebuilt. Forged extensions only in scratch, never in the corpus.
* No process-global state.
* Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty directory; three descriptors as three files, never `2>&1`.
* `rust/corpus/oracle-crashes.txt` first; never its entries. Never `NUMERIC DIGITS` above 1000,
  never `.Package~new` on a repository file, never `::OPTIONS TRACE ?<letter>`.
* No `cd X && ...; rest` building paths from variables. `/bin/grep -a` for counts.
* Comments minimal (`rust/CLAUDE.md`); never a set's size; no em-dashes; never drop a comment you
  did not make false; every citation you add printed and landing.
* Witnesses follow the Phase 8 corpus conventions; sourceline companions live in
  `rust/crates/rexx-parse/tests/sourceline_oracle/`, regenerated from scratch copies.

## How to work

* Scratch `scratchpad/residual-fix/`, one fresh directory per probe, no `rm` with a glob.
* `-j 4`; `rustfmt <path>`; per commit: `cargo fmt --all --check`, `cargo clippy -j 4 --workspace
  --all-targets -- -D warnings`, the touched crates' tests, `REXX_CORPUS_GATE=1 cargo test --release
  -p rexx-exec --test corpus`. Not the debug G4 gate.
* Stage explicit paths; never `git add -A`, amend, `reset --hard`, `checkout --` on an edited file,
  or bare `stash`. Messages via `git commit -F`, normal prose, ending with:

  ```
  Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_0157HT3JqnRRbgiMDb84iGSD
  ```
* Counts in commit messages are derived from a saved run you can point to, or omitted.
* No subagents. **Ask before implementing if anything is ambiguous or contradicts the tree.**

## Report

`.superpowers/sdd/2026-09-14-phase-8/residual-fix-report.md`, written first, a heading per fix,
appended as you go: reproduction, witness, prediction and result, commit. End with "Not done".
Message the controller when finished with status, commits, one-line test summary, concerns.
