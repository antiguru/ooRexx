# Final review of Phase 8's L2 slice — shared constraints for every slice

Repository: /home/moritz/dev/repos/ooRexx-rust-rewrite (a git worktree, branch plan/rust-rewrite).
Range under review: `659312de0..e64202ae7` (Phase 8's survey through its gate document).
Plan: docs/superpowers/plans/2026-09-14-phase-8.md. Spec: docs/superpowers/specs/2026-09-14-phase-8-native-api.md.
Evidence base: docs/superpowers/specs/2026-09-14-phase-8-scoping.md.
Ledger (per-task history, rulings, fix rounds): .superpowers/sdd/2026-09-14-phase-8/progress.md.

## You are read-only

* Sibling reviewers share this worktree. Do not edit, create, delete or format any tracked file.
  Do not run `git` commands that write (commit, add, stash, checkout, reset, restore, clean).
* Write only your own report file, in `.superpowers/sdd/2026-09-14-phase-8/`, and scratch files
  under /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/
  in a directory named after your slice (e.g. `final-a/`), never a generic name.
* Do not dispatch subagents.
* Build with `CARGO_TARGET_DIR=<your scratch dir>/target` if you build at all, so you do not
  contend with siblings or replace binaries they are running. Use `-j 4`.

## Hazards

* The C++ tree at the repo root, `build/`, `samples/`, `ootest/`, `oodocs/`, `api/` are read-only.
  `build/lib/librxregexp.so` must never be rebuilt.
* Oracle runs: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a **fresh empty directory** (a leftover .rex file on the search path gets called as an
  external routine). Compare stdout, stderr and exit status on three separate descriptors, never `2>&1`.
* Read `rust/corpus/oracle-crashes.txt` before any unusual probe and never run its entries. Never
  `NUMERIC DIGITS` above 1000. Never `.Package~new` on a repository file. Never
  `::OPTIONS TRACE ?<letter>`. A symbol `x` or `b` directly followed by a quote is a hex/binary literal.
* Never `cd X && ...; rest` with variables built after it -- a failed cd leaves paths at `/`. Use
  absolute paths or a subshell.
* `grep` here is a ugrep wrapper that silently skips binary and git-ignored files: use `/bin/grep -a`
  for counts, `-F` for data patterns.
* `testOORexx.rex` with no arguments writes fixtures into `ootest/`. Do not run it.

## Project rules a finding may cite

* `unsafe` only in `rust/crates/rexx-api/src/ffi.rs` and `src/load.rs` (D-U1), each block with a
  `SAFETY:` note naming the invariant and who establishes it.
* One new dependency only: `libloading` 0.8.9.
* No process-global state: no `set_var`/`remove_var`, no `set_current_dir`, no writes to fd 0/1/2
  from interpreter code.
* Correctness is byte-identical stdout, stderr and exit status against the C++ ooRexx oracle.
* Comments minimal (rust/CLAUDE.md): one-sentence overview, params/returns/panics, non-obvious
  properties only. A comment never states the size of a set. No em-dashes in comments.
* A number (error code, count, line) is measured or it is not written; every citation must land on
  its subject when that line is printed.

## What this review is for

Every task already had a task review and scoped re-reviews. Do **not** re-review hunks for local
style. Your value is what a per-task review structurally cannot see: cross-task drift, interfaces
between tasks, facts restated in several places where only some were updated, and instruments that
cannot fail. Prefer running something over reasoning about it; a finding you ran is worth several
you inferred.

## Report contract

* **Write the report file first**, with its headings, and append findings as you establish them.
  A session limit can kill you mid-flight and a partial file beats an unwritten one.
* Each finding: severity (Critical / Important / Minor), file:line, what is wrong, the command or
  probe that shows it (with its output), and whether you ran it or inferred it.
* End with a section **"Not reached"** listing what in your slice you did not examine. Silence is
  not coverage.
* When finished, reply with the report path, counts by severity, and one line per Critical or
  Important. Writing the file alone does not reach the controller.
