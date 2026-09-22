# The driver gap: `drive.rs:0` identified, and one candidate landed

Against `c3f125a88`. The measuring binaries, dumps and scripts are in the
session scratchpad under `drivegap/`; the A arm was built from `c3f125a88`
extracted with `git archive` into `drivegap/base`, and its `.text` sha256 is
`81d2f383dcc0270c45ce1644895fc3b838f9577e7cc7e0cfaef8d24a7d680532` -- the same
section hash `2026-09-21-reprofile-report.md` recorded, so **the A arm is a
rebuild of the binary the driver decomposition was taken on**, not a
re-derivation of it.

## Gates

The five gates were started after the commit, in the background, each status
written unpiped as it lands. They had not finished when this was written, so no
gate result is stated here: read them from the status file named at the end of
this document, whose first line is the commit they ran over.

What *was* run before the commit, and observed: `cargo fmt --all --check`
exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0 (warm
target, so provisional -- the gate run repeats it); and
`REXX_CORPUS_GATE=1 memcap 8G cargo test -p rexx-exec --no-fail-fast`, which
printed `mode: STRICT (the gate)` and `604 of 604 matching` for
`corpus_differential` and `test result: ok` for every binary once
`corpus/refusal-sites.tsv` had been re-derived.

## `drive.rs:0` is the register allocator, not the jump table

**27.77 Ir per clause, 555,387,711 Ir, spread over 246 addresses.** Taken from
the brief's own dump (`cg-instr.out`, sha256 `7c6977fa...`) with a parser whose
self total reconciles to the run's `summary:` line with difference 0, then each
address looked up in an `objdump` of the same binary.

**It is not the `match op` jump table.** Both dispatches are attributed to
source lines: the region dispatch's five instructions -- `movzbl (%rbx),%eax` /
`lea` the table / `movslq` the offset / `add` / `jmp *%rax` -- are all
`drive.rs:664`, and the outer one is `drive.rs:544` with two of its five
falling to `:0`.

By what the instruction is:

| | Ir | share |
|---|---:|---:|
| `mov`, reload from the stack frame | 162,569,541 | 29.3% |
| `mov`, register to register | 87,444,975 | 15.7% |
| `mov`, other | 79,222,985 | 14.3% |
| `add`/`inc` | 63,502,225 | 11.4% |
| `mov`, spill to the stack frame | 46,322,068 | 8.3% |
| `jmp` | 32,621,992 | 5.9% |
| compare/branch/other | 31,660,716 | 5.7% |
| `lea` | 26,441,344 | 4.8% |
| zeroing and constants | 25,601,865 | 4.6% |

**37.6% of it is spill and reload traffic, and most of the rest is block
glue** -- the `jmp`s, `lea`s and register copies LLVM emits at block boundaries
with no source position. `run_ops_from::<true>` is **15,087 bytes and 2,894
instructions** with a **1,416-byte stack frame** (`sub $0x588,%rsp`) and all six
callee-saved registers pushed; 42.9% of its instructions ever execute on
`rexxcps`. `run_ops_from::<false>` is 14,959 bytes with 15.5% executed.

The four largest named pieces inside it:

* `0x13d15b`/`0x13d160`, 45,002,316 each -- reloading the region iterator from
  `0x70(%rsp)` into `%rbx` after an arm that called out.
* `0x13d1c4`, 38,460,944 -- `add $0x10,%rbx`, the region loop's own advance in
  the arm the six trace ops share.
* `0x13c02a`, `0x13c052`, `0x13c068`, `0x13c06b`, 19,441,281 each -- the outer
  loop's per-op `pc` copy, its reload of the stream base, and its
  **loop-invariant** `lea` of the jump-table base, recomputed every iteration
  because no register is free to hold it.
* `0x13c07f`, `0x13c091`, `0x13c094`, `0x13c961`, 16,321,024 each -- spills
  around the `Op::Clause` arm's instruction lookup.

**So the budget for this function is complete, and the answer is that 11.0% of
the driver's self cost is the price of its size.** For scale: `drive.rs:664`'s
five instructions come to 504,320,040 Ir, so each one of them is about 0.50% of
the whole program -- and one of the five is the loop-invariant `lea` that a
free register would hoist. Nothing in the Rust source controls that directly.

## What landed

**Cache the packed TRACE byte instead of rebuilding it per clause.**

`Interp::chunk_trace()` answered `ChunkTrace::of(self.traced_mode())`, and
`drive.rs:581` asks it once per promoted clause. `ChunkTrace::of` reads six
`bool`s out of a `TraceMode` and shifts and ors them into one byte. Read off the
binary, `0x13c5dd`..`0x13c64f` is **18 instructions** executed **16,321,024**
times on `rexxcps` -- five `movzbl`s, five shifts, four adds, a spill, an `xor`
and a store. It is now **five instructions and no branch**: LLVM turns the
`debug_pause` test into a `cmovne`.

`TraceCache` holds the mode beside the byte. `TraceCache::of` is the only way to
build one and it derives the byte, so they cannot drift; `chunk_trace()`
asserts the cached byte against a recomputed one in debug. **The assertion was
inverted to prove it live**: storing `ChunkTrace::OFF` in the cache reddens
`ir_recorded` with `the cached ChunkTrace is not the one the setting in force
packs down to` at `trace.rs:642`.

`valgrind --tool=callgrind`, arms interleaved, own `CARGO_TARGET_DIR` per
build, every run rc 0, each from a fresh empty directory:

| axis | base | candidate | change |
|---|---:|---:|---:|
| `bench-programs/emptyloop.rex` | 9,923,576,984 | 9,748,588,475 | **-1.7634%** |
| `bench-programs/varlookup.rex` | 17,432,593,394 | 17,185,604,502 | **-1.4168%** |
| `bench-rexxcps/rexxcps.rex` | 20,294,025,709 | 20,187,037,327 | **-0.5272%** |

Two rounds per axis, A B A B. Within-arm spread: `emptyloop` 23,911 (0.00024%)
and 4,608 (0.00005%), `varlookup` 26,428 (0.00015%) and 13,286 (0.00008%),
`rexxcps` 3,209,815 (0.0158%) and 8,617,854 (0.0427%) -- wider than this axis's
usual 0.0066% because builds ran beside the measurement and `rexxcps` renders
`TIME()` into its own output. Per round: `emptyloop` -1.7632% and -1.7635%,
`varlookup` -1.4169% and -1.4167%, `rexxcps` -0.4981% and -0.5563%.

The driver's own self cost falls from **252.18 to 246.13 Ir per clause**;
`trace.rs` inlined into it from 11.78 to 6.70 and `activation.rs` from 7.85 to
2.62, while `run.rs` inlined into it rises from 18.75 to 22.26 as the line
attribution moves. **The whole-program figure is the one to read**: on the round
both dumps were taken, the driver saved 120,926,855 and the program
101,074,547, so about 20M came back outside the driver.

**No conditional was added to the per-clause path**, which is the constraint
that reverted an earlier commit; one was removed.

## Three findings that change the standing picture

### 1. The 38,460,944 trace ops are `rexxcps`'s own, not the interpreter's

Lead 3 and the re-profile's rank 3 rest on 38,460,944 value-echo ops executing
with tracing off, 41.35% of all region dispatches. **They are there because
`rexxcps.rex` contains two `TRACE VALUE` clauses** -- `trace value tracevar`
at line 38 and `trace value trace()` at line 71 -- and a `TRACE` whose value the
source does not fix is `TraceEvent::Unknown`, which the forward analysis carries
around the timed loop's back edge to every instruction in it.

Measured with `rexx-ir`, which prints the compiled stream, counting op names
with `grep -c` over its output:

    rexx-ir rexxcps.rex n              721 ops, 207 trace ops
    rexx-ir <both clauses deleted> n   518 ops,   8 trace ops

**199 of `rexxcps`'s 721 compiled ops, 27.6%, are trace ops that exist only
because of those two clauses** (the other four of the 203-op difference are the
deleted clauses' own). Deleting either clause alone changes nothing -- the
other still poisons the loop, which is why my first two probes came back
negative.

Run over every program in `bench-programs/` --
`for f in bench-programs/*.rex; do rexx-ir $f n | grep -c Trace; done` against
the same loop counting `TraceFunction` -- the two counts are equal for every
one of them: **`Op::TraceFunction` is the only trace op any of them emits at
all**, and `emptyloop`, `varlookup`, `arith`, `compound`, `dispatch`,
`dispatchclass`, `decloop`, `alloc`, `sayloop` and `startup` emit none. So rank
3's ~1.5% is a `rexxcps` figure with no counterpart on the other axes, and the
item should be re-ranked or closed rather than re-attempted.

### 2. `Op::TraceFunction`'s emission is not gated, and its five siblings' is

Every other value-echo op is pushed under `if echoes_values`. Both
`ops.push(Op::TraceFunction { .. })` sites in `compile.rs` are unconditional.
The consequence, reproducible in three lines:

    $ cat fn.rex
    e=0
    e=length("ab")+e
    say e
    $ rexx-ir fn.rex n
    ...
    6: CallArgs slot=0 path=root.L site=0 argc=1 dst=0
    7: TraceFunction index=0 slot=0 path=root.L src=0

Under `TRACE N` with nothing in the body that can change the setting, that op
can never echo; its arm is `if !self.tracing_intermediates() { continue; }`.
**Every call expression in every program carries one**, at the 8 instructions
the re-profile measured for one region op (five of dispatch, three of advance).
DERIVED, not measured: `bench-programs/strings.rex` carries four, all inside its
`do i = 1 to 3000000` loop, so about 12,000,000 dead dispatches on that axis.

I did not fix it. The change is one `if` at each site, but the six golden cases
that pin `TraceFunction`'s `path=root`/`root.L`/`root.R` address compile under
the default setting and would have to move to a tracing one, and the win is
zero on `rexxcps` -- `echoes_values` is true throughout its loop for the reason
above -- so it does not belong in this task's budget. It is a real defect with
a three-line reproduction and it should be its own item.

### 3. The per-clause path after this commit is 74 instructions

Every instruction in the driver whose execution count is the `Op::Clause` count
(16,321,024), by the file it is attributed to:

| file | instructions per clause |
|---|---:|
| `run.rs` inlined (`enter_stepped_clause`) | 19 |
| `core/src/slice/index.rs` | 11 |
| `drive.rs` | 10 |
| `trace.rs` (this commit's five, plus two) | 7 |
| `clause.rs` (the deadline counter) | 7 |
| `core/src/option.rs` | 7 |
| `rexx-core/src/.../mod.rs` | 5 |
| `activation.rs` | 3 |
| other | 5 |

Two things in it are visibly redundant and are the next places to look:

* **The same bounds check runs twice per clause** -- `cmp %rax,0x360(%rsp)` at
  `0x13c81c` and again at `0x13c874`, same index, same length. The second is
  `enter_stepped_clause` re-deriving the `Option<Position>` discriminant that
  `chunk.position_at(index)` already computed, rather than using the value in
  hand. Four instructions.
* **The region entry costs about eight instructions on every clause**, not only
  an empty one: `chunk.ops_in(pc + 1, end)` range-checks `pc + 1 <= end` and
  `end <= ops.len()`, then the loop-entry test compares the two ends again. The
  re-profile sized this at 0.11% by counting only the 13.7% of clauses with an
  empty region; charged to every clause it is about 0.64% on `rexxcps` and more
  on `emptyloop`. Cutting it needs `end` linked to a slice length the way
  `26ccef4ee` linked `stop`, and that is the technique the brief records at
  -0.86%, 0.00% and +2.02% on three sites, so it is a candidate and not a plan.

## Concerns

* **`rexxcps`'s spread this session was 0.0158% and 0.0427% within an arm**,
  against the 0.0066% the re-profile recorded, because I ran builds beside the
  measurement. The -0.5272% is six to twelve times that spread and the two
  rounds agree to 0.06 percentage points, so the sign and the magnitude hold;
  but a future sub-0.1% question on this axis needs a quiet machine.
* **Finding 1 is static evidence.** I counted ops in the compiled stream, not
  executions: the 38,460,944 figure is the re-profile's, taken at `3b850d885`,
  and I did not re-take it. What I measured is that deleting two clauses from
  `rexxcps.rex` removes 199 of its 207 trace ops.
* **I did not attempt the region walk itself**, which is lead 1 and the largest
  single line in the interpreter. The decomposition says its five-instruction
  dispatch and three-instruction latch are already minimal for a jump table in
  safe Rust; what is left there is *fewer ops*, and finding 1 says the largest
  group of removable ones is specific to the benchmark.
* **The driver's size is a standing cost nobody has priced.** 15,087 bytes in
  one function, 57% of it never executed on `rexxcps`, a 1,416-byte frame, and
  208,891,609 Ir -- 4.1% of the driver's self cost -- in spill and reload
  traffic alone. Outlining the cold arms is
  mechanical and would test it; it is also exactly the kind of change project
  memory records as moving cycles up to 4% per axis on layout alone, so it needs
  instruction counts and a control, and it did not fit this task.

## Gates

Started after the commit, writing each status unpiped as it goes to
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/drivegap/gates.status`,
whose first line is the commit sha and whose last line reads `finished`.
