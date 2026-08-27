# SDD ledger -- plan: docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md

## Pre-flight

Base `8b4a7459d`, corpus 251 of 251, all five gates green, tree clean.

**The plan was written from measurement, not from the previous plan's prose.** All five rows were run
against the oracle and the crate before any task text existed, and each row's sub-mechanisms were
probed separately so no task is sized from an assumption. Three findings changed the sizing:

* **Three of the five rows end in a deliberate error**, so they are smaller than "build the
  mechanism": `xscope` is oracle rc 159, `rexxinfo` is rc 159, `attribute__external__subkeyword` is
  rc 166.
* **`.RexxInfo` is an instance, not a class.** `.RexxInfo~class~id` is `RexxInfo`,
  `~class~superClass~id` is `Object`, `~string` is `a RexxInfo`, and `~id` on it raises 97.1. So
  Task 4 needs a class off the environment-reachable path plus a pre-built instance, which is what
  the spec says and what `DEFERRALS` already records -- not an environment-reachable class.
* **`xscope`'s error tail already matches byte for byte.** `say .sub~method("BASEONLY")` is rc 159
  with identical stderr on both sides today, so Task 2 is `Method~scope` and nothing else.

Also probed and already working, so not in any task's scope: `Class~baseClass`, `Class~superClass`,
`Array~makeString`, `Method~class`, `.Method~id`, and `~define` with a method object.

**Task interaction scan.** Tasks 1, 2, 4 and 5 touch disjoint mechanisms and no file pair is shared
in a way one task's output feeds another's input. Task 3 is the exception and its row is written into
the task: a class built at run time by `~subclass` meets machinery built for directive-installed
classes, so **which package it belongs to** and **whether the REXX_DEFINED lock covers it** are
decisions Task 3 must measure rather than inherit. Task 6 consumes all five. No task contradicts the
plan's global constraints, and nothing the plan mandates is a defect under the review rubric.

## Task 1: complete at `f81131f9d`

`corpus/gate-tables/directives/attribute__external__subkeyword.rex` **agrees** -- verified by me,
byte-identical on stdout, stderr and exit status on both engines, oracle rc 166 with
`90.998 Unable to find external method "GETzzz_no_entry"`. Gates run by me at that commit with
`--no-fail-fast`: fmt 0, clippy 0, release gate 0, **258 of 258 matching**, 102 `test result: ok`,
zero FAILED. Corpus 251 -> 258.

**The task corrected a false paragraph I wrote into the plan, and corrected it in the plan.** I had
written that because the registry exports no `GET*`/`SET*` entry, *every*
`::ATTRIBUTE ... EXTERNAL 'LIBRARY REXX x'` raises 90.998, which made the task look small and
complete. False: the prefix is unconditional only for the both-accessors form and the `::METHOD`
spelling. A `::ATTRIBUTE ... GET` or `... SET` prepends only where the decoded procedure is the
default, so an explicit third word differing from the upcased name resolves unchanged. Measured by
the task and re-measured by me: `::attribute at get class external 'LIBRARY REXX file_separator'` is
**rc 0** and `.k~at` answers `/` on the oracle and on both engines. The task built the resolving
forms too.

**My error's shape, since it is the second of its kind this session:** a measured true premise, a
convenient universal quantifier over the general case, and no probe of the general case. The tell
both times was that the inference made the work smaller, and I did not read that as a reason for
suspicion. See also the hash-order blocker in the previous plan's Task 23.

## Ruling: one consolidated review after Task 5 rather than five per-task reviews

The previous plan reviewed every task and the reviews earned their cost. These five are different in
kind: each is small, each is independent of the others, and each has a **differential acceptance test
I run myself** -- the named gate row byte-identical on three descriptors on both engines, plus the
five gates. That is a stronger instrument for this class of work than a code review, and it is not
available to most tasks.

So: implementers dispatched sequentially, each verified by me at its close, then **one consolidated
review over Tasks 1 through 5** before the flip, hunting what the row-level differential cannot see
-- silent wrong answers outside the row, cross-task interactions, and prose. Task 6's flip keeps its
own negative control regardless.

**Cost if wrong:** a defect that a per-task review would have caught survives to the consolidated
one, where it is more expensive to attribute and to fix. Against that, five review cycles on work
this size is the larger cost, and every implementer is dispatched with an instruction to probe
adversarially past its own row.

## Task 2: complete

`aa96cf05d` (the mechanism) and `637d0dd64` (the plan correction plus three quantifiers the first
commit shipped). Report at `task-2-report.md`.

**Verified by the controller, not taken from the report.** Release binary rebuilt at HEAD, then both
programs run against the oracle and both engines from a fresh empty directory with absolute paths,
three descriptors `cmp`-ed separately:

```
xscope        ir  / tree-walker  oracle_rc=159 crate_rc=159  OUT_SAME ERR_SAME RC_SAME
method_scope  ir  / tree-walker  oracle_rc=163 crate_rc=163  OUT_SAME ERR_SAME RC_SAME
```

Five gates re-run by the controller at `637d0dd64`: **G1 `cargo fmt --all --check` rc 0 and G2
`cargo clippy --workspace --all-targets -- -D warnings` rc 0 are valid readings.** G3, G4 and G5 are
**void** -- see the ruling below -- and are re-read at Task 3's commit, which has `637d0dd64` as an
ancestor. Task 2's implementer ran all five green at this commit; that reading stands unchallenged,
and my row-level differential above is independent of all of it.

**The plan was wrong again, in the same direction.** Task 2's Build paragraph said the field existed
and only a reader was missing. False: `Interp::method_object`, the `Class~method` route, minted the
object and never wrote its scope, so a reader alone would have answered `.nil` for every name a
class's own dictionary holds. The implementer corrected the plan file, which is where the next
reader looks. This is the second paragraph of mine this plan has shipped that was wrong in the
direction of making the task smaller -- Task 1's was the first. **Standing instruction for Tasks 3
through 5: the brief's "what is already there" paragraph is a claim to re-measure, not a premise.**

**Parked for the consolidated review** (found by the implementer, deliberately not changed):

* `gate_table_c.rs` `control` fields cite task numbers of the **superseded** plan -- `xscope` says
  "Task 9", `usingcl` says "Task 7". A sweep, not one row's business.
* `corpus/lang/class_method_own_dictionary.rex`'s opening comment and `gate_table_c.rs`'s "What this
  table cannot see" bullet both say no table C row exercises the scope question. The `xscope` row's
  third line arguably does. Both sentences were equally arguable before this task -- only the row's
  verdict moved -- and they become load-bearing at Task 6's flip.

A false comment block **was** deleted: `class_method_own_dictionary.rex` listed four sends as rc 120
refusals that all now answer, and its second half ("mints a fresh object per send") was false
independently of this task -- measured, `~objectName` set on a method object reads back, and
`~identityHash` compares equal.

## Ruling: Task 4's brief told the implementer to build a silent wrong answer

Pre-measuring Tasks 3, 4 and 5 against the oracle while Task 2's gates ran, I found the plan's Task 4
Build paragraph is false in a way that matters. It says the `.RexxInfo` instance "answers `~class`
and `~string` and refuses everything else with the oracle's own 97.1."

**The oracle refuses almost none of it.** Of the candidate names I probed, the instance answers
`hasMethod` for all but `ID` and `FILESEPARATOR`, and they are live: `~languageLevel` is `6.06`,
`~digits` `9`, `~form` `SCIENTIFIC`, `~fuzz` `0`, `~internalDigits` `18`, `~objectName`
`a RexxInfo`, all rc 0. Only `~id` raises, which is why the row is rc 159.

So building "97.1 for everything else" would answer *"this object does not understand VERSION"* about
an object that understands it perfectly well -- **a silent wrong answer at the crate's own choosing,
in a task whose brief opens by warning about silent wrong answers.**

**Ruling: the unbuilt surface refuses loudly at rc 120, never with 97.1; `~id` keeps its genuine
97.1.** Scope is unchanged -- the plan's "and nothing else" stands. Only the refusal's *shape* is
corrected, because a loud refusal is an honest "not built here" while a 97.1 is a false claim about
the language. Recorded in `task-4-brief.md` with the measurements.

**Cost if wrong:** the crate refuses loudly where the oracle answers, which is a visible gap that
costs a later task some work. The alternative error is invisible and outlives the phase.

Also measured and carried into the briefs: `parse version` is already byte-identical between oracle
and `ir`, so version identity is committed; a runtime-built class has **no** package (`~package` is
`.nil`) and does **not** inherit the REXX_DEFINED lock; `~define` returns no result; a body may be an
array of lines; and a body that does not parse fails at `~define` time with the **method name** where
a file path goes (`Error 35 running bad line 1`), oracle rc 221.

**Pattern, third instance on this plan.** Every one of my "what is already there" paragraphs that has
been checked was wrong, and always toward less work. The briefs for Tasks 3, 4 and 5 now each open by
saying so.

## Ruling: never run the gates while an implementer holds the worktree, and never end a gate wrapper with an echo

Two mistakes of mine in one command, caught within minutes of each other.

**One. The gate run was concurrent with a dispatched implementer, on a shared working tree.** I
started the five gates at `637d0dd64`, then dispatched `gc-task-3` before they had finished, reasoning
that the implementer would spend its first minutes reading and probing and that the only cost was
cargo lock contention. Wrong: the two share the *source tree*, not just `target/`. G4 and G5 failed to
compile with

```
error[E0425]: cannot find type `ClassPackage` in this scope
    --> crates/rexx-exec/src/lib.rs:3080:37
```

`ClassPackage` is Task 3's in-progress work. G3 failed its `-p rexx-exec --doc` target the same way.
So G3, G4 and G5 measured a tree that existed at no commit and are void -- **not** a finding about
Task 2. G1 and G2 completed before the dispatch and are valid.

**Rule: the worktree has one writer. Gates finish before an implementer is dispatched, or they run
after it reports.** The cheap recovery here is that Task 3's own gate run covers `637d0dd64` too,
since it is an ancestor.

**Two. The wrapper reported success over three failures.** The background command ended with

```
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast >$L/g5.log 2>&1; echo $? >$L/g5.rc
echo DONE
```

and its completion notification read **exit code 0**, because a shell's status is its *last*
command's and the last command was the `echo`. Three gates had exited 101. I only saw it because I
read the per-gate `.rc` files rather than the notification -- which is the only reason the practice of
writing each status to its own file is in the constraints at all.

**Rule: a gate wrapper's last statement is the last gate, or it ends with an explicit non-zero
propagation. Never a trailing `echo`.** This is the same defect as piping a gate into `tail` and
reading the pipe's status; it does not look like it, because there is no pipe.

**Cost if wrong:** nil here, caught immediately. Had I not read the `.rc` files, I would have
ledgered five green gates for a tree that never existed and carried that into the consolidated
review as evidence.

## Task 3: complete

`7d04f72a9` (the mechanism) and `edc9e69c3` (control and sizing correction). Report at
`task-3-report.md`.

**Verified by the controller.** Release binary rebuilt at HEAD, then the row and both new corpus
programs run against the oracle and both engines from a fresh empty directory, three descriptors
`cmp`-ed separately:

```
usingcl                  ir / tree-walker  oracle_rc=0    crate_rc=0    OUT_SAME ERR_SAME RC_SAME
class_subclass_factory   ir / tree-walker  oracle_rc=0    crate_rc=0    OUT_SAME ERR_SAME RC_SAME
class_subclass_refusals  ir / tree-walker  oracle_rc=168  crate_rc=168  OUT_SAME ERR_SAME RC_SAME
```

`class_rexx_defined_inherit.rex`, which the report cites as the lock's other half, was checked to
pre-date this task: `git log -1 -- <path>` is `7c3ab1ebf`.

**Both decisions came back as the addendum measured them**, re-run rather than taken on trust: a
runtime-built class has **no** package (`~package` is `.nil`, and it enters no package class table),
and the `REXX_DEFINED` lock does **not** apply to it while still applying to the library classes.

**The task found and closed the silent wrong answer this plan keeps warning about, on its own.**
`Interp::package_object_for` read an *absence* from `class_packages` as "the REXX package", so a
class built at run time would have answered `The REXX Package` at rc 0 where the oracle answers
`.nil`. Fixed with a third state that has one spelling, `plan.rs`'s `ClassPackage::{Program, Null}`.

**A third decision was ruled unobservable and correctly left unbuilt.** Whether the factory should
set `REXX_DEFINED` while the library bootstrap runs: no library source builds a class by message, so
no test can witness either branch. The cheaper arm was taken and the search that establishes it is
recorded with the claim.

### Findings carried to the consolidated review

1. **A new oracle SIGSEGV**, found while building this. `RexxClass::subclass` casts the answer of
   the `NEW` send without checking it (`ClassClass.cpp:1579`), so a metaclass with its own `NEW`
   returning a non-class crashes the C++ interpreter: rc 139, three runs of three, both descriptors
   empty. Bounded by a neighbour that proves it is the cast and not the overridden `NEW`. Written up
   as `corpus/oracle-crashes.txt` entry 8. **No upstream ticket filed -- Moritz's call, and this is
   one signal.** This crate cannot reach the shape.
2. **A class this crate builds is never freed**, and `~subclass` is the first *unbounded* route to
   building one. Measured on `do i = 1 to 200000; k = .object~subclass("k"); end`, both sides rc 0
   and byte-identical on all three descriptors: `ir` peak RSS **4,085,904 kB** against the oracle's
   **109,100 kB**. About 20 kB per class. This is the licensed OOM-divergence axis rather than a
   wrong answer, and no corpus program is near it, but it is the first route where a user program
   controls the count. Fixing it needs a collector that reaches class identities.
3. **`Class~queryMixinClass` is still a loud refusal**, so the mixin flag's only differential
   witnesses are `~baseClass` and `~inherit`'s 98.943. The gap pre-dates this task -- it refused for
   a `::CLASS ... MIXINCLASS` too.
4. **`test result: ok` line counts are not a stable figure** and I have been quoting them. The same
   G3 command printed 102 and 103 on two runs of the same tree, because parallel test targets
   interleave on one pipe and split a `running 0 tests` line. Count test binaries, or read
   `FAILED`/`^---- ` blocks, which are 0 either way.

## Ruling: I re-run G1, G2, G4, G5 myself and skip G3

G4 is G3 plus `REXX_CORPUS_GATE=1`, an environment variable that only *adds* checks -- it turns the
corpus comparison from report mode into gate mode and removes no test. So G4 strictly covers G3, and
running both doubles the most expensive gate to learn nothing. G5 is kept despite looking like a
debug repeat of G4, because the debug build has arithmetic overflow checks that release does not.

**Cost if wrong:** if `REXX_CORPUS_GATE=1` ever *changes* behaviour rather than adding to it, a
release-mode-only failure would go unseen by me -- though not by the implementer, who runs all five.

**Controller gate run at `edc9e69c3`, serialised, nothing else holding the tree.** Wrapper ends by
propagating the sum of the four statuses rather than an `echo`.

| gate | command | exit |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| G4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | 0 |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 |

`/bin/grep -ah '^[0-9]* of [0-9]* matching'` on both logs -> **`261 of 261 matching`**.
`/bin/grep -ac 'FAILED'` and `/bin/grep -ac '^---- '` -> **0** on both.
`/bin/grep -ah 'usingcl'` on G4's log -> `agree  loud=no  5a  usingcl Using Classes`.

This run also covers `637d0dd64` and `f81131f9d`, which are ancestors, closing the void Task 2 run.

The stale control-field task numbers are confirmed still present: G4's log prints
`usingcl 5a answer ~subclass with a class whose superclass is Object rather than the receiver -- Task 7`.
Still parked for the consolidated review.

## Ruling: gates move to a dedicated worktree, and run in two lanes

Moritz asked whether the gates could run in parallel. Measured first, from my own run's file
timestamps rather than an estimate:

| gate | wall |
| --- | --- |
| G1 `cargo fmt --all --check` | 1 s |
| G2 `cargo clippy` | 1 s, fully cached |
| G4 release corpus gate | ~19 min |
| G5 debug corpus gate | **~64 min** |

Inside G5, `/bin/grep -a 'finished in' g5.log | sort -rn` gives **2026.49 s** for one test binary
(`ir_dual`, 9 tests) and 1040.44 s for the next. So the serial total was ~84 min and one binary was
34 min of it.

**Built:** `/home/moritz/dev/repos/ooRexx-gates`, a detached worktree, driven by
`scratchpad/controller-gates.sh <sha> <logdir>`. It gates a **commit**, not a working tree, which is
what I should have been measuring all along. Two lanes with separate `CARGO_TARGET_DIR`s, because
cargo locks a target directory and two builds in one would simply serialise: lane A is G1 -> G2 ->
G5, lane B is G4.

`ootest/` and `oodocs/` are **git-ignored**, so a fresh worktree does not get them; they are
symlinked in. Their absence fails **loudly** -- `suite_sources` asserts `no .testGroup files under
{} -- the ootest checkout is missing base/{suite}` -- so this cannot produce a silently empty gate.
Checked before building anything.

**Both failure paths tested before trusting it**, since a gate script that cannot fail is worthless:
a nonexistent SHA exits 2; a dirty gate worktree exits 2 and refuses to check out, so it can never
destroy work left in the wrong tree. The guard's first version was broken in the safe direction --
it counted my own `ootest`/`oodocs` symlinks as dirt and would have refused every run -- and is now
narrowed to exactly those two lines, not a wider pattern.

Cargo's target directories now live **outside** the gate worktree, at `ooRexx-gates-target/{dbg,rel}`.
Inside, they were untracked, and the cleanliness guard cannot tell cargo's output from someone's
work; the first real run refused on 1.1 MB of `?? rust/target-dbg/...`.

## The trailing `echo` again, this time inside the mitigation

I ledgered the rule this morning: **a gate wrapper's last statement is the last gate, never an
`echo`.** Then I wrapped `controller-gates.sh` -- the script written to enforce exactly that, which
propagates the sum of its four statuses correctly -- in

```
$S/controller-gates.sh ... ; rc=$?; date +%s > ...end; echo "runner_exit=$rc"
```

and the completion notification read **exit code 0** while the script had exited **2**. The script
was right; my wrapper laundered it.

**Fix at the type level, not the prose level**, because a rule I wrote and then broke within the hour
is not a control: the timing now happens *inside* the script, so no caller needs a wrapper at all,
and the invocation is the background command's only statement. Same lesson as
`dispatch-prose-is-not-a-control`: the third instance shipped from a warning about the second.

**Not taken, and offered to Moritz rather than done unasked:** the corpus harnesses loop over the
programs serially inside a single `#[test]` -- `/bin/grep -rn 'rayon\|thread::spawn\|par_iter'
crates/rexx-exec/tests/*.rs` matches nothing, and `rayon` is in no `Cargo.toml` in the workspace. On
32 cores that is worth more than everything above. It changes test infrastructure mid-plan, and
concurrent oracle runs need one directory each because the scratchpad root is on the oracle's own
external-routine search path, so it wants its own task after the flip.

### Two defects in the gate runner, both found by running it rather than reading it

**One. `CARGO_TARGET_DIR` broke a test that was right.** The two-lane split originally used one
worktree with two target directories. Both G4 and G5 then failed at
`-p rexx-bench --bin rexx-bench-suite`:

```
thread 'tests::every_blocked_axis_still_fails_on_this_crate' panicked at
crates/rexx-bench/src/bin/rexx-bench-suite.rs:1363:17:
neither target/release/rexx-run nor target/debug/rexx-run exists. This test runs the blocked
axes through this crate, and skipping instead would let the roles below go unchecked while the
run stayed green. `cargo build -p rexx-exec --bin rexx-run` first.
```

The test looks for the binary by **hardcoded relative path** and **refuses to skip** when it is
absent, which is exactly right and is the discipline this tree is built on. My redirection hid the
binary. Fixed by giving each lane its **own worktree** using that worktree's default `rust/target`,
which is git-ignored and so never looks like dirt either. Elapsed on that run: **4057 s against 84
min serial**, so the two lanes are worth roughly the 19 min G4 was predicted to hide.

**Two. The cleanliness guard was silently disabled.**
`git status --porcelain -uall | grep -v -x -e '?? ootest' -e '?? oodocs'` -- `??` is an invalid regex
quantifier, so with `-x` the pattern is `^\(?? ootest\)$` and the wrapper errors:

```
ugrep: error: error at position 8
(?m)^\(?? ootest\)$\|^\(?? oodocs\)$
        \___invalid syntax
```

It printed nothing on stdout, so `dirt` was empty and **the guard reported clean unconditionally**.
It had fired correctly in an earlier test, which is what made it look sound: `grep` resolves to
`/bin/grep` in some contexts here and to the ugrep wrapper in others, so the same line passes or
silently disables itself depending on where it runs. Fixed with `/bin/grep -v -x -F`. Re-tested in
all three directions afterwards -- both worktrees clean passes and checks out the right SHA, dirt in
worktree A alone refuses, dirt in worktree B alone refuses -- because a guard I have already watched
disable itself once does not get to be trusted on a reading.

### The runner is validated

Run against `edc9e69c3` -- a commit already gated green serially -- **while `gc-task-4` held the
main worktree**, which is the whole claim:

```
g1.rc=0  g2.rc=0  g4.rc=0  g5.rc=0
checked-out-sha  edc9e69c3690b626388c1d2e8f036e22ae70e6ef
elapsed.seconds  3985   (66 min)
/bin/grep -ah '^[0-9]* of [0-9]* matching' g4.log g5.log | sort -u  ->  261 of 261 matching
/bin/grep -ac 'FAILED' and -ac '^---- '  ->  0 and 0, both logs
```

84 min serial and blocking becomes 66 min and off the critical path. From here, a task's gates start
the moment its commit lands and the next task is dispatched immediately.

## Ruling: one-hour waits, and it goes in the constraints rather than each dispatch

Moritz: implementers polling on ~10-minute timers are burning tokens. Told `gc-task-4` directly, and
put it in `global-constraints.md` so Tasks 5 and 6 inherit it -- a rule that lives only in a dispatch
prompt is prose, and today already showed twice that prose is not a control.

The arithmetic: a wake is a full turn plus a context re-read, and G5 alone is ~64 min, so a 10-minute
poll costs six wasted wakes before the first one that could find anything. A backgrounded command
re-invokes on exit anyway, so the long wait is a fallback against a hang rather than the mechanism.

## Task 4: complete

`02f13726e`, one commit. Report at `task-4-report.md`.

**Verified by the controller.** Release binary rebuilt at HEAD, then the row and the new corpus
program run against the oracle and both engines from a fresh empty directory, three descriptors
`cmp`-ed separately:

```
rexxinfo        ir / tree-walker  oracle_rc=159  crate_rc=159  OUT_SAME ERR_SAME RC_SAME
rexxinfo_entry  ir / tree-walker  oracle_rc=159  crate_rc=159  OUT_SAME ERR_SAME RC_SAME
```

Gates dispatched to the parallel worktrees at `02f13726e` **while Task 5 was dispatched into the main
tree** -- the first task where gating and implementation genuinely overlap.

**The 97.1 ruling held, and it falls out of dispatch rather than being special-cased.** `~id` and
`~new` are not in `RexxInfo`'s instance dictionary and not in `.Object`'s, so the lookup misses and
the send is the oracle's own genuine 97.1. `~digits` **is** in the dictionary with no
`NATIVE_METHODS` row, so it falls through to `Loud::native_method` at rc 120. Swept over every name
in `Setup.cpp`'s `RexxInfo` block on both engines -- 58 sends, **0 mismatches**.

**It overrode my dispatch on a judgment call, with a better argument.** I expected `DEFERRALS`'
`RexxInfo` entry to be narrowed; it was **retired**. The reason: `DEFERRALS` records what
`native_classes.rs` does not build, and that module now builds `RexxInfo` completely -- class object,
instance dictionary, superclass edge, metaclass, class-behaviour merge, `REXX_DEFINED`. The missing
*implementations* live in a different table in a different crate and are missing for every class in
the registry (`Class~defaultName`, `Class~queryMixinClass`, `Array~new` are all loud rc 120 and none
of their classes is deferred). Keeping the entry would give the table a second, incompatible meaning
and break the invariant `every_setup_class_is_native_or_deferred_with_a_reason` rests on. Correct,
and better reasoned than my instruction.

### Findings carried to the consolidated review

1. **`RexxInfo~copy` is a refusal on the oracle, not an answer** -- rc 163, `93.970 COPY method is not
   supported for object a RexxInfo`. The one name in the block the oracle does not answer at rc 0.
   Loud rc 120 here; closeable for the cost of one refusal.
2. **A pre-existing divergence, general and untouched by this commit.** `::CLASS K SUBCLASS <a
   non-class environment entry>` is `99.949 "X" is not a valid class` at **translate** time, rc 157,
   on the oracle, and `98.909 Class "X" not found` at **run** time, rc 158, here. The discriminator
   is whether the name resolves to a non-class entry, not whether it resolves at all: `Zork` resolves
   to nothing and both sides agree on 98.909. Measured over `NIL`, `TRUE` and `ENVIRONMENT`, all of
   which were in `.environment` before this commit. `.RexxInfo` joins an already-divergent set and
   answers what it answered before, so nothing here changed.
3. **`identityHash` on a `Body::Native` receiver**: `datatype(.environment~identityHash, 'W')` is `1`
   here and `0` on the oracle. Pre-existing and licensed at `native_identity_hash`'s own doc.
4. A handful of `RexxInfo`'s methods read off constants the crate already holds (report 6.1) --
   sizing for the consolidated review, deliberately not widened into this task.

## The gate runner had no lock, and Task 5 caught it

`gc-task-5` launched `controller-gates.sh` for its own commit while the `gates-t4` run was still in
its g5. Both gate worktrees were **clean** -- a checkout is clean -- so the dirt guard let the second
checkout through, both trees moved to `a4ee9284f` under the first run's cargo, and both g5 lanes then
built in the same `target/`. **`gates-t4/g5` is void**: a mix of two commits, with nothing in its
output saying so.

Verified rather than taken on report: pid 2524375 alive at 47 min still running
`cargo test --workspace --no-fail-fast`; both worktrees at `a4ee9284f`; `gates-t4` g1 and g2 stamped
01:33, g4 01:50, g5 absent. So g1/g2/g4 stand and g5 alone is void.

**What the reporter could not see.** Killing a cargo test run **orphans its spawned test binaries**.
Task 5's kill left `assertions-1d9a298f335ec15e` running in the debug worktree and
`assertions-b7e90b25bf7b1601` in the release one, both reparented to init, both still active in the
gate trees -- so the interference outlived the kill that was meant to stop it. Killed the void run's
whole process tree and both orphans; `pgrep -af '<worktree>/rust/target'` is the check.

**Fix: `flock` on `/home/moritz/dev/repos/.oorexx-gates.lock`**, taken before the dirt check, held for
the run, released on exit. A concurrent run **refuses** rather than queueing, because queueing hides
the collision behind a long wait. Tested both directions -- free lock proceeds and checks out; held
lock exits **3** with the holder's pid named, **and the worktrees stay at the first run's SHA**.

**The guard I already had could not have caught this**, and that is the lesson worth keeping. It asks
"is this tree dirty", and the second run's arrival is not dirt -- both trees were pristine checkouts.
A guard that tests the wrong property passes honestly and protects nothing. Shared state needs a
lock, not an inspection.

## Ruling: do not re-run Task 4's gates; gate Task 5's final commit instead

`git merge-base --is-ancestor 02f13726e a4ee9284f` confirms the ancestry, so gating Task 5's final
commit covers Task 4's tree. Same argument that recovered the void Task 2 run. Task 4's row-level
differential was verified by me directly and depends on no gate run.

**Cost if wrong:** a defect that Task 4 introduced and Task 5's tree happens to mask would go
unattributed -- though not undetected, since the gate is on the final tree either way.

## Task 5: mechanism verified, gates in flight

`a4ee9284f` (the mechanism) and `71327dc46` (two doc comments, no code). `71327dc46` is the SHA to
gate and the one the report names. Gate run pid 2750762 live at that SHA, holding the lock, logdir
`scratchpad/t5probe/gates3-71327dc46`. Three earlier logdirs there are renamed `VOID-*` by the
implementer so no status can be read out of them.

**Verified by the controller.** Release binary rebuilt at HEAD (`Compiling rexx-exec ... Finished in
31.13s`, so not a cached read of an older binary), then:

```
methna                    ir / tree-walker  oracle_rc=0    crate_rc=0    OUT_SAME ERR_SAME RC_SAME
method_from_source        ir / tree-walker  oracle_rc=163  crate_rc=163  OUT_SAME ERR_SAME RC_SAME
method_from_source_table  ir / tree-walker  oracle_rc=163  crate_rc=163  OUT_SAME ERR_SAME RC_SAME
```

**All five 5a gate rows now agree.** The plan's mechanism tasks are done.

### The plan's control for this row could not fail, and that is the second wrong premise it carried

**One.** The control as written -- "keep the as-written spelling as the dictionary key" -- is a
**no-op at the site it names**. `MethodDict` upcases on insert, and its lookups upcase separately in their own functions, so changing
`~define`'s half alone leaves `methna` at `agree` and the corpus at 264 of 264. The implementer
measured that twice before concluding it, then corrected both the plan and gate table C's own
`control` string: **both** sites have to stop upcasing together -- `dispatch.rs`'s `method_name_pair`
and `MethodDict::replace_method` -- and then exactly one row reddens.

**Two.** The plan said a compiled body "is real Rexx and reaches the interpreter". Measured wrong:
`~define` and `~defineMethods` install into a class's **instance** dictionary, and with `~new` unbuilt
no send this phase can make reaches one. The row's own bodies are never run. The single route that
could reach such a body is `~subclass`'s class-method table, where the oracle reports a failure
against the **method** rather than a file -- rc 214, `Error 42 running M line 1:`. Nothing here
answers `running M`, so Task 5 keeps no compiled body and both halves of that route refuse loudly.

This is the [[gate-criteria-failure-modes]] shape: a criterion that is achievable, states something
true-sounding, and cannot discriminate. It was caught by *running* the control rather than reading
it -- which is the only thing that catches this family.

## Task 5: complete

`a4ee9284f` + `71327dc46`. Report at `task-5-report.md`.

**Gates verified by the controller, read from the `.rc` files rather than the report:**
g1=0 g2=0 g4=0 g5=0, `checked-out-sha` `71327dc46d9dd6e234e0dc86ba0fe3ee8716339b`, `elapsed.seconds`
3911. `/bin/grep -ac '^failures:$'` is 0 in every log. Corpus `264 of 264 matching`. G3 was run
separately by the implementer in its own worktree and also 0.

**Counted myself, and this is Task 6's key input:**

```
/bin/grep -hcE '^  (agree|diverge-[a-z]+) +loud=(yes|no) +5a ' g5.log  ->  171
/bin/grep -hcE '^  agree +loud=(yes|no) +5a ' g5.log                   ->  171
/bin/grep -ahE '^  diverge-[a-z]+ +loud=(yes|no) +5a ' g5.log          ->  nothing
```

**Every 5a row in both tables agrees. No sixth row.** Both tables currently print `gated by this run:
0 row(s)`, which is what the flip changes.

**It checked my G3 ruling rather than taking it, and the ruling holds.** It read every reader of
`REXX_CORPUS_GATE` instead of grepping and assuming: `gate_mode()` is used at two sites, to label the
report and to guard the final verdict assertion, with the structural check above it unconditional;
`gate_tables/mod.rs` has the same shape; `oracle_deadline.rs` runs **only** with the variable set, so
G4 has a test G3 skips rather than the reverse; and `/bin/grep -rn "!gating()\|!strict\|!gate_requested"
crates/rexx-exec/tests/` matches nothing, so no test body runs only when the variable is absent.
**G4 strictly covers G3**, now verified rather than argued, and written into the report so nobody
re-derives it. It had already run G3 before my ruling arrived, so its 0 is recorded rather than
discarded.

### Two design decisions worth the reviewer's attention

1. **The compiled body is validated and then discarded.** Instance-side dictionary entries are
   unreachable while `~new` is unbuilt. The one reachable route to a compiled body -- `~subclass`'s
   class-method table -- reports failures as `Error 42 running M line 1:`, naming the **method** where
   a program names its path, and every `FailureSite::Clause` construction would have to learn that.
   So that route refuses loudly rather than running a body whose failure report would be wrong.
2. **`Method~new` is not built.** The strongest of its four reasons: the `.nil`-scope half of the
   Task 2 interaction is already witnessed at rc 0 by `.methods~z~scope`, so the factory adds no
   witness -- and its gate row is `METHOD_PHASE = 5c`.

### Two divergences found in passing, owned by nobody

* **`INTERPRET`'s parse-failure transcript**: the fragment clause echo is missing, and a prefix
  operator renders as `Prefix operator "&1"` instead of `"+"`, because `ParseError` carries no
  substitution values. Same gap that makes the `~define` parse failure a loud refusal here.
* **`condition('A')`** refuses loudly where the oracle answers the substitution array.

Both are for the consolidated review to file, not for Task 6.

## Task 6 dispatched

Base `71327dc46`. Working tree clean, both gate worktrees clean at that SHA, no stray processes,
gate lock free -- all four checked before dispatch.

## Task 6: the flip, and the control re-run by the controller

`743dd4109`, one line: `CLOSED_PHASES: &[&str] = &[]` -> `&["5a"]`. Report at `task-6-report.md`.

**I ran the negative control myself rather than reading it**, in the main tree, with both files backed
up to the scratchpad first and their sha256 recorded. Both upcasing sites dropped together --
`dispatch.rs`'s `method_name_pair` and `MethodDict::replace_method`:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast
mutated   exit 101   table C 13 passed 1 failed   table D 15 passed 0 failed
          the only non-agree 5a row: `diverge-both  loud=no  5a  methna Method Names`
          one table `gated by this run: 1 row(s)`, the other 0
restored  exit 0     table C 14 passed 0 failed   table D 15 passed 0 failed
          `agree  loud=no  5a  methna`, both tables `gated by this run: 0 row(s)`
```

**A nameable set of exactly one row**, which is what the plan asked for and what a control that broke
the bootstrap could not produce.

### The inversion failed the first time, and the cause was the restore

The restored tree still exited 101 with `methna` diverging. The content was correct -- `/bin/grep -c`
found the upcasing back in both files -- but **`cp -p` preserved the originals' older mtimes**, so the
restored sources were stamped 04:09:41 against a `target/release/rexx-run` of 05:22:49. Cargo's
fingerprint saw a binary newer than its source, skipped the rebuild, and the gate table ran the
**mutated binary against restored source**. `touch` on both files then `cargo build --release --bin
rexx-run` recompiled `rexx-classes` and `rexx-exec` in 30.84s, and the same command exited 0.

A restore that is byte-correct and artifact-stale reads as a failed inversion -- that is, as evidence
against a control that was in fact working. **`cp -p` is the trap**: preserving mtime is exactly what
defeats mtime-based fingerprinting.

### And the write of this very section failed silently the first time

The heredoc above was first run with a relative path from a shell whose cwd had drifted to `rust/`
after an earlier `cd`. The redirect failed with `No such file or directory`, and the *other* command
in the same call printed its success message, so the call read as a partial success rather than a
lost write. Caught by re-reading the file's line count instead of the exit status -- which is the
same rule as reading gate statuses from their own `.rc` files.

### Gates green at the flip

```
743dd41096182e16abf45ca10413b9371f8a016b   elapsed 3856s
g1=0 g2=0 g4=0 g5=0        failure blocks 0 in every log
264 of 264 matching
5a rows: total 171, agree 171, diverge none
  5a: 135 rows, 0 not yet `agree`      (table C)
  5a: 36 rows, 0 not yet `agree`       (table D)
gated by this run: 0 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
```

**Phase 5a is closed and gating.** All six tasks complete.

## Consolidated review: two dimensions clean, five prose findings, zero high

Report at `review-report.md`. Range `8b4a7459d..743dd4109`, 52 files, +3110/-346.

| dimension | verdict |
| --- | --- |
| silent wrong answers outside the rows | **clean** |
| cross-task interactions | **clean** |
| prose | **5 findings** -- 0 high, 2 medium, 3 low |

**The ruling to review once at the end rather than five times held.** The dimension a per-task review
would have owned -- code defects in the task's own diff -- came back clean twice over, and everything
found was prose, which is exactly what a consolidated pass sees better because it can compare files
committed by *different* tasks. F1 is the proof: two corpus programs committed by Tasks 2 and 5
disagree about what `~defineMethods` does, and no per-task review could have seen the pair.

* **F1 (medium).** `method_scope.rex` says `~defineMethods` "copies ... leaving the object it was
  handed alone". Measured false: a scope-less object is filled **in place**, exactly as by `~define`.
  Its `define-methods-copies` row only witnesses a copy because that object's scope was filled two
  rows earlier. Task 5's `method_from_source_table.rex` states the mechanism correctly and
  oppositely. **The code is right on both sides.**
* **F2 (medium).** The stale task-number citations are **26 lines, not the two the ledger parked**,
  spanning at least three plan generations, and exactly one line in the file names a plan path at all
  -- so none is resolvable. This matters more after the flip: a `control` field is now the
  reproduction instruction for a **gating** row.
* **F3 (low).** Both "no table C row exercises the scope question" sentences are false; `xscope` is a
  scope row and now gates, so "the whole of the protection" is wrong too. Neither is a regression from
  this plan, but Task 2 edited that file's tail and left this sentence four lines from its top --
  the correction-rounds shape again.
* **F4 (low).** `rexxinfo_entry.rex` calls `.RexxInfo` "the one `.environment` entry that is a
  pre-built instance rather than a class object". Measured false -- `NIL`, `TRUE`, `LOCAL`,
  `ENDOFLINE` and `ENVIRONMENT` are all pre-built instances.
* **F5 (low).** `methna`'s new `control` text says `MethodDict::replace_method` "upcases every key on
  insert and on lookup". It upcases on **insert** only; the lookups upcase in their own functions.
  **The control still works exactly as written and Task 6's run proves it** -- imprecise attribution,
  not a broken criterion. My ledger and Task 6's brief quote the same imprecise sentence; mine is
  corrected below.

## Ruling: F2 is fixed by deleting the citations, not by renumbering

None of the 26 resolves to a document. A citation that cannot be resolved is worse than none, because
it sends a reader looking for a plan that no longer exists at the moment they most need it -- to
reproduce a red on a gating row. Delete the task numbers, keep the substance.

**Cost if wrong:** provenance is lost for fields that had it in principle. Against that, the
provenance was already unrecoverable and the fields' substance is self-contained.

Fix round dispatched with deletion-not-rewriting as its governing instruction and the measured
correction-round hazard stated up front.

## Fix round: five findings, deletions, and one residual adjudicated

`fca800c98`, net **-27 lines** across 9 files -- almost entirely deletion, which is what the round was
instructed to prefer. Report at `fix-round-report.md`.

**Its verification is the part worth keeping.** The three edited corpus programs were re-run against
the oracle on both engines, three descriptors separately, after a rebuild it confirmed had actually
recompiled. The regenerated `sourceline_oracle` expectations were controlled: the driver was run
against the three edited programs **and five untouched ones**, all five reproducing their committed
expectations byte for byte under `cmp -s`. That control is what says the driver is faithful and that
the three regenerated files differ only by the intended edits -- without it, a regeneration is a
rewrite that cannot be distinguished from a correction.

**Two new claims, both measured**, including the row label `define-methods-copies` ->
`define-methods-does-not-rescope`, which is printed by the program and so changes the oracle's stdout
too. Re-verified on both engines at rc 163.

**A trap it found in a documented command.** `sourceline_oracle.rs`'s module comment gives a driver
invocation reading `build/bin/rexx` relative to the repository root. **In this worktree that path is a
different interpreter** -- 16,496 bytes dated Jul 27, against the 62,600 bytes of the constraints'
oracle at `/home/moritz/dev/repos/ooRexx/build/bin/rexx`. It used the constraints' path and checked
`parse version` first. A documented procedure that silently names the wrong binary would regenerate
every expectation in that directory against the wrong reference.

## Ruling: the residual task citation is fixed too, and the ruling is widened rather than overridden

It left `corpus/lang/class_method_class_side_raises.rex:9`, "The frame line is Task 6's", flagging it
for me. Fixed, for three reasons in order of weight:

1. **`rust/CLAUDE.md:107` reaches it**: "Comments say what the code does, not how it got there. No
   task numbers." F2's ruling was scoped to the two gate-table files because that is where the review
   measured, not because the rule stops there.
2. **Both gate tables now read 0** for `/bin/grep -rac 'Task [0-9]'`. Leaving exactly one behind in
   the same neighbourhood is the F3 shape -- Task 2 fixed a file's tail and left a false sentence four
   lines from its top. A round that repairs 26 citations and leaves 1 invites the next reader to read
   the survivor as deliberate.
3. **The file gates.** A reader reaches that comment while reproducing a failure.

**Held the scope explicitly**: that one comment, its expectation, its differential, and nothing else.
The `D50` set-size sentences stay -- they predate the round, they are not task citations, and widening
a fix round is how this project introduces new false statements.

**Cost if wrong:** a fourth corpus program's expectation is regenerated for a comment, which is churn
in a file that gates. Against that, the rule is explicit and the alternative is a known violation left
standing in the round that fixed its 26 siblings.
