# Task 2 table: base/class/Message

Produced at the Task 2 tree (uncommitted state before the first Task 2 commit), by
`REXX_GROUP_TABLE=<file> REXX_CORPUS_GATE=1 memcap 8G cargo test -p rexx-exec --test concurrency_tests the_outcome_table -- --nocapture`,
the Task 1 command, raw output in `task-2-message-table.raw.md`. Delta columns against `task-1-message-table.md`.

Summary: pass 39, refused 29 (Task 1: pass 35, refused 33)

Refusal messages:

- 11 x a wait inside a frame that pins its activity is not implemented
- 7 x method "REPLYWITH" of class "Message" is not implemented (Phase 9)
- 6 x method "REPLY" of class "Message" is not implemented (Phase 9)
- 2 x method "MAKEARRAY" of class "Object" is not implemented (Phase 9)
- 1 x method "HALT" of class "Message" is not implemented (Phase 9)
- 1 x method "NOTIFY" of class "Message" is not implemented (Phase 9)
- 1 x method "TARGET" of class "Message" is not implemented (Phase 9)

Changed rows:

| test | Task 1 | Task 2 |
|---|---|---|
| TEST_START | refused method "START" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_NOT_ARRAY | refused method "STARTWITH" of class "Message" is not implemented (Phase 9) | refused method "MAKEARRAY" of class "Object" is not implemented (Phase 9) |
| TEST_STARTWITH_NO_ARRAY | refused method "STARTWITH" of class "Message" is not implemented (Phase 9) | pass  |
| TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | refused method "STARTWITH" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_OVERRIDE_CONTEXT | refused method "STARTWITH" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_OVERRIDE_FROM_NONSELF | refused method "STARTWITH" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_OVERRIDE_NOT_FOUND | refused method "STARTWITH" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | refused method "STARTWITH" of class "Message" is not implemented (Phase 9) | pass  |
| TEST_STARTWITH_OVERRIDE_NO_METHOD | refused method "STARTWITH" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_TOO_MANY | refused method "STARTWITH" of class "Message" is not implemented (Phase 9) | pass  |
| TEST_START_OVERRIDE_AMONG_MIXINCLASSES | refused method "START" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_START_OVERRIDE_CONTEXT | refused method "START" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_START_OVERRIDE_FROM_NONSELF | refused method "START" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_START_OVERRIDE_NOT_FOUND | refused method "START" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |
| TEST_START_OVERRIDE_NOT_NON_SCOPE | refused method "START" of class "Message" is not implemented (Phase 9) | pass  |
| TEST_START_OVERRIDE_NO_METHOD | refused method "START" of class "Message" is not implemented (Phase 9) | refused a wait inside a frame that pins its activity is not implemented |

Full table:

| test | outcome | refusal |
|---|---|---|
| TEST_ARRAY_OPTION_NO_ARG | pass |  |
| TEST_HALT_START | refused | method "HALT" of class "Message" is not implemented (Phase 9) |
| TEST_INVALID_OPTION | pass |  |
| TEST_NOTIFY | refused | method "NOTIFY" of class "Message" is not implemented (Phase 9) |
| TEST_NO_ARGS | pass |  |
| TEST_NO_MESSAGE | pass |  |
| TEST_NO_OPTION | pass |  |
| TEST_NO_TARGET | pass |  |
| TEST_OVERRIDE_AMONG_MIXINCLASSES | pass |  |
| TEST_OVERRIDE_EMPTY_ARRAY | pass |  |
| TEST_OVERRIDE_EXTRA_STUFF | pass |  |
| TEST_OVERRIDE_FROM_NONSELF | pass |  |
| TEST_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass |  |
| TEST_OVERRIDE_MISSING_NAME | pass |  |
| TEST_OVERRIDE_MISSING_SCOPE | pass |  |
| TEST_OVERRIDE_NON_CLASS_SCOPE | pass |  |
| TEST_OVERRIDE_NON_CLASS_SCOPE2 | pass |  |
| TEST_OVERRIDE_NON_STRING_NAME | pass |  |
| TEST_REPLY | refused | method "REPLY" of class "Message" is not implemented (Phase 9) |
| TEST_REPLYWITH_NOT_ARRAY | refused | method "REPLYWITH" of class "Message" is not implemented (Phase 9) |
| TEST_REPLYWITH_NO_ARRAY | refused | method "REPLYWITH" of class "Message" is not implemented (Phase 9) |
| TEST_REPLYWITH_OVERRIDE_CONTEXT | refused | method "REPLYWITH" of class "Message" is not implemented (Phase 9) |
| TEST_REPLYWITH_OVERRIDE_NOT_FOUND | refused | method "REPLYWITH" of class "Message" is not implemented (Phase 9) |
| TEST_REPLYWITH_OVERRIDE_NOT_NON_SCOPE | refused | method "REPLYWITH" of class "Message" is not implemented (Phase 9) |
| TEST_REPLYWITH_OVERRIDE_NO_METHOD | refused | method "REPLYWITH" of class "Message" is not implemented (Phase 9) |
| TEST_REPLYWITH_TOO_MANY | refused | method "REPLYWITH" of class "Message" is not implemented (Phase 9) |
| TEST_REPLY_OVERRIDE_CONTEXT | refused | method "REPLY" of class "Message" is not implemented (Phase 9) |
| TEST_REPLY_OVERRIDE_NOT_FOUND | refused | method "REPLY" of class "Message" is not implemented (Phase 9) |
| TEST_REPLY_OVERRIDE_NOT_NON_SCOPE | refused | method "REPLY" of class "Message" is not implemented (Phase 9) |
| TEST_REPLY_OVERRIDE_NO_METHOD | refused | method "REPLY" of class "Message" is not implemented (Phase 9) |
| TEST_SEND | refused | method "TARGET" of class "Message" is not implemented (Phase 9) |
| TEST_SENDWITH_NOT_ARRAY | refused | method "MAKEARRAY" of class "Object" is not implemented (Phase 9) |
| TEST_SENDWITH_NO_ARRAY | pass |  |
| TEST_SENDWITH_OVERRIDE_AMONG_MIXINCLASSES | pass |  |
| TEST_SENDWITH_OVERRIDE_CONTEXT | pass |  |
| TEST_SENDWITH_OVERRIDE_FROM_NONSELF | pass |  |
| TEST_SENDWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass |  |
| TEST_SENDWITH_OVERRIDE_NOT_FOUND | pass |  |
| TEST_SENDWITH_OVERRIDE_NOT_NON_SCOPE | pass |  |
| TEST_SENDWITH_OVERRIDE_NO_METHOD | pass |  |
| TEST_SENDWITH_TOO_MANY | pass |  |
| TEST_SEND_OVERRIDE_AMONG_MIXINCLASSES | pass |  |
| TEST_SEND_OVERRIDE_AMONG_MIXINCLASSES_CLASS | pass |  |
| TEST_SEND_OVERRIDE_CONTEXT | pass |  |
| TEST_SEND_OVERRIDE_FROM_NONSELF | pass |  |
| TEST_SEND_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass |  |
| TEST_SEND_OVERRIDE_NOT_FOUND | pass |  |
| TEST_SEND_OVERRIDE_NOT_NON_SCOPE | pass |  |
| TEST_SEND_OVERRIDE_NO_METHOD | pass |  |
| TEST_START | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_NOT_ARRAY | refused | method "MAKEARRAY" of class "Object" is not implemented (Phase 9) |
| TEST_STARTWITH_NO_ARRAY | pass |  |
| TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_OVERRIDE_CONTEXT | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_OVERRIDE_FROM_NONSELF | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass |  |
| TEST_STARTWITH_OVERRIDE_NOT_FOUND | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | pass |  |
| TEST_STARTWITH_OVERRIDE_NO_METHOD | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_STARTWITH_TOO_MANY | pass |  |
| TEST_START_OVERRIDE_AMONG_MIXINCLASSES | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_START_OVERRIDE_CONTEXT | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_START_OVERRIDE_FROM_NONSELF | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass |  |
| TEST_START_OVERRIDE_NOT_FOUND | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_START_OVERRIDE_NOT_NON_SCOPE | pass |  |
| TEST_START_OVERRIDE_NO_METHOD | refused | a wait inside a frame that pins its activity is not implemented |
| TEST_SUPER_OVERRIDE | refused | method "REPLY" of class "Message" is not implemented (Phase 9) |
