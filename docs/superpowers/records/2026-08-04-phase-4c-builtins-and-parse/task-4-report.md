# Task 4 report: `builtin/word.rs`, the seven word builtins

`DELWORD SUBWORD WORD WORDINDEX WORDLENGTH WORDPOS WORDS`, delivered in a new
`crates/rexx-exec/src/builtin/word.rs` with seven rows in `builtin/mod.rs`.

---

## 1. The word-separator answer

**Exactly two bytes: blank `0x20` and horizontal tab `0x09`. No other byte
separates words -- not newline, not carriage return, not vertical tab, not
form feed, not NUL, and no byte at or above `0x80`.**

### The C++ that decides it

`RexxString::WordIterator`, `interpreter/classes/StringClass.hpp:135-315`. Two
methods, and each states the whole rule as one comparison:

```cpp
void skipBlanks(const char *&string, size_t &scanLength)      // :148
{
    for (;length > 0; scan++, length--)
    {
        if (*scan != ' ' && *scan != '\t') { break; }
    }
    ...
}

void skipNonBlanks(const char *&string, size_t &scanLength)   // :174
{
    for (;length > 0; scan++, length--)
    {
        if (*scan == ' ' || *scan == '\t') { break; }
    }
    ...
}
```

Every one of the seven reaches this: `StringUtil::wordCount`, `subWord`,
`word`, `words`, `wordIndex`, `wordLength` and `wordPos`
(`classes/support/StringUtil.cpp:1148-1638`) each construct a
`RexxString::WordIterator`, and `RexxString::delWord`
(`classes/StringClassWord.cpp:58`) constructs one directly. There is no second
scanner and no `isspace` anywhere on the path.

Note that `char` is signed on this target, so a byte at or above `0x80` is a
negative `char` and can never compare equal to `' '` or `'\t'`; it is word
content by the same test.

### The measurement, over the whole byte range rather than at plausible bytes

I did not sample. Program run from a fresh empty directory:

```rexx
out = ''
do i = 0 to 255
  c = d2c(i)
  if words('a' || c || 'b') = 2 then out = out i
end
say 'sep-bytes:' out
```

```
sep-bytes:  9 32
```

Two bytes, and 254 that are not. The supporting lines from the same run:

| probe | oracle |
|---|---|
| `words('')` | `0` |
| `words('   ')` | `0` |
| `words('09090909'x)` | `0` |
| `words('  a b  ')` | `2` |
| `words('a    b')` | `2` |
| `words('a'\|\|'09'x\|\|' '\|\|'09'x\|\|'b')` | `2` |
| `words('a'\|\|'0a'x\|\|'b')` | **`1`** -- newline is word content |
| `words('a'\|\|'a0'x\|\|'b')` | **`1`** -- a high byte is word content |
| `words('a'\|\|'00'x\|\|'b')` | **`1`** -- NUL is word content |

The NUL result is corroborated by the oracle's own committed suite:
`ootest/ooRexx/base/bif/DELWORD.testGroup:136` (test19) asserts that
`'asdf zxcv<NUL x5>ASDF ZXCV'` is three words. I re-measured that string
rather than reading it: `words` is 3, `wordlength(s,2)` is **13**
(`zxcv` + 5 NULs + `ASDF`), and `delword(s,3,2)` is
`asdf zxcv<NUL x5>ASDF ` -- the blank before the deleted word survives.

> Correction to the coordinator's addendum: it gave that word as **14** bytes.
> Measured here it is **13**. `4 + 5 + 4 = 13`, and the oracle's own
> `wordlength` says 13.

The rule is factored once. `is_blank` and `Words` in `word.rs` are the only
statement of it in this crate; `string.rs`'s `SPACE` previously carried a
second copy (`text.split(|&b| b == b' ' || b == b'\t')`) and now calls
`word::word_slices`.

---

## 2. The probe table

Every oracle invocation was wrapped as
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx ABSOLUTE_PATH )`
and run with an empty directory as `cwd`, with the program in a separate
directory, both by absolute path. stdout, stderr and exit status were read as
three separate descriptors, never `2>&1`.

### 2.1 Arity (`fix_args`), both ends

| probe | oracle | rc |
|---|---|---|
| `words()` | 40.3 `... minimum expected is 1.` | 216 |
| `word('a b')` | 40.3 `... minimum expected is 2.` | 216 |
| `wordindex('a b')` | 40.3 `... minimum expected is 2.` | 216 |
| `wordlength('a b')` | 40.3 `... minimum expected is 2.` | 216 |
| `subword('a b')` | 40.3 `... minimum expected is 2.` | 216 |
| `delword('a b')` | 40.3 `... minimum expected is 2.` | 216 |
| `wordpos('a')` | 40.3 `... minimum expected is 2.` | 216 |
| `words('a','b')` | 40.4 `... maximum expected is 1.` | 216 |
| `word('a b',1,2)` | 40.4 `... maximum expected is 2.` | 216 |
| `subword('a',1,2,3)` | 40.4 `... maximum expected is 3.` | 216 |
| `delword('a b',1,2,3)` | 40.4 `... maximum expected is 3.` | 216 |
| `wordpos('a','a b',1,2)` | 40.4 `... maximum expected is 3.` | 216 |

Rows: `DELWORD (2,3) SUBWORD (2,3) WORD (2,2) WORDINDEX (2,2) WORDLENGTH (2,2)
WORDPOS (2,3) WORDS (1,1)`, each confirmed at both ends and matching the
`x_Min`/`x_Max` constants in `expression/BuiltinFunctions.cpp:128-481`.

### 2.2 Interior omission (40.5) -- probed before every optional position

| probe | oracle | rc |
|---|---|---|
| `word(,1)` | 40.5 `... argument 1 is required.` | 216 |
| `subword('a b',,1)` | 40.5 `... argument 2 is required.` | 216 |
| `delword('a b',,1)` | 40.5 `... argument 2 is required.` | 216 |
| `wordpos('a',,1)` | 40.5 `... argument 2 is required.` | 216 |
| `words(,1)` | **40.4**, not 40.5 -- the maximum is checked first | 216 |
| `subword('a b',2,)` | `bb  cc` -- a trailing omission is not an argument | 0 |
| `delword('a b c',2,)` | `a ` -- likewise | 0 |
| `wordpos('the',h,)` | `3` -- likewise | 0 |

**Finding: none of the seven has a conditionally-required position.** Every
required position is a prefix of length `min`, so `check_arity`'s existing
count check produces the oracle's answer for all of them and no builtin here
raises 40.5 itself. This is the `DATE('S',,'S')` shape being *absent* from
this family, and I probed for it rather than assuming it.

### 2.3 Boundary positions -- 0, 1, last, one past the last, negative

Subject `'aa bb  cc'`, whose words begin at bytes 1, 4 and 8.

| n | `word` | `wordindex` | `wordlength` | `subword` | `delword` |
|---|---|---|---|---|---|
| 0 | 93.924 rc 163 | 93.924 rc 163 | 93.924 rc 163 | 93.924 rc 163 | 93.924 rc 163 |
| -1 | 93.924 rc 163 | 93.924 rc 163 | 93.924 rc 163 | 93.924 rc 163 | 93.924 rc 163 |
| 1 | `aa` | `1` | `2` | `aa bb  cc` | `` |
| 2 | `bb` | `4` | `2` | `bb  cc` | `aa ` |
| 3 | `cc` | `8` | `2` | `cc` | `aa bb  ` |
| 4 | `` | `0` | `0` | `` | `aa bb  cc` |
| 999999 | `` | `0` | `0` | `` | `aa bb  cc` |
| 999999999999999999 | `` | `0` | `0` | `` | `aa bb  cc` |

93.924's message is `Invalid position argument specified; found "0".` and
names neither the routine nor a position, which is why it is
`Raised::invalid_position` and not a 40.x raiser.

Leading separators, `'  aa'||'09'x||'bb  '`:

| n | `word` | `wordindex` | `wordlength` |
|---|---|---|---|
| 1 | `aa` | `3` | `2` |
| 2 | `bb` | `6` | `2` |
| 3 | `` | `0` | `0` |

### 2.4 `SUBWORD` -- separators kept inside, dropped outside

| probe | oracle |
|---|---|
| `subword('aa bb  cc',2)` | `bb  cc` (both blanks kept) |
| `subword('aa bb  cc',2,1)` | `bb` |
| `subword('aa bb  cc',1,2)` | `aa bb` |
| `subword('aa bb  cc',1,99)` | `aa bb  cc` |
| `subword('aa bb  cc',2,0)` | **`` (null string)** |
| `subword('aa bb  ',1)` | `aa bb` (trailing blanks dropped) |
| `subword('  aa bb',1)` | `aa bb` (leading blanks dropped) |
| `subword('aa'\|\|'09'x\|\|'09'x\|\|'bb cc',1,2)` | `aa<TAB><TAB>bb` (both tabs kept) |
| `subword('',1)`, `subword('   ',1)` | `` |
| `subword('a b',1,999999999999999999)` | `a b` |
| `subword('a b',1,1000000000000000000)` | 40.12, rc 216 |

### 2.5 `DELWORD` -- takes the separators after, leaves the ones before

| probe | oracle |
|---|---|
| `delword('aa bb  cc',2,1)` | `aa cc` |
| `delword('aa bb  cc',1,1)` | `bb  cc` |
| `delword('aa bb  cc',1,2)` | `cc` |
| `delword('aa bb  cc',2,0)` | `aa bb  cc` (unchanged) |
| `delword('aa bb  cc',4)` | `aa bb  cc` (unchanged) |
| `delword('  aa bb',1,1)` | `  bb` |
| `delword('aa bb   ',2)` | `aa ` (trailing separators go too) |
| `delword('aa'\|\|'09'x\|\|'bb cc',2,1)` | `aa<TAB>cc` |
| `delword('   ',1)` | `   ` (unchanged) |
| `delword('',1)`, `delword('',999)` | `` |

The whiteSpace pair from the coordinator's addendum, re-measured here:

| probe | oracle |
|---|---|
| `delword('hey'\|\|'09'x\|\|'is-this  you',2,1)` | `hey<TAB>you` |
| `delword('hey  is-this'\|\|'09'x\|\|'you',2,1)` | `hey  you` |

The surviving separator is the argument's own byte, unnormalised, and the
tab-vs-blank identity is preserved on both sides of the deletion.

### 2.6 `WORDPOS`

Haystack `h = 'now is the time for all good men'`.

| probe | oracle |
|---|---|
| `wordpos('the',h)` | `3` |
| `wordpos('the time',h)` | `3` |
| `wordpos('the   time',h)` | `3` (the phrase's own separators do not matter) |
| `wordpos('the'\|\|'09'x\|\|'time',h)` | `3` |
| `wordpos('  the  ',h)` | `3` |
| `wordpos('The',h)` | `0` (byte comparison, case-sensitive) |
| `wordpos('th',h)` | `0` (whole words only) |
| `wordpos('',h)`, `wordpos('   ',h)` | `0` |
| `wordpos('a','')`, `wordpos('','')` | `0` |
| `wordpos(h,h)` | `1` |
| `wordpos('good men',h)` | `7` |
| `wordpos('good men xx',h)` | `0` |
| `wordpos('the',h,3)` / `,4` / `,99` | `3` / `0` / `0` |
| `wordpos('men',h,8)` / `,9` | `8` / `0` |
| `wordpos('good men',h,7)` / `,8` | `7` / `0` |
| `wordpos('aa','aa bb aa',1)` / `,2` | `1` / `3` |
| `wordpos('a0'x,'x '\|\|'a0'x\|\|' y')` | `2` |
| `wordpos('00'x,'x '\|\|'00'x\|\|' y')` | `2` |
| `wordpos('a','a b',0)` / `,-1` | 93.924 rc 163 |
| `wordpos('a','a b','q')` | 40.12 `WORDPOS argument 3 ...` rc 216 |

### 2.7 The two argument-error layers, and their order

| probe | oracle | rc |
|---|---|---|
| `word('a b','x')` | 40.12 `WORD argument 2 must be a whole number; found "x".` | 216 |
| `word('a b',1.5)` | 40.12 `... found "1.5".` | 216 |
| `word('a b','')` | 40.12 `... found "".` | 216 |
| `word('a b',99999999999999999999)` | 40.12 `... found "99999999999999999999".` | 216 |
| `word('a b',0)` / `(-1)` | 93.924 | **163** |
| `subword('a b',1,'q')` | 40.12 `SUBWORD argument 3 ...` | 216 |
| `subword('a b',1,-1)` | 93.923 `Invalid length argument specified; found "-1".` | **163** |
| `delword('a b',1,-1)` | 93.923 | **163** |
| `subword('a b',0,'q')` | **40.12 argument 3** -- conversion beats range | 216 |
| `delword('a b',0,'q')` | **40.12 argument 3** | 216 |
| `subword('a b','q','r')` | 40.12 **argument 2** -- conversions run left to right | 216 |
| `subword('a b',0,-1)` | **93.924** -- the position's range check beats the count's | 163 |
| `delword('a b',0,-1)` | **93.924** | 163 |
| `subword('SUBWORD','30'x,'30'x)` | **93.924** -- not `''` at rc 0 | 163 |
| `delword('delWord','30'x,'30'x)` | **93.924** | 163 |

So the order is: convert argument 2, convert argument 3, range-check the
position, range-check the count, *then* honour a zero count.

The generous spellings the conversion accepts, each measured:
`word('a b c',' 2 ')` and `word('a b c','+2')` are `b`; `word('a b','007')`
is position 7 and `word('a b','1e2')` is position 100, both the null string.

### 2.8 D15, and the substitution's byte fidelity

D15 needs `DIGITS` to change between creation and rendering, and this is that
probe:

```rexx
numeric digits 12 ; v = 1/3 ; numeric digits 2 ; say word('a b', v)
```

```
Error 40.12:  WORD argument 2 must be a whole number; found "0.333333333333".
```

Twelve digits, not two: **the message substitutes the value's rendering as
fixed at creation**, not a re-render under the settings in force. That answers
the shared block's open question for this family.

The other side of D15, on the results:

| probe | oracle |
|---|---|
| `numeric digits 12; m = words('a b c d e f g h i j'); numeric digits 1; say m` | `10` |
| ... `say m + 0` | `1E+1` |
| `numeric digits 1; say words(...)` / `wordindex(...,10)` / `wordpos('j',...)` | `10` / `19` / `10` |

So `WORDS`, `WORDINDEX`, `WORDLENGTH` and `WORDPOS` create their results as
text, and the addition is a new operation under the digits then in force.

Byte fidelity of the substitution, measured on this family and not inherited:

| probe | `found "..."` |
|---|---|
| `word('a b','01'x\|\|'q')` | `?q` -- the control byte is sanitised |
| `word('a b','e9'x)` | the raw `0xe9` byte (`cat -A` shows `M-i`) |
| `subword('a b',1,'0a'x\|\|'z')` | raw `0x0a`; the message breaks across lines |

`0x0a` is outside `00-08 0b-0c 0e-1f` and is therefore not sanitised, which
is the existing rule and needed no change. I added no third application of it
and bypassed none.

### 2.9 Allocation

No builtin here can produce a result larger than its own string argument, so
none has an Error 5 path of the kind `COPIES`/`LEFT` have. Confirmed:
`subword(s,1,999999999999999999)` and `delword(s,1,999999999999999999)` on a
40-byte subject answer 39 and 0 bytes. `DELWORD` is the only one that builds
rather than slices, and it goes through the shared fallible `buffer` and
`Interp::text_owned` so the reservation is handed over rather than copied.
Every allocation is `Interp::text`/`text_owned`, which is `alloc_with`; no
`Heap::alloc*` call is reachable from this file.

---

## 3. The ooTest groups (brief Step 2)

`ootest/ooRexx/base/bif/{DELWORD,SUBWORD,WORD,WORDINDEX,WORDLENGTH,WORDPOS,WORDS}.testGroup`
-- **in this repository, not in the C++ oracle tree**; `/home/moritz/dev/repos/ooRexx/ootest`
does not exist.

`expectSyntax` totals across the seven, counted by me with `/bin/grep -a`:

| group | 40.12 | 93.924 |
|---|---|---|
| DELWORD | 10 | 3 |
| SUBWORD | 28 | 6 |
| WORD | 11 | 3 |
| WORDINDEX | 11 | 3 |
| WORDLENGTH | 10 | 3 |
| WORDPOS | 11 | 2 |
| WORDS | 0 | 0 |
| **total** | **81** | **20** |

**Nothing else.** A search for `40.3`, `40.5`, `40.03`, `40.05` and `93.90`
across all seven returns 0 in every file; the same pattern finds `40.5` in
`OVERLAY.testGroup`, `LEFT.testGroup` and `C2D.testGroup`, so the search is
not vacuous. **ooTest never exercises the omitted-required-argument path for
these seven**, so my own probes in §2.1 and §2.2 are the only evidence for
that behaviour, and the unit test `the_arity_rows_are_the_oracles_own` is
where it is now asserted.

The two error families ooTest does exercise are exactly the two this
implementation raises.

---

## 4. The corpus: its axes and how they were crossed

A generated differential sweep of **4,822 programs**, each its own file, each
run under both interpreters at the same absolute path from an empty working
directory, compared on all three descriptors. The generator and runner are
`gen.py` / `run_sweep.py` in the session scratchpad.

**The axes are crossed, not widened one at a time.** The full product of
subject x position x count is generated for `SUBWORD` and `DELWORD`, subject x
position for `WORD`/`WORDINDEX`/`WORDLENGTH`, and subject x phrase x start for
`WORDPOS`.

**Axis 1 -- the subject's alphabet (13 values).** Stated beside the count, as
required: `''`; `'   '`; `'09090909'x`; `'x'`; `' a'`; `'a '`;
`'aa bb  cc'`; `'  aa'||'09'x||'bb  '`;
`'a'||'e9'x||' '||'00'x||'b'||'09'x||'c'||'0a'x||'d'`;
`'a b c d e f g h i j'`; `'hey'||'09'x||'is-this  you'`;
`'hey  is-this'||'09'x||'you'`; `'a'||'a0'x||'b c'`.
This alphabet contains a byte at or above `0x80` (`0xe9`, `0xa0`), control
bytes (`0x00`, `0x0a`), the null string, tabs as separators and tabs as the
only separator, and leading/trailing/repeated separators.

**Axis 2 -- the position (13 values):** `1 2 3 4 999999 0 -1 'q' ''
999999999999999999 1000000000000000000 1.5 '007'` -- legal interior, the last
word, one past it, far past it, both range refusals, both conversion
refusals, either side of the argument-precision boundary, and two spellings
that convert to a legal value.

**Axis 3 -- the count (8 values):** omitted, `0`, `1`, `2`, `99`, `-1`, `'q'`,
`999999999999999999`.

**Axis 4 -- the `WORDPOS` phrase (15 values)** crossed with **the start (8
values)**: the empty phrase, a blanks-only phrase, single words that match and
that do not, a multi-word phrase, the same phrase with a different separator
run, a case-different phrase, phrases containing a NUL and a newline, a phrase
longer than the haystack, and the haystack itself; starts of omitted, 1, 2, 3,
11, 0, -1 and `'q'`.

Plus 42 hand-written shapes with no third axis: every interior omission, every
arity end, the layer-ordering pairs, the substitution-byte cases and two D15
programs.

**Result: 4,822 programs, 0 mismatches.**

### The negative control

`/bin/true` as the Rust side gives 4,822 mismatches. That control is nearly
worthless on its own, so the real one is **deliberately broken builds of my own
code**. Each mutation was applied to `word.rs`, rebuilt, and run against both
the sweep and the committed unit suite; the file was then restored from a copy
I made, never with `git checkout`.

| mutation | sweep | committed unit suite |
|---|---|---|
| M1 `is_blank` also accepts `\n` | **24 mismatches** | 3 tests fail |
| M2 `wordindex` returns `start` not `start + 1` | **26** | 3 fail |
| M3 `subword` drops the zero-count shortcut | **24** | 3 fail |
| M4b `delword` never absorbs the trailing blanks | **27** | 2 fail |
| M5 `wordpos` ignores its `start` | **37** | 2 fail |
| M6 `subword` range-checks before converting argument 3 | **54** | 1 fail |
| M7 `ALL_REMAINING_WORDS` is 5 rather than the oracle's max | **8** | **0 fail (initially)** |
| M9 `wordpos` drops the empty-phrase guard | **88** | 2 fail |
| M10 `wordpos` drops the too-long-phrase guard | **105** | 1 fail |
| M11 `subword`'s slice ends at the scan, not the word | **11** | 1 fail |
| M12 `delword` loses one leading byte | **80** | 3 fail |

**M7 is what makes the crossing earn its place rather than merely be large.**
A default count of 5 is indistinguishable on every subject with four words or
fewer, which is every subject the unit tests used for `SUBWORD`/`DELWORD`; the
sweep caught it because it crosses the ten-word subject with the
omitted-count axis. Following the project's "can fail is not adds coverage"
rule in the direction it points, I then added
`an_omitted_count_reaches_every_remaining_word` so the *committed* suite
catches it too -- re-run, M7 now fails 1 unit test as well.

### Two mutations that stayed green, and why that is not a gap

* **M4** -- replacing `if scan.skip(count - 1) { scan.skip_blanks(); }` with
  the unconditional pair: 0 sweep mismatches, 0 unit failures. Not a missing
  test: the two forms are **provably** the same, because a failed
  `Words::step` leaves the scan at the end of the string and the blank skip is
  then a no-op. The conditional was therefore a branch nothing can
  distinguish, and my comment on it implied otherwise. I removed the branch
  and rewrote the comment to state the equivalence and its reason.
* **M8** -- `start > haystack.len()` widened to `+ 1`: 0 mismatches, likewise
  equivalent, because `start..=len` is already empty when `start > len`. That
  guard turned out to be doing nothing at all, so I removed it and replaced it
  with `needle.len() > haystack.len()`, which *is* load-bearing (it is what
  the candidate-count subtraction cannot survive). M10 above is that guard's
  witness, and I added three unit cases for the shape.

---

## 5. What moved in the committed corpora

**`rust/corpus/builtin-status.txt`** -- exactly seven rows flipped
`loud` -> `implemented`: `DELWORD SUBWORD WORD WORDINDEX WORDLENGTH WORDPOS
WORDS`. **No row reads `divergent`.** `cargo test -p rexx-exec --test
builtin_status` derived that table live and now agrees with the file in both
directions (12 tests, 0 failures).

**`rust/corpus/keyword-exempt.txt`** -- **one row removed**:

```
ASSIGNMENT::test_4	4c
```

It now passes, and `the_exempt_set_matches_the_current_failures` named it.
It was **removed, not re-attributed**. The header's `4c 772 bodies` was
updated to `771`; the file goes from 778 to 777 non-comment rows (771 + 6
`defect:` rows).

I did not go looking for which of my seven unblocked it -- the harness
derives the owner and the row's job is membership.

---

## 6. Design decisions worth flagging

### The shared argument helpers were hoisted from `string.rs` into `mod.rs`

`word.rs` needs `required_string`, `whole_number`, `position_of`,
`length_of`, `arg` and `buffer`, all of which Task 3 had left private to
`string.rs`. Duplicating them would have been a drift hazard across the five
family tasks still to come, so I **moved** the two cohesive blocks -- "reading
arguments" (`ARGUMENT_DIGITS`, `arg`, `required_string`, `optional_string`,
`whole_number`, `pad_byte`) and "range-checking converted arguments"
(`length_of`, `position_of`, `count_of`) -- plus `buffer`, into `builtin/mod.rs`,
and `string.rs` now imports them from `super`. The move is mechanical: no body
changed, and the whole workspace suite is the check.

`option_letter` stayed in `string.rs`; it takes a `valid: &str` option set and
is a string-option concept, not part of the shared argument protocol.

**This modifies `string.rs`, which my brief's file list did not name.** I judged
one implementation better than two copies, and `mod.rs` -- which the brief does
name -- is where the shared protocol belongs. Flagging it as the one place I
went outside the named files.

### `SPACE`'s word scan now goes through `word.rs`

`string.rs` had its own five-line `words()` stating the blank/tab rule a second
time. The brief says to factor the rule once; it is now stated once, in
`word.rs`, and `SPACE` calls `word::word_slices`. Behaviour is identical (the
`split`/`filter` form and the iterator produce the same word sequence) and
`string.rs`'s own `SPACE` tests, including `space('a\tb')` and `space('a\nb')`,
pass unchanged.

---

## 7. The four addendum behaviours: had I got them right?

The coordinator asked for the count of what a careful reading would have
missed. Honestly:

1. **Position check before the zero-length shortcut** -- **I had it WRONG and
   was corrected by the message.** My first draft called
   `position_of(converted_position(...))` as one step, which put the position's
   *range* check ahead of argument 3's *conversion*. That is defect M6 in the
   table above, and it would have made `subword('a b',0,'q')` answer 93.924
   where the oracle answers 40.12. Two things are worth recording: my own
   committed test `the_call_layer_is_checked_before_the_operation_layer`,
   written from the shared block before the addendum arrived, already asserted
   the correct answer and would have caught it at the first `cargo test`; and
   the specific `('30'x,'30'x)` shape the addendum names was *not* one I had
   probed, and is now a test of its own.
2. **The 40.12 / 93.924 split being two layers** -- **already right.** It is in
   `string.rs`'s module doc from Task 3 and I had probed both kinds for every
   position argument (§2.7).
3. **`DELWORD` keeping the preceding separator byte-for-byte** -- **already
   right**, from reading `RexxString::delWord`'s `frontLength` and measuring
   `delword('  aa bb',1,1)` and `delword('aa'||'09'x||'bb cc',2,1)`. The
   addendum's two exact cases are now tests, and I re-measured both rather
   than transcribing them.
4. **ooTest not exercising 40.3/40.5 for these seven** -- **already covered by
   accident of method**: I built the arity probes from
   `BuiltinFunctions.cpp`'s `x_Min`/`x_Max` constants and the oracle, not from
   the test groups. I have since confirmed the absence and its positive
   control myself (§3).

**Score: one of four wrong, and it was the ordering one.**

---

## 8. Things the brief or the addendum got wrong

* **The addendum's NUL word is 13 bytes, not 14.** Measured: `wordlength` of
  word 2 of `'asdf zxcv<NUL x5>ASDF ZXCV'` is `13`. Everything else in that
  paragraph is confirmed.
* **The brief's Step 1 says to probe "leading, trailing and repeated blanks,
  and tabs".** That is necessary and not sufficient: the interesting half of
  the separator rule is which bytes are *not* separators, and a probe set
  built only from blanks and tabs cannot see that `0x0a`, `0x0b`, `0x0c`,
  `0x0d`, `0x00` and `0xa0` are word content. I swept all 256.
* The brief's step list has no step for the sweep or its negative control;
  those come only from the shared block. Not a contradiction, just a seam.
* No measurement contradicted the brief.

---

## 9. What I could not do

* **I could not read the ooTest groups through my subagent.** I dispatched one
  with the path the brief implied (`/home/moritz/dev/repos/ooRexx/ootest`),
  which does not exist; by the time the correct path arrived, the coordinator
  had a survey in hand and told me to stand the agent down, which I did. I
  read the seven groups' `expectSyntax` sets directly instead (§3), which is
  the part my implementation depends on, and read `DELWORD.testGroup:136`
  myself for the NUL case.
* **`ulimit -v` caveat.** Nothing in this task produced a memory-shaped
  finding, so the address-space-versus-RSS distinction did not bite. The one
  size-related claim (§2.9) is that no result here can exceed its argument,
  which is structural and independent of any limit.
* No `unsafe`, none needed.

---

## 10. Verify block, exit statuses read unpiped

```
VERIFY1  cargo test --offline --workspace --no-fail-fast            exit=0
         1075 passed, 0 failed, summed over every `test result:` line.
         16 of those are new, all in `builtin::word::tests`; no test was
         removed, and the brief's pre-task figure of 1,059 is consistent
         with that but was not re-measured by me.
VERIFY2  cargo fmt --all --check                                     exit=0
         (exit=1 on the first run; `cargo fmt --all` applied, re-checked)
VERIFY3  cargo clippy --offline --workspace --all-targets -- -D warnings
                                                                     exit=0
         and again from a CLEAN target directory                     exit=0
VERIFY4  REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus
                                                                     exit=0
         9 passed, 0 failed, 1 ignored
```

`VERIFY3` was run twice on purpose: `rust/CLAUDE.md` records that a warm target
directory can return exit 0 without having linted the code, so the second run
used `CARGO_TARGET_DIR` pointed at an empty scratchpad directory and rebuilt
every dependency.

Additionally, outside the block:

```
cargo test --offline -p rexx-exec --test builtin_status    12 passed, 0 failed
cargo test --offline -p rexx-exec --test keyword_assertions 7 passed, 0 failed
the 4,822-program differential sweep                        0 mismatches
```

---

## 11. Files changed

| path | change |
|---|---|
| `rust/crates/rexx-exec/src/builtin/word.rs` | **new** -- the seven builtins, the shared scanner, 16 tests |
| `rust/crates/rexx-exec/src/builtin/mod.rs` | `mod word;`, seven `IMPLEMENTED` rows, the hoisted shared helpers |
| `rust/crates/rexx-exec/src/builtin/string.rs` | imports the hoisted helpers; `SPACE` uses the shared scan |
| `rust/corpus/builtin-status.txt` | seven rows `loud` -> `implemented` |
| `rust/corpus/keyword-exempt.txt` | `ASSIGNMENT::test_4` removed; header count 772 -> 771 |

Nothing else in the working tree; no scratch file was written into the
repository.

---

## 12. Commit

Read back with `git log --format="%H %s" -1` after committing:

```
3af26b60bd76eb9ed23f4e6808eb49703c0341df Implement the seven word builtins
```

Five files, 1233 insertions, 140 deletions.

There is no second commit: `.superpowers` is in `.gitignore`, so this report
is written to the ledger path but is not a tracked file. The working tree is
clean at `3af26b60`.
