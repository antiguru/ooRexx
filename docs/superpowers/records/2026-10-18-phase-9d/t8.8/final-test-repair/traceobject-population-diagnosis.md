# TraceObject fixed outcome population diagnosis

The failure is a stale expected passing set, not evidence of a fixture population mismatch. At frozen HEAD `6b6f3fb460049c6ef63f565ceb78e11b5582f165`, `concurrency_tests.rs:2551-2555` still lists three passes from October 2, while the retained fresh per-test comparison reports all ten current tests as passing. The target was still running when this diagnosis was requested, so this report makes no claim about its eventual overall status. Its completed TraceObject branch is recorded in `/tmp/codex-p9d-t88-final-gates-resume/debug-154-concurrency_tests.stderr`, including the exact ten-name left side and three-name right side at the assertion at source line 2576.

## Old and current provenance

Blame attributes the entire constant and its test to `826b07b45a49683e368aba69b00889b32684c9a4`, “TraceObject activity fields; program end waits; UNINIT at activity end (Task 9)”, dated 2026-10-02. The original `.superpowers/sdd/2026-10-01-phase-6-s2-s5/task-9-report.md:85-94` explicitly says that all ten tests were discovered, three passed, three differed and four were refused. That commit also corrected the source-name scanner to skip RESOURCE bodies, which previously stopped after three discovered tests, but the constant added in the same commit records three passing outcomes out of ten, not three total tests. Therefore the seven extra names in the current passing vector are existing tests whose outcomes improved.

| Test | Original Task 9 disposition | Current fixture method line |
| --- | --- | --- |
| `TEST_CALLER_STACK_FRAME` | Differed on collector behavior | 604 |
| `TEST_CALLER_STACK_FRAME_REPLY_START` | Refused at GUARD WHEN | 668 |
| `TEST_OBJECT_AND_SCOPE` | Refused on executable identity | 496 |
| `TEST_RECEIVER` | Differed on collector behavior | 84 |
| `TEST_SETMAKESTRING_WITH_METHOD_OBJECT` | Passed | 486 |
| `TEST_TRACEOBJECT_COLLECTOR` | Refused at DO COUNTER OVER | 211 |
| `TEST_TRACEOBJECT_COLLECTOR_AND_NOTIFY_CLASS_ATTRIBUTES` | Differed on collector behavior | 761 |
| `TEST_TRACEOBJECT_OPTION` | Passed | 70 |
| `TEST_TRACEOBJECT_OPTION_INVALID` | Passed | 78 |
| `TEST_VARIABLE` | Refused at DO COUNTER OVER | 131 |

Task 6.10 implemented every-line TraceObjects, option P, NOTIFY and VARIABLE in `c239cd23f66953bf30eb68b16a12d336d73b13ee`, with related trace-delivery fixes described in its report. Its retained `docs/superpowers/records/2026-10-18-phase-9d/t6.10/suite/tests.tsv:22878-22887` already shows seven passes, leaving only CALLER_STACK_FRAME_REPLY_START, OBJECT_AND_SCOPE and TRACEOBJECT_COLLECTOR failing on package names. Task 6.11's `58902b630b3bb23814a3c628484b2759744d5fce` corrects dynamic package naming, and its report at lines 21-30 explicitly closes those three cases. The suite evidence committed in `3d5768bf74f8e6f48ec53a744d3ad6598cf5ea8e`, `t6.11/suite/tests.tsv:22878-22887`, records all ten passing on both interpreters.

The current fixture is `ootest/ooRexx/base/keyword/TRACE_TraceObject.testGroup`, outside the tracked Git source tree in the existing SVN checkout. Read-only SVN info reports revision 13178, last changed revision 13010 on 2025-08-25, and text last updated on 2026-07-27. Its 41,788 bytes are exactly identical to `.svn/pristine/5a/5a1f4c21ab6e271b9617eec15b0754fb10066a7a.svn-base`, with SHA1 `5a1f4c21ab6e271b9617eec15b0754fb10066a7a` and SHA256 `98e4999d07fca459f1d2a13793a5e5d15b5b2dba82460c615e410d0fdcd0d9b3`. This unchanged pristine fixture, the original ten-name account and the later ten-pass records give no basis for reverting the fixture or masking seven rows.

The final attempt-1 suite independently corroborates the improvement before the fresh debug target. Its `suite/tests.tsv:22878-22887` records all ten as passed/passed, `suite/table.tsv:649-650` records ten tests run and passed per interpreter, and `suite/verdicts.tsv:326` records the group green/equal. The corresponding raw descriptors are indexed by `suite-diagnostics-manifest.tsv:1855-1858,1871-1874` beneath `t8.8/final-gates/attempt-1/`. The current per-test run supplies the additional evidence that each test also passes when run separately through the concurrency harness.

## Exact invariant and narrow test-only proposal

`outcome_table` at `concurrency_tests.rs:1908` obtains the sorted distinct source test names, runs each selected name on both interpreters and requires the returned row count to match the selected population. `support/group_runner.rs:919-936` derives names from test-class methods, and its `run_tests` at line 1167 verifies that the oracle finishes and runs a nonzero test count. `compare` at line 1374 classifies a `rexx-exec:` refusal before any agreement check, while `passes`/`agree` at lines 399-408 require status 0, a passing summary, identical stderr/status and stdout equality under the existing mask. Thus each newly observed `Outcome::Pass` already excludes refusal, failure on both sides, unrun rows and differing descriptors.

Change only `rust/crates/rexx-exec/tests/concurrency_tests.rs` for this finding. Update the stale comment to describe the exact all-passing population, replace `TRACE_OBJECT_PASSING` with the ten sorted names below, and retain the existing equality on the passing-name vector. Add `assert_eq!(results.len(), TRACE_OBJECT_PASSING.len())` before filtering, so an additional refused, differing or unrun source row cannot silently disappear from the comparison. Together the exact passing list and exact total count require precisely these ten rows, all passing, without adding a new Rust test name or weakening any classifier.

```rust
    const TRACE_OBJECT_PASSING: &[&str] = &[
        "TEST_CALLER_STACK_FRAME",
        "TEST_CALLER_STACK_FRAME_REPLY_START",
        "TEST_OBJECT_AND_SCOPE",
        "TEST_RECEIVER",
        "TEST_SETMAKESTRING_WITH_METHOD_OBJECT",
        "TEST_TRACEOBJECT_COLLECTOR",
        "TEST_TRACEOBJECT_COLLECTOR_AND_NOTIFY_CLASS_ATTRIBUTES",
        "TEST_TRACEOBJECT_OPTION",
        "TEST_TRACEOBJECT_OPTION_INVALID",
        "TEST_VARIABLE",
    ];
```

Do not change `SwitchMode::None`, the all-source selection, descriptor masking, rxapi exclusions, refusal recognition or any other group's accepted outcomes. Keep the existing negative runner control `the_runner_passes_a_passing_group_and_reports_an_altered_test_as_differing` at line 1848 intact, and do not replace exact population equality with a subset or minimum-count assertion. The older whole-group normal/every/derived allowance keys at lines 2838-2863 are separate historical rows and are not licensed for modification by this single fresh failure. The new total-count assertion closes the relevant negative boundary locally: removing an expected pass or adding an unexpected nonpass must remain red.

## Freeze and limitations

Only read-only source/history inspection, local SVN metadata/pristine comparison and retained output inspection were performed. No oracle or interpreter run, Cargo action, source/index write or frozen-input mutation occurred, and only this scratch report was written. The earliest record proving all ten pass is the cited Task 6.11 suite, while the exact isolated transition commit for every one of the seven names was not re-executed or inferred beyond those retained reports. After the controller releases the freeze, the supported repair needs focused execution of this exact outcome-table test with the gate enabled, plus formatting and affected-target lint; this diagnosis does not preempt remaining independent gate results.
