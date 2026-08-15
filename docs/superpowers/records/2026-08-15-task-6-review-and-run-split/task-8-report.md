# Task 8 report: adopt timely-dataflow's clippy lint set

Commit `4c9f43658afd0f40cf6878a0703d0297b43b595a`, branch `plan/rust-rewrite`, read back with
`git log -1 --format=%H`. This amends `9fbafc54729f8990c3c54faf20a994d134c6591b`, whose message and
`allow` comments fix round 1 corrected; see section 8. The branch is unpushed, so the amend is safe
and the old hash exists nowhere else.

## 1. The lint set as fetched

Fetched 2026-08-15 with
`curl https://raw.githubusercontent.com/TimelyDataflow/timely-dataflow/master/Cargo.toml`, HTTP 200,
saved to the scratchpad as `timely-Cargo.toml`. The `[workspace.lints.clippy]` section holds 60
entries: 5 at `allow`, 54 at `warn`, and `as_conversions` present but commented out.

Allow-level: `type_complexity`, `option_map_unit_fn`, `wrong_self_convention`,
`should_implement_trait`, `module_inception`.

Warn-level: reproduced verbatim in `rust/Cargo.toml`. I did not retype the list into this report;
the file is the record, and a second copy here is a copy that can drift from it.

### Difference from the controller's reading

**One difference, and it is in the count, not the membership.** The brief says "42 of timely's
warn-level lints". The fetched file has **54** active warn-level entries plus the commented-out
`as_conversions`. Membership was compared programmatically rather than by eye: parsing both files'
`[workspace.lints.clippy]` sections gives 60 names on each side, empty set-difference in both
directions, and the only level difference is `as_conversions` (upstream commented out, ours an
explicit `allow`).

Whether the controller enabled a 42-lint subset or read a different revision I cannot tell from
here. It did not change the answer: my measurement over the full 55 (54 warn plus `as_conversions`
forced to `warn`) reproduced the controller's three non-zero counts exactly and found nothing else,
so the 12 extra lints contribute zero violations to this tree.

**`as_conversions` being commented out upstream matters for the record.** Moritz's decision was
"`as_conversions` is `allow`". I wrote it as an explicit `allow` rather than copying upstream's
comment-out, because an absent line is inherited-by-omission and a present one is a decision. The
comment says so.

## 2. My own violation counts, and the method

Method: `cargo clippy --workspace --all-targets --message-format=json`, into a file, from a fresh
`CARGO_TARGET_DIR` under the session scratchpad, with `as_conversions` and `shadow_unrelated`
temporarily set to `warn`. Counted with a Python script over the JSON lines, keying on
`message.code.code`. The script prints its total line count, its per-level histogram and a bucket for
diagnostics that carry no code, so an empty result and a broken parse look different from each other.

| lint | warnings | distinct source spans | files |
|---|---|---|---|
| `clippy::as_conversions` | 574 | 299 | 39 |
| `clippy::shadow_unrelated` | 380 | 314 | 40 |
| `clippy::needless_pass_by_ref_mut` | 20 | 10 | 5 |
| every other lint in the set | 0 | 0 | 0 |

Total diagnostics emitted: 974, all at level `warning`, all carrying a clippy code. Zero
`unknown_lints`, which is the evidence that every one of the 55 names is known to clippy 0.1.97
(rustc 1.97.1, `8bab26f4f`) rather than being silently ignored.

**The warning counts match the controller's exactly. The distinct-span counts are new and are the
more useful number, and this is a finding.** `--all-targets` lints a lib item once for the lib target
and again for its lib-test target, so a warning count over-reports the size of the job. The
multiplicity is not a clean doubling: `as_conversions` has 269 spans reported twice, 29 reported
once and one reported seven times; `shadow_unrelated` has 66 twice and 248 once. So "574 sites" would
have been a false statement. My first draft of the `allow` comments said exactly that, and it was
corrected before the commit. See the self-review section.

All crates opt into workspace lints: every one of the eight `crates/*/Cargo.toml` carries a `[lints]`
section with `workspace = true`. (Checked with `grep -A1 '^\[lints\]'`. A single-line
`grep 'lints.workspace'` returns *nothing* here, because the key is spelled over two lines. That is
the "a command that cannot match reads exactly like a command that found nothing" shape, and it
nearly became my finding that the workspace lints reach no crate at all.)

After the fixes: **zero** diagnostics of any code, from the same command.

## 3. The two `allow` comments, quoted

From `rust/Cargo.toml`, as they stand after fix round 1. Both gained the clause saying the
measurement was taken with the line set to `warn`, and `shadow_unrelated` gained the past tense; see
section 8, F2 and F3.

```toml
# Upstream carries this line commented out. Set explicitly here so the decision
# is recorded rather than inherited by omission.
#
# Measured 2026-08-15 with this line set to `warn`, by `cargo clippy --workspace
# --all-targets --message-format=json`, counted by `message.code.code`: 574
# warnings over 299 distinct source spans. Set the line back to `warn` before
# reproducing it; run against the tree as it stands, that command answers zero,
# which is the `allow` working and not the number having rotted.
#
# Each span is a truncation-and-sign judgement inside a numeric interpreter
# whose bar is byte-for-byte agreement with the C++ oracle, so a wrong rewrite
# is a silent behavioural change and not a build failure.
# What would close this: a checked cast helper in the shape of Materialize's
# `CastFrom`/`CastLossy`, which this tree does not have.
as_conversions = "allow"
```

```toml
# Upstream sets this to `warn`.
#
# Measured 2026-08-15 the same way, with this line set to `warn`: 380 warnings
# over 314 distinct source spans, in files then under active differential work.
# As of that date, clearing them meant a rename pass over exactly the code whose
# output was being compared byte for byte against the oracle: churn in the diffs
# that mattered most, and nothing else.
shadow_unrelated = "allow"
```

Both are dated, both name the command that produced the number, both say what state the file had to
be in for that command to produce it, and the `as_conversions` one names what would close it.

## 4. The `needless_pass_by_ref_mut` fixes

Ten sites were flagged. Eleven were fixed: narrowing the four `directive.rs` callees made
`Dir::dispatch`'s own `&mut ClauseCursor` unused-mutably, and clippy flagged it on the next pass.
That cascade is expected and stopped there: `parse_directive` genuinely calls `cursor.next()`.

Six `&mut self` to `&self`:

| site | old | new |
|---|---|---|
| `rust/crates/rexx-exec/src/clause.rs:552` | `fn leave_clause_without_boundary<T: ClauseValue>(&mut self, entry: ClauseEntry, ran: Result<T, Failure>)` | `&self` |
| `rust/crates/rexx-exec/src/eval.rs:805` | `fn arith_small_int(&mut self, op: Operator, left_value: ObjRef, right_value: ObjRef)` | `&self` |
| `rust/crates/rexx-exec/src/run.rs:2894` | `fn use_target_name(&mut self, code: &Code<'_>, target: &UseTarget)` | `&self` |
| `rust/crates/rexx-exec/src/run.rs:3403` | `fn novalue_check(&mut self, novalue: Novalue)` | `&self` |
| `rust/crates/rexx-exec/src/run.rs:4218` | `fn resolve_call(&mut self, name: &[u8], search_labels: bool)` | `&self` |
| `rust/crates/rexx-extract/src/bif.rs:867` | `fn assignment(&mut self, blank: &str)` | `&self` |

Five `cursor: &mut ClauseCursor` to `cursor: &ClauseCursor`, all in
`rust/crates/rexx-parse/src/directive.rs`, all on `impl<'a> Dir<'a>`:

| site | old | new |
|---|---|---|
| `:392` | `fn dispatch(&mut self, cursor: &mut ClauseCursor)` | `cursor: &ClauseCursor` |
| `:530` | `fn method(&mut self, cursor: &mut ClauseCursor)` | `cursor: &ClauseCursor` |
| `:636` | `fn attribute(&mut self, cursor: &mut ClauseCursor)` | `cursor: &ClauseCursor` |
| `:805` | `fn constant(&mut self, cursor: &mut ClauseCursor)` | `cursor: &ClauseCursor` |
| `:1175` | `fn routine(&mut self, cursor: &mut ClauseCursor)` | `cursor: &ClauseCursor` |

The four `Dir` callees only ever hand `cursor` to `check_directive(&self, cursor: &ClauseCursor, ..)`
and `has_body(&self, cursor: &ClauseCursor)`, both of which already took a shared reference.

**No body changed anywhere.** Every edit is a parameter type. No call site needed adjustment, which
the build confirms: relaxing `&mut T` to `&T` in a callee cannot break a caller that was already
producing a `&mut T`. Nothing needed a real change, so there was nothing to stop and report.

## 5. Step 5: proving the lints are in force

**Three lints, from four probes.** `dbg_macro`, `unused_async` and `todo` are evidence.
`zero_prefixed_literal` is not, and the reason is section 8's finding F1: it is warn-by-default, so
its red run says nothing about the table this task added. Corrected in fix round 1 after the review
ran the control I had not; the correction and the control are in section 8, and the rest of this
section is written as it now stands.

Each violation was appended to a real file after the file was copied to `scratchpad/step5-backup/`,
`cargo clippy --workspace --all-targets -- -D warnings` was run, and the file was restored **from
that copy** rather than with `git checkout --`. Restoration was verified by `md5sum` against the
pre-edit hashes; all four matched.

Each was also run **on its own**, because `-D warnings` aborts the build at the first failing crate
and a single combined run would only have proved whichever lint happened to be compiled first. A
combined `--message-format=json` run (below) shows all four firing simultaneously, and four separate
`-D warnings` runs give the four red exits.

**The control that separates the three from the fourth**, run by me on 2026-08-15 after the review:
delete the entire `[workspace.lints.clippy]` block from `rust/Cargo.toml`, apply all four probes, and
re-run. The block is what is on trial, so removing it is the only thing that distinguishes "this lint
fired" from "this lint fired **because of this table**".

```
CONTROL (no [workspace.lints.clippy] block), diagnostics:
  2 ('clippy::zero_prefixed_literal', 'crates/rexx-num/src/lib.rs:1077')
```

That is the whole output. `dbg_macro`, `unused_async` and `todo` emit nothing at all without the
block, and `zero_prefixed_literal` emits exactly what it emitted with it. Two `-D warnings` runs
under the same control confirm the exit statuses: the `07` probe alone exits **101**
(`= note: -D clippy::zero-prefixed-literal implied by -D warnings`), and the `dbg!` probe alone exits
**0**. Tree restored afterwards; `git diff --stat` empty.

**The rule this yields, which is the thing worth carrying forward: "it went red" is not the test,
"it went red because of this table" is.** A proof lint has to be one whose default level is `allow`.
Checking that is one control run and I did not do it.

### Why these three

* **`clippy::dbg_macro` (easy).** One token, one line, in a lib source. It is the control that the
  experiment itself works: if this had not gone red, nothing else in Step 5 would have meant anything.
* **`clippy::unused_async` (most expected to be silently inert).** There is no `async` anywhere in
  this workspace: `grep -rn async crates/ --include=*.rs` returns nothing. So this lint would never
  fire naturally, and if it were unreachable, no run would ever say so. It is the member of the set
  whose inertness is indistinguishable from its correctness. I placed it in an **integration test**
  (`crates/rexx-core/tests/heap.rs`), which additionally proves workspace lints reach a target kind
  that only `--all-targets` compiles.
* **`clippy::todo` (bench target).** Same argument one target kind further out. `rexx-bench`'s bench
  has `harness = false` and is the target most likely to be missed by a lint configuration.

### The fourth probe, and why it is not one of the three

* **`clippy::zero_prefixed_literal` (subtle, and disqualified).** I picked it because `07` is a valid
  decimal literal that compiles, runs, and looks like nothing at all, which in a numeric interpreter
  is the exact shape of a defect this project cannot afford. It went red. It is still not evidence:
  the lint is **warn-by-default**, so it fires with or without the table this task added, as the
  control above shows. The reason I picked it is precisely the reason it fails as a proof: a lint
  that catches something this important is a lint clippy already enables for everyone.

### The combined JSON run

```
2 ('clippy::dbg_macro', 'crates/rexx-exec/src/run.rs:9725')
1 ('clippy::todo', 'crates/rexx-bench/benches/interpreter.rs:58')
1 ('clippy::unused_async', 'crates/rexx-core/tests/heap.rs:60')
2 ('clippy::zero_prefixed_literal', 'crates/rexx-num/src/lib.rs:1077')
```

### The four red runs

The first, third and fourth are the evidence. The second is kept for the record because a reader who
finds `07` in the Step 5 history should find, at the same place, why it did not count.

`clippy::dbg_macro`, exit **101**:

```
error: the `dbg!` macro is intended as a debugging tool
    --> crates/rexx-exec/src/run.rs:9725:5
     |
9725 |     dbg!(v)
     |     ^^^^^^^
     = note: `-D clippy::dbg-macro` implied by `-D warnings`
error: could not compile `rexx-exec` (lib) due to 1 previous error
error: could not compile `rexx-exec` (lib test) due to 1 previous error
```

`clippy::zero_prefixed_literal`, exit **101** -- and **not evidence**, because the control run above
gets the identical output with `[workspace.lints.clippy]` deleted:

```
error: this is a decimal constant
    --> crates/rexx-num/src/lib.rs:1077:5
     |
1077 |     07
     |     ^^
     = note: `-D clippy::zero-prefixed-literal` implied by `-D warnings`
error: could not compile `rexx-num` (lib) due to 1 previous error
```

`clippy::unused_async`, exit **101**:

```
error: unused `async` for function with no await statements
  --> crates/rexx-core/tests/heap.rs:60:1
   |
60 | async fn clippy_probe_unused_async() {}
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: `-D clippy::unused-async` implied by `-D warnings`
error: could not compile `rexx-core` (test "heap") due to 1 previous error
```

`clippy::todo`, exit **101**:

```
error: `todo` should not be present in production code
  --> crates/rexx-bench/benches/interpreter.rs:58:5
   |
58 |     todo!()
   |     ^^^^^^^
   = note: `-D clippy::todo` implied by `-D warnings`
error: could not compile `rexx-bench` (bench "interpreter") due to 1 previous error
```

Each probe carried `#[allow(dead_code)]` so that `dead_code` could not be the thing that turned the
run red. The `= note:` line in each output names the lint that did.

## 6. Gates

All run from `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust`, unpiped, each exit status read on its
own, after the final state of the tree (that is, after the comment correction described in section 7).

| gate | exit |
|---|---|
| `cargo fmt --all --check` | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings`, `CARGO_TARGET_DIR` a freshly `rm -rf`'d scratchpad directory | **0** |
| `memcap 12G cargo test --release --workspace` | **0**, 1509 passed, 0 failed, 4 ignored |
| `REXX_CORPUS_GATE=1 memcap 12G cargo test --release --workspace` | **0**, 1509 passed, 0 failed, 4 ignored |

The cold-target clippy is the one the CLAUDE.md rule is about: `rm -rf` on the target directory, then
the full dependency graph rebuilt from scratch (`Compiling proc-macro2 ...` through
`Checking criterion`), then exit 0. Run twice on the final tree, both unpiped, both 0.

**After fix round 1** the only change was comment text inside `rust/Cargo.toml`. `cargo fmt --all
--check` and a fresh-target `cargo clippy --workspace --all-targets -- -D warnings` were re-run
against it, both exit **0**. The test gates were not re-run: the delta is TOML comment bytes, which
cargo hashes into a rebuild but which cannot change generated code, and the release profile carries
`lto = "fat"`. That is a judgement, not a measurement, and it is the one thing in this report that
rests on reasoning rather than a run.

### Both engines

| run | exit |
|---|---|
| `REXX_ENGINE=tree-walker REXX_CORPUS_GATE=1 memcap 12G cargo test --release --workspace` | **0**, 1509 passed, 0 failed, 4 ignored |
| `REXX_ENGINE=ir REXX_CORPUS_GATE=1 memcap 12G cargo test --release --workspace` | **0**, 1509 passed, 0 failed, 4 ignored |

### The corpus gate was run unfiltered, and it was observed

The gate run takes no `--test` or test-name filter, so it covers all three suites that key off
`REXX_CORPUS_GATE`: `corpus.rs`, `input_oracle.rs` and `parse_version_oracle.rs`.

The test totals are identical with and without the env var, because the gate flips REPORT into
STRICT rather than adding tests, and an identical pass count is exactly what a *skipped* gate would
also produce. So I checked it positively rather than inferring it:

* `corpus.rs` under `--nocapture` prints `mode: STRICT (the gate) -- REXX_CORPUS_GATE is set`.
* `input_oracle.rs` and `parse_version_oracle.rs` print no mode banner, so I A/B'd them. Without the
  var, their output contains
  `*** SKIPPED -- PARSE VERSION's string is not compared against the oracle unless REXX_CORPUS_GATE
  is set ***` and the corresponding unreadable-console and command-line-argument SKIPPED lines. With
  the var, those lines are gone and the oracle comparisons run. That diff is the witness.

Not run, and out of scope for this task's brief: `REXX_ASSERTIONS_GATE`, `REXX_BIF_GATE`,
`REXX_KEYWORD_GATE`. They are separate env vars, not name-filtered variants of this one.

## 7. Files changed, self-review, concerns

### Files changed

`git show --stat` on `4c9f43658afd0f40cf6878a0703d0297b43b595a`: 6 files, 97 insertions, 11 deletions.

* `rust/Cargo.toml` -- 86 insertions, **0 deletions**
* `rust/crates/rexx-exec/src/clause.rs` -- 1 line
* `rust/crates/rexx-exec/src/eval.rs` -- 1 line
* `rust/crates/rexx-exec/src/run.rs` -- 3 lines
* `rust/crates/rexx-extract/src/bif.rs` -- 1 line
* `rust/crates/rexx-parse/src/directive.rs` -- 5 lines

### Self-review: I reported one finding; the count was four

**What I said at the time.** One finding, from seven checks, corrected before the commit. That number
is below; it is left standing rather than rewritten, because the corrected count is the more useful
record and it belongs beside the wrong one.

**What the count actually was: four.** The review found three more, and all three are the same
species as the one I caught: a claim I could have falsified by running one command and instead
satisfied by reading. Section 8 has each in full. Restating the arithmetic here so this section is
not the optimistic one:

| # | finding | the check that would have caught it | did I run it |
|---|---|---|---|
| 1 | "574 sites" / "380 sites" were warning counts, not site counts | count distinct primary spans in the JSON, not lines | yes, caught it myself |
| 2 | `zero_prefixed_literal` proves nothing: warn-by-default | delete `[workspace.lints.clippy]`, re-run each probe | **no** |
| 3 | both `allow` comments cite a command that answers zero on the tree they sit in | run the comment's own command, verbatim, against the committed tree | **no** |
| 4 | three target kinds went unprobed: `bin`, `example`, `custom-build` | enumerate the target kinds `--all-targets` builds, then probe each | **no** |

The pattern across 2, 3 and 4 is one thing, not three. **Every one is a control I did not run, and
every one produced a result that looked like success.** Finding 2 is Step 5's own failure shape
turned on Step 5: my probe went red and read as evidence of adoption while being independent of it.
Finding 3 is a method that cannot reproduce its own number, which is indistinguishable from a number
that rotted. Finding 4 is coverage asserted from a sample: I probed lib, integration test and bench,
and inferred "no inert target kind" without enumerating the kinds. (The reviewer probed the three I
missed with `dbg_macro`; all three go red naming `-D clippy::dbg-macro`, so the conclusion held. It
held by luck, not by the check.)

I had written in section 5 that Step 5 is a control against a failure shape this plan has met four
times. It was, and I then reproduced the shape inside the control itself.

**Finding 1, as originally written.** My first draft of both `allow` comments said "574 sites" and "380 sites". Those are
warning counts, not site counts. I noticed while writing up the `needless_pass_by_ref_mut` fixes that
its "20 violations" were 10 distinct spans reported twice, went back to the JSON, and found
`as_conversions` is 299 distinct spans and `shadow_unrelated` is 314. Both comments now say
"N warnings over M distinct source spans", both numbers measured by me from the same JSON. Had this
shipped, it would have been two false statements in a file whose whole job is to be read years later
by someone deciding whether to turn the lint on.

I also considered explaining the discrepancy in the comment ("a lib item is linted once for the lib
target and again for its test target") and **deleted that sentence rather than shipping it**: the
multiplicity histogram is 269 twos, 29 ones and one seven for `as_conversions`, so the explanation is
partly false. The comments state the two measurements and no mechanism.

**What I checked to arrive at that count.**

1. `unsafe_code = "forbid"` and its comment are untouched. `git diff --numstat -- Cargo.toml` shows
   `81 0`: zero deleted lines in that file, so nothing above the new block was edited.
2. Lint membership against the fetched upstream file, by parsing both sections rather than by eye:
   60 names each side, empty difference both ways, one intentional level difference.
3. Every source edit is a parameter type. I read the full `git diff -- crates/` and confirmed no
   function body, no doc comment and no call site changed.
4. Comment hygiene: no em-dashes (`grep '—' Cargo.toml` finds none), no task or phase numbers, no
   history, no unmeasured number, both measurements dated and attributed to a named command.
5. The lint set actually reaches the code: section 5, four lints, four red runs, four restores
   verified by `md5sum`. **This is check 5 as I ran it, and it is where findings 2 and 4 hid:
   four red runs is not four proofs, and three target kinds is not every target kind.**
6. No scratch files left in the repository: `git status --short` shows only the six modified files.
7. Gates re-run on the *final* tree, after the comment correction, rather than reusing the green run
   from before it.

**The eighth check, which is the one that was missing: run every command a comment or a report
quotes, verbatim, against the tree as committed.** Check 4 read the comments for hygiene and check 5
ran the lints, but nothing executed the comments' own stated method. That single check catches
finding 3 outright and finding 2 as soon as it is applied to the probe rather than to the prose.

### Concerns

* **An untracked file appeared in the repository during this session that I did not write.**
  `docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md`, 47425 bytes, mtime 10:26. The
  branch was clean at the start of my session. I did not stage it, did not read past `ls`, and did
  not touch it. If another agent is live in this tree, that is worth knowing before the next task
  edits anything.
* **The brief's "42" is unexplained.** I could not reconcile it with the 54 warn-level entries I
  fetched, and I did not try to, per the instruction to report rather than reconcile. The three
  non-zero counts agree exactly, so nothing downstream depends on it, but if the controller
  hand-picked a 42-lint subset then the 12 lints outside that subset were adopted here on my
  measurement alone. My measurement covered all 55.
* ~~**`shadow_unrelated`'s `allow` comment contains one sentence that can rot**: "in files under
  active differential work".~~ **Closed in fix round 1** (M3). The substance stays, since it is
  Moritz's recorded reason, but the tense is now past and dated: "in files *then* under active
  differential work ... As of that date, clearing them *meant* ...". A dated past-tense claim about
  2026-08-15 cannot become false.
* **`needless_pass_by_ref_mut` is not a fixed point in one pass.** Narrowing a callee can expose its
  caller, as `Dir::dispatch` showed. Anyone re-running this after a refactor should iterate to zero
  rather than fixing the first list and stopping.
* **Nothing pins the lint set to upstream.** If timely adds a lint next month, nothing here notices.
  That is inherent to copying a list and is not a defect in this change, but it is the reason the
  block's header comment carries the fetch date.

## 8. Fix round 1

Review verdict **Approved** with two Important findings and two Minors. Both Importants and both
Minors are addressed. The commit was amended rather than followed by a fixup, because one of the
Importants is in the commit message itself and the branch is unpushed.

`4c9f43658afd0f40cf6878a0703d0297b43b595a` amends `9fbafc54729f8990c3c54faf20a994d134c6591b`. Read
back with `git log -1 --format='%H%n%s'`. Subject unchanged. Content of the tree changed only in
`rust/Cargo.toml`, and only in comments.

### What the review verified independently, which I am recording because it closes two of my concerns

* It re-fetched upstream and parsed both tables: 60 entries each side, empty set difference both
  ways, identical ordering, and it confirms **54** active warn-level entries against the brief's 42.
  My first concern in section 7 said the 42 was unexplained; it is now confirmed wrong rather than
  merely unreconciled, and the plan should carry 54.
* It reproduced the `Dir::dispatch` cascade: flagged spans at `:530`, `:636`, `:805`, `:1175` and
  **not** `:392`. Ten measured, eleven fixed, independently.
* It established the eleven narrowings cannot change behaviour: all are inherent impls, no `Deref` or
  `DerefMut` exists for the receivers, and every reference from outside the changed files is in a doc
  comment.

### F1 (review I1). `zero_prefixed_literal` is warn-by-default, so the live-lint proof is three

**Verified by me before accepting it**, since a finding is a claim like any other. With the entire
`[workspace.lints.clippy]` block deleted from `rust/Cargo.toml` and all four probes applied,
`cargo clippy --workspace --all-targets --message-format=json` over a fresh target directory emits
exactly one diagnostic:

```
CONTROL (no [workspace.lints.clippy] block), diagnostics:
  2 ('clippy::zero_prefixed_literal', 'crates/rexx-num/src/lib.rs:1077')
```

Under the same control, `-D warnings` with the `07` probe alone exits **101**, and with the `dbg!`
probe alone exits **0**. So `dbg_macro`, `unused_async` and `todo` are evidence of adoption and
`zero_prefixed_literal` is not.

Fixed in section 5, which now says three, carries the control output, and keeps the fourth probe
under a heading explaining why it does not count. Fixed in the commit message, which now names three
and says why the fourth is excluded.

**The generalisation, which is the part worth keeping:** a proof lint must have default level
`allow`. "It went red" is satisfied by any lint clippy already enables; only "it went red **because
of this table**" is the test, and the control that separates them is one run.

### F2 (review I2). Both `allow` comments cited a command that answers zero where they sit

The comments said "Measured 2026-08-15 by `cargo clippy --workspace --all-targets
--message-format=json` ... 574 warnings". Run verbatim against the tree those comments live in, that
command answers **0** `as_conversions` diagnostics, because the line directly beneath sets the lint
to `allow`. My report said the measurement needed both lines flipped to `warn`; the comments did not,
and the comments are what gets read.

The failure mode is specific and bad: a reader who follows the stated method gets nothing and cannot
tell **"the number rotted"** from **"the method was wrong"**. Both comments now name the required
state, and the `as_conversions` one names the zero answer explicitly so that getting it is a
confirmation rather than a puzzle:

> Measured 2026-08-15 **with this line set to `warn`**, by `cargo clippy ...`: 574 warnings over 299
> distinct source spans. Set the line back to `warn` before reproducing it; run against the tree as
> it stands, that command answers zero, which is the `allow` working and not the number having
> rotted.

The numbers themselves are unchanged and the review confirmed 299 and 314 independently.

### F3 (review M3). `shadow_unrelated`'s reason is now past tense and dated

"in files then under active differential work ... As of that date, clearing them meant a rename pass
over exactly the code whose output was being compared byte for byte against the oracle". Substance
kept, since it is Moritz's recorded reason. A dated past-tense claim cannot stop being true.

### F4 (review M4). Self-review count corrected from one to four

Section 7 now carries the four, the check that would have caught each, and the eighth check that was
missing from my list. I did not rewrite the original "one finding" sentence out of existence; it is
left in place with the corrected count beside it, because the useful record is that I audited my own
diff carefully and still reported a quarter of the defects.

### The review's own new finding, which I did not have: no inert target kind exists here

My three probes covered `lib`, `test` and `bench`. The review enumerated what `--all-targets`
actually builds and found `bin`, `example` and `custom-build` unprobed, then probed all three with
`dbg_macro`: all three go red naming `-D clippy::dbg-macro`. So workspace lints reach every target
kind in this workspace, which is now measured rather than assumed.

### Gates after fix round 1

The only change is comment text inside `rust/Cargo.toml`.

| gate | exit |
|---|---|
| `cargo fmt --all --check` | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings`, fresh `rm -rf`'d target dir | **0** |

Test gates not re-run; the reasoning, and the fact that it is reasoning rather than a run, is stated
at the end of section 6. Working tree after the amend: `git status --short` shows only the untracked
`docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md` that I did not write and did not
touch.

### Concerns after fix round 1

* **The brief's "42" is now confirmed wrong, not merely unreconciled.** Two independent fetches say
  54 active warn-level entries. That belongs in the plan, not only in two reports, or the next reader
  of the brief inherits it. Correcting the brief is the controller's to do; I am flagging it because
  the CLAUDE.md rule is to correct the plan rather than the message carrying the work.
* **Nothing in the tree keeps Step 5 honest.** The three proof lints and their control are recorded
  here and in the commit message, both of which are read once. If someone later sets one of the three
  to `allow`, or clippy promotes one to warn-by-default, no run anywhere notices. Making the proof a
  test rather than a transcript is out of scope for this task and is the obvious next thing.
