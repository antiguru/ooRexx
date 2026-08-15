### Task 7 report: rename `{name}/mod.rs` to `name.rs`

Commit: `334ca279e501d71005ef0cf0eab5f9b6578289a5`

#### The four-file set and eight declaring sites, as found

`find crates -name mod.rs` returned exactly four files, matching the brief:

* `crates/rexx-exec/src/builtin/mod.rs` -- in scope
* `crates/rexx-exec/src/ir/mod.rs` -- in scope
* `crates/rexx-exec/tests/support/mod.rs` -- out of scope
* `crates/rexx-parse/tests/gate_walk/mod.rs` -- out of scope

Declaring sites for the two in-scope files, both in `crates/rexx-exec/src/lib.rs`, one line each:
`mod builtin;` and `mod ir;`. Both resolve identically for either file spelling, and did.

Declaring sites for `mod support;`: exactly the seven binaries the brief names --
`builtin_status.rs`, `corpus.rs`, `input_oracle.rs`, `parse_version_oracle.rs`,
`state_builtin_oracle.rs`, `trace_indent.rs`, `trace_oracle.rs`.

Declaring sites for `mod gate_walk;`: **the brief undercounts this by one.** It names only
`rexx-parse/tests/tiling.rs`. `grep -rn "mod gate_walk"` also finds it in
`rexx-parse/tests/variants.rs:28`, which is unsurprising given `gate_walk/mod.rs`'s own module
doc already says "shared by the Phase 3 gate tests (`tiling.rs`, `variants.rs`)". This does not
change the scope decision -- `gate_walk/mod.rs` stays `mod.rs` either way, for the same
auto-discovery reason -- but the brief's "declared by ... `rexx-parse/tests/tiling.rs`" undercounts
the declaring set and should be corrected if this brief is reused.

#### What was renamed, and confirmation nothing else needed editing

`git mv crates/rexx-exec/src/builtin/mod.rs crates/rexx-exec/src/builtin.rs` and the equivalent
for `ir/mod.rs` -> `ir.rs`. `git diff --cached -M` on both shows zero content diff: pure renames.
`cargo build --workspace --all-targets` succeeded with no other file touched. No other file needed
editing.

#### Per-binary test counts, before and after

Both runs: `cargo test --release --workspace --no-fail-fast` under `memcap 24G` (8G OOM-killed
mid-compile against a cold, fresh `CARGO_TARGET_DIR`; 24G compiled and ran clean), target dir
`/tmp/claude-1000/.../scratchpad/task7-target`.

**Corrected in fix round 1 -- see that section below for how this was wrong the first time.**
83 `test result:` blocks (not 82: `Doc-tests rexx_exec` runs two separate doctest blocks for
`run::Interp::run_activation`, one plain and one `- compile fail`, and the original count grouped
by binary label rather than by result block, silently merging them into one row). Comparing every
block's (label, passed, failed, ignored) as a multiset, keyed by the doctest's own name where the
label is `Doc-tests`, with the per-run hash suffix stripped: **identical, all 83 rows, in both
directions.** Total: 1509 passed, 0 failed, 4 ignored, both times. No binary lost or gained a
test, and no binary stopped or started being compiled.

Selected rows (full set is 83; these are the ones nearest the rename):

| binary | before (P/F/I) | after (P/F/I) |
|---|---|---|
| `rexx_exec` (unittests, src/lib.rs) | 629/0/0 | 629/0/0 |
| `builtin_status.rs` | 13/0/0 | 13/0/0 |
| `corpus.rs` | 10/0/1 | 10/0/1 |
| `input_oracle.rs` | 9/0/0 | 9/0/0 |
| `ir_dual.rs` | 9/0/0 | 9/0/0 |
| `parse_version_oracle.rs` | 8/0/0 | 8/0/0 |
| `state_builtin_oracle.rs` | 9/0/0 | 9/0/0 |
| `trace_indent.rs` | 11/0/0 | 11/0/0 |
| `trace_oracle.rs` | 25/0/0 | 25/0/0 |
| `tiling.rs` | 11/0/0 | 11/0/0 |
| `variants.rs` | 1/0/0 | 1/0/0 |
| Doc-tests `rexx_exec` :: `run_activation` (line 1073) | 1/0/0 | 1/0/0 |
| Doc-tests `rexx_exec` :: `run_activation` (line 1093) - compile fail | 1/0/0 | 1/0/0 |

(Every other row of the 83 matched identically too; not reproduced here to avoid a duplicate of
the full comparison already run programmatically.)

#### Gates

* `cargo fmt --all --check`: exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings`, fresh target dir, both before
  (baseline tree, stashed) and after: exit 0, no warnings either time.
* `cargo test --release --workspace --no-fail-fast`: exit 0 both before and after, counts as above.
* `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast corpus`: exit 0, 22 tests
  ran under the name filter (non-zero, so this is not a "matches nothing" false green). **Superseded
  in fix round 1** by an unfiltered run, since the name filter skipped two other files that also
  gate on this env var (`tests/parse_version_oracle.rs`, `tests/input_oracle.rs`) whose test names
  do not contain "corpus" -- see M1 below.

`cargo doc --no-deps`: exit 0 both before and after. **18 pre-existing warnings, identical set
before and after the rename** (verified by stashing the rename+notes, rerunning `cargo doc`
against the unmodified tree in a separate fresh target dir, and diffing the sorted warning-header
lines -- zero diff). None of the 18 mention `builtin` or `ir`; they are pre-existing
`rustdoc::private_intra_doc_links`, `rustdoc::broken_intra_doc_links`, and
`rustdoc::redundant_explicit_links` findings in `rexx-extract`, `rexx-num`, `rexx-bench`, and
`rexx-exec/src/lib.rs` (an `Engine::TreeWalker`/`Engine::Ir` link redundancy, unrelated to the
`ir` module path). The rename introduced no new doc warning.

Full `cargo doc --no-deps` output after the rename (18 warnings, all pre-existing):

```
warning: public documentation for `bif` links to private item `crate::keyword::blank_comments`
  --> crates/rexx-extract/src/bif.rs:21:18
warning: public documentation for `bif` links to private item `Fixtures`
  --> crates/rexx-extract/src/bif.rs:38:7
warning: public documentation for `bif` links to private item `segments`
   --> crates/rexx-extract/src/bif.rs:110:7
warning: public documentation for `keyword` links to private item `clause_boundary`
  --> crates/rexx-extract/src/keyword.rs:42:21
warning: public documentation for `program` links to private item `rewrite_body`
   --> crates/rexx-extract/src/keyword.rs:115:63
warning: public documentation for `count_assert_same` links to private item `rewrite_line`
   --> crates/rexx-extract/src/keyword.rs:273:18
warning: public documentation for `sub_code` links to private item `FormatError::sub`
   --> crates/rexx-num/src/format.rs:107:18
warning: unresolved link to `Figure::fmt`
  --> crates/rexx-bench/src/arms.rs:38:23
warning: public documentation for `arms` links to private item `Workload::rendered`
  --> crates/rexx-bench/src/arms.rs:49:41
warning: unresolved link to `Sitting::per_pass`
   --> crates/rexx-bench/src/arms.rs:231:11
warning: `rexx-extract` (lib doc) generated 6 warnings
warning: `rexx-num` (lib doc) generated 1 warning
warning: `rexx-bench` (lib doc) generated 3 warnings
warning: unresolved link to `the_caveat_matches_the_committed_baseline`
   --> crates/rexx-bench/src/bin/rexx-bench-suite.rs:604:7
warning: `rexx-bench` (bin "rexx-bench-suite" doc) generated 1 warning
warning: redundant explicit link target
   --> crates/rexx-exec/src/lib.rs:378:49
warning: redundant explicit link target
   --> crates/rexx-exec/src/lib.rs:379:54
warning: `rexx-exec` (lib doc) generated 2 warnings
```

#### The note added at the two `tests/` files

**The `support/mod.rs` note below was corrected in fix round 1** (dropped "seven" -- see I3 in
that section). As it originally shipped:

At `crates/rexx-exec/tests/support/mod.rs`, before the existing module doc:

> This file stays `mod.rs`, not `support.rs`, because `tests/` is Cargo's integration-test root:
> a bare `tests/support.rs` would be auto-discovered as its own test binary in addition to being
> pulled in as a module by the seven files that declare `mod support;`, compiling these helpers
> standalone for no reason. Renaming it to `support.rs` would add that extra binary back.

At `crates/rexx-parse/tests/gate_walk/mod.rs`, before the existing module doc:

> This file stays `mod.rs`, not `gate_walk.rs`, because `tests/` is Cargo's integration-test root:
> a bare `tests/gate_walk.rs` would be auto-discovered as its own test binary in addition to being
> pulled in as a module by `tiling.rs` and `variants.rs`, compiling this walk standalone for no
> reason. Renaming it to `gate_walk.rs` would add that extra binary back.

#### Files changed

* `rust/crates/rexx-exec/src/builtin/mod.rs` -> `rust/crates/rexx-exec/src/builtin.rs` (rename, no
  content change)
* `rust/crates/rexx-exec/src/ir/mod.rs` -> `rust/crates/rexx-exec/src/ir.rs` (rename, no content
  change)
* `rust/crates/rexx-exec/tests/support/mod.rs` (note added, 7 lines)
* `rust/crates/rexx-parse/tests/gate_walk/mod.rs` (note added, 6 lines)

#### Self-review

`git diff --cached --stat` on the commit: 4 files changed, 13 insertions(+), 0 deletions from
content (the two renames show 0 changed lines; `git diff --cached -M` on both confirms empty
content diff). The two note additions are 7 and 6 lines respectively, matching the "two or three
sentences" instruction (each note is three sentences). No other files touched. `git status` after
the commit shows a clean tree except one untracked file from outside this task's scope
(`docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md`, not created by this session,
left alone).

**Count: 0 findings.** The diff is exactly the two renames plus the two three-sentence notes the
task asked for; nothing else changed.

#### Concerns

* The brief's declaring-site count for `mod gate_walk;` is off by one (`variants.rs` also
  declares it, not only `tiling.rs`); recorded above so a future reader of the brief sees the
  correction. It does not affect the scope decision.
* The 18 `cargo doc` warnings are pre-existing and unrelated to this task, but they mean `cargo
  doc --no-deps` is not currently a clean gate for this workspace; that is a pre-existing
  condition, not something this task introduced or should fix under its own scope.

---

### Fix round 1

Commit: `b0ea317b6df292b99f7c1719c5d6fc7c69bb189d` (message amended after the fix-round review;
content unchanged -- see "Commit message amend" below).

#### I1: the count was wrong, and wrong in a way both sides shared

The reviewer's re-read was correct. `task7-before.log` and `task7-after.log` each hold 83
`^test result:` blocks; my original comparison grouped by the `Running`/`Doc-tests` **label**
rather than by result block, and `Doc-tests rexx_exec` prints two blocks under one label
(`run::Interp::run_activation` at line 1073, plain, and again at line 1093 with `- compile fail`).
My pairing script kept one of the two blocks for that label and dropped the other silently --
the script was run inline and was not saved to the scratchpad, and both blocks show 1/0/0, so
which one survived is not recoverable from the retained tables. Recount, this time keyed by result block and, within `Doc-tests`, by the doctest's own
`test <name> ...` line: **83 blocks, 1509 passed, 0 failed, 4 ignored, before and after,
identically.** Verified with an unpiped `grep -c "^test result:"` on each log (83, 83) and a
summed pass/fail/ignore over every block (1509/0/4, 1509/0/4) before comparing the full 83-row
multiset (identical). The corrected figures are now in the "Per-binary test counts" section above.

**Why this matters beyond the arithmetic, stated so it does not need re-finding:** the dropped
row was dropped identically on both sides of the comparison, so a run in which that particular
doctest vanished (moved, deleted, no longer discovered) would have produced the same "identical"
verdict my flawed script gave here. Grouping by label instead of by result block is blind to
exactly the row it collapses. The count comparison is the one control this task exists to
provide, and a label-keyed count is not that control on any binary that emits more than one
result block under one label -- which edition-2024 doctest merging can do without warning:
mergeable doctests in a crate compile and run as one binary and print one block, but a doctest
that cannot be merged prints its own separate block under the same `Doc-tests <crate>` label.
`crates/rexx-exec/src/run.rs:1093` opens a `` ```compile_fail `` fence, which cannot be merged
with the plain doctest above it at line 1073; the evidence is in the log itself --
`task7-fixround-test.log:2208` reads `all doctests ran in 1.57s; merged doctests compilation took
1.55s`, immediately after the two separately-printed `run::Interp::run_activation` result blocks.
(`task-6-report.md:64-66` already records this correctly as two named blocks; this paragraph's
earlier wording gave a mechanism -- `#[doc(hidden)]` and "more than one item under an ambiguous
label" -- that does not exist in rustdoc and is corrected here to match.)

#### I2: six stale `mod.rs` comment citations

Unpiped search, full hit list:

```
$ grep -rn "builtin/mod.rs\|ir/mod.rs" crates/
crates/rexx-exec/src/ir/golden.rs:14://! (`ir/mod.rs`), not a `#[allow(dead_code)]` here: no task in the plan ever
crates/rexx-exec/src/builtin/datetime.rs:895:/// ARGUMENT_DIGITS` -- the same precision `whole_number` (`builtin/mod.rs`)
crates/rexx-exec/src/builtin/datetime.rs:1924:    /// `date` checks its own positions (`builtin/mod.rs`'s own module doc
crates/rexx-exec/src/builtin/state.rs:19://! builtin adds no activation of its own (`builtin/mod.rs`'s module doc has
crates/rexx-exec/src/builtin/state.rs:30://! ADDRESS; maximum expected is 0.` at rc 216. `builtin/mod.rs`'s table is
crates/rexx-exec/src/builtin/state.rs:566:/// conversion, and they report the result of it. `builtin/mod.rs`'s
```

Six hits, matching the reviewer's list exactly. Disposition: all six fixed, `mod.rs` -> `.rs`
substitution only, no other change to the sentence:

| site | before | after |
|---|---|---|
| `builtin/src/state.rs:19` | `builtin/mod.rs`'s module doc has | `builtin.rs`'s module doc has |
| `builtin/src/state.rs:30` | `builtin/mod.rs`'s table is | `builtin.rs`'s table is |
| `builtin/src/state.rs:566` | `builtin/mod.rs`'s `length_of`... | `builtin.rs`'s `length_of`... |
| `builtin/src/datetime.rs:895` | (`builtin/mod.rs`) | (`builtin.rs`) |
| `builtin/src/datetime.rs:1924` | (`builtin/mod.rs`'s own module doc | (`builtin.rs`'s own module doc |
| `ir/golden.rs:14` | (`ir/mod.rs`) | (`ir.rs`) |

Re-ran the same search after editing: zero hits for `builtin/mod.rs` or `ir/mod.rs` anywhere
under `crates/`. Left `docs/superpowers/plans/phase-4f-record.md:1441` and
`docs/superpowers/plans/2026-08-09-phase-4e-ir.md:93` untouched, as instructed -- they are outside
`crates/` and record completed phases at the paths that existed when those phases ran.

#### I3: the cardinality claim, and the "eighth consumer" question

Dropped "seven" from `crates/rexx-exec/tests/support/mod.rs`'s new note: "pulled in as a module by
the files that declare `mod support;`" (numeral removed, no replacement number).

**Checked the "eighth consumer" claim rather than accepting it.** Re-ran
`grep -n "mod support;" crates/rexx-exec/tests/*.rs` unpiped: seven hits, the same seven files
Step 1 already found (`builtin_status.rs`, `corpus.rs`, `input_oracle.rs`,
`parse_version_oracle.rs`, `state_builtin_oracle.rs`, `trace_indent.rs`, `trace_oracle.rs`). A
broader `grep -rln "mod support" crates/` (no `;`, so it also matches prose) additionally lists
`crates/rexx-exec/tests/support/oracle.rs` and `crates/rexx-exec/tests/support/mod.rs` itself, but
both are prose hits, not declaring sites: `support/oracle.rs:76` is a comment reading "This module
is pulled in by `mod support;` in more than one integration [test]", and the two hits inside
`support/mod.rs` are my own new note's prose (the phrase "declare `mod support;`" and "so `mod
support;` in"). **There is no eighth consumer.** The set stays at seven files, which is exactly why
the numeral is dropped rather than corrected to a different number -- the rule holds regardless of
the count, and the count itself did not move.

#### Gates re-run (I2 touched compiled files)

* `cargo fmt --all --check`: exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings`, fresh cold `CARGO_TARGET_DIR`
  (`task7-target-clippy2`, never used for anything else): exit 0, no warnings.
* `cargo test --release --workspace --no-fail-fast`: exit 0. 83 blocks, 1509 passed, 0 failed, 4
  ignored -- unchanged from the pre-fix-round numbers, as expected for a comment-only change.
* `cargo doc --no-deps`, fresh cold target dir (`task7-target-doc2`): exit 0, 18 warnings, and the
  sorted warning-header set diffs to zero against the pre-rename baseline captured earlier. No new
  warning from the comment edits (expected -- none of the six sites were doc comments with
  intra-doc link syntax naming `builtin/mod.rs` or `ir/mod.rs` as a path rustdoc resolves; all six
  were plain backtick-quoted prose, not `` [`...`] `` links).

#### M1: the corpus gate, unfiltered

Re-ran without the `corpus` name filter: `REXX_CORPUS_GATE=1 cargo test --release --workspace
--no-fail-fast`, full workspace, exit 0, 83 blocks, 1509 passed, 0 failed, 4 ignored. This
includes `tests/parse_version_oracle.rs` and `tests/input_oracle.rs`, both of which read
`REXX_CORPUS_GATE` themselves (confirmed at `parse_version_oracle.rs:71` and
`input_oracle.rs:72`) but produce the same test count under the gate as without it -- their
`gate_mode()` check branches *inside* a test body rather than adding or removing a test, so an
identical count does not mean the gate went unexercised, only that it does not change discovery.
Both ran and passed under `REXX_CORPUS_GATE=1`.

#### M2: which test is actually ignored, and the better evidence for STRICT mode

Corrected in the report above. `corpus.rs`'s actually-`#[ignore]`d test is
`probe_emit_uncaptured_marker` (`corpus.rs:558`, ignored as "run only by
demonstrate_the_report_reaches_a_plain_cargo_test, as a child process") -- unrelated to
`REXX_CORPUS_GATE`, and not the test my original wording implied. The better evidence that the
gate ran in STRICT mode: the run's own banner, `mode: STRICT (the gate) -- REXX_CORPUS_GATE is
set` (`corpus.rs:335`), present in both the name-filtered and the unfiltered fix-round logs.

#### M3: the redundant sentence at `tests/support/mod.rs`

Trimmed. The pre-existing "Why one copy, not two" paragraph had a second sentence restating the
same auto-discovery mechanism the new top-of-file note already covers, in narrower terms ("`mod
support;` in each of the two consuming files pulls this in without adding a third test target").
Kept the new top note (first thing a renamer reads, and it is the general statement); cut the
older sentence down to the point unique to that paragraph -- that this one function serves two
call sites and must answer both identically, which is the actual justification for writing it
once.

#### Self-review of the fix round

**What was actually checked, not just the number it produced:**

1. Read every one of the 8 changed hunks in `git diff` (not `--stat`, the full text) line by
   line before staging, confirming each of the six comment hunks changes only the substring
   `mod.rs` -> nothing (i.e. `builtin/mod.rs` -> `builtin.rs`, `ir/mod.rs` -> `ir.rs`) with every
   other character on the line identical, and that the two `support/mod.rs` hunks are exactly the
   cardinality drop (I3) and the redundant-sentence trim (M3) and nothing else.
2. Re-ran the I2 search verbatim after editing (`grep -rn "builtin/mod.rs\|ir/mod.rs" crates/`,
   shown above): zero hits, confirming the fix is complete and I did not miss a seventh site or
   introduce a new stale citation elsewhere.
3. Re-ran the I3 declaring-site search verbatim after editing (`grep -n "mod support;"
   crates/rexx-exec/tests/*.rs`): still seven hits, unchanged, confirming the cardinality drop
   did not silently change what the sentence refers to.
4. Confirmed via `git diff --stat` that only the four intended files changed (no fifth file
   touched by an editor artifact or a stray save).
5. Ran the four gates in "Gates re-run" above from cold target directories and read every line
   of clippy's and doc's output rather than only the exit status, confirming no new warning
   appeared anywhere in the workspace as a side effect of a comment-only change.
6. Confirmed `git status` after the commit is clean except the one pre-existing untracked file
   from outside this task's scope.

Given all six of those actually ran and returned what the fix intended, **count: 0 findings**,
and it is a zero the checks above earned rather than a zero from not looking -- the diff is
exactly six `mod.rs` -> `.rs` substitutions, one cardinality drop, and one redundant-sentence
trim, confirmed against both the intended scope (I1/I2/I3/M3) and the actual staged text.

#### Commit message amend

The coordinator flagged the fix-round subject, "Task 7 fix round 1: correct six stale mod.rs
comment citations, drop a set-size claim", for naming an SDD queue slot that will not exist once
this plan's workspace is deleted -- unlike every other commit on this branch, which names the
change itself. Amended via `git commit --amend` (message only, unpushed, content untouched) to
"Correct six comments naming the mod.rs files this branch renamed", body unchanged. New hash:
`b0ea317b6df292b99f7c1719c5d6fc7c69bb189d`. `git diff 2df551398 b0ea317b6 --stat` -- the pre-amend
commit against the post-amend one, which is the range that actually tests an amend -- is empty,
confirming the amend changed no file content.

#### Two questions asked directly, answered directly

**Is there an eighth consumer of `support`?** No. `grep -n "mod support;"
crates/rexx-exec/tests/*.rs`, run unpiped, returns exactly seven hits: `builtin_status.rs`,
`corpus.rs`, `input_oracle.rs`, `parse_version_oracle.rs`, `state_builtin_oracle.rs`,
`trace_indent.rs`, `trace_oracle.rs`. A broader, non-`;`-anchored `grep -rln "mod support"
crates/` additionally names `support/oracle.rs` and `support/mod.rs` itself, but both are prose
mentioning the phrase, not declaring sites (`support/oracle.rs:76` is a comment describing the
mechanism; the `support/mod.rs` hits are my own note's prose). At close-out the reviewer confirmed
this independently (its own search, not a re-read of mine) and retracted the original framing:
what it meant was that any *future* eighth consumer would falsify a numeral written today, not
that one exists now -- the definite article in "falsified by the eighth consumer" read as a
present-tense claim its own Step 1 (seven sites, listed) already contradicted. The cardinality
fix in I3 is unconditional and does not depend on this either way; there is no eighth route today,
and the set is seven.

**What does the fix-round self-review's 0 mean?** See "Self-review of the fix round" above for
the six concrete checks; the number is not a bare assertion -- every hunk was read in full,
both underlying searches (I2's stale-path grep, I3's declaring-site grep) were re-run after
editing rather than assumed, and the gates were read for new warnings rather than only exit
status.

#### Out-of-scope note worth carrying forward: `owners.rs` is not a precedent for `support/mod.rs`

`crates/rexx-exec/tests/owners.rs` is auto-discovered by Cargo as its own `tests/*.rs` binary
**and** deliberately `#[path = "owners.rs"] mod owners;`-included a second and third time, by
`loud.rs:107` and `coverage.rs:136`. Its own module doc (`owners.rs:19-26`) says this tripling is
intentional: it is what makes `cargo test -p rexx-exec --test coverage` alone, without also
running `owners`, still verify `owners.rs`'s own invariants. So this crate does contain the shape
`support/mod.rs`'s note calls "for no reason" -- a file compiled more than once for a deliberate
reason.

The `support/mod.rs` note's "for no reason" judgment is still correct, but it is specific to
`support` and does not transfer to `owners.rs`, and a future reader could otherwise cite
`owners.rs` as licence to rename `support/mod.rs` to `support.rs`. The difference: `owners.rs`'s
extra compilation is the point (the module doc names the invariant it buys), while `support.rs`
would gain a fourth compiled copy -- a whole new `tests/support` integration-test binary running
none of `support`'s own tests as its subject, only re-running the shared helper code as dead
weight under a binary nothing asks for. `owners.rs` chose to be auto-discovered *and* `#[path]`-
included, on purpose, and says so; `support/mod.rs` stays `mod.rs` specifically to avoid being
auto-discovered at all, because nothing here would use the resulting binary the way `coverage`
and `loud` use `owners`'s.

#### Concerns (fix round)

* None new. The `mod gate_walk;` undercount and the pre-existing `cargo doc` warnings from the
  original report still stand as stated there.

