# Task 6.14 review: numeric follow-ups and the numeric model

Reviewed `71af82f68..525ca9989` (`009fd1978`, `7a6eb8778`, `525ca9989`). I built HEAD into the
scratchpad (`rev614-target`, `cargo build --release --bin rexx`, exit 0). I used
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.13-r4/rexx-head` as the parent binary. The only commit
between it (`86f2a43b2`) and BASE is `71af82f68`, and that commit touches nothing under
`rust/crates`. Each probe ran from its own fresh `mktemp -d`, the crate under `memcap 2G timeout 20`
and the oracle under the standard wrapper. I compared stdout, stderr and rc separately. To run more
cases, I wrote a harness that builds the same program as `numeric_model.rs`'s `program()` (native
operation and `numeric_eval`, untrapped and trapped) and runs it on HEAD, on the parent and on the
oracle. In its output, `C`, `O` and `M` flag crate != model, oracle != model, and model-on-crate !=
model-on-oracle. The scratchpad is not durable, so the lines that decide each finding are quoted
below.

## Spec compliance

Step 1 and Step 3 comply. Step 2 complies in the main: the model works on digit strings, contains no
ANSI text, and cites ANSI only by section number. Two points fall short of "derived from the
documentation" (Ruled 13), though. The addition routine is this crate's `add_signed` restated in
Rexx (Important 3). And the oracle-side claim, "the departures are exactly Deviations 44 and 47
plus REPORTED", is true only for the case table. Beyond the table, the model finds oracle
departures, and one crate departure, that nobody reported (Important 1 and 2).

## Strengths

* **The model does not lean on the interpreter's arithmetic.** Under mutant M1 (the crate's
  rounding at `>= 6`), the model still answered `V[69.758]` while the native answer was
  `V[69.757]`. Under M2 (FUZZ ignored), the model still gave `V[1]` for rexxref's `4.9999 = 5`.
  The oracle test also asserts that the model's output is identical on both interpreters. I read
  the whole file. Host arithmetic touches single digits, column sums, lengths, exponents and the
  power's bits (Minor 6). Nothing adds two modelled operands.
* **No ANSI transcription.** I shingle-compared the model against `pdftotext` of j18pub.pdf (11786
  lines). Every 10-token overlap is a generic idiom such as `strip(x, 'L', '0')`, or the FORMAT
  rule "expp+2 blanks". That rule is rexxref's own text: "If the exponent would be 0 when a nonzero
  expp is specified, then expp+2 blanks are supplied". ANSI's `AddSubComp` is structured
  differently from the model's `add`.
* **`numeric_model.rs` can fail, on each check I mutated.** Every mutant below turned the named
  test red, `--no-fail-fast`, in a scratch worktree, and each was reverted with `git diff` empty:

  | mutant | crate test | oracle test | first failing case |
  | --- | --- | --- | --- |
  | M1 `round_change` `>= 5` to `>= 6` | red | red | `rexxref-operator-15` native 69.757, model 69.758 |
  | M2 `working_digits = digits` (FUZZ ignored) | red | red | `rexxref-fuzz-1-eq` |
  | M3 `lostdigits_raise_at` `<= digits + 1` | red | red | `maxdesc-1` |
  | M4 FUZZ LOSTDIGITS never raised | red | red | `rexxref-fuzz-1-eq` |
  | M5b `count_lostdigits` returns `Ok(())` | red | red | `do-count-rounds-whole` |
  | M6 `rounded_integer` never rounds | red | red | `maxstr-string` |
  | M7 sign shortcut removed (crate follows R1) | red | red | `fuzz-18` "was listed as departing" |
  | M8 FUZZ check at DIGITS, not DIGITS-FUZZ | red | red | `rexxref-fuzz-1-eq` |
  | M9 `do-for-wide` dropped from `ORACLE_DEPARTURES` | green | red | set mismatch |
  | M10 new case `0 / 1234567890.5` added | green | red | `review-zero-dividend` in the left set only |

  M9 and M10 show that the oracle test notices a new oracle departure, and also a departure that
  stops being listed. M5 itself did not compile, so I replaced it with M5b.
* **The three fixes match the oracle on the edges I tried.** Probed at HEAD against the oracle, the
  results were identical:
  * FUZZ: negatives, `' 1.00000001 '` (the description keeps the blanks), strict and string
    operators, FUZZ = DIGITS-1 (`9 8: 12 = 13 -> L 12`), DIGITS 1 and 2, `SIGNAL ON ANY`,
    `TRACE R`, `INTERPRET`, Array items.
  * MAX/MIN in both forms.
  * Controlled loops: `do i = 10 to 12` under FUZZ 8 gives `L[10] 10`, both by INTERPRET and
    compiled. Also `by -1`, `by 0.5`, `while`, `until`, nested loops, and the wide path
    (`do i = 99999999 to 100000000 -> L4 100000000 99999999`).
  * DO counts at DIGITS 3 and 1: `do n`, `do i = 1 for n`, `to ... for`, nested, `'0001234'`,
    `'1.234E3'`, `'-1234'`, `999.5`, `'9995E-1'`, `'0.0001'`, `'-0.0'`, `.array~new`, and a
    computed `999 + 1` (26.2, rc 230 on both). The parent gives `S 26.2` where both now give
    `L[...]`.

  `cargo test --release -p rexx-exec --lib` exits 0 with 1157 passed. `--test numeric_model
  --test licensed_divergences` exits 0 with 22 passed. The gated corpus result is at the end.
* **R1 to R4 reproduce**, on both interpreters and the model:
  * R1: `9 1: 1.00000001 = '-2'`, `1.00000001 > 0`, `'-1.00000001' < 1`. The model gives
    `L[...]`, both interpreters `V`.
  * R2: `max(100000001, 100000002)` at FUZZ 1. The model gives 100000001, both interpreters
    100000002. The message form gives 100000001 on the crate and 100000002 on the oracle.
  * R3: `max(1,,3)` and `min(1,,3)` are `S 93.903` on both. `max(1.5,,3)` is `S 40.5`.
    `1~max(,2)` is 40.5 here and 93.903 on the oracle.
  * R4: `format(12.3456789,,,,3)` is `12.3456789` on both, and the model gives `1.23456789E+1`.
    `format(12.3456789,,2,,3)` differs the same way: the model gives `1.23E+1`, both give `12.35`.
  * The passages these rest on:
    * R1 and R2: rexxref numarit.xml, "Numeric values are compared by subtracting the two numbers
      (calculating the difference) and then comparing the result with 0", and "the numbers are
      subtracted with a precision of DIGITS minus FUZZ digits during the comparison ... just as
      though NUMERIC DIGITS 8 had been put in effect".
    * R1 also rests on condtra.xml: LOSTDIGITS is "raised if a number used in an arithmetic
      operation has more digits than the current setting of NUMERIC DIGITS".
    * R3: rexxref states no rule for an omitted MAX argument. The derived answer rests on the
      error not depending on representation (rust/CLAUDE.md).
    * R4: funct.xml FORMAT, "If the number of places needed for the integer or decimal part
      exceeds expt or twice expt, respectively, the exponential notation is used."
* **Perf reproduces exactly**, and I traced where it moves. I ran `callgrind.sh -r 1 -B 0.5 -p
  "rexxcps decloop"` with pin, parent and my head build, exit 0. The libc-subtracted figures:
  * rexxcps: parent 17884976414, head 17852956555 (+0.5224% against the pin before, +0.3424%
    after, a task delta of -0.180%).
  * decloop: parent 2404454193, head 2405253919 (+0.0332%).

  Both agree with the report to the instruction. By self cost per function, from the task's own
  `t6.14-run` outputs:
  * rexxcps, -32.0M in all: `rexx_num::compare::compare_strings` drops 107.2M and
    `Interp::compare_values` gains 74.2M. `compare_strings` is now inlined into `compare_values`.
    That follows from the edit to `compare_values_body`: `lostdigits_check2` became the
    `#[inline(always)]` `lostdigits_check_compare` with a `fuzz` argument. It is an inlining
    change, not the loop path. The loop path costs a little: `accept_header_value` gains 0.56M (the
    new `count_lostdigits` armed test) and `loop_advance` gains 0.48M.
  * decloop, +0.80M: all of it is in `loop_advance` (+799896), which holds the inlined
    `controlled_within_wide` and its new `fuzz != 0` test with the `&mut self` receiver.

## Issues

### Critical

None.

### Important

1. **On strings, MAX/MIN's function form ignores FUZZ, and departs from both the oracle and the
   model.** Deviation 44's "decided by the value" sends a string written as a whole number down
   the crate's integer path. That path compares without FUZZ. The oracle sends strings down
   `NumberString::maxMin`, which applies FUZZ. The harness lines, the same on the parent:
   ```
   CO- 9 1 ; bif MAX | '100000001' | '100000002'
        model V[100000001] | V[100000001]
        crate V[100000002] | V[100000002]
        oracle V[100000001] | L[100000002]
   CO- 9 1 ; bif MIN | '100000002' | '100000001'   crate V[100000001], oracle and model V[100000002]
   CO- 8 7 ; bif MAX | '11' | '12'                 crate V[12], oracle and model V[11]
   -O- 9 1 ; msg MAX | '100000001' | '100000002'   crate V[100000001] = model
   ```
   It predates the task. But it is in the MAX-under-FUZZ cluster that Step 1 worked on. The report
   says that for R2's function form "the crate keeps the oracle's answer", and that is false for
   these inputs. Applying FUZZ here needs no ruling, because the oracle and the model agree on it.
   The case table has only unquoted literals, so the test cannot see this. Fix: compare with FUZZ
   on the integer path at least whenever the oracle would not have taken it (any string argument).
   Better, compare with FUZZ always, and keep only the unquoted-literal R2 rows for Moritz. Add the
   string rows to the table.

2. **The model finds oracle departures outside the table that no Deviation covers, and the report
   does not raise them.** The brief says "an oracle departure that no Deviation covers goes to
   Moritz". Each line below is the same on the parent, so the crate's answer predates the task, and
   in each the crate equals the model:
   ```
   -O- 9 0 ; infix / | 0 | 1234567890.5      model and crate V[0] | L[1234567890.5]   oracle V[0] | V[0]
   (the same for '0' / '1234567890.5', and for % and //)
   -O- 9 0 ; prefix - | 1234567890.5         model and crate L[1234567890.5]          oracle L[1.2345678905E+9]
   -O- 9 0 ; infix * | 1234567890.5 | 1      model and crate L[1234567890.5]          oracle L[1.2345678905E+9]
   -O- 9 0 ; infix ** | 1234567890.5 | 1     the same;  infix % | 1234567890.5 | 1  the same
   -O- 3 0 ; bif MAX | 1 | 12345.5           model and crate L[1.23455E+4]            oracle L[12345.5]
   -O- 3 0 ; do count | 12345.5              model and crate L[1.23455E+4]            oracle L[12345.5]
   -O- 1 0 ; bif MAX | 15 | 16.0             model and crate L[1.60E+1]               oracle L[16.0]
   ```
   * The first is the oracle skipping LOSTDIGITS on the divisor when the dividend is zero.
   * The rest are LOSTDIGITS descriptions that depend on whether the operand is an unquoted
     literal. For the same digits written as a string, the oracle answers as the crate does
     (`'1234567890.5' + 0`, and `max(5, '12345678905E-1')`: `L[1.2345678905E+9]` on both). That is
     a representation leak in the oracle.
   * The `do count | 12345.5` row comes from the new `count_lostdigits`: the parent gives `S 26.2`
     there.

   Fix: add these rows to `tests/numeric_model/task-6.14`, list them in `REPORTED` as `Oracle`,
   and list them in the report's Concerns for Moritz. M10 shows the test would carry them. The
   report's statement of the oracle-side set should say it covers the table.

3. **The model's addition is this crate's algorithm, not a derivation, and the plain reading of
   rexxref gives a different answer.** `add` follows `rexx-num/src/addsub.rs`'s `add_signed` step
   for step:
   * the fast path `adja + la > lb + k` (`adjusted_left_exp + left_len > right_len + digits`, which
     is also the oracle's `NumberStringMath.cpp:642`);
   * `over_a`/`over_b`, and `adjust` with `taken = min(adjust, adja)` (`adjusted_*_digits`, `taken`);
   * `droplow` (`drop_low_digits`).

   Its comment presents the rule as derived ("That is how rexxref's own FUZZ example comes out").
   But the FUZZ example supports only the rounding position. It does not support dropping a whole
   operand that lies outside the window of DIGITS+1 places. rexxref says the terms are "truncated,
   if necessary, to DIGITS + 1 significant digits" each, and "the operation is then carried out
   under up to double that precision". I ran 4000 random additions and subtractions (k from 1 to
   6) through the model. 138 differ from the exact sum of the truncated terms rounded from the
   leading aligned place. Both interpreters agree with the model on every one I ran natively,
   for example:
   ```
   numeric digits 2; say 27562E2 - 8E2      -> 2.8E+6 on crate and oracle  (2750000 - 800 = 2749200 -> 2.7E+6)
   numeric digits 2; say 56500E-4 + -4E-4   -> 5.7                          (5.65 - 0.0004 = 5.6496 -> 5.6)
   numeric digits 1; say -732E-4 - -256E0   -> 3E+2                         (249.927 -> 2E+2)
   ```
   ANSI's `AddSubComp` (draft pp. 52-53: `Align2 = min(Align2,NowDigits+1)`) also drops the term,
   so the crate has support. That support comes from the standard, though, not from the
   documentation the model claims to follow. Fix: either mark the window rule at `add` as "the
   documentation leaves this open; the model takes this crate's answer (ANSI agrees)", as the
   header promises for such points, or report it as R5 with the lines above. Moritz rules which.
   As written, the addition half of the model checks the crate against a copy of itself.

### Minor

4. **rexxref says more than the report allows about LOSTDIGITS on numeric builtin arguments.**
   funct.xml (the built-in function preamble) reads: "Any argument named as a number is rounded,
   if necessary, according to the current setting of NUMERIC DIGITS (as though the number had been
   added to 0) and checked for validity before use. This occurs in the following functions: ABS,
   FORMAT, MAX, MIN, SIGN, and TRUNC". Read with condtra.xml's LOSTDIGITS definition, that argues
   for LOSTDIGITS on every such argument. Today all three agree on no LOSTDIGITS:
   `3 0 ; bif ABS | '1234.567'` gives `V[1.23E+3] | V[1.23E+3]` on the model, crate and oracle
   alike, and the same holds for SIGN, TRUNC, FORMAT and the MAX target.
   * The model rests this on Ruled 10, whose premise is "only the ANSI standard differs".
   * Concern 3 says "No documentation backs" the quiet whole-number path, but the documentation in
     fact argues against it.

   This belongs with R1 to R4 for Moritz. I propose no Deviation.

5. **The MAX witness's "negative pair" never reaches the code changed.** In
   `max(-1111111111, -2222222222)` the operands are prefix-minus expressions, already rounded
   before MAX sees them. The line prints `-1.11111111E+9` on the parent, on HEAD and on the
   oracle. A line that does reach the new rounding is `min(-5, '-1111111111')`: it gives
   `-1111111111` on the parent and `-1.11111111E+9` on HEAD and on the oracle. The report's list of
   what the witness covers should drop "a negative pair", or the line should change.

6. **The model's `pow` uses host `//`, `%` and `*` on the power operand** (`v // 2`, `v % 2`,
   `abs(...) * p`). That is exact at the model's NUMERIC DIGITS 20, since `p` has at most 18
   digits. But the header says host arithmetic touches "nothing but counters, lengths and
   exponents". Either say that the power counts as a counter, or take its bits from the digit
   string.

7. **The test checks `ORACLE_DEPARTURES`' Deviation numbers only as a set** (`{44, 47}`). A case
   labelled with the wrong Deviation would pass. Each label I read matches its case.

### Concern 3 (asked: is the quiet path a representation leak?)

There is no leak in the crate. Values computed at DIGITS 12 and read at 9 behave like the literal
string in every case: `1111111111 + 0`, `11111111110 / 10`, `1e9 + 111111111`, `1111111111 * 1`,
`101 * 11001100.11 % 1`. Each prints `1111111111`, and each gives
`max(2, v)` quiet / `min(99999999999, v)` -> `L2 1111111111`.

The oracle does leak. For `c = 1e9 + 111111111` it raises `L1 1111111111`, and for the others it
stays quiet.

The crate's rule depends on spelling, though, not on numeric value. `' 1111111111'`,
`'1111111111 '`, `'+1111111111'`, `'1111111111.'` and `'1111111111e0'` all raise `L1 1111111111`
in the crate, while `'1111111111'` is quiet. Spelling is observable, so this is not a representation
leak. But it is not "decided by the value" in the numeric sense Deviation 44's text suggests, and
Minor 4's passage argues against having a quiet path at all. `max(1111111111+0, 2)` is not a test
of this point: the `+` itself raises LOSTDIGITS on `1111111111` before MAX runs.

## Gated corpus

`REXX_CORPUS_GATE=1 ... cargo test --release -p rexx-exec --test corpus`: exit 0, `1148 of 1148
matching`, 29 passed, 1 ignored.

## Assessment

**Task quality:** Good. The fixes are right where they reach, the model is a real differential
check, and every mutation I tried was caught.

**Reasoning:**
* The model was meant to find departures, and it does find them, but only within the table. Just
  past the table there are an unreported crate departure in the cluster's own MAX-under-FUZZ path
  (Important 1) and a set of uncovered oracle departures (Important 2).
* Its addition rule is the crate's code presented as a derivation (Important 3).
* All three are small to address: one comparison change, and table rows plus report lines.

**Verdict: CHANGES REQUESTED** (Importants 1 to 3. The Minors can ride along. Minor 4 goes to
Moritz with R1 to R4.)
