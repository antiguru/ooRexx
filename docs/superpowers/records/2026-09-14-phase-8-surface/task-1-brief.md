### Task 1: `.environment` and `.local` answer the whole `Directory` surface

`owns` (`rust/crates/rexx-exec/src/dispatch/hash.rs`) answers false for any `Body::Native`
receiver, and `.environment` and `.local` are `Body::Native` directories on `NativeObject`'s map,
so every store-family method but `at` and `put` reaches `not_this_task`: `method "HASENTRY" of
class "Directory" is not implemented (Phase 5)`. On a plain `.Directory~new` all nine D-L2 names
answer byte-identically. Phase 5h Task 4 kept these two `Body::Native` deliberately
(`docs/superpowers/records/2026-09-07-phase-5h-mapped-collections/task-4-report.md`).

**Files:** `rust/crates/rexx-exec/src/environment.rs`, `src/dispatch/hash.rs`, and whatever the
design step names; witnesses under `rust/corpus/lang/`.

- [ ] **Step 1: Find the blocker by running, before designing.** Moritz remembers one and not
      what it was. Each candidate gets a probe on both sides and a verdict in the report:
      (a) iteration order -- the oracle's `.environment~allIndexes` is bucket order, identical
      across runs, `LOCAL` last, and `.local`'s is `SYSCARGS INPUT TRACEOUTPUT DEBUGINPUT STDOUT
      OUTPUT STDERR STDIN STDQUE ERROR`; a `NativeObject` map cannot reproduce that; (b) `.local`'s
      streams and monitors are minted on first demand, so a whole-collection read must see them
      as present, and `STDQUE` is Phase 10's external queue; (c) `EnvironmentModel::unbuilt` is
      computed once and never shrinks as the library bootstrap installs names; (d) the D45 security
      chokepoint every `.local`/`.environment` read passes; (e) the cost of `.NAME` resolution,
      which reads `.environment` on every class reference: measure `rexxcps` and a `.NAME`-heavy
      loop before and after, interleaved, a few runs each; (f) anything else the Phase 5 records
      say (`docs/superpowers/records/2026-08-*-phase-5a*/`, `2026-09-07-phase-5h-*`).
- [ ] **Step 2: Choose the representation and write the choice into the report before code.** The
      likely shape is the store `dispatch/hash.rs` gives a `.Directory~new` instance, with its
      string-value key protocol, filled in the oracle's
      insertion order at the oracle's bucket size, with the security chokepoint and `.local`'s
      minting kept on the lookup path. If Step 1 shows a cheaper shape reproduces the order, take
      it. Whatever answers must answer for every `Directory` method, not only the nine: enumerate
      them from `Directory`'s method table at test time.
- [ ] **Step 3: Witnesses.** Every `Directory` method on `.environment` and on `.local` against
      the oracle: `allIndexes`, `allItems`, `supplier`, `makeArray` and `items` compared as
      ordered output; the entry family's upper-casing; `setEntry` with no value removing; a
      program-added entry; `remove` of a class name and a later `.NAME` for it; `.local`'s minted
      names before and after first use. Where `STDQUE` makes an answer Phase 10's, the refusal
      names Phase 10 and a witness pins that.
- [ ] **Step 4: Controls.** Restore `owns`'s `Body::Native` early return and predict which
      witnesses redden; reverse the insertion order and predict which ordered witnesses redden.
- [ ] **Step 5: Walk the framework.** Run `ooTest.frm`'s loading against this crate with the
      single-group form of `METHOD` and record, in the report and in `phase-8-l2.md`'s successor
      section, the next place it stops and who owns it. **Do not fix what that finds in this task.**
- [ ] **Step 6:** `closed_phases.rs` gains `"Phase 5"` if no refusal names it any more; otherwise
      list what still does, each with its owner, in the report. Commit.

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


# Rulings from the pre-flight scan that bind this task

* Task 1's Files also include `rust/crates/rexx-exec/tests/closed_phases.rs` (Step 6), `rust/corpus/phase-8.txt` (Step 3's rows, following the Phase 8 witness conventions: `corpus/lang/<name>.rex`, `.env`/`.d/` sidecars only if needed, a `rust/crates/rexx-parse/tests/sourceline_oracle/<name>.txt` companion regenerated from a scratch copy), and `docs/superpowers/plans/phase-8-l2.md` (Step 5: append a new section; do not edit the earlier sections, which are the record of the L2 walk).
* Step 6's condition is not expected to hold: `/bin/grep -rln --include=*.rs '"Phase 5"' rust/crates/*/src` names owners in `environment.rs`, `lib.rs`, `redirect.rs` and `run.rs` at the start, most unrelated to `Directory`. Report what still names Phase 5 after your change, each with the owner it should have, and add `"Phase 5"` to `closed_phases.rs` only if nothing does.
* Step 5 walks the framework and records where it stops next. It does not fix it: Task 8 owns that (ruling S4).

# Operational rules (binding)

* Worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`, BASE
  `d7eafeb54`; check `git log -1` before starting. Read `rust/CLAUDE.md`.
* Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a **fresh empty directory** per run; stdout, stderr and exit status as three files, never
  `2>&1`. Crate: `rust/target/release/rexx-run FILE` the same way. Read
  `rust/corpus/oracle-crashes.txt` before any unusual probe and never run its entries. Never
  `NUMERIC DIGITS` above 1000, never `.Package~new` on a repository file, never
  `::OPTIONS TRACE ?<letter>`. A symbol `x` or `b` directly followed by a quote is a hex/binary
  literal. `ootest/` is read-only: to walk `ooTest.frm`, copy what you need into scratch.
* Scratch: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-t1/`,
  a fresh directory per probe, no `rm` with a glob. Never `cd X && ...; rest` building paths from
  variables. `grep` is a ugrep wrapper that skips binary and ignored files; `/bin/grep -a` for counts.
* Performance (Step 1e): the benchmark tooling is `rust/crates/rexx-bench`; always include
  `rexxcps`; interleave the before and after binaries run by run, give each revision its own
  `CARGO_TARGET_DIR`, and prefer instruction counts (`valgrind --tool=callgrind`) over wall clock
  for a difference under a few percent.
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
