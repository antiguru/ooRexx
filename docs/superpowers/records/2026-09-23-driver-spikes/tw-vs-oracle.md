# Why the oracle's tree walker needs ~40% of ours: per construct

Branch `investigate/tw-vs-oracle` from `95d8cd7ed` (`spike/tree-walker`). No
interpreter change; one throwaway control build, reverted from a copy (below).
Binary: `cargo build --release -p rexx-exec --bin rexx-run --features
tree-walker`, `.text` sha256 `66b1f5c5...5d45447`, the same hash the
tree-walker record measured. Oracle: `build/lib/librexx.so.4`, `-O2 -g`,
under the standard wrapper. Micro-programs, scripts and derived tables are in
`tw-vs-oracle-files/`.

## Method

Each program is `n = 200000`, a fixed prelude, `do i = 1 to n; <construct>;
end` (`progs/*.rex`, written by `mkprogs.sh`). Cost per construct = (program
- `ctrl.rex`) / 200000; `x = a.i` is against `ctrlfill.rex` (same prelude plus
the fill loop that gives it something to read); the empty loop is `ctrl2`
(n = 400000) - `ctrl`. Callgrind `summary:` minus `libc.so.6` and `ld-linux`
(`sumobj.py`), both sides; the oracle also with libc. Two rounds, the second
in reverse engine order, every run from a fresh empty directory
(`measure.sh`); every run exit 0, stdout identical across the three engines
(`sanity.sh`). Max spread between rounds: oracle 0 instructions, walker
12,035, IR 8,480 over whole runs (under 0.07 per iteration); `table.py`
asserts under 0.1. A third, per-instruction run (`instr.sh`,
`--dump-instr=yes`) reproduced every walker and oracle figure in the table to
the unit.

## Per-construct cost (instructions per execution)

| construct | oracle ex-libc | oracle with libc | walker | IR | walker/oracle |
|---|---:|---:|---:|---:|---:|
| empty loop, per iteration | 460.4 | 471.4 | 276.0 | 279.0 | **0.60** |
| `nop` | 33.3 | 33.3 | 279.0 | 106.0 | **8.39** |
| `x = y` | 99.3 | 99.3 | 644.0 | 242.0 | 6.49 |
| `x = 'abc'` | 88.3 | 88.3 | 635.0 | 202.0 | 7.19 |
| `x = x + 1` | 311.4 | 322.4 | 962.0 | 377.0 | 3.09 |
| `x = a \|\| b` | 347.5 | 392.5 | 1091.1 | 531.0 | 3.14 |
| `if a = b then nop` | 254.3 | 275.4 | 1102.1 | 703.1 | 4.33 |
| `a.i = x` | 1609.9 | 2023.3 | 1171.4 | 985.4 | **0.73** |
| `x = a.i` | 630.8 | 968.7 | 1060.4 | 674.4 | 1.68 |
| `x = length(y)` | 334.3 | 334.3 | 1168.0 | 639.0 | 3.49 |
| `call r` (3 clauses) | 851.8 | 949.8 | 2097.0 | 1826.0 | 2.46 |
| `parse var s a b c` | 994.8 | 1075.8 | 1152.1 | 1042.0 | 1.16 |
| `x = y * 1.5` | 625.5 | 663.5 | 1985.9 | 1274.9 | 3.18 |

The oracle's libc share is zero on the constructs that allocate nothing and
11 to 413 per execution on the ones that do (`__memset` zeroing new objects,
`memcmp`); its libc use is allocation-shaped, ours (excluded above) is
`memcpy`/`memcmp`/malloc.

**The shape: the gap is per clause, not per operation.** A `nop` costs the
walker 279 against 33; every construct carries that 246 surplus, and the
constructs whose own work is large (PARSE, stems, the loop) are near parity or
cheaper here.

## Class totals

`classify.py` puts every retired instruction of each program-minus-control
listing (`tsv/`) into exactly one class, first matching rule wins, the same
rules on both engines; each row sums to the table total. Classes:

* **codegen**: `push`/`pop`/`ret`/`%rsp` adjust, any `%rsp`-relative
  load/store (spills, reloads, and `Result` values moved through memory), and
  the 77-instruction entry block of `walk_step_in_temps_frame`.
* **error propagation**: `Result`/`Option`/`Flow` niche-tag tests,
  `result.rs`, and the `ClauseOutcome<Result<Flow>>` unpacking lines.
* **Rust checks**: `slice/index.rs`, `Option` tests, the `roots.rs` asserts.
* **missing caching**: functions that re-derive per execution what the oracle
  keeps on the node: `slot_of` and its name map, the name lookup feeding it,
  `literal`, `inline_text_to_number`, `small_int_operand`/`spelled_int_arith`,
  `walk_site_resolution`.
* **D19 depth**: `enter_eval_node` and its decrement.
* **trace gating/state**: `trace.rs`, `plan.rs`, the indent/line/echo lines
  of `enter_stepped_clause`, the debug-pause decision; oracle
  `RexxActivation.hpp` trace tests.
* **rooting**: our temps `Vec`; the oracle's `ExpressionStack` and
  `ProtectedObject`.
* **clause bookkeeping** both do; **allocation**; **representation** (handle
  decode, `render`, exposure/alias tests); **work + dispatch** (the rest).

Walker minus oracle, per execution (`surplus.tsv`; per-engine columns in
`classes.tsv`):

| construct | codegen | err prop | Rust checks | missing cache | D19 | trace | rooting | clause bk | alloc | repr | work+disp | **total** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `nop` | 140 | 28 | 10 | 0 | 0 | 37 | 1 | 3 | 0 | 0 | 27 | **246** |
| `x = y` | 192 | 41 | 23 | 132 | 18 | 46 | 1 | 3 | 0 | 10 | 79 | **545** |
| `x = 'abc'` | 187 | 41 | 14 | 166 | 18 | 48 | 1 | 3 | 0 | 5 | 64 | **547** |
| `x = x + 1` | 229 | 57 | 30 | 175 | 52 | 55 | 4 | 3 | -109 | 25 | 130 | **651** |
| `x = a \|\| b` | 277 | 59 | 34 | 132 | 52 | 49 | 4 | 3 | -100 | 60 | 174 | **744** |
| `if a = b` | 266 | 61 | 44 | 66 | 52 | 43 | 6 | 10 | 0 | 97 | 203 | **848** |
| `length(y)` | 273 | 61 | 39 | 224 | 34 | 43 | -68 | 8 | 0 | 17 | 203 | **834** |
| `y * 1.5` | 385 | 91 | 47 | 315 | 52 | 44 | -22 | 3 | 32 | 45 | 368 | **1360** |
| `call r` | 707 | 118 | 80 | 121 | 0 | 137 | -44 | 59 | -33 | 15 | 86 | **1246** |
| empty loop | 20 | 8 | 12 | 0 | 0 | -4 | -37 | -9 | -106 | 17 | -85 | **-184** |

Codegen is the largest class in every row but the empty loop; missing caching
is second in `x = y`, `x = 'abc'`, `x = x + 1` and `length(y)`. Allocation,
rooting and the loop run in our favour.

## The per-clause 246, instruction by instruction

The `nop` listing (`ad.nop.tw.all.txt`), classified by hand rules in
`classify_nop.py` (`classify-nop.txt`):

| per clause | walker | oracle |
|---|---:|---:|
| hoisted loads and spills at `walk_step_in_temps_frame` entry | 77 | 0 |
| prologues/epilogues (frames 0x488 and 0x458 bytes) | 38 | 0 |
| eager trace/debug state (indent, line, echo gate, pause decision) | 59 | 6 |
| call layering: `walk_bounded` -> `walk_step_in_temps_frame` -> `walk_step` -> `exec_instruction` | 37 | 10 |
| `Result`/`ClauseOutcome` unpacking | 32 | 0 |
| bounds and `Option` checks | 12 | 0 |
| bookkeeping both do (deadline/yield counter, clock, trace-entry decay, pending traps, temps frame) | 24 | 17 |
| **total** | **279** | **33** |

The oracle's 33 is `RexxActivation.cpp:612`-`662` (counter, `current`/`next`,
one virtual `execute`, `stack.clear()`, `timeStamp.valid`, `clauseBoundary`,
`traceEntryAllowed`) plus `RexxInstructionNop::execute`'s two trace-bit tests
(`RexxActivation.hpp:375`, `:380`).

## The three largest individual causes

**1. The per-clause entry function's codegen.** `walk_step_in_temps_frame`
(`run/tree_walker.rs:112`) inlines `walk_step` (`:168`-`:263`) with its `IF`,
`SELECT` and `DO` arms, and opens every clause, a `nop` included, with a
0x488-byte frame and 77 instructions that load fields of `instruction`
(`%r8`) and `code` (`%rdx`) and store 38 of them to stack slots
(`0x13728f`-`0x137469`); 30 of those slots are never read back on the `nop`
path (the other 8 are); `exec_instruction`
(`run.rs:985`) then opens a 0x458-byte frame to answer `Ok(Flow::Next)`. The
oracle's equivalent is a loop body and one virtual call
(`RexxActivation.cpp:639`-`642`). **Control, run:** `walk_step` marked
`#[inline(never)]` (`control-noinline.patch`, `.text` `6631213f...`) and
nothing else: `nop` 279.0 -> 234.0, `x = y` 644.0 -> 599.0, `x = x + 1` 962.0
-> 917.0, `if a = b` 1102.0 -> 1064.0, `call r` (three clauses) 2097.0 ->
1962.0, empty loop 276.0 -> 274.0; both rounds agree to 0.1
(`results-control.tsv`). So at least 45 per clause is layout, not work. Why
LLVM hoists there is inferred, not measured.

**2. Assignment targets resolved by name.** `exec_instruction` passes `at:
None` (`run.rs:1007`), so `assign_expr_target` calls `slot_of(name)`
(`run.rs:2737` -> `plan.rs:738`-`750`), an Fx hash of the name, a SwissTable
probe and a `memcmp`: 127 per assignment for the calls, 132 with the name
lookup feeding it, on every simple assignment. Reads of the same variable go
through `Code::slot_for(id)` (`lib.rs:1378`, used at `lib.rs:5373`), an
indexed load. The oracle's `RexxSimpleVariable::assign`
(`ExpressionVariable.cpp:296`-`300`) indexes its local-variable array by the
`index` held on the node: 24 instructions, all in. The IR's store carries the
slot: the IR's `x = y` (242) calls no `slot_of`. A tree-path gap, not a
property of walking a tree.

**3. Eager per-clause trace and debug state.** Every clause computes and
stores its indent (`run.rs:4934`, through `printed_indent`, `:5455`-`:5463`,
since the walker passes no plan position) and line (`run.rs:4960`, through
`clause_line_at`, `:8317`-`:8337`), `instructions_traced_at_entry` (`:4949`),
runs the echo gate (`run.rs:4991` -> `:5073`, `trace.rs:268`-`274`,
`:629`-`:640`) and decides the debug pause (`tree_walker.rs:126`-`142`): 37 to
59 per clause depending on the rule set, against the oracle's 6, whose line is
a field on the instruction and whose trace tests are bit tests inside each
`execute`. Close behind: `Result` threading, 28 per clause and up to 118 per
construct in tag tests alone, not counting the `Result` moves the codegen class
absorbs.

Smaller named items: `literal` re-derives a literal's value every evaluation,
34 to 49 (`value.rs:102`) against `RexxString::evaluate`'s 10
(`StringClass.cpp:2075`); `inline_text_to_number` answers "is `abc` a number"
from a side table at 33 a call (`value.rs:753`) where `RexxString::comp` reads
a cached flag at 6 (`StringClass.cpp:548`-`560`, `:764`-`:765`); D19's depth
bookkeeping, 16 to 17 a node (`eval.rs:64`-`92`); no per-node counterpart
appears in the oracle's listings, and nothing under its `expression/` calls
`checkStackSpace`.

## What does NOT explain it

* **Bounds checks**: 10 to 80 per construct including `Option` tests, 10 of
  the `nop`'s 279. Consistent with the 0.99% record.
* **Allocation**: the walker allocates nothing for `nop`, `x = y`, `x = x +
  1`, `if a = b`; the oracle allocates a `RexxInteger` for each `+` result
  and each loop step here (109 per execution, `MemoryObject::newObject` under
  `RexxInteger::operator new`). In our favour.
* **Rooting**: 3 per clause here (`push_frame` only reads a length,
  `roots.rs:142`), 0 to 20 per construct, against the oracle's 2 to 79.
* **The arithmetic**: `small_int_arith` + `arith_small_int` are 59 against
  `RexxInteger::plus` + `callOperatorMethod`'s 59 plus 120 of allocation.
* **The loop**: the walker's empty loop is 0.60x the oracle's; PARSE is
  1.16x, stem store 0.73x.
* **Bookkeeping both engines do**: 6 to 25 per single-clause construct
  against 15.
* Prior art, `oorexx-oracle-is-a-tree-walker` (2026-08-08), checked against
  today's code: (1) line by binary search: **gone**, no `line_of` in any
  listing, the line is a table load now; (2) a `RootSet` frame per clause and
  per eval: **mostly gone**, 3 per clause; (3) `Result` through five layers:
  **holds**, see the error-propagation column and the call-layering row; (4)
  `activations.last().expect()`: **changed**, now `running: Option<Box<_>>`
  (`activation.rs:986`), 1 to 5 tests of about 3 instructions per construct.
* The 2026-09-22 category rollup was not used.

## Concerns

1. The classes are first-match rules: "codegen" absorbs every
   `%rsp`-relative move, including `Result` values passed through memory, so
   error propagation is understated there and codegen overstated. The two
   `nop` breakdowns differ for the same reason (trace 42 generic, 59 by hand
   rules); totals are exact in both.
2. The controlled result is the 45-per-clause drop, not the 140 codegen
   figure; how much of the rest of the codegen class a restructuring could
   remove is not measured.
3. The oracle is an `-O2` shared library with PLT calls; we are `-O3` fat
   LTO. Both favour us.
4. Construct shapes matter: `a.i = x` inserts new tails, `x = a.i` reads a
   filled stem, `if a = b` compares two non-numeric strings and is false.
5. Only `nop` and `x = y` were read at every instruction on both sides;
   `x = x + 1`, `if a = b`, the loop, `mul`, `length` and `call` rest on the
   per-function diffs (`fd/`) and the generic classifier.
6. Inferred, not measured: that the per-clause 246 accounts for about a third
   of `rexxcps`'s 740-per-clause gap (with libc, 1274.02 - 534.44); the
   `rexxcps` clause mix was not re-measured.
