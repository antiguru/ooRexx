# Task 1 review -- the stem's slot, resolved once (`5eaa3c8e8`, record entry `73d470474`, comments `4bfabd00f`)

Reviewed against `task-1-brief.md`, `task-1-report.md` and `review-9513c8b13..4bfabd00f.diff`, with the two `2026-08-13-compound-name-resolution/` reviews as the prior.
Everything below was run by me, in detached worktrees at `4bfabd00f`, `5eaa3c8e8` and `9513c8b13` under private `CARGO_TARGET_DIR`s and a private backup directory, with `ootest`, `rust/corpus-l1` and `rust/target/debug/rexx-run` symlinked in.
No file in `/home/moritz/dev/repos/ooRexx-rust-rewrite` was edited except this review; every source mutation was applied inside a worktree, restored from a `cp -p` backup, `cmp`-verified, `touch`ed and rebuilt, and the suite was re-run at the restored final state.

## Verdicts

* **Spec compliance: PASS.** Every step of the plan was done as written, including the two the plan deliberately refuses to answer for the implementer -- the site count found by reading, and the `extra` question answered by running. I reproduced both, plus the shadow probe and both halves of the tripwire pair, and every number matched. Behaviour did not move: 1481/0/4 at the final state, all fifteen `compound-names` stanzas re-capture byte for byte against the oracle, and the six `Role::Loop` programs produce byte-identical stdout and stderr under both binaries on both engines.
* **Code quality: PASS WITH CHANGES.** The code is right and the shape is the right one. The changes wanted are all in comments: `4bfabd00f` corrected three statements of one false claim and left at least three more instances of the same claim standing, one of which is measurably false with the one-line probe the implementer had already built, and one of which this task wrote itself.

## Findings, most severe first

### 1. The comment correction is incomplete: three more instances of the same false claim survive, and one of them is measurably false

`4bfabd00f` corrected `PlanSlot`'s doc, `write_slot`'s doc and `assign_expr_target`'s doc. All three corrections are true, and I reproduced the measurement behind them (finding 6 of the checks below). But the same claim -- *a bare stem write has no slot* -- is still asserted in three more places, two of them in the same file as one of the corrections:

* `rust/crates/rexx-exec/src/run.rs`, `control_slot`'s doc: "**Simple spellings only.** A stem control assigns the whole stem by name and a compound one resolves a tail key afresh on every pass ... so **neither has a slot that could be resolved ahead of the pass that uses it**." The compound half is right for the reason it gives. The stem half is false, and false in the strongest way: `do cv. = 1 to 3` binds `CV.` through `Plan::bind` exactly as `zs. = 'one'` does, so a slot for it exists before the loop is entered. Measured with the report's own probe shape, `assign_expr_target`'s `Stem` arm instrumented to print both numbers, release, both engines:

  ```
  BARESTEM "CV." by_symbol=Some(0) slot_of=0      (four times, ir)
  BARESTEM "CV." by_symbol=Some(0) slot_of=0      (four times, tree-walker)
  ```

  `control_slot` returns `None` for `NameShape::Stem` by choice -- the same "left on the table" decision `4bfabd00f` wrote down for `write_slot` -- and its doc says the slot does not exist.
* `rust/crates/rexx-exec/src/run.rs`, `bind_control`'s doc: "a stem or compound control writes through a name rather than through a slot."
* `rust/crates/rexx-exec/src/run.rs`, `bind_control`'s `NameShape::Stem` arm: "`None`, and not `at`: a stem write is `stem_assign` under a name, **so there is no slot for it to be the slot of**." This is the corrected `assign_expr_target` sentence, verbatim in meaning, 3,800 lines further down the same file.

This is the "correction rounds add false statements / re-review the neighbourhood rather than the findings" failure mode in its exact shape: the fix landed on the three sentences the investigation had in hand, and the neighbourhood was not swept. All three survivors are in `run.rs`, the same file as one of the corrections, and `control_slot`/`bind_control` are a callee/caller pair, so one of them found is all of them found.

**Recommendation:** correct the three, and note in `control_slot` that a stem control's slot is not carried for the same reason `write_slot` does not carry one -- nothing downstream uses it yet -- rather than because it does not exist.

### 2. This task's own new comment repeats the claim it went on to correct

`stem.rs`, `stem_slot`'s doc, written by `5eaa3c8e8`:

> A bare stem's own operations do not come through here: `stem_assign` and `replace_stem` take the whole spelling of an `ExprKind::Stem` or of a run-time string, which is not a compound's stem half, and **no caller has a slot to hand them**.

The first half is exactly right and is the finding the task should state. The last clause reads as "there is no slot", which is the thing `4bfabd00f`, three commits later and by the same author, established is wrong. What is true is narrower: no caller *passes* one today, because `write_slot` records `UNRESOLVED` for a stem target and `control_slot` answers `None` for one. Lower severity than finding 1 because the sentence is ambiguous rather than plainly false, but it is the one instance this task authored, and it sits in the doc comment that exists to state this boundary.

### 3. Entry 30 and `5eaa3c8e8`'s commit message state a cause where the entry has only a bound

Entry 30, on `arith`:

> So this is **codegen drift** on a program the change cannot otherwise touch, and it is recorded as the cost side rather than explained. **No attribution beyond that is offered**: ... separating drift from layout needs a do-nothing control this entry did not build.

The two sentences disagree by one notch. If drift and layout are two candidate causes and the control that separates them was not built, then "this is codegen drift" is a cause and the entry is entitled only to "this is not the change's semantics". The evidence supports the bound completely -- `arith` holds no compound (I confirmed by reading `arith.rex` as well as by the probe), so the changed code cannot run on it -- and does not reach the cause. `5eaa3c8e8`'s commit message carries the same sentence ("so it is codegen drift and the cost side"), where it cannot be edited.

The report's own wording is the careful one and should have been the entry's: "**I offer no attribution beyond that**".

### 4. Step 4's table is a base-arm measurement, and the report does not say so

The three `extra` rows reproduce identically on **both** arms -- I ran them on an instrumented `9513c8b13` and an instrumented `4bfabd00f` and got the same names, slots and counts. The control row cannot have come from head:

| program | base `9513c8b13` | head `4bfabd00f` |
|---|---|---|
| `zi = 1; zb.zi = 5; say zb.zi` | `plan "ZB." slot 1` twice | **nothing at all** |

At head the compound never enters `slot_of`, which is the whole point of the change. So the table as printed is one arm's output presented without naming the arm. Nothing in it is false -- and base is arguably the right arm, since the question is about a resolution mechanism that predates the change -- but a reader cannot reproduce the control row from the commit the report is about. The head run is in fact the stronger control: silence is direct evidence the precomputed slot is being used, which is the one thing the suite cannot see (M2, below).

### 5. `ir/compile.rs`'s rewritten sentence keeps a set cardinality

`4bfabd00f` rewrote "the other two arms are not omissions" into "**the two other target shapes** differ from each other". `rust/CLAUDE.md:62` forbids a comment naming the size of a set. The cardinality was there before, so this neither introduces nor fixes it -- but the edit rewrote that exact sentence, which was the moment to drop it ("the other two target shapes" costs nothing). The rest of the diff is clean on this rule; `three-source` is pre-existing repo vocabulary (`Plan::slot_of`'s doc at base) and not a new instance.

### 6. Entry 30's non-comparability caveat is placed adequately, not ideally

The caveat is directly under the base-against-head bucket table, bolded, before the disposition -- a reader going top to bottom meets it in time. But the `8.61%`/`9.33%` figures it disclaims appear far earlier, under "Cause, stated by the plan before the task", with no forward pointer, and the reader who pairs "went up, 8.61% to 9.33%" with "head 5.34/4.68/5.24%" has done so before reaching the sentence saying not to. One clause at the point of first mention ("not comparable with the table below, different sitting and invocation") would close it. This is a note, not a defect: the entry does not itself make the comparison anywhere.

## The six checks

### 1. The site decision -- **verified, and the distinction is real**

Read `stem.rs` at `9513c8b13` myself. `slot_of(stem_name)` appears at exactly five sites, in these functions:

| line | function | disposition |
|---|---|---|
| 309 | `stem_get` | changed |
| 367 | `stem_set` | changed |
| 403 | `stem_drop_tail` | changed |
| 439 | `stem_assign` | left |
| 522 | `replace_stem` | left |

Five, three changed, two left -- the report's count, independently obtained. At head, two remain (lines 515 and 598), which is the same two.

The distinction is real, and it is a property of the callers rather than of appearance. Every non-test caller of the two left sites reaches them with a name that is not a compound's stem half: `assign_expr_target`'s `ExprKind::Stem` arm and `assign_by_name`'s `NameShape::Stem` arm (a whole `ExprKind::Stem` spelling or a run-time string), `drop_by_name`'s `NameShape::Stem`, `builtin/datatype.rs`'s `VALUE`, and `stem_drop`. None of them holds a `CompoundName`, so `stem_at` has nothing to say about them. Conversely, every caller that *does* hold an entry now passes the slot -- `code.stem(` has six call sites and five pass it on; the sixth is `echo_symbol_read`'s `>C>` line, which reads nothing and binds `_` with a comment saying so. No compound site was missed.

### 2. The `extra` answer -- **reproduced, and the shadow probe rebuilt from scratch**

`Interp::slot_of` instrumented to print which of its three sources answered, release, `REXX_ENGINE=ir`, fresh empty directory. All three of the report's `extra` shapes reproduce exactly, at head:

```
do za.zi = 1 to 3   ->  grow-into-extra "ZI" slot 1, grow-into-extra "ZA." slot 2,
                        then hit-extra for both, six times each
interpret "zq.1 = 7"->  grow-into-extra "ZQ." slot 0, then hit-extra "ZQ." slot 0
zn='ZR.1'; drop (zn)->  grow-into-extra "ZR." slot 1
```

So **yes, a stem can be bound in `extra`**, `stem_at`'s `Option` is required rather than defensive, and the `DO` control variable is not a hypothetical. The control row is finding 4 above.

The reason a *carried* slot cannot shadow an `extra` binding is structural and I checked it in the code: a slot lands on `stem_at` only through `Plan::slot_for`, which is the same call that inserts the name into `plan.names`, and `Interp::slot_of` reads `plan.names` before `extra`. The fragment hazard -- a fragment's local slot numbers meaning nothing in the enclosing frame -- is already closed by `Code::plan` being `None` for a fragment, and the doc comment on that field says exactly why.

Rebuilt the accessor-level shadow check myself, independently of the report's: `assert_eq!(slot, self.slot_of(stem_name))` in `stem_slot`'s `Some` arm, against the full three-source resolution, whole workspace, `--no-fail-fast`:

| run | result | report |
|---|---|---|
| `assert_eq!` | **1481 passed, 0 failed, 4 ignored, zero firings** | same |
| inverted to `assert_ne!` | **1446 passed, 35 failed, fires 35 times** | same |

The zero is a live zero. (The 35 failures are 34 distinct names; one name exists in two test binaries.)

### 3. Concern 1 -- **the measurement reproduces, and the three corrected comments are now true**

`assign_expr_target`'s `Stem` arm instrumented to print `code.slots.get(id)` beside `self.slot_of(name)`, on `zs. = 'one'; zi = 2; zs.zi = 'two'; zt. = 'x'`, release, **both** engines:

```
BARESTEM "ZS." by_symbol=Some(0) slot_of=0
BARESTEM "ZT." by_symbol=Some(2) slot_of=2
```

Exactly the report's numbers. The mechanism is in the code and not only in the measurement: `Plan::note`'s `ExprKind::Variable(id) | ExprKind::Stem(id)` arm calls `bind(id, name)`, `bind` calls `slot_for(name)` -- which is what inserts into `plan.names` -- and stores the same number in `by_symbol[id]`; `stem_assign` then calls `slot_of` on the identical spelling, which reads `plan.names` first. Both branches of `stem_assign` (`is_stem`, and `replace_stem`) then do `roots.set_slot(frame, slot, ...)`, so "writes the symbol's own slot" is literally right.

All three corrected comments are true as written. **The un-taken optimisation is real** -- see the closing answer.

### 4. The tripwire pair -- **reproduced exactly, and the contrast with Task 2 is not an artifact**

`note_compound_name` storing `slot_for(&entry.stem) + 1`, whole workspace, `--no-fail-fast`, `timeout 3000`, rebuilt after each restore:

| run | mine | report |
|---|---|---|
| M3, tripwire present | 1445 passed, **36 failures across 35 distinct names**, tripwire panicked **35** times | 36 / 35, 35 panics |
| M3', tripwire deleted | 1452 passed, **29 failures across 28 distinct names**, 0 panics | 29 / 28, 0 panics |

The seven names caught **only** because the tripwire is there are the same seven the report lists, exactly:

```
builtin::datatype::tests::symbol_reads_the_variable_pool_for_names_and_compounds
builtin::datatype::tests::var_answers_the_documented_example_line_for_line
run::tests::do_over_a_parenthesised_stem_target_is_also_caught
run::tests::do_over_a_stem_target_takes_the_loud_path
run::tests::drop_of_the_indirect_form_is_a_subsidiary_list_of_words
run::tests::novalue_fires_for_a_simple_variable_and_a_compound_but_not_a_bare_stem
run::tests::use_arg_alias_reports_the_kind_mismatch_before_the_uninitialised_target
```

Task 2's pair (its `task-2-report.md`, rows M4/M4') came out at 21 failures / 20 names with the tripwire and the **same 20 names** without. So the two results really are opposite, and there is a mechanism for it rather than an accident: a wrong *tail-piece* slot changes the tail key, so the read lands on a different tail and a different byte is printed every time; a wrong *stem* slot sends the read to a frame slot that is often simply empty, and an empty stem slot auto-vivifies and derives the name -- which is what several of those seven tests print anyway. The seven are bare-stem, `NOVALUE`-on-a-bare-stem, `DO`-over-a-stem and `USE ARG` alias tests, all cases where the stem's *contents* are not what reaches stdout. The claim survives.

I also reran the two rows whose "only" and "nothing" claims are load-bearing:

* **M1** (`note_compound_name` computes the slot and stores `None`): **1480 passed, 1 failed**, and the one is `plan::tests::build_records_a_compounds_split_under_the_compounds_own_id`. The "only" is exact, and it is what makes the change to that test load-bearing.
* **M2** (`stem_slot` ignores `at` and always resolves the name): **1481 passed, 0 failed**. Nothing in the workspace catches "the slot is not being used". The report says so plainly; it is correct, and it is the reason finding 4's head-arm silence is worth having.

### 5. Record entry 30 -- **appended, and every number checks out**

`git show --stat 73d470474`: one file, 86 insertions, **0 deletions**, hunk `@@ -2888,3 +2888,89 @@` -- a pure append at the end of the file. No earlier entry was touched. The rule entry 29 exists to restate held.

Arithmetic, against the entry's own table: `compound` -9.878% -> -9.88%; `alloc4c` -3.156% -> -3.16%; `rexxcps` -2.166% -> -2.17%; `arith` +12,040,137 = +0.059% -> +0.06%; the three sub-thousand figures subtract correctly; "2.746 / 0.560 / 0.267 billion" all check; "about eight thousand times its own 1,250-to-1,439 span" is 8,367x at the top of the span.

Re-measured independently: `perf stat -e instructions:u`, `REXX_ENGINE=ir`, three rounds interleaved base/head, both arms staged at one fixed binary path, from a fresh empty directory, minimum of each arm:

| axis | my base | my head | mine | entry's |
|---|---:|---:|---:|---:|
| `compound` | 27,797,834,521 | 25,053,534,461 | **-9.87%** | -9.88% |
| `alloc4c` | 8,456,952,605 | 8,189,894,924 | **-3.16%** | -3.16% |
| `rexxcps` | 25,858,278,687 | 25,298,105,407 | **-2.17%** | -2.17% |
| `arith` | 20,404,153,443 | 20,416,193,394 | **+0.059%** | +0.06% |

Every `rexxcps` run of both arms calibrated to `100 x 100`. My `arith` same-binary spans were 1,681 (base) and 712 (head), in line with the entry's 1,439/1,250, so the `arith` movement is thousands of times the instrument's resolution in a second sitting on a second set of binaries. My `alloc4c` base span was 11,970,833 -- two hundred times the entry's 58,490, from one cold first run -- which is a reminder that these spans are properties of a sitting and not of an axis; it does not touch any conclusion, since `alloc4c` moved 267 million.

`perf record -F 999` over `rexxcps`, one run per arm (a corroboration, not a replication of the entry's three): base `hash_one::<&[u8]>` 2.82% / SipHash `write` 2.62% / `slot_of` 1.30% / `hash_one::<&SymbolId>` 0.95%, bucket ~7.7%; head 1.72% / 1.81% / 1.14% / 0.66%, bucket ~5.3%. Inside the entry's ranges on both arms, and the bucket falls.

Also checked: the entry names head as `5eaa3c8e8` while the branch tip is `4bfabd00f`. That is right, and I verified the premise rather than assuming it -- release builds of the two commits have byte-identical `.text` and `.rodata` (`objcopy -O binary --only-section=` then `sha256sum`), so the comment-only commit measures the same.

Caveat placement is finding 6.

### 6. `arith` +0.06% -- **honest in substance, one notch overstated in wording**

Honest: the movement is reported rather than buried; it is called the cost side; the spread was measured **before** the figure was read rather than after; the "holds no compound" premise is measured rather than asserted (and I confirmed it independently, by reading `arith.rex`: every variable in it is simple, and no name in it carries a period); and the missing do-nothing control is named as the thing a later task should build. I reproduced the number to within 186 instructions on different binaries in a different sitting, which is stronger evidence than the report claims for it.

Overstated: it is written as a **cause** ("so this is codegen drift") in the entry and in the commit message, where the evidence supports only a **bound** (not the change's semantics; drift or layout, unseparated). See finding 3. The report's own "I offer no attribution beyond that" is the right sentence and did not make it into the repository document.

## The process disclosure

**Nothing was lost, and disclosing it was the right behaviour.**

`git checkout-index -f -- <path>` rewrites the working file from the **index**. The file had already been committed, so index and `HEAD` agreed, and the only delta the command could destroy was the temporary probe it was aimed at. `git status` empty immediately afterwards is consistent with that and is weak evidence on its own -- an empty status proves the end state matches, not that the discarded delta was only the probe -- but the two together, plus the fact that the command names a single path and cannot touch any other file, close it. No uncommitted work existed to lose.

The standing rule forbids `git checkout --`, and `checkout-index -f` is the same hazard class under a different spelling: it is a silent, unrecoverable overwrite with no witness, which is exactly why the rule prefers a backup copy whose restore can be `cmp`-checked. The right disposition is the one the report took: use the backup everywhere else (which it did), and when the rule is broken once, write it down in the report rather than leave it in a transcript nobody will read. I would not ask for anything further.

## Gates, reproduced at the final state

From `rust/`, in the worktree, after every mutation was restored and `cmp`-verified:

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, `Checking rexx-exec` in the log |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1481 passed, 0 failed, 4 ignored** |

Baseline before the task was 1480; the one new test is `plan::tests::a_stem_with_no_plan_slot_binds_in_extra`, and the three new case-file stanzas run inside the existing `both_engines_agree_on_every_case_file`.

Behaviour, independently of the suite:

* **All fifteen** stanzas of `tests/ir_dual_cases/compound-names` extracted and re-run under the oracle at this commit, wrapped `( ulimit -v 1048576; LD_LIBRARY_PATH=... )` from a fresh empty directory with absolute paths and `</dev/null`: **15 matched, 0 mismatched**, every one rc 0 with empty stderr. The three new transcripts are right, including the `say zs.` -> `dflt` line the report flags as the one it would have got wrong.
* `compound`, `alloc4c`, `arith`, `emptyloop`, `strings`, `varlookup` under base and head binaries on **both** engines: stdout and stderr byte-identical across arms in all twelve pairs, rc 0 throughout.
* Step 6's eight behaviours are all present in the case file: the three new stanzas cover the stem read, the stem write, the bare stem, `DROP` of a stem and the `INTERPRET`; `PROCEDURE EXPOSE` is the second new one; and the changing-tail and compound-`DO`-control stanzas were already there (lines 87 and 188) and were correctly not duplicated.

## Is the bare-stem-write optimisation real?

**Yes.** `Plan::bind` puts an `ExprKind::Stem` symbol's own spelling into `plan.names` and its own id into `by_symbol`, both on one slot; `stem_assign` then hashes that same spelling through `slot_of` to arrive at that same number, on every bare stem write, on both engines. Measured above: `by_symbol=Some(0) / slot_of=0` and `by_symbol=Some(2) / slot_of=2`.

It applies in **three** places, not one, and the third is not in the report:

1. `assign_expr_target`'s `ExprKind::Stem` arm -- the slot is already a parameter (`at`), and `write_slot` sets `UNRESOLVED` rather than filling it in.
2. `assign_by_name`'s `NameShape::Stem` arm and `drop_by_name`'s -- run-time names, no slot available, correctly out of scope.
3. **`bind_control`'s `NameShape::Stem` arm**, reached on every pass of `do cv. = 1 to 3`. `control_slot` answers `None` for a stem by choice, and I measured the slot it could have answered: `by_symbol=Some(0)`, four writes in a three-pass loop, both engines. This one pays per iteration, not per statement.

Not taking it in this task was right -- it is a different path, it wants its own measurement, and folding it into a change whose whole claim is that behaviour did not move would have been wrong. `rexxcps` writes compound tails rather than bare stems, so it would not have shown in these numbers either way; the axis that would show it is a loop with a stem control variable, which no bench program currently has.
