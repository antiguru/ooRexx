# Phase 4c final whole-branch review

Reviewer: final gate, 2026-08-07.
Range `d73a5678..64ce92fe`, 85 commits, 81 files, +31,081 / -1,948.
Scope: cross-task coherence, the honesty of the gate document, triage of the deferred minors, and prose-vs-behaviour drift.
Per-task diffs were deliberately not re-read.

## Verdict

**NOT READY to close the phase.**

No behavioural defect was found, and every headline figure the gate quotes reproduces at head except one.
The blocking problems are in the permanent record: three of the twelve criteria state a property the instruments behind them do not have, one measured divergence is missing from the exclusions register, and one exempt file carries a whitelist that can absorb a future value divergence with no phase owning it.
All of them are cheap to fix -- prose plus two one-line assertions -- and none requires re-running a task.

Finding count: **0 Critical, 10 Important, 25 Minor.**

Fifteen of the Minor findings are one class: **prose-vs-behaviour drift**, a comment true when written and falsified by a later task. That is the class the brief predicted would dominate, and it did.

## What I reproduced myself

Every command run from `rust/`, every exit status read unpiped, stdout and stderr as separate descriptors.

| measurement | expected | measured | status |
|---|---|---|---|
| `cargo test --workspace --no-fail-fast` | 1,289 passed / 0 failed, 75 binaries, exit 0 | 1,289 / 0, 75 process headers (76 `test result:` lines), exit 0 | reproduces |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 50 of 50 | 50 of 50, exit 0 | reproduces |
| `REXX_KEYWORD_GATE=1 ... --test keyword_assertions` | 888 of 896 bodies, 1,737 of 1,773 calls | identical, exit 0 | reproduces |
| `REXX_ASSERTIONS_GATE=1 ... --test assertions` | 4,224 of 4,259 | identical, exit 0 | reproduces |
| `... --test bif_assertions` (ungated) | 4,920 of 4,999 value, 184 of 186 raise, 6,293 calls | identical, exit 0 | reproduces |
| `builtin-status.txt` | 81 rows, 66 implemented, 15 excluded, 0 loud, 0 divergent | identical | reproduces |
| trace prefix coverage | 16 witnessed, 3 owned, sum = 19 | `WITNESSED_PREFIX_COUNT` 16, `OUT_OF_SCOPE_PREFIX_COUNT` 3, sum asserted against `support::TRACE_PREFIXES` | reproduces |
| `cargo fmt --all --check` | clean | exit 0 | reproduces |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean | 61 crates checked from a genuinely fresh `CARGO_TARGET_DIR`, 0 warnings, exit 0 | reproduces |
| `unsafe` | none | zero `unsafe` blocks; the seven hits are prose in comments | reproduces |
| em-dashes in Rust comments | none | zero in `crates/`, zero in `mutate-4c.sh` | reproduces |
| `bash scripts/mutate-4c.sh` | 9 of 9 as declared | **9 of 9 as declared, exit 0**, both baselines 50 of 50 and 75 binaries / **1,289** passed | reproduces, figure moved |
| D11 (no `RANDOM`/`DATE`/`TIME` in a corpus program) | zero violations | zero: all nine whole-word hits across `corpus/lang/*.rex` are English inside `/* */`; zero call-shaped hits | reproduces |

The clean-target clippy matters: a first attempt reported "Checking" for only 3 of 8 crates in 3.28 s, which is the exact warm-cache shape `rust/CLAUDE.md` warns about.
The authoritative run used an `rm -rf`'d directory and checked 61 crates.

The mutation re-run left the tree byte-identical (`sha256sum -c` over all six mutated files: OK; `git status --short` empty).

---

## Important findings

### I1. Criterion 11 claims a derivation the harness deliberately does not perform, for 15 of its 81 rows

`docs/superpowers/plans/phase-4c-gate.md:167` and `:331`.

The criterion reads "one row per name in `NAMES`, **every row** derived by running that name's probe through **both** interpreters", and the assessment repeats "**Every row** is derived by running that name's probe through both interpreters, so the file cannot drift ahead of the executor or lag behind it."

`crates/rexx-exec/tests/builtin_status.rs:335-339` does the opposite for the excluded names:

```rust
for name in rexx_inventory::builtins::NAMES {
    if whole.contains(name) {
        derived.push(((*name).to_string(), Status::Excluded));
        continue;
    }
```

The harness is honest about it -- its module doc at `:34-37` says "Nothing is run for it: there is no probe and no oracle invocation" -- and `:492-504` *asserts* `oracle_invocations == in_scope.len() == 66` with the message "no excluded name may run anything".
So the code enforces the negation of what the criterion claims.

**Failure scenario.** Phase 7 implements `LINES`. `corpus/builtin-status.txt:76` still reads `excluded`, because that row is pushed from the static `wholly_excluded()` list and no probe is ever run for it. `the_status_file_matches_a_live_differential_run` stays green. Meanwhile the seven `LINES` rows in `bif-exempt.txt` start passing and go red, pointing the reader at the wrong file. The "cannot drift ahead of the executor" property the criterion sells is exactly what fails, for exactly the 15 rows the criterion says it covers.

The fix is one sentence: say the criterion is a live differential over the 66 in-scope names and a name-table assertion over the 15 excluded ones.

### I2. Criterion 5's stated falsification does not hold -- measured, not argued

`docs/superpowers/plans/phase-4c-gate.md:102`: "*Falsification:* removing the phase from a loud message, **or letting an unimplemented arm run rather than exit loudly**, fails `every_out_of_scope_variant_fails_loudly`".

I applied exactly that mutation and it does not.
`crates/rexx-exec/src/eval.rs:372`, `_ => Err(Loud::expression(&expr.kind).into())` changed to `_ => Ok(self.text(b""))`, which makes every environment symbol beyond `.NIL`/`.TRUE`/`.FALSE` silently answer the null string instead of exiting loudly:

```
cargo test -p rexx-exec --test loud --test owners   ->  exit 0, 8 passed, 5 passed
```

Criterion 5's own two instruments stay green.
The cause is structural, not incidental: `loud.rs:378-382` derives its witness set from `owners.rs`'s `Owner::Phase(_)` rows, and `owners.rs:220` gives `DotVariable` a single whole-variant `Owner::InScope` row. The variant *is* split -- `eval.rs:358-360` states the split in prose and `:365-373` implements it -- so this is a violation of criterion 5's own "at **arm** granularity where a variant is split" clause. Adding a witness row for it would panic in `table_owner` (`loud.rs:155-161`), so the criterion's instrument is not merely blind to this arm, it forbids covering it.

Workspace-wide the behaviour *is* covered: with `--no-fail-fast`, 1,287 passed / 2 failed, the catchers being `eval::tests::a_dot_variable_beyond_the_three_fails_loudly` and `bif_assertions::the_exempt_set_matches_the_current_failures` -- a unit test criterion 5 does not cite, and criterion 12's instrument.
So the defect is confined to the gate's claim. It is still the "witness that cannot fail" shape the gate says at `:13` it screened every criterion against, and it survived that screen.

Reachability is not theoretical: nine rows of `corpus/bif-exempt.txt` read `UNATTRIBUTED:an environment symbol` today.

### I3. Criterion 4's bar was lowered after measurement, while the gate presents its criteria as fixed in advance

`docs/superpowers/plans/phase-4c-gate.md:10-11` states the criteria "were fixed in `docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md` before this task ran anything, which is what makes them criteria rather than descriptions of what happened."

The plan, at `2026-08-04-phase-4c-builtins-and-parse.md:1945`:

> The control must target a root the **builtin's own result** holds between allocation and the caller's use.

The gate, at `:74`:

> a negative control deleting a root a **builtin** holds makes at least one test fail.

These are materially different, and the weakening was made *because* the plan's version proved unsatisfiable: the gate at `:92-94` reports the finding that a builtin's own result holds no root, and substitutes `VALUE(name, newvalue)`'s root over the old stem value.

The finding is genuine and well argued, and the substituted control fires (mutation row 9, reproduced at head, caught by exactly one test workspace-wide). The defect is that the gate does not say the criterion changed. As written, `:10-11` asserts of criterion 4 something that is false, and it is the sentence a reader relies on to distinguish a criterion from a post-hoc description.

Fix: state at criterion 4 that the plan's wording was found unsatisfiable and record the amended wording as an amendment, exactly as criterion 1 and criterion 4's own denominator pins are recorded.

### I4. `BEEP` is a measured divergence that no register row owns

`docs/superpowers/plans/phase-4-exclusions.txt` -- absent. `/bin/grep -ain beep` over that file returns nothing.

`BEEP` is recorded in the gate (`:346-349`, `:366`, `:373`) and in `corpus/bif-exempt.txt`'s header, and nowhere else. Measured on the oracle, `retc = beep(262, 1)` exits 0 and answers the null string; this crate raises 43.1. It is a routine of the interpreter's internal package (`InternalPackage.cpp:199`), so it is outside the 66 in-scope builtins *and* outside D4's fifteen exclusions -- nothing in Phase 4 owns it.

The exclusions register is the cross-phase artifact: it is what `builtin-status.txt:35-38` names as where a `divergent` status must have a `KNOWN GAP: <NAME>` row, and its own KNOWN GAPS header says **"ADDING a row needs no amendment: recording a divergence you have just measured should never require permission, or the incentive runs the wrong way and gaps go unrecorded."** There is no friction here to explain the omission.

**Failure scenario.** Phase 5 or Phase 7 planning enumerates open Phase 4 items from the register, as the register exists to allow. `BEEP` does not appear, so no phase picks it up. The gate document is a per-phase artifact the next phase need not read, and `builtin-status.txt`'s derived mechanism cannot reach `BEEP` because it is correctly not a builtin. A measured, reproducible behavioural divergence is dropped at the phase boundary with no mechanism able to notice.

### I5. The gate's provenance sentence is false, and criterion 6's recorded baseline no longer reproduces

`docs/superpowers/plans/phase-4c-gate.md:198`: "Every figure below was taken at commit `1c94e50b`, with the command shown, reading the exit status unpiped."

That is false, demonstrably, from the branch's own history. Commit `d0102460` ("Record criterion 4's pin, and **refresh the mutation baseline**") changed criterion 6's figure from 1,286 to 1,287 precisely because it had been re-measured on a later tree. Two further gate commits (`d0102460`, `64ce92fe`) added criterion 1's and criterion 4's pin paragraphs after `1c94e50b`.

And the figure is now stale again. Fix round 2 (`80ff9c1f`, `64ce92fe`) added two tests. The gate at `:254` records "75 test binaries, 1,287 passed, 0 failed, both times"; re-running `scripts/mutate-4c.sh` at head gives **75 binaries, 1,289 passed, 0 failed** at both baselines.

**Failure scenario.** A Phase 5 reader reproducing the gate checks out `1c94e50b`, runs the mutation script, and gets a number that matches neither the gate nor head; or runs at head and concludes the record is wrong about more than it is. The substance is fine -- I re-ran the whole script at head and got **9 of 9 as declared, exit 0** -- but the record does not say which tree any figure belongs to.

Fix: replace `:198` with a per-figure commit, or restate the figures at head. The cheapest correct action is to update `:254` to 1,289 and change `:198` to name the commits the criteria were measured at.

### I6. `MISMATCH` is a whitelisted exempt attribution with no rows, no documented rule, and no zero-pin

`crates/rexx-exec/tests/bif_assertions.rs:821`:

```rust
const DERIVED: &[&str] = &["MISMATCH", "RAISE-MISMATCH", "ANOMALY"];
```

`every_exempt_attribution_is_a_known_phase_or_a_declared_outcome` accepts any of the three. `corpus/bif-exempt.txt` today carries 47 `4c`, 22 `Phase 5`, 9 `UNATTRIBUTED:an environment symbol` and 3 `ANOMALY` -- **zero `MISMATCH`**. The file's header documents `Phase 5`, `UNATTRIBUTED`, `ANOMALY` and the `4c` limits; it documents **no `MISMATCH` category at all**.

`MISMATCH` is `RowOutcome::Mismatch`'s attribution (`:262`): a value row whose two renderings both produced output and **differ**. That is the single thing the whole `base/bif` differential exists to detect.

**Failure scenario.** Phase 5's dispatch work changes a rendering path and `LENGTH::test_3#1` starts answering `12` where the oracle answers `12 `. `the_exempt_set_matches_the_current_failures` goes red with "is failing (MISMATCH) and is not on the committed exempt list". The obvious repair is to add the row with attribution `MISMATCH`; `every_exempt_attribution_is_a_known_phase_or_a_declared_outcome` accepts it; the suite is green again; no phase owns the divergence; the headline moves from 4,920 to 4,919 and **nothing asserts on 4,920**. Contrast every other attribution: `4c`, `Phase 5` and `Phase 7` name an owner, `UNATTRIBUTED:` names the construct, and `ANOMALY`'s three rows have their cause written into the header.

This is not hypothetical history: `task-15-review.md:78` records that the extractor produced **12 `MISMATCH` rows** before three drop reasons removed them. The category was live during this task; only the rows went away.

The fix is one assertion, using a device this phase already built one file over. `extract_bif.rs:219-224` pins each `DropReason` at its exact count *including the ones standing at zero*, with the reasoning written out: "A category pinned at zero fails loudly the first time the corpus grows one, which is exactly when a reader needs to know." Pin `MISMATCH` and `RAISE-MISMATCH` at zero in `bif-exempt.txt`, or drop them from `DERIVED` so a value divergence stays red until someone rules on it.

Note the same shape exists one file over and is also unenforced: `corpus/keyword-exempt.txt:61-64` documents `RAISED` and states a rule -- "A row carrying it needs its cause written into this header" -- with nothing checking it. That is Task 10's deferred M3, still unowned.

### I7. The exclusions register's containment argument for the DATE/TIME timezone gap rests on a false premise

`docs/superpowers/plans/phase-4-exclusions.txt:2180`, inside `KNOWN GAP: DATE/TIME READ THE WALL CLOCK IN UTC, NEVER THE HOST'S LOCAL ZONE`:

> D11 bars both builtins from every differential comparison this crate has, so nothing here can turn divergent

`DATE` and `TIME` **are** in a differential comparison. `corpus/builtin-probes.txt:91` and `:118` carry probes for both, and `crates/rexx-exec/tests/builtin_status.rs:226-239` runs each through `rexx_exec::run_program` **and** `oracle.run`, comparing all three descriptors. Both are recorded `implemented` in `builtin-status.txt` on the strength of that comparison.

D11's own scope is narrower than the sentence claims: its heading (`2026-08-04-phase-4c-builtins-and-parse.md:127`) and the operative rule (`corpus/README.md:85-91`) both say *corpus program*, and `builtin-probes.txt` is not one.

The conclusion still holds today, but for a reason the sentence does not give: the probes use conversion forms (`date('S','2026-08-04','I')`, `time('S','12:34:56','N')`) that never reach `real_clock_base_time`. `builtin-probes.txt:47-58` states that choice deliberately.

**Failure scenario.** Someone strengthens `builtin-probes.txt`'s `TIME` row to `say time('N')` on the reasonable ground that a conversion-only probe is weak. The row flips `divergent`, `builtin_status.rs` goes red, and the register -- the document a reader consults to understand whether this gap is contained -- says such a divergence cannot occur. This is the "false justification rides a correct decision" pattern: the ruling is right, the reason given for it is not.

Fix: replace the D11 clause with the real containment, which is that `builtin-probes.txt` chose conversion forms and states why.

### I8. `mutate-4c.sh`'s truncation guard can silently disable itself

`rust/scripts/mutate-4c.sh:143`, `:302`, `:370-373`.

Criterion 6 device (e) is "a **binary-count assertion measured from its own baseline**", and the gate at `:109` calls it "a measured defect in this phase rather than a precaution" -- it exists because `cargo test --workspace` truncates at the first failing binary, which invalidated three of Task 9's six uniqueness claims.

`BASELINE_BINARIES` is initialised to `0` at `:143` and used as a sentinel for "not yet measured":

```sh
if [ "${BASELINE_BINARIES}" -eq 0 ]; then
    BASELINE_BINARIES="${binaries}"
elif [ "${binaries}" -ne "${BASELINE_BINARIES}" ]; then
```

and the guard it feeds short-circuits on the same value (`:302`):

```sh
if [ "${expected_binaries}" -ne 0 ] && [ "${binaries}" -ne "${expected_binaries}" ]; then
```

`suite_binaries()` (`:266`) is `grep -cE '^ +(Running|Doc-tests) ' "${SUITE_STDERR}" || true`, so a measured zero and "no expectation" are the same value.

**Failure scenario.** Cargo changes its per-target banner -- the leading-space indentation, or the `Doc-tests` wording. `suite_binaries` returns 0 at the baseline. `BASELINE_BINARIES` stays 0. Every one of the nine rows then skips the truncation check entirely, because `expected_binaries` is 0. `total_run` is unaffected (it is read from `test result:` lines on **stdout**, which the banner change does not touch), so every row still classifies PASSED or DIVERGED normally and the script reports "9 of 9 as declared" at exit 0. Device (e) is off, and the exact defect it was written to close -- a mutation truncating the suite at its first catcher and being scored a clean catch -- is back with nothing red. The post-restore drift check at `:370-373` takes the same branch and never fires either.

The script does print `baseline suite ok (...): 0 binaries, ...`, so it is visible to a careful reader, but nothing asserts. One line fixes it: fail if the baseline measured zero binaries. Latent today -- the regex matches cargo's current output, and my re-run measured 75 -- but it turns live with no test going red.

### I9. The ledger's closing handoff names five deferred minors where roughly thirty-three are recorded

`.superpowers/sdd/2026-08-04-phase-4c-builtins-and-parse/progress.md:1449-1450`:

> **Phase 4c's fifteen tasks are done.** Remaining before the phase closes: the final whole-branch review, which owns triage of **the five deferred minors above**.

The ledger carries deferred-minor blocks at `:169`, `:246`, `:359`, `:462`, `:659`, `:784`, `:905`, `:979` (six items, M1-M6), `:1056`, `:1145`, `:1259` (five items) and `:1351`-`:1431` (five items) -- roughly thirty-three items, twelve of them under a heading reading verbatim "Deferred minors, **for the 4c final review**".

**Failure scenario, already realised.** This review was dispatched with a list of eight. Twenty-five recorded, explicitly-deferred items therefore reached the gate unowned, several of them substantive: Task 10's **M1** (`Loud::builtin_option_object`'s doc contract is false for one of its four uses), **M3** (the `RAISED` attribution's prose-only control, which is I6's twin), and **M5** (see Minor N1); Task 13's `run.rs:5569`/`:5585` wrong exclusion-list description and its unexplained 5.7 MB RSS discrepancy held open as a finding.

This is the mechanism, not the count, that matters: a phase whose handoff sentence undercounts its own open items closes with them silently absorbed. Fix: make `:1450` enumerate, or point at the headings.

### I10. The `Activation::nested` inheritance contract names three fields where the struct carries five, and both additions were made by 4c tasks

`crates/rexx-exec/src/activation.rs:647-648`, `:658`, `:730-731`.

```
/// The activation a `CALL` pushes: it starts at `pc`, and it **inherits**
/// `settings`, `trace_mode` and `traps` from the caller ...
/// All three inheritances are measured, and all are one-way ...
```

and forty lines on, justifying `cached_clock`:

```
/// and this is that same "start invalid" rather than a fourth
/// inheritance to add to the three above.
```

`struct Inherited` at `:797-803` has **five** fields: `settings`, `trace_mode`, **`address`**, `traps`, **`condition`**. `address` was added by 4c Task 9 (`27606888`), `condition` by 4c Task 10 (`f03d69f1`). The `cached_clock` sentence was written by 4c Task 12 (`7891fd4b`), by which point the struct already had five, so "a fourth ... to add to the three above" was false when written. The same doc block contradicts itself at `:678`, which says "none of the **five** fields this one copies".

**This is the cross-task intersection defect the brief describes.** Three separate 4c tasks touched one shared struct; two extended it and none updated the contract stating what it carries.

**Failure scenario.** Phase 5 adds the activation a message send pushes -- the obvious next constructor beside `nested`. Its author reads the contract, which says a called activation inherits `settings`, `trace_mode` and `traps`, and copies those three. `address` and `condition` are silently dropped, reintroducing exactly what Tasks 9 and 10 built: an `ADDRESS` set in the caller invisible to the callee, and `CONDITION()` inside a handler losing its trapped condition across the call. Nothing in the type system catches it -- `Inherited` is constructed field by field at each site -- and the corpus would catch it only where `address_env.rex` and `state_builtins.rex` happen to reach, which is `CALL` and `::ROUTINE`, not message sends.

Fix: state the property rather than the list, or derive the list. The struct is the enumeration and it is in this repository, so `rust/CLAUDE.md`'s rule applies: assert it or do not write it down.

---

## Minor findings

**N1. `rust/CLAUDE.md:57` -- a correction that re-rotted inside the rule forbidding it.**
The parenthetical reads "This rule's own illustration used to give that table's size as '796-row'. **It shrank to 16 rows at 4c Task 10**, which is the rule catching itself". `corpus/keyword-exempt.txt` holds **8** data rows -- Task 14 removed seven. The rule's remedy is "Assert the boundary or do not write it down"; the correction wrote a new number down instead of deleting it, and it went stale one task later. `builtin_status.rs:26` was fixed correctly, by removing the count. A future agent reading `CLAUDE.md` for the exempt file's size gets a wrong answer from the file that forbids stating it.

**N2. `rust/corpus/README.md:180-187` omits `routine_dispatch.rex`.**
The "Phase 4c additions" table has six rows; `corpus/phase-4c.txt` names eight. `parse_template.rex` is documented in the earlier table at `:116`; `routine_dispatch.rex` appears nowhere in the README. Nothing polices the table (Task 10's deferred M6, whose fix added `state_builtins.rex` and `address_env.rex` and missed this one). Concrete cost: mutation row 7's `DIVERGED`/`DIVERGED` declaration was corrected specifically because `routine_dispatch.rex` defines `::routine length` and `::routine 'max'`; a reader pruning apparently-undocumented corpus programs would silently remove the corpus half of that row's evidence.

**N3. Criterion 3's "16 of 19" does not distinguish committed from live witnesses.**
`trace_oracle.rs:615-634`: fourteen prefixes are `Coverage::Witnessed` (committed `.expected`), two are `Coverage::WitnessedLive` -- and those two are `>I>` and `<I<`, two of 4c's own three. The criterion's headline is "Every committed trace witness's stderr matches the oracle byte for byte"; two of 4c's three prefixes have no committed expectation at all and are checked only through the corpus, which the same criterion says normalises the space run. The code is honest (`:559-571` explains the host-dependence reason, and `every_live_witness_emits_its_prefix_and_is_run_by_the_corpus` pins it); the gate does not carry the distinction.

**N4. Criterion 10 under-claims a property it has.**
`keyword_assertions.rs:418-452`'s `the_exempt_set_matches_the_current_failures` never consults `gate_mode()`; it runs under a plain `cargo test`, exactly like its `base/bif` twin. Criterion 12 makes ungatedness explicit teeth (`:184-186`); criterion 10 says only "Both live in `keyword_assertions.rs`" and its assessment quotes only the `REXX_KEYWORD_GATE=1` command, so a reader comparing the two concludes the keyword one is gated. Under-claiming, not over-claiming, so Minor -- but it hides the stronger fact.

**N5. `mutate-4c.sh:132-135` -- the comment justifying `RUN_TIMEOUT` is arithmetically false.**
"Generous against the slowest observed clean run and still **far below the nine-minute hang** device 3 above describes." `RUN_TIMEOUT=900` is fifteen minutes, 66 % *above* nine. The cap still fires, because the Task 14 hang was genuinely unterminating (`progress.md:1321-1329`), but the sentence at the decision point asserts the opposite of the arithmetic, in the unsafe direction.

**N6. `mutate-4c.sh:171` -- `require_clean` does not check what its message says.**
`git diff --quiet -- "${MUTATED_FILES[@]}"` compares worktree to index, so a staged-but-uncommitted edit passes silently while the message says "has uncommitted changes ... commit or stash first". Consequence is mild (the `cp` backup captures and restores it), but the results would then describe a tree that is not `HEAD` while the gate attributes them to a commit.

**N7. `mutate-4c.sh:589` -- a failing post-restore baseline suppresses the failure list.**
`require_baseline_pass "after the last restore"` is unguarded and exits 1 on failure, so the `NOT AS DECLARED` summary at `:591-601` never prints and any row that missed its declaration is lost from the output.

**N8. Gate (d) is over-claimed.** `:107` says "`--no-fail-fast` on every run" and `:261` says "every run used `--no-fail-fast`". `run_suite` (`:256-260`) carries it; `run_corpus` (`:207-211`) does not. Materially harmless -- `--test corpus` selects a single target, so there is no later binary to truncate -- and the script's own header at `:100` scopes the claim correctly to the suite. Only the gate's restatement is wrong.

**N9. `2026-08-04-phase-4c-builtins-and-parse.md:1935` says "Four traps specific to 4c" above five bullets** (`:1937`, `:1938`, `:1939`, `:1942`, `:1946`).

**N10. The assessment table's row 12 drops its denominator.** See the triage of `:1359` below.

**N11. "Twelve confidently-wrong rows" conflates two numbers.** See the triage of `:1351` below.

**N12. `D11` names two different decisions.** `2026-07-27-rust-rewrite.md:289` is "D11 -- RexxUtil / `Sys*` functions"; `2026-08-04-phase-4c-builtins-and-parse.md:127` is "D11 -- no `RANDOM`, `DATE` or `TIME` in any differential corpus program". Both are live plan documents on this branch and both are cited by bare ID. `phase-4-exclusions.txt:2180` cites "D11" without a document.

**N13. `corpus/keyword-exempt.txt:61-64`'s `RAISED` rule is prose-only.** "A row carrying it needs its cause written into this header" -- nothing checks it. Task 10's deferred M3, unowned; I6's twin, and lower severity only because `RAISED` means "exited non-zero and named no construct" rather than "the two renderings differ".

**N14. `crates/rexx-exec/src/lib.rs:763-764` states a call-site count and the count is wrong.**
"...is the only reason this function exists rather than a bare `format!` at each of the **two** call sites." `owned_message` has **four** call sites, printed rather than counted: `lib.rs:472`, `:496`, `:541`, `:796`. Two of the four (`:541`, `:796`) always pass `Some(owner)` and therefore never reach the `None` carve-out the comment is arguing for, so the sentence makes its case about the wrong population as well as the wrong number. `rust/CLAUDE.md` forbids this shape by name -- "It may not say how many call sites there are ... If such a claim is load-bearing, assert it in a test; if it is not, delete it." Task 13's deferred minors flagged the class at `plan.rs:69` and `activation.rs:266` (both line references have since drifted); the class is still live and unowned.

**N15. `crates/rexx-exec/src/run.rs:8684` narrates a past mutation with a repo aggregate: "leaving all 102 pre-existing tests green."** Two forbidden shapes in one clause -- a mutable in-repo count, and history rather than what the code does. Whether 102 was accurate when written I did not establish; the point is that nothing can tell, which is the rule's reason for existing.

### N16-N25. Prose-vs-behaviour drift, verified false

A sweep of the 34 files this phase touched, each candidate then checked against the tree.
The right-hand column says whether I reproduced it myself or am reporting the sweep's measurement.

| # | file:line | claimed | measured | reproduced by me |
|---|---|---|---|---|
| N16 | `src/builtin/numeric.rs:1207` | "The **five** builtins that answer a number ... and the **two** that answer text" | four answer numbers (`ABS`, `SIGN`, `MAX`, `MIN`), three answer text (`FORMAT`, `TRUNC`, `RANDOM`) | no -- sweep; 4c commit `2472fd8a` moved `RANDOM` to text and fixed `numeric.rs:55-57` but not this line, so the doc contradicts its own module header |
| N17 | `src/builtin/mod.rs:89-90` | the `Run` signature "is spelled in **three** places -- `Builtin::run`, every implementation in `string.rs`, and the tests' stand-in" | seven implementation files spell it (`string.rs`, `word.rs`, `convert.rs`, `datatype.rs`, `numeric.rs`, `datetime.rs`, `state.rs`) | no -- sweep; written at `63a9ea9f`, when only `mod.rs` and `string.rs` existed |
| N18 | `tests/support/mod.rs:14`, `:21`, `:25` | "shared by `tests/corpus.rs` and `tests/trace_oracle.rs`"; "compared against on **two call sites**"; "`mod support;` in each of **the two** consuming files" | **six** files declare `mod support;` -- `corpus.rs`, `trace_oracle.rs`, `builtin_status.rs`, `state_builtin_oracle.rs`, `input_oracle.rs`, `parse_version_oracle.rs`; the four new ones are all 4c's | **yes** |
| N19 | `tests/support/mod.rs:105` | cites `trace_oracle.rs`'s `the_trace_surfaces_coverage_is_**thirteen**_of_nineteen_with_owners_for_the_rest` as what pins `PREFIX_COVERAGE` | the test is `..._**sixteen**_of_nineteen_...` (`trace_oracle.rs:738`); no `thirteen` spelling exists anywhere | **yes** -- and the stale number is criterion 3's own headline, one figure behind |
| N20 | `tests/collect_stress.rs:56`, `:60` | "**7 of the 29** subset programs panic ... all **29** pass again" | the subset this file reads is the 50-program union | **yes** -- and this is criterion 4's own file describing criterion 4's own subject; `f0426f09` rewrote the opening paragraph to say "the union of every `rust/corpus/phase-*.txt` file" and left the negative-control paragraph at 29 |
| N21 | `tests/coverage.rs:79-80` | "a builtin-named call still fails loudly, through `Loud::unresolved_call` naming `4c`" | `say length('abc')` prints `3` at rc 0; only the fifteen wholly-excluded builtins reach `unresolved_call`, as `lib.rs:500-517` now documents | **yes** -- true before Task 2, falsified by it, in a module doc criteria 1 and 4 both depend on |
| N22 | `src/run.rs:13182` | "with that line skipped ... the corpus drops to **48 of 49**" | the union is 50 | no -- sweep; written at `9b4ee92a` when the union was 49, with no commit pin, unlike `corpus.rs`'s dated rows |
| N23 | `src/run.rs:14473`, `:14475-14476` | "the oracle prints nothing for the **five** it also refuses"; "`::CLASS foo SUBCLASS object` is **the one** source here the oracle runs at rc 0 -- and `phase-4-exclusions.txt`'s directive section carries the argument" | the oracle refuses **seven** of the nine and runs **two** at rc 0: `::class foo subclass object` and `::options digits 12` | **yes** -- ran the test's own nine sources through the oracle from a fresh empty directory: rc 158/158/158/**0**/213/**0**/157/158/158. The cited register does carry the second case (`phase-4-exclusions.txt:453-456`, "::OPTIONS IS REFUSED FOR THE SAME REASON AND WITHOUT THE TRADE"), so the comment misstates the file it cites. The divergence itself is owned; only the comment is wrong |
| N24 | `src/run.rs:14318` | "the **eleven** silences above are pinned to the nesting" | eleven rows, but ten silences -- row 7 (`"an INTERPRET"`, `interpret "trace l"`, `"BOTH"`) announces | no -- sweep; off by one when written |
| N25 | `src/run.rs:3790` | names `current_clause_line_is_restored_after_a_nested_call_or_signal` as "what fails if this one line is ever removed" | no such test exists or ever existed; the real one is `current_clause_line_is_restored_after_a_nested_expression_**call**` (`run.rs:10773`) | no -- sweep; `git log -S` finds no commit that ever added the cited name. Pre-4c (`0fce4f00`), wrong from the start, and it is a claim about what protects a line of production code |

N19, N21, N22 and N25 are the sharper shape: a comment naming a **test** or a **figure** that does not exist. Those read as evidence and are not.

Separately, the sweep counted **479 comment lines** across these files matching task-number or history-narration patterns, roughly 60 of them 4c-authored, which `rust/CLAUDE.md` forbids ("No task numbers, no 'used to', no account of what moved or shrank"). I have not enumerated them: they are rule violations rather than false statements. They are worth naming because density is the mechanism -- N16's contradiction with its own module header and I10's three-against-five are hard to see inside comment blocks that long.

---

## Triage of the eight deferred minors

| ledger | subject | ruling |
|---|---|---|
| `:659` | Task 7 -- the "eleven operations that move them" count is unverified | **FIX NOW** |
| `:784` | Task 8 -- `ProgramInput::Bytes` is a public variant with no constructor | **ACCEPT** |
| `:905` | Task 9 -- `AddressState` derives `Debug`/`PartialEq`/`Eq` unnecessarily | **ACCEPT** |
| `:1351` | Task 15 -- "twelve confidently-wrong rows" conflates 12 with 18 | **FIX NOW** |
| `:1354` | Task 15 -- `MISMATCH` whitelisted as an exempt attribution | **FIX NOW** (promoted to Important, I6) |
| `:1359` | Task 15 -- assessment row 12 drops `of 6,293` | **FIX NOW** |
| `:1361` | Task 15 -- `has_novalue_option` misreads `::options novalue off` | **ACCEPT** |
| `:1431` | Task 15 -- a transcript reads 1,286 / 3 where the measurement is 1,285 / 4 | **ACCEPT** |

**`:659` FIX NOW.** `rust/CLAUDE.md` bans exactly this shape by name -- "It may not say how many call sites there are" -- and the ledger's own note flags the trap: "True today, which is the failure mode." Deleting the count costs nothing and the sentence survives without it; N1 above is what this class looks like when it is left standing and then "corrected".

**`:784` ACCEPT.** A `pub` enum variant with no constructor outside its own unit tests is dead surface, not a defect: nothing can observe a wrong answer through it, and Phase 7's file I/O is the obvious consumer. Removing it now would only have to be re-added.

**`:905` ACCEPT.** Measured by the task itself: removing the derives keeps `-p rexx-exec` green. Three unused trait impls on one struct cost nothing and cannot produce a wrong result; this is a style preference, and the review brief excludes those.

**`:1351` FIX NOW.** `task-15-review.md:78` measured it: "**18 rows come back** (`C2X` +1, `COMPARE` +1, `COPIES` +1, `DELSTR` +8, `INSERT` +1, `VALUE` +6) and **12 of them are MISMATCHes**." The gate at `:363` says "produced twelve confidently-wrong rows before three drop reasons were added", which a reader takes as the number of rows removed. The correct figure is checkable from the drop table (`extract_bif.rs:242-244`: `NonUtf8Source` 11 + `SideEffectingAssertion` 6 + `ClockDependent` 1 = 18), so the wrong one is checkably wrong. The review's own wording -- "eighteen rows, twelve of them reported as mismatches" -- is the fix. It also matters because those 12 are the *only* `MISMATCH` rows this project has ever produced, which is the evidence for I6.

**`:1359` FIX NOW.** The gate's §12 body is honest ("out of 6,293 `assertSame` calls", `:338`) and so is the inherits section ("the drop table's largest categories are Phase 5's: 418 calls ... and 467", `:375`). The assessment table at `:213` -- the one line a reader skims -- reads "4,920 of 4,999 value rows, 184 of 186 raise rows" with no hint that 1,294 of 6,293 calls were dropped by the extractor's own choices. That is precisely the "population the extractor itself chose" problem, in the highest-visibility row. One token restores it.

**`:1361` ACCEPT.** `::options novalue off` is absent at ooTest r13178 and the misreading costs coverage rather than correctness -- it would drop a body rather than resolve it in the direction that asserts something the interpreter must never produce. `OOTEST_REVISION` is pinned at `bif_assertions.rs:103`, so a suite bump is a deliberate act with an obvious place to re-check. Fixing it now would be speculative work against an input that does not exist.

**`:1431` ACCEPT.** The error is in an SDD report transcript, not shipped prose, and it is in the safe direction: it under-reports the number of independent catchers by one, omitting `collect_stress.rs`'s own round-1 pin. The ledger records the correct measurement at `:1422` ("exit 101, **1,285 passed / 4 failed**"), so the record as a whole is right and the correction is already where a reader looks.

---

## Cross-task coherence: what I checked and found sound

* **`ADDRESS` state across a `::ROUTINE` boundary** (Task 9 x Task 13, an intersection neither task owned). Probed against the oracle from a fresh directory -- environment set in the caller, set inside a routine, read back on both sides of the return. Byte-identical, rc 0 both sides.
* **Criterion 4's load-bearing premise, "the 4a/4b subset calls no builtin."** Verified by matching all 66 implemented names in call position against the 42 programs `phase-4a.txt` and `phase-4b.txt` name: zero code hits, four hits all inside `/* */`.
* **The 47 `4c` rows in `bif-exempt.txt` all name an excluded builtin.** Their groups are exactly `CHARIN CHAROUT CHARS LINES QUALIFY RXQUEUE STREAM`, all seven in `builtin-status.txt`'s fifteen `excluded`. The header's "EVERY `4c` ROW BELOW IS ONE OF THOSE" holds.
* **Criterion 2's "no row reads `4c`."** All 35 `EXEMPT` rows in `assertions.rs` read `"Phase 5"`.
* **Criterion 12's denominators are asserted, not merely reported.** `extract_bif.rs:211-216` pins `(calls, rows, raises, dropped) == (6293, 4999, 186, 1294)`, `the_row_floor` pins a real lower bound at 4,000, and `the_drop_reasons_account_for_every_call_outside_the_population` pins all seventeen reasons including the five standing at zero. The 4,920 numerator follows from those plus the both-directions exempt-set assertion.
* **Every test name the gate cites exists**, all 21 checked by name, each with exactly one definition (`the_falsification_proof`, `the_exempt_set_matches_the_current_failures` and `the_row_floor` have two, one per harness, as intended).
* **All nine `mutate-4c.sh` patterns still match their target file exactly once**, so the script is not stale, and the run at head confirms it.

## What I could not verify

* **Whether the four owner-less loud paths beyond `DotVariable` were a deliberate scope decision.** `lib.rs:567-571`, `:602-604` and `:641-643` each argue individually that `owned_message`'s shape belongs to the variant-keyed tables, so `compound_expose`, `builtin_option_object` and `value_selector` are knowingly outside criterion 5's grain. `DotVariable` carries no such argument anywhere, and it is the one the gate itself flags at `:374`. I2 is scoped to `DotVariable` for that reason.
* **Whether Task 15's mutation figures were ever taken at a tree matching `1c94e50b`.** I re-ran at head and the script reproduces; I did not check out `1c94e50b` to test the gate's own provenance claim directly.
* **Whether the 5.7 MB RSS discrepancy Task 13 held open (`progress.md:1270-1273`) has an explanation.** Out of scope for this review and still unowned; it is one of the twenty-five items I9 describes.
* **N16, N17, N22, N24 and N25 I did not reproduce myself.** They come from the drift sweep, which stated its method and its commands for each; I reproduced six of the eleven it reported, including every one where the measurement needed the oracle, and all six held exactly. I record which is which in the table rather than presenting them uniformly.
* **Whether `run.rs:8684`'s "all 102 pre-existing tests" was accurate when written.** Establishing it means building an old commit; the finding does not depend on the answer.

## A note on method

Two things in this review would have been reported wrong had I stopped one step earlier, and both are worth recording.

**The clean-target clippy.** My first run set `CARGO_TARGET_DIR` to a fresh path and reported exit 0 after checking three of eight crates in 3.28 s. That is the warm-cache shape `rust/CLAUDE.md` warns about, and it reads exactly like a pass. The authoritative run `rm -rf`'d the directory first and checked 61 crates. A green clippy is evidence only if the linter re-examined the code.

**My own probe of the directive sources (N23).** I first invented five directive spellings of my own and got rc 0 on four of them, which would have supported a much larger claim than the truth. Reading the test's actual nine sources and running those gave 158/158/158/0/213/0/157/158/158 -- a different and smaller finding. The failure mode was mine, not the code's: a probe that answers a question adjacent to the one asked.
