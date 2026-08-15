# Task 11 review: the `base/keyword` L1 table

Reviewed at `5b2de07a`, diff range `d47239c9..5b2de07a` (the first commit,
`6e49abad`, is the controller's plan correction and is context, not work under
review).

**Spec compliance: PASS.** All six steps are done, the four pins exist and each
one can fail, the denominator is stated in the criterion's own text, both false
`ootest/` comments are corrected, and the one deviation from the brief (the
narrower population) is declared with its cost measured rather than asserted.

**Code quality: PASS with required corrections.** Every finding below is in a
comment, a doc or a report, not in behaviour. I found no case where the
extractor, the rewrite or the harness produces a wrong result today, and the
whole-population differential below is positive evidence that it does not.

---

## What I verified by running, beyond what the lead had already run

### 1. The rewrite is faithful across the whole population, not just the samples

The lead's first risk was that a wrong translation manufactures results. The
strongest available test of that is not reading the rewriter -- it is running
its output under the oracle, since ooTest asserts every one of these methods
passes.

I extracted all **896** bodies and ran each one under the C++ oracle, one at a
time, each from a freshly created empty directory with the body copied in as
`p.rex` (so the 896 files are never on the oracle's external-routine search
path), under `ulimit -v 1048576` and a 10s timeout, reading stdout, stderr and
status separately.

**Zero bodies had an assertion fail under the oracle.** 892 of 896 exited 0
with every emitted marker reading `1`. The four exceptions are finding I3
below and are not rewrite defects.

For the 100 bodies the harness calls passing I also compared the two
interpreters directly:

* identical marker **sequences** (not just marker sets) -- 0 differences;
* identical **full stdout**, byte for byte, including each body's own `SAY`
  output -- 0 differences;
* identical exit status -- 0 differences.

So the 100 passes are passes in the oracle's sense as well, and the "carrying
713 of 1,773" figure is not a count of coincidences.

I also probed the rewriter directly for the shapes most likely to break a
line-oriented translator: CRLF sources with a trailing `--` comment, a block
comment running to end of line, a block comment spanning lines, nested block
comments, two calls on one line, `THEN`/`ELSE`/`OTHERWISE`/`WHEN` targets, a
one-line `DO ... END; ELSE`, quoted parens and commas in operands. All emitted
correct text. The CRLF case deserves a note because the corpus does contain CR
line endings (`FORWARD.testGroup` 190 lines, `TRACE_TraceObject.testGroup`
940): `blank_comments` blanks a `\r` that falls inside a comment, which makes
that blanked line one byte longer than `str::lines()`'s view of the original.
It cannot bite, because a `\r` is only inside a comment when the comment runs
to end of line, so every code position on that line is before the divergence.
That is worth knowing, not fixing.

The one shape that does misbehave is `self~assertSame(, b)`, which rewrites to
`(() == (b))` -- invalid Rexx where the original would raise from `use strict
arg`. Not in the corpus, and it fails loudly.

### 2. The six divergences are one defect, correctly labelled, and there is no seventh

Both recorded witnesses reproduce exactly as written (`[5 6]` / `[5 CV.1]` and
`[8]` / `[4]`). All six bodies reproduce: the oracle passes every assertion in
all six, `rexx-exec` fails at least one in each.

Every failing assertion in all six is downstream of a compound variable used as
a `DO` control variable. `DO::test_DO_standardTest2B` is the useful check,
because it tests simple, compound and **stem** control variables in one body:
assertions 1 (simple) and 3 (`Do cv. = 1 To 5`) pass under `rexx-exec` and only
2 (`Do cv.j = 1 To 5`) fails. So the stem case is not a second defect hiding
under the label.

The exclusions row says `LEAVE`/`ITERATE`/`END` naming a compound "is part of
the same shape". I isolated that, since it is the obvious candidate for a
seventh defect:

```
c=0; j=1; Do i.j=0 to 2; c=c+1; End i.j; say '['c']'                       both [3]
c=0; j=1; Do i.j=0 to 6; c=c+1; If c=2 Then Leave i.j; End i.j; ...        both [2]
c=0; j=1; Do i.j=0 to 6; c=c+1; If c=2 Then Iterate i.j; End i.j; ...      both [7]
```

`LEAVE`, `ITERATE` and `END` naming a compound all work. The single gap is that
the control variable is never bound. The characterisation and the recording are
honest.

### 3. `a_body_whose_assertions_never_run_is_not_a_pass` tests the real harness

It constructs a `KeywordBody`, but it drives `evaluate` -> `run_program` ->
`classify`, which is the same path `measured_failures` and
`keyword_assertions_differential` use. Mutating `classify` so `markers.is_empty()`
returns `Pass` instead of `NoAssertionExecuted` turns **exactly that test** red
and nothing else -- the 896-body run is unaffected, which is what the report
says and is why the test is needed. Not a fixture-only property.

### 4. The exempt set's derivation: independent for 790 rows, circular for 6

For a body that fails loudly the attribution is parsed out of `rexx-exec`'s own
stderr and originates in `instruction_owner` / `expr_owner`, static tables in
`rexx-exec/src/lib.rs:684` and `:769`. Nothing in that path reads
`keyword-exempt.txt`. For the 790 `4c` rows the derivation is genuinely
independent of what it is checked against.

For the 6 `defect:` rows it is not derived at all -- see finding I3.

### 5. The four pins can each fail, and the set is not degenerately satisfiable

* Conservation is not a tautology here, but it is much closer to one than the
  brief expected, and for a reason worth recording. A blocked body's `dropped`
  is taken from the independent counter (`in_code`), not from what the scanner
  managed to parse, and the file-level residual is swept into
  `OutsideTestMethod`. So a scanner *blindness* shows up as a drop, not as a
  conservation violation -- conservation cannot catch the failure the brief
  invoked it for. What it does catch is a rewriter that silently skips an
  occurrence it did see; see M7 for the one shape that reaches it.
* Absolute literals, floor and revision pinning are all real.
* **The four pins alone constrain counts and never program text.** A degenerate
  rewriter emitting `nop` for each call satisfies all four. What constrains the
  text is `the_falsification_proof`, the 14 mechanics tests, and the committed
  exempt set -- a rewriter that made every comparison trivially true would turn
  796 committed rows green and go red. The set as a whole is not degenerately
  satisfiable; the extractor's own four are.

### 6. The "adds coverage" claim is verified, and is stronger than reported

Mutating the rewrite to emit `if 1 then say ...` in an isolated copy of the
tree at `5b2de07a`:

* red: `a_then_target_assertion_becomes_a_single_say_not_a_nested_if` **and**
  `an_assertion_after_a_semicolon_is_found_not_only_a_line_starting_one`
  (the report names only the first);
* the whole harness stayed green, 6 of 6, in **both** REPORT and STRICT mode,
  with 100 of 896 bodies and 713 of 1,773 calls unchanged and the exempt set
  unchanged.

So the mutation is a real semantic change that the 896-body run cannot see, and
the unit tests are load-bearing. Sources restored and the copy re-verified green
(19/19 and 6/6) afterwards.

### 7. The `ootest/` comment corrections are complete

`.gitignore:6` is `ootest/`; `git ls-files ootest` returns 0. Both named
comments are corrected and no remaining `.rs` file claims `ootest/` is
checked-in test data. The new comments in `keyword-groups.txt`,
`extract_keyword.rs` and `keyword_assertions.rs` state the same facts and each
one checks out.

### 8. Passing bodies are credited static assertion counts -- but the overcount is zero today

`measured_failures` and the differential credit a passing body `body.assertions`,
the count as written, not the count executed. A passing body with an assertion
in a never-taken branch would be over-credited. Measured across the 100 passing
bodies: 713 static, 713 distinct marker indices actually emitted. No overcount
today. Forward hazard only (M9).

---

## Findings

### Important

**I1. "Zero assertSame calls sit on a continued line" is false, and the task's
own committed table says so.**
`rust/crates/rexx-extract/src/keyword.rs:450` ("Measured across this group:
**zero** `assertSame` calls sit on a continued line, so nothing needs a
multi-line parse") and `:409` ("Zero calls in this group sit on either side of
a join today (measured)"). The committed drop table three files away records
`ContinuedLine` at **2 methods / 3 calls**, and the guard does fire:
`NUMERIC::test_digits_fuzz_default` (2 calls) and `NUMERIC::test_form_default`
(1), both `self~assertSame(..., self~runDynamicSource( -`. The same claim is in
the report at line 82 ("**Zero** assertion calls use a line continuation") and,
worse, has been written into the plan at
`docs/superpowers/plans/2026-08-03-phase-4b-procedures-and-conditions.md:1127`
("No assertion call uses a line continuation, which removes one hazard"), where
the next task to read it will inherit it. Fix the plan first.

**I2. The list 4c and Phase 5 inherit mis-attributes 26 of its 145 calls to
Phase 5.**
`docs/superpowers/plans/l1-coverage.md:642`: "Each is blocked by a message send
in the body, which is Phase 5's". Measured over those 13 groups: message send
**119**, another `assert*` spelling **24**, outside any test-prefixed method
**2**. `SIGNAL`'s 24 calls are blocked only by other assertion spellings --
this extractor's own population choice, reachable without any of Phase 5 -- and
`GUARD`'s 2 are outside a `test`-prefixed method. The section's stated purpose
is to hand on lists rather than counts, so a wrong owner on a quarter of one
list is the part that matters.

**I3. The `unblocked_by` column is a hardcoded constant for the 6 `defect:`
rows, and four bodies already exist that it will mislabel.**
`rust/crates/rexx-exec/tests/keyword_assertions.rs:200` maps *every*
`AssertionFailed` to `"defect:compound-do-control-variable"`. For those 6 rows
the set test compares a constant against a file containing the same constant.
The module doc at `:46` and the exempt file header at
`rust/corpus/keyword-exempt.txt:10` both say the column is "derived, not
asserted by hand" without excepting them.

This is not hypothetical. Of the 896 bodies, four are **not equivalent to the
method they came from**, which the oracle run found:

* `CALL::test_expression`, `CALL::test_literal`, `CALL::test_on_name` -- the
  oracle itself fails them with `Error 43, Routine not found`, because the body
  calls `::routine`s defined elsewhere in the `.testGroup` that the standalone
  program does not carry;
* `NUMERIC::test_42` -- exits **3** under the oracle, because the body falls
  through into its own `dig: Return digits()` and a program's `RETURN` value is
  its exit status.

All four are exempt as `4c` today only because `rexx-exec` blocks on
`routine "label"` / `""` / `"CHARIN"` / `"DIGITS"` first. When 4c lands, three
will produce assertion failures labelled as the compound-`DO` defect they have
nothing to do with, and the fourth will classify as `Raised`, whose `"RAISED"`
attribution `every_exempt_attribution_is_a_known_phase_or_a_declared_defect`
rejects outright. Two consequences worth stating in the docs now: the `4c`
attribution is "what blocks it first", not "what would make it pass" (both the
exempt header and `:46` claim the latter), and `l1-coverage.md`'s "its pass rate
is a direct measure of 4c's remaining surface" is off by at least these four.

**I4. `classify_sends`'s closing comment states the opposite of the code.**
`rust/crates/rexx-extract/src/keyword.rs:659`: "the fall-through is a body with
a send this cascade did not name; it takes `MessageSend`, the conservative
end." The fall-through is `if saw_assert_same_list { AssertSameList } else {
OtherAssertion }` -- the least conservative end, and specifically the category
the report presents as the measured price of the population choice. The code is
right; the comment inverts the safety argument for a number that is being
reported as a cost.

### Minor

**M1.** `rust/crates/rexx-exec/tests/keyword_assertions.rs:612` -- the
falsification proof's doc says it appends `|| 'ZZZ'` inside the "already
-parenthesised **right** operand". The code prepends
`'ZZZ-FALSIFICATION-MARKER' ||` inside the **left** one. The test is sound; the
sentence is wrong on both side and direction.

**M2.** `rust/crates/rexx-extract/src/keyword.rs:549` -- `clause_boundary`'s
enumeration claims to cover "all 2,441 exact-spelling calls" and lists
2,038 + 263 + 128 + 8 + 2 = **2,439**. Measured: the other two are preceded by
`*-*`, inside `TRACE.testGroup`'s block comment. Behaviourally inert (they are
dropped as `InsideComment` before `clause_boundary` is reached), but it is a
universal claim with a count that does not add up.

**M3.** `rust/crates/rexx-extract/src/keyword.rs:576` -- "Two calls in this
group pass one [third argument]." Measured: **one** inside an extracted body
(`ADDRESS::test_environment_path_null`), and roughly **38** in the group.

**M4.** `rust/crates/rexx-extract/src/keyword.rs:715` -- `unquoted` returns
`Option<&str>` and all four call sites use only `.is_some()` / `.is_none()`;
its doc says it is "used to name the offending text in a block reason", which
nothing does -- every `detail` comes from `line.trim()` at the call site. Should
be a `bool` predicate.

**M5.** `rust/crates/rexx-extract/src/keyword.rs:263` --
`KeywordExtraction::dropped_for` has no caller anywhere, including the tests,
and its doc describes a summing over `DropReason::ALL` that nothing performs.
`the_drop_reasons_account_for_every_call_outside_the_population` tallies
`blocked` itself.

**M6.** `docs/superpowers/plans/l1-coverage.md:650` -- "`DoControlled` and
`LoopControlled` ... each has 24 assertions and every one of them is
`assertSameList`." Each file has **92** `self~assert*` calls: 24
`assertSameList`, 49 `assertSyntaxError`, 15 `assertEquals`, 4 `assertTrue`.
True only under an unstated "prefix-matched `self~assertSame`" reading.

**M7. Conservation has exactly one hole, and it fails loudly.** An exact-spelling
`self~assertSame` inside a **string literal** is counted by `count_assert_same`
and skipped by `rewrite_line`, and no `DropReason` covers it. Verified:

```
::method test_1
   s = 'self~assertSame(1,2)'
   self~assertSame(1, 1)
```

yields rows 1, dropped 0, calls 2 -- `rows + dropped != calls`. No such case
exists at r13178, and the failure is red rather than silent, so this is a note
rather than a defect. `count_assert_same`'s doc ("does not parse Rexx ... an
occurrence this counts and the scanner cannot use has to show up as a drop with
a reason") slightly overstates what the code guarantees.

**M8.** `UnparsedCallShape` is one of three categories pinned at zero and the
only one with no reachability witness -- `AssertSameList` and `NotAClause` both
have one. It **is** reachable (`self~assertSame(a)` and
`self~assertSame(a,b,c,d)` each produce it, verified), so the pin is not
vacuous; nothing in the suite demonstrates that, and the report's sentence about
constructing a witness for "the zero-valued category" covers only
`assertSameList`.

**M9.** `rust/crates/rexx-exec/tests/keyword_assertions.rs:325` -- a passing
body is credited its **static** assertion count. Measured today the executed
count is identical (713 = 713 over the 100 passing bodies), so the headline is
honest; `NoAssertionExecuted` guards only the all-zero case, not the partial
one.

---

## What I could not verify

* The report's claim that making `classify_sends` return `MessageSend`
  unconditionally turned exactly three tests red while the harness stayed
  green. I did not reproduce that mutation; the one mutation the report frames
  as an *added-coverage* claim (the nested `IF`) I did reproduce, and it holds.
* The redone ceiling estimate (1,132 methods / 2,436 calls; 1,034 / 1,942) and
  the "+169 calls / 659 other assertions" figure behind the population choice.
  I verified the committed `OtherAssertion` column (138 methods / 169 calls),
  which is the same quantity computed by the extractor's own rule, but not the
  independent whole-body probe the report says reproduced it.
* Whether any `base/keyword` test method depends on framework `setUp` or
  instance state. The oracle run found no false pass attributable to it, and
  the four context-loss cases it did find are I3, but I did not audit the
  `.testGroup` classes for setup methods.
* Long-run stability of `the_falsification_proof`: it requires the first
  passing body's first assertion to execute exactly once
  (`AssertionFailed { failed: 1, .. }`). True today; it would go red for a
  reason unrelated to what it tests if the first passing body ever became one
  whose first assertion sits in a loop.
