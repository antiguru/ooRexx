# Task 6.14 report: numeric follow-ups and the numeric model in Rexx

Status: DONE_WITH_CONCERNS. Step 1's two fixes are in with their witnesses, plus a third fix the
model found (LOSTDIGITS on a DO count). `corpus/model/numeric.rex` models this crate's numeric
semantics. `tests/numeric_model.rs` agrees with this crate on every case except the REPORTED
rows. On the oracle, the departures are exactly Deviations 44 and 47 plus those same REPORTED rows.
The REPORTED rows are places where the derived rule and an interpreter part with no Deviation
ruled. They go to Moritz (Concerns).

## Commits

* `009fd1978` Step 1: MAX/MIN round a wide whole number, and LOSTDIGITS under FUZZ. It holds the
  code, `lang/lostdigits_fuzz_compare.rex` with its `sourceline_oracle` file and `phase-9.txt`
  line, the licensed-divergence row `max-min-wide-integer-rounds`, its marker under Deviation 44,
  and the updated unit test.
* `7a6eb8778` Steps 2 and 3: `corpus/model/numeric.rex`, `tests/numeric_model.rs` and its case
  files `tests/numeric_model/{spike,task-6.3,task-6.14}`. It also holds the DO count fix and
  `lang/lostdigits_do_count.rex` with its `sourceline_oracle` file and `phase-9.txt` line.
* The records commit that follows holds `docs/superpowers/records/2026-10-18-phase-9d/t6.14/`,
  this report, and the two consumed queued items marked closed.

## Step 1

### MAX and MIN round a wide whole number (Deviation 44)

`builtin/numeric.rs`'s `integer_path` answered the winning argument object as it stood, so
`max(2, 1111111111)` at DIGITS 9 was `1111111111` while `2~max(1111111111)` was `1.11111111E+9`.
The new `rounded_integer` rounds the integer path's answer to DIGITS when the answer is wider,
and so does the lone-target `MIN` return. LOSTDIGITS is unchanged: it is still decided by the
written value (`integer_path_applies`). The unit test `min_answers_a_lone_target_before_max_would`
asserted the oracle's unrounded `min(12345)` at DIGITS 3. It became
`a_lone_wide_target_rounds_at_both_ends`.

Witness: the `LicensedDivergence` row `max-min-wide-integer-rounds` in
`tests/licensed_divergences.rs`, with `LICENSED DIVERGENCE WITNESS: max-min-wide-integer-rounds`
under Deviation 44. It covers the function form at both ends, the lone target, a negative pair,
rounding up, the trailing-zero `1000000000`, and ENGINEERING.
* Red: the base binary (`71af82f68`) prints the oracle's unrounded line, byte for byte
  (`t6.14/witnesses/red/max-min-wide-integer-rounds/`).
* Green: this crate prints the row's `crate_stdout`, and `licensed_divergences` passes.

### LOSTDIGITS for a comparison under FUZZ

The oracle's `NumberString::comp` (`classes/NumberStringClass.cpp:3194`) checks both operands
against DIGITS. When the comparison has to subtract, `addSub` checks them again at DIGITS minus
FUZZ. It subtracts unless the signs differ or both operands are zero. rexxref "Numeric Comparisons"
says the subtraction runs "as though NUMERIC DIGITS [DIGITS minus FUZZ] had been put in effect".
New `Interp::lostdigits_check_compare` and `lostdigits_check_fuzz` (`run/condition.rs`) do the
second check, behind the existing `lostdigits_armed` gate, with the raise `#[cold]`. Sites:
* `compare_values_body` (the operators), with the operands as written as descriptions;
* `max_min_over`, on the rounded candidate and then the answer so far, rendered unrounded, as
  `maxMin` compares its two rounded clones;
* `controlled_within_wide`, a controlled loop's TO test under FUZZ, variable first.

Witness `lang/lostdigits_fuzz_compare.rex`: the operators, signs, zeros, strings with blanks, the
strict and string comparisons, MAX and MIN in both forms, the loop bound with BY 1 and BY -1,
DIGITS 3, ENGINEERING, untrapped, and `::OPTIONS LOSTDIGITS SYNTAX` (98.972, rc 158).
* Red: at `71af82f68`, stdout differs on every raising line and the run ends at rc 0.
* Green: stdout, stderr and rc are identical to the oracle's. I read the stdout, and every path
  prints its line.

## Step 2: `corpus/model/numeric.rex`

`numeric_eval(trapped, digits, fuzz, form, kind, name, count, args)` answers one operation as
`V[value]`, `S code` or `L[description]`. It covers number syntax and operand checks, rounding to
DIGITS, LOSTDIGITS, `+ - * / % // **` and prefix `+ -`, the comparisons with FUZZ (strict and
string included), `ABS`, `SIGN`, `TRUNC`, `FORMAT`, `MAX` and `MIN` in both forms,
`DATATYPE(x,'W')`, and the DO repetitor and FOR count. Values are digit strings, and every digit
is computed one decimal digit at a time. The host's arithmetic touches only counters, lengths and
exponents, at NUMERIC DIGITS 20 inside the routine. Its output is identical on both interpreters
for every case (the oracle test asserts it).

The rules come from rexxref ("Numbers and Arithmetic", the numeric built-in functions, "DO").
Section numbers of ANSI X3.274 are cited where they settle a point (7.4, 8.3.6, 9.2.1, 9.4). No
text or code from the standard is in the file. Where the documentation leaves a point open, the
model takes this crate's answer and says so at the rule:
* the LOSTDIGITS description of a MAX/MIN argument or a DO count;
* the written-whole-number test that silences MAX/MIN's LOSTDIGITS (Deviation 44, "decided by the
  value");
* the error codes of the builtins' argument checks.

Derivations worth a reviewer's look:
* **Addition rounds counting from the aligned leftmost place, not from the first significant
  digit.** rexxref's own FUZZ example (`4.9999 = 5` is 1 at DIGITS 5, FUZZ 1) only comes out this
  way. So does `1111111111 - 1111111110` = 0, which agrees with both interpreters. The ANSI
  spike's integer path answered 1 there, and the spike's "integer path" finding is therefore not a
  departure here.
* **A DO count is tested as DATATYPE W tests, by rounding, so a wider count raises LOSTDIGITS
  first** (ANSI 8.3.6). The oracle does this for every count it does not hold as a `RexxInteger`,
  and this crate did not, so the model found it as a crate departure. It is fixed:
  `count_lostdigits` (`run/loops.rs`), witness `lang/lostdigits_do_count.rex`. That witness is red
  at `71af82f68` on all 20 raising lines and identical to the oracle at head.
* `TRUNC`, `FORMAT`, `SIGN`, `ABS` and a MAX/MIN target raise no LOSTDIGITS (Ruled 10).

## Step 3: `tests/numeric_model.rs`

Each case runs a generated program. The program computes the native operation and `numeric_eval`,
each once untrapped and once under `SIGNAL ON LOSTDIGITS`, beside a copy of the model. The cases
come from three files: the spike's table (`spike`, every row but `datatype(…, 'N')`, which the
model does not cover), Task 6.3's `maxdesc.rex`/`maxstr.rex` probes and its witnesses' operations
(`task-6.3`), and this task's own cases (`task-6.14`: rexxref's examples, errors, FUZZ, DO counts,
and the reported rows). Each recorded output is the model's answer, plus a `crate:` line where
this crate differs.

* `the_model_answers_every_case_as_this_crate_does`: this crate's native answer differs from the
  model exactly on the `REPORTED` rows marked `Both`.
* `on_the_oracle_the_model_parts_only_where_the_numeric_deviations_say`: the model answers
  identically on the oracle. The set of cases where the oracle's native answer differs equals
  `ORACLE_DEPARTURES` ∪ `REPORTED`, and the Deviations named are exactly {44, 47}.
* Mutation: rounding at `>= 6` instead of `>= 5` in the model's `round` turns the crate test red.

## The rows reported to Moritz (Concerns 1-4)

These are the `REPORTED` rows. In each, the model follows the derived rule. For R1, R2 (function
form), R3 (function form) and R4, the crate keeps the oracle's answer: changing it would be a new
oracle departure, and that needs a ruling first.

* **R1 FUZZ LOSTDIGITS for operands of different signs or zero** (`fuzz-sign-differs`,
  `fuzz-sign-zero`, `fuzz-18`). rexxref defines the comparison as the subtraction at DIGITS minus
  FUZZ, so `1.00000001 = -2` under FUZZ 1 should raise. Both interpreters skip the check when the
  signs differ (`NumberString::comp`'s sign shortcut). Derived: raise. Crate and oracle: silent.
* **R2 MAX/MIN's whole-number path ignores FUZZ** (`max-fuzz-integers`,
  `max-message-fuzz-integers`). At FUZZ 1, `max(100000001, 100000002)` is `100000002` on both
  interpreters, where `100000001 = 100000002` is 1. The oracle's `RexxInteger::Max` compares
  `wholenumber_t`s, so its message form `100000001~max(100000002)` answers `100000002` too. This
  crate's message form already applied FUZZ (`100000001`), which is a divergence from the oracle
  that predates this task and that no Deviation covers. The function form here copies the integer
  path. It is the same representation leak as Deviation 44. Ruling the derived answer would extend
  44 and change the function form.
* **R3 an omitted MAX/MIN argument on the whole-number path** (`max-missing-argument`,
  `max-message-missing-argument`). `max(1,,3)` is 93.903 "argument 0 is required" on both
  interpreters, while `max(1.5,,3)` is 40.5. `1~max(,2)` is 93.903 on the oracle and 40.5 here (a
  pre-existing divergence). Derived: the omission's error does not depend on the target's
  representation.
* **R4 FORMAT's `expt` trigger for the decimal part of a number with an integer part**
  (`format-decimal-trigger`). rexxref: "If the number of places needed for the integer or decimal
  part exceeds expt or twice expt, respectively, the exponential notation is used."
  `format(12.3456789,,,,3)` needs 7 decimal places, more than 6. Both interpreters answer
  `12.3456789`, because they apply the decimal trigger only below one. The model answers
  `1.23456789E+1`. I am less sure of this derivation than of R1-R3, because with the default
  `expt` the two readings never differ.

## Checks

All from `rust/` with `CARGO_TARGET_DIR=<scratchpad>/t614-target` and `CARGO_INCREMENTAL=0`.
Outputs are in `docs/superpowers/records/2026-10-18-phase-9d/t6.14/`.

| command | exit |
| --- | --- |
| `timeout 590 memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx` | 0 |
| `cargo fmt --all --check` (after `cargo fmt --all` reflowed `numeric_model.rs`) | 0 |
| `timeout 590 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | 0 |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test licensed_divergences --test numeric_model` (`checks-tables.txt`) | 0 |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --lib` (`checks-lib.txt`) | 0 (1157 passed) |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` | 0 |
| `REXX_CORPUS_GATE=1 timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 0, 1148 of 1148 matching |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t6.14/suite/table.tsv ... --test suite l3_core_suite` | 101: 402 groups, 1 red, `SysFileTree TEST_TREE_DIRECTORY` |
| the same suite command again, table at `t6.14/suite-r2/table.tsv` | 0: 402 groups green |

## Suite delta

Both runs were against `t6.13/r4/suite/tests.tsv`, and the only row that changed is the known
flake `base/rexxutil/SysFileTree TEST_TREE_DIRECTORY`. The first run had crate passed / oracle
failed and was 1 red. The second had crate failed / oracle passed and was green, because the
group verdict compares the two sides. No other row changed in either run. The 9d set stays empty.

## Perf

`t6.14/callgrind.txt` holds pin, parent (`t6.13-r4/rexx-head`, the same code as the base
`71af82f68`) and head (`7a6eb8778`, `cg-9d/t6.14/rexx-head`) in one run, exit 0. Head minus
parent, with libc and ld-linux subtracted:

| program | parent -libc vs pin | head -libc vs pin | task delta |
| --- | --- | --- | --- |
| startup | -0.1201 | -0.1207 | -0.0006 |
| interpret | -4.6331 | -4.6331 | 0.0000 |
| parse | -0.1416 | -0.1416 | 0.0000 |
| churn | +0.1068 | +0.1068 | 0.0000 |
| sayloop | -0.0637 | -0.0640 | -0.0003 |
| arith | +0.0401 | +0.0401 | 0.0000 |
| decloop | -0.0080 | +0.0252 | +0.0332 |
| rexxcps | +0.5224 | +0.3424 | -0.1800 |

Every delta is under +0.05%. decloop's +0.033% is the largest. The raw delta, head minus parent,
is +0.0321 (decloop) and at most +0.0055 elsewhere. rexxcps drops by about 32M instructions. That
change is either layout or the loop path's `controlled_within_wide` becoming a method, and I did
not attribute it.

## Concerns

1. R1-R4 above. Each needs Moritz's ruling: either a Deviation, or a change to the crate (R1,
   R2 function form, R3 function form, R4) or to the model.
2. **Deviation 47's text and the oracle.** The oracle's run of `do 1111111111` is the
   `RexxInteger` representation: `do '1111111111'` raises 26.2 on the oracle too
   (`do-count-wide-string` agrees on both). With a LOSTDIGITS trap set, this crate now raises
   LOSTDIGITS before 26.2 for a wide count (`do-count-wide`'s trapped answer). The Deviation's
   text names only 26.2/26.3.
3. **MAX/MIN's quiet LOSTDIGITS** is decided by how the value is written: an optional minus, then
   at most 18 digits with no leading zero. No documentation backs this. It is how this crate
   implements Deviation 44's "decided by the value", and the model encodes it the same way.
4. The two consumed queued items are marked CLOSED. `queued-inventory.tsv` was not edited.

## Fix round 1

Review: `task-6.14-review.md`. Moritz ruled 2026-10-20 ("follow rexxref") on R1-R4, R6 and R7. R5
keeps the interpreters' answer. Status: **BLOCKED on the whole suite.** The suite is red because
R4 and R7 contradict the ooTest suite. The crate and the model follow the rulings everywhere else,
and every other check is green.

### Commits

* `9b07a4e33`: the code, the model, the test, the witness edits and their `sourceline_oracle`
  files.
* The records commit that follows holds `t6.14/r1/` and this section.

### What changed

Code is in `builtin/numeric.rs`, `dispatch/string.rs`, `run/condition.rs` and
`rexx-num/src/format.rs`.

* R1: `lostdigits_fuzz_raise` no longer skips operands of different signs or zeros.
* R2 and R3: MAX/MIN's whole-number path (`integer_path`, `integer_path_applies`,
  `integer_object`) is deleted, so both forms go through `max_min_over`. That path applies FUZZ,
  which also closes review Important 1. An omitted argument is now 40.5 in the function form,
  numbered by its position in the call, and 93.903 in the message form.
* R4: `initial_exponent` and `post_carry_exponent_error` apply the decimal-part trigger whether or
  not the number has an integer part.
* R6: no change. This crate already checked the divisor; only the oracle skips it.
* R7: `target_number` (ABS, SIGN, TRUNC, FORMAT) checks LOSTDIGITS on the number argument and names
  it as written. The message forms check their receiver the same way. `max_min_over` checks the
  target and every argument and names each by its number unrounded.
* Description rows: the crate's answer is kept. It does not depend on whether the operand is a
  literal or a string. The DO count keeps its rendering (the number unrounded, rendered at DIGITS,
  as MAX names an argument). I tried "as written": it matches the oracle on the literal row
  (`describe-do-literal`) and departs on two string rows: `'1111111111.5'` where the oracle names
  `1.1111111115E+9`, and `' 3.0000000000 '` where the oracle names `3.0000000000`. The rendered
  form departs on the literal row only, so I kept it.
* Model: R5 is marked at `add` as "the model takes the interpreters' answer, rexxref silent".
  `pow` now takes the power's bits by halving the digit string and multiplies digit strings for
  its overflow test (Minor 6). The MAX/MIN, builtin and DO count rules follow the above.
* Test (Minor 7): `REPORTED` is gone. `ORACLE_DEPARTURES` holds (case, Deviation, rule, the
  oracle's answer). The oracle test asserts:
  * each listed case departs, with exactly the recorded answer;
  * the rule maps to the Deviation (R1-R4, R6 and `description` to 53; R7 and `wide-integer` to
    44; `do-count` to 47);
  * the Deviation can license that kind of case (47 only `do`, 44 only `bif`/`msg`);
  * every unlisted case agrees.

  Relabelling `do-for-wide` as 44/R7 turns it red: "Deviation 44 cannot license a `do` case". The
  crate test asserts that this crate equals the model on every case.
* Minor 5: the licensed row's negative pair is now `min(-5, '-1111111111')`. It prints
  `-1111111111` at the base `71af82f68` and `-1.11111111E+9` at head and on the oracle.
* Witnesses the rulings overturn lost those lines. The lines moved to the model's table, where each
  departure is pinned:
  * `lang/lostdigits_max_min.rex`: a wide MAX/MIN target, the quiet whole-number lines, and the
    ABS/FORMAT/TRUNC/SIGN lines.
  * `lang/string_extremes_refusals.rex`: the omitted-argument lines.
  * `lang/lostdigits_fuzz_compare.rex`: the sign and zero lines, the wide target, and the
    ENGINEERING line.

  After the edit each program is identical to the oracle on stdout, stderr and rc
  (`t6.14/r1/witnesses/`), and its `sourceline_oracle` file is regenerated.

### Each ruled rule, measured

The full table is `t6.14/r1/oracle-departures.tsv`. Each line below gives the case, then this
crate's answer (which equals the model's), then the oracle's. Answers are untrapped `|` under
SIGNAL ON LOSTDIGITS.

* **R1** (Deviation 53, rule `R1`). rexxref numarit.xml "Numeric Comparisons": "Numeric values are
  compared by subtracting the two numbers ... the numbers are subtracted with a precision of DIGITS
  minus FUZZ digits during the comparison"; condtra.xml LOSTDIGITS: "raised if a number used in an
  arithmetic operation has more digits than the current setting of NUMERIC DIGITS". FUZZ 1:
  * `1.00000001 = '-2'`: `V[0] | L[1.00000001]`; oracle `V[0] | V[0]`
  * `1.00000001 > 0`: `V[1] | L[1.00000001]`; oracle `V[1] | V[1]`
  * `'-1.00000001' < 2`: `V[1] | L[-1.00000001]`; oracle `V[1] | V[1]`
  * `0 < 1.00000001`: `V[1] | L[1.00000001]`; oracle `V[1] | V[1]`
  * `max('-1.00000001', 1)`: `V[1] | L[-1.00000001]`; oracle `V[1] | V[1]`
* **R2** (53, `R2`). Same passages. FUZZ 1:
  * `max(100000001, 100000002)`: `V[100000001] | L[100000002]`; oracle `V[100000002] | V[100000002]`
  * `100000001~max(100000002)`: the same pair of answers
  * The string forms (`max('100000001', '100000002')`, `min('100000002', '100000001')`,
    `max('11', '12')` at 8/7, the message form) now agree with the oracle. That is Important 1.
* **R3** (53, `R3`). rexxref errnums.xml: 40.5 "Missing argument in invocation of routine;
  argument argument_number is required" and 93.903 "Missing argument in method; argument argument
  is required". A function is a routine and a message is a method.
  * `max(1,,3)` and `min(1,,3)`: `S 40.5`; oracle `S 93.903` (its whole-number path, "argument 0")
  * `'55'~max(,77)`, `'55'~max(77,,88)` and `'55'~min(,77)`: `S 93.903`; oracle `S 40.5`
  * Agreeing: `max(1.5,,3)` is `S 40.5` on both. `1~max(,2)` is `S 93.903` on both, because the
    oracle's whole-number path happens to give the method's code
    (`max-message-missing-argument`).
* **R4** (53, `R4`). rexxref funct.xml FORMAT: "If the number of places needed for the integer or
  decimal part exceeds expt or twice expt, respectively, the exponential notation is used."
  * `format(12.3456789,,,,3)`: `V[1.23456789E+1]`; oracle `V[12.3456789]`
* **R6** (53, `R6`). condtra.xml LOSTDIGITS, as for R1.
  * `0 / 1234567890.5`, `'0' / 1234567890.5`, `0 % 1234567890.5`, `0 // 1234567890.5`:
    `V[0] | L[1234567890.5]`; oracle `V[0] | V[0]`
* **R7** (44 as rewritten, `R7`). rexxref funct.xml, the built-in functions' preamble: "Any argument
  named as a number is rounded, if necessary, according to the current setting of NUMERIC DIGITS
  (as though the number had been added to 0) and checked for validity before use. This occurs in
  the following functions: ABS, FORMAT, MAX, MIN, SIGN, and TRUNC". There are 34 rows; three
  examples:
  * `abs('-1111111111')`: `V[1.11111111E+9] | L[-1111111111]`; oracle `V[1.11111111E+9] |
    V[1.11111111E+9]`
  * `max(1111111111, 2)`: `V[1.11111111E+9] | L[1111111111]`; oracle `V[1.11111111E+9] |
    V[1.11111111E+9]`
  * `1111111111~min(2222222222)`: `... | L[1111111111]`; oracle `L[2222222222]`
* **`wide-integer`** (44): the rows already under Deviation 44's rounding, for example `max(2,
  1111111111)` is `V[1.11111111E+9] | L[1111111111]`; oracle `V[1111111111] | V[1111111111]`.
* **`description`** (labelled 53; the controller may prefer 44, since it is the oracle's literal
  representation):
  * `- 1234567890.5`, `1234567890.5 * 1`, `1234567890.5 ** 1`, `1234567890.5 % 1`: `...
    L[1234567890.5]`; oracle `L[1.2345678905E+9]`
  * `max(1, 12345.5)` at DIGITS 3: `L[1.23455E+4]`; oracle `L[12345.5]`
  * `do 12345.5` at DIGITS 3: `S 26.2 | L[1.23455E+4]`; oracle `S 26.2 | L[12345.5]`
* **`do-count`** (47): unchanged.

### The suite (BLOCKED)

`REXX_SUITE_GATE=1 ... l3_core_suite` with the table at `t6.14/r1/suite/table.tsv`: exit 101,
402 groups, 3 red. Against `t6.14/suite-r2/tests.tsv` the changed rows are 63 in
`base/bif/FORMAT`, 491 in `base/class/String/format`, `base/keyword/LOSTDIGITS TESTBIFS`, and the
SysFileTree flake. I attributed them by running the three groups once with R4 reverted in the
working tree (restored afterwards, `git diff` empty):
* **R4: `base/bif/FORMAT` (63 tests) and 57 of the `String/format` tests.** For example `test259`
  asserts `'9.9999999900' = format(9.99999999,1,10,3,3)`. The number has 8 decimal places, more
  than twice `expt` 3, so R4 makes it exponential. ooTest encodes the interpreters' reading.
* **R7: the other 434 `String/format` tests and `LOSTDIGITS TESTBIFS`.** OOREXXUNIT.CLS:1568 runs
  every test under `signal on any`, which takes LOSTDIGITS. So a receiver such as
  `12345.67890123~format(5,5,2,1)`, with 13 digits at DIGITS 9, now fails the test:
  `signal on any` takes `LOSTDIGITS 12345.67890123` here, and the oracle prints
  `    1.23457E+04`. `TESTBIFS` says the opposite of the ruling in so many words: "the arithmetic
  builtin functions do not honor lostdigits, since they are defined as performing a round
  operation on the arguments before processing", and it asserts no LOSTDIGITS for `trunc`,
  `max`, `min` and `format` of `11111.11111`.

Making the suite green under these rulings needs `suite-licences.txt` rows under Deviations 53 and
44. Writing those rows is the controller's decision, not mine. The alternative is for Moritz to
reconsider R4 and R7 in light of ooTest. I did not revert either one.

### Checks

| command | exit |
| --- | --- |
| `cargo build --release -j 4 -p rexx-exec --bin rexx` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy -j 4 -p rexx-exec -p rexx-num --all-targets -- -D warnings` | 0 |
| refusal tables + `licensed_divergences` + `numeric_model` (`r1/checks-tables.txt`) | 0 |
| `cargo test --release -p rexx-exec --lib` | 0 (1157 passed) |
| `cargo test --release -p rexx-num` | 0 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `REXX_CORPUS_GATE=1 ... --test corpus` (first run, `r1/checks-corpus.txt`) | 101, 1147 of 1148 |
| the same again (`r1/checks-corpus-2.txt`) | 0, 1148 of 1148 |
| suite (`r1/suite/`) | 101, 3 red (above) |

The first corpus run's one mismatch was `lang/command_redirected_target_waits_elsewhere.rex`
(Task 6.13's), `guard 0 3 waited 0` against the oracle's `waited 1`. Six runs alone on this binary
all print `waited 1`, and the second gate run matches, so I read it as a scheduling flake under
load.

### Perf

`t6.14/r1/callgrind.txt`, with parent `t6.14/rexx-head` (`7a6eb8778`) and head `9b07a4e33`, exit
0. The delta with libc subtracted is +0.0004% (startup), 0.0000% (interpret, parse, churn, arith,
decloop), +0.0002% (sayloop) and **+0.0565% on rexxcps** (17852957243 to 17863037838). That is
over the +0.05% per-task line, and I did not attribute it. Against the task's starting parent
(`t6.13-r4`), rexxcps is still -0.1229% (17884977296 to 17863037838).

### Concerns (round 1)

1. The suite block above, on R4 and R7.
2. rexxcps +0.0565% for the round, unattributed.
3. The `description` rows' Deviation label (53 or 44) is for the controller to choose.

## Fix round 2

Moritz ruled 2026-10-20: revert R4 and R7. ooTest is the ooRexx authors' own reading of rexxref.
R1, R2, R3 and R6 stay (Deviation 53). Status: DONE. The whole suite is green.

### Commits

* `bc176f5e8`: the revert in the crate and the model, the restored witness lines and their
  `sourceline_oracle` files, and the departure table.
* The records commit that follows holds `t6.14/r2/` and this section.

### What changed

* R4: `rexx-num/src/format.rs` again applies FORMAT's decimal-part trigger only below one.
* R7: `target_number` (ABS, SIGN, TRUNC, FORMAT) and the four message forms no longer check
  LOSTDIGITS.
* `max_min_over` again leaves the target quiet. Arguments, and each comparison under FUZZ, raise
  LOSTDIGITS unless every value is written as a whole number and the target fits DIGITS
  (`written_whole`, Deviation 44). It keeps R2 (FUZZ on whole numbers) and R3 (40.5 for the
  function, 93.903 for the message).
* Model: the builtin preamble and FORMAT say "the model takes the interpreters' answer: ooTest's
  reading of rexxref (`base/keyword/LOSTDIGITS TESTBIFS`, `base/bif/FORMAT`)". MAX/MIN have their
  quiet rule back.
* Witnesses: `lang/lostdigits_max_min.rex` is back to its `7a6eb8778` text and
  `sourceline_oracle` file. `lang/lostdigits_fuzz_compare.rex` has its wide-target and ENGINEERING
  lines back, and only the R1 sign lines stay out (its `sourceline_oracle` file is regenerated).
  `lang/string_extremes_refusals.rex` keeps round 1's edit, because R3 stays. All four numeric
  witnesses are identical to the oracle on stdout, stderr and rc.
* Test: the R4 and R7 rows are gone from `ORACLE_DEPARTURES`. The rows where MAX/MIN stay quiet on
  a string written as a whole number and the oracle raises are labelled `written-value` (44).
  These are `maxstr-string`, the `*-fuzz-strings*` rows and `min-negative-wide-string`, and their
  answers equal the oracle's apart from that LOSTDIGITS.

### Final departures table

These are all the cases where the oracle's native answer differs from the model. On every case,
this crate's answer equals the model's. Answers are untrapped `;` under SIGNAL ON LOSTDIGITS. The
TSV is `t6.14/r2/oracle-departures.tsv`. The rexxref citation per rule:

* **R1** (53): numarit.xml "Numeric Comparisons": "Numeric values are compared by subtracting
  the two numbers ... the numbers are subtracted with a precision of DIGITS minus FUZZ digits
  during the comparison ... just as though NUMERIC DIGITS 8 had been put in effect"; condtra.xml
  LOSTDIGITS: "raised if a number used in an arithmetic operation has more digits than the current
  setting of NUMERIC DIGITS".
* **R2** (53): the same numarit.xml passages. MAX "Returns the largest number of the list" uses
  the numeric comparison, FUZZ included.
* **R3** (53): errnums.xml 40.5, "Missing argument in invocation of routine; argument
  argument_number is required", for the function. 93.903, "Missing argument in method; argument
  argument is required", for the message.
* **R6** (53): condtra.xml LOSTDIGITS, as for R1. The division's divisor is an operand whatever
  the dividend.
* **description** (53, the controller's label): rexxref condtra.xml gives "The number with
  excessive digits that caused the LOSTDIGITS condition" and fixes no rendering.
* **wide-integer** and **written-value** (44): as Deviation 44 states.
* **do-count** (47): as Deviation 47 states.

**R1** (Deviation 53, 5 rows)

| case | operation | settings | crate (= model) | oracle |
| --- | --- | --- | --- | --- |
| `fuzz-18` | `bif MAX / '-1.00000001' / 1` | fuzz=1 | `V[1] ; L[-1.00000001]` | `V[1] ; V[1]` |
| `fuzz-sign-differs` | `infix = / 1.00000001 / '-2'` | fuzz=1 | `V[0] ; L[1.00000001]` | `V[0] ; V[0]` |
| `fuzz-sign-zero` | `infix > / 1.00000001 / 0` | fuzz=1 | `V[1] ; L[1.00000001]` | `V[1] ; V[1]` |
| `fuzz-sign-negative` | `infix < / '-1.00000001' / 2` | fuzz=1 | `V[1] ; L[-1.00000001]` | `V[1] ; V[1]` |
| `fuzz-zero-left` | `infix < / 0 / 1.00000001` | fuzz=1 | `V[1] ; L[1.00000001]` | `V[1] ; V[1]` |

**R2** (Deviation 53, 2 rows)

| case | operation | settings | crate (= model) | oracle |
| --- | --- | --- | --- | --- |
| `max-fuzz-integers` | `bif MAX / 100000001 / 100000002` | fuzz=1 | `V[100000001] ; V[100000001]` | `V[100000002] ; V[100000002]` |
| `max-message-fuzz-integers` | `msg MAX / 100000001 / 100000002` | fuzz=1 | `V[100000001] ; V[100000001]` | `V[100000002] ; V[100000002]` |

**R3** (Deviation 53, 5 rows)

| case | operation | settings | crate (= model) | oracle |
| --- | --- | --- | --- | --- |
| `max-missing-argument` | `bif MAX / 1 / / 3` | digits=9 | `S 40.5 ; S 40.5` | `S 93.903 ; S 93.903` |
| `min-missing-argument` | `bif MIN / 1 / / 3` | digits=9 | `S 40.5 ; S 40.5` | `S 93.903 ; S 93.903` |
| `max-message-missing-first` | `msg MAX / '55' / / 77` | digits=9 | `S 93.903 ; S 93.903` | `S 40.5 ; S 40.5` |
| `max-message-missing-middle` | `msg MAX / '55' / 77 / / 88` | digits=9 | `S 93.903 ; S 93.903` | `S 40.5 ; S 40.5` |
| `min-message-missing-first` | `msg MIN / '55' / / 77` | digits=9 | `S 93.903 ; S 93.903` | `S 40.5 ; S 40.5` |

**R6** (Deviation 53, 4 rows)

| case | operation | settings | crate (= model) | oracle |
| --- | --- | --- | --- | --- |
| `div-zero-dividend` | `infix / / 0 / 1234567890.5` | digits=9 | `V[0] ; L[1234567890.5]` | `V[0] ; V[0]` |
| `div-zero-dividend-string` | `infix / / '0' / 1234567890.5` | digits=9 | `V[0] ; L[1234567890.5]` | `V[0] ; V[0]` |
| `intdiv-zero-dividend` | `infix % / 0 / 1234567890.5` | digits=9 | `V[0] ; L[1234567890.5]` | `V[0] ; V[0]` |
| `rem-zero-dividend` | `infix // / 0 / 1234567890.5` | digits=9 | `V[0] ; L[1234567890.5]` | `V[0] ; V[0]` |

**description** (Deviation 53, 7 rows)

| case | operation | settings | crate (= model) | oracle |
| --- | --- | --- | --- | --- |
| `describe-prefix-literal` | `prefix - / 1234567890.5` | digits=9 | `V[-1.23456789E+9] ; L[1234567890.5]` | `V[-1.23456789E+9] ; L[1.2345678905E+9]` |
| `describe-mul-literal` | `infix * / 1234567890.5 / 1` | digits=9 | `V[1.23456789E+9] ; L[1234567890.5]` | `V[1.23456789E+9] ; L[1.2345678905E+9]` |
| `describe-pow-literal` | `infix ** / 1234567890.5 / 1` | digits=9 | `V[1.23456789E+9] ; L[1234567890.5]` | `V[1.23456789E+9] ; L[1.2345678905E+9]` |
| `describe-intdiv-literal` | `infix % / 1234567890.5 / 1` | digits=9 | `S 26.11 ; L[1234567890.5]` | `S 26.11 ; L[1.2345678905E+9]` |
| `describe-max-literal` | `bif MAX / 1 / 12345.5` | digits=3 | `V[1.23E+4] ; L[1.23455E+4]` | `V[1.23E+4] ; L[12345.5]` |
| `describe-do-literal` | `do count / 12345.5` | digits=3 | `S 26.2 ; L[1.23455E+4]` | `S 26.2 ; L[12345.5]` |
| `describe-max-trailing` | `bif MAX / 15 / 16.0` | digits=1 | `V[2E+1] ; L[1.60E+1]` | `V[2E+1] ; L[16.0]` |

**wide-integer** (Deviation 44, 10 rows)

| case | operation | settings | crate (= model) | oracle |
| --- | --- | --- | --- | --- |
| `max-wide-argument` | `bif MAX / 2 / 1111111111` | digits=9 | `V[1.11111111E+9] ; V[1.11111111E+9]` | `V[1111111111] ; V[1111111111]` |
| `max-message-wide-argument` | `msg MAX / 2 / 1111111111` | digits=9 | `V[1.11111111E+9] ; V[1.11111111E+9]` | `V[1111111111] ; V[1111111111]` |
| `max-trailing-zeros` | `bif MAX / 2 / 1000000000` | digits=9 | `V[1.00000000E+9] ; V[1.00000000E+9]` | `V[1000000000] ; V[1000000000]` |
| `maxstr-literal` | `bif MAX / 1 / 1111111111` | digits=9 | `V[1.11111111E+9] ; V[1.11111111E+9]` | `V[1111111111] ; V[1111111111]` |
| `maxmin-11` | `msg MAX / 1 / 1111111111` | digits=9 | `V[1.11111111E+9] ; V[1.11111111E+9]` | `V[1111111111] ; V[1111111111]` |
| `min-lone-wide` | `bif MIN / 12345` | digits=3 | `V[1.23E+4] ; V[1.23E+4]` | `V[12345] ; V[12345]` |
| `msg-min-lone-wide` | `msg MIN / 12345` | digits=3 | `V[1.23E+4] ; V[1.23E+4]` | `V[12345] ; V[12345]` |
| `max-wide-rounds-up` | `bif MAX / 1 / 2 / 1111111115` | digits=9 | `V[1.11111112E+9] ; V[1.11111112E+9]` | `V[1111111115] ; V[1111111115]` |
| `max-engineering-wide` | `bif MAX / 2 / 11111111111` | form=engineering | `V[11.1111111E+9] ; V[11.1111111E+9]` | `V[11111111111] ; V[11111111111]` |
| `fuzz-engineering-integer` | `bif MAX / 123 / 1240` | digits=3 fuzz=1 form=engineering | `V[1.24E+3] ; V[1.24E+3]` | `V[1240] ; V[1240]` |

**written-value** (Deviation 44, 6 rows)

| case | operation | settings | crate (= model) | oracle |
| --- | --- | --- | --- | --- |
| `maxstr-string` | `bif MAX / 1 / '1111111111'` | digits=9 | `V[1.11111111E+9] ; V[1.11111111E+9]` | `V[1.11111111E+9] ; L[1111111111]` |
| `max-fuzz-strings` | `bif MAX / '100000001' / '100000002'` | fuzz=1 | `V[100000001] ; V[100000001]` | `V[100000001] ; L[100000002]` |
| `min-fuzz-strings` | `bif MIN / '100000002' / '100000001'` | fuzz=1 | `V[100000002] ; V[100000002]` | `V[100000002] ; L[100000001]` |
| `max-fuzz-strings-short` | `bif MAX / '11' / '12'` | digits=8 fuzz=7 | `V[11] ; V[11]` | `V[11] ; L[12]` |
| `max-message-fuzz-strings` | `msg MAX / '100000001' / '100000002'` | fuzz=1 | `V[100000001] ; V[100000001]` | `V[100000001] ; L[100000002]` |
| `min-negative-wide-string` | `bif MIN / '-5' / '-1111111111'` | digits=9 | `V[-1.11111111E+9] ; V[-1.11111111E+9]` | `V[-1.11111111E+9] ; L[-1111111111]` |

**do-count** (Deviation 47, 3 rows)

| case | operation | settings | crate (= model) | oracle |
| --- | --- | --- | --- | --- |
| `do-count-wide` | `do count / 1111111111` | digits=9 | `S 26.2 ; L[1111111111]` | `V[ran] ; V[ran]` |
| `do-for-wide` | `do for / 1111111111` | digits=9 | `S 26.3 ; L[1111111111]` | `V[ran] ; V[ran]` |
| `do-count-trailing-zeros` | `do count / 1000000000` | digits=9 | `S 26.2 ; L[1000000000]` | `V[ran] ; V[ran]` |

### Checks

| command | exit |
| --- | --- |
| `cargo build --release -j 4 -p rexx-exec --bin rexx` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy -j 4 -p rexx-exec -p rexx-num --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib` | 0 (1157 passed) |
| `cargo test --release -p rexx-num` | 0 |
| refusal tables + `licensed_divergences` + `numeric_model` (`r2/checks-tables.txt`) | 0 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `REXX_CORPUS_GATE=1 ... --test corpus` (`r2/checks-corpus.txt`) | 0, 1148 of 1148 |
| `REXX_SUITE_GATE=1 ... l3_core_suite`, table at `t6.14/r2/suite/table.tsv` | **0, 402 groups green** |

Against `t6.14/suite-r2/tests.tsv`, the one changed row is the SysFileTree flake, which moved back
to passed.

### Perf

`t6.14/r2/callgrind.txt`, with parent `t6.14/rexx-head` (`7a6eb8778`, the last measured head
before round 1) and head `bc176f5e8`, exit 0. The delta with libc subtracted is at most +0.0004%
on every program, and rexxcps is +0.0000% (61 instructions). So round 1's rexxcps +0.0565% went
with R7, whose new LOSTDIGITS checks were the only change to the paths rexxcps runs. Nothing is
over +0.05%, so there was nothing to trace.

## Fix round 3

Re-review: `task-6.14-rereview.md`. Status: DONE_WITH_CONCERNS. One new test,
`every_licensing_deviation_names_this_test`, stays red until the controller adds two sentences
to `phase-4-exclusions.txt`.

* `numeric_model.rs`'s `description` bullet now matches Deviation 53's corrected text. This crate
  names an operator's operand as written and a MAX, MIN or DO count argument rendered at the
  digits in force, whatever it came from. The oracle's rendering depends on the operator, and for
  MAX, MIN and a DO count on whether the clause was translated ahead or ran under INTERPRET.
* Minor 2: `target_number` and `target_parse` are one function again, `target_number`. Its doc
  says that the builtins round argument 1 without LOSTDIGITS, as ooTest's `TESTBIFS` reads
  rexxref.
* Minor 3: the new test reads `docs/superpowers/plans/phase-4-exclusions.txt` and splits it into
  numbered rows at each ` NN. ` heading. It asserts that every Deviation number in
  `ORACLE_DEPARTURES` has a row naming `numeric_model`. Today it fails with `[44, 47]`, and row 53
  passes. The sentences that would close it:
  * at the end of Deviation 44:
    `     Witnessed by tests/numeric_model.rs (ORACLE_DEPARTURES, rules wide-integer and written-value).`
  * at the end of Deviation 47:
    `     Witnessed by tests/numeric_model.rs (ORACLE_DEPARTURES, rule do-count).`

### Checks

| command | exit |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --test numeric_model --test licensed_divergences` | 101: only `every_licensing_deviation_names_this_test` fails, on `[44, 47]` |
| `cargo test --release -p rexx-exec --lib` | 0 (1157 passed) |
