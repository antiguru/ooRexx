# Phase 9c gate record

Spec: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md` (D4, D5, D6, criterion 4).
Plan: `docs/superpowers/plans/2026-10-15-phase-9c.md`. Ledger, reviews and rulings:
`.superpowers/sdd/2026-10-15-phase-9c/` (`final-review.md`, `final-rereview.md`).

## Close run

At `25197002a`, by `docs/superpowers/records/2026-10-15-phase-9c/close/run.sh`, on a fresh target
directory:

```sh
cd rust
cargo fmt --all --check
CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings
CARGO_INCREMENTAL=0 memcap 16G cargo test -j 4 --workspace --release --no-run
REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../close/suite/table.tsv CARGO_INCREMENTAL=0 \
    memcap 8G cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite -- --nocapture
```

Status file (`close/status.txt`): start 2026-10-17T19:23:01Z, fmt exit 0, clippy exit 0, build exit
0, workspace exit 0 at 19:55:57Z, suite exit 101 at 20:00:35Z, `finished`. The workspace log has
160 test binaries, 3381 tests passed and 0 failed. The suite's exit is its whole-suite assertion:
38 of 402 groups are not green, where the 9b1 close had 46.

The build step is separate because the first close run, at `bf5224d2b`, was OOM-killed at the 8G
cap while compiling on a fresh target directory (`memcap: OOM-killed at the 8G cap (peak 8.0G)`,
no test result). The second run, at `48fb59b51`, failed one test.
`environment_seam`'s `directory_lookup_passes_through_exactly_one_chokepoint` found a third
`env_seam::admit` call, added by Task 5's `.SYSCARGS` removal. `25197002a` routes the write and the
removal through one helper.

**Close command exit status:** workspace 0. That run includes `collect_stress`,
`representation_refusals` and the corpus differential under `REXX_CORPUS_GATE=1`.

## Criterion 4

**`rexx_command` and `rexxc` green under the harness.** Met. `close/suite/verdicts.tsv`:
`utilities/rexx/rexx_command` green and `utilities/rexxc/rexxc` green. In `close/suite/tests.tsv`,
every test of both groups passes except `rexx_command`'s `TEST_PASSINGARGS7`. That test's row says
`Deviation(40)`: it merges a child's stdout and stderr into one array, and Deviation 40 licenses the
order.

**`METHOD`, `Method` and `Routine`.** Met. `API/oo/METHOD`, `base/class/Method` and
`base/class/Routine` are green. `TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`, and both
`TEST_NEW_ARRAY_FROM_FILE` and both `TEST_NEW_FILE_COMPILED`, pass.

**The `REGISTERED` exemption's witness.** Met. `close/workspace.log`, `tests/rexxc.rs`:
`registered_is_refused_with_the_rxapi_reason ... ok`. The exemption covers only `install` rows
whose program names `REGISTERED`, ignoring case. The agreement test also requires each exempt row
to be refused with `(Phase 10 Rxapi)`.

**The TSV agreement and the D6 coverage test.** Met. From `close/workspace.log`:
* `tests/rexxc.rs`: `rexxc_agrees_with_every_recorded_parse_error_row ... ok`,
  `the_command_line_answers_as_the_oracles_does ... ok` and
  `the_oracle_refuses_this_crates_images_as_another_versions ... ok`.
* `rexx-parse/tests/errors.rs`:
  `every_message_the_parser_names_is_raised_by_a_corpus_row_or_recorded_as_unreached ... ok`,
  `every_secondary_message_matches_the_oracle ... ok` and
  `every_unreached_reason_still_holds ... ok`.

**`tests/compiled.rs`.** Met. `each_route_runs_a_restored_program_as_the_oracles_does ... ok` and
`a_foreign_or_damaged_image_is_refused ... ok`.

**Performance: every program within +0.5% of the Phase 9 pin.** Met. See below.

**No test regression against the 9b1 close.** Met. A join of `close/suite/tests.tsv` with
`docs/superpowers/records/2026-10-14-phase-9b1/close/suite/tests.tsv` on group, test and row adds or
removes no test. It changes 50 rows, every one from failed to passed:
* `utilities/rexx/rexx_command`: 21;
* `base/expressions/Literals`: 12;
* `utilities/rexxc/rexxc`: 8;
* `base/class/Method` and `base/class/Routine`: `TEST_NEW_ARRAY_FROM_FILE` and
  `TEST_NEW_FILE_COMPILED` in each;
* `base/directives/OPTIONS`: `TEST_ALL_SYNTAX_INVALID` and `TEST_ALL_SYNTAX_MISSING`;
* one each for `API/oo/METHOD` `TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`, `base/bif/ARG`
  `TEST_EXTERNAL_CALLS` and `base/source.file/incorrectCharacters` `TEST_INCORRECT_CHARACTERS`.

**`rexx-run` is gone.** Met. `git grep -n 'rexx-run' -- rust` at `25197002a` prints only lines of
`rust/bench-baselines/PINNED.md`, which names historical pin binaries.

## Callgrind

Met. `close/callgrind.txt`, run from the repo's `rust/`:

```sh
bench-programs/callgrind.sh -j 2 -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c/close \
    pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-9c-48fb59b51
```

Exit 0, every spread under 0.01%. The pin's sha256 begins `c4b4e8c4ce7f`. The head binary is
`cargo build --release --bin rexx` at `48fb59b51`, and its sha256 begins `5a64719dd8c3`.

**Whole-process change against the pin.**
* Largest rises: `churn` +0.165%, `sayloop` +0.145%, `decloop` +0.062% and `startup` +0.018%.
* Falls: `extcall` -13.1%, `intcall` -10.1%, `pkglocal` -6.9%, `fibcall` and `fibfunc` -3.4%,
  `dispatchclass` -3.2%, `interpret` -2.2% and `dirread` -2.2%.
* The close commit `25197002a` changes only the `.SYSCARGS` write path. `startup`, `sayloop` and
  `interpret` were re-run there (head sha256 begins `d5c6aaef19cb`): +0.016%, +0.144% and -2.186%.

**Earlier tables.**
* Task 5's A/B, with the front end's share, is in `docs/superpowers/records/2026-10-15-phase-9c/ab/`
  (`table.txt`, `head-minus-base.tsv`). The front end costs a fixed 28 to 30 thousand Ir per run.
* Task 7's `sayloop` table is `t7/callgrind-e95e41416.txt`: `sayloop` +0.139% from the bound check.

## Reviews

The final whole-branch review (`final-review.md`) approved with fixes. It ran:
* `rexxc -s` on every `parse-errors.tsv` row;
* `rexx` on the error rows;
* the translation rows through `INTERPRET` and through each side's own `.rxo`;
* about 40 targeted shapes.

It raised two Important 9c defects: a restored program dropped the `>I>`/`<I<` trace lines, and
`-o`/`-od` reached a restored program. `4a27fae3b` fixes both, with oracle witnesses in
`tests/compiled.rs`. `320eff355` seals `ParseError`'s `code` and `sub` behind accessors. The scoped
re-review (`final-rereview.md`) approved with none open.

## Deviations

These were added this phase, in `docs/superpowers/plans/phase-4-exclusions.txt`. Moritz ruled each
one.
* 38: DO OVER/WITH subkeys.
* 39: a label spelled THEN.
* 40: a redirected standard output is buffered.
* 41: an encoded image whose decoded bytes lack the tag answers 3.903.

## Queued items

**Closed.**
* `2026-10-11-output-lost-on-kill.md` (Task 7).
* `2026-10-10-translation-error-inserts.md` (Tasks 3 and 4).
* The inserts half of `2026-10-13-error-position-ampersand.md` (Tasks 2 and 4). Its
  `instanceMethod` item stays open.

**Carried, filed during 9c.**
* `2026-10-15-routines-table-silent-none.md`.
* `2026-10-16-stream-command-open-missing-file.md`.
* `2026-10-16-bignum-remainder-speed.md`.
* `2026-10-17-oracle-not-op-forwarding-bug-2117.md`.
* `2026-10-17-library-bind-before-options.md`.
* `2026-10-17-routine-call-frame-name.md`. A known-gap row sits after Deviation 41.
* `2026-10-17-newmethod-parent-package.md`.
* `2026-10-17-scanner-reports-later-clause-error.md`. It predates 9c.
* `2026-10-13-lossy-path-sites.md`. Task 5 made the program-path chain bytes, but none of the
  item's listed sites closes (`task-5-report.md`, "Shared paths").
