# Which heap reads the sharing instrument counts (Task 23 fix round 2)

At `2a9bbbe12`, from `rust/`:

```
python3 ../docs/superpowers/records/2026-10-01-phase-6-s2-s5/sharing-walks/tagged-sites.py > tagged-sites.txt
python3 ../docs/superpowers/records/2026-10-01-phase-6-s2-s5/sharing-walks/debug-checks.py > debug-checks.txt
```

`tagged-sites.txt` lists every call of a tagged accessor (`Heap::get`, `get_mut`, `body_text`,
`is_class`) with its function. `debug-checks.txt` lists every debug-only check that calls
anything, marked `unshared` where it reads under `unshared!` (touches paused).

## Walks: not counted

| Where | What it reads | How it is untagged |
|---|---|---|
| `rexx-core/src/heap.rs` `Heap::collect` | every reachable object | private `resolve` only |
| `rexx-core/src/heap.rs` `Heap::clear_uninit_all` | the UNINIT registry | `resolve` and a slot match |
| `rexx-exec/src/lib.rs` `Interp::collect_now` | the liveness prunes of kept strings, guard pools and watches, and semaphores, and whatever else a collection calls | the whole collection runs with touches paused, and the three liveness prunes use `Heap::peek` |
| `rexx-exec/src/stem.rs` `record_exposer` | the stem-exposer table's weak cells, pruned as it doubles | `Heap::peek` |
| `rexx-exec/src/stem.rs` `weak_target` | an exposer's weak cell | `Heap::peek` |

Every collection goes through `collect_now` (`grep -rn 'heap\.collect(' crates/rexx-exec/src`:
one site, in it).

## Debug-only checks: not counted

The `unshared` rows of `debug-checks.txt` read objects: the SAY route and a `StoreView`
re-derived, the `.NAME` cache's search, the required-string latch, an operand's operator gap, a DO
OVER snapshot's slots, and `to_text` beside `text_len`. Every other row calls nothing that
resolves a heap object: register, plan, slot, guard-table, trace-mode, number, library-table,
frame and root-counter reads, and `CALLING` in `ffi.rs`.

## Resolutions: counted

Every other row of `tagged-sites.txt` is an object read by the operation the running activity is
performing on it: a message send, a variable or stem access, a conversion, a collection method, an
UNINIT sent to the object (`run_one_uninit`), or `detach_exposed_tails`' reads and writes of the
stems that expose a cleared stem's tails.

## Check

The corpus table and the every-group table are measured in a release build and in a debug build at
`2a9bbbe12`: the corpus tables are byte-identical, and the every-group tables have the same shared
columns (`../sharing-corpus.md`, `../sharing-groups.md`).
