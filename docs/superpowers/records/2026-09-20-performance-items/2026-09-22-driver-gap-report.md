# The driver gap: `drive.rs:0` identified, and one candidate landed

Against `c3f125a88`. The measuring binaries, dumps and scripts are in the
session scratchpad under `drivegap/`; the A arm was built from `c3f125a88`
extracted with `git archive` into `drivegap/base`, and its `.text` sha256 is
`81d2f383dcc0270c45ce1644895fc3b838f9577e7cc7e0cfaef8d24a7d680532` -- the same
section hash `2026-09-21-reprofile-report.md` recorded, so **the A arm is a
rebuild of the binary the driver decomposition was taken on**, not a
re-derivation of it.

## Gates

Over `9a2eb522f`, the committed tree, which was not edited between the commit
and the last status line. Every status read unpiped from the run's own file.

| gate | command | rc |
|---|---|---:|
| G1 | `cargo fmt --all --check` | 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, cold `CARGO_TARGET_DIR` | 0 |
| G3 | `cargo build --workspace --all-targets --release`, outside the cap | 0 |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | 0 |
| G5 | `cargo build --workspace --all-targets`, outside the cap | 0 |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 |

Tallies summed from each log's own `test result:` lines, not from a summary:
**release 133 binaries, 2651 passed / 0 failed / 4 ignored**; **debug 133
binaries, 2652 passed / 0 failed / 4 ignored**; `corpus_differential`
**604 of 604 matching** in STRICT mode on both. Those are the brief's expected
figures.

**A first attempt at the release test gate exited 137, and it was my command,
not the tree.** `memcap 8G` was wrapping `cargo test --release`, which still had
the test targets to compile, so the cap OOM-killed **rustc** while building
`rexx-exec` -- `memcap: OOM-killed at the 8G cap (peak 8.0G)` with **zero**
`test result:` lines in the log. The fix is to build `--all-targets` outside the
cap first, as G3 and G5 above do; re-running the same command would only have
reproduced it.

## `drive.rs:0` is the register allocator, not the jump table

**27.77 Ir per clause, 555,387,711 Ir, spread over 246 addresses.** Taken from
the brief's own dump (`cg-instr.out`, sha256 `7c6977fa...`) with a parser whose
self total reconciles to the run's `summary:` line with difference 0, then each
address looked up in an `objdump` of the same binary.

**It is not the `match op` jump table.** The region dispatch's five
instructions -- `movzbl (%rbx),%eax` / `lea` the table / `movslq` the offset /
`add` / `jmp *%rax` -- all carry `drive.rs:664`. The outer dispatch is eleven
instructions across `drive.rs:543` and `:544`, including a `cmp $0x28` range
check on the discriminant; three of those eleven fall to `:0`, and they are the
reload of the stream base, the `add` that forms `&stream[pc]`, and the
loop-invariant `lea` of the table.

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

**37.6% of it is spill and reload traffic against the stack frame.** The rest
is register copies, block-boundary `jmp`s and `lea`s, and loads and stores LLVM
emitted with no source position -- none of it dispatch.
`run_ops_from::<true>` is **15,087 bytes and 2,894
instructions** with a **1,416-byte stack frame** (`sub $0x588,%rsp`) and all six
callee-saved registers pushed; 42.9% of its instructions ever execute on
`rexxcps`. `run_ops_from::<false>` is 14,959 bytes with 15.5% executed.

The four largest named pieces inside it:

* `0x13d15b`/`0x13d160`, 45,002,316 each -- reloading the region iterator from
  `0x70(%rsp)` into `%rbx` after an arm that called out.
* `0x13d1c4`, 38,460,944 -- `add $0x10,%rbx`, the region loop's own advance in
  the arm the six trace ops share.
* `0x13c02a`, `0x13c052`, `0x13c068`, `0x13c06b`, 19,441,281 each -- the outer
  loop's per-op `pc` copy, its reload of the stream base, the `add` that forms
  `&stream[pc]`, and its **loop-invariant** `lea` of the jump-table base,
  recomputed every iteration because no register is free to hold it.
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
with tracing off, 41.35% of all region dispatches. **Re-derived here at
`c3f125a88` rather than inherited**: the region jump table at `0x3e7f4` has 43
entries and 38 distinct targets, and **six of its entries share `0x13d1c4`**,
which is the arm the six trace ops share; that address's execution count in the
base dump is exactly **38,460,944**. Summing every arm's entry count gives
**93,024,008** region dispatches and **19,441,281** outer ones, both
reproducing the re-profile's figures at a different commit. (33 of the outer
table's 41 entries point at one arm whose count is **0**: only eight op kinds
ever appear in the outer stream.)

**They are there because
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
`dispatchclass`, `decloop`, `alloc`, `sayloop` and `startup` emit none.

**So rank 3 is worth about 1.5% on `rexxcps` and zero on every other axis we
have -- which is a real win with a caveat, not a non-win.** `rexxcps` is the
figure this project is compared on. What the finding removes is the belief that
those ops are a property of the interpreter; what it leaves standing is the
`rexxcps` number.

Two things belong beside it so the 1.5% is not read as available. **Reaching it
is blocked on a problem already built, gated green and reverted** -- `3c2e6825a`,
reverted by `4ef6af5bb`: the emission change was worth **-0.543%** and the guard
that keeps it honest cost **+0.501%**, and the guard is paid by programs
containing no `TRACE` at all, `emptyloop` **+1.25%** and `varlookup` **+1.08%**.
And **the unguarded ceiling for the whole shape is -1.706%**, so 1.5% is most of
what exists there rather than a slice of something larger. Both figures are in
`docs/superpowers/records/2026-09-20-performance-items/README.md` and
`2026-09-20-performance-todo.md`; I read them there and did not re-run them.

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
* **Finding 1's causal half is static evidence.** The 38,460,944 is measured
  at this commit, per address. What is *not* measured is the counterfactual: I
  compared two compiled streams, and did not run the program with the two
  `TRACE VALUE` clauses deleted to see the instruction count fall. Deleting them
  changes the clause count, so that run would need its own reading anyway.
* **I did not attempt the region walk itself**, which is lead 1 and the largest
  single line in the interpreter. The decomposition says its five-instruction
  dispatch and three-instruction latch are already minimal for a jump table in
  safe Rust; what is left there is *fewer ops*, and finding 1 says the largest
  group of removable ones is specific to the benchmark.
* **The driver's size is a standing cost, and this finding does not say which
  of two changes to make.** 15,087 bytes in one function, 57% of it never
  executed on `rexxcps`, a 1,416-byte frame, and 208,891,609 Ir -- 4.1% of the
  driver's self cost -- in spill and reload traffic alone. **A frame inflated by
  cold arms' locals and a frame inflated by state live across the hot arm point
  at opposite fixes and look identical in this profile.** Reading *which values*
  are spilled is what separates them, and nobody has done that.
  **One form of the first fix is already measured and rejected**:
  `docs/superpowers/records/2026-09-17-crexx-comparison/2026-09-17-crexx-derived-candidates.md`,
  "Candidate E, settled 2026-09-19", moved the cold arms into **one**
  `#[inline(never)]` non-generic helper and got `+2.80%`, `+3.70%` and `+2.53%`
  retired instructions on three axes, 64-way I1 unmoved at +0.16%, `.text` up
  7,328 bytes, the extracted arms costing 9,965 standalone against roughly 1,900
  inlined per instantiation. I did not re-run it. **That does not settle the
  per-arm variant** -- `#[cold]` plus `#[inline(never)]` on each arm separately,
  each with its own small frame -- because the bundled helper's frame is the
  union of every cold arm's needs and each pays for all the others. That variant
  is untested.
  **The direct instrument is the frame, not a percentage**: `sub $N, %rsp` in
  the prologue, 0x588 here. If it does not shrink, the change did not do the
  thing whatever the benchmark says; if it shrinks while instructions rise, that
  is Candidate E's price reappearing.

## Items this opened, none of them measured by me

All three were written by the controller while I held the tree, and I did not
re-run anything in them. They are gitignored, so the paths are workspace paths
rather than committed ones.

* `.superpowers/sdd/queued/2026-09-22-tracefunction-ungated.md` -- finding 2 as
  its own item, carrying the sizing problem: it cannot be measured on the
  current benchmark set, so whoever takes it brings a program or sizes it
  statically and says so.
* `.superpowers/sdd/queued/2026-09-22-driver-frame-pressure.md` -- the
  bundled-versus-per-arm distinction the concern bullet above makes, so the two
  records agree rather than drift.
* `.superpowers/sdd/queued/2026-09-22-clause-shape-distribution.md` -- `rexxcps`'
  timed body measured as 80 clauses in 34 distinct shapes, mean 2.81 ops per
  region with trace ops excluded, ten shapes covering 65%.

## Where the gate evidence is

`drivegap/gates.status` and `drivegap/gates2.status` in the session scratchpad,
each with the commit sha as its first line and `finished` as its last, and the
per-gate logs beside them.
