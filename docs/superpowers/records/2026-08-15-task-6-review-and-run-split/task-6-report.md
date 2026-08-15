# Task 6 report: move `run.rs`'s test module into `run/tests.rs`

Commit `4e71fe0d316e1ad1da673e1b511aa83f3de9ffbc`, "Move run.rs's test module into
run/tests.rs". Two paths staged by name: `rust/crates/rexx-exec/src/run.rs` (modified) and
`rust/crates/rexx-exec/src/run/tests.rs` (added). The tree the work started from was
`84ffd8aad`, not the `56d9d1c86` the plan's line numbers were taken at.

## 1. The scout's figures, verified

The scout's report was written against a working tree it says was live under another
session, so its own numbers are a snapshot. What the controller measured at the tree I found
is what I checked against.

| claim | source | measured at `84ffd8aad` | verdict |
|---|---|---|---|
| `run.rs` line count | scout: 17,381 | 17,431 | scout differs by 50, controller's figure confirmed |
| `#[cfg(test)]` attribute line | controller: 9720 | 9720 | confirmed |
| test module extent | scout: 9,668-17,381 (7,714 lines) | 9,720-17,431 (7,712 lines) | shifted by the same 50 lines; extent differs by 2 |
| `impl Interp` method count | scout: 95 (41 `pub(crate)`, 54 private) | 95 (41 `pub(crate)`, 54 private) | confirmed exactly |
| `ir/drive.rs` precedent | controller: `mod tests;` at `:1909`, `ir/drive/` exists | `mod tests;` at `:1909`, `ir/drive/tests.rs` exists | confirmed |
| mutation-script rows naming `${RUN_RS}` | scout: 17 | 17 | confirmed |
| no mutation pattern resolves into the test region | scout: zero | zero | confirmed, see §5 |

The scout's line count is stale rather than wrong; every structural claim survived
re-measurement.

## 2. The no-private-methods claim

**Established two ways, and the second is the one that decides the task.**

*Directly.* I extracted the 54 private and 41 `pub(crate)` method names from `impl Interp`
(lines 957-8585) and searched the test region for each. Fifteen private names appear
there -- `run_bounded`, `step`, `run_fragment`, `eval_condition` and the rest -- but every
single occurrence is inside a doc comment or a line comment. Filtering to non-comment lines
leaves zero hits, the only survivors being the English word "step" in two `expect("one step
fits")` strings. The test module's real calls on `Interp` are `new`, `run_activation`,
`chunk_node_at`, `exit_code_for`, `set_trace_mode`, `result_text`, `intermediate_text`,
`activation`, `to_text`, `text`, `trace_mode`, `slot_of`, `plan_for` and
`next_activation_id`. So the scout's claim holds as stated: **zero private `impl Interp`
methods are called.**

*Structurally, and this is stronger.* The claim did not need to hold. Rust privacy is
per-module and descends: an item private to `crate::run` is visible in `crate::run` and in
**every descendant module**, and `mod tests;` in a file makes `crate::run::tests` exactly
the same descendant that `mod tests { ... }` inline does. Inline and file-backed child
modules are indistinguishable to the privacy checker. **No visibility change is possible
from this move, whatever the test module calls.** That is why the scout's conclusion is safe
even though its premise was doing no work.

*What I would have done had the direct check found a real private call.* Nothing different,
and I would have said so rather than stopping: the brief's stop condition is "any item needs
`pub(super)` or wider", and the structural argument shows no item can. The brief's own
verification -- the build -- is the arbiter, and it is green. I did not widen anything, and
the diff contains no visibility keyword at all.

## 3. Per-binary test counts, before and after

Both runs are `cargo test --release --workspace` against the same fresh
`CARGO_TARGET_DIR` under the session scratchpad, under `memcap 32G`. Exit 0 both times.

A first attempt under `memcap 8G` was OOM-killed at exit 137 while compiling `rexx-exec` in
release -- worth knowing for anyone else capping a cold release build of this crate.

Cargo prints 83 result blocks against 82 `Running`/`Doc-tests` labels, because
`Doc-tests rexx_exec` emits two: `run.rs - run::Interp::run_activation (line 1073)` and the
`compile fail` one at `(line 1093)`. Both cite lines in the production region, which did not
move, so both names are unchanged. The pairing below is taken from a single merged-stream
capture so label and block cannot drift apart.

**All 83 blocks match row for row. 1509 passed and 4 ignored before, 1509 passed and 4
ignored after, zero failed either side.**

| # | crate :: target | before passed | after passed | before ignored | after ignored |
|---|---|---|---|---|---|
| 1 | `rexx_bench :: unittests src/lib.rs` | 30 | 30 | 0 | 0 |
| 2 | `rexx_arms :: unittests src/bin/rexx-arms.rs` | 0 | 0 | 0 | 0 |
| 3 | `rexx_bench_band :: unittests src/bin/rexx-bench-band.rs` | 10 | 10 | 0 | 0 |
| 4 | `rexx_bench_suite :: unittests src/bin/rexx-bench-suite.rs` | 10 | 10 | 0 | 0 |
| 5 | `rexx_time :: unittests src/bin/rexx-time.rs` | 0 | 0 | 0 | 0 |
| 6 | `rexx_core :: unittests src/lib.rs` | 6 | 6 | 0 | 0 |
| 7 | `behaviour :: tests/behaviour.rs` | 6 | 6 | 0 | 0 |
| 8 | `collect :: tests/collect.rs` | 14 | 14 | 0 | 0 |
| 9 | `handle :: tests/handle.rs` | 9 | 9 | 0 | 0 |
| 10 | `heap :: tests/heap.rs` | 4 | 4 | 0 | 0 |
| 11 | `roots :: tests/roots.rs` | 8 | 8 | 0 | 0 |
| 12 | `trace :: tests/trace.rs` | 3 | 3 | 0 | 0 |
| 13 | `uninit :: tests/uninit.rs` | 4 | 4 | 0 | 0 |
| 14 | `rexx_exec :: unittests src/lib.rs` | 629 | 629 | 0 | 0 |
| 15 | `rexx_run :: unittests src/bin/rexx-run.rs` | 2 | 2 | 0 | 0 |
| 16 | `assertions :: tests/assertions.rs` | 5 | 5 | 0 | 0 |
| 17 | `bif_assertions :: tests/bif_assertions.rs` | 5 | 5 | 0 | 0 |
| 18 | `builtin_status :: tests/builtin_status.rs` | 13 | 13 | 0 | 0 |
| 19 | `collect_policy :: tests/collect_policy.rs` | 2 | 2 | 0 | 0 |
| 20 | `collect_stress :: tests/collect_stress.rs` | 6 | 6 | 0 | 0 |
| 21 | `corpus :: tests/corpus.rs` | 10 | 10 | 1 | 1 |
| 22 | `coverage :: tests/coverage.rs` | 12 | 12 | 0 | 0 |
| 23 | `input_oracle :: tests/input_oracle.rs` | 9 | 9 | 0 | 0 |
| 24 | `ir_dual :: tests/ir_dual.rs` | 9 | 9 | 0 | 0 |
| 25 | `keyword_assertions :: tests/keyword_assertions.rs` | 7 | 7 | 0 | 0 |
| 26 | `loud :: tests/loud.rs` | 8 | 8 | 0 | 0 |
| 27 | `owners :: tests/owners.rs` | 5 | 5 | 0 | 0 |
| 28 | `parse_version_oracle :: tests/parse_version_oracle.rs` | 8 | 8 | 0 | 0 |
| 29 | `spike :: tests/spike.rs` | 8 | 8 | 0 | 0 |
| 30 | `state_builtin_oracle :: tests/state_builtin_oracle.rs` | 9 | 9 | 0 | 0 |
| 31 | `trace_indent :: tests/trace_indent.rs` | 11 | 11 | 0 | 0 |
| 32 | `trace_oracle :: tests/trace_oracle.rs` | 25 | 25 | 0 | 0 |
| 33 | `rexx_extract :: unittests src/lib.rs` | 0 | 0 | 0 | 0 |
| 34 | `rexx_extract :: unittests src/bin/rexx-extract.rs` | 0 | 0 | 0 | 0 |
| 35 | `rexx_extract_assertions :: unittests src/bin/rexx-extract-assertions.rs` | 0 | 0 | 0 | 0 |
| 36 | `extract :: tests/extract.rs` | 3 | 3 | 0 | 0 |
| 37 | `extract_assertions :: tests/extract_assertions.rs` | 19 | 19 | 0 | 0 |
| 38 | `extract_bif :: tests/extract_bif.rs` | 10 | 10 | 0 | 0 |
| 39 | `extract_keyword :: tests/extract_keyword.rs` | 22 | 22 | 0 | 0 |
| 40 | `rexx_inventory :: unittests src/lib.rs` | 0 | 0 | 0 | 0 |
| 41 | `builtins :: tests/builtins.rs` | 2 | 2 | 0 | 0 |
| 42 | `errors :: tests/errors.rs` | 5 | 5 | 0 | 0 |
| 43 | `oracle_agreement :: tests/oracle_agreement.rs` | 1 | 1 | 0 | 0 |
| 44 | `rexx_num :: unittests src/lib.rs` | 10 | 10 | 0 | 0 |
| 45 | `addsub :: unittests src/bin/addsub.rs` | 0 | 0 | 0 | 0 |
| 46 | `canon :: unittests src/bin/canon.rs` | 0 | 0 | 0 | 0 |
| 47 | `fmt_check :: unittests src/bin/fmt-check.rs` | 0 | 0 | 0 | 0 |
| 48 | `gen_cases :: unittests src/bin/gen-cases.rs` | 0 | 0 | 0 | 0 |
| 49 | `muldiv :: unittests src/bin/muldiv.rs` | 0 | 0 | 0 | 0 |
| 50 | `addsub :: tests/addsub.rs` | 7 | 7 | 0 | 0 |
| 51 | `compare :: tests/compare.rs` | 15 | 15 | 0 | 0 |
| 52 | `errors :: tests/errors.rs` | 17 | 17 | 0 | 0 |
| 53 | `format :: tests/format.rs` | 49 | 49 | 3 | 3 |
| 54 | `muldiv :: tests/muldiv.rs` | 11 | 11 | 0 | 0 |
| 55 | `parse :: tests/parse.rs` | 12 | 12 | 0 | 0 |
| 56 | `pow :: tests/pow.rs` | 9 | 9 | 0 | 0 |
| 57 | `rendered :: tests/rendered.rs` | 2 | 2 | 0 | 0 |
| 58 | `settings :: tests/settings.rs` | 17 | 17 | 0 | 0 |
| 59 | `whole :: tests/whole.rs` | 8 | 8 | 0 | 0 |
| 60 | `rexx_oracle :: unittests src/lib.rs` | 0 | 0 | 0 | 0 |
| 61 | `rexx_diff :: unittests src/bin/rexx-diff.rs` | 0 | 0 | 0 | 0 |
| 62 | `normalize :: tests/normalize.rs` | 2 | 2 | 0 | 0 |
| 63 | `rexx_parse :: unittests src/lib.rs` | 255 | 255 | 0 | 0 |
| 64 | `scan_check :: unittests src/bin/scan-check.rs` | 0 | 0 | 0 | 0 |
| 65 | `deep :: tests/deep.rs` | 6 | 6 | 0 | 0 |
| 66 | `errors :: tests/errors.rs` | 32 | 32 | 0 | 0 |
| 67 | `program :: tests/program.rs` | 23 | 23 | 0 | 0 |
| 68 | `samples :: tests/samples.rs` | 1 | 1 | 0 | 0 |
| 69 | `scanner :: tests/scanner.rs` | 35 | 35 | 0 | 0 |
| 70 | `sourceline :: tests/sourceline.rs` | 25 | 25 | 0 | 0 |
| 71 | `sourceline_oracle :: tests/sourceline_oracle.rs` | 1 | 1 | 0 | 0 |
| 72 | `tiling :: tests/tiling.rs` | 11 | 11 | 0 | 0 |
| 73 | `tokens :: tests/tokens.rs` | 9 | 9 | 0 | 0 |
| 74 | `variants :: tests/variants.rs` | 1 | 1 | 0 | 0 |
| 75 | `Doc-tests rexx_bench` | 0 | 0 | 0 | 0 |
| 76 | `Doc-tests rexx_core` | 0 | 0 | 0 | 0 |
| 77 | `Doc-tests rexx_exec` | 1 | 1 | 0 | 0 |
| 78 | `(second doc-test block of rexx_exec: the compile_fail one)` | 1 | 1 | 0 | 0 |
| 79 | `Doc-tests rexx_extract` | 0 | 0 | 0 | 0 |
| 80 | `Doc-tests rexx_inventory` | 0 | 0 | 0 | 0 |
| 81 | `Doc-tests rexx_num` | 0 | 0 | 0 | 0 |
| 82 | `Doc-tests rexx_oracle` | 0 | 0 | 0 | 0 |
| 83 | `Doc-tests rexx_parse` | 0 | 0 | 0 | 0 |
| | **total** | **1509** | **1509** | **4** | **4** |

The row that matters is `rexx_exec :: unittests src/lib.rs`, which carries the moved module:
629 before, 629 after. A module that had silently stopped being compiled would show there.

## 4. Gates

Each run unpiped from `rust/`, exit status read on its own.

| gate | exit | note |
|---|---|---|
| `cargo fmt --all --check` | 1, then 0 | see below |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | from a **clean** `CARGO_TARGET_DIR` (`rm -rf`'d first, 305 MB rebuilt from `proc-macro2` up, `Checking rexx-exec` present); zero `warning:`/`error:` lines |
| `cargo test --release --workspace` | 0 | table above |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | banner reads `mode: STRICT (the gate) -- REXX_CORPUS_GATE is set`, `55 of 55 matching` |
| `cargo doc --no-deps` | 0 | warnings below |

`cargo fmt --all --check` failed first, with fourteen diffs, all in `run/tests.rs` and all
the same shape: at four columns less indentation, a call that had been split across two
lines now fits in 100 columns, so rustfmt joins it. I ran `cargo fmt --all`; it touched only
`run/tests.rs` (`git status` showed no other tracked file modified) and the re-check exits 0.
Every gate above ran against the formatted content.

### That the reformatting changed nothing

The dedent is not byte-identical to the original, so I proved the equivalence mechanically
rather than asserting it. Re-indenting `run/tests.rs`'s body by four spaces, splicing it back
into `run.rs` at the position it came from, and running `rustfmt --edition 2024` on the
result reproduces the pre-move `run.rs` **byte for byte** (`diff` empty). The transform is
therefore exactly invertible: no assertion, name, string or test body differs.

Two supporting checks made before the move, which is what let me use a blind `sed` dedent at
all: every non-empty line of the module body starts with at least four spaces (only
`#[cfg(test)]`, `mod tests {` and the closing brace do not), and a scan of the region's
string literals found **zero** multi-line raw strings and **zero** plain strings containing a
hard newline. Every multi-line literal there uses the backslash-newline continuation, whose
leading whitespace Rust strips, so no literal's contents can shift with the indentation.

Comment-line count: 8,184 before, 8,194 after across the two files. The difference is exactly
the ten-line licence header that `run/tests.rs` carries, matching every other `.rs` file in
`rexx-exec` including `ir/drive/tests.rs`. No comment was dropped.

### `cargo doc --no-deps`

Exit 0, with warnings. **None is in `run.rs` or `run/tests.rs`, and none is a
`broken_intra_doc_links` in `rexx-exec`.**

* `rexx-extract` (lib doc), 6 warnings: public documentation links to private items
  `crate::keyword::blank_comments`, `Fixtures`, `segments`, `clause_boundary`,
  `rewrite_body`, `rewrite_line`.
* `rexx-bench` (lib doc), 3 warnings: unresolved link to `Figure::fmt`; public documentation
  for `arms` links to private item `Workload::rendered`; unresolved link to
  `Sitting::per_pass`.
* `rexx-bench` (bin "rexx-bench-suite" doc), 1 warning: unresolved link to
  `the_caveat_matches_the_committed_baseline`.
* `rexx-num` (lib doc), 1 warning: public documentation for `sub_code` links to private item
  `FormatError::sub`, at `parse.rs:107`.
* `rexx-exec` (lib doc), 2 warnings: `redundant explicit link target` at `lib.rs:378` and
  `lib.rs:379`, on `[`Engine::TreeWalker`](crate::Engine::TreeWalker)` and
  `[`Engine::Ir`](crate::Engine::Ir)`.

All of these are pre-existing and none can be a consequence of this commit: the only file
this commit changed the doc surface of is `run.rs`, whose lines 1-9719 are byte-identical to
before, and `run/tests.rs` is `#[cfg(test)]` and carries no documented public item. They are
reported because the brief asked for the warnings, not only the status. Whether they are
worth fixing is a separate question this task does not own.

## 5. Line counts

| file | lines |
|---|---|
| `crates/rexx-exec/src/run.rs` before | 17,431 |
| `crates/rexx-exec/src/run.rs` after | 9,721 |
| `crates/rexx-exec/src/run/tests.rs` after | 7,707 |

`run.rs` ends with the same two lines `ir/drive.rs` does, `#[cfg(test)]` then `mod tests;`.
`run/tests.rs` opens with the licence header and then `use super::*;`, with **no module doc,
and the crate's precedent points the other way.** `ir/drive.rs`'s *declaration* needs none,
which is what I copied, but the *file* it points at opens with `//!` at
`ir/drive/tests.rs:12`, as do `ir/golden_tests.rs:12` and `ir/corpus_shape_tests.rs:12`.
`run/tests.rs:12` is the only test file in `rexx-exec` that goes straight to `use super::*;`.
I left it that way because writing one would be adding test content, which this task forbids,
and because a module doc for this file is a judgement about what the module is for rather
than a fact the move establishes. It is a gap, not a justified omission.

`lib.rs`'s `mod run;` is untouched, and `run.rs` alongside a `run/` directory needs no
`mod.rs`.

### The mutation scripts need no edit

The three `scripts/mutate-4*.sh` mutate by fixed-string substitution, not by line number. I
extracted all 17 `run_one ... "${RUN_RS}"` from-patterns and counted their occurrences in the
pre-move `run.rs`, the post-move `run.rs` and `run/tests.rs`: **every count in the new
`run.rs` equals its count in the old one, and every count in `run/tests.rs` is zero.** No row
resolves into the moved region, so no gate instrument changes.

## 6. Files changed

* `rust/crates/rexx-exec/src/run.rs` -- the `#[cfg(test)] mod tests { ... }` block replaced
  by `mod tests;`. Lines 1-9719 byte-identical.
* `rust/crates/rexx-exec/src/run/tests.rs` -- new.

Nothing else. No production code, no visibility keyword, no assertion, no test name.

## 7. Self-review and found-and-not-fixed: five findings

Findings 1 and 2 are my own, found reviewing my own diff and fixed before reporting. Findings
3, 4 and 5 are things the move leaves stale that the task forbids touching; 4 and 5 were
added in fix round 1 from the review, and 4 is the class my first version of this section
missed.

1. **The commit message contained a false claim.** It read "The move is a four-space dedent
   of the module body and nothing else", which the fourteen rustfmt rejoins falsify. Fixed by
   amending before reporting: the message now states the dedent, the rejoin, and the
   round-trip that proves the pair equivalent.
2. **The commit was missing the branch's trailer.** Every one of the last fifty commits
   carries `Co-Authored-By:` and `Claude-Session:`; mine did not. I had checked the
   convention with `git log -1 --format='%B' | head -20`, and the `head -20` cut the trailer
   off -- truncated output read as absence. Fixed in the same amend. The commit hash in this
   report is the amended one, read back with `git log`.
3. **Stale `run.rs:NNNN` citations elsewhere, not fixed.** Nineteen markdown files under
   `.superpowers/sdd/` and `docs/superpowers/plans/` cite `run.rs` line numbers at or beyond
   9720, which now name nothing. **No file under `rust/` cites a line number** -- not a
   source file, not a test, not a script. They are historical records taken at particular
   commits and were already drifting before this move, so I left them; the plan's own
   constraint says line numbers in it will move and to grep for the sentence.
4. **Comments that say the tests are in this file, and now they are not. Not fixed, and
   the task forbade fixing them.** The line-number half of finding 3 is what was true of
   `rust/`; **this class is in code, and my first version of finding 3 read as though the
   code were clean.** Five doc and line comments in `run.rs`'s own production half say
   "(this file's own tests)" about tests that now live in `run/tests.rs`:
   `crates/rexx-exec/src/run.rs:169`, `:231`, `:4635`, `:8650`, `:8668`. The same class
   appears in four more places, all pointing at `run.rs` from outside:
   `crates/rexx-exec/src/eval.rs:1499` ("`run.rs`'s own test module"),
   `crates/rexx-exec/src/plan.rs:1405` ("output-level tests in `run.rs`"),
   `crates/rexx-parse/tests/sourceline_oracle/call_expression.txt:21` and
   `corpus/lang/call_expression.rex:20` (both "pinned by run.rs's own unit tests"). Each is
   now imprecise rather than false in the last four cases -- `run/tests.rs` is still
   `run.rs`'s test module -- but the five inside `run.rs` say "this file" and that is now
   wrong. Whoever fixes them should also fix finding 5, which sits in the same comment block
   as one of them.
5. **A comment naming a test that does not exist. Pre-existing, not caused by this move, not
   fixed.** `crates/rexx-exec/src/run.rs:4634` cites
   `current_clause_line_is_restored_after_a_nested_call_or_signal`, which appears nowhere in
   the tree. The test it means is
   `current_clause_line_is_restored_after_a_nested_expression_call`, now at
   `crates/rexx-exec/src/run/tests.rs:3423`. The wrong name predates this commit; the only
   thing the move changed is which file the right one is in.

## 8. Concerns

* **`git blame` on the moved region, and `-w` is not optional.** Plain
  `git blame run/tests.rs` attributes all 7,707 lines to this commit, and so does
  `git blame -C -C -C`: every moved line's leading whitespace changed, so copy detection
  alone does not see through it. Measured on lines 200-208, 1000-1020 and 5000-5020, all
  three ranges come back as `4e71fe0d3` without `-w`. **`git blame -w -C -C -C` does recover
  the real history** -- the same ranges then name `43e184624`, `958a06b90`, `92e802181`,
  `2c9b966c5`, `3a9d6446a` and `addf89b10`. `git log` recovers nothing in any form:
  `git log -M -C -- <path>`, `git log --follow -- <path>` and `git log -w --follow -- <path>`
  each return only the move commit, because no whole-file rename occurred. So the incantation
  a reader needs is `blame -w -C -C -C`, and there is no `log` equivalent. The scout named
  this as the largest permanent cost of any split and it is real. It is also smallest for
  exactly this shape, one contiguous block.
* **The green suite proves very little here**, as the 2026-08-07 note said it would: a pure
  move has no differential signal. The evidence that carries this commit is the byte-for-byte
  round-trip in §4, not the 1509 passing tests.
* **`cargo doc` reports nine warnings across three other crates**, listed in §4. None is
  mine, and none is denied anywhere in the workspace, so a future intra-doc breakage would
  land silently the same way. Adding `#![deny(rustdoc::broken_intra_doc_links)]` to the crate
  roots would close that, but it is out of this task's scope and would fail today on
  `rexx-bench` and `rexx-extract`.

## 9. Fix round 1

Verdict was Approved with two Important report corrections and two minors. **All four are
corrections to this report. No code, no commit and no gate changed**, and the commit is still
`4e71fe0d316e1ad1da673e1b511aa83f3de9ffbc`. I re-measured each of the four myself rather than
taking the review's word for it, and every one reproduced.

* **I1, `git blame` mitigation stated wrongly.** Confirmed: `git blame -C -C -C` on
  `run/tests.rs` lines 1000-1005 returns `4e71fe0d3` and nothing else, while
  `git blame -w -C -C -C` on 1000-1020 returns `2c9b966c5`, `3a9d6446a`, `43e184624`,
  `958a06b90` and `addf89b10`. `git log --follow` and `git log -M -C` on the path each return
  the single move commit. §8 now names `blame -w -C -C -C` and states that `log` has no
  working form, rather than restating it.
* **I2, the found-and-not-fixed list missed the class that lives in code.** Confirmed at all
  nine sites the review named, by reading each line. My finding 3 was true only of *line
  number* citations, and its bolded "no file under `rust/` does" invited the wrong reading.
  §7 now scopes finding 3 to line numbers and adds finding 4 with all nine paths.
* **M3, the module doc justified against a precedent that points the other way.** Confirmed:
  `ir/drive/tests.rs:12`, `ir/golden_tests.rs:12` and `ir/corpus_shape_tests.rs:12` each open
  with `//!`; `run/tests.rs:12` is `use super::*;`. §5 now records this as a gap rather than
  as something the precedent excuses. The file is unchanged.
* **M4, a comment naming a test that does not exist.** Confirmed:
  `current_clause_line_is_restored_after_a_nested_call_or_signal` appears only at
  `run.rs:4634` and nowhere else in `crates/`; the real test is
  `current_clause_line_is_restored_after_a_nested_expression_call` at `run/tests.rs:3423`.
  Pre-existing. Recorded as finding 5, beside finding 4 because they share a comment block.

Nothing this round is compiled, so no gate was re-run. The review's own three instruments --
the reproduced round trip, the 233,686-byte whitespace-stripped comparison, and the decode of
1,421 string and char literals showing 129 differing in source bytes and zero in value -- are
stronger evidence for the move than anything in §4 and are noted here so a later reader finds
them.
