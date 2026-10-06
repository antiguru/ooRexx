# Task 25 report

## Commits

- `69170e0e2` whole-group runner, first version
- `49a388186`, `500b69c30`, `c4dc6f135`, `c856def66`, `783d81a59`, `5314db73d`, `7c9871515`: the
  derived part, path masking, `skip` reading quoted and `?` names, the `DIFFERING` list keyed by
  refusal or summary, test selection by the derivation's `units`
- `dd578201c` corpus pinning report (`corpus.rs`), criterion-10 re-run, whole-group record
- `7532f17ba` pinning evidence, `alone.txt`, queue note (method elapsed clock)
- `fcf8e1470` `sort_by_key` for clippy
- the docs commit (hash in the lead's ledger; `git log -1` after this file)

## Step 1, criterion 1

`group_runs::whole_groups::each_group_of_the_derived_list_in_one_run_in_both_modes`, gate-only
(`REXX_CORPUS_GATE`), table via `REXX_WHOLE_GROUPS_TABLE`, descriptors via
`REXX_WHOLE_GROUPS_DUMP`. Each group file runs whole and with only its derived tests; oracle 5
runs, 30 when unsettled (P83); ours normal and every. At `7c9871515`: `test result: ok. 1 passed`,
674.84 s (`docs/superpowers/records/2026-10-01-phase-6-s2-s5/whole-groups/run.log`, `table.txt`).
Differences and reasons: `phase-6-gate.md` `### Criterion 1, the derived list's tests (Task 25)`.
Nothing fixed in the interpreter: every difference is a per-test row already recorded, a test
outside the derived list (other phases' refusals, STREAM and RAISE tests failing alone too), P41
(REPLY every) or P46 (MutexSemaphore every). New finding: a method does not start with a fresh
elapsed clock (TIME `_R` tests in one run); appended to the queued elapsed-clock item.

Runner changes in `support/group_runner.rs`: `run_oracle_within`, `run_crate_within` (deadline
parameters; the old functions call them with the old values); `skip` reads quoted names and
symbol names with `!`, `?`, `.`.

## Step 2, criterion 9

`measured::` rerun: 18 passed, `pinning-diff.py` shows no diff against the S3/S4 close tables.
Corpus report added (`corpus.rs` `pinning_report_over_the_corpus`, `--features pinning`), both
modes, 787 programs, none unfinished, no inverted or immovable wait. Record:
`phase-6-pinning.md` `## S5`, evidence `records/.../pinning-s5/`.

## Step 3, criterion 10

Re-run at this head, 30 runs per probe per side, same results as `s5-find-verify.md`:
`records/.../criterion-10/` (`run.sh`, probes, `summary.txt`).

## Checks

- `cargo fmt --all --check`: exit 0
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; `-p rexx-exec --features pinning`
  and `--features sharing`: exit 0 (scratch target, at `fcf8e1470`)
- `memcap 8G cargo test --workspace --release` (default target, built first with `--no-run`
  outside memcap: the build alone peaked over 8G and was OOM-killed under it): exit 0, 3042 passed,
  0 failed, 4 ignored (sum of `test result` lines), at `fcf8e1470`

## Concerns

- The whole-group test adds about 11 minutes to each gated test run (TIME's 30 oracle runs at about
  94 s each, 5 at once, dominate).
- Commits from `dd578201c` on carry the session line this session's system context gives
  (`session_01GGimEAe3NWM7Rh22JoLorg`) rather than the brief's.
