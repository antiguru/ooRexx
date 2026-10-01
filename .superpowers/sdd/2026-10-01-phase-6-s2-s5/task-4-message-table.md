# Task 4 table: base/class/Message

Produced at the Task 4 tree (before its commit) by `REXX_GROUP_TABLE=<file> REXX_SWITCHED_TABLE=<file> REXX_OBJECT_TABLE=<file> REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test concurrency_tests group_runs` (concurrency_tests.rs, module group_runs). Raw output in `task-4-message-table.raw.md`; the start tests under `EveryOpportunity` in `task-4-message-switched-table.raw.md`. Task 3 table: pass 51, refused 17.

Summary: differ 1, pass 51, refused 16

Refusal messages:

- 7 x method "REPLYWITH" of class "Message" is not implemented (Phase 9)
- 6 x method "REPLY" of class "Message" is not implemented (Phase 9)
- 2 x method "MAKEARRAY" of class "Object" is not implemented (Phase 9)
- 1 x method "NOTIFY" of class "Message" is not implemented (Phase 9)

Changed rows against task-3-message-table.md:

| test | Task 3 | now |
|---|---|---|
| TEST_HALT_START | refused method "HALT" of class "Message" is not implemented (Phase 9) | differ oracle Some("Assertions:         21"), ours Some("Assertions:         2"); ours stderr "" |

Full table:

| test | outcome | detail |
|---|---|---|
| TEST_ARRAY_OPTION_NO_ARG | pass |  |
| TEST_HALT_START | differ | oracle Some("Assertions:         21"), ours Some("Assertions:         2"); ours stderr "" |
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
| TEST_SEND | pass |  |
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
| TEST_START | pass |  |
| TEST_STARTWITH_NOT_ARRAY | refused | method "MAKEARRAY" of class "Object" is not implemented (Phase 9) |
| TEST_STARTWITH_NO_ARRAY | pass |  |
| TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | pass |  |
| TEST_STARTWITH_OVERRIDE_CONTEXT | pass |  |
| TEST_STARTWITH_OVERRIDE_FROM_NONSELF | pass |  |
| TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass |  |
| TEST_STARTWITH_OVERRIDE_NOT_FOUND | pass |  |
| TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | pass |  |
| TEST_STARTWITH_OVERRIDE_NO_METHOD | pass |  |
| TEST_STARTWITH_TOO_MANY | pass |  |
| TEST_START_OVERRIDE_AMONG_MIXINCLASSES | pass |  |
| TEST_START_OVERRIDE_CONTEXT | pass |  |
| TEST_START_OVERRIDE_FROM_NONSELF | pass |  |
| TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass |  |
| TEST_START_OVERRIDE_NOT_FOUND | pass |  |
| TEST_START_OVERRIDE_NOT_NON_SCOPE | pass |  |
| TEST_START_OVERRIDE_NO_METHOD | pass |  |
| TEST_SUPER_OVERRIDE | refused | method "REPLY" of class "Message" is not implemented (Phase 9) |
