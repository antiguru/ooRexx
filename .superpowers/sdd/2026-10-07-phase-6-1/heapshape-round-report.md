# Heapshape round 1: report

Implementer hs-round, 2026-10-09. BASE `7aedf9501`. Code commit `5214a2089`. Scratch:
`/tmp/claude-1000/p61/hsr` (`H` below).

## Change

The heap keeps `held_bytes`, the body bytes every live object holds now, as a running figure.
`Heap::collect` no longer sums `Body::held_bytes` over the survivors. It subtracts the
`held_bytes` of every body the sweep frees, and the figure it leaves becomes `live_bytes`, which
the byte trigger reads as before (`bytes_due = max(COLLECT_BYTES_FLOOR, live_bytes)`).
`bytes_since` is not touched, so no new charge is made anywhere and the trigger condition is R10's.

* `Heap::charge_body_bytes` is unchanged (it adds to `bytes_since` only). `Interp::charge_body_bytes`
  (`lib.rs`) also calls the new `Heap::hold_body_bytes`, so every R10 charge site (`charge_text`,
  `charge_growth`, `alloc_charged`) adds to the running figure where it charges today.
* `Heap::alloc` and `Heap::alloc_immortal` hold `body.held_bytes()` themselves. Neither is on the
  interpreter's allocation path (`alloc_with` calls `alloc_with_uncollected`, which holds nothing,
  as its doc now says). `alloc_immortal` covers `Interp::interned_literal`, whose non-inline literal
  texts were never charged and are live for the run.
* `Heap::release_body_bytes` and `Heap::rehold_body_bytes` record a shrink or a replacement.
  `release_body_bytes` has a `debug_assert!` against releasing more than is held.

### Every place that changes a body's held bytes

`Body::held_bytes` is nonzero for three things: a `Text` whose `Bytes` are not inline, an `Array`'s
slot capacity, and an `Instance` whose native state is a `Buffer` (its `capacity` field). The
enumeration commands, run in `rust/`:

1. Constructions:
   `grep -rn "Body::Text {" crates/*/src | grep -v "Body::Text { bytes, .. }\|Body::Text { .. }\|=> \|matches!\|let Body::Text\|if let\|Some(Body::Text"`;
   `grep -rn "Body::Array {" crates/*/src | grep -v "=>\|matches!\|let Body::Array\|if let\|Some(Body::Array\|Some(&Body::Array\|Some(&mut Body::Array"`;
   `grep -rn "Body::array(" crates/rexx-exec/src`;
   `grep -rn "NativeState::Buffer(BufferState\|BufferState {" crates/*/src`;
   and the clone, `grep -rnE "body\.clone\(\)" crates/rexx-exec/src`.
2. Heap entry points: `grep -rn "heap\.alloc(\|heap\.alloc_immortal(\|\.alloc_with_uncollected(\|heap\.mint_class" crates/*/src`.
3. Whole-body, text and native replacement:
   `grep -rnE "\.body = |\*body = |mem::(replace|take|swap)\(&mut [a-z_.]*body" crates/rexx-exec/src crates/rexx-core/src crates/rexx-api/src`;
   `grep -rnE "\*bytes = " crates/rexx-exec/src crates/rexx-core/src`;
   `grep -rnE "\*native = |\.native = |native\.take\(\)|native\.replace\(|mem::(take|replace)\(&mut \*?[a-z_.]*native" crates/rexx-exec/src crates/rexx-core/src`.
4. Slot capacity:
   `grep -rnE "slots\.(push|insert|resize|resize_with|extend|extend_from_slice|reserve|reserve_exact|try_reserve|try_reserve_exact|append|splice|shrink_to|shrink_to_fit|clone_from)\b|\*slots = |slots = |mem::(take|replace|swap)\(\s*slots|mem::(take|replace|swap)\(&mut \*?slots" crates/rexx-exec/src`,
   and `grep -rnE "Body::Array \{ *slots: [a-z_]+|slots: ref" crates/rexx-exec/src` for a renamed binding (none).
5. Buffer capacity: `grep -rn "\.capacity = " crates/rexx-core/src/body.rs` (only inside
   `BufferState`), then its callers: `grep -rn "set_buffer_size\|\.writable()\|ensure_capacity\|buffer_capacity(" crates/rexx-exec/src crates/rexx-api/src`,
   and the accessors handing out the state: `grep -rn "&mut BufferState\|&mut NativeState\|&mut Vec<Option<ObjRef>>" crates/rexx-exec/src crates/rexx-core/src`.

What they find, by disposition:

* **Charged today, now also held:** `Interp::text`, `text_owned`, `take_piece`, `new_raw_string`
  (`charge_text`); every `BehaviourId::ARRAY` construction and `copy_object` (`alloc_charged`);
  `MutableBuffer~new`, the native API's `new_mutable_buffer`; growth in `array_resize`,
  `array_splice_slot`, `array_grow`, the notify-list push, `ensure_capacity`'s callers and
  `set_mutable_buffer_capacity` (`charge_growth`).
* **Held, not charged:** `Interp::interned_literal` (immortal, through `Heap::alloc_immortal`); the
  test fakes and crate tests that call `Heap::alloc` (`rexx-api` `callbacks/fake.rs`,
  `invoke/tests.rs`, `rexx-exec` `tests.rs`, `rexx-core/tests`).
* **Released or reheld, new in this round:** the sweep (every freed body);
  `MutableBuffer~setBufferSize` (`dispatch/buffer.rs`, a shrink to the default or a smaller
  size); `mutable_buffer` and `set_mutable_buffer_capacity` in `dispatch/library/surface.rs`
  (`BufferState::writable` drops the capacity on a failed reservation); `finish_string` (rewrites a
  charged text at the made length, which may be shorter or inline); `array_reshape`
  (`dispatch/array.rs`, the replacement vector can hold fewer slots than an extended original);
  `stream_init` and `stream_uninit` (`dispatch/stream.rs`, which replace or drop whatever native
  state the receiver had; a buffer only where the method is run against a `MutableBuffer`).
* **No change to held bytes:** the weak-reference clear in `Heap::collect` (a `WeakRef` holds
  nothing); writes, pops, removes, clears and truncations inside an array's capacity (Vec
  capacity does not shrink); `BufferBytes::truncate`, `clear` and `edit` (the held figure is the
  `capacity` field, not the Vec's); stream's standard-stream instance (fresh from `new_instance`,
  no native state); `NativeState::zeroed` (`Data`, not `Buffer`).

The debug check ran over the whole debug workspace suite, the `collect_stress` L0 subset under
collect-on-every-allocation among it. Its first run (`$H/logs/test1.log`) failed on the immortal
literals (running figure 117 bytes short, for example `10325` against `10208`) and on the crate
tests that build arrays through `Heap::alloc`; both are covered above. The second run
(`test2.log`) left only `rexx-core/tests/collect.rs` `reference_cycles_are_collected`, which
replaced a body through `get_mut` (`32 body bytes released of 16 held`). That test now writes the
cycle's slot in place, which tests the same cycle; it is not an R10 test.

## Debug check

In debug builds `Heap::collect` sums `held_bytes` over the survivors in both mark loops and
asserts the sum equals the running figure after the sweep, with the message `the survivors hold
{survivor_bytes} body bytes, the running figure says {held}`.

Inversion: the `release_body_bytes(shrunk)` in `native_mutable_buffer_setbuffersize` replaced by
`let _ = shrunk;`, then
`CARGO_TARGET_DIR=$H/target memcap 8G cargo test -j 4 -p rexx-exec --lib bytes_a_body_stops`:

```
thread 'rexx-interp' (2083873) panicked at crates/rexx-core/src/heap.rs:371:9:
assertion `left == right` failed: the survivors hold 2010209 body bytes, the running figure says 42010208
```

The same inversion in release (`--release`, where the assertion is compiled out) fails the new
test's pin instead: `left: 8, right: 12`. The file was restored from a copy taken before the edit
(`$H/buffer.rs.keep`); `git status` was clean in `rust/` afterwards.

## Tests

* The R10 tests (`large_dead_strings_...`, `copied_strings_...`, `sized_arrays_...`,
  `grown_arrays_...`, `reshaped_arrays_...`, `grown_buffers_...`, `short_strings_keep_the_slot_cadence`,
  `heap::body_bytes_tests::a_collection_sums_the_survivors_body_bytes`) pass with their code
  unchanged. The last one's doc comment said the survivors are summed from both mark loops, which
  is no longer true in release; that sentence was reworded, its body not touched.
* New: `rexx-exec/src/tests.rs` `bytes_a_body_stops_holding_leave_the_live_figure`. A 40 MB
  `MutableBuffer` live through a loop of 100 dead 1 MB copies, cut by `setBufferSize(1)`, another
  loop, a live 40 MB string, another loop, `drop` of the string, another loop. It pins
  `collections == 12`.
* Against the base: `git archive 7aedf9501 rust interpreter` into `$H/base-src`, this commit's
  `tests.rs` diff applied with `patch`, every file touched, `CARGO_TARGET_DIR=$H/target-base memcap 8G
  cargo test -j 4 -p rexx-exec --lib bytes_a_body_stops`: `Compiling rexx-exec` present, 1 passed.
  The base, which sums the survivors at every collection, gives the same 12.

## Performance

Binaries, sha256 and the full tables are in the gate record, `## Task 9`, `### Heapshape round 1`.
`callgrind.sh -r 2` over the eight programs, base61 (sha256 matches `## Task 1`), head
(`7aedf9501`), r1 (`5214a2089`); exit 0, spreads at most 0.0001%. Against base61:

| program | head % | r1 % |
|---|---:|---:|
| pingmsg | -0.2545 | +0.0096 |
| pingguard | -0.3663 | -0.3664 |
| pingsem | -0.2494 | -0.2494 |
| alloc | +0.0738 | +0.2330 |
| alloc4c | -0.8139 | +0.0607 |
| heapshape | +1.1559 | **+0.0525** |
| rexxcps | +0.0924 | **+0.4409** |
| emptyloop | -0.3191 | -0.3191 |

Target met: heapshape is within +0.5% of base61 and no program is over +0.5%. `cgdiff.py` head
against r1 puts the change in `Interp::collect_now`: heapshape -25.75M, rexxcps +61.99M, alloc
+29.39M, alloc4c +27.47M, pingmsg +4.27M. The cost moved from a `held_bytes` per survivor to one
per freed body, and garbage-heavy programs free far more than they keep.

One variant was tried and dropped: a `Bytes::held_len` (one match instead of `is_inline` then
`as_slice`) and the sweep's class test folded into the same match. At `-r 1` it measured rexxcps
+0.4262% and alloc4c +0.9132% against head, worse than r1's +0.3482% and +0.8817% (built in the
working tree rather than an archive, so its paths differ; not a controlled comparison). Its files
were restored from `HEAD` with `git show HEAD:path > path`.

Wall clock (`wallclock.sh`, interleaved, `$H/wall2` at `-r 5`, `wall3` and `wall4` at `-r 11`) is
inside ±4% of base61 everywhere except pingsem (+8.94%, then +8.26%; head +1.63%, then +4.13%),
pingguard (+5.04%, then +4.20%), and emptyloop (+4.43%, then +3.48%; head +5.36%, then +3.02%).
Against head with a layout control (head plus `layout-pad.py 48`): pad48 pingsem +1.59%, pingguard
+0.00%; r1 pingsem +3.17%, pingguard +3.28%. r1 and head execute the same instructions on pingsem
and pingguard (310 and 114 Ir apart), so this is not added work. It is not attributed further:
`perf stat` here rejected `task-clock` and I did not pursue it.

## Per-task check

At `5214a2089`'s tree:

* `cargo fmt`: no change. `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings`:
  exit 0.
* `memcap 8G cargo test -j 4 --workspace --no-fail-fast` (debug, so with the check live):
  exit 0, 3122 passed, 0 failed, 4 ignored (`$H/logs/test3.log`).
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
  ir_recorded_oracle`: exit 0, corpus 29 passed and 1 ignored, ir_recorded_oracle 21 passed.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests
  the_seeded_gate`: exit 0, 1 passed (124.6 s).

All with `CARGO_TARGET_DIR=$H/target`. The base test and the head and layout-control binaries
used a second target directory, `$H/target-base`, so that the base tree's build did not invalidate
this one's.

## Concerns

1. **rexxcps is at +0.4409% against base61**, 0.06% under the budget. The per-freed-body read is
   the cost, and it grows with garbage. A second round could skip it for bodies that cannot hold
   bytes using a per-slot bit set at charge time, or count released bytes where a body is dropped
   rather than reading every dead body. Neither was tried.
2. **pingsem and pingguard wall clock** are over ±4% against base61 with identical instruction
   counts to head; head itself is +1.6% to +4.1% on pingsem. Unattributed.
3. Two test edits outside the new test: `rexx-core/tests/collect.rs`
   `reference_cycles_are_collected` now builds its cycle by a slot write, and the R10 heap test's
   doc comment was reworded.
4. Exactness rests on every held-byte change being enumerated. A path the greps above miss shows up
   only as the debug assertion firing in a test that reaches it; in release it moves the trigger
   (a missed release keeps `bytes_due` high, a missed hold lowers it).

## Fix round 1

Review: `heapshape-round-review.md`. Base `fe66504f3`. Code commit `cc21b5ae3`.

### I1: growth on a mutator's error exit

`buffer_capacity(state, added)` answered the growth for its caller to charge, and five mutators
(`INSERT`, `OVERLAY`, `REPLACEAT`/`[]=`, `CHANGESTR`, `CASELESSCHANGESTR`) had a `?` between it and
the charge. The shape is gone rather than the five sites patched: `grow_buffer(interp, receiver,
name, added)` (`dispatch/buffer.rs`) grows the receiver's state, charges the growth, and only then
answers the state, re-borrowed. Every growing mutator goes through it, so no caller holds a growth
it has not yet charged: those five, `APPEND` (which called `ensure_capacity` directly), `SETTEXT`
and `SPACE`. A caller that read the length first (`INSERT`, `REPLACEAT`) reads it before the call.

Re-check of every other growth and charge for the same shape, with
`grep -rn "charge_growth\|charge_text(\|alloc_charged" crates/rexx-exec/src` and reading each site
from the growth to the charge:

* `array_resize`: its one `?` is the `try_reserve_exact`, before any capacity change.
* `array_reshape`, `array_splice_slot`, `array_grow`, the notify-list push: no `?` between.
* `MutableBuffer~new` (`buffer.rs:378`): the `?`s (`try_reserve_exact` on a local, `new_instance`)
  precede the state's installation; the charge follows it directly.
* `setBufferSize`: `set_buffer_size`'s `?` returns before it writes `capacity`, so a failure grows
  nothing.
* Native API `new_mutable_buffer` and `set_mutable_buffer_capacity`: no `?` (the reservation is
  `unwrap_or(0)`).
* Text charges (`text`, `text_owned`, `take_piece`, `new_raw_string`) and `alloc_charged`: the
  charge follows the allocation with no `?` between.

**Test.** `rexx-exec/tests/buffer_growth_error_exit.rs`
`a_buffer_grown_before_a_failed_insert_is_counted` runs the debug `rexx-run`
(`CARGO_BIN_EXE_rexx-run`) under `ulimit -v 3500000`, the pattern `library_routine_memory.rs` uses.
`b~insert('z', 2000000000)` under `signal on syntax` grows the buffer to 2 000 000 001 bytes, then
fails the result's reservation; the handler churns 200 000 objects with the buffer live, drops it,
and churns again. It asserts rc 0, stdout `5.0 2000000001` and `done`, and empty stderr. The cap was
chosen by raising it by hand against the reviewer's probe on the debug build (`$H/p2/g.rex`, under
`memcap 2G` or `3G`): before the fix, 2 000 000 fails the buffer's reservation, and 3 000 000 to
4 500 000 fail only the second; after it, 3 500 000 to 4 500 000 fail only the second and 5 000 000
makes both (the run then writes the 2 GB, so the cap must stay below that).

* At HEAD (`fe66504f3` archive plus this test file, `$H/head2-src`, `Compiling rexx-exec`
  present, `$H/logs/i1-at-head.txt`): fails, rc 101, stderr `the survivors hold 2000010805 body
  bytes, the running figure says 11060`.
* At `cc21b5ae3`: passes.

The report's enumeration gains a sixth item: a `?` between a capacity change and its charge, which
greps that find the growth and the charge in one function do not see. Found by reading every
charge site, listed above.

### Minors

1. **`get_mut` contract.** `Heap::get_mut` now documents that a caller changing what a body holds
   outside its slot records it with `hold_body_bytes`, `release_body_bytes` or `rehold_body_bytes`
   before any step that can fail, and that `peek_mut` is under the same rule.
2. **Unwitnessed release sites.** Not tested, for these reasons. `finish_string` and the
   `writable` failure are reached only through the native API (a loaded library calling
   `RexxStringObject` finishing or `SetMutableBufferCapacity` under a failed reservation); no crate
   test loads a library that does either, and building one is not cheap. The `array_reshape` shrink
   needs an array whose slot capacity exceeds the slots a multidimensional extend builds; the
   reviewer's `a~empty` route refuses with 93.926, and I found no route that leaves spare capacity
   on a multidimensional array (its slots are built exact by `empty_slots` and replaced exact by
   the reshape). `stream_init`/`stream_uninit` against a buffer: the reviewer's `run` of the
   stream's `INIT` on a `MutableBuffer` fails with 97.1, and I did not find another route. The
   dispositions for these four rest on reading, as the review says.
3. **Gate record.** pingsem and pingguard now read `callgrind inside; wall clock unresolved` in the
   round 1 table, and the fix round's paragraph keeps that verdict.
4. **rexxcps margin.** Accepted as stated: rexxcps is +0.4409% against base61 at `cc21b5ae3`, 0.06%
   under the budget, and the per-freed-body read grows with garbage. Carried to Tasks 10 to 12's
   planning; no change in this round.

### Perf

`callgrind.sh -r 2` over the eight programs, base61 / head2 (`fe66504f3`) / fr1 (`cc21b5ae3`), exit 0,
spreads at most 0.0001% (`$H/cg3`; binaries and sha256 in the gate record). head2 and fr1 are equal
to four places against base61: pingmsg +0.0096, pingguard -0.3663, pingsem -0.2494, alloc +0.2330,
alloc4c +0.0607, heapshape +0.0525, rexxcps +0.4409, emptyloop -0.3191. fr1 against head2 is at
most 6,090 Ir (rexxcps).

### Per-task check at `cc21b5ae3`

* `cargo fmt`; `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings` exit 0;
  `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets --features pinning,sharing -- -D
  warnings` exit 0.
* `memcap 8G cargo test -j 4 --workspace --no-fail-fast`: exit 0, 3127 passed, 0 failed, 4 ignored
  (`$H/logs/test4.log`).
* `REXX_CORPUS_GATE=1` corpus pair: exit 0, corpus 29 passed and 1 ignored, ir_recorded_oracle 21
  passed.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests
  the_seeded_gate`: exit 0, 1 passed (124.3 s).

### Concerns after this round

1. The new test depends on the debug binary's address-space footprint staying under about 1.5 GB,
   so that the buffer's 2 GB reservation fits a 3.5 GB cap. A larger footprint fails that
   reservation, and the test then fails on its stdout, not silently. The two reservations together
   need 4 GB, more than the cap, so no footprint lets both succeed.
2. rexxcps margin and the pingsem/pingguard wall clock, as before.
