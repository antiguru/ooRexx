# Phase 5f Task 5 — file the witnesses, and close

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 5.
BASE `b6d7d1af1`. Closing at `PENDING`.

**Every one of `String`'s rows is bound.** It reads 129 `answers` and 6
`uncomparable`, with no `loud` row left.

## 1. What moved, enumerated from the tree

The plan asks for the before and after of every class that moved, "not only
`String`, because Task 1's extraction and Task 4's `hashCode` both reach
further". Derived by diffing `corpus/method-bodies.txt` between the phase's
base commit and this one, rather than read off the plan:

| class | rows moved | from | to |
|---|---|---|---|
| `String` | 112 | `loud` | 106 `answers`, 6 `uncomparable` |
| `DateTime` | 15 | `loud` | 14 `answers`, 1 `unstable` |
| `TimeSpan` | 6 | `loud` | 6 `answers` |
| `Object` | 1 | `loud` | 1 `unstable` |
| `TraceObject` | 1 | `loud` | 1 `answers` |

**135 rows across five classes**, and no row anywhere in the table regressed.
No row was added or removed. The phase is named for `String`'s 112 and the
overspill is a quarter as much again.

Per-class totals now, for those five:

| class | totals |
|---|---|
| `String` | 129 `answers`, 6 `uncomparable` |
| `DateTime` | 90 `answers`, 2 `unstable`, 2 `diverge` |
| `TimeSpan` | 57 `answers` |
| `Object` | 27 `answers`, 2 `unstable`, 3 `loud` |
| `TraceObject` | 14 `answers` |

`DateTime`'s two `diverge` and `Object`'s three `loud`
(`instanceMethod`, `instanceMethods`, `isInstanceOf`) predate this phase and
are untouched by it.

## 2. The witnesses are filed

42 corpus programs added. Each was checked to be in `corpus/phase-5c.txt`
exactly once, in `EXPECTED_SUBSET_5C` exactly once, to have a
`crates/rexx-parse/tests/sourceline_oracle/<name>.txt`, and to be absent from
`corpus/unfiled.txt`. All 42 pass all four.

**The check is reported with its negative control**, because the first two
versions of it were broken in ways that read as success:

* `git diff --name-only` emits repo-relative paths, so stripping
  `corpus/lang/` left a `rust/` prefix on every name and nothing matched
  anything. The run printed "none listed means all filed".
* `grep -c` prints `0` *and* exits 1 when it finds nothing, so
  `$(grep -c … || echo 0)` captured `"0\n0"`. Every row was then flagged
  incomplete while every underlying value was right.

The third version was run against `string_notreal.rex`, which is filed
nowhere, and flagged it — so the check can fail.

## 3. The six `uncomparable` rows are not this phase's to close

All six are `String`'s shift operators — `<<`, `<<=`, `>>`, `>>=`, `\<<`,
`\>>` — and every one carries the same evidence: **the oracle segfaults on
this send**. They are in `corpus/oracle-crashes.txt` territory: there is no
answer to compare against, so no implementation here could move the row. They
were `uncomparable` before the phase and are unchanged by it.

## 4. Handover

* **`File`'s 50 rows stay `unanswered`.** `.File~new` cannot construct: it now
  reaches past `MutableBuffer` and stops at the `file_qualify` LIBRARY REXX
  entry point, one of a family deferred to Phase 7. `File~hashCode` is among
  them and will fall out for free once the class constructs, its Rexx-level
  definition (`StreamClasses.orx:803`) being `self~qualifiedPath~hashCode`.
* **The collections are queued next** (Moritz, 2026-09-06). The survey at
  `docs/superpowers/plans/2026-09-06-collections-survey.md` is the scoping
  input, and its headline is that the `loud` count is the wrong instrument:
  of 188 collection rows reading `answers`, only 21 answered at rc 0.
* **`Bag~put`/`Set~put` is a shipped divergence** the survey found, small and
  self-contained, not dependent on the phase.
* **`String~verify`** answering `counted` where the builtin's past-the-end
  zero is untagged text is still open — `task-3a-report.md` §6.1, and the 5c
  follow-up left the same question.

## 5. No plan-level bookkeeping changed

D86: no `CLOSED_PHASES` change, no new subset file, no `class-set.txt` edit.
Confirmed unchanged.

## 6. Gates

| gate | command | status |
|---|---|---|
| G1 | `cargo fmt --all --check` | PENDING |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | PENDING |
| G3 | `cargo test --release --workspace --no-fail-fast` | PENDING |
| G4 | G3 with `REXX_CORPUS_GATE=1` | PENDING |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | PENDING |
| G6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1` | PENDING |
| G7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1` | PENDING |

This task changes no code — it is the enumeration and the filing check — so
the run over it is the same tree Task 4d's seven gates already read at 0.
