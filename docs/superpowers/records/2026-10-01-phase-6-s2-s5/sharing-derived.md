# criterion 6 over criterion 1's derived list of ooTest tests, at 0ac73b804, from rust/:
# memcap 8G cargo test --release -p rexx-exec --features sharing --test concurrency_tests -- --exact sharing::sharing_fraction_over_the_derived_list --nocapture

tests 202, objects 374510, made before the program 54944, shared 790

| test | outcome | objects | before program | shared |
|---|---|---|---|---|
| base/bif/STREAM.testGroup TEST_QUERYDIR_EXISTS | pass, rc 0 | 1595 | 272 | 0 |
| base/bif/TIME.testGroup TEST_2 | pass, rc 0 | 2557 | 272 | 0 |
| base/bif/TIME.testGroup TEST_3 | pass, rc 0 | 2575 | 272 | 0 |
| base/bif/TIME.testGroup TEST_4 | failure, rc 1 | 2720 | 272 | 0 |
| base/bif/TIME.testGroup TEST_5 | failure, rc 1 | 2742 | 272 | 0 |
| base/bif/TIME.testGroup TEST_8 | pass, rc 0 | 2567 | 272 | 0 |
| base/bif/TIME.testGroup TEST_9 | pass, rc 0 | 2583 | 272 | 0 |
| base/bif/TIME.testGroup TEST_10 | failure, rc 1 | 2719 | 272 | 0 |
| base/bif/TIME.testGroup TEST_11 | failure, rc 1 | 2742 | 272 | 0 |
| base/class/Alarm.testGroup TEST_BASE_ALARM | pass, rc 0 | 1618 | 272 | 26 |
| base/class/Alarm.testGroup TEST_ALARM_NO_TIME | pass, rc 0 | 1406 | 272 | 0 |
| base/class/Alarm.testGroup TEST_ALARM_NO_TARGET | pass, rc 0 | 1406 | 272 | 0 |
| base/class/Alarm.testGroup TEST_ALARM_BAD_TIME | pass, rc 0 | 1510 | 272 | 0 |
| base/class/Alarm.testGroup TEST_ALARM_NEGATIVE_TIME | pass, rc 0 | 1414 | 272 | 0 |
| base/class/Alarm.testGroup TEST_ALARM_BAD_TARGET | pass, rc 0 | 1408 | 272 | 0 |
| base/class/Class.testGroup TEST_SUBCLASSES | pass, rc 0 | 1757 | 272 | 0 |
| base/class/DateTime.testGroup TEST_ELAPSED1 | pass, rc 0 | 1598 | 272 | 0 |
| base/class/EventSemaphore.testGroup TEST_NEW_ONE_ARG | pass, rc 0 | 1424 | 272 | 0 |
| base/class/EventSemaphore.testGroup TEST_ISPOSTED_ONE_ARG | pass, rc 0 | 1421 | 272 | 0 |
| base/class/EventSemaphore.testGroup TEST_POST_ONE_ARG | pass, rc 0 | 1420 | 272 | 0 |
| base/class/EventSemaphore.testGroup TEST_RESET_ONE_ARG | pass, rc 0 | 1420 | 272 | 0 |
| base/class/EventSemaphore.testGroup TEST_POST_RESET | pass, rc 0 | 1345 | 272 | 0 |
| base/class/EventSemaphore.testGroup TEST_WAIT_TWO_ARGS | pass, rc 0 | 1420 | 272 | 0 |
| base/class/EventSemaphore.testGroup TEST_WAIT_NUMBER | pass, rc 0 | 1420 | 272 | 0 |
| base/class/EventSemaphore.testGroup TEST_WAIT_SIMPLE | pass, rc 0 | 1347 | 272 | 0 |
| base/class/EventSemaphore.testGroup TEST_WAIT_CONCURRENT | pass, rc 0 | 1361 | 272 | 5 |
| base/class/Message.testGroup TEST_START | pass, rc 0 | 1774 | 272 | 39 |
| base/class/Message.testGroup TEST_REPLY | pass, rc 0 | 1800 | 272 | 40 |
| base/class/Message.testGroup TEST_NOTIFY | pass, rc 0 | 1607 | 272 | 10 |
| base/class/Message.testGroup TEST_SUPER_OVERRIDE | pass, rc 0 | 1620 | 272 | 6 |
| base/class/Message.testGroup TEST_STARTWITH_NO_ARRAY | pass, rc 0 | 1661 | 272 | 0 |
| base/class/Message.testGroup TEST_STARTWITH_NOT_ARRAY | refused at method "MAKEARRAY" of class "Object" is not implemented (Phase 9) | 1427 | 272 | 0 |
| base/class/Message.testGroup TEST_STARTWITH_TOO_MANY | pass, rc 0 | 1661 | 272 | 0 |
| base/class/Message.testGroup TEST_REPLYWITH_NO_ARRAY | pass, rc 0 | 1661 | 272 | 0 |
| base/class/Message.testGroup TEST_REPLYWITH_NOT_ARRAY | refused at method "MAKEARRAY" of class "Object" is not implemented (Phase 9) | 1427 | 272 | 0 |
| base/class/Message.testGroup TEST_REPLYWITH_TOO_MANY | pass, rc 0 | 1661 | 272 | 0 |
| base/class/Message.testGroup TEST_START_OVERRIDE_CONTEXT | pass, rc 0 | 1595 | 272 | 3 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_CONTEXT | pass, rc 0 | 1597 | 272 | 3 |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_CONTEXT | pass, rc 0 | 1596 | 272 | 3 |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_CONTEXT | pass, rc 0 | 1598 | 272 | 3 |
| base/class/Message.testGroup TEST_START_OVERRIDE_NOT_FOUND | pass, rc 0 | 1663 | 272 | 22 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NOT_FOUND | pass, rc 0 | 1665 | 272 | 22 |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_NOT_FOUND | pass, rc 0 | 1664 | 272 | 22 |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NOT_FOUND | pass, rc 0 | 1666 | 272 | 22 |
| base/class/Message.testGroup TEST_START_OVERRIDE_NO_METHOD | pass, rc 0 | 1663 | 272 | 22 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NO_METHOD | pass, rc 0 | 1665 | 272 | 22 |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_NO_METHOD | pass, rc 0 | 1664 | 272 | 22 |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NO_METHOD | pass, rc 0 | 1666 | 272 | 22 |
| base/class/Message.testGroup TEST_START_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 1665 | 272 | 0 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 1668 | 272 | 0 |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 1665 | 272 | 0 |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 1668 | 272 | 0 |
| base/class/Message.testGroup TEST_HALT_START | pass, rc 0 | 1769 | 272 | 32 |
| base/class/Message.testGroup TEST_START_OVERRIDE_FROM_NONSELF | pass, rc 0 | 1688 | 272 | 7 |
| base/class/Message.testGroup TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | 1659 | 272 | 0 |
| base/class/Message.testGroup TEST_START_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | 1697 | 272 | 7 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF | pass, rc 0 | 1696 | 272 | 7 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | 1659 | 272 | 0 |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | 1710 | 272 | 7 |
| base/class/Method.testGroup TESTDIRECTIVES | refused at DO is not implemented | 907 | 272 | 0 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_CLASS | refused at DO is not implemented | 5539 | 272 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_OBJECT | refused at DO is not implemented | 5541 | 272 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STRING | refused at DO is not implemented | 5540 | 272 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_METHOD | refused at DO is not implemented | 5540 | 272 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_ROUTINE | refused at DO is not implemented | 5541 | 272 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_PACKAGE | refused at DO is not implemented | 5541 | 272 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MESSAGE | pass, rc 0 | 5732 | 272 | 11 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STREAM | refused at DO is not implemented | 5540 | 272 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MUTABLEBUFFER | refused at DO is not implemented | 5542 | 272 | 2 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_FILE | refused at DO is not implemented | 5539 | 272 | 2 |
| base/class/MutexSemaphore.testGroup TEST_NEW_ONE_ARG | pass, rc 0 | 1420 | 272 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_TWO_ARGS | pass, rc 0 | 1416 | 272 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_NUMBER | pass, rc 0 | 1416 | 272 | 0 |
| base/class/MutexSemaphore.testGroup TEST_RELEASE_ONE_ARG | pass, rc 0 | 1416 | 272 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_ACQUIRE_SIMPLE | pass, rc 0 | 1342 | 272 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_RELEASE_SIMPLE | pass, rc 0 | 1340 | 272 | 0 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_RELEASE_NESTED | pass, rc 0 | 1341 | 272 | 0 |
| base/class/MutexSemaphore.testGroup TEST_EXCLUSION | pass, rc 0 | 1345 | 272 | 3 |
| base/class/Object.testGroup TESTSTART01 | pass, rc 0 | 1936 | 272 | 2 |
| base/class/Object.testGroup TESTSTARTWITH01 | pass, rc 0 | 1937 | 272 | 2 |
| base/class/Object.testGroup TEST_START_NO_NAME | pass, rc 0 | 2003 | 272 | 0 |
| base/class/Object.testGroup TEST_START_NO_NAME2 | pass, rc 0 | 2003 | 272 | 0 |
| base/class/Object.testGroup TEST_START_NOT_STRING | pass, rc 0 | 2003 | 272 | 0 |
| base/class/Object.testGroup TEST_START_NO_METHOD | pass, rc 0 | 2003 | 272 | 21 |
| base/class/Object.testGroup TEST_START_OVERRIDE | pass, rc 0 | 1990 | 272 | 8 |
| base/class/Object.testGroup TEST_START_OVERRIDE_CONTEXT | pass, rc 0 | 1940 | 272 | 3 |
| base/class/Object.testGroup TEST_START_OVERRIDE_EMPTY_ARRAY | pass, rc 0 | 2004 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_MISSING_NAME | pass, rc 0 | 2006 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_MISSING_SCOPE | pass, rc 0 | 2005 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_EXTRA_STUFF | pass, rc 0 | 2006 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NON_STRING_NAME | pass, rc 0 | 2006 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NON_CLASS_SCOPE | pass, rc 0 | 2006 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NON_CLASS_SCOPE2 | pass, rc 0 | 2005 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NOT_FOUND | pass, rc 0 | 2008 | 272 | 22 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 2013 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NO_METHOD | pass, rc 0 | 2013 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_NO_NAME | pass, rc 0 | 2004 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_NO_NAME2 | pass, rc 0 | 2005 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_NOT_STRING | pass, rc 0 | 2005 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_NO_METHOD | pass, rc 0 | 2004 | 272 | 21 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE | pass, rc 0 | 1991 | 272 | 8 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_CONTEXT | pass, rc 0 | 1941 | 272 | 3 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_EMPTY_ARRAY | pass, rc 0 | 2006 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_MISSING_NAME | pass, rc 0 | 2008 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_MISSING_SCOPE | pass, rc 0 | 2007 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_EXTRA_STUFF | pass, rc 0 | 2008 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_STRING_NAME | pass, rc 0 | 2008 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE | pass, rc 0 | 2008 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE2 | pass, rc 0 | 2007 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NOT_FOUND | pass, rc 0 | 2009 | 272 | 22 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | 2015 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NO_METHOD | pass, rc 0 | 2015 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_FROM_NONSELF | pass, rc 0 | 2033 | 272 | 7 |
| base/class/Object.testGroup TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | 2004 | 272 | 0 |
| base/class/Object.testGroup TEST_START_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | 2040 | 272 | 7 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF | pass, rc 0 | 2039 | 272 | 7 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | 2006 | 272 | 0 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | 2047 | 272 | 7 |
| base/class/RexxContext.testGroup TEST_INTERPRETER_THREAD_INVOCATION | pass, rc 0 | 1625 | 272 | 20 |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_CANCEL | pass, rc 0 | 1435 | 272 | 2 |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_TRIGGER | pass, rc 0 | 1439 | 272 | 4 |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_CANCEL | pass, rc 0 | 1435 | 272 | 2 |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_TRIGGER | pass, rc 0 | 1439 | 272 | 4 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_CANCEL | pass, rc 0 | 1436 | 272 | 2 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER | pass, rc 0 | 1440 | 272 | 4 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_CANCEL | pass, rc 0 | 1436 | 272 | 2 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER | pass, rc 0 | 1440 | 272 | 4 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER_MULTIPLE | pass, rc 0 | 1440 | 272 | 4 |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER_MESSAGE | pass, rc 0 | 1444 | 272 | 7 |
| base/class/Ticker.testGroup TEST_TICKER_NO_ARGS | pass, rc 0 | 1501 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_NO_TIME | pass, rc 0 | 1502 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_NO_TARGET | pass, rc 0 | 1501 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_BAD_TIME | pass, rc 0 | 1502 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_NEGATIVE_TIME | pass, rc 0 | 1503 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_ALARM_TIME | pass, rc 0 | 1503 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_DATETIME_TIME | pass, rc 0 | 1509 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_NEGATIVE_TIMESPAN_TIME | pass, rc 0 | 1503 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_BAD_TARGET | pass, rc 0 | 1509 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_MESSAGE_NOTIFICATION_TARGET | pass, rc 0 | 1509 | 272 | 0 |
| base/class/Ticker.testGroup TEST_TICKER_FOUR_ARGS | pass, rc 0 | 1503 | 272 | 0 |
| base/class/Ticker.testGroup TEST_ATTACHMENT_TOO_MANY_ARGS | pass, rc 0 | 1508 | 272 | 2 |
| base/class/Ticker.testGroup TEST_CANCEL_TOO_MANY_ARGS | pass, rc 0 | 1511 | 272 | 4 |
| base/class/Ticker.testGroup TEST_CANCELED_TOO_MANY_ARGS | pass, rc 0 | 1512 | 272 | 2 |
| base/class/Ticker.testGroup TEST_CANCELLED_TOO_MANY_ARGS | pass, rc 0 | 1512 | 272 | 2 |
| base/class/Ticker.testGroup TEST_CANCEL_IMMEDIATELY | pass, rc 0 | 1436 | 272 | 2 |
| base/class/Ticker.testGroup TEST_CANCEL_TWICE | pass, rc 0 | 1436 | 272 | 2 |
| base/class/Ticker.testGroup TEST_INTERVAL_TOO_MANY_ARGS | pass, rc 0 | 1508 | 272 | 2 |
| base/class/Ticker.testGroup TEST_TICKER_INTERVAL_ZERO | pass, rc 0 | 1435 | 272 | 2 |
| base/class/Ticker.testGroup TEST_TICKER_NEW_INTERVAL_TIMESPAN | pass, rc 0 | 1437 | 272 | 2 |
| base/directives/ATTRIBUTE.testGroup TEST001 | pass, rc 0 | 1658 | 272 | 0 |
| base/directives/ATTRIBUTE.testGroup TESTDELEGATE | failure, rc 1 | 1885 | 272 | 0 |
| base/directives/CONSTANT.testGroup TEST_CONSTANT_METHOD_PROPERTIES | pass, rc 0 | 1440 | 272 | 0 |
| base/directives/METHOD.testGroup TESTGUARDEDACCESS | pass, rc 0 | 1636 | 272 | 2 |
| base/directives/METHOD.testGroup TESTDELEGATE | failure, rc 1 | 1853 | 272 | 0 |
| base/keyword/CALL.testGroup TEST_4 | failure, rc 1 | 1598 | 272 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_NOVALUE | pass, rc 0 | 1469 | 272 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_NOT_BOOLEAN | pass, rc 0 | 1494 | 272 | 0 |
| base/keyword/GUARD.testGroup TEST_ON_OFF_CONSECUTIVE | pass, rc 0 | 1425 | 272 | 0 |
| base/keyword/GUARD.testGroup TEST_ON_DEFAULT | pass, rc 0 | 1430 | 272 | 2 |
| base/keyword/GUARD.testGroup TEST_ON | pass, rc 0 | 1420 | 272 | 2 |
| base/keyword/GUARD.testGroup TEST_OFF | pass, rc 0 | 1430 | 272 | 2 |
| base/keyword/GUARD.testGroup TEST_UNGUARDED | pass, rc 0 | 1430 | 272 | 2 |
| base/keyword/GUARD.testGroup TEST_ON_OFF | pass, rc 0 | 1430 | 272 | 2 |
| base/keyword/GUARD.testGroup TEST_WHEN_SINGLE_NO_WAIT | pass, rc 0 | 1426 | 272 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_USE_LOCAL_NO_WAIT | refused at USE LOCAL in a ::METHOD body is not implemented (Phase 5) | 1262 | 272 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_SINGLE_UNINITIALIZED_NO_WAIT | pass, rc 0 | 1425 | 272 | 0 |
| base/keyword/GUARD.testGroup TEST_WHEN_MULTIPLE_NO_WAIT | pass, rc 0 | 1425 | 272 | 0 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE_TRIGGER | pass, rc 0 | 1434 | 272 | 2 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE | pass, rc 0 | 1430 | 272 | 2 |
| base/keyword/GUARD.testGroup TEST_WAIT_MULTIPLE | pass, rc 0 | 1430 | 272 | 2 |
| base/keyword/RAISE.testGroup TEST_RAISE_INSERT_CRLF | failure, rc 1 | 1649 | 272 | 25 |
| base/keyword/REPLY.testGroup TEST_REPLY_ROUTINE | pass, rc 0 | 1464 | 272 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_PROCEDURE | pass, rc 0 | 1465 | 272 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_CALL | pass, rc 0 | 1465 | 272 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_TWICE_REPLYASSERT | pass, rc 0 | 1429 | 272 | 4 |
| base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_REPLYASSERT | pass, rc 0 | 1429 | 272 | 4 |
| base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT | pass, rc 0 | 1429 | 272 | 4 |
| base/keyword/REPLY.testGroup TEST_REPLY_EXIT_CODE_REPLYASSERT | pass, rc 0 | 1429 | 272 | 4 |
| base/keyword/REPLY.testGroup TEST_REPLY_STACK_REPLYASSERT | pass, rc 0 | 1462 | 272 | 9 |
| base/keyword/REPLY.testGroup TEST_REPLY_PLAIN | pass, rc 0 | 1390 | 272 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_STRING | pass, rc 0 | 1390 | 272 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_ARRAY | pass, rc 0 | 1391 | 272 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_NIL | pass, rc 0 | 1390 | 272 | 0 |
| base/keyword/REPLY.testGroup TEST_REPLY_NOP | pass, rc 0 | 1395 | 272 | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY__CODE_RETURN | pass, rc 0 | 1395 | 272 | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY__CODE_EXIT | pass, rc 0 | 1395 | 272 | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_SAME_REPLYASSERT | pass, rc 0 | 1430 | 272 | 30 |
| base/keyword/REPLY.testGroup TEST_REPLY_CONCURRENT | pass, rc 0 | 1569 | 272 | 7 |
| base/keyword/TRACE.testGroup TEST_TRACE_GUARD | pass, rc 0 | 2209 | 272 | 0 |
| base/keyword/TRACE.testGroup TEST_TRACE_REPLY | pass, rc 0 | 2180 | 272 | 20 |
| base/keyword/TRACE_TraceObject.testGroup TEST_TRACEOBJECT_COLLECTOR | refused at DO is not implemented | 1530 | 272 | 13 |
| base/keyword/TRACE_TraceObject.testGroup TEST_CALLER_STACK_FRAME_REPLY_START | refused at DO is not implemented | 1469 | 272 | 12 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_NO_ARG | pass, rc 0 | 1418 | 272 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_TWO_ARGS | pass, rc 0 | 1418 | 272 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID | pass, rc 0 | 1420 | 272 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID_NEGATIVE | pass, rc 0 | 1421 | 272 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID_TOO_LARGE | pass, rc 0 | 1422 | 272 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_DURATION | pass, rc 0 | 1491 | 272 | 0 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_CONCURRENT | pass, rc 0 | 1399 | 272 | 7 |
| base/special.variables/RESULT_RC_SIGL.testGroup TEST_RESULT_WITH_REPLY | pass, rc 0 | 1393 | 272 | 2 |
| doc/rexxref/chapter5/Section1.testGroup TEST_OBJECT_START | pass, rc 0 | 1342 | 272 | 7 |
| regressions/bug2003_guard_when.testGroup TEST_GUARD_WHEN_1 | error, rc 2 | 1478 | 272 | 0 |
