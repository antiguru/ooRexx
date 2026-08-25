# Task 12 review: `UNKNOWN`, and the NOMETHOD condition

Range reviewed: `c18b877e3..8cdc40820`. The tree at review time is `4b836188a` (one document-only
commit past the range, the controller's spec note); no crate source differs between the two, and the
release binary was rebuilt from the pristine tree before and after every mutation below.

## Verdicts

**Spec compliance: PASS.** Every clause of the brief is built and measured. The `UNKNOWN` step sits in
front of the miss arm; the receiver's own behaviour is what is consulted, with no start scope; the
forward carries the missed name and an `Array` of the send's own arguments; the NOMETHOD condition
sits beneath it, trapped and untrapped. `Loud::unknown_forward` is removed rather than left as a
refusal, which is what the brief asked for ("it must remove that gate rather than a wrong error").
`message_send_unknown_method.rex` is untouched and still agrees. The `unkno` row reads `agree` and
its deletion control is recorded and reproduces. Both required program shapes are used as written
(returning arm as an expression, args-array arm as a clause). Nothing outside the task's scope is
touched: the diff reaches only `rexx-exec`'s `dispatch.rs`, `error.rs`, `lib.rs`, `run.rs`, the
coverage subset, two corpus programs with their `sourceline_oracle` files, and the baseline TSV.

**Task quality: PASS with required corrections.** The code is right, the rule is the C++'s rule, and
every instrument I could think to attack held. Two shipped comments state something false, one of
them the exact reading the task's own fix round withdrew; both are one-line prose corrections with no
behaviour change. Three further findings are non-blocking.

## What blocks

### 1. `rust/corpus/phase-5a.txt:359-361` carries the withdrawn stack-walk reading

The `condition_nomethod.rex` entry says:

```
# degraded to the 97.1 syntax error a SIGNAL ON SYNTAX takes. The offer walks
# the whole activation stack before it degrades, so a caller's NOMETHOD handler
# beats a callee's SYNTAX handler -- ...
```

The offer does not walk the activation stack. `0ce35233e` replaced the walk with
`Interp::trap_for` -- the running activation's own table -- precisely because the walk gives a wrong
answer, and the walking build is what the same program's `methodmiss` row now reddens. The observable
half of the sentence ("a caller's NOMETHOD handler beats a callee's SYNTAX handler") is true for an
internal `CALL`, but the mechanism it gives as the reason is the one that was withdrawn.

How verified: `git log -S"whole activation stack" -- rust/corpus/phase-5a.txt` returns `ee6ebbf64`
only, and `git show --stat 0ce35233e` does not list `phase-5a.txt`, so the fix round never revisited
it. The sentence is still in the file at HEAD (it hard-wraps between `walks` and `the whole`, which is
why a phrase grep for `"walks the whole"` returns nothing -- worth noting for anyone re-checking).

Fix: restate it the way the program's own header comment and `Interp::nomethod`'s doc already do --
the running activation's table decides, and an internal `CALL` inherits its caller's traps by copy
while a `::METHOD` activation inherits none.

### 2. `rust/crates/rexx-exec/src/dispatch.rs:1140` -- "that row panics in `to_text`" is false

The rooting comment says:

```
// Measured with `collect_stress.rs`'s
// collect-on-every-allocation and this root removed: that row panics
// in `to_text` and a name of seven bytes or fewer does not, ...
```

How verified: I removed `self.roots.push_temp(arguments);` and ran
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test collect_stress`, then the same in debug
under `memcap 8G`. Both runs redden `the_l0_subset_passes_again_under_collect_on_every_allocation` on
`lang/message_send_unknown_forward.rex`, and in both the stress run **exits 120 with**
`rexx-exec: a message send to a value whose object is no longer live is not implemented (Phase 5)`.
Nothing panics, and the site is `send_message`'s receiver guard (`dispatch.rs:699` ->
`Loud::receiver_class`), not `to_text`. Same claim is repeated in the report's rooting table.

The **substance the comment is defending is correct**, and I confirmed both halves of it:

* with the root removed and the long-name row intact, the stress subset reddens on exactly that
  program;
* with the root removed and the long name replaced by `zorkx`, the stress subset passes -- so the
  long-name row really is what witnesses this root, and the row earns its place.

Fix: replace "panics in `to_text`" with what the run does -- the row's later `a~items` reaches a
receiver whose object was collected, and the send guard refuses it -- or drop the mechanism and keep
"that row reddens the stress subset and a name of seven bytes or fewer does not".

## What does not block

### 3. Two sittings share `task = 12` *and* `commit`, against the file's own documented discriminator

`rust/bench-baselines/README.md:34` states: "**The `commit` column is the reliable discriminator
either way** -- it names the revision a reading describes, where `task` names only who took it." For
this file that is now false. File lines 5338-5649 and 5650-6141 are two separate sittings, both
`task = 12`, both `commit = ee6ebbf64`.

Measured: 312 rows in the file now carry a `(task, commit, axis, build, scope, arm, size,
instrument)` key that another row also carries with a different value.

```
awk -F'\t' 'NR>1 {k=$1"|"$2"|"$3"|"$4"|"$5"|"$6"|"$7"|"$8; c[k]++; if(c[k]>1) d++} END {print d+0}'
```

Nothing reads the TSV programmatically -- `/bin/grep -rn "phase-5a-arms" --include=*.rs crates/`
returns two prose citations in doc comments and no parser -- so no current consumer gets a wrong
answer. It is a data-hygiene defect against a rule the file documents about itself, not a wrong
measurement.

Not blocking because both sittings are honest measurements and the report discloses the collision.
Fix: relabel the second sitting's `task` column (the label is metadata, not a measured value -- e.g.
`12-prev`), or amend the README sentence so it stops promising something the file does not deliver.
The README's dated enumeration at `:43` ("Run 2026-08-22, that is `6`, `7`, `8` and `9` bare, ...")
is also now incomplete -- it predates Tasks 11 and 12 -- but it is explicitly a dated snapshot and is
the controller's, not this task's.

### 4. The performance conclusion is right; two sentences about it are not

I redid the arithmetic from the `12-fixround-1` `absolute` rows. Every head-over-`prev-c18b877e3`
`instructions:u` ratio, all axes, both arms, both sizes, lies within `2.38e-7` of 1. The widest is
`alloc4c/ir/small` at `1.000000238`. Two corrections to the report:

* **"1.000000 to seven decimal places" overstates the precision.** At seven decimal places
  `1.000000238` is `1.0000002`, and several other rows round to `0.9999999`. The claim is true to
  **six** decimal places, which is still far below anything the guard cares about.
* **The `1.00000024` figure is the third sitting's, not the second's.** In the second sitting
  (`NR` 5650-6141) `alloc4c/ir/small` reads `head/prev = 0.999999823`, and that sitting's widest
  departure is `1.000000032` on `varlookup/ir/large`.

Everything load-bearing checks out. `strings`/`ir` per pass: `pinned 5369.525223`, `head 5421.525283`,
so `+52.000060`, leaving `53.695178 - 52.000060 = 1.695118` -- the report's `+52.0001` and `1.6951`.
The strongest independent confirmation is one the report does not use: for every axis, arm and size the
`pinned>head` and `pinned>prev-c18b877e3` `across_builds` ratios are identical to six decimal places,
so this task's own commits move nothing. No axis reaches 1% against the pin (max `strings`/`ir`
`+0.9685%`), so no layout attribution is owed. `cycles:u` peaks at `7.89%` on `compound/tw` and is
correctly not reported as a result. The pin is live: `sha256` of
`bench-baselines/pinned/rexx-run-15a1ffa98` is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching `PINNED.md:16`, and the
staleness list `15a1ffa98..8cdc40820 -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml`
contains no commit outside this plan's body.

### 5. Set sizes in new comments

The plan forbids naming a set's size in a comment, true counts included. The new prose does it at
`dispatch.rs:1101` ("The last two steps"), `dispatch.rs:1106` ("The forward's two arguments"),
`dispatch.rs:1134` ("only one of the two roots"), `error.rs:1453` ("the three things a trapping
handler reads back") and `corpus/phase-5a.txt:354` ("One class returns from UNKNOWN and one only
says" -- the program actually has a third class whose `UNKNOWN` returns).

Recorded rather than pressed, because the surrounding untouched prose in the same module doc uses the
same shape ("`resolve` and `invoke` are two steps, not one fused send", "the `resolve`/`invoke`
pair") and every one of these counts is fixed by the C++ signature it cites. The controller should
rule whether the rule reaches structural counts of this kind; if it does, all five go, and
`phase-5a.txt:354` is the one that is also imprecise.

### 6. Borderline: a rejected design named in a doc comment

`Interp::nomethod`'s doc ends its `::METHOD` paragraph with "...where a version asking the whole
stack runs the `NOMETHOD` one", and `condition_nomethod.rex:24-25` says the same. By the plan's own
strike test the clause is removable without changing what the sentence says about the code, which
makes it historical framing. Against that, naming the discriminating counterexample beside the rule
is established practice in this tree (`21cde29af`, "Put the metaclass rule's counterexample beside
its test"), and in the corpus program the sentence doubles as the control statement. Left as the
controller's call; I would keep the corpus program's and drop the doc comment's.

## Rulings on the six flagged items

### 1. The corrected rule -- checked against the C++, not the report. Correct.

`reportNomethod` (`concurrency/ActivityManager.hpp:509`) does call `raiseCondition`, and both
`Activity::checkCondition` (`concurrency/Activity.cpp:648`) and
`Activity::raiseCondition(DirectoryClass *)` (`:683`) *do* loop over stack frames -- which is what
made the first reading plausible. What both loops also do, at `:661` and `:696`, is

```cpp
// for a normal condition, we stop checking at the first Rexx activation.
if (isOfClass(Activation, activation)) { return false; }   // resp. break
```

so the walk terminates at the first `RexxActivation` and only the running activation's trap table is
ever asked. The shipped rule is the C++'s rule. Every other citation in the new code re-checked and
exact: `ObjectClass.cpp:904` (`messageSend` -> `processUnknown`), `:1002` (`processUnknown`), `:1005`
(`behaviour->methodLookup(GlobalNames::UNKNOWN)`, no start scope), `:1013` (the argument array),
`:1018`-`:1019` (name then array), `ActivityManager.hpp:509`, `TrapHandler.cpp:118` (the `CALL ON`
refusal, which applies to an `ANY` trap -- an explicit `CALL ON NOMETHOD` never parses).

The neighbourhood the shipped code does not name, all run through the required wrapper from a fresh
empty directory, three descriptors read separately, both engines. All **MATCH** byte for byte:

| probe | oracle and crate |
|---|---|
| `signal on nomethod` + `signal on syntax` in the main body, miss inside a `::ROUTINE` body | rc 0, the **SYNTAX** handler -- a `::ROUTINE` activation inherits no traps either |
| same, miss inside a method called from a method | rc 0, the SYNTAX handler |
| trap armed **inside** the `::METHOD` body, miss there | rc 0, `mh C=NOMETHOD D=ZORK`, method returns `trapped` |
| `say .plainx~unknown` (the missing name *is* `UNKNOWN`) | rc 159, 97.1 on `"UNKNOWN"` |
| miss inside a `signal on nomethod` handler that already fired | rc 159, 97.1, handler not re-entered |
| `call on error name h` armed, miss | rc 159, 97.1 |
| `call on any` vs `signal on any` over the same miss | rc 159 untrapped vs rc 0 `NOMETHOD NOSUCHMSG` -- the measurement `Interp::nomethod`'s doc states |
| `signal on nomethod` armed, receiver *has* `UNKNOWN` | rc 0, the forward wins, no condition |

One shape is unreachable this phase rather than matching: an **instance-side** `::method unknown`
needs `~new`, which is still `rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)`.
The corpus therefore covers only the class-side arm. That is the phase's boundary, not this task's
gap, and no code here branches on the arm.

The report's item "`::ROUTINE` bodies were not probed for trap inheritance" can be closed: they were
now, and they behave as `::METHOD` does.

**The mutation claim holds and is tight.** I re-implemented the stack walk in `Interp::nomethod` from
`ee6ebbf64`'s own `trapped_anywhere` body and ran
`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`. Exactly one test fails --
`corpus_differential`, `155 of 156`, `lang/condition_nomethod.rex: stdout differ` -- and the diverging
line is exactly the `methodmiss` row (`mmn reached` here against the oracle's
`mms C=SYNTAX rc=[97]`). Nothing else in the gated release suite moves. So the program is what
reddens, it reddens alone, and it reddens on the row the report names.

I also ran the report's second control (`Interp::nomethod` never naming NOMETHOD). It reddens
`corpus_differential` on `lang/condition_nomethod.rex` and
`the_l0_subset_passes_again_under_collect_on_every_allocation`, and nothing else -- the report's
failing set exactly. That is what makes the program coverage rather than merely fallible.

And the `trap_for` widening: reverting `self.running_activation()?` to `self.activation()` panics
`dispatch::tests::a_class_identity_used_as_a_receiver_answers_from_the_class_side` and
`dispatch::tests::an_unimplemented_method_is_loud_where_an_unknown_one_is_a_condition` at
`activation.rs:1214`, the two tests and the site the report names.

### 2. `.environment~nosuch`'s new refusal. Right refusal; one in-crate test is what the plan allows.

The refusal moved from the search step to `Directory`'s own `UNKNOWN` body, and that body really is
the thing still missing: `corpus/docs/class-methods.txt:617` carries
`Directory<TAB>unknown<TAB>instance<TAB>covered`, so the name is in the registry with no native
behind it, and the oracle's `The NIL object` at rc 0 comes from running that body. Refusing loudly
there is correct under the standing rule that a refusal beats a wrong answer a program could trap.

One in-crate test is acceptable here, for two reasons. The plan's own constraint says so -- "If the
honest answer is 'an in-crate test only', the task says exactly that" -- and the report says exactly
that, twice, including under "what the checks could not see". And the test is a lib unit test, so it
is red in a plain `cargo test` with no gate variable; I confirmed it reddens under the D50 control
(`left: (159, "") right: (120, "")`). `/bin/grep -rn 'Directory' --include=*.rs crates/` filtered for
`of class` returns that assertion and its own doc comment and nothing else, so the report's
uniqueness claim holds.

Residual risk is real but is a handoff note rather than a defect: Task 17 looking for the old string
will not find it. That belongs in Task 17's text, not in this diff.

### 3. `CALL ON NOMETHOD`. Both halves verified; not this task's.

*Untouched*: the diff reaches no `rexx-parse` file at all (`git diff --stat c18b877e3 8cdc40820`).

*Generic*: run side by side, `call on nomethod name h` and `call on syntax name h` produce the
**byte-identical** crate refusal, `rexx-exec: 25.1: Invalid subkeyword found.` at rc 120, where the
oracle reports rc 231 with the clause echo and the keyword list (`...found "NOMETHOD"` /
`...found "SYNTAX"`). So this is the crate's standing parse-refusal surface, reachable through any
illegal `CALL ON` condition, and correctly left alone. Adding no corpus program for it is right --
there is no differential row to write for a refusal the oracle does not share.

### 4. Two sittings sharing `task = 12`. Finding, non-blocking -- see 3 above.

### 5. Performance. Supported; two prose corrections -- see 4 above.

### 6. Table C, 117 -> 115. Both rows verified in both directions.

At `8cdc40820` the report reads `5a: 135 rows, 115 not yet agree`, with

```
agree  loud=no  5a  xmeths ...  xmeths.rex
agree  loud=no  5a  unkno  ...  unkno.rex
```

read from the verdict cells, not inferred. Under the D50 control -- `unknown_or_nomethod`'s resolve
forced to miss, so a miss goes straight to NOMETHOD -- both cells become `diverge-both` and the
summary becomes `5a: 135 rows, 117 not yet agree`. So the control reddens **the row the report claims
plus the one it also claims**, and reproduces the pre-task figure. The loud inventory no longer lists
`the UNKNOWN forward for message "ZORK"`, checked in the live report.

Under the same control the differential corpus reads `155 of 156` on exactly
`lang/message_send_unknown_forward.rex: stdout, stderr, exit code differ`, and
`condition_nomethod.rex` stays green -- correct, since deleting the `UNKNOWN` step changes nothing
for a receiver that answers none. Two further tests redden that the report's D50 section does not
list (the directory in-crate test, and `collect_stress`); the report does not claim that section is
exhaustive, so this is not a false statement, only a less complete accounting than its other two
control sections give.

I did not read Table C at `c18b877e3` directly -- doing so would have meant checking out another
revision in this worktree. The control's reproduction of `117` with both rows red, and the fact that
no other row moves, is what I am resting the delta on.

## Method notes

* Every crate/oracle comparison above is three descriptors read separately, never `2>&1`, oracle
  under `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 10 .../bin/rexx FILE )`,
  crate on both `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, run from a fresh `mktemp -d` with
  absolute paths. Nothing from `corpus/oracle-crashes.txt` was run.
* Both new corpus programs were also run directly through that harness: both **MATCH** on all three
  descriptors on both engines.
* Every mutation was applied to a file copied to the scratchpad first, restored with `cp` and
  confirmed with `diff -q` plus `git status --short`; the release binary was rebuilt from the
  restored tree afterwards. The tree is clean and `dispatch::tests` is green at the end of this
  review.
* Two counts in this review were taken with `/bin/grep -a` or `awk -F'\t'` because the ugrep wrapper
  hides them: the stale sentence in `phase-5a.txt` hard-wraps mid-phrase, and `class-methods.txt` and
  the baseline are tab-separated.
