# Task 9 final fix round report

Status: DONE_WITH_CONCERNS. BASE `81bfd20c7`. Findings: `review-final8/task-9-review.md`; rulings:
progress.md "Ruling (final fix round)".

## I5 rxapi on the oracle side

Commit `c46e1cdf6`. Cause: the per-test loop and the `-S` listing run (which runs the whole group,
`worker.rex:563`) ran `FUNCTION.TEST_REXXQUEUE` on the oracle; `RECORDED` only renamed it out of
the whole-group run. Fix: `reaching_rxapi()` derives from each group file, and the files it
`::REQUIRES` or `loadPackage`s from its own directory, every test whose method body uses the data
queue or the function and macro-space registries (directly or through a `::METHOD`/`::ROUTINE`
of those files that does); `RECORDED` became `(name, runs)`, and the tests reaching rxapi must be
exactly the not-run entries, checked by its own test and as the gate test's first line, before
any oracle run. The listing run renames them out too.

**Beyond the fix, reported to the lead (not fixed here):** on the oracle every
`::REQUIRES` calls `RexxQueryMacro` (`interpreter/package/PackageManager.cpp:747`), as does every
external call (`RexxActivation.cpp:2996`), and that enters `LocalAPIManager::getInstance` ->
`establishServerConnection`, which starts rxapi when it cannot connect
(`rexxapi/client/LocalAPIManager.cpp:56-74,216-243`). `ooTest.frm:75-76` requires two files, so
no oracle ooTest run leaves rxapi untouched; the `rxfuncquery` removal never achieved that. The
corpus and several oracle tests also run PUSH/QUEUE/PULL/QUEUED on the oracle (session queue in
rxapi, `Activity.cpp:3282,3361`); the files, from a text grep (comments not excluded):
`corpus/lang/{pull_queue,push_queue,state_builtins,input_redirection,required_string_contexts,
required_string_default_name}.rex`, `corpus/gate-tables/*/*rexxqueue*`, `input_oracle.rs`,
`prompt_before_read.rs`, `state_builtin_oracle.rs`, `ir_recorded_cases/return-and-queue`.

Witness: `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test api_group_tests` at
`c46e1cdf6`'s tree: 23 passed, exit 0; the gate test asserts no listed name is a not-run one.
`the_detector_tells_a_queue_use_from_a_name_that_only_looks_like_one` pins the forms, including the
group's own `queue = ...`, `queue~queue(...)` and `queue~queued` non-uses.

Controls (predictions in `scratchpad/surface-9b/predictions.md`, written first):

| control | predicted | result |
|---|---|---|
| NC-I5a `TEST_REXXQUEUE` flagged as run | rxapi test red, gate test red at its first line before any run | both red, `left: {"FUNCTION.TEST_REXXQUEUE"} right: {}`, 0.01s |
| NC-I5b `.rexxqueue` form dropped | derived set empty, rxapi test red; detector red | both red as predicted |

### After the ruling (`eda65472c`)

The lead ruled that the constraint covers the daemon's persistent state only: named queues,
function registration and the macro space. The detector no longer counts the session queue.
The check's doc, the exclusions row and phase-8-gate.md now cite the read-only macro-space path:
- `PackageManager.cpp:747` and `RexxActivation.cpp:2996`, the two callers;
- `MacroSpaceApi.cpp:217-230`, the query;
- `LocalAPIManager.cpp:56-74` and `:177-243`, which start the daemon.

The gate record also names the corpus and test files that use the session queue, with the
command that lists them. The control that drops the `.rexxqueue` form went red again on the
narrowed detector.

## I1 crate PASS asserted; rexxc test recorded (Phase 9)

Commit `ccda0c619`. Cause: the instrument compared the two sides only; the rexxc test fails
identically on both (rc 127, no `rexxc` on `PATH`). Fix: the whole-group runs must also pass on
this crate (summary `pass`, exit 0), and each recorded test that runs must still not pass alone.
The pass sits in the whole-group run because the first witness run (per-test pass asserted)
was red on FUNCTION io tests (`TEST_BUFFERED_INPUT` ... `TEST_WRITE_BUFFER`) that fail alone
on both sides and pass after `TEST`, as the module doc already said. Recorded with owner Phase 9,
row 9 quoted; row 8 and section 10's sentence rewritten.

Witness: gate-mode `api_group_tests` 23 passed, exit 0.

| control | predicted | result |
|---|---|---|
| NC-I1 rexxc test removed from `RECORDED` | (revised before running, after the witness move) red only through the whole-group part, METHOD failing on both sides, stdout agreeing | red: `METHOD: oracle Some(1) failure, assertions 1128, ours Some(1) failure, assertions 1128; stdout agrees` |

Unwitnessed branch: "recorded but passing" (a recorded run test that starts passing) has no
control; making one needs a record line for a passing test.

## I2 native table calls use each class's own put/at/remove

Commit `84c138852`. Cause: `Surface::store_put` ran `hash::directory_put` (`insert`, replace) on
every class `hash::owns` admits. In the oracle only `DirectoryClass` overrides `put`/`get`/
`remove` (`grep` of `*.cpp` for `::put(RexxInternalObject`: `DirectoryClass.cpp:356` and
`HashCollection.cpp:423` alone); `HashCollection::put` calls `contents->put`, and a Relation's and
a Bag's contents are `MultiValueContents` (`RelationClass.cpp:131`, `BagClass.cpp:103`), whose
`put` is `addFront` (`HashContents.hpp:453-467`). `get` and `remove` are the contents' first
match, which the existing `merged_get`/`take_merged` already are for a store without a method
table. Fix: `hash::native_put` adds (`insert_front`) on a multi-value collection and otherwise
`insert`s, which still runs `DirectoryClass::put`'s method-table removal.

Witness `corpus/lang/library_collection_members.rex` (new; `.env`, `phase-8.txt` entry,
sourceline expectation generated from the oracle): at HEAD `out differs` on the Relation and Bag
lines only; after, identical rc 0; corpus gate `654 of 654 matching`. The reviewer's `c_*` and
`q1`-`q5` probes and `library_directory_members.rex` identical, `c_itable` differs as before
(DEVIATION 4).

| control | predicted | result |
|---|---|---|
| NC-I2a multi-value branch removed | coll differs on Relation/Bag lines only; library_directory_members identical | as predicted |
| NC-I2b add unconditional | coll differs on Set/Table/Properties/StringTable/Directory put lines; library_directory_members differs | as predicted (`put over method plain 2 1`) |

## I3 closed_phases blind spots

Commit `c4c76fc7f` (and `b1f488e7d`, clippy's `manual_strip` on I5's code). Cause: the scan
matched only the exact token `"Phase N"` per non-comment line, and read no exclusions file. Fix:
`literals()` lexes each source file (line and block comments, char literals incl. `'"'` and
`'\''`, lifetimes, raw strings, `\`-continued literals) and flags any literal naming a closed
phase as a word (`Phase 8` is not in `Phase 80`); `open_owners()` flags an `OWNER` sentence (not
`NO OWNER`, not `OWNERSHIP`) naming a closed phase unless the rest of its paragraph or the next
paragraph, before the next `OWNER`, says DELIVERED, CLOSED, RE-HOMED, FIXED or RESOLVED. Phase 7
is in `CLOSED`, so both apply to it. Unit tests pin the lexer and the row reader; the open-phase
control now also requires a longer literal naming Phase 9 in `rexx-api`.

Controls, each applied alone in the tree and restored from a copy (predictions first):

| control | predicted | result |
|---|---|---|
| NC-a `  OWNER: Phase 8` appended to exclusions | exclusions test red | red, `"OWNER: Phase 8"` |
| NC-b `ffi.rs:3468` `(Phase 9)`->`(Phase 8)` | `no_refusal_names_a_closed_phase` red at `src/ffi.rs:3468`, lexer test red | both red, `"src/ffi.rs:3468: Phase 8"` |
| NC-c `REFUSING_MEMBERS` gains a Phase 8 row | red at `src/layout.rs:<line>` | red, `src/layout.rs:379` |
| NC-d `alarm_startTimer` -> `"Phase 8"` | closed_phases red at `native.rs:116`; lib test red | both red |
| NC-e class-set.txt Alarm 6->8 | gate-mode `concept_and_class_gate_table` red; closed_phases green | red, "gated by this run: 7 row(s)"; closed_phases green |
| NC-f introspection-arity.tsv:196 `(Phase 9)`->`(Phase 8)` | `the_table_matches_the_three_sides` red; closed_phases green | as predicted |
| NC-g the Phase 7 row's `DELIVERED` removed | exclusions test red naming the Phase 7 row | **first run green, against the prediction**: the window then ran to the next `OWNER` and caught an unrelated DELIVERED 50 lines on. Narrowed to paragraph + next; re-run red, `"OWNER: Phase 7, the platform layer"`; NC-a re-run red |

## I4 rcond POSITION; outer9

### rcond: fixed, `2f7d9bd60`

Cause: `raise_held_condition` built the `CALL ON` object after `pop_native_frame`, so its frames
began at the caller and `POSITION` was the caller's line. The oracle builds it at the raise
(`Activity::createConditionObject`, `Activity.cpp:722-749`). At that point the native frame
leads, and `generateProgramInformation` (`:1080-1113`) takes the package from the first frame
that has one. That is the native frame, whose line is `.nil`, so there is no `POSITION`. The
`SIGNAL ON` path already agreed. Fix:
- `call_trapped_native_condition` builds the object before the frame comes off, whenever a
  `CALL ON` trap will take a non-`SYNTAX` condition.
- `NativeFrame::packaged` records whether the code reports a package.
- `build_condition_object_from` gives no `POSITION` when the leading native frame is packaged.
  The `GetConditionInfo` object shares that rule.

The Phase 9 record is rewritten as FIXED.

Witness: `corpus/lang/library_raise_condition_call_on.rex` (new). It raises from the main
program, an internal routine and a method, and prints `STACKFRAMES`, `TRACEBACK` and every entry
except the two lists. Before the fix (`rc2.rex` at HEAD) it differed on the `POSITION` lines, the
frame counts and the native `tb`/`sf` lines. After the fix it is identical, as are the reviewer's
`rcond.rex` and `rcond_rexx.rex`. Corpus gate: `655 of 655 matching`. `api_group_tests`: 23 passed.
rexx-exec lib: 856 passed.

| control | predicted | result |
|---|---|---|
| NC-I4a old after-pop build | differs as at HEAD: POSITION lines, frame counts, native lines | as predicted |
| NC-I4b POSITION rule off | differs on the POSITION lines only | as predicted |

### outer9: fixed for bound variables; the residual is loud and owned by Phase 6

Commits: `c32f4ce21`, `5acd03895` (the controller's; it committed my in-progress `surface.rs` and
does not build on its own) and `8b45fb06e`.

Ruling (a), with conditions, and how each was met:
1. **Guard type.** `Surface::in_caller` runs the member inside a `CallerSwap`, whose `Drop` swaps
   the caller back, an unwind included.
2. **GC witness.** `rexx-exec/tests/outer_context.rs` (gate-only, Linux) compiles the forge
   `outer9b.cpp` against `api/` and asserts it NEEDs no `librexx`. It runs `o9b.rex` against
   the oracle both plainly and collecting at every allocation, asserting `collections > 0`.
   `o9b.rex` compares RUN's context object before and after the members.
3. **Controls, predictions first:**
   - No swap: red.
   - No swap-back: red, a panic in `grow_slots`.
   - The displaced activation hidden from the collector: **green on the first run, against the
     prediction**. Every value lives in a `RootSet` slot, which the swap does not move. Once
     `o9b` compared the context object, the one object an activation owns outright, it went red
     (`a message send to a value whose object is no longer live`).
4. **Residual in the witness.** A corpus program cannot load a forge, so the witness is
   `outer_context.rs`, which also runs `o9c.rex` and asserts the rc 120 refusal naming Phase 6.

`o9b.rex` covers get, set and drop of bound simple, stem and compound-tail variables, plus
GetAll.

Cause: the outer `Activation`'s conversion is borrowed by the nested send, so
`ffi.rs::activation_of` fell back to the innermost activation, and the member ran against the
method's variables. Fix: each `Activation` names its host native frame. `variables_of` carries
that frame when the context's own activation is busy. While the frame's caller is swapped in,
slots are looked up with `bound_slot_of` and never grown.

Residual: a stem or compound read, or a set, of a name the outer activation never bound. It
refuses loudly, OWNER: Phase 6. The contiguous slot Vec grows only on the top frame
(`roots.rs:414-422`), and reaching a non-top frame's layout is D3 frame-ownership work.

Miri Stacked Borrows at `8b45fb06e`: 55 passed, 0 failed, 8 ignored, exit 0. The nested path is
exercised by `outer_context.rs`, not by Miri.

## M1 stash refusal cites the API contract

`c0a12eed8`: the code comment at the `Refused::StaleHandle` arm and the exclusions row now cite
`NativeActivation::createLocalReference` ("protects the object from GC until the environment
terminates", `NativeActivation.cpp:1179-1185`). The row says that the ordering licence is not
the reason. The refusal text is unchanged, so `refusal-sites.tsv` does not move.

## M2 queued

`.superpowers/sdd/queued/2026-09-29-release-local-reference-memory.md`, **uncommitted, because
`.superpowers/` is in `.gitignore:30`** and no queued file is tracked. It holds the forge source,
its build line and my re-measurement at `c0a12eed8`:
- oracle: 20296 KB.
- crate, 4 GB cap: 224648 KB at one million passes, 429752 KB at two million.
- crate, the oracle's 1 GB cap: aborts at two million passes (`memory allocation of 55 bytes
  failed`).

It also quotes the review's figures.

## M3 StringTableRemove witness

In I2's witness (`84c138852`): a plain hit, a subclass whose `REMOVE` override is bypassed, a
Properties hit, a Relation removing its front entry, and a miss raising 88.900 (identical).

## M5 -fsyntax-only witness

`4d10cbf77`, phase-8-gate.md section 9. Every `testbinaries/*.cpp` source compiled with `g++
-fsyntax-only -Iapi -Iapi/platform/unix -Itestbinaries`, and `orxclassic1.c` with `gcc`: each
`ok`. Both `diff -rq` commands print nothing. NC-M5 (renaming `RaiseCondition` in a scratch copy
of `api/`) was predicted to fail `orxmethod.cpp` and `orxinstance.cpp` only, and did.

## Gates

`scratchpad/surface-9b/gates.sh` (the surface-6 script with `S` set and a `G3 Compiling lines`
count added), run after touching every `rust/crates/*/src/*.rs`. The final run was at
`8b45fb06e`:

    G1 fmt exit 0
    G2 clippy(empty target) exit 0
    G3 release build (test --no-run) exit 0
    G3 Compiling lines: 11
    G4 release test exit 0
    G5 debug build (test --no-run) exit 0
    G6 debug test exit 0
    finished 2026-09-29T02:13:19+02:00

G4 2779/0/4 and G6 2780/0/4, summed from each log's `test result` lines. The corpus is `655 of
655 matching`, 0 rows were gated, and `a_kept_outer_context_reaches_its_callers_bound_variables`
is ok in both logs. HEAD did not move and `git status` was empty between the hashes. An earlier
run at `c32f4ce21` (`gates-run1/`) was also green, with 2758/0/4 and 2759/0/4. Section 10 was
rewritten from the final run in `eca6c0396`, docs only.

## Commits

    c46e1cdf6 I5  Run no test that reaches rxapi's queue or registries on either side
    ccda0c619 I1  Assert the Phase 8 groups pass on this crate, and record the rexxc test
    84c138852 I2  Add under an existing index on a Relation or Bag through the native put (+M3)
    b1f488e7d     clippy manual_strip on I5's code
    c4c76fc7f I3  Find a closed phase inside a refusal text and in open exclusions rows
    2f7d9bd60 I4  Build a native RaiseCondition's CALL ON object at the raise
    c0a12eed8 M1  Cite the API contract for the stashed local handle's refusal
    4d10cbf77 M5  Witness testbinaries against today's api/ with -fsyntax-only
    c32f4ce21 I4  Reach a kept outer call context's own caller from its variable members
    94897d79e     Rewrite phase-8-gate.md's close from the gate run at c32f4ce21
    eda65472c I5  Narrow the rxapi exclusion to the daemon's persistent state (ruling)
    5acd03895     (controller) the guard half of the outer9 swap; does not build alone
    8b45fb06e I4  Undo the outer-context swap by a guard, and witness it against the oracle
    eca6c0396     Rewrite phase-8-gate.md's close from the gate run at 8b45fb06e

## Concerns

1. **Edit collision.** The controller's `5acd03895` committed my half-finished `surface.rs`, so
   that commit does not build on its own. Its "OWNER: Phase 6" exclusions edit went into my
   `eda65472c`.
2. **M2's queued file is untracked**, because `.superpowers/` is gitignored.
3. **Two checks rest on thinner evidence than the rest:**
   - The "recorded but passing" branch of `api_group_tests` has no control.
   - Miri does not reach the nested outer-context path.
