**OPEN: found by the Phase 9d Task 6.7 review, not yet ruled.** Collection index arguments that are not plain strings are refused here where the oracle reads them.

# Index arguments the oracle converts and this crate refuses

Found 2026-10-19 by the Task 6.7 review (`task-6.7-review.md`, Minor 4). Each predates Task 6.7 and
lies outside its clusters.
* An index object that is not a string but has a `string` method (`::method string; return '2'`):
  `q~at(.S2~new)` answers the item on the oracle and raises 93.907 here, on Queue and on Array.
* A nested Array index on an Array receiver: `a~at(.array~of(.array~of(2)))` answers the item on
  the oracle and raises 93.907 here.
* An Array index on a List: `l~at(.array~of(2))` answers the item on the oracle and raises
  `93.918 Incorrect list index "2".` here. List `section`'s count raises 93.906 where the oracle
  raises 93.923.
