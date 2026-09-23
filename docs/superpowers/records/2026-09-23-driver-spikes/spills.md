# Stack spills in the IR driver

Branch `perf/spills` from `9f2bc14d7` (`plan/rust-rewrite`). Brief:
`round5/spills.md` in the session scratchpad. Scripts and listings:
`spills-files/`.

## 0. Result

Stopped after three consecutive candidates failed to pay (s3, s4, s5); a `nop`
clause still executes 23 spill instructions (17 of them instructions of their
own), not under 10. One of five candidates kept, `c6a2ad7a1`. Base `.text`
`adaacbb0`, head `.text` `597b63b4`, two interleaved rounds each
(`results-s2.txt`):

| axis | base | head | delta |
|---|---:|---:|---:|
| `nop` | 10,040,098,902 | 9,340,106,478 | -6.972% |
| `assign` | 20,240,368,588 | 19,540,366,283 | -3.458% |
| `rexxcps` | 18,017,680,045 | 17,897,276,261 | -0.668% |
| `varlookup` | 15,108,900,993 | 14,842,898,293 | -1.761% |
| `emptyloop` | 9,460,884,587 | 9,285,884,489 | -1.850% |
| `arith` | 11,543,842,890 | 11,519,337,007 | -0.212% |
| `dispatch` | 20,556,080,056 | 20,471,078,416 | -0.414% |
| `compound` | 9,304,400,794 | 9,234,396,950 | -0.752% |

Per clause: `nop` 97 -> **90** (the oracle's is 33.3), `x = y` 199 -> **192**
(99.3), `rexxcps` 900.88 -> 894.86. Spill instructions per clause: `nop`
29 -> 23, `x = y` 45 -> 39, `rexxcps` 91.53 -> 86.34.

**The spill count did not track the instruction count.** Every rejected
candidate lowered the spill count on at least one axis whose instruction
count rose, and s4 lowered it on all three while raising all three. The one
kept change removed a value carried from clause to clause (a copy of an
undefined payload); none of the other four did, and all four lost. The spill count is the
frame-size instrument's failure over again, one level down.

## 1. Census, on the base, before any code change

Base binary: `cargo build --release -p rexx-exec --bin rexx-run` at
`9f2bc14d7`, `.text` sha256
`adaacbb0ac3b9699af34902a7eac67cb42c0964b304795afc0df573f9bef26bc` -- the
clause-overhead round's head hash, so the two rounds measure one binary.

### Method: telling a spill slot from a stack object

LLVM reports its own frame layout. Building with
`RUSTFLAGS="-Cremark=stack-frame-layout"` prints, per function, every stack
slot with its offset from the incoming `%rsp`, its size, and its **type**:
`Spill` (a register-allocator slot), `Variable` (an alloca: an array, a value
built in place, an address-taken local), or `Fixed` (a stack-passed
argument), plus the debug variables LLVM last recorded in it
(`layout.run_ops_from.txt` is both driver instantiations). The remark build
has the base's `.text` hash, so the remarks describe the measured binary.

`spills.py` places every executed `off(%rsp)` operand in that layout using
the function's own prologue (`push` count P, `sub $F,%rsp`; remark offset =
`off - F - 8P`; the drivers push six and subtract 1,416) and classifies it:
a `mov` into a `Spill` slot is a **store**, a `mov` out of one is a
**reload**, a `cmp`/`test`/ALU read of one is a **folded** reload (a memory
operand, no instruction of its own), any access to a `Variable` slot is a
**stack object**, a `Fixed` read is a stack **argument**; `push`/`pop` are
callee-saved saves. Per clause is section 1 of `clause-overhead.md`'s method:
callgrind `--dump-instr=yes` of 100- and 50-clause bodies over 20,000 passes,
differenced, divided by 1,000,000; `rexxcps` is its whole run divided by its
20,000,000 clauses, with libc and ld-linux excluded throughout. Only 1.53
instructions per `rexxcps` clause fell outside the model (`%rsp` moved
mid-function).

The slot names in the remark are the union of every debug variable that ever
lived there, so a reused slot lists unrelated names (`SP-696` says `text`
and holds the region iterator's end). Every name below was confirmed by
reading the instruction that fills the slot.

### Executed stack traffic per clause

| | `nop` | `x = y` | `rexxcps` |
|---|---:|---:|---:|
| instructions (ex libc) | 97 | 199 | 900.88 |
| spill stores | 9 | 15 | 34.90 |
| spill reloads (`mov`) | 14 | 21 | 41.14 (+0.42 read-modify-write) |
| folded spill reads | 6 | 9 | 15.06 |
| **spill total** | **29** | **45** | **91.53** |
| of which an instruction of its own | 23 | 36 | 76.46 |
| stack-object accesses | 2 | 2 | 59.95 |
| stack-argument reads | 1 | 1 | 4.03 |
| callee-saved `push`/`pop` | 0 | 0 | 84.47 |

From `census.{nop,asg,rexxcps}.base.txt`. `nop` and `x = y` run entirely
inside `run_ops_from::<false>`. On `rexxcps` that function carries 47.04 of the
91.53 (13.89 stores, 33.15 reloads and folded reads);
`exec_parse` 13.89, `collect_now` 5.23, `apply_binary` 3.02, and
`run_ops_from::<true>` 2.16 are next.

### What is spilled: every slot on the `nop` path

All in `run_ops_from::<false>`; offsets are the remark's. This covers 100%
of `nop`'s spill traffic; "ld" counts folded reads.

| slot | value | st+ld per `nop` |
|---|---|---:|
| `SP-1312` | `index` (the `Op::Clause`'s instruction index, `usize`), reloaded for `position_at`'s bound twice and for `current_clause_index` | 1+3 |
| `SP-876`/`SP-880` | `position: Option<ClausePosition>`'s payload (`line`, `indent`); `SP-876` is reloaded *before* the bound test and stored back after, a loop-carried copy of the previous clause's value that the `None` arm's undefined payload merges with | 2+2 |
| `SP-1080` | `end` (the `Op::Clause`'s region end), live across the region walk as its fall-through answer | 1+2 |
| `SP-1088` | `sink`, the `TraceCache` byte `chunk_trace()` read, for `stale` and for `debugging` after the region | 1+2 |
| `SP-1320` | `%r13`, the **sret pointer** for `Result<Flow, Failure>`, moved out of its callee-saved register for the clause and back | 1+1 |
| `SP-1424` | `SteppedClause::frame`, the temps length `push_frame` read, for `pop_frame` | 1+1 |
| `SP-600` | `chunk.positions.len()`, loop-invariant, read for the bound and again where `shortcut` re-tests the `Option` as `index < len` | 0+2 |
| `SP-1297`/`SP-1298` | `chunk.trace()`'s byte (invariant) and the `stale` result | 1+1 |
| `SP-1336` | `clause: &Instruction` | 1+0 |
| `SP-936` | `stop` | 0+1 |
| `SP-1040` | `chunk` (reloaded at the loop head for the `pc >= stop` exit, overwritten before use on the hot path) | 0+1 |
| `SP-592` | the stream pointer | 0+1 |
| `SP-1072` | `&code.body.instructions` | 0+1 |
| `SP-504` | `chunk.positions`' pointer | 0+1 |
| `SP-512` | the op stream's length, for `ops_in`'s bound | 0+1 |

The stack argument is `SP+16`, reloaded at the loop head beside `chunk` for
the same exit. The two stack-object accesses are `header:
Option<LoopHeaderValues>` (176 bytes) written `None` and drop-tested.

**Two populations.** Loop invariants (`stop`, `chunk`, stream, instruction
table, positions, `chunk.trace()`, stream length): 9 reads, 5 of them
folded, no stores, because on this path the callee-saved registers hold
`self` (`%r14`), `pc` (`%r12`), `source` (`%rbp`) and the sret pointer
(`%r13`), and `%rbx` and `%r15` are reloaded at the loop head and overwritten
as scratch. Per-clause values (`index`, `position`, `end`, `sink`, the sret
pointer, the temps mark, `stale`, `clause`): 20, every store among them.
The `nop` path makes **no call**; each of these values is live across the
region walk, whose other arms do call.

`x = y` adds 16: the region's slice iterator lives in memory (`SP-1344`, the current op pointer: 2 stores and 2 reloads for two
ops; `SP-696`, its end: 1 store, 2 compares), the register frame's base
pointer is reloaded per register access (`SP-1360`, 2), `read_slot`'s value
tag is stored, compared and reloaded (`SP-1296`, 3), the symbol for its cold
`NOVALUE` path is stored (`SP-1032`, 1), and an address past the clause is
stored and not read on this path (`SP-1024`, 1). On `rexxcps` the same driver
slots lead (`SP-696` 5.33, `SP-1344` 5.15, `index` 3.38, the register base
3.27 per clause).

### Hot return types (`sizes.txt`, from `sizes.rs.txt` appended to `drive.rs` and removed again)

| type | bytes | returned in |
|---|---:|---|
| `Result<Flow, Failure>` (`run_ops_from`, `run_ops`, `settle` via `Settled`) | 24 | memory (sret) |
| `Flow`, `Ended`, `RegionEnd`, `Settled`, `Result<BranchEnd, Failure>` | 24 | memory |
| `Result<ClauseOutcome<RegionEnd>, Failure>` (`leave_stepped_clause`) | 24 | memory, inlined |
| `Result<ObjRef, Failure>`, `Result<(), Failure>`, `Result<bool, Failure>` | 16 | `rax:rdx` |
| `Failure` | 16 | |
| `SteppedClause` | 16 | |
| `Option<ClausePosition>` | 12 | |
| `Option<LoopHeaderValues>` | 176 | stack object |

`Flow` is 24 because `Option<ObjRef>` is 16 (no niche) and `Leave`/`Iterate`
carry an `Option<SymbolId>` beside a `Box`. No 24-byte value is built on the
`nop` or `x = y` path (their exit is `finish_plain_clause`); the one
per-clause cost of the sret return is the pointer's register (`SP-1320`).
Values reloaded from `self` after calls do not appear on either micro-axis:
`self` stays in `%r14`, and each field read through it is one load either
way.

## 2. Candidates, one per commit, measured

Each prediction was written to `predictions.txt` before its measurement. Each
row is callgrind `summary:` minus libc and ld-linux, two interleaved rounds
(`meas.sh`), against the previous kept head, every binary confirmed by `.text`
hash (`hashes.txt`); the remark build of each candidate was checked to have
the measured binary's `.text` hash before its census was taken (`cand.sh`).
Spill counts are per clause, `nop` / `x = y` / `rexxcps`, from `spills.py`;
the full axis tables are `results-<n>.txt`.

| # | change | predicted spills; `nop` / `assign` / `rexxcps` | spills | `nop` | `assign` | `rexxcps` | verdict |
|---|---|---|---|---:|---:|---:|---|
| s1 | `run_ops_from` answers `Result<(), Failure>`, the `Flow` handed back through an `Interp` field, freeing `%r13` from the sret pointer | 27 / 43; -2.1% / -1.0% / -0.1% | 28 / 48 / 96.48 | +2.012% | +2.974% | +1.117% | not kept (`s1-flow-through-self.patch`) |
| s2 | `Chunk::position_at` answers `(bool, ClausePosition)` with a zero position when untabled, so no arm leaves the payload undefined | 26-27 / 42-43; -2..-3% / -1..-1.5% / -0.2% | **23 / 39 / 86.34** | **-6.972%** | **-3.458%** | **-0.668%** | kept `c6a2ad7a1` |
| s3 | `std::hint::cold_path()` on the driver's `pc >= stop` exit, aimed at the two reloads hoisted into the loop header for it | 22 / 38; -2.2% / -1% / -0.3% | 23 / 32 / 79.39 | -0.011% | -1.540% | +0.137% | not kept (`s3-cold-range-end.patch`) |
| s4 | the source-text test folded into the position table once per range, so the per-clause shortcut no longer reads `source` (`%rbp`) | 21-23 / 37-39; -2..-4% / -1..-2% / -0.2..-0.4% | 21 / 37 / 84.68 | +1.071% | +0.512% | +0.093% | not kept (`s4-positions-with-text.patch`) |
| s5 | the region walked by `pc` itself, bound checked once per region, replacing the slice iterator whose pointers are the largest `rexxcps` slots | 22-23 / 34-36 / 78-82; ~0 / -1.5..-2.5% / -0.3..-0.7% | 23 / 34 / 82.57 | +3.223% | +0.005% | +1.264% | not kept (`s5-region-walked-by-pc.patch`) |

Rejected candidates were measured as uncommitted working-tree changes and
reversed from their saved patch with `git apply -R`; none reached a commit.
`nm` shows `arith_small_int`, `read_at` and the `Option<LoopHeaderValues>` drop
glue out of line in every measured binary, and on `rexxcps`, `varlookup`,
`nop` and `assign` every per-function self-cost delta above 1,000,000
instructions is in a `run_ops_from` instance (`flips.sh`), so no delta above
is an inlining flip.

**Attribution of s2.** Diffing the `nop` listings (`nop.base.tsv` against
`census.nop.s2.txt`'s rows): gone are `SP-876`'s loop-carried reload and
store, `SP-880`'s store and reload, one reload of `index` (it stays in
`%rdx` through the table load), a register copy of `indent` (loaded straight
into `%r15d`), and the `sink` byte's separate reload (it stays in `%bl` until
`stale` is computed). `nop` and `x = y` each lose exactly 7 per clause. **The `rexxcps` figure is uncontrolled**: it
is a driver edit under the round-1 README's 2.5% floor, and no control
separates its -0.668% from the allocator's; the micro-axes are attributed per
instruction, `rexxcps` is not.

**What the rejections show.** s1 freed a callee-saved register for the whole
function, and `nop` rose by 2 with two more stack-argument reads at the loop
head; `x = y` rose by 6. s3 did what it aimed at on
`x = y` (spills 39 -> 32, -3 per clause) and cost `rexxcps` 24.5 million.
s4 removed the one hot use of `source` and every axis rose. s5 retired the
iterator slots on `x = y` (39 -> 34) and cost 3 per `nop` and 1.26% on
`rexxcps`. The per-clause path is one
function's allocation, and a change that only redistributes registers in it
moves every axis by a few instructions in a direction no census predicted.

## 3. Gates

Committed first, tree frozen until the status file says `finished`
(`gates.sh`, `tally.sh`). The results are recorded in the commit after this
one, from the status file.

## 4. Concerns

1. `rexxcps` -0.668% is uncontrolled (section 2). The kept change removes
   data flow on every clause, so its sign is expected, not measured apart
   from allocation.
2. The spill census over-counts cost: a folded read is a memory operand, not
   an instruction (6 of `nop`'s 29). "Instructions of their own" is the
   better column, and it did not track either.
3. Remaining `nop` stack traffic on the head (`census.nop.s2.txt`): loop-head
   reloads for the cold range exit (`chunk` and a stack argument), `index`
   (store and two reloads), `end` (store and two), the sret pointer's
   register (two), the temps mark (two), `sink`/`stale` (three stores, two
   reads), and `header`'s `None` write and drop test. Each is a value live
   across the region walk; the candidates that moved them without removing
   work all lost.
4. `exec_parse` carries 13.9 spill instructions per `rexxcps` clause, more
   than any function but the driver; it is outside this brief's `nop`/`x = y`
   keep rule and was not attempted.
