# Phase 4c plan review: executability and internal consistency

Reviewed: `docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md` (831 lines)
Tree: `plan/rust-rewrite` @ `c22b9124`, working tree clean.
Nothing in the repo was modified. The oracle was not run.

---

## 0. The tree's actual state

```
$ cd rust && cargo test --offline --workspace --no-fail-fast
EXIT=0
```

Exit status read unpiped from the backgrounded command's own wrapper (`; echo "EXIT=$?"`),
not from a pipeline. The command did real work: 70 test binaries reported a
`test result:` line, and the slowest (`assertions.rs`) took 13.22 s.

| figure | value |
|---|---|
| test binaries reporting a result | 70 |
| passed | **1,020** |
| failed | **0** |
| ignored | 4 |
| `error`/`failures:` lines in output | 0 |

22 of the 70 binaries report `0 passed` -- these are the empty `tests/support`-style
and doc-test targets, not silently skipped suites.

```
$ git log --oneline -3
c22b9124 Write the Phase 4c plan, settling five deferred decisions and two new ones
d73a5678 Add the two general method rules Phase 4b paid for
3e1bbb4a State why loud.rs omits the SPLIT_TABLE_PHASES check, not that it used to
```

**The tree is green, as the plan claims.** Everything below is about the plan's text
against that green tree.

---

## 1. The extraction question, answered plainly

**Mechanical extraction of "Task 6" yields nothing. Seven implementers, not six, get a
brief with no steps.**

Every heading in the plan, verified:

```
318:### Task 1: Boundary infrastructure and the three attribution fixes
399:### Task 2: Builtin dispatch, the arity table, and the 40.x error family
480:### Tasks 3-6, 10-12: the seven builtin families
560:### Task 7: the `PARSE` template engine
624:### Task 8: `ARG`, `PULL`, `PARSE PULL`, `PARSE LINEIN`
649:### Task 9: the `ADDRESS` instruction and environment tracking
671:### Task 13: `::routine` dispatch, `>I>` and `<I<`
724:### Task 14: the compound-`DO` control-variable fix
749:### Task 15: the `base/bif` L1 harness, the 4c corpus subset, and `mutate-4c.sh`
```

There is no `### Task 3:`, no `### Task 4:`, `5:`, `6:`, `10:`, `11:`, or `12:`. The
per-task detail further down (`**Task 3 -- \`string.rs\`, 23 names.**`) is **bold text
inside the shared section**, not a heading -- so it is not addressable by any
heading-based slicer either.

The plan is aware of the problem and answers it with prose:

> **Each task's brief carries this section verbatim plus its own name list.**

That sentence is an instruction to whoever performs the extraction. It is not a
property of the document. The plan's own D5 ("one lane, never dispatch two implementers
in parallel") means a controller will be extracting fifteen briefs one at a time over a
long session, which is exactly the circumstance under which a prose instruction about
extraction gets skipped once.

**Verdict: BLOCKER.** The fix is mechanical -- give each of the seven its own `### Task N:`
heading and inline the shared Step 1-5 block into each. That costs seven copies of a
22-line block. The plan's own anti-skew section argues against duplicating *facts that
move*; the Step 1-5 template is a fixed procedure, so duplicating it costs nothing the
plan's own reasoning objects to.

---

## 2. Per-task: can an implementer who reads only this section do the work?

### Task 1 -- Boundary infrastructure and three attribution fixes

**BLOCKER: `EXCLUDED_BUILTINS` is unreachable from the file Task 1 creates.**

Step 1's classifier says "the name is in `coverage.rs`'s `EXCLUDED_BUILTINS` -> `excluded`".
That const is at `rust/crates/rexx-exec/tests/coverage.rs:701`, declared

```rust
const EXCLUDED_BUILTINS: &[&str] = &[
```

-- private, in an **integration test**. `tests/builtin_status.rs` is a separate crate;
it cannot see it. The only sharing mechanism in the tree is `tests/support/mod.rs`
(`mod support;` at `corpus.rs:182` and `trace_oracle.rs:150`), which exports only
`TRACE_PREFIXES` and `normalize_stderr`. Task 1's Files list names neither
`tests/coverage.rs` nor `tests/support/mod.rs`, so the implementer's three options are
all bad: copy the list (a fourth copy of the exclusion set, which is the drift the whole
task exists to prevent), edit a file the task does not name, or stop. The plan needs to
say which, and add the path.

The same problem recurs one task later and worse -- see Task 2.

**IMPORTANT: the three classifier bullets are order-dependent and are not presented as
ordered.** `EXCLUDED_BUILTINS` has **18** rows, three of which (`VALUE`, `ADDRESS`,
`QUEUED`) are *partial* -- in scope in one form. `coverage.rs:757` encodes that as

```rust
let in_scope = names.len() - (EXCLUDED_BUILTINS.len() - 3);
```

Step 2's expected counts (66 loud / 15 excluded / `implemented + loud == 66`) only come
out if `loud` is tested **before** `excluded`. Test `excluded` first and you get 63/18,
and the `== 66` assertion fails on a correct implementation. The bullets are a plain
list with no stated precedence. Say "in this order".

**IMPORTANT: D4's obligation on Task 1 is in D4, not in Task 1.** D4 says:

> **What Task 1 owes this decision:** the exclusions file gains **one sentence per
> excluded row saying why it is blocked** [...] Three rows need a reason that is not
> "the platform layer": `USERID` [...] `SETLOCAL`/`ENDLOCAL` [...]

None of Task 1's seven steps mentions it. Task 1's own title says "the three attribution
fixes", and this is a fourth, differently-shaped edit to the same file. An implementer
reading only Task 1's section does Steps 1-7 and D4's requirement is silently dropped.
**This is the recurring defect exactly: a required edit stated in a decision section with
no task body carrying it.**

**IMPORTANT: no step tells Task 1 what to do about `>.>`.** Task 1 modifies
`tests/trace_oracle.rs` only to flip `+++`. Fine -- but see Task 7 below, where the
consequence lands.

**MINOR: Step 7 says "Stage exactly the six paths above."** Five paths are listed:
`tests/builtin_status.rs`, `corpus/builtin-status.txt`, `tests/trace_oracle.rs`,
`phase-4-exclusions.txt`, `2026-07-30-phase-4a-executor-design.md`. If the sixth is
`tests/coverage.rs` (per the blocker above), say so; if not, the count is wrong.

**Verified correct in Task 1:**
* `trace_oracle.rs:529` is `("+++", Coverage::Owned("4c"))`. ✓
* `OWNER_PHASES` (`trace_oracle.rs:560`) is `&["4c", "Phase 5", "Phase 7"]`, so it does
  already admit `"Phase 7"`. ✓
* `phase-4-exclusions.txt:84` reads `six -- +++ and >.> (4c), >M> and >N> (Phase 5)`. ✓
* `phase-4-exclusions.txt:989-1011` is the `TRACE ?` row and it does end
  `Owner unassigned.` at `:1009`. ✓ (That row's own `:1002-1007` calls the defect "the
  sixth instance of one mechanism in this phase" and names extraction-per-task as the
  cause. The plan is repeating it.)
* `2026-07-30-phase-4a-executor-design.md:71` contains the exact quoted sentence
  including "every directive". ✓
* `phase-4-exclusions.txt:540` is the `QualifiedCall` row and does cite
  "every directive... [is] Phase 5's". ✓
* `run_program` is `pub fn run_program(path: &str, text: Vec<u8>) -> Outcome`
  (`lib.rs:1639`) and `NOT_IMPLEMENTED_EXIT` is a public re-export used by
  `tests/loud.rs:97`. ✓
* `rexx_inventory::builtins::NAMES` has 81 entries (`coverage.rs:728-733` asserts it and
  the suite is green). ✓

**Correction to D-R's supporting claim.** D-R says `phase-4-exclusions.txt:88` "says
`::routine` is 4c's (\"which 4c will have to meet\")". Line 88 is
`>I> / <I<   4c, deferred alongside ::routine dispatch itself.` The quoted phrase is at
**`:124`**, not `:88`. The substance (that row does place `::routine` in 4c) is right;
the quote is attributed to the wrong line, and Task 1 Step 6 tells an implementer to
"correct `phase-4-exclusions.txt:88`'s row" -- which is the `>I>`/`<I<` row and is not
wrong.

---

### Task 2 -- Builtin dispatch, the arity table, the 40.x family

**BLOCKER 1: the stated hook point is upstream of the argument evaluation the task says
it consumes.**

Task 2's Interfaces say:

> Consumes: `resolve_and_run_call`'s existing argument evaluation, **unchanged**.
> It already produces `Vec<Option<Argument>>`

Task 2's "Where it hooks in, exactly" says:

> The builtin step goes **between** the label lookup and that loud fallback.

The loud fallback is `run.rs:3218-3220`:

```
3218:        let Some(target) = target else {
3219:            return Err(Loud::unresolved_call(name).into());
3220:        };
```

The argument evaluation is **downstream of it**, at `run.rs:3244`:

```
3244:        let mut arguments: Vec<Option<Argument>> = Vec::with_capacity(args.len());
```

So the two sentences describe incompatible edits. Placing `dispatch` where the plan says
gives it no arguments at all; consuming the existing `Vec<Option<Argument>>` requires
restructuring the function so the label miss no longer early-returns, arguments are
evaluated on both paths, and dispatch happens after evaluation and before the activation
push at `:3329`. That restructure crosses several load-bearing invariants the function's
own comments pin, none of which Task 2 mentions:

* `set_sigl(self.clause_state.line())` at `:3277` -- must a builtin call set `SIGL`?
  Unstated. The oracle's answer is a measurement no step asks for.
* the `>A>` argument trace at `:3260-3261`, one line per position including omitted ones.
  Does a builtin call emit `>A>`? Unstated.
* `MAX_ACTIVATION_DEPTH` at `:3284`. Does a builtin consume a depth level? Unstated.
* `CallContext { name, arguments }` at `:3398-3404`. A builtin does not push an
  activation, so this must not be replaced -- but `ARG()` inside a builtin's own argument
  expression would then see the enclosing call's context, which is correct and worth
  saying once.

**BLOCKER 2: Step 3's set assertion is arithmetically wrong and the correction is
invisible to Task 2.**

> **Assert the dispatch's name set equals `rexx_inventory::builtins::NAMES` minus
> `EXCLUDED_BUILTINS`.**

`NAMES` is 81, `EXCLUDED_BUILTINS` is 18. That set is **63**. The plan's own File
structure table requires **66** dispatchable names, and three of the difference --
`VALUE` (`datatype.rs`, Task 11), `ADDRESS` and `QUEUED` (`state.rs`, Task 10) -- are in
`EXCLUDED_BUILTINS` *as partial rows*. An implementer who writes the assertion as stated
gets a test that will go red the moment Task 10 lands and will look like Task 10's bug.

The `- 3` correction exists in exactly two places, and Task 2's brief contains neither:
`coverage.rs:757`, and the prose of this plan's Tasks 10 and 11. This is the
invisible-decision shape again: a fact three tasks need, stated where none of them reads.

**BLOCKER 3 (shared with Task 1): `EXCLUDED_BUILTINS` is not reachable from `src/`.**
Task 2 puts this assertion in `src/builtin/mod.rs` or in a test beside it. Either way it
cannot reference a private const in `tests/coverage.rs`. No task says where the shared
copy lives.

**IMPORTANT: the two raisers Task 2 says to produce already exist under other names.**

Task 2's Interfaces: "Produces: `Raised::bad_argument_count`, `Raised::bad_argument_type`."

`error.rs` already has:

```
379:    pub(crate) fn not_enough_arguments(routine: &[u8], minimum: usize) -> Raised {
380-        Raised::syntax(40, 3, vec![ <routine>, <minimum> ])
397:    pub(crate) fn too_many_arguments(routine: &[u8], maximum: usize) -> Raised {
398-        Raised::syntax(40, 4, vec![ <routine>, <maximum> ])
```

`not_enough_arguments`' own doc records the measured message
`Not enough arguments in invocation of SUB2; minimum expected is 2.` -- byte-identical in
shape to the plan's own measured builtin probe
`Not enough arguments in invocation of SUBSTR; minimum expected is 2.` The plan does not
say whether these are to be reused, renamed, or duplicated. An implementer reading only
Task 2 writes a second 40.3 constructor.

**MINOR: `run.rs:3218` cited as "currently reads" the let-else.** Verified exact. ✓
**MINOR: `eval.rs:529` delegating to `resolve_and_run_call`.** Verified exact. ✓
**MINOR: `Argument` at `lib.rs:1340`.** Verified: `enum Argument` is declared at
`lib.rs:1340`, is a private two-variant enum, and `Argument::value()` at `lib.rs:1354`
has the doc `"USE ARG" without ">" and "ARG()" both want only this` -- so the plan's
"its own doc names `ARG()` as a caller" is exactly right. ✓

**IMPORTANT: no rooting rule.** `dispatch(interp: &mut Interp, name, args: &[Option<ObjRef>])`
receives bare `ObjRef`s and every builtin allocates its result. `resolve_and_run_call`
pushes each argument as a temp root (`self.roots.push_temp(argument.value())` at
`run.rs:3259`) but Task 2 says nothing about whether that frame is still live at dispatch
time, nor about `Interp::alloc_with` (`lib.rs:1585`). The plan *knows* this hazard --
Task 15's Step 6 says "Criterion 4's collector control must delete a root that a
*builtin* holds -- an argument between evaluation and the builtin's own use." That
sentence is in the gate criteria, twelve tasks after the machinery it constrains.
**Same shape as the sharpest historical case.**

---

### Tasks 3-6, 10-12 -- the seven builtin families

Beyond the extraction BLOCKER in section 1:

**BLOCKER: six of the seven never see the interface they are implementing against.**

Even granting the shared Step 1-5 block, that block contains no signature. Every fact
needed to write a builtin function lives only in Task 2's Interfaces section:

* `dispatch`'s signature `(interp: &mut Interp, name: &[u8], args: &[Option<ObjRef>]) -> Option<Result<ObjRef, Failure>>`
* that `dispatch` matches on the **upcased** name (Step 3)
* the arity table's row shape: `(min, max)` with `max: Option<usize>` for variadics (Step 3)
* **pass `Argument::value()`, not the `Argument`** -- and why
* **an omitted position stays `None` rather than being closed up** -- which every family
  task needs, because the shared Step 1 explicitly tells them to probe the `f(,2)` form

An implementer of `word.rs` reading only their own section knows seven names and a
probing procedure, and has no idea what function to write.

**Honest verdict on "probe the oracle and build a table":**

**Legitimate design, not a placeholder.** Three reasons. First, the semantics genuinely
are in the oracle: no plan can carry `FORMAT`'s edge cases (`FORMAT.testGroup` has
**767** `::method` bodies -- verified, and it is the largest in `base/bif`; next is
`TIME.testGroup` at 309). Second, the plan constrains the *method* far past "write some
tests": every optional argument position separately, a non-default pad where one exists,
the empty string and the `f(,2)` form, and the boundary values the ooTest group uses.
That is a falsifiable procedure -- a submitted table with one probe per builtin visibly
fails it. Third, Step 2 routes each task to a named reference file
(`ootest/ooRexx/base/bif/<NAME>.testGroup`, verified to exist -- 76 `.testGroup` files
plus `ARG_TEST.rex` and `lineout`) and states the cost of skipping it in terms of a later
task's output.

Contrast with a real placeholder: "similar to Task 3", "handle the edge cases", "TBD".
None of those appears anywhere in the plan. I searched; the plan is clean of them.

**What the family task bodies still need to become executable:**

1. Their own headings (section 1).
2. `dispatch`'s signature, the upcased-name rule, the arity-row shape, `Argument::value()`,
   and the omitted-position rule -- copied in, or Task 2 promoted to a Global-constraints
   -style restated block.
3. The probe-safety rules they depend on. **None of the seven restates the `ulimit`
   wrapper or the fresh-empty-directory rule**, and the plan's Global constraints section
   says in its own second sentence: "It is not extracted into task briefs, so a task that
   depends on one of these lines restates it." Every one of these tasks is a
   probe-first task. The `ulimit` line's own justification is "which has already cost a
   session and the machine's memory", and the fresh-directory line's is "**This bites 4c
   hardest of any sub-phase**". Verified: `ulimit`/`LD_LIBRARY_PATH`/`fresh empty` appear
   at plan lines 37, 38, 45 and **nowhere else in the document**.
4. Task 5 specifically: the Global section names `b2x`, `x2b`, `x2c`, `c2x`, `d2x`, `x2d`
   as "**the live hazard here**" for the symbol-followed-by-quote literal trap. Those are
   six of `convert.rs`'s twelve names. Task 5's body does not restate it. Task 5 *does*
   restate the `NUMERIC DIGITS <= 1000` rule -- so the restatement discipline was applied
   to one of the two constraints that bite this exact task and not the other.
5. D15 ("a value's rendering is fixed when the value is created"). Restated in Task 6
   only. `string.rs` (`LENGTH`, `POS`, `LASTPOS`, `COUNTSTR`, `COMPARE`, `VERIFY`),
   `word.rs` (`WORDS`, `WORDINDEX`, `WORDLENGTH`, `WORDPOS`) and `convert.rs` (`C2D`,
   `X2D`) all return numbers.
6. The rooting rule from Task 15's criterion 4.

**Task 10 specifics.**

*IMPORTANT: `CONDITION('I')` has no state to read.* Task 10 says "`CONDITION()` reads
4b's trap state -- probe every option letter inside a live handler". `ActiveCondition`
(`lib.rs:871`) is:

```rust
struct ActiveCondition {
    raised: Raised,
    site: Option<FailureSite>,
    sites: Vec<FailureSite>,
}
```

`Raised`'s first field is documented (`error.rs:132-136`) as "The condition name a
trapped Rexx program would see from `condition('c')`", so `('C')` and `('D')` are
reachable. But `CONDITION('I')` returns `CALL` or `SIGNAL`, and that discriminator lives
on `Trap::call` (`activation.rs:61`), a *trap-table* field that is not copied onto the
fired condition at any of the three `active_condition = Some(...)` sites (`run.rs:2605`,
`:2700`). `CONDITION('S')`'s ON/OFF/DELAYED status is likewise unmodelled. Extending
`ActiveCondition` means editing `src/lib.rs` and `src/run.rs` -- **neither is in Task 10's
Files list**, which is "create one `builtin/<family>.rs`; modify `builtin/mod.rs` and
`corpus/builtin-status.txt`", under the heading "No task in this group touches any other's
file."

*IMPORTANT: `ADDRESS()`'s Phase 7 default.* Task 9 measures that `say address()` prints
`sh` and rules that "That default is platform-supplied and therefore Phase 7's". Task 10
implements `ADDRESS()` and says only "`ADDRESS()` reads Task 9's tracked environment
name". What `ADDRESS()` returns before any `ADDRESS` instruction has run -- the Phase 7
default -- is not stated in Task 10. An implementer hard-codes `sh` and quietly ships a
platform-supplied value the plan puts out of scope.

**Task 11 specifics.**

*IMPORTANT: the named function cannot do the job.* Task 11 says `VALUE`'s two-argument
write form "needs `Plan::slot_of`'s idempotent path -- the same one 4b's `EXPOSE` uses --
and not a bare grow." There are two `slot_of`s:

```
plan.rs:528:    pub(crate) fn slot_of(&self, name: &[u8]) -> Option<usize>   // impl Plan
plan.rs:561:    pub(crate) fn slot_of(&mut self, name: &[u8]) -> usize        // impl Interp
```

`Plan::slot_of` is `&self -> Option<usize>` and its own doc (`:524-527`) says it
"Consults neither `extra` nor growth -- `Interp::slot_of` is the full three-source
resolution this is one third of." `value('newname', v)` on a name the body never mentions
must *create* the variable, which `Plan::slot_of` returns `None` for. The plan names the
non-growing one and warns against "a bare grow" -- the growing one is the only one that
works, and disambiguating them needs `plan.rs` open, which Task 11's Files list does not
include.

*IMPORTANT: `VALUE`'s Phase 7 loud path has no constructor and no file.* Task 11 requires
`value(name, , 'ENVIRONMENT')` to "fail loudly naming Phase 7". `Loud` has exactly five
constructors, all in `src/lib.rs` (`instruction` `:404`, `expression` `:472`,
`unresolved_call` `:496`, `compound_expose` `:535`, `missing_body` `:555`), and none takes
a free-form phase for a builtin argument. Task 11 must add one to `src/lib.rs`, which its
Files list forbids.

**Task 12 specifics.** Clean, given the shared-block fixes. `TIME('R')`'s "two probes
inside one second cannot distinguish a live clock read from a cached one; construct the
probe so they can" is a genuinely well-specified anti-vacuity instruction.

---

### Task 7 -- the PARSE template engine

**BLOCKER: `src/lib.rs` is edited by Step 4 and is not in the Files list.**

Files: create `parse_template.rs`; modify `src/run.rs`, `tests/owners.rs`,
`tests/loud.rs`, `tests/coverage.rs`, `tests/trace_oracle.rs`.

Step 4: "`src/lib.rs:761`'s arm loses `Parse`."

Verified, `lib.rs:758-761`:

```
758:        InstructionKind::Parse(_)
759:        | InstructionKind::Arg(_)
760:        | InstructionKind::Pull(_)
761:        | InstructionKind::Address(_) => Some("4c"),
```

(The `Parse` spelling is on `:758`; `:761` is the arm's `=>`. Close enough to find.)
Combined with the Global constraint "**Never `git add -A`. Stage the exact paths the task
names.**", an implementer following both instructions literally commits a tree that does
not compile. The same omission is in Tasks 8 and 9.

**IMPORTANT: `tests/trace_oracle.rs` is in the Files list and no step says what to do to
it.** `>.>` is `Coverage::Owned("4c")` at `trace_oracle.rs:531` and must flip to
`Witnessed`. Two committed counters must move with it:

```
551: const WITNESSED_PREFIX_COUNT: usize = 13;
555: const OUT_OF_SCOPE_PREFIX_COUNT: usize = 6;
```

Task 7 must make these 14 and 5; Task 13's Step 6 must make them 16 and 3. Neither task
names either constant. The sum is asserted, so a stale pair fails loudly rather than
silently -- which downgrades this from BLOCKER to IMPORTANT -- but Task 7's implementer
has no step at all pointing at the file their own Files list names.

**IMPORTANT: Step 5 tells Task 7 to write to a file that does not exist and is not in its
Files list.** "Add a `phase-4c.txt` witness and verify." `rust/corpus/` currently holds
`phase-4a.txt` and `phase-4b.txt` only; `phase-4c.txt` is listed under **Task 15's**
Created set. Task 8 and Task 13 both list `corpus/phase-4c.txt` in their Files; Task 7
does not.

**BLOCKER (plan-wide, surfaces first here): nothing ever teaches the harness to read
`phase-4c.txt`.** D6 says "the harnesses read the union of all three." The paths are
hard-coded in two places:

```
tests/corpus.rs:549-550:   &corpus_dir.join("phase-4a.txt"),
                            &corpus_dir.join("phase-4b.txt"),
tests/coverage.rs:618-619: &corpus_dir.join("phase-4a.txt"),
                            &corpus_dir.join("phase-4b.txt"),
```

`tests/corpus.rs` is the differential runner -- the file that actually executes corpus
programs against the oracle. **The string `corpus.rs` does not appear anywhere in the
plan**, and `tests/corpus.rs` is in no task's Files list and not in the File-structure
section's "Modified:" line either. As written, every `phase-4c.txt` witness added by
Tasks 7, 8, 13 and 15 is never run by the differential harness. The witnesses would be
inert and everything would stay green -- the precise vacuity failure mode Task 15's own
Step 4 was added to catch for `phase-4b.txt` ("nine of its twelve entries were deletable
with everything green, including one criterion's only witness").

**MINOR: Step 4's heading says "the four `owners.rs` rows"; the body moves one.** The body
is unambiguous (`Parse` at `owners.rs:165` and its row at `:352`), and the very next
paragraph says "`Arg` and `Pull` stay `4c` until Task 8", so the "four" in the heading is
a leftover. Verified `owners.rs:165` is `InstructionKind::Parse(_) => ("Parse", Owner::Phase("4c"))`
and `:352` is `("InstructionKind", "Parse", "4c")`. ✓

**MINOR: the table at `:352` is called `SPLIT_TABLE`; it is `EXPECTED_OUT_OF_SCOPE`**
(`owners.rs:347`). `SPLIT_TABLE_PHASES` (`owners.rs:378`) is a different thing -- the list
of admissible phase strings. The line number is right, so this is findable.

**Verified correct in Task 7 -- the AST description is accurate in every particular:**

```
ast.rs:1044: pub struct Parse { source: ParseSource, upper: bool, lower: bool,
                                caseless: bool, template: Vec<Option<ParseTrigger>> }
ast.rs:1050-1051: /// `None` is the comma fence between one template and the next
ast.rs:1071: pub struct ParseTrigger { kind: TriggerKind, value: Option<Expr>,
                                       targets: Vec<Option<Expr>> }
ast.rs:1076-1077: /// `None` is a `.` placeholder, which consumes a field and assigns nothing
ast.rs:1083: pub enum TriggerKind { End, Plus, Minus, Absolute, MinusLength,
                                    PlusLength, String, Mixed }   // eight ✓
ast.rs:1057: pub enum ParseSource { Arg, LineIn, Pull, Source, Version,
                                    Var(SymbolId), Value(Option<Expr>) }
```

Task 7's split (`Var`/`Value`/`Arg`/`Source`/`Version` here, `Pull`/`LineIn` in Task 8)
partitions `ParseSource` exactly. The plan never names the enum `ParseSource`, but the
variant spellings match, so it is findable. "This task writes no parser code" is true.

**IMPORTANT (missed dependency, see section 3): `PARSE ARG` at the top level has no
argument source and no task builds one.**

---

### Task 8 -- ARG, PULL, PARSE PULL, PARSE LINEIN

**BLOCKER: `src/lib.rs` again.** Step 3 moves the `Arg` and `Pull` rows; that requires
`lib.rs:759-760` as well as `owners.rs`. Files list: `src/run.rs`, `tests/owners.rs`,
`tests/loud.rs`, `tests/coverage.rs`, `corpus/phase-4c.txt`. No `src/lib.rs`.

**MINOR: "consumes ... 4b's `Interp::queue`" names a private field, not an API.**
`lib.rs:1289: queue: Queue`, with `mod queue;` at `:57`. There is no `Interp::queue()`
method. Findable, but the Interfaces line reads as if there is one.

**IMPORTANT: Step 1's "read stdin when the queue is empty" has no plumbing named.**
`run_program(path: &str, text: Vec<u8>) -> Outcome` takes no stdin handle and returns an
`Outcome` with captured `out`. Whether `PARSE LINEIN` reads the real process stdin,
whether that is testable at all from `tests/`, and how the differential harness supplies
a here-string, are all unstated. Step 1 tells the implementer to *measure* the oracle's
behaviour, which is right, and then says nothing about where our side's bytes come from.

---

### Task 9 -- the ADDRESS instruction

**BLOCKER: `src/lib.rs` again.** Step 2 moves the `owners.rs` row; `lib.rs:761`'s arm
carries `Address`. Files: `src/run.rs`, `src/activation.rs`, `tests/owners.rs`,
`tests/loud.rs`, `tests/coverage.rs`.

**BLOCKER: "delete the `loud.rs` witness" silently drops the Phase 7 half's coverage.**

`loud.rs:208-212`:

```rust
Witness {
    tag: "Address",
    source: "address cmd\n",
    category: Category::Instruction,
},
```

`address cmd` is the *environment-name* form -- `Address { environment: Some(b"CMD"),
dynamic: None, command: None, io: None }` -- which Task 9 puts **in scope**. So the
witness must be deleted. But Task 9 also requires that `command` and `io` "must still fail
loudly naming Phase 7". After the delete, that Phase 7 sub-form has:

* no `loud.rs` witness, and
* no row in `owners.rs`'s `EXPECTED_OUT_OF_SCOPE` demanding one.

`assert_witness_set_is_complete` (`loud.rs:366`) holds the witness set equal to the
out-of-scope table, so with no row there is nothing to be incomplete about -- the Phase 7
half falls out of both policing directions at once. The tree already has the right
pattern for this and the plan already knows it: `owners.rs:349-351` carries the
arm-grained row

```rust
// The one arm-grained row: `CALL`'s other three arms are in scope, so
// they appear in `INSTRUCTION_TAGS` and not here.
("InstructionKind", "Call::Qualified", "Phase 5"),
```

Task 9 needs the same treatment (`Address::Command` / `Address::With` -> `Phase 7`), a
replacement witness (`address sh 'ls'` and/or an `ADDRESS ... WITH` form), and a
`Loud::instruction` path that reports `Phase 7` for those shapes -- i.e. a shape-aware
`instruction_owner`, since today it is one arm for the whole variant. Step 2 says only
"move the `owners.rs` row, delete the `loud.rs` witness". Following it literally produces
a silently-wrong tree that stays green.

**Verified correct:** `ast::Address` (`ast.rs:1225`) carries exactly `environment:
Option<Box<[u8]>>`, `dynamic: Option<Expr>`, `command: Option<Expr>`,
`io: Option<Box<AddressIo>>`. `AddressIo` (`ast.rs:1242`) carries `input`/`output`/`error`
`Redirection`s plus two `OutputOption`s. The plan's description is accurate. ✓
Step 1's placement argument -- per-activation state, "the same shape as 4b's `trace_mode`
move" -- checks out: `Activation` does carry `trace_mode` (`run.rs:3307`,
`caller.trace_mode`, cloned into `Inherited`). ✓

---

### Task 13 -- `::routine` dispatch

**IMPORTANT: "its one construction site (`plan.rs:631`)" is wrong twice.**

The plan asserts, in D-R and again in Step 1:

> `plan.rs:79`'s `directive: Option<usize>` is `None` at its **one** construction site
> (`:631`).

`plan.rs:79` is `pub(crate) directive: Option<usize>` ✓. The rest is not. Searching for
the type name rather than the field name (the field name alone finds only doc comments),
there are **seven** `BodyKey { ... }` construction sites:

| site | `#[cfg(test)]`? |
|---|---|
| `lib.rs:1422` (inside `Interp::run`) | **no -- production** |
| `plan.rs:629` | yes (`mod tests` at `:615`) |
| `eval.rs:1019` | yes (`mod tests` at `:996`) |
| `run.rs:6753` | yes (`mod tests` at `:6736`) |
| `queue.rs:205` | yes (`mod tests` at `:136`) |
| `stem.rs:539` | yes (`mod tests` at `:519`) |
| `trace.rs:827` | yes (`mod tests` at `:644`) |

All seven pass `directive: None`, so "always `None` today" is true. But **the one site the
plan names is a `#[cfg(test)]` test helper** -- `fn activate(interp, program)`, five of
which are near-identical copies across the crate's test modules, with `plan.rs`'s own doc
saying "Copied rather than shared, matching every other test module in this crate". The
production site is `lib.rs:1422`. An implementer sent to `plan.rs:631` finds a test fixture
and either edits the wrong thing or has to rediscover the real one.

A separate consequence the plan does not draw: because six of the seven are test helpers
that each want a *main-body* plan, Task 13's new `Some(index)` is not an edit to any of
them -- it is a **new** construction site at wherever a `::routine` body is entered.
`activation.rs:397-413`'s `body_of` already resolves `Some(index)` against
`program.directives[index]`, and `run.rs:10548` has a test
(`the_body_selector_resolves_a_routine_directive_and_rejects_a_bad_index`) proving it
works -- so the machinery is there and Step 1's framing ("Set `BodyKey::directive`")
understates what is actually needed.

**IMPORTANT: `WITNESSED_PREFIX_COUNT`/`OUT_OF_SCOPE_PREFIX_COUNT` again** (see Task 7).
Step 6 says "Flip `>I>`/`<I<` to `Witnessed` and verify" without naming the two counters
at `trace_oracle.rs:551` and `:555`.

**Verified correct in Task 13:**
* `trace_oracle.rs:542` is `(">I>", Coverage::Owned("4c"))` and `:546` is
  `("<I<", Coverage::Owned("4c"))`. ✓
* `Loud::unresolved_call` is at **`lib.rs:496`**, not `error.rs` -- Task 13's Files list
  does include `src/lib.rs`, so this one is fine. The 128-byte truncation claim is
  supported by that function's own doc at `lib.rs:493`: "The oracle's own 43.1 does not
  truncate, so this is a..." ✓
* `run.rs:3219` is the single call site of `unresolved_call`. ✓
* D-R's three measured facts (own pool, builtins shadow, trace does not cross) are all
  carried into Steps 2, 3 and 4 rather than left in D-R. **This is the one decision in the
  plan that is correctly propagated into its task, and it is worth noting as the model the
  others should follow.** ✓

**MINOR:** Step 5's closing line -- "if the routine's own `trace l` is the only reachable
route, say so in the exclusions file rather than implementing `::options`" -- is a
conditional edit to `phase-4-exclusions.txt`, which is not in Task 13's Files list.

---

### Task 14 -- the compound-DO control-variable fix

**IMPORTANT: the "recorded cost" names a type that cannot do the job, and contradicts the
task's own diagnosis.**

> **Recorded cost:** a `rexx-parse` signature change -- `Controlled::control` carrying the
> `VariableRef` shape an assignment target already does

Both halves are false:

1. **An assignment target is not a `VariableRef`.** `ast.rs:708-711`:
   ```rust
   Assignment {
       target: Expr,
       value: Expr,
   },
   ```
   `VariableRef` is used by the `DROP`/`EXPOSE`/`USE LOCAL` variable *lists*
   (`ast.rs:834`, `:837`, `:858`, `:1207`), never by an assignment.

2. **`VariableRef` carries no tail structure.** `ast.rs:950-956`:
   ```rust
   pub enum VariableRef {
       Direct(SymbolId),
       Indirect(SymbolId),
   }
   ```
   Changing `Controlled::control` (`ast.rs:1014`) from `SymbolId` to `VariableRef` yields
   `Direct(SymbolId)` -- the same flat symbol, one layer deeper. The bug is not fixed.

And the same task, three paragraphs earlier, says:

> **It is not a parse gap**: `cv.j` is a single symbol token and the parser interns
> `"CV.J"` whole.

That is correct -- `ExprKind::Compound(SymbolId)` (`ast.rs:125`) is *also* just a
`SymbolId`, so an expression's compound resolution is a run-time split of the interned
name, not a parse-time shape. Which means the fix plausibly needs **no `rexx-parse` change
at all**: `bind_control` (`run.rs:5271-5275`)

```rust
fn bind_control(&mut self, code: &Code<'_>, control: SymbolId, indent: usize, value: ObjRef) {
    let name = code.symbols.name(control).as_bytes();
    let slot = self.slot_of(name);
    let frame = self.activation().frame;
    self.roots.set_slot(frame, slot, value);
```

needs to route through the same resolution the `Assignment` arm uses for
`ExprKind::Compound`, and the symbol's own class already tells it which. Task 14 tells an
implementer to make a cross-crate signature change to a type that does not carry the shape
needed, while also telling them the parser is not the problem. Whichever answer is right,
the task states both.

**Verified correct in Task 14:** `bind_control`'s use of the flat `self.slot_of(name)` ✓.
`corpus/keyword-exempt.txt` has exactly **790** rows whose `unblocked_by` is `4c` and
exactly **6** whose reason is `defect:compound-do-control-variable` -- counted by field,
not by `grep -c` (which returns 792 because two comment lines mention `4c`). Both of the
plan's numbers are right. ✓ `the_exempt_set_matches_the_current_failures`
(`keyword_assertions.rs:412`) and `REXX_KEYWORD_GATE` (`:134`) both exist, so Step 2's
"that red test is this task's success signal, and it is automatic" is true. ✓

---

### Task 15 -- the L1 harness, the corpus subset, the mutation script

**IMPORTANT: `tests/corpus.rs` is missing from Step 4's scope** (see Task 7's blocker).
Step 4 says "The union of all three subset files is what every harness reads" and lists
only `coverage.rs` in its Files. `corpus.rs:549-550` is the other half and is the one that
actually runs the differential.

**IMPORTANT: D8 is carried by no task at all.** D8 rules that `rexxcps.rex` becomes a
run-to-completion smoke test and that "the real gate [is] the dependency list, each item
with its own differential witness", enumerating twelve items and eight builtins. Searching
the plan: `rexxcps` appears in D8 (lines 148, 150, 153) and twice as a *justification* for
a builtin's behaviour (Task 10's `TRACE()`, Task 12's `TIME('R')`). No task creates the
smoke test, and Step 6's gate amendments -- which list five specific changes to 4b's ten
criteria -- do not include D8's dependency-list criterion. **A decision with measurements,
options weighed, and a ruling, landing in no task body.**

**Verified correct in Task 15 -- D12's measurements are almost all exact:**

| plan's figure | measured | |
|---|---|---|
| `base/bif` files | 78 | ✓ |
| `base/bif` lines | 31,162 | ✓ |
| `assertSame` (token, case-insensitive) | 5,441 | ✓ |
| capital-`A` `AssertSame` in `base/bif` | 0 | ✓ |
| `assertSameList` | 5 | ✓ |
| `expectSyntax` | 1,021 | ✓ |
| `assertTrue` / `assertEquals` / `assertFalse` | 186 / 106 / 51 | ✓ (sum 343 ✓) |
| `base/keyword` capital-`A` `AssertSame` | 510 | ✓ |
| `base/keyword` lowercase `assertSame` | 1,931 | ✓ |
| `FORMAT.testGroup` `::method` bodies | 767, and the largest | ✓ |
| `qualify(` outside `QUALIFY.testGroup` | zero -- only `QUALIFY.testGroup` matches | ✓ |
| `base/bif` `::method` bodies | plan says **4,420**; I measure **4,485** | ✗ |
| `base/keyword` `::method` bodies | plan says **2,105 in 39**; I measure **2,139 in 39** | ✗ (files ✓) |

The two `::method` counts are off by 65 and 34 respectively (I counted
`^\s*::method`, case-insensitive, over `*.testGroup`). Both are informational --
neither is a gate figure, and D12's own conclusion ("roughly 1,900 bodies; **that is an
extrapolation and Task 15 measures the real number**") is explicitly not relying on them.
MINOR, but they are stated as measurements.

`crates/rexx-extract/src/keyword.rs` exists with `DropReason` (`:144`) and its eight
variants and the conservation assertion (`:29`, "its own conservation assertion panics on
the..."), so D12's "reuse over writing" is grounded. ✓
`scripts/mutate-4a.sh` and `mutate-4b.sh` both exist. ✓
`phase_4a_subset_matches_the_committed_list` (`coverage.rs:526`) and
`phase_4b_subset_matches_the_committed_list` (`coverage.rs:580`) both exist, so Step 4's
"the pin 4b's gate found missing for `phase-4b.txt`" describes something since fixed --
the model to copy is right there. ✓

**Step 6's anti-vacuity paragraph is the strongest section of the plan.** "Every criterion
gets the same question asked of it before it is written: what degenerate implementation
satisfies this, and would deleting its subject leave it green?" plus three worked traps,
including "'Each of the 66 names is recognised' is satisfied by a stub returning `''` for
all 66" -- that is precisely the `/bin/true`-with-66-rows failure this project has paid
for. No finding against it.

---

## 3. Task ordering

**The five forced dependencies, each checked:**

| claim | verdict |
|---|---|
| Task 1 before everything ("before anything moves the boundary") | Real. Every later task's Step 4 re-runs its harness. |
| Task 2 before 3-6, 10-12 ("every family task depends on it") | Real, and stronger than stated -- see the BLOCKER: the family tasks depend on Task 2's *text*, which they never see. |
| Task 7 before Task 8 ("needs 7's engine") | Real. `ParseSource::Pull`/`LineIn` are two arms of the enum Task 7 builds the engine for. |
| Task 9 before Task 10 ("`ADDRESS()` needs 9") | Real. |
| Task 13 last ("needs the builtin table complete, for shadowing") | Real, and measured in D-R (`call max 1, 9` with a `::routine max` returns 9). |

**A missed dependency, and it is a real hole: nothing supplies a top-level program's
argument string.**

Three tasks read it and no task builds it:

* Task 7 implements `ParseSource::Arg` (`parse arg a b`).
* Task 8 implements the `ARG` instruction.
* Task 10 implements `ARG()`, `ARG(n)`, `ARG(n,'E'|'O')`.

Inside a routine the source exists -- `CallContext { name, arguments }` (`lib.rs:1299`),
set at `run.rs:3398-3404`. At the **top level** it does not:

```
lib.rs:1639:  pub fn run_program(path: &str, text: Vec<u8>) -> Outcome
src/bin/rexx-run.rs:25-26:  let mut args = std::env::args_os().skip(1);
                            let Some(path) = args.next() else {
```

`rexx-run` takes the path and stops; `run_program` has no argument parameter; `Interp::run`
(`lib.rs:1411`) never sets `call_context`. The oracle's `rexx prog.rex foo bar` makes
`ARG(1)` be `foo bar`. Nothing in the plan says whether 4c plumbs that through, or declares
"the top-level activation has zero arguments" and moves on. Either answer is fine; the
absence is not, because it is also **unfalsifiable by the gate**: the Global constraints'
oracle wrapper is `... /build/bin/rexx FILE` with no trailing arguments, so every
differential witness runs with zero arguments and a wrong answer here stays green forever.

**The plan's own ordering table contradicts a task body on this.** Row 10 says
"`ARG()` needs **8's argument model**"; Task 10's body says "`ARG()` and `ARG(n)` read
**4b's argument model**". Task 8 builds the `ARG` *instruction*, which is a `PARSE` variant
-- it builds no argument model. The body is right and the table is wrong, which matters
because the table is the scheduling artifact.

**The other two specific questions:**

* **Does `PARSE ARG` (Task 7) need anything from Task 8?** No -- given the hole above.
  `ParseSource::Arg` reads `CallContext.arguments` (or the missing top-level source);
  Task 8 adds `Pull` and `LineIn` and the `ARG` *instruction*'s own `upper` flag. Task 7's
  Step 3 ("Keep it that way -- Task 8 adds two sources and must add no engine code") is a
  correct and well-placed constraint.
* **Does `datatype.rs`'s `VALUE` write form (Task 11) need a mechanism no task provides?**
  Yes, in two ways: the growing `Interp::slot_of` (the plan names the non-growing
  `Plan::slot_of` -- see Task 11 above), and a `Loud` constructor for the Phase 7
  external-selector form (none of the five existing ones fits, and `src/lib.rs` is not in
  Task 11's Files list).

---

## 4. Under-specification and placeholders

**No literal placeholders.** "TBD", "add appropriate error handling", "write tests for the
above", "similar to Task N" -- none appears. The plan's own No-Placeholders bar is met on
the letter.

**The seven builtin family tasks: legitimate method, executable only after the section-1
and section-2 fixes.** Full verdict and the six-item list of what they still need is in
section 2 under "Tasks 3-6, 10-12".

**Under-specified elsewhere:**

* **Task 2's hook point** -- contradictory (BLOCKER, section 2).
* **Task 8 Step 1's stdin** -- says measure the oracle, says nothing about where our
  bytes come from.
* **Task 9's Phase 7 half** -- "must still fail loudly naming Phase 7" with no
  constructor, no `owners.rs` row, and a deleted witness (BLOCKER, section 2).
* **D11's seeded-reproducibility body compresses to four words.** D11 states it precisely:
  "`random(mi, ma, se)` followed by 99 unseeded calls must produce the identical sequence
  when the same seed is re-supplied." Task 6 carries it as "**`RANDOM` must be seedable and
  deterministic under a seed** (D11)". A generator that re-seeds on *every* call satisfies
  Task 6's phrasing and fails `RANDOM.testGroup`'s actual body. The 99-call shape is the
  whole content and it is in the decision, not the task.
* **Task 12's `TIME('R')` unit test** -- "must pin the reset semantics, not a value" is
  good; what "the reset semantics" are (does `TIME('R')` return the elapsed time *and*
  reset, or reset and return 0?) is left to the probe. Acceptable given Step 1's method.
* **Global constraints are restated by almost nobody.** The plan's own rule is at line 34:
  "It is not extracted into task briefs, so a task that depends on one of these lines
  restates it." Actual restatement rate: `NUMERIC DIGITS <= 1000` restated once (Task 5);
  D15 restated once (Task 6); `unsafe` restated once (D4's `SETLOCAL` reason); `ulimit`,
  fresh-directory, the stdout/stderr/exit-status separation, `Interp::alloc_with`, and the
  `b2x`/`x2d` literal hazard restated **zero** times, across eleven tasks that probe the
  oracle and seven that allocate on every call.

---

## 5. Scope leaks

* **Task 10 / `ADDRESS()`'s initial value.** Task 9 rules the platform-supplied default
  Phase 7's and the "Explicitly not in scope" list repeats it. Task 10 implements
  `ADDRESS()` with no statement of what it returns before any `ADDRESS` instruction has
  run. The natural implementation returns `sh`, which is the out-of-scope value.
* **Task 9 / `ADDRESS VALUE`.** `dynamic: Option<Expr>` is in scope; the *default* it
  swaps back to is not. Two-deep swap probing (Step 1) will hit the platform default at
  the bottom of the stack.
* **Task 14 / `rexx-parse`.** The task's Files list includes `crates/rexx-parse/src/ast.rs`
  and the plan's own "Modified:" summary line does not. Not a scope leak into an excluded
  area, but the two lists disagree.
* No task requires `ExprKind::List`, `::method`, `::class`, `::requires`, `::attribute`,
  `QualifiedCall`, command dispatch, `ADDRESS ... WITH`, or `TRACE ?`'s pause. Those
  exclusions hold.
* **Not a leak but worth stating:** Task 13's Step 5 discovers that `::options trace
  labels` may be the only route to `>I>`/`<I<`, and `::options` is out of scope. The plan
  pre-answers it ("if the routine's own `trace l` is the only reachable route, say so in
  the exclusions file rather than implementing `::options`") -- this is the right shape and
  the only place in the plan where a possible scope collision is resolved in advance.

---

## 6. Interface citations, consolidated

**Correct as stated** (spot-verified against the tree):
`run.rs:3218` let-else · `eval.rs:529` delegation · `lib.rs:1340` `Argument` ·
`Argument::value()` and its `ARG()` doc · `plan.rs:79` `directive: Option<usize>` ·
`trace_oracle.rs:529` `+++` · `:542` `>I>` · `:546` `<I<` · `:560` `OWNER_PHASES` admits
`"Phase 7"` · `owners.rs:165` and `:352` · `lib.rs:758-761`'s `4c` arm ·
`phase-4-exclusions.txt:84`, `:540`, `:989-1011` · design spec `:71` ·
`ast::Parse` / `ParseTrigger` / `TriggerKind` (all eight) / `ParseSource` /
`ast::Address` / `ast::AddressIo` · `Controlled::control: SymbolId` · `bind_control`'s
flat `slot_of` · 790 + 6 `keyword-exempt.txt` rows · `FORMAT.testGroup`'s 767 bodies ·
all of D12's assertion counts · `QUALIFY` confined to its own group ·
`corpus/phase-4a.txt:18` and `corpus/README.md:119` both say Phase 5 (D7 ✓).

**Wrong or misleading:**

| plan says | tree says |
|---|---|
| `BodyKey::directive` "one construction site (`plan.rs:631`)" | seven sites; `plan.rs:631` is `#[cfg(test)]`; production is `lib.rs:1422` |
| dispatch set = `NAMES` minus `EXCLUDED_BUILTINS` | that is 63; 66 are needed (`coverage.rs:757`'s `- 3`) |
| `Plan::slot_of`'s "idempotent path ... not a bare grow" | `Plan::slot_of` is `&self -> Option<usize>`, cannot create; `Interp::slot_of` (`plan.rs:561`) is the growing one |
| `Controlled::control` should carry "the `VariableRef` shape an assignment target already does" | assignment target is `Expr` (`ast.rs:709`); `VariableRef` is `Direct/Indirect(SymbolId)` and carries no tail |
| produce `Raised::bad_argument_count` / `bad_argument_type` | `Raised::not_enough_arguments` (40.3, `error.rs:379`) and `too_many_arguments` (40.4, `:397`) already exist |
| `phase-4-exclusions.txt:88` says "which 4c will have to meet" | that phrase is at `:124`; `:88` is the `>I>`/`<I<` header line |
| the `SPLIT_TABLE` row at `owners.rs:352` | the const is `EXPECTED_OUT_OF_SCOPE` (`:347`); `SPLIT_TABLE_PHASES` (`:378`) is unrelated |
| `base/bif` has 4,420 `::method` bodies | 4,485 |
| `base/keyword` has 2,105 `::method` bodies in 39 files | 2,139 in 39 (file count ✓) |
| Task 7's Step 4 heading: "the four `owners.rs` rows" | its body moves one |
| Task 1 Step 7: "the six paths above" | five are listed |
| Task 10 row: "`ARG()` needs 8's argument model" | Task 8 builds no argument model; Task 10's own body says 4b's |

**Rows the plan says a task must move or delete -- all exist, and the stated
consequence-if-left-stale is correct in each case:**

* `owners.rs:165` (`Parse`) and `:352` -- Task 7. The pair *is* policed against each
  other: `owners.rs:340-343` says "Any edit to an owner arm above that is not also made
  here is a test failure". ✓
* `owners.rs:166`/`:353` (`Arg`), `:167`/`:354` (`Pull`) -- Task 8. ✓
* `owners.rs:168`/`:355` (`Address`) -- Task 9. ✓ **but** see the Task 9 BLOCKER: a plain
  move loses the Phase 7 sub-form.
* `loud.rs:194-197` (`Parse`), `:198-202` (`Arg`), `:203-207` (`Pull`), `:208-212`
  (`Address`) -- and Task 7's claim that leaving one stale makes
  `assert_witness_set_is_complete` "fail the other way" is **correct**: `loud.rs:66` and
  `:167` document that it holds the witness tag set *equal* to the out-of-scope table, so a
  witness without a row fails just as a row without a witness does. ✓
* `trace_oracle.rs:529` (`+++`), `:531` (`>.>`), `:542`/`:546` (`>I>`/`<I<`) -- exist ✓,
  but the two committed counters at `:551`/`:555` are named by no task.
* `coverage.rs` -- listed as modified by Tasks 7, 8, 9, 15; the specific rows are never
  identified for 7/8/9.

---

## 7. Summary of what would have to change

Ordered by how much damage leaving it does.

1. Give Tasks 3, 4, 5, 6, 10, 11, 12 their own `### Task N:` headings with the Step 1-5
   block inlined.
2. Put `dispatch`'s signature, the upcased-name rule, the arity-row shape,
   `Argument::value()`, the omitted-position rule, `Interp::alloc_with`, the argument-
   rooting rule, and D15 into each family task's body.
3. Resolve Task 2's hook point: state that the argument evaluation moves above the
   resolution decision, and answer `SIGL`, `>A>` and the depth counter for the builtin path.
4. Fix Task 2's set assertion to 66 and state the three partial rows by name.
5. Decide where the shared `EXCLUDED_BUILTINS` lives and name the file in Tasks 1 and 2.
6. Add `src/lib.rs` to Tasks 7, 8 and 9's Files lists.
7. Add `tests/corpus.rs` to a task -- Task 7's, since it adds the first `phase-4c.txt`
   witness -- and say that `corpus.rs:549-550` and `coverage.rs:618-619` both grow a third
   path. Add `corpus/phase-4c.txt` to Task 7's Files.
8. Rewrite Task 9's Step 2 to add the arm-grained `Address` Phase 7 row and a replacement
   witness rather than only deleting one.
9. Move D4's per-row-reason obligation into a Task 1 step, and D8 into a task of its own or
   into Task 15's Step 6.
10. Correct the four wrong interface facts: `BodyKey`'s construction site, `Plan::slot_of`,
    `Controlled::control`/`VariableRef`, and the two already-existing 40.x raisers.
11. Name `WITNESSED_PREFIX_COUNT` and `OUT_OF_SCOPE_PREFIX_COUNT` in Tasks 7 and 13.
12. Restate the `ulimit` wrapper and the fresh-empty-directory rule in every probing task,
    and the `b2x`/`x2d` literal hazard in Task 5.
13. Decide and state the top-level program argument question.
14. Carry D11's 99-call re-seed shape into Task 6.
