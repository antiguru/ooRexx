# Phase 9a Task 9 review

Range `5d12347c0..ccd4d4f02`. Reviewer: opus, read-only on the tree. One build of an archived copy
of `ccd4d4f02` (outside the worktree, env-toggled probe, both directories removed afterwards).

## Verdicts

* **Spec: PASS.** Step 1 and Step 2 as briefed; Step 3 not judged as missing (its premise was wrong,
  the question is with Moritz), its diagnosis checked below and true.
* **Quality: PASS WITH FIXES.** One Important (the answer temp, the controller's open question), four
  Minor prose defects, one of them a doc block the insertion orphaned.

## Spec compliance

* Step 1. `relloop.cpp` matches the queued note's probe. The test builds it with exactly
  `c++ -shared -fPIC -O1 -I<repo>/api -I<repo>/api/platform/unix` and panics if `c++` does not run;
  runs 200,000 and 2,000,000 passes under `memcap 2G`; asserts `long <= 2 * short`; also asserts
  stdout `done`, empty stderr, rc 0. Red before (430344 vs 59572 KB, report), green after.
* Step 2. `Table::release` unroots the one handle; `phase-4-exclusions.txt` M2 row reads DELIVERED
  with figures and the test name.
* Step 3. See focus 4.

## Focus findings

### 1. Release semantics against the oracle

`NativeActivation::removeLocalReference` (`interpreter/execution/NativeActivation.cpp:1219`) clears
`firstSavedObject` or removes the object from the identity `saveList`; `ReleaseLocalReference`
(`interpreter/api/ThreadContextStubs.cpp:145`) acts on the thread's current native activation.
The crate matches on what can be observed:

* Released handle leaves `roots()` and nothing else does (`handles.rs` `release`); double release
  and a handle not in `entries` are no-ops, and neither is added to `released`. The oracle's
  removal on an object it does not hold is a no-op too.
* A released handle resolves only while `heap.peek` (generation-checked, `rexx-core` `Heap::resolve`)
  finds it, so a collected object answers `None` and the callback refuses loudly; no path hands a
  swept `ObjRef` to the interpreter. In Rust terms there is no use after free; the C side reading a
  `StringData`/`BufferData` pointer after release and a collection reads freed memory exactly as on
  the oracle (outside the contract on both).
* Registering again after release moves the handle back to `entries`.
* One divergence, collection timing only, no finding: the oracle's `firstSavedObject` slot means an
  object registered twice where the first registration took that slot is still in `saveList` after
  one release (`createLocalReference`, `:1185`-`:1209`), so it stays rooted there; the crate's single
  entry per handle unroots it. Where the second registration also went to `saveList` the oracle
  unroots too (identity-table `put` replaces).
* `register_local`'s temp pop takes only the top temp above the call's floor and only when it is the
  object just registered; every call site registers the object a Host constructor or a send just
  produced, so the popped temp is that constructor's. I found no site that relies on the popped temp
  after registering.
* Release loops over answers that are not `String()`'s: I built `RevSendLoop(target, name, n)`
  (`SendMessage0` then `ReleaseLocalReference`) against the archived copy and ran it under
  `memcap 2G /usr/bin/time`: builtin `REVERSE` 29256/29180/29260 KiB at 200,000/2,000,000/4,000,000;
  a Rexx method returning `copies('y', 60)` 32232/33292 KiB at 200,000/2,000,000. Flat, so the fix
  is not specific to `String()`.

### 2. The `end_library_*` answer temp: Important

`rust/crates/rexx-exec/src/dispatch/library.rs:127` and `:216` (`root_native_answer`, `:692`).

**Is it the answer's only root across the window?** Yes. Control: with an env toggle that skips the
push and a forced `collect_now()` right after `pop_native_frame` in `end_library_routine`, the answer
is swept and the run panics `a live value` at `dispatch.rs:1712`; with the push, the same forced
collection leaves it alive at every one of the probe's routine calls. So the push is live, and
without it a collection in that window is a panic, not a refusal.

**Is there a collection point in the window?** None found. With the push skipped and no forced
collection, `run_program_collect_every_alloc` over a program exercising a function call, `CALL` then
`say result`, a `size_t` answer, concatenation of two answers, `.array~of` of a routine answer and a
method answer, `trace r` over both, `Routine~call` on a `::ROUTINE ... EXTERNAL`, and a native method
(`::METHOD ... EXTERNAL`) answering a fresh heap string: stdout, stderr and rc identical to the run
with the push. On the success path `settle_native_call` answers `Ok(value)` without allocating and
the callers store it in a register first.

**Verdict.** Unwitnessed as shipped, so it cannot stay as it is. The oracle roots the same value at
the same point (`resultObj = result` at `NativeActivation.cpp:1353`, before `popStackFrame` at
`:1363`), and deleting it leaves "nothing in the window allocates" as an unenforced invariant whose
failure is a panic. Recommended: **witness it** by having stress mode (`stress_collect`, already
threaded into `native_call`) collect once after `pop_native_frame` in both `end_library_*`; the
existing stress tests (`release_local_reference.rs`'s, `collect_stress`) then go red without the
push, which is the witness. If the controller prefers no new stress point: **delete it** (both calls
and the function), since no reachable path needs it today; the release fix does not depend on it
(the memory test's figures do not involve it).

### 3. The memory test

* Measures the interpreter: `/usr/bin/time -f %M -o` is the innermost wrapper and `rexx-run` runs
  the program in-process (`src/bin/rexx-run.rs`, no child spawned); `memcap` is a cgroup cap
  (`~/.local/bin/memcap`), not a ulimit, so it does not change RSS.
* Can fail: 430344 > 2 x 59572 before.
* Load: no timing assertion. Wall at load 2.8 on the archived copy: 0.37 s for 2,000,000 passes; the
  bound is `timeout 120`, and `RelOne(1000000)` in-process runs under a 60 s deadline. Robust at load
  20-36.
* The ratio (1.002 after) leaves a wide margin under 2.

### 4. The Package diagnosis

* "Every package object is a global root": true. The only constructor is
  `Interp::package_object` (`rust/crates/rexx-exec/src/environment/identities.rs:120`), which
  `add_global`s it at `:143`; `object_packages.insert` appears nowhere else
  (`grep -rn "package_root_key\|object_packages.insert" rust/crates/rexx-exec/src`), and
  `copy_package` (`:1032`) ends in `self.package_object(Package::Program(copy))`.
* TESTRECLAIM01 loops until collection with no bound: `ootest/ooRexx/base/class/Package.testGroup:172`
  wraps the package in a `WeakReference`, `:179`-`:184` is `loop until package~value == .nil` over
  100 `copies('x', 1000000)` per pass.
* `PackageManager::addToRequiresCache` at `interpreter/package/PackageManager.cpp:921` does cache a
  `WeakReference` (`:927`). Cite correct.
* Edits: the plan's Step 3 correction, `phase-9a-gate.md:121`'s line and the queued note's
  paragraph are true and minimal; none claims a fix. I did not rerun the group (no suite reruns); the
  SIGTERM-halt account of "summary then no exit" rests on the report's two bounded runs.

### 5. `unsafe` and grants

`git diff 5d12347c0 ccd4d4f02 -- rust | grep '^+.*unsafe'`: no match. `unsafe_sites.rs` untouched.
The `rexx-api` changes are a `Table` field pair, two `Host` default methods and call-site renames.

### 6. Prose

`git diff 5d12347c0 ccd4d4f02 | grep -nP '^\+.*(—| -- )'`: only the report's clippy command line.
No set sizes in comments or records; no future mechanisms. Defects below.

## Findings

### Critical

None.

### Important

**I1. The answer temp is unwitnessed** (`dispatch/library.rs:127`, `:216`, `:692`). Section 2.
Failure scenario if left: a later change that allocates between `pop_native_frame` and the caller's
store (a trace line, a condition object, a guard wake-up that allocates) collects a fresh native
answer with nothing to say whether the push still matters; a deletion by the next cleanup is equally
silent until it panics. Fix: the stress-mode collection after the pop (witness), or delete.

### Minor

**M1. Orphaned doc block** (`dispatch/library.rs:1333`-`:1340`). `lives` was inserted between
`condition_of`'s doc comment and `condition_of`, so `lives`'s rustdoc now begins "The condition an
extension raised, whose number is the major and the minor packed as ..." and `condition_of` has no
doc. Move `lives` (with its own line) above the `condition_of` block.

**M2. The stress test's doc cites the wrong ground** (`tests/release_local_reference.rs:200`-`:204`).
It says the rc 120 for `RelTwice` is what "collection timing licenses"; the exclusions row it points
to and the ledger ruling (progress.md, "Task 9: Ruling") rest it on "a use after release is outside
the API contract", and the report's concern 3 says the timing licence does not cover an rc/stderr
difference. Say what the row says.

**M3. `Host::register_local` doc is false after a release** (`rexx-api/src/values.rs:677`-`:678`):
"roots it for the rest of the call" does not hold once `release_local` runs. "roots it until it is
released or the call ends".

**M4. `NativeFrame::temps_floor` doc overstates** (`rexx-exec/src/lib.rs:1972`-`:1973`): a temp pushed
since the call began and registered is rooted by `locals` alone only when it was the top temp at
registration (`pop_temp_above`); one that was not stays a temp. State the condition or say "the
floor below which `register_local` pops nothing".

## Checks run

Archived `ccd4d4f02` (`rust`, `api`, `interpreter`) to a directory outside the worktree; added an env
toggle around `root_native_answer`, a forced post-pop collection, `revans.cpp` and two probe tests;
`CARGO_TARGET_DIR=/tmp/claude-1000/p9/t9-review-target CARGO_INCREMENTAL=0 memcap 8G cargo test -j 2
--release -p rexx-exec --test release_local_reference --no-run`, twice (status 0); probe runs as
quoted above, each under `memcap 8G timeout`. Both directories removed with literal `rm -rf` paths.
The worktree, index and HEAD were not touched by this review.
