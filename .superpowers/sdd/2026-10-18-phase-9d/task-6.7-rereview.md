# Task 6.7 re-review: fix round 1

Reviewed `7a58fa2ae..3a68eb48c`, a scoped re-review of Important 1, Minor 2 and the `insert` huge-index abort. I built the crate from HEAD into the session scratchpad (`rev67r1-target`, `cargo build --release --bin rexx`, exit 0). Every crate run was under `memcap 2G timeout 20`, and every oracle run was under the standard wrapper, each from a fresh `mktemp -d`. The comparison binary is the pre-review build, `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.7-pre/rexx-head` (`a40a7262e`). Probe sources were in `scratchpad/rev67r1-probes.Qj4Z/`, which is not durable, so the deciding lines are quoted below.

## Spec compliance

Compliant. All three items are closed, and the round changes nothing else I could find.

## Findings

### Important 1 (Queue `next` and `previous`): closed

I reran my review's matrix: 20 index shapes, each through every indexed method (`put`, `[]=`, `at`, `[]`, `delete`, `remove`, `hasIndex`, `next`, `previous`, `insert`, `section` start and count, and the omitted forms). Receivers were Queue, CircularQueue, a Queue subclass, Array and List, one process per class and method. The shapes now include `.array~of(1, 2)`, `.array~new`, `.array~of(,2)` and `.array~new(3)~~put(2,3)`.

* On Queue, CircularQueue and the subclass, every row matches the oracle. A classifier that keys on (class, expression) and compares oracle, pre-review and HEAD finds no remaining Queue-family difference.
* The distinguishing shapes now agree, the same on all three receivers:
  * `q~next(.array~of('099')) -> The NIL object`
  * `q~previous(.array~of('099')) -> 3`
  * `q~next(.array~of(,2)) -> 3` and `q~previous(.array~of(,2)) -> 1`. The pre-review build stopped here with rc 120.
  * `q~next(.array~of(2)~~put(3,3)) ERR 93.907 ... found "an Array".` The pre-review build gave 93.926.
  * `q~next(.array~new(3)~~put(2,3)) -> 3`.
* Regressions: none. Every Array and List row that still differs from the oracle is identical on the pre-review build. On an Array receiver, a hole-bearing Array index still ends the process on both sides: the oracle segfaults (rc 139) and the crate refuses loudly (rc 120), as at review. That is unchanged and is the oracle's crash.

### Minor 2 (`section`'s count): closed

* Queue, the Queue subclass and Array accept `.array~of(2)`, `.array~of('099')`, `.array~of('1.0')`, `.array~of(' 2 ')`, `.array~new(5)~~put(2,1)`, `.array~of(,2)` and `.array~new(3)~~put(2,3)` as a count. For example, `q~section(1, .array~of(2)) -> a Queue`. All of these match the oracle.
* `.array~of(1, 2)` and `.array~new` are 93.923 "an Array" on both sides.
* CircularQueue's own `section` matches the oracle on every count shape (97.1 for an Array count), so the claim of 97.1 on both sides holds.
* Array with the non-hole shapes: no row differs from the oracle that does not also differ on the pre-review build. At review, the Array section count rows were "changed-still-diff"; they now match.

### Refused growth: closed for the paths named

Each case ran in its own process, comparing oracle, HEAD and pre-review:

| case | oracle | HEAD | pre-review |
|---|---|---|---|
| `.Array~of('p','q','r')~insert('z', 999999999999)` | 5.0 | 5.0 | abort, rc 134 |
| `.Array~new~insert('z', H)` | 5.0 | 5.0 | abort |
| sparse `.Array~new(5)~~put('x', 2)~insert('z', H)` | 5.0 | 5.0 | abort |
| `a~insert('z', 2**40)` | 5.0 | 5.0 | abort |
| `.Array~new(1)~~put('z', 1)~insert('y', 200000000000)` | 5.0 | 5.0 | abort |
| `put('z', H)`, `a[H] = 'z'`, `put('z', 2**40)` | 5.0 | 5.0 | 5.0 |
| `.Array~new(H)`, `.Array~new(H)~append('z')` | 5.0 | 5.0 | 5.0 |
| `.Array~new(H, H)` | 93.959 | 93.959 | 93.959 |
| `.Array~new(3,3)~~put('x', H, 1)` and `put('x', 1, H)` | 5.0 | 5.0 | 5.0 |
| `fill`, `section(1, H)`, `section(H)`, sparse `section(1, H)` | same answers | same | same |
| `.Queue~of('p')~insert('z', H)` | 5.0 | 93.966 | 93.966 |

The last row is the ruled `IndexUpdate` extension that is not copied.

Three sizes the allocator grants were OOM-killed by the 2G cgroup on HEAD and on pre-review alike: `.Array~new(100000, 100000)`, `.Array~new(0)~insert('z', 400000000)` and `~insert('y', 4000000000)`. The oracle runs under `ulimit -v 1048576`, so its allocation is refused instead. Rerun under `ulimit -v 4194304` inside `memcap 2G`, HEAD gives 5.0, rc 0, on all three. The difference comes from the environment (a cgroup kill against a refusal), not from an abort in the crate.

**Amortised growth holds.** I timed 200000 inserts per loop under `memcap 2G`, three rounds each, as pre-review and HEAD in seconds:

| loop | pre-review | HEAD |
|---|---|---|
| `insert(i)` (omitted index) | 0.063 to 0.088 | 0.064 to 0.091 |
| `insert(i, i)` | 0.098 to 0.106 | 0.098 to 0.113 |
| `insert(i, 2*i)` (past the end, 400001 slots) | 0.099 to 0.119 | 0.077 to 0.108 |
| `append` | 0.081 to 0.091 | 0.089 to 0.092 |

There is no quadratic term. `ArraySlots::try_reserve` delegates to `Vec::try_reserve`, so `resize` finds the capacity already there.

### Tests

* `cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed.
* `REXX_CORPUS_GATE=1 ... --test corpus --test licensed_divergences`: exit 0, `1093 of 1093 matching`.
* `-p rexx-core`: exit 0, all 13 binaries ok.
* `-p rexx-parse --test sourceline_oracle`: exit 0.

## Issues

None open.

## Assessment

**Verdict: APPROVED.**
