# Task 13 review -- the three access scopes, and the one chokepoint

Reviewed at `fd04468f0` (code identical to `6d12c033d`), diff
`review-1a2ec9664..6b1ef369d.diff`, report `task-13-report.md`, brief
`task-13-brief.md` with the controller's two corrections already applied.

## Verdicts

**Spec compliance: PASS.** Every "done when" clause is met and I re-ran each one. Both `PRIVATE`
programs match the oracle byte for byte on all three descriptors and both engines -- the refusal
(`say .K~m`, rc 159, `97.2 Object "The K class" cannot accept private message "M" from this
context.`) and the twin that must keep answering (`say .K~outer` over `self~m`, rc 0 `inner`), with
the keyword-free twin unchanged. The chokepoint test still holds (5 passed, exit 0) and states both
what it counts and what would let a second dispatch path pass it. The negative control the plan asks
for reddens the outside-scope case, and so does its inverse. The three limbs sit where the corrected
plan puts them: `PRIVATE` and `PACKAGE` in `Interp::resolve`
(`rust/crates/rexx-exec/src/dispatch.rs:1006`, `:1049`), `PROTECTED` inside `seam::clear` via
`Interp::method_is_protected` (`:957`). The corpus grew 156 -> 159 and is green, the exclusions
entry's `PRIVATE` and `PROTECTED` limbs came out in the same commit as the code, and the sitting is
committed with the duplicate-key check at `0`.

**Task quality: PASS WITH REQUIRED FIXES.** The engineering is careful and the measurements I
re-derived independently all reproduced -- including the send-path figures, to a hundredth of an
instruction per send. Two things must change before this is accepted, both small: one false
"nothing covers X" claim (F1), and one limb the task changed and left with no instrument of any kind
while its report says nothing about that (F2). Neither is a behaviour defect.

## The performance guard: what it compares, and whether Task 13 breaches it

**Not breached, on either reading, and every figure in the question is a `cycles:u` median rather
than an `instructions:u` one.**

**What the guard compares.** `rexx-arms` launches both binaries' both engine arms at two problem
sizes, interleaved round by round, and emits reductions; nothing in the tool applies a threshold.
The figure that compares this task's build against the pin is
`scope = across_builds`, `build = pinned>head`: head's absolute count divided by the pinned
binary's, per round, reduced to a median
(`rust/crates/rexx-bench/src/arms.rs:493`, emitted at
`rust/crates/rexx-bench/src/bin/rexx-arms.rs:269`-`:283`). The per-build `per_pass` rows carry the
same comparison with the fixed cost differenced out. Both are against
`bench-baselines/pinned/rexx-run-15a1ffa98`, so both measure **drift accumulated by the whole phase
since the pin**, not this task's delta. The 1% line exists only as prose in the plan's Global
Constraints, on `instructions:u`; the only 1% in the bench crate's code is the wall-clock suite's
sensitivity control (`rust/crates/rexx-bench/src/lib.rs:207`-`:218`), a different instrument.

**The quoted medians are `cycles:u`.** Grouped by (`task`, `commit`, `axis`, `build`, `scope`, `arm`,
`size`, `instrument`) -- the eight columns that identify a row -- every value in the question sits in
an `instrument = cycles:u` row: at `6d12c033d`, `emptyloop`/`tw` 1.047113 (small) and 1.046651
(large), `compound`/`tw` 1.024528, `varlookup`/`tw` 1.024307, `strings`/`ir` 1.021392,
`alloc4c`/`ir` 1.018610; at `b84685f44`, `compound`/`tw` 1.029007, `compound`/`ir` 1.021314,
`strings`/`ir` 1.020598. The plan pre-declares that class of figure as not a result on its own, with
+2.020%, -5.497% and +2.792% precedents where `instructions:u` stayed flat.

**The `instructions:u` rows.** Task 13's widest is `strings`/`ir` at 1.009685 (`across_builds`) and
1.009684 (`per_pass`), i.e. +0.97%: under the line. Nothing else clears +0.7%.

**Accumulated drift, not this task's delta -- checked against earlier tasks, as instructed.**
`strings`/`ir` reads exactly 1.009684 for tasks 9, 9-fixround-1, 10, 10-fixround-1, 11,
11-fixround-2, 12, 12-prev, 12-fixround-1 **and** 13: the drift entered at Task 9
(head per-pass 5369.5 -> 5421.5) and every later task inherits it. And earlier tasks already crossed
1% in `instructions:u` under that reading: Task 11 at `f4b21eadb` reads `strings`/`tw` 1.013710 and
`alloc4c`/`tw` 1.010758, Task 9 at `4e9a0369f` reads `strings`/`ir` 1.010615. So a per-task pass/fail
bound at 1% against the pin would already have failed twice before Task 13 -- the accumulated-drift
reading cannot be a gate, which settles the ambiguity the way the caller's own test predicts.

**Task 13's own contribution to the guard's axes is zero.** Its head per-pass `instructions:u`
medians agree with Task 12's on all six axes and both arms to within 0.003 instructions per pass
(e.g. `strings`/`ir` 5421.525224 and 5421.525320 against Task 12's 5421.524983; `arith`/`tw`
26490.778704 against 26490.775940). That is the figure that actually bounds this task, and it is
absent from the report.

**So the rule, stated so the plan and the report can say the same thing:** the >=1% line is a
per-axis, per-arm reading of `instructions:u` **against the pin**, i.e. of the phase's accumulated
drift, and what it triggers is a narrative obligation ("say whether it is the change or the layout"),
not a failure; it is not a bound on one task's delta and it is not enforced by any code. A task's own
cost is read by comparing its `head` rows against its predecessor's, which the TSV supports because
every task measures the same six axes against the same pin.

## The send path: verified independently, and the deferral was right

I rebuilt both sides in one tree to remove the code-placement question the way the report did: the
diff touches only `dispatch.rs`, `error.rs` and `lib.rs` of `rexx-exec`, so reverting those three
files to `1a2ec9664` yields the base binary from the same paths and the same toolchain. Seven
interleaved rounds per cell, `perf stat -e instructions:u`, `REXX_ENGINE=ir`, 3,000,000 passes,
medians:

| program | base | head | per pass | per send |
|---|---|---|---|---|
| loop with no send | 1,410,627,399 | 1,410,627,114 | -0.000 | -- |
| one native send (`s~length`) | 6,211,013,412 | 6,253,009,234 | +13.999 | **+13.999** |
| one class-method send (`.K~m`) | 17,993,563,847 | 18,071,530,763 | +25.989 | **+25.989** |
| two class-method sends (`.K~outer` over `self~m`) | 31,388,526,833 | 31,544,571,641 | +52.015 | **+26.007** |

Whole-program ratios 1.006762, 1.004333, 1.004971 against the report's 1.006756, 1.004334,
1.004970. The report's two headline figures are correct, the cost is unconditional (none of these
programs declares a special method), and it is zero on a loop that sends nothing -- which also
confirms that the guard's six axes cannot see it, because none of them dispatches.

**Declining the cross-crate change was right.** Moving the flags onto `rexx-classes`' dictionary
entry changes another crate's returned type for a correctness task's convenience, needs its own
sitting and its own review, and cannot be sanity-checked by anything Task 13 owns. Measuring it,
attributing it to the change rather than the layout, and recording it as a concern is the correct
disposal. The residual is F4 below: the number has no committed referent.

## Findings

### Blocking

**F1. The "nothing covers X" search is false as stated, three ways.**
`task-13-report.md:268`-`:271`. The quoted command is
`/bin/grep -rain 'PRIVATE ::METHOD|PRIVATE ::ATTRIBUTE' ...`; run exactly as written it matches
**nothing**, because `/bin/grep` without `-E` reads `|` literally -- I ran it. Run with `-E` against
the base revision it returns **three** hits in the searched extensions, not one:
`rust/crates/rexx-exec/src/dispatch.rs:2522` (the removed test row),
`rust/crates/rexx-exec/src/lib.rs:1592` and `:1601` (the two producers), plus two prose mentions
tree-wide (`docs/superpowers/plans/2026-08-17-phase-5a.md:1324`,
`docs/superpowers/plans/phase-4-exclusions.txt:3327`). And the citation is wrong: the removed row is
at `dispatch.rs:2522` in `1a2ec9664`, where `:2788` is an unrelated `Error 97.1:` assertion. The
**conclusion** survives -- the two extra `.rs` hits are the producers this task changed, so no other
test or exempt list named the retired refusals -- which is why this is a false-evidence finding
rather than a coverage one. *Fix*: restate with the pattern that was actually run, the hits it
actually returns, and either the corrected line or no line at all.

**F2. `PRIVATE` on `::ATTRIBUTE` was changed in both directions and has no instrument of any kind,
and the report does not say so.** `rust/crates/rexx-exec/src/lib.rs:1592`-`:1605` retired
`a PRIVATE ::ATTRIBUTE`, which had two observable consequences, both of which I ran on the oracle and
both engines from a fresh directory:

* the refusing side now **matches the oracle byte for byte** -- `say .K~a` with
  `::ATTRIBUTE a CLASS PRIVATE` is rc 159 `97.2 ... cannot accept private message "A" ...` on all
  three descriptors, where the crate was rc 120 before. It is therefore expressible as a corpus row
  and is not one: no corpus program contains a private attribute (searched `corpus/lang/`), no gate
  probe sends one (`corpus/gate-tables/directives/attribute__private__subkeyword.rex` is
  declare-only), and no in-crate test names one.
* the allowed side swaps which loud refusal a program gets -- `say .K~poke` over `self~a` is rc 120
  `a generated ::ATTRIBUTE accessor` where it was rc 120 `a PRIVATE ::ATTRIBUTE`. The only bodyless
  accessor row in `a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run`
  (`dispatch.rs:2828`) uses the non-private form, so nothing exercises the private one.

The Global Constraint is "where a task changes what is refused, it says what instrument catches a
regression... if the honest answer is 'an in-crate test only', the task says exactly that". Here the
honest answer is *nothing*, and the report's concern 4 says only that nothing named the retired
string. *Fix*: state it, and close it cheaply -- add `say .K~a` with `::ATTRIBUTE a CLASS PRIVATE`
to `corpus/lang/method_access_private_refused.rex`'s family (the bytes match today, verified) and add
the private bodyless-attribute row to the loud test's refused list.

### Non-blocking

**F3. The performance section states its criterion twice, in two different terms, and never names
the row the rule reads.** `task-13-report.md:296`-`:308` computes a per-pass delta against 1% of the
pin's per-pass ("+52.000348 against a ceiling of 53.695249"), then concludes "no axis reached 1%
against the pin", quoting `strings`/`ir` 1.009685, which is the `across_builds` ratio. Both numbers
are right and I re-derived both, but the section does not say which TSV row carries the criterion,
does not say the figure is cumulative since the pin, and attributes to this task a 0.97% that entered
at Task 9. *Fix*: name the row (`scope = across_builds`, `build = pinned>head`,
`instrument = instructions:u`), say it measures drift since the pin, and add the comparison that
bounds this task -- head per-pass against Task 12's, which is flat to 0.003 per pass on all six axes.
The plan's Global Constraints should gain the same sentence, since "under 1% on an axis is not a
finding" does not say against what.

**F4. The send-path figure has no committed referent.** The report says it
(`task-13-report.md:448`): the three programs were written for the report and committed nowhere, so
a later task cannot tell a regression from this baseline, and the guard's axis set cannot supply one
because no axis sends a message. `bench-programs/dispatch.rex` exists and would be the axis, but its
body needs `~new`, which is 5b's -- I confirmed `.K~new` is rc 120 in both engines. *Fix* (plan
level, not this task's): commit a class-method send program under `bench-programs/` with its
`NOT_BENCHMARKED`/`PROGRAMS` entry so the number has a home, and give 5b `dispatch.rex`'s promotion.

**F5. Concern 5 says the order invariant is held by "two `debug_assert`s"; only one of them is a
demonstrated witness.** The accessor's self-check (`dispatch.rs:944`) is proved live -- see below.
The record-side assert (`lib.rs:4057`) I could not falsify from inside this crate: appending the same
row twice passes it (the check runs before both pushes, and the accessor's scan agrees on a
duplicate), and reversing the vector fires the accessor's instead. It is a guard on a cross-crate
invariant -- `ClassRegistry::next_method_id` (`rust/crates/rexx-classes/src/registry.rs:84`) minting
from one increment-only counter, which I read and confirmed -- and that is a legitimate assert, but
the report should say which of the two catches the break it describes.

**F6. Borderline: the replaced shape's measurement lives in a doc comment.**
`rust/crates/rexx-exec/src/lib.rs:2444`'s field doc carries "with the rows in a `HashMap` ... the
pass costs 580 instructions more ... the whole of it is the default hasher, four times over" -- the
cost of the shape this task's own second commit replaced. The Task 1 ruling keeps a measurement that
justifies the design *as it stands* and sends the account of what was replaced to the report. This
one is both at once, and striking the historical half leaves "sorted and searched, 23 apart" saying
less than the contract needs. Moritz's call; I would keep the comparison and drop "four times over".

**F7. Observation, pre-existing and already recorded, not this task's.** `say .K~c` with
`::CONSTANT c 5` is oracle rc 0 `5` against crate rc 159 `97.1`, identical on the base binary --
recorded at `docs/superpowers/plans/phase-4-exclusions.txt:3723` ("install_directives records no
method for a ::CONSTANT at all"). Raised only because it sits one send away from this task's
subject.

**F8. Trivial.** The staleness paragraph (`task-13-report.md:288`-`:293`) says every commit in the
list except `ee6ebbf64` and `0ce35233e` is named in `progress.md`; its own two commits, `b84685f44`
and `6d12c033d`, are not either, which is expected at report time. I ran the list from the repository
root (it is empty from `rust/`, since the pathspec is root-relative) and checked each hash against
the ledger.

## Rulings on the five flagged items

**1. Gate table C's `pubpri`, both controls -- confirmed, and nothing else moves.** I ran both.
Allowing every private send (an early `return Ok(())` in `Interp::check_private`) takes `pubpri` from
`agree` to `diverge-both` (oracle rc 159, two stdout lines; crate rc 0, three), drops the corpus to
**157 of 159** on `lang/method_access_private.rex` (stdout) and `lang/method_access_private_refused.rex`
(stdout, stderr, exit code), and fails exactly the three named in-crate tests (709 passed, 3 failed
against 712). Refusing every private send instead takes `pubpri` to `diverge-both`, the corpus to
**158 of 159** on `lang/method_access_private.rex` with first frame `73 *-* return self~m`, and two
of the three tests. Diffing the whole verdict table base against control shows **one** row changing
in each direction, `pubpri`, with the table total moving 53 -> 52 `agree`: the control reddens the row
it claims and nothing else. The row's committed `control` field
(`crates/rexx-exec/tests/gate_table_c.rs:449`) describes the mutation I ran.

**2. The attribute refusal swap -- an instrument is owed and nothing covers it.** See F2. Swapping
one loud rc-120 phrase for another does not by itself need a differential; what needs one is that the
same edit made the *refusing* side match the oracle, which turned an inexpressible refusal into a
committable corpus row that was not committed. Rule: fix by committing the row, not by arguing the
swap is invisible.

**3. The order invariant -- adequately guarded for release, and the report's account is exact.**
I reproduced all three halves with `self.special_methods.reverse()` appended to the push. Release:
corpus **158 of 159**, `lang/method_access_private.rex` differing on stdout -- a wrong answer, as
stated. Release in-crate suite: **712 passed, 0 failed**, so the lib tests are blind to it, and the
report's reason is right (every program they run has at most one special method, so the search cannot
miss). Debug: panics at `dispatch.rs:944`, `the search over the access scopes disagrees with a scan
of the same rows`. So the debug corpus gate is load-bearing, exactly as concern 5 says. I judge this
**adequate**: the accessor's self-check is a real live witness in the mode where it runs, it is
inside the accessor rather than beside it, both gate commands run the debug corpus, and the release
failure mode is a red corpus rather than silence. What would make it inadequate is the debug corpus
ceasing to run, and nothing here depends on that.

**4. `PACKAGE`'s cross-package limb -- the test exists, covers the refusal, and can fail.**
`package_scope_refuses_a_caller_from_another_package` (`dispatch.rs:2982`) calls
`Interp::check_package` with four callers: no activation, another program's package, the method's own
package, and a program caller against the interpreter's package. Turning the different-package arm
into `Ok(())` fails it with "a caller in another program's package must be refused", **and nothing
else** -- the full lib suite reports 711 passed / 1 failed and the release corpus stays **159 of
159** under that mutation, which is the proof that this in-crate test is the whole instrument rather
than a duplicate of a differential.

**5. Class-method-only limit -- restated, not implied.** The report states it in four places
(the differential preamble, `check_private`'s doc, the test's doc, and "what the checks could not
see", which spells out that a route refusing on a class method is not thereby known to refuse on an
instance one), and the exclusions entry repeats it. Verified the premise: `.K~new` is rc 120
`method "NEW" of class "Object" is not implemented` on both engines, against an oracle that answers.
The debt is joined to 5b, as asked.

**D45, site one.** `dispatch_seam` is 5 passed / 0 failed at head. The report's account of what
`dispatch_passes_through_exactly_one_chokepoint` counts matches the test
(`crates/rexx-exec/tests/dispatch_seam.rs:178`-`:212`): `seam::clear(` exactly once,
`struct Cleared(())` exactly once, `Cleared(())` exactly twice, needles built with `format!` so the
test file does not count itself. Its escape list is re-read from the file and extended by one, and I
confirmed the added one by mutation: moving the `PROTECTED` question out of `seam::clear` into
`Interp::invoke` immediately beside the `seam::clear` call leaves **all five tests passing**, exactly
as `the_protected_question_is_asked_only_inside_the_seam`'s own doc says it would. The lexical
assertion is honestly labelled as the whole instrument for a branch no program can observe.

## What I ran

Oracle throughout: `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 10
.../build/bin/rexx FILE )`, three descriptors to separate files compared with `cmp`, exit status
read separately, fresh empty directory, absolute paths, both `REXX_ENGINE` settings. Nothing from
`rust/corpus/oracle-crashes.txt`.

* Differential re-runs that matched on both engines: `.K~m` private; the sibling refusal with its
  two-frame traceback; the `UNKNOWN` fall-through; `::ATTRIBUTE a CLASS PRIVATE` from outside
  (97.2 naming `"A"`); `PRIVATE PROTECTED` from outside and from inside; `PACKAGE` same-package;
  `PRIVATE ABSTRACT` from outside (97.2 -- the access check precedes the abstract refusal, as the
  oracle orders it); `.K~outer` over `self~m` with and without the keyword;
  `CONDITION('E')` 2 for a refused private send against 1 for a name miss.
* Divergences confirmed as the report describes them: the generated-accessor refusal
  (`self~a` on a bodyless private attribute, crate rc 120 against oracle rc 0 `A`);
  `.K~method('M')~setSecurityManager(.nil)` rc 120 against oracle `1`; and
  `.K~method('M')` answering `a Method` at rc 0 on both sides, which is the stale exclusions row the
  task re-measured.
* Mutations, each backed up and restored with `diff -q`: allow-every-private-send;
  refuse-every-private-send; `special_methods.reverse()` (release and debug);
  cross-package refusal removed; duplicate row pushed; the `PROTECTED` question moved out of the
  seam. Tree confirmed clean at `fd04468f0` afterwards, corpus re-checked at **159 of 159**.
* Tables and counts: gate table C base and under both controls (full verdict-table diffs); gate
  table D (34 `agree`, 45 `diverge-both`, 45 loud, 79 rows -- the report's numbers), with all six
  access-keyword rows confirmed declare-only probes that never send; corpus 101 -> 104 entries in
  `phase-5a.txt`, 156 -> 159 total.
* Guard: `phase-5a-arms.tsv` grouped by its eight identifying columns; per-pass and `across_builds`
  ratios recomputed for every task in the file; staleness test from the repository root with each
  commit checked against `progress.md`; `sha256sum` of the pinned binary against `PINNED.md`.
* Send path: base and head `rexx-run` built from one tree, seven interleaved rounds per cell.

## What blocks and what does not

Blocking: F1 (a false search claim, conclusion intact) and F2 (a changed refusal with no
instrument, unstated). Both are edits to the report plus, for F2, one corpus row and one test row.

Not blocking: F3 (guard wording, in the report and in the plan), F4 (no committed home for the
send-path number, plan level), F5 (which assert is the live witness), F6 (a doc-comment judgment
call for Moritz), F7 and F8 (observations).
