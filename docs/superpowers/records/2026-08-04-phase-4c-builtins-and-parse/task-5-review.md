# Task 5 review -- `builtin/convert.rs`, the twelve conversion builtins

Reviewed `ca0b63f9` against parent `add307c7`. HEAD (`56c73fe1`, documentation-only) ignored.
Tree was clean before and after; every mutation below was restored from my own copy
(`md5sum` re-checked, `git status --porcelain` empty), never from git.

**Spec compliance: PASS.**
**Quality: CHANGES-REQUESTED** -- one demonstrated coverage hole, one comment of the class this
project has already paid for twice, and two report statements that are false as written. No
builtin semantic is wrong.

---

## 1. Verification, re-run, each status unpiped

```
cargo test --offline --workspace --no-fail-fast                    exit 0   1095 passed, 0 failed
cargo fmt --all --check                                            exit 0
cargo clippy --offline --workspace --all-targets -- -D warnings    exit 0
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus exit 0   STRICT, 42 of 42 matching
```

`builtin::convert` contributes 20 tests; the workspace total of 1095 matches the report.

Because a same-session green clippy is provisional, it was re-run with
`CARGO_TARGET_DIR=<scratchpad>/rev5/clean-target`: **exit 0**, 252 MB of fresh artefacts, and the
log names all eight workspace crates being checked (`rexx-core`, `rexx-num`, `rexx-parse`,
`rexx-exec`, `rexx-oracle`, `rexx-extract`, `rexx-inventory`, `rexx-bench`) rather than reusing them.

Supporting gates:

```
cargo test --offline -p rexx-exec --test builtin_status       exit 0   12 passed
cargo test --offline -p rexx-exec --test keyword_assertions   exit 0   7 passed, "120 of 896"
```

`no unsafe`: `/bin/grep -ac unsafe convert.rs` is 0, and `Heap::alloc`/`alloc_with_uncollected`
appear zero times. Every string result goes through `Interp::text`/`text_owned`, and
`text_owned` (`value.rs:63`) is `self.alloc_with(BehaviourId::STRING, ...)`.

## 2. My own differential, and its positive control

A byte-exact harness (stdout **and** stderr **and** rc compared raw; the oracle wrapped exactly as
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx ABS )`; every run from a
freshly `mkdir`ed subdirectory with absolute paths) against `target/release/rexx-run`.

**Positive control first**: `say date()` reports as a mismatch (`5 Aug 2026`, rc 0 against rc 120
`routine "DATE" is not implemented`). The harness can fail.

| set | programs | mismatches |
|---|---|---|
| Step 0's eight behaviours, each as its own program | 26 | 0 |
| adversarial set: grouping, `XRANGE` classes, `DIGITS` boundaries, arity, error substitutions | 145 | 0 |
| second adversarial set: scan corners, wraps, `~class`, exponent overflow | 72 | **8**, all out of scope |
| **independent re-run of the task's own 9,326-program sweep** | 9,326 | **0** |

The eight are `~class` message sends (`Phase 5`) and `DATATYPE` (`4c`, another task's row). None
touches the twelve.

The 9,326 figure is real: `gen.py` survives in the session scratchpad, regenerates **9,326**
programs, and its operand alphabets do contain the null string, a byte at or above `0x80`, control
bytes and a NUL as §3 claims. I re-ran the whole sweep through both interpreters myself with
eight workers keyed on `os.getpid()` -- **0 mismatches**, with the same `say date()` control.

Notable individual confirmations, all matching:

* `x2b('4 142')`, `b2x('1 000')`, `b2x('11 000')` -- the odd-residue refusals.
* `xrange(65)` -> 40.28, `xrange('a',65)` -> 40.23.
* `d2x('1E999999999999')`, `d2x('1E-999999999999')`, `d2x('09'x||'12'||'09'x)`, `d2x('  -  12  ',4)`.
* `x2d('ff',0)`, `c2d('ff'x,0)`, `x2d(' ',0)` -- the zero-length shortcut ahead of every scan.
* `numeric digits 12; qq = c2d('0f4240'x); numeric digits 1; say qq` -- D15, both directions.

**A brief hazard worth recording:** `say c2d('ff'x)~class` is **`The String class`** on the oracle.
The brief's Step 0 note that `class/RexxInteger.testGroup` "requires `d2x`/`c2d`/`x2d` to return a
RexxInteger" is about the *method* forms; the **BIF** returns a String, which is what this crate
returns. That risk area is clear.

## 3. The eight Step 0 behaviours, each against the shipped code

Every one re-measured on the oracle and re-run through `rexx-run`:

| # | probe | oracle | rust |
|---|---|---|---|
| a | `numeric digits 9; c2d(copies('00'x,10)||'01'x)` / `c2d('ffffffff'x)` | `1` / 93.936 naming 9 | same |
| a | `numeric digits 1; c2d('ff'x,1)` / `c2d('7f'x,1)` | `-1` / 93.936 naming 1 | same |
| b | `c2d('01020304'x,2)` / `d2x(4096,2)` / `x2d('80',1)` | `772` / `00` / `0` | same |
| c | `c2d('80'x)`/`,1`/`,2` and `x2d('80')`/`,1`/`,2` | `128 -128 128` / `128 0 -128` | same |
| c | `d2x(-1)`, `d2c(-1)` | 93.927 | same |
| d | `c2x(bitand('ffff'x,'00'x))` / `,'00'x` / `bitand('ffff'x)` | `00FF` / `0000` / `FFFF` | same |
| e | `x2c('41'||'09'x||'42')` / `'0a'x` / `' 4142'` / `'4142 '` | `4142` / 93.933 / 93.931@1 / 93.931@5 | same |
| f | `x2c('414')`/`('4 1424')`/`('414 2434')`/`('414 243')` | `0414`/`041424`/`04142434`/93.976 | same |
| g | `x2b('414')` vs `x2c('414')`; `b2x('1')`,`b2x('11')` | 12 bits / `0414`; `1`,`3` | same |
| h | `xrange('a','b','c','d')`, `length(xrange())`, `length(xrange('cntrl'))` | `abcd`, 256, 33 | same |
| h | `c2x(xrange('digit','z'))` | `7A7B..FF`, **134 bytes, no digits** | same |

`c2x(xrange('cntrl'))` is `000102...1E1F7F`, 33 bytes -- and the table
(`convert.rs:896`) is a byte slice with an explicit length; `character_class` returns `&'static [u8]`
and `Piece::Class` uses `table.len()`. Nothing in the file constructs or consumes a C string.

## 4. The grouping rule, read from the C++ myself

`StringUtil::validateGroupedSet` (`StringUtil.cpp:744-812`) is exactly as the report transcribes,
including `spaceLocation = current` after the post-increment (so the trailing error names the
**last** byte of a run) and the uninitialised `char ch` that makes the null string every caller's
own problem.

**The brief's item 3 asks for an input where the cumulative rule and the naive per-group rule
differ. There is none, and this is provable rather than merely unobserved:** the residue is
compared at *every* whitespace byte, so the partial sums `S_1..S_k` are all congruent to
`S_1`; consecutive differences give `len(g_i) = 0 (mod m)` for every `i >= 2`, which is the naive
rule. The two are the same predicate. `convert.rs`'s module doc says so and is right.

**But there is a third reading that does differ, and it is the one the code is not protected
against** -- see finding 1.

## 5. The four "the brief was wrong" claims, verified independently

1. **`xrange('digit','z')` discards the digits.** Measured: 134 bytes, `7A`..`FF`. Confirmed.
   `xrange('digit','z','q')` does include them, and `xrange('upper','lower')` is the full alphabet.
   The code's `if count <= 2 { return ... }` reproduces `BUILTIN(XRANGE)`'s early return.
2. **There is a default pad and it is the identity element.** Read from the C++, not the report:
   `StringClassBit.cpp:66` is `optionalPadArgument(pad, (char)0xff, ARG_TWO)` for `bitAnd`, and
   `:144`/`:220` are `(char)0x00` for `bitOr`/`bitXor`. The surrounding body (`:76-119`) copies the
   longer string in, ANDs the shorter into its front and the pad into its tail -- structurally what
   `bit_operation` does. The brief's "no default pad; there is a passthrough" is wrong as a
   mechanism and right as an outcome. Correction stands.
3. **`b2x(,'x')` is 40.4.** Measured, byte-exact. A max-1 row checks the maximum first, so 40.5 is
   unreachable there. Confirmed.
4. **`d2c('abc',-1)` is 93.929, `d2x('1.5',-1)` is 93.923.** Measured, byte-exact, both directions.
   The C++ ordering agrees: `x2dC2d` calls `optionalLengthArgument` as its *first* statement
   (`StringClassConversion.cpp:396`) where `RexxString::d2c` builds the `NumberString` first.
5. **93.977 is tested nowhere in `ootest/base`.** Re-checked with `/bin/grep -arn` and a positive
   control: 93.976 has five hits (`class/String/x2d`, `x2b` twice, `x2c`, `bif/X2D`), 93.977 zero.

## 6. `NUMERIC DIGITS` boundaries

Never above 1000. All-`FF` input, oracle and rust identical on every row:

| DIGITS | last accepted `FF` bytes | `floor(D / log10 256)` |
|---|---|---|
| 1 | **0** (`c2d('ff'x)` is 93.936) | 0 |
| 2 | 0 | 0 |
| 3 | 1 | 1 |
| 5 | 2 | 2 |
| 7 | 2 | 2 |
| 8 | 3 | 3 |
| 9 | 3 | 3 |
| 100 | 41 (`length` 99); 42 refuses | 41 |
| 1000 | 415 (`length` 1000); 416 refuses | 415 |

`DIGITS 1` admits zero bytes, as the brief says. Also crossed with the sign: at `DIGITS 5`
`c2d('8000'x,2)` is `-32768` and at 4 it is 93.936, so the sign is not counted as a digit.

## 7. Mutation testing -- theirs and mine

Each applied from a pristine copy, then: (a) `cargo test -p rexx-exec --lib builtin::convert`,
(b) `REXX_CORPUS_GATE=1 cargo test -p rexx-exec -- --skip builtin::convert`. The `20 passed /
344 filtered out` counts confirm the tests exist and ran, so "does not exist" cannot read as
"passed". Baseline for (b) with the mutation absent: **fully green** (the report's "1073 passed,
2 failed" was a mid-run state before the two corpus files were committed; it is green now).

| mutation | convert tests | rest of package |
|---|---|---|
| M1 `BITAND` default pad -> `'00'x` | RED: `an_omitted_pad_leaves_the_longer_strings_tail_alone` | green |
| M6 `cntrl` table stops at its NUL | RED: `every_character_class_is_the_oracles_own_table` | green |
| M7 `X2D` odd-length sign bit -> `0x80` | RED: `a_length_makes_the_read_a_signed_window` | green |
| R1 (mine) `has_significant_decimals`' out-of-bounds branch -> `false` | RED: `decimals_are_significant_relative_to_the_precision` | green |
| R3 (mine) trailing whitespace names the *first* of a run | RED: `whitespace_at_either_end_names_its_position` | green |
| R4 (mine) `render_decimal` drops the sign | RED: 2 tests | green |
| **R2 (mine)** delete `XRANGE`'s `count == 1` early return | **all 20 green** | green |
| **R5 (mine)** residue checked only at the end, never at a later gap | **all 20 green** | **green** |

M1, M6 and M7 are confirmed exactly as reported, and confirmed *invisible* to everything else in
the package with the corpus gate on. That makes them coverage, not decoration.

---

## Findings

### 1. Important -- the per-gap residue check is load-bearing and nothing watches it fail

`convert.rs:186-194`. Deleting the in-loop comparison and keeping only the end-of-string one
(mutation R5) leaves **all 20 new tests green and the whole `rexx-exec` package green under
`REXX_CORPUS_GATE=1`**, while the mutated build silently converts three strings the oracle refuses:

```
x2c('41 4 1 42')    oracle 93.976 rc 163      mutated build  414142   rc 0
x2c('414 2 434')    oracle 93.976 rc 163      mutated build  04142434 rc 0
b2x('1010 10 10')   oracle 93.977 rc 163      mutated build  AA       rc 0
```

The shape is a three-group string whose *middle* group breaks the residue and whose *total*
restores it. Every refusal the committed tests assert (`'414 243'`, `'4 4'`, `'44 4'`,
`'101 000'`, `'10 10'`, `'1 0 0000'`) also fails the end-of-string check, so none separates the two.
This is a silent wrong answer, of the family this phase exists to prevent, reachable with a
four-token program. **Add one assertion of that shape per notation.** (`'414 2 434'` *is* in the
9,326-program sweep -- which is exactly why an uncommitted sweep does not substitute for a test.)

### 2. Important -- report §1's discriminating case does not discriminate

"a probe like `b2x('101 0000')` = `50` (residue 3, then an exact 4) is the discriminating case
either way" is false. That string separates the residue rule from a *residue-0* rule (which M3
covers), not the per-gap check from an end-only one. The genuine discriminator is finding 1's
shape. Restating a measurement is authorship: this sentence should name a string that actually
splits the two readings.

### 3. Important -- "The one variadic row" is a mutable in-repo aggregate

`builtin/mod.rs`, the `XRANGE` row's comment. `MAX_Max = argcount` and `MIN_Max = argcount`
(`interpreter/expression/BuiltinFunctions.cpp:1996` and `:2025`) make `MAX` and `MIN` variadic too,
and both land in this phase's Task 8 (`numeric.rs`, plan line 841). The sentence is a claim about
what every *other* row in `IMPLEMENTED` is, it is falsified by a later task rather than by re-reading
the line, and it is the same class as the three Task 4 shipped and had corrected. Delete the
"one variadic row" clause; the `XRANGE_Max = argcount` citation and the measured
eight-argument probe beside it are the parts that earn their place.

### 4. Important -- report §9's untested divergence is stated in a direction the code contradicts

§9 says `NUMERIC DIGITS` above ~500 million "would make the oracle's working-buffer allocation fail
(Error 5) where this crate's accumulator grows lazily". It does not grow lazily: `x2d_c2d`
(`convert.rs:539`) requests `buffer(digits + OVERFLOW_SPACE + 1)` and `d2x_d2c` (`:798`) requests
`buffer(max(result_size, digits) + OVERFLOW_SPACE)`, both up front and both fallible -- which is
precisely `new_buffer(currentDigits + NumberString::OVERFLOWSPACE + 1)` at
`StringClassConversion.cpp:550`. Both sides ask for the same allocation, so the stated direction
does not arise, and §7's own "the working buffers are reproduced, not skipped" contradicts it.

**Judgment on the `KNOWN GAP` question: no new row is needed.** The residual asymmetry at that
scale is that `ulimit -v` charges this crate 512 MiB of `INTERPRETER_STACK_BYTES` reservation the
oracle does not pay, so this crate raises Error 5 where the oracle returns -- the *opposite*
direction, already owned in detail at `docs/superpowers/plans/phase-4-exclusions.txt:1347`
("A LARGE RESULT ABORTS WHERE THE ORACLE RETURNS, AND RAISES Error 5 EARLIER THAN IT", second
cause). Adding a row for a divergence that does not exist would be worse than the missing row.
**Correct the §9 sentence instead.**

### 5. Minor -- `XRANGE`'s single-class early return is behaviourally dead

`convert.rs:978-981`. Removing `if count == 1 { return Ok(interp.text(table)); }` leaves all 20
tests and the whole package green, because with `count == 1` the loop pushes the class, `continue`s,
and immediately exits on `position < count`, writing the same bytes. It is not wrong; it is a branch
no test can distinguish. Either drop it or note that it only avoids one copy.

### 6. Minor -- `a_scan_takes_apart_every_text_the_number_parser_accepts` can go vacuous

`convert.rs:1622`, `assert!(scanned || !parsed)`. Nothing asserts that *any* of the 35 subjects
parses, so a regression making `Interp::to_number` reject everything satisfies the whole loop. One
`assert!(subjects.iter().filter(parses).count() >= N)` closes it. (The one-directional intent is
right and the doc says so; it is the missing floor that is the defect.)

### 7. Minor -- the decimal accumulators are infallible allocations outside `builtin::buffer`

`shift_in`'s `accumulator.push` (`:421`), `render_decimal`'s `Vec::with_capacity` (`:439`) and
`scan_decimal`'s `significand` (`:619`) are all infallible. The reproduced oracle buffer is
`drop`ped rather than used as the storage, so what is reproduced is the *refusal*, not the working
space -- which the comment at `:396-399` does disclose. Bounded by `digits` (the in-loop
`digit_count > digits` check) and by the argument's own rendered length, so unreachable under the
project's own probe rule; recorded because the shapes differ from the oracle's, not because a
program can see it.

---

## Cannot verify from the diff

* **The sweep is not in the repository.** §3's axes and §9's "no `.rex` corpus file was added" are
  consistent, and I regenerated and re-ran the sweep from the session scratchpad (9,326 programs,
  0 mismatches, alphabets as claimed) -- but nothing in the tree reproduces it, and the shared
  block's "run the new corpus against the build that had the bug" is not satisfiable for an
  artefact that is not a corpus file. The mutation table is what carries the coverage claim here,
  and finding 1 is exactly where that substitution leaks.
* **§4's "1073 passed, 2 failed" baseline** is no longer reproducible (the two corpus files it was
  waiting on are committed). I measured the equivalent as fully green, which is the stronger form.
* **`NUMERIC DIGITS` above 1000** is unprobed by rule, so finding 4's direction is argued from the
  two allocation sites and the C++, not measured.
* **`XRANGE`'s twelve class tables** were compared byte-for-byte against the oracle at run time
  rather than against the C++ tables; the oracle is the authority the phase names, so this is
  a note, not a gap.
