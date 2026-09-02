# Task 1 fix report

**Base:** `c245dc418`. Brief: `.superpowers/sdd/2026-08-27-phase-5b/task-1-fix-brief.md`.
Review: `task-1-review.md`. Prior report: `task-1-report.md`.

Items assigned: C1, I1, I3, I4, M1, M3, the missing self-review, and the performance sitting.
Ruled not mine: C2 (Task 5 runs next), I2 (Task 2's), M2 (record as a known divergence and add to the
plan's Task 9 list).

**Probe discipline for everything below:** a fresh empty directory under this session's scratchpad,
absolute paths, `timeout -s KILL 20` on crate runs, three descriptors read separately and never
`2>&1`, on `REXX_ENGINE=ir` **and** `REXX_ENGINE=tree-walker`. Oracle wrapper
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`. Nothing in `rust/corpus/oracle-crashes.txt` was
run.

_(This file is written before the work and appended to as it goes.)_

## Status

| item | state | measured by |
|---|---|---|
| C1 | done | implementer |
| the array arm, beyond the brief | done, ratified by the controller | implementer; re-measured by the controller |
| I1 | done | implementer |
| I3 | done | implementer |
| I4 | done, second option taken | implementer |
| M1 | done | implementer |
| M3 | done | implementer |
| self-review | done | implementer |
| five gates + phase gate | done | **controller** |
| performance sitting | its own follow-up commit | **controller** |
| commit | done | **controller** |

**The implementer did not finish this round.** It hit an account limit at 23:55 on 2026-08-30,
having reported "Pass 2: fmt 0, cold clippy 0, release workspace 0, gated release 0, phase gate 101.
Waiting on the debug gate", with the work uncommitted and this report complete except for this block
and the gates section. The controller backed the working tree up, re-ran every gate rather than
relaying the implementer's figures, took the sitting, and committed. Everything above the gates
section is the implementer's own text and its own measurements.

## The gates, re-run by the controller at the tree as committed

Each from `rust/`, each status read unpiped, no figure relayed from the implementer's pass 2.

| command | status | figures |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | |
| `cargo test --release --workspace` | 0 | 0 `FAILED`, 102 `test result: ok` |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 0 | 0 `FAILED`, 102 `test result: ok`, `271 of 271 matching` |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 | 0 `FAILED`, 102 `test result: ok`, `271 of 271 matching` under `mode: STRICT (the gate)` |
| `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | 101 | `5b: 6 rows, 4 not yet agree` (C), `5b: 2 rows, 0 not yet agree` (D) |

The corpus figure moves 270 -> 271, which is `lang/instance_named_operands.rex`. The phase gate's 101
is unchanged from before the round: the four red rows are `objcla`, `usesem`, `obdes` and
`methodsbyclass`, owned by Tasks 2, 3, 5 and 8, and `abscla` and `creo` still read `agree`. **This
round moves no gate row**, which is what it should do -- its whole subject is a surface no row covers.

---

## C1: what the surface actually is, measured

Probes were run at BASE against the release binary, then again after the change. Every row below is
oracle against `REXX_ENGINE=ir` **and** `REXX_ENGINE=tree-walker`, three descriptors, and the two
engines never disagreed with each other on any probe in this report.

The receiver is `o = .K~new` with `::CLASS K`, named by `o~objectName=`.

### The silent arms, which are the defect

| probe | oracle | crate at BASE | after |
|---|---|---|---|
| `o~objectName='123'; say (o = 123)` | rc 0, `0` | rc 0, `1` | rc 120 loud |
| `o~objectName='123'; say (o \= 123)` | rc 0, `1` | rc 0, `0` | rc 120 loud |
| `o~objectName='123'; say (o <> 123)` | rc 0, `1` | rc 0, `0` | rc 120 loud |
| `a = (1,); say (a = 1)` | rc 0, `0` | rc 0, `1` | rc 120 loud |

All four are rc 0 with empty stderr on both sides at BASE: this phase's worst defect class. The last
row is an **array**, not an instance, and is the finding the brief did not name -- see below.

### The loud arms

`o + 1`, `o - 1`, `o * 2`, `o / 2`, `o // 2`, `o % 2`, `o ** 2`, `o > 100`, `o < 200`, `o & 1`,
`o | 0`, `o && 1`, `\o`, `-o`, `+o`: oracle rc 159 `97.1 Object "123" does not understand message
"<op>".`, crate rc 0 with an answer at BASE, crate rc 120 with the operator-operand refusal after.
`a = (1,); say (a & 1)` and `say \a` are the same shape for an array.

### What agrees, at BASE and after, and must keep agreeing

`1 + o`, `2 * o`, `123 = o`, `'123' = o`, `1 & o`, `0 | o` (an instance as the **right** operand),
`'q' || o`, `'q' o`, `'q'o`, `if o`, `select when o`, `do while o`, `datatype(o)` (`CHAR` unnamed,
`NUM` named), `datatype(o,'N')`, `length`, `substr`, `pos`, `abs`, `trunc`, `sign`, `max`, `min`,
`format`, `copies('z',o)`, `left('abcdef',o)`, a compound tail `a.o`, `parse value o with v`,
`w = o; say w`, and `say o`. **Not one of them moved**, measured before and after on both engines.

`datatype(o)` answering `NUM` is what the deleted arm's own comment claimed to be there for. It is
not: the required-string protocol converts the instance before `DATATYPE` sees it, so the answer is
unchanged with the arm gone. Re-measured rather than taken from the review.

### The finding the brief did not name: the same defect for an array

`heap_to_number`'s `Body::Array` arm parses the array's joined string value, so an array whose items
render as a number is a number to arithmetic and to non-strict comparison. It is **live** and
**predates Phase 5b**:

```
a = (1,)
say (a = 1)      oracle rc 0 out=0     crate rc 0 out=1    (both engines, empty stderr both sides)
say a + 1        oracle rc 159 (97.1)  crate rc 0 out=2
say (a & 1)      oracle rc 159 (97.1)  crate rc 0 out=1
```

`(1,)` is `ExprKind::List`, which is the one route a program has to a `Body::Array` in this crate
today (`.array~of` and `~makearray` are still Phase 5 refusals, checked).

**It also falsifies `eval.rs`'s own `debug_assert`, with no instance anywhere in the program.**

```
$ REXX_ENGINE=ir <debug>/rexx-run c06_list_cmp.rex     # a = (1,) ; say (a = 1)
rc=101
thread 'rexx-interp' panicked at crates/rexx-exec/src/eval.rs:1486:9:
a left operand that parsed as a number reported an operator gap
```
(both engines; `cargo build --locked -p rexx-exec --bin rexx-run`.)

That assertion is the premise the brief asks me to reconcile and to assert in a test, and the
existing witness for the array arm -- `an_operator_sent_to_an_object_is_loud`'s
`.Object~superClasses` rows -- cannot see it, because that array is empty and its joined string
value is `''`, which parses as no number whatever the arm does.

**Ruling taken, and it is beyond the brief:** the array arm goes too. Measured, the deletion costs
nothing -- `datatype`, `abs`, `trunc`, `length`, `max`, `'z' || a` and `if a` on `a = (1,)` all still
agree byte for byte with the oracle on both engines -- and leaving it would have shipped a half-fixed
function, since the logical and prefix half of the array defect is closed by the C1 fix anyway
(`operator_operand_gap` moving ahead of the truth test is one code path for both value kinds).
Flagged to the controller rather than folded in silently.

## What changed

* **`Interp::heap_to_number` (`value.rs`)** -- `Body::Array(_) | Body::Instance { .. } => Err(NotNumeric)`.
  The named-instance arm and the array-string-value arm both go; the arm comment states the
  measurement for the design as it now stands and cites the test that holds the premise.
* **`Interp::logical_values_body` and `PrefixOp::Not` (`eval.rs`)** -- `operator_operand_gap` is asked
  **before** `logical_value`, not after it. A name of `0` or `1` is a valid truth value, so no numeric
  parse is involved and the check has to run first or it never runs.
* **`Interp::text_len_inner` (`value.rs`)** -- gains the `Body::Instance` arm its three siblings have
  (M1). `Redirect::of` is unchanged: it answers `InstanceDefault` for an unnamed instance only, and
  the new arm is the named half, in `to_text`'s own shape.
* **`Interp::compare_values`'s skip comment (`eval.rs`)** -- the prose enumeration of the gap's shapes
  was false as written (it omitted an array and an instance, and the array falsified the claim). It
  now states the premise and names the test that holds it.
* **`Interp::operator_operand_gap`'s doc and its array arm (`eval.rs`)** -- the doc said "the shapes
  are a class object and one of the interpreter's own objects", which the match has not been true of
  for some time; the array arm said the oracle answers 97.1 for every operator that asks, which is
  false for `=` (measured, `Object`'s identity test answers `0` at rc 0). Both corrected.
* **`Interp::pool_owner`'s doc (`run.rs`)** -- names both roots and the test that reddens on the
  second (I3).
* **Tests.** `a_named_instance_is_never_an_operators_left_operand` (new),
  `a_value_the_operator_gap_names_parses_as_no_number` (new), four array rows added to
  `an_operator_sent_to_an_object_is_loud`, and both instance shapes plus an array added to
  `text_len_agrees_with_to_text`'s grid.
* **`corpus/lang/instance_named_operands.rex`** (new), with `corpus/phase-5b.txt`,
  `EXPECTED_SUBSET_5B` and `sourceline_oracle/instance_named_operands.txt` in the same commit.
* **The plan** -- I3's narrowing and I4's admission at Task 1; I1, M2 and two more measured
  divergences at Task 9.

## Controls, each recorded as run

Each mutation was applied to the worktree, built, run, and restored from a scratchpad copy taken
immediately before (never `git checkout --`). `dispatch.rs` ends the round byte-identical to `HEAD`,
checked with `git status`.

**Control 1 -- the C1 mutation: `heap_to_number`'s named-instance arm restored.**

```
test eval::object_operand_tests::a_value_the_operator_gap_names_parses_as_no_number ... FAILED
test eval::object_operand_tests::a_named_instance_is_never_an_operators_left_operand ... FAILED
```
and, from `REXX_CORPUS_GATE=1 cargo test --release --locked -p rexx-exec --test corpus --no-fail-fast`,
**`271 of 271 matching`, exit 0.**

**The corpus does not witness C1, and that is the honest answer to the brief's "this surface needs a
committed witness".** A differential row can only pin a case where the two sides *agree*, and after
the fix every one of C1's shapes is a loud refusal here against the oracle's 97.1 or its `0` --
`o + 1`, `(o = 123)` and `(o & 1)` all diverge, loudly, by the licence this phase already grants the
class-object arm. So the committed witness is split, deliberately:

* the **refusals** are `a_named_instance_is_never_an_operators_left_operand`, an in-crate test over
  both engines, which control 1 and control 3 both redden;
* the **conversions** -- every position where a named instance IS the operand's value and the oracle
  answers -- are `corpus/lang/instance_named_operands.rex`, which is what a *wrong* fix reddens.

This is the global constraint's own rule that a task changing what is refused says what instrument
catches a regression, and here the honest answer for half the surface is "an in-crate test only".

**Control 2 -- the array mutation: `heap_to_number`'s array arm restored.**

```
test eval::object_operand_tests::a_value_the_operator_gap_names_parses_as_no_number ... FAILED
test eval::object_operand_tests::an_operator_sent_to_an_object_is_loud ... FAILED
test result: FAILED. 742 passed; 2 failed
```

**Control 3 -- the gap check moved back behind `logical_value` in both places.**

```
test eval::object_operand_tests::a_named_instance_is_never_an_operators_left_operand ... FAILED
test eval::object_operand_tests::an_operator_sent_to_an_object_is_loud ... FAILED
assertion `left == right` failed: (o & 1): ""
  left: 0
 right: 120
```

**Control 4 -- M1: `text_len_inner`'s new instance arm removed, which is the state at BASE.**

```
test value::tests::text_len_agrees_with_to_text ... FAILED
panicked at crates/rexx-exec/src/value.rs:626:22:
internal error: entered unreachable code: the value model only creates Text, Num, Stem, Array and
Native, got Instance { class: ObjRef(8589934596), name: Some([49, 50, 51]), pools: ... }
```

That is M1's latent `unreachable!` made live. The reviewer could not construct a program that reaches
it and neither could I; the grid entry reaches it directly, which is what turns "latent" into
"asserted".

**Control 5 -- the over-reach, and the coverage question the corpus program has to answer.**

Two mutations were tried before one bit, and the first two are worth recording because each *looks*
like a control and is not:

* **`logical_values_body` checks the gap on the `right` operand too** -- the mistake a careless copy
  of the new line makes. `271 of 271 matching`, and a direct probe answers `p 1`: the mutation is
  dead code, because `apply_binary` has already taken the right operand through the required-string
  protocol before the dispatch, which is what its own comment says and what makes an instance on the
  right always agree. A mutation the code structure makes a no-op reads exactly like one nothing
  catches.
* **`native_new` no longer sets `interp.reqstr_armed`** -- `271 of 271 matching`. So the corpus does
  not witness that latch either; see the open items below.
* **`Redirect::of` answers `InstanceDefault` for a named instance too**, so `to_text` stops reading
  the name something set:

```
270 of 271 matching
  [UNCLASSIFIED] lang/instance_named_operands.rex: stdout, stderr, exit code differ
      rust: ... exit=222
            "    40 *-* if o \nError 34 ... Error 34.1:  Value of expression following..."
```

**One program of 271 reddens and it is the new one** -- so this is coverage the suite did not have,
not merely a test that can fail. `instance_naming.rex`, `instance_naming_overrides.rex`,
`instance_naming_raises.rex` and `instance_self_reassigned.rex` all stay green under it, because
`~string` and `~objectName` reach the set name through `native_object_name` rather than through
`to_text`. The line that catches it is `if o`, and nothing else in the corpus puts an instance in a
truth test.

## I1, M2 and two more: divergences recorded, and where

The brief assigns I1 to the plan's Task 9 list and rules M2 recorded rather than built. Both are
written into the plan, and probing C1's neighbourhood found two more that belong beside them. All
four were measured on both engines, three descriptors, at BASE and again after the change; none
moved.

| shape | oracle | crate |
|---|---|---|
| **I1.** `::METHOD defaultName` returns `overridden`; `o~zzz` | rc 159, `Error 97.1:  Object "overridden" does not understand message "ZZZ".` | rc 159, `Object "a DN"` |
| **M2.** `::METHOD objectName` returns `self`; `say o~objectName` | rc 251, `Error 5 ... System resources exhausted.` | rc 0, `a K` |
| **new.** `::METHOD string` returns `'1'`; `if o then say 'yes'; else say 'no'` | rc 0, `yes` | rc 222, `Error 34.1: ... found "a K".` |
| **new.** `::METHOD defaultName` returns `'123'`; `say o + 1` | rc 159, `Object "123" does not understand message "+".` | rc 120, the operator-operand refusal naming no rendering |

The first three are the same line the plan's `TRACE` item is on: `Interp::to_text` derives the
article-and-id rendering where the oracle sends, and every rendering reached through an infallible
function is on the wrong side of it. **The `IF` one is new and is the widest of them**, because a
truth test is ordinary code where a `TRACE` transcript is not; it is loud in this direction and
cannot become silent, since an article and a class id is never `0` or `1`. The fourth is the
loud-refusal message's own substitution and is a consequence of this crate refusing the operator at
all, so it closes when the operator-as-message rule lands rather than with the rendering line.

`o~objectName = '123'` **overrides a `defaultName` override**, checked both ways round, so the first
row's divergence disappears the moment anything names the object.

## I4: the option taken, and what I did not do

The brief allows either a case that reddens when `message_term`'s `push_temp(receiver)` goes, or a
sentence in the plan saying the ordinary-send root is argued rather than witnessed. **I took the
second**, and the line stays.

What I did towards the first: re-ran the reviewer's control myself rather than relaying it. At BASE,
in a `git archive c245dc418 | tar -x` scratch copy that writes nothing to this worktree, with
`ootest/` and `oodocs/` symlinked in and its own `CARGO_TARGET_DIR`, with
`message_term`'s `self.roots.push_temp(receiver);` deleted:

```
$ CARGO_TARGET_DIR=<scratch>/target cargo test --release --locked -p rexx-exec \
      --test collect_stress --no-fail-fast
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.76s
exit 0
```

So the figure reproduces and the claim is mine. I also read the site and established that
`run_program_collect_every_alloc` is a library function with no binary entry point, so a candidate
witness has to be a `collect_stress.rs` case rather than a probe. **I did not write and sweep
candidate shapes, so "no witness exists" is not a claim I am making** -- only that I did not
construct one. The plan now says exactly that, and says the line stays because absence of a witness
is not evidence it does nothing.

## M3: the commit message's rendering sentence

The Task 1 commit message said the derived rendering "leaves only this crate's own renderings on the
wrong side of that line". That is wrong: `TRACE`'s value lines are on the wrong side too, and they
are the oracle's own rendering shown to the user, not this crate's. **The fix commit's message
carries the correction**, and the `IF` truth test above is a third thing on that side, found in this
round. `value.rs`'s own code comment already had it right and is unchanged; the plan never repeated
the loose sentence, checked with `/bin/grep -n "wrong side" docs/superpowers/plans/2026-08-27-phase-5b.md`
(no match) -- so there is nothing to correct there.

## Three lines in this seam that nothing witnesses

I4 asked one question of one line and the answer generalises. Measured this round, each by deleting
the line and running the harshest instrument available:

| line | instrument | result |
|---|---|---|
| `message_term`'s `push_temp(receiver)` (`dispatch.rs`) | `collect_stress`, all cases | **8 passed / 0 failed**, re-measured by me at BASE in a `git archive c245dc418 \| tar -x` scratch copy with its own `CARGO_TARGET_DIR`, so the reviewer's figure reproduces |
| `native_new`'s `interp.reqstr_armed = true;` (`dispatch.rs`) | `REXX_CORPUS_GATE=1 cargo test --release --locked -p rexx-exec --no-fail-fast` | **exit 0, every binary `ok`, `271 of 271 matching`** |
| `native_new`'s own `push_temp` (`dispatch.rs`) | `collect_stress` | 6 passed / 2 failed -- this one **is** witnessed |

**The middle row is new and I measured it in full**, not just against the corpus: the whole gated
release suite for `rexx-exec` -- lib tests and every integration binary, `collect_stress`,
`ir_dual`, both gate tables, the corpus differential -- passes with that assignment deleted. So the
line that the prior report describes as what "costs the protocol walk to a program that builds an
instance and cannot answer wrongly" has no witness at all.

**None of the three is deleted**, and the rule is I4's: absence of a witness is not evidence a line
does nothing. What the measurement changes is what may be *claimed* for them, which is why
`pool_owner`'s doc and the plan's Task 1 paragraph now say which root is witnessed and which is
argued. `reqstr_armed` is recorded here rather than in a comment, and it is the kind of thing a
later task that touches the protocol's arming should know before trusting the suite to catch it.

## Self-review

Reviewing my own diff against the constraints, before the gates.

**What I would flag if this were someone else's.**

1. **The array deletion is beyond the brief and I took it anyway.** The brief scoped C1 to the
   instance and explicitly ruled three other findings out to other tasks, so the precedent is that an
   out-of-scope defect gets recorded, not fixed. My reasons are in the C1 section and I think they
   hold -- the same function, the same defect class, live silent wrong answers, and it is what makes
   the `debug_assert` the brief asks me to reconcile actually true -- but this is the one judgement
   call in the round that the controller should ratify rather than discover. It is separable: the
   array half is one arm of one match and can be reverted without touching anything else.
2. **The corpus witness does not witness C1**, and the brief asked for one that does. I could not
   make one exist and I said why rather than shipping a program whose green tells you nothing. The
   substitute is an in-crate test on both engines plus a corpus program that catches the wrong fix,
   and control 1 and control 5c are the evidence that each does its half. If the controller wants a
   differential row for C1 specifically, the only thing that would produce one is implementing
   operators as message sends, which is not this task's.
3. **`a_named_instance_is_never_an_operators_left_operand` asserts a message this crate owns**, so it
   pins this crate's own wording rather than the oracle's behaviour. That is deliberate -- the
   refusal is the thing being protected and the oracle cannot supply an expectation for it -- but it
   means the test goes red on a wording change that is not a defect. The two adjacent successes are
   what stop it being satisfied by refusing an instance everywhere; control 3 shows it can fail for
   the reason it exists, and control 1 shows it fails on the arithmetic and comparison half too.
4. **`a_value_the_operator_gap_names_parses_as_no_number` asserts over three values, not over the
   gap's whole set.** A new `operator_operand_gap` arm whose value kind can render as a number would
   not be caught. Asserting over the set is not available -- the gap takes an `ObjRef` and there is
   no enumeration of the value kinds that reach it -- so the test names the kinds it covers and the
   first assertion in its loop (that each one renders as a number) is what stops it passing
   vacuously over a value whose rendering could never parse anyway.
5. **I re-measured every "what is already there" claim I relied on** rather than taking it from the
   review, per the phase's first added rule. Two of the review's claims did not survive unchanged:
   its C1 fix sketch says deleting the `heap_to_number` arm "closes the arithmetic and comparison
   surfaces", which is true, and that the logical and prefix surfaces need the gap before
   `logical_value` "at least for an instance receiver" -- I did not scope it to an instance, because
   the same reordering is what the array needs and a kind-specific check would be a second rule
   where one does. And its statement that `>` and `=` are both 97.1 is not what the oracle does:
   `=`, `\=` and `==` are `Object`'s identity test at rc 0 and only `>`, `<` and the arithmetic and
   logical operators are 97.1. That distinction is in the arm comments now.
6. **What a green gate here could not see.** The corpus gate cannot see a loud refusal that should
   be an answer, and this change creates several: `(o = 123)`, `(o \= 123)` and `(a = 1)` are all
   loud here against the oracle's rc 0. Those are licensed by the same rule the class-object arm has
   run under since 5a, and the instrument for them is the loud message and this paragraph. If the
   operator-as-message rule ever lands, every one of them is a row to revisit.
7. **A comment I did not write.** The `IF`/`WHEN`/`WHILE` divergence is a fact about the oracle that
   no reader can derive from the code, so it was tempting to put it at the truth-test site. It is in
   the plan's Task 9 list instead, because it is a statement about where the implemented boundary
   sits, which is the shape this project has had rot four times.

## The performance sitting, taken by the controller

Task 1's report recorded the sitting as unmet because the machine offered one usable counter and
`perf stat` scaled the `cycles:u,instructions:u` pair to about 50.00% each. **The absence was
transient.** The machine rebooted, and immediately before this sitting the pair scheduled whole on a
real workload:

```
$ perf stat -x, -e cycles:u,instructions:u -- .../rexx-run .../bench-programs/emptyloop.rex
1527085587,,cycles:u,520315833,100.00,,
9276866256,,instructions:u,520315833,100.00,,
```

That also prices what the shortage would have cost: 9.2769e9 instructions unmultiplexed against
Task 1's single-event 9.2766e9, where its multiplexed pair read 9.3622e9. The scaled estimate was
about 0.9% high on a quantity otherwise deterministic to eight significant figures, which is the size
of movement a sitting exists to detect.

Command, the 5a plan's guard block (checked against the constraints file's copy, which that file
warns can narrow silently; here they agree), pin sha256 verified as
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b` against `PINNED.md` first:

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --axis dispatchclass --axis bench-rexxcps/rexxcps.rex \
    --rounds 5 --task 1-fixround-1 --commit f06142745 --baseline bench-baselines/phase-5a-arms.tsv
```

380 rows appended.

**What the delta spans, which is not this round.** No 5b task has a sitting: the last rows before
these are `23-fixround-3` at `5cc3267cd`, which is 5a's. So every figure below covers Task 0, Task 1
and this fix round together, and **nothing here is attributable to this round alone.**

**Which instrument may be read, settled from this file's own do-nothing control rather than
asserted.** `21-fixround-4-comments` and `21-fixround-5-comments` are two comment-only sittings
against the same pin -- rounds that changed no codegen at all. Across the two, per axis, arm, size:

```
instructions:u   +0.00 on every one of the fourteen
cycles:u         -2.82 -2.76 -2.16 -1.24 -0.73 -0.39 -0.09 -0.01 +0.09 +0.14 +0.31 +0.73 +0.76 +0.88
```

So **instructions are exact across sittings and cycles carry about three per cent of cross-sitting
noise with no code change at all.** The cycles column below moves -8.25% to +5.68%, which is wider
than that band and still not readable: the machine rebooted between the two sittings and its PMU
behaviour demonstrably changed, so there is no control for this particular boundary. **Cycles are not
read here.** Instructions are.

**Instructions, `pinned>head`, this sitting against `23-fixround-3`:**

| axis / arm | delta |
|---|---|
| emptyloop ir large / small | **-1.30% / -1.25%** |
| varlookup ir large / small | -0.56% / -0.54% |
| arith, compound, strings, rexxcps, alloc4c, all arms | within +-0.25% |
| dispatchclass tw small / ir small / ir large / tw large | **+0.45% / +0.38% / +0.37% / +0.22%** |

The guard is met: no axis regresses by as much as half a per cent in the instrument that can be read,
across three commits' worth of work. The one direction worth naming for a later task is
`dispatchclass`, the send path's own axis, which is where Task 1 put `NATIVE_CLASS_METHODS` and the
instance receiver kind; +0.45% over three commits is not a finding, but it is the axis to watch as
5b keeps landing in that path.

`dispatch.rex` went live as a `Role::Loop` axis with Task 1 and still owes a first baseline row
rather than a comparison. It is **not** in the guard's eight axes and adding one is a plan change, so
it stays with the plan's Task 10, which already owns it.
