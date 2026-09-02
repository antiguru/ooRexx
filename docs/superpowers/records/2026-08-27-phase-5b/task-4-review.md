# Task 4 review: `FORWARD` and `DELEGATE` (D62)

Range `ccc6dbd13..8e891cd5a`. Report opened before reading the diff; appended as the work proceeds.
Every figure below is quoted beside the command that produced it. Work done in a
`git archive 8e891cd5a` extract with its own `CARGO_TARGET_DIR`; the live worktree was not written to
and no cargo ran in it.

## Summary

Three Critical, two Important, two Minor, one observation. Two of the three Criticals are **silent
wrong answers** -- same exit status, same or empty stderr, different stdout -- and both are on paths
this task built. Nothing in the committed rows sees any of them, and every arm the report claims
reproduces exactly, so the failure is coverage past the row rather than a control that lied.

| id | severity | one line |
|---|---|---|
| CRIT-1 | Critical | `FORWARD CLASS (x)` skips the scope-override validation the `~name:scope` path already does: 97.1 at rc 159 where the oracle has 93.957 at rc 163 |
| CRIT-2 | Critical | a condition raised by a **non-continuing** `FORWARD`'s send is trapped by the forwarding method; the oracle unwinds past it. Silent in the two-trap form: rc 0 and empty stderr on both sides, different stdout |
| CRIT-3 | Critical | `FORWARD ARGUMENTS` over a stem passes the stem as one argument where the oracle expands it: `seen 4` against `seen 1`, rc 0 and empty stderr on both sides |
| IMP-1 | Important | the 98.937 witness added in `acb015bd4` has no recorded control. I ran the two missing arms; both redden, so the record is what is short, not the witness |
| IMP-2 | Important | six loud divergences on the new `FORWARD ARGUMENTS` and `DELEGATE`-variable paths, none recorded; two of them the oracle answers with the 98.946 this task already raises |
| MIN-1 | Minor | under `trace i` an invalid `FORWARD CLASS` writes a `>K>` line the oracle does not, because the C++ raises between evaluating and tracing |
| MIN-2 | Minor | a stale "the controller's to correct" sentence in the plan; the `::ATTRIBUTE ... DELEGATE` getter is not sent by its gate row; four comments state a set size |
| OBS-1 | -- | arm M6 does not terminate on `forward_after_reply.rex`; evidence that one member of the licensed self-forward shape may hang rather than answer rc 245 |

**Why no committed row sees any of the three**, checked rather than asserted, over
`corpus/lang/forward_*.rex`, `corpus/lang/delegate_*.rex` and the two gate probes:
`/bin/grep -niE "signal on|call on"` names one line, `delegate_private.rex:22`, which is a trap around
a *delegate* send and that arm agrees; `/bin/grep -niE "forward[^;]*class *\("` over `corpus/lang`
names only `forward_class_super.rex`'s two `class (super)` clauses, both valid ancestors; and the only
`ARGUMENTS` values in the corpus are two array literals, `'abc'` and `.nil`.

**What the report got right and I re-measured rather than accepted** is listed at the end.

## CRIT-1 (Critical): `FORWARD CLASS (x)` skips the scope-override validation, and answers the wrong error

`FORWARD`'s `CLASS` value reaches `Interp::send_message` as a start scope without the check the
`~name:scope` form applies first, so a scope the receiver does not carry reports a *name miss*
instead of the oracle's *scope* refusal. Different exit status, different error number, different
text.

Probe, run from a fresh empty directory, three descriptors read separately:

```rexx
o = .K~new
say 'a' o~n
say 'b'
::class Base
::method m
  return 'base-m'
::class Other
::method m
  return 'other-m'
::class K subclass Base
::method n
  forward class (.Other) message('M')
```

```
oracle          rc 163
    12 *-* forward class (.Other) message('M')
     2 *-* say 'a' o~n
  Error 93 running T.REX line 12:  Incorrect call to method.
  Error 93.957:  Target object "a K" is not a subclass of the message override scope (The OTHER class).

ir, tree-walker rc 159
    12 *-* forward class (.Other) message('M')
     2 *-* say 'a' o~n
  Error 97 running T.REX line 12:  Object method not found.
  Error 97.1:  Object "a K" does not understand message "M".
```

`forward class (.Other)` with no `MESSAGE` (so the default name) gives the identical split.

**It is this task's, not pre-existing, and the crate already has the check.** The same bad scope
through the message-override syntax is byte-identical on all three sides:

```rexx
o = .K~new
say 'a' o~m:.Other
say 'b'
::class Base
::method m
  return 'base-m'
::class Other
::method m
  return 'other-m'
::class K subclass Base
::method m
  return 'k-m'
```

```
oracle, ir, tree-walker   rc 163, Error 93.957, "a K" / "The OTHER class"
```

`Interp::message_term` (`dispatch.rs:2950`-`:2970`) asks `scope.class_id().is_none()` and then
`self.receiver_has_scope(receiver, scope)`. `Interp::exec_forward` (`run.rs:3722`-`:3736`) asks only
the first. In the C++ both checks are inside `RexxObject::messageSend`'s scope-override overload
(`classes/ObjectClass.cpp:919`-`:924`, calling `validateScopeOverride`, defined at `:1950`), which is
the overload `RexxActivation::forward` calls on both its branches
(`execution/RexxActivation.cpp:1358` and `:1386`). The C++'s own comment on that call is **"validate
that the scope override is valid (FORWARD uses this method, this way no need to check TO option)"**,
so the oracle gets the check from the send and this crate has to ask it at each site.

**Valid ancestors are unaffected**, so no committed row sees this: `forward class (.Base)`,
`class (.Gp)` two levels up and `class (.K)` on the receiver's own class all answer
`a base-m` / `b gp-m` / `c k-m` at rc 0 on all three sides.

Suggested fix: the two lines `message_term` already runs, moved into `exec_forward` beside the
`class_id` check, plus a corpus row for the refusal. Whether the check belongs in `send_message`
itself is a wider question -- `send_to_delegate` passes `None` and cannot reach it.

## CRIT-2 (Critical): a condition raised by a non-continuing `FORWARD`'s send is trapped by the forwarding method, where the oracle unwinds past it

The oracle makes the forwarding activation a phantom **before** the send -- `settings.setForwarded(true)`
at `execution/RexxActivation.cpp:1372`, `stopExecution(RETURNED)` at `:1374`, `messageSend` at `:1382`
/`:1386` -- and `RexxActivation::trap` reads that flag first thing (`:2446`-`:2467`), with the comment
"if we're in the act of processing a FORWARD instruction, then this stack frame doesn't really exist
any more. We need to check the previous stack frame to see if it can handle this." So a `SIGNAL ON
SYNTAX` armed in the forwarding method does not see a condition the send raises. This crate leaves the
activation live and traps it. The program then takes a **different path and exits 0 with a
plausible stdout** where the oracle reports an error at rc 214.

The report itself names the phantom-before-send mechanism (its section on the crash shape) and uses it
to explain the licensed rc-245 divergence. This is the same mechanism's second, unlicensed
consequence.

```rexx
o = .K~new
say 'a' o~m
say 'b'
::class Inner
::method fail
  return 1/0
::class K
::attribute d
::method init
  expose d
  d = .Inner~new
::method m
  expose d
  signal on syntax name trap
  forward to (d) message('FAIL')
trap:
  return 'trapped-noncont'
```

```
oracle           rc 214, stdout EMPTY
                      6 *-* return 1/0
                     15 *-* forward to (d) message('FAIL')
                      2 *-* say 'a' o~m
                   Error 42 running T.REX line 6:  Arithmetic overflow/underflow.
                   Error 42.3:  Arithmetic overflow; divisor must not be zero.
ir, tree-walker  rc 0,   stdout "a trapped-noncont" / "b",   stderr EMPTY
```

**A second face of the same defect, with no `TO` and no second object** -- a non-continuing `FORWARD`
to a message name the receiver does not answer:

```rexx
::method missing
  signal on syntax name t3
  forward message('NOSUCHNAME')
t3:
  return 'trapped-missing'
```

```
oracle           rc 159, 97.1 Object "a K" does not understand message "NOSUCHNAME".
ir, tree-walker  rc 0,   stdout "c trapped-missing", stderr EMPTY
```

**The silent face of it.** With a trap in the caller *and* a trap in the forwarding method, both
sides exit 0 with empty stderr and the stdout differs by one word -- the oracle runs the caller's
handler, this crate runs the forwarder's:

```rexx
::method outer
  signal on syntax name otrap
  return self~m
otrap:
  return 'OUTER-handler'
::method m
  expose d
  signal on syntax name itrap
  forward to (d) message('FAIL')
itrap:
  return 'INNER-handler'
```

```
oracle           rc 0, stderr empty, stdout "a OUTER-handler" / "b"
ir, tree-walker  rc 0, stderr empty, stdout "a INNER-handler" / "b"
```

That is a **silent wrong answer**: same status, no report on either side, only the value differs.

**Three adjacent successes bound it to the send of a non-continuing `FORWARD`**, all byte-identical on
all three descriptors on both engines:

| arm | all three sides |
|---|---|
| the same trap with `CONTINUE` on the `FORWARD` | rc 0, `a trapped-cont` -- the oracle traps too, because a continuing forward is an ordinary send |
| the same failure reached through `DELEGATE`, trapped in the *sending* method | rc 0, `a trapped-delegate` |
| a `FORWARD` option's own refusal under a trap -- `arguments (.nil)` (98.946) and `class (5)` (88.914) | rc 0, `a trapped-args` / `b trapped-class` -- both are raised before the send, so both sides trap |
| the trap armed in the *caller* of the forwarding method instead, with no trap in the forwarder | rc 0, `a outer-trapped` -- the C++ drills to the previous non-forwarded frame, which is what this arm shows agreeing |

So it is not "conditions under `FORWARD`" and not "conditions under a delegated send": it is
specifically a condition arising **inside the send a non-continuing `FORWARD` performs**.

`Interp::exec_forward` performs `self.send_message(...)` and only afterwards returns `Flow::Return`.
Whatever the fix is, it has to make the trap search skip this activation for the duration of that
send while still leaving the `forward` clause on the traceback -- which the oracle does keep, verified
above and in `corpus/lang/forward_frame.rex`.

**No committed row sees this**: every `FORWARD` corpus program's failing arms fail at the top level
with no trap armed.

## MIN-1 (Minor): under `trace i`, an invalid `FORWARD CLASS` writes a `>K>` line the oracle does not

The C++ raises 88.914 **between** evaluating `CLASS` and tracing it
(`instructions/ForwardInstruction.cpp:168`-`:175`: `evaluate`, then `isInstanceOf(TheClassClass)`
with its `reportException`, and only then `traceKeywordResult(GlobalNames::CLASS, ...)`).
`Interp::forward_keyword` traces first and `exec_forward` checks after, so the crate writes one extra
`>K>` line on a path the oracle leaves silent. `rc` and stdout agree; **stderr does not**.

```rexx
o = .K~new
say 'a' o~m
::class K
::method m
  trace i
  forward message('OTHER') class (5)
::method other
  return 'other-m'
```

```
oracle           ...  >L>   "5"
                      <I< Method "M" with scope "K" in package "T.REX".
ir, tree-walker  ...  >L>   "5"
                      >K>   "CLASS" => "5"          <- not on the oracle
                      <I< Method "M" with scope "K" in package "T.REX".
```

Both sides then agree on `Error 88.914: Argument SCOPE must be an instance of the Class class.` at
rc 168.

**The adjacent success**, byte-identical on all three sides, which is what says the option order and
the good-path trace are right: `forward message('OTHER') class (super) to (self)` under `trace i`
emits `>V> SELF`, `>K> "TO"`, `>L> "OTHER"`, `>K> "MESSAGE"`, `>V> SUPER`, `>K> "CLASS"` in that
order on all three.

No committed row combines `trace i` with an invalid `CLASS`, so nothing sees this.

## IMP-1 (Important): the `98.937` witness added in `acb015bd4` has no recorded control

Global constraint: "A witness must be run against a control that makes it fail... **record the
control as run**, with its transcript, or the acceptance is not met." The report's control section is
taken at `d8e48d353` and its `row / reddened by` table lists eleven rows.
`lang/forward_after_reply.rex`, the twelfth entry, added in `acb015bd4` together with
`Interp::forward_after_reply`, is **not in that table and no arm in the report deletes the 98.937
check**. Its only recorded redden is M3, which deletes `FORWARD` entirely -- that shows the row runs,
not that it witnesses the rule the commit is about.

**I ran the two missing arms and the witness is sound** -- this is a record defect, not a behaviour
one. M10 deletes the check (`forward_after_reply` always answers `Ok`): `310 of 311`, with
`lang/forward_after_reply.rex` alone reddening on `stderr differ`. M11 keys the check to the *keyword*
rather than the replied value (`matches!(self.activation().reply, ReplyState::None)` in place of
`!self.activation().replied_a_value`): also `310 of 311`, that row alone, `stderr differ` -- so the
file's `bare` arm really does separate "a REPLY happened" from "a REPLY carried a value", which is
the whole claim `acb015bd4`'s message makes. Both arms leave table D's two rows at `agree`.

## Controls re-run (priority 4)

Every arm is a `git archive 8e891cd5a | tar -x` extract with `CARGO_TARGET_DIR` of its own, sources
`touch`ed before each build, and

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_d --test corpus --no-fail-fast
```

read unpiped from its own file. **Baseline at `8e891cd5a`, unmutated: exit 0, `311 of 311 matching`,
table D `5b: 2 rows, 0 not yet agree`.** (The report's figures are 310/311 at `d8e48d353` and
311/311 after `acb015bd4`; mine are all at the tip, so a row count is one higher than the report's
where it quotes the earlier commit.)

| arm | what it deletes | table D 5b | corpus | rows reddened |
|---|---|---|---|---|
| M1 | `send_to_delegate` answers the delegate object instead of sending | both `diverge-stdout`, exit 101 | 306 of 311 | the five `DELEGATE` rows, no `FORWARD` row |
| M1' | M1 **plus** the pre-Task-4 probes restored and Task 4's corpus entries removed | both **`agree`, exit 0**| **299 of 299, exit 0** | none |
| M2 | `delegate_variable` reads the directive's own name | `diverge-both` / `diverge-stdout`, exit 101 | 306 of 311 | the same five |
| M2-spec | M2, with the `::METHOD` probe's value changed to the spec's old `'abcdef'` | `diverge-both` / **`agree`** | 307 of 311 | four |
| M3 | the `InstructionKind::Forward` arm in `step` | both **`agree`, exit 0** | 304 of 311 | the seven `FORWARD` rows |
| M4 | `CONTINUE` ignored, every `FORWARD` returns | both `agree`, exit 0 | 310 of 311 | `forward_continue.rex` alone |
| M5 | `TO` evaluated and discarded | both `agree`, exit 0 | 309 of 311 | `forward_options.rex`, `forward_frame.rex` |

## CRIT-3 (Critical): `FORWARD ARGUMENTS` over a stem is a silent wrong answer

`Interp::forward_arguments` reuses `Interp::operator_operand_gap` (`eval.rs:1635`) to decide what a
non-array `ARGUMENTS` value may be. That predicate answers "what no operator can take", which is a
different set from "what `requestArray` cannot answer", and a **stem** is the arm where the two
disagree without any refusal to make it visible: the crate passes the stem object as one argument
where the oracle expands it through `StemClass::makeArray`.

```rexx
o = .K~new
say 'a' o~go
::class K
::method go
  s.0 = 3
  s.1 = 'p'
  s.2 = 'q'
  s.3 = 'r'
  forward message('SEEN') arguments (s.)
::method seen
  return 'seen' arg() '['arg(1)']' '['arg(2)']' '['arg(3)']'
```

```
oracle           rc 0, stderr empty, stdout "a seen 4 [0] [3] [2]"
ir, tree-walker  rc 0, stderr empty, stdout "a seen 1 [S.] [] []"
```

Four arguments against one. Two smaller faces of the same thing, also rc 0 with empty stderr on all
three sides:

| body of `go` | oracle | ir, tree-walker |
|---|---|---|
| `a. = 'dflt'` then `arguments (a.)` | `a seen 0 []` -- **no arguments at all** | `a seen 1 [dflt]` |
| `b.1 = 'x'` then `arguments (b.)` | `b seen 1 [1]` | `b seen 1 [B.]` |

Note the item *values* the oracle produces are its stem tails and their order is hash order, which is
not something to pin a row on; the **count** is deterministic and is what diverges. A witness should
compare `arg()` and not the items.

**The neighbouring arms of the same predicate are loud rather than silent** (see IMP-2), which is why
this one is the serious member of the group: nothing anywhere reports that a conversion did not
happen.

## IMP-2 (Important): the other `FORWARD ARGUMENTS` conversions are loud divergences and none is recorded

`forward_arguments` refuses whatever `operator_operand_gap` names, at rc 120. For two of those shapes
the oracle's answer is **98.946, the error this same function already raises for `.nil`**, so the
crate spends a `not implemented` refusal where the correct answer was one line away; for a third the
oracle answers rc 0. All measured from a fresh empty directory, both engines identical:

| `arguments (x)` where `x` is | oracle | ir, tree-walker |
|---|---|---|
| an instance of a user class (`self`) | rc 158, `98.946 FORWARD arguments must be a single-dimensional array of values.` | rc 120, `an instance of a user class as FORWARD ARGUMENTS is not implemented (Phase 5)` |
| a class object (`.String`) | rc 158, the same 98.946 | rc 120, `a class object as FORWARD ARGUMENTS is not implemented (Phase 5)` |
| a `.StringTable` with one entry | **rc 0**, `a seen 1 [AA]` | rc 120, `one of the interpreter's own objects as FORWARD ARGUMENTS is not implemented (Phase 5)` |

Two more are blocked behind class surfaces this phase has not built, so they are not this task's but
they are on this path and are unrecorded too: `arguments (.array~new(2,2))` is oracle rc 158 98.946
against rc 120 `method "NEW" of class "Array" is not implemented`, and a `.List` or `.Queue` is
oracle rc 0 `a seen 2 [p] [q]` against rc 120 on `~new`.

**And one on the `DELEGATE` side**: a delegate variable that is not a simple name is
`Loud::accessor_variable` at rc 120 -- `::method m delegate a.b` over `a.b = .Inner~new` is oracle
rc 0 `a inner-m` against rc 120 `a generated accessor for the attribute "A.B" is not implemented`,
and the stem form `delegate a.` is the same. That refusal existed for generated accessors; this task
routes a new construct into it, and the wording ("a generated accessor for the attribute") no longer
describes the site.

Task 9's divergence list gained three entries from this task (`.Array~of`, the `INTERPRET` traceback,
the self-forward licence). None of the six above is on it. The phase's own rule is that a divergence
is recorded or it reads as an unowned defect at audit.
| M6 | `MESSAGE` evaluated, converted, upcased and discarded | -- | **did not terminate** (see OBS-1) |
| M6b | M6, with `lang/forward_after_reply.rex` out of the subset -- which is the corpus the report's own M6 ran against, that file being later | both `agree`, exit 0 | 307 of 310 | `forward_options.rex`, `forward_continue.rex`, `forward_arguments_not_an_array.rex` |
| M7 | `CLASS` evaluated, checked, then dropped from the send | both `agree`, exit 0 | 310 of 311 | `forward_class_super.rex` alone |
| M8 | `ARGUMENTS`' converted values never reach the argument list | both `agree`, exit 0 | 310 of 311 | `forward_options.rex` alone |
| M9 | `ARRAY`'s evaluated items never reach the argument list | both `agree`, exit 0 | 310 of 311 | `forward_options.rex` alone |
| **M10** (mine, IMP-1) | `forward_after_reply` always answers `Ok` -- the 98.937 check deleted | both `agree`, exit 0 | 310 of 311 | `forward_after_reply.rex` alone, `stderr differ` |
| **M11** (mine, IMP-1) | the same check keyed to `ReplyState` instead of `replied_a_value` -- the keyword, not the value | both `agree`, exit 0 | 310 of 311 | `forward_after_reply.rex` alone, `stderr differ` |

**Every arm the report claims reproduces**, with the row counts one higher where the report quotes
`d8e48d353`. In particular M1' and M2-spec, the two that carry the argument for the task existing,
both reproduce exactly: a build where `DELEGATE` does nothing at all reads `agree`/`agree` at exit 0
over the *pre-Task-4* probes and `299 of 299` over the pre-Task-4 corpus; and the spec's old
`'abcdef'` value makes the `::METHOD` row `agree` under a build that reads the directive's name.

I ran `gate_table_d` and `corpus` as one `cargo test` invocation, so the `exit=101` on a reddening arm
is the corpus test's; the table D verdicts are read from the report the gate-table test prints.

## MIN-2 (Minor): a stale sentence in the plan, and one gap the gate row leaves to a unit test

* **Stale.** The plan's Task 4 section still ends "`docs/superpowers/specs/2026-08-27-phase-5b-instances.md`
  still carries the `'abcdef'` spelling in D62's quoted probe and **is the controller's to correct**"
  (`docs/superpowers/plans/2026-08-27-phase-5b.md:426`-`:427`, the paragraph's last sentence). The controller corrected it at
  `1aa54204b`; `/bin/grep -n abcdef` on the spec answers `799: d = 'abcdefgh'` and
  `828: d = 'abcdefgh'` and nothing else. The sentence now reads as an open item that is closed.
* **The `::ATTRIBUTE ... DELEGATE` getter is never sent by its gate row.**
  `attribute__delegate__subkeyword.rex` sends the setter (`k~at = 'via-set'`) and then reads back
  through `k~peek~at`, which is *Inner's* own generated getter, not K's delegating one. The delegate
  getter for that directive form is covered only by
  `dispatch.rs`'s `a_delegate_directive_forwards_under_every_key_it_claims`, whose second row is
  `say .K~a` over `::attribute a class delegate p`. Verified against the oracle here that the getter
  does work through an instance -- `::attribute at delegate d` over an `Inner` whose `at` is
  `inner-initial` prints `get inner-initial` then `get2 v2` at rc 0 on all three sides -- so this is
  a coverage note, not a defect. Worth one more send in that probe or a line in
  `delegate_variable.rex` if a future task touches either.
* **Set cardinality in comments.** The global constraint is "no comment states the size of a set
  (true counts included)". Four added comments do: `run.rs`'s `// FORWARD, with any of its six
  options`, `exec_forward`'s "`RexxActivation::forward`'s own three defaults",
  `forward_arguments`'s "**Three sources, and the third is the default.**", and
  `corpus/lang/forward_options.rex`'s "FORWARD's four value-carrying options, and the three defaults
  it fills in". All four are fixed language sets rather than in-repo aggregates, and the plan itself
  writes "names six options", so this may be inside the intended line; flagged for the controller to
  rule rather than asserted as a breach.

## OBS-1: arm M6 does not terminate, and what that says about the licensed self-forward

Running the report's M6 mutation at the tip, the corpus differential **did not finish in about six
minutes** and was killed. `ps` showed the test binary itself at 99% CPU with 36 MB RSS and **no child
process**, so the crate side runs in-process and the harness's 10 s deadline (`ORACLE_DEADLINE`,
`tests/support/oracle.rs:113`) does not bound it. Re-running the identical mutation with
`lang/forward_after_reply.rex` removed from `corpus/phase-5b.txt` finishes in the usual time (M6b
above, `307 of 310`), so that one program is the one that does not come back. The report's own M6 ran
at `d8e48d353`, before that file existed, which is why it did not meet this.

Under M6 that program's `::method bare` -- `reply` then `forward message('SEEN')` -- becomes a
non-continuing `FORWARD` that resolves back to the method it is in, i.e. the shape
`corpus/oracle-crashes.txt` names, **but reached after a `REPLY`**. Task 9's licence records that this
crate answers that shape `rc 245` because `MAX_ACTIVATION_DEPTH` counts the recursion; a self-forward
whose method has already replied returns to its caller at each `reply`, so the depth may never grow.
Measured directly, that one program under the M6 build, **crate alone, the oracle never handed it**:

```
REXX_ENGINE=ir <M6 build>/rexx-run corpus/lang/forward_after_reply.rex   under timeout -s KILL 60
  rc 137 (killed by the timeout), elapsed 60s, stdout 0 bytes, stderr 0 bytes
```

against the unmutated build's rc 0 in well under a second. So the non-termination is that program
under that mutation, not a harness artifact.

**I did not construct the equivalent unmutated program to check** -- it is the forbidden shape and
the rule is not to build toward it -- so the *explanation* above (that a replied activation returns
before the depth guard can count it) is inference, not measurement.

Worth the controller's attention because the licence as written says the crate's answer to the
self-forward shape is a clean `rc 245`, and this is evidence that one member of that shape may
instead hang.

## Probing past the rows: what agrees

Every row here is oracle against `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, three descriptors
read separately, run from a fresh empty directory under
`( cd "$D"; ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 20 .../build/bin/rexx FILE )`
for the oracle and `timeout -s KILL 20 rexx-run` for the crate. **All three sides byte-identical**
unless the row is one of the findings above.

`DELEGATE`, the receivers and shapes the brief named plus more:

| probe | answer |
|---|---|
| delegate whose value is `.nil` | rc 159, `Object "The NIL object" does not understand message "M"` |
| delegate to a class object, both a name it answers and one it does not | rc 0 `a String`; rc 159 naming `"The String class"` and `"ZZZ"` |
| a delegate chain two deep, with and without arguments | rc 0 `a deep-m 0` / `b deep-m 2` |
| a delegate naming a name the target does not answer | rc 159, the *delegate* named as the receiver |
| `::ATTRIBUTE at DELEGATE d` **getter** through an instance, then setter, then getter | rc 0 `get inner-initial` / `get2 v2` |
| a class-scope pair: `::method cm class delegate p` and `::attribute cat class delegate p` | rc 0 `a inner-cm` / `b inner-cat` / `c set-cls` |
| `PRIVATE` on the *delegating* method | rc 0 through a sibling method, 97.2 from outside, naming `"a K"` |
| `PRIVATE` on the *delegated-to* method reached by `FORWARD TO` | rc 159, 97.2 naming `"an INNER"` |
| the delegate variable reassigned between sends | rc 0 `a inner-m` / `b other-m` |
| a subclass instance through an inherited delegate, `hasMethod`, `~class~id` | rc 0 `c inner-m` / `d 1` / `e K` |
| a delegate declared in a `MIXINCLASS` and inherited | rc 0 `a inner-m` |
| `DELEGATE` with the `GET` modifier, and with `SET` | rc 0 for the built half, 97.1 for the absent one, both names right |
| omitted and trailing arguments through a delegate, and `o~m:.K` | rc 0 `a inner-m 3 [1] [] [3]` / `b`, `c` both `0` |

`FORWARD`:

| probe | answer |
|---|---|
| `FORWARD` out of an `::ATTRIBUTE ... GET` body and an `::ATTRIBUTE ... SET` body, with **no** `MESSAGE` so the default name carries the `=` | rc 0 `a set-through` / `b set-through` |
| `FORWARD` inside a `DO` inside a `SELECT`, and the arm that falls through | rc 0 `a other-m` / `b fell-through` |
| `MESSAGE` given a number, and given an object | rc 159 naming message `"7"`; rc 159 naming `"A K"` |
| `CLASS` naming a valid ancestor, a grandparent, and the receiver's own class | rc 0 `a base-m` / `b gp-m` / `c k-m` |
| `TO (.nil)` | rc 159 naming `"The NIL object"` |
| multi-`INIT` chaining, the use `provide.xml` documents: `::method init` doing `forward class (super)`, and the `CONTINUE` variant | rc 0 `a base 1` / `b base 7` |
| the arguments default after an intervening method call, and after `o~t(4,,)` | rc 0 `a seen 2 [1] [2] []` / `b seen 1 [4] [] []` |
| `ARRAY (6,,)` with a trailing omission | rc 0 `c seen 1 [6] [] []` |
| `ARGUMENTS` of a literal number and of a computed one | rc 0 `a seen 1 [7]` / `b seen 1 [7]` |
| the forwarded-to method issuing its own `REPLY` | rc 0 `a replied-from-inner` |
| 98.937 precedence: `class (5)` and `arguments (.nil)` after a valued `REPLY`, then a `CONTINUE` forward followed by a non-continuing one | 88.914, 98.946, then 98.937, all three at the same clause numbers |
| the `98.937` / `98.936` neighbourhood -- valued `REPLY` + `FORWARD`, bare `REPLY` + `FORWARD`, valued `REPLY` + `CONTINUE` forward, valued `REPLY` + `RETURN`, bare `REPLY` + valued `RETURN` | rc 0 with 98.937 once, 98.936 twice, everything else silent |
| `trace i` over `forward message('OTHER') class (super) to (self)`, whole transcript | identical, `>V> SELF`, `>K> "TO"`, `>L>`, `>K> "MESSAGE"`, `>V> SUPER`, `>K> "CLASS"` |
| `trace i` over a delegated send from the top level, whole transcript | identical, `>M> "M" => "inner-m 5"` and no method-entry line |
| a traceback through `DELEGATE`, through the written-out `expose`/`forward` equivalence, and through a `FORWARD` **inside** a delegated call | 2 clause lines, 3 clause lines, 3 clause lines -- the report's frame claim, both directions |

The one **new** loud divergence in this list is the `MESSAGE`-given-an-object arm's neighbour
`message(.array~of)`, which is the already-recorded `.Array~of` gap.

## Claims of the report I re-measured rather than accepted

* **The two replaced table D rows now send, and the replacement earns its place.** Reproduced: M1'
  is `agree`/`agree` at exit 0 over the pre-Task-4 probes on a build where `DELEGATE` does nothing.
* **The `'abcdef'` coincidence.** Reproduced: M2-spec makes the `::METHOD` row `agree` under a
  wrong-variable build; the committed `'abcdefgh'` does not. The spec now carries `'abcdefgh'` at
  both sites (`/bin/grep -n abcdef` on the spec: lines 799 and 828, both `abcdefgh`).
* **Table D cannot see `FORWARD` at all.** Reproduced: M3 leaves both rows `agree` while seven
  corpus rows redden.
* **`dire.xml`'s equivalence is not observably exact.** Reproduced on the oracle: `::method m
  delegate d` gives `5 *-* return 1/0` then `2 *-* say 'a' o~fail`; the written-out body adds
  `13 *-* forward to (d)` between them. Both rc 214, `Error 42.3`, and both engines match their side.
* **The `INTERPRET` traceback difference is pre-existing.** Reproduced with the pinned
  `bench-baselines/pinned/rexx-run-f558ea501`, which prints the same two clause lines the tip does
  against the oracle's three.
* **The licensed self-forward answer is honoured.** Measured **on the crate alone**, both engines,
  never handed to the oracle: rc 245, stdout empty, stderr ending `Error 11.1:  Insufficient control
  stack space; cannot continue execution.`, 10,002 lines, 0.07 s.
* **No other corpus program contains a `FORWARD` clause.** Checked with a wider pattern than the
  report's -- `/bin/grep -rilE "(^|[^a-z_.])forward([^a-z_]|$)" --include=*.rex` over `corpus/`, whose
  `find . -name '*.rex' | wc -l` is 640 -- which names the seven `forward_*.rex` files plus seven files where the
  hits are prose in a comment or a string literal, and nothing else.
* **The scope move touched nothing else.** `instruction_owner` splits one arm into two, `owners.rs`
  moves one row and its two counts, `loud.rs` drops one witness and its two counts; no other
  instruction's owner or refusal text moves in the range. Spot-checked against the pinned binary:
  `options 'x'` is `rexx-exec: OPTIONS is not implemented (Phase 5)` at rc 120 on both, and a
  top-level `guard on` is 99.911 at rc 157 on both.
* **Both commit-message corrections the report records are real.** `acb015bd4` cites
  `RexxActivation.cpp:1370`-`:1374`; the check is the `if (settings.isReplyIssued() && result !=
  OREF_NULL)` at `:1367`. And `git diff d8e48d353 164d5c106 -- <the plan>` is +11 lines, all of them
  the `INTERPRET` entry, so the two Task 4 plan corrections and the other two Task 9 entries are
  indeed in `d8e48d353`.
* **`forward_after_reply.rex` does not depend on an ordering the oracle fails to reproduce**, which
  matters because both its arms `REPLY`. Twelve oracle runs: one distinct stdout, one distinct
  stderr, rc 0 every time.
* **Every intra-doc link this task added resolves.** Each `[`item`]` link in the range's added
  source lines names an item `/bin/grep` finds in the crate -- `generated_methods` and
  `native_externals` as fields, the rest as functions.
* **No non-ASCII in any line the range adds under `rust/`**, checked with
  `git diff ... | perl -ne 'print if /^\+/ && /[^\x00-\x7F]/'`, which prints nothing over 1,548
  added lines.

## What I did not verify

* **The five gates and the phase gate.** The brief assigns them to the controller and I did not run
  `cargo fmt`, `clippy`, the full workspace tests or gate table C. My own runs are `corpus` and
  `gate_table_d` only.
* **The performance sitting**, including the blocked second one. Not attempted, per the brief.
* **The `tr_in` / `tr_in3` whole-transcript rows as the report ran them.** I ran my own `trace i`
  transcripts over a good `CLASS` path and a delegated send, and the bad-`CLASS` path that is MIN-1.
* **The report's `.Array~of` and `INTERPRET` Task 9 entries beyond one probe each**; both reproduce.
* **Whether an unmutated self-forward after a `REPLY` hangs** -- OBS-1's mechanism. That program is
  the shape `corpus/oracle-crashes.txt` names and I did not build it.
