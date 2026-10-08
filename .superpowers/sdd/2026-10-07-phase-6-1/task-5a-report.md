# Task 5a report: a collection trigger that counts bytes

Implementer t5a, 2026-10-08/09. BASE `37d874a36`. Commits: `8e52e14ca` (behaviour and tests),
`5f5b80b3d` (perf round 1), `0d18b0045` (`refusal-sites.tsv` re-derived), and the record commit
that adds this file. Scratch: `P` = `/tmp/claude-1000/p61/t5a`.

## Design

R10's rule, as built: a collection is also due when the body bytes charged since the last
collection reach `bytes_due`, the larger of `COLLECT_BYTES_FLOOR` (32 MiB) and the body bytes the
last collection found live. The slot test is unchanged.

- **What is counted.** A `Body::Text` whose bytes do not fit `Bytes`' inline buffer
  (`len > INLINE_BYTES`, 54); handle-inline strings and `Bytes`-inline strings hold no body bytes
  and are not charged. The size is the source's length at the point the body is built, never the
  allocator's. Other bodies (arrays, stems, directories, `Num` text) are not counted: their sizes
  are not fixed at creation.
- **Where it is charged.** After the allocation, from the source's length: `Interp::text`,
  `Interp::text_owned`, `take_piece` (PARSE pieces) and `new_raw_string` (native API), through
  `Interp::charge_text` (`value.rs:103`). The charge only adds to `Heap::bytes_since` and sets
  `Interp::collect_due`; the next allocation collects.
- **The allocation fast path is unchanged.** `collect_if_due` tests `collect_due` where it tested
  `stress_collect` (one bool load either way); `enable_stress_collect` sets both, and `collect_now`
  resets `collect_due` to `stress_collect`. No discriminant test was added to `alloc_with`.
- **Live bytes.** `Heap::collect` sums `Body::held_bytes` over every object it marks, in the main
  mark loop and the UNINIT resurrection loop (a resurrected object survives), and returns it as
  `CollectStats::live_bytes`.
- **Peak counter.** `Heap::peak_body_bytes` is the largest `live_bytes + bytes_since` seen at a
  collection or now: an upper bound on body bytes held at once (a body charged and dropped before
  a collection still counts). Reported as `Outcome::peak_body_bytes`, read by the crate test.

## Changes

- `rexx-core/src/body.rs:1021` `Body::held_bytes`.
- `rexx-core/src/heap.rs:51` `CollectStats::live_bytes`; `:73`-`:77` `bytes_since`, `live_bytes`,
  `peak_bytes`; `:148` `Heap::charge_body_bytes`; `:157` `Heap::peak_body_bytes`; `:195`, `:272`
  the two mark-loop sums; `:320`-`:322` the reset; `:738` `body_bytes_tests`.
- `rexx-exec/src/lib.rs:303` `COLLECT_BYTES_FLOOR`; `:322` `Outcome::peak_body_bytes`; `:1475`
  `collect_due`; `:1509` `bytes_due`; `:2654` `collect_if_due`; `:2665` `charge_body_bytes`;
  `:2941`-`:2942` reset in `collect_now`; `:3444` the outcome field.
- `rexx-exec/src/value.rs:97`, `:103`, `:214`; `parse_template.rs:430`;
  `dispatch/library/surface.rs:196`: the charge sites.
- `tests/watchdog/mod.rs`, `tests/method_bodies.rs`, `tests/support/oracle.rs`: the new
  `Outcome` field in their literal constructors.
- `rust/corpus/refusal-sites.tsv`: re-derived; every changed row (38) differs only in its
  `lib.rs` line number (checked by pairing removed and added rows with the line stripped: no
  unpaired row).

## The failing-then-passing test

`rexx-exec/src/tests.rs:1045` `large_dead_strings_are_collected_on_their_bytes` runs
`do i = 1 to 1000; x = copies('abc', 100000); end` and asserts `peak_body_bytes <=
COLLECT_BYTES_FLOOR + 2 * 300_000` and `collections >= 5`. `:1065`
`short_strings_keep_the_slot_cadence` runs `do i = 1 to 200000; x = copies('abcd', 5) || i; end`
(at most 26 bytes, inline) and pins `collections == 6`.

- At BASE (the test with the collections assertion only, since `peak_body_bytes` did not exist):
  `large: collections 0`, panicked at `assertion failed: outcome.collections > 0`; `short:
  collections 6`, which is the pinned value.
- At head: both pass; a probe print gave 8 collections and `peak_body_bytes` 33 900 000 for the
  large case.
- Inversions (each run, then reverted): with the charge in `text_bytes` disabled, the large case
  fails `0 collections for 1000 x 300000 bytes`; with the charge kept but never making a
  collection due, it fails `300000000 body bytes held at once`. These two were run on `8e52e14ca`'s
  shape (charge before the allocation, collect at once) and again on the landed shape after the
  `whole_groups` runs: `charge_text`'s length test made false fails `0 collections for 1000 x
  300000 bytes`; `charge_body_bytes` charging without setting `collect_due` fails `300000000 body
  bytes held at once`. Both reverted by scripted replacement; `git status` clean in `rust/`, both
  tests pass again.
- `rexx-core` `heap::body_bytes_tests::a_collection_sums_the_survivors_body_bytes`: a kept
  100-byte text, a dead 200-byte one, an unreachable 300-byte one flagged for UNINIT, an inline
  one; `live_bytes` 400, peak 600, then 600 and 950 after further charges. With the resurrection
  loop's sum removed it fails `left: 100, right: 400`.

## Peak memory

Programs in `$P/rss` (a fresh directory): `loop4000.rex` and `loop20000.rex` are
`do i = 1 to N; x = copies('abc', 100000); end` then `say 'done' i`; `loop999.rex` is
`do i = 1 to 1000000; x = copies('abc', 333) || i; end` (1002 to 1005 bytes each).

Ours, 5 runs per cell, `memcap 2G /usr/bin/time -f "%M %e %x" timeout -k 5 120 $BIN FILE`, with
`REXX_SWITCH_MODE=every` for the every cells; binaries `base` (`37d874a36`) and `t5a`
(`0d18b0045`) from `git archive`, built in their own target directories (sha256 below). Oracle,
5 runs, `( ulimit -v 1048576; LD_LIBRARY_PATH=$ORACLE/lib /usr/bin/time ... timeout -k 5 20 rexx
FILE )` from the empty directory `$P/oracle-run`. Every run printed `done N+1` and exited 0 except
where stated. Median max RSS in KB (min to max), median wall seconds:

| program | base normal | base every | t5a normal | t5a every | oracle |
|---|---|---|---|---|---|
| loop4000 | 1 191 136 (1 190 884-1 191 264), 1.17 s | 1 191 392 (1 190 732-1 191 656), 1.12 s | 52 404 (51 832-52 488), 0.97 s | 52 308 (52 072-52 484), 0.89 s | 13 380 (13 116-15 184), 0.70 s |
| loop20000 | memcap OOM at 2G, rc 137 (1 run) | not run | 51 932 (51 884-52 564), 4.75 s | 52 184 (51 960-52 536), 4.30 s | 13 016 (12 704-15 244), 3.46 s |
| loop999 | 90 064 (89 628-90 208), 1.05 s | 89 684 (89 280-90 248), 1.07 s | 55 364 (54 784-56 700), 1.23 s | 55 452 (55 108-55 668), 1.24 s | 20 580 (20 316-20 884), 0.77 s |

The tree-walker no longer exists, so "both engines" is read as the two switch modes. RSS is now
flat in N at 300 KB strings (about 52 MB at 4000 and at 20000 passes; it was linear, 1.19 GB at
4000).

`loop999`'s wall clock is the one cost seen: 10 interleaved runs, base 1.04 to 1.07 s, t5a
bimodal, 1.02 to 1.03 s (3 runs) or 1.13 to 1.24 s (7 runs). The instruction count is not the
cause: callgrind on the same loop at N = 100 000 gives 1 460 090 777 against 1 467 873 451
(+0.53%), and minor faults fall (21 260 to 13 123). Not diagnosed further.

## Performance

Binaries (`git archive <sha> rust interpreter`, every file touched, `CARGO_TARGET_DIR=$P/target-NAME
memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`, one `Compiling rexx-exec` each):

| binary | source | sha256 |
|---|---|---|
| base | `37d874a36` | `66a67962b9e8e4053f88d73167a8762d0b5df585081d210c5dd5227028d5a305` |
| pad48 | base + `layout-pad.py 48` | `e5737b82031b9de2c323c2c3767bb3745c15d27f228fc1a48f676ff2eea4bdf3` |
| t5a | `0d18b0045` | `0736d76c9d03230ca55835d52a52686cfa8b5a90e8022cbb4f600165ca91053a` |

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $P/cg-final -p "alloc alloc4c strings rexxcps emptyloop" base=$P/target-base/release/rexx-run pad48=$P/target-pad48/release/rexx-run t5a=$P/target-final/release/rexx-run
```

Exit 0, every spread 0.0000%. pad48 is +0.0000% on every program.

| program | t5a % | verdict |
|---|---:|---|
| alloc | +0.0434 | inside |
| alloc4c | -0.4070 | inside |
| strings | +0.1016 | inside |
| rexxcps | +0.2035 | inside |
| emptyloop | +0.0000 | inside |

The first commit (`8e52e14ca`, charge read from the built `Bytes` inside `text_bytes`, collect at
once) measured +0.80% on strings and +0.89% on rexxcps (`$P/cg`, `-r 3`). `cgdiff.py` put it in
`exec_parse`, `concat_values` and `text_built`, with no call count changed; line attribution showed
the `Body::Text` construction and `Bytes::from_slice` lines growing, the body copied into its slot
instead of built there. Variants at `-r 1` (`$P/cg-v2` to `$P/cg-v8`), rexxcps / strings:
`collect_due` flag, charge still in `text_bytes` +0.92 / +0.73; charge before `Bytes::from_slice`
in `text` +0.74 / +0.10; also after the allocation in `take_piece` +0.38 / +0.10;
`charge_body_bytes` `#[inline(never)]` +0.38 (no change, reverted); every site after the allocation
+0.38; `take_piece` uncharged (control) +0.10; `take_piece` charging `piece.len()` so no length is
live across the allocation +0.20 / +0.10, which is what landed. The remaining +0.10% on strings
and rexxcps is the mark-loop sum (`collect_now` +17.9M and +18.5M instructions, same number of
collections).

Wall clock, `PROGRAMS="alloc alloc4c strings rexxcps emptyloop" memcap 8G bash
rust/bench-programs/wallclock.sh -r 5 -o $P/wall base=... pad48=... t5a=...`, exit 0, load average
0.74 at start and 1.02 at end:

| program | pad48 % | t5a % |
|---|---:|---:|
| alloc | +0.37 | -0.75 |
| alloc4c | +2.15 | +3.76 |
| strings | -1.56 | +0.35 |
| rexxcps | -0.41 | +1.17 |
| emptyloop | +0.67 | -2.23 |

All inside the ±4% band.

## Commands and results

- `cargo fmt`; `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings`: Finished,
  no warnings.
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast` at `0d18b0045`: exit 0, 3066 passed,
  0 failed, 4 ignored (summed over the `test result` lines). It includes `collect_stress` (37
  passed, the L0 subset under collect-on-every-allocation among them) and `collect_policy` (2
  passed). The same run at `8e52e14ca` failed only `refusal_sites` (lines moved), fixed by
  `0d18b0045`.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
  ir_recorded_oracle` at `0d18b0045`: exit 0, 50 passed, 1 ignored.
- `REXX_REFUSAL_SITES_REFRESH=1 memcap 8G cargo test -j 4 -p rexx-exec --test refusal_sites`, then
  without the variable: 5 passed.
- No collection-count pin moved: the only count pinned on purpose is the new short-string test;
  no existing test failed.

## `whole_groups`

```
REXX_CORPUS_GATE=1 REXX_WHOLE_GROUPS_TABLE=$P/wgN/table.tsv memcap 8G /usr/bin/time -v cargo test -j 1 --release -p rexx-exec --test concurrency_tests whole_groups
```

| run | tree | result | wall | max RSS |
|---|---|---|---|---|
| before (Task 5 I2 run 3) | `a945a5ae8` (BASE's code but for a doc comment) | exit 0, 6 passed | 25:58 | 819 136 KB |
| 1 | `0d18b0045`, rebuilt (`Compiling rexx-exec`) | exit 0, 6 passed | 6:42 | 2 381 424 KB |
| 2 | `0d18b0045`, no rebuild | exit 0, 6 passed | 4:10 | 915 712 KB |

Run 1's figure includes rustc, as I2's run 2 did (2 385 136 KB). Run 2 against I2's run 3 is the
like-for-like pair: +96 MB for the whole cargo process tree, one run each, not attributed. The
harness leaves TEST_SUBCLASSES_GC out of every part since Task 5's I2, so these runs do not reach
the loop this task bounds. Between runs 1 and 2 our cells differ only on REPLY (which of the
oracle's racing outcomes ours matched); the oracle's outcome distributions differ on STREAM,
RexxContext and REPLY.

## Concerns

1. **`loop999` wall clock is bimodal**: 7 of 10 runs 1.13 to 1.24 s against base's 1.04 to 1.07 s,
   3 runs 1.02 to 1.03 s; instructions +0.53% (N = 100 000), minor faults fewer. The byte floor
   makes 1000-byte strings collect about every 33 000 allocations where the slot test waited for
   65 536 slots. Not diagnosed. A larger floor trades it back for memory.
2. **Only text bodies are counted.** Arrays, stems, directories, buffers and `Num` text grow after
   creation and are not charged; a loop of large dead arrays still collects on slots alone.
   `finish_string` (native API) rewrites a charged body in place at the charged length.
3. **`Outcome::peak_body_bytes` is a new public field**, where the brief says no outward
   interface; it is how the crate test reads the heap's counter, beside `Outcome::collections`.
   Four literal `Outcome` constructors in `tests/` gained it.
4. **The mark loop pays about 0.1%** on strings and rexxcps (`Body::held_bytes` per marked
   object). Summing at the sweep would cost per dead object instead, which at a slot-triggered
   collection is at least as many.
5. TEST_SUBCLASSES_GC still never ends (D59 keeps the class), so the harness's exclusion stays;
   this task bounds its memory, not its run time. Not run through the group here.

## Fix round 1

Review: `task-5a-review.md`. Code commit `d0a3d5db3`. `P` as above.

### Changes

- **Important 1, `~copy`.** `copy_object` allocates through the new `Interp::alloc_charged`, which
  takes `Body::held_bytes` before the move and charges it after the allocation. It covers every
  body kind the copy can clone.
- **Ruling: arrays and buffers count.** `Body::held_bytes` adds an `Array`'s slot capacity
  (`SLOT_BYTES` = `size_of::<Option<ObjRef>>()` each) and a `MutableBuffer`'s `capacity`; the mark
  loops sum them as before. Every `BehaviourId::ARRAY` allocation goes through `alloc_charged`
  (a regex replace of `alloc_with(BehaviourId::ARRAY,`; 31 sites, the same list
  `grep -rn "BehaviourId::ARRAY" rexx-exec/src` gives minus `object_protocol.rs:610`, which is
  the copy's behaviour choice). Growth charges the capacity delta where the capacity changes:
  `array_grow`, `array_splice_slot`, `array_resize` (slot capacity before and after);
  `BufferState::ensure_capacity` and `set_buffer_size` now answer the growth, charged by
  `append`, `setBufferSize`, `setText` and the six mutators that call `buffer_capacity`, and by
  the native API's `set_mutable_buffer_capacity`. Buffer creation (`MutableBuffer~new`, the native
  API's `new_mutable_buffer`) charges its capacity. Stems are not charged (the review found
  stem+DROP bounded).
- **Minors.** `stress_collect`, `collect_at` and `bytes_due` docs restated. `Outcome::peak_body_bytes`
  is `#[cfg(test)]`, and the four integration-test `Outcome` literals no longer name it. The
  short-string test also asserts `peak_body_bytes < 256 KiB`; with `charge_text`'s inline
  exemption removed it fails at 1 513 074 (normal: about 15 KB). `refusal-sites.tsv` re-derived
  (line numbers only; every removed row pairs with an added one once the line is stripped).

### Tests

`assert_collected_on_bytes` (peak at most the floor plus four values, at least 5 collections)
runs five loops: 300 KB strings, `~copy` of a 300 KB string, `.array~new(100000)` with
`a[100000] = i`, `.array~new` grown by `a[100000] = i`, and a MutableBuffer grown by three 100 KB
appends. Against `19e3b9d34` (an archive with this `tests.rs`, `SLOT_BYTES` written as 16, own
target directory, `Compiling rexx-exec` present) the four new loops fail with `0 collections for
...` and the string loop and short-string test pass. At `d0a3d5db3`: peaks 33.9 to 35.2 MB, 8 to
11 collections.

Peak RSS, one run each, the same commands as the first table:

| program | base | fr1 (`d0a3d5db3`) | oracle |
|---|---:|---:|---:|
| `y~copy` of 300 KB, 3000 passes | 898 520 KB | 52 352 KB | 13 848 KB |
| `.array~new(100000)`, 1000 passes | 1 582 256 KB | 54 884 KB | 22 156 KB |
| MutableBuffer, three 100 KB appends, 3000 passes | 911 376 KB | 44 388 KB | 15 448 KB |

### Per-task check

At `d0a3d5db3`: `cargo fmt`, `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings`
clean; `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 0, 3070 passed, 0 failed,
4 ignored (`collect_stress` 37 passed); `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec
--test corpus --test ir_recorded_oracle` exit 0, 50 passed, 1 ignored.

### Performance

Binaries rebuilt (`git archive`, own target directory, one `Compiling rexx-exec` each): `b2`
`37d874a36` sha256 `c9a451e632c9e6d64c924da7760287eefaf8798d6effb75816227228da8ae733`; `fr1`
`d0a3d5db3` sha256 `d2ac10ff755a3a5ece5e6b45a9c6ab3627eea2ec6caecd851e66ff45b5e05496`; `p2` = b2 +
`layout-pad.py 48`.

```
memcap 8G bash rust/bench-programs/callgrind.sh -r 3 -j 8 -o $P/cg-fr1 -p "alloc alloc4c strings rexxcps emptyloop" base=$P/target-b2/release/rexx-run fr1=$P/target-fr1/release/rexx-run
```

Exit 0, spreads 0.0000%: alloc +0.1905, alloc4c -0.3756, strings +0.1356, rexxcps +0.2383,
emptyloop +0.0000. All inside +0.5%. `array_fill20` is not a bench program, so it is in the
diagnosis below rather than in callgrind.

### Diagnosis of the wall-clock cost (one round)

This machine (AMD Ryzen AI MAX+ 395) counts two events per run, so each `perf stat` run is
`cycles` plus one more. Every run was pinned (`taskset -c 4`) under `memcap 2G`, from a fresh
`mktemp -d` directory. Scripts: `$P/tools/pstat.sh`, `$P/tools/pcyc.sh`.

**`loop999`.** Cycles in G, 8 interleaved runs each, sorted (`$P/pcyc-loop999.txt`):

| binary | cycles | instructions |
|---|---|---|
| b2 | 3.08 3.08 3.10 3.10 3.10 3.11 3.12 3.13 | 14.449 G |
| fr1 | 2.69 2.69 3.21 3.65 3.77 3.83 3.85 4.36 | 14.401 G |
| fr1, `COLLECT_BYTES_FLOOR` = 2^60 (byte trigger off, base's collection cadence) | 2.98 3.15 3.42 3.75 3.76 3.81 3.88 3.96 | 14.464 G |
| fr1, floor 64 MiB | 2.86 2.87 2.96 3.10 3.75 3.80 4.04 4.04 | 14.464 G |

- **It is not the extra collections.** With the byte trigger switched off, the binary collects on
  base's cadence and still has the same slow mode. `collect_now` takes 2.2 M cycles in both a fast
  and a slow run.
- **Where the cycles go.** `perf record -e cycles` on six fr1 runs: a 3.03 G run against a 4.07 G
  run differ by +806 M in `builtin::string::copies_bytes` (442 M to 1 248 M) and +240 M in libc
  `memmove`. Allocation, GC and the driver each differ by under 60 M.
- **Counter shape.** Frontend-idle cycles (`stalled-cycles-frontend`) are 0.50 G on base and 1.13
  to 1.22 G in fr1's slow runs (0.55 G in its fast run). L1 icache misses are lower on fr1 (2.6 M
  against 3.8 M), and branch misses are not higher.
- **It moves with data addresses.** Padding the environment by 0 to 3584 bytes moves the
  interpreter's heap layout, since the environment is copied at startup. That moves fr1 between
  2.70 and 3.66 G while base stays at 3.04 to 3.14 G. A same-binary bimodality that moves with the
  environment points at data placement, not code alignment. The likely mechanism is aliasing
  between `copies_bytes`' 3-byte source and its destination as the heap layout shifts with the
  grown `Interp` and `Heap` structs. Not measured further; that is the guess.
- **The loop's own shape is the lever.** `copies_bytes` extends the output one 3-byte piece at a
  time (333 times per value here). In a scratch-only build of fr1, filling by doubling
  (`extend_from_slice` once, then `extend_from_within`) takes `loop999` to 1.26 to 1.32 G cycles
  and 4.07 G instructions, unimodal, against base's 3.1 G and 14.4 G. Not landed: it is outside
  this task. It would remove this program's sensitivity, and is worth a task of its own.

**`array_fill20`** (`do i = 1 to 20; a = .array~new; do j = 1 to 10000; a~append(j); end; end`),
5 interleaved runs (`$P/pcyc-af20.txt`):

| binary | cycles G | instructions |
|---|---|---|
| b2 | 3.01 3.02 3.04 3.05 3.11 | 18.010 G |
| p2 | 3.00 3.01 3.02 3.03 3.06 | 18.010 G |
| rev, my build of `d9794b41f` (the reviewer's HEAD) | 3.01 3.01 3.03 3.03 3.05 | 18.010 G |
| fr1 | 2.98 2.98 2.99 3.00 3.01 | 18.011 G |

The reviewer's +35% on `d9794b41f` does not reproduce on my build of the same commit. Both trees
came from `git archive` but were built under different directories, and source paths are embedded
in the binary. So the regression belongs to one build's code placement, not to the source. That
fits the review's own finding that `occupied` moved from 0 to 32 mod 64. I did not try aligning
`occupied`: no build I have shows the slow mode to test against.

### Concerns after this round

1. The `loop999` slow mode remains in fr1 (median about 3.7 G cycles against 3.1 G). Its cost is in
   `copies_bytes`' per-piece loop and moves with heap placement, not with collections. The fix that
   removes it (doubling copy) is out of this task's scope.
2. `array_splice_slot` and `array_grow` compare the slot capacity before and after on every call.
   That is one subtraction per insert or append, not a branch into the collector.
3. Arrays charge on every allocation, argument arrays included (16 bytes per slot). For an
   argument-heavy program this moves the byte trigger only after 32 MiB of slots.
