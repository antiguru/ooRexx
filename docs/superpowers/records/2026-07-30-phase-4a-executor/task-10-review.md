STATUS: DONE

# Task 10 review: IF, SELECT, SELECT CASE in the executor

Commits under review: `addf89b1` (implementation), `87ed65b6` (fix round).
Reviewer scope: spec compliance and code quality, per-finding severity Critical / Important / Minor.

## Findings

### Verified: `run_bounded` forwards a `Flow` it does not own, unchanged (the Task 11 hinge)

`run.rs`, `run_bounded`'s loop is exactly three arms:
`Flow::Next => pc += 1`, `Flow::Goto(target) if target >= start && target <= end => pc = target`, `other => return Ok(other)`.
The catch-all is the forwarding kind, not the swallowing kind: any future variant, `Flow::Exit`, and an out-of-range `Goto` all return unchanged.
Both callers (`If`'s true arm, `Select`'s matched-`WHEN` arm) likewise pattern-match `Flow::Next => Ok(Flow::Goto(resume))` and forward `other => Ok(other)` untouched.
No variant is matched by name and mishandled.
This is the shape Task 11 needs.
Confirmed by code inspection of `rust/crates/rexx-exec/src/run.rs` at `87ed65b6` (the working tree is at that commit, clean).

### Verified: temps-frame granularity is one frame per clause

`run_bounded` calls `step_in_temps_frame` once per instruction inside a `while` loop, never one frame around a branch.
`step` has exactly one non-test caller: `grep -n 'self\.step('` finds line 717 (`step_in_temps_frame`) plus three doc-comment/doctest occurrences (lines 84, 202, 221), which are prose, not code.
The unconditional `pop_frame` healing in `step_in_temps_frame` is intact.

### Verified: `record_failure_site` fires only on failing paths, innermost wins

Call sites: `step_in_temps_frame` calls it only under `if flow.is_err()`; `Select`'s arm calls it only inside the two explicit `Err(failure)` match arms of `eval_condition` / `test_case_when`, immediately before `return Err(failure)`.
No success path can reach it.
Innermost-wins holds structurally: the deepest `step_in_temps_frame` (or `Select`'s direct call for a `WHEN` condition) runs before any enclosing wrapper sees the propagating `Err`, and the `failure_site.is_none()` guard makes that first call the winner.
The entry point (`lib.rs:839`) consumes it with `.take()`, and each run builds a fresh `Interp`, so the first-wins guard cannot go stale within 4a.
Forward-looking note, not a defect today: if a later phase catches a `Raised` (condition traps) and continues executing, nothing clears `failure_site` mid-run, and a second failure would report the first failure's site. Worth remembering when SIGNAL ON lands.

### Important: `Flow::Goto`'s doc comment contradicts the code this same change wrote

`run.rs:63-70`, the variant doc says "a fragment can never jump -- `run_fragment`'s own `unreachable!` on this variant still holds."
Both halves are now false, and this very diff is what falsified them: `run_fragment`'s `unreachable!` was deleted (replaced by the `run_bounded` call), and `run_fragment`'s own updated doc comment says the opposite, "`IF`/`SELECT` need no label and can still appear and jump *inside* a fragment", which the implementer's report even calls a latent bug the old `unreachable!` would have been.
The only `unreachable!` left on this variant is in the test helper `run_source` (line 1440), which is not `run_fragment`.
A Task 11 implementer reading `Flow`'s doc first is invited to reintroduce exactly the assumption this task removed.
The fix is one sentence: the variant is unreachable via a *label* in a fragment (47.1), but live via `IF`/`SELECT` everywhere, fragments included, and `run_bounded` owns the in-range ones.

### Mutation testing: the discriminating tests bite (scratch copy, shared tree untouched)

Method: copied the tracked `rust/` tree to the scratchpad (`.../scratchpad/mut`), symlinked `interpreter/` for `rexx-inventory`'s build script, baseline `cargo test -q -p rexx-exec --lib` = 102 passed. Each mutation applied, tested, reverted; the scratch `run.rs` diffs clean against pristine afterwards.

* **M1**, `If` true path resumes on the `Else` instead of past it (the naive fallthrough): 3 failures, including `if_then_else_runs_exactly_one_branch` and `if_else_if_chain_takes_exactly_one_link`. The brief's first discriminating shape exists and bites.
* **M2**, matched `WHEN`'s exit lands right after its own body (on the next `WHEN`) instead of past `END`: 8 failures, including `select_when_wrong_exit_would_land_in_a_later_whens_multi_instruction_body`. The brief's second discriminating shape exists and bites, despite being rebuilt with nested `SELECT`s instead of `DO` (Task 11's).
* **M4**, drop `Select`'s direct `record_failure_site` for a failing `WHEN` condition: 2 failures (the two `WHEN`-attribution tests). The fix-round code is defended.
* **M5**, remove the first-call-wins guard (last-wins): 3 failures. The innermost-wins property is defended for the condition/value paths.
* **M6**, `run_bounded` swallows an unowned `Flow` and steps past it: 2 failures (the `EXIT` tests). Swallowing does not survive the suite.

### Important: a surviving mutant on the round-1 defect class, raise inside a branch *body*

**M7**: make `If`/`Select` pass `None` instead of `source` into their `run_bounded` calls. **All 102 tests pass.**
Under M7, a raise inside a matched `WHEN`'s `THEN` body (or an `IF` branch body) is attributed to the enclosing construct's clause again, which is precisely the third finding the implementer's own report describes fixing in round 1 (`say 1/0` inside a matched `WHEN` reported the `SELECT`'s line).
The five new attribution tests cover a `WHEN` condition, a second `WHEN` condition, the `SELECT CASE` expression, a `WhenCase` value, and an `OTHERWISE` body, but the `OTHERWISE` body never goes through a nested `run_bounded` (it runs in the outer loop), so none of the five exercises the `source`-threading that heals a *branch body*.
The behaviour as shipped is correct (my differential spot checks and the coordinator's five oracle cross-checks confirm it), but the suite cannot defend it: a regression that stops threading `source` into branch bodies passes 102 of 102.
Fix is one small test each for `select / when 1 = 1 then / say 1/0 / otherwise nop / end` and `if 1 = 1 then / say 1/0`, asserting `failure_site`'s line and text the way the five new tests already do.

### Verified: the written indentation characterisation matches the oracle, including the inferred rows

Six probes (`.../scratchpad/indent/`), each a raising `say 2 & 1` inside the construct, oracle invoked under the mandated ulimit; extra spaces beyond the one-space `*-* ` baseline:

| Shape | Oracle extra spaces | Report's rule predicts |
|---|---|---|
| top level | 0 | 0 |
| `SELECT`/`WHEN`/`THEN` | 6 | 6 (3 frames) |
| `SELECT CASE`/`WHEN`/`THEN` | 6 | flagged "not measured, expect same as plain `WHEN`", confirmed |
| `ELSE IF` chain (`else if 1 = 1 then <raise>`) | 8 | flagged "not stress-tested", rule predicts 2+2+2+2 frames = 8, confirmed |
| `IF`/`THEN` alone | 4 | 4 (2 frames) |
| `SELECT`/`OTHERWISE` | 4 | 4 (2 frames) |

Task 11 can implement from the report's rule as written; both rows the report honestly flagged as inferred check out, and additivity holds through an `ELSE IF` chain.

### Verified: differential spot checks on our side

`rexx-run` (built from the identical sources in the scratch copy) on the `SELECT`/`WHEN`/`THEN` and `ELSE IF` probes: line number, clause text, both error lines and rc=222 all match the oracle byte for byte, except the echo indentation, which is the known Task 11 gap and not a finding.

### Verified: the lower-priority items

* **`case` evaluated once**: by inspection, `Select`'s arm evaluates `case` exactly once into `case_text` before the `whens` loop; nothing inside the loop or `test_case_when` re-evaluates it. Not behaviourally probed, because the 4a subset has no side-effecting expression to observe a double evaluation with.
* **The absorbed `WHEN` leads nowhere undefined**: its `step` arm is a pure no-op, its `Then`/consequence sit outside the winning body's `[when_index + 1, false_target)` range by the implementer's own (hand-checked) trace, its permanently `None` `exit` is only ever read for members of `whens`, which it is not, and no path reaches it on the `OTHERWISE` route. M2's failures include both absorption tests, so wrong exits around it are caught.
* **The `step`-inside-`step` recursion**: documented as unmeasured in `lib.rs`'s fourth `INTERPRETER_STACK_BYTES` bullet, with the 2,000-level sanity check cited as reassurance and not a figure, exactly as instructed. No figure demanded.

### Minor: structuring semicolons in new comments

The global comment rule says no structuring semicolons; the new comments add several ("`run_bounded`'s doc comment carries the resolution; `Then`, ...", "A comma list checks itself; a single expression does not", "asked for and approved (`logical_value`); this way needs none", the `skip_else` doc, the `if_then_else` test doc).
Quoted catalogue text ("...\"1\"; found...") is exempt, being the oracle's own message.
The pre-Task-10 file already carries 18 such lines, so this is codebase-wide drift the task continued rather than introduced; flagged for consistency with the stated constraint, not as a regression.

### Minor: `failure_site` is never cleared mid-run

First-call-wins plus no reset means that once a later phase catches a `Raised` (condition traps, `SIGNAL ON`) and continues, a second failure would report the first failure's site.
Harmless today: the entry point `.take()`s it (`lib.rs:839`) and each run builds a fresh `Interp`.
Worth a note when traps land, nothing to change now.

### Global constraints check

* No `unsafe` anywhere in the diff (grep count 0).
* Diff touches only `rust/crates/rexx-exec/src/{run,eval,lib}.rs`; the C++ tree is untouched.
* No em-dashes in the touched files (`grep '—'` clean); the semicolon finding is above.
* Tests are a `#[cfg(test)] mod tests` inside `run.rs`, the subject's own file, per the rule.
* clippy/fmt/728 workspace tests/corpus 12 of 26: verified by the coordinator, not re-run here.

## Verdicts

**Spec compliance: PASS.**
Every brief requirement is implemented and defended or oracle-verified: both discriminating jump shapes exist and bite under mutation (M1, M2), the absorbed-`WHEN` case matches the oracle's measured `0`, `SELECT CASE` compares strictly (`'007'` vs `7`), the `WhenCase` comma is an OR of `==` while a plain `WHEN`'s is an AND raising 34.6, the 34.1/34.2-vs-34.6 split routes through `eval_logical_list` without re-checking, 7.3 raises from the `END`'s own arm so the echo is the `END`'s, and the fix round's attribution is correct and mutation-defended for conditions and values.
Indentation is deferred exactly as scoped, and the written rule Task 11 will inherit is oracle-correct including its inferred rows.

**Code quality: PASS with findings.**
The structural resolution is implemented as approved: `run_bounded` forwards unowned `Flow`s unchanged through a forwarding catch-all (the Task 11 hinge), temps-frame granularity is one frame per clause with `step` still having exactly one non-test caller, and `record_failure_site` fires only on error paths with innermost-wins.
Two Important findings: the stale `Flow::Goto` doc comment that contradicts this same change's own code, and the M7 surviving mutant showing branch-body attribution is undefended by the suite.
Both are cheap to fix and neither is a behaviour defect in what shipped.

## Not reached

* Did not re-run clippy, fmt, the 728 workspace tests, or the corpus count; the coordinator stated those verified and I spent the effort elsewhere.
* Did not behaviourally probe `case` single-evaluation against the oracle (no side-effecting expression exists in the 4a subset); inspection only.
* Did not mutation-test the `WhenCase` OR-vs-AND semantics directly (the dedicated test asserts both directions; M2 shows the surrounding machinery's tests bite).
* Did not probe indentation beyond two nesting levels or for `DO` (not implemented on this branch).
* Did not review commit messages or the `eval.rs` visibility bump beyond confirming it is exactly the approved one line.

## Re-review of fix round 3 (`87ed65b6..a9997e55`)

Scope: the 304-line single-commit diff addressing both Importants and the semicolon Minor.
Method: read the diff in full, re-ran the M7 mutation against the scratch copy synced to `a9997e55`, checked the two catalogue quotations against `interpreter/messages/rexxmsg.xml`, and hunted the reworded prose for false statements on the assumption the round broke something.
It did break something, three times, all in prose. No behavioural defect: every non-test change in this diff is a comment line.

### Verified: the M7 kill, at exactly the claimed precision

Scratch copy synced to `a9997e55`, baseline `cargo test -q -p rexx-exec --lib` = 104 passed.
Re-applied M7 (`source` to `None` at both `run_bounded` call sites, `If`'s at run.rs:502 and `Select`'s at run.rs:609): **2 failed, 102 passed**, and the two failures are exactly `a_raise_inside_a_matched_whens_body_is_attributed_to_its_own_clause` and `a_raise_inside_an_ifs_then_body_is_attributed_to_its_own_clause`.
The implementer's strong claim ("only these two went red") reproduces exactly. The tests are aimed at the mechanism, not merely sensitive.

### Verified: the two new tests assert line and text, and neither expected line is 1

`WHEN`-body test: line 3, text `say 1/0`. `IF`-body test: line 2, text `say 1/0`.
Both assert the text as well as the line, so a defaulted `(0, ...)` site or a right-line-wrong-clause site both fail.

### Verified: the two remaining semicolons are verbatim quotations

`raised_if_not_logical`'s doc quotes `rexxmsg.xml:2467` ("Value of expression following IF keyword must be exactly \"0\" or \"1\"; found ...") and `raised_select_no_when`'s quotes `rexxmsg.xml:267` ("All WHEN expressions of SELECT are false; OTHERWISE expected."), both byte-matching the catalogue `<Text>` modulo `<q>` markup.
The reasoning holds: these are quotations, not paraphrases, and their punctuation is the oracle's.

### Minor: the new `WHEN`-body test's doc comment contains a false statement

run.rs, the mutation-provenance sentence says "passing `None` in place of `source` at **both of `Select`'s own `run_bounded` call sites**".
`Select`'s arm has exactly one `run_bounded` call site (run.rs:609); the two sites the mutation touches are `If`'s (502) and `Select`'s (609).
The `IF` test's own comment gets it right ("`If`'s own `run_bounded` call site"), so the intended sentence was clearly "both `run_bounded` call sites, `If`'s and `Select`'s".
Same comment, second imprecision: the `OTHERWISE` test is said to run "never through `run_bounded` at all", which is true of the production path (`run_activation`'s outer loop) but not of the test harness, where `run_source` runs everything through its own top-level `run_bounded` with `source` supplied directly.
The accurate claim is "never through a `run_bounded` nested inside `If`/`Select`'s own arm", which is also the property M7 actually proves.

### Minor: the semicolon cleanup is off by two on its own terms

The rewritten `Flow::Goto` comment introduces a new structuring semicolon ("`run_fragment` no longer has an `unreachable!` on this variant; it now runs through `run_bounded`..."), added by the very commit doing the cleanup.
And one Task-10-authored non-quotation semicolon survives at run.rs:886, `test_case_when`'s doc ("asked for and approved (`logical_value`); this way needs none").
Neither is pre-existing drift, so neither is covered by the leave-the-sweep-for-later rationale.

### Minor: `Flow::Goto`'s new comment points its cross-reference at the wrong function

The parenthetical "(that function's own doc comment has the argument for why every jump such a construct computes stays inside the fragment's own range)" follows "it now runs through `run_bounded`", making `run_bounded` the natural antecedent.
The argument lives in `run_fragment`'s doc comment (run.rs:939, the only occurrence of the quoted phrase); `run_bounded`'s doc mentions fragments zero times.
One-word fix: name `run_fragment` explicitly.

### On the coordinator's two disposition calls

Agree with both.
Leaving the 18 pre-existing semicolon lines alone keeps this diff reviewable; a sweep belongs in its own commit if it happens at all.
Deferring `failure_site` clearing to 4b is right: today's only consumer `.take()`s it at the entry point, no caller can resume after a `Raised`, and clearing now would be scaffolding for code that does not exist.
Recording it against 4b (condition traps) is the correct home.

### Re-review verdict

Both Important findings are addressed and verified, the semicolon Minor is mostly addressed, and the round introduced no behavioural defect.
Three new Minors, all comment prose: one false statement and one imprecision in the `WHEN`-body test's doc, two structuring semicolons the cleanup itself owns, and one wrong cross-reference antecedent.
None blocks; all three are single-sentence fixes that could ride along with any later commit to this file.
