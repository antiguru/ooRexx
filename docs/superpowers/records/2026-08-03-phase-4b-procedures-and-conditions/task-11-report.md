# Task 11 report: the `base/keyword` L1 table

**Status: DONE_WITH_CONCERNS.** All six steps are complete and every gate is
green. Two concerns, both stated in full below: two numbers in the brief
disagreed with what I measured (reported before proceeding, and the plan has
since been corrected), and the table surfaced **six confirmed divergences in
`rexx-exec` that this task does not fix**, because they are 4a's `DO` and a
measuring task should not quietly rewrite the thing it measures.

Commits, in order:

| Hash | What |
|---|---|
| `2e76303a` | The extractor (`rexx_extract::keyword`) and its four pins |
| `664fac9d` | Comment handling: keep comments rather than strip them |
| `fda5bff9` | The L1 harness, the committed exempt set, `REXX_KEYWORD_GATE` |
| `ee33daf2` | `l1-coverage.md`, the `KNOWN GAPS` row, one false doc comment |
| `5b2de07a` | Drop reasons counted by category; floor 1,500 -> 1,750 |
| `8d6790c6` | Review round 1: I1-I4, M1-M9, the provenance reword |
| `b23986d9` | Review round 2: five statements round 1 introduced |

Verification, run from `rust/` at `b23986d9`:

```
cargo build --workspace                             exit 0
cargo test                                          1019 passed, 0 failed
cargo fmt --all --check                             exit 0
cargo clippy --workspace --all-targets -- -D warnings   exit 0
REXX_KEYWORD_GATE=1 cargo test -p rexx-exec --test keyword_assertions
                                                    6 passed, 0 failed
```

990 at the base commit, 1019 now: 29 tests added (22 extractor, 7 harness).

---

## Step 1: reproducing the brief's measurements

`svn info ootest` reads **r13178**, and `base/keyword` holds 39 `.testGroup`
files. Reproduced exactly:

| Brief | Measured |
|---|---|
| 4,567 `self~assert*`, all spellings | 4,567 |
| 2,561 matching the prefix `self~assertSame` | 2,561 |
| 2,441 exactly `assertSame` | 2,441 |
| 120 `assertSameList` | 120 |
| 54 rows from 2,561 calls (2.1%) | 54 |
| panic: `ASSIGNMENT.testGroup: 0 rows + 39 dropped != 265 assertSame calls` | verbatim |
| `ASSIGNMENT` 226 mid-line, `IF` 126 mid-line | 226, 126 |
| zero `assertSameList` in `base/expressions` | zero |

### Concern 1: two numbers disagreed

Both were reported to the team lead before I changed anything, rather than
silently substituted. Both are diagnostic prose rather than committed
literals, and both make the task's case *stronger*. The plan has since been
corrected (commit `6e49abad`, not mine).

**"405 of the 2,561 calls are not at the start of a line."** 405 is right as
a count of calls the scanner never sees (2,561 total − 2,156 seen), but the
attribution is off by two. Measured: **403** are genuinely mid-line. The
other **2** are at the start of their line but sit in `GUARD.testGroup`'s
`waiter_multiple` (lines 329–330), a `::method` whose name does not begin
`test`, so `extract` never yields its body at all. The blindness has two
causes, not one, and a body-shaped extractor has to account for calls
outside any test method as well — which mine does, with its own drop reason.

**"Twenty-six of the 39 groups yield zero."** Measured: **34 of 39** yield
zero rows. Nine of those contain no `assertSame` at all, so the sharper
figure is **25 of the 30** that contain any. Neither is 26; the 26 appears
to have been carried over from the later paragraph's "26 candidate
statement-shaped groups". Only five groups produce a row at all: `DO` 3,
`NUMERIC` 28, `REPLY` 1, `TRACE` 1, `VarRef` 21.

### The ceiling estimate, redone properly

The brief asked for this, flagging its own as approximate. Using `extract`'s
method splitting, comments stripped, exact spelling:

* 1,132 test methods carry 2,436 exact `assertSame` calls — 2,441 less the 2
  outside any test method and 3 inside comments.
* The brief's population (bodies whose only `~` is `self~assert*`):
  **1,034 methods / 1,942 calls**, against its approximate 1,033 / 1,850.
* **Zero** assertion calls use a *trailing-comma* line continuation. This
  was stated in the report and in two code comments as "zero use a line
  continuation", which review finding I1 showed to be false: ooRexx also
  continues on a trailing `-`. **6 calls in 5 methods do**, across the whole
  corpus -- `NUMERIC::test_digits_fuzz_default` (2),
  `NUMERIC::test_form_default` (1), and `TRACE::test_trace_?a`, `?r`, `?i`
  (1 each). Only the first three reach the `ContinuedLine` drop category;
  TRACE's three drop as `MessageSend`, because their bodies hit a message
  send earlier. **An earlier version of this bullet said "3 calls in 2
  methods", which is the drop-category count stated as the corpus count** --
  the same "what it hits first, not what is true of it" confusion this
  report's own I3 section describes, applied to itself.

---

## Decisions that were mine

### Denominator: the exact `assertSame` spelling, 2,441

Stated in the criterion's own text (`extract_keyword.rs`'s module doc). The
prefix spelling would inflate the denominator to 2,561 and classify 120
`assertSameList` calls as *dropped `assertSame` calls* — a reported
shortfall against a population that never existed. The cost of the narrower
choice is that those 120 are counted nowhere, which is correct: they are a
different method.

**2,126 of the group's 4,567 assertions (46.6%) are deliberately outside the
population** — every other spelling, largest being `assertTrue` (797),
`assertSyntaxError` (400), `assertEquals` (361).

### Population: bodies whose only `~` is `self~assertSame`

The brief's wider population (any `self~assert*` spelling) yields 1,942
calls against my 1,773 — **+169 calls, +9.5%**. Taking it would require
replacing 659 *other* assertions with `NOP` in the very bodies then reported
as passing. I took the narrower one, so **every assertion in an extracted
body is one the harness actually checks** and nothing is silently deleted.
This is the one place I knowingly took fewer rows than were available, and
it is why 1,773 rather than ~1,942 is the number below.

### Mechanism: the brief's, with the marker made unconditional

Each `self~assertSame(A, B)` becomes `say '@@ASSERTSAME n' ((A) == (B))`.
`OOREXXUNIT.CLS` (read directly) defines `assertSame` as
`use strict arg expected, actual, msg = ""` then `if \ (expected == actual)`,
so the comparison is the assertion unchanged, and the third argument is a
failure message that is read and discarded.

Three details are load-bearing rather than stylistic:

* **Unconditional, not `if … then say`.** It makes the number of assertions
  that actually *executed* observable — a loop prints one line per pass, and
  a body whose assertions never run prints nothing. A conditional `IF` would
  report neither, and would rebind a following `ELSE` to itself wherever the
  call it replaced was a `THEN` target (128 of the 2,441 are).
* **Each operand parenthesised, then the comparison parenthesised again.**
  Concatenation binds tighter than comparison in Rexx.
* **The marker is a string literal, not a symbol**, so the `(` after it can
  never be read as an argument list.

No new interpreter feature, no `self`, nothing from Phase 5.

### The floor: 1,750

Derived from what is actually achieved (1,773) and set just under it, about
1.3% below — not a round number picked for being comfortably clear of 54.
Deliberately *not* `TOTAL_ROWS` itself: a floor equal to the committed
literal would be that literal twice and would move whenever it moved. The
division of labour is that both fail together on a large regression, and the
absolute test fails alone on a small deliberate change.

---

## Step 2: the tests went red first

Written against a stub returning an empty extraction, with the numbers
**predicted** from Step 1 rather than back-filled:

```
test the_committed_group_list_matches_the_checkout ... ok
test every_assert_same_is_a_row_or_an_accounted_for_drop ... ok
test the_row_floor ... FAILED     only 0 of 2441 ... below the floor of 1500
test base_keyword_yields_the_measured_counts ... FAILED  ADDRESS: measured 0/0, committed 222/11
```

The brief predicted this shape exactly: the absolute-literal test and the
floor are the red, and conservation passes at `0 + 0 == 0`. It does so at the
stub only because `count_assert_same` was stubbed too; in the finished
extractor it counts substrings independently of the scanner, which is what
makes the law non-vacuous.

The implementation then reproduced **1,773 rows / 668 dropped and the whole
per-group table** as predicted, with one correction: my prediction's `calls`
column used prefix counts, which I replaced with exact-spelling ones.

---

## The four pins (`rexx-extract/tests/extract_keyword.rs`)

1. **Conservation** — `rows + dropped == calls` per group, `calls` counting
   every occurrence including mid-line ones, from a substring counter that
   does not parse Rexx and shares no judgement with the scanner.
2. **Absolute literals** — 39 files, 2,441 calls, 1,773 rows, 668 dropped,
   and every group's own row count by name.
3. **A real floor** — 1,750.
4. **Revision pinning** — `r13178` in every failure message, plus the
   scanned file list committed as `rust/corpus/keyword-groups.txt` the way
   `phase-4a.txt` is pinned.

Plus a fifth pin the lead asked for after the first pass, and 14 mechanics
tests for the shapes `base/expressions` never contains.

### The 668 accounted for by reason

`calls - rows` as one number says only that something was lost. The
breakdown is committed and asserted to sum to 668:

| Reason | methods | calls |
|---|---|---|
| body uses a message send | 96 | 491 |
| body uses another `assert*` spelling | 138 | 169 |
| inside a comment, not a call | 3 | 3 |
| on a continued line | 2 | 3 |
| outside any `test`-prefixed method | 1 | 2 |
| body's only send is `assertSameList` | 0 | 0 |
| unparsed call shape | 0 | 0 |
| not a clause of its own | 0 | 0 |

**`message send` (491) is the Phase 5 share.** **`another assert* spelling`
(169) is the measured price of the population choice** — precisely the set a
wider rule would admit by rewriting those calls to `NOP`. It reproduces, from
a completely different rule, the 138 methods / 169 calls an independent
whole-body probe measured before the extractor existed, which is the closest
thing here to an independent confirmation.

The category is decided per **body**, not per line: a body whose first
offending line is an `assertTrue` but which sends a real message later reads
`MessageSend`, since the question is what would unblock the body. `detail`
still names the first offending line.

The three zero rows are kept and counted — a category pinned at zero fails
the first time the corpus grows one. `assertSameList` reading zero is **not**
"no body mixes the two spellings": five do (`DoOver::test_do_over`,
`DoWith::test_do_with`, `LoopOver::test_loop_over`,
`LoopWith::test_loop_with`, `REPLY::test_reply_same_replyAssert`), and every
one also sends a real message. A unit test constructs the mixed body so that
the zero-valued category is demonstrably *reachable* — a category that could
never fire would prove nothing by being pinned at zero.

---

## Steps 5 and 6: the result

**100 of 896 bodies pass, carrying 713 of 1,773 assertSame calls (40.2%).**

| Group | bodies | assertions |
|---|---|---|
| ADDRESS | 0/10 | 0/11 |
| ASSIGNMENT | 2/8 | 206/265 |
| CALL | 2/8 | 2/50 |
| DO | 33/85 | 432/507 |
| IF | 20/25 | 24/29 |
| INTERPRET | 1/2 | 2/3 |
| ITERATE | 17/18 | 19/20 |
| LEAVE | 14/15 | 16/17 |
| NOP | 2/2 | 3/3 |
| NUMERIC | 1/51 | 1/63 |
| PARSE | 0/659 | 0/778 |
| SELECT | 7/8 | 7/8 |
| SelectCase | 1/1 | 1/1 |
| TRACE | 0/3 | 0/13 |
| VarRef | 0/1 | 0/5 |

Both columns are reported because they diverge: `ASSIGNMENT` passes 2 of 8
bodies but 206 of 265 assertions, since that group writes hundreds of
assertions into a few very long methods and one unrunnable line takes a
whole body.

**Every phase gap here is 4c's. Not one body is blocked by Phase 5.** So 4b
owes this table nothing further, and its rate is a direct measure of 4c's
remaining surface. `PARSE` alone is 656 bodies; the rest are builtins
(`ARG` 51, `COPIES` 19, `DIGITS` 11, `FORM` 10, `FUZZ` 9, …), `ADDRESS` 10,
`PULL` 1.

The lists 4c and Phase 5 inherit are in `l1-coverage.md`, as lists and not
as counts: 13 groups with assertions but no extractable body (145 calls, all
blocked by a message send), 11 groups with no exact-spelling `assertSame` at
all, 4 groups that extract bodies and pass none.

`REXX_KEYWORD_GATE` follows the existing convention and passes today in both
modes. It fails on the same condition the always-on set test asserts, so the
gate adds no second notion of correctness.

---

## Concern 2: six confirmed divergences, left unfixed

Six bodies ran to completion and disagreed. **These are not gaps.** Each was
re-run in its rewritten form under the C++ oracle, which passes all six:

```
c=0; j=1; Do cv.j=1 To 5 ; c=c+1; End; say '['c cv.j']'
  oracle [5 6]   rexx-exec [5 CV.1]   -- a compound control variable is
                                         never assigned at all
a.=0; i=1; Do a.i=1 To 3; If i>7 Then Leave; i=i+1; End; say '['i']'
  oracle [8]     rexx-exec [4]        -- the oracle re-resolves the compound
                                         name each pass, so which variable
                                         controls the loop changes with `i`
```

`DO::test_DO_standardTest2B`, `2P`, `2Q`, `5-69`, `ITERATE::test_12`,
`LEAVE::test_11`. Recorded in `phase-4-exclusions.txt`'s `KNOWN GAPS` (a
section explicitly pinned so that *adding* a row needs no amendment) and in
`keyword-exempt.txt` under `defect:compound-do-control-variable`. Not fixed:
this task measures, and a `DO` change belongs with the tasks that own `DO`.
When it is fixed those six start passing and the exempt-set test goes red
until they are removed, which is the intended way to notice.

### A witness that does not depend on this harness

The six are ooTest bodies, so believing them means believing the body
rewrite. This does not:

```
j = 1
do cv.j = 1 to 3
end
say '['cv.1']' '['cv.j']'
```

oracle `[4] [4]`, rexx-exec `[CV.1] [CV.1]`. Two lines, no ooTest, no
message send. Reproduced independently by the team lead in a fresh
directory.

### Provenance is 4a's; ownership is unassigned

An earlier version of this section, and of the `KNOWN GAPS` row, the
`l1-coverage.md` section and the commit messages of `ee33daf2` and
`5b2de07a`, all said "this is a 4a defect". **That conflates two questions
and only one of the answers is 4a.**

* **Whose code contains the defect: 4a's.** `rexx-exec/src/run.rs:5284`'s
  `bind_control` is the executor's controlled-`DO` path, and
  `instruction_owner` returns `None` for `InstructionKind::Do`, which means
  *implemented*, not *deferred to a later phase*.
* **Whose backlog delivers the fix: nobody's.** Nothing schedules compound
  control variables and there is no owner string to attach. That is exactly
  why the row belongs in `KNOWN GAPS` ("a real divergence with no owner
  assigned") rather than in `EXCLUSIONS`.

The `KNOWN GAPS` row already ends "No owner is assigned", so it was
internally consistent, but the parenthetical read as an ownership claim and
Task 12 has to rule on precisely that. Approved reword, to be folded into
the review fix round: say plainly in all three places that **the defective
code is 4a's and the gap is unowned**. The commit-message text cannot be
edited, which is why the correction is recorded here.

**And it is not a `rexx-parse` gap**, which was the alternative worth ruling
out. Measured: parsing `do cv.j = 1 to 5` yields `Controlled { control:
SymbolId(129), … }` with `symbols.name(129) == "CV.J"` -- `cv.j` is a single
symbol token in Rexx, so the full compound name is interned and reaches the
executor intact. `bind_control` then takes those name bytes and calls
`slot_of`, a flat name-to-slot lookup with no tail resolution, while the
same executor resolves the same name correctly in `say cv.j` one line
later. The parse delivers what a correct implementation needs; the `DO` path
does not use the compound resolution the crate already has.

### `DO ... OVER`: still unverified, and my stated reason for that was wrong

Reading the AST, `LoopKind::Over` also carries `control: SymbolId` and binds
through the same `bind_control`, so `do cv.j over …` should share the
defect -- which would make this one *function* rather than one construct.
That remains **unverified and is deliberately not in the `KNOWN GAPS` row**.

I first reported it as blocked on Phase 5, on the strength of a probe whose
collection expression was `'a b c'~makearray(' ')` -- which contains a
message send, so the run died at Phase 5 before reaching the loop. **That
was a bad probe, not a real blocker**, and the team lead caught it. A stem
needs no message send at all:

```
a.1 = 'p'
a.2 = 'q'
j = 1
do cv.j over a.
  say 'pass' '['cv.1']' '['cv.j']'
end
```

oracle `pass [2] [2]` / `pass [1] [1]` at rc 0; rexx-exec exits **120** with
`rexx-exec: DO is not implemented`. Verified here as well as by the lead.
So the real blocker is that `DO OVER` is not implemented at all, and
**this becomes testable when `DO OVER` lands, not when Phase 5 does** --
recorded because "blocked on Phase 5" would park it behind the wrong
milestone.

The methodological note, since it is the shape that recurs: the first two
mistakes of this kind on this task were code-reading inferences that got the
answer right and the mechanism wrong. This third one was different and
worth distinguishing, because the fix differs -- I *did* run it, but the
witness I chose failed for a reason other than the one under test. "Run it
instead of explaining it" does not help there; checking what the probe
actually exercised does.

## A defect of my own, found by running rather than reading

Two further bodies were failing and looked identical to the six. They were
**my** bug. The first draft stripped comments to spaces; a Rexx comment ends
a token *without* contributing a blank. Measured on the oracle:

```
say '['1/**/05']'   -> [105]        say '['1 /**/ 05']'  -> [1 05]
zz = 1; say '['zz/**/05']'  -> [105]
```

`ITERATE::test_11` and `LEAVE::test_10` both write their expected value as
`(11/**/ 1/**irrelevant**/05  10/*…*/)` and depend on exactly this. Comments
are now preserved verbatim, with the scan working from a byte-aligned
*blanked* view for structure and the original for text — which also makes a
comma or paren inside a comment structurally invisible.

**This was only findable by running the rewritten programs under the oracle
as well**, which is not something the L1 rung requires. It is worth saying
plainly: two of eight apparent divergences were artefacts of the harness, and
nothing short of the differential check separated them.

## Evidence that the tests can fail, and that they add coverage

Per Task 10's distinction — a red mutation proves a test *can* fail, not that
it catches anything the suite misses.

**The exempt set, all three directions, by mutating the committed file:**

| Mutation | Result |
|---|---|
| remove `DO::test_DO_standardTest2B` | `is failing … and is not on the committed exempt list`; STRICT also fails |
| add a passing body (`IF::test_3`) | `now PASSES but is still on the committed exempt list -- remove it` |
| change one attribution to `Phase 5` | `is listed as "Phase 5" but now measures "4c" -- its blocker moved` |

**The drop-reason accounting.** Making `classify_sends` return `MessageSend`
unconditionally turned three tests red
(`the_drop_reasons_account_for_every_call_outside_the_population`,
`a_body_blocked_only_by_other_assertion_spellings_is_its_own_category`,
`assert_same_list_is_neither_counted_nor_rewritten`) while the harness stayed
green — correctly, since the gate keys on run outcomes rather than on
categories.

**Added coverage, verified against the suite without the test.** Mutating
the rewrite to emit `if 1 then say …` (a nested `IF`, the shape that would
steal a following `ELSE`) turned
`a_then_target_assertion_becomes_a_single_say_not_a_nested_if` red — while
**the whole harness stayed green, 6 of 6, exempt set unchanged**. So the
unit test catches something neither the corpus-level pins nor the 896-body
run does.

Two harness tests are forward guards and are described as such rather than
claimed as coverage: `a_body_whose_assertions_never_run_is_not_a_pass` uses
a constructed body because no body in the corpus emits zero markers today,
and `an_assertion_used_as_an_operand_blocks…` guards a shape all 2,441 calls
currently avoid.

## Two false comments corrected

Both named in the brief. `tests/assertions.rs`'s `suite_root` and
`rexx-extract-assertions.rs` each claimed `ootest/` is checked-in test data.
It is git-ignored (`.gitignore:6`), has zero tracked files, and is an SVN
working copy. A third, of my own making, was corrected in `ee33daf2`: after
the comment fix, `KeywordBody::program`'s doc still said comments were
removed and that the program contains no `~`.


## Review round 1

Review returned **spec compliance PASS, code quality PASS with required
corrections**, and found **no behavioural defect** -- it extracted all 896
bodies, ran each under the oracle, and found zero assertion failures among
them; for the 100 passing bodies the two interpreters agree on marker
sequence, full stdout and exit status.

All four Important and all nine Minor findings were verified against the tree
before being acted on, and **all thirteen held**. They are fixed in
`8d6790c6`, whose message records each one. The two headline corrections to
this report:

* **I1** -- the "zero line continuations" claim above, now corrected in
  place.
* **I3** -- four extracted bodies are **not equivalent to the method they
  came from**: `CALL::test_expression`, `CALL::test_literal` and
  `CALL::test_on_name` fail under the C++ oracle itself with `Error 43,
  Routine not found`, because they call `::routine`s the standalone program
  does not carry; `NUMERIC::test_42` exits 3, because the body falls through
  into its own `dig: Return digits()`. All four read `4c` today only because
  `rexx-exec` blocks on a builtin first. Consequently the claim that the pass
  rate is "a direct measure of 4c's remaining surface" was too strong and is
  now stated as a **lower bound on what 4c would leave**.

### The two claims the reviewer could not reproduce

Both re-run at `5b2de07a`, both reproduce, commands recorded here so neither
has to be taken on trust again.

**1. The `classify_sends` mutation.** Replace the tail of `classify_sends` in
`rust/crates/rexx-extract/src/keyword.rs`

```rust
    if saw_assert_same_list { DropReason::AssertSameList } else { DropReason::OtherAssertion }
```

with `DropReason::MessageSend`, then from `rust/`:

```
cargo test -p rexx-extract --test extract_keyword
    16 passed; 3 failed
      a_body_blocked_only_by_other_assertion_spellings_is_its_own_category
      assert_same_list_is_neither_counted_nor_rewritten
      the_drop_reasons_account_for_every_call_outside_the_population
cargo test -p rexx-exec --test keyword_assertions
    6 passed; 0 failed
```

Exactly three red, harness green.

**An earlier version of this section explained the non-reproduction by saying
`classify_sends` does not exist before `5b2de07a`. That is withdrawn.** Both
halves of the fact are true -- the function is that commit's own, and it does
not exist earlier -- but the inference does not follow: the review was
conducted *at* `5b2de07a`, where the function exists, and it reported that it
had not attempted the mutation, not that it had tried and failed. There is
nothing to explain, and I explained it anyway from a true premise. Third time
on this task that shape has cost something.

**2. The independent ceiling probe.** This one is no longer a transcript: it
is now a test, `the_population_choices_price_is_reproduced_by_an_independent_rule`
in `rust/crates/rexx-extract/tests/extract_keyword.rs`, so it is re-run on
every `cargo test` and cannot rot.

It re-derives the population figures by a **different rule** from the
extractor's: split methods with `extract`, strip comments with a local
stripper, and ask whether deleting every `self~assert*`/`self~expect*` token
leaves a `~` behind -- rather than the per-body cascade `classify_sends`
uses. Measured:

```
methods with >=1 exact assertSame: 1132, carrying 2436 calls
narrow (only self~assertSame):      896 methods, 1773 calls
wide   (only self~assert*/expect*): 1034 methods, 1942 calls
the wider rule's gain:              138 methods,  169 calls
other assert*/expect* calls inside the bodies it would add: 659
```

and the test asserts that gain equals the extractor's own `OtherAssertion`
column. Two routes, one number. The reviewer verified the column; what it
could not verify was that a second, independent rule lands on it, which is
what makes it a cross-check rather than the same rule reported twice.

### Correction to two commit messages

`ee33daf2` and `5b2de07a` both say "this is a 4a defect". That conflates
provenance with ownership: **the defective code is 4a's** (`bind_control`,
and `instruction_owner` returns `None` for `InstructionKind::Do`, meaning
implemented rather than deferred), while **the gap is unowned**, since
nothing schedules compound control variables -- which is what puts it in
`KNOWN GAPS` rather than `EXCLUSIONS`. Commit text cannot be edited, so the
correction lives here; `phase-4-exclusions.txt` and `l1-coverage.md` now say
it plainly.


## Review round 2

The re-review confirmed **all thirteen round-1 findings fixed** and both
previously unreproduced claims reproduced. It also found that **round 1
introduced five new false statements**, which is the shape this project
measured across Tasks 7-9 and the reason a scoped re-review exists. All five
were verified before being acted on; all five held. Fixed in `b23986d9`.

The one that blocked close, **NF1**, is worth recording as a failure mode
rather than an instance of carelessness. Two committed files said the four
non-equivalent bodies "read `4c` today only because `rexx-exec` blocks on a
**builtin** first". Measured:

```
CALL::test_expression   routine "label"   is not implemented (4c)
CALL::test_literal      routine ""        is not implemented (4c)
CALL::test_on_name      routine "CHARIN"  is not implemented (4c)
NUMERIC::test_42        routine "DIGITS"  is not implemented (4c)
```

Only `CHARIN` and `DIGITS` are builtins. `label` and `""` are the very
`::routine`s that *the same paragraph, two lines above* says the standalone
program does not carry. The correct sentence was already there and I
compressed it into a word that contradicted it. That is the same shape as
round 1's own I4 finding, arriving in the commit that fixed I4.

The other four: **NF4**, both docs quoted `SymbolId(129)` when the same loop
reads 125, 127 or 129 depending on what precedes it (the name `"CV.J"` is the
part that reproduces, and the number is now dropped rather than corrected);
**NF5**, `classify_sends`'s comment omitted `self~expect*` from the list of
sends that do not make a body a `MessageSend`; the **rewording**, where
"a lower bound on what 4c would leave" was true in spirit and not a
quantitative claim in either direction -- the supported statement is that the
**790 `4c` rows are an upper bound on what landing 4c would fix**; and
**NF2/NF3**, corrected in place above.

The optional item was taken: the `partial` reporting category round 1 added
was standing at zero with nothing showing it could fire, which is the gap M8
was raised for applied to a report line. It now has a witness -- and the
witness earned its place immediately, since the adjacent-success half of it
was wrong on first writing and the test caught it.

## What this task did not promise

`tests/assertions.rs`'s 35 exempt rows did not move, and nothing here
touches them.
