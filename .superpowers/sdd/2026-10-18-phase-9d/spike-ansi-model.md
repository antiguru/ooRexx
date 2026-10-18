# Spike: the ANSI X3.274 draft as a third voice on numerics

Throwaway spike, 2026-10-18, crate at `cc1285116`. Nothing from the standard is in the repo. The
transcription, stubs, driver and outputs are in the session scratchpad under `ansi/`
(`lib/arith.rex`, `lib/model.rex`, `cases.tsv`, `gen.sh`, `run.sh`, `table.md`, `out/`,
`out-asis/`). The source is `j18pub.pdf` (sha256 `f38ac9b1…058df`).

## What was extracted

* 7.4.2 to 7.4.5 and 7.4.7: prefix, power, multiplication, addition (with its integer path), and
  normal comparison.
* 7.4.10 `ArithOp`, with `Prepare`, `PostOp`, `Floating`, `AddSubComp`, `Multiply`, `DivType`
  and `Power`. LOSTDIGITS is raised only in `Prepare`.
* 9.2.1 `CheckArgs`, with the `NUM`, `WHOLE` and `WHOLE>=0` types and `Cdatatype`/`Edatatype`.
* 9.3.8 `DATATYPE`, without the `B` and `X` branches, and 9.4.1 to 9.4.6 (`ABS`, `FORMAT`, `MAX`,
  `MIN`, `SIGN`, `TRUNC`).
* 8.3.6 `DO`: only how the repetitor and the `FOR` count are accepted and normalised, not the
  iteration.
* 8.4.1 `#Raise`, as the basis of the LOSTDIGITS stub.

## Stubs and transcription decisions

* State variables become tails of one exposed stem, and the tails are constant symbols
  (`M.0Digits`). Plain tails collided with the standard's own variable `Digits`, so `Prepare`
  read `M.9`.
* `#Raise 'LOSTDIGITS'` records the operand and returns. That is what 8.4.1 does when the
  condition is disabled. The model's `+LD` in the table means "would fire if trapped".
* `#Raise 'SYNTAX'` prints the code and exits. Inside `Power`'s recursion it sets a flag instead,
  so the `'MN'` overflow path still works.
* `ArithOp` runs at `NUMERIC DIGITS 1000` on the host, as the text assumes ("sufficiently high").
  Its own mantissa arithmetic is on integers only.
* In the text, `Recursion` (power) uses the host's `*` and `/`. The stub calls `ArithOp` again at
  the raised digits. `WholeNumber2` calls the model's `DATATYPE` instead of the host's.
* The operator definitions check their operands with `datatype(x)=='NUM'`. In the model,
  `DATATYPE` normalises with `0 + x`. That is an addition, and its definition calls `datatype`
  again, so the definition never terminates. The stub checks operands by syntax only, without
  normalising.
* "Integer" in 7.4.5 is read as an optional sign followed by digits only. "Length of the integer"
  counts digits, not the sign.
* 7.4.7 decides `Difference > 0` with another comparison, which has the same problem. The stub
  reads the sign character of `Difference`.
* Every operator inside the BIF code (`0 + x`, `Max < Next`, `10**Num * Number % 1`,
  `Integer + 1`) goes through the model operators at the caller's digits, as 9.4 requires.
  Arithmetic on small counters and positions stays native.
* `#Bif_Digits.` is 9 for every BIF (8.2.1). `#Limit_ExponentDigits` is used as an exponent value
  of 999999999, because the text compares the exponent against it.
* FORMAT's `#Form.#Level == 'ENGINEERING'` and `form()` are read as the model's form `E`.
* **Erratum applied.** 7.4.7's shortcut for operands of opposite sign negates the second operand
  with `+` where it needs `-`. As written, any two positive numbers compare `G`. The as-written
  runs give `1111111111 = 1111111111.0` → 0, `max(2, 1111111111)` → 2, and
  `min(2, 1111111111)` → 1.11111111E+9 (`out-asis/`). The table uses the corrected comparison.

## Does it run

Yes, on both interpreters. The model's output is byte-identical on the oracle and the crate for
all 62 inputs, so the host's own arithmetic does not leak into it.

## Results

In a native column, `+LD` means that a run with `SIGNAL ON LOSTDIGITS` trapped LOSTDIGITS. The
native value comes from a separate run without the trap. In the Agreement column, `O=C` means
that the two interpreters agree and the model differs. `M=C` means that the model and the crate
agree and the oracle differs.

| Input (DIGITS) | Model | Oracle | Crate | Agreement |
|---|---|---|---|---|
| `1111111111 + 0` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `1111111111 - 1111111110` (9) | 1 | 0 +LD | 0 +LD | O=C |
| `1000000000 + 1` (9) | 1.00000000E+9 | 1.00000000E+9 +LD | 1.00000000E+9 +LD | O=C |
| `999999999 + 1` (9) | 1.00000000E+9 | 1.00000000E+9 | 1.00000000E+9 | all |
| `11111111110 + 0` (9) | 1.11111111E+10 +LD | 1.11111111E+10 +LD | 1.11111111E+10 +LD | all |
| `1111111111.4 + 0` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `1.1111111111E+5 + 0` (9) | 111111.111 +LD | 111111.111 +LD | 111111.111 +LD | all |
| `1.5 + 1111111111` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `1111111111 * 1` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `0 * 1111111111` (9) | 0 +LD | 0 +LD | 0 +LD | all |
| `1111111111 / 1` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `1111111111 % 1` (9) | SYNTAX 26.11 +LD | SYNTAX 26.11 +LD | SYNTAX 26.11 +LD | all |
| `1111111111 // 7` (9) | 5 +LD | 5 +LD | 5 +LD | all |
| `1111111111 ** 0` (9) | 1 +LD | 1 +LD | 1 +LD | all |
| `1111111111 ** 1` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `2 ** 40` (9) | 1.09951163E+12 | 1.09951163E+12 | 1.09951163E+12 | all |
| `2 ** -3` (9) | 0.125 | 0.125 | 0.125 | all |
| `-(1111111111)` (9) | -1.11111111E+9 +LD | -1.11111111E+9 +LD | -1.11111111E+9 +LD | all |
| `+(1111111111)` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `1111111111 = 1111111111.0` (9) | 1 +LD | 1 +LD | 1 +LD | all |
| `1111111111 > 1` (9) | 1 +LD | 1 +LD | 1 +LD | all |
| `1000000000 = 1E+9` (9) | 1 | 1 +LD | 1 +LD | O=C |
| `1.00000001 = 1.00000002` (9, FUZZ 1) | 0 | 1 +LD | 1 | none |
| `max(2, 1111111111)` (9) | 1.11111111E+9 +LD | 1111111111 | 1111111111 | O=C |
| `2~max(1111111111)` (9) | 1.11111111E+9 +LD | 1111111111 | 1.11111111E+9 | M=C on value, no LD in either |
| `max(1111111111, 2)` (9) | 1.11111111E+9 +LD | 1.11111111E+9 | 1.11111111E+9 | O=C |
| `1111111111~max(2)` (9) | 1.11111111E+9 +LD | 1.11111111E+9 | 1.11111111E+9 | O=C |
| `max(2, 1000000000)` (9) | 1.00000000E+9 | 1000000000 | 1000000000 | O=C |
| `min(2, 1111111111)` (9) | 2 +LD | 2 | 2 | O=C |
| `2~min(1111111111)` (9) | 2 +LD | 2 | 2 | O=C |
| `min(1111111111, 2222222222)` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `1111111111~min(2222222222)` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `min(1.5, 1111111111)` (9) | 1.5 +LD | 1.5 +LD | 1.5 +LD | all |
| `abs(-1111111111)` (9) | 1.11111111E+9 +LD | 1.11111111E+9 +LD | 1.11111111E+9 +LD | all |
| `abs(' -12.50 ')` (9) | 12.50 | 12.50 | 12.50 | all |
| `sign(1111111111)` (9) | 1 +LD | 1 | 1 | O=C |
| `sign('-0.0')` (9) | 0 | 0 | 0 | all |
| `trunc(1111111111)` (9) | SYNTAX 26.11 +LD | 1111111110 | 1111111110 | O=C |
| `trunc(1234.5678, 2)` (9) | 1234.56 | 1234.56 | 1234.56 | all |
| `trunc(12345678.95, 1)` (9) | 12345679.0 +LD | 12345679.0 | 12345679.0 | O=C |
| `trunc(1.5E+12)` (9) | SYNTAX 26.11 | 1500000000000 | 1500000000000 | O=C |
| `format(1111111111)` (9) | 1.11111111E+9 +LD | 1.11111111E+9 | 1.11111111E+9 | O=C |
| `format(1111111111,,2)` (9) | 1.11E+9 +LD | 1.11E+9 | 1.11E+9 | O=C |
| `format(12345.6789, 3, 2)` (9) | SYNTAX 40.38 | SYNTAX 93.942 | SYNTAX 93.942 | O=C, same failure |
| `format(0.0000001234,2,,,3)` (9) | ` 0.0000001234` | ` 1.234E-7` | ` 1.234E-7` | O=C |
| `format(12345.73,,,2,2)` (9) | 1.234573E+04 | 1.234573E+04 | 1.234573E+04 | all |
| `format(9.9999,1,2)` (9) | SYNTAX 40.38 | SYNTAX 93.942 | SYNTAX 93.942 | O=C, same failure |
| `datatype(1111111111, 'W')` (9) | 0 +LD | 0 +LD | 0 +LD | all |
| `datatype(1000000000, 'W')` (9) | 0 | 0 +LD | 0 +LD | O=C |
| `datatype('123456789.0', 'W')` (9) | 1 | 1 +LD | 1 +LD | O=C |
| `datatype('1E+9', 'W')` (9) | 0 | 0 | 0 | all |
| `datatype('1E+8', 'W')` (9) | 1 | 1 | 1 | all |
| `datatype(' 1111111111.5 ', 'W')` (9) | 0 +LD | 0 +LD | 0 +LD | all |
| `datatype(1111111111, 'N')` (9) | 1 +LD | 1 | 1 | O=C |
| `datatype('12345678901234567890.0', 'W')` (20) | 1 | 1 +LD | 1 +LD | O=C |
| `datatype('1E+20', 'W')` (20) | 0 | 0 | 0 | all |
| `datatype('99999999999999999999.5', 'W')` (20) | 0 +LD | 0 +LD | 0 +LD | all |
| `do 1111111111; end` (9) | SYNTAX 26.2 +LD | runs | SYNTAX 26.2 | M=C on outcome, crate raises no LD |
| `do i = 1 for 1111111111; end` (9) | SYNTAX 26.3 +LD | runs | SYNTAX 26.3 | M=C on outcome, crate raises no LD |
| `do 1000000000; end` (9) | SYNTAX 26.2 | runs | SYNTAX 26.2 | M=C |
| `do '3E+0'; end` (9) | accepted | runs | runs | all |
| `do 3.0; end` (9) | accepted | runs | runs | all |

The model and the interpreters differ systematically in four places, and the oracle and the crate
agree in all four:

* The model raises LOSTDIGITS only for a non-zero digit beyond DIGITS. Both interpreters also
  raise it for trailing zeros (`1000000000`, `123456789.0`).
* For whole numbers, 7.4.5 adds exactly whenever the exact result fits in DIGITS, without
  rounding the operands. That is why `1111111111 - 1111111110` is 1 in the model.
* `TRUNC` is defined through `%`, so a result wider than DIGITS raises 26.11.
* FORMAT's rule for switching to exponential notation differs (`0.0000001234` row).

There are two defects in the standard as well. 7.4.7 never reaches `ArithOp`'s comparator path,
so NUMERIC FUZZ has no effect on any comparison. The second is the sign erratum described above.

Separately from the model, the FUZZ row shows a divergence between the two interpreters. The
oracle traps LOSTDIGITS for `1.00000001 = 1.00000002` under FUZZ 1, and the crate does not. I
found no queued item that covers it.

## The queued numeric items

* **`2~max(1111111111)`.** The model rounds to `1.11111111E+9` because `CheckArgs` normalises
  every `NUM` argument with `0 + x`, so the crate's value is the standard's. The model also raises
  LOSTDIGITS, and neither interpreter does. Both interpreters answer an unrounded `1111111111`
  for the function form `max(2, 1111111111)`. The crate therefore rounds in the method form but
  not in the function form.
* **MIN (Deviation 44).** The model answers 2 for `min(2, 1111111111)`, the same value as both
  interpreters, and raises LOSTDIGITS. Neither interpreter raises it. They do raise it when a
  non-integer argument is present (`min(1.5, 1111111111)`).
* **`do 1111111111` and `do i=1 for 1111111111`.** The model raises 26.2 and 26.3. 8.3.6 checks
  the count with `datatype(..., 'W')` at the program's digits. `DATATYPE` normalises the count to
  `1.11111111E+9`, and the `E` makes the `W` test fail. LOSTDIGITS fires first. The crate matches
  the model's outcome but raises no LOSTDIGITS. The oracle runs the loop. The model also rejects
  `do 1000000000`, whose zeros raise no LOSTDIGITS, and so does the crate.
* **LOSTDIGITS on each operator.** The model and both interpreters agree on every non-integer
  operand. The integer path and the trailing zeros are the exceptions listed above.
* **`DATATYPE(x,'W')` on wide whole numbers.** All three agree on the value in every row. Only
  the LOSTDIGITS behaviour on trailing zeros differs.

## Cost of a committed, fetch-at-test-time version

A version that fetches the draft at test time has to rebuild everything that this spike did by
hand, without committing any of the standard's text:

* a pinned URL and checksum (the scratchpad records neither the PDF's origin nor its URL);
* `pdftotext` at test time;
* an extractor that locates the sections by their headings;
* a committed transformation script.

The `ArithOp` transformation is mechanical: five renames plus replacing `Recursion` and
`WholeNumber2`. The BIF code is not mechanical. About 20 sites have to be routed through the
model's operators, the variables need constant tails, and the erratum and the two stubs that
replace the self-referential definitions are needed as well. Each of these sites is a positional
edit against a fetched text, so the script breaks if the extraction changes by a single line. I
estimate one to two days to make it robust for this subset, plus a harness, roughly what
`cases.tsv`/`gen.sh`/`run.sh` do here. Adding a test-time dependency on the network and on
poppler is a further cost.

## Recommendation

Do not commit it as a gate or as a test. The model's arithmetic core is sound. It agrees with
both interpreters on the value of every operator row except two: the integer path
(`1111111111 - 1111111110`) and FUZZ. On LOSTDIGITS for operators, it differs only on the integer
path and on trailing zeros. Where it differs, ooRexx departs from the draft and the crate
follows ooRexx: LOSTDIGITS on trailing zeros, operand rounding for integers, TRUNC, FORMAT, and
LOSTDIGITS on BIF arguments. As a parity reference it would therefore produce standing `O=C` rows
that need licences rather than fixes. Use it as an ad hoc tie-breaker when ruling on an item where
the oracle and the crate disagree. That is how it served the three queued items here. For MAX and
DO, the standard agrees with the crate's value, and for MIN all three agree on the value. In all
three, the open question is LOSTDIGITS, and neither interpreter follows the model there. If it is kept, keep the scratchpad copy and record the
pinned source.
