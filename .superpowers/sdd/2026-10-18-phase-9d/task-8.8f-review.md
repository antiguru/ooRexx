Spec compliance: Issues found. The collection reentry requirement is incomplete.

Task quality: Needs fixes. Two correctness findings, both reproduced against the frozen release pins.

## Important findings

* **F1: A reentrant item comparison can make REMOVEITEM claim a removal it skipped.** `rust/crates/rexx-exec/src/dispatch/hash/removal.rs:169` ignores the `None` returned by `take_at` when its saved Store changes (`removal.rs:53`). If the wanted item's `==` inserts a colliding, unrelated entry, the free-list scalar changes, `take_at` abandons the removal, and the caller still returns the victim as removed. The focused control prints `removed victim items 1 victim-still-present 0` on e2 and `removed victim items 2 victim-still-present 1` on f, both status 0 with empty stderr. Revalidate the candidate and predecessor against the current store after comparison, without resending HASHCODE, and return only the item actually removed; the same ignored result exists in `rust/crates/rexx-exec/src/dispatch/hash/relation.rs:264`.
* **F2: ALLAT reads its answer from the store preceding the newly observable HASHCODE.** `rust/crates/rexx-exec/src/dispatch/hash/relation.rs:23` adds a user call inside `slots_for`, but `native_relation_all_at` already captured its Store at `relation.rs:52` and reads the returned slot numbers from that old store at `relation.rs:55`. A queried key whose HASHCODE empties and repopulates the Relation makes f print `answer old current new items 1`, status 0 with empty stderr, so the answer comes from an entry no longer in the collection. The e2 control prints `answer old current old items 1` because its ALLAT does not send HASHCODE, and `native_relation_supplier` has the same stale-store pattern at `relation.rs:176`. Return entries together with the validated store, or collect rooted values after conversion and equality reentry have been reconciled, so slot numbers never cross store identities.

## Checked invariants

* Required-string boundaries: `dispatch/hash/stem.rs:620`, `parse_template/trigger.rs:63`, and `run/condition/raise.rs:88` root the converted object before later interpreter work and consume its bytes through the existing helper. Stem separators use argument positions, including omitted and leading-empty indexes. PARSE converts before borrowing source bytes and leaves positional conversion separate. `builtin/numeric.rs:347` and `:410` reuse converted arguments for whole-number quietness and parsing while retaining borrowed ordinary arguments and stopping the prepass once quietness is impossible.
* TRACE: `run/assignment.rs:40` gates the pre-write result rendering on entry tracing, while the variable/stem/compound arms render assignment values after the write. `trace.rs:601` and `:612` gate the two rendering protocols independently. `tests/semantic_boundaries/trace-raise.rex:9` checks that the failed third naming send follows the assignment; `trace-gates.rex:1` covers OFF/R/I. The required-string changes do not substitute required STRING for TRACE naming.
* Stdout: `dispatch/stream.rs:348` requires both owned descriptors and persistent stdout classification. SIZE/SYS hand over pending output, logical WRITE CHAR includes pending bytes, and safe rustix descriptor borrowing introduces no ownership transfer. `invocation.rs:131` establishes that `own_descriptors` means this process's stdout/stderr; `input.rs:724` supplies the existing handover and error retention behavior. `tests/semantic_boundaries.rs:78` and `:90` cover embedding isolation and redirected-file contents, size, and shared offset.
* Reads and rulings: `dispatch/stream/read.rs:18` uses bounded chunk reads and propagates EINTR without a retry, discarding the incomplete result on error. The retained `semantics/fifo/verification.json:1` records empty results, no controlled unblock, and NOTREADY before HALT for clean and partial reads. It explicitly contains different outer-print placement, which is not treated as a promised ordering. `docs/superpowers/plans/phase-4-exclusions.txt:3097` adds only the authorized 82/83 rulings, with the approval quote and separate mappings.
* The reentrant count discrepancy remains separate: `semantics/equality-reentry/ruling-request.md:1` and the e2/fixed/oracle stdout files show 0/0/-1. Preserving e2's zero is not a new f regression, and neither 82 nor 83 licenses the oracle departure. Its pending controller/user disposition remains open. F1/F2 are separate store-consistency defects, not objections to that count ruling.

## Evidence and review limits

The source package spans `48e73fe3a16c10e96937546c03b11766a31419ef` through `d84ac3917`, with runtime source pinned at `3979b7e31`. I reviewed it in bounded passes and normalized the large RAISE extraction against its deleted block to verify that only module visibility and DESCRIPTION conversion changed. Focused reads outside the diff checked Store equality/generation and comparison helpers, the cut-off ALLAT/SUPPLIER callers, required-string fast paths, TRACE gates, and stdout sink ownership/handover. No source, index, HEAD, or branch mutation occurred, and the only repository write was this requested report.

The retained focused-final, reentry-final, collect, format, lint-complete, source-policy-final, and release-probes command records all report status 0. Their final stderr contains Cargo progress only, with no remaining compiler/lint warning, and the final reentry run records 1,235 library tests plus three semantic tests. The fixture driver at `tests/semantic_boundaries.rs:19` runs every fixture both normally and under collection on every allocation. No successful suite was rerun; full own-BASE performance, complete final corpus/suite/workspace gates, and unchanged cross-task contracts remain controller-owned requirements not established by this review.

## Focused reproductions

The two probes below were each run once on each pin using `memcap 2G timeout 20 /tmp/codex-p9d-t88-pins/<version>-rexx /tmp/codex-t88f-review/<probe>.rex`, from separate per-version directories. Initial sandbox attempts failed before executing either interpreter because memcap could not access a delegated cgroup. The authorized outside-sandbox reruns all exited 0 with empty stderr, producing the outputs shown below. SHA256 checks matched the retained pins: f `0c9fb8371ab3d6dcea780194e02c112ed7981447f7f396e606e5f418975583f0`, e2 `00e29fde4456ee4845a76941263f495b827c083af35363cb54eb3ac8c6b7c733`.

| Probe | e2 stdout | f stdout |
| --- | --- | --- |
| removeitem | `removed victim items 1 victim-still-present 0` | `removed victim items 2 victim-still-present 1` |
| allat | `answer old current old items 1` | `answer old current new items 1` |

### Removeitem

```rexx
t = .table~new
t~put('victim', .key~new(1))
w = .wanted~new(t)
say 'removed' t~removeItem(w) 'items' t~items 'victim-still-present' t~hasIndex(.key~new(1))
exit
::class key
::attribute label
::method init
  expose label
  use arg label
::method hashCode
  return 'key'
::method '=='
  expose label
  use arg other
  return label == other~label
::class wanted
::method init
  expose table armed
  use arg table
  armed = 1
::method '=='
  expose table armed
  if armed then do
    armed = 0
    table~put('extra', .key~new(2))
  end
  return 1
```

### Allat

```rexx
r = .relation~new
k = .key~new(r)
r~put('old',k)
q = .key~new(r)
q~arm
a = r~allAt(q)
say 'answer' a[1] 'current' r~at(q) 'items' r~items
exit
::class key
::method init
  expose table armed
  use arg table
  armed = 0
::method arm
  expose armed
  armed = 1
::method hashCode
  expose table armed
  if armed then do
    armed = 0
    table~empty
    table~put('new', .key~new(table))
  end
  return 'key'
::method '=='
  return 1
```
