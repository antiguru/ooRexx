### Task 2: The routine half of the calling protocol, at every load site

`::ROUTINE EXTERNAL` and a `::REQUIRES ... LIBRARY`'s routines install and then refuse loudly at the
call. **And a library's routines become callable only through `::REQUIRES ... LIBRARY`**: measured
by the final review with `rxmath`, `Package~loadLibrary('rxmath')`, `::routine sq external "LIBRARY
rxmath RxCalcSqrt"` and `.Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcSqrt')` each load
the library and then `RxCalcSqrt(16)` answers 43.1 at rc 213 where the oracle prints `4`. The
`Method` and `Routine` objects `loadExternalMethod`/`loadExternalRoutine` answer are shells whose
use refuses naming Phase 5.

**Files:** `rust/crates/rexx-api/src/invoke.rs`, `src/layout.rs`, `rust/crates/rexx-exec/src/lib.rs`,
`src/dispatch.rs`, `src/dispatch/package.rs`

- [ ] **Step 1:** `RexxRoutineEntry` carries a `style`: `ROUTINE_TYPED_STYLE` uses the same two-call
      protocol as a method, `ROUTINE_CLASSIC_STYLE` is the `RXSTRING` convention
      (`api/oorexxapi.h:190-208`). Build the typed style; leave classic refusing and name Phase 10,
      since its consumers are `rxsubcom` and the registered-function API.
- [ ] **Step 2:** Populate `CallContextInterface`, which is what a routine receives.
- [ ] **Step 3:** Register a library's routines at the one resolution path every load site shares,
      not at one caller of it, so every load site (the fix re-review enumerated nine for a first
      ask) makes the routines callable by name from any
      package, including one loaded before the library was. Measure on the oracle which packages
      see a routine a library loaded inside a required package's routine made available.
- [ ] **Step 4:** A `Routine` from `loadExternalRoutine` answers `~call`, and a `Method` from
      `loadExternalMethod` runs when a class defines it.
- [ ] **Step 5:** Delete the refusals, and update the expectations in `run/tests.rs` that assert
      their text.
- [ ] **Step 6:** Witnesses, using `rxmath` (routines) and `rxregexp` (methods): a typed routine
      answering through each load site; the classic style refusing; the two loaded objects used;
      each with `sourceline_oracle` companions and `corpus/phase-8.txt` rows. Commit.

---


---

# Plan context every task inherits (copied from the plan, binding)

## What the L2 slice established, which this half inherits

* **Nothing unwinds across the boundary.** `RaiseException` records a pending condition and returns
  to the extension; entry points stay `extern "C"` and a Rust panic aborts. A native method that
  raises still runs to completion and still returns a value.
* **A panic inside a callback aborts the whole binary** rather than failing one test, so a control
  that can only redden by abort gives a red with no name. Observe what a callback did after the
  call returns.
* **The conversion table is what drives conversion**, and deleting a row must redden exactly that
  row's tests. A second code-keyed match written outside the table is invisible to that control.
* **`values::repr` names the union member for a code.** Use it rather than a new match.
* **`NativeMethodEntry::entry_point` is private**, with `has_entry_point()` the only public reader,
  because an owned copy of that address outlives its `Library`. Add `pub(crate)` accessors, never
  `pub`.
* **`Interp::libraries` exposes only `get` and a non-replacing `hold`**, in its own module so the
  field is private to it.
* **A native argument's string value is `requiredString`'s**, `REQUEST('STRING')` and nothing
  else, not the operator protocol that falls back to the default name. The final review found the
  host using the wrong one, silently, on every string parameter.
* **Blame a directive through the helper that names its package.** Two helpers sit side by side
  in `lib.rs`; the one that names the running program was used where the declaring package was
  owed, and only a two-file witness sees the difference. Every witness for an error a directive or
  a native boundary raises declares the directive in a required package, not in the program that
  runs.
* **Every interface table slot is an `unsafe extern "C" fn`**, and `ffi::value_of` is `unsafe`: a
  safe path from a table to a callback let safe code hand it a forged context. A slot this half
  fills keeps that type, and a Rust test that calls one lives in `ffi.rs` or `load.rs`.
* **A library's shared code follows the oracle's objects.** A method's code is one object per
  spelling asked for; a routine's is one object per routine table entry, found by exact spelling
  and then without regard to case; the first directive that binds either names its package, as that
  directive resolves, and a `loadExternal*` object reads it retroactively. A package whose
  translation raised is not kept and has no routine, method or resource table and no prolog.
  `Libraries` holds a library that loaded, including one refused for its version, which answers
  loaded later with no routines. The final review and its rounds found a silent wrong answer in
  every one of these, so a task touching them probes past its witnesses.
* **A number is measured or it is not written.** Five error numbers taken from
  `interpreter/messages/RexxErrorCodes.h` named paths nothing runs, against none taken from a run.
* **Check every citation by printing that line**, including citations into files this plan itself
  edits: the roadmap line a decision cited moved under the phase's own insertions.

## Global Constraints

* **`unsafe` in `rexx-api` in exactly two files**, `rexx-api/src/ffi.rs` and `src/load.rs`, each
  block with a `SAFETY:` note naming the invariant and who establishes it (D-U1, Moritz 2026-09-14).
  `crates/rexx-core/tests/unsafe_sites.rs` is the record, is file-granular, and scans `tests/`.
* **No new dependency** beyond `libloading`.
* **The headers are frozen.** No edit under `api/`; one that seems necessary reopens D5.
* **The C++ tree, `build/`, `samples/`, `ootest/`, `oodocs/` and `testbinaries/` are read-only.**
  No extension is ever rebuilt: loading the oracle's own compiled extensions is the instrument, per
  the D5 amendment. The corpus differential loads them from the oracle checkout's `build/lib`.
* **Never load a prebuilt extension that NEEDs `librexx.so` or `librexxapi.so` into this crate,
  directly or through another library it NEEDs.** `liborxexits.so`, `liborxclassic.so` and
  `liborxclassic1.so` do directly, and `liborxinvocation.so` does through `liborxexits.so`; `dlopen`
  would map the oracle's own interpreter into this process, and a green result would be the
  oracle's. Before the first load of any library not named in this plan, follow its `readelf -d`
  `NEEDED` entries transitively, or run `ldd` on it.
* **No process-global state is mutated.**
* **Correctness is byte-identical stdout, stderr and exit status against the C++ oracle**, on three
  separate descriptors, never `2>&1`, through the standard wrapper from a fresh empty directory.
* **Probe hazards.** Read `rust/corpus/oracle-crashes.txt` first and never run its entries.
  `testOORexx.rex` with no arguments writes fixture files into `ootest/`; use the single-group form.
* **Comments minimal** (rust/CLAUDE.md, Moritz 2026-08-27), **never stating a set's size**, no
  em-dashes, every citation landing on its subject.
* **Every extent claim is derived and its command committed.**
* **Write each negative control's prediction before running it.**
* **A change to `rexx-api/src/ffi.rs` or `load.rs` runs `rexx-api`'s lib tests under Miri's Stacked
  Borrows** before it commits. Miri is not installable through `rustup component add` here; the
  L2 slice's fix round installed it into a scratch `RUSTUP_HOME` (its report says how). Tree
  Borrows passed the tree the F8 finding was about, so it is not a substitute.
* **The phase gates green at the closing commit**, and commit before a long gate run.

---


# Rulings and state that bind this task

* **Where Task 1 left the tree** (`e7cb210d9`): `.environment` and `.local` are store-backed
  `Directory`s with the whole `Directory` surface; `ooTest.frm` loads. None of that is this task's to
  change.
* **Ruling S2 from the pre-flight scan:** Task 3 will make `usedArglist` apply to every entry point
  that runs the two-call protocol, the routine one this task builds included. Build the routine
  entry point so the too-many check sits in one place both callers share, rather than a copy.
* **Files this task also touches**, beyond the plan's list: `rust/corpus/phase-8.txt` and its
  witnesses with `.env`/`.d/` sidecars as the Phase 8 witnesses do, companions under
  `rust/crates/rexx-parse/tests/sourceline_oracle/` regenerated from scratch copies,
  `rust/crates/rexx-exec/src/run/tests.rs` (Step 5), and `rust/corpus/refusal-sites.tsv` if the
  refusal constructors change (refresh with `REXX_REFUSAL_SITES_REFRESH=1`, then measure every row the
  refresh leaves empty).
* **A second Phase 8 refusal on the routine path the plan does not name:** `run/tests.rs` pins
  `::ROUTINE EXTERNAL naming REXX or REGISTERED is not implemented (Phase 8)`. Task 9 closes the phase
  only if no refusal names Phase 8. Measure both forms on the oracle: implement the `REXX` form if it
  is this phase's surface, and re-home `REGISTERED` (the RXAPI function registry) to Phase 10 with the
  measurement as the reason, unless the measurement says otherwise. Record which, and why, in the
  report.
* **The shared-code model the L2 fix rounds built** (see the inherited context above): a routine's
  code is one object per routine table entry, found by exact spelling then caselessly; the first
  directive that binds it names its package; a version-refused library has no routines. Step 3's
  registration must keep all of that true, and the witnesses for it
  (`library_routine_package.rex`, `library_package_discarded.rex`,
  `library_bound_before_refusal.rex`, `library_context_package.rex`) must stay green.
* **The final review's B7 probes** are the reproduction for Step 3:
  `docs/superpowers/records/2026-09-14-phase-8/final-review-b-integration.md`, section B7 (`r1`,
  `r5`, `r6`, `r7`, with `rxmath`'s `RxCalcSqrt`). Their scratch copies are under
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/final-b/p/`.
* **Oracle crash list entry 15** is the shape where an unbound `loadExternal*` object is a context
  and a name is resolved through it: never run it, and do not build a witness that does.

# Operational rules (binding)

* Worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`, BASE
  `e7cb210d9`; check `git log -1` before starting. Read `rust/CLAUDE.md`.
* Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a **fresh empty directory** per run; stdout, stderr and exit status as three files, never
  `2>&1`. Crate: `rust/target/release/rexx-run FILE` the same way. Read
  `rust/corpus/oracle-crashes.txt` before any unusual probe and never run its entries. Never
  `NUMERIC DIGITS` above 1000, never `.Package~new` on a repository file, never
  `::OPTIONS TRACE ?<letter>`. A symbol `x` or `b` directly followed by a quote is a hex/binary
  literal. `ootest/` is read-only.
* Scratch: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-t2/`,
  a fresh directory per probe, no `rm` with a glob. Never `cd X && ...; rest` building paths from
  variables. `grep` is a ugrep wrapper that skips binary and ignored files; `/bin/grep -a` for counts.
* A change to `rust/crates/rexx-api/src/ffi.rs` or `load.rs` runs `cargo miri test -p rexx-api --lib`
  under Stacked Borrows before it commits; Miri is installed in a scratch `RUSTUP_HOME`, and
  `docs/superpowers/records/2026-09-14-phase-8/final-fix-report.md` (F7/F8) says how.
* `-j 4`; `rustfmt <path>` on touched files, then `cargo fmt --all --check`; per commit
  `cargo clippy -j 4 --workspace --all-targets -- -D warnings`, the touched crates' tests, and
  `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`. Not the full debug gate.
* Stage explicit paths; never `git add -A`, amend, `reset --hard`, `checkout --` on an edited file,
  or bare `stash`. Commit messages in a file via `git commit -F`, normal prose, ending with:

  ```
  Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_0157HT3JqnRRbgiMDb84iGSD
  ```
* No subagents. Ask before implementing if anything is ambiguous or contradicts the tree.
