# Task 5 report -- `builtin/convert.rs`, the twelve conversion builtins

**Status: DONE.** Commit `ca0b63f917ddd33ccf4a4bc6ccf91b7b3648c3cb` (`ca0b63f9`), working tree clean.

Files: created `rust/crates/rexx-exec/src/builtin/convert.rs`; modified
`rust/crates/rexx-exec/src/builtin/mod.rs`, `rust/crates/rexx-exec/src/error.rs`,
`rust/corpus/builtin-status.txt`, `rust/corpus/keyword-exempt.txt`.

---

## 1. The grouping rule, read from the C++

**The controller's ambiguity 1 is resolved: the rule is a residue on a *running total*, and the
brief's inferred wording is equivalent to it but does not say the same thing.**

`StringUtil::validateGroupedSet` (`interpreter/classes/support/StringUtil.cpp:744-812`), which
is the single scanner behind `X2C`, `X2B`, `B2X` and `X2D` (via `packHex`):

```c++
if (*string == ch_SPACE || *string == ch_TAB)  reportException(hex ? hexblank : binblank, IntegerOne);
...
for (; length; length--) {
    ch = *current++;
    if (set[(unsigned char)ch] != '\xff')      count++;
    else if (ch == ch_SPACE || ch == ch_TAB) {
        spaceLocation = current;
        if (!spaceFound) { residue = (count % modulus); spaceFound = 1; }
        else if (residue != (count % modulus))  reportException(... invhex_group / invbin_group);
    }
    else reportException(... invhex / invbin, new_string(ch));
}
if (ch == ch_SPACE || ch == ch_TAB)            reportException(... hexblank, spaceLocation - string);
else if (spaceFound && ((count % modulus) != residue))  reportException(... invhex_group);
return count;
```

So, exactly:

1. Whitespace at index 0 is `93.931`/`93.932` naming position **1** -- a separate error from grouping.
2. `count` is a **cumulative** digit total, never a per-group one.
3. At the **first** run of whitespace, `residue = count % modulus` is recorded.
4. At **every later** run of whitespace, and once more **at the end of the string**, `count % modulus`
   must equal that residue.
5. `spaceLocation` is rewritten at *every* whitespace byte, so the trailing-whitespace error names the
   **last** byte of a trailing run, not the first.
6. `modulus` is 2 for hexadecimal and 4 for binary; `hex` is a plain `bool` selecting between paired
   catalogue entries at all three failures.

Because the total is cumulative, "cumulative ≡ residue at every gap" forces each *intermediate* group
to be ≡ 0 mod modulus, and the final check forces the last one to be too. **That makes the brief's
"the FIRST group sets the residue, every LATER group must be an exact multiple" equivalent in
outcome**, and it is also right that the first group is left-padded rather than rejected (`packHex`
takes `b = nibbles % 2 == 0 ? 2 : 1` on its first pass). The difference worth recording is the
*mechanism*: an implementation that measures each group separately, rather than carrying a running
total, gets the same answers -- but the C++'s own state is the residue.

**Two different readings have to be separated, and they need different witnesses.** `b2x('101 0000')`
= `50` (residue 3, then an exact 4) separates "residue from the first group" from "residue always 0";
it does **not** separate "checked at every gap" from "checked once at the end", because it passes both.
The witness for that second reading is a string that breaks the residue at an *interior* gap and then
comes back to it by the end -- `x2c('41 4 1 42')`, `x2c('414 2 434')`, `b2x('1010 10 10')`, each
93.976/93.977 on the oracle and each converting cleanly under an end-of-string-only check. See §10.

One further C++ fact the brief did not carry: `validateGroupedSet` reads `*string` **before** it
tests the length, so every caller answers the null string first (`packHex`, `x2b` and `b2x` each
have their own early return). `convert.rs` does the same, and `validate_grouped`'s doc says so.

---

## 2. The probe table

Every probe was run as
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx ABS )`
from a freshly `mkdir`ed scratchpad subdirectory, with stdout, stderr and status read as three
separate descriptors and the status read unpiped.

### 2.1 Step 0's eight behaviours, re-verified

| # | probe | oracle |
|---|---|---|
| a | `numeric digits 9; say c2d(copies('00'x,10)||'01'x)` | `1` rc 0 |
| a | `numeric digits 9; say c2d('ffffffff'x)` | `93.936 C2D result is not a valid whole number with NUMERIC DIGITS 9.` rc 163 |
| a | `numeric digits 1; say c2d('ff'x,1)` | `-1` rc 0 |
| a | `numeric digits 1; say c2d('7f'x,1)` | `93.936 ... DIGITS 1.` rc 163 |
| b | `say c2d('01020304'x,2)` | `772` |
| b | `say d2x(4096,2)` | `00` |
| b | `say x2d('80',1)` | `0` |
| c | `say c2d('80'x)` / `,1` / `,2` | `128` / `-128` / `128` |
| c | `say x2d('80')` / `,1` / `,2` | `128` / `0` / `-128` |
| c | `say d2x(-1)`, `say d2c(-1)` | `93.927 Length must be specified to convert a negative value.` rc 163 |
| d | `say c2x(bitand('ffff'x,'00'x))` | `00FF` |
| d | `say c2x(bitand('ffff'x,'00'x,'00'x))` | `0000` |
| d | `say c2x(bitand('ffff'x))` | `FFFF` |
| d | `bitor('ffff'x,'00'x)`, `bitxor('ffff'x,'00'x)`, `bitor('0000'x)`, `bitxor('0000'x)` | `FFFF`, `FFFF`, `0000`, `0000` |
| e | `say x2c('41'||'09'x||'42')` | `AB` |
| e | `say x2c('41'||'0a'x||'42')` | `93.933 ... character found "<LF>".` rc 163 |
| e | `say x2c(' 4142')` / `say x2c('4142 ')` | `93.931` position 1 / position 5, rc 163 |
| f | `x2c('414')` / `x2c('4 1424')` / `x2c('414 2434')` | `0414` / `041424` / `04142434` |
| f | `x2c('414 243')` | `93.976 Hexadecimal strings must be grouped in units that are multiples of two characters.` rc 163 |
| g | `x2b('414')` / `b2x('11000011')` / `b2x('1')` / `b2x('11')` / `x2b('4')` | `010000010100` / `C3` / `1` / `3` / `0100` |
| h | `xrange('a','b','c','d')` | `abcd` |
| h | `length(xrange())` / `length(xrange('cntrl'))` | `256` / `33` |
| h | `c2x(xrange('cntrl'))` | `000102...1E1F7F` (33 bytes) |
| h | `c2x(xrange('digit','z'))` | **`7A7B...FF` only -- 134 bytes, no digits** (see §7) |

### 2.2 The grouping and whitespace family

| probe | oracle |
|---|---|
| `x2c('41 42  ')` | 93.931 position **7** (the last of the trailing run) |
| `x2c('4142'||'09'x)` | 93.931 position 5 |
| `x2c('41'||'ff'x)` | 93.933 `character found "\377"` |
| `x2c('41'||'01'x)` / `x2c('41'||'00'x)` | 93.933 `character found "?"` (the report's byte-sanitiser) |
| `b2x('1'||'ff'x)` | 93.934 `character found "\377"` |
| `b2x('1012')` | 93.934 `character found "2"` |
| `b2x(' 1010')` / `b2x('1010 ')` | 93.932 position 1 / position 5 |
| `b2x('101 0000')` | `50` (residue 3, then an exact group of 4) |
| `b2x('1 0000 0011')` / `b2x('11 0000 0011')` | `103` / `303` |
| `x2c('4 4')` / `x2c('44 4')` | 93.976 both |
| `x2c('41 42 43')` | `ABC` |
| `x2b('4 1424')` / `x2b('414 2434')` | 20 bits / 28 bits, unpadded |
| `x2d(' ')` | 93.931 position 1 |
| `b2x('')`, `x2b('')`, `c2x('')`, `x2c('')` | all the null string |

### 2.3 The numeric four

| probe | oracle |
|---|---|
| `c2d('')`, `x2d('')`, `d2x(0)` | `0`, `0`, `0` |
| `d2c(0)` | `'00'x` -- one NUL byte, **not** the null string |
| `c2d('abc',0)`, `x2d('zz',0)` | `0`, `0` -- the zero length shortcut precedes all validation |
| `d2x(255,0)`, `d2c(255,0)` | both the null string |
| `d2c(1)`, `d2c(256)`, `d2c(255,1)`, `d2c(255,3)`, `d2c(-1,1)`, `d2c(-1,3)` | `01`, `0100`, `FF`, `0000FF`, `FF`, `FFFFFF` (as `c2x`) |
| `d2x(255,1)`, `d2x(255,5)`, `d2x(-1,3)`, `d2x(-255,3)`, `d2x(-16,1)`, `d2x(-16,2)` | `F`, `000FF`, `FFF`, `F01`, `0`, `F0` |
| `d2x(0,3)`, `d2c(0,3)`, `d2x(0,0)` | `000`, `000000`, `` |
| `x2d('8f',1)`, `x2d('18f',1)`, `x2d('18f',2)`, `x2d('18f',3)` | `-1`, `-1`, `-113`, `399` |
| `x2d('ff00',2/3/4/5)` | `0`, `-256`, `-256`, `65280` |
| `c2d('ff00'x,1)`, `c2d('ff00'x,2)`, `c2d('0080'x,2)` | `0`, `-256`, `128` |
| `c2d('01'x,5)`, `c2d('ff'x,5)`, `x2d('ff',5)`, `x2d('f',2)` | `1`, `255`, `255`, `15` |
| `numeric digits 3; c2d('ff'x)` / `c2d('ffff'x)` | `255` / 93.936 naming `3` |
| `numeric digits 3; x2d('ff')` / `x2d('ffff')` | `255` / 93.935 naming `3` |
| `numeric digits 1; x2d('ff',2)` / `x2d('8f',2)` | `-1` / 93.935 naming `1` |
| `numeric digits 1; c2d('00'x)` / `c2d('0000000000'x)` | `0` / `0` |
| `numeric digits 20; c2d('ffffffffff'x)` | `1099511627775` |
| `numeric digits 1000; length(c2d(copies('ff'x,100)))` | `241` |

### 2.4 `D2X`/`D2C`'s input-side precision rule

| probe | oracle |
|---|---|
| `numeric digits 5; d2x(12345)` / `d2x(123456)` | `3039` / 93.928 `found "123456"` |
| `numeric digits 5; d2x(1e5)` | 93.928 `found "1E5"` |
| `numeric digits 5; c2x(d2c(123456))` | 93.929 `found "123456"` |
| `numeric digits 3; d2x('000123')` / `d2x('12300')` | `7B` / 93.928 `found "12300"` |
| `numeric digits 3; d2x('1E2')` / `d2x('1E3')` | `64` / 93.928 |
| `numeric digits 3; d2x(999)` / `d2x(1000)` | `3E7` / 93.928 |
| `d2x('1E8')` / `d2x('1E9')` at DIGITS 9 | `5F5E100` / 93.928 |
| `d2x('  12  ')`, `d2x('+12')`, `d2x('- 12',4)`, `d2x('09'x||'12'||'09'x)` | `C`, `C`, `FFF4`, `C` |
| `d2x('1.0')`, `d2x('12.00')`, `d2x('1.23E4')`, `d2x('1200E-2')` | `1`, `C`, `300C`, `C` |
| `d2x('1.5')`, `d2x('1.50')`, `d2x('1234E-2')` | 93.928 each, `found` the argument's own text |
| `numeric digits 1; d2x('1.4')` / `numeric digits 2; d2x('1.4')` | `1` / **93.928** |
| `numeric digits 1; d2x('1.6')` | 93.928 |
| `numeric digits 2; d2x('1.04')` / `numeric digits 3; d2x('1.04')` | `1` / **93.928** |
| `d2x('1.0000000000004')` at DIGITS 9 | `1` |
| `d2x('0.0')`, `d2x('-0.0')`, `d2x('0.00000')`, `d2x('0E5')` | `0` each |
| `d2x('0.5')`, `d2x('0.01')`@1, `d2x('1E-9')`, `d2x('1E-100')`, `d2x('0.0000000001')`@9 and @20, `d2x('123E-4')`, `d2x('0.00000000000000001')` | 93.928 each |
| `numeric digits 3; d2x(-1000)` | 93.928 `found "-1.00E+3"` -- **D15: the value's rendering at creation** |
| `numeric digits 3; d2x(-1.5)` | 93.928 `found "-1.5"` |
| `numeric digits 3 ; zz = 2/3 ; numeric digits 9 ; d2x(zz)` | 93.928 `found "0.667"` |

### 2.5 Arity, interior omissions, and the two error layers

| probe | oracle |
|---|---|
| `b2x()` | 40.3 minimum 1, rc 216 |
| `b2x('1','2')` | 40.4 maximum 1 |
| `c2d('a','1','2')` | 40.4 maximum 2 |
| `bitand('a','b','c','d')` | 40.4 maximum 3 |
| `c2x('a','b')`, `x2c()` | 40.4 maximum 1, 40.3 minimum 1 |
| `c2d(,2)` | **40.5** argument 1 is required |
| `bitand(,'a')`, `bitor(,'a','b')`, `d2x(,'1')`, `x2d(,'1')` | 40.5 argument 1 |
| `b2x(,'x')`, `c2x(,'x')` | **40.4** -- the maximum is checked first, so a max-1 row can never show 40.5 |
| `b2x(,)` | 40.3 -- both omissions are trailing and are dropped |
| `c2d('ff'x,)`, `bitand('ffff'x,,'00'x)` | `255`, `0000` -- omissions past the minimum are legal |
| `xrange('a','b','c','d','e','f','g','h')` / twelve arguments | 8 bytes / 12 bytes, never 40.4 |
| `d2c('abc','def')` | 40.12 `D2C argument 2 must be a whole number; found "def".` rc 216 |
| `x2d('ZZ','zz')` / `x2d('ZZ',4)` | 40.12 / 93.933 -- the conversion beats the content check |
| `c2d('abc',-1)`, `d2x(1,-1)`, `x2d('ZZ',-1)`, `d2x('1.5',-1)`, `d2x(-1,-1)`, `numeric digits 3; d2x(1000,-1)` | 93.923 `found "-1"` rc 163 |
| `d2c('abc',-1)` / `d2x('abc',-1)` | **93.929 / 93.928**, not 93.923 -- the value's own check runs in `RexxString::d2c` before `d2xD2c` reads the length |
| `bitand('ab','cd','xx')` | 40.23 `BITAND argument 3 must be a single character; found "xx".` |
| `numeric digits 3 ; zz = 2/3 ; numeric digits 9 ; bitand('ab','cd',zz)` | 40.23 `found "0.667"` (D15) |
| `c2d('abc','1.0000000000000000000004')` / `c2d('abc','1E18')` | `99` / 40.12 -- `ARGUMENT_DIGITS` is 18 |
| `length(d2x(1,123456789012345678))` | **Error 5, rc 251** |
| `c2d('ff'x,123456789012345678)` | `255` -- a read length is not a size |
| `length(d2x(1,1000))` | `1000` |

### 2.6 `XRANGE`

| probe | oracle |
|---|---|
| `xrange()`, `xrange(,)`, `xrange('00'x)`, `xrange('00'x,'ff'x)` | 256 bytes |
| `xrange(,'z')`, `xrange('a',)`, `xrange('digit',)` | 123, 159, 10 bytes |
| `xrange('7f'x,'80'x)`, `xrange('ff'x,'00'x)`, `xrange('ff'x,'ff'x)`, `xrange('80'x,'7f'x)`, `xrange('a','a')` | `7F80`, `FF00`, `FF`, 256 bytes, `a` |
| all twelve classes, and `DIGIT`/`DiGiT`/`CNTRL` | the tables in §6, case-insensitive |
| `xrange('zork')`, `xrange('')` | 40.28 argument 1, `found "zork"` / `found ""` |
| `xrange('c1','c1')` | 40.28 argument 1 |
| `xrange('a','zz')`, `xrange('a','')`, `xrange('a','upper')` | 40.23 argument 2 |
| `xrange('a','b','zork')` / `xrange('a','b','c','zz')` | 40.28 argument 3 / 40.23 argument 4 |
| `xrange('digit','z')`, `xrange('cntrl','z')` | 134 bytes, `'z'`..`'ff'x` only |
| `xrange('digit','z','q')` | digits + `'z'`..`'ff'x`..`'q'` (258 bytes) |
| `xrange('upper','lower')`, `xrange('digit','Alpha')`, `xrange('CNTRL','CNTRL')` | alpha, alnum, cntrl twice |
| `xrange('digit','9','a')`, `xrange('a','b','digit')`, `xrange('upper','a','z')` | digits+`'9'..'a'`, `ab`+digits, alpha |

---

## 3. The corpus: axes, and how they were crossed

The corpus is generated (`gen.py` in the scratchpad) and run through both interpreters by an
eight-way parallel differential harness. **9,326 programs, 0 mismatches**, re-run on the final
formatted build.

Axes, and the crossings rather than the unions:

| group | axes crossed |
|---|---|
| `C2X`/`X2C` | 12-value **byte alphabet** × direct call and round trip; plus all 256 single bytes, each in both directions and in both letter cases |
| `X2C`/`X2B`/`B2X` | 27 hexadecimal and 21 binary **text shapes** (valid, odd, grouped at every residue, leading/interior/trailing whitespace, tab as well as blank, invalid byte at `0x00`/`0x01`/`0x0a`/`0xff`/`'z'`) × two output notations |
| `BITAND`/`BITOR`/`BITXOR` | 13 first strings × 9 second strings (including omitted and null) × 5 pads (including omitted) × 3 operations = 1,755, plus 8 pad shapes including the null string and a two-byte one, plus interior omissions and 40.4 |
| `C2D`/`X2D` | 7 **DIGITS** settings (omitted, 1, 2, 3, 9, 18, 40) × 13 byte subjects or 20 hexadecimal subjects × 11 (or 8) **lengths** -- omitted, 0, 1, 2, 3, 4, 5, −1, `'x'`, `' 2 '`, `1000000` |
| `D2C`/`D2X` | 7 DIGITS × 41 **values** (zero spellings, boundaries at 127/128/255/256, negatives, decimals, exponents, leading and trailing zeros, non-numeric, a byte string, a NUL) × 9 lengths |
| D15 | 10 builtins whose result is created at `DIGITS 12` and rendered at `DIGITS 1` |
| `XRANGE` | 17 class names (including three miscases, two invalid, the null string) × 4 argument positions; 10 characters (including omitted, null, two-byte, `'00'x`, `'ff'x`, `'80'x`, `'09'x`) × 10 in both slots; class-before-range and range-before-class at three and four arguments; wrap-around at every boundary |
| arity | every row at `min - 1` and `max + 1` |

**Every operand alphabet contains the null string, a byte at or above `0x80`, a control byte and a
NUL**, and the two whitespace bytes appear as data as well as as separators.

The crossings that matter (each is a case a one-axis-at-a-time corpus would miss):

* **length × DIGITS.** `c2d('ff'x,1)` is `-1` at `DIGITS 1`; `c2d('7f'x,1)` is an error at the same
  setting. Varying the length at `DIGITS 9`, or the setting with no length, sees neither.
* **length parity × sign bit.** `x2d('18f',1)`, `,2` and `,3` are `-1`, `-113` and `399`. Only the
  odd/even crossing reaches the `0x08`-vs-`0x80` sign test and the post-negation mask.
* **pad × unequal lengths × operation.** `bitand('ffff'x,'00'x)` = `00FF` needs the pad omitted *and*
  the lengths unequal; equal lengths agree under every candidate default.
* **decimals × DIGITS.** `d2x('1.4')` is `1` at `DIGITS 1` and an error at `DIGITS 2`; `d2x('1.04')`
  is `1` at 2 and an error at 3. A fixed-precision corpus sees one side only.
* **class × argument count.** `xrange('digit','z')` and `xrange('digit','z','q')` differ in *what the
  class contributes*, not in the range.

---

## 4. Negative control

Eight mutations of `convert.rs`, applied one at a time and restored from a copy (never
`git checkout --`). The harness asserts `passed + failed == 20` on every run, so "the test does not
exist" cannot read as "the test passed".

| mutation | result |
|---|---|
| M1 `BITAND`'s default pad becomes `'00'x` | CAUGHT by `an_omitted_pad_leaves_the_longer_strings_tail_alone` |
| M2 `pack_hex` takes the odd nibble last | CAUGHT by 4 tests (`x2c_pads_an_odd_nibble_where_x2b_does_not`, the grouping test, both length tests) |
| M3 the grouping check demands residue 0 rather than the first group's | CAUGHT by 3 tests |
| M4 `XRANGE`'s early return keeps the preceding class | CAUGHT by `a_two_argument_call_ending_in_a_range_drops_a_preceding_class` |
| M5 `C2D` bounds the input length rather than the result | CAUGHT by `the_precision_bounds_the_result_one_way_and_the_value_the_other` |
| M6 the `cntrl` table stops at its leading NUL | CAUGHT by `every_character_class_is_the_oracles_own_table` |
| M7 `X2D`'s odd-length sign bit is `0x80` | CAUGHT by `a_length_makes_the_read_a_signed_window` |
| M8 `D2X` pads a negative value with `0` | CAUGHT by 2 tests |

**And "can fail" against "adds coverage":** M1 and M6 were also run against the whole workspace with
`--skip builtin::convert`. Both reported **1073 passed, 2 failed** -- byte for byte the same two
failures the unmutated tree had at that moment (`builtin_status` and `keyword_assertions`, both
waiting for the corpus-file updates below). So neither mutation is visible to anything that existed
before this task; the new tests are the only thing that catches them. That is expected rather than
lucky -- all twelve names answered `Loud::unresolved_call` before this commit, so no pre-existing
test could reach the code at all.

Note the harness itself needed a fix mid-run: the first version keyed each worker's directory on the
job index modulo 8, which two pool workers can collide on. It reported 2,566 "mismatches" that were
pure interleaving. Keying on `os.getpid()` fixed it, and the fixed harness was re-checked with a
positive control (`say date()` -> 1 mismatch) before and after every real run.

---

## 5. Corpus-file changes

* `rust/corpus/builtin-status.txt`: the twelve rows `B2X BITAND BITOR BITXOR C2D C2X D2C D2X X2B
  X2C X2D XRANGE` flip `loud` -> `implemented`. **None is `divergent`.** Re-derived by
  `cargo test --offline -p rexx-exec --test builtin_status` -- 12 passed, exit 0.
* `rust/corpus/keyword-exempt.txt`: **`ASSIGNMENT::test_8` removed** (it now passes). It is removed,
  not re-attributed. Header count `4c 771 bodies` -> `4c 770 bodies`; the file total is 776 rows
  (770 + 6 `defect:`), which is what `the_exempt_set_matches_the_current_failures` now measures.
  The report line reads `120 of 896`.

No `.rex` corpus programs were added, matching Tasks 3 and 4: the crossed coverage lives in
`convert.rs`'s own 20 tests plus the 9,326-program sweep above.

---

## 6. Things the brief got wrong, or did not have

1. **`xrange('digit','z')` does not prepend the digits.** The brief says "`'z'` starts a *new* range
   running to `0xFF`", which implies digits followed by that range. Measured, the answer is 134 bytes:
   **`'z'` through `'ff'x` and nothing else.** `BUILTIN(XRANGE)`'s early return
   (`BuiltinFunctions.cpp:1697`) fires on `mode == CALC_LENGTH && argcount <= 2` and returns the range
   it has just computed, discarding `totalLength` and everything the loop accumulated. Two class names
   never reach that branch, so `xrange('upper','lower')` really is the full alphabet
   (`XRANGE.testGroup:273` asserts it), and the three-argument `xrange('digit','z','q')` does include
   the digits. This is the one place the brief's stated behaviour and the oracle disagree; the
   measurement wins and the code and its module doc follow the measurement.
2. **There *is* a default pad; it is the identity element.** The brief says "There is no default pad;
   there is a passthrough." `RexxString::bitAnd` calls
   `optionalPadArgument(pad, (char)0xff, ARG_TWO)` and `bitOr`/`bitXor` pass `(char)0x00`
   (`classes/StringClassBit.cpp`). The *observable* behaviour the brief describes is right -- the tail
   survives -- but it survives because `x & 0xff == x` and `x | 0x00 == x`, not because the tail is
   copied unprocessed. The correction matters for anyone reading the code: the implementation takes a
   `default_pad` parameter rather than branching on whether a pad was given.
3. **A max-1 row cannot show 40.5.** The brief (and the controller's point 3) asks for an interior
   omission before every optional position. For `B2X`, `C2X`, `X2B` and `X2C` the only such call has
   two arguments, and the maximum is checked first: measured, `b2x(,'x')` is **40.4**, not 40.5. The
   test asserts that split explicitly rather than asserting 40.5 for all eleven rows.
4. **`d2c('abc',-1)` is 93.929, not 93.923.** The brief's validation-order rule ("every `40.x` check
   precedes every `93.9xx` check") holds, but *within* the 93.9xx family `D2X`/`D2C` and `C2D`/`X2D`
   order the length's range check differently: `RexxString::d2c` builds the `NumberString` before
   calling `d2xD2c`, where `x2dC2d` calls `optionalLengthArgument` as its first statement. Measured
   both ways round, and asserted.
5. **`93.977` is still tested nowhere in `ootest/base`**, as the brief says -- confirmed by
   `/bin/grep -aho "expectSyntax( *[0-9.]*"` over the twelve groups, which yields 17 distinct numbers
   and no `93.977`. `convert.rs` asserts it in three shapes.
6. The brief's `XRANGE` note that "argument 1 takes a class name **or** a single character (40.28);
   argument 2 takes a single character **only** (40.23)" is exactly right, including that a *class
   name* in position 2 is 40.23 rather than 40.28.

---

## 7. Design notes worth carrying forward

* **`Number`'s digits and exponent are `pub(crate)` to `rexx-num` and unreachable from `rexx-exec`.**
  `value.rs`'s `small_int_for` already documents working around this by *rendering*. `D2X`/`D2C` need
  the significand and exponent, so `convert.rs` scans the argument's **rendered text** -- which is
  exactly what the oracle does, since `BUILTIN(D2X)` takes `required_string` and
  `RexxString::numberString()` rebuilds a `NumberString` from those same bytes. Acceptance stays
  `Interp::to_number`'s; the scan only takes apart what the parser already accepted, and
  `a_scan_takes_apart_every_text_the_number_parser_accepts` holds the two to that, one-directionally
  (the scan is free to be looser, and is: it does not range-check the assembled exponent).
  **If a later task wants this cleanly, the fix is a public accessor on `rexx-num::Number`** -- it was
  out of this task's file scope.
* **`hasSignificantDecimals` reads before the start of the digit array** when
  `digitsCount + numberExponent < 0`. Every value that reaches it is a non-zero magnitude below 0.1,
  and the oracle answers "significant" (i.e. refuses) for all six such probes measured
  (`d2x('0.01')`@1, `1E-9`, `1E-100`, `0.0000000001`@9 and @20, `123E-4`, `0.00000000000000001`).
  `convert.rs` returns `true` for that case and says why, rather than modelling the out-of-bounds read.
* **The working buffers are reproduced, not skipped.** `d2xD2c` allocates
  `max(currentDigits, resultSize) + OVERFLOWSPACE` (with `resultSize` already doubled for `D2C`)
  *before* it computes any digit, which is where `d2x(1,123456789012345678)` becomes Error 5 at rc 251
  rather than a very long wait; `x2dC2d` allocates `currentDigits + OVERFLOWSPACE + 1`. Both are asked
  for through `builtin::buffer` and dropped, so the refusal is the allocator's.
* `unsafe`: none. The workspace's `unsafe_code = "forbid"` is untouched.

---

## 8. The verify block, four unpiped exit statuses

Run from `rust/`, each status read unpiped:

```
cargo test --offline --workspace --no-fail-fast                    exit 0    1095 passed, 0 failed
cargo fmt --all --check                                            exit 0
cargo clippy --offline --workspace --all-targets -- -D warnings    exit 0
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus exit 0    STRICT, 42 of 42 matching
```

Because a same-session green `clippy` is provisional, it was **also run from a clean target
directory** (`CARGO_TARGET_DIR=<scratchpad>/clean-target`): exit 0, and the log shows every crate
being checked rather than reused.

The workspace count moved 1,075 -> 1,095: the 20 new tests in `builtin::convert`.

---

## 9. Anything not done

* No `.rex` file was added to `rust/corpus/`; there is no `phase-4c.txt` and Tasks 3 and 4 did not
  create one, so the 9,326-program sweep lives in the scratchpad and in this report rather than in
  the repository. If the phase wants a standing 4c corpus file, that is a decision above this task.
* ~~A divergence that exists but was not chased: `NUMERIC DIGITS` above roughly 500 million...~~
  **This note was backwards and is withdrawn** (fix round 1, Important 4). `x2d_c2d` and `d2x_d2c`
  each request the `digits`-sized working buffer up front through `builtin::buffer`, exactly as
  `StringClassConversion.cpp:550` and `NumberStringMath.cpp:98-107` do, so the accumulator does
  **not** grow lazily past a point the oracle would have refused, and the two behave the same way at
  any precision. The only asymmetry that remains at that scale is the 512 MiB `INTERPRETER_STACK_BYTES`
  reservation under `ulimit -v`, which is **already owned** by
  `docs/superpowers/plans/phase-4-exclusions.txt`'s "A LARGE RESULT ABORTS WHERE THE ORACLE RETURNS"
  row, second cause -- recorded there as not Phase 4's and belonging to whoever reopens D19. **No
  `KNOWN GAP` row is needed or added.**
* Commit: `ca0b63f917ddd33ccf4a4bc6ccf91b7b3648c3cb`. `git status` clean afterwards.

---

## 10. Fix round 1

Review: `.superpowers/sdd/2026-08-04-phase-4c-builtins-and-parse/task-5-review.md`. Spec compliance
passed; five quality items. All five addressed. Commit `c8f33d9ce0c34f29badd023d99f8a4b06391b2b0` (`c8f33d9c`), read back from `git log`; working tree clean.

### Important 1 -- the per-gap residue check is now pinned

The reviewer's finding is confirmed. Deleting the per-gap arm of `validate_grouped` and keeping only
the end-of-string one left **all 20 of the original tests green and the whole package green under
`REXX_CORPUS_GATE=1`**, while three strings converted where the oracle raises. Every refusal the
original tests asserted also failed the end-of-string check, so nothing separated the two readings.

The five witnesses, re-measured here on the oracle before being written into a test. Each breaks the
residue at an *interior* gap and then returns to it, so the end-of-string check passes on all five:

| probe | oracle | totals at the gaps | total at the end |
|---|---|---|---|
| `x2c('41 4 1 42')` | 93.976 rc 163 | 2 (residue 0), 3, 4 | 6 ≡ 0 |
| `x2c('414 2 434')` | 93.976 rc 163 | 3 (residue 1), 4 | 7 ≡ 1 |
| `x2c('41 4 1 4 2')` | 93.976 rc 163 | 2 (residue 0), 3, 4, 5 | 6 ≡ 0 |
| `b2x('1010 10 10')` | 93.977 rc 163 | 4 (residue 0), 6 | 8 ≡ 0 |
| `b2x('1010 101 0101')` | 93.977 rc 163 | 4 (residue 0), 7 | 12 ≡ 0 |

Paired with four adjacent successes that hold the residue at every gap they have, which is what pins
the refusals to the interior gap rather than to "more than two groups": `x2c('4 14 24')` = `041424`,
`x2c('41 42 43')` = `414243`, `b2x('101 0000 0000')` = `500`, `b2x('1010 1010 1010')` = `AAA`. All
nine measured on the oracle, and all nine added to the differential corpus (now **9,334 programs,
0 mismatches**).

New test: `the_residue_is_checked_at_every_gap_and_not_only_at_the_end`.

**The red/green states observed**, with the run count asserted at 21 on both so that "the test does
not exist" cannot read as "passed", and the file restored from a copy rather than with
`git checkout --`:

```
MUTATED (per-gap check deleted):  FAILED -- 20 passed, 1 failed
                                  the only failure: the_residue_is_checked_at_every_gap_and_not_only_at_the_end
RESTORED:                         ok     -- 21 passed, 0 failed
```

**And it adds coverage rather than merely being able to fail.** With the same mutation applied and
only the new test skipped:

```
MUTATED -- cargo test --workspace -- --skip the_residue_is_checked   1095 passed, 0 failed
MUTATED -- REXX_CORPUS_GATE=1 ... --test corpus                         9 passed, 0 failed
```

So nothing else in the tree catches it, which is exactly the gap the reviewer identified.

### Important 2 -- the false sentence in §1

Corrected. `b2x('101 0000')` separates *residue-from-the-first-group* from *residue-always-0* (that is
mutation M3 in §4); it passes under both readings of *where* the check runs and is therefore not a
witness for the per-gap arm. §1 now says which reading each witness separates and points at §10.

### Important 3 -- the mutable in-repo aggregate in `builtin/mod.rs`

"The one variadic row" is gone. Re-measured rather than taken on trust: `MAX_Max` and `MIN_Max` are
`argcount` in `BuiltinFunctions.cpp:1996` and `:2025`, and on the oracle `say max(1,2,3,4,5,6,7,8)`
prints `8` at rc 0 -- so the sentence would have been false the moment `MAX`/`MIN` landed. The comment
now states the property (`max: None` because `XRANGE_Max` is `argcount` itself) and keeps the two
oracle measurements, with no count of rows in the table it sits in.

### Important 4 -- the untested-divergence note was backwards

Withdrawn in §9, with the reasoning: both `x2d_c2d` and `d2x_d2c` ask for the `digits`-sized working
buffer up front through `builtin::buffer`, matching `StringClassConversion.cpp:550` and
`NumberStringMath.cpp:98-107`, so there is no precision-driven asymmetry to record. The remaining
asymmetry at that scale is the 512 MiB `INTERPRETER_STACK_BYTES` reservation under `ulimit -v`, and
`docs/superpowers/plans/phase-4-exclusions.txt` already owns it under "A LARGE RESULT ABORTS WHERE THE
ORACLE RETURNS", second cause, explicitly recorded there as not 4c's. **No `KNOWN GAP` row added.**

### Minor 5 -- `xrange`'s `count == 1` early return

Deleted, having first confirmed it is behaviourally dead: with one argument the loop condition
`position < count` is already false after the class branch's `continue`, so the two-pass path builds
the same bytes from the same table. Verified rather than reasoned -- `xrange('digit')`,
`length(xrange('cntrl'))` and the sweep's 17 class names × 4 positions all still match the oracle
across the 9,334-program run. The comment at that site now says the oracle has the shortcut and why
it is not reproduced, and contrasts it with the `count <= 2` shortcut, which is **not** dead and
changes the answer.

### The two minors not actioned

Recorded as the reviewer asked, unchanged: `a_scan_takes_apart_every_text_the_number_parser_accepts`
has no floor on how many of its subjects actually parse, so it would pass over a corpus that parsed
nothing; and `shift_in`/`render_decimal`/`scan_decimal` allocate infallibly outside `builtin::buffer`.

### Verify block, four unpiped exit statuses

```
cargo test --offline --workspace --no-fail-fast                    exit 0    1096 passed, 0 failed
cargo fmt --all --check                                            exit 0
cargo clippy --offline --workspace --all-targets -- -D warnings    exit 0
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus exit 0    STRICT, 42 of 42 matching
```

1095 -> 1096 is the one new test. The differential sweep is 9,334 programs, 0 mismatches.
