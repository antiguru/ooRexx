# Task 3b report -- the readers, over the cores

The second Task 3 commit of `docs/superpowers/plans/2026-09-04-phase-5c-followup.md`: eighteen
`MutableBuffer` readers bound over the cores Task 3a landed, one corpus witness filed with them.
BASE is `7e03fbaf2`. `$S` below is
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task3b`
and `$B` is `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3b`. Every claim is
marked **measured** (with the command or file that produced it) or **inferred**.

## 1. What landed

The agent died on a usage limit at 09:42 with everything below §3 written and nothing committed;
the controller backed the tree up (`$S/wip-backup/`, `wip.patch` plus the eight files), rebuilt
`target/release/rexx-run` (sha256 `695dd78eab08c31b…`, the value the agent had recorded), and
finished the task inline. `git diff --stat` at the commit: seven tracked files, three new.

* `crates/rexx-exec/src/dispatch.rs` -- eighteen `MutableBuffer` instance rows with bodies over
  Task 3a's cores: `[]`, `contains`, `containsWord`, `countStr`, `lastPos`, `match`, `matchChar`,
  `pos`, `startsWith`, `subChar`, `substr`, `subWord`, `verify`, `word`, `wordIndex`,
  `wordLength`, `wordPos`, `words`; the method-layer argument helpers
  `required_position_argument`, `string_method_argument`, `pad_method_argument` and
  `option_method_argument`; the shared `buffer_pos`, `buffer_wordpos` and `match_region`.
* `crates/rexx-exec/src/error.rs` -- `Raised::incorrect_pad` (93.922), the method layer's pad
  refusal, where the builtin layer's is 40.23.
* `crates/rexx-exec/src/builtin/string.rs` -- `find_forward`, `find_backward` and
  `count_occurrences` widened to `pub(crate)`; no body changed (the diff is those three
  signatures).
* `corpus/lang/mutablebuffer_readers.rex` (38 lines, 21 lines of output), filed in
  `corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C`, with its
  `rexx-parse/tests/sourceline_oracle/mutablebuffer_readers.txt` companion.
* `corpus/method-bodies.txt` -- refreshed: the eighteen rows above move `loud` to `answers` (§4).
* `corpus/refusal-sites.tsv` -- re-derived from `refusal_sites.rs`'s own panic, not edited by
  hand (`$S/ctrl/fast/misc.out`, the two lists parsed and applied by a script; the table before,
  `$S/ctrl/refusal-sites.before.tsv`): every error.rs definition below `incorrect_pad` moves
  down by the 13 lines its doc block and body occupy, the same 64 `(kind, name, surface)` keys on
  both sides of the disagreement, and one row is new -- `Raised incorrect_pad send
  error.rs:1343 agrees yes 93.922`, witness `.MutableBuffer~new('abcabc')~substr(1, 2, 'xx')`,
  probed from a fresh directory on the oracle and both engines before the row was written
  (`$S/ctrl/oracle-pad/`: rc 163 on all three, stdout empty, stderr `cmp`-identical to the
  oracle's). `--test refusal_sites` then reads 5 passed.

## 2. The witness and the oracle

### 2.1 `corpus/lang/mutablebuffer_readers.rex`

Written and run on the oracle before any crate file was touched. **Measured**, from a fresh empty
directory under the plan's wrapper (`$S/oracle/w1/`, descriptors `$S/oracle/witness.{out,err,rc}`):
rc 0, stderr 0 bytes, 21 lines of stdout. The program:

```rexx
/* The readers over a MutableBuffer's contents -- substr, [], pos, lastPos,
   countStr, verify, subWord, word, wordIndex, wordLength, words, wordPos,
   contains, containsWord, startsWith, match, matchChar and subChar -- each
   answer what the oracle answers, and none of them changes the buffer.
   Rendering the buffer itself -- say, concatenation, makeString -- is
   deliberately absent. */
buf = .MutableBuffer~new('abcabc')
/* substr pads past the end; [] defaults its length to one byte, caps it at
   the end and never pads. */
say buf~substr(2) '|' buf~substr(2, 3) '|' buf~substr(5, 4) '|' buf~substr(5, 4, '-') '|' buf~substr(9, 2, '*') '|' buf~substr(9) '|' buf~substr(1, 0)
say buf[2] '|' buf[2, 3] '|' buf[5, 10] '|' buf[7] '|' buf[7, 2] '|' buf[1, 0] '|' buf[6]
/* pos and lastPos with a start and a range; contains is pos as a truth value. */
say buf~pos('bc') buf~pos('bc', 3) buf~pos('bc', 3, 2) buf~pos('bc', 3, 4) buf~pos('x') buf~pos('') buf~pos('abc', 7) buf~pos('c', 6)
say buf~lastPos('bc') buf~lastPos('bc', 4) buf~lastPos('bc', 4, 2) buf~lastPos('bc', 4, 3) buf~lastPos('x') buf~lastPos('abc', 99) buf~lastPos('a', 1)
say buf~contains('ca') buf~contains('ca', 4) buf~contains('bc', 3, 2) buf~contains('bc', 3, 4) buf~contains('x') buf~contains('')
say buf~countStr('bc') buf~countStr('x') buf~countStr('abcabc') .MutableBuffer~new('aaaa')~countStr('aa')
/* verify: an empty reference, both options in either case, a range, and a
   start past the end. */
say buf~verify('abc') buf~verify('ab') buf~verify('ab', 'M') buf~verify('x', 'N', 2) buf~verify('ab', 'N', 2, 1) buf~verify('ab', 'n', 2, 2) buf~verify('c', 'match', 2)
say buf~verify('', 'M') buf~verify('', 'N') buf~verify('', 'N', 3) buf~verify('abc', 'N', 9) buf~verify('', 'N', 9) buf~verify('abc', 'M', 6, 1)
/* startsWith, match and matchChar: the empty string never matches, and a
   start or an offset off the end is 0 rather than a refusal. */
say buf~startsWith('ab') buf~startsWith('abcabc') buf~startsWith('abcabcd') buf~startsWith('b') buf~startsWith('')
say buf~match(1, 'abc') buf~match(4, 'abc') buf~match(2, 'abc') buf~match(4, 'xabc', 2) buf~match(4, 'xabcx', 2, 3) buf~match(4, 'xabcx', 2, 4) buf~match(6, 'c') buf~match(6, 'cd')
say buf~match(7, 'a') buf~match(1, '') buf~match(1, 'abc', 4) buf~match(1, 'abc', 2, 3) buf~match(1, 'abc', 4, 0) buf~match(3, 'zc', 2, 1)
say buf~matchChar(1, 'xa') buf~matchChar(2, 'xa') buf~matchChar(6, 'c') buf~matchChar(7, 'c') buf~matchChar(3, '') buf~matchChar(5, 'abc')
say buf~subChar(1) '|' buf~subChar(3) '|' buf~subChar(6) '|' buf~subChar(7)
/* The word readers, on a buffer with leading, repeated and trailing blanks. */
wbuf = .MutableBuffer~new('  now is  the time  ')
say wbuf~words wbuf~wordIndex(1) wbuf~wordIndex(3) wbuf~wordIndex(4) wbuf~wordIndex(5) wbuf~wordLength(1) wbuf~wordLength(4) wbuf~wordLength(5)
say wbuf~word(1) '|' wbuf~word(3) '|' wbuf~word(4) '|' wbuf~word(5)
say wbuf~subWord(2) '|' wbuf~subWord(2, 2) '|' wbuf~subWord(2, 0) '|' wbuf~subWord(5) '|' wbuf~subWord(4, 5) '|' wbuf~subWord(1, 1)
say wbuf~wordPos('the time') wbuf~wordPos('the') wbuf~wordPos('is', 3) wbuf~wordPos('now', 1) wbuf~wordPos('xx') wbuf~wordPos('') wbuf~wordPos('the   time') wbuf~wordPos('time', 4) wbuf~wordPos('time', 5)
say wbuf~containsWord('is') wbuf~containsWord('is', 3) wbuf~containsWord('the time') wbuf~containsWord('tim') wbuf~containsWord('')
say .MutableBuffer~new('')~words .MutableBuffer~new('   ')~words .MutableBuffer~new('')~word(1) '|' .MutableBuffer~new('')~subWord(1) '|' .MutableBuffer~new('')~wordIndex(1)
/* None of the readers changed either buffer. */
say buf~string buf~length
say wbuf~string'|'wbuf~length
```

The oracle's stdout, `cat -A` so the padding and the null strings show:

```text
bcabc | bca | bc   | bc-- | ** |  | $
b | bca | bc |  |  |  | c$
2 5 0 5 0 0 0 6$
5 2 0 2 0 4 1$
1 0 0 1 0 0$
2 0 1 2$
0 3 1 2 0 3 3$
0 1 3 0 0 6$
1 1 0 0 0$
1 1 0 1 1 0 1 0$
0 0 0 0 0 1$
1 0 1 0 0 1$
a | c | c | $
4 3 11 15 0 3 4 0$
now | the | time | $
is  the time | is  the |  |  | time | now$
3 3 0 1 0 0 3 4 0$
1 0 1 0 0$
0 0  |  | 0$
abcabc 6$
  now is  the time  |20$
```

Line 2 is the one a reader of `String~substr` would predict wrongly: `[]` on a `MutableBuffer` is
`StringUtil::substr`'s **two-argument overload** (`classes/support/StringUtil.cpp:128`), whose
length defaults to one byte and is capped at the end with no pad, so `buf[2]` is `b` and
`buf[5, 10]` is `bc`. Line 8's fourth field is `buf~verify('abc', 'N', 9)` → `0`, the start past
the end, the one call that reaches `verify_bytes`'s own guard (Task 3a §6.2).

After the crate change, **measured** on this build (`$S/probes/witness.{ir,tree-walker}.*`, run
from `$S/probes/w/`): both `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` are rc 0, stderr 0 bytes,
and `cmp` against `$S/oracle/witness.out` reports the stdout identical.

Filed in this commit: `corpus/phase-5c.txt` (appended after Task 2's two lines), `coverage.rs`'s
`EXPECTED_SUBSET_5C` (likewise), and `rexx-parse/tests/sourceline_oracle/mutablebuffer_readers.txt`,
generated with the driver from `sourceline_oracle.rs`'s module comment run from the repository
root (`$S/srclines.rex`; driver stderr 0 bytes; header `count 38`, the program's own `wc -l`; the
body `diff`-identical to the program -- **measured**).

### 2.2 Against BASE's binary -- the negative control

**Predicted before running**: rc 120 at line 10, the first reader sent (`substr`), stderr naming
`SUBSTR` and `MutableBuffer`, no stdout, on both engines.

**Measured**: BASE built from `git archive 7e03fbaf2 rust interpreter` under `$B/base/` with
`CARGO_TARGET_DIR=$B/target-base` (`$S/base-build.status`: `build rc 0`, binary sha256
`69ad8505c329a0a179c327e409fbaa57746b6cc34d47293b14758c589459a520`). The witness against it
(`$S/probes/base-witness.{ir,tree-walker}.*`): both engines rc 120, 0 lines of stdout, stderr
`rexx-exec: method "SUBSTR" of class "MutableBuffer" is not implemented (Phase 5)`. **Confirmed.**
(The archive needed the whole `interpreter/` tree, not just `rust/`: `rexx-classes/build.rs` and
`rexx-lib/build.rs` read `interpreter/memory/Setup.cpp`, `RexxClasses/*.orx` and
`platform/unix/PlatformObjects.orx` at build time -- two shorter archives failed at those reads.)

### 2.3 Refusals measured before they were written

Every refusal was run on the oracle first as a two-line program -- `buf = .MutableBuffer~new('abcabc')`
then the probe -- from a fresh empty directory (`$S/oracle/r/*.rex`, batch two `$S/oracle/r2/`,
runner `$S/run_oracle.sh`), then on this build under both engines (`$S/run_crate.sh`,
`$S/oracle/{r,r2}/*.{ir,tree-walker}.*`). **Measured**: every probe agrees with the oracle on all
three descriptors on both engines -- `cmp` on stdout, stderr and rc, the stderr compared raw
because both sides ran the same absolute path -- with none mismatching (the loop counted
208 probe-engine pairs).

| probe | oracle | which raiser here |
|---|---|---|
| `substr`, `[]`, `pos`, `lastPos`, `countStr`, `verify`, `subWord`, `word`, `wordIndex`, `wordLength`, `wordPos`, `contains`, `containsWord`, `match`, `matchChar`, `subChar` sent nothing | 93.903 `Missing argument in method; argument 1 is required.`, rc 163 | `required_position_argument` / `string_method_argument` → `Raised::missing_method_argument(1)` |
| `substr(, 2)` | 93.903 argument 1 | the same, an omitted position |
| `match(1)`, `matchChar(1)` | 93.903 `argument 2 is required` | `string_method_argument(.., 1)` |
| `startsWith`, `startsWith(,)` | 88.901 `Missing argument; argument match is required.`, rc 168 | `Raised::missing_named_argument("match")` |
| `startsWith(.nil)` | 88.909 `Argument match must have a string value.`, rc 168 | `required_string_named_argument(.., "match")` |
| `pos(.nil)`, `pos(.nil, 0)`, `lastPos(.nil)`, `countStr(.nil)`, `verify(.nil)`, `wordPos(.nil)`, `contains(.nil)`, `containsWord(.nil)` | 88.909 `Argument 1 must have a string value.`, rc 168 | `required_string_argument(.., 1)` -- the needle before the position, `pos(.nil, 0)` is 88.909 not 93.924 |
| `match(1, .nil)`, `matchChar(1, .nil)`, `verify('a', .nil)` | 88.909 `Argument 2` | `required_string_argument(.., 2)` |
| `substr(1, 2, .nil)` | 88.909 `Argument 3` | `pad_method_argument` → `required_string_argument(.., 3)` |
| `substr(0)`, `[0]`, `[.nil]`, `substr(.nil)`, `substr('1.5')`, `subWord(0)`, `subWord(.nil)`, `word(0)`, `wordIndex(0)`, `wordLength(0)`, `subChar(0)`, `subChar(.nil)`, `match(0, 'a')`, `matchChar(0, 'a')`, `pos('a', 0)`, `lastPos('a', 0)`, `verify('a', 'N', 0)`, `wordPos('a', 0)`, `contains('a', 0)`, `containsWord('a', 0)`, `match(1, 'a', 0)` | 93.924 `Invalid position argument specified; found "0"` / `"The NIL object"` / `"1.5"`, rc 163 | `optional_position_argument` |
| `substr(1, -1)`, `substr(9, -1)`, `substr(2, 99999999999999999999)`, `[1, -1]`, `pos('a', 1, -1)`, `pos('a', , -1)`, `lastPos('a', 1, -1)`, `verify('a', 'N', 1, -1)`, `verify('a', 'N', 9, -1)`, `verify('a', 'N', 1, 'x')`, `subWord(1, -1)`, `contains('a', 1, -1)`, `match(1, 'a', 1, -1)` | 93.923 `Invalid length argument specified; found "-1"` / `"999…"` / `"x"`, rc 163 | `optional_length_argument` -- checked even for a start past the end (`substr(9, -1)`, `verify(.., 9, -1)`) |
| `substr(1, 2, 'xx')`, `substr(1, 2, '')`, `substr(1, 2, 12)` | **93.922** `Incorrect pad or character argument specified; found "xx"` / `""` / `"12"`, rc 163 | `Raised::incorrect_pad`, **new** in `error.rs` -- the method layer's `padArgument` (`classes/StringClassUtil.cpp:261`), where the builtin layer's is 40.23 |
| `verify('a', 'X')`, `verify('a', '')` | 93.915 `Method option must be one of "MN"; found "X"` / `""`, rc 163 | `Raised::method_option_not_recognised("MN", ..)`, already on the send surface |
| `words(1)`, `word(1, 2)`, `wordIndex(1, 2)`, `wordLength(1, 2)`, `countStr('a', 'b')`, `subChar(1, 2)`, `startsWith('a', 'b')` | 93.902 `Too many arguments in invocation of method; N expected.`, rc 163 | `Arity::Fixed(N)`, the table |
| `[1, 2, 3]`, `subWord(1, 2, 3)`, `wordPos('a', 1, 2)`, `containsWord('a', 1, 2)`, `matchChar(1, 'a', 1)` | 93.902 `2 expected` | `Arity::Fixed(2)` |
| `substr(1, 2, '-', 4)`, `pos('a', 1, 1, 1)`, `lastPos('a', 1, 1, 1)`, `contains('a', 1, 1, 1)` | 93.902 `3 expected` | `Arity::Fixed(3)` |
| `verify('a', 'N', 1, 1, 1)`, `match(1, 'a', 1, 1, 1)` | 93.902 `4 expected` | `Arity::Fixed(4)` |

**Four probes answer at rc 0 where a reader of the argument list would predict a refusal**, and
each is the C++ returning before it converts the later argument:

| probe | oracle | C++ |
|---|---|---|
| `match(99, .nil)` | `0`, rc 0 | `MutableBuffer::match` (`:1460`) tests `_start > getLength()` before `stringArgument(other)` |
| `match(7, 'a', 0)` | `0` | the same test, before `optionalPositionArgument(offset_)` |
| `match(1, 'abc', 4, -1)` | `0` | an explicit offset past `other` returns before `optionalLengthArgument(len_)` |
| `matchChar(99, .nil)` | `0` | `MutableBuffer::matchChar` (`:1668`), the same shape |

The bodies reproduce that order: `native_mutable_buffer_match` and `_matchchar` read the contents'
length through `buffer_state` (copying out a `usize`, so no `&BufferState` is live across the
conversion), answer `0` past it, and only then convert the string argument. §7 says what this
does to the method-body discipline.

Semantics measured the same way, agreeing on both engines (`$S/oracle/r2/`): `verify('abc',
'00'x)` is `1` and `verify('', '00'x)` is `1` -- the `0x00` option byte is admitted, as
`option_letter` admits it for the builtin; `lastPos('c', 7)` on six bytes is `6` (the start is
capped, not refused); `pos('a', 9, 2)`, `pos('a', 7)` and `pos('a', 7, 0)` are `0`; `wordPos('a', 9)`
is `0`; `subWord(2, 99999999999)` is the null string; `buf[9, 0]`, `[6, 1]`, `[6, 2]`, `[7, 0]`
are the null string, `c`, `c`, the null string; `substr(' 2 ', '2.0', '-')` is `bc` and
`substr(2, 3, '00'x)~length` is `3`.

## 3. Controls, each predicted before it ran

| # | control | prediction | reading |
|---|---|---|---|
| C1 | the witness against BASE's binary (§2.2) | rc 120 at `substr`, stderr naming `SUBSTR`, both engines | **confirmed** |
| C2 | the witness on this build, both engines, against the oracle's descriptors | rc 0, stderr empty, stdout `cmp`-identical | **confirmed** (§2.1) |
| C3 | every refusal probe on this build against the oracle, both engines | all three descriptors identical | **confirmed**, 208 pairs, 0 mismatching (§2.3) |
| C4 | the probe comparison can see a difference | `cmp` of two oracle outputs that differ exits 1 | **confirmed**: `cmp $S/oracle/witness.out $S/oracle/r2/pos_startpast_range.out` (21 lines against `0 0 0`) exits 1, `differ: byte 1, line 1` |
| C5 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`, read with its `N of M matching` line | exit 0, `M of M matching` with M one more than before this commit | **confirmed**: rc 0, `360 of 360 matching` (`$S/ctrl/fast/corpus.err`; the Task 2 follow-up read 359 of 359) |
| C6 | the pin's inversion: `lang/mutablebuffer_readers.rex` removed from `EXPECTED_SUBSET_5C` | `phase_5c_subset_matches_the_committed_list` red | **confirmed**: `$S/ctrl/c6/out.txt`, rc 101, that one test FAILED and the other 19 ok; `coverage.rs` restored byte-identical from the WIP copy and touched |
| C7 | mutations, one per family (§3.1) | each caught by the named catcher, the release binary's sha256 unchanged | **confirmed** for the catch, **corrected** for one reading: five of five red, but M4 is caught by a panic rather than a mismatch (§3.1). Every mutant's own sha256 differs from every other's and the release binary reads `695dd78eab08c31b…` after each restore |

### 3.1 Mutations

Predictions written before any mutation ran (`$S/ctrl/mut/predictions.md`), the run
`$S/ctrl/mut/run_all.sh` 09:59 to 10:17, one mutation at a time, each applied by a script that
refuses unless the replaced text occurs exactly once. Every build is `--profile mutation`
(`target/mutation/`), so the release binary is never rebuilt; after each mutation the three edited
sources are restored from `$S/wip-backup/` and `cmp`-checked, and the release sha256 re-read. Each
mutant's `target/mutation/rexx-run` sha256 is recorded: `8dc022f06874d958`, `b9becba2d2faf8d1`,
`3546091e9a44a226`, `b589a9491c9251e8`, `b6d34a8113766985` -- five distinct values, so no run
measured a stale binary.

The three checks per mutation are `cargo test --profile mutation -p rexx-exec --lib`,
`REXX_CORPUS_GATE=1 ... --test corpus` and `... --test method_bodies`.

| id | site | mutation | predicted catcher | reading |
|---|---|---|---|---|
| M1 | `native_mutable_buffer_substr` | drop the `- 1` on `start` | STRICT corpus only | **confirmed**: lib 779/0, corpus rc 101 `359 of 360 matching` on `lang/mutablebuffer_readers.rex`, method_bodies 16/0 with 0 regressions |
| M2 | `buffer_pos` | `start - 1` becomes `start` | STRICT corpus only | **confirmed**: the same three readings |
| M3 | `native_mutable_buffer_words` | `counted(count)` becomes `counted(count + 1)` | STRICT corpus **and** method_bodies | **confirmed**: corpus rc 101 `359 of 360`, method_bodies rc 101 `regressions this run: 1` -- the only mutation the zero-argument probe can see, because `words` is the only reader that answers without an argument |
| M4 | `verify_bytes` (`string.rs`) | delete the `start >= string.len()` guard, `available` by `saturating_sub` | STRICT corpus only, via `verify('abc', 'N', 9)` | **red, by a different mechanism than predicted**: corpus rc 101 with no `N of M matching` line at all. `string[start..start + range]` is `string[8..8]` on six bytes, which panics (`crates/rexx-exec/src/builtin/string.rs:1110`, `range start index 8 out of range for slice of length 6`); the watchdog reports the interpreter thread's panic and the differential never reaches a comparison. The guard is load-bearing against a panic, not against a wrong answer |
| M5 | `native_mutable_buffer_startswith` | drop `!needle.is_empty() &&` | STRICT corpus only | **confirmed**: corpus rc 101 `359 of 360` |

`--lib` is green under all five, as predicted: the readers have no unit tests, and Task 3a's M7b
already measured that `--lib` cannot reach `verify_bytes`'s guard. The witness is what catches
every one of them, which is the reason the plan requires each task to file its own.

## 4. `corpus/method-bodies.txt` row moves

**Predicted before the refresh ran**, from the zero-argument sends measured in §2.3 (`*_none`
probes) -- the probe sends `say o~'NAME'()` to `.MutableBuffer~new('abc')`:

| rows | from | to | evidence |
|---|---|---|---|
| `[]`, `substr`, `pos`, `lastPos`, `countStr`, `verify`, `subWord`, `word`, `wordIndex`, `wordLength`, `wordPos`, `contains`, `containsWord`, `match`, `matchChar`, `subChar` (16 instance rows) | `loud` | `answers` | `rc 163` -- 93.903 on both sides |
| `startsWith` | `loud` | `answers` | `rc 168` -- 88.901 on both sides |
| `words` | `loud` | `answers` | `rc 0` -- `1` on both sides |
| every other row, `MutableBuffer`'s 33 others included | unchanged | unchanged | -- |

No row moves to `diverge`.

**Measured**: the refresh (`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test
method_bodies`, `$S/ctrl/fast/mb.{out,err}`) reads `regressions this run: 0. other drift from the
committed table: 18.`, and `git diff -- corpus/method-bodies.txt` is 18 rows changed, every one a
`MutableBuffer` instance row going `loud` to `answers`, the eighteen names above and no other;
none to `diverge`. The controller wrote its own prediction before the refresh ran
(`$S/ctrl/method-bodies-prediction.txt`, the same eighteen with the same rc per row) and it
matched the agent's. **Confirmed.**

## 5. Corrections to prose and tests

* The agent's §2.3 count of 208 probe-engine pairs with none mismatching was re-run by the
  controller over the recorded descriptors (`$S/oracle/{r,r2}/*.{out,err,rc}` against the
  `.ir.*` and `.tree-walker.*` beside each): 208 pairs, 0 mismatching. The witness was re-run
  from a fresh directory against the rebuilt binary on both engines, identical to the oracle's
  three descriptors, before the fast checks started (`$S/ctrl/oracle-w/`).
* `corpus/refusal-sites.tsv` is the one file the task changed that the brief did not name: the
  fast checks found it (§1), as the Task 2 follow-up did for the same reason. No prose in the
  agent's sections needed correcting.

## 6. Files created outside the tree

* `$B/base/` -- `git archive 7e03fbaf2 rust interpreter` extract; `$B/target-base/` -- its build.
* `$S/oracle/w1/`, `$S/oracle/run.*/`, `$S/probes/w/`, `$S/probes/base/`, `$S/probes/run.*/` --
  the fresh empty run directories.
* `$S/oracle/witness.{out,err,rc}`; `$S/oracle/r/`, `$S/oracle/r2/` -- the probes and their
  descriptors, oracle and both engines; `$S/probes/witness.*`, `$S/probes/base-witness.*`.
* `$S/run_oracle.sh`, `$S/run_crate.sh`, `$S/base-build.sh`, `$S/edit.py`, `$S/srclines.rex`,
  `$S/gates/run.sh`.
* `$S/{string,error,dispatch}.rs.base` -- copies of the three edited files at BASE, for restores.
* `$S/wip-backup/` -- the agent's uncommitted tree at its death, `wip.patch` and the eight files;
  every restore in this task copies from here, never from git.
* `$S/ctrl/fast/` -- the fast checks' descriptors and `status.txt`; `$S/ctrl/refusal.err`,
  `$S/ctrl/clippy.{out,err}`, `$S/ctrl/refusal-sites.before.tsv`, `$S/ctrl/oracle-pad/`,
  `$S/ctrl/oracle-w/`, `$S/ctrl/c6/`, `$S/ctrl/method-bodies-prediction.txt`.
* `$S/ctrl/mut/` -- `predictions.md` (written before any mutation ran), `mutate.sh`,
  `run_all.sh`, `status.txt`, `M1/` to `M5/` with each mutation's three checks and the mutant's
  sha256.

## 7. Open questions

1. **Argument conversion order is part of the answer.** `match(99, .nil)` is `0` on the oracle
   because `MutableBuffer::match` tests the start against the length before it converts the
   string argument (§2.3); the two bodies here mirror that order. The zero-argument
   method-bodies probe cannot see this class of divergence, and neither can a witness that only
   sends well-formed arguments: Tasks 3c to 3e have to read each method's C++ conversion order
   and probe the late-argument cases the way §2.3 did, or accept an untested order.
2. **`refusal-sites.tsv` cites definitions by line.** Every insertion into error.rs above a
   constructor re-derives every row below it: 19 rows in the Task 2 follow-up, 64 here. The
   test's panic makes the re-derivation mechanical and the script is in `§1`, but whether the
   `definition` column should be the file and name rather than the file and line is Moritz's
   call; the line is what makes a moved constructor visible at all.
3. From Task 3a (§6.1 there), unchanged: `MutableBuffer~verify` answers `counted` on every path
   while the builtin's past-the-end zero is an untagged text.
4. `delStr` stays `loud` (`MAKESTRING`) until the conversion commit binds it; the `string`,
   `makeString`, `makeArray` and `subWords` rows likewise.

## Fast checks before the commit

Run by the controller from `rust/` after the rebuild (`$S/ctrl/fast/status.txt`, 09:49 to 09:55,
then the re-derivation and its re-run):

| check | reading |
|---|---|
| `cargo build --release -p rexx-run` | rc 0, `target/release/rexx-run` sha256 `695dd78eab08c31b…` |
| `cargo test --release -p rexx-exec --lib` | 779 passed, 0 failed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | rc 0, `360 of 360 matching` |
| `cargo test --release -p rexx-exec --test refusal_sites --test coverage --test builtin_status` | rc 101: `the_table_holds_every_constructor_the_source_defines` red (§1); coverage 20 passed, builtin_status 19 passed. After the re-derivation `--test refusal_sites` alone: 5 passed |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 1 passed |
| `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies` | 16 passed, `regressions this run: 0. other drift from the committed table: 18.` (§4) |
| `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` | rc 0 and rc 0, run after the mutations restored the tree (`$S/ctrl/clippy.{out,err}`) |

## Gates

Run from `rust/` by `$S/gates/run.sh` in the background, writing each status unpiped to
`$S/gates/status.txt` as it goes -- the commit sha as its first line, `finished` as its last -- with
the pidfile `$S/gates/pid` beside it and each gate's descriptors in `$S/gates/g<N>.{out,err}`.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **G1** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **G2** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **G3** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **G4** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **G5** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G6** |
| 7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G7** |
