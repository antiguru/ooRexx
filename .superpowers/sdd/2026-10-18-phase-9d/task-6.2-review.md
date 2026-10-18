# Task 6.2 review: integer representation under Deviation 44

## Spec compliance

Spec compliant. No crate source changed, 19 licence rows written, 19 lines left the 9d set.

## Checks run

* Replay method. Rebuilt the crate (HEAD source), re-derived the oracle log (26332 lines, differs from the committed log only in DATE and TIME arguments, which are wall-clock values) and the replay table with the committed script. Replay run on both sides: 20 tests, 38761 assertions, 0 failures, 0 errors, stdout identical apart from timing lines, stderr empty. Original group: oracle 0 failures, crate 19 failures after 1181 assertions.
* Fallback exposure. Instrumented copy that prints each use of the line-only fallback: only lines 379 (2 uses) and 391 (4 uses), identical counts on both sides. Every other call hit its exact `line|argument` key, so the crate reaches `isInteger` with the oracle's arguments at all other sites. The `dataType("W")` comparison stays live on the crate.
* Mutated tables, run on the crate (the oracle fails identically, as it replays its own table): `256|1000` whole=0 gives ISINT-WHOLE and an error in the power test; `139|5` and `391` answer 0 give 2 failures; `168|0` answer 0 gives 1 failure. The replay is live, exact keys and the line-only fallback both bind.
* Independent probe, outside `isInteger`, compared byte for byte on both sides (stdout and stderr identical): `+ - * % // ~modulo` and `NumberString % . ~trunc ~ceiling` over 12 integers (test_integer_addition, subtraction, multiplication, integerdivide, remainder, modulo, numberstring_*); `base ** power` for base -30..30, power 0..40 with the `> internalMaxNumber` comparison, and `.RexxInfo` max/min and +1/-1 (test_integer_power, test_integer_rexxinfo); `d2c~c2d`, `x2d`, `d2x` incl. negatives (test_string_c2d, x2d, test_integer_d2x). No difference.
* Heading indent (`9fdf5bd99`). `suite_rows::deviations` (tests/support/suite_rows.rs:372) ends at the first non-empty line without a leading space. Before the edit it stopped at line 2295 (the margin heading), so 42 to 46 were invisible. A re-implementation of the scan on the edited file reaches line 2381 (`EXPRKIND OWNERSHIP`) and sees 42, 43, 44, 45, 46. The edit adds only 5 leading spaces to two lines, wording unchanged. The heading line cannot parse as a deviation number (`KNOWN GAP...` is not digits). No other file names the heading.
* Rows. Same four-column shape as the neighbouring Deviation 40 and Flaky rows, sorted, 19 rows for the 19 failing tests (D2C passes and has none). `suite-9d-set.txt` holds no RexxInteger line. I did not re-run the Rust `suite` test; the report's pass is consistent with the scan above.

## Findings

* Minor: the committed evidence is about 40k lines (`replay.tsv`, `log-oracle.tsv`). `replay.tsv` is derivable from the log by the committed script; one of them would do.
* Minor: replay keys 379 and 391 fall back to the line alone, so those two sites cannot witness a wrong DATE or TIME value, only a wrong length class and whole-ness (the `dataType` check and the length branch still run). Acceptable, the Deviation is about the representation.
* Minor: `TEST_TREE_DIRECTORY` flakes on both sides with no row. The report notes it, and the controller has a queued item for it.
* No Critical or Important finding. No real divergence is hidden by the licence that the probes can reach.

## Assessment

**Task quality:** Approved. The replay is sound: exact-keyed except two wall-clock sites, guarded by a crate-side `dataType` check, shown live by mutation, and the non-`isInteger` arithmetic is independently identical.
