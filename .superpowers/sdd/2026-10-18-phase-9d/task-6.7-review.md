# Task 6.7 review: collections

Reviewed `01db1850f..3920266a0`. I built the crate from HEAD into the session scratchpad (`rev67-target`, `cargo build --release --bin rexx`, exit 0). Every probe ran from its own fresh `mktemp -d`: the oracle under the standard wrapper, the crate under `memcap 2G timeout 20`. I compared stdout, stderr and rc separately. The parent binary is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.6-r2/rexx-head`, which the report says carries `01db1850f`'s code. I used it to sort each remaining difference into "pre-existing" or "changed by this task". The probe sources were in `scratchpad/rev67-probes.UKXZ/`, which is not durable, so the deciding lines are quoted below.

## Spec compliance

Compliant for the seven clusters. Every cluster test named in the report is fixed or licensed. The four main witnesses and the pre-review witness match the oracle in the gated corpus (1092 of 1092). The Deviation 45 row and the three Deviation 4 rows hold their conditions: every assertion outside the licensed one passes (finding 3 below). One fix-round claim is false, though. The pre-review section says `queue_index` is applied "in `position_in` ... Queue `delete`, `remove`, `hasIndex`, `next` and `previous`". It is not applied for `next` or `previous` (finding 1).

## Strengths

* **`queue_index` holds on put, `[]=`, at, `[]`, delete, remove, hasIndex, insert and section's start.** I ran 16 index shapes through Queue, CircularQueue and a Queue subclass, one process per class and method. The shapes were `.array~of(2)`, `'099'`, `'1.0'`, `-1`, `0`, `999999999999`, `'x'`, `.nil`, `.array~of('099')`, `.array~of('1.0')`, `.array~of(' 2 ')`, `.array~of(.array~of(2))`, `.array~of(2)~~put(3,3)`, `'2 '`, `1e0` and `.array~new(5)~~put(2,1)`, plus the omitted-argument forms. Every one of those methods matches the oracle on every shape, error number and message text included, apart from the gaps noted below. Examples that agree:
  * `q~put('z', .array~of('099')) ERR 93.918 Incorrect list index "an Array".`
  * `q~put('z', '099') ERR 93.918 Incorrect list index "099".`
  * `q~put('a', 1e2)` gives `"1E2"` on both sides, where the parent printed `"100"`.
  * The hole-bearing `.array~of(,2)` and `.array~new(3)~~put(2,3)` match on put, at, delete, remove, hasIndex, insert and section's start.
* **The a40a7262e receiver check holds.** For Array receivers, a classifier over the same matrix finds no row where the parent matched the oracle and HEAD does not (`REGRESSION` count 0). The same holds for List. The Array and List rows that still differ give the same output on the parent.
* **Set operations match on all 726 combinations.** I crossed eleven receivers with the same eleven argument classes: Queue, CircularQueue, Array, List, Bag, Set, Table, IdentityTable, Directory, Relation and StringTable. Each pair ran difference, xor, intersection, union, subset and disjoint, with duplicates on both sides (`a b b c c c d` against `b c c e e`). The script printed the class, count, sorted items, and indexes (sorted for unordered results). stdout is identical, 737 lines, with no ERR rows on either side.
* **Identity item equality matches the oracle.** IdentityTable `hasItem`, `index` and `removeItem` were probed with a `K` whose `==` and `=` always answer 1, and so were Set, Bag, Relation (including `hasItem(k2, 1)` and `removeItem(k2, 1)`), Directory and StringTable. An IdentityTable difference, intersection and xor over two K items also matches. stdout is identical.
* **`.nil` comparison matches across 78 operand pairs.** Left operands: `.nil`, `'The NIL object'`, `1`, `1.5`, `''`, a String subclass, a class overriding `=`, `==`, `<`, `>>` and `\=`, `.object~new`, `.array~new`, `.true`, a 20-digit number, `'abc'~upper`, and a String subclass overriding `=` and `<`. Right operands: `.nil`, a string, a number, `.object~new`, the overriding object, and an object with a `string` method. Each pair ran all 18 comparison operators. stdout and rc are identical. A String subclass's overridden `=` and `<` are still sent (`=S-eq ... <S-lt` against `.nil`), so operators stay messages. The concatenation's NOSTRING fires at the same line.
* **Queue `removeItem` matches** on Queue, CircularQueue (including wraparound after overflow, `3 3 1 2 3 z w v`), a Queue subclass, Array, an Array subclass, and List, including follow-up `append`, `pull`, `push`, `peek`, `next` and `previous`.
* **Perf reproduces.** I ran `callgrind.sh -r 1 -B 0.5 -p churn pin=... parent=t6.6-r2/rexx-head head=<my build>`, exit 0: `churn 631489428 632173200 632191406 +0.1083 +0.1112`. Head minus parent is +0.0029%, and the report gives +0.0028.
* **Tests.**
  * `cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed.
  * `REXX_CORPUS_GATE=1 ... --test corpus --test licensed_divergences --test refusal_sites`: exit 0, `1092 of 1092 matching`.
  * `-p rexx-parse --test sourceline_oracle`: exit 0.

## Issues

### Critical

None.

### Important

1. **`Queue~next` and `Queue~previous` still spread an Array index, despite the report's claim.** `array_step` (`array/surface.rs:317`) rebinds `receiver` to `store_of(receiver)` before it calls `position_in`. So `queue_index` sees the store, which is an Array and not a Queue, and returns the index unchanged. When the index is past the end, the fallback `positive_index(*only, 1)` then refuses the Array itself. The probe lines, the same on Queue, CircularQueue and a Queue subclass:
   * `q~next(.array~of('099'))`: the oracle gives `-> The NIL object`, the crate `ERR 93.907 ... found "an Array".`
   * `q~previous(.array~of('099'))`: the oracle gives `-> 3`, the crate `ERR 93.907 ... found "an Array".`
   * `q~next(.array~of(2)~~put(3,3))`: the oracle gives `93.907 ... found "an Array".`, the crate `93.926 Too many subscripts for array; 1 expected.`
   * `q~next(.array~of(,2))` and `q~next(.array~new(3)~~put(2,3))`: the oracle gives `-> 3`, while the crate stops the program with `rexx-exec: an array subscript that is an empty slot is not implemented`, rc 120. `previous` is the same, with the oracle giving `-> 1`.

   The parent gives the same output, so this is not a regression. But the round names `next` and `previous` as covered, and the witness cannot see the gap: `next(.array~of(1))` and `previous(.array~of(3))` answer the same whether the Array is spread or not. Fix: pass the original receiver to `position_in`, or call `queue_index` before the rebinding. Add a witness line with `.array~of('099')` and one with `.array~of(,2)` for each of `next` and `previous`.

### Minor

2. **`section`'s count rejects an Array that the oracle reads as a string.** On Queue, a Queue subclass and Array, `q~section(1, .array~of(2))` gives `-> a Queue` (or `an Array`) on the oracle and `ERR 93.923 Invalid length argument specified; found "an Array".` on the crate. `.array~of('099')`, `.array~of('1.0')`, `.array~of(' 2 ')` and `.array~new(5)~~put(2,1)` behave the same way. The parent refused these too, with 93.906, so the task changed the number without closing the gap. The oracle's `optionalLengthArgument` requests the string, and `optional_length_argument` does not. It is outside the cluster's tests, so it is reasonable to queue it with the other Array-argument item.
3. **The Deviation 4 rows check out, but the implementer's probe was weaker than the test.** `identity_rest.rex` checks `coll~at(indexes[i]) \== values[i]` where the test uses `assertSame`. My probe compares `identityHash` for each of the four fixture indexes under `at`, `[]` and `hasIndex`, and checks `assertNull(coll~at(.object~new))` and `coll[.object~new]`. Every one holds on both sides. So the only assertions that differ are the `"2"||"2"` and `"1223"~substr(2,2)` ones the rows name. No change is needed. A later licence probe should use the test's own comparison.
4. **Remaining divergences, all present in the parent and outside these clusters.** None needs action in this task, but they could be queued:
   * An index object that is not a string but has a `string` method (`::method string; return '2'`): `q~at(.S2~new)` gives `-> q` on the oracle and 93.907 on the crate, on Queue and on Array.
   * A nested Array index on an Array receiver: `a~at(.array~of(.array~of(2)))` gives `-> q` on the oracle and 93.907 on the crate.
   * An Array index on List, for example `l~at(.array~of(2))`: the oracle gives `-> r`, the crate `93.918 Incorrect list index "2".` List `section`'s count is 93.906 where the oracle gives 93.923.
   * `.Array~of(...)~insert('z', 999999999999)` aborts the crate process (`memory allocation of 15999999999984 bytes failed`, rc 134). The oracle raises 5.0, and the crate raises 5.0 for `put` with the same index. This falls under the OOM licence, but it is an abort, not a condition.
   * `Queue~insert('z', 999999999999)` is 5.0 on the oracle (the `IndexUpdate` extension of Concern 2) and 93.966 on the crate. This is consistent with the ruling not to copy that defect.

## Assessment

**Task quality:** Needs a small fix.

**Reasoning:** The identity, removeItem, set-operation and `.nil` fixes match the oracle on every shape I could construct, and the licence rows license only what they name. The Queue Array-index rule is right wherever it is actually applied. But the pre-review round claims `next` and `previous` as covered, they are not, and the witness cannot tell. The fix is one rebinding and two witness lines.

**Verdict: CHANGES REQUESTED** (finding 1. Finding 2 can be queued.)
