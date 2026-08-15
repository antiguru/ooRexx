# SDD ledger — plan: docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md

Phase 4c: builtins, PARSE, and the rest of the call chain. Fifteen tasks.

Plan committed at `c22b9124`. Base commit for Task 1 is recorded at dispatch.

## Pre-execution plan review

Four independent adversarial reviewers dispatched before Task 1, on the 4b
precedent: 4b's plan was reviewed by four reviewers before execution and its
first revision was substantially wrong -- two of its four "corrections to
inherited items" were themselves wrong, its activation-indent rule was
refuted, and it omitted an entire construct.

| lens | report |
|---|---|
| re-verify every measured claim | `plan-review-measurements.md` |
| attack decisions D-R and D-P | `plan-review-decisions.md` |
| executability and interface consistency | `plan-review-executability.md` |
| checks that cannot fail | `plan-review-vacuity.md` |

Findings and their disposition are recorded below before any task is
dispatched.

### Outcome: eleven blockers, plan revised before Task 1

Both new decisions survived: **D-R STANDS-WITH-CORRECTION**, **D-P
STANDS-WITH-CORRECTION**. The architecture (fifteen tasks, one file per
family, a derived boundary) held; the task bodies did not.

The three structural defects, each fixed in the revision:

1. **Task 1's status harness never consulted the oracle.** It keyed on the
   absence of a loud message, so 66 stubs returning `''` satisfied it, and
   Task 13 -- which deletes the only producer of that message -- would have
   made deleting the whole `builtin/` tree leave all 66 rows green. Now
   differential against `build/bin/rexx`, with an assertion that the oracle
   was invoked 66 times and a falsification that mutates the interpreter
   rather than the committed file.
2. **Seven tasks had no heading.** `### Tasks 3-6, 10-12` was one section,
   and briefs extract per heading. Verified after the fix: tasks 1, 3, 6, 12
   and 15 extract 105, 34, 31, 27 and 90 lines.
3. **Every `base/bif` count was short.** `grep` here wraps `ugrep -I`, which
   silently drops files with non-UTF-8 bytes; six `base/bif` groups are
   exactly that, because they test byte conversion. `assertSame` is **6,293**,
   not 5,441. Thirteen figures moved. `/bin/grep -a` is now a global
   constraint.

Other blockers folded in: the builtin hook was upstream of the argument
evaluation it claimed to consume; `NAMES` minus `EXCLUDED_BUILTINS` is 63,
not 66 (three rows are partial); `EXCLUDED_BUILTINS` was private to a test
binary and is moved to `rexx-inventory`; `tests/corpus.rs` hardcodes two
subset paths and appeared in no task, so every 4c witness would have been
inert; three tasks edited `instruction_owner` without listing `lib.rs`;
Task 9's `loud.rs` witness is `address cmd` and must split rather than be
deleted; `BodyKey::directive` has seven construction sites and the
production one is `lib.rs:1422`, not the `#[cfg(test)]` helper the first
revision named.

Two corrections the reviewers found that change scope rather than wording:

* **Task 13 cannot raise 43.1 unconditionally.** The oracle searches for an
  external `.rex` file first -- measured, `call zorkolo` runs `zorkolo.rex`
  from the cwd at rc 0. External routine resolution is now an explicit
  Phase 7 exclusion with a corpus rule.
* **Task 13 must fail loudly on non-`::ROUTINE` directives.** Implementing
  `::routine` removes the fallback masking that no directive is ever
  installed; the oracle aborts before `main` on a bad `::class` (98.909,
  rc 158) where `rexx-run` prints `main ran`.

All eighteen line citations taken from reviewer reports were independently
re-verified against the tree before landing.

---

## Task 1: boundary infrastructure and the four attribution fixes

BASE `7d8c43db`. Implementer commit **`aa7b3505`**, working tree clean.
Reported DONE_WITH_CONCERNS. Review dispatched against
`review-7d8c43db..aa7b3505.diff`.

Implementer's figures, to be re-run by the reviewer rather than trusted:
1,031 passed / 0 failed (up from 1,020, +11 from the new binary); fmt 0;
clippy 0; `REXX_CORPUS_GATE=1` 42 of 42. Step 3's predicted result confirmed
exactly: **66 loud, 15 excluded, 0 implemented, 0 divergent**.

**The falsification that mattered worked.** A name-table classifier -- one
that runs no program at all -- left **10 of 11 tests green** and was caught
only by the assertion that the oracle was invoked 66 times. That assertion
was added because the first revision's falsification only edited the
committed data file, which cannot detect a classifier that never runs
anything. Task 1 could not run Step 4.3 (mutating a builtin's dispatch arm),
because no builtin exists yet; **it is owed by Task 2 Step 5** and was not
substituted with something weaker.

### Concerns raised, and disposition

1. **The `ADDRESS` probe was the excluded half.** `say address()` with no
   `ADDRESS` instruction returns the platform default, which is Phase 7's.
   Changed to `address zork; say address()`. Correct, and the same trap
   applies to any builtin split across phases.
2. **The `+++` evidence omitted the trace setting.** Measured: `address sh`
   + `'exit 3'` emits nothing under `trace n`; `+++   "RC(3)"` needs
   `trace e`. Added.
3. **The plan's D-R contained a false sentence** -- it dismissed the
   `>I>`/`<I<` row's "which 4c will have to meet" as being about the trace
   gate. It is not: the trace gate is the separate "THE GATE IS TWO
   CONDITIONS" paragraph, and the dismissed sentence heads "THREE MEASURED
   WAYS A ::ROUTINE ACTIVATION IS NOT AN INTERNAL LABEL'S", the strongest
   ownership statement in the file. **Verified against the parent commit and
   corrected in the plan at `0786973b`.** The claim understated this plan's
   own case.
4. **Every line citation into `phase-4-exclusions.txt` moved**, because this
   task edits that file -- one row went `:1009` -> `:1147`. Since Task 13
   edits it too, line numbers into it are structurally wrong rather than
   unlucky. **All citations converted to quoted phrases at `0786973b`**, with
   a global constraint recording why.
5. **D4's "SETLOCAL/ENDLOCAL both return 1"** holds only for the pair;
   unpaired `endlocal()` returns 0. Corrected in the exclusions file.

Controller note: `0786973b` is a plan-document-only commit made **after**
the review package was snapshotted, so it cannot shift the reviewer's diff.

### Task 1 review: spec PASS, quality CHANGES-REQUESTED

Reviewer re-ran verification itself rather than accepting the report, and
independently reproduced all three of the original design defeats as closed:
the name-table classifier fails only on the invocation count (10 passed, 1
failed), 66 silent stubs classify `divergent` not `implemented`, and removing
the loud path makes all 66 `divergent` so the later task cannot silently
green it. Clippy was re-run **cold** on a pristine scratch copy with
fingerprints wiped, per the warm-cache rule, and is clean.

### Fix round 1 -- done by the controller, not the implementer

The implementer agent died mid-round on an API session limit, having
committed nothing; the only dirty file was a controller plan edit. No damage,
nothing to recover. The round was small and was completed directly.

Commit **`9746be64`**. 1,031 passed / 0 failed / 4 ignored, fmt 0, clippy 0,
`REXX_CORPUS_GATE=1` 42 of 42. Tree clean.

**Four probes could not fail.** `TIME`'s was an identity conversion, so
`return arg(2)` classified `implemented`; `VAR`, `SYMBOL` and `GC` each
returned one constant. All four re-measured on the oracle and rewritten to
make more than one call with differing outputs.

**The executor-side assertion is the interesting one, and it is recorded as
adding no coverage.** The review found the harness passes with `run_program`
stubbed. An invocation counter cannot catch that -- a stub is still called
once per name -- so the assertion is that the executor produced more than one
distinct reply across the 66 probes. Both mutations tried against it die
without it anyway: a constant loud reply also fails
`every_loud_row_is_loud_about_its_own_builtin`, and an executor run on a
fixed program fails set equality with 66 `divergent` rows. It is kept for
diagnosis quality and for stable reach -- the loud-row test covers only
`loud` rows, so its reach falls to nothing as builtins land -- and its
comment says all of this. **Claiming it as coverage would have been this
project's own recorded defect**: "can fail" is not "adds coverage", and the
check costs one run.

**The `+++` count moved again**, from four producers to five.
`Activity.cpp`'s `displayDebug` fetches the message and calls
`displayUsingTraceOutput` twice -- `:1496` primary, `:1507` secondary -- so
there are five sites and four functions. Read directly. Both places in the
exclusions file that state the number now say which question they answer.
The ruling is unchanged; every producer is Phase 7's.

**Deferred minors:** `coverage.rs`'s new `in_scope().len()` assertion is
nearly implied by the assertions above it (review Minor 5). `RANDOM`'s probe
is intrinsically weak and the reviewer asked for no change. Both for the 4c
final review to triage.

**Next: Task 2** -- builtin dispatch, the arity table, the 40.x family. It
also owes Task 1's Step 4.3, the falsification that mutates a builtin's
dispatch arm, which could not run before a builtin existed.

---

## Task 2: builtin dispatch, arity table, the 40.x family

BASE `3a51a3b0`. Dispatched. Brief is 124 lines: 62 extracted plus the
58-line shared-facts block the controller appends.

**A plan defect found at dispatch, and fixed at `3a51a3b0` before the agent
started.** Eight builtin tasks told their implementer the shared block was
"restated in your brief". It was not and could not be -- briefs extract per
heading and no task body contained it -- so all eight would have arrived
without the dispatch signature, the argument model, the pre-existing 40.x
raisers, the allocation rule, the probe safety rules or the verify block.
Exactly the mechanism that has cost this project five decisions recorded
where the implementer never read them. The section now names the controller
as the party who makes the sentence true, and the append asserts the block
still contains `pub(crate) fn dispatch(`, `not_enough_arguments` and
`REXX_CORPUS_GATE=1` before writing, so reorganising it fails loudly rather
than shipping a truncated span.

Task 2 also owes **Task 1's Step 4.3** -- deleting `LENGTH`'s dispatch arm
and confirming its status row flips `implemented` -> `loud` on its own. That
is the falsification proving the status harness observes the interpreter
rather than a name table, and it could not run before a builtin existed.

### Task 2: complete

Commit **`c10e40df`**. Review: spec **PASS**, quality **APPROVED**, no fix
round. 1,039 passed / 0 failed / 4 ignored; fmt 0; clippy 0 warm **and cold**;
`REXX_CORPUS_GATE=1` 42 of 42. Reviewer re-ran all of it and independently
reproduced every transcript in a fresh probe directory.

**Task 1's owed Step 4.3 discharged and reproduced by the reviewer.** Deleting
`LENGTH`'s dispatch arm gives 11 tests run / 10 passed / 1 failed, exactly one
row flipped -- `LENGTH: committed implemented, measured loud`, rust exit 120
against oracle `"6\n"` exit 0. Not a "matched nothing" green. The status file
observes the interpreter.

**Three brief corrections, all re-measured by the controller, folded into the
plan at `fc23d6ca` before the family tasks run:**

* A negative where a non-negative is required is **93.923 at rc 163**, not a
  40.x error at rc 216. The plan had listed it among the 40.x probes, so all
  seven family tasks would have shipped the wrong code and exit status with
  tests pinning both.
* **40.5** was missing: `substr('abc',,2)` is "Missing argument ... argument 2
  is required", distinct from 40.3.
* Trailing omitted arguments are not arguments -- `q(1,,2,,)` gives `arg()` =
  3 -- so only interior omissions reach `dispatch`.

Also measured: a quoted target reaches the builtin table but is
**case-sensitive** (`"LENGTH"('abc')` is 3, `"length"('abc')` is 43.1 rc 213).

**A false comment corrected at `0c58926e`.** `builtin/mod.rs` claimed an
implementation "cannot be reached with an argument list it did not ask for".
Measured, it can: `date()` and `date('S')` succeed so `DATE`'s minimum is 0,
yet `date('S',,'S')` is 40.5, because supplying position 3 makes position 2
required. `(min, max)` cannot express a conditional requirement. Nothing
shipped is wrong -- `LENGTH` has no optional positions -- but seven tasks
would have read that sentence as a guarantee and skipped their own checks.

**Two further findings carried into the plan as later-task obligations:**
whether 40.12/40.23 substitute the rendered value or the source spelling
(unmeasured; the neighbouring 88.928 raiser documents measuring exactly that
distinction), and that the **GC stress subset calls no builtin at all**, so
every allocation the 66 add is outside the collector's reach -- the same
shape as 4a's criterion 4 passing over 29 programs and zero call frames.

**Deferred minors, for the 4c final review:** `dispatch` builds a fresh
`Vec<Option<ObjRef>>` per call purely to strip `Argument` to `ObjRef`, on the
path all 66 builtins take; and `coverage.rs`'s `in_scope().len()` assertion
from Task 1 is nearly implied by its neighbours.

**Worth knowing for the gate:** the reviewer made the 63-vs-66 trap fire and
found `builtin_status.rs` stays **fully green** under it -- that single unit
test is its only guard. Incidentally, the restructure fixed `substr(1/0)` to
match the oracle exactly (42.3, rc 214) where the parent was loud.

**Next: Task 3**, `builtin/string.rs`, 22 names added around `LENGTH`.

---

## Task 3: builtin/string.rs, 22 names around LENGTH

BASE `0c58926e`. Dispatched. Brief 133 lines: 34 extracted plus the 95-line
shared block, which now carries Task 2's three measured corrections. The
append asserts six fragments survive in the block before writing, including
`93.923`, `40.5` and the count-not-shape sentence, so a reorganisation of
that section fails loudly rather than shipping a truncated brief.

Carried into the dispatch because the brief cannot know it: Task 3 **owns the
40.12/40.23 raisers**, which Task 2 deliberately left unwritten -- an unused
raiser is `dead_code` under `-D warnings` -- and owes the measurement of
whether those messages substitute the rendered value or the source spelling.
Also told that landing a builtin can turn a `keyword-exempt.txt` row green,
that such rows are **removed and never re-attributed**, and that Task 2
already moved one.

### Task 3: complete after two fix rounds

Commits `63a9ea9f` (implementation), `c0889d74` (round 1), **`53ba18b5`**
(round 2). 1,059 passed / 0 failed; fmt 0; clippy 0; `REXX_CORPUS_GATE=1`
42 of 42. Tree clean. Accepted without a third review round.

Round 1 closed the byte-substitution Critical. **It also shipped a silent
wrong answer** -- `verify('abcde','','00'x)` gave 1 on the oracle and 0 here,
because the fix collapsed two C++ branch tests that ask *opposite* questions
(an empty reference asks `VERIFY_MATCH`, a non-empty one `VERIFY_NOMATCH`).
A wrong answer inside a correctness round. Round 2 split them, each carrying
the C++ line it transcribes.

**The corpus diagnosis is sharper than the one I gave, and it is the lesson
worth keeping.** I said corpus C had no empty-reference case. Counted:
corpus A had 8 empty-reference `verify` programs but no `0x00` option;
corpus C had 384 `0x00` options but no empty reference. **Neither axis was
missing -- each corpus varied one and held the other safe**, so a defect at
their intersection was invisible to both while their case counts summed to
something that looked like coverage. Widening one alphabet would not have
found it; crossing them did.

**Proved by negative control rather than asserted:** the new crossing corpus
is **72 mismatches against the round-1 build and 0 against this one**, while
corpora B and C both still report 0 against that same buggy build. That is
"can fail is not adds coverage" applied to a corpus rather than a test.

Also in round 2: the byte rule now applies at both oracle sinks, each naming
its C++ counterpart in code; and the 14 empty-argument branches across the
string builtins were enumerated **from the C++ rather than from memory** and
are all driven by the new corpus.

**Two claims of mine were withdrawn at `41fc5474`**, both summaries of
someone else's measurement that I repeated without re-running:
"MEASURED AS THE WHOLE OF THIS CAUSE" (four one-line neighbours still abort
with the experiment applied) and "the file cannot hold a divergence its own
probe does not reproduce" (it can; the probe was the constraint, not the
harness). Recorded in place rather than edited away.

**The implementer's closing concern is stale**: it says the allocation and
address-space gaps have no owner. They were assigned at `f1db85b6` -- the
abort to Task 13 with a per-site trace-witness obligation, the address-space
half to whoever reopens D19 -- after that agent's context was last refreshed.

**Standing for Task 13:** the abort is a *shape* (an unguarded owned copy on
a path that may discard it), not a line; `strip`, `reverse`, `Interp::concat`
and the assignment render all still abort at 400 MB. Peak RSS is 978 MB here
against the oracle's 496 MB for the same result, so parity is the expectation
after the fix.

### Task 4: complete

Commit `3af26b60`, doc fixes at the commit below. Review: spec **PASS**,
quality **APPROVED**, four Minor doc findings and no behavioural defect --
the first task this phase to need no fix round. 1,075 passed / 0 failed;
fmt 0; clippy 0 warm and cold; `REXX_CORPUS_GATE=1` 42 of 42. 30 builtins
implemented, 36 loud, 15 excluded.

**Word separators are exactly `0x20` and `0x09`**, established two ways --
read from `WordIterator::skipBlanks`/`skipNonBlanks`
(`classes/StringClass.hpp:148,174`) and swept over all 256 bytes on the
oracle. I reproduced the sweep: bytes 9 and 32 only. Newline, CR, VT, FF, NUL
and everything above `0x7F` are word **content**. Either method alone would
have been a guess; together they are a fact.

**Calibration on the mid-task addendum**: of the four behaviours forwarded,
the implementer already had three from reading the C++ and got the fourth --
validation order -- wrong, caught by 54 sweep mismatches. So careful
reference reading gets most of the way, and the `('30'x,'30'x)` shape
(position 0 **and** length 0 together) is the kind nobody constructs
unprompted.

**It corrected the addendum**: the NUL word in `DELWORD.testGroup:136` is
**13** bytes, not 14. I relayed the 14 without checking arithmetic that was
`4 + 5 + 4`. Second relay error of the session.

Two claims the reviewer verified rather than accepted: the equivalent-mutant
call on `DELWORD`'s conditional blank-skip (restoring it leaves 16/16 green
on a harness that had just caught a real defect -- genuine equivalence), and
the replacement `WORDPOS` guard being load-bearing (removing it panics with
a subtract overflow). `ASSIGNMENT::test_4` was **removed, not re-attributed**;
777 rows / 771 `4c` / 6 `defect:`.

**Deferred, for the 4c final review:** the differential sweeps and their
generators remain uncommitted across Tasks 3 and 4, so committed cover is the
unit tests plus one `builtin-probes.txt` row per builtin; and `word_pos`
allocates two vectors per call and scans the whole haystack regardless of
`start`. Both are observations, not defects.

**Shared-protocol change Tasks 5-6 inherit**: the argument helpers moved from
`string.rs` to `mod.rs` and `SPACE` now routes through the word scanner. The
reviewer confirmed the move is mechanical with no body changed.

### Task 5: builtin/convert.rs, 12 names

BASE `add307c7`. Implementer commit **`ca0b63f9`**, tree clean, reported DONE.
Review dispatched against `review-add307c7..ca0b63f9.diff`.

Figures to be re-run by the reviewer rather than trusted: 1,095 passed / 0
failed (was 1,075); fmt, clippy warm and cold, and `REXX_CORPUS_GATE=1` all
exit 0; 9,326 crossed differential programs at 0 mismatches; 8 mutations each
caught by a named test and each said to be invisible to the pre-existing
suite. **42 implemented, 24 loud, 15 excluded.**

**A survey ahead of dispatch paid for itself and then was corrected by the
work.** The peer session's convert-family survey gave Task 5 eight measured
behaviours in its Step 0, of which the headline -- `NUMERIC DIGITS` bounding
the **result** rather than the input length, so eleven bytes convert where
four fail at `digits 9` -- is the kind an implementer gets wrong in both
directions. Also the silent left-truncating window, the window switching the
read to signed with `C2D` and `X2D` disagreeing on the same bytes, and the
`BIT*` tail passthrough.

**Four of those claims were then falsified by the implementation, and all
four re-verified here before correcting the plan at `56c73fe1`:**

* `xrange('digit','z')` is 134 bytes, `0x7A`-`0xFF`, digits **discarded** --
  the result begins `7A7B7C7D`. `argcount <= 2` discards everything before
  the final pair; three arguments keep it. The survey's "'z' starts a new
  range to `0xFF`" reads as though the digits survive in front.
* There **is** a default pad, the operation's **identity element** -- `0xff`
  for `BITAND`, `0x00` for `BITOR`/`BITXOR` -- which is why the tail passes
  through. "No default pad" was right about behaviour, wrong about mechanism.
* `d2c('abc',-1)` is 93.929: `D2X`/`D2C` check value-before-length where
  `C2D`/`X2D` do the reverse, so the ordering is **per builtin**, not per
  family.
* `b2x(,'x')` is 40.4 -- a `(1,1)` arity can never reach 40.5.

**Three of the four came from me generalising a measurement, not from a
measurement being wrong.** Each was true of what was probed and false as
stated. The one claim explicitly flagged as *inferred* -- the grouping rule --
is the one that got read from `validateGroupedSet` and closed cheapest: it is
a **cumulative digit total** with the residue fixed at the first whitespace
run, outcome-equivalent on the eight cases it was inferred from. Flagging
inferences is earning its keep.

**Open for the reviewer:** an untested divergence the report records rather
than chases -- `NUMERIC DIGITS` above ~500 million would be Error 5 on the
oracle and succeed here, unreachable under the project's own <=1000 probe
rule. Needs a ruling on whether it earns a `KNOWN GAP` row.

### Task 5: complete after one fix round

Commits `ca0b63f9` + **`c8f33d9c`**. Review: spec **PASS**, quality
CHANGES-REQUESTED, four Important, all closed. 1,096 passed / 0 failed; fmt 0;
clippy 0 warm and cold; `REXX_CORPUS_GATE=1` 42 of 42. 42 builtins
implemented, 24 loud, 15 excluded. Accepted without a second review round --
the evidence below is self-verifying and the round introduced no behaviour.

**The finding worth remembering: correct code that nothing pinned.** Deleting
the per-gap residue check left **all 20 tests and the whole gated package
green**, while `x2c('41 4 1 42')` silently converted where the oracle raises
93.976. The shipped code was right; every refusal the tests asserted also
failed the *end-of-string* check, so no test separated the two readings.

**Closed with a three-state proof, which is the standard the rest of the
phase should meet:**

* per-gap arm deleted -> 20 passed, **1 failed**, the failure being only the
  new `the_residue_is_checked_at_every_gap_and_not_only_at_the_end`;
* restored -> 21 passed;
* same mutation with **only that test skipped** -> workspace 1,095 passed,
  gated corpus green, so **nothing else in the tree catches it.**

The third state is the one that turns "can fail" into "adds coverage", and it
is the state usually skipped. Five witnesses, each re-measured on the oracle
and each paired with an adjacent success holding its residue at every gap.

**Two claims withdrawn rather than fixed**, which is the right outcome more
often than the ledger suggests:

* The report's "`b2x('101 0000')` is the discriminating case either way" was
  false -- it separates residue-from-first-group from residue-always-0, a
  different mutation. It would have sent the next reader after the wrong
  witness.
* The untested `NUMERIC DIGITS` divergence I had flagged for a `KNOWN GAP`
  ruling **does not exist**: both conversion sites request the digits-sized
  buffer up front exactly as `StringClassConversion.cpp:550` does. The
  residual asymmetry at that scale is the 512 MiB `ulimit -v` reservation,
  **already owned**. No row added. A gap process that only ever adds rows is
  one that is not checking.

**Fourth in-repo aggregate corrected this phase**: `builtin/mod.rs`'s "the one
variadic row" goes false at Task 8, since `MAX_Max`/`MIN_Max` are `argcount`
(`BuiltinFunctions.cpp:1996,2025`) and `max(1,2,3,4,5,6,7,8)` is 8. Deleted.

**Deferred to the final review:** `a_scan_takes_apart_every_text_the_number_parser_accepts`
has no floor on how many subjects parse, so it would pass over a corpus that
parsed nothing; and `shift_in`/`render_decimal`/`scan_decimal` allocate
infallibly outside `builtin::buffer`, disclosed in-comment and unreachable
under the <=1000 `DIGITS` rule.

### Task 6: builtin/numeric.rs, 7 names -- IN FLIGHT

BASE `7916cb86`. Dispatched with a 255-line brief carrying nine measured
Step 0 behaviours and **three flagged inferences to close rather than trust**
(the `MAX`/`MIN` dispatch mechanism, the validation order generalising past
`TRUNC`/`FORMAT`, and the `2*DIGITS+1` exponent trigger fitted to four
points).

**Observation to raise at review time, recorded now so it is not lost.** The
working tree shows the implementer modifying **`rexx-num`** -- `format.rs`,
`lib.rs` and that crate's tests -- which is **outside the file list its brief
named**. It may well be legitimate: `FORMAT` and `TRUNC` push directly on the
number model and D15's rendering rule lives there. But it is a different
crate from `rexx-exec`, and **Tasks 7-15 all inherit whatever changes there**,
so it needs the scrutiny Task 4's shared-helper move got -- is it mechanical,
is it minimal, and does anything already relying on `rexx-num` behave
differently afterwards. `builtin-status.txt` in the working tree already
reads 49 implemented against 42 committed, so all seven are landing.

**The controller did not touch the tree while this was live**, per the
recorded hazard about controller edits colliding with running agents.

---

## Survey programme: complete

Seven read-only surveys were commissioned from a peer session across Tasks
4-13 and verified here before any of it reached the plan. **Nine tasks that
had not yet run now open with a measured Step 0 section** instead of
inherited assumptions; the plan grew from ~830 lines to 1,808.

**The programme found more defects in the plan than in the implementations**,
which was not the expected result and is the useful one:

* **Two Task 13 instructions would have shipped regressions.** "Fail loudly
  on any non-`::ROUTINE` directive" would have rejected programs the oracle
  runs fine -- measured, every *well-formed* directive leaves `main`
  untouched at rc 0 and the oracle fails only on *resolution* failure. And
  `::options trace labels` was recorded as unreachable when it is a genuine
  second route to `>I>`/`<I<`.
* **Three convert-family claims were my generalisations of sound
  measurements** -- `xrange('digit','z')`, the "no default pad", and the
  per-family validation order. The surveyor's observations were right; my
  paraphrases were not.
* **A scan defect corrupted committed prose**: every survey used
  `~expectSyntax` and missed `~assertSyntaxError`, 4,654 against 733.
  `base/keyword` carries **more** of the unscanned form (282 / 400). Blast
  radius bounded and checked -- `rexx-extract` already treats it as a
  `DropReason`, so the 772-row exempt file and Task 15's model are sound, and
  the defect is confined to plan prose.

**Four distinct shapes of counting error were found, each distorting a
coverage claim by more than an order of magnitude**: bare-word inflation
(`ADDRESS` 707 against 208), harness boilerplate (407 of 440 `parse source`),
`bif/`-only scanning for constructs that are both instruction and function
(`bif/ADDRESS` 1 method against `keyword/ADDRESS`'s 97), and the two
assertion forms. All four were found by checking, none by suspecting.

**The habit that paid best: flag inferences as inferences.** Every claim
explicitly flagged was closed cheaply by the task that inherited it. Every
claim that later cost a correction was an *unflagged generalisation* -- and
three of those were mine.

### Task 6: complete after one fix round

Commits `510a070e` + **`2472fd8a`**. Review: spec **FAIL** then closed, quality
CHANGES-REQUESTED, two Critical and both fixed. 1,115 passed / 0 failed; fmt 0;
clippy 0 **from a clean target dir**; `REXX_CORPUS_GATE=1` 42 of 42.
**49 builtins implemented, 17 loud, 15 excluded.**

**All three flagged inferences were closed by reading the C++, not by fitting**
-- and one of them found the brief incomplete. `MAX`/`MIN` have **two**
implementations: `RexxInteger::Max` raises 93.903 with a **0-based** position
at rc 163, `NumberString::maxMin` raises **40.5** 1-based at rc 216 for the
identical shape. Verified: `max(1,,3)` and `max(1.0,,3)` differ in both error
number and exit code, decided purely by whether the first argument parses as
an integer. `2*DIGITS+1` was upgraded from a four-point fit to a citation
(`NumberStringClass.cpp:391`) and turned out already correct in `rexx-num`.

**Critical 1: `FORMAT` panicked the interpreter** at `expp >= 65536` (rc 101,
Rust's format width is `u16`) and aborted at rc 134 on a 3 GB allocation where
the oracle gives Error 5. `expp` was the one width not routed through
`builtin::buffer`. Fixed at the exact boundary; verified 65536 and 70000 now
match at rc 0. **This retired D2 entirely** rather than moving its boundary.

**Critical 2 is the instructive one: three instruments, three unrelated
blindnesses, one deterministic defect.** `RANDOM` re-rendered its result under
the current settings, so `numeric digits 3; random(12345,12345)` was `1.23E+4`
against the oracle's `12345` -- a D15 violation in the one builtin whose D15
behaviour nothing checked. It survived because the sweep **excludes `RANDOM`
by rule** (D11, which I wrote), both unit tests ran at **`DIGITS 9`** where no
in-range result can go exponential, and the status probe was **`random(5,5)`**,
which matches at every setting. Each choice was individually defensible.
Summing their coverage looked complete.

Sharper than Task 3's axis-crossing lesson: there, two corpora each varied one
axis. Here, **three different kinds of instrument were blind for three
unrelated reasons that happened to coincide.** The fix extended the status
probe and the unit tests; the sweep was deliberately **not** extended, because
D11 is right and its blindness there is structural.

**The oracle moved under the task, and nothing would have noticed.** The
binary was patched and rebuilt at 16:02 by Moritz -- `copyIfNecessary`'s
cached-string and form-polarity fix -- which retired the upstream `abs`
divergence Task 6 had found, in the direction that says **this crate was
right**. The sweep went 18 -> 12 mismatches with no code change. Recorded in
`phase-4-exclusions.txt` at `f322477f`, together with the standing
consequence: **no result in this project carries a build identity**, and
fingerprinting was considered and declined, so a figure is a claim about the
binary present when it was taken.

**Next: Task 7**, the `PARSE` template engine, whose Step 0 is already measured.

## Task 7 -- the `PARSE` template engine

**Dispatched.** BASE = `f322477f09f6b9712c7b4a416bf2353eedf8c6f7`.
Brief `task-7-brief.md`, report `task-7-report.md`, implementer on opus.

**Pre-dispatch prerequisite check found one wrong claim and four drifted line
numbers**, all carried into the dispatch as my resolution:

* **The brief's Step 1 parenthetical is false.** It says "4a's and 4b's `>K>`
  lines carry a bare value" and asks the implementer to measure whether the
  existing keyword path emits the `=>` continuation. `Trace::trace_keyword`
  (`trace.rs:487`) calls `push_tagged(.., ">K>", indent, true, keyword, " => ",
  value)` -- it already emits `>K>   "KW" => "value"`. Answered by reading the
  call, so the emitter is reusable and a second one must not be added.
* **The question the brief does not ask, and which the dispatch adds.**
  `trace_keyword` is gated on `trace_mode().results`, **not** `intermediates`.
  Every line of the Step 0(g) table was measured under `trace i`, which sets
  both, so whether PARSE's `>K>` appears under `trace r` is unmeasured -- and
  it decides whether the existing emitter is reusable unchanged. Same check
  extended to `>>>` and to the `intermediates`-gated prefixes.
* Line drift, names correct so findable: `WITNESSED_PREFIX_COUNT` is at
  `trace_oracle.rs:557` (= 13) not `:551`; `OUT_OF_SCOPE_PREFIX_COUNT` at
  `:561` (= 6) not `:555`; the `EXPECTED_OUT_OF_SCOPE` row is at
  `owners.rs:352` and the const at `:347`; `lib.rs`'s arm is at `:763` and is
  a **group** with `Arg` and `Pull`, which stay 4c until Task 8.
* Verified as written: the whole AST shape (`ast.rs:1044-1100`), the
  `owners.rs:165` arm, the `loud.rs:194` witness, `phase-4c.txt`'s absence,
  and the loud fallthrough at `run.rs:1915`.

**Task 7: complete.** Commits `da86c106` (engine and every witness), `26fb60d9`
(three false claims in the two corpus headers, caught by re-reading the headers
against the measured output), `c20c6d7f` (mine, the plan corrections),
`73802e7b` (fix round 1), `0f427f61` (mine, one unasserted count in a comment).
Verified by me at `0f427f61`: **72 `test result: ok` lines, no `FAILED`**, fmt
exit 0, `REXX_CORPUS_GATE=1` 42 of 42. Review: spec **PASS**, quality **strong,
no correctness defect**; scoped re-review **all findings addressed**.
`base/keyword`'s PARSE group went 0 -> **653 of 659 bodies / 763 of 778
assertions**, and `keyword-exempt.txt` lost 655 rows with no new failure.

**The finding worth keeping: a survey's *mode* was the blind spot, for the
second time in one construct.** An assigned target's value line is a **choice
of prefix**, not two independently gated lines -- `>=>` at `trace i`, `>>>` at
`trace r`, never both (`ParseTrigger.cpp:274-280`, `assign` then
`if (!tracingIntermediates()) traceResult`). Every line of the brief's Step
0(g) table was taken under `trace i`, which sets both gates, so **no `trace i`
survey however careful could have seen it**. That is the same shape as the
groundwork document's `trace r`-only probe missing `>.>` entirely. Twice on one
instruction the blind spot was the mode the probe ran in rather than the inputs
it chose, which is a sharper statement than "vary an axis you have no reason to
think matters": *the instrument's own configuration is an axis.*

**A correction that lives outside the plan is lost, and I made that mistake
here.** I put both of the brief's false claims into the dispatch prompt and my
reply, not the plan. `rust/CLAUDE.md`'s Method section forbids exactly this and
already names this task. Briefs regenerate from the plan, so Task 8 would have
received the same false `>K>` parenthetical. Fixed at `c20c6d7f`: Step 1 now
names `trace.rs:487` and says the emitter is reusable, Step 0(g) carries the
prefix-choice rule, and **Task 8's own `trace i`-only survey now carries it
too** with an instruction to probe both modes for `PULL` and `LINEIN`.

**A tautological pin, found independently by me and the reviewer.**
`assert_eq!(VERSION, b"<literal copy of VERSION>")` cannot fail on an oracle
rebuild, yet the report claimed "its unit test is what says so". Policy ruling:
the `const VERSION` **stays** (emitting this crate's own identity guarantees a
differential mismatch; leaving `Version` loud breaks one source while the rest
of `PARSE` runs), and the pin now consults the oracle under
`REXX_CORPUS_GATE`. Watched both ways: date moved one day fails under the gate,
skips green without it.

**My gating instruction rested on a false premise, which the implementer
caught.** I asked for the gate so an offline `cargo test` stays green.
`support::oracle::locate` **asserts** the binary exists rather than skipping,
and `builtin_status.rs` reaches it ungated, so that property was never true of
this crate. Reading `locate`'s own assertion message shows Task 1 chose
deliberately and chose the *stronger* goal: not "works offline" but **"no test
can silently pass without the oracle"**. Those diverge and Task 1's is better.
Not Task 7's to fix -- **phase-level item for the final review**.

**Task 7: minor (deferred):** the "eleven operations that move them" count is a
mutable in-repo figure; the `PULL`/`LINEIN` boundary is written at five comment
sites (all non-load-bearing, `keyword-exempt.txt` asserts it, and Task 8 edits
all five); `Cursor`/`new`/`string` are `pub(crate)` in a private module nothing
else references; `Cursor::next_word`'s leading-blank skip is bounded by the
whole string rather than the section end, mirrored from `ParseTarget.cpp:423-433`
and **flagged in the code as inference** -- no probe distinguishes it.

**One resolved by reading rather than reasoning.** I was going to add
`corpus/lang/source_arg.rex` to `phase-4c.txt` on the argument that a program
`PARSE` now makes runnable and no subset reads is a differential we pay for and
do not collect. It calls `arg()`, `arg(1,"o")`, `sourceline(1)` and
`sourceline()` -- all Task 10's `builtin/state.rs`. Adding it would have put an
out-of-scope program in the subset. The filename gave the wrong answer and one
`cat` gave the right one.

**Next: Task 8**, program arguments, `ARG`, `PULL`, `PARSE PULL`, `PARSE
LINEIN`. Its Step 0 is measured, and its trace section now carries Task 7's
both-modes rule.

## Task 8 -- program arguments, `ARG`, `PULL`, `PARSE PULL`, `PARSE LINEIN`

**Dispatched.** BASE = `0f427f61876823d04a5b564d1433442cd3e0129f`.
Brief `task-8-brief.md`, report `task-8-report.md`, implementer on opus.

**Prerequisite check found four things the brief does not say, two load-bearing:**

* **`LINEIN` the *builtin* is `excluded`, not loud** (`builtin-status.txt:76`), as
  are `CHARIN` and `LINES`. D4 keeps all fifteen excluded, so `LINEIN()` must
  keep failing. The brief presents the shared-cursor finding through a
  **five**-construct ooTest assertion, and **only three are in scope**:
  `d = linein()` is an excluded builtin and `e = .input~lineIn` is a message
  send (Phase 5). So `environmentEntries.testGroup:174-181` is **not a target
  this task can pass** -- it is evidence for the design, not a goal. Carried
  into the dispatch as an explicit do-not-attempt.
* **The `ARG` *builtin* is also loud** (`:44`) and is Task 10's `state.rs`. So
  Step 0(a)'s whole table was measured through a builtin this task does not
  implement: afterwards the `ARG` instruction and `PARSE ARG` work while
  `ARG()` still fails. The storage must therefore be reachable by Task 10 --
  that is the interface this task owes the next one.
* **This crate has no stdin path at all.** `/bin/grep -rn stdin` over `lib.rs`,
  `run.rs` and `queue.rs` returns nothing. Task 8 introduces the first one, and
  the `cargo test` behaviour is an unspecified design decision (a test reading
  the harness's real stdin is nondeterministic at best, hanging at worst).
* **`Oracle::run` pins stdin to `/dev/null` and passes no arguments**
  (`support/oracle.rs:152`, `.stdin(Stdio::null())`, `$@` = binary + path).
  This cuts both ways: it already witnesses Step 0(c)'s empty-queue/null-stdin
  case with no extension, but `PARSE LINEIN` reading real lines and any program
  argument have **no differential route** through it. The `sh -c` wrapper
  already forwards `"$@"`, so extending it is small.

**Two rulings given, since both were open in the brief:**

* **Extend the oracle harness rather than taking the `KNOWN GAP` branch.** An
  argument-passing feature whose only proof is an in-crate unit test is the
  exact shape the gate criteria exist to prevent. Fall back to the gap row only
  with a *specific* reason the extension cannot be made deterministic.
* **Change `run_program`'s signature rather than adding a sibling**, despite
  **44 call sites across 14 files**. `run_program_collect_every_alloc`'s own doc
  states the codebase's one-front-door principle ("not a second front-door
  choice beside `run_program`"), so a sibling would contradict it.

Verified as written: the four boundary rows (`owners.rs:171`/`:172`,
`EXPECTED_OUT_OF_SCOPE` `:357`/`:358`), both `loud.rs` witnesses (`:194`,
`:199`), `rexx-run.rs` discarding trailing argv, and `ORACLE_MEMORY_LIMIT_KIB`
= 1048576 matching the mandated wrapper. `lib.rs`'s arm has drifted off `:758`
and after Task 7 holds exactly `Arg` and `Pull`, so this task **empties** it
rather than shrinking it.

**Task 8: complete.** Commits `8eb5a402` (argument plumbing + 45 mechanical
`run_program` call-site edits), `81944c76` (the input model, both `PARSE`
sources, both instruction spellings, boundary moves, corpus program, new
`tests/input_oracle.rs`), `87b762b0` (mine, Task 15's criteria), `2070cd9d`
(fix round 1). Review: spec **PASS** on every requirement, quality **sound**;
scoped re-review **all findings addressed, no new defects**. Verified by me at
`81944c76`: **1152 passed / 0 failed / 73 `test result: ok`**, cold clippy 0,
`REXX_CORPUS_GATE=1` 42 of 42.

**My prerequisite check asserted the contents of code I never read, and the
implementer caught it.** I said `instruction_owner`'s arm held exactly `Arg`
and `Pull` so this task would *empty* it. At BASE it read
`Arg(_) | Pull(_) | Address(_)`; `Address` is Task 9's, so the arm shrank and
three boundary rows stayed. My `sed` window landed on a different region after
the line drifted and I wrote the claim from Task 7's brief text instead of the
file -- **in the same note that told the implementer the line had drifted and
to find it by name.** Bounded because they read the code and `owners.rs` would
have caught a wrong deletion: the type-level defence held, the prose did not.
Same lesson as [[dispatch-prose-is-not-a-control]], now with me as the source.

**Two false claims in the brief, both killed by running rather than reasoning.**
"Absent vs empty is only visible through `ARG()`" is false -- `use arg p`
distinguishes `P` from the null string and `use strict arg` turns it into
40.3/40.4 against rc 0, all in scope and now witnessed. And an unreadable stdin
(closed fd, or a directory) is end of input on the oracle at rc 0 with no
condition, which **deleted** an error path already being written. The
implementer nearly reported the first as unwitnessable.

**"Committed but inert" is the brief's phrase and it is too strong.**
`tests/corpus.rs` does not read `phase-4c.txt` until Task 15, so the gate
stays 42 -- but `coverage.rs::every_in_scope_variant_is_witnessed_by_the_phase_subsets`
reads the union **including** it, so those programs discharge the coverage
obligation today. Parsed, not run. This also corrects what Task 7's entry above
says. Calling them inert invites deleting them.

**The Task 7 tautology recurred one level up.** `assert_eq!(oracle.invocations(),
CASES.len())` derives both sides from the same array, so an empty `CASES` passes
having compared nothing -- the same shape as comparing a constant to a copy of
itself, at the level of the guard rather than the value. Fixed with a literal
`>= 15` floor beside the equality, watched firing by cutting `CASES` to one row.

**The witness the implementer chose is stronger than the one I offered.** I
suggested `input_oracle.rs` (live, gated) or `trace_oracle.rs`; they took
`trace_oracle.rs` because it runs **ungated** against a **committed
expectation**, so the fact needs no oracle at check time. No new program: only
`pull_queue.expected` is new. Its mutation proof is the three-state form done
properly -- `>K>` carrying the post-upcase value (invisible in stdout) and `ARG`
emitting a `>K>` were each caught by the new witness and by **nothing else in
the workspace**, `parse_placeholder` included, and the re-reviewer reproduced
both independently.

**`</dev/null` in a regeneration recipe is a harness property, not a quirk.**
Second instance this task, after `sourceline_oracle.rs` in round 1: `run_program`'s
default console is empty, so any recipe that reads real stdin bakes in an
expectation this crate can never reproduce.

**Task 8: minor (deferred):** `ProgramInput::Bytes` is a public variant with no
caller outside its own unit tests; `execute` calls `interp.text(&argument)` on
an owned `Vec<u8>` where `text_owned` avoids the copy; `input.rs:58`'s "There is
no reachable state in which a line read fails" reads as an unreachability claim
until the next paragraph rescues it.

**Next: Task 9**, the `ADDRESS` instruction and environment tracking. Note for
its prerequisite check: `instruction_owner`'s arm now holds `Address` alone, so
Task 9 is what finally empties it.

## Task 9 -- the `ADDRESS` instruction and environment tracking

**Dispatched.** BASE = `2070cd9dea779c8e67cf0673624ffc9ac0781b25`.
Brief `task-9-brief.md`, report `task-9-report.md`, implementer on opus.

**The brief's Step 2 rests on a false premise, and following it literally ships
a defect.** Step 2 says `loud.rs`'s witness is `address cmd`, "an `ADDRESS`
**with a command**, which stays Phase 7's", and to keep it while making the row
arm-grained. Measured on the oracle from a clean directory:

```
address cmd
say 'ok'
-> ok, rc=0
```

Nothing executed. **`address cmd` is the constant-environment form -- exactly
what this task implements** -- and `ADDRESS env command` is the command form.
So once Task 9 lands that source **stops failing loudly**, and a `loud.rs` row
still claiming it fails asserts that an in-scope construct is out of scope.
The *instruction* to make the row arm-grained is right; only its reasoning is
wrong, and the witness **source** must change to a real command form. Carried
into the dispatch with the measurement.

**The structural problem the brief does not mention, same shape as Task 8's
`ARG()`.** The `ADDRESS` **builtin** is `loud` (`builtin-status.txt:43`) and is
Task 10's, as are `DIGITS`, `FORM`, `FUZZ`, `TRACE`, `CONDITION`, `QUEUED` and
`VALUE`. So Step 0(a) measures the upcasing rule through `address()`, and Step
1 demands a corpus witness asserting the swap -- but after this task `ADDRESS()`
still fails, and the environment's only other observable effect is issuing a
command, which is Phase 7's. **The swap may be unobservable from inside a
program until Task 10.**

Deliberately did **not** hand this down as settled: Task 8's brief made the
identical claim about `ARG()` and was wrong, because `use arg` and `use strict
arg` observe the distinction, and that implementer nearly reported it
unwitnessable without checking. The dispatch says to search for an in-scope
observer first, and if there genuinely is none, to **write the corpus witness
obligation into Task 10's own section of the plan**, not just the report.

Verified as written: `ast::Address`'s four fields (`ast.rs:1225`), the
`owners.rs:172` row and its `EXPECTED_OUT_OF_SCOPE` entry, and the
`Call::Qualified` arm-grained pattern the brief points at. Drifted: the arm is
`lib.rs:802` not `:761` and is now **standalone**, so it is replaced by
arm-grained logic rather than shrunk -- something must still answer for the
command and `WITH` forms; the `loud.rs` witness is at `:193-197`, not `:208`.

**Task 9: complete.** Commits `27606888` (the instruction, environment state,
boundary moves, corpus wiring), `9a260d2f` (mine, Task 15's Step 4),
`cc83ee83` (fix round 1), `011589c1` + `236ebeff` (mine, the mutation
methodology). Review: spec **PASS**, **no correctness defect**; scoped
re-review **all ten findings addressed, no new defects**. 1159 passed / 0
failed / 73 `test result: ok`, cold clippy 0, `REXX_CORPUS_GATE=1` **47 of
47** (was 42), keyword gate green.

**The phase-level finding, and it invalidates coverage claims I had already
reported as proof.** `cargo test --workspace` **stops after the first failing
test binary**, so under a mutation the run is truncated at the first catcher
and every later binary silently never runs. Three of Task 9's six "caught by
this test and nothing else" claims were false: re-run with `--no-fail-fast`,
`corpus_differential` catches two and `keyword_assertions` a third.

**It hides almost perfectly and it flatters.** A green baseline runs every
binary, so the flag looks unnecessary -- the truncation happens only when a
mutation bites, which is precisely when the measurement is taken. And the
error is one-directional: it **over**-credits a new test with unique coverage
and never under-credits. So no correctness conclusion rests on it; the
coverage story does.

**This one is mine.** I spent the phase enforcing the three-state proof --
run the mutation against the suite *without* the new test -- as the difference
between "can fail" and "adds coverage", and never specified the flag. The
instruction was right and the command under it was truncating. Fixed at
`011589c1`: Task 15's `mutate-4c.sh` requires `--no-fail-fast` and requires
asserting the binary count baseline-against-mutated, so truncation is
`INFRA_FAILURE` rather than a clean catch. **Tasks 5, 6 and 7's uniqueness
claims were taken with the truncating command and are recorded as needing
re-verification at the gate rather than inheritance**; Task 8's re-review
happened to use the flag and stands.

**Two honest counts of the same run disagree, which the guard had to be told.**
72 `Running`/`Doc-tests` header lines against 73 `test result:` lines, because
`Doc-tests rexx_exec` emits **two** result blocks from one process, a normal
doctest and a `compile_fail` one. 72 is the process count and is what a
truncation guard compares. Recorded at `236ebeff` -- unstated, Task 15 writes
the guard against the wrong metric and it reads off by one forever, and a
guard wrong by a constant still catches gross truncation, which is exactly why
that error would have survived.

**"Committed but inert" was a defect wearing the plan's own words.**
`tests/corpus.rs` read only the 4a and 4b subsets, so **four witnesses across
Tasks 7 and 8 were parsed by `coverage.rs` and run by nothing**, while
`corpus.rs`'s module doc claimed otherwise -- and one of Task 9's mutations
was invisible to the whole suite until the wiring landed. I had reported that
state twice, first as "expected rather than a defect" (quoting the brief) and
then as "parsed, not run" after the Task 8 reviewer corrected me. Both times I
treated it as a design choice. **A witness that does not run is not a
witness.** Task 9's implementer wired it in -- a later task's step -- and I
kept the change and recorded at `9a260d2f` what Task 15 now inherits.

**Five of twelve review findings shared one mechanism: a corrected fact whose
other copies were never visited.** The toggle bound was fixed in the plan and
the `run.rs` test doc but not `activation.rs:92`; the reader count fixed in
the report but not the corpus program; `corpus.rs`'s module doc fixed but not
`read_subset`'s doc forty lines above the call site. This is the mirror of
[[correction-rounds-introduce-false-statements]] -- not a new falsehood beside
the fix, but **the old falsehood surviving where the fix did not look**, which
fails silently because the corrected copy gives no hint the others exist. The
instruction that worked: sweep the tree for every statement of a fact before
fixing any of it.

**Task 9: minor (deferred):** `AddressState` derives `Debug`/`PartialEq`/`Eq`
that nothing uses (measured: removing them keeps `-p rexx-exec` green).
Two stale "42 of 42" totals were found and **deliberately left** --
`phase-4b-gate.md` (nine, inside that gate's own transcript) and
`phase-4-exclusions.txt` at two places -- because they are dated records of
past runs whose substance is still true, and editing another gate's recorded
number falsifies a record rather than correcting one.

**Next: Task 10**, `builtin/state.rs`, 11 names. It owes the `ADDRESS` corpus
witness Task 9 could not write, already recorded in Task 10's own Step 0a.

## Task 10 -- `builtin/state.rs`, 11 names

**Dispatched.** BASE = `236ebeff3726c37c8ba3d03760631e68c83d7ad8`.
Brief `task-10-brief.md` **with the shared builtin block appended** (the
controller obligation the plan names: extracted from `## Shared facts every
builtin task needs` to the next `## `, 186 lines, and all three required
fragments -- `pub(crate) fn dispatch(`, `not_enough_arguments`,
`REXX_CORPUS_GATE=1` -- asserted present before writing). Implementer on opus.

**Prerequisite check:**

* `ActiveCondition` is `lib.rs:935`, not `:871`; its three fields are exactly
  as the brief says, so its claim that nothing records `CALL ON` vs `SIGNAL ON`
  holds.
* `AddressState` is `activation.rs:114` with both fields as described, and its
  doc **already states this task's central problem** in Task 9's words:
  rendering `None` means naming the platform default.
* The 11 names are exactly the `loud` rows minus Tasks 11's and 12's:
  **17 loud** = 11 + `DATATYPE`/`SYMBOL`/`VALUE`/`VAR` + `DATE`/`TIME`.
* **`/bin/grep -c loud` on that file returns 18 against 17 real rows** -- the
  extra is the header line explaining the word. Told the implementer, because
  Step 4 asserts a count and this is the phase's most-repeated error. Printing
  the hits settled it in one command, as it always does.
* Brief steps are numbered **0a, 1, 0, 2, 3...** -- a plan defect, nothing
  missing. Flagged so it does not read as a truncated brief.

**The design question, handed over with a precedent rather than a ruling.**
`ADDRESS()` must render `current == None` as something; the oracle says `sh`,
but the platform default is Phase 7's, which is exactly why Task 9 could not
land this builtin. Two precedents already exist in this phase and they point
opposite ways: `PARSE SOURCE`'s `LINUX COMMAND` is platform-supplied and went
**into** the corpus as a measured constant with the host-dependence flagged;
`PARSE VERSION` carries the oracle's **build date** and was deliberately kept
**out** of every corpus program, pinned instead by a gated differential,
because a build date moves under you and a platform name does not. The
dispatch asks which of the two `sh` resembles and for the reason to be written
at the decision point.

**This task owes Task 9's debt:** the `ADDRESS` corpus witness, three-deep
swap plus callee inheritance in both halves -- the second of which no in-crate
test can pin, since `resolve_and_run_call` pops the callee unconditionally.

## Task 10: `builtin/state.rs` -- complete

Commits: `f03d69f1` (implementation), `9b4ee92a` (fix round 1), `67382f88` (fix round 2).
Verified by the controller at `67382f88`: 1192 passed / 0 failed, corpus differential
green under `REXX_CORPUS_GATE=1`, **73** `Running`/`Doc-tests` process headers.

**Fix round 1** closed all five Critical/Important findings, and the re-review confirmed
each by re-running rather than by reading -- twelve C1 witnesses byte-identical on both
sides, the C++ citations read out of the oracle tree, and the coverage argument
reproduced by reverting the four call sites (four catchers: two unit tests, the corpus
differential at 48 of 49, and the state-builtin oracle test).

**Fix round 2 (`67382f88`) fixed one new Important the round itself created**, and it is
the mirror shape: `error.rs:554` said 40.14's substitutions were "as
`argument_not_whole`'s", whose doc defines `found` as the *rendered* value -- which the
round had just made false. `error.rs` was never in the fix commit's file list, so the
identifier sweep never reached it. Of four cross-references to `argument_not_whole` only
that one was false; 40.23, 40.28 and 93.928 all fire on values that never converted, so
the rendering claim is true of them. 40.34 and 40.903 were silent rather than wrong and
were given the contract, since their callers now depend on the distinction.

### Deferred minors, for the 4c final review

* **M1** `Loud::builtin_option_object`'s doc contract is false for one of its four uses
  (`CONDITION('D')` for `NOVALUE` answers a plain string).
* **M2** `caller_trap_for` (`run.rs:2943`) does not filter `delayed`. Left unfiltered and
  the comment corrected instead, decided by running it: no probe makes it observable,
  because `deliver_pending_trap` re-checks through `trap_for`.
* **M3** `RAISED` as an exempt attribution has prose as its only control, where
  `divergent` in `builtin-status.txt` requires a `KNOWN GAP:` marker.
* **M4** `trace('?')` -- the builtin-as-setter form -- is a third witness of the declared
  `?`-prefix gap and is not in the row.
* **M5** `rust/CLAUDE.md:57` and `tests/builtin_status.rs:26` both say "796-row" for
  `keyword-exempt.txt`, which now holds 16 rows. Wrong before this task too.
* **M6** `corpus/README.md`'s "Phase 4c additions" table has no row for
  `state_builtins.rex` or `address_env.rex`. Nothing polices the table.

### Two things the controller got wrong, recorded so the gate does not inherit them

* I relayed the fix report's claim that a silent no-op mutation "was fixed by asserting
  the replacement count". The re-review found it **unverifiable**: it describes the
  implementer's scratch mutation workflow, not anything in the tree. The lesson is real
  and belongs in the mutation discipline; the claim about this commit is not.
* The report's `state_builtin_oracle.rs` case count of **82** is wrong; it is **87**.
  Both grep methods reconcile only after the `struct Case {` declaration and the
  `source:` inside a format string come out -- the sixth instance this phase of a count
  matching its own file's legend. Corrected in the report.

## Task 11: `builtin/datatype.rs` -- complete

Commits: `d6e8a8c7` (implementation), `9c3d96df` (fix round 1), `f11c55aa` (controller
edit, prose only). Verified by the controller at `d6e8a8c7`: 1208 passed / 0 failed,
73 process headers; the implementer reports 1210 / 0 at 73 after the fix round, and
`builtin_status` is 13 passed / 0 failed after `f11c55aa`.

`DATATYPE SYMBOL VALUE VAR` all read `implemented`. The task also added
`Raised::argument_not_a_symbol` (40.26) and `Loud::value_selector`, neither of which the
brief's "Files" line named -- a plan defect in that line, since the shared block's text
anticipated both.

**The review found three Criticals; the controller reproduced two before dispatching.**

* **C1** 40.26 substituted the upcased text where the oracle substitutes the source
  spelling: `value('ab*')` is `found "ab*"` against `found "AB*"`, rc 216 and stdout
  identical, so only stderr sees it. The shipped justification cited
  `VariableDictionary::getVariableRetriever`, which does upcase -- **its own local** --
  while the raiser at `BuiltinFunctions.cpp:1840` passes the caller's string. A true
  premise carrying a false conclusion. Both committed tests used `*` and `5`, neither of
  which has a case, so neither could ever fail on it.
* **C2** `value`'s compound-write arm held an unrooted result across an allocation and
  panicked under collect-every-alloc. Latent only because the stress harness read
  `phase-4a.txt`, so the gate reached none of 4c's code.
* **C3** `value('.LOCAL')` answered `.LOCAL` at rc 0 against `The Local Directory`, while
  `say .LOCAL` was already loud -- the same name loud on one path and silently wrong on
  the other, across the whole environment directory.

**Two findings collided with plan text and the user ruled on both.** C3: declare in KNOWN
GAPS, do not build the subsystem -- enumerating the environment set is Phase 5's work and
the enumeration would live in this repo and rot. I3: drop `Loud::value_selector`'s owner,
matching `builtin_option_object`'s documented no-owner convention, and record the
per-path attribution in the exclusions file, since the oracle's selector dispatch is three
subsystems and only the third is unambiguously Phase 7's.

**The fix round's own incident, which is worth more than the findings it nearly undid.**
The implementer restored `datatype.rs` from a **stale backup** partway through and
silently reverted several already-applied fixes, with no error. They caught it by
re-grepping each fix's marker text. Restore-from-backup is the procedure this project
adopted *because* `git checkout --` discards uncommitted work, and it has now produced
the same class of damage from the other direction. A green suite says nothing here: the
reverted work was prose plus one behavioural change nothing else covered. The scoped
re-review was given the integrity question first and cleared it -- every changed line in
`datatype.rs` traces to one of the ten findings, tallied hunk by hunk against the deltas.

**The controller edit `f11c55aa`** fixed the one new Important the round introduced: the
C3 KNOWN GAP row explained the status row by quoting the probe that derives it, which is
mutable repo state nothing polices. Its own cited precedent, TRANSLATE's row, anchors to a
measured consequence instead. Reworded to the policed property.

### Deferred minors, for the 4c final review

* `task-11-report.md`'s M4 section states a grep methodology that does not reproduce --
  the two lines it claims to have found by `Task [0-9]` contain no such string. The
  finding it reports was still fixed correctly; only the account of how is wrong.
* `datatype.rs:795` and `:836` cite "the brief" as the source of two facts -- the same
  SDD-process provenance shape M4 removed from two other spots. Predates the fix round
  and matches tree-wide convention (`error.rs` and `lib.rs` carry dozens), so it is a
  convention question for the final review, not a Task 11 defect.

### Carried to Task 15, and now with a second instance behind it

`corpus/phase-4c.txt` exists and is **not** in the stress harness's subset, which reads
`phase-4a.txt` and `phase-4b.txt`. C2's witness went in as a dedicated inline test with
three adjacent controls, which is well targeted -- but the reason C2 existed at all is
that the stress gate reaches none of 4c's code, and that is still true. The gap is
declared at `collect_stress.rs:105-112` and owned by Task 15. Two of this round's ten
findings were missed-collector-coverage, which argues for pulling Task 15's Step 4
forward rather than letting more of these surface through review.

## Task 12: `builtin/datetime.rs` -- complete

Commits: `7891fd4b` (implementation), `20205641` (controller, plan correction),
`c0596259` (fix round 1), `9404ff0b` (fix round 2). Verified by the controller at
`9404ff0b`: 1246 passed / 0 failed, 73 process headers, corpus 49 of 49, exit 0.
`DATE` and `TIME` both read `implemented`.

**This task's unit tests are the only gate these two builtins will ever have.** D11 bars
both from every differential corpus program, and the status harness runs one shallow probe
per name. That is why every finding below about a missing test was treated as a defect
rather than a nicety.

**The brief was wrong and the implementer refuted it.** It split `DATE`'s letters into
host/locale-dependent (`L M W T F`) and deterministic (`B D E I N O S U`). All thirteen are
deterministic given a fixed input; the axis is the *argument*, not the letter, and only the
no-argument form reads today's clock. Confirmed three ways by the controller: hardcoded
English tables at `RexxDateTime.cpp:53`/`:65` with no locale call in the file, identical
output under `LC_ALL=de_DE.UTF-8`, and byte-identical `T`/`F` across runs. Corrected in the
plan at `20205641`. Fourth brief in this phase to carry a false claim, fourth found by an
implementer rather than by review.

**Two Criticals, both reproduced by the controller before dispatch.**

* **C1** `date('D','0','D')` aborted the process at rc 101 -- `usize::MAX` index -- where the
  oracle prints `0`. Not a Rexx condition: no `SIGNAL ON SYNTAX` could reach it. `0` is
  inside the accepted range on both sides and the C++ range check was ported exactly; the
  wrap came from a deliberate `month = 0` sentinel meeting an unguarded `month - 1`. Five of
  six output styles reached it; `M` survived only because it already had the guard the other
  five needed.
* **C2** a `'00'x` separator was accepted where the oracle raises 40.43, emitting
  `2007<NUL>09<NUL>22` at rc 0. The oracle tests membership with `strchr`, which returns a
  pointer to the terminator for a NUL, so a NUL is "in" every separator set. The probes did
  carry high bytes and control bytes -- **in option position 1 only**, never crossed with the
  separator positions, and `0x00` appeared nowhere. The crossed-axes failure the shared brief
  spends a paragraph on, reproduced exactly.

**Four of the six remaining findings were false statements beside correct code**: the single
`elapsed_anchor` is the right design justified by a false claim about `putSettings` (an
internal call inherits `elapsedTime` by value; the copy-back is guarded by `isInterpret()`),
and the divergence note named three input styles that do not diverge while missing the three
that do. `I3` needed architecture rather than prose -- the oracle applies `TIME('R')`'s reset
lazily at the next clause-cache refresh, so the cache split into a persisted value plus a
per-clause staleness flag.

### The finding worth carrying out of this task

**A correct fix with no test behind it, and a report claiming the test exists.** Round 1
fixed C1 and I3 correctly and pinned neither. The report asserted a test
`date_day_of_year_zero_is_defined_across_five_output_styles` that existed nowhere. Both holes
were proved by mutation rather than argued: reverting either fix left the whole workspace at
exit 0, zero failures, 73 headers.

The instructive part is that **the implementer caught this exact gap themselves for C2 in the
same round** -- verified the NUL fix by hand, never wrote the pin, and their own mutation step
found it. So the check works and was simply not applied to all three fixes. The lesson is that
the mutation proof has to be *per finding*, not per round; a round-level "mutations were run"
says nothing about which fixes they covered. Round 2 was scoped to the two tests and required a
red/green transcript with header counts for each.

### A new upstream ooRexx crash, not yet filed

`say date('M','0','D')` segfaults the oracle: **rc 139, three runs of three**, deterministic.
Found by fixing our own panic on the same input -- `0` is an accepted day-of-year on both
sides, we wrapped a `usize` and the C++ walks off the front of `monthNames`. `monthStarts[-1]`
happens to read `0` on this build, which is why the other five styles merely return a wrong-
looking-but-stable number; `monthNames[-1]` reads a garbage pointer. This crate answers the
empty string, declared at `month_name`'s own doc. **Filing is the user's call and has not been
done.**

### Deferred minors, for the 4c final review

* Step and brief references in comments (`datetime.rs:1204`, `:1208`, `:1555-1556`, `:1766`,
  `:1803`) are against the stated constraint but the pattern is tree-wide across 10+ files.
  Explicitly deferred as a convention question, not a Task 12 defect.
* `task-12-report.md`'s round-2 correction says the false test claim was "caught by round 2's
  own re-review". It was caught by the re-review *of round 1*. A provenance slip inside the
  paragraph correcting a false claim, which is the fix-round pattern in miniature.
* Three timing tests carry wall-clock margins (`r2 >= e1`, `e3 > e2 * 1.5`, `after < 0.05`).
  Stable across 8 consecutive runs and deliberately reasoned; noted because a loaded machine
  is the one thing that could make them intermittent, and nothing else in the suite is
  wall-clock dependent.

## Deferred decision: splitting `run.rs`

Raised 2026-08-07, **deferred by the user until Task 13 lands.** Do not open it before then.

Measured at `bf272e63`: `run.rs` is 13,466 lines -- 6,395 of them the test module after
`#[cfg(test)]` at `:7071`, and 4,436 of the remainder comment lines, leaving roughly
**2,600 lines of production code** in a single `impl Interp` block (`:565` onward, 85
methods). The next largest file is `lib.rs` at 2,592. So the headline number overstates
it: the density is this project's evidence-comment style, not sprawl.

If it goes ahead it belongs in a **post-4c consolidation plan, not a Task 16**:

* Task 13 is restructuring `resolve_and_run_call` and sweeping ~15 copy sites in this
  file right now.
* Task 15 is the gate. Behaviour-neutral churn between the last implementation task and
  the gate moves the mutation baseline and the process-header count the gate compares.
* A pure-move refactor has **no differential signal** -- output is unchanged, so a green
  corpus proves almost nothing. What can actually break is dropped comments (against the
  standing rule) and intra-doc links. Those need a mechanical check of their own: comment
  line count before against after, and `cargo test`'s doc-test pass for rustdoc links.

Mechanically cheap when the time comes: inherent impls may span modules in one crate, so
`run/instruction.rs`, `run/call.rs`, `run/condition.rs` can each carry their own
`impl Interp`.

Two items are already queued for that same consolidation: the stale hardcoded process
count in Task 15's mutation guard (the fix is to have the guard measure its own baseline,
not to bump the number), and `corpus/phase-4c.txt` still being absent from the stress
harness's subset, which has now cost two findings.

## Task 13: `::routine` dispatch, `>I>`/`<I<` -- complete

Commits: `e00528df` (dispatch third in the order), `3aa457e1` (directive refusal),
`622f7198` (the trace pair plus `EXIT`-in-a-routine), `88b3701f` (the render guard),
`34eca951` (fix round 1). Plan corrections at `bf272e63` and `fb99355b`.
Verified by the controller at `34eca951`: **1264 passed / 0 failed, 73 process headers,
corpus 50 of 50**, exit 0.

**Two user rulings, both taken on the recommended option:** refuse `::REQUIRES` and
`::CLASS ... SUBCLASS` naming Phase 5, knowing it diverges on programs the oracle runs at
rc 0, because the alternative is a silent wrong answer; and assign the third memory cause
to a post-4c consolidation plan.

### The brief was wrong about `::REQUIRES`, and the probe that cleared it is the lesson

Step 4's survey ran `::requires 'helper.rex'` with a **silent** helper, saw rc 0 and
`main ran`, and concluded that containing a directive you never use is harmless. Both
observations were true. With a helper whose first clause is `say 'PROLOG RAN'`, the oracle
prints that line **above** the requiring program's output. `::REQUIRES` runs the required
file's prolog, so presence is use, and it also imports names into the resolution chain.

The survey's evidence was equally consistent with "`::REQUIRES` is inert" and "`::REQUIRES`
runs the file"; nothing in it discriminated. Fifth false brief claim in this phase, fifth
found by an implementer rather than by review. The plan's closing rule was generalised at
`fb99355b`: **check whether a directive's own installation is observable before deciding a
program does not use it.**

### The two Criticals, both reproduced by the controller

* **C1** `>I>`/`<I<` announced where the oracle emits nothing, whenever a routine's first
  top-level instruction is a nested construct: oracle 0 bytes of stderr against this
  crate's 318, both rc 0. The permission flag was spent only at top-level instruction
  boundaries, but an `IF` branch, a `DO` body and an `INTERPRET` fragment all run inside
  the enclosing instruction's step. One top-level clause in front of the construct made
  both sides silent, which is what pinned the diagnosis.

  Uncaught because the existing test probed only flat clauses in front of the `TRACE`:
  the trace-gate axis and the nested-construct axis were each varied with the other held
  safe. **At least the fourth crossed-axes defect in this phase.**

  The fix needed a conjunct the suggested shape did not have -- `parent->isMethodOrRoutine()`
  (`RexxActivation.cpp:3646`), without which `interpret "interpret 'trace l'"` announces
  where the oracle writes nothing. Verified after the fix across five shapes, all matching:
  `if`/`do`/`interpret`/interpret-in-interpret all silent on both sides, and plain
  `interpret "trace l"` announcing 316 bytes on both. **That last row is the discriminator**
  -- a fix that merely suppressed everything passes the first four and fails it.

* **C2** four comments still claimed external resolution was 4c's, when this task's own
  exclusions row assigns it to Phase 7, and the report said the contradiction was settled.
  Sharpest detail: in `eval.rs` the **test five lines below was updated to assert 43.1 while
  the production doc above it was not**. The fix visited one copy; the survivor reads as
  original text with nothing to hint at it.

### Findings that outlive the task

* **The indent normalisation blinds both differential harnesses to the entire indent axis**,
  for every trace prefix, in every corpus program and every committed `.expected` file --
  the expectations are raw oracle bytes but are normalised at comparison time, so they pin
  nothing there either. This is DEVIATION 0's declared scope rather than a new defect, but
  it means **"the corpus pins this indent" is never true**. Only an in-crate exact-stderr
  assertion sees it. Task 15's gate must not count on the corpus for trace-indent coverage.
* **The scratchpad hazard has a second form: filename collision, not the search path.**
  Three probe files were silently overwritten by later batches reusing names, and the first
  sweep of the fix round reported them as regressions. It produces a false *regression*
  rather than a false pass, which is the survivable direction by luck alone. Every sweep in
  the round was re-run from freshly created directories.
* Two facts the brief never mentioned, both found by the implementer: `EXIT` inside a
  `::routine` ends the routine rather than the program (identical on both sides), and
  placing the routine step behind `is_builtin` would have sent the 15 wholly excluded
  builtins to 43.1, a condition the oracle never raises for them.

### Deferred minors, for the 4c final review

* `plan.rs:69` and `activation.rs:266` state call-site counts, which `rust/CLAUDE.md`
  forbids by name. True today, which is the failure mode.
* `run.rs:5569` and `:5585` are trace-only unguarded renders whose exclusion-list
  description is wrong (the arithmetic uses the `ObjRef`, not the bytes). Exposure is nil
  because a `DO` control value is necessarily numeric.
* `phase-4c.txt`'s absolute-path exception sits at the program's entry, ~25 lines below the
  rule it excepts.
* The third-memory-cause row's "three copies against the oracle's two" is a summary nobody
  can check from outside the process; the abort behaviour reproduces, the count does not.
* **An unexplained 5.7 MB**: after the render guard, peak RSS is ~490 MB here against the
  oracle's ~496 MB, re-measured independently twice. Parity was the arithmetic expectation
  and this crate lands *under* it. Held open as a finding rather than called noise, which
  is the right handling -- the instrument's own spread is ~800 kB.

## Task 14: the compound-`DO` control-variable fix -- complete

Commits: `1c2300d9` (controller, plan corrections), `d372dcdf` (fix), `1c519dfc` (fix round 1).
Verified by the controller at `1c519dfc`: **1271 passed / 0 failed, 73 process headers,
corpus 50 of 50**, keyword gate 888 of 896. The seven exempt rows are gone; the exempt
table went 15 data rows -> 8.

### Two prerequisite corrections, both from reading the tree

* **The exempt table said six and held seven.** `keyword-exempt.txt`'s own header block
  claimed "6 bodies"; the data rows were `DO::test_DO_standardTest2A/2B/2P/2Q/5-69`,
  `ITERATE::test_12`, `LEAVE::test_11`. The step told the implementer to assert "the six"
  **by name** before removing the rows, so following it literally would have left the
  seventh unasserted while removing all seven. Sixth instance this phase of a file's own
  header explaining its own values, and the worst form: the count was about to become the
  shape of a verification step.
* **No `rexx-parse` change was needed.** `Controlled::control` is already a `SymbolId` and
  the parser already interns `CV.J` whole -- the same starting point `say cv.j` has -- so
  the defect was entirely in `bind_control`'s flat name-to-slot lookup. `ast.rs` dropped
  from the file list, with an instruction to stop rather than quietly widen if that turned
  out wrong. It did not.

### The defect was wider and worse than recorded

`bind_control` ran **every** control-variable shape through the flat simple-variable write,
not only compounds. And for a stem the consequence is not a wrong value but a **panic** --
`stem.rs:288:60: a live value`, rc 101 -- as soon as a tail of that stem is touched
elsewhere, because the corrupted slot holds a bare scalar where `stem_set`/`stem_get`
expect a `Body::Stem`. Nobody had noticed because no exempt row pointed at stems.

### The finding worth carrying: a test named for the defect could not fail against it

`a_stem_control_variable_binds_through_stem_assign`'s program printed `24 / 11` on the
**pre-fix** build, byte-identical to what it asserted. A bare-stem read returns whatever
object sits in the slot without checking it is a `Body::Stem`, so the old write is
invisible until something reads a **tail** of the same stem. The test picked exactly the
shape where the bug hides. Ninth such test in this project, second this phase.

**The method that found it is the one to reuse: `git archive` the pre-fix commit, build it,
and run the new test's own program against it.** That answers "would this test have caught
the bug" directly, and it is stronger than a mutation because it uses the real defect rather
than a synthetic stand-in. It produced three of the four findings. The controller should
have specified it rather than asking only for mutation proof.

### The hang, identified

The keyword gate hung for nine minutes under the implementer's own mutation and was reported
as an unidentified body. It is **`DO::test_DO_standardTest2P`**, found in about fifteen
minutes with a per-body 2-second timeout harness: its `Do a.i=1 To 7` has a pre-existing
`a.=0` default and **no independent termination signal**, so under the mutation the read-back
falls to the default `0` -- a valid number, not a derived-name string -- and `current`
recomputes to `1` forever inside `[1,7]`. A second effect went unreported: **`5-68` starts
raising rc 215** under the same mutation (`Error 41.1 Nonnumeric value ("I.92")`).

Pinned now by a `FOR 1000`-bounded regression test reproducing the mechanism.

### Carried to Task 15

**The keyword gate can hang rather than fail, and a hang is not a failure signal.** Task 15
owns the mutation script; a per-body timeout is the fix. Deliberately not done here.

## Task 15: the `base/bif` L1 harness, the 4c subset, `mutate-4c.sh`, and the gate

BASE `284ef543`. Implementer commits `f0426f09`, `8ac61371`, `1c94e50b`, `89debc85`.
Review verdicts: spec compliance MET, quality high, 0 Critical, 2 Important, 4 Minor.

**Task 15: fix round 1** -- two Important findings dispatched to the original implementer.

* **The task's own Step 4 change is unpinned.** Deleting the `phase-4c.txt` line from
  `collect_stress.rs`'s `read_subset` call leaves the workspace at 1,286 passed / 0 failed /
  75 binaries, byte-identical to the unmutated run. Criterion 4's amendment is green by
  construction. Fix requires a demonstrated falsification, not just a pin.
* **The two corrected figures were never written back into the plan.** Both were re-derived
  independently three times. The `::options` one decides, in five files, whether a row is
  resolved in the direction that asserts a value the interpreter must never produce.

**Task 15: minor (deferred): the gate's "twelve confidently-wrong rows" is 12 mismatches of 18
rows restored** -- the count conflates the two.

**Task 15: minor (deferred): `MISMATCH` is whitelisted as an exempt attribution in
`bif-exempt.txt`**, so a future value divergence could be absorbed with no phase owning it.
Zero such rows today, which is why it is Minor rather than Important, and also why it would be
invisible when it stops being true.

**Task 15: minor (deferred): the assessment table's row 12 drops its `of 6,293` denominator.**

**Task 15: minor (deferred): `has_novalue_option` would misread `::options novalue off`.**
Absent at r13178 and the misreading is in the conservative direction (it would drop rather than
resolve), so it costs coverage rather than correctness.

**Verified by the controller independently, before the review returned**, because both figures
are plan facts rather than code:

* `::options all syntax` does enable NOVALUE. Probed on the oracle from a fresh directory:
  `say nv1` raises `Error 98.986: Reference to unassigned variable "NV1"` at exit 158 under
  both `::options all syntax` and `::options novalue syntax`, where the no-directive control
  prints `NV1` at exit 0. Counts: 38 files match `novalue`, 5 match `all`, union **43 of 76**.
* A nesting- and quote-aware scan gives **24** `assertSame` inside `/* */`, 23 of them in
  `LINES.testGroup` alone, against the plan's 120-135. The reviewer's independent scan closes
  the reconciliation exactly at `6,268 + 24 + 1 = 6,293`.

The plan's 120-135 band was wrong by roughly five times, and its "a line-oriented scan extracts
over a hundred assertions that never run" was the conclusion drawn from it. Both were written
as measured. Neither the task review nor the plan review found them; the implementer did, from
the tree.

**Task 15: fix round 1 outcome.** Commits `70be78e2`, `e04fb46f`, `d0102460`. Scoped re-review:
both Important findings **ADDRESSED**, no new defects. 1,287 passed / 0 failed / 75 binaries,
clippy and fmt clean, all exit 0.

The re-review reproduced the falsification itself rather than accepting the report: dropping
`phase-4c.txt` from `SUBSET_FILES` exits 101 with the assertion naming the missing file, and a
scratch `phase-4d.txt` fires it in the other direction too. The pin's right-hand side is
`fs::read_dir` on the corpus directory, so it is not a second hand-maintained list.

**Where the plan's wrong 120-135 band came from, which is the useful part.** A scanner without
quote-awareness gives **118**, because `ERRORTEXT.testGroup:64` holds the string literal
`'Unmatched "/*" or quote.'`. The figure was not a rough estimate that drifted; it was a
specific, findable scanner defect, recorded in the plan as measured and marked do-not-re-derive.

**Task 15: fix round 2** -- one finding, promoted from the implementer's own scope note after the
re-review measured it.

* **`corpus.rs`'s union call site is vacuous, exactly as `collect_stress.rs`'s was.** Dropping
  `phase-4c.txt` there leaves plain mode AND `REXX_CORPUS_GATE=1` at **exit 0**; the report text
  moves from "50 of 50" to "42 of 42" and nothing asserts on the total. The gate assertion is
  `assert!(!gate || mismatches.is_empty())`, which an empty subset satisfies trivially.
  Criterion 1 of the gate document quotes that figure, so the criterion is satisfied by shrinking
  its own denominator.
* `coverage.rs` IS protected today, but incidentally: dropping the file fails the variant-coverage
  test with `4 in-scope variant(s) unwitnessed`. That depends on 4c owning a variant 4a and 4b do
  not witness, which is a property of the current corpus and not an invariant. Pinned as well.

The implementer flagged this itself and declined to widen scope without a finding, which was the
right call. The deferral's stated reason -- that a shrunken subset moves an "N of N" figure other
criteria read -- turned out to describe a number a human might eyeball, not a check anything runs.

**Task 15: fix round 2 outcome.** Commits `80ff9c1f`, `64ce92fe`. Scoped re-review: finding
**ADDRESSED**, no new defects, tree clean.

All three union call sites now carry the same device: a named `SUBSET_FILES` constant asserted
against `fs::read_dir` on `rust/corpus/`, filtered `phase-*.txt`. The re-reviewer read all three
right-hand sides and confirmed none derives from `SUBSET_FILES` or from a second hand-written
list.

Falsified in both directions by the re-reviewer, independently of the implementer:

* Drop `phase-4c.txt` from all three constants: exit 101, **1,285 passed / 4 failed**.
* Add a scratch `corpus/phase-4d.txt` wired nowhere: exit 101, 1,286 passed / 3 failed, exactly
  the three pins and nothing else.

**The second direction is the one that carries the proof.** A pin comparing its constant against
a second hand-written list stays green there, because no constant was touched. Only a pin that
reads the directory goes red. Record that as the falsification of choice for this shape of pin;
the first direction alone does not distinguish the two designs.

**Task 15: minor (deferred): the report's transcript of direction A reads 1,286 / 3 where the
measurement is 1,285 / 4.** It omits `collect_stress.rs`'s own round-1 pin, which also fires. The
error is in the safe direction (one more independent catcher than claimed) and is in an SDD
artifact rather than shipped prose, so it is recorded here rather than corrected in place.

**The ruling on the gate assertion stands, and the re-review agreed.** `corpus.rs`'s
`assert!(!subset.is_empty(), ...)` fires only when the union names zero programs, so a union
missing one whole file never reaches it. A non-zero-total assertion would be strictly weaker than
the pin: it catches a shrink to zero and passes on the 50->42 shrink that actually happened.

**Task 15: complete.** Commits `f0426f09`, `8ac61371`, `1c94e50b`, `89debc85`, `70be78e2`,
`e04fb46f`, `d0102460`, `80ff9c1f`, `64ce92fe`.

Verified independently by the re-reviewer, each unpiped: `cargo test --workspace --no-fail-fast`
**1,289 passed / 0 failed** over **75** binaries (67 `Running` + 8 `Doc-tests`), exit 0; clippy
clean, exit 0; `cargo fmt --all --check` clean, exit 0; `REXX_CORPUS_GATE=1` **50 of 50 matching**,
exit 0.

**Phase 4c's fifteen tasks are done.** Remaining before the phase closes: the final whole-branch
review, which owns triage of **every deferred item recorded above -- 37 of them, across 12
blocks**, not the five an earlier version of this sentence named.

**They are not findable by one obvious grep, which is how the undercount happened.** The blocks
are written five different ways: `**Deferred minors:**`, `**Deferred minors, for the 4c final
review:**`, `**Deferred, for the 4c final review:**`, `**Deferred to the final review:**`, and
`**Task N: minor (deferred):**`, four of them under a `###` heading rather than in a paragraph.
This finds all twelve blocks, from this directory:

```sh
/bin/grep -n "minor (deferred)\|Deferred minors\|Deferred, for the\|Deferred to the final review" progress.md
```

Sixteen hits, twelve blocks -- Task 15's five paragraphs are one block. The count of *items* is
larger than the count of blocks because a block routinely carries several, separated by
semicolons inside one paragraph; a triage that rules on a block's first sentence and stops has
left the rest of that block unowned, which happened to three of the eight the final review was
dispatched with.
