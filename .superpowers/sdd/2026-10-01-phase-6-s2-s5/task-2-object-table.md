# Task 2 table: base/class/Object, start tests

Produced at the Task 2 fix round 1 tree (before its commit) by `REXX_GROUP_TABLE=<file> REXX_OBJECT_TABLE=<file> REXX_CORPUS_GATE=1 memcap 8G cargo test -p rexx-exec --test concurrency_tests the_outcome_table -- --nocapture` (concurrency_tests.rs, module group_runs). The run takes the tests whose name holds START: `TEST_UNINIT` and `TEST_UNINIT_CLASS` each reach 3.6 GB in this crate (measured, `/usr/bin/time` over a single-test run), and two at once outgrow the gate's 8 GB cap. Raw output in `task-2-object-table.raw.md`. The S1 pinning report lists 14 of these as `pass` with a `MessageResult` arrival; all pass again.

Summary: pass 40


Full table:

| test | outcome | refusal |
|---|---|---|
| TESTSTART01 | pass |  |
| TESTSTARTWITH01 | pass |  |
| TEST_STARTWITH_NOT_STRING | pass |  |
| TEST_STARTWITH_NO_METHOD | pass |  |
| TEST_STARTWITH_NO_NAME | pass |  |
| TEST_STARTWITH_NO_NAME2 | pass |  |
| TEST_STARTWITH_OVERRIDE | pass |  |
| TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | pass |  |
| TEST_STARTWITH_OVERRIDE_CONTEXT | pass |  |
| TEST_STARTWITH_OVERRIDE_EMPTY_ARRAY | pass |  |
| TEST_STARTWITH_OVERRIDE_EXTRA_STUFF | pass |  |
| TEST_STARTWITH_OVERRIDE_FROM_NONSELF | pass |  |
| TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass |  |
| TEST_STARTWITH_OVERRIDE_MISSING_NAME | pass |  |
| TEST_STARTWITH_OVERRIDE_MISSING_SCOPE | pass |  |
| TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE | pass |  |
| TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE2 | pass |  |
| TEST_STARTWITH_OVERRIDE_NON_STRING_NAME | pass |  |
| TEST_STARTWITH_OVERRIDE_NOT_FOUND | pass |  |
| TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | pass |  |
| TEST_STARTWITH_OVERRIDE_NO_METHOD | pass |  |
| TEST_START_NOT_STRING | pass |  |
| TEST_START_NO_METHOD | pass |  |
| TEST_START_NO_NAME | pass |  |
| TEST_START_NO_NAME2 | pass |  |
| TEST_START_OVERRIDE | pass |  |
| TEST_START_OVERRIDE_AMONG_MIXINCLASSES | pass |  |
| TEST_START_OVERRIDE_CONTEXT | pass |  |
| TEST_START_OVERRIDE_EMPTY_ARRAY | pass |  |
| TEST_START_OVERRIDE_EXTRA_STUFF | pass |  |
| TEST_START_OVERRIDE_FROM_NONSELF | pass |  |
| TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass |  |
| TEST_START_OVERRIDE_MISSING_NAME | pass |  |
| TEST_START_OVERRIDE_MISSING_SCOPE | pass |  |
| TEST_START_OVERRIDE_NON_CLASS_SCOPE | pass |  |
| TEST_START_OVERRIDE_NON_CLASS_SCOPE2 | pass |  |
| TEST_START_OVERRIDE_NON_STRING_NAME | pass |  |
| TEST_START_OVERRIDE_NOT_FOUND | pass |  |
| TEST_START_OVERRIDE_NOT_NON_SCOPE | pass |  |
| TEST_START_OVERRIDE_NO_METHOD | pass |  |
