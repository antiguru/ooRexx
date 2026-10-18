# Phase 9d Task 2 review (Steps 1-2)

## Verdicts

* Spec compliance: PASS with two gaps (bgcmd has no row; three probes are not reproducible from the committed files).
* Task quality: PASS WITH FIXES. No wrong `closed:` found. One wrong diagnosis in a `9d` reason (lossy).

## Checks

1. Listing: `queued-listing.sh` reproduces the 200-line item column exactly (`diff` empty, no duplicate item). All 29 bullets of the 9a "Carried to 9b-9d" section, the 4 of 9b1 and the 9 of 9c are covered. The 9a gate has 3 more continuation lines (467, 484, 486-487) that are not bullets.
2. Closed rows: ran 41 probes on both sides, all matching the recorded outputs (except the paths below). Agree rows reproduced: pmt csp uam codm re2a re2b replyvis smrun ctxexe dowith notif interpguard ntstdin di pa isyntb nostr hexit pend4 elapsed unwind b7a t4b ctx3 b3 b4 cs e103 clsstr hash_subclass_init_capacity builtin_argument_call ins25 ins35 ins47 rnmsg startw rexxpkgcopy. Evidence-only rows checked: 14 commits exist; tests `appends_and_reads_copy_slots_linearly` and `a_native_row_on_a_receiver_of_another_class_refuses_its_type` exist and pass (ran); `the_row_check_reads_only_a_resolution_that_opens_a_clause` exists (not run); `EXEMPT` is empty; CLOSED lists Phase 5; TIME and CALL rows are gone from DIFFERING; Deviations 21 and 22 exist in phase-4-exclusions.txt; L7, L8 and L9 are "fixed" in phase-9b1-gate.md; the 10 brief report files exist.
3. 9d rows: reproduced the recorded difference for rstart, sortrm, rcfn, libbind, scanlater, lossy, dropstem, rnov, tre, b1, isg, clsdn, fspec, lv4o, stdindrain, streamflush, dimeth (17).
4. Phase and not-a-divergence rows: fit Moritz's rulings. Re-ran lkJ (UNINIT timing only), doto, z26, notop, baghas: each is an oracle-side difference the crate licenses.
5. Duplicates: below.
6. bgcmd: reproduced. Oracle rc 0 `v=[abc] READY`; crate rc 124 with the 4.1 halt at line 2. Short timeouts; no leftover process (checked `/proc/*/cwd`). Not a row in the table.

## Findings

* MEDIUM: `bgcmd` is a live divergence with no row and no queued file. The batch for Moritz drops it unless a row is added. Related to, but not the same mechanism as, cluster `redirected-command-holds-baton` (that one parks on the baton, this one waits for pipe EOF held by the child).
* MEDIUM: the `lossy` reason is wrong. "DIRECTORY() ... ends 72 here" reads the last byte of the starting directory's name (72 and 79 across reruns). `directory(d)` returns empty for a non-UTF-8 name, `SysMkDir` succeeds, and the crate never enters it. The divergence is real, the stated symptom is an artifact of the run directory name.
* LOW: probes `frames1`, `frames2` and `exttr` need companion files (`p.cls`, `mid2.cls`, `ext.rex`) that are not committed. They cannot be re-run from the repo (exttr gives rc 213 on both sides without `ext.rex`). The agreement of the exttr half of `9a:469` is unreproducible as committed.
* LOW: closed rows resting on a text line plus a test name without a run: full-pool-inline-command-hang, literals-rows-need-a-test-case, output-lost-on-kill (the report already says so). I did not find them wrong.
* LOW: `named-semaphores-design` and `sysfork-design` are `closed:Deviation 42/43`, but the reason says Task 5 still relabels the refusals. They are ruled, not yet closed.
* LOW: concurrent agree rows (pend4, re2b, notif, replyvis, codm, hexit) ran once a side here too; same limit the report states.
* LOW: `lr2` is a wall-clock probe with a 0.5 s threshold. Treat as flaky evidence.

## Duplicates (gate bullet points at a queued file that already has a row)

* 9a:473 -> 2026-10-12-buffer-copy-uncharged.md
* 9a:475 -> 2026-10-12-randomised-hash-iteration.md
* 9a:479 -> 2026-10-12-package-as-real-object.md
* 9a:470 -> method-source-compile-retention.md (its gc-trigger twin was removed)
* 9a:471 -> 2026-10-10-directory-setmethod-unknown-traceback-line.md (same stemunk probe)
* 9a:469 shares t4c with dynamic-package-name (compiled-method-package-in-trace)
* 9b1:115, 9b1:117 (a list of 7 queued files), 9b1:123
* 9c:138 to 9c:146, all nine
Proposals agree between each pair. Folding them into the queued row would drop 9a:473/475/479/471 and 9c:138-146, 9b1:115/117.

## Rows whose proposal I would change

* NEW row for `bgcmd`: `9d`, kind divergence, probe `queued-probes/bgcmd/bgcmd.rex:1`, with a queued file.
* `2026-10-13-lossy-path-sites.md` and `gate:phase-9c-gate.md:146`: keep `9d`, replace the reason with "directory(d) answers empty and stays in the start directory for a non-UTF-8 name".
* `2026-10-13-phase-9b-carry-in.md`: `phase:9.1` mixes a perf item (N1) with minors m1, m3, m4 that were not checked. Propose `9d` for the minors and keep N1 at 9.1, or split.
* `2026-10-13-named-semaphores-design.md`, `2026-10-13-sysfork-design.md`, `gate:phase-9b1-gate.md:121`: `9d` (Task 5) instead of `closed`, until the relabel lands. Alternative: leave closed and let Task 4 of Step 4 stamp them.
* `2026-09-16-worktree-build-precondition.md`: `phase:10` rests on "row 10 recompiles"; it is a test-harness path (`dispatch/library/tests.rs:26`) like `arity-harness-ignores-target-dir`. I would propose `9d`, unless Moritz wants it with the extension build.
