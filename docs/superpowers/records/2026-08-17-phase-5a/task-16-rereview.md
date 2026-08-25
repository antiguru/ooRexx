# Task 16 re-review: fix round 1

Scoped re-review of `7eabdfb59..8610d6334` -- two commits, `8265ead58` (fix round 1) and
`8610d6334` (the reviewer's own addition to `oracle-crashes.txt` entry 7). Read against
`task-16-review.md`'s seven findings, `task-16-report.md`'s fix-round section and
`task-16-brief.md`.

**Verdict: CHANGES REQUESTED.** Six defects, every one of them a sentence in a comment or in the
report. No behaviour defect, no dead instrument, no number that gates anything. All seven original
findings are closed on their substance; four of the six new defects are statements *about* those
fixes that the fixes themselves do not support.

Gates were not re-run for their own sake -- they are established at `8610d6334` -- but the tree was
mutated twice for finding 3 and restored, and the restored tree is green again (see **Tree state**).

## Priority 1: my own commit, `8610d6334`

**The stronger clause is false, and it is worse than "unsupported": a runnable counterexample
kills it.**

The committed sentence is:

> So the oracle refuses at translation time a `WHEN` expression that names no exposed variable, and
> only an expression that *can* in principle be satisfied by another activity reaches the wait.

and the commit message goes further -- "because the oracle rejects at translation time a `WHEN`
expression that no other activity could ever satisfy. Only an expression that could in principle be
satisfied reaches the wait."

The first clause holds. The second does not, in both directions, and neither direction needed the
C++ to settle it. Two probes, fresh empty directory, absolute paths, three descriptors read
separately, both sides bounded, three runs each:

```
guard on when w = 0    (w local, expression already TRUE)
  oracle rc=157 stdout=[] stderr=[     5 *-* guard on when w = 0|... Translation error.|
                                     Error 99.913:  GUARD instruction did not include
                                     references to exposed variables.|]
  crate ir / tree-walker rc=120  rexx-exec: 99.913: GUARD instruction did not include ...

guard on when v = v    (v exposed, expression can never be false)
  oracle rc=0 stdout=[ran|] stderr=[]
  crate ir / tree-walker rc=0 stdout=[ran|] stderr=[]
```

An expression that is *already satisfied*, and satisfied without any other activity, is refused;
an expression that no activity's change could ever alter runs. So reaching the wait is neither
implied by nor implies satisfiability. What the check tests is a reference test on names:
`LanguageParser::captureGuardVariable` adds a retriever only when `isExposed(varname)`
(`parser/LanguageParser.cpp:2024`-`:2027`), and `guardNew` raises when that set is empty
(`parser/InstructionParser.cpp:2640`-`:2646`).

The C++ *does* support a weaker version of what I was reaching for, and it is worth quoting because
it is the maintainers' own words at the raise site (`InstructionParser.cpp:2642`):

```
// if using GUARD WHEN, we will never wake up if there are
// not at least one object variable accessed.
```

That is the check's motivation, not its extension. `expose v; v = 0; guard on when v = 1 & 1 = 0`
compiles (it names `v`) and cannot be woken by anything -- I did **not** run that, because it is the
entry-7 shape; the claim that it compiles is derived from the parser above, not measured.

**Required narrowing.** Keep the measured clause; replace the inference with the reference test and
the C++'s own reason, marked as the reason rather than as the rule. Add the `w = 0` and `v = v`
readings -- they are cheap, they are already in this file's neighbour paragraph's idiom, and they are
what makes the narrowing checkable next time. The commit message cannot be edited without rewriting
history; the file is the record that matters, and it should not be left agreeing with it.

**Everything else in the commit stands.** The block reading it settles (rc 137, 0.00 s user and
0.00 s system over 5.00 s elapsed, nothing on either descriptor) closes the implementer's standing
concern, and the entry's cause paragraph checks out line for line: `guardOn()`/`guardOff()` is set
*before* the expression and the same `do { guardWait() } while (!truthValue(...))` loop runs either
way (`instructions/GuardInstruction.cpp:152`-`:185`), so "`GUARD OFF WHEN` blocks identically" is
right, and `:167`-`:185` brackets exactly the false-branch loop while `:168` is the
`truthValue(Error_Logical_value_guard)` the sub-number comes from.

One nit inside that paragraph, park: "the flag decides only what the wait holds while it sleeps" is
backwards. `guardWait` releases the lock either way (the C++ comment at `:150` says so); the flag
decides what is held when the activity is *not* sleeping.

## Priority 2: the seven findings

| # | Verdict | Basis |
| --- | --- | --- |
| 1 | **CLOSED** | Entry 7 present, header's third mode present, `Loud::guard_when_false` names both the crashes file and its instrument, as `array_index_hole` does. New defect **E** sits beside it. |
| 2 | **CLOSED** | Re-derived from the TSV myself. New defect **G** is a label, not the figure. |
| 3 | **CLOSED** | All three universals re-derived by running them; every figure reproduces. |
| 4 | **CLOSED** on the claim, **OPEN** on its reason -- new defect **C**. |
| 5 | **CLOSED** on the marking, **OPEN** on the count -- new defect **F**. |
| 6 | **CLOSED** | Readers named, and the four names are the four readers. New defect **B** is the justification attached to the fix. |
| 7 | **CLOSED** on the contradiction and the sole-instrument record; **OPEN** on a third row the rewrite mis-grouped -- new defect **D**. |

### Finding 2, verified from the TSV

Every non-`dispatchclass` row of the report's contribution table is byte-identical to
`bench-baselines/phase-5a-arms.tsv`'s `16-contribution / across_builds / instructions:u` median,
brackets included, `instrument` and `size` kept throughout. The report's `awk` reproduces:

```
dispatchclass  base  ir  small  med=12916149038 min=12890142050 spread=0.2018%
dispatchclass  base  ir  large  med=25850879966 min=25730852836 spread=0.4665%
```

60 `absolute` cells; exactly those two exceed 0.0018%, and the third-largest is
`rexxcps/changed/tw/small` at 0.00183%, which the report's own `%.4f` prints as `0.0018%`. So "the
only contaminated cell" holds and "<= 0.0018% everywhere else" is true at the precision it is
quoted at.

The corrected figures are the right ones and the reconciliation closes:
`1.012279 x 1.004202 = 1.016532` against the accumulated `pinned>head` ir large read of `1.016525`,
agreeing to four decimals either way (min/min gives `1.004197` and `1.016527`). The gate conclusion
is unchanged: +0.42% is under the 1% line.

Had the median been the honest statistic and the min the artifact, this projection would have looked
exactly the same -- median-vs-min cannot tell you which one moved. What rules that out is Task 15's
independently measured `base` median of 25,731,033,256, which agrees with this sitting's `min` and
not with its median. That reasoning is in the previous review and should be in the report, which
currently argues only from the spread.

### Finding 3, verified by running both mutations

`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`, one mutation at a time on the
shipped tree, restored between:

```
98.936/98.937 raise deleted     exit 101, exactly 2 "... FAILED" in the whole workspace
  test run::tests::a_value_returned_after_a_reply_reports_the_oracles_own_98_936 ... FAILED
  test corpus_differential ... FAILED
  183 of 185 matching; mismatches (2): lang/method_reply.rex, lang/method_reply_exit_status.rex

98.935 raise deleted            exit 101, exactly 2 "... FAILED" in the whole workspace
  test run::tests::a_second_reply_reports_the_oracles_own_98_935 ... FAILED
  test corpus_differential ... FAILED
  184 of 185 matching; mismatches (1)
```

Both counts and both test names are the report's. `grep -c '\.\.\. FAILED'` is 2 in each log, so
"reddens two things" is exact and not a floor. The third universal I re-derived from the table
above: tw moves further than ir on `alloc4c`, `arith`, `compound`, `strings` and `varlookup`;
`emptyloop` is equal; `dispatchclass` inverts; `rexxcps` moves in opposite directions. Five of eight.

### Finding 5, verified by counting the markers myself

All fifteen marker sites, each resolved to the item it documents:

```
roots.rs:146      RootSet::park            roots.rs:341      RootSet::frame_aliases
activation.rs:397 ReplyState::Owed         activation.rs:568 Activation::reply
activation.rs:1231 Activation::object_roots
dispatch.rs:1526  park_reply               dispatch.rs:1583  run_deferred_replies
dispatch.rs:1626  resume_reply
lib.rs:1214       Loud::guard_when_false   lib.rs:1240       Loud::reply_inside_construct
lib.rs:2459       Interp::deferred         lib.rs:3492       Argument::object_roots
lib.rs:3532       CallContext::object_roots
run.rs:3516       exec_guard               run.rs:3576/3585  exec_reply (both kinds)
run.rs:3676       returned_value's check   run.rs:11910      raised_guard_not_logical
```

The marking is genuinely complete this time: the two functions the earlier report misdescribed,
`resume_reply` and `run_deferred_replies`, both carry SCHEDULING markers, and the whole parked group
is marked. Every item in the report's two lists sits at one of these sites. The parked group's
deletability claim is sound -- `park`, `release`, `live_parked` and `frame_aliases` have exactly one
caller each and all four are in `park_reply`/`run_deferred_replies`/`resume_reply`.

The defect is the derivation, not the marking -- see **F**.

### Finding 7, verified

`Loud::reply_inside_construct` is constructed at `run.rs:3602` and its text asserted at exactly one
place, `run/tests.rs:8711`; `Loud::guard_when_false` at `run.rs:3562` and `run/tests.rs:8703`. Both
sole-instrument claims are true. The two answer rows the rewrite keeps are the oracle's own, checked
rather than taken:

```
guard on outside a method   oracle rc=157  both engines rc=157, all three descriptors identical
guard on when v (v = 'x')   oracle rc=222  both engines rc=222, all three descriptors identical
                            Error 34.902:  Value of expression following GUARD keyword ...
```

Row 4 embeds the entry-7 program as a Rust string run only through `rexx-exec`, and its doc bullet
carries the do-not-run warning. That is the right shape.

## Priority 3: the distribution disagreement

**The report's resolution is right, and its stated reason is wrong. The two measurements used
different programs, which the report half-says and then contradicts.**

I ran a two-object shape of my own, 30 runs, bounded, stdout only:

```
13  A-replied|A-after|B-after|B-replied|main-end
 9  A-replied|B-after|A-after|B-replied|main-end
 6  A-replied|B-replied|main-end|B-after|A-after
 1  A-replied|B-replied|main-end|A-after|B-after
 1  A-replied|B-after|B-replied|main-end|A-after
```

Five orders, and rows 1, 2 and 5 are three of the five the previous review reported. So the review's
shape reproduces at five from a third sitting -- the count of orders is *stable per program*.

The report's own shape gives two orders in which `A-after` always precedes `B-replied`. In my shape
`A-after` follows `B-replied` in 8 of 30 runs; drawing 0 of 30 at that rate is about 1e-4. The
programs are different, not the sittings.

So the answer to "evasion or resolution" is: **resolution**, and both earlier readings were correct.
What is left is a false clause in a shipped comment -- defect **C** -- and the fact that neither
program is recorded next to its numbers, which is why this took a third sitting to settle.

## Priority 4: the set-size sweep, fourth time

**This is the first round on this plan where the shape was not relocated.** Every number in every
comment line the diff adds is a measurement (`18 and 12`, `30 runs`, `rc 137`, `0.00 s`, `5.00 s`),
an error number, a source line, or a phase number. Four genuine set-size phrases were removed and
none was reintroduced: `Three readers`, `the pair of probes`, `The last two rows`, and `the four
legality refusals, one fatal each`.

What replaced them is closed enumeration by name, which is the prescribed remedy and which is
better -- but three of them have no instrument at all and one of them claims to have one:

* `RootSet::park`: "may delete the whole parked group -- this, `release`, `live_parked` and
  `frame_aliases`, with `parked` and `parked_free`". Adding a fifth member leaves this silently
  incomplete. Partly self-enforcing: naming the *fields* means deleting them is a compile error
  everywhere they are touched.
* the guard test's doc: "the rows carrying an oracle answer are ... and the 99.911 and 34.902
  raises". Enumerated by error number, so nothing notices a sixth row. This one is also **wrong
  today** -- defect **D**.
* `Loud::guard_when_false`: "whose refusal rows are this one and `reply_inside_construct`'s".
* `Activation::reply`: claims the enumeration is self-checking and it is not -- defect **B**.

Park the first three. Ranking them must-change would be trading a stale count for a stale list,
which is what the last three rounds did.

## New defects

### B. `Activation::reply` claims a self-check it does not have (prose, must-change)

```
/// **Named rather than counted.** The readers are a set the tree can
/// enumerate, so a number here is a fact that goes stale with nothing to
/// notice; a name that stops existing is a compile error at its own site.
```

The four reader names in the paragraph above it -- `Interp::exec_reply`,
`Interp::returned_value`, `Interp::enter_method_body`, `Interp::resume_reply` -- are plain code
spans. Renaming any of them is not a compile error and is not any error; the doc goes stale exactly
as a count does. The same doc block links `[`ReplyState`]` and `[`ReplyState::Owed`]` properly, so
the difference is visible three lines apart.

This is the justification for finding 6's fix, and it is the shape it was meant to remove.

Two ways out, and the choice matters. Make them intra-doc links: `cargo doc` *does* warn on
unresolved links (four such warnings exist today, all pre-existing, none from this diff), so there
would be a real instrument -- but `cargo doc` is not in the five gates, so the warning would land
among nineteen others and nothing would fail. Or drop the second half of the sentence and keep the
first. Either is honest; the current text is not.

### C. The order-count difference is attributed to sittings, not to programs (prose, must-change)

`Interp::deferred`'s doc:

> A second measurement, of a differently written two-object shape, gave five orders over its own 30
> runs -- so the count of orders is not stable between sittings and no figure here is *the*
> distribution.

and the report, more plainly: "the count of orders is not stable across sittings either." The
measurement above refutes it: the five-order shape gives five orders again, from a third sitting,
with three of the same rows. The doc's own preceding clause names the real cause ("differently
written") and the next clause overrides it.

The conclusion survives -- the oracle has no single answer, no figure is *the* distribution, and the
program cannot be a corpus row. Only the reason has to change: the distribution depends on how the
shape is written, so quoting one is quoting a property of that program.

Fold in the traceability half: both programs are four to eleven lines and neither is recorded beside
its numbers. Whatever the comment ends up saying, the report should carry the program its 18/12 came
from.

### D. The guard test's doc gives 34.902 a corpus backstop it does not have (test-instrument, must-change)

> The rows carrying an oracle answer are `corpus/lang/method_guard_instruction.rex`, ... and the
> 99.911 and 34.902 raises; each was measured on `build/bin/rexx`. **The gated corpus is the
> stronger comparison for those** and needs the C++ oracle, where this runs without it.

No corpus program produces 34.902. `corpus/lang/` holds two guard programs --
`method_guard_instruction.rex`, whose `WHEN` is true, and `method_guard_outside_method.rex`, which is
99.911 -- and `corpus/errors/parse-errors.tsv` carries translation-time 99.913, not the run-time
value check. So `raised_guard_not_logical` has the same status as the two Louds: this test row is its
only instrument. The doc's bullet list says that of the refusal rows specifically, which now reads as
an assurance that the answer rows do not need it.

This is a rewrite of the exact doc finding 7 was about, and it introduced a new false claim about
what a check can see. Cheapest fix: add a `lang/method_guard_when_not_logical.rex` corpus program and
the claim becomes true. Otherwise the sentence has to name 34.902 as a third sole-instrument row.

### E. `exec_guard`'s doc is an unmarked second copy of entry 7 (prose, must-change)

`Loud::guard_when_false` used to point at `exec_guard` ("`Interp::exec_guard`'s own doc has the pair
of probes"). Round 1 removed that pointer and rewrote the `Loud` to name the crashes file and the
do-not-run rule. `exec_guard`'s doc was not touched, and it still reads:

> measured, `guard on when v = 1` with `v` at `0` was killed at a **6-second** timeout with no
> output, and `guard off when v = 1` likewise.

So the tree now holds two records of the same never-run program: the authoritative one, which states
its deadline and says the program must not be run, and this one, which quotes the program inline,
states a different deadline, and says neither. The header of the crashes file asserts "Every such
entry states the deadline it was killed at" -- this is not an entry, but it is where a reader of
`GUARD` arrives first.

Fix is one clause: point `exec_guard` at entry 7 and drop or reconcile the 6-second figure.

### F. The marker count does not come out of the command it is derived from (prose, must-change)

The report presents the marking as derived rather than remembered, quotes the command, and gives
`crates/rexx-exec/src/activation.rs:3`. The command gives 4:

```
$ /bin/grep -c 'SCHEDULING\|LEGALITY' crates/rexx-exec/src/activation.rs
4
```

`Activation::reply`'s marker wraps across two lines (`activation.rs:568`-`:569`), and `grep -c`
counts lines. The three *sites* are right, and the marking is complete -- I resolved all fifteen
markers to their items above. But the quoted number is not what the quoted command prints, and a
corrected count is still a count.

The deeper half is that `grep -c` cannot see this claim's subject at all. Had a site been left
unmarked, the per-file line count could still have matched, because one marker wrapping to two lines
pays for one marker missing. What the report wants is the site list it already writes down; the
number adds nothing the list does not, and it is the part that went wrong.

### G. "Recomputed from `min`" is min/min on two rows and median/min on the other two (prose, park)

The contribution table tags all four `dispatchclass` cells `min-derived`, and the block below says
"Recomputed from `min`, which is the uncontaminated statistic on both arms". The tw rows are
min/min. The ir rows are `changed` **median** over `base` min:

```
                    report      min/min     changed stat used
dispatchclass ir small  1.004191    1.004185    median 12944164395 (min is 12944091129)
dispatchclass ir large  1.004202    1.004197    median 25838965683 (min is 25838847950)
dispatchclass tw small  1.002757    1.002757    min
dispatchclass tw large  1.002765    1.002765    min
```

Substantively harmless -- the `changed` arm's median is uncontaminated, so it is a defensible
numerator, the difference is 0.0006 percentage points, and the reconciliation closes either way. It
is the label that is wrong, in the one block of the report whose whole subject is which statistic is
being read.

### H. "19 unresolved-link warnings" is 19 warnings, 4 of them unresolved links (prose, park)

Counted: `cargo doc --workspace --no-deps` emits 4 "unresolved link" and 15 "links to private item".
None of the 4 is from this diff -- all four are in `rexx-bench` and `rexx-classes` -- so the round's
new intra-doc links all resolve, which is the fact the claim was reaching for.

### I. The fix table misdescribes its own sweep (prose, park)

"five more set-size phrases reworded, including the test's own name, which carried a count". The old
name, `the_guard_instructions_answers_are_the_oracles_own`, carried a false universal, not a count --
that is what finding 7 said about it. And "five more" is itself a count in a report about removing
counts; I can identify four genuine set-size phrases removed by the diff.

### J. The `raised_guard_not_logical` insertion orphaned the catalogue sentence (prose, park)

The new LEGALITY paragraph was inserted between the summary line and the rest, so
"`Error_Logical_value_guard`, catalogue text ..." now continues the LEGALITY paragraph rather than
the summary. Its sibling `raised_if_not_logical`, six lines above, keeps the catalogue text in the
summary block, so the insertion also broke the local convention. Cosmetic, and the same shape as the
orphaned-doc-block hazard.

## Checks that reproduce, with nothing to change

* **Binding constraints.** Zero non-ASCII bytes and zero occurrences of `unsafe` in the added lines
  of the diff. `instructions:u` throughout, with `instrument` and `size` kept in every projection;
  `cycles:u` is not read as a result anywhere in the round.
* **Both engines** on every probe I ran: the two 99.913 shapes, `v = v`, 99.911 and 34.902 all agree
  with the oracle on exit status, stdout and stderr separately, never `2>&1`, except the two 99.913
  shapes, where the crate reports at `NOT_IMPLEMENTED_EXIT` with its own text -- which is
  `execute`'s documented parse-error convention and predates this task, not a Task 16 gap.
* **coverage.rs's rewritten comment.** `method_reply_twice.rex` is rc 0 with a traceback, so "not
  the same as each being fatal" is right, and the count is gone.
* **The parked group's single callers**, listed above, so "Phase 6 may delete it" is not a guess.
* **The C++ citations added by the round** all land: `GuardInstruction.cpp:167`-`:185`, `:168`,
  `InstructionParser.cpp:2646`, `LanguageParser.cpp:2024`.

## Tree state

I mutated `crates/rexx-exec/src/run.rs` twice for finding 3 -- the 98.936/98.937 raise deleted, then
the 98.935 raise deleted -- and restored it from a byte-identical copy each time. Final state:
`md5sum` back to `fc63ed248fa08d5fc495b8b748d509c6`, `git status --porcelain` empty, and the release
binary rebuilt from the restored source, so no instrumented artifact outlives the revert. Re-run on
the restored tree: `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --lib --test corpus
--no-fail-fast` is `724 passed; 0 failed`, `17 passed; 0 failed`, **185 of 185 matching**.

All oracle probes ran from a fresh empty directory with absolute paths, bounded in time and memory
on both sides, three descriptors read separately. I did **not** run entry 7's program or any
neighbour of it that blocks.
