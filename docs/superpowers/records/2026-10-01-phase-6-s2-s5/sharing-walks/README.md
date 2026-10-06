# Which heap reads the sharing instrument counts (Task 23 fix rounds 2 and 3)

At `f078489bc`, from `rust/`:

```
python3 ../docs/superpowers/records/2026-10-01-phase-6-s2-s5/sharing-walks/tagged-sites.py > tagged-sites.txt
python3 ../docs/superpowers/records/2026-10-01-phase-6-s2-s5/sharing-walks/debug-checks.py > debug-checks.txt
python3 ../docs/superpowers/records/2026-10-01-phase-6-s2-s5/sharing-walks/iterating-functions.py tagged-sites.txt > iterating-functions.txt
```

`tagged-sites.txt` lists every call of a tagged accessor (`Heap::get`, `get_mut`, `body_text`,
`is_class`) with its function. `debug-checks.txt` lists every debug-only check that calls
anything, marked `unshared` where it reads under `unshared!` (touches paused); a field or
statement under `#[cfg(debug_assertions)]` ends at its `,` or `;`. `iterating-functions.txt` lists
the functions of `tagged-sites.txt` with a loop in their own body; a loop that calls a function
holding a tagged read is not listed.

## Walks: not counted

| Where | What it reads | How it is untagged |
|---|---|---|
| `rexx-core/src/heap.rs` `Heap::collect` | every reachable object | private `resolve` only |
| `rexx-core/src/heap.rs` `Heap::clear_uninit_all` | the UNINIT registry | `resolve` and a slot match |
| `rexx-exec/src/lib.rs` `Interp::collect_now` | the liveness prunes of kept strings, guard pools and watches, and semaphores, and whatever else a collection calls | the whole collection runs with touches paused, and the three liveness prunes use `Heap::peek` |
| `rexx-exec/src/stem.rs` `record_exposer` | the stem-exposer table's weak cells, pruned as it doubles | `Heap::peek` |
| `rexx-exec/src/stem.rs` `weak_target` | an exposer's weak cell | `Heap::peek` |
| `rexx-exec/src/stem.rs` `detach_exposed_tails` | the stems the exposer table names for a cleared stem, read and rewritten; until a collection the table still reaches the local stem of a returned `PROCEDURE EXPOSE` | `Heap::peek` and `Heap::peek_mut`; so the rewrite of a live exposer is not counted either |

Every collection goes through `collect_now` (`grep -rn 'heap\.collect(' crates/rexx-exec/src`:
one site, in it).

## Debug-only checks: not counted

The `unshared` rows of `debug-checks.txt` read objects: the SAY route and a `StoreView`
re-derived, the `.NAME` cache's search, the required-string latch, an operand's operator gap, a DO
OVER snapshot's slots, and `to_text` beside `text_len`. Every other row calls nothing that
resolves a heap object: register, plan, slot, guard-table, trace-mode, number, library-table,
frame and root-counter reads, and `CALLING` in `ffi.rs`.

## Resolutions: counted

Every other row of `tagged-sites.txt`, `get_mut` and `body_text` rows included, is an object read
or written by the operation the running activity is performing on it: a message send, a variable
or stem access, a conversion, a collection method, an UNINIT sent to the object
(`run_one_uninit`, counted for the activity that runs it, so an UNINIT a collection runs in
another activity counts its object shared), or the read of the stem `detach_exposed_tails` clears. Each function of
`iterating-functions.txt` iterates the contents of the one object its operation targets (an array,
a stem, a condition, a table or collection being read, a DO OVER snapshot), not a table of the
interpreter's.

## Check

The corpus table and the every-group table are measured in a release build and in a debug build at
`2a9bbbe12`: the corpus tables are byte-identical, and the every-group tables have the same shared
columns (`../sharing-corpus.md`, `../sharing-groups.md`). At `f078489bc` the corpus, derived-list
and every-group runs (release) give the same shared counts as those records.
