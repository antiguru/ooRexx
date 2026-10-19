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

The Task 6.7 fix round 1 (`task-6.7-report.md`, "Fix round 1", Concerns) widens this. The oracle
reads an Array as its lines for every whole-number method argument, not only `section`'s count.
Here `'abcdef'~substr`, `~left` and `~copies`, and MutableBuffer's `substr` and `new` size, still
refuse an Array. The fix belongs in the `whole_method_argument` family in `method_arguments.rs`.
