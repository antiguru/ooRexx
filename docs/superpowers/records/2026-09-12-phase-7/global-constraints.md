# Phase 7 — global constraints (draft; the plan's Global Constraints section is copied from this)

* **No `unsafe`.** The workspace lint is `deny`; a per-site exception is Moritz's alone to grant and
  has not been granted for this phase.
* **No process-global state is mutated** (spec A2): no `std::env::set_var`/`remove_var`, no
  `std::env::set_current_dir`, no write to file descriptor 0, 1 or 2 from interpreter code. The
  interpreter's shadow environment and shadow directory are the only ones Phase 7 code reads after
  `run_program` starts, and a test asserts the crate's sources name none of those three functions.
* **No new dependency** unless the spec names it.
* **Correctness is byte-identical stdout, stderr and exit status against the C++ oracle**, read on
  **three separate descriptors, never `2>&1`**. Oracle runs use the standard wrapper
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty directory.
* **Probe hazards.** Read `rust/corpus/oracle-crashes.txt` before an unusual probe and never run its
  entries. Never `select; when 1 = 0 then; when 2 = 2 then nop; end`. Never `NUMERIC DIGITS` above
  1000. Never `.Package~new` on a file inside the repository. A symbol `x` or `b` directly followed
  by a quote is a hex or binary literal. Put `timeout` on both sides of any probe that loops or reads
  input.
* **File-system and command probes and witnesses stay inside their own directory.** Commands are
  limited to `echo`, `printf`, `true`, `false`, `sh -c 'exit N'`, `cat`, `ls`, `pwd`, `env`.
* **Four gates, all green at the closing commit:** `cargo fmt --all --check`;
  `cargo clippy -j 4 --workspace --all-targets -- -D warnings`;
  `cargo test -j 4 --release --workspace --no-fail-fast -- --test-threads=8`;
  `REXX_CORPUS_GATE=1 cargo test -j 4 --workspace --no-fail-fast -- --test-threads=8`.
  Record each exit status unpiped and the count of `ok` result lines. **Commit before a long gate run
  and leave the tree frozen until it finishes.** Wait on a running gate in ~1-hour stretches, not
  10-minute polls.
* **Comments are minimal** (`rust/CLAUDE.md`, Moritz 2026-08-27): a one-sentence overview,
  parameters/returns/panics, and properties the implementation does not show. No narrative, no
  history, no counts of in-repo sets. Measurements and reasoning go in the spec or a record.
* **Every extent claim is derived and its command committed.** No "all", "none", "every" or a count
  in prose without the enumeration that produced it.
* **Write each negative control's prediction before running it**, and mark each part confirmed,
  falsified, or unobservable.
* **A method that exists and does nothing is not implemented.** Build the body or leave the name
  refusing loudly; never a shell that answers nothing.
* **A refusal names the phase that owes the work**: `Phase 8` for native libraries, `Phase 10` for
  RXAPI and the RexxUtil remainder. After this phase, no refusal names `Phase 7`.
* Never `git add -A`; stage explicit paths. Never amend; never force-push; never
  `git reset --hard`; never `git checkout -- <path>` on an edited file; never bare `git stash` /
  `git stash pop`. Commit messages are written to a file and passed with `-F`.
* Scratch files live in the session scratchpad, never in the repository.
