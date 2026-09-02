# Task 4 fix round -- report

BASE: c9c730ba7 on `plan/rust-rewrite`. Tree clean at start.

Status: IN PROGRESS. This file is written first and appended as work proceeds.

## Plan of record

Order is the brief's, by severity: CRIT-2, CRIT-3, CRIT-1, IMP-1, IMP-2, OBS-1, Minors.

(Sections below are appended as each item is finished.)

## Reproduction, before any edit, at BASE `c9c730ba7`

Every probe below ran from a fresh empty directory under
`( cd "$D"; ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -s KILL 20 .../build/bin/rexx "$D/T.REX" )`
for the oracle and `REXX_ENGINE=<engine> timeout -s KILL 20 rexx-run "$D/T.REX"` for the crate, three
descriptors redirected to separate files. `ir` and `tree-walker` were byte-identical to each other on
every row in this report unless a row says otherwise.

All three Criticals reproduce exactly as the review states them.

| finding | oracle | ir, tree-walker |
|---|---|---|
| CRIT-2 loud (`signal on syntax` + non-continuing `forward to (d) message('FAIL')` over `1/0`) | rc 214, stdout empty, three clause lines then `Error 42.3` | rc 0, stdout `a trapped-noncont` / `b`, stderr empty |
| CRIT-2 silent (a trap in the caller as well) | rc 0, stderr empty, stdout `a OUTER-handler` / `b` | rc 0, stderr empty, stdout `a INNER-handler` / `b` |
| CRIT-3 (`s.0=3` .. `s.3='r'`, `arguments (s.)`, callee returns `'seen' arg()`) | rc 0, `a seen 4` | rc 0, `a seen 1` |
| CRIT-1 (`forward class (.Other) message('M')`, `.Other` not an ancestor) | rc 163, `Error 93.957: Target object "a K" is not a subclass of the message override scope (The OTHER class).` | rc 159, `Error 97.1: Object "a K" does not understand message "M".` |

### The C++ the review cites, read here rather than taken

* `RexxActivation::forward` (`execution/RexxActivation.cpp:1336`-`:1391`). Non-continuing branch:
  the 98.937 check (`settings.isReplyIssued() && result != OREF_NULL`), then
  `settings.setForwarded(true)`, `stopExecution(RETURNED)`, `resetDebug()`, and only then
  `messageSend`. The continuing branch calls `messageSend` with no flag set. Confirms the review.
* `RexxActivation::trap` (`:2446`-`:2465`) and **`RexxActivation::willTrap` (`:2577`-`:2597`)**, which
  the review does not name. Both open with the same `settings.isForwarded()` block and both **drill**
  to the previous non-forwarded frame and return *that frame's* answer. `willTrap` matters because it
  is the cheap pre-check `Activity::checkCondition` runs, which is the C++ analogue of this crate's
  `Interp::trap_for` gate in `dispatch.rs`.
* `RexxObject::messageSend`'s scope-override overload (`classes/ObjectClass.cpp:919`-`:923`) opens
  with `validateScopeOverride(startscope)` and carries the comment the review quotes.
  `validateScopeOverride` is at `:1948`-`:1959` and raises
  `Error_Incorrect_method_array_noclass` with `this` and `scope` substituted.
* `RexxInstructionForward::execute` (`instructions/ForwardInstruction.cpp:128`-`:210`) evaluates
  `TO`, `MESSAGE`, `CLASS` (with the `isInstanceOf(TheClassClass)` 88.914 **between** the evaluate
  and the `traceKeywordResult`, which is MIN-1) and then `ARGUMENTS` through `requestArray`, refusing
  `TheNilObject` or a multi-dimensional array with 98.946.

### Three facts the review does not have, all measured here

**CRIT-1 is wider than "non-continuing".** `forward continue class (.Other) message('M')` with no
trap is oracle rc 163 / 93.957 against crate rc 159 / 97.1 -- the same split. The C++ says why: the
check is inside `messageSend`, which both branches of `RexxActivation::forward` call.

**CRIT-1's scope is validated against the `TO` target, not against `SELF`.**
`forward to (t) class (.Other) message('M')` where `t` is `a TGT` is oracle
`Target object "a TGT" is not a subclass of the message override scope (The OTHER class).`, so the
fix substitutes the resolved target.

**CRIT-1 and CRIT-2 meet, and the meeting is the control that separates them.** Under a
`signal on syntax` in the forwarding method:

| arm | oracle | why |
|---|---|---|
| `forward class (.Other) message('M')`, non-continuing | rc 163, **not trapped** | 93.957 is raised inside `messageSend`, after `setForwarded` |
| `forward class (5) message('M')` | rc 0, `b trapped-notaclass` | 88.914 is raised by `ForwardInstruction`, before `forward()` is called at all |
| `forward continue class (.Other) message('M')` | rc 0, `c trapped-cont-scope` | 93.957 again, but a continuing forward sets no flag |

The second and third **already agree** at BASE, and the third agrees **for the wrong reason**: this
crate raises 97.1 there, not 93.957, and the trap hides which one it was. The untrapped continuing
probe above is the control that separates them.

### A third face of CRIT-2 the review does not have: `NOMETHOD`

`signal on nomethod` in the **caller** of a method whose non-continuing `FORWARD` names a message
nothing answers:

```
oracle           rc 0, stderr empty, stdout "b outer-nomethod" / "c"
ir, tree-walker  rc 159, 97.1 Object "a K" does not understand message "NOSUCHNAME2".
```

The adjacent success that says this is `FORWARD`'s and not `NOMETHOD`'s: the same caller-side trap
over an ordinary failed send (`return self~nosuchname`, no `FORWARD` anywhere) is **rc 159 with the
identical traceback on all three sides** -- the oracle does not trap that either, because
`Activity::raiseCondition` stops at the first Rexx activation and the sending frame has no
`NOMETHOD` trap. So the crate's "ask the running activation" gate is right in general, and wrong
only where that activation is a phantom, which is exactly the C++'s `willTrap` drill.

### A fourth divergence on this path, found here, on no list: a multi-line string as `ARGUMENTS`

`v = 'p' || '0a'x || 'q'` then `forward message('SEEN') arguments (v)`:

```
oracle           rc 0, stderr empty, "a seen 2 [p] [q]"
ir, tree-walker  rc 0, stderr empty, "a seen 1 [p<LF>q]"
```

`requestArray` on a string is `StringUtil::makearray` (`classes/support/StringUtil.cpp:545`-`:638`),
which splits on `\n`, drops a `\r` immediately before one, and appends a tail piece only when the
string does not end at a separator. Silent, rc 0 both sides. This is the same predicate defect as
CRIT-3 and is fixed with it.

## OBS-1: a member of the licensed shape DOES hang, and the licence is wrong as written

**Answered on the crate alone. The oracle binary was never invoked on any program in this section**,
and none of them is `corpus/oracle-crashes.txt`'s program. The runner used here
(`crate_only.sh`) contains no path to the oracle at all.

Four members of the family, `timeout -s KILL 60`, both engines, three descriptors separate:

| member | ir | tree-walker |
|---|---|---|
| `::METHOD a` forwards `message('B')`, `::METHOD b` forwards `message('A')` -- no `REPLY` | **rc 245**, 0.076 s, stdout 0 B, stderr 320,241 B ending `Error 11.1:  Insufficient control stack space; cannot continue execution.` | rc 245, 0.068 s, identical |
| `::METHOD m` doing `forward to (self) message('M')` -- no `REPLY` | **rc 245**, 0.074 s, stderr 420,231 B, same last line | rc 245, 0.070 s, identical |
| `::METHOD bare` doing `reply` then `forward message('BARE')` | **rc 137**, killed at 60.004 s, stdout 0 B, stderr 0 B | rc 137, 60.004 s, identical |
| `::METHOD a` doing `reply` then `forward message('B')`, with `b` forwarding back to `a` | **rc 137**, killed at 60.003 s, both descriptors empty | rc 137, 60.004 s, identical |

**The discriminator is the `REPLY`, not the route.** Two routes that are not the forbidden literal --
a mutual pair and an explicit `to (self)` -- both answer the licensed rc 245 in under a tenth of a
second. The same two routes with a `reply` in front of the `FORWARD` do not come back.

**It is not an unbounded allocation and it is not deep recursion.** Sampled once a second for 24 s
while hung: `VmRSS` is **17,760 kB at every sample**, threads 2. And the stack, taken with
`gdb -p <pid> -batch -ex "thread apply all bt 25"` on the interpreter thread, is **24 frames** and
does not grow:

```
Interp::run_deferred_replies   (dispatch.rs:2473)
Interp::resume_reply           (dispatch.rs:2731)
Interp::run_activation         (run.rs:1487)
  ... the IR driver ...
Interp::exec_forward           (run.rs:3807)
Interp::send_message           (dispatch.rs:2791)
Interp::invoke / enter_method_body
```

So the resumed body forwards, the forward enters the method again, that invocation replies and queues
another deferred body, and `run_deferred_replies` drains a queue that refills as fast as it empties.
`MAX_ACTIVATION_DEPTH` is never approached, which is why the resource guard the licence relies on
never fires. This is a **measurement of the site**, not an inference: the frames above are what the
sampler printed.

**It is not this fix round's.** The same two programs against a `git archive c9c730ba7` extract built
with its own `CARGO_TARGET_DIR`: `f1_reply` rc 137 at 40.004 s and `f4_reply_mutual` rc 137 at
40.003 s, both descriptors empty, while `f2_mutual` is rc 245 in 0.074 s. Identical to the tip.

### What the licence should read

The current text (`corpus/oracle-crashes.txt`, and Task 9's list in the same terms) says this crate
answers the shape `rc 245`. That is true only for the members with no `REPLY` in front. It should
read, as two sentences rather than one:

> This crate sends with the forwarding activation still on the stack, so `MAX_ACTIVATION_DEPTH`
> counts the recursion: measured on the crate alone, both engines, a self-resolving non-continuing
> `FORWARD` is **rc 245**, stdout empty, stderr ending `Error 11.1`.
> **With a `REPLY` before it, this crate does not terminate.** The replied body is queued and
> `Interp::run_deferred_replies` drains a queue each drained body refills, so no activation depth
> grows and no guard fires: measured, rc 137 under `timeout -s KILL 60`, both descriptors empty,
> `VmRSS` flat at 17,760 kB. Neither answer is the oracle's, which cannot be measured for this shape
> at all.

**I did not invent a bound**, and the reason is that there is no oracle to check one against: every
member of this family is a shape the safety rule forbids handing to the oracle, so a cap on the
deferred-reply chain would be behaviour with nothing to validate it, and any cap wide enough to be
safe for ordinary `REPLY` programs is a number chosen rather than measured. That is a decision for
the controller and for Moritz, not for a fix round.

**One mitigation that is not a behaviour change and is worth the controller's attention.** The
differential harness bounds only the *oracle* side -- `ORACLE_DEADLINE` in `tests/support/oracle.rs`
-- and the crate side runs in-process with no deadline at all, which is why OBS-1's mutation arm
stalled a whole corpus run rather than reporting one red row. A crate-side deadline would turn any
future non-termination into a failing row instead of a hung gate.

## What was changed

Three commits on `plan/rust-rewrite` from BASE `c9c730ba7`:

| commit | what |
|---|---|
| `e071cae35` | the three Criticals and MIN-1, with four corpus witnesses and their subset wiring |
| `5ab863759` | the records: the licence's second half, five entries for Task 9's list, the `DELEGATE` refusal's own wording, the plan's stale sentence, and the set-size comments |
| `5ad238d81` | the `::ATTRIBUTE ... DELEGATE` gate probe gains the send that reaches its getter |

### CRIT-2: the forwarding activation is a phantom for condition delivery

`Activation::forwarded` is set in `Interp::exec_forward` after the 98.937 check and before the send,
for a `FORWARD` with no `CONTINUE` -- the position `settings.setForwarded(true)` holds in the C++.
Two readers:

* `Interp::offer_to_trap` declines while the running activation carries it, so the failure leaves
  that activation and the caller's own clause loop offers it. That is how this crate reaches the
  frame `RexxActivation::trap` drills to.
* `Interp::trap_frame` does the drill itself, for the callers that ask *whether* a condition would be
  trapped at all rather than trapping it -- `dispatch.rs`'s `NOMETHOD` and `NOSTRING` gates. That is
  `RexxActivation::willTrap`, and it is what the caller-side `NOMETHOD` face needs.

The `CONTINUE` path sets nothing, which is the C++'s own split.

### CRIT-1: the send validates the scope override, so both sites ask it

`Interp::validate_scope_override` is new in `dispatch.rs` beside `receiver_has_scope`.
`Interp::message_term` now calls it in place of the two lines it had, and `Interp::exec_forward`
calls it immediately before `send_message` on **both** branches -- after the phantom flag is set, so
a bad scope under a non-continuing `FORWARD` is not trapped by the forwarding method, which is what
the oracle does. It takes the resolved **target**, so the report names the `TO` value.

### CRIT-3 and part of IMP-2: `ARGUMENTS` through `requestArray`

`Interp::forward_arguments_conversion` replaces the `operator_operand_gap` call and answers which arm
of `RexxInternalObject::requestArray` a value takes:

| value | arm | measured |
|---|---|---|
| an array | its own slots | unchanged |
| a string, a number, a small integer | `StringUtil::makearray` over its text | `''` is no arguments, `'p'` and `'p\n'` are `[p]`, `'p\r\nq'` is `[p] [q]`, `'p\n\nq'` is `[p] [] [q]`, `'\nq'` is `[] [q]`, a lone `\r` is kept -- all eight rows byte-identical to the oracle |
| a stem | its assigned tails, and never its default | `a seen 4` where the crate had `a seen 1`; `a. = 'dflt'` with no tail is `seen 0` where the crate had `seen 1` |
| `.nil`, a class object, an instance whose behaviour has no `MAKEARRAY` | 98.946 | oracle rc 158 on all three, and the last two were rc 120 before |
| an instance whose behaviour **has** `MAKEARRAY` | still a loud refusal | oracle rc 0 `a seen 2 [x] [y]`; on Task 9's list |
| a `.StringTable`, a `.Directory`, a `.List`, a `.Queue`, a multi-dimensional array | still a loud refusal | on Task 9's list |

**The stem's item order is a divergence this fix does not close, and it is recorded rather than
smoothed.** The count agrees; the order does not -- oracle `[0] [3] [2] [1]` against `[0] [1] [2] [3]`
here for tails assigned `0`,`1`,`2`,`3`. The oracle's order is a walk of `CompoundVariableTable`'s
balanced tree, `first()` taking the deepest left leaf and `next()` coming up from below
(`classes/support/CompoundVariableTable.cpp:343`-`:425`), so it depends on the order the tails were
assigned in. `Body::Stem`'s `tails` is a `rexx_core::NameMap`, which is a `HashMap`, and holds no
insertion order at all, so reproducing it is a change to the representation of the interpreter's
hottest data structure and owes a performance sitting. Deterministic on the oracle, five runs. This
crate orders by the oracle's own comparison -- length, then bytes
(`classes/support/CompoundVariableTail.hpp:170`) -- which agrees for a stem with no assigned tail and
for one with a single tail, and the corpus row reports items only in the single-tail case.

### MIN-1: the `>K>` line an invalid `CLASS` does not write

`exec_forward`'s `CLASS` arm evaluates, checks `class_id`, and only then traces, which is the C++'s
order at `instructions/ForwardInstruction.cpp:168`-`:175`. `Interp::forward_keyword` is split so that
its tracing half, `trace_forward_keyword`, can be called at a separate point.

## Controls: what reddens, and what the existing corpus does not see

Every arm is a `git archive 5ab863759` extract with its own `CARGO_TARGET_DIR`, sources `touch`ed
before each build, restored from a `cp -a` copy between arms and `touch`ed again. Each arm is read
twice from the same build:

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_d --test corpus --no-fail-fast
```

once over the committed subset, and once with this round's four rows commented out of
`corpus/phase-5b.txt` -- which is the "can fail is not adds coverage" check, and it needs no rebuild
because `read_subset` reads the file at run time.

**Baseline, unmutated: exit 0, `315 of 315 matching`, table D `5b: 2 rows, 0 not yet agree`.**
Without this round's rows: `311 of 311`, exit 0.

| arm | what it deletes | full | rows reddened | without this round's rows |
|---|---|---|---|---|
| MA | `exec_forward` never sets `Activation::forwarded` | 313 of 315, exit 101 | `forward_phantom_trap.rex`, `forward_class_scope.rex`, both `stdout, stderr, exit code differ` | **311 of 311, exit 0** |
| MB | `trap_frame` answers the running activation and never drills | 314 of 315, exit 101 | `forward_phantom_trap.rex` alone | **311 of 311, exit 0** |
| MC | `exec_forward` drops the `validate_scope_override` call | 314 of 315, exit 101 | `forward_class_scope.rex` alone | **311 of 311, exit 0** |
| MD | a stem yields only its first assigned tail | 314 of 315, exit 101 | `forward_arguments_converted.rex`, `stdout differ` | **311 of 311, exit 0** |
| ME | `makearray_lines` answers the whole text as one piece | 314 of 315, exit 101 | `forward_arguments_converted.rex`, `stdout differ` | **311 of 311, exit 0** |
| MF | `CLASS` traced before the `class_id` check | 314 of 315, exit 101 | `forward_class_trace.rex`, `stderr differ` | **311 of 311, exit 0** |
| MG | an instance refuses loudly instead of 98.946 | 314 of 315, exit 101 | `forward_arguments_converted.rex`, `stderr, exit code differ (loud failure: ... rust rc 120, oracle rc 158)` | **311 of 311, exit 0** |

So each new row reddens under the deletion of the behaviour it witnesses, and **no arm is caught by
the corpus that existed before this round** -- the right-hand column is `311 of 311` in every one.

**MA and MB separate the two halves of the CRIT-2 fix.** MA takes the flag away and two rows go; MB
leaves the flag and takes only the drill away, and one row goes -- the `NOMETHOD` arm, which is the
only thing in the corpus that asks whether a condition *would* be trapped rather than trapping it.

**MC's row agrees for a reason a weaker probe would not have found.** Before this round the
`forward continue class (.Other)` arm under a trap agreed on all three sides while this crate raised
97.1 and the oracle raised 93.957, because the trap hid which one it was. The committed row reports
`rc` from inside the handler -- `contbad 93`, `notaclass 88` -- so the arm now separates the error
from the fact that something was trapped.

### MH: the gate row that was green over a wrong implementation

A separate arm, run over the **whole** `rexx-exec` suite rather than the two harnesses, because the
question was what the suite sees rather than what the corpus sees. It makes
`attribute_dictionary_keys` register a plain `Getter` beside the delegating setter for
`::ATTRIBUTE ... DELEGATE`:

```
corpus                          315 of 315 matching
table D                         5b: 2 rows, 0 not yet `agree`
reddened                        dispatch::tests::a_delegate_directive_forwards_under_every_key_it_claims
                                tests::a_delegate_method_is_a_generated_method
```

So the delegating **getter** was sent by no corpus program: the gate probe drove the setter through
the delegate and read back through `k~peek~at`, which is Inner's own generated getter.
With the probe's line changed to `say 'main' k~at k~peek~at`, the same mutated build prints
`main AT via-set` -- **the added send alone moves** -- and the run reads `314 of 315` with
`5b: 2 rows, 1 not yet agree` at exit 101. The tree prints `main via-set via-set` on oracle, `ir` and
`tree-walker`.

That arm's whole-suite run also carried **eight environmental failures unrelated to the mutation**:
`crates/rexx-extract` panics with `cannot read .../ootest/ooRexx/base/expressions` because `ootest/`
is git-ignored and a `git archive` extract does not have it. They are the same eight with and without
the mutation, and none of the seven arms above could see them, since those ran only
`--test gate_table_d --test corpus`.

## IMP-1: the two arms the review ran, recorded rather than re-derived

The review ran the two arms `d8e48d353`'s table was missing for `acb015bd4`'s `98.937` witness and
both behave. Recorded here as the review reports them, not re-run:

| arm | what it deletes | table D 5b | corpus | rows reddened |
|---|---|---|---|---|
| M10 | `forward_after_reply` always answers `Ok` -- the 98.937 check gone | both `agree`, exit 0 | 310 of 311 | `lang/forward_after_reply.rex` alone, `stderr differ` |
| M11 | the check keyed to `ReplyState` rather than `replied_a_value` -- the keyword, not the value | both `agree`, exit 0 | 310 of 311 | the same row alone, `stderr differ` |

So the file's `bare` arm does separate "a `REPLY` happened" from "a `REPLY` carried a value", which is
the claim `acb015bd4`'s message makes, and the record was what was short rather than the witness.

## IMP-2: what closed, what is recorded, and one row the review does not have

Closed, and now agreeing byte for byte on all three descriptors on both engines:

| probe | before | now |
|---|---|---|
| `arguments (self)` | oracle rc 158 `98.946` against rc 120 | rc 158, `98.946`, identical |
| `arguments (.String)` | oracle rc 158 `98.946` against rc 120 | rc 158, `98.946`, identical |
| `arguments (p)` for an instance of a class with no `makeArray` | not in the review; was rc 120 | rc 158, `98.946`, identical |
| a multi-line string as `ARGUMENTS` | **not in the review**: oracle `a seen 2 [p] [q]` against `a seen 1 [p<LF>q]`, rc 0 and empty stderr both sides -- a silent wrong answer | identical, and five more line shapes with it |

Recorded on Task 9's list with their measurements, in the plan:

* the stem item **order**, which is the silent one, with what closing it costs;
* a `.StringTable` or `.Directory`, oracle rc 0 `a seen 1` against rc 120, which carries D61's
  ordering question as well as the conversion (`HashCollection::makeArray` is `allIndexes`);
* an instance whose class defines `makeArray`, oracle rc 0 `a seen 2 [x] [y]` against rc 120;
* a multi-dimensional array, a `.List` and a `.Queue`, each blocked behind a class surface this phase
  has not built;
* `::method m delegate a.b` and `delegate a.`, oracle rc 159 `Object "A.B" does not understand
  message "M".` against rc 120.

The last one's **refusal text is fixed** rather than its behaviour: `Interp::delegate_variable`
raised `Loud::accessor_variable`, whose message says "a generated accessor for the attribute", which
does not describe a `DELEGATE` directive. `Loud::delegate_variable` now says
`a DELEGATE to the variable "A.B" is not implemented`.

The review's route to that refusal does not reach it any more, and the report says so rather than
quoting the review's transcript: `::method m delegate a.b` with `expose a.b` in `init` is now
`rexx-exec: EXPOSE of the single compound tail "A.B" is not implemented` at rc 120, a different
refusal reached first. Assigning through `expose a.` instead, or leaving the variable unset, reaches
`delegate_variable`.

## Minors

* **MIN-1** is fixed and witnessed by `forward_class_trace.rex`; see above.
* **The plan's stale sentence** at its Task 4 section is corrected: the spec's `'abcdef'` spelling was
  the controller's to correct, was corrected at `1aa54204b`, and `/bin/grep -n abcdef` on the spec
  answers `799: d = 'abcdefgh'` and `828: d = 'abcdefgh'` and nothing else -- checked here, not taken
  from the review.
* **The `::ATTRIBUTE ... DELEGATE` getter** is now sent by its gate row; the arm above is the
  measurement that says why it had to be.
* **The four set-size comments** are gone. `run.rs`'s `// FORWARD, with any of its six options` is
  `// FORWARD and its options`; `exec_forward`'s "own three defaults" is "own defaults";
  `forward_arguments`'s "**Three sources, and the third is the default.**" is
  "**`ARGUMENTS`, `ARRAY`, or the method's own arguments.**"; and `forward_options.rex`'s comment
  drops both "four value-carrying options ... three defaults" and "all three at once". That file's
  `sourceline_oracle` expectation was regenerated with the driver its own module comment specifies
  (`count 37`, matching the file's 37 lines).

## The performance sitting this owes, not taken

Per the brief, no sitting was attempted. What one would be for:

* `Activation` gains one `bool`. `Interp::exec_forward` gains one branch and one store on the
  non-continuing path only.
* `Interp::trap_for` now goes through `Interp::trap_frame`, which adds one predicate on the running
  activation to **every** `trap_for` call -- the `NOVALUE` gate among them, which is the one on a hot
  path. The reverse iteration only runs when that predicate is true, which no ordinary program
  reaches.
* `Interp::offer_to_trap` gains one predicate on the failure path.
* `Interp::forward_arguments` no longer calls `operator_operand_gap`; `FORWARD ARGUMENTS` over a
  non-array now allocates one string per converted item. `FORWARD` with no `ARGUMENTS` is untouched.
* `Interp::message_term` is unchanged in work: the same two questions moved behind one call.

The axes to read are the dispatch ones and any axis with a controlled loop, since `trap_for`'s
`NOVALUE` gate is on the variable-read path.

**One bound rather than a substitute: `size_of::<Activation>()` is still 384.** Read out of the
compiler at the tip with `const _: [(); 0] = [(); size_of::<Activation>()];`, which reports
`expected an array with a size of 0, found one with a size of 384`; the file was restored afterwards
and `git status` is clean. That is the same figure Task 4's own report recorded before this change,
so the new `bool` fits in padding the struct already had and the activation pool's per-frame cost has
not moved. It says nothing about the `trap_for` predicate, which is what the sitting is for.

## Corrections this report owes, which the commits cannot carry

**`5ab863759`'s message says "Four comments stated the size of a set ... the sets are named without
their counts instead", and that commit carries only one of the four.** The three in
`crates/rexx-exec/src/run.rs` are in `e071cae35`, because they were edited before that commit was
staged; only `corpus/lang/forward_options.rex`'s is in `5ab863759`. The work is done and is where
this paragraph says, and no commit was amended.

**The review's `NameMap` claim is not one it made, and the correction is mine to state.** The review
says the oracle's stem items are "in hash order"; they are not. `CompoundVariableTable` is a balanced
binary tree and the order is a walk of it, deterministic across runs -- measured, five identical runs
of the same probe. That does not change the review's conclusion or the brief's instruction to compare
`arg()`; it changes why the order cannot be reproduced here, which is insertion order rather than
hashing.

## Probing past the rows this round added

Every 5b task turns a loud refusal into an answer, which is where a silent wrong answer gets
introduced, so each new answer was probed past its own witness. All from a fresh empty directory,
three descriptors separate, both engines. Byte-identical to the oracle unless the row says otherwise.

| probe | answer |
|---|---|
| a stem with a **dropped** tail -- `s.1`,`s.2`,`s.3` assigned then `drop s.2` | rc 0 `a count 2`; the tombstone is not a tail |
| a stem whose only tail is a **compound** one, `t.i.5` with `i = 4` | rc 0 `b seen 1 [4.5]` -- the resolved tail name, not the written one |
| two `FORWARD`s in progress at once, the innermost raising | rc 0 `a TOP-handler` -- the drill skips **both** phantoms and the outermost trap fires, which is the C++'s "we can have multiple forwardings in process" loop |
| `CALL ON USER` armed in the forwarding method and in its caller, `raise user` in the forwarded-to method | rc 165, `91.999 Message "M" did not return a result.`, identical -- neither `CALL` trap fires and the forwarding method answers nothing |
| a `CLASS` that is an ancestor of the **`TO` target** but not of `SELF` | rc 0 `a otherbase-m` -- **the control that separates "validated against the target" from "validated against the receiver"**: a build asking about `SELF` raises 93.957 here, and this send succeeds |
| a class-side `FORWARD` with a bad scope | rc 163, `Target object "The KC class" is not a subclass of the message override scope (The OTHER class).` |
| `FORWARD CLASS` naming a class the receiver does not carry, through the class side, untrapped | rc 159 `Object "The K class" does not understand message "CM".` where the scope is valid, so the miss is a name miss and not a scope one |

**One pre-existing divergence found while probing and not this round's**, recorded so it is not read
as new: `call on syntax` is not valid Rexx, and where the oracle reports
`Error 25.1: CALL ON must be followed by one of the keywords ERROR, FAILURE, HALT, NOTREADY, USER, or
ANY; found "SYNTAX".` at rc 231, this crate answers `rexx-exec: 25.1: Invalid subkeyword found.` at
rc 120. Measured identical against the `c9c730ba7` extract, so it is at BASE and has nothing to do
with `FORWARD`; it belongs to whoever owns parse-error reporting.

## The five gates and the phase gate, at `5ad238d81`

Run from `rust/`, sequentially, each status written to its own file by `echo $? > file` on the line
after its command -- never chained, never piped. Tree clean, `git status --short` empty.

| # | command | status |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace` | **0**, no `test result: FAILED` in the log |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0**, `315 of 315 matching`, `4246 of 4259 matching` on the trace-surface report, no `test result: FAILED` |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0**, `315 of 315 matching`, no `test result: FAILED` |

Phase gate:

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
```

**exit 101, by design.** Table C by owning phase: `5a: 135 rows, 0 not yet agree`,
`5b: 6 rows, 1 not yet agree` -- the one is `gate-tables/concepts/methodsbyclass.rex`, Task 8's, which
the harness names in its own failure line. Table D: `5a: 36 rows, 0 not yet agree`,
`5b: 2 rows, 0 not yet agree`.

Gate 2's green is **provisional in the sense `rust/CLAUDE.md` gives it**: it ran against a warm
target directory in the same session as the edits, finishing in seconds, so it is evidence that the
linter re-examined `rexx-exec` (its log says `Checking rexx-exec`) rather than proof that a cold run
would say the same.

## Status

Everything the brief assigns is done.

| item | outcome |
|---|---|
| CRIT-2 | fixed, both faces plus a third the review did not have; witnessed by `forward_phantom_trap.rex` with four adjacent successes; MA and MB separate the fix's two halves |
| CRIT-3 | the count agrees; witnessed by `forward_arguments_converted.rex` comparing `arg()`; the item **order** is a recorded divergence with the cost of closing it |
| CRIT-1 | fixed, and wider than the review had it -- the `CONTINUE` path too, and the report names the `TO` target; witnessed by `forward_class_scope.rex` |
| IMP-1 | recorded from the review's own transcripts, not re-derived |
| IMP-2 | four rows closed (two of them the review's, one it did not have, one silent and new); five recorded on Task 9's list; the `DELEGATE` refusal's wording fixed |
| OBS-1 | **the licence is wrong as written**: a member of the family does not terminate, measured on the crate alone with a stack and an RSS series, and present at BASE. No bound invented; the text it should carry is written out above and in the plan |
| MIN-1 | fixed and witnessed by `forward_class_trace.rex` |
| MIN-2 | the plan's stale sentence corrected; the gate probe now sends the delegating getter, with the arm that shows why; the four set-size comments gone |
| sitting | not attempted, per the brief; what it is for is listed, with `size_of::<Activation>()` still 384 as a bound |

**Left open, none of it silent except where the row says so:** the stem item order (silent, recorded,
needs `Body::Stem` to keep an insertion order it does not have); the four other `ARGUMENTS`
conversions and the compound `DELEGATE` variable (all loud, recorded); the self-forward-after-`REPLY`
non-termination (no bound chosen, the controller's and Moritz's call); the harness's missing
crate-side deadline; and the spec's quoted `::ATTRIBUTE` probe, which now carries one send where the
committed probe carries two, and which the plan says is the controller's to align.
