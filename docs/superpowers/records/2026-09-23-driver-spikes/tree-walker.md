# Bake-off: the tree-walker against the IR (spike/tree-walker)

Base `b5dd351d6`; base `.text` sha256
`748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32`, matching
the round-2 setup's figure.

Prior art, cited not repeated: on 2026-09-10 at `86ca524e6` the tree-walker
was slower than the IR on all ten axes, 0.9% (`dispatchclass`) to 85%
(`varlookup`), same binary under `REXX_ENGINE`, in cycles/wall (project
memory, `oorexx-parity-shape-by-axis`). Deleted 2026-09-11 by `47ae2a18d`,
`64cc47c81`, `8657503a9`, `458f49f76`.

## Prediction (written 2026-09-23 before any measurement)

Phase A, historical pair, callgrind ex-libc, tree-walker relative to IR on
the same binary:

| axis | predicted TW vs IR | reasoning |
|---|---:|---|
| rexxcps | +40% (IR ~1,150/cl, TW ~1,600/cl) | per-clause temps frame, recursive `eval` over `Expr`, loop constructs re-entering `run_bounded` per pass |
| varlookup | +80% | the 2026-09-10 cycle ratio was 85% and instructions track cycles here |
| arith | +30% | arithmetic helpers are shared; the tree only adds per-clause stepping |
| emptyloop | +60% | pure clause loop, all machinery |
| dispatch | +5% | time is in the send path, shared |
| compound | +35% | tail building is shared, clause cost is not |

Phase C, today's tree, same feature-on binary: the tree-walker gains every
improvement made to shared code (`exec_instruction`, `eval`, PARSE in place,
hashing) and none of the IR-only ones (fusion, flat loops, register file). I
expect the ratio to widen slightly: rexxcps TW ~1,500/cl against IR ~1,015/cl
(+45%), varlookup +90%, arith +30%, emptyloop +70%, dispatch +5%, compound
+35%.

Category split on rexxcps: the tree-walker spends **more** in interpretation
machinery (recursive `eval`, per-clause clause-unit entry and temps frames,
`run_bounded` re-entry per loop pass) by roughly +400/cl, and about the same
in PARSE, built-ins, conversion, allocation and libc. The one place I expect
it to spend **less** is the register file (`roots.rs` traffic of the IR's
`temp_at`/`set_temp`, 28.9/cl) and the driver's spill/reload; I do not expect
those to outweigh the eval recursion.

Feature-on IR against feature-off base IR: within 1% on every axis, but any
figure under 2.5% is not evidence either way (round 1's layout floor).

## Phase C: the bake-off on today's tree

Instrument: callgrind `summary:` minus `libc.so.6` and `ld-linux`
(`sumobj.py` from `cold-arms-files/`), two rounds, the second in reverse
order, every run from a fresh empty directory, mean of the two rounds. Every
run exited 0; the five bench programs' stdout is byte-identical across the
three configurations (rexxcps differs only in its timing line). Binaries by
`.text` sha256: base (feature off, built from `b5dd351d6`) `748b2f06...`; the
spike tree built feature-off `748b2f06...` (identical); feature-on
`66b1f5c5...`. Scratch: `round2/tree-walker/mC2/results.tsv`.

| axis | base IR (feature off) | IR, feature on | on vs base | tree-walker, same binary | TW vs IR |
|---|---:|---:|---:|---:|---:|
| rexxcps | 18,638,728,650 | 18,670,086,343 | +0.168% | 23,911,017,386 | **+28.07%** |
| varlookup | 17,122,899,557 | 17,122,906,326 | +0.000% | 35,818,875,212 | **+109.19%** |
| arith | 11,764,230,548 | 11,765,228,196 | +0.008% | 13,107,813,082 | **+11.41%** |
| emptyloop | 9,685,886,531 | 9,685,888,168 | +0.000% | 13,935,863,322 | **+43.88%** |
| dispatch | 20,711,078,932 | 20,771,086,280 | +0.290% | 23,556,044,660 | **+13.41%** |
| compound | 9,914,436,523 | 9,914,436,874 | +0.000% | 14,359,666,967 | **+44.84%** |

Max spread between the two rounds of any cell: 17,310 instructions.
`rexxcps` ex-libc per clause (20,000,000 clauses): **IR 933.50, tree-walker
1195.55**; base 931.94.

**Feature on against feature off, IR both:** under 0.3% everywhere, under the
2.5% floor. Not purely layout: the feature-on IR also runs the
`if self.tree_walker` tests at activation entry and at each call, consistent
with the two axes that moved being the call-heavy ones (`dispatch` +0.290%,
`rexxcps` +0.168%) while the loop-only axes moved by under 3,000
instructions. Not controlled further; it does not touch the verdict.

An earlier full run (`mC/`, feature-on `.text` `9e093da5...`) had the walker's
call-site cache on a SipHash `HashMap`: `rexxcps` TW was +30.40%, 26.7/cl of
it hashing (`hash_one` 411,659,976 inclusive under `walk_site_resolution`,
2,800,408 calls). It was switched to `rexx_core::NameMap` (Fx) before the
numbers above; every other axis was unchanged to within the round spread.

### Category split on `rexxcps`

Method and scripts are `2026-09-22-category-rollup.md`'s (`parse_cg2.py`,
`classify.py`), self cost per function, whole run including libc, **each
column sums to its own `summary:`, diff 0**. That rule list predates the
walker and some renames, so rules were added for names it has never seen,
applied to both engines: walker stepping, `condition_value`, `scan_when`,
`apply_flow`, `run_bounded` -> machinery; walker call-site functions,
`invoke_call`, `eval_traced_argument` -> call handling; `slot_of` and the
`Box<[u8]>->ObjRef` map it probes, whose sole hot caller is
`assign_expr_target`, -> compound; `arith_small_int`, `arith_general` ->
arithmetic. With them "everything else" is 3.01/cl on both sides, the same
startup remainder as the record. The oracle column is the record's.

| category | IR /cl | tree-walker /cl | TW - IR | oracle /cl (record) |
|---|---:|---:|---:|---:|
| interpretation machinery | 399.44 | 584.09 | **+184.65** | 98.29 |
| variable access, simple | 15.46 | 18.86 | +3.40 | 23.64 |
| variable access, compound | 86.32 | 104.49 | +18.17 | 49.51 |
| arithmetic and comparison | 99.13 | 104.20 | +5.07 | 68.55 |
| text/number conversion, string building | 108.36 | 144.05 | **+35.69** | 46.97 |
| PARSE | 93.95 | 94.40 | +0.45 | 52.30 |
| built-in functions | 37.86 | 37.86 | 0.00 | 14.58 |
| call and argument handling | 42.23 | 49.77 | +7.54 | 30.23 |
| allocation and collection, own | 47.77 | 54.83 | +7.06 | 110.95 |
| the C allocator | 45.02 | 44.28 | -0.74 | 0.00 |
| libc string and memory | 32.40 | 34.16 | +1.76 | 38.59 |
| everything else | 3.01 | 3.00 | -0.01 | 0.82 |
| **total** | **1010.95** | **1274.02** | **+263.07** | **534.44** |

The IR column reproduces the record's own to within a few per clause
(machinery 399.44 against 403.96, total 1010.95 against 1014.59).

**Where the tree-walker spends less than the IR: nowhere that matters.** The
C allocator (-0.74/cl), and inside call handling the IR's
`run_call_args`/`push_call_arg`/`resolve_fixed_call`/SipHash (-25.4/cl
together) against the walker's `invoke_builtin_call` and site lookup
(+29.9/cl).

**Where it spends more, per function:**

* **Clause stepping costs the same in both engines.** The walker's
  `walk_step_in_temps_frame` (clause unit, `walk_step`, the `IF`/`SELECT`/loop
  arms and `walk_bounded`, all inlined into it) is **242.45/cl** self; the
  IR's whole driver, `run_ops_from::<true>` + `'2` + `::<false>`, is
  **246.13/cl** self, and that includes expression evaluation. By inlined
  source file: the walker's is 121.45 `tree_walker.rs`, 35.79 `run.rs`, 18.29
  `trace.rs`, 9.35 `clause.rs`; the IR driver's is 115.51 `drive.rs`, 22.26
  `run.rs`, 17.03 `roots.rs` (the register file), 6.70 `trace.rs`.
* **The machinery gap is the expression evaluator.** `eval_node` +109.62/cl,
  `eval` +32.87, `enter_eval_node` +28.59: **+171.1/cl** of recursive `Expr`
  walking that the IR has compiled into register ops inside the 246/cl above.
  Plus `exec_instruction` +21.62, which the IR mostly bypasses with its own
  ops.
* **Conversion +35.69 is one function, `literal` (0.60 -> 36.30/cl)**: the
  tree path renders a literal into a value each time it is evaluated, the IR
  holds it as a constant. The oracle's `RexxLiteral` holds its value too, so
  this is a missing tree-path optimisation, not a property of the tree shape.
* **Compound +18.17**: `assign_expr_target` resolves the stem's slot by name
  (`slot_of` plus an Fx map probe, 11.7/cl) where the IR's compiled store
  carries the slot. Also a tree-path gap, not structural.
* Simple reads +3.40; allocation +7.06 (`collect_now`, `alloc_with`, `Body`
  drops: more short-lived values, `literal`'s among them).

**Verdict: the tree shape has no structural advantage that the IR driver
cannot get.** Stepping clauses through a recursive walker costs what the IR's
driver costs to step *and* evaluate them, and the walker then pays +171/cl for
expressions on top. The oracle's 98.29/cl of machinery is not reached by
restoring a tree: our walker's stepping alone is 2.5x the oracle's whole
machinery category. What the pair does isolate is that most of the IR
driver's 246/cl is per-clause bookkeeping common to both engines (the clause
unit, trace gating, frames), not op dispatch.

**Prediction check.** rexxcps predicted +45% (TW ~1,500/cl), measured +28.07%
(1,195.55/cl ex-libc); varlookup +90% predicted, +109.19%; emptyloop +70%,
+43.88%; compound +35%, +44.84%; arith +30%, +11.41%; dispatch +5%, +13.41%.
Machinery predicted +400/cl, measured +184.65. The register file: I expected
the walker to save the IR's `roots.rs` 17/cl; it does, and spends it back in
the same function on temps pushes and run-time trace gating.

## Phase A: the historical pair

Last commit with both engines under `REXX_ENGINE`: `47ae2a18d` (Op::Generic
already deleted; the engines are whole-run alternatives there). Switch
verified: `REXX_ENGINE=bogus` exits 2 with its rejection message; the
tree-walker's `rexxcps` dump names no `run_ops_from` while the IR's names it
three times and has no `step_in_temps_frame`. Built in a detached worktree in
scratch (removed afterwards), `.text` `88eea307...`. Its pinned `rexxcps` and
the five bench programs are byte-identical to today's (`cmp`). Same
instrument and interleaving as Phase C (`round2/tree-walker/mA/results.tsv`).

| axis | IR ex-libc | tree-walker ex-libc | TW vs IR |
|---|---:|---:|---:|
| rexxcps | 19,428,513,748 | 23,344,086,818 | +20.15% |
| varlookup | 17,692,266,126 | 32,113,237,486 | +81.51% |
| arith | 11,820,363,418 | 12,700,960,744 | +7.45% |
| emptyloop | 9,560,244,775 | 12,010,224,857 | +25.63% |
| dispatch | 19,905,436,152 | 21,170,404,231 | +6.35% |
| compound | 9,978,803,000 | 13,218,977,879 | +32.47% |

**Instructions per clause on `rexxcps`, ex-libc: IR 971.43, tree-walker
1167.20** (with libc 1047.78 and 1255.34). Same ordering as the 2026-09-10
cycle measurement; on `varlookup` the instruction ratio (+81.5%) sits next to
its cycle ratio (85%). Since then the IR went 971.43 -> 933.50/cl ex-libc
(-3.9%) and the gap widened from +20.2% to +28.1%, while the restored walker
went 1167.20 -> 1195.55 (+2.4%, part of it the call-site cache and debug pause
that correctness now needs and the 09-11 walker did not have).

## Correctness

* `REXX_ENGINE=tree-walker REXX_CORPUS_GATE=1 memcap 8G cargo test --release
  -p rexx-exec --features tree-walker --test corpus corpus_differential`:
  **`604 of 604 matching`**, exit 0, on the measured build. With
  `REXX_ENGINE=ir`: `604 of 604 matching`, exit 0.
* **The corpus really runs the walker**: with the walker's `IF` decision
  inverted (`if !holds`) the same command prints `0 of 604 matching`, exit
  101 (zero because the bootstrap library is walked too). Restored from a
  copy.
* The first restore reached 600/604. Added since the deletion, with no tree
  path: (1) the interactive-debug pause after a clause (`trace_debug`,
  `trace_debug_skip`, `external_trace`), added to `walk_step_in_temps_frame`
  with the IR's rule (no `Flow`, no pending trap; `=` reruns the clause); (2)
  call sites that keep their resolution (`library_routine_site_kept_merged`),
  added as a per-node cache keyed by the call node's address with the IR's
  generation rule. Its first version kept misses and broke
  `call_miss_not_cached` and `library_routine_load_library`; the IR's "a miss
  is never kept" rule fixed both. Nothing was stubbed.
* Default build unchanged: the feature-off release `.text` is
  `748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32`, equal
  to base, re-checked after every edit to shared code.
* `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --
  -D warnings` exit 0 with and without `--features rexx-exec/tree-walker`.

## The no-generic guarantee

* The walker is `rust/crates/rexx-exec/src/run/tree_walker.rs`, declared
  `#[cfg(feature = "tree-walker")] mod tree_walker;` in `run.rs`; cargo
  feature `tree-walker = []`, off by default. Nothing under `ir.rs` or
  `src/ir/` changed.
* Engine choice is one bool on `Interp`, set once from
  `Invocation::with_tree_walker()` before the bootstrap library runs, read at
  `run_activation` (every body) and `run_fragment` (every `INTERPRET`).
  `rexx-run` reads `REXX_ENGINE=ir|tree-walker` under the feature (an
  environment variable because every word after the path is the program's
  argument); the corpus harness reads the same variable.
* Structural: the walker's entry points are `pub(in crate::run)` or private,
  so IR code cannot call them. **Red run 1**: a feature-gated method in
  `ir/drive.rs` calling `self.walk_activation(..)` fails to compile with
  `error[E0624]: method walk_activation is private`.
* `tests/tree_walker_isolation.rs` (feature on) scans `ir.rs` and every file
  under `src/ir/` for `tree_walker`, `tree-walker` and every `fn` name the
  walker module declares (derived from the file; asserted non-vacuous).
  **Red run 2**: a method in `ir/drive.rs` calling the crate-visible
  `walk_eval_call` compiles, and the test fails naming `tree-walker` and
  `walk_eval_call` in `drive.rs`. Restored from a copy; green again.
* The direction is walker into shared code only (`exec_instruction`, `eval`,
  the stepped-clause unit, `leave_select`, `loop_advance`, ...).
  `run_loop_with_header`/`run_repeating` take the IR's `BodyEngine`, so the
  walker carries its own copies rather than a new variant on an IR type.

## Concerns

1. **This is not the 09-11 walker.** Its loop runner is a copy of today's, and
   it gained the debug pause and the call-site cache. Phase C measures a
   correct tree-walker on today's tree, which is what the question needs.
2. **Two tree-path costs are fixable and were not fixed**: `literal`
   (+35.7/cl) and name-resolved compound stores (about +11.7/cl). Fixing both
   would leave `rexxcps` roughly +23% behind, and would not touch the +171/cl
   evaluator.
3. **The debug pause is taken after non-block clauses only**; the IR also
   pauses after an `IF`/`SELECT`/`WHEN`/`DO` header region. No corpus program
   covers it.
4. **Fragments keep no call sites** under the walker (their nodes are dropped
   after the `INTERPRET`); the IR keeps them for one fragment execution. Not
   in the corpus.
5. The call-site cache is keyed by node address, safe because
   `Interp::programs` keeps every `Program` alive and fragments are excluded.
   Spike-grade, not a design.
6. The category split added rules to the 2026-09-22 list (scratch
   `round2/tree-walker/cat/classify.py`); the machinery and conversion findings
   rest on per-function rows (`cat/top.py`), not on the rules.
7. `cargo test -p rexx-exec --release`: the first attempt was OOM-killed at the
   8G cap while *compiling* (exit 137, no test ran); it now builds outside the
   cap and runs under it. Result: see the gate status line below.

## Gate status

`cargo test -p rexx-exec --release --no-fail-fast` at `bc0db4d6d`, built
outside the cap, run under `memcap 8G`: feature off 1577 passed / 1 failed,
feature on (`--features tree-walker`) 1578 passed / 1 failed, 50 test binaries
each, exit 101 both. The one failure both times was
`refusal_sites::the_table_holds_every_constructor_the_source_defines`:
`corpus/refusal-sites.tsv` records `run.rs` line numbers, which the feature
hooks shifted. Re-derived with `REXX_REFUSAL_SITES_REFRESH=1`; the diff is
line numbers only (identical with every `.rs:N` masked), and `--test
refusal_sites` is then 5 passed / 0 failed in both configurations.

