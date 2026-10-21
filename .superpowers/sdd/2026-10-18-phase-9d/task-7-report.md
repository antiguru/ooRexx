# Task 7.1 report: the reached refusals, and the split by disposition

Status: DONE_WITH_CONCERNS. Base `eeab7f128`. Commits: `acd38baee` (code, tables, witnesses),
`a9bec0326` (refusal-sites.tsv re-derived after rustfmt), and the records and report commit
that follows them.

Records: `docs/superpowers/records/2026-10-18-phase-9d/t7.1/`.

## Rows implemented

| row (refusals.tsv) | now | witness |
|---|---|---|
| context.rs StackFrame EXECUTABLE, a library routine's frame | the `Routine` object: the one `~call`/`~callWith` was sent to, else the one `.routines` and `findRoutine` hold for the library code | `lang/stackframe_library_routine_executable.rex`, `lang/stackframe_merged_library_executable.rex` |
| object_protocol.rs OBJECTNAME= on .nil by a scope override | renames .nil for every rendering (`say`, `~string`, `~objectName`, concatenation, an array's `toString`, error text, `TRACE R`) | `lang/object_scope_override_nil.rex` |
| object_protocol.rs COPY on .nil by a scope override | a distinct `Object` with .nil's behaviour (COPY, START, OBJECTNAME= hidden), its current name, and .nil's hash | `lang/object_scope_override_nil.rex` |

Mechanism, StackFrame: `NativeFrame` gains `executable: Option<ObjRef>` (traced in
`object_roots`, reset in `push_native_frame`), set by `run_library_routine`'s new argument
(`Some(receiver)` from `begin_routine`, `None` from every call by name). Both frame builders
answer it, or `library_routine_object(code)`: `build_native_frame_at` for a live frame, and
`blame_native_routine` (now passed the frame's `executable` and `code`, read before the pop) for
the frame a condition carries. The crate already makes `.routines~r`, `loadExternalRoutine` and
`findRoutine` one object per library code (probe `id1.rex`, `1 1 1` on both).

Mechanism, .nil: `Interp::nil_name` (`Option<Box<[u8]>>`), read by `to_text`, `text_len_inner` and
`try_text`'s `Decoded::Nil` arms. The copy is `Body::Instance { class: .Object, behaviour:
nil_behaviour, name: Some(current name) }`; `hash_value` answers `NIL_HASH` for an instance on
`nil_behaviour`, since `RexxNilObject::hashValue` is a member the clone copies.

Semantics (plan Ruled 13): rexxref `oneof.xml:109-112` gives .nil "only the methods of the Object
class", and `fundclasses.xml:2740-2742` and `:3094` define COPY and OBJECTNAME= for any receiver.
The oracle agrees. StackFrame~executable: rexxref and the oracle agree.

Witnesses: red at the base binary (`witnesses/*.base.{out,err,rc}`: rc 120, `method "COPY" of class
"Object"` and `method "EXECUTABLE" of class "StackFrame"`), identical to the oracle on stdout, stderr
and rc at the head (`witnesses/*.{oracle,head}.*`), oracle output identical over 4 runs each. Each
passes under collect-on-every-allocation (521, 208 and 309 collections, output identical).
Mutations (`mutations.txt`), each red: the live-frame arm answering `None` (merged witness, `live`
lines), ignoring `ran` (library witness, `copy` line), `set_nil_name` a no-op and the `NIL_HASH`
arm removed (nil witness).

## Rows not implemented: rexxref and the oracle part

COPY through `.Message~new(x, .array~of('COPY', .Object))~send` on a built-in class, a Rexx-defined
class, a RexxContext and .RexxInfo (seeds `copy_class_message`, `copy_class_rexx_message`,
`copy_context_message`, `copy_rexxinfo`; outputs in `probes/`).

* Oracle: answers a copy. `copied Class 0 Array 2`; `copied Class 0 ZZ m 0`; `copied RexxContext`;
  `copied RexxInfo 0`. A class copy shares the original's method dictionary and instance behaviour
  until a `define` (probe `cls1.rex`: `c~new~class == .zz`, `c~new~isA(c)` 0; after `c~define('N',
  ...)` `.zz~new~hasMethod('N')` 0; a `.zz~define` afterwards does not reach `c`'s instances).
  `RexxClass::copyRexx` (`classes/ClassClass.cpp:161-171`) carries the comment "An override for the
  copy method to keep Class objects from being copied"; the scope override reaches
  `RexxObject::copyRexx` instead.
* rexxref: `fundclasses.xml:63-65`, "Note that the copy method is forbidden for Class and all other
  class objects, and will result in an error." `utilityclasses.xml:7934-7942` (RexxInfo): "Only
  one instance of the RexxInfo class can be obtained via the .REXXINFO environment symbol, other
  instances cannot be created or copied." `utilityclasses.xml:7549` and `:7574` (RexxContext): the source comments "new() and copy() are
  forbidden" and "copy() is forbidden", and `:7545`, "They cannot be directly created by the user."
* Derived answer: 93.970, as the plain sends answer on both sides today (probe `plain.rex`: `COPY
  method is not supported for object The Array class.`, `... a RexxContext.`, `... a RexxInfo.`).
* Held: `Loud::uncopyable_copy`, owner Phase 9, rc 120. No Deviation written. If Moritz rules for
  the derived rule, the change is the three `copy_object` arms returning
  `Raised::copy_not_supported` over the receiver's string value.

## The split (template's last instance)

`grep -rn 'Loud::native_method(' rust/crates/rexx-exec/src` now finds nothing; the constructor is
gone. `awk -F'\t' '!/^#/ && $NF != "disposition" {print $NF}' refusals.tsv | sort | uniq -c` gives `guard`,
`implement` and `oracle-crash` only: no row is ruled Deviation or later phase.

| site | constructor | disposition |
|---|---|---|
| dispatch.rs `invocable` | `unbound_method` | GUARD, the dispatch.rs row |
| hash.rs `refusal_outside` | `hash_collection_without_store` | DEVIATION, oracle-crashes entry 50 |
| context.rs `frame_executable` (what is left: a package hook's frame, one built from a failure site with none) | `frame_without_executable` | GUARD, the context.rs row |
| object_protocol.rs `copy_object`, Directory and StringTable `Body::Native` | `interpreter_table_copy` | GUARD, that row |
| object_protocol.rs `copy_object`, class object, RexxContext, RexxInfo | `uncopyable_copy` | Phase 9, held for the ruling above |
| object_protocol.rs `native_object_name_set` | removed | implemented |

hash.rs: the guard rows (condition object, a subclass whose INIT does not forward, the package
tables) and the crash row reach one site, which cannot tell them apart; the guard receivers answer
before it, so the site keeps one constructor, DEVIATION, and its row says so.
`the_scanner_reads_each_owner_form` now asserts the phase-valued form on `uncopyable_copy`;
`the_scanner_finds_constructors_and_only_constructors` names `unbound_method`.
`src/environment/tests.rs`'s refusing destination (entry 50's shape) expects the new text, and
`oracle-crashes.txt` entry 50's sentence quoting this crate's refusal is corrected.

## Checks

| command (from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t7-target`, `memcap 8G`, `-j 4`) | exit |
|---|---|
| `cargo fmt --all --check` | 1: `crates/rexx-exec/tests/stream_model.rs` only, unformatted at `eeab7f128` and untouched here |
| `cargo clippy -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `cargo test --release -p rexx-exec --lib` (`checks-lib.txt`) | 0, 1167 passed |
| `cargo test --release -p rexx-exec --no-fail-fast --test refusal_sites --test refusal_dispositions --test representation_refusals --test owners --test closed_phases --test phase_ten_reasons` (`checks-tables.txt`) | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 0, 1152 of 1152 matching |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `cargo test --release -p rexx-exec --no-fail-fast --test method_bodies --test native_entries --test library_routine_memory --test collect_stress --test concurrency_tests --test gate_table_d` (`checks-touched.txt`) | 101: `collect_stress`'s `the_l0_subset_passes_again_under_collect_on_every_allocation` only, red at `eeab7f128` too |

The whole-suite run was not made: the dispatch's check list does not name it.

## Perf

`callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -p "startup interpret parse churn
sayloop" -o .../cg-9d/t7.1-run pin=.../rexx-run parent=cg-9d/t7.1/rexx-parent head=cg-9d/t7.1/rexx-head`,
parent built from `eeab7f128`, head from `acd38baee`'s code, exit 0. Own delta, libc subtracted,
head less parent: startup +0.0002, interpret 0.0000, parse 0.0000, churn 0.0000, sayloop +0.0001.
Head against the pin, libc subtracted: startup -0.1206, interpret -4.6330, parse -0.1416, churn +0.1068, sayloop
-0.0639. New refusal constructors are `#[cold]`.

## Concerns

1. **A copied library `Routine` reached by name answers the canonical object.** `probes/addr.rex`
   (`.context~package~addRoutine('X', .routines~r~copy)`) and `probes/wr.rex` (`.routines['X'] =
   copy`): the oracle's frame answers the copy (`ROUTINE X 1 0`), this crate the canonical object
   (`ROUTINE X 0 1`). Both routes end at `Resolved::MergedLibraryRoutine(code)`, which drops the
   object; base refused these loudly. Carrying the object needs it rooted in the call-site
   `Resolved` cache (kept for the body's life, as the oracle's instruction keeps it), or a lookup
   at frame time. Proposed owner: a Task 7.1 fix round.
2. **COPY on a class, RexxContext or RexxInfo by a scope override** waits on Moritz (above).
3. **`collect_stress` is red at base.** `stress.txt`: `package_option_writes.rex` and the
   `raise_*` and `reply_as_the_last_clause.rex` programs panic at `not_in_arena`'s "a live value"
   (`dispatch.rs:1799`) under collect-on-every-allocation, at `eeab7f128` and here alike; the
   panic ends the test at its first such program, so the rest of the subset never runs there.
   Proposed owner: the controller.
4. **`stream_model.rs` fails `cargo fmt --check` at base** (Task 6.15's last commit). Left alone:
   another agent may hold it.
5. **`.Routine~new(...)~call`'s frames differ, unrelated to this task.** `probes/rn.rex`: the oracle
   lists `ROUTINE m` then `METHOD CALL`, this crate `ROUTINE CALL` alone; `sf9.rex`'s `viam` lines
   show it. No library is involved. Proposed owner: the controller.
6. **.nil's name is per interpreter instance**; the oracle's `TheNilObject` is one per process. Only
   a process running several instances can tell.
7. Plain `.nil~copy` and `.nil~objectName=` are 97.1 on both sides (`nil_hidden_methods.rex`),
   which rexxref's "only the methods of the Object class" does not say. Not acted on.

## Fix round 1

Commits: `b7c45ef45` (the items below), `c0d2d69f1` (a `.nil` sentinel in place of an `Option`, for
perf), and the records and report commit after them. Records: `t7.1/r1/`.

1. **COPY of a class object, a RexxContext or .RexxInfo raises 93.970 on every route** (Moritz,
   2026-10-21). One `copy_object` arm raises `Raised::copy_not_supported` over the receiver's
   string value; `Loud::uncopyable_copy` and its refusal-sites row are gone. The seven `implement`
   rows of `refusals.tsv` read `done (Task 7.1)`, with a header line saying so. Witnesses:
   `lang/object_copy_uncopyable_receivers.rex` (the plain `~copy` of `.Array`, a `::class`,
   `.context` and `.RexxInfo`, 93.970 on both sides, identical to the oracle), and
   `dispatch::tests::a_scope_override_copy_of_an_uncopyable_object_raises` for the scope-override
   route, which the corpus cannot hold until the Deviation licenses it. `r1/probes/copyrules.rex`
   records both routes on both sides: the oracle answers `override copied Class`, `Class`,
   `RexxContext`, `RexxInfo` where this crate raises 93.970.
2. **I1: a copied library Routine reached by name answers the copy.** `Resolved::MergedLibraryRoutine`
   carries the `Routine` the lookup found, `.nil` for the one standing for the code:
   `MergedRoutine::LibraryObject` and a written `.routines` entry that is not the canonical object
   carry it. At the call, `Interp::found_routine_object` checks the copy is live and runs that code,
   then roots it globally. That covers the call-site cache keeping the resolution for the body's
   life, as the oracle's instruction keeps the routine. A copy collected before its first call
   answers `RoutineObject::Gone`, so the frame has no executable and refuses through
   `frame_without_executable` (its doc and GUARD row now name this case). Witness:
   `lang/stackframe_library_routine_copy_executable.rex` (addRoutine copy, `.routines['Y'] =
   copy` by CALL and as a function, all `1 0`, identical to the oracle). Probes `r1/probes/addr.rex`
   and `wr.rex` now match too.
   **Measured:** `Option<ObjRef>` in the variant widened `Resolved` and cost parse +0.077% and churn
   +0.053% (libc subtracted, against `t7.1/rexx-head`). A `.nil` sentinel keeps the payload at
   the old 16 bytes, and that cost is gone (below).
3. **M1:** the same witness reads a copy's `~call` from a live `.context~stackFrames` over
   `TestNestedAttach` (`live TestNestedAttach 1 0`). It prints ROUTINE frames only, because this
   crate's live stack lists no `METHOD CALL` level for a `~call` where the oracle does
   (`r1/probes/livecopy.rex`, the only difference). That is concern 5's class, which predates
   this task.
4. **M2: base's behaviour is restored.** `blame_native_routine` takes `native_reraise` first, then
   returns early on a recorded failure site, so the early return no longer keeps the flag.
5. **M3:** `the_scanner_reads_each_owner_form` reads the `Owner::Phase9` form from a fixture
   constructor.
6. **M4:** `value::tests::a_named_nil_renders_its_name_through_every_reader` reads `.nil` through
   `to_text`, `try_text` and `text_len` before and after `set_nil_name`, in the release `--lib` run.

Mutations (`r1/mutations.txt`), each red: `found_routine_object` always answering the canonical
object (`addRoutine`/`routines`/`function` lines); the live-frame arm ignoring the frame's object
(`live` line); RexxContext moved from the 93.970 arm to the copying arm (the unit test answers
`copied RexxContext`); `text_len_inner`'s `.nil` arm back to the fixed text (the unit test). The
first R3 attempt did not compile and is recorded as such beside its rerun.

### Checks (round 1, at `c0d2d69f1`'s code)

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-exec --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib` (`r1/checks-lib.txt`) | 0, 1169 passed |
| the refusal tables with `representation_refusals`, `owners`, `closed_phases`, `phase_ten_reasons` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0, 1154 of 1154 matching |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `method_bodies`, `native_entries`, `library_routine_memory`, `concurrency_tests`, `gate_table_d` (`r1/checks-touched.txt`, at `b7c45ef45`) | 0 |

Collect on every allocation (`r1/stress.txt`, an in-process probe): the five Task 7.1 witnesses
answer the same plain and under stress. `collect_stress` itself was not rerun; it is red at base
(concern 3).

### Perf (round 1)

`r1/callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -p "startup interpret parse churn
sayloop" -o .../cg-9d/t7.1-r1-run pin=.../rexx-run parent=cg-9d/t7.1/rexx-head
head=cg-9d/t7.1-r1/rexx-head`, exit 0. Own delta, libc subtracted, head less parent: startup
0.0000, interpret 0.0000, parse 0.0000, churn 0.0000, sayloop -0.0001.

### Deviation candidate (for the controller to write as Deviation 63)

* **Title:** COPY OF A CLASS OBJECT, A REXXCONTEXT OR .REXXINFO RAISES 93.970 ON EVERY ROUTE.
* **Rule:** `Object~copy` reached through a scope override (`.Message~new(o, .array~of('COPY',
  .Object))~send`) on a class object, a RexxContext or `.RexxInfo` raises 93.970, `COPY method is
  not supported for object <o's string value>`, as the plain `~copy` does on both sides.
* **Measurement:** the oracle copies through the override. `copied Class 0 Array 2` (built-in
  class), `copied Class 0 ZZ m 0` (a `::class`), `copied RexxContext`, `copied RexxInfo 0` (seeds
  `copy_class_message`, `copy_class_rexx_message`, `copy_context_message`, `copy_rexxinfo`;
  `t7.1/probes/`, `t7.1/r1/probes/copyrules.rex`). Its class copy shares the original's method
  dictionary and instance behaviour until a `define` (`t7.1/probes/cls1.rex`).
  `RexxClass::copyRexx` (`classes/ClassClass.cpp:161-171`) is commented "to keep Class objects
  from being copied"; the scope override reaches `RexxObject::copyRexx` instead.
* **rexxref:** `fundclasses.xml:63-65` ("the copy method is forbidden for Class and all other
  class objects, and will result in an error"); `utilityclasses.xml:7934-7942` (RexxInfo: "other
  instances cannot be created or copied"); `utilityclasses.xml:7545`, `:7549`, `:7574`
  (RexxContext: "They cannot be directly created by the user", and the comments "new() and copy()
  are forbidden", "copy() is forbidden").
* **Witnessed by** `dispatch::tests::a_scope_override_copy_of_an_uncopyable_object_raises`
  (`rust/crates/rexx-exec/src/dispatch/tests.rs`), the override route on all four receivers, and
  `rust/corpus/lang/object_copy_uncopyable_receivers.rex`, the plain route, oracle-identical.
  If the controller wants the oracle side asserted too, the program in the unit test fits a
  `licensed_divergences.rs` row. Its `oracle_stdout` would be the four `copied` lines of
  `copyrules.oracle.out`'s `override` rows, and the row needs the Deviation's `LICENSED DIVERGENCE
  WITNESS:` marker in the same commit.

### Concerns (round 1)

1. `RoutineObject::Gone` is reachable only if the call's own arguments drop the copy from its
   table and a collection runs before the call. No probe did that, and the GUARD row says so.
2. A copied *internal* routine (REXX package, `MergedRoutine::InternalObject`) reached by name
   has the same shape. `MergedInternal` drops the object, and `build_internal_frame_at` answers the
   canonical one. It is outside these rows and was not probed. Proposed owner: the controller.
3. Concern 5 above now has a second instance: a live `~call` on a library routine lists no
   `METHOD CALL` frame (`r1/probes/livecopy.rex`).
4. Each copy a call by name runs is rooted for the life of the process, one global root per
   distinct copy.

## Fix round 2

Commits: `9a5621ec0` (the items below), `f84c45029` (the pin derived from the kept resolution, for
perf), and the records and report commit after them. Records: `t7.1/r2/`.

**I-1: not the preferred fix, and why.** The brief asked for copies to stay valid only until
`routine_generation` moves. Built and run, that reproduced the oracle less well than base on
both behaviours it changes:

* **Which routine runs.** Once a call returns, the oracle keeps its routine at the instruction for
  the instruction's life (`RexxInstructionCall::execute`, `instructions/CallInstruction.cpp:190-197`:
  `setField(externalTarget, ...)` after `externalCall`). A later rebind of the name does not reach
  that site. `r2/probes/recode.rex` rebinds `Y` from a copy of `i` to a copy of `r`. On the
  oracle and at base the site keeps running `i` (`answered 93900`); the generation design ran `r`
  (`raised 93.900`).
* **The frame.** `which.rex`'s third line: the oracle answers `answered 93900` (the kept `c2`), and
  the generation design answered `raised Y 0 0 1` (it re-resolved to `c3`).

What landed instead keeps the oracle's lifetime:

* A call site **pins** the `Routine` copy its kept resolution names, through `Interp::repin` and a
  count per copy in `Interp::pinned_copies`. One global root is held per copy while any site keeps
  it.
* The pin is dropped when the site resolves to something else, when a raised call makes the site
  forget, and when the chunk is dropped: `prune_program_rows`, and the INTERPRET fragment's end,
  which walks nothing when nothing is pinned.
* No root is added per call and no `format!` runs per call. A pin or an unpin happens only on a
  call-site miss, or on a raise from a copy.
* `RoutineObject::Gone` is deleted: a copy is live because its site pins it or the lookup just
  read it from its table. A `debug_assert` in `found_routine_object` holds that.

Memory (`r2/rss.txt`, peak RSS, the re-review's loops plus `g-raise-site.rex`, which rebinds a
fresh raising copy at one site each pass):

| program | n | oracle | head | base `3b24a6445` |
|---|---|---|---|---|
| g-routines-interp | 100000 | 20332 KB | 47228 KB | 46956 KB |
| g-routines-interp | 200000 | 20012 KB | 46372 KB | 46968 KB |
| g-add-interp | 200000 | 20616 KB | 46012 KB | 46304 KB |
| g-routines-site | 200000 | 20060 KB | 49984 KB | 49844 KB |
| g-raise-site | 100000 | 20772 KB | 38904 KB | 38920 KB |
| g-raise-site | 200000 | 20612 KB | 39084 KB | 39680 KB |

Head is flat and equal to base. With the unpin skipped (`r2/mutations.txt`, P4), g-raise-site
grows to 392028 KB and then 743568 KB.

Calls to a copy (`r2/hot-callgrind.txt`, `hot.rex 100000`, whole-process Ir):

| calls to | head | base | change |
|---|---|---|---|
| a copy | 369873445 | 368571586 | +0.35% |
| the canonical object | 369893344 | 368594230 | +0.35% |

That is about 13 Ir per call, the same for both. An intermediate build that checked the copy's
record on every call measured 374266381 (+1.55%).

Wall clock on `g-raise-site.rex` 200000 is 3.85 s at head against 3.63 s at base: the pin and
unpin on each raise.

**Q1.** A call that raises makes its site forget its resolution (`Interp::forget_site`, in every
call op that keeps a site: `begin_call_args`, `begin_call_expr`, `begin_call_named` and
`begin_call_tree`), so the next call looks the name up again, as the oracle does. The forget
covers calls whose error comes back through the op: library and native routines and builtins. A
Rexx routine runs in an activation of its own, so its raise does not pass through the op, and
its site keeps its resolution as before.

**Q2.** `note_routines_write` moves `routine_generation`, so a site whose routine the oracle looks
up on every call sees a `.ROUTINES` write (`r2/probes/gen.rex`, `q2.rex`: head equals the oracle,
base answered `7` after the write).

**m-1, witness** `lang/call_site_routine_copy_lifetime.rex`:

* Q1, and the rebind-to-canonical line (`raised first ... 1 0 0`, `answered then`, `kept`,
  `raising copy`, `canonical raised X 1`).
* A kept copy whose table entry was rebound and dropped, run again after 300000 allocations
  (`stale frame Z Routine 0 1`).
* Q2 through a library-exported name (`exported 2 raised`).

It is identical to the oracle. Base differs on the four Q1 lines, the stale frame and the Q2
lines. Mutations, each red on the witness (`r2/mutations.txt`):

* no pin: rc 120, `a message send to a value whose object is no longer live`;
* no forget on raise: `answered then raised Y 1 0 0`;
* no generation move on a `.ROUTINES` write: `exported 2 answered 7`.

**m-2.** `frame_without_executable`'s doc and its GUARD row no longer name a collected copy: that
path is gone. Round 1's concern 1 was false, as the re-review measured: a call's arguments run
before its lookup.

Re-review probes (`r2/probes/`, head against the oracle): `which`, `recode`, `rebind`, `gen`,
`rooted`, `gone`, `stale` and `q2` are identical. `gone3/4/5` (the activation depth limit) differ
at base too, and `gone3` differs run to run on this crate. The depth the limit trips at moves.

### Checks (round 2, at `f84c45029`'s code)

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-exec --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib` | 0, 1169 passed |
| refusal tables, `representation_refusals`, `owners`, `closed_phases`, `phase_ten_reasons` | 0 |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 0, 1155 of 1155 matching |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `native_entries`, `library_routine_memory`, `concurrency_tests`, `gate_table_d` | 0 |
| `REXX_SUITE_GATE=1 ... --test suite l3_core_suite` (`r2/suite/`) | 101 then 0. The first run's only changed row is the known flake `SysFileTree TEST_TREE_DIRECTORY` (`run-1.txt`, `tests-1.tsv`); the second run matches `t6.15/r3/suite/tests.tsv` row for row. The run at `9a5621ec0` also matched it. |

### Perf (round 2)

`r2/callgrind.txt`: `callgrind.sh -B 0.5 -j 4 -p "startup interpret parse churn sayloop"`, pin
`rexx-run`, parent `cg-9d/t7.1-r1/rexx-head`, head `cg-9d/t7.1-r2/rexx-head`, exit 0.

| program | own delta, libc subtracted | own delta, whole process |
|---|---|---|
| startup | -0.0003 | -0.0015 |
| interpret | +0.0190 | +0.0109 |
| parse | 0.0000 | 0.0000 |
| churn | 0.0000 | -0.0002 |
| sayloop | 0.0000 | -0.0008 |

A pin field on each `CallSite`, the first build, cost interpret +0.18% (libc subtracted). It also
moved parse and churn by about -0.15%, a layout effect with no code behind it. The pin is now
read off the kept resolution, so `CallSite` keeps its old size.

### Concerns (round 2)

1. **I-1 is answered with the oracle's lifetime, not the brief's mechanism**, for the measured
   reasons above. The brief's mechanism is one edit away, in `kept_until_routines_change`, if the
   controller still wants it. It changes which routine runs in `recode.rex`.
2. **Q1 covers raises that come back through the call op.** A raise inside a Rexx routine's own
   activation leaves its site's resolution kept. In the oracle, that site would look the name up
   again.
3. `Interp::pinned_copies` is a count keyed by `ObjRef`: GC pin bookkeeping, not object state.
4. Q3 (copied REXX-package routines) is the controller's queued item, as the brief says.

## Fix round 3

Commits: `d6daeeaca` (I-2 and m-3), and the records and report commit after it. Records:
`t7.1/r3/`.

**I-2.** A call op forgets its site only when this call missed there (it resolved and remembered)
and then raised:

* `begin_call_args` and `begin_call_named` carry a `missed` flag out of their miss arm.
* `site_resolution_before_arguments` returns whether the site had a kept answer on entry, which
  `begin_call_expr` and `begin_call_tree` read.
* A routine kept from an earlier return stays kept through a later raise, as the oracle's
  `externalTarget` does (`instructions/CallInstruction.cpp:167-197`). `forget_site`'s doc says so.

Witness `lang/call_site_kept_routine_raises.rex`: the CALL form and the function form, each with a
copy and with the canonical object. Each keeps a routine, rebinds the name, raises through the
kept routine (`abc raised 88.907`), and then still runs the kept one (`2 answered 2`). It is
identical to the oracle. With the forget made unconditional, those four `2` lines turn red
(`2 raised 98.941`).

The witness's argument section (a raise in an argument) matches the oracle, but that mutation
does not reach it: in the CALL form the arguments run in ops ahead of the call op, so their raise
never comes back through it.

Q1's own line and the rebind-to-canonical line are unchanged: `call_site_routine_copy_lifetime.rex`,
`r3/probes/which.rex` and `rebind.rex` are identical to the oracle. The re-review's
`kept-raise.rex`, `kept-raise-fn.rex` and `kept-raise-canon.rex` are identical too.
`rexx-raise.rex` still differs; that is Q4, which is queued.

Hot path (`r3/hot-callgrind.txt`, `hot.rex` at 100000 and 200000 calls, per-call slope in Ir):

| calls to | round 2 | round 3 | change |
|---|---|---|---|
| the canonical object | 2501.1 | 2502.1 | +1 |
| a copy | 2501.2 | 2502.1 | +1 |

The +1 Ir is the miss flag. The fixed part is unchanged.

**m-3.** `tests/library_routine_memory.rs` gains `run_capped`, which runs a `main.rex` under the
oracle's `ulimit -v 1048576`; the existing test now goes through it. Two new tests use it:

* `interpreted_calls_to_fresh_routine_copies_run_under_the_oracles_memory_cap`: the re-review's
  INTERPRET shape at 400000 passes.
* `a_raising_routine_copy_rebound_at_one_site_runs_under_the_oracles_memory_cap`: a raising copy
  rebound at one site, at 300000 passes.

Each goes red with its unpin disabled (`r3/mutations.txt`). The INTERPRET-end unpin disabled
gives `memory allocation of 2032 bytes failed`, rc none. The repin's unpin skipped gives
`memory allocation of 19136528 bytes failed`. The test binary runs in 9.66 s.

### Checks (round 3, at `d6daeeaca`)

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-exec --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib` | 0, 1169 passed |
| refusal tables, `representation_refusals`, `owners`, `closed_phases`, `phase_ten_reasons` | 0 |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 0, 1156 of 1156 matching |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `native_entries`, `library_routine_memory` (23 passed), `concurrency_tests`, `gate_table_d` | 0 |
| `REXX_SUITE_GATE=1 ... --test suite l3_core_suite` (`r3/suite/`) | 0, `tests.tsv` identical to `r2/suite/tests.tsv` |

### Perf (round 3)

`r3/callgrind.txt`: parent `cg-9d/t7.1-r2/rexx-head`, head `cg-9d/t7.1-r3/rexx-head`, exit 0.

| program | own delta, libc subtracted | own delta, whole process |
|---|---|---|
| startup | 0.0000 | -0.0019 |
| interpret | -0.0045 | -0.0020 |
| parse | 0.0000 | -0.0001 |
| churn | 0.0000 | -0.0004 |
| sayloop | 0.0000 | -0.0011 |

### Concerns (round 3)

1. The canonical hot path is +1 Ir per call over round 2: the miss flag. It is not zero, as the
   brief asked.
2. Q4 is unchanged here; it is queued.
