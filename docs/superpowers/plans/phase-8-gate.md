# Phase 8 gate — the native API, L2 slice

**Assessed 2026-09-14.** The commit and the gate readings are in section 6, which was written from
the run rather than before it. The tree was clean at `d1796f69c` when the run started and nothing
touched it until the run finished.

The plan's row for this phase reads:

> `testbinaries/` compile unchanged against frozen headers; native-API ooTest groups pass.

**That sentence is not met, and this document does not claim it is.** What closed is the L2 slice,
which the plan document scopes explicitly: load a shared library, call the methods it exports, and
measure how far the framework then gets. The surface half — the function pointers the slice does
not fill, `testbinaries/`, and the three extension-only ooTest API groups (`METHOD`, `CONVERSION`
and `FUNCTION`; the other five are re-homed, section 7) — has its own plan at
`docs/superpowers/plans/2026-09-14-phase-8-surface.md` and is unbuilt. The phase's row in the
roadmap records both halves.

## 1. What the slice was, and what instrument reads each part

| criterion | instrument | where it is asserted |
|---|---|---|
| a shared library loads | `librxregexp.so` opened by name, its `RexxGetPackage` read | `rexx-api/tests/load.rs`, `corpus/lang/library_*.rex` |
| the interface layouts match the frozen header | field offsets and sizes against the C++ structs | `rexx-api/tests/layout.rs` |
| a stale handle misses rather than answering | generation-tagged registry, probed after recycling | `rexx-api/tests/handles.rs` |
| the two-call protocol | signature call, then argument call, on a real extension | `rexx-api/tests/invoke.rs`, `corpus/lang/library_*.rex` |
| conversion between Rexx objects and declared types | one case per union member the extension reaches | `rexx-api/tests/values.rs` |
| the context tables an extension calls back through | each reachable pointer exercised from the extension side | `rexx-api/tests/context.rs` |
| the directives that name a library | `::REQUIRES LIBRARY` and the three `EXTERNAL` forms | `corpus/phase-8.txt`, gate table D |
| every load failure reports what the oracle reports | the one-file failure programs and, since `40093e99b`, the two-file ones, three descriptors each | `corpus/lang/library_*_missing.rex` |
| `unsafe` stays inside its grant | a workspace scan with a positive control | `rexx-core/tests/unsafe_sites.rs` |
| the entry points built are the ones the plan names | the family table read back against the plan | `rexx-exec/tests/native_entries.rs` |

Every one of them is derived and run. None is a count written into prose.

## 2. What the slice built

A new crate, `rexx-api`, holding the C boundary: library loading, the `#[repr(C)]` interface
tables, the handle registry, the conversion table, and the two-call invocation path. Two modules
carry `unsafe` under D-U1 and nothing else does. In `rexx-exec`, the directives that name a library
now reach the loader instead of refusing, a resolved library is held for the interpreter's life
behind a cache whose map is private to its own write rule, and a raise crossing the boundary is
reported against the package that declared the method.

What arrived at the close-out rather than in a task of its own, each because a criterion was
checked rather than assumed:

* **`corpus/refusal-sites.tsv` had no way to be re-derived.** Every other derived table here has
  one, and this table's absence had left `the_table_holds_every_constructor_the_source_defines` red
  across two phases. The refresh mode added at `4122da9ac` regenerates the derived columns, carries
  each measured column forward by name, and leaves a new send-surface row's measured columns empty
  rather than inventing them, so the red set narrows to rows that genuinely need a probe. It named
  four; three were reachable and were probed against the oracle.
* **A constructor that delegates named no error identifier of its own.** The scanner read a
  delegating constructor as producing no identifier, so a probe that did reach it was scored as
  having gone elsewhere. Identifier derivation now follows one level of delegation, which is what
  the source does.
* **Five error numbers in the spec and the plan had been read out of `RexxErrorCodes.h` rather
  than taken from a run.** All five were wrong, and one of them — 98.978 — is reachable from no
  surface at all. Corrected across `b83ed8312` (the load-failure numbers: 98.982 for `::REQUIRES`
  and the two entry-missing paths), `378d5613b` (the argument refusals) and `165373cf5` (the last
  one retired).

## 3. The negative controls

Each was predicted before it was run.

* The use-after-unload probe: `Library::method` handing out an owned entry gave a call into an
  unmapped page from six lines of safe code. The first fix did not close it, because a raw pointer
  is `Copy` and a temporary lives to the end of its statement; making the field private and
  answering `has_entry_point()` did, confirmed by the re-review reaching `E0616`.
* The `Libraries` cache: control C probed the private map only from outside its module, where it
  was already unreachable. Redone from inside, with a positive control that compiles, it found
  `self.libraries.held.remove(name)` compiling — the hole the control existed to find.
* The `compile_fail` doctest that rustdoc reported as "compiled successfully, but marked
  compile_fail". Rewritten to use the borrow after the statement, with two controls each reddening
  exactly one block.
* Three tests that could not fail were found and either fixed or removed: a `dangling_mut()`
  sentinel that matched the address its control wrote, a `returned(0)` satisfied by an untouched
  descriptor, and a collection watcher looking at a `CStringPool` copy outside the heap.

## 4. What the instruments cannot see

* **A panic inside a callback aborts the whole binary**, because it crosses `extern "C"`. A
  control that only reddens by abort gives a red with no test name, so the boundary's panic
  behaviour is asserted by construction rather than by a failing run.
* **The corpus differential cannot see an address.** A `.Pointer` renders differently on every
  run, so `corpus/phase-8.txt`'s header records that a program there prints none.
* **`librxregexp.so` is a prebuilt artifact and is never rebuilt, and two builds of it are loaded.**
  The corpus differential hands both interpreters the oracle checkout's
  `/home/moritz/dev/repos/ooRexx/build/lib/librxregexp.so` (the `{oraclelib}` a `.env` sidecar
  expands, `tests/support/sidecar.rs`); the in-crate tests --
  `rexx-api/tests/{load,invoke,context}.rs` and `dispatch/library.rs`'s unit tests -- open this
  worktree's `build/lib/librxregexp.so`, a second build from identical sources
  (`diff -rq extensions/rxregexp` against the oracle checkout prints nothing) that the oracle
  never runs; measured 2026-09-15 with `sha256sum` and `readelf -n`. Either is the test
  instrument for the measured half of D5, whose `NEEDED` set and
  zero `rexx` imports hold on both, and that measurement says the *extension* half of the ABI comes
  out binary-compatible. The *embedding* half is source-compatible by promise and is Phase 9's.
* **No gate table owns a Phase 8 row.** Tables C and D carry no rows attributed to this phase, so
  `REXX_PHASE_GATE=8` would gate nothing and the run did not set it. That is asserted rather than
  assumed: `every_closed_phase_this_table_owns_rows_for_is_gated` reddens for any owner that has a
  committed corpus subset file and is not gated, `corpus/phase-8.txt` is now committed, and the
  test passes. The slice's evidence is the corpus subset, the crate's own tests, and gate table D's
  two Phase 7 rows in section 6.

## 5. What the slice leaves

Recorded in `docs/superpowers/plans/phase-4-exclusions.txt`'s KNOWN GAPS section with their
transcripts, and in `docs/superpowers/plans/phase-8-l2.md` for the L2 walk itself:

* **L2 is still not reached, and the blocker is no longer this phase's.** The framework now gets
  past `.ENDOFLINE`, past `rxregexp.cls` and past `::METHOD INIT EXTERNAL "LIBRARY rxregexp
  RegExp_Init"`, and stops at `ooTest.frm:49`, `.local~hasEntry(...)`. Nine `Directory` methods
  refuse on `.local` and `.environment`; every refusal names Phase 5, which is closed. Decision
  D-L2 records that no open phase owns them.
* **The surface half is unbuilt**, with its own plan document.
* **Three divergences the L2 walk found**, two of them not Phase 8's and the third partly this
  phase's -- its library rows were loud refusals before this slice, silent wrong file names during
  it, and are fixed at `40093e99b`; the `LIBRARY REXX` and superclass forms still name the
  program -- and **the L1 extractor gap**, each in the exclusions file.
* **`ootest/testOORexx.rex` writes fixtures into its working directory**, which is a hazard for
  whoever drives the framework next; `phase-8-l2.md` section 6 records it.

## 6. The gate readings

Recorded from the run, unpiped, at `d1796f69c`. Both failing sets were enumerated rather than
counted, and every member is attributed below.

| gate | command | exit | figures |
|---|---|---|---|
| G1 | `cargo fmt --all --check` | 0 | |
| G2 | `cargo clippy -j 4 --workspace --all-targets -- -D warnings` | 0 | |
| G3 | `cargo test -j 4 --release --workspace --no-fail-fast -- --test-threads=8` | 101 | 2471 passed / 5 failed |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast -- --test-threads=8` | 101 | 2471 / 6 |
| G5 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | **531 of 531** |

The corpus differential is 531 of 531 in both G4 and G5, against 516 of 516 at Phase 7's close;
`corpus/phase-8.txt` contributes the fifteen new programs. `base/keyword` is 892 of 896 bodies,
`base/bif` is 4992 of 4999 value rows and 186 of 186 raise rows, and the assertion table is 4247 of
4259 rows — all three in report mode, which is a progress signal and not a gate.

**G3's failing set**, every member of it older than this phase:
`a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop`,
`the_l0_subset_passes_again_under_collect_on_every_allocation`, and the three `ir::drive` counter
tests (`a_call_site_resolves_once_and_answers_from_what_it_kept`,
`a_long_constant_is_built_once_however_many_passes_read_it`,
`the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk`). **G4's** is that set plus
`concept_and_class_gate_table`.

Two members left the failing sets during this phase and none joined:

* **`the_table_holds_every_constructor_the_source_defines`**, red at Phase 7's close and at Phase
  5's, passes at `4122da9ac`. Section 2 records how.
* **`directive_option_gate_table`**, red in G4 at Phase 7's close, passes at `08d232ecc`. Gate
  table D's two Phase 7 rows — `::REQUIRES LIBRARY` and `::ROUTINE EXTERNAL` — now read `agree`,
  and the table's only rows that are not `agree` are owned by `deferred-parse-error-rendering`,
  which is neither closing nor closed, so nothing is gated.

`concept_and_class_gate_table` is red in G4 and green in G3 because the gated set is populated only
when `REXX_CORPUS_GATE` is set (`gate_tables/mod.rs:209`); it is a corpus-gate difference, not a
debug-versus-release one. Its 82 gated rows are all Phase 7's and all `unanswered` — `File`
instance 50, `Stream` instance 24, `StreamSupplier` instance 8. That is the same blindness Phase
7's own close named in its section 4 for `method-bodies.txt`, where a receiver that raises
identically on both sides leaves a row saying nothing; here the row is labelled `unanswered`
(`gate_tables/mod.rs:215-222`, the label for a row with no verdict) and the tally counts every
non-`agree` row as still open (`:307`), so a row nothing could answer never makes the gated count
smaller. (The first version of this paragraph cited Phase 7's section 4 for the `unanswered` rule
itself, which that section does not state.)

## 7. After the close: the final review, the fix rounds and their re-reviews

Every document named below is in this plan's SDD ledger, whose committed copy lands under
`docs/superpowers/records/` when the phase closes. No gate reading is written here; the controller
appends the readings taken at the commit after this one.

**The final review** of `659312de0..e64202ae7`, three read-only slices, each building its own copy
of the tree: the boundary (`final-review-a-boundary.md`: 0 Critical, 4 Important, 5 Minor), the
integration (`final-review-b-integration.md`: 0 Critical, 5 Important, 7 Minor) and the claim
documents (`final-review-c-claims.md`: 1 Critical, 7 Important, 10 Minor). The controller's triage
(`progress.md`, "Triage of the final review") split them three ways: behaviour fixed now; design or
scope routed to the surface plan (the thread context's lifetime, routine registration at every
library load site, a signature-level layout test, the special return codes); and prose, for the
documentation pass that wrote this section.

**The fix round**, F1-F11 (`final-fix-report.md`), eleven commits `04a286913` through `cf92ff4fb`:
a native string argument is converted by `REQUEST('STRING')` alone and a raise inside its
`MAKESTRING` is that raise (F1); the boundary's own refusals -- 88.909 and the parameter-side
93.968 -- are lineless and name the declaring package, and the result-side 93.968 keeps its line
against the sender (F2); a library load failure in a required package names that package's file and
line (F3); `~package` of a library-backed `EXTERNAL` method and of a `loadExternal*` answer is what
the oracle answers, through one record per procedure that the first binding directive sets (F4);
`Libraries` holds only a library that loaded, a version-refused one is held and answers loaded on
later asks with no routines, and 98.982 is raised on the first ask at every load site (F5); the
library search path is taken once when the interpreter starts (F6); `value_of` is `unsafe` and
`as_union` zeroes the word first (F7); the method context pointer is derived from the whole wrapper
(F8); argument presence is checked before the code and the result word read unstripped (F9); a
second open of a held name is counted (F10); every harness runs a corpus program with its sidecar,
and the sidecar control asks per part (F11). Its re-review, two slices
(`fix-rereview-boundary.md`: 0 Critical, 1 Important, 3 Minor; `fix-rereview-integration.md`:
0 Critical, 1 Important, 9 Minor), closed B1-B6, B8 and B10 on their original probes.

**The residual round**, X1-X5 (`residual-fix-report.md`): every interface table slot is
`unsafe extern "C" fn`, with a `compile_fail` doctest on `METHOD_CONTEXT` (`69a579370`); a library
routine's shared object is one per routine table entry, and a later `::ROUTINE` binder reports the
first binder (`0e57202cd`); a directive binds a library procedure while its package is translated
(`f83b028a7`); a package whose translation raised is not kept (`9b7f77e4c`); the per-variable
sidecar control, `ir_recorded`'s check that a library the oracle's build directory holds was
reached, the thread-table Miri test and three doc corrections (`e8a6b7667`). Its follow-ups: an
unbound `loadExternal*` object as a context argument hands on the `REXX` package, where the oracle
segfaults on the first name resolved through it (`9c0d44d8e`, `corpus/oracle-crashes.txt` entry
15), and `Library::routine` finds the exact spelling before a caseless match (`233d2766d`). Its
re-review (`residual-rereview.md`: 0 Critical, 1 Important, 3 Minor) found the round's own X3
meeting X4 -- a package whose translation raised still answered its routines through a bound
library object's parent walk -- and Z1 (`362f50453`) leaves such a package no routine, method or
resource table and no prolog while keeping its `::OPTIONS`, as the oracle hands its tables over
only in `resolveDependencies`.

**Parked, and where.** A user `REQUEST` method never sent by any string conversion (R9); the
condition object a trap reads naming the running program for a boundary raise (B12); a class
resolved through a package parent answering the unresolved symbol (residual re-review finding 4);
the oracle build's `RUNPATH` empty element; the `LIBRARY REXX` `unresolved_external` arm; the
package loader and unloader hooks; the `::ATTRIBUTE` getter-first order; `Method~new`'s third
argument; the instruments outside the gate; the blocked `collect_stress` L0 test: all in
`phase-4-exclusions.txt`'s Phase 8 section, each with its transcript and its owner or the reason
it has none. Two wrong counts in commit messages (R6: `ca79611e5`'s "12 of 13" is 11 of 13,
`04a286913`'s "five silent conversions" is six) and one one-witness claim (the fix report's "13/13
under both models" is Stacked Borrows alone, since Tree Borrows passes the unfixed tree too) are
recorded in the ledger, and the commits are left as they are.

### Refusals this phase re-homed

* **`handle_set`** to Phase 10, at `08d232ecc`: it needs `from_raw_fd`, which is the same grant
  RXAPI and the external queues need, and it never needed the loader (`dispatch/native.rs`'s doc
  on `deferred`). The scoping survey's section 5 had listed it among this phase's refusals.
* **Five of the eight native-API ooTest groups**, by measurement on 2026-09-15 (the scoping
  survey's section 3 carries the commands): `RexxStart`, `ProcessRexxStart`, `INVOCATION` and
  `ProcessInvocation` to Phase 9, since each loads `INVOCATIONTester.cls`, whose `orxinvocation`
  NEEDs `liborxexits.so` and through it `librexx.so.4`, importing `RexxCreateInterpreter` and
  `RexxStart`; `CLASSIC` to Phase 10, since `orxclassic` and `orxclassic1` import the function,
  subcom, queue and macro-space registries. Recorded in the roadmap's rows 9 and 10 and in
  `phase-4-exclusions.txt`. `METHOD`, `CONVERSION` and `FUNCTION` stay this phase's.
