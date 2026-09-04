# Phase 5d Task 3 — operator methods on instances

**BASE `71153941a`.** Tree held alone. Plan `docs/superpowers/plans/2026-09-03-phase-5d.md`
(Task 3), spec `docs/superpowers/specs/2026-09-03-phase-5d-foundation.md` (§1, D77, D78).

**Committed before the gates**, per the plan's 2026-09-04 rule. The gate section below carries
placeholders and nothing else; the statuses are read from the background run's own file.

---

## The mechanism

**An operator is a message sent to its left operand.** `RexxObject`'s operator vtable
(`classes/ObjectClass.cpp:2738`-`:2795`) is thirty-two entries and each generic one is a
`messageSend` of the operator's own spelling, so an instance answers whatever its class defines,
`Object`'s own six comparisons and three concatenations when it defines none, and 97.1 when neither
does. A prefix operator sends **no argument at all** —
`prefixOperatorMethod`'s `operand == OREF_NULL ? 0 : 1`.

That is one function and three call sites:

* `Interp::operator_message_receiver` (`eval.rs`) — the object to send to, or `None`. It redirects
  through a stem's default and a variable reference's value exactly as `operator_operand_gap` does,
  and answers the redirected object rather than the stem or the reference.
* `Interp::send_operator` (`eval.rs`) — `send_message` with the caller's own context, and 91.999
  when the method returns nothing.
* asked from `Interp::apply_binary` (concatenation, comparison, logical), from
  `Interp::arith_left_operand` (the seven arithmetic operators and prefix `+`/`-`), and from
  `apply_prefix_body`'s `Not` arm. All three are entered by both engines.

**`Object`'s nine operator methods are registered natively** (`Setup.cpp:521`-`:530`, every row at
count 1): `=` and `==` as identity, `\=` `\==` `<>` `><` as its negation, and `||` `""` `" "` as
`requestString()` on the receiver followed by that string's own concatenation. Without them an
instance's `=` resolved to `Object`'s documented-but-hollow entry and refused.

### Where the argument list matters, measured rather than assumed

| probe | oracle |
|---|---|
| `o + 1` with `::METHOD "+"` | the method's answer |
| `o + 1` with no `"+"` | `97.1 Object "a K" does not understand message "+".` rc 159 |
| `-o` with `::METHOD "-"` | the method, `arg()` = **0**; `o - 1` reaches the same method with `1` |
| `o = p` on two plain instances | `0` — identity, not a comparison of renderings |
| `o = p` where the class defines `"="` but not `"\="` | `\=` is `Object`'s own, **not** the negation of the user's `=` |
| `o < 1` where the class defines `::METHOD unknown` | `UNKNOWN` catches it; `o = p` does not, because `Object` holds `=` |
| `::METHOD "+"` ending in a bare `return` | `91.999 Message "+" did not return a result.` rc 165 |
| `::METHOD "+" PRIVATE` | `97.2 ... cannot accept private message "+" from this context.` rc 159 |
| `s. = .K~new; say s. + 1` | the instance's own `+` |
| `zz = .K~new; say (>zz) + 1` | the instance's own `+` |
| `.DateTime~new + .TimeSpan~fromSeconds(5)` | rc 0, a `DateTime` |

Every one of those agrees byte for byte on all three descriptors, on both engines, at HEAD.

### The half that was a silent wrong answer, not a refusal

Concatenation never asked `operator_operand_gap` at all, so an instance on the **left** of `||`,
a blank or an abuttal rendered its default name and exited 0. Measured at BASE against the oracle,
three descriptors:

```
::METHOD string    returning 'STR'    o || 'x'    BASE  a Sx     oracle  STRx
::METHOD makeString returning 'MKS'   o || 'x'    BASE  a Sx     oracle  MKSx
::METHOD '||'      returning 'CAT<>'  o || 'x'    BASE  a Cx     oracle  CAT<x>
```

rc 0 and empty stderr on both sides in every row — the worst defect class this project recognises,
and the one no refusal-shaped witness would have caught. `'x' || o` was already right at BASE, which
is why the right-operand path is untouched.

---

## `String~sign`, and why it is here

`DateTime~compareTo` ends `return (utcTimeStamp - othertime)~sign` (`CoreClasses.orx:2517`), so all
twenty of that class's documented comparison operators refused on one missing `String` body while
its arithmetic already worked. `RexxString::sign` is `ArithmeticMethod(Sign(), "SIGN")`
(`classes/StringClass.cpp:1084`) — the `SIGN` builtin's own computation — so
`builtin::numeric::sign_of` is now shared by the builtin and the method rather than written twice.

This is D78's licence read as D78 states it: the documented method should work. It is **not** row
progress and is not reported as any — see D77 below.

`TimeSpan`'s comparisons needed nothing: measured, `t = t` and `t < t` answer at BASE+operators
without it, because `TimeSpan~compareTo` does not reach `~sign`.

---

## Sizing it: the compiler, not grep

`operator_operand_gap`'s return type was changed from `Option<&'static str>` to a two-variant enum
and the workspace checked. **Twelve errors**: four inside the function's own arms, and **eight call
sites** — `eval.rs`'s prefix `\`, arithmetic left operand, comparison and logical, and `run.rs`'s
`RAISE ADDITIONAL`, `DO OVER` target, `DO` header value and controlled-loop control variable. The
probe was reverted.

**The concatenation site is a ninth, and neither the compiler nor a grep over that name could have
found it**, because it never asked the gap. It is where the silent wrong answers above lived.

---

## What the run.rs half does *not* do, and the measurement that decided it

`DO`'s header positions and a controlled loop's control variable **are** operator applications on
the oracle — measured, `do i = 1 to o` on a plain instance is `97.1 ... does not understand message
"+".` — and they keep their existing loud refusal here. Two oracle probes say why, and both are
about the *bound*, not the send:

```
::METHOD '+' returning '1234'   numeric digits 2   do i = o to 9999 for 3
    oracle  1234 / 1.2E+3 / 1.2E+3     -- the send's answer is NOT rounded a second time
::METHOD '+' returning 'abc'                       do i = 1 to o for 3
    oracle  1 / 2 / 3                  -- the bound stays 'abc' and `i > 'abc'` is a string compare
```

`ControlledLoop::setup` stores what the `+` answered as an **object** and compares through the
control variable's own `>` (`instructions/DoBlockComponents.cpp:129`-`:173`), where
`Interp::header_number` returns a `Number`. Making those two positions dispatch means changing what
a loop's bound is, which is a change to the loop's own hot path and not this one. They stay rc 120,
which is safe and is what they were.

`RAISE ADDITIONAL` and `DO OVER`'s target are `requestArray`, not operators, and are untouched.

---

## Receiver kinds this does not extend to, each with the oracle's answer

Only `Body::Instance` (and a stem or reference that redirects to one) dispatches. Class objects,
arrays and the interpreter's own objects keep `Loud::operator_operand`. Measured:

```
.Array = .Array          oracle 1     .Array + 1        oracle 97.1 rc 159
.Array == .String        oracle 0     a = .Array~of(1,2); a + 1   oracle 97.1 rc 159
.methods == .routines    oracle 0     a = a             oracle 1
.environment + 1         oracle rc 0, "The NIL object"
```

The last line is why the set stops where it does: a `Directory` answers `+` **through its own
`UNKNOWN`**, which this crate models nothing of, so extending the send to `Body::Native` would turn
a loud refusal into a wrong 97.1. Class objects and arrays are coherent and were left out as scope
rather than as a difficulty; `Class`'s six comparison rows and `String`'s thirty-four operator rows
belong with them.

---

## A pre-existing divergence this widens the reach of, and does not create

`::METHOD string` returning another instance recurses in the oracle's `requestString` and is
`Error 5, System resources exhausted` at rc 251; this crate answers the default name at rc 0. That
is `Interp::required_string_value`, and it is reachable at BASE from the **right** operand —
measured, `'x' || o` on such a class is rc 0 here and rc 251 on the oracle at BASE and at HEAD
alike. `Object~||` now reaches the same converter from the left, so the same divergence has one
more route. Nothing about it changed; it is named here so it is not read as new.

---

## The method-body table

Refreshed with `REXX_METHOD_BODIES_REFRESH=1`. **Eleven rows moved `loud` → `answers` and nothing
else moved**; the refresh did not refuse.

| verdict | BASE | HEAD |
|---|---|---|
| `loud` | 673 | **662** |
| `answers` | 665 | **676** |
| `diverge` | 7 | 7 |
| `unstable` | 2 | 2 |

The eleven: `Object`'s nine operator rows, each `loud [method "=" of class "Object"]` → `answers
[rc 163]` — the zero-argument send's `93.903`, which is what the oracle gives too — plus
`String sign` (`answers [rc 163]`) and `TimeSpan sign` (`answers [rc 0]`, a value the two sides
agree on).

### The table carried an evidence value that changed with the hour, and it did so at BASE

Running the sweep against the committed table at 07:03 CEST reported one row's drift that no change
of mine can reach: `DateTime today (class arm): diverge [diverge-stdout] -> diverge [agree]`. The
verdict is stable — the three-environment rule Task 2 built is what makes it so — but the
**evidence** was `verdict(compare_raw(crate, oracle))` against *this machine's zone alone*, and that
comparison is exactly the one that agrees for twenty-two hours a day. Measured the same minute:

```
crate                     2026-09-04T00:00:00.000000
oracle TZ=machine (CEST)  2026-09-04T00:00:00.000000     agrees
oracle TZ=Etc/GMT+12      2026-09-03T00:00:00.000000     differs
oracle TZ=Etc/GMT-14      2026-09-04T00:00:00.000000     agrees
```

`DateTime~today`'s body is `self~fromStandardDate(date('s'),, offset)` and reaches no operator, so
this is Task 2's residual and not mine: **the committed table was red at BASE at this hour**, and
would have been red for the other two hours had I committed `agree`. `channels_that_moved_anywhere`
now unions the three environments' comparisons, which for that row is `diverge-stdout` at every
hour because at least one environment always disagrees and never on any other channel. The
committed row is byte-identical to BASE's again.

---

## D77: the 110 open rows did not move

From the phase gate's own report at the committed tree:

```
5a: 135 rows, 0 not yet agree      6:  13 rows, 13 not yet agree
5b:   6 rows, 0 not yet agree      7:  94 rows, 82 not yet agree
5c: 1225 rows, 0 not yet agree     deferred-rexxcontext-stackframes: 10 rows, 10 not yet agree
                                   never-expected-to-agree: 5 rows, 5 not yet agree
gated by this run: 0 row(s)
```

`13 + 82 + 10 + 5 = 110`, identical to Task 2's reading. Table D is unchanged: `5d: 1 rows, 1 not
yet agree`, which is Task 5's `requires__namespace__subkeyword`.

**`Alarm` and `Ticker` were not touched and do not construct.** Measured against the BASE build:
`.Alarm~new(1, .Object~new)` and `.Ticker~new(1, .Object~new)` are rc 168 at BASE and rc 168 at
HEAD, with **byte-identical stderr** in each case. No `class-set.txt` row was edited and
`corpus/docs/` is unmodified. The operator work and `String~sign` move both classes *closer* to
constructing, which is D77's named temptation and is not progress against anything.

---

## The witness

`rust/corpus/lang/operator_methods.rex`, run by
**`rust/crates/rexx-exec/tests/operator_methods.rs`** — the interim test binary Task 6 folds into
`corpus/phase-5d.txt` and deletes. `corpus/unfiled.txt` names the program, so
`every_lang_program_is_run_or_named_unfiled` sees it. **`corpus/phase-5d.txt` was not created.**

It covers a user-defined `+`, `-`, `*`, `=`, `<` and prefix `-`; `UNKNOWN` catching `**` and `>>`;
`Object`'s six identity comparisons and three concatenations; a class overriding `string` and one
overriding `makeString`; `DateTime` + `TimeSpan` arithmetic and the four ordering comparisons that
needed `~sign`; the stem and variable-reference redirects; a receiver named `'1'`, whose every row
would answer differently if the operator read that rendering as text; and, trapped by `SIGNAL ON
SYNTAX` so the program can continue past it, **the 97.1 that must remain when no operator method is
defined**.

**Shown to fail at BASE.** In a `git archive` extract of `71153941a` at
`/home/moritz/dev/repos/claude-build-scratch/task-3/base/` with `CARGO_TARGET_DIR` at
`.../task-3/base-target` (sha256 `ee36f252561c20ad6ea8a4111da129383fe6a74e8926cb7ff804c8f62ea1f227`,
distinct from HEAD's `50941de731b6639ffc2233ca9281145e8b558c92f8d250f624190e51d8bd37d0`):

```
BASE tree-walker  rc 120  stdout empty
BASE ir           rc 120  stdout empty
                  stderr: rexx-exec: the operator `+` applied to an instance of a user class
                          is not implemented (Phase 5)
oracle            rc 0    16 lines of stdout, empty stderr
```

---

## Committed text this change falsified, corrected rather than left

* `corpus/lang/instance_named_operands.rex`'s header said its refusals were "this crate's own loud
  ones and no differential row can carry them". They are the oracle's own 97.1 now. The comment is
  corrected and `crates/rexx-parse/tests/sourceline_oracle/instance_named_operands.txt` regenerated
  with that test module's documented `.Package~new` driver — the four changed lines and nothing
  else. The program's output is unchanged.
* `eval.rs`'s `a_named_instance_is_never_an_operators_left_operand` asserted the rc-120 refusal on
  eighteen expressions. It is now
  `an_operator_on_an_instance_is_the_send_the_oracle_makes`: twenty-one expressions asserting
  `97.1 Object "1" does not understand message "<op>".` at rc 159, ten asserting the values the
  oracle gives (`o = 1` is `0`, where a converted receiver would give `1`), and the three adjacent
  successes that stop it being satisfied by sending every operator.
* `corpus/refusal-sites.tsv`'s `method_target_not_a_number` row moved `body` →
  `body+send`/`agrees`/`yes`/`93.943`/`'abc'~sign`, because `String~sign` puts that constructor on
  the send surface. Its own test re-derives columns 1–4 and was what caught it.

---

## Performance

**Read after the change, with a do-nothing control, because the axis that moved most is not the axis
the change touches.** Three release builds, own `CARGO_TARGET_DIR` each, distinct sha256s:
`base` (`71153941a`), `control` (BASE plus every new function, every `NATIVE_METHODS` row and
`String~sign`, with the three operator-path **call sites removed** — it refuses the witness at rc 120
exactly as BASE does), and `head`. Five rounds, `instructions:u`, large size, committed as
`bench-baselines/phase-5d-arms.tsv`.

| axis | arm | `base>control` | `base>head` |
|---|---|---|---|
| `alloc4c` | tw | +0.000% | +0.307% |
| `alloc4c` | ir | -0.075% | +0.357% |
| `arith` | tw | **+0.990%** | **+0.991%** |
| `arith` | ir | **+0.987%** | **+0.985%** |
| `compound` | tw | +0.002% | +0.000% |
| `compound` | ir | -0.092% | -0.091% |
| `dispatch` | tw | -0.000% | -0.000% |
| `dispatch` | ir | -0.046% | -0.047% |
| `dispatchclass` | tw | -0.142% | -0.185% |
| `dispatchclass` | ir | -0.072% | -0.072% |
| `emptyloop` | tw | -0.001% | -0.001% |
| `emptyloop` | ir | -0.263% | -0.260% |
| `rexxcps` (small) | tw | +0.031% | **+0.845%** |
| `rexxcps` (small) | ir | -0.027% | **+0.967%** |
| `strings` | tw | +0.001% | +0.164% |
| `strings` | ir | -0.069% | +0.178% |
| `varlookup` | tw | +0.000% | +0.000% |
| `varlookup` | ir | -0.226% | -0.225% |

**The control is the finding, and it points the opposite way on the two axes that moved most.**
`arith` reads +0.99% under a build that does **no operator dispatch at all**, and +0.99% under the
real change: that axis's figure is code layout, and the dispatch costs it nothing. `rexxcps` reads
+0.03%/−0.03% under the control and +0.85%/+0.97% under the change, so **that one is real** and is
the largest cost this lands — it is the clause-rate axis, and `Interp::apply_binary`'s check is on
every comparison and every concatenation it runs. `alloc4c` (+0.31%/+0.36% over a flat control) and
`strings` (+0.16%/+0.18%) are the same check, smaller. Every other axis is within a twentieth of a
percent of its control.

Cycles are an order of magnitude noisier here and are not the A/B instrument: `dispatchclass ir`
reads +6.67% in cycles at −0.072% in instructions, and `dispatch tw` reads −3.73% at −0.000%. The
whole cycles column is in the committed file.

`Interp::arith_left_operand` asks for the send only where `Interp::to_number` has already refused,
which is sound because every value `operator_message_receiver` names is one that refuses;
`a_value_the_operator_gap_names_parses_as_no_number` now holds that premise as well as the older
one. **Its own effect is below the layout floor on the axis it touches and I make no claim for it.**

---

## Controls

* **The witness fails at BASE**, on both engines, in a separate extract with its own target
  directory. Above.
* **The do-nothing control** for the benchmark, above: a third build that carries every new
  definition and no call site, confirmed to refuse the witness.
* **The blast-radius probe**: the type change and its twelve errors, reverted and the file restored
  from a copy taken before it (`cmp` clean — `git status` reported no modification to `eval.rs`
  before the real work began).
* **The method-body gate's own refusal path was not exercised by this task**; Task 2's controls 1b
  and 13 are its witnesses and nothing here weakened them.

---

## What I did not do

* **`String`'s thirty-four operator method rows and `Class`'s six** — `'abc'~'+'(1)`, `5~'+'(1)`,
  `.Array~'='(.Array)` — stay `loud`. They are the operator *sent as an explicit message to a
  primitive receiver*, a different surface from the operator syntax this task fixes, and they carry
  an arity matrix of their own (`String`'s `+`, `-` and `\` have a prefix form at zero arguments:
  measured, `AddMethod("\\", RexxString::notOp, 0)` against `AddMethod("+", RexxString::plus, 1)`).
  Every one is a safe rc-120 refusal today. Measured targets are in "Receiver kinds" above.
* **The `DO` header's three positions and the controlled-loop control variable** stay loud, for the
  measured reason above: the oracle keeps the bound as an object and `header_number` returns a
  `Number`.
* **`Body::Native`, class objects and arrays** keep `Loud::operator_operand`. `.environment + 1` is
  the reason.
* **`.DateTime~new - .TimeSpan~...` still cannot print through `~string`**: `TimeSpan~string`
  reaches `String~right`, which is unimplemented. rc 120, unchanged. `~isoDate` and
  `~totalSeconds` work and are what the witness uses.
* **I fixed none of the seven `diverge` rows**, and did not touch `DATE()`/`TIME()`. The six
  `DateTime` rows the spec's handover attributes to that Phase 4 defect are still `diverge`.
* **I did not create `corpus/phase-5d.txt`** and did not add a row to any `SUBSET_FILES` list.
* **I did not run `clippy` from a clean target directory.** `rust/CLAUDE.md` asks for that at a
  phase boundary; this is not one.
* **I did not measure `startup`, `alloc`, `heapshape` or the `bench-control` axes** — the sitting
  covers the nine axes `phase-5b-arms.tsv` carries.
* **The `commit` column of `phase-5d-arms.tsv` holds `71153941a`, the base of the comparison, not
  the commit the `head` build is.** A file cannot carry the sha of the commit that contains it; the
  `build` column is what names each side, which is what `bench-baselines/README.md` says it is for.
* **One sitting was started and voided rather than reported.** It was measuring
  `target/release/rexx-run` when a two-comment edit rebuilt that path mid-run, so three of its nine
  axes describe a different binary from the other six. Killed, and the sitting below runs against a
  copy at `claude-build-scratch/task-3/head-bin/rexx-run` that no rebuild can reach. **A doc comment
  does change the binary** -- `debug = true` puts the source in the debug info -- which is what made
  the void one void.

---

## Files

* `rust/crates/rexx-exec/src/eval.rs` — `operator_message_receiver`, `send_operator`,
  `ArithOperand`, the three call sites, and the rewritten test
* `rust/crates/rexx-exec/src/dispatch.rs` — `Object`'s nine operator methods, `String~sign`, and
  their `NATIVE_METHODS` rows
* `rust/crates/rexx-exec/src/builtin.rs`, `rust/crates/rexx-exec/src/builtin/numeric.rs` —
  `numeric::sign_of`, shared by the builtin and the method
* `rust/crates/rexx-exec/tests/method_bodies.rs` — `channels_that_moved_anywhere`
* `rust/corpus/method-bodies.txt` (refreshed), `rust/corpus/refusal-sites.tsv` (re-derived),
  `rust/corpus/unfiled.txt`
* `rust/corpus/lang/operator_methods.rex` (new), `rust/crates/rexx-exec/tests/operator_methods.rs`
  (new, **the interim witness binary Task 6 folds in**),
  `rust/crates/rexx-parse/tests/sourceline_oracle/operator_methods.txt` (new)
* `rust/corpus/lang/instance_named_operands.rex` and its sourceline expectation — the falsified
  comment
* `rust/bench-baselines/phase-5d-arms.tsv` (new)
* `docs/superpowers/records/2026-09-03-phase-5d/task-3-report.md` (this file; the briefed
  `.superpowers/sdd/` path is git-ignored)

**Scratch left behind, not deleted:**
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task-3/`
(the oracle and differential probe directories `probe/d001`–`d023` with their `orc.sh`, `crt.sh` and
`diff.sh` drivers, the pre-change copy of `eval.rs`, the pre-refresh copy of `method-bodies.txt`,
the sourceline regeneration driver, and every gate and benchmark log) and
`/home/moritz/dev/repos/claude-build-scratch/task-3/` on real disk (`base/` and `control/` source
extracts, `base-target/` and `control-target/` build trees).

---

## Gates

Run from `rust/` by a background job writing each status **unpiped** to a file as it goes, with a
pidfile. Started after the commit below; the controller reads the statuses and fills this table in.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |
| 7 | `REXX_PHASE_GATE=5d …` (same command) | **101** |

**Filled in by the controller from the run's own `statuses.txt`, not by the task agent**, under the
commit-before-gating protocol this task was the first to use: the agent committed at `c1afdb9e9`,
started the suite in the background and stopped without waiting. The status file pins the commit,
`c1afdb9e9b8d5527ee34d9270cfe67657060d87c`, so the gated tree is the committed tree by construction.
The run took 27m45s (08:37:58 to 09:05:43) — the window that lost Task 2.

**G7 is expected non-zero and is not this task's**: table D's `requires__namespace__subkeyword` row
is Task 5's, exactly as it was after Task 2.
