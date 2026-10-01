# 2026-10-01-condition-object-directory-methods

Found by the Phase 6 S2-S5 Task 3 review (M1), ruled not that task's (queued 2026-10-01). Not fixed.

`Message~errorCondition` answers the interpreter's own condition object, a `Body::Native`
Directory (`rexx-exec/src/condition.rs` `build_condition_object_from`, the `native_instance` arm).
Only `[]`, `AT`, `PUT`, the entry-name methods and `STRING` answer on it; the rest of the Directory
protocol refuses loudly as "method ... of class "Directory" is not implemented (Phase 9)" (ITEMS,
HASINDEX, ALLINDEXES, ALLITEMS, ENTRY, HASENTRY, INDEX, HASITEM, SUPPLIER, MAKEARRAY, ISEMPTY,
SETENTRY, REMOVE, EMPTY; COPY as class "Object"), an owner not checked against any plan row.
Before Task 3 no Rexx code reached this object: `CONDITION('O')` answers a store-backed copy
(`condition_copy`). The oracle's object is a real Directory (`Activity::createExceptionObject`).
Identity matters: the same object is updated in place by a re-raise (`Activity::reraiseException`,
`concurrency/Activity.cpp:1330`), so a fix gives this object the store Directory surface rather than
answering a copy.

Probe (from a fresh empty directory):

    m = .t~new~start('boom')
    m~wait
    call SysSleep 0.3
    c = m~errorCondition
    signal on syntax name s1
    say 'items' c~items
    s1: if symbol('RC') == 'VAR' then say 'items refused' condition('O')~code
    say 'copy items' c~copy~items
    ::class t
    ::method boom unguarded
      return 1/0

Oracle: stdout `items 13`, `copy items 13`; stderr the started report; rc 0.
This crate (9c9f6e510 and the fix round after it): stdout empty; stderr the started report then
`rexx-exec: method "ITEMS" of class "Directory" is not implemented (Phase 9)`; rc 120.
