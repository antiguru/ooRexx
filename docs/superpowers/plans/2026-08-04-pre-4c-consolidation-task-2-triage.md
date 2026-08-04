# Task 2 triage table: boundary prose in `rust/crates/`

Plan: `docs/superpowers/plans/2026-08-04-pre-4c-consolidation.md`, Task 2.
Base: `f9cfb060` (Task 1 complete).

## The search, re-derived

```
grep -rnE '^\s*(//|/\*|\*)' --include='*.rs' rust/crates/ |
  grep -Ei '\b(4a|4b|4c|Phase 5)\b|\btoday\b|\bcurrently\b|\bnot yet\b|\bso far\b|\bfor now\b|\bas of\b'
```

This is the search that produced the gate's figure: it returns **522** at `e96f3435`,
the commit whose `rust/CLAUDE.md` records that number. At `f9cfb060` it returns **476**
-- Task 1 removed 46 (`337217da` 41, `f9cfb060` 5).

Per file, at `f9cfb060`:

| File | Hits | Plan's figure |
|---|---:|---:|
| `rexx-exec/src/run.rs` | 82 | 82 |
| `rexx-exec/src/lib.rs` | 71 | 73 |
| `rexx-exec/tests/coverage.rs` | 38 | 42 |
| `rexx-exec/tests/trace_oracle.rs` | 24 | 24 |
| `rexx-exec/tests/owners.rs` | 22 | 48 |
| `rexx-exec/tests/loud.rs` | 20 | 34 |
| **the six** | **257** | 263 |
| 38 other files | 219 | -- |
| **total** | **476** | 522 |

The four files whose count moved are the four Task 1 edited.

## Categories

* **D** -- delete. History ("Task N implemented X, so the row is gone"), or a status
  fact nothing needs.
* **C** -- convert. The sentence is trying to say something true regardless of where
  the boundary sits; the replacement is given.
* **K** -- keep, byte for byte. Three kinds qualify:
  * the word is about **run-time state**, not the phase boundary -- "the frame
    currently executing", "the DIGITS currently in force", "not yet asked". These were
    never the target and the search cannot tell them apart from prose that was;
  * it **cites something outside this repository** -- a C++ file and line, an oracle
    transcript, a measured figure against a pinned upstream commit;
  * it **names a committed artefact** whose name contains a phase: `phase-4a.txt`,
    `phase-4b.txt`, `mutate-4a.sh`, `2026-07-30-phase-4a-executor-design.md`. The file
    is the referent, not the boundary.

A line's category is **what happens to that line in the diff**: K means byte-identical.

## Scope of the edits

The plan's Task 2 names six files under **Files:**, in the order it gives them, one
commit each. That is what is edited here. The other 38 files' 219 lines are triaged in
the last section at file grain and are **not** edited -- see the report for the count of
real targets among them and the recommendation.

---

## `rexx-exec/src/run.rs` (82)

| Line | Cat | Treatment |
|---|:-:|---|
| 55 | C | "4b's Task 3 is the first to make the **activation stack** real, and it is `run_activation` that changes shape rather than only gaining arms: it reads" -> "`run_activation` is the activation stack's own driver, and it is shaped by having more than one activation to run: it reads" |
| 187 | D | "Live since 4b's Task 6." -- status fact, nothing needs it. |
| 191 | K | "the body currently being stepped" -- run-time state, quoting `run_bounded`'s contract. |
| 222 | K | "a range `run_bounded` is currently absorbing a `Goto` into" -- run-time state. |
| 239 | D | "`run_activation` had a bare `Option<ObjRef>` through 4a, when every activation was the program's only one and every way out of it stopped the program." -- history; the next sentence already states the property. |
| 305 | C | "(and, from Phase 5, dispatch)" -> "(and, once dispatch exists, dispatch)" |
| 356 | C | "and since 4b's Task 2 it passes its own fragment source, so" -> "and it passes its own fragment source, so" |
| 757 | C | "**Both lines are produced now, and 4b's Task 2 is what closed it**" -> "**Both lines are produced.**" |
| 758 | D | "-- through 4a and 4b's Task 1 this reproduced only the second." -- history. |
| 776 | C | "The condition-trap boundary (4b's Task 7), and the reason" -> "The condition-trap boundary, and the reason" |
| 893 | C | "**Every caller passes `Some` since 4b's Task 2**" -> "**Every caller passes `Some`**" |
| 1103 | C | "run it against **this** activation. 4a built the machinery (`run_fragment`) and 4b's Task 1 is the keyword; the arm used to" -> "run it against **this** activation, through `run_fragment`. The arm is thin because `run_fragment` does the work." |
| 1104 | D | "be gated on an `interpret_spike` flag that only a test entry point set, and deleting that flag is the whole of the keyword's implementation, because `run_fragment` already did the work." -- history. |
| 1145 | C | "and it took the whole of 4b's Task 2 rather than the one-line change it looks like. Handing `run_fragment` a `Some(&fragment.source)` and nothing else was built twice and measured wrong twice, for two independent reasons:" -> "Handing `run_fragment` a `Some(&fragment.source)` and nothing else is not enough, for two independent reasons:" |
| 1198 | C | "That is a 4a divergence with nothing to do with fragments" -> "That is a divergence with nothing to do with fragments" |
| 1733 | C | **False as written** -- `Call::Trap` is in scope (`owners.rs`'s `INSTRUCTION_TAGS`), so two arms do not stay loud. "The other two arms of `rexx_parse::Call` stay loud and keep their own owners (`instruction_owner`, `lib.rs`): `Trap` (`CALL ON`/`CALL OFF`) is Task 7's, `Qualified` (`CALL ns:name`) is Phase 5's." -> "The one arm of `rexx_parse::Call` that stays loud keeps its own owner (`instruction_owner`, `lib.rs`): `Qualified` (`CALL ns:name`) is Phase 5's." |
| 1870 | C | "`PUSH`/`QUEUE line` (4b's Task 8, I15)." -> "`PUSH`/`QUEUE line` (I15)." |
| 1879 | C | "why nothing here reads a line back: `PULL`, `PARSE PULL` and `QUEUED()` are all 4c's." -> "why nothing here reads a line back: reading one back is `PULL`/`PARSE PULL`/`QUEUED()`'s, not this arm's." |
| 1911 | C | "The 4a invariant `grow_slots` asserts is therefore untouched by this task:" -> "The invariant `grow_slots` asserts is therefore untouched:" |
| 2072 | C | "the 99.910 arm is unreached today.**" -> "the 99.910 arm is unreached.**" (the paragraph's own eight measured shapes are what support it) |
| 2188 | K | "a target that is not currently unset is 98.995" -- run-time state of the variable. |
| 2235 | K | "The target must be **currently unset**." -- run-time state. |
| 2411 | K | "a `PROCEDURE`d callee that has not yet transferred control itself" -- run-time state. |
| 2487 | C | "rather than have 4b retrofit a raise into it" -> "rather than leave a raise to be retrofitted into it" |
| 2745 | C | "So this line changes no output today." -> "So this line changes no output while that holds." |
| 3178 | C | "and 4b builds only the front of it:" -> "and this crate builds only the front of it:" |
| 3179 | K | "fails loudly naming `4c`, which owns the builtin table" -- names the emitted message's owner string, which is the frozen contract `loud.rs` pins with `ends_with`. |
| 3188 | C | "What stops 4b dispatching is the step in front:" -> "What stops this crate dispatching is the step in front:" |
| 3191 | C | "and without 4c's table this arm would silently run the wrong routine" -> "and without the builtin table this arm would silently run the wrong routine" |
| 3229 | C | **False as written** -- `USE ARG` is implemented (`owners.rs`: `Use` is `InScope`; this file's own next paragraph says "Task 5 keeps them"). "Unobservable in this task except through failure -- `USE ARG` (Task 4) and `ARG()` (4c) are what read an argument, and both are still loud -- but the failure is real and measured:" -> "Observable through failure, and the failure is real and measured:" |
| 3635 | D | "-- corrected at 4b's Task 7, whose own brief flagged it, because a task told to clear that field would otherwise go looking in the callers and find nothing." Deleted with its lead-in at 3633-3634, "**This paragraph said ... and was wrong about the expression and about where it lives**": an account of what the comment used to say. The corrected statement is the paragraph above it. |
| 3640 | C | "(4b's Task 2). The guard is unchanged; what changed is that `run_fragment` calls" -> "`run_fragment` calls" |
| 3717 | C | "The debug tripwire I22 scheduled in 4a and left unbuilt, added" -> "The debug tripwire I22 asks for, alongside" |
| 3718 | C | "here by 4b's Task 1 along with `RootSet::temps_len`, its one prerequisite." -> "`RootSet::temps_len`, its one prerequisite." |
| 3781 | D | "Corrected at 4b's Task 7, the task that had to clear the field." -- history; with "-- this line used to say it was both" at 3779, which is an account of the comment's own past. |
| 4107 | C | **False as written** -- condition trapping exists (`Signal::Trap` is in scope), so "a raise in 4a is always fatal (no `SIGNAL ON`/condition trapping exists yet)" no longer holds. Not probeable from here: the absorbed-`WHEN` false path that sets this offset is a documented deliberate deviation (SF #2018 segfaults the oracle one line away, `run.rs`'s own note at 1594-1604), so no program reaches `run_otherwise` under a trap. Narrowed to what is true: "a raise in 4a is always fatal (no `SIGNAL ON`/condition trapping exists yet), so nothing runs" -> "a raise that is not trapped is fatal, so nothing runs". Flagged in the report. |
| 4167 | C | "Task 11's own fourteen-point probe fixed that behaviour and this fix leaves it exactly as it was, because" -> "The fourteen-point probe behind this rule leaves it exactly as it was, because" (keeps the measurement, drops the task) |
| 4186 | K | "any escape elevation currently in force" -- run-time state. |
| 4189 | C | "Through 4a the six sites that needed `+ self.indent_offset` each wrote it out" -> "The six sites that needed `+ self.indent_offset` each wrote it out" |
| 4193 | C | "**That was a live 4a divergence, not one 4b created.**" -> "**The divergence does not need a fragment.**" |
| 4198 | C | "What 4b's Task 2 changed was only how easy it is to reach -- a plain `SELECT`" -> "A plain `SELECT`" |
| 4256 | C | "That covers `Flow::Exit` today and is written to keep covering whatever Task 11 adds for `LEAVE`/`ITERATE`:" -> "That covers `Flow::Exit`, and is written to keep covering `LEAVE`/`ITERATE` too:" |
| 4304 | C | "which nothing in 4a answers (no message dispatch at all yet)" -> "which nothing in this crate answers (no message dispatch at all)" |
| 4807 | C | "**This clause's boundary is currently unobservable" -> "**This clause's boundary is unobservable on every probe tried" (the 38 probes are named two lines down) |
| 5281 | C | "Measured, and **owned by 4c** since 4b's Task 12 ruled on it" -> "Measured, and **owned by 4c**" |
| 5283 | D | "; it was a KNOWN GAP, in two separate rows, until that ruling merged them and assigned the owner" -- history. The `phase-4-exclusions.txt` citation beside it stays. |
| 5502 | C | "# The outward direction, measured (4b Task 1)" -> "# The outward direction, measured" |
| 5552 | C | "Both are produced since 4b's Task 2, where 4a and 4b's Task 1 produced the second alone." -> "Both are produced." |
| 5590 | C | "It is an `Rc` because that is the shape 4b needs, where an `INTERPRET`" -> "It is an `Rc` because an `INTERPRET`" |
| 5605 | C | "**`Some(&fragment.source)`, where 4a passed `None`.** The fragment resolves its own clauses now:" -> "**`Some(&fragment.source)`.** The fragment resolves its own clauses:" |
| 5671 | C | **False as written** -- `resolve_and_run_call` also seals (`run.rs:3450`). "Today that is `run_fragment` alone; Task 3's `CALL` is the next, and the rule for it is the same --" -> "That is `run_fragment` and `resolve_and_run_call`, and the rule is the same for both:" |
| 5680 | C | "`offer_to_trap`'s (4b's Task 7, inherited item I11)" -> "`offer_to_trap`'s (inherited item I11)" |
| 5683 | D | "Through 4a a raise was always fatal, so a stale stack could not be observed;" -- history. The transcript that observes it, named next, stays. |
| 5809 | C | "4a has no `::OPTIONS` to move the package default away from `Scientific`" -> "This crate has no `::OPTIONS` to move the package default away from `Scientific`" |
| 5999 | C | "why this is a method and not the free function it was through 4a.**" -> "why this is a method and not a free function.**" |
| 6010 | C | "passes `Some(&fragment.source)` since 4b's Task 2 gave the report an echo per level." -> "passes `Some(&fragment.source)`, which is what gives the report an echo per level." |
| 6074 | K | "which iteration of an enclosing loop is currently running" -- run-time state. |
| 6119 | C | "That row once said they did; 4b Task 9 closed the value lines alone, by moving the `BY` increment into `loop_advance`, and nothing about this indent changed." -> "Closing the value lines -- the `BY` increment in `loop_advance` -- does not close this indent." |
| 6121 | C | "It is 4a's, not this function's to fix under any task" -> "It is not this function's to fix under any task" |
| 6122 | C | "that has run so far. **What matters here" -> "**What matters here" |
| 6132 | C | "spot that hid two of 4b Task 2's own four mutations." -> "spot that hid two of four mutations in one round." |
| 6531 | D | "like every other raiser in this file since 4b's Task 7." -- a claim about what every other site does, which `rust/CLAUDE.md` names as the forbidden shape. |
| 6535 | C | "**Three outcomes, all measured, and 4a never had to know about two of them because `RAISE` is the first construct that lets a program name an arbitrary error number.**" -> "**Three outcomes, all measured.** Two of them arise only because `RAISE` lets a program name an arbitrary error number." |
| 6790 | C | "through `Interp::run_activation` itself since 4b's Task 7, not through a miniature of it." -> "through `Interp::run_activation` itself, not through a miniature of it." |
| 6816 | C | "**`run_activation` itself since 4b's Task 7, not a miniature of it.** This used to be a hand-rolled `run_bounded` loop that reproduced `run_activation`'s own `Flow` dispatch arm by arm, and it drifted exactly the way a second copy does: Task 6 had to teach it about `Flow::Signal`, and Task 7's condition traps -- which live in" -> "**`run_activation` itself, not a miniature of it.** A hand-rolled `run_bounded` loop here would reproduce `run_activation`'s own `Flow` dispatch arm by arm, and a second copy drifts: it needs teaching about every new `Flow` variant, and about condition traps, which live in" (keeps the reason, drops the account) |
| 8215 | C | "`InstructionKind::Do`, 4a's own -- `lib.rs`'s `owned_message`" -> "`InstructionKind::Do`, which this crate implements -- `lib.rs`'s `owned_message`" |
| 8216 | K | "(there is none to blame; `DO WITH` is Phase 5's *reason*, but the message names the construct, not the reason)" -- the distinction is the point of the test and does not move. |
| 8218 | C | "Mutation-kill for deleting the `\"4a\"`/`None` carve-out in `owned_message`" -> "Mutation-kill for deleting the `None` carve-out in `owned_message`" (the `"4a"` spelling is a literal `owned_message` no longer keys on) |
| 8364 | C | "`DO`/`LOOP` is 4a's own regardless of which deviation" -> "`DO`/`LOOP` is implemented regardless of which deviation" |
| 9091 | D | "**The last line is 4b Task 2's half, and it is asserted now.** Task 1 landed the `>>>` and left this expectation one line short of the oracle on purpose, because `run_fragment` passed `source: None` and `step_in_temps_frame` had no clause site to echo." -- history. The transcript above it and "The whole transcript is compared byte for byte below" carry the contract. |
| 9471 | C | "at the level `run_activation` hardcoded through 4a: the callee runs" -> ": the callee runs" |
| 9802 | K | "see `phase-4b.txt`'s own entry for `lang/call_expression.rex`" -- names a committed artefact. |
| 9993 | C | "4b's answer is the loud builtin/external fallback naming 4c;" -> "This crate's answer is the loud builtin/external fallback naming `4c`;" |
| 9994 | C | "and so not this phase's to make." -> "and so not this crate's to make." |
| 10195 | K | "whose own currently-executing instruction is the `INTERPRET` itself" -- run-time state, and it is the oracle's `signalTo` being cited. |
| 10251 | C | "two further `>>>` lines this crate does not yet reproduce (the documented \"KNOWN GAP\" at `loop_advance`'s own `Controlled` arm, unrelated to `SIGNAL` and out of this task's scope)" -> "two further `>>>` lines this crate does not reproduce (the documented \"KNOWN GAP\" at `loop_advance`'s own `Controlled` arm, unrelated to `SIGNAL`)" |
| 10330 | C | "taking `CALL`'s own loud 4c fallback." -> "taking `CALL`'s own loud unresolved-call fallback." |
| 10602 | C | "// (4b Task 5) ----" -> "// ----" (folded into the section header on 10601) |
| 10681 | C | "`DROP (v)` took the identical correction in 4a." -> "`DROP (v)` has the identical shape." |
| 11131 | K | "requires its target to be **currently unset**" -- run-time state. |
| 11568 | C | "// ---- 4b Task 7: condition traps, RAISE and NOVALUE ----" -> "// ---- condition traps, RAISE and NOVALUE ----" |
| 12005 | C | "4a concluded that `SIGNAL ON SYNTAX` cannot accumulate a temps leak" -> "I16 concluded that `SIGNAL ON SYNTAX` cannot accumulate a temps leak"; and "This is the first task where a trap actually acts, so the conclusion is measured here:" -> "The conclusion is measured here:" |

Totals: **D 10, C 60, K 12** (82).

---

## `rexx-exec/src/lib.rs` (71)

| Line | Cat | Treatment |
|---|:-:|---|
| 12 | C | "The Phase 4a executor, at the size Task 3's borrow-shape spike needs it." -> "The executor." (the sentence's remainder, "This crate is a spike, and the thing it exists to prove is one sentence from the design's \"The borrow shape\"", already says what it is) |
| 27 | K | "any construct not yet implemented fails loudly with `NOT_IMPLEMENTED_EXIT` rather than silently, which is a gate criterion" -- this is the property the paragraph above it was written to replace an enumeration with, and it is asserted by `loud.rs`. Deleting it would undo the fix this task is modelled on. |
| 55 | C | "The in-process external data queue (I15) `PUSH`/`QUEUE` write to (4b's Task 8)." -> "The in-process external data queue (I15) `PUSH`/`QUEUE` write to." |
| 56 | C | "Nothing reads it yet -- `PULL`/`PARSE PULL`/`QUEUED()` are 4c's." -> "Reading it back is `PULL`/`PARSE PULL`/`QUEUED()`'s, none of which this crate has." |
| 66 | K | "everything about the frame currently executing (D16)" -- run-time state. |
| 97 | C | "The exit code for a construct Phase 4a does not implement." -> "The exit code for a construct this crate does not implement." |
| 277 | C | "it is what the two-sided bound above is checked against today, and it needed" -> "it is what the two-sided bound above is checked against, and it needed" |
| 301 | C | "Nothing in 4a's corpus does that, and Phase 7's stream model replaces this whole shape." -> "Nothing in the corpus does that, and a streaming model would replace this whole shape." |
| 372 | C | "A construct 4a does not implement, on its way to becoming an exit code" -> "A construct this crate does not implement, on its way to becoming an exit code" |
| 385 | C | "An instruction 4a does not execute." -> "An instruction this crate does not execute." |
| 454 | C | "An expression form 4a does not evaluate." -> "An expression form this crate does not evaluate." |
| 480 | K | "the next steps are the builtin table and then external resolution, and **both are 4c's**" -- the owner named here is the one the message on the next line emits, and `loud.rs` pins that message with an `ends_with`. |
| 483 | K | "`\"routine \\\"NAME\\\" is not implemented (4c)\"`" -- quotes the frozen message text verbatim; the sentence says so. |
| 525 | C | "and 4b's exposure mechanism aliases whole slots" -> "and this crate's exposure mechanism aliases whole slots" |
| 530 | K | "unlike `unresolved_call`'s `4c`, the steps behind this are not another phase's to build -- nothing has been scheduled to build them" -- states why this message carries no owner at all, which is the decision the code makes two lines down. |
| 547 | C | "Unreachable through any program today, since nothing constructs a `Some(index)` selector at all" -> "Unreachable through any program, since nothing constructs a `Some(index)` selector at all" |
| 566 | D | "4b's Task 2 built it -- `impl From<&ParseError> for Raised` in `error.rs`, which `run_fragment` now uses." Deleted with the block it heads (561-569), "**There is no `Loud::parse` any more, and its absence is the fix.** A fragment that did not parse used to become ...": an account of a deleted item. `execute`'s own parse arm, which the block points at, carries the live statement. |
| 575 | C | "this is called on nodes 4a cannot evaluate" -> "this is called on nodes this crate cannot evaluate" |
| 624 | K | "An earlier version keyed this on the literal `\"4a\"`, which doubles a phase *name* as a sentinel for a different property" -- the whole paragraph is the argument for `Option<&'static str>` over a sentinel string, and the sentinel it rejects has to be named to be rejected. It ends "so it survives every later phase unchanged", which is the property. |
| 627 | K | Same paragraph. |
| 628 | K | Same paragraph. |
| 629 | K | Same paragraph. |
| 638 | C | "where the outer `InstructionKind`/`ExprKind` is 4a's own but the specific reason" -> "where the outer `InstructionKind`/`ExprKind` is implemented but the specific reason" |
| 656 | C | "Who is responsible for an `InstructionKind` that is not (yet) 4a's own," -> "Who is responsible for an `InstructionKind` this crate does not implement," |
| 658 | K | "`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, \"The split\"" -- names a committed artefact. |
| 659 | C | "-- `None` for a variant 4a already implements (see" -> "-- `None` for a variant this crate implements (see" |
| 660 | K | "[`owned_message`]'s doc for why that is `None` and not a `\"4a\"` string" -- points at the decision kept at 624. |
| 686 | D | "`InstructionKind::Signal` was the second arm-grained one until 4b's Task 7 implemented `Signal::Trap`; all three of its arms answer `None` now, so it needs no nested match either." -- history. The live statement, "Every other variant here stays coarse", is the line above. |
| 714 | D | "In scope since 4b's Task 1: the fragment machinery was 4a's and the keyword is this task's, so `Interpret` is `None` here (implemented in this crate) rather than `Some(\"4b\")`." -- history on a `None` arm. Task 1's assertion now checks this arm against `owners.rs` per row. |
| 716 | D | Same comment. |
| 725 | C | "`Call::Named` and `Call::Dynamic` are implemented (Task 3) and `Call::Trap` is (Task 7), so \"4b\" would be a false statement in a table whose only job is to be true" -> "`Call::Named`, `Call::Dynamic` and `Call::Trap` are all implemented, so any owner string here would be a false statement in a table whose only job is to be true" |
| 730 | K | "through `Loud::unresolved_call`, naming `4c`: the builtin and external steps behind the label search are that phase's" -- names the emitted message's owner, the frozen contract. |
| 739 | C | "In scope since 4b's Task 5, both of them. `Use` is `None` even" -> "`Use` is `None` even" |
| 748 | C | "In scope since 4b's Task 7. All three `Signal` arms are implemented (`Label`/`Value` at Task 6, `Trap` here), so unlike" -> "All three `Signal` arms are implemented, so unlike" |
| 752 | K | "does so through `ExprKind::List`'s own `Phase 5` owner" -- names the owner the emitted message carries, and `owners.rs` asserts that row. |
| 756 | C | "In scope since 4b's Task 8. Both keywords are whole:" -> "Both keywords are whole:" |
| 789 | C | "**`None` since Task 4, not `Some(\"4b\")`.** `ExprKind::Call` has" -> "**`ExprKind::Call` is `None`, not an owner string.** It has" |
| 790 | C | "exactly two `CallTarget` forms and both are 4b's own (the target field is checked, per this task's own brief) -- unlike" -> "exactly two `CallTarget` forms and this crate evaluates both -- unlike" |
| 798 | K | "through `Loud::unresolved_call`, naming `4c`" -- the frozen message's owner. |
| 838 | C | "`SIGNAL ON NOVALUE` in 4b changes what an uninitialised read does" -> "`SIGNAL ON NOVALUE` changes what an uninitialised read does" |
| 840 | D | "Through 4a and most of 4b the flag was read and discarded, which was the correct amount of nothing;" -- history. |
| 842 | C | "is the reader D16 was holding it for, added at 4b's Task 7, and the retrofit" -> "is the reader D16 was holding it for, and the retrofit" |
| 857 | C | "kept so `RAISE PROPAGATE` can re-raise it (4b's Task 7)." -> "kept so `RAISE PROPAGATE` can re-raise it." |
| 883 | C | "handler runs (4b's Task 7)." -> "handler runs." |
| 956 | C | "(4b's Task 7)." -> "" (the doc's first line already names the field) |
| 1091 | C | "4b's Task 2 was told to carry an `INTERPRET` fragment's activation base here as well, on the reasoning that both are" -> "Carrying an `INTERPRET` fragment's activation base here as well is the mistake it invites, on the reasoning that both are" |
| 1130 | C | "**`0` throughout every 4a program**, which is what makes adding it at" -> "**`0` for every program with no fragment and no call**, which is what makes adding it at" |
| 1131 | C | "a site incapable of moving a 4a expectation: nothing but a fragment" -> "a site incapable of moving such a program's expectation: nothing but a fragment" |
| 1169 | C | "**First-wins *within one level*, and 4b's Task 2 left that unchanged on purpose (inherited item I11).**" -> "**First-wins *within one level* (inherited item I11).**" |
| 1185 | K | "`failure_site` is the level currently unwinding" -- run-time state. |
| 1187 | C | "and is called by exactly the constructs that open a level -- today `run_fragment` and (since Task 3) `resolve_and_run_call`." -> "and is called by exactly the constructs that open a level -- `run_fragment` and `resolve_and_run_call`." |
| 1223 | K | "Task 16's collect-on-every-allocation gate criterion (4a exit gate, criterion 4)" -- names a committed gate document's criterion. |
| 1240 | K | "The depth-1 address of the chain currently being evaluated" -- run-time state. |
| 1292 | C | "The in-process external data queue (I15, 4b's Task 8): every line" -> "The in-process external data queue (I15): every line" |
| 1295 | C | "and why nothing here reads it back yet -- `PULL`, `PARSE PULL` and `QUEUED()` are all 4c's." -> "and why nothing here reads it back -- that is `PULL`/`PARSE PULL`/`QUEUED()`'s, none of which this crate has." |
| 1379 | D | "**This took an `interpret_spike: bool` until 4b's Task 1**, and the hundred-plus callers that argument's removal touched are the direct cost the paragraph above was weighing. `INTERPRET` is implemented now, so there is no mode left to select between: every caller passed `false` except the two spike tests, and all of them now say" -- history; deleted with the rest of its sentence at 1380-1384. The paragraph above it states the live reason the flag is set after construction. |
| 1501 | C | "which is where it already sits on every path 4a can reach. Measured:" -> "which is where it already sits on every path here. Measured:" |
| 1585 | K | "all 29 of `phase-4a.txt`'s programs" -- names a committed artefact, and the figure is the measurement's own. |
| 1659 | K | "The 4a exit gate's criterion 4" -- names a committed gate document's criterion. |
| 1666 | D | "(Task 3's own `run_program_interpret_spike` carried the identical note until 4b's Task 1 deleted it, `INTERPRET` being implemented; this is now the crate's only hidden entry point.)" -- history about a deleted item. |
| 1713 | C | "**A top-level parse failure stays loud, and 4b's Task 2 checked whether it could stop being loud rather than assuming either way.**" -> "**A top-level parse failure stays loud, and that was checked rather than assumed.**" |
| 1789 | C | "which nothing in 4a or 4b-so-far can do; it" -> "which nothing in this crate can do; it" |
| 1865 | C | "every operator, prefix and dyadic, is implemented within Phase 4a -- the spike's witness had to move from `+` to `=` when Task 7 landed and would have moved again for Task 8." -> "every operator, prefix and dyadic, is implemented here -- a witness picked from the operators has to move each time one lands." |
| 1903 | D | "-- and asked 4b to re-make it rather than inherit it." Deleted with the block at 1895-1914 down to "Re-made here, and the argument that once favoured the integration test now settles it the other way", which is an account of the move. The live argument that follows it is kept. |
| 1920 | C | "Measured on the oracle in 4a: the binding outlives the fragment." -> "Measured on the oracle: the binding outlives the fragment." |
| 2005 | C | "4b Task 2: a condition raised inside an `INTERPRET` fragment reports" -> "A condition raised inside an `INTERPRET` fragment reports" |
| 2156 | C | "**Already a live 4a divergence before any fragment base existed**: a" -> "**The divergence does not need a fragment base**: a" |
| 2172 | D | "That used to read \"omits two `>>>` lines this crate does not yet emit\"; the omission" -- an account of the comment's own past. |
| 2173 | D | "was fixed at 4b Task 9 and the reason for the plain `do` is now only that this test is about a fragment's own indent." -- same sentence; replaced by keeping the live reason, "A plain `do` block rather than `do z = 1 to 1` on purpose: a `Controlled` loop's re-tested pass traces its own control-variable value lines, which this test is not the place to encode." |
| 2201 | C | "4b Task 2, Step 5b: a fragment that does not parse raises" -> "A fragment that does not parse raises" |
| 2205 | D | "Through 4a and 4b's Task 1 both were `Loud::parse` at rc 120 -- correct-but-loud while `INTERPRET` was unreachable, and a live divergence once Task 1 made it reachable." -- history. The measured oracle numbers above it stay. |

Totals: **D 11, C 41, K 19** (71).

---

## `rexx-exec/tests/coverage.rs` (38)

| Line | Cat | Treatment |
|---|:-:|---|
| 12 | K | "The 4a exit gate's criterion 1, coverage half" -- names a committed gate document's criterion. |
| 14 | K | "`rust/corpus/phase-4a.txt`" -- committed artefact. |
| 41 | C | "A variant outside 4a's scope does not get a witness" -> "A variant this crate does not implement does not get a witness" |
| 46 | K | "must be one of `\"4b\"`, `\"4c\"`, `\"Phase 5\"` or `\"Phase 7\"`, spelled exactly as the split table" -- the set is `SPLIT_TABLE_PHASES`, asserted in this test binary by `assert_owner_strings_are_split_table_phases`, which the same sentence names. |
| 48 | K | Names the design spec file. |
| 52 | C | "The **set** of out-of-4a variants is pinned" -> "The **set** of out-of-scope variants is pinned" |
| 54 | K | "rather than merely \"whatever the `tags!` tables currently say\"" -- the quoted phrase is the rejected alternative, not a status claim. |
| 65 | K | "\"argument attachment inside Call, QualifiedCall, Message, List and VariableReference is exercised by 4b and 4c\"" -- a verbatim quotation of the design spec. |
| 80 | K | "`Loud::unresolved_call` naming `4c`" -- the frozen message's owner. |
| 85 | C | "The four that remain (`QualifiedCall`, `ClassResolver`, `List`, `Message`) are all `Phase 5`'s." -> deleted as a sentence and replaced by "Which variants remain, and who owns each, is `owners.rs`'s `EXPR_TAGS` and `EXPECTED_OUT_OF_SCOPE`; both are asserted here rather than described." (a count of what is left is exactly the shape that rotted) |
| 91 | K | "That is not a gap 4b or 4c will close, so it does not get a phase string" -- the sentence's subject is `Owner::Unreachable`, and `only_backslash_is_unreachable` in this file asserts it. |
| 97 | K | "except `With` really is owed to Phase 5" -- `owners.rs`'s `LOOP_TAGS` row, asserted through `EXPECTED_OUT_OF_SCOPE`. |
| 105 | C | "trimmed to what the 4a subset actually contains" -> "trimmed to what the subset actually contains" |
| 144 | C | "Trimmed from `rexx-parse/tests/gate_walk/mod.rs` to what the 4a subset actually contains: no directives." -> "Trimmed from `rexx-parse/tests/gate_walk/mod.rs` to what the subset actually contains: no directives." |
| 409 | C | "so a later task's own subset file (4b's, say) can run *alongside*" -> "so a later phase's own subset file can run *alongside*" |
| 410 | K | "`phase-4a.txt`" -- committed artefact. |
| 412 | C | "each phase's own harness run choosing between \"4a's programs\" and \"my own programs\"" -> "each phase's own harness run choosing between the earlier programs and its own" |
| 415 | D | "Task 0 shipped the widened signature with every caller still passing a one-element slice containing only `phase-4a.txt`, so Step 4 itself was a signature change with no behaviour change;" -- history. |
| 416 | D | "4b's Task 1 (`a462e3e9`) is where a second file was first passed, `phase-4b.txt` alongside it." -- history. |
| 417 | D | Same sentence. |
| 442 | K | "`phase-4a.txt` and `phase-4b.txt` do **not** overlap -- 30 entries and 12, union 42" -- a measurement, and it is the reason this test's hand-written inputs exist. |
| 476 | K | "`phase-4a.txt`'s exact line list" -- committed artefact. |
| 484 | K | "`mutate-4a.sh` falls from 9 of 9 caught to 5 of 9" -- a measurement from a named review. |
| 523 | C | "4b Task 2's own addition, and the plan amendment this assertion exists to make visible." -> "The plan amendment this assertion exists to make visible." |
| 524 | C | "The program is 4a by content -- 25 plain `DO` blocks around a failing clause -- and pins" -> "The program is 25 plain `DO` blocks around a failing clause, and pins" |
| 526 | C | "which 4a diverged on and no other program in this list nests deeply enough to reach." -> "which no other program in this list nests deeply enough to reach." |
| 545 | K | "`phase-4b.txt`'s exact line list, the same device [`EXPECTED_SUBSET`] is" -- committed artefact. |
| 546 | C | "for `phase-4a.txt` and added for the same reason, one sub-phase later." -> "for `phase-4a.txt`, and added for the same reason." |
| 548 | C | "**4b's gate review measured what its absence cost, and it was not theoretical.**" -> "**What its absence costs was measured, and it was not theoretical.**" |
| 549 | K | "Removing **one entry at a time** from `phase-4b.txt`" -- the measurement's own method. |
| 552 | K | "only needs *some* program to construct each variant, and by the end of 4b most variants have several witnesses" -- the measurement's own explanation, and the transcript is quoted below it. |
| 561 | K | "the 4b gate's criterion 8 witness" -- names a committed gate document's criterion. |
| 562 | K | "`mutate-4b.sh`'s mutations (row 6, `SIGL` off by one)" -- names a committed script and row. |
| 601 | K | "read against the **union** of every phase's subset file rather than `phase-4a.txt` alone" -- states the property the test has, and names a committed artefact. |
| 603 | D | "Renamed from `..._by_the_phase_4a_subset` at 4b's Task 1, which is the" -- history about the test's own name. |
| 605 | K | "its witness cannot live in `phase-4a.txt`, whose own header excludes `INTERPRET` by definition" -- committed artefact and its header; this is the reason the union exists. |
| 607 | C | "is what keeps every 4a witness exercised as later phases add their own subsets -- the reason `read_subset` takes a slice at all (Task 0's Step 4), and this is the first call site to pass more than one path." -> "is what keeps every earlier witness exercised as later phases add their own subsets, which is why `read_subset` takes a slice at all." |
| 612 | K | "it pins `phase-4a.txt`'s exact line list" -- committed artefact. |

Totals: **D 4, C 13, K 21** (38).

---

## `rexx-exec/tests/trace_oracle.rs` (24)

| Line | Cat | Treatment |
|---|:-:|---|
| 27 | C | The module doc's prefix paragraph (26-35) restates `PREFIX_COVERAGE`, which this file asserts. "Measured reachable from pure-4a code: ... all nine covered here. `>E>` is *not* on that list but is, in fact, reachable ... 4b's calls add three more (`>A>`, `>F>`, `>R>`, Task 9), witnessed below. The remaining six ... are later phases' and have no witness here" -> "Which of them a witness below reaches, and who owns each of the rest, is [`PREFIX_COVERAGE`]'s -- asserted, not described here. One correction to the design spec's own \"measured reachable from pure-4a code\" list is worth keeping because it was got wrong once: `>E>` is not on that list and is nevertheless reachable (`dotvariable_beyond_the_list.rex`)." |
| 30 | C | Same paragraph. |
| 31 | C | Same paragraph. |
| 139 | C | "(Task 14a's own file, already a Phase 4a subset member) rather than being duplicated here" -> "(already a corpus subset member) rather than being duplicated here" |
| 235 | C | "the exact program that closed this task's own four remaining corpus failures (`rust/corpus/phase-4a.txt`), read from `rust/corpus/` rather than duplicated here." -> "read from `rust/corpus/` rather than duplicated here." |
| 261 | C | **False as written** -- three `PrefixOp` variants are in scope (`owners.rs`: `Plus`, `Minus`, `Not`), so "the two prefix operators 4a implements" is wrong; the witness file covers two of them. "`>P>`: the two prefix operators 4a implements, `+` and `\\`." -> "`>P>`: the prefix operators this witness covers, `+` and `\\`." |
| 270 | C | "a correction this task found (`.nil` is one of" -> "a correction worth keeping because it was got wrong once (`.nil` is one of" |
| 271 | C | "`ExprKind::DotVariable`'s own three 4a-admissible names, D15, so it is reachable)" -> "`ExprKind::DotVariable`'s own three admissible names, D15, so it is reachable)" |
| 403 | C | "The nine prefixes the design spec's own \"measured reachable from pure-4a code\" list names, plus `>E>` (4a's own correction) and the three 4b's calls add (`>A>`/`>F>`/`>R>`, Task 9) -- thirteen total." -> "Every prefix a witness below is expected to reach, between them." |
| 404 | D | Same sentence. |
| 508 | C | "**Criterion 3's coverage measure** (D14 amendment 3, delivered by 4b's Task 9). Before" -> "**Criterion 3's coverage measure** (D14 amendment 3). Before" |
| 516 | K | "`+++` -- 4c. Two producers, and this row is the *command* one" -- the owner is `PREFIX_COVERAGE`'s own asserted data and the row justifies it from `RexxActivation.cpp:4468`. |
| 518 | K | Same row; cites `RexxActivation.cpp:4468`. |
| 520 | C | "which `TRACE ?` reaches today and which has its own" -> "which `TRACE ?` reaches and which has its own" |
| 523 | K | "`>.>` -- 4c. The `PARSE` template's placeholder" -- cites `ParseTrigger.cpp:285`, read directly. |
| 524 | K | Same row. |
| 525 | K | "`>M>` -- Phase 5. Message sends" -- cites the exclusions file's ownership table, and the owner is the asserted const's. |
| 527 | K | "`>N>` -- Phase 5. `traceClassResolution`" -- same. |
| 528 | K | Same row. |
| 529 | K | "`>I>`/`<I<` -- 4c ... Measured rather than assumed, and the exclusions file's own row carries the transcripts" -- cites `RexxActivation.cpp:3655`. |
| 532 | C | "(`RexxActivation.cpp:3655`), so **both** halves are needed, and 4b reaches neither" -> "(`RexxActivation.cpp:3655`), so **both** halves are needed, and this crate reaches neither" |
| 533 | K | "`::routine` is deferred to 4c by decision, not by unreachability" -- the row's own point, and the owner is the asserted const's. |
| 559 | D | "**Thirteen of nineteen at the end of 4b**, up from ten at the 4a gate: `>A>`, `>F>` and `>R>` are Task 9's." -- restates the const below it, which is asserted, and narrates its history. |
| 568 | C | "Same four strings `coverage.rs`/`loud.rs` police for `ExprKind` ownership, minus `4b` -- a prefix this phase owns is witnessed by now, not owned by a phase that has finished." -> "A phase that has finished cannot own a prefix -- whatever it owned is witnessed by then -- so a finished phase's name does not appear here." |

Totals: **D 2, C 13, K 9** (24).

---

## `rexx-exec/tests/owners.rs` (22)

| Line | Cat | Treatment |
|---|:-:|---|
| 20 | D | "in 4a, so each carried its own `Owner` enum, its own `tags!` macro and its own seven tag tables, kept in sync by hand. Nothing caught a divergence between them." Deleted with the "# Why this used to be two copies (inherited item I36)" heading and its paragraph (16-21): an account of a structure that no longer exists. The live half -- "This file is read by both instead, through `#[path = \"owners.rs\"] mod owners;`" and its named test -- is kept and moved under the opening paragraph. |
| 59 | C | "4a's own: must be witnessed by at least one program in the subset." -> "Implemented here: must be witnessed by at least one program in the subset." |
| 120 | C | "// ---- 4a's own twenty, plus Interpret (4b's Task 1) ----" -> "// ---- implemented here ----" (a count of the rows below it is the shape that rotted four times) |
| 142 | D | "In scope since 4b's Task 1: 4a built the fragment machinery and that task built the keyword on top of it." -- history on an `InScope` row. |
| 145 | D | "In scope since 4b's Task 3, with `CALL`." -- history on an `InScope` row. |
| 160 | K | "the parenthesised list is an *expression* this crate cannot evaluate, reported against that expression ... (measured, the two spellings' reports are byte-identical)" -- a measured oracle fact, and it is the reason `Raise` has no arm-grained row. |
| 172 | C | "// ---- 4c's four ----" -> "// ---- 4c's ----" |
| 177 | K | "// ---- Phase 5's ----" -- a section marker over rows whose `Owner::Phase("Phase 5")` is right beside it and is asserted against `EXPECTED_OUT_OF_SCOPE`. |
| 205 | C | "// ---- 4a's own nine, plus Call (4b's Task 4) ----" -> "// ---- implemented here ----" |
| 215 | D | "In scope since 4b's Task 4: unlike `InstructionKind::Call`, which stays split -- `Owner::Phase(\"4b\")` at the time this comment was written, because `Call::Trap`/`Call::Qualified` were both still loud; `Owner::Phase(\"Phase 5\")` since Task 7 moved `Call::Trap` in scope, leaving only `Call::Qualified` loud (review round 1's M6 corrects this comment, which went stale the same way `loud.rs`'s own `INSTRUCTION_WITNESSES` doc did at the same task and for the same reason: an edit at line 129 below was not propagated here) --" -- this is the comment `rust/CLAUDE.md` cites as having rotted four times, narrating its own corrections. Replaced by the property: "`ExprKind::Call`'s own `CallTarget` has exactly two forms and this crate evaluates both, so unlike `InstructionKind::Call` there is no later-phase arm left hiding inside it -- see `eval_call`'s own doc (`eval.rs`) for the resolution order a name still falls through to the loud fallback for." |
| 216 | D | Same comment. |
| 218 | D | Same comment. |
| 224 | D | Same comment. |
| 226 | D | Same comment. |
| 229 | C | "In scope since 4b's Task 5: `>x`/`<x` evaluates to the referenced" -> "`>x`/`<x` evaluates to the referenced" (the measurement that follows is kept) |
| 246 | C | "`DO WITH ... OVER` sends SUPPLIER, which nothing in 4a answers." -> "`DO WITH ... OVER` sends SUPPLIER, which nothing in this crate answers." |
| 353 | C | "The out-of-4a variant set this file's `tags!` tables are allowed to produce" -> "The out-of-scope variant set this file's `tags!` tables are allowed to produce" |
| 385 | K | Names the design spec file and the table inside it. |
| 547 | C | "not a private copy that happens to agree with it today." -> "not a private copy that happens to agree with it." |
| 574 | C | "any task moving an `InstructionKind`, `ExprKind` or `LoopKind` variant into 4a's scope (or otherwise changing which phase owns it)" -> "any task moving an `InstructionKind`, `ExprKind` or `LoopKind` variant into scope (or otherwise changing which phase owns it)" |
| 580 | C | "the pinned `(category, tag, phase)` set every out-of-4a variant must appear in exactly once." -> "the pinned `(category, tag, phase)` set every out-of-scope variant must appear in exactly once." |
| 583 | K | "`phase-4a.txt`, checked by that file's own `phase_4a_subset_matches_the_committed_list`" -- committed artefact and the test that pins it. |

Also in this file, **added** rather than culled: `SPLIT_TABLE_PHASES` (387) gains the
reasoning currently stranded in `loud.rs`'s witness array at 212-215 -- why a finished
phase's name stays a valid owner string. It is a non-obvious decision about *this*
constant, and `loud.rs` is not where it belongs. See `loud.rs` 212/213 below.

Totals: **D 8, C 10, K 4** (22); plus one addition, below.

---

## `rexx-exec/tests/loud.rs` (20)

| Line | Cat | Treatment |
|---|:-:|---|
| 12 | K | "The 4a exit gate's criterion 5" -- names a committed gate document's criterion. |
| 13 | C | "variant either belongs to 4a's named set, or a program constructing it" -> "variant either belongs to the set this crate implements (`owners.rs`), or a program constructing it" |
| 17 | K | Names the design spec file and the section inside it. |
| 27 | C | **Incomplete as written** -- the corpus reads `phase-4a.txt` and `phase-4b.txt` as a union (`corpus.rs`'s own module doc), so citing only the first is no longer the whole reason. "`tests/corpus.rs`'s subset is defined to contain none of them, by `phase-4a.txt`'s own header" -> "`tests/corpus.rs`'s subset is defined to contain none of them, by its subset files' own headers" |
| 29 | K | "\"one criterion closes a surface larger than 4a's own\"" -- a verbatim quotation of the design spec. |
| 31 | C | "`4a executes it` is already established" -> "`this crate executes it` is already established" |
| 56 | K | "the object model and is Phase 5's, mirroring `ExprKind::QualifiedCall`" -- this is the reason `Call` is arm-grained at all, and Task 1's assertion in this same file now checks it against `owners.rs` per row. |
| 83 | D | "`PROCEDURE` was on that list until 4b's Task 5 moved it in scope and deleted its row; a bare top-level `procedure` is now the oracle's error 17.1, not a gap." -- history about a deleted row; the same oracle fact is recorded again at 197-203, which is also deleted. |
| 95 | D | "where it used to answer `rexx-exec: a variable reference is not implemented (4b)`" -- history. The two measured facts around it (`say >x` prints the referenced value; `say 'text' >x` is a comparison) are kept. |
| 193 | D | "**No `Call::Trap` row since 4b's Task 7.** `CALL ON`/`CALL OFF` is implemented, so a row here would assert a loud failure that correctly no longer happens -- the same way `Procedure`'s and `Use`'s rows had to go at Task 5." -- the plan's own worked example of category 1. |
| 197 | D | "**No `Procedure` or `Use` row since 4b's Task 5.** ..." (197-203) -- same. |
| 204 | D | "**No `Signal` row of any grain since 4b's Task 7, and no `Raise` row.** ..." (204-209) -- same. |
| 210 | D | "**No `Push` or `Queue` row since 4b's Task 8 (I15).** Both moved into scope: `queue.rs` stores every line either writes." -- same. |
| 212 | D | "Nothing in `owners.rs`'s tables is owned by `\"4b\"` any more" -- a status fact about another file's table, unasserted here. |
| 213 | D | "and `SPLIT_TABLE_PHASES` keeping `\"4b\"` as a valid phase name is not stale, since a phase can still owe nothing right now and owe something again if a later task's audit finds otherwise." -- deleted **here** and the reasoning moved to `owners.rs`, on `SPLIT_TABLE_PHASES` itself, phrased as a property: a phase name is valid because the split table names it, not because something currently owes to it. |
| 269 | C | "every one wrapped in `SAY` (4a's own) -- see" -> "every one wrapped in `SAY`, which is implemented, so the wrapper is never itself the gap -- see" |
| 272 | D | "**No `Call` row since 4b's Task 4.** `ExprKind::Call` moved fully into scope ..." (272-278) -- history about a deleted row. |
| 276 | D | Same block. |
| 434 | D | "4 since 4b's Task 5 moved `ExprKind::VariableReference` in scope and deleted its row (5 after Task 4 did the same for `ExprKind::Call`, 6 before that)." -- a comment narrating the descent of the number asserted on the very next line. |
| 513 | K | "which a message reading `[4b] CALL: unimplemented` also satisfies" -- an invented example of a string `contains` would wrongly accept, not a status claim; it cannot go stale. |

Totals: **D 11, C 4, K 5** (20). Lines 198-203, 205-209 and 273-278 carry no hit of their own and go with the blocks they belong to.

---

## The other 38 files (219 lines) -- triaged, not edited

The plan's Task 2 lists six files under **Files:**. These 219 lines are outside that
list and are left alone; this section records the judgement so the decision is visible
rather than implicit.

**Roughly half are K on the run-time-state reading of the search term** and were never
targets. The search cannot distinguish them and neither could the 522:

| File | Lines | Why K |
|---|---|---|
| `rexx-core/src/roots.rs` | 54, 89, 149, 175, 199, 393 | "every currently active activation", "currently pushed frame", "currently rooted", "currently open", "currently holds" |
| `rexx-num/src/settings.rs` | 28, 34, 141, 254, 263 | "the DIGITS currently in force" |
| `rexx-exec/src/clause.rs` | 185, 203 | "the clause currently being stepped", the oracle's "currently-executing instruction" |
| `rexx-exec/src/activation.rs` | 12, 87 | "the frame currently executing" |
| `rexx-exec/src/value.rs` | 42 | "not yet asked" -- a cache state |
| `rexx-core/src/body.rs` | 52 | "not yet asked" -- a cache state |
| `rexx-exec/src/stem.rs` | 463, 650 | "currently a heap `Body::Stem`", "what `r.`'s slot currently holds" |
| `rexx-exec/src/error.rs` | 486 | "a target that is not currently unset" |
| `rexx-parse/src/expr.rs` | 486 | "how many levels are currently open" |
| `rexx-parse/src/block.rs` | 371, 372 | "whatever currently ends the chain" |
| `rexx-num/src/muldiv.rs` | 360 | "not yet reached the first significant digit" |
| `rexx-num/src/lib.rs` | 115 | "Currently raised only by ..." |
| `rexx-inventory/build.rs` | 42 | "which child we are currently accumulating" |

**A further group is K because the referent is outside this repository or is a committed
artefact**: `rexx-inventory/build.rs:216` and `rexx-inventory/tests/errors.rs:5` ("as of
8c880bdd", a commit in the C++ tree); `rexx-exec/tests/corpus.rs:13`, `:250`, `:252`,
`collect_stress.rs:12`, `:75`, `:100`, `extract_keyword.rs:68` (subset filenames);
`rexx-exec/tests/keyword_assertions.rs:223` (a gate document's criterion);
`rexx-extract/src/keyword.rs:182`, `:662`, `spike.rs:101`, `:109`, `:165` (message
dispatch is Phase 5's, which is what makes `~` the one witness no task can implement out
from under the test -- the argument depends on the phase being named).

**The remainder are real targets** of the same three kinds as the six files above --
principally `rexx-exec/src/eval.rs` (22, 24, 25, 472, 482, 484, 507, 509, 1718, 1740,
1741, 1761, 1762), `rexx-exec/src/queue.rs` (16, 17, 40, 44, 69, 71, 87, 144, 229, 230),
`rexx-exec/src/activation.rs` (25, 103, 104, 120, 127, 131, 135, 235, 239, 246, 247,
324), `rexx-exec/src/trace.rs` (13, 15, 41, 62, 63, 80, 81, 82, 189, 237, 271, 294, 504,
507, 508, 740, 764), `rexx-exec/src/error.rs` (25, 136, 138, 178, 559, 572, 589, 593,
645, 691, 692, 753, 771, 806, 885, 960, 987, 1093, 1099, 1182), `rexx-exec/src/stem.rs`
(62, 63, 64, 227, 370, 375, 393, 398, 401, 530), `rexx-core/src/roots.rs` (66, 347, 350,
356, 357), `rexx-core/tests/collect.rs` (168, 170, 178, 194, 195, 196),
`rexx-exec/tests/assertions.rs` (74, 78, 82, 87, 89, 90, 92, 100, 101, 102, 103, 280,
310, 327, 340, 344, 345, 630, 791), `rexx-exec/tests/keyword_assertions.rs` (12, 42, 59,
60, 61, 86, 91, 92, 483, 633, 751, 828), `rexx-exec/tests/corpus.rs` (20, 21, 24, 28, 30,
36, 74, 195, 539, 541), `rexx-exec/tests/spike.rs` (22, 94, 99, 105, 155),
`rexx-exec/tests/collect_stress.rs` (102, 103, 105), `rexx-exec/src/plan.rs` (25, 77,
127, 599), `rexx-exec/src/clause.rs` (15, 107), `rexx-exec/src/value.rs` (64, 177),
`rexx-extract/src/keyword.rs` (78, 137), `rexx-extract/src/lib.rs` (172, 203, 358),
`rexx-extract/tests/extract_assertions.rs` (224, 278, 321),
`rexx-extract/tests/extract_keyword.rs` (621),
`rexx-extract/src/bin/rexx-extract-assertions.rs` (30), `rexx-core/src/heap.rs` (49,
100), `rexx-parse` (`tiling.rs:38`, `sourceline_oracle.rs:40`, `expr.rs:275`,
`expr/differential.rs:210`, `:214`, `instruction/tests.rs:1874`, `error/tests.rs:164`,
`examples/depth_probe.rs:46`, `benches/parse.rs:56`).

Count: **about 110 real targets**, the rest K. See the report.
