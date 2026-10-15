# Phase 9b1 Task 8 report: the side-map sweep

Status: DONE_WITH_CONCERNS. Base `d965c149a`. Commits:

- `93e7a9a4c` the Task 8 refusal rows (Step 4)
- `900fbfd8e` the side-map sweep's fixes (copy identities, retired behaviours, chunk literals, `Class~new` charge)
- `6a4327069` reuse a collected class's behaviour slots
- `882c826eb` prune freed classes outside `collect_now` (codegen, see Budget)
- `d81bacfad` records, `side-maps.tsv`, four stale queued items closed

This report is not committed. Records: `docs/superpowers/records/2026-10-14-phase-9b1/task-8/` and
`docs/superpowers/records/2026-10-14-phase-9b1/side-maps.tsv`. Raw `.cg` files:
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-8-{a,b,c,d}/`.

## Step 1: enumeration

`task-8/enumerate-maps.sh` (run from `rust/`) prints every map, set and vector field (byte buffers
aside) of `Interp` and what it holds by value (`Programs`, `ObjectModel`, `EnvironmentModel`,
`Libraries`, `PackageTable`, `Activities`, `GuardTable`, `Semaphores`, `Activity`), `PackageData`,
`Heap`, `RootSet`/`ActivityRoots`, `ClassRegistry`/`ClassGraph`/`ClassDef`/`MethodDict`, and the
`rexx-api` handle tables. `task-8/side-maps-build.py ENUMERATION OUT` classifies each row and exits 1
if a row is unclassified or a classification names no row; its output is `side-maps.tsv`, one row per
field, columns site, struct, field, type, the object it describes, key, and state with the pruning
site. Re-run at `882c826eb`: identical to the committed table's input.

## Step 2: fixes

**Method copy identities (Task 7 concern 1).** The ownership rule, stated at
`ExecutableRecord::owns_installed` (`lib.rs`): an identity minted for one `Method` object alone (a
`~copy`, a `newScope` copy, a subclass instance copy, a `loadExternalMethod` answer) belongs to that
object, and its rows go at the sweep that frees it. It is safe because every dictionary that installs
such an identity holds the object (a class's `owned` list through `hold_method_object`, an
`ObjectMethod`'s `executable`, a `run` activation's `executable`, and the object `.context~executable`
answered). A sentence here claiming an activation reads none of its identity's rows was false; see fix round 1. `prune_dead_object_rows` calls
`forget_method_identity` (`dispatch.rs`, beside `copy_method_identity`) for each swept owner, which
drops the identity's rows in every MethodId table, `natives` and `special_methods` included. Task 7's
loaded-method special case is subsumed. Witnesses: `run::tests::a_collection_drops_the_identity_rows_of_swept_method_copies`,
`collect_stress` `a_dropped_method_copy_keeps_running` (a self-unsetting copy, a private copy, a
primitive copy, a `run` copy, an old instance's behaviour after `define`, fresh private copies) and
`a_swept_rebuilt_executable_leaves_its_identity_alone`. Mutations M1-M4 in `task-8/mutations.md`: the
`owns_installed` test, the class hold, the `ObjectMethod` trace and (fix round 1) the `run` activation's
hold are each load-bearing. Memory: `copy` 55168 / 56268 /
56220 kB at 20000 / 80000 / 320000 (+2.0%; base 54912 / 57592, oracle 12288 / 19816).

**A class's retired behaviours.** Every `define` (one per `::METHOD` installed, too) gives the class a
fresh instance behaviour; `ClassGraph::expunge` emptied only the last two, so each collected class left
a full dictionary copy per method behind. `ClassDef::retired_behaviours` records them and `expunge`
empties them. A one-class two-method `Package~new` loop went from 325044 / 1174604 kB to 53780 / 53012
/ 53068 (oracle 28480 / 58344).

**Behaviour slot reuse.** `ClassGraph::free_behaviours`: an expunged class's emptied slots are handed
out again. Only `Body::Instance` holds a `BehaviourHandle` outside the graph, and an instance traces its
class. Witnesses: `behaviour_wiring` `expunge_empties_the_behaviours_define_retired` and
`a_new_class_reuses_an_expunged_classs_behaviours`, `collect_stress`
`a_reused_behaviour_slot_leaves_live_instances_alone`. `cm4`, `cm1`, `constmeth`, `cn3` and `sub3`
are flat to 320000.

**Chunk literals.** `interned_literal` allocated a long literal immortal for the run, so every
`Method~new`/`Package~new` program, every `INTERPRET` fragment and every literal `::CONSTANT` kept its
long literals forever. `Heap::release_immortal` (`rexx-core`) makes them ordinary objects again:
`prune_program_rows` releases a freed program's chunks' values, `run_fragment` its own chunk's, and a
literal `::CONSTANT` is now a plain allocation held by its package's `constants` field. `litmeth2`
54888 / 73136 → 50016 / 50856 / 50716; `interp` 36396 / 53464 → 36544 / 36588 / 36552. Witnesses:
`run::tests::an_interpreted_literal_is_released_with_its_chunk`,
`a_freed_programs_literals_are_released_with_it` (L1-L3 red), `collect_stress`
`a_held_literal_outlives_its_chunk`.

**`Class~new` charge.** A class `Class~new` made was collected but never charged to the trigger:
`.class~new` in a loop reached 984 MB before its first collection, clearing a WeakReference at
iteration 65184 (oracle 871). `native_new_class` now calls `charge_class` as `~subclass` does. Witness
`corpus/lang/class_new_collected.rex` (in `phase-9.txt`, sourceline file from the driver on a scratch
copy, oracle `class cleared: 1` 3 of 3, base `0`), pinned by `class_lifetime::a_loop_of_class_new_collects_its_classes`.

The full memory table, with the oracle and the base beside the head, is `task-8/memory/memory.md`.

### Reported, not fixed

1. **`RootSet::cells` (`rexx-core/src/roots.rs`).** A variable a `>name` reference was taken to moves
   to a cell that is never freed, and every cell's value is a root. `varref3` (`r = >y` in a procedure
   over a 1000-byte value): 46652 / 112748 / 385864 kB, oracle 20340 / 20328. `varref` (`call q >y`):
   24200 / 33564 / 40688. Freeing a cell needs the collector to mark cells from the `VarRef` bodies and
   frame aliases that name them, a change to `rexx-core`'s marking. This is the largest leak found.
2. **A live class keeps every behaviour `define` retired.** `.k~define`/`.k~delete` in a loop keeps a
   full dictionary copy per call while the class lives: `define` 57644 / 170956 kB at 5000 / 20000,
   oracle 20328 / 20460. An instance made before a `define` may hold any of them, so freeing them needs
   the collector to report which handles live instances hold.
3. **`special_methods` and `natives` are vectors indexed by MethodId**, whose length follows the newest
   identity holding a row; MethodIds are never reused. A loop that keeps minting private or primitive
   copies grows about 50 bytes per identity: `copypriv` 55284 / 59116 / 70724 kB, `nativecopy` 54452 /
   57472 / 71760 (within 10% at 80000). Bounding them needs either MethodId reuse (a new identity rule)
   or a map beside each vector, which the global constraints forbid without a ruling.

## Step 3: the queued clock and SETLOCAL items

Both stale and closed (`git rm`, `d81bacfad`). Probes `task-8/scope-probes/`, 3 runs per side on
`882c826eb`, all 3 of 3 agree (stdout as a multiset, stderr, rc): the queued item's own `c1`
(routine starts its own clock) and `c2` (an internal `TIME('R')` leaves the caller's read clock),
method clocks, both REPLY seams (`c4` the method's own clock across `REPLY`, `c5` the continuation
untouched by main's reset), `RANDOM` seeds across a method and across `REPLY`, and `SETLOCAL` across a
method and across `REPLY` (`s2`: `cont endlocal 1`, `main endlocal 0`, the oracle's order). A shape
with a routine's `SETLOCAL`, then an internal routine's `SETLOCAL` and the caller's `ENDLOCAL`,
aborted the oracle (`free(): invalid pointer`, rc 134, 3 of 3); it is `oracle-crashes.txt` entry 10's
family and is not in the records. No pre-6.1 binary was run, so the probes' power rests on `c1`/`c2`
being the queued item's verbatim probes, whose recorded answers differed at the time.

## Step 4: the Task 8 refusal rows

- `context.rs`'s "a stack frame this crate did not build" (three literals) is reachable: a StackFrame
  method `run` or `setMethod`-installed on an object of another class. The oracle segfaults
  (`oracle-crashes.txt` entry 33 already lists `.StackFrame~method('NAME')`), so this is Deviation 28's
  shape: it now refuses as "a value that is not a stack frame", and
  `dispatch::tests::a_native_row_on_a_receiver_of_another_class_refuses_its_type` gained `NAME` and
  `EXECUTABLE`. The row is gone.
- Retagged "internal guard", with the reason beside each row in `representation_refusals.rs`: the
  four freed-handle guards and Task 5's package guard (only a rooting defect reaches a swept handle),
  `construct.rs`'s body-shape guard (its callers pass only what a `Method`, `Routine` or `Package`
  constructor answered), and `object_protocol.rs`'s StackFrame `objectName` arm (`receiver_kind`
  answers `StackFrame` only for a native body of that class). Probes `sf.rex`/`sf2.rex` (subclass and
  `new`, frames from programs, internal calls, routines, methods, interpret, conditions, a sort
  callback, a started method, a copy) reached none of them; their only differences from the oracle
  are the queued native-method-frame and routine-name items. "Task 8" is gone from `OWNERS`, replaced
  by "internal guard". `refusal-sites.tsv` re-derived: unchanged.

## Queued class-collection item

Stale and closed with `2026-10-12-gc-trigger-blind-to-class-weight.md`. `task-8/class-shapes/shapes.rex`
clears a dropped class on the oracle and the head for every shape: `~subclass`, `Class~new`,
`~mixinclass`, a metaclass, a subclass of a subclass, a class with `UNINIT`, `Package~new`, a
`::REQUIRES`d file's class, `Routine~new` with a `::CLASS`, a class whose instance was dropped, and a
`Class~new` class's subclass. The base fails `Class~new` within 20000 iterations, the charge above. No
class shape is never collected. The trigger item's clauses are all measured flat now (`sub3`, `cm4`,
`pcopy` in `memory.md`).

## Budget

`timeout 590 bench-programs/callgrind.sh -B 0.5 -j 16 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-8-d pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t8h`
at `882c826eb`: exit 0, full set, 3 rounds, no OVER, no SPREAD. Whole-process: startup +0.3665%,
churn +0.3919%, pingguard +0.3476%, sayloop +0.2634%, decloop +0.2190%, hashnew +0.1166%,
compound +0.1065%, alloc -0.7307%, sendloop +0.0720%. Records `task-8/callgrind/`.

At `6a4327069` the same run exited 1, churn +0.6550% (`callgrind/churn-codegen/`). The difference was
inside `collect_now` (+1.76 M Ir over the same 29 collections, in inlined `heap.rs`/`body.rs` lines)
in a program that frees no class: inlining `expunge`'s new push perturbed the sweep's codegen.
`882c826eb` moves the class block to a cold, non-inlined `prune_freed_classes`.

## Checks (from `rust/`, `CARGO_INCREMENTAL=0 -j 4 timeout 590 memcap 8G`, statuses unpiped)

| commit | command | exit | result |
|---|---|---|---|
| `882c826eb` | `cargo fmt --all --check` | 0 | |
| `882c826eb` | `cargo clippy -p rexx-exec -p rexx-classes -p rexx-core --all-targets -- -D warnings` | 0 | |
| `882c826eb` | `cargo test --release -p rexx-exec --lib` | 0 | 1131 passed |
| `6a4327069` | `cargo test -p rexx-exec --lib` (debug) | 0 | 1131 passed |
| `882c826eb` | `cargo test --release -p rexx-exec --test collect_stress` | 0 | 60 passed |
| `882c826eb` | `cargo test -p rexx-exec --test collect_stress` (debug) | 0 | 60 passed |
| `6a4327069` | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 29 passed, 1 ignored |
| `6a4327069` | the same, debug | 0 | 29 passed, 1 ignored |
| `6a4327069` | `cargo test --release -p rexx-exec --test representation_refusals --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test class_lifetime --test class_weak_reference --test package_requires` | 0 | 9 binaries ok |
| `6a4327069` | `cargo test --release -p rexx-classes -p rexx-core` | 0 | `unsafe_sites` included |
| `6a4327069` | `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 | 1 passed |
| `900fbfd8e` tree | `REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites` | 0 | `refusal-sites.tsv` unchanged |
| `882c826eb` | suite, 12 groups (below) | 101 | see below |

```
REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Class,base/class/Method,base/class/Object,base/class/Package,base/class/StackFrame,base/class/WeakReference,base/directives/CLASS,base/directives/CONSTANT,base/directives/METHOD,base/keyword/INTERPRET,base/class/Routine,base/class/Message REXX_SUITE_TABLE=<records>/task-8/suite/table.tsv CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite -- --nocapture
```

701 tests in 12 groups. Against the 9b close2 `tests.tsv`, one row changed: `base/class/Package`
`TESTRECLAIM01` blocked (timeout) to passed. Against Task 7's record, identical on the shared rows. The
remaining failures are the ones close2 records.

`882c826eb` only moves a block of `collect_now` into a function; the corpus, tables and debug lib runs
were not repeated after it.

## TDD evidence

The tests were written after the code. `mutations.md` is the evidence each is load-bearing: every fix
has a mutation that turns its test red (M1-M3, L1-L3, B1, R1); the witness program is red on the base
binary (`class cleared: 0`).

## Files outside the brief's list

`rexx-core/src/heap.rs` (`release_immortal`), `rexx-classes/src/class_graph.rs` (retired behaviours and
the free list), `ir.rs` (`Chunk::interned_values`), `run/interpret.rs`, `install.rs`, `dispatch.rs`,
`class_protocol.rs`, `identities.rs`, `dispatch/context.rs`. Each is the reader or owner of the table it
fixes.

## Concerns

1. `RootSet::cells` leaks every referenced variable's value (reported item 1): 385 MB at 320000
   iterations of `r = >y`, oracle 20 MB. Needs a marking change in `rexx-core`.
2. A live class's retired behaviours (item 2) and the MethodId-indexed vectors (item 3) still grow.
3. startup is at +0.3665% against the pin (headroom 0.13%), churn +0.3919%, pingguard +0.3476%.
4. Behaviour slot reuse rests on "only an instance holds a handle, and it traces its class"; a future
   holder of a `BehaviourHandle` that does not hold its class would dispatch through a reused slot.

## Fix round 1

Commits: `5e53d7bd7` (I1), `57abaeeb5` (m1 and the bench program), `27c7e4c6c` (m2), `b3619002a` (m3),
and the records commit that follows this report.

- **I1.** `ExecutableRecord::owns_installed`'s doc now names the holders (a class or object method
  dictionary, a `run` activation's `executable`, the object `.context~executable` answered) and no
  longer says an activation reads none of its identity's rows. New `collect_stress`
  `a_run_activation_holds_the_copy_it_runs`: `self~run(.k~method('RUNNER')~copy~~setPrivate)` with
  nothing else holding the copy, a filler, then `.context~executable`'s `isPrivate isGuarded`. Oracle
  `exe Method 1 1`, 3 of 3. Under M4 (the hold at `object_protocol.rs` replaced by `let _ =
  executable;`) it is red: `exe Method 0 1`. `mutations.md`'s M4 row is corrected.
- **m1.** `Heap::release_immortal(from, released)` filters only the immortal objects interned since
  the count `from`; `run_fragment` takes that count before running its chunk, so a pass costs the few
  objects it interned. `prune_program_rows` passes 0 with an `FxHashSet`. Two variants measured on the
  way, recorded here because each moved churn: a pending list on `Heap` filtered at the next collection
  put churn at +1.96% (the new `Heap` field changed `collect`'s codegen; moving the filter out of line
  did not help), and the same list on `Interp` left churn at +0.42% but interpret at +0.95%. New bench
  program `bench-programs/interpret.rex` (20000 passes of a fragment with a 27-byte literal), listed in
  `callgrind.sh`, `rexx-bench`'s `PROGRAMS` and the suite's axes, with a README row. L1 is still red on
  `an_interpreted_literal_is_released_with_its_chunk` (3 against 1).
- **m2.** `copy_instance_behaviour` skips the retired list for a class in the image (`rexx_defined`),
  which is never collected and on which a program's `define` is refused 98.985.
- **m3.** Deviation 28's entry in `phase-4-exclusions.txt` names StackFrame with the review's figures.

Callgrind, `timeout 590 bench-programs/callgrind.sh -B 0.5 -j 16 -p "startup churn compound interpret"
-o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-8-fix1-d pin=.../rexx-run base=.../rexx-run-9b1-t8h
head=.../rexx-run-9b1-t8l`, head at `27c7e4c6c`'s code (sha256 7df287ed5fe54e44...): exit 1 only for
`base`'s interpret; head startup +0.3385%, churn +0.4179%, compound -0.0106%, interpret +0.4785%
(-0.1046% with libc subtracted). Records `task-8/fix1/`.

interpret's whole-process figure is mostly glibc malloc work its own code does not account for. The
same program on earlier binaries (`fix1/interpret-history-table.txt`): close2 +0.6408%, t5c +0.7255%,
t7b +0.3367%, t8base +0.3293% against the pin, each within 0.012% of it with libc subtracted. Its
+0.4785% has little headroom for that reason.

Checks (`CARGO_INCREMENTAL=0 -j 4 timeout 590 memcap 8G`, unpiped):

| commit | command | exit | result |
|---|---|---|---|
| `27c7e4c6c` | `cargo fmt --all --check` | 0 | |
| `27c7e4c6c` | `cargo clippy -p rexx-exec -p rexx-classes -p rexx-core -p rexx-bench --all-targets -- -D warnings` | 0 | |
| `27c7e4c6c` | `cargo test --release -p rexx-exec --lib` | 0 | 1131 passed |
| `27c7e4c6c` | `cargo test --release -p rexx-classes` | 0 | |
| `27c7e4c6c` | `cargo test --release -p rexx-bench` | 0 | |
| `b3619002a` | `cargo test --release -p rexx-exec --test licensed_divergences --test closed_phases --test refusal_dispositions --test coverage --test builtin_status --test bif_assertions` | 0 | |
| `b3619002a` | `cargo test --release -p rexx-exec --test collect_stress` | 0 | 61 passed |
| `b3619002a` | `cargo test -p rexx-exec --test collect_stress` (debug) | 0 | 61 passed |

INTERPRET itself is 0.45 s on this crate and 0.09 s on the oracle for `interpret.rex`, a gap no task
owns.
