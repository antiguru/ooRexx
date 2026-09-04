# 5c follow-up Task 0 — the instrument gets teeth before anything is implemented

**BASE `6e24e5e90`.** Plan `docs/superpowers/plans/2026-09-04-phase-5c-followup.md`, spec
`docs/superpowers/specs/2026-09-04-phase-5e-mutablebuffer.md`. Done by the controller inline: one
`const` entry, one doc sentence, and the measurements below.

## What landed

`rust/crates/rexx-exec/tests/method_bodies.rs`: `RECEIVER_OVERRIDES` gains
`("MutableBuffer", ".MutableBuffer~new('abc')")`, and the const's doc gains one sentence saying why.
`check_receivers_match_table_c` already exempts every overridden class, so table C's committed
`o = .MutableBuffer~new` probe and this table's receiver are allowed to differ. No other file.

## Predictions, written before each run, and what each run read

Scratch: `…/scratchpad/task0/` (`prediction.txt`, `status`, `mb.err`, `refresh.*`, `invert/`,
`sweep/`, `array5/`, `defaultsize*/`).

| claim | predicted | read |
|---|---|---|
| P1 `cargo clippy --workspace --all-targets -- -D warnings` | 0 | `clippy rc=0` — **confirmed** |
| P2 `cargo test --release -p rexx-exec --test method_bodies` | 0; `regressions 0, other drift 0`; 16 passed | `method_bodies rc=0`; `regressions this run: 0. other drift from the committed table: 0.`; `16 passed` — **confirmed** |
| P3 `REXX_METHOD_BODIES_REFRESH=1 …` leaves `corpus/method-bodies.txt` byte-identical | empty `git diff --stat` | empty; same `0 / 0` line; `16 passed` — **confirmed** |
| Inversion: receiver `.MutableBuffer~new(1,2,3,4)` | exactly 51 rows move, all `MutableBuffer` instance, none `loud` after, class-arm `new` untouched, no other class | diffstat `51 insertions(+), 51 deletions(-)`; `51 instance answers` (`rc 163`), `1 class loud`; every `+`/`-` line is `MutableBuffer`; `other drift from the committed table: 51` — **confirmed** |

**One wrong reading on the way, corrected by running.** After P3 I read the newest
`target/tmp/method-bodies-*` staging directory and found `o = .MutableBuffer~new` in all 51 probes —
the old receiver. That directory was a 21:25 leftover: the run removes its own staging at
`method_bodies.rs:1170`, so a finished run leaves nothing to read. The liveness evidence is instead
the test binary's mtime (`23:47:37`, after the `23:46:47` edit) and
`/bin/grep -c -a -F ".MutableBuffer~new('abc')" target/release/deps/method_bodies-b7cd57c307f5027c`
→ `1`, plus the inversion above, which is the strongest of the three.

## The sweep the plan left open (review question 2)

`…/scratchpad/task0/sweep/sweep.py`: every instance row of each candidate class, run on the oracle
under the bare receiver and under a populated one; a row whose three descriptors differ is one the
populated receiver makes unfakeable. Prediction (`sweep/prediction.txt`): the `MutableBuffer`
control gives exactly the plan's ten; the rest unpredicted.

```text
MutableBuffer   51 rows  10 differ  length lower makeArray makeString space string subWords translate upper words   <- control, exactly the plan's ten
String         118 rows  43 differ  ('' against 'abc'; not a candidate -- its committed receiver already carries content)
Array           44 rows  17 differ  allIndexes allItems dimension dimensions first firstItem insert isEmpty items last lastItem makeArray makeString size sort stableSort toString
List            38 rows  10 differ  allIndexes allItems first firstItem insert isEmpty items last lastItem makeArray
Queue           43 rows  13 differ  allIndexes allItems first firstItem insert isEmpty items last lastItem makeArray peek pull size
CircularQueue   47 rows  16 differ  insert makeArray makeString string allIndexes allItems items first firstItem last lastItem sort stableSort isEmpty peek pull
Stem            27 rows   7 differ  allIndexes allItems hasItem index isEmpty items makeArray
Directory       31 rows   5 differ  allIndexes allItems isEmpty items makeArray
Table           24 rows   5 differ  allIndexes allItems isEmpty items makeArray
StringTable     29 rows   5 differ  allIndexes allItems isEmpty items makeArray
IdentityTable   24 rows   5 differ  allIndexes allItems isEmpty items makeArray
Set             24 rows   5 differ  allIndexes allItems isEmpty items makeArray
Bag             28 rows   6 differ  allIndexes allItems isEmpty items makeArray uniqueIndexes
Relation        28 rows   6 differ  allIndexes allItems isEmpty items makeArray uniqueIndexes
Properties      39 rows   5 differ  allIndexes allItems items makeArray isEmpty
```

**105 rows across the thirteen collection classes would sharpen.** Today 100 of them are `loud` and
5 are `answers` — `Array` `dimension items makeString size toString`. Those five were run under
`.Array~of(1,2,3)` on the oracle and on both engines (`array5/check.py`): all five agree (`1`, `3`,
`1\n2\n3`, `3`, `1\n2\n3`). **So sweeping all thirteen would move zero rows today**, the same
property Task 0 has.

**Not done here, deliberately.** Thirteen more entries also exempt thirteen more classes from
`check_receivers_match_table_c`, which exists to hold this table's receiver and table C's together.
That is a design cost and Moritz's call. Recommendation: yes, as its own commit ahead of Task 2, so
that the cost is paid while it still moves nothing and Task 2's table diff stays readable.

## Found for Task 1, and written into the plan's Task 1 text

* **`defaultSize` is observable, so the representation must carry it.** `setBufferSize(0)` shrinks
  capacity back to `defaultSize` (`MutableBufferClass.cpp:686`-`:691`); nothing else does. Oracle,
  `defaultsize2/`: `new(copies('x',400))` then `setBufferSize(0)` → `0 256`; with a second argument
  of `300` → `0 300`; `new('abc',500)` grown to `1203 2000` then `setBufferSize(0)` → `0 500`.
  `delete` and `setText('')` leave capacity where it was (`0 400`, `0 1000`).
* **Growth is `max(needed, 2 × capacity)`** (`ensureCapacity`, `:243`). Oracle: `new('',10)` under
  seven-byte appends reads `10 20 40 40 40 80 80 80`; `500` → `1000` → `2000`.
* **`String`'s method surface in this crate is not byte machinery to reuse.** `NATIVE_METHODS` has
  `LENGTH REVERSE SIGN UPPER NEW` for `String` and `'abc'~substr(2)` is rc 120 on both engines. The
  byte implementations are the builtin *functions* in `rexx-exec/src/builtin/string.rs` and
  `word.rs`, with `(interp, name, Args)` signatures. Whether they factor over a `&[u8]` the buffer
  can hand them is Task 1's first question, replacing the plan's "does `String` already have it".

## Corrections to the documents

* Spec §1 said the class-arm `new` row answers. It is `loud` (`method "MAKESTRING"`), because
  `say .MutableBuffer~new()` renders through `makeString`. All 52 rows are `loud`. Corrected in the
  spec.

## Gates

Run from `rust/` by a background job writing each status unpiped to a file as it goes, the commit
sha as its first line, with a pidfile. Started after the commit at `fdf4c6624`; the seven exits
below are read from that file (`…/scratchpad/task0/gates/status.txt`, first line `fdf4c6624`,
last line `finished`), G1 and G2 on a warm target directory.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |
| 7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |
