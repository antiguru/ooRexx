# Task 23 report: the instruments for criteria 4 to 7

Base `811610f63`. Code commit `0ac73b804`; records, spec amendments and the S5 gate section in the
commit after it.

## Criteria 4 and 5

- P71's command run at `0ac73b804`; output committed as
  `docs/superpowers/records/2026-10-01-phase-6-s2-s5/criterion-4-inventory.txt`. Compared with the
  audit's `s5-evidence/audit/proposed-pattern.txt` with line numbers stripped, the only difference
  is island.rs's grant line (now `unsafe impl<T: IslandPayload> Send`). Every row of the audit's
  table C re-checked at `0ac73b804`; moved lines: `island.rs:38` to `:52`, `lib.rs:1223` to `:1238`,
  `lib.rs:3124` to `:3151`, `scheduler.rs:627/:658/:713` to `:633/:664/:723`, `frame.rs:88/:139`
  to `:117/:168`, `body.rs:171` to `:177`.
- A1 (`frame.rs:90`, `:99`, control `:108`), A2 (`body.rs:138`), A3 (`island.rs:130-138`), A4
  (`scheduler.rs:171`), A5 (`island.rs:31`, `:52`). Clippy `-D warnings` on A3: clean, in every
  feature combination below.
- A3 control, `let _ = <u64 as AmbiguousIfSend<_>>::some_item;` added to the `const _`
  (`cargo build -p rexx-exec`):

  ```
  error[E0283]: type annotations needed
     --> crates/rexx-exec/src/island.rs:138:13
  ```

- A5 control, a third payload `Islanded::new(1u64, ..)` required to be `Send`
  (`cargo test -p rexx-exec --lib --no-run`):

  ```
  error[E0277]: the trait bound `u64: IslandPayload` is not satisfied
  help: the trait `IslandPayload` is implemented for `i32`
  ```

  `impl IslandPayload for i32` is `#[cfg(test)]`, for island.rs's own two `should_panic` tests,
  whose payload is `1`.
- Both controls were temporary edits, restored by copying the saved file back (`cmp` clean).
- P73: spec 2.5 and section 5 amended (two sentences). P74: in the criterion 5 row.

## Criterion 6

- `sharing` on `rexx-exec`, forwarding a `sharing` feature on `rexx-core`. Deviation from the
  brief's "a counter in rexx-exec": the tags live in `rexx-core`'s `Heap`, because resolution is
  `Heap::get`/`get_mut`/`body_text` and rexx-exec reaches objects through those at every site. The
  private `Heap::resolve` is not tagged, so the collector's own walks do not count as touches.
  Activity tags come from `Heap::sharing_tag` at `Interp::new_activity`, and the heap's current
  tag is switched in `switch_to` and `swap_running` (not in `swap_idle`, which reads an idle
  activity on the running one's behalf).
- Witnesses: `concurrency_tests` `sharing::one_activity_shares_nothing` and
  `sharing::an_object_read_by_a_started_activity_is_shared` pass. With both `share_as` lines in
  scheduler.rs replaced by `let _ = ();`, the second fails and the first passes (debug run,
  restored after, `grep -c share_as` back to 2).
- Zero cost when off: `docs/superpowers/records/2026-10-01-phase-6-s2-s5/sharing-off-text-hash.txt`.
  The first comparison (base vs head) differed; a determinism control (base built twice) matched,
  and a third tree (base plus A1-A5 only) matched head, so the sharing code adds no byte. Base and
  head list the same functions with the same sizes apart from `Lent::wait`'s symbol name, whose
  impl index A5's impls renumber; that the placement difference comes from it is not shown.
- Figures (records `sharing-corpus.md`, `sharing-derived.md`): corpus 1088 shared of 520530
  objects; derived list 790 of 374510. "ooTest" is criterion 1's derived list, the in-process
  ooTest runner that exists; at this head the record's outcome column includes refusals,
  failures and an error, which are criterion 1's to settle.

## Criterion 7

- `pingpong/pingmsg.rex`, `pingsem.rex`, `pingguard.rex` under `rust/bench-programs/`, in a
  subdirectory: `rexx-bench`'s two list tests count every `.rex` directly in `bench-programs/` as
  a criterion or suite axis, and failed when the programs sat there. `wallclock.sh` gained a
  `PROGRAMS` override and names its output files with `/` spelled `_`.
- 9 interleaved rounds, load in `pingpong/binaries.txt`; 30 runs per side for output stability.
  The first run (programs at the top level, `0ac73b804`) was replaced by the run at the new paths;
  the `rexx-run` binary's sha256 is the same in both.

## Checks

At `e57dc8315` (`/tmp` logs, not kept): fmt 0; clippy `-D warnings` 0 with no feature, with
`rexx-exec/sharing`, and with `sharing,pinning`; `cargo test -p rexx-core --doc` 0. `cargo test
--workspace --release --no-fail-fast` under `memcap 8G`: the first attempt was OOM-killed while
compiling (memcap covered the build); built first, then run: 3033 passed, 3 failed: the two
`rexx-bench` list tests (above) and `refusal_sites` `the_table_holds_every_constructor_the_source_defines`
(lib.rs line numbers moved by `SharingReport`). `corpus/refusal-sites.tsv` re-derived by
`REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites -- --test-threads=1 the_table_holds_every_constructor`;
only the definition column changed (`diff` of every other column empty). After the fix:
`rexx-bench` and `refusal_sites` pass.

At `300aa2695`: `cargo test --workspace --release --no-fail-fast` (built first, then run under
`memcap 8G`) exits 0, 3036 passed and 0 failed (sum of `test result` lines).

## Concerns

- The sharing tags live in `rexx-core`'s heap behind a forwarded feature, not in `rexx-exec` alone.
- "ooTest" for criterion 6 is criterion 1's derived list, not every ooTest test.
- Clippy and fmt were last run at `e57dc8315`; `300aa2695` changes no Rust source.

## Fix round 1

Code commit `d9e17e5c8`; records and gate in the commit after it.

1. C1: the grant's SAFETY note (`island.rs:36-55`) names `HostRef`'s `Deref`/`DerefMut` through
   `Island::host` beside `Lent::interp`, and the abandoned call's off-baton drop of the box.
   Checked: `OffBaton` is `HeldCall` (`Held` = optional stub fn pointer + `Arc<Mapping>`, plus a
   `MethodId` or a `c_int`), `NativeCall` (words) and `CStringPool` (`ObjRef` words, boxed and
   vector bytes): no `Rc` or `Cell`. The `ThreadContext` is dropped explicitly (`drop(thread)`)
   while `ended`, the recall's lend, is live, so no change was needed there.
2. C2: `clear_uninit_all` filters with `resolve` and a slot match. Witness
   `sharing::uninit_objects_another_activity_never_names_are_not_shared`: with the old
   `self.get(r)` filter restored temporarily it fails, `program: SharingCount { objects: 112,
   shared: 102 }`; with the fix it passes. Restored by copy, `cmp` clean.
3. C3: two debug-only read paths found by a temporary backtrace on each new shared mark, diffed
   between a debug and a release `rexx-run` on `lang/method_reply.rex`: `output_route`'s
   `#[cfg(debug_assertions)]` re-derivation and `directory_get`'s `debug_assert_eq!` re-reading a
   `StoreView`. Both now run with touches paused. A first version used a bool and the nested pause
   (route, then a directory read inside it) cleared it early, which showed as new marks; the pause
   is a depth now. Corpus table in a debug build and in a release build at `d9e17e5c8`:
   byte-identical (`diff` empty).
4. C4: `SharingReport { bootstrap, program }`, each a `SharingCount { objects, shared }`.
   `Heap::sharing_program_starts` at the bootstrap's end marks later slots as the program's.
5. C5: `-x ORACLE_PROGRAMS`.
6. F1: populations measured at `d9e17e5c8`: corpus; every `.testGroup` run whole in process (new
   test `sharing::sharing_fraction_over_every_ootest_group`, skipping groups `reaching_rxapi`
   lists); criterion 1's derived list; the keyword, bif and expression assertion harnesses
   (through a `REXX_SHARING_LOG` line per run, written by `watchdog::log_sharing`, which
   `run_bounded_with` and `keyword_assertions::evaluate` call); and `api_group_tests` under its gate
   with the same log, because the every-group run skips `API/oo/FUNCTION` whole while
   `api_group_tests` runs its tests that do not reach rxapi. The
   every-group run at the default 32 rayon threads was OOM-killed at the 8G cap after logging
   386 runs; with `RAYON_NUM_THREADS=4` it completes. Each log's sums equal its table's totals.
7. F2: the P74 clause names both producers of `NativeState::Pointer`: `new_pointer`
   (`dispatch/library.rs:1428`) and `handle_object` (`dispatch/time_support.rs:201`).
8. F3, F4: `ffi.rs:1320` dropped; island and signal test threads added to the test row and the
   signal row; "test statics". Inventory re-run at `d9e17e5c8`: same line set as before but four
   moved lines (island.rs, lib.rs, watchdog), record replaced.
9. F5: "(rulings P52, P72)". F6: no count.
10. F7: `thirty-runs.sh` takes the binary as `$1` and runs it from its own fresh empty dir; re-run.
    The ping-pong wall clock was re-run too, with a binary built without features in its own
    target dir: the earlier runs used `target/release/rexx-run` of a target dir where
    `--features sharing` test builds also ran, and whether that file was the feature-off build was
    not established. Medians moved by at most 0.14 s on the oracle side and 0.03 s on ours.

Not done: a type-level check that `OffBaton` holds no `Rc` or `Cell`; the claim is in the SAFETY
note only.

Checks at `d8f5d541e` (own target dir): fmt 0; clippy `-D warnings` 0 with no feature, `sharing`,
and `sharing,pinning` (and `pinning` alone at `d9e17e5c8`'s code before commit); `cargo test -p
rexx-core --doc` 3 passed and 5 compile_fail passed; `cargo test --workspace --release
--no-fail-fast` (built first, then run under `memcap 8G`) exits 0, 3036 passed and 0 failed. The
`sharing` witnesses are feature-gated and are not in that count; at `e64886a0f`,
`memcap 8G cargo test --release -p rexx-exec --features sharing --test concurrency_tests -- sharing::uninit sharing::one_activity sharing::an_object`:
3 passed.

## Fix round 2

Code commit `2a9bbbe12`; records, gate and this section in the commit after it.

1. C2 again. `Interp::collect_now` now runs with touches paused for its whole body, and its three
   liveness prunes (kept strings, guard pools and watches, semaphores) use a new untagged
   `Heap::peek`. Walks outside a collection found by the enumeration: `record_exposer`'s prune of
   the stem-exposer table and `weak_target`'s read of a weak cell, both `Heap::peek` now. Debug-only
   checks that read objects (found by `debug-checks.py`) all read under a new `unshared!` macro
   (`lib.rs`), which also replaces the two hand-written pauses of fix round 1: the `.NAME` cache's
   search (`environment.rs`), the required-string latch (`reqstr.rs`), an operand's operator gap
   (`eval.rs`), a DO OVER snapshot's slots (`run/loops.rs`) and `to_text` beside `text_len`
   (`value.rs`) are new. Enumeration, commands and classification:
   `docs/superpowers/records/2026-10-01-phase-6-s2-s5/sharing-walks/`. Classifying a row of
   `debug-checks.txt` as reading no heap object rests on the names it calls, not on reading each
   callee's body; the debug-release comparison below is the check on it.
   Witness `sharing::a_collection_in_another_activity_shares_nothing_it_prunes` (the reviewer's
   probe: 100 `.MutexSemaphore`s acquired and released by main, then `call GC 'Force'` in a started
   activity, asserting program shared below 50): with `lib.rs` restored to `4db3412ee`'s
   temporarily it fails, `program: SharingCount { objects: 111, shared: 102 }`; at `2a9bbbe12` it
   passes. Restored by copy, `cmp` clean.
2. Criterion 6 re-derived at `2a9bbbe12` over every population. The shared counts equal fix round
   1's in every population; program objects moved in the every-group (deadline and timing rows)
   and keyword rows. Corpus: release and debug tables byte-identical. Every group: release and debug
   shared columns identical; `TIME` (deadline) and `SysSleep` (failure in one, pass in the other)
   rows differ in objects.
3. F1: the every-group population is stated as each group run until its end or its first refusal
   or deadline, with the outcome counts quoted from the record's header beside the grep command.
4. N1: the every-group test is `#[ignore]` with its reason; the gate quotes its command with
   `RAYON_NUM_THREADS=4` and `--ignored`.
5. Line citations re-derived: the inventory record at `2a9bbbe12` (only `lib.rs` lines moved),
   `lib.rs:1252` and `:3160` in the gate; `time_support.rs:202` (the reviewer's F2 nit).
6. The feature-off `.text` hash at `2a9bbbe12` equals `0ac73b804`'s (`sharing-off-text-hash.txt`).

Checks at `efda725c6` (own target dir): fmt 0; clippy `-D warnings` 0 with no feature, `sharing`,
`pinning`, and `sharing,pinning`; `cargo test -p rexx-core --doc` 3 passed and 5 compile_fail
passed; `cargo test --workspace --release --no-fail-fast` (built first, then run under `memcap 8G`)
exits 0, 3036 passed and 0 failed. The four `sharing` witnesses passed at `2a9bbbe12` in the
measurement run (`--release --features sharing --test concurrency_tests -- sharing:: --skip
every_ootest --skip derived_list`: 4 passed).

## Fix round 3

Code commit `f078489bc`; records, gate and this section in the commit after it.

1. N3: `detach_exposed_tails` reads the exposer stems through `Heap::peek` and rewrites them
   through a new `Heap::peek_mut` (untagged `get_mut`); its read of the stem being cleared stays
   tagged. Witness `sharing::a_stem_cleared_in_another_activity_shares_no_dead_exposer` (the
   reviewer's probe: `h. = 0`, 100 calls of `p: procedure expose h.1`, then `s.~empty` in a
   started activity; asserts stdout `ok` and `0`, the oracle's in 3 runs, and program shared below
   50): with `stem.rs` restored to `cce32934b`'s temporarily it fails,
   `program: SharingCount { objects: 212, shared: 103 }`; at `f078489bc` it passes. Restored by
   copy, `cmp` clean. A live exposer's rewrite is now uncounted too; the walks README says so.
2. Coverage of `get_mut` and `body_text`: `tagged-sites.py` matched all four tagged accessors
   already; its `get_mut`, `body_text` and `is_class` rows were listed and read by function, and a
   new `iterating-functions.py` lists every function among the tagged sites that iterates
   anything. Each listed function iterates the one object its operation targets; no further walk
   of an interpreter table was found.
3. N4: `debug-checks.py` ends a field or statement under `#[cfg(debug_assertions)]` at its `,` or
   `;` outside brackets (`->` excepted). The `ir.rs:646` row and the `ir/compile.rs:1221`
   struct-literal row, both artefacts, are gone; `ir.rs:691` now lists `copied get`.
4. Criterion 6: at `f078489bc` the corpus, derived-list and every-group runs (release) give the
   same bootstrap and program shared counts as the records at `2a9bbbe12`, so the records are not
   re-derived. Program objects differ only in the `TIME` (deadline) and `SysSleep` (pass this
   time) rows of the every-group run. The feature-off `.text` hash at `f078489bc` equals
   `0ac73b804`'s.

Checks at `ea52f92d1` (own target dir): fmt 0; clippy `-D warnings` 0 with no feature, `sharing`,
`pinning`, and `sharing,pinning`; `cargo test -p rexx-core --doc` 3 passed and 5 compile_fail
passed; `cargo test --workspace --release --no-fail-fast` (built first, then run under `memcap 8G`)
exits 0, 3036 passed and 0 failed. At `f078489bc`, the five `sharing` witnesses (`--release
--features sharing --test concurrency_tests -- sharing:: --skip every_ootest --skip derived_list`):
5 passed; `refusal_sites`: 5 passed.
