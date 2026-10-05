# Task 20 review: lent island memory

Reviewed range `2fa650384..55dd10af8` (HEAD `55dd10af8`). Reviewer: s4-t20-rev.

Reviewer forge and runner, kept in scratch and not committed:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/p6-scratch/t20rev/work/`.
- `forge/lent.cpp` defines these routines:
  - HOLDBUF: rexx-api's `hold_buffer` transcribed to C++.
  - HOLDBUFLATE: HOLDBUF plus a delay.
  - HOLDSTR: a `CSTRING` held across a wait.
  - FINISHSHORT and FINISHPEEK: `StringData` taken before `FinishBufferString`.
- The programs are `forge/*.rex`.
- `runner.sh` runs each program from a fresh `mktemp -d` directory. The oracle runs under
  `ulimit -v 1048576` and `timeout -k 5 20`. The Rust side is `rexx-run` built from HEAD.

## Verdicts

- **Spec compliance: PASS**, with corrections to the record (F3, F4).
  - Spec 2.5's per-kind rule holds on every completion path. A lent buffer is not freed or
    reallocated in place while its call is in flight. A held kept string is not pruned.
  - The one exception is the abandon path (F5). Today nothing can observe it, because every failure
    that abandons a call also ends the run.
  - The licensed divergence is recorded, but the row is misworded and misfiled.
- **Task quality: CHANGES REQUIRED.** One Important finding (F1): a Task 20 test fails 15 times in
  60 when its module runs on parallel test threads. On the failing schedule, its native reads past
  the end of an allocation.

## Explicit verdicts

### 1. Concern 1: what the oracle does after a reallocation

- **The implementer's reading of the C++ is correct.**
  - `MutableBuffer::ensureCapacity` (`interpreter/classes/MutableBufferClass.cpp:246`) does
    `setField(data, data->expand(bufferLength))`.
  - `BufferClass::expand` (`BufferClass.cpp:102-113`) allocates a new buffer object and copies
    the data into it.
  - The old data object is then unreferenced, so it lives until a collection reclaims it.
  - C writes through the old address therefore never reach the buffer, on either interpreter.
- **The spec sentence is also half right.**
  - Once a collection runs during the call, the oracle's old storage is reclaimed. From then on, the
    call's reads and writes touch freed memory.
  - **Measured.** `growgc.rex` grows the buffer and then allocates until collections run. The call
    then read garbage on the oracle in 30 runs of 30 (first bytes `\x80\xb4…`, not `abcd…`).
  - **Ours.** Rust read the original 26 bytes in the 7 runs of 10 that took the call-first
    schedule.
- **The report's premise is false.** It says the oracle's natives run holding the kernel lock and
  so cannot take this interleaving. They do not hold it: `NativeActivation::run` releases access
  around the call (`NativeActivation.cpp:1304-1306`), and so does `callNativeRoutine` (`:1420-1422`).
  **So the oracle can run the scenario. Measured, oracle, 30 runs of 30 each:**
  - `grow.rex`: `abcdefghijklmnopqrstuvwxyz` / `abcd 5026` / `grown`, rc 0.
  - `overlay.rex`: `abXYefghijklmnopqrstuvwxyz` / `ZZXY 26` / `overlaid`, rc 0.

  Both are byte-identical to what the two Rust tests assert.
- **Ruling: is it a divergence at all?** Not in any behaviour the oracle defines.
  - With no collection between the growth and the call's end, the two interpreters agree: the call
    reads the old bytes, and Rexx does not see its writes.
  - They differ only after a collection. There, the oracle's call reads and writes freed memory, and
    ours reads stable old storage.
  - So the licensed divergence is the **lifetime of the old storage**, not "writes are not seen".
- **The exclusions row is not true as worded.** See F3:
  - Its title asserts as a deviation ("C WRITES ... ARE NOT SEEN") a behaviour that both
    interpreters share.
  - It is not in DEVIATIONS. It sits at `phase-4-exclusions.txt:5628`, under "PHASE 8 CLOSES,
    2026-09-28: WHAT ITS CLOSE FOUND" (`:5564`). That header says each entry was measured against
    the oracle at 2ae06085c with probes under `task-9-probes/`, and neither is true of this row.
  - The body is accurate. It already says the oracle does not see the writes either.
- **Replacement for spec lines 200-202.** Replace "C writes that land in the old storage after such
  a reallocation are not seen by Rexx, a licensed divergence (in the oracle they are writes into
  freed memory)." with:

  > C reads and writes through the old address after such a reallocation do not reach the buffer,
  > as in the oracle, where growth gives the buffer a new data object
  > (`MutableBuffer::ensureCapacity`, `interpreter/classes/MutableBufferClass.cpp:246`) and leaves
  > the old one as garbage. The licensed divergence is that storage's lifetime: the oracle's next
  > collection reclaims it under the call, after which the call's accesses touch freed memory;
  > here it stays valid until the lending call completes.

### 2. FinishBufferString

- **Correct, and oracle-like for every byte up to the finished length.**
  - The oracle's `RexxString::finish` sets only `length` (`StringClass.hpp:541`), and
    `StringData` and `BufferStringData` answer the same storage.
  - The rewrite keeps the address `StringData` answered valid. Before the fix that address was freed
    under the call. The removal branch is reached only when no copy is kept: `written`
    (`rexx-api/src/values.rs:487-491`) truncates to at most the length the string was made at, and
    the kept copy is at least that length plus its NUL.
- **The forge is real and reproduces.** I built it with its own `build.sh`. It answered `hello`, rc 0,
  empty stderr, on the oracle (3 of 3) and on HEAD's `rexx-run` (3 of 3).
- **One introduced difference: the terminator (F4, Minor).**
  - `surface.rs:212` writes a NUL at the finished length. The oracle writes none.
  - `FINISHSHORT('hello', 10)` writes `helloxxxxx`, finishes at 5, and reads the early address as
    ASCIIZ. The oracle answers `helloxxxxx` (3 of 3); ours answers `hello` (3 of 3).
  - Mutant R4 deletes that line and stays green, so no test witnesses the line.
- **Pre-existing, not introduced:** a read through the early address *before* the finish. Oracle
  `FINISHPEEK('hello')` gives `68656C6C6F`; ours gives `0000000000`, because the kept copy is a
  snapshot. This is noted, not a finding against Task 20.

### 3. "Kept strings needed no new code" under P52 interleaving

**Holds.**
- **Handle-carried values.**
  - `kept_c_string` counts the holder on the innermost frame of the running activity. A callback
    runs with the call's own activity switched in.
  - `drop_loose_kept_strings` retains any copy whose object has a holder, whichever activity's call
    triggers the prune.
  - The implementer's 5000-call prune test covers this, and their M1 is red.
- **Heap strings.** I checked the case the report does not test: a fresh string value.
  - A `MutableBuffer` passed as `CSTRING` converts through `string_value` to a new heap string,
    which is held only as a temp of the calling activity.
  - A temporary test held that value's kept copy while another activity allocated until
    collections ran (`collections > 0` asserted). It was green 5 of 5, and green under valgrind
    with 0 errors.
  - I then mutated the collection so it dropped every heap copy. The same test went red, and
    valgrind reported 41 invalid reads from `hold_c_string`, so the instrument was live.
  - The temporary test and the mutation were both reverted.
- The forge `freshstr.rex` agrees: "same" on the oracle (30 of 30) and on `rexx-run` (10 of 10).

### 4. Lent-buffer leaks and release

**Released on every path where the call has ended. Not held on abandon (F5).**
- **Normal completion and a raised condition.** `end_library_method` and `end_library_routine` both
  call `pop_native_frame`, which drains `frame.lent` and releases each buffer
  (`dispatch/library.rs:708-712`). Hooks and command handlers pop the same way.
  - Mutant R5 (no release at pop) is red in `a_lent_buffers_old_storage_lives_until_its_last_call_ends`
    and in nothing else. The interleaving tests cannot see a leak.
- **Frame reuse.** `native_spares` frames come back with `lent` drained.
- **Panic.** A pool-thread panic is posted after the native has unwound, and resumed on the holder
  with `resume_unwind` (`scheduler.rs:1207`). No frame is popped. The storage goes only if the
  `Interp` drops, and by then that call is over. Other calls in flight at that moment are P53's.
- **Abandon (F5).** `abandon_native_call` pops the frame while the native may still be running on its
  pool thread. That releases the lent buffers, freeing retired storage, and the kept-string holders.
  The test-only `fail_native_wait` path shows the call does keep running after an abandon
  (`scheduler/tests/callbacks.rs:340-405`).

## Findings

### F1 (Important): the grown-buffer test is schedule-dependent, and on one schedule its native reads out of bounds

- **Location:**
  - `rust/crates/rexx-exec/src/scheduler/tests/lent.rs:109`
    (`a_buffer_grown_under_a_call_keeps_the_storage_the_call_holds`). The in-place test at `:130`
    has the same shape.
  - `rust/crates/rexx-api/src/load.rs:918-920` (`hold_buffer`).
- **Failure scenario 1: the grow runs first.**
  - Nothing orders the other activity's `append` after the native's `MutableBufferData`. The native
    asks for the address from the pool thread, by a callback that must take the baton. The holder
    may already have run `grow`.
  - The native then holds the grown storage and reads 5026 bytes, and Rexx sees its `ZZ`.
  - The test fails with left `"abcdefghijklmnopqrstuvwxyzyyyy…\nZZcd 5026\ngrown\n"`.
- **Failure scenario 2: the grow runs between the two callbacks.**
  - `MutableBufferData` and `MutableBufferLength` are separate callbacks. When the grow lands between
    them, the native holds the old address with the new length.
  - It then reads 5026 bytes from the old, smaller storage: a heap over-read in test code (UB). The failure
    shows heap garbage after the 26 letters.
- **Demonstrated by running** the HEAD binary:
  - 5 tests in `scheduler::tests::lent::` on default parallel test threads: **15 of 60 runs red**,
    all in this test, all scenario 1.
  - The same module with `--test-threads=1`: 1 of 40 red, scenario 2. The left value begins
    `abcdefghijklmnopqrstuvwxyz\0\0…` followed by what look like heap pointers.
  - `rexx-run` with the transcribed forge (`grow.rex`): scenario 1 in 3 of 10 runs.
  - The full workspace run was green once (below). The flake is real but rare there.
- **Fix:** add a rendezvous, so the native holds its address before the growth.
  - The native signals (for example, writes a second file) after `MutableBufferData` and
    `MutableBufferLength`.
  - The `grow` and `overlay` methods poll for that signal before changing the buffer.
- Both schedules are legal; the oracle simply took call-first 30 of 30. This is a test defect, not
  an implementation defect.

### F2 (Minor): the report's oracle claim is false, so the row went without an oracle witness that exists

- **Location:** `task-20-report.md`, Checks, "the oracle's natives run holding the kernel lock".
- **Failure scenario:** the record says the interleaving cannot be checked against the oracle.
  - It can: `NativeActivation.cpp:1304-1306` and `:1420-1422` release access around the native.
  - The reviewer's `grow.rex` and `overlay.rex` give the Rust tests' expected outputs on the oracle,
    30 of 30 each.
- **Demonstrated by running.**
- **Fix:**
  - Correct the report sentence.
  - Commit a HOLDBUF forge (as was done for `finish`) as the oracle witness for the row, with run
    counts.
  - A version with F1's rendezvous makes it deterministic on both sides.

### F3 (Minor): the divergence row names a shared behaviour as the deviation, and is misfiled

- **Location:** `docs/superpowers/plans/phase-4-exclusions.txt:5628-5642`.
- **Failure scenario:** a reader takes "C writes ... are not seen" for a deviation, but the oracle
  does not see them either (verdict 1).
  - The real deviation is the lifetime of the old storage after a collection. Oracle: garbage, 30 of
    30, `growgc.rex`.
  - The row sits under the Phase 8 close header. That header asserts an oracle measurement at
    2ae06085c and probes this row does not have.
  - The report says the row is in DEVIATIONS (`:713`), which is false.
- **Demonstrated by running** the oracle side; the placement was checked by reading.
- **Fix:**
  - Retitle the row along the lines of: A NATIVE CALL'S OLD MUTABLEBUFFER STORAGE OUTLIVES A
    COLLECTION.
  - Move it to its own dated Phase 6 heading, or into DEVIATIONS if that section takes such rows.
  - Cite the oracle forge from F2.
  - Apply the spec replacement in verdict 1.

### F4 (Minor): FinishBufferString writes a terminator the oracle does not, and no test witnesses it

- **Location:** `rust/crates/rexx-exec/src/dispatch/library/surface.rs:212`.
- **Failure scenario:** an extension takes `StringData` before writing and finishing a shorter
  string, then reads the early pointer as ASCIIZ. The oracle reads up to the made length's NUL,
  `helloxxxxx`. Ours stops at the finished length, `hello`.
- **Demonstrated by running:** `finishshort.rex`, 3 of 3 on each side. Mutant R4, which deletes the
  line, is green in `rexx-exec` and `rexx-core`.
- **Fix:** copy the whole written buffer, as made, into the kept copy and leave the made-length NUL.
  - `written()` truncates first, so the surface needs the untruncated bytes.
  - Or keep today's behaviour, record it as a divergence, and add a test that pins it.
  - The case is unusual (`StringData` on an unfinished buffer string), hence Minor.

### F5 (Minor): an abandon releases lent storage and kept holders while the abandoned native may still run

- **Location:** `rust/crates/rexx-exec/src/dispatch/library.rs:368-380` (`abandon_native_call`)
  reaches the release at `:708-712`.
- **Failure scenario:** a failure ends a native park while the native is still running on a pool
  thread.
  - The frame pops, the buffer's retired storage is freed, and holder counts drop.
  - The native's later reads and writes through the old address hit freed memory.
  - A held handle-carried kept string becomes prunable.
  - This is spec 2.5's rule broken on one path.
- **Not demonstrated as a use-after-free.** Every production failure that abandons a call also ends
  the run, as Task 17's review found. `abandon.rex` (HOLDBUFLATE plus a loud refusal in the growing
  activity) ends at rc 120 with empty stdout 5 of 5 before the native resumes.
  - The oracle prints the 26 letters, then `after`, rc 0 (5 of 5). That is Task 17's known
    loud-path divergence, not this task's.
  - Only the test-only `fail_native_wait` injection continues past an abandon.
- **Fix:**
  - Either move the frame's `lent` list and `kept` set onto the abandoned call's record and release
    them when its completion is drained.
  - Or record next to P53 that the guarantee depends on "every abandon ends the run".
  - The cost falls on embedding, together with P53's leak.

## Mutation round (reviewer's, in place)

Each mutant was applied by exact-string replacement with one occurrence asserted, and restored from a
saved copy with equality asserted. Each run built with `cargo test --release -p rexx-exec -p rexx-core
--no-run` outside memcap, then ran `memcap 8G cargo test ... --no-fail-fast`. M0 is unmutated: 69
result lines, 1939 passed, 0 failed.

| Mutant | Result |
|---|---|
| R1 `lent_buffer` lends to the outermost native frame (`first_mut`) | green. Safe over-retention: the outermost frame ends last, so storage lives longer, never shorter. No finding. |
| R3 lent growth drops the old storage instead of retiring it | red: `each_growth_path_moves_lent_bytes`, `growth_of_lent_bytes_keeps_the_old_storage_until_the_last_release`, `a_lent_buffers_old_storage_lives_until_its_last_call_ends`, `a_buffer_grown_under_a_call_keeps_the_storage_the_call_holds` |
| R4 FinishBufferString writes no terminator | green (F4) |
| R5 `pop_native_frame` releases no lent buffer | red: `a_lent_buffers_old_storage_lives_until_its_last_call_ends` only |
| collection drops every heap kept copy (verdict 3) | red: the reviewer's fresh-string test, `a_held_heap_string_survives_another_activitys_collection`, and once the F1 flake. Valgrind: 41 invalid reads. |

`git diff --stat` after the round showed only the lead's `progress.md`.

## P50 and unsafe

- **P50 holds.** `lend` and `release` run only on the baton: `lent_buffer` runs inside surface
  callbacks, which take the baton, and `pop_native_frame` runs on the holder. The retired storages
  are freed on the baton. The pool thread touches only raw bytes. The new code adds no `Rc` and no
  interior access.
- **Unsafe.** The only new `unsafe` is in the three `doc(hidden)` test natives in
  `rexx-api/src/load.rs`. `hold_c_string` and `finished_in_place` are sound.
- **`hold_buffer` is not sound under every schedule (F1 scenario 2).** Its SAFETY comment says
  "at least the length answered with it", but the length is answered by a second callback that a
  growth can follow.
- **Design note, no finding.** The in-place case races by design: the holder writes through `&mut
  [u8]` while C reads and writes the same bytes on the pool thread. That is a data race in Rust's
  model. Spec 2.5 chooses this sharing, and the oracle shares it in C++.

## Checks run

All from `rust/`, `CARGO_TARGET_DIR=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/p6-scratch/t20rev/target`,
on HEAD's tree with no edits present:
- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release --no-fail-fast`: exit 0, 142 `test result` lines, 2985 passed, 0 failed. No `Compiling`
  line appeared in the run, so it ran the binaries already built. This is one green run; F1's flake
  did not fire in it.
- Oracle runs (fresh directory each, `ulimit -v 1048576`):
  - 30 each: `grow.rex`, `overlay.rex`, `growgc.rex`, `freshstr.rex`.
  - 3 each: `finish.rex` (the implementer's), `finishshort.rex`, `finishpeek.rex`.
  - 5: `abandon.rex`.
- `rexx-run` runs: 10 each of `grow`, `overlay`, `growgc` and `freshstr`, 5 of `heapstr`, 3 each of
  `finish`, `finishshort` and `finishpeek`, and 5 of `abandon`.
