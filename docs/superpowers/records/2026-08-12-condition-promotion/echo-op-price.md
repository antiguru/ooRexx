# What the unconditional intermediate echo ops cost when nothing is traced

Measurement spike, 2026-08-12/13. Base commit **`0459167cc8c513eda448b5bb3a66cb0fdd64a836`**
(`plan/rust-rewrite` tip at the moment the spike started), measured in a detached
worktree with its own `CARGO_TARGET_DIR`. Nothing here is a candidate patch and
nothing here was merged.

## What arm B is, and why it is wrong on purpose

Arm B skips pushing `Op::TraceLiteral`, `Op::TraceRead`, `Op::TraceOperator`,
`Op::TracePrefix` and `Op::TraceFunction` when the chunk's compile-time trace
setting does not echo intermediates. **A build that does this is incorrect in
general, and I am not proposing it.** The ops being present unconditionally is
what lets a `TRACE I` executed part-way through a run start printing
intermediates against a chunk that was already compiled. Arm B loses exactly
that, and the loss is directly visible:

```
trace i          arm A (head)                  arm B (gated)
a = 2              2 *-* a = 2                   2 *-* a = 2
say a + 3            >L>   "2"                     >>>   "2"
                     >>>   "2"                     >=>   A <= "2"
                     >=>   A <= "2"                3 *-* say a + 3
                   3 *-* say a + 3                 >>>   "5"
                     >V>   A => "2"
                     >L>   "3"
                     >O>   "+" => "5"
                     >>>   "5"
```

The chunk is compiled before the `TRACE I` clause runs, so under B the setting
arrives after the ops that would have served it were never emitted. B is a price
probe. It answers "what are these ops worth", not "should we do this".

## The four arms

| arm | what it is | behaviour | md5 |
|---|---|---|---|
| **A** | the unmodified base commit | echoes everything | `1aad0ea5f32075e18be36ca7c1b68402` |
| **B** | gated: `echo_intermediates` threaded into `push_native`/`push_value`/`push_read`, omitted at emission | drops intermediates a mid-run `TRACE I` would want | `ae9213e80c43213d2735ccfc15fd8b64` |
| **C** | control 1: B's plumbing exactly, flag forced `std::hint::black_box(true)` | identical to A | `3f92b082b9e63a9554b5b75cd85e3f14` |
| **D** | control 2: same plumbing, flag forced `!std::hint::black_box(false)` | identical to A | `5e85dbced9dd235eb4eaee1c09a7f860` |

B, C and D differ in **one line** of `compile.rs` and nothing else. `black_box`
rather than a literal `true` so the branch in `push_native` survives constant
folding: C and D pay B's per-emission branch and take it every time. A, C and D
are byte-for-byte identical in behaviour, so every difference between them is
code layout.

D was not in the brief. I added it because with only A and C there is exactly
one sample of the layout distribution, and the A-vs-C movement turned out to be
the same size as the whole effect being measured. Two independent
identical-behaviour pairs are what turn "the control moved" into a band.

Omission is at emission, so `close_region` and the patch list see the shorter
stream and every jump target stays an index into the stream that exists. No
finished op vector is filtered.

### The compile-time assertions needed no loosening, and that is a finding

The brief expected `assert_literal_echoes_follow_their_load` and its siblings to
need relaxing. **They did not, and I changed none of them.** Every one of the six
is written as *"for each echo op present, check the op immediately before it"* —
none asserts that an echo op must be present at all. An omitted echo is
invisible to them. The evidence that they really ran and really held: arm B
builds, `ir::corpus_shape_tests::every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops`
(which compiles a corpus through `compile()` under `TraceMode::NORMAL`) reports
`ok` on B, and none of the six assertion messages
appears anywhere in B's test output.

That is worth writing down for whoever designs a real version: the checks that
look like they pin the echo ops in place do not. Nothing in `compile.rs` would
have gone red if the ops had been dropped by accident.

## Correctness gate: byte-identity, per program

Every program that was timed, run under each arm from a fresh empty working
directory with `ulimit -v 8388608` and `REXX_ENGINE=ir`; stdout, stderr and exit
status compared.

| program | A vs B | A vs C | A vs D | exit |
|---|---|---|---|---|
| `rexxcps-fixed.rex` (the timed rexxcps) | identical | identical | identical | 0 |
| `bench-programs/arith.rex` | identical | identical | identical | 0 |
| `bench-programs/varlookup.rex` | identical | identical | identical | 0 |
| `bench-programs/strings.rex` | identical | identical | identical | 0 |
| `bench-programs/alloc4c.rex` | identical | identical | identical | 0 |

Also checked, not timed: `compound`, `emptyloop`, `startup` (identical, exit 0)
and `dispatch`, `alloc`, `heapshape` (identical, exit 120 — the Phase 5 gaps,
identical on every arm). The per-echo-op probes `q1`/`q2`/`q3` are identical on
A and B too. **No program diverged, so no timing is void.**

rexxcps needed a deterministic variant to be gate-able at all, because the stock
program prints timing-derived figures. `rexxcps-fixed.rex` is the oracle sample
copied out (the oracle tree was not written to) with three changes: the adaptive
second trial forced off (`if total>1 | trial=2 then leave` → `if 1 then leave`,
so a fast arm cannot run different work from a slow one), and the two
timing-derived `say`s stripped of their `format(total,…)` and
`format(1000/thousand,…)` terms. Everything inside the timed loop is untouched.
**I fixed `count`=100 and `averaging`=100 (the file's own defaults) and used wall
time**, rather than the printed clauses-per-second figure, because that figure is
self-calibrated and therefore not comparable across arms — and because with the
figure suppressed the program's output is deterministic and can be gated.

## What `cargo test --workspace --no-fail-fast` does to arm B

Run under `memcap 8G`. Harness totals: **B — 1408 passed, 57 failed. A (same
worktree, same command, as a baseline) — 1438 passed, 27 failed.**

The A baseline is not zero, and the reason is environmental rather than
anything about this spike: a detached worktree has no `ootest/` tree (it is
untracked in the main checkout), so `rexx-extract` cannot read
`ootest/ooRexx/base/expressions` and everything downstream of it panics. Those
same failures appear on both arms and are not B's doing. Taking the difference,
the tests **B breaks and A does not** are:

*The op-stream goldens* — they name the echo ops in literal expected op vectors:
`ir::golden_tests::` `a_bare_symbol_compiles_to_a_native_read_in_each_of_its_three_kinds`,
`a_call_promotes_at_the_root_and_below_it`,
`a_chain_of_operators_reuses_the_destination_register`,
`a_compiled_read_names_the_symbol_its_expression_does`,
`a_compiled_write_names_a_slot_only_for_a_simple_target`,
`a_constant_symbol_is_a_native_load`,
`an_assignment_of_a_literal_compiles_to_a_constant_load_and_a_store`,
`an_expression_that_only_contains_a_symbol_is_more_than_that_symbols_read`,
`an_if_with_an_else_compiles_to_a_clause_region_and_two_jumps`,
`an_if_with_no_else_emits_no_branch_end_jump`,
`a_prefix_operator_compiles_to_a_native_op_and_its_own_echo`,
`a_say_of_a_bare_symbol_compiles_to_a_native_read`,
`a_select_cases_own_value_outlives_the_registers_its_whens_take`,
`a_select_with_an_otherwise_compiles_to_a_scan_chain_and_two_frames`,
`a_select_with_no_otherwise_scans_out_onto_its_own_end`,
`a_traced_if_carries_its_clause_echo_as_an_op_of_the_region`,
`a_traced_select_echoes_its_header_and_each_listed_when`,
`every_binary_operator_but_arithmetic_compiles_to_one_op`,
`nested_ifs_reuse_their_registers`,
`one_literal_written_twice_is_one_interned_constant`,
`precedence_decides_which_operator_is_the_inner_one`,
`two_assignments_and_two_says_in_one_body_reuse_one_register`.

*The dual-engine harnesses* — real divergence, the tree-walker still printing
what the compiled stream no longer can: `both_engines_agree_on_every_branch_shape`
(`[if under trace i, true path]`, tree-walker emits `>L> >L> >O>`, compiled
stream emits none of them) and `both_engines_agree_on_every_case_file`.

*The oracle trace transcripts* — the compiled stream now diverges from the C++
oracle under `TRACE I`: `compound_read_write_covers_the_resolved_compound_name`,
`call_arguments_covers_the_argument_prefix_at_every_position_shape`,
`control_variable_reread_covers_a_body_that_writes_the_control_variable`,
`function_call_covers_the_function_prefix_after_the_callees_own_lines`,
`prefix_operators_covers_plus_and_backslash`,
`trace_output_covers_clause_result_assignment_literal_variable_and_operator`.

That list *is* part of the answer: it maps exactly what the gating breaks —
goldens (cosmetic, they would be regenerated), engine agreement under a mid-run
`TRACE I` (the real defect), and oracle fidelity under `TRACE I` (the same
defect seen from outside). Nothing in the untraced population notices.

## Method for the timings

Wall clock; `ulimit -v 8388608`; `REXX_ENGINE=ir`; a fresh empty working
directory created and destroyed per run; all four arms interleaved inside each
round; arm order rotated between rounds through `A B C D`, `B C D A`, `C D A B`,
`D A B C`; one sitting; a host-idle gate of six consecutive five-second
`/proc/stat` samples at or above 90% idle before starting (recorded: 93.88,
91.68, 93.38, 94.76, 91.27, 90.37 per cent). 20 rounds. Every run exited 0.

An earlier three-arm sitting (A/B/C, 16 rounds, its own idle gate) is kept as an
independent replication; its numbers are quoted at the end and they agree.

## Wall clock: the answer, and why it is not the number it looks like

Medians over 20 rounds, seconds:

| axis | A (head) | B (gated) | C (control 1) | D (control 2) |
|---|---|---|---|---|
| rexxcps | 3.2556 | 3.2606 | 3.2751 | 3.2843 |
| arith | 2.1704 | 2.1766 | 2.3666 | 2.2870 |
| varlookup | 2.4593 | 2.3840 | 2.5704 | 2.5656 |
| strings | 3.2878 | 3.3463 | 3.4062 | 3.3819 |
| alloc4c | 1.1808 | 1.1614 | 1.1841 | 1.1952 |

**B vs C — the mechanism as the brief framed it — with round-by-round wins:**

| axis | B-vs-C | rounds B faster |
|---|---|---|
| rexxcps | −0.44% | 14/20 |
| arith | −8.03% | 20/20 |
| varlookup | −7.25% | 20/20 |
| strings | −1.76% | 19/20 |
| alloc4c | −1.92% | 18/20 |

**A vs C — the control movement, same rounds, two builds that are byte-for-byte
identical in behaviour:**

| axis | A-vs-C | rounds A faster | C-vs-D | rounds C faster | A-vs-D |
|---|---|---|---|---|---|
| rexxcps | −0.60% | 16/20 | −0.28% | 12/20 | −0.87% |
| arith | −8.29% | 20/20 | +3.48% | 0/20 | −5.10% |
| varlookup | −4.32% | 20/20 | +0.19% | 7/20 | −4.14% |
| strings | −3.48% | 20/20 | +0.72% | 6/20 | −2.78% |
| alloc4c | −0.28% | 13/20 | −0.93% | 7/20 | −1.21% |

**On arith the control moved the axis by more than the entire B-vs-C
difference**, in the same direction, with the same 20/20 unanimity. A two-arm
A-vs-B read would have been worthless here in a subtler way than usual: on
arith B is +0.29% against A and ahead in only 6 of 20 rounds, so the naive
comparison says "no gain" while B-vs-C says "−8%", and both are reading layout.

The three behaviourally identical builds give the floor directly:

| axis | fastest of A/C/D | slowest of A/C/D | do-nothing band | B | B vs the fastest identical build |
|---|---|---|---|---|---|
| rexxcps | 3.2556 | 3.2843 | 0.88% | 3.2606 | +0.16% (inside the band) |
| arith | 2.1704 | 2.3666 | **9.04%** | 2.1766 | +0.29% (inside the band) |
| varlookup | 2.4593 | 2.5704 | 4.51% | 2.3840 | **−3.06%** (faster than all three) |
| strings | 3.2878 | 3.4062 | 3.60% | 3.3463 | +1.78% (inside the band) |
| alloc4c | 1.1808 | 1.1952 | 1.22% | 1.1614 | **−1.65%** (faster than all three) |

**On three of the five axes B lands inside the band that three
identical-behaviour builds already span.** On the other two it is outside it —
varlookup by 3.06% (B beats A, C and D 20/20, 20/20 and 20/20) and alloc4c by
1.65% (18/20, 18/20, 18/20) — but both are under this project's stated ~7-point
resolution floor on these axes, and under the 9.04% do-nothing movement this
sitting measured on arith. **I am not presenting any of these as a real wall-time
win.**

## Instruction counts, beside the wall clock and not instead of it

`perf stat -e instructions:u`, three runs per arm per axis, median. Run-to-run
spread within an arm was at most 0.09%, and C and D agree to 0.00% — so this
instrument sees the op stream and is nearly blind to the layout that dominates
the wall clock.

| axis | A | B | B-vs-A | B-vs-C | A-vs-C (the plumbing itself) |
|---|---|---|---|---|---|
| rexxcps | 28,816,872,127 | 28,345,072,864 | −1.64% | −1.85% | −0.21% |
| arith | 20,368,308,303 | 19,844,305,493 | −2.57% | −2.67% | −0.10% |
| varlookup | 44,441,816,629 | 40,812,815,658 | −8.17% | −8.87% | −0.76% |
| strings | 44,703,788,431 | 43,917,762,155 | −1.76% | −2.03% | −0.28% |
| alloc4c | 9,367,885,262 | 8,968,929,885 | −4.26% | −4.52% | −0.28% |

So the ops are real work: **1.6% to 8.2% of every user instruction these programs
retire is an intermediate echo op deciding not to print.** And that instruction
share is *not* a time share — 8.17% of instructions on varlookup bought 3.06% of
wall clock, and 2.57% on arith bought nothing measurable at all.

### The price of one echo op, exactly

Three probes, `x = 1`, `x = 1 + 1`, `x = 1 + 1 + 1`, each in a five-million
iteration loop, A minus B:

| probe | instructions removed per iteration | echo ops per iteration |
|---|---|---|
| `x = 1` | 40.000 | 1 `TraceLiteral` |
| `x = 1 + 1` | 141.004 | 2 `TraceLiteral` + 1 `TraceOperator` |
| `x = 1 + 1 + 1` | 242.000 | 3 `TraceLiteral` + 2 `TraceOperator` |

Each additional literal-plus-operator pair costs 101.00 instructions, twice over
(101.004 and 100.996), so the relation is linear and the individual costs solve:

> **`Op::TraceLiteral` = 40, `Op::TraceRead` = 45, `Op::TraceOperator` = 61 user
> instructions each, when nothing is being traced.**

`TraceRead` comes from varlookup, whose promoted body is
`2 × TraceRead + TraceLiteral + TraceOperator` per iteration and which removes
191.00 instructions per iteration.

The check that this model is not a story: applied to `arith.rex`'s five promoted
clauses it predicts 1048.02 instructions removed per iteration, i.e. 524,008,034
over the run. Measured: **524,002,810**. That is 0.001% error against a number
the model never saw.

Forty to sixty-one instructions is far more than "a dispatch, a register read and
a call that returns". Whoever designs the real version should look at why: the
gate each of these reaches is `self.trace_mode().intermediates`, which goes
through the activation, and it is asked once per echo op rather than once per
clause or once per chunk.

## Second question: rexxcps's own `TRACE` and `ADDRESS` clauses

`rexxcps-notrace.rex` is `rexxcps-fixed.rex` with the inner-loop
`trace value trace(); address value address()` removed, and the per-averaging-pass
`trace value tracevar` and `trace off` removed with it. The inner pair is the one
that can matter: it runs `count × averaging × 14` = 140,000 times per run against
the outer two at 100 each.

Timed on **arm A only**, both copies inside the same rounds of the same sitting,
order swapped between rounds, `count` fixed at 100 and `averaging` at 100:

> **3.2556 s → 3.1877 s, −2.09% (−0.0679 s), the no-trace copy faster in 20 of 20
> rounds.** Paired per-round deltas: median −0.0637 s, range −0.1033 to −0.0317 s.
> `instructions:u`: 28,816,872,127 → 28,292,179,054, **−1.82%**.

**Removing clauses invalidates the program's own clauses-per-second figure**,
which divides by a fixed 1000 clauses per iteration, so that figure is not used
anywhere here — wall time at a fixed `count` is, for both copies.

Worth putting side by side: rexxcps's own three `TRACE`/`ADDRESS` clauses cost it
**524,693,073 instructions**, which is *more* than gating away every intermediate
echo op in the entire program (471,799,263). A benchmark that exercises the trace
machinery in its inner loop is paying more for the clauses it writes than for the
ops the compiler emits behind everything else it writes.

## Replication

The earlier three-arm sitting (A/B/C, 16 rounds, its own idle gate, before D
existed) agrees closely on everything it covers: arith B-vs-C −8.14% (20-round
sitting: −8.03%) with A-vs-C −8.20% (−8.29%); varlookup B-vs-C −7.24% (−7.25%)
with A-vs-C −4.41% (−4.32%); rexxcps B-vs-C −0.09% (−0.44%). The arith control
anomaly reproduces at 16/16 and 20/20 in two separate sittings, so it is a
property of the C binary and not of an afternoon.

## Does the price justify designing a real version?

**Not on this evidence.** The ops are measurable work — 1.6% to 8.2% of retired
user instructions, at an exactly-pinned 40/45/61 instructions each — but on four
of five axes that converts to nothing that clears the layout noise these builds
generate for free, and the one axis that moves at all (varlookup, −3.06%) moves
less than a do-nothing recompile moved arith (+9.04%). A correct version also
costs more than arm B does: it has to keep a chunk usable when `TRACE I` arrives
mid-run, which means either recompiling on the setting change, keying the chunk
cache on the intermediates flag as well, or a cheaper run-time gate — and the
first two put the cost back in a different place. The instruction counts do point
somewhere worthwhile, but it is not this: **40 to 61 instructions to decide not to
print is the defect worth attacking**, and hoisting that decision out of the
per-op path would help the traced and untraced cases both, without making a
mid-run `TRACE I` wrong.

## Housekeeping

* Base commit measured: `0459167cc8c513eda448b5bb3a66cb0fdd64a836`.
* All work was done in a detached worktree under the session scratchpad with its
  own `CARGO_TARGET_DIR`. **The main working tree was never written to by this
  spike** — it was only read, and the binaries were run from fresh empty
  directories elsewhere. Its `git status` shows another implementer's
  in-progress edits and none of mine.
* Every arm was rebuilt after every source change; the four binaries were saved
  aside by md5 and the timings were taken from the saved binaries, so no arm was
  measured with another arm's build.
* The oracle tree at `/home/moritz/dev/repos/ooRexx/` was read only;
  `rexxcps.rex` was copied out before being modified.
* Nothing was committed. The worktree was removed on completion.
