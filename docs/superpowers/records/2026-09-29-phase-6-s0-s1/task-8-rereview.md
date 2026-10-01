# Task 8 re-review of fix round 1 (b558488a1..4b0128186)

Verdict: Approved. All four findings addressed, no new problem found.

## I1 Addressed
`call_condition_three_deep.rex:39` now ends `return`. Oracle run from a fresh dir (rc 0) prints
`y 14`, `handled USER DONE from deep 38` (CALL ON handler), `deep 2`, `deep 3`, `after trio trio u`,
`two 14` (SELECT calls), `caught 42 SYNTAX 32` plus the empty condition('D') line (SYNTAX trap).
Every path the file claims runs. sourceline_oracle copy carries the same one-line change.

## I2 Addressed
`a = copies('left', 4)`, `copies('ab', 6)`, `copies('rr', i)`. Values held across allocating callees:
`a||i` (17 bytes), `a||'x'`/`a||'z'` (17), the `abababababab` pending argument (12), `4 rrrrrrrr` (10,
held across `rt(0)`); all above INLINE_TEXT 7. Oracle stdout read and matches the fix report; rc 0.
The file is in phase-8.txt, which the collect-stress subset reads.

## I3 Addressed
`pinned!(TreeEval)` wraps the non-leaf `eval_traced_argument` in `begin_invoke_call`,
`invoke_builtin_call` and `arguments_before_failure`; callee entry stays unpinned; nothing made
stackless. Three FRAME_PROBES cases added (`x = f(g())`, `call f g()`, `x = abs(g())`). Built 4b0128186
via git archive with `--features pinning`: `measured::a_park_under_each_frame_kind_records_it`,
`a_park_records_the_pinned_frames_above_it` and the two other park tests pass. The one failing test in
that filter, `pinned_parks_over_the_derived_list`, needs the `ootest` checkout, absent from a git
archive ("cannot read .../ootest/ooRexx"); environmental, unrelated to the change. The false report
sentence was removed per the fix report (the report file is untracked, not checked by me).

## I4 Addressed, behaviour identical
Read against the removed inline code:
- CallExpr: old = node lookup, name, `site_resolution_before_arguments`, `enter_eval_node`,
  `eval_call_resolved` (= `begin_eval_call` + `complete_function`), `depth -= 1` on every outcome.
  New = `begin_call_expr` (same steps, `depth -= 1` unless `Entered`) then `run_activation` +
  `finish_function_op` (`finish_function`, then `depth -= 1`). The depth is released exactly once on
  every path, including begin failure. Same ordering.
- Call: old = `site_resolution_before_arguments`, `resolved_after_arguments`, `invoke_named_call`
  (= base_indent capture, `invoke_call` = `begin_invoke_call` + `complete_call`, `settle_call_result`).
  New = `begin_call_tree` (same resolution, same base_indent capture, `begin_invoke_call` with
  Subroutine/Written) + `complete_subroutine` (`run_activation`, `finish_call`, `settle_call_result`).
  Same call type, entry, and ordering.
So the begin logic now exists once, and the I3 pin in `begin_invoke_call` covers the non-TOP routes too.

## M1
No change, as ruled.

## New problems
None. The perf table shows fibcall/fibfunc unchanged between t8r3 and fix1 and rexxcps -0.13 points;
the over-budget verdict from round 0 is untouched by this round (not re-run, per instruction).

Scratch tree and target dir deleted.
