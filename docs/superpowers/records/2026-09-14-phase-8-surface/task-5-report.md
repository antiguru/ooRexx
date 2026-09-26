# Task 5 report: the thread, method-context and call-context tables

Base: `e67b2f703`. Scratch: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-5/`
(below, `$S`).

## Status

Done at `fb52b794e`, gates green. Throw* members need a ruling (Concerns).

## Commits

* `8ea8a0f05` derived tests: refusing members, every member's type, what the test extensions reach
* `5a07762cc` group: object construction (numbers, logical, double, strings, value descriptors)
* `aa4652e98` group: conditions (raise, check, clear, info, decode, display)
* `cf82465dd` group: strings, buffers, mutable buffers, pointers, CSELF
* `dc49a3a3f` group: collections (array, directory, string table, stem, supplier)
* `d087c5a01` group: variables (context, object, references, guards)
* `d485573d4` group: messages, class search, the running call's description
* `d28d85f44` group: packages, libraries, executables
* `fb52b794e` group: global and local references, object memory, RegisterLibrary, GetInterpreterInstance, AttachThread/DetachThread on the calling thread; later owners for the embedding and guard members

## Design

* Every slot keeps its `unsafe extern "C" fn` type. Its body reads the context, finds the innermost
  activation, and hands decoded Rust values to `Activation` (`callbacks.rs`), which reaches the host
  through the `Surface` trait (`Host::surface()`; `None` records a loud refusal naming the member).
  `Interp` implements `Surface` in `rexx-exec/src/dispatch/library/surface.rs`; `FakeHost` implements
  it for the `ffi.rs` unit tests.
* Collection, send and class operations are message sends. Where the oracle's C++ `get` answers
  `OREF_NULL`, a `HASINDEX` send first gives NULL.
* A condition an extension raises is held on its `NativeFrame` and raised when the call returns,
  even if the extension answered normally (`settle_native_call`). A non-`SYNTAX` condition with no
  trap returns RESULT, as the oracle does.
* Global references: a table on `Interp`, rooted. A handle is the same pointer as a local, so
  `Host::resolve` consults locals then globals. `ReleaseGlobalReference` keeps the handle live,
  mirroring the oracle (`InterpreterInstance.cpp:678` calls `addGlobalReference`).
* Object memory: a Buffer-backed `NativeState::Data` per allocation, kept per receiver.
* `AttachThread` answers only on the home thread with a call in flight (the nested case
  `orxfunction`'s `TestNestedAttach` uses); any other use panics naming Phase 9. `DetachThread`
  does nothing.
* `REFUSING_MEMBERS` (`layout.rs`) names each still-refusing member and its owner;
  `refusal_owner` feeds both the abort message and `rexx-exec`'s Loud message.

## Slots filled and still refusing

Derived from the header by `tests/layout.rs`'s `a_populated_table_refuses_exactly_the_members_it_names`
(populated tables' stubs vs `REFUSING_MEMBERS`). Every member not in `REFUSING_MEMBERS` answers.
Still refusing, by owner:

* Phase 9 (embedding): `RexxInstanceInterface.Terminate`, `.Halt`, `.SetTrace`,
  `RexxThreadInterface.HaltThread`, `.SetThreadTrace`
* Phase 8 Task 6: `RexxInstanceInterface.AddCommandEnvironment`
* Phase 6 (guards): `MethodContextInterface.SetGuardOnWhenUpdated`, `.SetGuardOffWhenUpdated`
* (The Throw members of both tables were here, needing a ruling; filled in fix round 1, `9f7e7c96c`.)

Which test extension reaches which refusing member: `the_test_extensions_reach_only_members_that_answer`,
`STILL_REFUSING` = the guard-on-update members and `AddCommandEnvironment` (the Throw members
left it in fix round 1).

Reach of the filled members, derived with `$S/reach.py` extended over `task-5-forge/reach.cpp`
(command in `$S`, run from the oracle checkout): every filled member is reached by `orxmethod`/
`orxfunction`, by the forge, or by an `ffi.rs` unit test; the only one reached by neither
extension nor forge is `ValuesToObject`, which `ffi.rs`'s unit test drives with two descriptors.

## Oracle witnesses

Corpus programs (STRICT differential, the oracle's own `build/lib`): `library_callback_numbers.rex`,
`library_callback_conditions.rex`, `library_callback_strings.rex`, `library_callback_collections.rex`,
`library_callback_variables.rex`, `library_callback_messages.rex`, `library_callback_packages.rex`.

Forge: `task-5-forge/reach.cpp`, built by `task-5-forge/build.sh` with `g++ -shared -fPIC -O1`
against the oracle checkout's `api/` (byte-identical to this tree's, `diff -r`), NEEDED `libc.so.6`
only, no undefined `Rexx` symbol. Compared by `task-5-forge/compare.sh` (the standard wrapper from a
fresh `mktemp -d`, three descriptors; `$S` paths; it rebuilds `rexx-run` first, because a
`cargo test -p rexx-exec` rebuilds that binary from whatever tree it tests). On `aa4652e98`'s
binary: `cond1`, `nframe`, `raises` identical; `num1`, `num2` identical on `5a07762cc`'s; `bufs`,
`isstr`, `numstr`, `strs` identical on `cf82465dd`'s. On `fb52b794e`'s binary, every probe in `task-5-forge/probes/` identical on
stdout and stderr except `cond2`, `cond3`, `rcond`; `refs` identical on stdout and stderr, rc 139 vs 0
(the oracle segfaults at termination after `RegisterLibrary`). `cond2` stops at `StackFrame~EXECUTABLE`,
a crate Phase 5 refusal; `cond2b` (the same without that field) is identical.

Measured, not matched (see Concerns): `cond3` (a native `SYNTAX` re-raised into a trap) and `rcond`
(a `RaiseCondition` taken by `CALL ON` or `SIGNAL ON`).

`orxmethod`'s `TestRaiseCondition` casts its `OPTIONAL_CSTRING` description to a
`RexxStringObject`; reading that `DESCRIPTION` back segfaulted the oracle (rc 139, `CALL ON` handler
reading `condition('o')`). The witnesses pass no description.

## Negative controls

Each prediction written before its run; each mutated file restored from a copy and checked with `cmp`.

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| S1 | `ArrayAppendString`'s last argument `isize` in `layout.rs` | `every_member_has_the_type_the_header_declares` red, the rest of `tests/layout.rs` green | as predicted, 19 passed 1 failed |
| S3a | `THREAD` stops filling `StringData` | `a_populated_table_refuses_exactly_the_members_it_names` and `the_test_extensions_reach_only_members_that_answer` red, the rest green | as predicted, 22 passed 2 failed |
| S3b | `REFUSING_MEMBERS` loses `HaltThread`'s row | the first of those red, the reach test green (no test extension reaches `HaltThread`) | as predicted, 23 passed 1 failed |
| N1 | `logical_object` answers `.true` for zero too | the STRICT corpus gate red on `library_callback_numbers.rex` alone | as predicted, 604 of 605 |
| N3a | `set_mutable_buffer_capacity` asks `ensure_capacity` for the difference over the length | green: every witness grows a buffer whose length equals its capacity, a blind spot | green, 607 of 607; a case growing a buffer longer than its contents was then added (`grow` line) |
| N3b | the same, after that case | STRICT red on `library_callback_strings.rex` alone | as predicted, 606 of 607 |
| N4 | `item_at` sends `AT` without asking `HASINDEX` | STRICT red on `library_callback_collections.rex` alone | as predicted, 607 of 608 |
| N5 | `context_variable` answers an unset simple variable's derived name | STRICT red on `library_callback_variables.rex` alone | as predicted, 608 of 609 |
| N6a | `send_message` does not upper-case the name | STRICT red on `library_callback_messages.rex` alone | **falsified**: green, 610 of 610. This interpreter's `send_message` finds `LEFT` for `left`. **The conclusion first drawn here, that the mutant is unobservable, was false** (behaviour review 4): the witness was blind. See "N6a again" in fix round 1 |
| N6b | `send_message` drops the scope | STRICT red on `library_callback_messages.rex` alone | as predicted, 609 of 610 |
| N7 | `run_routine_as_program` runs the routine as a subroutine | STRICT red on `library_callback_packages.rex` alone | as predicted, 610 of 611 |
| N2 | `raise_held_condition` answers nothing where no trap takes a non-`SYNTAX` condition | STRICT red on `library_callback_conditions.rex` alone, at its `untrapped` line, 91.999 | as predicted, 605 of 606, rc 165 |
| N8 | `reallocate_object_memory` grows without copying the old bytes | STRICT red on `library_callback_memory.rex` alone, at the `0 hello` line | as predicted, 611 of 612, `0 ` for `0 hello` |

## Miri

`RUSTUP_HOME=<surface-4>/rustup-home CARGO_TARGET_DIR=$S/target-miri cargo +nightly miri test -p rexx-api --lib --offline`, Stacked Borrows:

* at `8ea8a0f05`'s tree: exit 0, 29 passed, 8 ignored (`$S/miri-0.txt`)
* at `5a07762cc`'s tree: exit 0, 31 passed, 8 ignored (`$S/miri-1.txt`)
* at `aa4652e98`'s tree: exit 0, 33 passed, 8 ignored (`$S/miri-2.txt`)
* at `cf82465dd`'s tree: exit 0, 34 passed, 8 ignored (`$S/miri-3.txt`)
* at `dc49a3a3f`'s tree: exit 0, 34 passed, 8 ignored (`$S/miri-4.txt`)
* at `d087c5a01`'s tree: exit 0, 34 passed, 8 ignored (`$S/miri-5.txt`)
* at `d485573d4`'s tree: exit 0, 34 passed, 8 ignored (`$S/miri-6.txt`)
* at `d28d85f44`'s tree: exit 0, 34 passed, 8 ignored (`$S/miri-7.txt`)
* at `fb52b794e`'s tree: exit 0, 34 passed, 8 ignored (`$S/miri-8.txt`)

## Gates

`$S/gates.sh` (Task 4's `gates2.sh` pattern), status `$S/gates/status.txt`, HEAD `fb52b794e` at start
and end, `git status --short` empty at end.

* G1 `cargo fmt --all --check`: exit 0
* G2 clippy `--workspace --all-targets -D warnings`, empty target dir: exit 0
* G3 `REXX_CORPUS_GATE=1 cargo test --workspace --release --no-fail-fast --no-run`: exit 0
* G4 same under `memcap 8G`, no `--no-run`: exit 0, 2685 passed / 0 failed / 4 ignored (summed
  `test result` lines of `g4-test-release.txt`), 0 Compiling lines, corpus 612 of 612; load at
  start 10.76 7.84 3.24
* G5 debug `--no-run`: exit 0
* G6 debug under `memcap 8G`: exit 0, 2686 / 0 / 4, 0 Compiling lines, corpus 612 of 612; load at
  start 5.57 6.38 4.00, after 32.43 30.72 16.74

Against base (2675/0/4, 2676/0/4, 604 of 604): +10 tests each profile, +8 corpus programs (the
`library_callback_*` witnesses).

## Concerns

* Throw* members: ruled (controller, provisional) and implemented in fix round 1 (`9f7e7c96c`).
* Divergences, recorded not matched:
  * a native `SYNTAX` re-raised into a caller's trap lacks the native frame and `PROPAGATED=1` (`cond3`);
  * a `RaiseCondition` taken by `CALL ON`/`SIGNAL ON` carries `POSITION`, the oracle's has none (`rcond`);
  * class searches from a method context use the caller activation;
  * `ObjectToValue` writes no partial value on failure;
  * collection operations are message sends, where the oracle calls C++ directly;
  * a buffer string keeps a side area until the call ends.
* `refs.rex` crashes the oracle at termination (rc 139) after correct stdout; which call leaves the state it trips on was not isolated (this entry first named `RegisterLibrary` without isolating it). Recorded as `oracle-crashes.txt` entry 16 in fix round 1.
* `orxmethod`'s `TestRaiseCondition` with a description segfaults the oracle on reading it back.
* N6a: withdrawn. The upper-casing is observable (UNKNOWN's name argument, `.context~name`, 97.1's message); the messages witness now sees it (fix round 1).
* `build.sh` once ran with no argument and wrote `/libreach.so`; removed, and `build.sh` now
  refuses without `OUTDIR`.

## Fix round 1

Base `fb52b794e`. Reviews: `task-5-review-boundary.md`, `task-5-review-behaviour.md`. Probes the
reviewers wrote are ported into `task-5-forge/reach.cpp` (`MBInfo`, `MBFill`, `KeepStr`,
`ReadKept`, `Outer`, `UseOuter`, `ForeignAttach`, `ZeroAlloc`) with `Throw`, `MThrow`, `SendThrow`
added; `build.sh` links the C++ runtime statically (`-static-libstdc++ -static-libgcc`), so NEEDED
stays `libc.so.6` (and its loader `ld-linux-x86-64.so.2`). Before outputs at `fb52b794e`'s binary
are in `$S/before/`.

### Controls (prediction written before each run)

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| F1 | `BufferState::writable` skips the reservation | `a_copied_mutable_buffer_is_writable_to_its_capacity` red under `cargo test` (the allocation assertion, or a heap abort) and under Miri (out-of-bounds write); `mbfill` differs again | as predicted: `cargo test` SIGABRT, Miri "attempting to access 64 bytes, but got alloc... only 3 bytes from the end", `mbfill` rc 134; restored, `mbfill` identical (`4000`, `4000`, rc 0) |
| F2 | `Interp::kept_c_string` answers `None` (the call's pool again) | `kept` differs as before (`same_address=0`) | as predicted, `same_address=0 kept=[ReadKept]` |
| F3 | the collection's prune drops every kept copy (`retain` false) | `kept` differs (`same_address=1` but the bytes read freed memory) if a collection runs between the calls; identical would mean the probe never crosses a collection | **identical**: the probe crossed no collection. `kept.rex` then gained `say gc('F')` between the calls; rerun: `same_address=0 kept=[<freed bytes>]`, red; restored, identical |
| F4 | the prune keeps every copy (`retain` true) | `a_kept_c_string_lives_and_dies_with_its_string` red at the length after the collection (2, not 1) | as predicted, `left: 2 right: 1`; restored |
| F5 | `activation_of` answers its own activation even while busy | `outer` rc 134 `RefCell already borrowed` again | as predicted, rc 134 `RefCell already borrowed`; restored |
| F6 | `NativeState::zeroed(0)` holds an empty `Vec` | `zero` gives `distinct=0 realloc_null=1` again | as predicted; restored |
| F7 | the Throw slots return after recording instead of unwinding | STRICT red on `library_callback_throw.rex` alone: `continue 1` and `CONTINUE` answering `1` | as predicted, 612 of 613; `method0 3 3.0 0 1`, `continue 1`; restored |
| F8 | `call_stub` resumes every payload, the marker too | STRICT red on `library_callback_throw.rex` alone, the process ending at its first Throw | as predicted: the resumed marker reached the interpreter thread, which panicked ("the interpreter thread running .../library_callback_throw.rex panicked"); no `extern "C"` frame sits above a top-level call to abort it first; restored |
| F9 | `array_dimension` answers `DIMENSION` as it is (no floor of 1) | STRICT red on `library_callback_collections.rex` alone (`0` for the empty array) | as predicted, 613 of 614; restored |
| F10 | `pool_variable_name` rejects every name with a period again | STRICT red on `library_callback_stems.rex` alone | as predicted, 613 of 614 (stdout, stderr and exit); restored |
| N6a again | `send_message` does not upper-case the name, after the witness gained the name-case lines | STRICT red on `library_callback_messages.rex` alone (`unknown got foo`, `whoami`, `"nosuchmethod"`) | as predicted, 613 of 614: `"nosuchmethod"`, `unknown got foo | whoami | unknown got fOo`; restored. N6a's first conclusion ("unobservable") was false: the witness printed 97.1's code and never its message, and a send's name reaches UNKNOWN and `.context~name` |
| F11 | the native raise builds `Raised::syntax(major, minor)` directly again | STRICT red on `library_callback_raise_codes.rex` alone | as predicted, 616 of 617; restored |
| F12 | `double_text` takes the general path at precision 0 | STRICT red on `library_callback_precision.rex` alone | as predicted, 616 of 617; restored |
| F13 | `find_class` drops the method's own package | STRICT red on `library_callback_class_search.rex` alone (`Hidden` not found) | as predicted, 616 of 617; restored |
| F14 | `Activation::object_variable` answers null | `the_object_variable_members_set_read_and_drop` red at the read-back, the other new ffi tests green | as predicted, 49 passed 1 failed; restored |

### Fix round 1: commits

* `366845a06` Boundary C1: a mutable buffer's bytes writable to its capacity (`BufferState::writable`), the forge gains the review probes
* `d669330c2` Boundary I1: a `CSTRING` answer lives as long as its string (`kept_strings`)
* `52c52b947` Boundary M1, M2, M3: home thread checked first; a busy outer context answers through the innermost activation; an empty buffer gets its own address
* `50ff2b3d3` format fix for `d669330c2` (a line edited after `cargo fmt` ran)
* `9f7e7c96c` Throw* ruling: the Throw members unwind (`extern "C-unwind"`, a marker payload, `call_stub` catches it); spec note appended
* `d75931bbd` Behaviour 2 and 3: `ArrayDimension` of an empty array is 1; a stem as an object variable
* `3562b16af` Behaviour 4: the messages witness sees a send name's case
* `5bf655524` Behaviour minors: unknown raise numbers (98.941), precision 0, a method's own class search; `cond3.rex` fixed
* `7b7b0a2f2` Records: `phase-4-exclusions.txt` and `oracle-crashes.txt` (entries 16-18)
* `99f10902a`, `e1c9d32a0` Boundary M4: Miri coverage

### Fix round 1: before and after

At `fb52b794e` (`$S/before/`) and at `e1c9d32a0`, the forge's probes by its `compare.sh`:

* C1 `mbfill.rex`: oracle `4000`/`4000` rc 0; before, ours `realloc(): invalid next size` rc 134; after, identical.
* I1 `kept.rex` (now with `say gc('F')` between the calls): oracle `same_address=1 kept=[hello world, kept]`; before, ours `same_address=0 kept=[ReadKept]`; after, identical. How the pointer is kept: `Interp::kept_strings` holds one NUL-terminated boxed copy per object (a `Box<[u8]>`, whose heap address never moves), keyed by the `ObjRef` with its generation; it is not a root. `collect_now` drops the copy of every heap object the collection freed; `FinishBufferString` drops the copy of the string it rewrites; a handle-carried value (inline text, small integer) keeps its copy for the interpreter's lifetime, one per distinct value. Names are kept by name in `kept_names`, never freed. The test host keeps none and answers the call's pool.
* Throw `throw.rex` (destructors logged into a buffer `Dtors` returns, because this crate's SAY is buffered to exit and a `printf` would reorder): before, ours rc 134 `CallContextInterface.ThrowException0 is not implemented (Phase 8)`; after, identical to the oracle, rc 0:
  `0: trapped 40 40.1 0 External routine "&1" failed. 0;` ... `A: trapped 93 93.900 1 Fred. A;`, `handler USER BAR desc add res`, `user: returned res USER BAR;`, `method: trapped 40 40.3 1 1;`, `method user: signalled USER BAR desc add res USER BAR;`.
  `sendthrow.rex` (a Throw two sends down): identical but for `propagated 1` against `propagated 0`, recorded.
* METHOD and FUNCTION shapes, `rust/corpus/lang/library_callback_throw.rex` (STRICT): identical; oracle and ours `method0 3 3.0 0 The NIL object` (CONTINUE never set), `continue 0` before each routine's condition, `routine 88 88.907 4 [min] [80] [100] [0]`.
* M2 `outer.rex`: before rc 134 `RefCell already borrowed`; after `USEOUTER`, identical. M3 `zero.rex`: before `distinct=0 realloc_null=1`; after identical `distinct=1 realloc_null=0`. M1 has no probe that can show it: the read it moves was a race before an abort; `attach.rex` still aborts loudly (Phase 9) where the oracle answers `1`.

### Fix round 1: Miri reach

`python3 $S/miri-reach.py` from `rust/crates/rexx-api/src` (members `table.X =` assigns in `ffi.rs`, against members the `ffi.rs` test code and `invoke/tests.rs` call; no test there is ignored under Miri): 86 of 180 reached (the review counted 26 of 168 at `fb52b794e`). Miri at `e1c9d32a0`'s tree: exit 0, 45 passed, 8 ignored (`$S/miri-16.txt`).
Still unreached, and why: the members whose answer is the interpreter's class, package, collection or variable-reference model (`Array*`, `Directory*`, `StringTable*`, `*Stem*`, `Supplier*`, `New*` of those and of methods and routines, the `Is*` predicates, `Find*Class`, `LoadPackage*`, `LoadLibrary`, `CallProgram`, `CallRoutine`, `GetPackage*`, `Get*Package`, `GetMethod`, `GetRoutine`, `GetSelf`, `GetScope`, `GetSuper`, `GetArgument(s)`, `GetCallerContext`, the environments, `ForwardMessage`, `SendMessageScoped`, `HasMethod`, `IsInstanceOf`, `IsOfType`, the variable references, `SetGuardOn/Off`, `RegisterLibrary`); the test host models none of these, and each slot's unsafe part is the same `activation_of`/`innermost_activation`/`name_of` the reached slots run. They are reached through the oracle's `orxmethod`/`orxfunction` in the STRICT corpus, not under Miri.

### Fix round 1: not fixed

* Directory subclasses overriding `AT`/`PUT`: recorded, not matched (the collection members would each need a scoped send to the base class's own method); `phase-4-exclusions.txt`, owner Phase 8.
* The divergences in the report's Concerns (native frame and `PROPAGATED`, `POSITION` on a trapped `RaiseCondition`, `ObjectToValue`'s partial value) and a blocking member on a busy outer context: recorded with owners, not fixed.
* The buffer string's side area: not recorded as a divergence; no Rexx program can observe it (behaviour review 3).
* `M3`'s alignment note (`Vec<u8>` promises alignment 1): not changed.

### Fix round 1: gates

`$S/gates.sh`, status `$S/gates/status.txt` (the first round's moved to `$S/gates-r0/`); HEAD `e1c9d32a0` at start and end, `git status --short` empty at end.

* G1 fmt: exit 0
* G2 clippy, empty target dir: exit 0
* G3 release `--no-run`: exit 0
* G4 release under `memcap 8G`: exit 0, 2697 / 0 / 4 (summed `test result` lines), 0 Compiling lines, corpus 612 → 617 of 617; load at start 12.38 10.21 7.51
* G5 debug `--no-run`: exit 0
* G6 debug under `memcap 8G`: exit 0, 2698 / 0 / 4, 0 Compiling lines, 617 of 617; load at start 7.39 7.83 7.33, after 3.68 7.37 7.55

Against `fb52b794e` (2685 / 2686, 612): +12 tests each profile, +5 corpus programs (`library_callback_throw`, `_stems`, `_raise_codes`, `_precision`, `_class_search`).

## Fix round 2

Base `e1c9d32a0`. Re-review: `task-5-rereview.md` (probes in `rereview-s5/p`, its `rr.cpp`, `libstatic`).
Before outputs are in `$S/r2/before/`.

### Controls (prediction written before each run)

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| G1 | `throw_exception0`'s record step panics (`panic!("planted")`) | the forge's `throw.rex` ends at rc 134 at the slot, stderr carries `planted`, stdout empty (SAY is buffered to exit), no `Dtors` line: the abort precedes any unwind | as predicted: rc 134, stdout 0 bytes, stderr `panicked at crates/rexx-api/src/ffi.rs:954:9: planted`; restored. Without `throw_after` the re-review measured rc 101 after the unwind (`RR_BUGPANIC`) |
| G2 | `Word` is `#[repr(C, packed)]` (alignment 1) | `aligned_bytes_are_aligned_and_read_back` red at `align_of`; `object_memory_is_aligned_for_any_c_object` green under `cargo test` (glibc hands out 16-aligned blocks, a blind spot) and red under Miri, which aligns an allocation only as asked | as predicted: `left: 1 right: 16`; green under `cargo test`; Miri `left: [2013844, 2024139, ...]` misaligned; restored |
| G3 | (the I1 fix itself, prediction for the witness) `grow.rex` at 1M calls | crate max RSS for `str` and `int` within 4 MB of its `none` baseline; before, 123 MB | as predicted: `none` 19104, `str` 18612, `int` 18796 KB (oracle 20696, 20472, 20400); before 18108, 123152, 123028 |
| G4 | `release_kept` drops the copy even while a global reference holds the value | `keptshort.rex` differs (`same_address=0`), `kept.rex` identical (a heap string, pruned only by collection) | as predicted: `same_address=0 kept=[<freed bytes>]`, `kept` identical; restored |
| G5 | (the I2(c) fix, prediction for the witness) `loadthrow.rex` with the re-review's `librr`/`librr2`, and this round's `lthrow.rex` and `lraise.rex` | `loadthrow` and `lthrow` identical to the oracle (`trapped 40 40.1 loadthrow;`, no line after `LoadLibrary`); `lraise` (a loader that only raises) stays identical | as predicted: all three identical, `loadthrow` stdout `trapped 40 40.1 loadthrow;`, stderr `loader throwing` then `rr unloader ran` on both |
| G6 | `load_library` ignores the hook's Throw (does not resume the unwind) | `loadthrow` differs by a `load-null;` (or `loaded;`) in the log, `lthrow` by an `after LoadLibrary` stderr line; both still trap 40.1 | as predicted: `trapped 40 40.1 load-null;loadthrow;`; stderr `after LoadLibrary 0 held=1`; restored |

### Fix round 2: commits

* `d5924d4ea` M3 (a defect panic inside a Throw member aborts at the slot, `throw_after`) and M5 (the call context's Throw members under Miri)
* `d26c340f7` alignment: object memory and Buffer bytes 16-byte aligned (`AlignedBytes` of `#[repr(C, align(16))]` words, safe code; no `unsafe` outside the granted files, so no `alloc` with a raw `Layout`)
* `aeb7edcbf` I1: a handle-carried value's kept `CSTRING` ends with its last call, unless a global reference holds the value
* `409221421` I2(c): a Throw in a library loader leaves the extension that loaded it
* `7d25fa15d` I2(a), I2(b): recorded as divergences with no owner; second spec note

### Fix round 2: before and after

* `grow.rex` (`$S/r2/grow.sh`: `/usr/bin/time -f %M` max RSS in KB, 1M calls, fresh dirs):

  | kind | before (`e1c9d32a0`'s behaviour) | after (`aeb7edcbf`) | oracle |
  |---|---|---|---|
  | none | 18108 | 19104 | 20692 / 20696 |
  | str | 123152 | 18612 | 20568 / 20472 |
  | int | 123028 | 18796 | 20756 / 20400 |

* `loadthrow.rex` (the re-review's `librr`/`librr2`): before, ours rc 134 "panic in a function that cannot unwind" after `loader throwing`; after, identical to the oracle: stdout `trapped 40 40.1 loadthrow;`, stderr `loader throwing`, `rr unloader ran`, rc 0. The forge's `loader/lthrow.rex` shows the same with a stderr line after `LoadLibrary` that neither side prints; `loader/lraise.rex` (a loader that only raises) was identical before and after: the extension continues, `LoadLibrary` answers 0 with the condition held.
* How the kept copy of a handle-carried value ends: each `NativeFrame` records in `kept` the handle-carried values it asked a copy of; `Interp::kept_holders` counts the frames in flight holding each; `pop_native_frame` releases the frame's, and the last release drops the copy unless `global_references` holds the value. `ReleaseGlobalReference` mirrors the oracle (`InterpreterInstance.cpp:678` re-adds), so a value once held by a global reference keeps its copy for the interpreter's lifetime, as a heap object held that way is never collected. The forge's `keptshort.rex` (a 5-byte string kept by a global reference across `gc('F')`) and `kept.rex` are identical to the oracle.

### Fix round 2: not fixed

* I2(a) swallowing `catch (...)` and I2(b) static `libgcc`: recorded in `phase-4-exclusions.txt`, owner none (Rust cannot raise a C++ exception); re-measured on `409221421`'s binary: `catchall.rex` rc 134 "fatal runtime error: Rust panics must be rethrown, aborting" against the oracle's `trapped 40 40.1 inner;caught;after-catch;`; `rethrow`, `cuw`, `after` over `libstatic` rc 134 with empty stderr against the oracle's rc 0.
* M4, for Task 6: the exit context's `Throw*` members are `unwinds` and their refusing stubs `abort_now`; unreachable today (`exit_context_interface()` aborts). Task 6, which fills the exit context, owns filling them, with `C-unwind` and the marker as the method and call contexts have them, and the exit handler's call boundary catching it.
* The Throw marker through a package hook is not under Miri: no test drives a hook that throws on a call context.

### Fix round 2: gates

`$S/gates/status.txt` at `7d25fa15d` (round 1's moved to `$S/gates-r1/`), HEAD the same at start and end, tree clean: G1 fmt 0; G2 clippy (empty target) 0; G3 0; G4 release 0, 2701 / 0 / 4, 0 Compiling lines, 617 of 617, load at start 10.92 10.47 8.51; G5 0; G6 debug 0, 2702 / 0 / 4, 617 of 617, load at start 10.04 8.98 8.48, after 2.12 7.01 8.05.

## Fix round 3

Base `7d25fa15d`. Re-check: `task-5-rereview.md`, "Fix round 2".

### Controls (prediction written before each run)

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| H1 | `panic!("planted")` at the top of `LoadLibrary`'s body | the re-review's `loadthrow.rex` with `RR_DTOR_STDERR=1`: rc 134, stderr `planted`, no `dtor` line; unplanted, the same run prints `dtor loadthrow` (the instrument sees destructors) | as predicted: planted rc 134, stderr `panicked at crates/rexx-api/src/ffi.rs:3090:17: planted`, 0 `dtor` lines; unplanted rc 0 with `dtor loader`, `dtor loadthrow`, identical to the oracle; restored |
| H2 | (the fix itself, prediction for the witnesses) handle-carried copies kept until the next collection | `keptots2.rex` identical to the oracle (`310A32`); `kept.rex`, `keptshort.rex` identical; `grow.rex` at 1M calls: `str` and `int` max RSS within 4 MB of `none` | **falsified in part**: `keptots2` (`310A32`), `kept`, `keptshort` identical, but `grow` `str` 123616 and `int` 123900 KB against `none` 18896: its loop allocates nothing on the heap (short text and small integers are carried in the handle), so no collection ever runs |
| H3 | handle-carried copies also pruned (as a collection prunes them) once they pass twice what the last pass left, and 4096 | `grow.rex` `str` and `int` within 4 MB of `none`; `keptots2`, `kept`, `keptshort` still identical | as predicted: `none` 18124, `str` 19780, `int` 19324 KB (oracle 20424 each); all three identical. H2 is the bound's control: without it, 123616 and 123900 KB |

### Fix round 3: commits

* `facf431aa` `LoadLibrary`/`RegisterLibrary` bodies run under `loading()`: any panic aborts; only the Throw marker unwinds
* `6fa3e4eb7` a handle-carried value's kept `CSTRING` lives until a collection (not the call's end); also pruned, as a collection prunes, once such copies pass twice what the last pass left, and 4096
* `7eeb77846` exclusions: the true reason (a C++ shim would add a C++ compiler to the build) and the re-review's extension moved to `task-5-forge/rereview/` (`rr.cpp`, `rrc.c`, `build.sh`, `probes/`); third spec note

### Fix round 3: numbers

* `keptots2.rex` (the re-review's; now `task-5-forge/rereview/probes/`): at `7d25fa15d` ours read `9FB72EB04B7F` (freed memory) where the oracle reads `310A32`; at `6fa3e4eb7` identical.
* `grow.rex` at 1M calls, max RSS KB (`$S/r2/grow.sh`): `none` 18124, `str` 19780, `int` 19324 (oracle 20424 each). With collection alone as the trigger (H2): `str` 123616, `int` 123900.
* `kept.rex`, `keptshort.rex`: identical throughout. The re-review's `loadthrow.rex`, `rethrow.rex` with this round's `build.sh`: identical; `catchall.rex` and the static-libgcc `rethrow`/`cuw`/`after` reproduce the two recorded divergences (rc 134; the static ones with empty stderr).
