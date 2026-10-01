# SDD ledger — plan: docs/superpowers/plans/2026-09-29-phase-6-s0-s1.md

Spec: docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md (50adaa724). Plan commit 0d911ab7c.
Started 2026-09-29.

## Pre-flight scan

| Pair / task | Produces vs consumes | Finding |
|---|---|---|
| T1-T10 | T1 removes `static FLAT`/SPIKE markers in run/loops.rs; T10 edits `flat_loop_start` in the same file | sequential, no conflict |
| T1-T2 | T2's base = its parent, after T1 | Ruling P1 |
| T2-T4..T11 | T2 produces the base figures, script and programs every perf gate consumes | consistent |
| T3-T8/T9/T10 | T3's counter hooks sit on paths T8-T10 rewrite | Ruling P2 |
| T4-T5 | T4 `Activity` holds per-execution fields; T5 `ActivityRoots` said to live in `RootSet` | Ruling P3 |
| T5-T7 | T5 keeps LIFO asserts; T7 rewrites positions activation-relative and lets a non-top frame grow | T7 must restate the asserts, as its Step 2 says; consistent |
| T7-T8/T9 | T7 produces activation-relative positions; T8/T9 push frames stackless | consistent |
| T8-T9 | T8 produces begin/finish halves and the resume entry; T9 extends them to sends | consistent |
| T9-T10 | both add driver outcomes; T10 adds the seam | consistent |
| T10 | seam trait methods unused in S1 (only run_until_park, stop_the_world act) | Ruling P4 |
| T6 | `ObjRef` `!Send` check phrased oddly ("static_assertions-free trait probe") | Ruling P5 |
| T8 | "parked state kept in the Activity" vs spec 2.1 "arena-resident" | Ruling P6 |
| each task self-consistency | tests specified before code in T7-T10; T1-T6 mechanical with gates | no internal contradiction found |

Ruling P1: the Phase 6 perf base is Task 2's parent commit (after T1) -- T1's flat-path adoption removes only an env-var toggle and comments, behaviour-identical by default -- if wrong, T1's cost hides in the base; T2 also measures T1's parent once to show it is flat.
Ruling P2: the pinned-park counter's hooks are maintained by every later task that moves a hooked path; each task's report shows the counter still compiles away (.text hash) -- the counter is Task 11's instrument -- if wrong, Task 11 re-measures a stale counter.
Ruling P3: `ActivityRoots` may live inside `Activity` or inside `RootSet` as T5 finds best, provided the running activity's part is one indexed load from `Interp` and `RootSet::iter`/object_roots cover all activities -- the spec fixes the observable, not the location -- cost if wrong: a later move.
Ruling P4: seam methods unused in S1 are kept with `#[expect(dead_code, reason = "...")]` naming the stage that uses them, never `allow` -- expect fails loudly once used -- cost: none.
Ruling P5: the `!Send` check is a `compile_fail` doctest asserting a function requiring `Send` rejects `ObjRef` -- plan wording was loose -- cost: none.
Ruling P6: "arena-resident" parked state means state reachable through the activity by offset handles, never Rust references; a Box inside `Activity` satisfies it -- matches spec 2.1's intent (named by handle) -- if wrong, S2 moves it.
Task 1: dispatched p6-t1 (opus) base 0d911ab7c 2026-09-29T14:06:29+02:00
Task 1: review p6-t1-review -> Needs fixes (I1 stale 'exactly two candidates', I2 ObjRef !Send present-tense; 4 minors). Ruling: I1 in scope -- file on the task list, grants contradict it -- cost: none. Fix round 1 sent to p6-t1 2026-09-29T15:08:41+02:00
Fix round 1: 9d863ccc5 -- controller checked the diff (all six findings addressed; unsafe_sites.rs and allow sites verified)
Task 1: complete (66073d0a2, 4b694fe7c, 9d863ccc5) 2026-09-29T15:09:29+02:00
Task 2: dispatched p6-t2 (opus) base 9d863ccc5 2026-09-29T15:10:31+02:00
Ruling P7 (Moritz 2026-09-29): even less prose -- comments, decision blocks, reports state what is true now plus a pointer; never describe a future mechanism; a false sentence is deleted, not reworded; reviewers file no prose-polish findings. Goes into rust/CLAUDE.md between tasks (not while an agent is live).
P7 CLAUDE.md bullet approved by Moritz (lgtm) 2026-09-29T15:41:44+02:00; pending: land after Task 2 closes, before Task 3 dispatch.
Task 2: implementer DONE_WITH_CONCERNS 55ee6c08b 0696153b4 f0e3e727e cc74f959c; gates green; concerns: seeded-HashMap Ir excursions (extcall +0.42%), LD_LIBRARY_PATH on all axes 2026-09-29T16:30:42+02:00
Task 2: review p6-t2-review -> Needs fixes (I1 Ir excursions from process-seeded std HashMap, I2 oracle comparison uncommitted; M1-M4). Implementer committed 1f652a43d (prose trim, incl. .rex comment bytes) after its report.
Ruling P8: remove the nondeterminism, don't smooth it -- every process-seeded HashMap/HashSet in non-test rexx-* src becomes rustc-hash (already in the graph via rexx-core); the Phase 6 base moves to Task 2's fix commit (amends P1) and is re-measured with the three controls -- an instrument with +0.4% random excursions cannot gate a +0.3% budget -- cost if wrong: one re-measure; iteration order of a converted map was already random, so no observable determinism is lost.
Ruling P9: LD_LIBRARY_PATH only on the extcall axis (M3); oracle comparison and every figure's command committed (I2, M2); callgrind.sh fails on spread > 0.01% -- cost: none.
Fix round 1 sent to p6-t2 2026-09-29T16:37:03+02:00
Fix round 1 (T2): 1754a3b5a (FxHash; new Phase 6 base), 44c465281, e4928c929, 833847a9b, d1086bb6e; gates green at d1086bb6e. extcall 32/32 identical; rexxcps/heapshape residual variance traced to TIME() (chrono/getenv/statx).
Ruling P10: wall-clock ±4% binds only where the program's own layout-control band is inside ±4%; otherwise the bar is its measured band (decloop +6.10%, startup -7.69%, both <0.3 s) -- instruction counts stay the binding gate -- cost if wrong: a short-program wall-clock regression goes unflagged.
Ruling P11: the hasher change's own delta (extcall -16.44%, startup -4.97%, alloc +0.27%) is recorded in phase-6-perf.md -- measured fact, not boundary prose -- cost: none.
P7 landed in rust/CLAUDE.md: 407113d94. T2 docs 72efa5121; re-review p6-t2-rereview dispatched 2026-09-29T17:54:21+02:00
T2 re-review: Approved; N1 (band extcall env), N2 (wallclock.sh no rc/stdout check) -> fix round 2 sent 2026-09-29T17:58:44+02:00
Fix round 2 (T2): 1cd896327 -- controller checked diff (band for_axis; wallclock rc+stdout)
Task 2: complete (55ee6c08b..1cd896327); Phase 6 perf base = 1754a3b5a 2026-09-29T18:01:26+02:00
Task 3: dispatched p6-t3 (opus) base 1cd896327 2026-09-29T18:01:57+02:00. Resolution: would-be park point = arrival, contended or not; refused tests recorded with refusal site.
Task 3: e076572fc, gates green 20:49 (controller read status.txt); implementer stalled after; review dispatched 2026-09-29T22:26:36+02:00
Task 3 review: Needs fixes (I1 Native ~RESULT artefact = all 38 non-tree arrivals; I2 MethodArgs missed via class ACTIVATE; I3 no gate builds --features pinning).
Ruling P12: every later task gates with scratchpad/p6-gates/gates.sh (G1-G6 plus G7 clippy --features pinning, G8 pinning self-tests) and the counter gains one probe per Rexx-reachable frame kind -- enforces P2 mechanically instead of by dispatch prose -- cost: ~1 min per gate.
Fix round 1 sent to p6-t3 2026-09-29T22:33:12+02:00
Fix round 1 (T3): 027ef469c; gates G1-G8 green 22:57 (G8 via implementer's corrected filter; p6-gates/gates.sh G8 fixed to put filters after --). Controller checked diff: native() skips RESULT/WAIT/ACQUIRE by name (fine for an instrument), per-kind probes, ACTIVATE reach, all-tree measured summary.
Task 3: complete (e076572fc, 027ef469c) 2026-09-29T22:57:51+02:00
Task 4: dispatched p6-t4 (opus) base 027ef469c 2026-09-29T22:58:14+02:00
Task 4: implementer DONE_WITH_CONCERNS 0a4c5f46a, 8f0437be3, edacf562d; gates G1-G8 green at 8f0437be3 (controller read status.txt). Perf: boxed +9.65% nop / +4.03% rexxcps -> inline within ±131 Ir.
Ruling P13: the running activity is held inline in Interp; parked activities live boxed elsewhere, and a switch is a mem::swap of the inline value with the parked one -- measured: the box costs up to 9.65% on every access, a swap costs O(size_of Activity) only at a switch (slice rate or a wait); sound because nothing holds a Rust reference into Activity (P6) and arena blocks do not move (D-U4) -- deviates from brief's Box<Activity> and spec 2.1's "pointer swap" wording in mechanism, not observable -- cost if wrong: S2 reboxes and pays the measured cost.
Note: 8f0437be3's message says fields "sit where they sat"; not true of offsets (commit messages are immutable; recorded here).
T4 review: Approved; size_of Activity 1040 B (active_condition 200 B is first cut if swaps show). Minor 2 fixed by controller 0993b9457; minors 1,3 are in the untracked report / immutable commit message, recorded here.
Task 4: complete (0a4c5f46a, 8f0437be3, edacf562d, 0993b9457) 2026-09-29T23:51:03+02:00
Task 5: dispatched p6-t5 (opus) base 0993b9457; P13 applied to ActivityRoots (inline) 2026-09-29T23:51:26+02:00
Task 5: implementer DONE_WITH_CONCERNS 50cce849c, 5ecf22e37; gates G1-G8 green; perf flat. Miri missing from default toolchain -> sent scratch RUSTUP_HOME recipe (surface-4/rustup-home). Review dispatched 2026-09-30T00:25:23+02:00
T5 Miri (Stacked Borrows, scratch RUSTUP_HOME surface-4/rustup-home, +nightly, RUSTUP_AUTO_INSTALL=0): --lib 17/17 on base and 50cce849c; --test roots/collect/heap/uninit all ok at 50cce849c. 2026-09-30T00:28:21+02:00
T5 review: Approved (0 C / 0 I / 3 M). M3 fixed by controller 97f564413 (sourceline + corpus tests green). M2 report-only. 
Ruling P14 (M1): frame.rs's "no RegFrame is live across the move" stands -- a switch happens only at a driver exit (P6-3), where run_chunk has returned, so it is a requirement on S2's switch, true of every switch the design allows -- cost if wrong: S2 finds a mid-chunk switch and must drop the RegFrame first.
Reviewer note for Task 7: SlotFrame/SlotRef are not tagged with their activity.
Task 5: complete (50cce849c, 5ecf22e37, 97f564413) 2026-09-30T00:30:51+02:00
Task 6: dispatched p6-t6 (sonnet) base 97f564413 2026-09-30T00:31:11+02:00
Task 6: implementer DONE_WITH_CONCERNS 6956e90f2, 98d9fae33, b9f85325e; gates G1-G8 green (01:32). S0 Ir gate: all programs inside budget (max nop +0.0303%). Wall: dispatch +10.44% (re-run +11.62%); Step-1-only binary (-5 Ir) +14.91%; identical-binary control -0.79%.
Ruling P15: extends P10 -- a program's wall-clock bar is the larger of ±4% and its measured zero-work-diff band (a code change adding no instructions); dispatch's is +14.91%, sendloop's is measured the same way when it next exceeds ±4% -- the control shows layout, not work -- cost if wrong: a real wall regression on dispatch under ~15% passes S0 unflagged; instruction counts still bind. Flagged to Moritz as a ruling taken without him.
S0 CLOSED at b9f85325e (subject to Task 6 review).
T6 review: spec-compliant; one stale line fixed by deletion (controller) 8b253ef97.
Task 6: complete (6956e90f2, 98d9fae33, b9f85325e, 8b253ef97). S0 CLOSED. 2026-09-30T01:42:58+02:00
Task 7: dispatched p6-t7 (opus) base 8b253ef97 2026-09-30T01:43:27+02:00
Task 7: implementer DONE_WITH_CONCERNS 31a394482, a0a82f2b7, 863d147d1; gates G1-G8 + Miri green; Ir in budget at round 2. Queued drop-unset-stem-symbol divergence. Review dispatched 2026-09-30T03:47:29+02:00
T7 review: Needs fixes (I1 release-silent wrong-frame fast path; I2 u32 native id aliasing a live frame; M1-M5). Wall medians inside bars, no re-run.
Rulings: I1 fix by serial compare on the fast path (correct by construction) + assert_eq in segment(); I2 fix by row<<32|id token checked at the row; M4 carried to Task 11 (criterion 8 enumeration must grep messages/owners, since constructor scanning misses struct-literal refusals); M5 skipped (YAGNI). Fix round 1 = perf round 3 of 3. Sent 2026-09-30T03:52:13+02:00
T7 fix round 1: d112d68f9, 8d26923bb; gates G1-G8 + Miri green. extcall +0.3642% (over +0.3% after round 3); assign wall +4.21% (Ir +0.0145%).
Ruling P16: the +0.3% budget binds at stage close (S1, Task 11), as the plan's performance rule states it per stage; a task over it carries its figure forward in the running total and Task 11 gates the stage, with Moritz's ruling there if still over -- extcall's +6 Ir/call buys two correctness fixes (I1, I2) -- cost if wrong: S1 closes needing a round on extcall.
assign wall +4.21% with Ir flat: treated under P15 as layout; its zero-work band is measured at Task 11 if it persists.
T7 re-review: Approved (fast_serial refreshed at every segment/alias change; begin_indirect gone, CallerSwap correct by serial uniqueness; token decode panic-free).
Task 7: complete (31a394482, a0a82f2b7, 863d147d1, d112d68f9, 8d26923bb). S1 running total carries extcall +0.3642% (P16). 2026-09-30T04:49:35+02:00
Task 8: dispatched p6-t8 (opus) base 8d26923bb 2026-09-30T04:50:01+02:00
Task 8: implementer DONE_WITH_CONCERNS 4822d01e0, 1f976e121 (round 1), 403b89925; gates G1-G8 green. Ir over: fibcall +4.65, fibfunc +5.25, sendloop +3.07, extcall +0.91, rexxcps +0.76, dispatch +0.32. Two rounds unused -> sent back to spend them before review. Queued min-with-function-arg divergence (pre-existing). Oracle-crash entry 20 added (unbounded internal recursion under SIGNAL ON SYNTAX).
Ruling P17: Phase 6 witnesses may sit in phase-8.txt until Task 11 closes Phase 6 (a phase-6.txt would mark it closed in gate table C) -- cost: Task 11 moves them.
Task 8: rounds 2-3 done 7d7ace817, 5b8c2af83, b558488a1; gates G1-G8 green at 5b8c2af83. Ir over: fibcall +2.96, fibfunc +3.64, sendloop +1.12, rexxcps +0.63. Wall over: fib ~+10%, heapshape +4.09%. Implementer used git checkout-index -f on own uncommitted edit (noted). Review dispatched 2026-09-30T09:49:25+02:00
p6-gates backed up to workspace p6-gates/ (tmpfs reboot) 2026-09-30T10:06:51+02:00
T8 review: spec not compliant until I1+I2; quality approve-with-fixes. I1 three_deep witness exits at RAISE (paths unrun); I2 callee_allocates witness only inline strings; I3 nested call in arg list recurses unpinned (report false); I4 non-TOP CallExpr/Call arms duplicate begin_call_*; M1 no change.
Rulings: I1, I2, I4 fix as reviewer says. I3: pin tree arg evaluation as TreeEval + FRAME_PROBES case, delete the report sentence; stackless argument positions are Task 9's scope -- cost if wrong: Task 9 compiles them anyway, the pin is removed there. Fix round 1 pending (Moritz rebooting) 2026-09-30T10:09:58+02:00
T8 fix round 1: dispatched p6-t8-fix (opus, fresh) base b558488a1 2026-09-30T10:11:03+02:00
T8 fix round 1: STOPPED by Moritz before work ('Don't start!'); redispatch after reboot 2026-09-30T10:11:11+02:00
T8 fix round 1: re-dispatched p6-t8-fix (opus) after reboot, base b558488a1 2026-09-30T10:23:30+02:00
T8 fix round 1: 460034fb4 I1, 47f70e30b I2, c929acecf I4, 8428ca61b I3, 4b0128186 perf; Ir fib unchanged, rexxcps +0.50. Gates running. 2026-09-30T10:57:14+02:00
T8 fix round 1 gates G1-G8 green at 4b0128186 (11:15). Re-review dispatched p6-t8-rereview (sonnet) 2026-09-30T11:15:41+02:00
T8 re-review: Approved (I1-I4 addressed, no new problem; pinned_parks_over_the_derived_list red only in git-archive copy for lack of ootest, green in G8).
Task 8: complete (4822d01e0, 1f976e121, 403b89925, 7d7ace817, 5b8c2af83, b558488a1, 460034fb4, 47f70e30b, c929acecf, 8428ca61b, 4b0128186). S1 running total: fibcall +2.96, fibfunc +3.64, sendloop +1.12, rexxcps +0.50, extcall -0.98 (P16, gated at Task 11). 2026-09-30T11:17:47+02:00
Task 9: dispatched p6-t9 (opus) base 4b0128186 2026-09-30T11:17:47+02:00
Ruling P18 (Moritz 2026-09-30): Task 10 dispatch trial for agent cost. (1) name functions/line ranges, no whole-file Read of big files; lumen semantic search to locate; (2) no "read rust/CLAUDE.md", interface summary instead of whole reports; (3) split implementation and perf rounds into two agents (perf agent gets a lean brief); (4) exploratory perf: target program + one control, -r 1; full set only for the committed round; build variants in parallel target dirs, measure in one batch.
Baseline for the comparison (p6-t9 at ~13:28, mid-task): 189 API calls, context 635k, cache_read 74.5M, cache_write 5.4M, 9 full misses all after >5 min foreground perf calls (build+callgrind), Read 479k chars + sed/grep 322k chars, output 207k tok. Measure Task 10 agents the same way (tools/tok.py, tok2.py, plus the >5 min call listing) after Task 10 and compare per task.
Task 9: gates at 85a3a1db6 G4/G6 red (dispatch_seam: method_is_protected has two callers, dispatch.rs:1909 and :2093); sent back to fix via one seam, not by relaxing the test. 2026-09-30T15:41:05+02:00
Task 9: implementer DONE_WITH_CONCERNS 35bd340f0 805a1fb5d 31f4a5485 54b07d745 f00c8921e 54c4c28e5 85a3a1db6 8d65d9406; gates green at 54c4c28e5. Ir: fibcall +2.64 fibfunc +3.87 sendloop +1.23 rexxcps +0.50. WALL: sendloop +19.7 dispatch +19.0 dispatchclass +8.8 fibfunc +8.2; sendloop L1i misses 55.6M vs 0.84M base, undiagnosed. 2026-09-30T16:16:46+02:00
Task 9 cost baseline (final, p6-t9): 287 API calls, final context 772k, cache_read 134.2M, cache_write 15.75M, 23 full misses, 25 tool calls >5 min, wall 4h59, tool results 1.0M chars (Read 479k, Bash 519k), output 287k tok.
icache diagnosis: .superpowers/sdd/2026-09-29-phase-6-s0-s1/icache-diagnosis.md -- regression starts at Task 8 (sendloop L1i 1M -> 30M), Task 9 doubles it; per-send path spans drive+begin_invoke+ops_loop_steady+resume_region. 2026-09-30T16:24:29+02:00
T9 review: spec NOT YET; quality approve after C1+I1. C1 five shapes lost pins while recursing (DELEGATE send; INIT via ~new on Directory/Array/Table/Class subclasses; PinKind::native exempts NEW/SEND/CALL names). I1 min(3,f()) compiled 3 vs tree 5 in one binary (oracle 3). M1 native_tails rooting unwitnessed. M2 other ~new INIT still recursive (disclosed).
Rulings: C1 fix as reviewer says (drop names from RESUMABLE_OR_PARKING, pin delegate send, 5 shapes in pin test). I1 fix the tree form to answer the oracle's 3 (closes queued min-with-function-arg), witness both forms. M1 add a witness where a tail is an object's only root, show it red with object_roots deleted. M2 no change beyond C1. Fix round 1 goes to a fresh agent (P18: the implementer's context is 772k) -- cost if wrong: re-reading. 2026-09-30T16:31:10+02:00
T9 review addendum: every recursive ~new (Array, Directory, hash-new rows, Class, MutableBuffer, WeakReference) parks unpinned. Ruling: brief Step 3 requires ~new into INIT resumable, so every NEW row shares Object's native_new begin half (one INIT tail), witnessed by flat native stack at 50 vs 500 and no pin; plus names dropped from RESUMABLE_OR_PARKING. Sent to p6-t9-fix -- cost if wrong: bigger fix round than the pin-only minimum. 2026-09-30T16:36:01+02:00
T9 fix round 1: 4633bb58e I1, 7092dc75a C1, 73433148a M1 (collect_now unit test, accepted), 9e09cbc85 C1 widened (all NEW rows share begin_init); gates G1-G8 green at 9e09cbc85 (17:50). alloc +0.10% Ir left to perf round. Oracle-crashes entry 21 added. Re-review dispatched 2026-09-30T17:51:09+02:00
T9 re-review: Approved (C1 widened, I1, M1, M2 addressed; no new problem). Two pre-existing divergences queued (hash-subclass-init-capacity, stringtable-subclass-new-arg). Controller 2050300f4 widens oracle-crashes 21 (self~items read, reproduced 3/3).
Task 9: complete (35bd340f0, 805a1fb5d, 31f4a5485, 54b07d745, f00c8921e, 54c4c28e5, 85a3a1db6, 8d65d9406, 4633bb58e, 7092dc75a, 73433148a, 9e09cbc85). 2026-09-30T18:03:08+02:00
S1 front-end perf round: dispatched p6-perf1 (opus, fresh, P18 lean brief perf-s1-frontend-brief.md) base 2050300f4 2026-09-30T18:03:08+02:00
P18 trial note: p6-perf1 round 1 committed 49a55bea9; it idled 20:30-20:45 waiting on a background job's completion notification that never woke it (batch done 20:34). Nudged; told to use foreground bounded until-loops. Round-1 Ir: sendloop -1.08, fibfunc +4.30, fibcall +2.12, dispatch +0.12, rexxcps +0.09. 2026-09-30T20:45:52+02:00
S1 front-end round: p6-perf1 DONE_WITH_CONCERNS 49a55bea9 (Op::CallingClause split, resume in calling expansion, grant_for), 92ac5c054 figures; gates green 21:38. Ir sendloop -1.08, dispatch +0.12, fibfunc +4.30, fibcall +2.12, rexxcps +0.09. Wall still over: sendloop +17.06, dispatchclass +7.03, fibfunc +8.86, fibcall +6.05, rexxcps +5.02; dispatch +5.87 (was +18.85). Agent's finding: L1i/op-cache swing 2x-20x between builds of one design (sendloop 1.18M vs 70M), so further rounds pick layouts; reliable levers are per-send footprint or build-level layout control (function order / PGO) -- flagged to Moritz.
P18 cost, p6-perf1: 253 calls, final ctx 510k, cache_read 69.0M, cache_write 5.46M, 12 misses, 12 calls >5 min, wall 3h37, results 443k chars (all Bash).
Review of perf round dispatched 2026-09-30T21:40:18+02:00
Ruling P19 (Moritz 2026-09-30): no iteration on layout lottery; no layout control (function order/PGO) for S1. Wall excess from layout is carried to Task 11 as recorded; footprint reduction only where a task touches the path anyway. 2026-09-30T21:41:50+02:00
perf1 review: no Critical/Important; M1 false sentence in perf1-report.md deleted by controller (fibfunc/dispatch still run both expansions per call; recorded). S1 front-end round: complete (49a55bea9, 92ac5c054).
Task 10: dispatched p6-t10 (opus, P18 lean: implementation only, perf round separate if needed) base 92ac5c054 2026-09-30T21:54:24+02:00
Ruling P20: Task 10 seam trait declares only methods the driver calls in S1 (park, run_until_park, spawn, stop_the_world if called), each with a real single-activity meaning; the rest of the brief's list land with their first caller; no expect(dead_code) -- CLAUDE.md no-shell rule and Moritz's don't-lock-in -- cost if wrong: later stage adds the methods. ExecOutcome {Done(Flow), Park(ParkReason), Split(Flow)}, Deliver::Wake approved. 2026-09-30T21:58:42+02:00
Task 10: implementer DONE_WITH_CONCERNS 2fb2bd50c, 224216c7c, 0f5aef01c; gates green at 0f5aef01c. Seam declares all ten methods (P20 not applied: message crossed or ignored). Ir vs 92ac5c054 up per clause: assign +3.72, nop +3.25, varlookup +2.50, textnum +1.32, compound +1.31, emptyloop +1.08, nine more +0.33..+0.73; fibcall -1.30, fibfunc -1.58. Review dispatched 2026-09-30T23:13:49+02:00
P18 cost, p6-t10 (implementation only): 122 calls, final ctx 283k, cache_read 22.3M, cache_write 1.80M, 6 misses, 7 calls >5 min, wall 1h19, results 314k chars (Read 220k, Bash 92k).
T10 review: spec FAIL on P20; behaviour sound (45 plain-DO probes identical to 92ac5c054; 4 pre-existing oracle diffs). I1 seven uncalled seam methods + trait expect(dead_code); I2 false "no bench program has a plain DO" (rexxcps has two in its timed loop); I3 per-clause Ir attributed to Park/Split arms in region_ops! and the Deliver::Wake epilogue match (drive.rs:1653), control reverts to 92ac5c054 exactly; M1 false split_continuation doc; M2 scripted Split cannot split REPLY (test-only).
Rulings: I1 delete the seven methods, their docs and the trait-level expect; keep spawn/park/run_until_park (None = single activity cannot spawn, park records nothing); the cfg_attr(not(test), expect(dead_code)) on ExecOutcome::Park/Split/ParkReason::Guard stays (self-expiring, wired and tested). I2, M1 delete the sentences. I3 fix in this round, not a layout matter: nop and assign Ir back to 92ac5c054 (Wake's resume point computed at the park site, Park/Split arms cold/out of line). M2 no change (S1 produces only Done). Fix round 1 resumes p6-t10. 2026-09-30T23:27:16+02:00
T10 fix round: I1 6a16b5e44, I2+M1 a60b30178. I3: exact equality unreachable with any live park arm in ops_loop (six variants; codegen perturbation +-3%, sign varies; only a loud Park equals). Ruling P21: land the variant with the smallest max per-program rise if <= +1.0% Ir everywhere (P19, record perturbation); else keep Park loud in S1 and land the driver's Park path with S2's first real parker -- cost if wrong: S2 pays the arm's perturbation then. 2026-10-01T00:02:31+02:00
P21 applied: v5 max +2.12% (assign) > +1.0%, x7 worse; option b (x1): Park loud in driver in S1, test asserts refusal, Park path lands with S2's first parker. 2026-10-01T00:14:37+02:00
T10 fix round 1 done: 6a16b5e44 I1, a60b30178 I2/M1, a7e53d4c0 I3 (option b), 1f9be8ea5 perf; gates green at 1f9be8ea5. -r 3 vs 92ac5c054: per-clause programs +0.00, max arith +0.24, fibcall/fibfunc -2.32, rexxcps -0.96. Scheduler trait now spawn() only. Re-review dispatched 2026-10-01T01:01:43+02:00
T10 re-review: Approved. Its two "future mechanism" clauses in the report are pointers the CLAUDE.md docs rule allows ("lands with S2's first real parker"); kept. reply2 (oracle runs the REPLY continuation's output before the caller's 'done'; both builds print done first) is pre-existing on 92ac5c054: input to the S2 plan (REPLY continuation as a real activity).
Task 10: complete (2fb2bd50c, 224216c7c, 0f5aef01c, 6a16b5e44, a60b30178, a7e53d4c0, 1f9be8ea5). 2026-10-01T01:04:09+02:00
Task 11: dispatched p6-t11 (sonnet, measurement/record task) base 1f9be8ea5 2026-10-01T01:04:09+02:00
Controller: Task 11's slow cap-lifted EXIT is SYNTAX 44 unwinding through deep frames to a trap, superlinear; queued 2026-10-01-syntax-unwind-superlinear.md. p6-t11 stalled 01:37-07:30 without gates/report; nudged. 2026-10-01T07:30:32+02:00
Ruling P22 (Moritz 2026-10-01): accept fibfunc +1.88% Ir over the S1 budget; S1 closes with it recorded as accepted. 2026-10-01T08:01:14+02:00
Task 11: p6-t11 committed 7aeb6b0e6 then died at 01:37 on a permission prompt (rm /msg1.txt after a failed cd); stopped. Controller finished: 6a621dbbd (P22 recorded), gates G1-G8 green at 6a621dbbd (G4 2803/0, G6 2805/0), 70030c6ed status lines in phase-6-gate.md.
Task 11: complete (7aeb6b0e6, 6a621dbbd, 70030c6ed). S1 CLOSED. 2026-10-01T11:22:36+02:00
Final whole-branch review dispatched (fable), range 0d911ab7c..70030c6ed 2026-10-01T11:22:36+02:00
Session OOM-killed ~12:1x during final review slice A (likely its deep-recursion probes); slice A's file final-review-a.md was complete up to findings ("Not reached" unfilled); agents lost.
Final review B (records): B1-B10, all prose/records. Final review A (code): A1 Important (external .rex routine / library / ::REQUIRES prologue run via run_loaded recurses on the Rust stack unpinned); pre-existing B1 (external routine recursion has no depth cap: native stack overflow rc 134 instead of Error 11), B2-B5 queued, B6 already queued, B7 upstream traceIndent.
Rulings (final fix dispatch, one round): A1 pin run_loaded as PinKind::Program when an activation is running, FRAME_PROBES case; A-B1 count external/library program levels in the activation depth so Error 11 fires (it is a crash, adjacent code); slice B B1-B10 as the reviewer says (deletions; B2/B4/B9/B10 minimal factual replacements; B3 update lines). 2026-10-01T12:31:32+02:00
Final fix round: 1242ba315 (external recursion raises 11.1), ead73e1f1 (PinKind::Program, A1), 1a81353e3 (records B1-B10); gates green at 1a81353e3 (G4 2804/0, G6 2806/0). Ir flat. Re-review dispatched 2026-10-01T13:09:51+02:00
Plan complete. Final re-review Approved (final-rereview.md). Workspace copied to docs/superpowers/records/2026-09-29-phase-6-s0-s1 (review packages listed in REVIEW-PACKAGES.md). 2026-10-01T13:39:54+02:00
