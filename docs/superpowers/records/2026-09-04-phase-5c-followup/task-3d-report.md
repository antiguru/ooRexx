# Task 3d report -- the caseless family

BASE `3403c98f70f3161acdf0ccaaea73c78de0a85c5a`. Scratch
`$S` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task3d`,
build scratch `$B` = `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3d`.

Every claim below is marked **measured** (with the command or file that produced
it) or **inferred** (with what it was inferred from).

---

## 1. What landed

`git diff --stat` at the commit: six tracked files changed, two new.

* `crates/rexx-exec/src/builtin/string.rs` -- `caseless_eq`,
  `caseless_find_forward`, `caseless_find_backward`,
  `caseless_count_occurrences`, `caseless_changestr_bytes`, and the three
  private shapes the two spellings share (`find_backward_with`, `count_with`,
  `changestr_with`). `find_backward`, `count_occurrences` and `changestr_bytes`
  become one-line calls into those; **no existing body changed**, which is the
  point of the wrapper shape rather than a comparator parameter on the public
  functions.
* `crates/rexx-exec/src/builtin/word.rs` -- `caseless_wordpos_bytes` and
  `wordpos_with`; `wordpos_bytes` becomes a one-line call into it.
* `crates/rexx-exec/src/dispatch.rs` -- eleven `NATIVE_METHODS` rows and their
  bodies, plus the shared `buffer_caseless_pos` and `buffer_caseless_wordpos`.
  `match_region` gains a `name` and a `matches` parameter and its one existing
  caller passes `b"MATCH"` and `<[u8]>::eq`; no other signature moved.
* `corpus/lang/mutablebuffer_caseless.rex` (84 lines, 38 lines of output),
  **filed in this commit** in `corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C`,
  with its `rexx-parse/tests/sourceline_oracle/mutablebuffer_caseless.txt`
  companion.
* `corpus/method-bodies.txt` -- refreshed; the eleven rows move `loud` to
  `answers` (§5).

**`corpus/refusal-sites.tsv` did not change and needed no re-derivation.** No
constructor was added to `error.rs`: every refusal these eleven raise already
had a raiser (§4), so no row moved. **Measured**: `--test refusal_sites` reads
5 passed and `git status --short` does not name the file.

The eleven rows, at `memory/Setup.cpp:1416`-`:1477`'s declared counts:
`CASELESSCHANGESTR` 3, `CASELESSCONTAINS` 3, `CASELESSCONTAINSWORD` 2,
`CASELESSCOUNTSTR` 1, `CASELESSENDSWITH` 1, `CASELESSLASTPOS` 3,
`CASELESSMATCH` 4, `CASELESSMATCHCHAR` 2, `CASELESSPOS` 3,
`CASELESSSTARTSWITH` 1, `CASELESSWORDPOS` 2. They sort contiguously between
`APPEND` and `CHANGESTR` in the table's own order.

---

## 2. Which twins share a scan and which do not

**The brief's trap is real and it is confined to one scan** -- the forward
search. `CASELESSPOS`, `CASELESSCONTAINS`, `CASELESSCOUNTSTR` and
`CASELESSCHANGESTR` reach it; every other name in the family is its
case-sensitive twin with the comparator folded, and that was measured rather
than assumed.

| caseless name | twin | shares the scan? | the probe |
|---|---|---|---|
| `CASELESSPOS` | `POS` | **no** | `$S/oracle/twins/t01_pos.rex`, oracle rc 0: over `.MutableBuffer~new('axan')`, `pos('an',1,3)` is `3` and `caselessPos('an',1,3)` is `0`; over `.MutableBuffer~new('AXAN')`, `pos('AN',1,3)` is `3` and `caselessPos('AN',1,3)` is `0`. At range 4 both are `3` |
| `CASELESSCONTAINS` | `CONTAINS` | **no** -- it is `caselessPos` > 0 | `t02_contains.rex` line A: `contains('an',1,3)` is `1`, `caselessContains('an',1,3)` is `0` |
| `CASELESSCOUNTSTR` | `COUNTSTR` | **the loop yes, the inner search no** | see below |
| `CASELESSCHANGESTR` | `CHANGESTR` | **the rebuild yes, the inner search no** | see below |
| `CASELESSLASTPOS` | `LASTPOS` | yes | `t03_lastpos.rex` A/B: the decoy `lastPos('345',8,4)`/`(8,5)` on `'Y3Y345YYYYYY'` is `0`/`4` and `caselessLastPos` answers `0`/`4` on the same arguments |
| `CASELESSWORDPOS` | `WORDPOS` | yes | `t07_wordpos.rex` |
| `CASELESSCONTAINSWORD` | `CONTAINSWORD` | yes | `t07_wordpos.rex` E/F |
| `CASELESSMATCH` | `MATCH` | yes | `t06_match.rex` A-E, including the three conversion-order cases |
| `CASELESSMATCHCHAR` | `MATCHCHAR` | yes | `t06_match.rex` I-K |
| `CASELESSSTARTSWITH` | `STARTSWITH` | yes | `t06_match.rex` F, H |
| `CASELESSENDSWITH` | `ENDSWITH` | yes | `t06_match.rex` G, H |

### 2.1 Why `caselessPos` is a different scan, and where the line falls

**Inferred from the C++**, read at `classes/support/StringUtil.cpp:206`
(`pos`) and `:268` (`caselessPos`). `pos` finds the needle's first byte with
`memchr` over `endpointer - haypointer`, and **recomputes that length from the
candidate it rejected**, so its window creeps one byte past `endpointer` per
rejection and `memcmp` then reads a match that begins or ends outside `range`.
`caselessPos` instead computes `count = _range - needle_length + 1` up front
and walks that many probes with `caselessCompare`; it cannot reach past the
window. `builtin/string.rs`'s `find_forward` doc block already recorded the
same asymmetry for the `String` class and cites DEVIATION 3.

**Measured, and the two are not merely different in principle.** A sweep over
every haystack of up to five bytes and every needle of up to three drawn from
`ab`, at every start in 1..4 and every range in 0..6 -- 24,696 argument sets --
found **122** where `pos` and `caselessPos` disagree
(`$S/oracle/sweep/s02.rex`, oracle rc 0). Since the data is all lower case,
every one of those 122 is the window and none is the fold.

**So `caseless_find_forward` is written as the C++'s own probe walk**, not as
`find_forward` over folded copies. M1 (§6) is that exact wrong implementation
and the witness catches it.

### 2.2 `CASELESSCOUNTSTR` and `CASELESSCHANGESTR`: the search differs, the
difference does not show

The C++'s `caselessCountStr` (`:1220`) and `caselessChangeStr`
(`classes/MutableBufferClass.cpp:1136`) are their twins with `pos` replaced by
`caselessPos` and `countStr` by `caselessCountStr`; everything else -- the
non-overlapping step, the three length branches, the capacity call -- is
copied verbatim. So this crate parameterises the search and shares the rest.

**But no argument distinguishes the two searches at the range those callers
use**, and that is measured rather than argued. Both pass the whole remaining
haystack as the range, at which the overrun position is always `len -
needle.len() + 1` -- one past the last start at which the needle fits, so the
match would have to run past the end. A sweep over the same 882 same-case
(haystack, needle) pairs found `countStr` and `caselessCountStr` agreeing on
**all 882**, and `changeStr`/`caselessChangeStr` likewise
(`$S/oracle/sweep/s01.rex`, oracle rc 0, `cases 882 diffs 0`).

**That zero has a control, and the first one I wrote was worthless.** Changing
the alphabet from `ab` to `aB` left both the haystacks and the needles drawn
from it, so no case mismatch was ever created and the control reported
`cases 882 diffs 0` -- identical to the finding it was meant to falsify
(`$S/oracle/sweepctl/s01ctl.rex`). The corrected control draws haystacks from
`ab` and needles from `AB` and reports `cases 882 diffs 768`
(`$S/oracle/sweepctl2/s01ctl2.rex`), so the comparison is live and the 0 is a
measurement.

**The implementation still uses the caseless search**, because that is what the
C++ calls and the equality is a property of these two callers' range rather
than of the two scans. M6 (§6) is the mutation that swaps it back.

### 2.3 The folding is ASCII and matches what this crate already does

**Measured by reading** `common/Utilities.hpp:52`: `toUpper` is
`isLower(c) ? c & ~0x20 : c`, with `isLower` at `:51` `c >= 'a' && c <= 'z'`
over a signed `char`, so no byte at or above `0x80` folds.
`StringUtil::caselessCompare` (`:654`) is `toUpper` on each side. That is
`u8::to_ascii_uppercase`'s rule, which `case_shift_bytes` and `option_letter`
already apply. **No finding here**: none of the eleven folds differently.

`caseless_eq` is spelled `eq_ignore_ascii_case`, which folds to lower rather
than to upper. **Inferred**: the two give the same equivalence, because each
maps exactly `{X, x}` together and moves nothing else. `clippy`'s
`manual_ignore_case_cmp` is what forced the spelling; the first version
compared `to_ascii_uppercase()` on both sides and was rejected at
`-D warnings`.

---

## 3. The witness

`corpus/lang/mutablebuffer_caseless.rex`, 84 lines, 38 lines of output.
**Written and confirmed on the oracle before the crate was touched**
(`$S/oracle/w/`, then widened once at `$S/oracle/w2/` -- §3.2).

**Measured**, `$S/run_oracle.sh $S/oracle/w2`: rc 0, stderr 0 bytes.
**Measured**, `$S/run_crate.sh $S/probes/w2`: rc 0 and stderr 0 bytes on both
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, and `cmp` reports stdout
byte-identical to the oracle's on both.

It contains no `say buf`, no concatenation of a buffer and no `makeString`; the
contents are read back with `buf~string` and `buf~length`, and the buffer
variables are `buf`, `low`, `upp`, `dec`, `same`, `same2`, `both`, `both2`,
`short`, `long`, `miss`, `cap`, `grow`, `grow2` -- none of them `b`.

The oracle's three descriptors, in full:

```
$ cat mutablebuffer_caseless.oracle.rc
0
$ wc -c mutablebuffer_caseless.oracle.err
0
$ cat mutablebuffer_caseless.oracle.out
len 27 MiXeD case MIXED Case mixed
pos 23 1 7 7 0
posd 12 23 12 0
pose 0 0 0
over1 3 0 0
over2 3 3 3
over3 3 0 0
over4 3 3 3
con 0 1 1 1 0
con2 1 0 0 0
last 0 12 23 23 0
last2 23 0
last3 0 0 4
cnt 1 3 2 2 0
cnt2 2 0 2
wp 5 1 2 2 0
wp2 1 3 0
cw 0 1 1 1 0
cw2 1 1 0
mat 0 1 1 1 0
mat2 0 1 0
mat3 1 0 0
mc 0 1 1 1 0
mc2 1 0 0
sw 0 1 1 1 0
sw2 0 0
ew 0 1 1 1 0
ew2 0 0
chg1 QXQXA
chg2 QXQXQ 5
chg3 aYaYA
chg4 aYaYA
chg5  0
chg6 ZZZZZZZZZ 9
chg7 aXaXA aXaXA
chg8 bbbbaA 6
grow1 42 45
grow2 404 512
```

### 3.1 What each printed value observes

Task 3c's M2 is why this is a separate pass: **a line that reaches a path is
not a line that observes it**, and the check has to be made against the
committed witness text rather than against the exploration program. The full
per-field table is at `$S/ctrl/witness-observations.md`, left as it was
written -- **two of its numbers are wrong and the corrections are here rather
than there**: its `grow1` row says a skipped capacity call answers `42 80`,
where reading `ensure_capacity` says it leaves the capacity at 10, and
`$S/ctrl/predictions.md`'s M6 row carries the same 80 and a `404 811` beside
it. §6.2 says where M6 actually lands. The load-bearing rows:

* **`over1` and `over3` fields 2-3 are the overrun pair, in both directions**
  -- `0 0` where field 1 prints `3`. A caseless scan built on `find_forward`
  prints `3 3` there. `over2`/`over4` print `3 3 3`, the range at which the two
  scans agree, which pins the difference to the window rather than the fold.
* Every name's line carries its twin's answer beside its own for the
  "both find" case, so "the fold over-matched" and "the fold did not happen"
  are separate readings rather than one.
* The defaults each have a field that moves if they are wrong: `posd` 1 (start
  default vs the explicit 2), `posd` 2 (a range default of `len - start`
  answers 0), `last` 4 (a start default of `len - 1` answers 12), `wp2` 2,
  `cnt2` 1 (a limit default of 1 answers 1), `mat2` 1 and `mat3` 2 (a length
  default of `other.len() - offset` answers 1).
* `mat3` 3 and `mc2` 3 are `caselessMatch(99, .nil)` and
  `caselessMatchChar(99, .nil)`: **the conversion order**. Converting the
  string argument before the start check raises 88.909 at rc 168 and the
  program stops, so this is a value that could not be printed by an
  implementation that read its arguments in the listed order.
* `grow1` (`42 45`) and `grow2` (`404 512`) are the growth past the current
  capacity read through `getBufferSize`, for both capacity origins -- an
  explicit `10` and the 256 default. `grow1` separates the candidate arguments
  to `ensure_capacity`, **inferred** from
  `rexx-core/src/body.rs:272`-`:280` (`needed = bytes.len() + added`, and
  `capacity = max(needed, capacity * 2)` only when `needed` exceeds it): over
  three bytes at capacity 10, passing the result's length 42 gives
  `max(45, 20)` = **45**, which is the measured value; passing the growth 39
  alone gives `max(42, 20)` = 42; and skipping the call leaves the capacity at
  10, since `replace_buffer_contents` does not touch that field. M6 is the
  mutation that reaches the third of those, and §6.2 records where it actually
  lands.

### 3.2 One gap found by that pass, and closed before the mutations ran

Nothing in the first version sent an **empty needle** to
`caseless_find_forward`: `cnt2` 2 is `caselessCountStr('')`, which returns at
`count_with`'s own guard before the scan. The `pose` line
(`caselessPos('')`, a start past the end, a zero range) was added for that, and
`con2` gained `caselessContains('')`.

**The widening happened before the mutation run rather than after**, at the
cost of killing a run that was two minutes in, so that every mutation is
measured against one witness text and no reading needs a version note.

### 3.3 Against BASE's binary -- the negative control

**Predicted before it ran** (`$S/ctrl/predictions.md`, C1): rc 120, stdout the
`len` line only, stderr naming `CASELESSPOS`.

**Measured**, `$B/base` a `git archive 3403c98f7 rust interpreter` extract
built with `CARGO_TARGET_DIR=$B/target-base` (build rc 0):

```
rc 120, both engines
stdout: len 27 MiXeD case MIXED Case mixed
stderr: rexx-exec: method "CASELESSPOS" of class "MutableBuffer" is not implemented (Phase 5)
```

**Confirmed.** The first `say` line runs because `buf~length` and `buf~string`
are Task 2's; the second dies at the first caseless send, after `buf~pos` has
already answered.

---

## 4. Refusals measured before they were written

Every refusal was run on the oracle first as a two-line program --
`buf = .MutableBuffer~new('aBcaBc')` then the probe -- from a fresh empty
directory (`$S/oracle/r/r001.rex`-`r070.rex`, runner `$S/run_oracle.sh`), then
on this build under both engines (`$S/run_crate.sh $S/oracle/r`, so both sides
ran the same absolute path and stderr is compared raw).

**Measured**: over 70 probes and two engines, **140 probe-engine pairs, 0
mismatching** on all three descriptors.

| probe | oracle | which raiser here |
|---|---|---|
| `caselessPos`, `caselessContains`, `caselessLastPos`, `caselessCountStr`, `caselessContainsWord`, `caselessWordPos`, `caselessChangeStr` sent nothing | 93.903 `Missing argument in method; argument 1 is required.`, rc 163 | `string_method_argument(.., 0)` → `Raised::missing_method_argument(1)` |
| `caselessMatch`, `caselessMatchChar` sent nothing | 93.903 argument 1 | `required_position_argument(.., 0)` |
| `caselessMatch(1)`, `caselessMatchChar(1)`, `caselessChangeStr('a')` | 93.903 `argument 2 is required` | `string_method_argument(.., 1)` |
| `caselessStartsWith`, `caselessStartsWith(,)`, `caselessEndsWith`, `caselessEndsWith(,)` | 88.901 `Missing argument; argument match is required.`, rc 168 | `named_string_argument(.., 0, "match")` |
| `caselessStartsWith(.nil)`, `caselessEndsWith(.nil)` | 88.909 `Argument match must have a string value.`, rc 168 | the same, through `required_string_named_argument` |
| `caselessPos(.nil)`, `caselessContains(.nil)`, `caselessLastPos(.nil)`, `caselessCountStr(.nil)`, `caselessContainsWord(.nil)`, `caselessWordPos(.nil)`, `caselessChangeStr(.nil)` | 88.909 `Argument 1 must have a string value.`, rc 168 | `required_string_argument(.., 1)` |
| `caselessMatch(1, .nil)`, `caselessMatchChar(1, .nil)`, `caselessChangeStr('a', .nil)` | 88.909 `Argument 2` | `required_string_argument(.., 2)` |
| `caselessPos('a', 0)` / `('a', .nil)` / `('a', '1.5')`, `caselessContains('a', 0)`, `caselessLastPos('a', 0)`, `caselessContainsWord('a', 0)` / `('a', .nil)`, `caselessWordPos('a', 0)`, `caselessMatch(0)` / `(.nil)` / `(1, 'a', 0)` / `(1, 'a', .nil)`, `caselessMatchChar(0)` / `(.nil)` | 93.924 `Invalid position argument specified; found "0"` / `"The NIL object"` / `"1.5"`, rc 163 | `optional_position_argument` and `required_position_argument` |
| `caselessPos('a', 1, -1)` / `('a', , -1)` / `('a', 1, 'x')`, `caselessContains('a', 1, -1)`, `caselessLastPos('a', 1, -1)` / `('a', , -1)`, `caselessMatch(1, 'a', 1, -1)` / `(1, 'a', 1, 'x')` | 93.923 `Invalid length argument specified; found "-1"` / `"x"`, rc 163 | `optional_length_argument` |
| `caselessChangeStr('a','b',-1)` / `('a','b','x')` / `('a','b',.nil)` / `('a','b','1.5')` | **93.906** `Method argument 3 must be zero or a positive whole number; found "-1"`, rc 163 | `optional_non_negative_argument` -- `changeStr`'s raiser, not a length one |
| `caselessCountStr('a','b')`, `caselessStartsWith('a','b')`, `caselessEndsWith('a','b')` | 93.902 `Too many arguments in invocation of method; 1 expected.`, rc 163 | `Arity::Fixed(1)`, the table |
| `caselessContainsWord('a',1,2)`, `caselessWordPos('a',1,2)`, `caselessMatchChar(1,'a',1)` | 93.902 `2 expected` | `Arity::Fixed(2)` |
| `caselessPos('a',1,1,1)`, `caselessContains('a',1,1,1)`, `caselessLastPos('a',1,1,1)`, `caselessChangeStr('a','b',1,1)` | 93.902 `3 expected` | `Arity::Fixed(3)` |
| `caselessMatch(1,'a',1,1,1)` | 93.902 `4 expected` | `Arity::Fixed(4)` |

**No caseless method uses the named 88.910/88.911/88.912 family.** Task 3c
found `replaceAt` reaching the named position/length/pad overloads where its
neighbours use positional ones, and the brief asked which overload each of
these uses: the answer is that only `caselessStartsWith` and `caselessEndsWith`
reach the named family at all, and only its *string* member
(`stringArgument(other, "match")`, `classes/MutableBufferClass.cpp:1557`,
`:1592`), whose 88.901/88.909 pair `named_string_argument` already provides.
Every other caseless argument is positional. **So no `error.rs` constructor was
added and `refusal-sites.tsv` did not move** -- the first commit in this family
for which that is true.

**Four probes answer at rc 0 where the argument list predicts a refusal**, each
the C++ returning before it converts a later argument, and each identical to
its case-sensitive twin's (Task 3b §2.3 measured the twins):

| probe | oracle | C++ |
|---|---|---|
| `caselessMatch(99, .nil)` | `0`, rc 0 | `:1510` tests `_start > getLength()` before `stringArgument(other)` at `:1514` |
| `caselessMatch(7, 'a', 0)` | `0` | the same test, before `optionalPositionArgument(offset_)` |
| `caselessMatch(1, 'abc', 4, -1)` | `0` | an explicit offset past `other` returns before `optionalLengthArgument(len_)` |
| `caselessMatchChar(99, .nil)` | `0` | `:1711`, the same shape, before `:1715` |

Semantics measured the same way and agreeing on both engines
(`$S/oracle/twins*`): `caselessChangeStr('a','b',0)` answers the receiver
unchanged; `caselessStartsWith('')` and `caselessEndsWith('')` are `0`;
`caselessLastPos('C', 20)` on six bytes is `7` (the start is capped, not
refused); `caselessMatchChar(6,'')` is `0`.

---

## 5. `corpus/method-bodies.txt` row moves

**Predicted before the refresh ran** (`$S/ctrl/predictions.md`, C2, written
from the oracle probes at `$S/oracle/r/`).

**The brief's expectation for `caselessChangeStr` is falsified and the
prediction says so before the run.** The brief says the row "stays `loud` with
the evidence moving to `MAKESTRING`, as five of Task 3c's did". It does not:
sent no arguments, `caselessChangeStr` raises 93.903 at
`stringArgument(needle, ARG_ONE)` and never reaches the receiver it would have
answered, so the probe's own `say` is never evaluated. **Measured on the oracle
before any code was written**, `$S/oracle/r/r061`: rc 163, 93.903 argument 1.

| rows | from | to | evidence |
|---|---|---|---|
| `caselessChangeStr`, `caselessContains`, `caselessContainsWord`, `caselessCountStr`, `caselessLastPos`, `caselessMatch`, `caselessMatchChar`, `caselessPos`, `caselessWordPos` | `loud`, the method's own name | `answers` | `rc 163` -- 93.903 on both sides |
| `caselessEndsWith`, `caselessStartsWith` | `loud` | `answers` | `rc 168` -- 88.901 `argument match` on both sides |
| every other row, `MutableBuffer`'s others included | unchanged | unchanged | -- |

**Measured**: `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec
--test method_bodies` reads rc 0, 16 passed,
`regressions this run: 0. other drift from the committed table: 11.`
(`$S/ctrl/fast/mb.err`), and `git diff -- corpus/method-bodies.txt` is those
eleven rows and no other. **No row moves to `diverge`**, and
`/bin/grep -ac diverge corpus/method-bodies.txt` reads **7**, unchanged.
Prediction confirmed row for row, rc for rc.

---

## 6. Controls, each predicted before it ran

Predictions in `$S/ctrl/predictions.md`, written before the runs they name; the
per-field observation table in `$S/ctrl/witness-observations.md`, written
against the committed witness.

| id | control | prediction | reading |
|---|---|---|---|
| C1 | the witness against BASE's binary | rc 120 at the first caseless send, stdout the `len` line, stderr naming `CASELESSPOS` | **confirmed**, §3.3, both engines |
| C2 | `method-bodies.txt` moves | eleven rows `loud` → `answers`, nine at rc 163 and two at rc 168, `diverge` stays 7 | **confirmed** row for row, §5 |
| C3 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | rc 0, `362 of 362 matching` | **confirmed**: rc 0, `362 of 362 matching`, 18 passed (`$S/ctrl/fast/corpus.err`) |
| C4 | the pin's inversion -- the witness line deleted from `EXPECTED_SUBSET_5C` while it stays in `phase-5c.txt` | `--test coverage` rc 101, `phase_5c_subset_matches_the_committed_list` failing | **confirmed**: rc 101, `19 passed; 1 failed`, the panic at `coverage.rs:1411`; restored from `$S/ctrl/coverage.rs.good` and `cmp`-identical, then rc 0 and 20 passed (`$S/ctrl/c4.out`, `$S/ctrl/c4-after.out`) |

### 6.1 The mutation instrument had to be fixed first

**The first mutation run recorded a sha that was measuring nothing, and the
controller caught it.** The harness read `sha256sum target/mutation/rexx-run`
straight after `cargo test --profile mutation -p rexx-exec --lib`. `--lib`
builds the library and its unit-test binary and **does not build `rexx-run`**,
so the value read was a leftover: **measured**, that file's mtime was
`2026-09-05 11:37:13`, inside Task 3c's run window, and its sha `846162ab...`
is the value Task 3c's report records for its own M2.

**The deeper fault is that it was the wrong artifact either way.**
`corpus.rs:256` runs this crate **in process** --
`watchdog::run_bounded(path_str, text, rexx_exec::Invocation::none())` -- so
the differential never spawns `rexx-run` and that binary's sha says nothing
about what the reading measured. The mutation reaches the test through the
relinked corpus test executable.

**So the instrument is now `sha256sum target/mutation/deps/corpus-*`**, read
after the corpus run rather than after `--lib`, and §6.2 records whether two
different mutations produce two different values -- which is the check that
says whether it can see anything at all.

**Cross-task finding, for the controller to rule on.** Task 3c's harness read
the same file at the same point. `method_bodies.rs:501` uses
`env!("CARGO_BIN_EXE_rexx-run")`, which *does* make cargo build that binary, so
Task 3c's per-iteration value was most plausibly the binary left by the
*previous* iteration's `method_bodies` run rather than the current mutant's.
That would produce six distinct-looking values that are nonetheless off by one
iteration, and its sentence "six distinct values, so no run measured a stale
binary" does not follow from them. **Its red and green readings are
unaffected**, because those came from the same in-process corpus differential.

### 6.2 Mutations

Predictions written before any mutation ran (`$S/ctrl/predictions.md`). The run
is `$S/mut/run_all.sh`, one mutation at a time, each applied by
`$S/mut/mutate.py`, which refuses unless the replaced text occurs exactly once
(**measured before the run**: all seven anchors occur exactly once in their
files). Every build is `--profile mutation` (`target/mutation/`); after each
mutation the edited file is restored from a copy under `$S/mut/good/`,
`cmp`-checked and `touch`ed. Every command runs under `memcap 8G`.

The checks per mutation are
`cargo test --profile mutation -p rexx-exec --lib --no-fail-fast`,
`REXX_CORPUS_GATE=1 … --test corpus --no-fail-fast`,
`… --test method_bodies --no-fail-fast`, and -- where the new witness is the
claimed catcher -- **the same STRICT corpus again with
`lang/mutablebuffer_caseless.rex` taken out of `corpus/phase-5c.txt`**, because
"can fail" is not "adds coverage".

**That last run's exit status argues against its own reading, and the reading
is the right one.** It exits **101**, and a `361 of 361 matching` line beside
an rc 101 looks like a contradiction. It is not: the failing test is
`every_lang_program_is_run_or_named_unfiled` (`corpus.rs:845`), which fires
because pulling the line out of `phase-5c.txt` leaves the program in
`corpus/lang/` named by no subset file -- bookkeeping, not the differential.
`corpus_differential` passes in every one of those runs. **Measured**, e.g.
`$S/mut/M1/corpus-nw.out`:

```
test every_lang_program_is_run_or_named_unfiled ... FAILED
test corpus_differential ... ok
test result: FAILED. 17 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out
---- every_lang_program_is_run_or_named_unfiled stdout ----
panicked at crates/rexx-exec/tests/corpus.rs:845:5:
{"lang/mutablebuffer_caseless.rex"} are in …/corpus/lang and named by no phase
subset file, so nothing compares them against the oracle and the differential's
headline does not count them. File each one, or name it in unfiled.txt with the
reason it is not run
```

So the reading is the `N of M matching` line and never the exit status.

**M1b is the cleaner control the same measurement can be taken with**, run
afterwards on M1's mutation: the witness line is moved from `phase-5c.txt`
into `corpus/unfiled.txt` -- which `read_unfiled` accepts as
`path<TAB>reason` and which `corpus.rs:845` is satisfied by, provided no
subset file also names it -- so the whole binary is expected to exit 0 and its
matching line to say the same thing the status does. §6.3 records it.
(`phase_5c_subset_matches_the_committed_list` would then disagree with
`EXPECTED_SUBSET_5C`, but that test lives in `--test coverage`, which no
mutation check runs.)

| id | site | mutation | predicted catcher | reading |
|---|---|---|---|---|
| M1 | `caseless_find_forward` (`builtin/string.rs`) | the probe walk replaced by `find_forward` over folded copies -- **the wrong implementation the brief named** | STRICT corpus only, on the new witness | **confirmed**: lib rc 0, corpus rc 101 `361 of 362 matching` on `lang/mutablebuffer_caseless.rex`, method_bodies `regressions 0, drift 0`; without the witness `361 of 361` |
| M2 | `caseless_eq` (`builtin/string.rs`) | `to_ascii_uppercase` on the left operand only | STRICT corpus only | **confirmed**: `361 of 362`; without the witness `361 of 361` |
| M3 | `caseless_find_backward` (`builtin/string.rs`) | `find_backward_with`'s comparator becomes `<[u8]>::eq` | STRICT corpus only | **confirmed**: `361 of 362`; without the witness `361 of 361` |
| M4 | `caseless_wordpos_bytes` (`builtin/word.rs`) | `wordpos_with`'s comparator becomes `<[u8]>::eq` | STRICT corpus only | **confirmed**: `361 of 362`; without the witness `361 of 361` |
| M5 | `buffer_caseless_pos` (`dispatch.rs`) | the start default `unwrap_or(1)` becomes `unwrap_or(2)` | STRICT corpus only | **confirmed**: `361 of 362`; without the witness `361 of 361` |
| M6 | `native_mutable_buffer_caselesschangestr` (`dispatch.rs`) | `caseless_count_occurrences` becomes `count_occurrences` in the capacity arithmetic | STRICT corpus only, and the prediction said it **might** come back green | **confirmed red**, `361 of 362`; without the witness `361 of 361`. The prediction named the wrong field and the wrong numbers -- see §6.4 |
| M7 | `count_with` (`builtin/string.rs`) | the non-overlapping step `next = found - 1 + needle.len()` becomes `next = found` | red on the existing suite, **adding no coverage** | **confirmed, and the reading is stronger than the prediction**: `--lib` rc **101** (it breaks `COUNTSTR`'s own core, which has unit tests) and the corpus's one diverging program is **`lang/mutablebuffer_readers.rex`**, not the new witness -- so the new witness does not move under M7 at all |

`--test method_bodies` is `regressions this run: 0. other drift from the
committed table: 0.` under all seven: every one of the eleven rows raises on
its first argument before any scan, comparator or default is reached.

**The unmutated control, run last over the restored tree**: STRICT corpus
rc **0**, `362 of 362 matching`. And `target/release/rexx-run` reads
`51b13c9859e93dcd…` before the run, after the full rebuild at the end of it,
and again after the field runs of §6.4 -- which is also the control that every
restore was byte-exact.

**The corpus test executable's sha256, per run**: `7c7bdd3074fa3f24`,
`4add7ab6769ab124`, `a9f7af5fdec3e8ac`, `382a4c776c3d766f`,
`bcd2f6ec8d4e0535`, `6a7161668ce7d54f`, `79e138d2cc6a2761`, and
`a0dc754616e85a22` for the unmutated control -- eight distinct values for
eight distinct sources, so the instrument discriminates every mutant rather
than reporting a leftover.

### 6.3 M1b -- the clean without-witness control

**Predicted before it ran**: moving the witness line from `phase-5c.txt` into
`corpus/unfiled.txt` satisfies `corpus.rs:845` (`read_unfiled` at `:758` takes
`path<TAB>reason`, and the `both.is_empty()` assertion above it only forbids
a program named by *both*), so the binary should exit **0** and its matching
line say the same thing its status does.

**Measured**, M1's mutation re-applied (`$S/mut/status-fields.txt`):
rc **0**, `361 of 361 matching`, **0 failing tests**,
`test result: ok. 18 passed; 0 failed; 1 ignored`. **Confirmed**, and this is
the shape a later task should use.

### 6.4 Which field of the witness each mutation moves

**The six reds all name the same program**, which is consistent with six
independent catches and equally consistent with one field doing all the work.
The corpus report truncates both stdout strings, so it cannot tell those
apart. **Predicted first, from the code, for all six**
(`$S/ctrl/predicted-fields.md`, written before any of the three runs below),
then measured for three chosen as the ones predicted to move different lines.

Each measurement re-applies its mutation, builds
`cargo build --profile mutation -p rexx-exec --bin rexx-run`, runs the witness
through that binary on both engines and diffs its stdout against the oracle's.

| id | predicted first-moving line | measured |
|---|---|---|
| M1 | `over1` (2,3: `0 0` → `3 3`), then `over3` and `con2` 2; **not** `posd` 4 or `pose` 3 | **confirmed exactly**: the diff is `over1 3 0 0` → `3 3 3`, `over3 3 0 0` → `3 3 3`, `con2 1 0 0 0` → `1 1 0 0`, and nothing else. Binary sha `828c5ebcfd544393`, rc 0 on both engines |
| M4 | `wp` (2: `1` → `5`) | **confirmed, and three more lines move than the prediction named**: `wp` `5 1 2 2 0` → `5 5 2 2 0`, `wp2` `1 3 0` → `0 5 0`, `cw` `0 1 1 1 0` → `0 0 1 1 0`, `cw2` `1 1 0` → `0 0 0`. Binary sha `d4bda700ffeed3e5` |
| M6 | **only** `grow2` (`404 512` → `404 256`); `grow1`, `chg6` and `chg8` do not move | **confirmed exactly**: the diff is one line, `grow2 404 512` → `404 256`. Binary sha `06bc350ea8ef0dda` |
| M2 | `pos` (2: `1` → `0`) | **predicted from the code, not measured** |
| M3 | `last` (2: `12` → `0`; field 4 stays `23`) | **predicted from the code, not measured** |
| M5 | `pos` (2: `1` → `12`) | **predicted from the code, not measured** |

**So the witness is not one field wearing six hats.** The three measured
mutations move three disjoint groups of lines -- the overrun block, the word
block, and one capacity field -- and the three predicted ones name `pos` and
`last`, which neither measured group touches. Five distinct first-moving lines
across M1-M6.

**What the M6 prediction got wrong, and what that cost.**
`$S/ctrl/predictions.md`'s M6 row predicted a red at `grow1` with `42 45`
becoming `42 80`, and offered `404 811` for `grow2`; both numbers are
arithmetic errors and both fields are wrong. Reading
`rexx-core/src/body.rs:272`-`:280` gives the right answer -- a skipped
`ensure_capacity` leaves the capacity where it was, and the case-sensitive
count of `'B'` in `'aBc'` is 1, the same as the caseless one, so `grow1`
cannot move at all. `$S/ctrl/predicted-fields.md` says `grow2` alone and the
measurement agrees. **The prediction files are left as they were written**;
correcting a prediction after its run is what makes predictions worthless.

---

## 7. Fast checks before the commit

Run from `rust/` in the foreground, each status read unpiped
(`$S/ctrl/fast/`).

All of them re-read in this order **after** the mutation runs and the field
runs of §6.4, over the restored tree (`$S/ctrl/fast2/`).

| check | reading |
|---|---|
| `cargo fmt --all --check` | rc 0 (rc 1 on the first pass, `cargo fmt --all` applied, then rc 0) |
| `cargo clippy --workspace --all-targets -- -D warnings` | rc 0 (rc 101 on the first pass, `clippy::manual_ignore_case_cmp` on `caseless_eq`; §2.3) |
| `cargo build --release -p rexx-exec --bin rexx-run` | rc 0, sha256 `51b13c9859e93dcd…` |
| `cargo test --release -p rexx-exec --lib` | rc 0, 779 passed, 0 failed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | rc 0, `362 of 362 matching`, 18 passed |
| `cargo test --release -p rexx-exec --test refusal_sites` | rc 0, 5 passed -- **no re-derivation needed**, `corpus/refusal-sites.tsv` is not in `git status` |
| `cargo test --release -p rexx-exec --test coverage` | rc 0, 20 passed |
| `cargo test --release -p rexx-exec --test builtin_status` | rc 0, 19 passed, `corpus/builtin-status.txt` unchanged |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | rc 0, 1 passed |
| `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies` | rc 0, 16 passed, `regressions this run: 0. other drift from the committed table: 11.` |
| `cargo test --release -p rexx-exec --test method_bodies` (no refresh, after the run) | rc 0, 16 passed, `regressions this run: 0. other drift from the committed table: 0.` |

Every run count is non-zero, which is the check `cargo test <name>` needs
because it exits 0 when it matches nothing.

`git status --short` then names six modified tracked files and three new ones
(this report among them), and `rust/Cargo.lock` is not among them.
`crates/rexx-exec/src/builtin/string.rs` carries the corrected
`common/Utilities.hpp:52` citation, which is the check that the mid-run
collision of §8 did not cost the fix.

---

## 8. Corrections to prose and briefs met on the way

* **The brief's `CASELESSCHANGESTR` method-bodies expectation is wrong**, §5:
  sent no arguments it raises before it can answer the receiver, so the row
  moves to `answers` rather than staying `loud` at `MAKESTRING`.
* **"Fold-then-call-the-existing-core" is right for eight of the eleven names
  and wrong for three**, §2. The brief warned it was a hypothesis; the answer is
  that the hypothesis holds everywhere except the forward search, and it fails
  there for a reason (`pos`'s rescan) that has no counterpart in `lastPos`,
  `wordPos`, `primitiveMatch` or `matchChar`.
* **Task 3c's mutant-sha claim does not follow from its own numbers**, §6.1.
* `clippy::manual_ignore_case_cmp` forces `eq_ignore_ascii_case` over a
  `to_ascii_uppercase` comparison; §2.3 records why the two are the same
  equivalence rather than leaving the substitution unexplained.
* **Four C++ line citations were wrong and were caught by running them**, not
  by rereading: `Utilities::toUpper` is `common/Utilities.hpp:52` and `:51` is
  `isLower`; `caselessMatch`'s start test is `:1510` and `:1509` a comment;
  `caselessMatchChar`'s is `:1711` and `:1709` a comment; and
  `stringArgument(other, "match")` is `:1557` and `:1592`, where `:1556` and
  `:1591` are the opening braces. The check that found them was
  `sed -n "<line>p"` on every citation in the diff and the report, which is
  cheap and is the only thing that separates a true claim pinned to the right
  line from one pinned to its neighbour.

**One process mistake, recorded because it was mine and self-inflicted.** The
`Utilities.hpp:52` fix was applied to `builtin/string.rs` **while this task's
own mutation run held the tree**, which is the collision the phase already has
a rule about. No reading was harmed -- the edit is a comment and M2's
`361 of 362 matching` had already been written -- but the restore would have
silently discarded it, because the harness restores from `$S/mut/good/`. The
fix was therefore carried into the pristine copy as well, its three remaining
anchors re-checked for uniqueness against the corrected copy, and the tree left
alone until the status file said `finished`. §7 confirms the committed file
carries the `:52` spelling.

---

## 9. Files created outside the tree

* `$B/base/` -- a `git archive 3403c98f7 rust interpreter` extract;
  `$B/target-base/` -- its build (`$S/base-build.sh`, `.out`, `.err`, `.pid`,
  `.status`).
* `$S/run_oracle.sh`, `$S/run_crate.sh` -- the two runners; every
  `$S/**/run.*` directory is a fresh empty run directory one of them made.
* `$S/oracle/twins/`, `$S/oracle/twins2/` -- the twin-pair probes (§2) and their
  three descriptors, oracle and both engines.
* `$S/oracle/sweep/`, `$S/oracle/sweepctl/`, `$S/oracle/sweepctl2/` -- the
  brute-force sweeps, the control that could not have run, and the corrected
  control (§2.2).
* `$S/oracle/r/` -- the 70 refusal probes and their three descriptors, oracle
  and both engines.
* `$S/oracle/w/`, `$S/oracle/w2/` -- the witness before and after the widening;
  `$S/probes/w/`, `$S/probes/w2/` -- the same against this build;
  `$S/probes/base/` -- against BASE's binary.
* `$S/srclines/` -- the `sourceline_oracle` driver and its two stderr files.
* `$S/ctrl/` -- `predictions.md`, `witness-observations.md` and
  `predicted-fields.md`, each written before the run it names;
  `coverage.rs.good`, `method-bodies.txt.base`, `release-sha.txt`, `c4.out`,
  `c4.err`, `c4-after.out`, and the two fast-check directories `fast/` and
  `fast2/` (before and after the mutation runs).
* `$S/mut/` -- `mutate.py`, `spec.tsv`, `pairs/`, `good/` (the pristine
  copies); the three runners `run_all.sh`, `run_fields.sh` and the two
  superseded drafts `run_m1b.sh`, `run_m6b.sh` (folded into `run_fields.sh`
  and never run under their own names); `status.txt`, `status-fields.txt`,
  `pid`, `pid-fields`; the per-mutation directories `M1/`-`M7/`, the three
  field runs `M1b/`, `M4b/`, `M6b/`, the clean control `M1bctl/`; and
  `clean-corpus.{out,err}`, `rebuild.{out,err}`, `rebuild2.{out,err}`,
  `run_all.{out,err}`, `run_fields.{out,err}`.
* `$S/gates/` -- the gate runner, its status file and its pidfile.
* `$S/base-{string,word,dispatch}.rs` -- BASE copies, used only to check that
  `match_region` is the sole pre-existing signature that moved (§1).
* `$S/commit-msg.txt` -- the commit message, `git commit -F`'s argument.
* `$S/ps.txt`, `$S/ps2.txt` -- two process-list snapshots, taken while
  stopping the first mutation run and while confirming none survived.

Enumerated from the filesystem rather than from memory. Nothing under `$S` or
`$B` was deleted.

---

## 10. Open questions

1. **`caseless_find_forward` has no DEVIATION-3 counterpart, and that is a
   property of the C++ rather than a choice here.** `find_forward` declines to
   invent the terminator byte the oracle reads past the end of a `RexxString`;
   the caseless scan never probes that position at all, so there is nothing to
   decline. Whether `MutableBuffer`'s own data is even NUL-terminated -- and so
   what the oracle's `pos` reads there for a buffer rather than a string -- was
   not established and no probe here goes near it.
2. **`corpus/method-bodies.txt` cannot see argument conversion order**, carried
   unchanged from Task 3b §7. This commit's witness does probe it, at `mat3` 3
   and `mc2` 3, but only for the two methods that have it.
3. **The mutant-sha instrument, §6.1**: Task 3c's report may need a correction
   commit, which is Moritz's call and not made here.
4. Carried from Task 3a §6.1, Task 3b §7 and Task 3c §7, unchanged:
   `MutableBuffer~verify` answers `counted` on every path while the builtin's
   past-the-end zero is an untagged text; `refusal-sites.tsv` cites constructor
   definitions by line; and `MutableBuffer::translate` numbers its range
   `ARG_FOUR`.
5. `makeString`, `makeArray` and `subWords` stay unbound until the conversion
   commit. **No row in this family is left `loud` by that**, unlike Task 3c's
   five.

---

## Gates

Run from `rust/` by `$S/gates/run.sh` in the background, each status written
unpiped to `$S/gates/status.txt` as it goes -- the commit sha as its first
line, `finished` as its last -- with the pidfile `$S/gates/pid` beside it and
each gate's descriptors in `$S/gates/g<N>.{out,err}`.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **G1** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **G2** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **G3** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **G4** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **G5** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G6** |
| 7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G7** |
