# Task 12 report: `UNKNOWN`, and the NOMETHOD condition

Status: **DONE_WITH_CONCERNS**. The concerns are listed at the end; none of them is an
unfinished part of this task.

Commits, oldest first:

| SHA | what |
|---|---|
| `ee6ebbf64f47d84f72172e7f8de4f007f1723e39` | the `UNKNOWN` forward and the NOMETHOD condition |
| `d68f2b5441b1ffe9daf6ee27b0ab328be223e391` | that commit's two sittings |
| `0ce35233e856e74831d02ae8dc19b3da03998f9b` | fix: NOMETHOD is offered to the raising activation and no further |
| `527445adf2b3d0b78f18a1d4a47143b84696d592` | the fix round's sitting |
| `8cdc40820031f63fedf5a352920873f10a1a69a7` | module comment naming the search order's tail |
| `977cf4b9e925687b834b4c9059bc6595e65b27ae` | fix round 1, whose own section is at the end of this file |

The oracle behind every measurement below answers `parse version`
`REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`, checked at the end of the task through the
required wrapper. Every crate run is both engines, `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`
(the spelling `tw` is rejected at rc 2, which cost one probe round). Every probe ran from a fresh
empty directory with absolute paths, three descriptors read separately.

## What the brief asked to be told

The brief said: "if you find any sentence claiming the crate answers 97.1 for a receiver that HAS
an `UNKNOWN` method, that sentence is stale and I want to know." **There is one, and it is in the
binding spec rather than the plan or the brief.**

`docs/superpowers/specs/2026-08-17-phase-5-object-model.md:25`, inside the transcript block that
opens the document's own justification:

```
say .k~zork(1,2)     with  ::METHOD unknown CLASS returning "unknown:" n "args" a~items
  oracle  rc 0    stdout  unknown: ZORK args 2      stderr empty
  crate   rc 159  stdout  empty                     stderr Error 97.1 ... does not understand "ZORK"
```

The `crate` line is stale twice over: it became rc 120 with `Loud::unknown_forward` at `f4b21eadb`,
and it is rc 0 matching the oracle from `ee6ebbf64`. The block is introduced as "Measured this
session", so it reads as a dated measurement rather than a standing claim -- but the prose two lines
above it ("Both are, today, silent wrong answers at rc 0 or a wrong error rather than a loud
refusal") is present tense and rests on it. **I did not edit the spec**; it is the binding document
and the correction is the controller's. Found by:

```
/bin/grep -an "97.1" docs/superpowers/plans/2026-08-17-phase-5a.md \
    docs/superpowers/specs/2026-08-17-phase-5-object-model.md \
    .superpowers/sdd/2026-08-17-phase-5a/task-12-brief.md
```

The plan's own rows at `:1263` and the brief's at `:10` are the amended text, which states the rc 159
figure as what the paragraph *used to say* and then corrects it, so neither is stale. Every other row
the grep returns is about a different mechanism -- `~method`'s own-dictionary rule, `::CONSTANT`, a
donated name -- or about `message_send_unknown_method.rex`'s receiver, which answers no `UNKNOWN`.

**Two such sentences existed in the crate and both are gone**: the `None if
self.answers_unknown(receiver)` arm's comment in `dispatch.rs`, which said this crate "implements no
such forward", and `Loud::unknown_forward`'s doc in `lib.rs`. Neither claimed 97.1 *was* the answer;
both stated the refusal as the current state, which this task ends.

## The before state, re-measured

Measured at `c18b877e3` (the revision this task started from), release build, both engines:

| program | oracle | crate before |
|---|---|---|
| `say .k~zork(1,2)`, `::method unknown class` returning `'unknown:' n` | rc 0, `unknown: ZORK` | rc 120, `rexx-exec: the UNKNOWN forward for message "ZORK" is not implemented (Phase 5)` |
| `.k~zork(1,2)` as a clause, the method saying `'unknown:' n 'args' a~items` | rc 0, `unknown: ZORK args 2` | rc 120, same refusal |
| `say .k~zork` (no arguments) | rc 0, `unknown: ZORK args 0` | rc 120, same refusal |
| `signal on nomethod` over `say 'abc'~nosuchmsg` | rc 0, trapped | rc 159, the 97.1 report |
| `say 'abc'~nosuchmsg` | rc 159, 97.1 | rc 159, 97.1 -- agreed already |

So the task had two halves rather than one: a loud refusal to convert into the answer, and a
condition the crate raised as `SYNTAX` where the oracle raises `NOMETHOD` first.

## What was built

### The `UNKNOWN` step

`Interp::send_message`'s two `None` arms collapsed into one call to a new
`Interp::unknown_or_nomethod` (`rust/crates/rexx-exec/src/dispatch.rs`). It resolves `UNKNOWN` on
the receiver by the ordinary lookup with no start scope -- `RexxObject::processUnknown`
(`classes/ObjectClass.cpp:1002`), reached from `messageSend` at `:904`, whose own lookup is
`behaviour->methodLookup(GlobalNames::UNKNOWN)` at `:1005` -- and invokes it with two arguments, the
missed name and an `Array` of the send's own arguments (`:1013`, then `:1018`-`:1019`). Every C++
line cited was re-read at the path given before it was written down.

`Loud::unknown_forward` and `Interp::answers_unknown` are deleted rather than left unused. The
`answers_unknown` predicate asked the same question the resolve now answers, so keeping it would
have been a second table to hold in step.

**The array is the argument list as the send holds it.** Measured on the oracle, one `UNKNOWN`
printing `n`, `a~items` and `a~size`:

```
.k~zork(1,2)    n=ZORK  items=2  size=2
.k~zork(1,,3)   n=ZORK  items=2  size=3
.k~zork(1,)     n=ZORK  items=1  size=1
.k~zork()       n=ZORK  items=0  size=0
```

so an interior omission is an empty slot and a trailing one in a call's argument list is no slot at
all, and `a~class~id` is `Array`. The crate answers those four rows byte for byte on both engines.

### The NOMETHOD condition

`Raised::nomethod` (`rust/crates/rexx-exec/src/error.rs`) is `Raised::no_method`'s 97.1 with the
condition named `NOMETHOD`, `rc` left alone and `description` set to the message name. The
`code_sub` a trapping handler reads through `CONDITION('E')` needs nothing new: `offer_to_trap`
already fills it only for `SYNTAX`.

Measured, `say 'abc'~nosuchmsg` under the two traps -- oracle first, then the crate, identical:

```
                C           D            E     I       RC
signal on nomethod  NOMETHOD    NOSUCHMSG    ''    SIGNAL  untouched
signal on syntax    SYNTAX      ''           1     SIGNAL  97
```

`CONDITION('A')` is the one option the crate cannot answer here. The oracle reports the receiver
(`abc`); this crate refuses `CONDITION('A')` loudly for every condition, which predates this task
(`crates/rexx-exec/src/builtin/state.rs:481`, unmodified) and is the reason the corpus program below
asks `C`, `D`, `E`, `I`, `S` and `RC` and not `A`.

`CALL ON NOMETHOD` needed no work: `rexx-parse` already refuses it. It refuses with this crate's
generic parse refusal (`rexx-exec: 25.1: Invalid subkeyword found.`, rc 120) where the oracle reports
25.1 at rc 231 -- **and that is not this task's**, verified by `call on syntax` producing the
byte-identical refusal. No corpus program was added for it.

### The scope of the NOMETHOD offer -- and the defect I shipped and then fixed

`ee6ebbf64` asked **every live activation** whether it had a NOMETHOD trap, on the reading that
`reportNomethod` calls `raiseCondition`, which walks. Three probes supported that reading and a
fourth killed it.

The fourth: `signal on nomethod` in the main body, and the miss inside a `::METHOD` body.

```rexx
signal on nomethod name nm
say .k~m
say 'not reached'
exit 0
nm:
say 'nm' condition('C') condition('D')
exit 0
::class k
::method m class
  return 'inner:' .plain~zork
::class plain
```

Oracle: **rc 159**, the 97.1 report, `nm` never entered. `ee6ebbf64`: **rc 0**, `nm NOMETHOD ZORK`,
both engines. A wrong answer, and one the corpus as committed at `ee6ebbf64` did not see -- the
mutation that produced the correct behaviour left the corpus at 156 of 156, which is how I found it.

`0ce35233e` replaces the walk with `Interp::trap_for` -- the running activation alone -- because
that table is already the union it needs to be: an internal `CALL` inherits its caller's traps by
copy (`Activation::traps`' own doc carries the three probes for that), and a `::METHOD` activation
inherits none (`Activation::method` sets `TrapMap::default()`). The three observable rows:

| shape | which handler runs, oracle and crate |
|---|---|
| caller arms NOMETHOD, callee (internal `CALL`) arms SYNTAX, miss in the callee | the caller's NOMETHOD |
| nothing arms NOMETHOD, the same callee arms SYNTAX | that callee's SYNTAX, `RC` 97 |
| caller arms **both**, miss inside a `::METHOD` body | SYNTAX, `RC` 97 |

The third row is the discriminator and is now `condition_nomethod.rex`'s `methodmiss`. With the
stack walk restored on top of the fixed tree, `REXX_CORPUS_GATE=1 cargo test --release --test
corpus` reads **155 of 156** with `lang/condition_nomethod.rex: stdout differ`, and nothing else.

`trap_for` also had to stop panicking where nothing is running: `dispatch.rs`'s own `send_message`
tests call the send with no activation on the stack, and the walk tolerated that where `trap_for`'s
`self.activation()` did not. It now answers `None` there. The two tests that caught this are
`dispatch::tests::a_class_identity_used_as_a_receiver_answers_from_the_class_side` and
`dispatch::tests::an_unimplemented_method_is_loud_where_an_unknown_one_is_a_condition`; both
panicked at `activation.rs:1214` before the widening.

`Interp::trapped_anywhere` is deleted, not left unused.

### Where the choice between the two conditions is made

At the raise, in `Interp::nomethod`, not in `offer_to_trap`. `offer_to_trap` sees one activation at
a time as a failure unwinds, and a NOMETHOD the raising activation declines has to become the 97.1
**there**, in time for that same activation's own `SYNTAX` trap. Measured: a routine whose only trap
is its own `SIGNAL ON SYNTAX` does take its own missed send, at rc 0 with `RC` 97, on the oracle and
here. A build that named the condition NOMETHOD unconditionally and degraded it later would answer
that row with the fatal 97.1 report.

## What changed that is not an answer

`.environment~nosuch` and `.local~nosuch` were `rexx-exec: the UNKNOWN forward for message "NOSUCH"
is not implemented (Phase 5)` and are now `rexx-exec: method "UNKNOWN" of class "Directory" is not
implemented (Phase 5)`. Both rc 120; the oracle answers `The NIL object` at rc 0. **Not a
regression**: the refusal moved from the search step, which now exists, to the thing that is still
missing, which is `Directory`'s own `UNKNOWN` body -- Task 17's. `corpus/docs/class-methods.txt`
carries `Directory unknown instance`, so the name is in the registry with no native behind it and
`Interp::invocable` names it.

The brief said the corpus should tell me if this changed. It cannot, and I checked rather than
assumed:

```
/bin/grep -an "environment~\|local~\|\.environment\b\|\.local\b" corpus/lang/*.rex | /bin/grep -a "~[a-zA-Z]"
```

Every directory send in the corpus names `~at`, `~put`, `~class`, `~isA` or `~hasMethod` -- all
answered -- so none reaches the forward. **The instrument for those bytes is an in-crate test only**,
`dispatch::tests::a_directory_forwards_a_missing_name_to_an_unknown_this_phase_lacks`, and it is the
only assertion on that string in the workspace:

```
/bin/grep -rn "UNKNOWN\\\\\" of class\|of class \\\\\"Directory" --include=*.rs crates/
```

returns one line, that test.

`corpus/lang/message_send_unknown_method.rex` is untouched and still agrees: rc 159, `97.1 Object
"abc" does not understand message "NOSUCHMSG".`, both engines, and it is one of the two corpus
programs the D50 control leaves green.

## Corpus programs added

Both in `corpus/phase-5a.txt`, `coverage.rs`'s `EXPECTED_SUBSET_5A`, and with a committed
`sourceline_oracle/<name>.txt` regenerated by the driver in `sourceline_oracle.rs`'s module comment.
Neither is in `collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS`, so both run under
collect-on-every-allocation.

* **`corpus/lang/message_send_unknown_forward.rex`** -- rc 0. The forward's two arguments, the four
  omission shapes, inheritance of a superclass's `UNKNOWN`, a name the subclass does answer not
  reaching it, `~~` yielding the target, the message-assignment form arriving as `ZORK=` with the
  assigned value as the array's one item, and one row whose message name is longer than a handle
  holds.
* **`corpus/lang/condition_nomethod.rex`** -- rc 0. Seven routines: trapped by name, trapped by
  `ANY`, degraded to SYNTAX, the caller-beats-callee pair, `CALL ON ANY` declining, and the
  method-body row.

Both are byte-identical to the oracle on all three descriptors under both engines, checked
program-by-program before the gate run as well as by it.

## Controls run

### D50's control: delete the `UNKNOWN` step

Applied at the final HEAD by forcing `unknown_or_nomethod`'s resolve to miss, so a miss goes
straight to NOMETHOD -- the mutation `xmeths`' own `control` field names.

* **The `unkno` concept row reddens**: `agree loud=no` becomes `diverge-both loud=no`. So does
  **`xmeths`**, whose probe has an `UNKNOWN` line of its own. Gate table C's 5a count moves from
  **115 not yet `agree`** to **117**.
* The corpus reddens too, at exactly one program: `155 of 156`, `lang/message_send_unknown_forward.rex:
  stdout, stderr, exit code differ`. `condition_nomethod.rex` stays green, correctly -- deleting the
  `UNKNOWN` step changes nothing for a receiver that answers none.
* Reverted with `git checkout --`, and the tree re-verified clean and green afterwards.

The row I claim reddens is `unkno`, and I read its verdict cell in the report before and after
rather than inferring it from the count.

### Never raise the NOMETHOD condition

`Interp::nomethod`'s gate forced to `false`, so every miss is the plain `SYNTAX` 97.1. Reddens
`corpus_differential` at `lang/condition_nomethod.rex` and
`the_l0_subset_passes_again_under_collect_on_every_allocation`, and nothing else in the gated release
suite. **This is what says the second corpus program adds coverage** rather than only being able to
fail.

### The whole-stack walk

Described above. Reddens `lang/condition_nomethod.rex` alone, on the `methodmiss` row.

### Rooting, three sites, and only one of the three is witnessed

Under `collect_stress.rs`'s collect-on-every-allocation:

| deleted root | witnessed |
|---|---|
| `push_temp(arguments)` -- the arguments array | **yes**, by `message_send_unknown_forward.rex`'s long-name row: the stress run exits 120 with `rexx-exec: a message send to a value whose object is no longer live is not implemented (Phase 5)` where the plain run prints the row |
| `push_temp(missed)` -- the message-name value | no; nothing in the workspace reddens |
| a loop rooting each of `args` | no; nothing reddens |

The first was inert until the long-name row existed: with a name of seven bytes or fewer,
`Interp::text` never allocates, so nothing collects between building the array and handing it over,
and the deleted root leaves the case unwitnessed. That is why the row is in the program and why the
comment beside the root says so.

The arg loop is **deleted** on the strength of its control being inert: the one production caller
(`Interp::message_term`) roots every argument value as it evaluates it
(`Interp::eval_traced_argument`), which is the same rooting every other allocation reached from a
send already relies on, so the loop was duplicating a root rather than adding one.

`push_temp(missed)` is **kept** despite its control being inert, and the comment says why in those
terms: it is the only root that value has -- the slice handed to `invoke` is not a root and nothing
else holds the name -- and its control being inert says `invoke` reaches its argument binding
without allocating, not that the value would survive if it did.

## Gates

All five, from `rust/`, at `8cdc40820`:

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --release --workspace` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 0 |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 |

`memcap` is present in this session (`/home/moritz/.local/bin/memcap`), checked rather than assumed.
The differential corpus is **156 of 156 matching** under both gate commands, up from 154 by the two
programs added. `--no-fail-fast` was used for every mutation run, and the mutation counts above are
the whole failing set rather than the first catcher.

Gate table C: `5a: 135 rows, 115 not yet agree`, against **117** at `c18b877e3`. The two rows that
moved are `unkno` and `xmeths`, both now `agree loud=no`, read from the report's own verdict cells.
Gate table D reads `36 rows, 7 not yet agree`; **I did not capture its pre-task figure**, so I am
reporting the number rather than claiming it did not move. Gate table C's loud inventory no longer
lists `the UNKNOWN forward for message "ZORK": 2`, which the pre-task capture of that report does.

## Performance

**Pin staleness, run before trusting it.** `bench-baselines/pinned/rexx-run-15a1ffa98` has sha256
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching
`bench-baselines/PINNED.md`. Every commit `git log --oneline 15a1ffa98..HEAD -- rust/crates
rust/Cargo.toml rust/Cargo.lock Cargo.toml` lists is named in this plan's ledger; I checked each
abbreviated hash against `progress.md` and the plan file in a loop rather than by eye, and the loop
printed nothing.

The sitting question -- can this change affect what executes -- is plainly yes, so sittings were
run. **Three of them**, all appended to `bench-baselines/phase-5a-arms.tsv`:

| rows | `task` | builds | what |
|---|---|---|---|
| 5338-5649 | `12` | `pinned`, `head` | the guard's two-build shape at `ee6ebbf64` |
| 5650-6141 | `12-prev` | `pinned`, `prev-c18b877e3`, `head` | the same, plus the task's starting revision |
| 6142-6633 | `12-fixround-1` | `pinned`, `prev-c18b877e3`, `head` | the same builds at `0ce35233e` |

**The `task` labels were made distinct in fix round 1**; as first committed, the first and second
sittings both read `task = 12` against `commit = ee6ebbf64` and were told apart only by file order.
The second exists because the budget question -- how much of the **1.6949** instructions per pass
left on `strings`/`ir` this task spends -- is a difference between head and the task's starting
revision, and a per-pass figure is comparable within a sitting and not across two. The third exists
because `0ce35233e` changes `src/`.

Third sitting, `instructions:u`, per pass:

| axis | arm | pinned | prev-c18b877e3 | head |
|---|---|---|---|---|
| `strings` | ir | 5369.5252 | 5421.5253 | 5421.5253 |
| `strings` | tw | 9044.5187 | 9129.5189 | 9129.5186 |
| `alloc4c` | ir | 3594.6860 | 3603.6865 | 3603.6847 |
| `arith` | ir | 24059.5746 | 24062.5719 | 24062.5725 |
| `compound` | ir | 1909.4801 | 1914.4801 | 1914.4801 |
| `emptyloop` | ir | 376.0000 | 376.0001 | 376.0000 |
| `varlookup` | ir | 871.0000 | 871.0000 | 871.0000 |

head over `prev-c18b877e3` in `instructions:u` is **1.000000 to six decimal places on every axis,
both arms, both sizes** -- computed inside each sitting from its own `absolute` rows. The widest
departure from 1 is `2.4e-7`, at `alloc4c/ir/small` reading `1.000000238` in the `12-fixround-1`
sitting; in the `12-prev` sitting it is `2.6e-7`, at `alloc4c/ir/large` reading `0.999999741`. So
`strings`/`ir` sits at
**+52.0001** against the 53.695178 ceiling, **1.6951 instructions per pass left**, and this task
spends none of it. That is what the shape predicts: `unknown_or_nomethod` is `#[cold]
#[inline(never)]` and no benchmark axis sends a message that misses, so the only change on an
executed path is `send_message`'s two match arms becoming one call.

No axis moved 1% in `instructions:u` against the pin either, so nothing here needs the
change-versus-layout attribution the guard asks for above that threshold. `cycles:u` moved by up to
8% on `compound/tw` across builds and is not reported as a result, per the guard's own rule; the
figures are in the file.

## What the checks could not see

* **The corpus cannot see a refusal the oracle does not share.** The `.environment~nosuch` message
  change is covered by one in-crate assertion and nothing else; had the message been wrong, both
  gate commands would still have read 156 of 156.
* **Gate table C's `unkno` row is one program.** Anything the `UNKNOWN` section claims that
  `unkno.rex` does not exercise is outside the row, which is what its `control` field is for.
* **`CONDITION('A')` for a trapped NOMETHOD is unmeasured against the oracle beyond the one probe**,
  because the crate refuses the option. If a later task implements it, the oracle answer for this
  condition is the receiver's string value (`abc` for `'abc'~nosuchmsg`).
* **`::ROUTINE` bodies were not probed for trap inheritance.** The `::METHOD` row above is what
  pins the non-inheritance case; whether a `::ROUTINE` activation behaves the same way was not run,
  and no code here branches on it.
* **The rooting controls that came back inert prove nothing about a future `invoke`.** They say what
  allocates today on the paths the subset covers, and the comment states it that way.

## Concerns

1. **I shipped a wrong answer in `ee6ebbf64` and fixed it in `0ce35233e`.** The reading that
   produced it -- `reportNomethod` calls `raiseCondition`, so the offer walks the stack -- was drawn
   from the C++ and confirmed by three probes; the fourth probe, which only exists because a
   mutation of the correct code left the corpus green, killed it. The mechanism sentence in
   `ee6ebbf64`'s doc comment was wrong in exactly the way naming a mechanism from a citation rather
   than from a run tends to be. The shipped tree carries the observable rule and cites
   `Activation::traps` for why the running activation's table is the right one to ask.
2. **`.environment~nosuch`'s refusal text changed and only an in-crate test guards it.** If Task 17
   is expected to find that string where it was, it will not.
3. **The parse refusal for `CALL ON NOMETHOD` is 25.1 loud at rc 120 against the oracle's 25.1
   report at rc 231.** Generic to this crate's parse errors, unchanged by this task, and no corpus
   program covers it. Flagging it because the brief's "trapped and untrapped" wording could be read
   as including the illegal-trap spelling.
4. ~~**Two sittings share `task = 12` in the TSV.**~~ **Closed in fix round 1**: the second sitting
   is relabelled `12-prev`, the duplicate-key count is `0`, and `bench-baselines/README.md` now
   carries the check and the label-versus-measurement rule that permits the relabel.
5. **The plan's Global Constraints say the corpus is "106 of 106".** It is 156 of 156 and was 154
   before this task. The sentence is prose about a mutable aggregate; the constraint it carries
   ("stays there and grows") is met.
6. **The spec's opening transcript now understates this crate on both of its rows.** The `UNKNOWN`
   row is the one the brief asked about, above. The `makeString` row beneath it (`say .k` answering
   `The K class` where the oracle answers `K says hello`) is Task 14's and I did not re-measure it,
   so I do not know whether it still holds -- I am flagging the block, not the second row.

---

# Fix round 1

Reviewer verdicts were spec compliance PASS and task quality PASS with required corrections. Nothing
about the behaviour changed in this round: no executed line moved, and the only source edits are
comments plus a test-side comment. **The sitting question, answered in a sentence as the controller
directed: this round changes comments, two corpus programs, a corpus manifest, a baseline label and a
README, all of which the guard's own list names as cases that cannot affect what executes, so no
sitting was run.** No comment fix moved a line of code, so there is nothing here I think could
matter.

Commit: `977cf4b9e` (all four items in one commit, because the TSV relabel and the README amendment
that permits it must land together).

## 1. `corpus/phase-5a.txt` carried the withdrawn mechanism (blocker)

**What it said:** "The offer walks the whole activation stack before it degrades, so a caller's
NOMETHOD handler beats a callee's SYNTAX handler -- the pair of routines that show both outcomes for
the same callee is what a build degrading too early answers identically."

The observable half was true; the mechanism was the one `0ce35233e` withdrew, and that commit never
touched this file. Reproduced the reviewer's finding before fixing: `git log --oneline -S"whole
activation stack" 8cdc40820 -- rust/corpus/phase-5a.txt` names `ee6ebbf64` alone (at HEAD it now names
`977cf4b9e` too, the removal), and `git show --stat 0ce35233e` does not list the file.

**What it says now:** whose trap table decides is the running activation's; an internal `CALL`
inherits its caller's traps by copy so a caller's NOMETHOD handler does beat a callee's SYNTAX
handler, while a `::METHOD` activation inherits none and the same main-body handler is never reached;
a build asking the whole stack answers the method-body row with the NOMETHOD handler. That is the rule
`Interp::nomethod`'s doc and the program's own header already state.

## 2. `dispatch.rs`'s rooting comment named the wrong symptom (blocker)

**What it said:** "Measured with `collect_stress.rs`'s collect-on-every-allocation and this root
removed: that row panics in `to_text` and a name of seven bytes or fewer does not."

**Reproduced both halves myself** rather than taking the finding on trust. With
`self.roots.push_temp(arguments);` removed, `cargo test --release -p rexx-exec --test collect_stress`
reddens `the_l0_subset_passes_again_under_collect_on_every_allocation` on
`lang/message_send_unknown_forward.rex` with

```
plain   exit=0   ... "unknown: AVERYLONGMISSINGMESSAGENAME items 2\n"  stderr=""
stress  exit=120 ... (that line absent)  stderr="rexx-exec: a message send to a value whose object is no longer live is not implemented (Phase 5)\n"
```

Nothing panics. The site is `Interp::receiver_kind`'s dead-handle arm, `dispatch.rs:699`, reached by
the row's own `a~items` sending to the collected array; the refusal is `Loud::receiver_class`. With
the same root removed and the long name replaced by `zorkx`, the stress subset passes (6 of 6), so the
long-name row is still what witnesses this root. Both files were restored from scratchpad copies and
`git status --short` confirmed clean before the real edits.

**What it says now:** the row's own `a~items` sends to a collected array, `receiver_kind` refuses it,
the stress run exits 120 with that message where the plain run prints the row, and a message name of
seven bytes or fewer leaves the subset green because `Interp::text` inlines it. The same correction
went into the report's rooting table and into
`corpus/lang/message_send_unknown_forward.rex`'s own header, which carried the same wrong symptom.

## 3. Set sizes in new comments

The reviewer's list names `dispatch.rs:1101`, `:1106` and `:1134`, `error.rs:1453` and
`corpus/phase-5a.txt:354`. I re-read every comment line this task added rather than only those
(`git diff c18b877e3 -- crates/ corpus/ | /bin/grep "^+"`, filtered for numerals and for
`both`/`pair`), which turned up more. The table is the whole set, and every row of it is fixed:

| where | was | now |
|---|---|---|
| `dispatch.rs` `unknown_or_nomethod` doc | "The last two steps of the documented search order" | "What the documented search order has left after the class chain" |
| same doc | "The forward's two arguments are..." | "The forward's arguments are the missed message name and then an `Array`..." |
| same doc | "answers `0` for both" | "answers `0` for each" |
| same doc | "on both of `messageSend`'s paths" | "wherever `messageSend` reaches it" |
| `dispatch.rs` rooting comment | "Both of the forward's arguments are rooted, and only one of the two roots has a witness" | "Each of the forward's arguments is rooted, and only the array's root has a witness" |
| `dispatch.rs` `nomethod` doc | "which of two conditions it is" / "the two are separate answers" / "where the two readings part" | "which condition that is" / "they are separate answers" / "where the readings part" |
| `error.rs` `Raised::nomethod` doc | "differing from it in the three things a trapping handler reads back" | "differing from it in what a trapping handler reads back -- the table below is that difference" |
| `coverage.rs` subset comment | "which are the search order's last two steps" | "which are what the documented search order has after the class chain" |
| `corpus/phase-5a.txt` | "One class returns from UNKNOWN and one only says" | "Class J's UNKNOWN only says where the others return" |
| `corpus/lang/message_send_unknown_forward.rex` | "so the two arms are written differently" | "so the arms are written differently" |
| same file | "the first of the forward's two arguments allocates and the second one has to survive it" | "building the name argument allocates and the array argument, built before it, has to survive that" |
| `corpus/lang/condition_nomethod.rex` | "which is what the first three routines are for" | "which is what byname, byany and bysyntax are for" |
| same file | "the last three routines are the three answers that follow from it" | "outerwins, innersyntax and methodmiss are the answers that follow from it" |
| same file | "methodmiss arms both and gets the SYNTAX one" | "methodmiss arms NOMETHOD and SYNTAX alike and gets the SYNTAX handler" |

The `phase-5a.txt` row was also **imprecise**, which is the reviewer's point about it: the program
declares an `UNKNOWN` on `K`, on `J` and on `BASE`, so "one class returns and one only says" was
wrong as well as a count. Naming the set instead makes it checkable -- `J`'s says, the others return.

**What I did not change**, and why: `//! \`exec_call\`'s relation to the call pair exactly` in
`dispatch.rs`'s module doc. That clause is pre-existing prose; my edit appended a sentence to the same
source line, which is why it shows as an added line in a diff. The reviewer recorded that the
untouched prose around it uses the same shape ("`resolve` and `invoke` are two steps") and left the
general question to the controller, so widening the sweep into text this task did not write is not
mine to do. Also kept: `both` where the sentence has already named the things individually ("`signal
on nomethod` and `signal on syntax` both armed"), and every measured number ("`~items` is `2` and
`~size` is `3`", "seven bytes", "the array's one item" -- that last one is what `~items` answers for
the message-assignment form's array, not a count of a set the comment is enumerating).

## 4. Performance prose

**What it said:** "head over `prev-c18b877e3` in `instructions:u` is **1.000000 to seven decimal
places on every axis, both arms, both sizes** -- computed inside the sitting from its own `absolute`
rows, the widest departure being `1.00000024` on `alloc4c/ir/small` in the second sitting."

Two defects, both confirmed by recomputing from the TSV rather than from the review:

* **Seven decimal places overstates it.** At seven places `1.000000238` is `1.0000002` and
  `0.999999741` is `0.9999997`. Six is the honest figure, and it is still four orders of magnitude
  below the guard's 1% threshold.
* **`1.00000024` is the `12-fixround-1` sitting's, not the `12-prev` sitting's.**

Recomputed per sitting over every `(axis, arm, size)` from that sitting's own `absolute`
`instructions:u` rows:

| sitting | widest departure from 1 | where |
|---|---|---|
| `12-prev` | `2.6e-7` (`0.999999741`) | `alloc4c/ir/large` |
| `12-fixround-1` | `2.4e-7` (`1.000000238`) | `alloc4c/ir/small` |

The review names `1.000000032` on `varlookup/ir/large` as `12-prev`'s widest; that is the largest
ratio **above** 1 in that sitting, which I confirmed, and it is a different quantity from the largest
departure in either direction. I have stated which quantity mine is rather than picking between them.
Everything load-bearing is unchanged: `strings`/`ir` at `+52.0001` against the `53.695178` ceiling,
`1.6951` per pass left, no axis within 1% of the pin.

## 5. The TSV keys

**Before:** 312 rows carried a `(task, commit, axis, build, scope, arm, size, instrument)` key that
another row also carried with a different value -- Task 12's first two sittings, both `task = 12`
against `commit = ee6ebbf64`. Measured with the review's own `awk`, and 312 is exactly the first
sitting's row count, so every row of it collided and no other task's rows are involved.

**After:** the second sitting's `task` column reads `12-prev`. The duplicate-key count is `0`.

**No measurement was touched**, proved rather than asserted: `paste` of `cut -f2-` over the before and
after files reports columns 2 through 12 identical on every line, and the two files have the same line
count. Only 492 lines differ, and only in column 1.

`bench-baselines/README.md` said the opposite of what this does -- "**The `commit` column is the
reliable discriminator either way**", and "the rows are not rewritten \[to a spelling decided
afterwards]: they are measurements, and editing them to a spelling decided afterwards would be editing
data" -- so it is amended in the same commit, as the controller directed:

* the false discriminator sentence is replaced by the property that actually holds, `(task, commit)`,
  **stated as a runnable check** (the duplicate-key `awk`, which prints `0`) rather than as a claim;
* the label-versus-measurement split is written down: a measured value is never rewritten, a `task`
  label may be corrected, and one task taking two sittings of one revision is the case that arises;
* the suffix convention now names the shape rather than one instance of it, and cites `11-bisection`
  beside `12-prev`;
* **the dated enumeration went too.** It read "Run 2026-08-22, that is `6`, `7`, `8` and `9` bare ...
  plus `9-fixround-1`, `10` and `10-fixround-1`", where the labels its own command returns are now
  `10`, `10-fixround-1`, `11`, `11-bisection`, `11-fixround-2`, `12`, `12-fixround-1`, `12-prev`,
  `6`, `7`, `8`, `9` and `9-fixround-1`. Rows were appended under labels it does not list on the
  same date it names, so a transcription is what went wrong rather than the date. The command stays
  and the transcription is gone, which is the only form of that section that cannot rot. The column
  table's cross-reference to "either of the two spellings below" went with it.

## What this round did not close

* **`::ROUTINE` trap inheritance.** The report's "what the checks could not see" entry says this task
  did not probe it. The review did, and reports a `::ROUTINE` body behaving as a `::METHOD` body
  does -- rc 0 with the SYNTAX handler. I have left my entry as written, because it describes what
  **this task** ran; the review's probe is the record of the answer.
* **Finding 6, the rejected design named in a doc comment.** The reviewer left it to the controller
  and I was not asked to act on it, so `Interp::nomethod`'s "where a version asking the whole stack
  runs the `NOMETHOD` one" and the corpus program's equivalent both stand.
* **Concerns 1, 2, 3, 5 and 6** of the original report are unchanged and still stand.

## Gates, at `977cf4b9e`

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --release --workspace --no-fail-fast` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | 0 |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 |

Differential corpus **156 of 156**. Gate table C `5a: 135 rows, 115 not yet agree`, gate table D
`36 rows, 7 not yet agree` -- both unmoved from `8cdc40820`, which is what a comment-only round should
produce.

Both corpus programs changed length, so both `sourceline_oracle` expectations were regenerated with
the driver in `sourceline_oracle.rs`'s module comment and both programs were re-run through the
required oracle wrapper from a fresh empty directory: **MATCH on all three descriptors on both
engines**, oracle rc 0 for each.
