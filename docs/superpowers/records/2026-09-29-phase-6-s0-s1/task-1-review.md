# Task 1 review: rulings, grants, flat-loop adoption (0d911ab7c..4b694fe7c)

## Spec Compliance

- ✅ Step 1: R2, R3 and R5 are added to D3's 2026-09-29 note (roadmap :206-215). I checked each quote against the spec's section 1.1 (`specs/2026-09-29-phase-6-concurrency-design.md:36-50`). R2 and R3 are verbatim. R5 is verbatim apart from the spec's bold markup. The numbering is consistent throughout the roadmap. The note already had (1) = R1 and (2) = R4, and the task adds (3) = R2, (4) = R3 and (5) = R5. The Global Constraints amendment cites "ruling 2" (R4, nightly), D-U3 cites "rulings 4 and 5" (R3, R5), and D-U3's "(ruling 4)" for the registry exception matches the spec's own "(R3)" at :282.
- ✅ Step 2, D-U2 (island `Send`): it names a module (`rexx-exec/src/island.rs`), lands in S4, and has all four bar-3 parts. Its invariant matches spec 2.5 and cites the same `dispatch/library.rs` lines, which I checked: :86, :94, :155 and :165 are the `Rc::clone`/`thread.clone()` sites. One present-tense claim is false (Important 2).
- ✅ Step 2, D-U3 (signal module): R3 and R5, lands in S4, with the async-signal-safe invariant. The file name is marked as proposed.
- ✅ Step 2, D-U4 (`frame.rs`): one arena per activity and LIFO per activity. I read the module doc (`rexx-core/src/frame.rs:12-29`) once. D-U4 restates properties 1-5 accurately, and it is right that none of them depends on release order. I checked `release` (:220-233): a foreign or misordered frame only moves `top`, and in the worst case (`wrapping_sub` goes huge) `reserve` opens a fresh block (property 2). So "LIFO is a logical property, not a safety one" holds. The claim that the grant is recorded at `#![allow(unsafe_code)]` holds (`frame.rs:31`, "Granted by Moritz, 2026-09-23").
- ✅ Step 2, row 6: "kernel lock" now points at D3's note, ruling 1 (:657).
- ✅ Step 2, nightly line: the Global Constraints line is amended for R4 (:26-28): TSan gate only, installed toolchain, never shipped.
- ✅ Step 3: the toggle is removed. At HEAD, `git grep REXX_NO_FLAT` has no hit outside `.md` records, so no code or test depended on it (named risk 1). `static FLAT` is gone. I re-ran the brief's command, `/bin/grep -a -rn 'SPIKE\|spike' rust/crates/rexx-exec/src`, and its output matches the report's "after" list. The remaining hits are Phase 4's Task 3 spike and `tests/spike.rs`, not flat-loop markers. The `reason = "spike"` allow is replaced by a real reason, and it chains correctly: `flat_loop_start` cites `run_loop_with_header` (:726-729), which cites `run_repeating` (:952-955).
- ✅ Named risk 3 (comments): every marker edit deletes only the `**SPIKE[, not for commit].** ` prefix and leaves the remaining line short rather than re-wrapping the paragraph. The two rewordings ("this spike drives" / "the spike drives" to "the flat path drives") keep their meaning. Only one comment was dropped, the `// SPIKE:` switch comment, and it described only the deleted switch. `loop_header_plan is the whole refusal` (`loops.rs:1296`) is now true outright, where before the env switch was a second refusal.
- ✅ Step 4: docs and code are separate commits (66073d0a2 docs, 4b694fe7c code). ⚠️ I did not run the gates, as instructed. The controller's `status.txt` reports G1-G6 exit 0 at 4b694fe7c.
- ✅ Global constraints: no `unsafe`, no dependency, and no behaviour change beyond the env toggle's removal. New roadmap prose uses `--`, with no em-dashes. One set-size violation (Minor 1).

## Strengths

- D-U4 is the strongest part. Instead of asserting that the per-activity change is safe, it derives from frame.rs's own numbered properties that release order was never a safety property. It also names the one new fact the change adds: the struct moves and the blocks do not, and a `RegFrame` borrow makes a live frame across the move unrepresentable.
- The blocks mark honestly where they go beyond the spec: D-U2's wrapper shape "is this block's proposal", and D-U3's file is "proposed".
- The quotes are verbatim and the ruling numbering is consistent across four places in the roadmap.
- The removed `reason = "spike"` got a substantive replacement instead of a lint-silencing one, and the report explains why the allow could not simply go.

## Issues

### Critical

None.

### Important

1. **`docs/superpowers/plans/2026-07-27-rust-rewrite.md:38`: "Expect exactly two candidates over the whole project"** (named risk 2, the report's concern 3).
   - **What:** this is the Global Constraints bullet that governs the grants this task records, and after the task it is wrong on three counts. The tree has `bytes.rs` and `frame.rs` beyond the `rexx-api` pair. D-U2 and D-U3 add two more sites, and neither is in `rexx-api` or `rexx-sys`. And it states a set's size, which the task's constraints forbid.
   - **Why this task:** the task's constraint says roadmap prose must not keep stale facts about the done/not-done boundary. This task is the one that writes the unsafe grants into that section's decision blocks, so it leaves the governing sentence contradicting three blocks it just added. "Outside this task's list" does not hold, because the file is on the list.
   - **Fix:** one line, for example: "The granted sites are the D-U blocks in Section 1, and `crates/rexx-core/tests/unsafe_sites.rs` is their enumeration; a site outside them is a new decision." Keep the "*candidates*, not exemptions" and the D5 sentence.
2. **`docs/superpowers/plans/2026-07-27-rust-rewrite.md:338-339` (D-U2, "What enforces it instead"): "`ObjRef` is `!Send` and `!Sync` (spec section 5)".**
   - **What:** this is false today. `pub struct ObjRef(u64)` (`rexx-core/src/handle.rs:47`) is `Send + Sync`, and spec section 5 says it "**becomes**" `!Send`/`!Sync`.
   - **Why:** it is one of the enforcement arguments of an `unsafe` grant, stated as a present fact while the tree contradicts it. That is exactly the done/not-done prose rot the constraints warn about. A reader checking the grant at S4 could take it as already in place.
   - **Fix:** "`ObjRef` becomes `!Send` and `!Sync` (spec section 5) before this grant lands, so ...".

### Minor

1. **`rust-rewrite.md:206`: "The same ruling adds three more".** This states a set's size in prose, which the constraints forbid. It is also awkward, since "the same ruling" refers to a group of rulings. Fix: "The same rulings continue, quoted from the design's section 1.1:".
2. **`rust-rewrite.md:217`, D3's "Evidence" paragraph: "if the nightly ban in Global Constraints blocks this, substitute `loom` ... record which was used".** Ruling 2 and this task's amendment resolve that conditional: both are used. It is stale boundary prose in the block this task edited. Fix: append "(Resolved 2026-09-29 by ruling 2: both.)" or reword the clause.
3. **`rust-rewrite.md:353-356` (D-U3 Question): "only where no handler is already set ... as the oracle's library does".** Spec section 4 qualifies this: the oracle reads only SIGHUP's previous action (`:136-139`), and under `nohup` it installs none, where this design halts, which is a licensed divergence. Stated without the qualification, "as the oracle's library does" overclaims. Fix: add "(the oracle checks only SIGHUP's; this design checks each, a licensed divergence, spec section 4)".
4. **`rust-rewrite.md:109` against :357-361: the index rows are inconsistent.** D-U2's row presents `island.rs` as closed, while D-U3's block calls its file "proposed". Neither file is named by the spec or by the S0/S1 plan (grep of `plans/2026-09-29-phase-6-s0-s1.md`). The brief did ask D-U2 to name its module, so this is only a consistency point. Fix: either treat both names the same way, or add "(named here; the spec leaves it to the plan)" to D-U2's row.

## Assessment

**Task quality: Needs fixes.** The code half is clean and complete, and the three new decision blocks are accurate, with D-U4 correctly derived from `frame.rs`'s invariants. Two roadmap statements must change before this is a sound record of the grants: the stale "exactly two candidates" governing sentence, and D-U2's present-tense `ObjRef` `!Send` claim. The minor items are one-line edits in the same file.
