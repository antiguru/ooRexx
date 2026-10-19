# Task 6.9 report: conditions

Clusters `raise-propagate-from-method`, `raise-result-entry`, `call-on-trap-argument`,
`signal-trap-missing-label` and `reply-thread-error-report-line-ends`, plus the items earlier tasks
handed this instance. Every cluster test passes. In the whole suite, only those tests' rows change.
Status: DONE_WITH_CONCERNS (see Concerns: files outside the clusters' list were touched).

## Commits

* `fe1e7d9e0` CALL ON argument, SIGNAL ON missing label, RAISE RESULT.
* `312d2c64c` an error report line keeps its line ends.
* `4fca20885` condition object fields.
* `94b09da09` RAISE senders, NOMETHOD propagate, MESSAGE without a sub.
* `688f592ed` the five witnesses, their `sourceline_oracle/` files and `phase-9.txt` lines.
* `2402bc7c7` records, `suite-9d-set.txt`, the re-derived `refusal-sites.tsv` (line numbers only),
  and the queued item's closing line.
* The commit after `2402bc7c7` holds this report.

## What changed, per cluster

### call-on-trap-argument

`RexxInstructionCallOn::trap` passes the condition object as the one argument to a label, a builtin
or an external routine. `deliver_one_pending_trap` passed none. It now calls the new
`resolve_and_run_call_over` (`run/call.rs`), which resolves the name as `resolve_and_run_call` did
and runs it over the evaluated arguments. A builtin target runs over the argument stack. Every other
target goes through `begin_evaluated_call`, which is `begin_invoke_call`'s code after the argument
loop, moved out unchanged. `resolve_and_run_call` had no other caller and is gone.

### signal-trap-missing-label

`RexxInstructionSignalOn::trap` turns the trap off and then raises 16.1. `offer_to_trap` now removes
the matched trap (its own condition, or `ANY`) and offers the 16.1 to the same activation's table.
Untrapped, it is reported at the raising clause as before.

### raise-result-entry

* `RAISE SYNTAX ... RETURN|EXIT expr` sets `pending_result`, so the condition object carries
  `RESULT` (`Activity::raiseException`).
* `RAISE PROPAGATE RETURN|EXIT expr` evaluates the value, traces its `>K>` line, and puts it in the
  active object's `RESULT` (`RexxActivation::raise`). This closes the write-only
  `query position write line` consequence. `t6.5/r1/probes/write_only_query_result/wo.rex` now
  matches the recorded oracle output exactly.
* **The sender a RETURN tail offers to** (`raise_sender`, `Sender`), measured and witnessed. The
  oracle's `raise` (RETURN) offers the condition to the raising level's sender, and `raiseExit` (no
  tail, or EXIT) offers it to the caller of the first non-internal activation. From an `INTERPRET`
  fragment, the sender is the activation that runs it. Measured: `interpret "raise user u return 1"`
  in a routine runs that routine's own `CALL ON` handler, and `interpret "raise propagate return
  'p1'"` of a SYNTAX condition is trapped by the routine's next `SIGNAL ON SYNTAX`. The crate used the
  EXIT rule for every propagate and offered fragment raises to the caller. Plain RAISE,
  non-SYNTAX propagate and SYNTAX propagate now use this one rule.
* After a `REPLY`, a non-SYNTAX `RAISE` that returns a value is 98.936 at the RAISE (`returnFrom`).
  The same goes for `raise user x return 5` in a replied method (Task 6.4 re-review).

### reply-thread-error-report-line-ends

`Activity::display` writes each traceback echo and each `Error` line as one trace-output line.
`Raised::report_lines` builds the report as those lines, and `report` joins them. The REPLY-thread
report (`scheduler.rs`) and both main-program reports (`lib.rs`) deliver them through the new
`write_trace_report_lines` (`environment/route.rs`). `deliver_trace_line_as` dropped every trailing
line end. It now drops only the one its formatter appended. Measured: a clause echo ending in `'0a'x`
is delivered with it, and so is a `13.1` message holding one (the INTERPRET witness).

### raise-propagate-from-method

Task 6.4 closed it. This task adds the INTERPRET and REPLY witnesses (`raise_result_and_sender.rex`
and `condition_object_origin.rex`).

## Items handed to this instance

* **CONDITION_SIGNAL's missing ADDITIONAL.** NOMETHOD (`dispatch.rs` `nomethod`) and NOSTRING
  (`dispatch/reqstr.rs`) set `pending_additional` to the receiver when a SIGNAL trap takes them
  (`reportNomethod`, `ObjectClass.cpp:1247`).
* **Condition-object fields from `t6.4/probes/d`.**
  * The interpreter's library programs are the `REXX` package in a condition object
    (`reported_package`). Before, they named the main program's path. Measured on the oracle:
    `.Validate~number` 88.902 names `REXX` as both PACKAGE and PROGRAM.
  * A `LIBRARY REXX` method (`blame_external_method`) is a REXX-package level with no line, which
    removes POSITION for NOTREADY and for the boundary's own 88.922.
  * A NOTREADY that a CALL ON trap queues inside a `LIBRARY REXX` method is recorded in
    `Activity::native_frames_owed`. `settle_native_frames_owed` (`dispatch.rs`, cold) then leads the
    object with that method's frame once it returns: STACKFRAMES 2 against 1 before, plus the
    traceback line, PACKAGE and PROGRAM `REXX`, and no POSITION. The only hot-path cost is an
    `is_empty` check on the `ExternalBody::Implemented` arm.
  * RAISE PROPAGATE of a condition other than SYNTAX keeps the object's POSITION, PROGRAM and
    PACKAGE (`trap_propagated_condition_object`). The oracle's `raise` does not call
    `reraiseException` for it.
  * A live frame of a library program shows the `(no source available)` traceback line, which is
    the rule `sourceless_site` already applied to failure sites.
  * A SYNTAX object whose code has no sub carries MESSAGE `.nil`.
* **Propagated NOMETHOD traceback (6.4 M7).** `exec_raise_propagate` chose the SYNTAX path by
  `reportable()` (a nonzero number), and a trapped NOMETHOD carries 97.1. It now chooses by
  condition name. Untrapped, the report is 97.1 at the `raise propagate` line. The ADDITIONAL test
  reads the entry including `.nil` (`condition_entry_held`). `reportable` had no other caller and is
  removed.
* **Queued item** `2026-09-28-condition-object-divergences-found-in-fix-round-a.md` (owner
  `call-on-trap-argument`): its `callarg` and `coorder` probes are identical to the oracle on all
  three descriptors (`t6.9/queued/`). Its first line now reads CLOSED.

## Witnesses, red then green

Red is the parent binary `31b57465c`. Green is `94b09da09`. `t6.9/witnesses/run.sh` runs each
witness from a fresh directory and writes `summary.tsv`:

| program | red vs oracle | green vs oracle |
|---|---|---|
| `condition_call_on_argument` | stdout stderr rc differ | identical |
| `signal_on_missing_label` | stdout stderr differ | identical |
| `raise_result_and_sender` | stdout stderr rc differ | identical |
| `error_report_line_ends` | stdout differs | identical |
| `condition_object_origin` | stdout stderr differ | identical |

What each green stdout prints (read):
* `condition_call_on_argument`: `arg() 1`, a Directory equivalent to but not `==` the CONDITION('O')
  copy, for USER, ERROR and NOTREADY to a label; `reverse ran` for a builtin; and a `::routine`
  handler for USER, ERROR through `CALL ON ANY`, and HALT.
* `signal_on_missing_label`: 16.1 trapped at the raising line by SIGNAL ON SYNTAX, the NOVALUE trap
  OFF afterwards, 16.1 taken by ANY, a NOTREADY inside a native method in an internal call, and an
  untrapped 16.1 report at the end.
* `raise_result_and_sender` (REPLY, INTERPRET and propagated paths):
  * RESULT for SYNTAX RETURN, EXIT and none;
  * propagated SYNTAX with and without a tail through INTERPRET;
  * USER propagate RETURN and EXIT under CALL ON and SIGNAL ON, and untrapped;
  * the routine's own handler for an interpreted `return` raise, the caller's handler for `exit`,
    and an internal call's caller;
  * propagate RETURN from INTERPRET answering its value;
  * SYNTAX propagate RETURN trapped by the routine itself, through INTERPRET and through an internal
    call;
  * a replied method raising with no value (quiet) and with a value (98.936 on stderr).
* `error_report_line_ends` (REPLY and INTERPRET paths): three REPLY-thread trace lines with the
  insert's `0A` inside line 3, then the INTERPRET 13.1 report with its echo ending `0A` and the 13.1
  message holding `0A`.
* `condition_object_origin` (propagated path):
  * ADDITIONAL for NOMETHOD on a string and on `.nil`, and for NOSTRING;
  * NOTREADY under CALL ON from LINEIN, CHARS and LINES, each with the method frame;
  * NOTREADY under SIGNAL ON, direct and propagated through `Stream~command`;
  * library 88.902 and 88.922 objects;
  * MESSAGE `.nil` for `raise syntax 93` and for LINEIN's 93;
  * a propagate through an internal call;
  * a propagated NOMETHOD left untrapped, reported at the propagate.

The concurrent witnesses `raise_result_and_sender` and `error_report_line_ends` were run 8 times
each on the oracle and on the task. Stdout, stderr and rc were the same every time on both sides
(`t6.9/witnesses/stability.txt`).

## The tests

These leave `suite-9d-set.txt`, and each passes in the whole-suite run:
* `base/bif/CONDITION` `TEST_CONDITION_CALL`, `TEST_CONDITION_CALL_BIF`,
  `TEST_CONDITION_CALL_EXTERNAL` and `TEST_CONDITION_SIGNAL`;
* `base/keyword/RAISE` `TEST_RAISE_INSERT_CRLF`, `TEST_RAISE_PROPAGATE`,
  `TEST_RAISE_SYNTAX_EXIT_02` and `TEST_RAISE_SYNTAX_RETURN_02`;
* `base/keyword/SIGNAL TEST_BUG_1877`.

No exclusion row is touched.

## Changed rows

`t6.9/suite/changed-rows.txt` (its first line is the command) diffs against `t6.8/r2/suite/tests.tsv`.
Exactly the tests listed above change, each from `passed failed` to `passed passed`. No other row
changed, and the `SysFileTree TEST_TREE_DIRECTORY` flake did not show. Suite: 402 groups, 13 red.

## Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t69-target`, `CARGO_INCREMENTAL=0`, `-j 4`. Each exit
status was read unpiped. They ran at `688f592ed`, and the code is unchanged from `94b09da09`:
* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed.
* `memcap 16G cargo test --release -p rexx-exec --test refusal_sites --test owners --test
  closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_dispositions
  --test coverage --test rexxc --test licensed_divergences --test stream_handle --test loud`:
  * the first run was exit 101: `refusal_sites` saw `error.rs` line numbers shift;
  * `REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites` re-derived the table. With line numbers
    stripped, the old and new tables are identical (sorted diff empty);
  * the rerun was exit 0.
* `memcap 16G cargo test --release -p rexx-parse`: exit 0 (includes `sourceline_oracle`).
* `memcap 16G cargo test --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -p rexx-exec --test corpus`: exit 0, 1101 of
  1101 matching.
* `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/t6.9/suite/table.tsv
  memcap 8G cargo test --release -p rexx-exec --test suite l3_core_suite`: exit 101, as expected while
  the phase is open. 402 groups, 13 red, 255.9 s.

## Perf

`t6.9/callgrind.txt` comes from `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn
sayloop"`, with pin, parent `f48b4d866` (`cg-9d/t6.8-r2/rexx-head`) and head `94b09da09` in one run,
exit 0. The ledger's previous figures are `t6.8/r2/callgrind.txt` for interpret and churn, and
`t6.8/r1/callgrind.txt` for the rest.

| program | head vs pin | previous vs pin | own delta (ledger) | vs parent, same run |
|---|---|---|---|---|
| startup | +0.0449 | +0.0433 | +0.0016 | +0.0032 |
| interpret | -2.1638 | -2.1653 | +0.0015 | +0.0015 |
| parse | -0.0858 | -0.0860 | +0.0002 | +0.0003 |
| churn | +0.1149 | +0.1143 | +0.0006 | +0.0006 |
| sayloop | +0.0483 | +0.0473 | +0.0010 | +0.0019 |

Every own delta is under +0.05%, and every figure against the pin is under +0.5%. The
libc-subtracted figures move by at most 0.0002. The new error paths (`settle_native_frames_owed`,
`raise_result`, the propagate helpers) are cold, or run only on RAISE and trap delivery.

## Concerns

1. **Files outside the clusters' list.** The line-end fix needed `environment/route.rs`
   (`write_trace_report_lines`, and dropping one line end rather than all), `scheduler.rs` and
   `lib.rs` (one call each). The condition-object items needed `dispatch.rs`, `dispatch/context.rs`,
   `dispatch/reqstr.rs` and `activity.rs`. Each change is the smallest one that reaches the measured
   cause, and the suite and corpus show no other changed result. The controller should confirm this
   breadth was acceptable.
2. **The RETURN-tail sender rule is wider than the brief.** It changes non-propagate `RAISE ...
   RETURN` inside INTERPRET as well (the routine's own trap now takes it). That is oracle-measured,
   witnessed, and changes no suite row.
3. `display_condition` (native API `DisplayCondition`, `dispatch/library/surface.rs`) still splits its
   report on line ends through `write_trace_report`. No witness reaches it, and it was not changed.
