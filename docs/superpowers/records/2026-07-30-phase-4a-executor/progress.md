# SDD ledger — plan: docs/superpowers/plans/2026-07-30-phase-4a-executor.md

Phase 4a. Spec revision 4 at 742c7908, plan at e24b0e1d.

Task 1: complete. Commits 5a5feadc (rename) + cb80e2a9 (cargo fmt, separate as required).
BASE was e24b0e1d. 392/392 tests before and after, clippy -D warnings clean, fmt clean.
Measured and settled: an INTERPRET fragment can never contain a label, 47.1 both ways, so
Fragment::body.labels is always empty and 4b needs no label table there.
Spec moved to revision 5 (fdc3a6e4) after Task 1 was dispatched. Nothing in revision 5 touches
Task 1's surface; Tasks 2, 8, 11, 12 and 15 all changed and their briefs must be re-extracted.

Spec revision 6 (d243c9af): the r3/r4 review's 9 Importants + 8 Minors folded. Two change the
build. (1) rexx-num needs a byte-slice comparison entry point: the existing compare takes &str,
which cannot carry a non-UTF-8 Rexx string (D14) and re-parses every call, defeating D15's cache.
That amendment is now named in Task 8 Step 3a rather than discovered mid-task. (2) The sized
interpreter thread moves from the rexx-run binary into rexx-exec's public entry point, because the
L0 and assertion-table harnesses run in-process on a cargo test thread whose stack is far smaller
than the depth limit is calibrated against.
Also: DO control numbers corrected (26.3 = FOR count, 26.2 = DO repetitor, 41.1 = non-numeric
initial/TO/BY, and `do i = 1.5 to 3` raises nothing); INTERPRET fragments get a plan key;
the coverage criterion's owner arm is policed; deviations are covered by the set assertion.

Task 2 dispatched at BASE fdc3a6e4 (rexx-core: value bodies, Body::trace, RootSet slot frames).
Task 1 review dispatched over e24b0e1d..cb80e2a9. Both in flight.

Task 1: REVIEWED CLEAN. Spec compliance PASS, code quality PASS, 0 Critical, 0 Important,
1 Minor (an extra debug_assert! in parse_interpret encoding the 47.1 measurement; accepted,
no fix round). The reviewer re-ran the 47.1 probes and the 392-test count itself rather than
trusting the report, and checked the comment consolidation against translate_block's two call
sites to confirm the generalised claims are true of every CodeBody and not only of main.

Task 2: pre-flight found a real brief gap. NotNumeric was named in the required Body::Text shape
but defined nowhere in the tree, and rexx-core cannot depend on rexx-exec without inverting the
dependency direction. Ruled: bare marker struct in rexx-core/src/body.rs, because nothing
observable distinguishes "not valid UTF-8" from "valid UTF-8 that is not a number" -- 41.1
substitutes the value and never says why -- and the byte-slice parse entry point coming in Task 8
removes the separate from_utf8 failure anyway. That is five consecutive pre-flights finding a
real defect in a brief I wrote.

Task 2: complete. Commits a3178cff (functional) + c7d51f1c (fmt). rexx-core 36/36, workspace
0 failed / 3 ignored, clippy clean. My pre-flight answer arrived after it had proceeded on its
own stated default, which matched the ruling, so no divergence. Two brief defects it found and
fixed: the Files list missed benches/heap.rs, tests/heap.rs, tests/trace.rs and tests/uninit.rs,
which also construct Body::String; and the brief's verbatim test text used
assert_eq!(x.is_some(), true) which clippy::bool_assert_comparison rejects under -D warnings.
Review dispatched over d243c9af..c7d51f1c.

Task 3 dispatched: the borrow-shape spike, BASE c7d51f1c.

Task 2 follow-up: 87fe09d8 adds Copy to NotNumeric and the reasoning doc comments (41.1 never
distinguishes the two failure causes; the coming byte-slice entry point removes the distinction;
one exact parse serves DIGITS 5 and DIGITS 20; None means "not yet asked"). Task 2 is three
commits: a3178cff, c7d51f1c, 87fe09d8. The reviewer's diff file stopped at c7d51f1c and was told
to fold 87fe09d8 in rather than report the doc reasoning as missing -- a stale review range would
have produced a finding that was true when dispatched and false when read.

Spec revision 7: Task 3's pre-flight found that six revisions said the slot frame grows and none
said where the grown slot's NAME is recorded. The plan cannot hold it (shared Rc, built by an
upfront pass). Activation gains extra: HashMap<Box<[u8]>, usize>. Proved not to be a corner case:
`interpret "newvar = 7"` then a plain `say newvar + 1` prints 8, so a fragment binding outlives
the fragment and is visible to the enclosing body. DROP (v) has the same shape.
Revision 6's (enclosing body, fragment id) plan key withdrawn: sound and useless, since fragment
text varies per execution so every lookup misses while every entry is retained.
Rulings to Task 3: extra map approved; no fragment arm in BodyKey; loud-failure exit code 120
(outside 157..253, below 126 so it cannot read as signal death); two entry points with the spike
one naming 4b as the deleter, NOP-hooking rejected as it lies about which node owns the fragment;
Outcome carries buffered stdout/stderr, which is also what makes it Send across the join.
t2-core asked to export NotNumeric (mod body is private) and to check SlotFrame for the same wall.

Task 2 second follow-up: efe5d2d2 exports NotNumeric and SlotFrame from rexx-core's lib.rs. Both
were private because mod body and mod roots are, and Task 2's own tests never had to NAME either
type -- an integration test can use a returned SlotFrame by inference. The wall only appears for a
second crate putting the type in a struct field, which is Task 6's Activation. Task 2 is four
commits: a3178cff, c7d51f1c, 87fe09d8, efe5d2d2. Reviewer given the final range and asked whether
the export set is now right rather than merely sufficient to compile Tasks 4 and 6.

Task 2: REVIEWED. Spec compliance PASS, code quality PASS. 0 Critical, 1 Moderate, 2 Minor.
The Moderate is a REPORT defect, not a code one: the report claimed "cargo test --workspace:
392 tests, 0 failed, 3 ignored (same as Task 1)". Real figure is 579 passed / 0 failed /
3 ignored. 392 is rexx-parse's own count, and Task 1's report never recorded an ignored count;
the 3 ignored are rexx-num's format tests. A measurement was reported as run when it was recalled
from another task and relabelled, which defeats rather than fails the exact check the review was
asked to make. I re-ran the workspace independently: green. Correction requested.
Minors: two structuring semicolons in comments (fix requested); the NotNumeric doc reasoning was
thin in the reviewed range and was already deepened by 87fe09d8, outside that range.

The reviewer went past the brief in a way worth copying: it wrote its own #[should_panic] test
for grow_slots on a non-top frame, because NEITHER of the brief's two tests exercises the panic
path at all. The brief asked for a panic and specified no test that could see it.

Task 3 pre-flight Q5 (NotNumeric/SlotFrame unexported) was CORRECT when raised -- verified at
87fe09d8, where line 21 read `pub use body::{BehaviourId, Body, Object};`. efe5d2d2 fixed it after
the question. The implementer then re-read the fixed file and retracted its own finding; I sent
the evidence back. A wrongly-retracted finding costs more than the gap did, and the lesson is to
ask whether the tree changed rather than whether you were wrong.

Task 14 first half dispatched in parallel (corpus programs only, file-scoped to rust/corpus/).

Task 2 re-review at HEAD (all four commits): verdicts unchanged, PASS/PASS, 0 Critical.
It verified 41.1's text against the primary source, interpreter/messages/errnums.xml ERR41
subcode 001, rather than trusting the commit message -- "Nonnumeric value ("value") used in
arithmetic operation", which substitutes only the value. So the doc comment's central claim is
now a checked fact. It also confirmed Copy is harmless by reasoning rather than assertion
(Box<T> is never Copy, so the containing Result stays non-Copy), and hand-diffed every pub item
in rexx-core against lib.rs's re-exports: nothing else is stranded, and SlotFrame's fields stay
private on purpose, matching FrameId's handle pattern.
New Minor: NotNumeric's doc says "eighteen-digit value" for 1.234567890123456789, which has
nineteen significant digits (eighteen after the point). Folded into the same fix commit as the
report correction and the two semicolons.

OPERATIONAL HAZARD, worth remembering: a crate directory containing only Cargo.toml with no src/
breaks cargo for the WHOLE workspace, because the root manifest globs members = ["crates/*"].
Task 3's in-progress rexx-exec scaffolding sat in that state and Task 2's reviewer had to
git archive HEAD into scratch to review anything. Rule: when scaffolding a crate in a shared
worktree, create src/lib.rs in the same step as Cargo.toml.

Task 2 fix round: e1317591 splits three structuring semicolons (the two the review named plus a
third on SlotFrame the implementer found itself) and corrects the report's fabricated workspace
figure IN PLACE rather than appending a note beside wrong numbers. Real figure confirmed twice
independently: 579 passed / 0 failed / 3 ignored, the 3 being rexx-num's multi-GB allocation
tests. The implementer named the defect as fabrication rather than a slip, which is the handling
this project wants: a report carrying a corrected figure is worth more than one that never
carried a wrong one, because the record now shows how it was checked.
Outstanding: one word at body.rs:63, "eighteen-digit" for a nineteen-significant-digit literal.
My message crossed with the commit.

Task 2 commits: a3178cff, c7d51f1c, 87fe09d8, efe5d2d2, e1317591.

Task 14a: complete, commit 7f8f6922. 15 new corpus programs, all verified twice against the
oracle for byte-identical stdout/stderr/rc. phase-4a.txt lists 25 qualifying programs. README's
hard-coded "24 programs" fixed to report rather than assert -- and it was already stale before
anyone touched it, the real count being 28.

Two findings, both verified by me:
1. My brief said "LEAVE naming an outer label". The agent measured that a CLAUSE label fails
   (28.3, since a clause label is a SIGNAL target) and generalised to "LEAVE names the control
   variable, not a label". Half right: `do label outer i = 1 to 3` + `leave outer` WORKS. That
   leaves Loop::label unconstructed by any corpus program, which Task 16's coverage criterion
   requires. One more program requested.
2. A DETERMINISTIC SEGV IN THE ORACLE, found while writing select_when_absorption. Four lines:
   select / when 1 = 0 then / when 2 = 2 then nop / end. rc 139, 3/3 runs, and rexxc parses the
   same file at rc 0, so it is a run-time defect. The true-condition variant is fine, so the
   crash is specifically falling through a FALSE WHEN whose THEN branch absorbed the next WHEN.
   Correctly kept out of the corpus (cannot be byte-identical with a crash) and documented in
   README's "learned the hard way". Recorded in memory as oorexx-orphaned-when-segv. UNFILED:
   filing upstream is Moritz's call.

Task 3: commits 5b5ccaf6 (spike, 982 lines in lib.rs + 235 test lines) and 41378f46 (fmt).
Report still said IN PROGRESS at the time, so no review dispatched yet -- reviewing a moving
target has cost this phase two stale ranges already. Status requested.
The commit message records what D19 demanded: 512 MiB stack, 784 bytes per eval level in debug,
192 in release, over a 100,000-term expression. Those are now carried into Task 11's brief.
It also independently hit the Activation::extra gap from its own pre-flight, which is the
revision-7 amendment, so the spike and the spec agree.
One question put to it before review: the commit says the non-compiling borrow shape is
"compiled here rather than paraphrased". A doc comment cannot fail to compile, so either that is
a compile_fail doctest or the claim is stronger than the tree. Better to settle it now than in a
fix round.

WORKSPACE IS RED, and the cause is a Phase 3 defect the corpus flushed out.
rust/corpus/lang/deep_nested_expr.rex (3001 terms) aborts cargo test -p rexx-parse --test program
with a stack overflow, SIGABRT. Diagnosed rather than guessed: parsing is ITERATIVE (expr.rs's
precedence loop) and the spike's rexx-run parses the same file fine on its 512 MiB thread, so
what overflows on a default 2 MiB test thread is the compiler's recursive drop glue for a
3000-deep Box<Expr> chain. rexx-parse has no impl Drop anywhere. Scheduled as Task 3b, which
must find the real cliff before fixing, add an iterative Drop, and check PartialEq/Debug for the
same exposure. Deliberately NOT fixed by shortening the corpus program: that would hide a defect
reachable from ordinary user code, and a stack overflow aborts with no message or exit code,
which is the one outcome D19 exists to exclude.

Second defect from the same run: the spike's loud-failure message {:?}-dumps the whole Expr, so
one unimplemented construct printed 364 KB. Fix requested: name the variant, not the value, plus
a length-bound test.

pop_slots: t2-core pushed back on my ask to pin 4a-scoped wording there, and was right. Popping a
non-top frame is LIFO discipline, true in 4a and 4b alike; only grow_slots carries the 4a-only
invariant. f5f0e2d5 stands as committed. It held the position after I repeated the ask without
engaging, which is the behaviour I want.

Task 3: DONE, commits 5b5ccaf6 + 41378f46. Shape holds. Review dispatched over 7f8f6922..41378f46.
Its five concerns, in order of consequence:

1. THE PARENTHESIS CLIFF -- the phase's biggest finding so far. Nested parens recurse in
   rexx-parse, which D19's eval-side counter cannot see. Measured by me: oracle rc 0 at 38,000
   parens, rc 245 Error 11.1 at 40,000. Ours succeeds to 85,000 (debug, sized thread) and aborts
   with no message at 90,000, so we diverge in BOTH directions on one axis. Crucially the oracle
   RAISES here rather than crashing, unlike the flat-term axis, so a counter in the parser is
   PARITY not a deviation. Spec revision 8 (b186ffa3) records both cliffs; Task 3c adds the
   counter, after 3b since both touch rexx-parse.
2. It confirmed my Q5 evidence independently: efe5d2d2 was needed, its own retraction was wrong,
   and rexx-exec would not compile without SlotFrame exported.
3. Workspace red has two causes, both from the corpus commit and neither from rexx-exec: the
   deep_nested_expr Drop overflow (Task 3b, dispatched; RUST_MIN_STACK=67108864 makes it pass,
   and the file's max paren nesting is 1, which independently confirms Drop and not parsing), and
   a missing sourceline expectation for exit_with_value.rex (routed back to t14-corpus).
4. SymbolId's inner u32 is private with no accessor, so nothing outside rexx-parse can use it as
   an array index. Task 6 must decide deliberately: add SymbolId::index() or keep the hash. Var
   lookup is 8.1%/32.2% of runtime, so this is not bookkeeping.
5. My brief's predicted E0502 was WRONG. The real diagnostic blames the loop condition, not the
   call argument, and it proved this by compiling a second wrong variant taking no arguments at
   all -- so "pass the body differently" is not the fix; where `body` is rooted is. Both are in
   run_activation's doc comment now.

Task 3b: implementer REFUSED the brief's diagnosis and was right. My "recursive drop glue" call
was an inference from evidence that could not distinguish the candidates (a stack overflow, plus
the spike's 512 MiB thread parsing the file fine -- true under every deep-recursion hypothesis).
It measured instead: mem::forget left the cliff at exactly 2,450 terms, ruling Drop out; the same
tree built without the parser survived 6-8x deeper; gdb named block.rs::visit_expr, a hand-written
per-clause walk called from add_clause, which runs DURING parsing.
Three recursions, three cliffs: visit_expr 2,450; drop glue ~10,000-20,000; parse_subterm parens
~85,000. Task 3b now covers the first two (visit_expr first, it is what blocks everyone), Task 3c
the third. Spec revision 9 and the plan both corrected; the plan had asserted my wrong diagnosis
in writing.
Requirement added to 3b: the iterative visit_expr must visit exactly the nodes the recursive one
did, since GUARD and exposed-variable handling read the set it builds.

Task 3: DONE at three commits (5b5ccaf6, 41378f46, 2838974d). 9 integration + 2 doctests.
Workspace 592/0/3 with RUST_MIN_STACK raised; still aborts on a default stack in rexx-parse's
corpus walk, which is Task 3b's.
Three things from its final round worth keeping:
1. The "compiled rather than paraphrased" claim was PROSE and it said so unprompted. Now a
   doctest pair. And it found a general trap: compile_fail,E0502 does NOT pin the error code --
   a snippet annotated exactly that whose body is `let x: u32 = "not a u32";` passes, and that is
   E0308. The must-compile twin narrows it; a typo inside the failing snippet's own line remains
   uncatchable and the doc comment says so. Saved as memory rust-compile-fail-doctests.
2. The AST-dumping message was 373,332 bytes, now 128, via an exhaustive form_name with no _ arm.
   The test asserts the PROPERTY (same message for 1-term and 3,000-term) plus a loose bound,
   rather than an exact byte count that would enforce the wording.
3. EVAL IS NOT WHAT BINDS THE STACK. Drop costs ~820 bytes/level against eval's 784, and phases
   unwind sequentially, so adding eval does not move the cliff. Honest budget ~860 bytes/level,
   ~620,000 levels. Task 11's numbers corrected. Consequence: a depth limit in eval CANNOT close
   the abort path -- `exit` then a 700,000-term expression aborts in the drop with nothing
   evaluated. Only Task 3b's iterative Drop closes it.
Recommended limit for Task 11: exactly 100,000, raising ABOVE it not AT it, since a 100,000-term
expression reaches depth 100,000 and that is the depth the oracle survives.

Task 14a: REVIEWED CLEAN. 16/16 programs deterministic and in scope; the 10 pre-existing
phase-4a.txt entries re-checked the same way; do_loop_forms is byte-for-byte do_variants minus
its one DO OVER block; all 16 sourceline_oracle expectations regenerated independently and
byte-matching. The review did the discrimination check properly rather than asserting it: for
each of the four control-flow shapes it traced what a wrong jump index would print and confirmed
the difference is visible (leave_nested_outer 7 lines vs 2; iterate_from_select gains a spurious
"keep 2"; if_else_chain bodies of 2/1/3/1 misplace text; select_when_bodies 3/2/1/2 bleeds one
WHEN into the next). It also ran the ordinary two-WHEN form beside the absorption file to prove
the outputs differ (42 vs 0) rather than trusting the claim, and reproduced the segfault 3/3.

ONE DEFECT, in documentation, and the propagation path is the lesson: the corpus agent reported
"47.2 if the label sits directly before the DO, 28.3 otherwise"; I repeated it in a correction
without testing that half; it landed in README's "learned the hard way". Verified now: BOTH
placements are accepted by rexxc and fail at the LEAVE with 28.3. 47.2 is a real error for a
label written INSIDE a block body (Error_Unexpected_label_do/_if/_select, "found within").
do_label.rex already said 28.3, so the corpus contradicted itself. Fix dispatched.
I checked the DO LABEL half because it changed what the corpus needed; I did not check the error
numbers because they changed nothing. Documentation-only claims get less scrutiny precisely
because they decide nothing, and they are what later readers trust. Saved to memory.

47.2 fix landed as 602af596. The agent re-measured both constructs itself before editing rather
than trusting my message, which is the right response to a correction that was itself about
propagating an unverified claim.
It also caught a coupling nobody had documented: editing a corpus program's COMMENT changes its
line count, which stales that program's committed sourceline_oracle expectation in a different
crate. Behaviour was byte-identical -- same output, same exit code, deterministic across runs --
so every check a careful person would think to run passed, and only the line count moved. Asked
for a README entry naming the regeneration command, since the next person has no reason to
suspect a typo fix breaks a rexx-parse test.

Task 4 (value model) dispatched in parallel with Task 3b, scoped to rust/crates/rexx-exec only.
Four agents in flight: 3b (visit_expr), 4 (value model), Task 3's review, and the corpus README.

Task 3: REVIEWED. Spec compliance PASS, code quality PASS, 0 blocking, 2 medium + 8 minor.
The review verified rather than accepted: reproduced all three E0502 variants (confirming the
brief's sketch was right for the argument-only case and wrong for the case in the file), mutated
the extra lookup and the fragment frame handling on a scratch copy to prove the fragment tests
fail for the right reasons, reproduced 784.0 exactly, and independently confirmed the
drop-binds-not-eval correction by bisecting rexx-run at 500k (rc 0) and 700k (rc 134).

Two findings the spike's own tests could not have caught:
M1 StackSpan's two ends can come from different Rust call chains -- stack_first is rewritten on
   every depth-1 entry, stack_deepest only on a new max. Measured: a 1000-term expression followed
   by an interpret reports 782.16 against the true 784.0, and the error runs in the UNSAFE
   direction, since a smaller bytes-per-level implies more survivable levels. Outcome.stack is
   public and its doc points Task 11 at it.
M2 run_activation binds code to its own activation but the pc to activations.last(). Balanced
   today; 4b's CALL pushes inside step, and this will not surface as a borrow error.
Plus m3: Loud::instruction's `_ => unreachable!()` would ABORT for a new keywordless
InstructionKind instead of failing loudly -- filed minor, but it contradicts a stated rule.
Also caught: at the reviewed commits the "compiled rather than paraphrased" claim WAS false, and
2838974d fixed it. The reviewer independently reproduced the compile_fail,E0502 trap.
Fix round dispatched: M1, m3, M2 first, then six comment items, plus #[doc(hidden)] on the spike
entry point and a note that a #[cfg(test)] mod could have proved the lifetime with zero public
surface.

Task 4 pre-flight (the eighth consecutive one to find a real brief defect): my test sketch called
eval_str/eval_with/settings_mut/Interp::new(), none of which exist, and building them literally
would pull Task 6's activation and Task 7's arithmetic into Task 4 as throwaway scaffolding.
Ruled option 2: construct Numbers through rexx-num directly, no evaluator. Two clarifications
that dissolved rather than patched the question -- number() takes created_digits and created_form
explicitly, so the value model needs no ambient Settings at all and Task 6 changes nothing here;
and tests go in a #[cfg(test)] mod inside src/value.rs so Interp stays private, which is Task 3's
review lesson applied from the other direction. Plan corrected.

Task 3 review round 2: verdicts unchanged PASS/PASS. One recorded measurement wrong, three doc
items. The 820 bytes/level figure was an artifact of a 100,000-wide bracket; bisected to +/-5,000
all three shapes survive 629,687 and fail at 634,375, so ~850, and NEITHER eval NOR Plan::note
moves the cliff -- stronger than the claim it replaced. Task 11's plan text corrected; the ~860 /
~620,000 operational figure stands and is conservative.
Also confirmed with a controlled pair that compile_fail,EXXXX is inert: an E0308 body annotated
E0502 passes, alongside the same body correctly annotated. And a precondition nothing in the tree
states -- rustdoc DOES collect doctests on private items, so if it did not both would silently
not exist rather than fail. Both saved to memory.
New doc finding: the doctest miniature can drift from run_activation (models Rc<CodeBody> over
Vec<u32>; the real one has Rc<Program> and a three-field Code<'a>), so the honest framing is that
the compiler keeps the FUNCTION honest and the pair keeps the DOCUMENTATION honest. Forwarded.
The reviewer also checked the other two loud paths unprompted: both bounded, one because
keyword() returns &'static str, one because ParseError deliberately carries no substitutions. So
the size contract holds on all three paths, which is better than the tree claims.

Task 3b: DONE, commit 0f33843a. Three walks made iterative, not the two I named:
  block.rs::visit_expr -- needed ExprKind::for_each_child's lifetime named explicitly
    (for<'a> ... &'a Expr), since an explicit worklist must stash a yielded reference past the
    call producing it; elided it was E0521.
  Expr's Drop -- iterative via a new for_each_child_mut twin, mem::replace into a worklist.
  tests/gate_walk/mod.rs::each_expr -- THIRD walk, named by neither of us, shared by tiling.rs and
    variants.rs, same shape, same corpus file. It fixed this without asking and flagged the
    decision explicitly. Right call: test-only, mechanically identical to the approved changes,
    and leaving it would have kept 2 of the 3 named tests red.
  Subtlety it caught: each_expr's doc promises parents-before-children, so the fix pushes children
  in reverse before popping to reproduce the ORDER, where visit_expr only had to preserve set
  membership (referenced is a BTreeSet read via .contains).
Verified by me: rexx-parse fully green including the new tests/deep.rs (5,000 and 100,000 terms).
Parse now handles 500,000 terms on a default 2 MiB thread, against 2,450 before.

KNOWN LIMIT, recorded not fixed: Debug, PartialEq and Clone on Expr are still recursive, cliff
~2,000-2,500 -- SHALLOWER than the bug just fixed. I checked reachability: the only {:?} in
non-test code formats a byte string, and nothing in rexx-exec clones an Expr, so it is test-only
today. Trigger for scheduling it: the first test that formats or compares a deep tree.

Coordination error of mine: I dispatched Task 4 into rexx-exec while Task 3's fix round was still
open in the same crate's lib.rs. Resolved by sequencing -- Task 4 writes src/value.rs only and
holds its one-line lib.rs edit until Task 3 lands its doc round. Task 3's behaviour fixes are in
as 5244d6f4 and the crate compiles.

Task 3b: REVIEWED. PASS/PASS, 0 blocking, 0 major, 2 minor, 2 informational.
The review is the model for verifying a semantics-preserving rewrite: it compiled the PRE-DIFF
recursive walk verbatim alongside the new iterative one via #[path], ran both on a 512 MiB thread
so the recursive reference survived, and compared visit sequences BY NODE ADDRESS across every
parseable program in rust/corpus and all of samples/ -- hundreds of files, including the
6,001-node deep one -- plus crafted trap shapes (super_class with omitted args and a bracket
message, cascades, logical comma lists, >v references). Identical everywhere.
It also checked the visit_expr set-membership claim at every read site rather than accepting it:
`referenced` is written once and read once, in compound_is_cached's single .contains, so order and
duplication are unobservable. Valgrind on the iterative Drop: 0 errors, 0 bytes leaked.
Independently re-measured: parse at 500,000 terms ok; deep.rs's oracle claims (100,000 prints
100001 rc 0; 150,000 exits 139) accurate; derive cliffs exact at both edges -- Debug ok 2,000 /
abort 2,050, PartialEq and Clone ok 2,100 / abort 2,200.
Residual now precisely scoped rather than vaguely feared: the only non-test path reaching a
recursive derive on a deep tree is the panic!("... {other:?}") sites formatting an InstructionKind,
which fire only when the parser is already panicking, so the worst case is a panic message
becoming a silent abort.
Minors: depth_probe's doc omits the third cliff (Task 3c's) while the report claims it records
three, and the report contradicts itself about whether depth_probe was kept. Both folded into 3c.

=== SESSION-LIMIT WAVE, 2026-07-30 ~15:13. Three agents died, reset 20:20. ===

Task 4: COMMITTED BY ME as 75990fc9. Its agent finished, staged, and died at the commit step.
I verified before committing: rexx-exec 8 unit + 10 integration + doctests, clippy -D warnings
clean, both rc 0 unpiped, and the full workspace at 617 passed / 0 failed / 3 ignored.
Its real discovery, worth keeping: Number's fields are pub(crate) to rexx-num, so SmallInt
admissibility cannot be decided by inspecting the magnitude. It decides by RENDERING instead --
format_form(created_digits, Scientific), admit only if no '.' and no 'E' -- which makes a
SmallInt's rendering and a Body::Num's rendering provably the same computation rather than two
rules that agree today. A refused exponential rendering is discarded rather than seeded into the
text cache, because the probe renders in Scientific and the object's real form may be Engineering
(D15's 1E+10 vs 10E+9). Verified against the oracle that 20.00 + 0 keeps its point and is
therefore correctly refused.
Concern it flagged for Task 7: Body::Num's created_digits is u32 while Settings allows DIGITS up
to MAX_WHOLENUMBER (999_999_999_999_999_999), so the narrowing needs a deliberate decision rather
than a silent `as u32`.

Task 15a: UNCOMMITTED AND UNREVIEWED, left in the working tree deliberately. 735 lines across
rexx-extract (lib.rs +330, extract.rs +20, a new assertions binary, a new test file). Tests pass
(11 + 3) but the agent NEVER WROTE A REPORT despite the report-first instruction, so there is no
record of what it verified or what it knows is incomplete. Not committing unreviewed work with no
report. When that agent resumes it must write the report first, then finish and commit.

Task 3c: DONE by t3b-drop before it died, commit 6285d98f. Review dispatched to t3-spike.
Oracle paren cliff is a NOISY bracket [39,900, 39,950], non-deterministic in between, confirmed
by 3x reruns. Our cliff: 88,800 ok / 89,000 abort sized; 337 ok / 338 abort on a DEFAULT thread,
two orders of magnitude below the oracle's. MAX_PAREN_DEPTH = 50,000 raising 11.1, and error.rs
needed no change since 11.1's text is already in the generated table.
The counter only protects a sized caller: on a 2 MiB thread the native abort at 338 fires long
before the check at 50,000 could. Documented in three places plus a test that documents the gap
rather than closing it.
Step 4, two gaps flagged and deliberately unfixed: prefix chains (- - - -1) abort at 1,150-1,200
on a default thread, and nested calls f(f(f(...))) abort at 350-360, SHALLOWER than plain parens
and through a different arm than the counter guards. Nested DO/END is not recursive at all
(Vec<Frame>, verified to 100,000); SELECT/WHEN reaches the same loop but was read, not measured.

The pipe hazard bit twice more: `cargo build ... | tail -1` swallowed a FAILED build, so a
bisection measured a stale binary and produced wrong cliffs -- the same shape that made a
reviewer's round-2 numbers wrong. Saved as memory shell-pipe-exit-status.

=== RESTART AFTER THE 20:20 RESET ===
Task 3c: REVIEWED PASS/PASS, 0 Critical, 3 Medium, 3 Minor. The review measured from a CLEAN
EXPORT of the commit rather than the working tree, and its first run caught a binary two minutes
stale -- the unpiped-build discipline earning its keep on contact. It tested a midpoint of the
"noisy" oracle bracket (39,925 gave 245 0 245 across three runs), confirming genuine
non-determinism rather than a sloppy bisection, and it closed the report's one open caveat by
measuring nested SELECT/WHEN to 100,000 clean -- noting that its first attempt used
select/otherwise, which raises 7.1 BEFORE any nesting happens and would have looked like a pass
while measuring nothing.
M3 promoted to Task 3d (2d067c2a): nested calls descend through arg_list, not the guarded arm, and
on the SIZED path we abort with no message above ~92,000 where the oracle reports 11.1. The
report had deferred this partly because the oracle's cliff was unknown; the review measured it,
removing the reason. The prefix-chain gap stays deferred on the review's own argument: it aborts
only on a default thread and every in-tree consumer is on a sized one.
M1: the default-thread paren cliff is 331/332, not 337/338 -- the counter's own field and check
cost ~6 levels, so THE FIX MADE THE UNPROTECTED CASE SLIGHTLY WORSE, and one of the four places
carrying the stale number is a test whose stated job is documenting that gap accurately.
M2: "nested calls are shallower than plain parens" is backwards; measured like-for-like, parens
331 and calls 349.
Useful context it established: the corpus's deepest actual paren nesting across 12,103 files is 5.

Dispatched after the reset: Task 3d to t3-spike (it idled without starting the first time), Task
15a resumption to t14-corpus (report FIRST, then finish and commit), Task 5 (stems) to t4-value.
Three lanes, disjoint crates: rexx-parse, rexx-extract, rexx-exec.

CORRECTION TO AN EARLIER LEDGER ENTRY: I wrote that Task 15a's agent "NEVER WROTE A REPORT".
The file was genuinely absent when I checked at ~15:14 and is present now with an mtime after the
20:20 reset, so both observations were true at different times and my entry was unfair as a
characterisation. The agent flagged the discrepancy rather than accepting my description, which is
the right instinct even when the disagreement is with me.

Task 15a: COMMITTED as f8ab5386. All three hard parts done with their own tests: sequential
DIGITS carry-through (extended to FORM as well, since 5 of 11 groups use ENGINEERING before some
assertions -- flagged as a deviation rather than done silently), CONCATENATION's assignment
prelude, and per-group produced/dropped counts pinned as a test rather than left as a percentage.
4,269 assertSame calls, 4,259 rows, 10 dropped, matching the 1,226 PRECEDENCE / 388 CONCATENATION
figures exactly.

AND IT FOUND A PHASE 0 DEFECT. rexx-extract's method scanner stripped only double-quoted method
names, so every ::method 'name' was invisible: 8,888 test methods across 122 of 409 files.
Task 0.4's measurement therefore missed 10,459 of 24,581 methods, 43% of the suite, silently --
an unseen method is absent from BOTH columns rather than reported as unextractable.
Re-measured with the fixed extractor and regenerated l1-coverage.md wholesale (60630407):
24,581 methods, 22,251 extractable, 90.5%, against the recorded 14,122 / 12,176 / 86.2%.
D8 survives and is BETTER supported, since the corrected fraction is higher against a 40% bar.
The mechanism is the lesson: its extractor asserted rows + dropped == calls and that invariant
panicked on the first real run. A percentage cannot notice a missing population; a conservation
law can.

Task 5: design pre-flight approved, the ninth in a row to find a real defect in a brief of mine --
tail_key's stated signature does not compile, since resolving a Tail::Variable piece needs the
Code<'_> bundle. Its q.1='x' probe is a BETTER justification for Body::Stem.name than the spec's:
it never bare-assigns q., so at alias-read time no reference site exists to derive a name from.
Asked for that probe to go in the doc comment rather than the report.
Task 2b added from the same pre-flight: RootSet cannot express an unset slot.

Task 3d: DONE, commit ae7e8bce. Verified by me: 11 uses of MAX_EXPR_DEPTH, MEASURED_NATIVE_CLIFF
= 88_800 with a const assert pinning the limit below it, the deepest-nesting-is-5 sentence in the
doc. My "not started" check at 21:18 predated its 21:26 commit; both views were right about
different moments.
ONE SHARED BUDGET, argued as a correctness requirement rather than a preference: the two
recursions differ in code path and per-level cost but agree in spending the same stack, so two
budgets of 50,000 would let f((f((...)))) reach 100,000 real levels and abort as if uncounted. The
test puts half the budget in each construct, so only a shared counter can answer it.
Verified BY MUTATION: with the arg_list guard removed the call test aborts the binary and the
shared-budget test fails, while the four pre-existing tests still pass -- so neither new test
rides on an old one.
Oracle's call cliff [34,500, 34,760], sharp on both sides, unlike its noisy paren bracket. Note
the two implementations order the constructs OPPOSITELY: the oracle's call cliff is lower than its
paren cliff (34.6k vs 39.9k) while ours is higher (92k vs 89k), so no "calls cost more" story
holds for both.
M1 DEMONSTRATED ITSELF A SECOND TIME: 3d's own guard moved calls from 349/350 to 341/342 while
parens stayed at 331/332. Every counter makes the unprotected case shallower than the measurement
that justified it, and that sentence is now in the tree beside the numbers.
It deliberately did NOT edit Task 3c's report, the fourth M1 site, on the ground that its numbers
were true for the code they measured and rewriting them destroys the record of when. Endorsed and
saved to memory as a general rule.
It also caught its own invented precision: a first draft quoted "350/351" for a configuration it
never built, where 3c had only a 10-wide bracket.

Dispatched: Task 2b to t3-spike (rexx-core, nobody else there), Task 3d's review to t3b-drop
(knows the crate, did not write the commit). Task 5 live with t4-value in rexx-exec.

Task 2b: DONE, commit 4ce5aae8. rexx-core 41 passed (38 + 3), workspace 622/0/3. Verified by me:
the .nil probe reproduces (drop on a variable HOLDING .nil returns it to the derived name, so the
two states are observationally distinct) and all three tests exist including
growth_does_not_recycle_a_cleared_slot.

THE TECHNIQUE WORTH KEEPING: it BUILT THE PLAUSIBLE-WRONG DESIGN FIRST -- a `cleared: HashSet`
consulted only by `slot`, leaving `iter` alone -- and ran the candidate tests against it. The
obvious test ("does the read answer unset?") PASSED against the broken version; only the
collection test failed. A suite carrying just the obvious test would have shipped an object rooted
by nothing, surfacing whenever a collection next landed. Saved to memory as a distinct move from
mutating finished code.
clear_slot chosen over an Option-taking set_slot on call-site grounds: DROP is a construct the
language has, so clear_slot(frame, i) says what it means where set_slot(frame, i, None) reads like
a caller that forgot to compute a value -- and every existing call site stays untouched rather
than taking a mechanical Some(...) wrap, which is where a wrong edit hides.
growth_does_not_recycle_a_cleared_slot pinned as a REQUIREMENT, not an observation: a cleared slot
still belongs to its name, so DROP a then a = 1 must land in the same place, and recycling would
alias two variables onto one slot with the symptom appearing as one variable's assignment
changing another's.
Ruled on its fmt question: folding fmt into the commit was right BECAUSE the reflow touched only
lines that commit added. The rule exists to stop a sweep hiding churn in unrelated files; with no
unrelated churn it has nothing to protect. Split whenever fmt reaches a line the commit did not
otherwise touch.
Unblocks plain DROP in Task 9 and 4b's NOVALUE. Was never blocking Task 5.

Dispatched to t3-spike: the phase-4-exclusions.txt file (Task 16 Step 4), docs-only so it avoids
all three crates with live work. Flagged for it that 11.1 has TWO statuses -- parity on the paren
and call axes where the oracle also raises it, deviation on the flat-term axis where the oracle
segfaults silently -- which the spec does not spell out.

phase-4-exclusions.txt: WRITTEN BY ME (7aff84f4) after t3-spike idled mid-task for the third
time. Its partial report had scoped the work and stopped at the section it called "The depth row".
Faster to write than to re-dispatch a fourth time.
Contents: 15 whole exclusions (11 Phase 7, 4 Phase 10) leaving 66 of 81; 3 partial rows; 2
deviations. The depth row needed the care I had flagged -- 11.1 is PARITY on the paren and call
axes, where the oracle raises it too, and a DEVIATION only on the flat-term axis, where the oracle
segfaults with no condition and there is no number to reproduce.
Writing it surfaced a THIRD STATUS the spec had no home for: a KNOWN GAP is a measured divergence
with no owner -- prefix chains, the still-recursive derives, and the counter protecting only a
sized caller. Filing those as exclusions would imply someone owes them; as deviations, that
someone chose them, which is worse since a deviation is meant to be permanent. Spec revision 10
records it (the file had three sections while the spec described two).

CONVENTION ADOPTED, from t3-spike's suggestion: every task report's FIRST LINE is now
`STATUS: IN PROGRESS | DONE | BLOCKED`, nothing else on it, updated as the task moves. One grep
tells me every task's state instead of inferring it from a file's size or a commit's absence.

The race it prompted, checked by timestamp rather than impression, ran the opposite way from its
reconstruction: I looked at Task 3d at 21:18 and ae7e8bce landed at 21:26; I committed the
exclusions file at 21:38 and its report reached its complete 125-line form at 21:41. Both my
observations were accurate when made, and nothing was duplicated -- it verified my file rather
than writing a second, which is what the tool's read-before-write rule bought. Offering the
coordination data rather than defending itself was still the right instinct, and the convention is
worth having regardless since the cure is cheap and the failure is expensive.

Its verification of the exclusions file singled out the right claim: the stem order
`1 B 3 2 ZZ 10` appears NOWHERE in the spec (D15a gives `1 10 2 3 B ZZ` only as the BTreeMap
contrast), so the file asserted something with no prior source. Reproduced and correct.
Its sharpening of the asymmetric pin is worth keeping: for exclusions and deviations the hazard is
rows being ADDED to excuse yourself; for known gaps it is rows being REMOVED, so a gap quietly
deleted reads as a gap closed. Same assertion, opposite directions.

Dispatched: a spec-versus-plan-versus-tree consistency audit to t3-spike. The spec is at revision
10 and the plan has been amended eight or nine times, mostly by me under time pressure, and three
stack figures have already been superseded. Read-only, no crate conflict.

OPEN FOR MORITZ, both surfaced and neither actioned:
  * A self-referential symlink interpreter/interpreter -> .../interpreter, created 21:38 by an
    agent. Untracked, not oracle content, but inside the read-only tree, and a recursion trap for
    anything that walks it. Awaiting his word before deletion.
  * The unfiled orphaned-WHEN segfault (four lines, rc 139, rexxc-clean). Filing is his call.

Task 3d: REVIEWED PASS/PASS, 0 blocking, 0 major, 1 minor (comment style). Three things above the ask:
  * It BUILT THE REAL ALTERNATIVE -- a second call_depth field independently capped at 50,000, the
    design a reasonable person would have chosen -- rather than just removing the guard. Five tests
    pass against the wrong design and exactly one fails, which both proves the shared budget is a
    correctness requirement and identifies which test carries the claim.
  * A NEW MEASUREMENT TRAP, now in memory: raising MAX_EXPR_DEPTH in the current tree to find the
    native cliff gives numbers several hundred to 1000+ levels SHALLOWER, because the guard's own
    per-level bookkeeping sits on the stack being measured. Archiving 0f33843a and 6285d98f and
    building each reproduced the cliffs to the exact integer. A guard added to protect a limit
    moves the limit.
  * It reported its own mistake: the self-referential symlink in interpreter/ was its `ln -sf`
    while setting up a scratch copy. Caught by its own final git status, removed, written down.
    That closes the question I had escalated to Moritz. Confirmed gone.
CI note worth keeping: once a test aborts the process, every sibling in that binary stops being
reported, so an aborting test silently truncates its neighbours. Does not bite today since the
shipped code never aborts.
It also independently recounted the deepest-paren-nesting-is-5 claim across all 12,103 files,
reproduced the select/otherwise 7.1 trap, traced every Parser::new site to confirm expr_depth
cannot be bypassed by a fresh Parser mid-recursion, and confirmed Task 3c's report still reads the
original 337/338 untouched.

Dispatched: Task 4's review to t3b-drop. Task 4 (75990fc9) had NO review -- its implementer hit a
usage limit at the commit step and I committed its staged work after verifying tests and clippy,
which is not a review. Everything downstream depends on the value model, so it is the most
consequential unreviewed thing in the phase.

Task 5: DONE, commit 28a62383. Six functions in stem.rs plus a Body::Stem arm in to_text. A bare
stem read needed no new code -- it goes through the ordinary variable-read path unchanged.
IT CORRECTED ITS OWN APPROVED PRE-FLIGHT DESIGN while implementing. "stem_assign replaces the
object" cannot explain a.=1; b.=a.; a.1=2; say b.1 -> 2: if b.=a. wrapped a.'s VALUE as b.'s new
default, a later tail write through a. would never show through b. The only model that works is
that b. shares the SAME Body::Stem object, so stem_assign now checks whether the assigned value is
already a Stem rather than always wrapping.
Verified by me against the oracle, including the case it invented and I had not given it: after a
fresh a. = 9, b. still answers 2 and 1 (the old object, old default) while a. answers 9 and 9. That
is what separates "shares the object" from "copied the value". Its original design would have
failed on the FIRST transcript.
Its scoping call was right: Task 5 is the standalone library, and dispatching Stem/Compound from
eval_node and step is Task 7's and Task 9's. The comment at lib.rs:965 that says "Task 5 takes the
others" is stale and it is fixing it in its next commit.

Dispatched: Task 6 (the resolution plan) to t4-value, with the SymbolId question put to it
explicitly rather than left to be answered silently -- the inner u32 is private so the plan is
name-keyed, and variable lookup is 8.1%/32.2% of runtime, so a SymbolId::index() accessor plus a
dense array is worth ASKING rexx-parse for if it judges that way.

SPEC/PLAN/TREE AUDIT: DONE, 8 findings, fixed as 9b11836d.
Finding 1 is the one that mattered and was caught by minutes: Task 6's Interfaces specified
Plan { slots, len } while the tree has had Plan { names, by_symbol } since Task 3. by_symbol was
absent entirely, and it is the field the hot path uses -- evaluation reaches a slot through the
SymbolId the AST already carries instead of hashing a byte string per access, which is the exact
path D16 justifies the design by costing at 8.1%/32.2%. The contradiction sat INSIDE a paragraph
asserting the opposite: "Task 3 built this; you are inheriting its shape, not inventing one",
true of extra and false of Plan in one breath. Task 6 was already dispatched; implementer stopped
before writing plan.rs.
Root cause is real ambiguity in D16, not carelessness: "one HashMap per plan, through which both a
SymbolId and tail pieces resolve" admits both "at build time" and "on every access", and Task 6
resolved it without noticing there was a choice.
Finding 4 sharper than reported: the SymbolId constraint is not merely undocumented, it is already
in the tree at lib.rs:418 with "Task 6 either adds that accessor or keeps the hash; recorded here
so the choice is made rather than inherited". I had asked the implementer that question as if open
without reading the comment.
Finding 5: Task 9 never named clear_slot, which Task 2b landed to unblock it, nor said writing
ObjRef::NIL is wrong. Fixed with the measured transcript.
Others: criterion 7 credited "the Task 1 spike" (the spike is Task 3; spec said Task 1 in three
places, Task 3 in two); D19's two-cliffs table described removed behaviour in the present tense,
now as-of-marked with the corrected 40,000-50,000 band; parse_subterm -> subterm,
MAX_PAREN_DEPTH -> MAX_EXPR_DEPTH, "three recursions" -> four.
AUDIT HEADLINE WORTH KEEPING: the measurements kept up and the names did not. Every stack figure
agrees across documents and matches the tree, including two superseded twice in one day, and the
exit code appears as a bare integer nowhere. Drift was in names and task numbers -- a much better
failure mode, and it tells us what to re-check first next time.

Dispatched: Task 5's review to t3-spike (unreviewed, and the one task where the implementer
overturned its own approved design mid-implementation).

Task 5: REVIEWED. Spec compliance PASS with one Important, code quality PASS, 0 Critical.
THE DEFECT IS IN THE PAIR I ASKED IT TO HUNT. stem_get uses stem_name for two different jobs --
finding the slot, where the READ SITE's spelling is right, and deriving the name of an UNRESOLVED
tail, where the OBJECT's is. Once b. and a. share one object those diverge. Verified by me against
the oracle: a.1='x'; b.=a.; say b.2 gives A.2 not B.2; two hops give G.9; and m.1='x'; n.2='y';
m.=n.; say m.1 gives N.1 because m.'s own object was discarded. Control: p.1='x'; r.=p.; drop r.1;
say p.1 gives P.1, where site and object agree, which is why the code looks right. Never-touched
zz.5 gives ZZ.5, which is why the unset early return must keep using stem_name.
The module STATES the rule -- "never a copy of the read site's spelling" -- and to_text applies it
for a bare stem read while stem_get does not for a tail read.
WHY NO TEST REACHED IT, which is the transferable part: the aliasing test is the only one that
aliases and every read in it RESOLVES, while derived_tail_name is reachable only when a tail does
NOT resolve. The aliasing test and the tombstone test never meet, and each is individually
correct. Hunting pairs rather than rules is what found it.
Important not Critical only because the #[allow(dead_code)] is honest -- nothing outside the
module's tests calls it yet. It becomes wrong output the moment Task 9 wires ExprKind::Compound,
and the existing tests will still pass when it does.
The reviewer also re-ran the design correction's evidence and added a case nobody had: two-hop
aliasing (k.=h.=g.) gives G.9, which the share-the-object model gets right for free and a wrapper
model would not -- so the correction is load-bearing beyond the seven transcripts that prompted it.
Minors: three unreachable! sites format a whole Body with {:?}, which for a Stem is its entire
tails map (same class as the message defect 3d bounded, lower stakes on an abort path); rule 4's
case-sensitivity is tested in one direction only, and the other direction is correct.
Stale comment of mine flagged: stem_drop's doc still argues no API can write "unset", which
clear_slot (4ce5aae8) made false.

MY OWN ERROR, recorded: commit 9b11836d swept in another agent's staged source edits. I staged
only docs/superpowers, but `git commit` commits the whole INDEX, and that agent had already staged
body.rs and stem.rs. Content correct and verified, message silent about them. In a shared worktree
use `git commit -- <paths>` or read `git diff --cached` before committing.

Task 4: REVIEWED. PASS/PASS, 0 blocking / 0 major / 0 minor -- the cleanest of the phase, reached
by proving rather than agreeing:
  * It PROVED the Scientific probe cannot bias SmallInt admissibility, by reading format_with's
    trigger logic and finding the exponential-versus-plain decision is computed BEFORE form is
    consulted. That turns "the probe form is arbitrary" from a plausible claim into a structural
    one.
  * It found the trap in the spec's own wording: D15 says "whole", which invites a conversion
    check, but 1.00+1 gives 2.00 and 20.00+0 gives 20.00, so the oracle's "whole" means RENDERS
    WITH NO DECIMAL POINT. The implementation is right for the measured reason, not the written
    one -- the word in the spec is the weaker artifact.
  * The ambient-settings check was STRUCTURAL: Interp has no settings field at all at that commit,
    so a future caller has nothing to reach for by mistake, and that stays true once Task 6 puts
    Settings on the Activation.
It also re-verified all seven oracle transcripts, confirmed the tri-state cache has exactly two
touch sites, and confirmed the u32/u64 digits mismatch is real and correctly left to Task 7.
Isolation recipe worth spreading, from its near-repeat of the ln -sf mistake: `git archive`
ALREADY includes the whole tree, interpreter/ included, so no symlinking is needed at all. That
assumption is what put the stray symlink in the real repo during the 3d review.

Dispatched: Task 15a's review to t3b-drop (rexx-extract, no collision with live Task 6 work).

Task 6 pre-flight, two decisions, both approved:
  1. SymbolId::index() -- APPROVED and scoped as Task 6b to t3-spike. Its argument, which I
     verified in token.rs rather than accepting: intern assigns SymbolId(u32(names.len())) and the
     PUBLIC name() indexes names[id.0 as usize], so the raw value is already a dense zero-based
     table-local index and an accessor pins no new invariant. It missed that `pub fn len()` already
     exists at token.rs:134, which makes the Vec sizeable exactly rather than incrementally.
     Task 6 proceeds with the HashMap (Task 3's shape) and the Vec swap is a follow-up.
  2. Deferring blocks: Vec<Block> to Task 11 -- APPROVED. My Task 6 Interfaces listed it, but
     Block's only definition anywhere is the design doc's DO/LOOP passage, which is Task 11's to
     build. Adding it now is the same throwaway-scaffolding shape the eval_str correction ruled
     out. GENERALISED RULE, told to the implementer: if a field's shape is defined only by a task
     that has not run, adding it now is scaffolding, not inheritance.
It also fixed the stale lib.rs comment more precisely than I asked -- Task 9 for the
assignment-target dispatch, Task 7 for eval_node needing to evaluate those forms first, since
those are genuinely two pieces of work.

Task 4: REVIEWED PASS/PASS with zero findings, recorded above.
Dispatched: Task 6b (SymbolId::index) to t3-spike; Task 15a's review to t3b-drop.

Task 6b: DONE, commit 180875a9. SymbolId::index() exposed, rexx-parse 399 passed.
IT CORRECTED MY ARGUMENT, and the correction matters: I said name() already panics on an id from
another table, so the accessor adds no hazard. That holds only when the value is OUT OF RANGE. An
in-range id from a different table returns a different symbol SILENTLY, with no panic anywhere,
and index() makes that easier to reach because a bare usize travels where a SymbolId cannot. The
doc now separates the two cases and names Fragment's own table as the concrete way to hit it,
which is better than the doc I asked for.
Test verified by mutation: a stride-2 intern fails it with [0,2,4,6,8] against [0,1,2,3,4], and
EVERY OTHER TEST IN THE CRATE STILL PASSES under that mutation. So the density guarantee had been
load-bearing in public API with nothing pinning it, and a later change would have surfaced inside
a consumer's Vec as a panic or a silently wrong slot.
Two notes forwarded to Task 6: Option<usize> is still required because keywords, labels and
constants share the SymbolTable, so a dense Vec has holes that must stay distinguishable from slot
0; and lib.rs:418's comment now describes finished work and should record which way the choice
went. It was right not to touch that file, which Task 6 owns today.

Workspace is red on rexx-exec (E0063, missing field `settings` in Activation) -- Task 6's expected
in-flight state. --exclude rexx-exec gives 603 passed / 0 failed, clippy clean.

t3-spike told to HOLD: everything remaining (7, 8, 9-11, 12, 13) is in rexx-exec, which Task 6 has
live, and the gate harnesses need an executor that runs more than SAY. Offered it an optional
note recording the git-archive isolation recipe and its known exception -- rexx-parse's test binary
include_str!s CoreClasses.orx and StreamClasses.orx, so an isolated build of that crate needs those
paths present.

Task 15a: REVIEWED PASS/PASS, 0 blocking, 0 major, and it CORRECTED A CLAIM I HAD REPEATED IN
THREE PLACES. "All 388 CONCATENATION assertions would silently pass" overstates it: with a..g
unset each renders as a distinct single-character name, so only rows whose expected value matches
that all-distinct pattern coincide -- the 56 strict ==/\== rows. The other 332 use non-strict = and
fail VISIBLY (line 71 expects 0 0 1 1 0 1 0, which unset operands do not produce). Fix unchanged,
since every row needs the prelude either way; the reason to be precise is that a reader who checks
"all 388" and finds 332 loud failures concludes the hazard was imagined. Corrected as 03c10606.
Its verification: reverted the single-quote fix in a scratch copy to prove rows+dropped==calls is
ENFORCED rather than computed (both sites panic naming file and shortfall), traced all 10 dropped
rows to source, and rebuilt the "old rendering executes nothing" claim with a deliberately-wrong
assertion that should have exited 1 -- got rc=0, no output.

Task 6: DONE, commit ca005497. Plan/BodyKey/ProgramId moved to plan.rs, Activation to
activation.rs, shape kept as corrected (names/by_symbol). No blocks field, by_symbol still a
HashMap pending the Vec swap. Review dispatched to t3-spike.
It caught a bug in its OWN first test draft: names_are_keyed_upcased_but_tail_values_are_not
called slot_of with hand-written differently-cased bytes and asserted equal slots, which is false
-- slot_of never upcases, that happens once upstream in SymbolTable::intern before a SymbolId
exists. THAT CORRECTION HAS A CONSEQUENCE FOR TASK 9, now in the plan (5931aa18): DROP (v) never
goes through the scanner, and measured, v='x'; x=1; drop (v); say x prints X, so the VALUE is
upcased before it names a variable. A path handing slot_of raw bytes misses the existing slot and
silently allocates a second one for the same variable.

ISOLATION NOTE written (isolating-a-build.md) and it found more than the anecdote it was asked to
record: rust/corpus/ is a COMPILE-TIME dependency of rexx-parse (13 include_str!s in
instruction/tests.rs, precedence.tsv in expr/differential.rs, trace_output.rex via include_bytes!),
so copying only rust/crates/ breaks the build; and the .orx files are needed by --lib, not only
--all-targets, since the include_str! sits in a cfg(test) mod in src/directive/. Consequence it
recorded rather than left to be found: an isolated build PINS THE CORPUS to whatever state it
copied -- right for reproducing a commit, a trap when copying a working tree while another agent
adds corpus files, which was our situation twice today. Saved to project memory.

=== PICKUP STATE, 2026-07-31 ~00:15. All three agents out until 02:00. ===
HEAD 7a628261. Tree CLEAN. Workspace 635 passed / 0 failed / 3 ignored, clippy and fmt clean,
all verified unpiped with exit statuses read directly.

Task 5's defect fix: FINISHED AND COMMITTED BY ME (7a628261). Its implementer died mid-edit,
leaving the tree UNCOMPILABLE -- one unreachable! site converted to body_variant_name(...) and the
helper never written. I wrote the helper, converted the other two sites, and hit one trap worth
recording: inserting the function directly above `impl Interp` put it BETWEEN the
#[allow(dead_code)] attribute and the impl block it governs, so the allowance landed on my function
and every method in the impl became a dead-code error. Moved it above the attribute's comment
block. An attribute and its target are a unit; do not insert between them.

DONE AND REVIEWED: 1, 2, 2b, 3, 3b, 3c, 3d, 4, 5 (+fix), 6b, 14a, 15a, the exclusions file, the
spec/plan/tree audit, and isolating-a-build.md.
DONE, REVIEW INCOMPLETE: Task 6 (ca005497) -- t3-spike's review died at STATUS: IN PROGRESS, so
re-dispatch it rather than assuming a verdict.
NOT STARTED: Task 7 (brief extracted at task-7-brief.md, dispatched, agent died before beginning).
REMAINING AFTER 7: 8 (comparison, needs rexx-num's byte-slice compare entry point first), 9-11
(instruction loop), 12 (errors), 13 (trace), 15b (assertion harness), 16 (gate harnesses).

CARRY INTO TASK 7: the u32/u64 digits narrowing needs a deliberate decision (Body::Num's
created_digits is u32, Settings permits DIGITS to MAX_WHOLENUMBER); route arithmetic through
rexx-num rather than reimplementing; every unimplemented ExprKind still fails loudly through the
exhaustive form_name; and the temps discipline is inherited, not invented -- step roots every eval
result in a frame that lives exactly one clause.
CARRY INTO TASK 9: an indirect DROP name is upcased before it resolves (5931aa18), and clear_slot
is what "dropped" means, never ObjRef::NIL.

The orphaned-WHEN segfault is CLOSED as an open question: Moritz established it is already SF
#2018, built trunk, found the crash site at EndIf.cpp:142 where else_end is NULL in the
KEYWORD_ENDWHEN branch, verified jfaucher's guard fixes it, and posted a comment. Do not re-raise
it or file anything.

=== RESTART AFTER THE 02:00 RESET (dispatched ~07:30) ===
Three lanes, disjoint crates:
  t4-value  -> Task 7, expression evaluation part one (rexx-exec/src/eval.rs)
  t3-spike  -> Task 6's review, restarted (it died at STATUS: IN PROGRESS; told to recover or
               start clean and say which)
  t3b-drop  -> Task 8a, a byte-slice comparison entry point in rexx-num
Task 8a is scoped OUT of Task 8 deliberately so it unblocks rather than follows: the existing
compare takes &str, which cannot carry a non-UTF-8 Rexx string (D14), and re-parses per call,
which defeats Body::Text's tri-state cache. Both new paths must reach the one string_order
implementation -- duplicating it would move the divergence this task prevents inside rexx-num.
Its tests must include a non-UTF-8 operand, since that is the case the current signature cannot
express at all and therefore the proof the new one is needed.

Task 6: REVIEWED. Spec compliance PASS with two Important gaps, code quality PASS.
I1 THE UPFRONT PASS IS EMPTY FOR STEM/COMPOUND BODIES. note handles Variable/Prefix/Binary then
`_ => {}`; build handles Assignment/Say/Interpret then `_ => {}`. Measured: "say a.b", "a.1='x'"
and "q.=1" all produce a plan of len 0, while "say v" produces len 1. So any body with a stem or
compound gets nothing from the plan and every name is created by grow_slots one at a time -- THE
LAZY ALGORITHM D16 DEFINES ITSELF AGAINST -- and the performance argument goes unrealised exactly
where it was measured, since D16's 32.2% figure is stem-heavy code.
Both `_ => {}` arms carry comments moved verbatim from Task 3's spike promising that TASK 6 would
close them, so they now read as the task promising itself.
I2 NO TEST PINS THE PASS. Neutering Plan::build to return an empty plan leaves 19 of 20 tests
passing, the single failure being about fragments rather than the pass. None of plan.rs's four
tests asserts a built plan's CONTENTS, so every property they check is reachable through the
fallback -- which is what let I1 through.
The tail-piece property DOES hold (b=2; say a.b gives A.2) because resolution is name-keyed and
both paths reach slot_of, not because the plan arranged it.
Minors: the by_symbol comment calls the accessor decision "pending" though SymbolId::index()
landed at 180875a9, two minutes before the commit, and the comment even predicts its spelling; and
the blocks deferral is right for a wrong stated reason -- the spec DOES define Block at line 382,
and `settings` is equally unread, so the stated reason does not distinguish the two fields.
Fix round dispatched to t4-value, to follow Task 7 rather than interrupt it. The bar for the new
test is explicit: neutering Plan::build must fail it.

Task 8a: DONE, commit 5bf9b03d. compare_bytes and compare_decoded added, and the structure is the
point: `compare` (the &str version) is now itself a one-line call into compare_decoded, so there is
exactly ONE place numeric_order and string_order are invoked from regardless of entry point. That
makes "no second copy" architectural rather than a discipline someone maintains.
string_order retyped &str -> &[u8] as a PURE signature change -- its body already worked on bytes,
the &str was only ever the public constraint -- proven by ten pre-existing tests passing unmodified.
compare_decoded borrows a caller-supplied Number rather than cloning it, so the case it exists to
make cheap does not quietly cost a clone.
Chose Option<&Number> over a three-state type, with the reasoning stated: a confirmed-not-numeric
fast path is a different and smaller optimisation nobody asked for, and rexx-core::NotNumeric is
not reachable from rexx-num since the dependency runs the other way.
It traced WHY the first transcript row discriminates rather than asserting it: "pad the shorter on
the right" compares padded 'a ' against ' a' and disagrees at the first byte (0x61 vs 0x20), where
"strip leading blanks" reduces both to "a" -- and confirmed none of the other eight rows can tell
the two rules apart. Added a genuinely non-UTF-8 case (a lone 0xC3) since that is the gap the &str
signature cannot express at all.
Two defeat-the-mechanism tests: one supplies a Number that DISAGREES with what its bytes would
parse to, proving the parameter is consulted rather than silently re-derived; the other confirms a
supplied Number does not leak into strict comparison, which compares text rather than a value.

STILL OUTSTANDING and blocking Task 7's four error tests: ArithError::sub_code is still private at
lib.rs:142. My request crossed with the completion message. Re-sent, together with the request for
a test pinning the power-operator asymmetry (2**'x' and 2**2.5 both 26.8, 'y'**2 and 'y'**'x' both
41.1, base checked first).

Task 8a addendum: DONE, commit 4a320f1c. sub_code made pub, matching the shape Task 7's own
in-progress error.rs doc comment had already requested rather than inventing the sub()/code()
split I offered -- reading the consumer's stated need instead of choosing freely.
It declined to change code()'s self-by-value signature for symmetry, having found real callers in
the associated-function and function-pointer forms (ArithError::code(e) in bin/muldiv.rs and
bin/addsub.rs, .map_err(ArithError::code) in two tests) which do not auto-ref the way method
syntax does. Four files would have broken for something nobody asked for.
It also flagged a latent compile error in Task 7's UNCOMMITTED draft: From<ArithError> calls
error.code(), which takes self by value, then error.additional() and match error -- a
use-after-move the moment mod error; is wired in. Adopting sub_code() resolves it as a side effect.

THE POW ASYMMETRY IS NOT REXX-NUM'S, AND THAT IS THE FINDING. pow's signature is
pub fn pow(&self, exponent: &Number, digits: u64) -- BOTH operands are already-parsed Numbers, so
pow structurally cannot receive a non-numeric one. 2**'x', 'y'**2, 'y'**'x' and the base-first
ordering can never be exercised by any call that crate can make; the routing is decided one layer
up by whichever caller parses each operand, which is Task 7. It pinned only the two rows genuinely
its own (2**-1 and 2**2.5 -> the full (26,8) pair through the new accessor), recorded the four-row
exclusion in the test's doc comment, and named why a later simplification cannot unify the paths
without changing pow's signature to accept unparsed text.
Consequence sent to Task 7: implement the ordering deliberately -- parse base first, 41.1 on
failure, then exponent where non-numeric and non-whole both fold into 26.8 -- and put the four
transcripts in ITS comment, since rexx-num cannot express those cases.

Dispatched: Task 6's fix round to t3b-drop, scoped STRICTLY to rexx-exec/src/plan.rs since Task 7
is live in eval.rs/error.rs/lib.rs in the same crate. Acceptance bar stated explicitly and taken
from the reviewer's own mutation: neutering Plan::build to return an empty plan MUST fail the new
test, and the report must say what fails with and without it.
Dispatched: Task 8a's review to t3-spike, with the pow finding singled out for a second opinion --
if "pow cannot see a non-numeric operand, so the 41.1/26.8 split is the caller's parse order" is
wrong, Task 7 is currently implementing something twice.

Note: Task 7's code (error.rs, eval.rs) is still UNCOMMITTED in the working tree. Commit 58a62152
was the plan doc only. If that session dies, the same recovery applies as for Tasks 4 and 5:
verify unpiped, then commit on its behalf with the substance credited.

Task 8a: REVIEWED PASS/PASS, 0 Critical, 0 Important, 1 Minor. rexx-num 147 passed.
THE POW CLAIM IS CONFIRMED IN SUBSTANCE AND WRONG IN ARITHMETIC, and I propagated the error.
It is THREE rows and three, not two and four. Caller's, because a non-numeric operand cannot be
represented as an argument to pow at all: 2**'x' (26.8), 'y'**2 (41.1), 'y'**'x' (41.1). The
crate's, because both operands parse and pow really decides: 2**2.5 (26.8), 2**-1 (0.5), 0**0 (1).
All three of the crate's are pinned -- the new test covers 2**-1 and 2**2.5, and 0**0 was already
at tests/pow.rs:25 -- so no coverage is missing, which is why this is Minor. The report's HEADING
said "two of six ... four structurally cannot be" while its own bullet list below grouped three as
rexx-num's; I repeated the heading's figure into Task 7's dispatch. Corrected there.
The linchpin, reconfirmed: 2**'x' is 26.8 and not 41.1, so a non-numeric EXPONENT is a whole-number
complaint while a non-numeric BASE is a nonnumeric-value one -- exactly the asymmetry pow cannot
see, because it never receives unparsed text.
The review also improved on the report's own evidence: the &str -> &[u8] retype was defended with
"ten unmodified tests pass", which only covers tested cases, where the diff shows the body already
worked on bytes throughout and the only change is .as_bytes() moving from inside the function to
outside -- no reachable difference on tested OR untested cases.
And it noted the implementer caught its own weak assertion before shipping: 1 < 5 and 1 < 999 are
both true, so that comparison pinned nothing; it changed to 6 < 5 false, where 6 < 999 would be
true, which actually discriminates whether the supplied Number is consulted.

Task 7: DONE, commit 3a9d6446. Task 6 fix: DONE, commit 3cf3f03c. Workspace 661 passed / 0 failed
/ 3 ignored, verified by me unpiped. Review of Task 7 dispatched to t3b-drop.

The Task 6 fix went past its brief in the right direction: build now registers every name an
instruction's fields COULD name, not only the three kinds this phase executes, because matching
only current kinds and writing "Task N will handle this" on the rest is exactly the
self-promising-comment anti-pattern that caused the original defect. It read eval.rs and stem.rs
first to establish how each shape actually resolves, which is why the fix reaches
Redirection::Stem and the Drop/Expose/Procedure/Use Local targets rather than only my four repro
cases. Mutation run as asked: neutering Plan::build gives 36 passed / 2 failed, the new content
test failing immediately on "say a.b" alongside the fragment test my earlier mutation found alone.
It refused to touch activation.rs, outside its stated scope, and flagged the correction instead --
with a sharper diagnosis than mine: the comment claimed Block had no real definition WHILE LISTING
THAT DEFINITION IN THE SAME SENTENCE. Fixed by me as 8485f725; the deferral stands, the reason is
now "nothing reads it yet, and Task 11 should pick the representation against a real reader".

Criterion 1 gained two enumeration rules and one raiser (73ecb4f6): take variant identity from the
variant and never from keyword(), since it maps both When and WhenCase to "WHEN" and a test keyed
on it lets any WHEN satisfy WhenCase; Operator::Backslash carries an owner string rather than a
witness, being prefix-only with dyadic \ as 35.1; and trace 5 outside interactive debugging raises
24.901, so Trace::Skip is a raiser rather than a silent no-op, which settles that all four Trace
variants are 4a's.
The three List-using subset programs (num/digits_rounding, num/exponential, num/operators) must
come OUT of phase-4a.txt -- confirmed, say (-2) ** 3 , (-2) ** 2 prints two lines at rc 0, so a
comma builds a list SAY prints element-wise, which the spec assigns away from 4a. Gap analysis
being re-run with them excluded, since removing them can only uncover more variants.

Criterion-1 gap analysis: DONE, re-run with the three List programs excluded. Nineteen in-scope
variants unconstructed, and THE GAP LIST DOES NOT CHANGE when the three are dropped -- they were
contributing nothing to coverage, which settles drop-versus-rewrite. Three new programs close
everything.
Task 7's depth re-measurement is the fourth value that figure has taken in two days: ~820, ~850,
~783, now ~1600 bytes per eval level after eval_node grew to fifteen match arms (~335,000 levels,
still 3x D19's minimum). Every value was correct for the code that produced it. Task 11's text now
states the RULE -- re-measure at implementation, never quote a predecessor's figure -- with the
superseded number kept beside it so a reader finding ~783 elsewhere knows it is history rather
than a contradiction (8d9791e9).
Task 7 also handled the shared worktree correctly: it found Task 6's uncommitted fix in plan.rs,
left it untouched and unstaged, and flagged it rather than absorbing it into its own commit. That
is the failure I caused yesterday by running `git commit` after staging only my own paths.

Dispatched: Task 14b (three corpus programs + dropping the three List entries) to t14-corpus, with
the three traps spelled out -- Trace::Skip raises 24.901 so a program using it exits non-zero, no
DO OVER on a stem, and every added or edited program needs its sourceline_oracle expectation
regenerated even for a comment-only change. Task 6's fix re-review to t3-spike, which found the
original defect, with its own mutation as the bar and an instruction to re-run it rather than
accept the reported number.

MY COORDINATION ERROR: I queued the Task 6 fix round with t4-value ("fold these in after Task 7")
and then dispatched the same work to t3b-drop without cancelling the first. t3b-drop landed it as
3cf3f03c; t4-value arrived after, checked before acting, found it done, and left the existing
report alone rather than clobbering it. Cost was one context load, not lost work.
Its detour found a real residual neither commit could have caught: activation.rs's `settings`
carried #[allow(dead_code, reason = "nothing reads or mutates this yet -- Task 9's NUMERIC is the
first to touch it")] while Task 7's eval.rs already read it for every arithmetic result's
rendering. Fixed as 639c1f43.
GENERAL LESSON, passed on: an allow whose REASON names a future task becomes a lie the moment that
future arrives, and nothing in the build checks it. #[expect(...)] warns when the lint does NOT
fire, so it self-expires -- but only where the item is unused in EVERY compilation. Where the only
readers are cfg(test) ones, as in stem.rs, expect is wrong because the lint fires in the library
compilation and not the library-as-test one, and the unfulfilled expectation is its own warning.
That distinction is now recorded rather than rediscovered a third time.

Gap analysis final: nineteen variants (up from sixteen purely from the Trace ruling), three
programs. Trace::Skip terminates the program -- rc 232, clause echo then two error lines -- so its
witness is its own program ending there, while bare `trace` and `trace value 'N'` are quiet at
rc 0 and fold into program A.
Four lanes live: Task 8 (eval.rs), Task 14b (corpus), Task 6-fix re-review, Task 7 review.

Task 6 fix: RE-REVIEWED PASS/PASS, 0 Critical, 0 Important, 2 Minor. The defect is closed.
The reviewer re-ran the mutation itself (36 passed / 2 failed, the right two) and, decisively,
measured the EMPTY rows rather than reading the diff: `leave lbl` registers nothing, because a
LEAVE label is not a variable, and `say .nil` registers nothing. Those are the cases a
mechanically-wider pass gets wrong, and both are right. Also `say a.b` -> ["A.", "B"] not
["A.B"], and `a.1 = 'x'` -> ["A."] only, since a bare digit tail is a constant piece.
It measured the id/name split rather than inferring it: say a.b gives 2 names and 0 ids; q. = 1
gives 1 and 1; drop a.b.c gives 3 and 0. An id is registered exactly where evaluation looks one
up, and a tail piece has none because compound_parts yields borrowed text rather than a token.
And it checked the nine extracted helpers changed nothing, including that slots are still assigned
by first mention -- a helper that renumbered would be invisible to a name-set assertion while
moving every slot.
m1, queued with the implementer: the new test asserts each expected name is PRESENT and never that
anything else is ABSENT, so over-registration would pass it -- the same gap that let the original
`_ => {}` through. Closing it with assert_eq! on the sorted key set for `leave lbl` and `say .nil`,
chosen because an EMPTY expected set cannot pass vacuously.

The reviewer put on record that the self-promising comments blamed for the original defect were
ITS OWN, written in Task 3's spike as a handover and carried verbatim into Task 6. Its conclusion
generalises and is now in memory: REMOVE THE PLACE A STALE COMMENT CAN LIVE rather than wording it
more carefully -- an exhaustive match has nowhere to put "Task N will handle this", where a
`_ => {}` arm is an invitation to write one.

Dispatched: Task 12 (the error subsystem) to t3-spike, scoped to error.rs since Task 8 is live in
eval.rs, with the measured format, the 256-major rule confirmed across five majors, and the raiser
list flagged as open-by-construction rather than authoritative.

Task 14b: DONE and committed by its author as eeca9951 -- three programs in, three List-using
programs out of phase-4a.txt, 26 entries either side. I verified independently before its commit
landed: each program run twice for byte-identical stdout/stderr/rc (rc 0, rc 0, rc 232), the
subset arithmetic, and cargo test -p rexx-parse green including the sourceline oracle. My own
commit attempt found nothing left to stage, so no duplicate.
Convention note: its report read STATUS: DONE while the body was still only a plan and nothing was
committed, which nearly had me commit on its behalf. DONE should mean committed AND written up;
otherwise the convention invites exactly the recovery work it exists to prevent. It completed
within minutes, so no harm, and the report is now 196 lines.

SECOND SCHEDULING COLLISION OF MINE, caught before damage: I dispatched Task 12 into
rexx-exec/src/error.rs while Task 8 had uncommitted changes in that same file (adding the 34.6
logical-list constructor its work needs). Resolved by splitting on activity rather than on file:
Task 12 holds all error.rs edits and does the half that needs no file -- capturing the oracle's
exact stderr and exit code for every raiser family into committed expectation files, plus two
things nobody has established, namely what the clause echo prints for a multi-line clause, a
commented clause and one inside a DO, and what appears when the error is not on line 1. Task 8
commits when it reaches a sensible point rather than splitting artificially.
Rule for me: check for uncommitted work in a file before dispatching a task that owns it. `git
status` answers it in one command and I have now failed to run it twice.

Task 14b closed. Two findings from it, one scheduled as Task 14c:
INFRASTRUCTURE GAP: the documented sourceline_oracle driver uses .Package~new, which EXECUTES the
file's prolog -- and trace_numeric_request.rex's whole point is a prolog that raises 24.901, so the
driver dies before printing. A driver that works only on programs that succeed is not a driver for
a corpus that deliberately contains failures, and this phase has just started writing such programs
on purpose. The agent worked around it locally with a SIGNAL ON SYNTAX + LINEIN fallback, verified
it does not change behaviour for non-crashing files, and correctly left the module doc alone as
out of scope. Now scoped as Task 14c to fold in.
Same mechanism as the standing rule never to instantiate .Package~new on a repository file: it has
been safe only because corpus prologs were trivial, and this is the first file whose prolog does
something. The next could do worse than raise.
Also in 14c: README's "Phase 4a additions" and num/ tables are behind phase-4a.txt after three out
and three in.

Convention refined and passed on: STATUS: DONE means committed AND written up. A DONE marker over
a plan-only body nearly had me commit on an agent's behalf as a rescue it did not need.

## 2026-07-31, controller, after three implementers died on the 5h session limit at once

t4-value (Task 8), t3-spike (Task 12) and t3b-drop all failed with "session limit, resets 11:50".
None had committed. I preserved their work rather than waiting:

Task 6 minor m1: complete, committed d7633695. The exact-key-set assertion for `leave lbl` and
`say .nil`. Task 6's remaining minors unchanged.
Task 12: commit eb9eb0c8 was its own; its orphaned Cargo.lock committed as b9430637.
STATUS: DONE stands, but see the open wiring item below.
Task 8: committed 9843c90d. NOT marked complete -- the implementer never reported or self-reviewed,
so this still needs its task review. What I verified myself before committing, so a reviewer knows
what is already covered: 22 programs against build/bin/rexx beyond the agent's own tests, values and
raised major.sub agreeing on all of them, including `0 & (1/0)` raising rather than short-circuiting
and `'01' == '1'` being 0 while `01 = 1` is 1.

One test was red on the whole tree, `a_loud_failure_message_does_not_grow_with_the_expression`,
and its cause is a plan-level defect worth naming: a test whose fixture is "a form that is not
implemented yet" gets its witness implemented out from under it by the next task. It began on `+`,
moved to `=` for Task 7, went red for Task 8. Every operator is inside 4a, so the fix was to take
the witness outside the phase entirely (a message send, Phase 5's) and move the coverage it lost --
the two form_name arms that call format! -- into a unit test that needs no unimplemented form.
Commit a6abb0d7, which also deletes the "Task 3's spike evaluates ... only" enumeration from both
Loud constructors and the module doc, all three of which were false by Task 6.

Rule for whoever writes the remaining briefs: do not let a test depend on a sibling task NOT having
happened. And `cargo fmt -p <crate>` is package-wide -- it reformatted a live sibling's file this
session. Use `rustfmt <path>` while others are live in the same crate.

OPEN, and the next thing I am doing: Task 12's ClauseSite/report/exit_code are still dead code.
execute() in lib.rs prints "raised SYNTAX 42.3 (not yet rendered: Task 12)" and exits 120 where it
should render the catalogue message and exit 256-major. Every raising program therefore still
differs from the oracle on stderr and rc, while already agreeing on the major.sub. t3-spike scoped
the work precisely and offered to do it "once Task 8 is out of the crate"; Task 8 is now out.

Tree at a6abb0d7: 682 tests pass, clippy clean, fmt clean, working tree clean.

CLOSED the open wiring item above, myself, since the three implementers are out until 11:50.
Commits 56e60d96 (the wiring) and ef683ab2 (three comments that named Task 12 as a future owner).

A raised condition now produces the oracle's exact three lines and 256-major exit code. Verified
byte for byte on eleven programs, stdout and stderr and rc: ten match exactly, the eleventh is `IF`
and still loud because Task 10 has not run. The mechanism the two agents could not share is
Interp::failure_site, recorded by the instruction loop because run() pops the activation before
execute() sees the error, and storing the resolved line and clause bytes rather than a span because
only a loop knows which source its spans index into.

run_program now takes the program's path; rexx-run canonicalises it. Measured: the oracle prints
the absolute dot-normalised path, identical for `./sub/../sub/rel.rex` and for a bare `rel.rex` run
from that directory.

New KNOWN GAP in phase-4-exclusions.txt: the report prints one clause echo, the oracle prints one
per nesting level. Measured on an INTERPRET fragment, which gives two, innermost first, both on the
enclosing clause's line. Unreachable from the corpus, since the fragment spike is the only nesting
4a has and run_program does not expose it. 4b's CALL is where the shape decision belongs.

Task 12's report says STATUS: DONE and stays that way. This wiring is not part of its diff and is
recorded here instead, per the rule about not rewriting a finished task's report to match later
code.

Tree at ef683ab2: 683 tests pass, clippy clean, fmt clean, working tree clean.
Still open, in order: Task 8's review (code committed at 9843c90d, never reviewed), then tasks 9,
10, 11, 13, 15b, 16, then Task 6's remaining minors and the Plan::by_symbol HashMap -> Vec swap.

## Dispatched 2026-07-31, two lanes

t8-review (fable, read-only): Task 8's review, range d7633695..9843c90d. It never got an
implementer self-review, so this is the only gate it will see. Told it explicitly which 22 programs
I already checked, and that the comma list is the weakest evidence: ExprKind::Logical's
short-circuit and its 34.6 rest on the implementer's own tests written from the same understanding
as the code.
t9-run (sonnet): Task 9, the instruction loop. BASE ef683ab2. Permitted files run.rs, lib.rs,
bin/rexx-run.rs. Tasks 10 and 11 queue behind it, all three owning run.rs.

Four defects in Task 9's brief, corrected in the dispatch rather than left to be discovered:
(1) the Produces signature is stale in three ways -- it says `step(&mut self, body: &CodeBody,
index: usize) -> Result<Flow, Raised>` where the code has `step(&mut self, code: &Code<'_>,
instruction: &Instruction) -> Result<Flow, Failure>`, and following the brief literally would break
the borrow discipline the crate exists to prove; (2) Flow already exists in lib.rs; (3) SAY,
Assignment, bare Exit and the Interpret spike are already implemented, so Task 9 extends and moves
rather than creates; (4) Step 5's git add names tests/run_basic.rs, contradicting the brief's own
Files section and the global constraint against integration-testing a private subject.
Also carried into the dispatch: the exit-code wrap fix, which lives only in a rexx-run.rs source
comment and would not have reached the implementer.

INDEPENDENTLY VERIFIED the 34.x rule myself, since three later tasks inherit it and Task 8's
evidence for it is self-referential. Twelve oracle probes, and the rule holds exactly as the report
states it:

  if 'x' then nop                 34.1      select; when 'x' then nop; end     34.2
  do while 'x'; end               34.3      do until 'x'; end                  34.4
  say 2 & 1                       34.901    ANY multi-element comma list       34.6

Short-circuit confirmed: `if 0, 'x' then say 'reached'` and `if 1, 0, 'x' then nop` both exit 0.

One case the report does not have, and it is the discriminating one: `if 'x', 1 then nop` is
**34.6, not 34.1**. So the sub-number is decided by the clause being a list at all, not by which
element failed, which rules out "the first element is evaluated as if it were a single expression".

For Task 11: `do until 'x'; end` echoes **`end`** as the failing clause, not the `do` line, because
UNTIL is tested at the bottom. That falls out correctly from failure_site taking the executing
instruction's clause_span, but only if the UNTIL condition is evaluated while executing the End
instruction rather than eagerly.

Task 8: REVIEWED PASS/PASS, 0 Critical, 2 Important, 3 Minor. Review at task-8-review.md.
Task 8: fix round 1 complete, commit 34f2a4d6. Task 8 is now COMPLETE.

Neither Important was a behaviour defect. The reviewer verified the shipped mapping and the
shipped short-circuit against the oracle and both are right; what it found is that the suite would
not have noticed if they were not. Three fixes, all applied by me since the implementer's session
is gone:

* Seven of eighteen compare_op rows unpinned, because equality is the only case separating
  LessEqual from Less and every operand pair in the mapping test was unequal. A measurement gap
  that became a test gap: the fourteen oracle measurements behind that test have the same blind
  spot. Seven assertions added, each value measured against the oracle first.
* strict_comparison_never_calls_to_number could not fail, and its comment's failure story was
  false. compare_decoded returns on op.is_strict() before reading either Number, so the property
  the name claimed is a performance one that no result can observe. Renamed to what it does pin,
  with the ordinary form beside the strict one.
* The short-circuit test's 'x' could not distinguish skipping evaluation from skipping the check.
  Now (1/0), with `if 1, (1/0)` as a control.

Verified the fixes rather than trusting them, which is the whole point of a fix round that
introduces defects two times in three here. Re-ran all eight previously-surviving mutations: all
eight now killed. Re-ran the is_strict_compare one: still survives, which is what the new comment
claims, so the honest fix was right and inventing an assertion would have been wrong. And proved
the sharper claim instead of asserting it -- against a mutant that evaluates every element and
checks only to the first false one, the OLD 'x' probe passes and the NEW (1/0) probe fails.

Two review findings by the reviewer that go beyond this task, worth carrying:
* The short-circuit skips EVALUATION, not just the check. `if 0, (1/0)` and `if 1, 0, (1/0)` both
  exit 0 on the oracle. Tasks 10 and 11 build the instructions that own these conditions.
* `'y' & (1/0)` gives 42.3 with byte-identical stderr on both sides, which pins the
  evaluate-both-then-check-left ordering `&` and `|` have.

Reviewer's stated coverage boundary: GUARD/34.5, FUZZ interaction, invalid UTF-8 through
eval_compare end to end, my own 22-program set (taken as stated), and GC rooting under allocation
pressure (pattern-matched only; the ?-skips-pop_frame shape is pre-existing crate-wide, not new
here, and is still open as Task 7's Minor).

## Corpus baseline, 2026-07-31, at 34f2a4d6

Ran all 26 entries of corpus/phase-4a.txt through both interpreters, comparing stdout, stderr and
exit code. **3 pass, 23 fail, and every one of the 23 fails as a loud "X is not implemented" with
no output before it.** Nothing produces a wrong answer, which is the failing-loudly criterion doing
exactly its job: the gap is visible rather than silent.

The 23 partition cleanly by owner, with no surprises outside the queued tasks:

  DO / SELECT / IF        11 files   Tasks 10 and 11
  TRACE                    5 files   Task 13
  NUMERIC                  3 files   Task 9
  stem assignment          3 files   Task 9 (stem.rs exists; eval has no ExprKind::Stem arm yet)
  EXIT with a value        1 file    Task 9

So Task 9 should take this from 3 to about 10, Tasks 10 and 11 to about 21, and Task 13 closes the
rest. Re-run this loop after each task lands rather than at the gate, since a number that only
appears at the end cannot show which task moved it.

Worth noting what this measurement does NOT establish: passing means byte-identical on the three
observable channels for the programs in the subset, and the subset was chosen to exclude what 4a
does not implement. It is a progress signal, not the phase gate.

Task 8 fix round: RE-REVIEWED CLEAN. No new findings at any severity. **Task 8 is closed.**

The re-review was stronger than the first pass in two ways worth copying. It ran the seven mapping
mutations INDIVIDUALLY rather than combined, so each new assertion is shown to discriminate on its
own rather than the set being killed collectively. And it rebuilt the decisive mutant itself --
evaluate every element, check only up to the first false -- rather than accepting my report that
the new probe catches what the old one could not.

It also corrected me: I listed three factual claims in my comments for it to check, and the third
("UNTIL-style evaluation not being involved") appears nowhere in the committed diff. I invented an
item on my own check-list. The reviewer said so rather than quietly checking two of three, which is
the behaviour to keep asking for -- a controller's list of what to verify is itself unverified.

One thing it did not re-run, and said so: my claim that the OLD 'x' test passes against that
mutant. The old suite is superseded, so nothing rests on it. I did run it, and it does pass.

Correction to the corpus baseline above: I wrote that the three stem rows fail because "eval has no
ExprKind::Stem arm yet". That is **false**, and I inferred it from an error message instead of
reading the code. `eval.rs` handles `ExprKind::Stem` at line 86 (sharing the arm with `Variable`)
and `ExprKind::Compound` at line 95, and `say a.b` on an unset compound correctly prints `A.B`.

What actually fails is the assignment TARGET. `step`'s `Assignment` arm takes `ExprKind::Variable`
and returns `Loud::expression` for anything else, which is why the loud message names an expression
form and misled me. Its own comment already states the split: Task 5 built the `stem_assign` and
`stem_set` library, and recognising a Stem or Compound target and dispatching into it is Task 9's.

The conclusion in the table was right and the reason was wrong, which is the more dangerous of the
two: an owner nobody re-derives, attached to a mechanism nobody re-checks. Same class as the probe
errors that keep showing up here. Read the code, not the error message.

## Temps-frame investigation, 2026-07-31: DOCUMENT, DO NOT RESTRUCTURE

Full analysis in temps-frame-investigation.md. Task 7's parked Minor is **not** a defect, and the
reasons are worth keeping because two of my own assumptions were wrong.

Seven push_frame sites. Six leaky, all in eval.rs, all one shape (eval_prefix, eval_arithmetic,
concat, eval_compare, eval_logical, eval_logical_list). The seventh, step_in_temps_frame, captures
step's result and pops UNCONDITIONALLY, and its doc comment already names this hazard as the reason
the frame closes there rather than inside step. So it is six places plus one deliberate healer, not
"everywhere" -- which is exactly why I asked for a table instead of a characterisation.

Read from rexx-core rather than inferred: push_frame is FrameId(temps.len()) and mutates nothing;
pop_frame is temps.truncate(watermark). A later pop with an outer watermark therefore unwinds every
leaked inner frame as a side effect, and no corrupt state is representable. Nothing asserts temps
balance anywhere, though the slot side of the same file does assert.

**My belief was right in verdict and wrong in mechanism, and the difference is the whole finding.**
I said the leak is benign because a raised condition terminates the program. It is benign because
step_in_temps_frame truncates the moment the failing instruction returns, long before execute
renders anything. Termination is not doing the work. Proof: the crate's own tests already continue
after a raise on the same Interp, and the eval_condition test helpers bypass the wrapper and
genuinely leak, invisibly, because alloc_with never collects and nothing asserts balance.

Both my candidates for when it stops being benign came up empty, each for a reason I would not have
found:
* 4b's SIGNAL ON SYNTAX cannot accumulate leaks. A trap acts at instruction-loop level, and the
  wrapper has already truncated before the Failure reaches the loop's Err arm.
* A collector arriving does not make it a correctness problem. This defect only ever OVER-roots,
  which is a retention cost under mark-sweep or moving collection alike. The correctness hazard in
  a root set is UNDER-rooting, which is a different discipline and already documented as the
  "unrooted from here to the caller's push_temp" window.
* Nothing in Tasks 9-11 reaches it sooner: IF/SELECT/DO/LOOP compile to Flow::Goto pc jumps, so
  every condition re-evaluation of every iteration runs under a fresh wrapper.

The Drop guard **does not compile here**, which is the answer to "can it be written" rather than
"is it elegant": the guard must hold &mut RootSet across self.eval(&mut self), two live &mut
borrows. The escapes are a raw pointer (unsafe for no strict need) or RefCell-ing RootSet (relaxing
the borrow discipline to fix a non-defect). A with_temps_frame closure helper does compile, but it
is insurance rather than a correctness fix and is not worth scheduling on its own.

SCHEDULED, small: a comment on pop_frame that truncation semantics are load-bearing and that an
assert must never be added there without balancing the six eval sites first; a comment in eval.rs
that the ? skips are deliberate and Tasks 10 and 11 should copy the pattern. OPTIONAL, after Task 9
lands: a debug tripwire in step_in_temps_frame asserting temps balance on the Ok path, which
catches the one shape the wrapper currently masks in silence, a success-path leak. Needs a
temps_len() accessor.

CAVEAT, and I have passed it to t9-run as a constraint: every conclusion above rests on both
instruction loops routing through step_in_temps_frame. Task 9 is moving those loops into run.rs
right now. If the rewrite moves execution off that chokepoint, this analysis has to be redone
rather than assumed.

Operational note: t8-review sent its investigation verdict, then sent the identical verdict again
after I re-sent the job. The work was never lost and nothing was redone. What happened is that its
idle notification arrived roughly two minutes after dispatch, while it was still working; I checked
for the report file inside that window, found nothing, and applied the standing rule "re-send if a
notification arrives with no report". The rule is still right, but it needs a guard: an idle
notification is not proof the agent has stopped. Check the file twice with a gap, or ask the agent,
before re-sending a whole job. Cost here was one duplicated report and nothing else.

Docs commit 9495c974, refined by 3a405228 after the second copy supplied a detail the first lacked:
a stale FrameId taken DEEPER than the current top truncates to a larger index and is a silent
no-op, so both misuse directions are sound rather than just the one I had written down.

## Task 9: DONE, commit 43e18462. Review dispatched.

Verified rather than accepted: commit touches exactly the three permitted files, working tree
clean, 699 tests pass (683 before), clippy -D warnings rc 0, fmt rc 0. Both instruction loops in
run.rs still route through step_in_temps_frame (run.rs:236 and 496), which was the constraint I
sent mid-flight.

**Corpus moved 3/26 to 9/26**, against my predicted "about 10". Every NUMERIC, stem and EXIT row
closed. The remaining 17 are DO (10), TRACE (4), SELECT (2) and IF (1), and every one still fails
as a clean loud failure rather than diverging.

Concern 3 handled here: stem.rs's blanket dead_code allow removed, commit d8aa4bc5. That doubles as
a check rather than a tidy-up, since the allow sat on the whole impl block precisely because every
function formed one unreachable component. Clippy passing with it gone proves each of stem_assign,
stem_set, stem_drop, stem_drop_tail, tail_key and stem_get now has a live caller; a partial wiring
would have failed there.

Concern 1, and it corrects something I passed on: the EXIT numeric-conversion "measured fact" in my
dispatch was an artifact. The earlier test values were exact multiples of 256, which makes
"converted, low byte 0" and "rejected, defaulted to 0" indistinguishable at the rc level, so an
apparent large rejected range in positive i32 space was never there. The real asymmetry is that
EXIT's unary minus is arithmetic and rounds to the active NUMERIC DIGITS at creation, so
`exit -2147483647` rounds before any range check runs, and it needs no special-casing because D15
already says precision is fixed at creation. I have asked the reviewer to check this against the
oracle with values that are NOT multiples of 256 rather than admire the story.

**Concern 2, operational, and the important one.** Mid-task the implementer ran
`git checkout -- lib.rs` to undo a temporary test mutation it had already hand-reverted, and so
discarded every real edit the task had made to that file. It caught this from git status and
reapplied everything, rebuilt and retested, and disclosed it unprompted. No committed history was
touched. Two things follow. First, `git checkout -- <path>` belongs in the same never-do class as
`git reset --hard`: it is silent, unrecoverable for uncommitted work, and its blast radius is the
whole file rather than the change being undone. Put it in future dispatches by name. Second, this
is exactly the shape where something is dropped and the suite stays green because the dropped thing
had no test, so the Task 9 review leads with "verify the move is complete and faithful against
`git show 3a405228:...lib.rs`", not with the new code.

Monitoring lesson, mine: I read this agent as dead. Its report file sat at a 5-line stub for 40
minutes, the tree was clean, and it did not answer a status ping. I was three minutes from
re-dispatching Task 9, which would have put two agents in run.rs. It was in fact composing a 52 KB
file, which lands all at once. A quiet report file plus a quiet tree is not evidence of death when
the deliverable is one large file, and my own instruction to append to the report as you go is what
would have made the signal reliable, so enforce that rather than inferring liveness from mtime.

Checked t9-run's claim that `step` has exactly one caller. It is right about the code that ships
and wrong as stated. Non-test callers: exactly one, `step_in_temps_frame` at run.rs:453, and the
wrapper still pops unconditionally (`let flow = self.step(..); self.roots.pop_frame(frame); flow`).
The chokepoint holds. But there is a second real call at **run.rs:806, inside `#[cfg(test)]`**: the
helper `run_source`, which loops over instructions calling `interp.step` directly.

That is a new instance of the exact shape the temps-frame investigation named as the one unhealed
case, and it is worse than eval.rs's helpers in one respect: its own doc comment calls it "a
miniature `run_activation`", while omitting the single thing that makes the real one balance. A
later reader extending or copying it inherits a silent leak, and nothing asserts temps balance.
Benign today for the reasons already established (test-only, alloc_with never collects).

**Deliberately NOT sent to the reviewer**, which has the chokepoint as its explicit item 2. Passing
it my finding would collapse the second lens into an echo of the first, and whether the review
finds this independently is itself information about how much the next review's silence is worth.
Also not edited yet: run.rs is under review at 43e18462, and touching it now would make that range
stale, which has produced a "finding true when written and false when read" on this project before.
Fix after the review lands, either as a review finding or as my own follow-up.

## Task 9: REVIEWED. Code quality PASS, spec compliance ONE CRITICAL. Fix round 1 dispatched to t9-run.

Move fidelity came back clean and was checked mechanically rather than by eye, which is the right
answer to the checkout incident: every nonblank line of 3a405228's lib.rs was tested for a verbatim
match in the union of the new lib.rs and run.rs, and exactly 27 lines miss, all inside the six
stated edits. One retyped word in the borrow-shape comment, either doctest, or my failure_site
block would have surfaced. The incident left no scar.

The EXIT rounding story was confirmed adversarially, not admired: 14 fresh probes, none a multiple
of 256, including the bisection the story predicts (-2147483641 -> 8, in range at DIGITS 9, versus
-2147483645 and -2147483647 -> 0, rounding up past INT32_MIN) and the decisive
current-DIGITS-independence pair (digits 3 with exit 2147483647 -> 255 and exit 12345 -> 57, where
a conversion using the active DIGITS would give 0 for both). The "large rejected positive range" I
passed along in the dispatch was definitively a mod-256 artifact. Two lessons: a probe set whose
values share a factor with the modulus cannot distinguish conversion from rejection, and a measured
fact in a source comment is still only as good as the values behind it.

CRITICAL, and it is silent in half its rows: DROP (v) reads the wrapper's value as one verbatim
name, where the oracle reads a blank-separated subsidiary list of validated symbols. Six
divergences, which I re-measured myself before dispatching the fix. Three are wrong values with no
error at all ('a b' drops nothing, ' x ' is not trimmed, 'a. x' tombstones key " X" instead of
dropping the stem and X); three are missing errors (31.2, 31.3, and 20.928 for '(w)', which shows
the list is not recursive). Every existing test passes because their indirect values are all
single-word, valid and unpadded, which is exactly where the two readings coincide.

Carried into the fix dispatch, because the reviewer nearly lost time to it: with UNSET targets the
two readings agree by coincidence, since a cleared slot named "A B" and two dropped variables both
render as derived names. A test written the obvious way passes against the bug.

Minor 1 is the one to watch beyond this task: EXIT's result ObjRef is unrooted from the wrapper's
pop all the way to exit_code_for. That is UNDER-rooting, the direction that actually breaks when a
collector lands, and longer than the windows the crate already documents. Minor 2, shape_of accepts
names no scanner would produce, is what let the invalid indirect names through, so it closes with
the Critical.

Mutation testing 4/4 this time, each killed by exactly the right test, so the falsifiability answer
on this task is finally yes.

**The reviewer did NOT find run_source**, my held-back finding, and its not-reached list does not
mention test helpers. So the withhold was worth doing: it is a genuine independent addition rather
than a duplicate, and it calibrates what a clean review's silence is worth here. Sent to t9-run
with the fix round now that the review is closed and there is no independence left to protect.

## Task 9 fix round 1: commit 18c0bd17. Re-review dispatched.

Critical closed. DROP (v) now splits the wrapper's value into a subsidiary list, validates every
word before dropping any of it, and reuses one per-name drop path. I re-verified the review's six
rows plus a seventh of my own (a bad word mid-list) byte-identical against the oracle. 79 unit + 11
integration + 2 doctests green, clippy and fmt clean, corpus subset still 9/26 since DROP (v) is
not in it.

Two implementer findings that sharpen the fix beyond what the review asked for: a literal newline
does NOT separate list items and instead fails as an invalid character, so the predicate is
space-or-tab rather than is_ascii_whitespace; and the character-set check running first is what
makes both '(w)' and 'a-b' fail as 20.928, so no special recursion guard is needed. Both are single
measurements carrying a generalisation, and I have asked the re-review to re-measure them.

RULED, on Minor 1, since the implementer asked rather than assumed: the deferral stands. Its own
comment is what convinced me, by showing why the ordinary remedy does not apply -- EXIT's result
must outlive the clause that produced it, so push_temp is the wrong tool, and the real fix needs a
root that survives a frame pop, a mechanism with no other user today. Building it now is
scaffolding for a caller that does not exist, which is the same call activation.rs made about its
body selector.

But a deferral is only safe if it is found again, so it did not stay a bare comment. c67dd343 puts
the pointer on Heap::collect in rexx-core: a comment at the leak site is read by someone already
working on EXIT, while the person who turns that window into a use-after-free is whoever wires a
collector into the interpreter, and that is the function they will have open. It also tells them to
sweep rexx-exec for the same shape first, since an unrooted window is invisible to the compiler and
surfaces as a wrong value rather than a crash.

Task 10 is held until the re-review is clean. It owns run.rs, and so would any second fix round.

## Task 9: CLOSED. Fix round re-reviewed CLEAN.

The re-review confirmed the Critical is fixed faithfully, the drop_by_name refactor broke no
previously-working direct DROP, both my rulings were right, and the empty/blank test is not a
witness that cannot fail (removing the empty-word filter kills it, and the filter is load-bearing
against a panic on word[0]).

One real gap found and closed by me, commit 63a4092e: a mutant using is_ascii_whitespace as the
separator passed all 79 tests. The code was right and nothing in the repository said so, because no
test carried a newline and the distinction rested on an end-to-end diff nobody re-runs. One
assertion, measured byte for byte including the raw newline inside the substitution, and confirmed
to kill the mutant.

Worth recording, because it is the honest version of a survivor: a stem-arm mutant rerouting
stem_drop to a slot clear SURVIVES all tests, and that is an EQUIVALENT mutant rather than coverage
rot. stem_drop is replace_stem(name, None), and a freshly rebound stem is observationally identical
to a cleared slot until something can hold a second reference to the old stem object, which nothing
in 4a can. It becomes pinnable exactly when stem aliasing arrives in 4b. That is the distinction
worth insisting on: an equivalent mutant is replaced or explained, never excused, and this one is
explained by a property of the phase rather than by a gap in the tests.

Second time this session I have written a commit hash into this ledger before the commit existed,
and both were wrong (9ceb2a3 for 3a405228, 4d24bc4 for 63a4092e). Once is a slip, twice is a habit,
so the rule is now explicit: never write a hash into the ledger in the same command that creates the
commit. Commit first, read the hash back from git, then record it. A wrong hash here is worse than
no hash, because this file is the recovery map a later session trusts without re-deriving, and it is
the same citation-drift class this project has already measured seven times in prose.

## 2026-07-31 17:22, after the 5h limit reset

t10-select died on the session limit having written only its stub, so nothing was recoverable and
nothing was lost. Re-dispatched as t10b-select from the same corrected brief, told explicitly that
its predecessor did no work so it does not go looking for phantom progress.

Also dispatched: pre-flight audit of Tasks 13 and 16 to t8-review, plus one scoping question I need
answered before Task 16 can go out at all. Task 16's harnesses are written as gate-day instruments
run once at the end, but the ad-hoc corpus loop I have been running by hand after every task has
been the most useful progress signal in this phase (3/26 to 9/26 when Task 9 landed, and it names
which task owns each remaining failure). Asked whether the harness can be built NOW so it becomes
that repeatable instrument, what part of Task 16 that is, and what should still wait.

Two dependencies flagged into that audit that neither brief can know: Task 16's exclusions file
already exists and is populated, so its "Create" line is stale and I have added rows to it this
session; and Task 13 shares the clause-echo indentation rule with Task 10, since the error report
reuses TRACE's own two-spaces-per-nesting-level indentation. Task 13's brief predates that
discovery, and Task 10 is characterising the per-construct counting right now.

Queue as it stands: Task 10 in flight, then Task 11 (which owns run.rs and eval.rs and error.rs,
per the corrected Files section), then 13, then 15b and 16. Task 8, 9 and 12 are closed; 14a, 14b,
14c closed earlier.

## Pre-flight on Tasks 13 and 16, and D17's failure. Plan and spec corrected at e0e57825.

**D17 was not implemented, and Task 13 is now the retrofit D17 existed to prevent.** Measured:
zero trace emission points in eval.rs or run.rs, and the only trace-sink writer is execute's error
path. Tasks 7, 8 and 9 each built part of the dispatch loop with no event hook, because D17 lives
in the design spec and none of their task bodies carried it. This is my failure and it is the
fourth instance of one mechanism: a fact outside a task's own section does not reach its
implementer. Recorded in Task 13's body, where it will be paid for, rather than as a general note.
Consequence is scope, not correctness: the hook threads through roughly eighteen eval_node arms
plus run.rs, so Task 13 now sequences after Task 11 and reads committed code rather than the plan.

Task 16's Step 4 said to write phase-4-exclusions.txt. **Following it literally would have
REGRESSED the file**, dropping a deviation and the entire KNOWN GAPS section, both added during
the phase. What 16 still owes it is the set assertion in the harness and a phrasing sync. Step 5
named phase-4a-gate.md as the file to assess; it does not exist and is that step's output.

The 342 trace-line figure reproduced exactly, and is now annotated against the NEXT recount rather
than corrected. The file carries six uppercase ::RESOURCE blocks a case-sensitive scan misses, with
32 more expected lines whose prefixes are the >I>/<I< invocation pair; a case-insensitive scan
gives 37 blocks and 374 lines. Both are right answers to different questions. Two earlier recounts
already went astray on anchoring (239 and 393), so the note asks a recounter to state which scan
they used rather than which number they got. This is the third time this one figure has been
re-derived, and the first time the derivation itself was written down.

SCOPING RULED: the continuous instrument gets built now, and it is Task 14's tests/corpus.rs rather
than Task 16's harnesses. A differential runner has no dependency on phase completeness, since both
interpreters are fully defined today and 17 of 26 failing is as honest a measurement as 0 of 26.
The only genuinely gate-day part is the strict switch, so it runs in report mode with a flag the
gate flips. Dispatched to t14-runner, with the requirement that report mode state loudly it is not
the gate: a green cargo test line that actually means "17 of 26 disagree" would be exactly the
silent-vacuous-harness failure this project keeps finding in its own instruments, and the worst
place to introduce it is the instrument that measures the others.

Also ruled, so it is not re-litigated at review: tests/corpus.rs, tests/coverage.rs and
tests/loud.rs are integration tests and that is correct. The constraint forbids integration tests
for PRIVATE subjects; these three take rexx_exec's public cross-crate surface, and tests/spike.rs
is the standing precedent.

## 2026-07-31 21:22. Three agents lost to the WEEKLY limit, then dispatch recovered.

t10b-select, t14-runner and t8-review all died within two minutes of each other on the weekly
limit (reported as resetting Aug 3). Recovered on a later reset. What survived on disk:

* task-13-16-preflight.md, 241 lines, STATUS: DONE. Fully consumed, findings already acted on.
* task-10-report.md, 20 lines, IN PROGRESS: a reading log and a to-do list, no measurements. One
  useful confirmation, which I verified independently: step_in_temps_frame is the ONLY caller of
  step in the crate now, test helpers included, so the fix round's run_source change holds.
* controller-edit-audit.md, 19 lines, IN PROGRESS: its commission restated, no findings established.
  **The audit of my own plan and spec corrections is therefore still OPEN**, and it is the one piece
  of outstanding work with no owner. Re-dispatch it: four rounds of my edits to Tasks 10, 11, 13, 16
  and the design spec have never been checked by anyone else, and on this plan a correction landing
  in one place while the contradiction survives in another has happened three separate times.
* task-14-runner-report.md: absent. Nothing done.

The write-the-report-first rule keeps paying, but unevenly: it preserved a finished audit and two
skeletons. What the skeletons prove is that "append as you go" has to mean **append a measurement
the moment you take it**, not "append when a section is finished" -- both partials had a plan and no
evidence, which is the least useful thing to preserve.

Re-dispatched, both with the accumulated knowledge rather than the original briefs:
* t10c-select, Task 10, carrying the four jump targets I read out of ast.rs myself (If.false_target,
  Else.then_exit, When.false_target/exit, WhenCase.values) so it does not rediscover them. Getting
  those four wrong is the whole risk of the task, and Phase 3 structurally cannot see a wrong jump
  target.
* t14b-runner, the corpus runner, warned that IF and SELECT are being implemented under it and its
  9-of-26 baseline may legitimately move mid-flight.

## Task 14's corpus runner: commit 19e9e286, one defect sent back.

402 lines, tests/corpus.rs only, run.rs untouched. Reproduces 9 of 26 exactly, partitioned DO 10 /
TRACE 4 / SELECT 2 / IF 1, and the implementer confirmed it with an independent shell loop before
writing the test. STRICT mode via REXX_CORPUS_GATE=1 fails with "17 of 26 disagree", verified.

**The defect is the exact one the requirement existed to prevent, and it arrived through the harness
rather than through the wording.** In report mode a plain `cargo test --test corpus` prints
`test corpus_differential ... ok` and nothing else: libtest captures a passing test's stdout and
shows it only on failure, so the "NOT THE GATE" banners, the count and the owner-grouped mismatch
list are all swallowed unless someone passes `-- --nocapture`. The operator-visible result of a
17-of-26 disagreement was therefore a green ok. Found by running it rather than by reading the diff,
which is the whole argument for verifying a subagent's claims: every sentence in its report was
true.

Sent back with the two constraints stated rather than a dictated fix, because they pull against each
other: the tree must stay green under a plain workspace test run, since every task this phase has
been verified that way, and a disagreement must be visible without special flags. Options given: a
child process writing to the inherited stderr fd, which libtest's capture does not touch since it
replaces Rust's own sinks and not the process descriptors; or splitting the roles so the gate is the
test and the report is a deliberately-run binary whose output is never captured. Ruled out
explicitly: documenting --nocapture, because a visibility mechanism that depends on the reader
already knowing to ask is the same defect as the silent pass. Required a demonstration that the
number reaches the terminal, not an argument that it should.

Two design choices of its own worth keeping. The memory limit is imposed as
`sh -c 'ulimit -v ... && exec "$0" "$@"'` rather than a pre_exec/setrlimit closure, because the
workspace forbids unsafe_code at the lint level and pre_exec would not compile without weakening
that -- and it verified the limit bites, with a 2 GB allocation raising MemoryError under the
wrapper. And the oracle's absence FAILS rather than skipping, verified against a nonexistent path,
so a machine without the build cannot report a vacuous 0 of 0 and go green.

Process: the implementer ran `git checkout -- <path>` mid-verification and disclosed it unprompted.
Harmless, since the file was untracked and git refused. **That is the second implementer to reach
for that command despite an explicit prohibition in its dispatch**, which says the prohibition is
fighting a strong habit rather than filling a knowledge gap. Keep it in dispatches, but do not
expect it to hold on its own; what actually saved both cases was the agent noticing and saying so.

Corpus runner fix: commit 3363b278. **Task 14's runner is DONE and the visibility defect is closed.**

Verified by me the same way I found the defect, by running it rather than reading it: a plain
`cargo test --workspace` with no flags now prints the banner, the 9-of-26 count and the full
owner-grouped mismatch list inline. 705 tests pass, zero failing suites.

The chosen mechanism is worth recording because it is not obvious. The report is built into a String
with no println! anywhere and piped through `sh -c 'cat >&2'` with stderr inherited. A child's
inherited descriptor is dup'd from the parent's real fd 2 at spawn time, which is upstream of
libtest's capture, since libtest replaces Rust's own thread-local sinks and not the process
descriptors. No unsafe, which matters because the workspace forbids unsafe_code at the lint level and
both a raw-fd Stdio constructor and dup2 would have needed it.

Better than the fix: it added a permanent regression test that re-executes the test binary against
itself, with no --nocapture, and asserts a probe's marker reaches the child's captured output. So the
property is checked on every run instead of resting on an experiment somebody remembers doing. That
is the right response to "demonstrate it rather than argue it" -- turn the demonstration into
something that keeps demonstrating.

One more thing the implementer disclosed unprompted: its first draft of the correction section
claimed it had run the forbidden `git checkout --` a second time this round. It had not. It caught
the fabrication before sending and replaced it with what actually happened. A subagent inventing an
incident in its own report is a failure mode I had not seen here, and it points the same way as the
citation-drift class: the details most likely to go wrong are the ones nobody re-checks, including a
confession, which a reader would never think to verify.

## Task 10 stopped to ask, and found a structural gap Phase 3 left. Resolution approved.

**Phase 3 elided the branch-end markers, and nothing noticed until an executor needed them.** The
C++ closes a THEN branch with a synthetic marker carrying the jump, while `Else` itself only traces.
Phase 3's flat list has no instruction meaning "the branch ended", so:

* `if c then A else B`: `If.false_target` is the `Else`'s OWN index, verified by me at block.rs:432
  where `set_false_target(parent, self.next_index())` fires as the THEN branch completes. The true
  path falls through A onto `Else`, and the false path jumps to `Else`. Same index, opposite required
  behaviour, and `step` receives nothing distinguishing them.
* `WHEN` is worse: there is no marker after a `WHEN`'s branch at all, so falling off the end of a true
  branch lands on the next `WHEN` when it must go to `exit`. corpus/lang/select_when_bodies.rex exists
  to catch exactly this and would have caught it.

This is the third defect class Phase 3's gate structurally could not see, and the ledger already
named it: "a control-flow target wired to the wrong index" is invisible to node counts. It took an
executor to surface it, which is what the phase ordering predicted.

I looked for a flat, stateless fix and it only works for IF: jump to false_target + 1 when the
instruction there is an `Else`, and let `Else`'s arm jump to then_exit, disambiguating via the marker
that does exist. No analogue for WHEN, so it buys a special case rather than a design. Rejected.

APPROVED the implementer's resolution: each of `If` and `Select` resolves its whole construct inside
its own `step` arm, running a bounded sub-loop over the winning branch (the pattern run_fragment
already uses in that file) and returning one Goto past everything else. Three constraints attached:

1. The sub-loop calls step_in_temps_frame PER INSTRUCTION, not once around a branch, or the frame
   healing silently coarsens from one clause to one branch.
2. **An unrecognised Flow must propagate outward rather than be swallowed or caught by a catch-all.**
   This is the 10/11 collision: `leave sel` where sel labels a SELECT exits that SELECT, so Task 11's
   LEAVE has to unwind out of Task 10's nested Rust call. Flow::Exit is the case testable today and
   is the template. Required the report to state what the sub-loop does with a Flow it does not own,
   because that sentence is what Task 11 builds against.
3. Nesting step inside step adds Rust stack depth per source nesting level. Bounded by text, not
   data, so not D19's unbounded case, but INTERPRETER_STACK_BYTES's doc names its three consumers and
   must now name a fourth. Told it to state an honest gap rather than guess a number: that figure has
   moved four times this phase and a guessed fifth is worse than none.

Ruled on the file-boundary question: bump eval.rs's `logical_value` to pub(crate) and reuse it rather
than duplicating a four-line 0/1 match. Permitted set widened to that one line.

Also warned it of something its writeup missed: the absorbed-WHEN case means a `When` instruction
executes that its enclosing `Select`'s `whens` list does not contain, so that path needs to be
defined and checked at rc 0 rather than assumed.

## Task 10: committed addf89b1. Corpus 9 to 12 of 26. One defect sent back.

Verified: 723 tests green workspace-wide, clippy and fmt clean, exactly two files (run.rs plus the
approved one-line eval.rs visibility bump). Absorbed-WHEN and both SELECT CASE forms diff
byte-identical against the oracle.

DEFECT, found by running the differential myself rather than reading the diff: **a WHEN condition
failure is credited to the SELECT clause.** `select / when 'x' then nop / end` reports line 1 clause
`select` where the oracle reports line 2 clause `when 'x' `. Wrong clause AND wrong line, so not the
indentation gap.

The cause is the approved design working as designed, which is worth recording as a cost of that
choice rather than as a surprise: dispatch lives in the Select arm reading When nodes as data, so a
WHEN's condition is evaluated inside the Select's own step call and never inside a
step_in_temps_frame for the When instruction. The implementer's own fix moved site resolution into
the wrapper with the innermost winning, which is right, but there is no inner wrapper call for a
failing WHEN condition, so the innermost remains the Select. Its report described that fix as closing
the misattribution; it closed it for instructions inside a branch and not for the condition of a
WHEN. Two different paths, one covered.

IF is already correct, and the contrast is the explanation: `if 'x' then nop` blames the `if`, which
is right because the IF is the failing clause, whereas for a WHEN the failing clause is the WHEN, a
distinct instruction with its own clause_span and line.

Asked it to check three neighbours I have not measured, since they are the same question: a raising
SELECT CASE expression (probably the select's own, but confirm), a raising WhenCase value, and raises
in an OTHERWISE and in the second of two WHENs so the expected line has to MOVE. A test whose
expected line is 1 cannot distinguish a correct line from a defaulted one.

RULED on both open items. Indentation stays out of Task 10 and **Task 11 owns it**: Task 11 already
has error.rs in its permitted set for the 11.1 catalogue test, and DO is what makes block nesting
common rather than incidental. The rule is now measured on both sides rather than half-inferred: two
spaces per open block frame, an IF's THEN counts as two frames (`if 1=1 then say 2 & 1` indents four),
a SELECT adds one more (`when 1=1 then say` indents six). And yes to the fourth
INTERPRETER_STACK_BYTES bullet, written as an honest unmeasured gap: step now recurses through
run_bounded once per source nesting level, bounded by program text rather than data, so it is not
D19's unbounded case. A fifth guessed number would be worse than none.

## Task 10 fix round: commit 87ed65b6. Review dispatched.

Re-verified with the probes that found the defect. Clause and line are now right in all five cases:
a raising WHEN condition blames the WHEN at its own line, a raising SECOND WHEN moves the line to 3
rather than defaulting, a raising SELECT CASE expression blames the SELECT and matches byte for byte,
a raising WhenCase value blames the WHEN, and a raise in an OTHERWISE branch blames its own clause.
**Every residual difference is now purely the indentation**, which is Task 11's. 728 tests green,
clippy and fmt clean, corpus still 12 of 26 because no corpus program yet raises inside a WHEN's own
condition.

The implementer added five unit tests asserting interp.failure_site directly, line AND text, on the
grounds that nothing checked it before and that is what let the round-1 defect through 97 passing
tests. That is the right lesson drawn from its own defect rather than from my message.

INFRASTRUCTURE NOTE for whoever picks this up: the superpowers plugin cache is gone from this
machine, so scripts/task-brief, scripts/review-package and scripts/sdd-workspace no longer exist. All
briefs for the remaining tasks were generated before it vanished and are on disk. Review packages now
have to be built by hand, which the skill documents as the fallback: git log --oneline, git diff
--stat, and git diff -U10 over the range, all redirected into one file. Task 10's was built that way
at review-3363b278..87ed65b6.diff, 1416 lines.

Dispatched Task 10's review to fable, read-only, with three priorities. First and most consequential:
whether run_bounded propagates a Flow it does not own outward and unchanged, because Task 11's LEAVE
must unwind out of Task 10's nested Rust call and a catch-all swallowing an unknown variant would cost
Task 11 a rewrite. Second, whether the new tests can fail, given this task's own round-1 defect
survived 97 of them. Third, to check the implementer's written indentation rule against the oracle
rather than reporting the unimplemented indentation as a defect, because Task 11 implements from that
writeup without re-deriving it and it is marked as partly measured and partly inferred.

## Task 10 REVIEWED: spec PASS, quality PASS, 0 Critical, 2 Important. Fix round 2 dispatched.

Everything I was most worried about holds in the code as written rather than as described.
run_bounded's catch-all is the forwarding kind, `other => return Ok(other)`, and both If/Select
callers forward untouched, so **Task 11's LEAVE hinge is safe**. Temps granularity is one frame per
clause with step still at exactly one non-test caller. record_failure_site fires only on Err paths,
first-call-wins, and the entry point takes it. Five mutations killed, including the naive If
fallthrough, a SELECT exit landing on the next WHEN (8 failures, including the multi-instruction-body
test the brief demanded), and run_bounded swallowing an unowned Flow.

The indentation characterisation was verified against the oracle including both rows the report marked
inferred: SELECT CASE's THEN is six extra spaces and an ELSE IF chain is eight, both as the written
rule predicts. **Task 11 can implement from that writeup without re-deriving it**, which was the whole
point of asking for it in an implementable form.

Important 1 is the same defect class twice: passing None instead of `source` into run_bounded survives
all 102 tests, so attribution for a raise inside a branch BODY is correct but undefended. The five new
attribution tests cover conditions, values, the case expression and OTHERWISE, but OTHERWISE runs in
the OUTER loop, so none of them exercises the source threading that heals a THEN body. Two tests fix
it, and I asked for the mutation to be re-run rather than assumed. Habit worth generalising, since
this is the second time on one task that correct behaviour had no test able to defend it: for each
fix, name the mutation the new test kills.

Important 2, verified by me at run.rs:65: Flow::Goto's doc still says a fragment can never jump and
that run_fragment's unreachable! on this variant "still holds". This diff deleted that unreachable!
and made a fragment able to jump through a nested IF or SELECT, and run_fragment's own new comment now
says the opposite. Two comments in one file contradicting each other is worse than one being merely
stale, and Task 11 reads Flow first, so it is the first thing Task 11 will read and disbelieve.

Also asked for: the structuring semicolons in the newly added comments only. The reviewer is right
that it continues about eighteen lines of pre-existing drift and is not a regression, but the
constraint is explicit and those are the lines under review; sweeping the pre-existing ones would
obscure the diff.

NOT fixed, recorded for 4b: failure_site is never cleared mid-run. It matters only once a condition
trap can resume execution after a raise, so clearing it now would be scaffolding for a caller that
does not exist.

## Task 10 fix round 2: commit a9997e55. Scoped re-review dispatched.

Verified: 730 tests green, clippy and fmt clean, run.rs the only file, tree clean, corpus still 12 of
26 since no corpus program yet raises inside a branch body.

Both Importants closed. The M7 mutant now dies, and the implementer confirmed the kill rather than
assuming it: reverted source to None at both run_bounded call sites, watched exactly the two new tests
go red with all 102 others staying green, restored. That precision is worth more than the fix itself,
because "only these two went red" establishes the tests are aimed at the right mechanism rather than
merely being sensitive to something. Flow::Goto's comment now separates the two cases correctly: a
label jump inside a fragment is still 47.1, while IF/SELECT can jump inside one with no label
involved.

Six structuring semicolons rewritten, two deliberately left because they sit inside verbatim
quotations of the oracle's own catalogue text, on the reasoning that a quoted message's punctuation is
not ours to restructure. Asked the re-review to confirm they really are quotations rather than
paraphrases that resemble them, and to check that rewriting prose to satisfy a punctuation constraint
has not introduced a false statement, which is a cheap way to do exactly that in a file that has
already produced one comment contradicting another.

Task 11 stays blocked until this closes: it owns run.rs, and so would a third fix round. When it goes
out it carries three things this task produced -- the verified indentation rule in an implementable
form, the guarantee that run_bounded forwards an unowned Flow so LEAVE can unwind out of a nested
call, and error.rs in its permitted set for both the 11.1 catalogue test and the indentation work.

## Task 10 CLOSED. Re-review clean on behaviour, three prose Minors fixed by me at 6ff4e93a.

The re-review reproduced the M7 kill at exactly the claimed precision, which is the part worth
recording: scratch copy synced to a9997e55, baseline 104 green, source->None at If's (run.rs:502) and
Select's (run.rs:609) call sites gives exactly 2 failed and 102 passed, and the two failures are the
two new body-attribution tests. "Only these two went red" reproduces. Both tests assert line and text
with expected lines 3 and 2, neither 1, so a defaulted line cannot pass them. The two surviving
semicolons are genuine verbatim quotations of rexxmsg.xml:2467 and :267, byte-matching modulo <q>
markup.

**A comment-only round broke prose three times**, which is the document-edit base rate on this branch
holding precisely, and the mechanism deserves its name: rewriting a sentence to satisfy a punctuation
rule is a cheap way to introduce a false statement. The best example is self-referential -- the
semicolon cleanup introduced a NEW structuring semicolon into the very comment it was rewriting, and
missed one Task-10-authored survivor. It also left a cross-reference pointing at run_bounded, whose
doc does not mention fragments, when the argument lives in run_fragment's. And the WHEN-body test's
doc said "both of Select's own run_bounded call sites" where Select has one and the two are If's and
Select's, plus a claim about OTHERWISE that is true of the production outer loop and false in the test
harness.

Fixed all three myself rather than dispatching a fourth round, because each was one sentence and Task
11 owns run.rs next and reads Flow first. Verified the cross-reference target actually carries the
argument rather than taking the finding on trust. Pre-existing drift still untouched, and worth noting
for whoever eventually sweeps it: most remaining semicolon matches in that file are Rexx clause
separators inside doc examples, where the semicolon is the language's rather than English prose.

**Task 10 is done. Next: Task 11**, which now inherits three things this task produced: the
oracle-verified indentation rule in implementable form, the guarantee that run_bounded forwards an
unowned Flow so LEAVE can unwind out of a nested Rust call, and error.rs in its permitted set for both
the 11.1 catalogue test and the indentation work. Its Files section was corrected earlier to add
eval.rs for the depth counter, which eval.rs:70-71 already claims.

## Correction from Moritz: the structuring-semicolon rule was never this project's.

It belongs to a different repository's conventions and I imported it into Global Constraints by
mistake, then enforced it through several task reviews. Withdrawn at 610642a3, as a note rather than a
deletion, because committed review reports cite it and a reader meeting one needs to know it was
withdrawn rather than forgotten. Em-dashes stay as a consistency preference only, since the tree uses
`--` everywhere.

**Enforcing it was not merely wasted effort, it was harmful, and that is the part to carry forward.**
The comment-only round whose entire purpose was rewriting semicolons into separate sentences introduced
three false statements, including a new structuring semicolon inside the very comment being rewritten
and a cross-reference pointing at a function whose doc does not mention the subject. Every edit to
prose is a chance to make prose wrong, so a style rule that mandates edits for their own sake carries a
defect rate and no offsetting benefit. Saved to memory as comment-rule-scope.

I am NOT reverting 6ff4e93a. Its semicolon rewrites were unnecessary, but the same commit corrected
three genuinely false claims, and the prose it left is accurate.

## Adversarial audit of my own edits: two agents, deliberately non-overlapping lenses.

The open item with no owner now has two. Split so their silence means something independently:

* audit-claims (fable): factual accuracy, one claim at a time. Told never to verify a citation by
  reading the line it names, since six of seven wrong citations on this branch named the right function
  and a line inside a different one with correct prose; to re-derive every number; to re-measure every
  oracle claim rather than reasoning about it; and specifically to check that where I attribute a fact
  to another agent I transcribed it faithfully, which is the risk unique to my role. A number right
  when measured and wrong when copied looks exactly like a correct claim.
* audit-coherence (opus): contradictions scoped to the section rather than the document, whether the
  documents are safe to build from as a hostile implementer would read them, the plan's per-task
  extraction seam (anything outside a task's own section is invisible to it, a mistake I have made four
  times), and four named rulings it is asked to overturn.

The rulings I put up for challenge, rather than the ones I am confident about: deferring the EXIT
under-rooting window when gate criterion 4 requires the corpus to pass under collect-on-every-
allocation and nothing has ever called collect from the interpreter, which may mean I deferred the
thing most likely to fail my own gate; accepting the nested-bounded-loop design instead of repairing
Phase 3's elided end-of-branch markers; assigning indentation to Task 11; and declaring D17 unmet,
which is a process-failure verdict resting on my reading of the spec alone.

Task 11 waits for both audits, per Moritz. It is about to be built directly from a task body I edited,
which is exactly what an audit of my edits is for.

## Both adversarial audits in. Fixes at 9eae6c2d (coherence) and the claims round.

audit-claims found 4 real errors and 1 misleading wording, all mine, and everything else it checked
reproduced exactly -- including all fourteen oracle-attributed claims I had flagged as most suspect.
That is the useful shape of the result: the measurements were sound and my transcription of them was
where the errors were.

F5, the worst because Task 11 is built from it: "leave sel, where sel labels a SELECT, exits the
SELECT" reads as the ordinary-label form, which is 28.3. Only SELECT LABEL sel is leavable.
F1: I blamed a measurement gap for the seven unpinned mapping rows. Task 8's report carries the very
'9' <<= '9' that would have exposed the <<= mutation, plus '9' \== '9'. Both measured, neither reached
a test, so the loss was between report and assertions.
F2: the spec's recount annotation named a scan giving 40 blocks and 437 lines while claiming 37 and
374. Reaching 37/374 needs a further filtering step the sentence attributed to the scan -- the same
shape of error the annotation exists to prevent.
F3: two lib.rs comments dated the stale Loud enumeration to Task 6; after Task 6 it was still true.
F4, unfixable, it is in commit 56e60d96's message: error 42.11 does not exist. Recorded here instead.

**Three of the four were transcriptions of other agents' correct work, which is this role's
characteristic failure and is invisible by construction.** A figure right when measured and wrong when
copied reads exactly like a correct claim. The countermeasure that worked was an auditor told never to
verify a citation by reading the line it names and to re-derive every number, so it ran the scan
instead of reading the sentence about the scan.

Worth keeping from the coherence audit as a template: tests/corpus.rs:451 records "9 of 26 at commit
e0e57825" -- it names the commit its figure belongs to and forbids adjusting the comment to match a
later number. That is the only duplicated figure in this phase that defends itself, and it is the
pattern every other one should follow.

Both audits' not-reached lists are on disk. Neither ran mutations (read-only), audit-coherence ran no
probes at all by design, and Tasks 1-8 and 12's bodies were never audited.

Task 11 is now unblocked and goes out next.

## Task 11 dispatched. BASE d7b84dc5.

Brief regenerated by hand from the corrected plan section, since the plugin cache holding
scripts/task-brief is gone from this machine: `awk '/^### Task 11:/{f=1} /^### Task 12:/{f=0} f'` over
the plan. 103 lines, and it now carries the four things the audits found missing, so the stale
pre-audit brief is fully replaced rather than patched.

Permitted files run.rs, eval.rs and error.rs, matching the corrected Files section: eval.rs because
eval.rs:70-71 already claims the depth check belongs to eval, error.rs for both the 11.1 catalogue entry
and Raised::report's indentation.

Four things singled out in the dispatch as the ones that decide whether the task works, all of them
findings from this session rather than from the original plan: run_bounded absorbing an in-range Goto
without telling the arm that produced it, which is criterion 6's own mutation by an undocumented route;
the depth test needing run_program's sized thread and a left-deep operator chain rather than
parentheses, since MAX_EXPR_DEPTH raises the same 11.1 from the parser; that leave sel needs SELECT
LABEL sel and an ordinary clause label is 28.3, which was wrong in the brief until the claims audit
caught it; and the indentation, with the warning that its trace half is asserted and never measured.

Also carried: the corpus is the progress instrument, 12 of 26 now, 10 of the remaining 14 blocked on DO
and 4 on TRACE, so this task should reach about 22. And the habit Task 10 established after shipping
correct behaviour twice with no test able to defend it: for each feature, name the mutation the new
test kills.

Told explicitly that there is no structuring-semicolon prohibition here, so the dispatch does not
re-import the rule that caused three false statements when it was enforced.

## Task 11 stopped to ask, twice usefully. Permitted set widened to include tests/spike.rs.

The first was a contradiction I wrote: the brief said to drive the depth test through run_program and
cited tests/spike.rs and tests/corpus.rs as precedent, while permitting neither those files nor lib.rs.
Approved its reading, which is right and costs nothing: run_program spawns its own thread with
INTERPRETER_STACK_BYTES, so a `#[cfg(test)] mod tests` inside eval.rs calling crate::run_program gets
the identical sized stack with no new file, and a unit test exercising public surface violates nothing
(the rule forbids integration-testing a PRIVATE subject).

The second is the one worth keeping. **Adding MAX_EVAL_DEPTH to eval silently changes what
tests/spike.rs's records_the_stack_cost_of_one_eval_frame measures.** That test is the sole provenance
of the 784-bytes-per-level figure and of INTERPRETER_STACK_BYTES's whole justification, and its method
is bisecting until the guard page kills the process. After the cap lands, the thing it bisects for is
unreachable through the public API. The implementer proposed leaving it alone because the file was not
permitted, and flagged it rather than ignoring it, which is the right instinct with the wrong
conclusion: a test that still passes while no longer witnessing what its name claims is precisely the
defect class this branch keeps finding, and a file-permission list is the wrong reason to ship one.

So spike.rs is now permitted, with the requirement to make the test honest either way -- measure below
the cap and say the cap bounds it, or state that the measurement needs the cap raised deliberately and
record how. **And record the consequence, which outlives the task: D19 requires the limit to sit below
what the stack survives, that bracket came from measuring the upper bound, and after this change the
upper bound is no longer reachable through run_program.** That figure has moved four times already; a
fifth revision with no way to re-derive it would be worse than the four.

Also told it not to overwrite INTERPRETER_STACK_BYTES's existing numbers with a new measurement unless
they disagree for a real reason. Three separate measurements of that cliff have each been correct for
their own commit, and flattening them would hide that every counter added makes the unprotected case
shallower than the measurement justifying it.

lib.rs granted to Task 11 as well: Interp's fields have no other home and execute is the only unpacker
of failure_site, so it is structural rather than convenience. Permitted set is now run.rs, eval.rs,
error.rs, tests/spike.rs, lib.rs and the report. That is most of the crate, which is honest for a task
this size rather than a failure of scoping, but it does mean no other lane can run beside it.

**Challenged its design rather than waving it through: a running `indent: usize` on Interp may be the
wrong shape.** Task 10's report, the only measurement anyone has taken of the rule, concludes the depth
is derivable from the AST statically. A given instruction's lexical nesting does not change between
iterations. If that holds, a mutable counter is redundant state whose only distinctive property is that
it can be WRONG, maintained on every exit path including the error paths and the Goto paths where
run_bounded swallows a jump without telling the arm. Same shape as the skipped pop_frame, except that
one has a chokepoint healing it and this would not.

Not overruled, because it is building a block stack anyway and may have found the static derivation
impractical. But required: decide deliberately, say which and why, and if the counter stays, write **a
raise that happens after a loop has exited, at top level**, where an undecremented counter over-indents
and a static derivation cannot. A counter with no desync test will desync silently, and the symptom is
two spaces of stderr that nothing asserts.

Also refused a three-element failure_site tuple. Option<(usize, Vec<u8>, usize)> distinguishes line
from indent by position alone, which transposes into plausible-but-wrong output rather than into a
failure. Named struct instead.

Flagged for whichever representation wins: Task 13 needs the same quantity for TRACE's *-* lines, from
a point in the code that may not be inside Task 11's loop machinery at all.

## Task 11 DONE: commit 2c9b966c. Corpus 12 -> 22 of 26. Review dispatched.

Verified: 781 tests green, clippy and fmt clean, five files exactly as permitted, tree clean. Eleven
differential probes byte-identical, covering every indentation case that was failing before this task
(one, two and three nesting levels, IF's THEN, WHEN's THEN), do until versus do while echoing different
clauses, an ordinary label on a loop, DO LABEL, and leave i reaching an outer loop from a nested one.
**The four remaining corpus failures are all TRACE**, so Task 13 closes the corpus.

It took the static indentation after I challenged the running counter, which is the outcome the
coherence audit predicted was correct, and wrote the desync test I named. Its stated reason for
rejecting the counter is the one I gave: the depth is a pure function of the flat instruction list.

Two things it stopped to ask about, both real conflicts I had written into the brief, and both resolved
by widening the permitted set rather than by it guessing. That is now five consecutive dispatches on
this project where asking found a genuine defect in the dispatch.

**Third occurrence of `git checkout -- <path>`**, disclosed unprompted again, and again harmless. The
pattern is now unambiguous: three of three implementers who were told in writing not to run that
command ran it anyway, and all three disclosed it. The instruction is not what is protecting the work;
the disclosure habit is. Worth treating as a fact about the tool rather than about the agents.

Also worth recording: spike.rs's loud-failure probe used `do i = 1 to 3` and Task 11 implemented it, so
the test would have started asserting 0 == NOT_IMPLEMENTED_EXIT. **That is the third time a witness has
been implemented out from under a test on this branch** (the first was the size-contract test moving
from `+` to `=` to a message send). The implementer caught and fixed it itself, moving the probe to
CALL. I have asked the review to confirm CALL is genuinely outside 4a rather than merely unimplemented
today, because that is exactly how the first two happened.

The review is with the agent that reviewed Task 10, so it already holds run_bounded, Flow and the
indentation rule. Four priorities, the first being to mutation-test the Goto-absorption trap rather
than read the code that claims to avoid it.

## Task 11 REVIEWED: spec PASS with 2 Important, quality PASS. Fix round dispatched.

The big claims hold and were checked by mutation rather than by reading. The Goto-absorption
avoidance is sound in both directions: a terminating mid-construct-return mutant kills 19 tests
including the named one with the lost-total signature (2 vs 9), and the true re-entry mutant makes it
hang, timeout 124.

**I was wrong to demand "only that test goes red" for that mutation, and the reviewer said so.** It is
unachievable by construction, because loop state lives in run_repeating's stack frame, so any
mid-construct return breaks every repeating loop including at top level. Worth remembering as a limit
on that heuristic: precision of a mutation kill is evidence about the tests only when the mutant is
local to what the test targets, and asking for it where the design forbids it invites an implementer to
contort tests to satisfy a metric.

Important 1, and the substantive one: **the 28.x indent rule is overfit.** Seven of fourteen oracle
probes diverge byte-for-byte on the indent alone -- 28.1 through an IF is 4 not 0, 28.5 through a
SELECT is 2 not 8, five more. Two of the current rule's cases do match, which is why it looked right.
The rule fitting all fourteen: every SELECT and every DO/LOOP except an unlabelled Simple block owns a
search frame, and each popped frame restores the indent to that construct's own static_indent, with the
error reporting the residual. **The static design survives intact**, which vindicates the challenge that
produced it; the fix is one origin.indent update in the forwarding arms. Indent-only, so clause, line
and rc all match and neither the corpus nor my eleven probes could have caught it.

Told the implementer to re-measure all fourteen itself rather than trust the reviewer's table, and to
add at least two shapes nobody has probed. A rule derived from fourteen points is exactly the thing
that fits its sample and misses the fifteenth.

Important 2, which I verified myself: **the new depth figure never reached the tree.** 1840 and 291,777
appear nowhere in lib.rs, eval.rs or spike.rs; INTERPRETER_STACK_BYTES still presents Task 7's ~1600 as
current. The report claims lib.rs "now carries this row". So the report needs correcting as well as the
code. Fifth revision of that figure, and one that exists only in a report nobody re-reads is the worst
of the five.

Minor: the OVER-stem comment says `over (a.)` is NOT detected. It is, and it takes the loud path. A
comment understating a safety property invites someone to "fix" it.

## Task 11 fix round: commit 1ff7ba61. Re-review dispatched. Corpus 22/26, all four remaining are TRACE.

Verified myself rather than accepting: 783 tests green, clippy and fmt clean, corpus unchanged at 22 as
expected for an indent-only fix, and 1840/291,777 now present in both lib.rs and tests/spike.rs.

**The substantive check was re-running the indent differential myself on eleven shapes**: the seven that
diverged, the two that previously matched, and two nobody in the chain had listed (a named LEAVE from a
loop nested in another loop, and a LEAVE inside a DO inside a WHEN). All byte-identical. That last pair
is the result that matters, because the worry with a rule fitted to fourteen points is that it covers
its sample and misses the fifteenth; two unlisted shapes passing is weak evidence it generalises rather
than none.

The implementer re-measured all fourteen itself before touching code rather than trusting the reviewer's
table, as instructed, and added three shapes of its own. The corrected rule keeps the static design:
origin.indent starts at full lexical depth, and every SELECT or non-unlabelled-Simple DO/LOOP the search
examines without matching resets it to that construct's static_indent before forwarding. Implemented as
pop_search_frame, still a pure function at each propagation step, so the round-one design decision
survives and only the two-family shortcut was wrong.

Re-review asked for the things my own verification cannot reach: whether pop_search_frame, now on the
normal propagation path, alters a search that MATCHES immediately; whether the twelve-shape table test
can fail, or merely encodes the implementation it was written after; and whether the depth row was added
beside the Task 7 row rather than replacing it, with the three earlier figures still reading as
correct-for-their-commit rather than as superseded errors.

State of the phase: Tasks 1-12 and 14 are done, 15a done. **Remaining: 13 (Trace, which closes the last
four corpus programs), 15b, and 16 (the gate harnesses and the gate assessment).** Task 13 waits on this
re-review, since it modifies run.rs and eval.rs and a second fix round would collide.

## Task 11 CLOSED. Re-review clean, one wording nit fixed at b8b8f16c.

The re-review answered all five of my questions with evidence rather than assurance. pop_search_frame
cannot alter a matching search, because match arms precede pop arms and both pop mutants left every
consumed-LEAVE/ITERATE behaviour test green, so the helper can only move FailureSite.indent and never
control flow. The twelve-shape table test dies on the right rows under three separate mutants,
including one that fails on exactly the single row discriminating the labelled-Simple clause. And the
independence was real: the reviewer oracle-probed the implementer's two new shapes itself before
comparing, rather than reading its table.

The depth row landed correctly, beside Task 7's rather than replacing it, with the whole five-value
lineage (820, 850, 783-784, 1600, 1840) each reading as correct-for-its-commit. Nit fixed: eval.rs
called that lineage dated when it is task-anchored, and a pointer describing a neighbouring document in
the wrong terms sends a reader looking for something that is not there.

## Task 13 dispatched, the last executor task. BASE b8b8f16c.

Brief extracted by hand again (39 lines) since the plugin cache is gone. Five things carried in the
dispatch that postdate the brief entirely:

* The retrofit is ONE insertion point in eval, not the eighteen arms the withdrawn D17 note claimed,
  because eval was already split from eval_node so every exit path passes through one place.
* static_indent and pop_search_frame already exist from Task 11, and TRACE's *-* indentation is the same
  quantity with a different formatter. Measured, not assumed: a reviewer probed a TRACE run with an IF
  and got four, matching what the error report produces for that shape. Told it that if it finds itself
  computing depth a second way, stop and say so, because two computations of one quantity is how they
  drift.
* Commit expectations rather than running the oracle live, and that is deliberately the opposite of
  tests/corpus.rs. Both are right; the distinction is between an instrument and an expectation.
* The 342 figure is 4b and 4c's, not its to satisfy, and if it recounts it must say which scan it used.
* It closes the corpus: 22 of 26 now, all four remaining are TRACE, so it should reach 26 and a shortfall
  is the most interesting thing it could report.

Also warned about the witness-implemented-out-from-under-a-test trap, three occurrences now, most
recently Task 11 implementing the DO that spike.rs used as its not-implemented probe. If any test of its
depends on something not working, it must pick something outside Phase 4 entirely.

## Task 13 found a real defect in Task 11's static_indent, including a live panic. Approved as a
## separate commit.

Seventh consecutive dispatch where asking before implementing found something real. I verified both
probes myself before approving a change to already-reviewed code: `else` traces at indent 2 with its
body at 4, `otherwise` at 2 with its body at 4, and the unreachable! is at run.rs:2268 inside
static_indent.

Four wrong answers, and they are not all the same severity. Three are wrong indents on marker clauses
(IF's own THEN returns 4 for 2, IF's ELSE falls through a strict `>` and gets the enclosing level, a
WHEN's THEN returns 6 for 4). **The fourth is a panic**: SELECT's OTHERWISE marker falls to
`unreachable!("a resolved SELECT's own range holds only its WHENs and OTHERWISE")`, and OTHERWISE's own
clause is unconditionally traced, so it is a dead process the moment TRACE runs over a SELECT with an
OTHERWISE.

**Why nothing caught it, which is the reusable part: none of these four marker clauses carries an
expression, so none can ever be a FailureSite, so the error-report path structurally cannot reach any
of them.** TRACE echoes every stepped instruction including markers, so this task is the first that
could see them. A defect invisible to every test that existed and reachable only by a later feature.
Task 11's review was mutation-thorough and could not have found this from the diff.

Conditions attached. It lands as a SEPARATE commit before the trace feature, because a fix to shipped
code buried in a 2,000-line feature diff is neither bisectable nor findable by its reviewer. The
OTHERWISE case gets its own test noting it was a crash rather than a wrong number. Task 11's
twelve-shape table must be re-run and the result stated, since narrowing two `>=` to `>` is exactly
where an off-by-one regression hides, and that table is a real instrument -- it was mutation-verified
and dies on the right rows.

And I asked it to reconsider the unreachable! rather than merely route around it. This crate has a
standing rule that the diagnostic path must not turn a reportable condition into a crash; error.rs's
catalogue miss renders visibly for that reason and run.rs:536 already cites it. static_indent now feeds
both the error report and trace, so it is squarely on that path. "Genuinely unreachable" is also what
that arm claimed before three ways in were found.

Also warned: the fix makes a marker exactly two less than its body in all four cases, a uniform rule
from four data points plus the C++. Probe two shapes outside the set first. A rule fitted to its own
sample is precisely what the previous task's fix round was about.

## static_indent fix landed at 4ec4884d, separate from the trace work as required.

Verified: 784 tests green, clippy and fmt clean, the new four-marker test passes, and Task 11's
fourteen-shape table still passes unchanged. The implementer said explicitly why that is expected rather
than lucky, which is the right way to report a green regression test: none of the fourteen shapes targets
a marker's own index, only body clauses, so the two comparisons that were narrowed cannot affect them.

It reproduced the panic against the unfixed tree before touching it, with a temporary test calling
static_indent directly and RUST_BACKTRACE on. That is defeat-the-mechanism applied to its own claim, and
it is the habit worth propagating: the alternative is fixing something that was never broken and
reporting a green suite as evidence.

The unreachable! became a documented `return 0` fallback rather than a reasserted invariant, per the
crate's diagnostic-path rule.

**Flagged one sibling it left behind**: static_indent still holds a live unreachable! at run.rs:2297,
for a `whens` entry that is not a When/WhenCase. Same function, same path, same argument it had just
accepted -- which is the "correction lands in one place and the contradiction survives in another"
pattern, arriving inside a single function this time. Its provenance is also no better than the one that
failed: it rests on Phase 3's invariant that `whens` holds only WHENs, and Phase 3's invariants have not
been perfect this phase, having elided the end-of-branch markers and admitted an absorbed WHEN that
executes while absent from its own SELECT's list. Told it to fold that into the trace commit rather than
amend 4ec4884d, since the fixed one was a measured defect with a reproduction and this is hardening, and
mixing them would overstate what was found. run_bounded's own unreachable! at 2768 stays: that is a
control-flow invariant this crate owns end to end, not a diagnostic formatter.

## Task 13 measured a real trace divergence, and proposed a resolution I rejected.

The finding is sound and well measured: **the oracle re-echoes a DO/LOOP's own clause, and its `end`,
on every iteration**, because the DO instruction genuinely re-executes each pass. Confirmed on both
`do while` and `do i = 1 to 2`. Our design resolves a whole loop inside one `step` call, so
step_in_temps_frame visits the DO's position exactly once however many iterations run.

Rejected the proposal on two grounds.

**The constraint it reasoned from binds control flow, not emission.** Task 10 and 11 keep a whole
construct inside one step call so a Flow is never returned mid-construct, because run_bounded would
absorb an in-range Goto. That is an argument about RETURNING. Emitting a trace line at the top of each
pass returns nothing and cannot touch the Goto trap, and run_repeating already knows its own instruction
index and re-evaluates its own condition every pass, which is exactly where the oracle re-executes. So
the hook is the loop driver, not the single insertion point. "The single insertion point does not cover
it" is an argument against that insertion point.

**And it proposed choosing the >K> witness from SELECT CASE's CASE specifically because that evaluates
once and so cannot expose the divergence.** That is the failure this project keeps finding, in its
purest form: criterion 3 would pass while the defect it exists to catch is present and untested.
Criterion 6 was rewritten because /bin/true satisfied its predecessor, criterion 4 turned out to have no
way to fail at all, and two tests have been caught passing against deliberately broken implementations.
A witness selected to avoid a known divergence is worse than an honest gap, because it produces a green
gate rather than a recorded one.

Ruled: if the divergence survives an honest attempt at the driver hook, the witness must still expose
it -- either fix it, or record the gap AND keep a loop-shaped >K> witness whose committed expectation is
the oracle's real output, with criterion 3 reported not met for that row. What is not acceptable is a
table that is green because of what it chose not to look at. Also rejected the supporting argument that
none of the four corpus programs needs it: that is a fact about the corpus, not about trace, and "no
current test proves it against us" is how a measured divergence becomes permanent by default.

Asked for three more measurements before it implements: whether DO FOREVER and a DO ... TO n containing
ITERATE re-echo the same way, and whether >K> reappears on a pass where the condition is not
re-evaluated. If the rule differs by loop kind, better to know before.

## The trace divergence dissolved: it read the C++ instead of inferring from output.

`DoBlock::checkControl` explains all of it. On every pass after the first, a TO-style loop reads the
control variable and traces it, computes value+by and traces that, then assigns; the first pass is
skipped explicitly, with a comment saying the initial assignment was already traced during setup. So
`>K> "TO"` firing once is not a divergence to reproduce, it is the oracle evaluating TO/BY/FOR once at
entry exactly as our run_loop does, and WHILE/UNTIL's `>K>` re-firing every pass is the oracle
re-evaluating that condition every pass exactly as our run_repeating does. **Both fall out for free
once emission is hooked at the evaluation points the drivers already visit.** No gap to disclose,
which is a better outcome than the honest gap I had ruled for.

I verified the citation by function rather than by reading the cited line, per the standing rule:
checkControl is at DoBlock.cpp:182 and both traceResult calls and the "already traced the initial
assignment" comment are inside it. The cited range starts a few lines before the function, but the
substance is genuinely there. Not a drift case.

The measured shape, for whoever needs it later: re-echo the DO/LOOP's own clause every pass; re-echo
END only on a pass that falls through the bottom, NOT on the pass where LEAVE fires; and for a TO-style
loop trace the old value then the new on every pass after the first. DO FOREVER re-echoes its clause
with no >K> and no value pair, having no control variable. ITERATE versus falling through makes no
difference to the pattern, because the value pair is about the control machinery rather than about how
the pass ended.

Asked for three things before it closes. The END-on-LEAVE rule gets its own test with a named mutation,
since it is one line of stderr conditional on how a pass ended and is exactly what works by accident
and breaks silently. DO OVER gets probed rather than assumed to match FOREVER, since it is the one loop
kind with a restricted target in 4a and so has had the least exposure. And the report must say which
rules were measured and which were derived from checkControl, because a measurement can be wrong about
the general rule while a source reading can be wrong about what the shipped binary does, and where the
two agree that is the strongest evidence available here.

DO OVER probed rather than assumed, and it has no wrinkle: `>K> "OVER"` fires once on the first pass,
the clause re-echoes on the failing pass exactly as FOREVER does, and there is no value pair, because
the pair is specific to a Controlled loop's `+BY` arithmetic in checkControl and OVER has none. So
TO-style and FOREVER between them cover the whole space. Did not interrupt it further; it asked nothing
and was proceeding correctly, and an acknowledgement would have cost it a turn.

## Task 15b dispatched in parallel with Task 13. They share no files.

Task 15a shipped the extractor half (extract_assertions, plus a reporting CLI and an invariant of
rows+dropped==assertSame calls that caught a real bug in the shared extractor). 15b is the consuming
half: rust/crates/rexx-exec/tests/assertions.rs, which is a new file touching nothing Task 13 owns.
Steps 1 to 3 of the brief are 15a's and done; 4, 5 and 6 are 15b's.

Scoped hard because Task 13 is live in trace.rs, eval.rs, run.rs, lib.rs and tests/trace_oracle/: 15b
may write only its own test file and its report, must ask before touching rexx-extract, and if it needs
something from rexx-exec's public API that does not exist it must stop rather than add it. Told it to
expect those files changing under it, which is fine for a test using the public entry point.

The five things emphasised, in order: compare byte for byte and never numerically, since a numeric
comparison would hide the entire created-digits story across thousands of rows; **prove the table can
fail by perturbing an expected value**, because criterion 2 has already been vacuous once, when it
quantified over extracted programs that executed nothing; carry NUMERIC DIGITS per row, since those
files run it from 1 to 100 and a row at the wrong precision passes while testing the wrong thing;
CONCATENATION's prelude trap with its precise 56-silent/332-loud split, including why the original
all-388-silent claim being wrong is itself load-bearing; and report blocked rows with the sub-phase that
unblocks each, since a blocked row is honest and a silently dropped one is not.

Also pointed it at tests/corpus.rs as a model for the report-versus-gate distinction and for the
inherited-stderr trick, since a harness that prints counts hits the same invisible-output problem.

## Task 15b found a second extractor modelling gap. Ruled: convert those rows, do not block them.

`self~expectSyntax(code)` sets up an expectation, enforced by OOREXXUNIT.CLS's own condition trap, that
some LATER statement in the same method raises that condition. So an assertSame after such a marker does
not mean "does expr equal expected", it means "does evaluating expr raise this condition". 184 of 4,259
rows: DIVISION 60, EXPONENT 6, REMAINDER 118. PRECEDENCE's 139 assertSyntaxError uses are wrapping calls
and unaffected.

I verified both halves before ruling: `"-5678932" % "-37"` under digits 5 raises 26.11 on the oracle,
and expectSyntax(26.11) carries the number inline. The marker is also not necessarily adjacent -- a
NUMERIC DIGITS sits between it and the assertion in the measured case -- so a previous-line check would
not have caught it either.

**Rejected the proposal to block them.** Those 184 are the only assertions in the entire table that
exercise the RAISE path; criterion 2 otherwise quantifies wholly over expressions that produce a value,
so blocking would discard the coverage that is hardest to get anywhere else in order to tidy a count.
And the conversion is nearly free, because the harness already has both halves: the expected number is
in the marker, and the harness already detects the escaping condition, which is precisely what it is
reporting as a false anomaly. So a false anomaly becomes a passing assertion and a real regression in
our error numbering becomes a failing one.

Conditions: match the full major.sub rather than the major, since 26.11 versus 26.2 is the difference
between a whole-number result and a DO repetitor and this project has already confused two error numbers
that way; state the marker's scoping rule explicitly, including what happens if a second marker changes
the expectation partway and why PRECEDENCE's wrapping form differs; and if conversion proves expensive,
block instead but list all 184 with the reason and record in the report that criterion 2 then covers no
raise path at all, so the gate assessment can carry it.

The transferable lesson, worth its own sentence in the report: 15a's comment reasoned that these markers
assign no variables and therefore cannot invalidate later state. **That is true, and irrelevant, and
being true is what made it convincing.** The marker does not change variable state; it changes what a
later assertion means. The same shape will recur wherever a harness models a suite's mechanics.

Also worth noting: 184 false anomalies is good news about the harness. It noticed. One that had silently
compared a raise against `1` and reported a mismatch would have been far harder to diagnose, and one
that somehow passed would have been worse.

## Tasks 13 and 15 both DONE. THE CORPUS IS CLOSED: 26 of 26, report AND strict mode.

Task 13, commits 4ec4884d (the static_indent fix, separate and first as required) and b3e2e112 (trace).
Task 15b, commit 7eeb309d: 4,224 of 4,259 assertion rows pass, zero value mismatches, zero raise
mismatches, zero anomalies. 804 workspace tests, clippy and fmt clean.

The 35 remaining assertion rows are attributed gaps, all in Literals.testGroup: 33 message sends to
Phase 5 and 2 function calls to 4b. Strict mode fails on exactly those, so criterion 2's gate is live
rather than nominal.

Task 13 built the per-iteration re-echo for real at the loop driver, which is the outcome I ruled for
after rejecting both the "hook only at the single insertion point" reasoning and the proposal to pick a
>K> witness that dodged the divergence. It then read DoBlock::checkControl rather than continuing to
infer, and the divergence largely dissolved.

Two more defects of the same family as the static_indent markers, found by re-running its own design-time
probes against the finished binary and diffing rather than assuming: OTHERWISE's own clause was missing
entirely from the echo, and a Simple DO...END block's own END was too. The family is "a clause that no
body range contains", and I have asked the review to look for a third.

**One gap survives and I recorded it myself at 6539ac11**, since the implementer could not write to
phase-4-exclusions.txt and flagged it rather than staying quiet: a Controlled loop's re-tested pass omits
two >>> value lines for its control variable. Cause read from the C++, not guessed. The fix means moving
the increment into loop_advance, restructuring a split that review rounds and a mutation suite have
certified, which is a poor trade at the end of a phase for two lines of trace output. Everything else
about loop tracing is byte-identical. Criterion 3's assessment must report the Controlled pair as
unwitnessed.

Both reviews dispatched. Task 13's went to the reviewer that did 10 and 11, with the committed-expectation
vacuity question first: a harness comparing our output against a file generated from our output is the
shape this project keeps finding. Task 15b's went to a fresh reviewer with the strongest framing I have,
that criterion 2 has already been vacuous once, so the question is not whether it passes but whether it
could fail.

**Remaining after these: Task 16 alone** -- the coverage enumeration, loud.rs, the mutation script, the set
assertion against the exclusions file, and the gate assessment that writes phase-4a-gate.md.

## Task 13 REVIEWED: spec PASS, quality PASS, 2 Important. One is a silent wrong answer.

The vacuity question came back clean, which is the answer I most wanted: all five committed
expectations regenerate byte-identical from the oracle, so they are the oracle's bytes and not our
output blessed after the fact. A one-byte perturbation fails exactly the right witness, and behavioural
mutations bite (dropping the per-pass re-echo, reclassifying TRACE R as I). One vacuous mutant disclosed
honestly: the outer intermediates gate is redundantly re-guarded inside every trace.rs method, so
flipping it alone survives -- defence in depth rather than a test gap, and mutating the mode constant
itself does die.

**The family hunt found a third member and it is not a trace defect at all.** The oracle EVALUATES an
absorbed WHEN's condition; we step it as a pure no-op. I verified it: `select / when 1=1 then / when
1/0 then nop / otherwise nop / end / say 'after'` gives the oracle rc 214 with 42.3 at line 3, and gives
us **rc 0 printing 'after'**. A silently wrong answer, which is the exact class this phase's
failing-loudly discipline exists to exclude. **Upgraded from Important to Critical** in the fix dispatch.

It was inherited from Task 10's pure-no-op When arm, whose reasoning was right for a WHEN in the whens
list and wrong for an absorbed one, which is a THEN instruction Select never sees -- Phase 3's own
comment says exactly that at LanguageParser.cpp:1319. It survived Task 10's review, Task 11's and mine
because **every probe anyone wrote used a side-effect-free true condition**. Trace is what exposed it,
by echoing a clause nothing else had reason to evaluate.

Also fixing: a bare count `do 2` emits no `>K> "FOR" => "2"`; LoopKind::Count produces no >K> at all.
Verified. The report claimed >K> verified correct while never probing Count, and that half of the
finding is the more useful one -- a verification claim that silently excluded a variant.

Corrected my own exclusions row at 90853346. I had written that closing the Controlled >>> gap means
restructuring a certified split; the reviewer costed it instead of repeating me, and it is about twenty
lines behind a not-first-pass flag mirroring checkControl, with re-verification of bound-before-test,
FOR and ITERATE being the real cost. Half a day. **An overstated cost in a gap row is how a cheap fix
stays open**, and the row is read by exactly the person deciding whether to close it.

## Task 15 REVIEWED: spec PASS with two rulings for me, quality PASS, 0 Critical.

The table can fail, proven six independent ways. The strongest is one the author never tried and the
reviewer did: forcing DIVISION test_262's carried DIGITS from 5 to the default 9 stops it raising
entirely, exit 0 printing 153484 -- so a digits-defaulting harness would have failed the raise rows
loudly rather than passing them quietly. Byte-not-numeric confirmed independently too, via a
numerically-equal-but-byte-different expected (20.000E+12 against 2.0000E+13), with `say` of the literal
rendering verbatim on both sides so there is no common-mode normalisation hiding the comparison.

**RULED on F7, the STRICT gate.** It cannot pass within 4a, because all 35 blocked rows ultimately need
Phase 5 and criterion 2 contemplates only 4b and 4c. That is a defect in the criterion, not the work --
the identical shape to Phase 2's gate, three of whose five criteria assumed an interpreter the phase
ordering does not deliver until Phase 4. Exempt the attributed rows, **and police the attribution the
way criterion 5 already polices its own**: commit the exempt set as an explicit list, assert it, and fail
STRICT both when an unlisted row fails AND when a listed row starts passing. Joining or leaving the
exemption then requires amending a committed list and shows in a diff, rather than being computed at run
time. Without that device, "attributed" is exactly the escape hatch the spec's own owner-arm paragraph
was written to prevent.

**RULED on F6.** Attribution-by-first-hit does not answer the brief's question. The reviewer found the
sharper truth: the two rows attributed to 4b hit xrange() first but re-block on message sends, so **all
35 are unblocked only by Phase 5**. Keep first-blocker as an observable, label it as such, and record the
unblocking phase separately. "33 on Phase 5 and 2 on 4b" would have someone at the end of 4b expecting
two rows to light up, finding they do not, and hunting a regression.

Four evidence errors to correct, all in comments and reports rather than behaviour, including one where
lib.rs states a false mechanism (assertSyntaxError does not check within the same call; OOREXXUNIT.CLS
:1203 shows it relies on the raise escaping to the trap) and the corpus-safety conclusion happens to
hold anyway.

**F8 was mine**, in the plan, fixed at 4f282fdc: the CONCATENATION silent/loud split is 106/282, not
56/332. Both halves wrong while the point they support was right, which is the awkward kind -- that
paragraph exists to stop a reader concluding the prelude hazard was imagined, and it argued so correctly
from numbers that were not.

## Task 13 fix round: be7cc8ef. The Critical is closed. Task 16 dispatched, the last task in the phase.

Verified the Critical myself with the exact case: rc 214 against the oracle's 214, byte-identical on all
three channels. The true-condition absorbed case, `trace r / do 2`, and a traced absorbed SELECT all
match too. Corpus still 26 of 26 in both modes, workspace green, 165 lib tests.

The implementer's own account of how it found the real rule is worth keeping: the original justification
rested on a probe where `n` stays 0, which cannot distinguish "never evaluated" from "evaluated and
discarded". A third probe with a printing absorbed branch separated them -- the condition IS evaluated,
so a raise escapes, but the branch never runs. **The defect was not that the measurement was wrong; it
was that the measurement could not see the difference**, and nobody asked what else it was consistent
with. Its new test distinguishes the two models by content rather than by a variable's final value.

Also: its first hand-composed expectation for the Count `>K>` was one re-echo pass short, found by
reading the real bytes back instead of trusting the guess, and it reported that rather than silently
correcting it.

Task 16 dispatched with the four things that decide whether the gate is worth anything: criterion 4's
rewritten anti-vacuity device and its negative control (a control you did not run is a claim, not a
check); criterion 5's owner arm needing its set asserted or a hard variant just gets relabelled; the
mutation script's exit-non-zero-on-unapplied-pattern guard, which has fired in four separate tasks here;
and that **the assessment is the deliverable, not the harnesses**, with an explicit instruction to say
where a criterion is met but weak. Two things it must report rather than paper over: criterion 2's strict
mode cannot pass within 4a, since all 35 blocked rows need Phase 5 while the criterion contemplates only
4b and 4c, and criterion 3's Controlled `>>>` pair is unwitnessed.

Told it plainly that phase-4-exclusions.txt exists and is ahead of its brief, and that following Step 4
literally would regress the file by dropping a deviation and the whole KNOWN GAPS section.

## Two more found by perimeter probing after the fix round. One is another silent wrong answer.

The re-review declared Task 13's fix round clean and revert-mutation-defended, then found two things by
probing the perimeter of the arm that changed. I verified both.

**F3 is a wrong answer, not a trace gap.** `select case 2 / when 2 then / when 3 then nop / otherwise
say 'O' / end / say 'after'` prints `O` then `after` on the oracle and only `after` for us. Evaluate-
and-discard is not the whole rule: on a FALSE absorbed WhenCase the oracle branches to its false_target,
which is the OTHERWISE. A true absorbed WhenCase matches, so the divergence is exactly the false path's
branch. The implementer's own comment called this narrowness "cannot reproduce the >>> pair", which is
the smaller half; the visible half is stdout. Sent for fixing, on the standard I have applied throughout:
a wrong answer gets fixed, not recorded.

**An asymmetry that must not be flattened**: the plain-WHEN false-absorbed analogue is SF #2018's
segfault, so the oracle cannot say what it should do and we must not probe it. WhenCase is measurable
only because the oracle survives it. Asked for that reason to be in the comment, since "upstream crashes,
so there is no oracle answer" is a far better comment than silence next to an untreated sibling.

**F4: comma-list conditions emit no >>> lines.** Verified: `when 1, 1 then nop` under trace r gets three
from the oracle and one from us, and it presumably applies at every comma-list site. Trace-only. Asked
for it if cheap at eval_logical_list, which already walks the elements with the values in hand, and to
stop and report rather than restructure if it reaches the trace architecture.

**Told t16-gate both, and to write criterion 3's assessment LAST** so it does not finalise against a
moving target. Also gave it the framing that matters regardless of how these land: criterion 3's
witnesses were regenerated byte-identical from the oracle, which is genuinely strong, but **three
separate trace gaps have now been found by probing shapes the witnesses do not cover** -- the Controlled
pair, the absorbed WhenCase, and the comma lists. A criterion whose instrument keeps passing while
adjacent probes keep finding divergences is met-but-weak, and the honest statement is that it verifies
what its five witnesses cover while its coverage of the trace surface is not itself measured.

## Ruled the ExprKind owner split, which the spec never made. Task 16 found the gap.

Criterion 5's owner arm needs one phase name per out-of-4a variant, and **the spec assigns exactly one
of the six**: Message is Phase 5's. The other five were never individually assigned, and the sentence
that looks like it does the job ("argument attachment inside Call, QualifiedCall, Message, List and
VariableReference is exercised by 4b and 4c") names two phases jointly for five variants and is about
when Phase 3's blind spot gets tested, not about delivery. Good catch; nobody had noticed.

Ruled: Call 4b, VariableReference 4b, QualifiedCall Phase 5, ClassResolver Phase 5, List Phase 5,
Message Phase 5.

The interesting one is Call, which is genuinely split -- a Rexx call resolves internal routine, then
builtin, then external, so 4b delivers one half and 4c the other. Named 4b, on the same rule I applied
to Task 15's blocked-row attribution an hour earlier: **the owner is the phase after which the variant
stops failing loudly.** Required the 4c half to be stated in the same comment, so a reader at the end
of 4b who finds `f(1)` still loud for a builtin name knows that is expected and not a regression. A
single owner string with an honest note beats a set the assertion cannot police.

Two conditions. The gate report must say five of six were a judgement call made at gate time, attribute
it to me, and give the one-line reason for each -- a gate presenting an invented assignment as though
the spec stated it is worse than one that marks its inferences. And **the split goes into
phase-4-exclusions.txt, not only into coverage.rs**, because that file is already the durable home for
"what 4a does not do and who owns it" and is what the set assertion points at. This project's most
repeated failure is a decision recorded where the next reader does not look: five instances this phase,
most recently the indentation living in four documents with four owners and in none of the places an
implementer reads. A comment in a test file is that shape exactly.

Also asked it to check its six phase strings against the split table's own spelling, since the assertion
compares strings and a criterion that fails over "Phase 4b" versus "4b" is one that gets weakened rather
than fixed.

## Task 15 CLOSED. Commit 8aa18b55. STRICT gate passes.

Verified: 810 tests, 0 failing, fmt clean, REXX_ASSERTIONS_GATE=1 now passes, counts unchanged at
4,259 rows / 4,224 passing / 35 exempt.

The exempt-and-police device is real, and its mutation evidence is the strongest kind: dropping
program_for's prelude write moved the not-passing count from 35 to 336 and **produced both violation
shapes at once** -- 22 stale exemptions that now pass but are still listed, and 323 unattributed
regressions. That is exactly what I asked the set assertion to catch in both directions, demonstrated
rather than argued. It also incidentally proved CONCATENATION's own a..g prelude is load-bearing, not
just the three Literals methods.

F6 resolved better than I framed it: it re-read test_string_range's source and found xrange() is
immediately followed by a message send in the same prelude, so 4b landing Call would not unblock those
two rows either. All 35 are unblocked only by Phase 5. It kept the observed first-hit construct as a
separate honest fact and **deleted owning_subphase**, the field that had been conflating the two.

TOOLING CORRECTION, and my standing dispatch instruction has been wrong: **`rustfmt <path>` with no
`--edition` uses 2015 and rejects let-chains.** Confirmed here on rustfmt 1.9.0: "let chains are only
allowed in Rust 2024 or later". It errors rather than silently mis-formatting, so nothing was damaged,
but every dispatch I have written says `rustfmt <path>` and should say `rustfmt --edition 2024 <path>`.
Fix it in the remaining dispatches.

MY OWN ERROR, worth recording because it is the class I keep flagging in others: I reported clippy as
showing 3 errors, then found it clean at exit 0 on a re-run. **I had sampled the workspace while another
agent was mid-write.** Running the suite against a tree two live agents are editing produces
measurement artifacts, and I did exactly what I criticised the EXIT range measurement for -- reported a
number without asking what else it was consistent with. Re-run before believing a red result when lanes
are live.

## F4 is a fix. Criterion 3's gap list is now closed at exactly one item.

Comma-list conditions now emit one >>> per element, one trace_result in eval_logical_list gated on
results. It covers all four keywords in one place because IF, WHEN, WHILE and UNTIL reach a comma-list
condition only through that shared function. I verified `if 1, 1`, `when 1, 1` and the raising
`if 1, 'x'` byte-identical.

Re-swept nine trace shapes myself: single- and multi-pass UNTIL, WHILE, FOREVER, OVER, three comma-list
shapes, and a Controlled loop. **Everything matches except the Controlled pair, and its diff is exactly
the two >>> lines per re-tested pass and nothing else.** So the gap is bounded and measured rather than
merely believed, which is a stronger statement than the exclusions row could make an hour ago.

**A latent DO UNTIL double-echo was exposed by the F4 fix**, not by any test failing. UNTIL's re-echo
had been wired to the top-of-loop site, right for WHILE, FOREVER and Controlled and wrong for UNTIL,
whose only re-entry event is its own bottom check. Found by re-sweeping every original probe after
making an unrelated change, which is the habit worth naming: **a change that touches shared emission
machinery invalidates every earlier measurement taken through it**, and the only way to know is to
re-run them.

That makes **four trace defects found by probing shapes the five committed witnesses do not cover** --
the Controlled pair, the absorbed WhenCase, the comma lists, and now the UNTIL double-echo. Four is a
pattern rather than three coincidences, and it is the sharpest thing the gate can say about criterion 3:
its witnesses verify what they cover, they were regenerated byte-identical from the oracle, and the
coverage of the trace surface is not itself measured by anything. Passed that framing to t16-gate with
the instruction to say it plainly, so a reader meeting a green trace suite does not conclude the trace
surface is verified.

F3 remains in flight and is criterion 1's territory rather than criterion 3's, since it is a stdout
defect. Corpus stays 26 of 26 either way, no corpus program having that shape.

## F3 and F4 landed at bca025c2. I found one divergence inside F3's own new path.

Verified: the OTHERWISE case prints O;after matching, the true-match WhenCase still never runs its
consequence, the plain absorbed WHEN raise is 214 both sides, corpus 26 of 26 both modes, workspace and
clippy clean, 169 lib tests. It read the parsed AST fields directly via a throwaway debug binary instead
of guessing a third time from output, and that is what found the real mechanism: the outer WhenCase's
false_target already structurally excludes the absorbed body on a true match, so only the false path
needed anything, and Flow::Goto(false_target) escapes the bounding run_bounded exactly as LEAVE already
does. No new mechanism.

**My own probe found the leftover**: with no OTHERWISE, the false absorbed branch lands on the END and
raises 7.3, and the clause echo indents 0 for us against the oracle's 4. Checked whether it was
pre-existing: plain SELECT and plain SELECT CASE reaching 7.3 are both byte-identical, so it is specific
to the absorbed node's false_target landing on the END -- the path F3 newly enables. Sent back with the
hypothesis that it is the same rule pop_search_frame already implements, an escaping Goto not popping
the frames the search would have popped, so the residual indent is the absorbed node's rather than the
enclosing construct's. If those are one rule they should be written as one rather than special-cased.

## Ruled (a) on criterion 6's uncaught mutation: add the witness, corpus files granted.

t16-gate applied the Controlled::order mutation for real and found **nothing catches it** -- 169 lib
tests and 26 of 26 corpus all still pass with TO/BY/FOR evaluation reversed. Its diagnosis is right and
the distinction it drew is the one that matters: this is not an equivalent mutant. The two pre-flagged
equivalents are semantically identical code; this is genuinely different code that is unobservable only
because **4a has no side-effecting expressions**, so TO/BY/FOR can only be literals or variable reads.
A coverage gap in the subset, the same shape as Task 14b's variant-coverage gap.

Documenting it instead would be the exact failure I rejected when Task 13 proposed a >K> witness that
could not expose a known divergence: a gate that reports a mutation as uncaught and ships with the
witness unwritten is a gate that chose not to look. Three conditions on the witness -- verify it fails
under the mutation before committing rather than reasoning that it should, capture the expectation from
the oracle rather than from us, and report rather than quietly adjust if adding a 27th program moves
criterion 1's enumerating expectations. Warned it that TO/BY/FOR are evaluated once at setup, so a
witness whose discriminating power depends on per-pass behaviour would not work.

## Task 13 CLOSED. Five commits: 4ec4884d, b3e2e112, be7cc8ef, bca025c2, 50774cd0.

170 lib tests, corpus 26 of 26 in both modes, workspace and clippy clean, and all 28 differential probes
from the task's history re-swept against the final binary with no divergence.

I tested the leak the fix claims to prevent rather than accepting the claim, since a set-once field on
Interp is the same class of mutable state I rejected in Task 11: an escape that lands harmlessly on an
OTHERWISE, followed by an unrelated failure later in the program, both at top level and nested inside a
DO. Both byte-identical, so consuming unconditionally on the next step rather than only on a failing one
does what it says.

**Its design reasoning is worth keeping, because it is the right shape of refusal.** It considered
growing Flow::Goto an indent payload to route the residual through pop_search_frame, and rejected it:
Flow::Goto is the ordinary resume mechanism at dozens of sites that have no residual to carry, so giving
all of them one to serve a single escape is the restructuring I had told it to flag rather than do. It
used the same conceptual rule through a field instead, and said so. Recognising that "write the two as
one rule" and "route them through one mechanism" are different requests, and that only the first was
worth having, is a distinction I did not draw when I asked.

It also confirmed the rustfmt edition finding independently: the bare form defaults to 2015 and rejects
this crate's let-chains outright, so it is a loud failure rather than a silent under-format.

Phase state: **every executor task is done.** Only Task 16 remains -- the coverage enumeration, loud.rs,
the mutation script with its now-granted Controlled::order witness, the exclusions set assertion, and
phase-4a-gate.md itself. After that, the whole-branch review.

## Criterion 4's mode was never built. Granted heap.rs and lib.rs to build it at gate time.

t16-gate blocked rather than fudging it, and it is right: **collect-on-allocation does not exist under
any name.** I verified independently -- no stress flag, no wrapped Heap, no cfg-gated build, Heap::collect
has no caller outside rexx-core's own tests, and alloc_with never collects. rexx-core/tests/collect.rs
unit-tests the collector in isolation and never runs anything through rexx-exec.

**My error, and it is a sharper version of the defect I was fixing.** Earlier today an audit found
criterion 4 could not fail, and I rewrote its anti-vacuity device to require a non-zero collection count
and a negative control. I never checked that the mode it quantifies over had been built. So I
strengthened an assertion whose subject was missing entirely.

Granted, tightly scoped, because the reason is bigger than the criterion: **this crate's whole rooting
discipline has never run against a collector.** Six eval.rs functions rely on temps-frame healing,
step_in_temps_frame exists solely to make that healing work, push_temp placement is argued at length in
several doc comments, and one under-rooted window is documented. All of it is careful reasoning
validated by nothing, and 4b is about to build on it. The cost is a flag and a call.

Conditions: off by default and provably inert, with the off path obviously unchanged rather than merely
equal; the non-zero collection count and the delete-a-push_temp negative control as already required;
and **if turning it on breaks real programs, report rather than fix** -- a rooting defect needs its own
round with whoever owns that code, and a hasty fix is worse than a recorded one. Better to end the phase
knowing about a real GC bug than not knowing. If it proves bigger than a flag and a call, it stops and I
take option 2, recording criterion 4 unmet with its diagnosis.

Either way the gate report must say the mode was built at gate time rather than during the phase, so
criterion 4 tested the tree for the first time on the day it was assessed. That is a real fact about how
much confidence it earns, and burying it would make the criterion look stronger than it is.

Granted value.rs and stem.rs too, for four one-line renames. Heap::alloc_with has no RootSet -- collect
needs one and Interp holds it as a sibling field -- so the stress call has to sit at the Interp level
right after each allocation returns, and the four production call sites are in value.rs's text()/number()
and stem.rs's two stem-creation sites. Nothing in lib.rs allocates directly.

Its rejection of the coarser alternative is the part worth keeping: collecting once per top-level
instruction would be a weaker claim wearing criterion 4's name, **and a criterion that names something
it does not do is worse than one recorded unmet.**

Required one addition beyond the four renames. "Exactly four call sites, verified by grep" is true today
and is exactly the fact that goes stale silently here: a fifth allocation site added by 4b would fail
nothing, the mode would simply collect less often, and criterion 4 would keep passing while testing less
than it claims. Same shape as every vacuity finding this phase. So a bare Heap::alloc_with call must
become the obviously wrong thing to write -- suggested renaming it alloc_with_uncollected so the name
warns at the point of use, with the Interp wrapper keeping the friendly name. **The guarantee has to
survive someone who has not read the report.**

The grant now covers heap.rs, lib.rs, value.rs, stem.rs, three test files, the corpus, the exclusions
file and the gate document. That is most of the crate for a gate task, and I told it to say so in the
report next to criterion 4 rather than leaving it in my ledger: **a gate task that had to build
production capability in order to assess a criterion is itself a finding about the phase.**

## Criterion 4's mode is built and the rooting discipline survived it. Corpus 26 -> 29.

The finding inside the finding: **its first version collected AFTER the allocation**, which swept the
value the call was about to return before the caller could root it, so all 29 subset programs panicked
including `say 1`. It caught that itself and named it correctly -- a mode that fails everything tests
nothing, the same shape as /bin/true. Collecting BEFORE the allocation instead asks whether everything
an earlier push_temp rooted still survives now that a new allocation is requested, which is the actual
discipline those doc comments argue for. After the fix, 29 of 29 pass with output identical to plain
run_program, 3,863 collections across the subset, every program above zero.

**The negative control fired: deleting eval_arithmetic's push_temp(left_value) panics 7 of 29.** And the
sharper half, which I would not have thought to ask for: the sibling push_temp two lines below is an
**inert control**, because right_value is read once immediately with nothing allocating in between, so
nothing ever asks whether it survived. Choosing that one would have produced a confident "the control
does not fire" and a false conclusion about the mode. **Not every push_temp is load-bearing, so a
negative control has to be chosen by where an allocation can intervene, not by convenience.**

No other rooting bug surfaced. So the phase's push_temp discipline, for what this subset exercises, held
against the first thing that has ever tested it.

The corpus went 26 to 29: it added witnesses for three previously-uncaught mutations, not only the
Controlled::order one I granted. All 29 match.

Closed the doc debt it flagged but could not reach, at e9e5c10b. Two run.rs comments named
Heap::alloc_with by its old name, and both said more than the name -- they argued the surrounding rooting
discipline was unverified because nothing ever collected. No longer true, so they now say what verified
it and how far that reaches, including the concrete seven-programs-panic evidence.

## Whole-branch review dispatched in four slices, plus 4b/4c scoping.

Range 9f68662a..HEAD: 112 commits, 115 files, 22,650 insertions. Too much for one reviewer, and a
reviewer who runs out of room narrows silently rather than saying so, so it is split by area with each
slice told to state what it did NOT reach.

* wb-exec (fable): run.rs, eval.rs, lib.rs. Told to hunt cross-task interactions rather than re-run nine
  per-task reviews -- run_bounded's Goto absorption, Task 11's block stack and Task 13's trace hooks were
  written by three different agents and sit inside each other. Plus every set-once Interp field, and any
  path that can produce a wrong answer at rc 0.
* wb-value (fable): the value model and all four supporting crates. D15's two rules, D15a's stem rules,
  the root set's truncate-to-watermark contract, and the collect-on-alloc mode built on the last day.
  Warned specifically about NUMERIC DIGITS above 1000, since this slice is the one most likely to try it.
* wb-harness (opus): every harness and the gate document. **Its single question is which instrument
  cannot fail**, given five prior instances on this project, all found by someone asking rather than by
  anything going red. The exempt list in assertions.rs is named as the attack surface.
* wb-docs (fable): plan, spec, exclusions, gate, corpus README, commit messages. Every number re-derived
  rather than re-read, every citation checked by containment rather than by reading the cited line, and
  the five-instances-this-phase pattern of decisions recorded where the next reader will not look.

Each carries the citation-drift method, the oracle's ulimit rule, the SF #2018 segfault warning, and the
note that the structuring-semicolon rule is withdrawn so nobody re-raises it.

Also dispatched scope-4bc (opus): the groundwork for both plans, not the plans. Its first job is the
inherited work list -- everything 4a deferred, **with a citation for each**, since that scattering is
this project's most repeated failure and I can already name six items off the top of my head. Then the
decisions Moritz must make before either plan can be written, stated as questions with options and a
recommendation. Then sequencing with the actual coupling named. Then what the 4a harnesses give 4b and
4c for free, including the warning that all 35 exempt assertion rows are blocked on Phase 5 rather than
on 4b or 4c, so nobody should promise they light up. Then how each of this phase's four recurring traps
applies to the next two.

Told it explicitly not to write the plans: Moritz decides scope and ordering, and the job is to make that
decision cheap.

## Branch review slice 1 (harnesses): one Critical, three Important. The gate's own instrument was vacuous.

**CRITICAL, confirmed by me at the source: mutate-4a.sh reports 9 of 9 caught, exit 0, with the oracle
absent.** The decision is `if run_subset; then NOT CAUGHT else caught`, so any non-zero exit counts as a
catch, including corpus.rs failing because the oracle binary is missing. Reproduced by the reviewer with
oracle_root repointed at a nonexistent path: nine for nine, having compared nothing.

**This is the /bin/true defect arriving from the other side.** Criterion 6's predecessor was satisfied by
a binary producing no output; its replacement is satisfied by a harness that cannot run. The script's own
header argues at length against reporting coverage that does not exist, and its stale-pattern guard was
built for this family -- it simply does not cover the instrument failing rather than the pattern. Worth
sitting with: **we rewrote a criterion specifically to escape vacuity, and the replacement was vacuous in
a way the rewrite did not contemplate.**

Important: 17 of 29 phase-4a.txt lines are silently droppable. Deleting Task 16's three mutation
witnesses leaves cargo test fully green while mutate-4a.sh falls 9/9 to 5/9. Adding a corpus program is
guarded because sourceline_oracle walks corpus/lang; removing one from the subset is not. Those witnesses
are load-bearing for criterion 6 and nothing protects them.

Important: trace_oracle's prefix-to-witness mapping is prose. The reviewer swapped keyword_while.rex for a
program emitting no >K> at all, regenerated its expectation from the oracle, and all five tests stayed
green. The expectations themselves are sound and all five reproduce byte-identically. But keyword_while
was chosen as a complete >K> answer **after I rejected a witness that dodged a known gap**, and that
choice turns out to be unprotected.

Important: criterion 6's gate verdict rests on a number from an instrument that cannot distinguish its own
failure from a catch. Asked for it to be rewritten after the real figure is known, showing the correction
rather than silently acquiring a new number.

What held, verified rather than assumed: corpus.rs cannot pass with the oracle absent and its STRICT mode
really fails; its uncaptured-report demonstration is a real negative control, since swapping to eprint!
makes it fail and the report vanishes; assertions.rs's EXEMPT fires both ways and reproduces Task 15b's
control to the row; coverage.rs and loud.rs genuinely fail to compile on a new variant; no mutation is
caught by a compile error. **The strongest result is criterion 4**: deleting push_temp(left_value) fails
collect_stress while the corpus stays 29 of 29 on the same tree, so the stress mode is independent rather
than a restatement of criterion 1.

## Branch review slice 2 (exec core): the cross-task interaction the slice existed to find.

F-EX1, confirmed by me: `select label s case 2 / when 2 then / when 3 then nop / otherwise say 'O' /
leave s / end / say 'after'` is O/after/rc 0 on the oracle and O then Error 28.3 **rc 228** on ours; the
iterate variant is 28.5 against our 28.4. The control is what makes it diagnostic -- the same program
**without** the absorbed WHEN is byte-identical, so it is specific to F3's escape.

Cause: F3's Flow::Goto leaves the Select arm without passing leave_select, so the OTHERWISE body runs
under the outer loop with no SELECT frame on the block stack -- **the exact pre-Task-11 shape that
leave_select was built to fix.** Neither task could have caught it. Task 11 predates the escape; Task 13
had no reason to write a labelled SELECT. This is the whole justification for a whole-branch review
after nine passing per-task reviews, and it is the second time this phase that Task 13's work exposed a
defect in a predecessor's.

F-EX2, latent but load-bearing: Flow::Leave/Iterate carry a fragment-table SymbolId across run_fragment
while every consumer above resolves it against the program's table. Spike-only today, **but 4b builds a
real INTERPRET on exactly this machinery**, so it is far cheaper now than debugged through a new feature.

Verified clean and worth recording as much as the findings: step's single-non-test-caller invariant
holds; all four set-once Interp fields are consumed on every path that can set them; and **all fourteen
external citations are accurate, checked function-first.** Given this branch's measured citation-drift
history, a slice finding zero is a real result rather than an absence of effort.

## Scoping for 4b/4c is in: 37 inherited items, 14 open decisions.

At docs/superpowers/specs/2026-08-01-phase-4bc-scoping.md, 647 lines, every inherited item carrying a
file:line or ledger citation. It found 31 beyond the six I could name unaided.

Four measurements that change the shape of both plans:
* **CALL does more to the report than the recorded gap says.** Each echo carries its OWN activation's
  line number, where INTERPRET's both carry the enclosing line, and each activation adds two spaces on
  top of Task 11's lexical static_indent. So 4b must add an activation base to a quantity deliberately
  built as a pure function of the flat instruction list, and that base has to sit outside static_indent
  or Task 11's design property dies.
* **rexxcps.rex cannot be a byte-for-byte differential**, which is what the parent plan assumes: it
  prints wall-clock timings and auto-adjusts its loop count from measured elapsed time, so host speed
  changes its control flow. The end-of-Phase-4 gate needs rethinking.
* Of the 15 excluded builtins, 11 are genuinely blocked, QUALIFY is not blocked at all, and
  USERID/SETLOCAL/ENDLOCAL are blocked for a reason nobody wrote down: std::env::set_var is unsafe in
  edition 2024 and the workspace forbids unsafe.
* Unseeded RANDOM is deterministic across separate oracle processes, so "the values vary" is not
  evidence of anything and parity means reproducing ooRexx's PRNG exactly.

Sequencing: **4b strictly first.** 4c depends on 4b in six named places and 4b depends on 4c nowhere.
The only real parallel slice is the ExprKind::Call arm plus the pure-string builtins, and only if the
builtin table gets its own module -- every scheduling collision in 4a was two agents in one file.

Its strongest recommendation, which I endorse: **every one of the 37 items must appear in the body of
the task that pays for it.** A plan that cites the scoping document instead repeats D17's mechanism
exactly, because per-task brief extraction makes anything outside a task's own section invisible.

## Whole-branch review COMPLETE, four slices. 2 Critical, 9 Important.

**F5 is the most serious finding of the phase, and I reproduced it: `a. = 5` then `say a. + 1` aborts
the process, rc 101, `unreachable!` at value.rs:219, where the oracle prints 6.** to_text directly above
handles Body::Stem; to_number never learned the value model creates one. Every to_number caller is
reachable with a bare stem operand: arithmetic, comparison, EXIT, DO counts.

**Why it survived everything, which is the finding about the instruments rather than the code.** 824
tests, corpus 29 of 29, nine per-task reviews, a seven-criterion gate assessment, and a mutation script
all passed over this. **No corpus program does arithmetic on a stem.** The gate's own coverage criterion
enumerates *variants*, not *combinations*, so `Stem` is covered and `Stem + arithmetic` is not. That is
the sharpest available statement of what 4a's gate does and does not buy, and it belongs in the 4b plan
rather than in a postmortem nobody reads.

Also confirmed by me: F4, `b.=a.; a.1=5; say b.1` gives 5 on the oracle and `A.` from us, because an
unset bare stem read must auto-vivify. F3, `numeric digits 3; do 12345` is 26.2 rc 230 on the oracle and
runs clean for us.

Dispatched fix-value for F5 and F4 (value.rs, stem.rs only, no collision with t13-trace in run.rs).
F1/F2/F3 are run.rs and queue behind t13-trace's current F-EX1/F-EX2 work.

Two findings from the docs slice needing action before 4b/4c planning:
* **TRACE ? is instance SIX of the decision-outside-the-brief failure.** The spec's Risks bullet demanded
  measuring `?` with no tty and either reproducing it or failing loudly; no task body carried it, so the
  implementation silently skips `?`. Measured: the oracle emits an interactive-trace banner and a
  `+++ "LINUX COMMAND ..."` stderr line we do not. A real unrecorded divergence, absent from KNOWN GAPS.
* **My own CONCATENATION correction was half a fix.** 4f282fdc corrected 56/332 to 106/282 in the plan
  and left the spec at line ~459 carrying the old figures AND a wrong mechanism, and left the plan's own
  corrected paragraph still saying a checker finds 332 loud failures. Correcting one document and leaving
  the governing one is precisely the contradiction-survives-elsewhere pattern, committed by me while
  fixing an instance of it.

What re-derived clean is worth as much: every number in the docs slice reproduced exactly, including all
four of D17's scan counts and the whole 820-to-1840 stack lineage, and every citation passed by
containment across three slices. **The citation-drift problem did not recur anywhere on this branch.**

## F4's real site is eval.rs, not stem.rs. Granted eval.rs, denied lib.rs on collision grounds.

fix-value traced it properly before writing code: eval_node's `ExprKind::Variable(id) | ExprKind::Stem(id)`
arm shares one path, and lib.rs's generic `read` returns a plain Text of the derived name on an unset
slot, discarding which stem was read. So `b. = a.` gets back Text("A."), stem_assign's is_stem check sees
a Text, and it takes the wrap-as-new-default branch instead of sharing the object. **The primitive
already exists**: stem.rs::read_by_name's own doc says it is "exactly what a bare stem read needs". It
was never wired. One call site, not a missing capability.

Refused the proposal to add the primitive unwired and flag it. That leaves F4 broken while looking
addressed and depends on a future reader connecting two halves nobody told them about, which is the
failure mode this phase hit six times.

Granted eval.rs, denied lib.rs -- **on collision grounds rather than scope**, since t13-trace is editing
lib.rs for the stale trace-sink comment and two agents in one file is how this project has lost work.

Option 2 is also the better design independent of permissions: the shared arm IS the bug. A bare stem
read and a simple variable read are not the same operation, one auto-vivifies and the other does not,
and the arm's own comment argues that a bare stem read "is not a new operation" -- a reasonable
inference from rendering-only evidence that is now measurably wrong. Option 1's trailing-dot sniff inside
a generic read() would have hidden the same distinction behind a string test.

Told it to keep what the old doc claim got right when correcting it: rendering-only measurements really
do show no allocation is needed, since `say b.` prints `B.` either way. The claim failed because
**aliasing is where object identity becomes observable**, and aliasing is 4b's. That account is more use
to 4b than a flat correction.

Also asked for two checks it would not otherwise make: whether auto-vivification is observable anywhere
other than through aliasing (drop on a never-assigned stem, a bare stem as a DO OVER target), and that
the vivified default is None rather than the derived name -- getting that backwards would break D15a's
tombstone rule while fixing F4.

## All four harness findings fixed. Gate instruments now earn their numbers.

mutate-4a.sh gained require_baseline_pass (the unmutated subset must report every program matching,
before the first mutation and after the last restore) and subset_status, which classifies each run as
PASSED / DIVERGED / INFRA_FAILURE **by parsing the actual "N of M matching" line rather than trusting the
exit code**. That fixes the class rather than the instance: an exit code cannot distinguish "the thing I
measured broke" from "my measuring device broke", and that was the entire defect. It reproduced the
reviewer's attack against the FIXED script before believing it -- now aborts at the first baseline check,
having touched no mutation. Still 9 of 9, now earned.

The subset list is pinned in coverage.rs the same way EXEMPT pins the assertion rows, and trace_oracle
now has WITNESS_PREFIXES as data with a test that each witness's expected stderr contains every prefix
it is named for, plus that the five-witness union equals the ten claimed prefixes exactly. Both verified
by reproducing the exact attack and restoring.

**Coordination note, and the scheduling error is mine.** t16-gate is blocked on a non-compiling tree
because two implementers are live in overlapping crate territory: fix-value in value.rs/stem.rs/eval.rs
and t13-trace in run.rs/lib.rs. Their file sets are disjoint so they are not colliding with each other,
but their union leaves the tree transiently uncompilable, and a third agent needing a clean tree to
produce gate numbers cannot work. Told it that waiting is correct and I will not ask for a workaround: a
gate document is precisely the artifact that must not carry numbers measured mid-refactor.

Also told it to treat its earlier clean run as **invalidated rather than re-confirmed**. fix-value is
changing to_number and how a bare stem read evaluates, which is upstream of arithmetic, comparison, EXIT
and DO counts, so the baseline, all nine mutations, the corpus and the workspace count can all move.

And required the gate to say the assessed tree is not the tree it first measured: two Criticals were
found by a whole-branch review AFTER the gate was written, one a process abort on a two-line program that
824 tests, 29-of-29, nine per-task reviews and its own seven criteria all passed over. **The coverage
criterion enumerates variants, and Stem was covered; what was missing was Stem combined with arithmetic.**
That limitation belongs in the gate rather than in this ledger, because 4b is about to be planned on the
assumption these instruments mean something, and they mean something narrower than they appear to.

## F5 and F4 fixed. One claim of the fixer's was overstated and I caught it by probing.

Verified against the oracle: `a. = 5; say a. + 1` gives 6 where it aborted the process; `b.=a.; a.1=5`
then `say b.1 / say b.7` gives 5 / A.7; a tail assignment over a default and a drop of an assigned stem
both still match. 175 lib tests, workspace green, corpus 29 of 29 both modes.

**It reported `say b. + 1` as matching "exactly including rc 215". It does not.** The rc and the 41.1
text match; the oracle emits one extra stderr line we do not:

    *-* Compiled method "+" with scope "String".

Found because I diffed the case rather than accepting the sentence. Not the fixer's to repair -- that
line is method-dispatch bookkeeping and needs the object model, so Phase 5 owns it.

**What makes it worth recording rather than shrugging at: the fix MADE it reachable.** `say 'abc' + 1`
has matched byte for byte all phase; a stem operand did not, because we used to hand back a plain Text
where the oracle has a real object whose `+` goes through method dispatch. Correctly vivifying the
object exposed an oracle behaviour that was previously unreachable. Same shape as F4 itself: object
identity becoming observable. A reader meeting this row must not read it as a regression, so the row has
to say it arrived with the fix.

Told it to correct the report first, on the grounds that "matches exactly" where it does not is worse
than silence, because it is the sentence a later reader trusts instead of re-measuring. Then add the
KNOWN GAPS row -- that section is pinned asymmetrically precisely so a measured divergence never goes
unrecorded for want of permission -- then commit its three source files plus the row, explicitly staged,
leaving run.rs's fmt diff alone since that is another agent's in-flight work.

## F5/F4 committed at 40da017e. Both Criticals from the branch review are closed.

Exactly four files staged -- eval.rs, stem.rs, value.rs and the exclusions row -- with the other three
agents' in-flight work verified unstaged before and after. 175 lib tests. It corrected the overstated
claim visibly, as a "Correction (post-review)" paragraph rather than a silent edit, which is the right
form: the old sentence is what a later reader would have trusted, so striking it silently would leave
them unable to tell the claim had ever been made.

The KNOWN GAPS row says the fix made the divergence reachable rather than caused it -- before the fix,
to_number's unreachable! aborted before the oracle's extra line could ever be compared -- and notes that
`say 'abc' + 1` still matches, which is what pins the gap to the stem's object identity.

STATE AT THIS POINT. Committed through 40da017e. Uncommitted and in flight: t13-trace in run.rs and
lib.rs on the leave_select bypass (F-EX1) and the fragment SymbolId id-space issue (F-EX2); t16-gate in
coverage.rs, trace_oracle.rs, mutate-4a.sh and phase-4a-gate.md, parked on a poll until the tree settles,
then re-running everything from a fresh baseline rather than re-confirming its earlier numbers. The
4b/4c scoping document is untracked and wants committing once someone has read it.

Branch-review findings still open, all Important, none Critical: F1 (controlled-loop header values not
rounded at loop entry), F2 (NUMERIC with an expression never traces >K>), F3 (DO and FOR counts validated
under fixed digits 9 rather than current DIGITS) -- all three are run.rs and queue behind t13-trace. Plus
the docs slice's TRACE ? divergence, which is instance six of the decision-outside-the-brief failure and
needs either a KNOWN GAPS row or a decision, and the spec's stale 56/332 CONCATENATION figures, which is
my own half-finished correction.

## Task 13's branch-review round: 958a06b9. Both Importants closed. Only t16-gate remains.

Verified all five shapes byte-identical, workspace green, corpus 29 of 29 both modes, 180 lib tests.
F-EX1's fix extracts OTHERWISE into a shared run_otherwise used by both the ordinary no-match path and
the escape redirect, so a LEAVE naming the enclosing SELECT LABEL from inside an escaped OTHERWISE now
resolves through leave_select. rc 0 where it was 28.3; the iterate variant is 28.5 where it was 28.4.
pop_search_frame's existing mechanism was enough -- no block-stack restructuring, as expected.

**It caught an error in its OWN round-3 formula, and my verification had passed that formula.** The
escape's indent_offset needed to be the constant 4, not current_value_indent - 2, which matched only by
coincidence at top level and gave 6 one DO deeper. I had probed the END landing at one nesting depth and
never the OTHERWISE-body shape deeper, so my probe set was incomplete in precisely the way I keep
flagging in others: **a passing differential is evidence about the shapes you ran, and the shapes you did
not run are invisible in exactly the same way whether or not they would have failed.** Its re-measurement
across three landing shapes at two depths is what found it.

F-EX2 took the contained fix rather than the restructuring: a named LEAVE/ITERATE crossing the fragment
boundary is now refused loudly at the one point that still holds the fragment's own SymbolTable, instead
of forwarding a table-relative id every consumer compares against the wrong table. It considered and
rejected both a new rexx-parse method (outside its files) and changing Flow's payload (the restructuring
I told it to flag), and documented the one residual: a bare unresolved LEAVE from a fragment still
reports <no failing clause recorded>, unreachable except through the spike, with nothing measuring the
oracle's behaviour there to aim at.

REMAINING: t16-gate alone, parked on its poll. It re-runs everything from a fresh baseline and revises
the gate document's framing. After that the branch is coherent and 4b can be planned.

## Documentation debt closed at be72c5fd. Two things left to close 4a.

TRACE ? now has a KNOWN GAPS row carrying its measured bytes rather than a description: the oracle with
stdin at /dev/null emits `+++ "LINUX COMMAND <path>"` before the first clause echo and
`+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++` at the end, and we emit
neither. stdout and rc match; only stderr differs. Owner unassigned, with the choice stated: reproduce
the banner, or fail loudly on `?` as the spec already sanctioned.

The CONCATENATION split is now right in **both** documents. My 4f282fdc corrected the plan and left the
governing spec carrying 56/332 **and** a wrong mechanism -- the silent set is not "the strict rows", it
is all 56 strict plus 50 non-strict. The plan's own residual "finds 332 loud failures" is fixed too.
Correcting one document and leaving the authoritative one is the contradiction-survives-elsewhere
pattern, committed by me while fixing an instance of it, which is worth remembering as the sharpest
version of that failure this phase produced.

The 4b/4c scoping document is committed unedited.

REMAINING TO CLOSE 4a, exactly two items:
1. t13-trace's last round -- F3 (DO/FOR counts under fixed digits 9, a silently wrong answer: `numeric
   digits 3; do 12345` runs for us and is 26.2 rc 230 on the oracle), F2 (NUMERIC with an expression
   never traces >K>), F1 (controlled-loop headers not rounded at entry). Dispatched.
2. t16-gate's final pass -- re-run everything from a fresh baseline, not a re-confirmation, and revise
   the gate document to say the assessed tree is not the tree it first measured.

Two Minors stay open by choice and are recorded rather than fixed: Heap::alloc still bypasses the stress
hook under a friendly name, and a DO/LOOP's temps frame grows for its whole run rather than one clause
(documented at step_in_temps_frame by Task 13's own round).

For whoever writes the 4b plan: **the F2 finding is the most transferable thing this review produced.**
Task 13's report enumerated the traceKeywordResult callers and missed NumericInstruction, and its
reviewer then checked our behaviour against that enumeration rather than against the C++. A list an
implementer supplies becomes the thing it is audited against, so an omission in the list is invisible to
the audit that would otherwise catch it. Reviewers should re-derive an enumeration from the source, never
from the report under review.

## Last code round closed: 556a84f7. F1, F2, F3 all fixed. Only t16-gate remains.

Verified: F3's DO and FOR shapes both 26.2/26.3 rc 230 matching, EXIT unaffected at rc 57 both sides,
183 lib tests, workspace clean, corpus 29 of 29 both modes.

**Two of my own probe results were false positives, and both would have been reported as regressions.**
My helper compared `2>&1` as one string. F2 showed DIFF and is actually identical on **both channels
separately** -- the merge conflated an interleaving D17 explicitly says is unobservable, since stdout and
the trace sink are separate descriptors. F1 showed DIFF and its stderr difference is exactly the two
`>>>` lines of the already-recorded Controlled-loop gap. So: **a probe that merges two channels whose
relative order is unobservable cannot distinguish a divergence from an ordering artifact**, and I have
been running that helper all session. It happened to be safe until now because nothing else emitted on
both channels in the same clause.

Three things the implementer did that are worth carrying into 4b's plan:
* It checked the **converse** edge nobody asked for -- `numeric digits 20; do i = 1 to 3 for 123456789`,
  which the old fixed-9 rule would have wrongly *rejected* -- so the fix is pinned from both sides rather
  than only where the bug was.
* It re-derived the traceKeywordResult caller set **from NumericInstruction.cpp** rather than from its own
  earlier report, which is exactly the correction that finding called for. It also measured the precise
  gate before writing: DIGITS and FUZZ trace only with an expression, FORM's keyword spellings never do,
  FormValue does, and the trace fires before validation.
* It rewrote whole_nonneg's doc comment rather than leaving a corrected implementation under the old
  "same rule as EXIT" argument, which was wrong by measurement. **A fix that leaves its own false
  rationale in place invites the next reader to revert it.**

REMAINING TO CLOSE 4a: t16-gate alone -- re-run everything from a fresh baseline, then revise the gate
document to state that the assessed tree is not the tree it first measured.
