# Phase 8 gate — the native API, L2 slice

**Phase 8 closed on 2026-09-28; section 10 is the close.** What follows up to section 9 is the
record as it was written.

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
  surface at all. Corrected across `b83ed8312` (the load-failure numbers: 98.903 where 98.982 had
  been written for `::REQUIRES`, and the two entry-missing paths), `378d5613b` (the argument refusals) and `165373cf5` (the last
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

* **L2 is still not reached, and its blocker is now this phase's.** The framework now gets
  past `.ENDOFLINE`, past `rxregexp.cls` and past `::METHOD INIT EXTERNAL "LIBRARY rxregexp
  RegExp_Init"`, and stops at `ooTest.frm:49`, `.local~hasEntry(...)`. Nine `Directory` methods
  refuse on `.local` and `.environment`; every refusal names Phase 5, which is closed. Decision
  D-L2 recorded that no open phase owned them, and Moritz then ruled that Phase 8 does: they are
  Task 1 of `2026-09-14-phase-8-surface.md`.
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
  `ProcessInvocation` to Phase 10 (first recorded as Phase 9's; corrected 2026-09-28, section 9),
  since each loads `INVOCATIONTester.cls`, whose `orxinvocation` NEEDs `liborxexits.so` and through
  it `librexx.so.4`, importing `RexxCreateInterpreter` and `RexxStart` and also the exit and subcom
  registries; `CLASSIC` to Phase 10, since `orxclassic` and `orxclassic1` import the function,
  subcom, queue and macro-space registries. Recorded in the roadmap's rows 9 and 10 and in
  `phase-4-exclusions.txt`. `METHOD`, `CONVERSION` and `FUNCTION` stay this phase's.

## 8. The gate readings after the rounds

Recorded from the run, unpiped, at `476347f52`, the commit that lands the documentation pass, the
amended surface plan and the copied records. The tree was clean before and after the run and HEAD
did not move. Both failing sets were enumerated.

| gate | command | exit | figures |
|---|---|---|---|
| G1 | `cargo fmt --all --check` | 0 | |
| G2 | `cargo clippy -j 4 --workspace --all-targets -- -D warnings` | 0 | |
| G3 | `cargo test -j 4 --release --workspace --no-fail-fast -- --test-threads=8` | 101 | 2524 passed / 5 failed |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --workspace --no-fail-fast -- --test-threads=8` | 101 | 2524 / 6 |
| G5 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | **545 of 545** |

The corpus differential is 545 of 545 in both G4 and G5, against 531 of 531 in section 6. The
report-mode figures did not move: `base/keyword` 892 of 896 bodies, `base/bif` 4992 of 4999 value
rows and 186 of 186 raise rows, the assertion table 4247 of 4259 rows.

**Both failing sets are section 6's, member for member.** G3's is the three `ir::drive` counter
tests, `a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop` and
`the_l0_subset_passes_again_under_collect_on_every_allocation`; G4's is that set plus
`concept_and_class_gate_table`. No member joined or left during the rounds; the same sets were
read at `cf92ff4fb` and `233d2766d` on the way.

Miri is not among these gates: `rexx-api`'s lib tests ran under Stacked Borrows during the rounds
from a scratch toolchain, as the INSTRUMENTS OUTSIDE THE GATE entry of `phase-4-exclusions.txt`
records.

## 9. The surface plan's Task 7: frozen headers, and which phase owns each API group

Measured 2026-09-28 at `c750719b7`, from the worktree root, the oracle checkout at
`/home/moritz/dev/repos/ooRexx`.

**"`testbinaries/` compile unchanged against frozen headers" is witnessed by the oracle's own
build.** Both commands print nothing and exit 0:

    diff -rq api /home/moritz/dev/repos/ooRexx/api
    diff -rq testbinaries /home/moritz/dev/repos/ooRexx/testbinaries

and every target `testbinaries/CMakeLists.txt` declares has its product in the oracle's build (each
`ls` succeeds; the libraries land in `build/lib`, the `rexxinstance` and `provoke_locks`
executables in `build/bin`):

    O=/home/moritz/dev/repos/ooRexx/build
    for t in $(/bin/grep -a -o -e 'generate_test_library([a-z0-9]*' -e 'add_library([a-z0-9]\+' \
                 testbinaries/CMakeLists.txt | sed 's/.*(//'); do ls $O/lib/lib$t.so; done
    for t in $(/bin/grep -a -o 'add_executable([a-z_]*' testbinaries/CMakeLists.txt | sed 's/.*(//'); do
      ls $O/bin/$t; done

So the sources the plan row names, against headers byte-identical to the ones this crate implements,
compiled and linked. **That says the headers are compatible, not that the entry points behind them
work**: a build checks declarations, and a declaration this crate leaves refusing compiles exactly
as well as one it implements. Whether the entry points work is what running the groups says (Task
8). One caveat on the witness: the products are dated 2026-08-05 16:02, and three files under the
oracle's `testbinaries/` (`CMakeLists.txt`, `provoke_locks.cpp`, `provoke_locks.rex`) have a later
mtime, 2026-08-22; `git status --short api testbinaries` in the oracle checkout is empty and
`git log -1 -- api testbinaries` there is `334ab5460`, 2026-08-02, so the content those builds saw
is the committed content unless the tree was edited and restored in between, which nothing records.

**A compile of today's tree, added 2026-09-29 (final review M5).** The builds above are dated
2026-08-05. Every `testbinaries/` source was compiled against this worktree's `api/` with
`-fsyntax-only` from the worktree root, `g++ (Debian 16.2.0-3) 16.2.0` and `gcc` of the same
version. Each line printed `ok`, and both `diff -rq` commands above still print nothing:

    for f in testbinaries/*.cpp; do
      g++ -fsyntax-only -Iapi -Iapi/platform/unix -Itestbinaries "$f" && echo "ok $f" || echo "FAIL $f"
    done
    gcc -fsyntax-only -Iapi -Iapi/platform/unix testbinaries/orxclassic1.c \
      && echo "ok testbinaries/orxclassic1.c" || echo FAIL

This checks declarations, not linking. A negative control, with its prediction written first,
used a scratch copy of `api/` whose `oorexxapi.h` renamed `RaiseCondition` to `RaiseConditionX`.
Exactly the two sources that call it failed, `orxmethod.cpp` and `orxinstance.cpp`.

**Which phase owns each API group is derived, not recorded**, by
`rexx-exec/tests/api_group_partition.rs`:

    cargo test -p rexx-exec --test api_group_partition

It reads every `.testGroup` under `ootest/ooRexx/API` and the package files each names through
`loadPackage`, collects the libraries bound by `EXTERNAL "LIBRARY <name> ..."` and by `rxfuncadd`,
follows each library's `readelf -d` `NEEDED` entries transitively from the oracle's `build/lib`
(stopping at `librexx.so` and `librexxapi.so`), and collects the undefined `Rexx*` symbols of every
library it reaches (`readelf --dyn-syms`). A group whose closure needs no interpreter library is
Phase 8's; any other group belongs to the latest phase one of its imports needs, from the test's own
map (embedding to Phase 9, the RXAPI registries to Phase 10; memory and the variable pool name no
phase). Nothing is loaded. The test's list and the derivation agree: `METHOD`, `CONVERSION` and
`FUNCTION` are Phase 8's; `CLASSIC` is Phase 10's; and `RexxStart`, `ProcessRexxStart`,
`INVOCATION` and `ProcessInvocation` are **Phase 10's, not Phase 9's** as section 7 first recorded.
Each binds `orxinvocation`, which NEEDs `liborxexits.so`, which imports `RexxCreateInterpreter` and
`RexxStart` but also `RexxRegisterExitDll`, `RexxRegisterExitExe`, `RexxDeregisterExit`,
`RexxQueryExit`, `RexxRegisterSubcomExe` and `RexxDeregisterSubcom`, the registries
`librexxapi.so.4` exports (and whose client registers through the rxapi daemon,
`rexxapi/client/LocalRegistrationManager.cpp:70`). The full import list, `RexxVariablePool` and
`RexxFreeMemory` besides, is what

    readelf --dyn-syms -W /home/moritz/dev/repos/ooRexx/build/lib/liborxexits.so \
      | awk '$7=="UND"{print $8}' | /bin/grep -a '^Rexx'

prints. A group runs only once every import it needs exists, so it is the later phase's.

The test's own negative control, `a_group_whose_library_changes_changes_phase`, rewrites
`METHODPackage.cls`'s library to `orxinvocation` in a copy under `CARGO_TARGET_TMPDIR` and asserts
`METHOD` moves to Phase 10. Two controls were run by hand with their predictions written first
(`.superpowers/sdd/2026-09-14-phase-8-surface/task-7-report.md`): pointing the test at a scratch copy
whose `FUNCTIONPackage.cls` binds `orxclassic` turned it red on the `FUNCTION` row alone, and
replacing the maximum with the first classified import turned it red on the `INVOCATIONTester.cls`
groups alone, each at Phase 9.

## 10. The surface plan's close: the gate readings, and what Phase 8 leaves

**Phase 8 closed on 2026-09-29 at `c32f4ce21`**, after the final review's fix round
(`.superpowers/sdd/2026-09-14-phase-8-surface/task-9-fix-report.md`). Task 9 first called it
closed at `fc32f74aa` on 2026-09-28. The final review then found that the groups' "pass" was
measured as agreement only (I1), a `Relation` regression in `DirectoryPut` (I2), two blind spots in
`closed_phases` (I3), two re-homes the oracle contradicts (I4), and the gate starting the oracle's
rxapi (I5). The gate script is the surface plan's own (`scratchpad/surface-9b/gates.sh`), with a
`G3 Compiling lines` count added. It ran unpiped after every rust source was touched, and its status
file is quoted here line for line:

    c32f4ce21881c6b0dcc44171857f40abdb78a09b
    started 2026-09-29T01:20:04+02:00
    load at start 5.43 6.14 6.76 4/2578 873333
    G1 fmt exit 0
    G2 clippy(empty target) exit 0
    G3 release build (test --no-run) exit 0
    G3 Compiling lines: 11
    load G4 14.94 15.52 10.71 2/2587 879282 2026-09-29T01:23:29+02:00
    G4 release test exit 0
    G4 Compiling lines: 0
    load after G4 3.14 9.01 9.54 4/2585 1007845
    G5 debug build (test --no-run) exit 0
    load G6 8.22 9.82 9.80 3/2592 1016639 2026-09-29T01:29:55+02:00
    G6 debug test exit 0
    G6 Compiling lines: 0
    load after G6 5.03 7.83 9.11 3/2580 1145216
    c32f4ce21881c6b0dcc44171857f40abdb78a09b
    finished 2026-09-29T01:36:14+02:00

HEAD did not move and `git status --short` printed nothing between the two hashes. G3's and G5's
logs each carry a `Compiling` line for every workspace crate, so G4 and G6 ran binaries built from
`c32f4ce21`. The earlier close's G3 had compiled nothing (review M4).

| gate | command | exit | figures |
|---|---|---|---|
| G1 | `cargo fmt --all --check` | 0 | |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target directory | 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | 0 | 2758 passed / 0 failed / 4 ignored |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 | 2759 / 0 / 4 |

(G3 and G5 build what G4 and G6 run, outside the memory cap.) **Both failing sets are empty.** In
both runs:
- the corpus differential is **655 of 655**;
- `api_group_tests` is green: `every_test_of_the_phase_8_groups_passes_and_matches_the_oracle_but_the_recorded`,
  `the_tests_reaching_rxapi_run_on_neither_side` and the detector's own test;
- `closed_phases`'s `no_refusal_names_a_closed_phase` and `no_open_exclusions_row_names_a_closed_phase` are green;
- the gate tables print "gated by this run: 0 row(s)".

Report-mode figures, which are progress signals and not gates: the assertion table 4247 of 4259
rows, `base/bif` 4992 of 4999 value rows and 186 of 186 raise rows, `base/keyword` 892 of 896
bodies.

**What `api_group_tests` asserts now.**
- Each test of `METHOD`, `CONVERSION` and `FUNCTION` runs alone on both sides. A test whose
  outcome, assertion count, stderr or exit status differs must be in `RECORDED`, and each
  recorded test that runs must still not pass alone on this crate.
- Each group then runs whole with the recorded tests renamed out, and must pass on this crate
  (summary `pass`, exit 0) as well as agree on all three descriptors. The pass is asserted there
  because `FUNCTION`'s io tests fail alone on both sides and pass after `TEST`.
- `RECORDED` holds `FUNCTION.TEST_REXXQUEUE` (Phase 10), which runs on neither side, and
  `METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA` (Phase 9). The second runs `rexxc`,
  which is not on the gate's `PATH`, so it fails on both sides with rc 127, and agreement alone
  could not see it. `DETAIL_DIFFERS` is empty.

**The oracle and rxapi.**
- The instrument used to run `TEST_REXXQUEUE` on the oracle, alone and inside the `-S` listing
  run, which runs the whole group. That test creates a named `RexxQueue` in rxapi's daemon.
- The constraint, as the controller ruled on 2026-09-29, is never to read or change the daemon's
  persistent state: named queues, function registration, the macro space. The tests whose group
  source (or a file the group loads) uses a named queue or those registries are derived, and the
  gate fails before any oracle run unless they are exactly `RECORDED`'s not-run entries. They
  are renamed out of the listing run as well.
- What no ooTest run can avoid: the oracle asks rxapi's macro space, read-only, on every
  `::REQUIRES` and every external call (`interpreter/package/PackageManager.cpp:747`,
  `interpreter/execution/RexxActivation.cpp:2996`). That goes through `RexxQueryMacro`
  (`rexxapi/client/MacroSpaceApi.cpp:217-230`) to `LocalAPIManager::getInstance`, `initProcess`
  and `establishServerConnection` (`rexxapi/client/LocalAPIManager.cpp:56-74`, `:177-243`),
  which start the daemon when it is not running.
- The session queue: on the oracle `PUSH`, `QUEUE`, `PULL`, `PARSE PULL` and `QUEUED()` go to
  `.local`'s `STDQUE`, an rxapi queue (`interpreter/concurrency/Activity.cpp:3282`, `:3361`),
  which lives for the process. The corpus has used it since Phase 4, and by the same ruling it is
  not re-scoped. The files with those forms, from a text grep that does not exclude comments,
  run from `rust/`:

        for f in $(git ls-files corpus crates/rexx-exec/tests | /bin/grep -aE '\.(rex|rs)$|ir_recorded_cases/'); do
          n=$(/bin/grep -a -ciE '\.rexxqueue|rxqueue\(|queued\(|rxfunc(add|drop|query)|sys[a-z]*rexxmacro|^[[:space:]"]*(push|queue|pull)\b|parse[[:space:]]+(upper[[:space:]]+)?(caseless[[:space:]]+)?pull' $f)
          [ "$n" != 0 ] && echo "$f $n"; done

  It printed, at `94897d79e` with this round's detector edit uncommitted: `corpus/gate-tables/classes/rexxqueue.rex`,
  `corpus/gate-tables/hierarchy/queue__orderedcollection.rex`,
  `corpus/gate-tables/hierarchy/rexxqueue__object.rex`,
  `corpus/gate-tables/methods/rexxqueue__class.rex`,
  `corpus/gate-tables/methods/rexxqueue__instance.rex`, `corpus/lang/input_redirection.rex`,
  `corpus/lang/pull_queue.rex`, `corpus/lang/push_queue.rex`,
  `corpus/lang/required_string_contexts.rex`, `corpus/lang/required_string_default_name.rex`,
  `corpus/lang/state_builtins.rex`, `crates/rexx-exec/tests/api_group_partition.rs`,
  `api_group_tests.rs`, `collection_scopes.rs`, `input_oracle.rs`,
  `ir_recorded_cases/return-and-queue`, `prompt_before_read.rs` and `state_builtin_oracle.rs`
  (each with its count of matching lines).

**Miri is a recorded run, not a gate.** rexx-api's lib tests ran under Stacked Borrows (no
`MIRIFLAGS`), miri 0.1.0 (f7575a9da8 2026-09-24), from a scratch `RUSTUP_HOME`, on the tree at
`c32f4ce21`, the last change to `rexx-api`: exit 0, 55 passed, 0 failed, 8 ignored. The busy-context
path that change added runs only under the forged probes, not under Miri or the corpus.

**No refusal and no open exclusions row names Phase 8, or Phase 7.**
- `closed_phases.rs` now lexes each source file's string literals (comments, characters and
  lifetimes skipped, continued lines joined). It flags a literal that names a closed phase as a
  word, alone or inside a longer refusal text such as `"... is not implemented (Phase 8)"`.
- Its second test holds `phase-4-exclusions.txt`'s OWNER sentences to the same rule, unless the
  rest of the paragraph or the next one says DELIVERED, CLOSED, RE-HOMED, FIXED or RESOLVED
  before the next OWNER.
- The review's controls NC-a to NC-f, and NC-g, were re-run with predictions written first
  (`task-9-fix-report.md`). NC-g removes the Phase 7 row's DELIVERED. Each is red.
- The manual enumeration, run from the worktree root:

    # (A) every OWNER sentence naming Phase 8, NO OWNER sentences dropped
    tr '\n' ' ' < docs/superpowers/plans/phase-4-exclusions.txt | tr -s ' ' \
      | /bin/grep -a -oE '(NO )?OWNER[^.]*Phase 8[^.]*' | /bin/grep -av '^NO OWNER'
    # (B) every sentence naming Phase 8, for attribution
    tr '\n' ' ' < docs/superpowers/plans/phase-4-exclusions.txt | tr -s ' ' \
      | /bin/grep -a -oiE '[^.]*phase 8[^.]*'
    # (C) every quoted "Phase 8" outside a comment in any crate's sources
    git grep -n '"Phase 8"' -- rust/crates | /bin/grep -a '/src/' \
      | /bin/grep -avE '^[^:]+:[0-9]+:[[:space:]]*//'

At `2ae06085c` (A) printed `OWNER: Phase 5 for the rest of ::REQUIRES, and Phase 8 for EXTERNAL`
and `OWNER: Phase 8` three times, and (C) printed `rexx-api/src/layout.rs:395`, `rexx-exec/src/
dispatch/library.rs:564` and the `OPEN` list. From `fc32f74aa` on, (A) and (C) print nothing, and
the tests above now assert what (A) and (C) read.

What each Phase 8 row became. Every row was re-run against the oracle on three descriptors from
fresh directories. The probes are in
`docs/superpowers/records/2026-09-14-phase-8-surface/task-9-probes/` and `task-5-forge/probes/`.

* **Matched, so DELIVERED or CLOSED**: the EXTERNAL refusal forms and `::REQUIRES ... LIBRARY`; the
  interface members Task 5 left (`RxCalcSin(30, 2, 'Q')` is 88.916 on both); the package loader
  and unloader hooks (Task 4); and Task 5's native frame and `PROPAGATED` (its `cond3`,
  `sendthrow` and `nframe` probes are identical).
* **Fixed at the close.**
  * `da17061ba`: `DirectoryAt`, `DirectoryPut`, `DirectoryRemove` and the `StringTable` members
    call the collection's own `get`, `put` and `remove` on a collection the crate stores. That
    includes the `setMethod` side table and a `setMethod` `UNKNOWN`.
  * `84c138852`: a `Relation` and a `Bag` add under an index they already hold, since their
    contents' `put` is `addFront`. Witness: `corpus/lang/library_collection_members.rex`, which
    covers every hashed class and `StringTableRemove`.
  * `ReleaseLocalReference` keeps the handle until the call ends
    (`a_released_local_reference_still_answers`, the forge's `stale.rex`).
  * `2f7d9bd60`: a `RaiseCondition` taken by `CALL ON` has its object built at the raise. The
    native frame leads it, and the object has no `POSITION`. Witness:
    `corpus/lang/library_raise_condition_call_on.rex`.
  * `c32f4ce21`: the context-variable members, reached through a call context kept from an
    enclosing call, reach that call's caller (`outer9.rex`, `o9b.rex`).
* **Still differs, loud, no owner assigned**: a stem or compound read, or a set, through such a
  kept context of a name its activation has never bound (`o9c.rex`, rc 120). A new variable needs
  a slot, and only the top frame's slots can grow (`rust/crates/rexx-core/src/roots.rs:414-422`).
  This is raised with the controller.
* **Relabelled**: `Loud::native_method` names Phase 9 instead of the closed Phase 5 (the m4
  ruling), which re-homes `Method~new`'s third-argument refusal with it; `refusal_owner` names no
  phase for a member only an unpopulated table reaches. The stale-handle refusal names none. The
  forge's `stash.rex` still reaches it with a local handle kept from an earlier call (the oracle
  answers, this crate refuses at rc 120). A local reference protects its object only until the
  call that made it ends (`NativeActivation::createLocalReference`), so that use is outside the
  API contract.
* **Found by the close and recorded**: `LIBRARY rexxutil`, an internal package on the oracle and a
  failed `dlopen` here, to Phase 10 by D11's "the remainder in Phase 10";
  `METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA` to Phase 9; and
  `ReleaseLocalReference`'s hold-until-call-end memory growth, queued (the final review's M2).

Refusals naming Phase 5 other than `native_method`'s are unchanged; `closed_phases.rs` still
records Phase 5 as a debt deliberately absent from `CLOSED`, and the sites are listed in a queued
record for whichever phase takes them.

**The roadmap's row 8 exit criteria are met**: `testbinaries/` compile unchanged against frozen
headers (section 9), and the native-API ooTest groups this row owns pass on this crate, whole and
with every test the instrument's `RECORDED` does not name, but for tests owned by later rows:
`FUNCTION`'s `TEST_REXXQUEUE` (row 10's rxapi queue, run on neither side) and `METHOD`'s
`TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`, which runs `rexxc` and fails on both sides
here with rc 127 because none is on the gate's `PATH` (row 9: "`rexx`, `rexxc`, `rxqueue`,
`rxsubcom` ship"). Until 2026-09-29 the instrument asserted only agreement with the oracle, which
could not see the second. **L2 is still not reached**, and what blocks it is not this phase's: the framework,
unmodified, stops at its ticker, a `GUARD ... WHEN` that another activity satisfies (Phase 6),
and with `-U` it runs a group's tests and stops in `printSummary` at `RXFUNCQUERY`, which the
oracle answers through rxapi (Phase 10); both measured 2026-09-28 at `2ae06085c` with
`testOORexx.rex -f METHOD.testGroup -V 1` from a scratch copy. The Rung column reads
`L2 -> 6, 10`.
