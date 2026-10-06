# criterion 6 over criterion 1's derived list of ooTest tests, at 2a9bbbe12, from rust/:
# REXX_SHARING_LOG=LOG memcap 8G cargo test --release -p rexx-exec --features sharing --test concurrency_tests -- --exact sharing::sharing_fraction_over_the_derived_list --nocapture

tests 202

| | objects | shared |
|---|---|---|
| bootstrap | 54944 | 306 |
| program | 319566 | 391 |

| test | outcome | bootstrap objects | bootstrap shared | program objects | program shared |
|---|---|---|---|---|---|
| base/bif/STREAM.testGroup TEST_QUERYDIR_EXISTS | pass, rc 0 | 272 | 0 | 1323 | 0 |
| base/bif/TIME.testGroup TEST_2 | pass, rc 0 | 272 | 0 | 2285 | 0 |
| base/bif/TIME.testGroup TEST_3 | pass, rc 0 | 272 | 0 | 2303 | 0 |
| base/bif/TIME.testGroup TEST_4 | failure, rc 1 | 272 | 0 | 2448 | 0 |
| base/bif/TIME.testGroup TEST_5 | failure, rc 1 | 272 | 0 | 2470 | 0 |
| base/bif/TIME.testGroup TEST_8 | pass, rc 0 | 272 | 0 | 2295 | 0 |
| base/bif/TIME.testGroup TEST_9 | pass, rc 0 | 272 | 0 | 2311 | 0 |
| base/bif/TIME.testGroup TEST_10 | failure, rc 1 | 272 | 0 | 2447 | 0 |
| base/bif/TIME.testGroup TEST_11 | failure, rc 1 | 272 | 0 | 2470 | 0 |
| base/class/Alarm.testGroup TEST_BASE_ALARM | pass, rc 0 | 272 | 3 | 1346 | 22 |
| base/class/Alarm.testGroup TEST_ALARM_NO_TIME | pass, rc 0 | 272 | 0 | 1134 | 0 |
| base/class/Alarm.testGroup TEST_ALARM_NO_TARGET | pass, rc 0 | 272 | 0 | 1134 | 0 |
| base/class/Alarm.testGroup TEST_ALARM_BAD_TIME | pass, rc 0 | 272 | 0 | 1238 | 0 |
| base/class/Alarm.testGroup TEST_ALARM_NEGATIVE_TIME | pass, rc 0 | 272 | 0 | 1142 | 0 |
| base/class/Alarm.testGroup TEST_ALARM_BAD_TARGET | pass, rc 0 | 272 | 0 | 1136 | 0 |
| base/class/Class.testGroup TEST_SUBCLASSES | pass, rc 0 | 272 | 0 | 1485 | 0 |
| base/class/DateTime.testGroup TEST_ELAPSED1 | pass, rc 0 | 272 | 0 | 1326 | 0 |
| base/class/EventSemaphore.testGroup TEST_NEW_ONE_ARG | pass, rc 0 | 272 | 0 | 1152 | 0 |
| base/class/EventSemaphore.testGroup TEST_ISPOSTED_ONE_ARG | pass, rc 0 | 272 | 0 | 1149 | 0 |
| base/class/EventSemaphore.testGroup TEST_POST_ONE_ARG | pass, rc 0 | 272 | 0 | 1148 | 0 |
| base/class/EventSemaphore.testGroup TEST_RESET_ONE_ARG | pass, rc 0 | 272 | 0 | 1148 | 0 |
| base/class/EventSemaphore.testGroup TEST_POST_RESET | pass, rc 0 | 272 | 0 | 1073 | 0 |
| base/class/EventSemaphore.testGroup TEST_WAIT_TWO_ARGS | pass, rc 0 | 272 | 0 | 1148 | 0 |
| base/class/EventSemaphore.testGroup TEST_WAIT_NUMBER | pass, rc 0 | 272 | 0 | 1148 | 0 |
| base/class/EventSemaphore.testGroup TEST_WAIT_SIMPLE | pass, rc 0 | 272 | 0 | 1075 | 0 |
| base/class/EventSemaphore.testGroup TEST_WAIT_CONCURRENT | pass, rc 0 | 272 | 0 | 1089 | 4 |
| base/class/Message.testGroup TEST_START | pass, rc 0 | 272 | 19 | 1502 | 19 |
| base/class/Message.testGroup TEST_REPLY | pass, rc 0 | 272 | 19 | 1528 | 20 |
| base/class/Message.testGroup TEST_NOTIFY | pass, rc 0 | 272 | 0 | 1335 | 9 |
| base/class/Message.testGroup TEST_SUPER_OVERRIDE | pass, rc 0 | 272 | 0 | 1348 | 5 |
| base/class/Message.testGroup TEST_STARTWITH_NO_ARRAY | pass, rc 0 | 272 | 0 | 1389 | 0 |
| base/class/Message.testGroup TEST_STARTWITH_NOT_ARRAY | refused at method "MAKEARRAY" of class "Object" is not implemented (Phase 9) | 272 | 0 | 1155 | 0 |
| base/class/Message.testGroup TEST_STARTWITH_TOO_MANY | pass, rc 0 | 272 | 0 | 1389 | 0 |
| base/class/Message.testGroup TEST_REPLYWITH_NO_ARRAY | pass, rc 0 | 272 | 0 | 1389 | 0 |
| base/class/Message.testGroup TEST_REPLYWITH_NOT_ARRAY | refused at method "MAKEARRAY" of class "Object" is not implemented (Phase 9) | 272 | 0 | 1155 | 0 |
| base/class/Message.testGroup TEST_REPLYWITH_TOO_MANY | pass, rc 0 | 272 | 0 | 1389 | 0 |
| base/class/Message.testGroup TEST_START_OVERRIDE_CONTEXT | pass, rc 0 | 272 | 0 | 1323 | 2 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_CONTEXT | pass, rc 0 | 272 | 0 | 1325 | 2 |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_CONTEXT | pass, rc 0 | 272 | 0 | 1324 | 2 |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_CONTEXT | pass, rc 0 | 272 | 0 | 1326 | 2 |
| base/class/Message.testGroup TEST_START_OVERRIDE_NOT_FOUND | pass, rc 0 | 272 | 14 | 1391 | 7 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NOT_FOUND | pass, rc 0 | 272 | 14 | 1393 | 7 |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_NOT_FOUND | pass, rc 0 | 272 | 14 | 1392 | 7 |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NOT_FOUND | pass, rc 0 | 272 | 14 | 1394 | 7 |
| base/class/Message.testGroup TEST_START_OVERRIDE_NO_METHOD | pass, rc 0 | 272 | 14 | 1391 | 7 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NO_METHOD | pass, rc 0 | 272 | 14 | 1393 | 7 |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_NO_METHOD | pass, rc 0 | 272 | 14 | 1392 | 7 |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NO_METHOD | pass, rc 0 | 272 | 14 | 1394 | 7 |
| base/class/Message.testGroup TEST_START_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 272 | 0 | 1393 | 0 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 272 | 0 | 1396 | 0 |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 272 | 0 | 1393 | 0 |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 272 | 0 | 1396 | 0 |
| base/class/Message.testGroup TEST_HALT_START | pass, rc 0 | 272 | 15 | 1497 | 16 |
| base/class/Message.testGroup TEST_START_OVERRIDE_FROM_NONSELF | pass, rc 0 | 272 | 0 | 1416 | 6 |
| base/class/Message.testGroup TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | 272 | 0 | 1387 | 0 |
| base/class/Message.testGroup TEST_START_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | 272 | 0 | 1425 | 6 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF | pass, rc 0 | 272 | 0 | 1424 | 6 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | 272 | 0 | 1387 | 0 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | 272 | 0 | 1438 | 6 |
| base/class/Method.testGroup TESTDIRECTIVES | refused at DO is not implemented | 272 | 0 | 635 | 0 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_CLASS | refused at DO is not implemented | 272 | 0 | 5267 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_OBJECT | refused at DO is not implemented | 272 | 0 | 5269 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STRING | refused at DO is not implemented | 272 | 0 | 5268 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_METHOD | refused at DO is not implemented | 272 | 0 | 5268 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_ROUTINE | refused at DO is not implemented | 272 | 0 | 5269 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_PACKAGE | refused at DO is not implemented | 272 | 0 | 5269 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MESSAGE | pass, rc 0 | 272 | 0 | 5460 | 3 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STREAM | refused at DO is not implemented | 272 | 0 | 5268 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MUTABLEBUFFER | refused at DO is not implemented | 272 | 0 | 5270 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_FILE | refused at DO is not implemented | 272 | 0 | 5267 | 2 |
| base/class/MutexSemaphore.testGroup TEST_NEW_ONE_ARG | pass, rc 0 | 272 | 0 | 1148 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_TWO_ARGS | pass, rc 0 | 272 | 0 | 1144 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_NUMBER | pass, rc 0 | 272 | 0 | 1144 | 0 |
| base/class/MutexSemaphore.testGroup TEST_RELEASE_ONE_ARG | pass, rc 0 | 272 | 0 | 1144 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_ACQUIRE_SIMPLE | pass, rc 0 | 272 | 0 | 1070 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_RELEASE_SIMPLE | pass, rc 0 | 272 | 0 | 1068 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_RELEASE_NESTED | pass, rc 0 | 272 | 0 | 1069 | 0 |
| base/class/MutexSemaphore.testGroup TEST_EXCLUSION | pass, rc 0 | 272 | 0 | 1073 | 2 |
| base/class/Object.testGroup TESTSTART01 | pass, rc 0 | 272 | 0 | 1664 | 1 |
| base/class/Object.testGroup TESTSTARTWITH01 | pass, rc 0 | 272 | 0 | 1665 | 1 |
| base/class/Object.testGroup TEST_START_NO_NAME | pass, rc 0 | 272 | 0 | 1731 | 0 |
| base/class/Object.testGroup TEST_START_NO_NAME2 | pass, rc 0 | 272 | 0 | 1731 | 0 |
| base/class/Object.testGroup TEST_START_NOT_STRING | pass, rc 0 | 272 | 0 | 1731 | 0 |
| base/class/Object.testGroup TEST_START_NO_METHOD | pass, rc 0 | 272 | 14 | 1731 | 6 |
| base/class/Object.testGroup TEST_START_OVERRIDE | pass, rc 0 | 272 | 0 | 1718 | 7 |
| base/class/Object.testGroup TEST_START_OVERRIDE_CONTEXT | pass, rc 0 | 272 | 0 | 1668 | 2 |
| base/class/Object.testGroup TEST_START_OVERRIDE_EMPTY_ARRAY | pass, rc 0 | 272 | 0 | 1732 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_MISSING_NAME | pass, rc 0 | 272 | 0 | 1734 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_MISSING_SCOPE | pass, rc 0 | 272 | 0 | 1733 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_EXTRA_STUFF | pass, rc 0 | 272 | 0 | 1734 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NON_STRING_NAME | pass, rc 0 | 272 | 0 | 1734 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NON_CLASS_SCOPE | pass, rc 0 | 272 | 0 | 1734 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NON_CLASS_SCOPE2 | pass, rc 0 | 272 | 0 | 1733 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NOT_FOUND | pass, rc 0 | 272 | 14 | 1736 | 7 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 272 | 0 | 1741 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NO_METHOD | pass, rc 0 | 272 | 0 | 1741 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_NO_NAME | pass, rc 0 | 272 | 0 | 1732 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_NO_NAME2 | pass, rc 0 | 272 | 0 | 1733 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_NOT_STRING | pass, rc 0 | 272 | 0 | 1733 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_NO_METHOD | pass, rc 0 | 272 | 14 | 1732 | 6 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE | pass, rc 0 | 272 | 0 | 1719 | 7 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_CONTEXT | pass, rc 0 | 272 | 0 | 1669 | 2 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_EMPTY_ARRAY | pass, rc 0 | 272 | 0 | 1734 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_MISSING_NAME | pass, rc 0 | 272 | 0 | 1736 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_MISSING_SCOPE | pass, rc 0 | 272 | 0 | 1735 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_EXTRA_STUFF | pass, rc 0 | 272 | 0 | 1736 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_STRING_NAME | pass, rc 0 | 272 | 0 | 1736 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE | pass, rc 0 | 272 | 0 | 1736 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE2 | pass, rc 0 | 272 | 0 | 1735 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NOT_FOUND | pass, rc 0 | 272 | 14 | 1737 | 7 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 272 | 0 | 1743 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NO_METHOD | pass, rc 0 | 272 | 0 | 1743 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_FROM_NONSELF | pass, rc 0 | 272 | 0 | 1761 | 6 |
| base/class/Object.testGroup TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | 272 | 0 | 1732 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | 272 | 0 | 1768 | 6 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF | pass, rc 0 | 272 | 0 | 1767 | 6 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | 272 | 0 | 1734 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | 272 | 0 | 1775 | 6 |
| base/class/RexxContext.testGroup TEST_INTERPRETER_THREAD_INVOCATION | pass, rc 0 | 272 | 7 | 1353 | 12 |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_CANCEL | pass, rc 0 | 272 | 0 | 1163 | 1 |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_TRIGGER | pass, rc 0 | 272 | 0 | 1167 | 3 |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_CANCEL | pass, rc 0 | 272 | 0 | 1163 | 1 |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_TRIGGER | pass, rc 0 | 272 | 0 | 1167 | 3 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_CANCEL | pass, rc 0 | 272 | 0 | 1164 | 1 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER | pass, rc 0 | 272 | 0 | 1168 | 3 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_CANCEL | pass, rc 0 | 272 | 0 | 1164 | 1 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER | pass, rc 0 | 272 | 0 | 1168 | 3 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER_MULTIPLE | pass, rc 0 | 272 | 0 | 1168 | 3 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER_MESSAGE | pass, rc 0 | 272 | 0 | 1172 | 6 |
| base/class/Ticker.testGroup TEST_TICKER_NO_ARGS | pass, rc 0 | 272 | 0 | 1229 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_NO_TIME | pass, rc 0 | 272 | 0 | 1230 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_NO_TARGET | pass, rc 0 | 272 | 0 | 1229 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_BAD_TIME | pass, rc 0 | 272 | 0 | 1230 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_NEGATIVE_TIME | pass, rc 0 | 272 | 0 | 1231 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_ALARM_TIME | pass, rc 0 | 272 | 0 | 1231 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_DATETIME_TIME | pass, rc 0 | 272 | 0 | 1237 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_NEGATIVE_TIMESPAN_TIME | pass, rc 0 | 272 | 0 | 1231 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_BAD_TARGET | pass, rc 0 | 272 | 0 | 1237 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_MESSAGE_NOTIFICATION_TARGET | pass, rc 0 | 272 | 0 | 1237 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_FOUR_ARGS | pass, rc 0 | 272 | 0 | 1231 | 0 |
| base/class/Ticker.testGroup TEST_ATTACHMENT_TOO_MANY_ARGS | pass, rc 0 | 272 | 0 | 1236 | 1 |
| base/class/Ticker.testGroup TEST_CANCEL_TOO_MANY_ARGS | pass, rc 0 | 272 | 0 | 1239 | 3 |
| base/class/Ticker.testGroup TEST_CANCELED_TOO_MANY_ARGS | pass, rc 0 | 272 | 0 | 1240 | 1 |
| base/class/Ticker.testGroup TEST_CANCELLED_TOO_MANY_ARGS | pass, rc 0 | 272 | 0 | 1240 | 1 |
| base/class/Ticker.testGroup TEST_CANCEL_IMMEDIATELY | pass, rc 0 | 272 | 0 | 1164 | 1 |
| base/class/Ticker.testGroup TEST_CANCEL_TWICE | pass, rc 0 | 272 | 0 | 1164 | 1 |
| base/class/Ticker.testGroup TEST_INTERVAL_TOO_MANY_ARGS | pass, rc 0 | 272 | 0 | 1236 | 1 |
| base/class/Ticker.testGroup TEST_TICKER_INTERVAL_ZERO | pass, rc 0 | 272 | 0 | 1163 | 1 |
| base/class/Ticker.testGroup TEST_TICKER_NEW_INTERVAL_TIMESPAN | pass, rc 0 | 272 | 0 | 1165 | 1 |
| base/directives/ATTRIBUTE.testGroup TEST001 | pass, rc 0 | 272 | 0 | 1386 | 0 |
| base/directives/ATTRIBUTE.testGroup TESTDELEGATE | failure, rc 1 | 272 | 0 | 1613 | 0 |
| base/directives/CONSTANT.testGroup TEST_CONSTANT_METHOD_PROPERTIES | pass, rc 0 | 272 | 0 | 1168 | 0 |
| base/directives/METHOD.testGroup TESTGUARDEDACCESS | pass, rc 0 | 272 | 0 | 1364 | 1 |
| base/directives/METHOD.testGroup TESTDELEGATE | failure, rc 1 | 272 | 0 | 1581 | 0 |
| base/keyword/CALL.testGroup TEST_4 | failure, rc 1 | 272 | 0 | 1326 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_NOVALUE | pass, rc 0 | 272 | 0 | 1197 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_NOT_BOOLEAN | pass, rc 0 | 272 | 0 | 1222 | 0 |
| base/keyword/GUARD.testGroup TEST_ON_OFF_CONSECUTIVE | pass, rc 0 | 272 | 0 | 1153 | 0 |
| base/keyword/GUARD.testGroup TEST_ON_DEFAULT | pass, rc 0 | 272 | 0 | 1158 | 1 |
| base/keyword/GUARD.testGroup TEST_ON | pass, rc 0 | 272 | 0 | 1148 | 1 |
| base/keyword/GUARD.testGroup TEST_OFF | pass, rc 0 | 272 | 0 | 1158 | 1 |
| base/keyword/GUARD.testGroup TEST_UNGUARDED | pass, rc 0 | 272 | 0 | 1158 | 1 |
| base/keyword/GUARD.testGroup TEST_ON_OFF | pass, rc 0 | 272 | 0 | 1158 | 1 |
| base/keyword/GUARD.testGroup TEST_WHEN_SINGLE_NO_WAIT | pass, rc 0 | 272 | 0 | 1154 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_USE_LOCAL_NO_WAIT | refused at USE LOCAL in a ::METHOD body is not implemented (Phase 5) | 272 | 0 | 990 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_SINGLE_UNINITIALIZED_NO_WAIT | pass, rc 0 | 272 | 0 | 1153 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_MULTIPLE_NO_WAIT | pass, rc 0 | 272 | 0 | 1153 | 0 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE_TRIGGER | pass, rc 0 | 272 | 0 | 1162 | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE | pass, rc 0 | 272 | 0 | 1158 | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_MULTIPLE | pass, rc 0 | 272 | 0 | 1158 | 1 |
| base/keyword/RAISE.testGroup TEST_RAISE_INSERT_CRLF | failure, rc 1 | 272 | 19 | 1377 | 5 |
| base/keyword/REPLY.testGroup TEST_REPLY_ROUTINE | pass, rc 0 | 272 | 0 | 1192 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_PROCEDURE | pass, rc 0 | 272 | 0 | 1193 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_CALL | pass, rc 0 | 272 | 0 | 1193 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_TWICE_REPLYASSERT | pass, rc 0 | 272 | 2 | 1157 | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_REPLYASSERT | pass, rc 0 | 272 | 2 | 1157 | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT | pass, rc 0 | 272 | 2 | 1157 | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_EXIT_CODE_REPLYASSERT | pass, rc 0 | 272 | 2 | 1157 | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_STACK_REPLYASSERT | pass, rc 0 | 272 | 5 | 1190 | 3 |
| base/keyword/REPLY.testGroup TEST_REPLY_PLAIN | pass, rc 0 | 272 | 0 | 1118 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_STRING | pass, rc 0 | 272 | 0 | 1118 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_ARRAY | pass, rc 0 | 272 | 0 | 1119 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_NIL | pass, rc 0 | 272 | 0 | 1118 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_NOP | pass, rc 0 | 272 | 0 | 1123 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY__CODE_RETURN | pass, rc 0 | 272 | 0 | 1123 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY__CODE_EXIT | pass, rc 0 | 272 | 0 | 1123 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_SAME_REPLYASSERT | pass, rc 0 | 272 | 9 | 1158 | 20 |
| base/keyword/REPLY.testGroup TEST_REPLY_CONCURRENT | pass, rc 0 | 272 | 3 | 1297 | 3 |
| base/keyword/TRACE.testGroup TEST_TRACE_GUARD | pass, rc 0 | 272 | 0 | 1937 | 0 |
| base/keyword/TRACE.testGroup TEST_TRACE_REPLY | pass, rc 0 | 272 | 13 | 1908 | 6 |
| base/keyword/TRACE_TraceObject.testGroup TEST_TRACEOBJECT_COLLECTOR | refused at DO is not implemented | 272 | 9 | 1258 | 3 |
| base/keyword/TRACE_TraceObject.testGroup TEST_CALLER_STACK_FRAME_REPLY_START | refused at DO is not implemented | 272 | 9 | 1197 | 2 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_NO_ARG | pass, rc 0 | 272 | 0 | 1146 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_TWO_ARGS | pass, rc 0 | 272 | 0 | 1146 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID | pass, rc 0 | 272 | 0 | 1148 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID_NEGATIVE | pass, rc 0 | 272 | 0 | 1149 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID_TOO_LARGE | pass, rc 0 | 272 | 0 | 1150 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_DURATION | pass, rc 0 | 272 | 0 | 1219 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_CONCURRENT | pass, rc 0 | 272 | 0 | 1127 | 6 |
| base/special.variables/RESULT_RC_SIGL.testGroup TEST_RESULT_WITH_REPLY | pass, rc 0 | 272 | 0 | 1121 | 0 |
| doc/rexxref/chapter5/Section1.testGroup TEST_OBJECT_START | pass, rc 0 | 272 | 0 | 1070 | 6 |
| regressions/bug2003_guard_when.testGroup TEST_GUARD_WHEN_1 | error, rc 2 | 272 | 0 | 1206 | 0 |
