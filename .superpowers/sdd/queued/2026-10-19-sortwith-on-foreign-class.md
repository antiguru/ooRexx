**OPEN: found by the Phase 9d Task 6.11 re-review, not yet ruled.** Two `SORTWITH` differences that predate Task 6.11.

# `SORTWITH` defined elsewhere, and OrderedCollection's stub

Found 2026-10-19 (`task-6.11-rereview.md`, Minor 4); Task 6.11's parent gives the same.
* `.k~define('SORTWITH', .array~method('SORTWITH'))` on a class that is not an Array, then
  `.ksub~new~sortWith(...)`: the oracle raises `98.975 Missing array element at position 2.`. The
  crate aborts with the refusal `a message send to a value that is not an array is not
  implemented`, rc 120.
* `.OrderedCollection~method('SORTWITH')~source~items` answers 0 on the oracle, whose method is
  native, and 1 here, where it is a Rexx stub
  (`forward message('STABLESORTWITH') -- just use the stable variety`).
* `.array~new(3)~sortWith(.numericComparator~new)`, a sort of an array with no items, answers on
  the oracle and raises 98.975 here (Task 6.11 fix round 2 report; the task's parent the same).
