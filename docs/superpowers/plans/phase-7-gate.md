# Phase 7 gate — streams and platform

**Assessed 2026-09-13.** The commit and the gate readings are in section 8, which was written from
the run rather than before it.

The plan's row for this phase reads:

> `StreamClasses.orx` runs; stream model, `ADDRESS`, file system green **on the host**; the `Sys*`
> subset ooTest needs (D11) works. The other four platforms are Phase 11's, per D-P1.

with **L2** in the plan table's `Rung` column. **The exit gate is the sentence, and it is met**;
section 7 is about the rung, which is not, and which Phase 5 also carried while closing without
ooTest running -- the column is ordinal rather than a second gate.

## 1. The criteria, and what instrument reads each

| criterion | instrument | where it is asserted |
|---|---|---|
| `StreamClasses.orx` runs | the embedded library installs and its classes answer | `corpus/lang/*.rex`, `method-bodies.txt` |
| the stream model | `Stream`, `StreamSupplier` and the eight builtins against the oracle | `corpus.rs`, `method_bodies.rs` |
| `ADDRESS` | command dispatch, the handler table, `WITH` redirection | `corpus/phase-7.txt`'s command rows |
| the file system | `.File`'s rows and the `Sys*` file routines | `method_bodies.rs`, `sys_file_functions.rex` |
| the `Sys*` subset | the nine routines D11 names | `internal_routines.rs`, `sys_file_functions.rex` |
| no refusal names this phase | a source scan with a negative control | `closed_phases.rs` |

Every one of them is derived and run. None is a count written into prose.

## 2. What the phase built

The stream model and its state, `.File`, the standard streams and the monitors, commands and
`ADDRESS` including `WITH` redirection, the platform layer (a shadow environment and a shadow
current directory, with no process-global write anywhere), external routine resolution, the nine
`Sys*` routines, the internal-routine table that serves `DIRECTORY`, `FILESPEC` and `BEEP` beside
them, interactive `TRACE ?` with its pause and prompt, `RXTRACE=ON`, and the security manager's six
checkpoints.

What arrived at the close-out rather than in a task of its own, each because a criterion was
checked rather than assumed:

* **`.local`'s entries were readable as `.NAME` and not as entries.** `.output` answered a monitor
  where `.local['OUTPUT']` refused, because the bundle is minted on first demand and only one of
  the two paths counted as a demand. `corpus/lang/local_entries.rex` pins all nine names and the
  identities between the two spellings.
* **Positioning a transient stream by line or character was allowed.** `SEEK` had the check and
  `lineIn(n)`/`charIn(n)` did not, so `.Stream~new('/dev/null')~supplier` built a supplier where
  the oracle raises 93.958.
* **A required argument to a native method reported the wrong error.** `stream_position` declares
  its options string without `OPTIONAL_`, so omitting it is 88.901 against the package -- `Error 88
  running REXX`, no program line. The crate answered 93.903.

## 3. The negative controls

Each was predicted before it was run.

* `corpus/lang/call_miss_not_cached.rex` raises 43.1 on its first pass, writes the file, and finds
  it on the second. Deleting the guard in `Calls::remember` makes the second pass raise too.
* `closed_phases.rs` finds a phase that is still open by the same walk that finds none of the
  closed one, so an empty result is not the scan looking in the wrong place. Re-introducing one
  `"Phase 7"` owner reddens it.
* `prompt_before_read.rs` waits for a prompt with nothing yet written to the child's standard
  input. Making the handover return early fails both cases with the read still blocked.
* `trace.rs`'s `off_refuses_the_debug_flag_from_either_side` carries `?R` rows as its positive
  control, so a guard that refused debug everywhere would not pass it.

## 4. What the instruments cannot see

* **`method-bodies.txt` records `answers` for a row whose receiver raised identically on both
  sides.** The send never happened; the two sides agreed about that. `Stream` and `StreamSupplier`
  were in exactly that state at the start of the close-out -- 32 rows saying nothing -- and giving
  each class a receiver that actually constructs is what turned the two defects in section 2 up.
  The rows now carry `rc 0` where they answer.
* **The corpus differential cannot see anything that is not a byte on a descriptor.** The prompt
  ordering in section 3 is the case: a prompt written at exit and one written before the read
  compare equal once the run is over.
* **A sidecar cannot see a witness that was deleted**, only one whose content changed.
* **The oracle's own crashes bound what may be probed.** `corpus/oracle-crashes.txt` gained this
  phase's entries; entry 9, `::OPTIONS TRACE ?<letter>`, is an indefinite block, so the `TRACE OFF`
  rule in section 2 was settled against the C++ source rather than against a run.

## 5. The platforms

**Linux, measured.** Every figure in this document was taken on this host.

**The other four: CANNOT ASSESS, and that is Phase 11's, not an accepted divergence.** Ruling D-P1
records it: what is deferred is portability, not correctness, because on a platform the crate does
not run there is no output to differ. The measured cost of the seam is `PLATFORM = b"LINUX"` and
`LINE_END` in `parse_template.rs`, eight `std::os::unix` uses across six files, and nineteen
`rustix` sites in two. The unblocker is a second host, which Moritz will provide.

## 6. Refusals this phase re-homed

Nothing in the crate names Phase 7 any more, and `closed_phases.rs` asserts it. The refusals it
left behind went to the phase that actually owes each:

* **Phase 8** -- `::ROUTINE EXTERNAL`, `::REQUIRES LIBRARY`, the `::METHOD` and `::ATTRIBUTE`
  `EXTERNAL` forms naming a library other than `REXX`, `loadExternalMethod`/`loadExternalRoutine`,
  `Package~loadLibrary`, and the `handle_set` entry point. All of them open a shared library.
* **Phase 10** -- the queue entry points, `.STDQUE`, the RexxUtil remainder, and
  `Package~options(name, value)`.
* **Phase 5** -- the two `.STREAM` fallbacks, which can only fire before the library bootstrap has
  installed `StreamClasses.orx`.

## 7. The L2 rung is not reached, and the blocker is Phase 8's

The ladder's L2 is *"`ooTest.frm` loads and a single test group executes"*. It does not load, and
the chain was walked rather than guessed:

1. `ootest/testOORexx.rex` stopped at `.ENDOFLINE`, which the framework's own prologue reads
   (`OOREXXUNIT.CLS:77`). No test group ran. That entry was the last unbuilt name in
   `.environment`; it is a one-line string constant and this phase built it rather than leave the
   suite blocked on a closed phase's omission.
2. The framework then needs `rxregexp.cls`, which ships with the interpreter rather than with the
   suite, so this crate -- which is installed nowhere -- has to be pointed at a copy.
3. Pointed at one, it stops at `::METHOD INIT EXTERNAL "LIBRARY rxregexp RegExp_Init"`.

**That is a shared library, and loading one is Phase 8's.** So the rung cannot be reached from
inside Phase 7 by any amount of work on streams or the platform layer. Nothing here is a Phase 7
gap: what stands between the framework and its first test group is `dlopen`, and the `Rung` column
put L2 against this phase before that was known. The step it names should sit against Phase 8,
whose entry already lists 7.

Two smaller facts from the same walk, both worth having written down. `.ENDOFLINE` was the last
unbuilt name in `.environment`, so that directory is now complete. And `rxregexp.cls` ships with
the interpreter rather than with the suite -- Phase 5's row already records it as off this build's
search path -- so a crate that is installed nowhere has to be pointed at a copy before the question
of `dlopen` even arises.

## 7a. What this phase leaves open, beyond the rung

Both are in `phase-4-exclusions.txt`'s KNOWN GAPS section with their transcripts, and both were
recorded there at the close rather than during the work -- the first because it had been left in a
session message, which is the failure this file exists to prevent.

* **Task 17 Step 3 was designed and not written.** A callee that does not parse is a loud refusal
  here and the callee's own error on the oracle, rc 120 against 229. **The refusal names Phase 5**,
  so `closed_phases.rs` does not see it. Its two halves -- the substitution `rexx-parse` cannot yet
  supply, and the callee's own attribution -- are sized in the plan's own Step 3 text.
* **No native entry point checks its required argument count at the boundary.** `stream_position`
  was the one case a program can reach and is fixed; which parameters are required lives in the
  C++ `RexxMethodN` declarations and nothing here derives it.

## 8. The gate readings

Recorded from the run, unpiped, at `73aed8f25`. Both failing sets were enumerated rather than
counted, and every member is a pre-existing failure carried from before this phase.

| gate | command | exit | figures |
|---|---|---|---|
| G1 | `cargo fmt --all --check` | 0 | |
| G2 | `cargo clippy -j 4 --workspace --all-targets -- -D warnings` | 0 | |
| G3 | `cargo test -j 4 --release --workspace --no-fail-fast -- --test-threads=8` | 101 | 2348 passed / 6 failed |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast -- --test-threads=8` | 101 | 2347 / 8 |
| G5 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | **516 of 516** |

The corpus differential is 516 of 516 in both G4 and G5. `base/keyword` is 892 of 896 bodies, with
`corpus/keyword-exempt.txt` holding the four and policing the set in both directions; `base/bif` is
4992 of 4999 value rows and 186 of 186 raise rows.

**G3's failing set**, every member of it older than this phase:
`a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop`,
`the_l0_subset_passes_again_under_collect_on_every_allocation`,
`the_table_holds_every_constructor_the_source_defines`, and the three `ir::drive` counter tests
(`a_call_site_resolves_once_and_answers_from_what_it_kept`,
`a_long_constant_is_built_once_however_many_passes_read_it`,
`the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk`). **G4's** is that set plus the two
gate-only tables, `concept_and_class_gate_table` and `directive_option_gate_table`.

One member left the set during this phase rather than joining it:
`dispatch::native::tests::every_family_still_defers_something` asserted that every entry-point
family still owes something, which stopped being true when `file` was finished. It now reads
"every deferred entry point names an **open** phase", which is the property that was wanted.

**`corpus/refusal-sites.tsv`'s wholesale re-derivation is still owed**, and
`the_table_holds_every_constructor_the_source_defines` is the assertion that says so on every run.
Its rows' line numbers drift as the crate moves; the constructor set itself is checked in both
directions, and this phase kept it current by hand.
