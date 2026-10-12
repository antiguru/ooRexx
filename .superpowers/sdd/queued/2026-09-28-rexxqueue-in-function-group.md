# FUNCTION's TEST_REXXQUEUE needs RexxQueue (owner: Phase 10)

`.rexxqueue~create('ADDRESSWITH')` is rc 120, `the LIBRARY REXX entry point
"rexx_create_queue" is not implemented (Phase 10)`. Recorded in phase-4-exclusions.txt;
remove it from `RECORDED` in `api_group_tests.rs` when the queues land.
