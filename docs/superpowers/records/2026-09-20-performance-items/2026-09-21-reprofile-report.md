# Re-profile and regenerated candidate queue, 2026-09-21

At `3b850d885` ("Record item 7, and retire the method that sized it"), on top of
`26ccef4ee`. Read-only: nothing in the repository was edited and nothing was
committed. This file is written in the measuring worktree,
`/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-aad6f8a8191b14361`,
and a copy is at
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/reprofile-2026-09-21/`.

**This queue replaces `.superpowers/sdd/queued/2026-09-20-performance-todo.md`.**
Every figure below was taken at this HEAD. Evidence classes are the to-do's own:
MEASURED (an A/B was run), DERIVED (computed from a measured constant),
ATTRIBUTION (what the machine currently spends, which is not what a fix
recovers), UNKNOWN.

---

## The ranked queue

### 1. Memoise the compound-variable tail key

**What stops happening.** `Interp::append_tail_key` re-renders a compound
variable's tail from its pieces on *every* read and *every* write: for each
`TailPiece::Variable` it reads the variable, forces a string value, renders it
into a byte buffer, and the buffer is then hashed to index the stem's tail map.
`acompound.key1.loop` inside `rexxcps`' `j` loop rebuilds `Key Bee.1` from
scratch four times per iteration while neither `key1` nor `loop` has changed.

| | Ir | share | calls | Ir/call |
|---|---|---|---|---|
| `append_tail_key` inclusive | **1,030,200,354** | **5.08%** | 3,880,000 | 265.5 |
| — from `read_symbol` | 670,680,000 | 3.31% | 2,500,000 | 268.3 |
| — from `assign_expr_target` | 359,520,354 | 1.77% | 1,380,000 | 260.5 |
| `read_symbol` inclusive (whole compound read) | 1,246,055,945 | 6.14% | 2,500,000 | 498.4 |
| `stem_get_at` inclusive (the map lookup) | 365,375,945 | 1.80% | 2,500,000 | 146.2 |
| `stem_set_at` inclusive | 362,188,450 | 1.78% | 1,380,000 | 262.5 |

**Evidence class: ATTRIBUTION, and it is a ceiling, not a size.** 5.08% is what
tail-key construction costs today; a memo recovers it only on a hit and the hit
rate has not been measured. Treat it as the largest function worth opening, in
the brief's own words, and not as a recoverable figure.

**What would falsify it.** (a) An invalidation that costs per access what the
render costs — the exact shape that killed item 2, whose guard cost +0.501%
against a +0.543% win. (b) A low hit rate: the ceiling assumes tails repeat, and
nothing here measures how often they do. Measure the hit rate against a counter
before designing the memo, not after.

**Tests that catch a mistake.** `corpus_differential` in STRICT mode; the stem
tests in `rexx-exec`; and specifically the tail **insertion ordinal** — the
`usize` in `Body::Stem`'s `tails: NameMap<Vec<u8>, (usize, Option<ObjRef>)>`,
which exists because `allIndexes`/`supplier` answer in the oracle's post-order
tree walk. A memo that bypasses the map must not bypass the ordinal.

**Not a candidate, checked and dead:** replacing the hasher. `NameMap` already
uses `rustc_hash::FxBuildHasher` (`rexx-core/src/lib.rs:43`), so the 430,866,916
Ir (2.12%) in the tail map's `HashMap` arcs is not SipHash waiting to be removed.

### 2. PARSE candidate 1 — bind the template once

**Re-derived at this HEAD, and it reproduces the `parse-in-place` report's four
components to the digit.**

    iterator walks (core/src/slice/iter/macros.rs in exec_parse)   111,720,075
    per-execution matches (parse_template.rs :609 :882 :900)        67,200,045
    eval_node + enter_eval_node arcs out of exec_parse             140,000,104
    exec_parse prologue + epilogue (:519 :550)                      38,080,034
                                                                   -----------
                                                                   357,000,258   = 1.759%

**Evidence class: DERIVED.** `26ccef4ee` did not move it — it touched the
driver's op slice, not `parse_template.rs`.

**What would falsify it.** The `parse-in-place` report's own finding that the
match-line component is **build-dependent**: that report's predecessor attributed
87,360,062 over eight lines where this build attributes 67,200,045 over three.
The other three components have now reproduced across three builds.

**Overlap.** None with rank 1. It does overlap the rest of `exec_parse`: the
function's inclusive cost is 2,662,652,625 (13.12%), so candidate 1 is 13.4% of
it. Its predecessors already took their share — `exec_parse` inclusive was
2,803,777,215 (13.71%) at `bb81ae522` and is 2,662,652,625 now.

**Tests.** `corpus_differential` STRICT (604 of 604 on each of today's PARSE
commits), the PARSE golden tests, and the oracle trace differential over
`TRACE I`/`TRACE R` that the PARSE tasks used.

### 3. Item 2 again, and only with a guard that is not per clause

**A new measurement, which is the reason this is still on the list.** The
region-walk arm that all six trace ops share is **three instructions**, and the
region dispatch preamble is **five**. Both were read off the binary and their
execution counts off the dump:

    0x13d1b1..0x13d1c2  movzbl/lea/movslq/add/jmp   93,024,008 each   (every region op)
    0x13d1c4..0x13d1d0  add/cmp/jne                 38,460,944 each   (the trace arm)

So one surviving trace op costs exactly **8 instructions**, and the 38,460,944 of
them in `rexxcps` are **307,687,552 Ir, 1.516%**. MEASURED, per-address, not an
A/B.

This sits inside the already-measured bracket: an unguarded speculation of every
clause was measured at **-1.706%** and the per-clause guard at **+0.501%**. The
item is worth ~1.5% and nothing has changed about why it was reverted.

**What would falsify it.** Any guard that is paid per clause. Five shapes were
tried and none was cheaper; project memory records that *any* conditional in the
driver's `Op::Clause` arm costs ~0.5% on `rexxcps` whatever it does.

### 4. Compare a small integer against a `Number` without materialising one

`apply_binary` calls `Number::from_i64` **3,360,000** times for
**169,320,000 Ir (0.834%)**, 50.4 Ir per call, and `compare_numbers` exactly
3,360,000 times.

**Evidence class: DERIVED, ceiling 0.834%.** A specialised comparison still has
to do work, so the realised figure is lower.

**A larger version of this candidate is dead, and the derivation is the finding.**
"Give a small-int/small-int comparison a fast path" would be worth
169,320,000 + 456,480,000 = 625,800,000 (3.08%) if such comparisons existed here.
They do not: `from_i64` fires **once** per `compare_numbers` call, not twice, so
on `rexxcps` **at most one operand of a numeric comparison is a small integer**
and a both-sides fast path is worth zero. Do not put the 3.08% in a brief.

### 5. The numeric cluster, re-derived

| function | self Ir | share | Ir per dispatched op |
|---|---|---|---|
| `rexx_num::compare::numeric_order` | 485,600,000 | 2.393% | 4.00 |
| `Number::add_signed` | 468,250,849 | 2.308% | 3.86 |
| `Number::mul` | 315,600,000 | 1.555% | 2.60 |
| `Number::from_i64` | 169,320,000 | 0.834% | 1.39 |
| `Number::parse_bytes` | 155,383,188 | 0.766% | 1.28 |
| **cluster** | **1,594,154,037** | **7.86%** | **13.13** |

Against the to-do's 9.93 Ir/op for its three-function version of this cluster;
the same three are 10.46 Ir/op now. `numeric_order` costs **115.6 Ir per call**
over 4,200,000 calls. ATTRIBUTION; no A/B, and no candidate inside it has been
sized except rank 4.

### 6. Allocation and collection, re-derived

| function | self Ir | share | inclusive | calls |
|---|---|---|---|---|
| `collect_now` | 359,515,098 | 1.772% | 817,710,367 (4.03%) | 94 |
| `alloc_with` | 314,298,363 | 1.549% | 1,133,281,406 (5.58%) | 6,140,867 |
| `free` | 193,917,189 | 0.956% | 433,486,946 (2.14%) | 5,772,216 |
| `_int_malloc` | 183,231,575 | 0.903% | 191,115,017 (0.94%) | 1,542,502 |
| `drop_glue::<rexx_core::body::Body>` | 179,527,544 | 0.885% | 450,242,953 (2.22%) | 6,140,909 |
| **cluster self** | **1,230,489,769** | **6.06%** | | |

Against the to-do's 6.79 Ir/op for its three-function version; those three are
7.15 Ir/op now. **184.5 Ir per allocation** inclusive, 6,140,867 allocations.
ATTRIBUTION. `collect_now` is 94 collections for 817,710,367 Ir, which is a
different question from the 6.1M allocations and should not be merged with it.

### 7. The text/number conversion cluster, re-derived — and it shrank

| function | self Ir | share | Ir per dispatched op | calls |
|---|---|---|---|---|
| `inline_text_to_number` | 291,860,319 | 1.438% | 2.40 | 5,580,002 |
| `heap_to_number` | 270,580,205 | 1.333% | 2.23 | 3,340,405 |
| `to_text` | 165,261,390 | 0.814% | 1.36 | 3,361,112 |
| **cluster** | **727,701,914** | **3.59%** | **5.99** | |

Against the to-do's **7.09 Ir/op**. This cluster is the one that moved: it is
now 5.99 Ir/op, down 15%, and the 5,580,002 conversions
`bench-programs/README.md` records still reproduce exactly as
`inline_text_to_number`'s call count. The render side belongs beside it —
`value::write_small_int`, 182,380,094 (0.899%, 1.50 Ir/op) over 5,880,002 calls,
which the to-do's cluster did not list and which is partly rank 1's (2,800,000
of those calls are `append_tails` rendering an integer tail piece).

ATTRIBUTION, no A/B, no sized candidate inside it.

---

## Entries that cannot be sized without building them

These carry no number on purpose.

* **Item 9, per-send method cache.** It has no `rexxcps` figure at all: `Message`
  executes **118** times in the whole run and `MethodDict` appears only through
  `add_method`'s 70,190 startup calls. It is a `dispatch`/`dispatchclass` item
  and must be ranked on those axes, which this profile does not cover.
* **Item 10, ops off the AST re-entry path.** On `rexxcps` the re-entry is
  `eval_node`, inclusive 527,328,592 (2.60%) over 1,960,473 calls — and
  1,400,001 of those calls are `exec_parse`'s, already owned by rank 2. What is
  left is `exec_address`' 280,000 (`address value address()`, line 79) and
  `eval`'s 280,144. Under 1% once PARSE's share is removed, and not separable
  from rank 2 without building both.
* **Item 11, placement.** Untouched and still a question rather than a task.
  Callgrind cannot answer it: `Ir` is flat under the padding sweep that moves
  cycles and L1i, so nothing in this profile bears on it.
* **Item 12, hot-set footprint.** Untouched. No cache simulation was run here;
  the standing figures come from a `--cache-sim` run that this session did not
  repeat.
* **`numeric_order` at 115.6 Ir per call** and **`builtin::run` at 594 Ir per
  builtin call** (`LENGTH` 704 Ir over 560,000 calls, `SUBSTR` 472 over 840,000,
  `WORD` 378 over 560,000). Both are large and neither has a candidate inside it
  that anyone has written down, so neither gets a figure.

---

## Which old items are now dead, and why

* **Item 1** — landed (`7a28f68e6`, `7fbadfecb`, `941b5fb82`). Closed.
* **Item 2** — built and reverted. Survives only as rank 3 above, and only under
  the non-per-clause-guard precondition.
* **Item 3** — landed at `725aa8863`, -1.6747%. Closed. Confirmed here: the
  static stream renders 720 ops, `Condition` 0, `JumpUnless` 0,
  `ConditionJump` 19; `ConditionJump` executes 5,900,006 times and `JumpUnless`
  never.
* **Item 4** — dead, unsound. Unchanged.
* **Item 5** — dead as a two-op fusion; alive only as a three-op fusion whose
  value is a function of item 2. **Its precondition is unchanged**: the hot pairs
  are still separated by a live `TraceLiteral`, because 38,460,944 trace ops
  still execute.
* **Item 6 — now DEAD, and this is the first time it has been sized.**
  MEASURED at the branch that decides it (`0x13ca9e cmp`, `0x13caa0 jne`):
  **2,240,007** of the **16,321,024** executed `Clause` ops have an empty region,
  13.7%. Statically, 25 of 129 `Clause` ops are empty-region and **20 of the 25
  are in the timed body** (lines 47-73, all `then` or `select`). What an
  empty-region clause could stop paying is the region entry — the `ops_in` bounds
  pair, the slice construction and the loop-entry test, about ten instructions —
  which is **~22.4M Ir, 0.11%**. DERIVED, and below the item's own 0.3% floor.
  The rest of a `Clause` op's cost is the clause boundary, which a `then` owes
  like any other clause.
* **Item 7** — landed at `26ccef4ee`. Closed; its own record says nothing further
  under it can be sized from a profile, and this profile agrees.
* **Item 8** — opened for PARSE only. Its numeric, text/number and allocation
  clusters are re-derived above as ranks 5 and 6; none has been A/B'd, which is
  what it said a day ago.
* **PARSE candidate 2** — landed at `5e765dc5a`, -0.6761%. Closed.
* **PARSE candidates 3, 4** — landed. **Candidate 5** — dead, measured +0.0324%.

---

## Standing figures at `3b850d885`

### Binaries

Two independent `cargo build --release` runs of the same tree, the second with
its own `CARGO_TARGET_DIR`, both rc 0.

    sha256 rexx-run (A)  d028f1dfca75ae314cc1c0eca656cca0507eaea471c7ba901b9683b670cea0bd
    sha256 rexx-run (B)  9d28ce218f4da03d46c348b3a20ca9ab903e60a7faf0cd0b0be204baa842aa16
    objcopy -O binary --only-section=.text <bin> <out> && sha256sum <out>
    .text (A) = .text (B) = 81d2f383dcc0270c45ce1644895fc3b838f9577e7cc7e0cfaef8d24a7d680532
    sha256 rexx-ir       8853aff86986081e64d8dd0ff34d43618325ffc1c22342f60604a0008bfac91f

The whole-file hashes differ and the `.text` hashes do not, which is the
`debug = true` path-in-debug-info effect the brief names, observed rather than
assumed. **The measured binary is a build of the committed tree.**

### Whole-program instruction counts

`valgrind --tool=callgrind --dump-instr=yes --callgrind-out-file=<out> <binary>
<program>`, every run rc 0, run from a fresh empty directory. `rexxcps` rounds
interleaved A1 B1 A2 B2.

| program | rounds | mean Ir | widest spread |
|---|---|---|---|
| `bench-rexxcps/rexxcps.rex` | 20,292,462,660 / 20,291,834,967 / 20,293,182,693 / 20,293,134,507 | **20,292,653,707** | 1,347,726 = **0.00664%** |
| `bench-programs/emptyloop.rex` | 9,923,576,287 / 9,923,572,361 | **9,923,574,324** | 3,926 = **0.00004%** |
| `bench-programs/varlookup.rex` | 17,432,591,225 / 17,432,599,562 | **17,432,595,394** | 0.00005% |

Within one build, `rexxcps` spread 0.00355% (A) and 0.00640% (B); between the two
builds, 0.00664%. The brief's recorded HEAD figures — 20,292,230,586,
9,923,587,247, 17,432,604,196 — all reproduce inside their own spreads.

### Question 1: where the program stands

Ops dispatched by summing execution counts at the driver's `jmp *` addresses
(`opcount.py`, the 2026-09-20 method): **121,425,289**, identical on A1 and A2.

| quantity | 2026-09-20 `f9ffe9a8b` | now `3b850d885` | change |
|---|---|---|---|
| instructions | 21,147,250,696 | **20,292,653,707** | **-4.04%** |
| ops dispatched | 127,886,118 | **121,425,289** | **-5.05%** |
| clauses | 20,000,000 | 20,000,000 | |
| **ops per clause** | 6.39 | **6.07** | -5.05% |
| **instructions per op** | 165.4 | **167.1** | **+1.04%** |
| instructions per clause | 1,057.4 | **1,014.6** | -4.04% |

**Ir per op rose while the program got 4% cheaper**, for the reason the
2026-09-20 document already recorded when the TRACE A/B did the same: the ops
removed were the cheapest in the stream. `Ir per op` remains the wrong thing to
minimise.

The driver's three instantiations are **5,043,544,989 Ir, 24.86% of the program,
41.5 Ir per dispatched op**, against 5,250,353,399 / 24.8% / 41.1 a day ago — so
the driver's own per-op overhead did not move.

**The 20.0 Ir dispatch constant at the top of the to-do is wrong, and it is the
constant most of that file's DERIVED figures rest on.** Measured here, the loop
overhead of one region op is **8 instructions** — five of dispatch and three of
advance. The 20.0 came from a whole-program delta that also removed the `TRACE`
clauses' own execution, so it overstates an elision by about 2.5x; and the one
fusion actually measured came to 54.8 because it removed a value handoff as well.
**Use 8 as the floor for removing an op, not 20.**

### The executed op mix at this HEAD

Decoded from the two jump tables in `.rodata` (`0x3e750`, 41 entries;
`0x3e7f4`, 43 entries), counts read at each arm's first instruction. The arms sum
**exactly** to the counts at the two `jmp *` instructions, which is the check.

Outer dispatch, 19,441,281: `Clause` 16,321,024 (83.95%), `EndBranch` 840,006,
`LoopNext` 600,250, `SelectCaseText` / `EndWhen` / `EnterWhen` 560,000 each.

Region dispatch, 93,024,008:

| op | executions | share |
|---|---|---|
| the six trace ops (one shared arm) | **38,460,944** | 41.35% |
| `LoadConstant` | 9,880,016 | 10.62% |
| `Binary` | 7,860,011 | 8.45% |
| `Load` | 7,560,421 | 8.13% |
| `ConditionJump` | 5,900,006 | 6.34% |
| `Const` | 5,060,417 | 5.44% |
| `PushArg` | 4,760,417 | 5.12% |
| `Store` | 3,120,256 | 3.35% |
| `Parse` | 2,240,002 | 2.41% |
| `CallArgs` | 1,960,207 | 2.11% |
| `Arith` | 1,380,205 | 1.48% |
| `LoopHeaderValue` | 1,160,207 | 1.25% |
| `LoopRun` | 1,140,205 | 1.23% |
| `TraceKeyword` | 860,101 | 0.92% |

`Condition` and `JumpUnless` are never entered, and `TraceClause` is never
entered — `rexxcps` runs a stream compiled without clause echoes.

### Question 2: largest functions by self cost, and who has opened them

Self cost summed over every file a function is attributed to; the column sums to
the run's own `summary:` line exactly (20,292,462,660, difference 0).

| self Ir | share | function | opened? |
|---|---|---|---|
| 4,132,699,745 | 20.37% | `run_ops_from::<true>` | repeatedly |
| 1,633,801,001 | 8.05% | `exec_parse` | today |
| 874,321,446 | 4.31% | `apply_binary` | **no** |
| 528,193,834 | 2.60% | `__memcpy_avx_unaligned_erms` | **no** |
| 499,525,244 | 2.46% | `run_ops_from::<true>'2` | repeatedly |
| 485,600,000 | 2.39% | `numeric_order` | **no** |
| 468,250,849 | 2.31% | `Number::add_signed` | **no** |
| 417,520,017 | 2.06% | `append_tails` | **no — rank 1** |
| 411,320,000 | 2.03% | `run_ops_from::<false>` | repeatedly |
| 381,923,558 | 1.88% | `concat_values` | **no** |
| 359,515,098 | 1.77% | `collect_now` | **no** |
| 329,032,377 | 1.62% | `builtin::run` | **no** |
| 315,600,000 | 1.56% | `Number::mul` | **no** |
| 314,298,363 | 1.55% | `alloc_with` | **no** |
| 309,209,311 | 1.52% | `read_at` | **no** |
| 291,860,319 | 1.44% | `inline_text_to_number` | **no** |
| 270,580,205 | 1.33% | `heap_to_number` | **no** |
| 223,463,931 | 1.10% | `run_call_args` | **no** |
| 216,160,110 | 1.07% | `next_template` | today |
| 210,000,000 | 1.04% | `read_symbol` | **no — rank 1** |

By **inclusive** cost, which is what a candidate removes rather than what a line
is charged: `apply_binary` **16.35%** over 8,420,027 calls (394 Ir each),
`exec_parse` **13.12%**, `run_call_args` 8.15%, `builtin::run` 7.38%,
`read_symbol` **6.14%**, `alloc_with` 5.58%, `assign_expr_target` 4.86%,
`loop_advance` 4.73%, `concat_values` 4.57%, `append_tails` 4.45%,
`collect_now` 4.03%.

**`exec_parse`'s self cost is not comparable across the day and looks like a
regression when it is not.** It reads 8.05% against the to-do's 6.77%, +202M in
absolute terms, while its *inclusive* cost fell from 2,803,777,215 (13.71%) at
`bb81ae522` to 2,662,652,625 (13.12%) — the callee `5e765dc5a` removed was
inlined into it, moving cost from an arc into self. Rank by inclusive here.

### Question 4: what the oracle does differently on rank 1

Measured, `( ulimit -v 8388608; LD_LIBRARY_PATH=.../build/lib valgrind
--tool=callgrind .../build/bin/rexx .../rexxcps.rex )`, rc 0. **`build/` is
`-O2` (`RelWithDebInfo`), so no ratio taken from this run is a sanctioned
figure**; what follows is structural.

The oracle does not build a heap key and does not hash. `CompoundVariableTail`
is a stack object with an inline buffer; `RexxString::copyIntoTail` writes tail
pieces into it, and `CompoundVariableTable::findEntry` then walks a **balanced
binary tree** comparing `(length, bytes)` — which is also why our
`Body::Stem` carries an insertion ordinal beside each tail. Its cost shape:
`CompoundVariableTail::buildTail` inclusive 522,080,000 (4.88% of its own run),
`StemClass::evaluateCompoundVariableValue` 271,953,680 (2.54%),
`CompoundVariableTable::findEntry` 215,363,030 (2.01%),
`StemClass::setCompoundVariable` 248,484,810 (2.32%). Ours:
`append_tail_key` 1,030,200,354 (5.08%), `stem_get_at` 365,375,945 (1.80%),
`stem_set_at` 362,188,450 (1.78%). **The oracle re-renders the tail on every
access too** — it has no memo either — so the difference is in the per-access
constant, not in the algorithm: it renders into a stack buffer and compares,
where we render into a heap-backed buffer and hash. A memo would be a departure
from the oracle's design rather than a catch-up to it, and it must preserve the
post-order ordinal the tree walk gives.

---

## Concerns

* **Every "share" in the queue above is an attribution, and today produced four
  separate demonstrations that an attribution overstates what a fix recovers.**
  Rank 1's 5.08% and ranks 5 and 6 are the largest such numbers here. They are
  offered as places to open, and the first thing any of them owes is a hit-rate
  or coverage count taken *before* the work, not after.
* **The 20.0 Ir dispatch constant is retired above and I did not re-derive every
  figure that rests on it.** Item 5's revived three-op form was priced at 0.33%
  (floor) to 0.90% (ceiling) using 20.0 and 54.8. At the measured 8 Ir floor its
  lower bound falls to about 0.14%. Its upper bound is untouched, because 54.8
  was measured directly.
* **`rexxcps` is one program and rank 1 is sized only on it.** 2,500,000
  compound reads in 20,000,000 clauses is what *this* program does;
  `bench-programs/compound.rex` exists and was not profiled here.
* **The empty-region count (2,240,007) is a single-round reading** at two
  addresses in one build. Everything else in the standing table has two or four
  rounds.
* **No cache simulation was run**, so items 11 and 12 are carried forward on
  their old figures and this report adds nothing to either.
* **I did not run the test suite**, per the brief; the fourteen `NotFound`
  panics in a fresh worktree are known and were not investigated.
* **The four-component PARSE candidate 1 total reproducing to the digit is
  evidence the components are stable, not evidence the candidate is worth
  357,000,258.** It is the same derivation as before, re-taken, not an A/B.
