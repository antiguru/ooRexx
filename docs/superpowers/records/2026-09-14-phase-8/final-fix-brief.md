# Phase 8 L2 slice — final-review fix dispatch

Read this first. It is your requirements.

## Where this fits

Phase 8 of a clean-room Rust rewrite of ooRexx (the C++ interpreter is the oracle; correctness is
byte-identical stdout, stderr and exit status). The L2 slice of the native API is built and closed
(`docs/superpowers/plans/2026-09-14-phase-8.md`, spec `docs/superpowers/specs/2026-09-14-phase-8-native-api.md`).
A whole-branch review in three slices then found defects. You fix the behavioural ones listed
below. Documentation findings are a separate pass after you; do not edit `docs/`.

Worktree: `/home/moritz/dev/repos/ooRexx-rust-rewrite`, branch `plan/rust-rewrite`, HEAD `e64202ae7`
when you start (check `git log -1`). Rust workspace under `rust/`. Read `rust/CLAUDE.md`.

## The three review reports — read the findings named below in full

* `.superpowers/sdd/2026-09-14-phase-8/final-review-a-boundary.md` (slice A)
* `.superpowers/sdd/2026-09-14-phase-8/final-review-b-integration.md` (slice B)
* `.superpowers/sdd/2026-09-14-phase-8/final-review-c-claims.md` (slice C, for context only)

Their scratch directories (probes, forged extensions, mutants) are under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/final-{a,b,c}/`.
Reuse their probe programs rather than rewriting them; `final-b/p/<name>/` holds each B probe and
`final-a/ext/forge.cpp` + `libforge.so` the forged extension. Do not modify those directories;
copy what you need into your own `scratchpad/final-fix/`.

## The fixes, in the order to do them

Each is its own commit. For each: reproduce the finding against the oracle first (three
descriptors, fresh empty directory), write the witness, **write the negative control's
prediction before running it**, fix, confirm the witness agrees byte for byte, confirm the control.

**F1 — A2 / B1: native string arguments use the wrong protocol.** `Host::string_value`
(`rust/crates/rexx-exec/src/dispatch/library.rs`, the `impl Host for Interp`) answers
`required_string_value(object).ok()`, the NOSTRING-resuming protocol that falls back to the
default name. The oracle's native argument conversion is `RexxInternalObject::requiredString`
(`interpreter/classes/ObjectClass.cpp`, reached from `NativeActivation::cstring` and the
`RexxStringObject` arm through `stringArgument`): send `REQUEST('STRING')` only; `.nil` is no
string value, which the conversion table already maps to 88.909. It must also not swallow a raise
from inside the argument's `MAKESTRING` (B1's `p3`: oracle 40.1 rc 216, crate 88.909 rc 168).
`Host::string_value` is declared in `rust/crates/rexx-api/src/values.rs` as answering
`Option<ObjRef>`, which cannot carry a raise; change the trait (and its test impl in
`rexx-api/src/invoke.rs`) if that is what not swallowing the raise takes.
Find the crate's existing implementation of that protocol if one exists (grep for `requiredString`
/ `request_string` / REQUEST sends) rather than writing a third. Witnesses: at least
`.object~new`, an object with `STRING` but no `MAKESTRING`, an object with `MAKESTRING` (answers,
must agree), an object whose `MAKESTRING` raises, and `.nil`, each through an `rxregexp` method
declared in a required package (not in the program sending), for `CSTRING` and
`RexxStringObject` parameters. Slice B's M-D mutant (answer `Some(object)` with no conversion)
must redden your witness; predict which lines.

**F2 — B2: 88.909 (and 93.968) raised at the boundary is not lineless.** `error.rs`
`argument_needs_a_string_value` does not set `delivery.lineless`, so `c9616b3d0`'s package-blame
rule misses it: crate `Error 88 running main.rex line 3`, oracle `Error 88 running re.cls:` with no
line. Check `incorrect_method_signature` (93.968) against `NativeActivation::reportSignatureError`
and give it the same delivery if the oracle's is the same; slice A's forged extension can reach a
signature error (see A's section 3). Witness: two-file, method declared in the required package.

**F3 — B6: a load failure in a required package names the running program.**
`Interp::resolve_directive_library` (`rust/crates/rexx-exec/src/lib.rs`) calls `blame_directive`;
the `::REQUIRES ... LIBRARY` arm in `load_required_packages` calls `blame_directive_in(id, ...)`,
which is right. Pass the `ProgramId` through from `install_directives` and use the same helper.
Witnesses: B's `a1`..`a4` shapes (`pk.cls` line 3: `::method` with a missing library, `::method`
with a missing entry, `::routine` with a missing entry, `::attribute` with a missing entry), each a
two-file program. The existing one-file `library_*_missing.rex` witnesses stay.

**F4 — B5: `Method~package` of a library-backed method answers `REXX`.**
`Interp::installed_executable_source` (`lib.rs`) answers `ExecutableSource::Native` for an
`EXTERNAL` binding; `Interp::external_packages` (added at `c9616b3d0`) already holds the declaring
package per `MethodId`. Make the package reader consult it, for `::METHOD ... EXTERNAL "LIBRARY x
y"` and for `.Method~loadExternalMethod` (whose package on the oracle is the package that sent
`loadExternalMethod`; measure it). B's `t5b` is the probe. The `LIBRARY REXX` form answering
`REXX` predates this range (B's `t5c`); fix it too if the same reader covers it, and say in the
commit which you did. Check that `make_method_private` and `PACKAGE`/`PRIVATE` access still agree
(B's `t7`).

**F5 — B3: a failed library load is held as a miss.** The oracle removes the package on a failed
load and retries on the next ask (`interpreter/package/PackageManager.cpp`, `loadLibrary`, around
the `packages->remove(name)`; print the lines and cite what you print). Hold only a success.
The comment on `Interp::libraries` attributes the opposite rule to the oracle; correct it. The test
`a_name_that_resolves_to_nothing_is_held_as_a_miss` asserts the wrong rule; replace it. Keep
`Libraries`' API narrow (`get`, non-replacing `hold`, private map in its own module). B's `miss`
probe is the witness shape (it copies a library into a search directory between two asks; do it in
scratch only, never under `build/`).

**F6 — B4: `LD_LIBRARY_PATH` is read on every resolve.** `library_search_path` reads the shadow
environment each time, so `VALUE('LD_LIBRARY_PATH', x, 'ENVIRONMENT')` widens the search, which
the oracle's loader (start-up-cached) never does. Take the value once when the interpreter is
constructed. B's `env2` probe. No process-global write: the value comes from the interpreter's own
environment as handed in, as it does now.

**F7 — A3: `ffi::value_of` can read uninitialised union bytes from safe code.** Make every
`ValueUnion` the crate produces fully initialised (zero the word, then write the member, which is
what `processArguments` does), and remove the safe path to a mismatched read: either `unsafe fn`
with the precondition in its signature, or take the `Repr` from the descriptor's own `type`. Keep
`unsafe` inside `ffi.rs`/`load.rs`. A's `zz_uninit_probe.rs` is the shape of a compile-level
witness; a `compile_fail` doctest or a test that the union's bytes are all written is acceptable.

**F8 — A4: `owner_of` reads outside the reference it derives from.** `Contexts::method` returns
`&mut self.method.context`; the stub's pointer is cast back to the whole `Owned` and `owner` read at
offset 24. Derive the pointer from the whole `Owned` (`&raw mut self.method`) and carry a raw
pointer where the reference is carried now; fix the `SAFETY:` note to name provenance. A's
section 1.2 names the two callers. If you can get Miri to run in your scratch (A could not; the
download cache is read-only), run it; otherwise say so.

**F9 — A5 and A6: two forged-signature divergences.** Unknown non-optional code with no argument:
oracle checks presence first (88.901), crate answers 93.968; `tests/values.rs` pins the crate's
order as the rule, so correct that test. Return type carrying the optional bit: oracle 93.968
(`valueToObject` switches on it unstripped), crate converts. A's `libforge.so` reaches both; the
crate's unit tests are the right home, and a corpus witness through `libforge.so` is **not**
acceptable (the corpus loads only the oracle's own prebuilt extensions).

**F10 — B10: `a_library_named_twice_is_opened_once` cannot fail.** Under B's mutant M-F (the
held-answer early return in `resolve_library` disabled) it passes with three `dlopen`s against
two. Make it count opens (a counter on the resolution path that the test reads, or equivalent)
so M-F reddens it. Run M-F and record that it does.

**F11 — B8 and B11: instruments that run nothing.** `rust/corpus/lang/external_method_package_blame.env`
is inert (the program loads no shared object): remove it, and check whether the sidecar control
`a_sidecar_changes_what_one_of_the_interpreters_answers` (`tests/corpus.rs`) can be made to see an
inert half beside a live one; if it can, do it, if not, say why. `collect_stress.rs` and
`ir_recorded.rs` (and check `coverage.rs`) list `phase-8.txt` but run its programs with no
library path and no `.d/` fixtures, so they reach only the load-failure path: either give them
the sidecar environment and fixtures the corpus harness gives, or remove `phase-8.txt` from their
lists with a comment naming why. Prefer the first if the harness path already exists to reuse.

## Global constraints (binding)

* `unsafe` only in `rust/crates/rexx-api/src/ffi.rs` and `src/load.rs` (D-U1), each block with a
  `SAFETY:` note naming the invariant and who establishes it. `rust/crates/rexx-core/tests/unsafe_sites.rs`
  is the record.
* No new dependency beyond `libloading` 0.8.9.
* No edit under `api/`. The C++ tree at the repo root, `build/`, `samples/`, `ootest/`, `oodocs/`,
  `testbinaries/` are read-only. `build/lib/librxregexp.so` is never rebuilt.
* No process-global state: no `std::env::set_var`/`remove_var`, no `set_current_dir`, no writes to
  fd 0/1/2 from interpreter code.
* Oracle runs: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a **fresh empty directory**. Compare stdout, stderr and exit status as three separate files,
  never `2>&1`.
* Read `rust/corpus/oracle-crashes.txt` before any unusual probe and never run its entries. Never
  `NUMERIC DIGITS` above 1000. Never `.Package~new` on a repository file. Never
  `::OPTIONS TRACE ?<letter>`. A symbol `x` or `b` directly followed by a quote is a hex/binary literal.
* Never `cd X && ...; rest` building paths from variables; use absolute paths or a subshell.
* `grep` is a ugrep wrapper that silently skips binary and ignored files: `/bin/grep -a` for counts.
* Comments minimal (`rust/CLAUDE.md`): one-sentence overview, params/returns/panics, non-obvious
  properties only. A comment never states the size of a set. No em-dashes in comments. Never drop
  an existing comment you did not make false; correct it instead.
* A number (error code, line) is measured or not written; every citation you add must land on its
  subject when printed. Print it.
* New corpus witnesses follow the existing Phase 8 ones: `rust/corpus/lang/library_*.rex` with
  their `.env` sidecar and `.d/` fixture directory, a row in `rust/corpus/phase-8.txt`, and a
  `rust/crates/rexx-exec/tests/sourceline_oracle/<name>.txt` companion where the neighbours have one.
  A two-file witness puts the declaring package in the `.d/` directory.

## How to work

* Scratch: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/final-fix/`.
  Probe directories: one fresh directory per probe, never reused, never `rm` with a glob.
* Build with `-j 4`. Format with `rustfmt <path>` on files you touched, then `cargo fmt --all --check`.
* Per fix, before committing: `cargo clippy -j 4 --workspace --all-targets -- -D warnings`, the
  tests of the crates you touched, and `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`.
  Do **not** run the full debug `REXX_CORPUS_GATE=1 memcap 8G cargo test` gate; the controller runs
  the phase gates after the re-review.
* Never `git add -A`; stage explicit paths. Never amend, never `git reset --hard`, never
  `git checkout -- <path>` on an edited file, never bare `git stash`. Commit messages in a file,
  passed with `git commit -F`, in normal prose (what and why, the measurement), ending with:

  ```
  Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_0157HT3JqnRRbgiMDb84iGSD
  ```
* Do not dispatch subagents.
* **Ask before implementing if anything here is ambiguous, contradicts the tree, or looks wrong**:
  message the controller and wait. Pre-flight questions have been the most valuable thing a
  dispatch produces on this project.

## Report

Write `.superpowers/sdd/2026-09-14-phase-8/final-fix-report.md` **first**, with a heading per fix,
and append as you go: for each fix, the reproduction (both sides), the witness, the control's
prediction and result, the commit hash, and anything you did not do and why. End with "Not done".
When finished, message the controller with status (DONE / DONE_WITH_CONCERNS / BLOCKED), the
commit list, a one-line test summary, and concerns. Writing the file alone does not reach the
controller.
