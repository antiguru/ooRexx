# SDD ledger — plan: docs/superpowers/plans/2026-07-28-phase-3-parser.md

Context: Phase 3 of the ooRexx Rust rewrite — the parser (`rexx-parse`).
Phase 2 (`rexx-num`) is complete and is a dependency.

Verification method carried over from Phase 2, which is the project's asset:
differential testing against `build/bin/rexx`. For a parser the oracle is
less direct than for arithmetic — a parse tree cannot be compared to the
interpreter — so the substitutes are (a) evaluate parsed expressions with a
throwaway evaluator over `rexx-num` and diff results, (b) provoke each syntax
error and compare number, sub-number, line and column, (c) reconstruct
SOURCELINE and TRACE text from the AST.

Phase 2's twelve differential sets must stay at 0 throughout: 128,368 cases.
Regenerate with `python3 rust/crates/rexx-num/tests/gen-curated-sets.py <name>`
— addsub addsub2 muldiv md2 pow cmp fmt fmt2 fmt3 fmtedge fmtcarry signblank.

Tasks: 3.1 D10 spike, 3.2 ProgramSource/SOURCELINE, 3.3 scanner, 3.4 clause
splitting, 3.5 expression grammar, 3.6 the 35 keyword instructions,
3.7 directives, 3.7b public entry point, 3.8 errors with line and column,
3.9 TRACE formatting, 3.10 parse throughput.

Carried in from Phase 2, and the reason each is written down:
- A probe drawn from one dimension cannot reveal a second exists. The NUMERIC
  DIGITS rule took three attempts for exactly this reason.
- Target error boundaries deliberately. Phase 2's corpus sampled values and
  missed a defect living where a valid result becomes an error.
- Two of Phase 2's last three fix rounds INTRODUCED a defect while fixing one.
  Assume a fix round does this and hunt for it.
- Do not sort the keyword table; the C++ indexes it by position.
- Keywords are NOT reserved words. `if = 2; say if` prints 2. Recognition is
  positional and cannot live in the scanner.

PRE-FLIGHT: not started. Nothing dispatched yet — the plan is under review by
review-plan3 and Phase 2's final re-review (review-H2) is also outstanding.
Do not dispatch Task 3.1 until both return.

=== PICKUP STATE, 2026-07-28 13:20 ===
HEAD 08c8f355, tree clean, 78 commits on the branch. Nothing dispatched.

Phase 3 plan went through THREE review rounds (review-plan3). Each round found
defects, including in the previous round's fixes:
  round 1: 4 Critical, 10 Important, 7 Minor
  round 2: 2 Critical (both introduced by round 1's fixes), 7 Important
  round 3: 1 Critical (introduced by round 2's fixes), 5 Important, 1 Minor
All fixed. Exit gate went 6/8 -> 7/9 -> 9/9 satisfiable. The reviewer's round-3
verdict was "one short edit round away; nothing structural is wrong now".
The FOURTH-pass go/no-go was requested and never arrived — the agent died with
the session.

Also outstanding: Phase 2's re-review of commit 7f664aa7 (the H1/H2/H3 fix
round) has NEVER run. Three attempts: review-branch hit a session limit,
review-H2 produced nothing across two dispatches, review-H3 died with the
session. What IS verified is controller-direct: twelve sets at 0 across 128,368
cases, 176 tests, clippy clean, and each fix checked against build/bin/rexx on
its discriminating case. What is NOT verified is whether that fix round
introduced anything — and two of the previous three rounds did.

TRACE scope was cut deliberately after the user questioned the investment
(commit 08c8f355). `*-*` source lines stay gated because they cost only the
spans SOURCELINE already needs; the value lines are deferred to a Phase 4
decision because emitting an event per evaluation step forbids constant folding
and fusion. Also removed a per-node depth field the plan had required — TRACE
indentation is static nesting plus call depth, so both halves are derivable.

TO RESUME, in order:
1. Re-dispatch Phase 2's re-review over review-H-round1.diff (fresh agent; the
   dispatch text is in this repo's history of my messages, or rebuild from
   task-H-report.md "Fix round 1").
2. Optionally re-dispatch the Phase 3 fourth pass for a go/no-go.
3. Then Task 3.1 — the D10 spike. Throwaway code by design, so it is the
   cheapest place to be wrong if the fourth pass is skipped.

=== PRE-FLIGHT, 2026-07-28 (session 3) ===

Both outstanding reviews re-dispatched in parallel:
- Phase 2 re-review of 7f664aa7 over review-H-round1.diff (fable), report to
  .superpowers/sdd/2026-07-27-phase-2-numeric-core/review-H-round1-review.md
- Phase 3 plan fourth-pass go/no-go (opus), report to
  .superpowers/sdd/phase-3-plan-review-round4.md
Both told explicitly to assume the previous fix round introduced a defect, and
both told to message rather than only write the file.

Controller-verified preconditions for Task 3.1:

1. chumsky 0.13.0 is present in the offline registry AND builds. Probe crate
   built clean with `cargo build --offline`: 28 packages locked, nothing
   fetched. The plan's claim holds.
2. Two facts the probe incidentally confirmed against build/bin/rexx:
   `say -2 ** 2` prints 4, and `if = 2; say if` prints 2. Both are load-bearing
   in the plan and both are right.

Two data points for the D10 spike that the plan does not mention, found while
verifying the above. Neither is a defect; both belong in d10-decision.md:
- chumsky pulls 28 transitive packages against the hand-written option's zero.
- It depends on `stacker` -> `psm`, which has a `cc` build script, so choosing
  chumsky puts a C compiler on the build path. This branch's CI builds on five
  platforms including OpenBSD, so that is a portability cost, not a nit.

PRE-FLIGHT CONFLICT (one, resolved by the controller):
Task 3.1 Step 2 says "Four hours each, hard." A subagent has no wall clock, so
as written the timebox is unenforceable. Resolution carried in the dispatch:
the timebox means EQUAL EFFORT on both arms, and "the combinator arm did not
reach a working expression grammar" stays a legitimate decisive result. The
implementer stops an arm when it works or visibly stalls, and records which.
The plan text is left alone; this is a dispatch-level resolution.

Rest of the plan scanned for wall-clock timeboxes: only this one.

TOOLING HAZARD, found by testing rather than by hitting it later:
`scripts/task-brief PLAN 3.7` extracts BOTH Task 3.7 and Task 3.7b — 131 lines
with two `## Task` headings. The script's awk pattern is
`Task[ \t]+<n>([^0-9]|$)`, and `b` satisfies `[^0-9]`. So when Task 3.7 comes
up, extract it by hand or pass an explicit OUTFILE and trim, or the implementer
silently gets two tasks in one brief. `3.7b` on its own extracts correctly, and
`3.1` does not collide with `3.10` because a digit follows.
Task 3.1's brief is generated: task-3.1-brief.md, 113 lines.
Correct invocation is `task-brief PLAN 3.1`, not `3` — bare `3` also matches
`Task 3.1` for the same reason.

=== BOTH REVIEWS RETURNED, 2026-07-28 (session 3) ===

PHASE 3 PLAN, round 4: **NO-GO** — 1 Critical, 7 Important, 8 Minor.
Report: .superpowers/sdd/phase-3-plan-review-round4.md (452 lines).
C1: nothing in the plan produces the clause spans gate criterion 6 needs.
THEN/ELSE end a clause mid-line (InstructionParser.cpp:133 trimClause/
reclaimClause, NOT the clause splitter), a clause span includes its terminating
`;` and any trailing blank, and neither `Program` nor `parse_interpret` retains
clause spans. Touches Tasks 3.4, 3.6, 3.7b, 3.9.
The round-4 reviewer used `build/bin/rexxc`, which all three earlier rounds
never did, and that is where six of the seven Importants came from.
Round-5 fix dispatched (opus) with five controller rulings pre-decided.

CONTROLLER-VERIFIED, independently, before acting on the review:
- `rexxc FILE` is a PARSE-ONLY oracle. A file running `address system` +
  `"echo ..."` produces no output under rexxc, rc 0; under rexx it runs the
  command. On `x = )` rexxc gives Error 37 / 37.2 with the line, rc 219.
  It also gives the negative direction (this file parses), which running cannot.
- `samples/` is 301 .rex files, 67,519 lines, and ALL 301 pass rexxc.
  Adopted as a gate criterion; it is the only honest instrument for "every
  Instruction/Expr variant constructed".

PHASE 2 re-review of 7f664aa7: **CHANGES NEEDED** — 1 Critical, 2 Minor.
Report: .superpowers/sdd/2026-07-27-phase-2-numeric-core/review-H-round1-review.md
C1: at very large NUMERIC DIGITS the oracle raises error 5 where the crate
computes a result. Controller-confirmed and CHARACTERISED FURTHER than the
review did, at DIGITS 999999999999999999:

  oracle: error 5 for all seven of + - * ** / % //
  crate:  error 5 for / % //  ->  correct
          returns 5.0 / -1.0 / 6.0 / 8 for + - * **  ->  four divergences

But the divergence is NOT a clean function of DIGITS, and this is what the
review missed:
  '2.0'+0  ok      '2.0'*0  ok      '2.0'**0  ok     (zero operand short-circuits
  '2.0'+3  ERROR   '2.0'*1  ERROR   '2.0'**1  ERROR   before any allocation)
  2+3      ok  <-- all-literal operands are folded BEFORE the DIGITS change
  '2'+3    ERROR    applies, so they never see the large setting
  everything fine at DIGITS 1000.
So the review's proposed fix (a reservation at each operator entry point) is too
coarse: it would break '2.0'+0, which the oracle accepts. Any real fix must
mirror C++'s allocation points including the zero short-circuits.

MY OOM: bisecting the DIGITS threshold killed the session/machine. At 10^18 the
interpreter raises error 5 cleanly, but at intermediate settings it requests
gigabytes and gets OOM-killed. I capped the Rust harness with `ulimit -v` and
never capped the oracle. Guard now at scratchpad/rexx-capped (ulimit -v 1048576).
Threshold hunt ABANDONED — curiosity, not decision-relevant.

Probe errors I made this round, both already-known shapes:
- `numeric digits value d` is not NUMERIC DIGITS syntax; it abutted to
  "VALUE1000", so the bisect predicate was false for every input and the
  "threshold = 1000/1001" result was an artifact of lo never moving.
- `abs ('2.5')` evaluates to the string ABS2.5, because a blank before `(`
  makes it abuttal concatenation rather than a call. That is the very `f (x)`
  hazard the Phase 3 plan flags as the D10 discriminator, hitting my own probe.

=== PHASE 2 CLOSED OUT, commit f31753a3 ===
User ruled: record the error-5 gap as a deviation, do not chase parity.
Applied: both false uniqueness comments corrected (lib.rs SystemResources doc,
muldiv.rs reservation site), M1 (ten -> eleven sibling sets) and M2 (parse doc
now cites parseNumber's FSM at NumberStringClass.cpp:2586, not the
numberStringScan pre-check) fixed, and phase-2-gate.md gained a
"recorded deviation" section with the measurements, the three counter-intuitive
facts (zero-operand short-circuit, literal folding, fine at DIGITS 1000) and the
warning that a future fix must not sit inside sub/mul.
Verified after: 176 passed / 0 failed / 3 ignored, clippy clean with -D warnings.
Phase 2 now has NO outstanding review findings for the first time.

User also confirmed both parent-plan edits (drop column, adopt samples/).

CORRECTION: Phase 2 closeout commit is 5386195a, not f31753a3.
f31753a3 was amended away because I ran `cargo fmt`, which reformatted 33
files across the workspace and pulled unrelated churn into a comment-only
commit. DO NOT run `cargo fmt` in this repo: the workspace is deliberately not
rustfmt-clean (159 diffs at default width, 98 at max_width=125, no
rustfmt.toml, and adopting a width is still an open project decision). The
"format after editing Rust" rule in the global CLAUDE.md is scoped to
~/dev/repos/materialize and does not apply here.
Churn discarded with `git checkout --` on the 33 untouched files; my three
comment edits were re-applied by hand and the commit amended. Re-verified after:
clippy clean, 176 passed / 0 failed.

SUPERSEDED, same session: the "never run cargo fmt" note above is now WRONG.
Moritz adopted `cargo fmt` as the convention. One-off pass committed as
0fa62ca8: default rustfmt 1.9.0, no rustfmt.toml, 35 files, ZERO comment lines
changed (wrap_comments is off by default, which is what makes default safe for
this tree's hand-wrapped C++ citations). Verified after: clippy clean, 176
passed / 0 failed, `cargo fmt` idempotent.
Rule from here: run `cargo fmt` as its own commit, never mixed into a
substantive one. That mixing is exactly what went wrong before the amend.
Every Task 3.x dispatch should carry this instruction.
Branch now at 0fa62ca8, 80 commits.

=== ROUND-5 PLAN FIX DONE, re-review dispatched ===
Commits: ca4a7525 (parent plan, column removed), 26aefbce (Phase 3 plan, C1 +
I1-I7 + M1-M9), a2924ee3 (Instruction::clause_span named in all three tasks).
Fixer reports 16 of 16 fixed, 0 deferred. Treat that as the claim under test:
all four previous rounds broke something.
Re-review dispatched (opus) over .superpowers/sdd/phase-3-plan-round5.diff
(1667 lines, -U15), report to phase-3-plan-review-round5.md.

Fixer corrections I verified myself, both hold:
- `samples/*.rex` as a glob matches 36; `find samples -name '*.rex'` matches
  301. The gate criterion must use find. My original 301 came from find, so the
  measurement was right and the plan's loop needed fixing.
- 15 trace markers, not 11: `*-*` plus fourteen value markers
  (>=> >A> >C> >E> >F> >I> >K> >L> >M> >N> >O> >P> >R> >V>).

Fixer's other finds, not independently checked yet (flagged for the re-review):
- TokenCursor::next would trip clippy::should_implement_trait under -D warnings,
  failing gate criterion 8. Renamed `advance`.
- A third column claim in the parent plan at line 376, inside D10's own decision
  block, which is what Task 3.1 implements.
- Task 3.9 still said `*-*` "costs nothing beyond the spans SOURCELINE and error
  reporting already need" -- the exact sentence that let Task 3.4 ship with no
  clause-span rule. Corrected.
- I3's "7 names in both tables" is wrong as worded: 7 is the intersection of
  enum-constant suffixes, only 5 spellings are rows in both.

Two accepted deviations, recorded rather than fixed:
1. Plan markdown keeps `-` bullets and em-dashes, matching the existing document
   instead of the global style rules. `- [ ]` is required by the step syntax.
   Reflowing 1,000 lines would bury the substantive diff. Offered as a separate
   commit if Moritz wants it.
2. Label splitting sits in `split_clauses`, one layer below where the C++ does
   it, because `symbol :` at clause start is recognisable from tokens alone.
   THEN/ELSE cannot move the same way: keywords are not reserved, so only the
   instruction parser can tell a THEN from a variable named `then`.

MY ERROR, worth not repeating: `git checkout -- $(git status --short | awk
'{print $2}')` discarded the concurrent fixer agent's in-progress edit to the
plan file. I meant to revert only my own cargo fmt churn. Two lessons: scope a
revert to an explicit path list, never to whatever `git status` happens to show,
and never run a broad revert while an agent is writing in the same worktree.
The agent re-applied its edit, so nothing was lost.

=== ROUND 5 REVIEW: NO-GO. FIXED BY CONTROLLER IN 094a7c45 ===
Round 5 verdict: NO-GO, 1 Critical + 2 Important + 5 Minor, 16 of 16 round-4
findings addressed and 15 cleanly. Report: phase-3-plan-review-round5.md.
FIFTH consecutive round where the previous round's fix introduced a defect.

C1(new), confirmed by me against the oracle before acting: `split_before` took a
single cut point, but the oracle does NOT partition. Measured under trace r for
`if 1 = 1   then    say "a"`:
    `if 1 = 1   `   keeps all three trailing blanks
    `then`          no blank either side, despite four following in source
    `say "a"`       starts at `say`, zero leading blanks
Three spans, two gaps. In the C++: RexxClause::trim moves only the start
(Clause.cpp:138), RexxInstructionIf ends at the THEN token's start
(IfInstruction.cpp:58-66), RexxInstructionThen takes the token's whole location
(ThenInstruction.cpp:76). With one cut point, split_before(7) leaves a leading
blank on `say` and split_before(8) a trailing blank on `then`, so criterion 6
was unmeetable either way.

I applied all 8 fixes myself rather than delegating a sixth round, because the
fix was small, precisely specified, and I had the oracle output in hand. Fix
review dispatched over phase-3-plan-round6.diff (637 lines) -> round6 report.

Also verified myself while fixing:
- `end` traces as its own clause; `otherwise` alone with no trailing blank;
  `when 0 = 1 ` keeps its trailing blank.
- A `do i = 1 to 2` loop over one `say` prints SEVEN *-* lines for THREE
  clauses, so criterion 6 cannot be a sequence-length comparison.

MY VERIFICATION ERROR this round, the seventh of the session and the same shape
as the others: I reported 14 value markers because I grepped `>[A-Za-z=]>`,
a character class that structurally cannot match `>>>`. `>>>` appears 49 times
in TRACE.testGroup. There are FIFTEEN value markers plus `*-*`; the plan was
already right and I nearly "fixed" a correct number. Lesson is the standing one:
when counting things made of a character, do not exclude that character from the
pattern, and re-read the output instead of accepting the count.
The plan now carries that warning inline so an implementer does not repeat it.

=== ROUND 6: GO. CONDITIONS CLOSED IN b61d586a. PHASE 3 STARTED. ===
Round 6 verdict: GO, 0 Critical, 2 Important, 6 Minor, conditional on two edits.
6 of 8 round-5 findings fixed clean, 2 partial. Reviewer: "No seventh round
warranted." Report: phase-3-plan-review-round6.md.
Both Importants were the recurring shape again, and this time the survivor was in
MY fix:
- N1: Task 3.4 still said `span` is derivable from any token sub-range, four
  lines below the paragraph I had just added saying it is not. Falsified by the
  plan's own worked example: THEN spans tokens 6..8 but its byte span stops at
  token 6's end.
- N2: Step 3b bound three keywords while Task 3.6 Step 4 needs five and cited
  Step 3b for all five. Step 3b runs FIRST, so a tree-shape implementer could
  fold END into Do, satisfy 3b, and fail Step 4 five tasks later.
Both fixed in b61d586a, plus four stale spots my own sweep found afterwards
(Task 3.6 Step 4's three-vs-five wording, the Notes bullet, the SECOND marker
enumeration at line 1450, probe A's missing `end`).

Round 6 confirmed the things I asked it to doubt: WHEN measured independently
(not by analogy from END), the assert correct at all three boundaries with the
`|| end_at == cur.span.end` disjunct required, criterion 1's property 2 really
does permit interstitial whitespace, and the worked example's token indices
consistent with the signature.

MARKER COUNT, settled properly at last. Authority is the interpreter's table:
NINETEEN prefixes, `*-*` plus eighteen (RexxActivation.hpp:92-110), all nineteen
exercised by TRACE.testGroup. Sixteen have the `>X>` shape; the other two are
`<I<` and `+++`. Wrong three times running because every attempt matched a shape
that excluded what it sought. Both plan enumerations now defer to the table.

New, out of gate scope but recorded: a continued clause's traced text is NOT a
contiguous byte range. `say "x",` / newline / `    "y"` traces as `say "x","y"`:
comma kept, newline removed, continuation line's leading blanks kept. Neither a
slice nor a trim-and-join. Verified trace_output.rex and both Task 3.9 probes
contain no continuations, so criterion 6 is unaffected.

EXIT GATE: 10 of 10 satisfiable by this phase, 10 of 10 as written.
Criterion 1 clean for the first time in three rounds.

TASK 3.1 DISPATCHED (opus). BASE = b61d586a.
Brief: task-3.1-brief.md (165 lines, regenerated after the plan edits).
Report expected at task-3.1-report.md.
Dispatch carried: the timebox resolution (equal effort, not wall clock), the two
chumsky measurements (28 packages, cc on the build path via psm), rexxc as the
error-fidelity instrument, the three safety rules (no DIGITS > 1000, no broad
git checkout, no cargo fmt in a substantive commit), and probe discipline.
Remember when reviewing: generate the package with BASE b61d586a, never HEAD~1.

=== ROUND 7 REVIEW DISPATCHED: both unreviewed commits of mine ===
Scope: 094a7c45..a9bbb2a1, docs/ only. Package: phase-3-plan-round7.diff (850
lines). Report expected at phase-3-plan-review-round7.md. Model opus.
Two commits, neither reviewed by anyone:
- b61d586a closed round 6's two GO conditions (N1, N2) plus four stale spots my
  own sweep found. Round 6 never saw it.
- a9bbb2a1 added symbol interning to Task 3.3 and recorded the decision NOT to
  hash-cons the AST. No review of any kind.
Reviewer told explicitly that I wrote both, that my error rate this session is
documented, and to extend no benefit of the doubt. Also told the interning design
is three tasks from implementation and cheap to remove today, so a "reconsider
this" verdict is wanted if warranted rather than suppressed.
Reviewer warned another agent is live in this worktree on
rust/crates/rexx-parse-spike/ and rust/Cargo.lock, and to stay in docs/.

WHY INTERNING, since the plan now carries a decision no review asked for:
Moritz asked whether a term graph would cut allocations. It would not. D13
already gives one arena per code body with index references, so allocation count
is amortized O(1); hash-consing cuts NODE count, not allocations. It is also
structurally blocked: every node holds a byte range for SOURCELINE, error
reporting and TRACE, so two structurally identical subtrees at different
positions are not equal terms, and sharing them means moving spans into a side
table keyed by the identity sharing destroys. Plus a content hash per node sits
on the parse hot path, which under D2 IS cold-start time. Term graph is right for
a Phase 4 optimiser IR, and is now pointed there, where it belongs beside the
value-trace decision that forbids folding and fusion anyway.

What interning DOES buy: symbol occurrences outnumber distinct upcased symbols by
roughly an order of magnitude in the two bootstrap files. Stated as an order of
magnitude ON PURPOSE -- four crude counts gave 10.4x, 12.2x, 12.6x, 16.4x and
disagree because stripping /* */ comments, -- line comments and quoted literals
correctly needs the very scanner Task 3.3 builds. My first count reported THE as
the most frequent symbol, i.e. it was counting prose from the licence header.
Task 3.3 Step 5 is told to record the real ratio once the scanner exists.

SEMANTIC I NEARLY SHIPPED WRONG, now pinned in the plan: keying Program::labels
by SymbolId looked obviously right, but SIGNAL VALUE matches its computed name
LITERALLY against upcased label names. With a label spelled `target:`,
`signal value 'TARGET'` reaches it, `signal value 'target'` raises 16.1 quoting
"target", and static `signal TaRgEt` reaches it because that form goes through
intern. So the runtime lookup must NOT upcase. Hence SymbolTable::get, which is
documented as exact and non-upcasing for exactly this reason.

Task 3.1 remains undisturbed: every hunk of both commits is at line 331+, and
Task 3.1's section ends around line 215. Its brief was extracted to a file
before any of this anyway.

=== SCOPE DECISION, 2026-07-28: parse errors need not match C++ 1:1 ===
Moritz's call, after I laid out the cost structure. The agreed line:
  KEEP  correct error number AND sub-number.
  DROP  byte-exact message text.
  DROP  error 36's position substitution entirely.
  GATE  relaxed from byte-for-byte to "correct number and sub-number, on a
        plausible line".

Why numbers stay despite the relaxation: the message table is ALREADY generated
from interpreter/messages/rexxmsg.xml by rexx-inventory/build.rs and Phase 2
already consumes it, so number + sub-number + text are a table lookup, not work.
And parse errors ARE observable at runtime through INTERPRET: measured,
`signal on syntax` around `interpret "x = )"` traps with rc 37. 22 ooTest files
touch syntax, and L0 syntax errors is Phase 3's stated corpus level, so throwing
numbers away would have cost enumerated ooTest exclusions for no real saving.
What the relaxation actually buys: the byte-exact gate, Task 3.8's differential
ground-truth capture, and all per-line position tracking.

PLAN EDITS STILL OWED (deferred deliberately: the round-7 reviewer is live in
docs/ and editing under it is the collision I already caused once today):
1. Gate criterion 4 -> number and sub-number on a plausible line, not byte-exact.
2. Task 3.8 -> drop the differential byte-exact error capture; keep number and
   sub-number selection.
3. Error 36 position -> drop the requirement. Do NOT restore the old rationale
   sentence; the correct statement is "the oracle DOES expose a byte position in
   error 36's substitutions, and we deliberately do not reproduce it", not "no
   column exists".
4. Global Constraint line 15 -> scope it so a reader cannot read it as licence to
   gate on message text.

KNOCK-ON, worth recording so nobody later finds D10 resting on a dropped
criterion: error fidelity was the axis the plan called "most likely to decide"
D10, and Task 3.1's strongest argument was that error 36 is unreachable with
chumsky 0.13 (repeated()/or_not()/choice() rewind a partially-consumed
alternative and discard its error; no cut in 0.13). Devaluing that axis weakens
the RATIONALE but not the DECISION: D10(b) stands on the 8.6x throughput gap in
the grammar layer plus 12 net-new packages and a C compiler on the build path,
either of which is decisive alone under D2.

MY ERRORS #7 AND #8, both caught by the Task 3.1 agent from facts I had asserted
in its own brief:
- "There is no column anywhere in the oracle" is WRONG. Error 36.901 substitutes
  a 1-based BYTE offset within the token's own physical line. Measured:
  `x = "ää" || (a` reports position 15 where `(` is the 13th character. Subtler:
  `x = 1 ,` / `    + (a` puts line 2 in the main message (clause start) and
  line 3 position 7 in the substitution. One error, two different line numbers.
  What was right and survives: condition('o') has no column field.
  I repeated the wrong version in roughly six dispatches before it was caught.
- "`abs ('2.5')` is the string ABS2.5" is WRONG. It is `ABS 2.5`; blank abuttal
  inserts one blank, which is the blank operator and distinct from ||. My own
  earlier probe output showed the blank and I misread it. Same shape as the
  other seven: re-reading the output would have caught it, suspicion did not.

=== TASK 3.1 COMPLETE + AUDITED ===
Commit 43c4ba5f (d10-decision.md), follow-up 4abd61f4 (methodology + position
guidance). Status was DONE_WITH_CONCERNS; both concerns were MY errors, see below.

D10 DECIDED: (b) HAND-WRITTEN RECURSIVE DESCENT. Contradicts the plan's stated
starting position (a) on all four axes.
  LOC            277 hand vs 336 chumsky (+21%)
  throughput     hand 1.08-1.15 ms vs comb 9.3-12.1 ms, MEDIAN 8.3x
                 (agent first said 8.6x from ONE run, then self-corrected to a
                 median of seven; per-run 8.2 8.3 8.3 8.3 8.7 8.7 10.5)
  dependencies   12 net-new packages + a C compiler on the build path via psm
  error fidelity 27/27 both, but chumsky only with a hand-written pre-pass
STEP 3b: flat instruction chain, Vec<Instruction> + next: Option<InstructionId>,
nesting by index. Satisfies the five-keyword constraint by construction: there is
no parent node for THEN/ELSE/OTHERWISE/WHEN/END to be absorbed into.
The strongest single argument, and I would not have predicted it: error 36 is
UNREACHABLE with chumsky 0.13. repeated()/or_not()/choice() all rewind a
partially-consumed alternative and discard its error, and 0.13 has no cut, so
`(a[1` blames the outer paren. 27/27 came only from bolting on a hand-written
bracket-balance loop.

SPIKE PRESERVED, because the plan told it to delete the crate and that would have
left the numbers unauditable -- the same mistake Phase 2 made with its oracle
captures, which is why gen-curated-sets.py exists.
  branch spike/d10 = afdd3b1b (unmerged, parent 43c4ba5f, 11 files)
  patch .superpowers/sdd/2026-07-28-phase-3-parser/d10-spike.patch (2740 lines)
  run: cd rust && cargo test --offline -p rexx-parse-spike
       cd rust && cargo run --offline --release -p rexx-parse-spike --example throughput
I AUDITED IT MYSELF in a throwaway worktree: 7 tests green, throughput
reproduced at 8.0x against their median 8.3x. The archive genuinely builds, so
the decision's basis is checkable rather than asserted. Cargo.lock on
plan/rust-rewrite verified byte-identical to its committed content.
PLAN DEFECT worth fixing before any future spike: Task 3.1 Step 5 says "delete
the spike crate". It should say preserve-then-remove.

=== ROUND 7: GO. FIXED IN b00110e1 ===
0 Critical, 5 Important, 8 Minor. Round 6's two conditions both closed, no
round-6 fix introduced a defect. Report: phase-3-plan-review-round7.md.
I1 was a real defect in MY interning design: Program::labels keyed by SymbolId
cannot represent a LITERAL label, and was wrong in both directions. A label may
be a symbol OR a literal and the C++ keys on the token's value -- upcased for a
symbol, verbatim for a literal (InstructionParser.cpp:153 isSymbolOrLiteral,
labelNew at :2795). Measured all six cases: with 'MiXeD': present,
signal value 'MiXeD' reaches it, while signal value 'MIXED' AND signal MiXeD both
raise 16.1. Now BTreeMap<Box<str>, usize>. Nothing in this phase's gate could
have caught it: 16.1 is runtime, so it would have landed in Phase 4's interface.
Knock-on: Literal is the asymmetric token kind. Its value is NOT a slice of its
span ('it''s' decodes to it's; the x/b suffixes convert bytes), so it carries a
decoded value, which is where a literal label's key comes from.
I2 Keywords was named 3x and specified nowhere -> now defined over all SIX
spelling tables, not the two the old sentence named. I3 the Symbol payload broke
Task 3.3's own five scanner tests (E0308) -> payload-free Tag added. I4 intern's
fast path allocated on BOTH branches because Box<str>: From<&str> copies, so my
comment claiming otherwise was false and the branch bought nothing -> Cow.
I removed SymbolTable::get: it existed only for the label lookup that no longer
goes through SymbolId, and an accessor with no caller is speculative.

=== UNICODE: PARKED 2026-07-28, recorded as D14 ===
Commits c20b89a0 and 0cf362f4 in docs/superpowers/plans/2026-07-27-rust-rewrite.md.
Moritz's intent: after the rewrite works, switch to proper UTF-8 with character
length and byte length as distinct functions and checked conversion, explicitly
WITHOUT Python's intermediate state where Unicode strings were a separate type.
Not being done now. D14 records what must not block it.

State of the world, measured not assumed:
  ooRexx 5.3 strings are BYTE strings. length('ää')=4, substr(s,1,1)=C3 (splits a
  UTF-8 sequence), reverse gives A4C3A4C3 (invalid UTF-8), pos('ä',s)=1.
  utf8proc IS vendored but used in ONE place, Scanner.cpp:49, solely so error
  13.1 can print a whole character. We no longer reproduce parse-error text, so
  we need no equivalent.
  SysToUnicode/SysFromUnicode are Windows-only RexxUtil codepage helpers.
  No Encoding class exists.

THE PROBE THAT SETTLES THE DESIGN: x2c('C3A4') == 'ä' is 1. A source literal is
not text that gets converted to bytes, it ALREADY IS the bytes. So there is no
text/bytes distinction and nothing to convert -- Moritz's proposed automatic
conversion (x2c yields bytes, bitand converts a generic string) presupposes a
distinction the language lacks. Stream I/O agrees: FF FE C3A4 00 41 round-trips
through charout/charin unchanged, embedded NUL included, length 6.

SO THE OPEN QUESTION IS NARROWER and is what to pick up later: what does a
CHARACTER-level operation do when the bytes are not valid UTF-8? Raise, replace
with U+FFFD, or fall back to one character per byte. A total-function question.
Second open fork: does LENGTH keep byte semantics with a new character BIF, or
become characters with a new byte BIF? Compatibility judgement, not
representation. D14 forecloses neither.

The distinction that makes the objection precise: Python 2's failure was not
implicitness, it was that the encoding was AMBIENT and locale-dependent. Python 3
did not remove that, it moved it to the I/O boundary where open() still consults
the locale. Byte strings with byte-transparent I/O remove the failure mode rather
than relocating it.

A CAUGHT DEFECT, found by Moritz asking the Unicode question one task before it
would have mattered: the plan typed the retained source as Rust String in six
places with line() -> Option<&str>. String requires valid UTF-8; Rexx source does
not. Measured, a literal holding a raw FF FE runs, c2x gives FFFE, length gives 2.
A String-typed ProgramSource REJECTS LEGAL PROGRAMS. Now Vec<u8> in, &[u8] out,
with a test pinning it. Costs nothing above the scanner because characterTable is
zero for every byte 0x80-0xFF, so a symbol cannot hold non-ASCII (bäc = 2 is
error 13.1) and symbol->&str conversion is infallible.

Audit result for the future switch, all clean: rexx-core has NO Rexx string type
yet so the decisive choice is unmade; the only &str on a value path in rexx-num
is compare/parse, and string_order is already byte-based underneath so it is a
signature change not a rewrite; Number::parse is sound as-is because a non-UTF-8
sequence can never be a valid Rexx number and maps to error 41; all 13
character-oriented call sites are message rendering over generated ASCII.
D14's cheapest forward-compat rule: leave room for a LAZILY computed encoding tag
on the string object, so checked conversion is O(1) after the first check.

=== WHERE TO RESUME ===
HEAD 0cf362f4. Plan at GO after seven review rounds, exit gate 10/10 satisfiable.
NEXT: dispatch Task 3.2 (ProgramSource and SOURCELINE). BASE for its review
package is whatever HEAD is at dispatch -- never HEAD~1.
Task 3.2 is the task that carries the byte-source change, so its brief must be
regenerated after b00110e1 rather than reused.
Owed, small: Task 3.1 Step 5 should say preserve-then-remove the spike crate.

=== TASK 3.2 DISPATCHED (sonnet). BASE = 0cf362f4 ===
Brief regenerated after b00110e1: task-3.2-brief.md, 111 lines. The old brief
would have been stale, since 3.2 is the task that carries the byte-source change.
Report expected at task-3.2-report.md.

Dispatch carried four things the brief cannot know:
1. rexx-parse DOES NOT EXIST yet. Task 3.2 creates it. Workspace uses
   members = ["crates/*"] so rust/Cargo.toml needs no edit, but the new
   Cargo.toml MUST carry [lints] workspace = true or unsafe_code = "forbid"
   does not apply to it. No dependencies; rexx-num is not needed until later.
2. ParseError belongs to Task 3.3, not here. Nothing in 3.2 fails: new() is
   infallible, line() returns Option. Told explicitly not to invent an error type.
3. Task 3.1's spike crate is deleted and must not be recreated or looked for.
4. Vec<u8> not String is the load-bearing part; told that reaching for String or
   &str for the source or a line IS the defect this task exists to avoid.

Two ambiguities I resolved rather than leaving to guesswork, both told to defer
to the oracle if it disagrees:
- a line's slice excludes its terminator
- line starts go AFTER any \r, so CRLF content carries no stray carriage return

Probe targets named, because guessing them is the likely failure mode:
SOURCELINE(0), out-of-range, no-argument SOURCELINE, whether the count includes
a final line with no trailing newline, and an empty file.

Task 3.2: complete. Commits 68a3b553 (implementation) + fb2924db (Minor fixes).
Review: spec PASS, quality APPROVED, 0 Critical, 0 Important, 2 Minor.
Report task-3.2-report.md, review task-3.2-review.md.
Created the rexx-parse crate: Cargo.toml with [lints] workspace = true, no
dependencies, src/lib.rs, src/source.rs, tests/sourceline.rs.
Verified by me, not taken on report: 11 crate tests, 187 workspace, 0 failures,
clippy clean -D warnings, fmt clean, no String/&str on the source or line path.

Both Minors fixed immediately rather than deferred, because they sat at the
boundaries whose spans ten later tasks depend on, and Phase 2's lesson is that
eight defects were found after tasks were reported done, none by tests written
alongside the code. No defect was found: every new assertion passed against the
existing implementation, so the boundaries were right and merely unpinned.
  M1 line_of's doc comment stated no contract (clamp/terminator behaviour lived
     only inline, inverting the doc-states-contract convention). Now stated.
  M2 crlf_pair_..._lone_cr tested only half its own name; and NOTHING exercised
     line_of at a terminator byte, at exactly len, past the end, or on an empty
     source. Added, asserting CONTENT not just counts, since a count cannot
     distinguish one terminator from two.
Oracle claim I introduced and then checked rather than inferred: a file of three
newlines reports sourceSize 3; ab\ncd\n reports 2.

The reviewer also ran two oracle probes nobody had: `a\r\rb` gives 3 and
`a\n\n\rb` gives 4, both matching the implementation.

FORWARD GAP for Task 3.9, noted not fixed: ProgramSource exposes new/line_count/
line/line_of and NO accessor for the whole text or its length. Task 3.9 must
reconstruct clause text from a byte range and cannot, so it will need one. Its
Files list already includes src/source.rs, so that is the right place. Not added
now: YAGNI, and the brief's interface list did not have it.

=== NEXT: Task 3.3 (scanner and tokens) ===
BASE for its review package = fb2924db, or whatever HEAD is at dispatch. Never
HEAD~1.
Regenerate its brief: 3.3 changed heavily after b00110e1 and 8188cacf (interning,
Keywords over six tables, the Tag enum, Ctrl-Z and CR/LF terminator semantics).
Task 3.3 is the biggest task in the phase: 19 token classes, the significant-blank
rule, the Eoc model, ParseError's minimum definition, ParseCtx, TokenCursor,
SymbolId/SymbolTable/Keywords, and the differential scan check.
Consider splitting it if the implementer reports it as too large.

=== TASK 3.3 DISPATCHED (opus). BASE = fb2924db ===
Brief regenerated after b00110e1 and 8188cacf: task-3.3-brief.md, 514 lines --
much the largest in the phase (3.1 was 165, 3.2 was 111).
Dispatched WHOLE rather than pre-split. The six-step TDD structure is intact and
the seams are already there; splitting would mean inventing an interface between
halves that the plan does not define, which risks more than it saves. Told to
commit once per Step, and that reporting BLOCKED with a proposed cut line and the
interface the halves would share is a useful answer rather than a failure.

INTERFACE GAP I RESOLVED, because the brief could not be implemented as written:
ProgramSource exposes new/line_count/line/line_of and NO way to reach the bytes,
while Task 3.3's Files list excludes src/source.rs. So the scanner had nothing to
scan. Resolution: it may add ONE accessor,
    pub fn line_span(&self, n: usize) -> Option<Range<usize>>
returning the line's content range in the retained text, and must iterate lines
via line_count + line_span, scanning each line's slice and adding the line start
to get absolute spans. That is what the C++ does (ProgramSource slices lines
before the scanner runs), and it keeps the terminator rules in ONE place, which is
what the plan's "must not re-derive line boundaries" instruction demands.
Explicitly told NOT to also add a whole-text accessor, since a second one invites
re-deriving line boundaries, and to message me rather than adding more silently.
This supersedes the ledger's earlier "forward gap for Task 3.9" note: line_span
lands in 3.3, so 3.9 needs whatever remains on top of it, not the whole thing.

Also carried: Ctrl-Z and terminator handling are ALREADY DONE in 3.2 and must not
be redone; D10 closed so write it by hand and chumsky must not become a dependency
of this crate; ParseError minimal here and completed in 3.8; parse errors are not
1:1 so record number and sub-number only, no text or substitution machinery;
symbol->&str conversion is infallible because characterTable is zero for
0x80-0xFF; do not rename TokenCursor::advance back to next (clippy
should_implement_trait vs gate criterion 8).
Warned specifically that `abs ('2.5')` is `ABS 2.5` and that this case IS the
significant-blank discriminator, since I got that one wrong myself earlier today.

=== TASK 3.3 IMPLEMENTED, review dispatched ===
Four commits, one per Step, base fb2924db:
  4e549dcf ProgramSource::line_span
  121aea8c token.rs -- 19 token classes, SymbolTable, Keywords, ParseCtx,
           TokenCursor, ParseError
  3526a813 scanner.rs + tests/scanner.rs
  410b2783 harness payload dump, hex/bin literal differential, panic sweep
Verified by me: 56 tests in rexx-parse, 232 workspace, 0 failures, clippy clean
-D warnings, zero unsafe, chumsky NOT a dependency, tree clean.
Review dispatched (opus) over review-fb2924db..410b2783.diff (119 KB).

Differential coverage the implementer ran: 12,059 + 790 + 142 + 17 files against
rexxc, 40 crafted error cases, 518 hex/bin literals, agreeing on number,
sub-number and line.

TWO FINDINGS I VERIFIED MYSELF, both real and both gaps in the brief:
1. A `#!` line on line 1 must be SKIPPED BY THE SCANNER but RETAINED by
   ProgramSource. Oracle: rexxc rc 0, sourceline() counts it (3 for a 3-line
   file), sourceline(1) returns it verbatim, and `#!` on line 2 is error 13.1.
   So it is line-1-only and the scanner is the right place, not ProgramSource --
   which is what the implementer chose. Found by differential testing, not by
   reading: 494 of 790 files under ootest/ and samples/ were error 13.1 line 1.
2. ::RESOURCE bodies are RAW TEXT and must not be tokenised. Oracle: a body
   containing `'unmatched and /* unclosed` gives rexxc rc 0, and
   package~resources returns it verbatim. Tokenising invents 6.2/6.1. So
   Scanned needed a fourth field beyond the brief's three; not scope creep.

THREE CONCERNS STILL TO ADJUDICATE (handed to the reviewer):
- Eager scan can report a LATER scan error where the oracle reports an EARLIER
  parse error. Measured: `say )` line 1 + `'unclosed` line 3 gives oracle
  37.2/line 1 vs ours 6.2/line 3. Once in 12,059 files. Note this is a different
  NUMBER, not just a different line, so the relaxed "number and sub-number on a
  plausible line" gate does not automatically cover it.
- ParseCtx/TokenCursor are pub, not the pub(crate) the brief specifies, because
  with no in-crate caller pub(crate) trips dead_code under -D warnings. Narrowing
  is probably owed to Task 3.5.
- ParseError.byte is the CLAUSE START, not the offending character. Measured:
  6.1/6.2/13.1/15.3 all report the clause's line. Consistent with error 36's main
  message reporting the clause line while its substitution reports the token's.

MEASUREMENT THAT CORRECTS MINE: the symbol repetition ratio is 15.4x for
CoreClasses.orx (8,118 occurrences over 526 distinct) and 7.8x for
StreamClasses.orx. My four crude counts gave 10.4x to 16.4x, so the order of
magnitude held. Also: PlatformObjects.orx is ONE COMMENT LINE on this platform
and produces no tokens, so it cannot have been one of the files my estimate came
from.

HAZARD, worth never repeating: `.Package~new` on a file inside this repo EXECUTES
that file's prolog. The implementer did it to read resources and a repo file's
prolog wrote two untracked .sh files into support/portable/. It removed them and
re-measured on a scratchpad copy; tree verified clean. Copy to the scratchpad
before instantiating a Package. The Task 3.3 review dispatch carries this warning.

=== TASK 3.3 FIX ROUNDS 1 AND 2 ===
Round 1: cf5f9852. Review verdict was spec PASS / quality PASS with 2 Important
and 8 Minor; the re-review confirmed 12 of 12 genuinely fixed but found round 1
had INTRODUCED a new Important. Fifth consecutive fix round on this project to do
that, controller-authored rounds included.
  IMP-1 the #! skip had to be suppressed under INTERPRET. Verified by me:
        interpret "#! nothing here" is rc 13, the same text as a file's line 1 is
        accepted.
  IMP-2 a token span could not be turned back into bytes from outside the crate,
        and tests/scanner.rs:436 only passed by luck -- it indexed a LINE-RELATIVE
        slice with an ABSOLUTE span, which works for 0..3 and would panic for
        spans[1] = 12..15 on a 7-byte line 1. Now goes through span_bytes and
        covers the two occurrences that are not on line 1.

Round 2: 9d263b22. The new Important, and it was a design fault rather than a
missing case: ALL THREE of ProgramSource::new's behaviours are program-only, and
round 1's ScanMode could only reach the third, because CR/LF splitting and Ctrl-Z
truncation happen in new() before any mode exists. Partly my fault -- I accepted
ScanMode-as-parameter without asking what else new() does unconditionally.
Fixed by moving the distinction onto the source: ScanMode deleted, SourceKind
{ Program, Interpret } is new()'s second parameter, scan reads source.kind(), and
scan(&ProgramSource) is back to the brief's signature. It is now impossible to
build a source one way and scan it the other.
Under Interpret, new() does essentially nothing: one line spanning the whole text,
so the offending bytes stay on that line and characterTable rejects them like any
other invalid character. No scanner special-casing at all.

ORACLE TABLE, all measured by me, and the last two rows are the guard against
over-correcting:
  #! under INTERPRET            13.1 on '23'X   (accepted as a file's line 1)
  raw 0a / 0d / 0d0a            13.1, CRLF case names '0D'X
  trailing 0a                   13.1, so not a terminator either
  1a anywhere, including last   13.1, so Ctrl-Z is NOT truncation here
  1a inside a literal           survives as data: prints 1A
  ; between clauses             still separates
  interpret ""                  accepted, as ONE EMPTY LINE (an empty program has none)

Verified after round 2: 61 crate tests, 237 workspace, 0 failures, clippy clean,
fmt clean, ScanMode gone from the crate.
Round-2 re-review dispatched over review-round2-code.diff, scoped to the code
commit only (4fb7e079..9d263b22, since my plan commit sits between the two code
commits and would otherwise have been in the package).

PLAN COMMITS: 4fb7e079 recorded round 1's interfaces, then cdda308e had to correct
it one commit later because round 2 deleted ScanMode. cdda308e also fixed Task
3.2's constructor signature and its three test examples, which 4fb7e079 had left
stale -- the same correction-lands-in-one-task shape this plan keeps producing.

FIXTURE HAZARD worth keeping: an interpret differential fixture must have NO
trailing newline. The oracle side reads it with linein, which strips; ours reads
raw bytes, where a trailing 0a is now correctly 13.1. The implementer's own
fixtures had this bug and it surfaced as a false failure; it confirmed the scanner
was right by measuring interpret "say 1" || '0a'x directly.

Task 3.3: complete. Commits 4e549dcf, 121aea8c, 3526a813, 410b2783 (implementation)
+ cf5f9852 (fix round 1) + 9d263b22 (fix round 2). Plan: 4fb7e079, cdda308e.
Round-2 re-review: FIX CONFIRMED, TASK COMPLETE. All five checks clean, no new
defects -- the first round in six that did not break something.
Independently probed by the reviewer against raw byte files rather than the
crate's own fixtures: over-correction clean (; still separates, interpret "" still
accepted), Program path clean (shebang line 1 skip, line 2 error, CRLF collapse,
LF-then-CR empty line, mid-line Ctrl-Z truncation all matching), the fixture bug
pinned TOWARD correctness ("say 1\n" asserted Err((13,1))), all four accessors
total under Interpret (line_count 1 even for empty text, line_of 1 for every offset
including usize::MAX), and no Default impl for SourceKind so the kind cannot be
silently omitted.
Final state: 61 crate tests, 237 workspace, 0 failures, clippy clean, fmt clean.

ONE REPORTED MISMATCH, CLOSED as expected rather than left dangling: a 300-file
spot-check of corpus-l1 flagged LINES_test_stdin_count.rex, where the oracle gives
35.1 "Incorrect expression detected at *" on line 17 and our scanner accepts.
Verified: Error_Invalid_expression is raised in LanguageParser.cpp,
DirectiveParser.cpp and InstructionParser.cpp and NEVER in Scanner.cpp, so a
scanner that accepts it is correct. Task 3.5 covers it. Not a scanner gap.

=== NEXT: Task 3.4 (clause splitting) ===
Regenerate the brief -- 3.4 changed under b00110e1 (rule 4's two-adjustment model,
span-not-derivable-from-tokens), 8188cacf (what "end of line" and "end of file"
mean now) and cdda308e.
3.4 owns rules 1, 2 and 3 only. Rule 4 (THEN/ELSE/OTHERWISE ending a clause
mid-line) is Task 3.6's, via split_before's TWO byte positions.
The span semantics are where this plan has been wrong most often, so this is worth
opus rather than sonnet despite being smaller than 3.3.

Task 3.4: complete. Commits 5eec1fcb (implementation) + 3b3a0f51 (Minor fixes).
Plan: cc-committed with rule 3, 47.1, 3.9's join, and the pub(crate) contradiction.
Review: spec PASS, quality PASS, 0 Critical, 0 Important, 2 Minor -- both fixed
rather than deferred. Final: 79 crate tests, 255 workspace, 0 failures, clippy
clean, fmt clean.

STRONGEST VERIFICATION OF ANY TASK SO FAR, on both sides. The implementer proved
spans byte-identical to trace r across a 42-clause file (diff empty, 42/42), held
structural invariants over 1,039,513 clauses in 12,986 files, and MUTATION-TESTED
its own implementation three ways, each mutation failing 5-6 of 17 tests. The
reviewer then re-ran all three mutations against the committed file and got exact
matches (5/17, 6/17, 6/17), rather than trusting the numbers, and added CRLF,
bare-CR, \n\r, `a:b` and `a: b:` probes the implementer had not run. That is the
first time a task here answered "would these tests catch anything" instead of
asserting it.

THREE ORACLE FINDINGS, all confirmed by me:
- Rule 3 was WRONG in the plan. A label's colon ends the clause UNCONDITIONALLY,
  not "when tokens follow": here: ; nop traces `here:` then `nop`, so the label
  span stops at the colon even when the ; is the real terminator, and that ;
  belongs to NO clause. Contrast nop; say "x" which traces `nop;`. Rules 2 and 3
  cannot share one mechanism.
- Error 47.1 exists: interpret "here: nop" is rc 47, `found "HERE"`. Owed to 3.6,
  reachable via ParseCtx::source + SourceKind with no signature change.
- A continued clause's span CONTAINS the terminator (span 0..12 for say 1, / + 2)
  while trace r drops it, so 3.9 needs a terminator-stripping join.

MY OWN ERROR this round, harmless: I ran `cargo fmt -p rexx-parse -- --check` from
the repo root, where there is no Cargo.toml, read the non-zero exit as "formatting
dirty", and nearly reported it. It exits 0 from rust/. Run cargo from the manifest
directory.

=== NEXT: Task 3.5 (expression grammar) ===
Regenerate the brief. 3.5 carries the FIRST in-crate caller, so it owns the owed
narrowing: pub(crate) for ParseCtx, TokenCursor, Clause and ClauseCursor, plus
moving tests out of tests/tokens.rs, tests/clause.rs and tests/scanner.rs into
#[cfg(test)] modules. Narrowing without moving the tests does not compile.
D10 landed on hand-written recursive descent, so 3.5 writes it by hand; chumsky
must not become a dependency.
Also relevant to 3.5: LINES_test_stdin_count.rex is the corpus file whose 35.1
"Incorrect expression detected at *" the scanner correctly does not raise.

=== TASK 3.5 DISPATCHED (opus). BASE = b2f32e7c ===
Brief regenerated: task-3.5-brief.md, only 65 lines, and I checked it before
dispatching rather than after.

PLAN DEFECT FOUND PRE-FLIGHT: Task 3.5's brief does NOT define `Expr` at all. It
says "Produces: Expr in ast.rs" and stops, where Task 3.3 got full code for
SymbolTable, Keywords, TokenCursor and ParseError. The central type of the task is
unspecified. It also gives no precedence table (just "LanguageParser.cpp" with no
line) and no enumeration of the expression forms.
Not blocking, because the METHOD in the brief is sound -- differential evaluation
of results with a deliberately bounded throwaway evaluator. So I supplied what it
omitted instead of stalling:

THE PRECEDENCE TABLE, read out of the C++ so the implementer need not hunt.
Authority is RexxToken::precedence() at interpreter/parser/Token.cpp:111:
  8  \ (prefix NOT)
  7  **
  6  * / % //
  5  + - (binary)
  4  abuttal, ||, and the BLANK operator
  3  all 18 comparisons
  2  &
  1  | &&   (OR and XOR)
  0  everything else
PREFIX - AND + DO NOT APPEAR IN THAT TABLE. That is why -2 ** 2 is 4: prefix binds
to the operand before ** is considered, a property of where the unary parse sits
in the grammar, not of a precedence number. Told to verify prefix against both **
and \ since it is the likeliest thing to get wrong.
The C++ is a stack machine popping while precedence() > second->precedence()
(LanguageParser.cpp:2924-2953); we are recursive descent, so levels become nested
functions and the > vs >= distinction is where associativity errors hide.

Also supplied: the 14 in-scope expression forms from interpreter/expression/, and
the two constraints that pin Expr's shape -- D13's arena plus Step 3b's flat
instruction chain (expressions nest, so a tree is right; Box vs index is the
implementer's call and must be justified in a doc comment), and gate criterion 1
property 1, which requires every Expr node's span to CONTAIN its operands' spans.
Told to report BLOCKED rather than invent an Expr shape Phase 4 must live with.

NARROWING DEBT handed to this task, being the first in-crate caller: narrow
ParseCtx, TokenCursor, Clause and split_clauses to pub(crate), moving the tests
that touch them out of tests/tokens.rs and tests/clause.rs into cfg(test) modules.
CORRECTION to my earlier ledger note: only TWO test files are affected, not three.
tests/scanner.rs does not touch any of the four -- checked, not assumed.
Told that if narrowing fights the design once a real caller exists, argue for
leaving them pub rather than making a mechanical change that gets reverted.

Probe warnings carried: abs ('2.5') is ABS 2.5 with a blank, and f(x) vs f (x) is
in this task's own grammar; a = b = c with all-equal operands cannot distinguish
associativity, so use a=2 b=2 c=1 which gives 1.
Standard to match, stated explicitly: Task 3.4 mutation-tested its own
implementation three ways to prove its tests bite.

=== TASK 3.5 IMPLEMENTED, review dispatched ===
Commits 911a8ee3 (grammar + AST + narrowing) and aa05315b (differential).
Plan/doc corrections: 5da394de.
Verified by me: 143 crate tests (up from 79), 319 workspace, 0 failures, clippy
clean, fmt clean, tree clean.
Nine mutations applied by the implementer, eight caught, the ninth prompting a
code change. Review told to spot-check at least three itself, per the standard
Task 3.4's reviewer set.

Expr's shape, settled from the two constraints I supplied: a tree of Box/Vec
children with a span on every node, owned inline by the instruction arena.
Expr::new WIDENS the extent it is given, so a caller computing a span too narrowly
still gets a containing one -- that is how gate criterion 1 property 1 is met by
construction rather than by care. Parentheses create no node and do not widen, so
(a) + b has root span 1..7, matching the C++.

FOUR CORRECTIONS TO DOCUMENTS I OWN, all confirmed by me before recording:
- d10-decision.md said f(,) is a call with two omitted arguments. It passes ZERO.
  arg() in the callee gives 0 for f(,) and 1 for f(1,), because parseArgList
  returns realcount and pops trailing omitted args, while parseFullSubExpression
  returns total, so (1,) IS a two-element array. Same trailing comma, different
  meaning in the two forms.
  HOW THE ERROR WAS MADE, worth more than the fix: the first probe used ~items,
  which counts non-nil elements and cannot distinguish a two-element array with a
  hole from a one-element one. ~size is the instrument that answers it.
  My own re-probe of the array half was ALSO wrong -- I called arg() in a routine
  named arr, measuring argument count rather than array size -- so I confirmed
  only the function-call half and left the array half on the implementer's ~size
  measurement rather than claiming it.
- ParseCtx::symbols is NOT read-only, which Task 3.3's doc comment asserted. A
  literal message name (a~'length', which resolves case-insensitively exactly as
  a~length: both 3) and the bracket form's implicit [] never reach the scanner as
  symbol tokens. ExprKind::Message::name is Box<[u8]> as a result, the one name in
  the tree that is not a SymbolId.
- Task 3.6 owes the empty-expression sub-numbers: 35.918 in an assignment, 35.929
  in an IF. Not knowable inside the expression grammar, so 3.5 emits a placeholder
  35.1 that the interpreter does not use there and 3.6 must replace.
- parse_logical returns Result<Expr>, not Result<Option<Expr>>: `if , 1 = 1 then
  nop` is 35.929, so the oracle raises on an absent FIRST element too, which makes
  requiredLogicalExpression's null check dead code in the C++. Do not port it.

DECISION OWED BEFORE 3.6 -- the narrowing ratchet. Narrowing to pub(crate) costs
allow(dead_code), because the library target compiles with cfg(test) off and the
items are unused until a real caller lands. expect is unusable: the lint fires in
one of the two compilations and not the other, so expect itself warns.
There are 10 allow(dead_code) in the crate and only 4 name the task that deletes
them. The rule I am imposing: every one must name its owner, and a task that
becomes a real caller deletes the ones it satisfies. An allow with no owner is
permanent.
The implementer proposes ONE narrowing pass at the end instead of a per-task
ratchet, since 3.6 and 3.7 will add more before the last clears. The review is
asked to recommend, and I decide before dispatching 3.6.

Also open, cheap: TokenCursor::back has no caller and this grammar will never give
it one -- the parser peeks then consumes, where the C++ consumes then rewinds.
Delete if 3.6 does not use it.

=== TASK 3.5 REVIEW: spec PASS, quality PASS WITH CHANGES ===
0 Critical, 3 Important, 5 Minor. Report task-3.5-review.md. Fix round 1 dispatched.
The reviewer re-applied SIX of the implementer's nine mutations to the committed
code and reverted each, all caught, counts within one of the report, and the
4,240-row corpus statistics matched exactly. Test honesty holds.

I1, confirmed by me with rexxc: super_class_term admits only Variable|DotSymbol
where the C++ isVariableOrDot() (Token.hpp:572,576) also admits STEM and COMPOUND.
  a~b:c.  a~b:c.d  a~b:c.d.e   all rc 0 (fail later at run time, 88.914)
  a~b:1   a~b:.                20.917, and those rejections must STAY
I2: Expr::shape collides on f(_,1) vs f(,1) -- `_` is a legal symbol char -- and on
a~"b c" vs a~b(c). The first collision hides argument counting, which is the exact
property f(,) vs f(1,) exists to pin.
I3: NINE allow(dead_code), four naming no owner in the code.

MY COUNT WAS WRONG TOO, and instructively: I grepped and got 10, because one hit
was a PROSE MENTION of the attribute inside a lib.rs doc comment, not an attribute.
The implementer said 8, I said 10, the truth is 9. Fourth instrument error of the
session and the same shape as the others -- the pattern matched something that
looked like the thing rather than the thing.

RATCHET DECIDED (commit above): keep per-task narrowing. Deferring does not avoid
the test migration, it enlarges it and lands it in the phase's biggest wiring
change, and leaves ParseCtx/TokenCursor publicly reachable for four more tasks.
Owner rule is now MECHANICAL: trailing comment on the attribute line, and gate
criterion 8 asserts every such line matches `Task 3\.\d` with the grep spelled out.
The justification is I3 itself: the report's table said one thing, the code said
another, and only the code ships.

ALSO CHANGED: the empty-expression sub-number is now a PARAMETER, mirroring the
C++'s requiredExpression(terminators, error), rather than a 35.1 placeholder that
Task 3.6 owed a fix for. Removing the wrong number beats scheduling its removal.
So 3.6 no longer owes this -- the earlier ledger entry saying it does is superseded.

Adjudications that asked for a change and went into the fix round: delete
TokenCursor::back (3.6/3.7 are the same peek-then-consume style, so it stays dead);
and the sub-number parameter above.
Accepted as-is: Message::name as Box<[u8]>, with a note that Phase 4 will want its
own method-name intern table; parse_logical returning Result<Expr>, confirmed at
LanguageParser.cpp:4283-4288; f(,) matching the corrected fact; tests/expr.rs being
the Files list's error.

=== TASK 3.5 FIX ROUND 1: 1dd397f7. Re-review dispatched. ===
All 3 Importants and 5 Minors addressed. Verified by me: gate criterion 8's new
grep runs CLEAN (8 attributes, every one carrying `// deleted by Task 3.x`), I1's
gate is now exactly Variable|Stem|Compound|DotSymbol matching isVariableOrDot,
143 crate tests, 319 workspace, 0 failures, clippy clean, fmt clean, tree clean.
Confirmed M3 against rexxc myself: >a~b, <a~b, >a.~b, >a[1], >a~b~c all rc 0.

THE NEW GATE CRITERION CAUGHT SOMETHING ON ITS FIRST USE, which is the useful
part: lib.rs's explanatory comment originally SPELLED OUT the attribute, so the
grep would have flagged it as a ninth ownerless line. The implementer reworded it
to "dead-code allowances" and added a rule that the attribute must not be spelled
out except on an item. I have asked the re-review for an OPINION on that rule
rather than acceptance -- a gate that forbids naming a thing in prose is unusual
and may be brittle, and I would rather hear the argument than inherit it.

WENT ONE STEP PAST INSTRUCTION, with a measured justification I accepted:
parse_expr also takes the terminator SET as a parameter, not just the sub-number,
because requiredExpression's 18 call sites pass FIVE distinct sets (6x TERM_CONTROL,
8x TERM_EOC, 2x TERM_OVER, one each of TERM_EOC|TERM_WITH|TERM_KEYWORD and
TERM_RIGHT). Fixed at end-of-clause, DO's six required expressions would have had
to reimplement the required check. It also checked rather than assumed that all 13
sub-numbers those sites pass are in the 35.9xx block, since the parameter is a bare
u16 with an implicit major 35. The re-review is verifying both counts.

I2's fix went further than quoting: omitted arguments render <omitted>, and names
and literals render through {:?}, quoted AND escaped, because 'a''b' decodes to
a'b which an unescaped '...' cannot render unambiguously either. Re-review told to
attack it with a literal whose text is `<omitted>`, a name containing a double
quote, and one containing a backslash.

Five mutations applied against the FIXED code, all caught, including deliberately
testing BOTH directions of I1 -- the regression and the opposite error of widening
past isVariableOrDot -- because a test that checks only the accepted cases or only
the rejected ones catches one and misses the other.

HARNESS FAILURE MODE, new and worth naming separately from my instrument errors:
the implementer's first I1 probe reused a scratchpad script guarded by
`ls ... || cat > ...`, which left a STALE differently-shaped script in place. It
printed `rc=221 Error 35.918`, a plausible-looking result that meant nothing.
Third time on this project the probe HARNESS rather than the reasoning was the weak
link. Rule now carried in dispatches: write each probe script fresh, never reuse a
path with an existence guard.

Task 3.5: COMPLETE. Commits 911a8ee3, aa05315b (implementation), 1dd397f7 (fix
round 1). Plan/docs: 5da394de, d37c5b6a, and the gate-grep anchor below.
Re-review: ALL FINDINGS ADDRESSED, TASK COMPLETE. All 10 items genuinely fixed
(3 Important, 2 adjudications, 5 Minor), no new defects. Final: 143 crate tests,
319 workspace, 0 failures, clippy clean, fmt clean.

The re-review re-derived rather than trusted, on both items I flagged:
- I2: reproduced the quoted() function outside the crate and attacked it with a
  literal whose text is `<omitted>` (confirmed valid input, rexxc rc 0), a name
  with a double quote, and one with a backslash. No collision, because Rust's
  Debug escaping of valid UTF-8 is injective.
- The terminator-set claim: verified against the C++ itself, 18 call sites of
  requiredExpression (1 in LanguageParser.cpp, 17 in InstructionParser.cpp), five
  distinct sets with exactly the claimed 6/8/1/2/1 split, and all 13 error codes
  inside RexxErrorCodes.h's 35904-35933 range.

TWO CORRECTIONS TO ME, both accepted:
1. I called the terminator parameter "one step past what was asked". It was not.
   Adjudication 7 named requiredExpression(int terminators, RexxErrorCodes error)
   as the signature to mirror, which already has BOTH parameters, so adding both
   is literally what was asked. My framing was wrong, not the implementer's scope.
2. My gate grep was unanchored and therefore matched prose. The prose rule adopted
   to work around that is brittle -- enforced by nobody, silently re-broken by any
   future comment. Anchoring to ^\s*#\[allow\(dead_code\)\] makes it correct
   regardless of prose, so the rule is dropped. Committed.
   FIFTH instrument error of the session, same shape as the other four: a pattern
   matching something that resembled the thing rather than the thing.

RESIDUAL, pre-existing and out of scope, recorded so it is not lost: quoted() uses
from_utf8_lossy, which collapses distinct invalid-UTF-8 byte sequences to the same
U+FFFD text -- quoted(&[0xFF]) == quoted(&[0xFE]). Reachable only through a raw
binary literal, nothing tests it, and the pre-fix code was equally lossy for
Literal. Relevant to D14's byte-string direction if a later task renders literals.

Cosmetic, not filed: impl Terminators's attribute says "deleted by Task 3.7" while
its doc comment two lines above attributes the block to Tasks 3.6 and 3.7 jointly.

=== NEXT: Task 3.6 (the 35 keyword instructions) ===
Biggest remaining task. Regenerate the brief; it changed under b00110e1, 8188cacf,
cdda308e, b2f32e7c, 5da394de, d37c5b6a.
3.6 owes: rule 4 via split_before's TWO byte positions; error 47.1 for a label in
INTERPRET text, via ParseCtx::source + SourceKind; deleting the allow(dead_code)
attributes it satisfies; supplying its own empty-expression sub-numbers (35.918 /
35.929) through parse_expr's parameter.
It does NOT owe a 35.1 placeholder replacement -- that was removed at the source.

=== TASK 3.6 DISPATCHED (opus). BASE = 088ee942 ===
Brief regenerated: 313 lines, and CHECKED before dispatch this time. It is well
specified -- ClauseCursor, split_before, the family tables and the bare-clause
reachability table are all present -- unlike 3.5's, which omitted its central type.
No pre-flight defect found.

Also committed first, 088ee942: lib.rs still stated the prose rule ("do not spell
the attribute out anywhere but on an item") that I had just removed from the plan
when I anchored the gate grep. That is the contradiction-survives-elsewhere shape
again, authored by me one commit earlier, so it is fixed rather than left.

Dispatch carried the exact interfaces, because 3.6 consumes more of them than any
other task: ProgramSource's six methods plus kind(), scan/Scanned, Token/Tag,
ParseCtx, TokenCursor (forward-only -- back() was DELETED in 3.5, do not re-add),
Clause/split_clauses, Terminators with its named sets CONTROL/COND/OVER/IF/
PARSE_WITH, and parse_expr(ctx, cursor, term, missing: u16).

The four owed items, all spelled out with their measurements:
1. Rule 4 via split_before's TWO byte positions. IF/WHEN pass the THEN token's
   START, THEN/ELSE/OTHERWISE pass their own token's END, and the blanks between
   belong to no clause.
2. Error 47.1 at label recognition from ParseCtx::source's SourceKind, mirroring
   isInterpret(). No signature changes needed.
3. Delete the allow(dead_code) attributes it satisfies; the anchored gate grep is
   in the dispatch so it can check itself.
4. Supply 35.918 / 35.929 through parse_expr's `missing` parameter. Explicitly told
   it does NOT owe a 35.1 placeholder replacement -- that was removed at source.

Central hazard stated as the hazard it is, rather than buried among the 35:
keywords are NOT reserved, recognition is positional, and getting it wrong is a
rewrite of the dispatch rather than a patch. Told to run
corpus/lang/keyword_as_variable.rex, which exercises all 35 as variables plus a
stem `end.`, a compound tail `if`, and PARSE while `parse` is a variable.

Test bar stated explicitly with precedent: 3.4 mutation-tested three ways, 3.5 nine
ways, and reviewers re-applied samples and got matching counts. Plus: test BOTH
directions of any gate, since checking only the accepted or only the rejected cases
catches one error and misses its opposite -- which is how 3.5's I1 fix was verified.

Probe discipline carried in full: fresh script path every time (the stale-harness
failure), instruments that can distinguish (the five pattern-matched-the-wrong-thing
errors, named), and no .Package~new inside the repo.

=== TASK 3.6 PRE-FLIGHT: three rulings given ===
The implementer read the brief, all five modules, Task 3.1's report, the ledger and
four C++ files, then asked BEFORE writing. Its Step 1 extraction gives exactly 35
and matches token.rs's INSTRUCTIONS byte for byte.

Q1, WHERE BLOCK STRUCTURE LIVES -- ruled YES to its proposal.
Its archaeology: block wiring is NOT in nextInstruction, it is in translateBlock
(LanguageParser.cpp:1176), and every misplaced-block error is raised there. Two
things resist a stateless parser: nextInstruction DOES raise 8.1 itself for a bare
THEN (which my Step 4 table asserts), and whenNew alone consults
topBlockInstruction().
Ruling: 3.6 owns per-clause parsing plus the rule-4 splits, and ClauseCursor
carries exactly ONE bit -- "the next clause yielded may begin with THEN", set by
IF/WHEN when they split. Verified by me that this buys both errors: `if 1 = 1` /
`nop` is 18.1, and `select` / `when 1 = 1` / `nop` / `end` is 18.2. One bit is the
minimum that keeps the Step 4 table true; less diverges, more is the full stack.

3.7b IS THE NAMED OWNER of everything deferred, and I am lifting it into 3.7b's own
plan text rather than leaving it in a report -- the implementer explicitly warned
about the crack between the two tasks, which is where this phase has lost things
before. Deferred: errors 7.2, 8.2, 9.2, 10.1, 14.3, stack-dependent 18.1/18.2, the
misplaced-label errors, EXPOSE/USE LOCAL must-be-first (99.907/99.910, which read
lastInstruction), and the chain indices.
CONSEQUENCE I TOOK ON: 3.7b was scoped as a small composition task and is now
accumulating translateBlock's whole job. I owe it a re-scope, and a split if it will
not fit.

Q2, SELECT CASE's WHEN -- ruled DIFFERENTLY from its proposal, and from my own first
instinct. Verified: `select case 1` / `when , then nop` is 35.934, plain `select` is
35.929. It proposed recording the divergence. NOT SAFE: the relaxation keeps number
and sub-number EXACT, so this is outside it, and unlike the other deferrals it
cannot be repaired later -- 3.6 raises it DURING WHEN's expression parse, which
aborts before 3.7b ever sees the clause, so "3.7b fixes it" is unavailable.
Ruling: parse WHEN as the logical form always but THREAD the sub-number instead of
hard-coding it, using parse_expr's existing `missing: u16`. Pass 35.929 today; when
the stack arrives the change is one argument at one call site rather than a
re-parse. An unrepairable divergence becomes a one-line change.

Q3, Instruction's shape -- ruled YES. { kind, clause_span }, no `next`, no jump
targets. In a Vec the chain IS index order, and a field nothing sets is worse than
an absent one because it reads as a contract. 3.7b adds them with the stack.

NOTE 2 CONFIRMED, and it is bigger than the implementer framed it. `if 1 = 1;` with
THEN on the next line traces as `if 1 = 1` WITHOUT the semicolon, where `nop;`
traces WITH it. So for IF/WHEN the clause span ends at the START OF WHATEVER TOKEN
ENDS IT -- the THEN token's start on the same line, the end-of-clause token's start
on the next -- not "the end of the terminating token", which is rule 2 for ordinary
clauses. That is the THIRD exception to rule 2 after the label colon, and all three
are the same shape: some bytes belong to no clause. Told to implement the
start-of-terminator rule generally rather than special-casing the same-line form,
with both spellings tested. Owed: record it in the plan.

Note 1 accepted: no TokenCursor::back needed. createLoop's markPosition/
resetPosition is only two-token lookahead, and lookahead over ctx.tokens bounded by
Clause::tokens.end covers every use, so nothing rewinds.

Task 3.6: COMPLETE. Ten family commits 9fa23902..1d06d7c3, fix rounds e72cd1d9 and
d8bf0088. Plan: 87dbee7b, 9a955ce8, 5244debb, 3743c717.
Review: spec PASS, quality PASS with findings, 0 Critical, 3 Important, 8 Minor.
No wrong behaviour in ~120 parser probes against 168 fresh oracle runs.
Final: 167 crate tests, 400 workspace, 0 failures, clippy 0, fmt 0, gate grep empty,
20 mutations all applied and all caught, script exits non-zero on an unapplied one.

THE M1 DISAGREEMENT, and it is the most instructive thing in the task. The reviewer
said 18.1 reports against the offending clause; the implementer measured four cases
and concluded it reports against the IF. BOTH HAD REAL EVIDENCE, because in every
probe either ran the offender sat immediately after the IF, so moving the IF moved
the offender too and both hypotheses fit identically. Blank lines separate them:
  nop / if(2) / blank / blank / nop(5)
    Error 18 running ... line 5: THEN expected.        <- MAIN, the offender
    Error 18.1: IF instruction on line 2 requires ...  <- SUBSTITUTION, the IF
Two independent fields. We gate the MAIN line and do not reproduce substitutions,
so ParseError.byte belongs on the offending clause, and fix round 1 had introduced
a divergence in the one place round 0 had right.

ROOT CAUSE, diagnosed by the implementer and better than "write a better regex":
its harness grepped `on line [0-9]+`. The substitution reads `on line 2`; the main
field reads `line 5:` with NO `on`. The instrument could not match the field the
question turned on, so every probe returned the IF's line and looked consistent.
Generalisation now recorded in memory: A FIELD-EXTRACTING PROBE CAN ONLY CONFIRM THE
HYPOTHESIS THAT CHOSE THE FIELD. A probe about which of two positions is reported
must print the oracle's whole message.

MY SIX ROWS WERE INSUFFICIENT AND IT CAUGHT THAT. Mutation 18b, replacing the stored
byte with clause_span.end, SURVIVED all six, because on a single-line IF the clause's
start and end are the same line. It added two rows with a CONTINUED IF spanning lines
2-3: no offender reports line 2, offender on 4 reports line 4. Both verified by me.
That pins the byte to the clause's START rather than merely to somewhere inside it.

Two false test comments it caught in its own work and rated worse than the missing
tests, correctly: `guard on when 1` is 99.913 not rc 0, and select/end 1 is 10.7 not
10.3 because the number depends on what the END failed to close.
M2 earned itself on its first hardened run: mutation 5's pattern had gone stale from
this very round's expect_then signature change, so a SECOND never-applied mutation
was hiding behind the same weakness. A mutation script that silently matches nothing
is a mutation never applied, and that is the one failure mode mutation testing
cannot self-detect.

=== NEXT: Task 3.7 (directives) ===
114 lines, nine directives. Regenerate the brief.
3.7 consumes Scanned::resources, keyed by the `::` token index, with one integer
comparison. Its resolution goes through ParseCtx::keywords.directives and
.sub_directives -- SymbolId comparisons, NOT a sorted string table.
Three allow(dead_code) remain: one naming 3.7, two naming 3.7b. 3.7 deletes its own.

=== TASK 3.7 REVIEW: spec PASS, quality CHANGES REQUIRED ===
1 Critical, 3 Important, 7 Minor. Six commits d8bf0088..c62fcd69, 200 crate lib
tests, 433 workspace, 32 mutations all applied and caught.
The reviewer re-applied five mutations getting exact counts, and CONFIRMED THE
UNAPPLIED-PATTERN GUARD WORKS by planting a bogus pattern: it printed
"NOT APPLIED: pattern found 0 times" and exited 1. That guard is worth more than
the mutations it protects.

C1, AND IT IS THE MOST IMPORTANT FINDING OF THE PHASE SO FAR, for process reasons
rather than code. scan_number rejects a blank between the sign and the digits, which
parseNumber's NUMBER_SIGN_WHITESPACE state allows. Measured by me:
  trace "+ 9"             rc 0   (ours 24.1)
  trace "- 9"             rc 0   (ours 24.1)
  ::options digits "+ 9"  rc 0   (ours 26.5)
  ::options digits "- 9"  26.5   -- but ONLY because -9 < 1
THAT LAST ROW IS WHY THE TEST PINNED THE WRONG RULE. convert/tests.rs:58 used the
one input whose failure comes from the range check rather than the sign-blank rule,
so the probe could not see the field the question turned on. Seventh instance of the
shape, and mutation 30 does not catch it either.

THE PART THAT MATTERS: rexx-num ALREADY IMPLEMENTS THIS RULE, and lib.rs:389 says so
in as many words -- "Blanks are allowed between the sign and the digits". It was one
of Phase 2's eight escaped defects, and the signblank case set, 2,320 cases, exists
specifically for it. Phase 3 reintroduced the class in a different function in a
different crate because nothing carried the knowledge across the crate boundary.
So the fix round asks a design question rather than for a patch: why does rexx-parse
have its own number-acceptance rule at all, when rexx-num::Number::parse is verified
across 128,368 cases? A scanner must CLASSIFY without converting, which is a
different job, but the ACCEPTANCE rule should be one rule in one place or at minimum
differentially tested against Number::parse so the two cannot drift again.

I1: error 99.925 missing at four getRetriever call sites and on NO deferred list.
Verified with controls: ::attribute 3, ::attribute .a and ::method m delegate 5 are
99.925 and accepted here; ::attribute a. and ::method 3 are rc 0. Purely local, so
"needs the accumulated package" does not cover it. The syntaxError lives in
LanguageParser.cpp, OUTSIDE the 2,867 lines the task was scoped to -- which is how it
was missed, and the next task scoped to one file has the same exposure. Worth
carrying into future dispatches: scoping a task to a file scopes its blind spot too.

I2: is_number has no exponent-magnitude limit. Verified with controls:
-1e1000000000 and -99e999999999 are 19.916 and accepted here; -9e999999999 and
-1e999999999 are rc 0. The boundary is the exponent's DIGIT COUNT, not nine-nines.

I3: the_other_shipped_packages_parse asserts only !is_empty() while its doc comment
claims 7/139/5/2. Counts right, unasserted, and the report overstated the test.

Also told to retire two hedges rather than carry them, both answerable from the C++
and both resolving in the implementer's favour: WordIterator splits on space and tab
only, and Utilities::toUpper is ASCII-only. An unresolved hedge in a doc comment
reads as a known gap forever.

Task 3.7: COMPLETE. Commits f35b83f6..c62fcd69 (six), review round cd12a305,
f37b4a86, c0e42d39, plus 05d40f1d moving numberValue into rexx-num.
Plan/docs: d4d12403, bc7dc119.
Final: 204 crate tests, 437 workspace, 139 in rexx-num, 47 mutations all applied and
all caught, clippy 0, fmt 0, gate grep empty. Three allowances remain, all naming 3.7b.

THE NUMBER-ACCEPTANCE ANSWER, and it is the structural fix rather than a patch:
rexx-parse no longer owns the rule. is_number and whole_number's first step call
rexx_num::Number::parse, and rexx-num is a real dependency now. Its three reasons,
the second of which is better than the question I asked:
  1. it mirrors the C++ layering -- LanguageParser calls RexxString::numberString()
     rather than implementing number syntax, so the dependency edge is the
     interpreter's own;
  2. one rule cannot drift from itself, and a drift test would only have DATED the
     reintroduction rather than prevented it;
  3. Number::parse has 128,368 differential cases behind it.
That fixed I2 for free -- both exponent limits came with the delegation.

TWELVE SETS AT 0, AND I VERIFIED THEM MYSELF rather than reading the table. Ran the
runner independently: signblank 2320 cases 0 divergences, TOTAL 128368 0, and I
checked both output files held 2320 real lines rather than two empty files diffing
clean, which is the one hole in the runner's construction (it counts diff output
without asserting either side produced anything). First oracle row is `3|+ 3|+|0=3`,
the exact sign-blank shape C1 was about, so the set that exists for that class is
green after the delegation.
IT RAN THE SETS TWICE, before and after touching rexx-num, which is better than the
post-change run I asked for: without the first run, twelve zeros are equally
consistent with a harness that cannot fail.

I WITHDREW AN INSTRUCTION. I asked it to keep the one-directional decompose/parse
invariant; it deleted decompose and argued the invariant away. Correct: decompose
existed only to re-walk text for fields Number already holds, so once numberValue is
a method there is nothing to walk, and the asymmetry was a mitigation for a
duplication it had just removed. Keeping it would mean keeping the duplication to
justify the mitigation. convert.rs now holds no number syntax and no number
arithmetic.

C1 UNCOVERED MORE THAN THE REVIEW REACHED: requestNumber ROUNDS to the precision
before asking whether the result is an integer, so a fraction survives it. Verified:
trace 999999999.4, trace "1.0000000001" and trace "0.9999999999" are rc 0, all
previously rejected here; controls trace "999999999.6" (carry makes ten digits) and
trace "99999999.6" (nine digits, nothing rounds) are 24.1.

SEVENTH PROBE ERROR, AND THE FIRST IN THE DANGEROUS DIRECTION: the implementer
asserted the carry rule from reasoning rather than the oracle and was wrong --
claimed whole("0.99999999989", 9) is None, it is Some(1). Verified: 0.99999999989 and
0.99999999999 rc 0, 0.99999999899 and 0.4999999999 are 24.1. The rule compares the
NINE KEPT digits against nine, so the tenth only decides whether a carry happens. A
wrong assertion that happens to pass locks in a false rule, which is worse than one
that fails.

MUTATION RUNNER BUG, worth more than the mutations: it ran two crates but returned
the FIRST `test result:` line, so a mutation caught only in the second read as a
SURVIVOR -- a silent false negative in the tool whose whole job is detecting false
negatives. Fixed to scan every line. Four patterns went stale in the move and the
guard caught all four, on top of the two in 3.6.

UPSTREAM DEFECT, WRITE-UP AUTHORISED, FILING NOT. numberValue's carry-only return is
`carry ? 1 : 0` with no `* numberSign`, where every other return path multiplies by
it and no comment marks it deliberate. Latent, not live. I established WHY it is
unobservable rather than merely resistant: trace -1, trace 1 and
trace "-0.9999999999" all reach runtime 24.901, "Numeric TRACE requests are valid
only from interactive debugging". Write-up going to
upstream-numbervalue-carry-sign.md. FILING IS MORITZ'S CALL -- outward-facing, and I
will surface it rather than act.

=== NEXT: Task 3.7b (the public entry point) ===
139 lines, unchanged scope after the 3.7c split. Deletes the three remaining
allowances if it becomes their caller.

=== TASK 3.7b DISPATCHED (sonnet). BASE = 05d40f1d ===
Brief 139 lines, extracted cleanly (one task heading -- the 3.7/3.7b/3.7c collision
only bites the bare `3.7` pattern). Composition task, no new grammar, so sonnet.

Dispatch carried the three things easiest to get wrong:
1. The borrow order, and WHY it compiles: every surviving span is a BYTE range, so no
   Instruction or Expr may hold a token index. If one does the composition will not
   compile, which is the correct outcome rather than something to work around.
2. Program::labels keyed by the label token's VALUE -- upcased for a symbol, VERBATIM
   for a literal. With 'MiXeD': present, signal value 'MiXeD' reaches it while
   signal value 'MIXED' and signal MiXeD are both 16.1. Interning the key is wrong in
   BOTH directions and nothing in this phase's gate would catch it, because 16.1 is
   raised at run time.
3. A Fragment's SymbolIds are not comparable with the enclosing Program's.

Housekeeping: all three remaining allow(dead_code) name 3.7b, one in directive.rs and
two in instruction.rs. Told to delete all three, and that if any turns out NOT to be
reachable from the entry point that is a FINDING to report rather than a reason to
leave the attribute.

Scope fence stated explicitly, since 3.7c does not exist in code yet: block structure,
the control stack, chain and jump indices, END matching, the 13 translateBlock errors
(7.1, 7.2, 8.2, 9.2, 10.1, 10.7, 14.3, stack-dependent 18.1/18.2, misplaced-label),
EXPOSE/USE LOCAL must-be-first, the per-body exposed-variable table behind 99.913, and
SELECT CASE's WHEN node shape. So 3.7b's Program accepts every valid program and does
NOT yet reject invalid block structure -- expected, and gate criterion 4 already
records that it cannot be met until 3.7c. Told not to add fields nothing sets, and to
stop and ask rather than grow a stack.

Test bar carried with its evidence: 3.7 shipped 47 mutations all applied and caught;
the unapplied-pattern guard has now caught SIX never-applied patterns across two
tasks; and 3.7's runner was found returning only the first `test result:` line while
testing two crates, so a mutation caught in the second read as a survivor.

Task 3.7: FULLY CLOSED at 8a98ee40. Eleven commits, f35b83f6..8a98ee40.
437 workspace tests, 47 mutations all applied and caught, clippy 0, fmt 0, gate grep
empty, tree clean.

WRITING THE RULE IN PROSE CAUGHT A SECOND ERROR IN THE SAME PLACE, and this is the
transferable lesson. I asked for the carry rule to be stated in words rather than only
in values. Doing so exposed that the comment's EXPLANATION was wrong even though its
values were right: it said 0.4999999999 is rejected "because nothing carries", but the
carry DOES happen there -- the dropped digit is a 9 -- and the rejection is because the
first KEPT digit is 4 rather than 9. Right values, wrong reason, and it had survived a
review round immediately after I wrote a paragraph about that exact hazard.
The rule as it now reads: the FIRST DROPPED digit decides only whether there is a
carry; the KEPT digits then decide wholeness, against 0 normally and against 9 when the
carry set, because only an all-nines tail can absorb the +1 and leave zeros.
Verified by me, and the decisive pair is identical kept digits with different dropped
digits: 0.9999999994 is 24.1, 0.99999999999 is rc 0. Plus 1.0000000004 rc 0 for the
no-carry-reaching-whole branch. Six rows.

RUNNER HOLE CLOSED WITH A NEGATIVE CONTROL, which is the part that matters. I noted
that the runner counted diff output without asserting either side produced anything, so
two empty files would diff clean. It now prints oracle and Rust line counts beside the
case count, asserts all three equal and non-zero, and reports broken_sets separately.
Then it CONTROLLED THE GUARD by pointing at a generator emitting nothing: broken_sets=12,
exit 1, where the old runner would have printed twelve zeros and exited 0. A guard
nobody has seen fire is untested. Corrected runner reproduced in the report so it
outlives the scratchpad.

THE UPSTREAM WRITE-UP WIDENED THE FINDING FROM ONE SITE TO TWO, and the evidence is the
asymmetry rather than the code. Four carry-only return branches in
NumberStringClass.cpp, all literally `carry ? 1 : 0`:
  595  numberValue           signed,   does not reject negatives  -> DEFECT
  679  unsignedNumberValue   unsigned, rejects at 650             -> correct
  1083 int64Value            signed,   does not reject negatives  -> DEFECT
  1181 unsignedInt64Value    unsigned, rejects at 1150            -> correct
Verified all four lines and both isNegative() guards myself. The same expression is
right in two functions and wrong in two, which is the signature of a line copied
between them rather than a decision taken four times. Every other return path in both
signed functions multiplies by numberSign; only this branch does not; and nothing marks
it deliberate in a file that explains its other numeric edge cases at length.
Write-up at upstream-numbervalue-carry-sign.md, NOT FILED. It states in its first three
lines that it is not a reproducible report, and that the missing work is auditing the
other callers of requestNumber and int64Value for one that both reaches the branch and
exposes its result. FILING IS MORITZ'S CALL -- outward-facing.

=== TASK 3.7b IMPLEMENTED, review dispatched ===
Commit 5aeb3255. Plan fix d2cc74f8 (labels key type + first-duplicate-wins).
Verified by me: 276 rexx-parse tests, 460 workspace, 0 failures, clippy 0, fmt 0,
tree clean, and ZERO allow(dead_code) remain in the crate.

THE NARROWING RATCHET FULLY CLEARED AT 3.7b, NOT 3.11. That vindicates the decision I
took after Task 3.5, against the implementer's suggestion of one pass at the end: the
per-task route was predicted to accumulate attributes through 3.6 and 3.7 and clear
only at 3.11. It peaked at ten, and 3.7b removed the last three. Deferring would have
enlarged the test migration for no benefit, exactly as the review that recommended
per-task argued. Worth remembering when a ratchet looks like it is growing without
bound: count the actual owners rather than extrapolating.

IT TOOK THE SINGLE-CURSOR DESIGN I ASKED ABOUT rather than defending two, and the cost
was smaller than either of us assumed: parse_instructions had exactly ONE
non-definition call site (the parse_kind helper every instruction-grammar test funnels
through), so changing it to accept a caller-supplied cursor touched one function plus
lib.rs, not the 200-plus tests behind it. No second split_clauses, no fast-forward, no
independent-splits-agree assumption.

ALL THREE ALLOWANCES WERE REACHABLE -- no finding there. Worth noting because I framed
an unreachable one as a discovery worth reporting, and the answer was that three tasks'
assumptions were all correct.

MUTATION TESTING FOUND A COVERAGE HOLE, which is its actual purpose rather than a
by-product. Its first pass was 10 of 10 caught, but mutating directive_has_body's
Method arm to always-false SURVIVED: no test exercised ::METHOD's or ::ATTRIBUTE's body
flag at all, only ::ROUTINE's. Two tests added, re-run at 12/12 with a clean revert
confirmed by diff. A 10-for-10 pass is not evidence of coverage; a survivor is
evidence of a gap.

Also confirmed by it directly, closing a claim it had only spot-checked: the
bodiless-directive fallthrough to 99.916 from the NEXT parse_directive's own ::-check
holds for all five of ::CLASS, ::OPTIONS, ::REQUIRES, ::ANNOTATE and ::RESOURCE, not
just the one it first tried.

Task 3.7b: COMPLETE. Commit 5aeb3255, plan d2cc74f8, Minor fix 9dddc404.
Review: spec PASS, quality PASS, 0 Critical, 0 Important, 1 Minor.
Final: 276 rexx-parse tests, 460 workspace, 0 failures, clippy 0, fmt 0, zero
allow(dead_code) anywhere in the crate.

The reviewer re-applied three mutations, all caught with exact counts, and verified
four things beyond them: the borrow-order invariant by reading ast.rs and
Resource::lines rather than only Instruction/Expr; the 99.914 byte position with two
probes, confirming it points at the first ::-clause (byte 7 for "say 1; ::routine r")
and STAYS at byte 7 with two directive-shaped clauses present, which is what proves the
check fires once rather than per-directive; four of the five bodiless-directive cases
plus a ::CONSTANT counter-case; and the one-call-site blast radius by grep.

MINOR FIXED RATHER THAN DEFERRED, because it was cheap and blocks everyone downstream:
Program, Fragment and ProgramSource derived nothing, not even Debug. Pre-existing,
traced to ProgramSource, and invisible to the task's own tests because they never print
these types -- but the reviewer hit it the moment it wrote a throwaway probe, and Phase
4 would hit it constantly.
Program and Fragment now derive Debug. ProgramSource got a HAND-WRITTEN Debug reporting
kind, line count and byte length rather than its text: a derived one would dump the
whole program into every failing assertion mentioning a ProgramSource, which is what
would have made it undebuggable in practice rather than merely underived. That
distinction is the reason not to reach for derive reflexively on a buffer type.

=== NEXT: Task 3.7c (block structure and the control stack) ===
The largest remaining piece: translateBlock is 509 C++ lines raising 13 distinct
errors. Regenerate the brief -- note the extractor's `3.7` pattern matches 3.7, 3.7b
and 3.7c alike, so extract with the literal `3.7c` or trim by hand.
3.7c owes, all recorded in the plan: the control stack; the 13 errors including 7.1 and
10.7; END matching with a number or stem as a legal block name; EXPOSE/USE LOCAL
must-be-first via lastInstruction; the per-body exposed-variable table behind 99.913,
which is NOT a translateBlock error and NOT method-specific; the chain and jump indices,
which is where Instruction finally gains next; and SELECT CASE's WHEN needing BOTH the
35.934 literal at instruction.rs:2538 and the case-value-list node shape.
Gate criterion 4 becomes satisfiable only when this lands.
It also inherits the file-scoping warning: translateBlock is in LanguageParser.cpp but
the errors it raises and the state it reads reach into InstructionParser.cpp and the
instruction classes, so enumerate the call-outs before implementing.

=== TASK 3.7c DISPATCHED (opus). BASE = 9dddc404 ===
Brief 108 lines, extracted cleanly with the literal `3.7c`. Largest remaining piece:
translateBlock is 509 C++ lines raising 13 distinct errors.

Dispatch carried, in this order of emphasis:
1. THE FILE-SCOPING WARNING FIRST, because it cost Task 3.7 a defect. 3.7 was scoped to
   DirectiveParser.cpp and missed 99.925 entirely, since that syntaxError lives in
   LanguageParser.cpp. 3.7c has the same exposure and worse: translateBlock is in
   LanguageParser.cpp but the errors it raises and the state it reads reach into
   InstructionParser.cpp and the instruction classes. Told to ENUMERATE the call-outs
   before implementing and put the enumeration in the report.
2. All 13 errors by symbolic name, with the instruction to capture number and
   sub-number from rexxc BEFORE implementing, raw output in the report, and an explicit
   ban on inferring a sub-number from a symbolic name. Calibration given: select/end is
   7.1 and select/nop/end is 7.2 -- different errors, and the names do not say which.
3. Every owed item with its measurement: END matching (number or stem legal, 10.3 under
   DO vs 10.7 under SELECT, and 3.6's first test asserted 20.909 and was wrong);
   EXPOSE/USE LOCAL must-be-first via lastInstruction; 99.913 with all three rows
   proving it is neither method-specific nor a translateBlock error; the chain and jump
   indices, which Instruction gains HERE with ast.rs:470 documenting why they were
   omitted until now; and SELECT CASE's WHEN needing BOTH the 929 literal at
   instruction.rs:2538 and the case-value-list node shape.
4. What 3.6 already does, so it is not redone: the one-bit THEN flag buying 8.1 and the
   non-stack parts of 18.1/18.2, and the settled fact that 18.1's REPORTED line is the
   offending clause's while the IF's line is a substitution we do not reproduce -- with
   a mutation guarding both directions. Told not to disturb it.
5. The probe warning specialised to this task: it will be measuring 13 block errors
   whose triggers are adjacent, which is exactly the shape where two probes agree and
   both are uninformative. Told that blank lines are what discriminated the IF case and
   to choose inputs that separate the variables.
6. Test bar with its evidence: 3.7's 47 and 3.7b's 12 mutations, the guard's six
   never-applied catches across three tasks, and that 3.7b's 10-for-10 first pass still
   hid a coverage hole -- a survivor is evidence of a gap, a clean pass is not evidence
   of coverage.

Task 3.7c: COMPLETE. Commits 1e51ada7, c043400a, f1d30e2d, 964aefd5, then 5cb6cca7
(I1) and 42d1389a (five Minors). Plan: eec97292, 3cd52b9a.
Final: 317 crate, 501 workspace, 0 failures, clippy 0, fmt 0, gate grep 0,
90 mutations all applied and all caught.
Review verdicts: spec compliant, quality good with one correctness defect, and
criterion 4 NOT met -- for wording, not substance.

GATE CRITERION 4 WAS A SOUNDNESS CONDITION ONLY, and this is the phase's most useful
finding about my own work. "For every parse-time error the parser raises" is satisfied
VACUOUSLY by a parser that accepts everything, having raised no errors to check. The
property I meant -- the parser rejects what the oracle rejects -- is COMPLETENESS and
was never stated. It also quantified over an unenumerated set and defined "parse-time"
as "rexxc rejects it", which mis-classifies 98.9xx load failures as parse errors (two
real inputs, ::ROUTINE ... EXTERNAL "LIBRARY x" and the ::method spelling).
Phase 2 failed three of five criteria by writing them against a capability the phase
never had -- visibly unmeetable. This one was satisfiable without meaning anything,
which is harder to notice and I would have ticked it.
Rewritten in 3cd52b9a with soundness AND completeness over a named reproducible
corpus, both deviations enumerated, and "parse-time" redefined as rexxc rejects it AND
the failure is a translation error.

I1 PORTED, no deviation needed, and the implementer found the axis that made it
implementable. Verified by me:
  NAME slots do not feed the cache -- signal a.1, do label a.1/end a.1 stay LEGAL
  VARIABLE slots do -- drop a.1, drop (a.1), parse var a.1 x are all 99.913
That killed the cheap token-scan implementation, because `end a.1` and `drop a.1`
spell the symbol identically. for_each_variable_name encodes the split with an
exhaustive match so a new VARIANT fails to compile.
The simplification that avoided threading the cache through expr.rs: intra-clause
ordering does not matter, since guard on when a.1 & a.1 is rc 0 (verified). So the
requirement reduces to a per-body SET of names from instructions ALREADY in the chain,
filled by add_clause, and nothing outside block.rs changed signature.

SECOND UPSTREAM DEFECT, and the evidence is that the code contradicts its own comment.
addSimpleVariable (:2069) and addStem (:2106) both carry "we need to always perform the
capturing test because we allow e. g. USE LOCAL; GUARD ON/OFF WHEN v > 0" -- verified
both lines exist -- and addCompound's early return on a cache hit defeats exactly the
case that comment names. Stronger than the carry-sign finding, which rested on
asymmetry between four branches. Not filed; surface to Moritz with the first.

M3 WAS BEHAVIOURAL, NOT COSMETIC, and the reason it hid is instructive: 18.1/18.2
reported the IF's line where the oracle reports the TERMINATING DIRECTIVE's, and at
plain end of file the two coincide -- which is exactly why the original tests passed.
Both directions now have a mutation.

RESIDUAL FORWARD HAZARD, flagged honestly by the implementer and worth carrying:
for_each_variable_name now encodes "which slots are variables" in a SECOND place
alongside the parser that builds them. Exhaustive matching catches a new variant but
NOT a new field on an existing variant -- if a later task adds an expression slot to
Raise or Address, the guard check quietly stops seeing it and nothing fails. The
faithful fix is threading the cache through the construction funnels. A comment says so.

Process: five mutation patterns needed re-anchoring after this round moved their code,
caught by the NOT-APPLIED guard rather than passing silently -- third time that guard
has earned its keep. And one added mutation was an equivalent mutant (the guard check
completes before add_clause runs, so registration order is unobservable by
construction); replaced rather than excused, and the replacement found a real gap
because drop (a.1) had been reasoned about rather than measured. Right by luck.

=== TASK 3.8 DISPATCHED (opus). BASE = f60c7c4b ===
Brief regenerated at 145 lines after I corrected FOUR stale things in it, found by a
pre-flight check before dispatch rather than by the implementer afterwards:
  - the title promised substitutions, which the scope decision dropped
  - Interfaces named ProgramSource::position, which does not exist (it is line_of,
    renamed when the column requirement went)
  - the framing asked for substitution values as though they were gated
  - Step 1 defined its own ground-truth collection where criterion 4 now names both
    corpora explicitly
That is the fourth consecutive task where checking the brief first paid off. Told the
implementer to tell me if it finds a fifth.

ONE DESIGN QUESTION HANDED TO IT RATHER THAN LEFT IMPLICIT: ParseError.subs exists and
is never populated -- every construction site passes an empty vector. That is a field
nothing sets, which is exactly what 3.6 and 3.7b were forbidden for next and the jump
targets, so exempting it here would be inconsistent. Fill it where the parser has the
values, or remove it and render without splicing. Not both, and not neither.
Also stated the distinction the scope decision blurred: PRODUCING a message was never
dropped, only differentially TESTING its text. Without that line an implementer could
read "substitutions not gated" as "do not render messages", leaving &1 in user output.

Carried the corpus definition from the rewritten criterion 4 rather than letting 3.8
invent one: soundness over the 385 error-asserting programs extractable by instrumenting
the ok/err helpers, completeness over that plus 301 samples/ and both bootstrap files,
both counts to be reported. Plus the two recorded deviations it must NOT fix and must
let the gate see, and the 98.9xx exception in the completeness direction.

Carried three measured facts that bear on it: the oracle uses at least three line
conventions in adjacent errors (7.1 the SELECT's, 7.2 the offender's, 10.x the END's);
18.1/18.2 report the terminating directive's line, which coincides with the IF's at
plain EOF and is why a wrong test passed for a whole task; and 36.901/36.902 DO
substitute a byte offset, so the old "no column anywhere" claim is false even though we
do not reproduce it.

=== TASK 3.8 REVIEW: pass / good with two doc defects / criterion 4 met IN SUBSTANCE ===
Commits 868749b1, 0e48dc4c, e0f0f3f1. Plan 689400e3.
337 crate, 521 workspace, 0 failures, clippy 0, fmt 0, zero allowances, 106 mutations
all applied and caught, none by a compile error.
0 Critical, 5 Important, 6 Minor. Fix round dispatched; all five Importants closeable
and three of them close criterion 4 as literally worded.

THE REVIEW'S VERIFICATION IS THE BEST IN THE PHASE. It re-measured ALL 1002 corpus rows
against rexxc rather than sampling: 444 rc 0 / 558 rejected, 0 expect mismatches, 0 line
mismatches. 558 of 558 messages byte-identical to one of rexxc's two lines (203 sub, 355
major, 0 neither), and it RECONCILED the implementer's 196+355=551 against its own 203 by
noting all seven added rows are sub-branch. 547 of 547 lines agree. It confirmed the
floor assertion genuinely fires by dropping the multi-line rows.
And it answered the contamination question TOTALLY rather than by sample: every row
reproduces from rexxc today, and rows 378/697 record the oracle's 35.1 rather than our
18.1 -- the one shape that would make the gate self-confirming does not occur.

MY CRITERION 4 HAS NOW FAILED TWO DIFFERENT WAYS, BOTH MINE. The first wording was
VACUOUS -- a soundness condition a parser accepting everything would satisfy. The
rewrite was BLIND -- it named three test files that assert zero scanner-class errors,
so it could not observe the eager-scan deviation it itself enumerates. Both fixed. The
lesson is not "write criteria carefully" but that a criterion needs an adversarial read
asking what would satisfy it WITHOUT delivering the property, which is a different
question from whether it is achievable.

FOURTH DISTINCT MUTATION-HARNESS DEFECT THIS PHASE, and worth naming as a class: G12
was a FALSE SURVIVOR OF A PER-TARGET HARNESS -- scored against the wrong cargo target,
so a mutation that was actually covered read as unreached, and the false conclusion got
baked into the corpus header as a justification for five rows. Previous three: returning
only the first `test result:` line while testing two crates; patterns going stale after
the code moved; equivalent mutants. Every one is the tool that detects false negatives
producing one.

I1 IS THE CARRY-RULE SHAPE AGAIN: the implementer fabricated an oracle message text,
caught it on first run, reported it honestly -- and the fabrication SURVIVES in the
comment three lines above the corrected assertion. Confessing it in the report does not
remove it from the code, and the comment is what the next reader believes.

AND THE IMPLEMENTATION REPEATED THE MISTAKE THE CRITERION WARNS ABOUT: deviation()
classifies the non-translation exception by major 98|90, a CODE PREFIX, where the
criterion says to define it by not being a translation error precisely because a prefix
missed the whole 90.999 class. Told to fix it on principle rather than to satisfy a rule.

Assertion granularity is now demonstrated rather than argued: a shared #[test] hid the
implementer's second fabricated text behind the first, and the reviewer reproduced that
by breaking a second assertion invisibly.

=== PICKUP STATE, 2026-07-29 (compaction point) ===
HEAD dfe495d7, 80 commits on plan/rust-rewrite since 08c8f355.

WORKING TREE IS DIRTY AND 3 TESTS FAIL. THIS IS NOT A REGRESSION. The Task 3.8 fix
round is mid-flight in agent afd7118e9eb87ab2a, with five files modified:
corpus/errors/parse-errors.tsv, error.rs, error/tests.rs, instruction/tests.rs,
tests/errors.rs. 434 passed / 3 failed is that work in progress. Do NOT revert it, do
NOT run `git checkout --`, and wait for the agent's message before judging the tree.
Last committed green state was e0f0f3f1 at 521 passed / 0 failed.

TASK STATUS, 10.5 of 12 done:
  3.1  DONE   D10 = hand-written recursive descent, spike preserved on branch spike/d10
  3.2  DONE   ProgramSource, byte-oriented
  3.3  DONE   scanner, 19 token classes, interning, ParseCtx/TokenCursor
  3.4  DONE   clause splitting, spans byte-identical to trace r over 42 clauses
  3.5  DONE   expression grammar, Expr::new widens so criterion 1 holds by construction
  3.6  DONE   35 keyword instructions, ten family commits
  3.7  DONE   nine directives, number acceptance delegated to rexx-num
  3.7b DONE   public entry point, narrowing ratchet fully cleared
  3.7c DONE   block structure and control stack, 20 errors
  3.8  FIX ROUND IN FLIGHT  error gate, 5 Importants dispatched
  3.9  READY  brief audited and three defects fixed in dfe495d7
  3.10 READY  brief audited, one defect fixed; paths and criterion 0.8.2 verified

WHEN 3.8'S FIX ROUND RETURNS: verify, then re-review scoped to the fix diff, then close.
Its five Importants were I1 a fabricated oracle message text surviving in a comment,
I2 a false-survivor mutation baked into the corpus header as justification, I3
tests/program.rs's eight error programs missing from the corpus, I4 the eager-scan
deviation having no test, I5 the closeable INTERPRET residual.
Then 3.9, then 3.10.

EXTRACTION HAZARD for 3.10: it is the last task, so `task-brief 3.10` pulls the Exit
gate and Notes too -- 341 lines for 38 lines of task. Trim to the `## Exit gate`
boundary. Same class as the 3.7/3.7b/3.7c collision, where the bare `3.7` pattern
matches all three.

=== 2026-07-29, Task 3.8 fix round returned ===
Three commits: fec2b92d (I1 + two doc defects), 01dc673b (class column + I3/I4/I5),
c85a43f1 (a tenth finding the implementer found itself while checking I3).
Verified independently, not taken on report: tree clean, 549 passed / 0 failed over the
workspace with --no-fail-fast, HEAD c85a43f1.

I1's replacement comment was checked against the oracle rather than read. `nop` then
`else nop` under rexxc prints `Error 8 ... Unexpected THEN or ELSE.` and
`Error 8.2:  ELSE has no corresponding THEN clause.`, byte-for-byte what
src/error/tests.rs:26-36 now claims. The point of I1 was a fabricated oracle line
surviving in a comment, so a comment that merely LOOKS measured is worth nothing.

TWO THINGS THE IMPLEMENTER FOUND ITSELF, both worth keeping:
* "Every test file" was still unmet after the file list was widened -- src/expr/tests.rs
  had seven error() assertions outside the transcribed table, gating nothing for 20.917
  and 20.930. The fix replaces the list with a DIFF: every pair any test file asserts
  against every pair the gate covers, residue empty. A list of files goes stale
  silently; the diff is now the artefact. Third attempt at this clause, first one that
  cannot rot.
* FIFTH MUTATION-HARNESS DEFECT: cargo stops at the first failing binary, so re-running
  a survivor against the whole crate attributed J10's catchers to the lib tests only and
  made the new INTERPRET gate test look like a miss. Now --no-fail-fast. Fail-fast can
  hide a catcher but never invent one, so no verdict was wrong, only the attribution.
  Class is unchanged: the tool that finds false negatives produced one.

Re-review dispatched (fable, agent a58f451c5d522c878) scoped to dfe495d7..c85a43f1,
told to assume the round broke something and to attack the "every pair any test file
asserts" claim first, since the implementer's own first two attempts at it were
incomplete.

Two claims I have NOT verified and the re-review is asked to: the class field's exact
count of 9, and 202/93 versus the 195/93 I measured. The implementer says 195 counts
98.903 and 90.999, two pairs this parser can never raise, and that 202 = 195 + the seven
INTERPRET-only pairs. Same 93 either way, so nothing downstream turns on it.

BRIEFS PREPARED, both audited, neither dispatched:
  task-3.9-brief.md   167 lines, extracted clean
  task-3.10-brief.md  trimmed 348 -> 45 lines at the `## Exit gate` boundary, per the
                      extraction hazard recorded above
Holding 3.9 until the re-review returns: the reviewer re-derives numbers by running the
suite, and a mid-edit crate that does not compile would make it report false failures.

Task 3.8: complete at c85a43f1.
Re-review (fable, a58f451c5d522c878) returned I1-I5 all ADDRESSED, no new Critical, no
new Important. Report: task-3.8-rereview-2.md.

It rebuilt the two claims I would not take on trust:
* The pair diff, independently, over all FIFTEEN test files. Zero genuinely asserted
  pairs escape the gate; its five raw hits were regex false positives, one of which was
  the NEGATIVE-direction assert at errors.rs:546. So the diff artefact holds.
* The numbers. 202/93 is right and my 195/93 counted 98.903 and 90.999, two pairs this
  parser cannot raise. I re-derived 1020 = 567 translation + 9 install + 444 negative
  myself from field 1 of the TSV, and the 9 install rows, so the arithmetic is checked
  from two directions.

Task 3.8: minor (deferred): cases() dropped the "program field must not contain a raw
  tab" assertion, so splitn(4) at tests/errors.rs:137 now folds a raw tab into program
  bytes silently. Corpus has none today. This is a FORMAT-GUARD REGRESSION introduced by
  this fix round, which is the only thing in the round that got worse -- flag it to the
  final review rather than losing it.
Task 3.8: minor (deferred): errors.rs:595 still stacks two measured expectations in one
  #[test], the exact shape this round split elsewhere. Both correct today.
Task 3.8: minor (deferred): src/instruction/tests.rs:1490 says tests/errors.rs
  "re-measures" the interpret errors; it asserts recorded measurements and measures
  nothing at run time.
Task 3.8: minor (deferred): report internals stale in two places -- "one assertion each"
  for the label-colon tests (code has two and four) and "82 of 181" in the residual
  section (tree and both derivations say 93 of 202). Tree is consistent; the report is not.
Task 3.8: minor (deferred): near-duplicate corpus rows differing only in a trailing
  newline. Byte-dedup passes, so redundancy only.

Task 3.9 dispatched, BASE c85a43f1.

=== Task 3.9 pre-flight, 2026-07-29 ===
FIFTH CONSECUTIVE PRE-FLIGHT TO FIND REAL BRIEF DEFECTS: three, three, four, four, now
five. Two of the five were mine and both reproduce. Plan fixed in ad1c708f, brief
regenerated.

DEFECT 1, and it is the shape probe-discipline warns about: the task displayed the
continuation example as tracing `say "x","y"` while its own next clause said the
continuation line's four leading blanks are kept. Both cannot hold. Measured under
trace r: `     2 *-* say "x",    "y"` -- blanks kept, so the PROSE was right and the
displayed string was a simplification I never measured. It appeared twice, task body and
Notes. The task body now records that an earlier draft showed the wrong one, so the
correction cannot be re-derived away. Also worth keeping: the traced line number is the
clause's FIRST line, 2, not the continuation's.

DEFECT 2: trace_output.rex sets `trace i`, not `trace r`, so the real capture interleaves
value-marker lines and the program's own output. The plan presented the *-* lines as "the
file's own output". Acceptance filters to *-* so it does not change the task, but the
block was lying about what it was.

Answers given, for the record:
* The forbidden accessor is a raw WHOLE-TEXT one, because that is what lets a caller
  re-derive line boundaries ProgramSource owns. A join method beside span_bytes is the
  opposite of that. Reading (a) confirmed, Cow<'_, [u8]>, borrowed single-line case.
* Files list is permission, not obligation -- but "no span repair was needed" must come
  from Step 2's failing test, not from a prediction, and the report must name which
  constructs the test actually exercised so coverage is distinguishable from luck.
* Tests stay in tests/sourceline.rs. Fewer test binaries is worth more than tidier
  naming in this phase specifically, because test-target fragmentation has produced two
  wrong conclusions already: the per-target false survivor and the fail-fast
  misattribution.

Task 3.9 implemented at 6614ac34, one commit: ProgramSource::join_span plus eight tests
in tests/sourceline.rs. 557 passed / 0 failed re-run independently, clippy clean, only
two files touched (source.rs +47, tests/sourceline.rs +203).

NO CLAUSE SPAN NEEDED REPAIR. Tasks 3.4 and 3.6 produced spans every reconstruction
matched, so src/clause.rs, src/ast.rs and src/instruction.rs are untouched. Treat that as
a result of the test rather than a prediction only because the implementer was told to
demonstrate it; it also names which constructs the tests exercised, so coverage is
separable from luck.

SIX ORACLE CLAIMS RE-MEASURED BY ME, all byte-identical:
  probe C  `say "x",    "y"`, four blanks kept
  CRLF spelling of probe C traces the identical text, so the join drops both bytes
  probe A  nine *-* lines, the loop's three clauses repeating per iteration, `end` its own
  probe B  `here:` / `nop;` / `say "two"`, three clauses on one line
  probe D  `say 1,  + 2`
  probe F  interpret "nop; say 1" traces `nop;` and `say 1` at the INTERPRET's own line
Doing this myself is the point: four fabricated-oracle-line defects have been caught on
this branch and a comment that merely looks measured is worth nothing.

THE IMPLEMENTER'S OWN CONCERN IS THE ONE WORTH ACTING ON: Step 2's red state was a
COMPILE ERROR, not a behavioural failure, which proves nothing -- same shape as a mutation
"caught" by a compile error, already a defect class here. Its mitigation is two tests
asserting the raw span still contains the terminator the join drops. The review is asked
to settle it by experiment: defeat join_span to return span_bytes unchanged and see which
of the eight tests go red. Any that still passes is not testing the join.

Also handed to the review: join_span's edge cases (span on a terminator byte, empty span,
end mid-CRLF, LF-CR as two terminators, Ctrl-Z truncation), the borrowed/owned contract
since callers can match on Cow, and the coverage the implementer admits is missing --
continued ELSE/OTHERWISE and multi-label clauses have only Task 3.6's single-line pins.

Review dispatched (sonnet, a9c840d6ba8f6109f), BASE c85a43f1.

Task 3.9 review: spec PASS, quality approved, no Critical, no functional defect.
Report: task-3.9-review.md.

THE DEFEAT EXPERIMENT SETTLED THE COMPILE-ERROR-RED-STATE QUESTION, and in the
implementer's favour: replacing join_span's body with span_bytes turns FOUR of the eight
tests red, not the two the report credits. The other four stay green correctly, having no
continuation in them. So a compile-error red state at Step 2 did not leave the join
untested here -- but the only reason we know that is that someone defeated the function
and looked. Make that experiment the standard answer whenever an implementer reports a
red state that was a compile error.

FIX ROUND 1 DISPATCHED, four items:
* Important: the coverage gap the implementer disclosed is real and the reviewer proved
  the constructs REACHABLE and the code CORRECT on all of them, so it is a missing-test
  item. Continued ELSE arm (I re-measured: `say 1,    2`, blanks kept), continued
  OTHERWISE arm, a THREE-fragment continuation, and a multi-label clause. The
  three-fragment case matters most: all eight shipped tests join exactly two fragments,
  so join_span's loop has never run more than one extra iteration.
* Important: assert_traced cannot detect an INCOMPLETE expectation list, since nothing
  ties it to the oracle transcript. Not exploited -- the reviewer cross-checked all four
  call sites, 6/6, 9/9, 4/4, 2/2. Fix is a floor, not a mechanism: reject an empty list
  and put the caller's obligation in the doc comment. An index swap is already caught
  because every clause text in these probes is textually distinct.
* Minor: two structuring semicolons in comments.
* Minor: the value-marker list in a comment is factually right for that file but is the
  shape of list that has been wrong three times; keep it only if it says it is what that
  file emits rather than the set.
Plus: join_span(0..0) on a zero-line source returns owned-empty where the one-line case
returns borrowed-empty. Unreachable and disclosed, so either fix the fast path or make
the doc comment true, but they must agree -- callers can match on Cow.

The reviewer's one repo edit was the defeat experiment, reverted with a path-scoped
git checkout; I confirmed the tree clean at 6614ac34 and 21 tests in the sourceline
binary before dispatching the fix.

Task 3.9 fix round 1 landed at ab07e536. 561 passed / 0 failed re-run independently
(557 + 4), clippy clean, tree clean, two files, +102 lines.

ALL FOUR NEW PROBES RE-MEASURED BY ME against the oracle, byte-identical:
  probe G  continued ELSE:      `if 1 = 2 ` / `else` / `say 1,    2` / `trace off` at 5
  probe H  continued OTHERWISE: `select` / `when 1 = 2 ` / `otherwise` / `say 1,    2`
                                / `end` / `trace off`, so WHEN keeps its trailing blank
                                the way IF does
  probe I  three fragments:     `say 1,  2,    3`
  probe J  multi-label:         `a:` / `b:` / `nop`
Probe I is the first test in the phase that makes join_span's loop iterate more than once.

The contract was rewritten rather than the corner patched: borrowed exactly when the span
contains no terminator byte, owned exactly when the join dropped something, with an
early return for the empty span. I hand-checked both directions on the terminator-byte
and CRLF cases and they hold, but a contract callers can `match` on is worth a second
pair of eyes, so the re-review is asked to build counterexamples rather than reason.

THE IMPLEMENTER'S SELF-REPORT IS THE USEFUL PART AGAIN: it credited two tests with
catching a span_bytes regression where the defeat experiment measured four, and says
plainly that it INFERRED instead of running the replacement. Same shape as the earlier
"reported a defect, did not fix it" admission. The lesson generalises: a claim about what
a test suite would catch is a measurement, not a deduction.

Scoped re-review dispatched (sonnet, a8e8b36fe36a06dc6) over 6614ac34..ab07e536, told to
assume the round broke something, to defeat join_span and report which of the FOUR new
tests go red (probe I especially: if it survives a defeated join, the three-fragment path
is still untested), and to check probe G's two unmeasured comment claims -- that the
untaken THEN branch is not traced and that `else` carries no blank on either side.

Task 3.9: complete at cf5f7650.
Re-review of the fix round returned all five items addressed and the contract exact in
BOTH directions -- checked by an exhaustive checker over 20 synthetic sources, every
(start, end) pair, with ground truth computed from line_span rather than from join_span,
covering the single terminator byte, mid-CRLF end, LF-CR two-terminator empty line,
only-terminators source, Ctrl-Z truncation, zero-line source and Interpret source. Zero
violations. It also proved the new empty-span early return changes behaviour ONLY for the
zero-line 0..0 case, so nothing was masked.

ONE NEW IMPORTANT, and it is a counting defect rather than a code defect: PROBE J ADDS NO
DEFEAT-DETECTABLE COVERAGE OF THE JOIN. Defeating join_span reddens 7 of 25 tests in the
file -- the 4 the first review found plus G, H and I -- but J stays green, because none of
its three clause spans holds a terminator and span_bytes already answers correctly there.
J is real coverage of multi-label span extraction and zero coverage of the join, so the
round added THREE tests of the join, not four. Fixed in cf5f7650 by saying so in the test
itself; a comment is the right place because the number is what a later reader would
otherwise recount wrongly.

The general lesson, third instance this phase: A TEST THAT ASSERTS THE RIGHT VALUE IS NOT
AUTOMATICALLY A TEST OF THE MECHANISM YOU ADDED IT FOR. Defeat the mechanism and see what
goes red. That experiment has now corrected two counts, 2->4 and 4->3, both of which had
been arrived at by inference.

Probe G's two unmeasured comment claims also hold, re-measured by the reviewer on its own
capture: the untaken THEN branch is not traced at all, and `else` carries no blank on
either side (the leading spaces in the raw oracle line are TRACE's indentation, not clause
content).

Task 3.10 dispatched, BASE cf5f7650. Last task in the phase.

=== Task 3.10 implemented, 2026-07-29 ===
9cd1752f: benches/parse.rs, the Cargo.toml dev-dependency, and a new section in
d10-decision.md. Sixth consecutive pre-flight to find real brief defects (3, 3, 4, 4, 5,
5), and two of this round's are mine, fixed in c1f84b0a:

* Step 4 said to "say plainly whether it fits", which CONTRADICTS the correction two
  paragraphs below in the same task and asks for exactly the conclusion the data cannot
  support. This is the third time a Phase 3 task has carried a correction in one place and
  the thing it corrects in another. The implementer followed the correction, not the step.
* d10-decision.md told a later reader to re-measure the spike as a whole-file parse "when
  Task 3.11 exists". No Task 3.11 exists; the phase's twelve tasks end at 3.10, and 3.10
  is that re-measurement, sitting in the very next section.

NUMBERS RE-RUN BY ME AND THEY REPRODUCE: CoreClasses.orx 2.6317-2.6429 ms at 50.9-51.1
MiB/s, StreamClasses.orx 651.15-658.63 us at 54.4-55.1 MiB/s, p = 0.40 and 0.62 against
the stored baseline so no drift. Combined about 3.29 ms.

THE BEST FINDING OF THE TASK IS THE IMPLEMENTER'S, and it is the blindness class again:
asserting Program::instructions.len() alone is nearly worthless on these two files,
because the main body holds 41 and 7 instructions while the real content, 2,390 and 610,
lives nested inside directive bodies. A parser that dropped every directive body would
still post a passing count. It asserts a triple instead: main-body instructions,
directive count, nested total. Exactly the "over what set is this quantified, and can that
set exhibit the deviation" question from the gate-criteria notes, applied to a benchmark
assertion rather than a gate.

ONE CLAIM OF ITS OWN THAT DOES NOT HOLD: it says the counts reuse "the numbers already
pinned in src/directive/tests.rs". That file pins 347 directives and nothing else -- 41, 7,
2390 and 610 appear nowhere else in the crate, so four of the six are newly measured by
the benchmark itself and are self-consistent rather than independently pinned. Still a
useful change detector, but it must not be described as a pin from elsewhere. Handed to
the review.

Review dispatched (sonnet, a70e721be7e14baa5) over cf5f7650..c1f84b0a, asked to construct
a plausible regression the triple would NOT catch, to settle where the unavoidable clone
of the owned Vec<u8> sits relative to the timed region, and to hunt for any sentence in
the new d10 section that a reader could mistake for a cold-start verdict.

Task 3.10 review: spec PASS, quality NOT APPROVED on one Critical. Fix round landed at
c0e1321a; 561 passed / 0 failed re-run, tree clean.

THE CRITICAL WAS A FALSE PROVENANCE CLAIM AT FOUR SITES, not three: the benchmark's
module doc, the Case struct doc, the panic message, and d10-decision.md all said
src/directive/tests.rs pins all three asserted counts. It pins directives.len() == 347
and per-kind counts summing to 153. main_instructions (41, 7) and nested_instructions
(2390, 610) are the benchmark's OWN first measurement, hardcoded from the implementer's
throwaway pre-flight. Still a useful change detector, but a change detector dressed as an
independent pin is the exact dishonesty this task exists to prevent. All four fixed.

The implementer's own account is the useful part again, and it is a new shape: IT FOUND
THE PINNED-VERSUS-FIRST-MEASURED DISTINCTION IN ITS OWN PRE-FLIGHT AND THEN WROTE THE DOC
COMMENTS AS IF EVERYTHING WERE PINNED. The finding did not survive from discovery to
artefact. Third variant of one failure this phase: found-and-not-fixed, reported-and-left-
in-a-comment, and now discovered-then-contradicted-by-my-own-prose.

ASSERTION LEFT UNEXPANDED, DELIBERATELY. The reviewer established why the triple works:
instruction lists are flat and source-ordered and If/Do/Select hold target INDICES into
them, so a dropped clause anywhere moves a count. Three classes escape: a control-flow
target wired to the wrong index with counts unchanged, a clause moved across a directive
boundary while the cross-directive sum holds, and anything inside an Expr. That limit is
shared with tests.rs's own acceptance tests, and a benchmark is the wrong place to grow a
structural checksum. Recorded as prose in both artefacts and carried into the parent plan
as PHASE 3'S FOURTH HANDOVER (bd233ed1), since Phase 4 meets all three as run-time
misbehaviour rather than as a failing parser test.

Clone placement settled by measurement, not argument: outside the timed region, and the
clone costs 1.01 us for the 141,049-byte buffer, 0.04% of the parse. Immaterial either
way, which also means the implementer's "would have overstated the cost" framing was
overdramatic and has been softened.

=== A DOCUMENT-WIDE DEFECT FOUND WHILE CLOSING 3.10, worth its own entry ===
"Parse time IS cold-start time" survived in SIX places across the three plan documents
(455d6b65), including in the very paragraph that the parent plan's own corrections list
names as the origin of the wrong claim. The parent plan has recorded the correction since
its THIRD PASS and the assertion outlived it by three more.

The mechanism is the one already in agent-workflow-preferences: a correction lands where
it was noticed and the original assertion is left standing elsewhere. What is new is the
scale -- this was not one stale sentence but a claim reproduced in every document that
touched the subject, so a per-task contradiction scan could never have caught it. Two of
the six sites now say where the claim came from, because a correction that leaves the
original in place is exactly how it survived. When a claim is corrected, grep every plan
document for the CLAIM, not for the section.

=== EXIT GATE INVENTORY, 2026-07-29, twelve of twelve tasks implemented ===
Eleven criteria. What the tasks already discharge, and what nothing does yet.

ALREADY EVIDENCED:
  corpus/lang parses            tests/program.rs:325, with a floor of 14 programs
  CoreClasses + StreamClasses   directive/tests.rs (347 directives) and 3.10's bench
  parse errors both directions  Task 3.8, criterion rewritten twice and now a diff
  TRACE *-* reconstruction      Task 3.9, four assert_traced call sites
  throughput recorded           Task 3.10, 3.29 ms for both files
  clippy clean, zero unsafe     continuous
  allow(dead_code) owners       Task 3.8 reported ownerless zero; re-check at the gate

NOTHING BUILDS THESE FOUR YET, and no task owned them:
  1. samples/ round-trip, 301 files / 67,519 lines. NO test references samples/ at all.
     This is the parent plan's own Phase 3 criterion and the gate's primary breadth
     instrument.
  2. Tiling over corpus/lang: expression spans nest, consecutive clause_spans are
     ordered and non-overlapping, and only whitespace, comments and continuations sit
     in the interstices.
  3. Variant enumeration: every Instruction and Expr variant constructed at least once,
     asserted by enumerating variants rather than by inspection, over corpus AND
     samples together with samples as the primary instrument.
  4. SOURCELINE(n) against the interpreter for every line of every corpus program, via
     the separate .Package~new driver so the files under test are not edited.
  Plus a re-run of Phase 2's twelve differential sets, 128,368 cases, to confirm still 0.

That is gate work, not task work, and it is the last thing between here and the phase
review. Worth naming that it is roughly the size of a task and nobody planned it: the
plan put four buildable criteria in the gate and no task in the body that builds them.

SEVENTH COPY OF THE COLD-START CLAIM, found in the gate itself and fixed in 441551b2.
The criterion asked for "a plain statement of whether it fits". Yesterday's sweep missed
it because it greps for the words "cold-start time" while this line states the claim as a
CONCLUSION TO DRAW instead. A criterion can restate a corrected claim in vocabulary the
correction's own grep cannot match, which is a sharper version of the lesson than the one
recorded above: grep for the claim, and then look for the places that ask for it as an
answer rather than assert it as a fact.

MORITZ'S DIRECTIVE, 2026-07-29: the samples/ round-trip must run as a `cargo test`, not
as a shell loop. Applied to the gate work generally -- a criterion enforced by a script
nobody runs is not enforced, and the gate's own text had offered a shell loop as the
oracle half. Anything that genuinely cannot be a Rust test (an oracle capture) is a
driver plus a committed expectation file that a Rust test reads.

Gate work dispatched (fable, a98446be77a312859), scoped to NEW test files under
rust/crates/rexx-parse/tests/, a scratchpad driver, and a new
docs/superpowers/plans/phase-3-gate.md modelled on phase-2-gate.md. Told explicitly not
to touch src/source.rs or benches/parse.rs, since the 3.10 re-review is running
concurrently and may edit the latter.

Four harnesses plus three confirmations. Two things it was told that are not obvious:
* Report counts, do not assert them. 301 files and 67,519 lines will rot, and a test
  finding ZERO files must fail, so it asserts a floor rather than a total.
* Prefer an exhaustive match that fails to COMPILE when a variant is added over a
  hand-written variant list. An enumeration that can go stale is worth much less, and
  this phase has already had a list go stale three times.

=== GATE PRE-FLIGHT, six findings, and one of them is a gate defect ===
Seventh consecutive pre-flight to find real defects.

THE ONE THAT MATTERS: TILING PROPERTY 1 IS VACUOUS. Expr::new widens a node's span to
contain its children BEFORE storing it, so containment holds by construction and NO INPUT
CAN FALSIFY IT. Checking it says nothing about the parser. This was even recorded in the
3.5 line of this ledger -- "Expr::new widens so criterion 1 holds by construction" -- and
nobody, me included, drew the conclusion that the criterion is therefore empty. Third
vacuity in this phase, and the first that was written down a week before it was noticed.

Ruled: keep the containment check, because it pins the invariant against a future change
to Expr::new, but record it as MET BY CONSTRUCTION WITH NO FALSIFYING INPUT POSSIBLE and
name the code that makes it so. A hand-built lying span tests the checker, not the parser,
and the gate doc must not let those read as the same evidence. Then add the property that
IS falsifiable: containment survives widening even when the tree is wrong, but TIGHTNESS
AND ORDERING do not. For a binary operator, the operator's own token bytes must lie
between the left operand's span end and the right operand's span start, and operand spans
must not overlap. A mis-nesting satisfies containment and violates that. Property 2 gets
the same treatment if no parsed input can violate it either.

Other five, answered:
* corpus/lang has NO file without a trailing newline, so that clause of the SOURCELINE
  criterion is untestable as written. Authorised one witness file, to be noted in the doc
  as created for the purpose rather than found.
* Variant enumeration: hard-gate InstructionKind (39) and ExprKind (14) as worded, and
  GATE DirectiveKind and LoopKind too if coverage comes out complete. "Report, do not
  gate" is how a check rots.
* `;` in an interstice: stay strict and reject, since a null clause produces no
  instruction. Flagged that this must be revisited before the checker is ever extended
  to samples/, where 301 real programs are far likelier to hold one than 15 hand-written.
* Oracle expectations under tests/sourceline_oracle/, committed, with the regenerating
  command in a comment in the reading test rather than only in the report -- the report
  is gitignored and the expectations are not.
* Assess all eleven criteria, and say for each whether the evidence was verified
  first-hand or cited from a task report. Different strengths, and the doc must not blur
  them.

Two observations of its own worth keeping: some samples/ files are ISO-8859 and NOT
UTF-8, which is the first hard evidence for D14 from real files rather than probes and
makes the bytes-not-str rule load-bearing; and no corpus file uses ::RESOURCE, whose raw
body lines would sit in interstices and break tiling for a reason unrelated to a dropped
clause.

Task 3.10: complete. Re-review APPROVED. Report: task-3.10-rereview.md.
All four items confirmed fixed, verified against src/directive/tests.rs directly rather
than against the new prose: 347 is a literal there, 7/139/5/2 are literals, and 41, 2390,
610 and 153 appear as literals nowhere. It also PROVOKED the panic message by editing an
expected constant and running the bench, which is the right way to check a message no test
path reaches, and reverted path-scoped.

It found the seventh cold-start copy INDEPENDENTLY by grep before discovering 441551b2 had
already fixed it. Two agents converging on the same gap by different routes is the
strongest evidence yet that the sweep-then-miss pattern is systematic rather than
careless.

ONE IMPORTANT AGAINST MY OWN EDIT, and it is correct: REMOVING A FALSE PREMISE LEFT THE
CONFIDENT SENTENCE IT HAD BEEN HOLDING UP. The axis-3 paragraph still said throughput at
8.3x was "decisive by itself" after the premise that made it so was deleted. Two concrete
problems the reviewer verified in the source data rather than argued from prose:
* The 8.3x was measured on the spike's EXPRESSION GRAMMAR LAYER ALONE, over 1,912 of
  CoreClasses.orx's 4,193 lines, with the shared scanner subtracted and no instruction or
  directive layer written. Task 3.10's 3.29 ms times the WHOLE SHIPPED PARSER. Multiplying
  one onto the other conflates two scopes.
* Even granting it, roughly 24 ms of extra parse time called decisive against a ~55 ms
  budget whose other components the same document says twice are unmeasured.
Fixed in 23e33ee6: axis 4 carries the decision alone as a measured fact, axis 3 points the
same way with an unestablished magnitude, and the paragraph now says which is which.
D10(b) unchanged. Also found while there: the error-36 section still claimed ITS axis was
decisive on its own, written before the scope decision devalued it, and now defers.

A NEW FAILURE SHAPE, and the sharpest of the phase: a correction can be locally right and
globally wrong. Deleting the false premise was correct; leaving the conclusion it
supported turned a false argument into an unsupported one, which is harder to notice
because nothing in the paragraph is now untrue. WHEN REMOVING A PREMISE, RE-READ WHAT IT
WAS HOLDING UP.

=== EXIT GATE CLOSED, 2026-07-29 ===
Four harnesses built, all as cargo tests per Moritz's directive. Commits 31db9b89,
f8887d34, c6267de1, 7ce33726. Assessment in docs/superpowers/plans/phase-3-gate.md.

VERIFIED BY ME, NOT TAKEN ON REPORT:
* 575 passed / 0 failed / 3 ignored across the workspace.
* samples/ is 301 files and 67,519 lines by my own find and wc, and the test PRINTS
  exactly those numbers, so the harness and the filesystem agree.
* The four gate binaries run: samples 1 test, tiling 11, variants 1, sourceline_oracle 1.
* TILING SHIPS TEN CAN-FAIL PROBES AND ALL TEN PASS, which is the thing that makes the
  criterion mean anything: rejects a child escaping its parent, overlapping operands, a
  mis-nested operand, a missing operator, an operator with nothing in front, a dropped
  clause, overlapping clause spans, bytes after the last clause, an interstitial
  semicolon, and permits comments and continuations. That is the falsifiability the
  criterion previously lacked.
* variants.rs builds its match through a macro with NO wildcard arm, so a new variant is
  a compile error rather than a silent gap. The claim holds by reading.

Criteria: ten met, one (expression containment) met but recorded as VACUOUS AS PARSER
EVIDENCE, with the binary-tightness property added as the falsifiable substitute. The AST
retains no operator token position, which is why tightness had to be expressed through
operand spans; noted for Phase 4.

Its own two admissions worth keeping:
* The interstice scanner shipped a real bug before the corpus caught it -- a subtract
  operator swallowed as a phantom continuation in `recurse(n - 1)`. The corpus found it,
  which is the corpus doing its job, but it also means the checker was wrong in exactly
  the direction that would have hidden a dropped clause.
* The 128,368-case differential run is still a script plus oracle, not a cargo test,
  because it needs build/bin/rexx. Unchanged from Phase 2 and now recorded rather than
  implied.

Open minor: tests/gate_walk/mod.rs carries a scoped allow(dead_code) with no named owner.
It sits outside criterion 10's src/ anchor so the grep does not see it, which is the
letter of the criterion and not its spirit. Handed to the final review.

=== FINAL WHOLE-BRANCH REVIEW DISPATCHED, five slices in parallel ===
98 commits, 94 files, ~30k lines of Rust. One reviewer cannot hold that, so it is split:
scanner+clause (fable), expr+ast (fable), instructions (fable), directives+block+errors
(fable), API+gate harnesses+docs (sonnet). All READ-ONLY and told so explicitly, since
five agents share the worktree.

Each was told what the per-task reviews STRUCTURALLY could not see: inconsistency between
families built in separate commits, comments gone stale relative to code, C++ citations
that look checkable and are never rechecked, and tests that cannot fail. The directives
reviewer was additionally told to sample the corpus TSV against rexxc itself, since three
of the four fabricated oracle lines caught on this branch were in that slice.

=== FINAL REVIEW, slice 5 of 5 (API, gate harnesses, docs) returned ===
No Critical. Public surface sound, Phase 4 has clause spans, labels, source, directive
bodies and the symbol table. All four gate harnesses can genuinely fail and none has a
silent-no-op path. Report: final-review-api.md.

TWO IMPORTANTS, BOTH VERIFIED BY ME RATHER THAN ACCEPTED:

1. THE FALSIFIABLE SUBSTITUTE COVERS 2 OF 9 RECURSIVE ExprKind VARIANTS. Confirmed by
   grep: tiling.rs destructures only ExprKind::Binary and ExprKind::Prefix. Call.args,
   QualifiedCall.args, Message.target/.super_class/.args, List, Logical and
   VariableReference.inner are covered by CONTAINMENT ALONE, which is the vacuous check,
   because Expr::new widens the parent over every child whether or not the child is
   attached correctly. So a bug attaching the wrong argument to a Call, or mis-slotting a
   Message's super_class, passes the gate undetected. That is the same failure mode the
   vacuity finding was about, surviving inside its own fix. The reviewer is careful that
   the document does not BLUR vacuous versus falsifiable -- it states the distinction
   correctly throughout -- but oversells the SCOPE in one summary sentence.
   This is the sharpest thing the final review has produced: a fix for vacuity that is
   itself 78% vacuous, and nobody would have noticed from the prose.

2. "1,021 corpus rows" is 1,020. Verified two ways by me: the TSV is 1,082 lines, 61
   comment lines and 1 header, and `grep -vc '^#\|^class\t\|^$'` gives 1020 directly. It
   also matches my own earlier derivation, 567 translation + 9 install + 444 negative.
   The error is subtracting the comment lines and forgetting the header is not a data
   row, in a section that explicitly prides itself on counting precisely.

Three Minors: the doc says eight probe tests and lists eight where tiling.rs has nine
rejects plus one permits; "Property 2" names instruction ordering in the gate doc and
binary tightness in tiling.rs's module doc, each internally consistent and colliding
across files; and d10-decision.md states "roughly 24 ms" in the same paragraph that says
computing it that way conflates two scopes. That last one is mine, and stating a number
in order to disclaim it is worse than not stating it -- a skimming reader keeps the
number and drops the disclaimer.

HOLDING ALL FIXES until the other four slices return. The SDD rule for a final review is
ONE fix dispatch and one scoped re-review, and fixing piecemeal now would mean re-reviewing
the same files four more times.

=== FINAL REVIEW, expression grammar and AST slice returned ===
No Critical. Report: final-review-expr.md.

ONE IMPORTANT, AND IT IS THE BRANCH'S TRACKED DEFECT CLASS IN A NEW SPELLING: three C++
citations in expr.rs give line numbers that land inside a DIFFERENT function than the one
named. Verified by me: parseConstantExpression is at 2632 and the comment cites 3400,
which is inside parseMessage; parenExpression is at 2695 and the comment cites 3465;
needVariable is at 885 and the comment cites 3555, inside parseMessageTerm. A borderline
fourth cites 3245 for parseQualifiedSymbol, whose doc starts at 3250 and body at 3261, so
it lands past a function boundary rather than inside the usual comment slack.

The DESCRIBED BEHAVIOUR is right in all four; only the numbers are wrong. That is exactly
what makes it worth fixing: a citation that looks checkable and is never rechecked is how
the fabricated-oracle-line class started, and a wrong line number sends the next reader
into an unrelated function and makes them doubt the correct prose. The reviewer checked
about thirty other citations across the slice and found them accurate, including every
error constant number, so this is three bad ones and not a systemic rot.

WHAT IT VERIFIED RATHER THAN READ, and this is the strongest evidence the phase has for
the expression layer: the precedence table matches Token.cpp:111 level for level including
the <=-pop left associativity, and 32 value probes plus 17 error sub-number probes ALL
reproduce against build/bin/rexx. Among them the counter-intuitive ones this project has
got wrong before: -2**2 = 4, 2**3**2 = 64, f(,) passes ZERO arguments while f(,1) passes
two, (1,)~size = 2 and (1,,)~size = 3, and 'ABS'(-3) = 3 where lowercase 'abs' raises 43.1.
Blank abuttal boundaries too: a comment is transparent so a/*x*/b abuts to 23, but
a/*x*/(b) is a CALL and the oracle says 43.1 "routine A".

Two of its Minors are the vacuity class again:
* expr/tests.rs:741's containment check cannot fail on parser output for the same
  Expr::new reason, and its comment implies a falsifiability it does not have. tiling.rs
  now admits this about itself; this test does not.
* differential.rs:208's SYNTAX_ERRORS lists 6/13/25, which never occur in the corpus and
  contradict the TSV header's own list of 35/36/37/19/20. Unexercised today, so it would
  silently reclassify rows after a regeneration rather than fail.

=== 2026-07-30: THREE OF FIVE FINAL-REVIEW SLICES DIED ON THE SESSION LIMIT ===
Not on failure. scanner+clause, instructions, and directives+block+errors all terminated
with "You've hit your session limit" mid-verification, and none left a report file. Their
last visible lines show they had got somewhere and were still going:
  scanner      "The keyword tables match exactly. Now let me compare the C++
                locateToken/sourceNextToken against the port."
  instructions "Citations to translateBlock are exact."
  directives   "One more provenance check on the corpus header's install-class claim --
                where does 98.903 actually fire in the C++?"
Those three slices are OUTSTANDING and must be re-dispatched. Do not treat the final
review as complete: two of five slices returned.

Rather than re-dispatch three agents into a limited budget, I applied the two completed
slices' findings myself, since every one was already verified. c568497d:
* Four C++ citations in expr.rs corrected. Each named the right function and a line inside
  a DIFFERENT one: 2632 not 3400, 2695 not 3465, 885 not 3555, 3261 not 3245. I checked
  all four against the C++ myself before editing.
* The gate doc's oversold scope. The falsifiable substitute reaches Binary and Prefix, two
  of nine child-holding ExprKind variants; Call and Message argument attachment is still
  on containment alone. Scope stated, remedy named (sibling ordering over argument lists),
  left for Phase 4 where a wrong attachment is loud rather than silent.
* 1,020 rows not 1,021, with the derivation and the arithmetic error both recorded.
* Nine reject probes plus one positive control, not eight.
* expr/tests.rs's containment test now says it cannot fail on parser output and what it
  does buy, which is a guard on Expr::new's widening rather than evidence about the parser.
* SYNTAX_ERRORS explains why it is wider than the corpus header's list: it is the property,
  every translation-time number, not an inventory of today's generator output.
* d10-decision.md no longer states the 24 ms it immediately disclaimed. MY OWN DEFECT, and
  the general rule is worth keeping: computing a figure in order to caveat it leaves the
  reader holding the figure and dropping the caveat.
* tiling.rs's Property numbering now says it differs from the gate doc's, since the two
  documents used "Property 2" for different properties.
575 passed / 0 failed, clippy clean, after the edits.

NOT FIXED, deliberately: gate_walk/mod.rs's allow(dead_code) has no named owner, and
should not have one. Its comment already reasons that the allowance is per-binary surplus
in a shared test module rather than a not-yet-called item, so nothing will ever delete it
and "name the task that deletes it" does not apply. The reviewer flagged it as ownerless;
the honest answer is that the criterion is scoped to src/ for exactly this reason.

Three dead slices re-dispatched (fable each): scanner+clause ace8d7045325b3a11,
instructions ad3bdbee0c193d258, directives+block+errors abaab61e809f077f5.

ONE CHANGE, AND IT IS THE LESSON FROM LOSING THEM: each is told to CREATE ITS REPORT FILE
FIRST and append after every meaningful check, rather than composing it at the end. All
three previous agents died holding real findings and left nothing on disk. A bare partial
file beats a perfect unwritten one, and the harness gives no warning before a session
limit. Make this standard in every long-running review dispatch.

Each also carries the prior run's last words as a hint about where it had got to, marked
UNVERIFIED so it is a starting point rather than a result to inherit:
  scanner      keyword tables matched; was moving to locateToken/sourceNextToken
  instructions translateBlock citations exact; was about to bulk-verify error numbers
  directives   was asking where 98.903 actually fires in the C++
And each is told the expr.rs citation defect is LIVE -- four citations naming the right
function and a line inside a different one -- so citation checking is now a named priority
in all three slices rather than a general instruction. Each must also report what it did
NOT reach, so the coverage boundary is explicit instead of inferred from silence.

=== FINAL WHOLE-BRANCH REVIEW COMPLETE, all five slices in, 2026-07-30 ===
NO CRITICAL IN ANY SLICE. Reports: final-review-{scanner,expr,instructions,directives,api}.md.
Fixes in c568497d (round one) and 9f68662a (round two). 576 passed / 0 failed, clippy clean.

THE ONE REAL CODE DEFECT THE WHOLE REVIEW FOUND: DO WHILE and DO UNTIL with an empty
condition raised 35.908 and 35.909. Those are DEAD CODE in the C++ -- parseLogical raises
Error_Invalid_expression_logical_list itself as soon as a sub-expression comes back null,
so requiredLogicalExpression's per-caller numbers are unreachable for every caller.
Verified by me under rexxc: `do while`, `do until`, and both controlled forms all give
35.929. What makes it worth more than one sub-number: THE MODULE ALREADY KNEW. The IF arm
passes 929 with a comment explaining exactly why, and GUARD passes 929. loop_conditional
was the single logical() site that missed the rule its own file documents. Nothing pinned
either number, so 575 green tests covered it. Fixed with a test.

THE SHARPEST DOCUMENTATION DEFECT: the corpus header's install-class provenance was
backwards. It claimed the nine rows fire AFTER translation, from LibraryDirective::install
via PackageClass::processInstall running from a called stub. Wrong three ways -- none of
the nine is a ::REQUIRES, which is the only path through that function; the real sites are
inside the parser; and rexxc HAS NO INSTALL STEP YET REPORTS BOTH, which I confirmed at
rc 158 and rc 157. The decision to accept the nine rows never rested on the mechanism, only
on the failure depending on the machine rather than the program text, so the classification
survives. It mattered anyway: Phase 4 could have built an install-time error ordering on it.
The same false claim was repeated twice in tests/errors.rs.

Also fixed: two more citations naming the right function and a line inside a different one
(initializeForParsing's :764 attributed to translate, scanSymbol at 1650 not 1792), the
shebang counts (494 of 790 with the exact spelling matches NEITHER real figure -- 492 have
any shebang, 446 that spelling; I reproduced both), and a token-cursor test that ended by
comparing its own array literal with itself, now checking that the cursor's indices SELECT
the intended tokens.

WHAT THE FIVE SLICES BOUGHT, worth recording because the per-task reviews had already
passed all of this: one wrong sub-number, one backwards provenance in the phase's most
load-bearing artefact, seven bad C++ citations across three files, three stale measured
counts, and three tests that could not fail. Zero of those were findable from a single
task's diff, which is the argument for the whole-branch pass existing at all.

Citation drift is now a MEASURED defect class on this branch, not a suspicion: seven wrong
line numbers, of which six named the right function and a line inside a different one. The
substance was correct every single time. That is what makes it dangerous -- the prose is
trustworthy and the pointer is not, so a reader who follows it doubts the prose.

Two slices reported coverage boundaries rather than claiming completeness, which is the
right habit: OTHERWISE is the thinnest instruction (accepted-in-place probe only), and no
novel probe input was run through the Rust parser end-to-end for the directive slice
because no full-parse CLI exists.

=== PHASE 3 COMPLETE. PICKUP STATE, 2026-07-30 (compaction point) ===
HEAD 9f68662a on plan/rust-rewrite. Tree CLEAN. 576 passed / 0 failed / 3 ignored
(the 3 ignored are pre-existing 2-3 GB allocation tests). clippy -D warnings clean,
zero unsafe.

Twelve of twelve tasks done, exit gate closed, whole-branch review done in five slices,
two fix rounds applied. Moritz chose KEEP THE BRANCH AS-IS: no merge, no push. Base is
ci/platforms, fork point 8c880bdd, and ci/platforms has since advanced to a7d8ac11, so a
merge would be a real merge and would have to be redone after Phase 4 anyway. The worktree
is host-owned (a sibling of the main repo, not under .worktrees/) so it stays in place.

DO NOT re-run any of Phase 3. The ledger above is the record and every commit it names is
in git.

WHAT PHASE 4 INHERITS, four handovers in the parent plan plus two from the gate:
  1. resolveCalls is deliberately NOT ported. CodeBody::labels carries the same
     information per body. Whoever owns call resolution must not assume it is done.
  2. TRACE's value lines are Phase 4's and gated on a decision. Everything except *-*
     carries an evaluated value, eighteen of the nineteen prefixes. Emitting an event per
     evaluation step forbids constant folding and fusion, so decide before designing the
     dispatch loop. TRACE.testGroup holds 239 expected trace-output lines.
  3. A method-name intern table. ExprKind::Message::name is Box<[u8]> because a literal
     message name never reaches the scanner as a symbol token.
  4. NOTHING IN PHASE 3 OBSERVES AST SHAPE, ONLY NODE COUNTS. Flat source-ordered
     instruction lists mean a dropped clause changes a count, so early stopping is caught.
     Three classes are not: a control-flow target wired to the wrong index, a clause moved
     across a directive boundary with the sum preserved, and anything inside an Expr.
  5. The gate's falsifiable expression property reaches Binary and Prefix only, 2 of 9
     child-holding ExprKind variants. Call and Message ARGUMENT ATTACHMENT is on
     containment alone, which is vacuous. Phase 4 exercises this on every dispatch.
  6. The AST retains NO operator token position, which is why tightness had to be
     expressed through operand spans. Worth knowing before designing anything that wants
     to point at an operator.

STILL OPEN, unrelated to Phase 3:
  * Two upstream ooRexx write-ups awaiting Moritz's filing decision: numberValue's missing
    `* numberSign` on the carry-only return, and addCompound's early return defeating
    captureGuardVariable.
  * No CI builds the Rust tree. Offered, never requested. Every "clippy clean, N tests
    pass" in this ledger is a local claim.
  * The 128,368-case differential run is a script plus oracle, not a cargo test, because
    it needs build/bin/rexx.
