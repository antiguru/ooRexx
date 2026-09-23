# Stack spills in the IR driver

Branch `perf/spills` from `9f2bc14d7` (`plan/rust-rewrite`). Brief:
`round5/spills.md` in the session scratchpad. Scripts and listings:
`spills-files/`.

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
a `mov` into a `Spill` slot is a **store**, a `mov` out of one or a `cmp`/
`test`/ALU read of one is a **reload**, any access to a `Variable` slot is a
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
| spill reloads | 20 | 30 | 56.20 (+0.42 read-modify-write) |
| **spill total** | **29** | **45** | **91.53** |
| stack-object accesses | 2 | 2 | 59.95 |
| stack-argument reads | 1 | 1 | 4.03 |
| callee-saved `push`/`pop` | 0 | 0 | 84.47 |

`nop` and `x = y` run entirely inside `run_ops_from::<false>`. On `rexxcps`
that function carries 47.04 of the 91.53 (13.89 stores, 33.15 reloads);
`exec_parse` 13.89, `collect_now` 5.23, `apply_binary` 3.02, and
`run_ops_from::<true>` 2.16 are next (`census.rexxcps.txt`).

### What is spilled: every slot on the `nop` path

All in `run_ops_from::<false>`; offsets are the remark's. This covers 100%
of `nop`'s spill traffic.

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
table, positions, `chunk.trace()`, stream length): 9 reloads, no stores,
because on this path the callee-saved registers hold `self` (`%r14`), `pc`
(`%r12`), `source` (`%rbp`) and the sret pointer (`%r13`), and `%rbx` and
`%r15` are reloaded at the loop head and overwritten as scratch. Per-clause values (`index`, `position`, `end`,
`sink`, the sret pointer, the temps mark, `stale`, `clause`): 20, every store
among them. The `nop` path makes **no call**; all of this is the allocator's
function-wide choice, because each of these values is live across the region
walk, whose other arms do call.

`x = y` adds 16 (`census.asg.txt`): the region's slice iterator lives in
memory (`SP-1344`, the current op pointer: 2 stores and 2 reloads for two
ops; `SP-696`, its end: 1 store, 2 compares), the register frame's base
pointer is reloaded per register access (`SP-1360`, 2), `read_slot`'s value
tag is stored, compared and reloaded (`SP-1296`, 3), the symbol for its cold
`NOVALUE` path is stored (`SP-1032`, 1), and an address past the clause is
stored and not read on this path (`SP-1024`, 1). On `rexxcps` the same driver
slots lead (`SP-696` 5.33, `SP-1344` 5.15, `index` 3.38, the register base
3.27 per clause).

### Hot return types (`sizes.txt`, a throwaway `size_of` test, not committed)

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
