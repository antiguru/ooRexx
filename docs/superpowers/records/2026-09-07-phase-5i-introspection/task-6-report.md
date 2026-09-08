# Task 6 — `RexxContext` and `StackFrame`

BASE: **`d8b013026`**, the HEAD found on release. `git status --porcelain` was empty at the moment
work started. (At dispatch it was `a43d780bc`; the tree was not free — Task 5 still had three files
modified and a `cargo` binary with its cwd in the main worktree — and Task 5's fix `d8b013026`
landed before I touched anything.)

Twenty-four rows: fourteen `RexxContext` readers, ten `StackFrame` ones, plus `RexxContext~package`,
which was already green and was **wrong** (below).

## Pre-flight: what the brief got wrong

Four findings, all measured before any code was written, all now rulings in the brief.

**`~invocation` is not a depth count.** `RexxActivation::getIdntfr`
(`execution/RexxActivation.cpp:94`) is `if (idntfr == 0) idntfr = ++counter;` off a process-global
atomic, so the ids follow the order they were first **asked** for. The brief's `1 2 3 4` is an
artifact of `createStackFrame` asking innermost-first. Measured, oracle rc 0, a program reading
`.context~invocation` at the top level before calling:

```
top-invocation= 1
1 ROUTINE INNER1 16 2 The NIL object
2 INTERNALCALL OUTER 11 3 The NIL object
3 PROGRAM <path> 7 1 The NIL object        <- the PROGRAM frame is 1, not 3
```

**`~type` has six values and `INTERPRET` is reachable on the oracle.** `StackFrameClass.cpp:52`-`:57`
is `COMPILE ROUTINE METHOD INTERNALCALL INTERPRET PROGRAM`. Declared as a divergence, not closed —
see *Divergences*.

**The brief's dead-frame transcript does not prove what it was offered for.** `f~arguments~items ->
0` came from a routine that took no arguments. Re-measured: a frame captured inside
`snapshot('p1','p2')` and read after the return answers `items` **2**. What a dead frame actually
retains, measured on the oracle at rc 0 — `~name`, `~line`, `~traceLine`, `~type`, `~target`,
`~invocation`, `~context` and a full `~arguments`; that is *everything* a `StackFrame` answers. The
snapshot-versus-live-handle split is right and the `98.981` half of that transcript stands.

**`~variables` in a method holds `SELF` and `SUPER`**, measured `OV,LOC,SELF,SUPER`; and a stem is
one entry under its trailing-period name with no entry for a compound.

## The C++ this task mirrors

| method | C++ | what it reads here |
| --- | --- | --- |
| `~args` | `RexxContext::getArgs` | `Activation::call_arguments` |
| `~condition` | `RexxContext::getCondition` | `Activation::condition` |
| `~digits`/`~form`/`~fuzz` | `getDigits`/`getForm`/`getFuzz` | `Activation::settings`, **in force** |
| `~executable` | `getExecutable` | `.ROUTINES`' entry, `Interp::method_objects`, or `Interp::program_routine_objects` |
| `~interpreter` | `getInterpreter` | a constant — see below |
| `~invocation` | `getInvocation` -> `getIdntfr` | `Activation::invocation`, minted on first ask |
| `~line` | `getLine` -> `getContextLine` | `ClauseSnapshot::line` |
| `~name` | `getName` -> `getCallname` | `Activation::invoked_as` |
| `~package` | `getPackage` | the **receiver's** activation's program |
| `~rs` | `getContextReturnStatus` | the unset branch — see below |
| `~stackFrames` | `getStackFrames` -> `Activity::generateStackFrames` | `Interp::frames` |
| `~thread` | `getThread` | a constant — see below |
| `~variables` | `getAllLocalVariables` | the activation's plan names, `extra` and frame |
| `StackFrame`'s ten | `StackFrameClass`'s accessors | the frame's own `NativeObject` entries |

**The three readers that answer from a constant, and what state each stands for.**
`~thread` is `Activity::getIdntfr` (`concurrency/Activity.cpp:108`), which mints one id per *system
thread* off a counter of its own; this crate runs every activation on one interpreter thread, so
there is exactly one thread to have an id and `1` is its. `~interpreter` is the same one level up,
per `InterpreterInstance`, and a `rexx-run` process creates one. `~rs` is
`settings.returnStatus`, which only a **command clause** writes — D12/Phase 7's, refused loudly
here, the same position `.RS` itself is in (`Interp::rexx_variable`). So `~rs` is the unset branch
rather than a stand-in: nothing this crate runs can take the other one.

**`~condition` answers `.nil` and refuses the other arm.** Building the condition object is
`CONDITION('O')`'s job and that is not implemented; answering `.nil` inside a handler would be a
wrong answer where the oracle hands back a `Directory`. Measured, oracle rc 0 in a `SIGNAL ON
SYNTAX` handler: `.context~condition~class~id` is `Directory` where this crate is rc 120,
`CONDITION option "O" answers a Directory, which is not implemented`.

## The state three rows needed and the crate did not have

`~line`, `~traceLine` and `~arguments` are about an activation that is **not running**, and all
three quantities lived on `Interp`, saved and restored in Rust locals, so no outer frame could read
them. `Activation::pc` cannot stand in for the clause: `run_bounded_instructions` steps a nested
construct's clauses on a **local** program counter, so a call made from inside a `DO` leaves `pc` on
the `DO`.

* **`ClauseSnapshot`** (`activation.rs`) — `{line, indent, index}`, written **once per call** by
  `Interp::push_activation` from the clause state in force as the activation is suspended.
  `Interp::clause_of` answers depth 0 from `Interp::clause_state` instead, so nothing is written per
  clause except one `usize`: `ClauseState::current_clause_index`, guarded by `clause_line_override`
  so an `INTERPRET` fragment leaves the enclosing clause's index in force.
* **`Activation::call_name` / `call_arguments`** — refcount clones of the calling convention, taken
  in `Interp::run_activation`, the one point every path that starts a body passes with both the
  activation and its convention in place (`invoke_call_over` pushes *before* it replaces the
  convention; `enter_method_body` replaces *before* it pushes, so neither site can take it itself).
  A `Entry::Method` activation stores no name: `method_identity` already carries it.
* **`Activation::invocation`** — `Option<u32>` off `Interp::next_invocation`, minted on first ask.

`Activation::object_roots` names `call_arguments`, and its exhaustive destructuring is what forced
each new field to be decided rather than silently unrooted.

## Performance

`instructions:u`, interleaved, five rounds, medians, the BASE binary and the HEAD binary as separate
files (`sha256` recorded in the scratch directory):

| axis | BASE | HEAD | ratio |
| --- | --- | --- | --- |
| `emptyloop` | 9,497,131,868 | 9,547,148,495 | +0.527% |
| `varlookup` | 16,785,147,812 | 16,861,167,592 | +0.453% |
| `compound` | 11,025,543,041 | 11,045,557,831 | +0.182% |
| `strings` | 22,188,711,095 | 22,218,722,947 | +0.135% |
| `dispatch` | 30,019,536,119 | 30,369,547,733 | **+1.166%** |
| `dispatchclass` | 24,903,940,855 | 25,175,951,588 | **+1.092%** |

The two axes the ruling named are under 1%. The two over it are the **message-send** axes, which the
ruling did not name and which I added because this change also touches the call path.

**`dispatch` first read +6.579%, and three of those points are two allocation traps.** Both were
found by isolation builds rather than by reading the diff:

* `Rc<[T]>::from(&[])` **allocates a header even for a zero-length slice**, so the empty default on
  every `Activation` was a `malloc`/`free` pair per activation. `Option<Rc<...>>` fixed it:
  6.579% -> 3.481%.
* The same trap one level up: `dispatch.rex` sends a **no-argument** message, and `args.to_vec()` on
  an empty slice allocates nothing where `Rc::from(args)` allocates. `Interp::shared_arguments`
  hands out one shared empty list: 3.481% -> 1.732%.
* `CallContext.name` then went back to an owned `Vec`, with a method activation's name read from the
  `method_identity` it already carries, so the hot path builds no `Rc<[u8]>`: 1.732% -> 1.166%.

Two isolation builds bound where the residue is **not**: removing the clause snapshot from
`push_activation` is worth 0.08pp and removing the per-activation convention snapshot entirely is
worth 0.38pp.

`samples/rexxcps.rex` was **not** measured: it is time-boxed rather than fixed-work, so its
instruction count is not a quantity an A/B can compare. `dispatch` and `dispatchclass` were measured
in its place and are the more relevant axes for this change.

## Divergences

All three at **rc 0 on both sides** — this phase's own pattern.

**`INTERPRET` frames are missing.** A fragment runs inside the enclosing activation here and pushes
none, so `stackFrames` taken inside an `INTERPRET` answers one frame fewer. Measured:

```
oracle:  1 [INTERPRET][4][     4 *-*   f = .context~stackFrames;]
         2 [INTERNALCALL][4][     4 *-*   interpret "f = .context~stackFrames; ..."]
         3 [PROGRAM][1][     1 *-* call outer]
ours:    1 [INTERNALCALL][4][     4 *-*   interpret "f = .context~stackFrames; ..."]
         2 [PROGRAM][1][     1 *-* call outer]
```

Every frame this crate *does* produce is byte-identical to the oracle's corresponding one. Closing
it means pushing an activation for `INTERPRET`, which changes `run_fragment`'s semantics; declared
per the controller's ruling, not closed.

**`COMPILE` frames are unreachable**, for a different reason: it is a parse-time frame
(`LanguageParser::createStackFrame`, `parser/LanguageParser.cpp:866`).

**A trapped condition consumes an invocation id on the oracle and none here**, because building the
condition object builds stack frames and `CONDITION('O')` is not implemented. Measured with two
programs differing only by a trapped `SIGNAL ON SYNTAX`: the routine's own id reads `2` without it
and `3` with. This is why `rexx_context.rex` traps nothing above the line that reads `~invocation`,
and its header says so.

**Declined, with their owners.** `StackFrame~executable` (`Setup.cpp:1707`) and `RexxContext~copy`
(`:1217`, `Error_Unsupported_copy_method`) are rows in neither table; both are left loud, per the
controller's ruling, so the next reader knows they were seen.

All of these are rows 6 and 8-11 of
`docs/superpowers/records/2026-09-07-phase-5i-introspection/found-not-fixed-register.md`, appended
rather than restructured.

## `RexxContext~package` was green and wrong

`native_context_package` (`dispatch.rs:5913` at BASE) answered **the running program's** package
rather than the receiver context's, and never checked validity. Its own comment claimed "nothing here
hands one out that outlives its activation", which measurement contradicts. Run against the BASE
binary and the HEAD binary in turn, on `c = r(); say c~package~class~id` over `::routine r; return
.context`:

```
oracle    rc 158   Error 98.981:  Target RexxContext is no longer active.
BASE      rc 0     dead package: Package
HEAD      rc 158   byte-identical to the oracle on all three descriptors
```

**The row read `answers` in `method-bodies.txt` and `agree` in `introspection-arity.tsv` the whole
time it was wrong**, which is this phase's own finding about its instruments seen once more: both
tables probe a *live* `.context`, and the defect is only reachable through a dead one. It now goes
through the same receiver-to-activation lookup as the other fourteen, and the false sentence is gone
rather than hedged.

## Witnesses

Seven programs, each byte-identical to the oracle on **stdout, stderr and exit status**, on **both**
engines (`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`):

| program | rc | what it pins |
| --- | --- | --- |
| `corpus/lang/rexx_context.rex` | 0 | all fifteen readers at the top level, in a label, in a `::METHOD` and in a `::ROUTINE`; `digits`/`form`/`fuzz` in force against `.RexxInfo`'s defaults in one program; `~args` holes; `~variables`' contents including `SELF`/`SUPER` and the stem/compound split; the `~executable` identities; and `R6`, which distinguishes the ask-order rule from the depth rule |
| `corpus/lang/rexx_context_arity.rex` | 163 | `93.902` for an argument to a zero-parameter row, whole transcript |
| `corpus/lang/stack_frames.rex` | 158 | the four frame kinds on one stack, all ten `StackFrame` rows, the snapshot outliving its frame, and `98.981` for the matching dead context, whole transcript |
| `corpus/lang/rexx_context_new.rex` | 163 | `93.967` naming the `RexxContext` class |
| `corpus/lang/stack_frame_new.rex` | 163 | `93.967` naming the `StackFrame` class |
| `corpus/lang/map_collection_of_index.rex` | 168 | `88.923`, whose first substitution is `.context~name` |
| `corpus/lang/map_collection_of_pair.rex` | 168 | `88.924`, the other one |

Every object-answering row is read through a **second send**: `~args`, `~variables`, `~stackFrames`,
`~executable`, `~package` and `StackFrame~context` are each asked `~class~id` *and* read for a
value, per the ruling that the arity instrument cannot see a class.

**The two witnesses this task owed Task 1, and the answer to its question.** `MapCollection~of`'s
`88.923` and `88.924` arms substitute `.context~name`, which was loud when Task 1 ran. **This crate
now reaches the oracle's messages exactly** — including the traceback frame above them:

```
  1258 *-*       Method OF with scope "MapCollection" in package "REXX" (no source available).
    12 *-* d = .Directory~of('k1')
Error 88 running REXX line 1258:  Invalid argument.
Error 88.923:  OF argument 1 must be a single-dimensional array; found "k1".
```

The first substitution is `OF` — the *message name* the method was invoked under — so these two
programs witness `RexxContext~name` from inside a Rexx-coded library method as much as they witness
`of`.

**A refusal is witnessed untrapped, and that is forced rather than chosen**: inside a `SIGNAL ON
SYNTAX` handler, `rc` carries only the major code (measured `93`, both sides) and `condition('D')`
is empty, so the subcode and the message — the part these rows are about — are only visible on
stderr. That is why five of the seven programs end at a raise.

## Controls

Six mutations, each **predicted in writing before it was applied**
(`scratchpad/t6/controls-predictions.md`), each applied by a script that asserts its own edit
matched exactly once, each reverted from a `cp` copy whose `sha256` was checked afterwards.

| # | mutation | predicted | observed |
| --- | --- | --- | --- |
| 1 | remove the `RexxContext~NAME` row | both witnesses rc 120 | **confirmed** — `rexx_context.rex` rc 120 `method "NAME" of class "RexxContext" is not implemented`, and `map_collection_of_index.rex` rc 120 too. Its *stdout* is unchanged, because its one `say` precedes the raise |
| 2 | bind `StackFrame~TYPE` to `frame_name` | B1's first field becomes `INNER` | **confirmed** — `B1 [INNER][INNER][66][1][The NIL object]` |
| 3 | a row naming a method the class does not answer | panics at `ObjectModel::build` | **confirmed, with a correction below** |
| 4 | delete the clause snapshot | every `~line` reads 1, every `~traceLine` loses its text | **confirmed** — `B1 [ROUTINE][INNER][1][1][...]`, `C1 [     1 *-* ]` |
| 5 | mint `~invocation` by depth | `rexx_context.rex` R6 becomes `[1][2]`; **`stack_frames.rex` stays green** | **confirmed, both halves** |
| 6 | drop the arguments snapshot | `E1`/`M2`/`R2`/`R3` and `X1`/`X2`/`D3` redden | **confirmed except `E1`** |

**Control 3's chosen test could not see its own subject.** `cargo test --release -p rexx-exec --lib
no_written_directive` ran (`1 passed`, so it was not the empty-filter trap) and stayed **green**: it
builds no `ObjectModel`. Running an actual program panics as predicted, at `dispatch.rs:1302`,
`NATIVE_METHODS names RexxContext~ZORKNOTAMETHOD, which that class's behaviour does not answer`. The
prediction "every test that builds a model fails" was too broad and the test I picked was the wrong
instrument; the finding stands only because the second command was run.

**Control 5's green half is the point of `R6`.** `stack_frames.rex` stays byte-identical under the
depth rule, because nothing asks for an invocation id before its walk, so ask order and depth
coincide at 1,2,3,4 there. The brief's wrong rule would have passed that witness. `R6` — read from
inside a `::ROUTINE` after the top level has already asked — is the row that reddens.

**A seventh control, on an assertion rather than on a row, and the assertion failed it.**
`Interp::frame_at_mut` indexes `Interp::suspended` **backwards** where `Interp::frames` counts
forwards, so an off-by-one would write one activation's `~invocation` id onto its caller — two
plausible numbers, nothing to notice. I added a `debug_assert_eq!` holding the two indexings equal
and then mutated the arithmetic to `below + 2` to prove it could fail. **It stayed green**, because
the first version recomputed the backward index *inside* the assertion and so compared `frames`
against a formula rather than against the answer. Rewritten to take its left-hand side from what the
function is about to return, the same mutation is rc 101, `frame_at and frame_at_mut disagree about
the activation at depth 1`. The binary's mtime was checked either side of each build, because
`cargo build` printed `Finished` in 1.26s and an incremental build reads exactly like a skipped one.

**Control 6's prediction was falsified in one cell.** `E1` reads `[Array][0]`, which is also the
*correct* answer: the top-level activation has no arguments. The reddening came from `M2`, `R2`,
`R3`, `X1`, `X2` and `D3`.

## Shared artifacts, before and after

**`corpus/introspection-arity.tsv`** — refreshed with
`REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity`,
exit 0, 25 passed.

| rows | before | after |
| --- | --- | --- |
| `RexxContext`'s fourteen readers | `send-differs` | `agree rc0` |
| `RexxContext package` | `agree rc0` | `agree rc0` (now for the right reason — see above) |
| `StackFrame`'s ten | `setup-differs` | `agree rc0` |

Checked by the method rather than by eye: `24 24` changed lines, **zero** removed lines carrying
`agree`, **zero** carrying `answers`, and **zero** changed rows outside `RexxContext` and
`StackFrame`.

**`corpus/refusal-sites.tsv`** — `cargo test --release -p rexx-exec --test refusal_sites`, exit 0,
5 passed. `Raised::context_not_active` is a new row (`send`, `agrees`, `98.981`).
`Loud::builtin_option_object` moved from `body` / `off-send-surface` to `body+send` / `diverges`,
because `RexxContext~condition` now constructs it. **That verdict was run, not reasoned**: the
program in its witness column answers `Directory` on the oracle at rc 0 and rc 120 here. 125 further
rows changed only their line number, because inserting `context_not_active` into `error.rs` moved
every constructor below it.

**`corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C`** — the seven witnesses added to both, in the same
order, which is what `coverage.rs` holds them against.

**`corpus/method-bodies.txt`** — refreshed with
`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`, exit 0,
21 passed. Three readings, because the brief asked for the receiver override's before and after and
the answer needed a third:

| rows | at BASE | with the work, before the override | with the override |
| --- | --- | --- | --- |
| `RexxContext`'s fourteen | `loud method "…" of class "RexxContext"` | `answers rc 0` | `answers rc 0` |
| `RexxContext package` | `answers rc 0` | `answers rc 0` | `answers rc 0` (green and wrong at BASE — above) |
| `StackFrame`'s ten | `loud method "NEW" of class "StackFrame"` | **`unanswered`** | `answers rc 0` |

**The hollowness the brief feared is caught by the instrument itself, and the middle column is the
proof.** Binding `~new` moves the ten rows off `loud` — and onto `unanswered  a bare ~new raises;
the method was never sent`, not onto `answers`. `method-bodies.txt` already distinguishes "both sides
agreed" from "the method was asked", which is the lesson `File`'s fifty rows taught it. So `new`
could not have closed those ten rows even if it had been bound first.

Checked by the method: `24 24` changed lines, **zero** removed lines carrying `answers`, **zero**
carrying `agree`, **zero** changed rows outside `RexxContext` and `StackFrame`.

**A third instrument finding, and it is why the override needed a code change to take effect.**
`method_bodies.rs` decided "was the method really sent" from `class.construction.is_some()` alone,
which does not consult `RECEIVER_OVERRIDES`. **Every override before this one was on a class that
*also* had a construction expression**, so the two questions had the same answer and the gap could
not show; `StackFrame` is the first override on a class with none. With the override in force and
the methods genuinely being sent, the ten rows still recorded `unanswered`. The fix is
`receiver_is_committed`, one predicate used where the flag is set, and it moved **no other class's
rows** — which is what says the reasoning about why it never showed is right rather than convenient.

## Fast checks

Run in the working tree, before any commit.

| check | command | result |
| --- | --- | --- |
| format | `cargo fmt --all --check` | exit 0, no output |
| lint | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no output |
| tests | `cargo test --release --workspace --no-fail-fast` | exit 0, 116 `test result: ok`, **2282 passed**, no `FAILED` |

Neither run was wrapped in `memcap`, per the ruling. Two other instruments were run separately and
are green:
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` reports **457 of 457 matching**
with `mode: STRICT (the gate)` on its stderr, and `cargo test --release -p rexx-exec --test
collect_stress` is exit 0, 9 passed — that is the collect-on-every-allocation instrument, and it
covers the eight new witnesses, which is what says the frames' rooting is right rather than lucky.

An earlier run of the same command had a second failure,
`the_seam_token_is_named_only_by_the_dispatch_module`: `src/dispatch/context.rs` is a new consumer of
the seam's clearance token and had to be added to `CLEARANCE_CONSUMERS`, which the global constraints
ask for by name.

## Gates

| gate | result |
| --- | --- |
| G1 | **G1** |
| G2 | **G2** |
| G3 | **G3** |
| G4 | **G4** |
| G5 | **G5** |
| G6 | **G6** |
| G7 | **G7** |

## Rulings table

Every ruling in the brief's `## Rulings` section, re-read at the end rather than from memory, against
what was done with it.

| ruling | what was done with it |
| --- | --- |
| **`.StackFrame~new` — do the work first and let `new` follow it** | **Applied in that order.** The readers landed first; `new` is bound in `context::NATIVE_CLASS_METHODS` through Task 2's `native_unsupported_new`. |
| **`.RexxContext~new` is this task's, and no table's rows are receiver-blocked on it** | **Applied and verified.** `class-set.txt:96` gives `RexxContext`'s receiver as `.context`, so no row moved; `rexx_context_new.rex` is its witness. |
| **The arity instrument cannot see an object's class — witness each object-answering row through a second send** | **Applied.** `~args`, `~variables`, `~stackFrames`, `~executable`, `~package` and `StackFrame~context` are each read for `~class~id` *and* for a value; `~executable`'s row also asserts identity with `.routines['RTN']` and `.kk~method('M')`. |
| **A control's green needs the same suspicion as its red** | **Applied, and it caught two things.** Control 5's green half is reported with the row that should have reddened and why it did not. Control 3's chosen test was green *because it could not see its subject*; the finding stands only because a second command was run. |
| **The witness you write is drawn from your own implementation — enumerate the frame kinds, the `~type` spellings, and what a dead `StackFrame` answers** | **Applied, and it found a defect.** Enumerating produced `rexx_context_edges.rex` (DROP, `PROCEDURE`, both routine routes, an inherited method's scope-versus-receiver, a clause nested two `DO`s deep) and the `R6` line, which is what reddens under control 5 where `stack_frames.rex` does not. The dead frame was re-measured: it retains everything. |
| **Every defect this phase has found returned success** | **Confirmed again.** All three declared divergences are rc 0 on both sides, and the `native_context_package` defect was rc 0 answering a `Package`. |
| **`setSecurityManager` is DECLINED, not licensed** | **Not applicable** — no row of this task touches it. |
| **The shared artifacts your change moves are yours to commit, with before/after verdicts and the refresh command quoted** | **Applied** for the arity table, `refusal-sites.tsv`, `phase-5c.txt` and `EXPECTED_SUBSET_5C`, checked by the method rather than by eye. `method-bodies.txt` is **blocked** — see *Concerns*. |
| **`rustfmt --edition 2024 <path>`, never bare; no `memcap` on the release fast check** | **Applied.** Every format run named its paths with `--edition 2024`; the fast check ran bare. |
| **Do not run two `cargo` commands in the working tree at once** | **Applied**, and the same check found the tree was not free at dispatch. |
| **`~invocation` is a lazily minted id, not a depth count** | **Applied.** `Interp::next_invocation` plus `Activation::invocation: Option<u32>`, minted on first ask; control 5 mutates it to the depth rule and `R6` reddens. |
| **`~type` has six values and `INTERPRET` is a DIVERGENCE, declared not closed** | **Applied.** Declared with the three-frames-against-two transcript; `COMPILE` named beside it. Not closed. |
| **The brief's dead-frame transcript does not prove what it was offered for — re-measure and report what a dead frame retains** | **Applied.** Re-measured: `items` is 2, and every one of the ten rows still answers. |
| **The per-activation state: design approved, with an interleaved A/B; stop and tell me if any axis moves more than about 1%** | **Applied, and I stopped.** The design shipped as approved in substance with two changes forced by the measurement (the snapshot is taken per call rather than per clause, and the name is not `Rc`-shared on the method path). `emptyloop` +0.527% and `varlookup` +0.453%; `dispatch` +1.166% and `dispatchclass` +1.092% are over the bar and were reported before committing. |
| **`native_context_package` is a green row that is wrong; fix it, report the verdict before and after, correct the false sentence** | **Applied.** Fixed, both verdicts reported, the transcript taken against the BASE and HEAD binaries, and the false sentence deleted rather than hedged. |
| **`StackFrame~executable` and `RexxContext~copy` stay loud** | **Applied.** Both left loud and named in *Divergences*. |
| **BASE is the HEAD you find** | **Applied.** `d8b013026`, stated at the top, and the tree-not-free finding that preceded it is stated too. |

## Concerns

**1. `StackFrame`'s renderings needed a field the crate did not have, and the ask was granted.**
`SAY` on an `Array` joins its items through `Interp::to_text`, whose `Body::Native` arm answered
`NativeObject::rendered()` — the *default name*. `StackFrameClass` overrides **`stringValue()` and
`makeString()`** to answer the traceback line and leaves `defaultName()` alone, so one field could
not answer both: measured, oracle rc 0, six renderings of one frame — `say f`, `~string`,
`~makeString` and an `Array` join are the traceback line, `~objectName` and `~defaultName` are
`a StackFrame`. `NativeObject` now carries an optional `string_value` that `to_text` prefers, set
**for `StackFrame` alone** and `None` everywhere else, and `~objectName` reads `rendered` through an
arm of its own beside `Primitive::VariableReference`, which already sat there for the same reason.
All six renderings agree, and so does the `~objectName=` corner (`f~objectName = 'tagged'` then
`~objectName` is `tagged` while `say f` is unchanged). **No other native class was surveyed for the
same split**, per the ruling that scoped this; if one has it, it is not recorded here because I did
not look.

**This is the divergence the table refresh found and my own witnesses did not**, because none of
them renders an array of frames. It is also the one where the cheap workaround would have been the
phase's own recurring defect: setting `rendered` to the traceback line buys a green array join and a
silently wrong `~objectName`, which `no_row_started_diverging_or_stopped_answering` would not have
caught either.

**2. The two send axes are at +1.17% and +1.09%, and Task 9 owes them a measurement against the
phase's start.** Reported before committing per the ruling and committed on it. Two isolation builds
bound the residue **from below**: it is not in the clause snapshot (0.08pp) or the per-activation
convention snapshot (0.38pp), so what is left is the `Rc` conversion of `CallContext.arguments` and
the 48 bytes `Activation` grew — three fields. A later redesign starts from those two figures rather
than re-deriving them.

**`dispatch` and `dispatchclass` must be measured against the phase's starting commit, not against
this task's BASE.** Phase 5a passed *every* per-step perf gate under 1%/2% and summed to **+32%**; a
per-step threshold structurally cannot see a total, and this is the second Phase 5i task to move a
send axis. Recorded as row 12 of `found-not-fixed-register.md` so it does not depend on being
remembered.

**3. `~thread` and `~interpreter` are constants and will stay wrong under Phase 6.** Each stands for
a lazily minted id off a counter this crate has exactly one subject for. The moment a second thread
or a second interpreter instance exists they are wrong, and nothing in this crate will notice — the
differential runs one thread. Named here because that is the phase's own rule about readers that
answer from a constant, and because the row that breaks them is not a row this task can write.

**4. `~condition` refuses rather than answering inside a handler.** The `.nil` arm is the one the
tables probe, so both rows close on it; the other arm is `CONDITION('O')`'s and is recorded in
`refusal-sites.tsv` as `diverges` with its own witness. A later phase closing `CONDITION('O')` closes
this with it, and if it does not, this row is a shell in the corner the tables do not reach.
