# Branch review: value model and supporting crates

STATUS: DONE

Slice: `rexx-exec/src/{value,stem,plan,activation,error,trace}.rs` plus all of `rexx-core`, `rexx-num`, `rexx-parse`, `rexx-extract`.
Range: `9f68662a..HEAD` (112 commits).

Priorities, per the review brief:

1. D15 rendering-fixed-at-creation and SmallInt admissibility, including paths added by Tasks 9-13.
2. D15a stem rules (tombstones, replace vs mutate, verbatim case-sensitive tails).
3. rexx-core root set and heap, incl. the last-day `alloc_with` collect-on-every-allocation mode.
4. Citation drift: verify C++ citations by containment, not by the named line.

## Findings

### F1 (Important): controlled-loop header values are not rounded at loop entry

`setup_controlled` (`rust/crates/rexx-exec/src/run.rs`, ~line 2155) stores the **exact parse** of `initial`, `to` and `by` (`arith_operand`, no rounding).
The oracle rounds all three with prefix `+` at loop entry: `ControlledLoop::setup`, `DoBlockComponents.cpp:126-129` (`_initial = _initial->callOperatorMethod(OPERATOR_PLUS, ...)`), and the same `OPERATOR_PLUS` call for `TO` (~:145) and `BY` (~:166), verified by containment.
The gap is masked while `NUMERIC DIGITS` stays constant (every later use re-rounds to the same width) and becomes wrong output the moment digits changes inside the loop body.

Measured divergences (probes in scratchpad, oracle vs ours):

* `numeric digits 3; do i = 1.23456 to 3; say i; numeric digits 9; end` — oracle `1.23 / 2.23 / 3.23`, ours `1.23 / 2.23456 / 3.23456`.
* `numeric digits 3; do i = 1 to 9 by 1.2345; say i; numeric digits 9; end` — oracle steps by `1.23` (`2.23, 3.46, ...`), ours by the exact `1.2345` (`2.2345, 3.4690, ...`).
* The same shape applies to `to` (stored exact vs `3.00`), observable when digits increases mid-loop.

This is exactly D15's "rendering/value fixed at creation" applied to the loop header: the oracle *creates* the header values under entry digits; ours defers, so the value observed later is derived under the wrong digits.
Task 11's tests could not see it because none of them change `NUMERIC DIGITS` inside a controlled loop with a wide header literal.

### F2 (Important): NUMERIC DIGITS/FUZZ/FORM with an expression never traces `>K>`

Oracle: `RexxInstructionNumeric::execute` calls `traceKeywordResult(DIGITS, result)` at `NumericInstruction.cpp:98`, `FUZZ` at `:135`, `FORM` at `:174` (verified by containment) whenever an expression is present, including a literal `9`.
Ours: `exec_numeric` (`rust/crates/rexx-exec/src/run.rs:2593`) never calls `trace_keyword` at all.

Measured: `trace r; numeric digits 9; ...` — oracle emits `>K>   "DIGITS" => "9"` after the clause echo; ours emits nothing.

Task 13's report (task-13-report.md, the `>K>` bullet) enumerated `traceKeywordResult` callers as "DO's control-clause components and SELECT CASE's CASE -- both 4a-owned constructs", which missed `NumericInstruction.cpp` — a 4a-owned construct too.
The per-task review checked ours against that enumeration, so it structurally could not catch the omission.
Note the oracle traces the **raw** expression result before validation, so on the error path (`numeric digits 'x'`) the `>K>` line still appears before error 26; a fix must trace before validating, as `setup_controlled` already does for `TO`/`BY`/`FOR`.

### F3 (Important): repeat count and FOR count validated under fixed digits 9, not current DIGITS

`whole_nonneg` (`rust/crates/rexx-exec/src/run.rs`, used by the bare `DO n` count and `FOR`) converts via `rexx_num::ARGUMENT_DIGITS` (9), with a doc comment reasoning "a loop bound is no more digits-limited than EXIT's own result is".
That reasoning is wrong by measurement: the oracle validates the count under the **current** `NUMERIC DIGITS` (`ForLoop::setup`, `DoBlockComponents.cpp` ~:80-100, verified by containment: prefix-`+` rounding under current settings, then `requestNumber(count, number_digits())`; a literal-integer fast path applies only when `context->digits() >= Numerics::DEFAULT_DIGITS`).

Measured divergences:

* `numeric digits 3; do 12345; ...; end` — oracle: error 26.2, rc 230; ours: runs 12345 iterations, rc 0.
* `do i = 1 to 99999 for 12345` under digits 3 — oracle: error 26.3, rc 230; ours: runs.
* Converse edge: under `digits 20`, a count wider than 9 digits would be *rejected* by ours (whole_value(9) fails) where the oracle runs it.

`EXIT 12345` under digits 3 matched (rc 57 both), so `ARGUMENT_DIGITS` is right for `exit_code_for` and wrong for the loop counts; the two are genuinely different rules.

### F4 (Important): reading an unset bare stem must auto-vivify the stem object; aliasing an untouched stem diverges

`stem.rs`'s module doc claims a bare stem read is "not a new operation ... no `Body::Stem` is ever allocated for that case (measured: `say never_touched.5` ...)".
The two measurements behind that claim (`say never_touched.5`, `drop x.`) are both rendering-only; neither *aliases* the read's result, which is where the object identity becomes observable.

Measured divergence (probe s2/s3 in scratchpad):

```text
b. = a.        /* a. never touched */
a.1 = 5
say b.1        oracle: 5      ours: A.
say b.7        oracle: A.7    ours: A.
```

Oracle mechanism, verified by containment: `VariableDictionary::getStemVariable` (`VariableDictionary.hpp:178-183`) calls `createStemVariable` (`VariableDictionary.cpp:473`) on any miss, reads included, so a bare stem read always yields a real `StemClass` object; `b. = a.` then shares it, and later writes through `a.` are visible through `b.`.
Ours: `read_by_name` returns a derived-name `Body::Text` for an unset stem slot, so `stem_assign` takes its wrap-as-default branch, and `b.` ends up with default `"A."` instead of sharing `a.`'s object.

Fix shape: an unset stem-named slot read (at least when the read's result can escape, which 4a cannot distinguish, so: always) should allocate `Body::Stem { name, default: None, tails: {} }`, bind it to the slot, and return it. Rendering stays identical (`to_text` on a defaultless stem gives its name), so the existing tests keep passing.
This also interacts with F1's sibling in 4b (aliasing is 4b's stated scope), but the divergence above is reachable in pure 4a programs today.

### F5 (Critical): arithmetic or comparison on a bare stem value panics the interpreter

`to_number` (`rust/crates/rexx-exec/src/value.rs:218-220`) declares `unreachable!("the value model only creates Text and Num")` for any other body, but `to_text` directly above it already handles `Body::Stem` — the value model does create stems, and a stem value flows into every `to_number` caller.

Measured:

```text
a. = 5
say a. + 1     oracle: 6, rc 0
               ours: panic, "entered unreachable code: ... got Stem { ... }", value.rs:219, rc 101
```

Every `to_number` path is affected once the operand is a stem read: arithmetic, comparison (`eval_compare` calls `to_number(..).ok()`), `exit a.`, `do a.`, `numeric digits a.`, `do i = a. to ...`.
This is a process abort on a two-line pure-4a program, exactly the failure class the crate's own "fail loudly, never abort" rule exists to prevent.
The corpus never does arithmetic on a bare stem, which is why 29/29 and 824 tests could not see it.

Fix shape: `to_number` on `Body::Stem` should convert through the same redirect `to_text` uses (default if `Some`, else the name, then parse), mirroring the oracle where a stem forwards to its value. `say a. + 1` with `a. = 5` must give 6; `say q. + 1` on an untouched stem must give whatever the oracle gives for the nonnumeric name (expected 41.1).

### F6 (Minor): `Heap::alloc` keeps a friendly name that bypasses the stress hook

The `alloc_with` -> `alloc_with_uncollected` rename (`rust/crates/rexx-core/src/heap.rs:253`) makes a bypassing call announce itself, but `Heap::alloc` (`heap.rs:232`) still exists, is `pub`, forwards to `alloc_with_uncollected`, and carries no warning in its name.
A future `rexx-exec` site calling `self.heap.alloc(body)` would bypass the stress mode as quietly as the pre-rename sites did.
Today only `rexx-core`'s own tests and benches call it (verified: no `heap.alloc(` in `rexx-exec`), so this is a residual gap in the rename's stated goal, not a live defect.

### F7 (Minor): a WHILE/UNTIL loop's per-pass temporaries accumulate until the loop exits

`eval_condition` pushes its condition result with `push_temp` on every pass, and the enclosing watermark is the `DO` instruction's own `step_in_temps_frame` frame, which pops only when the whole loop finishes.
A long-running `do while ...` therefore accumulates one temp (and keeps one small heap object rooted) per pass for the loop's lifetime.
Safe direction (over-rooting, never under-rooting), and invisible at corpus scale, but it is unbounded memory growth proportional to iteration count and will matter for a permanent collector; a per-pass watermark around the condition evaluation would fix it.

## Verification record

* Off path of the stress mode: `git diff` of `heap.rs` over the branch shows the only non-comment changes are the rename, the `collections` counter, and test updates; `Interp::alloc_with` off-path is one bool read plus the identical call. Confirmed genuinely unchanged.
* Rename visibility: no `rexx-exec` call site reaches `alloc_with_uncollected` except `Interp::alloc_with` itself; no `heap.alloc(` in `rexx-exec` (but see F6).
* `pop_frame` watermark healing: `step_in_temps_frame` pops unconditionally with an outer watermark (`run.rs:1254-1256`); `pop_frame` truncates; the six `?`-escaping frames are healed as documented. `Body::trace` covers `Stem` (default plus live tails, tombstones skipped) so the collector sees stem contents.
* Citations added on this branch in this slice (7, all in `trace.rs`) verified by containment, not by the named line: `RexxActivation.cpp:3565` (format constants), `TraceSetting.cpp:49-54` (flag sets), `TraceSetting.cpp:135` (`parseTraceSetting`), `RexxActivation.hpp:341/353` (`traceVariable`/`tracePrefix`), `ExpressionVariable.cpp:299` (`traceAssignment`), `RexxActivation.cpp:4791` (`evaluateLocalCompoundVariable`). Also verified: `RexxMemory.cpp:426-433` (heap.rs weak-ref ordering), `NumberStringClass.cpp:3194` (compare.rs), `DoBlock.cpp:182`, `SelectInstruction.cpp:372`, `IfInstruction.cpp:140` (run.rs). All check out; no drift found in this slice.
* D15 differential probes that MATCHED the oracle: rendering fixed at creation (`digits 9; y=1/3; digits 3; say y; say y+0`), SmallInt admissibility under `digits 1` (`15+0` is `2E+1`, `x+6` is `3E+1`), FORM captured at creation (`10E+9` stays), fuzz comparison battery, controlled loop stepping without mid-loop digits change, `to 2.9996`/`by 1.2345` without digits change (masked, see F1).
* D15a stem probes that MATCHED: tombstone vs default, alias tail-write visibility, whole-stem replace leaves aliases, aliased tombstone deriving the *object's* name (`d.1 -> C.1`, `m.1 -> N.1`, `k.9 -> G.9`), verbatim case-sensitive tail keys, multi-level keys. The `stem_get` object-name reasoning (the known-equivalent-mutant site) is empirically right end to end; the gap was one step earlier, in what an unset stem read *returns* (F4).
* `rexx-num` branch changes (`compare_bytes`/`compare_decoded`, `sub_code` made pub) reviewed: one comparison rule with three entry points, strict family still byte-only on original spelling; `string_order` byte-slice conversion is faithful to the C++.
* `rexx-parse` branch changes reviewed: `MAX_EXPR_DEPTH` (shared budget, const-asserted below the measured native cliff, prefix-chain gap disclosed), iterative `Drop for Expr` and iterative `visit_expr` (both sound; `for_each_child`/`for_each_child_mut` lockstep), `SymbolId::index` density contract, `Program::main` as `CodeBody`, `Fragment::body`. No defects found.
* `rexx-extract` assertion extraction and `plan.rs`/`activation.rs`/`error.rs` read in full; no defects found beyond what is filed. `error.rs`'s substitution scanner correctly refuses re-substitution.

## Not reached

* `rexx-parse/tests/*` (deep.rs, program.rs, sourceline_oracle fixtures, gate_walk, tiling) and `rexx-parse/examples/depth_probe.rs`: skimmed for shape only, not line-audited.
* `rexx-core/tests/collect.rs` new tests: skimmed (substantive, assert sweep counts both directions); not line-audited.
* `rexx-num/tests/*` additions: not read.
* The gate instruments' negative controls (Task 16) beyond the off-path question: left to the gate reviewer.
* Trace output interleaving: our runner emits stdout and the trace sink separately, so oracle-interleaved order differs under `2>&1`; D17 records stdout/stderr relative order as unobservable, so not filed. Noting it here because F2's probe surfaced it.
* `DO OVER` / `SELECT CASE` / `>C>`-`>V>` intermediate-trace call-site correctness in `run.rs`/`eval.rs`: the exec slice's files; only touched here where a probe crossed them (F1, F2, F3, F5 all live in exec files but are value-model rules, filed here).

STATUS: DONE (findings F1-F7: 1 Critical, 4 Important, 2 Minor)
