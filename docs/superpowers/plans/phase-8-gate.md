# Phase 8 gate — the native API, L2 slice

**Assessed 2026-09-14.** The commit and the gate readings are in section 6, which was written from
the run rather than before it. The tree was clean at `d1796f69c` when the run started and nothing
touched it until the run finished.

The plan's row for this phase reads:

> `testbinaries/` compile unchanged against frozen headers; native-API ooTest groups pass.

**That sentence is not met, and this document does not claim it is.** What closed is the L2 slice,
which the plan document scopes explicitly: load a shared library, call the methods it exports, and
measure how far the framework then gets. The surface half — the function pointers the slice does
not fill, `testbinaries/`, and the six non-embedding ooTest API groups — has its own plan at
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
| every load failure reports what the oracle reports | four failure programs, three descriptors each | `corpus/lang/library_*_missing.rex` |
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
  surface at all. Corrected at `378d5613b` and `165373cf5`.

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
* **`build/lib/librxregexp.so` is the oracle's own prebuilt artifact and is never rebuilt.** It is
  the test instrument for the measured half of D5, and that measurement says the *extension* half
  of the ABI comes out binary-compatible. The *embedding* half is source-compatible by promise and
  is Phase 9's.
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
* **Three divergences the L2 walk found, none of them Phase 8's**, and **the L1 extractor gap**,
  each in the exclusions file.
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
instance 50, `Stream` instance 24, `StreamSupplier` instance 8 — which is the hazard Phase 7's own
close named in its section 4: a row whose receiver cannot be constructed compares nothing, and
`unanswered` is deliberately counted as open so that a row nothing could answer never makes the
gated count smaller.
