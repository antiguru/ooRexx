# Phase 5b review fix round -- report

**BASE:** `60256a8cc`, branch `plan/rust-rewrite`, worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`. Oracle checked before anything else:
`parse version` answers `REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`, so the
differential is version-matched.

Every oracle probe below ran from a fresh empty directory under
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 20 .../build/bin/rexx FILE )`,
with stdout, stderr and exit status captured to three separate files, never `2>&1`, and every crate
run on both `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`. `rust/corpus/oracle-crashes.txt` was
read first; no program in it and no shape from it was constructed. No `unsafe` was written.

Binary hashes are `sha256sum rust/target/release/rexx-run`:

| build | hash |
|---|---|
| BASE, unmodified | `d73757e856773f42444abb1dd97f562fb9136c31afcad9b9b2507ed27e57b158` |
| with the B1 fix (every figure below unless another hash is named) | `4b3a8542f828da867e6b212c40373162627fc8cafc31fcc6db00c901ff7f832f` |
| A3's mutation (both root pushes deleted) | `4e1995c5acee840f485f486aabd55c343f8b37abbc2b5de1961c3062b6c3cc58` |
| A2's `multidim-transpose-offset` | `45421184904bb2b33eb04a0b9c39d1a8f443cbff17f6087c8593115fdd7ff688` |
| A2's naive `.rev()`, in a `git archive 60256a8cc` extract | `11ba60245b3147bd7ff35ae8defd4464dbaa27be11247e248728a06453e9b039` |
| A3's mutation, in that same extract | `3de868d5cabe16401fac9ba8461616c8a2290b9033a8f0e9cb4d26877d56539b` |

Both mutated files were restored from a `cp` copy taken before the edit, not from `git checkout --`,
and each restore was verified: `activation.rs` back to
`e0eee2738eee7010ae89d132e3a7e2bbbaacad9e5a2978d27d2e691ee3343e1a` and byte-identical to
`git show 60256a8cc:rust/crates/rexx-exec/src/activation.rs`; `dispatch.rs` back to
`d858f2b6446604e88afd65bc9616a8f5e82c6f234ea0032a6a8ee6f8fe1bf086`, its pre-mutation hash, and the
rebuilt binary back to `4b3a8542...`, which is the pre-mutation binary's hash.

---

## 1. B1 -- the four silent wrong answers. Fixed, with a corpus witness each

### The oracle's four answers, re-measured here before anything was built

Binary `d73757e8...` (BASE), both engines, three descriptors:

| program | oracle | crate at BASE |
|---|---|---|
| `o~sendWith('M', .array~new(2,2))` | rc 168, `Error 88.913:  Argument message arguments must be a single-dimensional array.` | rc 0, `a seen 0` / `r` |
| `o~startWith('M', .array~new(2,2))` | rc 158, `Error 98.913:  Unable to convert object "an Array" to a single-dimensional array value.` | rc 0, `a seen 0` / `r` |
| `forward message('N') arguments (.array~new(2,2))` | rc 158, `Error 98.946:  FORWARD arguments must be a single-dimensional array of values.` | rc 0, `a seen 0` / `r` |
| `self~run(source, 'A', .array~new(2,2))` | rc 168, `Error 88.913:  Argument argument array must be a single-dimensional array.` | rc 0, `a seen 0` / `r` |

The brief's table and strand B's transcripts reproduce exactly, on both engines.

### The C++, audited rather than taken on trust

`runtime/MethodArguments.hpp:675` and `:703` are the two `arrayArgument` overloads and both reject
`array->isMultiDimensional()` after `requestArray` (the raises are at `:687` and `:714`). The
brief's `:675` is right and its `:717` names the named overload's closing brace rather than its
raise; the reviewer's conclusion is unaffected. They differ in more than their number: the
**named** overload raises `Error_Invalid_argument_noarray` (88.913) substituting the argument's
name, the **positional** one raises `Error_Execution_noarray` (98.913) substituting the object's own
string value. The call sites are `arrayArgument(args, "message arguments")`
(`classes/ObjectClass.cpp:1980`), `arrayArgument(args, ARG_TWO)` (`:2051`) and
`arrayArgument(arguments[2], "argument array")` (`:2224`). `FORWARD ARGUMENTS` has its own copy of
the test and its own error, `instructions/ForwardInstruction.cpp:189`-`:191`.
`ArrayClass::isMultiDimensional` is `dimensions != OREF_NULL && dimensions->size() != 1`
(`classes/ArrayClass.hpp:312`), which is what `Interp::is_multi_dimensional_array` now answers.

### What landed

* `Interp::is_multi_dimensional_array` in `value.rs`, beside `array_body`.
* `dispatch.rs`'s `array_argument`, with an `ArrayArgument` enum naming which overload a site calls,
  used by `message_arguments` (`~sendWith`), `native_start_with` and `native_run`'s `'A'` arm.
* `forward_arguments_conversion`'s `Body::Array` arm answers `Conversion::Refused` for a
  multi-dimensional array, which is already routed to `Raised::forward_arguments` -- one condition
  and one raise, as the C++ has it.
* Two `Raised` constructors, appended at the end of `impl Raised` so no existing definition's line
  number moved for that reason.

### After the fix: all four agree, and so do their neighbours

Binary `4b3a8542...`, both engines, three descriptors, fresh empty directory per run: all four
programs above **AGREE**, and so do the single-dimensional neighbours --
`o~sendWith('M', a)` for a 4-slot array with slot 1 filled, `.array~new(0)`, a 2-slot array with
slot 2 filled, `o~startWith('M', .array~new(2))`, `forward ... arguments (.array~new(3))` and
`self~run(source, 'A', .array~new(3))`.

### The corpus witnesses

Four programs, one per member, each running its single-dimensional neighbour first and then ending
on the **untrapped** refusal, so the row carries the substitution as well as the number -- which a
trapped one cannot, because `CONDITION('A')` is not implemented (measured: rc 120,
`CONDITION option "A" answers an Array or .NIL, which is not implemented`).

```
corpus/lang/object_send_multidimensional.rex          AGREE rc=168  stdout "m 0"
corpus/lang/object_start_multidimensional.rex         AGREE rc=158  stdout "m 0"
corpus/lang/forward_arguments_multidimensional.rex    AGREE rc=158  stdout "m 0"
corpus/lang/object_run_multidimensional.rex           AGREE rc=168  stdout "ran 0"
```

Each is in `corpus/phase-5b.txt` and in `EXPECTED_SUBSET_5B` in the same commit, and each has a
`crates/rexx-parse/tests/sourceline_oracle/<name>.txt` generated by the driver that test's module
comment sanctions.

### `refusal-sites.tsv`

Two rows added, `argument_not_single_dimensional` (88.913) and `object_not_single_dimensional`
(98.913), both `send`/`agrees`/`reached=yes`. Both are live: replacing either row's `answer` with
`ZZQQ7731X` reddens `a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not`
and the failure names that row. The `definition` column moved for other rows as a consequence of the
edits, and every one of those was taken from the test's own derived list rather than computed by
hand.

`Raised::forward_arguments`'s doc said "`.nil` is the reachable half here"; both halves are
reachable now and it says so.

---
## 2. C1 -- `CLOSED_PHASES` now has a failing witness

`gate_table_c.rs` gains `every_closed_phase_this_table_owns_rows_for_is_gated`, built on the shape
`corpus.rs`'s `the_differential_reads_every_phase_subset_file` uses: the answer is derived from the
corpus directory rather than typed a second time. It asserts that every phase this table owns rows
for -- `CONCEPTS`' own assignments plus `WIRING_PHASE` and `METHOD_PHASE` -- for which
`corpus/phase-<id>.txt` exists, is in `CLOSED_PHASES`. A committed subset file is the project's own
record that the phase's programs agree with the oracle, so a phase that has one and owns rows here
owes those rows an exit status.

**Controls, both run**, `cargo test --release -p rexx-exec --test gate_table_c --no-fail-fast
every_closed_phase`, one test running in each and not zero:

```
# "5b" removed from CLOSED_PHASES
test every_closed_phase_this_table_owns_rows_for_is_gated ... FAILED
["5b"] own rows in gate table C and have a committed corpus subset file, so their programs agree
with the oracle, but CLOSED_PHASES does not name them -- ...
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out

# CLOSED_PHASES emptied
["5a", "5b"] own rows in gate table C and ...
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out
```

`gate_tables/mod.rs` was restored from a `cp` copy afterwards and its hash checked back against the
copy, with `git diff` on that path empty.

**The premise re-measured, not taken from strand C.** In a `git archive 60256a8cc` extract with its
own `CARGO_TARGET_DIR` -- so a tree that does not carry the new test -- with `"5b"` dropped from
`CLOSED_PHASES` **and** the `methodsbyclass` 5b row actually broken by the offset transposition:

```
REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test gate_table_c --no-fail-fast
  mode: STRICT (the gate) -- REXX_CORPUS_GATE is set, verdicts gated for 5a
  diverge-stdout loud=no  5b   methodsbyclass ...
  5a: 135 rows, 0 not yet `agree`     5b: 6 rows, 1 not yet `agree`
  gated by this run: 0 row(s)
  test result: ok. 14 passed; 0 failed        EXIT=0
```

A broken 5b row, and the gate is green. That is what the new test now makes impossible to reach by
editing `CLOSED_PHASES` alone.

The assertion is one-directional and says so in its own doc: it catches a phase being *dropped*, and
a phase added before it closes needs no assertion because it gates rows that do not agree and the
next gate run is an exit status.

Today the derivation yields exactly `{5a, 5b}`. Read from the two tables' own printed summaries
rather than from a grep: table C reports rows for 5a, 5b and 5c, table D for 5a, 5b, 5c and 7, and
`corpus/` carries `phase-4a/4b/4c/5a/5b.txt`, so 5c and 7 are filtered out by the subset-file test.

## 3. A2 -- the `methodsbyclass` control now names the spelling that works

The row's `control` field gains the sentence that makes it runnable. **Both halves measured here.**

*The form that works*, applied to `multi_dimension_position` -- keep the bounds pass pairing each
subscript with its own dimension, collect the positions, and accumulate the offset over
`positions.iter().zip(dimensions).rev()`:

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec \
    --test gate_table_c --no-fail-fast          (binary 45421184...)
  diverge-stdout loud=no  5b   methodsbyclass ...
      oracle rc=0  out="element z23 a12 a21\norder a21 a12 z23\ndimensions 2 2 3\n..."
      crate  rc=0  out="element z23 a12 a21\norder a12 a21 z23\ndimensions 2 2 3\n..."
  5a: 135 rows, 0 not yet `agree`     5b: 6 rows, 1 not yet `agree`
  test result: FAILED. 14 passed; 1 failed        EXIT=101
```

The `element` line is identical and only the `order` line moves, which is the row's own claim.

*The form that aborts*, `subscripts.iter().rev().zip(dimensions)`, measured on a `git archive
60256a8cc` extract with its own `CARGO_TARGET_DIR` (binary `11ba6024...`), `rexx-run` on
`methodsbyclass.rex` from a fresh empty directory:

```
rc 134, stdout empty, stderr:
  thread 'rexx-interp' (1625556) has overflowed its stack
  fatal runtime error: stack overflow, aborting
```

so strand A's figure reproduces on a build of mine. `dispatch.rs` in the working tree was restored
from a `cp` copy and re-verified by hash, and the extract was never the tree under gate.

## 4. A4 -- `instance_self_reassigned.rex`'s sentence made accurate

`phase-5b.txt` said the row is "the shape the receiver's rooting turns on". It is not, and I could
not make it so (see A3 below), so the sentence now says what the row does pin -- that a body may
assign over `SELF`, in `INIT` and in an ordinary send, and still reach its exposed pool afterwards --
and states the measurement that removes the rooting claim. The row is unchanged and still in the
subset.

## 5. B2 -- the false reason in `refusal-sites.tsv`'s header

"nothing in this crate can" is gone. The header now gives the real obstacle -- the `witness` column
is prose rather than a runnable program -- and notes in parentheses that integration-test binaries
under `crates/rexx-exec/tests/` do reach the oracle through `support::oracle`,
`licensed_divergences.rs` among them, so a reader deciding whether to close the gap in 5c is not
told the tooling does not exist.

## 6. C4 -- D61's stale clause

D61 forbade a `phase-5b.txt` program depending on the class-object termination order "because D60
has not characterised it yet". Task 5 characterised it, so D60's conditional lapsed and the clause
with it, while `corpus/lang/uninit_class_sweep_order.rex` -- a `phase-5b.txt` program whose whole
subject is that order -- was left contradicting the literal text.

Corrected in `docs/superpowers/specs/2026-08-27-phase-5b-instances.md`, in four places, all one
fact:

* **D60** now carries the characterisation instead of "characterising it is a task this phase's plan
  owes": the oracle's `uninitTable` is an `IdentityTable` walked bucket by bucket and a class
  object's bucket is the hash of its **id string** rather than of its address, which is why the order
  reproduces where an instance's does not. The citations are
  `ClassRegistry::take_uninit_classes_in_sweep_order`'s own doc, and
  `rexx-classes/tests/uninit_sweep_order.rs` asserts the transcripts.
* **D61** is now about instances alone, and carries a paragraph saying **what was removed and why**,
  that it is removed rather than narrowed because what replaces it is D60's characterisation and not
  a second rule, and what that means for 5c: a row there may depend on the class sweep's order and
  may not depend on the order over several instances.
* The prose passage above D60 that repeated the same conditional.
* The Risks table row that repeated it a third time, and the "What I could not check" entry that
  said the plan still owes the characterisation.

## 7. A1 -- the two undiscriminated pairs, recorded and then asserted

**Re-measured here, not taken from the review.** Transposing both the `answer` and the `witness` of
`argument_needs_a_string_value` with `named_argument_needs_a_string_value` (88.909), and of
`argument_not_a_class` with `scope_override_not_a_class` (88.914), leaves `refusal_sites.rs` at
`5 passed; 0 failed` in each case. Table-only edits; the file was restored from a `cp` copy between
them and its hash checked back.

I chose **record** rather than **fix**, and then made the record self-policing rather than prose.
The reason for not fixing: `admits` accepts an answer that is one of the constructor's own
`syntax(M, N)` numbers, so discriminating within a shared number means relating the `witness` column
to the constructor, and the `witness` column is prose. Changing what `admits` accepts touches every
send-surface row and the instrument strand A has just validated by mutation, which is a poor trade
in a fix round.

What landed: the file's header names both pairs explicitly, so a reader can check the claim rather
than trust it; and `the_answers_more_than_one_row_shares_are_the_recorded_ones` holds the set of
`answer` values carried by more than one send-surface row against a `SHARED_ANSWERS` constant, so
the header cannot go stale.

**Controls, both run.**

* Table side: planting `88.909` into `not_one_of`'s `answer` reddens the new test *and*
  `a_reached_row...`, `3 passed; 2 failed`.
* Source side, so the new test's own subject is shown load-bearing: dropping the `88.914` entry from
  `SHARED_ANSWERS` reddens the new test **alone**, `4 passed; 1 failed`.

Both files were restored from `cp` copies and hashed back.

**What this does not do:** it does not close A1. A transposition inside either pair is still
invisible. The gap is now named in the file it is in, and a third row joining either pair is red.

---

## 8. A3 -- the investigation. The pushes did not redden, and something beside them did

**The mutation, as strand A specified it.** `out.push(*receiver)` (`activation.rs:1351`) and
`out.push(*owner)` (`:1362`) deleted, the two neighbouring `out.push(*scope)` calls left alone. Each
`OLD` pattern was required to occur exactly once before the edit. Built to
`4e1995c5acee840f485f486aabd55c343f8b37abbc2b5de1961c3062b6c3cc58`.

**The controller's candidate does not redden.** Run on both engines under the mutation:

```
say .K~new~m   with ::method init exposing v = 'held' and ::method m
               forcing a collection and answering 'ok' v self~class~id
  ir / tree-walker  rc 0  "ok held K"      -- byte-identical to the unmutated answer
```

**Nor do the shapes below, built to defeat the "held somewhere else" explanation.** All are rc 0 and
byte-identical to the unmutated crate's answer under the mutation:

| shape | what it was meant to remove | answer, mutated and unmutated |
|---|---|---|
| `self = 'clobbered'` before the forced collection, exposed value 85 bytes | `SELF` as a second root | `ok 85 held-yzyzy clobbered` |
| the same inside a 400-iteration loop, forcing a collection each time and allocating between them | a freed slot never being reused | `ok 85 held-yzyzy 1092` |
| an `UNINIT` finalizer that forces a collection and reads its own exposed value | a receiver with no program-side holder at all | `fin 85 held-y`, then `after` |
| the same reached through `~sendWith` rather than a direct send | the send's own literal receiver | `ok 85 held-yzyzy` |
| the same reached through `~startWith` and read at `~result` | the caller's frame | `ok 85 held-yzyzy` |
| `self = 'clobbered'`, a `.D~new` dropped, then the collection | -- this is the control, below -- | `D collected` then `ok 85 held-yzyzy` |

**The control says the collection is real.** The last row's `D collected` is printed by a finalizer
that runs *inside* the method, under the mutation, after `SELF` has been assigned over. So a
collection ran with the activation live and the exposed pool was still read back intact: the null
result is a measurement and not an artefact of nothing being collected.

**Half of it is redundant by reading, and I state that as a reading rather than a run.**
`exec_expose` takes its `owner` from `Interp::pool_owner`, which answers either the receiver itself
(the `Body::Instance` arm, `run.rs:3140`) or a per-class variables object that
`roots.add_global(".class-variables ...")` roots permanently (`:3166`-`:3168`). In the first case
`out.push(*owner)` pushes what the line above it already pushed; in the second it pushes something a
global root holds. I did not delete the global root to prove it.

**Verdict.** It does not redden, under every shape I could construct. `out.push(*receiver)` and
`out.push(*owner)` stay: whether to remove them is the collector owner's call and not this round's,
and "no program I could write distinguishes them" is not "nothing can".

### 8a. What the investigation did find: a silent wrong answer at BASE

The shape that finally moved is `REPLY`, plus an assignment over `SELF`, plus a forced collection,
plus a **read of the exposed variable afterwards** -- and it moves at **`60256a8cc` with no mutation
at all**. The controller could not reproduce my first spelling and sent theirs; running both settled
it, and theirs is the shorter, so it is the one in the plan:

```rexx
o = .K~new
say 'late' o~m
::class K
::method init
  expose v
  v = 'held-yzyzy'
::method m
  expose v
  reply 85
  self = .K~new
  call gc 'force'
  say 'after-gc' v
```

```
oracle           rc 0   late 85 / after-gc held-yzyzy      stderr empty
ir, tree-walker  rc 0   late 85 / after-gc V               stderr empty
```

`after-gc V` is what an **unset** variable reads as, so the instance's pools are no longer reachable
from the replied body. rc 0 and empty stderr on both sides: a silent wrong answer.

**Why the controller's own run agreed, and it is the sharpest thing this section has.** Their
program ends `return v` where mine ends in a `SAY`. With `return v` all three sides **agree** -- rc
0, `late 85` on stdout, `Error 98.936:  RETURN cannot return a value after a REPLY.` on stderr --
because the raise fires *before* the value is read, so that spelling never reads `v` after the
collection at all. One line apart, and the difference decides whether the defect is visible. This is
exactly the "tries the obvious spelling, sees agreement, deletes the row" failure the plan's own
`witness` column exists to stop, which is why the program is now in the plan as a fenced block
rather than described.

**Every other part is load-bearing**, each dropped in turn from the controller's shape and
re-measured on three sides:

| program | verdict |
|---|---|
| `reply` + `self =` + `call gc 'force'` + a read | **DIVERGE** on stdout |
| the same with `return v` instead of the read | AGREE (98.936 fires first) |
| no `reply` | AGREE |
| no `self =` | AGREE |
| no `call gc 'force'` | AGREE |

The `SELF` assignment reproduces with a string and with another instance; my own first spelling used
a string and an 85-byte exposed value, and answered `late 1 V` against the oracle's
`late 85 held-yzyzy`.

**It is not A3's pushes.** Under the mutation my spelling answers identically, byte for byte, so
deleting them changes nothing here either. What a deferred reply's activation is reachable from is
the open question.

Beside it and separately: a `call gc 'force'` inside a replied body moves the interleaving -- the
oracle prints the replied body's line before the caller's next one and this crate after it -- where
the same program without the collection agrees on order.

**Recorded in the plan** (`docs/superpowers/plans/2026-08-27-phase-5b.md`, Task 9's open list, which
is where this project keeps open divergences) with the program verbatim and **Owner: whoever owns
the collector and `Interp::run_deferred_replies`**. Not fixed here: it is outside every item in my
brief, it is pre-existing rather than 5b's, and the fix is in the collector, which the brief
reserves.

**A3's own null result is recorded in the plan too**, at the controller's instruction, with the
shapes tried and the control written out so the next person adds a new one rather than repeating
these. The report is gitignored; the plan is not, and the finding is worth committing.

## 9. A fifth member of B1's mechanism, found while bounding the fix. Recorded, not fixed

Probing the neighbouring `Interp::array_slots_of` call sites for the same shape -- which B1's own
finding asks about ("nothing in the tree reddens if this spreads to a fifth caller") -- turned one
up:

```
raise syntax 40.4 additional (.array~new(2,2))
  oracle           rc 158  Error 98.939:  Additional information for SYNTAX errors must be a
                           single-dimensional array of values.
  ir, tree-walker  rc 216  Error 40.4:  Too many arguments in invocation of ; maximum expected is .
```

The C++ is the same `requestArray`-then-`isMultiDimensional` test at a different site,
`instructions/RaiseInstruction.cpp:286`-`:289`, with an error of its own rather than
`arrayArgument`'s. The crate reaches it through `exec_raise`'s own `array_slots_of`.

**Put to the controller rather than taken, and ruled a record.** It shares B1's four's C++ check --
`requestArray` and then `isMultiDimensional` -- at its own site, with its own number and its own
substitution, so it would need a witness of its own rather than riding theirs; and it is **loud on
both sides** rather than a silent wrong answer, which is the class B1's fix ruling was about. The
controller reproduced the oracle's 98.939 independently and ruled: record, do not widen a round
whose gates were already in flight. It is recorded beside B1's four in the plan's open list,
**Owner: 5c**.

The other neighbours are clean or unreachable: `~UNKNOWN`'s argument list and `.Message~new`'s both
refuse the multi-dimensional array on the oracle (98.913 and 88.913) and are rc 120 Phase 5 refusals
here, because `.Directory~new` and `.Message~new` are not built; `self~run(.array~new(2,2))` in the
`I` style agrees on all three descriptors.

## 10. Recorded with a named owner, not built

* **A5 -- five 5b behaviours no test distinguishes.** `native_set_method`'s option/restricted-check
  order (its `native_run` sibling *is* witnessed, so this is an asymmetry between siblings);
  `native_start_with`'s argument-position substitution; `native_copy`'s non-instance arm;
  `started_message`'s `validate_scope_override`; and `check_restricted_method`'s no-receiver arm.
  **Owner: 5c**, which inherits all five surfaces. The last one is recorded as **unwitnessed and not
  as unreachable**: the code's own comment predicts no caller can reach it, and that is the shape
  this project has been wrong about repeatedly. I did not try to build a program for any of the
  five.
* **B3 -- `refusal-sites.tsv`'s `verdict` column is held against nothing.** The phase measured the
  check's yield at one bad row in twelve (`message_array_shape`), strand B re-probed thirteen
  `agrees` rows by hand and all still agree, and nothing keeps them so. **Owner: 5c.** The two rows
  I added carry `agrees` verdicts I measured myself, which does not change the column's standing.
* **B4 -- `trace_oracle.rs`'s control arm still cannot fail**, now for a new reason: the only
  `Coverage::WitnessedLive` row is in a phase-4 subset, so dropping `phase-5b.txt` from the
  unguarded literal changes nothing that test looks at. **Owner: whichever phase first adds a live
  trace witness to its own subset file.**
* **C2 -- D59a is named nowhere under `rust/`.** Both consequences it assigns to 5c are unreachable
  today, so no ledger entry is possible yet; the exposure is that when 5c makes them reachable, the
  only thing pointing at the licence is a sentence in the previous phase's spec. **Owner: the 5c
  spec**, which is written against the parent spec and will not see it otherwise.
* **C3 is ruled not a defect** by the brief and I built nothing for it.

---

## 11. What I did not do

* **I did not close A1.** The check still cannot discriminate inside either shared-answer pair. What
  landed is the record the brief's second option asks for, plus an assertion so the record cannot
  rot. My reason for not fixing is in section 7.
* **I did not fix the fifth `arrayArgument`-shaped divergence** (`RAISE ... ADDITIONAL`), and I did
  not fix the `REPLY` + `SELF` + collection divergence. Both are recorded in the plan with owners;
  section 8a and section 9 give my reasons.
* **I did not remove `Activation`'s two root pushes**, and did not run the corpus differential or
  the workspace suite under A3's mutation -- strand A did both and reported `327 of 327` and an
  identical failing set, and re-running them would have measured strand A rather than the tree. What
  I added is seven shapes it could not have covered, a control that the collection really runs, and
  a reading of `pool_owner` that accounts for one of the two pushes.
* **I did not do a prose pass.** Sentences outside the items in my brief were left alone, including
  ones I read and found unlovely. The exceptions are the three passages that repeat D61's stale
  clause -- correcting one and leaving three copies would have been the correction-round failure
  this project measures -- and `Raised::forward_arguments`'s doc, which the fix itself falsified.
* **I did not re-measure the performance sitting**, Task 10's pin, or any figure outside the items
  listed.
* **I did not build a witness for C3**, which the brief rules not a defect, and I built nothing for
  A5, B3, B4 or C2.
* **I did not copy this round's records into `docs/superpowers/records/2026-08-27-phase-5b/`.**
  `.superpowers/` is gitignored, so this report, the three review reports, the review plan and my
  brief exist only in the untracked workspace. The phase's pattern is that the controller copies the
  whole set in one record commit, which is what `60256a8cc` did for the tasks.
* **No `unsafe`**, no file restored with `git checkout --`, no bare `git stash`, no `rm` with a star
  glob, no `git add -A`, no amend.
---

## 12. The gates

Run from `rust/` by a script that writes each status to its own file as it finishes, so a run does
not have to survive the turn that started it. Tree hash taken before the first and after the last,
over tracked files' bytes **and** untracked files' bytes:

```
{ git ls-files -z | xargs -0 sha256sum; git ls-files -o --exclude-standard -z | xargs -0 sha256sum; } \
  | sort | sha256sum
before  012d875eb2403b34989ffbee3d9888a25cde42a4ec3d72c9677cc1cf3fa74075
after   012d875eb2403b34989ffbee3d9888a25cde42a4ec3d72c9677cc1cf3fa74075
```

They match, so the tree the gates ran on is the tree committed. `rust/Cargo.lock` is
`0c2fc3f3c5ae44cf450fc6eccd0dee99dc9087381efd706cbdb54d444f756ea2` before and after, and
`git diff -- rust/Cargo.lock` is empty.

| gate | command | status |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets --offline -- -D warnings` | **0** |
| 3 | `cargo test --release --offline --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --offline --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --offline --workspace --no-fail-fast` | **0** |
| 5b verdicts | `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release --offline -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |

Read out of the logs rather than off the exit status alone: **zero** `test result: FAILED` lines in
any of them, 105 `test result: ok` blocks in each of gates 3, 4 and 5, `331 of 331 matching` in gate
4 (327 at BASE plus this round's four rows), and `5b: 6 rows, 0 not yet agree` in table C with
`5b: 2 rows, 0 not yet agree` in table D. No cargo process was left running.

### The clippy figure, and the control that makes it evidence

`rust/CLAUDE.md`'s rule is that a same-session clippy green is provisional and the lint must be run
from a **clean target directory**, because the recorded incident had two candidate causes -- cargo
reusing a per-crate result, and a toolchain moving mid-session -- and only one of them is a
fingerprint problem. `touch` would have addressed that one; a fresh target directory addresses both,
and it does not cost the warm build the other gates are using:

```
CARGO_TARGET_DIR=<fresh scratchpad dir> cargo clippy --workspace --all-targets --offline -- -D warnings
  exit 0, 67 crates checked        cargo 1.98.0 (797e8a9bc)   clippy 0.1.98 (88d9e12ae1)
```

**A clean-target green says the lint ran. It does not say the lint looked at my new code**, so the
positive control is separate: a copy of this tree (excluding `rust/target`, `.git`, `oodocs` and
`ootest`), its own fresh target directory, and two violations planted **inside my own new
`array_argument`**:

```
baseline in the copy                                             exit 0
+ `let clippy_control_unused = slots.len();` and a needless `return`
  error: unused variable: `clippy_control_unused`
     --> crates/rexx-exec/src/dispatch.rs:6723:9
  error: unneeded `return` statement
     --> crates/rexx-exec/src/dispatch.rs:6724:5
  error: could not compile `rexx-exec` (lib) due to 2 previous errors
                                                                 exit 101
```

One rustc lint and one clippy lint, both naming the function this round added. The working tree was
never touched: `git status --porcelain` stayed at 19 lines throughout and the tree hash above is
unchanged.

Two earlier attempts at this control are recorded because they do **not** work and the next person
should not repeat them: `cargo clippy --workspace ... -- -D clippy::pedantic` and the same scoped
`-p rexx-exec` both die at `rexx-extract` with 80 errors before reaching `rexx-exec`, because cargo
passes `--` arguments to path dependencies too. Planting the violation is the only form of this
control that reaches the crate under test.

---

## 13. The commit

`9ec812de9d6c5b052e854be08b60211374f2c203`, read back with `git log -1 --format=%H` after
committing rather than written from memory. 19 files, 521 insertions, 76 deletions; every path named
explicitly on `git add`, no `git add -A`, no amend, `git commit -F`. `Cargo.lock` is absent from the
staged set and from the commit, checked both ways. `git status --porcelain` is empty afterwards.
