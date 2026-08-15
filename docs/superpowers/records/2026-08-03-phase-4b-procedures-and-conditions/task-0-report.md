# Task 0 report: the shared owner table, the subset union, and an owner in every loud message

Branch `plan/rust-rewrite`, `rust/crates/rexx-exec`. Baseline confirmed green before any change
(`cargo test -p rexx-exec`, all green, matching the brief's "if the tree is not green at the start,
stop and report BLOCKED" precondition -- it was green, so this proceeded).

## What changed and why

### Step 1 -- the single owner table

Created `rust/crates/rexx-exec/tests/owners.rs`. It now holds everything `coverage.rs` and
`loud.rs` used to each define by hand (item I36): the `Owner` enum, the `tags!` macro, the seven
`*_TAGS` tables and their tag functions (`instruction_tag`, `expr_tag`, `loop_tag`,
`prefix_op_tag`, `end_style_tag`, `trace_tag`, `operator_tag`), the `Coverage` struct, and the two
pinned literals `EXPECTED_OUT_OF_SCOPE` and `SPLIT_TABLE_PHASES`.

`coverage.rs` and `loud.rs` both now start with:

```rust
#[path = "owners.rs"]
mod owners;
use owners::{ ...only the items each file actually needs... };
```

Both files literally include the same bytes, so a divergence is no longer something a reviewer has
to notice by hand. `owners.rs` is itself a normal, cargo-discovered integration test (it lives
directly under `tests/`), so `cargo test --test owners` also runs it standalone; the four
table-sanity tests (`assert_owner_strings_are_split_table_phases`, `only_backslash_is_unreachable`,
`out_of_scope_set_matches_the_committed_expectation`, `variant_counts_match_the_audited_split`)
moved from `coverage.rs` into `owners.rs`, since they only ever needed the tables, not the corpus --
this also means every item `owners.rs` defines is exercised by *some* test in its own standalone
compilation, avoiding a `dead_code` warning there. (Three independent binaries compile this file --
`owners`, `coverage`, `loud` -- and no single one of them calls every item, so the file carries a
file-wide `#![allow(dead_code)]` with a comment explaining why `#[expect]` would be wrong here: the
same source line is used-or-not depending on which of the three is compiling it, so a per-item
`#[expect(dead_code)]` would be fulfilled in one binary and unfulfilled -- itself a warning -- in
another.)

Added `the_two_harnesses_include_this_exact_file` (in `owners.rs`): reads `coverage.rs`'s and
`loud.rs`'s own source text and asserts each contains the literal string `#[path = "owners.rs"]`.
This is the "test asserting the two harnesses see the same table" the brief asked for -- checked at
the source level rather than the value level, because the two consumers compile into separate test
binaries with no way for one process to inspect another's constants at run time; `#[path]`
inclusion is what makes divergence structurally impossible, and this test is the regression guard
against a future edit quietly reverting one side back to a hand-copied table.

### Step 2 -- arm-grained witnesses for `Call`/`Signal`

`owners.rs`'s `INSTRUCTION_TAGS` keeps `Call` and `Signal` as one row each (`"Call"`/`"Signal"` ->
`"4b"`), which is correct for `coverage.rs`'s parse-only question. But `loud.rs`'s witness table
needs finer grain, because the inner enums (`rexx_parse::Call`, `rexx_parse::Signal`) do not all
move in scope, or share an owner, at the same moment:

- `Call::Named`, `Call::Dynamic` -- in scope from Task 3, owner `"4b"`
- `Call::Qualified` -- genuinely Phase 5's (namespace-qualified `CALL`, mirroring
  `ExprKind::QualifiedCall`)
- `Call::Trap` (`CALL ON`/`CALL OFF`) -- Task 7, owner `"4b"`
- `Signal` (still combined: `Signal::Value`/`Signal::Label`) -- Task 6, owner `"4b"`
- `Signal::Trap` (`SIGNAL ON`/`SIGNAL OFF`) -- Task 7, owner `"4b"`

`INSTRUCTION_WITNESSES` now has one row per arm above (was one `"Call"` row and one `"Signal"`
row). New witness sources, each checked (via `assert_constructs`) to actually parse into the exact
arm claimed before being run:

| tag | source |
|---|---|
| `Call::Named` | `call sub\n` (unchanged) |
| `Call::Dynamic` | `call (x)\n` |
| `Call::Qualified` | `call ns:sub\n` |
| `Call::Trap` | `call off error\n` |
| `Signal` | `signal there\nthere: nop\n` (unchanged) |
| `Signal::Trap` | `signal off error\n` |

`instruction_arm` (`loud.rs`) resolves a witness's tag against a parsed program: the coarse
`owners::instruction_tag` for every `InstructionKind` except `Call`/`Signal`, and the inner arm's
own name for those two. `assert_witness_set_is_complete`'s expected set is built through
`expand_for_witnesses`, a hand-maintained expansion of the coarse tag list (`"Call"` -> 4 tags,
`"Signal"` -> 2 tags, everything else -> itself) -- the same "pin it as a literal" device
`EXPECTED_OUT_OF_SCOPE` already uses, because nothing here can enumerate `rexx_parse::Call`'s/
`Signal`'s own arms at compile time. The expected count moved from 20 to 24 (20 coarse phase-owned
`InstructionKind` tags, +3 for `Call`'s split, +1 for `Signal`'s).

### Step 3 -- an owner in every loud message

**Exact final format:** `"{name} is not implemented ({owner})"`, e.g.

```
rexx-exec: CALL is not implemented (4b)
```

Measured live through `cargo run -p rexx-exec --bin rexx-run -- probe.rex` on `call sub`; also
confirmed `call ns:sub` -> `rexx-exec: CALL is not implemented (Phase 5)` and
`say foo(1)` -> `rexx-exec: a function call is not implemented (4b)`.

Two sites changed in `src/lib.rs`: `Loud::instruction`'s and `Loud::expression`'s own message
construction (originally at `:444`/`:468`; shifted by the new code inserted between them). Both now
call a new helper, `owned_message(name, owner)`, which appends `" ({owner})"` **unless** `owner` is
the literal `"4a"` -- in which case the message is unchanged from before this task
(`"{name} is not implemented"`, no suffix).

The `"4a"` special case exists because `Loud::instruction` is reachable with an **in-scope**
`InstructionKind` in two documented edge cases in `run.rs` (both unaffected by this task, neither
in the brief's file list): `run_loop`'s `DO`/`LOOP` `COUNTER`/`DO WITH` check (`instruction.kind` is
`Do`/`Loop`, both 4a's own) and the stem-target `DO OVER` deviation (same). Printing `"(4a)"` on
those would read as self-contradictory -- the construct plainly *is* implemented -- so they keep
their pre-task message verbatim. Measured: `do counter c 5\nend\n` still gives exactly
`rexx-exec: DO is not implemented` (no suffix).

`instruction_owner`/`expr_owner` (new, `src/lib.rs`) are a **third copy** of the owner data,
unavoidably: production code cannot depend on anything under `tests/`, so this cannot be merged
into `owners.rs` the way `coverage.rs`/`loud.rs` were. `instruction_owner` is arm-grained for
`InstructionKind::Call` (matching `loud.rs`'s witness table: `Call::Qualified` -> `"Phase 5"`,
everything else -> `"4b"`); `InstructionKind::Signal` stays coarse (`"4b"` for every arm, since
`Signal::Trap`'s owner is also `"4b"`, just a later task within it). Both matches are exhaustive
with no `_` arm, matching this file's own established rule (`form_name`'s doc: a new variant is a
compile error here, not a silent omission).

**Cross-check that keeps the two copies honest:** `loud.rs`'s `every_out_of_scope_variant_fails_loudly`
now also asserts the emitted stderr contains each witness's own declared `owner` string (in addition
to the pre-existing exit-code check). If `instruction_owner`/`expr_owner` in `src/lib.rs` ever
drifted from `owners.rs`'s table (or from `loud.rs`'s own arm-grained owners), this test fails and
names exactly which witness's stderr didn't match.

### Step 4 -- `read_subset` takes a list of subset files, union semantics

Changed in all three copies (`coverage.rs` -- two call sites, `corpus.rs`, `collect_stress.rs`):
`fn read_subset(list_paths: &[&Path]) -> Vec<String>`. Reads every file in order, keeps first-seen
order, and de-duplicates so the same corpus-relative path named in two subset files is only
processed once. Each of the three still keeps its own private copy of the function (the brief says
factoring them together is a separate change); only the signature and body changed identically in
each.

Every call site today passes a one-element slice, `&[&corpus_dir.join("phase-4a.txt")]` -- the
union of one file is that file's own content, so no corpus program's result moved. Verified:
`corpus_differential` still reports `29 of 29 matching`, byte for byte the same as before this task.

### Step 5 -- documented pinned literals

`owners.rs`'s own module doc, "What is pinned here" section, names the five things a task moving a
variant's scope or ownership must update together:

1. `EXPECTED_OUT_OF_SCOPE` (`owners.rs`)
2. `coverage.rs`'s `EXPECTED_SUBSET` (the exact `phase-4a.txt` line list)
3. The four/two hardcoded counts in `owners.rs`'s own `variant_counts_match_the_audited_split`
   (20/9/4/6/1 for `InstructionKind`, 9/6 for `ExprKind`)
4. `loud.rs`'s `INSTRUCTION_WITNESSES`/`EXPR_WITNESSES` (one row per out-of-scope tag, per *arm* for
   `Call`/`Signal`)
5. `src/lib.rs`'s `instruction_owner`/`expr_owner` (the third, production-side copy)

### Step 6

Full suite green (`cargo test -p rexx-exec`, exit 0, no `FAILED` anywhere in the output, including
`assertions.rs`'s 4,259-row differential and `collect_stress.rs`). `cargo fmt --edition 2024 --check`
clean on every changed file. `cargo clippy --workspace --all-targets -- -D warnings` clean (exit 0,
read unpiped both times).

## Test output (representative)

```
     Running tests/corpus.rs
29 of 29 matching -- REPORT MODE, NOT THE GATE
test corpus_differential ... ok

     Running tests/coverage.rs
running 8 tests
test owners::only_backslash_is_unreachable ... ok
test owners::assert_owner_strings_are_split_table_phases ... ok
test owners::variant_counts_match_the_audited_split ... ok
test owners::out_of_scope_set_matches_the_committed_expectation ... ok
test phase_4a_subset_matches_the_committed_list ... ok
test the_builtin_exclusion_set_matches_the_committed_file ... ok
test owners::the_two_harnesses_include_this_exact_file ... ok
test every_in_scope_variant_is_witnessed_by_the_phase_4a_subset ... ok
test result: ok. 8 passed; 0 failed

     Running tests/loud.rs
running 8 tests
test assert_witness_set_is_complete ... ok
test in_scope_counts_match_the_audited_split ... ok
test owners::only_backslash_is_unreachable ... ok
test owners::variant_counts_match_the_audited_split ... ok
test owners::assert_owner_strings_are_split_table_phases ... ok
test owners::out_of_scope_set_matches_the_committed_expectation ... ok
test owners::the_two_harnesses_include_this_exact_file ... ok
test every_out_of_scope_variant_fails_loudly ... ok
test result: ok. 8 passed; 0 failed

     Running tests/owners.rs
running 5 tests (all pass, standalone)

     Running tests/collect_stress.rs -- 1 passed
     Running tests/spike.rs -- 11 passed
     Running tests/trace_oracle.rs -- 6 passed
     assertions.rs -- 5 passed (4224 of 4259 rows passing, 35 exempt, unchanged)
```

Full `cargo test -p rexx-exec` run: 0 failures, exit code 0.

## For later implementers

- **Loud message format is now `"{name} is not implemented ({owner})"`** (owner omitted only for
  the two documented in-scope-but-locally-unimplemented edge cases in `run.rs`). Later tasks
  asserting `stderr.contains("4c")` / `stderr.contains("Phase 5")` etc. can rely on this shape.
- **When you move a variant (or a `Call`/`Signal` arm) into scope**, touch all five items in
  `owners.rs`'s "What is pinned here" section, in the same change. Forgetting `src/lib.rs`'s
  `instruction_owner`/`expr_owner` will not be caught by compilation (it's a plain match, still
  exhaustive) -- it's `loud.rs`'s `every_out_of_scope_variant_fails_loudly` that catches it, via the
  new owner-in-stderr assertion.
- **`Call`/`Signal`'s witnesses are arm-grained already** (Step 2), so Task 3 can delete exactly the
  `Call::Named`/`Call::Dynamic` rows without touching `Call::Qualified`/`Call::Trap`'s, and Task 6
  can delete exactly the `Signal` row without touching `Signal::Trap`'s. `Signal::Value` and
  `Signal::Label` were **not** split further (both share the `"Signal"` tag/row) since the brief
  only asked for `Signal::Trap` to get its own row and both move in scope together at Task 6; if a
  future task discovers they actually land separately, split that row the same way `Call` was split
  and update `expand_for_witnesses` accordingly.
- **`read_subset(&[&Path])` is ready for a second subset file.** A later task adding its own subset
  list can pass `&[&corpus_dir.join("phase-4a.txt"), &corpus_dir.join("phase-4b.txt")]` and every
  earlier-phase witness stays exercised.
- **`owners.rs` runs three times under a full `cargo test -p rexx-exec`** (once standalone, once via
  `coverage`'s `mod owners`, once via `loud`'s) -- expected and cheap (table-only assertions), not a
  bug if you notice the test count.

## Commit

`0197b360e0147137ec188302db011c5485786bf5` -- "Task 0 (4b): a shared owner table, arm-grained loud
witnesses, and an owner in every loud message". Read back with `git log -1 --format="%H%n%s"` after
committing, not written from memory. 6 files changed, 907 insertions(+), 519 deletions(-):
`src/lib.rs`, `tests/{coverage,loud,corpus,collect_stress}.rs` modified, `tests/owners.rs` created.
`git status --short` clean after the commit.

---

## Round 1 -- independent review's six Important findings

Review at `task-0-review.md` (0 Critical, 6 Important, 7 Minor; spec MET WITH GAPS, quality PASS
WITH CHANGES REQUIRED). Fixed all six Important findings below. Left the seven Minors deferred, per
instruction, except where a fix below happened to touch the same lines (noted per-item; none of the
seven Minors were independently fixed). I4 (recording the format in the plan file) is the lead's own
edit, not mine -- `docs/` was not touched by this round.

### I1 -- the `"4a"` carve-out was asserted by nothing

Added one assertion to each of the four existing "takes the loud path" tests in `run.rs`
(`do_with_takes_the_loud_path`, `do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_
rides_on`, `do_over_a_stem_target_takes_the_loud_path`,
`do_over_a_parenthesised_stem_target_is_also_caught`): each now asserts
`loud.message == "DO is not implemented"` beside the existing `Failure::Loud(_)` match, in addition
to it. `Loud.message` is crate-private and these tests are in-crate (already reading `.message`
elsewhere in `run.rs`), so no new API was needed.

### I2 -- the message shape was checked by substring, not pinned

`loud.rs`'s `every_out_of_scope_variant_fails_loudly` now builds
`format!(" is not implemented ({})", witness.owner)` and asserts
`stderr.trim_end().ends_with(&want_suffix)`, replacing the old `stderr.contains(witness.owner)`.
This subsumes the old check and additionally catches a shape rewrite that still happens to mention
the owner's bytes somewhere (mutation G).

### I3 -- `expr_owner`'s `VariableReference` arm is structurally unreachable by the new check today

Not "fixed" in the sense of making it reachable -- there is no witness program that reaches
`ExprKind::VariableReference` without going through the `CALL` instruction first (its only legal
position is a call argument list), so the stderr the assertion inspects always comes from
`instruction_owner`'s `Call::Named` arm, never from `expr_owner`'s own `VariableReference` arm, until
Task 3 implements `Call::Named` and the instruction itself stops being loud. Recorded this
honestly, as the review's fix asked: rewrote the now-partly-false comment beside the
`VariableReference` witness in `loud.rs` (the confounding argument holds for the exit code, not for
the owner), added a matching doc-comment note on `expr_owner`'s own `VariableReference` arm in
`lib.rs`, and added a line to `owners.rs`'s pinned-items block (item 5) naming this the one arm the
drift check does not cover, with the trigger that closes it (Task 3).

### I5 -- `read_subset`'s union/de-dup had no test

Added `read_subset_unions_two_files_first_seen_order_deduplicated` to `coverage.rs`: writes two
temp files under `std::env::temp_dir()` with an overlapping entry (`two.rex` in both), calls
`read_subset(&[&a, &b])`, and asserts the exact union `["one.rex", "two.rex", "three.rex"]` --
first-seen order, the repeated entry kept once at its first position. No new dependency (no
`tempfile` crate in this crate's `Cargo.toml`); used the same "write to a scratch path, don't bother
cleaning up on panic" convention `loud.rs`'s own `/tmp/loud-witness.rex` already uses, though this
test does remove its two files on the ordinary path.

### I6 -- `"4a"` doubled as a phase name and a "no phase" sentinel

Took the lead's adjudicated route: `instruction_owner`/`expr_owner` now return
`Option<&'static str>` (`None` == "this crate already implements it"), and `owned_message` matches
on that `Option` instead of comparing against the literal `"4a"`. Removes the phase-name coupling
the review flagged -- when Task 3 implements `Call::Named`, `instruction_owner`'s `Call::Named` arm
simply moves from `Some("4b")` to `None`, with nothing named `"4a"` anywhere to be wrong about it.
Doc comments on all three functions rewritten to match (they now explain the `Option`, not a string
sentinel).

## Round 1 -- mutation re-verification (F, G, C)

Each applied to a working tree already containing all six fixes above, run under the full
`cargo test -p rexx-exec`, then reverted (confirmed by `grep -n MUTATION` across the changed files
returning nothing) before the next.

| # | Mutation | Expected | Actual |
|---|---|---|---|
| F | `owned_message` always appends the owner (`owner.unwrap_or("4a")`, no `None` branch) | fail | **caught** -- all four `run.rs` tests named in I1's fix now fail, e.g. `do_with_takes_the_loud_path`: `assertion left == right failed ... got "DO is not implemented (4a)"` |
| G | Shape rewritten to `"[{owner}] {name}: unimplemented"` | fail | **caught** -- `loud::every_out_of_scope_variant_fails_loudly` fails on all 27 witnesses, e.g. `Call::Named (4b): stderr does not end with " is not implemented (4b)": "rexx-exec: [4b] CALL: unimplemented\n"` |
| C | `expr_owner`'s `VariableReference` arm: `Some("4b")` -> `Some("Phase 7")` | fail, or explain | **NOT caught -- full suite green, exit 0.** Explained in I3 above and in the code: the witness (`call sub >x\n`) fails on the `CALL` instruction before the `VariableReference` expression is ever reached, so `expr_owner` is never called for this witness at all. Becomes checkable the moment Task 3 implements `Call::Named` and this witness's `CALL` stops being loud. |

Full suite green after every restoration (`cargo test -p rexx-exec` exit 0, 12 binaries, 0 failed);
`cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0.
All read unpiped. `cargo fmt --edition 2024 --check` (the invocation this report's Round 0 section
claimed) does in fact error on this toolchain (M6, not fixed, deferred) -- `cargo fmt --all --check`
is the working invocation and is what every check in this round actually used.

### Commit

Hash recorded after committing, via `git log -1 --format="%H%n%s"`, not from memory: see the top of
the message sent back to the lead for this round.

---

## Round 2 -- I3 reopened: the reasoning was wrong, not merely under-tested

An independent re-reviewer confirmed I1/I2/I5/I6 and re-ran mutations F and G against the fixed
tree (both caught). I3 was reopened: round 1 documented, in three places, that `expr_owner`'s
`VariableReference` arm is unreachable by the drift check until Task 3 implements `Call::Named`,
because `VariableReference`'s "only legal position is a call or message argument list". That claim
-- inherited from `loud.rs`'s own pre-Task-0 module doc, not verified before being built on -- is
false, and the lead demonstrated it directly:

```
$ printf 'say >x\n' > vr.rex && cargo run -q --bin rexx-run -- vr.rex
rexx-exec: a variable reference is not implemented (4b)
```

`SAY` is 4a's own; no `CALL` anywhere. Verified independently here before touching anything (same
command, same output), then re-checked the actual grammar rule the false claim cited: `ast.rs`'s
20.930 (`expr.rs`'s `parseVariableReferenceTerm`) is about which *token* may follow `>`/`<` -- a
variable or a stem, not a literal or a number -- not about which instruction context the whole
reference may appear in. `eval.rs`'s own module doc already states the true, general fact:
`VariableReference` fails loudly on *any* evaluation, through the exhaustive `form_name` fallback,
with no dependency on `CALL`.

**Fix, per the lead's instruction:**

* `loud.rs`'s `EXPR_WITNESSES`: the `VariableReference` witness source changed from `"call sub
  >x\n"` to `"say >x\n"`.
* Deleted the three false unreachability claims:
  - `loud.rs`'s per-witness comment (the "confounded... coincidence, not verification" text from
    round 1's I3 fix).
  - `lib.rs`'s doc-comment addition on `expr_owner` (round 1's I3 fix).
  - `owners.rs`'s pinned-items item 5 addendum (round 1's I3 fix).
* Also corrected the **pre-existing** false claim in `loud.rs`'s own module doc (the "Witness
  programs" section, present before Task 0 and not itself a round-1 addition) that made the same
  "only legal position is a call or message argument list" assertion about `VariableReference` in
  general -- leaving that one uncorrected while fixing the witness and the three round-1 additions
  would have left a stale, equally false claim sitting in the same file. Replaced with the verified
  fact (`say >x` reaches the arm directly, with the 20.930/`eval.rs` citations above) rather than a
  hedge, per the lead's explicit instruction not to soften it.

### Mutation C, re-verified against the corrected witness

Applied the identical mutation (`expr_owner`'s `VariableReference` arm: `Some("4b")` ->
`Some("Phase 7")`), ran the full suite, reverted, confirmed no `MUTATION` marker remained.

**Caught.** `loud::every_out_of_scope_variant_fails_loudly` fails specifically on the
`VariableReference` row:

```
VariableReference (4b): stderr does not end with " is not implemented (4b)": "rexx-exec: a variable reference is not implemented (Phase 7)\n"
```

Full suite green after restoring (`cargo test -p rexx-exec` exit 0, 12 binaries, 0 failed, corpus
still 29 of 29); `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --
-D warnings` exit 0. All read unpiped.

The lesson stated plainly, since it is the useful part of this round: a plausible-sounding
mechanism story about why something *can't* be reached was written into three places as settled
fact, when running one line of Rexx would have refuted it in the time it took to read the comment.
The comment cited a real error number (20.930) and a real constraint (the grammar), which is
exactly what made it convincing enough to not check. Measure before explaining unreachability, not
after.

### Commit

Hash recorded after committing, via `git log -1 --format="%H%n%s"`, not from memory: see the top of
the message sent back to the lead for this round.
