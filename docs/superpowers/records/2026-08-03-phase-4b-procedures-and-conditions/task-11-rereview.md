# Task 11 re-review: fix round 1 (`5b2de07a..8d6790c6`)

Scoped re-review, not a fresh one. Two questions: are I1-I4 / M1-M9 actually
fixed, and did the fix round introduce new false statements.

All work was done in a throwaway `git worktree` under the session scratchpad
(`rr11`, since removed) so the implementer's live tree was never touched.
Oracle probes ran from fresh empty directories under `ulimit -v 1048576`.

---

## Part 1: the named findings

| # | Verdict | Evidence |
|---|---|---|
| I1 | **fixed in code; the report's replacement number is wrong** | `keyword.rs:409` and `:453` no longer restate a number -- they point at `DropReason::ContinuedLine`'s own committed count, which is true. The plan's replacement (`877921d9`) is scoped to the drop table and is true. But see NF2: the report's new sentence gets the corpus count wrong. |
| I2 | **fixed** | Measured over the 13 groups: MessageSend **119**, OtherAssertion **24**, OutsideTestMethod **2**; SIGNAL = 24 + 3, GUARD = 13 + 2. Every number in the new `l1-coverage.md` table and the sentences under it checks out. |
| I3 | **fixed, with one new false sentence** | The module doc, the exempt header and `l1-coverage.md` all now state both limits. The four bodies' descriptions are accurate (see below). The sentence "blocks on a builtin first" is false for two of them -- NF1. |
| I4 | **fixed** (minor omission) | The code does return `MessageSend` from inside the loop; the tail is the weak end. The new enumeration omits `self~expect*`, which `blank_self_assertions` also blanks -- NF5. |
| M1 | **fixed** | Doc now says left operand / prepend; the code is `replacen("'@@ASSERTSAME 1' ((", "'@@ASSERTSAME 1' (('ZZZ-FALSIFICATION-MARKER' \|\| ")`. The parenthetical about `assertions.rs` is accurate: it wraps `expected` in parens and appends, for the stated regrouping reason. |
| M2 | **fixed** | Measured: 2,441 exact occurrences = nothing 2,038 + `;` 263 + `THEN` 128 + `:` 8 + `ELSE` 2 + `*-*` 2. The two `*-*` ones are `TRACE.testGroup:1099` and `:1133`, both inside `/* this is the expected TRACE ?A/?I output ... */`. Exactly as written. |
| M3 | **fixed** | Measured: **38** three-argument calls. Exactly one (`ADDRESS::test_environment_path_null`) is in an extracted body, and that body carries exactly 1 assertion, so "buys one row" is right. All 37 others are in bodies the extractor blocks (all `MessageSend`), including `CALL::"test_4"`. |
| M4 | **fixed** | `unquoted` is gone; `contains_unquoted` returns `bool`; all four call sites are boolean. Its new doc ("the offending text ... comes from the unblanked line at the call site") matches `keyword.rs:433`, `line.trim()`. |
| M5 | **fixed** | `dropped_for` removed; no references anywhere. |
| M6 | **fixed** | Measured on both files: 92 `self~assert*` = 49 `assertSyntaxError` + 24 `assertSameList` + 15 `assertEquals` + 4 `assertTrue`, and **zero** exact `assertSame`. (Each also has 8 `self~expectSyntax`, outside the `assert*` claim.) |
| M7 | **fixed** | `count_assert_same`'s doc now scopes the guarantee to "counts a substring", and `a_call_inside_a_string_literal_breaks_conservation_loudly` pins the hole with the clean case beside it. Passes. |
| M8 | **fixed, and it pins reachability** | `the_unparsed_call_shape_category_is_reachable` builds three sources (`(a)`, `(a,b,c,d)`, no argument list) and asserts `(rows, dropped) == (0, 1)` **and** `reason == UnparsedCallShape` for each -- it never asserts the corpus zero. Two adjacent successes (`(a,b)`, `(a,b,"msg")`) pin it to the argument count. Passes. |
| M9 | **fixed, and the headline is genuinely unchanged** | Ran the differential at `8d6790c6`: headline reads `100 of 896 bodies passing, carrying 713 of 1773 assertSame calls`, and no "did not run every assertion" line. Substituting `markers.len()` for the de-duplicated set changes the headline to **730** and lists 2 bodies -- so the committed count is the distinct-executed one, not a re-spelling of the static 713. |

### The two claims the reviewer could not reproduce

**The `classify_sends` mutation reproduces exactly.** At `5b2de07a`, replacing
the tail with `DropReason::MessageSend`:

```
cargo test -p rexx-extract --test extract_keyword   16 passed; 3 failed
    a_body_blocked_only_by_other_assertion_spellings_is_its_own_category
    assert_same_list_is_neither_counted_nor_rewritten
    the_drop_reasons_account_for_every_call_outside_the_population
cargo test -p rexx-exec  --test keyword_assertions   6 passed; 0 failed
```

Exactly the three named tests, harness green. Sources restored, worktree
re-verified at `8d6790c6` (22/22 extractor, 6/6 harness). The *explanation*
offered for the reviewer's non-reproduction is wrong -- NF3.

**`the_population_choices_price_is_reproduced_by_an_independent_rule` is a
real cross-check, not a tautology.** I re-ran its rule outside the test:
1,132 methods / 2,436 calls; narrow 896 / 1,773; wide 1,034 / 1,942; gain
138 / 169; 659 other `assert*`/`expect*` calls in the added bodies. Every
figure in the report's block reproduces.

On independence: the two routes share `rexx_extract::extract` (which the
test's own doc names) and `count_assert_same` (which it does not). What is
*not* shared is the classification -- `classify_sends` is a per-line cascade
over a comment-blanked view using `blank_calls`, and the test's rule is a
whole-body token-deletion over a separately written comment stripper -- and
the extractor additionally applies the ordering that produces `ContinuedLine`
/ `InsideComment` / `OutsideTestMethod` / `NotAClause`, which the test's rule
does not model at all. The `classify_sends` mutation above would turn this
test red too. So the claim holds; the only overstatement is "two independent
routes" glossing over the shared call counter.

### The four non-equivalent bodies (I3), checked under the oracle

All four descriptions are accurate about *why*:

* `CALL::test_expression` -> `call ("label")`, `CALL::test_literal` ->
  `call ""`, `CALL::test_on_name` -> `call on notready name trap`. All three
  rc **213**, `Error 43 ... Routine not found`. `CALL.testGroup:78-83` does
  define `::routine label`, `::routine "arg"`, `::routine ""`, `::routine
  trap`, none of which the standalone program carries. Correct as described.
* `NUMERIC::test_42` -> all four markers read `1`, then the body falls
  through into its own trailing `dig: Return digits()` after
  `Numeric Digits 3`, and the program exits **3**. Correct as described.

---

## Part 2: new false statements introduced by this round

### NF1 (the one that matters). "blocks on a builtin first" is false for two of the four.

* `rust/corpus/keyword-exempt.txt:24` -- "All four read 4c today only
  because rexx-exec blocks on a builtin first."
* `rust/crates/rexx-exec/tests/keyword_assertions.rs:86-87` -- same sentence.
* Also in `task-11-report.md:471-472` and in `8d6790c6`'s commit message.

Measured, running each extracted body under `rexx-run` at `8d6790c6`:

```
CALL::test_expression   rexx-exec: routine "label"  is not implemented (4c)
CALL::test_literal      rexx-exec: routine ""       is not implemented (4c)
CALL::test_on_name      rexx-exec: routine "CHARIN" is not implemented (4c)
NUMERIC::test_42        rexx-exec: routine "DIGITS" is not implemented (4c)
```

`CHARIN` and `DIGITS` are builtins. `label` and `""` are **not** -- they are
the very `::routine`s the same paragraph has just said the standalone program
does not carry. The true statement is the review's own: all four read `4c`
because `rexx-exec` blocks on an unresolved routine call first, two of which
happen to be builtins. This is a compression of a correct source sentence
into a wrong one, in the commit that corrected the previous error.

### NF2. The report's corrected line-continuation count is half the corpus figure.

`task-11-report.md:86-88` -- "ooRexx also continues on a trailing `-`, and
**3 calls in 2 methods** do (`NUMERIC::test_digits_fuzz_default`,
`NUMERIC::test_form_default`...)".

Measured over all 39 groups: **6** exact-spelling `assertSame` calls sit on a
line ending in a continuation, in **5** methods. The three the sentence omits
are `TRACE::test_trace_?a` (line 1104), `test_trace_?r` (1119) and
`test_trace_?i` (1145), all `self~assertSame("+++ *-* +++ Interactive *-*", -`.
They do not appear in the `ContinuedLine` drop category because those bodies
hit a message send on an earlier line and drop as `MessageSend` first --
which is exactly the "what it hits first, not what is true of it" distinction
this same commit introduced elsewhere. 3/2 is the drop-category count; the
sentence states it as the corpus count. (Zero use a trailing comma, so that
half is true. The committed `keyword.rs` comments avoid the number entirely
and are fine; the plan's replacement is scoped to the drop table and is fine.)

### NF3. The explanation for the reviewer's non-reproduction is inapplicable.

`task-11-report.md:499-501` -- "The likeliest reason it did not reproduce is
that `classify_sends` does not exist before `5b2de07a` -- it is that commit's
own function -- so the mutation has no site at any earlier commit."

Both halves of the fact are true (`git log -S` puts `classify_sends` in
`5b2de07a`; it is absent from `2e76303a`). The inference is not: the review
was conducted **at** `5b2de07a` (`task-11-review.md:3`), where the function
exists, and the review says "I did not reproduce that mutation", not that it
tried and could not. So the stated reason cannot be the reason. Harmless in
effect, but it is a fabricated cause presented as the likely one.

### NF4 (minor). `SymbolId(129)` belongs to a different program.

`docs/superpowers/plans/phase-4-exclusions.txt:951` and
`docs/superpowers/plans/l1-coverage.md:715-716` -- "parsing `do cv.j = 1 to
5` yields `Controlled { control: SymbolId(129), ... }`".

Measured: parsing exactly that line yields `SymbolId(125)`. `129` is what the
*witness* program `c=0; j=1; Do cv.j=1 To 5 ; c=c+1; End; say '['c cv.j']'`
produces. The load-bearing part is true and reproduces in all three spellings
I tried: `p.symbols.name(control) == "CV.J"`, i.e. the whole compound name is
one interned symbol and reaches the executor intact. The id is an artefact of
how many symbols were interned before it and should not be quoted at all.

### NF5 (minor). I4's new enumeration omits `self~expect*`.

`rust/crates/rexx-extract/src/keyword.rs:669-671` -- "`MessageSend` is
returned ... the moment any line shows a send that is neither an `assertSame`
nor an `assertSameList` nor another `self~assert*`".

`blank_self_assertions` blanks `self~assert…` **and** `self~expect…` (its own
doc says so, and the prefix test is `starts_with("assert") || starts_with
("expect")`), so a line whose only other send is `self~expectSyntax` does not
return `MessageSend`. The sentence's list is one item short.

### Not false, but imprecise: "a lower bound on what 4c would leave"

`keyword_assertions.rs:60-61` and `l1-coverage.md:629-631` replace "a direct
measure of 4c's remaining surface" with "a lower bound on what 4c would
leave". The correction is right in substance and the four bodies justify it,
but the phrase does not parse into a true quantitative claim in either
direction. What the evidence supports is: *the 790 `4c` rows are an upper
bound on what landing 4c would fix, because at least four of them would not
pass even then.* Worth one sentence's rewording.

### Not a false statement, but the same shape as M8

The new `partial` list in `build_report` is a reporting category standing at
zero with no reachability witness -- the exact gap M8 was raised for. The
commit is honest that it is zero today ("The report gains a line naming any
such body; zero today"), and I confirmed the branch is live by forcing it
(substituting `markers.len()` made it print 2 bodies). A constructed witness
would close it; nothing asserts it today.

---

## What I checked and found clean

* Every count in the rewritten `l1-coverage.md` prose: 119/24/2, 26 of 145,
  92 = 49+24+15+4, zero exact `assertSame` in `DoControlled`/`LoopControlled`.
* Every count in the rewritten `keyword.rs` docs: 2,439 + 2 = 2,441 with the
  two `*-*` cases located; 38 third-argument calls with exactly one extracted.
* The provenance/ownership reword's three code claims: `bind_control` is at
  `rexx-exec/src/run.rs:5284` and calls `slot_of(name)` on the whole name
  bytes; `instruction_owner` returns `None` for `InstructionKind::Do(_)`
  (`rexx-exec/src/lib.rs:688`); the parse interns `"CV.J"` whole.
* `keyword-exempt.txt` still holds 796 rows = 790 `4c` + 6 `defect:`.
* `RunOutcome::attribution` returns `None` only for `Pass`, so the differential's
  `_ => 0` arm cannot silently credit a non-passing body.
* Attribution of the oracle sweep to the review rather than to the
  implementer, in both the report and the module doc.

## Verdict

Every one of I1-I4 and M1-M9 is fixed, and the two claims the original review
could not check both reproduce. This round introduced one false statement in
committed files (NF1, two locations), two in the report (NF2, NF3), and two
minor ones (NF4, NF5). All are prose; none touches behaviour, and the tree
was green throughout.

**The task can close once NF1 is corrected in the two committed files.** NF2
and NF3 are report-only; NF4/NF5 are one-line edits worth folding in. None of
these needs another full review round -- only confirmation that the
replacement sentences say what the measurements above say.
