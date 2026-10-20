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
