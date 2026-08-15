# Re-review of Task 7 fix round 1

Scope: commit `431e2698` against its parent `f906aabc`. Did fix round 1
close the seven findings of `task-7-review.md`, and did it introduce
anything new. **Not** a re-review of Task 7 as a whole; the prior review's
non-findings are taken as settled.

Working tree clean at start and at end. Every tracked-file mutation below
was applied, measured, and reverted with
`git checkout -- rust/crates/rexx-exec/src/`; `git status --porcelain` is
empty now and was empty after each restore. No tracked file was left
modified.

Every claim is labelled **RAN** or **REASONED**.

## How things were established

Three tools built for this re-review, all outside the repository:

* `<scratchpad>/stressprobe` -- a throwaway cargo crate with a path
  dependency on `rexx-exec`, calling the `#[doc(hidden)]`
  `run_program_collect_every_alloc` and `run_program` on the same file and
  comparing the three descriptors. This is how the rooting question was
  answered by running rather than by reading, **with a negative control**.
* `<scratchpad>/wt-before` -- a detached `git worktree` at `f906aabc` with
  its own `rexx-run`, so "is this new?" is a measurement rather than an
  inference. Used on nine probes.
* `<scratchpad>/rr7` -- a fresh probe directory, `mkdir`ed for this review.
  Oracle wrapper `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib
  .../build/bin/rexx FILE )`, stdout/stderr/status as three separate
  descriptors, absolute paths for every redirect.

Gates, re-established here rather than taken from the report -- RAN, each
exit status read unpiped: `cargo fmt --all --check` **0**;
`cargo clippy --workspace --all-targets -- -D warnings` **0**;
`cargo test --workspace` **exit 0, 964 passed**;
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec` **exit 0, `38 of 38
matching`, mode STRICT**; `cargo test -p rexx-exec --lib` **285 passed**.

---

## Per-finding verdict

| finding | verdict |
|---|---|
| CRITICAL 1 -- pending trap dropped / misdelivered on `RETURN`/`EXIT` | **closed** |
| IMPORTANT 2 -- `active_condition` never cleared | **partially closed** (NEW 1) |
| IMPORTANT 3 -- two implemented behaviours with no test | **closed** |
| IMPORTANT 4 -- `run_source`'s false doc comment | **closed** |
| IMPORTANT 5 -- `RAISE SYNTAX` argument not validated | **partially closed** (NEW 2, NEW 3, NEW 4) |
| MINOR 6 -- `Delivery`'s doc names one writer of three | **closed** |
| MINOR 7 -- `pending_trap`'s doc asserts what finding 1 breaks | **partially closed** (NEW 5) |

---

## CRITICAL 1 -- closed

**Established: RAN**, five ways.

**The measured shapes now match.** The review's own probes, rebuilt from
its text and re-run here: `pa2` (trapping clause is `return bb()`), `pa`
(the same with a later `call cc`), and the report's own third shape `ps`
(the pending activation unwound by an error the caller traps) are all
**MATCH** on stdout, stderr and rc. `ps` on the `f906aabc` worktree prints
`after mark= HANDLER-AT 13` against the oracle's `after mark= NOMARK`, so
the report's account of that shape is confirmed independently, including
that it is the shape a depth cannot close.

**The claimed mutation table reproduces exactly.** RAN, each mutation
applied to the tracked source, `cargo test -p rexx-exec --lib` run, then
restored:

| mutation | result |
|---|---|
| M-PLACEMENT: move the check back below the `match`, verbatim as it was | 283/2 -- kills `a_pending_trap_is_delivered_when_the_trapping_clause_is_a_return` **and** `a_pending_trap_is_not_delivered_into_a_later_activation_at_the_same_depth`; the identity test survives |
| M-IDENTITY: `next_activation_id` returns `activations.len() + 1` -- an id that *is* a stack depth | 284/1 -- kills `a_pending_trap_whose_activation_is_gone_is_never_delivered` and nothing else |

That is precisely the table the report claims, documented survival
included. Two mechanisms, two tests, one each.

**`ActivationId` minting is a property of the type, not of the diff.**
RAN (grep) and REASONED. `Activation` has no `derive`, is never cloned,
and the struct literal appears in exactly two places, both inside
`Activation::new` and `Activation::nested`, both of which now *require* an
`ActivationId` parameter. All seven construction sites in the crate
(`lib.rs`, `run.rs`'s `resolve_and_run_call`, and the five test helpers in
`eval.rs`/`stem.rs`/`plan.rs`/`trace.rs`/`run.rs`) take theirs from
`Interp::next_activation_id`, which is the only producer. The activation
stack is mutated in exactly three places (`push` x2, `pop` x2, no
`insert`/`swap`/`truncate`). So a new construction site cannot compile
without minting an id -- this is the type-level fix, not a convention.
Counter wrap is `u64 += 1` per activation: a debug overflow panic and a
release wrap at 2^64, not reachable. -- REASONED.

**The rooting across the handler is correct, and the check can fail.**
RAN, with a negative control, which is the part that matters.

`root1.rex` puts a heap value on the `Flow::Return` path (`aa: return
bb() || 'TAIL'`, `bb` raising a trapped `USER` condition, the handler
allocating ~60 strings in a loop); `root4.rex` does the same on the
`Flow::Exit` path (`exit sub() || '7'`). Both **MATCH** the oracle, and
both survive `run_program_collect_every_alloc` with real collections (79
and 70) -- so the mode was doing something.

The control: deleting the two-line

```rust
if let Flow::Return(Some(value)) | Flow::Exit(Some(value)) = &flow {
    self.roots.push_temp(*value);
}
```

makes **both** panic under the stress mode -- `root1` at
`value.rs:125:47` and `root4` at `value.rs:221:55`, both `a live value`.
Restored afterwards. So the `push_temp` is load-bearing on both arms and
is genuinely the thing keeping the value alive; the report's claim is not
merely plausible, it is the difference between a pass and a panic.

Two secondary rooting questions, both answered:

* *Does the un-popped temp accumulate?* No, and not measurably. The push
  lands above the enclosing clause's `step_in_temps_frame` watermark, so
  the caller's own clause truncates it. Measured anyway, because the
  reasoning is the kind that has been wrong here: a 200,000-iteration loop
  of the `aa: return bb()` shape peaks at **177,192 kB** on `431e2698` and
  **177,568 kB** on `f906aabc` -- identical, and both explained by the
  never-collecting heap (the same loop with no raise at all is 127,724
  kB). No retention was introduced. -- RAN.
* *`self.activations[len - 2]` in `exec_raise`.* Guarded: that arm is
  reachable only when `caller_trap_for` returned `Some`, and
  `caller_trap_for` starts with `checked_sub(2)?`. -- REASONED.

**What else changed order, checked rather than argued.** The check moved
above `match flow`, so it now also precedes the `Leave`/`Iterate` arms
that `return Err`. Probed: `lv1.rex` (a `call sub` that queues a trapped
condition, then `leave nosuchloop` in the same `DO`) is **MATCH** on
`431e2698` -- handler output then 28.3 at rc 228 -- and on `f906aabc` the
handler output is missing. So the reordering improves that shape too. The
`Goto`/`Signal` arms only assign `pc` on the activation the handler
cannot reach, so their end state is order-independent. -- RAN and
REASONED.

---

## IMPORTANT 2 -- partially closed

**The measured half is closed.** RAN: `pc.rex` is now **MATCH** (98.918,
rc 158, `UH ran` / `resumed` on stdout), and `pf.rex` -- the adjacent
success that says the clearing must *not* be symmetric -- is still
**MATCH** (42.3, rc 214). Both tests exist and both are non-vacuous:
M-CLEAR (drop `self.active_condition = None`) kills
`a_returned_call_handler_leaves_no_active_condition_to_propagate` alone,
and M-SIGNAL-CLEAR (clear on the `SIGNAL` path too) kills
`a_signal_handler_that_runs_on_can_still_propagate` alone (plus the
pre-existing `raise_propagate_re_raises_the_original_condition_and_its_site`).

**Not closed: the clearing is `= None` where the oracle restores.** See
NEW 1.

---

## IMPORTANT 3 -- closed

**Established: RAN.** Both of the review's survivors now die, each to one
test:

| mutation | result |
|---|---|
| M-REARM: `if let Some(trap) = removed { … }` -> `let _ = removed;` | 284/1 -- kills `a_call_trap_is_put_back_after_its_handler_returns` |
| M-REPORTABLE: delete the `if !active.raised.reportable() { … }` guard | 284/1 -- kills `raise_propagate_of_an_unreportable_condition_ends_the_program_silently` |

The second test also asserts no `Error 0` reaches the trace sink, which is
the failure mode the review reached by accident, so it pins the
consequence and not only the guard.

For completeness: all **eight** new tests were killed by a targeted
mutation here (M-PLACEMENT x2, M-IDENTITY, M-CLEAR, M-SIGNAL-CLEAR,
M-REARM, M-REPORTABLE, M-VALIDATE). None of them is a test that cannot
fail. -- RAN.

---

## IMPORTANT 4 -- closed

**Established: RAN** (grep) **and source reading.** `run_source` ->
`run_activated` -> `interp.run_activation()`, and `run_activation` builds
`Code { …, slots: &plan.by_symbol }` (`run.rs:605`). The corrected comment
says exactly that, states that the old text is now false rather than
hedging it, and its supporting claim checks out: `grep -n "slots: &"`
shows `eval.rs`, `stem.rs` and `plan.rs` still passing `&HashMap::new()`
in their own helpers, so the by-name fallback keeps its coverage. Every
sentence in the replacement is true.

---

## IMPORTANT 5 -- partially closed

**The rule that was inferred is right as far as it goes, and its
exception clause is structural rather than fitted.** RAN, two ways.

*By probe:* all **21** rows of the report's own table re-run here against
the oracle -- `40.4 40.5 40.912 40 98.941 40.10 40.999 999 'abc' 0 0.5 100
3.1 10.1 26.1 88.1 99.5 40.001 2 1.1 2.1` -- are **MATCH**, condition and
rc. All three global-constraint violations the review named (rc 216, rc
25, rc 0) are gone.

*By source:* the C++ is `Interpreter::messageNumber`
(`/home/moritz/dev/repos/ooRexx/interpreter/runtime/Interpreter.cpp:678-708`),
and it confirms the inferred rule *and* explains the exception clause the
review flagged as fitted to two rows. `createExceptionObject`
(`interpreter/concurrency/Activity.cpp:1017`) raises `98.941` with a
**dot-formatted** substitution (`snprintf(work, "%d.%1zd", errcode/1000,
errcode - primary)`) when the *primary* message is missing, while
`buildMessage` (`:1229`) raises the same error with the **integer**
`major*1000+sub` when only the *secondary* is missing. Two call sites,
two substitution forms -- so "the major itself is unknown" and "the major
is known but this sub is not" really are two cases in the oracle's own
code, and `raise_syntax_condition`'s `lookup(major, 0).is_some()` branch
is the right shape rather than a curve fit.

**Not closed:** the C++ rule has two clauses the fix does not implement
(NEW 2, NEW 4), and the doc comment states a false fact about the
catalogue (NEW 3).

---

## MINOR 6 -- closed

**Established: RAN** (grep, then attributing each line to its enclosing
`fn`). Exactly seven writes of `delivery.search`/`delivery.positionless`,
in exactly three functions -- `offer_to_trap` (`run.rs:2422`), `exec_raise`
(`:2760, :2773, :2813, :2822`) and `exec_raise_propagate` (`:2870, :2871`).
The replacement comment names all three, describes each correctly, and
singles out `offer_to_trap`'s as the one that mutates a `Delivery` already
in flight, which is true.

---

## MINOR 7 -- partially closed

The sentence the review named ("`run_activation`'s check runs in every
activation", asserting the guarantee finding 1 broke) is gone, and the two
properties the replacement states -- placement before the `Flow` dispatch,
and delivery keyed on an identity -- are both established and both
mutation-pinned above.

But the replacement asserts more than holds: "`run_activation`'s check
runs once per *clause*, on every path out of one". A clause inside a `DO`
body, a `WHEN` body or an `INTERPRET` fragment reaches no such check --
see NEW 5, where three oracle DIFFs falsify it. So the doc comment is
still, in a new way, describing a property the code does not have.

---

# New findings

## NEW 1 (IMPORTANT, RAN) -- clearing `active_condition` to `None` instead of restoring the previous one wipes an enclosing handler's condition

`deliver_pending_trap`'s `Ended::Returned` arm sets `self.active_condition
= None`. Where a condition was already active, that is one clearing too
many: the oracle keeps it.

`ac1.rex` -- one clause both queues a `CALL ON USER` condition and raises
a `SIGNAL ON SYNTAX`-trapped failure; the `SIGNAL` handler ends in `raise
propagate`:

```rexx
 1  signal on syntax name sh
 2  call on user foo name uh
 3  zq = sub() + 1/0
 …
11  sh:
12  say 'SH ran' sigl
13  raise propagate
```

| | stdout | stderr | rc |
|---|---|---|---|
| oracle | `UH ran 3` / `SH ran 3` | `3 *-* zq = sub() + 1/0` + `Error 42` / `Error 42.3` | **214** |
| `431e2698` | `UH ran 3` / `SH ran 3` | `13 *-* raise propagate` + `Error 98.918` | **158** |
| `f906aabc` | `UH ran 3` / `SH ran 3` | *(empty)* | **0** |

`ac2.rex` is the plainer shape -- a `SIGNAL ON SYNTAX` handler that calls
a routine raising a `CALL ON`-trapped `USER` condition, then propagates --
and splits identically: oracle 42.3 rc 214, ours 98.918 rc 158, pre-fix
silence at rc 0.

Both interpreters agree on the delivery *order* in `ac1` (`UH` then `SH`),
so the divergence isolates to the `raise propagate` and to nothing else.

This is not a regression against the oracle -- `f906aabc` was wrong here
too, differently -- but it is the mechanism question the fix was asked to
answer, and the answer chosen is measurably not the oracle's. Saving
`active_condition` before `resolve_and_run_call` and restoring it in the
`Returned` arm gives the oracle's answer in **all three** measured shapes:
`pc.rex` (nothing was active, restore `None`, 98.918 -- unchanged), `ac1`
and `ac2` (the `SYNTAX` condition comes back with its own site, 42.3).

It also makes one sentence of the new `exec_raise_propagate` doc false:
"a condition stays active for as long as its `SIGNAL` handler's activation
does". `ac2` is that activation, and the condition does not survive a
`CALL` handler returning inside it.

## NEW 2 (IMPORTANT, RAN) -- the sub is not bounded at 1000, and the doc comment asserts the opposite of the oracle for exactly that input

The C++ rejects a secondary outside `0 <= sub < 1000` with the same
`Error_Expression_result_raise` it uses for a bad major
(`Interpreter.cpp:702-706`). `raise_syntax_condition` has no such check --
it widens the sub to `u32` and lets any unknown pair fall through to
`98.941`.

| program | oracle | `431e2698` | |
|---|---|---|---|
| `raise syntax 40.1000` | `Error 33.904`, rc **223** | `Error 98.941 … found "41000"`, rc **158** | DIFF |
| `raise syntax 40.1001` | `Error 33.904`, rc 223 | `98.941 … found "41001"`, rc 158 | DIFF |
| `raise syntax 40.99999` | `Error 33.904`, rc 223 | `98.941 … found "139999"`, rc 158 | DIFF |
| `raise syntax 40.999` | `98.941 … found "40999"` | identical | MATCH (the boundary below) |

`40.1000` needs no quoting and no computed expression -- it is a plain
numeric literal.

The `u32` widening's own comment states the opposite of what the oracle
does, naming this very input:

> `u32` rather than `u16` for the sub: `raise syntax 40.99999` must reach
> the "unknown code" answer rather than wrap into a code that happens to
> exist

The oracle answers `33.904` for `40.99999`. Per `rust/CLAUDE.md` that
comment must be corrected, not hedged; `raise_syntax_validates_its_
argument` should gain the `40.999` / `40.1000` pair, since the boundary is
what pins the rule.

## NEW 3 (MINOR, RAN) -- "majors 1 and 2 are the only ones with no `(major, 0)` entry" is false; there are 45

`raise_syntax_condition`'s doc comment:

> majors 1 and 2 are the only ones in 1..=99 with no `(major, 0)` entry in
> the generated catalogue, confirmed by looking them up

and the same claim inside `raise_syntax_validates_its_argument`
("Majors 1 and 2 are the only ones in range with no `(major, 0)` catalogue
entry"). Counted from the generated `errors.rs`, the majors in 1..=99 with
no `(major, 0)` entry are **1, 2, 12, 32, 50-87, 94, 95, 96** -- 45 of
them.

The *code* is right, because it performs the lookup instead of hard-coding
the pair, and the oracle agrees on the ones I probed: `raise syntax 50.1`,
`12.1`, `32.5`, `96.2` and `87` are all **MATCH**, all rendering the dot
form (`"50.1"`, `"87.0"`, …). Only the two comments are false, and the
report's own text repeats the claim. Correct them, or drop the count and
keep the mechanism sentence, which is the true one.

## NEW 4 (MINOR, RAN) -- the argument is parsed with Rust integer parsing, not Rexx `numberValue`

The C++ runs each half through `RexxString::numberValue`, which accepts
the full Rexx number syntax, and requires the part *after* a decimal point
to be non-empty and parseable.

| program | oracle | `431e2698` | |
|---|---|---|---|
| `raise syntax '4E1'` | `Error 40 …: Incorrect call to routine`, rc **216** | `Error 33.904`, rc **223** | DIFF |
| `raise syntax '40.1E2'` | `Error 98.941 … found "40100"`, rc 158 | `Error 33.904`, rc 223 | DIFF |
| `raise syntax '40.'` | `Error 33.904`, rc 223 | `Error 40 …`, rc **216** | DIFF |
| `raise syntax '+40'` | `Error 40 …`, rc 216 | identical | MATCH |
| `raise syntax 40.0` | `Error 40 …`, rc 216 | identical | MATCH |

Minor because all three DIFFs need a quoted argument, and none produces an
`Error 0` or a successful exit; but `'40.'` in particular is the same
class the fix was written to close -- a plausible Rexx condition at a
plausible rc for an argument the oracle rejects outright.

## NEW 5 (IMPORTANT, RAN) -- "there is now no path at all between the clause finishing and this check" is false: a clause inside `DO`/`SELECT`/`INTERPRET` never reaches it

The fix's central comment in `run_activation` claims:

> there is now no path at all between "the clause finished" and this
> check, so a third `Flow` arm that returns cannot reintroduce it

and `Interp::pending_trap`'s replacement doc says the check "runs once per
*clause*, on every path out of one". `run_bounded` -- which steps every
clause in a `DO` body, a `WHEN`/`THEN` body and an `INTERPRET` fragment --
has no such check. Delivery is deferred to the boundary of the *enclosing
instruction*.

| probe | oracle | `431e2698` | |
|---|---|---|---|
| `do2` (`call sub` then `say` inside a 2-iteration `DO`) | `after call 1 mark= HANDLER-AT 4` / `after call 2 mark= HANDLER-AT 4` / `end mark= HANDLER-AT 4` | `after call 1 mark= NOMARK` / `after call 2 mark= NOMARK` / `end mark= HANDLER-AT 5` | DIFF |
| `sel1` (the same inside a `SELECT`/`WHEN … DO`) | `in when mark= HANDLER-AT 5` / `end mark= HANDLER-AT 5` | `in when mark= NOMARK` / `end mark= HANDLER-AT 6` | DIFF |
| `int1` (`interpret "call sub; say … "`) | `inside mark= HANDLER-AT 3` / `end …` | `inside mark= NOMARK` / `end mark= HANDLER-AT 3` | DIFF |

Both the timing and, in `do2`/`sel1`, the resulting `SIGL` are wrong.

**The behaviour is pre-existing**: all three are byte-identical on the
`f906aabc` worktree. What is new is the comment asserting the property.
The neighbouring shapes the fix *did* close are real -- `do1` (`return
bb()` inside a `DO`) goes from `NOMARK` to **MATCH**, and `do3` (an `IF`
whose whole branch is the raising call) matches because the enclosing
instruction ends at the same point -- which is exactly why the general
claim reads as established when it is not.

Either correct the two comments to say what is true (the check runs once
per instruction stepped by `run_activation`'s own loop, which is not the
same as once per Rexx clause), or move the check into `run_bounded`'s loop
and take the three DIFFs with it. The second is a behaviour change and
belongs to a task, not to a comment fix; the first is required by the
comment rule either way.

---

## What I checked and found correct

Recorded because a clean result is part of the answer.

* **Gates.** RAN, exit statuses read unpiped: fmt 0, clippy 0,
  `cargo test --workspace` exit 0 / 964 passed, corpus gate exit 0 /
  `38 of 38` / STRICT, `-p rexx-exec --lib` 285 passed. The controller's
  baseline reproduces exactly.
* **No `unsafe` was introduced.** The workspace still sets
  `unsafe_code = "forbid"` and clippy is clean; the diff adds no
  allocation site of its own (`push_temp` is a root push, not an
  allocation).
* **All eight new tests can fail**, each to a targeted mutation, listed
  under finding 3.
* **The report's account of probe `ps` is accurate**, including the
  pre-fix `HANDLER-AT 13`, verified against the `f906aabc` worktree rather
  than taken from the text.
* **The finding-5 exception clause is not a curve fit**, confirmed in the
  oracle's own C++ at two distinct call sites (finding 5 above).
