# Task 3 fix round 3 re-review: 6a3091cc6..43621d78e

Ours: `rexx-run` built in release from `git archive 43621d78e rust interpreter` (and, first, from
`eb9c3c7b8`), own target dir, one `Compiling rexx-exec` line each. Every probe below was re-run on the
`43621d78e` binary; our output is unchanged from `eb9c3c7b8` on all of them except the answers that
`945e31f33` changed (`probes2/`). Base: the same from `6a3091cc6`. Oracle: the standard wrapper. Each probe ran
from its own empty dir through `cmp.sh`. Probes, script and outputs are under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t3rr3/`
(`probes/`, `cmp.sh`, `out/<probe>[-tag]/`).

### Finding Verdicts

- **Important (Finding 2): `is_true_object` was an identity test on the inline '1', not the truth-value
  test Deviation 25 states** -- ADDRESSED.
  - `loop_truth` (`run/loops.rs:2718-2728`) does the ruling's three steps: identity with `LOGICAL_TRUE`
    or `LOGICAL_FALSE`, then the text through `logical_value`, then 34.901. Both callers use it: the BY
    sign check (`:963`) and the TO test (`:2767-2768`).
  - TO, `tt_to2.rex`, `tt_to3.rex`, `true1.rex`: every form of '1' ends the loop with no pass run.
    Covered: `0 + 1`, `1 * 1`, literal `1`, `'1'`, `left('12', 1)`, `'1' || ''`, `d2c(49)`, `1~string`,
    `.true~copy`, `.true~string`.
    - A comparison, `.true` and `abbrev` end it on both sides.
    - `0`, `1 = 2` and `1 - 1` run 3 passes on both sides.
  - BY, `tt_by2.rex`: the same forms of '1' count down (`<<<`) where the oracle sends `>`. `0` and `1 = 2`
    count up on both sides. `by_dir2.rex`: the oracle sends `>` once; ours counts down, sending `<` five
    times, as the row says.
  - 34.901 on `'yes'`, `' 1'`, `''` and `.object~new` (`d_yes`, `d_sp1`, `d_empty`, `d_obj`). The found
    value is `yes`, ` 1`, empty and `an Object`. On a BY's `<` answering `'abc'` (`by_err.rex`), 34.901
    is blamed on the DO line.
  - Report shape against WHILE: line 1 is `Error 34 ... Logical value not 0 or 1.` on both. The traceback
    line is the DO clause on the first pass, as the oracle's WHILE reports `w_yes`, `w_sp1` and `w_obj`.
    On a later pass (`abc_later.rex`) it is the `END` line, the line the oracle's WHILE blames in
    `abc_while.rex`. The condition object (`cd_to.rex`) matches WHILE's (`cd_while.rex`) except code and
    message. Description is empty, ADDITIONAL is `[yes]`, position is the DO line.
  - What is left is a different defect, which `945e31f33` made wider; see New Breakage 1.
- **Minor 1: a TO or BY that converted as a number was a new object at every pass** -- ADDRESSED.
  - Numbers become objects once per loop: `control_number_object` is called from `object_control_state`
    (`run/loops.rs:986-1003`) and from `object_control_from_numbers` (`:2691-2713`). Each pass then sends
    `ctl.to` and `ctl.by` themselves (`:2746`, `:2761`).
  - The line the report says still differs is not a difference. `ident2.rex` compares identity hashes
    with `=`, which is numeric at NUMERIC DIGITS 9. The hashes are 15 digits, so two different objects
    whose first 9 digits agree compare equal. In `ident4.rex`, line 10, the oracle's hashes are
    `-139782418180849` and `-139782418213233`: two objects, both alive.
  - `ident4.rex` is `ident2` with `==`, every TO and BY kept alive in an array, and the hashes printed. Its
    same-or-not column agrees with the oracle on every line, 5 runs of 5.
  - `ident5.rex` is `ident2` with `==` only: identical on all three descriptors, 5 runs of 5.
  - So there is no address-reuse line, and none is licensed or needed. The report's explanation is false;
    see New Breakage 4.

### New Breakage in the Fix Diff

1. **Important: `loop_truth` does not take the answer's string value the way truthValue's
   `requestString` does, and `945e31f33` made an Array answer diverge from WHILE.**
   - `945e31f33` reads the answer with `string_value_text` (`run/loops.rs:2726`). Its doc calls it
     `stringValue()` (`value.rs:376-380`), the default rendering. The oracle's truthValue
     (`classes/ObjectClass.cpp:523-527`) calls `requestString` (`:1235-1260`) instead. For a primitive
     object that is `primitiveMakeString`, which for an Array joins the items with newlines
     (`classes/ArrayClass.cpp:1829`). For a user object it sends REQUEST('STRING').
   - Measured at `43621d78e` (`probes2/`, `D_*` DO TO, `W_*` WHILE, `U_*` UNTIL, one run per engine):

     | answer | oracle WHILE / UNTIL | our WHILE / UNTIL | our DO TO |
     |---|---|---|---|
     | `.array~of(1)` | true / true | true / true | 34.901, found "an Array" |
     | `.array~of(0)` | false / false | false / false | 34.901, found "an Array" |
     | `.array~of(1, 2)` | 34.3 / 34.4, found "1\n2" | the same | 34.901, found "an Array" |
     | `.array~new` | 34.3 / 34.4 | the same | 34.901 |
     | object whose `string` answers `1` | true / true | 34.3 / 34.4, "an U" | 34.901, "an U" |
     | object whose `string` answers `0` | false / false | 34.3 / 34.4, "an U" | 34.901, "an U" |
     | object with no STRING method | 34.3 / 34.4, "a P" | the same | 34.901, "a P" |
     | `.list~of(1)` | 34.3 / 34.4, "a List" | the same | 34.901, "a List" |

   - The Array rows are a regression from `eb9c3c7b8`, where `d_arr.rex` ended the loop on the
     single-item array exactly as both engines' WHILE do. The ruling says the answer is "judged as
     WHILE/UNTIL judge theirs". After `945e31f33`, our DO TO disagrees with our own WHILE on arrays.
   - False sentences this commit added:
     - Deviation 25: "as WHILE does, from its string value as truthValue's requestString takes it".
     - The new row `.array~of(1)  3  34.901, "an Array"` is a correct measurement, but WHILE does not
       give that answer, so the row records a defect as if it were the rule.
     - The commit message: "As truthValue's requestString does, so an array answer is found as 'an
       Array'".
     - The report addendum: `string_value_text`, "the counterpart of truthValue's requestString".
   - A user object whose `string` answers `1` or `0` still raises 34.901 (`D_ustr1`, `D_ustr0`). The
     oracle's WHILE takes the value. Our WHILE has the same gap (Out-of-Scope 1).
   - Fix shape, not checked: read the answer the way this crate's WHILE does, which agrees with the
     oracle's for Arrays. Then let the user-object case follow whatever WHILE is fixed to. Add the
     Array and user-STRING rows to the agreeing witness and the table.
2. **Minor: two doc comments name `LoopState::object_key`, which does not exist.** They are
   `activity.rs:84` and `run/loops.rs:347`. The method is `FlatLoop::object_key` (`run/loops.rs:339`).
   At `:347` the name is an intra-doc link.
3. **Minor: the dead arm of `run_loop_with_header` now holds loop objects and never releases them.**
   - `run_loop_with_header` calls `object_control_state` at `run/loops.rs:1204-1207`, and that call now
     pushes to `loop_objects` (`hold_object_control`, `:1022-1029`).
   - No `release_loop_objects` is reachable on that path. The only calls are at `:1789`, `:1799`, `:1860`,
     `:1875` and `ir/drive.rs:3714`.
   - Task 2's deferred minor records that the arm is dead (only `Simple` reaches it). I did not verify
     that here. If it is reached, the entries leak for the rest of the activity.
   - The arm needs either a release or the `unreachable!` that minor proposes.
4. **Minor (record): the report's account of `ident2` is false.**
   - `task-3-report.md`, Fix round 3, item 2, says the oracle answers `1` because "a freed object's
     address was reused". The addendum's Concerns repeat it. The probe's `=` explains the `1`, as shown
     under Minor 1 above.
   - Deviation 25 also cites WHILE and UNTIL as "(:277-316)" of `DoBlockComponents.cpp`. That range ends
     before UNTIL's `truthValue` call at `:321`.

### Out-of-Scope Observations

1. This crate's WHILE does not send STRING to a user object answer. In `w_ustr1.rex` and `w_ustr0.rex` the
   oracle runs 3 passes and 0 passes; ours raises 34.3, found `an U`. The WHILE code is not in the diff.
2. This crate's WHILE blames a later pass's 34.3 on the DO line. The oracle blames the `END` line
   (`abc_while.rex`). The base `6a3091cc6` binary does the same.
3. In `lkJ.rex`, a trapped SYNTAX raised inside a user `>` during a DO header's first comparison leaves
   the TO object alive. It survives 4 forced collections until another object-controlled loop ends.
   - The same expression outside a loop (`lkJ3.rex`) is freed at the first collection, and the base
     `6a3091cc6` binary behaves the same.
   - It is not `loop_objects`. An instrumented copy (stderr prints in `hold_object_control` and
     `release_loop_objects`, scratch only) shows the hold (3 entries) and the release at the same key.
   - It matches the shape of Task 2's deferred minor (a trapped SYNTAX in a header and its temps).

### Checks run

- Truth table: `tt_to.rex`, `tt_to2.rex`, `tt_to3.rex`, `tt_by.rex`, `tt_by2.rex`, `true1.rex`,
  `by_dir2.rex`, `gt_one.rex`, `gt_one2.rex`. Errors: `d_{yes,sp1,obj,arr,empty,ustr1,ustr0}.rex`
  against `w_*` with the same answers, `abc_later.rex`, `abc_while.rex`, `by_err.rex`, `cd_to.rex`,
  `cd_while.rex`. One run per engine each.
- At `eb9c3c7b8`, `d_arr` and `w_arr` (`.array~of(1)`) agreed with each other on both engines: true.
  At `43621d78e`, the `probes2/` table above.
- At `43621d78e`, these give the same stdout, stderr and rc as at `eb9c3c7b8`: every probe in the
  truth-table and error lines, `st1`, `lk1`, `ident5`, `dead2`, `dead3`, `dead7`, `reg1`, the witness
  and `licwit`. `rp2` ran 3 times, rc 0, with all four UNINITs.
- Release on every exit path, crate only, by UNINIT after `call gc 'force'` (`lk1.rex`). The paths are
  normal end, LEAVE, ITERATE to an outer loop, SIGNAL out, a trapped error in the body, RETURN inside,
  nested loops with an object BY, recursion inside the body, a trapped 34.901, a trapped error in a later
  `>`, and LEAVE to an outer label. Each TO or BY object's UNINIT ran before the next marker. The only
  exception is the header-error case (Out-of-Scope 3).
- Rooting:
  - `st1.rex` covers those paths with `+` answering fresh objects, allocating bodies, and a numeric loop
    switched to an object with heap-sized TO and BY. Plain and `REXX_SWITCH_MODE=every` give the same
    stdout. It differs from the oracle only in Deviation 25's 34.901 line.
  - `st1.rex` and `reg2.rex` under `run_program_collect_every_alloc`, from a throwaway test in the scratch
    copy only (`zz_probe_t3rr3.rs`): stdout, stderr and exit match the plain run, with 497 and 375
    collections.
- REPLY:
  - `rp2.rex` replies inside a DO whose TO and BY are fresh `+` answers. Main forces 5 collections while
    the continuation is parked in the body, then the loop resumes, sending `>` with `R+` and `+` with
    `B+`, and both are released before `main end`.
  - 5 runs each engine, plus 5 under `REXX_SWITCH_MODE=every`, rc 0 throughout. The only difference
    from the oracle is UNINIT order and placement. Under collect-every-alloc: 3 runs, 42 collections
    each, plain and stress agree on rc and stderr.
- Identity: `ident2.rex` (3 runs), `ident3.rex` (5), `ident4.rex` (5), `ident5.rex` (5), `ident.rex`.
- Earlier rounds: `dead2`, `dead3`, `dead7` and `reg1` are identical on all three descriptors.
- Witnesses:
  - `do_object_compare_true.rex` is identical on all three descriptors, and its comment matches its
    output.
  - The `licensed_divergences` row's program (`licwit.rex`) prints `pass` 3 times then `end` on the
    oracle and `end` alone here, both rc 0 with empty stderr, as recorded.
- Deviation 25 text: each table row was measured above. The citations `DoBlock.cpp:213`,
  `DoBlockComponents.cpp:173`, `IntegerClass.cpp:1432` and `StringClass.cpp:645` hold. "No 34.x names a
  DO header" holds against the full subcode list in `rexxmsg.xml`.
- Perf, not re-run: the gate record quotes the callgrind command with `base=$B/base/rexx-run` and the
  pads, at `0c75907d3` from `git archive`, with a `Compiling` line and the binary's sha256. Its figures
  match the report's.
- Report checks: fmt, clippy, `--lib`, `collect_stress`, the gated corpus, `ir_recorded_oracle`,
  `sourceline_oracle`, `licensed_divergences` and the gated `concurrency_tests` are named with results.
  The debug workspace run is stated as not re-run this round. Not re-run here.

### Verdict

**Fix round:** Findings remain open. Both findings under verification are addressed, but New
Breakage 1 is Important:
- `loop_truth` reads `stringValue`, not `requestString`. Since `945e31f33`, an Array answer raises
  34.901 where both engines' WHILE take it as 1 or 0. That breaks the ruling, and Deviation 25, the
  commit message and the report each state it as requestString's behaviour.

Three Minors:
- Doc comments name `LoopState::object_key`, which does not exist.
- The dead `run_loop_with_header` arm holds loop objects without a release.
- The report's address-reuse explanation and Deviation 25's `:277-316` citation are wrong.
