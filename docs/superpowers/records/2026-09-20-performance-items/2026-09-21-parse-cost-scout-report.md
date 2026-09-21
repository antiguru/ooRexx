# Scout: where the instructions inside `exec_parse` go

Read-only diagnosis, 2026-09-21. Written in the scout's own worktree
`.claude/worktrees/agent-ae27531bc164a0447`; `.superpowers/` is git-ignored
(`.gitignore:30`), so this file is not in the main checkout and nothing here
touched it.

**Four things the brief did not expect, first.**

1. **6.77% understates `PARSE` by 2.20x.** That is `exec_parse`'s *self*
   cost. Its inclusive cost is **14.90%** -- 3,150,862,887 instructions,
   1,406.6 per `PARSE`. Confirmed by A/B: deleting the `PARSE` instructions
   removes 2,673,174,085.
2. **And it is not where we are losing.** The oracle, measured on the same
   program, spends 2,296,486,650 on the same 2,240,011 `PARSE` instructions --
   **21.49% of its own run**. We are 1.37x the oracle on `PARSE` where we are
   1.98x overall, so this is one of our better axes and the whole headroom
   against its design is 854,376,237 instructions. (The installed oracle is
   `-O2`, so that 1.37x is not a sanctioned ratio -- see Q5.)
3. **The variable pattern `(p0)` is free.** Replacing it with the literal it
   always holds made the run 5,639,869 instructions *slower*.
4. **The allocations are all required.** Exactly 2,520,002 per run, every one
   a piece longer than the seven-byte inline handle; the model predicts the
   profile's count to the unit, and the oracle makes 4,480,008 for the same
   targets.

## Instrument and baseline

Retired instructions, `valgrind --tool=callgrind`, which is what every figure
below is in. The whole-run baseline:

```
valgrind --tool=callgrind --callgrind-out-file=.../base.out --dump-instr=yes \
  rust/target/release/rexx-run .../rexxcps.rex
```

Each A/B variant ran the same way without `--dump-instr=yes`, which changes
the granularity of the dump and not the counting, and each wrote its own exit
status unpiped to `status.txt`: every one is `EXIT=0`.

`EXIT=0`. `summary: 21,146,777,445` at `4ef6af5bb`, against the
21,147,250,696 recorded at `f9ffe9a8b` on 2026-09-20 -- a difference of
473,251, 0.0022%. `git log f9ffe9a8b..4ef6af5bb` is the TRACE-emission series
and nothing else, with its last member reverted, so the residue is the three
emission commits that stand. Copies of the binary and the pinned
`rexxcps.rex` were taken into a fresh directory of the scout's own, and every
variant below was run from it.

`exec_parse` self cost reproduces the recorded share:
**1,431,080,968 Ir, 6.767%** against the 6.77% recorded, over **2,240,002**
`PARSE` executions.

That 2,240,002 is derivable from the program rather than only read off the
profile, which is how the execution model below was checked before it was
used: `averaging=100` x `count=200` = 20,000 body runs, x `do loop=1 to 14` =
280,000 inner iterations, x 8 `PARSE` instructions each -- line 75, lines 78
to 81, and the three inside `subroutine:`, which the loop calls once -- =
2,240,000, plus the two in the prolog.

## The first correction: 6.77% is under half of what `PARSE` costs

`exec_parse`'s **inclusive** cost is **3,150,862,887 Ir, 14.90% of the run**
-- `callgrind_annotate --inclusive=yes base.out`, and independently the same
number by summing self plus every call arc out of it. 6.77% is self only, and
`PARSE`'s allocation, its source rendering, its target assignment and its
operand evaluation all live in callees.

**`Op::Parse` is 2,240,002 of 127,885,295 dispatched ops -- 1.75% of the ops
and 14.90% of the instructions.** A `PARSE` clause is `Clause`+`Parse`, two
ops (three for `PARSE VALUE`, which loads its expression first -- rendered
with `rexx-ir`), ~1,407 Ir; the run's average op is 165.4.

The 127,885,295 is re-derived at this revision rather than inherited: the
eight indirect jumps inside `run_ops_from` in the binary, four of which
execute, summed at their instruction addresses in the callgrind dump, which is
the method `2026-09-20-instructions-per-op.md` used. It lands 823 ops
(0.0006%) from the 127,886,118 recorded there at `f9ffe9a8b`, and the outer
table's 19,441,281 reproduces to the digit.

Confirmed by A/B rather than by attribution alone. Every `PARSE` the timed
body reaches except line 117 replaced by `nop`, so the clause count is
unchanged and stdout is byte-identical but for the clauses/second line:

| run | `summary:` | delta | executions removed | Ir each |
|---|---|---|---|---|
| baseline | 21,146,777,445 | | | |
| `v_noparse` | 18,473,603,360 | **-2,673,174,085** (-12.64%) | 1,960,000 | **1,363.9** |
| `v_nop78` (the four `parse var rc pN (p0) pM`) | 19,423,442,178 | -1,723,335,267 (-8.15%) | 1,120,000 | **1,538.7** |
| `v_nop75` (`parse value 'Foo Bar' with v1 +5 v2 .`) | 20,829,589,043 | -317,188,402 (-1.50%) | 280,000 | **1,132.8** |
| `v_nopsub` (the subroutine's two `parse var`) | 20,515,939,041 | -630,838,404 (-2.98%) | 560,000 | **1,126.5** |

The three shape-specific deltas sum to 2,671,362,073 against `v_noparse`'s
2,673,174,085 -- they agree to 0.07%, which is the check that the A/Bs are
additive and that no variant moved anything else.

**One caveat on two of those rows.** `v_noparse` and `v_nop78` also collect
less: `collect_now` self falls from 184,893,689 to 116,053,785 in both,
because removing 2,240,000 allocations removes collections. That 68,839,904
is 4.0% of the `v_nop78` delta and is genuinely `PARSE`'s, but it is GC and
not template work. **Every other variant reports `collect_now` self at
184,893,689, the baseline's own figure** -- `v_nop75`, `v_nopsub`,
`v_noupper`, `v_litpat`, `v_nodot`, `v_plus2var`, `v_plus2dot` -- so their
deltas carry no GC-count shift at all.

## Q1. Where the instructions go, by line

`exec_parse` self, 1,431,080,968 Ir, **638.9 Ir per `PARSE`**. Self attributes
an inlined callee to its host, so a third of it is `parse_template.rs`'s own
text and the rest is `core`/`alloc`/`rexx-core` inlined into it:

| Ir | share of self | inlined from |
|---|---|---|
| 512,680,345 | 35.8% | `rexx-exec/src/parse_template.rs` |
| 137,480,095 | 9.6% | `core/src/slice/iter/macros.rs` -- the two `Vec` walks |
| 90,160,083 | 6.3% | `alloc/src/vec/mod.rs` -- the pooled source buffer |
| 87,640,064 | 6.1% | `core/src/num/uint_macros.rs` -- cursor arithmetic |
| 76,160,052 | 5.3% | `core/src/ptr/non_null.rs` |
| 71,960,039 | 5.0% | `core/src/slice/index.rs` -- bounds checks on the piece |
| 59,360,016 | 4.1% | `rexx-core/src/handle.rs` -- `ObjRef::inline_text` |
| 56,280,039 | 3.9% | `core/src/result.rs` |
| 35,280,020 | 2.5% | `rexx-core/src/bytes.rs` -- `Bytes::from_slice` |
| 32,480,024 | 2.3% | `rexx-exec/src/trace.rs` -- the gates |
| 32,480,000 | 2.3% | `rexx-exec/src/builtin/string.rs` -- `find_byte` |

`parse_template.rs`'s own lines, largest first:

| Ir | :line | source |
|---|---|---|
| 89,600,059 | -- | no line attribution |
| 67,200,036 | :771 | `self.assign_expr_target(` -- the seven-argument call setup |
| 44,800,024 | :782 | `if !self.tracing_intermediates() {` |
| 23,520,018 | :305 | `remainder`'s `if self.subcurrent >= self.end` |
| 22,400,020 | :416 | `let mut cursor = self.next_template(...)` -- moving a 72-byte `Cursor` |
| 20,160,018 | :408 | the `exec_parse` prologue |
| 19,880,018 | :486 | `let (keyword, value) = match &parse.source {` |
| 17,920,016 | :438 | the epilogue |
| 13,440,012 | :282 | `next_word`'s scan for the terminating blank |
| 12,880,008 | :739 | `let piece = if index == last {` |
| 12,880,008 | :744 | `match target {` |
| 11,200,006 | :762 | `let at = match &target.kind {` |
| 11,200,000 | :317 | `find`'s guard |
| 10,360,008 | :655 | `let operand = match trigger.kind {` |
| 8,960,008 | :415 | `let mut strings = self.parse_strings(...)` |
| 8,400,006 | :419 | `let Some(trigger) = entry else {` |
| 7,840,004 | :275 | `next_word`'s leading-blank skip |
| 7,840,000 | :567 | `let traced = self.rendered_into_parse_buffer(value);` |
| 7,560,006 | :370 | `give_parse_buffer`'s capacity test |

And the calls out, 1,719,781,919 Ir, the other 54.6% of the inclusive cost:

| callee | site | calls | inclusive Ir | Ir each |
|---|---|---|---|---|
| `alloc_with` | `value.rs:213` | 2,520,002 | 433,460,004 | 172.0 |
| `assign_expr_target` | `parse_template.rs:771` | 5,600,003 | 369,600,198 | 66.0 |
| `rendered_into_parse_buffer` | `:567` | 1,960,000 | 292,320,374 | 149.1 |
| `next_template` | `:416` / `:424` | 2,240,002 / 280,000 | 167,440,080 / 101,080,261 | 74.7 / 361.0 |
| `eval_node` + `enter_eval_node` | `eval.rs:96`,`:97` | 1,400,001 | 140,000,104 | 100.0 |
| `read_at` (the `PARSE VAR` source) | `lib.rs:5343` | 1,680,000 | 70,560,000 | 42.0 |
| `to_text` (the string trigger's needle) | `:685` | 1,120,000 | 66,080,000 | 59.0 |
| `memcpy` (`Bytes::from_slice`) | `handle.rs:105`, `maybe_uninit.rs:575` | 2,520,001 | 64,120,025 | 25.4 |
| `whole_nonneg` (the `+5`) | `:702` | 280,001 | 15,120,054 | 54.0 |

Read as an answer to "what is the work": for one execution of the
`parse var rc p1 (p0) p5` shape, whose A/B cost is 1,538.7 Ir, the parts are
2 allocations (344) + the source copy (149) + the operand evaluation (100) +
2 target assignments (132) + `next_template` (75) + the needle rendering (59)
+ the source variable read (42) = 901, on top of `exec_parse`'s own code,
which averages 639 per `PARSE`. That sums to 1,540 against the A/B's 1,538.7.
**Read it as a consistency check, not as a second derivation**: the 639 is an
average across all four shapes and this is one of them, so the two agreeing
this closely is partly luck. What it does establish is that the call arcs
account for the A/B delta rather than leaving a gap.

## Q2. How much is re-derived on every execution

**All of the template interpretation.** `Op::Parse { index, src }` carries only
the clause index and, for `PARSE VALUE`, a register; `exec_parse` receives
`&Parse` -- the AST node -- and walks `Vec<Option<ParseTrigger>>`, each
trigger's `Option<Expr>` operand, and each trigger's `Vec<Option<Expr>>`
targets, deciding the shape again on every firing. Nothing about the template
is compiled, bound or cached between executions. Measured pieces, all per
`rexxcps` run:

* **137,480,095** walking the two `Vec`s (`core/src/slice/iter/macros.rs`
  inlined into `exec_parse`).
* **77,280,057** on the matches that re-decide shape: `:486` `match
  &parse.source`, `:655`+`:660` the trigger kind and its operand, `:683` the
  kind again, `:744` `Some(target)`/`None`, `:762`+`:763` `target.kind` and
  the slot, `:419` the fence `Option`.
* **140,000,104** calling the general `eval_node` for the trigger operand
  1,400,001 times, at 100.0 Ir each. Of those, 280,000 are the literal `+5` on
  line 75, whose value is `ExprKind::Constant` -- a symbol whose spelling is
  re-looked-up, re-canonicalised to a small integer, re-pushed as a temp, and
  then re-converted back to an integer by `whole_nonneg` (15,120,054 more).
* **202,926,832** on the per-call frame of the general `assign_expr_target`
  (derivation in candidate 3).
* **38,080,034** on `exec_parse`'s own prologue and epilogue.

Against that, the part that is a property of the *data* is small and mostly
irreducible: the cursor's byte scanning (`:305`, `:282`, `:275`, `:271`,
`:279`, `:740`, `:317` = 66,920,040), the pattern search
(`builtin/string.rs` `find_byte`, 32,480,000), and the allocation of the
pieces (497,580,029 including the copies, see Q4).

`code.slot_for(id)` deserves a separate note because it looks like a
name-resolution cost and is not: `Code::slot_for` is `self.slots.get(id.index())`,
a `Vec` index, and `:763` carries 5,600,003 Ir over 5,600,003 targets -- one
instruction per target attributed to that line, with the load itself spread
into the lines around it by inlining. The slot is already bound by the upfront
pass; the cost is in reaching it through the AST, not in resolving it.

## Q3. The variable pattern `(p0)` costs nothing -- measured

This candidate is dead, and the measurement says so in the wrong direction.
`v_litpat` replaces `(p0)` with the literal `'b'` that `p0` always holds, on
all four of lines 78-81, leaving the trigger kind and the match identical:

```
v_litpat  summary: 21,152,417,314     baseline 21,146,777,445
```

**+5,639,869 instructions, +0.027%** -- the literal is +5.0 Ir per execution
*more* expensive than the variable, over 1,120,000 executions, with
`collect_now` self identical in both runs so nothing moved but the operand.

The reason is in the code rather than in the number. A variable operand is
`ExprKind::Variable(id)` -> `read_symbol` -> `read_at` with `code.slot_for(id)`
-- a `Vec` index and a slot read, 42.0 Ir measured. A literal operand is
`ExprKind::Literal` -> `Interp::literal` -> `canonical_small_int` (which fails
for `b`) -> `Interp::text` -> `ObjRef::inline_text`. There is no variable-pool
lookup on the `PARSE` pattern path at all, and the branch on "is this pattern
a variable" is the `match` in `eval_node` that every expression pays.

So: a per-execution branch, as the brief guessed, and not a re-resolution.
Nothing to win here, and an attempt to special-case it would lose.

## Q4. Allocation

**One execution of line 78 performs exactly two heap allocations, and both are
required by the language.**

`assign_targets` builds a value only in the `Some(target)` arm, through
`Interp::text`, which returns a tagged handle without allocating when the
bytes fit `INLINE_TEXT = 7` (`rexx-core/src/handle.rs:23`). So an allocation
happens exactly when a piece is longer than seven bytes. Walking the four
shapes by hand, before reading the profile:

| clause | pieces | over 7 bytes | executions | allocations |
|---|---|---|---|---|
| `:75` `parse value 'Foo Bar' with v1 +5 v2 .` | `Foo B`(5), `ar`(2), `.` | none | 280,000 | 0 |
| `:78` `parse var rc p1 (p0) p5` | `This is an awfully `(19), `oring program`(13) | both | 280,000 | 2 |
| `:79` | (14), (18) | both | 280,000 | 2 |
| `:80` | (11), (21) | both | 280,000 | 2 |
| `:81` | (8), (24) | both | 280,000 | 2 |
| `:117` `parse upper arg a1 a2 a3 ., a4` | `WITH`(4), `2`(1), `ARGS`(4), `.`, `(This is the second)11`(22) | the last | 280,000 | 1 |
| `:118` `parse var a3 b1 b2 b3 .` | `ARGS`(4), ``, `` | none | 280,000 | 0 |
| `:119` `parse var rc c1 c2 c3` | `WITH`, `2`, `ARGS` | none | 280,000 | 0 |

Predicted total 2,520,000, plus 2 in the prolog. **The profile reports
`alloc_with` called 2,520,002 times from `exec_parse`.** The model is exact,
which is also why the 5,600,003 target count and the 2,240,002 instruction
count below it can be trusted.

Cost: 433,460,004 Ir inclusive for those calls (172.0 each -- and that
includes the collections they trigger, since `collect_now` is reached through
`alloc_with`), plus 64,120,025 of `memcpy` copying the bytes in, plus
35,280,020 of `Bytes::from_slice` inlined into `exec_parse`.

**None of it is an artifact of how the targets are built.** Every allocation
is a string value the program stores in a variable, and the inline handle
already removes the ones short enough to avoid. The only way to remove more is
a representation change -- a piece that borrows its parent's bytes rather than
copying them -- which is a value-model decision, not a `PARSE` one.

**And the oracle allocates 1.78x as many, measured.** `RexxTarget::remainder`
returns the string object itself only when the piece is the whole string and
calls `RexxString::extract` otherwise, with no small-string inlining under it;
in its callgrind output `RexxString::newString` is reached **1,680,006 times
from `RexxTarget::getWord` and 2,800,002 from `RexxTarget::remainder`,
4,480,008 in all**, against our 2,520,002 for the same 5,600,003 targets.

Two measured marginal costs, since "how much does one more target cost" is the
number a change would move. Both keep the clause count and produce identical
stdout:

* `v_plus2var` -- two more assigned targets on line 119, both getting the null
  string, so no allocation is added: **21,257,968,186, +111,190,741 for
  560,000 targets = 198.6 Ir per assigned `PARSE` target with nothing to
  allocate.**
* `v_plus2dot` -- two more `.` placeholders instead: **21,181,789,503,
  +35,012,058 for 560,000 = 62.5 Ir per placeholder.** And `v_nodot`, removing
  line 75's trailing `.`, gives 20,150,757 / 280,000 = **72.0 Ir**, the same
  quantity measured in the other direction on a different template position.

So an assigned target costs 136 Ir more than a placeholder that does the same
cursor work -- and the difference is `Interp::text`, `push_temp`,
`slot_for` and the general `assign_expr_target`, not the parsing.

## Q5. What the oracle does -- read, and then measured

**It interprets the template per execution too, and every leaf of it is a
pre-resolved retriever reached through one virtual call.** That is the whole
difference, and it says the floor is not "compile `PARSE` away".

### The measurement, and what it does to the size of this prize

The oracle runs the same pinned `rexxcps.rex` under callgrind, from a fresh
directory, under the project's own wrapper (`ulimit -v`,
`LD_LIBRARY_PATH=.../build/lib`, three separate descriptors), `EXIT=0`,
stdout byte-identical to ours but for the clauses/second line:

```
summary: 10,688,744,881
RexxInstructionParse::execute   inclusive  2,296,486,650  (21.49%)
```

**The two interpreters do structurally identical `PARSE` work on this
program**, which is what makes the counts comparable at all:

| | this crate | oracle |
|---|---|---|
| `PARSE` instructions executed | 2,240,002 | 2,240,011 |
| template steps (`next_template` / `RexxTarget::next`) | 2,520,002 | 2,520,011 |
| triggers applied (`apply_trigger` / `ParseTrigger::parse`) | 3,920,018 | 3,920,018 |
| `PARSE` inclusive Ir | 3,150,862,887 | 2,296,486,650 |
| **Ir per `PARSE`** | **1,406.6** | **1,025.2** |
| Ir per trigger | 803.8 | 585.8 |
| share of its own run | 14.90% | 21.49% |

The trigger row is the only one of ours that is not read off a profile:
`apply_trigger` is inlined into `exec_parse` and has no call count of its own,
so ours is derived from the program -- 14 per inner iteration (2 on line 75, 2
on each of 78 to 81, 2 on 117, 1 on 118, 1 on 119) x 280,000, plus 18 in the
prolog. The oracle's 3,920,018 *is* measured, and it is the same number, which
is the check on both.

**So `PARSE` is 1.37x the oracle where the whole run is 1.98x
(21,146,777,445 / 10,688,744,881). It is one of our better axes, not a worse
one, and the total headroom against the oracle's design is 854,376,237
instructions.**

**What that figure is not.** `build/` is `RelWithDebInfo`,
`CXX_FLAGS = -O2 -g -DNDEBUG` (checked in `CMakeCache.txt` today; `bin/rexx`
62,600 bytes and `lib/librexx.so.4` 17,853,856, the 5.3 binary `CLAUDE.md`
records), and this tree's standing rule is that a performance *ratio* against
that build is void, so **1.37x is not a sanctioned ratio and should not be
quoted as one**. `-O2` against `-O3` is not measured anywhere here; the only
recorded pair is `-O0` against `-O3`, which is a different question. And an
instruction ratio is not a time ratio for the reasons `CLAUDE.md` gives. What
the two counts *are* good for is sizing a candidate: the oracle's design
costs about 1,025 Ir per `PARSE` of these shapes on the binary that is
installed, and the candidates below have to be read against that rather than
against zero.

`RexxInstructionParse::execute` (`interpreter/instructions/ParseInstruction.cpp:136`)
switches on `stringSource`, calls `target.init(...)`, and then loops
`for (i = 0; i < triggerCount; i++)` over a flat `ParseTrigger*[]` built at
parse time, calling `trigger->parse(context, stack, &target)` (`:230`-`:245`).
`ParseTrigger::parse` (`ParseTrigger.cpp:189`) switches on a `ParseTriggerType`
enum and then runs an assignment loop over `RexxVariableBase *variables[]`.
Structurally that is our `for entry in &parse.template` with
`apply_trigger`/`assign_targets` under it. What differs:

1. **The source is never copied.** `RexxTarget::next` (`ParseTarget.cpp:82`)
   keeps the `RexxString*`, pushes it on the evaluation stack to protect it,
   reads `string->getStringData()` directly in `getWord` and `remainder`, and
   builds a new string only for `PARSE UPPER`/`LOWER` (`:115`-`:122`). We copy
   the source into a pooled `Vec<u8>` on every execution, 2,520,000 times, at
   a measured 381,360,635 Ir per run.
2. **The trigger's operand is a pre-built retriever**, `value->evaluate(context, stack)`
   -- one virtual call which, for a constant, returns the already-built object.
   Ours is a general `eval_node` over an `ExprKind`, 100.0 Ir a time.
3. **Each target is a pre-built `RexxVariableBase`**, and `variable->assign(context, value)`
   is one virtual call into a retriever that already holds the variable's
   index. Ours is `assign_expr_target`, a seven-argument function that
   re-matches `target.kind` and re-derives the name, 66.0 Ir a time.
4. **The trace decision is taken once per trigger, not once per target.**
   `ParseTrigger::parse` has *two* copies of the assignment loop and chooses
   between them on `context->tracingResults()` (`:248` and `:290`), and the
   untraced copy skips building the value for a `.` entirely
   (`skipWord`/`skipRemainder`, `:319`-`:328`). We test
   `tracing_intermediates()` inside the loop, once per target.
5. **It allocates more than we do** for the pieces (point in Q4).

The implication for the floor: the oracle's per-execution work is one enum
switch per trigger plus one indirect call per operand and per target. It is
not working from a compiled form -- so a compiled op stream for `PARSE` is not
what closes this gap, and something much cheaper than that is enough to match
its shape.

## Ranked candidates

Every figure is retired instructions per `rexxcps` run, and each row says
whether it is an A/B on a modified program or an attribution from the
baseline profile. **Candidates 1 and 3 overlap**: the
`assign_expr_target` call-frame share (202,926,832) is inside both, and they
must not be added.

**And they are ceilings, provably.** 1 + 2 + 4 + 5, the non-overlapping set,
is 1,091,554,489 instructions -- more than the 854,376,237 that separates us
from the oracle's whole `PARSE` implementation (Q5). Each row is what the
machine currently spends on a thing it would stop doing; none is what the
replacement would cost.

### 1. Bind the template once: a prepared per-instruction form instead of the AST

**What the machine stops doing.** Walking two `Vec`s of AST nodes per
execution; re-matching `parse.source`, the trigger kind, the operand `Option`,
the target `Option` and the target's `ExprKind`; entering the general
`eval_node` for an operand whose form was fixed at parse time; entering the
general seven-argument `assign_expr_target` for a target whose slot was bound
by the upfront pass.

**Figure (attribution, ceiling).** 595,767,122 Ir per run = **2.82% of the
run**, from:

```
iterator walks (core/src/slice/iter/macros.rs in exec_parse)   137,480,095
per-execution matches (:486 :655 :660 :683 :744 :762 :763 :419) 77,280,057
eval_node + enter_eval_node arc                                140,000,104
assign_expr_target per-call frame, PARSE's share               202,926,832
exec_parse prologue + epilogue (:408 :438)                      38,080,034
```

It is a **ceiling**, not an estimate of the win: a prepared form still walks
an array, still fetches each operand and still stores each target. What it
removes is the AST shape, not the work.

**What it breaks if done wrong.** The template semantics -- which target gets
the remainder, where the match position sits after a pattern, whether a
backward trigger assigns the null string. `corpus/lang/parse_triggers.rex`
exists to catch exactly that and names the wrong answer each mistake prints;
it runs under `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`.
`corpus/lang/parse_sources.rex` and `parse_template.rex` cover the sources and
the placeholder.

**Reach.** Large. Touches `rexx-parse`'s `Parse`/`ParseTrigger`, `Plan::build`
(`note_parse` already binds the names), and either `Op::Parse` or a side table
the clause index reaches. This is the one candidate that is not local.

### 2. Parse the source where it already is, instead of copying it per execution

**What the machine stops doing.** `rendered_into_parse_buffer` -- taking a
`Vec<u8>` from the pool, rendering the source value into it through `to_text`,
handing it back afterwards -- on every execution of every `PARSE` whose source
is a value or an argument. The oracle does not do this at all (Q5.1).

**Figure (attribution, ceiling).** 402,920,653 Ir per run = **1.91% of the
run**: `rendered_into_parse_buffer` inclusive 381,360,635
(`callgrind_annotate --inclusive=yes`, 2,520,000 calls, 151.3 each) plus
21,560,018 on the pool lines inside `exec_parse` -- `:370` 7,560,006 and
`:372` 5,040,004 in `give_parse_buffer`, `:382` 6,720,006 in
`give_parse_strings`, `:436` 2,240,002 at its call site.

**Feasibility, stated because it is the reason this is second and not first.**
The bytes live in a `Bytes::Inline` inside the arena `Object`
(`rexx-core/src/bytes.rs:41`), and the template walk *allocates*, so the arena
`Vec` can reallocate mid-walk and move them. A plain `&[u8]` borrow across the
`&mut self` calls is therefore both unsound and rejected by the borrow
checker. A sound version has `Cursor` hold the rooted `ObjRef` and re-derive
the slice after each call that can allocate -- cheap if done per trigger,
ruinous if done per byte, since `next_word` indexes the source byte by byte.
**`PARSE UPPER`/`LOWER` must keep copying**, which is measured: `v_noupper`
(line 117 as `parse arg` rather than `parse upper arg`, same clause count,
`collect_now` unchanged) is 21,100,522,147, **-46,255,298 for 280,000
executions = 165.2 Ir for the UPPER pass** over the two argument strings.

**What it breaks if done wrong.** A stale slice after a collection -- wrong
piece contents, or worse. `tests/collect_stress.rs` is the test that exercises
allocation during interpretation; the corpus differential above catches the
content. Run the debug gate for it, not only release: `Interp::enter_clause`'s
`debug_assert` and the temps-frame watermark are compiled out of `--release`.

**Reach.** Local to `parse_template.rs` plus a `Cursor` that holds an
`ObjRef`. Contained.

### 3. A store for a `PARSE` target that is a plain variable with a bound slot

**What the machine stops doing.** Entering a seven-argument general
assignment function, re-matching `target.kind`, re-deriving
`code.symbols.name(*id)` unconditionally when only the trace gate reads it
(`run.rs:2688`), re-deciding `match at`, and paying a call frame -- 5,600,003
times, for what is in every one of those cases "write this `ObjRef` into slot
N".

**Figure (attribution).** The arc `exec_parse -> assign_expr_target` costs
**369,600,198 Ir per run, 66.0 Ir for each of 5,600,003 stores**
(measured directly). The removable part is the per-call frame and dispatch:
`assign_expr_target`'s own self cost on its per-call lines -- `:2677`
prologue 101,640,042, `:2686` `match &target.kind` 64,160,024, `:2688` the
name 11,200,006, `:2689` `match at` 16,800,009, `:2695` the `rendered` test
11,200,006, `:2771` epilogue 58,080,024 = 263,080,111 -- of which `PARSE`'s
share by call count (5,600,003 of 7,260,003; the other caller is
`assign_evaluated`) is **202,926,832 Ir = 0.96% of the run**. The assumption
that makes this a division rather than a measurement is that those six lines
cost the same per call whichever caller entered -- they are the prologue, the
epilogue and three tests, so it is a reasonable one, but it is an assumption
and the number is not an A/B.

Independent bracket from an A/B: one assigned target costs **198.6 Ir**
end to end (`v_plus2var`), a placeholder doing the same cursor work costs
**62.5** (`v_plus2dot`) / **72.0** (`v_nodot`).

**What it breaks if done wrong.** A target that is *not* a plain variable --
`parse var s a.b` writes a compound tail, `parse var s stem.` replaces a stem
-- silently writing the wrong slot. `corpus/lang/parse_template.rex` and the
compound corpus programs cover it; the `debug_assert!(at.is_none(), ...)`
tripwires already in `assign_expr_target`'s `Stem` and `Compound` arms are
what turn a mis-specialisation into a failure rather than a wrong answer, and
they only fire in the debug gate.

**The fourth shape is already a refusal, which narrows this.** The parser
accepts a message term as a `PARSE` target -- `instruction.rs:2221`,
`parse_variable_or_message_term`, with `parse arg q~x is rc 0` measured beside
it -- but `assign_expr_target` has no `Message` arm and falls through to
`Loud::expression` at `run.rs:2768`. Measured today: `parse value 'aa bb' with
o~v1 w2` against an `::attribute v1` is oracle rc 0 (`v1=[aa]`, `w2=[bb]`) and
**exit 120, `rexx-exec: a message send is not implemented`** here, while the
same assignment written `o~v1 = 'aa'` outside `PARSE` is rc 0 on both sides.
So the target shapes `exec_parse` can reach are Variable, Stem and Compound
and nothing else. That is a fact about where the boundary sits today, not a
property, so a specialisation resting on it should assert it.

**Reach.** `parse_template.rs` and a new entry point beside
`assign_expr_target` in `run.rs`. Contained; no parser or op-stream change,
because `code.slot_for(id)` already answers.

### 4. Decide the trace shape once per `PARSE`, not once per target

**What the machine stops doing.** Loading the trace cache, testing
`debug_pause`, and branching, inside the per-target loop -- which is precisely
what the oracle avoids by keeping two copies of that loop (Q5.4).

**Figure (attribution, ceiling).** The gates inside `exec_parse` cost
86,800,050 Ir per run (0.410%): `:782` `if !self.tracing_intermediates()`
44,800,024, the `trace.rs` bodies inlined into `exec_parse` 32,480,024,
`:600` 6,720,000, `:679` 2,800,002. One gate per `PARSE` instead of one per
target would leave about 2,240,002 x 8 = 17,920,016, so **68,880,034 Ir =
0.326% of the run**.

**What it breaks if done wrong.** The `TRACE R` / `TRACE I` transcripts, which
are byte-compared: `tests/trace_oracle/parse_placeholder.rex` and its
`.expected`, and `corpus/lang/parse_template.rex`.

**The property a hoist rests on is that the setting cannot change part-way
through one `PARSE`, and it was run rather than argued.** A target can be a
message term, so an assignment *can* invoke a method, so it looked reachable.
Probe: a setter that does both `trace i` and `call trace 'I'`, called from a
`PARSE` target inside a `TRACE R` region. Oracle rc 0, and the transcript
shows the setter's own clauses traced at `I` in its own activation while the
remaining targets of the enclosing `PARSE` still emit plain `>>>` -- the
caller's setting, unchanged. `TRACE` is per-activation and the setter is a new
one. On our side the same program is exit 120 because the message target is
unimplemented at all (see candidate 3), so today nothing a `PARSE` target does
can reach a `TRACE` instruction. **Assert that rather than inherit it**: both
halves are boundary facts that move.

**Reach.** `parse_template.rs` only.

### 5. Index the piece once per target instead of three times

**What the machine stops doing.** `&cursor.string()[piece]` is built three
times per assigned target -- once for `Interp::text`, once as the `rendered`
argument that only `trace_assignment`'s gate reads, once for `trace_result` --
each a bounds-checked slice index, with two `piece.clone()`s around them
(`parse_template.rs:750`, `:775`, `:783`).

**Figure (attribution, ceiling).** Slice indexing inlined into `exec_parse` is
71,960,039 Ir per run (0.340%); one of the three is **23,986,680 Ir =
0.113%**.

**What it breaks if done wrong.** Nothing observable if the slice is hoisted
correctly; the borrow is already immutable and `Cursor` is a local. The risk
is passing the wrong range after a hoist, which the corpus programs catch.

**Reach.** `parse_template.rs` only. Smallest change on the list.

### Not a candidate, and the measurement that closed it

**The variable pattern.** Q3: replacing `(p0)` with the literal it holds made
the run 5,639,869 instructions *slower*. There is no variable-pool lookup on
that path.

**Allocation of the pieces.** Q4: 2,520,002 allocations, every one a string
the program stores, every one over the seven-byte inline threshold, and the
oracle makes 4,480,008 for the same targets. Nothing to remove without
changing the value model.

## Concerns

* **A `PARSE` target that is a message term is unimplemented here**, and the
  oracle does it. `parse value 'aa bb' with o~v1 w2` against an
  `::attribute v1`: oracle rc 0, `v1=[aa]` `w2=[bb]`; this crate exit 120,
  `rexx-exec: a message send is not implemented`, from `assign_expr_target`'s
  `other =>` fallback (`run.rs:2768`). The same assignment written
  `o~v1 = 'aa'` outside `PARSE` is rc 0 on both sides, so it is `PARSE`'s
  reach into the assignment path that is missing, not the assignment. Found
  while checking whether a target assignment can run Rexx code; not otherwise
  in this scout's scope, and not looked for elsewhere.

* **Candidates 1 and 3 overlap and must not be summed.** 202,926,832 is in
  both.
* **Every ceiling above is a ceiling.** Each says what the machine stops
  doing and what that currently costs; none says what the replacement costs,
  because the replacement does not exist to measure. Project memory records
  what happens when a "share of a share" is quoted as a win, and these are
  one step better -- counts of instructions, from one deterministic
  instrument -- but they are still not deltas.
* **Two A/B rows carry a GC-count change.** `v_noparse` and `v_nop78` collect
  less than the baseline (`collect_now` self 116,053,785 against
  184,893,689). That 68.8M is real and is `PARSE`'s, but it is allocation
  pressure, not template work. The other seven variants are GC-identical to
  the baseline.
* **The A/Bs price removing a whole instruction, not a mechanism.** They
  bound what `PARSE` costs; they cannot attribute it. The mechanism figures
  come from the profile, and the two agree where they can be compared (the
  1,538.7 A/B against 1,540 built from the call arcs).
* `v_noupper` changes the *contents* of `a1 a2 a3` from upper to lower case
  while keeping every length identical, so nothing downstream changes size.
  Stdout is unchanged. But it is the one variant whose delta is small enough
  (46M) that a second-order effect could be a visible fraction of it.
* The oracle figures in Q5 are read from `interpreter/`, which is 5.3 and
  matches the binary at `build/`. No oracle *timing* claim is made here, and
  1.37x is not a sanctioned ratio because `build/` is `-O2`.
* **`apply_trigger`'s execution count on our side is derived, not measured**
  -- it is inlined and has no call count. It agrees with the oracle's measured
  3,920,018, which is the only check on it.
* **Nothing in the repository was edited.** Everything above ran from
  `/tmp/claude-1000/.../scratchpad/parse-scout` (this crate's runs and the
  variants), `.../scratchpad/oracle-parse` (the oracle) and
  `.../scratchpad/trace-probe` (the setter probe), each a fresh directory,
  with copies of the pinned `rexxcps.rex` rather than the file in the tree.
  The variants are generated by `mkvariants*.py` there, which asserts each
  line it replaces is non-empty, so a variant that silently missed its target
  cannot be produced. This report is in the scout's worktree; the main
  checkout has no copy.
