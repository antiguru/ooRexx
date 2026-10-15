# Phase 9b1 Task 8b report: the three remaining leaks

Status: DONE_WITH_CONCERNS. Base `efbf0fa1d`. Commits:

| sha | subject |
|---|---|
| `2d82c08d1` | Step 1: method rows keep minted identities in a map |
| `25968aeba` | Step 2: free retired behaviours no instance holds |
| `fb9b644de` | Step 3: cell holders and their collect_stress shapes |
| `1ec116aa5` | Step 3: a variable-reference cell is freed with its object |
| `9344cced8` | Step 4: one match in the mark loop, and the bench set |
| `800f2b7fa` | re-derive refusal-sites.tsv |
| `aaf302608` | suite record |

Records: `docs/superpowers/records/2026-10-14-phase-9b1/task-8b/` (`memory.md`, `cell-holders.md`,
`callgrind.md` and its tables, `suite/`). Raw `.cg` files and the binaries each step measured are in
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/t8b/`.

## What changed

### Step 1, MethodId rows

`special_methods` and `ObjectModel::natives` are `MethodRows` (`rexx-exec/src/method_rows.rs`): a
vector indexed by the id for the rows filed before `seal`, and an `FxHashMap` for the rest, pruned by
`forget_method_identity` and `prune_program_rows`. The natives table seals at the end of
`ObjectModel::build`. `special_methods` seals at the end of `bootstrap_library`, or after
`install_object_model` on the lazy path (`build_object_model`, a cold function: inlining the seal
into `object_model` outlined `object_model` and cost sendloop +2.5%). `MethodRows::get` answers a
reference (committed in Step 2), because copying the row before testing it cost alloc +0.56%.

### Step 2, retired behaviours

- `ClassGraph::free_unheld_behaviours` (rexx-classes), called from `collect_now` after the expunge
  when `has_retired_behaviours()`: it collects every `Body::Instance` handle in the heap
  (`Heap::live_objects`, new), empties and frees each retired behaviour none names, and answers the
  `(class, MethodId)` pairs a freed behaviour named that no remaining behaviour or own dictionary
  names.
- `Interp::free_unheld_behaviours` (dispatch.rs) then lets the class go of those identities'
  `Method` objects (`class_disowns`, the inverse of `class_owns`), unless an activation on any
  activity is running that body (`Activity::running_bodies`, matched by program and directive).
- The mutators that retire a behaviour (`define` with and without a method, `delete`,
  `defineMethods`) charge it at 128 bytes per method entry and collect if that makes one due
  (`charge_retired_behaviours`). Without the charge a `define` loop allocates too little to collect.
- `keep_retired_behaviours` drops the image's retired lists, unfreed, at the end of the library
  bootstrap. The library retires behaviours on classes not yet marked `REXX_DEFINED`; freeing them at
  the first collection cost churn +2.4%.

The disowning is wider than the review's sketch, which named only the behaviours. Without it the
`define` loop still grew by about 0.5 kB per iteration, because `hold_method_object` makes the class
own every scoped copy `define` makes.

### Step 3, variable-reference cells

A cell is a `Body::Cell(index)` heap object (rexx-core). `Target::Cell` carries the object, the index
and a generation. The storage stays in `RootSet::cells`, so `frame_slot` and its siblings keep their
signatures. A first version kept the value in the object and passed `&Heap` to the accessors, and
cost assign +4.88% (`callgrind-step3-heap-accessors.txt`). The object is traced from `Body::VarRef`
and from alias entries in every `ActivityRoots` (`ActivityRoots::iter`). `Heap::collect` marks a
cell's value with its object, in the mark loop and the resurrection pass, and reports freed cells in
`CollectStats::freed_cells`. `collect_now` releases them (`RootSet::release_cells`), moving the
generation on, so a holder the collector did not see panics on its next read. `Interp::promote_slot`
reserves a cell, allocates its object and promotes.

The holder enumeration (the grep is committed in `cell-holders.md`) found VarRef bodies, alias
entries in the running, idle and `REPLY`-continuation `ActivityRoots`, and a VarRef reachable only
from an object awaiting `UNINIT`. Every other `SlotRef` is a Rust local inside one step while one of
those holds the same cell. Nothing needed a design change.

### Step 4

The mark loop now dispatches `Text`, `WeakRef`, `Cell` and the rest in one match. Step 3's separate
cell check cost churn +0.31%. After the change churn is +0.23% against the pin, and base was +0.42%.

## Loop witnesses

`/usr/bin/time -f %M`, `memloop.sh`, oracle beside. Full tables in `memory.md`.

| program | n | oracle | base | head |
|---|---|---|---|---|
| copypriv | 20000 / 80000 / 320000 | 12728 / 19536 / 20188 | 55328 / 58796 / 71712 | 55584 / 56560 / 57004 |
| nativecopy | same | 12188 / 19536 / 20572 | 54292 / 57404 / 68868 | 54436 / 55340 / 55532 |
| methnewpriv | same | 20128 / 20384 / 20324 | 46104 / 48608 / 55400 | 45268 / 44912 / 45240 |
| define | 5000 / 20000 / 80000 / 320000 | 20352 / 20668 / 20308 / 20300 | 57068 / 170352 / 628364 / - | 57224 / 93156 / 92376 / 92260 |
| varref3 | 20000 / 80000 / 320000 | 20092 / 20328 / 20388 | 44972 / 112172 / 386440 | 47328 / 49580 / 50260 |

The Step 1 and Step 2 figures are those steps' binaries, and the Step 3 figures are the Step 3
binary. `define` at 5000 sits below its first collection.

## Tests

New tests, each shown load-bearing by a mutant that turned it red (commands and red sets in the
records, each mutant reversed by an exact edit and checked with `grep -c`):

- `run::tests::minted_method_identities_do_not_grow_the_method_rows`: red with `seal` a no-op.
- `run::tests::a_collection_frees_retired_behaviours_no_instance_holds`: red with the pass removed
  from `collect_now`, and red with nothing counted as held.
- `run::tests::a_class_lets_go_of_methods_only_freed_behaviours_named`: red with `class_disowns`
  removed (16 to 217 `executable_sources` rows).
- `collect_stress`: `a_retired_behaviour_outlives_none_of_its_instances` and
  `a_finalized_instance_keeps_its_retired_behaviour` (red with nothing held),
  `a_replaced_method_keeps_its_object_while_it_runs` (red with the running check removed: `exe 0 1
  1 new` against `exe 1 0 1 new`), and the varref shapes in `cell-holders.md`
  (`a_reference_outlives_its_procedure`, `a_reference_argument_aliases_the_callers_variable`,
  `a_reference_in_a_collection_outlives_its_procedure`,
  `a_variable_another_activity_references_survives`, `a_reference_outlives_its_dropped_variable`,
  `a_finalized_objects_reference_keeps_its_variable`). Each was measured against the oracle 3 of 3
  with the 300 x `copies('x', 100000)` filler before the marking changed.
- rexx-core: `collect.rs` `a_cells_value_lives_as_long_as_its_object`, `a_collected_cell_is_refused`;
  `roots.rs` cell tests reworked for `promote(frame, index, object, cell)`.

The varref shapes' first version used literal values, which are interned, and stayed green with cell
values unmarked. They now hold values `copies` computes.

## Checks, at the commit named

| commit | command | exit | result |
|---|---|---|---|
| `aaf302608` | `cargo fmt --all --check` | 0 | |
| `aaf302608` | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | the code is `9344cced8`'s |
| `800f2b7fa` | `cargo test --release -p rexx-exec --lib` | 0 | 1134 passed |
| `800f2b7fa` | `cargo test -p rexx-exec --lib` (debug) | 0 | 1134 passed |
| `9344cced8` | `cargo test --release -p rexx-exec --test collect_stress` | 0 | 70 passed |
| `800f2b7fa` | `cargo test -p rexx-exec --test collect_stress` (debug) | 0 | 70 passed |
| `800f2b7fa` | `cargo test -p rexx-core` (debug), `--release -p rexx-core -p rexx-classes -p rexx-api` | 0 | `unsafe_sites` included |
| `9344cced8` | `cargo test --release -p rexx-exec --test refusal_sites` | 101 | stale `lib.rs` line numbers, below |
| `800f2b7fa` | `--test representation_refusals --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test class_lifetime --test class_weak_reference --test package_requires` | 0 | |
| `800f2b7fa` | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 29 passed, 1 ignored |
| `800f2b7fa` | suite, Task 8's groups plus `base/keyword/VarRef`, `USE`, `EXPOSE`, `REPLY` | 101 | 816 rows: one changed against close2 (`Package` `TESTRECLAIM01` blocked to passed), none against Task 8 |
| `9344cced8` binary | `callgrind.sh -B 0.5 -j 16`, full set in two halves | 0 and 0 | every program within budget: churn +0.23%, startup +0.33%, interpret +0.29%, sayloop +0.24% |

All commands ran with `CARGO_INCREMENTAL=0 memcap 8G -j 4` and a `timeout` under ten minutes.

## Concerns

1. **Files outside the brief's list.** Step 2 is in rexx-classes (`class_graph.rs`, `registry.rs`,
   `method_dict.rs`), where the retired-behaviour state lives. The controller ruled the brief's file
   list incomplete and approved this (2026-10-15). rexx-classes is covered by the workspace clippy
   at `aaf302608` and by `cargo test --release -p rexx-classes` at `800f2b7fa`, both exit 0.
2. **Step 2's disowning is a new holder rule.** A class keeps a `Method` object `define` installed
   only while a behaviour names its identity or an activation runs its body. The running check
   matches by program and directive, so it errs toward keeping.
3. **`RootSet::cells` stays a numeric-index table.** Each row is pruned when its `Body::Cell` object
   is swept (state 2 in spirit), and a stale read is refused by generation. It is not a field of the
   object. That needs a ruling against the global constraint, as does `MethodRows`' map, which the
   brief cites as ruled 2026-10-15.
4. **refusal-sites.tsv was stale from `2d82c08d1` to `9344cced8`.** Step 1's module declaration
   moved the `lib.rs` Loud constructors, and I re-derived the table in a separate commit rather than
   in the one that moved them.
5. **Found, not fixed:**
   - The oracle segfaults on assigning through a reference to a returned procedure's variable
     followed by a collection, 3 of 3. I added it as `rust/corpus/oracle-crashes.txt` entry 49, a
     shared file.
   - On the crate, base included, `.context~executable` asked first after the running method was
     redefined on its class answers the template `.k~method('BODY')`'s flags (`exe 0 1 1` against
     the oracle's `exe 1 0 1`, no collection involved). Probe: `.k~define('FOO',
     .k~method('BODY')~copy~~setPrivate~~setUnguarded)`, then `.s~new~go` with `go` sending `foo`,
     and `BODY` redefining `FOO` before it asks.
   - A finished program's `RESULT` stays reachable between programs on one `Interp`: a unit test
     saw `.local~remove('OLD')`'s answer keep an instance alive until `drop result`.
6. `varref.rex` (`use arg >z`) is +14.3% from 20000 to 80000 and flat from 80000 to 320000. That is
   where the first collection falls, not growth. The brief's witness, `varref3`, is +6.2%.

## Fix round 1 (`c44f4cf7b`)

- m3: `rust/corpus/oracle-crashes.txt` entry 49 gains the review's instance-variable shape and its
  probe path. I did not run it on the oracle. `collect_stress`
  `an_instance_variable_reference_outlives_its_object` pins the crate's answer (`p10a 26 ovo`,
  `p10b 14 nwn`, `p10c 26`), crate-only. The review's probe run on `rexx-run-step4a` answers the same.
- m6: `MethodRows::get_mut` and `remove` debug-assert that no map row has an id the vector covers.
  The check is placed so the vector-covered case is the one tested. New unit tests in
  `method_rows.rs`:
  - `a_minted_row_below_the_vector_is_refused` goes red with the assertion made `true`.
  - `a_sealed_table_files_by_the_vectors_reach` goes red when a sealed insert always goes to the map.
  Neither existing test reached the invariant: that mutant left `run::tests` green.

| commit | command | exit | result |
|---|---|---|---|
| tree of `c44f4cf7b` | `cargo fmt --all --check` | 0 | |
| tree of `c44f4cf7b` | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | |
| tree of `c44f4cf7b` | `cargo test -p rexx-exec --test collect_stress` (debug) | 0 | 71 passed |
| tree of `c44f4cf7b` | `cargo test -p rexx-exec --lib -- method_rows` (debug) | 0 | 3 passed |
