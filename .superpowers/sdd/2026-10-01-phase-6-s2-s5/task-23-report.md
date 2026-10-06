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

- `pingmsg.rex`, `pingsem.rex`, `pingguard.rex`, not added to `callgrind.sh`'s `PROGRAMS`, which
  Task 26's gate runs against its base; `wallclock.sh` gained a `PROGRAMS` override instead.
- 9 interleaved rounds, load in `pingpong/binaries.txt`; 30 runs per side for output stability.

## Checks

See the final section.
