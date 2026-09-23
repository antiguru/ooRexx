# Spike cold-arms: per-arm outlining of the driver's never-executed arms

Branch `spike/cold-arms`, cut from `0ba0f3876` (the worktree tool cut from the
master lineage at `c2edef977`; the coordinator authorised
`git switch -c spike/cold-arms 0ba0f3876`, and `git rev-parse HEAD` read
`0ba0f387664dac94e91bf62f9acbf6dbeb034241` after it).

Base binary: `bin/base-rexx-run`, built at `0ba0f3876` in its own
`CARGO_TARGET_DIR`, `.text` sha256
`748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32`. The
frame-arena, handler-table and pgo spikes' `bin/base.text` have the same hash.

## Prediction (written before any measurement of a variant)

Committed as `98e7d6b71`, before the hot-arm list existed.

**What the change can move.** The outlined arms do not run on any axis, by
construction, so no axis executes a new call. The only thing that can change
is how LLVM allocates registers and lays out the hot arms once the cold arms'
bodies are gone from the function. So the brief's question is purely the one
`2026-09-22-frame-instrument-falsified.md` raised: does dividing the driver
into functions differently help the hot code.

**Why I expect little.** Most arms that look cold are already a single call
into a non-inlined function (`exec_message`, `exec_expose`,
`exec_instruction`, `signal_to_label`, `scan_when`, ...) plus the
`Result`-to-`break 'cold` mapping. Outlining such an arm removes the match on
`clause.kind`, the loud-error construction and one call's argument setup,
which is tens of instructions per arm, not hundreds. The 57% never-executed
share of `run_ops_from::<true>` is mostly error and slow paths *inside hot
arms* (the `Err(failure) => break 'cold` tails, `arith_general`'s fallback,
`assign_evaluated`), which this spike leaves alone. So the change removes a
minority of the cold code.

**Engaging the precedent.** The out-of-line store tail and Candidate E both
put a call on a path that ran; that is a per-op price, and this spike has no
such price. What it shares with them is the codegen perturbation the
store-fusion report measured as non-monotone: four never-firing arms cost
`rexxcps` +2.32% and moved `emptyloop` -1.03%. Removing code should be
subject to the same lottery in the other direction, not a guaranteed win.

**Numbers (full outlining, ex-libc, head vs base):**

| axis | predicted |
|---|---:|
| `rexxcps` | -0.5% |
| `varlookup` | -1.0% |
| `arith` | 0.0% |
| `emptyloop` | -0.5% |
| `dispatch` | -0.5% |
| `compound` | -0.5% |

Every cell is inside the 2.5% noise floor, and I expect the signs to be mixed
in reality. **For the control I predict the proportionality test fails**: the
half variant will not sit at roughly half of the full variant's delta with
the same sign on most axes, which would make any movement allocator noise
rather than an effect of removing cold code. The prediction is "does not
stick"; the value of the spike is in measuring that cleanly.

## Which arms execute

Method (`cold-arms-files/hot.sh`, `cold-arms-files/jt.py`): the base binary
under `valgrind --tool=callgrind --dump-instr=yes --dump-line=yes
--compress-pos=no --compress-strings=no` on all six axes; every indirect
`jmp *%r` in both `run_ops_from` instantiations whose index is the op's tag
byte (`movzbl` from the op, and for the outer table `cmp $0x28`); each table
read from the ELF, one entry per `Op` variant in declaration order; each
target's first-instruction execution count read from the dump. The order
assumption was checked, not assumed: addr2line puts each region-table target
on its own arm's lines (`LoadConstant` at `drive.rs:916`, `Load` 985, `Arith`
1086, `Binary` 1147, `Store` 1258, `Say` 1398, `Return` 1449, `Queue` 1508,
`CallNamed` 1580, `Exec` 1630, `ConditionJump` 1704, `JumpUnless` 1752,
`WhenTest` 1764). Output: `cold-arms-files/jt.true.tsv`, `jt.false.tsv`.

`<true>` has one outer table (41 entries; tags above 40 go to the `_` arm) and
**two** region tables, a tail-duplicated copy of the region dispatch. `<false>`
has the same three. One further indirect jump in each instantiation indexes
by something other than the op tag and was skipped.

**Line attribution alone was tried first and is wrong**: it called region
`Jump`, `JumpUnless` and all six value-echo trace arms cold. The trace arms
are hot (38,460,944 entries on `rexxcps`): LLVM merged their shared
`tracing_intermediates()` gate into one target that all six table entries
point at, so which of the six fired is not distinguishable, and all six are
treated as hot. `Jump` and `JumpUnless` inside a region really are cold at
this base (0 entries everywhere); the op-mix table in
`2026-09-20-instructions-per-op.md` predates the `ConditionJump` fusion.

An arm is hot if any table entry for it counted above zero on any axis in
either instantiation; the counts include startup, so e.g. `Say` is hot at 1.

**Cold on every axis:**

* region walk: `TraceClause`, `CallExpr`, `Prefix`, `Queue`, `Call`,
  `WhenTest`, `Jump`, `JumpUnless`, and the arms for ops that do not belong in
  a region (`LoopNext`, `Clause`, `SelectCaseText`, `EnterWhen`,
  `EnterOtherwise`, `EndBranch`, `EndWhen`);
* outer loop: `EnterOtherwise` and the `_` arm.

**Left inline** (brief step 4): region `Jump` and `JumpUnless`. Each body is
a control transfer (`break 'region *target`, and `register_holds` plus a
three-way transfer); there is no body to move that is not the transfer
itself.

## The variants

Every outlined function is `#[cold] #[inline(never)]`. The not-a-region-op
arms share one `cold_not_driven(&'static str) -> Failure`, since their bodies
are identical apart from the name; it takes one argument, so it is not a
union of anything.

| binary | source | `.text` sha256 | `run_ops_from::<true>` |
|---|---|---|---|
| `base` | `0ba0f3876` | `748b2f06...4c32` | 15,061 bytes, 2,886 instructions |
| `full` | `a021d37e1`: every cold arm above outlined | `db2d7656...76ea` | **gone: inlined into `run_activation`** (7,768 to 19,859 bytes) |
| `half` | `full` with every other cold arm in source order re-inlined (`cold-arms-files/half.patch`) | `e8965850...5281` | 12,710 bytes, 2,430 instructions |
| `fullni` | `full` plus `#[inline(never)]` on `run_ops_from` (`fullni.patch`) | `54f113c4...d987` | 11,558 bytes, 2,230 instructions |
| `baseni` | `base` plus the same `#[inline(never)]` | `748b2f06...4c32` | **identical to `base`** |

Instruction counts are `objdump -d --disassemble=<sym> | grep -c '^  [0-9a-f]*:'`.

**The half split**, by source order of the outlined arms: kept out of line
are `TraceClause`, `Prefix`, `Call`, region `LoopNext`, region
`SelectCaseText`, region `EnterOtherwise`, region `EndWhen` and the outer `_`
arm; re-inlined are `CallExpr`, `Queue`, `WhenTest`, region `Clause`, region
`EnterWhen`, region `EndBranch` and outer `EnterOtherwise`.

**`full` changed the function boundary, which the brief did not anticipate.**
With the cold arms out, `run_ops_from::<true>` fell under LLVM's inlining
threshold and was inlined through `run_ops` into `Interp::run_activation`.
So `full` against `base` measures outlining *and* a merge of the driver into
its caller. `fullni` holds the boundary fixed, and `baseni` shows the
attribute alone is a no-op on base (byte-identical `.text`), so **`fullni`
against `base` is the outlining alone**, and it is the row the verdict rests
on.

## Measurement

`cold-arms-files/measure.sh`: `valgrind --tool=callgrind
--compress-strings=no`, two rounds, round 1 in the order base, full, half,
fullni and round 2 reversed, every run from its own fresh empty directory,
each run's binary `.text` hashed inside that directory before running (12
runs per hash, four hashes, all matching the table above). Ex-libc is
`summary:` minus the self cost of every object ending `libc.so.6` or
containing `ld-linux` (`sumobj.py`, which asserts its per-object sum equals
`summary:`). Mean of the two rounds; spread is the widest round-to-round
difference inside any one build on that axis.

| axis | base ex-libc | full | fullni | half | spread |
|---|---:|---:|---:|---:|---:|
| `rexxcps` | 18,638,735,749 | 18,826,108,034 **+1.005%** | 18,813,122,898 **+0.936%** | 18,812,367,762 **+0.932%** | 0.0000% |
| `varlookup` | 17,122,899,274 | 17,920,911,182 **+4.660%** | 17,711,903,356 **+3.440%** | 17,768,900,673 **+3.773%** | 0.0001% |
| `arith` | 11,764,226,356 | 11,845,339,708 **+0.689%** | 11,831,832,552 **+0.575%** | 11,831,838,073 **+0.575%** | 0.0000% |
| `emptyloop` | 9,685,884,789 | 9,935,885,312 **+2.581%** | 9,810,887,772 **+1.291%** | 9,810,888,932 **+1.291%** | 0.0001% |
| `dispatch` | 20,711,081,272 | 20,721,083,058 **+0.048%** | 20,811,078,586 **+0.483%** | 20,826,084,775 **+0.555%** | 0.0000% |
| `compound` | 9,914,441,191 | 10,159,455,176 **+2.471%** | 10,109,449,294 **+1.967%** | 10,124,445,832 **+2.118%** | 0.0002% |

**Every variant is slower on every axis.** The prediction had the sign wrong
on five of six axes.

**No outlined function executed in any of the 36 variant runs**
(`sumobj.py`'s last column lists the self Ir of every symbol containing
`cold_`; it is empty in all of them). So the change of work done is zero and
everything above is codegen.

## Control: half against full

The brief's rule: same direction and roughly proportional means real; half
moving more than full, or the opposite way, means allocator noise.

Against `fullni` (the outlining alone), `half` moves **the same amount** on
`rexxcps` (+0.932% against +0.936%), `arith` (+0.575% both) and `emptyloop`
(+1.291% both, the same 125,000,000 instructions to within a few thousand),
and **more** on `varlookup` (+3.773% against +3.440%), `dispatch` (+0.555%
against +0.483%) and `compound` (+2.118% against +1.967%). Against `full` it
also moves more on `dispatch`. Nowhere is half near half. **By the brief's
rule this is not an effect of how much cold code was removed.**

## Where the instructions went

`cold-arms-files/split.py` diffs self Ir per host symbol (callgrind's `fn=`)
between base and each variant, round 1, and splits it three ways: the
driver's own symbols (every `run_ops_from` and, for `full`, `run_activation`
that now hosts it), three callees whose inlining moved, and the rest.

| variant | axis | driver self | callees whose inlining moved | total |
|---|---|---:|---:|---:|
| fullni | `rexxcps` | +0.599% | +0.337% | +0.935% |
| fullni | `varlookup` | +0.333% | +3.107% | +3.440% |
| fullni | `arith` | +0.225% | +0.349% | +0.579% |
| fullni | `emptyloop` | +0.000% | +1.291% | +1.291% |
| fullni | `dispatch` | **-0.072%** | +0.555% | +0.483% |
| fullni | `compound` | **-0.353%** | +2.320% | +1.967% |
| half | `rexxcps` | +0.595% | +0.337% | +0.932% |
| half | `varlookup` | +0.666% | +3.107% | +3.773% |
| half | `dispatch` | +0.000% | +0.555% | +0.555% |
| half | `compound` | **-0.202%** | +2.320% | +2.118% |
| full | `varlookup` | +1.553% | +3.107% | +4.660% |
| full | `dispatch` | **-0.507%** | +0.555% | +0.048% |

(`split.md` has every row.) "Other" is under 0.006% in every row.

**Two separate things happened, and neither is what the spike set out to
measure.**

1. **Hot callees stopped being inlined into the driver, identically in all
   three variants.** In base, `Interp::arith_small_int` and the drop glue of
   `Option<LoopHeaderValues>` have no self cost of their own (they are
   inlined); in every variant they are out-of-line calls on the hot path:
   `arith_small_int` 342,000,000 on `varlookup`, the drop glue 125,000,290 on
   `emptyloop` (5 instructions per iteration) and 190,000,295 on `varlookup`.
   On `rexxcps` and `arith`, `arith_general_body` was folded into
   `arith_general` instead. The column is the **same number to the
   instruction in `full`, `fullni` and `half`** on every axis, so it is a
   step that any of these edits triggers, not something that scales with the
   outlining. The `header: Option<LoopHeaderValues>` whose drop moved is the
   region walk's local, dropped at the end of every region. I did not find
   why LLVM's inliner changed its mind; only that it did, and where.
2. **The driver's own self cost moves both ways**: +0.6% on `rexxcps` in all
   three variants, -0.51% to +1.55% elsewhere, with `full` (driver merged
   into `run_activation`) better on `dispatch` and worse on `varlookup` than
   `fullni`. Mixed sign, all under 2.5%, and not ordered by how much was
   outlined: allocator noise, as predicted.

So **the allocator question the brief asked is answered "noise"**, and on
top of that noise the edit costs a fixed inlining step on every axis that
outweighs any allocator gain found (the best driver-self figure, `full` on
`dispatch` at -0.507%, is cancelled by the +0.555% inlining step on the same
axis).

## Correctness

At `a021d37e1` (`full`), tree unchanged during the run (`git status` clean
apart from the untracked `.claude/`):

    cargo build --release -p rexx-exec --all-targets                        -> rc 0
    memcap 8G cargo test -p rexx-exec --release --no-fail-fast              -> rc 101, 1546 passed / 32 failed / 1 ignored, 49 binaries
    REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus corpus_differential
                                                                            -> rc 0, "mode: STRICT (the gate) -- REXX_CORPUS_GATE is set", "604 of 604 matching"

**All 32 failures are environmental**: this worktree has no `build/`
(`dispatch/library.rs:795`, `run/tests/routines.rs`: "the worktree's
build/lib is three directories above this crate: ... NotFound") and no
`ootest/` (`rexx-extract/src/lib.rs:503`: "cannot read .../ootest/ooRexx/base/
expressions"), both untracked checkouts in the main rust-rewrite worktree.
The same command on base source in this worktree (`tests-base.sh`, base
`drive.rs` copied in, built outside the cap, restored after):
`rc 101, 1546 passed / 32 failed / 1 ignored, 49 binaries`, and the sorted
`FAILED` lines of the two runs are identical (`diff` empty). (A first
attempt compiled inside `memcap 8G` and was OOM-killed during the build; it
produced no test results and is not counted.)

## Verdict

**Does not stick.** Outlining every cold arm makes every axis slower, `rexxcps`
by +0.94% with the driver boundary held fixed (+1.01% as built), outside a
0.0000% spread; the half-outlined control moves as much as the full one or
more, so the driver's own movement is allocator noise, and the consistent loss
is a fixed inlining step (`arith_small_int` and `Option<LoopHeaderValues>`'s
drop glue leaving the driver) that every variant pays identically.

## Concerns

* **The inlining step may be separable.** Forcing `arith_small_int` inline,
  or keeping the region's `header` out of the per-region drop, might recover
  the step; the remaining driver-self movement is still mixed-sign and under
  the noise floor, so I would not expect a net win, but it is untested.
* **A smaller driver gets inlined into its caller.** Any change that shrinks
  `run_ops_from` enough (this one, or others in this round) can cross the
  same threshold; `full` against `fullni` shows the merge alone moving the
  axes by -0.44% (`dispatch`) to +1.29% (`emptyloop`). Other spikes shrinking the driver should
  check `nm` for `run_ops_from::<true>` before reading their numbers.
* The `CallExpr` body now runs `enter_eval_node`'s stack probe one frame
  deeper. It is cold on every axis and the corpus is unchanged, but a
  program sitting exactly at the depth limit through that op could see a
  different threshold.
* The six value-echo trace arms share one merged gate target, so which of
  them executes cannot be read off the jump table; all six were kept inline.
* The report's instruction counts for `run_ops_from::<true>` (2,886) differ
  from the 2,894 in `2026-09-22-driver-frame-pressure.md`; the command here
  counts objdump instruction lines and that record did not say how it
  counted.
