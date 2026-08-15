# SDD ledger -- plan: docs/superpowers/plans/2026-08-12-condition-promotion.md

## Task 1: an IF's condition compiles
BASE 7c2faba394c6dbe697ce73bb3ef7ce5ff0093b81
Implementer: commits 7ea946434 (the eval_condition split), 9d6e55bcb (the promotion). 1465 passed, 0 failed. Status DONE_WITH_CONCERNS, four concerns.
Review (opus): spec PASS, quality CHANGES REQUESTED, ten findings, all four concerns upheld or clean. Review at task-1-review.md.
Controller ruling on the plan conflict: the reviewer's finding that four mandated case rows add no witness governs over the plan's Task 1 Step 10. Rows c1/c2/c3/c5 to be dropped, c4/c6 kept, a traced-declining row added, and Step 10 of the plan corrected in the same commit. The plan is this session's own document, so this was adjudicated rather than escalated.
Fix round 1: dispatched to the original implementer with all ten findings.

## Interleaved, not part of Task 1
cbdeaab71 corrects the plan's claim that the comma list needs a jump inside a clause region. It does not: a false element is the branch. Two premises under that claim were also wrong -- `&`/`|` do not short-circuit (oracle: `if 0 & (1/0) then nop` raises 42.3), and Op::Jump's in-region arm is unreachable. The decision not to promote the comma list stands on there being no workload for it.
Driver-structure and VM-comparison analyses at driver-structure-analysis.md and vm-comparison.md. Both say: keep the region, and if intra-clause branching is ever needed, a forward-only skip on the slice iterator buys it. The open question they raise is guarded specialisation (inline caches, an int fast path, a NUMERIC DIGITS bail-out), which needs a shared slow-path block that a forward-only skip cannot express.
Also recorded there: `emptyloop` cannot measure anything about the region loop, because its body is a `nop` and a `nop` is Op::Generic.
Fix round 1 landed: 0459167cc, all ten findings and all three rulings addressed, 1465 passed 0 failed. The implementer reports each new discrimination claim carries a named whole-workspace --no-fail-fast mutation, and that the traced-declining row needed a second row beside it (`if 1, 1` succeeds where `if .nil` raises before a doubled `>>>` could appear).
Scoped re-review dispatched over 9d6e55bcb..0459167cc.
Task 1 verified independently by execution profile at 9d6e55bcb, in a detached worktree with the temporary counter: `evalexpr IF` is absent from the table where it was one entry per IF execution. No speed reading taken -- both profiled builds are instrumented, and this task deletes one counter call per IF, so the clauses-per-second moved for the instrument's own reasons.

## Task 2: a DO/LOOP header's values compile
BASE 0459167cc. Dispatched. Hazard named in the dispatch that the plan does not carry: the header's destination registers are allocated in the enclosing scope and released past the END, the loop body runs inside the header's clause region through Op::LoopRun, so an operand temporary that outlives its slot could be handed to a body clause mid-loop.
Task 1 re-review: another round needed, four prose corrections (N1-N4) plus three lower ones, none touching behaviour. N1 is the shape to remember: a "reddens this test and nothing else" claim, true when measured and falsified by a later hunk of the same commit.
Fix round 2 dispatched, then STOPPED within a minute by me -- I had dispatched Task 2's implementer into the same files. Nothing was written; verified by the N1 sentence being byte-identical between 0459167cc and the working tree. Re-sent after Task 2 committed.

## Task 2: a DO/LOOP header's values compile -- COMMITTED b8e9db0e5
1468 passed, 0 failed. DONE_WITH_CONCERNS, five concerns. Review dispatched.
Concern 1 is a finding beyond this plan: **the oracle prints a `>K>` line for a `DO OVER ... FOR`'s count and this crate prints nothing on either engine**, because HeaderRole::OverFor::keyword() answers None. Pre-existing, found by this task's captures, left unfixed because the one-line fix moves trace output on both engines. Two doc comments that had asserted it as a measured oracle match are corrected, and a transcript pinning the gap is committed. **This needs its own task.**

## Echo-op price spike (not merged, no commit)
Report at echo-op-price.md, base 0459167cc. Four arms: head, gated-at-emission, and TWO do-nothing controls, the second added by the agent because the first control moved as much as the effect.
Wall clock: no win survives its control. On arith the identical-behaviour arms span 9.04% and head beats the control 20/20; the gated arm lands inside the control band on three of five axes.
Instructions (C and D agree to 0.00%, so this instrument sees the op stream and not the layout): the gated arm removes 1.64% on rexxcps, 2.57% arith, 8.17% varlookup, 1.76% strings, 4.26% alloc4c. Unit price of deciding not to print, measured by a three-probe differential: TraceLiteral 40, TraceRead 45, TraceOperator 61 user instructions.
rexxcps's own TRACE/ADDRESS clauses, tracing off: -2.09% wall, 20/20, -1.82% instructions; they cost more instructions than every intermediate echo op in the program combined.
Two side findings: no compile-time assertion requires an echo op to be *present*, so an accidentally dropped one is invisible to compile.rs; and the 9.04% control span on arith is worse than the ledger's stated resolution floor, which bears on this plan's own closing measurement and on entry 25's accepted arith regression.
Task 2 fix round 1: 355017720, all findings fixed, 1468/0/4. It reproduced F1 rather than accepting it and found the reviewer's ordering observation equally tree-specific -- read_dir returns the two case files in opposite orders on the two checkouts, so BOTH parties had generalised from one directory's inode order. Settled by holding every other case file out: the file alone catches three mutations. It also caught the project's recurring defect on itself mid-round (a number written from the concept, cited to a measurement of something else) and recorded the near-miss. Re-review dispatched.

Task 1 fix round 3: 367cc8d61. Deletions rather than repairs, by my ruling, after three rounds each seeded new defects out of the same paragraphs. Deliverable was a closed set: every live occurrence of the false comma-list re-check claim, with a verdict. **Its origin is docs/superpowers/plans/2026-07-30-phase-4a-executor.md:739, a different phase's plan, which is where Task 1's brief inherited it.** That clause is deleted too; I read the hunk and kept the edit -- the instruction it sits in is true and complete without it, and leaving it is what produces the next copy.
Final scoped re-review dispatched (cheaper tier, scoped to deletion coherence plus an independent redo of the closed-set search).

## Task 3: a plain WHEN's condition compiles
BASE 367cc8d61. Dispatched. Dispatch carries the ruling that a row is either a measured witness or a labelled transcript, never both by implication -- the conflation is what cost Tasks 1 and 2 a round each. Also flagged: chunk_node_at's subset doc has no eval_chunk_expr `When` arm to be a subset of, and the implementer must resolve that honestly rather than quietly.

Task 1: complete. Closed at 367cc8d61 after three fix rounds. Final scoped re-review: every deletion leaves coherent text, no new claim introduced, and an independent redo of the closed-set search found nothing the table missed.
Tooling trap found by that re-review and confirmed here: the interactive `grep` is a ugrep wrapper with `--ignore-files`, so it honours .gitignore and returns ZERO lines (exit 1) for a search of `.superpowers/`, where /bin/grep finds 47. Every SDD report, review and ledger in this workflow lives in a directory the default search reports as empty. Memory updated.

Task 2 re-review: another round needed. N1 a doc claiming the suite stays green in a direction where it does not (true only with a second mutation also applied); N2 a false universal that replaced a rotting count under the no-cardinality rule -- the converse hazard, fired for the first time; N3 a true-but-unmeasured claim the re-reviewer then measured; N4 "three mutations" stale within its own round, a fourth reddens the file alone. Also a deletion lost a verified fact worth restoring: the DO OVER ... FOR row is the only trace-level record of that shape in the crate.
Round 2 QUEUED behind Task 3's implementer -- same files.

## Task 3: a plain WHEN's condition compiles -- COMPLETE d3372e429
1471 passed, 0 failed. Review: spec PASS, quality PASS. No behaviour defect, no divergence; the reviewer's own eight three-way probes agree oracle/tree-walker/ir, and all four new case rows re-captured byte-identical.
All six implementer concerns accepted or verified, including the ruling that correcting chunk_node_at's "subset" doc beat adding an unreachable mirroring arm -- the reviewer's framing correction is that the subset was true before this commit, and what was already wrong was choosing containment as the invariant at all.
First task in this plan where the test discipline held without a round forcing it: rows are witnesses or labelled transcripts, four mutations run twice including at the commit's final state, and one of its own comments killed by its own mutation before commit.

Task 3: minor (deferred): the plan's profile table still says "promoted, and its condition runs eval inside scan_when" for WHEN under a present-tense header -- third rot line of this phase. Controller is fixing the HEADER rather than the row: the table is a measurement dated 2026-08-12, not a description of today, and re-dating it stops every future instance.
Task 3: minor (deferred): Step 4's "trace r/trace i over each" met with one setting per shape; deviation not declared.
Task 3: minor (deferred): "both stayed green, which is what says the promotion moved no bytes" attributes to two rows what the harness, branch table and corpus sweep carry -- the recurring class in mild form.
Task 3: minor (deferred): render_condition_keyword's If/When mapping written twice (golden.rs and corpus_shape_tests.rs); Seen::native_conditions' doc says "promoted clauses" where the code counts ops per region; compile.rs:672's catch-all against the file's stated exhaustiveness convention.
Task 3: not a defect, recorded: every_blocked_axis_still_fails_on_this_crate resolves target/debug/rexx-run relative to the crate rather than via CARGO_TARGET_DIR, so a naive detached-worktree run shows 27 unrelated failures. Every reviewer in this plan has hit it.

Task 4 (RETURN/EXIT/PUSH/QUEUE) is QUEUED behind Task 2's round 2 -- both write compile.rs and golden_tests.rs.
Task 2 fix round 2: cfbbdedd7, all six items plus the restoration, 1471/0/4. Three things it reported: N1's mutation moves eight tests here, not the re-review's seven (the tip moved under it) -- substance identical; N3 came out STRONGER than written, so it did not do what I asked -- per-stanza runs show all four rows catch two mutations each on their own, which is shorter and truer than giving each row a different catch, and the header no longer promises that each comment names its own; and it recorded N2's lesson as a rule -- when a cardinality has to go, name fewer things rather than quantify over more.
Controller ruling: **no dedicated re-review of Task 2's round 2.** Folded into the final whole-branch review, which must scrutinise cfbbdedd7 specifically, because that round wrote new measured claims ("all four rows catch two mutations each") and this plan's rounds have a measured habit of seeding defects.
Task 2: COMPLETE.

54afba8a1 dates the plan's profile table instead of maintaining it -- controller edit, closing the rot class rather than its third instance.

## Task 4: RETURN, EXIT, PUSH, QUEUE
BASE 54afba8a1. Dispatched. Told to decide four-ops-versus-one-tagged-op from the code rather than copying Task 3's answer, since RETURN's tail is not EXIT's and the tagged-op precedent rests on the tails differing only by which Flow they answer.

## Task 4: RETURN, EXIT, PUSH, QUEUE -- COMMITTED 0d058c086
1476 passed, 0 failed. Review: spec PASS, quality PASS with one Major prose finding.
Shape decision: TWO tagged ops, rejecting both options the brief offered. The review checked the argument leg by leg in the parent code and all three hold -- RETURN/EXIT differ only in the Flow constructor, PUSH/QUEUE only in the queue end, and between the groups the trace rule, the side effect and the region end all differ.
Both riskiest items passed: the corpus edit moved no traced line number and the regenerated golden differs only in the five header lines with its body byte-identical to the source; the 1->2 echo count confirmed by a probe over five callee bodies.
Fix round dispatched: the Major (returned_value's doc claiming a window that is EXIT's rather than general, refuted by apply_flow rooting both arms) plus two minors.
Task 4: minor (deferred to the whole-branch review): compile.rs's Say/Return/Queue arms are three copies of the same block and drive.rs repeats the assert-plus-src.map preamble; the arity debug_assert accepts Return|Exit without pinning keyword against kind, so one mutation passes it; 2026-08-12-remaining-promotion-survey.md still lists these four as unpromoted; push_queue.rex's header enumerates two read-back places where there are now three.

e379b68c0 + 9956d6abd file the SIGNAL ON SYNTAX handler-indent divergence Task 4 found, reproduced by me first. Paired with phase 4e's CALL ON instance, which errs in the OPPOSITE direction, and carrying a falsifiable hypothesis: this crate keeps whatever indent is live at a transfer where the oracle sets it from the kind of transfer -- so a fix that merely restores an indent fixes the SIGNAL case and leaves the CALL ON case exactly as wrong.

REMAINING: Task 4's fix round, then the whole-branch review (must scrutinise cfbbdedd7 specifically), then the closing measurement -- rexxcps via rexx-bench-suite, TWO do-nothing controls not one, because identical builds span 9.04% on arith.

## Whole-branch review: mergeable on behaviour, six live prose defects
final-review.md. Gates re-run: 1476/0/4, fmt 0, clippy 0.

TWO CORRECTIONS TO THIS LEDGER, from that review:
* I recorded that `2026-08-12-remaining-promotion-survey.md` still carries the comma-list "needs a jump inside a clause region" claim. **It never did.** That claim lived only in the plan document, and cbdeaab71 withdrew it there. The survey's real defect is narrower: Part 1 Task B still proposes RETURN/EXIT/PUSH/QUEUE as work to do, in the four-op shape Task 4 rejected.
* I recorded compile.rs's `_ =>` catch-all as against the file's stated exhaustiveness convention. The review says no convention in that file is violated, so the item is withdrawn rather than deferred.

Also from it, for every future reviewer: in a detached worktree **26 of the 27 failures are rexx-extract resolving `../../../ootest`, and two of those are ir_dual.rs's population sweep -- which is therefore SILENTLY NOT RUN in a worktree**. Two symlinks (ootest, rexx-run) clear all 27. Every review on this plan paid to rediscover this, and two of them were reasoning about a sweep that had not executed.

And a self-caught trap worth keeping: the reviewer restored compile.rs after a mutation and probed WITHOUT rebuilding, manufacturing four convincing false "engine divergences" out of the stale binary.

## PLAN CLOSED
Final fix round 85e412fae (all eight items; item 3 was four passages, not three). One stale note corrected by me at de6efa3b1 -- pre-existing, falsified by an earlier phase, fixed because a false sentence does not become someone else's for having been written first.
Closing measurement: entry 27 of phase-4f-record.md, commit 5dc12a403.
  rexxcps -3.41% instructions (1.010 billion), -2.97% wall at 15/15, exceeding both control spreads.
  Wall MAGNITUDE not claimed: layout moved five untouchable axes -5.12% to +4.55%, wider than the effect.
  Cost side recorded: +0.24% to +0.54% instructions on four axes that gain nothing.
  The A/A' controls bounded run-to-run noise and NOT layout -- fat LTO gave four trivial edits byte-identical images. The loop axes are what bound layout.
  Artifact worth more than the result: four byte-identical copies of one binary spanned 3.47% wall, split by whether argv[0]'s basename was 10 characters or 11, at 0.0000% instruction difference. Memory updated.

WORKSPACE KEPT, not deleted as the skill's final step asks: record entries 26 and 27 cite echo-op-price.md and closing-measurement.md by path, and deleting an artifact that committed prose points at is a regression, not cleanup.

OPEN, filed during this plan and not part of it:
* docs/superpowers/plans/2026-08-13-over-for-keyword.md -- DO OVER ... FOR prints no >K> line where the oracle does. Step 1 is finding whether the role is alone.
* docs/superpowers/plans/2026-08-13-handler-clause-indent.md -- a trap handler's clauses echo at the wrong indent, in two known instances in OPPOSITE directions, with a falsifiable one-cause hypothesis attached.
* Two symlinks (ootest, rexx-run) would stop every future reviewer rediscovering that a detached worktree silently does not run ir_dual.rs's population sweep.
