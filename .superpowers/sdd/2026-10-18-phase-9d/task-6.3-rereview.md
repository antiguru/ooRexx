# Task 6.3 fix round 1: scoped re-review

Verdict: approve. No open Critical or Important findings.

Method: built head (1019eb92c, crate sources identical to cc1285116 apart from records), ran the oracle and the crate on the witness and on probes from a fresh dir.

## I1: fixed

* Witness `lostdigits_max_min.rex`: oracle and crate agree line for line, including the `::OPTIONS LOSTDIGITS SYNTAX` run (98.972, rc 158). The only textual difference is the order of the stderr block against stdout, which is stream interleaving.
* 19 probes of my own agree with the oracle: mixed signs (`max(-1.5, 1111111111, -2222222222)`), exponent arguments (`1E+12`, `1E10`, quoted `1e10`, `1111111111E+7`, `1111111111E-15`), `max(1.0, 1111111111)`, wide argument last or in the middle of 3 to 5 arguments, leading `+`, leading zeros with blanks, ENGINEERING form, and the message form on non-integer receivers (`1.5~max(1111111111)`, `1.5~max(2,1111111111)`, `'1.5'~min(' 1111111111')`).
* Integer first argument stays silent in both forms: `max(1, 2, 1111111111)`, `max(1111111111, 1.5)`, `1~max(2,1.5)`. `max(1, 1.5, 1111111111)` and `1~max(1.5, 1111111111)` raise, as on the oracle.

## M1 and M3: fixed

The witness covers ABS, FORMAT, TRUNC, SIGN and the first argument as silent. The report wording is now accurate.

## `Number::render_unrounded`

* Output matches the oracle's condition description for: leading and trailing blanks (blanks dropped), leading `+` (dropped), leading zeros (dropped, `'  00001111111111.00  '` gives `1.11111111100E+9`), exponent forms, and ENGINEERING (`11.11111111E+15`, `111.1111111E-15`, `1.111111111E+12`, `11.11111111E-12`).
* `rexx-num` tests pass (23). The function has no unit test of its own. Its coverage is the witness, which only runs on the oracle diff and not in `cargo test` (the `sourceline_oracle` test passed in 0.01 s, so it validates the file, it does not execute the cases). Minor, M4.

## Perf

Message form: `lostdigits_armed && !integer_path_applies(...)` short-circuits on the flag, so the integer test and `to_text` calls run only when LOSTDIGITS is armed. Function form: `lostdigits_check_with` returns on `!armed` before the closure runs, and the description closure runs only on a raise. Nothing is added to the unarmed path beyond one flag test per argument.

## Findings

* M4 (minor): no direct unit test for `render_unrounded`; it is exercised only by the oracle-diffed witness.

## Out-of-scope observations (not findings)

* `1~max(1111111111, 1111111112)` answers `1111111112` on the oracle and `1.11111111E+9` here. That is the queued `max-integer-unrounded` shape, not LOSTDIGITS.
