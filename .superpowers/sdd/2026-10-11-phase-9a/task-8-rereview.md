# Phase 9a Task 8 re-review (fix round 1, b0628b9be..0af69ae87)

Read-only; no builds, no suite reruns. One oracle probe (below). Items not run: the mutant of (a) and
the Properties row of `method_bodies` on our side (no built binary in the worktree).

* **I1: addressed.** `object_protocol.rs:661-671`: the cloned body's `NativeState::Text` length is
  charged with `charge_unheld` right after `alloc_charged`, the same call and the same byte count
  `native_string_new` makes (`construct.rs:449,461`). Other kinds: `NativeState::Buffer` is already
  reported by `held_bytes` (`body.rs:1075`) and so by `alloc_charged`; Stem, Method, Routine and
  Message copies carry no native bytes (a Stem copy's tails are `Body::Stem`, `held_bytes` 0 for a base
  stem too). Not covered, pre-existing and not subclass text: `NativeState::Data` (`Buffer`) clones
  `held_bytes` 0. The test (`tests/instance_copy_memory.rs`) runs `rexx-run` under `ulimit -v
  1048576` and asserts `(Some(0), "done 1000000 K\n", "")`; 3000 x 1 MB uncharged is ~3 GB of
  address space, so an uncollected run aborts and the tuple differs (it can fail). The implementer's
  mutant result (exit 101, allocation failed) is consistent with that; I did not rerun it.
* **I2: addressed.** `store_holders` is `Vec<ObjRef>` resolved in `ObjectModel::build`
  (`dispatch.rs:1405-1412`, field `:1238`) from the native registry by name, once; a user
  `::class Directory` cannot rebind it (and the old per-call lookup was the registry too).
  `holds_store_class` (`hash.rs:252`) is a membership test then `is_a`. Properties: oracle
  `.properties~superclass~id` is `Directory` (probed, /tmp/claude-1000/p9/t8-rr/p.rex), so the
  `is_a` arm covers it and dropping it from the list is correct. `initialize_store` now tests
  `pools.get(scope, HASH_INDEXES)` instead of a full `read_store`; equivalent because `install_store`
  writes all five entries together (`hash.rs:319-323`). It returns early for a non-`Instance` body
  where the old code reached `install_store`; no INIT row reaches such a receiver. Performance claim
  (+20.5% -> +0.41% on hashnew) is the implementer's callgrind, not re-measured by me.
* **Minor 1: addressed.** All four queued files carry a closing line naming witness and `1c0e1d769`
  (the two ignored ones are now committed).
* **Minor 2: addressed.** Report audit table internal row names `upper()` and `lower()`.
* **Minor 4: addressed.** `put_native` overwrites (`dispatch.rs:232-237`); `NATIVE_CLASS_METHODS`
  is written first and `package::RESUMABLE_CLASS_METHODS` after it (`dispatch.rs:1330-1343`), same
  `lookup_class_method` slot, so the deleted `("Package","NEW",native_package_new)` row was dead.
  `refusal-sites.tsv` has no `unbuilt_new`/Package row to go stale.
* **Minor 5: addressed.** No added line in `rust/` contains ` -- ` or an em dash.
* **(d) hashnew registration: complete.** `callgrind.sh` PROGRAMS (checked against the directory),
  `rexx-bench/src/lib.rs` PROGRAMS, `rexx-bench-suite.rs` AXES, `bench-programs/README.md`. The only
  other mentions of `dirread` are historical plans.

## New findings

None Critical or Important.

* Minor: `NativeState::Data` (`Buffer`) copy is uncharged, the same hole as I1 for a non-subclass
  body. Pre-existing and outside Task 8; worth a queued line.
