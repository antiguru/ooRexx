# Task 1 re-review 3: the third fix round

Re-reviewed: `367cc8d61` against `task-1-re-review-2.md`'s NN1-NN5.
Scope as dispatched: for each of the round's 7 hunks, did the deletion leave coherent text and lose nothing true and load-bearing; is the closed-set search complete (redone independently); did the round introduce any new claim.
Design, and everything the first two re-reviews closed, are out of scope. The phase-4a-executor.md trailing-clause deletion is approved and not relitigated, only checked for local coherence.

**Verdict: Task 1 closed.** All 7 hunks read as complete thoughts; my independent redo of the closed-set search found nothing the report's table does not already account for; the round introduced no new claim, mutation-backed or otherwise; the one deliberately-left item (`eval.rs:1090`) is listed in the report and untouched; the approved out-of-item edit reads correctly; gates are green at baseline (1468/0/4).

---

## 1. Per-hunk coherence, all 7

| # | file : hunk | reads as a complete thought | true/load-bearing content lost |
|---|---|---|---|
| 1 | `phase-4a-executor.md:739`, trailing clause of "Do not check a comma-list condition yourself" | yes -- "A comma list raises **34.6** from inside `eval_logical_list`, which already does it. Measured across all four keywords..." is a complete, self-supporting instruction | no -- the deleted clause ("re-checking the result would replace 34.6 with 34.1") was the false part; the surviving sentence's own measurement is untouched. Approved edit, not relitigated |
| 2 | `condition-promotion.md:82`, Step 1's `checked` sentence | yes -- "`checked` says whether the value arrives already validated as exactly `0`/`1`, in which case the answer is read back rather than checked again" stands alone and matches `run.rs`'s actual doc | no -- the deleted sentence's only true residue (that `checked` means a readback) is restated, generically and correctly |
| 3 | `condition-promotion.md`, Step 10's bullet + closing paragraph | yes -- the bullet now names the actual captured program (`if 1, 1 then say 'both'`), and the closing sentence points at `task-1-report.md` rather than asserting anything | **partial, intentional**: the deleted paragraph's true residue -- which specific rows were measured out vs. argued out, and on what mutation each was caught -- is no longer in the plan itself, only in `task-1-report.md`'s Fix round 1 section, which the new sentence points to by name. Same shape as round 1's N1 (golden_tests.rs uniqueness paragraph), already ruled acceptable there. Not a defect; named per the brief's request to flag any true content a deletion drops |
| 4 | `run.rs` `eval_condition` doc, the `if 0, 'x'` evidence clause | yes -- "stops at the first that is `0`, and otherwise answers `b"0"` or `b"1"` and nothing else" is a complete, checkable statement | no -- the deleted clause was the insufficient probe (rc 0 shows "never checked", not "never evaluated"); the true, stronger evidence for the same conclusion lives 30 lines away in `eval_logical_list`'s own doc, unedited |
| 5 | `run.rs` `condition_value` doc, "raised on any element" sentence | yes -- "`eval_logical_list` answers `b"0"` or `b"1"` and nothing else, so the check it skips is one that could not have failed" is complete and matches the function's actual `Ok` path | no -- the deleted quantifier was false for `if 0, 'x'`; the guarantee that survives (the return value, unconditional) is what the new sentence states |
| 6 | `run.rs` `condition_value`'s `if checked` arm, inline comment | yes -- `if checked { Ok(text == b"1") } else { ... }` needs no comment; nothing dangles | no -- the function's own doc comment above already explains the readback rationale; the inline comment was redundant as well as false |
| 7 | `tests/ir_dual_cases/conditions`, "the tail ... enters" + program name | yes -- "the tail a condition's value enters" reads correctly and is not refuted by the file's own first stanza the way "every condition" was | no -- pure wording/attribution fixes, nothing subtracted |

## 2. Closed-set search, redone independently

Searched at `367cc8d61` over `rust/crates`, `docs/`, and (live, since `.superpowers/` is entirely gitignored per `.gitignore:19` and therefore absent from any git commit or worktree) `.superpowers/`.

**Method note, worth recording on its own:** the interactive `grep` in this environment is a shell function wrapping `ugrep --ignore-files ... -I`, which silently returns **zero matches for the entire `.superpowers/` subtree** because that directory is gitignored -- not a binary-file skip, a whole-directory skip. `grep -rn "already validated" .superpowers` exits 1 (no output) even though the string is present in six files under it. Bypassing the wrapper (`\grep`, or `find ... | xargs \grep`) is required for any `.superpowers/` search to mean anything. Flagging this because it is the same shape as the recorded "`grep` skips binary files silently" trap, on a different trigger, and it would make a `.superpowers/`-scoped search silently vacuous for anyone who does not know to route around it.

Patterns run (case-insensitive): `already (validated|checked|raised|does it)`, `every element`, `re-?check`/`recheck`, `report(s|ing)? 34\.6 as 34\.1`, `eval_logical_list.{0,40}already`.

**Result: no occurrence found beyond the report's table.** Specifically:
* `rust/crates` and `docs/` (not gitignored, wrapper is reliable there, cross-checked against `\grep` with an empty diff): identical hit sets in both tools, all already itemised in the report's table (`run.rs:6903`, `:7002`, `:7422`, `:7477`, `:7496`; `eval.rs:1066`, `:1090`, `:2250`; `ir/mod.rs:785,961`; error/raiser rows at `run.rs:8930,9764,10254,11034`; `error.rs:361-372`; `2026-08-12-remaining-promotion-survey.md:26`; `phase-4a-gate.md`, `phase-4b-gate.md`, etc. -- none of these carry the false mechanism, all are either unrelated "already validated/checked" uses or true statements the table already names).
* `.superpowers/sdd/2026-08-12-condition-promotion/` (live, `\grep`-based): every live hit is one of the seven rows already in the report's closed-set table, plus the historical review/report/diff files the table's catch-all covers.
* One hit outside the condition-promotion directory: `.superpowers/sdd/2026-08-09-phase-4e-ir/task-4b-report.md:139`, "the answer is a Rexx logical value, which `eval_condition` has already validated as exactly `0` or [1]". Checked by hand: this is the **true** claim (about the returned value, not about which elements were checked), same shape as `run.rs:6903`, from an unrelated phase's report. Not the false mechanism, not something the table needed to name.
* `vm-comparison.md:841` spot-checked directly: it is explicitly framed as "`[read]` the brief's Step 1 is explicit that..." -- an attribution of what the (frozen, unedited) brief says, not an independent assertion. `task-1-brief.md:20` still literally contains that sentence, so the attribution is accurate. Matches the report's verdict.

No occurrence the table misses.

## 3. New claims introduced this round

Extracted every `+` line of the diff (7 lines of substantive new prose, one per hunk touched by an addition; hunk 6 is a pure deletion). None asserts what a mutation reddens or catches; each either (a) restates an existing, already-verified-true mechanism generically, (b) points to another document by name instead of restating a claim, or (c) is a factual correction of a program name / wording, not a new empirical claim. The one sentence in the tree that does claim a mutation result -- the `if 1, 1` stanza's "this row catches a second validating op emitted behind the fallback" -- is **unchanged text carried over** from before this round (present verbatim in both the removed and added versions of that hunk, just rewrapped); its measurement is round 2's R2MXB/R3MXB, reproduced independently in `task-1-re-review-2.md` as R3B. The report's own "Fix round 3" section states this explicitly ("No mutation was re-run this round: nothing this round wrote makes a claim about what a mutation reddens"), and the diff confirms it.

## Item 3 (deliberately not fixed)

`eval.rs:1090`, F4's note ("every element traces its own `>>>`, under `TRACE R` alone") is listed in `task-1-report.md`'s closed-set table and "Not fixed" section, and is untouched by this diff (`eval.rs` does not appear in the diff's file list at all). Confirmed present, unedited, and its subject (the trace setting) is what it measures and gets right, per the accepted framing.

## Gates

From `rust/`, each status read unpiped:

* `cargo fmt --all --check` -- exit 0
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0
* `memcap 8G cargo test --workspace --no-fail-fast` -- exit 0, **1468 passed, 0 failed, 4 ignored** (summed from every crate's `test result:` line), matching the stated baseline

## Verdict

**Task 1 closed.** Nothing outstanding from NN1-NN5, the approved out-of-item edit reads correctly, the one deliberately-left item is accounted for, and the independent closed-set redo turned up nothing new.
