# Task 4a report: one truth judgment

Status: DONE_WITH_CONCERNS (emptyloop +0.6478% instructions, over the +0.5% budget; see Concerns).

Commits: `d27a9d441` (behaviour, witnesses, datadriven table, Deviation 25), `08876442c`
(`refusal-sites.tsv` re-derived), and the commit carrying this report and the gate record section.

## Step 1 table

In `docs/superpowers/plans/phase-6-1-gate.md` `## Task 4a`: the oracle's 256 cells, this crate's at
`d27a9d441` and at the base. At `d27a9d441` every cell outside DO TO and BY is byte-identical to the
oracle on stdout, stderr and status (224 cells); the 26 DO TO/BY cells that differ are Deviation
25's. Probe generator, runner and tabulator: `/tmp/claude-1000/p61/t4a/gen/{gen.py,run.sh,table.py}`.

## Changes

- `rust/crates/rexx-exec/src/eval.rs:1424` `logical_value` now private; `:1437`
  `truth_without_conversion` (the logical constants and small ints 0/1); `:1460` `Interp::truth`
  and `:1472` `truth_of_string_value` (`required_string_value`, then exactly 0 or 1, else the
  caller's raiser). Prefix `\` `:673`, `&`/`|`/`&&` `:1160-1161`, list elements `:1347`.
- `run.rs:3636-3687` `condition_value`: traces as before, debug-asserts a `checked` value is a
  logical constant, judges with `truth`; `condition_holds` deleted. `run.rs:3693` `test_case_when`
  sends `==` to the CASE value (`apply_binary(StrictEqual, case, value)`), traces the value and the
  answer, judges with `truth` and 34.905.
- `activation.rs:378` `current_case: Option<ObjRef>` (was the case text), rooted in
  `object_roots`; size assertion 480 -> 472. `run/select.rs` `open_select_case`/`scan_when` take
  the object; `run/interpret.rs` saves and roots it around a debug line; `ir/drive.rs` `WhenTest`
  passes the register's object.
- `run/loops.rs`: `loop_truth` deleted; BY sign `:963` and TO test `:2756` call `truth` (34.901);
  DO WITH `AVAILABLE` `:2824` calls `truth` with 34.906 (was `string_value_text`, so an array
  answered "an Array" and a user `STRING` was not sent).
- `run/raised.rs:64,70` `raised_when_case_not_logical` (34.905), `raised_available_not_logical`
  (34.906).
- `dispatch/collection.rs:216` `same_item` raises 34.901 (was `unwrap_or(false)`).
- `dispatch/string.rs:1254` `String~"?"`, `security.rs:122`, `dispatch/library.rs:1499` (native
  logical: `truth` on the converted string, failure mapped to the boundary's `Err(text)`).
- `ir/drive.rs:1477` IF/WHEN quick path and `:3915` `register_holds` use
  `truth_without_conversion`, the fast half of the one function; `eval/tests.rs`
  `the_unconverted_truth_agrees_with_the_string_value` checks it against the slow half.
- Prose: `ir.rs:134`, `ir/compile.rs:495,588`, `ir/drive.rs:3561` say "value" for the case.
  `phase-4-exclusions.txt` Deviation 25 drops the sentence saying neither the DO test nor WHILE
  sends a user STRING.
- Tests: `tests/truth.rs` and `tests/truth/values` (datadriven; one case per value, one line per
  context; asserts every judging context agrees on true/false/error and each context raises one
  sub-number). Corpus `lang/truth_user_string.rex`, `select_case_object.rex` (with `trace r`),
  `select_case_not_logical.rex`, `collection_equal_answer.rex`, `collection_equal_not_logical.rex`,
  `do_with_available_string.rex`, in `phase-6-1.txt`, each with a SOURCELINE expectation captured
  by the module-comment driver from a scratch copy. Each matches the oracle on all three
  descriptors and differs at the base; stdout read for each.

## Sites routed

Re-derived at `d27a9d441` with the survey's patterns plus `truth(`: every truth decision in
`rexx-exec/src` goes through `truth` or `truth_without_conversion`. Left alone, not truth
judgments: `semaphores.rs:272` (absent answer, per brief); `dispatch/time_support.rs:250`
`is_cancelled` (the oracle tests identity with `.true`); `builtin/stream.rs:154` (`LINES` nonzero).

## Mutation check

Recorded in the gate record: `condition_value` judging `to_text` reddens the table on `.s1~new`;
`same_item` reading `to_text == "1"` reddens it on `'banana'`. Both restored from copies.

## Performance

Gate record. Callgrind against `24394ca34`: rexxcps +0.1847%, emptyloop +0.6478%, decloop -0.0833%,
dispatch +0.1943%. Wall clock, 5 interleaved runs: all inside ±4% except dispatch -8.54% under load
2.7.

## Commands run

- `cargo fmt --all --check`: exit 0.
- `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings`: exit 0 (at both commits).
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast`: exit 101 at `d27a9d441`
  (`refusal_sites` only), exit 0 at `08876442c`.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
  ir_recorded_oracle`: exit 0 (29 passed 1 ignored; 21 passed).
- `memcap 8G cargo test -j 4 -p rexx-exec --test truth`: 1 passed.
- callgrind and wallclock commands as in the gate record, both exit 0.
- `whole_groups`: not run; no whole-group expectation line changed.

## Concerns

1. **emptyloop +0.6478%**, over budget, on a program that makes no truth judgment. 2 instructions
   a pass in `ops_loop_steady` from register allocation. Four variants (quick path written inline,
   `Activation` padded to 480, `open_select_case` out of line, `register_holds` as a `match`) each
   left it at exactly +0.6478%. Not a pads-controlled band. Needs a ruling: accept as codegen
   perturbation, or more rounds.
2. `whole_groups` was not run. Behaviour changes that could move a group's outcome: SELECT CASE now
   sends `==` to an object CASE value, collections raise on a non-logical `==` answer, a user
   `STRING` is sent where a logical value is required.
3. `test_case_when` now uses `apply_binary` for string CASE values too, rather than a text fast
   path; the oracle sends `==` there too, and the table and corpus agree, but it is a slower path
   for SELECT CASE on strings (not in the perf programs).
4. The first wall-clock run (`$P/wall`) used a binary I had rebuilt with a perf variant into the
   head target directory; it is discarded and the recorded run (`$P/wall2`) uses a fresh build of
   `d27a9d441` (`target-head2`).
