# Task 4 review: `builtin/word.rs`, the seven word builtins (`3af26b60`)

**Spec compliance: PASS. Quality: APPROVED**, with four Minor documentation
findings and two observations. No behavioural defect found in ~10,700
differential programs of my own, run independently of the implementer's sweep.

---

## 1. Verification re-run by me, each status unpiped

```
VERIFY1  cargo test --offline --workspace --no-fail-fast         exit=0
         1075 passed, 0 failed, summed over 71 `test result:` lines
VERIFY2  cargo fmt --all --check                                 exit=0
VERIFY3  cargo clippy --offline --workspace --all-targets -- -D warnings
                                                                 exit=0
         and again with CARGO_TARGET_DIR pointed at an empty scratchpad
         directory, every dependency rebuilt                     exit=0
VERIFY4  REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus
                                                                 exit=0
         9 passed, 0 failed, 1 ignored
```

Outside the block:

```
cargo test --offline -p rexx-exec --test builtin_status     12 passed, 0 failed, exit=0
cargo test --offline -p rexx-exec --test keyword_assertions  7 passed, 0 failed, exit=0
cargo test --offline -p rexx-exec --lib builtin::word       16 passed, 0 failed
```

Every number in the report's §10 reproduces. The working tree was clean before
and after; `git status --porcelain` empty at every restore point.

---

## 2. Semantics: my own crossed differential, independent of the report's

Three tiers, all run with the oracle wrapped exactly as
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx ABS )`,
programs written to one scratchpad subdirectory and the interpreters run with a
*different*, empty subdirectory as `cwd`, stdout/stderr/exit status read as
three separate descriptors.

**Tier A -- 8,118 legal call expressions, batched 60 to a program, 136
programs, 0 mismatches.**
Alphabet, stated beside the count as the shared block requires.
*Subjects (18):* `''`; `'   '`; `'09090909'x`; `'x'`; `' a'`; `'a '`;
`'aa bb  cc'`; `'  aa'||'09'x||'bb  '`;
`'a'||'e9'x||' '||'00'x||'b'||'09'x||'c'||'0a'x||'d'`; `'a b c d e f g h i j'`;
`'hey'||'09'x||'is-this  you'`; `'hey  is-this'||'09'x||'you'`;
`'a'||'a0'x||'b c'`; `'0d0a'x`; `'a'||'0b'x||'0c'x||'b'`; `'ff'x||' '||'80'x`;
`'09'x||'a'||'09'x||' '||'b'||' '||'09'x`; `'asdf zxcv'||'0000000000'x||'ASDF ZXCV'`.
That set carries CR, VT and FF, which the report's own 13-subject alphabet does
not.
*Positions (14):* `1 2 3 4 5 11 999999 999999999999999999 ' 2 ' '+2' '007'
'1e1' 2.0 '2.0000'`.
*Counts (9, plus omitted):* `0 1 2 3 9 99 999999999999999999 '2 ' 2.0`.
*`WORDPOS`:* 18 subjects x 17 phrases x 7 starts, the phrases including the
empty one, a blanks-only one, a tab-separated one, a `'00'x` one, a `'0d0a'x`
one, a case-different one, the subject itself and one longer than the subject.
*`SPACE`:* 18 subjects x {omitted, 0, 1, 2, 3} x {default pad, `'-'`} = 144.
The full product of subject x position x count is generated for `SUBWORD` and
`DELWORD`, not one axis varied at a time.

**Tier B -- 1,745 individual programs over the argument-error cross, 0
mismatches.** 3 subjects x 20 position spellings x 12 count spellings for
`SUBWORD`/`DELWORD`, the same positions for `WORD`/`WORDINDEX`/`WORDLENGTH`, 3
phrases x 10 starts for `WORDPOS`, plus 36 hand-written shapes: both arity ends
for all seven, every interior omission, both trailing-omission shapes, five D15
programs and the quoted-literal target. Position and count spellings include
`0`, `-1`, `-99`, `'q'`, `''`, `1.5`, `'1.5'`, `1000000000000000000`,
`-1000000000000000000`, `'01'x||'q'`, `'e9'x`, `'0a'x||'z'`, `' '`, `'+'`,
`'0'`, `'-0'`, `'00'x`. This is the tier that pins the validation order, and it
pins it for every one of the seven, not by inference from a neighbour.

**Tier C -- all 632 method bodies of the seven `<NAME>.testGroup` files**, each
rewritten (`self~assertSame(a,b)` -> one `say` comparing both sides,
`self~expectSyntax(n)` dropped so the raise stands) and run under both
interpreters. 335 are blocked by *other* 4c gaps (`X2D`, `DATATYPE`,
`ERRORTEXT`, environment symbols) and were excluded on the Rust side's
`is not implemented` message; **297 ran end to end, 0 mismatches.** 24 bodies
were not convertible (other `self~` calls, `.ooRexxUnit.architecture`).

**`base/source.file/whiteSpace.testGroup::test_TABs_DELWORD`**, all 12
assertions, run the same way: 0 mismatches. Confirmed against the file that
both it and `DELWORD.testGroup` contain **zero** literal tab bytes
(`/bin/grep -ac "$(printf '\t')"` -> 0 for each; positive control, the same
pattern against `rust/corpus/keyword-exempt.txt` -> 777), so the tabs really do
come from `TAB = "09"x`.

**Harness positive control.** With `ALL_REMAINING_WORDS` mutated to 5, Tier A
reports **5 mismatching programs**. The harness detects.

---

## 3. The word-separator claim, verified twice and independently

**C++.** `interpreter/classes/StringClass.hpp:148` (`skipBlanks`) and `:174`
(`skipNonBlanks`) -- the report's line numbers are exact. Each states the whole
rule as one comparison against `' '` and `'\t'`; there is no `isspace` and no
second scanner. All seven reach it: `StringUtil::wordCount` (`:1157`),
`subWord` (`:1373`), `word` (`:1462`), `wordIndex` (`:1514`), `wordLength`
(`:1542`), `wordPos` (`:1590`) each construct a `RexxString::WordIterator`, and
`RexxString::delWord` (`classes/StringClassWord.cpp:78`) constructs one
directly.

**Oracle.** I ran the 256-byte sweep myself from a fresh empty directory rather
than reading the report's:

```
sep-bytes: 9 32
nl: 1  cr: 1  vt: 1  ff: 1  nul: 1  a0: 1  ff: 1
nulwords: 3  len2: 13
```

Newline, CR, VT, FF, NUL, `0xa0` and `0xff` are word **content**. The report's
correction of the coordinator's addendum is right: the NUL word of
`'asdf zxcv<NUL x5>ASDF ZXCV'` is **13** bytes, not 14, and the oracle's own
`wordlength` says so.

---

## 4. Validation order, per builtin

`RexxString::delWord` and `StringUtil::subWord` each run
`positionArgument(ARG_ONE)` then `optionalLengthArgument(..., ARG_TWO)` and only
then the `count == 0` shortcut; the BIF wrapper
(`expression/BuiltinFunctions.cpp:128,371`) has already converted both
arguments left to right through `required_integer`/`optional_integer`. The
shipped code is that order exactly -- `converted_position(.., 2)?`,
`whole_number(.., 3)?`, `position_of(n)?`, `length_of(value)?`, then
`count == 0`. Tier B's 720-program cross for each of `SUBWORD` and `DELWORD`
confirms it for every pair of spellings, including
`subword('SUBWORD','30'x,'30'x)` -> 93.924 rc 163 and
`subword('a b',0,'q')` -> 40.12 argument 3 rc 216.

`delWord`'s extra `if (isNullString()) return NULLSTRING;` ahead of the
`count == 0` test is unobservable -- both answer the null string -- and the
Rust's single `count == 0 || !scan.skip(position)` is equivalent.

Arity rows match the C++ `_Min`/`_Max` constants for all seven: `DELWORD(2,3)
SUBWORD(2,3) WORD(2,2) WORDINDEX(2,2) WORDLENGTH(2,2) WORDPOS(2,3) WORDS(1,1)`.

---

## 5. The out-of-brief change

**The move is mechanical.** Comparing the removed hunk in `string.rs` against
the added hunk in `mod.rs` line by line: `ARGUMENT_DIGITS`, `arg`,
`required_string`, `optional_string`, `whole_number`, `pad_byte`, `length_of`,
`position_of`, `count_of` and `buffer` all arrive with **identical bodies**.
The only textual changes are in `arg`'s doc comment ("every builtin below" ->
"every builtin", and `check_arity` gaining intra-doc brackets), both of which
are corrections the move *required*. `option_letter` correctly stayed behind.

**`SPACE` still behaves identically.** `text.split(sep).filter(non-empty)` and
`word_slices` both yield the maximal non-separator runs, including on the empty
string. Tier A's 144 `SPACE` expressions across the 18-subject byte alphabet
mismatch on nothing; `string.rs`'s own nine `SPACE` assertions (including
`space("a\tb")` -> `a b` and `space("a\nb")` -> `a\nb`) are unchanged and pass;
`corpus/builtin-probes.txt:104`'s `SPACE` row passes under `builtin_status`.

---

## 6. The two "equivalent mutant" claims, and the replacement guard

**M4 (`DELWORD`'s conditional blank skip) -- equivalence confirmed, twice.**
Analytically: `Words::step` calls `skip_blanks()` *before* testing for the end,
so a `step` that returns `false` leaves `next == text.len()`; `skip`'s
`(0..count).all(..)` short-circuits there, so the subsequent unconditional
`skip_blanks()` has nothing to consume. The C++ `next()` has the same shape
(`skipBlanks` then `if (scanLength == 0) return false`). Empirically: I put the
conditional form back and got **16/16 unit tests green and 0 Tier A
mismatches** on a harness that had just detected M7. The claim is not a covered
gap.

**M8 (`WORDPOS`'s `start > haystack.len()` guard) -- equivalent, correctly
removed.** With `needle.len() >= 1`, `haystack.len() - needle.len() + 1 <=
haystack.len() < start`, so the range is already empty. The guard was dead.

**The replacement guard is load-bearing -- I made it fire.** Deleting
`needle.len() > haystack.len()` gives:

```
thread '...wordpos_matches_words_and_not_their_separators' panicked at
crates/rexx-exec/src/builtin/word.rs:398:18: attempt to subtract with overflow
test result: FAILED. 15 passed; 1 failed
```

It is what stands between `haystack.len() - needle.len()` and an underflow.

**"Can fail" vs "adds coverage" for the one new test.** With
`ALL_REMAINING_WORDS = 5`, **exactly one** test fails and it is
`an_omitted_count_reaches_every_remaining_word`; the other 15 stay green. The
report's account of why it was added is accurate, and the test earns its place.

---

## 7. `keyword-exempt.txt`

`ASSIGNMENT::test_4` appears **nowhere** in the file -- removed, not
re-attributed (`/bin/grep -an "test_4\b"` -> no match; the four surviving
`ASSIGNMENT::` rows are `test_3`, `test_6`, `test_7`, `test_8`). Counts check
out: 777 non-comment rows, 771 tagged `4c`, 6 `defect:` rows, header reads
`4c  771 bodies`. The body genuinely **passes**:
`the_exempt_set_matches_the_current_failures` asserts the set in both
directions in every mode -- a listed body that starts passing is as red as an
unlisted failure -- and it is green, with the harness reporting 119 of 896
passing and 777 not, matching the file exactly. The body is unblocked by this
task's own work: `ASSIGNMENT.testGroup:338` drives its loops off
`words(cl)`/`words(vl)` and `word(cl,ci)`/`word(vl,vi)`.

---

## 8. The sweep's alphabet, its crossing, and its proof of worth

The report states its alphabet beside the count in §4 (13 subjects, 13
positions, 8 counts, 15 phrases x 8 starts) and states which products were
generated, so the "count without its alphabet" failure is avoided. The crossing
is real for `SUBWORD`/`DELWORD` (full subject x position x count) and for
`WORDPOS` (subject x phrase x start), which is the Task 3 lesson applied. My
own Tier A/B reproduce the crossing on a wider alphabet and agree.

The "prove it against a build that had the bug" obligation is met by **M7**,
and I verified that claim rather than accepting it: M7 is invisible to all 15
pre-existing unit tests and is caught by the sweep. That is the right shape.

`builtin-status.txt`: all seven rows read `implemented`; the only occurrence of
`divergent` in the file is a comment on line 35, no row. The live-derivation
test (`the_status_file_matches_a_live_differential_run`) is green.

---

## 9. Findings

1. **Minor.** `word.rs:156-159` -- `word_slices`'s doc names `WORDS` among "the
   callers that want the words themselves". `words()` scans through `Words`
   directly and never calls `word_slices`; its only callers are `word_pos`
   (twice) and `string.rs`'s `space`. A **false** comment, and an in-repo
   call-site enumeration of exactly the shape `rust/CLAUDE.md` says to assert
   or delete.
2. **Minor.** `crates/rexx-exec/src/value.rs:57` still directs the reader to
   "`builtin/string.rs`'s own `buffer`". This commit moved `buffer` to
   `builtin/mod.rs`; the move did not carry the reference with it, so a true
   sentence was made false by this task.
3. **Minor.** `word.rs:30-31` -- "Every one of the seven scans through
   [`Words`], so that rule is stated once" counts statements of a rule inside
   this repository. Tasks 5 and 6 can falsify it without anyone rereading the
   sentence; the same fact is already asserted by
   `only_blank_and_tab_separate_words`, so the prose can go.
4. **Minor.** `word.rs:59-60` -- the D15 paragraph ends "`say n + 0` on the same
   value is `1E+1`", but no `n` is bound anywhere in the example; the report's
   own probe binds `m`. Dangling referent in an otherwise correct measurement.
5. **Minor / observation.** The 4,822-program sweep and its generator live only
   in the session scratchpad. Permanent regression cover for the seven is the
   16 unit tests plus one probe row each in `builtin-probes.txt`; the committed
   differential corpus (`corpus/lang/`) has **one** `word`/`words` call, in
   `string_builtins.rex`, and nothing for the other five. This matches what
   Task 3 did and is not a spec breach, but the phase's strongest evidence for
   this family is not reproducible from the repository.
6. **Observation, not a defect.** `word_pos` materialises two `Vec<&[u8]>` over
   the whole phrase and the whole string on every call, where the C++ compares
   through copied iterators and never allocates; and it scans the entire
   haystack even when `start` is near its end. Correct, and irrelevant at test
   sizes; noted because Tasks 5-6 inherit this module.

## 10. Could not verify from the diff

* The report's own 4,822-program sweep and the per-mutation mismatch counts for
  M1, M2, M3, M5, M6, M9, M11 and M12 -- the generator is not committed. I
  verified M4, M7 and M10 (the replacement guard) directly and substituted my
  own 8,118 + 1,745 + 632-body sweep for the rest.
* §10's note that `cargo fmt --all --check` exited 1 on a first run before being
  applied -- history, not observable now.
* §9's account of the stood-down subagent -- process, not in the tree.
