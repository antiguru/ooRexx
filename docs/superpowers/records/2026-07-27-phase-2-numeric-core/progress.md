# SDD ledger — plan: docs/superpowers/plans/2026-07-27-phase-2-numeric-core.md

Context: Phase 2 of the ooRexx Rust rewrite. Tasks 2.1-2.4 complete; 2.5
implemented with 2 known residual divergences.

Verification for every task: the differential harness. Oracle side is
`build/bin/rexx rust/crates/rexx-num/tests/data-addsub-oracle.rex <cases>`,
Rust side is `rust/target/release/muldiv <cases>`, compared with diff.
Regression corpus lives in the session scratchpad.

Tasks in this run:
- A: 2.5 residual — port dividePower so negative powers match
- B: 2.6 — comparison operators, numeric and strict, with FUZZ
- C: 2.7 — formatting: ENGINEERING form, FORMAT(), TRUNC()

Task A: complete (commits 31ba5fe8..84e05ce8, all 8 differential sets at 0,
81 tests, clippy clean). Corrected a false earlier finding: the low-bit-first
exponentiation claim was compensating for the broken reciprocal; the C++
high-bit-first order is correct once dividePower is ported.
Task A: concern (deferred) — pow's final check_range has no C++ counterpart;
no corpus case distinguishes them today.
Task A: review clean (spec OK, quality approved).
Task A: minor (deferred) — subtract_multiple in pow.rs:229 merges the C++'s
two borrow branches into one formula with no comment saying so. Reviewer
verified the merge is correct (no rounding decision involved, so the
port-don't-reformulate hazard does not apply) but notes a future auditor
diffing line-by-line against the C++ would have to re-derive the borrow
algebra. One-line comment would fix it. For final-review triage.
Task A: complete (commits 31ba5fe8..84e05ce8, review clean, 1 minor deferred)

Note for future briefs: the "files permitted" list should say the plan
document is controller-managed, since every task commit updates it and a
reviewer will otherwise flag it as out of scope.
Task B: complete (commits 42615ac7..35e04071, 0 divergences on independent 32,368-case set)
Task B: review clean (spec OK, quality approved). Fast-path omission resolved
with a closed-form argument plus 64,316 targeted cases: it does not move a
rounding point, which is what separates it from the six failures this phase
has had. Not merely plausible -- resolved.
Task B: minor (deferred) — compare() does not re-assert fuzz < digits; the
invariant is enforced upstream in Settings, consistent with the crate's other
leaf entry points.
Task B: minor (deferred) — .nil comparison special-casing unimplemented,
unreachable from the string-only path; revisit when compare is wired to the
object layer.
Task B: minor (deferred) — cargo fmt flags a few lines, but the same style
appears in untouched pre-existing files; toolchain artifact, not this diff.
Task B: complete (commits 42615ac7..35e04071, review clean, 3 minors deferred)
Task C: fix round 1/5 dispatched — 3 findings (expt=0 does not force
exponential; expp pads a plain result; TRUNC must pad decimals). Verified
against build/bin/rexx directly. Implementer went idle without acting once;
re-sent with explicit commands. Divergences at dispatch: 88/1116, 343/7080.
Harness for this task is rust/crates/rexx-num/src/bin/fmt-check.rs
(controller-written, outside the implementer's file scope).
Task C: fix round 1/5 verified by the controller, not taken on report. Both
prior sets 0/1116 and 0/7080. Independent third set (fmt3, 12,136 cases,
values disjoint from fmt/fmt2) also 0 — it adds ENGINEERING form and the
before/after x expp/expt combination neither earlier set emitted. 127 tests
pass workspace-wide, clippy clean.
Task C: the implementer reported DONE but never committed; the work was
untracked in the tree. Controller committed it as e9fb7c1d. Watch for this.
Task C: harness gap the controller found and fixed (e79f6fb7) — fmt-check
hardcoded Form::Scientific and the oracle driver never set NUMERIC FORM, so
the ENGINEERING path had never been differentially tested despite being one
of the two ways a displayed exponent collapses to zero.
Task C: re-review dispatched over f360bb35..e79f6fb7.
Task C: minor (deferred) — format_with takes Option<u32> for before/after/
expp/expt, but the interpreter accepts values above u32::MAX: expp=4294967296
returns a 4,294,967,305-character result, no cap. Unreachable today since
nothing dispatches to format_with yet, but whichever layer wires the builtin
must not narrow the argument to u32. Controller-verified against build/bin/rexx.
Task C: pre-adjudicated non-finding — the huge allocation from
`" ".repeat(width + 2)` is fidelity, not a defect. format(3.14159,,,100000,0)
really is 100,009 characters in the interpreter and Rust matches exactly.
Not passed to the reviewer, so an independent report of it is signal.
Task C: controller found a HARNESS defect, not an implementation one —
fmt-check asserted <E41> for an unparseable argument, but FORMAT and TRUNC
both raise 93 (41 is for a bad literal in source). Fixed in 0052b434 with a
new `fmtedge` set (640 cases, exponent extremes + non-numbers) that found it.
All four sets now 0: 1,116 + 7,080 + 12,136 + 640 = 20,972 cases.
Task C: note for anyone extending fmtedge — expp=0 suppresses exponential
form and TRUNC never uses it, so either shape on a 1e999999999-scale value
materialises ~1e9 characters. That is faithful behaviour; it filled /tmp twice
during this work. Keep TRUNC to short values and keep FORMAT in exponential.
Task C: re-review returned spec PASS with C++ citations (trigger
NumberStringClass.cpp:2029, ENG grouping :2034-2043, displayed-exponent-0
blanks :2356-2371, error order :2057/:2190 before :2298) plus 2 Important
findings, both controller-reproduced:
  1. format.rs:334 `-(places as i32)` panics for places >= 2^31. Interpreter
     accepts: length(trunc(1, 2147483648)) = 2147483650.
  2. render_integer_padded `before as i32` wraps for before >= 2^31 ->
     spurious 93. Interpreter accepts: length(format(1, 3000000000)) = 3e9.
Controller correction to the reviewer's suggested remedy: clamping at
MAX_EXPONENT is WRONG. Both accepted values exceed 999999999, so the fix must
widen to i64/usize and preserve the full u32 range. Sent as such.
Task C: reviewer minor (fmt-check arg() swallowed malformed fields) was
controller-owned; fixed directly, all four sets still 0.
Task C: fix round 2/5 dispatched to sdd-format.

Task F (NEW, controller-found): the NUMERIC DIGITS upper bound is missing.
Found by asking whether the reviewer's i32-cast defect class existed outside
Task C's diff, which the review could not have covered. It does, in two
already-"complete" tasks:
  1. settings.rs set_digits_str accepts any u32. Interpreter caps DIGITS at
     999999999 (= MAX_EXPONENT) and raises 26 above it. Measured: 999999999
     accepted (length(1/3)=1000000001), 1000000000 -> E26, 2147483647 -> E26.
  2. lib.rs:301 `2 * digits as i32` overflows above 1073741823. Repro:
     `2147483647|1|+|1e-30` through debug muldiv panics "attempt to multiply
     with overflow". Release wraps silently and picks the wrong display form.
Fixing 1 makes 2 unreachable via Settings, but format() is pub and takes a
bare u32, so both need fixing. Brief written: task-F-brief.md.
Dispatch AFTER Task C round 2 lands — F touches lib.rs and C's fix is in
format.rs; not worth the shared-worktree conflict risk to overlap them.
Lesson recorded: a per-task review is scoped to that task's diff by
construction, so a defect *class* it identifies must be swept for elsewhere
by the controller. This one was two tasks back.

Task E: complete (commit e15e9b64). Rust arith 1.9804 s [1.9756, 1.9857] vs
C++ 1.15 s re-measured immediately before, so 1.72x SLOWER while doing
strictly less work. Phase 2 parity gate FAILS. Controller re-ran the
benchmark independently (1.9804 vs the agent's 1.9666, within 0.7%) and
re-measured the C++ side rather than trusting the recorded baseline.
Task E: the agent's root-cause speculation (Vec<u8> digits + per-iteration
parse allocation) is WRONG and must not be acted on. perf over the bench
binary, 30k samples: long_divide 48.20%, mul_magnitudes 10.22%, all of
libc malloc/free/memmove ~10.5% summed. One function, not the representation.
Task G (NEW, controller-found): long_divide finds each quotient digit by
repeated subtraction (up to 9 full-width passes per digit, ~95 per division
at DIGITS 20) while pow.rs's divide_power already contains the C++ estimating
algorithm from the dividePower port. Brief: task-G-brief.md.

Ordering from here — all three remaining tasks touch different files, but
each is sequenced behind whatever could collide with it:
  round 2 (format.rs, running) -> F (settings.rs, lib.rs, pow.rs)
  -> G (muldiv.rs, pow.rs) -> D (error text; touches format.rs + settings.rs)
G must follow F because both may touch pow.rs; D must follow round 2.
Task E: controller added an equivalence assertion to the benchmark
(commit follows e15e9b64). The Rust replay produces exactly
4629643519330627.7808, the same string `say total` prints under
build/bin/rexx, so the 1.72x gap is two implementations doing the same
4,000,000 operations. This also guards the Task G division rewrite: a
changed answer now fails the benchmark instead of quietly reporting a
faster number for different work.

PLAN CONFLICT — for the human to decide, does not block any running task.
Phase 2's exit gate has five criteria. Three cannot be met as written:
  1. "All eleven rust/corpus/num/ programs run under the Rust build with zero
     divergences" — rexx-diff runs a Rexx PROGRAM under an interpreter, and
     there is no Rust interpreter at Phase 2. Parser is Phase 3, dispatch is
     Phase 4. Also the count is now twelve, not eleven (form_notation.rex and
     format_trunc.rex arrived with Tasks 2.6/2.7).
  2. "Every arithmetic assertion extractable from ooTest passes" — same
     blocker; those are Rexx programs.
  3. "ANSI X3.274 arithmetic test vectors pass" — no vectors exist anywhere
     in the repo. Needs sourcing or a documented substitute. This one is NOT
     interpreter-blocked; it could run at the Number level today.
The parent plan states the premise itself at line 40: the parity gate applies
"from Phase 2 on, once there is a real interpreter to measure." At Phase 2
there is not.

How this reframes Task E rather than excusing it: the benchmark is the same
directional Rust-API-vs-interpreted-C++ comparison Phase 1 used and gave a
looser 1.5x bar for. But the asymmetry runs the OTHER way here — the Rust
side does strictly less work — so 1.72x is a LOWER BOUND on the real gap.
Once dispatch exists and pays parse/lookup costs, end-to-end will be worse.
The failure is real and if anything understated; it is not a gate artifact.
Recommendation: pursue Task G, defer criteria 1-2 to Phase 4 with the gate
text corrected, and decide criterion 3 separately since it is not blocked.
Task C: fix round 2 verified by controller and committed as e4fb4ed1.
Both overflow repros correct; rexx-num tests green in release (format 40/40,
2.93s so the big-allocation cases really run); all four sets still 0.
The fix was more than a widening: padding no longer travels through
Number::exponent at all (that field is i32 by crate-wide invariant and a
places count near u32::MAX needs an exponent no i32 can hold). Helpers now
return unchanged when decimals already fit, and render_integer_padded pads
from the original u32 in u64/usize. pad_to_exponent removed as dead.
Task C: scoped re-review of round 2 dispatched to the original reviewer,
asked specifically whether the data-flow change moves a rounding point and
whether the i32-carry-path safety argument holds.
Note: the implementer flagged the concurrent lib.rs diff as not its own
rather than claiming it. Correct behaviour in a shared worktree; that diff
is Task F's.
Gate criterion 3 (ANSI X3.274 vectors) investigated: no vector files exist
anywhere in the tree, and the session is offline so they cannot be fetched.
Note the criterion partly answers itself — the plan's own rule is that where
the standard and ooRexx disagree, the interpreter wins. So ANSI vectors are a
secondary check by construction, and the ~200k-case differential corpus tests
against the authoritative oracle directly. What the vectors would add that
the corpus does not is an enumeration of the deviations, which is exactly
what the criterion asks to document and what cannot be produced without them.
Option for criterion 2 that does NOT need an interpreter: rexx-extract
already pulls programs out of ooTest. Simple assertions of the form
assertEquals(<literal>, <literal><op><literal>) could be mechanically
rewritten into differential cases and run through the existing harness. That
would cover a real slice of criterion 2 at Phase 2 rather than deferring all
of it to Phase 4. Real scope though — not started, flagged as an option.

Task F: complete (commit b9faa2d0). All three defects fixed with no clamping;
diffs are minimal and commented. Controller-verified: 134 tests, clippy clean,
and SEVEN differential sets at 0 — fmt 1800, fmt2 6720, fmt3 12136, fmtedge
640, pow 2112, muldiv 17424, addsub 8712 = 49,544 cases. pow.rs changed so the
arithmetic sets were re-run, not assumed.
Task F: the implementer reported case counts that did not match the ones it
was given instead of writing the mismatch off as noise, which exposed a
defect in the CONTROLLER's own earlier commit f360bb35. That commit claimed
to capture four curated generators; fmt3 and fmtedge reproduce byte-for-byte
(checked with diff) but fmt and fmt2 are reconstructions emitting 1800/6720
against the originals' 1116/7080. Both reconstructions were then run against
the oracle and sit at 0, so they are sound sets, just not the same sets.
Docstring corrected and the claim withdrawn in 579dfdc5. Lesson: that commit
verified the two artifacts it wrote and asserted the two it had not.
Tasks G and D dispatched in parallel — file sets are disjoint (G: muldiv.rs,
pow.rs; D: lib.rs, settings.rs, format.rs, Cargo.toml), each dispatch names
its permitted files and forbids the other's.

Task C: COMPLETE. Round 2 re-review clean — both Important findings resolved
with an argument from the code rather than from the differential sets. The
equivalence proof: padding is adjusted-invariant, since appending k zero
digits while lowering the exponent by k leaves exponent+len-1 unchanged, so
every trigger decision, carry re-derivation and before-oversize check
computes identical values. The i32 carry path holds trivially: the deep path
needs target > n.exponent with target <= 0 always, and anything strictly
between an i32 and 0 fits an i32. pad_to_exponent confirmed unreferenced.
Task C: minor (deferred) — the saturating_sub and Some(0)=>None branches
silently absorb a violation of the "already cut to <= after decimals"
invariant. A debug_assert would make a future violation loud.
Task C: minor UPGRADED by the controller to must-fix-before-phase-close —
the three new boundary tests peak at 12.5 GB RSS. Measured:
`cargo test -p rexx-num --test format` is 3.0s wall but 12,491,740 KB peak.
CI runs ubuntu-24.04 and windows-2022, and bsd.yml runs its suite inside a
VM hosted on ubuntu-24.04 — that VM will never have 12.5 GB. Not breaking
anything today because no workflow runs `cargo test` yet; the Rust tree is
not wired into CI at all. Fix when tests/format.rs is free (sdd-errors has
it in scope right now): #[ignore] the gigabyte cases with a comment saying
why, and keep a cheap small-magnitude test on the same code path.
Separately worth noting for the phase gate: gate criterion 5 (clippy clean,
zero unsafe) is only ever checked by hand today, since CI does not build or
test the Rust tree.

Task G: complete (commit ec5f5626). 1.9666 s -> 1.4067 s, -28.5%, gap to C++
1.70x -> 1.22x. Controller-verified: bench re-run at 1.4095 s with the
equivalence assertion passing every sample; 40,000 fresh cases on unused seed
20260728 at 0; 17,101 division-only cases re-run under the DEBUG build with
the never-overshoot assert armed, 0 and no assert fired.
Task G: post-change profile verified independently — long_divide 26.29%,
mul_magnitudes 18.75%, libc alloc/memmove ~15.2% summed. The curve is now
FLAT: no single hotspot remains, so the next 18% needed for parity is spread
across division, multiplication and allocation, which is representation-level
work touching D1. That is materially different from Task G, which was one
48% hotspot with the fix already sitting in the tree.
DECISION NEEDED (surfaced to the human): keep optimising toward parity, take
one bounded pass at the ~15% allocation share, or record 1.22x as debt and
re-measure at Phase 4 — which the parent plan already schedules for D1
("re-measure at Phase 4 when dispatch exists", line 40).

Perf gate DECIDED by the human: record 1.22x as debt, do not keep optimising.
Written into d1-decision.md as a Phase 2 addendum (commit aace6762), with the
caveat that 1.22x is a LOWER BOUND — it times Rust arithmetic alone against a
C++ figure that includes parse/dispatch/lookup, so it widens once Phase 3-4
add those. Re-measure at Phase 4, which D1 already schedules.

Task D: complete (commit 5de2d330). Controller-verified all message texts
against build/bin/rexx, 147 tests, clippy clean, differential sets 0.
Task D: controller nearly recorded a false finding here. The implementer
claimed 25.011 preserves case; my first probe showed `numeric form bogus`
reporting "BOGUS" and looked like a contradiction. It was not — a bare symbol
is uppercased by the TOKENIZER before the message ever sees it.
`numeric form value 'bogus'` reports "bogus" and 'BoGuS' reports "BoGuS".
The probe that only used a bare symbol could not tell the two apart. Same
rule as the E-notation and bit-order mistakes: rule out the alternative.
Task D: the bare-42/26 fallback gap is MY constraint's doing, not the
implementer's — I fenced it out of muldiv.rs/pow.rs while Task G was in
flight. Those are free now; follow-up dispatched to the same agent.
Controller fix (commit 2654fbe7): the three boundary tests peaked at
12,491,740 KB together because cargo test runs concurrently. Now #[ignore]d;
peak 128,976 KB, file 3.0s -> 0.21s, and all three still pass under
--ignored. Ignored rather than deleted — nothing cheaper reaches that code
path, since any input big enough to exercise it produces a result that big.
Task D review dispatched, pointed at the pre-carry exponent logic, the
unit-Copy -> String-carrying-Clone API change, and &N substitution edges.

Task D review: NOT approved — 1 Critical, controller-reproduced.
The diff removed the post-resolve exponent-width check and replaced it with a
pre-carry one. The C++ checks TWICE: NumberStringClass.cpp:2057 pre-carry and
:2190 inside the [bugs:#1474] post-rounding recheck. With only the first, a
carry that widens the exponent past expp goes unreported.
  9|FORMAT|9.996E99||0|2|         oracle <E93>, rust 1E+100
  15|FORMAT|9999999999.6||0|1|10  oracle <E93>, rust 1E+10
The second is worse: pre-carry state is plain, so initial_eng_exp is None and
nothing checks at all.
THE SCOPE LESSON: Task D's brief was "wire errors to the message table". It
silently relocated a validation. A task that is supposed to change how
something is *reported* changed what is *detected*, and the task review is
what caught it — the differential sets could not, because the successful
render path agrees at the same boundary and only the error path diverges.
Worth carrying into future briefs: say explicitly when a task must not move
behaviour, and diff for moved checks, not just added ones.
Controller built the regression corpus BEFORE dispatching the fix: new
`fmtcarry` set, 15,840 cases, committed FAILING at 148 divergences (64190235)
so the fix has something to prove itself against. All 21,296 earlier FORMAT
cases missed this.
Task D fix round 1 dispatched to sdd-errors, queued behind its sub-message
follow-up, with format.rs added back to its permitted files.
Also sent: the Minor from the same review — lib.rs:372 `substitute` replaces
sequentially and re-scans injected text, so substitute("&1 &2", ["&2","X"])
makes both X. Unreachable today but goes live exactly when the sub-message
work starts substituting operand text.
Review recommendation NOT yet acted on (controller undecided): carry
substitution values and render in message() on demand instead of storing a
rendered String. Reviewer notes the interpreter exposes them separately —
condition('o')~additional is ["5","10"] for 33.001 — and a rendered String
cannot be un-spliced. Decide after the sub-message work lands, since that
changes what the right shape is.
Controller independently confirmed the review's C++ claim before trusting the
fix direction. interpreter/classes/NumberStringClass.cpp (note: classes/, not
runtime/ as the review cited) has TWO Error_Incorrect_method_exponent_oversize
raise sites — :2059 pre-carry and :2192 inside the [bugs:#1474] block opening
at :2128 — plus Error_Incorrect_method_before_oversize at :2300. The second
check is `if (exponentSize > mathexp)` and it runs AFTER mathRound() at :2126,
which is why its substitution carries the rounded value's trailing zeros
rather than the trimmed form. Every point of the review's account held up.
Task D follow-up: complete (commit 4f022ad2). Overflow/NotWholeNumber split
into variants with real sub-messages: 42.901/902/903/001, 26.008/011/012.
Controller re-provoked each against build/bin/rexx. Verified the claim I most
doubted — 42.901's "exceeds 9 digits" stays 9 at DIGITS 9, 15 AND 20, so it
really is DEFAULT_DIGITS and not the active setting.
Documented gap, not a defect: 42.001/26.008 substitute the operand as typed
in Rexx SOURCE ("1e10" reports "1E10"), which Number has normalised away and
cannot recover. Rendering now uses full stored precision, fixing truncation
but not spelling.
Count discrepancy resolved in the controller's favour: the agent reported
fmtcarry at 460/15840, it is 148. Re-measured after rebuilding at its commit;
the Rust output is byte-identical to the run that created the set (0 differing
lines). Told the agent to re-run with exact commands before starting, and to
escalate immediately if it still gets 460 — a non-reproducible oracle capture
would be a far worse problem than the defect itself.
Only Phase 2 item outstanding: Task D fix round 1 (the post-carry
ExponentOversize check + the single-pass substitute Minor).
Task D fix round 1: verified and committed as 881a01cb. fmtcarry 148 -> 0
over 15,840 cases; all EIGHT sets at 0 (fmt 1800, fmt2 6720, fmt3 12136,
fmtedge 640, fmtcarry 15840, muldiv 17424, addsub 8712, pow 2112 = 64,752
cases); 158 tests, 3 ignored (mine), clippy clean; my #[ignore] markers
survived untouched.
Controller verified the reviewer's "unreachable" call on substitute's greedy
digit consumption rather than accepting it: the generated table uses only &1
through &4 across all 704 messages, and the grep used would have surfaced an
&N followed by a literal digit. Safe.
Note the implementer's own account: its first hand-trace of the C++ carry
behaviour was WRONG (assumed grow-not-bump) and it only settled the matter by
reading NumberStringMath.cpp:315 rather than continuing to guess. That is the
same discipline this phase has needed repeatedly, and it self-reported it.
Re-review dispatched, asked to check the gating against :2192's reachability
rather than merely against my two examples, and to check the trailing-zero
STORY not just the output — a right answer for a wrong reason breaks the next
time someone edits nearby.

Count discrepancy CLOSED: the agent had counted every diff line (header, <,
---, >) rather than `^<`. Re-ran with the exact commands and got 148, matching
the controller. No reproducibility problem with the oracle captures, which
was the real worry. Stale "460" corrected in task-D-report.md.

API DECISION MADE (controller ruled, was deferred): errors must carry the
substitution VALUES plus (major, sub) and render in message() on demand.
Storing a pre-rendered String is wrong.
The reason is conformance, not style, and it is now measured rather than
argued. `condition('o')~additional` returns an Array of the raw substitution
values, separate from `~message`:
    33.001 -> [5, 10]        93.942 -> [123456, 2]
    42.003 -> []             26.008 -> [1.5]
Note 42.003 yields an EMPTY array, not .nil, so even a no-substitution error
carries one. A Rexx program can read these directly, so an error that only
holds spliced text makes condition('o')~additional unimplementable — the
values cannot be recovered from the rendered string.
Do this reshape now, while rexx-num has no callers. Dispatch AFTER the Task D
re-review returns: the reshape touches lib.rs, settings.rs, format.rs,
muldiv.rs and pow.rs, and the review is looking at format.rs right now.
Task D: COMPLETE. Fix round 1 re-review APPROVED, both findings resolved,
nothing new. Quality of the review worth recording:
 - Gating traced gate-by-gate against NumberStringClass.cpp:2082-2196, with
   new probes beyond the controller's two cases, including an ENGINEERING
   group-crossing carry (format(9.99996E101,,0,2) at d9 -> 93.941
   `Exponent of "1.00" is too large for 2 spaces.`) that neither new test
   covers and both sides match.
 - Derived that the eng_exp0.is_some() arm can never be the SOLE cause of an
   error: a carry only moves a negative adjusted exponent toward zero, so
   exp2 never needs more digits than exp0, which already passed the
   pre-check. Its lack of dedicated coverage is therefore fine.
 - mathRound story confirmed by reading NumberStringMath.cpp:315-357 — half
   up on the single first-dropped digit, full carry sets leading 1 and bumps
   the exponent with the count fixed, exactly round_to's insert/pop/+=1 at
   lib.rs:408. Right answer for the right reason, which is what was asked.
 - Tests verified BY MUTATION: disabling the pre-check fails 4 tests,
   removing the post-check fails exactly the 2 new ones, tree restored after
   each. Both checks independently load-bearing and independently pinned.
Recorded behaviour change, accepted: substitute's greedy digit consumption
means `&1` followed by a literal digit now passes through where the old code
substituted. Unreachable per the &1-&4 table scan, and the better reading.

DEFERRED-MINOR TRIAGE (controller, for the final review):

RESOLVED WITH EVIDENCE — Task A's open concern, "pow's final check_range has
no C++ counterpart". It has one: NumberStringBase::checkOverflow() at
interpreter/classes/NumberStringClass.cpp:316. Same test and the same
asymmetry our lib.rs documents — adjusted exponent (numberExponent +
digitsCount - 1) against MAX_EXPONENT for the top, raw numberExponent against
MIN_EXPONENT for the bottom. It is a general helper rather than something
inside the power routine, which is why reading the power code did not find
it. 180 targeted boundary cases (powers at 1e500000000, 1e999999999,
1e-999999999 and neighbours, exponents -10..10) at 0 divergences. It also
corroborates the sub-message work independently: 42.901 substitutes the
adjusted exponent, 42.902 the raw one, exactly as reported.

STILL OPEN, ranked:
1. Task C — format_with takes Option<u32> but the interpreter accepts values
   above u32::MAX (expp=4294967296 returns a 4,294,967,305-char result).
   Unreachable until a dispatch layer exists; that layer must not narrow.
2. Task C — the saturating_sub and Some(0)=>None branches silently absorb a
   violation of the "already cut to <= after decimals" invariant. A
   debug_assert would make it loud. Cheap, do it in a cleanup pass.
3. Task A — subtract_multiple (moved to muldiv.rs by Task G) merges the C++'s
   two borrow branches into one formula with no comment saying so. Verified
   correct; one comment would save a future auditor the re-derivation.
4. Task B — .nil comparison special-casing unimplemented. Unreachable from
   the string-only path; revisit when compare meets the object layer.
5. Task B — compare() does not re-assert fuzz < digits. Enforced upstream in
   Settings, consistent with the crate's other leaf entry points. No action.

NOT A DEFECT, needs a project decision at branch-finish time — Task B's
cargo-fmt minor. The workspace is not rustfmt-formatted: 159 diffs at the
default max_width, and still 98 at max_width=125, so line width is not the
driver and the divergence is real. There is no rustfmt.toml. Pre-existing
files nobody touched this session are included, so this is not Phase 2's
doing. Either run `cargo fmt` once as its own commit before merge, or state
that the tree is not fmt-clean. Gate criterion 5 names clippy, not fmt, so
this is hygiene rather than a gate item.
Error reshape: complete (commit 5ab9b1c0). Typed fields, variants split rather
than tagged, additional() in XML Sub position order, message() literally
error_text(major, sub, additional()), with a per-type test asserting every
additional() value appears in the message so the two cannot drift.
FULL GATE VERIFICATION RUN by the controller — all ELEVEN sets regenerated
from the committed generator and re-run against build/bin/rexx from scratch:
  addsub 8712, addsub2 8112, muldiv 17424, md2 20184, pow 2112, cmp 32368,
  fmt 1800, fmt2 6720, fmt3 12136, fmtedge 640, fmtcarry 15840
  = 126,048 cases, 0 divergences. 166 tests, 3 ignored, clippy clean.
Note the agent self-reported a bug in one of its OWN probes (uninitialised
variable) caught before it drew conclusions from it. Worth recording as the
behaviour to want.
Error reshape review: APPROVED, zero findings. Task D fully complete
(implementation + sub-message follow-up + Critical fix round 1 + reshape, all
reviewed).
Confirmed a suspicion the controller raised in the dispatch: the "every
additional() value appears in message()" test does NOT pin ordering, because
message() derives from additional() so a wrong order corrupts both
consistently and contains() still passes. Its real job is arity drift. Order
is pinned by the ordered-vector asserts and exact-string tests instead, and
the reviewer then verified interpreter order LIVE with 7 fresh probes, all
matching item for item. With the controller's 4 earlier probes, all ten
substitution-carrying variants plus the empty-array case are covered.
Concrete finding worth keeping: 42.001's additional is [100, **, 999999999] —
the OPERATOR is its own substitution item, XML <Sub position="2"
name="operator"/>.
Corroborates the controller's own C++ read: checkOverflow at :321/:327 passes
DEFAULT_DIGITS as a real second reportException argument, so the two-item
["exp","9"] shape is the interpreter's arity rather than an invention. A third
raise site at :410 is unreachable for anything check_range admits.
Holding the two cheap deferred minors (format.rs debug_assert, muldiv.rs
borrow-branch comment) until the whole-branch review returns, so its findings
and these land in one cleanup pass rather than moving files under it.

WHOLE-BRANCH REVIEW: NOT FIT TO MERGE until C1 fixed. (Agent went idle without
messaging; report was on disk at branch-review.md. Check the file, not the
notification.)
It independently re-verified and found clean: 1,639 error-boundary cases,
23,904 FORMAT/TRUNC boundary cases, 135,000 NUMERIC FUZZ differential cases
(a real gap — no curated set has a fuzz column), 60,000 randomised arithmetic
on unused seeds, 30,000 carry-biased FORMAT. It also checked every error
variant's sub-code, message text and additional() array against condition('o')
— all eleven byte-identical — and found no lying C++ citation anywhere.

C1 (CRITICAL) IS THE CONTROLLER'S DEFECT. The DIGITS/FUZZ rule is not a fixed
cap at 999999999. It is requestUnsignedNumber(setting, number_digits()) —
NumericInstruction.cpp:103 for DIGITS, :139 for FUZZ — so the new value must
fit within the DIGITS CURRENTLY IN FORCE, up to MAX_WHOLENUMBER
999999999999999999 (Numerics.hpp:86). Verified live:
    from d3  digits 1000                 E26      (4 digits > 3)
    from d1  digits 10                   E26
    from d9  digits 1000000000           E26
    from d10 digits 1000000000           OK
    from d18 digits 999999999999999999   OK
My Task F brief specified the wrong rule because I probed ONLY from the
default DIGITS 9, where a fixed 999999999 cap and the real rule agree on every
value I tried — 999999999 is nine digits, the rejected ones are ten. The two
rules coincide there by accident. An implementer implemented my wrong rule
faithfully; a task review approved it; only the whole-branch review caught it.
FOURTH instance this session of "a probe must rule out the alternative", and
the first one that shipped code. Also implies Settings::digits must be u64.

I1 IS ALSO THE CONTROLLER'S. A third u32->i32 narrowing at muldiv.rs:177,
missed by my own sweep because I piped the sweep through `head -30` and
treated the truncated output as complete. Second lesson: a truncated search is
not a search.

Task H dispatched (fable) with all findings. The brief explicitly tells the
implementer to re-derive the rule from the interpreter rather than trust my
table, and says why.

Task H: complete (commit e91a333d). Controller-verified: 170 tests, clippy
clean, all eleven sets re-run at 0 across 126,048 cases after a widening that
touched 17 files, plus six interpreter probes of the rule itself.

THE RULE TOOK THREE ATTEMPTS, EACH MORE COMPLETE THAN THE LAST:
  1. Controller: "fixed cap at 999999999."  WRONG. Probed only from default
     DIGITS 9, where the cap and the real rule coincide.
  2. Branch reviewer: "must fit within the DIGITS currently in force."
     Right but INCOMPLETE — integer dimension only.
  3. Implementer (probed from 11 starting DIGITS): round the candidate at the
     DIGITS in force, remaining decimals must reduce to zero, result
     <= 10^min(D,18)-1. Verified live by the controller:
        d4  digits 999.9999       -> OK, sets 1000  (carry through nines)
        d9  digits 0.99999999995  -> OK, sets 1
        d3  digits 999.6          -> E26 (carry one position too wide)
        d3  digits 99.6           -> E26 (decimal does not reduce to zero)
        d5  digits 12.5           -> E26
     This also kills the format-string approach: 0.99999999995 at DIGITS 9
     renders "1.00000000", a decimal point in a case the interpreter ACCEPTS.
     unsignedNumberValue/checkIntegerDigits/createUnsignedValue ported digit
     for digit, pinned by a 1,056-case differential comparing outcome,
     sub-code AND substitution values.
The lesson is not "probe more". Each of the three probed. It is that a probe
set drawn from one dimension cannot reveal a second dimension exists.
Re-review dispatched, told explicitly that both earlier statements of the rule
were incomplete and to assume a third dimension exists.
Implementer self-caught a defect its own widening introduced (2*expt overflow
at the low-end display trigger, saturating_mul, pinned by a debug-failing
test) and audited all 100 cast hits without truncating the output — the
specific failure that made the controller's own sweep miss muldiv.rs:177.

Task H re-review: rule confirmed correct. C++ port read line by line
(unsignedNumberValue :632, checkIntegerDigits :937, createUnsignedValue :788,
maxValueForDigits/validMaxWhole Numerics.cpp:59-77) and found faithful, two
structural gaps both traced unreachable. 39,916 settings cases + 180,763
post-widening cases. I1/I2/M1/M2 all genuinely resolved. Three new findings,
all controller-verified:
  H1 Important PRE-EXISTING — Number::parse rejects blanks/tabs between sign
     and digits; NumberStringClass.cpp:1289-1295 skips them. '+ 3'+0 = 3,
     '+ .5'+0 = 0.5, while '+ 3 e2' and '3 4' are 41 on BOTH sides. Invisible
     to all 126,048 cases because no generator emits a blank after a sign.
  H2 Important INTRODUCED — muldiv.rs:296 adds a now-saturated `want`;
     div(123456,2,u64::MAX,IntegerDivide) panics in debug. Saturation was
     added at the producer, this consumer still adds.
  H3 Important ENABLED — the 10^18 ceiling makes inexact division
     non-terminating. Interpreter fails fast: after stepping DIGITS 9->18->
     999999999999999999, `1/7` gives rc=5 (Error_System_resources,
     rexxmsg.xml:145). Rust would grind out 10^18 digits.
CONTROLLER PROBE ERROR, same trap AGAIN: my first H3 probe set
`numeric digits 999999999999999999` directly from the default and got E26 —
because 18 digits do not fit in DIGITS 9. I had just implemented that rule.
Had to step 9 -> 18 -> 10^18. Recording it because the failure mode is not
ignorance of the rule; it is not applying a known rule to my own probe.
Also noted: the interpreter's own message for rc=5 renders as "The NIL
object", which looks like an interpreter-side message-lookup bug. Not ours,
but do not trust condition('o')~message for that code.
Fix round 1 dispatched. Implementer allowed to edit gen-curated-sets.py for
H1 specifically (new set, not a change to an existing one, so counts stay
comparable) — reversing an earlier blanket instruction.
Controller probed for a SECOND lexical gap beyond H1, on the reasoning that a
parser accepting one undocumented form probably accepts others. Read the C++
validator at NumberStringClass.cpp:1270-1382 (skip blanks/tabs, optional sign,
skip blanks/tabs, optional period, digits, optional period+digits, optional
E/e with optional sign and >=1 digit, skip trailing blanks/tabs, must end).
Hypothesised from that reading that a bare "." would parse, since the digit
loop can match zero digits and still reach the valid return.
WRONG — '.' is E41. That routine is not the only gate; the conversion path
enforces a digit requirement the reading did not reach. Tested rather than
reported, which is the only reason it did not become a false finding.
Result is still useful as a NEGATIVE: on 9 hand-built lexical edge cases
('.', '+.', '-.', '.e1', '5.', '.5', '  .  ', '+ 3', '+ .5') the Rust parser
matches the interpreter on 7 and diverges ONLY on the two H1 forms. So H1's
scope is exactly "blanks/tabs after a sign" and the period/exponent dimension
is clean. That bounds the fix.

Task H fix round 1: complete (commit 7f664aa7). All twelve sets at 0 —
original eleven unchanged at 126,048 plus signblank 2,320 (480 of them error
rows) = 128,368. 176 tests, clippy clean.
H3's root cause is better than the "find a threshold" the controller asked
for. NumberString::Division allocates 3*((digits+1)*2+1) bytes UP FRONT
(NumberStringMath2.cpp:401-417, past the 48-byte fast buffer) and error 5 is
that allocation failing (RexxMemory.cpp:1266). Controller-verified the
discriminating evidence: at DIGITS 999999999999999999, `4.0 / 2` and
`123456.0 % 2` are BOTH rc 5 — sized before the operands are examined — while
`0.001 // 7` succeeds because the no-integer-part early-out runs first, and
`4 / 2` succeeds via a RexxInteger fast path above this crate. The fix mirrors
the cause (fallible reservation of the same size, same place, same cutoff)
rather than guessing a digit bound.
Bonus defect the implementer found while fixing H1: str::trim treats LF, CR,
VT and FF as blanks; the interpreter's class is exactly {space, tab}.

TWO MORE CONTROLLER PROBE ERRORS this round, both caught by looking at the
output rather than by being careful:
  1. `numeric digits 1000000000000000000 - 1` — interpret evaluates the
     expression, yielding 1.0E+18, which the rule then rejects. Needed a
     literal.
  2. `'0a'x || '3' + 0` — in Rexx `+` binds tighter than `||`, so this
     concatenated LF onto ('3'+0) and never converted anything. It printed a
     literal newline, which is what gave it away. The corrected probe
     confirmed the implementer exactly: space and tab accepted, LF/CR/VT/FF
     rejected.
Running tally of controller probe errors this session: 5. Every one produced
a plausible-looking wrong answer, and every one was caught by re-reading the
output rather than by suspicion.

RE-REVIEW OF FIX ROUND 1 DID NOT RUN. review-branch failed with "session limit
· resets 11:50am (Europe/Zurich)", not with a verdict. Phase 2 therefore closes
with its final re-review OUTSTANDING, not clean.
What IS verified, by the controller directly: all twelve differential sets at
0 (128,368 cases), 176 tests, clippy -D warnings clean, and each of H1/H2/H3
checked against build/bin/rexx on its discriminating cases —
'+ 3'+0 = 3 and '+ .5'+0 = 0.5; LF/CR/VT/FF rejected while space/tab accepted;
4.0/2 and 123456.0 % 2 both rc 5 at DIGITS 1e18 while 0.001 // 7 succeeds.
What is NOT verified: whether the fixes introduced anything new. That is
exactly the category that produced H2 (introduced by the previous fix) and the
2*expt overflow (introduced by the widening). Two of the last three rounds
introduced a defect while fixing one, so the base rate here is not low.
ACTION REQUIRED when limits reset: re-run the scoped re-review over
review-H-round1.diff. The dispatch message is already in review-branch's
inbox.
