# f4 profiling pass -- working notes

Raw material behind entry 2 of `docs/superpowers/plans/phase-4f-record.md`.
Nothing here is a conclusion the record does not carry; this file holds the numbers the record quotes and the commands that produced them.

## Provenance

* Repo at `610cb4efe9ae9378a63560eb584c982bab64dee1`, `git status --porcelain` empty before and after.
* `rust/target/release/rexx-run` sha256 `f166bb30215747c9753379d2009011f8956c6695bd5efce130711a1999c98255` -- byte-identical to entry 1's.
* Oracle `/home/moritz/dev/repos/ooRexx/build`, the same three objects entry 1 fingerprints.
* `samply` 0.13.1, `--save-only`, 1 kHz on this crate and 4 kHz on the oracle (the oracle's runs are about a second, so 1 kHz gives about 1100 samples).
* Every run from a fresh empty directory, `/dev/null` on stdin, `ulimit -v 8388608`, stdout and stderr as separate files.
* This crate's arm named explicitly: `REXX_ENGINE=ir` on every run, in the script rather than inherited.
* Analysed through `pollard` with `expand_inlines`; `unsymbolicated_pct` 0.0012% to 0.020% across the thirteen profiles.

Scripts: `prof-ir.sh`, `prof-oracle.sh` in the session scratchpad at
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/f4prof/`.

## The profiler did not distort the workload

Profile durations against entry 1's two IR blocks and two oracle-side medians.

| axis | IR profile | entry 1 IR medians | oracle profile | entry 1 oracle medians |
|---|---:|---|---:|---|
| `varlookup` | 4557 ms | 4552.2 / 4551.6 | 1183 ms | 1214.3 / 1219.3 |
| `arith` | 3136 ms | 3086.4 / 3085.9 | 1174 ms | 1158.8 / 1156.6 |
| `compound` | 6765 ms | 6741.6 / 6743.5 | 1181 ms | 1141.5 / 1144.8 |
| `strings` | 9183 ms | 9141.4 / 9149.3 | 851 ms | 863.5 / 865.8 |
| `alloc4c` | 2350 ms | 2278.9 / 2279.3 | 1162 ms | 1145.1 / 1157.9 |
| `emptyloop` | 2867 ms | 2838.0 / 2825.5 | -- | -- |
| `rexxcps` | 4796 ms | -- | 1815 ms | -- |

Largest deviation on this crate's side is `alloc4c` at +3.1%; every other axis is inside 1.6%.
Every axis printed the same bytes on both sides (`varlookup` 19000000, `arith` 4629643519330627.7808, `compound` 5000000, `strings` 138000000, `alloc4c` 12888896).

`rexxcps` self-calibrated differently on the two sides, as entry 1 records: the oracle ran `200 x 100` and this crate `100 x 100`, so the two wall times are not comparable and only the cps figures are (oracle 16,667,722, this crate 2,227,880 -- 7.48x, against entry 1's 7.35x).

## Per-axis shares, IR arm

Subtree totals unless the row says self.

### `varlookup` (4575 samples on `rexx-interp`)

| what | share |
|---|---:|
| `Interp::slot_of` -> `Plan::slot_of`, the **name-keyed** `HashMap<Box<[u8]>, usize>` | **29.1%** |
| `[u8]::to_vec`, the fresh `Vec<u8>` copy of the name that lookup is given | 6.3% |
| `HashMap<SymbolId, usize>::get`, the **id-keyed** map on the read side | 6.3% |
| **the three together** | **41.7%** |
| `leave_clause::<RegionEnd>` self -- the clause boundary's own copy | 11.4% |
| `loop_advance` (whole) | 25.1% |
| -- of which `bind_control` -> `slot_of` | 33.9% of `loop_advance` |
| -- of which `Interp::read` -> id-keyed get | 25.4% of `loop_advance` |
| `run_repeating` self | 7.3% |
| `run_ops::<false>` self -- the op-dispatch loop itself | 4.9% |
| `clause_line` -> `ProgramSource::line_of`, the line-table binary search | 2.8% |
| glibc allocator family, self | 5.3% |
| hashbrown + SipHash + `memcmp`, self | 26.1% |

### `arith` (3127 samples)

| what | share |
|---|---:|
| `arith_general` | 80.2% |
| -- `Number::div` | 28.5% |
| -- `Interp::number` | 19.4%, of which `small_int_for` **14.8%** and `Number::format_with` 13.8% |
| -- `Number::add_signed` | 11.1% |
| -- `Number::pow` | 8.2% |
| glibc allocator family, self | 34.7% |
| this crate's arena side, self | 4.5% |
| `Interp::slot_of` | 2.3% |
| `run_ops::<false>` self | 1.1% |

### `compound` (6800 samples)

| what | share |
|---|---:|
| **`Number::div`** -- `k = i // tails` with no small-integer path | **42.0%** |
| `Interp::tail_key` | 12.0% |
| `rexx_parse::ast::compound_parts`, the run-time tail split | 9.9% |
| `Interp::slot_of`, name-keyed | 10.3% |
| `stem_get` + `stem_set` | 5.0% + 4.7% |
| `loop_advance` | 5.5% |
| glibc allocator family, self | 29.4% |
| this crate's arena side, self | 0.0% |
| `memcmp` self | 13.9% |
| `run_ops::<false>` self | 1.9% |

### `strings` (9184 samples)

| what | share |
|---|---:|
| glibc allocator family, self | **36.8%** |
| this crate's arena side (`alloc_with_uncollected` + `ptr::write::<Slot>`), self | **8.9%** |
| `builtin::dispatch` (whole subtree) | 30.7% |
| -- `is_builtin` (UTF-8 validation + `&str` hash set) | 5.4% |
| -- `Iter<Builtin>::find`, the linear scan | 3.7% |
| -- `changestr` | 11.4% |
| -- `pos` / `find_forward` | 5.4% / 5.1% |
| -- `substr` | 5.1% |
| `Interp::concat` | 6.5% |
| `Interp::slot_of` | 4.6% |
| `run_ops::<false>` self | 1.7% |

### `alloc4c` (2353 samples)

| what | share |
|---|---:|
| glibc allocator family, self | 38.8% |
| this crate's arena side, self | 3.7% |
| `line_of` binary search | 6.2% |
| `leave_clause::<RegionEnd>` self | 3.7% |

### `rexxcps` (4782 samples)

| what | share |
|---|---:|
| glibc allocator family, self | 38.7% |
| this crate's arena side, self | 9.7% |
| `Interp::eval` (the tree-walker expression path) | 45.5% |
| `line_of` binary search | 4.5% |
| `run_ops::<false>` self | 1.4% |

### `emptyloop` (2833 samples) -- read with its recorded codegen sensitivity in mind

| what | share |
|---|---:|
| `loop_advance` | 53.9% |
| `line_of` binary search | 12.0% |
| `run_repeating` self | 11.3% |
| `leave_clause::<Flow>` self | 6.5% |

## The oracle on the same axes

| axis | what the oracle spends its time on |
|---|---|
| `varlookup` | `MemoryObject::newObject` 55.6% total, `DeadObjectPool::findFit` 23.8% self, `RexxInteger::plus` 66.2% total. `RexxActivation::getLocalVariable` **1.3% total** -- variable binding is an array index. |
| `arith` | `NumberString::addMultiplier` **41.0% self**, `multiplyPower` 36.7% total, `Division` 29.2% total, `subtractDivisor` 7.4% self. Allocation is small: `newObject` 7.8% total. |
| `arith`, grouped | every `NumberString`/`Numerics::` frame: **74.0% self, 91.9% total**. Against every `rexx_num::` frame on this crate: **18.3% self, 74.9% total**. In seconds: oracle 0.869 self / 1.079 total of 1.174; this crate 0.574 self / 2.349 total of 3.136. The kernels are not the gap; their callees are (0.21 s against 1.78 s). |
| `compound` | `memcmp` 19.7% self, `CompoundVariableTail::compare` 25.2% total, `findEntry` 19.8% total, `buildTail` 16.3% total, `formatWholeNumber` 6.6% self. **`RexxInteger::remainder` 7.7% total** -- the `//` is an integer remainder, not decimal division. `newObject` 17.6% total, `collect()` 2.2%. |
| `strings` | `findFit` 16.5% self, `newObject` 32.7% total, `allocateObject` 25.0% total, `StringUtil::pos` 8.7% total, `builtin_function_SUBSTR` 4.7%, `CHANGESTR` 4.9%. |
| `alloc4c` | `newObject` **74.5% total**, `collect()` **60.5% total**, `markObjectsMain` 50.0% total, `CompoundTableElement::live` 20.9% self. |
| `rexxcps` | `newObject` 22.5% total, `findFit` 7.7% self, `RexxActivation::run` 5.7% self, `RexxBinaryOperator::evaluate` 45.7% total. |

## Two facts read from the binary rather than from a profile

* **`size_of::<Failure>()` is 104 and `size_of::<Raised>()` is 104**, read with
  `gdb -batch -ex 'print sizeof(rexx_exec::error::Failure)' target/release/rexx-run`.
  `FailureSite` is 40.
* **The compiled clause epilogue in `run_ops::<false>` copies 0x68 = 104 bytes stack-to-stack, twice**, at `+3605`..`+3696` and `+3728`..`+3752`, through `movaps`/`movups` pairs behind two discriminant tests.
  One instruction of that sequence, `movaps xmm1, xmmword [rsp + 0xd0]` at `+3657`, carries **509 of `varlookup`'s 4575 samples**.
  A sequence of aligned 16-byte loads reading back what 8-byte stores just wrote is the shape that blocks store-to-load forwarding, which is one candidate explanation for why a copy costs that much; the counter that would settle it (`ld_blocks.store_forward`) is not exposed by `perf` on this machine.
  The width and the copies are static properties of the emitted code either way.

## Peak resident set, both sides

`/usr/bin/time -f %M`, same wrapper, one run each.

| axis | this crate, IR | oracle | ratio |
|---|---:|---:|---:|
| `strings` | 3,728,048 KB | 20,728 KB | 180x |
| `alloc4c` | 579,432 KB | 536,808 KB | 1.08x |
| `arith` | 440,604 KB | 20,472 KB | 21.5x |
| `varlookup` | 150,264 KB | 20,484 KB | 7.3x |
| `compound` | 41,464 KB | 20,504 KB | 2.0x |

`rexxcps` on this crate peaks at 1,798,548 KB.

## Peak resident set, IR against tree-walker

Taken to settle whether the entry state's 3% wall-clock regression on `compound` and `rexxcps` is a footprint effect.

| axis | IR | tree-walker |
|---|---:|---:|
| `compound` | 40,640 KB | 41,688 KB |
| `strings` | 3,727,388 KB | 3,728,840 KB |
| `varlookup` | 149,728 KB | 150,184 KB |
| `rexxcps` | 1,798,548 KB | 1,799,196 KB |

Within 0.3% on all four, and the IR arm is the lower of the two on every one.
So the regression is not working-set size.

## One thing looked for and not found

The record and the plan both cite "`smallvec` for `Number::digits` was measured and rejected" as the example of a preserved dead end.
Case-insensitive searches of the working tree, of `git log -S` over all refs, and of `.superpowers/` find the string in exactly two places: those two sentences.
**The measurement they cite is not in this repository.**
Anyone attempting an inline representation for `Number`'s digits should find the original figures or re-measure and record them, rather than treating the citation as the result.
