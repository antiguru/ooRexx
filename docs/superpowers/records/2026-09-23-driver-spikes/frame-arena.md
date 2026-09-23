# Spike: frame-arena

Branch `spike/frame-arena`, cut from `0ba0f3876` (verified `git rev-parse HEAD`
= `0ba0f387664dac94e91bf62f9acbf6dbeb034241` after `git switch -c`; the
worktree tool had first cut the worktree from the master lineage at
`c2edef977`, which I refused to build on).

Base binary: `bin/base-rexx-run`, built from `0ba0f3876` with
`CARGO_TARGET_DIR=<scratch>/target-base cargo build --release -p rexx-exec --bin rexx-run`.

    .text sha256 748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32

## Prediction (written before any edit or measurement)

Design I intend to build: a `FrameArena` in a new `rexx-core/src/frame.rs`
of fixed blocks that never move; a `Frame` handle (base pointer + length)
that replaces `FrameId` as the driver's `registers` argument, so that the
base is already a local of `run_ops_from` and no access goes through
`self.roots`; accessors take `u16`; a validator wrapping the chunk's op vector.

Reasoning. `roots.rs` is 28.92 Ir/clause on `rexxcps`, but that file slice
also holds `frame_slot`/`set_frame_slot` (variable slots, not registers),
which this spike does not touch. I guess the register half is about 18-20 of
it. A `temp_at` today is roughly: load `temps.ptr` and `temps.len` from
`self`, add the frame offset, compare, branch, load: ~6. Through a local base
pointer it is ~1-2 (the base will likely be spilled in a 1,416-byte frame, so
one stack reload plus the indexed load). Saving ~4 per access over ~4-5
accesses per clause is ~15-18 Ir/clause, about -1.5% to -1.8%. Against that,
`Frame` is 16 bytes where `FrameId` was 8, passed to helpers and held across
the region walk, which adds pressure; and the regalloc noise is +-2.5%.

| axis | unchecked-arena vs base | checked-arena vs base |
|---|---:|---:|
| rexxcps | -1.5% | -0.8% |
| varlookup | -2.0% | -1.2% |
| arith | -1.0% | -0.5% |
| emptyloop | -1.5% | -0.8% |
| dispatch | -1.0% | -0.5% |
| compound | -1.0% | -0.5% |

So: arena (base vs checked) about -0.8% and the check (checked vs unchecked)
about -0.7% on `rexxcps`. Both halves are inside the 2.5% noise floor, so my
expected verdict is **inconclusive** unless the effect is larger than I think
or the two controls line up in sign across every axis.

## What was built

Commits on `spike/frame-arena`:

* `a24ed03e7` the unchecked arena (the head measured as "unchecked").
* `1429029df` the same with `CHECKED = true` (the control, measured as "checked").
* `563b205bc` flips `CHECKED` back; its tree is identical to `a24ed03e7`
  (`git diff a24ed03e7 HEAD --stat` is empty).

Pieces:

* `rust/crates/rexx-core/src/frame.rs` (new, the only `unsafe`): `FrameArena`
  of blocks of `2 * 65536` cells, allocated with `alloc_zeroed` and leaked, so
  they never move and are never freed. A `RegFrame` is `{ base, len: u16,
  block: u16, start: u32 }` (16 bytes; `FrameId` was 8). `get(u16)`/`set(u16, _)`
  read through `base` with no bound check (a `debug_assert!` against `len`;
  `CHECKED` turns it into an `assert!`). **Memory safety does not rest on the
  validator**: a frame is only carved at an offset `<= 65536` of a block of
  `131072` cells, and the index is a `u16`, so `base + index` is always inside
  the block; blocks are leaked so no handle can dangle; every `u64` bit pattern
  is a valid `ObjRef`; cells are `Cell<ObjRef>` so the collector's shared reads
  and the driver's writes do not alias a `&mut`. The pointer is derived from
  the whole tail slice so its provenance covers every reachable cell.
  Reservation fills the frame with `ObjRef::NIL`, as `reserve_temps` did.
  Release asserts LIFO order.
* `RootSet` gains `frames: FrameArena`, `reserve_frame`/`release_frame`, and
  its `iter()` walks every live frame (after `temps`, as before).
* `rust/crates/rexx-exec/src/ir/valid.rs` (new): `ValidOps(Vec<Op>)`, private
  field, one constructor that refuses any register operand `>= registers`
  (`ARG_OMITTED` excepted), exhaustive match with no wildcard so a new op
  variant fails to compile until it is classified. `Chunk::ops` is `ValidOps`,
  so no `Chunk` exists with an unvalidated stream; `compile` returns the
  refusal as `ChunkTooLarge`.
* Driver and the two other reservers (`run_chunk`, `run_fragment` for
  `INTERPRET`, `flat_loop_start`'s snapshot write): `FrameId` ->
  `RegFrame`, every `self.roots.temp_at(registers, x as usize)` ->
  `registers.get(x)`, every `set_temp` -> `registers.set`. `temps` keeps
  `push_temp`, `push_frame`/`pop_frame` for everything else.
* The frame base is **a local the driver holds** (the `RegFrame` value
  argument), never re-read from `self`. The alternative (base re-read from
  `self` per access) was not built or measured.
* `crates/rexx-core/tests/unsafe_sites.rs`'s `unsafe`-block list gains
  `frame.rs` (this branch only; the opt-in is on `mod frame` in `lib.rs`).
* Unit tests: `frames_spill_into_a_second_block_and_come_back` (arena block
  spill, collector walk, LIFO release) and
  `a_register_at_the_count_is_refused_and_one_below_it_is_accepted`.

Binaries (`.text` sha256):

    base       748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32  (0ba0f3876)
    unchecked  13b33e7bd117b9393e7dd81e9f0506269eb2cf2d8547a270f55c337e685973ab  (a24ed03e7)
    checked    767f48864a05882ce014c4cd684080393eb6ab50463d4e657c212f5b5a897500  (1429029df)

Each built in its own target dir (`target-base`, `target-head`,
`target-checked`) and copied to `bin/`.

## Measurements, first version (a24ed03e7 / 1429029df)

`valgrind --tool=callgrind` `summary:` minus every cost whose object is
`libc.so.6` or `ld-linux`, computed by `exlibc.py` (self cost by `ob=`; its
self-sum equals `summary:` to the unit on every run). Two rounds, round 1 in
the order base, unchecked, checked and round 2 reversed, each from a fresh
empty directory under `runs/`. Every run rc 0.

| axis | base | unchecked | d% | checked | d% | checked->unchecked |
|---|---:|---:|---:|---:|---:|---:|
| rexxcps | 18,638,741,166 | 18,327,872,336 | -1.668% | 18,531,421,092 | -0.576% | -1.098% |
| varlookup | 17,122,907,746 | 16,457,902,346 | -3.884% | 16,913,900,470 | -1.221% | -2.696% |
| arith | 11,764,231,177 | 11,667,920,366 | -0.819% | 11,722,919,510 | -0.351% | -0.469% |
| emptyloop | 9,685,889,023 | 9,585,891,038 | -1.032% | 9,685,887,514 | -0.000% | -1.032% |
| dispatch | 20,711,077,650 | 20,896,079,849 | +0.893% | 21,036,082,794 | +1.569% | -0.666% |
| compound | 9,914,438,048 | 9,664,422,952 | -2.522% | 9,819,435,131 | -0.958% | -1.579% |

Round-to-round spread is under 25,000 instructions on every cell except
checked `rexxcps` (7,553,315, 0.04%).

Findings from this version, each from `fndiff.py` (per-function self cost,
`callgrind_annotate`, A against B):

* **`dispatch` regressed because of `reserve_frame`, not the accessors.**
  `RootSet::reserve_frame` appeared out of line at 385,000,937 Ir, 77 per call
  over 5,000,000 method invocations. Fixed in the second version.
* **`emptyloop` has no register traffic** (`frame.rs` slice 308-682 Ir in
  both heads) and still moved -1.032% unchecked and 0.000% checked. That whole
  -100,000,871 is inside `run_ops_from::<true>` self cost: the check's
  presence elsewhere in the function moved an unrelated path by 4 Ir per
  iteration. **This is the regalloc noise the brief warns about, visible on an
  axis where the edit does nothing,** and it is as large as the effect on
  `rexxcps`.
* `varlookup`'s whole -665,000,976 (unchecked vs base) is in
  `run_ops_from::<true>` self cost, which is where the inlined accessors land.
* `rexxcps` changed inlining: `loop_advance` (749,923,722 Ir in base) is gone
  as a function in the unchecked head and `run_ops_from::<false>` appears; the
  checked head differs from the unchecked one by the same kind of shuffle
  (`alloc_with`, `eval_node`, `loop_advance` each appearing or vanishing).
* A correctness defect: `run::tests::conditions::a_trap_that_resumes_does_not_accumulate_temps_frames`
  failed, 600 against 1200. Popping the register region off `temps` at chunk
  exit also truncated any temps a trapped clause had leaked; moving the
  registers out removed that. Fixed in the second version by keeping a temps
  watermark around each chunk run.

### Correction: the "inlining shuffle" above was a naming artefact

Checked at the coordinator's request (`inlinecheck.py`, `callgrind_annotate
--auto=no`, self Ir with the recursion-cycle suffixes `'2` merged, and `nm -C`
per binary). The `fndiff.py` rows that showed `loop_advance` or `eval_node`
"vanishing" compared differently named recursion-cycle members, not different
inlining. With cycles merged:

* `nm`: every binary (base, both v1 heads, both v2 heads) defines
  `run_ops_from::<true>`, `run_ops_from::<false>` and `arith_small_int`, so
  `run_ops_from::<true>` was not inlined into `run_activation` anywhere.
* `arith_small_int` has no self-cost row on any axis in any binary (inlined in
  all of them), so it did not flip.
* `drop_glue::<Option<LoopHeaderValues>>` self Ir is identical base vs every
  head on every axis (`rexxcps` 28,201,025; 20 to 25 elsewhere). No flip.
* `loop_advance` self Ir is identical base vs every head on every axis.
* So on every axis the difference is in `run_ops_from` self cost plus the
  arena's own functions. `rexxcps` v1 unchecked vs base: `run_ops_from::<true>`
  -264,651,090, `::<false>` -25,760,000, `reserve_frame` +26,320,808.
* `emptyloop`'s -100,001,110 is `run_ops_from::<true>` self (4 Ir per
  iteration) with no register access on that path: the noise witness stands.

## Second version (9c5502476 unchecked / 8434023ce checked; head 249cba953)

Changes: the arena keeps the current block and its top in two fields, and
switching blocks is `#[cold]`, so `reserve`/`release` are a compare, a NIL
fill and two stores. `run_chunk` and `run_fragment` take a `temps` watermark
around the run again (the correctness fix above). `refusal-sites.tsv`
re-derived with `REXX_REFUSAL_SITES_REFRESH=1`; its diff is line numbers only
(checked by normalising `run.rs:N` and pairing every removed row with an added
one).

Binaries (`.text` sha256):

    unchecked2  9eeda4d6bfe5a3d427de48980095a17583c2643552a83cf4082191f7c7c53e0a  (9c5502476 == 249cba953 tree)
    checked2    5b802be5dcabaee722644eef9545f53899d09510d8ecb3c88ba2d96e0500e978  (8434023ce)

Rounds 3 (base, unchecked2, checked2) and 4 (reversed). Base is the mean of
all four base rounds; its spread is under 25,000 on every axis.

| axis | base ex-libc | unchecked ex-libc | d% | checked ex-libc | d% | checked->unchecked |
|---|---:|---:|---:|---:|---:|---:|
| rexxcps | 18,638,736,321 | 18,319,174,497 | **-1.715%** | 18,518,953,250 | -0.643% | -1.079% |
| varlookup | 17,122,903,999 | 16,457,901,116 | -3.884% | 16,913,897,296 | -1.221% | -2.696% |
| arith | 11,764,228,658 | 11,667,923,978 | -0.819% | 11,722,920,486 | -0.351% | -0.469% |
| emptyloop | 9,685,885,937 | 9,585,886,730 | -1.032% | 9,685,888,372 | +0.000% | -1.032% |
| dispatch | 20,711,077,922 | 20,731,082,298 | +0.097% | 20,871,079,176 | +0.773% | -0.671% |
| compound | 9,914,442,674 | 9,664,422,318 | -2.522% | 9,819,435,674 | -0.958% | -1.579% |

Spread between rounds 3 and 4, per head cell: at most 5,457.

Only `dispatch` moved between the two versions (+0.893% to +0.097%, and the
checked control by the same 165,000,000): the v2 arena changed reservation and
nothing in the driver. On `dispatch` the arena's reservation, release and
temps watermark are now inlined into `run_activation`, whose self cost rose
190,000,151 (38 per method call), against `run_ops_from::<true>` -170,001,154.

### The accessor cost the brief asked for, on `rexxcps`

File slice (`exlibc.py`, self cost by the source file of each line, inlined
lines to their own file), per clause over 20,000,000 clauses:

| | `roots.rs` | `frame.rs` | together | /clause |
|---|---:|---:|---:|---:|
| base | 577,519,210 | 0 | 577,519,210 | 28.88 |
| unchecked2 | 232,966,234 | 6,441,122 | 239,407,356 | 11.97 |
| checked2 | 232,966,234 | 210,949,087 | 443,915,321 | 22.20 |

`roots.rs` in the heads is what remains there (variable slots, temps, the
arena calls through `RootSet`), so the register file's own share of base was
about 344,552,976 (17.23/clause). The unchecked accessors' own lines retire
6,441,122 (0.32/clause): the indexed load itself is attributed to the
driver's line, so this undercounts, but the difference between the two heads
(204,507,965, 10.23/clause) is the check alone.

### The inlining check (coordinator's two items), second version

Identical result to the first: `nm` shows `run_ops_from::<true>`,
`run_ops_from::<false>` and `arith_small_int` defined in base, unchecked2 and
checked2; `arith_small_int` has no self row anywhere; `drop_glue::<Option<LoopHeaderValues>>`
and `loop_advance` self Ir are identical base vs both heads on every axis
(`rexxcps` 28,201,025 and 195,120,629). The whole difference on every axis is
`run_ops_from` self cost plus the arena's reservation.

## Correctness floor, at 249cba953 (tree == 9c5502476)

Status file `gates.txt`, first line the sha, each status unpiped; tests built
outside the cap first (`--no-run`, same package set), run under `memcap 8G`.

    249cba953d39f152f10414355c47d392eabe2373
    build rexx-exec tests (no cap): 0
    cargo test -p rexx-exec --release: 101     1547 passed / 32 failed / 1 ignored, 49 binaries
    build corpus (no cap): 0
    corpus_differential: 0                     "604 of 604 matching", "mode: STRICT (the gate) -- REXX_CORPUS_GATE is set"
    cargo test -p rexx-core --release: 0       76 passed / 0 failed
    finished

The `rexx-exec` failures are environmental, every panic one of three
messages: "the worktree's build/lib is three directories above this crate"
(14), "cannot read .../ootest/ooRexx/base/..." (13), "this test compares the
release binary; build it first" (5). This worktree has no `build/lib` or
`ootest/` three directories up. The two failures the first version caused
(`a_trap_that_resumes_does_not_accumulate_temps_frames` and
`the_table_holds_every_constructor_the_source_defines`) are not in the set.
The base comparison is below.

Base comparison: the same `cargo test -p rexx-exec --release` on a
`git archive 0ba0f3876 rust interpreter` copy (scratch `base-src/`, own target
dir) gives 1544 passed / 34 failed. **Head's failure set is a strict subset of
base's**: base additionally fails `every_divergent_row_has_a_known_gap`,
`the_prose_rows_and_this_table_name_the_same_divergences` and two unnamed
`result:` rows, which are relative-path failures of the copy's different
directory depth. No test fails on head that passes on base.

## The control, and what it separates

Two binary pairs, and one axis that the edit cannot touch.

* **checked -> unchecked (prices the check).** On every register-using axis
  the difference is accounted for by the check's own lines: `varlookup`
  455,996,180 between the heads against a `frame.rs` slice of 456,000,686 in
  checked and 306 in unchecked; `rexxcps` 199,778,753 against a slice
  difference of 204,507,965. So the -1.08% on `rexxcps` is the check's
  instructions, not relocation elsewhere.
* **But the same binary pair moves `emptyloop` by -1.032%**, with a
  `frame.rs` slice under 700 in either head and no register reads on its
  path; 100,001,001 of the 100,001,642 is `run_ops_from::<true>` self cost. The noise in this
  pair is therefore as large as the effect on `rexxcps`, in the same function,
  and a slice attribution cannot rule out an equal and opposite shift hiding
  under the `rexxcps` figure.
* **base -> checked (prices the arena).** `rexxcps` -0.643%, `dispatch`
  +0.773%, `emptyloop` 0.000%. No control separates this half; it is under the
  floor and uncontrolled.
* Outside the noise: `varlookup` -3.884% and `compound` -2.522% unchecked vs
  base, with -2.696% and -1.579% attributable to the check by the slice.

## Prediction against result

| axis | predicted unch | measured unch | predicted chk | measured chk |
|---|---:|---:|---:|---:|
| rexxcps | -1.5% | -1.715% | -0.8% | -0.643% |
| varlookup | -2.0% | -3.884% | -1.2% | -1.221% |
| arith | -1.0% | -0.819% | -0.5% | -0.351% |
| emptyloop | -1.5% | -1.032% | -0.8% | 0.000% |
| dispatch | -1.0% | +0.097% | -0.5% | +0.773% |
| compound | -1.0% | -2.522% | -0.5% | -0.958% |

`rexxcps` landed where predicted. I under-predicted the check on the
variable-heavy axes and missed two things: that `emptyloop` has no register
traffic at all (its move is pure layout), and that `dispatch` would pay for
reservation per method call.

## Verdict

**Inconclusive on `rexxcps`**: -1.715% is under the 2.5% floor, and although
the checked control attributes -1.08% of it to the bound check by that check's
own lines, the same binary pair moves `emptyloop` by -1.03% with no register
access, so the control does not separate the check from layout. The arena half
(-0.64%) is uncontrolled. The effect is real in magnitude on `varlookup`
(-3.9%) and `compound` (-2.5%).

## Concerns

* **Soundness depends on leaking.** Blocks (1 MiB each, `alloc_zeroed`, so
  untouched pages are not resident) are never freed, because a `RegFrame` is
  `Copy` and nothing ties it to the arena's lifetime. One block per
  interpreter in practice; a test process creating many interpreters leaks
  that much address space each. A non-leaking version needs a lifetime or a
  generation check on the handle.
* **Memory safety does not use the validator.** The `u16` index plus the
  64 Ki-cell slack after every frame start makes every access in bounds of its
  block whatever the index; the validator buys only "a register names this
  frame's own cells". That is stronger than the brief asked, and it means the
  brief's (2) is a logical invariant, not a safety one.
* **The temps watermark matters.** Moving registers off `temps` silently
  removed the truncation that healed leaked temps after a trapped clause;
  `a_trap_that_resumes_does_not_accumulate_temps_frames` caught it. Any
  production version must keep the watermark (as v2 does) or fix the leaks at
  their source.
* `RegFrame` is 16 bytes against `FrameId`'s 8, passed to helpers.
* The "base re-read from `self` per access" variant was not built; only the
  local-handle form was measured.
* `unsafe_sites.rs` was edited to admit `frame.rs`; that is the spike's grant
  only.
* Debug-build tests and clippy were not run (not required for a spike).
