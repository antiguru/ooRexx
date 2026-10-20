# Task 6.14 re-review: fix rounds 1 and 2

Scoped to `345c6d04a..bca9e99d0`: the code commits `9b07a4e33` and `bc176f5e8`, the records, and the
controller's Deviation 53 and the Deviation 47 extension. I built HEAD (`bca9e99d0`) into a fresh
scratchpad target, `rerev614-target`, with exit 0. Probes ran from fresh `mktemp -d` directories,
the crate under `memcap 2G timeout 20` and the oracle under the standard wrapper. I compared stdout,
stderr and rc separately. The model harness from the first review was pointed at the new binary.
It builds the same program as `numeric_model.rs`, and its output marks crate != model as `C`,
oracle != model as `O`, and model-on-crate != model-on-oracle as `M`.

## Spec compliance

The rulings are implemented as Moritz gave them. R1, R2, R3 and R6 follow rexxref. R4 and R7 are
reverted. Deviation 44's rounding is unchanged, and the extension of Deviation 47 holds. One
sentence of Deviation 53, and the matching doc line in `numeric_model.rs`, misstate what the oracle
does (Important 1).

## Strengths

* **The first review's findings are closed:**
  * **Important 1.** The integer path is gone, so both forms go through `max_min_over`, which
    applies FUZZ. `9 1 ; bif MAX | '100000001' | '100000002'` now gives `V[100000001]` on the
    crate, the model and the oracle untrapped. The trapped LOSTDIGITS the oracle adds is listed as
    `written-value` under Deviation 44.
  * **Important 2.** The zero-dividend and description rows are in the table, labelled R6 and
    `description` under Deviation 53. But my first review framed the description rows as
    "literal versus string". That framing was wrong (Important 1 below).
  * **Important 3.** `add`'s comment now says the window rule is the interpreters' answer and
    that rexxref is silent on it.
  * **Minor 5.** The row now reads `min(-5, '-1111111111')`.
  * **Minor 6.** `pow` takes the power's bits by halving the digit string (`halve`), and its
    overflow test multiplies digit strings.
  * **Minor 7.** Each departure carries its rule, `deviation_of` maps each rule to its
    Deviation, and the test checks that the Deviation can license that kind of case.
* **R4 and R7 are fully reverted.**
  * `git diff 525ca9989 bca9e99d0 -- rust/crates/rexx-num rust/corpus/lang/lostdigits_max_min.rex
    .../sourceline_oracle/lostdigits_max_min.txt` prints nothing.
  * These rows give the same answer on the model, the crate and the oracle:
    `3 0 ; bif FORMAT | 12.3456789 | | | | 3` gives `V[12.3]`, `9 0 ; bif FORMAT | 12.3456789 | | |
    | 3` gives `V[12.3456789]`, `3 0 ; bif ABS | '1234.567'` gives `V[1.23E+3] | V[1.23E+3]`, and
    `3 0 ; bif MAX | '1234.567' | 1`, `bif TRUNC | '1234567890.5'` and `msg ABS | '1234567890.5'`
    give no LOSTDIGITS.
  * The whole suite, which I ran myself (`REXX_SUITE_GATE=1 ... l3_core_suite`), exits 0 with
    402 groups green in `verdicts.tsv`.
* **Every example in Deviation 53 holds on both sides.** In each row the crate equals the model:

  | example | crate | oracle |
  | --- | --- | --- |
  | `1.00000001 = -2` (FUZZ 1, literal and string) | `V[0] \| L[1.00000001]` | `V[0] \| V[0]` |
  | `1.00000001 > 0` (FUZZ 1) | `V[1] \| L[1.00000001]` | `V[1] \| V[1]` |
  | `max(100000001, 100000002)` (FUZZ 1, function and message) | `V[100000001]` | `V[100000002]` |
  | `min(100000002, 100000001)` (FUZZ 1) | `V[100000002]` | `V[100000001]` |
  | `max(1,,3)`, `min(1,,3)`, `max(1,2,,4)` | `S 40.5` | `S 93.903` |
  | `'55'~max(,77)`, `'55'~min(77,,88)` | `S 93.903` | `S 40.5` |
  | `0 / 1234567890.5`, `'0' // '1234567890.5'`, `0 % 1234567890.5` | `V[0] \| L[1234567890.5]` | `V[0] \| V[0]` |
  | `-1234567890.5` at DIGITS 9 | `L[1234567890.5]` | `L[1.2345678905E+9]` |

  Deviation 47's extension holds too. `do 1111111111` and `do i = 1 for 1111111111` give
  `S 26.2 | L[1111111111]` (26.3 for the FOR count) here and `V[ran] | V[ran]` on the oracle. The
  string spelling `'1111111111'` agrees on both.
* **This crate's description does not depend on spelling.** I generated 184 literal/string pairs
  and ran each under INTERPRET:
  * operands: `1234567890.5`, `1.2345678905E9`, `12345678905E-1`, `0001234567890.5`,
    `1234567890.50`, `1234567891`;
  * operations: `+ - * / % // ** = < >= \=`, both operand positions, prefix `-` and `+`,
    `max(1, x)`, `min(1, x, 2)`, `1~max(x)`, `datatype(x,'W')`, `do x`, `do i = 1 for x`, and `=`
    at DIGITS 11, FUZZ 2.

  No pair differs on the crate. Compiled code agrees: `- 1234567890.5` and `- '1234567890.5'` both
  describe `1234567890.5`. A value computed at DIGITS 3 (`v = '12345' || '.5'`) gives the same
  descriptions as the literal.
* **The departures table matches the oracle.** `numeric_model` asserts each listed case's exact
  oracle answer and that every unlisted case agrees, and it passes. Every row I re-ran in the
  harness (above) matches the table in `r2/oracle-departures.tsv`.
* **Tests.**
  * `cargo test --release -p rexx-exec --test numeric_model --test licensed_divergences`: exit 0,
    and `the_model_answers_every_case_as_this_crate_does`,
    `on_the_oracle_the_model_parts_only_where_the_numeric_deviations_say`,
    `the_prose_rows_and_this_table_name_the_same_divergences` and
    `the_licensed_divergences_still_diverge_exactly_as_recorded` all pass.
  * The four numeric witnesses (`lostdigits_fuzz_compare`, `lostdigits_do_count`,
    `lostdigits_max_min`, `string_extremes_refusals`) are identical to the oracle on stdout,
    stderr and rc (158, 0, 158, 163).
* **No test requires a marker under Deviation 53, and the marker reader accepts any row.**
  `licensed_divergences.rs`'s `names_the_exclusions_file_claims` takes every line containing
  `LICENSED DIVERGENCE WITNESS: `, whichever Deviation it sits under. It asserts only that this
  set equals the `LICENSED_DIVERGENCES` names. Deviation 53 has no marker. Its witness is
  `numeric_model`'s `ORACLE_DEPARTURES`, which asserts the set of Deviation numbers `{44, 47,
  53}`. It does not read `phase-4-exclusions.txt` (Minor 3).

## Issues

### Critical

None.

### Important

1. **Deviation 53's description bullet is false, and so is `numeric_model.rs`'s `description`
   doc line. My first review's Important 2 introduced the error.** The bullet says the oracle
   "renders an unquoted literal from its internal form (`-1234567890.5` at DIGITS 9 describes
   `1.2345678905E+9`) and gives the written digits for the same value as a string". The quoted
   string renders the same way. Compiled, at DIGITS 9:
   ```
   C A1 - lit        1234567890.5        O A1 - lit        1.2345678905E+9
   C A2 - str        1234567890.5        O A2 - str        1.2345678905E+9
   C A3 str * 1      1234567890.5        O A3 str * 1      1.2345678905E+9
   = A4 str + 1      1234567890.5
   ```
   Under INTERPRET the oracle's description differs between the literal and the string spelling
   in 0 of the 184 pairs. Only the Deviation 44 and 47 rows differ there (`max(1, 1234567891)`
   quiet against `max(1, '1234567891')` raising, and `do 1234567891` running).

   What the oracle's description depends on:
   * **The operator.** Infix `+` and `-` and the comparisons give the operand as written. Prefix
     `-` and `+`, `*`, `/`, `%`, `//` and `**` render it (`'12345678905E-1' * 1` describes
     `1.2345678905E+9`).
   * **For MAX and DO, whether the value is source text or computed.** Compiled at DIGITS 3:
     ```
     = B1 lit * 1  d3  12345.5      = B2 str * 1  d3  12345.5
     O B3 max lit  d3  12345.5      O B4 max str  d3  12345.5   (crate 1.23455E+4 for both)
     O B5 do lit   d3  12345.5      O B6 do str   d3  12345.5   (crate 1.23455E+4 for both)
     O computed v * 1   1.23455E+4  (crate 12345.5)
     ```
     Under INTERPRET at DIGITS 3 the same `max(1, 12345.5)` describes `1.23455E+4` on the
     oracle.

   The `numeric_model.rs` doc line has the same two errors: "the oracle names an unquoted
   literal's LOSTDIGITS by its spelling at parse time, and this crate by its value". For the
   operators, this crate names the operand as written (`'12345678905E-1' * 1` gives
   `L[12345678905E-1]`), not by its value.

   The licensed observable is right: the crate's description is the same whatever the spelling,
   and the oracle's rendering differs on the listed rows. Only the explanation is false. Fix: the
   controller rewrites the bullet's second sentence, for example: "The oracle renders the operand
   of prefix `+`/`-`, `*`, `/`, `%`, `//`, `**`, MAX/MIN and a DO count where this crate gives it
   as written (operators) or rendered at the digits in force (MAX/MIN, DO), and for a value from
   the source text it renders at the digits the clause was translated at." Correct the
   `description` doc line to match. The licence and the table rows stay.

### Minor

2. **`target_number` is now a no-op wrapper.** It only calls `target_parse`. It is what is left of
   R7's check after the revert, and its doc carries the TESTBIFS reading. Fold it back into one
   function, or keep it and say why.
3. **No test ties Deviation 53 to its executable witness.** `numeric_model` asserts the numbers
   `{44, 47, 53}`, but nothing checks that `phase-4-exclusions.txt` has a row 53, or that row 53
   names `numeric_model`. The R1 and R3 lines removed from the corpus witnesses are now pinned only
   in `ORACLE_DEPARTURES`, which is acceptable. This is a gap to note, not a failure.
4. **A harness-encoding note, no action needed.** `bif MAX |` (an empty field) encodes `max()`
   as one omitted argument, so the model answers `S 40.5` where both interpreters give `S 40.3`
   (too few arguments). No table case uses that encoding.

I did not re-measure perf. The round 2 record gives rexxcps +0.0000%, and no code on the paths
rexxcps runs changed after `bc176f5e8`.

## Assessment

**Task quality:** Good. The code, the model and the table implement exactly the rulings. R4 and
R7 are gone without residue, and the suite is green.

**Reasoning:** The one open item is prose in the licence: Deviation 53 describes a literal-versus-
string distinction the oracle does not make. It came from my own first review. The crate's
behaviour and the listed rows are correct.

**Verdict: CHANGES REQUESTED** (Important 1, a text correction to Deviation 53 and one doc line.
The Minors can ride along.)
