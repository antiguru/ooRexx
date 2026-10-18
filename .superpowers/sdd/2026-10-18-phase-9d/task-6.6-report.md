# Task 6.6 report: parser and directives

Clusters `label-in-if-select-branch`, `attribute-body-error-number`, `superclass-not-a-class`
and `constant-expression-novalue-option`. All 12 tests pass, and the whole suite changes in
exactly those 12 rows.

## Commits

* `ab941ce38` Phase 9d Task 6.6: parser and directives. The four fixes, their witnesses, the
  `refusal-sites.tsv` re-derivation, and the move of `lang/class_member_class_keyword_needs_class.rex`
  to a `parse-errors.tsv` row.
* `f568d48c0` Phase 9d Task 6.6: class search uncached, label peeling out of line. The perf
  follow-up, the 12 lines out of `suite-9d-set.txt`, and the queued item closed.
* The records commit that follows holds `docs/superpowers/records/2026-10-18-phase-9d/t6.6/` and
  this report.

## Root causes

### label-in-if-select-branch

`ClauseCursor::split_before` re-presents what follows a THEN, ELSE or OTHERWISE as a new clause,
but only `split_clauses` peeled a leading `name:` into a label clause. So `then lab: nop` reached
`parse_instruction` as an instruction starting `lab:`. It was read as a namespace-qualified call
(98.987), as `lab: do` with the DO lost (10.1 at its END), or accepted.

The oracle (`LanguageParser::translateBlock`, `LanguageParser.cpp:1224`-`1244`) finds the label
in the trimmed clause through `nextInstruction` and raises 47.3 or 47.4 by the control stack's
top, which the crate's `label_error` already mirrors. Its insert is not the label's name:
`resetPosition(1); nextReal()` reads the first token of the source clause, which after a trim is
still the IF, WHEN, THEN, ELSE or OTHERWISE keyword. Measured: `if 1 = 1 then lab: nop` is 47.3
`found "IF"`, `then lab: nop` on its own line is `found "THEN"`, and the error line is the clause's
first line (`if 1 = 1 then ,` / `lab: nop` reports line 2).

Fix: `split_before` takes a `Remainder`. `Remainder::Instruction` (after THEN, ELSE, OTHERWISE)
peels leading labels as `split_clauses` does, in the cold `peel_labels`. `Remainder::Then` (the
IF/WHEN condition split) does not, because `translateBlock` reads that THEN as a token. The
misplaced-label error now names `ClauseCursor::origin(byte)`'s first token and is reported at that
clause's start. For a label clause of its own, that is the label, so the existing 47.x rows hold.

Deviation 39 is untouched. The `if 1 = 1 then: nop` and `when 1 = 1 then: nop` rows stay 35.1 on
both sides, and `the_label_colon_rule_covers_exactly_the_two_corpus_rows_it_was_written_for` stays
green. A mutant passing `Remainder::Instruction` for the IF/WHEN split turns those two new rows
into 18.1/18.2, and that test fails (run below).

### attribute-body-error-number

The CLASS-keyword half of `checkDuplicateMethod` (`DirectiveParser.cpp:510`-`516`) ran at install
(`Interp::check_member_keys`), after the parser's body checks. The oracle runs it inside
`methodDirective` (`:822`, ahead of every shape) and `attributeDirective` (`:1664`, `:1725`,
`:1790`, after `getRetriever` and ahead of the body check). So `::attribute 'foo' class` with a
body was 99.937 here and 99.905 there.

Fix: `parse_directive` takes `in_class` (a `::CLASS` came before, the oracle's `activeClass`), and
`Dir::require_class` raises 99.905 at those two points. The install branch became unreachable and
is a `debug_assert!`; `Raised::class_keyword_needs_class` is deleted. `::constant k class` stays
accepted on both sides.

`lang/class_member_class_keyword_needs_class.rex` no longer parses, which
`every_corpus_lang_program_parses` (`rexx-parse/tests/program.rs`) forbids. A translation error's
witness is a TSV row, so it moved there verbatim (row 1217), and left `phase-5a.txt`, `coverage.rs`
and `sourceline_oracle/`. The comments in both lists lost the clause that named it.

### superclass-not-a-class

`Interp::directive_class` searched the package chain, then `.environment` alone, then the native
table. Its comment said `.local` is not in the search. `ClassResolver::lookup` calls
`PackageClass::findClass` (`PackageClass.cpp:1081`-`1159`), whose order after the package's own
tables is the REXX package's public classes, the package's local directory, `.local`, then
`.environment`. `.INPUT` is `.local`'s stream, so the oracle answers 99.949 where this crate found
nothing (98.909).

Fix: the rest of `findClass`'s order, with a non-class first hit raising 99.949. Measured beside
it: `.local~foo = .array` then a called file's `::class t subclass foo` is now Array on both sides
(98.909 before), `.local~array = 5` does not shadow Array, `.local~foo` over `.environment~foo`
wins, and INHERIT and METACLASS INPUT are 99.949 on both (98.909 and 98.908 before).

### constant-expression-novalue-option

`Interp::eval_constant_expression` evaluated in a directive activation built with default
settings. `ClassDirective::resolveConstants` (`ClassDirective.cpp:257`) runs the expression as a
method of the package, which starts from the package's `::OPTIONS`. Fix: the activation takes
`package_start`'s numeric and condition settings. Measured: NOVALUE ERROR, NOVALUE SYNTAX and ALL
SYNTAX now raise 98.986, and `(1/3)` under `::OPTIONS DIGITS 5` is `0.33333` (it was
`0.333333333`).

## Witnesses, red then green

Red is the parent `ebeab2d35`, built from `git archive` (all files touched, `Compiling rexx-exec`
asserted). Green is `f568d48c0`.

TSV rows in `rust/corpus/errors/parse-errors.tsv`, derived with `corpus/errors/derive.sh`. The
derivation changed only the `?` rows:

* 1200-1209, 47.3/47.4: THEN, ELSE and OTHERWISE on one line, `lab: do`, SELECT CASE, THEN on its
  own line, a literal label, a continuation, and `then lab:` at a clause end.
* 1210-1216, 99.905: the test's shape, GET with a bad body, `::method` with a bad body and with
  `delegate 5`, `external "LIBRARY x"`, after a `::ROUTINE`, and bare.
* 1217, 99.905: the moved corpus program.
* 1218-1221, controls that already agreed: `then:` after IF and after WHEN (35.1, the Deviation 39
  neighbours), `else: nop` (accepted), `otherwise: nop` (47.4 `found "OTHERWISE"`).

`cargo test --release -p rexx-parse --test errors` on the parent tree with this TSV exits 101.
Rows 1200-1217 each fail (`row N: expected ..., got ...`), recorded in `t6.6/witnesses/tsv-red.log`.
`every_sample_program_parses` also fails there because the archive has no `samples/`. On
`f568d48c0` it exits 0, 35 passed (`tsv-green.log`). `rexxc.rs` agrees with the TSV.

Corpus programs (`rust/corpus/phase-9.txt`, with their `sourceline_oracle/` files from that test's
oracle driver), `t6.6/witnesses/summary.tsv`:

| program | oracle | parent | t6.6 |
|---|---|---|---|
| `class_constant_expression_numeric_options` | rc 0 | rc 0, stdout differs | identical |
| `class_constant_expression_novalue_option` | rc 158, 98.986 | rc 215, 41.1 | identical |
| `class_subclass_local_not_a_class` | rc 157, 99.949 | rc 158, 98.909 | identical |

Probe sets in `t6.6/probes/{label,directive,constant,superclass}/`, each with `summary.tsv`
(oracle, parent and t6.6 rc, and whether stdout and stderr match), run by `t6.6/probes/probe.sh`.
Every probe matches the oracle except these, each named under Concerns:
`directive_10`, `directive_11`, `constant_c`, `superclass_e`, `superclass_g` and `superclass_l`.

## The tests

`suite-9d-set.txt` loses these 12 lines, and each passes in the whole-suite run:
`base/directives/ATTRIBUTE TESTMISPLACEDCLASSMETHOD`, `base/directives/CLASS
TEST_SUBCLASS_NO_CLASS_LOCAL`, `base/directives/CONSTANT TEST_EXPRESSION_NOVALUE_ERROR`, and
`base/keyword/LABEL` `TEST_LABEL_IF_1_2`, `_2_1`, `_2_3`, `_3_1`, `_3_5`, `TEST_SELECT_1_3`, `_1_5`,
`_2_3`, `_2_5`.

Queued item `2026-10-10-misplaced-class-attribute-error.md` is closed: its probe is byte-identical
to the oracle, rc 157 on both. No exclusion row or licence row is touched.

## Changed rows

Against `t6.5/r4/suite/tests.tsv`, by the command at the top of `t6.6/suite/changed-rows.txt`:
exactly the 12 tests above, each `failed` to `passed`. No other row changed, and the
`SysFileTree TEST_TREE_DIRECTORY` flake did not show. Suite: 402 groups, 24 red (28 at 6.5 r4).

## Perf

`t6.6/callgrind.txt`, `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn
sayloop" -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.6 pin=.../rexx-run head=.../t6.6/rexx-head`,
exit 0, 3 rounds. Own delta is this figure minus `t6.4/r1/callgrind.txt`, the last perf-checked
task (6.5 did not run one), so it also carries 6.5's changes. `t6.6/callgrind-ab.txt` measures
against the parent directly.

| program | vs pin | t6.4 r1 vs pin | own delta (ledger) | vs parent (A/B) |
|---|---|---|---|---|
| startup | +0.0436 | +0.0134 | +0.0302 | +0.0284 |
| interpret | -2.1475 | -2.1783 | +0.0308 | +0.0303 |
| parse | -0.0858 | -0.0879 | +0.0021 | +0.0021 |
| churn | +0.1321 | +0.1262 | +0.0059 | +0.0054 |
| sayloop | +0.0500 | +0.0298 | +0.0202 | +0.0187 |

Every own delta is under +0.05%, and every figure against the pin is under +0.5%.

`ab941ce38` alone was over: interpret +0.2261% whole process, +0.0290% with libc subtracted. The
same number of blocks was freed, and the excess was in `_int_malloc` and friends. Two causes,
each measured in `t6.6/callgrind-experiments.txt`:

* The REXX package step went through `rexx_package_class`, whose cache inserts allocate. Uncached
  (`exp2`), interpret falls to +0.0824%.
* With the REXX package answering first, the bootstrap no longer reached the directory lookup that
  built the environment model, so the model's allocations moved later and shifted the heap. The
  control `ctl` (exp2's other changes, the parent's `directive_class`) is +0.0209%. Building the
  model at the top of the directory steps (`exp4`, what `f568d48c0` commits) gives +0.0303%.

The comment at that call states the reason. Peeling labels in a cold function moved startup from
+0.0488% to +0.0417% (`exp3`). Error construction stays `#[cold]` and out of line:
`misplaced_label`, `Dir::error` and `peel_labels`.

## Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/target-9d-t6`, `CARGO_INCREMENTAL=0`, each exit status
read unpiped, on `f568d48c0` unless named:

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-parse -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 8G cargo test -j 4 --release -p rexx-parse`: exit 0, every binary ok (includes
  `errors`, `program`, `sourceline_oracle`).
* `memcap 8G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1152 passed.
* `memcap 8G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test owners --test
  closed_phases --test phase_ten_reasons --test representation_refusals --test rexxc --test
  coverage --test refusal_dispositions --test environment_seam --test package_requires`: exit 0.
* `refusal-sites.tsv` was re-derived (`REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites`) in
  `ab941ce38`. With `error.rs` line numbers normalised, the only change is the deleted
  `Raised class_keyword_needs_class` row. No `Loud` site changed.
* `memcap 8G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0, 2 passed.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1085 of 1085 matching, 29 passed, 1 ignored.
* `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/t6.6/suite/table.tsv
  memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`: exit 101 as
  expected, 402 groups, 24 red, 4m47s.
* Mutation, `Remainder::Then` to `Remainder::Instruction` at the IF/WHEN split: `--test errors`
  exit 101, `the_label_colon_rule_covers_exactly_the_two_corpus_rows_it_was_written_for` fails.
  Reverted by exact edit.
* `memcap 8G cargo test -j 4 --release -p rexx-exec --test directive_options`: exit 101 on this
  tree and on the parent tree alike. See Concerns.

## Concerns

* **Duplicate members still check at install.** The duplicate half of `checkDuplicateMethod`
  (99.902/99.931/99.932) stays in `check_member_keys`. So a duplicate followed by a body error or a
  later parse error reports the later error (`probes/directive/directive_10`: 99.937 for 99.931;
  `directive_11`: 35.929 for 99.902), and the crate's `rexxc` accepts a file whose only fault is a
  duplicate. Moving it means the parser tracking each class's keys, as the oracle's
  `addMethod`/`ClassDirective::checkDuplicateMethod` do. No test of the set depends on it.
* **`::OPTIONS TRACE` on a `::CONSTANT` expression.** The oracle traces the `::CONSTANT` method's
  entry, clause, results and exit (`probes/constant/constant_c`). This crate traces nothing there,
  before and after this task. The fix applies numeric and condition settings only, because the
  trace mode alone would print the expression's results without the method lines.
* **An install error in a called program names the caller's file** in `Error N running <file>`
  (`superclass_e`, `superclass_g`, and `superclass_l`, a plain 98.909 that differs on the parent
  too).
* **`directive_options` fails on the parent too.**
  `lostdigits_raises_98_972_where_the_oracle_does_and_nowhere_else` panics on its `signal-on-trap`
  row: both sides now print `trapped` / `end` at rc 0, and the row says it belongs deleted. That
  is outside this task's files.
* `ClassDirective`'s search keeps `Access::Direct`, as before. The oracle's `findClass` would
  consult a package security manager around `.local` and `.environment`. Not probed.
