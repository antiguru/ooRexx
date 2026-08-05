# Phase 4c implementation plan: builtins, PARSE, and the rest of the call chain

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** finish classic Rexx.
After 4c, a program that calls builtins, parses strings, reads the queue, tracks an `ADDRESS` environment and calls a `::routine` runs byte-for-byte as `build/bin/rexx` runs it, and Phase 4's parent row closes.

**Architecture:** 4b made the activation stack real.
4c hangs the builtin table off the one call-resolution chokepoint 4b built, adds a `PARSE` template engine, and closes the call chain's last in-scope step with `::routine` dispatch.
The 66 builtins live in a new `builtin/` module tree, one file per family, so no two tasks touch one file.
No new crate; `rexx-exec` grows.

**Tech stack:** Rust 1.96.1, no `unsafe`, `cargo fmt` default, `clippy -D warnings`.
Depends on `rexx-core`, `rexx-num`, `rexx-parse`, `rexx-inventory`.

## Revision note

**This plan's first revision was substantially wrong and was corrected before any task ran.**
Four independent reviewers attacked it: one re-verified every measurement, one tried to refute the two new decisions, one checked executability, one hunted checks that cannot fail.
They returned **eleven blockers**.
The three worst are recorded here because each is a defect an implementer would otherwise inherit:

* **Task 1's status harness never consulted the oracle.** It classified a builtin `implemented` by the *absence of a loud message*, so 66 stubs returning `''` satisfied it -- the exact `/bin/true` shape this plan warns about -- and Task 13, which deletes the only producer of that message, would have made deleting the entire `builtin/` tree leave all 66 rows green.
* **Seven of the fifteen tasks had no heading.** Tasks 3-6 and 10-12 shared one `### Tasks 3-6, 10-12` section, and briefs are extracted per heading, so seven implementers would have received empty briefs.
* **Every measurement over `ootest/ooRexx/base/bif` was short.** `grep` on this machine is a bash function wrapping `ugrep --ignore-files -I`, which silently skips files containing non-UTF-8 bytes -- and six `base/bif` groups are exactly that, because they test byte conversion. Thirteen figures were wrong from that one cause. **Use `/bin/grep -a` for any count in this phase.**

Everything below marked *measured 2026-08-04* was taken or re-taken after that review.

## The governing documents, and what each is for

* **`docs/superpowers/plans/phase-4-exclusions.txt`** is the live record of what Phase 4 does not do.
  Adding a `KNOWN GAP` row needs no permission; removing one does.
  **Four of its rows are corrected by Task 1** and must not be read as authority until then: `+++`'s owner, the `TRACE ?` row's missing owner and its "only stderr differs" claim, and the `>I>`/`<I<` row's citation.
* **`docs/superpowers/plans/phase-4b-gate.md`** is the criterion set 4c's gate derives from, under D14.
* **`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`** governs the value model, the borrow shape, and D15 to D19.
  **Its line 71 is amended by Task 1** -- see D-R.
* **`docs/superpowers/specs/2026-08-01-phase-4bc-scoping.md`** is groundwork, not requirements.
  **Do not read it to find your requirements.**
  Its `trace r` probe for `>.>` could not have seen the prefix, and three of its inherited items are superseded here.

---

## Global constraints

Every task's requirements implicitly include this section.
**It is not extracted into task briefs, so a task that depends on one of these lines restates it.**
The tasks below that need a constraint restate it in their own bodies; that duplication is deliberate.

* **The C++ tree is the oracle and is never modified.** `interpreter/`, `samples/`, `build/`, `ootest/` are read-only.
* **Wrap every oracle invocation** as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  Without the ulimit the interpreter requests gigabytes mid-range and is OOM-killed, which has already cost a session and the machine's memory.
* **Use `/bin/grep -a`, not `grep`, for any count or any search.**
  `grep` here is a function wrapping `ugrep --ignore-files -I`; `-I` drops files with non-UTF-8 bytes and says nothing about having done so.
  Measured 2026-08-04: over `base/bif` it reports 5,441 `assertSame` where the true figure is **6,293**.
  Seven groups trip it -- `C2X`, `COPIES`, `D2C`, `DATATYPE`, `DELSTR`, `DELWORD`, `INSERT` -- and they are the byte-handling groups, so **the risk is highest exactly where 4c's work is**.

  **"Silently skips" understates it, and the true failure mode is worse than a wrong number.**
  Measured on `DELWORD.testGroup`, which carries five NUL bytes at line 136:

  ```
  grep -c 'delword' DELWORD.testGroup          ->  no output at all, exit 1
  /bin/grep -ac 'delword' DELWORD.testGroup    ->  1, exit 0
  ```

  It prints **nothing** and exits **1** -- which in a shell is indistinguishable from a legitimate "no matches".
  So it fails in the direction that looks like a valid negative result, and any "X is absent" conclusion drawn with `grep` over these files is worthless.
  **Pair every absence claim with a positive control** that finds the same pattern somewhere it does exist.
* **Cite `phase-4-exclusions.txt` by quoted phrase, never by line number.**
  Tasks 1 and 13 both edit it, and Task 1 alone moved every line below its first edit -- one row's citation shifted from `:1009` to `:1147`.
  A line number into a file this plan itself rewrites is stale before the task that reads it runs.
  The 4b gate cites `keyword_assertions.rs` the same way, by the fragments `now PASSES` and `is not on the committed`, for the same reason.
* **State which scan produced any count.** Measured over `base/bif`: `^::method` gives **5,397**, `^[[:space:]]*::method` case-insensitively gives **5,462**.
  Both are right, for different questions.
  `DATE.testGroup` and `TIME.testGroup` alone spell 50 directives `::METHOD`.
* **Read stdout, stderr and exit status as separate descriptors.** Never `2>&1`. Read exit status unpiped.
* **Never set `NUMERIC DIGITS` above 1000** in a probe.
  This binds 4c harder than 4b: `FORMAT`, `TRUNC`, `D2X` and `X2D` all take a digits-shaped argument, and the natural "try a big one" probe is the one that kills the machine.
* **Never instantiate `.Package~new`** on a file inside the repository: it executes that file's prolog.
* **Never probe `select; when 1 = 0 then; when 2 = 2 then nop; end`.** It segfaults the oracle (upstream SF #2018).
* **Run every oracle probe from a fresh empty subdirectory of the scratchpad**, using absolute paths.
  The scratchpad root is on the oracle's **external-routine search path** and holds hundreds of stale `.rex` files.
  Measured: `say "f"(1)` with an internal `f:` reports 44.1 rc 212 in the root and 43.1 rc 213 in a clean directory.
  **This bites 4c hardest of any sub-phase, and Task 13 makes it worse rather than better**: an unresolved name reaches the external search, and 4c deliberately does not implement it (see D-R's R-3 note).
* **Beware Rexx literal syntax in probes.** A symbol named `x` or `b` immediately followed by a quoted string parses as a hex or binary literal, so `say '['x']'` is error 15.3.
  **Task 5 is where this bites**: `b2x`, `x2b`, `x2c`, `c2x`, `d2x`, `x2d`. `say x2d('ff')` is fine; `say x'41'` is a different program entirely.
* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`.
  If a task appears to need it, stop and report BLOCKED.
* **Never `git add -A`.** Stage the exact paths the task names.
  Do not `git reset --hard`, do not force-push.
* Comments state the contract at the top and the reasoning at the decision point.
  Never delete a true comment to make a change easier. Prefer `--` over an em-dash.
  The "no structuring semicolons" rule does **not** apply to this repository.
* **A comment may say what the oracle does and what was measured. It may not state where the implemented/not-implemented boundary sits.**
  **Assert the boundary or do not write it down** -- Task 1 builds the mechanism that makes asserting it free.
* A value's rendering is fixed when the value is created (D15).
  Any code that formats a number with `settings.digits()` instead of the value's own captured pair is wrong.
* Anything Phase 4c does not implement **fails loudly**: `NOT_IMPLEMENTED_EXIT` (120), outside `157..=253` where `256 - major` lives.
* **Every new allocation site goes through `Interp::alloc_with`**, the wrapper around `Heap::alloc_with_uncollected`.
  **4c adds more allocation sites than 4a and 4b combined** -- every builtin returning a string allocates -- so this constraint fails by volume, not by misunderstanding.
* **Commit first, then read the hash back, then record it.**
* **`cargo fmt --all --check` from `rust/`.** Not `cargo fmt --edition 2024 --check`, which exits 2 without doing any work.
* **Run clippy from a clean target directory at each task's end.**
  Measured at the pre-4c boundary: the identical command reported exit 0 across a whole session at two commits that fail it on a cold target.

---

## Decisions taken for 4c

### D4 -- the fifteen excluded builtins stay excluded, and `QUALIFY` with them

**Measured 2026-08-04:** `/bin/grep -aE '\bqualify *\('` over `base/bif` returns hits in **`QUALIFY.testGroup` only**.
The stated condition for pulling it in ("needed by an L1 group 4c must pass") is measured **false**.
`EXCLUDED_BUILTINS` keeps its 18 rows and `in_scope` stays 66.

**Beware the 18/15 distinction; it has already produced two defects in this plan.**
`EXCLUDED_BUILTINS` has **18** rows, of which **three are partial** -- `VALUE`, `ADDRESS` and `QUEUED` are **in scope** and must dispatch.
`coverage.rs` computes `81 - (18 - 3) = 66`.
Any task that writes "`NAMES` minus `EXCLUDED_BUILTINS`" gets **63** and is wrong.

**Task 1 owes this decision one sentence per excluded row saying why it is blocked.**
Three need a reason that is not "the platform layer":

* `USERID` -- Rust `std` has no username API; `getpwuid` needs libc and the workspace forbids `unsafe`. Host-dependent, so single-machine.
* `SETLOCAL` / `ENDLOCAL` -- they save and restore the process environment, and `std::env::set_var` is `unsafe` in edition 2024 (verified: `error[E0133]`).
  Phase 7 needs a shadow environment or a lint exception, and **that is a decision, not a task**.

### D11 -- no `RANDOM`, `DATE` or `TIME` in any differential corpus program

**Measured 2026-08-04.** `RANDOM.testGroup` asserts only *properties*: 100 in-range reps per range, a seeded sequence generated twice inside one process and compared to itself, and three degenerate ranges where the answer is forced (`random(1,1)`, `random(0)`, `random(,0)`).
**No `assertSame` pins a generator output**, so reproducing ooRexx's PRNG and default seed is not forced.

**Two requirements this hides, both of which a loose reading loses:**

* **The reproducibility property is stronger than "seedable".** The group seeds once and makes **99 further unseeded calls**, then re-seeds and repeats, and requires all 100 to match.
  A generator that re-seeds on every call satisfies "seedable and deterministic" and **fails this**.
  The property is: seeding sets a stream, and unseeded calls advance it.
* `RANDOM.testGroup` also holds **8 `expectSyntax` cases** (six 40.12, one 40.13, one 40.33), so its argument validation is pinned even though its values are not.

The corpus rule goes in `corpus/README.md` beside the `DO OVER` one.

### D12 -- `base/bif` reuses the whole-body extractor from 4b's Task 11; no third extractor

**Measured 2026-08-04 with `/bin/grep -a`, over the 76 `.testGroup` files (31,115 lines).**
The earlier figures in this plan were taken with `ugrep -I` and were short by the six binary-containing groups.

| method | calls | files |
|---|---|---|
| `assertSame` | 6,293 | 73 |
| `expectSyntax` | 1,230 | 62 |
| `assertTrue` | 232 | 18 |
| `assertEquals` | 116 | 19 |
| `assertFalse` | 76 | 9 |
| `assertSameList` | 5 | 1 |

The shape is `assertSame`-dominated exactly as `base/keyword` was, so `rexx-extract`'s existing whole-body extractor is the right instrument and a third is not written.
**Reuse over writing, and the conservation invariant comes with it**: `rows + dropped == calls`, the property that caught the Phase 0 defect, must be preserved rather than re-derived.

**Size, stated like for like.**
`base/bif`: **5,397 `::method` under `^::method`**, or 5,462 allowing indentation and any case, in 76 files.
`base/keyword`: **2,105** under the same anchored scan, in 39 files, from which the extractor emitted **896** bodies -- a 42.6% yield.
At that yield `base/bif` would emit roughly **2,300**; that is an extrapolation and Task 15 measures the real number.

**Two case traps, and they point opposite ways.**

* **`assertSame` is all lowercase here.** 6,293 lowercase, **zero** capital-`A`.
  In `base/keyword` it is 1,931 lowercase plus **510 `AssertSame`**, which is how its true total reaches 2,441 -- a case-sensitive count there drops a fifth of the table.
  Match case-insensitively: correct in both, costs nothing here.
* **`::method` is *not* all lowercase here.** `DATE.testGroup` and `TIME.testGroup` spell 50 directives `::METHOD`.
  A case-sensitive body scan drops them.

`assertSameList` is a different method -- **match the token, never the prefix.**

The 424 calls in the `assertTrue`/`assertEquals`/`assertFalse` tail are dropped by an `assertSame`-only extractor.
**That is acceptable and belongs in the file's header**, where the conservation invariant carries the number rather than a comment.

### D8 -- `rexxcps.rex` is a run-to-completion smoke test; the dependency list is the gate

Read in full 2026-08-01. Not byte-comparable, for two independent reasons: it prints wall-clock timings, and **its loop count is auto-adjusted from measured elapsed time** (`count=(1%total + 1) * count`, repeated until `total>1`), so its output length and control flow depend on host speed.

Keep it as a smoke test asserting **rc 0 and completion**; make the real gate the dependency list, each item with its own differential witness: `parse var`, `parse version`, `parse value`, `parse upper`, `parse source`, `trace value`, `trace off`, `signal on novalue`, one internal `call subroutine`, the `call time 'R'` call-to-builtin **instruction** form, `address value` with `ADDRESS()`, and eight builtins -- `TIME`, `SUBSTR`, `FORMAT`, `WORD`, `TRACE`, `LENGTH`, `LEFT`, `ADDRESS`.

**Task 15 Step 6 carries this ruling; no other task does.**

### D7 -- closed in 4b, recorded so it is not reopened

`ExprKind::List` is Phase 5's; the three `num/` corpus programs return then.
Both halves of the two-file contradiction were fixed during 4b.
**Nothing in 4c touches this.**

### D-R -- `::routine` is 4c's, and the design spec's line 71 is amended

**The tree contradicts itself.**
`2026-07-30-phase-4a-executor-design.md:71` says "every directive ... [is] Phase 5's", and the `QualifiedCall` row in `phase-4-exclusions.txt` leans on that sentence to place `QualifiedCall` in Phase 5.
But the `>I>`/`<I<` row in the same file assigns `::routine` to 4c, in **two** places, and `trace_oracle.rs:542,546` **machine-assert** `Coverage::Owned("4c")`.
`::routine` is a directive. Both cannot hold.

**Cite that row by its own words, not by line number.**
Task 1 edits this file and every later line moves; the durable citations are the phrases themselves, which is how the 4b gate cites `keyword_assertions.rs`. The two ownership sentences are:

* "**4b declines to implement it because getting builtin-colliding names right needs 4c's table**" -- the reason for the deferral.
* "**THREE MEASURED WAYS A ::ROUTINE ACTIVATION IS NOT AN INTERNAL LABEL'S, which 4c will have to meet**" -- a direct statement that 4c implements the activation.

An earlier revision of this plan dismissed the second as "about the trace gate, not ownership".
**That was wrong and it understated the record**: the trace gate is the separate "THE GATE IS TWO CONDITIONS" paragraph, and the sentence dismissed is the strongest ownership statement in the file.

**Ruling: 4c owns `::routine` dispatch.** Line 71 is amended to "every directive except `::ROUTINE`".

Two reasons, and a third that was in the first revision and is withdrawn:

1. **The reason 4b deferred it is discharged exactly here.** 4b declined because builtin-colliding names need 4c's table, and 4c is where that table lands.
2. **It is one mechanism, and splitting it strands the resolution chain across a phase boundary.** Rexx resolves internal label, then builtin, then `::routine`.
   4b built step 1, 4c builds step 2; leaving step 3 to Phase 5 puts a hole in the middle of one chain for a whole phase.
3. ~~Otherwise Phase 4 closes with a construct the oracle runs at rc 0 still failing loudly.~~
   **Withdrawn: it proves too much.** By that argument almost anything deferred is in scope, and measured, `keyword-exempt.txt` has approximately **zero** rows blocked on `::routine`, so the practical benefit is small.
   The decision rests on 1 and 2.

**Cost, measured rather than feared.** `::requires` and namespaces do **not** travel with it, and "the package object" the `>I>` witness needs reduces to **one `String` field on `Interp`**.
`::options` is confirmed unnecessary: `>I>`/`<I<` fire on the routine's own `trace r` as well as `trace l`, because `earlyTraceEntry` accepts A/I/L/R.
So any routine-body trace witness emits them whether it means to or not.

**Six measured ways a `::routine` activation differs from an internal label's, not three.**
The first revision listed three; an implementer copying `run.rs:3304-3313` gets **three wrong answers, two of them silent**:

| property | internal label | `::routine` |
|---|---|---|
| variable pool | shared with caller | **its own** (`nn = 5` in caller; routine saying `nn` prints `NN`) |
| builtin collision | n/a | **builtin wins** (`call max 1, 9` returns 9, routine never runs) |
| trace | caller's setting crosses in | **does not cross** |
| `NUMERIC` | inherited | **not inherited** (`digits=9 fuzz=0 form=SCIENTIFIC` under a caller at 5/2/ENGINEERING) |
| `ADDRESS` | inherited | **not inherited** (`sh`, not the caller's `ZORKENV`) |
| condition traps | inherited | **not inherited** (routine with its own `shared:` label; the caller's trap still fires) |

**The resolution order is two orders, not one.**
A **quoted** target skips the internal label but still finds the `::routine`: `call 'ZORKOLO'` reaches the routine, while the builtin still wins over both.

**`::method` does not travel with it.** The C++ gate is one predicate (`isMethodOrRoutine`), but `::method` needs the object model and stays Phase 5's.

### D-P -- `+++` moves to Phase 7, and the `TRACE ?` row gets an owner

**`trace_oracle.rs:529` asserts `("+++", Coverage::Owned("4c"))` and that is wrong.**

**Four producers of a `+++`-prefixed line, not two** -- the first revision said two and attributed the whole two-line banner to one of them:

* `RexxActivation.cpp:4468` -- a command's non-zero return code, formatted `RC(n)`. Measured: `address sh` then `'exit 3'` under `trace r` prints `+++   "RC(3)"`.
* `RexxActivation.cpp:4024` -- `traceSourceString`, whose only caller is guarded by `inDebug()` at `:4305`. This produces the **first** banner line only.
* `RexxActivation.cpp:4237` -- the debug prompt (`Message_Translations_debug_prompt`), which produces the **second**.
* `Activity.cpp:1496` -- `debug_error`.

All four are Phase 7's: three are interactive debug, one is command dispatch.
**`+++` becomes `Coverage::Owned("Phase 7")`.**

**State the reason as "the emitter is command dispatch and interactive debug, both Phase 7's", not "no 4c construct emits it".**
The weaker phrasing is needed because `AddressInstruction.cpp:163` -- a 4c *instruction* -- is one of `command()`'s two callers.
4c implements `ADDRESS`'s environment tracking and **not** its command dispatch, so the prefix stays out of reach, but the reason is the split inside `ADDRESS`, not the absence of any path.

**The `TRACE ?` row is a gap that has drifted unowned since 4a, and this closes it.**
The `TRACE ?` row in `phase-4-exclusions.txt` -- find it by the phrase "**TRACE ? (the interactive prefix) is silently ignored**" -- records that the oracle emits two `+++` banner lines under `trace ?r` with stdin at `/dev/null`, which this crate does not, and ends "**Owner unassigned**".

**Ruling: `TRACE ?` is Phase 7's, with the rest of interactive debug.**
The evidence that settles it is a probe the first revision did not run: **with non-empty stdin, `trace ?r` drains stdin and issues each line as a shell command** (`/bin/sh: 1: LINE2: not found`), so a following `PULL` reads `""` instead of the line.
**stdout diverges, not only stderr.**
Interactive debug is built on command dispatch -- D18's Phase 7 subsystem -- so reproducing the banner alone (about 30-60 lines) would be byte-exact at `/dev/null` and **wrong on stdout for every `PULL` program**: a new lie, not a partial fix.

Two further corrections Task 1 makes to that row: it is also reached by **`RXTRACE=ON`** with no `TRACE` instruction in the program, and its "only stderr differs" holds **only** at `/dev/null`.

**Consequence for the gate:** 4c's trace-prefix target is **16 of 19**.
The three 4c adds are `>.>`, `>I>` and `<I<`.

### D6, D3, D5, D13, D14 -- carried from 4b unchanged

* **D6.** `rust/corpus/phase-4c.txt` is created beside the other two and the harnesses read the union of all three.
  **`tests/corpus.rs:548-550` hardcodes two paths and must be edited**, or every 4c witness is inert.
* **D3.** No corpus program may contain `DO OVER` on a stem.
  Measured: no in-scope 4c builtin exposes stem traversal order, so it remains the only exposure through the end of Phase 4.
* **D5.** One lane. **Never dispatch two implementers in parallel against this plan.**
* **D13.** 4c gets its own gate document.
* **D14.** The 4b criterion set carries forward with the amendments named in Task 15.

---

## The anti-skew design

4b spent a large share of its commits correcting prose about where the implemented/not-implemented boundary sat.
Measured: a **796-row derived file** that computes those facts at runtime and polices them in both directions needed **zero** corrections across thirteen tasks, while a **one-line count comment** stating the same kind of fact rotted **four times**.

**4c moves that boundary 66 times.** Three structural countermeasures, all in Task 1:

1. **The implemented set is derived by differential comparison against the oracle, never listed and never inferred from a message.**
   The first revision keyed on the absence of a loud message and was defeated three ways; see the Revision note.
2. **One file per family, one task each.** A family task's diff is one file plus dispatch rows.
3. **The four wrong attributions are fixed first**, not swept up at the end.

**The phases are not re-sliced.** 4b's one genuine slicing artifact was an enum whose arms landed in different phases, and the fix was the granularity of the table describing it.
4c's coupling is forced by the language.

---

## File structure

**Created:**

| file | responsibility |
|---|---|
| `crates/rexx-exec/src/builtin/mod.rs` | name -> function dispatch, arity table, the 40.x raisers, the shared in-scope name list |
| `crates/rexx-exec/src/builtin/string.rs` | 23: `ABBREV` `CENTER` `CENTRE` `CHANGESTR` `COMPARE` `COPIES` `COUNTSTR` `DELSTR` `INSERT` `LASTPOS` `LEFT` `LENGTH` `OVERLAY` `POS` `REVERSE` `RIGHT` `SPACE` `STRIP` `SUBSTR` `TRANSLATE` `VERIFY` `LOWER` `UPPER` |
| `crates/rexx-exec/src/builtin/word.rs` | 7: `DELWORD` `SUBWORD` `WORD` `WORDINDEX` `WORDLENGTH` `WORDPOS` `WORDS` |
| `crates/rexx-exec/src/builtin/convert.rs` | 12: `B2X` `BITAND` `BITOR` `BITXOR` `C2D` `C2X` `D2C` `D2X` `X2B` `X2C` `X2D` `XRANGE` |
| `crates/rexx-exec/src/builtin/numeric.rs` | 7: `ABS` `FORMAT` `MAX` `MIN` `RANDOM` `SIGN` `TRUNC` |
| `crates/rexx-exec/src/builtin/datatype.rs` | 4: `DATATYPE` `SYMBOL` `VALUE` `VAR` |
| `crates/rexx-exec/src/builtin/datetime.rs` | 2: `DATE` `TIME` |
| `crates/rexx-exec/src/builtin/state.rs` | 11: `ADDRESS` `ARG` `CONDITION` `DIGITS` `ERRORTEXT` `FORM` `FUZZ` `GC` `QUEUED` `SOURCELINE` `TRACE` |
| `crates/rexx-exec/src/parse_template.rs` | the `PARSE` template engine, source-independent |
| `crates/rexx-exec/tests/builtin_status.rs` | the derived, differential implemented-set harness |
| `crates/rexx-extract/src/bif.rs` | `base/bif` extraction |
| `crates/rexx-exec/tests/bif_assertions.rs` | the `base/bif` L1 harness |
| `rust/corpus/builtin-status.txt` | derived implemented/loud/excluded classification |
| `rust/corpus/builtin-probes.txt` | one meaningful probe program per in-scope builtin |
| `rust/corpus/bif-exempt.txt` | derived `base/bif` exempt set |
| `rust/corpus/phase-4c.txt` | 4c's differential subset |
| `rust/scripts/mutate-4c.sh` | 4c-shaped mutations |
| `docs/superpowers/plans/phase-4c-gate.md` | the gate |

**Verified as a set, not as a sum:** the seven family lists are a true partition of the 66 in-scope builtins -- nothing missing, nothing duplicated, nothing out of scope.

**Modified:** `src/run.rs`, `src/eval.rs`, `src/lib.rs`, `src/plan.rs`, `src/error.rs`, `src/activation.rs`, `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `tests/corpus.rs`, `tests/trace_oracle.rs`, `tests/collect_stress.rs`, `docs/superpowers/plans/phase-4-exclusions.txt`, `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, `corpus/README.md`.

---

## Shared facts every builtin task needs

**This block is a controller obligation, not a task's own text, and that distinction is load-bearing.**
Briefs are extracted per heading, so nothing outside a task's own section reaches its implementer.
Tasks 2, 3, 4, 5, 6, 10, 11 and 12 each say "restated in your brief" -- **the controller makes that true by appending this section to the generated brief file before dispatching.**
An earlier revision said each task "restates this block in their own body", which was false: no task body contained it and the extractor could not supply it.
That is the same mechanism that cost this project five decisions recorded where the implementer never read them.

The append is bounded by this heading and the next `## `, and asserts the block still contains `pub(crate) fn dispatch(`, `not_enough_arguments` and `REXX_CORPUS_GATE=1` before writing -- so a reorganisation of this section fails loudly rather than shipping a brief with a silently truncated span.

**The dispatch interface.**

```rust
pub(crate) fn dispatch(
    interp: &mut Interp,
    name: &[u8],            // already upcased by the caller
    args: &[Option<ObjRef>],
) -> Option<Result<ObjRef, Failure>>
```

`None` means "not a builtin name", which is what lets resolution fall through to `::routine`.
`Some(Err(..))` is a raised condition, including the 40.x family.

**Arguments arrive already evaluated.** `resolve_and_run_call` (`src/run.rs`) evaluates the argument expressions into `Vec<Option<Argument>>`, where `Argument` is `lib.rs:1340`'s private two-variant enum.
Pass `Argument::value()`, which yields `ObjRef`; that method exists for this and its own doc names `ARG()` as a caller.
The `Reference` variant's alias data is `USE ARG >`'s business and no builtin takes a variable reference.
**An omitted position stays `None` rather than being closed up** -- the rule 4b established for `call sub 1,,3`.

**The arity rows** live beside the dispatch as `(min, max)` per name, `max` as `Option<usize>` for the variadic ones.

**The 40.x raisers already half exist.** `error.rs` has `not_enough_arguments` (40.3) and `too_many_arguments` (40.4) from 4b's `USE STRICT ARG`.
**Reuse them; do not write a second pair.**
What is new is the *type* family.

**The argument-error families, measured 2026-08-05 by Task 2. An earlier revision of this block got the third row's family wrong.**

| probe | error | rc |
|---|---|---|
| `substr('abc')` | 40.3 `Not enough arguments in invocation of SUBSTR; minimum expected is 2.` | 216 |
| `substr('abc',,2)` | **40.5** `Missing argument in invocation of SUBSTR; argument 2 is required.` | 216 |
| `substr('abc',2,-1)` | **93.923** `Invalid length argument specified; found "-1".` | **163** |
| `substr('abc','x')` | 40.12 `SUBSTR argument 2 must be a whole number; found "x".` | 216 |
| `substr('abc',2,3,'pq')` | 40.23 `SUBSTR argument 4 must be a single character; found "pq".` | 216 |

**A negative where a non-negative is required is not a 40.x error at all** -- it is **93.923**, "Incorrect call to method", at **rc 163** rather than 216.
An earlier revision listed it among the 40.x probes to take, which would have shipped the wrong code and the wrong exit status across all seven family tasks.
**Probe the family, do not assume it from the neighbouring row.**

**Trailing omitted arguments are not arguments, and this changes the argument model.**
Measured: `q(1,,2,,)` gives `arg()` = **3**, so `length('abc',)` prints `3` while `length(,)` is 40.3.
Only **interior** omissions reach `dispatch` as `None`; trailing ones are dropped before it sees them.

**`check_arity` is a count check, not a shape check, and your builtin must check its own positions.**
`(min, max)` cannot express which positions are required, because required-ness is **conditional on what comes after**.
Measured: `date()` and `date('S')` both succeed, so `DATE`'s minimum is 0 -- yet `date('S',,'S')` is **40.5**, "argument 2 is required", because supplying position 3 makes position 2 mandatory.
The shared machinery will not raise this for you.
**Probe each of your builtins with an interior omission before every optional position**, and raise 40.5 where the oracle does.

**Rexx strings are byte strings, and every probe alphabet must say so.**
Measured at Task 3, and it cost two Critical findings: the error raisers rendered a value through UTF-8, so a byte `>= 0x80` in `found "..."` became U+FFFD where the oracle emits the raw byte, and control bytes stayed raw where the oracle emits `?`.
**A 62,144-program differential sweep reported zero mismatches and could not have found it**: its operand corpus was seven printable-ASCII strings, with zero hex literals and zero bytes above `0x7F`.
Nine committed ooTest cases already reached the defect.

So: **every probe set in this phase includes a byte `>= 0x80`, a control byte, and the null string**, and any sweep reports **the alphabet it drew from** beside the case count.
A count without its alphabet is not a coverage claim -- the same shape as a count without its scan.
This binds Task 5 hardest, since `B2X`, `C2X`, `X2C` and `D2C` are *about* bytes above `0x7F`.

**Never render a Rexx value through `String::from_utf8_lossy` on a path whose bytes are compared.**
It is silent, it is lossy in exactly one direction, and it looks correct in every ASCII test.

**Cross the axes; widening one is not enough.**
Measured at Task 3, and it cost a **silent wrong answer** that two separate corpora both reported clean.
`verify('abcde','','00'x)` is `1` on the oracle and was `0` here, because two C++ branch tests ask *opposite* questions -- an empty reference asks `VERIFY_MATCH`, a non-empty one `VERIFY_NOMATCH`.
Corpus A had **8** empty-reference `verify` programs and no `0x00` option; corpus C had **384** `0x00` options and no empty reference.
**Neither axis was missing. Each corpus varied one and held the other at a safe value**, so the defect at their intersection was invisible to both while their case counts summed to something that looked like coverage.

So a family task's corpus must **cross every argument position's alphabet with every option value**, not vary one position at a time.
And prove the crossing earns its place the way Task 3 did: **run the new corpus against the build that had the bug.**
Its 72 mismatches, against 0 from the two older corpora on that same build, is what turned "this corpus can fail" into "this corpus catches something the others could not".

**Enumerate a builtin's branches from the C++, not from its documentation or from probing.**
Task 3 found 14 empty-argument branches across the string builtins that way.
A branch you did not know exists is one your probes cannot be varied against.

**The ooTest suite lives in *this* repo, not under the oracle.**
`/home/moritz/dev/repos/ooRexx/ootest` **does not exist**; the C++ tree carries no suite at all beyond three stray `.testGroup` files under `extensions/`.
The path is `ootest/ooRexx/base/<group>/` relative to the repository root.
Worth stating because "the oracle is at `/home/moritz/dev/repos/ooRexx`" and "read the ooTest group" sit next to each other in every brief, and an absolute path built from the two is wrong.

**A builtin's test group is not the only place its behaviour is asserted.**
Measured at Task 4: `DELWORD`'s whitespace rule -- the deleted word takes the run *after* it while the run *before* it survives byte for byte, tab-vs-blank identity included -- is asserted in `base/source.file/whiteSpace.testGroup`, **not** in `DELWORD.testGroup`.
So `/bin/grep -a` the whole of `ootest/ooRexx/base/` for your builtin's name, not just its own file.

**And the bytes those cases test are not in the source, which matters to Task 15's extractor.**
Measured: `whiteSpace.testGroup` contains **zero 0x09 bytes**.
`TAB` is a Rexx *variable* -- `TAB = "09"x` at `:63`, `PLANK = " "` at `:66`, `TAB2 = TAB||TAB` -- so the tab exists only in the data at run time.
A scan for whitespace **literals** finds nothing there and silently concludes the tab-separator corpus does not exist.

For a probe author: build separator probes from `"09"x` and the other byte values directly, and never read "no literal tabs in the suite" as evidence about the oracle's separator set.
**For the `base/bif` extractor: whole-body extraction over a file like this needs the variable assignments resolved, not just the assertion lines matched** -- the same class of modelling requirement as `base/expressions` needing the `NUMERIC DIGITS` setting carried forward, and it must be handled or the affected bodies dropped explicitly rather than extracted wrongly.

**Argument *type* and argument *range* are validated in different layers and raise different errors.**
Measured: `word('a b c',1.5)` is **40.12 at rc 216** from the BIF wrapper's integer conversion, while `word('a b c',0)` and `word('a b c',-12)` are **93.924 at rc 163** from the String method's `positionArgument`.
Different number, different exit code, same argument.
**Probe a bad *type* and a bad *range* for every numeric position you take** -- a probe set testing only one kind cannot see the other.

**Validation order is observable, and the C++ order is often structural rather than intended.**
Measured: `subword('SUBWORD','30'x,'30'x)` -- position 0 *and* length 0 -- raises 93.924 rather than returning `''`, because `positionArgument` is called before the `count == 0` test.
An implementation that early-returns on a zero length gets it wrong at rc 0.
Read the function body for the order; it is documented nowhere else.

**Measure whether a 40.12 or 40.23 message substitutes the rendered value or the source spelling.**
The neighbouring 88.928 raiser in `error.rs` documents having measured exactly this distinction, and it is invisible until an argument's two forms differ -- `'007'` against `7`, or a number whose `DIGITS` rendering is not its literal text.
Task 2 did not record which it is, and the first family task that raises a typed error owes the measurement.

**A quoted literal target reaches the builtin table, and it is case-sensitive.**
Measured: `"LENGTH"('abc')` is `3`; `"length"('abc')` is **43.1 rc 213**, `Could not find routine "length"`.
So the caller upcases a *symbol* target and does **not** upcase a quoted one -- `dispatch` receives the name already upcased only on the symbol path.

**Allocation.** Every builtin returning a string allocates, and every such site goes through `Interp::alloc_with`, never `Heap::alloc_with_uncollected` or `Heap::alloc`.
**A builtin's result must be rooted before any subsequent allocation.**

**D15.** A value's rendering is fixed when the value is created.
A builtin producing a number captures the `DIGITS`/`FORM` pair in force at creation; formatting it later with `settings.digits()` is wrong.
**A probe cannot see this unless `DIGITS` or `FORM` changes between creation and rendering** -- construct at least one probe per numeric builtin that does.

**Probe safety, restated because these are the two that bite this work:**

* **Run every probe from a fresh empty subdirectory of the scratchpad, with absolute paths.**
  The scratchpad root is on the oracle's external-routine search path.
  A probe of a not-yet-implemented builtin name reaches exactly that search, and a stale `.rex` file will be found and run.
  Measured: 44.1 rc 212 from the root against 43.1 rc 213 from a clean directory -- different error, different rc, different meaning.
* **Wrap every oracle call** as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx ABSOLUTE_PATH )`.

**The verify block every task ends with**, and no task may substitute a bare "run the tests":

```bash
cd rust
cargo test --offline --workspace --no-fail-fast
cargo fmt --all --check
cargo clippy --offline --workspace --all-targets -- -D warnings
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus
```

Read each exit status **unpiped**.
`REXX_CORPUS_GATE=1` matters: without it `corpus.rs` reports mismatches and still passes (`!gate || mismatches.is_empty()`), so a builtin that diverges byte-for-byte from the oracle leaves `cargo test --workspace` green.

---

## Task list

| # | task | why here |
|---|---|---|
| 1 | Boundary infrastructure and the four attribution fixes | before anything moves the boundary |
| 2 | Builtin dispatch, arity table, the 40.x family | every family task depends on it |
| 3 | `builtin/string.rs` (23) | free |
| 4 | `builtin/word.rs` (7) | free |
| 5 | `builtin/convert.rs` (12) | free |
| 6 | `builtin/numeric.rs` (7) | free |
| 7 | `PARSE` engine and the five non-queue sources | needs nothing from 2-6 |
| 8 | Program arguments, `ARG`, `PULL`, `PARSE PULL`, `PARSE LINEIN` | needs 7's engine and 4b's queue |
| 9 | `ADDRESS` instruction and environment tracking | before `ADDRESS()` |
| 10 | `builtin/state.rs` (11) | needs 8 and 9 |
| 11 | `builtin/datatype.rs` (4) | free |
| 12 | `builtin/datetime.rs` (2) | isolated because `TIME('R')` is stateful |
| 13 | `::routine` dispatch, `>I>`/`<I<` | needs the builtin table complete, for shadowing |
| 14 | The compound-`DO` control-variable fix | independent; late so its L1 rows move once |
| 15 | `base/bif` L1 harness, the 4c subset, `mutate-4c.sh`, the gate | measures everything above |

---

### Task 1: Boundary infrastructure and the four attribution fixes

**Files:**
* Create: `crates/rexx-exec/tests/builtin_status.rs`, `rust/corpus/builtin-status.txt`, `rust/corpus/builtin-probes.txt`
* Modify: `crates/rexx-exec/tests/trace_oracle.rs`, `docs/superpowers/plans/phase-4-exclusions.txt`, `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`

**Interfaces produced:** `corpus/builtin-status.txt`, one row per name in `rexx_inventory::builtins::NAMES` order, `NAME<TAB>STATUS` with `STATUS` in `implemented` / `loud` / `divergent` / `excluded`.
Every later task's obligation to this file is to re-run the harness and commit the rows it flips.

**Why first.** Tasks 2-14 each move the boundary. This builds the one place that records where it sits, and fixes the four places that record it wrongly.

- [ ] **Step 1: Write `corpus/builtin-probes.txt`**

One line per **in-scope** builtin: the name, a tab, and a **meaningful** one-line Rexx program that calls it and prints a non-empty result -- `say substr('abcdef',2,3)`, not `say substr()`.

**Why meaningful and not zero-argument.** A zero-argument probe compares two error messages.
That is a real comparison but it tests the 40.x family rather than the builtin, and it is satisfied by a builtin that exists and computes nothing.
The gate criterion this file underwrites is "assert a *value* per builtin, captured from the oracle", and a value is what this file must elicit.

66 rows. The 15 whole exclusions get no probe.

- [ ] **Step 2: Write the differential status harness**

For each name in `NAMES`:

1. **If the name is one of the 15 whole exclusions, classify `excluded` and run nothing.**
   Derive the 15 as `EXCLUDED_BUILTINS` minus the three partial rows (`VALUE`, `ADDRESS`, `QUEUED`), which are **in scope and must be probed**.
   Writing "in `EXCLUDED_BUILTINS`" gives 18 and makes the count assertions in Step 3 fail on a correct implementation.
2. Otherwise run its probe through **both** `build/bin/rexx` and `run_program`, comparing stdout, stderr and exit status as three descriptors.
   * ours exits `NOT_IMPLEMENTED_EXIT` -> `loud`
   * all three match the oracle -> `implemented`
   * anything else -> `divergent`

**`EXCLUDED_BUILTINS` is a private `const` in `tests/coverage.rs` and cannot be reached from another test binary or from `src/`.**
Move it to `crates/rexx-inventory/src/lib.rs` as `pub const EXCLUDED: &[&str]` alongside `NAMES`, and have `coverage.rs` read it from there.
That is this task's one production-side edit and it is what makes the list shareable with Task 2's dispatch.

**Reuse `corpus.rs`'s oracle machinery** -- `oracle_root()`, the `ulimit` wrapper, the three-descriptor comparison -- rather than writing a second copy.
It already fails hard when the oracle binary is absent, which is the guard that stops a vacuous "0 of 0".

- [ ] **Step 3: Assert, including that the harness ran**

Four assertions, and the third is the one that stops a classifier that consults only name tables:

1. The derived set equals the committed file, **in both directions**, with messages that say which way it went.
2. `excluded` is exactly 15 and the total is 81, so `implemented + loud + divergent == 66`.
3. **The oracle was invoked exactly 66 times.** Count the invocations and assert the count.
   Without this, a classifier of the form `if EXCLUDED.contains(n) {excluded} else if DISPATCHED.contains(n) {implemented} else {loud}` satisfies every other check while running no program at all.
4. `divergent` is empty unless the row is committed as such.
   A divergent builtin is a defect, not a status; committing one requires a `KNOWN GAP` row naming it.

**One consequence for Task 13, recorded here because this file is where it is visible.**
`divergent` is the classification for "ran, and disagreed with the oracle".
While the loud fallback exists, an unimplemented name exits `NOT_IMPLEMENTED_EXIT` and classifies `loud`.
**When Task 13 replaces that fallback with a real 43.1 raise, every still-unimplemented name stops being `loud` and becomes `divergent`** -- 43.1 is a plausible Rexx condition, not a loud exit, so the harness can no longer tell it from a wrong answer.
The failure is loud rather than silent, which is the design working, but it means **Task 13 must land after every builtin task**, and any name still unimplemented at that point needs its own `KNOWN GAP` row.
The task order already places Task 13 after Tasks 2-12; this is why, and it is not free to reorder.

- [ ] **Step 4: Falsify it three ways, two of them against the interpreter**

The first revision falsified only by editing the committed file, which cannot detect a classifier that never runs anything.

1. Delete a row from `corpus/builtin-status.txt`; the test must fail **by name**.
2. Hand-edit a row from `loud` to `implemented`; the other direction must fail.
3. **Mutate the interpreter, not the data:** at Task 2's completion this step is re-run by deleting `LENGTH`'s dispatch arm and confirming its row flips `implemented` -> `loud` on its own.
   Record in this task that Step 4.3 is owed by Task 2, because it cannot run before a builtin exists.

- [ ] **Step 5: Fix `+++`'s owner (D-P)**

In `tests/trace_oracle.rs`, change `("+++", Coverage::Owned("4c"))` at `:529` to `Coverage::Owned("Phase 7")`.
`OWNER_PHASES` already admits `"Phase 7"`.
**`WITNESSED_PREFIX_COUNT` and `OUT_OF_SCOPE_PREFIX_COUNT` at `:551` and `:555` are not touched here** -- the prefix moves owner, not witnessed-ness.

In `phase-4-exclusions.txt`, correct the paragraph containing "**Four of the six -- +++ and >.> (4c)**". The corrected statement, with its evidence:

> `+++` is Phase 7's. A `+++`-prefixed line has four producers in the C++:
> `RexxActivation.cpp:4468`, a command's non-zero `RC`, measured live as
> `+++   "RC(3)"` after `address sh` and `'exit 3'`; `:4024`
> (`traceSourceString`, guarded by `inDebug()` at `:4305`), the first banner
> line; `:4237`, the debug prompt, the second; and `Activity.cpp:1496`,
> `debug_error`. Three are interactive debug and one is command dispatch,
> both Phase 7's under D18 and under the `TRACE ?` row below. The reason is
> that split rather than the absence of any path: `AddressInstruction.cpp
> :163` is a 4c instruction and one of `command()`'s two callers, so it is
> `ADDRESS`'s own 4c/Phase 7 division that keeps the prefix out of reach.

- [ ] **Step 6: Give the `TRACE ?` row an owner and correct two of its claims (D-P)**

Replace that row's "Owner unassigned" with `Owner: Phase 7, with the rest of interactive debug.`, and add:

* the reason -- measured, **with non-empty stdin `trace ?r` drains stdin and issues each line as a shell command**, so a following `PULL` reads `""`; reproducing the banner alone would be byte-exact at `/dev/null` and wrong on stdout for every `PULL` program;
* that the row's "only stderr differs" holds **only** at `/dev/null`;
* that the same path is reached by **`RXTRACE=ON`** with no `TRACE` instruction in the program.

**This row and Step 5's constant land in the same commit.** The assertion is what stops the row drifting a third time.

- [ ] **Step 7: Amend the design spec and correct the `::routine` citations (D-R)**

In `2026-07-30-phase-4a-executor-design.md:71`, change "every directive" to "every directive except `::ROUTINE`, which is 4c's (see the 4c plan's D-R)".

In `phase-4-exclusions.txt`, add one line to the `QualifiedCall` row noting that its "every directive" citation now carries the `::ROUTINE` carve-out, and that `QualifiedCall` is unaffected because namespaces come from `::REQUIRES`.
Leave the `>I>`/`<I<` row's own wording alone: both of its ownership sentences are correct as written (see D-R).

- [ ] **Step 8: Add the D4 reason sentences**

One sentence per excluded row in `phase-4-exclusions.txt` saying why it is blocked, with the three non-obvious ones (`USERID`, `SETLOCAL`, `ENDLOCAL`) as D4 states them.

- [ ] **Step 9: Verify and commit**

Run the shared verify block. Stage exactly the paths this task names.

---

### Task 2: Builtin dispatch, the arity table, and the 40.x error family

**Files:**
* Create: `crates/rexx-exec/src/builtin/mod.rs`, `crates/rexx-exec/src/builtin/string.rs` (holding `LENGTH` alone)
* Modify: `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/lib.rs`, `crates/rexx-exec/src/error.rs`, `rust/corpus/builtin-status.txt`

**Read "Shared facts every builtin task needs" -- it is restated in your brief and carries the dispatch signature, the argument model, the existing 40.x raisers, the allocation rule, the probe safety rules and the verify block.**

- [ ] **Step 1: Measure the 40.x family**

Measured 2026-08-04, rc **216** in every case:

```
     1 *-* say substr('abc')
Error 40 running <path> line 1:  Incorrect call to routine.
Error 40.3:  Not enough arguments in invocation of SUBSTR; minimum expected is 2.
```

| probe | sub-code | secondary text |
|---|---|---|
| `substr('abc')` | 40.3 | `Not enough arguments in invocation of SUBSTR; minimum expected is 2.` |
| `substr('abc','x')` | 40.12 | `SUBSTR argument 2 must be a whole number; found "x".` |
| `substr('abc',2,3,'pq')` | 40.23 | `SUBSTR argument 4 must be a single character; found "pq".` |

Probe further, because guessing a sub-code ships a whole family wrong: too many arguments; a missing *required* argument in a middle position (`substr('abc',,2)`, which is **40.5**, not 40.3); a negative where non-negative is required (**93.923 at rc 163**, not a 40.x error at all); and the same type error on a builtin **not** named `SUBSTR`, to confirm the name is interpolated.
The name in the message is uppercased while the `*-*` echo carries the source spelling -- confirm both.
The full measured table is in "Shared facts every builtin task needs".

- [ ] **Step 2: Write the failing tests**

`say length('abc')` prints `3`, exit 0; and `say length()` produces the measured 40.3 bytes.
Both fail: `dispatch` does not exist.

- [ ] **Step 3: Build `builtin/mod.rs` and hook it in at the right place**

**The hook goes *after* argument evaluation, not at the label-lookup fallback.**
`run.rs`'s `resolve_and_run_call` looks up the label, returns `Loud::unresolved_call` if absent, and evaluates the argument expressions **after** that point.
The first revision put the builtin step between the lookup and the loud fallback, which is upstream of the evaluation it claimed to consume unchanged.
Restructure so the name resolves to one of three outcomes *before* the loud return, then evaluate arguments once, then dispatch.

**Answer these three for the builtin path explicitly, because the label path answers them and the builtin path must not inherit the answers by accident:** whether `SIGL` is set, whether `>A>` argument trace lines fire, and whether the activation depth counter increments.
Measure each on the oracle and state the answer in the code's own doc comment.

The name set the dispatch covers is `NAMES` minus the **15 whole exclusions**, which is **66** -- not `NAMES` minus `EXCLUDED_BUILTINS`, which is 63, because `VALUE`, `ADDRESS` and `QUEUED` are partial rows that must dispatch.
Read the list from `rexx_inventory` (Task 1 moved it there); do not copy it.

**Only `LENGTH` is implemented here**, and it goes straight into `builtin/string.rs`, which this task creates.
A one-builtin file is not a placeholder: `dispatch` needs one real name to prove the chain end to end, and staging it through `mod.rs` buys a rename and a diff that says nothing.

- [ ] **Step 4: Leave the loud fallback in place**

A name that is neither a label nor a builtin still returns `Loud::unresolved_call`. Task 13 replaces it.

- [ ] **Step 5: Discharge Task 1's Step 4.3**

Delete `LENGTH`'s dispatch arm, run `builtin_status.rs`, and confirm its row flips `implemented` -> `loud` on its own.
Restore from a copy, not `git checkout --`, and confirm `git status` is clean.
**This is the falsification that proves the status harness observes the interpreter rather than a name table**, and Task 1 could not run it.

- [ ] **Step 6: Re-run the status harness, commit the flipped row, verify and commit**

---

### Task 3: `builtin/string.rs`

**Files:** modify `crates/rexx-exec/src/builtin/string.rs` (created by Task 2, holding `LENGTH`), `crates/rexx-exec/src/builtin/mod.rs` (dispatch and arity rows only), `rust/corpus/builtin-status.txt`.

**The 22 names to add:** `ABBREV CENTER CENTRE CHANGESTR COMPARE COPIES COUNTSTR DELSTR INSERT LASTPOS LEFT OVERLAY POS REVERSE RIGHT SPACE STRIP SUBSTR TRANSLATE VERIFY LOWER UPPER`.
`LENGTH` is already here and is not rewritten.

**Read "Shared facts every builtin task needs", restated in your brief.**

- [ ] **Step 1: Build the probe table before writing any code**

For each name: the documented base case; **every optional argument position separately** (a one-argument probe cannot distinguish a builtin that honours its optional arguments from one that ignores them); at least one probe with a **non-default pad character** where one is taken; the empty string and the `f(,2)` omitted-position forms; and the boundary values the ooTest group uses.

**Run each probe from a fresh empty scratchpad subdirectory with absolute paths.**
A probe run from the scratchpad root can return a stale file's output -- measured, 44.1 rc 212 against the true 43.1 rc 213 -- and a wrong expected value transcribed here becomes an implementation written to match it, with test and code agreeing forever.

- [ ] **Step 2: Read `ootest/ooRexx/base/bif/<NAME>.testGroup` for each name**

It is the reference for edge cases and the same file Task 15's L1 harness will run.
A case it covers that the implementation misses surfaces at Task 15 as an exempt row that should not be there.
`STRIP` and `VERIFY` take option letters -- probe each letter and an invalid one.
`CENTER` and `CENTRE` are one function under two names and dispatch to one implementation.

- [ ] **Step 3: Write failing tests from the table, then implement**

- [ ] **Step 4: Re-run the status harness and assert this family's completeness**

All 23 of this file's names must read `implemented`, and none may read `divergent`.
**Assert the family's list specifically**, not just that the file validates: without it, implementing 20 of 23 and committing 20 flipped rows is green.

- [ ] **Step 5: Run the shared verify block and commit**

---

### Task 4: `builtin/word.rs`

**Files:** create `crates/rexx-exec/src/builtin/word.rs`; modify `builtin/mod.rs`, `rust/corpus/builtin-status.txt`.

**The 7 names:** `DELWORD SUBWORD WORD WORDINDEX WORDLENGTH WORDPOS WORDS`.

**Read "Shared facts every builtin task needs", restated in your brief.**

- [ ] **Step 1: Build the probe table** -- as Task 3 Step 1, from a fresh empty scratchpad subdirectory with absolute paths.

The blank-delimiter rule is shared by all seven; factor it once.
Probe leading, trailing and repeated blanks, and tabs.

- [ ] **Step 2: Read each `<NAME>.testGroup`**

- [ ] **Step 3: Write failing tests, then implement**

- [ ] **Step 4: Re-run the status harness; assert all 7 read `implemented` and none `divergent`**

- [ ] **Step 5: Run the shared verify block and commit**

---

### Task 5: `builtin/convert.rs`

**Files:** create `crates/rexx-exec/src/builtin/convert.rs`; modify `builtin/mod.rs`, `rust/corpus/builtin-status.txt`.

**The 12 names:** `B2X BITAND BITOR BITXOR C2D C2X D2C D2X X2B X2C X2D XRANGE`.

**Read "Shared facts every builtin task needs", restated in your brief.**

- [ ] **Step 0: Eight measured behaviours the reference does not give you -- read before writing any code**

Surveyed and re-verified 2026-08-05. Each is a **silent wrong answer** if missed, and the obvious implementation is wrong for most of them.

**(a) `NUMERIC DIGITS` bounds the RESULT, not the input length.** This is the one most likely to be got wrong in both directions:

```rexx
numeric digits 9;  say c2d(copies('00'x,10)||'01'x)   -- 1,  eleven bytes, fine
numeric digits 9;  say c2d('ffffffff'x)               -- 93.936, four bytes
numeric digits 1;  say c2d('ff'x,1)                   -- -1, fits
numeric digits 1;  say c2d('7f'x,1)                   -- 93.936, 127 does not
```

Eleven bytes succeed and four fail under the same setting.
`C2D`/`X2D` are sensitive on **output** (93.936 / 93.935); `D2X`/`D2C` on **input** (93.928 / 93.929), where the *argument* must be a valid whole number under the current setting before conversion starts.
For all-`FF` input the bound is `floor(DIGITS / log10(256))` -- so **`DIGITS 1` admits zero bytes**, since one byte is already 255.
Both message texts name the setting.

**(b) The length argument is a right-aligned window that truncates from the LEFT, silently.**
`c2d('01020304'x,2)` is **772** (`0x0304`), `d2x(4096,2)` is `00`, `x2d('80',1)` is `0`. **No error.** An implementation that raises one breaks all of them.

**(c) The length argument also switches the read to SIGNED, and the window sets the sign bit.**

| | no length | `,1` | `,2` |
|---|---|---|---|
| `c2d('80'x)` | 128 | **-128** | 128 |
| `x2d('80')` | 128 | **0** | **-128** |

Same bytes, three answers, and **`C2D` and `X2D` disagree with each other**.
`d2x(-1)` and `d2c(-1)` without a length are 93.927, "Length must be specified to convert a negative value."

**(d) `BITAND`/`BITOR`/`BITXOR` with unequal lengths and no pad pass the longer string's tail through UNCHANGED.**
`c2x(bitand('ffff'x,'00'x))` is **`00FF`** -- the tail survives. Supply a pad and it is combined: `c2x(bitand('ffff'x,'00'x,'00'x))` is `0000`. One argument is legal: `bitand('ffff'x)` is `FFFF`.

**The mechanism is a default pad equal to the operation's IDENTITY element** -- `0xff` for `BITAND`, `0x00` for `BITOR` and `BITXOR` -- which is *why* the tail survives.
Measured: `bitor('ffff'x,'00'x)` and `bitxor('ffff'x,'00'x)` are both `FFFF`, and `bitand('0000'x,'ff'x)` is `0000`.
Defaulting `BITAND`'s pad to `'00'x` is the obvious implementation and it is wrong; defaulting it to `'ff'x` is right.
*An earlier revision of this step said "there is no default pad, there is a passthrough" -- right about the behaviour, wrong about the mechanism, and wrong in a way that misleads anyone reading the C++.*

**(e) Hex and binary string whitespace is `{0x20, 0x09}` -- blank and tab only**, the same set as the word separators. `x2c('41'||'09'x||'42')` is `4142`; LF is 93.933. Leading or trailing whitespace is 93.931.

**(f) Grouping, read from `StringUtil::validateGroupedSet`: the scanner keeps a CUMULATIVE digit total**, records `total % modulus` as a residue at the first whitespace run, and requires that same residue at every later run and at end of string.
Modulus is 2 for hex, 4 for binary; the first group is left-padded rather than rejected.
`x2c('414')` is `0414`; `x2c('4 1424')` is `041424`; `x2c('414 2434')` is `04142434`; `x2c('414 243')` is 93.976.
*Outcome-equivalent to "the first group sets the residue and later groups are exact multiples", which is how an earlier revision inferred it from eight cases -- but the state is a running total, not a per-group check, and an implementation written from the inferred rule will diverge on inputs the eight cases did not reach.*

**(g) `X2C` and `X2B` disagree on odd input.** `x2c('414')` pads to a whole byte (`0414`); `x2b('414')` gives 12 bits, unpadded. `b2x` pads to a multiple of 4 bits.

**(h) `XRANGE` is variadic over PAIRS and a class name consumes one slot.**
`xrange('a','b','c','d')` is `abcd` -- two ranges concatenated. `length(xrange())` is 256.

**`xrange('digit','z')` is 134 bytes, `0x7A` through `0xFF`, and the digits are DISCARDED** -- not prepended, not a range from `0`.
Measured: the result begins `7A7B7C7D`, which is `0x7A..0xFF` exactly.
`BUILTIN(XRANGE)`'s `argcount <= 2` early return throws away everything accumulated before the final pair; **three arguments keep them** (`length(xrange('digit','z','a','c'))` is 399).
*An earlier revision said `'z'` "starts a new range to `0xFF`", which reads as though the digits survive in front of it. They do not.*
The 12 POSIX class names are case-insensitive (`BuiltinFunctions.cpp:1639-1648`), and **`cntrl` contains a leading NUL** -- `length(xrange('cntrl'))` is 33 and it begins `00010203`, so anything using `strlen` truncates it to nothing.
Argument asymmetry: argument 1 takes a class name **or** a single character (40.28); argument 2 takes a single character **only** (40.23).

**Validation order: every `40.x` argument-conversion check precedes every `93.9xx` content check, on all arguments.**
`d2c('abc','def')` is **40.12**, not 93.929 -- the *length*'s type error beats the *value*'s. `x2d('ZZ','zz')` is 40.12 while `x2d('ZZ',4)` is 93.933.
`40.x` is rc **216**; `93.9xx` is rc **163**.

**But within the `93.9xx` family the two pairs order oppositely, so there is no single rule to carry.**
Measured: `d2c('abc',-1)` is **93.929** -- `D2X`/`D2C` check the *value* before the *length*, where `C2D`/`X2D` do the reverse.
Establish the order per builtin rather than per family.

**A `(min, max)` of `(1, 1)` can never reach 40.5.** `b2x(,'x')` is **40.4**, too many arguments, because the omitted first position is still a position. Do not write a 40.5 path for a single-argument builtin.

**What the suite checks, so you know what it cannot catch.** 212 `expectSyntax` calls over 17 distinct numbers across the twelve groups, against 899 `assertSame`.
Gaps, each established with a positive control: **`93.977` (binary grouping) is raised by the implementation and tested nowhere** in `ootest/base`, though its hex twin 93.976 is tested in four places; `BITAND`/`BITOR` test exactly one error number each and nothing asserts their 40.4 at four arguments; `C2X` tests only 40.4.

**Two behaviours are asserted outside these groups**: `class/RexxInteger.testGroup:276-359` requires `d2x`/`c2d`/`x2d` to return a **RexxInteger** and `RexxInteger~d2x` to equal `NumberString~d2x`; `expressions/Literals.testGroup:162` ties `.String~xdigit~x2c` to the character classes.

**A probe warning from this survey, arriving from an unexpected direction.** `bitor ('0000'x,...)` **with a space** parses as concatenation with the uninitialised symbol `BITOR`, and produces plausible-looking output (`4249544F52…` is `"BITOR "`). The literal-syntax trap is not confined to `x` and `b`; any builtin name followed by a space and a parenthesis is a symbol, not a call.

- [ ] **Step 1: Build the probe table, with two hazards specific to this family**

* **Rexx literal syntax will silently change your probe.** A symbol named `x` or `b` immediately followed by a quoted string parses as a hex or binary literal.
  `say x2d('ff')` is fine; `y = 'ff'; say x2d(y)` is fine; `say x'41'` is a *literal*, not a call.
  This family is where that bites.
* **`NUMERIC DIGITS` interacts with `C2D`, `D2C`, `D2X` and `X2D`.** Probe each at a non-default `DIGITS`, and **never above 1000**.

Also: the bit builtins take a pad character; `XRANGE`'s arguments are single characters, so 40.23 is its live error.

**These are the six groups whose `.testGroup` files contain non-UTF-8 bytes** (`C2X`, `COPIES`, `D2C`, `DATATYPE`, `DELSTR`, `INSERT`), so use `/bin/grep -a` when searching them and expect high bytes in the expected values.

- [ ] **Step 2: Read each `<NAME>.testGroup`**

- [ ] **Step 3: Write failing tests, then implement**

- [ ] **Step 4: Re-run the status harness; assert all 12 read `implemented` and none `divergent`**

- [ ] **Step 5: Run the shared verify block and commit**

---

### Task 6: `builtin/numeric.rs`

**Files:** create `crates/rexx-exec/src/builtin/numeric.rs`; modify `builtin/mod.rs`, `rust/corpus/builtin-status.txt`.

**The 7 names:** `ABS FORMAT MAX MIN RANDOM SIGN TRUNC`.

**Read "Shared facts every builtin task needs", restated in your brief -- in particular D15, which this family is the one most exposed to.**

- [ ] **Step 0: Measured behaviours the reference does not give you -- read before writing any code**

Surveyed and re-verified 2026-08-05. **Three rc values in this family where earlier ones had two:** `40.x` -> 216, `93.x` -> 163, **`41.1` -> 215**.

**(a) `MAX`/`MIN` have ZERO error coverage in the whole suite, and their error depends on argument POSITION.**
Zero `expectSyntax` in `bif/MAX.testGroup`, `bif/MIN.testGroup`, `class/String/max.testGroup` or `class/String/min.testGroup` -- verified with a positive control, since 93.903/93.904 *are* asserted at `directives/METHOD.testGroup:278` and `keyword/VarRef.testGroup:127,143`. **Your probes are the only evidence for all of it.**

```
max()          -> 40.3
max(5)         -> 5
max('a',1,3)   -> 93.943   MAX method target must be a number; found "a".
max(1,'a',3)   -> 93.904   Method argument 1 must be a number; found "a".
max(1,2,'a')   -> 93.904
max(1,,3)      -> 93.903   Missing argument in method; argument 0 is required.
```

Three things to get right: **argument 1 raises a different number from arguments 2+**; the `93.904` insert is **off by one against Rexx's own numbering** (the bad value in `max(1,'a',3)` is argument *2* and the message says *1*); and `93.903` says **"argument 0 is required"**, literally zero, measured rather than mistranscribed.

*Flagged as inference, not measurement:* the "method target" wording suggests dispatch as `arg1~max(arg2,…)`, which would explain both the split and the off-by-one. The behaviour is measured; **the mechanism is unconfirmed in the C++ -- read it rather than trusting this.**

**(b) `41.1` is never these builtins' own error.** `SIGN.testGroup`'s two `41.1` cases raise *before* `SIGN` is entered:

```
sign(-1E1234567890)   -> 41.1    rc 215   the unary minus is arithmetic, raised first
sign('-1E1234567890') -> 93.943  rc 163   quoted, no arithmetic, reaches SIGN
abs(17+'c')           -> 41.1    rc 215   the addition raises; nothing to do with ABS
```

**Wiring `41.1` into `SIGN` because its test group asserts it is wrong.**

**(c) `FORMAT` rounds half-up away from zero, not banker's.** `format(2.5,,0)` is **3**, `format(3.5,,0)` is 4, `format(-2.5,,0)` is -3, `format(1.245,,2)` is 1.25.
A banker's-rounding implementation gives 2 for the first.

**(d) `FORMAT`'s `before=0` always fails, even for zero.** `format(1,0)` and `format(0,0)` are both **93.942**, "Integer part of "0" is too large for 0 spaces" -- while `format(0)` is `0`.

**(e) `expp=0` suppresses exponential and BEATS `expt=0`, which forces it.**

```
format(12345,,,0)     = 12345          format(12345,,,,0)   = 1.2345E+4
format(12345,,,0,0)   = 12345          format(12345,,,2,0)  = 1.2345E+04
format(12345,,,4,0)   = 1.2345E+0004   format(1e10,,,,20)   = 10000000000
```

`FORMAT` also honours `NUMERIC FORM`: `format(1e10,,,,0)` is `1E+10` under SCIENTIFIC and `10E+9` under ENGINEERING.
All four optional arguments reject negatives with 93.906, and `format(1,,,,)` with every optional explicitly omitted is legal at rc 0.

**(f) `TRUNC` rounds its input to `DIGITS` FIRST, and never raises LOSTDIGITS.**
Measured: `numeric digits 3; trunc(123456,2)` is **123000.00**. `trunc(1e20)` at digits 9 is `100000000000000000000` -- never exponential.
`keyword/LOSTDIGITS.testGroup:388-391` asserts TRUNC does **not** raise LOSTDIGITS, with the reason in a source comment: the arithmetic builtins round their arguments before processing. **Nothing in `TRUNC.testGroup` says either thing.**

**(g) `RANDOM`'s negative first argument is 40.33, not 40.13.**
`random(-1)` is **40.33**, "RANDOM argument 1 ("-1") must be less than or equal to argument 2 ("")" -- and argument 2's insert is the **empty string** because it was omitted. The zero-or-positive argument (40.13) is the **seed**: `random(1,2,-1)`.
Degenerate ranges are legal: `random(5,5)` is 5, `random(0,0)` is 0. `random(5,1)` is 40.33; `random(1.5)` is 40.12.

**(h) Validation order, measured for `TRUNC` and `FORMAT` only: argument-2 TYPE > argument-1 TARGET > argument-2 RANGE.**

```
trunc('AB.CD','V') -> 40.12     format(1,'x')  -> 40.12
trunc('AB.CD',-1)  -> 93.943    format('a',-1) -> 93.943
trunc(1.5,-1)      -> 93.906    format(1,-1)   -> 93.906
```

*Flagged:* measured for those two only. No value-before-length inversion was found like `D2X`/`D2C`'s, but the analogous probe could not be constructed here -- **establish the order per builtin, as Task 5 had to.**

**(i) D15 holds across `FORM` as well as `DIGITS`.** A value created under `DIGITS 9`/`SCIENTIFIC` keeps `1.23456789E+11` after either setting changes; recomputing gives `123456789012` or `123.456789E+9`.
The exponential trigger is `exp >= DIGITS` positive and `exp >= 2*DIGITS+1` negative -- *flagged: a fit to four data points, not read from the source.*

**What the suite checks:** 181 `expectSyntax` calls, 8 distinct numbers, against 876 `assertSame`. `111×93.942 · 34×93.906 · 19×93.943 · 11×40.12 · 2×41.1 · 1×40.33 · 1×40.3 · 1×40.13`. Per group: ABS `40.3 93.943` · FORMAT `93.906 93.942` · **MAX none** · **MIN none** · RANDOM `40.12 40.13 40.33` · SIGN `41.1 93.943` (both of which do not exercise SIGN) · TRUNC `40.12 93.943`.

**Cross-file, and one false lead killed:** `class/RexxInteger.testGroup:301-307` requires `number~trunc` to be a RexxInteger equal to `integer~trunc(0)`, and `:415` requires MIN's result to contain no `E`; neither is in `TRUNC.testGroup`.
**`bif/DATE.testGroup` has 725 word-boundary hits for `FORMAT` and zero `format(` calls** -- they are the English word and the date-option name. **A bare name grep is badly misleading for `FORMAT` and `MIN`; require the `(`.**

- [ ] **Step 1: Build the probe table, including at least one D15 probe per numeric builtin**

Every result here obeys D15: a value's rendering is fixed when the value is created.
**A probe cannot see a D15 violation unless `DIGITS` or `FORM` changes between the value's creation and its rendering**, so each numeric builtin needs at least one probe that changes one of them in between.

`FORMAT` is the largest single builtin in the group -- `FORMAT.testGroup` has **767** `::method` bodies, the biggest in `base/bif` -- so budget for it.
Never set `NUMERIC DIGITS` above 1000.

- [ ] **Step 2: `RANDOM`'s requirement is a stream, not a seed**

`RANDOM.testGroup` seeds once and makes **99 further unseeded calls**, re-seeds, repeats, and requires all 100 to match.
**A generator that re-seeds on every call satisfies "seedable and deterministic" and fails this.**
Pin the stream property in a unit test.
`RANDOM` must **not** appear in any corpus program (D11), and the group's 8 `expectSyntax` cases (six 40.12, one 40.13, one 40.33) pin its argument validation.

- [ ] **Step 3: Read each `<NAME>.testGroup`, write failing tests, then implement**

- [ ] **Step 4: Re-run the status harness; assert all 7 read `implemented` and none `divergent`**

- [ ] **Step 5: Run the shared verify block and commit**

---

### Task 7: the `PARSE` template engine

**Files:**
* Create: `crates/rexx-exec/src/parse_template.rs`
* Modify: `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/lib.rs` (`instruction_owner`, `:758`), `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `tests/trace_oracle.rs`, `rust/corpus/phase-4c.txt`

**`rust/corpus/phase-4c.txt` does not exist yet. This task creates it**, and `tests/corpus.rs` does not read it until Task 15 Step 4 -- so this task's witness is committed but inert, and that is expected rather than a defect.

**The AST is complete and this task writes no parser code.**
`rexx_parse::ast::Parse` carries `source`, `upper`, `lower`, `caseless`, and `template: Vec<Option<ParseTrigger>>` where `None` is the comma fence between templates.
`ParseTrigger` carries `kind`, an optional `value`, and `targets: Vec<Option<Expr>>` where `None` is a `.` placeholder.
`TriggerKind` has all eight variants: `End`, `Plus`, `Minus`, `Absolute`, `MinusLength`, `PlusLength`, `String`, `Mixed`.

**Sources in this task:** `Var`, `Value`, `Arg`, `Source`, `Version`. `Pull` and `LineIn` are Task 8's.

**`PARSE ARG` here means the argument strings of the *current activation*, which 4b built.
A top-level program's own argument string does not exist yet and is Task 8's.**

- [ ] **Step 0: The template semantics, measured -- read before writing any engine code**

Surveyed and re-verified 2026-08-05. Source is `'abcdefghij'` throughout.

**(a) `Minus` and `MinusLength` are not variants of each other, and conflating them is the likeliest engine bug.**

```
parse value 'abcdefghij' with p 5 q -2 r   ->  [abcd][efghij][cdefghij]
parse value 'abcdefghij' with p 5 q <2 r   ->  [abcd][cd]    [cdefghij]
```

Same movement, unrelated assignment for `q`. **`<n` means "the `n` characters ending at the current position"; `-n` means "move back `n`, then assign to the end".**

**(b) `Absolute` and `Plus`/`Minus` share ONE rule: if the new position is greater than the current, the target gets `[current, new)`; otherwise it gets `[current, END]`.**
Backward movement never assigns the null string.

```
p 5 q      -> p='abcd'         q='efghij'
p 1 q      -> p='abcdefghij'   q='abcdefghij'    1 is not > 1, so remainder
p 11 q     -> p='abcdefghij'   q=''              past the end
p 5 q 5 r  -> r='efghij'                          equal counts as backward
p 5 q -99 r-> r='abcdefghij'                      clamped at 1, never before
p +0 q     -> both the whole string
```

**`>n`/`<n` do NOT follow it** -- they assign an exact slice, clamped at the ends: `p >0 q` and `p <0 q` both give `p=''`.

Errors: a fractional or non-numeric positional is **26.4**; `p =x q` is **38.2**.

**(c) Pattern triggers.** An **absent pattern matches at END**: `p 'z' q` gives `p` the whole string and `q=''`. The **empty pattern behaves as absent**. A pattern at position 1 gives `p=''`. Searches are non-overlapping and the next search starts after the previous match.

**The subtle one: after a string pattern the next *target* starts after the match, but a following *relative* trigger measures from the match START.**
`p 'c' +1 q` and `p 'c' q` are identical; `p 'c' -1 q` gives `q='bcdefghij'`.

**(d) `CASELESS` folds ASCII only, verified against a byte alphabet.**
Pattern `'e9'x` against a source containing `'c9'x` does **not** match; `'c9'x` matches itself exactly. **Every byte `>= 0x80` matches only itself.**
`CASELESS` preserves the original case in assignments; `UPPER`/`LOWER` transform the **source** before parsing.
`upper caseless` and `caseless upper` are both legal and order-independent; `upper lower` is **25.12**.

**(e) The comma fence assigns the null string to every unmatched target -- it never leaves one unset.**
`parse value 'a b' with p , q` gives `q=''`. An omitted *middle* argument empties that template only and does not shift the others.
Extra targets in one template also get `''`, and **only the final target keeps its leading blanks**: `p q r` on `'a  b  c'` gives `r=' c'`.

**(f) `PARSE SOURCE` field 2 varies by CONTEXT, not by call depth.** `LINUX COMMAND` at top level, in an internal subroutine and in an internal function alike; `LINUX METHOD` inside a `::method`.
*Flagged as inference: which fields vary across **hosts** was not measured, since one machine cannot show it.*

**(g) The trace shape, all eight kinds under `trace i`. All trace output is on stderr.**

| construct | lines, in order |
|---|---|
| source `VALUE` | `>L> "<expr>"` · `>K> "VALUE" => "<src>"` · `>>> "<src>"` |
| source `VAR` | `>V> NAME => "<src>"` · `>K> "VAR" => "<src>"` · `>>> "<src>"` |
| source `SOURCE`/`VERSION` | `>K> "<kw>" => "<src>"` · `>>> "<src>"` |
| source `ARG` | **no `>K>` at all** -- straight to `>>>` |
| any positional trigger | `>L> "<n>"` · `>>> "<n>"`, **before** the preceding target's assignment |
| `String` and `Mixed` | `>L>` · `>>>` -- **identical; caseless is not distinguishable in the trace** |
| target | `>=> NAME <= "<value>"` |
| `.` placeholder | `>.> "<consumed>"`, **emitted even when it consumes nothing** |
| `End` | nothing |
| comma fence | `>>> "<next template's source>"` |

**A trigger's numeric operand is evaluated BEFORE the preceding target is assigned** -- for `p 5 q -2 r` the order is `>L>"5"`, `>>>"5"`, `>=>P`, `>L>"2"`, `>>>"2"`, `>=>Q`, `>=>R`.
The traced literal for `+3`/`-2`/`>3`/`<2` is the **bare number without the sign**.

**(h) What the suite checks.** `keyword/PARSE.testGroup` is the **only** PARSE group: 682 `::method` bodies, 792 `assertSame`, **19 `expectSyntax` -- 18× 26.4 and 1× 38.2**.
**25.12 is asserted nowhere** in `ootest/ooRexx/base` despite being reachable via `parse upper lower` (positive control: the same scan finds 26.4 eighteen times).
UPPER/LOWER/CASELESS have **7 assertions out of 792** -- thin but not zero, and concentrated at `PARSE.testGroup:235-251`.

**A false lead already killed, of the `FORMAT`-in-`DATE` kind:** `parse source` occurs **440** times across `base/`, of which **407 are the harness prologue** present in nearly every group and only **6** are in `PARSE.testGroup`. A raw count overstates coverage roughly seventy-fold.

**Asserted outside PARSE's group:** `class/RexxInfo.testGroup:120-121` requires `parse version version` to equal `.RexxInfo~name`, and `:138-139` requires `parse source platform .` to equal `.RexxInfo~platform`.

- [ ] **Step 1: Measure the trace shape, which is not what the groundwork document says**

Measured 2026-08-04 under **`trace i`**, not `trace r`:

```
     2 *-* parse value 'a b c' with p . q
       >L>   "a b c"
       >K>   "VALUE" => "a b c"
       >>>   "a b c"
       >=>   P <= "a"
       >.>   "b"
       >=>   Q <= "c"
```

* **`>.>` fires at `trace i`, not `trace r`.** The groundwork document probed `trace r`, saw nothing, and recorded the absence -- an instrument that could not have produced the answer.
* **`>.>` carries the value the placeholder consumed.**
* **`>=>` is emitted per *assigned* target**; the `.` placeholder gets `>.>` instead, so the two partition the targets.
* **`>K>` carries a `=>` continuation here**, where 4a's and 4b's `>K>` lines carry a bare value.
  **Measure whether the existing `trace.rs` keyword path emits it before writing any of this**, and state which it was.

Probe all eight `TriggerKind` variants under `trace i`. The four a `trace r`-only probe will miss are `Plus`, `Minus`, `MinusLength`, `PlusLength`.

- [ ] **Step 2: Measure `PARSE SOURCE` and `PARSE VERSION` on this host**

Both are host-dependent; `SOURCE` carries an absolute path.
**Their witnesses belong in the live corpus, not in a committed `.expected` file.**

- [ ] **Step 3: Write the failing engine tests, then implement**

The engine is source-independent: byte string plus template in, assignments out.
Keep it so -- Task 8 adds two sources and must add no engine code.
**The comma fence is a template boundary, not a trigger**: a `None` entry advances to the next source string.

- [ ] **Step 4: Move `Parse` in scope**

`tests/owners.rs:165` (`InstructionKind::Parse => Owner::Phase("4c")`) becomes `Owner::InScope`; its row in **`EXPECTED_OUT_OF_SCOPE`** (`owners.rs:347`, **not** a const named `SPLIT_TABLE`) is removed; `src/lib.rs:758`'s arm loses `Parse`.
Its `loud.rs` witness must be **deleted**, not left stale, or `assert_witness_set_is_complete` fails the other way.

**`Arg` and `Pull` stay `4c` until Task 8** -- separate variants, moved separately.

- [ ] **Step 5: Move the `>.>` prefix**

`trace_oracle.rs`: `>.>` becomes `Witnessed`, and **`WITNESSED_PREFIX_COUNT` (`:551`) and `OUT_OF_SCOPE_PREFIX_COUNT` (`:555`) both move by one.**
A `Witnessed` entry is chained to `CLAIMED_PREFIXES` and thence to committed `.expected` bytes, so the witness must actually exist before the constant moves.

- [ ] **Step 6: Run the shared verify block and commit**

---

### Task 8: program arguments, `ARG`, `PULL`, `PARSE PULL`, `PARSE LINEIN`

**Files:** modify `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/lib.rs` (`instruction_owner`, `:758`; `run_program`'s signature), `crates/rexx-exec/src/bin/rexx-run.rs`, `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `rust/corpus/phase-4c.txt`

**Consumes Task 7's engine unchanged and 4b's `Interp::queue`.**

- [ ] **Step 1: Supply a top-level program's argument string**

**Nothing in the crate provides one today.** `run_program` takes no arguments and neither does `rexx-run`, yet `PARSE ARG` at top level, the `ARG` instruction and `ARG()` all read it.
This task adds the plumbing: `rexx-run` accepts trailing arguments after the program path, and `run_program` gains a parameter carrying them.

**Measure the oracle's model first.** `rexx foo.rex a b c` supplies **one** argument string, not three -- confirm that, and confirm what `ARG()` returns for it, before choosing the representation.

**The standing oracle wrapper passes no arguments**, so a corpus witness cannot exercise this unless the harness is extended.
If extending it is out of scope here, say so and record a `KNOWN GAP` row rather than shipping an untested path.

- [ ] **Step 2: Measure the queue-empty fallback**

`PULL` and `PARSE PULL` read the queue and **read stdin when the queue is empty**.
Probe with stdin at `/dev/null` and with a here-string, and record what an empty queue plus closed stdin produces.
`PARSE LINEIN` reads stdin directly.

**This is where 4b's `PUSH`/`QUEUE` gets its first differential witness.** 4b's gate records the queue as shipping with storage verified only in-crate; a corpus program that pushes and pulls closes that, and Task 15's gate should say so.

- [ ] **Step 3: Confirm `ARG template` is `PARSE UPPER ARG`**

They are the same instruction with `upper` set. Confirm rather than assume, and confirm `ARG` with no template is legal and does nothing.

- [ ] **Step 4: Write failing tests, implement, move `Arg` and `Pull` in scope**

Both `owners.rs` rows, both `EXPECTED_OUT_OF_SCOPE` rows, `lib.rs:758`'s arm, and both `loud.rs` witnesses deleted.

- [ ] **Step 5: Run the shared verify block and commit**

---

### Task 9: the `ADDRESS` instruction and environment tracking

**Files:** modify `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/activation.rs`, `crates/rexx-exec/src/lib.rs` (`instruction_owner`, `:761`), `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `rust/corpus/phase-4c.txt`

**Scope is the environment name only.**
`ast::Address` carries `environment`, `dynamic`, `command` and `io`.
This task implements `environment` and `dynamic`.
**`command` and `io` are Phase 7's under D18 and must still fail loudly naming Phase 7.**

- [ ] **Step 1: Measure the default and the swap semantics**

Measured: `say address()` with no `ADDRESS` instruction prints `sh` on this host.
**That default is platform-supplied and therefore Phase 7's**, so a corpus witness must assert the **swap**, not the initial value.

`ADDRESS` with no operand swaps to the previous environment.
Probe: the two-deep swap; the swap with no prior environment; and whether the setting survives a `CALL` and a `RETURN`.
It is per-activation state like `Settings`, so it belongs on `Activation` and not on `Interp` -- the same shape as 4b's `trace_mode` move.
**Measured (D-R): a `::routine` does *not* inherit it**, which Task 13 depends on.

- [ ] **Step 2: Split the `loud.rs` witness rather than deleting it**

`loud.rs:208`'s witness is `address cmd` -- an `ADDRESS` **with a command**, which stays Phase 7's.
Deleting the row removes the witness for a half that is still out of scope, and `assert_witness_set_is_complete` cannot catch that.
**Make the row arm-grained**, following the pattern already in `owners.rs:349-351`: the environment form moves in scope, the command and `WITH` forms keep a Phase 7 witness.

- [ ] **Step 3: Write failing tests, implement, move the `owners.rs` row**

- [ ] **Step 4: Run the shared verify block and commit**

---

### Task 10: `builtin/state.rs`

**Files:** create `crates/rexx-exec/src/builtin/state.rs`; modify `builtin/mod.rs`, `crates/rexx-exec/src/lib.rs`, `crates/rexx-exec/src/run.rs`, `rust/corpus/builtin-status.txt`.

**The 11 names:** `ADDRESS ARG CONDITION DIGITS ERRORTEXT FORM FUZZ GC QUEUED SOURCELINE TRACE`.

**Read "Shared facts every builtin task needs", restated in your brief.**

**`lib.rs` and `run.rs` are in this task's file list on purpose** -- see Step 2.

- [ ] **Step 1: Build the probe table**

`ADDRESS()` reads Task 9's tracked environment name.
`ARG()`, `ARG(n)` and `ARG(n,'E'|'O')` read the argument model, which is 4b's inside a routine and Task 8's at top level -- probe both.
`GC()` returns 0, measured.
`TRACE()` with no argument returns the current setting, which `rexxcps.rex` depends on.
`QUEUED()` reads 4b's queue, and **its differential is single-program only**: `rxapi` is running on this host and `rxqueue('G')` returns `SESSION`, so a cross-process comparison can never match.

- [ ] **Step 2: `CONDITION()`'s `I` and `S` options have no state to read, and this task adds it**

`ActiveCondition` (`lib.rs:871`) carries only `raised`, `site` and `sites`.
The trap's `call`-vs-`signal` kind is never copied onto the fired condition, so `CONDITION('I')` and `CONDITION('S')` have nothing to return.
Add the field where the trap fires and read it here.
**Probe every option letter from inside a live handler, not outside one** -- outside a handler the answers are all empty and cannot distinguish a correct implementation from a stub.

- [ ] **Step 3: Read each `<NAME>.testGroup`, write failing tests, then implement**

- [ ] **Step 4: Re-run the status harness; assert all 11 read `implemented` and none `divergent`**

- [ ] **Step 5: Run the shared verify block and commit**

---

### Task 11: `builtin/datatype.rs`

**Files:** create `crates/rexx-exec/src/builtin/datatype.rs`; modify `builtin/mod.rs`, `rust/corpus/builtin-status.txt`.

**The 4 names:** `DATATYPE SYMBOL VALUE VAR`.

**Read "Shared facts every builtin task needs", restated in your brief.**

- [ ] **Step 1: Build the probe table**

`DATATYPE`'s option letters are its whole surface -- probe every one.
`DATATYPE.testGroup` is one of the six groups containing non-UTF-8 bytes; use `/bin/grep -a` on it.

- [ ] **Step 2: `VALUE` is split, and its two-argument form writes the pool**

The variable-access form (`value('name')`, `value('name', newval)`) is 4c's.
**The external-selector form (`value(name, , 'ENVIRONMENT')`) is Phase 7's and must fail loudly naming Phase 7**, not silently ignore the third argument.

The two-argument form writes, so it needs the **growing** slot resolver.
That is **`Interp::slot_of`** (`plan.rs:561`), which checks plan, then `extra`, then grows, and is idempotent.
It is **not** `Plan::slot_of` (`plan.rs:528`), which is `&self -> Option<usize>` and cannot create.
An implementer who writes "if not in plan, grow" instead leaks one slot per call.

- [ ] **Step 3: Read each `<NAME>.testGroup`, write failing tests, then implement**

- [ ] **Step 4: Re-run the status harness; assert all 4 read `implemented` and none `divergent`**

- [ ] **Step 5: Run the shared verify block and commit**

---

### Task 12: `builtin/datetime.rs`

**Files:** create `crates/rexx-exec/src/builtin/datetime.rs`; modify `builtin/mod.rs`, `rust/corpus/builtin-status.txt`.

**The 2 names:** `DATE TIME`.

**Read "Shared facts every builtin task needs", restated in your brief.**

- [ ] **Step 1: Neither may appear in a corpus program (D11), so the unit tests are the whole gate**

`TIME('R')` is **stateful** -- it resets an elapsed-time clock, and `rexxcps.rex` depends on it.
Pin the **reset semantics**, not a value: after a reset, a later `TIME('R')` returns elapsed-since-reset rather than elapsed-since-start.

**Two probes inside the same second cannot distinguish a live clock read from a cached one.**
Construct the probe so they can -- separate the reads by a measurable interval, or drive the state rather than the clock.

`DATE`'s option letters are its surface; probe every one, and note which are locale- or host-dependent.
`DATE.testGroup` and `TIME.testGroup` spell 50 directives `::METHOD` uppercase, so scan them case-insensitively.

- [ ] **Step 2: Write failing tests, then implement**

- [ ] **Step 3: Re-run the status harness; assert both read `implemented` and neither `divergent`**

- [ ] **Step 4: Run the shared verify block and commit**

---

### Task 13: `::routine` dispatch, `>I>` and `<I<`

**Files:** modify `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/plan.rs`, `crates/rexx-exec/src/lib.rs`, `crates/rexx-exec/src/eval.rs`, `crates/rexx-exec/src/error.rs`, `tests/coverage.rs`, `tests/trace_oracle.rs`, `docs/superpowers/plans/phase-4-exclusions.txt`, `rust/corpus/phase-4c.txt`

**Why last of the implementation tasks.** Builtins shadow `::routine`s, so the resolution order cannot be verified until the table is complete.

- [ ] **Step 1: Set `BodyKey::directive` at the production site**

`plan.rs:79`'s `directive: Option<usize>` is `None` at **seven** construction sites, six of which are test-only.
**The production site is `lib.rs:1422`**; `plan.rs:631` is inside a `#[cfg(test)] mod tests` (the `cfg` is at `:615`), and the first revision of this plan sent an implementer to that fixture.

- [ ] **Step 2: Give a `::routine` activation its own pool, and five other non-inheritances**

D-R's table lists six measured differences from an internal label's activation.
**`run.rs:3304-3313` inherits `NUMERIC`, `ADDRESS` and the condition traps, and a `::routine` inherits none of the three** -- two of those three failures are silent.

**This is not 4b's `PROCEDURE` isolation reused.** A `::routine` has a different `CodeBody`, therefore a different `Plan`, therefore a different name-to-slot map, so the slot-index-identity property `PROCEDURE EXPOSE`'s alias bitset rests on does **not** hold across bodies.

- [ ] **Step 3: Place it third, and record what sits in front of 43.1**

Internal label, then builtin, then `::routine`.
Measured: `call max 1, 9` with a `::routine max` present returns 9.
**A quoted target is a second order:** `call 'ZORKOLO'` skips the internal label but still finds the `::routine`, while the builtin still wins.
Routine lookup **upcases both sides**, unlike `CodeBody::labels`.

**Replacing `Loud::unresolved_call` with an unconditional 43.1 is wrong, and this is the correction that matters most in this task.**
The oracle searches for an **external file** before raising 43.1: measured, with `zorkolo.rex` in the current directory, `call zorkolo` runs it at rc 0, and with a `::routine zorkolo` present the routine wins.
4c does not implement external routine resolution.
So this task must:

* raise 43.1 when the name resolves to none of label, builtin or `::routine`;
* add an `EXCLUSIONS` row -- **external routine resolution, Phase 7** -- carrying both transcripts;
* add a corpus rule that no corpus program may depend on external routine resolution, and note that the scratchpad's stale `.rex` files make this the easiest probe error in the phase;
* settle the contradiction between `lib.rs:479` ("external ... 4c's") and `eval.rs:484` ("external third (Phase 7)") in the same commit.

`eval.rs:507`'s own comment argues against an unconditional substitution here -- read it before changing it.

- [ ] **Step 4: Fail loudly on every non-`::ROUTINE` directive**

Implementing `::routine` removes the loud fallback that currently masks the fact that **no directive is ever installed**.
Measured: the oracle aborts before `main` on `::class foo subclass zzznotaclass` (98.909, rc 158) and on a missing `::requires` (43.901, rc 213), while `rexx-run` prints `main ran`.
Add a loud failure naming Phase 5 for any directive that is not `::ROUTINE`.

- [ ] **Step 5: Trace does not cross into a `::routine`**

Measured: a caller's `trace r` echoes its own clauses and none of the routine's, while an internal label's clauses *are* echoed under the same setting.
A deliberate difference between two paths that share a function.

- [ ] **Step 6: `>I>`/`<I<` under the two-condition gate**

`tracingLabels() && isMethodOrRoutine()` (`RexxActivation.cpp:3655`).
Measured:

* `trace l` in the **caller**, targeting a `::routine` -> nothing. The caller's setting does not cross.
* The routine's **own** non-dynamic trace instruction fires both lines, and `earlyTraceEntry` accepts **A, I, L and R** -- so `trace r` in the routine body emits them too.
  Any routine-body trace witness therefore emits them whether it intends to or not.
* The content is **not** a clause echo. It carries a **7-space leading indent**:

```
       >I> Routine "ZORKOLO" in package "<absolute path>".
       <I< Routine "ZORKOLO" in package "<absolute path>".
```

The absolute path makes any committed expectation host-dependent, so **the witness lives in the live corpus, not in `tests/trace_oracle/`**.
The package field is **one `String` on `Interp`**, not a package object.
`::options` is confirmed unnecessary.

- [ ] **Step 7: Let a `::routine` program into the corpus**

`tests/coverage.rs`'s `assert_program_has_no_directives` (`:150`) panics on any `::` directive in the subset union, and this task's witness is the first corpus program to carry one.
Relax it to permit `::ROUTINE` and keep the panic for the rest.
**Without this, keeping the witness out of the subset file is the path of least resistance, and then the `>I>`/`<I<` witness is read by no harness and the "`::routine` before builtins" mutation loses its catcher.**

- [ ] **Step 8: Flip `>I>`/`<I<` to `Witnessed` and move both counts**

`WITNESSED_PREFIX_COUNT` (`:551`) and `OUT_OF_SCOPE_PREFIX_COUNT` (`:555`) each move by two.

- [ ] **Step 9: Stop rendering an argument nobody will print**

**Inherited from Task 3's fix round, and assigned here because this task restructures `resolve_and_run_call` anyway and is the last 4c task whose file list already contains `run.rs`.**

`resolve_and_run_call` renders every evaluated argument to an owned `Vec` purely to hand it to `trace_argument`, which discards it unless the trace mode asks for intermediates.
A value of any size is therefore copied whether or not anything will print it, the copy is unguarded, and it **aborts the process** rather than raising.
Measured at the project's own `ulimit -v 1048576`: `say length(copies('a',400000000))` is `400000000` at rc 0 on the oracle and **SIGABRT at rc 134** here.

**Measured:** with the argument render placed behind `if self.trace_mode().intermediates`, `say length(copies('a',N))` at both 300000000 and 400000000 returns rc 0 with the oracle's answer.
That experiment was run and reverted, not shipped.

**It is not the whole of the cause, and an earlier revision of this step said it was.**
With the same experiment applied, four one-line neighbours still SIGABRT at 400 MB where the oracle returns rc 0:

```rexx
say length(strip(copies('a',400000000)))
say length(reverse(copies('a',400000000)))
say length(copies('a',400000000) || 'x')
x = copies('a',400000000)
```

The first two are `required_string`'s infallible `into_owned()`, the third is `Interp::concat`, the fourth the assignment path's own render.
**You are looking for a shape -- an unguarded owned copy on a path that may discard it -- not for one line.**
The false generalisation came from probing a single expression shape (`length` of a `copies`); vary the surrounding expression before believing any fix is complete.

**It costs real memory, not only address space, and that is the stronger reason to fix it.**
Measured at a 4 GiB limit where both sides succeed, peak RSS for `say length(copies('a',500000000))` is **978,460 kB here against the oracle's 495,860 kB** -- the oracle holds one copy of the result and this crate holds two.
482 MB of the 483 MB difference is one redundant copy of a 500 MB string.
**Re-measure that pair after your fix and expect parity.**
That expectation is arithmetic rather than a measurement, so treat a non-parity result as a finding rather than as noise.

**What the measurement does not say is which of the roughly fifteen similar sites need the same guard, and a wrong guard silently drops a trace line.**
So: **owe a trace witness for every site you change**, and change no site you cannot witness.
The assignment path's own `>>>` render is the same shape.

The **second** cause in that gap row -- `ulimit -v` limiting address space while `INTERPRETER_STACK_BYTES` reserves 512 MiB of it -- is **not yours and not closable in Phase 4**. It follows from D19's sized-thread choice. Do not attempt it, and do not let its presence stop you closing the first.

- [ ] **Step 10: Run the shared verify block and commit**

---

### Task 14: the compound-`DO` control-variable fix

**Files:** modify `crates/rexx-exec/src/run.rs` (`bind_control`), `crates/rexx-parse/src/ast.rs`, `rust/corpus/keyword-exempt.txt`, `docs/superpowers/plans/phase-4-exclusions.txt`

**Assigned to 4c by the 4b gate's Step 3c ruling.**

**What the divergence is.** `do cv.j = 1 to 5` is legal Rexx and the oracle iterates it, assigning the compound `CV.J` on every pass.
`bind_control` writes the control variable through a flat name-to-slot lookup, so `CV.J` becomes the literal name of one simple variable and no tail is resolved -- while the same executor resolves the same name correctly in `say cv.j` one line later.
**It is not a parse gap**: `cv.j` is a single symbol token and the parser interns `"CV.J"` whole.
The `LEAVE`/`ITERATE`/`END` forms naming a compound all dispatch correctly.

**The recorded cost cites a `rexx-parse` change, and its justification is wrong -- re-derive it.**
The recorded phrasing is "`Controlled::control` carrying the `VariableRef` shape an assignment target already does".
Both halves are false: an assignment target is an `Expr` (`ast.rs:709`), and `VariableRef` is `Direct`/`Indirect(SymbolId)` with **no tail**, which also contradicts this row's own "it is not a parse gap".
**Establish what shape `Controlled::control` actually needs by reading the code, and state it, before touching `rexx-parse`.**
It is possible the fix is confined to `rexx-exec`; if so, drop `ast.rs` from the file list.

- [ ] **Step 1: Reproduce all three narrowing probes before changing anything**

- [ ] **Step 2: Fix, then re-run `REXX_KEYWORD_GATE=1`**

Six `base/keyword` bodies turn on this and they are the **only assertion failures in the whole table**.
When the fix lands, all six start passing and `the_exempt_set_matches_the_current_failures` goes red until they are removed.

**A red test is not the success signal on its own** -- three different outcomes turn it red, and editing the six rows' attribution instead of removing them turns it green again.
**Assert that the six bodies now pass**, by name, before removing them.

- [ ] **Step 3: Remove the six rows, move the exclusions row to `CLOSED DEFECTS`, run the shared verify block, commit**

---

### Task 15: the `base/bif` L1 harness, the 4c subset, `mutate-4c.sh`, and the gate

**Files:**
* Create: `crates/rexx-extract/src/bif.rs`, `crates/rexx-exec/tests/bif_assertions.rs`, `rust/corpus/bif-exempt.txt`, `rust/scripts/mutate-4c.sh`, `docs/superpowers/plans/phase-4c-gate.md`
* Modify: `rust/corpus/phase-4c.txt`, `rust/corpus/README.md`, `crates/rexx-exec/tests/coverage.rs`, `crates/rexx-exec/tests/corpus.rs`, `crates/rexx-exec/tests/collect_stress.rs`

- [ ] **Step 0: What the extractor must model, measured 2026-08-05 -- read this before writing any code**

**D12's reuse decision holds, but not as a drop-in.** Without the three additions below, only **39.9% of calls extract correctly and the rest are silently wrong rather than dropped**, which the conservation invariant cannot see.

**The denominator is not what a naive scan gives.** `^[[:space:]]*::method` case-insensitively gives 5,462; **three of those sit inside `/* … */` block comments** (`CHARS.testGroup`, `LINES.testGroup` ×2), so live method directives are **5,459**.
All 6,293 `assertSame` calls reconcile by location: roughly **6,150-6,170 in live method bodies, 120-135 inside block comments, 5 in `::routine` bodies, 1 behind a `--`**.
The block-comment band is a range because ooRexx block comments **nest** and a same-line `/* … */` is easy to mis-detect; the total reconciles exactly either way.
**A line-oriented scan extracts over a hundred assertions that never run.**

**The composition, by body (a body counts in every category it touches):**

| category | bodies | calls |
|---|---|---|
| total live | 4,237 | 6,169 |
| **self-contained, no dependency** | **2,169 (51.2%)** | **2,464 (39.9%)** |
| local variable, simple literal RHS | 402 | 1,474 |
| local variable, computed RHS | 372 | 922 |
| `.local~` fixture set in another body | 962 | 969 |
| `self~` attribute | 24 | 126 |
| in-file `::routine` | 4 | 7 |
| `NUMERIC` in body | 363 | 565 |
| loop or conditional around the assertion | 113 | 531 |
| `expectSyntax` present | 200 | 210 |

**Add exactly three capabilities, all lookup or carry-forward -- no expression evaluator:**

1. **File-scoped fixture resolution.** Collect `.local~NAME =` per file and resolve `.NAME`, **stem-aware**: `WORD.testGroup:276` does `v8. = .v8` then `word((v8.10),4)`, and exact-name matching misses it. That case defeated the classifier that produced these numbers on its first pass and overstated self-contained by 25 bodies. Buys 962 bodies.
2. **`NUMERIC` carry-forward within the body.** 363 bodies, and **362 of them set `NUMERIC` *before* the first `assertSame`** -- the dangerous ordering. `base/expressions` already needed this and it is the category with a precedent for being got wrong silently, because such bodies' operands are literals and they therefore *look* self-contained. `ABBREV.testGroup:526` is the shape: both operands literal, `Numeric Digits 1` the only thing that changes the answer.
3. **`expectSyntax` routing**, per Step 2.

**Then drop the rest explicitly and count it: 418 bodies / 1,103 calls (17.9%)** -- computed local RHS, loops, `self~` attributes, in-file `::routine` calls. Bounded, nameable, covered by `rows + dropped == calls`. Drop the block-comment calls too, and rule deliberately on the 5 in `::routine` bodies.

**Two categories nobody thought to ask about, and the first is the larger risk:**

* **`::options novalue` is a file-level directive that inverts body semantics.** **38 of the 76 files** carry it. Under it an unassigned symbol **raises**; in the other 38 files it evaluates to its own uppercased name. **Same body text, opposite meaning, and the deciding directive is outside the body.** Roughly 1,325 bodies / 2,027 calls live under it. Verified: none of the seven word groups carry it, which is exactly why they can write `word(nv,1)` = `'NV'`.
* **The NOVALUE idiom itself** -- assertions that read a never-assigned symbol and rely on symbol-equals-own-name, e.g. `BITAND.testGroup:185`'s `assertSame(bitand('3', nv2.3), '02'x||'V2.3')`. Resolvable from the body's own bytes **only if the extractor knows the rule**; to a naive reader it looks like unresolved indirection. **Bounded at ≥178 firm, ≤307 loose** -- the residue contains false positives from instruction forms (a `PARSE` target read later), and the gap was not closed.

**Confidence, stated so it can be checked rather than trusted.** Firm: the reconciliation, the 5,459/5,462 split, the 362-of-363 `NUMERIC` ordering, the 38-of-76 `::options` split, the `expectSyntax` 200/210. Softer: the resolvable/not split turns on a "simple versus computed RHS" rule, and that rule is conservative, so the mechanically-resolvable set is probably slightly larger than stated.

- [ ] **Step 1: Extract `base/bif` by reuse**

D12: no third extractor. Preserve the conservation invariant `rows + dropped == calls` and the `DropReason` detail field.
**Match the assertion token case-insensitively, never by prefix**, and **scan `::method` case-insensitively** -- `DATE`/`TIME` spell 50 of them `::METHOD`.
**Use `/bin/grep -a`** for any count over this directory; six groups contain non-UTF-8 bytes.

Measured population: **6,293 `assertSame`** across 73 of 76 files, 5 `assertSameList`, 1,230 `expectSyntax`, and a 424-call `assertTrue`/`assertEquals`/`assertFalse` tail the extractor drops.
State the drop in the header and let the conservation invariant carry the number.

- [ ] **Step 2: `expectSyntax` couples, more sharply than in `base/expressions` -- measured 2026-08-05, do not re-derive**

**The mechanism, read from `ootest/framework/OOREXXUNIT.CLS` rather than inferred.**
`expectSyntax` (`:1322`) **wraps nothing**; it is a pure state-setter on the TestCase instance.
The checking is a frame up in the framework's own `doTheTest` (`:1563`), which installs `signal on any name exceptionHandler` **in the framework method, not in the test method**, then runs the body as `.message~new(self, methodName)~send`.
A matching condition returns immediately (`:1599`); `check4ConditionFailure` (`:1589`) is reached only if the body ran to completion.

**So a raise inside a test body abandons the rest of that body.**
The expectation is method-scoped, not file-scoped -- `clearCondition` is called only from `Assert~init` and `TestSuite` builds one instance per test method -- so the coupling is strictly intra-body.

**The consequence for the extractor.** The dominant shape puts the raiser **inside `assertSame`'s own argument list**, so arguments evaluate, the raise fires, and `assertSame` is never entered:

```rexx
::method "test_21"                       -- C2D.testGroup:129
   self~expectSyntax(40.5)
   self~assertSame(C2D(,-1), '-1')
```

Verified on the oracle: `c2d(,-1)` is **40.5 at rc 216**. **`'-1'` is not an expected value** -- nothing ever compares anything to it. The body asserts one thing: that the call raises 40.5.

An extractor that ignores `expectSyntax` emits *"`C2D(,-1)` equals `'-1'`"* -- **confidently backwards**, and `rows + dropped == calls` balances, so the conservation invariant certifies it.
That is worse than a dropped row: it points an implementer at a return value where the required behaviour is a raise.

**The rule: a `::method` body containing `expectSyntax` yields no `assertSame` rows from any call at or after that line.**
Route those bodies to an error-expectation path keyed on the syntax code, and count the suppressed calls as **dropped** so the invariant still closes over them.

**The bound: 200 bodies contain both; 199 have the `assertSame` at or after the `expectSyntax`, carrying 205 calls.**
Of the 199, **189** have the raiser inside `assertSame`'s argument list and **10** are a separate `ret = <call>` clause.
Exactly one body asserts before expecting (`LINEOUT.testGroup:207`), which is the correct ordering and the only instance of it.
No body has more than one `expectSyntax`.

**Segment bodies at the next `::` directive of ANY kind, not at the next `::method`.**
Getting this wrong is how the count above was first got wrong, by me: `CONDITION.testGroup`'s `test_novalue_override` is three lines, and a `^::method` scan swallows the three following `::routine` bodies into it, inventing a 201st coupled body and five phantom reachable calls.
`assertSame` inside a `::routine` is a **trap handler's** assertion and genuinely runs.

**The unreachability is conditional; the hazard is not.**
If an expected condition failed to raise, the body would run on and the `assertSame` would execute -- but `check4ConditionFailure` then fails the test anyway.
So in neither case is that `(expected, actual)` pair a statement about correct behaviour.

- [ ] **Step 3: Build the harness, with the set assertion ungated**

`REXX_BIF_GATE=1` selects STRICT reporting, as `REXX_KEYWORD_GATE` does.
**But the both-direction set assertion must run under a plain `cargo test`, not behind the env var.**
That is where `keyword_assertions.rs`'s teeth are, and putting `bif-exempt.txt`'s equivalent behind the flag leaves it policed by nothing anyone runs.

The exempt set's attribution is **derived from the loud message**, not hand-written.

- [ ] **Step 4: Wire `phase-4c.txt` into all three harnesses**

`tests/corpus.rs:548-550` hardcodes `phase-4a.txt` and `phase-4b.txt`.
So do `tests/coverage.rs` and `tests/collect_stress.rs`.
**All three must read the three-file union**, or every 4c witness is inert and D6 is undischarged.

Add `phase_4c_subset_matches_the_committed_list` to `coverage.rs` -- the pin 4b's gate found missing for `phase-4b.txt`, where **nine of twelve entries were deletable with everything green**, including one criterion's only witness.

**Corpus rules for 4c, written into `corpus/README.md` beside the `DO OVER` one:** no `RANDOM`, no `DATE`, no `TIME` (D11); `QUEUED()` single-program only; no dependence on external routine resolution (Task 13); no `DO OVER` on a stem (D3).

- [ ] **Step 5: Write `mutate-4c.sh`**

Carry 4a's and 4b's guard: exact-match, exactly-once application; a baseline before the first mutation and after the last restore; three-way `PASSED`/`DIVERGED`/`INFRA_FAILURE` that never folds an infrastructure failure into either bucket; a non-zero test-run count per target, because `cargo test <name>` exits 0 when it matches nothing.

**Declare each mutation's outcome per instrument in advance**, so an unexpected catch fails as loudly as an unexpected survival.

**A `PASSED`/`PASSED` declaration needs a written justification in the script**, naming the instrument that *should* have caught it and why it cannot.
4b's row 12 was a genuine equivalent mutant; without this rule, declaring a mutation a survivor because nothing happens to test it scores "as declared" and reports coverage that does not exist.

Suggested shapes, each needing an instrument named: a builtin's optional argument ignored; an arity bound off by one; a `PARSE` trigger boundary off by one; a `.` placeholder assigning instead of discarding; the comma fence treated as a trigger; `ADDRESS`'s swap keeping the old name; `::routine` resolved before the builtin table; `TIME('R')` not resetting.
**`TIME('R')` is barred from the corpus by D11**, so its declared catcher must be Task 12's unit test -- name it.

- [ ] **Step 6: Write the gate document, criteria first**

**Write the criteria before running anything.** Carry 4b's ten forward with these amendments:

* **Criterion 2** (`tests/assertions.rs`) will report **the same 4,224 of 4,259 with 35 RUNTIME-BLOCKED** as 4a and 4b did.
  All 35 are `unblocked_by: "Phase 5"` and no row is `4c`.
  **State plainly that deleting the whole of 4c leaves this criterion green**, so it is carried as a regression check and not as evidence of 4c's delivery.
* **Criterion 3**'s target is **16 of 19**: `>.>`, `>I>`, `<I<` are 4c's; `+++` is Phase 7's under D-P.
* **Criterion 10**'s 790 `4c` rows in `keyword-exempt.txt` fire here.
  **790 is an upper bound on what 4c fixes, not a measure of its remaining surface**: three `CALL` bodies fail **under the C++ oracle itself** (`Error 43, Routine not found`) and `NUMERIC::test_42` exits 3 by falling through into its own `dig:` label.
  **No task owns these rows**, so they will fire ungated across the family tasks; say which task removed which, or the criterion is green here by construction.
* **New: the builtin-status criterion**, both directions, whose falsification is Task 2's Step 5 -- the interpreter mutation, not the file edit.
* **New: `base/bif`**, reported as a measurement and **not gated on a threshold**, with the set assertion ungated per Step 3.

**Ask of every criterion: what degenerate implementation satisfies this, and would deleting its subject leave it green?**
Four traps specific to 4c:

* **"Each of the 66 names is recognised" is satisfied by 66 stubs returning `''`.** The criterion must assert a value per builtin against the oracle. Task 1's differential harness is what makes this real rather than nominal.
* **A `PARSE` criterion asserting "exited 0" passes for a program that parsed nothing.** Assert the assigned values, chosen so an unset target renders as its own derived name and is recognisably wrong.
* **No builtin is under the collector at all until this task fixes it.**
  Measured at Task 2: the 42-program stress subset calls **no builtin**, so every allocation the 66 add is outside `run_program_collect_every_alloc`'s reach.
  The union must gain at least one program per family that allocates, or criterion 4 passes over a subset that never exercises the code 4c added -- the same defect 4a's version had when it ran 29 programs and zero call frames.
* **Criterion 4's collector control must delete a root a *builtin* holds.**
  Because builtins reuse `resolve_and_run_call`'s argument evaluation, the obvious root in that window is `run.rs:3259`'s `push_temp(argument.value())` -- which is **verbatim `mutate-4b.sh` row 9**.
  Re-running it re-tests 4b, which is what the criterion's own second sentence forbids.
  The control must target a root the **builtin's own result** holds between allocation and the caller's use.
* **A "reported, not gated" measurement can still be vacuous** if the set assertion behind it is behind the env var.

- [ ] **Step 7: Run everything; record each figure with its command and unpiped exit status**

---

## Explicitly not in scope

* **`ExprKind::List`** -- Phase 5's (D7).
* **`::method`, `::class`, `::requires`, `::attribute`, `::options`** -- Phase 5's. Only `::ROUTINE` is carved out (D-R), and Task 13 makes the rest fail loudly.
* **`QualifiedCall`** -- Phase 5's; namespaces come from `::REQUIRES`.
* **External routine resolution** (a `.rex` file found on the search path) -- Phase 7's, recorded by Task 13.
* **Command dispatch, `ADDRESS ... WITH` redirection, the platform-supplied default environment, and `+++`** -- Phase 7's (D18, D-P).
* **`TRACE ?`'s interactive pause and its banner lines** -- Phase 7's (D-P).
* **The fifteen excluded builtins** (D4), and `VALUE`'s external-selector form.
* **I32-I35**, unowned and unchanged: prefix-operator recursion outside the depth budget; recursive `Debug`/`PartialEq`/`Clone` on `Expr`; the depth counter protecting a sized caller only; `Plan::by_symbol` as a `HashMap` where D16 wants a `Vec` index.
* **The `DO OVER`-on-a-stem traversal-order deviation** (D3).
