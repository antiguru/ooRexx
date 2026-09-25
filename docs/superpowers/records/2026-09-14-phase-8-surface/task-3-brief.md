### Task 3: The conversion table's remaining rows

**Files:** `rust/crates/rexx-api/src/values.rs`, `src/invoke.rs`, `tests/values.rs`

- [ ] **Step 1:** Fill every `REXX_VALUE_*` row `NativeActivation::processArguments`
      (`interpreter/execution/NativeActivation.cpp:219`) handles, one row at a time, each with the
      `Repr` its code uses.
- [ ] **Step 2:** `ARGLIST` carries `usedArglist` (`NativeActivation.cpp:680`), so the too-many
      check in `invoke::method` is skipped for a signature that takes one.
- [ ] **Step 3:** The special codes as *return* types answer `Failure::ResultSignature` (93.968,
      reported with its line against the sender, as F9 measured for the result side), which is
      `valueToObject`'s `default:`, rather than `Unfilled`. Not `Failure::Signature`, which is the
      argument side's lineless delivery against the declaring package.
- [ ] **Step 4:** Per-row tests both directions, and the row-deletion control re-run on a row this
      task adds rather than one it inherited. Commit.

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

* **Where Tasks 1 and 2 left the tree** (`5d84dd8cb`): `invoke::routine` and `invoke::method` share
  one private `run` holding the one too-many check (ruling S2); Task 2 already filled `double` and
  `positive_wholenumber_t` to_native (88.921, 88.905 measured) and `RexxObjectPtr` from_native, and
  the thread table's `DoubleToObjectWithPrecision`. A refusing slot records itself and `run` refuses
  loudly after the stub (`Failure::UnfilledSlot`); `Throw*` and the pointer-returning slots still
  abort.
* **Ruling S2:** `usedArglist` applies to every entry point `run` serves, routines included.
* **The special codes as return types** answer `Failure::ResultSignature` (93.968 with its line
  against the sender; a routine's is 40.918), not `Failure::Signature` (the argument side's lineless
  delivery). Check `Failure::error_number` routes both.
* **`Refused::Unfilled` renders a doubled suffix** (`rexx-exec: Phase 8 owes the FromNative conversion
  for REXX_VALUE_size_t (33) is not implemented (Phase 8)`); `phase-4-exclusions.txt` makes it this
  task's. A row this task leaves unfilled must render once.
* **Rows shipped extensions reach today** and refuse: `CSTRING` from_native (`rxmath`'s
  `MathLoadFuncs`/`MathDropFuncs`), `size_t` from_native (`orxmethod`'s `TestInterpreterVersion`).
  Witness those through the extensions where a path exists, byte-identical to the oracle.
* **Files this task also touches:** `rust/corpus/phase-8.txt` and its witnesses, sourceline companions
  under `rust/crates/rexx-parse/tests/sourceline_oracle/`, `rust/corpus/refusal-sites.tsv` if refusal
  constructors change, and `phase-4-exclusions.txt`'s `Refused::Unfilled` entry when it closes.
* **Forged extensions** may be built in scratch to reach rows no shipped extension declares; they
  witness only through unit tests or scratch transcripts, never the corpus.
* **Never load a prebuilt extension that NEEDs `librexx.so`/`librexxapi.so`, directly or transitively**
  (`liborxinvocation.so`, `liborxexits.so`, `liborxclassic.so`, `liborxclassic1.so`).
* **Never register anything with the rxapi daemon.**

# Operational rules (binding)

* Worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`, BASE
  `5d84dd8cb`; check `git log -1` before starting. Read `rust/CLAUDE.md`.
* Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a **fresh empty directory** per run; stdout, stderr and exit status as three files, never
  `2>&1`. Crate: `rust/target/release/rexx-run FILE` the same way. Read
  `rust/corpus/oracle-crashes.txt` before any unusual probe and never run its entries. Never
  `NUMERIC DIGITS` above 1000, never `.Package~new` on a repository file, never
  `::OPTIONS TRACE ?<letter>`. A symbol `x` or `b` directly followed by a quote is a hex/binary
  literal. `ootest/` is read-only.
* Scratch: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-t3/`,
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
* **`/tmp` is a shared tmpfs:** `df -h /tmp` before each build; give each revision its own
  `CARGO_TARGET_DIR` and delete it by explicit path when done.
* No subagents. Ask before implementing if anything is ambiguous or contradicts the tree.
