# Phase 6.1 parse round 2 review

Range `3d43b65cb..c28a63f37`: `23d78ec1b` (`rust/crates/rexx-core/src/heap.rs`) and `c28a63f37` (gate record, roadmap row 6.1).

## Verdicts

* Spec compliance: **PASS**. Critical 0, Important 0, Minor 2.
* Quality: **PASS**. Critical 0, Important 0, Minor 4.

## What was run

All runs used a `git archive 23d78ec1b rust interpreter` copy under `/tmp/claude-1000/p61/pr2rev/src`, with `CARGO_INCREMENTAL=0`, `-j 4`, target `/tmp/claude-1000/p61/pr2rev/target` (both deleted afterwards), and `memcap` with `timeout`. A scratch module `review_probe` was appended to the copy's `heap.rs`. It is kept as `parse-round2-review-probe.rs`, and the mutation driver as `parse-round2-review-mut.sh`. Logs are in `/tmp/claude-1000/p61/pr2rev/`.

The probe has three tests:
* `probe_hand_scenario` charges strings, grows an array with `rehold_body_bytes`, replaces a string body with a larger one, frees it by leaving it unrooted (a weak reference to it clears), resurrects an UNINIT array that reaches a 300-byte string only through the resurrect loop, holds an immortal string, then collects twice, the second time after `clear_uninit_all`. Each figure is checked against hand-computed sums and against `recompute`, which sums `held_bytes()` over every live slot.
* `probe_fuzz` runs 300 seeds of 400 random steps each. The steps allocate heap and inline strings and arrays over random edges, grow and shrink arrays, replace strings, set UNINIT, make weak references, allocate immortals, change roots, clear half the pending finalizers and collect. After every step it asserts that the running `held_bytes` equals `recompute`. After every collection it also asserts that `stats.live_bytes`, `held_bytes` and `live_bytes` equal it and that `bytes_since` is 0. Totals: 20093 collections, 699 resurrections, 49175 sweeps.
* `probe_missed_charge` replaces a rooted string body without calling `rehold_body_bytes`, then collects.

Results:

| run | result |
|---|---|
| debug, `cargo test -p rexx-core --lib review_probe` | exit 0, 3 passed; `probe_missed_charge` panicked at `heap.rs:390` (the collection assertion), as expected |
| release, `cargo test --release -p rexx-core --lib -- review_probe body_bytes_tests` | exit 0, 4 passed; `probe_missed_charge` did not panic and reported `live_bytes` 300 (the figure silently reset) |

Mutations, each applied by `sed` to one line, checked by `diff` and restored by `cp` plus `cmp`:

| mutation | build | committed `a_collection_sums_the_survivors_body_bytes` | probe |
|---|---|---|---|
| resurrect loop's `survivor_bytes +=` dropped (l.323) | release | FAILED, left 100 right 400 | hand and fuzz FAILED |
| string fast path answers 0 (l.240) | release | FAILED, left 300 right 400 | all FAILED |
| non-string mark arm answers 0 (l.245) | release | **ok** | hand FAILED (525 vs 1549), fuzz FAILED |
| non-string mark arm answers 0 (l.245) | release, `cargo test -p rexx-exec --lib -- on_their_bytes slot_cadence uninit` | 21 passed, 0 failed | n/a |
| resurrect loop's sum dropped (l.323) | debug | FAILED at `heap.rs:390`, left 100 right 400 | FAILED at `heap.rs:390` |
| `Heap::alloc` hold dropped (l.411) | debug | FAILED at `heap.rs:168` (release underflow) | FAILED |

The report's two mutation figures (100/400, 300/400) reproduce.

## Focus questions

**Every survivor path is summed exactly once.** A slot is marked only at `heap.rs:228` (the mark loop) and `:316` (the resurrect loop). Both use `mem::replace` on the mark, so no slot is summed twice. Each loop adds the body's bytes at the point where it marks the slot (`:248` and `:323`). Roots and immortals enter through `work`, so the mark loop covers them. Pass 1 rewrites a dead weak reference to `WeakRef(NIL)` after it has been summed, and `held_bytes` is 0 for `WeakRef` either way. No other code sets `marks`. The fuzz probe's per-collection `recompute` equality covers all of these together.

**The string fast path covers every `Body::Text` kind.** `Body::Text` has the fields `bytes: Bytes` and `num: Option<Result<Box<Number>, NotNumeric>>`. `Body::held_bytes` for `Text` is exactly `bytes.heap_len()` (`body.rs:1032`), and `heap_len` covers both `Repr::Inline` (0) and `Repr::Heap` (`bytes.rs:107`). `Body::trace` for `Text` is empty (`body.rs:1050`). The `num` cache holds no `ObjRef` and is not charged on either path. Inline and heap strings both appear in the fuzz probe. A string reached only through resurrection is summed by the resurrect loop's `held_bytes()` and never takes the fast path. The hand scenario covers that case.

**R10 is unchanged.** `peak_bytes = peak_body_bytes()` still runs before `live_bytes` is overwritten, so the peak still reads the previous `live_bytes + bytes_since`. `Interp`'s `bytes_due = COLLECT_BYTES_FLOOR.max(stats.live_bytes)` (`rexx-exec/src/lib.rs:3080`) reads `stats.live_bytes`. That equals the old figure whenever the debug assertion holds, and the fuzz probe shows it equal to the recomputation in release. `bytes_since` handling is untouched. The committed test's `peak_body_bytes` assertions (600, 600, 950) pass in release.

**The debug assertion still compares two independent computations.** One side is `survivor_bytes`, summed by the two marking loops. The other is the running `held_bytes`, maintained by `hold`, `release` and `rehold` at the charge sites, less `freed_bytes`, which the sweep sums over unmarked slots in debug. Breaking either side alone fires the assertion: dropping the resurrect sum fires it at `:390`, and so does a missed `rehold` (`probe_missed_charge`). Dropping `alloc`'s hold fires the underflow check in `release_body_bytes`, also debug-only. The new `debug_assert_eq!(held, object.body.held_bytes())` at `:247` is a separate, local check on the fast path.

**The gate record's lines point at outputs that exist.** `bg/23d78ec1b/status.txt` reads `23d78ec1b started 2026-10-10T20:18:02`, G1 through G9 exit 0, P48 reruns 0, `finished 2026-10-10T21:03:14+02:00`. Summed `test result` lines give 3149 passed, 0 failed and 4 ignored in `logs/g4-test-release.txt`, and 3153 passed, 0 failed and 4 ignored in `logs/g6-test-debug.txt`, matching the record. `cg-fin.log` ended at 20:17:27, before the gate started. `c28a63f37` was committed at 21:03:56, after it finished, so the gate ran alone on `23d78ec1b`. The record's 27-row table matches `/tmp/claude-1000/p61/pr2/cg-fin.log` exactly (base Ir, cur d%, fin d%). `fin` minus `cur` is positive only for emptyloop (+195), extcall (+59), pingguard (+701) and pingsem (+24), so the claim of no rise above 1,000 Ir holds. `bin/fin/rexx-run` hashes to the recorded `2d56a4e2...` and `t1/bin/base/rexx-run` to `2ea19b3e...`.

## Spec compliance findings

### Minor

1. **The perf evidence for the record lives only in `/tmp`.** `## Parse round 2` cites `callgrind.sh` output without a path, and the report points at `/tmp/claude-1000/p61/pr2/cg-fin/`. The earlier `### Criterion 7` section cites committed evidence (`task-12-evidence/cg-close2/`). The criterion 7 closure table therefore has no surviving witness once `/tmp` is cleared. Copy `cg-fin.log` (and `cgdiff-a4.txt`) into the workspace and cite them.
2. **`bg/23d78ec1b/` and the report are untracked**, while earlier `bg/<sha>/` directories are committed (ruling: SDD records are committed with `add -f`). The record cites `status.txt` at a path that is not in the tree yet. This is the controller's step, noted so it is not lost.

## Quality findings

### Minor

1. **No release test witnesses the non-string mark-loop arm.** With `heap.rs:245` answering 0, the committed `a_collection_sums_the_survivors_body_bytes` and the rexx-exec byte-trigger and UNINIT tests (21) all stay green in release. The committed test holds only strings. Only the debug `debug_assert_eq!` at `:247` (G6) catches the mutation. Because the arm is not `cfg`-split, the debug run does exercise the same code. Still, the gate record's sentence "now checks that sum in release" holds only for the string and resurrect paths. Add a kept `Body::Array` (grown with `rehold`) to the committed test.
2. **The fast path restates "a string traces nothing" without a check.** If `Body::Text` ever gains a traced field, `Body::trace` and the mark loop would both need changing. Only the `held_bytes` half of the duplication is asserted (`:247`). A debug-only `trace` into an empty vector inside the `Text` arm, with an `is_empty` assertion, would close it. The report rejected A3 for exactly this kind of second definition.
3. **In release the running `held_bytes` is now write-only.** Its only reader is `collect` under `cfg(debug_assertions)`, and release overwrites it at `:396`. Every `hold_body_bytes`, `release_body_bytes` and `rehold_body_bytes` call therefore does dead work in release. The field's doc ("what was charged or held less what was released and what the sweeps freed") no longer describes release, where the sweep frees nothing from it. The report's concern 2 states the consequence: release no longer carries a missed charge site's drift, and `probe_missed_charge` confirms it. Correct the doc. Gating the running figure on `debug_assertions` would be a further perf option, outside this round's scope.
4. **The `debug_assert_eq!` at `:247` compares `held_bytes()` with itself on the non-string arm.** It is harmless, but it is meaningful only inside the `Text` arm, where it would read as the fast path's contract.
