# Task 25 report

## Commits

- `69170e0e2` whole-group runner, first version
- `49a388186`, `500b69c30`, `c4dc6f135`, `c856def66`, `783d81a59`, `5314db73d`, `7c9871515`: the
  derived part, path masking, `skip` reading quoted and `?` names, the `DIFFERING` list keyed by
  refusal or summary, test selection by the derivation's `units`
- `dd578201c` corpus pinning report (`corpus.rs`), criterion-10 re-run, whole-group record
- `7532f17ba` pinning evidence, `alone.txt`, queue note (method elapsed clock)
- `fcf8e1470` `sort_by_key` for clippy
- `a79128b7d` S5 records for criteria 1, 9, 10 in `phase-6-gate.md` and `phase-6-pinning.md`; this report

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

## Fix round 1

Commits: `aefb2eec4` (keys, started markers, rest part, `rexx` on PATH, corpus asserts),
`9efe2b267` (`REST_LEFT_OUT`), `573076d31` (a refusal before any test ends the rest part),
`bf558da52` (`DIFFERING` regenerated, P48 rerun, `alone.sh`/`alone.txt`, clock probes, queue note),
and `cd3f30781` (records; it also removes the corpus asserts that a
replace had also put into `sharing_fraction_over_the_corpus`, which broke `--features sharing`).

- C1: `pinning_report_over_the_corpus` asserts no unfinished run, no inverted wait, no pushed frame.
  Red in a scratch copy with the reviewer's two programs added to `phase-6.txt`: both modes
  `programs that did not finish: ["lang/zz_hang.rex"]`; with the inverted one alone `programs with an
  inverted wait: ["lang/zz_inverted.rex"]`.
- C2, F2: a run's key is refusal or summary, rc, last test started, failing tests; `DIFFERING` keyed
  on it; `table.txt` carries every key.
- C3: `rexx` on PATH per side (oracle `build/bin`, ours a link to `CARGO_BIN_EXE_rexx-run`,
  `REXX_SWITCH_MODE=every` under every opportunity), in `run_oracle_within`/`run_crate_within`, so
  the per-test rows get it too. `bug2003_guard_when` now passes on all sides, 12 assertions.
- C4: a listed row that agrees is printed (`REPLY` whole every, this run).
- F1/P85: `rest` part; `started.txt`; Message row names TEST_REPLYWITH_NOT_ARRAY.
  `REST_LEFT_OUT`: Class TEST_SUBCLASSES_GC and Object TEST_UNINIT, TEST_UNINIT_CLASS, which alone
  grow to the 4 GB `ulimit -v` here (3.4 GB maxrss) and in process took the test binary to the 8G
  memcap (first whole run of this round: `memcap: OOM-killed at the 8G cap`).
- F3: clock probes a/b/d/e, 30 runs each side, `whole-groups/clock/`; queue item updated.
- F4-F9: record text as asked; F6: one run per mode stated, the review's 30-run tallies committed
  as `whole-groups/ours-30-runs.txt` with the PATH caveat; F7: per-test paragraph removed.

Checks at `bf558da52` (+ the sharing-only corpus.rs fix): whole-groups gate test `ok. 1 passed`,
768.74 s, no P48 rerun; `measured::` 18 passed, corpus pinning both modes ok; fmt 0; clippy
workspace, pinning, sharing 0 (sharing after the fix); `memcap 8G cargo test --workspace --release`
exit 0, 3042 passed, 0 failed, 4 ignored (built first with `--no-run` outside memcap).

Concerns: the whole-group test now takes about 13 minutes; a whole part that stops refusing before
TEST_SUBCLASSES_GC or TEST_UNINIT would run them in process and hit the memcap.

## Fix round 2

Base `ccc606fb3` (the lead's P86 change). Commits: `9cef9f27b` (N1 `counted_only` masks the
assertion count and compares with `agree`, N2 its four unit tests, N3 Method's reason and
`alone.sh`), `ac9754ed8` (`alone.txt` rerun with Method TEST_NEW_NO_ARGS, N5 sentences),
`a603bccfd` (N4: the whole-groups run at `ac9754ed8` and its record), and this report's commit.

- N2 mutants, each run of `whole_groups::`: `counted_only` without `seen.len() > 1` fails
  `one_oracle_outcome_is_not_counted_only`; the old key-based trigger fails
  `outcomes_differing_in_stderr_alone_are_not_counted_only`; file restored from a copy.
- N4: whole-groups at `ac9754ed8`: `test result: ok. 6 passed; 0 failed`, 773.19 s, no P48 rerun;
  printed as listed and agreeing: `REPLY` whole every and `REPLY` derived every (both under P86).
- Checks at `a603bccfd`: fmt 0; clippy workspace, pinning, sharing 0; `memcap 8G cargo test
  --workspace --release` (default target, built first with `--no-run`) exit 0, 3047 passed, 0 failed,
  4 ignored (sum of `test result` lines).
