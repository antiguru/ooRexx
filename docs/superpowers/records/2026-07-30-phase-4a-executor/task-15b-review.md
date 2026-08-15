STATUS: DONE

## Verdicts

**Spec compliance: PASS, with two reservations for the lead.**
The criterion's substance is met and is not vacuous: every row is evaluated, the count is reported, the comparison is on bytes, and the table's ability to fail is proven by the author's two committed proofs (re-run green) plus four independent perturbations of my own, including two the author did not try.
Reservations: the STRICT gate as implemented cannot pass within 4a at all (F7), and "the sub-phase that unblocks it" is really "the first blocked construct's owner" (F6); both need a ruling, not a code fix chosen unilaterally.

**Code quality: PASS.**
The harness is careful and conservative; the defects found are all in comments and reports, not behavior: one claimed check that does not exist (F5), two misstated pieces of evidence (F1, F3), and one framework-unfaithful justification (F2).

## Falsification evidence (the "could it fail" question)

* Author's proofs re-run green as part of `cargo test -p rexx-exec --test assertions` (4 tests, 1.28s): value perturbation, raise perturbation including 26.11 vs 26.2 (same major, different sub), and the digits/form-defaulted witness.
* My own, via a scratch crate driving `rexx_exec::run_program` with a mirrored `program_for` (the committed `classify` is a plain `Vec<u8>` equality, verified by inspection):
  * **Numerically-equal, byte-different expected**: ADDITION `test_198`'s expected `20.000E+12` replaced with `2.0000E+13` (the same number) mismatches -- lines render `"20.000E+12"` vs `"2.0000E+13"`. A numeric comparison would have passed this; the byte comparison fails it. Also anchors the expected side: `say 20.000E+12` renders verbatim in both `rexx_exec` and the oracle (checked under the oracle, rc 0, identical bytes), so there is no common-mode normalization hiding behind comparing rexx_exec against itself.
  * **Carried `NUMERIC DIGITS` perturbed on a raise row** (the perturbation the author did not do): DIVISION `test_262` with `digits` forced from 5 to the default 9 stops raising entirely (exit 0, prints `153484`), which classifies as `RaiseMismatch(None)`. A harness that silently defaulted digits would therefore have failed all the digits-dependent raise rows loudly, not passed them.
  * **Trailing-blank perturbation** on a quoted expected (`'1'` -> `'1' || ' '`) mismatches -- the case non-strict `=` would forgive and `==`/bytes must not.
  * Oracle ground truth for the raise witness confirmed: `numeric digits 5; say "-5678932" % "-37"` raises Error 26.11, rc 230.

## Counts and invariant, re-verified

* `cargo test -p rexx-extract`: 18 tests green, including both whole-corpus pins.
* `rexx-extract-assertions --suite`: 11 groups, 4,269 assertSame calls, 4,259 rows, 10 dropped, per-group table identical to the 15a report, and the binary's internal rows+dropped==calls assertion did not fire.
* Conversion extent re-derived independently (Python scan of the 11 files): 184 methods where a marker precedes an assertSame, exactly one assertSame after each marker, split 60/6/118 across DIVISION/EXPONENT/REMAINDER, zero elsewhere -- matching the pinned test.
* REPORT-mode run: 4,224 of 4,259 passing, "by kind: RUNTIME-BLOCKED: 35", zero MISMATCH / RAISE-MISMATCH / ANOMALY, matching the report.

# Task 15b review -- the `base/expressions` assertion-table harness (gate criterion 2)

Reviewer scope: commit `7eeb309d` (`rexx-exec/tests/assertions.rs`, `rexx-extract` conversion changes).
Question under review: not "does the table pass" but "could it fail".

Planned checks, findings appended as established:

1. Re-run the report's two falsification proofs; add my own perturbations, including a carried `NUMERIC DIGITS` perturbation.
2. Verify the comparison is byte-for-byte, including raise rows.
3. Audit the 184 converted `expectSyntax` rows: full major.sub match, marker scope, second-marker semantics, assertSyntaxError/assertRuntimeError/expectCondition distinctions.
4. Verify the 35 failing rows' phase attributions against the split table.
5. Confirm the rows+dropped invariant and the 4259/10 figures.

## Findings

### Established so far (evidence in hand)

**F1 (Important): the report's PRECEDENCE mechanism claim is false, though its conclusion holds.**
The 15b report says "PRECEDENCE.testGroup uses self~assertSyntaxError 139 times but as a wrapping call".
Measured: PRECEDENCE has **0** `assertSyntaxError` occurrences and **139 real `self~expectSyntax` markers**, each followed by a raising assignment (`xre=0/0` etc.), never an assertSame.
`assertSyntaxError` lives in Literals (33 occurrences, wrapping form `self~assertSyntaxError((15.1, 1), self~hex(" "))`).
The load-bearing conclusion (0 PRECEDENCE rows convert; no assertSame follows assertSyntaxError/assertRuntimeError anywhere) was re-verified independently and holds.
The same false claim is repeated in `rexx-extract/tests/extract_assertions.rs`'s `base_expressions_expect_syntax_conversion_counts` doc comment ("PRECEDENCE's 139 self~assertSyntaxError calls").

**F2 (Important): the "checks within the same call" reasoning for assertSyntaxError is wrong per OOREXXUNIT.CLS.**
`lib.rs`'s comment says assertSyntaxError/assertRuntimeError "perform their own check *within the same call*, so nothing is left pending".
The framework source (OOREXXUNIT.CLS:1203-1220) shows no check within the call: `assertSyntaxError` calls `self~expectSyntax(error)` then `.package~new(...)` and relies on the raise escaping to the method trap.
If the wrapped code did NOT raise, the expectation WOULD be left pending for later statements, exactly like a bare `expectSyntax`.
Corpus-safe today (verified: no assertSame follows either call in any method), but the stated distinction is framework-unfaithful and there is no forward guard for assertSame-after-assertSyntaxError the way there is for expectCondition.

**F3 (Minor): "19 occurrences" is wrong; it is 184.**
`parse_raise_expectation`'s doc ("the shape every one of the 19 occurrences preceding an assertSame in base/expressions uses") and the report's same claim: measured, 184 marker-methods each with exactly one assertSame after the marker, distinct codes {26.11, 26.12, 26.8, 42.3}, all plain `(major.sub)`.
The conclusion (all plain form, no array/msg form) holds; the count does not.

**F4 (verified good): marker scope and second-marker semantics.**
Independently confirmed: no method in base/expressions has two expectSyntax markers; each of the 184 converted rows is itself the raising statement (distribution is exactly 1 assertSame after each marker), so the conversion's per-row "expr must raise major.sub" is faithful to what the oracle tests.
SPECIAL (67 markers) and PRECEDENCE (139 markers) contribute 0 conversions because their markers precede raising assignments, not assertSame calls; the pinned per-group counts (60/6/118) match my independent scan.

**F5 (Important): `collect_all`'s doc comment claims a recount that does not exist.**
`rexx-exec/tests/assertions.rs:156`: "recomputed independently below in `assertions_differential` from `count_assert_same`, so a future regression in either crate's counting is still caught from this side too."
`count_assert_same` exists only in `rexx-extract/src/bin/rexx-extract-assertions.rs`; `assertions_differential` recounts nothing, it only asserts the row set is non-empty.
The safety net does exist, but on the extractor side (pinned-count tests); the harness-side claim of an independent recount is false, which is precisely the claimed-check-that-does-not-check shape this criterion's history warns about.

**F6 (Important): the 2 rows attributed to 4b are not unblocked by 4b.**
The 2 "a function call" rows are `Literals::test_string_range`; their first blocked construct is `xrange()` in the row's own prelude (`all = xrange()`), not `self~runDynamicSource` as the report narrates (that is a message send, and `xrange`'s implementation is a builtin, 4c's).
Both rows also contain `~changeStr(...)` sends in the prelude and `self~runDynamicSource(...)`/`self~q(...)` in the expected text, so after 4b lands `Call` they will block again on "a message send" (Phase 5).
The criterion's wording is "listed with the sub-phase that unblocks it"; the harness lists the sub-phase owning the FIRST construct hit, which is honest and self-updating but not the unblocking phase for these 2 rows.
The 33 "a message send" -> Phase 5 attributions are consistent with the split table (`Message` is Phase 5's).

**F7 (Important, needs a ruling): the STRICT gate as implemented cannot pass within 4a.**
`REXX_ASSERTIONS_GATE=1` fails on any non-Pass row, including the 35 RUNTIME-BLOCKED rows, 33 of which need Phase 5 and 2 of which need 4b + 4c + Phase 5 (F6).
Criterion 2's own wording ("a row whose operands need 4b or 4c is listed with the sub-phase that unblocks it") reads as an allowance: listed rows do not block the gate.
Two further wrinkles: the criterion contemplates only 4b/4c-blocked rows, and all 35 actual blockers need Phase 5, a case the criterion never names; and if STRICT were changed to exempt attributed rows, the attribution itself becomes load-bearing and must be policed (a misattributed 4a gap would then pass the gate).
The current over-strict choice is the safe direction and cannot produce a false pass; it just means criterion 2 closes at Phase 5, not 4a, unless the lead rules the 35 listed rows admissible.
The 15b report frames this honestly ("these 35 are the only thing between this table and a clean pass") but does not surface that this makes the gate unsatisfiable within 4a's scope.

**F8 (Minor, against the brief/spec, not 15b): the 56/332 silent/loud split is wrong; it is 106/282.**
Measured by running all 388 CONCATENATION rows with their preludes cleared: 106 pass anyway (silent), 282 fail loudly, 56 is only the strict `==`/`\==` subset -- 50 non-strict rows (e.g. `(a=a) (b=a) ...` expecting `1 0 0 0 0 0 0`) are also satisfied by unset single-character variables.
Consequence for the harness: there is no harness-side prelude falsification test; prelude wiring in `program_for` is guarded by the extractor's own tests plus the 282 rows that fail loudly if the prelude is dropped.
That guard is real (282 loud failures is not subtle), so this is calibration of the spec's number, not a gate hazard.

## Not reached

* The concurrent `rexx-exec/src/` work, `NOT_IMPLEMENTED_EXIT`'s internals, and the `Raised` visibility claim were taken as given.
* The STRICT-mode run itself was not repeated (verified upstream per the dispatch).
* The 33 message-send rows were verified by mechanism and report listing, not row by row against their method bodies.
* `emit_uncaptured`'s equivalence to `corpus.rs` was inspected, not re-measured.
* 804-test workspace run, clippy, fmt: verified upstream, not repeated.

